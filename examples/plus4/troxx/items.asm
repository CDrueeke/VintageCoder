; =====================================================================================================================
; items.asm - Zusatzfunktionen: Bonusstern und Eisblock (nur mit EXTRA_STAR / EXTRA_ICE)
; =====================================================================================================================
; ZWECK     Ab und zu erscheint auf einer Linie ein Gegenstand (als Zeichen von figs.asm gezeichnet): Stern (Zusatzpunkte, 50) oder Eisblock (alle Gegner stehen fuenf Sekunden
;           still, die Figur laeuft weiter).  Er verschwindet nach zehn Sekunden, wenn ihn die Figur nicht aufnimmt; danach vergehen vier bis fuenf
;           Sekunden bis zum naechsten.  Alle Zeiten zaehlt tick in Bildern (nur im Spielzustand; die Tabellen LIFE und FROZEN gelten je Norm).
;           Mit beiden Zusatzfunktionen ist jeder vierte Gegenstand ein Eisblock.
; DATEN     IT_KIND (0 keiner, 1 Stern, 2 Eisblock), IT_X, IT_Y, IT_TIME (Sichtzeit), IT_WAIT (Wartezeit), FREEZE (Rest der Starre), alle im RAM
;           ab VARS + $20 (main.asm).  Zufall aus dem Zufallsgenerator der Gegner (enemies.rnd).
; =====================================================================================================================

items: {

; reset: Gegenstand entfernen, Starre beenden, Wartezeit neu.  Veraendert A.
reset:  lda #0
        sta IT_KIND
        sta FREEZE
        sta FREEZE + 1
        ; weiter mit set_wait

; set_wait: Wartezeit bis zum naechsten Gegenstand = 4 Sekunden (FPS * 4) plus 0..63 Bilder.  Veraendert A.
set_wait:
        lda RND
        and #63
        sta T0
        lda FPS
        asl
        asl
        clc
        adc T0
        sta IT_WAIT
        lda #0
        adc #0
        sta IT_WAIT + 1
        rts

; norm_idx: X = 0 (PAL) oder 1 (NTSC).  Veraendert A, X.
norm_idx:
        lda FLAGS
        and #F_PAL
        eor #F_PAL
        tax
        rts

; tick: einmal je Bild im Spielzustand: Starre zaehlen, Sichtzeit und Wartezeit zaehlen, Gegenstand erscheinen lassen.  Veraendert A, X, T0, T1.
tick:
.if EXTRA_ICE
        lda FREEZE
        ora FREEZE + 1
        beq .nofz
        lda FREEZE
        bne .f1
        dec FREEZE + 1
.f1:    dec FREEZE
.nofz:
.endif
        lda IT_KIND
        bne .active
        lda IT_WAIT                     ; wartet auf den naechsten Gegenstand
        ora IT_WAIT + 1
        beq .spawn
        lda IT_WAIT
        bne .w1
        dec IT_WAIT + 1
.w1:    dec IT_WAIT
        rts
.active:
        lda IT_TIME
        bne .t1
        dec IT_TIME + 1
.t1:    dec IT_TIME
        lda IT_TIME
        ora IT_TIME + 1
        bne .out
        lda #0                          ; Sichtzeit abgelaufen
        sta IT_KIND
        jmp set_wait
.out:   rts
.spawn:                                 ; neuen Gegenstand setzen: Typ, dann eine freie Linienzelle
        lda #1
.if EXTRA_STAR && EXTRA_ICE
        sta T2
        jsr enemies.rnd
        and #3
        bne .kind
        lda #2
        sta T2
.kind:
.else
.if EXTRA_ICE
        lda #2
.endif
        sta T2
.endif
.try:   jsr enemies.rnd
        and #63
        cmp #GRID_W
        bcs .try
        sta T0
.ty:    jsr enemies.rnd
        and #31
        cmp #GRID_H
        bcs .ty
        sta T1
        jsr grid.cell_type
        beq .try                        ; Innenraum: nochmal
        lda T0
        cmp PX
        bne .ok1
        lda T1
        cmp PY
        beq .try                        ; nicht auf der Figur
.ok1:   ldx T0
        ldy T1
        jsr enemies.find
        bcs .try                        ; nicht auf einem Gegner
        lda T0
        sta IT_X
        lda T1
        sta IT_Y
        lda T2
        sta IT_KIND
        jsr norm_idx
        lda LIFE_LO,x
        sta IT_TIME
        lda LIFE_HI,x
        sta IT_TIME + 1
        rts

; check: die Figur hat eine Zelle betreten (PX, PY): liegt dort der Gegenstand, aufnehmen.  Veraendert A, X, Y.
check:  lda IT_KIND
        beq .out
        lda PX
        cmp IT_X
        bne .out
        lda PY
        cmp IT_Y
        bne .out
        ldx IT_KIND
        lda #0
        sta IT_KIND
.if EXTRA_STAR
        cpx #1
        bne .ice
        lda #$50                        ; Stern: 50 Punkte
        ldx #0
        jsr hud.bcd_add
        lda #Fx.bonus
        jsr sound.play
        jmp set_wait
.endif
.ice:
.if EXTRA_ICE
        jsr norm_idx                    ; Eisblock: Gegner stehen fuenf Sekunden still
        lda FROZEN_LO,x
        sta FREEZE
        lda FROZEN_HI,x
        sta FREEZE + 1
        lda #Fx.ice
        jsr sound.play
.endif
        jmp set_wait
.out:   rts

LIFE_LO:   .byte <500, <600             ; Sichtzeit 10 Sekunden in Bildern (PAL, NTSC)
LIFE_HI:   .byte >500, >600
FROZEN_LO: .byte <250, <300             ; Starre 5 Sekunden
FROZEN_HI: .byte >250, >300
}
