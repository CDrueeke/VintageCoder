#!/bin/bash
# build_regel0.sh - baut build/regel0.prg (ENEMY_DESTROYS_LINES = 0, alte Regel) samt Debug-Information fuer troxx_regel0.vctest.
# Aufruf im Projektordner: tools/build_regel0.sh [Pfad zu vcc] [Datenordner]
VCC=${1:-vcc}
DATA=${2:-}
mkdir -p build
$VCC asm main.asm --target c64 --cpu 6510 --illegals deny ${DATA:+--data-dir "$DATA"} -o build/regel0.prg --debug build/regel0.vcdbg -D ENEMY_DESTROYS_LINES=0
