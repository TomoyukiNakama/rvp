# rvp UI strings — 한국어
# 형식과 규칙은 en.ftl 첫머리를 참조.

language-name = 한국어

## Title bar

window-exit-fullscreen = 전체 화면 종료 (Esc)
window-close = 닫기
window-restore = 이전 크기로
window-maximize = 최대화
window-minimize = 최소화
window-about = rvp 정보 / 라이선스 (F1)
window-open-folder = 폴더 열기 (Shift+O)
window-open-file = 파일 열기 (O)
window-language = 언어 (Language)

## Control bar

controls-pause = 일시 정지 (Space)
controls-play = 재생 (Space)
controls-replay = 처음부터 재생 (Space)
controls-prev = 이전 (B)
controls-next = 다음 (N)
controls-exit-fullscreen = 전체 화면 종료 (F)
controls-fullscreen = 전체 화면 (F)
controls-playlist = 재생 목록 (P)
speed-tooltip = 재생 속도 ([ / ], Backspace로 초기화)
volume-mute = 음소거 (M)
volume-unmute = 음소거 해제 (M)
volume-level = 볼륨 { $percent }%
repeat-off = 반복: 끔
repeat-one = 반복: 현재 항목
repeat-all = 반복: 전체 재생 목록
shuffle-on = 셔플: 켬
shuffle-off = 셔플: 끔

## Subtitle / audio menu

tracks-tooltip = 자막 / 오디오 (V / S / A)
tracks-subtitles = 자막
tracks-off = 끔
tracks-unsupported = { $label } (지원 안 함)
tracks-unsupported-why = 이미지 자막은 지원하지 않습니다
tracks-load-subtitle = 자막 파일 불러오기…
tracks-timing = 싱크
tracks-delay-later = 늦추기 (X)
tracks-delay-earlier = 앞당기기 (Z)
tracks-delay-reset = 클릭하여 초기화
tracks-hide-subtitles = 자막 숨기기 (V)
tracks-show-subtitles = 자막 표시 (V)
tracks-audio = 오디오
tracks-no-audio-output = 오디오 출력 없음
delay-seconds = { $sign }{ $value }초

## Track languages (from the file's metadata)

lang-ja = 일본어
lang-en = 영어
lang-zh = 중국어
lang-ko = 한국어
lang-fr = 프랑스어
lang-de = 독일어
lang-es = 스페인어
lang-it = 이탈리아어
lang-ru = 러시아어
lang-pt = 포르투갈어

## Playlist panel

playlist-title = 재생 목록
playlist-close = 닫기 (P)
playlist-save = 재생 목록 저장 (Ctrl+S)
playlist-add = 파일 추가
playlist-empty-hint = 파일을 여기에 끌어다 놓아 추가

## Empty window / drag and drop

empty-drop = 동영상이나 음악을 끌어다 놓아 재생
empty-open-file = 파일 열기
empty-open-folder = 폴더 열기
empty-hint = O로 파일, Shift+O로 폴더를 열 수 있습니다
drop-append = 놓아서 재생 목록에 추가
drop-play = 놓아서 재생

## File dialogs

dialog-open-media = 파일 열기
dialog-open-folder = 폴더 열기
dialog-add-to-playlist = 재생 목록에 추가
dialog-save-playlist = 재생 목록 저장
dialog-load-subtitle = 자막 파일 불러오기
dialog-filter-media = 미디어 파일 및 재생 목록
dialog-filter-playlist = 재생 목록
dialog-filter-subtitles = 자막
dialog-filter-all = 모든 파일

## On-screen messages

osd-play = 재생
osd-pause = 일시 정지
osd-seek = { $sign }{ $value }초
osd-muted = 음소거
osd-speed = 속도 { $speed }
osd-language = 언어: { $name }
osd-added = { $count }개 항목을 재생 목록에 추가함
osd-no-playable = 재생할 수 있는 파일이 없습니다
osd-resumed = 마지막 위치에서 이어서 재생 ({ $time })
osd-playlist-end = 재생 목록의 끝입니다
osd-playlist-empty = 재생 목록이 비어 있습니다
osd-playlist-saved = 재생 목록 저장됨: { $name }
osd-playlist-save-failed = 재생 목록을 저장할 수 없습니다: { $error }
osd-open-video-first = 먼저 파일을 여세요
osd-subtitle-loaded = 자막: { $name } ({ $count }개)
osd-audio-track = 오디오: { $label }
osd-subtitle-track = 자막: { $label }
osd-subtitles-shown = 자막: 표시
osd-subtitles-hidden = 자막: 숨김
osd-subtitle-delay = 자막 싱크: { $delay }
osd-hwdec-auto = 하드웨어 디코딩: 자동
osd-hwdec-off = 하드웨어 디코딩: 끔
osd-single-audio-track = 오디오 트랙이 하나뿐입니다
osd-media-info =
    비디오: { $codec } { $width }×{ $height } · 디코더: { $decoder }
    오디오: { $audio }
