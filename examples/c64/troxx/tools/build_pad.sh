#!/bin/bash
# build_pad.sh - baut build/pad.prg (mit Huellen fuer die kurzen Testroutinen, siehe testpad.inc) samt Debug-Information fuer troxx_kurz.vctest.
# Aufruf im Projektordner: tools/build_pad.sh [Pfad zu vcc] [Datenordner]
VCC=${1:-vcc}
DATA=${2:-}
mkdir -p build
$VCC asm main.asm --target c64 --cpu 6510 --illegals deny ${DATA:+--data-dir "$DATA"} -o build/pad.prg --debug build/pad.vcdbg -D TESTPAD=1
