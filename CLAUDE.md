# rvp — Rust + egui video player

Spec and roadmap: `SPEC.md` (Japanese). Phases are listed in SPEC §6; keep SPEC in sync when decisions change.

## Commands
- Build / run: `cargo build`, `cargo run -- <video>` (dev profile is optimized enough for real-time playback)
- Test: `cargo test` (player integration test generates a clip via the `ffmpeg` CLI; skips if missing)
- Test media: `scripts/gen-test-media.sh` → `target/test-media/`
- README screenshots: `scripts/gen-readme-shots.sh` → `docs/images/{en,ja}-*.webp` (README.md = English, README.ja.md = Japanese; keep both in sync)
- After changing dependencies: `python3 scripts/gen-licenses.py` (regenerates `licenses/third-party.toml`; `--check` verifies)
  and `python3 flatpak/gen-cargo-sources.py` (vendored crate list for the offline Flatpak build; `--check` verifies)
- Flatpak: `flatpak-builder --user --install --force-clean flatpak/build-dir flatpak/io.github.TomoyukiNakama.rvp.yml`,
  then `flatpak run io.github.TomoyukiNakama.rvp`. Runtime 26.08 provides FFmpeg 8.1 (+ codecs-extra); app id comes from
  `$FLATPAK_ID`. Config lives in `~/.var/app/<id>/config/rvp/` (`--env=XDG_CONFIG_HOME` is overridden by flatpak).
