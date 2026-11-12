; rasterbars.asm – Rasterbalken auf dem C64: die Rahmenfarbe wechselt von Rasterzeile zu Rasterzeile
;
; Zweck: zeigt Include (lib/vic2.inc), Makro (wait_line), Wiederholung (.repeat) beim Erzeugen einer Tabelle und bedingte Assemblierung
;   (.if DEBUG). Die Balken entstehen ohne Interrupt: das Programm wartet aktiv auf Rasterzeilen und schreibt die Rahmenfarbe $D020
;   zeilenweise, so dass jede Rasterzeile eine andere Farbe bekommt.
;
; Bedienung: `vcc build examples/c64/rasterbars` (die Projektdatei setzt DEBUG = 1), PRG laden, RUN. Mit `vcc asm … -D DEBUG=0`
;   bzw. `-D DEBUG=1` wählt man die Fassung selbst; ohne Angabe meldet der Assembler E202 (unbekanntes Symbol DEBUG).
;   Beenden nur mit RESET (SEI ohne CLI, Endlosschleife).
;
; Ablauf je Bild (PAL: 50 mal je Sekunde):
;   1. wait_line $30: warten, bis die Rasterzeile $30 (48) erreicht ist.
;   2. phase um eins erhöhen (Animation: die Balkenfarben wandern von Bild zu Bild).
;   3. 16 Balken (BARS): je Balken auf die nächste Rasterzeile warten (Y merkt sich die alte Zeile, .sync wartet auf eine andere),
;      die Farbe `colors[balkenindex + phase]` in $D020 schreiben und weiterzählen. Die Balken liegen auf den Zeilen $31 bis $40.
;   4. Zurück zu 1. Die Rahmenfarbe des letzten Balkens bleibt bis zum nächsten Bild stehen.
;   Mit DEBUG = 1 wird dieselbe Farbe auch in die Hintergrundfarbe $D021 geschrieben, die Balken sind dann auch im Bildschirmfenster
;   sichtbar (ohne DEBUG nur dort, wo der Rahmen sichtbar ist).
;   Zeitbedarf: Jeder Balken ist eine Rasterzeile hoch. Das gelingt nur, weil die Schleife je Balken kürzer ist als eine Zeile
;   (PAL 63 Takte; Schleife etwa 35 Takte, mit DEBUG etwa 39). Badlines (Zeilen, in denen der VIC-II der CPU Takte stiehlt) können
;   die Farbwechsel vermutlich um einige Takte verschieben und einzelne Balken sichtbar verzittern lassen.
;
; Hardware: VIC-II $D012 (Rasterzeile lesen), $D020 (Rahmenfarbe), $D021 (Hintergrundfarbe 0).
; Speicherplan: $0801 BASIC-Zeile `10 SYS 2061`, Programm ab $080D (rund 50 Bytes Code), direkt dahinter die Tabelle `colors`
;   (256 Bytes). Zeropage: $FB = phase (Animationsphase, frei benutzbare Zeropage-Zelle des C64).
; Tabelle colors: 256 Einträge (i * 3 + 2) & 15, d. h. alle 16 C64-Farben in einer Reihenfolge mit Schritt 3; benachbarte Balken haben
;   deshalb deutlich verschiedene Farben. Der Index balkenindex + phase läuft im Akkumulator (8 Bit) über und passt genau auf 256 Einträge.
; Grenzen: kein sauberes Ende (nur RESET); aktive Warteschleifen ohne Zyklenzählung; die genannten Zeilenzeiten gelten für PAL.
.target c64
.cpu 6510 { illegals = deny }
.include "lib/vic2.inc"             ; vic_raster, vic_border, vic_bg0 und das Makro wait_line

.const BARS = 16                ; Anzahl der Balken = Anzahl der Rasterzeilen, die je Bild gefärbt werden
.const phase = $fb              ; Zeropage: Animationsphase

.org $0801
.basicstub start

start:  sei                     ; kein IRQ: der KERNAL würde die Zeilenzeiten stören
        lda #0
        sta vic_bg0             ; Hintergrund schwarz
        sta phase               ; Phase 0
frame:  wait_line $30           ; je Bild einmal: Start kurz vor dem sichtbaren Bereich
        inc phase
        ldx #0                  ; X = Balkenindex 0..BARS-1
.bar:   ldy vic_raster
.sync:  cpy vic_raster          ; auf die nächste Rasterzeile warten
        beq .sync
        txa
        clc
        adc phase               ; Farbe = Balkenindex + Phase
        tay
        lda colors,y
        sta vic_border
.if DEBUG
        sta vic_bg0             ; im Debug-Build auch den Hintergrund färben
.endif
        inx
        cpx #BARS
        bne .bar
        jmp frame

; 256 Farbwerte; .repeat zählt i von 0 bis 255 und erzeugt je ein .byte
colors: .repeat 256, i
        .byte (i * 3 + 2) & 15
.endrepeat
