//! Playback engine. Owns the demux/decode threads and the master clock.
//!
//! Every seek bumps a *serial* number; packets and frames carry the serial they
//! were produced under, and anything with an outdated serial is discarded at
//! each stage (same scheme as ffplay).

mod audio_decode;
mod audio_out;
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

pub use audio_out::AudioDevice;
use audio_out::AudioShared;
use clock::Clock;

/// Serial numbers are unique across players so that a shared audio queue can
/// never accept stale chunks from a previous player's threads.
static NEXT_SERIAL_BASE: AtomicU64 = AtomicU64::new(1 << 32);

const PACKET_QUEUE: usize = 128;
const AUDIO_PACKET_QUEUE: usize = 256;
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
    /// Audio codec name; `None` when the file has no audio or output failed.
    pub audio_codec: Option<String>,
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
    AudioEof { serial: u64 },
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
    audio: Option<Arc<AudioShared>>,
    audio_eof: bool,
    error: Option<String>,
}

struct AudioSetup {
    index: usize,
    time_base: f64,
    decoder: ffmpeg::decoder::Audio,
}

fn setup_audio(input: &ffmpeg::format::context::Input) -> anyhow::Result<Option<AudioSetup>> {
    let Some(stream) = input.streams().best(ffmpeg::media::Type::Audio) else {
        return Ok(None);
    };
    let tb = stream.time_base();
    let decoder = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?
        .decoder()
        .audio()
        .context("音声デコーダを開けませんでした")?;
    Ok(Some(AudioSetup {
        index: stream.index(),
        time_base: f64::from(tb.numerator()) / f64::from(tb.denominator()),
        decoder,
    }))
}

impl Player {
    /// Opens `path`. Audio plays through `audio` when given (and the file has audio).
    pub fn open(path: &Path, repaint: egui::Context, audio: Option<&AudioDevice>) -> anyhow::Result<Self> {
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

        let (audio_setup, audio_error) = match audio.map(|_| setup_audio(&input)).transpose() {
            Ok(a) => (a.flatten(), None),
            Err(e) => {
                log::warn!("audio disabled: {e:#}");
                (None, Some(format!("音声を再生できません: {e:#}")))
            }
        };

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
            audio_codec: audio_setup
                .as_ref()
                .and_then(|a| a.decoder.codec().map(|c| c.name().to_owned())),
        };

        let base = NEXT_SERIAL_BASE.fetch_add(1 << 32, Ordering::Relaxed);
        let serial = Arc::new(AtomicU64::new(base));
        let (cmd_tx, cmd_rx) = unbounded();
        let (video_tx, video_rx) = bounded(PACKET_QUEUE);
        let (frame_tx, frame_rx) = bounded(FRAME_QUEUE);
        let (event_tx, event_rx) = unbounded();

        let (audio_route, audio_shared) = match (audio_setup, audio) {
            (Some(a), Some(dev)) => {
                {
                    let mut q = dev.shared.lock();
                    q.serial = base;
                    q.chunks.clear();
                    q.queued_frames = 0;
                    q.clock = None;
                    q.hold = true;
                    q.paused = false;
                }
                let (tx, rx) = bounded(AUDIO_PACKET_QUEUE);
                let ctx = audio_decode::AudioDecodeCtx {
                    decoder: a.decoder,
                    time_base: a.time_base,
                    start_time,
                    shared: dev.shared.clone(),
                    serial: serial.clone(),
                    pkt_rx: rx,
                    event_tx: event_tx.clone(),
                    initial_serial: base,
                };
                thread::Builder::new()
                    .name("rvp-adec".into())
                    .spawn(move || audio_decode::run(ctx))?;
                (Some((a.index, tx)), Some(dev.shared.clone()))
            }
            _ => (None, None),
        };

        thread::Builder::new().name("rvp-demux".into()).spawn({
            let ctx = demux::DemuxCtx {
                input,
                video_index,
                start_time,
                cmd_rx,
                video_tx,
                audio: audio_route,
                event_tx: event_tx.clone(),
                initial_serial: base,
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
                initial_serial: base,
            };
            move || video_decode::run(ctx)
        })?;

