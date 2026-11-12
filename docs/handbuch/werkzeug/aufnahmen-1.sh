#!/bin/zsh
# Gruppe 1 (Kapitel 01-05): Rohaufnahmen nach bilder/roh/. Nur eine IDE-Instanz gleichzeitig (shot.sh läuft nacheinander).
D=${0:A:h}; O=${HANDBUCH_ROH:-$D/../bilder/roh}; S=$D/shot.sh
$S $O/ui-gesamt.png c64/music_demo music_demo.asm "" 60
$S $O/willkommen-seite.png - "" "" 60
$S $O/editor-syntax.png c64/rasterbars rasterbars.asm "" 60
$S $O/editor-vervollstaendigung.png c64/rasterbars rasterbars.asm complete 60
$S $O/editor-falten.png c64/rasterbars rasterbars.asm fold 60
$S $O/editor-kontextmenue.png c64/rasterbars rasterbars.asm ctxmenu 60
$S $O/editor-zeichenwaehler.png c64/rasterbars rasterbars.asm charpick 60
$S $O/editor-hilfe-f1.png c64/rasterbars rasterbars.asm showhelp 60
$S $O/editor-suchen.png c64/rasterbars rasterbars.asm sidesearch 60
$S $O/bauprotokoll.png c64/rasterbars rasterbars.asm build 120
$S $O/befehlspalette.png c64/rasterbars rasterbars.asm palette 60
VC_SET_PAGE=general $S $O/einstellungen.png c64/rasterbars rasterbars.asm settings 60
$S $O/projekt-einstellungen.png c64/rasterbars rasterbars.asm projed 60
$S $O/memory-map.png c64/rasterbars rasterbars.asm showmm 120
$S $O/debug-ueberblick.png c64/rasterbars rasterbars.asm debug,snap 925
$D/aufnahmen-1b.sh; $D/ausschnitte.sh
