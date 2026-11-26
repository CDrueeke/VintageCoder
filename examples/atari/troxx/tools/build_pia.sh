#!/bin/bash
# build_pia.sh - baut build/pia.xex (mit der Testhilfe pia_read, siehe main.asm) fuer troxx_pia.vctest.
# Aufruf im Projektordner: tools/build_pia.sh [Pfad zu vcc] [Datenordner]
VCC=${1:-vcc}
DATA=${2:-}
mkdir -p build
$VCC asm main.asm --target atari8 --cpu sally ${DATA:+--data-dir "$DATA"} -o build/pia.xex --debug build/pia.vcdbg -D TESTPIA=1
