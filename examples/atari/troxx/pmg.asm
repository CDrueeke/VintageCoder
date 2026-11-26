; =====================================================================================================================
; pmg.asm - Player/Missile-Grafik: Figur, bis zu vier Gegner; Pixelbewegung zwischen den Zellen
; =====================================================================================================================
; ZWECK     Die Spiellogik rechnet in Zeichenzellen (gx, gy) und kennt nur Schritte von Zelle zu Zelle.  Die Figuren zeigen diese Schritte glatt:
;           Jede Figur hat ihre Zelle vor dem Schritt (POX/POY, E_OX/E_OY) und ihre Zelle danach (PX/PY, E_X/E_Y).  Im Spielzustand laeuft die
;           Figur waehrend der STEPLEN Bilder eines Schritts um FRAME * 8 / STEPLEN Punkte von der alten zur neuen Zelle (SPPROG = 0..7, waagerecht
;           halbiert: ein Farbtakt je zwei Punkte); in allen anderen Zustaenden steht sie auf der neuen Zelle (SPPROG = 8).
; ZUORDNUNG Spieler 0 = Figur; Spieler 1, 2, 3 = Gegner 1..3 (Farbe nach Typ); Gegner 4 = die vier Missiles (links bis rechts M0, M1, M2 nebeneinander,
;           zusammen sechs Pixel breit; wegen PRIOR Bit 4 in der Farbe COLPF3).  Einstellbar: Breite SIZEP/SIZEM = 0 (Normalbreite), PRIOR = $11
;           (Spieler vor dem Spielfeld, Missiles als fuenfter Spieler) in antic.asm.  Der Gegenstand (Stern, Eisblock) ist ein Zeichen im Spielfeld.
; LAGE      Eine Zelle ist 4 Farbtakte breit und 8 Rasterzeilen hoch.  Figuren sind 6 Pixel breit und 8 Zeilen hoch, mittig auf der Zelle: HPOS = 47 +
;           4 * (gx + GRID_COL0) (der linke Rand des Normalfeldes ist Farbtakt 48), Zeile im Spielerspeicher = 32 + 8 * (gy + GRID_ROW0) (Spielerspeicher
;           in Einzelzeilen-Aufloesung; die erste Textzeile beginnt hinter den 24 Leerzeilen in Rasterzeile 32).
; =====================================================================================================================

