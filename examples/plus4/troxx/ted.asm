; =====================================================================================================================
; ted.asm - Bild des TED: Norm erkennen, Register, Zeichensatz, Bildtakt
; =====================================================================================================================
; ZWECK     Alles, was den Video-Chip betrifft und nur einmal oder je Bild geschieht.
; ABLAUF    init: Farben -> Norm erkennen (PAL/NTSC) -> Zeichen-ROM (Bildschirmcodes 0..127) nach $4000 kopieren, eigene Grafik (Codes 128..) aus dem
;           Programm danach -> Register -> Bildschirm leeren.
; HARDWARE  $FF06 = $1B (Bild an, 25 Zeilen, Y-Versatz 3), $FF07 = $98 (Zeichen 128..255 getrennt, Mehrfarben moeglich, 40 Spalten), $FF12 Bit 2 = 0
;           (Zeichensatz aus dem RAM), $FF13 = $40 (Zeichensatz bei $4000), $FF14 = $08 (Farbspeicher $0800, Bildschirm $0C00), $FF15 Hintergrund 0,
;           $FF16 und $FF17 Mehrfarben-Hintergrund 1 und 2 (Bitpaare 01 und 10; Paar 11 = Farbe der Zelle mit Farbton 0..7 und deren Helligkeit; Paar 00 =
;           Hintergrund 0), $FF19 Rahmen.  Farbspeicher-Bit 3 = 1 schaltet ein Zeichen auf Mehrfarben (Fuellungen, Logo); Zeichen mit Bit 3 = 0 sind
;           einfarbig (Linien, Text).  $FF1C Bit 0 und $FF1D = Rasterzeile (PAL 0..311, NTSC 0..261).  Der Zeichen-ROM liegt bei $D000 (nur bei
;           eingeblendetem KERNAL-ROM lesbar), Bildschirmcode n bei $D000 + 8 n.
;           Der Bildtakt kommt vom Rasterinterrupt (raster.asm); er zaehlt FRAMEC in Zeile 204 hoch (unter dem Bild; das Bild belegt die Zeilen 4..203).
; =====================================================================================================================

ted: {

.const PAL_LINE = 8                     ; Rasterzeile 256 + 8 = 264: nur PAL (312 Zeilen) erreicht sie, NTSC hoert bei 261 auf

; init: Veraendert A, X, Y.
init:
        lda #BG0_COLOR
        sta TED_BGCOLOR
        lda #BORDER_COLOR
        sta TED_BORDERCOLOR
        jsr game_colors
        jsr detect_norm
        lda FLAGS
        and #F_PAL
        beq .ntsc
        lda #50
        bne .fps
.ntsc:  lda #60
.fps:   sta FPS
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
        ldx #0
.gfx:   lda GFX,x                       ; eigene Grafik ab Code 128 (CHARSET + 1024): NGFX * 8 = 552 Byte
        sta CHARSET + 128 * 8,x
        lda GFX + $100,x
        sta CHARSET + 128 * 8 + $100,x
        inx
        bne .gfx
        ldx #NGFX * 8 - $200 - 1        ; Rest (die dritte Seite)
.g3:    lda GFX + $200,x
        sta CHARSET + 128 * 8 + $200,x
        dex
        bpl .g3
        lda TED_BITMAP_BASE
        and #$fb                        ; Bit 2 = 0: Zeichensatz aus dem RAM (die uebrigen Bits, v. a. die Tonhoehe von Stimme 1, bleiben)
        sta TED_BITMAP_BASE
        lda #$40
        sta TED_CHARSET_BASE            ; Zeichensatz bei $4000, einfacher Takt
        lda #$08
        sta TED_SCREEN_BASE             ; Farbspeicher $0800, Bildschirm $0C00
        lda #$98
        sta TED_CTRL2
        lda #$1b
        sta TED_CTRL1
        jmp clear

; game_colors: Mehrfarben-Hintergrund 1 und 2 fuer das Spiel (Rechteckzeile 2; mit FX_RASTER wechseln sie je Rechteckzeile).  Veraendert A.
game_colors:
        lda BAND_BG1 + 2
        sta TED_COLOR1
        lda BAND_BG2 + 2
        sta TED_COLOR2
        rts

; title_colors: Mehrfarben-Hintergrund 1 und 2 fuer das Titelbild (Logo-Bloecke: dunkler Schatten, heller Glanz).  Veraendert A.
title_colors:
        lda #$21
        sta TED_COLOR1
        lda #$71
        sta TED_COLOR2
        rts

; clear: Bildschirm = Leerzeichen, Farbspeicher = weiss, gezeichnete Figuren vergessen.  Veraendert A, X.
clear:  ldx #0
.clr:   lda #CH_SPACE
        sta SCREEN_RAM,x
        sta SCREEN_RAM + $100,x
        sta SCREEN_RAM + $200,x
        sta SCREEN_RAM + $300,x
        lda #COL_WHITE
        sta COLOR_RAM,x
        sta COLOR_RAM + $100,x
        sta COLOR_RAM + $200,x
        sta COLOR_RAM + $300,x
        inx
        bne .clr
        lda #$ff
        ldx #5
.fg:    sta FIG_OX,x
        dex
        bpl .fg
        rts

; detect_norm: Rasterzeile ueber rund acht Bilder beobachten: Zeilen ab 264 gibt es nur bei PAL.  Setzt F_PAL in FLAGS.  Veraendert A, X, Y.
detect_norm:
        ldy #40
        ldx #0
.l:     lda TED_RASTER_HI
        and #1
        beq .no                         ; Zeile unter 256
        lda TED_RASTER_LO
        cmp #PAL_LINE
        bcs .pal
.no:    dex
        bne .l
        dey
        bne .l
        rts                             ; NTSC (FLAGS unveraendert)
.pal:   lda FLAGS
        ora #F_PAL
        sta FLAGS
        rts

; wait_line: ein Bild lang warten, nur mit der Rasterabfrage (Zeile 100 verlassen und wieder erreichen), unabhaengig vom Interrupt.  Veraendert A.
wait_line:
.l1:    lda TED_RASTER_LO
        cmp #100
        beq .l1
.l2:    lda TED_RASTER_LO
        cmp #100
        bne .l2
        rts

; wait_frame: wartet auf den naechsten Bildanfang (der Rasterinterrupt in Zeile 204 erhoeht FRAMEC).  Veraendert A.
;   Kam das Bild schon, ehe der Aufruf erfolgt (die Spiellogik brauchte laenger), kehrt er sofort zurueck.
wait_frame:
.w:     lda FRAMEC
        cmp FRAMEW
        beq .w
        sta FRAMEW
        rts
}
