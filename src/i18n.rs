//! UI localisation with Project Fluent (`.ftl` message files).
//!
//! - Built-in translations live in `locales/<tag>.ftl` and are embedded at build time.
//! - `<tag>.ftl` files in the user locale directory (`~/.config/rvp/locales/`) and in
//!   `$RVP_LOCALES_DIR` override single messages of a built-in language or add a new
//!   language. They are re-read while rvp runs (`poll_reload`).
//! - Lookups fall back to English, then to the message id itself.
//! - The language can be switched at any time (`set_language`); `generation()` changes
//!   whenever the active messages do, so caches of translated text can refresh.
//!
//! Use the `t!` macro: `t!("osd-speed")`, `t!("osd-added", count = n)`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use unic_langid::LanguageIdentifier;

/// Translates a message: `t!("id")` or `t!("id", name = value, …)`.
/// Values may be strings or numbers (numbers take part in plural selection).
macro_rules! t {
    ($id:expr) => {
        $crate::i18n::tr($id, None)
    };
    ($id:expr, $($key:ident = $val:expr),+ $(,)?) => {{
        let mut args = ::fluent_bundle::FluentArgs::new();
        $( args.set(stringify!($key), ::fluent_bundle::FluentValue::from($val)); )+
        $crate::i18n::tr($id, Some(&args))
    }};
}

/// Reference language: complete by definition, used for missing messages.
pub const FALLBACK: &str = "en";

/// Built-in languages in menu order.
const BUILTIN: &[(&str, &str)] = &[
    ("ja", include_str!("../locales/ja.ftl")),
    ("en", include_str!("../locales/en.ftl")),
    ("de", include_str!("../locales/de.ftl")),
    ("es", include_str!("../locales/es.ftl")),
    ("fr", include_str!("../locales/fr.ftl")),
    ("it", include_str!("../locales/it.ftl")),
    ("pl", include_str!("../locales/pl.ftl")),
    ("pt-BR", include_str!("../locales/pt-BR.ftl")),
    ("ru", include_str!("../locales/ru.ftl")),
    ("id", include_str!("../locales/id.ftl")),
    ("vi", include_str!("../locales/vi.ftl")),
    ("ko", include_str!("../locales/ko.ftl")),
    ("zh-CN", include_str!("../locales/zh-CN.ftl")),
    ("zh-TW", include_str!("../locales/zh-TW.ftl")),
];

type Bundle = FluentBundle<FluentResource>;

struct State {
    tag: String,
    bundle: Bundle,
    /// English, when the active language isn't English.
    fallback: Option<Bundle>,
}

impl State {
    fn build(tag: &str) -> Self {
        let dirs = user_dirs();
        Self {
            tag: tag.to_owned(),
            bundle: load_bundle(tag, &dirs),
            fallback: (tag != FALLBACK).then(|| load_bundle(FALLBACK, &dirs)),
        }
    }
}

static STATE: LazyLock<RwLock<State>> = LazyLock::new(|| RwLock::new(State::build(FALLBACK)));
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// Looks up and formats a message in the active language (see `t!`).
pub fn tr(id: &str, args: Option<&FluentArgs>) -> String {
    let state = STATE.read().unwrap_or_else(|e| e.into_inner());
    format(&state.bundle, id, args)
        .or_else(|| state.fallback.as_ref().and_then(|b| format(b, id, args)))
        .unwrap_or_else(|| id.to_owned())
}

fn format(bundle: &Bundle, id: &str, args: Option<&FluentArgs>) -> Option<String> {
    let pattern = bundle.get_message(id)?.value()?;
    let mut errors = Vec::new();
    let text = bundle.format_pattern(pattern, args, &mut errors).into_owned();
    if !errors.is_empty() {
        log::debug!("i18n {id}: {errors:?}");
    }
    Some(text)
}

/// Active language tag (e.g. "ja", "pt-BR").
pub fn current() -> String {
    STATE.read().unwrap_or_else(|e| e.into_inner()).tag.clone()
}

/// Changes whenever the active messages change (language switch or reload).
pub fn generation() -> u64 {
    GENERATION.load(Ordering::Relaxed)
}

