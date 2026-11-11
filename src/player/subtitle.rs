//! Text subtitles: cue storage, file parsers (SRT / ASS / WebVTT), ASS tag
//! stripping, and charset detection for external files.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

/// Cues sorted by start time. Embedded tracks fill up incrementally (and
/// may receive the same cue again after a seek), so inserts deduplicate.
#[derive(Debug, Default, Clone)]
pub struct SubtitleTrack {
    cues: Vec<Cue>,
    /// Longest cue duration, bounding how far back `active` must look.
    max_len: f64,
}

impl SubtitleTrack {
    pub fn from_cues(mut cues: Vec<Cue>) -> Self {
        cues.retain(|c| c.end > c.start && !c.text.trim().is_empty());
        cues.sort_by(|a, b| a.start.total_cmp(&b.start));
        let max_len = cues.iter().map(|c| c.end - c.start).fold(0.0, f64::max);
        Self { cues, max_len }
    }

    pub fn insert(&mut self, cue: Cue) {
        if cue.end <= cue.start || cue.text.trim().is_empty() {
            return;
        }
        let i = self.cues.partition_point(|c| c.start < cue.start);
        let dup = self.cues[i..]
            .iter()
            .take_while(|c| c.start == cue.start)
            .any(|c| c.text == cue.text);
        if !dup {
            self.max_len = self.max_len.max(cue.end - cue.start);
            self.cues.insert(i, cue);
        }
    }

    pub fn len(&self) -> usize {
        self.cues.len()
    }

    /// Texts of all cues showing at `t`, in start order.
    pub fn active(&self, t: f64) -> Vec<&str> {
        let hi = self.cues.partition_point(|c| c.start <= t);
        let lo = self.cues[..hi].partition_point(|c| c.start < t - self.max_len);
        self.cues[lo..hi].iter().filter(|c| t < c.end).map(|c| c.text.as_str()).collect()
    }
}

// ---- Text cleanup --------------------------------------------------------------

/// Strips ASS override blocks (`{\an8\i1}`) and converts `\N`, `\n`, `\h`.
pub fn ass_to_plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' => depth += 1,
            '}' if depth > 0 => depth -= 1,
            _ if depth > 0 => {}
            '\\' => match chars.peek() {
                Some('N') | Some('n') => {
                    chars.next();
                    out.push('\n');
                }
                Some('h') => {
                    chars.next();
                    out.push('\u{a0}');
                }
                _ => out.push('\\'),
            },
            _ => out.push(c),
        }
    }
    out.trim().to_owned()
}

/// Removes simple HTML-ish tags used in SRT/VTT (`<i>`, `<font color=..>`, `<c.x>`).
pub fn strip_markup(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_tag = false;
    for c in text.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if in_tag => {}
            _ => out.push(c),
        }
    }
    // SRT files sometimes carry ASS-style tags too.
    ass_to_plain(&out.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">"))
}

/// The text field of an ASS `Dialogue` event as produced by FFmpeg's decoders
/// (`ReadOrder,Layer,Style,Name,MarginL,MarginR,MarginV,Effect,Text`).
pub fn ass_event_text(event: &str) -> String {
    let body = event.strip_prefix("Dialogue:").unwrap_or(event).trim_start();
    ass_to_plain(body.splitn(9, ',').nth(8).unwrap_or(body))
}

// ---- Files ---------------------------------------------------------------------

/// Decodes subtitle file bytes: BOM (UTF-8/16), UTF-8, else Shift_JIS
/// (the usual legacy encoding for Japanese subtitles).
pub fn decode_text(bytes: &[u8]) -> String {
    if let Some((enc, bom_len)) = encoding_rs::Encoding::for_bom(bytes) {
        return enc.decode_without_bom_handling(&bytes[bom_len..]).0.into_owned();
    }
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_owned(),
        Err(_) => encoding_rs::SHIFT_JIS.decode_without_bom_handling(bytes).0.into_owned(),
    }
}

/// `00:01:02,500` / `00:01:02.500` / `1:02.5` / ASS `0:01:02.50`.
fn parse_timestamp(s: &str) -> Option<f64> {
    let s = s.trim().replace(',', ".");
    let parts: Vec<&str> = s.split(':').collect();
    let secs: f64 = parts.last()?.parse().ok()?;
    let mut total = secs;
    let mut mul = 60.0;
    for p in parts.iter().rev().skip(1) {
        total += p.trim().parse::<f64>().ok()? * mul;
        mul *= 60.0;
    }
    Some(total)
}

pub fn parse_srt(text: &str) -> Vec<Cue> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut cues = Vec::new();
    for block in text.split("\n\n") {
        let mut lines = block.lines().skip_while(|l| l.trim().is_empty());
        let Some(mut first) = lines.next() else { continue };
        // Optional index line; WebVTT cue identifiers look the same.
        if !first.contains("-->") {
            match lines.next() {
                Some(l) => first = l,
                None => continue,
            }
        }
        let Some((a, b)) = first.split_once("-->") else { continue };
        // VTT may append cue settings after the end time.
        let b = b.split_whitespace().next().unwrap_or("");
        let (Some(start), Some(end)) = (parse_timestamp(a), parse_timestamp(b)) else { continue };
        let body: Vec<&str> = lines.collect();
        cues.push(Cue { start, end, text: strip_markup(&body.join("\n")) });
    }
    cues
}

