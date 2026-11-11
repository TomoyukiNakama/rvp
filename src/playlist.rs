//! Playlist model: ordering, next/previous with repeat & shuffle, folder expansion.

use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "m4v", "mkv", "webm", "mov", "avi", "wmv", "flv", "ts", "m2ts", "mts", "mpg", "mpeg", "ogv", "3gp", "vob",
];

pub fn is_video(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| VIDEO_EXTENSIONS.iter().any(|v| v.eq_ignore_ascii_case(e)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Repeat {
    #[default]
    Off,
    One,
    All,
}

impl Repeat {
    /// Off → this video → whole playlist → off.
    pub fn cycle(self) -> Self {
        match self {
            Repeat::Off => Repeat::One,
            Repeat::One => Repeat::All,
            Repeat::All => Repeat::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Repeat::Off => "ループ: オフ",
            Repeat::One => "ループ: この動画",
            Repeat::All => "ループ: プレイリスト全体",
        }
    }
}

#[derive(Debug, Default)]
pub struct Playlist {
    items: Vec<PathBuf>,
    /// Index into `items` of the current entry.
    current: Option<usize>,
    pub repeat: Repeat,
    shuffle: bool,
    /// Play order when shuffling (indices into `items`).
    order: Vec<usize>,
    rng: u64,
}

/// "file2" < "file10", case-insensitive.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let take_num = |it: &mut std::iter::Peekable<std::str::Chars>| {
                    let mut s = String::new();
                    while let Some(c) = it.peek().copied().filter(char::is_ascii_digit) {
                        s.push(c);
                        it.next();
                    }
                    s
                };
                let (na, nb) = (take_num(&mut a), take_num(&mut b));
                let (ta, tb) = (na.trim_start_matches('0'), nb.trim_start_matches('0'));
                let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb)).then_with(|| na.len().cmp(&nb.len()));
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (Some(x), Some(y)) => {
                let ord = x.to_lowercase().cmp(y.to_lowercase());
                if ord != Ordering::Equal {
                    return ord;
                }
                a.next();
                b.next();
            }
        }
    }
}

fn file_name(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Videos directly inside `dir`, naturally sorted.
pub fn videos_in(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_file() && is_video(p)).collect())
        .unwrap_or_default();
    v.sort_by(|a, b| natural_cmp(&file_name(a), &file_name(b)));
    v
}

/// Expands dropped/opened paths: directories become their videos.
pub fn expand(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for p in paths {
        if p.is_dir() {
            out.extend(videos_in(&p));
        } else {
            out.push(p);
        }
    }
    out
}

impl Playlist {
    /// Replaces the contents. A single file with `with_siblings` becomes its
    /// folder's videos, positioned at that file.
    pub fn set(&mut self, paths: Vec<PathBuf>, with_siblings: bool) {
        let paths = expand(paths);
        let (items, current) = match paths.as_slice() {
            [single] if with_siblings => {
                let siblings = single.parent().map(videos_in).unwrap_or_default();
                let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
                let me = canon(single);
                match siblings.iter().position(|s| canon(s) == me) {
                    Some(i) => (siblings, Some(i)),
                    None => (paths.clone(), Some(0)),
                }
            }
            _ => {
                let cur = (!paths.is_empty()).then_some(0);
                (paths, cur)
            }
        };
        self.items = items;
        self.current = current;
        self.reshuffle();
    }

    /// Appends (expanding folders). Returns how many were added.
    pub fn append(&mut self, paths: Vec<PathBuf>) -> usize {
        let new = expand(paths);
        let n = new.len();
        self.items.extend(new);
        if self.current.is_none() && !self.items.is_empty() {
            self.current = Some(0);
        }
        self.reshuffle();
        n
    }

