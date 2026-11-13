//! "rvp について" dialog: rvp's licence (GPL-3.0-or-later) and a table of the
//! third-party software it uses, with each component's licence text.

use egui::text::{LayoutJob, TextWrapping};
use egui::{
    Align, Color32, CornerRadius, FontId, Frame, Id, Layout, Margin, Rect, RichText, ScrollArea, Sense,
    Stroke, TextFormat, Ui, vec2,
};
use egui_phosphor::regular as icon;

use super::widgets::{icon_button, icon_button_ex, icon_font};
use crate::about::{self, Component};
use crate::theme::{color, panel_shadow, size};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    License,
    Software,
}

#[derive(Default)]
pub struct AboutState {
    pub tab: Tab,
    filter: String,
    /// Index into `about::components()` whose details are shown.
    selected: Option<usize>,
}

impl AboutState {
    pub fn new(tab: Tab) -> Self {
        Self { tab, ..Self::default() }
    }

    /// Software tab with the named component's details open (debug screenshots).
    pub fn with_component(name: &str) -> Self {
        let selected = about::components().iter().position(|c| c.name == name);
        Self { tab: Tab::Software, selected, ..Self::default() }
    }
}

const ROW_HEIGHT: f32 = 30.0;
const BODY_SIZE: f32 = 13.5;

/// Shows the dialog over `window`. Returns `false` once it should close.
pub fn about_dialog(ctx: &egui::Context, window: Rect, radius: f32, state: &mut AboutState) -> bool {
    // Backdrop on its own layer so it follows the rounded window corners
    // (the Modal's own backdrop is a plain rectangle).
    ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, Id::new("about_backdrop"))).rect_filled(
        window,
        radius,
        Color32::from_black_alpha(120),
    );

    let size = vec2((window.width() - 48.0).min(760.0), (window.height() - 48.0).min(600.0));
    let mut open = true;
    let frame = Frame::new()
        .fill(color::SURFACE_OPAQUE)
        .stroke(Stroke::new(1.0, color::BORDER))
        .corner_radius(CornerRadius::same(size::PANEL_RADIUS))
        .shadow(panel_shadow())
        .inner_margin(Margin::same(18));
    let resp = egui::Modal::new(Id::new("about_dialog"))
        .backdrop_color(Color32::TRANSPARENT)
        .frame(frame)
        .show(ctx, |ui| {
            let inner = size - vec2(36.0, 36.0);
            ui.set_min_size(inner);
            ui.set_max_size(inner);
            if header(ui, state) {
                open = false;
            }
            ui.add_space(10.0);
            match state.tab {
                Tab::License => license_tab(ui),
                Tab::Software => software_tab(ui, state),
            }
        });
    if resp.should_close() {
        open = false;
    }
    open
}

/// Title, tabs and close button. Returns `true` when close was clicked.
fn header(ui: &mut Ui, state: &mut AboutState) -> bool {
    let mut close = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new(icon::FILM_SLATE).font(icon_font(22.0)).color(color::ACCENT));
        ui.label(RichText::new(about::APP_NAME).size(20.0).strong());
        ui.label(RichText::new(t!("about-version", version = about::VERSION)).color(color::TEXT_DIM));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if icon_button_ex(ui, icon::X, &t!("about-close"), size::ICON_SIZE - 2.0, None).clicked() {
                close = true;
            }
        });
    });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (tab, label) in [(Tab::License, t!("about-tab-license")), (Tab::Software, t!("about-tab-software"))] {
            if tab_button(ui, &label, state.tab == tab) {
                state.tab = tab;
            }
        }
    });
    let r = ui.available_rect_before_wrap();
    ui.painter().hline(r.x_range(), r.top() + 2.0, Stroke::new(1.0, color::BORDER));
    ui.add_space(4.0);
    close
}

