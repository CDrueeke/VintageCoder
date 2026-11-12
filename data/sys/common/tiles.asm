// SPDX-License-Identifier: GPL-3.0-or-later
// Kachel-Bibliothek, systemneutraler Teil (Konzept Kachel- und Screen-Editor, Abschnitt 6).
// Nicht direkt einbinden: #include <c64/tiles.asm> (oder das Verzeichnis des Zielsystems) setzt die Systemkonstanten
// und bindet diese Datei ein.
//
// ---------------------------------------------------------------- Überblick
// Eine Kachel ist B x H Zeichenzellen. Die Daten kommen aus dem Export des Kachel-Editors (Format: Anhang A im Konzept):
//   <satz>_c0, _c1, ... eine Tabelle je Zelle (planar), Index = Kachelnummer, unmittelbar hintereinander im Speicher
//   <satz>_col         Farbe je Kachel     <satz>_props   Eigenschaftenbyte je Kachel
//   <satz>_tw, _th, _count                 Kachelbreite, -höhe (Zellen) und Anzahl (Konstanten)
//   <bank>_screens_lo/_hi                  Adressen der Kartenfelder (Kachelnummern, zeilenweise), Screen n = Eintrag n
//   <bank>_tiles_w, _tiles_h               Kartengröße in Kacheln (für alle Screens der Bank gleich)
//
//   #include <c64/tiles.asm>
//   .include "build/export/hg.asm"
//   .include "build/export/levels.asm"
//           tiles_use hg_c0, hg_col, hg_props, hg_tw, hg_th, hg_count, levels_screens_lo, levels_screens_hi, levels_tiles_w, levels_tiles_h
//           ...
//           screen_draw 1               ; Screen 1 wählen und komplett zeichnen
//           tile_draw 5, 10, 3          ; Kachel 5 an Kachelposition (10, 3), Konstanten, ohne Unterprogramm
//           lda #1 : ldx #4 : ldy #6
//           jsr tile_put                ; A = Kachel, X = Spalte, Y = Zeile (Kachelposition)
//   tiles_runtime                       ; EINMAL, hinter dem Programm: die Unterprogramme
//
// Warum tiles_use so lang ist: Der Assembler kann aus "hg" kein "hg_c0" bilden (Makroargumente sind Textersatz ohne
// Zusammensetzen von Namen). Die Zeile lässt sich aus dem Export erzeugen; ohne Screen-Bank genügt tiles_use_set.
//
// ---------------------------------------------------------------- Befehle
//   tiles_use c0, col, props, tw, th, count, lo, hi, mw, mh   verbindet Satz und Bank (beliebig oft; gilt für alles, was danach steht)
//   tiles_use_set c0, col, props, tw, th, count               nur den Satz
//   tiles_use_bank lo, hi, mw, mh                             nur die Bank
//   tile_draw tn, tx, ty             Makro, Konstanten: Zeichen und Farbe der Kachel tn nach (tx, ty) (Kachelposition)
//   tile_draw_color tn, tx, ty, f    wie tile_draw, aber Farbe f statt der Kachelfarbe
//   tile_fill tn, tx, ty, w, h       Makro, entrollt: Rechteck w x h Kacheln (Konstanten)
//   screen_select sn                 Makro: Screen sn wählen (kopiert die Karte, wenn TILE_MAPRAM gesetzt ist)
//   screen_draw [sn]                 Makro: optional Screen wählen, dann die ganze Kachelebene zeichnen
//   jsr tile_put                     A = Kachel, X = Spalte, Y = Zeile; verändert A, X, Y, tz_sp, tz_cp, tz_t
//   jsr screen_select_a              A = Screen-Nummer
//   jsr screen_draw_all              alle Kachelzeilen des gewählten Screens
//   jsr screen_draw_rows             A = erste Kachelzeile, X = Anzahl der Zeilen (mindestens 1)
//   jsr screen_map_get               X = Spalte, Y = Zeile (Karte, in Kacheln) -> A = Kachelnummer
//   jsr tile_prop                    X = Spalte, Y = Zeile -> A = Eigenschaftenbyte der Kachel an dieser Stelle
// Register: A, X, Y und Flags werden verändert. Zeropage: nur die Gruppe TILE_ZP (8 Byte, Standard $F7-$FE).
//
// ---------------------------------------------------------------- Schalter (vor dem Einbinden setzen, oder vcc asm -D NAME=wert)
//   TILE_SMC=1     screen_draw/screen_draw_rows ändern ihren eigenen Code (schneller, nur im RAM, nicht im ROM/Modul).
//                  Ohne den Schalter (oder mit 0) arbeitet die Zeigerversion über Zeropage-Zeiger: ROM-tauglich.
//                  tile_put arbeitet in beiden Fällen mit Zeigern.
//   TILE_ZP=adr    Zeropage-Gruppe, Standard $F7 (belegt adr bis adr+7)
//   TILE_SCREEN    Adresse des Bildschirmspeichers (Standard laut Systemdatei)
//   TILE_COLORRAM  Adresse des Farbspeichers; ohne Farbspeicher TILE_NOCOLOR definieren
//   TILE_MAPRAM    Adresse eines RAM-Puffers: screen_select kopiert die Karte dorthin (Spiellogik darf sie ändern;
//                  screen_draw zeichnet dann aus dem Puffer). Ohne den Schalter liest alles direkt aus dem Kartenfeld.
//
// Grenzen: höchstens 256 Kacheln (Index 1 Byte), Kartenbreite höchstens 128 Kacheln; Kartenfeld bis 64 KB.
// Zeigerversion von screen_draw: Kachelhöhe höchstens 6 (Offset in einem Byte). tile_put und der Rest haben kein Limit.

