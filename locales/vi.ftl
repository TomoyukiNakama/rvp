# rvp UI strings — Tiếng Việt
# Định dạng và quy tắc: xem đầu tệp en.ftl.

language-name = Tiếng Việt

## Title bar

window-exit-fullscreen = Thoát toàn màn hình (Esc)
window-close = Đóng
window-restore = Khôi phục
window-maximize = Phóng to
window-minimize = Thu nhỏ
window-about = Giới thiệu rvp / giấy phép (F1)
window-open-folder = Mở thư mục (Shift+O)
window-open-file = Mở tệp (O)
window-language = Ngôn ngữ (Language)

## Control bar

controls-pause = Tạm dừng (Space)
controls-play = Phát (Space)
controls-replay = Phát lại từ đầu (Space)
controls-prev = Trước (B)
controls-next = Tiếp (N)
controls-exit-fullscreen = Thoát toàn màn hình (F)
controls-fullscreen = Toàn màn hình (F)
controls-playlist = Danh sách phát (P)
speed-tooltip = Tốc độ phát ([ / ], Backspace để đặt lại)
volume-mute = Tắt tiếng (M)
volume-unmute = Bật tiếng (M)
volume-level = Âm lượng { $percent }%
repeat-off = Lặp: tắt
repeat-one = Lặp: mục này
repeat-all = Lặp: cả danh sách phát
shuffle-on = Ngẫu nhiên: bật
shuffle-off = Ngẫu nhiên: tắt

## Subtitle / audio menu

tracks-tooltip = Phụ đề / âm thanh (V / S / A)
tracks-subtitles = Phụ đề
tracks-off = Tắt
tracks-unsupported = { $label } (không hỗ trợ)
tracks-unsupported-why = Không hỗ trợ phụ đề dạng hình ảnh
tracks-load-subtitle = Tải tệp phụ đề…
tracks-timing = Thời gian
tracks-delay-later = Trễ hơn (X)
tracks-delay-earlier = Sớm hơn (Z)
tracks-delay-reset = Nhấp để đặt lại
tracks-hide-subtitles = Ẩn phụ đề (V)
tracks-show-subtitles = Hiện phụ đề (V)
tracks-audio = Âm thanh
tracks-no-audio-output = Không có đầu ra âm thanh
delay-seconds = { $sign }{ $value } giây

## Track languages (from the file's metadata)

lang-ja = Tiếng Nhật
lang-en = Tiếng Anh
lang-zh = Tiếng Trung
lang-ko = Tiếng Hàn
lang-fr = Tiếng Pháp
lang-de = Tiếng Đức
lang-es = Tiếng Tây Ban Nha
lang-it = Tiếng Ý
lang-ru = Tiếng Nga
lang-pt = Tiếng Bồ Đào Nha

## Playlist panel

playlist-title = Danh sách phát
playlist-close = Đóng (P)
playlist-save = Lưu danh sách phát (Ctrl+S)
playlist-add = Thêm tệp
playlist-empty-hint = Thả tệp vào đây để thêm

## Empty window / drag and drop

empty-drop = Thả video hoặc âm thanh vào để phát
empty-open-file = Mở tệp
empty-open-folder = Mở thư mục
empty-hint = Nhấn O để mở tệp, Shift+O để mở thư mục
drop-append = Thả để thêm vào danh sách phát
drop-play = Thả để phát

## File dialogs

dialog-open-media = Mở tệp
dialog-open-folder = Mở thư mục
dialog-add-to-playlist = Thêm vào danh sách phát
dialog-save-playlist = Lưu danh sách phát
dialog-load-subtitle = Tải tệp phụ đề
dialog-filter-media = Tệp đa phương tiện và danh sách phát
dialog-filter-playlist = Danh sách phát
dialog-filter-subtitles = Phụ đề
dialog-filter-all = Tất cả tệp

## On-screen messages

osd-play = Phát
osd-pause = Tạm dừng
osd-seek = { $sign }{ $value } giây
osd-muted = Đã tắt tiếng
osd-speed = Tốc độ { $speed }
osd-language = Ngôn ngữ: { $name }
osd-added = Đã thêm { $count } mục vào danh sách phát
osd-no-playable = Không tìm thấy tệp có thể phát
osd-resumed = Tiếp tục từ vị trí lần trước ({ $time })
osd-playlist-end = Đã hết danh sách phát
osd-playlist-empty = Danh sách phát trống
osd-playlist-saved = Đã lưu danh sách phát: { $name }
osd-playlist-save-failed = Không thể lưu danh sách phát: { $error }
osd-open-video-first = Hãy mở tệp trước
osd-subtitle-loaded = Phụ đề: { $name } ({ $count } dòng)
osd-audio-track = Âm thanh: { $label }
osd-subtitle-track = Phụ đề: { $label }
osd-subtitles-shown = Phụ đề: hiện
osd-subtitles-hidden = Phụ đề: ẩn
osd-subtitle-delay = Thời gian phụ đề: { $delay }
osd-hwdec-auto = Giải mã phần cứng: tự động
osd-hwdec-off = Giải mã phần cứng: tắt
osd-single-audio-track = Chỉ có một luồng âm thanh
osd-media-info =
    Video: { $codec } { $width }×{ $height } · Bộ giải mã: { $decoder }
    Âm thanh: { $audio }
