; =====================================================================================================================
; enemies.asm - Gegner: Zufall, Entscheidung, Bewegung
; =====================================================================================================================
; ZWECK     1 bis 4 Gegner laufen zeichenweise auf den Linien, je Spielschritt ein Schritt.  Entscheidungen fallen nur an Knoten: erlaubt ist jede
;           Richtung im Gitter ausser der Umkehr.  Zwei Typen: Laeufer (mit Wahrscheinlichkeit p, nach Stufe 50 bis 90 %, die Richtung, die den
;           Abstand zur Figur verkleinert (zufaellig eine der Richtungen auf die Figur zu); sonst Zufall) und Streuner (immer Zufall).
;           Gegner liegen nie auf derselben Zelle wie die Figur oder ein anderer Gegner: ein Schritt auf eine besetzte Zelle faellt aus; trifft
;           ein Gegner so auf die Figur, ist das ein Treffer (HITF), ausser waehrend der Unverwundbarkeit.
; DATEN     EN: je Gegner x, y, Richtung, Typ, Farbe der Zelle darunter (Zustand der Linie, siehe grid.asm).  Gegner i hat Typ i mod 2.
; ZUFALL    8-Bit-LFSR (Galois, Rueckkopplung $B8, Periode 255), fester Startwert je Stufe.
; =====================================================================================================================

