; =====================================================================================================================
; player.asm - Spieler: Eingabe, Bewegung, Faerben
; =====================================================================================================================
; ZWECK     Die Figur (ein Zeichen, zwei Bilder) laeuft zeichenweise auf den Linien des Gitters, solange eine Richtung gehalten wird.
; BEDIENUNG Joystick im Anschluss.  Je Spielschritt (STEPLEN Bilder) ein Schritt.  Sind mehrere Richtungen gehalten, gilt die erste mit freier
;           Linie in der Reihenfolge hoch, runter, links, rechts.
; REGEL     Ein Schritt ist erlaubt, wenn die Zielzelle im Gitter liegt und eine Linienzelle ist (gx mod 4 = 0 oder gy mod 5 = 0): auf einer Linie
;           nur vor und zurueck, Abbiegen nur an Knoten.  Beim Betreten wird die Zelle gelb (grid.paint, Punkte, Segmente, Rechtecke); die
;           verlassene Zelle bekommt ihr Linienzeichen zurueck, in Gelb.  Steht auf der Zielzelle ein Gegner, ist das ein Treffer (HITF), ausser
;           waehrend der Unverwundbarkeit (PINV > 0): dann bleibt die Figur stehen.
; HARDWARE  VIA 1 $9111 Bit 2 hoch, 3 runter, 4 links, 5 Feuer (aktiv niedrig).  Rechts liegt auf VIA 2 $9120 Bit 7 (aktiv niedrig); dazu muss die
;           Datenrichtung $9122 Bit 7 Eingang sein.  Die Tastatur wird nicht benutzt, darum steht $9122 ganz auf Eingang.
; AUTOPLAY  Mit -D AUTOPLAY=1 waehlt der Rechner die Richtung: an Knoten bevorzugt zu einer noch blauen Nachbarzelle, sonst zufaellig.
;           Zusaetzlich -D AUTOGOD=1: dauernd unverwundbar (Dauerlauf bis in hohe Stufen).
; =====================================================================================================================

player: {

; setup: Datenrichtungen der Eingabe setzen (einmal beim Start).  Veraendert A.
setup:
        lda #0
        sta VIA2_DDRB                   ; Port B = Eingang: Bit 7 liest die Joystick-Richtung rechts
        lda VIA1_DDRA
        and #%11000011                  ; Bit 2..5 (hoch, runter, links, Feuer) = Eingang
        sta VIA1_DDRA
        rts

; init: Figur auf den Startknoten unten links, zeichnen.  Veraendert A, X, Y, T0, T1, PTR_A, PTR_B.
init:
        lda #0
        sta PX
        sta PANIM
        sta PDIR
        sta PINV
        lda #GRID_H - 1
        sta PY
        ; weiter mit draw

; draw: Figur an (PX, PY) zeichnen, Bild PANIM; waehrend der Unverwundbarkeit blinkt sie (weiss/cyan).  Veraendert A, X, Y, T0, T1, PTR_A, PTR_B.
draw:   lda PX
        sta T0
        lda PY
        sta T1
        ldx #COL_WHITE
        lda PINV
        and #1
        beq .col
        ldx #COL_CYAN
.col:   lda PANIM
        clc
        adc #CH_PLAYER
        jmp grid.put

; erase: Figur entfernen: die Zelle bekommt ihr Linienzeichen zurueck, gelb.  Veraendert A, X, Y, T0, T1, PTR_A, PTR_B.
erase:  lda PX
        sta T0
        ldy PY
        sty T1
        ldx PX
        jsr grid.char_at
        ldx #COL_DONE
        jmp grid.put

; read_joy: A = Maske der gedrueckten Richtungen und Feuer (IN_*, 1 = gedrueckt).  Veraendert A, X.
read_joy:
        lda VIA1_ORA
        eor #$ff                        ; gedrueckt = 1
        lsr
        lsr
        and #$0f                        ; Bit 0 hoch, 1 runter, 2 links, 3 Feuer
        ldx VIA2_ORB
        bmi .done                       ; Bit 7 = 1: rechts nicht gedrueckt
        ora #IN_RIGHT
.done:  rts

.ifdef AUTOPLAY
; auto: waehlt PWANT fuer den naechsten Schritt.  Auf Linien geradeaus, an Knoten zu einer noch blauen Nachbarzelle, sonst zufaellig.
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
        beq .take                       ; Nachbarzelle noch blau
.nx:    dex
        bpl .d
        jsr enemies.pick_random         ; keine blaue Nachbarzelle: zufaellig
.take:  lda DIR_BIT,x
        sta PWANT
        rts
.endif

; step: ein Spielschritt der Figur.  Setzt bei einem Treffer HITF.  Veraendert A, X, Y, T0..T7, PTR_A, PTR_B.
step:
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
        lda ROWKIND,y
        bpl .free                       ; gy mod 5 = 0: waagerechte Linie
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
.move:  jsr erase
        lda T4
        sta PX
        sta T0
        lda T5
        sta PY
        sta T1
        lda T6
        sta PDIR
        jsr grid.paint
        lda PANIM
        eor #1
        sta PANIM
        jsr draw
        lda #Fx.step
        jmp sound.play
}
