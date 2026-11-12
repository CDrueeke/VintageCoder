#!/bin/zsh
# Ausschnitte der Studio-Bilder (nach aufnahmen-3.sh, Rohbilder 3000x1726). crop ID QUELLE BREITE HÖHE Y X
D=${0:A:h}; R=${HANDBUCH_ROH:-$D/../bilder/roh}
crop(){ sips -c $4 $3 --cropOffset $5 $6 $R/$2.png --out $R/$1.png >/dev/null; }
crop sprite-c64-multicolor sprite-c64 388 794 416 2144
crop tracker-muster tracker-ueberblick 626 309 450 768
crop tracker-instrument tracker-ueberblick 406 750 450 2126
crop tracker-huellkurve tracker-ueberblick 406 150 1055 2126
crop kachel-bank kachel-editor 503 529 803 2035
crop palette-atari sprite-atari-pmg 430 880 430 2100
cp $R/einstellungen-roms.png $R/rom-verzeichnis.png
swift $D/kuerzel-karte.swift $R/kuerzel-karte.png
sips -c 160 1560 --cropOffset 1250 620 $R/system-vergleich-c64.png --out $R/emulator-bild-projektbild.png >/dev/null   # Etappe 4: Emulator-Knopfleiste
cp $R/asm-fehler.png $R/probleme-panel.png   # gleiche Aufnahme
