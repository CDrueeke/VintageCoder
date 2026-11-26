; =====================================================================================================================
; figs.asm - Figuren als Zeichen: Spieler, Gegner und Gegenstand auf dem Gitter zeichnen (der Plus/4 hat keine Sprites)
; =====================================================================================================================
; ZWECK     Sechs Plaetze: 0 Spieler, 1..4 Gegner, 5 Gegenstand (Stern oder Eisblock).  update laeuft je Bild (aus tick) und
;           1. stellt die zuvor gezeichnete Zelle eines Platzes aus der Schattenkopie des Gitters wieder her, wenn der Platz die Zelle gewechselt hat oder
;              verschwunden ist (der Bildschirm einer Zelle ohne Figur ist immer die Schattenkopie),
;           2. zeichnet danach den Gegenstand, die Gegner und zuletzt den Spieler (er liegt oben) mit ihrem Bild und ihrer Farbe.
;           Das Bild wechselt alle acht Bilder (vier Animationsbilder; Spieler, Laeufer, Streuner, Waechter haben je vier Zeichen); die Farbe wechselt mit:
;           der Spieler ist weiss (cyan im Wechsel, solange er unverwundbar ist; rot und Treffer-Zeichen im Zustand HIT), eingefrorene Gegner (Eisblock) sind
;           hellblau, der Gegenstand blinkt.  Die Spiellogik kennt nur Zellen; hier liegt allein die Darstellung.
; DATEN     FIG_OX, FIG_OY (RAM): gezeichnete Zelle je Platz, $FF = nichts gezeichnet.  FIGOFF = 1 verbirgt alle Figuren.
; =====================================================================================================================

figs: {

; update: einmal je Bild.  Veraendert A, X, Y, T0..T3, PTR_A, PTR_B, PTR_D, PTR_E.
update:
        lda FRAMEC
        lsr
        lsr
        lsr
        and #3
        sta FIG_T                       ; Bild 0..3
        ldx #5                          ; 1. Schritt: Zellen wiederherstellen, deren Figur nicht mehr dort steht
.restore:
        lda FIG_OX,x
        cmp #$ff
        beq .nx
        jsr want                        ; T0 = gewuenschte Zelle x ($FF = keine), T1 = y
        lda T0
        cmp FIG_OX,x
        bne .rest
        lda T1
        cmp FIG_OY,x
        beq .nx                         ; unveraendert
.rest:  stx FIG_T + 1
        lda FIG_OX,x
        ldy FIG_OY,x
        tax
        jsr restore                     ; X = gx, Y = gy
        ldx FIG_T + 1
        lda #$ff
        sta FIG_OX,x
.nx:    dex
        bpl .restore
        ldx #5                          ; 2. Schritt: zeichnen (Gegenstand zuerst, Spieler zuletzt)
.draw:  jsr want
        lda T0
        cmp #$ff
        beq .skip
        stx FIG_T + 1
        jsr shape                       ; A = Zeichen, T2 = Farbe
        ldy T1
        ldx T0
        jsr put
        ldx FIG_T + 1
        lda T0
        sta FIG_OX,x
        lda T1
        sta FIG_OY,x
.skip:  dex
        bne .draw
        ; Platz 0 (Spieler) zuletzt
        jsr want
        lda T0
        cmp #$ff
        beq .out
        stx FIG_T + 1                   ; X = 0
        jsr shape
        ldy T1
        ldx T0
        jsr put
        lda T0
        sta FIG_OX
        lda T1
        sta FIG_OY
.out:   rts

; want: gewuenschte Zelle des Platzes X: T0 = gx ($FF = nichts), T1 = gy.  Veraendert A, T0, T1.  X bleibt.
want:   lda FIGOFF
        bne .none
        cpx #0
        beq .player
        cpx #5
        beq .item
        txa                             ; Gegner 1..4: nur die vorhandenen (NENEMY)
        cmp NENEMY
        beq .en
        bcs .none
.en:    txa                             ; Abstand in EN = (Nummer - 1) * E_SIZE

        asl
        sta T1
        asl
        clc
        adc T1                          ; 6 * Nummer
        sec
        sbc #E_SIZE
        tay
        lda EN + E_X,y
        sta T0
        lda EN + E_Y,y
        sta T1
        rts
.player:
        lda PX
        sta T0
        lda PY
        sta T1
        rts
.item:
.if EXTRA_ITEMS
        lda IT_KIND
        beq .none
        lda IT_X
        sta T0
        lda IT_Y
        sta T1
        rts
.endif
.none:  lda #$ff
        sta T0
        rts

; shape: Zeichen (A) und Farbe (T2) fuer Platz X.  Veraendert A, Y, T2, T3.  X bleibt.
shape:  cpx #0
        beq .player
        cpx #5
        beq .item
        txa                             ; Gegner: Typ aus EN
        asl
        sta T3
        asl
        clc
        adc T3
        sec
        sbc #E_SIZE
        tay
        lda EN + E_KIND,y
        tay
        lda KIND_COL,y
        sta T2
.if EXTRA_ICE
        lda FREEZE
        ora FREEZE + 1
        beq .nofz
        lda #COL_LTBLUE                 ; eingefroren
        sta T2
.nofz:
.endif
        lda KIND_BASE,y                 ; erstes Zeichen des Typs
        clc
        adc FIG_T                       ; + Bild
        rts
.player:
        lda SPHIT
        beq .pl
        lda #COL_RED
        sta T2
        lda #CH_HIT
        rts
.pl:    lda #COL_WHITE
        sta T2
        lda PINV
        beq .pc
        lda FRAMEC                      ; unverwundbar: weiss und cyan im Wechsel (je zwei Bilder)
        and #2
        beq .pc
        lda #COL_CYAN
        sta T2
.pc:    lda FIG_T
        clc
        adc #CH_PLAYER
        rts
.item:
.if EXTRA_ITEMS
        ldy IT_KIND
        lda ITEM_COL - 1,y
        sta T2
        lda FRAMEC                      ; blinkt: im Wechsel Farbe und weiss (je vier Bilder)
        and #4
        beq .it
        lda #COL_WHITE
        sta T2
.it:    lda ITEM_CH - 1,y
        rts
.endif
        rts

; restore: Zelle X = gx, Y = gy aus der Schattenkopie auf den Bildschirm zurueckschreiben.  Veraendert A, Y, PTR_A, PTR_B, PTR_D, PTR_E.
restore:
        stx T0
        jsr grid.set_row
        ldy T0
        lda (PTR_D),y
        sta (PTR_A),y
        lda (PTR_E),y
        sta (PTR_B),y
        rts

; put: Zeichen A und Farbe T2 auf die Zelle X = gx, Y = gy des Bildschirms schreiben (die Schattenkopie bleibt).  Veraendert A, Y, T0, PTR_A, PTR_B, PTR_D, PTR_E.
put:    stx T0
        pha
        jsr grid.set_row
        ldy T0
        pla
        sta (PTR_A),y
        lda T2
        sta (PTR_B),y
        rts
}

KIND_BASE: .byte CH_ENEMY0, CH_ENEMY0 + 4, CH_ENEMY0 + 8
ITEM_CH:   .byte CH_STAR, CH_ICE
