//! Settings (`~/.config/rvp/config.toml`) and resume positions
//! (`~/.local/state/rvp/resume.toml`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::player::HwDec;
use crate::playlist::Repeat;

fn dirs() -> Option<directories::ProjectDirs> {
    directories::ProjectDirs::from("", "", "rvp")
}

/// Writes atomically (temp file + rename) so a crash never leaves a torn file.
fn write_atomic(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, path)
}

fn load_toml<T: for<'de> Deserialize<'de> + Default>(path: Option<PathBuf>) -> T {
    let Some(path) = path else { return T::default() };
    match std::fs::read_to_string(&path) {
        Ok(s) => toml::from_str(&s).unwrap_or_else(|e| {
            log::warn!("ignoring invalid {}: {e}", path.display());
            T::default()
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => T::default(),
        Err(e) => {
            log::warn!("cannot read {}: {e}", path.display());
            T::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// 0.0..=1.5
    pub volume: f32,
    pub muted: bool,
    pub repeat: Repeat,
    pub shuffle: bool,
    /// Opening a single file queues the other videos in its folder.
    pub playlist_from_folder: bool,
    /// Resume files where they were left off.
    pub resume: bool,
    /// Seek step for ←/→ and Shift+←/→ in seconds.
    pub seek_small: f64,
    pub seek_large: f64,
    /// Hardware decoding: auto / off / cuda / vaapi.
    pub hwdec: HwDec,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            volume: 1.0,
            muted: false,
            repeat: Repeat::Off,
            shuffle: false,
            playlist_from_folder: true,
            resume: true,
            seek_small: 5.0,
            seek_large: 60.0,
            hwdec: HwDec::Auto,
        }
    }
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        Some(dirs()?.config_dir().join("config.toml"))
    }

    pub fn load() -> Self {
        let mut c: Self = load_toml(Self::path());
        c.volume = c.volume.clamp(0.0, 1.5);
        c
    }

    pub fn save(&self) {
        let Some(path) = Self::path() else { return };
        let text = match toml::to_string_pretty(self) {
            Ok(t) => t,
            Err(e) => return log::error!("config serialize: {e}"),
        };
        if let Err(e) = write_atomic(&path, &text) {
            log::error!("cannot write {}: {e}", path.display());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResumeEntry {
    position: f64,
    /// Unix seconds, for pruning the oldest entries.
    updated: u64,
}

/// Last playback position per file.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ResumeStore {
    #[serde(default)]
    files: BTreeMap<String, ResumeEntry>,
    #[serde(skip)]
    path: Option<PathBuf>,
    #[serde(skip)]
    dirty: bool,
}

/// Don't bother resuming near the start, or near the end (credits):
/// within `END_MARGIN` seconds or the last `END_FRACTION`, whichever is shorter.
const MIN_POSITION: f64 = 10.0;
const END_MARGIN: f64 = 15.0;
const END_FRACTION: f64 = 0.05;
const MAX_ENTRIES: usize = 1000;

fn key(path: &Path) -> String {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()).to_string_lossy().into_owned()
}

impl ResumeStore {
    pub fn load() -> Self {
        let path = dirs().map(|d| d.state_dir().unwrap_or(d.data_local_dir()).join("resume.toml"));
        Self::load_from(path)
    }

    fn load_from(path: Option<PathBuf>) -> Self {
        let mut s: Self = load_toml(path.clone());
        s.path = path;
        s
    }

    pub fn get(&self, file: &Path) -> Option<f64> {
        self.files.get(&key(file)).map(|e| e.position)
    }

    /// Records `position`; clears the entry when it's not worth resuming.
    pub fn record(&mut self, file: &Path, position: f64, duration: Option<f64>) {
        log::debug!("resume record {} at {position:.1}s", file.display());
        let k = key(file);
        let near_end = duration.is_some_and(|d| position > d - END_MARGIN.min(d * END_FRACTION));
        if position < MIN_POSITION || near_end {
            self.dirty |= self.files.remove(&k).is_some();
            return;
        }
        let updated = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
        self.files.insert(k, ResumeEntry { position, updated });
        self.dirty = true;
        if self.files.len() > MAX_ENTRIES {
            let mut by_age: Vec<(u64, String)> = self.files.iter().map(|(k, e)| (e.updated, k.clone())).collect();
            by_age.sort();
            for (_, k) in by_age.into_iter().take(self.files.len() - MAX_ENTRIES) {
                self.files.remove(&k);
            }
        }
    }

    pub fn forget(&mut self, file: &Path) {
        self.dirty |= self.files.remove(&key(file)).is_some();
    }

    pub fn save(&mut self) {
        if !self.dirty {
            return;
        }
        let Some(path) = &self.path else { return };
        match toml::to_string(self) {
            Ok(text) => match write_atomic(path, &text) {
                Ok(()) => self.dirty = false,
                Err(e) => log::error!("cannot write {}: {e}", path.display()),
            },
            Err(e) => log::error!("resume serialize: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrip_and_partial_files() {
        let c = Config { volume: 0.4, repeat: Repeat::All, ..Default::default() };
        let back: Config = toml::from_str(&toml::to_string_pretty(&c).unwrap()).unwrap();
        assert_eq!(back, c);
        // Unknown/missing keys fall back to defaults (hand-edited files).
        let partial: Config = toml::from_str("muted = true\nfuture_option = 3\n").unwrap();
        assert!(partial.muted);
        assert_eq!(partial.volume, 1.0);
    }

    #[test]
    fn resume_rules_and_persistence() {
        let dir = std::env::temp_dir().join(format!("rvp-resume-{}", std::process::id()));
        let path = dir.join("resume.toml");
        let mut s = ResumeStore::load_from(Some(path.clone()));
        let f = Path::new("/videos/movie.mkv");

        s.record(f, 5.0, Some(600.0));
        assert_eq!(s.get(f), None, "too close to the start");
        s.record(f, 595.0, Some(600.0));
        assert_eq!(s.get(f), None, "too close to the end");
        s.record(f, 123.5, Some(600.0));
        assert_eq!(s.get(f), Some(123.5));
        // Short clips: the end margin shrinks with the duration.
        let short = Path::new("/videos/clip.mp4");
        s.record(short, 12.0, Some(20.0));
        assert_eq!(s.get(short), Some(12.0));
        s.record(short, 19.5, Some(20.0));
        assert_eq!(s.get(short), None);
        s.save();

        let mut s2 = ResumeStore::load_from(Some(path));
        assert_eq!(s2.get(f), Some(123.5));
        s2.forget(f);
        assert_eq!(s2.get(f), None);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
