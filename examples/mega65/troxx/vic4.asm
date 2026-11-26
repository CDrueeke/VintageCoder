; =====================================================================================================================
; vic4.asm - MEGA65: Bildchip VIC-IV (80 x 50 Zeichen, Vollfarb-Kacheln, Palette, Sprites), Bildtakt, DMA
; =====================================================================================================================
; ZWECK     Alles, was die Hardware des MEGA65 betrifft, ausser Ton (sound.asm) und Eingabe (input.asm).
;
; WAS DER CODE AUF DER HARDWARE TUT (am Emulator Xemu geprueft, siehe KONZEPT.md Abschnitt 15.4)
;   Start     map mit A = X = Y = Z = 0 hebt die Speicherumblendung des BASIC-65-Starts auf, $01 = $35 blendet alle ROMs aus (RAM bis $FFFF, Ein-/Ausgabe
;             bei $D000); der VIC-IV wird mit $47, $53 nach $D02F freigeschaltet; $D030 Bit 2 schaltet auf die Registerpalette.
;   Bild      $D031 = $C8 (Bit 7 H640, Bit 6 FAST, Bit 3 V400): 640 x 400 Bildpunkte = 80 x 50 Zeichen zu 8 x 8; DANACH $D054 = $55 (CHR16: 16-Bit-
;             Zeichennummern, FCLRHI, SPRH640: Sprites in echten Bildpunkten, VFAST: 40,5 MHz), $D058/$D059 = 160 (Byte je Zeile), $D05E = 80,
;             Bildschirm $0001D000 ($D060..$D062), Farb-RAM $0FF80000 ($D064, $D065 = 0), 2 Byte je Zelle in beiden.  Ein Schreiben auf $D011, $D016,
;             $D018, $D031 oder $DD00 setzt diese VIC-IV-Register zurueck: danach immer seam aufrufen.
;   Zeichen   Zeichennummer = 16 Bit (niedriges Byte zuerst).  Nummern ab $0100 sind Vollfarb-Kacheln (FCM): Daten bei Nummer * 64 im Chip-RAM,
;             8 x 8 Punkte, ein Byte je Punkt = Palettenindex; 0 = durchsichtig, 255 = Farbe aus dem Farb-RAM (zweites Byte der Zelle, ganzer Wert
;             als Palettenindex).  Kacheln liegen ab $12000 (Nummer $0480), die Schrift (4 Themen zu 64 Zeichen) ab $14000 (Nummer $0500).
;   Palette   Register $D100 (rot), $D200 (gruen), $D300 (blau) mit vertauschten Halbbytes; $D070: Bits 7-6 waehlen die Palettenbank in $D100..$D3FF,
;             Bits 3-2 die der Sprites.  Sprite n benutzt die Eintraege 16 n bis 16 n + 15 der Sprite-Bank (Bank 1: $D070 = $44 zum Schreiben, danach $04).
;   Sprites   16 x 16 Punkte mit 16 Farben ($D06B = $FF, Hoehe 16 in $D055/$D056, V400 in $D076); Zeiger bei SPR_PTR (VIC-Bank 2), Daten ab SPR_DATA.
;             Koordinaten in 640 x 400: X' = x + 79, Y' = y + 107.
;   Bildtakt  Rasterabfrage ueber das Flag $D019 Bit 0 bei Zeile RASTER_LINE (kein Interrupt).
;   DMA       DMAgic, Auftragsliste im Format F018A (11 Byte): Kopieren der Kacheln und Sprites, Bildschirm loeschen, Fuellanimation.
; =====================================================================================================================

