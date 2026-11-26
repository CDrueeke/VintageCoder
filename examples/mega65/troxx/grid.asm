; =====================================================================================================================
; grid.asm - Gitter: Datenmodell, Zeichnen, Faerben, Segmente, Rechtecke, Fuellen, Zerstoeren
; =====================================================================================================================
; Herkunft: examples/vic20/troxx/grid.asm und examples/c64/troxx/grid.asm, Stand Commit 4697d074 der Referenz.  Kopien (Pruefsumme, siehe KOPIEN.txt):
;   box_add, legal_mask.  Angepasst an das MEGA65-Gitter (14 x 7 Rechtecke, Innenraum 4 x 5, Teiler 5 und 6), das Schattengitter CELL und zwei Figuren
;   (WHO: Spieler, dem Punkte und Farbe gehoeren): set_row, cell_type, nb_boxes, seg_*, paint, fill_row, fill_box, seg_destroy.  Neu: draw, render,
;   Fuellanimation per DMA (fill_dma).
;
; ZWECK     Alles, was das feste Gitter betrifft.  Die Gitterzelle (gx, gy) liegt auf Bildschirmspalte gx + GRID_COL0 und Zeile gy + GRID_ROW0
;           (Spalten 4..74, Zeilen 0..42; die Tafeln liegen in den Zeilen 43..49).
;
; DATENMODELL DES GITTERS (Abschnitt 5 des Konzepts, Fassung MEGA65)
;   71 x 43 Zellen.  Linienzellen: gx mod 5 = 0 (senkrecht) oder gy mod 6 = 0 (waagerecht); Knoten: beides.  Innenraeume 4 x 5 Zellen.
;   Drei Ebenen, jede mit genau einem Zweck:
;   1. Schattengitter CELL ($C000, 80 Byte je Zeile): Zustand jeder Zelle (CS_*): Linienzelle unberuehrt, von Spieler 1 oder 2 abgefahren, gefuellt (dann die
;      Kachelnummer).  Die Figuren sind Sprites und aendern CELL nicht.  Die Logik liest und schreibt nur CELL; render bringt eine Zelle auf den Bildschirm.
;   2. BOXCNT (98 Byte): je Rechteck die Zahl abgeschlossener Kanten; bei 4 wird gefuellt (Pruefung ohne Schleife ueber alle Rechtecke).
;   Ein Segment gilt als abgeschlossen, wenn alle seine Innenzeichen abgefahren sind (Zustand CS_DONE1 oder CS_DONE2); das tritt genau beim Faerben des
;   letzten unberuehrten Zeichens ein.  Der Spieler, der es faerbt, bekommt die Praemie und das ganze Segment seine Farbe (seg_recolor).
;   Nummerierung: waagerecht H(r, c) = 14 r + c (r 0..7, c 0..13, 0..111); senkrecht V(r, c) = 112 + 15 r + c (r 0..6, c 0..14, 112..216);
;   Rechteck B(r, c) = 14 r + c (0..97).  Aus der Zelle: r = gy div 6, c = gx div 5 (Tabellen SEGBASE = 14 r und DIV5).
;   Kanten von B: oben H(r,c) = b, unten H(r+1,c) = b + 14, links V(r,c), rechts V(r,c+1).  Nachbarn eines Segments aus b0 = 14 r + c der Zelle:
;   H: unten b0 (wenn gy < 42), oben b0 - 14 (wenn gy > 0);  V: rechts b0 (wenn gx < 70), links b0 - 1 (wenn gx > 0).
; =====================================================================================================================

