//! Track menu: subtitle / audio selection, subtitle timing, loading files.

use egui::{Align2, CornerRadius, FontId, Popup, PopupCloseBehavior, RectAlign, RichText, Sense, Ui, vec2};
use egui_phosphor::regular as icon;

use super::widgets::{icon_button, icon_font};
use crate::player::{SubSel, TrackInfo};
use crate::theme::color;

pub struct TracksModel<'a> {
    pub audio: &'a [TrackInfo],
    pub audio_sel: Option<usize>,
    pub subs: &'a [TrackInfo],
    /// File names of loaded external subtitle files.
    pub externals: Vec<String>,
    pub sub_sel: SubSel,
    pub visible: bool,
    pub delay: f64,
}

pub enum TracksAction {
    Audio(usize),
    Subtitle(SubSel),
    LoadSubtitle,
    ToggleVisible,
    /// Adjust subtitle delay by this many seconds (0.0 = reset).
    Delay(f64),
}

/// All selectable subtitle choices in menu order, with labels.
pub fn subtitle_choices(m: &TracksModel<'_>) -> Vec<(SubSel, String)> {
    let mut v = vec![(SubSel::Off, t!("tracks-off"))];
    v.extend(m.subs.iter().filter(|t| t.supported).map(|t| (SubSel::Embedded(t.stream), t.label())));
    v.extend(m.externals.iter().enumerate().map(|(i, n)| (SubSel::External(i), format!("{} {n}", icon::FILE_TEXT))));
    v
}

pub fn format_delay(d: f64) -> String {
    t!("delay-seconds", sign = if d >= 0.0 { "+" } else { "−" }, value = format!("{:.1}", d.abs()))
}

fn row(ui: &mut Ui, label: &str, selected: bool, enabled: bool) -> bool {
    row_ex(ui, label, selected, enabled, None)
}

fn row_ex(ui: &mut Ui, label: &str, selected: bool, enabled: bool, why_disabled: Option<&str>) -> bool {
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::click());
    let resp = match why_disabled {
        Some(why) if !enabled => resp.on_hover_text(why),
        _ => resp,
    };
    if enabled && resp.hovered() {
        ui.painter().rect_filled(r, CornerRadius::same(7), color::SURFACE_HOVER);
    }
    let fg = match (enabled, selected) {
        (false, _) => color::TEXT_DIM.gamma_multiply(0.6),
        (true, true) => color::ACCENT,
        (true, false) => color::TEXT,
    };
    if selected {
        ui.painter().text(r.left_center() + vec2(14.0, 0.0), Align2::CENTER_CENTER, icon::CHECK, icon_font(14.0), fg);
    }
    let galley = ui.painter().layout(label.to_owned(), FontId::proportional(14.0), fg, r.width() - 36.0);
    ui.painter().galley(r.left_center() + vec2(30.0, -galley.size().y / 2.0), galley, fg);
    enabled && resp.clicked()
}

fn heading(ui: &mut Ui, text: &str) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.label(RichText::new(text).size(12.0).color(color::TEXT_DIM).strong());
    });
}

pub fn tracks_button(ui: &mut Ui, m: &TracksModel<'_>) -> Option<TracksAction> {
    let resp = icon_button(ui, icon::SUBTITLES, &t!("tracks-tooltip"));
    if m.sub_sel != SubSel::Off && m.visible {
        // Small accent dot: subtitles are on.
        ui.painter().circle_filled(resp.rect.right_top() + vec2(-9.0, 9.0), 3.0, color::ACCENT);
    }

    crate::debug_shot::maybe_open_popup(ui.ctx(), "tracks", &resp);
    let mut action = None;
    const MENU_WIDTH: f32 = 300.0;
    Popup::menu(&resp)
        .align(RectAlign::TOP)
        .gap(10.0)
        .width(MENU_WIDTH)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_width(MENU_WIDTH);
            ui.spacing_mut().item_spacing.y = 2.0;

            heading(ui, &t!("tracks-subtitles"));
            for (sel, label) in subtitle_choices(m) {
                if row(ui, &label, sel == m.sub_sel, true) {
                    action = Some(TracksAction::Subtitle(sel));
                    ui.close();
                }
            }
            for t in m.subs.iter().filter(|t| !t.supported) {
                row_ex(ui, &t!("tracks-unsupported", label = t.label()), false, false, Some(&t!("tracks-unsupported-why")));
            }
            if row(ui, &format!("{}  {}", icon::FOLDER_OPEN, t!("tracks-load-subtitle")), false, true) {
                action = Some(TracksAction::LoadSubtitle);
                ui.close();
            }

            // Timing + visibility (popup stays open while adjusting).
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(22.0);
                ui.label(RichText::new(t!("tracks-timing")).size(13.0).color(color::TEXT_DIM));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    if icon_button(ui, icon::PLUS, &t!("tracks-delay-later")).clicked() {
                        action = Some(TracksAction::Delay(0.1));
                    }
                    let d = ui.add(
                        egui::Label::new(RichText::new(format_delay(m.delay)).monospace().size(13.0))
                            .sense(Sense::click()),
                    );
                    if d.on_hover_text(t!("tracks-delay-reset")).clicked() {
                        action = Some(TracksAction::Delay(0.0));
                    }
                    if icon_button(ui, icon::MINUS, &t!("tracks-delay-earlier")).clicked() {
                        action = Some(TracksAction::Delay(-0.1));
                    }
                });
            });
            if m.sub_sel != SubSel::Off {
                let label = t!(if m.visible { "tracks-hide-subtitles" } else { "tracks-show-subtitles" });
                if row(ui, &label, false, true) {
                    action = Some(TracksAction::ToggleVisible);
                }
            }

            if !m.audio.is_empty() {
                ui.add_space(4.0);
                let sep = ui.available_rect_before_wrap();
                ui.painter().hline(sep.x_range(), sep.top(), egui::Stroke::new(1.0, color::BORDER));
                ui.add_space(4.0);
                heading(ui, &t!("tracks-audio"));
                for t in m.audio {
                    let enabled = m.audio_sel.is_some();
                    if row_ex(ui, &t.label(), m.audio_sel == Some(t.stream), enabled, Some(&t!("tracks-no-audio-output"))) {
                        action = Some(TracksAction::Audio(t.stream));
                        ui.close();
                    }
                }
            }
        });
    action
}
