# rvp UI strings — Português (Brasil)
# Formato e regras: veja o início de en.ftl.

language-name = Português (Brasil)

## Title bar

window-exit-fullscreen = Sair da tela cheia (Esc)
window-close = Fechar
window-restore = Restaurar
window-maximize = Maximizar
window-minimize = Minimizar
window-about = Sobre o rvp / licenças (F1)
window-open-folder = Abrir pasta (Shift+O)
window-open-file = Abrir arquivo (O)
window-language = Idioma (Language)

## Control bar

controls-pause = Pausar (Space)
controls-play = Reproduzir (Space)
controls-replay = Reproduzir desde o início (Space)
controls-prev = Anterior (B)
controls-next = Próximo (N)
controls-exit-fullscreen = Sair da tela cheia (F)
controls-fullscreen = Tela cheia (F)
controls-playlist = Lista de reprodução (P)
speed-tooltip = Velocidade de reprodução ([ / ], Backspace para redefinir)
volume-mute = Silenciar (M)
volume-unmute = Ativar som (M)
volume-level = Volume { $percent }%
repeat-off = Repetir: desligado
repeat-one = Repetir: este vídeo
repeat-all = Repetir: lista inteira
shuffle-on = Aleatório: ligado
shuffle-off = Aleatório: desligado

## Subtitle / audio menu

tracks-tooltip = Legendas / áudio (V / S / A)
tracks-subtitles = Legendas
tracks-off = Desativadas
tracks-unsupported = { $label } (não suportada)
tracks-unsupported-why = Legendas em imagem não são suportadas
tracks-load-subtitle = Carregar arquivo de legenda…
tracks-timing = Sincronização
tracks-delay-later = Atrasar (X)
tracks-delay-earlier = Adiantar (Z)
tracks-delay-reset = Clique para redefinir
tracks-hide-subtitles = Ocultar legendas (V)
tracks-show-subtitles = Mostrar legendas (V)
tracks-audio = Áudio
tracks-no-audio-output = Sem saída de áudio
# $sign is "+" or "−", $value e.g. "0.5"
delay-seconds = { $sign }{ $value } s

## Track languages (from the file's metadata)

lang-ja = Japonês
lang-en = Inglês
lang-zh = Chinês
lang-ko = Coreano
lang-fr = Francês
lang-de = Alemão
lang-es = Espanhol
lang-it = Italiano
lang-ru = Russo
lang-pt = Português

## Playlist panel

playlist-title = Lista de reprodução
playlist-close = Fechar (P)
playlist-save = Salvar lista (Ctrl+S)
playlist-add = Adicionar arquivos
playlist-empty-hint = Solte arquivos aqui para adicioná-los

## Empty window / drag and drop

empty-drop = Solte um vídeo para reproduzi-lo
empty-open-file = Abrir arquivo
empty-open-folder = Abrir pasta
empty-hint = Pressione O para abrir um arquivo, Shift+O para abrir uma pasta
drop-append = Solte para adicionar à lista
drop-play = Solte para reproduzir

## File dialogs

dialog-open-video = Abrir vídeo
dialog-open-folder = Abrir pasta
dialog-add-to-playlist = Adicionar à lista de reprodução
dialog-save-playlist = Salvar lista de reprodução
dialog-load-subtitle = Carregar arquivo de legenda
dialog-filter-media = Vídeos e listas de reprodução
dialog-filter-playlist = Listas de reprodução
dialog-filter-subtitles = Legendas
dialog-filter-all = Todos os arquivos

## On-screen messages

osd-play = Reproduzir
osd-pause = Pausa
# $sign is "+" or "−", $value is a number of seconds
osd-seek = { $sign }{ $value } s
osd-muted = Sem som
osd-speed = Velocidade { $speed }
osd-language = Idioma: { $name }
osd-added =
    { $count ->
        [one] 1 item adicionado à lista
       *[other] { $count } itens adicionados à lista
    }
osd-no-playable = Nenhum vídeo reproduzível encontrado
osd-resumed = Retomado da última posição ({ $time })
osd-playlist-end = Fim da lista de reprodução
osd-playlist-empty = A lista de reprodução está vazia
osd-playlist-saved = Lista salva: { $name }
osd-playlist-save-failed = Não foi possível salvar a lista: { $error }
osd-open-video-first = Abra um vídeo primeiro
osd-subtitle-loaded =
    { $count ->
        [one] Legendas: { $name } (1 entrada)
       *[other] Legendas: { $name } ({ $count } entradas)
    }
