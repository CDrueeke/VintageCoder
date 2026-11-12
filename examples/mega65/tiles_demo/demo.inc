; =====================================================================================================================
; demo.inc - Beispiel tiles_demo für den MEGA65 (VIC-II-Textmodus des VIC-IV): gemeinsamer Teil der vier Programme
; =====================================================================================================================
; Zweck
;   Zeigt die Kachel-Bibliothek (#include <mega65/tiles.asm>, systemneutraler Teil data/sys/common/tiles.asm): Eine Kachel ist 3 x 2 oder
;   2 x 2 Zeichenzellen; ein Kachelsatz (hg, 6 Kacheln) und eine Screen-Bank (levels, 3 Karten) kommen im Exportformat des Kachel-Editors
;   aus data_3x2.inc / data_2x2.inc (frei erfundene Muster, kein echter Bildinhalt).  Zugleich enthält die Datei die Prüfroutinen für
;   `vcc test` (tiles_demo_*.vctest), die Bildschirm und Farb-RAM per CRC-16 mit den Referenzwerten aus expect_*.inc vergleichen.
;
; Die vier Programme (tiles_demo_<größe>_<variante>.asm) legen vor dem Einbinden dieser Datei nur Schalter fest:
;   DEMO_SET = 3 / 2     Kachelgröße 3x2 (Karte 12 x 10 Kacheln) bzw. 2x2 (Karte 15 x 10 Kacheln)
;   TILE_SMC = 0 / 1     0 = Zeigerversion (ROM-tauglich), 1 = selbständernder Code (schneller, nur im RAM)
;   TILE_MAPRAM          nur 3x2_smc: die Karte wird beim Wählen eines Screens in den RAM-Puffer $3000 kopiert
;
; Was das Programm auf dem Bildschirm tut (Ablauf von `main`)
;   Rahmen und Hintergrund werden schwarz, der Bildschirm gelöscht, dann wird der Screen 0 der Bank (lv1_id) komplett gezeichnet; darüber
;   kommen eine einzelne Kachel (tile_draw), eine Kachel in anderer Farbe (tile_draw_color), ein 2 x 2-Block (tile_fill) und eine Kachel,
;   die zur Laufzeit gesetzt wird (tile_put).  Danach hält das Programm in einer Endlosschleife (`jmp *`).
;
; Hardware / Register (alles VIC-II-kompatibel; der VIC-IV wird nur freigeschaltet und in den VIC-II-Zustand gebracht)
;   Bildschirm bei TILE_SCREEN = $0800 (SCREEN_RAM_C65, 40 x 25 Zellen, 1 Byte je Zelle), Farb-RAM $D800; $D016, $D018, $D020, $D021 wie am C64;
;   $D030, $D031, $D054, $D058, $D059, $D05E sind VIC-IV-Register (C65-ROM-Fenster, H640, 16-Bit-Zeichen, Zeilenabstand, Zeichen je Zeile).
;
; Speicherplan
;   $2001  BASIC-Starter (SYS) und `jmp main`, dahinter Daten, Bibliothek, Prüfroutinen; das Programm muss vor $7000 enden (.assert unten)
;   $0800-$0BE7 Bildschirm, $D800-$DBE7 Farb-RAM, $3000 Kartenpuffer (nur 3x2_smc)
;   Zeropage: $F7-$FE Kachelbibliothek (TILE_ZP); $22-$29 die Prüfroutinen (ZP_CRC ... ZP_CNT); $02-$21 bleiben frei
;   (vctest-Prüfung "zeropage forbid $02-$21").
;
; Bauen und Prüfen: `vcc build` im Projektordner baut alle vier Programme (build/tiles_demo_*.prg), `vcc test` führt die statischen Prüfungen
;   (Überlappung, Zeropage, Zyklenbudget von tile_put) und die Aufrufe der t_*-Routinen aus tiles_demo_*.vctest aus.
; Erzeugt von tools/gen_tiles_demo.py (nicht von Hand ändern; Änderungen an den Kommentaren dort nachziehen).
; =====================================================================================================================
.target mega65
.cpu 45gs02 { illegals = deny }
#include <mega65/tiles.asm>
#include <mega65/gs4502.asm>   ; Makro mega65_enable_vic4

; ZP für die Prüfroutinen (außerhalb der Kachelgruppe $F7-$FE)
.const ZP_CRC = $22             ; CRC-16 (2 Byte)
.const ZP_PTR = $24             ; Zeiger auf die zu prüfenden Daten (2 Byte)
.const ZP_X   = $26             ; Zwischenspeicher (Maske in crc_area; Spalte für t_map_get/t_prop)
.const ZP_Y   = $27             ; Zwischenspeicher (Zeile für t_map_get/t_prop)
.const ZP_CNT = $28             ; Zähler (2 Byte)
.const AREA   = TILE_COLS * TILE_ROWS   ; Zellen des Bildschirms = Länge der CRC-Prüfung (40 x 25 = 1000)

.org $2001                     ; Anfang des BASIC-65-Speichers
.basicstub start                ; "10 SYS <start>", damit RUN das Programm startet
start:  jmp main                ; Einsprung nach dem BASIC-Starter

; ---------------------------------------------------------------- Daten und Bibliothek
.if DEMO_SET == 3
.include "data_3x2.inc"         ; Kachelsatz hg (3x2), Screen-Bank levels, Konstanten lv1_id ...
.include "expect_3x2.inc"       ; Referenz-CRCs EXP_*, Kartenbreite MAP_TILES_W, Blockposition DEMO_FX/FY
.else
.include "data_2x2.inc"
.include "expect_2x2.inc"
.endif

        ; Kachelsatz und Screen-Bank der Bibliothek bekanntmachen (Erklärung im Kopf von common/tiles.asm)
        tiles_use hg_c0, hg_col, hg_props, hg_tw, hg_th, hg_count, levels_screens_lo, levels_screens_hi, levels_tiles_w, levels_tiles_h
.const FX = DEMO_FX            ; Kachelposition (Spalte, Zeile) des 2 x 2-Blocks von tile_fill
.const FY = DEMO_FY

; ---------------------------------------------------------------- Programm: zeigt Screen 0, dann ein paar Einzelkacheln
main:
        sei                         ; keine Unterbrechungen (nur dieses Programm läuft)
        mega65_enable_vic4          ; VIC-IV freischalten ($47, $53 nach $D02F)
        lda #0
        sta $d030                  ; keine C65-ROM-Fenster
        sta $d031                  ; 40 Spalten (H640 aus)
        sta $d054                  ; keine 16-Bit-Zeichen (CHR16 aus; der Wert 0 löscht auch das Bit VFAST = kein 40-MHz-Takt)
        lda #40
        sta $d058                  ; Zeilenabstand 40 Byte
        lda #0
        sta $d059
        lda #40
        sta $d05e                  ; 40 Zeichen je Zeile
        lda #((TILE_SCREEN >> 6) & $f0) | 4
        sta $d018                  ; Bits 7-4 = Bildschirm (TILE_SCREEN / 1 KB), Bits 3-1 = 2: Zeichensatz-Abbild bei $1000 (Zeichen-ROM des VIC)
        lda #$08
        sta $d016                  ; Bit 3 = 40 Spalten, Bit 4 = 0: kein Mehrfarbtext
        lda #0
        sta VIC_BORDERCOLOR
        sta VIC_BGCOLOR0
        jsr clear_all
        screen_draw lv1_id          ; Screen 0 wählen und zeichnen
        tile_draw 3, 5, 2           ; Kachel 3 an Kachelposition (5, 2), nur Konstanten
        tile_draw_color 1, 0, 0, 7
        tile_fill 4, FX, FY, 2, 2   ; 2 x 2 Kacheln der Kachel 4
        lda #1
        ldx #4
        ldy #6
        jsr tile_put                ; Laufzeit: A = Kachel, X = Spalte, Y = Zeile
        jmp *                       ; Endlosschleife: das Bild bleibt stehen

; ---------------------------------------------------------------- Prüfroutinen (Aufruf durch vcc test, siehe tiles_*.vctest)
; clear_all: Bildschirm und Farb-RAM löschen (Zeichen = $20 Leerzeichen, Farbe = 1 weiß).  Eingabe: keine.  Verändert A, X, Flags.
;   Vier Durchläufe über je 256 Byte (Offset $000, $100, $200, $2E8); die letzte Seite überlappt die vorige, damit genau 1000 Zellen gelöscht werden.
clear_all:
        ldx #0
        lda #$20
.l:
        sta TILE_SCREEN + $0,x
        sta TILE_SCREEN + $100,x
        sta TILE_SCREEN + $200,x
        sta TILE_SCREEN + $2e8,x
        lda #$01
        sta TILE_COLORRAM + $0,x
        sta TILE_COLORRAM + $100,x
        sta TILE_COLORRAM + $200,x
        sta TILE_COLORRAM + $2e8,x
        lda #$20
        inx
        bne .l
        rts

; crc_area: CRC-16 (XMODEM: Polynom $1021, Startwert 0) über AREA Byte ab (ZP_PTR); X = Maske, mit der jedes Byte vorher UND-verknüpft wird
;   ($FF beim Bildschirm, $0F beim Farb-RAM, dessen obere Halbbytes nicht definiert sind) -> Ergebnis in ZP_CRC.  Verändert A, X, Y, ZP_PTR+1
;   (wird weitergezählt), ZP_CNT, ZP_X.
crc_area:
        stx ZP_X
        lda #0
        sta ZP_CRC
        sta ZP_CRC+1
        lda #<AREA
        sta ZP_CNT
        lda #>AREA
        sta ZP_CNT+1
        ldy #0
.byte:  lda (ZP_PTR),y
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
        bne .pg
        inc ZP_PTR+1
.pg:    lda ZP_CNT
        bne .d1
        dec ZP_CNT+1
.d1:    dec ZP_CNT
        lda ZP_CNT
        ora ZP_CNT+1
        bne .byte
        rts

; check(exp_scr, exp_col): A = 0, wenn Bildschirm und Farb-RAM die erwartete CRC haben; 1 = Bildschirm falsch, 2 = Farbe falsch
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

t_screen1:                          ; Screen 1 (lv2_id) löschen, zeichnen und prüfen: A = 0 richtig, 1 Bildschirm falsch, 2 Farbe falsch
        jsr clear_all
        screen_draw lv2_id
        check EXP_S1_SCR, EXP_S1_COL

t_screen1_nocheck:                  ; wie t_screen1, aber ohne CRC (der vctest prüft Stichproben im Speicher)
        jsr clear_all
        screen_draw lv2_id
        rts

t_draw_only:                        ; nur das Zeichnen von Screen 1, ohne Löschen und Prüfung (für die Zyklenmessung)
        screen_draw lv2_id
        rts

t_rows:                             ; Kachelzeilen 3..5 von Screen 2 (lv3_id; Screens zählen ab 0) mit screen_draw_rows zeichnen und prüfen (A wie t_screen1)
        jsr clear_all
        screen_select lv3_id
        lda #3
        ldx #3
        jsr screen_draw_rows
        check EXP_ROWS_SCR, EXP_ROWS_COL

t_macros:                           ; Makros tile_draw, tile_draw_color (Farbe 14) und tile_fill zeichnen und prüfen (A wie t_screen1)
        jsr clear_all
        tile_draw 3, 5, 2
        tile_draw_color 1, 0, 0, 14
        tile_fill 4, FX, FY, 2, 2
        check EXP_MACRO_SCR, EXP_MACRO_COL

t_puts:                             ; tile_put dreimal (Kachel 5 nach (2,3), Kachel 0 in die letzte Kachelzelle, Kachel 3 nach (0,0)); prüfen (A wie t_screen1)
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

t_map_get:                          ; X = Spalte, Y = Zeile (in Kacheln) von Screen 1 (lv2_id) -> A = Kachelnummer (screen_map_get)
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp screen_map_get

t_prop:                             ; X = Spalte, Y = Zeile von Screen 1 (lv2_id) -> A = Eigenschaftenbyte der Kachel dort (tile_prop)
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp tile_prop

; ---------------------------------------------------------------- Laufzeit der Bibliothek (einmal, hinter dem Programm)
        tiles_runtime               ; die Unterprogramme der Bibliothek (tile_put, screen_draw_rows ...) einmal hier ablegen
        .assert * <= $7000, "Programm überschreitet die Grenze des freien Speichers (Bildschirm, Display-Liste oder Karte)"