// ---- Voreinstellungen
.ifndef TILE_ZP
.const TILE_ZP = $f7
.endif

.ifndef TILE_SMC
.const TILE_SMC = 0
.endif

// Zyklen von lda/sta (absolut) für die Zyklenbudgets der Makros: 6502 4 und 4; der MEGA65 (40-MHz-Tabelle des 45GS02) liest in 5
.ifndef TILE_CYC_LDA
.const TILE_CYC_LDA = 4
.endif
.ifndef TILE_CYC_STA
.const TILE_CYC_STA = 4
.endif

.ifndef TILE_COLS
.error "TILE_COLS fehlt: <system>/tiles.asm einbinden, nicht common/tiles.asm"
.endif

// ---- Zeropage-Belegung (8 Byte)
.const tz_sp  = TILE_ZP          ; Zeiger auf die Bildschirmzelle (SMC: Zeilenadresse)
.const tz_cp  = TILE_ZP + 2      ; Zeiger auf die Farbzelle
.const tz_mp  = TILE_ZP + 4      ; Zeiger in der Karte
.const tz_t   = TILE_ZP + 6      ; Zwischenspeicher
.const tz_idx = TILE_ZP + 7      ; gewählter Screen (muss zwischen den Aufrufen erhalten bleiben)

// ---- Bindung an Satz und Bank (Variablen, damit tiles_use mehrfach aufgerufen werden darf)
.var TILE_C0    = 0
.var TILE_COL   = 0
.var TILE_PROPS = 0
.var TILE_TW    = 1
.var TILE_TH    = 1
.var TILE_COUNT = 1
.var TILE_LO    = 0
.var TILE_HI    = 0
.var TILE_MW    = 1
.var TILE_MH    = 1
.var tz_pg      = 0
.var tz_sz      = 0

.macro tiles_use_set(c0, col, props, tw, th, count)
        .set TILE_C0 = c0
        .set TILE_COL = col
        .set TILE_PROPS = props
        .set TILE_TW = tw
        .set TILE_TH = th
        .set TILE_COUNT = count
.endmacro

.macro tiles_use_bank(lo, hi, mw, mh)
        .set TILE_LO = lo
        .set TILE_HI = hi
        .set TILE_MW = mw
        .set TILE_MH = mh
