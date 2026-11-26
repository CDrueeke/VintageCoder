; =====================================================================================================================
; player.asm - Spieler: Eingabe, Bewegung, Faerben (ein oder zwei Figuren)
; =====================================================================================================================
; Herkunft: examples/c64/troxx/player.asm und examples/vic20/troxx/player.asm, Stand Commit 4697d074 der Referenz (Bewegungsregeln und Selbstspiel
;   unveraendert; Eingabe und Darstellung MEGA65).  Neu: Figuren als Felder (PX..POY je Figur), Besitzer OWN, Farbe und Punkte nach WHO.
; ZWECK     Jede Figur (ein Sprite mit zwei Bildern) laeuft zeichenweise auf den Linien des Gitters, solange eine Richtung gehalten wird; das Sprite
;           gleitet dabei in acht Punkten von Zelle zu Zelle (sprites.asm).  Mit zwei Spielern gleichzeitig gibt es zwei Figuren (FIG 0 und 1).
; BEDIENUNG Joystick im Anschluss der Figur (input.asm).  Je Spielschritt (STEPLEN Bilder) ein Schritt.  Sind mehrere Richtungen gehalten, gilt die
;           erste mit freier Linie in der Reihenfolge hoch, runter, links, rechts.
; REGEL     Ein Schritt ist erlaubt, wenn die Zielzelle im Gitter liegt und eine Linienzelle ist (gx mod 5 = 0 oder gy mod 6 = 0): auf einer Linie
;           nur vor und zurueck, Abbiegen nur an Knoten.  Beim Betreten wird die Zelle in der Farbe des Spielers gefaerbt (grid.paint, Punkte, Segmente,
;           Rechtecke).  Steht auf der Zielzelle ein Gegner, ist das ein Treffer (HITF, Bit der Figur), ausser waehrend der Unverwundbarkeit (PINV > 0).
; AUTOPLAY  Mit -D AUTOPLAY=1 waehlt der Rechner die Richtung: an Knoten bevorzugt zu einer noch unberuehrten Nachbarzelle, sonst zufaellig.
;           Zusaetzlich -D AUTOGOD=1: dauernd unverwundbar (Dauerlauf bis in hohe Stufen).
; =====================================================================================================================

