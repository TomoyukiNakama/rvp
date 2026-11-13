//! Playback engine. Owns the demux/decode threads and the master clock.
//!
//! Every seek bumps a *serial* number; packets and frames carry the serial they
//! were produced under, and anything with an outdated serial is discarded at
//! each stage (same scheme as ffplay).

mod audio_decode;
mod audio_out;
mod clock;
mod demux;
mod frame;
mod hwaccel;
mod subtitle;
mod tempo;
mod video_decode;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use anyhow::{Context as _, anyhow};
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use ffmpeg_next as ffmpeg;

pub use audio_out::AudioDevice;
pub use frame::{GPU_16BIT, Matrix, PlaneLayout, VideoFrame};
pub use hwaccel::HwDec;
pub use subtitle::{SUBTITLE_EXTENSIONS, decode_text, is_subtitle_file};
use subtitle::{Cue, SubtitleTrack};
use audio_out::AudioShared;
use clock::Clock;

/// Serial numbers are unique across players so that a shared audio queue can
/// never accept stale chunks from a previous player's threads.
static NEXT_SERIAL_BASE: AtomicU64 = AtomicU64::new(1 << 32);

const PACKET_QUEUE: usize = 128;
const AUDIO_PACKET_QUEUE: usize = 256;
const FRAME_QUEUE: usize = 6;

#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub title: String,
    pub duration: Option<f64>,
    pub width: u32,
    pub height: u32,
    pub codec: String,
    /// Audio codec name; `None` when the file has no audio or output failed.
    pub audio_codec: Option<String>,
    pub audio_tracks: Vec<TrackInfo>,
    /// Embedded subtitle streams (bitmap ones are listed but unsupported).
    pub subtitle_tracks: Vec<TrackInfo>,
}

#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub stream: usize,
    /// 1-based number among tracks of its kind.
    pub number: usize,
    /// ISO 639 code from the metadata (`und` dropped).
    pub language: Option<String>,
    pub title: Option<String>,
    /// e.g. "(aac 2ch)".
    pub detail: String,
    pub supported: bool,
    pub default: bool,
}

impl TrackInfo {
    /// Menu label, e.g. "#1 Japanese Commentary (aac 2ch)", in the current UI language.
    pub fn label(&self) -> String {
        let mut parts = vec![format!("#{}", self.number)];
        if let Some(lang) = &self.language {
            parts.push(language_name(lang).unwrap_or_else(|| lang.clone()));
        }
        parts.extend(self.title.clone());
        parts.push(self.detail.clone());
        parts.join(" ")
    }
}

/// Which subtitles are shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubSel {
    Off,
    /// Embedded stream index.
    Embedded(usize),
    /// Index into the loaded external files.
    External(usize),
}

const TEXT_SUB_CODECS: &[&str] = &["subrip", "srt", "ass", "ssa", "mov_text", "webvtt", "text"];

/// Localised name of a track language (ISO 639-1/-2 code), for the common ones.
fn language_name(code: &str) -> Option<String> {
    let short = match code {
        "jpn" | "ja" => "ja",
        "eng" | "en" => "en",
        "chi" | "zho" | "zh" => "zh",
        "kor" | "ko" => "ko",
        "fre" | "fra" | "fr" => "fr",
        "ger" | "deu" | "de" => "de",
        "spa" | "es" => "es",
        "ita" | "it" => "it",
        "rus" | "ru" => "ru",
        "por" | "pt" => "pt",
        _ => return None,
    };
    Some(t!(&format!("lang-{short}")))
}

