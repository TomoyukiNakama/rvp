//! Playback-speed pill button with a preset popup.

use egui::{Align2, CornerRadius, FontId, Popup, RectAlign, Sense, Ui, vec2};

use crate::theme::{color, size};

pub const PRESETS: &[f32] = &[0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0, 4.0];

pub fn format_speed(speed: f32) -> String {
    // 1x, 1.5x, 1.25x — no trailing zeros.
    let s = format!("{speed:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    format!("{s}x")
}

/// Next preset above (`up`) or below the current speed.
pub fn step(speed: f32, up: bool) -> f32 {
    if up {
        PRESETS.iter().copied().find(|&p| p > speed + 1e-3).unwrap_or(speed)
    } else {
        PRESETS.iter().rev().copied().find(|&p| p < speed - 1e-3).unwrap_or(speed)
    }
}

/// Returns a newly chosen speed.
pub fn speed_button(ui: &mut Ui, speed: f32) -> Option<f32> {
    let label = format_speed(speed);
    let font = FontId::monospace(13.0);
    let galley = ui.painter().layout_no_wrap(label, font, color::TEXT);
    let w = (galley.size().x + 16.0).max(44.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(w, size::ICON_BUTTON), Sense::click());
    let pill = egui::Rect::from_center_size(rect.center(), vec2(w, 26.0));
    let p = ui.painter();
    let non_default = (speed - 1.0).abs() > 1e-3;
    if resp.hovered() || Popup::is_id_open(ui.ctx(), Popup::default_response_id(&resp)) {
        p.rect_filled(pill, CornerRadius::same(13), color::SURFACE_HOVER);
    }
    if non_default {
        p.rect_stroke(pill, CornerRadius::same(13), egui::Stroke::new(1.0, color::ACCENT), egui::StrokeKind::Inside);
    }
    let text_color = if non_default { color::ACCENT } else { color::TEXT };
    p.galley(pill.center() - galley.size() / 2.0, galley, text_color);
    let resp = resp.on_hover_text("再生速度 ([ / ], Backspace でリセット)");

    let mut chosen = None;
    Popup::menu(&resp).align(RectAlign::TOP).gap(10.0).show(|ui| {
        ui.set_min_width(96.0);
        ui.spacing_mut().item_spacing.y = 2.0;
        for &preset in PRESETS.iter().rev() {
            let current = (preset - speed).abs() < 1e-3;
            let (r, item) = ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::click());
            if current {
                ui.painter().rect_filled(r, CornerRadius::same(7), color::ACCENT.gamma_multiply(0.18));
            } else if item.hovered() {
                ui.painter().rect_filled(r, CornerRadius::same(7), color::SURFACE_HOVER);
            }
            let c = if current { color::ACCENT } else { color::TEXT };
            ui.painter().text(r.center(), Align2::CENTER_CENTER, format_speed(preset), FontId::monospace(13.5), c);
            if item.clicked() {
                chosen = Some(preset);
                ui.close();
            }
        }
    });
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting_and_steps() {
        assert_eq!(format_speed(1.0), "1x");
        assert_eq!(format_speed(1.5), "1.5x");
        assert_eq!(format_speed(0.25), "0.25x");
        assert_eq!(step(1.0, true), 1.25);
        assert_eq!(step(1.0, false), 0.75);
        assert_eq!(step(4.0, true), 4.0);
        assert_eq!(step(0.25, false), 0.25);
        assert_eq!(step(1.1, false), 1.0, "off-preset values snap to the neighbour");
    }
}
