use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, unbounded};
use egui::{
    Align2, Color32, CornerRadius, CursorIcon, FontId, Id, Key, Modifiers, Rect, Sense, ViewportCommand, pos2,
    vec2,
};
use egui_phosphor::regular as icon;

use crate::config::{Config, ResumeStore};
use crate::player::{AudioDevice, GPU_16BIT, HwDec, PlayState, Player, SUBTITLE_EXTENSIONS, VideoFrame, is_subtitle_file};
use crate::playlist::{Playlist, VIDEO_EXTENSIONS};
use crate::render::{VideoPaint, VideoRenderer};
use crate::theme::{self, color, motion, size};
use crate::ui::controls::{self, ControlsModel};
use crate::ui::playlist_panel::{self, PlaylistAction, PlaylistModel};
use crate::ui::seekbar::SeekEvent;
use crate::ui::speed::{self, format_speed};
use crate::ui::tracks::{TracksAction, TracksModel, format_delay, subtitle_choices};
use crate::ui::volume::MAX_VOLUME;
use crate::ui::widgets::{format_time, icon_font};
use crate::ui::window;

const VOLUME_STEP: f32 = 0.05;
/// "Previous" restarts the current file if we're further in than this.
const PREV_RESTART_AFTER: f64 = 3.0;
const RESUME_SAVE_INTERVAL: f64 = 30.0;
const CONFIG_SAVE_DELAY: f64 = 1.0;

struct Osd {
    text: String,
    until: f64,
    error: bool,
}

#[derive(Clone, Copy)]
enum DialogKind {
    Open,
    OpenFolder,
    Append,
    Subtitle,
}

/// Result of the background file dialog.
struct DialogResult {
    paths: Vec<PathBuf>,
    kind: DialogKind,
}

pub struct RvpApp {
    player: Option<Player>,
    audio: Option<AudioDevice>,
    /// Frame currently on screen (uploaded to the GPU by the paint callback).
    current: Option<Arc<VideoFrame>>,
    /// File the current player was opened from.
    current_path: Option<PathBuf>,
    /// The current player's end has been handled (auto-advance done).
    end_handled: bool,
    playlist: Playlist,
    show_playlist: bool,
    config: Config,
    /// Time of the last unsaved config change.
    config_dirty_since: Option<f64>,
    resume: ResumeStore,
    last_resume_save: f64,
    video_aspect: f32,
    last_activity: f64,
    osd: Option<Osd>,
    dialog_tx: Sender<Option<DialogResult>>,
    dialog_rx: Receiver<Option<DialogResult>>,
    dialog_open: bool,
    wheel_accum: f32,
    /// A compositor move/resize grab was started: it swallows the button
    /// release, so one is injected into egui's input on the next frame.
    inject_release: bool,
    /// Drops on Wayland (winit only delivers them on X11).
    dnd_rx: Option<Receiver<crate::wayland_dnd::DndEvent>>,
    /// A file drag is hovering over the window.
    drag_hover: bool,
    /// Window-to-video aspect fitting: last seen size, when it last changed,
    /// and the (size, aspect) pair already handled.
    fit_size: egui::Vec2,
    fit_changed_at: f64,
    fit_done: Option<(egui::Vec2, f32)>,
    /// Playback speed, kept across files for the session (not persisted).
    speed: f32,
    debug_shot: Option<crate::debug_shot::DebugShot>,
}

impl RvpApp {
    pub fn new(cc: &eframe::CreationContext<'_>, files: Vec<PathBuf>) -> Self {
        crate::fonts::install(&cc.egui_ctx);
        if let Some(rs) = &cc.wgpu_render_state {
            let gpu16 = rs.device.features().contains(egui_wgpu::wgpu::Features::TEXTURE_FORMAT_16BIT_NORM);
            GPU_16BIT.store(gpu16, Ordering::Relaxed);
            VideoRenderer::install(rs);
            let info = rs.adapter.get_info();
            log::info!(
                "GPU: {} ({:?}), target {:?}, 16-bit textures: {gpu16}",
                info.name,
                info.backend,
                rs.target_format
            );
        }
        theme::apply(&cc.egui_ctx);

        // Open the audio device up front: it plays a few hundred ms of silence
        // while starting, which would otherwise delay the first file.
        let (audio, audio_error) = match AudioDevice::open() {
            Ok(d) => (Some(d), None),
            Err(e) => {
                log::warn!("audio device unavailable: {e:#}");
                (None, Some(format!("音声出力を開けません: {e:#}")))
            }
        };

        let config = Config::load();
        let mut playlist = Playlist::default();
        playlist.repeat = config.repeat;
        playlist.set_shuffle(config.shuffle);

        let (dialog_tx, dialog_rx) = unbounded();
        let mut app = Self {
            player: None,
            audio,
            current: None,
            current_path: None,
            end_handled: false,
            playlist,
            // Debug aid for screenshots (see debug_shot.rs).
            show_playlist: std::env::var_os("RVP_DEBUG_PLAYLIST").is_some(),
            config,
            config_dirty_since: None,
            resume: ResumeStore::load(),
            last_resume_save: 0.0,
            video_aspect: 16.0 / 9.0,
            last_activity: 0.0,
            osd: None,
            dialog_tx,
            dialog_rx,
            dialog_open: false,
            wheel_accum: 0.0,
            inject_release: false,
            dnd_rx: start_wayland_dnd(cc),
            drag_hover: false,
            fit_size: egui::Vec2::ZERO,
            fit_changed_at: 0.0,
            fit_done: None,
            // Debug aid for screenshots (see debug_shot.rs).
            speed: std::env::var("RVP_DEBUG_SPEED").ok().and_then(|s| s.parse().ok()).unwrap_or(1.0),
            debug_shot: crate::debug_shot::DebugShot::from_env(),
        };
        if let Some(e) = audio_error {
            app.show_osd(&cc.egui_ctx, e, true);
        }
        if !files.is_empty() {
            app.open_paths(&cc.egui_ctx, files, false);
        }
        app
    }