/// Switches the UI language. Unknown tags fall back to English messages.
pub fn set_language(tag: &str) {
    let state = State::build(tag);
    *STATE.write().unwrap_or_else(|e| e.into_inner()) = state;
    GENERATION.fetch_add(1, Ordering::Relaxed);
    let mut watch = WATCH.lock().unwrap_or_else(|e| e.into_inner());
    watch.stamp = file_stamp(tag);
    log::info!("UI language: {tag}");
}

// ---- Languages ------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Language {
    pub tag: String,
    /// Native name (the file's `language-name` message).
    pub name: String,
}

/// Built-in languages followed by languages that only exist as user files.
pub fn available() -> Vec<Language> {
    let dirs = user_dirs();
    let mut tags: Vec<String> = BUILTIN.iter().map(|(t, _)| (*t).to_owned()).collect();
    let mut extra: Vec<String> = dirs
        .iter()
        .filter_map(|d| std::fs::read_dir(d).ok())
        .flatten()
        .filter_map(|e| {
            let p = e.ok()?.path();
            if p.extension()? != "ftl" {
                return None;
            }
            let stem = p.file_stem()?.to_str()?.to_owned();
            stem.parse::<LanguageIdentifier>().is_ok().then_some(stem)
        })
        .filter(|t| !tags.iter().any(|b| b.eq_ignore_ascii_case(t)))
        .collect();
    extra.sort();
    extra.dedup();
    tags.extend(extra);
    tags.into_iter()
        .map(|tag| {
            let name = sources(&tag, &dirs)
                .iter()
                .rev()
                .find_map(|src| language_name_in(src))
                .unwrap_or_else(|| tag.clone());
            Language { tag, name }
        })
        .collect()
}

/// Value of a plain `language-name = …` line.
fn language_name_in(src: &str) -> Option<String> {
    src.lines().find_map(|l| {
        let v = l.strip_prefix("language-name")?.trim_start().strip_prefix('=')?.trim();
        (!v.is_empty()).then(|| v.to_owned())
    })
}

/// Best UI language for the OS locale settings (`LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, `LANG`…).
pub fn detect() -> String {
    let mut requested: Vec<String> = Vec::new();
    // GNU `LANGUAGE` is a colon-separated priority list (sys-locale ignores it).
    if let Ok(list) = std::env::var("LANGUAGE") {
        requested.extend(list.split(':').filter(|s| !s.is_empty()).map(str::to_owned));
    }
    requested.extend(sys_locale::get_locales());
    let tags: Vec<String> = available().into_iter().map(|l| l.tag).collect();
    let tag = negotiate(&requested, &tags);
    log::info!("OS locales {requested:?} → UI language {tag}");
    tag
}

/// Picks the best of `available` for the user's `requested` locales (in priority order).
pub fn negotiate(requested: &[String], available: &[String]) -> String {
    let avail: Vec<(&String, LanguageIdentifier)> =
        available.iter().filter_map(|t| Some((t, t.parse::<LanguageIdentifier>().ok()?))).collect();
    for req in requested.iter().filter_map(|r| parse_posix_locale(r)) {
        let same_lang = || avail.iter().filter(|(_, a)| a.language == req.language);
        if let Some((t, _)) = avail.iter().find(|(_, a)| *a == req) {
            return (*t).clone();
        }
        if req.language.as_str() == "zh" {
            // Script decides for Chinese: Traditional (Hant / TW / HK / MO) vs Simplified.
            let traditional = req.script.is_some_and(|s| s.as_str() == "Hant")
                || req.region.is_some_and(|r| matches!(r.as_str(), "TW" | "HK" | "MO"));
            let want = if traditional { "TW" } else { "CN" };
            if let Some((t, _)) = same_lang().find(|(_, a)| a.region.is_some_and(|r| r.as_str() == want)) {
                return (*t).clone();
            }
        }
        if let Some((t, _)) = same_lang().find(|(_, a)| a.region.is_some() && a.region == req.region) {
            return (*t).clone();
        }
        if let Some((t, _)) = same_lang().next() {
            return (*t).clone();
        }
    }
    FALLBACK.to_owned()
}

/// `ja_JP.UTF-8`, `de_DE@euro`, `zh-Hant-TW` → language identifier (`C` / `POSIX` → none).
fn parse_posix_locale(s: &str) -> Option<LanguageIdentifier> {
    let s = s.split(['.', '@']).next()?.replace('_', "-");
    if s.is_empty() || s.eq_ignore_ascii_case("c") || s.eq_ignore_ascii_case("posix") {
        return None;
    }
    s.parse().ok()
}

