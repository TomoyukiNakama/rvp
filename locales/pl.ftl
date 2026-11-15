# rvp UI strings — Polski
# Format i zasady: patrz początek en.ftl.

language-name = Polski

## Title bar

window-exit-fullscreen = Wyjdź z pełnego ekranu (Esc)
window-close = Zamknij
window-restore = Przywróć
window-maximize = Maksymalizuj
window-minimize = Minimalizuj
window-about = O programie rvp / licencje (F1)
window-open-folder = Otwórz folder (Shift+O)
window-open-file = Otwórz plik (O)
window-language = Język (Language)

## Control bar

controls-pause = Pauza (Space)
controls-play = Odtwarzaj (Space)
controls-replay = Odtwórz od początku (Space)
controls-prev = Poprzedni (B)
controls-next = Następny (N)
controls-exit-fullscreen = Wyjdź z pełnego ekranu (F)
controls-fullscreen = Pełny ekran (F)
controls-playlist = Lista odtwarzania (P)
speed-tooltip = Prędkość odtwarzania ([ / ], Backspace — reset)
volume-mute = Wycisz (M)
volume-unmute = Włącz dźwięk (M)
volume-level = Głośność { $percent }%
repeat-off = Powtarzanie: wyłączone
repeat-one = Powtarzanie: ten element
repeat-all = Powtarzanie: cała lista
shuffle-on = Losowo: włączone
shuffle-off = Losowo: wyłączone

## Subtitle / audio menu

tracks-tooltip = Napisy / dźwięk (V / S / A)
tracks-subtitles = Napisy
tracks-off = Wyłączone
tracks-unsupported = { $label } (nieobsługiwane)
tracks-unsupported-why = Napisy w postaci obrazów nie są obsługiwane
tracks-load-subtitle = Wczytaj plik napisów…
tracks-timing = Synchronizacja
tracks-delay-later = Opóźnij (X)
tracks-delay-earlier = Przyspiesz (Z)
tracks-delay-reset = Kliknij, aby zresetować
tracks-hide-subtitles = Ukryj napisy (V)
tracks-show-subtitles = Pokaż napisy (V)
tracks-audio = Dźwięk
tracks-no-audio-output = Brak wyjścia audio
# $sign is "+" or "−", $value e.g. "0.5"
delay-seconds = { $sign }{ $value } s

## Track languages (from the file's metadata)

lang-ja = Japoński
lang-en = Angielski
lang-zh = Chiński
lang-ko = Koreański
lang-fr = Francuski
lang-de = Niemiecki
lang-es = Hiszpański
lang-it = Włoski
lang-ru = Rosyjski
lang-pt = Portugalski

## Playlist panel

playlist-title = Lista odtwarzania
playlist-close = Zamknij (P)
playlist-save = Zapisz listę (Ctrl+S)
playlist-add = Dodaj pliki
playlist-empty-hint = Upuść pliki tutaj, aby je dodać

## Empty window / drag and drop

empty-drop = Upuść plik wideo lub audio, aby go odtworzyć
empty-open-file = Otwórz plik
empty-open-folder = Otwórz folder
empty-hint = Naciśnij O, aby otworzyć plik, lub Shift+O, aby otworzyć folder
drop-append = Upuść, aby dodać do listy
drop-play = Upuść, aby odtworzyć

## File dialogs

dialog-open-media = Otwórz plik
dialog-open-folder = Otwórz folder
dialog-add-to-playlist = Dodaj do listy odtwarzania
dialog-save-playlist = Zapisz listę odtwarzania
dialog-load-subtitle = Wczytaj plik napisów
dialog-filter-media = Pliki multimedialne i listy odtwarzania
dialog-filter-playlist = Listy odtwarzania
dialog-filter-subtitles = Napisy
dialog-filter-all = Wszystkie pliki

## On-screen messages

osd-play = Odtwarzanie
osd-pause = Pauza
# $sign is "+" or "−", $value is a number of seconds
osd-seek = { $sign }{ $value } s
osd-muted = Wyciszono
osd-speed = Prędkość { $speed }
osd-language = Język: { $name }
osd-added =
    { $count ->
        [one] Dodano 1 element do listy
        [few] Dodano { $count } elementy do listy
        [many] Dodano { $count } elementów do listy
       *[other] Dodano { $count } elementu do listy
    }
osd-no-playable = Nie znaleziono plików do odtworzenia
osd-resumed = Wznowiono od ostatniej pozycji ({ $time })
osd-playlist-end = Koniec listy odtwarzania
osd-playlist-empty = Lista odtwarzania jest pusta
osd-playlist-saved = Zapisano listę: { $name }
osd-playlist-save-failed = Nie można zapisać listy: { $error }
osd-open-video-first = Najpierw otwórz plik
osd-subtitle-loaded =
    { $count ->
        [one] Napisy: { $name } (1 wpis)
        [few] Napisy: { $name } ({ $count } wpisy)
        [many] Napisy: { $name } ({ $count } wpisów)
       *[other] Napisy: { $name } ({ $count } wpisu)
    }
