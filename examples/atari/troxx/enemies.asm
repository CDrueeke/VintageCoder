; =====================================================================================================================
; enemies.asm - Gegner: Zufall, Entscheidung, Bewegung
; =====================================================================================================================
; Herkunft: examples/vic20/troxx/enemies.asm, Stand Commit 4697d074; Atari-Fassung unveraendert nach examples/c64/troxx/enemies.asm (Stand Commit d173cb4a).
;   Kopien (Pruefsumme, siehe KOPIEN.txt): rnd, level_idx, nxt, find, pick_random, greedy.  Angepasst: place_all, choose (Waechter), one, step (Starre durch
;   den Eisblock; Gegner sind Player/Missiles, kein Zeichnen und keine gemerkte Linienfarbe mehr, dafuer die Zelle vor dem Schritt fuer die Bewegung).
; ZWECK     1 bis 4 Gegner laufen zeichenweise auf den Linien, je Spielschritt ein Schritt.  Entscheidungen fallen nur an Knoten: erlaubt ist jede
;           Richtung im Gitter ausser der Umkehr.  Typen: Laeufer (mit Wahrscheinlichkeit p, nach Stufe 50 bis 90 %, die Richtung, die den
;           Abstand |dx| + |dy| zur Figur am staerksten verkleinert; sonst Zufall), Streuner (immer Zufall) und
;           Waechter (EXTRA_GUARD): geradeaus, wenn die Linie weiterfuehrt, sonst die einzige Abbiegung; von einem Eckknoten aus ergibt das einen
;           festen Rundgang um den Rand des Gitters.  Gegner liegen nie auf derselben Zelle wie die Figur oder ein anderer Gegner: ein Schritt auf
;           eine besetzte Zelle faellt aus; trifft ein Gegner so auf die Figur, ist das ein Treffer (HITF), ausser waehrend der Unverwundbarkeit.
; DATEN     EN: je Gegner x, y, Richtung, Typ, Zelle vor dem Schritt (ox, oy).  Typ nach Nummer: KIND_OF (data.inc).
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

; place_all: Gegnerzahl der Stufe setzen, Gegner auf die Startknoten.  Veraendert A, X, Y, T0, T2, ECUR.
place_all:
        jsr level_idx
        lda COUNT,y
        sta NENEMY
        asl
        sta T0
        asl
        clc
        adc T0
        sta ELIM                        ; Gegnerzahl * 6
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
        sta EN + E_OX,x
        lda START_Y,y
        sta EN + E_Y,x
        sta EN + E_OY,x
        lda KIND_OF,y
        sta EN + E_KIND,x
        inc T2
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

; greedy: T4 = gx, T5 = gy, T7 = Maske.  X = Richtung aus der Maske, die den Abstand zur Figur am kleinsten macht.  Veraendert A, X, EBEST, EDIR, EDIST.
greedy: lda #$ff
        sta EBEST
        ldx #3
.d:     lda BITMASK,x
        and T7
        beq .nx
        lda T4
        clc
        adc DIR_DX,x
        sec
        sbc PX
        bcs .p1
        eor #$ff
        adc #1
.p1:    sta EDIST
        lda T5
        clc
        adc DIR_DY,x
        sec
        sbc PY
        bcs .p2
        eor #$ff
        adc #1
.p2:    clc
        adc EDIST
        cmp EBEST
        bcs .nx
        sta EBEST
        stx EDIR
.nx:    dex
        bpl .d
        ldx EDIR
        rts

; choose: Richtung des Gegners ECUR an einem Knoten bestimmen.  Veraendert A, X, Y, T4..T7, EBEST, EDIR, EDIST.
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
.if EXTRA_GUARD
        cmp #KIND_GUARD
        beq .guard
.endif
        cmp #KIND_RUNNER
        bne .rand                       ; Streuner
        jsr rnd
        sta EDIST
        jsr level_idx
        lda EDIST
        cmp PROB,y
        bcs .rand
        jsr greedy
        jmp .sel
.rand:  jsr pick_random
.sel:   txa
        ldx ECUR
        sta EN + E_DIR,x
        rts
.if EXTRA_GUARD
.guard: lda EN + E_DIR,x                ; Waechter: geradeaus, wenn die Linie weiterfuehrt
        tax
        lda BITMASK,x
        and T7
        bne .sel
        ldx #0                          ; sonst die (einzige) Abbiegung
.g:     lda BITMASK,x
        and T7
        bne .sel
        inx
        bne .g
.endif

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
        lda EN + E_X,x                  ; Zelle vor dem Schritt merken (Sprite-Bewegung)
        sta EN + E_OX,x
        lda EN + E_Y,x
        sta EN + E_OY,x
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
        bcs .out
        lda T4
        cmp #GRID_W
        bcs .out
        cmp PX
        bne .free
        lda T5
        cmp PY
        bne .free
        lda PINV                        ; Zielzelle = Figur
        bne .blk
        lda #1
        sta HITF
        bne .out
.free:  ldx T4
        ldy T5
        jsr find
        bcc .step
.blk:   ldx ECUR                        ; Gegner oder unverwundbare Figur im Weg: umdrehen (sonst blockieren sie sich auf einer Linie fuer immer)
        lda EN + E_DIR,x
        eor #1
        sta EN + E_DIR,x
        rts
.step:
.if ENEMY_DESTROYS_LINES
        jsr touch
.endif
        ldx ECUR
        lda T4
        sta EN + E_X,x
        lda T5
        sta EN + E_Y,x
.out:   rts

; step: ein Spielschritt aller Gegner (Bild wechseln, jeden Gegner bewegen).  Steht ein Eisblock-Effekt an (FREEZE), bewegt sich keiner.
;   Veraendert A, X, Y, T0..T7, ECUR und die Hilfswerte.
step:   lda EPH
        eor #1
        sta EPH
        lda #0
        sta ECUR
.if EXTRA_ICE
        lda FREEZE
        ora FREEZE + 1
        beq .l
.fz:    ldx ECUR                        ; starr: die Sprites stehen auf der aktuellen Zelle
        lda EN + E_X,x
        sta EN + E_OX,x
        lda EN + E_Y,x
        sta EN + E_OY,x
        jsr nxt
        bcc .fz
        rts
.endif
.l:     jsr one
        jsr nxt
        bcc .l
        rts
}
