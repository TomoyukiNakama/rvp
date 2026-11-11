use std::path::PathBuf;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, unbounded};
use egui::{
    Align2, Color32, CornerRadius, CursorIcon, FontId, Id, Key, Modifiers, Rect, Sense,
    TextureHandle, TextureOptions, ViewportCommand, pos2, vec2,
};
use egui_phosphor::regular as icon;

use crate::player::{PlayState, Player};
use crate::theme::{self, color, motion, size};
use crate::ui::controls::{self, ControlsModel};
use crate::ui::seekbar::SeekEvent;
use crate::ui::widgets::icon_font;
use crate::ui::window;

const SEEK_SMALL: f64 = 5.0;
const SEEK_LARGE: f64 = 60.0;

struct Osd {
    text: String,
    until: f64,
    error: bool,
}

pub struct RvpApp {
    player: Option<Player>,
    texture: Option<TextureHandle>,
    video_aspect: f32,
    last_activity: f64,
    osd: Option<Osd>,
    open_tx: Sender<PathBuf>,
    open_rx: Receiver<PathBuf>,
    dialog_open: bool,
    debug_shot: Option<crate::debug_shot::DebugShot>,
}

impl RvpApp {
    pub fn new(cc: &eframe::CreationContext<'_>, files: Vec<PathBuf>) -> Self {
        crate::fonts::install(&cc.egui_ctx);
        theme::apply(&cc.egui_ctx);

        let (open_tx, open_rx) = unbounded();
        let mut app = Self {
            player: None,
            texture: None,
            video_aspect: 16.0 / 9.0,
            last_activity: 0.0,
            osd: None,
            open_tx,
            open_rx,
            dialog_open: false,
            debug_shot: crate::debug_shot::DebugShot::from_env(),
        };
        // Playlist support comes later; for now open the first argument.
        if let Some(first) = files.into_iter().next() {
            app.open(&cc.egui_ctx, first);
        }
        app
    }

    fn open(&mut self, ctx: &egui::Context, path: PathBuf) {
        // Drop the old player first so its threads stop before new ones start.
        self.player = None;
        match Player::open(&path, ctx.clone()) {
            Ok(p) => {
                let info = p.info();
                self.video_aspect = if info.height > 0 {
                    info.width as f32 / info.height as f32
                } else {
                    16.0 / 9.0
                };
                ctx.send_viewport_cmd(ViewportCommand::Title(format!("{} — rvp", info.title)));
                self.player = Some(p);
                self.texture = None;
            }
            Err(e) => {
                log::error!("{e:#}");
                self.show_osd(ctx, format!("{e:#}"), true);
            }
        }
    }

    fn show_osd(&mut self, ctx: &egui::Context, text: String, error: bool) {
        let now = ctx.input(|i| i.time);
        self.osd = Some(Osd {
            text,
            until: now + if error { 5.0 } else { 1.2 },
            error,
        });
    }

