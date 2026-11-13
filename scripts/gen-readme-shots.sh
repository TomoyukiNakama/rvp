#!/usr/bin/env bash
# Regenerates the README screenshots (docs/images/<lang>-*.webp) in English and Japanese.
# Demo clips are synthetic (ffmpeg lavfi sources, silent audio) and go to target/readme-media.
# The app window opens on screen for a few seconds per shot; keep the pointer away from it.
# Usage: scripts/gen-readme-shots.sh   (after `cargo build`)
set -euo pipefail
cd "$(dirname "$0")/.."

media=target/readme-media
out=docs/images
bin=target/debug/rvp
mkdir -p "$media" "$out"
ff() { ffmpeg -hide_banner -loglevel error -y "$@"; }
bt709=(-colorspace bt709 -color_primaries bt709 -color_trc bt709 -color_range tv)
silent=(-f lavfi -i anullsrc=r=48000:cl=stereo)

# Demo clips for the playlist (folder per language so the hero clip can default to that language's subtitles).
clip() { # name source duration
  ff -f lavfi -i "$2" "${silent[@]}" -t "$3" -vf "format=yuv420p" "${bt709[@]}" \
     -c:v libx264 -preset veryfast -crf 20 -c:a aac -shortest "$media/$1"
}
if [[ ! -f "$media/clips.done" ]]; then
  clip "Mandelbrot.mp4" "mandelbrot=size=1920x1080:rate=30" 20
  clip "Cellular Automaton.mp4" "cellauto=size=1920x1080:rate=30:rule=110:random_fill_ratio=0.5,scale=1920:1080:flags=neighbor,negate" 20
  clip "Game of Life.mkv" "life=size=480x270:rate=30:mold=10:life_color=#9aa7ff:death_color=#1a1d2e:mold_color=#4b55a0,scale=1920:1080:flags=neighbor" 20
  clip "Test Pattern.mp4" "testsrc2=size=1920x1080:rate=30" 20
  touch "$media/clips.done"
fi

# Hero clip: soft gradients, two audio tracks, English + Japanese subtitles.
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
printf '1\n00:00:00,300 --> 00:01:00,000\nrvp — a video player built with Rust and egui\n' > "$tmp/en.srt"
printf '1\n00:00:00,300 --> 00:01:00,000\nrvp — Rust と egui で作った動画プレイヤー\n' > "$tmp/ja.srt"
for lang in en ja; do
  mkdir -p "$media/$lang"
  for f in "$media"/*.mp4 "$media"/*.mkv; do ln -sf "../$(basename "$f")" "$media/$lang/"; done
  [[ -f "$media/$lang/Aurora.mkv" ]] && continue
  if [[ $lang == en ]]; then def=(-disposition:s:0 default -disposition:s:1 0); else def=(-disposition:s:0 0 -disposition:s:1 default); fi
  ff -f lavfi -i "gradients=size=1920x1080:rate=30:n=4:speed=0.015:c0=0x1b2a6b:c1=0x7b3fa0:c2=0x0f7c8c:c3=0xe0607e,format=yuv420p" \
     "${silent[@]}" "${silent[@]}" -i "$tmp/en.srt" -i "$tmp/ja.srt" -t 60 \
     -map 0:v -map 1:a -map 2:a -map 3 -map 4 \
     "${bt709[@]}" -c:v libx264 -preset veryfast -crf 18 -c:a aac -c:s srt \
     -metadata:s:a:0 language=jpn -metadata:s:a:0 title="Main" \
     -metadata:s:a:1 language=eng -metadata:s:a:1 title="Commentary" \
     -metadata:s:s:0 language=eng -metadata:s:s:1 language=jpn "${def[@]}" \
     "$media/$lang/Aurora.mkv"
done

# Isolated config/state/data: no resume positions, saved language or egui UI state.
export XDG_CONFIG_HOME="$tmp/config" XDG_STATE_HOME="$tmp/state" XDG_DATA_HOME="$tmp/data"  # data: egui memory (scroll offsets)

shot() { # lang name [env...] -- args...
  local lang=$1 name=$2; shift 2
  local envs=()
  while [[ $# -gt 0 && $1 != -- ]]; do envs+=("$1"); shift; done
  shift
  rm -rf "$XDG_STATE_HOME" "$XDG_DATA_HOME"
  local png="$tmp/$lang-$name.png" webp="$out/$lang-$name.webp"
  rm -f "$png"
  # Exit status ignored: rvp sometimes crashes while exiting (known issue); the PNG is written before that.
  env RVP_DEBUG_LANG="$lang" RVP_SCREENSHOT="$png" RVP_SCREENSHOT_AFTER=1.6 RVP_SCREENSHOT_EXIT=1 \
    "${envs[@]}" "$bin" "$@" >/dev/null 2>&1 || true
  [[ -f "$png" ]] || { echo "failed: $lang-$name" >&2; exit 1; }
  # WebP with alpha (rounded window corners) is ~10x smaller than the PNG.
  ff -i "$png" -c:v libwebp -pix_fmt yuva420p -quality 90 "$webp"
  echo "$webp"
}

for lang in en ja; do
  dir="$media/$lang"
  shot "$lang" player -- "$dir/Aurora.mkv"
  shot "$lang" playlist RVP_DEBUG_PLAYLIST=1 -- "$dir"
  shot "$lang" tracks RVP_DEBUG_POPUP=tracks -- "$dir/Aurora.mkv"
  shot "$lang" speed RVP_DEBUG_POPUP=speed RVP_DEBUG_SPEED=1.5 -- "$dir/Aurora.mkv"
  shot "$lang" language RVP_DEBUG_POPUP=language -- "$dir/Game of Life.mkv"
  shot "$lang" about RVP_DEBUG_ABOUT=software -- "$dir/Aurora.mkv"
  shot "$lang" empty --
done
