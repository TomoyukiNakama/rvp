# rvp UI strings — Русский
# Формат и правила: см. начало en.ftl.

language-name = Русский

## Title bar

window-exit-fullscreen = Выйти из полноэкранного режима (Esc)
window-close = Закрыть
window-restore = Восстановить
window-maximize = Развернуть
window-minimize = Свернуть
window-about = О программе rvp / лицензии (F1)
window-open-folder = Открыть папку (Shift+O)
window-open-file = Открыть файл (O)
window-language = Язык (Language)

## Control bar

controls-pause = Пауза (Space)
controls-play = Воспроизвести (Space)
controls-replay = Воспроизвести с начала (Space)
controls-prev = Назад (B)
controls-next = Далее (N)
controls-exit-fullscreen = Выйти из полноэкранного режима (F)
controls-fullscreen = Во весь экран (F)
controls-playlist = Плейлист (P)
speed-tooltip = Скорость воспроизведения ([ / ], Backspace — сброс)
volume-mute = Выключить звук (M)
volume-unmute = Включить звук (M)
volume-level = Громкость { $percent }%
repeat-off = Повтор: выкл.
repeat-one = Повтор: это видео
repeat-all = Повтор: весь плейлист
shuffle-on = Перемешивание: вкл.
shuffle-off = Перемешивание: выкл.

## Subtitle / audio menu

tracks-tooltip = Субтитры / звук (V / S / A)
tracks-subtitles = Субтитры
tracks-off = Выкл.
tracks-unsupported = { $label } (не поддерживается)
tracks-unsupported-why = Графические субтитры не поддерживаются
tracks-load-subtitle = Загрузить файл субтитров…
tracks-timing = Синхронизация
tracks-delay-later = Задержать (X)
tracks-delay-earlier = Ускорить (Z)
tracks-delay-reset = Щёлкните для сброса
tracks-hide-subtitles = Скрыть субтитры (V)
tracks-show-subtitles = Показать субтитры (V)
tracks-audio = Звук
tracks-no-audio-output = Нет вывода звука
# $sign is "+" or "−", $value e.g. "0.5"
delay-seconds = { $sign }{ $value } с

## Track languages (from the file's metadata)

lang-ja = Японский
lang-en = Английский
lang-zh = Китайский
lang-ko = Корейский
lang-fr = Французский
lang-de = Немецкий
lang-es = Испанский
lang-it = Итальянский
lang-ru = Русский
lang-pt = Португальский

## Playlist panel

playlist-title = Плейлист
playlist-close = Закрыть (P)
playlist-save = Сохранить плейлист (Ctrl+S)
playlist-add = Добавить файлы
playlist-empty-hint = Перетащите файлы сюда, чтобы добавить

## Empty window / drag and drop

empty-drop = Перетащите видео, чтобы воспроизвести
empty-open-file = Открыть файл
empty-open-folder = Открыть папку
empty-hint = Нажмите O, чтобы открыть файл, или Shift+O, чтобы открыть папку
drop-append = Отпустите, чтобы добавить в плейлист
drop-play = Отпустите, чтобы воспроизвести

## File dialogs

dialog-open-video = Открыть видео
dialog-open-folder = Открыть папку
dialog-add-to-playlist = Добавить в плейлист
dialog-save-playlist = Сохранить плейлист
dialog-load-subtitle = Загрузить файл субтитров
dialog-filter-media = Видео и плейлисты
dialog-filter-playlist = Плейлисты
dialog-filter-subtitles = Субтитры
dialog-filter-all = Все файлы

## On-screen messages

osd-play = Воспроизведение
osd-pause = Пауза
# $sign is "+" or "−", $value is a number of seconds
osd-seek = { $sign }{ $value } с
osd-muted = Звук выключен
osd-speed = Скорость { $speed }
osd-language = Язык: { $name }
osd-added =
    { $count ->
        [one] В плейлист добавлен { $count } элемент
        [few] В плейлист добавлено { $count } элемента
        [many] В плейлист добавлено { $count } элементов
       *[other] В плейлист добавлено { $count } элемента
    }
osd-no-playable = Воспроизводимые видео не найдены
osd-resumed = Продолжено с последней позиции ({ $time })
osd-playlist-end = Конец плейлиста
osd-playlist-empty = Плейлист пуст
osd-playlist-saved = Плейлист сохранён: { $name }
osd-playlist-save-failed = Не удалось сохранить плейлист: { $error }
osd-open-video-first = Сначала откройте видео
osd-subtitle-loaded =
    { $count ->
        [one] Субтитры: { $name } ({ $count } реплика)
        [few] Субтитры: { $name } ({ $count } реплики)
        [many] Субтитры: { $name } ({ $count } реплик)
       *[other] Субтитры: { $name } ({ $count } реплики)
    }
