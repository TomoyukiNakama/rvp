//! Small custom widgets shared by the chrome.

use egui::{Align2, Color32, CornerRadius, FontFamily, FontId, Response, Sense, Ui, vec2};

use crate::theme::{color, size};

pub fn icon_font(px: f32) -> FontId {
    FontId::new(px, FontFamily::Name("phosphor".into()))
}

/// A square, borderless icon button with a rounded hover background.
pub fn icon_button(ui: &mut Ui, icon: &str, tooltip: &str) -> Response {
    icon_button_ex(ui, icon, tooltip, size::ICON_SIZE, None)
}

pub fn icon_button_ex(
    ui: &mut Ui,
    icon: &str,
    tooltip: &str,
    icon_px: f32,
    hover_fill: Option<Color32>,
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(size::ICON_BUTTON, size::ICON_BUTTON), Sense::click());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let radius = CornerRadius::same(size::BUTTON_RADIUS);
        let mut fg = color::TEXT;
        if resp.is_pointer_button_down_on() {
            p.rect_filled(rect, radius, hover_fill.unwrap_or(color::SURFACE_ACTIVE));
        } else if resp.hovered() {
            p.rect_filled(rect, radius, hover_fill.unwrap_or(color::SURFACE_HOVER));
        } else {
            fg = color::TEXT.gamma_multiply(0.92);
        }
        p.text(rect.center(), Align2::CENTER_CENTER, icon, icon_font(icon_px), fg);
    }
    if tooltip.is_empty() {
        resp
    } else {
        resp.on_hover_text(tooltip)
    }
}

/// `1:02:03` or `2:03`.
pub fn format_time(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    let (h, m, s) = (s / 3600, (s / 60) % 60, s % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}