    fn open_dialog(&mut self, ctx: &egui::Context) {
        if self.dialog_open {
            return;
        }
        self.dialog_open = true;
        let tx = self.open_tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let picked = rfd::FileDialog::new()
                .set_title("動画を開く")
                .add_filter(
                    "動画",
                    &["mp4", "mkv", "webm", "mov", "avi", "m4v", "ts", "m2ts", "flv", "wmv", "mpg", "mpeg", "ogv"],
                )
                .add_filter("すべてのファイル", &["*"])
                .pick_file();
            if let Some(p) = picked {
                let _ = tx.send(p);
            }
            // Signal completion (an empty path means "cancelled").
            let _ = tx.send(PathBuf::new());
            ctx.request_repaint();
        });
    }

    fn toggle_fullscreen(ctx: &egui::Context) {
        let fs = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(!fs));
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        // Files from the dialog thread.
        while let Ok(p) = self.open_rx.try_recv() {
            if p.as_os_str().is_empty() {
                self.dialog_open = false;
            } else {
                self.open(ctx, p);
            }
        }

        // Drag & drop.
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).collect());
        if let Some(first) = dropped.into_iter().next() {
            self.open(ctx, first);
        }

        let (now, moved) = ctx.input(|i| (i.time, i.pointer.delta() != egui::Vec2::ZERO || i.pointer.any_down()));
        if moved {
            self.last_activity = now;
        }

        let key = |m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_key(m, k));

        if key(Modifiers::NONE, Key::O) {
            self.open_dialog(ctx);
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

        let mut osd = None;
        if let Some(p) = self.player.as_mut() {
            if key(Modifiers::NONE, Key::Space) {
                p.toggle_pause();
                osd = Some(if p.state() == PlayState::Playing { "再生" } else { "一時停止" }.to_owned());
            }
            for (m, k, d) in [
                (Modifiers::SHIFT, Key::ArrowLeft, -SEEK_LARGE),
                (Modifiers::SHIFT, Key::ArrowRight, SEEK_LARGE),
                (Modifiers::NONE, Key::ArrowLeft, -SEEK_SMALL),
                (Modifiers::NONE, Key::ArrowRight, SEEK_SMALL),
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

    fn update_video(&mut self, ctx: &egui::Context) {
        let Some(p) = self.player.as_mut() else { return };
        if let Some(frame) = p.update() {
            self.video_aspect = frame.display_aspect();
            let image = egui::ColorImage::from_rgba_unmultiplied([frame.width, frame.height], &frame.rgba);
            match &mut self.texture {
                Some(t) => t.set(image, TextureOptions::LINEAR),
                None => self.texture = Some(ctx.load_texture("video", image, TextureOptions::LINEAR)),
            }
        }
        if let Some(e) = p.take_error() {
            self.show_osd(ctx, e, true);
        }
        if let Some(d) = self.player.as_ref().and_then(Player::next_wakeup) {
            ctx.request_repaint_after(d);
        }
    }

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

        let btn_rect = Rect::from_center_size(center + vec2(0.0, 74.0), vec2(176.0, 38.0));
        let resp = ui.interact(btn_rect, Id::new("empty_open"), Sense::click());
        let fill = if resp.hovered() { color::ACCENT_HOVER } else { color::ACCENT };
        let p = ui.painter();
        p.rect_filled(btn_rect, CornerRadius::same(size::BUTTON_RADIUS + 2), fill);
        p.text(
            btn_rect.center(),
            Align2::CENTER_CENTER,
            format!("{}  ファイルを開く", icon::FOLDER_OPEN),
            FontId::proportional(15.0),
            Color32::from_rgb(0x10, 0x12, 0x1a),
        );
        if resp.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        if resp.clicked() {
            self.open_dialog(ui.ctx());
        }
        p.text(
            center + vec2(0.0, 112.0),
            Align2::CENTER_CENTER,
            "O キーでも開けます",
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
}

impl eframe::App for RvpApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4] // fully transparent: the rounded window shape is painted by us
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_input(&ctx);
        self.update_video(&ctx);

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

        // Window body.
        ui.painter().rect_filled(window, window_radius, color::BG);

        // Background interaction: drag to move the window, double-click for fullscreen.
        let bg = ui.interact(window, Id::new("window_bg"), Sense::click_and_drag());
        if bg.double_clicked() {
            Self::toggle_fullscreen(&ctx);
        } else if bg.drag_started() && !fullscreen {
            ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        }

        // Video.
        if let Some(tex) = &self.texture {
            let video = window::fit_rect(window, self.video_aspect);
            let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            ui.painter().add(
                egui::epaint::RectShape::filled(video, window_radius, Color32::WHITE).with_texture(tex.id(), uv),
            );
        }

        // Auto-hiding chrome.
        let hover_pos = ctx.input(|i| i.pointer.hover_pos());
        let bar = controls::bar_rect(window);
        let over_chrome = hover_pos.is_some_and(|p| {
            bar.expand(8.0).contains(p) || p.y < window.top() + size::TITLEBAR_HEIGHT
        });
        let playing = self.player.as_ref().is_some_and(|p| p.state() == PlayState::Playing);
        let idle = now - self.last_activity;
        let visible = !playing || over_chrome || idle < motion::HIDE_AFTER;
        if visible && playing && !over_chrome {
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
        let tb = window::title_bar(ui, window, &title, alpha, fullscreen);
        if tb.open {
            self.open_dialog(&ctx);
        }

        if let Some(p) = self.player.as_mut() {
            let model = ControlsModel {
                state: p.state(),
                position: p.position(),
                duration: p.info().duration,
                fullscreen,
            };
            let a = controls::control_bar(ui, window, &model, alpha);
            if a.toggle_pause {
                p.toggle_pause();
            }
            match a.seek {
                Some(SeekEvent::Scrub(t)) => p.seek(t, false),
                Some(SeekEvent::Commit(t)) => p.seek(t, true),
                None => {}
            }
            if a.toggle_fullscreen {
                Self::toggle_fullscreen(&ctx);
            }
        }

        self.paint_osd(ui, window);

        if !fullscreen && !maximized {
            window::resize_grips(ui, window);
        }

        if let Some(shot) = &mut self.debug_shot {
            shot.tick(&ctx);
        }

    }
}
