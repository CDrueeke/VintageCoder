; =====================================================================================================================
; sprites.asm - Sprites: Figuren, Gegner, Gegenstand (16 x 16, 16 Farben, Pixelbewegung)
; =====================================================================================================================
; Herkunft: Idee und Aufbau wie examples/c64/troxx/sprites.asm (Stand Commit 4697d074 der Referenz); fuer den VIC-IV neu geschrieben; nicht in der
;   Kopienliste.
; ZWECK     Die Sprites zeigen die Figuren (Sprite 0 und 1), die Gegner (Sprites 2..6) und den Gegenstand (Sprite 7).  Sie gleiten in acht Punkten
;           von der Zelle vor dem Schritt (POX, POY bzw. E_OX, E_OY) zur neuen Zelle: Fortschritt SPPROG = FRAME * 8 / STEPLEN.  Die Entscheidung
;           bleibt an Knoten, die Kollision auf Zeichenebene (grid.asm, player.asm, enemies.asm); die Sprites sind nur Darstellung.
; HARDWARE  640 x 400 ($D054 Bit 4 SPRH640, $D076 V400): ein Spritepunkt ist ein echter Bildpunkt; Koordinaten X' = x + 79 (10 Bit: $D000.., Bit 8 in $D010,
;           Bit 9 in $D05F), Y' = y + 107 (9 Bit: $D001.., Bit 8 in $D077).  Das 16 x 16-Bild liegt mittig auf der 8 x 8-Zelle (4 Punkte nach links und oben
;           versetzt, siehe data.inc).  Sprite n benutzt die Paletteneintraege 16 n bis 16 n + 15 der Sprite-Bank (Farbe je Sprite fest).
;           Zeiger: SPR_PTR + n, Wert SPR_BLK + 2 * Bild (jedes Bild belegt zwei Bloecke zu 64 Byte).
; BILDER    0 Figur A, 1 Figur B, 2 Figur getroffen, 3/4 Laeufer, 5/6 Streuner, 7/8 Waechter, 9 Stern, 10 Eisblock.
; =====================================================================================================================

sprites: {

; hide: alle Sprites aus.  Veraendert A.
hide:   lda #0
        sta $d015
        rts

; place: Sprite X (0..7) an die Stelle zwischen den Zellen (RT0, RT1) und (RT2, RT3) setzen (Fortschritt SPPROG); die hohen Koordinatenbits sammeln sich in
;   SPX8, SPX9, SPY8.  Veraendert A, Y, T0..T3.
place:  ldy RT0
        lda XLO,y
        sta T0
        lda XHI,y
        sta T1
        lda RT2
        cmp RT0
        beq .xd
        bcc .xm
        lda T0                          ; nach rechts
        clc
        adc SPPROG
        sta T0
        bcc .xd
        inc T1
        bne .xd
.xm:    lda T0                          ; nach links
        sec
        sbc SPPROG
        sta T0
        bcs .xd
        dec T1
.xd:    ldy RT1
        lda YLO,y
        sta T2
        lda YHI,y
        sta T3
        lda RT3
        cmp RT1
        beq .yd
        bcc .ym
        lda T2                          ; nach unten
        clc
        adc SPPROG
        sta T2
        bcc .yd
        inc T3
        bne .yd
.ym:    lda T2                          ; nach oben
        sec
        sbc SPPROG
        sta T2
        bcs .yd
        dec T3
.yd:    txa
        asl
        tay
        lda T0
        sta $d000,y
        lda T2
        sta $d001,y
        lda BITMASK,x
        sta T0                          ; Maske des Sprites
        lda T1
        and #1
        beq .n8
        lda T0
        ora SPX8
        sta SPX8
.n8:    lda T1
        and #2
        beq .n9
        lda T0
        ora SPX9
        sta SPX9
.n9:    lda T3
        and #1
        beq .ny
        lda T0
        ora SPY8
        sta SPY8
.ny:    rts

; update: einmal je Bild: Positionen, Bilder und Freigabe aller Sprites setzen.  Veraendert A, X, Y, T0..T3, RT0..RT3, SPN, SPMASK, SPX8, SPX9, SPY8.
update: lda STATE
        cmp #St.title
        beq .off                        ; Titelbild: keine Sprites
        lda SPOFF
        beq .go
.off:   lda #0
        sta $d015
        rts
.go:    lda FRAME                       ; Fortschritt = FRAME * 8 / STEPLEN (0..7)
        asl
        asl
        asl
        ldx #0
.d:     cmp STEPLEN
        bcc .dd
        sbc STEPLEN
        inx
        bne .d
.dd:    stx SPPROG
        lda #0
        sta SPMASK
        sta SPX8
        sta SPX9
        sta SPY8
        ; ---- Figuren
        ldx #0
.fig:   lda PACT,x
        beq .nf
        stx SPN
        lda PX,x
        sta RT2
        lda PY,x
        sta RT3
        lda POX,x
        sta RT0
        lda POY,x
        sta RT1
        jsr place
        ldx SPN
        lda SPHIT
        bne .hit
        lda PANIM,x
        asl                             ; Bild A oder B: Zeiger SPR_BLK + 0 oder + 2
        jmp .img
.hit:   lda #4
.img:   clc
        adc #SPR_BLK
        sta SPR_PTR,x
        lda BITMASK,x
        ora SPMASK
        sta SPMASK
.nf:    inx
        cpx #2
        bne .fig
        ; ---- Gegner (Sprites 2..6)
        lda #0
        sta ECUR
        ldx #2
.en:    cpx #7
        beq .item
        lda ECUR
        cmp ELIM
        bcs .item
        stx SPN
        ldy ECUR
        lda EN + E_X,y
        sta RT2
        lda EN + E_Y,y
        sta RT3
        lda EN + E_OX,y
        sta RT0
        lda EN + E_OY,y
        sta RT1
        jsr place
        ldy ECUR
        lda EN + E_KIND,y
        tay
        lda ENIMG,y                     ; Bild des Typs (Phase A)
        clc
        adc EPH
        asl                             ; * 2 Bloecke
        clc
        adc #SPR_BLK
        ldx SPN
        sta SPR_PTR,x
        lda BITMASK,x
        ora SPMASK
        sta SPMASK
        lda ECUR
        clc
        adc #E_SIZE
        sta ECUR
        inx
        jmp .en
        ; ---- Gegenstand (Sprite 7)
.item:
.if EXTRA_ITEMS
        lda IT_KIND
        beq .done
        lda IT_X
        sta RT0
        sta RT2
        lda IT_Y
        sta RT1
        sta RT3
        ldx #7
        jsr place
        lda IT_KIND
        asl
        clc
        adc #(SPR_BLK + 2 * 8)          ; Stern: Bild 9 = Block 18, Eisblock: Bild 10 = Block 20
        sta SPR_PTR + 7
        lda #$80
        ora SPMASK
        sta SPMASK
.endif
.done:  lda SPX8
        sta $d010
        lda SPX9
        sta $d05f
        lda SPY8
        sta $d077
        lda SPMASK
        sta $d015
        rts

ENIMG: .byte 3, 5, 7                    ; Bild je Typ (Phase A): Laeufer 3, Streuner 5, Waechter 7
}