- `cargo test` does **not** rebuild `target/debug/rvp`; run `cargo build` before launching the app.
- Keep manual runs out of the real config/state: `XDG_CONFIG_HOME=$TMPDIR/xdg/config XDG_STATE_HOME=$TMPDIR/xdg/state`.
  `RVP_DEBUG_PLAYLIST=1` starts with the playlist panel open, `RVP_DEBUG_SPEED=2` at 2× (for screenshots).
  `RVP_DEBUG_POPUP=speed|tracks|language` opens that menu once at startup; `RVP_DEBUG_LANG=<tag>` overrides the UI language (not saved).
  `RVP_DEBUG_ABOUT=license|software|<component name>` opens the about dialog (that tab / that component's details).
- Clicks can't be screenshotted: test interactive widgets headlessly with synthetic `RawInput` events through
  `Context::run_ui` (see `ui::playlist_panel::tests`; install the Phosphor font and clear `textures_delta`).
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
- Subtitles/tracks: `player/subtitle.rs` (cue store with dedupe, SRT/VTT/ASS parsers, ASS tag stripping, UTF-8/BOM/
  Shift_JIS detection, `find_external`). Embedded *text* subs are decoded inline in the demux thread (`decode_subtitle`;
  call `avsubtitle_free` — ffmpeg-next doesn't) and sent as `PlayerEvent::SubtitleCue`. Track switches go through
  `DemuxCmd::SelectAudio/SelectSubtitle` followed by an exact re-seek (`Player::reseek`); audio switch ships a new
  decoder to the audio thread via `PacketMsg::NewAudioDecoder`. Bitmap subs (PGS/DVD) are listed as unsupported.
- HW decode: `player/hwaccel.rs` — `HwDec` (config `hwdec = auto|off|cuda|vaapi`; auto = CUDA then VAAPI). Devices
  are cached per process; VAAPI probes every `/dev/dri/renderD*` (this machine: renderD128 = NVIDIA without VA driver,
  renderD129 = AMD iGPU with radeonsi). `get_format` falls back to software when the hw format isn't offered.
  Frames are downloaded (`av_hwframe_transfer_data`) to NV12/P010 → semi-planar path. Repeated hw errors make the
  decode thread swap in a SW decoder and emit `PlayerEvent::HwFallback` (player re-seeks). Vulkan hwaccel hangs here.
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
  M3U/M3U8: `expand` turns playlist files into their entries (HLS manifests with `#EXT-X-` stay media); `write_m3u`
  saves (panel button / Ctrl+S, entries under the playlist's folder are written relative).
- `config.rs` — `Config` (`~/.config/rvp/config.toml`, `#[serde(default)]` so hand-edited/partial files load) and
  `ResumeStore` (`~/.local/state/rvp/resume.toml`, keyed by canonical path; skips first 10 s and the last min(15 s, 5%)).
  Writes are atomic (tmp + rename).
- `ui/` — `window.rs` (borderless window: title bar, resize grips, aspect fit), `controls.rs` (floating bar), `seekbar.rs`,
  `volume.rs`, `speed.rs`, `tracks.rs` (subtitle/audio popup), `playlist_panel.rs` (floating right panel,
  drag-reorder via egui dnd payloads), `widgets.rs`. Subtitle overlay: `app.rs::paint_subtitle` (8-way outline).
- `theme.rs` — design tokens (colors/sizes/motion) + egui style. Dark theme only. Use tokens, never ad-hoc colors.
  Translucent colors must use `from_rgba_unmultiplied_const` (premultiplied with rgb > a renders as additive white).
- Licensing: rvp is GPL-3.0-or-later (`LICENSE`; required by the GPL FFmpeg build). `about.rs` = data (crates from
  the generated `licenses/third-party.toml`, hand-listed system libs, FFmpeg version/licence queried at run time);
  `ui/about.rs` = "rvp について" modal (title-bar ⓘ / F1). New deps must be GPL-3-compatible.
- `i18n.rs` — Project Fluent. Messages in `locales/<tag>.ftl` (embedded; `en.ftl` is the reference/fallback), looked up
  with `t!("id", var = value)` (macro is `#[macro_use]`, usable in every module and thread). `set_language` switches at
  run time; anything caching translated text keys on `i18n::generation()` (e.g. `about::components()`), otherwise
  translate at draw time (e.g. `TrackInfo::label()`). First run: `detect()` from the OS locale, saved as `config.language`.
  User `.ftl` files in `~/.config/rvp/locales/` override/add languages and are hot-reloaded (`poll_reload`).
  New UI strings: add the id to **every** `locales/*.ftl` (`cargo test` checks ids and variables match `en.ftl`).
  egui has no shaping/RTL, so no Thai/Indic/Arabic UI. Picker: `ui/language.rs` (title bar).
- `fonts.rs` — UI font per language via `fc-match sans-serif:lang=<lang>` (+ Japanese CJK fallback for non-CJK
  languages), re-installed on language switch; Phosphor icon font (`ui::widgets::icon_font`).

## Windowing gotchas (Wayland)
- winit 0.30 has no Wayland drag-and-drop: `wayland_dnd.rs` joins winit's `wl_display` (`Backend::from_foreign_display`)
  with its own queue/thread and `wl_data_device`, reads `text/uri-list` on drop. X11 drops still come via egui.
  Forcing X11 instead is not an option on NVIDIA: the X11 Vulkan surface has no transparent alpha mode (no rounded corners).
- `StartDrag`/`BeginResize` hand the pointer to the compositor, which swallows the button release; `raw_input_hook`
  injects a release next frame (`inject_release`), otherwise egui keeps the drag "active" and the next drag fails.
- Hover-revealed child buttons (e.g. playlist row ×): decide visibility by pointer position (`rect_contains_pointer`),
  not the parent's `hovered()` — the child becomes the hovered widget and would vanish before the click lands.
- Popup rows must use a fixed width (`Popup::width` + `set_width`); `available_width()` inside a popup is the parent's.
- Anything painted at the window edges must respect the rounded corners (e.g. `window::rounded_top_gradient`).
- Window follows the video aspect (`fit_window_to_video`): on open and after a manual resize settles (0.35 s), the
  height is snapped via `ViewportCommand::InnerSize` (capped at 90% of the monitor). Not while maximized/fullscreen.
- Auto-hide is purely idle-based (pointer motion / button / key / wheel events), playing or paused; kept visible
  while hovering the control bar or playlist panel, while a popup is open, or with no file loaded.

## Conventions
- UI strings go through `t!` / `locales/*.ftl` — never hard-code user-visible text. Docs and SPEC are Japanese.
- Window is borderless + transparent; the rounded window body is painted by us (`clear_color` is transparent). Corner radius is 0 when maximized/fullscreen.
- Verify UI changes with a screenshot, not just a successful build.
