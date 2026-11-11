//! Playback engine. Owns the demux/decode threads and the master clock.
//!
//! Every seek bumps a *serial* number; packets and frames carry the serial they
//! were produced under, and anything with an outdated serial is discarded at
//! each stage (same scheme as ffplay).

mod clock;
mod demux;
mod video_decode;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::{Context as _, anyhow};
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use ffmpeg_next as ffmpeg;

use clock::Clock;

const PACKET_QUEUE: usize = 64;
const FRAME_QUEUE: usize = 6;

/// A decoded, display-ready video frame.
pub struct VideoFrame {
    pub serial: u64,
    /// Presentation time in seconds (relative to media start).
    pub pts: f64,
    pub duration: f64,
    pub width: usize,
    pub height: usize,
    pub sample_aspect: f32,
    pub rgba: Vec<u8>,
}

impl VideoFrame {
    /// Display aspect ratio (width / height), taking non-square pixels into account.
    pub fn display_aspect(&self) -> f32 {
        self.width as f32 * self.sample_aspect / self.height as f32
    }
}

#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub title: String,
    pub duration: Option<f64>,
    pub width: u32,
    pub height: u32,
    pub codec: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    Playing,
    Paused,
    Ended,
}

pub(crate) enum DemuxCmd {
    Seek {
        target: f64,
        serial_new: u64,
        exact: bool,
    },
    Stop,
}

pub(crate) enum PacketMsg {
    Packet {
        packet: ffmpeg::Packet,
        serial: u64,
    },
    Flush {
        serial: u64,
        drop_before: Option<f64>,
    },
    Eof {
        serial: u64,
    },
}

pub(crate) enum PlayerEvent {
    Eof { serial: u64 },
    Error(String),
}

pub fn init() -> anyhow::Result<()> {
    ffmpeg::init().context("FFmpeg の初期化に失敗しました")?;
    ffmpeg::util::log::set_level(ffmpeg::util::log::Level::Error);
    Ok(())
}

pub struct Player {
    info: MediaInfo,
    cmd_tx: Sender<DemuxCmd>,
    frame_rx: Receiver<VideoFrame>,
    event_rx: Receiver<PlayerEvent>,
    serial: Arc<AtomicU64>,
    clock: Clock,
    state: PlayState,
    pending: Option<VideoFrame>,
    /// Set after open/seek: the next frame re-anchors the clock.
    waiting_first_frame: bool,
    /// Position to report while waiting for the first frame after a seek.
    seek_target: f64,
    last_shown_pts: f64,
    last_shown_end: f64,
    eof: bool,
    error: Option<String>,
}

impl Player {
    pub fn open(path: &Path, repaint: egui::Context) -> anyhow::Result<Self> {
        let input = ffmpeg::format::input(path)
            .with_context(|| format!("開けませんでした: {}", path.display()))?;

        let stream = input
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or_else(|| anyhow!("映像ストリームが見つかりません"))?;
        let video_index = stream.index();
        let tb = stream.time_base();
        let time_base = f64::from(tb.numerator()) / f64::from(tb.denominator());
        let fps = stream.avg_frame_rate();
        let frame_duration = if fps.numerator() > 0 && fps.denominator() > 0 {
            f64::from(fps.denominator()) / f64::from(fps.numerator())
        } else {
            1.0 / 30.0
        };

        // SAFETY: plain field read on a valid stream.
        let stream_sar = unsafe { (*stream.as_ptr()).sample_aspect_ratio };
        let stream_sar = (stream_sar.num > 0 && stream_sar.den > 0)
            .then(|| f64::from(stream_sar.num) / f64::from(stream_sar.den));

        let mut codec_ctx = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
        codec_ctx.set_threading(ffmpeg::codec::threading::Config {
            kind: ffmpeg::codec::threading::Type::Frame,
            count: 0,
            ..Default::default()
        });
        let decoder = codec_ctx
            .decoder()
            .video()
            .context("映像デコーダを開けませんでした")?;

        // SAFETY: plain field reads on a valid, opened AVFormatContext.
        let (start_time, duration) = unsafe {
            let p = input.as_ptr();
            let tb = f64::from(ffmpeg::ffi::AV_TIME_BASE);
            let start = (*p).start_time;
            let dur = (*p).duration;
            (
                if start == ffmpeg::ffi::AV_NOPTS_VALUE { 0.0 } else { start as f64 / tb },
                (dur > 0 && dur != ffmpeg::ffi::AV_NOPTS_VALUE).then(|| dur as f64 / tb),
            )
        };

        let info = MediaInfo {

            title: input
                .metadata()
                .get("title")
                .map(str::to_owned)
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| {
                    path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
                }),
            duration,
            width: decoder.width(),
            height: decoder.height(),
            codec: decoder
                .codec()
                .map(|c| c.name().to_owned())
                .unwrap_or_default(),
        };