player: {

; init: Figuren auf die Startknoten (Figur 0 unten links, Figur 1 unten rechts) und Besitzer setzen.  Veraendert A, X.
init:   ldx #1
.f:     lda #0
        sta POX,x
        sta PANIM,x
        sta PDIR,x
        sta PINV,x
        sta PWANT,x
        lda #GRID_H - 1
        sta PY,x
        sta POY,x
        dex
        bpl .f
        lda #0
        sta PX
        sta POX
        lda #GRID_W - 1
        sta PX + 1
        sta POX + 1
        lda #1
        sta PACT                        ; Figur 0 ist immer im Spiel
        lda #0
        sta PACT + 1
        lda NFIG
        cmp #2
        bne .own
        lda #1
        sta PACT + 1
.own:   lda #0
        sta OWN
        sta OWN + 1
        lda NFIG
        cmp #2
        beq .two
        lda CUR                         ; ein Spieler am Zug: Figur 0 gehoert dem Spieler am Zug
        sta OWN
        rts
.two:   lda #1
        sta OWN + 1
        rts

;  respawn: nach einem Treffer alle Figuren zurueck auf die Startknoten (PACT und Besitzer bleiben), kurze Unverwundbarkeit.  Veraendert A, X.
respawn:
        ldx #1
.f:     lda #0
        sta PDIR,x
        sta PANIM,x
        sta PWANT,x
        lda #GRID_H - 1
        sta PY,x
        sta POY,x
        lda #INVULN
        sta PINV,x
        dex
        bpl .f
        lda #0
        sta PX
        sta POX
        lda #GRID_W - 1
        sta PX + 1
        sta POX + 1
        rts

.ifdef AUTOPLAY
; auto: waehlt PWANT der Figur FIG fuer den naechsten Schritt.  Auf Linien geradeaus, an Knoten zu einer noch unberuehrten Nachbarzelle, sonst
;   zufaellig.  Veraendert A, X, Y, T0, T1, T4..T7, ATMP, PTR_B.
auto:   ldx FIG
        lda PX,x
        sta T0
        lda PY,x
        sta T1
        jsr grid.cell_type
        cmp #CT_NODE
        beq .node
        ldx FIG
        ldy PDIR,x
        lda DIR_BIT,y
        sta PWANT,x
        rts
.node:  ldx FIG
        lda PX,x
        sta T4
        lda PY,x
        sta T5
        lda PDIR,x
        sta T6
        jsr grid.legal_mask
        sta T7
        ldx #3
.d:     lda BITMASK,x
        and T7
        beq .nx
        stx ATMP
        ldy FIG
        lda PY,y
        clc
        adc DIR_DY,x
        sta T1
        lda PX,y
        clc
        adc DIR_DX,x
        tax
        ldy T1
        jsr grid.color_at
        ldx ATMP
        cmp #CS_LINE
        beq .take                       ; Nachbarzelle noch unberuehrt
.nx:    dex
        bpl .d
        jsr enemies.pick_random         ; keine unberuehrte Nachbarzelle: zufaellig
.take:  ldy FIG
        lda DIR_BIT,x
        sta PWANT,y
        rts
.endif

; step: ein Spielschritt der Figur FIG.  Setzt bei einem Treffer das Bit der Figur in HITF.  Veraendert A, X, Y, T0..T7, PTR_B.
step:   ldx FIG
        lda PX,x                        ; Zelle vor dem Schritt (fuer die Sprite-Bewegung)
        sta POX,x
        lda PY,x
        sta POY,x
.ifdef AUTOPLAY
        jsr auto
.endif
.ifdef AUTOGOD                          ; Dauerlauf-Test: immer unverwundbar (Treffer entfallen, Stufen werden erreichbar)
        ldx FIG
        lda #100
        sta PINV,x
.endif
        ldx FIG
        lda PINV,x
        beq .go
        dec PINV,x
.go:    ldx #0                          ; Richtung 0 hoch, 1 runter, 2 links, 3 rechts
.try:   ldy FIG
        lda PWANT,y
        and DIR_BIT,x
        beq .nx
        lda PX,y                        ; Zielzelle
        clc
        adc DIR_DX,x
        sta T4
        lda PY,y
        clc
        adc DIR_DY,x
        sta T5
        cmp #GRID_H                     ; -1 wird $FF und faellt hier ebenfalls heraus
        bcs .nx
        lda T4
        cmp #GRID_W
        bcs .nx
        ldy T4
        lda MOD5,y
        beq .free                       ; gx mod 5 = 0: senkrechte Linie (oder Knoten)
        ldy T5
        lda MOD6,y
        beq .free                       ; gy mod 6 = 0: waagerechte Linie
.nx:    inx
        cpx #4
        bne .try
        rts                             ; keine freie Richtung: stehen bleiben
.free:  stx T6
        ldx T4
        ldy T5
        jsr enemies.find
        bcc .move
        ldy FIG
        lda PINV,y                      ; Gegner auf der Zielzelle
        bne .stay
        lda BITMASK,y
        ora HITF
        sta HITF
        rts
.stay:  lda T6                          ; unverwundbar: stehen bleiben, die Richtung merken (AUTOPLAY dreht so um)
        eor #1
        sta PDIR,y
        rts
.move:  ldy FIG
        lda T4
        sta PX,y
        sta T0
        lda T5
        sta PY,y
        sta T1
        lda T6
        sta PDIR,y
        lda OWN,y
        sta WHO
        jsr grid.paint
        ldy FIG
        lda PX,y
        sta T0
        lda PY,y
        sta T1
        jsr grid.paint_node             ; Knoten faerben sich beim Betreten (nur Optik)
        ldy FIG
        lda PANIM,y
        eor #1
        sta PANIM,y
.if EXTRA_ITEMS
        jsr items.check                 ; Gegenstand aufgenommen?
.endif
        lda #Fx.step
        jmp sound.play
}

