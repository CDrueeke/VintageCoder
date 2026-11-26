#!/bin/bash
# build_regel0.sh - baut build/regel0.xex (-D ENEMY_DESTROYS_LINES=0) fuer troxx_regel0.vctest.  Aufruf im Projektordner: tools/build_regel0.sh [Pfad zu vcc] [Datenordner]
VCC=${1:-vcc}
DATA=${2:-}
mkdir -p build
$VCC asm main.asm --target atari8 --cpu sally ${DATA:+--data-dir "$DATA"} -o build/regel0.xex --debug build/regel0.vcdbg -D ENEMY_DESTROYS_LINES=0