    // ---- Opening & playlist ---------------------------------------------------

    /// Replaces (or appends to) the playlist and starts playing if needed.
    fn open_paths(&mut self, ctx: &egui::Context, paths: Vec<PathBuf>, append: bool) {
        if append {
            let n = self.playlist.append(paths);
            self.show_osd(ctx, format!("{n} 件をプレイリストに追加"), false);
            if self.player.is_none() {
                self.play_current(ctx);
            }
            return;
        }
        let single = paths.len() == 1;
        self.playlist.set(paths, single && self.config.queue_siblings);
        if self.playlist.len() == 0 {
            self.show_osd(ctx, "再生できる動画が見つかりません".into(), true);
            return;
        }
        self.play_current(ctx);
    }

    fn play_current(&mut self, ctx: &egui::Context) {
        if let Some(path) = self.playlist.current().map(Path::to_path_buf) {
            self.open(ctx, path);
        }
    }

    fn open(&mut self, ctx: &egui::Context, path: PathBuf) {
        self.remember_position();
        // Drop the old player first so its threads stop before new ones start.
        // The last frame stays on screen until the new file's first frame
        // replaces it (no black flash between consecutive files).
        self.player = None;
        self.current_path = None;
        self.end_handled = false;
        match Player::open(&path, ctx.clone(), self.audio.as_ref(), self.config.hwdec) {
            Ok(mut p) => {
                p.set_volume(self.config.volume, self.config.muted);
                p.set_speed(self.speed);
                if let Some(e) = p.take_error() {
                    self.show_osd(ctx, e, true);
                }
                let info = p.info();
                // With a previous frame still shown, keep its aspect until the
                // new first frame (which carries the exact SAR) arrives.
                if self.current.is_none() && info.height > 0 {
                    self.video_aspect = info.width as f32 / info.height as f32;
                }
                ctx.send_viewport_cmd(ViewportCommand::Title(format!("{} — rvp", info.title)));

                let resumed = self.config.resume.then(|| self.resume.get(&path)).flatten();
                if let Some(pos) = resumed {
                    p.seek(pos, true);
                    self.show_osd(ctx, format!("前回の位置から再開 ({})", format_time(pos)), false);
                } else if self.playlist.len() > 1 {
                    let n = self.playlist.current_index().map_or(0, |i| i + 1);
                    self.show_osd(ctx, format!("{n}/{}  {}", self.playlist.len(), info.title), false);
                }
                self.player = Some(p);
                self.current_path = Some(path);
            }
            Err(e) => {
                log::error!("{e:#}");
                self.current = None;
                self.show_osd(ctx, format!("{e:#}"), true);
            }
        }
    }

    /// Stores the current file's position for resuming later.
    fn remember_position(&mut self) {
        let (Some(p), Some(path)) = (&self.player, &self.current_path) else { return };
        if p.state() == PlayState::Ended {
            self.resume.forget(path);
        } else {
            self.resume.record(path, p.position(), p.info().duration);
        }
    }

    fn next(&mut self, ctx: &egui::Context) {
        if self.playlist.next().is_some() {
            self.play_current(ctx);
        } else {
            self.show_osd(ctx, "プレイリストの最後です".into(), false);
        }
    }

    fn prev(&mut self, ctx: &egui::Context) {
        if let Some(p) = self.player.as_mut()
            && p.position() > PREV_RESTART_AFTER
        {
            p.seek(0.0, true);
            return;
        }
        if self.playlist.prev().is_some() {
            self.play_current(ctx);
        } else if let Some(p) = self.player.as_mut() {
            p.seek(0.0, true);
        }
    }

    /// Auto-advance when the current file finishes.
    fn handle_end(&mut self, ctx: &egui::Context) {
        let ended = self.player.as_ref().is_some_and(|p| p.state() == PlayState::Ended);
        if !ended {
            self.end_handled = false;
            return;
        }
        if self.end_handled {
            return;
        }
        self.end_handled = true;
        if let Some(path) = &self.current_path {
            self.resume.forget(path);
        }
        let before = self.playlist.current_index();
        if self.playlist.on_finished().is_some() {
            // Looping back to the same entry (repeat one, or a one-item playlist
            // with repeat all): restart in place instead of reopening the file.
            if self.playlist.current_index() == before
                && let Some(p) = self.player.as_mut()
            {
                p.play(); // from Ended: seeks to 0 and plays
            } else {
                self.play_current(ctx);
            }
        }
    }

    fn cycle_repeat(&mut self, ctx: &egui::Context) {
        self.playlist.repeat = self.playlist.repeat.cycle();
        self.config.repeat = self.playlist.repeat;
        self.mark_config_dirty(ctx);
        self.show_osd(ctx, self.playlist.repeat.label().into(), false);
    }

