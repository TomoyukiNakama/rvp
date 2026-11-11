#!/usr/bin/env bash
# Generates small synthetic test videos for manual checks.
# Usage: scripts/gen-test-media.sh [out_dir]   (default: target/test-media)
set -euo pipefail

out="${1:-target/test-media}"
mkdir -p "$out"
ff() { ffmpeg -hide_banner -loglevel error -y "$@"; }
# Convert RGB sources with the BT.709 matrix *and* tag the stream, so HD clips
# aren't subject to the "untagged → guess" heuristic.
bt709=(-vf "scale=out_color_matrix=bt709:out_range=tv,format=yuv420p"
       -colorspace bt709 -color_primaries bt709 -color_trc bt709 -color_range tv)

# 1080p30 H.264 + 440 Hz tone, keyframe every 2 s, burnt-in timestamp (testsrc2).
ff -f lavfi -i testsrc2=size=1920x1080:rate=30:duration=20 \
   -f lavfi -i sine=frequency=440:duration=20 \
   "${bt709[@]}" -c:v libx264 -g 60 -c:a aac "$out/h264_1080p30.mp4"

# 4:3 HEVC Main10 (letterboxing + 10-bit path).
ff -f lavfi -i testsrc=size=720x540:rate=25:duration=10 \
   -c:v libx265 -x265-params log-level=error -pix_fmt yuv420p10le "$out/hevc10_4x3.mkv"

# Anamorphic (SAR 32:27 → 16:9 display) VP9 in WebM.
ff -f lavfi -i testsrc=size=720x480:rate=30:duration=10 \
   -vf setsar=32/27,format=yuv420p -c:v libvpx-vp9 -b:v 1M "$out/vp9_anamorphic.webm"

# 60 fps for smoothness / frame-drop checks.
ff -f lavfi -i testsrc2=size=1280x720:rate=60:duration=15 \
   "${bt709[@]}" -c:v libx264 "$out/h264_720p60.mp4"

# Colour accuracy: SMPTE HD bars (compare against the PNG reference).
ff -f lavfi -i smptehdbars=size=1920x1080:rate=30:duration=5 \
   "${bt709[@]}" -c:v libx264 -crf 1 "$out/bars_bt709.mp4"
ff -f lavfi -i smptehdbars=size=1920x1080 -frames:v 1 "$out/bars_reference.png"

ls -lh "$out"