osd-audio-track = Dźwięk: { $label }
osd-subtitle-track = Napisy: { $label }
osd-subtitles-shown = Napisy: widoczne
osd-subtitles-hidden = Napisy: ukryte
osd-subtitle-delay = Synchronizacja napisów: { $delay }
osd-hwdec-auto = Dekodowanie sprzętowe: automatyczne
osd-hwdec-off = Dekodowanie sprzętowe: wyłączone
osd-single-audio-track = Dostępna jest tylko jedna ścieżka dźwiękowa
osd-media-info =
    Wideo: { $codec } { $width }×{ $height } · Dekoder: { $decoder }
    Dźwięk: { $audio }
osd-media-info-audio = Dźwięk: { $audio }
media-info-software = programowy
media-info-none = brak

## Errors

error-audio-device = Nie można otworzyć wyjścia audio: { $error }
error-no-audio-device = Brak urządzenia wyjściowego audio
error-audio-config = Nie można pobrać konfiguracji wyjścia audio
error-sample-format = Nieobsługiwany format próbek: { $format }
error-audio-start = Nie można uruchomić wyjścia audio
error-open-file = Nie można otworzyć: { $path }
error-no-playable-stream = Nie znaleziono strumienia wideo ani audio do odtworzenia
error-video-decoder = Nie można otworzyć dekodera wideo
error-audio-decoder = Nie można otworzyć dekodera audio
error-decoder-open = Nie można otworzyć dekodera: { $error }
error-audio-playback = Nie można odtworzyć dźwięku: { $error }
error-audio-track-open = Nie można otworzyć ścieżki dźwiękowej: { $error }
error-read = Błąd odczytu: { $error }
error-subtitle-load = Nie można wczytać napisów: { $path }
error-subtitle-empty = Nie znaleziono napisów: { $path }
error-hw-fallback = Dekodowanie sprzętowe nie powiodło się; przełączono na programowe

## About dialog

about-version = Wersja { $version }
about-close = Zamknij (Esc)
about-tab-license = Licencja
about-tab-software = Oprogramowanie innych firm
about-license-line = { $copyright }   Licencja: { $spdx }
about-notice =
    rvp jest wolnym oprogramowaniem; możesz go rozpowszechniać dalej i/lub modyfikować na warunkach Powszechnej Licencji Publicznej GNU (GPL), wydanej przez Fundację Wolnego Oprogramowania (Free Software Foundation) — według wersji 3 tej Licencji lub (według twojego wyboru) którejś z późniejszych wersji.
    rvp rozpowszechniany jest z nadzieją, iż będzie on użyteczny — jednak BEZ JAKIEJKOLWIEK GWARANCJI, nawet domyślnej gwarancji PRZYDATNOŚCI HANDLOWEJ albo PRZYDATNOŚCI DO OKREŚLONYCH ZASTOSOWAŃ. Bliższe informacje znajdziesz w pełnym tekście GPL poniżej.
about-filter-hint = Filtruj według nazwy lub licencji
about-count = { $shown } / { $total }
about-click-hint = Kliknij wiersz, aby zobaczyć pełny tekst licencji.
about-col-name = Nazwa
about-col-version = Wersja
about-col-license = Licencja
about-col-kind = Typ
about-back = Wróć do listy
about-build-config = Konfiguracja kompilacji
about-no-text = Brak tekstu licencji (nie dołączono go do rvp).
about-kind-library = Biblioteka
about-kind-font = Czcionka
about-kind-crate = Crate
about-version-system = Systemowa
about-version-bundled = Dołączona
about-mit-standard = MIT (tekst standardowy)
about-note-ffmpeg =
    Dekodowanie wideo i dźwięku, resampling i zmiana prędkości. Łączona dynamicznie z systemowymi bibliotekami współdzielonymi (libavcodec itd.).
    Licencja (według biblioteki): { $license }
about-note-alsa = Wyjście audio (przez cpal). Łączona dynamicznie z systemową biblioteką współdzieloną.
about-note-vulkan = Renderowanie (przez wgpu). Ładowana w czasie działania.
about-note-wayland = Okna i przeciąganie plików w Waylandzie. Korzysta z systemowej biblioteki współdzielonej.
about-note-xkb = Obsługa klawiatury (przez winit). Ładowana w czasie działania.
about-note-phosphor = Czcionka ikon, dołączona do crate'a egui-phosphor.
about-font-name = Czcionki interfejsu (systemowe)
about-font-license = Zależy od czcionki
about-font-note =
    Wczytywane są czcionki systemowe wybrane przez fontconfig (fc-match); nie są dołączone do rvp.
    W użyciu: { $paths }
    Każda czcionka ma własną licencję (np. Noto Sans CJK ma OFL-1.1).
about-font-missing = fontconfig nie znalazł czcionki dla interfejsu.