    fn handle_playlist_action(&mut self, ctx: &egui::Context, action: PlaylistAction) {
        match action {
            PlaylistAction::Play(i) => {
                if self.playlist.select(i).is_some() {
                    self.play_current(ctx);
                }
            }
            PlaylistAction::Remove(i) => {
                let was_current = self.playlist.current_index() == Some(i);
                self.playlist.remove(i);
                if was_current {
                    if self.playlist.len() == 0 {
                        self.remember_position();
                        self.player = None;
                        self.current = None;
                        self.current_path = None;
                        ctx.send_viewport_cmd(ViewportCommand::Title("rvp".into()));
                    } else {
                        self.play_current(ctx);
                    }
                }
            }
            PlaylistAction::Move { from, to } => self.playlist.move_item(from, to),
            PlaylistAction::CycleRepeat => self.cycle_repeat(ctx),
            PlaylistAction::ToggleShuffle => {
                let on = !self.playlist.shuffle();
                self.playlist.set_shuffle(on);
                self.config.shuffle = on;
                self.mark_config_dirty(ctx);
                self.show_osd(ctx, if on { "シャッフル: オン" } else { "シャッフル: オフ" }.into(), false);
            }
            PlaylistAction::Add => self.open_dialog(ctx, DialogKind::Append),
            PlaylistAction::Close => self.show_playlist = false,
        }
    }

    // ---- Misc helpers ---------------------------------------------------------

    fn show_osd(&mut self, ctx: &egui::Context, text: String, error: bool) {
        let now = ctx.input(|i| i.time);
        self.osd = Some(Osd {
            text,
            until: now + if error { 5.0 } else { 1.5 },
            error,
        });
    }

    fn mark_config_dirty(&mut self, ctx: &egui::Context) {
        if self.config_dirty_since.is_none() {
            self.config_dirty_since = Some(ctx.input(|i| i.time));
        }
        ctx.request_repaint_after(Duration::from_secs_f64(CONFIG_SAVE_DELAY));
    }

    /// Debounced config save + periodic resume save.
    fn persist(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        if let Some(since) = self.config_dirty_since
            && now - since >= CONFIG_SAVE_DELAY
        {
            self.config.save();
            self.config_dirty_since = None;
        }
        if now - self.last_resume_save >= RESUME_SAVE_INTERVAL {
            self.last_resume_save = now;
            if self.player.as_ref().is_some_and(|p| p.state() == PlayState::Playing) {
                self.remember_position();
            }
            self.resume.save();
        }
    }

    fn open_dialog(&mut self, ctx: &egui::Context, kind: DialogKind) {
        if self.dialog_open {
            return;
        }
        self.dialog_open = true;
        let tx = self.dialog_tx.clone();
        let ctx = ctx.clone();
        let start_dir = self.current_path.as_ref().and_then(|p| p.parent()).map(Path::to_path_buf);
        std::thread::spawn(move || {
            let mut dialog = rfd::FileDialog::new();
            if let Some(dir) = start_dir {
                dialog = dialog.set_directory(dir);
            }
            if matches!(kind, DialogKind::OpenFolder) {
                let picked = dialog.set_title("フォルダを開く").pick_folders();
                let _ = tx.send(picked.map(|paths| DialogResult { paths, kind }));
                ctx.request_repaint();
                return;
            }
            dialog = match kind {
                DialogKind::Open | DialogKind::OpenFolder | DialogKind::Append => dialog
                    .set_title(if matches!(kind, DialogKind::Append) { "プレイリストに追加" } else { "動画を開く" })
                    .add_filter("動画", VIDEO_EXTENSIONS),
                DialogKind::Subtitle => dialog.set_title("字幕ファイルを読み込む").add_filter("字幕", SUBTITLE_EXTENSIONS),
            };
            let picked = dialog.add_filter("すべてのファイル", &["*"]).pick_files();
            let _ = tx.send(picked.map(|paths| DialogResult { paths, kind }));
            ctx.request_repaint();
        });
    }

    fn load_subtitles(&mut self, ctx: &egui::Context, paths: Vec<PathBuf>) {
        let Some(p) = self.player.as_mut() else {
            self.show_osd(ctx, "先に動画を開いてください".into(), true);
            return;
        };
        let mut msg = None;
        for path in paths {
            msg = Some(match p.add_external_subtitle(&path) {
                Ok(n) => (
                    format!("字幕: {} ({n} 件)", path.file_name().map_or_else(|| "".into(), |n| n.to_string_lossy())),
                    false,
                ),
                Err(e) => (format!("{e:#}"), true),
            });
        }
        if let Some((text, err)) = msg {
            self.show_osd(ctx, text, err);
        }
    }

