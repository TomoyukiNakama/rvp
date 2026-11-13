//! Borderless window chrome: title bar, window buttons and resize grips.

use egui::{
    Align, Color32, CursorIcon, Id, Layout, Mesh, Rect, ResizeDirection, Sense, Ui, UiBuilder,
    ViewportCommand, pos2, vec2,
};
use egui_phosphor::regular as icon;

use super::widgets::{icon_button, icon_button_ex, icon_font};
use crate::theme::{color, size};

#[derive(Default)]
pub struct TitleBarActions {
    pub open: bool,
    pub open_folder: bool,
    pub about: bool,
    /// Newly chosen UI language.
    pub language: Option<String>,
}

/// Vertical gradient (top → bottom colour) filling `rect` with its top corners
/// rounded by `radius`, so it never paints outside a rounded window.
pub fn rounded_top_gradient(painter: &egui::Painter, rect: Rect, radius: f32, top: Color32, bottom: Color32) {
    let color_at = |y: f32| {
        let t = ((y - rect.top()) / rect.height()).clamp(0.0, 1.0);
        egui::ecolor::Rgba::from(top) * (1.0 - t) + egui::ecolor::Rgba::from(bottom) * t
    };
    let r = radius.min(rect.width() / 2.0).min(rect.height());
    let mut outline = vec![rect.left_bottom()];
    const STEPS: usize = 10;
    for (cx, from) in [(rect.left() + r, std::f32::consts::PI), (rect.right() - r, 1.5 * std::f32::consts::PI)] {
        for i in 0..=STEPS {
            let a = from + (i as f32 / STEPS as f32) * std::f32::consts::FRAC_PI_2;
            outline.push(pos2(cx + r * a.cos(), rect.top() + r + r * a.sin()));
        }
    }
    outline.push(rect.right_bottom());

    // The shape is convex: a fan from its centre covers it exactly.
    let mut mesh = Mesh::default();
    let c = rect.center();
    mesh.colored_vertex(c, color_at(c.y).into());
    for p in &outline {
        mesh.colored_vertex(*p, color_at(p.y).into());
    }
    for i in 1..outline.len() as u32 {
        mesh.add_triangle(0, i, i + 1);
    }
    painter.add(mesh);
}

/// Paints a horizontal gradient (left color → right color) into `rect`.
pub fn horizontal_gradient(painter: &egui::Painter, rect: Rect, left: Color32, right: Color32) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), left);
    mesh.colored_vertex(rect.right_top(), right);
    mesh.colored_vertex(rect.left_bottom(), left);
    mesh.colored_vertex(rect.right_bottom(), right);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 3, 2);
    painter.add(mesh);
}

