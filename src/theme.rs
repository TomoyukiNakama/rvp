//! Design tokens and the egui style derived from them. Dark theme only.

use egui::epaint::Shadow;
use egui::{Color32, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle, Visuals};

pub mod color {
    use egui::Color32;

    /// Window background (behind letterboxing).
    pub const BG: Color32 = Color32::from_rgb(0x0b, 0x0c, 0x10);
    /// Floating panels (control bar, popups) — translucent.
    pub const SURFACE: Color32 = Color32::from_rgba_unmultiplied_const(18, 20, 26, 214);
    pub const SURFACE_HOVER: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 22);
    pub const SURFACE_ACTIVE: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 36);
    pub const BORDER: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 18);

    pub const TEXT: Color32 = Color32::from_rgb(0xec, 0xee, 0xf4);
    pub const TEXT_DIM: Color32 = Color32::from_rgb(0x9a, 0xa0, 0xae);

    pub const ACCENT: Color32 = Color32::from_rgb(0x8b, 0x9c, 0xff);
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0xa8, 0xb5, 0xff);

    pub const TRACK: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 40);

    pub const CLOSE_HOVER: Color32 = Color32::from_rgb(0xe5, 0x48, 0x4d);
    pub const WARN: Color32 = Color32::from_rgb(0xff, 0xb4, 0x5c);
    pub const DANGER: Color32 = Color32::from_rgb(0xff, 0x6b, 0x6b);
}

pub mod size {
    /// Corner radius of the (borderless) window itself.
    pub const WINDOW_RADIUS: u8 = 14;
    pub const PANEL_RADIUS: u8 = 14;
    pub const BUTTON_RADIUS: u8 = 9;
    pub const TITLEBAR_HEIGHT: f32 = 40.0;
    pub const CONTROLBAR_HEIGHT: f32 = 56.0;
    pub const CONTROLBAR_INSET: f32 = 14.0;
    pub const ICON_BUTTON: f32 = 36.0;
    pub const ICON_SIZE: f32 = 20.0;
    /// Width of the invisible resize grips along the window edge.
    pub const RESIZE_GRIP: f32 = 6.0;
}

pub mod motion {
    /// Fade time of auto-hiding chrome.
    pub const FADE: f32 = 0.18;
    /// Idle time before the chrome hides during playback.
    pub const HIDE_AFTER: f64 = 2.0;
}

pub fn panel_shadow() -> Shadow {
    Shadow {
        offset: [0, 6],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(110),
    }
}

pub fn apply(ctx: &egui::Context) {
    let mut v = Visuals::dark();
    v.panel_fill = color::BG;
    v.window_fill = color::SURFACE;
    v.extreme_bg_color = Color32::from_rgb(0x14, 0x16, 0x1c);
    v.faint_bg_color = Color32::from_rgb(0x16, 0x18, 0x1f);
    v.override_text_color = Some(color::TEXT);
    v.hyperlink_color = color::ACCENT;
    v.selection.bg_fill = color::ACCENT.gamma_multiply(0.45);
    v.selection.stroke = Stroke::new(1.0, color::ACCENT);
    v.window_corner_radius = CornerRadius::same(size::PANEL_RADIUS);
    v.menu_corner_radius = CornerRadius::same(size::BUTTON_RADIUS + 2);
    v.window_shadow = panel_shadow();
    v.popup_shadow = panel_shadow();
    v.window_stroke = Stroke::new(1.0, color::BORDER);

    let r = CornerRadius::same(size::BUTTON_RADIUS);
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = r;
        w.bg_stroke = Stroke::NONE;
    }
    v.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    v.widgets.inactive.bg_fill = color::SURFACE_HOVER;
    v.widgets.hovered.weak_bg_fill = color::SURFACE_HOVER;
    v.widgets.hovered.bg_fill = color::SURFACE_HOVER;
    v.widgets.active.weak_bg_fill = color::SURFACE_ACTIVE;
    v.widgets.active.bg_fill = color::SURFACE_ACTIVE;
    v.widgets.hovered.expansion = 0.0;
    v.widgets.active.expansion = 0.0;

    ctx.set_visuals(v);

    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(12.0, 6.0);
        s.spacing.menu_margin = Margin::same(8);
        s.text_styles = [
            (TextStyle::Heading, FontId::new(22.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(14.5, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(14.5, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(13.5, FontFamily::Monospace)),
        ]
        .into();
    });
}
