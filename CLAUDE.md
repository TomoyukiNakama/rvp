# rvp — Rust + egui video player

Spec and roadmap: `SPEC.md` (Japanese). Phases are listed in SPEC §6; keep SPEC in sync when decisions change.

## Commands
- Build / run: `cargo build`, `cargo run -- <video>` (dev profile is optimized enough for real-time playback)
- Test: `cargo test` (player integration test generates a clip via the `ffmpeg` CLI; skips if missing)
- Test media: `scripts/gen-test-media.sh` → `target/test-media/`
- `cargo test` does **not** rebuild `target/debug/rvp`; run `cargo build` before launching the app.
- Keep manual runs out of the real config/state: `XDG_CONFIG_HOME=$TMPDIR/xdg/config XDG_STATE_HOME=$TMPDIR/xdg/state`.
  `RVP_DEBUG_PLAYLIST=1` starts with the playlist panel open, `RVP_DEBUG_SPEED=2` at 2× (for screenshots).
- Screenshot without a compositor tool (grim doesn't work on GNOME/Wayland):
  `RVP_SCREENSHOT=out.png RVP_SCREENSHOT_AFTER=2 RVP_SCREENSHOT_EXIT=1 cargo run -- <video>` (see `src/debug_shot.rs`)

## Environment notes
- Linux / Wayland (GNOME). System FFmpeg 8 (`ffmpeg-next` 9). Bash sandbox (bubblewrap) can't start here, so commands run unsandboxed.
- egui/eframe are pinned to **0.36** because `egui-phosphor` 0.14 depends on egui 0.36. Bump them together.
- eframe 0.36 uses `App::ui(&mut self, ui, frame)` (not `update`).

## Architecture (src/)
- `player/` — engine. `Player` (UI-thread API) + `demux.rs` thread + `video_decode.rs` / `audio_decode.rs` threads, linked by bounded crossbeam channels.
  Seeks bump a **serial**; stale packets/frames/audio chunks are dropped at every stage. Serials are unique across players (`NEXT_SERIAL_BASE`).
- Audio: `audio_out.rs` — one `AudioDevice` (cpal stream) per app, shared by successive players (re-opening a device costs ~200 ms of start-up silence).
  Decoder pushes pts-tagged f32 chunks into a mutex queue; the device callback publishes an `AudioClockSample`.
  `Player::update` disciplines the system `Clock` to `audible_time()` (audio = master); without fresh audio the clock free-runs.
  `hold` keeps audio silent until the first video frame after open/seek is shown.
  Resampling uses swresample via raw FFI (`Resampler`), not ffmpeg-next's wrapper (its output buffer is too small when up-sampling).
- Speed: `Player::set_speed` scales the system clock and sets `AudioQueue::speed`; the audio decoder runs
  `player/tempo.rs` (libavfilter `atempo`, chained for < 0.5×) only while speed ≠ 1. Each `AudioChunk` carries its
  speed, so one device sample = `speed / rate` media seconds. `atempo` output pts are in the *output* domain —
  media pts are derived from the emitted sample count instead.
- Video frames: `player/frame.rs` — decoder output is kept as YUV planes (owned `ffmpeg::frame::Video`, no copy) tagged with
  layout (planar / semi-planar / RGBA fallback), bit depth, matrix (BT.601/709/2020; untagged → HD 709, SD 601) and range.
  Formats the shader can't take go through swscale (`FrameConverter`). 16-bit planes need `TEXTURE_FORMAT_16BIT_NORM`
  (requested in `main.rs::wgpu_options`, flagged in `GPU_16BIT`), else converted to 8-bit.
- `render/` — `VideoRenderer` (in egui-wgpu `CallbackResources`) uploads planes and draws via `VideoPaint` paint callback;
  `video.wgsl` does Y'CbCr→RGB and the rounded-corner SDF mask (premultiplied alpha). Colour math mirrored in `color_params` + tests.
  WGSL gotcha: `layout` is a reserved word.
- `app.rs` — `eframe::App`: input, playlist/auto-advance, resume, debounced config save, layout of chrome over the video.
- `playlist.rs` — pure model (next/prev, repeat Off/All/One, shuffle order, natural sort, folder expansion). Unit-tested.
- `config.rs` — `Config` (`~/.config/rvp/config.toml`, `#[serde(default)]` so hand-edited/partial files load) and
  `ResumeStore` (`~/.local/state/rvp/resume.toml`, keyed by canonical path; skips first 10 s and the last min(15 s, 5%)).
  Writes are atomic (tmp + rename).
- `ui/` — `window.rs` (borderless window: title bar, resize grips, aspect fit), `controls.rs` (floating bar), `seekbar.rs`,
  `volume.rs`, `playlist_panel.rs` (floating right panel, drag-reorder via egui dnd payloads), `widgets.rs`.
- `theme.rs` — design tokens (colors/sizes/motion) + egui style. Dark theme only. Use tokens, never ad-hoc colors.
  Translucent colors must use `from_rgba_unmultiplied_const` (premultiplied with rgb > a renders as additive white).
- `fonts.rs` — CJK font via `fc-match`, Phosphor icon font (`ui::widgets::icon_font`).

## Conventions
- UI strings are Japanese.
- Window is borderless + transparent; the rounded window body is painted by us (`clear_color` is transparent). Corner radius is 0 when maximized/fullscreen.
- Verify UI changes with a screenshot, not just a successful build.
