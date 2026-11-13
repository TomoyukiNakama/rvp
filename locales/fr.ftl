# rvp UI strings — Français
# Format et règles : voir l'en-tête de en.ftl.

language-name = Français

## Title bar

window-exit-fullscreen = Quitter le plein écran (Esc)
window-close = Fermer
window-restore = Restaurer
window-maximize = Agrandir
window-minimize = Réduire
window-about = À propos de rvp / licences (F1)
window-open-folder = Ouvrir un dossier (Shift+O)
window-open-file = Ouvrir un fichier (O)
window-language = Langue (Language)

## Control bar

controls-pause = Pause (Space)
controls-play = Lecture (Space)
controls-replay = Relire depuis le début (Space)
controls-prev = Précédent (B)
controls-next = Suivant (N)
controls-exit-fullscreen = Quitter le plein écran (F)
controls-fullscreen = Plein écran (F)
controls-playlist = Liste de lecture (P)
speed-tooltip = Vitesse de lecture ([ / ], Backspace pour réinitialiser)
volume-mute = Muet (M)
volume-unmute = Réactiver le son (M)
volume-level = Volume { $percent }%
repeat-off = Boucle : désactivée
repeat-one = Boucle : cette vidéo
repeat-all = Boucle : toute la liste
shuffle-on = Aléatoire : activé
shuffle-off = Aléatoire : désactivé

## Subtitle / audio menu

tracks-tooltip = Sous-titres / audio (V / S / A)
tracks-subtitles = Sous-titres
tracks-off = Désactivé
tracks-unsupported = { $label } (non pris en charge)
tracks-unsupported-why = Les sous-titres en image ne sont pas pris en charge
tracks-load-subtitle = Charger un fichier de sous-titres…
tracks-timing = Synchronisation
tracks-delay-later = Retarder (X)
tracks-delay-earlier = Avancer (Z)
tracks-delay-reset = Cliquer pour réinitialiser
tracks-hide-subtitles = Masquer les sous-titres (V)
tracks-show-subtitles = Afficher les sous-titres (V)
tracks-audio = Audio
tracks-no-audio-output = Aucune sortie audio
# $sign is "+" or "−", $value e.g. "0.5"
delay-seconds = { $sign }{ $value } s

## Track languages (from the file's metadata)

lang-ja = Japonais
lang-en = Anglais
lang-zh = Chinois
lang-ko = Coréen
lang-fr = Français
lang-de = Allemand
lang-es = Espagnol
lang-it = Italien
lang-ru = Russe
lang-pt = Portugais

## Playlist panel

playlist-title = Liste de lecture
playlist-close = Fermer (P)
playlist-save = Enregistrer la liste (Ctrl+S)
playlist-add = Ajouter des fichiers
playlist-empty-hint = Déposez des fichiers ici pour les ajouter

## Empty window / drag and drop

empty-drop = Déposez une vidéo pour la lire
empty-open-file = Ouvrir un fichier
empty-open-folder = Ouvrir un dossier
empty-hint = O pour ouvrir un fichier, Shift+O pour un dossier
drop-append = Déposer pour ajouter à la liste
drop-play = Déposer pour lire

## File dialogs

dialog-open-video = Ouvrir une vidéo
dialog-open-folder = Ouvrir un dossier
dialog-add-to-playlist = Ajouter à la liste de lecture
dialog-save-playlist = Enregistrer la liste de lecture
dialog-load-subtitle = Charger un fichier de sous-titres
dialog-filter-media = Vidéos et listes de lecture
dialog-filter-playlist = Listes de lecture
dialog-filter-subtitles = Sous-titres
dialog-filter-all = Tous les fichiers

## On-screen messages

osd-play = Lecture
osd-pause = Pause
# $sign is "+" or "−", $value is a number of seconds
osd-seek = { $sign }{ $value } s
osd-muted = Muet
osd-speed = Vitesse { $speed }
osd-language = Langue : { $name }
osd-added =
    { $count ->
        [one] { $count } élément ajouté à la liste de lecture
       *[other] { $count } éléments ajoutés à la liste de lecture
    }
osd-no-playable = Aucune vidéo lisible trouvée
osd-resumed = Reprise à la dernière position ({ $time })
osd-playlist-end = Fin de la liste de lecture
osd-playlist-empty = La liste de lecture est vide
osd-playlist-saved = Liste enregistrée : { $name }
osd-playlist-save-failed = Impossible d'enregistrer la liste : { $error }
osd-open-video-first = Ouvrez d'abord une vidéo
osd-subtitle-loaded =
    { $count ->
        [one] Sous-titres : { $name } ({ $count } réplique)
       *[other] Sous-titres : { $name } ({ $count } répliques)
    }
