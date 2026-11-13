# rvp UI strings — Deutsch
# Format und Regeln: siehe Kopf von en.ftl.

language-name = Deutsch

## Title bar

window-exit-fullscreen = Vollbild beenden (Esc)
window-close = Schließen
window-restore = Wiederherstellen
window-maximize = Maximieren
window-minimize = Minimieren
window-about = Über rvp / Lizenzen (F1)
window-open-folder = Ordner öffnen (Shift+O)
window-open-file = Datei öffnen (O)
window-language = Sprache (Language)

## Control bar

controls-pause = Pause (Space)
controls-play = Wiedergabe (Space)
controls-replay = Von vorn abspielen (Space)
controls-prev = Zurück (B)
controls-next = Weiter (N)
controls-exit-fullscreen = Vollbild beenden (F)
controls-fullscreen = Vollbild (F)
controls-playlist = Wiedergabeliste (P)
speed-tooltip = Wiedergabegeschwindigkeit ([ / ], Backspace zum Zurücksetzen)
volume-mute = Stumm (M)
volume-unmute = Ton an (M)
volume-level = Lautstärke { $percent }%
repeat-off = Wiederholen: aus
repeat-one = Wiederholen: dieses Video
repeat-all = Wiederholen: ganze Liste
shuffle-on = Zufall: an
shuffle-off = Zufall: aus

## Subtitle / audio menu

tracks-tooltip = Untertitel / Audio (V / S / A)
tracks-subtitles = Untertitel
tracks-off = Aus
tracks-unsupported = { $label } (nicht unterstützt)
tracks-unsupported-why = Bildbasierte Untertitel werden nicht unterstützt
tracks-load-subtitle = Untertiteldatei laden…
tracks-timing = Timing
tracks-delay-later = Verzögern (X)
tracks-delay-earlier = Vorziehen (Z)
tracks-delay-reset = Klicken zum Zurücksetzen
tracks-hide-subtitles = Untertitel ausblenden (V)
tracks-show-subtitles = Untertitel einblenden (V)
tracks-audio = Audio
tracks-no-audio-output = Keine Audioausgabe
# $sign is "+" or "−", $value e.g. "0.5"
delay-seconds = { $sign }{ $value } s

## Track languages (from the file's metadata)

lang-ja = Japanisch
lang-en = Englisch
lang-zh = Chinesisch
lang-ko = Koreanisch
lang-fr = Französisch
lang-de = Deutsch
lang-es = Spanisch
lang-it = Italienisch
lang-ru = Russisch
lang-pt = Portugiesisch

## Playlist panel

playlist-title = Wiedergabeliste
playlist-close = Schließen (P)
playlist-save = Liste speichern (Ctrl+S)
playlist-add = Dateien hinzufügen
playlist-empty-hint = Dateien hierher ziehen, um sie hinzuzufügen

## Empty window / drag and drop

empty-drop = Video hierher ziehen zum Abspielen
empty-open-file = Datei öffnen
empty-open-folder = Ordner öffnen
empty-hint = O öffnet eine Datei, Shift+O einen Ordner
drop-append = Loslassen, um zur Liste hinzuzufügen
drop-play = Loslassen zum Abspielen

## File dialogs

dialog-open-video = Video öffnen
dialog-open-folder = Ordner öffnen
dialog-add-to-playlist = Zur Wiedergabeliste hinzufügen
dialog-save-playlist = Wiedergabeliste speichern
dialog-load-subtitle = Untertiteldatei laden
dialog-filter-media = Videos und Wiedergabelisten
dialog-filter-playlist = Wiedergabelisten
dialog-filter-subtitles = Untertitel
dialog-filter-all = Alle Dateien

## On-screen messages

osd-play = Wiedergabe
osd-pause = Pause
# $sign is "+" or "−", $value is a number of seconds
osd-seek = { $sign }{ $value } s
osd-muted = Stumm
osd-speed = Tempo { $speed }
osd-language = Sprache: { $name }
osd-added =
    { $count ->
        [one] 1 Eintrag zur Liste hinzugefügt
       *[other] { $count } Einträge zur Liste hinzugefügt
    }
osd-no-playable = Keine abspielbaren Videos gefunden
osd-resumed = Fortgesetzt an letzter Position ({ $time })
osd-playlist-end = Ende der Wiedergabeliste
osd-playlist-empty = Die Wiedergabeliste ist leer
osd-playlist-saved = Liste gespeichert: { $name }
osd-playlist-save-failed = Liste konnte nicht gespeichert werden: { $error }
osd-open-video-first = Zuerst ein Video öffnen
osd-subtitle-loaded =
    { $count ->
        [one] Untertitel: { $name } (1 Eintrag)
       *[other] Untertitel: { $name } ({ $count } Einträge)
    }
