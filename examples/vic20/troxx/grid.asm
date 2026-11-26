; =====================================================================================================================
; grid.asm - Gitter: Datenmodell, Zeichnen, Faerben, Segmente, Rechtecke, Fuellen
; =====================================================================================================================
; ZWECK     Alles, was das feste Gitter betrifft.  Die Gitterzelle (gx, gy) liegt auf Bildschirmspalte gx und Bildschirmzeile gy + 1
;           (Zeile 0 = Statuszeile, Zeile 22 = Anzeige).
;
; DATENMODELL DES GITTERS (Abschnitt 5 des Konzepts)
;   21 x 21 Zellen.  Linienzellen: gx mod 4 = 0 (senkrecht) oder gy mod 5 = 0 (waagerecht); Knoten: beides.  Innenraeume 3 x 4 Zellen.
;   Drei Ebenen, jede mit genau einem Zweck:
;   1. Farbspeicher: Zustand jedes Linienzeichens (blau unberuehrt, gelb abgefahren).  Steht eine Figur auf der Zelle, zeigt der Farbspeicher
;      die Figurenfarbe; den Zustand merkt sich dann der Gegner (E_UNDER); die Figur selbst faerbt die Zelle schon beim Betreten gelb.
;   2. BOXCNT (20 Byte): je Rechteck die Zahl abgeschlossener Kanten; bei 4 wird gefuellt (Pruefung ohne Schleife ueber alle Rechtecke).
;   Ein Segment gilt als abgeschlossen, wenn alle seine Innenzeichen gelb sind; das tritt genau beim Faerben des letzten blauen Zeichens ein
;   (jede weitere Zelle ist schon gelb und endet in paint sofort).  Ein eigenes Bitfeld SEGBITS (49 Bit) gegen doppelte Wertung wurde im
;   Spike umgesetzt und getestet, aber als ueberfluessig wieder entfernt (spart 90 Byte; die Nummerierung bleibt im Kommentar und in nb_boxes).
;   Nummerierung: waagerecht H(r, c) = 5 r + c (r 0..4, c 0..4, 0..24); senkrecht V(r, c) = 25 + 6 r + c (r 0..3, c 0..5, 25..48);
;   Rechteck B(r, c) = 5 r + c (0..19).  Aus der Zelle: r = gy div 5, c = gx div 4 (Tabelle SEGBASE = 5 r; gx div 4 = zwei Rechtsschiebungen).
;   Kanten von B: oben H(r,c) = b, unten H(r+1,c) = b + 5, links V(r,c), rechts V(r,c+1).  Nachbarn eines Segments aus b0 = 5 r + c der Zelle:
;   H: unten b0 (wenn gy < 20), oben b0 - 5 (wenn gy > 0);  V: rechts b0 (wenn gx < 20), links b0 - 1 (wenn gx > 0).
; =====================================================================================================================

