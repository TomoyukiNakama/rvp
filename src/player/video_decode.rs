//! Video decode thread: packets -> decoded frames -> RGBA -> frame queue.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam_channel::{Receiver, Sender};
use ffmpeg_next as ffmpeg;

use super::frame::{FrameConverter, VideoFrame};
use super::hwaccel;
use super::{PacketMsg, PlayerEvent};

/// Consecutive hardware errors before switching to software decoding.
const HW_ERROR_LIMIT: u32 = 5;

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
    /// Hardware decoding is configured on `decoder`.
    pub hw: bool,
    /// For rebuilding a software decoder if hardware decoding fails.
    pub params: ffmpeg::codec::Parameters,
}

/// Opens a software decoder with frame threading.
pub(super) fn open_sw_decoder(params: ffmpeg::codec::Parameters) -> Result<ffmpeg::decoder::Video, ffmpeg::Error> {
    let mut ctx = ffmpeg::codec::context::Context::from_parameters(params)?;
    ctx.set_threading(ffmpeg::codec::threading::Config {
        kind: ffmpeg::codec::threading::Type::Frame,
        count: 0,
        ..Default::default()
    });
    ctx.decoder().video()
}

struct State {
    serial: u64,
    drop_before: Option<f64>,
    first_after_flush: bool,
    last_pts: f64,
    converter: FrameConverter,
    decoded: ffmpeg::frame::Video,
    hw_errors: u32,
    reported_hw: bool,
}

pub(super) fn run(mut c: VideoDecodeCtx) {
    let mut st = State {
        serial: c.initial_serial,
        drop_before: None,
        first_after_flush: true,
        last_pts: 0.0,
        converter: FrameConverter::default(),
        decoded: ffmpeg::frame::Video::empty(),
        hw_errors: 0,
        reported_hw: false,
    };

    while let Ok(msg) = c.pkt_rx.recv() {
        match msg {
            PacketMsg::Packet { packet, serial } => {
                if serial != st.serial || c.serial.load(Ordering::Acquire) != serial {
                    continue;
                }
                if let Err(e) = c.decoder.send_packet(&packet) {
                    log::debug!("send_packet: {e}");
                    if c.hw && hw_error(&mut c, &mut st) {
                        return;
                    }
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
            PacketMsg::NewAudioDecoder { .. } => {} // audio-only message
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
/// Counts a hardware failure; past the limit, swaps in a software decoder and
/// asks the player to re-seek. Returns `true` if the thread should exit.
fn hw_error(c: &mut VideoDecodeCtx, st: &mut State) -> bool {
    st.hw_errors += 1;
    if st.hw_errors < HW_ERROR_LIMIT {
        return false;
    }
    log::warn!("hwdec: too many errors, switching to software decoding");
    match open_sw_decoder(c.params.clone()) {
        Ok(d) => {
            c.decoder = d;
            c.hw = false;
            st.hw_errors = 0;
            c.event_tx.send(PlayerEvent::HwFallback).is_err()
        }
        Err(e) => {
            let _ = c.event_tx.send(PlayerEvent::Error(format!("デコーダを開けません: {e}")));
            true
        }
    }
}

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

        let mut decoded = std::mem::replace(&mut st.decoded, ffmpeg::frame::Video::empty());
        let on_gpu = hwaccel::is_hw_frame(&decoded);
        if !st.reported_hw {
            st.reported_hw = true;
            let _ = c.event_tx.send(PlayerEvent::HwActive(on_gpu));
        }
        if on_gpu {
            match hwaccel::download(&decoded) {
                Ok(sw) => {
                    decoded = sw;
                    st.hw_errors = 0;
                }
                Err(e) => {
                    log::warn!("hwdec download failed: {e}");
                    if hw_error(c, st) {
                        return false;
                    }
                    continue;
                }
            }
        }
        let Some(frame) = st.converter.make(decoded, st.serial, pts, c.frame_duration, c.stream_sar) else {
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
