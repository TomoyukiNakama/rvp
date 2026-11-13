# rvp UI strings — Español
# Formato y reglas: véase la cabecera de en.ftl.

language-name = Español

## Title bar

window-exit-fullscreen = Salir de pantalla completa (Esc)
window-close = Cerrar
window-restore = Restaurar
window-maximize = Maximizar
window-minimize = Minimizar
window-about = Acerca de rvp / licencias (F1)
window-open-folder = Abrir carpeta (Shift+O)
window-open-file = Abrir archivo (O)
window-language = Idioma (Language)

## Control bar

controls-pause = Pausa (Space)
controls-play = Reproducir (Space)
controls-replay = Reproducir desde el principio (Space)
controls-prev = Anterior (B)
controls-next = Siguiente (N)
controls-exit-fullscreen = Salir de pantalla completa (F)
controls-fullscreen = Pantalla completa (F)
controls-playlist = Lista de reproducción (P)
speed-tooltip = Velocidad de reproducción ([ / ], Backspace para restablecer)
volume-mute = Silenciar (M)
volume-unmute = Activar sonido (M)
volume-level = Volumen { $percent }%
repeat-off = Repetir: desactivado
repeat-one = Repetir: este vídeo
repeat-all = Repetir: toda la lista
shuffle-on = Aleatorio: activado
shuffle-off = Aleatorio: desactivado

## Subtitle / audio menu

tracks-tooltip = Subtítulos / audio (V / S / A)
tracks-subtitles = Subtítulos
tracks-off = Desactivados
tracks-unsupported = { $label } (no compatible)
tracks-unsupported-why = Los subtítulos en imagen no son compatibles
tracks-load-subtitle = Cargar archivo de subtítulos…
tracks-timing = Sincronización
tracks-delay-later = Retrasar (X)
tracks-delay-earlier = Adelantar (Z)
tracks-delay-reset = Clic para restablecer
tracks-hide-subtitles = Ocultar subtítulos (V)
tracks-show-subtitles = Mostrar subtítulos (V)
tracks-audio = Audio
tracks-no-audio-output = Sin salida de audio
# $sign is "+" or "−", $value e.g. "0.5"
delay-seconds = { $sign }{ $value } s

## Track languages (from the file's metadata)

lang-ja = Japonés
lang-en = Inglés
lang-zh = Chino
lang-ko = Coreano
lang-fr = Francés
lang-de = Alemán
lang-es = Español
lang-it = Italiano
lang-ru = Ruso
lang-pt = Portugués

## Playlist panel

playlist-title = Lista de reproducción
playlist-close = Cerrar (P)
playlist-save = Guardar lista (Ctrl+S)
playlist-add = Añadir archivos
playlist-empty-hint = Suelta archivos aquí para añadirlos

## Empty window / drag and drop

empty-drop = Suelta un vídeo para reproducirlo
empty-open-file = Abrir archivo
empty-open-folder = Abrir carpeta
empty-hint = Pulsa O para abrir un archivo, Shift+O para una carpeta
drop-append = Suelta para añadir a la lista
drop-play = Suelta para reproducir

## File dialogs

dialog-open-video = Abrir vídeo
dialog-open-folder = Abrir carpeta
dialog-add-to-playlist = Añadir a la lista de reproducción
dialog-save-playlist = Guardar lista de reproducción
dialog-load-subtitle = Cargar archivo de subtítulos
dialog-filter-media = Vídeos y listas de reproducción
dialog-filter-playlist = Listas de reproducción
dialog-filter-subtitles = Subtítulos
dialog-filter-all = Todos los archivos

## On-screen messages

osd-play = Reproducir
osd-pause = Pausa
# $sign is "+" or "−", $value is a number of seconds
osd-seek = { $sign }{ $value } s
osd-muted = Silenciado
osd-speed = Velocidad { $speed }
osd-language = Idioma: { $name }
osd-added =
    { $count ->
        [one] 1 elemento añadido a la lista de reproducción
       *[other] { $count } elementos añadidos a la lista de reproducción
    }
osd-no-playable = No se encontraron vídeos reproducibles
osd-resumed = Reanudado desde la última posición ({ $time })
osd-playlist-end = Fin de la lista de reproducción
osd-playlist-empty = La lista de reproducción está vacía
osd-playlist-saved = Lista guardada: { $name }
osd-playlist-save-failed = No se pudo guardar la lista: { $error }
osd-open-video-first = Abre primero un vídeo
osd-subtitle-loaded =
    { $count ->
        [one] Subtítulos: { $name } (1 línea)
       *[other] Subtítulos: { $name } ({ $count } líneas)
    }
