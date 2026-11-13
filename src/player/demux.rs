//! Demux thread: reads packets from the container and routes them per stream.
//! Text subtitles are decoded right here (they are tiny and sparse) and sent
//! to the player as cues.

use crossbeam_channel::{Receiver, Sender, select};
use ffmpeg_next as ffmpeg;

use super::subtitle::{Cue, ass_event_text, strip_markup};
use super::{DemuxCmd, PacketMsg, PlayerEvent};

pub(super) struct SubDecoder {
    pub index: usize,
    pub decoder: ffmpeg::decoder::Subtitle,
    pub time_base: f64,
}

pub(super) struct DemuxCtx {
    pub input: ffmpeg::format::context::Input,
    pub video_index: usize,
    /// Container start time in seconds (added back when seeking).
    pub start_time: f64,
    pub cmd_rx: Receiver<DemuxCmd>,
    pub video_tx: Sender<PacketMsg>,
    /// Audio stream index and its packet queue, if audio is being played.
    pub audio: Option<(usize, Sender<PacketMsg>)>,
    pub subtitle: Option<SubDecoder>,
    pub event_tx: Sender<PlayerEvent>,
    pub initial_serial: u64,
}

pub(super) fn open_sub_decoder(input: &ffmpeg::format::context::Input, index: usize) -> Option<SubDecoder> {
    let stream = input.stream(index)?;
    let tb = stream.time_base();
    let decoder = ffmpeg::codec::context::Context::from_parameters(stream.parameters())
        .and_then(|c| c.decoder().subtitle())
        .map_err(|e| log::warn!("subtitle decoder for stream {index}: {e}"))
        .ok()?;
    Some(SubDecoder {
        index,
        decoder,
        time_base: f64::from(tb.numerator()) / f64::from(tb.denominator()),
    })
}

pub(super) fn run(mut d: DemuxCtx) {
    let mut serial = d.initial_serial;
    let mut eof = false;
    let mut consecutive_errors = 0u32;

    loop {
        // Commands first (non-blocking unless we are idle at EOF).
        let cmd = if eof {
            match d.cmd_rx.recv() {
                Ok(c) => Some(c),
                Err(_) => return,
            }
        } else {
            match d.cmd_rx.try_recv() {
                Ok(c) => Some(c),
                Err(crossbeam_channel::TryRecvError::Empty) => None,
                Err(crossbeam_channel::TryRecvError::Disconnected) => return,
            }
        };
        if let Some(cmd) = cmd {
            match handle_cmd(&mut d, cmd, &mut serial) {
                Flow::Continue => eof = false,
                Flow::Stop => return,
            }
            continue;
        }

        let mut packet = ffmpeg::Packet::empty();
        match packet.read(&mut d.input) {
            Ok(()) => consecutive_errors = 0,
            Err(ffmpeg::Error::Eof) => {
                eof = true;
                if !broadcast(&d, || PacketMsg::Eof { serial }) {
                    return;
                }
                continue;
            }
            Err(e) => {
                consecutive_errors += 1;
                log::warn!("demux read error: {e}");
                if consecutive_errors > 100 {
                    let _ = d.event_tx.send(PlayerEvent::Error(t!("error-read", error = e.to_string())));
                    eof = true;
                    broadcast(&d, || PacketMsg::Eof { serial });
                }
                continue;
            }
        }

        let idx = packet.stream();
        if d.subtitle.as_ref().is_some_and(|s| s.index == idx) {
            decode_subtitle(&mut d, &packet);
            continue;
        }
        let tx = if idx == d.video_index {
            d.video_tx.clone()
        } else if let Some((_, tx)) = d.audio.as_ref().filter(|(a, _)| *a == idx) {
            tx.clone()
        } else {
            continue;
        };

        // Block on the (bounded) packet queue, but stay responsive to commands.
        let msg = PacketMsg::Packet { packet, serial };
        select! {
            send(tx, msg) -> res => if res.is_err() { return },
            recv(d.cmd_rx) -> cmd => {
                // The pending packet is dropped: seeks invalidate it anyway, and
                // track switches are always followed by a seek.
                let Ok(cmd) = cmd else { return };
                if let Flow::Stop = handle_cmd(&mut d, cmd, &mut serial) {
                    return;
                }
                eof = false;
            }
        }
    }
}