grid: {

; set_row: Y = Gitterzeile gy (0..20).  PTR_A = Bildschirm, PTR_B = Farbspeicher, beide am Anfang der Zeile.  Veraendert A.
;   Die Low-Bytes der Zeilenanfaenge sind in Bildschirm und Farbspeicher gleich ($1E00 / $9600), nur das High-Byte unterscheidet sich.
set_row:
        lda ROW_LO + 1,y                ; Bildschirmzeile gy + 1
        sta PTR_A
        sta PTR_B
        lda #>SCREEN_RAM
        cpy #11                         ; ab Bildschirmzeile 12 ueber die Seitengrenze ($1F..)
        bcc .hi
        lda #(>SCREEN_RAM) + 1
.hi:    sta PTR_A + 1
        clc
        adc #COLOR_DELTA
        sta PTR_B + 1
        rts

; char_at: X = gx, Y = gy.  A = Zeichen der Gitterzelle (Linie, Knoten oder leerer Innenraum).  X bleibt.  Veraendert A, Y, T3.
char_at:
        lda ROWKIND,y
        bmi .inner
        sta T3
        txa
        and #3
        bne .hline
        txa                             ; Knoten: Reihe der Zeilenart + gx div 4
        lsr
        lsr
        clc
        adc T3
        tay
        lda NODETAB,y
        rts
.hline: lda #CH_HLINE
        rts
.inner: txa
        and #3
        beq .vline
        lda #CH_SPACE
        rts
.vline: lda #CH_VLINE
        rts

; draw: Gitter zeichnen (alle Linien blau).  Veraendert A, X, Y, T0, T3, PTR_A, PTR_B.
draw:
        lda #0
        sta T0                          ; gy
.row:   ldy T0
        jsr set_row
        ldx #0
.cell:  ldy T0
        jsr char_at
        pha
        txa
        tay
        pla
        sta (PTR_A),y
        lda #COL_LINE
        sta (PTR_B),y
        inx
        cpx #GRID_W
        bne .cell
        inc T0
        lda T0
        cmp #GRID_H
        bne .row
        rts

; put: ein Zeichen mit Farbe in eine Gitterzelle.  A = Zeichen, X = Farbe, T0 = gx, T1 = gy.  Veraendert A, Y, PTR_A, PTR_B.
put:    pha
        ldy T1
        jsr set_row
        ldy T0
        pla
        sta (PTR_A),y
        txa
        sta (PTR_B),y
        rts

; color_at: Farbe (Bit 0-2) der Zelle X = gx, Y = gy.  A = Farbe.  X bleibt.  Veraendert A, Y, PTR_A, PTR_B.
color_at:
        jsr set_row
        txa
        tay
        lda (PTR_B),y
        and #7
        rts

; state_at: Zustand der Linienzelle X = gx, Y = gy (blau oder gelb): steht ein Gegner darauf, gilt seine gemerkte Farbe.  A = Zustand.
state_at:
        sty EQY
        jsr color_at
        cmp #COL_DONE
        beq .out
        cmp #COL_LINE
        beq .out
        ldy EQY                         ; color_at hat Y veraendert
        jmp enemies.under_at            ; Gegnerfarbe: Zustand aus dem Gegner
.out:   rts

; cell_type: T0 = gx, T1 = gy.  A = CT_NONE, CT_H, CT_V oder CT_NODE (Z-Flag passend zu A).  Veraendert A, Y.
cell_type:
        ldy T1
        lda ROWKIND,y
        bpl .lr                         ; Zeile mit waagerechter Linie
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

;   Segment-Durchlauf: seg_first (T0 = gx, T1 = gy, T2 = Zelltyp einer Zelle des Segments) setzt SDX, SDY auf die erste Innenzelle und SDN auf die Anzahl
;   (3 oder 4); seg_next rueckt um eine Zelle vor (Z = 1 nach der letzten).  Veraendert A, Y.
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
.v:     lda T0                          ; senkrecht: erste Innenzelle gy = 5 r + 1, 4 Zellen
        sta SDX
        ldy T1
        lda SEGBASE,y                   ; 5 r
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

; seg_full: T0, T1, T2 wie seg_first.  C = 1, wenn alle Innenzeichen des Segments gelb sind.  Veraendert A, X, Y, SDX, SDY, SDN, PTR_A, PTR_B.
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
        lda #Fx.box
        jsr sound.play
        lda #1
        ldx #1
        jmp hud.bcd_add                 ; 100 Punkte
.out:   rts

; nb_boxes: T0 = gx, T1 = gy, T2 = Zelltyp (CT_H oder CT_V) eines Segments: NB0 und NB1 = die ein oder zwei angrenzenden Rechtecke ($FF = keins).
;   Aus b0 = 5 r + c der Zelle:  waagerecht: unten b0 (wenn gy < 20), oben b0 - 5 (wenn gy > 0);  senkrecht: rechts b0 (wenn gx < 20), links b0 - 1
;   (wenn gx > 0).  Veraendert A, X, Y, T3.
.assert GRID_W == GRID_H
nb_boxes:
        ldy T1
        lda T0
        lsr
        lsr
        clc
        adc SEGBASE,y
        sta NB0                         ; b0
        ldx T1                          ; waagerecht: Koordinate gy, Nachbar oben = b0 - 5
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
        cpx #GRID_H - 1
        bcc .ok
        lda #$ff                        ; Rand: kein Nachbar unten/rechts
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
;   0 zaehlen und aufblitzen: SDP zaehlt die gelben Zeichen (Gegner und Figur auf der Zelle zaehlen mit ihrer gemerkten Farbe, die Figur als gelb);
;     gelbe Linienzeichen werden zum Treffer-Zeichen (gelb)
;   1 zuruecksetzen: Linienzeichen, blau; ein Gegner auf der Zelle merkt sich blau; die Figur bleibt stehen.
;   Veraendert A, X, Y, PTR_A, PTR_B.
cell_op:
        jsr color_at                    ; PTR_A, PTR_B am Zeilenanfang
        cmp #COL_DONE
        beq .line
        cmp #COL_LINE
        beq .line
        ldy SDY                         ; Figur oder Gegner auf der Zelle
        lda SDM
        bne .m1
        jsr enemies.under_at
        cmp #COL_DONE
        bne .r
        inc SDP
.r:     rts
.m1:    jsr enemies.find
        bcc .r
        lda #COL_LINE
        sta EN + E_UNDER,x
        rts
.line:  ldy SDX
        ldx SDM
        bne .reset
        cmp #COL_DONE
        bne .r                          ; blau: nichts zu zaehlen
        inc SDP
        lda #CH_FLASH
        sta (PTR_A),y
        rts
.reset: ldx SDX
        ldy SDY
        jsr char_at
        ldy SDX
        sta (PTR_A),y
        lda #COL_LINE
        sta (PTR_B),y
        rts

; seg_destroy: ein Gegner betritt das Innenzeichen T0 = gx, T1 = gy (T2 = Zelltyp CT_H oder CT_V).  Grenzt das Segment an ein gefuelltes Rechteck, geschieht nichts.
;   Sonst, wenn mindestens ein Zeichen gefaerbt ist: aufblitzen (2 Bilder) mit Ton, das ganze Segment zuruecksetzen, Punkte abziehen (1 je gefaerbtes
;   Zeichen, 10 mehr bei abgeschlossenem Segment, nie unter 0), bei abgeschlossenem Segment BOXCNT der angrenzenden Rechtecke - 1.
;   Veraendert A, X, Y, T0..T3, T7, SDX..SDM, PTR_A, PTR_B (nicht T4, T5, ECUR).
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
        jsr vic.wait_frame
        jsr vic.wait_frame
        inc SDM
        bne .pass
.fin:   ldx #BOX_W                      ; Anzahl der Zeichen des Segments
        lda T2
        cmp #CT_V
        bne .cnt
        ldx #BOX_H
.cnt:   lda SDP
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

; nodes_free: alle 30 Knoten pruefen (Regel 16.2, Knoten): Ein gelber Knoten wird blau, wenn keine seiner acht Nachbarzellen gelb oder ein
;   Fuellzeichen ist, also kein gelbes Segment mehr anschliesst und kein angrenzendes Rechteck gefuellt ist.  Knoten zaehlen nicht fuer Punkte.
;   Veraendert A, X, Y, NODEX, NODEY, SDN, SDM, PTR_A, PTR_B.
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

; node_free: der Knoten (NODEX, NODEY).  Ablauf: nur gelbe Knoten (Farbe; steht die Figur oder ein Gegner darauf, bleibt er); die acht
;   Nachbarzellen im Gitter: Farbe gelb (angrenzendes Segment) oder Fuellzeichen 38..41 (angrenzendes Rechteck gefuellt) = bleibt; sonst blau.
;   Grenzfall: steht die Figur oder ein Gegner auf einer Nachbarzelle, zaehlt diese Zelle nicht als gelb (ihre Farbe ist die der Figur).
node_free:
        ldx NODEX
        ldy NODEY
        jsr color_at
        cmp #COL_DONE
        bne .out
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
        lda (PTR_A),y                   ; Zeichen: Fuellmuster = Rechteck gefuellt
        cmp #CH_FILL0
        bcc .col
        cmp #CH_FILL0 + 4
        bcc .out
.col:   lda (PTR_B),y
        and #7
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
        ldy NODEY                       ; frei stehend: zuruecksetzen (Zeichen bleibt)
        jsr set_row
        ldy NODEX
        lda #COL_LINE
        sta (PTR_B),y
.out:   rts
.endif

; paint: die Figur betritt die Linienzelle T0 = gx, T1 = gy (vor dem Zeichnen der Figur).  Knoten und Innenraeume tun nichts.
;   Zelle blau: gelb faerben, 1 Punkt; ist damit das Segment voll (das geschieht je Segment genau einmal): 10 Punkte, Nachbarrechtecke (box_add);
;   schliessen zwei Rechtecke zugleich, kommen 200 Punkte dazu (Kettenreaktion, zusammen 400).
;   Veraendert A, X, Y, T2..T6, PTR_A, PTR_B.
paint:  jsr cell_type
        beq .out                        ; kein Linienzeichen
        cmp #CT_NODE
        bcs .out
        sta T2
        ldx T0
        ldy T1
        jsr color_at                    ; setzt PTR_B auf den Anfang der Zeile
        cmp #COL_DONE
        beq .out                        ; schon gelb: nichts zu tun
        lda #COL_DONE
        ldy T0
        sta (PTR_B),y
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

; legal_mask: T4 = gx, T5 = gy, T6 = bisherige Richtung.  A = Maske der Richtungen (Bit = Dir) mit Zielzelle im Gitter, ohne die Umkehr.  Gilt an Knoten, dort fuehrt jede Richtung im Gitter auf eine Linie.  Veraendert A, X, T7.
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

; fill_row: eine Innenzeile eines Rechtecks fuellen.  X = Rechteckzeile r (0..3), Y = Rechteckspalte c (0..4), A = Innenzeile k (0..3).
;   Muster 38 + ((r + c) mod 4), Farbe nach Zeile (BOX_COLOR).  Veraendert A, X, Y, T0..T6, PTR_A, PTR_B.
.assert BOX_W == 3 && BOX_H == 4, "fill_row rechnet mit 4 * c und 5 * r"
fill_row:
        sta T6
        stx T4
        sty T5
        txa
        clc
        adc T5
        and #3
        clc
        adc #CH_FILL0
        sta T2                          ; Zeichen
        lda BOX_COLOR,x
        sta T3                          ; Farbe
        tya
        asl
        asl
        clc
        adc #1
        sta T0                          ; erste Innenspalte gx = 4 c + 1
        lda T4
        asl
        asl
        clc
        adc T4
        clc
        adc #1
        clc
        adc T6
        tay                             ; Zeile gy = 5 r + 1 + k
        jsr set_row
        ldy T0
        ldx #BOX_W
.cell:  lda T2
        sta (PTR_A),y
        lda T3
        sta (PTR_B),y
        iny
        dex
        bne .cell
        rts

; fill_row_b: wie fill_row, mit A = Rechtecknummer b (0..19), X = Innenzeile k.  Veraendert wie fill_row, dazu T7.
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

}