fn tab_button(ui: &mut Ui, label: &str, selected: bool) -> bool {
    let font = FontId::proportional(14.0);
    let galley = ui.painter().layout_no_wrap(label.to_owned(), font, color::TEXT);
    let (rect, resp) = ui.allocate_exact_size(galley.size() + vec2(24.0, 14.0), Sense::click());
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, CornerRadius::same(size::BUTTON_RADIUS), color::SURFACE_ACTIVE);
    } else if resp.hovered() {
        p.rect_filled(rect, CornerRadius::same(size::BUTTON_RADIUS), color::SURFACE_HOVER);
    }
    let fg = if selected { color::ACCENT } else { color::TEXT_DIM };
    p.galley_with_override_text_color(rect.center() - galley.size() / 2.0, galley, fg);
    resp.clicked()
}

fn license_tab(ui: &mut Ui) {
    ui.label(RichText::new(t!("about-license-line", copyright = about::COPYRIGHT, spdx = about::LICENSE_SPDX)).strong());
    ui.add_space(4.0);
    ui.label(RichText::new(t!("about-notice")).size(BODY_SIZE).color(color::TEXT_DIM));
    ui.add_space(8.0);
    text_box(ui, "gpl", about::LICENSE_TEXT);
}

/// Scrollable monospace licence text filling the remaining space.
fn text_box(ui: &mut Ui, id: &str, text: &str) {
    Frame::new()
        .fill(color::TEXT_BOX)
        .corner_radius(CornerRadius::same(size::BUTTON_RADIUS))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ScrollArea::vertical().id_salt(id).auto_shrink(false).show(ui, |ui| {
                ui.add(
                    egui::Label::new(RichText::new(text).monospace().size(12.0).color(color::TEXT))
                        .wrap()
                        .selectable(true),
                );
            });
        });
}

fn software_tab(ui: &mut Ui, state: &mut AboutState) {
    let all = about::components();
    if let Some(i) = state.selected
        && let Some(c) = all.get(i)
    {
        if component_details(ui, c) {
            state.selected = None;
        }
        return;
    }

    let rows: Vec<usize> = (0..all.len()).filter(|&i| all[i].matches(&state.filter)).collect();
    ui.horizontal(|ui| {
        ui.label(RichText::new(icon::MAGNIFYING_GLASS).font(icon_font(16.0)).color(color::TEXT_DIM));
        ui.add(
            egui::TextEdit::singleline(&mut state.filter)
                .hint_text(t!("about-filter-hint"))
                .background_color(color::TEXT_BOX)
                .margin(Margin::symmetric(8, 5))
                .desired_width(260.0),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(t!("about-count", shown = rows.len(), total = all.len())).size(12.5).color(color::TEXT_DIM));
        });
    });
    ui.label(
        RichText::new(t!("about-click-hint"))
            .size(12.0)
            .color(color::TEXT_DIM),
    );
    ui.add_space(6.0);

    let cols = columns(ui.available_width());
    // Header.
    let (hr, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
    let titles = ["about-col-name", "about-col-version", "about-col-license", "about-col-kind"].map(|id| t!(id));
    for ((x, w), title) in cols.iter().zip(&titles) {
        cell(ui, Rect::from_min_size(hr.min + vec2(*x, 0.0), vec2(*w, hr.height())), title, color::TEXT_DIM, true);
    }
    ui.painter().hline(hr.x_range(), hr.bottom(), Stroke::new(1.0, color::BORDER));

    ScrollArea::vertical().id_salt("software").auto_shrink(false).show_rows(ui, ROW_HEIGHT, rows.len(), |ui, range| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for &i in &rows[range] {
            let c = &all[i];
            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), Sense::click());
            if resp.hovered() {
                ui.painter().rect_filled(r, CornerRadius::same(6), color::SURFACE_HOVER);
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            let kind = c.kind.label();
            let values = [c.name.as_str(), c.version.as_str(), c.license.as_str(), kind.as_str()];
            for (k, ((x, w), v)) in cols.iter().zip(values).enumerate() {
                let fg = if k == 0 { color::TEXT } else { color::TEXT_DIM };
                cell(ui, Rect::from_min_size(r.min + vec2(*x, 0.0), vec2(*w, r.height())), v, fg, false);
            }
            if resp.clicked() {
                state.selected = Some(i);
            }
        }
    });
}

