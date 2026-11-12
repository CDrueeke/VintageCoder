; sprite_demo.asm – ein mit dem Sprite-Editor gezeichnetes Multicolor-Sprite auf dem C64 anzeigen
;
; Zweck: zeigt, wie ein exportiertes Sprite (graphics/hero.vcsprite) in ein Programm kommt und wie man Sprite 0 des VIC-II einschaltet,
;   in den Multicolor-Modus versetzt, positioniert und einfärbt.
;
; Vorbereitung und Bedienung: `vcc build examples/c64/sprite_demo` exportiert das Sprite selbst (oder im Editor als ASM exportieren,
;   bzw. `vcc media export graphics/hero.vcsprite`) und erzeugt build/export/hero.asm mit dem Label `hero` und den Konstanten
;   hero_len (128 Bytes = 2 Frames je 64 Bytes), hero_sprite (7), hero_mc1 (2), hero_mc2 (6) und hero_count (2). PRG laden, RUN: Sprite 0
;   erscheint als kleine Figur bei Bildschirmposition X = 100, Y = 100 (Koordinaten der Sprite-Register, nicht des Textrasters). Das
;   Programm bleibt in einer Endlosschleife (SEI ohne CLI); beenden mit RESET. Das Sprite hat zwei Frames; angezeigt wird nur Frame 0,
;   der Zeiger bleibt unverändert (ein Wechsel auf $81 würde Frame 1 zeigen; hier sind beide Frames gleich).
;
; Hardware (VIC-II, Register als vic2.* aus den Zieldaten): Die Sprite-Zeiger stehen hinter den 1000 Zeichen des Bildschirmspeichers,
;   der Zeiger von Sprite 0 also bei $0400 + $03F8 = $07F8. Er enthält die Sprite-Adresse geteilt durch 64, bezogen auf die VIC-Bank 0
;   ($0000-$3FFF).
;   $D015 spena (Sprites an), $D01C spmc (Sprite Multicolor), $D000/$D001 sp0x/sp0y (Position), $D027 sp0col (Sprite-Farbe),
;   $D025/$D026 spmc0/spmc1 (die beiden gemeinsamen Multicolor-Farben aller Sprites).
;   $D010 msigx (X-Bit 8) setzt das Programm nicht (nach dem Einschalten 0).
; Farben (C64-Palette): Sprite 7 = Gelb, Multicolor 0 = 2 (Rot), Multicolor 1 = 6 (Blau) – dieselben Werte wie hero_sprite, hero_mc1 und
;   hero_mc2 im Export und im Sprite-Editor (dort heißen die gemeinsamen Farben mc1/mc2, im VIC-II spmc0/spmc1).
;
; Speicherplan: $0801 BASIC-Zeile `10 SYS 2061` und Programm (40 Bytes, Ende bei $0835), Sprite-Daten ab $2000 (128 Bytes, Frame 0 bei $2000,
;   Frame 1 bei $2040). Die Lücke zwischen Programmende und $2000 füllt der Assembler im PRG mit Nullbytes: Das PRG ist deshalb rund 6 KB
;   groß. Zeropage: unbenutzt.
; Grenzen: setzt den Standard-Bildschirmspeicher $0400 voraus (Sprite-Zeiger = Bildschirmspeicher + $3F8) und die VIC-Bank 0.
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

start:  sei
        lda #$80                ; Sprite-Zeiger 0: Daten bei $2000 ($2000 / 64 = $80)
        sta $07f8
        lda #1
        sta vic2.spena          ; Sprite 0 an (Bit 0 = Sprite 0)
        sta vic2.spmc           ; Multicolor (Bit 0 = Sprite 0)
        lda #100
        sta vic2.sp0x           ; X = 100 und Y = 100 (Sprite-Koordinaten; X-Bit 8 in $D010 ist 0)
        sta vic2.sp0y
        lda #7
        sta vic2.sp0col         ; Einzelfarbe des Sprites (Gelb)
        lda #2
        sta vic2.spmc0          ; gemeinsame Multicolor-Farbe 0 (Rot)
        lda #6
        sta vic2.spmc1          ; gemeinsame Multicolor-Farbe 1 (Blau)
loop:   jmp loop

.org $2000                      ; Sprite-Daten auf 64-Byte-Raster: Zeigerwert $80 = $2000 / 64
.include "build/export/hero.asm"
