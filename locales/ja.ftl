# rvp UI 文字列 — 日本語
# 形式・ルールは en.ftl の先頭を参照。

language-name = 日本語

## Title bar

window-exit-fullscreen = フルスクリーン解除 (Esc)
window-close = 閉じる
window-restore = 元のサイズに戻す
window-maximize = 最大化
window-minimize = 最小化
window-about = rvp について・ライセンス (F1)
window-open-folder = フォルダを開く (Shift+O)
window-open-file = ファイルを開く (O)
window-language = 言語 (Language)

## Control bar

controls-pause = 一時停止 (Space)
controls-play = 再生 (Space)
controls-replay = 最初から再生 (Space)
controls-prev = 前へ (B)
controls-next = 次へ (N)
controls-exit-fullscreen = フルスクリーン解除 (F)
controls-fullscreen = フルスクリーン (F)
controls-playlist = プレイリスト (P)
speed-tooltip = 再生速度 ([ / ], Backspace でリセット)
volume-mute = ミュート (M)
volume-unmute = ミュート解除 (M)
volume-level = 音量 { $percent }%
repeat-off = ループ: オフ
repeat-one = ループ: この動画
repeat-all = ループ: プレイリスト全体
shuffle-on = シャッフル: オン
shuffle-off = シャッフル: オフ

## Subtitle / audio menu

tracks-tooltip = 字幕・音声 (V / S / A)
tracks-subtitles = 字幕
tracks-off = オフ
tracks-unsupported = { $label } (未対応)
tracks-unsupported-why = 画像字幕には未対応です
tracks-load-subtitle = 字幕ファイルを読み込む…
tracks-timing = タイミング
tracks-delay-later = 遅らせる (X)
tracks-delay-earlier = 早める (Z)
tracks-delay-reset = クリックでリセット
tracks-hide-subtitles = 字幕を隠す (V)
tracks-show-subtitles = 字幕を表示 (V)
tracks-audio = 音声
tracks-no-audio-output = 音声出力がありません
delay-seconds = { $sign }{ $value } 秒

## Track languages

lang-ja = 日本語
lang-en = 英語
lang-zh = 中国語
lang-ko = 韓国語
lang-fr = フランス語
lang-de = ドイツ語
lang-es = スペイン語
lang-it = イタリア語
lang-ru = ロシア語
lang-pt = ポルトガル語

## Playlist panel

playlist-title = プレイリスト
playlist-close = 閉じる (P)
playlist-save = プレイリストを保存 (Ctrl+S)
playlist-add = ファイルを追加
playlist-empty-hint = ファイルをドロップして追加

## Empty window / drag and drop

empty-drop = 動画をドロップして再生
empty-open-file = ファイルを開く
empty-open-folder = フォルダを開く
empty-hint = O でファイル、Shift+O でフォルダを開けます
drop-append = ドロップしてプレイリストに追加
drop-play = ドロップして再生

## File dialogs

dialog-open-video = 動画を開く
dialog-open-folder = フォルダを開く
dialog-add-to-playlist = プレイリストに追加
dialog-save-playlist = プレイリストを保存
dialog-load-subtitle = 字幕ファイルを読み込む
dialog-filter-media = 動画・プレイリスト
dialog-filter-playlist = プレイリスト
dialog-filter-subtitles = 字幕
dialog-filter-all = すべてのファイル

## On-screen messages

osd-play = 再生
osd-pause = 一時停止
osd-seek = { $sign }{ $value }秒
osd-muted = ミュート
osd-speed = 速度 { $speed }
osd-language = 言語: { $name }
osd-added = { $count } 件をプレイリストに追加
osd-no-playable = 再生できる動画が見つかりません
osd-resumed = 前回の位置から再開 ({ $time })
osd-playlist-end = プレイリストの最後です
osd-playlist-empty = プレイリストが空です
osd-playlist-saved = プレイリストを保存: { $name }
osd-playlist-save-failed = プレイリストを保存できません: { $error }
osd-open-video-first = 先に動画を開いてください
osd-subtitle-loaded = 字幕: { $name } ({ $count } 件)
osd-audio-track = 音声: { $label }
osd-subtitle-track = 字幕: { $label }
osd-subtitles-shown = 字幕: 表示
osd-subtitles-hidden = 字幕: 非表示
osd-subtitle-delay = 字幕のタイミング: { $delay }
osd-hwdec-auto = ハードウェアデコード: 自動
osd-hwdec-off = ハードウェアデコード: オフ
osd-single-audio-track = 音声トラックは 1 つだけです
osd-media-info =
    映像: { $codec } { $width }×{ $height } · デコード: { $decoder }
    音声: { $audio }
