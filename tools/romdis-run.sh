#!/bin/bash
# Erzeugt die Listings in ausgabe/ aus deinen eigenen ROMs (nicht im Repository; ROM-Ordner siehe `vcc roms where`).
# Vorher: KLASEK_HEADS=<Datei> python3 tools/romnames.py  (optional: zusätzliche lokale Namen *.local.names) (Namensdateien tools/names/*.names)
cd "$(dirname "$0")/.."
D=ausgabe
# ROM-Dateien aus dem ROM-Verzeichnis von VintageCoder (Benutzerordner, VC_ROMS, auf Wunsch andere Programme): `vcc roms path <system/rolle>`; vcc per VCC=… wählbar
rp() { "${VCC:-vcc}" roms path "$1" || exit 1; }
Y="--syms data/sys/c64/basic.asm --syms data/sys/c64/kernal.asm --syms data/sys/c64/hardware.asm"
BS=""; for a in B391 B1AA BBA2 BBD4 BC0C BBFC B850 B853 B867 B86A BA28 BA2B BB0F BB12 BF7B BF78 BF71 BFED B9EA BFB4 BC58 BC39 BCCC BC5B BCF3 BDDD AB1E BDCD; do BS="$BS --seed $a"; done
KS=""; for a in E264 E26B E2B4 E30E E097; do KS="$KS --seed $a"; done
python3 tools/romdis.py $(rp c64/basic) A000 $D/c64-basic-A000.txt --title 'C64 BASIC-ROM 901226-01 ($A000-$BFFF)' $Y --names tools/names/c64-basic.names $(test -f tools/names/c64-basic.local.names && echo --names tools/names/c64-basic.local.names) --words A000:2 --table A00C:35:1 --table A052:23:1 $BS
python3 tools/romdis.py $(rp c64/kernal) E000 $D/c64-kernal-E000.txt --title 'C64 KERNAL-ROM 901227-03 ($E000-$FFFF)' $Y --names tools/names/c64-kernal.names $(test -f tools/names/c64-kernal.local.names && echo --names tools/names/c64-kernal.local.names) --words FFFA:3 --vector FFFA --vector FFFC --vector FFFE --jumptable FF81:38 --table FD30:16:0 $KS
A="--syms data/sys/atari8bit/os.asm --syms data/sys/atari8bit/antic.asm --syms data/sys/atari8bit/gtia.asm --syms data/sys/atari8bit/pokey.asm --syms tools/atari-fp.asm"
FP=""; for a in D800 D8E6 D9AA D9D2 DA60 DA66 DADB DB28 DD89 DD8D DDA7 DDB6 DDC0 DDCC DECD DED1; do FP="$FP --seed $a"; done
python3 tools/romdis.py $(rp atari8/os-a) D800 $D/atari-os-b-D800.txt --title 'Atari OS-B ($D800-$FFFF; Fließkommapaket ab $D800)' $A --names tools/names/atari-os-b.names --words FFFA:3 --vector FFFA --vector FFFC --vector FFFE --jumptable E450:16 $FP
python3 tools/romdis.py $(rp atari8/os-xl) C000 $D/atari-xl-os-C000.txt --title 'Atari XL/XE-OS ($C000-$FFFF; Lage der Abschnitte ungeprüft)' $A --names tools/names/atari-xl.names --words FFFA:3 --vector FFFA --vector FFFC --vector FFFE --jumptable E450:16 $FP
python3 tools/romdis.py $(rp atari8/basic) A000 $D/atari-basic-A000.txt --title 'Atari-BASIC-ROM ($A000-$BFFF)' $A --names tools/names/atari-basic.names --vector BFFA --vector BFFE