    pub fn items(&self) -> &[PathBuf] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current
    }

    pub fn current(&self) -> Option<&Path> {
        self.current.and_then(|i| self.items.get(i)).map(PathBuf::as_path)
    }

    pub fn select(&mut self, index: usize) -> Option<&Path> {
        if index < self.items.len() {
            self.current = Some(index);
        }
        self.current()
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn set_shuffle(&mut self, on: bool) {
        self.shuffle = on;
        self.reshuffle();
    }

    fn sequence(&self) -> Vec<usize> {
        if self.shuffle { self.order.clone() } else { (0..self.items.len()).collect() }
    }

    /// Entry after the current one, honouring shuffle and `Repeat::All`
    /// (not `Repeat::One`: explicit "next" always moves on).
    fn neighbour(&self, forward: bool) -> Option<usize> {
        let seq = self.sequence();
        let cur = self.current?;
        let pos = seq.iter().position(|&i| i == cur)?;
        let n = seq.len();
        let next = if forward {
            if pos + 1 < n {
                Some(pos + 1)
            } else {
                (self.repeat == Repeat::All).then_some(0)
            }
        } else if pos > 0 {
            Some(pos - 1)
        } else {
            (self.repeat == Repeat::All).then(|| n - 1)
        };
        next.map(|p| seq[p])
    }

    /// Moves to the next entry (user action). `None` at the end without repeat.
    pub fn next(&mut self) -> Option<&Path> {
        let i = self.neighbour(true)?;
        self.current = Some(i);
        self.current()
    }

    pub fn prev(&mut self) -> Option<&Path> {
        let i = self.neighbour(false)?;
        self.current = Some(i);
        self.current()
    }

    pub fn has_next(&self) -> bool {
        self.neighbour(true).is_some()
    }

    /// What to play when the current entry finishes on its own.
    pub fn on_finished(&mut self) -> Option<&Path> {
        if self.repeat == Repeat::One {
            return self.current();
        }
        self.next()
    }

    pub fn remove(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }
        self.items.remove(index);
        self.current = match self.current {
            _ if self.items.is_empty() => None,
            Some(c) if c > index => Some(c - 1),
            Some(c) if c == index => Some(c.min(self.items.len() - 1)),
            other => other,
        };
        self.reshuffle();
    }

    /// Moves the entry at `from` so it ends up at index `to`.
    pub fn move_item(&mut self, from: usize, to: usize) {
        if from >= self.items.len() || to >= self.items.len() || from == to {
            return;
        }
        let item = self.items.remove(from);
        self.items.insert(to, item);
        if let Some(c) = self.current {
            self.current = Some(if c == from {
                to
            } else if from < c && to >= c {
                c - 1
            } else if from > c && to <= c {
                c + 1
            } else {
                c
            });
        }
        self.reshuffle();
    }

    /// New random order with the current entry first.
    fn reshuffle(&mut self) {
        let n = self.items.len();
        self.order = (0..n).collect();
        if !self.shuffle || n < 2 {
            return;
        }
        if self.rng == 0 {
            self.rng = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0x9E37_79B9_7F4A_7C15, |d| d.as_nanos() as u64)
                | 1;
        }
        for i in (1..n).rev() {
            // xorshift64
            self.rng ^= self.rng << 13;
            self.rng ^= self.rng >> 7;
            self.rng ^= self.rng << 17;
            let j = (self.rng % (i as u64 + 1)) as usize;
            self.order.swap(i, j);
        }
        if let Some(c) = self.current
            && let Some(p) = self.order.iter().position(|&i| i == c)
        {
            self.order.swap(0, p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pl(names: &[&str]) -> Playlist {
        let mut p = Playlist::default();
        p.set(names.iter().map(PathBuf::from).collect(), false);
        p
    }

    fn cur(p: &Playlist) -> &str {
        p.current().unwrap().to_str().unwrap()
    }

    #[test]
    fn natural_ordering() {
        let mut v = vec!["ep10.mkv", "ep2.mkv", "Ep1.mkv", "ep02b.mkv", "a.mkv"];
        v.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(v, ["a.mkv", "Ep1.mkv", "ep2.mkv", "ep02b.mkv", "ep10.mkv"]);
    }

    #[test]
    fn next_prev_and_repeat() {
        let mut p = pl(&["a", "b", "c"]);
        assert_eq!(cur(&p), "a");
        assert!(p.prev().is_none());
        assert_eq!(p.next().unwrap().to_str(), Some("b"));
        p.next();
        assert!(p.next().is_none(), "end without repeat");
        assert_eq!(cur(&p), "c");

        p.repeat = Repeat::All;
        assert_eq!(p.next().unwrap().to_str(), Some("a"));
        assert_eq!(p.prev().unwrap().to_str(), Some("c"));

        p.repeat = Repeat::One;
        assert_eq!(p.on_finished().unwrap().to_str(), Some("c"));
        assert!(p.next().is_none(), "explicit next ignores repeat-one");
    }

    #[test]
    fn remove_and_move_keep_current() {
        let mut p = pl(&["a", "b", "c", "d"]);
        p.select(2); // c
        p.remove(0);
        assert_eq!(cur(&p), "c");
        p.move_item(1, 0); // c to front
        assert_eq!(cur(&p), "c");
        assert_eq!(p.items()[0].to_str(), Some("c"));
        p.move_item(2, 0); // d to front: c shifts right
        assert_eq!(cur(&p), "c");
        p.remove(p.current_index().unwrap());
        assert_eq!(p.len(), 2);
        assert!(p.current().is_some());
    }

    #[test]
    fn shuffle_visits_everything_once() {
        let names: Vec<String> = (0..20).map(|i| format!("{i}")).collect();
        let mut p = pl(&names.iter().map(String::as_str).collect::<Vec<_>>());
        p.select(5);
        p.set_shuffle(true);
        let mut seen = vec![cur(&p).to_owned()];
        while let Some(n) = p.next() {
            seen.push(n.to_string_lossy().into_owned());
        }
        assert_eq!(seen[0], "5");
        seen.sort();
        let mut all = names.clone();
        all.sort();
        assert_eq!(seen, all);
    }

    #[test]
    fn single_file_expands_to_folder() {
        let dir = std::env::temp_dir().join(format!("rvp-pl-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for n in ["ep10.mp4", "ep2.mp4", "ep1.mkv", "notes.txt"] {
            std::fs::write(dir.join(n), b"").unwrap();
        }
        let mut p = Playlist::default();
        p.set(vec![dir.join("ep2.mp4")], true);
        let names: Vec<_> = p.items().iter().map(|p| file_name(p)).collect();
        assert_eq!(names, ["ep1.mkv", "ep2.mp4", "ep10.mp4"]);
        assert_eq!(p.current_index(), Some(1));

        p.set(vec![dir.clone()], true);
        assert_eq!(p.len(), 3);
        assert_eq!(p.current_index(), Some(0));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
