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
    let mut v = vec![(SubSel::Off, "オフ".to_owned())];
    v.extend(m.subs.iter().filter(|t| t.supported).map(|t| (SubSel::Embedded(t.stream), t.label.clone())));
    v.extend(m.externals.iter().enumerate().map(|(i, n)| (SubSel::External(i), format!("{} {n}", icon::FILE_TEXT))));
    v
}

pub fn format_delay(d: f64) -> String {
    format!("{}{:.1} 秒", if d >= 0.0 { "+" } else { "−" }, d.abs())
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
    let tip = "字幕・音声 (V / S / A)";
    let resp = icon_button(ui, icon::SUBTITLES, tip);
    if m.sub_sel != SubSel::Off && m.visible {
        // Small accent dot: subtitles are on.
        ui.painter().circle_filled(resp.rect.right_top() + vec2(-9.0, 9.0), 3.0, color::ACCENT);
    }

    let mut action = None;
    Popup::menu(&resp)
        .align(RectAlign::TOP)
        .gap(10.0)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_min_width(260.0);
            ui.set_max_width(360.0);
            ui.spacing_mut().item_spacing.y = 2.0;

            heading(ui, "字幕");
            for (sel, label) in subtitle_choices(m) {
                if row(ui, &label, sel == m.sub_sel, true) {
                    action = Some(TracksAction::Subtitle(sel));
                    ui.close();
                }
            }
            for t in m.subs.iter().filter(|t| !t.supported) {
                row_ex(ui, &format!("{} (未対応)", t.label), false, false, Some("画像字幕には未対応です"));
            }
            if row(ui, &format!("{}  字幕ファイルを読み込む…", icon::FOLDER_OPEN), false, true) {
                action = Some(TracksAction::LoadSubtitle);
                ui.close();
            }

            // Timing + visibility (popup stays open while adjusting).
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(RichText::new("タイミング").size(13.0).color(color::TEXT_DIM));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    if icon_button(ui, icon::PLUS, "遅らせる (X)").clicked() {
                        action = Some(TracksAction::Delay(0.1));
                    }
                    let d = ui.add(
                        egui::Label::new(RichText::new(format_delay(m.delay)).monospace().size(13.0))
                            .sense(Sense::click()),
                    );
                    if d.on_hover_text("クリックでリセット").clicked() {
                        action = Some(TracksAction::Delay(0.0));
                    }
                    if icon_button(ui, icon::MINUS, "早める (Z)").clicked() {
                        action = Some(TracksAction::Delay(-0.1));
                    }
                });
            });
            if m.sub_sel != SubSel::Off {
                let label = if m.visible { "字幕を隠す (V)" } else { "字幕を表示 (V)" };
                if row(ui, label, false, true) {
                    action = Some(TracksAction::ToggleVisible);
                }
            }

            if !m.audio.is_empty() {
                ui.add_space(4.0);
                let sep = ui.available_rect_before_wrap();
                ui.painter().hline(sep.x_range(), sep.top(), egui::Stroke::new(1.0, color::BORDER));
                ui.add_space(4.0);
                heading(ui, "音声");
                for t in m.audio {
                    let enabled = m.audio_sel.is_some();
                    if row_ex(ui, &t.label, m.audio_sel == Some(t.stream), enabled, Some("音声出力がありません")) {
                        action = Some(TracksAction::Audio(t.stream));
                        ui.close();
                    }
                }
            }
        });
    action
}