vic: {

; init: MEGA65 einrichten (einmal beim Start).  Veraendert A, X, Y, Z, sq, cq.
init:
        lda #0
        tax
        tay
        taz
        map
        eom                             ; Speicherumblendung aufheben, Interruptsperre des map beenden
        lda #$35
        sta $01                         ; RAM ueberall, Ein-/Ausgabe bei $D000
        mega65_enable_vic4
        lda #0
        sta $d030                       ; keine C65-ROM-Fenster
        lda #$40
        sta $d031
        sta $d054                       ; erst im C64-Takt Norm erkennen
        lda #0
        sta $d059
        lda #40
        sta $d058
        sta $d05e
        lda #$7f
        sta $dc0d                       ; alle CIA-Interrupts aus
        sta $dd0d
        lda $dc0d
        lda $dd0d
        lda $dd02
        ora #3
        sta $dd02                       ; CIA 2 Port A Bits 0..1 als Ausgang (VIC-Bank)
        lda $dd00
        and #$fc
        ora #$01
        sta $dd00                       ; VIC-Bank 2 ($8000-$BFFF): Sprite-Zeiger und -Daten
        lda #$12
        sta $d018
        lda #$1b
        sta $d011
        lda #$18
        sta $d016
        lda #0
        sta $d020
        sta $d021
        lda #$53
        sta $d05c
        jsr detect_norm                 ; PAL (50 Hz) oder NTSC (60 Hz) am Rasterzaehler
        lda #$c8
        sta $d031                       ; 640 x 400, FAST
        jsr seam
        lda #RASTER_LINE
        sta $d012                       ; Zeile fuer wait_frame
        lda #0
        sta $d070
        lda $d030
        ora #4
        sta $d030                       ; Registerpalette
        jsr pal_init
        jsr upload
        jsr font_init
        jsr spr_init
        jsr clear
        rts

; seam: VIC-IV-Register des Bildmodus setzen (nach jedem Schreiben auf $D011/$D016/$D018/$D031/$DD00 erneut).  Veraendert A.
seam:   lda #$55
        sta $d054                       ; CHR16, FCLRHI, SPRH640, VFAST
        lda #160
        sta $d058
        lda #0
        sta $d059
        lda #80
        sta $d05e
        lda #0
        sta $d060
        lda #>SCRQ
        sta $d061
        lda #1
        sta $d062                       ; Bildschirm bei $0001D000
        lda #0
        sta $d064
        sta $d065                       ; Farb-RAM bei $0FF80000 + 0
        lda #$00
        sta $d068
        sta $d06a
        lda #$d0
        sta $d069                       ; Zeichensatz (ROM) bei $0002D000 fuer die Zeichen unter 256
        lda #$02
        sta $d06a
        lda #<SPR_PTR
        sta $d06c
        lda #>SPR_PTR
        sta $d06d
        lda #0
        sta $d06e                       ; Sprite-Zeiger in der VIC-Bank, 8 Bit je Zeiger
        rts

; detect_norm: PAL oder NTSC aus dem groessten Rasterwert zweier Bilder (PAL 312 Zeilen, NTSC 262; Grenze 285).  Setzt F_PAL in FLAGS.  Veraendert A, X, Y.
detect_norm:
        lda #0
        sta T0
        sta T1                          ; groesster Wert (Bit 8 in T1)
        sta T2
        sta T3
.frm:   ldy #0                          ; mehrere Bilder lang abfragen
.w:     lda $d011
        asl                             ; Bit 7 (Rasterzeile Bit 8) in den Uebertrag
        lda #0
        rol
        tax
        lda $d012
        cpx T1
        bcc .nx
        bne .big
        cmp T0
        bcc .nx
.big:   sta T0
        stx T1
.nx:    iny
        bne .w
        inc T2
        lda T2
        cmp #200
        bcc .frm
        lda T1
        beq .ntsc                       ; Maximum < 256: NTSC
        lda T0
        cmp #29                         ; 256 + 29 = 285
        bcc .ntsc
        lda FLAGS
        ora #F_PAL
        sta FLAGS
.ntsc:  rts

; ---------------------------------------------------------------------------------------------------------------------
; Palette
; ---------------------------------------------------------------------------------------------------------------------
; pal_init: 256 Eintraege der Palettenbank 0 und 128 Eintraege der Sprite-Bank 1 schreiben.  Veraendert A, X.
pal_init:
        ldx #0
.p:     lda PAL_R,x
        sta $d100,x
        lda PAL_G,x
        sta $d200,x
        lda PAL_B,x
        sta $d300,x
        inx
        bne .p
        lda #$44
        sta $d070                       ; Bank 1 in $D100..$D3FF, Sprites benutzen Bank 1
.s:     lda SPAL_R,x
        sta $d100,x
        lda SPAL_G,x
        sta $d200,x
        lda SPAL_B,x
        sta $d300,x
        inx
        cpx #128
        bne .s
        lda #$04
        sta $d070                       ; Bank 0 wieder in $D100..$D3FF
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Kacheln, Schrift, Sprites
; ---------------------------------------------------------------------------------------------------------------------
; upload: Kacheln ($12000) und Sprite-Daten (SPR_DATA) per DMA kopieren.  Veraendert A.
upload:
        lda #<(NTILES * 64)
        ldx #>(NTILES * 64)
        jsr dma_len
        lda #<TILES
        sta DMA_SRC
        lda #>TILES
        sta DMA_SRC + 1
        lda #0
        sta DMA_SRCB
        lda #<TILE_RAM
        sta DMA_DST
        lda #>TILE_RAM
        sta DMA_DST + 1
        lda #(TILE_RAM >> 16)
        sta DMA_DSTB
        jsr dma_copy
        lda #<(NSPR_IMG * 128)
        ldx #>(NSPR_IMG * 128)
        jsr dma_len
        lda #<SPRITES
        sta DMA_SRC
        lda #>SPRITES
        sta DMA_SRC + 1
        lda #0
        sta DMA_SRCB
        lda #<SPR_DATA
        sta DMA_DST
        lda #>SPR_DATA
        sta DMA_DST + 1
        lda #0
        sta DMA_DSTB
        jmp dma_copy

; font_init: Schrift aus dem Zeichen-ROM ($0002D000, Grossbuchstaben) als Vollfarb-Kacheln erzeugen: 4 Themen (Hintergrund = Palettenindex der Tafel
;   0, 1, 2 oder Index 0) mal 64 Zeichen; gesetzter Punkt = 255 (Farbe aus dem Farb-RAM), sonst Hintergrund.  Ziel FONT_RAM.  Veraendert A, X, Y, Z, T0..T3.
font_init:
        lda #<FONT_RAM
        sta cq
        lda #>FONT_RAM
        sta cq + 1
        lda #(FONT_RAM >> 16)
        sta cq + 2
        lda #0
        sta cq + 3
        lda #0
        sta T3                          ; Thema 0..3
.th:    ldx T3
        lda FONT_BG,x
        sta T2                          ; Hintergrundindex
        lda #0
        sta T0                          ; Zeichen 0..63
        lda #$00
        sta sq
        lda #$d0
        sta sq + 1
        lda #$02
        sta sq + 2
        lda #0
        sta sq + 3                      ; Zeichen-ROM
.ch:    ldy #0                          ; Zeile 0..7
.row:   tya
        taz
        lda [sq],z                      ; Bitmuster der Zeile
        sta T1
        tya
        asl
        asl
        asl
        taz                             ; Z = Zeile * 8: Ziel in cq
        ldx #8
.px:    asl T1
        lda T2
        bcc .bg
        lda #255
.bg:    sta [cq],z
        inz
        dex
        bne .px
        iny
        cpy #8
        bne .row
        lda sq                          ; naechstes Zeichen im ROM (8 Byte)
        clc
        adc #8
        sta sq
        bcc .ns
        inc sq + 1
.ns:    lda cq                          ; naechste Kachel (64 Byte)
        clc
        adc #64
        sta cq
        bcc .nc
        inc cq + 1
.nc:    inc T0
        lda T0
        cmp #64
        bne .ch
        inc T3
        lda T3
        cmp #4
        bne .th
        ldz #0
        rts

FONT_BG: .byte PAL_PAN0, PAL_PAN0 + 3, PAL_PAN0 + 6, 0   ; Hintergrund je Thema: Tafel Spieler 1, Tafel Spieler 2, Tafel Stufe, ohne

; spr_init: Spriteregister (16 x 16, 16 Farben, 640 x 400) einrichten, Zeiger auf die Bilder.  Veraendert A, X.
spr_init:
        lda #0
        sta $d015                       ; alle aus
        sta $d01b                       ; vor dem Hintergrund
        sta $d01c                       ; keine Mehrfarb-Sprites
        sta $d017
        sta $d01d                       ; keine Verdopplung
        lda #$ff
        sta $d06b                       ; 16-Farben-Sprites (16 Punkte breit)
        sta $d055                       ; eigene Hoehe ...
        sta $d076                       ; V400
        lda #16
        sta $d056                       ; ... 16 Zeilen
        lda #0
        ldx #7
.c:     sta $d027,x                     ; Spritefarbe 0 (im 16-Farben-Modus durchsichtig)
        dex
        bpl .c
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Bildtakt
; ---------------------------------------------------------------------------------------------------------------------
; wait_frame: auf die naechste Bildzeile RASTER_LINE warten (einmal je Bild).  Veraendert A.
wait_frame:
        lda #1
        sta $d019
.w:     lda $d019
        and #1
        beq .w
        rts

; ---------------------------------------------------------------------------------------------------------------------
; DMA (DMAgic, Auftrag im Format F018A, 11 Byte, Liste in DMA_JOB)
; ---------------------------------------------------------------------------------------------------------------------
; dma_len: Anzahl A (niedrig), X (hoch) in den Auftrag.
dma_len:
        sta DMA_LEN
        stx DMA_LEN + 1
        rts

; dma_copy: Auftrag "Kopieren" (Quelle, Ziel, Laenge schon gesetzt) ausfuehren.  Veraendert A.
dma_copy:
        lda #0
        sta DMA_CMD
        jmp dma_go

; dma_fill: Auftrag "Fuellen" mit dem Wert A (Ziel und Laenge schon gesetzt) ausfuehren.  Veraendert A.
dma_fill:
        sta DMA_SRC
        lda #0
        sta DMA_SRC + 1
        sta DMA_SRCB
        lda #3
        sta DMA_CMD
        ; weiter mit dma_go

; dma_go: Auftrag ausfuehren (Modulo 0).  Veraendert A.
dma_go: lda #0
        sta DMA_MOD
        sta DMA_MOD + 1
        sta $d702                       ; Bank der Liste: 0
        sta $d704                       ; Erweiterung 0
        lda #>DMA_JOB
        sta $d701
        lda #<DMA_JOB
        sta $d700                       ; Schreiben startet den Auftrag
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Bildschirm
; ---------------------------------------------------------------------------------------------------------------------
; clear: Bildschirm (Leerzeichen) und Farb-RAM loeschen.  Veraendert A, X, Z, sq, cq.
clear:
        lda #<SCRQ
        sta sq
        lda #>SCRQ
        sta sq + 1
        lda #1
        sta sq + 2
        lda #0
        sta sq + 3
        ldz #0
        lda #CH_SPACE
        sta [sq],z                      ; erste Zelle: Leerzeichen
        inz
        lda #0
        sta [sq],z
        lda #<(80 * 50 * 2 - 2)         ; DMA: ueberlappendes Kopieren wiederholt die Zelle
        ldx #>(80 * 50 * 2 - 2)
        jsr dma_len
        lda #<SCRQ
        sta DMA_SRC
        lda #>SCRQ
        sta DMA_SRC + 1
        lda #1
        sta DMA_SRCB
        lda #<(SCRQ + 2)
        sta DMA_DST
        lda #>(SCRQ + 2)
        sta DMA_DST + 1
        lda #1
        sta DMA_DSTB
        jsr dma_copy
        lda #0
        sta cq
        lda #$f8
        sta cq + 2
        lda #$0f
        sta cq + 3
        ldx #0
.p:     stx cq + 1
        ldz #0
.c:     lda #0
        sta [cq],z
        inz
        lda #COL_WHITE
        sta [cq],z
        inz
        bne .c
        inx
        cpx #32
        bne .p
        ldz #0
        rts

; set_ptr: sq und cq auf die Zelle Spalte X (0..79), Zeile Y (0..49) setzen.  Veraendert A.
set_ptr:
        txa
        asl
        clc
        adc ROW160_LO,y
        sta sq
        sta cq
        lda ROW160_HI,y
        adc #0
        sta cq + 1
        clc
        adc #>SCRQ
        sta sq + 1
        lda #1
        sta sq + 2
        lda #$f8
        sta cq + 2
        lda #0
        sta sq + 3
        lda #$0f
        sta cq + 3
        rts

; put_tile: Vollfarb-Kachel A (Nummer TILE0 + A) an Spalte X, Zeile Y; Farb-RAM 0.  Veraendert A, Z, sq, cq.
put_tile:
        pha
        jsr set_ptr
        pla
        ldz #0
        clc
        adc #<TILE0
        sta [sq],z
        lda #>TILE0
        adc #0
        inz
        sta [sq],z
        lda #0
        sta [cq],z
        dez
        sta [cq],z
        rts

; put_blank: Leerzeichen (ohne Farbe) an Spalte X, Zeile Y.  Veraendert A, Z, sq, cq.
put_blank:
        jsr set_ptr
        ldz #0
        lda #CH_SPACE
        sta [sq],z
        inz
        lda #0
        sta [sq],z
        sta [cq],z
        dez
        sta [cq],z
        rts

; put_row4: vier gleiche Vollfarb-Kacheln A ab Spalte X, Zeile Y per DMA in den Bildschirm kopieren (Farb-RAM bleibt).  Veraendert A, X, Y und die DMA-Liste.
put_row4:
        pha
        txa
        asl
        clc
        adc ROW160_LO,y
        sta DMA_DST
        lda ROW160_HI,y
        adc #>SCRQ
        sta DMA_DST + 1
        lda #1
        sta DMA_DSTB
        pla
        clc
        adc #<TILE0
        sta FILLBUF
        sta FILLBUF + 2
        sta FILLBUF + 4
        sta FILLBUF + 6
        lda #>TILE0
        adc #0
        sta FILLBUF + 1
        sta FILLBUF + 3
        sta FILLBUF + 5
        sta FILLBUF + 7
        lda #<FILLBUF
        sta DMA_SRC
        lda #>FILLBUF
        sta DMA_SRC + 1
        lda #0
        sta DMA_SRCB
        lda #8
        ldx #0
        jsr dma_len
        jmp dma_copy

; put_tile_c: wie put_tile, mit Farbbyte T4 (Palettenindex fuer Punkte mit Wert 255).
put_tile_c:
        pha
        jsr set_ptr
        pla
        ldz #0
        clc
        adc #<TILE0
        sta [sq],z
        lda #>TILE0
        adc #0
        inz
        sta [sq],z
        lda T4
        sta [cq],z
        lda #0
        dez
        sta [cq],z
        rts

; put_char: Schriftzeichen A (Bildschirmcode 0..63) in Thema T5 (0..3) an Spalte X, Zeile Y, Farbe T4 (Palettenindex).  Veraendert A, Z, sq, cq.
put_char:
        pha
        jsr set_ptr
        pla
        and #63
        ldz #0
        sta RT7
        lda T5
        asl
        asl
        asl
        asl
        asl
        asl                             ; Thema * 64
        ora RT7
        clc
        adc #<FONT0
        sta [sq],z
        lda #>FONT0
        adc #0
        inz
        sta [sq],z
        lda T4
        sta [cq],z
        lda #0
        dez
        sta [cq],z
        rts
}




