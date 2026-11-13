//! Floating playlist panel on the right side of the window.

use std::path::PathBuf;

use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Frame, Id, Layout, Margin, Rect, RichText, ScrollArea, Sense,
    Stroke, Ui, UiBuilder, pos2, vec2,
};
use egui_phosphor::regular as icon;

use super::widgets::{icon_button, icon_button_ex, icon_font};
use crate::playlist::Repeat;
use crate::theme::{color, panel_shadow, size};

pub enum PlaylistAction {
    Play(usize),
    Remove(usize),
    Move { from: usize, to: usize },
    CycleRepeat,
    ToggleShuffle,
    Add,
    Save,
    Close,
}

pub struct PlaylistModel<'a> {
    pub items: &'a [PathBuf],
    pub current: Option<usize>,
    pub repeat: Repeat,
    pub shuffle: bool,
}

const ROW_HEIGHT: f32 = 38.0;

pub fn panel_rect(window: Rect) -> Rect {
    let inset = size::CONTROLBAR_INSET;
    let width = (window.width() * 0.4).clamp(260.0, 360.0);
    let top = window.top() + size::TITLEBAR_HEIGHT + 4.0;
    let bottom = window.bottom() - inset - size::CONTROLBAR_HEIGHT - 10.0;
    Rect::from_min_max(pos2(window.right() - inset - width, top), pos2(window.right() - inset, bottom.max(top + 120.0)))
}

/// Toggle button whose icon is tinted with the accent colour when `on`.
pub fn toggle_icon(ui: &mut Ui, glyph: &str, tip: &str, on: bool) -> egui::Response {
    let resp = icon_button_ex(ui, "", tip, size::ICON_SIZE, None);
    let c = if on { color::ACCENT } else { color::TEXT_DIM };
    ui.painter().text(resp.rect.center(), Align2::CENTER_CENTER, glyph, icon_font(size::ICON_SIZE - 2.0), c);
    resp
}

pub fn playlist_panel(ui: &mut Ui, window: Rect, m: &PlaylistModel<'_>, alpha: f32) -> Vec<PlaylistAction> {
    let mut actions = Vec::new();
    if alpha <= 0.0 {
        return actions;
    }
    let rect = panel_rect(window);
    let mut child = ui.new_child(UiBuilder::new().max_rect(rect).layout(Layout::top_down(Align::Min)));
    child.multiply_opacity(alpha);
    let ui = &mut child;
    // Swallow clicks/drags on the panel background so they don't move the window.
    ui.interact(rect, Id::new("playlist_blocker"), Sense::click_and_drag());

    Frame::new()
        .fill(color::SURFACE)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(CornerRadius::same(size::PANEL_RADIUS))
        .shadow(panel_shadow())
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_min_size(rect.size() - vec2(16.0, 16.0));
            ui.set_max_size(rect.size() - vec2(16.0, 16.0));

            // Header.
            ui.horizontal(|ui| {
                ui.add_space(6.0);
                // Five buttons leave little room in a narrow panel: drop the
                // counter first, then show it in place of the title.
                let w = ui.available_width();
                if w >= 290.0 {
                    ui.label(RichText::new(t!("playlist-title")).strong().size(15.0));
                }
                if !(290.0..330.0).contains(&w) {
                    let pos = m.current.map_or(0, |c| c + 1);
                    ui.label(RichText::new(format!("{pos}/{}", m.items.len())).color(color::TEXT_DIM).size(12.5));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    if icon_button_ex(ui, icon::X, &t!("playlist-close"), size::ICON_SIZE - 2.0, None).clicked() {
                        actions.push(PlaylistAction::Close);
                    }
                    if icon_button(ui, icon::FLOPPY_DISK, &t!("playlist-save")).clicked() {
                        actions.push(PlaylistAction::Save);
                    }
                    if icon_button(ui, icon::PLUS, &t!("playlist-add")).clicked() {
                        actions.push(PlaylistAction::Add);
                    }
                    let rep_icon = if m.repeat == Repeat::One { icon::REPEAT_ONCE } else { icon::REPEAT };
                    let rep_tip = m.repeat.label();
                    if toggle_icon(ui, rep_icon, &rep_tip, m.repeat != Repeat::Off).clicked() {
                        actions.push(PlaylistAction::CycleRepeat);
                    }
                    let shuf_tip = t!(if m.shuffle { "shuffle-on" } else { "shuffle-off" });
                    if toggle_icon(ui, icon::SHUFFLE, &shuf_tip, m.shuffle).clicked() {
                        actions.push(PlaylistAction::ToggleShuffle);
                    }
                });
            });
            ui.add_space(4.0);
            let sep = ui.available_rect_before_wrap();
            ui.painter().hline(sep.x_range(), sep.top(), Stroke::new(1.0, color::BORDER));
            ui.add_space(6.0);

            if m.items.is_empty() {
                ui.add_space(24.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(t!("playlist-empty-hint")).color(color::TEXT_DIM));
                });
                return;
            }

            ScrollArea::vertical().auto_shrink([false, false]).show_rows(ui, ROW_HEIGHT, m.items.len(), |ui, range| {
                for i in range {
                    row(ui, i, m, &mut actions);
                }
            });
        });

    actions
}

