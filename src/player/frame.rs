//! Display-ready video frames: raw YUV planes plus the metadata the GPU
//! shader needs to convert them (layout, bit depth, matrix, range).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use ffmpeg_next as ffmpeg;
use ffmpeg::format::Pixel;
use ffmpeg::software::scaling;
use ffmpeg::util::color;

/// Set at startup when the GPU can sample 16-bit normalized textures
/// (`TEXTURE_FORMAT_16BIT_NORM`); otherwise >8-bit video is converted to 8-bit.
pub static GPU_16BIT: AtomicBool = AtomicBool::new(false);

static NEXT_FRAME_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlaneLayout {
    /// Y, U, V in three planes (any chroma subsampling).
    Planar,
    /// Y plane + interleaved UV plane (NV12 / P010).
    SemiPlanar,
    /// Already RGB (fallback for unusual formats).
    Rgba,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Matrix {
    Bt601,
    Bt709,
    Bt2020,
}

impl Matrix {
    /// Rows of the Y'CbCr → R'G'B' matrix, applied to (Y, Cb−½, Cr−½).
    pub fn rows(self) -> [[f32; 3]; 3] {
        let (kr, kb) = match self {
            Matrix::Bt601 => (0.299, 0.114),
            Matrix::Bt709 => (0.2126, 0.0722),
            Matrix::Bt2020 => (0.2627, 0.0593),
        };
        let kg = 1.0 - kr - kb;
        [
            [1.0, 0.0, 2.0 * (1.0 - kr)],
            [1.0, -2.0 * kb * (1.0 - kb) / kg, -2.0 * kr * (1.0 - kr) / kg],
            [1.0, 2.0 * (1.0 - kb), 0.0],
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlaneInfo {
    /// Size in texels.
    pub width: u32,
    pub height: u32,
    /// Components per texel (1 = Y/U/V, 2 = interleaved UV, 4 = RGBA).
    pub components: u32,
}

pub struct VideoFrame {
    /// Unique id; lets the renderer skip re-uploading the same frame.
    pub id: u64,
    pub serial: u64,
    /// Presentation time in seconds (relative to media start).
    pub pts: f64,
    pub duration: f64,
    pub width: usize,
    pub height: usize,
    pub sample_aspect: f32,
    pub layout: PlaneLayout,
    /// Samples are 16-bit (otherwise 8-bit).
    pub high_depth: bool,
    /// Effective bit depth of the sample values (8, 10, 12, 16).
    pub bits: u32,
    /// Factor turning a sampled (normalized) value into the `bits`-deep
    /// normalized value: 10-bit stored in the low bits of u16 needs 65535/1023.
    pub value_scale: f32,
    pub matrix: Matrix,
    pub full_range: bool,
    pub planes: Vec<PlaneInfo>,
    frame: ffmpeg::frame::Video,
}

// SAFETY: the frame is immutable once constructed; only read-only access to
// its plane data is exposed.
unsafe impl Sync for VideoFrame {}

impl VideoFrame {
    /// Display aspect ratio (width / height), taking non-square pixels into account.
    pub fn display_aspect(&self) -> f32 {
        self.width as f32 * self.sample_aspect / self.height as f32
    }

    /// Bytes and stride (bytes per row) of plane `i`.
    pub fn plane_data(&self, i: usize) -> (&[u8], usize) {
        (self.frame.data(i), self.frame.stride(i))
    }

    pub fn bytes_per_sample(&self) -> u32 {
        if self.high_depth { 2 } else { 1 }
    }
}

/// How a decoded pixel format reaches the GPU.
enum Route {
    Direct { layout: PlaneLayout, high_depth: bool, msb_aligned: bool },
    Convert(Pixel),
}

fn route(fmt: Pixel, gpu16: bool) -> Route {
    use Pixel::*;
    let direct = |layout, high_depth, msb_aligned| Route::Direct { layout, high_depth, msb_aligned };
    match fmt {
        YUV420P | YUV422P | YUV444P | YUV440P | YUVJ420P | YUVJ422P | YUVJ444P | YUVJ440P => {
            direct(PlaneLayout::Planar, false, false)
        }
        NV12 => direct(PlaneLayout::SemiPlanar, false, false),
        YUV420P9LE | YUV420P10LE | YUV420P12LE | YUV420P14LE | YUV420P16LE | YUV422P9LE | YUV422P10LE
        | YUV422P12LE | YUV422P14LE | YUV422P16LE | YUV444P9LE | YUV444P10LE | YUV444P12LE
        | YUV444P14LE | YUV444P16LE => {
            if gpu16 {
                direct(PlaneLayout::Planar, true, false)
            } else {
                Route::Convert(YUV420P)
            }
        }
        P010LE | P012LE | P016LE => {
            if gpu16 {
                direct(PlaneLayout::SemiPlanar, true, true)
            } else {
                Route::Convert(NV12)
            }
        }
        _ => Route::Convert(RGBA),
    }
}

/// Reusable converter for formats the shader can't take directly.
#[derive(Default)]
pub(super) struct FrameConverter {
    scaler: Option<(scaling::Context, Pixel, Pixel, u32, u32)>,
}

impl FrameConverter {
    /// Wraps a decoded frame for display (taking ownership: no copy), converting
    /// its pixel format if needed.
    pub fn make(
        &mut self,
        decoded: ffmpeg::frame::Video,
        serial: u64,
        pts: f64,
        duration: f64,
        stream_sar: Option<f64>,
    ) -> Option<VideoFrame> {
        let (fmt, w, h) = (decoded.format(), decoded.width(), decoded.height());
        if w == 0 || h == 0 {
            return None;
        }
        let gpu16 = GPU_16BIT.load(Ordering::Relaxed);

        // Colour metadata comes from the *decoded* frame, before any conversion.
        let matrix = match decoded.color_space() {
            color::Space::BT709 => Matrix::Bt709,
            color::Space::BT470BG | color::Space::SMPTE170M | color::Space::SMPTE240M | color::Space::FCC => {
                Matrix::Bt601
            }
            color::Space::BT2020NCL | color::Space::BT2020CL => Matrix::Bt2020,
            // Unspecified: same heuristic as mpv/ffplay (HD → 709, SD → 601).
            _ => {
                if w >= 1280 || h > 576 {
                    Matrix::Bt709
                } else {
                    Matrix::Bt601
                }
            }
        };
        let full_range = decoded.color_range() == color::Range::JPEG
            || matches!(fmt, Pixel::YUVJ420P | Pixel::YUVJ422P | Pixel::YUVJ444P | Pixel::YUVJ440P);

        let (frame, layout, high_depth, msb_aligned) = match route(fmt, gpu16) {
            Route::Direct { layout, high_depth, msb_aligned } => (decoded, layout, high_depth, msb_aligned),
            Route::Convert(target) => {
                let out = self.convert(&decoded, target)?;
                let (layout, _, _) = match route(target, false) {
                    Route::Direct { layout, high_depth, msb_aligned } => (layout, high_depth, msb_aligned),
                    Route::Convert(_) => (PlaneLayout::Rgba, false, false),
                };
                (out, layout, false, false)
            }
        };

        // SAFETY: av_pix_fmt_desc_get returns a static descriptor (or null).
        let (cw_shift, ch_shift, depth) = unsafe {
            let d = ffmpeg::ffi::av_pix_fmt_desc_get(frame.format().into());
            if d.is_null() {
                (1, 1, 8)
            } else {
                ((*d).log2_chroma_w as u32, (*d).log2_chroma_h as u32, (*d).comp[0].depth as u32)
            }
        };
        let (w32, h32) = (w, h);
        let cw = (w32 + (1 << cw_shift) - 1) >> cw_shift;
        let chh = (h32 + (1 << ch_shift) - 1) >> ch_shift;
        let planes = match layout {
            PlaneLayout::Planar => vec![
                PlaneInfo { width: w32, height: h32, components: 1 },
                PlaneInfo { width: cw, height: chh, components: 1 },
                PlaneInfo { width: cw, height: chh, components: 1 },
            ],
            PlaneLayout::SemiPlanar => vec![
                PlaneInfo { width: w32, height: h32, components: 1 },
                PlaneInfo { width: cw, height: chh, components: 2 },
            ],
            PlaneLayout::Rgba => vec![PlaneInfo { width: w32, height: h32, components: 4 }],
        };

        let bits = if high_depth { depth } else { 8 };
        let value_scale = if high_depth && !msb_aligned {
            65535.0 / ((1u32 << bits) - 1) as f32
        } else {
            1.0
        };

        let frame_sar = frame.aspect_ratio();
        let sar = stream_sar.unwrap_or(if frame_sar.numerator() > 0 && frame_sar.denominator() > 0 {
            f64::from(frame_sar)
        } else {
            1.0
        });

        Some(VideoFrame {
            id: NEXT_FRAME_ID.fetch_add(1, Ordering::Relaxed),
            serial,
            pts,
            duration,
            width: w as usize,
            height: h as usize,
            sample_aspect: sar as f32,
            layout,
            high_depth,
            bits,
            value_scale,
            matrix,
            full_range: full_range || layout == PlaneLayout::Rgba,
            planes,
            frame,
        })
    }

    fn convert(&mut self, src: &ffmpeg::frame::Video, target: Pixel) -> Option<ffmpeg::frame::Video> {
        let (fmt, w, h) = (src.format(), src.width(), src.height());
        let fresh = !matches!(&self.scaler, Some((_, f, t, sw, sh)) if *f == fmt && *t == target && *sw == w && *sh == h);
        if fresh {
            match scaling::Context::get(fmt, w, h, target, w, h, scaling::Flags::BILINEAR) {
                Ok(s) => self.scaler = Some((s, fmt, target, w, h)),
                Err(e) => {
                    log::error!("swscale init failed for {fmt:?} -> {target:?} {w}x{h}: {e}");
                    return None;
                }
            }
            log::info!("converting {fmt:?} -> {target:?} on the CPU");
        }
        let (scaler, ..) = self.scaler.as_mut()?;
        let mut out = ffmpeg::frame::Video::empty();
        if let Err(e) = scaler.run(src, &mut out) {
            log::error!("swscale failed: {e}");
            return None;
        }
        Some(out)
    }
}