osd-audio-track = Áudio: { $label }
osd-subtitle-track = Legendas: { $label }
osd-subtitles-shown = Legendas: exibidas
osd-subtitles-hidden = Legendas: ocultas
osd-subtitle-delay = Sincronização das legendas: { $delay }
osd-hwdec-auto = Decodificação por hardware: automática
osd-hwdec-off = Decodificação por hardware: desligada
osd-single-audio-track = Há apenas uma faixa de áudio
osd-media-info =
    Vídeo: { $codec } { $width }×{ $height } · Decodificador: { $decoder }
    Áudio: { $audio }
media-info-software = software
media-info-none = nenhum

## Errors

error-audio-device = Não foi possível abrir a saída de áudio: { $error }
error-no-audio-device = Nenhum dispositivo de saída de áudio
error-audio-config = Não foi possível obter a configuração da saída de áudio
error-sample-format = Formato de amostra não suportado: { $format }
error-audio-start = Não foi possível iniciar a saída de áudio
error-open-file = Não foi possível abrir: { $path }
error-no-video-stream = Nenhum fluxo de vídeo encontrado
error-video-decoder = Não foi possível abrir o decodificador de vídeo
error-audio-decoder = Não foi possível abrir o decodificador de áudio
error-decoder-open = Não foi possível abrir o decodificador: { $error }
error-audio-playback = Não foi possível reproduzir o áudio: { $error }
error-audio-track-open = Não foi possível abrir a faixa de áudio: { $error }
error-read = Erro de leitura: { $error }
error-subtitle-load = Não foi possível carregar as legendas: { $path }
error-subtitle-empty = Nenhuma legenda encontrada: { $path }
error-hw-fallback = A decodificação por hardware falhou; alternado para decodificação por software

## About dialog

about-version = Versão { $version }
about-close = Fechar (Esc)
about-tab-license = Licença
about-tab-software = Software de terceiros
about-license-line = { $copyright }   Licença: { $spdx }
about-notice =
    rvp é software livre: você pode redistribuí-lo e/ou modificá-lo sob os termos da Licença Pública Geral GNU (GPL), conforme publicada pela Free Software Foundation, na versão 3 da Licença ou (a seu critério) qualquer versão posterior.
    rvp é distribuído na esperança de que seja útil, mas SEM QUALQUER GARANTIA; sem mesmo a garantia implícita de COMERCIALIZAÇÃO ou ADEQUAÇÃO A UM PROPÓSITO PARTICULAR. Veja o texto completo da GPL abaixo para mais detalhes.
about-filter-hint = Filtrar por nome ou licença
about-count = { $shown } / { $total }
about-click-hint = Clique em uma linha para ver o texto completo da licença.
about-col-name = Nome
about-col-version = Versão
about-col-license = Licença
about-col-kind = Tipo
about-back = Voltar à lista
about-build-config = Configuração da compilação
about-no-text = Sem texto da licença (não incluído no rvp).
about-kind-library = Biblioteca
about-kind-font = Fonte
about-kind-crate = Crate
about-version-system = Do sistema
about-version-bundled = Incluído
about-mit-standard = MIT (texto padrão)
about-note-ffmpeg =
    Decodificação de vídeo/áudio, reamostragem e mudança de velocidade. Vinculado dinamicamente às bibliotecas compartilhadas do sistema (libavcodec etc.).
    Licença (conforme informada pela biblioteca): { $license }
about-note-alsa = Saída de áudio (via cpal). Vinculado dinamicamente à biblioteca compartilhada do sistema.
about-note-vulkan = Renderização (via wgpu). Carregado em tempo de execução.
about-note-wayland = Janelas e arrastar e soltar no Wayland. Usa a biblioteca compartilhada do sistema.
about-note-xkb = Entrada de teclado (via winit). Carregado em tempo de execução.
about-note-phosphor = Fonte de ícones, incluída no crate egui-phosphor.
about-font-name = Fontes da interface (do sistema)
about-font-license = Depende da fonte
about-font-note =
    São carregadas as fontes do sistema escolhidas com o fontconfig (fc-match); elas não são incluídas no rvp.
    Em uso: { $paths }
    Cada fonte tem sua própria licença (por exemplo, Noto Sans CJK é OFL-1.1).
about-font-missing = O fontconfig não encontrou nenhuma fonte para a interface.
