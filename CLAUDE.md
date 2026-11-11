# rvp — Rust + egui video player

Spec and roadmap: `SPEC.md` (Japanese). Phases are listed in SPEC §6; keep SPEC in sync when decisions change.

## Commands
- Build / run: `cargo build`, `cargo run -- <video>` (dev profile is optimized enough for real-time playback)
- Test: `cargo test` (player integration test generates a clip via the `ffmpeg` CLI; skips if missing)
- Test media: `scripts/gen-test-media.sh` → `target/test-media/`
- Screenshot without a compositor tool (grim doesn't work on GNOME/Wayland):
  `RVP_SCREENSHOT=out.png RVP_SCREENSHOT_AFTER=2 RVP_SCREENSHOT_EXIT=1 cargo run -- <video>` (see `src/debug_shot.rs`)

## Environment notes
- Linux / Wayland (GNOME). System FFmpeg 8 (`ffmpeg-next` 9). Bash sandbox (bubblewrap) can't start here, so commands run unsandboxed.
- egui/eframe are pinned to **0.36** because `egui-phosphor` 0.14 depends on egui 0.36. Bump them together.
- eframe 0.36 uses `App::ui(&mut self, ui, frame)` (not `update`).

## Architecture (src/)
- `player/` — engine. `Player` (UI-thread API) + `demux.rs` thread + `video_decode.rs` thread, linked by bounded crossbeam channels.
  Seeks bump a **serial**; stale packets/frames are dropped at every stage. Clock in `clock.rs` (audio will become master in phase 2).
- `app.rs` — `eframe::App`: input, texture upload, layout of chrome over the video.
- `ui/` — `window.rs` (borderless window: title bar, resize grips, aspect fit), `controls.rs` (floating bar), `seekbar.rs`, `widgets.rs`.
- `theme.rs` — design tokens (colors/sizes/motion) + egui style. Dark theme only. Use tokens, never ad-hoc colors.
  Translucent colors must use `from_rgba_unmultiplied_const` (premultiplied with rgb > a renders as additive white).
- `fonts.rs` — CJK font via `fc-match`, Phosphor icon font (`ui::widgets::icon_font`).

## Conventions
- UI strings are Japanese.
- Window is borderless + transparent; the rounded window body is painted by us (`clear_color` is transparent). Corner radius is 0 when maximized/fullscreen.
- Verify UI changes with a screenshot, not just a successful build.