sprites: {

; hide: alle Figuren aus (Titelbild, Namenseingabe).  Veraendert A, X.
hide:   lda SPHID
        bne .done
        jsr antic.hide_pm
        lda #0
        ldx #4
.o:     sta OLDY,x
        dex
        bpl .o
        lda #1
        sta SPHID
.done:  rts

; update: einmal je Bild nach wait_frame: Positionen, Bilder, Farben setzen.  Veraendert A, X, Y, SPT0..SPT7, SPN.
update:
        lda SPOFF
        bne hide
        lda STATE
        cmp #St.title
        beq hide
        lda #0
        sta SPHID
        lda STATE
        cmp #St.playing
        beq .calc
        lda #8                          ; ausserhalb des Spiels: Figur steht auf der neuen Zelle
        bne .sp
.calc:  lda FRAME                       ; Fortschritt = FRAME * 8 / STEPLEN (FRAME < STEPLEN)
        asl
        asl
        asl
        ldx #0
.div:   cmp STEPLEN
        bcc .dd
        sbc STEPLEN
        inx
        bne .div
.dd:    txa
.sp:    sta SPPROG
        ; ---- Figur (Spieler 0)
        lda POX
        sta SPT0
        lda PX
        sta SPT1
        lda POY
        sta SPT2
        lda PY
        sta SPT3
        jsr place
        lda #0
        sta SPN
        lda SPHIT
        beq .anim
        lda #FIG_HIT                    ; Treffer-Bild
        ldx #COL_HIT
        bne .pset2
.anim:  lda PANIM
        asl
        asl
        asl
        sta SPT7                        ; Bild 0 oder 1 (je 8 Byte) ...
        lda #COL_PLAYER
        sta SPT6
        lda PINV
        beq .pw
        and #1
        beq .pw
        lda #COL_PLAYER2                ; Unverwundbarkeit: weiss/hellblau blinken
        sta SPT6
.pw:    lda SPT7
        clc
        adc #FIG_PLAYER
        ldx SPT6
        jmp .pset2
.pset2: stx PCOLR0
        jsr put
        ; ---- Gegner
        lda #1
        sta SPN
        ldx #0                          ; Abstand in EN
.en:    cpx ELIM
        bcs .rest
        stx SPT6
        lda EN + E_OX,x
        sta SPT0
        lda EN + E_X,x
        sta SPT1
        lda EN + E_OY,x
        sta SPT2
        lda EN + E_Y,x
        sta SPT3
        jsr place
        ldx SPT6
        lda EN + E_KIND,x
        tay
        asl
        asl
        asl
        asl                             ; Typ * 16 (zwei Bilder zu 8 Byte)
        sta SPT7
        lda EPH
        asl
        asl
        asl
        clc
        adc SPT7                        ; + Bild * 8
        pha
        lda KIND_COL,y                  ; Farbe nach Typ, bei Starre (Eisblock) hellblau
        ldx FREEZE
        bne .fr
        ldx FREEZE + 1
        beq .nf
.fr:    lda #COL_ICE
.nf:    ldx SPN
        cpx #4
        bcs .miss
        sta PCOLR0,x
        jmp .pf
.miss:  sta PF3COL
.pf:    pla
        jsr put
        inc SPN
        lda SPT6
        clc
        adc #E_SIZE
        tax
        bne .en
.rest:  ; nicht belegte Plaetze (SPN .. 4): leeren
.r:     lda SPN
        cmp #5
        bcs .out
        tax
        lda OLDY,x
        beq .rn
        jsr clr_slot
.rn:    inc SPN
        jmp .r
.out:   rts

; clr_slot: Platz X (0..4) leeren (Figur loeschen, HPOS 0).  Veraendert A, X, Y, PMP.
clr_slot:
        lda PMHI,x
        sta PMP + 1
        lda #0
        sta PMP
        ldy OLDY,x
        lda #0
        sta OLDY,x
        ldx #8
.c:     lda #0
        sta (PMP),y
        iny
        dex
        bne .c
        ldx SPN
        cpx #4
        bcs .m
        sta HPOSP0,x
        rts
.m:     sta HPOSM0
        sta HPOSM0 + 1
        sta HPOSM0 + 2
        rts

; place: Position zwischen den Zellen (SPT0, SPT2) und (SPT1, SPT3): Zelle vor dem Schritt plus SPPROG Punkte in Richtung der neuen Zelle.
;   Ergebnis SPX (Farbtakt, HPOS) und SPY (Zeile im Spielerspeicher).  Veraendert A, X, SPX, SPY, SPT4.
place:  ldx SPT0
        lda XPOS,x
        sta SPX
        lda SPT1
        cmp SPT0
        beq .xd
        lda SPPROG
        lsr
        sta SPT4                        ; Farbtakte = Punkte / 2
        lda SPT1
        cmp SPT0
        bcc .xneg
        lda SPX
        clc
        adc SPT4
        sta SPX
        jmp .xd
.xneg:  lda SPX
        sec
        sbc SPT4
        sta SPX
.xd:    ldx SPT2
        lda YPOS,x
        ldx SPT3
        cpx SPT2
        beq .yd
        bcc .yneg
        clc
        adc SPPROG
        jmp .yd
.yneg:  sec
        sbc SPPROG
.yd:    sta SPY
        rts

; put: Figur (A = Abstand in FIG) auf Platz SPN an (SPX, SPY) zeichnen; die alte Lage wird vorher geloescht.  Veraendert A, X, Y, SPT5, SPT7, PMP.
put:    sta SPT7
        ldx SPN
        lda PMHI,x
        sta PMP + 1
        lda #0
        sta PMP
        lda OLDY,x
        beq .noerase
        tay
        lda #0
        ldx #8
.e:     sta (PMP),y
        iny
        dex
        bne .e
.noerase:
        ldx SPN
        lda SPY
        sta OLDY,x
        tay                             ; Y = Zeile im Spielerspeicher
        ldx SPT7                        ; X = Abstand in FIG
        lda #8
        sta SPT5
        lda SPN
        cmp #4
        bcs .mfig
.d:     lda FIG,x
        sta (PMP),y
        inx
        iny
        dec SPT5
        bne .d
        ldx SPN
        lda SPX
        sta HPOSP0,x
        rts
.mfig:                                  ; Gegner 4: sechs Pixel auf M0, M1, M2 (je zwei Pixel, hoeherer Wert = linkes Pixel)
.m:     lda FIG,x
        sta SPT7
        lsr
        lsr
        lsr
        lsr
        lsr
        lsr
        sta SPT4
        lda SPT7
        lsr
        lsr
        lsr
        lsr
        and #3
        asl
        asl
        ora SPT4
        sta SPT4
        lda SPT7
        lsr
        lsr
        and #3
        asl
        asl
        asl
        asl
        ora SPT4
        sta (PMP),y
        inx
        iny
        dec SPT5
        bne .m
        lda SPX
        sta HPOSM0
        clc
        adc #2
        sta HPOSM0 + 1
        adc #2
        sta HPOSM0 + 2
        rts
}

; ---- Daten: Zeilen der Figuren (sechs Pixel breit, Bit 7 = linkes Pixel, untere zwei Bit leer), acht Zeilen je Bild
FIG:
        .byte %01111000, %11111100, %10110100, %11111100, %11111100, %10110100, %10000100, %00000000   ; Laeufer, Bild 0
        .byte %01111000, %11111100, %10110100, %11111100, %11111100, %01001000, %01001000, %00000000   ; Laeufer, Bild 1
        .byte %10000100, %01001000, %01111000, %11001100, %11111100, %01111000, %10110100, %00000000   ; Streuner, Bild 0
        .byte %10000100, %01001000, %01111000, %11001100, %11111100, %01111000, %01001000, %00000000   ; Streuner, Bild 1
        .byte %11111100, %11111100, %10110100, %11111100, %01111000, %01001000, %10000100, %00000000   ; Waechter, Bild 0
        .byte %11111100, %11111100, %10110100, %11111100, %01111000, %10000100, %01001000, %00000000   ; Waechter, Bild 1
.const FIG_HIT = 48
        .byte %10000100, %01001000, %00110000, %00110000, %00110000, %00110000, %01001000, %10000100   ; Treffer
.const FIG_PLAYER = 56
        .byte %00110000, %01111000, %11111100, %10110100, %11111100, %11111100, %01111000, %00110000   ; Figur, Bild 0
        .byte %00110000, %01111000, %11111100, %10110100, %11111100, %11111100, %01111000, %01001000   ; Figur, Bild 1

PMHI: .byte >PM_P0, >(PM_P0 + $100), >(PM_P0 + $200), >(PM_P0 + $300), >PM_M   ; Seite des Platzes 0..3 (Spieler) und 4 (Missiles)

; Lage der Zellen: HPOS = 47 + 4 * (gx + GRID_COL0); Zeile = 32 + 8 * (gy + GRID_ROW0)
XPOS:
.for gx in range(GRID_W)
        .byte 47 + 4 * (gx + GRID_COL0)
.endfor
YPOS:
.for gy in range(GRID_H)
        .byte 32 + 8 * (gy + GRID_ROW0)
.endfor
