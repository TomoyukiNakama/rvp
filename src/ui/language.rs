//! Title-bar language button with a menu of the available UI languages.

use egui::{Align2, CornerRadius, FontId, Popup, RectAlign, Sense, Ui, vec2};
use egui_phosphor::regular as icon;

use super::widgets::{icon_button, icon_font};
use crate::i18n;
use crate::theme::color;

/// Returns the tag of a newly chosen language.
pub fn language_button(ui: &mut Ui) -> Option<String> {
    let resp = icon_button(ui, icon::TRANSLATE, &t!("window-language"));
    crate::debug_shot::maybe_open_popup(ui.ctx(), "language", &resp);

    let mut chosen = None;
    // Fixed width: rows must not stretch to the parent (title bar) width.
    const MENU_WIDTH: f32 = 200.0;
    Popup::menu(&resp).align(RectAlign::BOTTOM_END).gap(6.0).width(MENU_WIDTH).show(|ui| {
        ui.set_width(MENU_WIDTH);
        ui.spacing_mut().item_spacing.y = 1.0;
        let current = i18n::current();
        for lang in i18n::available() {
            let selected = lang.tag == current;
            let (r, item) = ui.allocate_exact_size(vec2(MENU_WIDTH, 28.0), Sense::click());
            #[cfg(test)]
            tests::ROWS.with(|rows| rows.borrow_mut().push((lang.tag.clone(), r)));
            if selected {
                ui.painter().rect_filled(r, CornerRadius::same(7), color::ACCENT.gamma_multiply(0.18));
            } else if item.hovered() {
                ui.painter().rect_filled(r, CornerRadius::same(7), color::SURFACE_HOVER);
            }
            let fg = if selected { color::ACCENT } else { color::TEXT };
            if selected {
                ui.painter().text(r.left_center() + vec2(14.0, 0.0), Align2::CENTER_CENTER, icon::CHECK, icon_font(14.0), fg);
            }
            ui.painter().text(r.left_center() + vec2(30.0, 0.0), Align2::LEFT_CENTER, &lang.name, FontId::proportional(14.0), fg);
            ui.painter().text(
                r.right_center() - vec2(10.0, 0.0),
                Align2::RIGHT_CENTER,
                &lang.tag,
                FontId::proportional(11.5),
                color::TEXT_DIM,
            );
            if item.clicked() {
                chosen = Some(lang.tag);
                ui.close();
            }
        }
    });
    chosen
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        pub static ROWS: RefCell<Vec<(String, egui::Rect)>> = const { RefCell::new(Vec::new()) };
    }

    /// Clicking the button opens the menu; clicking a row returns its tag.
    #[test]
    fn choosing_a_language() {
        let ctx = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        ctx.set_fonts(fonts);
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1280.0, 720.0));
        let button = std::cell::Cell::new(egui::Rect::NOTHING);
        let mut frame = |events: Vec<egui::Event>| -> Option<String> {
            ROWS.with(|r| r.borrow_mut().clear());
            let input = egui::RawInput { screen_rect: Some(screen), events, ..Default::default() };
            let mut chosen = None;
            let mut out = ctx.run_ui(input, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(600.0);
                    let before = ui.cursor().min;
                    chosen = language_button(ui);
                    button.set(egui::Rect::from_min_size(before, vec2(32.0, 32.0)));
                });
            });
            out.textures_delta.clear(); // no renderer in tests
            chosen
        };
        let click = |pos: egui::Pos2| {
            let press = |pressed| egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            };
            vec![vec![egui::Event::PointerMoved(pos)], vec![press(true)], vec![press(false)], vec![]]
        };

        frame(vec![]);
        for events in click(button.get().center()) {
            assert!(frame(events).is_none());
        }
        let de = ROWS.with(|r| r.borrow().iter().find(|(t, _)| t == "de").map(|(_, r)| *r)).expect("menu is open");
        let chosen: Vec<String> = click(de.center()).into_iter().filter_map(&mut frame).collect();
        assert_eq!(chosen, ["de"]);
    }
}