fn track_info(stream: &ffmpeg::format::stream::Stream<'_>, n: usize, audio: bool) -> TrackInfo {
    let meta = stream.metadata();
    let params = stream.parameters();
    let codec = params.id().name().to_owned();
    let detail = if audio {
        // SAFETY: field read on valid codec parameters.
        let ch = unsafe { (*params.as_ptr()).ch_layout.nb_channels };
        format!("({codec} {ch}ch)")
    } else {
        format!("({codec})")
    };
    TrackInfo {
        stream: stream.index(),
        number: n,
        language: meta.get("language").filter(|l| *l != "und").map(str::to_owned),
        title: meta.get("title").filter(|t| !t.trim().is_empty()).map(str::to_owned),
        detail,
        supported: audio || TEXT_SUB_CODECS.contains(&codec.as_str()),
        default: stream.disposition().contains(ffmpeg::format::stream::Disposition::DEFAULT),
    }
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
    SelectAudio(usize),
    SelectSubtitle(Option<usize>),
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
    /// Audio track switch (audio queue only).
    NewAudioDecoder {
        decoder: ffmpeg::decoder::Audio,
        time_base: f64,
    },
}

pub(crate) enum PlayerEvent {
    Eof { serial: u64 },
    AudioEof { serial: u64 },
    SubtitleCue { stream: usize, cue: Cue },
    /// First decoded frame was on the GPU (`true`) or in system memory.
    HwActive(bool),
    /// Hardware decoding failed and the decoder was replaced by a software one.
    HwFallback,
    Error(String),
}

pub fn init() -> anyhow::Result<()> {
    ffmpeg::init().context("FFmpeg initialisation failed")?;
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
    speed: f32,
    /// Selected audio stream (when audio is playing).
    audio_stream: Option<usize>,
    sub_embedded: HashMap<usize, SubtitleTrack>,
    sub_external: Vec<(PathBuf, SubtitleTrack)>,
    sub_sel: SubSel,
    sub_visible: bool,
    /// Positive = subtitles appear later.
    sub_delay: f64,
    /// Hardware device configured for decoding, and whether frames actually come from it.
    hw_device: Option<String>,
    hw_active: bool,
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
        .with_context(|| t!("error-audio-decoder"))?;
    Ok(Some(AudioSetup {
        index: stream.index(),
        time_base: f64::from(tb.numerator()) / f64::from(tb.denominator()),
        decoder,
    }))
}

