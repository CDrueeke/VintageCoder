; =====================================================================================================================
; sprites.asm - Hardware-Sprites: Figur, bis zu vier Gegner, Gegenstand; Pixelbewegung zwischen den Zellen
; =====================================================================================================================
; ZWECK     Die Spiellogik rechnet in Zeichenzellen (gx, gy) und kennt nur Schritte von Zelle zu Zelle.  Die Sprites zeigen diese Schritte glatt:
;           Jede Figur hat ihre Zelle vor dem Schritt (POX/POY, E_OX/E_OY) und ihre Zelle danach (PX/PY, E_X/E_Y).  Im Spielzustand laeuft das Sprite
;           waehrend der STEPLEN Bilder eines Schritts um FRAME * 8 / STEPLEN Punkte von der alten zur neuen Zelle (SPPROG = 0..7); in allen anderen
;           Zustaenden steht es auf der neuen Zelle (SPPROG = 8).  Entscheidungen und Kollisionen bleiben auf Zeichenebene.
; ZUORDNUNG Sprite 0 Figur, Sprite 1..4 Gegner (EN), Sprite 6 Gegenstand (Stern oder Eisblock).  Sprite-Daten: Bloecke SPR_BLK + 0/1 Figur (Bild 1/2),
;           + 2 Treffer, + 3..8 Gegner (Typ * 2 + Bild), + 9 Stern, + 10 Eisblock (sprites.inc, erzeugt von tools/gen_sprites.py).
; HARDWARE  Sprite-Position (X: $D000 + 2 n, Bit 8 in $D010; Y: $D001 + 2 n); ein Zeichen (gx, gy) liegt bei X = 24 + 8 (gx + GRID_COL0), Y = 50 + 8 (gy + GRID_ROW0).
;           Die Bilder sind 16 x 16 Punkte, mittig im 24 x 21-Feld; darum X = 16 + 8 (gx + GRID_COL0), Y = 8 gy + 60 (Tabellen XLO, XHI, YPOS).
;           Die Register werden nur am Bildanfang (Rasterzeile 251, unter dem Bild) beschrieben: keine Risse.
; =====================================================================================================================

sprites: {

; hide: alle Sprites aus.  Veraendert A.
hide:   lda #0
        sta VIC_SPR_ENABLE
        rts

; update: einmal je Bild nach wait_frame: Positionen, Bilder, Farben und Freigabe setzen.  Veraendert A, X, Y, SPT0..SPT6, SPN.
update:
        lda SPOFF
        bne .off
        lda STATE
        cmp #St.title
        bne .on
.off:   lda #0
        sta VIC_SPR_ENABLE
        rts
.on:    cmp #St.playing
        beq .calc
        lda #8                          ; ausserhalb des Spiels: Sprite steht auf der neuen Zelle
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
        lda #0
        sta SPMSB
        ; ---- Figur (Sprite 0)
        lda POX
        sta SPT0
        lda PX
        sta SPT1
        lda POY
        sta SPT2
        lda PY
        sta SPT3
        ldx #0
        jsr place
        lda SPHIT
        beq .anim
        lda #2                          ; Treffer-Bild, rot
        ldx #COL_RED
        bne .pset
.anim:  lda PANIM
        ldx #COL_LIGHTGRAY
        ldy PINV
        beq .pset
        tya
        and #1
        beq .pw
        ldx #COL_CYAN                   ; Unverwundbarkeit: weiss/cyan blinken
.pw:    lda PANIM
.pset:  clc
        adc #SPR_BLK
        sta SPRITE_PTRS
        stx VIC_SPR0_COLOR
        ; ---- Gegner (Sprite 1..4)
        lda #1
        sta SPN
        ldx #0                          ; Abstand in EN
.en:    cpx ELIM
        bcs .items
        stx SPT6
        lda EN + E_OX,x
        sta SPT0
        lda EN + E_X,x
        sta SPT1
        lda EN + E_OY,x
        sta SPT2
        lda EN + E_Y,x
        sta SPT3
        ldx SPN
        jsr place
        ldx SPT6
        lda EN + E_KIND,x
        tay
        asl
        clc
        adc EPH
        adc #SPR_BLK + 3
        ldx SPN
        sta SPRITE_PTRS,x
        lda FREEZE
        ora FREEZE + 1
        beq .ncol
        lda #COL_LIGHTBLUE              ; starr: hellblau
        bne .col
.ncol:  lda KIND_COL,y
.col:   sta VIC_SPR0_COLOR,x
        inc SPN
        lda SPT6
        clc
        adc #E_SIZE
        tax
        bne .en
.items:
.if EXTRA_ITEMS
        ; ---- Gegenstand (Sprite 6)
        lda IT_KIND
        beq .noit
        lda IT_X
        sta SPT0
        sta SPT1
        lda IT_Y
        sta SPT2
        sta SPT3
        ldx #6
        jsr place
        ldx IT_KIND                     ; 1 Stern (gelb), 2 Eisblock (hellblau)
        lda #SPR_BLK + 8
        clc
        adc IT_KIND
        sta SPRITE_PTRS + 6
        lda ITEM_COL - 1,x
        sta VIC_SPR6_COLOR
.noit:
.endif
        lda SPMSB
        sta VIC_SPR_X_MSB
        ; ---- Freigabe: Figur, NENEMY Gegner, Gegenstand
        ldx NENEMY
        lda ENMASK,x
        ldx IT_KIND
        beq .en2
        ora #$40
.en2:   sta VIC_SPR_ENABLE
        rts

; place: Sprite X an die Stelle zwischen den Zellen (SPT0, SPT2) und (SPT1, SPT3) setzen: Zelle vor dem Schritt plus SPPROG Punkte in
;   Richtung der neuen Zelle.  Setzt Bit 8 der X-Koordinate in SPMSB.  Veraendert A, X, Y, SPT4, SPT5.
place:  txa
        asl
        tay                             ; Y = 2 * Sprite (Registerabstand)
        ldx SPT0
        lda XLO,x
        sta SPT4
        lda XHI,x
        sta SPT5
        lda SPT1
        cmp SPT0
        beq .xd
        bcc .xneg
        lda SPT4
        clc
        adc SPPROG
        sta SPT4
        bcc .xd
        inc SPT5
        bne .xd
.xneg:  lda SPT4
        sec
        sbc SPPROG
        sta SPT4
        bcs .xd
        dec SPT5
.xd:    lda SPT4
        sta VIC_SPR0_X,y
        ldx SPT2
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
.yd:    sta VIC_SPR0_Y,y
        lda SPT5
        beq .out
        tya
        lsr
        tax
        lda BITMASK,x
        ora SPMSB
        sta SPMSB
.out:   rts
}

; ---- Daten: Sprite-Vorlagen (11 * 64 Byte, werden von vic.init nach SPRITE_RAM kopiert)
SPRITE_DATA:
.include "sprites.inc"
