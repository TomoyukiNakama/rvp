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

# Two audio tracks + embedded subtitles (SRT/jpn default, ASS/eng) + an external SRT.
tmp="$(mktemp -d)"
printf '1\n00:00:01,000 --> 00:00:04,000\nこんにちは、字幕のテストです\n\n2\n00:00:05,000 --> 00:00:08,000\n<i>二行目も</i>\n表示できます\n' > "$tmp/ja.srt"
printf '[Script Info]\nScriptType: v4.00+\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:01.00,0:00:04.00,Default,,0,0,0,,{\\i1}English{\\i0} subtitle\\Nsecond line\n' > "$tmp/en.ass"
ff -f lavfi -i testsrc2=size=1280x720:rate=30:duration=12 \
   -f lavfi -i sine=frequency=440:duration=12 -f lavfi -i sine=frequency=660:duration=12 \
   -i "$tmp/ja.srt" -i "$tmp/en.ass" \
   -map 0:v -map 1:a -map 2:a -map 3 -map 4 \
   "${bt709[@]}" -c:v libx264 -c:a aac -c:s:0 srt -c:s:1 ass \
   -metadata:s:a:0 language=jpn -metadata:s:a:1 language=eng \
   -metadata:s:s:0 language=jpn -metadata:s:s:1 language=eng \
   -disposition:s:0 default -disposition:s:1 0 "$out/tracks.mkv"
mkdir -p "$out/external"
cp "$out/h264_720p60.mp4" "$out/external/clip.mp4"
printf '1\n00:00:00,500 --> 00:00:05,000\n外部 SRT ファイル（Shift_JIS）\n' | iconv -f UTF-8 -t SHIFT_JIS > "$out/external/clip.srt"
rm -rf "$tmp"

ls -lh "$out"
