//! Video decode thread: packets -> decoded frames -> RGBA -> frame queue.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam_channel::{Receiver, Sender};
use ffmpeg_next as ffmpeg;
use ffmpeg::format::Pixel;
use ffmpeg::software::scaling;

use super::{PacketMsg, PlayerEvent, VideoFrame};

pub(super) struct VideoDecodeCtx {
    pub decoder: ffmpeg::decoder::Video,
    /// Seconds per stream timestamp unit.
    pub time_base: f64,
    pub start_time: f64,
    pub frame_duration: f64,
    /// Container-level sample aspect ratio; preferred over the codec's (like
    /// `av_guess_sample_aspect_ratio`), since e.g. WebM stores it only there.
    pub stream_sar: Option<f64>,
    pub serial: Arc<AtomicU64>,
    pub pkt_rx: Receiver<PacketMsg>,
    pub frame_tx: Sender<VideoFrame>,
    pub event_tx: Sender<PlayerEvent>,
    pub repaint: egui::Context,
    pub initial_serial: u64,
}

struct State {
    serial: u64,
    drop_before: Option<f64>,
    first_after_flush: bool,
    last_pts: f64,
    scaler: Option<(scaling::Context, Pixel, u32, u32)>,
    decoded: ffmpeg::frame::Video,
    rgba: ffmpeg::frame::Video,
}

pub(super) fn run(mut c: VideoDecodeCtx) {
    let mut st = State {
        serial: c.initial_serial,
        drop_before: None,
        first_after_flush: true,
        last_pts: 0.0,
        scaler: None,
        decoded: ffmpeg::frame::Video::empty(),
        rgba: ffmpeg::frame::Video::empty(),
    };

    while let Ok(msg) = c.pkt_rx.recv() {
        match msg {
            PacketMsg::Packet { packet, serial } => {
                if serial != st.serial || c.serial.load(Ordering::Acquire) != serial {
                    continue;
                }
                if let Err(e) = c.decoder.send_packet(&packet) {
                    log::debug!("send_packet: {e}");
                }
                if !drain(&mut c, &mut st) {
                    return;
                }
            }
            PacketMsg::Flush {
                serial,
                drop_before,
            } => {
                c.decoder.flush();
                st.serial = serial;
                st.drop_before = drop_before;
                st.first_after_flush = true;
            }
            PacketMsg::Eof { serial } => {
                if serial != st.serial {
                    continue;
                }
                let _ = c.decoder.send_eof();
                if !drain(&mut c, &mut st) {
                    return;
                }
                // Reset the EOF state so decoding can resume after a seek.
                c.decoder.flush();
                if c.event_tx.send(PlayerEvent::Eof { serial }).is_err() {
                    return;
                }
                c.repaint.request_repaint();
            }
        }
    }
}

/// Pulls every available frame out of the decoder. Returns `false` when the
/// player is gone and the thread should exit.
fn drain(c: &mut VideoDecodeCtx, st: &mut State) -> bool {
    while c.decoder.receive_frame(&mut st.decoded).is_ok() {
        if c.serial.load(Ordering::Acquire) != st.serial {
            continue; // stale: a newer seek is on its way
        }

        let pts = st
            .decoded
            .timestamp()
            .map(|t| t as f64 * c.time_base - c.start_time)
            .unwrap_or(st.last_pts + c.frame_duration);
        st.last_pts = pts;

        if let Some(target) = st.drop_before {
            if pts + c.frame_duration * 0.5 < target {
                continue;
            }
            st.drop_before = None;
        }

        let Some(frame) = convert(st, pts, c.frame_duration, c.stream_sar) else {
            continue;
        };
        if c.frame_tx.send(frame).is_err() {
            return false;
        }
        if st.first_after_flush {
            st.first_after_flush = false;
            c.repaint.request_repaint();
        }
    }
    true
}

fn convert(st: &mut State, pts: f64, duration: f64, stream_sar: Option<f64>) -> Option<VideoFrame> {
    let src = &st.decoded;
    let (fmt, w, h) = (src.format(), src.width(), src.height());
    if w == 0 || h == 0 {
        return None;
    }

    let needs_new = !matches!(&st.scaler, Some((_, f, sw, sh)) if *f == fmt && *sw == w && *sh == h);
    if needs_new {
        match scaling::Context::get(fmt, w, h, Pixel::RGBA, w, h, scaling::Flags::BILINEAR) {
            Ok(s) => st.scaler = Some((s, fmt, w, h)),
            Err(e) => {
                log::error!("swscale init failed for {fmt:?} {w}x{h}: {e}");
                return None;
            }
        }
    }
    let (scaler, ..) = st.scaler.as_mut()?;
    if let Err(e) = scaler.run(src, &mut st.rgba) {
        log::error!("swscale failed: {e}");
        return None;
    }

    let (w, h) = (w as usize, h as usize);
    let stride = st.rgba.stride(0);
    let data = st.rgba.data(0);
    let mut rgba = Vec::with_capacity(w * h * 4);
    for row in 0..h {
        rgba.extend_from_slice(&data[row * stride..row * stride + w * 4]);
    }

    let frame_sar = src.aspect_ratio();
    let sar = stream_sar.unwrap_or(if frame_sar.numerator() > 0 && frame_sar.denominator() > 0 {
        f64::from(frame_sar)
    } else {
        1.0
    });

    Some(VideoFrame {
        serial: st.serial,
        pts,
        duration,
        width: w,
        height: h,
        sample_aspect: sar as f32,
        rgba,
    })
}