grid: {

; set_row: Y = Gitterzeile gy (0..42).  PTR_B = Anfang der Zeile im Schattengitter.  Veraendert A.
set_row:
        lda ROW_LO,y
        sta PTR_B
        lda ROW_HI,y
        sta PTR_B + 1
        rts

; draw: Gitter neu aufbauen: Linienzellen unberuehrt, Innenraeume leer, alles auf den Bildschirm.  Veraendert A, X, Y, T0, T1, PTR_B und die Zeiger von render.
draw:   lda #0
        sta T0                          ; gy
.row:   ldy T0
        jsr set_row
        ldy T0
        lda ROWKIND,y
        asl
        asl
        sta T1
        ldy #0
.cell:  lda COLKIND,y
        clc
        adc T1
        tax
        lda SHAPE,x
        cmp #255
        lda #CS_LINE
        bcc .st
        lda #CS_NONE
.st:    sta (PTR_B),y
        iny
        cpy #GRID_W
        bne .cell
        inc T0
        lda T0
        cmp #GRID_H
        bne .row
        ; ---- alles zeichnen (weiter mit redraw)
redraw: lda #0
        sta T0
.rr:    ldx #0
.rc:    stx T1
        ldy T0
        jsr render
        ldx T1
        inx
        cpx #GRID_W
        bne .rc
        inc T0
        lda T0
        cmp #GRID_H
        bne .rr
        rts

; render: Zelle X = gx, Y = gy aus dem Schattengitter auf den Bildschirm bringen.  Veraendert A, X, Y, Z, PTR_B, RT0..RT3 und die Zeiger sq, cq.
render: stx RT0
        sty RT1
        lda ROW_LO,y
        sta PTR_B
        lda ROW_HI,y
        sta PTR_B + 1
        ldy RT0
        lda (PTR_B),y                   ; Zustand
        beq .blank
        cmp #16
        bcs .tile                       ; gefuellt: Kachelnummer
        sta RT2
        ldy RT1
        lda ROWKIND,y
        asl
        asl
        sta RT3
        ldy RT0
        lda COLKIND,y
        clc
        adc RT3
        tay
        lda SHAPE,y                     ; Form 0..10
        asl
        asl                             ; * 4 Zustaende (unberuehrt, Spieler 1, Spieler 2, Aufblitzen)
        clc
        adc RT2
        sec
        sbc #1
.tile:  ldy RT1
        pha
        lda RT0
        clc
        adc #GRID_COL0
        tax
        pla
        jmp vic.put_tile
.blank: lda RT0
        clc
        adc #GRID_COL0
        tax
        ldy RT1
        jmp vic.put_blank

; color_at: Zustand der Zelle X = gx, Y = gy (CS_*).  A = Zustand.  X bleibt.  Veraendert A, Y, PTR_B.
color_at:
        jsr set_row
        txa
        tay
        lda (PTR_B),y
        rts

; state_at: Zustand einer Linienzelle X = gx, Y = gy.  Figuren sind Sprites, das Schattengitter gilt unmittelbar.
state_at:
        jmp color_at

; cell_type: T0 = gx, T1 = gy.  A = CT_NONE, CT_H, CT_V oder CT_NODE (Z-Flag passend zu A).  Veraendert A, Y.
cell_type:
        ldy T1
        lda MOD6,y
        bne .col                        ; keine waagerechte Linie
        ldy T0
        lda MOD5,y
        beq .node
        lda #CT_H
        rts
.node:  lda #CT_NODE
        rts
.col:   ldy T0
        lda MOD5,y
        bne .none                       ; Innenraum
        lda #CT_V
        rts
.none:  lda #CT_NONE
        rts

;   Segment-Durchlauf: seg_first (T0 = gx, T1 = gy, T2 = Zelltyp einer Zelle des Segments) setzt SDX, SDY auf die erste Innenzelle und SDN auf die Anzahl
;   (4 oder 5); seg_next rueckt um eine Zelle vor (Z = 1 nach der letzten).  Veraendert A, Y.
seg_first:
        lda T2
        cmp #CT_V
        beq .v
        ldy T0                          ; waagerecht: erste Innenzelle gx = 5 c + 1, 4 Zellen
        lda DIV5,y
        sta SDX
        asl
        asl
        clc
        adc SDX
        clc
        adc #1
        sta SDX
        lda T1
        sta SDY
        lda #BOX_W
        bne .go
.v:     lda T0                          ; senkrecht: erste Innenzelle gy = 6 r + 1, 5 Zellen
        sta SDX
        ldy T1
        lda SEGROW,y                    ; 6 r
        clc
        adc #1
        sta SDY
        lda #BOX_H
.go:    sta SDN
        rts

seg_next:
        lda T2
        cmp #CT_V
        beq .dy
        inc SDX
        dec SDN
        rts
.dy:    inc SDY
        dec SDN
        rts

; seg_full: T0, T1, T2 wie seg_first.  C = 1, wenn alle Innenzeichen des Segments abgefahren sind.  Veraendert A, X, Y, SDX, SDY, SDN, PTR_B.
seg_full:
        jsr seg_first
.cell:  ldx SDX
        ldy SDY
        jsr state_at
        cmp #CS_DONE1
        bcc .no
        cmp #CS_DONE2 + 1
        bcs .no
        jsr seg_next
        bne .cell
        sec
        rts
.no:    clc
        rts

; seg_recolor: alle Innenzeichen des Segments (T0, T1, T2 wie seg_first) in die Farbe des Spielers WHO setzen und neu zeichnen.  Veraendert wie seg_full.
seg_recolor:
        jsr seg_first
.cell:  ldx SDX
        ldy SDY
        jsr set_row
        ldy SDX
        lda WHO
        clc
        adc #CS_DONE1
        sta (PTR_B),y
        ldx SDX
        ldy SDY
        jsr render
        jsr seg_next
        bne .cell
        rts

; box_add: X = Rechtecknummer ($FF = keins): eine Kante mehr.  Bei 4 wird das Rechteck zum Fuellen vorgemerkt (genau einmal) und 100 Punkte
;   gebucht.  Veraendert A, X, Y.
box_add:
        cpx #$80
        bcs .out
        inc BOXCNT,x
        lda BOXCNT,x
        cmp #4
        bne .out
        ldy FILLS_NOW
        txa
        sta FILLQ,y
        inc FILLS_NOW
        inc BOXES_DONE
.ifdef TESTDEMO
        inc TD_BASE + 5                 ; Zaehler "Rechtecke gesamt" (Dauerlauf)
.endif
        lda #Fx.box
        jsr sound.play
        lda #1
        ldx #1
        jmp hud.bcd_add                 ; 100 Punkte
.out:   rts

; nb_boxes: T0 = gx, T1 = gy, T2 = Zelltyp (CT_H oder CT_V) eines Segments: NB0 und NB1 = die ein oder zwei angrenzenden Rechtecke ($FF = keins).
;   Aus b0 = 14 r + c der Zelle:  waagerecht: unten b0 (wenn gy < 42), oben b0 - 14 (wenn gy > 0);  senkrecht: rechts b0 (wenn gx < 70), links b0 - 1
;   (wenn gx > 0).  Veraendert A, X, Y, T3.
nb_boxes:
        ldy T1
        ldx T0
        lda DIV5,x
        clc
        adc SEGBASE,y
        sta NB0                         ; b0
        ldx T1                          ; waagerecht: Koordinate gy, Nachbar oben = b0 - 14
        ldy #BOX_COLS
        lda T2
        cmp #CT_V
        bne .go
        ldx T0                          ; senkrecht: Koordinate gx, Nachbar links = b0 - 1
        ldy #1
.go:    sty T3
        lda NB0
        sec
        sbc T3
        cpx #0
        bne .b1
        lda #$ff                        ; Rand: kein Nachbar oben/links
.b1:    sta NB1
        lda T2
        cmp #CT_V
        beq .vlim
        cpx #GRID_H - 1
        bcc .ok
        bcs .none
.vlim:  cpx #GRID_W - 1
        bcc .ok
.none:  lda #$ff                        ; Rand: kein Nachbar unten/rechts
        sta NB0
.ok:    rts

.if ENEMY_DESTROYS_LINES
; box_filled: X = Rechtecknummer ($FF = keins).  C = 1, wenn das Rechteck gefuellt ist (BOXCNT = 4).  Veraendert A.
box_filled:
        cpx #$80
        bcs .no
        lda BOXCNT,x
        cmp #4
        rts
.no:    clc
        rts

; seg_is_closed: T0 = gx, T1 = gy, T2 = Zelltyp eines Segments.  C = 1, wenn das Segment geschuetzt ist: mindestens ein angrenzendes Rechteck ist
;   gefuellt (auch ein ganz abgefahrenes Segment ist sonst nicht geschuetzt).  Setzt NB0, NB1.  Veraendert A, X, Y, T3.
seg_is_closed:
        jsr nb_boxes
        ldx NB0
        jsr box_filled
        bcs .out
        ldx NB1
        jmp box_filled
.out:   rts

; cell_op: eine Zelle (X = gx, Y = gy) des Segments im Durchgang SDM bearbeiten:
;   0 zaehlen und aufblitzen: SDP und SDQ zaehlen die abgefahrenen Zeichen von Spieler 1 und 2; diese Zeichen werden zum Aufblitzen (weiss)
;   1 zuruecksetzen: Linienzeichen, unberuehrt.
;   Veraendert A, X, Y, PTR_B, RT0..RT3 und die Zeiger von render.
cell_op:
        jsr set_row
        ldy SDX
        lda (PTR_B),y
        ldx SDM
        bne .reset
        cmp #CS_DONE1
        bne .n1
        inc SDP
        bne .fl
.n1:    cmp #CS_DONE2
        bne .out                        ; unberuehrt: nichts zu zaehlen
        inc SDQ
.fl:    lda #CS_FLASH
        sta (PTR_B),y
        ldx SDX
        ldy SDY
        jmp render
.reset: lda #CS_LINE
        sta (PTR_B),y
        ldx SDX
        ldy SDY
        jmp render
.out:   rts

; seg_destroy: ein Gegner betritt das Innenzeichen T0 = gx, T1 = gy (T2 = Zelltyp CT_H oder CT_V).  Grenzt das Segment an ein gefuelltes Rechteck, geschieht nichts.
;   Sonst, wenn mindestens ein Zeichen gefaerbt ist: aufblitzen (2 Bilder) mit Ton, das ganze Segment zuruecksetzen, Punkte abziehen (1 je gefaerbtes
;   Zeichen, 10 mehr bei abgeschlossenem Segment, nie unter 0; jeder Spieler verliert die Punkte seiner Zeichen), bei abgeschlossenem Segment BOXCNT der
;   angrenzenden Rechtecke - 1.  Veraendert A, X, Y, T0..T3, T7, SDX..SDQ, PTR_B (nicht T4, T5, ECUR).
seg_destroy:
        jsr seg_is_closed
        bcc .sd
        jmp .ret
.sd:
        lda #0
        sta SDP
        sta SDQ
        sta SDM
.pass:  jsr seg_first
.cell:  ldx SDX
        ldy SDY
        jsr cell_op
        jsr seg_next
        bne .cell
        lda SDM
        bne .fin
        lda SDP
        ora SDQ
        beq .ret                        ; nichts gefaerbt: nichts zu zerstoeren
        lda #Fx.destroy
        jsr sound.play
        jsr vic.wait_frame
        jsr vic.wait_frame
.ifdef HOLD_DESTROY                     ; nur Testfassung: das Aufblitzen HOLD_DESTROY Bilder lang halten (fuer Bildschirmaufnahmen)
        lda #HOLD_DESTROY
.hold:  pha
        jsr vic.wait_frame
        pla
        sec
        sbc #1
        bne .hold
.endif
        inc SDM
        bne .pass
.fin:   ldx #BOX_W                      ; Anzahl der Zeichen des Segments
        lda T2
        cmp #CT_V
        bne .cnt
        ldx #BOX_H
.cnt:   lda #$ff                        ; abgeschlossen (alle Zeichen von einem Spieler)?  T3 = dieser Spieler, sonst $FF
        sta T3
        cpx SDP
        bne .c2
        lda #0
        sta T3
.c2:    cpx SDQ
        bne .c3
        lda #1
        sta T3
.c3:    lda WHO
        pha                             ; WHO bleibt erhalten (die Punkte gehen an die Besitzer der Zeichen)
        lda T3
        bmi .pts
        ldx NB0                         ; BOXCNT der angrenzenden Rechtecke - 1
        bmi .n1
        dec BOXCNT,x
.n1:    ldx NB1
        bmi .n2
        dec BOXCNT,x
.n2:    lda T3
        sta WHO
        lda #$10
        jsr hud.bcd_sub                 ; 10 Punkte des Segments
.pts:   lda SDP
        beq .p2
        ldx #0
        stx WHO
        jsr hud.bcd_sub                 ; 1 Punkt je Zeichen von Spieler 1 (Anzahl < 10: BCD gleich Dualzahl)
.p2:    lda SDQ
        beq .end
        ldx #1
        stx WHO
        jsr hud.bcd_sub
.end:   jsr nodes_free                  ; freistehende gelbe Knoten zuruecksetzen (Knotenregel)
        pla
        sta WHO
.ret:   rts

; nodes_free: alle 120 Knoten pruefen (Regel 16.2, Knotenregel; Referenz 9fa111ad): Ein gefaerbter Knoten wird wieder blau, wenn an ihm kein
;   gefaerbtes Segment mehr anschliesst (keine der acht Nachbarzellen abgefahren) und kein angrenzendes Rechteck gefuellt ist.  Knoten zaehlen nicht
;   fuer Punkte.  Angepasst: MEGA65-Gitter (Schattengitter CELL, Sprites, zwei Figuren).  Veraendert A, X, Y, NODEX, NODEY, SDN, SDM, PTR_B, EQX, EQY.
nodes_free:
        lda #0
        sta NODEY
.row:   lda #0
        sta NODEX
.col:   jsr node_free
        lda NODEX
        clc
        adc #BOX_W + 1
        sta NODEX
        cmp #GRID_W
        bcc .col
        lda NODEY
        clc
        adc #BOX_H + 1
        sta NODEY
        cmp #GRID_H
        bcc .row
        rts

; node_free: der Knoten (NODEX, NODEY).  Nur gefaerbte Knoten; steht eine Figur oder ein Gegner darauf, bleibt er.  Die acht Nachbarzellen im Gitter:
;   abgefahren (CS_DONE1/2) oder Kachel (Zustand >= 16 = angrenzendes Rechteck gefuellt) = bleibt; sonst zurueck auf CS_LINE.
node_free:
        ldy NODEY
        jsr set_row
        ldy NODEX
        lda (PTR_B),y
        cmp #CS_DONE1
        bcc .out
        cmp #CS_DONE2 + 1
        bcs .out                        ; nur gefaerbt (Spieler 1 oder 2)
        ldx #1                          ; steht eine aktive Figur auf dem Knoten?
.fg:    lda PACT,x
        beq .nf
        lda NODEX
        cmp PX,x
        bne .nf
        lda NODEY
        cmp PY,x
        beq .out
.nf:    dex
        bpl .fg
        ldx NODEX
        ldy NODEY
        jsr enemies.find
        bcs .out                        ; ein Gegner steht auf dem Knoten
        lda #$ff
        sta SDN                         ; dy = -1
.ry:    lda #$ff
        sta SDM                         ; dx = -1
.rx:    lda SDM
        ora SDN
        beq .nx                         ; die Mitte (der Knoten selbst) zaehlt nicht
        lda NODEX
        clc
        adc SDM
        cmp #GRID_W
        bcs .nx
        sta EQX
        lda NODEY
        clc
        adc SDN
        cmp #GRID_H
        bcs .nx
        tay
        jsr set_row
        ldy EQX
        lda (PTR_B),y
        cmp #CS_DONE1
        bcc .nx                         ; leer, unberuehrt oder aufblitzend
        cmp #CS_DONE2 + 1
        bcc .out                        ; CS_DONE1 oder CS_DONE2: gefaerbte Nachbarzelle
        cmp #16
        bcs .out                        ; Kachel: angrenzendes Rechteck gefuellt
.nx:    inc SDM
        lda SDM
        cmp #2
        bne .rx
        inc SDN
        lda SDN
        cmp #2
        bne .ry
        ldy NODEY                       ; frei stehend: zuruecksetzen
        jsr set_row
        ldy NODEX
        lda #CS_LINE
        sta (PTR_B),y
        ldx NODEX
        ldy NODEY
        jmp render
.out:   rts
.endif

; paint_node: eine Figur des Spielers WHO betritt die Zelle T0 = gx, T1 = gy: ist es ein Knoten, wird er in der Farbe des Spielers gefaerbt (ohne Punkte;
;   in der Referenzfassung macht das erase der Figur).  Neu im MEGA65-Projekt (Figuren sind Sprites).  Veraendert A, X, Y, PTR_B, RT0..RT3 und die Zeiger von render.
paint_node:
        jsr cell_type
        cmp #CT_NODE
        bne .out
        ldy T1
        jsr set_row
        lda WHO
        clc
        adc #CS_DONE1
        ldy T0
        sta (PTR_B),y
        ldx T0
        ldy T1
        jmp render
.out:   rts

; paint: eine Figur des Spielers WHO betritt die Linienzelle T0 = gx, T1 = gy (vor dem Zeichnen der Figur).  Knoten und Innenraeume tun nichts.
;   Zelle unberuehrt: faerben, 1 Punkt; ist damit das Segment voll (das geschieht je Segment genau einmal): 10 Punkte, ganzes Segment in der Farbe des
;   Spielers, Nachbarrechtecke (box_add); schliessen zwei Rechtecke zugleich, kommen 200 Punkte dazu (Kettenreaktion, zusammen 400).
;   Veraendert A, X, Y, T2..T6, PTR_B, SDX..SDN und die Zeiger von render.
paint:  jsr cell_type
        beq .out                        ; kein Linienzeichen
        cmp #CT_NODE
        bcs .out
        sta T2
        ldx T0
        ldy T1
        jsr color_at                    ; setzt PTR_B auf den Anfang der Zeile
        cmp #CS_LINE
        bne .out                        ; schon abgefahren: nichts zu tun
        lda WHO
        clc
        adc #CS_DONE1
        ldy T0
        sta (PTR_B),y
        ldx T0
        ldy T1
        jsr render
        lda #1
        ldx #0
        jsr hud.bcd_add                 ; 1 Punkt
        jsr seg_full
        bcc .out
        jsr seg_recolor
        lda #$10
        ldx #0
        jsr hud.bcd_add                 ; 10 Punkte
        lda #Fx.seg
        jsr sound.play
        jsr nb_boxes
        ldx NB0
        jsr box_add
        ldx NB1
        jsr box_add
        lda FILLS_NOW
        cmp #2
        bne .out
        lda #Fx.chain                   ; Kettenreaktion: zwei Rechtecke zugleich
        jsr sound.play
        lda #2
        ldx #1
        jmp hud.bcd_add                 ; +200 Punkte
.out:   rts

; legal_mask: T4 = gx, T5 = gy, T6 = bisherige Richtung.  A = Maske der Richtungen (Bit = Dir) mit Zielzelle im Gitter, ohne die Umkehr
;   Gilt an Knoten, dort fuehrt jede Richtung im Gitter auf eine Linie.  Veraendert A, X, T7.
legal_mask:
        lda #0
        sta T7
        ldx #3
.d:     lda T4
        clc
        adc DIR_DX,x
        cmp #GRID_W
        bcs .no
        lda T5
        clc
        adc DIR_DY,x
        cmp #GRID_H
        bcs .no
        lda BITMASK,x
        ora T7
        sta T7
.no:    dex
        bpl .d
        lda T6
        eor #1                          ; Umkehr
        tax
        lda BITMASK,x
        eor #$ff
        and T7
        rts                             ; (nie leer: an jedem Knoten fuehrt mindestens eine Richtung ausser der Umkehr ins Gitter)

; fill_tile: Kachelnummer der Fuellung fuer das Rechteck mit der Zeile r (T4) und der Innenzeile k (A): T_FILL + 5 * BOX_COLOR[r] + k.  A = Kachel.
fill_tile:
        sta T6
        ldy T4
        lda BOX_COLOR,y
        asl
        asl
        clc
        adc BOX_COLOR,y                 ; 5 * Farbe
        clc
        adc T6
        clc
        adc #T_FILL
        rts

; fill_row: Schritt k der Fuellanimation eines Rechtecks.  X = Rechteckzeile r (0..6), Y = Rechteckspalte c (0..13), A = Schritt k (0..5):
;   k < 5: Innenzeile k im Schattengitter fuellen und auf dem Bildschirm als Glanzzeile zeigen (per DMA); k > 0: die Zeile davor erhaelt ihre Fuellkachel
;   (ebenfalls per DMA).  Schritt 5 schliesst die letzte Zeile ab.  Veraendert A, X, Y, T0..T6, PTR_B und die DMA-Liste.
.assert BOX_W == 4 && BOX_H == 5, "fill_row rechnet mit 5 * c und 6 * r"
fill_row:
        stx T4
        sty T5
        sta T0                          ; Schritt k
        lda T5                          ; erste Innenspalte gx = 5 c + 1
        sta T1
        asl
        asl
        clc
        adc T1
        clc
        adc #1
        sta T1                          ; gx
        lda T4                          ; Zeile gy = 6 r + 1 (Innenzeile 0)
        asl
        sta T2
        asl
        clc
        adc T2
        clc
        adc #1
        sta T2                          ; gy der Innenzeile 0
        lda T0
        cmp #BOX_H
        bcs .prev                       ; Schritt 5: nur die letzte Zeile abschliessen
        jsr fill_tile
        sta T3                          ; Fuellkachel der Zeile
        lda T2
        clc
        adc T0
        tay                             ; gy der Zeile k
        jsr set_row
        ldy T1
        ldx #BOX_W
.cell:  lda T3
        sta (PTR_B),y
        iny
        dex
        bne .cell
.if FX_FILL
        lda T1
        clc
        adc #GRID_COL0
        tax
        lda T2
        clc
        adc T0
        clc
        adc #GRID_ROW0
        tay
        lda #T_GLINT
        jsr vic.put_row4                ; Glanzzeile
.else
        lda T1
        clc
        adc #GRID_COL0
        tax
        lda T2
        clc
        adc T0
        clc
        adc #GRID_ROW0
        tay
        lda T3
        jsr vic.put_row4
.endif
.prev:  lda T0
        beq .out
        sec
        sbc #1                          ; Zeile davor
        sta T6
        jsr fill_tile
        sta T3
        lda T1
        clc
        adc #GRID_COL0
        tax
        lda T2
        clc
        adc T6
        clc
        adc #GRID_ROW0
        tay
        lda T3
        jsr vic.put_row4
.out:   rts
; fill_row_b: wie fill_row, mit A = Rechtecknummer b (0..97), X = Innenzeile k.  Veraendert wie fill_row, dazu T7.
fill_row_b:
        stx T7
        ldy #0
.d:     cmp #BOX_COLS
        bcc .e
        sbc #BOX_COLS
        iny
        bne .d
.e:     sta T5                          ; c
        tya
        tax                             ; r
        ldy T5
        lda T7
        jmp fill_row

; fill_box: ganzes Rechteck fuellen.  X = Rechteckzeile r, Y = Rechteckspalte c.  Veraendert wie fill_row.
fill_box:
        stx RT4
        sty RT5
        lda #0
        sta RT6
.k:     ldx RT4
        ldy RT5
        lda RT6
        jsr fill_row
        inc RT6
        lda RT6
        cmp #BOX_H
        bcc .k
        rts
}