osd-media-info-audio = Âm thanh: { $audio }
media-info-software = phần mềm
media-info-none = không có

## Errors

error-audio-device = Không thể mở đầu ra âm thanh: { $error }
error-no-audio-device = Không có thiết bị đầu ra âm thanh
error-audio-config = Không thể lấy cấu hình đầu ra âm thanh
error-sample-format = Định dạng mẫu không được hỗ trợ: { $format }
error-audio-start = Không thể khởi động đầu ra âm thanh
error-open-file = Không thể mở: { $path }
error-no-playable-stream = Không tìm thấy luồng video hoặc âm thanh có thể phát
error-video-decoder = Không thể mở bộ giải mã video
error-audio-decoder = Không thể mở bộ giải mã âm thanh
error-decoder-open = Không thể mở bộ giải mã: { $error }
error-audio-playback = Không thể phát âm thanh: { $error }
error-audio-track-open = Không thể mở luồng âm thanh: { $error }
error-read = Lỗi đọc: { $error }
error-subtitle-load = Không thể tải phụ đề: { $path }
error-subtitle-empty = Không tìm thấy phụ đề: { $path }
error-hw-fallback = Giải mã phần cứng thất bại; đã chuyển sang giải mã phần mềm

## About dialog

about-version = Phiên bản { $version }
about-close = Đóng (Esc)
about-tab-license = Giấy phép
about-tab-software = Phần mềm bên thứ ba
about-license-line = { $copyright }   Giấy phép: { $spdx }
about-notice =
    rvp là phần mềm tự do: bạn có thể phân phối lại và/hoặc sửa đổi nó theo các điều khoản của Giấy phép Công cộng GNU (GPL) do Free Software Foundation công bố, phiên bản 3 hoặc (tùy bạn chọn) bất kỳ phiên bản nào mới hơn.
    rvp được phân phối với hy vọng sẽ hữu ích, nhưng KHÔNG CÓ BẤT KỲ BẢO ĐẢM NÀO; thậm chí không có bảo đảm ngầm định về KHẢ NĂNG THƯƠNG MẠI hay SỰ PHÙ HỢP CHO MỘT MỤC ĐÍCH CỤ THỂ. Xem toàn văn GPL bên dưới để biết chi tiết.
about-filter-hint = Lọc theo tên hoặc giấy phép
about-count = { $shown } / { $total }
about-click-hint = Nhấp vào một hàng để xem toàn văn giấy phép.
about-col-name = Tên
about-col-version = Phiên bản
about-col-license = Giấy phép
about-col-kind = Loại
about-back = Quay lại danh sách
about-build-config = Cấu hình bản dựng
about-no-text = Không có văn bản giấy phép (không kèm theo rvp).
about-kind-library = Thư viện
about-kind-font = Phông chữ
about-kind-crate = Crate
about-version-system = Hệ thống
about-version-bundled = Đi kèm
about-mit-standard = MIT (văn bản chuẩn)
about-note-ffmpeg =
    Giải mã video/âm thanh, lấy mẫu lại và đổi tốc độ. Liên kết động với các thư viện dùng chung của hệ thống (libavcodec, v.v.).
    Giấy phép (do thư viện báo cáo): { $license }
about-note-alsa = Đầu ra âm thanh (qua cpal). Liên kết động với thư viện dùng chung của hệ thống.
about-note-vulkan = Kết xuất đồ họa (qua wgpu). Được nạp khi chạy.
about-note-wayland = Cửa sổ và kéo thả trên Wayland. Dùng thư viện dùng chung của hệ thống.
about-note-xkb = Nhập liệu bàn phím (qua winit). Được nạp khi chạy.
about-note-phosphor = Phông chữ biểu tượng, đi kèm crate egui-phosphor.
about-font-name = Phông chữ giao diện (hệ thống)
about-font-license = Tùy theo phông chữ
about-font-note =
    Nạp các phông chữ hệ thống do fontconfig (fc-match) chọn; chúng không đi kèm rvp.
    Đang dùng: { $paths }
    Mỗi phông chữ có giấy phép riêng (ví dụ Noto Sans CJK là OFL-1.1).
about-font-missing = fontconfig không tìm thấy phông chữ nào cho giao diện.
