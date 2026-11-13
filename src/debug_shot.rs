//! Development aid: save a screenshot of the window to a PNG file.
//!
//! - `RVP_SCREENSHOT=/path/out.png` enables it
//! - `RVP_SCREENSHOT_AFTER=<secs>` delay after startup (default 2.0)
//! - `RVP_SCREENSHOT_EXIT=1` close the app once the file is written
//! - `RVP_DEBUG_PLAYLIST=1` start with the playlist panel open (read in app.rs)
//! - `RVP_DEBUG_SPEED=<x>` start at the given playback speed (read in app.rs)
//! - `RVP_DEBUG_POPUP=speed|tracks|language` open that menu once at startup
//! - `RVP_DEBUG_LANG=<tag>` use that UI language (not saved; read in app.rs)
//! - `RVP_DEBUG_ABOUT=license|software|<component name>` open the about dialog (read in app.rs)
//!
//! Needed because compositor-level tools (e.g. `grim`) don't work on GNOME/Wayland.

use std::path::PathBuf;
use std::time::Duration;

pub struct DebugShot {
    path: PathBuf,
    after: f64,
    exit: bool,
    requested: bool,
}

impl DebugShot {
    pub fn from_env() -> Option<Self> {
        let path = PathBuf::from(std::env::var_os("RVP_SCREENSHOT")?);
        let after = std::env::var("RVP_SCREENSHOT_AFTER")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2.0);
        let exit = std::env::var("RVP_SCREENSHOT_EXIT").is_ok_and(|v| v == "1");
        Some(Self {
            path,
            after,
            exit,
            requested: false,
        })
    }

    /// Call once per frame.
    pub fn tick(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        if !self.requested {
            if now >= self.after {
                self.requested = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            } else {
                ctx.request_repaint_after(Duration::from_secs_f64(self.after - now));
            }
            return;
        }
        let shot = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = shot {
            let [w, h] = img.size;
            let bytes: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect();
            match image::save_buffer(&self.path, &bytes, w as u32, h as u32, image::ExtendedColorType::Rgba8) {
                Ok(()) => log::info!("screenshot saved: {}", self.path.display()),
                Err(e) => log::error!("screenshot failed: {e}"),
            }
            if self.exit {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

/// Opens the named popup once (for screenshots of menus).
pub fn maybe_open_popup(ctx: &egui::Context, name: &str, button: &egui::Response) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static DONE: AtomicBool = AtomicBool::new(false);
    if DONE.load(Ordering::Relaxed) || std::env::var("RVP_DEBUG_POPUP").ok().as_deref() != Some(name) {
        return;
    }
    DONE.store(true, Ordering::Relaxed);
    egui::Popup::open_id(ctx, egui::Popup::default_response_id(button));
}
