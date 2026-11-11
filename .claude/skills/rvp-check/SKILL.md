---
name: rvp-check
description: Build, test and visually verify the rvp video player — generate test media, launch the app on a clip, capture an in-app screenshot and inspect it. Use after changing player or UI code, or when asked to run / screenshot the app.
---

# Verify rvp

1. Build and test:
   ```bash
   cargo build 2>&1 | grep -E "^(warning|error)" -A8
   cargo test 2>&1 | tail -15
   ```
2. Test media (once): `scripts/gen-test-media.sh` → `target/test-media/`
   (h264_1080p30.mp4 has a burnt-in timestamp at top-left: use it to check A/V timing/seeks.)
3. Screenshot the running app (GNOME/Wayland: `grim` does not work, use the built-in hook):
   ```bash
   RVP_SCREENSHOT="$TMPDIR/rvp.png" RVP_SCREENSHOT_AFTER=2.5 RVP_SCREENSHOT_EXIT=1 \
     timeout 30 ./target/debug/rvp target/test-media/h264_1080p30.mp4
   ```
   Then Read the PNG and check: rounded window corners (transparent), video aspect/letterboxing,
   control bar legibility, Japanese text renders (no tofu), burnt-in timestamp ≈ AFTER seconds.
   Without a file argument the empty state is shown.
4. Report what was checked and anything that looked wrong. Interactive behaviour (drag, resize,
   keyboard) can't be screenshotted — say it was not verified rather than claiming it works.
