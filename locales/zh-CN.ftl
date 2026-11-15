# rvp UI strings — 简体中文
# 格式与规则见 en.ftl 开头。

language-name = 简体中文

## Title bar

window-exit-fullscreen = 退出全屏 (Esc)
window-close = 关闭
window-restore = 还原
window-maximize = 最大化
window-minimize = 最小化
window-about = 关于 rvp / 许可证 (F1)
window-open-folder = 打开文件夹 (Shift+O)
window-open-file = 打开文件 (O)
window-language = 语言 (Language)

## Control bar

controls-pause = 暂停 (Space)
controls-play = 播放 (Space)
controls-replay = 从头播放 (Space)
controls-prev = 上一个 (B)
controls-next = 下一个 (N)
controls-exit-fullscreen = 退出全屏 (F)
controls-fullscreen = 全屏 (F)
controls-playlist = 播放列表 (P)
speed-tooltip = 播放速度 ([ / ], Backspace 重置)
volume-mute = 静音 (M)
volume-unmute = 取消静音 (M)
volume-level = 音量 { $percent }%
repeat-off = 循环：关
repeat-one = 循环：当前项目
repeat-all = 循环：整个播放列表
shuffle-on = 随机播放：开
shuffle-off = 随机播放：关

## Subtitle / audio menu

tracks-tooltip = 字幕 / 音频 (V / S / A)
tracks-subtitles = 字幕
tracks-off = 关闭
tracks-unsupported = { $label } (不支持)
tracks-unsupported-why = 不支持图像字幕
tracks-load-subtitle = 加载字幕文件…
tracks-timing = 时间轴
tracks-delay-later = 延后 (X)
tracks-delay-earlier = 提前 (Z)
tracks-delay-reset = 点击重置
tracks-hide-subtitles = 隐藏字幕 (V)
tracks-show-subtitles = 显示字幕 (V)
tracks-audio = 音频
tracks-no-audio-output = 无音频输出
delay-seconds = { $sign }{ $value } 秒

## Track languages (from the file's metadata)

lang-ja = 日语
lang-en = 英语
lang-zh = 中文
lang-ko = 韩语
lang-fr = 法语
lang-de = 德语
lang-es = 西班牙语
lang-it = 意大利语
lang-ru = 俄语
lang-pt = 葡萄牙语

## Playlist panel

playlist-title = 播放列表
playlist-close = 关闭 (P)
playlist-save = 保存播放列表 (Ctrl+S)
playlist-add = 添加文件
playlist-empty-hint = 将文件拖放到此处以添加

## Empty window / drag and drop

empty-drop = 拖放视频或音频以播放
empty-open-file = 打开文件
empty-open-folder = 打开文件夹
empty-hint = 按 O 打开文件，按 Shift+O 打开文件夹
drop-append = 松开以添加到播放列表
drop-play = 松开以播放

## File dialogs

dialog-open-media = 打开文件
dialog-open-folder = 打开文件夹
dialog-add-to-playlist = 添加到播放列表
dialog-save-playlist = 保存播放列表
dialog-load-subtitle = 加载字幕文件
dialog-filter-media = 媒体文件和播放列表
dialog-filter-playlist = 播放列表
dialog-filter-subtitles = 字幕
dialog-filter-all = 所有文件

## On-screen messages

osd-play = 播放
osd-pause = 暂停
osd-seek = { $sign }{ $value } 秒
osd-muted = 已静音
osd-speed = 速度 { $speed }
osd-language = 语言：{ $name }
osd-added = 已添加 { $count } 项到播放列表
osd-no-playable = 未找到可播放的文件
osd-resumed = 已从上次位置继续播放 ({ $time })
osd-playlist-end = 已到播放列表末尾
osd-playlist-empty = 播放列表为空
osd-playlist-saved = 播放列表已保存：{ $name }
osd-playlist-save-failed = 无法保存播放列表：{ $error }
osd-open-video-first = 请先打开文件
osd-subtitle-loaded = 字幕：{ $name } ({ $count } 条)
osd-audio-track = 音频：{ $label }
osd-subtitle-track = 字幕：{ $label }
osd-subtitles-shown = 字幕：显示
osd-subtitles-hidden = 字幕：隐藏
osd-subtitle-delay = 字幕时间：{ $delay }
osd-hwdec-auto = 硬件解码：自动
osd-hwdec-off = 硬件解码：关
osd-single-audio-track = 只有一个音轨
osd-media-info =
    视频：{ $codec } { $width }×{ $height } · 解码器：{ $decoder }
    音频：{ $audio }
osd-media-info-audio = 音频：{ $audio }
media-info-software = 软件
media-info-none = 无

## Errors

error-audio-device = 无法打开音频输出：{ $error }
error-no-audio-device = 没有音频输出设备
error-audio-config = 无法获取音频输出配置
error-sample-format = 不支持的采样格式：{ $format }
error-audio-start = 无法启动音频输出
error-open-file = 无法打开：{ $path }
error-no-playable-stream = 未找到可播放的视频或音频流
error-video-decoder = 无法打开视频解码器
error-audio-decoder = 无法打开音频解码器
error-decoder-open = 无法打开解码器：{ $error }
error-audio-playback = 无法播放音频：{ $error }
error-audio-track-open = 无法打开音轨：{ $error }
error-read = 读取错误：{ $error }
error-subtitle-load = 无法加载字幕：{ $path }
error-subtitle-empty = 未找到字幕：{ $path }
error-hw-fallback = 硬件解码失败，已切换为软件解码

## About dialog

about-version = 版本 { $version }
about-close = 关闭 (Esc)
about-tab-license = 许可证
about-tab-software = 第三方软件
about-license-line = { $copyright }   许可证：{ $spdx }
about-notice =
    rvp 是自由软件：您可以依据自由软件基金会发布的 GNU 通用公共许可证（GPL）第 3 版，或（由您选择）任何更新的版本，重新分发和/或修改本软件。
    发布 rvp 是希望它能有用，但不提供任何担保；甚至不附带对适销性或特定用途适用性的默示担保。详情请参阅下方的 GPL 全文。
about-filter-hint = 按名称或许可证筛选
about-count = { $shown } / { $total }
about-click-hint = 点击某一行可查看完整许可证文本。
about-col-name = 名称
about-col-version = 版本
about-col-license = 许可证
about-col-kind = 类型
about-back = 返回列表
about-build-config = 构建配置
about-no-text = 无许可证文本（未随 rvp 附带）。
about-kind-library = 库
about-kind-font = 字体
about-kind-crate = Crate
about-version-system = 系统
about-version-bundled = 内置
about-mit-standard = MIT (标准文本)
about-note-ffmpeg =
    视频/音频解码、重采样和变速。动态链接到系统共享库（libavcodec 等）。
    许可证（由该库报告）：{ $license }
about-note-alsa = 音频输出（通过 cpal）。动态链接到系统共享库。
about-note-vulkan = 渲染（通过 wgpu）。运行时加载。
about-note-wayland = Wayland 下的窗口与拖放。使用系统共享库。
about-note-xkb = 键盘输入（通过 winit）。运行时加载。
about-note-phosphor = 图标字体，随 egui-phosphor crate 一同提供。
about-font-name = 界面字体（系统）
about-font-license = 取决于字体
about-font-note =
    加载通过 fontconfig (fc-match) 选定的系统字体，这些字体不随 rvp 附带。
    使用中：{ $paths }
    每种字体有各自的许可证（例如 Noto Sans CJK 为 OFL-1.1）。
about-font-missing = fontconfig 未找到可用于界面的字体。
