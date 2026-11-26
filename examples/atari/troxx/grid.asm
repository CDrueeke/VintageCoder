; =====================================================================================================================
; grid.asm - Gitter: Datenmodell, Zeichnen, Faerben, Segmente, Rechtecke, Fuellen
; =====================================================================================================================
; Herkunft: examples/vic20/troxx/grid.asm, Stand Commit 4697d074; Atari-Fassung nach examples/c64/troxx/grid.asm (Stand Commit d173cb4a).  Kopien (Pruefsumme,
;   siehe KOPIEN.txt): color_at, cell_type, box_add, box_filled, seg_is_closed, seg_next, seg_full, paint, legal_mask, fill_row_b.  Angepasst: set_row, draw, seg_full,
;   fill_row (Rahmen mit Abschraegung, Muster in der Mitte).  Neu: state_at, show, save, load.
;
; ZWECK     Alles, was das feste Gitter betrifft.  Die Gitterzelle (gx, gy) liegt auf Bildschirmspalte gx + GRID_COL0 und Bildschirmzeile
;           gy + GRID_ROW0 (Spalten 1..33, Zeilen 1..21; Zeile 0 Kopfzeile, Zeile 23 Meldungen, Seitenleiste in den Spalten 34..39).
;
; DATENMODELL DES GITTERS (Abschnitt 5 des Konzepts)
;   33 x 21 Zellen.  Linienzellen: gx mod 4 = 0 (senkrecht) oder gy mod 4 = 0 (waagerecht); Knoten: beides.  Innenraeume 3 x 3 Zellen.
;   Zwei Ebenen mit je einem Zweck, getrennt von der Anzeige:
;   1. GRIDST (Spielfeldzustand im RAM, 33 Byte je Zeile): Zustand jedes Linienzeichens (COL_LINE unberuehrt, COL_DONE abgefahren).  Der Atari hat
;      keinen Farbspeicher; die Farbe der Linien folgt aus dem Zeichen (unberuehrt Pixelwert 01, abgefahren Pixelwert 10).  Figuren sind Player/
;      Missiles und aendern weder GRIDST noch den Bildschirm; `show` setzt das Zeichen einer Zelle aus ihrem Zustand.
;   2. BOXCNT (40 Byte): je Rechteck die Zahl abgeschlossener Kanten; bei 4 wird gefuellt (Pruefung ohne Schleife ueber alle Rechtecke).
;   Ein Segment gilt als abgeschlossen, wenn alle seine Innenzeichen abgefahren sind; das tritt genau beim Faerben des letzten unberuehrten Zeichens
;   ein (jede weitere Zelle ist schon abgefahren und endet in paint sofort); ein Bitfeld SEGBITS ist darum nicht noetig (wie in der Referenzfassung).
;   Nummerierung: waagerecht H(r, c) = 8 r + c (r 0..5, c 0..7, 0..47); senkrecht V(r, c) = 48 + 9 r + c (r 0..4, c 0..8, 48..92);
;   Rechteck B(r, c) = 8 r + c (0..39).  Aus der Zelle: r = gy div 4, c = gx div 4 (Tabelle SEGBASE = 8 r; gx div 4 = zwei Rechtsschiebungen).
;   Kanten von B: oben H(r,c) = b, unten H(r+1,c) = b + 8, links V(r,c), rechts V(r,c+1).  Nachbarn eines Segments aus b0 = 8 r + c der Zelle:
;   H: unten b0 (wenn gy < 20), oben b0 - 8 (wenn gy > 0);  V: rechts b0 (wenn gx < 32), links b0 - 1 (wenn gx > 0).
; =====================================================================================================================

