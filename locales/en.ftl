# rvp UI strings — English (reference / fallback locale).
#
# Format: Project Fluent (https://projectfluent.org/). Every other locale file
# must define the same message ids and use the same $variables
# (checked by `cargo test`). Missing messages fall back to this file.
#
# Users can override or add translations without rebuilding: put
# `<tag>.ftl` into ~/.config/rvp/locales/ (reloaded while rvp is running).

# Native name of this language, shown in the language menu.
language-name = English

## Title bar

window-exit-fullscreen = Exit full screen (Esc)
window-close = Close
window-restore = Restore
window-maximize = Maximize
window-minimize = Minimize
window-about = About rvp / licenses (F1)
window-open-folder = Open folder (Shift+O)
window-open-file = Open file (O)
window-language = Language

## Control bar

controls-pause = Pause (Space)
controls-play = Play (Space)
controls-replay = Play from the start (Space)
controls-prev = Previous (B)
controls-next = Next (N)
controls-exit-fullscreen = Exit full screen (F)
controls-fullscreen = Full screen (F)
controls-playlist = Playlist (P)
speed-tooltip = Playback speed ([ / ], Backspace to reset)
volume-mute = Mute (M)
volume-unmute = Unmute (M)
volume-level = Volume { $percent }%
repeat-off = Loop: off
repeat-one = Loop: this video
repeat-all = Loop: whole playlist
shuffle-on = Shuffle: on
shuffle-off = Shuffle: off

## Subtitle / audio menu

tracks-tooltip = Subtitles / audio (V / S / A)
tracks-subtitles = Subtitles
tracks-off = Off
tracks-unsupported = { $label } (unsupported)
tracks-unsupported-why = Image-based subtitles are not supported
tracks-load-subtitle = Load subtitle file…
tracks-timing = Timing
tracks-delay-later = Delay (X)
tracks-delay-earlier = Advance (Z)
tracks-delay-reset = Click to reset
tracks-hide-subtitles = Hide subtitles (V)
tracks-show-subtitles = Show subtitles (V)
tracks-audio = Audio
tracks-no-audio-output = No audio output
# $sign is "+" or "−", $value e.g. "0.5"
delay-seconds = { $sign }{ $value } s

## Track languages (from the file's metadata)

lang-ja = Japanese
lang-en = English
lang-zh = Chinese
lang-ko = Korean
lang-fr = French
lang-de = German
lang-es = Spanish
lang-it = Italian
lang-ru = Russian
lang-pt = Portuguese

## Playlist panel

playlist-title = Playlist
playlist-close = Close (P)
playlist-save = Save playlist (Ctrl+S)
playlist-add = Add files
playlist-empty-hint = Drop files here to add them

## Empty window / drag and drop

empty-drop = Drop a video to play it
empty-open-file = Open file
empty-open-folder = Open folder
empty-hint = Press O to open a file, Shift+O to open a folder
drop-append = Drop to add to the playlist
drop-play = Drop to play

## File dialogs

dialog-open-video = Open video
dialog-open-folder = Open folder
dialog-add-to-playlist = Add to playlist
dialog-save-playlist = Save playlist
dialog-load-subtitle = Load subtitle file
dialog-filter-media = Videos and playlists
dialog-filter-playlist = Playlists
dialog-filter-subtitles = Subtitles
dialog-filter-all = All files

## On-screen messages

osd-play = Play
osd-pause = Pause
# $sign is "+" or "−", $value is a number of seconds
osd-seek = { $sign }{ $value } s
osd-muted = Muted
osd-speed = Speed { $speed }
osd-language = Language: { $name }
osd-added =
    { $count ->
        [one] Added 1 item to the playlist
       *[other] Added { $count } items to the playlist
    }
osd-no-playable = No playable videos found
osd-resumed = Resumed from the last position ({ $time })
osd-playlist-end = End of the playlist
osd-playlist-empty = The playlist is empty
osd-playlist-saved = Playlist saved: { $name }
osd-playlist-save-failed = Could not save the playlist: { $error }
osd-open-video-first = Open a video first
osd-subtitle-loaded =
    { $count ->
        [one] Subtitles: { $name } (1 cue)
       *[other] Subtitles: { $name } ({ $count } cues)
    }
osd-audio-track = Audio: { $label }
osd-subtitle-track = Subtitles: { $label }
osd-subtitles-shown = Subtitles: shown
osd-subtitles-hidden = Subtitles: hidden
osd-subtitle-delay = Subtitle timing: { $delay }
osd-hwdec-auto = Hardware decoding: auto
osd-hwdec-off = Hardware decoding: off
osd-single-audio-track = There is only one audio track
osd-media-info =
    Video: { $codec } { $width }×{ $height } · Decoder: { $decoder }
    Audio: { $audio }
media-info-software = software
media-info-none = none

## Errors

error-audio-device = Cannot open audio output: { $error }
error-no-audio-device = No audio output device
error-audio-config = Cannot get the audio output configuration
error-sample-format = Unsupported sample format: { $format }
error-audio-start = Cannot start audio output
error-open-file = Could not open: { $path }
error-no-video-stream = No video stream found
error-video-decoder = Could not open the video decoder
error-audio-decoder = Could not open the audio decoder
error-decoder-open = Cannot open the decoder: { $error }
error-audio-playback = Cannot play audio: { $error }
error-audio-track-open = Cannot open the audio track: { $error }
error-read = Read error: { $error }
error-subtitle-load = Cannot load subtitles: { $path }
error-subtitle-empty = No subtitles found: { $path }
error-hw-fallback = Hardware decoding failed; switched to software decoding

## About dialog

about-version = Version { $version }
about-close = Close (Esc)
about-tab-license = License
about-tab-software = Third-party software
about-license-line = { $copyright }   License: { $spdx }
about-notice =
    rvp is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License (GPL) as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
    rvp is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the full text of the GPL below for details.
about-filter-hint = Filter by name or license
about-count = { $shown } / { $total }
about-click-hint = Click a row to show the full license text.
about-col-name = Name
about-col-version = Version
about-col-license = License
about-col-kind = Type
about-back = Back to the list
about-build-config = Build configuration
about-no-text = No license text (not bundled with rvp).
about-kind-library = Library
about-kind-font = Font
about-kind-crate = Crate
about-version-system = System
about-version-bundled = Bundled
about-mit-standard = MIT (standard text)
about-note-ffmpeg =
    Video/audio decoding, resampling and speed change. Dynamically linked against the system shared libraries (libavcodec etc.).
    License (as reported by the library): { $license }
about-note-alsa = Audio output (via cpal). Dynamically linked against the system shared library.
about-note-vulkan = Rendering (via wgpu). Loaded at run time.
about-note-wayland = Windowing and drag and drop on Wayland. Uses the system shared library.
about-note-xkb = Keyboard input (via winit). Loaded at run time.
about-note-phosphor = Icon font, bundled with the egui-phosphor crate.
about-font-name = UI fonts (system)
about-font-license = Depends on the font
about-font-note =
    System fonts chosen with fontconfig (fc-match) are loaded; they are not bundled with rvp.
    In use: { $paths }
    Each font has its own license (e.g. Noto Sans CJK is OFL-1.1).
about-font-missing = fontconfig found no font for the UI.