osd-media-info-audio = 오디오: { $audio }
media-info-software = 소프트웨어
media-info-none = 없음

## Errors

error-audio-device = 오디오 출력을 열 수 없습니다: { $error }
error-no-audio-device = 오디오 출력 장치가 없습니다
error-audio-config = 오디오 출력 설정을 가져올 수 없습니다
error-sample-format = 지원하지 않는 샘플 형식: { $format }
error-audio-start = 오디오 출력을 시작할 수 없습니다
error-open-file = 열 수 없습니다: { $path }
error-no-playable-stream = 재생할 수 있는 비디오 또는 오디오 스트림을 찾을 수 없습니다
error-video-decoder = 비디오 디코더를 열 수 없습니다
error-audio-decoder = 오디오 디코더를 열 수 없습니다
error-decoder-open = 디코더를 열 수 없습니다: { $error }
error-audio-playback = 오디오를 재생할 수 없습니다: { $error }
error-audio-track-open = 오디오 트랙을 열 수 없습니다: { $error }
error-read = 읽기 오류: { $error }
error-subtitle-load = 자막을 불러올 수 없습니다: { $path }
error-subtitle-empty = 자막을 찾을 수 없습니다: { $path }
error-hw-fallback = 하드웨어 디코딩에 실패하여 소프트웨어 디코딩으로 전환했습니다

## About dialog

about-version = 버전 { $version }
about-close = 닫기 (Esc)
about-tab-license = 라이선스
about-tab-software = 서드파티 소프트웨어
about-license-line = { $copyright }   라이선스: { $spdx }
about-notice =
    rvp는 자유 소프트웨어입니다. 자유 소프트웨어 재단이 공표한 GNU 일반 공중 사용 허가서(GPL) 버전 3 또는 (선택에 따라) 그 이후 버전의 조건에 따라 이 프로그램을 재배포하거나 수정할 수 있습니다.
    rvp는 유용하게 쓰이기를 바라며 배포되지만, 어떠한 보증도 제공하지 않습니다. 상품성이나 특정 목적에의 적합성에 대한 묵시적 보증조차 없습니다. 자세한 내용은 아래 GPL 전문을 참조하세요.
about-filter-hint = 이름 또는 라이선스로 필터링
about-count = { $shown } / { $total }
about-click-hint = 행을 클릭하면 라이선스 전문을 표시합니다.
about-col-name = 이름
about-col-version = 버전
about-col-license = 라이선스
about-col-kind = 종류
about-back = 목록으로 돌아가기
about-build-config = 빌드 구성
about-no-text = 라이선스 본문이 없습니다 (rvp에 포함되어 있지 않음).
about-kind-library = 라이브러리
about-kind-font = 글꼴
about-kind-crate = 크레이트
about-version-system = 시스템
about-version-bundled = 내장
about-mit-standard = MIT (표준 텍스트)
about-note-ffmpeg =
    비디오/오디오 디코딩, 리샘플링, 속도 변경. 시스템 공유 라이브러리(libavcodec 등)에 동적으로 링크됩니다.
    라이선스(라이브러리가 보고한 값): { $license }
about-note-alsa = 오디오 출력 (cpal 경유). 시스템 공유 라이브러리에 동적으로 링크됩니다.
about-note-vulkan = 렌더링 (wgpu 경유). 실행 시 불러옵니다.
about-note-wayland = Wayland의 창 관리 및 드래그 앤 드롭. 시스템 공유 라이브러리를 사용합니다.
about-note-xkb = 키보드 입력 (winit 경유). 실행 시 불러옵니다.
about-note-phosphor = 아이콘 글꼴. egui-phosphor 크레이트에 포함되어 있습니다.
about-font-name = UI 글꼴 (시스템)
about-font-license = 글꼴에 따라 다름
about-font-note =
    fontconfig(fc-match)로 선택한 시스템 글꼴을 불러옵니다. rvp에는 포함되어 있지 않습니다.
    사용 중: { $paths }
    라이선스는 글꼴마다 다릅니다 (예: Noto Sans CJK는 OFL-1.1).
about-font-missing = fontconfig에서 UI에 사용할 글꼴을 찾지 못했습니다.