impl Player {
    /// Opens `path`. Audio plays through `audio` when given (and the file has audio).
    pub fn open(path: &Path, repaint: egui::Context, audio: Option<&AudioDevice>, hwdec: HwDec) -> anyhow::Result<Self> {
        let input = ffmpeg::format::input(path)
            .with_context(|| t!("error-open-file", path = path.display().to_string()))?;

        let stream = input
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or_else(|| anyhow!(t!("error-no-video-stream")))?;
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

        let params = stream.parameters();
        let mut codec_ctx = ffmpeg::codec::context::Context::from_parameters(params.clone())?;
        let hw_device = hwaccel::attach(&mut codec_ctx, hwdec);
        let decoder = match &hw_device {
            // Hardware: a single decode thread is enough (and keeps the surface pool small).
            Some(name) => match codec_ctx.decoder().video() {
                Ok(d) => Ok(d),
                Err(e) => {
                    log::warn!("hwdec {name}: open failed ({e}), using software");
                    video_decode::open_sw_decoder(params.clone())
                }
            },
            None => video_decode::open_sw_decoder(params.clone()),
        }
        .with_context(|| t!("error-video-decoder"))?;

        let (audio_setup, audio_error) = match audio.map(|_| setup_audio(&input)).transpose() {
            Ok(a) => (a.flatten(), None),
            Err(e) => {
                log::warn!("audio disabled: {e:#}");
                (None, Some(t!("error-audio-playback", error = format!("{e:#}"))))
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

        let (mut audio_tracks, mut subtitle_tracks) = (Vec::new(), Vec::new());
        for st in input.streams() {
            match st.parameters().medium() {
                ffmpeg::media::Type::Audio => audio_tracks.push(track_info(&st, audio_tracks.len() + 1, true)),
                ffmpeg::media::Type::Subtitle => {
                    subtitle_tracks.push(track_info(&st, subtitle_tracks.len() + 1, false));
                }
                _ => {}
            }
        }

        // Subtitles: an external file next to the video wins, then a default embedded track.
        let sub_external: Vec<(PathBuf, SubtitleTrack)> = subtitle::find_external(path)
            .into_iter()
            .filter_map(|p| match subtitle::load_file(&p) {
                Ok(t) if t.len() > 0 => Some((p, t)),
                Ok(_) => None,
                Err(e) => {
                    log::warn!("subtitle {}: {e}", p.display());
                    None
                }
            })
            .collect();
        let sub_sel = if !sub_external.is_empty() {
            SubSel::External(0)
        } else {
            subtitle_tracks
                .iter()
                .find(|t| t.supported && t.default)
                .map_or(SubSel::Off, |t| SubSel::Embedded(t.stream))
        };
        let sub_decoder = match sub_sel {
            SubSel::Embedded(i) => demux::open_sub_decoder(&input, i),
            _ => None,
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
            audio_tracks,
            subtitle_tracks,
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
                    q.speed = 1.0;
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

        let audio_stream = audio_route.as_ref().map(|(i, _)| *i);
        thread::Builder::new().name("rvp-demux".into()).spawn({
            let ctx = demux::DemuxCtx {
                input,
                video_index,
                start_time,
                cmd_rx,
                video_tx,
                audio: audio_route,
                subtitle: sub_decoder,
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
                hw: hw_device.is_some(),
                params,
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
            speed: 1.0,
            audio_stream,
            sub_embedded: HashMap::new(),
            sub_external,
            sub_sel,
            sub_visible: true,
            sub_delay: 0.0,
            hw_device,
            hw_active: false,
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

    /// Name of the hardware decoder in use (`None` = software).
    pub fn hwdec(&self) -> Option<&str> {
        self.hw_active.then_some(self.hw_device.as_deref()).flatten()
    }

    // ---- Tracks & subtitles ----------------------------------------------------

    pub fn audio_stream(&self) -> Option<usize> {
        self.audio_stream
    }

    /// Switches the audio track (only when audio output is active).
    pub fn select_audio(&mut self, stream: usize) {
        if self.audio.is_none() || self.audio_stream == Some(stream) {
            return;
        }
        self.audio_stream = Some(stream);
        let _ = self.cmd_tx.send(DemuxCmd::SelectAudio(stream));
        self.reseek();
    }

    pub fn subtitle_selection(&self) -> SubSel {
        self.sub_sel
    }

    pub fn external_subtitles(&self) -> impl Iterator<Item = &Path> {
        self.sub_external.iter().map(|(p, _)| p.as_path())
    }

    pub fn select_subtitle(&mut self, sel: SubSel) {
        if sel == self.sub_sel {
            return;
        }
        let was_embedded = matches!(self.sub_sel, SubSel::Embedded(_));
        self.sub_sel = sel;
        match sel {
            SubSel::Embedded(i) => {
                let _ = self.cmd_tx.send(DemuxCmd::SelectSubtitle(Some(i)));
                // Re-read from here so cues already passed by the demuxer arrive.
                self.reseek();
            }
            _ if was_embedded => {
                let _ = self.cmd_tx.send(DemuxCmd::SelectSubtitle(None));
            }
            _ => {}
        }
        self.sub_visible = true;
    }

    /// Loads an external subtitle file and selects it.
    pub fn add_external_subtitle(&mut self, path: &Path) -> anyhow::Result<usize> {
        let track = subtitle::load_file(path).with_context(|| t!("error-subtitle-load", path = path.display().to_string()))?;
        if track.len() == 0 {
            anyhow::bail!(t!("error-subtitle-empty", path = path.display().to_string()));
        }
        let n = track.len();
        self.sub_external.push((path.to_path_buf(), track));
        self.select_subtitle(SubSel::External(self.sub_external.len() - 1));
        Ok(n)
    }

    pub fn subtitles_visible(&self) -> bool {
        self.sub_visible
    }

    pub fn set_subtitles_visible(&mut self, visible: bool) {
        self.sub_visible = visible;
    }

    pub fn subtitle_delay(&self) -> f64 {
        self.sub_delay
    }

    pub fn set_subtitle_delay(&mut self, delay: f64) {
        self.sub_delay = (delay * 1000.0).round() / 1000.0;
    }

    /// Subtitle text to show right now, if any.
    pub fn subtitle_text(&self) -> Option<String> {
        if !self.sub_visible {
            return None;
        }
        let track = match self.sub_sel {
            SubSel::Off => return None,
            SubSel::Embedded(i) => self.sub_embedded.get(&i)?,
            SubSel::External(i) => &self.sub_external.get(i)?.1,
        };
        let lines = track.active(self.position() - self.sub_delay);
        (!lines.is_empty()).then(|| lines.join("\n"))
    }

    /// Seeks to the current position (after a track switch).
    fn reseek(&mut self) {
        let pos = self.position();
        let resume = self.state == PlayState::Playing;
        self.seek(pos, true);
        if resume {
            self.play();
        }
    }

    pub const MIN_SPEED: f32 = 0.25;
    pub const MAX_SPEED: f32 = 4.0;

    /// Playback speed; audio keeps its pitch (`atempo`).
    pub fn set_speed(&mut self, speed: f32) {
        let speed = speed.clamp(Self::MIN_SPEED, Self::MAX_SPEED);
        self.speed = speed;
        self.clock.set_speed(f64::from(speed));
        if let Some(a) = &self.audio {
            a.lock().speed = speed;
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
                PlayerEvent::SubtitleCue { stream, cue } => self.sub_embedded.entry(stream).or_default().insert(cue),
                PlayerEvent::HwActive(on) => {
                    self.hw_active = on && self.hw_device.is_some();
                    log::info!("video decoding: {}", self.hwdec().unwrap_or("software"));
                }
                PlayerEvent::HwFallback => {
                    self.hw_active = false;
                    self.error = Some(t!("error-hw-fallback"));
                    self.reseek();
                }
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
            Some(f) if !self.waiting_first_frame => {
                ((f.pts - self.clock.time()) / f64::from(self.speed)).max(0.0)
            }
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
        let mut p = Player::open(&path, egui::Context::default(), Some(&dev), HwDec::Off).unwrap();
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

        // Speed change: audio (atempo) keeps driving the clock at the new rate.
        for (speed, secs) in [(2.0f32, 1.5), (0.5, 1.5), (1.0, 0.5)] {
            p.seek(2.0, true);
            p.play();
            p.set_speed(speed);
            next_frame(&mut p);
            run_for(&mut p, 0.4); // let the old-speed queue drain
            let start = p.position();
            let drift = run_for(&mut p, secs);
            let advanced = p.position() - start;
            let want = secs * f64::from(speed);
            assert!((advanced - want).abs() < 0.08 * want.max(1.0), "speed {speed}: advanced {advanced}, want {want}");
            assert!(drift < 0.03, "speed {speed}: A/V drift {drift}");
            let audible = p.audio.as_ref().unwrap().audible_time(p.serial.load(Ordering::Acquire));
            assert!(audible.is_some(), "speed {speed}: audio should be playing");
        }

        // Switching files on the same device starts audio promptly and in sync.
        drop(p);
        let mut p = Player::open(&path, egui::Context::default(), Some(&dev), HwDec::Off).unwrap();
        next_frame(&mut p);
        let drift = run_for(&mut p, 0.5);
        assert!(drift < 0.02, "A/V drift after reopen {drift}");
        let pos = p.position();
        assert!((0.4..0.6).contains(&pos), "position 0.5 s after reopen: {pos}");
    }

    /// 8 s clip with two audio tracks and two embedded subtitle tracks
    /// (SRT/jpn default, ASS/eng). Returns the clip path.
    fn sample_tracks(dir: &Path) -> Option<PathBuf> {
        std::fs::create_dir_all(dir).ok()?;
        let path = dir.join("tracks.mkv");
        if path.exists() {
            return Some(path);
        }
        let srt = dir.join("in_ja.srt");
        std::fs::write(&srt, "1\n00:00:01,000 --> 00:00:03,000\nこんにちは\n\n2\n00:00:05,000 --> 00:00:07,000\n<i>二番目</i>\n").ok()?;
        let ass = dir.join("in_en.ass");
        std::fs::write(
            &ass,
            "[Script Info]\nScriptType: v4.00+\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize\nStyle: Default,Arial,20\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:03.00,Default,,0,0,0,,{\\i1}Hello{\\i0}\\Nworld\n",
        )
        .ok()?;
        let ok = Command::new("ffmpeg")
            .args(["-loglevel", "error", "-y"])
            .args(["-f", "lavfi", "-i", "testsrc=size=320x240:rate=30:duration=8"])
            .args(["-f", "lavfi", "-i", "sine=frequency=440:sample_rate=48000:duration=8"])
            .args(["-f", "lavfi", "-i", "sine=frequency=880:sample_rate=48000:duration=8"])
            .arg("-i").arg(&srt)
            .arg("-i").arg(&ass)
            .args(["-map", "0:v", "-map", "1:a", "-map", "2:a", "-map", "3", "-map", "4"])
            .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-g", "30", "-c:a", "aac", "-c:s:0", "srt", "-c:s:1", "ass"])
            .args(["-metadata:s:a:1", "language=eng", "-metadata:s:s:0", "language=jpn", "-metadata:s:s:1", "language=eng"])
            .args(["-disposition:s:0", "default", "-disposition:s:1", "0"])
            .arg(&path)
            .status()
            .ok()?
            .success();
        ok.then_some(path)
    }

    fn play_until(p: &mut Player, t: f64) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while p.position() < t {
            p.update();
            assert!(Instant::now() < deadline, "stuck at {}", p.position());
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn tracks_and_subtitles() {
        init().unwrap();
        let dir = std::env::temp_dir().join(format!("rvp-tracks-{}", std::process::id()));
        let Some(path) = sample_tracks(&dir) else {
            eprintln!("ffmpeg CLI not available; skipping");
            return;
        };
        let dev = AudioDevice::open().ok();
        let mut p = Player::open(&path, egui::Context::default(), dev.as_ref(), HwDec::Off).unwrap();

        let info = p.info().clone();
        assert_eq!(info.audio_tracks.len(), 2);
        assert_eq!(info.subtitle_tracks.len(), 2);
        assert!(info.subtitle_tracks.iter().all(|t| t.supported));
        assert!(info.subtitle_tracks[0].label().contains("Japanese"), "{}", info.subtitle_tracks[0].label());
        assert!(info.audio_tracks[1].label().contains("English"), "{}", info.audio_tracks[1].label());
        let (ja, en) = (info.subtitle_tracks[0].stream, info.subtitle_tracks[1].stream);

        // Default-flagged embedded track is picked automatically.
        assert_eq!(p.subtitle_selection(), SubSel::Embedded(ja));
        next_frame(&mut p);
        play_until(&mut p, 1.5);
        assert_eq!(p.subtitle_text().as_deref(), Some("こんにちは"));
        play_until(&mut p, 3.3);
        assert_eq!(p.subtitle_text(), None, "cue ended at 3 s");

        // Switching to the ASS track re-reads from the current position.
        p.seek(1.2, true);
        p.select_subtitle(SubSel::Embedded(en));
        next_frame(&mut p);
        play_until(&mut p, 1.6);
        assert_eq!(p.subtitle_text().as_deref(), Some("Hello\nworld"));

        // Delay shifts cue timing; hiding suppresses text.
        p.set_subtitle_delay(1.0); // cue now spans 2..4 s
        assert_eq!(p.subtitle_text(), None);
        p.set_subtitle_delay(0.0);
        p.set_subtitles_visible(false);
        assert_eq!(p.subtitle_text(), None);
        p.set_subtitles_visible(true);

        // Audio track switch keeps playing (when an output device exists).
        if dev.is_some() {
            let second = info.audio_tracks[1].stream;
            p.select_audio(second);
            assert_eq!(p.audio_stream(), Some(second));
            next_frame(&mut p);
            let start = p.position();
            let drift = run_for(&mut p, 0.8);
            assert!(drift < 0.03, "A/V drift after audio switch {drift}");
            assert!(p.position() - start > 0.6, "playback stalled after audio switch");
        }

        // An external file next to the video takes precedence on open.
        drop(p);
        let ext = dir.join("tracks.srt");
        std::fs::write(&ext, "1\n00:00:00,500 --> 00:00:02,000\n外部字幕\n").unwrap();
        let mut p = Player::open(&path, egui::Context::default(), None, HwDec::Off).unwrap();
        assert_eq!(p.subtitle_selection(), SubSel::External(0));
        next_frame(&mut p);
        play_until(&mut p, 0.8);
        assert_eq!(p.subtitle_text().as_deref(), Some("外部字幕"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Same frame decoded in hardware and software must match closely.
    #[test]
    fn hardware_decode_matches_software() {
        init().unwrap();
        let Some(path) = sample_video() else { return };
        let grab = |mode: HwDec| {
            let mut p = Player::open(&path, egui::Context::default(), None, mode).unwrap();
            p.pause();
            p.seek(3.0, true);
            let f = next_frame(&mut p);
            (p.hwdec().map(str::to_owned), f)
        };
        let (none, sw) = grab(HwDec::Off);
        assert!(none.is_none());
        let mut tested = 0;
        for mode in [HwDec::Cuda, HwDec::Vaapi] {
            let (name, hw) = grab(mode);
            let Some(name) = name else {
                eprintln!("{mode:?} unavailable; skipping");
                continue;
            };
            tested += 1;
            assert_eq!(hw.layout, PlaneLayout::SemiPlanar, "{name}: downloaded as NV12");
            assert!((hw.pts - sw.pts).abs() < 1e-6, "{name}: pts {} vs {}", hw.pts, sw.pts);
            assert_eq!((hw.width, hw.height), (sw.width, sw.height));
            let (hy, hs) = hw.plane_data(0);
            let (sy, ss) = sw.plane_data(0);
            let mut diff = 0u64;
            for row in 0..sw.height {
                for x in 0..sw.width {
                    diff += u64::from(hy[row * hs + x].abs_diff(sy[row * ss + x]));
                }
            }
            let mean = diff as f64 / (sw.width * sw.height) as f64;
            assert!(mean < 1.0, "{name}: mean luma difference {mean}");
            eprintln!("{name}: mean luma diff {mean:.3}");
        }
        eprintln!("hardware decoders tested: {tested}");
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
        let mut p = Player::open(&path, egui::Context::default(), None, HwDec::Off).unwrap();
        assert_eq!((p.info().width, p.info().height), (320, 240));
        assert!((p.info().duration.unwrap() - 12.0).abs() < 0.1);

        let f = next_frame(&mut p);
        assert!(f.pts.abs() < 0.05, "first pts {}", f.pts);
        assert_eq!(f.layout, PlaneLayout::Planar);
        assert_eq!((f.planes[0].width, f.planes[1].width, f.planes[1].height), (320, 160, 120));
        assert_eq!(f.matrix, Matrix::Bt601);
        let (y, stride) = f.plane_data(0);
        assert!(stride >= 320 && y.len() >= stride * 239 + 320);

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
