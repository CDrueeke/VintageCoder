#!/bin/zsh
# Ausschnitte und Montagen aus Rohaufnahmen (nach den Aufnahmeskripten ausführen). crop ID QUELLE BREITE HÖHE Y X (Pixel der Rohaufnahme)
D=${0:A:h}; R=${HANDBUCH_ROH:-$D/../bilder/roh}
crop(){ sips -c $4 $3 --cropOffset $5 $6 $R/$2.png --out $R/$1.png >/dev/null; }
crop bauen-knopf editor-syntax 1900 80 70 1
crop ui-reiter editor-vervollstaendigung 1700 60 145 600
crop projektbaum editor-syntax 560 780 140 80
crop ui-modus-code editor-syntax 2200 1570 140 600
crop ui-modus-debug debug-ueberblick 2200 1570 140 600
VC_THEME=light $D/shot.sh $R/_hell.png c64/rasterbars rasterbars.asm "" 60
crop _d editor-syntax 1500 700 0 1; crop _h _hell 1500 700 0 1
swift $D/split.swift $R/_d.png $R/_h.png $R/theme-hell-dunkel.png; rm -f $R/_*.png