.endmacro

.macro tiles_use(c0, col, props, tw, th, count, lo, hi, mw, mh)
        tiles_use_set c0, col, props, tw, th, count
        tiles_use_bank lo, hi, mw, mh
.endmacro

// ---------------------------------------------------------------- Zeichnen mit Konstanten
// Adresse der Zelle (cy, cx) der Kachel an (tx, ty): TILE_SCREEN + (ty*TH + cy)*COLS + tx*TW + cx
// Je Zelle: lda <satz>_c<k> + tn ; sta Bildschirm.  Dazu je Zelle Farbe (4 + 4 Zyklen; 2x2 mit Farbe: 52 Zyklen, 39 Byte).
.macro tile_draw(tn, tx, ty)
.cycles tile_draw max=((TILE_CYC_LDA+2*TILE_CYC_STA)*TILE_TW*TILE_TH+TILE_CYC_LDA)
        .repeat TILE_TW * TILE_TH, k
        lda TILE_C0 + k * TILE_COUNT + tn
        sta TILE_SCREEN + ((ty) * TILE_TH + k / TILE_TW) * TILE_COLS + (tx) * TILE_TW + k % TILE_TW
        .endrepeat
.ifndef TILE_NOCOLOR
        lda TILE_COL + tn
        .repeat TILE_TW * TILE_TH, k
        sta TILE_COLORRAM + ((ty) * TILE_TH + k / TILE_TW) * TILE_COLS + (tx) * TILE_TW + k % TILE_TW
        .endrepeat
.endif
.endcycles
.endmacro

.macro tile_draw_color(tn, tx, ty, fc)
.cycles tile_draw_color max=((TILE_CYC_LDA+2*TILE_CYC_STA)*TILE_TW*TILE_TH+2)
        .repeat TILE_TW * TILE_TH, k
        lda TILE_C0 + k * TILE_COUNT + tn
        sta TILE_SCREEN + ((ty) * TILE_TH + k / TILE_TW) * TILE_COLS + (tx) * TILE_TW + k % TILE_TW
        .endrepeat
.ifndef TILE_NOCOLOR
        lda #fc
        .repeat TILE_TW * TILE_TH, k
        sta TILE_COLORRAM + ((ty) * TILE_TH + k / TILE_TW) * TILE_COLS + (tx) * TILE_TW + k % TILE_TW
        .endrepeat
.endif
.endcycles
.endmacro

// Rechteck aus Kachel tn: w x h Kacheln ab (tx, ty); entrollt, also nur für kleine Flächen; sonst Schleife mit tile_put.
.macro tile_fill(tn, tx, ty, fw, fh)
        .repeat fh, fy
        .repeat fw, fx
        tile_draw tn, tx + fx, ty + fy
        .endrepeat
        .endrepeat
.endmacro

// ---------------------------------------------------------------- Screens
.macro screen_select(sn)
        lda #sn
        jsr screen_select_a
.endmacro

.macro screen_draw(sn=255)
        .if sn != 255
        lda #sn
        jsr screen_select_a
        .endif
        jsr screen_draw_all
.endmacro

// ---------------------------------------------------------------- Laufzeit
// Ein Zeichen nach (tz_sp),y / Farbe nach (tz_cp),y: Offsets über 255 verlangen einen Seitenwechsel des Zeigers.
// Die Zelle k der Kachel liegt bei Offset (k / TW) * COLS + k % TW.
.macro tz_rt_put
tile_put:                                  ; A = Kachel, X = Spalte, Y = Zeile
.cycles tile_put max=(50+22*TILE_TW*TILE_TH)
        sta tz_t
.if TILE_TW == 1
        txa
.elseif TILE_TW == 2
        txa
        asl
.elseif TILE_TW == 4
        txa
        asl
        asl
.elseif TILE_TW == 8
        txa
        asl
        asl
        asl
.else
        lda tile_colofs,x
.endif
        clc
        adc tile_row_lo,y
        sta tz_sp
