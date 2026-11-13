# rvp

Rust と egui で作った、Linux 向けの動画プレイヤーです。

- 枠なしで角丸のダークテーマのウィンドウ。コントロールは操作しないと自動で隠れる
- FFmpeg でデコードし、YUV から RGB への変換は GPU（wgpu のシェーダ）で行う。BT.601/709/2020、10bit に対応
- ハードウェアデコード（CUDA / VAAPI）。使えない場合はソフトウェアに自動で切り替える
- 音声を基準にした同期、音量 0〜150%、ピッチを保った再生速度変更（0.25〜4 倍）
- プレイリスト（同じフォルダの動画から自動で作成、リピート、シャッフル、ドラッグでの並べ替え）
- 前回の位置からの再開
- 字幕：SRT / ASS / WebVTT（外部ファイルと埋め込み）。Shift_JIS にも対応。音声トラックと字幕トラックの切り替え

## 必要なもの

- Rust（stable、edition 2024）
- FFmpeg 8 の開発ライブラリ（`libavcodec-dev`、`libavformat-dev`、`libavfilter-dev`、`libswscale-dev`、`libswresample-dev`）
- ALSA の開発ライブラリ（`libasound2-dev`）。PipeWire や PulseAudio の環境でも ALSA 経由で音が出る
- 表示用のフォントは fontconfig で言語ごとに選ぶ。日本語・中国語・韓国語には CJK フォント（例：Noto Sans CJK）が必要

## 使い方

```bash
cargo run --release -- 動画.mkv [動画2.mp4 ...]
```

ファイルを開くとそのファイルだけ、フォルダを開くと中の動画をすべて再生します。ファイルやフォルダはウィンドウにドロップしても開けます（Wayland / X11 の両方に対応）。字幕ファイルをドロップすると、再生中の動画の字幕として読み込みます。

## キー操作

| キー | 動作 |
|---|---|
| Space | 再生 / 一時停止 |
| ← / →、Shift + ← / → | 5 秒 / 60 秒シーク |
| ↑ / ↓、ホイール | 音量 |
| M | ミュート |
| `[` / `]`、Backspace | 再生速度を下げる / 上げる、等速に戻す |
| N / B | 次 / 前の項目 |
| L | ループ（オフ → この動画 → プレイリスト全体） |
| P | プレイリスト |
| V / S / A | 字幕の表示切り替え / 字幕トラック / 音声トラック |
| Z / X | 字幕を 0.1 秒早める / 遅らせる |
| F、ダブルクリック、Esc | フルスクリーン / 解除 |
| F1 | rvp について（ライセンスと使用しているソフトウェア） |
| O / Shift+O | ファイルを開く / フォルダを開く |
| I | メディア情報 |
| Ctrl+H | ハードウェアデコードの切り替え |
| Q | 終了 |

## 設定

`~/.config/rvp/config.toml`（アプリが自動で書き込みます。手で編集してもかまいません）

```toml
volume = 1.0              # 0.0〜1.5
muted = false
repeat = "off"            # off / all / one
shuffle = false
queue_siblings = false    # true: ファイルを開くと同じフォルダの動画も続けて再生
resume = true
seek_small = 5.0
seek_large = 60.0
hwdec = "auto"            # auto / off / cuda / vaapi
```

再生位置は `~/.local/state/rvp/resume.toml` に保存されます。

## 開発

```bash
cargo test                    # 単体テストと統合テスト（ffmpeg CLI でテスト動画を作る）
scripts/gen-test-media.sh     # 手動確認用のテスト動画を target/test-media/ に作る
```

依存クレートを変えたら、ライセンス一覧（`licenses/third-party.toml`）を作り直してください。

```bash
python3 scripts/gen-licenses.py           # 再生成
python3 scripts/gen-licenses.py --check   # 最新かどうかの確認
```

仕様は `SPEC.md`、開発メモは `CLAUDE.md` を参照してください。

## ライセンス

rvp は [GNU General Public License v3.0](LICENSE)（またはそれ以降のバージョン）で配布します。
システムの FFmpeg（GPL ビルド）を動的リンクするため GPL にしています。

使用しているソフトウェアとそのライセンスの一覧は、アプリの「rvp について」（タイトルバーの ⓘ ボタンか F1）で確認できます。
クレートのライセンス本文は `licenses/third-party.toml` にあります。
