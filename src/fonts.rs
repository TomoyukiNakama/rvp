//! Font setup: a CJK system font (via fontconfig) plus Phosphor icons.

use std::process::Command;
use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

const CJK_KEY: &str = "system-cjk";

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    match find_cjk_font() {
        Some((path, index)) => match std::fs::read(&path) {
            Ok(bytes) => {
                let mut data = FontData::from_owned(bytes);
                data.index = index;
                fonts.font_data.insert(CJK_KEY.into(), Arc::new(data));
                // Primary UI font: covers Latin and Japanese consistently.
                for family in [FontFamily::Proportional, FontFamily::Monospace] {
                    let list = fonts.families.entry(family.clone()).or_default();
                    let at = if family == FontFamily::Proportional { 0 } else { list.len() };
                    list.insert(at, CJK_KEY.into());
                }
                log::info!("CJK font: {path} (index {index})");
            }
            Err(e) => log::warn!("failed to read font {path}: {e}"),
        },
        None => log::warn!("no CJK font found via fontconfig; Japanese text may not render"),
    }

    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}

/// Asks fontconfig for the best Japanese sans-serif font.
fn find_cjk_font() -> Option<(String, u32)> {
    let out = Command::new("fc-match")
        .args(["-f", "%{file}\n%{index}", "sans-serif:lang=ja"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let mut lines = text.lines();
    let file = lines.next()?.trim().to_owned();
    let index = lines.next().and_then(|i| i.trim().parse().ok()).unwrap_or(0);
    (!file.is_empty()).then_some((file, index))
}
