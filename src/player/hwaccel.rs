//! Hardware video decoding (CUDA/NVDEC, VAAPI) via FFmpeg hwaccels.
//!
//! Decoded GPU surfaces are downloaded to system memory (NV12 / P010) with
//! `av_hwframe_transfer_data` and then take the normal semi-planar upload path.
//! Device contexts are created once per process and cached (CUDA init is slow).

use std::collections::HashMap;
use std::ffi::CString;
use std::sync::{Mutex, OnceLock};

use ffmpeg_next as ffmpeg;
use ffmpeg::ffi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HwDec {
    /// Try CUDA, then VAAPI; fall back to software.
    #[default]
    Auto,
    Off,
    Cuda,
    Vaapi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    Cuda,
    Vaapi,
}

impl Kind {
    fn av_type(self) -> ffi::AVHWDeviceType {
        match self {
            Kind::Cuda => ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_CUDA,
            Kind::Vaapi => ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_VAAPI,
        }
    }
}

/// A process-lifetime hardware device context.
struct Device {
    buf: *mut ffi::AVBufferRef,
    name: String,
}

// SAFETY: AVHWDeviceContext is reference counted and thread safe to share;
// we only ever take new references from it.
unsafe impl Send for Device {}

fn devices() -> &'static Mutex<HashMap<Kind, Option<Device>>> {
    static D: OnceLock<Mutex<HashMap<Kind, Option<Device>>>> = OnceLock::new();
    D.get_or_init(Default::default)
}

fn create(kind: Kind, node: Option<&str>) -> Option<*mut ffi::AVBufferRef> {
    let c_node = node.map(|n| CString::new(n).expect("device path"));
    let mut buf = std::ptr::null_mut();
    // SAFETY: standard device creation; `buf` is only used on success.
    let r = unsafe {
        ffi::av_hwdevice_ctx_create(
            &mut buf,
            kind.av_type(),
            c_node.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
            std::ptr::null_mut(),
            0,
        )
    };
    (r >= 0 && !buf.is_null()).then_some(buf)
}

fn probe(kind: Kind) -> Option<Device> {
    match kind {
        Kind::Cuda => create(kind, None).map(|buf| Device { buf, name: "cuda".into() }),
        Kind::Vaapi => {
            // libva picks the first render node by default, which may be a GPU
            // without a VA driver (e.g. NVIDIA next to an iGPU): try each.
            let mut nodes: Vec<String> = std::fs::read_dir("/dev/dri")
                .map(|rd| {
                    rd.filter_map(|e| e.ok())
                        .map(|e| e.path().to_string_lossy().into_owned())
                        .filter(|p| p.contains("renderD"))
                        .collect()
                })
                .unwrap_or_default();
            nodes.sort();
            nodes.into_iter().find_map(|n| create(kind, Some(&n)).map(|buf| Device { buf, name: format!("vaapi ({n})") }))
        }
    }
}

/// Returns a new reference to the (cached) device and its display name.
fn device(kind: Kind) -> Option<(*mut ffi::AVBufferRef, String)> {
    let mut map = devices().lock().unwrap_or_else(|e| e.into_inner());
    let dev = map.entry(kind).or_insert_with(|| {
        let d = probe(kind);
        match &d {
            Some(d) => log::info!("hwdec: {} available", d.name),
            None => log::info!("hwdec: {kind:?} unavailable"),
        }
        d
    });
    // SAFETY: `buf` is a live device reference owned by the cache.
    dev.as_ref().map(|d| (unsafe { ffi::av_buffer_ref(d.buf) }, d.name.clone()))
}

/// Hardware pixel format the decoder supports for `kind`, if any.
fn hw_pix_fmt(codec: *const ffi::AVCodec, kind: Kind) -> Option<ffi::AVPixelFormat> {
    let mut i = 0;
    loop {
        // SAFETY: iterating the codec's static hw config table until null.
        let cfg = unsafe { ffi::avcodec_get_hw_config(codec, i) };
        if cfg.is_null() {
            return None;
        }
        // SAFETY: non-null config entry.
        let cfg = unsafe { &*cfg };
        if cfg.methods & ffi::AV_CODEC_HW_CONFIG_METHOD_HW_DEVICE_CTX as i32 != 0 && cfg.device_type == kind.av_type() {
            return Some(cfg.pix_fmt);
        }
        i += 1;
    }
}

/// Picks the hardware format offered by the decoder, else lets FFmpeg choose
/// a software format (e.g. unsupported profile → transparent SW fallback).
unsafe extern "C" fn get_format(ctx: *mut ffi::AVCodecContext, fmts: *const ffi::AVPixelFormat) -> ffi::AVPixelFormat {
    // SAFETY: `opaque` holds the wanted pix_fmt (set in `attach`); `fmts` is
    // an AV_PIX_FMT_NONE-terminated list provided by FFmpeg.
    unsafe {
        let want = (*ctx).opaque as usize as i32;
        let mut p = fmts;
        while *p != ffi::AVPixelFormat::AV_PIX_FMT_NONE {
            if *p as i32 == want {
                return *p;
            }
            p = p.add(1);
        }
        log::info!("hwdec: format not offered for this stream, using software");
        ffi::avcodec_default_get_format(ctx, fmts)
    }
}

/// Configures `ctx` (not yet opened) for hardware decoding. Returns the
/// device name on success.
pub fn attach(ctx: &mut ffmpeg::codec::context::Context, mode: HwDec) -> Option<String> {
    let kinds: &[Kind] = match mode {
        HwDec::Off => return None,
        HwDec::Auto => &[Kind::Cuda, Kind::Vaapi],
        HwDec::Cuda => &[Kind::Cuda],
        HwDec::Vaapi => &[Kind::Vaapi],
    };
    // SAFETY: field reads/writes on an allocated, unopened codec context.
    unsafe {
        let raw = ctx.as_mut_ptr();
        let codec = ffi::avcodec_find_decoder((*raw).codec_id);
        if codec.is_null() {
            return None;
        }
        for &kind in kinds {
            let Some(fmt) = hw_pix_fmt(codec, kind) else { continue };
            let Some((buf, name)) = device(kind) else { continue };
            (*raw).hw_device_ctx = buf; // context takes ownership of this reference
            (*raw).opaque = fmt as i32 as usize as *mut std::ffi::c_void;
            (*raw).get_format = Some(get_format);
            return Some(name);
        }
    }
    None
}

/// True if `frame` lives in GPU memory and must be downloaded.
pub fn is_hw_frame(frame: &ffmpeg::frame::Video) -> bool {
    // SAFETY: field read on a valid frame.
    unsafe { !(*frame.as_ptr()).hw_frames_ctx.is_null() }
}

/// Downloads a GPU frame into system memory (keeping timestamps/colour props).
pub fn download(hw: &ffmpeg::frame::Video) -> Result<ffmpeg::frame::Video, ffmpeg::Error> {
    let mut sw = ffmpeg::frame::Video::empty();
    // SAFETY: both frames are valid; `sw` is empty so FFmpeg allocates it.
    unsafe {
        let r = ffi::av_hwframe_transfer_data(sw.as_mut_ptr(), hw.as_ptr(), 0);
        if r < 0 {
            return Err(ffmpeg::Error::from(r));
        }
        let r = ffi::av_frame_copy_props(sw.as_mut_ptr(), hw.as_ptr());
        if r < 0 {
            return Err(ffmpeg::Error::from(r));
        }
    }
    Ok(sw)
}