osd-audio-track = Audio: { $label }
osd-subtitle-track = Subtítulos: { $label }
osd-subtitles-shown = Subtítulos: visibles
osd-subtitles-hidden = Subtítulos: ocultos
osd-subtitle-delay = Sincronización de subtítulos: { $delay }
osd-hwdec-auto = Decodificación por hardware: automática
osd-hwdec-off = Decodificación por hardware: desactivada
osd-single-audio-track = Solo hay una pista de audio
osd-media-info =
    Vídeo: { $codec } { $width }×{ $height } · Decodificador: { $decoder }
    Audio: { $audio }
media-info-software = software
media-info-none = ninguno

## Errors

error-audio-device = No se puede abrir la salida de audio: { $error }
error-no-audio-device = No hay dispositivo de salida de audio
error-audio-config = No se puede obtener la configuración de la salida de audio
error-sample-format = Formato de muestra no compatible: { $format }
error-audio-start = No se puede iniciar la salida de audio
error-open-file = No se pudo abrir: { $path }
error-no-video-stream = No se encontró ninguna secuencia de vídeo
error-video-decoder = No se pudo abrir el decodificador de vídeo
error-audio-decoder = No se pudo abrir el decodificador de audio
error-decoder-open = No se puede abrir el decodificador: { $error }
error-audio-playback = No se puede reproducir el audio: { $error }
error-audio-track-open = No se puede abrir la pista de audio: { $error }
error-read = Error de lectura: { $error }
error-subtitle-load = No se pudieron cargar los subtítulos: { $path }
error-subtitle-empty = No se encontraron subtítulos: { $path }
error-hw-fallback = Falló la decodificación por hardware; se cambió a decodificación por software

## About dialog

about-version = Versión { $version }
about-close = Cerrar (Esc)
about-tab-license = Licencia
about-tab-software = Software de terceros
about-license-line = { $copyright }   Licencia: { $spdx }
about-notice =
    rvp es software libre: usted puede redistribuirlo y/o modificarlo bajo los términos de la Licencia Pública General de GNU (GPL) tal como la publica la Free Software Foundation, ya sea la versión 3 de la Licencia o (a su elección) cualquier versión posterior.
    rvp se distribuye con la esperanza de que sea útil, pero SIN NINGUNA GARANTÍA; sin siquiera la garantía implícita de COMERCIABILIDAD o IDONEIDAD PARA UN PROPÓSITO PARTICULAR. Consulte el texto completo de la GPL más abajo para más detalles.
about-filter-hint = Filtrar por nombre o licencia
about-count = { $shown } / { $total }
about-click-hint = Haz clic en una fila para ver el texto completo de la licencia.
about-col-name = Nombre
about-col-version = Versión
about-col-license = Licencia
about-col-kind = Tipo
about-back = Volver a la lista
about-build-config = Configuración de compilación
about-no-text = Sin texto de licencia (no incluido en rvp).
about-kind-library = Biblioteca
about-kind-font = Fuente
about-kind-crate = Crate
about-version-system = Sistema
about-version-bundled = Incluida
about-mit-standard = MIT (texto estándar)
about-note-ffmpeg =
    Decodificación de vídeo/audio, remuestreo y cambio de velocidad. Enlazada dinámicamente con las bibliotecas compartidas del sistema (libavcodec, etc.).
    Licencia (según la biblioteca): { $license }
about-note-alsa = Salida de audio (mediante cpal). Enlazada dinámicamente con la biblioteca compartida del sistema.
about-note-vulkan = Renderizado (mediante wgpu). Se carga en tiempo de ejecución.
about-note-wayland = Ventanas y arrastrar y soltar en Wayland. Usa la biblioteca compartida del sistema.
about-note-xkb = Entrada de teclado (mediante winit). Se carga en tiempo de ejecución.
about-note-phosphor = Fuente de iconos, incluida en el crate egui-phosphor.
about-font-name = Fuentes de la interfaz (sistema)
about-font-license = Según la fuente
about-font-note =
    Se cargan las fuentes del sistema elegidas con fontconfig (fc-match); no se incluyen con rvp.
    En uso: { $paths }
    Cada fuente tiene su propia licencia (p. ej., Noto Sans CJK: OFL-1.1).
about-font-missing = fontconfig no encontró ninguna fuente para la interfaz.
