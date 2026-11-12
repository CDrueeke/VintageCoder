#!/bin/zsh
# Ausschnitte der Debugger-Bilder (nach aufnahmen-2.sh). crop ID QUELLE BREITE HÖHE Y X
D=${0:A:h}; R=${HANDBUCH_ROH:-$D/../bilder/roh}
crop(){ sips -c $4 $3 --cropOffset $5 $6 $R/$2.png --out $R/$1.png >/dev/null; }
crop debug-haltepunkt-gutter debug-ueberblick 800 160 740 620
crop debug-pc-zeile debug-ueberblick 800 160 740 620
crop debug-register debug-ueberblick 595 600 150 2205
crop debug-disassembler debug-ueberblick 1580 440 1230 620
