; =====================================================================================================================
; vic.asm - Bild des VIC-II: Norm erkennen, Register, Zeichensatz, Sprite-Daten, Bildtakt
; =====================================================================================================================
; ZWECK     Alles, was den Video-Chip betrifft und nur einmal oder je Bild geschieht.
; ABLAUF    init: Norm erkennen (PAL/NTSC) -> VIC-Bank 0 -> Zeichen-ROM (Bildschirmcodes 0..127) nach $3000 kopieren -> eigene Grafik
;           (Codes 128..) und Sprite-Daten ($3800) aus dem Programm kopieren -> Register -> Bildschirm leeren.
; HARDWARE  $D011 = $1B (Text, 25 Zeilen, Bit 7 = Rasterzeile Bit 8), $D016 = $18 (40 Spalten, Mehrfarben-Zeichen moeglich), $D018 = $1C
;           (Bildschirm $0400, Zeichensatz $3000), $D020 Rahmen, $D021..$D023 Hintergrundfarben 0..2 (Mehrfarben: Bitpaare 00, 01, 10; Paar 11 =
;           Farbspeicher Bit 0..2).  Farbspeicher-Bit 3 = 1 schaltet ein Zeichen auf Mehrfarben (Fuellungen); Farben 0..7 mit Bit 3 = 0 sind
;           einfarbig (Linien, Text).  Der Zeichen-ROM liegt bei $D000 (nur mit $01 = $33 sichtbar), Bildschirmcode n bei $D000 + 8 n.
;           Rasterzeile $FB (251) liegt unter dem Bild (25 Zeilen enden bei Zeile 250): dort beginnt jedes Bild.
; =====================================================================================================================

vic: {

.const RASTER_FRAME = 251               ; Rasterzeile unter dem Bild, an der jedes Bild beginnt
.const BORDER_COL   = COL_LIGHTBLUE        ; Rahmen: hellblau
.const BG_COL0      = COL_BLUE          ; Hintergrund 0: blau (Spielfeld)
.const BG_COL1      = COL_DARKGRAY        ; Mehrfarben-Hintergrund 1: schwarz (Fuellmuster, Schatten)
.const BG_COL2      = COL_LIGHTBLUE        ; Mehrfarben-Hintergrund 2: dunkelgrau (Fugen)

; init: Veraendert A, X, Y.
init:
        lda #BORDER_COL
        sta VIC_BORDERCOLOR
        lda #BG_COL0
        sta VIC_BGCOLOR0
        lda #0
        sta VIC_SPR_ENABLE              ; Sprites aus
        lda #$7f                        ; Sprites 0..6 mehrfarbig: Umriss schwarz, Glanz weiss, Koerperfarbe je Sprite
        sta VIC_SPR_MULTICOLOR
        lda #COL_BLACK
        sta VIC_SPR_MCOLOR0
        lda #COL_WHITE
        sta VIC_SPR_MCOLOR1
        lda #0
        sta VIC_SPR_EXPAND_X
        sta VIC_SPR_EXPAND_Y
        sta VIC_SPR_PRIORITY
        jsr detect_norm
        lda FLAGS
        and #F_PAL
        beq .ntsc
        lda #50
        bne .fps
.ntsc:  lda #60
.fps:   sta FPS
        lda CIA2_DDRA                   ; VIC-Bank 0 ($0000-$3FFF): Bits 0 und 1 von $DD00 = 1
        ora #3
        sta CIA2_DDRA
        lda CIA2_PRA
        ora #3
        sta CIA2_PRA
        lda #$33                        ; Zeichen-ROM sichtbar machen ($01: LORAM, HIRAM, CHAREN = 0)
        sta CPU_PORT
        ldx #0
.rom:   lda $d000,x                     ; Bildschirmcodes 0..127: 1024 Byte
        sta CHARSET,x
        lda $d100,x
        sta CHARSET + $100,x
        lda $d200,x
        sta CHARSET + $200,x
        lda $d300,x
        sta CHARSET + $300,x
        inx
        bne .rom
        lda #$37                        ; Normalbelegung (BASIC, Ein-/Ausgabe, KERNAL)
        sta CPU_PORT
        ldx #NGFX * 8 - 1
.gfx:   lda GFX,x                       ; eigene Grafik ab Code 128 (CHARSET + 1024)
        sta CHARSET + 128 * 8,x
        dex
        cpx #$ff
        bne .gfx
        ldx #0
.spr:   lda SPRITE_DATA,x               ; 11 Sprites zu 64 Byte = 704 Byte (drei Seiten, die letzte nur zum Teil benutzt)
        sta SPRITE_RAM,x
        lda SPRITE_DATA + $100,x
        sta SPRITE_RAM + $100,x
        lda SPRITE_DATA + $200,x
        sta SPRITE_RAM + $200,x
        inx
        bne .spr
        lda #BG_COL1
        sta VIC_BGCOLOR1
        lda #BG_COL2
        sta VIC_BGCOLOR2
        lda #$1b
        sta VIC_CTRL1
        lda #$18                        ; 40 Spalten, Mehrfarben-Zeichen erlaubt
        sta VIC_CTRL2
        lda #$1c                        ; Bildschirm $0400 (Bits 4-7 = 1), Zeichensatz $3000 (Bits 1-3 = 6)
        sta VIC_MEMORY
        jmp clear

; clear: Bildschirm = Leerzeichen, Farbspeicher = weiss (Sprite-Zeiger bleiben unberuehrt).  Veraendert A, X.
clear:  ldx #0
.clr:   lda #CH_SPACE
        sta SCREEN_RAM,x
        sta SCREEN_RAM + $100,x
        sta SCREEN_RAM + $200,x
        lda #COL_WHITE
        sta COLOR_RAM,x
        sta COLOR_RAM + $100,x
        sta COLOR_RAM + $200,x
        sta COLOR_RAM + $300,x
        inx
        bne .clr
        ldx #$e7                        ; letzte Seite: $0700..$07E7 (dahinter stehen die Sprite-Zeiger)
.last:  lda #CH_SPACE
        sta SCREEN_RAM + $300,x
        dex
        cpx #$ff
        bne .last
        rts

; detect_norm: Rasterzeile (Bit 8 in $D011, Bits 0-7 in $D012) ueber rund zwei Bilder beobachten: Zeilen ab 272 gibt es nur bei PAL (312 Zeilen;
;   NTSC hat 262/263).  Setzt F_PAL in FLAGS.  Veraendert A, X, Y.
detect_norm:
        ldy #16
        ldx #0
.l:     lda VIC_CTRL1
        bpl .no                         ; Bit 7 = 0: Zeile unter 256
        lda VIC_RASTER
        cmp #$10
        bcs .pal                        ; Zeile >= 272: PAL
.no:    dex
        bne .l
        dey
        bne .l
        rts                             ; NTSC (FLAGS unveraendert)
.pal:   lda FLAGS
        ora #F_PAL
        sta FLAGS
        rts

; wait_frame: wartet auf den Anfang des naechsten Bildes (Rasterzeile 251).  Veraendert A.
;   Erste Schleife: steht der Strahl noch in Zeile 251, abwarten (sonst kehrte ein zweiter Aufruf sofort zurueck).
;   Zweite Schleife: warten, bis der Strahl Zeile 251 erreicht.  Danach bleiben bis zur ersten Bildzeile mindestens 4000 Takte (NTSC).
wait_frame:
.leave: lda VIC_RASTER
        cmp #RASTER_FRAME
        beq .leave
.enter: lda VIC_RASTER
        cmp #RASTER_FRAME
        bne .enter
        rts
}
