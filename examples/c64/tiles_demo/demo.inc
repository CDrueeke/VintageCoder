; demo.inc – gemeinsamer Teil des Beispiels tiles_demo (C64): zeigt die Kachel-Bibliothek (#include <c64/tiles.asm>) und enthält die
; Prüfroutinen für vcc test.
;
; Aufbau des Beispiels: Die vier Programme tiles_demo_3x2_ptr.asm, tiles_demo_3x2_smc.asm, tiles_demo_2x2_ptr.asm und tiles_demo_2x2_smc.asm
; unterscheiden sich nur in der Kachelgröße (DEMO_SET = 3 für 3x2-Kacheln, 2 für 2x2), in TILE_SMC (0 = Zeigerversion von screen_draw,
; ROM-tauglich; 1 = selbstverändernder Code, schneller) und in TILE_MAPRAM (nur 3x2_smc: $C800, die Karte wird beim Wählen eines Screens
; dorthin kopiert). Sie legen diese Schalter fest und binden dann diese Datei ein. Jedes hat eine eigene Testdatei tiles_demo_<variante>.vctest.
;
; Daten: data_3x2.inc bzw. data_2x2.inc (Kachelsatz hg mit 6 Kacheln und Screen-Bank levels mit 3 Screens, im Exportformat des Kachel-
; Editors) und expect_3x2.inc bzw. expect_2x2.inc (Sollwerte der Prüfsummen). Alle drei Dateien erzeugt tools/gen_tiles_demo.py.
; Screens werden von 0 an gezählt: Screen 0 = lv1_id, Screen 1 = lv2_id, Screen 2 = lv3_id.
;
; Bedienung: `vcc build examples/c64/tiles_demo` baut die vier PRG-Dateien; Start eines PRG: das Hauptprogramm `main` löscht den Bildschirm,
; zeichnet Screen 0 und danach einige Einzelkacheln (tile_draw, tile_draw_color, tile_fill, tile_put). `vcc test examples/c64/tiles_demo`
; führt die statischen Prüfungen aus (Überlappung, Zeropage $02-$21 verboten, Zyklenbudget von tile_put, Programm endet vor $2000);
; mit `--backend vice` laufen zusätzlich die Tests im Emulator: sie rufen die Prüfroutinen t_* (unten) über eine Falle auf, beschreiben
; Bildschirm und Farb-RAM mit der Bibliothek und vergleichen eine CRC-16 über beide Bereiche mit der unabhängig berechneten Referenz
; (EXP_*), dazu Stichproben einzelner Zellen sowie Ergebnisse von screen_map_get, tile_prop und tile_put samt Zyklenmessung.
;
; Speicherplan: $0801 BASIC-Zeile `10 SYS 2061` und `start: jmp main`, dahinter Daten und Bibliotheksbindung (tiles_use erzeugt keinen
; Code), das Hauptprogramm, die Prüfroutinen und ganz hinten die Unterprogramme der Bibliothek (tiles_runtime). Bildschirm $0400,
; Farb-RAM $D800 (Standard der Bibliothek, 40 x 25 Zeichen). Zeropage: Bibliothek $F7-$FE (tz_sp, tz_cp, tz_mp, tz_t und tz_idx = $FE,
; der gewählte Screen), Prüfroutinen $22-$29 (siehe unten). Die Falle der Tests liegt beim C64 standardmäßig bei $C000 (frei).
; Grenzen: Die Prüfroutinen sind kein Beispiel für gutes Spielcode, sondern Messhilfen; es gibt keine Rückkehr ins BASIC (jmp * am Ende
; von main).
.target c64
.cpu 6510 { illegals = deny }
#include <c64/tiles.asm>

; ZP für die Prüfroutinen (außerhalb der Kachelgruppe $F7-$FE); die Testdatei verbietet $02-$21, deshalb liegen sie ab $22
.const ZP_CRC = $22             ; CRC-16 (2 Byte)
.const ZP_PTR = $24             ; Zeiger auf die zu prüfenden Daten (2 Byte)
.const ZP_X   = $26             ; 1 Byte: Maske in crc_area, gemerkte Spalte (X) in t_map_get und t_prop
.const ZP_Y   = $27             ; 1 Byte: gemerkte Zeile (Y) in t_map_get und t_prop
.const ZP_CNT = $28             ; Restzähler der Bytes in crc_area (2 Byte: $28/$29)

.org $0801
.basicstub start
start:  jmp main                ; über Daten und Bibliotheksbindung hinweg zum Hauptprogramm

; ---------------------------------------------------------------- Daten und Bibliothek
.if DEMO_SET == 3
.include "data_3x2.inc"         ; Kachelsatz hg (3x2) und Screen-Bank levels (3 Karten zu 12 x 10 Kacheln)
.include "expect_3x2.inc"       ; Sollwerte EXP_* (CRC-16) und MAP_TILES_W
.else
.include "data_2x2.inc"         ; Kachelsatz hg (2x2) und Screen-Bank levels (3 Karten zu 15 x 10 Kacheln)
.include "expect_2x2.inc"
.endif

; Satz hg und Bank levels an die Bibliothek binden (setzt nur Assemblervariablen, erzeugt keinen Code)

        tiles_use hg_c0, hg_col, hg_props, hg_tw, hg_th, hg_count, levels_screens_lo, levels_screens_hi, levels_tiles_w, levels_tiles_h


; ---------------------------------------------------------------- Programm: zeigt Screen 0 und darauf ein paar Einzelkacheln
; main – Hauptprogramm (Einstieg über `start`). Rahmen und Hintergrund schwarz, Bildschirm löschen, Screen 0 zeichnen und dann einzelne
;   Kacheln mit den Makros und mit tile_put darübersetzen. Endet in einer Endlosschleife; verändert alle Register.
main:   sei
        lda #0
        sta VIC_BORDERCOLOR
        sta VIC_BGCOLOR0
        jsr clear_all
        screen_draw lv1_id          ; Screen 0 wählen und komplett zeichnen
        tile_draw 3, 5, 2           ; Kachel 3 an Kachelposition (5, 2): Zeichen und Kachelfarbe, nur Konstanten (Makro, entrollt)
        tile_draw_color 1, 0, 0, COL_YELLOW   ; Kachel 1 links oben, aber mit der Farbe Gelb statt der Kachelfarbe
        tile_fill 4, 6, 6, 2, 2     ; 2 x 2 Kacheln der Kachel 4
        lda #1
        ldx #4
        ldy #6
        jsr tile_put                ; Laufzeit: A = Kachel, X = Spalte, Y = Zeile
        jmp *

; ---------------------------------------------------------------- Prüfroutinen (Aufruf durch vcc test, siehe tiles_demo_*.vctest)
; clear_all – Bildschirm mit dem Leerzeichen ($20) und Farb-RAM mit 1 (Weiß) füllen, damit jede Prüfung vom gleichen Zustand ausgeht.
;   Eine Schleife über X = 0..255 beschreibt vier überlappende 256-Byte-Fenster (Versätze 0, $100, $200, $2E8 = 744), zusammen 1000 Zellen.
;   Verändert A, X und die Flags.
clear_all:
        ldx #0
.l:     lda #$20
        sta TILE_SCREEN,x
        sta TILE_SCREEN+$100,x
        sta TILE_SCREEN+$200,x
        sta TILE_SCREEN+$2e8,x
        lda #1
        sta TILE_COLORRAM,x
        sta TILE_COLORRAM+$100,x
        sta TILE_COLORRAM+$200,x
        sta TILE_COLORRAM+$2e8,x
        inx
        bne .l
        rts

; crc_area – CRC-16 (XMODEM: Polynom $1021, Startwert 0, MSB zuerst) über 1000 Byte.
;   Eingabe: ZP_PTR = Anfangsadresse; X = Maske, die vor der Rechnung per UND auf jedes Byte angewandt wird ($FF für den Bildschirm,
;   $0F für das Farb-RAM, dessen obere vier Bits nicht belegt sind). Ausgabe: ZP_CRC (niederwertiges Byte zuerst). Verändert A, X, Y,
;   ZP_PTR (zeigt danach hinter den Bereich), ZP_CNT und ZP_X. Schleifenaufbau: Byte laden, Maske, in das obere CRC-Byte einmischen,
;   achtmal schieben und bei herausgeschobenem Bit das Polynom ($1021) einmischen; dann Zeiger und Zähler weiterschalten.
crc_area:
        stx ZP_X
        lda #0
        sta ZP_CRC
        sta ZP_CRC+1
        lda #<1000
        sta ZP_CNT
        lda #>1000
        sta ZP_CNT+1
        ldy #0
.byte:  lda (ZP_PTR),y          ; (lokale Marke, nicht die Direktive .byte)
        and ZP_X
        eor ZP_CRC+1
        sta ZP_CRC+1
        ldx #8
.bit:   asl ZP_CRC
        rol ZP_CRC+1
        bcc .nx
        lda ZP_CRC+1
        eor #$10
        sta ZP_CRC+1
        lda ZP_CRC
        eor #$21
        sta ZP_CRC
.nx:    dex
        bne .bit
        iny
        bne .pg                     ; Y übergelaufen: Zeiger auf die nächste Seite
        inc ZP_PTR+1
.pg:    lda ZP_CNT                  ; 16-Bit-Zähler herunterzählen: Übertrag ins obere Byte, wenn das untere 0 ist
        bne .d1
        dec ZP_CNT+1
.d1:    dec ZP_CNT
        lda ZP_CNT
        ora ZP_CNT+1
        bne .byte
        rts

; check(exp_scr, exp_col) – Makro: vergleicht die CRC-16 des Bildschirms mit exp_scr und die des Farb-RAM (nur Bits 0-3) mit exp_col.
;   Ausgabe: A = 0, wenn beide stimmen; 1 = Bildschirm falsch; 2 = Farbe falsch (Rücksprung per RTS, das Makro beendet also die
;   aufrufende Prüfroutine). Verändert A, X, Y und die Flags.
.macro check(exp_scr, exp_col)
        lda #<TILE_SCREEN
        sta ZP_PTR
        lda #>TILE_SCREEN
        sta ZP_PTR+1
        ldx #$ff
        jsr crc_area
        lda ZP_CRC
        cmp #<exp_scr
        bne @bad1
        lda ZP_CRC+1
        cmp #>exp_scr
        bne @bad1
        lda #<TILE_COLORRAM
        sta ZP_PTR
        lda #>TILE_COLORRAM
        sta ZP_PTR+1
        ldx #$0f
        jsr crc_area
        lda ZP_CRC
        cmp #<exp_col
        bne @bad2
        lda ZP_CRC+1
        cmp #>exp_col
        bne @bad2
        lda #0
        rts
@bad1:  lda #1
        rts
@bad2:  lda #2
        rts
.endmacro

; t_screen1 – Test: Screen 1 (lv2_id) ganz zeichnen und mit der Referenz vergleichen. Ausgabe: A = 0 (in Ordnung) oder 1/2 wie bei check.
t_screen1:                          ; ganzer Screen 1
        jsr clear_all
        screen_draw lv2_id
        check EXP_S1_SCR, EXP_S1_COL

; t_screen1_nocheck – wie t_screen1, aber ohne Vergleich (RTS am Ende): für Stichproben einzelner Zellen durch den Test.
t_screen1_nocheck:
        jsr clear_all
        screen_draw lv2_id
        rts

; t_draw_only – nur screen_draw von Screen 1, ohne vorheriges Löschen und ohne Vergleich: für die Zyklenmessung der ganzen Kachelebene.
t_draw_only:                        ; nur das Zeichnen (für die Zyklenmessung)
        screen_draw lv2_id
        rts

; t_rows – Test von screen_draw_rows: nur die Kachelzeilen 3..5 von Screen 2 (lv3_id) zeichnen und mit der Referenz vergleichen.
t_rows:                             ; Kachelzeilen 3..5 von Screen 2
        jsr clear_all
        screen_select lv3_id
        lda #3
        ldx #3
        jsr screen_draw_rows
        check EXP_ROWS_SCR, EXP_ROWS_COL

; t_macros – Test der Makros tile_draw, tile_draw_color und tile_fill auf leerem Bildschirm; Ausgabe wie t_screen1.
t_macros:
        jsr clear_all
        tile_draw 3, 5, 2
        tile_draw_color 1, 0, 0, 14
        tile_fill 4, 6, 6, 2, 2
        check EXP_MACRO_SCR, EXP_MACRO_COL

; t_puts – Test von tile_put: Kachel 5 nach (2, 3), Kachel 0 in die rechte untere Ecke und Kachel 3 nach (0, 0); Ausgabe wie t_screen1.
t_puts:
        jsr clear_all
        lda #5
        ldx #2
        ldy #3
        jsr tile_put
        lda #0
        ldx #levels_tiles_w - 1
        ldy #levels_tiles_h - 1
        jsr tile_put
        lda #3
        ldx #0
        ldy #0
        jsr tile_put
        check EXP_PUT_SCR, EXP_PUT_COL

; t_map_get – Test von screen_map_get. Eingabe: X = Spalte, Y = Zeile der Karte von Screen 1 (lv2_id). Ausgabe: A = Kachelnummer.
;   Merkt X und Y in ZP_X/ZP_Y, weil screen_select die Register verändert.
t_map_get:
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp screen_map_get

; t_prop – Test von tile_prop. Eingabe: X = Spalte, Y = Zeile der Karte von Screen 1 (lv2_id). Ausgabe: A = Eigenschaftenbyte der Kachel dort.
t_prop:
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp tile_prop

; ---------------------------------------------------------------- Laufzeit der Bibliothek (einmal, hinter dem Programm)
        tiles_runtime                   ; Unterprogramme screen_*, tile_put, tile_prop und die Tabellen der Bibliothek