    fn tracks_model(p: &Player) -> TracksModel<'_> {
        TracksModel {
            audio: &p.info().audio_tracks,
            audio_sel: p.audio_stream(),
            subs: &p.info().subtitle_tracks,
            externals: p
                .external_subtitles()
                .map(|e| e.file_name().map_or_else(|| e.to_string_lossy(), |n| n.to_string_lossy()).into_owned())
                .collect(),
            sub_sel: p.subtitle_selection(),
            visible: p.subtitles_visible(),
            delay: p.subtitle_delay(),
        }
    }

    fn handle_tracks_action(&mut self, ctx: &egui::Context, action: TracksAction) {
        let Some(p) = self.player.as_mut() else { return };
        let osd = match action {
            TracksAction::Audio(stream) => {
                p.select_audio(stream);
                let label = p.info().audio_tracks.iter().find(|t| t.stream == stream).map(|t| t.label.clone());
                format!("音声: {}", label.unwrap_or_default())
            }
            TracksAction::Subtitle(sel) => {
                p.select_subtitle(sel);
                let m = Self::tracks_model(p);
                let label = subtitle_choices(&m).into_iter().find(|(s, _)| *s == sel).map(|(_, l)| l);
                format!("字幕: {}", label.unwrap_or_default())
            }
            TracksAction::LoadSubtitle => {
                self.open_dialog(ctx, DialogKind::Subtitle);
                return;
            }
            TracksAction::ToggleVisible => {
                let v = !p.subtitles_visible();
                p.set_subtitles_visible(v);
                (if v { "字幕: 表示" } else { "字幕: 非表示" }).to_owned()
            }
            TracksAction::Delay(d) => {
                let new = if d == 0.0 { 0.0 } else { p.subtitle_delay() + d };
                p.set_subtitle_delay(new);
                format!("字幕のタイミング: {}", format_delay(p.subtitle_delay()))
            }
        };
        self.show_osd(ctx, osd, false);
    }

    /// Ctrl+H: hardware decoding auto ⇄ off; reopens the current file in place.
    fn toggle_hwdec(&mut self, ctx: &egui::Context) {
        self.config.hwdec = if self.config.hwdec == HwDec::Off { HwDec::Auto } else { HwDec::Off };
        self.mark_config_dirty(ctx);
        let reopen = self.player.as_ref().zip(self.current_path.clone()).map(|(p, path)| (path, p.position(), p.state()));
        if let Some((path, pos, state)) = reopen {
            self.open(ctx, path);
            if let Some(p) = self.player.as_mut() {
                p.seek(pos, true);
                if state == PlayState::Paused {
                    p.pause();
                }
            }
        }
        let mode = if self.config.hwdec == HwDec::Off { "オフ" } else { "自動" };
        self.show_osd(ctx, format!("ハードウェアデコード: {mode}"), false);
    }

    /// `I`: codec / resolution / decoder summary.
    fn show_media_info(&mut self, ctx: &egui::Context) {
        let Some(p) = self.player.as_ref() else { return };
        let i = p.info();
        let decoder = match p.hwdec() {
            Some(hw) => format!("HW ({hw})"),
            None => "ソフトウェア".to_owned(),
        };
        let audio = i.audio_codec.clone().unwrap_or_else(|| "なし".into());
        let text = format!("映像: {} {}×{} · デコード: {decoder}\n音声: {audio}", i.codec, i.width, i.height);
        self.show_osd(ctx, text, false);
    }

    /// Next subtitle (`S`) or audio (`A`) track, wrapping around.
    fn cycle_track(&mut self, ctx: &egui::Context, audio: bool) {
        let Some(p) = self.player.as_ref() else { return };
        let action = if audio {
            let tracks = &p.info().audio_tracks;
            let Some(cur) = p.audio_stream() else { return };
            if tracks.len() < 2 {
                return self.show_osd(ctx, "音声トラックは 1 つだけです".into(), false);
            }
            let i = tracks.iter().position(|t| t.stream == cur).map_or(0, |i| (i + 1) % tracks.len());
            TracksAction::Audio(tracks[i].stream)
        } else {
            let choices = subtitle_choices(&Self::tracks_model(p));
            let i = choices.iter().position(|(s, _)| *s == p.subtitle_selection()).map_or(0, |i| (i + 1) % choices.len());
            TracksAction::Subtitle(choices[i].0)
        };
        self.handle_tracks_action(ctx, action);
    }

    fn set_volume(&mut self, ctx: &egui::Context, volume: f32, muted: bool) {
        self.config.volume = (volume * 100.0).round().clamp(0.0, MAX_VOLUME * 100.0) / 100.0;
        self.config.muted = muted;
        if let Some(p) = self.player.as_mut() {
            p.set_volume(self.config.volume, self.config.muted);
        }
        self.mark_config_dirty(ctx);
        let text = if muted {
            "ミュート".to_owned()
        } else {
            format!("音量 {:.0}%", self.config.volume * 100.0)
        };
        self.show_osd(ctx, text, false);
    }

    fn set_speed(&mut self, ctx: &egui::Context, speed: f32) {
        self.speed = speed.clamp(Player::MIN_SPEED, Player::MAX_SPEED);
        if let Some(p) = self.player.as_mut() {
            p.set_speed(self.speed);
        }
        self.show_osd(ctx, format!("速度 {}", format_speed(self.speed)), false);
    }

    fn toggle_fullscreen(ctx: &egui::Context) {
        let fs = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!fs));
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        // Files from the dialog thread.
        while let Ok(res) = self.dialog_rx.try_recv() {
            self.dialog_open = false;
            if let Some(r) = res
                && !r.paths.is_empty()
            {
                match r.kind {
                    DialogKind::Open | DialogKind::OpenFolder => self.open_paths(ctx, r.paths, false),
                    DialogKind::Append => self.open_paths(ctx, r.paths, true),
                    DialogKind::Subtitle => self.load_subtitles(ctx, r.paths),
                }
            }
        }

        // Drag & drop: winit (X11) and our Wayland listener.
        let mut dropped: Vec<PathBuf> =
            ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).collect());
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            self.drag_hover = true;
        } else if self.dnd_rx.is_none() {
            self.drag_hover = false;
        }
        if let Some(rx) = &self.dnd_rx {
            for ev in rx.try_iter() {
                match ev {
                    crate::wayland_dnd::DndEvent::Hover(h) => self.drag_hover = h,
                    crate::wayland_dnd::DndEvent::Drop(paths) => dropped.extend(paths),
                }
            }
        }
        if !dropped.is_empty() {
            self.drag_hover = false;
            self.handle_dropped(ctx, dropped);
        }

        // Activity = pointer motion or any discrete input. (Not `any_down()`: a
        // compositor move grab can leave a button stuck "down" in egui.)
        let (now, active) = ctx.input(|i| {
            let discrete = i.raw.events.iter().any(|e| {
                matches!(
                    e,
                    egui::Event::PointerButton { .. } | egui::Event::Key { .. } | egui::Event::MouseWheel { .. }
                )
            });
            (i.time, i.pointer.delta() != egui::Vec2::ZERO || discrete)
        });
        if active {
            self.last_activity = now;
        }

        let key = |m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_key(m, k));

        if key(Modifiers::SHIFT, Key::O) {
            self.open_dialog(ctx, DialogKind::OpenFolder);
        }
        if key(Modifiers::NONE, Key::O) {
            self.open_dialog(ctx, DialogKind::Open);
        }
        if key(Modifiers::CTRL, Key::H) {
            self.toggle_hwdec(ctx);
        }
        if key(Modifiers::NONE, Key::I) {
            self.show_media_info(ctx);
        }
        if key(Modifiers::NONE, Key::V) {
            self.handle_tracks_action(ctx, TracksAction::ToggleVisible);
        }
        if key(Modifiers::NONE, Key::S) {
            self.cycle_track(ctx, false);
        }
        if key(Modifiers::NONE, Key::A) {
            self.cycle_track(ctx, true);
        }
        if key(Modifiers::NONE, Key::Z) {
            self.handle_tracks_action(ctx, TracksAction::Delay(-0.1));
        }
        if key(Modifiers::NONE, Key::X) {
            self.handle_tracks_action(ctx, TracksAction::Delay(0.1));
        }
        if key(Modifiers::NONE, Key::Q) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        if key(Modifiers::NONE, Key::F) {
            Self::toggle_fullscreen(ctx);
        }
        if key(Modifiers::NONE, Key::Escape) && ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)) {
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
        }
        if key(Modifiers::NONE, Key::L) {
            self.cycle_repeat(ctx);
        }
        if key(Modifiers::NONE, Key::P) {
            self.show_playlist = !self.show_playlist;
        }
        if key(Modifiers::NONE, Key::N) {
            self.next(ctx);
        }
        if key(Modifiers::NONE, Key::B) {
            self.prev(ctx);
        }

        if key(Modifiers::NONE, Key::CloseBracket) {
            self.set_speed(ctx, speed::step(self.speed, true));
        }
        if key(Modifiers::NONE, Key::OpenBracket) {
            self.set_speed(ctx, speed::step(self.speed, false));
        }
        if key(Modifiers::NONE, Key::Backspace) {
            self.set_speed(ctx, 1.0);
        }

        if key(Modifiers::NONE, Key::ArrowUp) {
            self.set_volume(ctx, self.config.volume + VOLUME_STEP, false);
        }
        if key(Modifiers::NONE, Key::ArrowDown) {
            self.set_volume(ctx, self.config.volume - VOLUME_STEP, false);
        }
        if key(Modifiers::NONE, Key::M) {
            self.set_volume(ctx, self.config.volume, !self.config.muted);
        }

        let (small, large) = (self.config.seek_small, self.config.seek_large);
        let mut osd = None;
        if let Some(p) = self.player.as_mut() {
            if key(Modifiers::NONE, Key::Space) {
                p.toggle_pause();
                osd = Some(if p.state() == PlayState::Playing { "再生" } else { "一時停止" }.to_owned());
            }
            for (m, k, d) in [
                (Modifiers::SHIFT, Key::ArrowLeft, -large),
                (Modifiers::SHIFT, Key::ArrowRight, large),
                (Modifiers::NONE, Key::ArrowLeft, -small),
                (Modifiers::NONE, Key::ArrowRight, small),
            ] {
                if key(m, k) {
                    p.seek_relative(d);
                    osd = Some(format!("{}{}秒", if d > 0.0 { "+" } else { "−" }, d.abs()));
                }
            }
        }
        if let Some(text) = osd {
            self.show_osd(ctx, text, false);
        }
    }

    /// Resizes the window to the video's aspect ratio so there are no bars:
    /// when a file opens, and after the user finishes resizing (size stable
    /// for a moment). Only the height changes, unless that would overflow the
    /// monitor.
    fn fit_window_to_video(&mut self, ctx: &egui::Context, size: egui::Vec2, now: f64) {
        const SETTLE: f64 = 0.35;
        if size != self.fit_size {
            self.fit_size = size;
            self.fit_changed_at = now;
        }
        let aspect = self.video_aspect;
        if !(aspect.is_finite() && aspect > 0.0) || self.fit_done == Some((size, aspect)) {
            return;
        }
        let waited = now - self.fit_changed_at;
        if waited < SETTLE {
            ctx.request_repaint_after(Duration::from_secs_f64(SETTLE - waited + 0.01));
            return;
        }
        self.fit_done = Some((size, aspect));
        let fitted = window::fit_rect(Rect::from_min_size(egui::Pos2::ZERO, size), aspect);
        if size.x - fitted.width() < 2.0 && size.y - fitted.height() < 2.0 {
            return; // already (almost) exact
        }
        let mut target = egui::vec2(size.x, (size.x / aspect).round());
        if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
            let max_h = monitor.y * 0.9;
            if target.y > max_h {
                target = egui::vec2((max_h * aspect).round(), max_h.round());
            }
        }
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(target));
    }

    /// Subtitle files attach to the current video; everything else replaces the
    /// playlist (or is appended while the playlist panel is open).
    fn handle_dropped(&mut self, ctx: &egui::Context, dropped: Vec<PathBuf>) {
        let (subs, media): (Vec<PathBuf>, Vec<PathBuf>) = dropped.into_iter().partition(|p| is_subtitle_file(p));
        if !subs.is_empty() {
            self.load_subtitles(ctx, subs);
        }
        if !media.is_empty() {
            self.open_paths(ctx, media, self.show_playlist);
        }
    }

    fn update_video(&mut self, ctx: &egui::Context) {
        let Some(p) = self.player.as_mut() else { return };
        if let Some(frame) = p.update() {
            self.video_aspect = frame.display_aspect();
            self.current = Some(Arc::new(frame));
        }
        if let Some(e) = p.take_error() {
            self.show_osd(ctx, e, true);
        }
        if let Some(d) = self.player.as_ref().and_then(Player::next_wakeup) {
            ctx.request_repaint_after(d);
        }
        self.handle_end(ctx);
    }

    // ---- Painting -------------------------------------------------------------

    fn paint_empty_state(&mut self, ui: &mut egui::Ui, window: Rect) {
        let center = window.center();
        let p = ui.painter();
        p.circle_filled(center - vec2(0.0, 40.0), 44.0, color::ACCENT.gamma_multiply(0.12));
        p.text(
            center - vec2(0.0, 40.0),
            Align2::CENTER_CENTER,
            icon::FILM_SLATE,
            icon_font(40.0),
            color::ACCENT,
        );
        p.text(
            center + vec2(0.0, 26.0),
            Align2::CENTER_CENTER,
            "動画をドロップして再生",
            FontId::proportional(18.0),
            color::TEXT,
        );

        // Primary: open file(s). Secondary (outlined): open a folder.
        let buttons = [
            (-94.0, icon::FILE_VIDEO, "ファイルを開く", DialogKind::Open, true),
            (94.0, icon::FOLDER_OPEN, "フォルダを開く", DialogKind::OpenFolder, false),
        ];
        for (dx, glyph, label, kind, primary) in buttons {
            let btn_rect = Rect::from_center_size(center + vec2(dx, 74.0), vec2(176.0, 38.0));
            let resp = ui.interact(btn_rect, Id::new(("empty_open", label)), Sense::click());
            let radius = CornerRadius::same(size::BUTTON_RADIUS + 2);
            let p = ui.painter();
            let text_color = if primary {
                let fill = if resp.hovered() { color::ACCENT_HOVER } else { color::ACCENT };
                p.rect_filled(btn_rect, radius, fill);
                Color32::from_rgb(0x10, 0x12, 0x1a)
            } else {
                if resp.hovered() {
                    p.rect_filled(btn_rect, radius, color::SURFACE_HOVER);
                }
                p.rect_stroke(btn_rect, radius, egui::Stroke::new(1.0, color::ACCENT), egui::StrokeKind::Inside);
                color::ACCENT
            };
            p.text(btn_rect.center(), Align2::CENTER_CENTER, format!("{glyph}  {label}"), FontId::proportional(15.0), text_color);
            if resp.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
            if resp.clicked() {
                self.open_dialog(ui.ctx(), kind);
            }
        }
        let p = ui.painter();
        p.text(
            center + vec2(0.0, 112.0),
            Align2::CENTER_CENTER,
            "O でファイル、Shift+O でフォルダを開けます",
            FontId::proportional(12.5),
            color::TEXT_DIM,
        );
    }

    fn paint_osd(&mut self, ui: &egui::Ui, window: Rect) {
        let now = ui.input(|i| i.time);
        let Some(osd) = &self.osd else { return };
        let remaining = osd.until - now;
        let alpha = ui.ctx().animate_bool_with_time(Id::new("osd"), remaining > 0.0, motion::FADE);
        if remaining <= 0.0 && alpha <= 0.0 {
            self.osd = None;
            return;
        }
        if remaining > 0.0 {
            ui.ctx().request_repaint_after(Duration::from_secs_f64(remaining));
        }
        let font = FontId::proportional(15.0);
        let text_color = if osd.error { color::DANGER } else { color::TEXT };
        let galley = ui.painter().layout(osd.text.clone(), font, text_color, window.width() - 80.0);
        let pad = vec2(16.0, 10.0);
        let rect = Rect::from_center_size(
            pos2(window.center().x, window.top() + size::TITLEBAR_HEIGHT + 28.0),
            galley.size() + pad * 2.0,
        );
        let p = ui.painter().clone().with_layer_id(egui::LayerId::new(egui::Order::Foreground, Id::new("osd_layer")));
        let fill = color::SURFACE.gamma_multiply(alpha);
        p.add(egui::epaint::RectShape::filled(rect, CornerRadius::same(size::PANEL_RADIUS), fill));
        p.galley(rect.min + pad, galley, text_color.gamma_multiply(alpha));
    }

    fn handle_wheel(&mut self, ctx: &egui::Context) {
        // Mouse notches arrive in lines; touchpads in points (accumulate).
        let lines: f32 = ctx.input(|i| {
            i.raw
                .events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::MouseWheel { unit, delta, .. } => Some(match unit {
                        egui::MouseWheelUnit::Line => delta.y,
                        egui::MouseWheelUnit::Point => delta.y / 50.0,
                        egui::MouseWheelUnit::Page => delta.y * 3.0,
                    }),
                    _ => None,
                })
                .sum()
        });
        self.wheel_accum += lines;
        let steps = self.wheel_accum.trunc();
        if steps != 0.0 {
            self.wheel_accum -= steps;
            self.set_volume(ctx, self.config.volume + steps * VOLUME_STEP, false);
        }
    }
}

