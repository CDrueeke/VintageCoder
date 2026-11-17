; Gemeinsamer Teil des Beispiels tiles_demo (Plus/4): zeigt die Kachel-Bibliothek (#include <plus4/tiles.asm>) und enthält die
; Prüfroutinen für vcc test. Die vier Programme tiles_demo_*.asm unterscheiden sich nur in Kachelgröße (DEMO_SET = 3 für
; 3x2, 2 für 2x2), TILE_SMC und TILE_MAPRAM; sie legen diese Schalter fest und binden dann diese Datei ein.
; Erzeugt von tools/gen_tiles_demo.py (nicht von Hand ändern).
.target plus4
.cpu 7501 { illegals = deny }
#include <plus4/tiles.asm>

; ZP für die Prüfroutinen (außerhalb der Kachelgruppe $F7-$FE)
.const ZP_CRC = $22             ; CRC-16 (2 Byte)
.const ZP_PTR = $24             ; Zeiger auf die zu prüfenden Daten (2 Byte)
.const ZP_X   = $26
.const ZP_Y   = $27
.const ZP_CNT = $28             ; Zähler (2 Byte)
.const AREA   = TILE_COLS * TILE_ROWS

.org $1001
.basicstub start
start:  jmp main

; ---------------------------------------------------------------- Daten und Bibliothek
.if DEMO_SET == 3
.include "data_3x2.inc"
.include "expect_3x2.inc"
.else
.include "data_2x2.inc"
.include "expect_2x2.inc"
.endif

.ifndef DEMO_RLE
.const DEMO_RLE = 0                ; 1 = Karten lauflängenkodiert (levels_rle_*), TILE_RLE und TILE_MAPRAM kommen aus der Programmdatei
.endif
.if DEMO_RLE
        tiles_use hg_c0, hg_col, hg_props, hg_tw, hg_th, hg_count, levels_rle_lo, levels_rle_hi, levels_tiles_w, levels_tiles_h
.else
        tiles_use hg_c0, hg_col, hg_props, hg_tw, hg_th, hg_count, levels_screens_lo, levels_screens_hi, levels_tiles_w, levels_tiles_h
.endif
.const FX = DEMO_FX
.const FY = DEMO_FY

; ---------------------------------------------------------------- Programm: zeigt Screen 0, dann ein paar Einzelkacheln
main:
        sei
        lda #0
        sta $ff19                  ; Rahmen schwarz
        sta $ff15                  ; Hintergrund schwarz
        jsr clear_all
        screen_draw lv1_id          ; Screen 0 wählen und zeichnen
        tile_draw 3, 5, 2           ; Kachel 3 an Kachelposition (5, 2), nur Konstanten
        tile_draw_color 1, 0, 0, 119
        tile_fill 4, FX, FY, 2, 2   ; 2 x 2 Kacheln der Kachel 4
        lda #1
        ldx #4
        ldy #6
        jsr tile_put                ; Laufzeit: A = Kachel, X = Spalte, Y = Zeile
        jmp *

; ---------------------------------------------------------------- Prüfroutinen (Aufruf durch vcc test, siehe tiles_*.vctest)
; Bildschirm löschen (Zeichen = blank, Farbe = blank_col)
clear_all:
        ldx #0
        lda #$20
.l:
        sta TILE_SCREEN + $0,x
        sta TILE_SCREEN + $100,x
        sta TILE_SCREEN + $200,x
        sta TILE_SCREEN + $2e8,x
        lda #$71
        sta TILE_COLORRAM + $0,x
        sta TILE_COLORRAM + $100,x
        sta TILE_COLORRAM + $200,x
        sta TILE_COLORRAM + $2e8,x
        lda #$20
        inx
        bne .l
        rts

; crc_area: CRC-16 (XMODEM) über AREA Byte ab (ZP_PTR); X = Maske für jedes Byte -> Ergebnis in ZP_CRC
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
        ldx #$ff
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

t_screen1:                          ; ganzer Screen 1
        jsr clear_all
        screen_draw lv2_id
        check EXP_S1_SCR, EXP_S1_COL

t_screen1_nocheck:
        jsr clear_all
        screen_draw lv2_id
        rts

t_draw_only:                        ; nur das Zeichnen (für die Zyklenmessung)
        screen_draw lv2_id
        rts

t_rows:                             ; Kachelzeilen 3..5 von Screen 3
        jsr clear_all
        screen_select lv3_id
        lda #3
        ldx #3
        jsr screen_draw_rows
        check EXP_ROWS_SCR, EXP_ROWS_COL

t_macros:
        jsr clear_all
        tile_draw 3, 5, 2
        tile_draw_color 1, 0, 0, 14
        tile_fill 4, FX, FY, 2, 2
        check EXP_MACRO_SCR, EXP_MACRO_COL

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

; t_erase - tile_erase: Screen 1 (lv2_id) wählen, nur die Karten-Kacheln bei (3,4) und rechts unten zeichnen; Ausgabe wie t_screen1.
t_erase:
        jsr clear_all
        screen_select lv2_id
        ldx #3
        ldy #4
        jsr tile_erase
        ldx #levels_tiles_w - 1
        ldy #levels_tiles_h - 1
        jsr tile_erase
        check EXP_ERASE_SCR, EXP_ERASE_COL

; t_erase_at - tile_erase einzeln für den Test: ZP_X, ZP_Y = Kachelposition (Screen 1 ist schon gewählt, siehe setup mem).
t_erase_at:
        jsr tile_erase
        rts

; t_restore - tile_restore (Rechteck 4 x 3 ab (2,1) aus der Karte von Screen 1); Ausgabe wie t_screen1.
t_restore:
        jsr clear_all
        screen_select lv2_id
        tile_restore 2, 1, 4, 3
        check EXP_RESTORE_SCR, EXP_RESTORE_COL

; t_restore_bg - Hintergrundwiederherstellung: Screen 1 zeichnen, eine Figur (Kachel 4, 3 x 2) darüber setzen, dann das Rechteck neu
;   zeichnen; das Ergebnis muss wieder Screen 1 sein.
t_restore_bg:
        jsr clear_all
        screen_draw lv2_id
        tile_fill 4, 2, 2, 3, 2
        tile_restore 2, 2, 3, 2
        check EXP_S1_SCR, EXP_S1_COL

; t_row - tile_row: Kachelzeile 5 mit Kachel 3 füllen; Ausgabe wie t_screen1.
t_row:
        jsr clear_all
        tile_row 5, 3
        check EXP_ROW_SCR, EXP_ROW_COL


t_map_get:                          ; X = Spalte, Y = Zeile von Screen 2 (lv2_id) -> A = Kachelnummer
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp screen_map_get

t_prop:
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp tile_prop

; ---------------------------------------------------------------- Laufzeit der Bibliothek (einmal, hinter dem Programm)
        tiles_runtime
        .assert * <= $4000, "Programm überschreitet die Grenze des freien Speichers (Bildschirm, Display-Liste oder Karte)"
