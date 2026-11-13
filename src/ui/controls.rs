//! Floating control bar at the bottom of the window.

use egui::{Align, CornerRadius, Frame, Layout, Margin, Rect, RichText, Stroke, Ui, UiBuilder, vec2};
use egui_phosphor::regular as icon;

use super::seekbar::{SeekEvent, seek_bar};
use super::speed::speed_button;
use super::tracks::{TracksAction, TracksModel, tracks_button};
use super::volume::{VolumeActions, volume_control};
use super::widgets::{format_time, icon_button, icon_button_ex};
use crate::player::PlayState;
use crate::theme::{color, panel_shadow, size};

pub struct ControlsModel<'a> {
    pub state: PlayState,
    pub position: f64,
    pub duration: Option<f64>,
    pub fullscreen: bool,
    pub volume: f32,
    pub muted: bool,
    pub has_audio: bool,
    pub has_prev: bool,
    pub has_next: bool,
    pub playlist_open: bool,
    pub speed: f32,
    pub tracks: TracksModel<'a>,
    pub repeat: crate::playlist::Repeat,
}

#[derive(Default)]
pub struct ControlsActions {
    pub toggle_pause: bool,
    pub seek: Option<SeekEvent>,
    pub toggle_fullscreen: bool,
    pub volume: VolumeActions,
    pub prev: bool,
    pub next: bool,
    pub toggle_playlist: bool,
    pub speed: Option<f32>,
    pub tracks: Option<TracksAction>,
    pub cycle_repeat: bool,
}

/// Rect the control bar occupies (used for hover detection by the caller).
pub fn bar_rect(window: Rect) -> Rect {
    let inset = size::CONTROLBAR_INSET;
    let width = (window.width() - inset * 2.0).min(980.0);
    Rect::from_center_size(
        egui::pos2(window.center().x, window.bottom() - inset - size::CONTROLBAR_HEIGHT / 2.0),
        vec2(width, size::CONTROLBAR_HEIGHT),
    )
}

pub fn control_bar(ui: &mut Ui, window: Rect, m: &ControlsModel<'_>, alpha: f32) -> ControlsActions {
    let mut a = ControlsActions::default();
    if alpha <= 0.0 {
        return a;
    }
    let rect = bar_rect(window);
    let mut child = ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)));
    child.multiply_opacity(alpha);
    let ui = &mut child;

    Frame::new()
        .fill(color::SURFACE)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(CornerRadius::same(size::PANEL_RADIUS))
        .shadow(panel_shadow())
        .inner_margin(Margin::symmetric(10, 0))
        .show(ui, |ui| {
            ui.set_min_size(rect.size() - vec2(20.0, 0.0));
            ui.spacing_mut().item_spacing.x = 6.0;

            let (play_icon, tip) = match m.state {
                PlayState::Playing => (icon::PAUSE, t!("controls-pause")),
                PlayState::Paused => (icon::PLAY, t!("controls-play")),
                PlayState::Ended => (icon::ARROW_COUNTER_CLOCKWISE, t!("controls-replay")),
            };
            ui.add_enabled_ui(m.has_prev, |ui| {
                if icon_button(ui, icon::SKIP_BACK, &t!("controls-prev")).clicked() {
                    a.prev = true;
                }
            });
            if icon_button_ex(ui, play_icon, &tip, size::ICON_SIZE + 2.0, None).clicked() {
                a.toggle_pause = true;
            }
            ui.add_enabled_ui(m.has_next, |ui| {
                if icon_button(ui, icon::SKIP_FORWARD, &t!("controls-next")).clicked() {
                    a.next = true;
                }
            });
            ui.add_space(2.0);

            let time = RichText::new(format_time(m.position)).color(color::TEXT).monospace();
            ui.add(egui::Label::new(time).selectable(false));

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let (fs_icon, tip) = if m.fullscreen {
                    (icon::CORNERS_IN, t!("controls-exit-fullscreen"))
                } else {
                    (icon::CORNERS_OUT, t!("controls-fullscreen"))
                };
                if icon_button(ui, fs_icon, &tip).clicked() {
                    a.toggle_fullscreen = true;
                }
                let looping = m.repeat != crate::playlist::Repeat::Off;
                let loop_icon =
                    if m.repeat == crate::playlist::Repeat::One { icon::REPEAT_ONCE } else { icon::REPEAT };
                let tip = format!("{} (L)", m.repeat.label());
                if super::playlist_panel::toggle_icon(ui, loop_icon, &tip, looping).clicked() {
                    a.cycle_repeat = true;
                }
                let pl = icon_button_ex(
                    ui,
                    icon::QUEUE,
                    &t!("controls-playlist"),
                    size::ICON_SIZE,
                    m.playlist_open.then_some(color::SURFACE_ACTIVE),
                );
                if m.playlist_open {
                    ui.painter().rect_filled(
                        egui::Rect::from_center_size(pl.rect.center_bottom() - vec2(0.0, 3.0), vec2(12.0, 2.0)),
                        1,
                        color::ACCENT,
                    );
                }
                if pl.clicked() {
                    a.toggle_playlist = true;
                }
                if m.has_audio {
                    ui.add_space(4.0);
                    a.volume = volume_control(ui, m.volume, m.muted);
                }
                ui.add_space(2.0);
                a.speed = speed_button(ui, m.speed);
                a.tracks = tracks_button(ui, &m.tracks);
                ui.add_space(6.0);
                let total = m.duration.map(format_time).unwrap_or_else(|| "--:--".into());
                ui.add(egui::Label::new(RichText::new(total).color(color::TEXT_DIM).monospace()).selectable(false));

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(6.0);
                    ui.set_max_width(ui.available_width() - 6.0);
                    a.seek = seek_bar(ui, m.position, m.duration);
                });
            });
        });

    a
}