fn decode_subtitle(d: &mut DemuxCtx, packet: &ffmpeg::Packet) {
    let Some(sd) = d.subtitle.as_mut() else { return };
    let mut sub = ffmpeg::Subtitle::new();
    let got = match sd.decoder.decode(packet, &mut sub) {
        Ok(g) => g,
        Err(e) => {
            log::debug!("subtitle decode: {e}");
            return;
        }
    };
    if got {
        let base = packet.pts().or(packet.dts()).map_or(0.0, |p| p as f64 * sd.time_base) - d.start_time;
        let start = base + f64::from(sub.start()) / 1000.0;
        let end = match sub.end() {
            0 | u32::MAX if packet.duration() > 0 => base + packet.duration() as f64 * sd.time_base,
            0 | u32::MAX => start + 5.0,
            e => base + f64::from(e) / 1000.0,
        };
        let mut lines = Vec::new();
        for rect in sub.rects() {
            match rect {
                ffmpeg::subtitle::Rect::Ass(a) => lines.push(ass_event_text(a.get())),
                ffmpeg::subtitle::Rect::Text(t) => lines.push(strip_markup(t.get())),
                _ => {}
            }
        }
        let text = lines.join("\n");
        if !text.trim().is_empty() {
            let _ = d.event_tx.send(PlayerEvent::SubtitleCue {
                stream: sd.index,
                cue: Cue { start, end, text },
            });
        }
    }
    // SAFETY: frees the rects allocated by avcodec_decode_subtitle2 (ffmpeg-next doesn't).
    unsafe { ffmpeg::ffi::avsubtitle_free(sub.as_mut_ptr()) };
}

enum Flow {
    Continue,
    Stop,
}

/// Applies a track switch. Returns `false` if the player is gone.
fn select_track(d: &mut DemuxCtx, cmd: DemuxCmd) -> bool {
    match cmd {
        DemuxCmd::SelectSubtitle(index) => {
            d.subtitle = index.and_then(|i| open_sub_decoder(&d.input, i));
            true
        }
        DemuxCmd::SelectAudio(index) => {
            let Some((cur, tx)) = d.audio.as_mut() else { return true };
            let Some(stream) = d.input.stream(index) else { return true };
            let tb = stream.time_base();
            match ffmpeg::codec::context::Context::from_parameters(stream.parameters())
                .and_then(|c| c.decoder().audio())
            {
                Ok(decoder) => {
                    *cur = index;
                    let msg = PacketMsg::NewAudioDecoder {
                        decoder,
                        time_base: f64::from(tb.numerator()) / f64::from(tb.denominator()),
                    };
                    tx.send(msg).is_ok()
                }
                Err(e) => {
                    let _ = d.event_tx.send(PlayerEvent::Error(t!("error-audio-track-open", error = format!("{e:#}"))));
                    true
                }
            }
        }
        DemuxCmd::Seek { .. } | DemuxCmd::Stop => unreachable!("not a track command"),
    }
}

fn handle_cmd(d: &mut DemuxCtx, cmd: DemuxCmd, serial: &mut u64) -> Flow {
    let (mut target, mut serial_new, mut exact) = match cmd {
        DemuxCmd::Seek { target, serial_new, exact } => (target, serial_new, exact),
        DemuxCmd::Stop => return Flow::Stop,
        other => {
            return if select_track(d, other) { Flow::Continue } else { Flow::Stop };
        }
    };
    // Coalesce queued seeks (e.g. while dragging the seek bar): only the latest matters.
    while let Ok(next) = d.cmd_rx.try_recv() {
        match next {
            DemuxCmd::Seek { target: t, serial_new: s, exact: e } => {
                target = t;
                serial_new = s;
                exact = e;
            }
            DemuxCmd::Stop => return Flow::Stop,
            other => {
                if !select_track(d, other) {
                    return Flow::Stop;
                }
            }
        }
    }

    let ts = ((target + d.start_time) * f64::from(ffmpeg::ffi::AV_TIME_BASE)) as i64;
    if let Err(e) = d.input.seek(ts, ..ts) {
        log::debug!("backward seek failed ({e}), retrying unbounded");
        if let Err(e) = d.input.seek(ts, ..) {
            log::warn!("seek to {target:.3}s failed: {e}");
        }
    }
    if let Some(sd) = d.subtitle.as_mut() {
        sd.decoder.flush();
    }

    *serial = serial_new;
    if !broadcast(d, || PacketMsg::Flush {
        serial: serial_new,
        drop_before: exact.then_some(target),
    }) {
        return Flow::Stop;
    }
    Flow::Continue
}

/// Sends a control message to every active stream queue. `false` = player gone.
fn broadcast(d: &DemuxCtx, msg: impl Fn() -> PacketMsg) -> bool {
    let mut ok = d.video_tx.send(msg()).is_ok();
    if let Some((_, tx)) = &d.audio {
        ok &= tx.send(msg()).is_ok();
    }
    ok
}
