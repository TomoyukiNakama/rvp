# rvp UI strings — 繁體中文
# 格式與規則請見 en.ftl 開頭。

language-name = 繁體中文

## Title bar

window-exit-fullscreen = 結束全螢幕 (Esc)
window-close = 關閉
window-restore = 還原
window-maximize = 最大化
window-minimize = 最小化
window-about = 關於 rvp / 授權 (F1)
window-open-folder = 開啟資料夾 (Shift+O)
window-open-file = 開啟檔案 (O)
window-language = 語言 (Language)

## Control bar

controls-pause = 暫停 (Space)
controls-play = 播放 (Space)
controls-replay = 從頭播放 (Space)
controls-prev = 上一個 (B)
controls-next = 下一個 (N)
controls-exit-fullscreen = 結束全螢幕 (F)
controls-fullscreen = 全螢幕 (F)
controls-playlist = 播放清單 (P)
speed-tooltip = 播放速度 ([ / ], Backspace 重設)
volume-mute = 靜音 (M)
volume-unmute = 取消靜音 (M)
volume-level = 音量 { $percent }%
repeat-off = 循環：關
repeat-one = 循環：目前項目
repeat-all = 循環：整個播放清單
shuffle-on = 隨機播放：開
shuffle-off = 隨機播放：關

## Subtitle / audio menu

tracks-tooltip = 字幕 / 音訊 (V / S / A)
tracks-subtitles = 字幕
tracks-off = 關閉
tracks-unsupported = { $label } (不支援)
tracks-unsupported-why = 不支援影像字幕
tracks-load-subtitle = 載入字幕檔…
tracks-timing = 時間軸
tracks-delay-later = 延後 (X)
tracks-delay-earlier = 提前 (Z)
tracks-delay-reset = 按一下重設
tracks-hide-subtitles = 隱藏字幕 (V)
tracks-show-subtitles = 顯示字幕 (V)
tracks-audio = 音訊
tracks-no-audio-output = 無音訊輸出
delay-seconds = { $sign }{ $value } 秒

## Track languages (from the file's metadata)

lang-ja = 日語
lang-en = 英語
lang-zh = 中文
lang-ko = 韓語
lang-fr = 法語
lang-de = 德語
lang-es = 西班牙語
lang-it = 義大利語
lang-ru = 俄語
lang-pt = 葡萄牙語

## Playlist panel

playlist-title = 播放清單
playlist-close = 關閉 (P)
playlist-save = 儲存播放清單 (Ctrl+S)
playlist-add = 新增檔案
playlist-empty-hint = 將檔案拖放到此處以新增

## Empty window / drag and drop

empty-drop = 拖放影片或音訊以播放
empty-open-file = 開啟檔案
empty-open-folder = 開啟資料夾
empty-hint = 按 O 開啟檔案，按 Shift+O 開啟資料夾
drop-append = 放開以加入播放清單
drop-play = 放開以播放

## File dialogs

dialog-open-media = 開啟檔案
dialog-open-folder = 開啟資料夾
dialog-add-to-playlist = 加入播放清單
dialog-save-playlist = 儲存播放清單
dialog-load-subtitle = 載入字幕檔
dialog-filter-media = 媒體檔案與播放清單
dialog-filter-playlist = 播放清單
dialog-filter-subtitles = 字幕
dialog-filter-all = 所有檔案

## On-screen messages