osd-audio-track = Audio: { $label }
osd-subtitle-track = Untertitel: { $label }
osd-subtitles-shown = Untertitel: an
osd-subtitles-hidden = Untertitel: aus
osd-subtitle-delay = Untertitel-Timing: { $delay }
osd-hwdec-auto = Hardware-Dekodierung: automatisch
osd-hwdec-off = Hardware-Dekodierung: aus
osd-single-audio-track = Es gibt nur eine Tonspur
osd-media-info =
    Video: { $codec } { $width }×{ $height } · Decoder: { $decoder }
    Audio: { $audio }
media-info-software = Software
media-info-none = keine

## Errors

error-audio-device = Audioausgabe kann nicht geöffnet werden: { $error }
error-no-audio-device = Kein Audioausgabegerät
error-audio-config = Audio-Konfiguration nicht abrufbar
error-sample-format = Nicht unterstütztes Sample-Format: { $format }
error-audio-start = Audioausgabe kann nicht gestartet werden
error-open-file = Öffnen fehlgeschlagen: { $path }
error-no-video-stream = Kein Videostream gefunden
error-video-decoder = Video-Decoder konnte nicht geöffnet werden
error-audio-decoder = Audio-Decoder konnte nicht geöffnet werden
error-decoder-open = Decoder kann nicht geöffnet werden: { $error }
error-audio-playback = Audio kann nicht wiedergegeben werden: { $error }
error-audio-track-open = Tonspur kann nicht geöffnet werden: { $error }
error-read = Lesefehler: { $error }
error-subtitle-load = Untertitel konnten nicht geladen werden: { $path }
error-subtitle-empty = Keine Untertitel gefunden: { $path }
error-hw-fallback = Hardware-Dekodierung fehlgeschlagen; auf Software-Dekodierung umgeschaltet

## About dialog

about-version = Version { $version }
about-close = Schließen (Esc)
about-tab-license = Lizenz
about-tab-software = Fremdsoftware
about-license-line = { $copyright }   Lizenz: { $spdx }
about-notice =
    rvp ist freie Software: Sie können es unter den Bedingungen der GNU General Public License (GPL), wie von der Free Software Foundation veröffentlicht, weitergeben und/oder modifizieren, entweder gemäß Version 3 der Lizenz oder (nach Ihrer Option) jeder späteren Version.
    rvp wird in der Hoffnung veröffentlicht, dass es nützlich sein wird, aber OHNE JEDE GEWÄHRLEISTUNG; sogar ohne die implizite Gewährleistung der MARKTFÄHIGKEIT oder EIGNUNG FÜR EINEN BESTIMMTEN ZWECK. Details finden Sie im vollständigen GPL-Text unten.
about-filter-hint = Nach Name oder Lizenz filtern
about-count = { $shown } / { $total }
about-click-hint = Zeile anklicken, um den vollständigen Lizenztext anzuzeigen.
about-col-name = Name
about-col-version = Version
about-col-license = Lizenz
about-col-kind = Typ
about-back = Zurück zur Liste
about-build-config = Build-Konfiguration
about-no-text = Kein Lizenztext (nicht in rvp enthalten).
about-kind-library = Bibliothek
about-kind-font = Schrift
about-kind-crate = Crate
about-version-system = System
about-version-bundled = Mitgeliefert
about-mit-standard = MIT (Standardtext)
about-note-ffmpeg =
    Video-/Audio-Dekodierung, Resampling und Geschwindigkeitsänderung. Dynamisch an die System-Bibliotheken (libavcodec usw.) gebunden.
    Lizenz (laut Bibliothek): { $license }
about-note-alsa = Audioausgabe (über cpal). Dynamisch an die System-Bibliothek gebunden.
about-note-vulkan = Darstellung (über wgpu). Wird zur Laufzeit geladen.
about-note-wayland = Fensterverwaltung und Drag-and-drop unter Wayland. Nutzt die System-Bibliothek.
about-note-xkb = Tastatureingabe (über winit). Wird zur Laufzeit geladen.
about-note-phosphor = Symbolschrift, im Crate egui-phosphor enthalten.
about-font-name = UI-Schriften (System)
about-font-license = Je nach Schrift
about-font-note =
    Per fontconfig (fc-match) gewählte Systemschriften werden geladen; sie sind nicht in rvp enthalten.
    In Verwendung: { $paths }
    Jede Schrift hat ihre eigene Lizenz (z. B. Noto Sans CJK: OFL-1.1).
about-font-missing = fontconfig hat keine Schrift für die Oberfläche gefunden.
