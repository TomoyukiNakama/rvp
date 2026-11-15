# rvp UI strings — Italiano
# Formato e regole: vedere l'inizio di en.ftl.

language-name = Italiano

## Title bar

window-exit-fullscreen = Esci da schermo intero (Esc)
window-close = Chiudi
window-restore = Ripristina
window-maximize = Ingrandisci
window-minimize = Riduci a icona
window-about = Informazioni su rvp / licenze (F1)
window-open-folder = Apri cartella (Shift+O)
window-open-file = Apri file (O)
window-language = Lingua (Language)

## Control bar

controls-pause = Pausa (Space)
controls-play = Riproduci (Space)
controls-replay = Riproduci dall'inizio (Space)
controls-prev = Precedente (B)
controls-next = Successivo (N)
controls-exit-fullscreen = Esci da schermo intero (F)
controls-fullscreen = Schermo intero (F)
controls-playlist = Playlist (P)
speed-tooltip = Velocità di riproduzione ([ / ], Backspace per ripristinare)
volume-mute = Disattiva audio (M)
volume-unmute = Riattiva audio (M)
volume-level = Volume { $percent }%
repeat-off = Ripetizione: no
repeat-one = Ripetizione: questo elemento
repeat-all = Ripetizione: intera playlist
shuffle-on = Casuale: sì
shuffle-off = Casuale: no

## Subtitle / audio menu

tracks-tooltip = Sottotitoli / audio (V / S / A)
tracks-subtitles = Sottotitoli
tracks-off = Disattivati
tracks-unsupported = { $label } (non supportato)
tracks-unsupported-why = I sottotitoli basati su immagini non sono supportati
tracks-load-subtitle = Carica file di sottotitoli…
tracks-timing = Sincronizzazione
tracks-delay-later = Ritarda (X)
tracks-delay-earlier = Anticipa (Z)
tracks-delay-reset = Clic per ripristinare
tracks-hide-subtitles = Nascondi sottotitoli (V)
tracks-show-subtitles = Mostra sottotitoli (V)
tracks-audio = Audio
tracks-no-audio-output = Nessuna uscita audio
# $sign is "+" or "−", $value e.g. "0.5"
delay-seconds = { $sign }{ $value } s

## Track languages (from the file's metadata)

lang-ja = Giapponese
lang-en = Inglese
lang-zh = Cinese
lang-ko = Coreano
lang-fr = Francese
lang-de = Tedesco
lang-es = Spagnolo
lang-it = Italiano
lang-ru = Russo
lang-pt = Portoghese

## Playlist panel

playlist-title = Playlist
playlist-close = Chiudi (P)
playlist-save = Salva playlist (Ctrl+S)
playlist-add = Aggiungi file
playlist-empty-hint = Trascina qui i file per aggiungerli

## Empty window / drag and drop

empty-drop = Trascina un video o un audio per riprodurlo
empty-open-file = Apri file
empty-open-folder = Apri cartella
empty-hint = Premi O per aprire un file, Shift+O per aprire una cartella
drop-append = Rilascia per aggiungere alla playlist
drop-play = Rilascia per riprodurre

## File dialogs

dialog-open-media = Apri file
dialog-open-folder = Apri cartella
dialog-add-to-playlist = Aggiungi alla playlist
dialog-save-playlist = Salva playlist
dialog-load-subtitle = Carica file di sottotitoli
dialog-filter-media = File multimediali e playlist
dialog-filter-playlist = Playlist
dialog-filter-subtitles = Sottotitoli
dialog-filter-all = Tutti i file

## On-screen messages

osd-play = Riproduci
osd-pause = Pausa
# $sign is "+" or "−", $value is a number of seconds
osd-seek = { $sign }{ $value } s
osd-muted = Audio disattivato
osd-speed = Velocità { $speed }
osd-language = Lingua: { $name }
osd-added =
    { $count ->
        [one] Aggiunto 1 elemento alla playlist
       *[other] Aggiunti { $count } elementi alla playlist
    }
osd-no-playable = Nessun file riproducibile trovato
osd-resumed = Ripresa dall'ultima posizione ({ $time })
osd-playlist-end = Fine della playlist
osd-playlist-empty = La playlist è vuota
osd-playlist-saved = Playlist salvata: { $name }
osd-playlist-save-failed = Impossibile salvare la playlist: { $error }
osd-open-video-first = Apri prima un file
osd-subtitle-loaded =
    { $count ->
        [one] Sottotitoli: { $name } (1 riga)
       *[other] Sottotitoli: { $name } ({ $count } righe)
    }
