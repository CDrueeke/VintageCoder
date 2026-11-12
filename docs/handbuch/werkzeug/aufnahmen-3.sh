#!/bin/zsh
# Gruppe 3: Grafik- und Klang-Studios (Kapitel 11-18). Studio per VC_DEMO=embed + VC_STUDIO (Datei im Projekt), VC_EMBED=1 (sechster Parameter).
D=${0:A:h}; O=${HANDBUCH_ROH:-$D/../bilder/roh}; S=$D/shot.sh; V=${VC_VCC:-$HOME/.cache/vintagecoder-target-bilder/debug/vcc}; R=$D/../../..
export VC_DATA=/tmp/demo/VintageCoder/data   # vcc findet die Hardware-Datenbank sonst nicht (neutrales HOME)
st(){ id=$1; proj=$2; file=$3; at=${4:-220}; VC_ZOOM=${VC_STUDIO_ZOOM:-0.75} VC_WIN=${VC_STUDIO_WIN:-2000x1300} VC_STUDIO=$file $S $O/$id.png $proj "" embed $at e; }   # Zoom 0.75 und großes Fenster: ganze Studios sichtbar
# Hilfsprojekte für neue Assets (Kopie unter /tmp/demo/Retro, damit examples/ unverändert bleibt)
rm -rf /tmp/demo/Retro/gfx /tmp/demo/Retro/pal; cp -R $R/examples/c64/gfx_demo /tmp/demo/Retro/gfx; cp -R $R/examples/mega65/pal_demo /tmp/demo/Retro/pal
(cd /tmp/demo/Retro/gfx && HOME=/tmp/demo/home $V media new bunt --profile c64.char.multicolor --dir graphics >/dev/null 2>&1)
(cd /tmp/demo/Retro/pal && HOME=/tmp/demo/home $V media new voll --profile mega65.char.fcm --dir graphics >/dev/null 2>&1)
st zeichensatz-ueberblick c64/gfx_demo graphics/font.vcchar
st zeichensatz-multicolor /tmp/demo/Retro/gfx graphics/bunt.vcchar
st zeichensatz-mega65-farbe /tmp/demo/Retro/pal graphics/voll.vcchar
st sprite-c64 c64/sprite_demo graphics/hero.vcsprite
st sprite-atari-pmg atari/pmg_ship graphics/ship.vcpmg
st bildschirm-editor c64/gfx_demo graphics/title.vcscreen
st kachel-editor c64/gfx_demo graphics/level_tiles.vctiles
st kachel-karte c64/gfx_demo graphics/level.vcscreen
st palette-mega65 mega65/pal_demo graphics/pal65.vcpal
st tracker-ueberblick c64/music_demo music/tune.vcsong 220
VC_TRK_PLAY=1 st tracker-hoerprobe c64/music_demo music/tune.vcsong 400
st tracker-pokey atari/music_demo music/atune.vcsong 220
st play-tracker mega65/play_demo music/tune_play.vcplay 220
# Disk-Studio und Segmente
(cd $R/examples/c64/prallball && HOME=/tmp/demo/home $V build >/dev/null 2>&1)
ls $R/examples/c64/prallball/build/*.d64 >/dev/null 2>&1 && VC_IMAGE=$R/examples/c64/prallball/build/disk.d64 $S $O/disk-studio.png c64/prallball "" disk 120
VC_IMAGE=$R/examples/c64/prallball/build/disk.d64 $S $O/disk-validieren.png c64/prallball "" disk,diskval 150
$S $O/segmente-baenke.png c64/cart_demo "" projbuild,showmm 200
