//! Font setup: system fonts chosen per UI language (via fontconfig) plus Phosphor icons.
//!
//! The primary UI font is fontconfig's best sans-serif for the UI language, so e.g.
//! Simplified Chinese gets SC glyph shapes and Korean a Hangul font. For non-CJK
//! languages a Japanese CJK font is added as fallback (CJK file names, subtitles).

use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

use egui::{FontData, FontDefinitions, FontFamily};

/// Files of the system fonts in use (shown in the licence dialog).
static IN_USE: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn font_paths() -> Vec<String> {
    IN_USE.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// (Re)installs the fonts for the UI language `lang` (a BCP 47 tag).
pub fn install(ctx: &egui::Context, lang: &str) {
    let mut fonts = FontDefinitions::default();
    let fc = fc_lang(lang);
    let mut queries = vec![(format!("sans-serif:lang={fc}"), fc.clone())];
    if !is_cjk(&fc) {
        queries.push(("sans-serif:lang=ja".to_owned(), "ja".to_owned()));
    }

    let mut used = Vec::new();
    for (i, (query, query_lang)) in queries.iter().enumerate() {
        let Some((path, index)) = fc_match_lang(query, query_lang) else {
            log::warn!("fontconfig: nothing for {query}");
            continue;
        };
        if used.contains(&path) {
            continue;
        }
        let Some(data) = load(&path, index) else { continue };
        let key = format!("system-{i}");
        fonts.font_data.insert(key.clone(), data);
        // Proportional: ours first (consistent Latin + script); monospace: fallback only.
        let prop = fonts.families.entry(FontFamily::Proportional).or_default();
        prop.insert(used.len().min(prop.len()), key.clone());
        fonts.families.entry(FontFamily::Monospace).or_default().push(key);
        log::info!("UI font for {lang} ({query}): {path} (index {index})");
        used.push(path);
    }
    *IN_USE.lock().unwrap_or_else(|e| e.into_inner()) = used;

    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}

/// fontconfig language for a UI language tag.
fn fc_lang(tag: &str) -> String {
    let t = tag.to_ascii_lowercase();
    match t.as_str() {
        "zh-cn" | "zh-tw" | "zh-hk" => t,
        _ if t.starts_with("zh-hant") => "zh-tw".into(),
        _ if t.starts_with("zh") => "zh-cn".into(),
        _ => t.split('-').next().unwrap_or("en").to_owned(),
    }
}

fn is_cjk(fc: &str) -> bool {
    fc == "ja" || fc == "ko" || fc.starts_with("zh")
}

/// Font files are large (CJK collections are ~20 MB): read each once.
fn load(path: &str, index: u32) -> Option<Arc<FontData>> {
    static CACHE: Mutex<Option<HashMap<(String, u32), Arc<FontData>>>> = Mutex::new(None);
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let cache = cache.get_or_insert_with(HashMap::new);
    if let Some(d) = cache.get(&(path.to_owned(), index)) {
        return Some(d.clone());
    }
    match std::fs::read(path) {
        Ok(bytes) => {
            let mut data = FontData::from_owned(bytes);
            data.index = index;
            let data = Arc::new(data);
            cache.insert((path.to_owned(), index), data.clone());
            Some(data)
        }
        Err(e) => {
            log::warn!("failed to read font {path}: {e}");
            None
        }
    }
}

/// Region suffix of pan-CJK font families (Noto Sans CJK, Source Han Sans…) for a fontconfig language.
fn cjk_region(fc: &str) -> Option<&'static str> {
    Some(match fc {
        "ja" => "JP",
        "ko" => "KR",
        "zh-cn" => "SC",
        "zh-tw" => "TC",
        "zh-hk" => "HK",
        _ => return None,
    })
}

/// The same pan-CJK family for another region: "Noto Sans CJK KR" → "Noto Sans CJK JP".
fn regional_family(family: &str, region: &str) -> Option<String> {
    let (base, suffix) = family.rsplit_once(' ')?;
    (["JP", "KR", "SC", "TC", "HK"].contains(&suffix) && suffix != region).then(|| format!("{base} {region}"))
}

/// Best font for `pattern`, making sure a pan-CJK family is the face for `lang`.
/// fontconfig's own configuration may rank e.g. the KR face first for `lang=ja`
/// (Flatpak runtimes do): its kanji have Korean glyph shapes.
fn fc_match_lang(pattern: &str, lang: &str) -> Option<(String, u32)> {
    let (file, index, family) = fc_match(pattern)?;
    if let Some(wanted) = cjk_region(lang).and_then(|r| regional_family(&family, r)) {
        if let Some((f, i, fam)) = fc_match(&format!("{wanted}:lang={lang}")).filter(|m| m.2 == wanted) {
            log::info!("fontconfig: {family} → {fam} for {lang}");
            return Some((f, i));
        }
    }
    Some((file, index))
}

/// Asks fontconfig for the best match of `pattern`: (file, face index, first family name).
fn fc_match(pattern: &str) -> Option<(String, u32, String)> {
    let out = Command::new("fc-match").args(["-f", "%{file}\n%{index}\n%{family[0]}", pattern]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let mut lines = text.lines();
    let file = lines.next()?.trim().to_owned();
    let index = lines.next().and_then(|i| i.trim().parse().ok()).unwrap_or(0);
    let family = lines.next().unwrap_or("").trim().to_owned();
    (!file.is_empty()).then_some((file, index, family))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fontconfig_languages() {
        assert_eq!(fc_lang("ja"), "ja");
        assert_eq!(fc_lang("zh-CN"), "zh-cn");
        assert_eq!(fc_lang("zh-TW"), "zh-tw");
        assert_eq!(fc_lang("zh-Hant"), "zh-tw");
        assert_eq!(fc_lang("pt-BR"), "pt");
        assert!(is_cjk("ko") && !is_cjk("vi"));
        assert_eq!(regional_family("Noto Sans CJK KR", "JP").as_deref(), Some("Noto Sans CJK JP"));
        assert_eq!(regional_family("Source Han Sans SC", "TC").as_deref(), Some("Source Han Sans TC"));
        assert_eq!(regional_family("Noto Sans CJK JP", "JP"), None);
        assert_eq!(regional_family("DejaVu Sans", "JP"), None);
    }
}