osd-audio-track = Audio: { $label }
osd-subtitle-track = Sottotitoli: { $label }
osd-subtitles-shown = Sottotitoli: visibili
osd-subtitles-hidden = Sottotitoli: nascosti
osd-subtitle-delay = Sincronizzazione sottotitoli: { $delay }
osd-hwdec-auto = Decodifica hardware: automatica
osd-hwdec-off = Decodifica hardware: disattivata
osd-single-audio-track = C'è una sola traccia audio
osd-media-info =
    Video: { $codec } { $width }×{ $height } · Decoder: { $decoder }
    Audio: { $audio }
osd-media-info-audio = Audio: { $audio }
media-info-software = software
media-info-none = nessuno

## Errors

error-audio-device = Impossibile aprire l'uscita audio: { $error }
error-no-audio-device = Nessun dispositivo di uscita audio
error-audio-config = Impossibile ottenere la configurazione dell'uscita audio
error-sample-format = Formato dei campioni non supportato: { $format }
error-audio-start = Impossibile avviare l'uscita audio
error-open-file = Impossibile aprire: { $path }
error-no-playable-stream = Nessun flusso video o audio riproducibile trovato
error-video-decoder = Impossibile aprire il decoder video
error-audio-decoder = Impossibile aprire il decoder audio
error-decoder-open = Impossibile aprire il decoder: { $error }
error-audio-playback = Impossibile riprodurre l'audio: { $error }
error-audio-track-open = Impossibile aprire la traccia audio: { $error }
error-read = Errore di lettura: { $error }
error-subtitle-load = Impossibile caricare i sottotitoli: { $path }
error-subtitle-empty = Nessun sottotitolo trovato: { $path }
error-hw-fallback = Decodifica hardware non riuscita; passato alla decodifica software

## About dialog

about-version = Versione { $version }
about-close = Chiudi (Esc)
about-tab-license = Licenza
about-tab-software = Software di terze parti
about-license-line = { $copyright }   Licenza: { $spdx }
about-notice =
    rvp è software libero: è lecito redistribuirlo e/o modificarlo secondo i termini della GNU General Public License (GPL) come pubblicata dalla Free Software Foundation, nella versione 3 della Licenza o (a propria scelta) in qualsiasi versione successiva.
    rvp è distribuito nella speranza che sia utile, ma SENZA ALCUNA GARANZIA; senza neppure la garanzia implicita di COMMERCIABILITÀ o IDONEITÀ PER UN PARTICOLARE SCOPO. Per maggiori dettagli si veda il testo completo della GPL qui sotto.
about-filter-hint = Filtra per nome o licenza
about-count = { $shown } / { $total }
about-click-hint = Fai clic su una riga per mostrare il testo completo della licenza.
about-col-name = Nome
about-col-version = Versione
about-col-license = Licenza
about-col-kind = Tipo
about-back = Torna all'elenco
about-build-config = Configurazione di compilazione
about-no-text = Nessun testo della licenza (non incluso in rvp).
about-kind-library = Libreria
about-kind-font = Font
about-kind-crate = Crate
about-version-system = Di sistema
about-version-bundled = Incluso
about-mit-standard = MIT (testo standard)
about-note-ffmpeg =
    Decodifica video/audio, ricampionamento e cambio di velocità. Collegato dinamicamente alle librerie condivise di sistema (libavcodec ecc.).
    Licenza (come dichiarata dalla libreria): { $license }
about-note-alsa = Uscita audio (tramite cpal). Collegato dinamicamente alla libreria condivisa di sistema.
about-note-vulkan = Rendering (tramite wgpu). Caricato in fase di esecuzione.
about-note-wayland = Finestre e trascinamento su Wayland. Usa la libreria condivisa di sistema.
about-note-xkb = Input da tastiera (tramite winit). Caricato in fase di esecuzione.
about-note-phosphor = Font di icone, incluso nel crate egui-phosphor.
about-font-name = Font dell'interfaccia (di sistema)
about-font-license = Dipende dal font
about-font-note =
    Vengono caricati i font di sistema scelti con fontconfig (fc-match); non sono inclusi in rvp.
    In uso: { $paths }
    Ogni font ha la propria licenza (ad es. Noto Sans CJK è OFL-1.1).
about-font-missing = fontconfig non ha trovato alcun font per l'interfaccia.