impl eframe::App for RvpApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4] // fully transparent: the rounded window shape is painted by us
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if std::mem::take(&mut self.inject_release) {
            let pos = ctx.input(|i| i.pointer.latest_pos()).unwrap_or_default();
            raw_input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: ctx.input(|i| i.modifiers),
            });
        }
    }

    fn on_exit(&mut self) {
        self.remember_position();
        self.resume.save();
        self.config.save();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_input(&ctx);
        self.update_video(&ctx);
        self.persist(&ctx);

        let window = ui.max_rect();
        let (fullscreen, maximized, now) = ctx.input(|i| {
            (
                i.viewport().fullscreen.unwrap_or(false),
                i.viewport().maximized.unwrap_or(false),
                i.time,
            )
        });
        let radius = if fullscreen || maximized { 0 } else { size::WINDOW_RADIUS };
        let window_radius = CornerRadius::same(radius);

        if !fullscreen && !maximized && self.player.is_some() {
            self.fit_window_to_video(&ctx, window.size(), now);
        }

        // Window body.
        ui.painter().rect_filled(window, window_radius, color::BG);

        // Background interaction: drag to move the window, double-click for fullscreen.
        let bg = ui.interact(window, Id::new("window_bg"), Sense::click_and_drag());
        if bg.double_clicked() {
            Self::toggle_fullscreen(&ctx);
        } else if bg.drag_started() && !fullscreen {
            ctx.send_viewport_cmd(ViewportCommand::StartDrag);
            self.inject_release = true;
        }
        // Mouse wheel over the video adjusts volume (one notch ≈ 5%).
        if bg.hovered() && self.player.is_some() {
            self.handle_wheel(&ctx);
        }

        // Video.
        if let Some(frame) = &self.current {
            let video = window::fit_rect(window, self.video_aspect);
            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                video,
                VideoPaint {
                    frame: frame.clone(),
                    rect: video,
                    radius: f32::from(radius),
                },
            ));
        }

        // Auto-hiding chrome.
        let hover_pos = ctx.input(|i| i.pointer.hover_pos());
        let bar = controls::bar_rect(window);
        let panel = playlist_panel::panel_rect(window);
        // Hide after a short idle period (playing or paused), unless the pointer
        // rests on the controls, a menu is open, or nothing is loaded.
        let over_chrome =
            hover_pos.is_some_and(|p| bar.expand(8.0).contains(p) || (self.show_playlist && panel.contains(p)));
        let keep = self.player.is_none() || over_chrome || egui::Popup::is_any_open(&ctx);
        let idle = now - self.last_activity;
        let visible = keep || idle < motion::HIDE_AFTER;
        if visible && !keep {
            ctx.request_repaint_after(Duration::from_secs_f64(motion::HIDE_AFTER - idle + 0.01));
        }
        let alpha = ctx.animate_bool_with_time(Id::new("chrome"), visible, motion::FADE);
        if !visible && alpha <= 0.0 && hover_pos.is_some() {
            ctx.set_cursor_icon(CursorIcon::None);
        }

        if self.player.is_none() {
            self.paint_empty_state(ui, window);
        }

        let title = self.player.as_ref().map_or("rvp", |p| p.info().title.as_str()).to_owned();
        let tb = window::title_bar(ui, window, &title, alpha, fullscreen, f32::from(radius));
        if tb.open {
            self.open_dialog(&ctx, DialogKind::Open);
        }
        if tb.open_folder {
            self.open_dialog(&ctx, DialogKind::OpenFolder);
        }

        // Subtitles sit above the video, below the chrome; they move up while the bar is shown.
        if let Some(text) = self.player.as_ref().and_then(Player::subtitle_text) {
            let video = window::fit_rect(window, self.video_aspect);
            paint_subtitle(ui, &text, video, window, controls::bar_rect(window), alpha);
        }

        if let Some(p) = self.player.as_ref() {
            let model = ControlsModel {
                state: p.state(),
                position: p.position(),
                duration: p.info().duration,
                fullscreen,
                volume: self.config.volume,
                muted: self.config.muted,
                has_audio: p.info().audio_codec.is_some(),
                has_prev: true, // restarts the current file at least
                has_next: self.playlist.has_next(),
                playlist_open: self.show_playlist,
                speed: self.speed,
                tracks: Self::tracks_model(p),
                repeat: self.playlist.repeat,
            };
            let a = controls::control_bar(ui, window, &model, alpha);
            if let Some(p) = self.player.as_mut() {
                if a.toggle_pause {
                    p.toggle_pause();
                }
                match a.seek {
                    Some(SeekEvent::Scrub(t)) => p.seek(t, false),
                    Some(SeekEvent::Commit(t)) => p.seek(t, true),
                    None => {}
                }
            }
            if a.toggle_fullscreen {
                Self::toggle_fullscreen(&ctx);
            }
            if a.volume.toggle_mute {
                self.set_volume(&ctx, self.config.volume, !self.config.muted);
            } else if let Some(v) = a.volume.set {
                self.set_volume(&ctx, v, false);
            }
            if a.prev {
                self.prev(&ctx);
            }
            if a.next {
                self.next(&ctx);
            }
            if a.toggle_playlist {
                self.show_playlist = !self.show_playlist;
            }
            if let Some(s) = a.speed {
                self.set_speed(&ctx, s);
            }
            if let Some(t) = a.tracks {
                self.handle_tracks_action(&ctx, t);
            }
            if a.cycle_repeat {
                self.cycle_repeat(&ctx);
            }
        }

        let panel_alpha = ctx.animate_bool_with_time(Id::new("playlist_panel"), self.show_playlist, motion::FADE);
        if panel_alpha > 0.0 {
            let model = PlaylistModel {
                items: self.playlist.items(),
                current: self.playlist.current_index(),
                repeat: self.playlist.repeat,
                shuffle: self.playlist.shuffle(),
            };
            let actions = playlist_panel::playlist_panel(ui, window, &model, panel_alpha);
            for a in actions {
                self.handle_playlist_action(&ctx, a);
            }
        }

        self.paint_osd(ui, window);

        let drop_alpha = ctx.animate_bool_with_time(Id::new("drop_hint"), self.drag_hover, motion::FADE);
        if drop_alpha > 0.0 {
            paint_drop_hint(ui, window, f32::from(radius), drop_alpha, self.show_playlist);
        }

        if !fullscreen && !maximized {
            if window::resize_grips(ui, window) {
                self.inject_release = true;
            }
        }

        if let Some(shot) = &mut self.debug_shot {
            shot.tick(&ctx);
        }
    }
}

