#!/bin/zsh
# Gruppe 5: Dialoge und Ansichten, die vorher nur per Klick erreichbar waren (Demo-Werte dlgexport, split, disknew mit disk).
D=${0:A:h}; O=${HANDBUCH_ROH:-$D/../bilder/roh}; S=$D/shot.sh; V=${VC_VCC:-$HOME/.cache/vintagecoder-target-bilder/debug/vcc}; R=$D/../../..
export VC_DATA=/tmp/demo/VintageCoder/data
VC_IMAGE=$R/examples/c64/prallball/build/disk.d64 $S $O/disk-neu.png c64/prallball "" disk,disknew 150
VC_ZOOM=0.75 VC_WIN=2000x1300 VC_STUDIO=graphics/hero.vcsprite $S $O/grafik-exportieren.png c64/sprite_demo "" embed,dlgexport 300 e
VC_SPLIT=lib/vic2.inc $S $O/editor-zweiter-bereich.png c64/rasterbars rasterbars.asm split 120
# Makro mit Listing im zweiten Bereich
cd /tmp/demo/Retro && rm -rf makro anim && HOME=/tmp/demo/home $V new makro --target c64 --template asm-hello >/dev/null 2>&1
cat > makro/src/c64/main.asm <<'ASM'
; Ein Makro setzt Rahmen- und Hintergrundfarbe
.target c64
.basicstub start

.macro farbe(c)
        lda #c
        sta $d020
        sta $d021
.endmacro

start:  farbe(0)
        farbe(6)
        rts
ASM
VC_SPLIT=build/main.lst VC_SPLIT_AT=300 $S $O/asm-makro.png /tmp/demo/Retro/makro src/c64/main.asm projbuild,split 360
# Sprite mit Animation (Fixture: sprite_demo mit angelegter Animation)
cp -R $R/examples/c64/sprite_demo anim
python3 - <<'PY'
import json
p="/tmp/demo/Retro/anim/graphics/hero.vcsprite"; d=json.load(open(p))
d["animations"]=[{"name":"anim","mode":"pingpong","steps":[{"frame":0,"delay":4},{"frame":1,"delay":4},{"frame":0,"delay":2}]}]
json.dump(d,open(p,"w"),indent=1)
PY
VC_ZOOM=0.75 VC_WIN=2000x1300 VC_STUDIO=graphics/hero.vcsprite $S $O/sprite-animation.png /tmp/demo/Retro/anim "" embed 220 e
