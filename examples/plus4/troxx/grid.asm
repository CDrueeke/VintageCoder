; =====================================================================================================================
; grid.asm - Gitter: Datenmodell, Zeichnen, Faerben, Segmente, Rechtecke, Fuellen
; =====================================================================================================================
; Herkunft: examples/c64/troxx/grid.asm (Stand Commit d173cb4a) und examples/vic20/troxx/grid.asm, Stand Commit 4697d074.  Kopien (Pruefsumme, siehe
;   KOPIEN.txt): cell_type, box_add, seg_next, seg_full, seg_is_closed, box_filled, legal_mask, fill_row_b.  Angepasst: set_row (Schattenkopie), draw, color_at, state_at, seg_first,
;   nb_boxes (Gittermasse), cell_op und seg_destroy (Regel 16.2: Gegner zerstoeren Linien; Schattenkopie, Aufblitzen ueber den Farbspeicher),
;   paint (Schattenkopie), fill_row (Zeichen je Lage im Block, Farbverlauf).  Neu: redraw, prefill (zweites Layout), fill_colour, save, load.
;
; ZWECK     Alles, was das feste Gitter betrifft.  Die Gitterzelle (gx, gy) liegt auf Bildschirmspalte gx + GRID_COL0 und Bildschirmzeile
;           gy + GRID_ROW0 (Spalten 3..35, Zeilen 2..22; Zeile 0 Anzeigen, Zeile 1 Meldungen, Zeilen 23 und 24 Leben und Tempo).
;
; DATENMODELL DES GITTERS (Abschnitt 5 des Konzepts)
;   33 x 21 Zellen.  Linienzellen: gx mod 4 = 0 (senkrecht) oder gy mod 4 = 0 (waagerecht); Knoten: beides.  Innenraeume 3 x 3 Zellen.
;   Drei Ebenen, jede mit genau einem Zweck:
;   1. Schattenkopie (SHCH Zeichen, SHCOL Farben; je 33 Byte je Gitterzeile): der Zustand des Gitters ohne Figuren.  Die Figuren sind Zeichen auf dem
;      Bildschirm (figs.asm) und ueberdecken Zellen; sie aendern die Schattenkopie nie, und beim Weiterziehen einer Figur stellt figs.update die Zelle
;      aus der Schattenkopie wieder her.  Linienzustand: blau (COL_LINE) unberuehrt, hellgelb (COL_DONE) abgefahren.
;   2. Bildschirm und Farbspeicher: Anzeige (Schattenkopie plus Figuren).
;   3. BOXCNT (40 Byte): je Rechteck die Zahl abgeschlossener Kanten; bei 4 wird gefuellt (Pruefung ohne Schleife ueber alle Rechtecke).
;   Ein Segment gilt als abgeschlossen, wenn alle seine Innenzeichen gelb sind; das tritt genau beim Faerben des letzten blauen Zeichens ein
;   (jede weitere Zelle ist schon gelb und endet in paint sofort); ein Bitfeld SEGBITS ist darum nicht noetig (wie in der Referenzfassung).
;   Nummerierung: waagerecht H(r, c) = 8 r + c (r 0..5, c 0..7, 0..47); senkrecht V(r, c) = 48 + 9 r + c (r 0..4, c 0..8, 48..92);
;   Rechteck B(r, c) = 8 r + c (0..39).  Aus der Zelle: r = gy div 4, c = gx div 4 (Tabelle SEGBASE = 8 r; gx div 4 = zwei Rechtsschiebungen).
;   Kanten von B: oben H(r,c) = b, unten H(r+1,c) = b + 8, links V(r,c), rechts V(r,c+1).  Nachbarn eines Segments aus b0 = 8 r + c der Zelle:
;   H: unten b0 (wenn gy < 20), oben b0 - 8 (wenn gy > 0);  V: rechts b0 (wenn gx < 32), links b0 - 1 (wenn gx > 0).
;   Zweites Layout (ab Stufe 5, EXTRA_LAYOUT): sechs Rechtecke sind zu Beginn gefuellt (graue Bloecke, Tabelle PRE_B); ihre Kanten sind gelb, BOXCNT
;   aller Rechtecke wird aus den Kanten neu gezaehlt, BOXES_DONE beginnt bei 6.  Alles andere (Zaehler, Regeln) bleibt.
; =====================================================================================================================