/// Draws subtitle text centred near the bottom of the video with a dark outline.
fn paint_subtitle(ui: &egui::Ui, text: &str, video: Rect, window: Rect, bar: Rect, chrome_alpha: f32) {
    let visible = video.intersect(window);
    let size = (visible.height() * 0.052).clamp(18.0, 60.0);
    let mut job = egui::text::LayoutJob::simple(
        text.to_owned(),
        FontId::proportional(size),
        Color32::WHITE,
        visible.width() * 0.9,
    );
    job.halign = egui::Align::Center;
    let galley = ui.painter().layout_job(job);

    let rest = visible.bottom() - visible.height() * 0.05;
    let raised = bar.top() - 14.0;
    let bottom = egui::lerp(rest..=rest.min(raised), chrome_alpha);
    let pos = pos2(visible.center().x, bottom - galley.size().y);

    let p = ui.painter();
    let outline = (size * 0.07).max(1.5);
    let shadow = Color32::from_black_alpha(230);
    for (dx, dy) in [(-1.0, -1.0), (0.0, -1.0), (1.0, -1.0), (-1.0, 0.0), (1.0, 0.0), (-1.0, 1.0), (0.0, 1.0), (1.0, 1.0)] {
        p.galley_with_override_text_color(pos + vec2(dx, dy) * outline, galley.clone(), shadow);
    }
    p.galley(pos, galley, Color32::WHITE);
}