/// (x offset, width) of each table column.
fn columns(width: f32) -> [(f32, f32); 4] {
    let kind = 104.0;
    let version = 120.0;
    let rest = (width - kind - version).max(100.0);
    let name = rest * 0.45;
    let license = rest - name;
    [(0.0, name), (name, version), (name + version, license), (name + version + license, kind)]
}

/// One table cell: single line, elided with "…".
fn cell(ui: &Ui, rect: Rect, text: &str, fg: Color32, header: bool) {
    let pad = 10.0;
    let mut job = LayoutJob::single_section(
        text.to_owned(),
        TextFormat::simple(FontId::proportional(if header { 12.5 } else { BODY_SIZE }), fg),
    );
    job.wrap = TextWrapping::truncate_at_width(rect.width() - pad * 2.0);
    let galley = ui.painter().layout_job(job);
    ui.painter().galley(
        rect.left_center() + vec2(pad, -galley.size().y / 2.0),
        galley,
        fg,
    );
}

/// Detail view of one component. Returns `true` when "back" was clicked.
fn component_details(ui: &mut Ui, c: &Component) -> bool {
    let mut back = false;
    ui.horizontal(|ui| {
        if icon_button(ui, icon::ARROW_LEFT, &t!("about-back")).clicked() {
            back = true;
        }
        ui.label(RichText::new(&c.name).size(17.0).strong());
        ui.label(RichText::new(&c.version).color(color::TEXT_DIM));
    });
    ui.add_space(4.0);
    egui::Grid::new("component_meta").num_columns(2).spacing(vec2(14.0, 4.0)).show(ui, |ui| {
        ui.label(RichText::new(t!("about-col-license")).color(color::TEXT_DIM));
        ui.label(&c.license);
        ui.end_row();
        ui.label(RichText::new(t!("about-col-kind")).color(color::TEXT_DIM));
        ui.label(c.kind.label());
        ui.end_row();
        if !c.url.is_empty() {
            ui.label(RichText::new("URL").color(color::TEXT_DIM));
            ui.hyperlink(&c.url);
            ui.end_row();
        }
    });
    if let Some(note) = &c.note {
        ui.add_space(4.0);
        ui.add(egui::Label::new(RichText::new(note).size(12.5).color(color::TEXT_DIM)).wrap());
    }
    if let Some(details) = &c.details {
        egui::CollapsingHeader::new(RichText::new(t!("about-build-config")).size(12.5).color(color::TEXT_DIM))
            .id_salt("component_details")
            .show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(details).monospace().size(11.5).color(color::TEXT_DIM)).wrap());
            });
    }
    ui.add_space(8.0);
    if c.texts.is_empty() {
        ui.label(RichText::new(t!("about-no-text")).color(color::TEXT_DIM));
        return back;
    }
    let body: String = if c.texts.len() == 1 {
        c.texts[0].1.to_string()
    } else {
        c.texts.iter().map(|(t, b)| format!("──── {t} ────\n\n{b}\n")).collect()
    };
    text_box(ui, &format!("text_{}_{}", c.name, c.version), &body);
    if ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Extra1)) {
        back = true;
    }
    back
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_fill_width() {
        let cols = columns(700.0);
        let (x, w) = cols[3];
        assert!((x + w - 700.0).abs() < 0.01);
        assert!(cols.windows(2).all(|p| (p[0].0 + p[0].1 - p[1].0).abs() < 0.01));
    }

    #[test]
    fn components_include_ffmpeg_and_crates() {
        let all = about::components();
        assert!(all.iter().any(|c| c.name == "FFmpeg" && c.kind == about::Kind::Library));
        assert!(all.iter().any(|c| c.name == "egui" && c.kind == about::Kind::Crate));
    }
}
