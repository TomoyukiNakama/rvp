//! Demux thread: reads packets from the container and routes them per stream.

use crossbeam_channel::{Receiver, Sender, select};
use ffmpeg_next as ffmpeg;

use super::{DemuxCmd, PacketMsg, PlayerEvent};

pub(super) struct DemuxCtx {
    pub input: ffmpeg::format::context::Input,
    pub video_index: usize,
    /// Container start time in seconds (added back when seeking).
    pub start_time: f64,
    pub cmd_rx: Receiver<DemuxCmd>,
    pub video_tx: Sender<PacketMsg>,
    pub event_tx: Sender<PlayerEvent>,
}

pub(super) fn run(mut d: DemuxCtx) {
    let mut serial = 0u64;
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
                if d.video_tx.send(PacketMsg::Eof { serial }).is_err() {
                    return;
                }
                continue;
            }
            Err(e) => {
                consecutive_errors += 1;
                log::warn!("demux read error: {e}");
                if consecutive_errors > 100 {
                    let _ = d.event_tx.send(PlayerEvent::Error(format!("読み込みエラー: {e}")));
                    eof = true;
                    let _ = d.video_tx.send(PacketMsg::Eof { serial });
                }
                continue;
            }
        }

        if packet.stream() != d.video_index {
            continue;
        }

        // Block on the (bounded) packet queue, but stay responsive to commands.
        let msg = PacketMsg::Packet { packet, serial };
        select! {
            send(d.video_tx, msg) -> res => if res.is_err() { return },
            recv(d.cmd_rx) -> cmd => {
                // The pending packet is dropped: every command invalidates it anyway.
                let Ok(cmd) = cmd else { return };
                if let Flow::Stop = handle_cmd(&mut d, cmd, &mut serial) {
                    return;
                }
                eof = false;
            }
        }
    }
}

enum Flow {
    Continue,
    Stop,
}

fn handle_cmd(d: &mut DemuxCtx, cmd: DemuxCmd, serial: &mut u64) -> Flow {
    let DemuxCmd::Seek {
        mut target,
        mut serial_new,
        mut exact,
    } = cmd
    else {
        return Flow::Stop;
    };
    // Coalesce queued seeks (e.g. while dragging the seek bar): only the latest matters.
    while let Ok(next) = d.cmd_rx.try_recv() {
        match next {
            DemuxCmd::Seek {
                target: t,
                serial_new: s,
                exact: e,
            } => {
                target = t;
                serial_new = s;
                exact = e;
            }
            DemuxCmd::Stop => return Flow::Stop,
        }
    }

    let ts = ((target + d.start_time) * f64::from(ffmpeg::ffi::AV_TIME_BASE)) as i64;
    if let Err(e) = d.input.seek(ts, ..ts) {
        log::debug!("backward seek failed ({e}), retrying unbounded");
        if let Err(e) = d.input.seek(ts, ..) {
            log::warn!("seek to {target:.3}s failed: {e}");
        }
    }

    *serial = serial_new;
    let flush = PacketMsg::Flush {
        serial: serial_new,
        drop_before: exact.then_some(target),
    };
    if d.video_tx.send(flush).is_err() {
        return Flow::Stop;
    }
    Flow::Continue
}