/// Hooks up Wayland drag-and-drop when running on Wayland.
fn start_wayland_dnd(cc: &eframe::CreationContext<'_>) -> Option<Receiver<crate::wayland_dnd::DndEvent>> {
    use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
    let handle = cc.display_handle().ok()?;
    let RawDisplayHandle::Wayland(wl) = handle.as_raw() else { return None };
    let (tx, rx) = unbounded();
    // SAFETY: the display belongs to the main window, which lives as long as the app.
    match unsafe { crate::wayland_dnd::start(wl.display, tx, cc.egui_ctx.clone()) } {
        Ok(()) => Some(rx),
        Err(e) => {
            log::warn!("wayland drag-and-drop unavailable: {e:#}");
            None
        }
    }
}

/// Overlay shown while files are dragged over the window.
fn paint_drop_hint(ui: &egui::Ui, window: Rect, radius: f32, alpha: f32, append: bool) {
    let p = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Foreground, Id::new("drop_hint_layer")));
    let r = CornerRadius::same(radius as u8);
    p.rect_filled(window, r, Color32::from_black_alpha((150.0 * alpha) as u8));
    let inner = window.shrink(14.0);
    p.rect_stroke(
        inner,
        CornerRadius::same(size::PANEL_RADIUS),
        egui::Stroke::new(2.0, color::ACCENT.gamma_multiply(alpha)),
        egui::StrokeKind::Inside,
    );
    let c = window.center();
    p.text(c - vec2(0.0, 22.0), Align2::CENTER_CENTER, icon::DOWNLOAD_SIMPLE, icon_font(42.0), color::ACCENT.gamma_multiply(alpha));
    let label = if append { "ドロップしてプレイリストに追加" } else { "ドロップして再生" };
    p.text(c + vec2(0.0, 26.0), Align2::CENTER_CENTER, label, FontId::proportional(18.0), color::TEXT.gamma_multiply(alpha));
}