        let serial = Arc::new(AtomicU64::new(0));
        let (cmd_tx, cmd_rx) = unbounded();
        let (video_tx, video_rx) = bounded(PACKET_QUEUE);
        let (frame_tx, frame_rx) = bounded(FRAME_QUEUE);
        let (event_tx, event_rx) = unbounded();

        thread::Builder::new().name("rvp-demux".into()).spawn({
            let ctx = demux::DemuxCtx {
                input,
                video_index,
                start_time,
                cmd_rx,
                video_tx,
                event_tx: event_tx.clone(),
            };
            move || demux::run(ctx)
        })?;
        thread::Builder::new().name("rvp-vdec".into()).spawn({
            let ctx = video_decode::VideoDecodeCtx {
                decoder,
                time_base,
                start_time,
                frame_duration,
                stream_sar,
                serial: serial.clone(),
                pkt_rx: video_rx,
                frame_tx,
                event_tx,
                repaint,
            };
            move || video_decode::run(ctx)
        })?;

        log::info!(
            "opened {} ({} {}x{}, {:?}s)",
            path.display(),
            info.codec,
            info.width,
            info.height,
            info.duration
        );

        Ok(Self {
            info,
            cmd_tx,
            frame_rx,
            event_rx,
            serial,
            clock: Clock::default(),
            state: PlayState::Playing,
            pending: None,
            waiting_first_frame: true,
            seek_target: 0.0,
            last_shown_pts: 0.0,
            last_shown_end: 0.0,
            eof: false,
            error: None,
        })
    }

    pub fn info(&self) -> &MediaInfo {
        &self.info
    }

    pub fn state(&self) -> PlayState {
        self.state
    }

    pub fn take_error(&mut self) -> Option<String> {
        self.error.take()
    }

    /// Current playback position in seconds.
    pub fn position(&self) -> f64 {
        let t = if self.waiting_first_frame {
            self.seek_target
        } else if self.state == PlayState::Ended {
            self.last_shown_end
        } else {
            self.clock.time()
        };
        match self.info.duration {
            Some(d) => t.clamp(0.0, d),
            None => t.max(0.0),
        }
    }

    pub fn play(&mut self) {
        match self.state {
            PlayState::Playing => {}
            PlayState::Paused => {
                self.state = PlayState::Playing;
                self.clock.set_paused(false);
            }
            PlayState::Ended => {
                self.state = PlayState::Playing;
                self.clock.set_paused(false);
                self.seek(0.0, true);
            }
        }
    }

    pub fn pause(&mut self) {
        if self.state == PlayState::Playing {
            self.state = PlayState::Paused;
            self.clock.set_paused(true);
        }
    }

    pub fn toggle_pause(&mut self) {
        if self.state == PlayState::Playing {
            self.pause();
        } else {
            self.play();
        }
    }

    /// Seeks to `target` seconds. With `exact`, frames before the target are
    /// decoded and dropped; otherwise playback resumes at the previous keyframe.
    pub fn seek(&mut self, target: f64, exact: bool) {
        let target = match self.info.duration {
            Some(d) => target.clamp(0.0, (d - 0.05).max(0.0)),
            None => target.max(0.0),
        };
        let serial = self.serial.fetch_add(1, Ordering::AcqRel) + 1;
        self.pending = None;
        while self.frame_rx.try_recv().is_ok() {}
        self.waiting_first_frame = true;
        self.seek_target = target;
        self.eof = false;
        if self.state == PlayState::Ended {
            self.state = PlayState::Paused;
            self.clock.set_paused(true);
        }
        let _ = self.cmd_tx.send(DemuxCmd::Seek {
            target,
            serial_new: serial,
            exact,
        });
    }

    pub fn seek_relative(&mut self, delta: f64) {
        let base = self.position();
        self.seek(base + delta, true);
    }

    /// Advances the engine. Returns a frame that should be shown now, if any.
    /// Call once per UI frame.
    pub fn update(&mut self) -> Option<VideoFrame> {
        let serial = self.serial.load(Ordering::Acquire);

        while let Ok(ev) = self.event_rx.try_recv() {
            match ev {
                PlayerEvent::Eof { serial: s } if s == serial => self.eof = true,
                PlayerEvent::Eof { .. } => {}
                PlayerEvent::Error(e) => self.error = Some(e),
            }
        }

        let mut show = None;
        loop {
            if self.pending.is_none() {
                match self.frame_rx.try_recv() {
                    Ok(f) if f.serial == serial => self.pending = Some(f),
                    Ok(_) => continue, // stale frame from before a seek
                    Err(_) => break,
                }
            }
            let Some(f) = self.pending.as_ref() else { break };

            if self.waiting_first_frame {
                self.waiting_first_frame = false;
                self.clock.set(f.pts);
            } else if f.pts > self.clock.time() {
                break;
            }
            // Due (or overdue): newer due frames replace older ones (= frame drop).
            show = self.pending.take();
        }

        if let Some(f) = &show {
            self.last_shown_pts = f.pts;
            self.last_shown_end = f.pts + f.duration;
        }

        if self.eof
            && self.state == PlayState::Playing
            && self.pending.is_none()
            && self.frame_rx.is_empty()
            && !self.waiting_first_frame
            && self.clock.time() >= self.last_shown_end
        {
            self.state = PlayState::Ended;
            self.clock.set_paused(true);
            self.clock.set(self.last_shown_end);
        }

        show
    }

    /// How long the UI may sleep before the next call to [`Self::update`].
    pub fn next_wakeup(&self) -> Option<Duration> {
        if self.state != PlayState::Playing {
            return None;
        }
        let wait = match &self.pending {
            Some(f) if !self.waiting_first_frame => (f.pts - self.clock.time()).max(0.0),
            // Waiting on the decoder: poll briefly.
            _ => 0.005,
        };
        Some(Duration::from_secs_f64(wait.min(0.1)))
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        // Invalidate in-flight work and let the threads observe disconnection.
        self.serial.fetch_add(1, Ordering::AcqRel);
        let _ = self.cmd_tx.send(DemuxCmd::Stop);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::Instant;

    /// 12 s, 30 fps, keyframe every 2 s. `None` if the ffmpeg CLI is unavailable.
    fn sample_video() -> Option<PathBuf> {
        let dir = std::env::temp_dir().join(format!("rvp-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok()?;
        let path = dir.join("sample.mp4");
        if !path.exists() {
            let ok = Command::new("ffmpeg")
                .args(["-loglevel", "error", "-y", "-f", "lavfi", "-i", "testsrc=size=320x240:rate=30:duration=12"])
                .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-g", "60", "-keyint_min", "60", "-sc_threshold", "0"])
                .arg(&path)
                .status()
                .ok()?
                .success();
            if !ok {
                return None;
            }
        }
        Some(path)
    }

    fn next_frame(p: &mut Player) -> VideoFrame {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(f) = p.update() {
                return f;
            }
            assert!(Instant::now() < deadline, "timed out waiting for a frame");
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    #[test]
    fn open_seek_and_end() {
        init().unwrap();
        let Some(path) = sample_video() else {
            eprintln!("ffmpeg CLI not available; skipping");
            return;
        };
        let mut p = Player::open(&path, egui::Context::default()).unwrap();
        assert_eq!((p.info().width, p.info().height), (320, 240));
        assert!((p.info().duration.unwrap() - 12.0).abs() < 0.1);

        let f = next_frame(&mut p);
        assert!(f.pts.abs() < 0.05, "first pts {}", f.pts);
        assert_eq!(f.rgba.len(), 320 * 240 * 4);

        // Exact seek lands on the requested frame even mid-GOP.
        p.pause();
        p.seek(7.0, true);
        let f = next_frame(&mut p);
        assert!((f.pts - 7.0).abs() < 0.04, "exact seek pts {}", f.pts);
        assert!((p.position() - 7.0).abs() < 0.04);
        assert_eq!(p.state(), PlayState::Paused);

        // Keyframe seek snaps back to the GOP start (keyframes every 2 s).
        p.seek(9.5, false);
        let f = next_frame(&mut p);
        assert!((f.pts - 8.0).abs() < 0.04, "keyframe seek pts {}", f.pts);

        // Play out the tail and reach Ended.
        p.seek(11.5, true);
        p.play();
        let deadline = Instant::now() + Duration::from_secs(5);
        while p.state() != PlayState::Ended {
            p.update();
            assert!(Instant::now() < deadline, "never reached Ended");
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!((p.position() - 12.0).abs() < 0.1, "end position {}", p.position());

        // Play after the end restarts from the beginning.
        p.play();
        let f = next_frame(&mut p);
        assert!(f.pts.abs() < 0.05, "restart pts {}", f.pts);
    }
}
