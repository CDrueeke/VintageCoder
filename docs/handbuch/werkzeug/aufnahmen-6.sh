#!/bin/zsh
# Gruppe 6: Tracker-Dialoge (Dateiwahl per VC_PICK ersetzt), Umbenennen mit „Code anpassen“, Vorschau-Reiter.
D=${0:A:h}; O=${HANDBUCH_ROH:-$D/../bilder/roh}; S=$D/shot.sh; R=$D/../../..
export VC_DATA=/tmp/demo/VintageCoder/data
cp $R/examples/c64/music_demo/music/tune.vcsong /tmp/demo/tune.vcsong
head -c 4096 /dev/zero > /tmp/demo/demo.rmt   # Platzhalterdatei: der Dialog zeigt nur den Namen, geladen wird erst nach „Importieren“
Z="VC_ZOOM=0.75 VC_WIN=2000x1300"
env VC_ZOOM=0.75 VC_WIN=2000x1300 VC_STUDIO=music/atune.vcsong VC_TRK_DLG=convert VC_PICK=/tmp/demo/tune.vcsong $S $O/tracker-sid-pokey.png atari/music_demo "" embed 260 e
env VC_ZOOM=0.75 VC_WIN=2000x1300 VC_STUDIO=music/atune.vcsong VC_TRK_DLG=rmt VC_PICK=/tmp/demo/demo.rmt $S $O/tracker-import-rmt.png atari/music_demo "" embed 260 e
VC_RENAME_FROM=lib/vic2.inc VC_RENAME_TO=vic2_neu.inc $S $O/datei-umbenennen-dialog.png c64/rasterbars rasterbars.asm dlgren 60
VC_PREVIEW=lib/vic2.inc $S $O/ui-reiter-vorschau.png c64/rasterbars rasterbars.asm previewtab 100
