; =====================================================================================================================
; player.asm - Spieler: Eingabe, Bewegung, Faerben
; =====================================================================================================================
; Herkunft: examples/vic20/troxx/player.asm, Stand Commit 4697d074 (Bewegungsregeln und Selbstspiel unveraendert; Eingabe und Darstellung C64).
; ZWECK     Die Figur (ein Sprite mit zwei Bildern) laeuft zeichenweise auf den Linien des Gitters, solange eine Richtung gehalten wird; das Sprite
;           gleitet dabei in acht Punkten von Zelle zu Zelle (sprites.asm).
; BEDIENUNG Joystick im Anschluss des aktiven Spielers (JOYMODE).  Je Spielschritt (STEPLEN Bilder) ein Schritt.  Sind mehrere Richtungen gehalten,
;           gilt die erste mit freier Linie in der Reihenfolge hoch, runter, links, rechts.
; REGEL     Ein Schritt ist erlaubt, wenn die Zielzelle im Gitter liegt und eine Linienzelle ist (gx mod 4 = 0 oder gy mod 4 = 0): auf einer Linie
;           nur vor und zurueck, Abbiegen nur an Knoten.  Beim Betreten wird die Zelle gelb (grid.paint, Punkte, Segmente, Rechtecke).  Steht auf
;           der Zielzelle ein Gegner, ist das ein Treffer (HITF), ausser waehrend der Unverwundbarkeit (PINV > 0): dann bleibt die Figur stehen.
; HARDWARE  CIA 1: Joystick Anschluss 2 liest man an $DC00, Anschluss 1 an $DC01 (Bit 0 hoch, 1 runter, 2 links, 3 rechts, 4 Feuer; aktiv niedrig).
;           Beide Datenrichtungsregister stehen auf Eingang; die Tastatur wird nicht benutzt.  Achtung: gedrueckte Tasten verfaelschen die Abfrage
;           (die Tastaturmatrix liegt auf denselben Leitungen).
; AUTOPLAY  Mit -D AUTOPLAY=1 waehlt der Rechner die Richtung: an Knoten bevorzugt zu einer noch violetten Nachbarzelle, sonst zufaellig.
;           Zusaetzlich -D AUTOGOD=1: dauernd unverwundbar (Dauerlauf bis in hohe Stufen).
; =====================================================================================================================

player: {

; init: Figur auf den Startknoten unten links.  Veraendert A.
init:
        lda #0
        sta PX
        sta POX
        sta PANIM
        sta PDIR
        sta PINV
        lda #GRID_H - 1
        sta PY
        sta POY
        rts

.ifdef AUTOPLAY
; auto: waehlt PWANT fuer den naechsten Schritt.  Auf Linien geradeaus, an Knoten zu einer noch violetten Nachbarzelle, sonst zufaellig.
;   Veraendert A, X, Y, T0, T1, T4..T7, ATMP, PTR_A, PTR_B.
auto:   lda PX
        sta T0
        lda PY
        sta T1
        jsr grid.cell_type
        cmp #CT_NODE
        beq .node
        ldx PDIR
        lda DIR_BIT,x
        sta PWANT
        rts
.node:  lda PX
        sta T4
        lda PY
        sta T5
        lda PDIR
        sta T6
        jsr grid.legal_mask
        sta T7
        ldx #3
.d:     lda BITMASK,x
        and T7
        beq .nx
        stx ATMP
        lda PY
        clc
        adc DIR_DY,x
        tay
        lda PX
        clc
        adc DIR_DX,x
        tax
        jsr grid.color_at
        ldx ATMP
        cmp #COL_LINE
        beq .take                       ; Nachbarzelle noch violett
.nx:    dex
        bpl .d
        jsr enemies.pick_random         ; keine violette Nachbarzelle: zufaellig
.take:  lda DIR_BIT,x
        sta PWANT
        rts
.endif

; step: ein Spielschritt der Figur.  Setzt bei einem Treffer HITF.  Veraendert A, X, Y, T0..T7, PTR_A, PTR_B.
step:
        lda PX                          ; Zelle vor dem Schritt (fuer die Sprite-Bewegung)
        sta POX
        lda PY
        sta POY
.ifdef AUTOPLAY
        jsr auto
.endif
.ifdef AUTOGOD                          ; Dauerlauf-Test: immer unverwundbar (Treffer entfallen, Stufen werden erreichbar)
        lda #100
        sta PINV
.endif
        lda PINV
        beq .go
        dec PINV
.go:    ldx #0                          ; Richtung 0 hoch, 1 runter, 2 links, 3 rechts
.try:   lda PWANT
        and DIR_BIT,x
        beq .nx
        lda PX                          ; Zielzelle
        clc
        adc DIR_DX,x
        sta T4
        lda PY
        clc
        adc DIR_DY,x
        sta T5
        cmp #GRID_H                     ; -1 wird $FF und faellt hier ebenfalls heraus
        bcs .nx
        lda T4
        cmp #GRID_W
        bcs .nx
        and #3
        beq .free                       ; gx mod 4 = 0: senkrechte Linie (oder Knoten)
        ldy T5
        lda ROW_TPL,y
        cmp #TPL_INNER
        bne .free                       ; gy mod 4 = 0: waagerechte Linie
.nx:    inx
        cpx #4
        bne .try
        rts                             ; keine freie Richtung: stehen bleiben
.free:  stx T6
        ldx T4
        ldy T5
        jsr enemies.find
        bcc .move
        lda PINV                        ; Gegner auf der Zielzelle
        bne .stay
        lda #1
        sta HITF
        rts
.stay:  lda T6                          ; unverwundbar: stehen bleiben, die Richtung merken (AUTOPLAY dreht so um)
        eor #1
        sta PDIR
        rts
.move:  lda T4
        sta PX
        sta T0
        lda T5
        sta PY
        sta T1
        lda T6
        sta PDIR
        jsr grid.paint
        jsr grid.paint_node             ; Knoten faerben sich beim Betreten gelb (nur Optik)
        lda PANIM
        eor #1
        sta PANIM
.if EXTRA_ITEMS
        jsr items.check                 ; Gegenstand aufgenommen?
.endif
        lda #Fx.step
        jmp sound.play
}
