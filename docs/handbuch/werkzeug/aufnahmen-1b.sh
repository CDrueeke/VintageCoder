#!/bin/zsh
# Gruppe 1, Teil 2: Wizard, Einstellungen, Dialoge, Zonen. Rohaufnahmen nach bilder/roh/.
D=${0:A:h}; O=${HANDBUCH_ROH:-$D/../bilder/roh}; S=$D/shot.sh; P=c64/rasterbars; F=rasterbars.asm
$S $O/wizard-system.png $P $F dlgp 60
$S $O/wizard-sprache.png $P $F dlgp,wizvic 90
VC_WIN=1500x1150 $S $O/wizard-details.png $P $F dlgp,wizbasic 90
$S $O/einstellungen-emulator.png $P $F settings 60
VC_SET_PAGE=roms $S $O/einstellungen-roms.png $P $F settings 60
VC_SET_ARGS=1 $S $O/argumente.png $P $F settings 60
$S $O/ui-zonen.png $P $F zones 60
$S $O/datei-neu-dialog.png $P $F dlgnew 60
VC_RENAME_FROM=lib/vic2.inc VC_RENAME_TO=lib/vic2_neu.inc $S $O/datei-umbenennen-dialog.png $P $F dlgren 60
VC_DROP=/tmp/demo/drop/font.bin:/tmp/demo/drop/notizen.txt $S $O/drag-drop-rueckfrage.png $P $F dropfiles 60
$S $O/projekt-uebersicht.png $P $F projbuild,showov 150
$S $O/roms-einrichten.png $P $F romwiz2 60
$S $O/editor-ersetzen.png $P $F findrep 60
$S $O/editor-gehe-zu.png $P $F goto 60
$S $O/editor-referenz.png $P $F ref,showref 60
