; =====================================================================================================================
; vic.asm - Bild des VIC-I: Norm erkennen, Register, Zeichensatz, Bildtakt
; =====================================================================================================================
; ZWECK     Alles, was den Video-Chip betrifft und nur einmal oder je Bild geschieht.
; ABLAUF    init: Rahmen/Hintergrund schwarz -> Norm erkennen (PAL/NTSC) -> Bildlage -> Zeichen A-Z, 0-9 aus dem Zeichen-ROM
;           nach $1C00 kopieren -> Bildschirm leeren -> Spalten/Zeilen/Zeichenbasis einstellen.
; HARDWARE  $9000/$9001 Bildlage (je Norm), $9002 Spalten (22) + Bildschirmadresse Bit 9, $9003 Zeilen (23), $9004 Rasterzaehler (Zeile / 2),
;           $9005 Speicherzeiger (Bit 4-7 Bildschirm, Bit 0-3 Zeichenbasis; $FF = Bildschirm $1E00, Zeichen $1C00), $900F Hintergrund/Rahmen.
;           Der Zeichen-ROM liegt ab $8000 (Grossbuchstaben und Grafik), Bildschirmcode n steht bei $8000 + 8 * n.
; =====================================================================================================================

; copy_chars: kopiert n Zeichen ab Code c0 aus dem Zeichen-ROM in den Zeichensatz im RAM (n * 8 <= 256 Byte)
.macro copy_chars(c0, cn)
        ldx #(cn) * 8
@l:     lda CHAR_ROM + (c0) * 8 - 1,x
        sta CHARSET + (c0) * 8 - 1,x
        dex
        bne @l
.endmacro

vic: {

; Rasterwert (Zeile / 2), unter dem der Strahl im oberen Rand steht, noch vor der ersten Bildzeile (NTSC ab 25, PAL ab 38)
.const RASTER_TOP = 10
; Erkennung: PAL zaehlt bis 155 (312 Zeilen), NTSC bis 130 (261 Zeilen); Grenze dazwischen
.const PAL_LIMIT  = 143

; init: Veraendert A, X, Y, T0.
init:
        lda #$08                        ; Hintergrund und Rahmen schwarz, normale Darstellung
        sta VIC_COLOR
        jsr detect_norm
        ldx #0                          ; X = 0 (NTSC) oder 1 (PAL)
        lda FLAGS
        and #F_PAL
        beq .norm
        inx
.norm:  lda ORIGIN_X,x                  ; Bildlage waagerecht und senkrecht je Norm
        sta VIC_ORIGIN_X
        lda ORIGIN_Y,x
        sta VIC_ORIGIN_Y
        copy_chars 1, 26                ; A-Z
        copy_chars 48, 10               ; 0-9
        jsr clear
        lda #$80 | SCR_COLS             ; 22 Spalten, Bit 7 = Bildschirmadresse Bit 9 (Farbspeicher $9600)
        sta VIC_COLS
        lda #SCR_ROWS << 1              ; 23 Zeilen, 8 x 8 Punkte
        sta VIC_ROWS
        lda #$ff                        ; Bildschirm $1E00, Zeichenbasis $1C00
        sta VIC_MEMPTR
        rts

; clear: Bildschirm = Leerzeichen, Farbspeicher = weiss (2 Seiten zu 256 Byte).  Veraendert A, X.
clear:  ldx #0
.clr:   lda #CH_SPACE
        sta SCREEN_RAM,x
        sta SCREEN_RAM + $100,x
        lda #COL_WHITE
        sta COLOR_RAM,x
        sta COLOR_RAM + $100,x
        inx
        bne .clr
        rts

; detect_norm: Rasterzaehler ueber rund sieben Bilder beobachten, groesster Wert > PAL_LIMIT = PAL.  Setzt F_PAL in FLAGS.  Veraendert A, X, Y, T0.
detect_norm:
        lda #0
        sta T0                          ; bisheriger Hoechstwert
        ldy #40                         ; 40 * 256 Messungen zu rund 15 Takten = 150000 Takte (PAL 6,9 Bilder, NTSC 8,8)
.outer: ldx #0
.inner: lda VIC_RASTER
        cmp T0
        bcc .lower
        sta T0
.lower: dex
        bne .inner
        dey
        bne .outer
        lda T0
.ifdef SPIKE_PROBE
        sta $033c                       ; Messung: groesster Rasterwert (Kassettenpuffer)
.endif
        cmp #PAL_LIMIT
        bcc .done                       ; kleiner: NTSC
        lda FLAGS
        ora #F_PAL
        sta FLAGS
.done:  rts

; wait_frame: wartet auf den Anfang des naechsten Bildes (Rasterzaehler laeuft ueber den oberen Rand hinaus).  Veraendert A.
;   Erste Schleife: steht der Strahl noch im oberen Rand, abwarten, bis er ihn verlaesst (sonst kehrte der Aufruf sofort zurueck).
;   Zweite Schleife: warten, bis der Zaehler wieder im oberen Rand steht; danach bleiben mindestens 3000 Takte bis zur ersten Bildzeile.
wait_frame:
.leave: lda VIC_RASTER
        cmp #RASTER_TOP
        bcc .leave
.enter: lda VIC_RASTER
        cmp #RASTER_TOP
        bcs .enter
        rts
}
