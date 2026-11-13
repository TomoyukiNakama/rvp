# rvp UI strings — Bahasa Indonesia
# Format dan aturan: lihat bagian atas en.ftl.

language-name = Bahasa Indonesia

## Title bar

window-exit-fullscreen = Keluar dari layar penuh (Esc)
window-close = Tutup
window-restore = Pulihkan
window-maximize = Maksimalkan
window-minimize = Minimalkan
window-about = Tentang rvp / lisensi (F1)
window-open-folder = Buka folder (Shift+O)
window-open-file = Buka berkas (O)
window-language = Bahasa (Language)

## Control bar

controls-pause = Jeda (Space)
controls-play = Putar (Space)
controls-replay = Putar dari awal (Space)
controls-prev = Sebelumnya (B)
controls-next = Berikutnya (N)
controls-exit-fullscreen = Keluar dari layar penuh (F)
controls-fullscreen = Layar penuh (F)
controls-playlist = Daftar putar (P)
speed-tooltip = Kecepatan putar ([ / ], Backspace untuk mengatur ulang)
volume-mute = Bisukan (M)
volume-unmute = Bunyikan (M)
volume-level = Volume { $percent }%
repeat-off = Ulangi: mati
repeat-one = Ulangi: video ini
repeat-all = Ulangi: seluruh daftar
shuffle-on = Acak: hidup
shuffle-off = Acak: mati

## Subtitle / audio menu

tracks-tooltip = Subjudul / audio (V / S / A)
tracks-subtitles = Subjudul
tracks-off = Mati
tracks-unsupported = { $label } (tidak didukung)
tracks-unsupported-why = Subjudul berbasis gambar tidak didukung
tracks-load-subtitle = Muat berkas subjudul…
tracks-timing = Waktu
tracks-delay-later = Tunda (X)
tracks-delay-earlier = Majukan (Z)
tracks-delay-reset = Klik untuk mengatur ulang
tracks-hide-subtitles = Sembunyikan subjudul (V)
tracks-show-subtitles = Tampilkan subjudul (V)
tracks-audio = Audio
tracks-no-audio-output = Tidak ada keluaran audio
# $sign is "+" or "−", $value e.g. "0.5"
delay-seconds = { $sign }{ $value } d

## Track languages (from the file's metadata)

lang-ja = Jepang
lang-en = Inggris
lang-zh = Mandarin
lang-ko = Korea
lang-fr = Prancis
lang-de = Jerman
lang-es = Spanyol
lang-it = Italia
lang-ru = Rusia
lang-pt = Portugis

## Playlist panel

playlist-title = Daftar putar
playlist-close = Tutup (P)
playlist-save = Simpan daftar putar (Ctrl+S)
playlist-add = Tambah berkas
playlist-empty-hint = Jatuhkan berkas di sini untuk menambahkan

## Empty window / drag and drop

empty-drop = Jatuhkan video untuk memutarnya
empty-open-file = Buka berkas
empty-open-folder = Buka folder
empty-hint = Tekan O untuk membuka berkas, Shift+O untuk membuka folder
drop-append = Jatuhkan untuk menambah ke daftar putar
drop-play = Jatuhkan untuk memutar

## File dialogs

dialog-open-video = Buka video
dialog-open-folder = Buka folder
dialog-add-to-playlist = Tambah ke daftar putar
dialog-save-playlist = Simpan daftar putar
dialog-load-subtitle = Muat berkas subjudul
dialog-filter-media = Video dan daftar putar
dialog-filter-playlist = Daftar putar
dialog-filter-subtitles = Subjudul
dialog-filter-all = Semua berkas

## On-screen messages

osd-play = Putar
osd-pause = Jeda
# $sign is "+" or "−", $value is a number of seconds
osd-seek = { $sign }{ $value } d
osd-muted = Dibisukan
osd-speed = Kecepatan { $speed }
osd-language = Bahasa: { $name }
osd-added = { $count } item ditambahkan ke daftar putar
osd-no-playable = Tidak ada video yang dapat diputar
osd-resumed = Dilanjutkan dari posisi terakhir ({ $time })
osd-playlist-end = Akhir daftar putar
osd-playlist-empty = Daftar putar kosong
osd-playlist-saved = Daftar putar disimpan: { $name }
osd-playlist-save-failed = Gagal menyimpan daftar putar: { $error }
osd-open-video-first = Buka video terlebih dahulu
osd-subtitle-loaded = Subjudul: { $name } ({ $count } baris)
osd-audio-track = Audio: { $label }
osd-subtitle-track = Subjudul: { $label }
osd-subtitles-shown = Subjudul: tampil
osd-subtitles-hidden = Subjudul: tersembunyi
osd-subtitle-delay = Waktu subjudul: { $delay }
osd-hwdec-auto = Dekode perangkat keras: otomatis
osd-hwdec-off = Dekode perangkat keras: mati
osd-single-audio-track = Hanya ada satu trek audio
osd-media-info =
    Video: { $codec } { $width }×{ $height } · Dekoder: { $decoder }
    Audio: { $audio }