.ifndef TILE_NOCOLOR
        sta tz_cp
.endif
        lda tile_row_hi,y
        adc #0
        sta tz_sp+1
.ifndef TILE_NOCOLOR
        clc
        adc #>(TILE_COLORRAM - TILE_SCREEN)
        sta tz_cp+1
.endif
        ldx tz_t
        .set tz_pg = 0
        .repeat TILE_TW * TILE_TH, k
        .if ((k / TILE_TW) * TILE_COLS + k % TILE_TW) / 256 != tz_pg
        .set tz_pg = ((k / TILE_TW) * TILE_COLS + k % TILE_TW) / 256
        inc tz_sp+1
        .endif
        ldy #(((k / TILE_TW) * TILE_COLS + k % TILE_TW) & 255)
        lda TILE_C0 + k * TILE_COUNT,x
        sta (tz_sp),y
        .endrepeat
.ifndef TILE_NOCOLOR
        lda TILE_COL,x
        .set tz_pg = 0
        .repeat TILE_TW * TILE_TH, k
        .if ((k / TILE_TW) * TILE_COLS + k % TILE_TW) / 256 != tz_pg
        .set tz_pg = ((k / TILE_TW) * TILE_COLS + k % TILE_TW) / 256
        inc tz_cp+1
        .endif
        ldy #(((k / TILE_TW) * TILE_COLS + k % TILE_TW) & 255)
        sta (tz_cp),y
        .endrepeat
.endif
        rts
.endcycles
.endmacro

// Karte der Kachelzeile Y (Zeiger tz_mp = Anfang der Zeile); verändert A, X; Y bleibt
.macro tz_rt_setmp
tz_setmp:
.ifdef TILE_MAPRAM
        clc
        lda tile_maprow_lo,y
        adc #<TILE_MAPRAM
        sta tz_mp
        lda tile_maprow_hi,y
        adc #>TILE_MAPRAM
        sta tz_mp+1
.else
        ldx tz_idx
        clc
        lda tile_maprow_lo,y
        adc TILE_LO,x
        sta tz_mp
        lda tile_maprow_hi,y
        adc TILE_HI,x
        sta tz_mp+1
.endif
        rts
.endmacro

.macro tz_rt_select
screen_select_a:                           ; A = Screen-Nummer
        sta tz_idx
.ifdef TILE_MAPRAM
        tax
        lda TILE_LO,x
        sta tz_sp
        lda TILE_HI,x
        sta tz_sp+1
        lda #<TILE_MAPRAM
        sta tz_cp
        lda #>TILE_MAPRAM
        sta tz_cp+1
        .if (TILE_MW * TILE_MH) >= 256
        ldx #(TILE_MW * TILE_MH) / 256
        ldy #0
.pgl:  lda (tz_sp),y
        sta (tz_cp),y
        iny
        bne .pgl
        inc tz_sp+1
        inc tz_cp+1
        dex
        bne .pgl
        .endif
        .if ((TILE_MW * TILE_MH) & 255) != 0
        ldy #0
.rst:  lda (tz_sp),y
        sta (tz_cp),y
        iny
        cpy #((TILE_MW * TILE_MH) & 255)
        bne .rst
        .endif
.endif
        rts
.endmacro

.macro tz_rt_map
screen_map_get:                            ; X = Spalte, Y = Zeile -> A = Kachelnummer
        stx tz_t
        jsr tz_setmp
        ldy tz_t
        lda (tz_mp),y
        rts

tile_prop:                                 ; X = Spalte, Y = Zeile -> A = Eigenschaftenbyte
        jsr screen_map_get
        tax
        lda TILE_PROPS,x
        rts
.endmacro

// Zeigerversion: alle Zellen einer Kachel mit einem Zeiger, Zellspalte in tz_t
.macro tz_rt_draw_ptr
screen_draw_all:
        lda #0
        ldx #TILE_MH