osd-audio-track = Звук: { $label }
osd-subtitle-track = Субтитры: { $label }
osd-subtitles-shown = Субтитры: показаны
osd-subtitles-hidden = Субтитры: скрыты
osd-subtitle-delay = Синхронизация субтитров: { $delay }
osd-hwdec-auto = Аппаратное декодирование: авто
osd-hwdec-off = Аппаратное декодирование: выкл.
osd-single-audio-track = Есть только одна звуковая дорожка
osd-media-info =
    Видео: { $codec } { $width }×{ $height } · Декодер: { $decoder }
    Звук: { $audio }
media-info-software = программный
media-info-none = нет

## Errors

error-audio-device = Не удалось открыть вывод звука: { $error }
error-no-audio-device = Нет устройства вывода звука
error-audio-config = Не удалось получить конфигурацию вывода звука
error-sample-format = Неподдерживаемый формат сэмплов: { $format }
error-audio-start = Не удалось запустить вывод звука
error-open-file = Не удалось открыть: { $path }
error-no-video-stream = Видеопоток не найден
error-video-decoder = Не удалось открыть видеодекодер
error-audio-decoder = Не удалось открыть аудиодекодер
error-decoder-open = Не удалось открыть декодер: { $error }
error-audio-playback = Не удалось воспроизвести звук: { $error }
error-audio-track-open = Не удалось открыть звуковую дорожку: { $error }
error-read = Ошибка чтения: { $error }
error-subtitle-load = Не удалось загрузить субтитры: { $path }
error-subtitle-empty = Субтитры не найдены: { $path }
error-hw-fallback = Аппаратное декодирование не удалось; включено программное

## About dialog

about-version = Версия { $version }
about-close = Закрыть (Esc)
about-tab-license = Лицензия
about-tab-software = Стороннее ПО
about-license-line = { $copyright }   Лицензия: { $spdx }
about-notice =
    rvp — свободная программа: вы можете перераспространять её и/или изменять её на условиях Стандартной общественной лицензии GNU (GPL) в том виде, в каком она была опубликована Фондом свободного программного обеспечения (Free Software Foundation); либо версии 3 лицензии, либо (по вашему выбору) любой более поздней версии.
    rvp распространяется в надежде, что она будет полезной, но БЕЗО ВСЯКИХ ГАРАНТИЙ; в том числе без неявной гарантии ТОВАРНОГО СОСТОЯНИЯ ПРИ ПРОДАЖЕ и ПРИГОДНОСТИ ДЛЯ ОПРЕДЕЛЁННЫХ ЦЕЛЕЙ. Подробнее см. полный текст GPL ниже.
about-filter-hint = Фильтр по имени или лицензии
about-count = { $shown } / { $total }
about-click-hint = Щёлкните строку, чтобы показать полный текст лицензии.
about-col-name = Название
about-col-version = Версия
about-col-license = Лицензия
about-col-kind = Тип
about-back = К списку
about-build-config = Конфигурация сборки
about-no-text = Текст лицензии отсутствует (не входит в состав rvp).
about-kind-library = Библиотека
about-kind-font = Шрифт
about-kind-crate = Крейт
about-version-system = Системная
about-version-bundled = Встроенная
about-mit-standard = MIT (стандартный текст)
about-note-ffmpeg =
    Декодирование видео и звука, передискретизация и изменение скорости. Динамически связана с системными общими библиотеками (libavcodec и др.).
    Лицензия (по данным библиотеки): { $license }
about-note-alsa = Вывод звука (через cpal). Динамически связана с системной общей библиотекой.
about-note-vulkan = Отрисовка (через wgpu). Загружается во время работы.
about-note-wayland = Окна и перетаскивание в Wayland. Использует системную общую библиотеку.
about-note-xkb = Ввод с клавиатуры (через winit). Загружается во время работы.
about-note-phosphor = Шрифт значков, входит в состав крейта egui-phosphor.
about-font-name = Шрифты интерфейса (системные)
about-font-license = Зависит от шрифта
about-font-note =
    Загружаются системные шрифты, выбранные через fontconfig (fc-match); в состав rvp они не входят.
    Используются: { $paths }
    У каждого шрифта своя лицензия (например, Noto Sans CJK — OFL-1.1).
about-font-missing = fontconfig не нашёл шрифт для интерфейса.