// ---- Loading --------------------------------------------------------------------

/// Directories whose `<tag>.ftl` files override the built-in ones (later wins).
pub fn user_dirs() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(d) = directories::ProjectDirs::from("", "", "rvp") {
        v.push(d.config_dir().join("locales"));
    }
    if let Some(d) = std::env::var_os("RVP_LOCALES_DIR") {
        v.push(PathBuf::from(d));
    }
    v
}

fn builtin(tag: &str) -> Option<&'static str> {
    BUILTIN.iter().find(|(t, _)| t.eq_ignore_ascii_case(tag)).map(|(_, s)| *s)
}

fn user_file(dir: &Path, tag: &str) -> PathBuf {
    dir.join(format!("{tag}.ftl"))
}

/// Sources for `tag`, lowest priority first.
fn sources(tag: &str, dirs: &[PathBuf]) -> Vec<String> {
    let mut v: Vec<String> = builtin(tag).map(str::to_owned).into_iter().collect();
    for d in dirs {
        let path = user_file(d, tag);
        match std::fs::read_to_string(&path) {
            Ok(s) => v.push(s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => log::warn!("cannot read {}: {e}", path.display()),
        }
    }
    v
}

fn load_bundle(tag: &str, dirs: &[PathBuf]) -> Bundle {
    let langid: LanguageIdentifier = tag.parse().unwrap_or_default();
    let mut bundle = FluentBundle::new_concurrent(vec![langid]);
    // Unicode bidi isolation marks would render as boxes in egui.
    bundle.set_use_isolating(false);
    for src in sources(tag, dirs) {
        let res = FluentResource::try_new(src).unwrap_or_else(|(res, errors)| {
            log::warn!("{tag}.ftl: {} syntax error(s); first: {:?}", errors.len(), errors.first());
            res
        });
        bundle.add_resource_overriding(res);
    }
    bundle
}

// ---- Live reload ----------------------------------------------------------------

struct Watch {
    last_check: Option<Instant>,
    stamp: Vec<Option<SystemTime>>,
}

static WATCH: Mutex<Watch> = Mutex::new(Watch { last_check: None, stamp: Vec::new() });

/// Modification times of everything that affects the active messages.
fn file_stamp(tag: &str) -> Vec<Option<SystemTime>> {
    let mtime = |p: &Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    user_dirs()
        .iter()
        .flat_map(|d| [mtime(d), mtime(&user_file(d, tag)), mtime(&user_file(d, FALLBACK))])
        .collect()
}

/// Whether a user locale directory exists (then it is worth polling for changes).
pub fn has_user_files() -> bool {
    user_dirs().iter().any(|d| d.is_dir())
}

/// Re-reads the user locale files if they changed (checked at most once a second).
/// Returns `true` when the messages were reloaded.
pub fn poll_reload() -> bool {
    let tag = current();
    {
        let mut watch = WATCH.lock().unwrap_or_else(|e| e.into_inner());
        if watch.last_check.is_some_and(|t| t.elapsed() < Duration::from_secs(1)) {
            return false;
        }
        watch.last_check = Some(Instant::now());
        let stamp = file_stamp(&tag);
        if stamp == watch.stamp {
            return false;
        }
        watch.stamp = stamp;
    }
    log::info!("locale files changed; reloading {tag}");
    *STATE.write().unwrap_or_else(|e| e.into_inner()) = State::build(&tag);
    GENERATION.fetch_add(1, Ordering::Relaxed);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    /// Message id → variables referenced (from the parsed AST).
    fn messages(src: &str) -> BTreeMap<String, BTreeSet<String>> {
        use fluent_syntax::ast;
        fn vars_in_pattern(p: &ast::Pattern<&str>, out: &mut BTreeSet<String>) {
            for el in &p.elements {
                if let ast::PatternElement::Placeable { expression } = el {
                    vars_in_expr(expression, out);
                }
            }
        }
        fn vars_in_expr(e: &ast::Expression<&str>, out: &mut BTreeSet<String>) {
            match e {
                ast::Expression::Select { selector, variants } => {
                    vars_in_inline(selector, out);
                    for v in variants {
                        vars_in_pattern(&v.value, out);
                    }
                }
                ast::Expression::Inline(i) => vars_in_inline(i, out),
            }
        }
        fn vars_in_inline(i: &ast::InlineExpression<&str>, out: &mut BTreeSet<String>) {
            match i {
                ast::InlineExpression::VariableReference { id } => {
                    out.insert(id.name.to_owned());
                }
                ast::InlineExpression::Placeable { expression } => vars_in_expr(expression, out),
                _ => {}
            }
        }
        let res = fluent_syntax::parser::parse(src).unwrap_or_else(|(_, e)| panic!("syntax errors: {e:?}"));
        res.body
            .iter()
            .filter_map(|entry| match entry {
                ast::Entry::Message(m) => {
                    let mut vars = BTreeSet::new();
                    if let Some(v) = &m.value {
                        vars_in_pattern(v, &mut vars);
                    }
                    Some((m.id.name.to_owned(), vars))
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn every_locale_is_complete_and_uses_the_same_variables() {
        let reference = messages(builtin(FALLBACK).unwrap());
        assert!(reference.len() > 100);
        for (tag, src) in BUILTIN {
            let msgs = messages(src);
            let missing: Vec<_> = reference.keys().filter(|k| !msgs.contains_key(*k)).collect();
            let extra: Vec<_> = msgs.keys().filter(|k| !reference.contains_key(*k)).collect();
            assert!(missing.is_empty() && extra.is_empty(), "{tag}: missing {missing:?}, extra {extra:?}");
            for (id, vars) in &reference {
                assert_eq!(&msgs[id], vars, "{tag}: variables of {id}");
            }
        }
    }

    #[test]
    fn every_locale_formats_without_errors() {
        for (tag, _) in BUILTIN {
            let bundle = load_bundle(tag, &[]);
            let mut args = FluentArgs::new();
            args.set("count", 3);
            args.set("percent", "50");
            args.set("sign", "+");
            args.set("value", "0.5");
            for name in ["language-name", "osd-added", "volume-level", "delay-seconds", "about-notice"] {
                let text = format(&bundle, name, Some(&args)).unwrap_or_else(|| panic!("{tag}: {name}"));
                assert!(!text.contains('{') && !text.contains('\u{2068}'), "{tag}: {name} = {text}");
            }
            let one = format(&bundle, "osd-added", Some(&FluentArgs::from_iter([("count", 1)]))).unwrap();
            assert!(!one.is_empty());
        }
    }

    #[test]
    fn language_names_are_native() {
        let names: BTreeMap<_, _> = BUILTIN.iter().map(|(t, s)| (*t, language_name_in(s).unwrap())).collect();
        assert_eq!(names["ja"], "日本語");
        assert_eq!(names["en"], "English");
        assert_eq!(names["de"], "Deutsch");
    }

    #[test]
    fn negotiation() {
        let avail: Vec<String> = BUILTIN.iter().map(|(t, _)| (*t).to_owned()).collect();
        let pick = |req: &[&str]| negotiate(&req.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>(), &avail);
        assert_eq!(pick(&["ja_JP.UTF-8"]), "ja");
        assert_eq!(pick(&["de-AT"]), "de");
        assert_eq!(pick(&["pt_PT.UTF-8"]), "pt-BR");
        assert_eq!(pick(&["zh_CN.UTF-8"]), "zh-CN");
        assert_eq!(pick(&["zh_TW.UTF-8"]), "zh-TW");
        assert_eq!(pick(&["zh-HK"]), "zh-TW");
        assert_eq!(pick(&["zh-Hant"]), "zh-TW");
        assert_eq!(pick(&["zh-SG"]), "zh-CN");
        assert_eq!(pick(&["C"]), "en");
        assert_eq!(pick(&["C.UTF-8", "fr_CA"]), "fr");
        assert_eq!(pick(&["xx", "ko_KR"]), "ko");
        assert_eq!(pick(&["sv_SE.UTF-8"]), "en");
        assert_eq!(pick(&[]), "en");
    }

    #[test]
    fn posix_locales_parse() {
        assert_eq!(parse_posix_locale("de_DE@euro").unwrap().to_string(), "de-DE");
        assert!(parse_posix_locale("POSIX").is_none());
    }
}
