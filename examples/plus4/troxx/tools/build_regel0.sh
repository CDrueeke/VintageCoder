#!/bin/bash
# build_regel0.sh - baut build0/main.prg (mit -D ENEMY_DESTROYS_LINES=0, alte Regel) samt Debug-Information fuer troxx_regel0.vctest.
# Aufruf im Projektordner: tools/build_regel0.sh [Pfad zu vcc] [Datenordner]
VCC=${1:-vcc}
DATA=${2:-}
mkdir -p build0
$VCC asm main.asm --target plus4 --cpu 7501 --illegals deny ${DATA:+--data-dir "$DATA"} -o build0/main.prg --debug build0/main.vcdbg -D ENEMY_DESTROYS_LINES=0
