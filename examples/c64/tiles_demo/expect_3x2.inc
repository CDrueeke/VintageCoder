; expect_3x2.inc – Sollwerte für das Beispiel tiles_demo (Kachelgröße 3 x 2). Erzeugt von tools/gen_tiles_demo.py: nicht von Hand ändern.
; Erwartete CRC-16 (XMODEM) über Bildschirm (1000 Byte) und Farb-RAM (1000 Byte, Bits 0-3) je Szenario; das Skript berechnet sie
; unabhängig vom Assembler-Code der Bibliothek, indem es die Screens selbst in ein Abbild zeichnet. demo.inc rechnet dieselbe CRC über
; den echten Bildschirm und vergleicht (Makro `check`).
;   EXP_S1_*     Screen 1 (lv2_id) ganz gezeichnet                 EXP_MACRO_*  tile_draw/tile_draw_color/tile_fill (t_macros)
;   EXP_ROWS_*   nur die Kachelzeilen 3..5 von Screen 2 (t_rows)   EXP_PUT_*    dreimal tile_put (t_puts)
;   _SCR = Sollwert für den Bildschirm, _COL = für das Farb-RAM.   MAP_TILES_W  Kartenbreite in Kacheln (im Beispiel nicht benutzt)
.const EXP_S1_SCR = $596a
.const EXP_S1_COL = $1664
.const EXP_ROWS_SCR = $7e4e
.const EXP_ROWS_COL = $5aa0
.const EXP_MACRO_SCR = $d3a3
.const EXP_MACRO_COL = $2a20
.const EXP_PUT_SCR = $4586
.const EXP_PUT_COL = $48c6
.const MAP_TILES_W = 12