grid: {

; set_row: Y = Gitterzeile gy (0..20).  PTR_A = Bildschirm, PTR_B = Spielfeldzustand GRIDST, beide am Anfang der Zeile (Spalte gx = 0).  Veraendert A.
set_row:
        lda ROW_LO + GRID_ROW0,y        ; Bildschirmzeile gy + GRID_ROW0, Spalte GRID_COL0
        clc
        adc #GRID_COL0
        sta PTR_A
        lda ROW_HI + GRID_ROW0,y
        adc #0
        sta PTR_A + 1
        lda GST_LO,y
        sta PTR_B
        lda GST_HI,y
        sta PTR_B + 1
        rts

; draw: Gitter zeichnen (alle Linien unberuehrt).  Veraendert A, X, Y, T0, PTR_A, PTR_B.
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
        lda #COL_LINE
        sta (PTR_B),y
        inx
        iny
        cpy #GRID_W
        bne .cell
        inc T0
        lda T0
        cmp #GRID_H
        bne .row
        rts

; color_at: Zustand (Bit 0-2 von GRIDST) der Zelle X = gx, Y = gy.  A = Zustand.  X bleibt.  Veraendert A, Y, PTR_A, PTR_B.
color_at:
        jsr set_row
        txa
        tay
        lda (PTR_B),y
        and #7
        rts

; state_at: Zustand der Linienzelle X = gx, Y = gy (COL_LINE oder COL_DONE).  Figuren sind Player/Missiles, GRIDST gilt unmittelbar.  A = Zustand.
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

; Segment-Durchlauf: seg_first (T0 = gx, T1 = gy, T2 = Zelltyp einer Zelle des Segments) setzt SDX, SDY auf die erste Innenzelle und SDN auf die Anzahl (3);
;   seg_next rueckt um eine Zelle vor (Z = 1 nach der letzten).  Veraendert A.
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
;   Aus b0 = 8 r + c der Zelle:  waagerecht: unten b0 (wenn gy < 20), oben b0 - 8 (wenn gy > 0);  senkrecht: rechts b0 (wenn gx < 32), links b0 - 1
;   (wenn gx > 0).  Das Gitter ist nicht quadratisch: der Grenzwert je Richtung steht in NBL.  Veraendert A, X, Y, T3.
nb_boxes:
        ldy T1
        lda T0
        lsr
        lsr
        clc
        adc SEGBASE,y
        sta NB0                         ; b0
        ldx T1                          ; waagerecht: Koordinate gy, Nachbar oben = b0 - 8
        ldy #BOX_COLS
        lda #GRID_H - 1
        sta NBL
        lda T2
        cmp #CT_V
        bne .go
        ldx T0                          ; senkrecht: Koordinate gx, Nachbar links = b0 - 1
        ldy #1
        lda #GRID_W - 1
        sta NBL
.go:    sty T3
        lda NB0
        sec
        sbc T3
        cpx #0
        bne .b1
        lda #$ff                        ; Rand: kein Nachbar oben/links
.b1:    sta NB1
        cpx NBL
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

; seg_destroy: ein Gegner betritt das Innenzeichen T0 = gx, T1 = gy (T2 = Zelltyp CT_H oder CT_V).  Grenzt das Segment an ein gefuelltes Rechteck, geschieht nichts.
;   Sonst, wenn mindestens ein Zeichen abgefahren ist: Ton, das ganze Segment im Zustand zuruecksetzen (GRIDST), Punkte abziehen (1 je abgefahrenes Zeichen, 10 mehr bei
;   abgeschlossenem Segment, nie unter 0), bei abgeschlossenem Segment BOXCNT der angrenzenden Rechtecke - 1.  Der Effekt ist von der Logik getrennt und wartet nicht:
;   flash setzt Blockzeichen und die weisse Zeilenfarbe, tick ruft zwei Bilder spaeter flash_end (Zeichen zurueck, Zeilenfarbe zurueck).
;   Veraendert A, X, Y, T0..T3, T7, SDX..SDP, NB0, NB1, PTR_A, PTR_B (nicht T4..T6, ECUR).
seg_destroy:
        jsr seg_is_closed
        bcc .go
        rts                             ; geschuetzt
.go:    lda FLN
        cmp #4
        bcc .room
        jsr flash_end                   ; Warteschlange der Blitze voll: sofort beenden
.room:  lda #0
        sta SDP
        jsr seg_first
.cnt:   ldx SDX                         ; Durchgang 1: abgefahrene Zeichen zaehlen
        ldy SDY
        jsr state_at
        cmp #COL_DONE
        bne .n
        inc SDP
.n:     jsr seg_next
        bne .cnt
        lda SDP
        beq .ret                        ; nichts abgefahren: nichts zu zerstoeren
        lda #Fx.destroy
        jsr sound.play
        ldx FLN                         ; Blitz vormerken (bis zu vier Segmente je Gegnerschritt, z. B. an einem Knoten)
        lda T0
        sta FLX,x
        lda T1
        sta FLY,x
        lda T2
        sta FLTY,x
        inc FLN
        jsr flash                       ; Durchgang 2: aufblitzen (Anzeige)
        lda #2
        sta FLT
        jsr seg_first
.rs:    ldy SDY                         ; Durchgang 3: Zustand zuruecksetzen
        jsr set_row
        ldy SDX
        lda #COL_LINE
        sta (PTR_B),y
        jsr seg_next
        bne .rs
        ldx #BOX_W                      ; Anzahl der Zeichen des Segments
        lda T2
        cmp #CT_V
        bne .c2
        ldx #BOX_H
.c2:    lda SDP
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

; nodes_free: alle 54 Knoten pruefen (Regel 16.2, Knoten): Ein abgefahrener (gelber) Knoten wird zurueckgesetzt, wenn keine seiner acht Nachbarzellen abgefahren oder gefuellt
;   ist, also kein gelbes Segment mehr anschliesst und kein angrenzendes Rechteck gefuellt ist.  Knoten zaehlen nicht fuer Punkte.  Aus der Referenz (Stand 9fa111ad),
;   angepasst: Atari-Gitter, Zustand in GRIDST (gefuellte Zellen COL_FILL), Zeichen sofort zurueck.  Veraendert A, X, Y, NODEX, NODEY, SDN, SDM, PTR_A, PTR_B, EQX, EQY.
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

; node_free: der Knoten (NODEX, NODEY).  Nur abgefahrene Knoten; steht die Figur oder ein Gegner darauf (eigene Pruefung, Player/Missile), bleibt er.  Die acht Nachbarzellen im
;   Gitter: Zustand abgefahren (angrenzendes Segment) oder gefuellt (angrenzendes Rechteck) = bleibt; sonst Zustand und Zeichen zurueck.
node_free:
        ldx NODEX
        ldy NODEY
        jsr color_at
        cmp #COL_DONE
        beq .chk
.out:   rts
.chk:   lda NODEX
        cmp PX
        bne .nop
        lda NODEY
        cmp PY
        beq .out                        ; die Figur steht auf dem Knoten
.nop:   ldx NODEX
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
        lda (PTR_B),y
        and #7
        cmp #COL_DONE
        beq .out                        ; abgefahrene Nachbarzelle
        cmp #COL_FILL
        beq .out                        ; gefuelltes Rechteck
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
        lda #COL_LINE
        sta (PTR_B),y
        ldx NODEY
        lda ROW_TPL,x
        clc
        adc NODEX
        tax
        lda TPL,x                       ; Grundzeichen des Knotens
        sta (PTR_A),y
        rts

; flash: das Segment (T0, T1, T2) aufblitzen lassen: abgefahrene Zeichen werden Blockzeichen, die Bildschirmzeilen des Segments bekommen die Blitzfarbe (RC_P2).
;   Veraendert A, X, Y, SDX..SDN, PTR_A, PTR_B.
flash:  jsr seg_first
.f:     ldy SDY
        lda #$0e
        sta RC_P2 + GRID_ROW0,y         ; Zeilenfarbe weiss (der DLI setzt sie in der Zeile)
        ldx SDX
        jsr state_at
        cmp #COL_DONE
        bne .nx
        lda #CH_BLOCK
        ldy SDX
        sta (PTR_A),y
.nx:    jsr seg_next
        bne .f
        rts

; flash_end: das Aufblitzen aller vorgemerkten Segmente beenden (nur wenn FLN > 0): Grundzeichen der Zellen des Segments zurueck, Zeilenfarben aus GAME_P2, der Gegenstand wird neu gezeichnet.
;   Ruft tick nach zwei Bildern; veraendert A, X, Y, SDX..SDN, PTR_A, PTR_B (T0..T2 bleiben).
flash_end:
        lda FLN
        beq .none
        lda T0
        pha
        lda T1
        pha
        lda T2
        pha
.q:     dec FLN
        ldx FLN
        lda FLX,x
        sta T0
        lda FLY,x
        sta T1
        lda FLTY,x
        sta T2
        jsr seg_first
.e:     ldy SDY
        jsr set_row
        ldx SDY
        lda ROW_TPL,x
        clc
        adc SDX
        tax
        lda TPL,x                       ; Grundzeichen der unberuehrten Linie
        ldy SDX
        sta (PTR_A),y
        lda SDY
        clc
        adc #GRID_ROW0
        tax
        lda GAME_P2,x
        sta RC_P2,x
        jsr seg_next
        bne .e
        lda FLN
        bne .q
        sta FLT
.if EXTRA_ITEMS
        lda IT_KIND
        beq .noit
        jsr items.draw_on               ; lag der Gegenstand auf dem Segment: wieder zeichnen
.noit:
.endif
        pla
        sta T2
        pla
        sta T1
        pla
        sta T0
.none:  rts
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

; paint_node: die Figur betritt die Zelle T0 = gx, T1 = gy: ist es ein Knoten, wird er abgefahren (Zustand COL_DONE; das Zeichen zeigt grid.show).  Nur Optik und
;   Knotenregel: die Spielwertung liest Knoten nicht.  Die Figur ist ein Player/Missile-Objekt (wie in der C64-Fassung wird beim Betreten gefaerbt).
;   Veraendert A, X, Y, PTR_A, PTR_B.
paint_node:
        jsr cell_type
        cmp #CT_NODE
        bne .out
        ldx T0
        ldy T1
        jsr color_at
        lda #COL_DONE
        ldy T0
        sta (PTR_B),y
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

; fill_row: eine Innenzeile eines Rechtecks fuellen.  X = Rechteckzeile r (0..4), Y = Rechteckspalte c (0..7), A = Innenzeile k (0..2).
;   Rahmenzeichen mit Abschraegung (FILLB, hell oben und links, dunkel unten und rechts), in der Mitte ein Muster CH_FILL0 + ((r + c) mod 4);
;   die Farbe kommt aus der Zeilenfarbe des DLI (je Rechteckzeile ein Farbton).  Veraendert A, X, Y, T0..T6, PTR_A, PTR_B.
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
        lda FILLP,x
        sta T2                          ; Muster fuer die Mitte
        lda T6
        asl
        clc
        adc T6
        sta T3                          ; Abstand in FILLB: k * 3
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
        adc #1
        clc
        adc T6
        tay                             ; Zeile gy = 4 r + 1 + k
        jsr set_row
        ldy T0
        ldx T3
.cell:  lda FILLB,x
        bpl .put
        lda T2                          ; Marke: Mitte des Rechtecks
.put:   sta (PTR_A),y
        pha
        lda #COL_FILL
        sta (PTR_B),y                   ; Zustand: gefuellt (node_free beachtet angrenzende gefuellte Rechtecke)
        pla
        iny
        inx
        txa
        sec
        sbc T3
        cmp #BOX_W
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

; show: Zeichen der Zelle T0 = gx, T1 = gy aus ihrem Zustand setzen (Linien: abgefahren oder unberuehrt; Knoten: Grundzeichen); Innenraeume tun nichts.
;   Stellt auch eine Zelle wieder her, auf der ein Gegenstand lag.  Veraendert A, X, Y, T3, T4, PTR_A, PTR_B.
show:   jsr cell_type
        beq .out
        sta T3
        ldx T0
        ldy T1
        jsr color_at                    ; PTR_A = Bildschirmzeile, A = Zustand
        sta T4
        ldy T1
        lda ROW_TPL,y
        clc
        adc T0
        tax
        lda TPL,x                       ; Grundzeichen (unberuehrt)
        ldx T3
        cpx #CT_NODE
        beq .node
        ldy T4
        cpy #COL_DONE
        bne .put
        lda LINECH_D - 1,x              ; abgefahrene Linie
.put:   ldy T0
        sta (PTR_A),y
.out:   rts
.node:  ldy T4                          ; Knoten: gelb, wenn abgefahren (NODE_FROM -> NODE_TO)
        cpy #COL_DONE
        bne .put
        ldx #8
.nf:    cmp NODE_FROM,x
        beq .nt
        dex
        bpl .nf
        bmi .put
.nt:    lda NODE_TO,x
        bne .put

.if EXTRA_2P
; save: das Gitter (je Zeile 33 Bildschirmcodes, dann 33 Zustaende) nach (PTR_C) sichern.  Veraendert A, Y, T0, PTR_A..PTR_C.
save:   lda #0
        sta T0
.row:   ldy T0
        jsr set_row
        ldy #GRID_W - 1
.s:     lda (PTR_A),y
        sta (PTR_C),y
        dey
        bpl .s
        jsr adv
        ldy #GRID_W - 1
.c:     lda (PTR_B),y
        sta (PTR_C),y
        dey
        bpl .c
        jsr adv
        inc T0
        lda T0
        cmp #GRID_H
        bne .row
        rts

; load: das Gitter aus (PTR_C) wiederherstellen (Gegenstueck zu save).  Veraendert A, Y, T0, PTR_A..PTR_C.
load:   lda #0
        sta T0
.row:   ldy T0
        jsr set_row
        ldy #GRID_W - 1
.s:     lda (PTR_C),y
        sta (PTR_A),y
        dey
        bpl .s
        jsr adv
        ldy #GRID_W - 1
.c:     lda (PTR_C),y
        sta (PTR_B),y
        dey
        bpl .c
        jsr adv
        inc T0
        lda T0
        cmp #GRID_H
        bne .row
        rts

; adv: PTR_C um eine Zeile (GRID_W Byte) weiter.  Veraendert A.
adv:    lda PTR_C
        clc
        adc #GRID_W
        sta PTR_C
        bcc .o
        inc PTR_C + 1
.o:     rts
.endif
}