/// Title bar overlaid on top of the video. `alpha` fades the whole bar.
pub fn title_bar(ui: &mut Ui, window: Rect, title: &str, alpha: f32, fullscreen: bool, radius: f32) -> TitleBarActions {
    let mut actions = TitleBarActions::default();
    let rect = Rect::from_min_size(window.min, vec2(window.width(), size::TITLEBAR_HEIGHT));

    let mut child = ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)));
    child.multiply_opacity(alpha);
    if alpha <= 0.0 {
        return actions;
    }
    let ui = &mut child;

    let shade = Rect::from_min_size(rect.min, vec2(rect.width(), rect.height() * 1.8));
    rounded_top_gradient(ui.painter(), shade, radius, Color32::from_black_alpha(150), Color32::TRANSPARENT);

    ui.add_space(14.0);
    ui.label(egui::RichText::new(icon::FILM_SLATE).font(icon_font(16.0)).color(color::ACCENT));
    ui.add_space(2.0);
    ui.add(egui::Label::new(egui::RichText::new(title).color(color::TEXT).strong()).truncate().selectable(false));

    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.add_space(6.0);
        ui.spacing_mut().item_spacing.x = 2.0;
        let ctx = ui.ctx().clone();
        if fullscreen {
            if icon_button(ui, icon::CORNERS_IN, &t!("window-exit-fullscreen")).clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
            }
        } else {
            if icon_button_ex(ui, icon::X, &t!("window-close"), size::ICON_SIZE - 2.0, Some(color::CLOSE_HOVER)).clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
            let (max_icon, tip) = if maximized {
                (icon::COPY, t!("window-restore"))
            } else {
                (icon::SQUARE, t!("window-maximize"))
            };
            if icon_button_ex(ui, max_icon, &tip, size::ICON_SIZE - 3.0, None).clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
            }
            if icon_button(ui, icon::MINUS, &t!("window-minimize")).clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
            }
        }
        ui.add_space(4.0);
        if let Some(tag) = super::language::language_button(ui) {
            actions.language = Some(tag);
        }
        if icon_button(ui, icon::INFO, &t!("window-about")).clicked() {
            actions.about = true;
        }
        if icon_button(ui, icon::FOLDER_OPEN, &t!("window-open-folder")).clicked() {
            actions.open_folder = true;
        }
        if icon_button(ui, icon::FILE_VIDEO, &t!("window-open-file")).clicked() {
            actions.open = true;
        }
    });

    actions
}

/// Invisible grips along the edges that start an OS-driven resize.
/// Returns `true` when a resize was started this frame.
pub fn resize_grips(ui: &Ui, window: Rect) -> bool {
    let mut started = false;
    let g = size::RESIZE_GRIP;
    let c = g * 2.5; // corners are larger to be easy to hit
    let (l, r, t, b) = (window.left(), window.right(), window.top(), window.bottom());
    let grips: [(Rect, ResizeDirection, CursorIcon); 8] = [
        (Rect::from_min_max(pos2(l, t), pos2(l + c, t + c)), ResizeDirection::NorthWest, CursorIcon::ResizeNorthWest),
        (Rect::from_min_max(pos2(r - c, t), pos2(r, t + c)), ResizeDirection::NorthEast, CursorIcon::ResizeNorthEast),
        (Rect::from_min_max(pos2(l, b - c), pos2(l + c, b)), ResizeDirection::SouthWest, CursorIcon::ResizeSouthWest),
        (Rect::from_min_max(pos2(r - c, b - c), pos2(r, b)), ResizeDirection::SouthEast, CursorIcon::ResizeSouthEast),
        (Rect::from_min_max(pos2(l + c, t), pos2(r - c, t + g)), ResizeDirection::North, CursorIcon::ResizeNorth),
        (Rect::from_min_max(pos2(l + c, b - g), pos2(r - c, b)), ResizeDirection::South, CursorIcon::ResizeSouth),
        (Rect::from_min_max(pos2(l, t + c), pos2(l + g, b - c)), ResizeDirection::West, CursorIcon::ResizeWest),
        (Rect::from_min_max(pos2(r - g, t + c), pos2(r, b - c)), ResizeDirection::East, CursorIcon::ResizeEast),
    ];
    for (i, (rect, dir, cursor)) in grips.into_iter().enumerate() {
        let resp = ui.interact(rect, Id::new(("resize_grip", i)), Sense::drag());
        if resp.hovered() || resp.dragged() {
            ui.ctx().set_cursor_icon(cursor);
        }
        if resp.drag_started() {
            ui.ctx().send_viewport_cmd(ViewportCommand::BeginResize(dir));
            started = true;
        }
    }
    started
}

/// Fits a frame with the given display aspect into `area`, centered.
pub fn fit_rect(area: Rect, aspect: f32) -> Rect {
    if !(aspect.is_finite() && aspect > 0.0) {
        return area;
    }
    let mut size = vec2(area.width(), area.width() / aspect);
    if size.y > area.height() {
        size = vec2(area.height() * aspect, area.height());
    }
    Rect::from_center_size(area.center(), size)
}