screen_draw_rows:                          ; A = erste Kachelzeile, X = Anzahl
        tay
        txa
        pha                                ; Stapel: Anzahl
        tya
        pha                                ; Stapel: Zeile (oben)
        jsr tz_setmp                       ; Y = Zeile
.row:   tsx
        ldy $0101,x                        ; Y = aktuelle Kachelzeile
        lda tile_row_lo,y
        sta tz_sp
.ifndef TILE_NOCOLOR
        sta tz_cp
.endif
        lda tile_row_hi,y
        sta tz_sp+1
.ifndef TILE_NOCOLOR
        clc
        adc #>(TILE_COLORRAM - TILE_SCREEN)
        sta tz_cp+1
.endif
        lda #0
        sta tz_t
.col:
.cycles screen_draw_tile max=(45+24*TILE_TW*TILE_TH+8*(TILE_TH-1))
        ldy #0
        lda (tz_mp),y
        tax
        inc tz_mp
        bne .nc
        inc tz_mp+1
.nc:    ldy tz_t
        .repeat TILE_TW * TILE_TH, k
        lda TILE_C0 + k * TILE_COUNT,x
        sta (tz_sp),y
.ifndef TILE_NOCOLOR
        lda TILE_COL,x
        sta (tz_cp),y
.endif
        .if k + 1 < TILE_TW * TILE_TH
        .if (k + 1) % TILE_TW != 0
        iny
        .else
        tya
        clc
        adc #TILE_COLS - TILE_TW + 1
        tay
        .endif
        .endif
        .endrepeat
        tya
        clc
        adc #((1 - (TILE_TH - 1) * TILE_COLS) & 255)
        sta tz_t
        cmp #TILE_MW * TILE_TW
.endcycles
        bcs .rowend
        jmp .col
.rowend:
        tsx
        inc $0101,x                        ; nächste Kachelzeile
        dec $0102,x                        ; Anzahl - 1
        beq .done
        jmp .row
.done:  pla
        pla
        rts
.endmacro

// SMC-Version: je Kachelzeile werden die Operanden der Schreibbefehle auf die Zeilenadresse gesetzt,
// die Schleife über die Kacheln braucht dann nur noch Y = Zellspalte und X = Kachelnummer.
// Aufbau des Schleifenkörpers (Abstände werden mit .assert geprüft):
//   +0 ldx tz_t (2)  +2 lda karte,x (3, Operand bei +3)  +5 tax (1)
//   +6+6k: lda c_k,x (3), sta bild_k,y (3, Operand bei +10+6k)
//   +6+6N: lda col,x (3), +9+6N+3j: sta farbe_j,y (3, Operand bei +10+6N+3j)
.macro tz_rt_draw_smc
screen_draw_all:
        lda #0
        ldx #TILE_MH
screen_draw_rows:                          ; A = erste Kachelzeile, X = Anzahl
        stx tz_cp+1                        ; Anzahl der Zeilen
        sta tz_cp                          ; aktuelle Zeile
        tay
        jsr tz_setmp
.row:   ldy tz_cp
        lda tile_row_lo,y
        sta tz_sp
        lda tile_row_hi,y
        sta tz_sp+1
        lda tz_mp                          ; Karte
        sta @body + 3
        lda tz_mp+1
        sta @body + 4
        .repeat TILE_TW * TILE_TH, k
        lda tz_sp
        .if ((k / TILE_TW) * TILE_COLS + k % TILE_TW) != 0
        clc
        adc #<((k / TILE_TW) * TILE_COLS + k % TILE_TW)
        .endif
        sta @body + 10 + 6 * k
        lda tz_sp+1
        .if ((k / TILE_TW) * TILE_COLS + k % TILE_TW) != 0
        adc #>((k / TILE_TW) * TILE_COLS + k % TILE_TW)
        .endif
        sta @body + 11 + 6 * k
        .endrepeat
