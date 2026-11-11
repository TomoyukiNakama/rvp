//! Audio decode thread: packets -> decoded frames -> f32 at device format -> queue.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use ffmpeg_next as ffmpeg;
use ffmpeg::ffi;

use super::audio_out::{AudioChunk, AudioShared};
use super::{PacketMsg, PlayerEvent};

/// Upper bound of decoded audio waiting for the device.
const MAX_QUEUED_SECS: f64 = 0.3;

pub(super) struct AudioDecodeCtx {
    pub decoder: ffmpeg::decoder::Audio,
    pub time_base: f64,
    pub start_time: f64,
    pub shared: Arc<AudioShared>,
    pub serial: Arc<AtomicU64>,
    pub pkt_rx: Receiver<PacketMsg>,
    pub event_tx: Sender<PlayerEvent>,
    pub initial_serial: u64,
}

struct State {
    serial: u64,
    drop_before: Option<f64>,
    next_pts: Option<f64>,
    resampler: Option<Resampler>,
    frame: ffmpeg::frame::Audio,
}

pub(super) fn run(mut c: AudioDecodeCtx) {
    let mut st = State {
        serial: c.initial_serial,
        drop_before: None,
        next_pts: None,
        resampler: None,
        frame: ffmpeg::frame::Audio::empty(),
    };

    while let Ok(msg) = c.pkt_rx.recv() {
        match msg {
            PacketMsg::Packet { packet, serial } => {
                if serial != st.serial || c.serial.load(Ordering::Acquire) != serial {
                    continue;
                }
                if let Err(e) = c.decoder.send_packet(&packet) {
                    log::debug!("audio send_packet: {e}");
                }
                drain(&mut c, &mut st);
            }
            PacketMsg::Flush {
                serial,
                drop_before,
            } => {
                c.decoder.flush();
                st.serial = serial;
                st.drop_before = drop_before;
                st.next_pts = None;
                st.resampler = None; // drop buffered resampler state
            }
            PacketMsg::Eof { serial } => {
                if serial != st.serial {
                    continue;
                }
                let _ = c.decoder.send_eof();
                drain(&mut c, &mut st);
                c.decoder.flush();
                if c.event_tx.send(PlayerEvent::AudioEof { serial }).is_err() {
                    return;
                }
            }
        }
    }
}

fn drain(c: &mut AudioDecodeCtx, st: &mut State) {
    let out_rate = c.shared.rate;
    let out_ch = usize::from(c.shared.channels);

    while c.decoder.receive_frame(&mut st.frame).is_ok() {
        if c.serial.load(Ordering::Acquire) != st.serial {
            continue;
        }
        let in_rate = st.frame.rate();
        let n_in = st.frame.samples();
        if in_rate == 0 || n_in == 0 {
            continue;
        }

        let pts = st
            .frame
            .timestamp()
            .map(|t| t as f64 * c.time_base - c.start_time)
            .or(st.next_pts)
            .unwrap_or(0.0);
        st.next_pts = Some(pts + n_in as f64 / f64::from(in_rate));

        let key = FormatKey::of(&st.frame);
        if st.resampler.as_ref().is_none_or(|r| r.key != key) {
            match Resampler::new(&st.frame, out_rate, c.shared.channels) {
                Ok(r) => st.resampler = Some(r),
                Err(e) => {
                    log::error!("swresample init failed: {e}");
                    continue;
                }
            }
        }
        let Some(resampler) = st.resampler.as_mut() else { continue };
        let mut samples = match resampler.convert(&st.frame) {
            Ok(s) => s,
            Err(e) => {
                log::error!("swresample failed: {e}");
                continue;
            }
        };
        let mut pts = pts;

        // Precise seek: trim everything before the target.
        if let Some(target) = st.drop_before {
            let dur = (samples.len() / out_ch) as f64 / f64::from(out_rate);
            if pts + dur <= target {
                continue;
            }
            if pts < target {
                let skip_frames = ((target - pts) * f64::from(out_rate)) as usize;
                samples.drain(..(skip_frames * out_ch).min(samples.len()));
                pts = target;
            }
            st.drop_before = None;
        }
        if samples.is_empty() {
            continue;
        }

        push(c, st.serial, pts, samples);
    }
}

