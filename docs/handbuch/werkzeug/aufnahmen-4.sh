#!/bin/zsh
# Gruppe 4: Einstieg-Beispiele (Kapitel 02, 05-08), Systeme (19). Hilfsprojekte entstehen unter /tmp/demo/Retro (Code aus den Kapiteln).
D=${0:A:h}; O=${HANDBUCH_ROH:-$D/../bilder/roh}; S=$D/shot.sh; V=${VC_VCC:-$HOME/.cache/vintagecoder-target-bilder/debug/vcc}; R=$D/../../..
export VC_DATA=/tmp/demo/VintageCoder/data
cd /tmp/demo/Retro && rm -rf hallo fehler farbe zaehler bild-a bild-b
HOME=/tmp/demo/home $V new hallo --target c64 --template asm-hello >/dev/null 2>&1
$S $O/projekt-neu-ergebnis.png /tmp/demo/Retro/hallo src/c64/main.asm "" 60
cat > hallo/src/c64/main.asm <<'ASM'
; Mein erstes Programm: roter Rahmen und ein Gruß auf dem Bildschirm
.target c64
.basicstub start            ; BASIC-Zeile "10 SYS 2061", damit RUN das Programm startet

start:  lda #2              ; 2 = Farbe Rot
        sta $d020           ; Rahmenfarbe setzen
        ldx #0
loop:   lda text,x          ; ein Zeichen holen ...
        sta $0400,x         ; ... und in den Bildschirmspeicher schreiben
        inx
        cpx #5              ; fünf Zeichen kopiert?
        bne loop            ; nein: weiter
        rts                 ; zurück zu BASIC

text:   .screen "hallo"     ; Text als Bildschirmcodes
ASM
$S $O/asm-erstes-programm.png /tmp/demo/Retro/hallo src/c64/main.asm "" 60
VC_WAIT=150 $S $O/ausgabe-panel.png /tmp/demo/Retro/hallo src/c64/main.asm runit,showout 450
cp -R hallo fehler; cat > fehler/src/c64/main.asm <<'ASM'
.target c64
.basicstub start
start:  lda #2
        sta $d020
        jmp ende
ASM
$S $O/asm-fehler.png /tmp/demo/Retro/fehler src/c64/main.asm build,showprob 150
cp -R hallo farbe; cat > farbe/src/c64/main.asm <<'ASM'
; Rahmen und Hintergrund färben
.target c64
.basicstub start

.const FARBE = 6            ; Konstante: 6 = Blau

start:  lda #FARBE
        sta vic2.bordr      ; Tippfehler: richtig wäre vic2.border
        sta $d021           ; dieselbe Wirkung, aber mit Adresse
        rts
ASM
$S $O/editor-fehlerzeile.png /tmp/demo/Retro/farbe src/c64/main.asm "" 60
HOME=/tmp/demo/home $V new zaehler --target c64 --template basic-hello >/dev/null 2>&1
mv zaehler/src/c64/main.bas zaehler/src/c64/zaehler.bas; sed -i '' 's#src/c64/main.bas#src/c64/zaehler.bas#' zaehler/zaehler.vcprj
cat > zaehler/src/c64/zaehler.bas <<'BAS'
; Mein erstes Programm
print "{clr}hallo welt"
summe = 0
for i = 1 to 10
  summe = summe + i
next i
print "summe ="; summe
@ende:
goto @ende
BAS
$S $O/basic-marken.png /tmp/demo/Retro/zaehler src/c64/zaehler.bas build 150
VC_WAIT=240 $S $O/basic-langenamen.png c64/basic_langenamen punkte.bas build 300
# Willkommensseite mit Projektbildern (Projektbild entsteht beim Bauen in der IDE)
cp -R $R/examples/c64/rasterbars bild-a; cp -R $R/examples/c64/music_demo bild-b
VC_KEEP_IMG=1 $S $O/_b1.png /tmp/demo/Retro/bild-a rasterbars.asm build 200
VC_KEEP_IMG=1 $S $O/_b2.png /tmp/demo/Retro/bild-b music_demo.asm build 200; rm -f $O/_b?.png
python3 - <<PY
import json,time; n=int(time.time())
json.dump([{"path":"/tmp/demo/Retro/bild-a","name":"C64: rasterbars","pinned":False,"last":n-3600},{"path":"/tmp/demo/Retro/bild-b","name":"C64: music_demo","pinned":False,"last":n-86400}],open("/tmp/demo/recent_bild.json","w"))
PY
VC_RECENT=/tmp/demo/recent_bild.json VC_KEEP_IMG=1 $S $O/projektbild-willkommen.png - "" "" 90
# Systeme (Kapitel 19)
VC_BP_LINE=11 VC_DBG_TAB=emulator $S $O/system-vergleich-c64.png c64/border border.asm bp,debug,dbgtab 960
$S $O/system-vergleich-atari.png atari/colorbars colorbars.asm build 450
$S $O/system-vergleich-mega65.png mega65/hello hello.asm build 450
# Einstellungen: Hardware-Bereich (nichts eingetragen)
VC_SET_SCROLL=900 $S $O/ultimate-start.png c64/rasterbars rasterbars.asm settings 60
# Dialoge (Selbsttest-Werte dlgstd, dlgsheet, dlgb2a)
cp $O/editor-syntax.png /tmp/demo/raster.png
$S $O/zeichensatz-standard.png c64/standard_charsets "" dlgstd 60
VC_SHEET=/tmp/demo/raster.png $S $O/zeichensatz-png-import.png c64/rasterbars rasterbars.asm dlgsheet 60
$S $O/basic-nach-asm.png /tmp/demo/Retro/zaehler src/c64/zaehler.bas dlgb2a 60