osd-audio-track = Audio : { $label }
osd-subtitle-track = Sous-titres : { $label }
osd-subtitles-shown = Sous-titres : affichés
osd-subtitles-hidden = Sous-titres : masqués
osd-subtitle-delay = Synchro des sous-titres : { $delay }
osd-hwdec-auto = Décodage matériel : auto
osd-hwdec-off = Décodage matériel : désactivé
osd-single-audio-track = Il n'y a qu'une seule piste audio
osd-media-info =
    Vidéo : { $codec } { $width }×{ $height } · Décodeur : { $decoder }
    Audio : { $audio }
media-info-software = logiciel
media-info-none = aucun

## Errors

error-audio-device = Impossible d'ouvrir la sortie audio : { $error }
error-no-audio-device = Aucun périphérique de sortie audio
error-audio-config = Impossible d'obtenir la configuration de la sortie audio
error-sample-format = Format d'échantillon non pris en charge : { $format }
error-audio-start = Impossible de démarrer la sortie audio
error-open-file = Ouverture impossible : { $path }
error-no-video-stream = Aucun flux vidéo trouvé
error-video-decoder = Impossible d'ouvrir le décodeur vidéo
error-audio-decoder = Impossible d'ouvrir le décodeur audio
error-decoder-open = Impossible d'ouvrir le décodeur : { $error }
error-audio-playback = Impossible de lire l'audio : { $error }
error-audio-track-open = Impossible d'ouvrir la piste audio : { $error }
error-read = Erreur de lecture : { $error }
error-subtitle-load = Impossible de charger les sous-titres : { $path }
error-subtitle-empty = Aucun sous-titre trouvé : { $path }
error-hw-fallback = Échec du décodage matériel ; passage au décodage logiciel

## About dialog

about-version = Version { $version }
about-close = Fermer (Esc)
about-tab-license = Licence
about-tab-software = Logiciels tiers
about-license-line = { $copyright }   Licence : { $spdx }
about-notice =
    rvp est un logiciel libre ; vous pouvez le redistribuer et/ou le modifier selon les termes de la Licence Publique Générale GNU (GPL) telle que publiée par la Free Software Foundation, soit la version 3 de la licence, soit (à votre gré) toute version ultérieure.
    rvp est distribué dans l'espoir qu'il sera utile, mais SANS AUCUNE GARANTIE ; sans même la garantie implicite de COMMERCIALISABILITÉ ni d'ADÉQUATION À UN OBJECTIF PARTICULIER. Consultez le texte complet de la GPL ci-dessous pour plus de détails.
about-filter-hint = Filtrer par nom ou licence
about-count = { $shown } / { $total }
about-click-hint = Cliquez sur une ligne pour afficher le texte complet de la licence.
about-col-name = Nom
about-col-version = Version
about-col-license = Licence
about-col-kind = Type
about-back = Retour à la liste
about-build-config = Configuration de compilation
about-no-text = Pas de texte de licence (non fourni avec rvp).
about-kind-library = Bibliothèque
about-kind-font = Police
about-kind-crate = Crate
about-version-system = Système
about-version-bundled = Incluse
about-mit-standard = MIT (texte standard)
about-note-ffmpeg =
    Décodage vidéo/audio, rééchantillonnage et changement de vitesse. Liée dynamiquement aux bibliothèques partagées du système (libavcodec, etc.).
    Licence (selon la bibliothèque) : { $license }
about-note-alsa = Sortie audio (via cpal). Liée dynamiquement à la bibliothèque partagée du système.
about-note-vulkan = Rendu (via wgpu). Chargée à l'exécution.
about-note-wayland = Fenêtrage et glisser-déposer sous Wayland. Utilise la bibliothèque partagée du système.
about-note-xkb = Saisie clavier (via winit). Chargée à l'exécution.
about-note-phosphor = Police d'icônes, incluse dans le crate egui-phosphor.
about-font-name = Polices de l'interface (système)
about-font-license = Selon la police
about-font-note =
    Les polices système choisies via fontconfig (fc-match) sont chargées ; elles ne sont pas fournies avec rvp.
    Utilisées : { $paths }
    Chaque police a sa propre licence (par ex. Noto Sans CJK : OFL-1.1).
about-font-missing = fontconfig n'a trouvé aucune police pour l'interface.