/// Waits for room in the queue, then appends. Gives up if the serial changes.
fn push(c: &AudioDecodeCtx, serial: u64, pts: f64, samples: Vec<f32>) {
    let max_frames = (MAX_QUEUED_SECS * f64::from(c.shared.rate)) as usize;
    let frames = samples.len() / usize::from(c.shared.channels);
    loop {
        if c.serial.load(Ordering::Acquire) != serial {
            return;
        }
        {
            let mut q = c.shared.lock();
            if q.serial != serial {
                return;
            }
            if q.queued_frames < max_frames {
                q.queued_frames += frames;
                q.chunks.push_back(AudioChunk {
                    serial,
                    pts,
                    samples,
                    offset: 0,
                });
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct FormatKey {
    format: i32,
    rate: u32,
    channels: i32,
}

impl FormatKey {
    fn of(frame: &ffmpeg::frame::Audio) -> Self {
        // SAFETY: field reads on a valid decoded frame.
        unsafe {
            let f = frame.as_ptr();
            Self {
                format: (*f).format,
                rate: (*f).sample_rate as u32,
                channels: (*f).ch_layout.nb_channels,
            }
        }
    }
}

/// Thin swresample wrapper converting to interleaved f32 at the device format.
///
/// Uses the C API directly: unlike ffmpeg-next's wrapper it sizes the output
/// for rate conversion, so up-sampling never strands samples in the context.
struct Resampler {
    ctx: *mut ffi::SwrContext,
    key: FormatKey,
    out_channels: usize,
}

// SAFETY: the context is only ever used from the owning (decode) thread.
unsafe impl Send for Resampler {}

impl Resampler {
    fn new(frame: &ffmpeg::frame::Audio, out_rate: u32, out_channels: u16) -> Result<Self, ffmpeg::Error> {
        let key = FormatKey::of(frame);
        // SAFETY: standard swresample setup; layouts are initialised before use
        // and uninitialised afterwards; `ctx` is freed on error and in Drop.
        unsafe {
            let f = frame.as_ptr();
            let mut in_layout: ffi::AVChannelLayout = std::mem::zeroed();
            if (*f).ch_layout.order == ffi::AVChannelOrder::AV_CHANNEL_ORDER_UNSPEC {
                ffi::av_channel_layout_default(&mut in_layout, key.channels);
            } else {
                let r = ffi::av_channel_layout_copy(&mut in_layout, &(*f).ch_layout);
                if r < 0 {
                    return Err(ffmpeg::Error::from(r));
                }
            }
            let mut out_layout: ffi::AVChannelLayout = std::mem::zeroed();
            ffi::av_channel_layout_default(&mut out_layout, i32::from(out_channels));

            let mut ctx: *mut ffi::SwrContext = std::ptr::null_mut();
            let r = ffi::swr_alloc_set_opts2(
                &mut ctx,
                &out_layout,
                ffi::AVSampleFormat::AV_SAMPLE_FMT_FLT,
                out_rate as i32,
                &in_layout,
                std::mem::transmute::<i32, ffi::AVSampleFormat>(key.format),
                key.rate as i32,
                0,
                std::ptr::null_mut(),
            );
            ffi::av_channel_layout_uninit(&mut in_layout);
            ffi::av_channel_layout_uninit(&mut out_layout);
            if r < 0 || ctx.is_null() {
                ffi::swr_free(&mut ctx);
                return Err(ffmpeg::Error::from(r.min(-1)));
            }
            let r = ffi::swr_init(ctx);
            if r < 0 {
                ffi::swr_free(&mut ctx);
                return Err(ffmpeg::Error::from(r));
            }
            Ok(Self {
                ctx,
                key,
                out_channels: usize::from(out_channels),
            })
        }
    }

    fn convert(&mut self, frame: &ffmpeg::frame::Audio) -> Result<Vec<f32>, ffmpeg::Error> {
        // SAFETY: `out` is sized for `cap` frames of interleaved f32; input
        // planes come straight from a valid decoded frame.
        unsafe {
            let n_in = frame.samples() as i32;
            let cap = ffi::swr_get_out_samples(self.ctx, n_in);
            if cap < 0 {
                return Err(ffmpeg::Error::from(cap));
            }
            let mut out = vec![0.0f32; cap as usize * self.out_channels];
            let out_ptr = out.as_mut_ptr().cast::<u8>();
            let n = ffi::swr_convert(
                self.ctx,
                &out_ptr,
                cap,
                (*frame.as_ptr()).extended_data as *const *const u8,
                n_in,
            );
            if n < 0 {
                return Err(ffmpeg::Error::from(n));
            }
            out.truncate(n as usize * self.out_channels);
            Ok(out)
        }
    }
}

impl Drop for Resampler {
    fn drop(&mut self) {
        // SAFETY: ctx was allocated by swr_alloc_set_opts2.
        unsafe { ffi::swr_free(&mut self.ctx) };
    }
}