.ifndef TILE_NOCOLOR
        .repeat TILE_TW * TILE_TH, k
        lda tz_sp
        clc
        adc #<((k / TILE_TW) * TILE_COLS + k % TILE_TW + TILE_COLORRAM - TILE_SCREEN)
        sta @body + 10 + 6 * TILE_TW * TILE_TH + 3 * k
        lda tz_sp+1
        adc #>((k / TILE_TW) * TILE_COLS + k % TILE_TW + TILE_COLORRAM - TILE_SCREEN)
        sta @body + 11 + 6 * TILE_TW * TILE_TH + 3 * k
        .endrepeat
.endif
        lda #TILE_MW - 1
        sta tz_t
        ldy #(TILE_MW - 1) * TILE_TW
@body:
.cycles screen_draw_tile max=(25+16*TILE_TW*TILE_TH)
        ldx tz_t
        lda $0200,x
        tax
        .repeat TILE_TW * TILE_TH, k
        lda TILE_C0 + k * TILE_COUNT,x
        sta $0200,y
        .endrepeat
.ifndef TILE_NOCOLOR
        lda TILE_COL,x
        .repeat TILE_TW * TILE_TH, k
        sta $0200,y
        .endrepeat
.endif
        dec tz_t
        .repeat TILE_TW
        dey
        .endrepeat
.endcycles
.ifndef TILE_NOCOLOR
        .assert * - @body == 9 + 9 * TILE_TW * TILE_TH + 2 + TILE_TW, "SMC-Schleifenkörper: Abstände stimmen nicht mehr"
.else
        .assert * - @body == 6 + 6 * TILE_TW * TILE_TH + 2 + TILE_TW, "SMC-Schleifenkörper: Abstände stimmen nicht mehr"
.endif
        bmi .rowdone
        jmp @body
.rowdone:
        clc                                ; Karte: nächste Zeile
        lda tz_mp
        adc #TILE_MW
        sta tz_mp
        bcc .nc
        inc tz_mp+1
.nc:    inc tz_cp
        dec tz_cp+1
        beq .done
        jmp .row
.done:  rts
.endmacro

.macro tiles_runtime
        .assert TILE_TW > 0 && TILE_TH > 0, "tiles_use fehlt vor tiles_runtime"
        .assert TILE_COUNT <= 256, "mehr als 256 Kacheln: Index passt nicht in ein Byte"
        .assert TILE_MW <= 128, "Karte breiter als 128 Kacheln"
.ifndef TILE_NOCOLOR
        .assert (TILE_COLORRAM - TILE_SCREEN) & 255 == 0, "Bildschirm- und Farbspeicher müssen dasselbe Low-Byte haben"
.endif
        .if TILE_SMC == 0
        .assert (TILE_MW - 1) * TILE_TW + (TILE_TH - 1) * TILE_COLS + TILE_TW - 1 <= 255, "Zeigerversion von screen_draw: Kachel zu hoch (Offset über 255), TILE_SMC=1 benutzen"
        .endif
        tz_rt_put
        tz_rt_setmp
        tz_rt_select
        tz_rt_map
        .if TILE_SMC
        tz_rt_draw_smc
        .else
        tz_rt_draw_ptr
        .endif
        // Tabellen: Adresse der ersten Zelle jeder Kachelzeile, Kartenzeilen, Spaltenversatz
tile_row_lo:
        .repeat TILE_ROWS / TILE_TH, i
        .byte <(TILE_SCREEN + TILE_COLS * TILE_TH * i)
        .endrepeat
tile_row_hi:
        .repeat TILE_ROWS / TILE_TH, i
        .byte >(TILE_SCREEN + TILE_COLS * TILE_TH * i)
        .endrepeat
tile_maprow_lo:
        .repeat TILE_MH, i
        .byte <(TILE_MW * i)
        .endrepeat
tile_maprow_hi:
        .repeat TILE_MH, i
        .byte >(TILE_MW * i)
        .endrepeat
        .if TILE_TW != 1 && TILE_TW != 2 && TILE_TW != 4 && TILE_TW != 8
tile_colofs:
        .repeat TILE_COLS / TILE_TW + 1, i
        .byte i * TILE_TW
        .endrepeat
        .endif
.endmacro
