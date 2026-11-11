//! Volume control: speaker button (mute toggle) + compact custom slider.

use egui::{CornerRadius, Rect, Sense, Stroke, Ui, pos2, vec2};
use egui_phosphor::regular as icon;

use super::widgets::icon_button;
use crate::theme::color;

pub const MAX_VOLUME: f32 = 1.5;
const SLIDER_WIDTH: f32 = 84.0;

pub fn speaker_icon(volume: f32, muted: bool) -> &'static str {
    if muted || volume <= 0.0 {
        icon::SPEAKER_X
    } else if volume < 0.5 {
        icon::SPEAKER_LOW
    } else {
        icon::SPEAKER_HIGH
    }
}

#[derive(Default)]
pub struct VolumeActions {
    pub toggle_mute: bool,
    pub set: Option<f32>,
}

/// Lays out right-to-left: slider first, then the speaker button.
pub fn volume_control(ui: &mut Ui, volume: f32, muted: bool) -> VolumeActions {
    let mut a = VolumeActions::default();
    if let Some(v) = slider(ui, volume, muted) {
        a.set = Some(v);
    }
    let tip = if muted { "ミュート解除 (M)" } else { "ミュート (M)" };
    if icon_button(ui, speaker_icon(volume, muted), tip).clicked() {
        a.toggle_mute = true;
    }
    a
}

fn slider(ui: &mut Ui, volume: f32, muted: bool) -> Option<f32> {
    let (rect, resp) = ui.allocate_exact_size(vec2(SLIDER_WIDTH, 22.0), Sense::click_and_drag());
    let mut new = None;
    if let Some(p) = resp.interact_pointer_pos()
        && (resp.dragged() || resp.clicked() || resp.drag_started())
    {
        let t = ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        // Snap to 100% when close, since that's the "unity gain" sweet spot.
        let mut v = t * MAX_VOLUME;
        if (v - 1.0).abs() < 0.03 {
            v = 1.0;
        }
        new = Some(v);
    }
    if resp.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            new = Some((volume + scroll * 0.002).clamp(0.0, MAX_VOLUME));
        }
    }

    let shown = new.unwrap_or(volume);
    let active = resp.hovered() || resp.dragged();
    let t = ui.ctx().animate_bool_with_time(resp.id.with("hover"), active, 0.12);
    let thickness = egui::lerp(4.0..=6.0, t);
    let track = Rect::from_center_size(rect.center(), vec2(rect.width(), thickness));
    let r = CornerRadius::same((thickness / 2.0).round() as u8);
    let p = ui.painter();
    p.rect_filled(track, r, color::TRACK);

    let frac = (shown / MAX_VOLUME).clamp(0.0, 1.0);
    let fill = if muted {
        color::TEXT_DIM
    } else if shown > 1.0 {
        color::WARN
    } else {
        color::ACCENT
    };
    if frac > 0.0 {
        let mut filled = track;
        filled.set_right(track.left() + track.width() * frac);
        p.rect_filled(filled, r, fill);
    }
    // 100% tick.
    let x100 = track.left() + track.width() / MAX_VOLUME;
    p.line_segment(
        [pos2(x100, track.top() - 3.0), pos2(x100, track.bottom() + 3.0)],
        Stroke::new(1.0, color::TEXT_DIM.gamma_multiply(0.6)),
    );
    if t > 0.0 {
        let c = pos2(track.left() + track.width() * frac, track.center().y);
        p.circle_filled(c, 6.0 * t, egui::Color32::WHITE);
    }

    resp.on_hover_text(format!("音量 {:.0}%", shown * 100.0));
    new
}