osd-play = 播放
osd-pause = 暫停
osd-seek = { $sign }{ $value } 秒
osd-muted = 已靜音
osd-speed = 速度 { $speed }
osd-language = 語言：{ $name }
osd-added = 已將 { $count } 個項目加入播放清單
osd-no-playable = 找不到可播放的檔案
osd-resumed = 已從上次的位置繼續播放 ({ $time })
osd-playlist-end = 已到播放清單結尾
osd-playlist-empty = 播放清單是空的
osd-playlist-saved = 播放清單已儲存：{ $name }
osd-playlist-save-failed = 無法儲存播放清單：{ $error }
osd-open-video-first = 請先開啟檔案
osd-subtitle-loaded = 字幕：{ $name } ({ $count } 則)
osd-audio-track = 音訊：{ $label }
osd-subtitle-track = 字幕：{ $label }
osd-subtitles-shown = 字幕：顯示
osd-subtitles-hidden = 字幕：隱藏
osd-subtitle-delay = 字幕時間：{ $delay }
osd-hwdec-auto = 硬體解碼：自動
osd-hwdec-off = 硬體解碼：關
osd-single-audio-track = 只有一個音軌
osd-media-info =
    影片：{ $codec } { $width }×{ $height } · 解碼器：{ $decoder }
    音訊：{ $audio }
osd-media-info-audio = 音訊：{ $audio }
media-info-software = 軟體
media-info-none = 無

## Errors

error-audio-device = 無法開啟音訊輸出：{ $error }
error-no-audio-device = 沒有音訊輸出裝置
error-audio-config = 無法取得音訊輸出設定
error-sample-format = 不支援的取樣格式：{ $format }
error-audio-start = 無法啟動音訊輸出
error-open-file = 無法開啟：{ $path }
error-no-playable-stream = 找不到可播放的影像或音訊串流
error-video-decoder = 無法開啟影片解碼器
error-audio-decoder = 無法開啟音訊解碼器
error-decoder-open = 無法開啟解碼器：{ $error }
error-audio-playback = 無法播放音訊：{ $error }
error-audio-track-open = 無法開啟音軌：{ $error }
error-read = 讀取錯誤：{ $error }
error-subtitle-load = 無法載入字幕：{ $path }
error-subtitle-empty = 找不到字幕：{ $path }
error-hw-fallback = 硬體解碼失敗，已切換為軟體解碼

## About dialog

about-version = 版本 { $version }
about-close = 關閉 (Esc)
about-tab-license = 授權
about-tab-software = 第三方軟體
about-license-line = { $copyright }   授權：{ $spdx }
about-notice =
    rvp 是自由軟體：您可以依據自由軟體基金會所發布的 GNU 通用公共授權條款（GPL）第 3 版，或（由您選擇）任何更新的版本，重新散布及／或修改本軟體。
    發布 rvp 是希望它能發揮功用，但不提供任何擔保；甚至不附帶對適售性或特定目的適用性的默示擔保。詳情請參閱下方的 GPL 全文。
about-filter-hint = 依名稱或授權篩選
about-count = { $shown } / { $total }
about-click-hint = 按一下某一列可檢視完整授權條款。
about-col-name = 名稱
about-col-version = 版本
about-col-license = 授權
about-col-kind = 類型
about-back = 返回清單
about-build-config = 建置設定
about-no-text = 無授權條款文字（未隨 rvp 附帶）。
about-kind-library = 函式庫
about-kind-font = 字型
about-kind-crate = Crate
about-version-system = 系統
about-version-bundled = 內建
about-mit-standard = MIT (標準文字)
about-note-ffmpeg =
    影片／音訊解碼、重新取樣與變速。以動態方式連結系統共用函式庫（libavcodec 等）。
    授權（由該函式庫回報）：{ $license }
about-note-alsa = 音訊輸出（透過 cpal）。以動態方式連結系統共用函式庫。
about-note-vulkan = 繪製（透過 wgpu）。於執行時載入。
about-note-wayland = Wayland 下的視窗與拖放。使用系統共用函式庫。
about-note-xkb = 鍵盤輸入（透過 winit）。於執行時載入。
about-note-phosphor = 圖示字型，隨 egui-phosphor crate 一同提供。
about-font-name = 介面字型（系統）
about-font-license = 視字型而定
about-font-note =
    載入以 fontconfig (fc-match) 選定的系統字型，這些字型不隨 rvp 附帶。
    使用中：{ $paths }
    每種字型各有其授權（例如 Noto Sans CJK 為 OFL-1.1）。
about-font-missing = fontconfig 找不到可用於介面的字型。