media-info-software = ソフトウェア
media-info-none = なし

## Errors

error-audio-device = 音声出力を開けません: { $error }
error-no-audio-device = 音声出力デバイスがありません
error-audio-config = 音声出力の設定を取得できません
error-sample-format = 未対応のサンプル形式: { $format }
error-audio-start = 音声出力を開始できません
error-open-file = 開けませんでした: { $path }
error-no-video-stream = 映像ストリームが見つかりません
error-video-decoder = 映像デコーダを開けませんでした
error-audio-decoder = 音声デコーダを開けませんでした
error-decoder-open = デコーダを開けません: { $error }
error-audio-playback = 音声を再生できません: { $error }
error-audio-track-open = 音声トラックを開けません: { $error }
error-read = 読み込みエラー: { $error }
error-subtitle-load = 字幕を読み込めません: { $path }
error-subtitle-empty = 字幕が見つかりません: { $path }
error-hw-fallback = ハードウェアデコードに失敗したため、ソフトウェアに切り替えました

## About dialog

about-version = バージョン { $version }
about-close = 閉じる (Esc)
about-tab-license = ライセンス
about-tab-software = 使用しているソフトウェア
about-license-line = { $copyright }　ライセンス: { $spdx }
about-notice =
    rvp はフリーソフトウェアです。Free Software Foundation が公表した GNU General Public License（GPL）のバージョン 3、または（選択により）それ以降のバージョンの条件の下で、再配布や改変ができます。
    rvp は役に立つことを願って配布していますが、無保証です。商品性や特定目的への適合性の暗黙の保証もありません。詳しくは以下の GPL の全文を参照してください。
about-filter-hint = 名前・ライセンスで絞り込み
about-count = { $shown } / { $total } 件
about-click-hint = 行をクリックするとライセンス全文を表示します。
about-col-name = 名前
about-col-version = バージョン
about-col-license = ライセンス
about-col-kind = 種別
about-back = 一覧に戻る
about-build-config = ビルド設定
about-no-text = ライセンス本文はありません（rvp に同梱していないため）。
about-kind-library = ライブラリ
about-kind-font = フォント
about-kind-crate = クレート
about-version-system = システム
about-version-bundled = 同梱
about-mit-standard = MIT (標準テキスト)
about-note-ffmpeg =
    動画・音声のデコード、リサンプリング、速度変更。システムの共有ライブラリ（libavcodec ほか）を動的リンクしています。
    ライセンス（ライブラリの申告）: { $license }
about-note-alsa = 音声出力（cpal 経由）。システムの共有ライブラリを動的リンクしています。
about-note-vulkan = 描画（wgpu 経由）。実行時に読み込みます。
about-note-wayland = Wayland 上のウィンドウとドラッグ＆ドロップ。システムの共有ライブラリを使用しています。
about-note-xkb = キーボード入力（winit 経由）。実行時に読み込みます。
about-note-phosphor = アイコンフォント。egui-phosphor クレートに同梱されています。
about-font-name = UI フォント（システム）
about-font-license = フォントによる
about-font-note =
    fontconfig（fc-match）で選んだシステムのフォントを読み込んでいます。rvp には同梱していません。
    使用中: { $paths }
    ライセンスはフォントごとに異なります（例：Noto Sans CJK は OFL-1.1）。
about-font-missing = fontconfig で表示用フォントが見つかりませんでした。