enemies: {

; rnd: naechste Zufallszahl.  A = RND (nie 0).
rnd:    lda RND
        lsr
        bcc .ok
        eor #$b8
.ok:    sta RND
        rts

; level_idx: Y = Stufe 1..9 als Tabellenindex 0..8 (Stufe ab 10 gilt wie 9; LEVEL ist BCD).  Veraendert A, Y.
level_idx:
        lda LEVEL
        cmp #10
        bcc .ok
        lda #9
.ok:    tay
        dey
        rts

; nxt: naechster Gegner (ECUR).  C = 1 nach dem letzten.  Veraendert A.
nxt:    lda ECUR
        clc
        adc #E_SIZE
        sta ECUR
        cmp ELIM
        rts

; place_all: Gegnerzahl der Stufe setzen, Gegner auf die Startknoten, zeichnen.  Veraendert A, X, Y, T0, T1, ECUR, PTR_A, PTR_B.
place_all:
        jsr level_idx
        lda COUNT,y
        sta ELIM                        ; Gegnerzahl * 5 (Tabelle)
        lda #0
        sta ECUR
        sta EPH
        sta T2                          ; Nummer des Gegners
.l:     ldx ECUR
        ldy T2
        lda #0
        sta EN + E_DIR,x
        lda START_X,y
        sta EN + E_X,x
        lda START_Y,y
        sta EN + E_Y,x
        tya
        and #1                          ; Typ: gerade Nummer Laeufer, ungerade Streuner
        sta EN + E_KIND,x
        ldy EN + E_Y,x
        lda EN + E_X,x
        tax
        jsr grid.color_at
        ldx ECUR
        sta EN + E_UNDER,x
        jsr draw_one
        inc T2
        jsr nxt
        bcc .l
        rts

; draw_one: Gegner ECUR in seinem Bild und seiner Typfarbe zeichnen.  Veraendert A, X, Y, T0, T1, PTR_A, PTR_B.
draw_one:
        ldx ECUR
        lda EN + E_X,x
        sta T0
        lda EN + E_Y,x
        sta T1
        ldy EN + E_KIND,x
        lda KIND_COL,y
        tax
        lda EPH
        asl
        clc
        adc #CH_ENEMY_A
        jmp grid.put

; erase_one: Gegner ECUR entfernen: Linienzeichen und gemerkte Farbe zurueck.  Veraendert A, X, Y, T0, T1, PTR_A, PTR_B.
erase_one:
        ldx ECUR
        lda EN + E_X,x
        sta T0
        lda EN + E_Y,x
        sta T1
        tay
        ldx T0
        jsr grid.char_at                ; Zeichen der Linie
        sta T2
        ldx ECUR
        lda EN + E_UNDER,x              ; Farbe
        tax
        lda T2
        jmp grid.put

; erase_all: alle Gegner entfernen.  Veraendert A, X, Y, T0, T1, ECUR, PTR_A, PTR_B.
erase_all:
        lda #0
        sta ECUR
.l:     jsr erase_one
        jsr nxt
        bcc .l
        rts

; find: X = gx, Y = gy.  C = 1, wenn dort ein Gegner steht; X = sein Abstand in EN.  Veraendert A, X, EQX, EQY.
find:   stx EQX
        sty EQY
        ldx #0
.l:     lda EN + E_X,x
        cmp EQX
        bne .nx
        lda EN + E_Y,x
        cmp EQY
        beq .yes
.nx:    txa
        clc
        adc #E_SIZE
        tax
        cpx ELIM
        bcc .l
        clc
        rts
.yes:   sec
        rts

; under_at: X = gx, Y = gy.  A = gemerkte Zellenfarbe eines Gegners dort, sonst COL_DONE.  Veraendert A, X, EQX, EQY.
under_at:
        jsr find
        bcc .no
        lda EN + E_UNDER,x
        rts
.no:    lda #COL_DONE
        rts

; pick_random: T7 = Maske (nicht 0).  X = zufaellig gewaehlte Richtung aus der Maske.  Veraendert A, X.
pick_random:
        jsr rnd
        and #3
        tax
.s:     lda BITMASK,x
        and T7
        bne .out
        inx
        txa
        and #3
        tax
        jmp .s
.out:   rts

; toward: T4 = gx, T5 = gy des Gegners.  A = Maske der Richtungen (Bit = Dir), die den Abstand zur Figur verkleinern (hoechstens eine je Achse).
;   Veraendert A, X.
toward: lda #0
        ldx PX
        cpx T4
        beq .y
        bcc .l
        ora #1 << Dir.right
        bne .y
.l:     ora #1 << Dir.left
.y:     ldx PY
        cpx T5
        beq .e
        bcc .u
        ora #1 << Dir.down
        rts
.u:     ora #1 << Dir.up
.e:     rts

; choose: Richtung des Gegners ECUR an einem Knoten bestimmen.  Veraendert A, X, Y, T4..T7, EDIST.
choose: ldx ECUR
        lda EN + E_X,x
        sta T4
        lda EN + E_Y,x
        sta T5
        lda EN + E_DIR,x
        sta T6
        jsr grid.legal_mask
        sta T7
        ldx ECUR
        lda EN + E_KIND,x
        bne .rand                       ; Streuner
        jsr rnd
        sta EDIST
        jsr level_idx
        lda EDIST
        cmp PROB,y
        bcs .rand
        jsr toward                      ; Laeufer: zufaellig eine der Richtungen auf die Figur zu (sofern erlaubt)
        and T7
        beq .rand
        sta T7
.rand:  jsr pick_random
        txa
        ldx ECUR
        sta EN + E_DIR,x
        rts

.if ENEMY_DESTROYS_LINES
; touch: Der Gegner hat die Zelle (T4, T5) betreten (Regel 16.2).  Innenzeichen eines Segments: das Segment pruefen und ggf. zerstoeren
;   (grid.seg_destroy).  Knoten: alle bis zu vier Segmente, die dort enden, ebenso.  Veraendert A, X, Y, T0..T3, T7, ATMP, SDX..SDM, PTR_A, PTR_B (nicht T4, T5, ECUR).
touch:
        lda T4                          ; Zielzelle: Innenzeichen eines Segments oder Knoten
        sta T0
        lda T5
        sta T1
        jsr grid.cell_type
        cmp #CT_NODE
        bcc .seg
        ldx #3                          ; Knoten: alle bis zu vier Segmente, die hier enden, wie beim Betreten eines Innenzeichens pruefen
.nd:    stx ATMP
        lda T4
        clc
        adc DIR_DX,x
        cmp #GRID_W
        bcs .nxd
        sta T0
        lda T5
        clc
        adc DIR_DY,x
        cmp #GRID_H
        bcs .nxd
        sta T1
        lda #CT_H                       ; links/rechts: waagerechtes Segment, hoch/runter: senkrechtes
        cpx #2
        bcs .typ
        lda #CT_V
.typ:   sta T2
        jsr grid.seg_destroy
.nxd:   ldx ATMP
        dex
        bpl .nd
        rts
.seg:   sta T2
        jmp grid.seg_destroy
.endif

; one: ein Schritt des Gegners ECUR.  Veraendert A, X, Y, T0, T1, T4..T7 und die Hilfswerte.
one:    ldx ECUR
        lda EN + E_X,x
        sta T0
        lda EN + E_Y,x
        sta T1
        jsr grid.cell_type
        cmp #CT_NODE
        bne .move
        jsr choose
.move:  ldx ECUR
        lda EN + E_DIR,x
        tay
        lda EN + E_X,x
        clc
        adc DIR_DX,y
        sta T4
        lda EN + E_Y,x
        clc
        adc DIR_DY,y
        sta T5
        cmp #GRID_H
        bcs .draw
        lda T4
        cmp #GRID_W
        bcs .draw
        cmp PX
        bne .free
        lda T5
        cmp PY
        bne .free
        lda PINV                        ; Zielzelle = Figur
        bne .blk
        lda #1
        sta HITF
        bne .draw
.free:  ldx T4
        ldy T5
        jsr find
        bcc .step
.blk:   ldx ECUR                        ; Gegner oder unverwundbare Figur im Weg: umdrehen (sonst blockieren sie sich auf einer Linie fuer immer)
        lda EN + E_DIR,x
        eor #1
        sta EN + E_DIR,x
        jmp .draw
.step:
.if ENEMY_DESTROYS_LINES
        jsr touch
.endif
        jsr erase_one
        ldx ECUR
        lda T4
        sta EN + E_X,x
        lda T5
        sta EN + E_Y,x
        ldx T4
        ldy T5
        jsr grid.color_at
        ldx ECUR
        sta EN + E_UNDER,x
.draw:  jmp draw_one

; step: ein Spielschritt aller Gegner (Bild wechseln, jeden Gegner bewegen).  Veraendert A, X, Y, T0..T7, ECUR und die Hilfswerte.
step:   lda EPH
        eor #1
        sta EPH
        lda #0
        sta ECUR
.l:     jsr one
        jsr nxt
        bcc .l
        rts
}