        log::info!(
            "opened {} ({} {}x{}, audio {:?}, {:?}s)",
            path.display(),
            info.codec,
            info.width,
            info.height,
            info.audio_codec,
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
            audio: audio_shared,
            audio_eof: false,
            error: audio_error,
        })
    }

    /// Volume as a linear slider value in `0.0..=1.5`, mapped to a cubic gain.
    pub fn set_volume(&mut self, volume: f32, muted: bool) {
        if let Some(a) = &self.audio {
            let v = volume.clamp(0.0, 1.5);
            a.lock().gain = if muted { 0.0 } else { v * v * v };
        }
    }

    fn sync_audio_paused(&self) {
        if let Some(a) = &self.audio {
            a.lock().paused = self.state != PlayState::Playing;
        }
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
                self.sync_audio_paused();
            }
            PlayState::Ended => {
                self.state = PlayState::Playing;
                self.clock.set_paused(false);
                self.sync_audio_paused();
                self.seek(0.0, true);
            }
        }
    }

    pub fn pause(&mut self) {
        if self.state == PlayState::Playing {
            self.state = PlayState::Paused;
            self.clock.set_paused(true);
            self.sync_audio_paused();
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
        self.audio_eof = false;
        if self.state == PlayState::Ended {
            self.state = PlayState::Paused;
            self.clock.set_paused(true);
        }
        if let Some(a) = &self.audio {
            let mut q = a.lock();
            q.serial = serial;
            q.chunks.clear();
            q.queued_frames = 0;
            q.clock = None;
            q.hold = true;
            q.paused = self.state != PlayState::Playing;
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
                PlayerEvent::AudioEof { serial: s } if s == serial => self.audio_eof = true,
                PlayerEvent::Eof { .. } | PlayerEvent::AudioEof { .. } => {}
                PlayerEvent::Error(e) => self.error = Some(e),
            }
        }

        // Discipline the clock to what is actually audible.
        if self.state == PlayState::Playing && !self.waiting_first_frame {
            if let Some(t) = self.audio.as_ref().and_then(|a| a.audible_time(serial)) {
                if (self.clock.time() - t).abs() > 0.008 {
                    self.clock.set(t);
                }
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
                // Video is ready: let audio start.
                if let Some(a) = &self.audio {
                    a.lock().hold = false;
                }
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
            && self.audio_drained()
        {
            self.state = PlayState::Ended;
            self.clock.set_paused(true);
            self.clock.set(self.last_shown_end);
        }

        show
    }

    fn audio_drained(&self) -> bool {
        match &self.audio {
            None => true,
            Some(a) => self.audio_eof && a.lock().chunks.is_empty(),
        }
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
        if let Some(a) = &self.audio {
            let mut q = a.lock();
            q.chunks.clear();
            q.queued_frames = 0;
            q.clock = None;
            q.hold = true;
        }
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

    /// 10 s, 30 fps video + 44.1 kHz stereo AAC tone (exercises resampling to the device rate).
    fn sample_av() -> Option<PathBuf> {
        let dir = std::env::temp_dir().join(format!("rvp-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok()?;
        let path = dir.join("sample_av_aac.mkv");
        if !path.exists() {
            let ok = Command::new("ffmpeg")
                .args(["-loglevel", "error", "-y", "-f", "lavfi", "-i", "testsrc=size=320x240:rate=30:duration=10"])
                .args(["-f", "lavfi", "-i", "sine=frequency=440:sample_rate=44100:duration=10"])
                .args(["-ac", "2", "-c:v", "libx264", "-pix_fmt", "yuv420p", "-g", "60", "-c:a", "aac"])
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

    /// Runs the engine for `secs`, returning the max |video clock − audible audio| seen.
    fn run_for(p: &mut Player, secs: f64) -> f64 {
        let end = Instant::now() + Duration::from_secs_f64(secs);
        let mut max_drift: f64 = 0.0;
        while Instant::now() < end {
            p.update();
            let serial = p.serial.load(Ordering::Acquire);
            if let Some(t) = p.audio.as_ref().and_then(|a| a.audible_time(serial)) {
                max_drift = max_drift.max((p.clock.time() - t).abs());
            }
            std::thread::sleep(Duration::from_millis(4));
        }
        max_drift
    }

    #[test]
    fn audio_drives_clock_and_follows_seek() {
        let _ = env_logger::builder().is_test(true).try_init();
        init().unwrap();
        let Some(path) = sample_av() else {
            eprintln!("ffmpeg CLI not available; skipping");
            return;
        };
        let dev = match AudioDevice::open() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("no audio output device; skipping ({e:#})");
                return;
            }
        };
        // Let the device get past its start-up silence (as the app does at launch).
        std::thread::sleep(Duration::from_millis(400));
        let mut p = Player::open(&path, egui::Context::default(), Some(&dev)).unwrap();
        assert_eq!(p.info().audio_codec.as_deref(), Some("aac"));
        next_frame(&mut p);

        let drift = run_for(&mut p, 1.0);
        assert!(drift < 0.02, "A/V drift {drift}");
        let pos = p.position();
        assert!((0.9..1.15).contains(&pos), "position after 1 s: {pos}");

        // Seek: audio restarts at the target and keeps the clock in step.
        p.seek(6.0, true);
        next_frame(&mut p);
        let drift = run_for(&mut p, 0.5);
        assert!(drift < 0.02, "A/V drift after seek {drift}");
        let audible = p.audio.as_ref().unwrap().audible_time(p.serial.load(Ordering::Acquire));
        let audible = audible.expect("audio should be playing after seek");
        assert!((6.3..6.8).contains(&audible), "audible after seek: {audible}");

        // Pause freezes the position; audio stops reporting.
        p.pause();
        let frozen = p.position();
        run_for(&mut p, 0.3);
        assert!((p.position() - frozen).abs() < 1e-9);
        assert!(p.audio.as_ref().unwrap().audible_time(p.serial.load(Ordering::Acquire)).is_none());

        // Play to the end: Ended once both streams drain.
        p.seek(9.0, true);
        p.play();
        let deadline = Instant::now() + Duration::from_secs(5);
        while p.state() != PlayState::Ended {
            p.update();
            assert!(Instant::now() < deadline, "never reached Ended");
            std::thread::sleep(Duration::from_millis(5));
        }

        // Switching files on the same device starts audio promptly and in sync.
        drop(p);
        let mut p = Player::open(&path, egui::Context::default(), Some(&dev)).unwrap();
        next_frame(&mut p);
        let drift = run_for(&mut p, 0.5);
        assert!(drift < 0.02, "A/V drift after reopen {drift}");
        let pos = p.position();
        assert!((0.4..0.6).contains(&pos), "position 0.5 s after reopen: {pos}");
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
        let mut p = Player::open(&path, egui::Context::default(), None).unwrap();
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
