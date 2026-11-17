; Gemeinsamer Teil des Beispiels tiles_demo (Atari 8 Bit (ANTIC-Modus 2)): zeigt die Kachel-Bibliothek (#include <atari8bit/tiles.asm>) und enthält die
; Prüfroutinen für vcc test. Die vier Programme tiles_demo_*.asm unterscheiden sich nur in Kachelgröße (DEMO_SET = 3 für
; 3x2, 2 für 2x2), TILE_SMC und TILE_MAPRAM; sie legen diese Schalter fest und binden dann diese Datei ein.
; Erzeugt von tools/gen_tiles_demo.py (nicht von Hand ändern).
.target atari8
.cpu sally { illegals = deny }
#include <atari8bit/tiles.asm>
#include <atari8bit/test.inc>

; ZP für die Prüfroutinen (außerhalb der Kachelgruppe $F7-$FE)
.const ZP_CRC = $80             ; CRC-16 (2 Byte)
.const ZP_PTR = $82             ; Zeiger auf die zu prüfenden Daten (2 Byte)
.const ZP_X   = $84
.const ZP_Y   = $85
.const ZP_CNT = $86             ; Zähler (2 Byte)
.const AREA   = TILE_COLS * TILE_ROWS

.org $2000
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
        test_begin                  ; Postfach-Modus: das Programm prüft sich selbst (siehe unten), zeigt dann die Demo
        jsr selftest
        jsr setup_display
        jsr clear_all
        screen_draw lv1_id          ; Screen 0 wählen und zeichnen
        tile_draw 3, 5, 2           ; Kachel 3 an Kachelposition (5, 2), nur Konstanten
        tile_draw_color 1, 0, 0, 0
        tile_fill 4, FX, FY, 2, 2   ; 2 x 2 Kacheln der Kachel 4
        lda #1
        ldx #4
        ldy #6
        jsr tile_put                ; Laufzeit: A = Kachel, X = Spalte, Y = Zeile
        test_end

; Display-Liste: 3 x 8 leere Zeilen, 24 Textzeilen im ANTIC-Modus 2 (40 Spalten) ab TILE_SCREEN (LMS), Zeichensatz im ROM ($E000).
; Modus 4 (Mehrfarbtext) unterscheidet sich nur im Befehlsbyte ($44/$04 statt $42/$02) und braucht einen eigenen Zeichensatz.
SDLSTL = $0230
CHBAS  = $02f4
setup_display:
        lda #<dlist
        sta SDLSTL
        lda #>dlist
        sta SDLSTL+1
        lda #$e0
        sta CHBAS
        rts

selftest:                        ; alle Prüfungen nacheinander; jede liefert A = 0, wenn die Bildschirm-CRC stimmt
        jsr t_screen1
        assert_a_eq 0
        jsr t_rows
        assert_a_eq 0
        jsr t_macros
        assert_a_eq 0
        jsr t_puts
        assert_a_eq 0
        jsr t_erase
        assert_a_eq 0
        jsr t_restore
        assert_a_eq 0
        jsr t_restore_bg
        assert_a_eq 0
        jsr t_row
        assert_a_eq 0
        ldx #SELF_X0
        ldy #SELF_Y0
        jsr t_map_get
        assert_a_eq SELF_T0
        ldx #SELF_X0
        ldy #SELF_Y0
        jsr t_prop
        assert_a_eq SELF_P0
        ldx #SELF_X1
        ldy #SELF_Y1
        jsr t_map_get
        assert_a_eq SELF_T1
        ldx #SELF_X1
        ldy #SELF_Y1
        jsr t_prop
        assert_a_eq SELF_P1
        ldx #SELF_X2
        ldy #SELF_Y2
        jsr t_map_get
        assert_a_eq SELF_T2
        ldx #SELF_X2
        ldy #SELF_Y2
        jsr t_prop
        assert_a_eq SELF_P2
        rts

; ---------------------------------------------------------------- Prüfroutinen (Aufruf durch vcc test, siehe tiles_*.vctest)
; Bildschirm löschen (Zeichen = blank, Farbe = blank_col)
clear_all:
        ldx #0
        lda #$00
.l:
        sta TILE_SCREEN + $0,x
        sta TILE_SCREEN + $100,x
        sta TILE_SCREEN + $200,x
        sta TILE_SCREEN + $2c0,x
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

; check(exp_scr): A = 0, wenn der Bildschirm die erwartete CRC hat; 1 = falsch (kein Farbspeicher)
.macro check(exp_scr)
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
        lda #0
        rts
@bad1:  lda #1
        rts
.endmacro

t_screen1:                          ; ganzer Screen 1
        jsr clear_all
        screen_draw lv2_id
        check EXP_S1_SCR

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
        check EXP_ROWS_SCR

t_macros:
        jsr clear_all
        tile_draw 3, 5, 2
        tile_draw_color 1, 0, 0, 14
        tile_fill 4, FX, FY, 2, 2
        check EXP_MACRO_SCR

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
        check EXP_PUT_SCR

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
        check EXP_ERASE_SCR

; t_erase_at - tile_erase einzeln für den Test: ZP_X, ZP_Y = Kachelposition (Screen 1 ist schon gewählt, siehe setup mem).
t_erase_at:
        jsr tile_erase
        rts

; t_restore - tile_restore (Rechteck 4 x 3 ab (2,1) aus der Karte von Screen 1); Ausgabe wie t_screen1.
t_restore:
        jsr clear_all
        screen_select lv2_id
        tile_restore 2, 1, 4, 3
        check EXP_RESTORE_SCR

; t_restore_bg - Hintergrundwiederherstellung: Screen 1 zeichnen, eine Figur (Kachel 4, 3 x 2) darüber setzen, dann das Rechteck neu
;   zeichnen; das Ergebnis muss wieder Screen 1 sein.
t_restore_bg:
        jsr clear_all
        screen_draw lv2_id
        tile_fill 4, 2, 2, 3, 2
        tile_restore 2, 2, 3, 2
        check EXP_S1_SCR

; t_row - tile_row: Kachelzeile 5 mit Kachel 3 füllen; Ausgabe wie t_screen1.
t_row:
        jsr clear_all
        tile_row 5, 3
        check EXP_ROW_SCR


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
        .assert * <= $3800, "Programm überschreitet die Grenze des freien Speichers (Bildschirm, Display-Liste oder Karte)"

        .org $3800                  ; Display-Liste (darf keine 1-KB-Grenze überschreiten)
dlist:  .byte $70,$70,$70
        .byte $42,<TILE_SCREEN,>TILE_SCREEN
        .repeat TILE_ROWS - 1
        .byte $02
        .endrepeat
        .byte $41,<dlist,>dlist
        .run start