grid: {

; set_row: Y = Gitterzeile gy (0..20).  PTR_A = Bildschirm, PTR_B = Farbspeicher, PTR_D = Schattenkopie Zeichen, PTR_E = Schattenkopie Farben, alle am
;   Anfang der Zeile (Spalte gx = 0).  Veraendert A.  Die Low-Bytes der Zeilenanfaenge sind in Bildschirm und Farbspeicher gleich ($0C00 / $0800), nur das
;   High-Byte unterscheidet sich; ebenso in beiden Schattenkopien.
set_row:
        lda ROW_LO + GRID_ROW0,y        ; Bildschirmzeile gy + GRID_ROW0, Spalte GRID_COL0 (kein Uebertrag: die Zeilenanfaenge sind Vielfache von 8)
        clc
        adc #GRID_COL0
        sta PTR_A
        sta PTR_B
        lda ROW_HI + GRID_ROW0,y
        sta PTR_A + 1
        clc
        adc #COLOR_DELTA
        sta PTR_B + 1
        lda SH_LO,y
        sta PTR_D
        sta PTR_E
        lda SH_HI,y
        sta PTR_D + 1
        clc
        adc #SH_DELTA
        sta PTR_E + 1
        rts

; draw: Gitter zeichnen (alle Linien blau): Schattenkopie und Bildschirm.  Veraendert A, X, Y, T0, PTR_A, PTR_B, PTR_D, PTR_E.
draw:
        lda #0
        sta T0                          ; gy
.row:   ldy T0
        jsr set_row
        ldy T0
        ldx ROW_TPL,y                   ; Anfang der Schablone dieser Zeile
        ldy #0
.cell:  lda TPL,x
        sta (PTR_A),y
        sta (PTR_D),y
        lda #COL_LINE
        sta (PTR_B),y
        sta (PTR_E),y
        inx
        iny
        cpy #GRID_W
        bne .cell
        inc T0
        lda T0
        cmp #GRID_H
        bne .row
        rts

; redraw: Schattenkopie auf den Bildschirm kopieren (nach load).  Veraendert A, Y, T0, PTR_A, PTR_B, PTR_D, PTR_E.
redraw:
        lda #0
        sta T0
.row:   ldy T0
        jsr set_row
        ldy #GRID_W - 1
.c:     lda (PTR_D),y
        sta (PTR_A),y
        lda (PTR_E),y
        sta (PTR_B),y
        dey
        bpl .c
        inc T0
        lda T0
        cmp #GRID_H
        bne .row
        rts

; color_at: Farbe (Schattenkopie, ganzes Byte) der Zelle X = gx, Y = gy.  A = Farbe.  X bleibt.  Veraendert A, Y, PTR_A, PTR_B, PTR_D, PTR_E.
color_at:
        jsr set_row
        txa
        tay
        lda (PTR_E),y
        rts

; state_at: Zustand der Linienzelle X = gx, Y = gy (COL_LINE oder COL_DONE).  Figuren sind Zeichen auf dem Bildschirm, die Schattenkopie gilt
;   unmittelbar.  A = Zustand.
state_at:
        jmp color_at

; cell_type: T0 = gx, T1 = gy.  A = CT_NONE, CT_H, CT_V oder CT_NODE (Z-Flag passend zu A).  Veraendert A, Y.
cell_type:
        ldy T1
        lda ROW_TPL,y
        cmp #TPL_INNER
        bne .lr                         ; Zeile mit waagerechter Linie
        lda T0
        and #3
        bne .none                       ; Innenraum
        lda #CT_V
        rts
.none:  lda #CT_NONE
        rts
.lr:    lda T0
        and #3
        beq .node
        lda #CT_H
        rts
.node:  lda #CT_NODE
        rts

; Segment-Durchlauf: seg_first (T0 = gx, T1 = gy, T2 = Zelltyp einer Zelle des Segments) setzt SDX, SDY auf die erste Innenzelle und SDN auf die Anzahl
;   (3); seg_next rueckt um eine Zelle vor (Z = 1 nach der letzten).  Veraendert A, Y.
seg_first:
        lda T2
        cmp #CT_V
        beq .v
        lda T0                          ; waagerecht: erste Innenzelle gx = 4 c + 1, 3 Zellen
        and #$fc
        clc
        adc #1
        sta SDX
        lda T1
        sta SDY
        lda #BOX_W
        bne .go
.v:     lda T0                          ; senkrecht: erste Innenzelle gy = 4 r + 1, 3 Zellen
        sta SDX
        lda T1
        and #$fc
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

; seg_full: T0, T1, T2 wie seg_first.  C = 1, wenn alle Innenzeichen des Segments gelb sind.  Veraendert A, X, Y, SDX, SDY, SDN, PTR_A..PTR_E.
seg_full:
        jsr seg_first
.cell:  ldx SDX
        ldy SDY
        jsr state_at
        cmp #COL_DONE
        bne .no
        jsr seg_next
        bne .cell
        sec
        rts
.no:    clc
        rts

; box_add: X = Rechtecknummer ($FF = keins): eine Kante mehr.  Bei 4 wird das Rechteck zum Fuellen vorgemerkt (genau einmal) und 100 Punkte gebucht.
;   Veraendert A, X, Y.
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
        lda #Fx.box
        jsr sound.play
        lda #1
        ldx #1
        jmp hud.bcd_add                 ; 100 Punkte
.out:   rts

; nb_boxes: T0 = gx, T1 = gy, T2 = Zelltyp (CT_H oder CT_V) eines Segments: NB0 und NB1 = die ein oder zwei angrenzenden Rechtecke ($FF = keins).
;   Aus b0 = 8 r + c der Zelle:  waagerecht: unten b0 (wenn gy < 20), oben b0 - 8 (wenn gy > 0);  senkrecht: rechts b0 (wenn gx < 32), links b0 - 1
;   (wenn gx > 0).  Veraendert A, X, Y, T3.
nb_boxes:
        ldy T1
        lda T0
        lsr
        lsr
        clc
        adc SEGBASE,y
        sta NB0                         ; b0
        lda T2
        cmp #CT_V
        beq .v
        lda NB0                         ; waagerecht
        sec
        sbc #BOX_COLS
        ldx T1
        bne .h1
        lda #$ff                        ; oberster Rand: kein Nachbar oben
.h1:    sta NB1
        cpx #GRID_H - 1
        bcc .ok
        lda #$ff                        ; unterster Rand: kein Nachbar unten
        sta NB0
.ok:    rts
.v:     lda NB0                         ; senkrecht
        sec
        sbc #1
        ldx T0
        bne .v1
        lda #$ff                        ; linker Rand: kein Nachbar links
.v1:    sta NB1
        cpx #GRID_W - 1
        bcc .ok2
        lda #$ff                        ; rechter Rand: kein Nachbar rechts
        sta NB0
.ok2:   rts

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
;   0 zaehlen und aufblitzen: SDP zaehlt die gelben Zeichen; deren Zeichen auf dem Bildschirm wird zum Treffer-Zeichen in Weiss (die Schattenkopie bleibt)
;   1 zuruecksetzen: Linienzeichen und blau, in Schattenkopie und Bildschirm.
;   Veraendert A, X, Y, PTR_A..PTR_E.
cell_op:
        jsr color_at                    ; PTR_A..PTR_E am Zeilenanfang
        ldy SDX
        ldx SDM
        bne .reset
        cmp #COL_DONE
        bne .r                          ; blau: nichts zu zaehlen
        inc SDP
        lda #CH_HIT
        sta (PTR_A),y
        lda #COL_WHITE
        sta (PTR_B),y
.r:     rts
.reset: ldx SDY
        lda ROW_TPL,x
        clc
        adc SDX
        tax
        lda TPL,x
        sta (PTR_A),y
        sta (PTR_D),y
        lda #COL_LINE
        sta (PTR_B),y
        sta (PTR_E),y
        rts

; seg_destroy: ein Gegner betritt das Innenzeichen T0 = gx, T1 = gy (T2 = Zelltyp CT_H oder CT_V).  Grenzt das Segment an ein gefuelltes Rechteck, geschieht
;   nichts.  Sonst, wenn mindestens ein Zeichen gefaerbt ist: aufblitzen (zwei Bilder, TED-Farbwechsel) mit Ton, das ganze Segment zuruecksetzen, Punkte
;   abziehen (1 je gefaerbtes Zeichen, 10 mehr bei abgeschlossenem Segment, nie unter 0), bei abgeschlossenem Segment BOXCNT der angrenzenden Rechtecke - 1.
;   Veraendert A, X, Y, T0..T3, T7, SDX..SDM, PTR_A..PTR_E (nicht T4, T5, ECUR).
seg_destroy:
        jsr seg_is_closed
        bcs .ret
        lda #0
        sta SDP
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
        beq .ret                        ; nichts gefaerbt: nichts zu zerstoeren
        lda #Fx.destroy
        jsr sound.play
        ldx #FLASH_FRAMES               ; zwei Bilder zeigen den Blitz (ohne Interrupt: nur Rasterabfrage; Figuren und Klang ruhen so lange)
.w:     jsr ted.wait_line
        dex
        bne .w
        inc SDM
        bne .pass
.fin:   ldx #BOX_W                      ; Anzahl der Zeichen des Segments
        lda SDP
        cpx SDP
        bne .pts                        ; nicht abgeschlossen
        clc
        adc #$10                        ; abgeschlossen: 10 Punkte mehr
        ldx NB0                         ; BOXCNT der angrenzenden Rechtecke - 1
        bmi .n1
        dec BOXCNT,x
.n1:    ldx NB1
        bmi .n2
        dec BOXCNT,x
.n2:
.pts:   pha                             ; Punktabzug merken
        jsr nodes_free                  ; freistehende gelbe Knoten zuruecksetzen
        pla
        jmp hud.bcd_sub
.ret:   rts

; nodes_free: alle 30 Knoten pruefen (Regel 16.2, Knoten): Ein gelber Knoten wird blau, wenn keine seiner Nachbarzellen gelb oder Fuellzeichen ist,
;   also kein gelbes Segment mehr anschliesst und kein angrenzendes Rechteck gefuellt ist.  Knoten zaehlen nicht fuer Punkte.
;   Veraendert A, X, Y, NODEX, NODEY, SDN, SDM, EQX, EQY, PTR_A..PTR_E (nicht T4, T5, ECUR).
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

; node_free: der Knoten (NODEX, NODEY).  Nur gelbe Knoten; steht die Figur oder ein Gegner darauf (auch als Ziel des laufenden Schritts: T4, T5), bleibt er.
;   Die acht Nachbarzellen im Gitter: Schattenfarbe gelb (angrenzendes Segment) oder Fuellzeichen (angrenzendes Rechteck gefuellt) = bleibt; sonst blau.
node_free:
        ldx NODEX
        ldy NODEY
        jsr color_at
        cmp #COL_DONE
        beq .yel
        rts
.yel:   lda NODEX
        cmp PX
        bne .p
        lda NODEY
        cmp PY
        bne .p
        rts                             ; die Figur steht darauf
.p:     lda NODEX
        cmp T4
        bne .e
        lda NODEY
        cmp T5
        bne .e
        rts                             ; ein Gegner betritt ihn gerade
.e:     ldx NODEX
        ldy NODEY
        jsr enemies.find
        bcc .free
        rts                             ; ein Gegner steht darauf
.free:
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
        tax
        lda NODEY
        clc
        adc SDN
        cmp #GRID_H
        bcs .nx
        tay
        jsr set_row
        txa
        tay
        lda (PTR_D),y                   ; Zeichen: Fuellmuster = Rechteck gefuellt
        cmp #CH_FILL0
        bcs .out
        lda (PTR_E),y
        cmp #COL_DONE
        beq .out                        ; gelbe Nachbarzelle
.nx:    inc SDM
        lda SDM
        cmp #2
        bne .rx
        inc SDN
        lda SDN
        cmp #2
        bne .ry
        ldy NODEY                       ; frei stehend: zuruecksetzen (das Zeichen bleibt)
        jsr set_row
        ldy NODEX
        lda #COL_LINE
        sta (PTR_B),y
        sta (PTR_E),y
.out:   rts
.endif

; paint_node: die Figur verlaesst die Zelle T0 = gx, T1 = gy: ist es ein Knoten, wird er gelb (keine Punkte; Schattenkopie und Bildschirm).
;   Veraendert A, X, Y, PTR_A..PTR_E.
paint_node:
        jsr cell_type
        cmp #CT_NODE
        bne .out
        ldx T0
        ldy T1
        jsr color_at
        ldy T0
        lda #COL_DONE
        sta (PTR_B),y
        sta (PTR_E),y
.out:   rts


; paint: die Figur betritt die Linienzelle T0 = gx, T1 = gy (vor dem Zeichnen der Figur).  Knoten und Innenraeume tun nichts.
;   Zelle violett: gelb faerben, 1 Punkt; ist damit das Segment voll (das geschieht je Segment genau einmal): 10 Punkte, Nachbarrechtecke (box_add);
;   schliessen zwei Rechtecke zugleich, kommen 200 Punkte dazu (Kettenreaktion, zusammen 400).
;   Veraendert A, X, Y, T2..T6, PTR_A, PTR_B.
paint:  jsr cell_type
        beq .out                        ; kein Linienzeichen
        cmp #CT_NODE
        bcs .out
        sta T2
        ldx T0
        ldy T1
        jsr color_at                    ; setzt PTR_B und PTR_E auf den Anfang der Zeile
        cmp #COL_DONE
        beq .out                        ; schon gelb: nichts zu tun
        lda #COL_DONE
        ldy T0
        sta (PTR_B),y
        sta (PTR_E),y
        lda #1
        ldx #0
        jsr hud.bcd_add                 ; 1 Punkt
        jsr seg_full
        bcc .out
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
;   (ist nur die Umkehr frei, gilt sie).  Gilt an Knoten, dort fuehrt jede Richtung im Gitter auf eine Linie.  Veraendert A, X, T7.
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
        bne .out
        lda T7                          ; nur die Umkehr ist frei
.out:   rts

; fill_colour: Farbe (mit Mehrfarben-Bit) der Innenzeile A = k (0..2) des Rechtecks in Rechteckzeile X = r, Spalte Y = c.  Der Farbton hat je Rechteckzeile
;   einen eigenen Wert (BOX_TONE), die Helligkeit steigt je Zeile des Blocks (Verlauf) und springt je nach (r + c) um eins (Muster-Abwechslung);
;   graue Hindernisbloecke (FILLGRAY) haben eine feste Farbe.  A = Farbe.  Veraendert A, T6.  X und Y bleiben.
fill_colour:
        sta T6
        lda FILLGRAY
        beq .col
        lda #COL_GRAY | COL_MULTI
        rts
.col:
.if FX_SHADE
        stx T7                          ; (T7 nur als Zwischenspeicher)
        tya
        clc
        adc T7
        and #1                          ; (r + c) und 1
        clc
        adc T6                          ; + Zeile k
        clc
        adc #3                          ; Helligkeit 3..6
        asl
        asl
        asl
        asl
        ldx T7
        ora BOX_TONE,x
        ora #COL_MULTI
        rts
.else
        lda BOX_TONE,x
        ora #$50 | COL_MULTI
        rts
.endif

; fill_row: eine Innenzeile eines Rechtecks fuellen (Zeichen und Farbe in Bildschirm und Schattenkopie).  X = Rechteckzeile r (0..4), Y = Rechteckspalte c (0..7),
;   A = Innenzeile k (0..2).  Zeichen CH_FILL0 + 9 * ((r + c) mod 4) + 3 * k + Spalte im Block (Mehrfarben-Muster mit Glanz oben/links und Schatten
;   unten/rechts, siehe charset.asm), Farbe aus fill_colour.  Veraendert A, X, Y, T0..T7, PTR_A, PTR_B, PTR_D, PTR_E.
.assert BOX_W == 3 && BOX_H == 3, "fill_row rechnet mit 4 * c und 4 * r"
fill_row:
        sta T6
        stx T4
        sty T5
        txa
        clc
        adc T5
        and #3
        tax
        lda FILLGRAY
        beq .pat
        ldx #2                          ; Hindernisbloecke: immer das Muster Mauer
.pat:   lda PAT_BASE,x                  ; erstes Zeichen des Musters
        ldx T6
        clc
        adc K3,x                        ; + 3 * k
        sta T2                          ; Zeichen der ersten Zelle der Zeile
        ldx T4
        ldy T5
        lda T6
        jsr fill_colour
        sta T3                          ; Farbe
        lda T5
        asl
        asl
        clc
        adc #1
        sta T0                          ; erste Innenspalte gx = 4 c + 1
        lda T4
        asl
        asl
        clc
        adc #1
        clc
        adc T6
        tay                             ; Zeile gy = 4 r + 1 + k
        jsr set_row
        ldy T0
        ldx #BOX_W
.cell:  lda T2
        sta (PTR_A),y
        sta (PTR_D),y
        lda T3
        sta (PTR_B),y
        sta (PTR_E),y
        inc T2
        iny
        dex
        bne .cell
        rts

; fill_row_b: wie fill_row, mit A = Rechtecknummer b (0..39), X = Innenzeile k.  Veraendert wie fill_row, dazu T7.
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
        stx T4
        sty T5
        lda #0
.k:     ldx T4
        ldy T5
        jsr fill_row
        lda T6
        clc
        adc #1
        cmp #BOX_H
        bcc .k
        rts

.if EXTRA_LAYOUT
; prefill: zweites Layout: die Rechtecke aus PRE_B (6 Eintraege) vorfuellen: grau, Kanten gelb; danach BOXCNT aller Rechtecke aus den gelben Kanten neu
;   zaehlen, BOXES_DONE = 6.  Voraussetzung: grid.draw ist gelaufen, BOXCNT und BOXES_DONE sind 0.  Keine Punkte, kein Ton.
;   Veraendert A, X, Y, T0..T7, PTR_A, PTR_B, PTR_D, PTR_E.
prefill:
        lda #1
        sta FILLGRAY
        ldx #5
.p:     stx ATMP
        lda PRE_B,x
        jsr fill_box_b                  ; Innenraum grau fuellen
        ldx ATMP
        lda PRE_B,x
        jsr edges_done                  ; vier Kanten gelb
        ldx ATMP
        dex
        bpl .p
        lda #0
        sta FILLGRAY
        ldx #NBOX - 1                   ; BOXCNT neu zaehlen: je Rechteck die Zahl der Kanten, deren Zellen gelb sind
.n:     stx ATMP
        jsr count_edges
        ldx ATMP
        sta BOXCNT,x
        dex
        bpl .n
        lda #6
        sta BOXES_DONE
        rts

; fill_box_b: ganzes Rechteck A = Nummer fuellen.  Veraendert wie fill_row, dazu T7.
fill_box_b:
        ldy #0
.d:     cmp #BOX_COLS
        bcc .e
        sbc #BOX_COLS
        iny
        bne .d
.e:     sta T5                          ; c
        tya
        tax                             ; X = r
        ldy T5                          ; Y = c
        jmp fill_box

; edges_done: die vier Kanten des Rechtecks A = Nummer gelb faerben (Schattenkopie und Bildschirm; die Knoten bleiben).  Veraendert A, X, Y, T0..T5, PTR_A..PTR_E.
edges_done:
        ldy #0
.d:     cmp #BOX_COLS
        bcc .e
        sbc #BOX_COLS
        iny
        bne .d
.e:     asl
        asl
        sta T4                          ; gx0 = 4 c
        tya
        asl
        asl
        sta T5                          ; gy0 = 4 r
        ldx #1                          ; Innenzelle 1..3 der waagerechten Kanten: oben (gy0) und unten (gy0 + 4), links (gx0) und rechts (gx0 + 4)
.k:     stx T3
        lda T4
        clc
        adc T3
        sta T0                          ; gx0 + k
        lda T5
        jsr .one                        ; oben
        lda T5
        clc
        adc #4
        jsr .one                        ; unten
        lda T4
        sta T0
        lda T5
        clc
        adc T3
        jsr .one                        ; links
        lda T4
        clc
        adc #4
        sta T0
        lda T5
        clc
        adc T3
        jsr .one                        ; rechts
        ldx T3
        inx
        cpx #4
        bne .k
        rts
.one:   tay                             ; Zelle (T0, A) gelb
        jsr set_row
        ldy T0
        lda #COL_DONE
        sta (PTR_B),y
        sta (PTR_E),y
        rts

; count_edges: X = Rechtecknummer.  A = Zahl der Kanten (0..4), deren Innenzeichen gelb sind (die mittlere Zelle jeder Kante genuegt: Kanten werden
;   immer ganz oder gar nicht gefaerbt, ausser waehrend des Spiels; hier nur nach prefill benutzt).  Veraendert A, X, Y, T0..T5, PTR_A..PTR_E.
count_edges:
        txa
        ldy #0
.d:     cmp #BOX_COLS
        bcc .e
        sbc #BOX_COLS
        iny
        bne .d
.e:     asl
        asl
        sta T4                          ; gx0
        tya
        asl
        asl
        sta T5                          ; gy0
        lda #0
        sta T3                          ; Zaehler
        lda T4
        clc
        adc #2
        sta T0                          ; Mitte der oberen/unteren Kante: gx0 + 2
        lda T5
        jsr .test                       ; oben
        lda T5
        clc
        adc #4
        jsr .test                       ; unten
        lda T4
        sta T0
        lda T5
        clc
        adc #2
        jsr .test                       ; links
        lda T4
        clc
        adc #4
        sta T0
        lda T5
        clc
        adc #2
        jsr .test                       ; rechts
        lda T3
        rts
.test:  tay
        jsr set_row
        ldy T0
        lda (PTR_E),y
        cmp #COL_DONE
        bne .no
        inc T3
.no:    rts
.endif

.if EXTRA_2P
; save: die Schattenkopie (Zeichen, dann Farben; je drei Seiten = 768 Byte, die ersten SHLEN gehoeren dazu) nach (PTR_C) sichern.
;   Veraendert A, X, Y, PTR_A, PTR_C.
save:   lda #<SHCH
        sta PTR_A
        lda #>SHCH
        sta PTR_A + 1
        jsr .one
        lda #<SHCOL
        sta PTR_A
        lda #>SHCOL
        sta PTR_A + 1
.one:   ldx #3
        ldy #0
.b:     lda (PTR_A),y
        sta (PTR_C),y
        iny
        bne .b
        inc PTR_A + 1
        inc PTR_C + 1
        dex
        bne .b
        rts

; load: die Schattenkopie aus (PTR_C) wiederherstellen (Gegenstueck zu save) und den Bildschirm daraus neu aufbauen.  Veraendert A, X, Y, T0, PTR_A..PTR_E.
load:   lda #<SHCH
        sta PTR_A
        lda #>SHCH
        sta PTR_A + 1
        jsr .one
        lda #<SHCOL
        sta PTR_A
        lda #>SHCOL
        sta PTR_A + 1
        jsr .one
        jmp redraw
.one:   ldx #3
        ldy #0
.b:     lda (PTR_C),y
        sta (PTR_A),y
        iny
        bne .b
        inc PTR_A + 1
        inc PTR_C + 1
        dex
        bne .b
        rts
.endif
}