fn row(ui: &mut Ui, i: usize, m: &PlaylistModel<'_>, actions: &mut Vec<PlaylistAction>) {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::click_and_drag());
    #[cfg(test)]
    tests::ROW_RECTS.with(|r| r.borrow_mut().push(rect));
    let is_current = m.current == Some(i);
    // Pointer position, not `resp.hovered()`: once the pointer is on the remove
    // button, that button (not the row) is the hovered widget, and the button
    // must still exist on the frame the click lands.
    let hovered = ui.rect_contains_pointer(rect) && !ui.ctx().dragged_id().is_some_and(|d| d != resp.id);
    let p = ui.painter();
    let radius = CornerRadius::same(size::BUTTON_RADIUS);

    if is_current {
        p.rect_filled(rect, radius, color::ACCENT.gamma_multiply(0.16));
    } else if hovered || resp.dragged() {
        p.rect_filled(rect, radius, color::SURFACE_HOVER);
    }

    // Leading: play glyph for the current entry, else the 1-based index.
    let lead = pos2(rect.left() + 18.0, rect.center().y);
    if is_current {
        p.text(lead, Align2::CENTER_CENTER, icon::PLAY, icon_font(14.0), color::ACCENT);
    } else {
        p.text(lead, Align2::CENTER_CENTER, (i + 1).to_string(), FontId::monospace(12.0), color::TEXT_DIM);
    }

    let name = m.items[i]
        .file_name()
        .map_or_else(|| m.items[i].to_string_lossy(), |n| n.to_string_lossy())
        .into_owned();
    let text_right = rect.right() - if hovered { 40.0 } else { 10.0 };
    let text_color = if is_current { color::TEXT } else { color::TEXT.gamma_multiply(0.85) };
    let galley = ui.painter().layout(name.clone(), FontId::proportional(14.0), text_color, f32::INFINITY);
    let text_rect = Rect::from_min_max(pos2(rect.left() + 36.0, rect.top()), pos2(text_right, rect.bottom()));
    let clipped = ui.painter().with_clip_rect(text_rect.intersect(ui.clip_rect()));
    clipped.galley(pos2(text_rect.left(), rect.center().y - galley.size().y / 2.0), galley.clone(), text_color);
    if galley.size().x > text_rect.width() {
        // Fade the overflow instead of hard-cutting it.
        let fade = Rect::from_min_max(pos2(text_rect.right() - 24.0, rect.top()), pos2(text_rect.right(), rect.bottom()));
        super::window::horizontal_gradient(&clipped, fade, Color32::TRANSPARENT, color::SURFACE);
    }

    // Remove button on hover.
    let mut removed = false;
    if hovered {
        let btn = Rect::from_center_size(pos2(rect.right() - 20.0, rect.center().y), vec2(28.0, 28.0));
        let b = ui.interact(btn, Id::new(("pl_remove", i)), Sense::click());
        if b.hovered() {
            ui.painter().rect_filled(btn, CornerRadius::same(7), color::SURFACE_ACTIVE);
        }
        ui.painter().text(btn.center(), Align2::CENTER_CENTER, icon::X, icon_font(14.0), color::TEXT_DIM);
        if b.clicked() {
            actions.push(PlaylistAction::Remove(i));
            removed = true;
        }
    }

    let resp = resp.on_hover_text(&name);
    if !removed && resp.double_clicked() {
        actions.push(PlaylistAction::Play(i));
    }

    // Drag to reorder.
    resp.dnd_set_drag_payload(i);
    if let Some(from) = resp.dnd_hover_payload::<usize>()
        && *from != i
    {
        let y = if *from < i { rect.bottom() } else { rect.top() };
        ui.painter().hline(rect.x_range(), y, Stroke::new(2.0, color::ACCENT));
    }
    if let Some(from) = resp.dnd_release_payload::<usize>()
        && *from != i
    {
        actions.push(PlaylistAction::Move { from: *from, to: i });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        pub static ROW_RECTS: RefCell<Vec<Rect>> = const { RefCell::new(Vec::new()) };
    }

    /// Drives the panel with synthetic pointer input: hovering a row and
    /// clicking its × must yield `Remove`.
    #[test]
    fn remove_button_click() {
        let ctx = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        ctx.set_fonts(fonts);
        let items: Vec<PathBuf> = ["a.mp4", "b.mp4", "c.mp4"].iter().map(PathBuf::from).collect();
        let model = PlaylistModel { items: &items, current: Some(2), repeat: Repeat::Off, shuffle: false };
        let screen = Rect::from_min_size(egui::Pos2::ZERO, vec2(1280.0, 720.0));

        let mut frame = |events: Vec<egui::Event>| -> Vec<PlaylistAction> {
            ROW_RECTS.with(|r| r.borrow_mut().clear());
            let input = egui::RawInput { screen_rect: Some(screen), events, ..Default::default() };
            let mut actions = Vec::new();
            let mut out = ctx.run_ui(input, |ui| {
                actions = playlist_panel(ui, screen, &model, 1.0);
            });
            out.textures_delta.clear(); // no renderer in tests
            actions
        };

        frame(vec![]);
        frame(vec![]);
        let row = ROW_RECTS.with(|r| r.borrow()[1]);
        let x = egui::pos2(row.right() - 20.0, row.center().y);
        let press = |pressed| egui::Event::PointerButton {
            pos: x,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        let mut all = Vec::new();
        all.extend(frame(vec![egui::Event::PointerMoved(x)]));
        all.extend(frame(vec![]));
        all.extend(frame(vec![press(true)]));
        all.extend(frame(vec![press(false)]));
        all.extend(frame(vec![]));
        assert!(
            all.iter().any(|a| matches!(a, PlaylistAction::Remove(1))),
            "clicking × on row 2 should remove it"
        );
        assert!(!all.iter().any(|a| matches!(a, PlaylistAction::Play(_))), "single click must not play");
    }
}