pub fn parse_ass(text: &str) -> Vec<Cue> {
    let mut cues = Vec::new();
    let mut in_events = false;
    // Column positions from the `Format:` line (defaults per spec).
    let (mut i_start, mut i_end, mut i_text, mut n_fields) = (1, 2, 9, 10);
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_events = line.eq_ignore_ascii_case("[events]");
            continue;
        }
        if !in_events {
            continue;
        }
        if let Some(fmt) = line.strip_prefix("Format:") {
            let cols: Vec<String> = fmt.split(',').map(|c| c.trim().to_ascii_lowercase()).collect();
            let find = |name: &str| cols.iter().position(|c| c == name);
            if let (Some(s), Some(e), Some(t)) = (find("start"), find("end"), find("text")) {
                (i_start, i_end, i_text, n_fields) = (s, e, t, cols.len());
            }
        } else if let Some(ev) = line.strip_prefix("Dialogue:") {
            let fields: Vec<&str> = ev.splitn(n_fields, ',').collect();
            if fields.len() <= i_text.max(i_start).max(i_end) {
                continue;
            }
            if let (Some(start), Some(end)) = (parse_timestamp(fields[i_start]), parse_timestamp(fields[i_end])) {
                cues.push(Cue { start, end, text: ass_to_plain(fields[i_text]) });
            }
        }
    }
    cues
}

pub const SUBTITLE_EXTENSIONS: &[&str] = &["srt", "ass", "ssa", "vtt"];

pub fn is_subtitle_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| SUBTITLE_EXTENSIONS.iter().any(|s| s.eq_ignore_ascii_case(e)))
}

pub fn load_file(path: &Path) -> std::io::Result<SubtitleTrack> {
    let text = decode_text(&std::fs::read(path)?);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let cues = match ext.as_str() {
        "ass" | "ssa" => parse_ass(&text),
        _ => parse_srt(text.strip_prefix("WEBVTT").unwrap_or(&text)),
    };
    Ok(SubtitleTrack::from_cues(cues))
}

/// Subtitle files next to `video` sharing its stem: `movie.srt`, `movie.ja.ass`, …
/// Sorted so that an exact-stem match comes first.
pub fn find_external(video: &Path) -> Vec<PathBuf> {
    let (Some(dir), Some(stem)) = (video.parent(), video.file_stem().and_then(|s| s.to_str())) else {
        return Vec::new();
    };
    let dir = if dir.as_os_str().is_empty() { Path::new(".") } else { dir };
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| is_subtitle_file(p))
                .filter(|p| {
                    p.file_stem().and_then(|s| s.to_str()).is_some_and(|s| s == stem || s.starts_with(&format!("{stem}.")))
                })
                .collect()
        })
        .unwrap_or_default();
    found.sort_by_key(|p| (p.file_stem().and_then(|s| s.to_str()) != Some(stem), p.clone()));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srt_parsing_with_markup_and_crlf() {
        let srt = "1\r\n00:00:01,000 --> 00:00:02,500\r\n<i>Hello</i>\r\nworld\r\n\r\n2\r\n00:00:03,000 --> 00:00:04,000\r\n{\\an8}上だよ\r\n";
        let cues = parse_srt(srt);
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0], Cue { start: 1.0, end: 2.5, text: "Hello\nworld".into() });
        assert_eq!(cues[1].text, "上だよ");
    }

    #[test]
    fn vtt_with_settings_and_ids() {
        let vtt = "WEBVTT\n\nintro\n00:01.000 --> 00:02.000 align:start\n<c.yellow>Hi</c>\n";
        let cues = parse_srt(vtt.strip_prefix("WEBVTT").unwrap());
        assert_eq!(cues, vec![Cue { start: 1.0, end: 2.0, text: "Hi".into() }]);
    }

    #[test]
    fn ass_file_and_event_text() {
        let ass = "[Script Info]\nTitle: x\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.50,0:00:03.00,Default,,0,0,0,,{\\i1}こんにちは{\\i0}\\N二行目, です\n";
        let cues = parse_ass(ass);
        assert_eq!(cues, vec![Cue { start: 1.5, end: 3.0, text: "こんにちは\n二行目, です".into() }]);
        // FFmpeg's decoded event layout (ReadOrder first).
        assert_eq!(ass_event_text("3,0,Default,,0,0,0,,{\\b1}Bold\\Nline"), "Bold\nline");
    }

    #[test]
    fn shift_jis_and_bom_detection() {
        let (sjis, _, _) = encoding_rs::SHIFT_JIS.encode("字幕テスト");
        assert_eq!(decode_text(&sjis), "字幕テスト");
        let mut bom = vec![0xEF, 0xBB, 0xBF];
        bom.extend_from_slice("ok".as_bytes());
        assert_eq!(decode_text(&bom), "ok");
    }

    #[test]
    fn active_cues_overlap_and_dedupe() {
        let mut t = SubtitleTrack::default();
        t.insert(Cue { start: 0.0, end: 10.0, text: "long".into() });
        t.insert(Cue { start: 2.0, end: 3.0, text: "short".into() });
        t.insert(Cue { start: 2.0, end: 3.0, text: "short".into() }); // re-delivered after seek
        assert_eq!(t.len(), 2);
        assert_eq!(t.active(2.5), vec!["long", "short"]);
        assert_eq!(t.active(5.0), vec!["long"]);
        assert!(t.active(10.0).is_empty());
    }

    #[test]
    fn external_files_are_found_by_stem() {
        let dir = std::env::temp_dir().join(format!("rvp-sub-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for n in ["movie.mkv", "movie.ja.srt", "movie.srt", "movie2.srt", "other.ass"] {
            std::fs::write(dir.join(n), b"").unwrap();
        }
        let found: Vec<String> = find_external(&dir.join("movie.mkv"))
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(found, ["movie.srt", "movie.ja.srt"]);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