media-info-software = perangkat lunak
media-info-none = tidak ada

## Errors

error-audio-device = Tidak dapat membuka keluaran audio: { $error }
error-no-audio-device = Tidak ada perangkat keluaran audio
error-audio-config = Tidak dapat mengambil konfigurasi keluaran audio
error-sample-format = Format sampel tidak didukung: { $format }
error-audio-start = Tidak dapat memulai keluaran audio
error-open-file = Gagal membuka: { $path }
error-no-video-stream = Aliran video tidak ditemukan
error-video-decoder = Gagal membuka dekoder video
error-audio-decoder = Gagal membuka dekoder audio
error-decoder-open = Tidak dapat membuka dekoder: { $error }
error-audio-playback = Tidak dapat memutar audio: { $error }
error-audio-track-open = Tidak dapat membuka trek audio: { $error }
error-read = Kesalahan membaca: { $error }
error-subtitle-load = Tidak dapat memuat subjudul: { $path }
error-subtitle-empty = Subjudul tidak ditemukan: { $path }
error-hw-fallback = Dekode perangkat keras gagal; beralih ke dekode perangkat lunak

## About dialog

about-version = Versi { $version }
about-close = Tutup (Esc)
about-tab-license = Lisensi
about-tab-software = Perangkat lunak pihak ketiga
about-license-line = { $copyright }   Lisensi: { $spdx }
about-notice =
    rvp adalah perangkat lunak bebas: Anda dapat mendistribusikan ulang dan/atau memodifikasinya di bawah ketentuan GNU General Public License (GPL) yang diterbitkan oleh Free Software Foundation, baik versi 3 dari Lisensi, atau (sesuai pilihan Anda) versi yang lebih baru.
    rvp didistribusikan dengan harapan dapat bermanfaat, tetapi TANPA JAMINAN APA PUN; bahkan tanpa jaminan tersirat tentang KELAYAKAN UNTUK DIPERDAGANGKAN atau KESESUAIAN UNTUK TUJUAN TERTENTU. Lihat teks lengkap GPL di bawah untuk rincian.
about-filter-hint = Saring menurut nama atau lisensi
about-count = { $shown } / { $total }
about-click-hint = Klik baris untuk menampilkan teks lisensi lengkap.
about-col-name = Nama
about-col-version = Versi
about-col-license = Lisensi
about-col-kind = Jenis
about-back = Kembali ke daftar
about-build-config = Konfigurasi build
about-no-text = Tidak ada teks lisensi (tidak disertakan dalam rvp).
about-kind-library = Pustaka
about-kind-font = Fon
about-kind-crate = Crate
about-version-system = Sistem
about-version-bundled = Disertakan
about-mit-standard = MIT (teks standar)
about-note-ffmpeg =
    Dekode video/audio, resampling, dan perubahan kecepatan. Ditautkan secara dinamis ke pustaka bersama sistem (libavcodec dll.).
    Lisensi (menurut pustaka): { $license }
about-note-alsa = Keluaran audio (melalui cpal). Ditautkan secara dinamis ke pustaka bersama sistem.
about-note-vulkan = Rendering (melalui wgpu). Dimuat saat dijalankan.
about-note-wayland = Jendela dan seret-lepas di Wayland. Menggunakan pustaka bersama sistem.
about-note-xkb = Masukan papan ketik (melalui winit). Dimuat saat dijalankan.
about-note-phosphor = Fon ikon, disertakan dalam crate egui-phosphor.
about-font-name = Fon antarmuka (sistem)
about-font-license = Tergantung fon
about-font-note =
    Fon sistem yang dipilih dengan fontconfig (fc-match) dimuat; fon tersebut tidak disertakan dalam rvp.
    Digunakan: { $paths }
    Setiap fon memiliki lisensinya sendiri (mis. Noto Sans CJK: OFL-1.1).
about-font-missing = fontconfig tidak menemukan fon untuk antarmuka.
