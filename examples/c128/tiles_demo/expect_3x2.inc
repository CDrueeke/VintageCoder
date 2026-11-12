; Erwartete CRC-16 (XMODEM) über Bildschirm (1000 Byte) und Farb-RAM (1000 Byte, Maske $0f) je Szenario; erzeugt von tools/gen_tiles_demo.py
; Szenarien (siehe t_* in demo.inc): S1 = ganzer Screen 1, ROWS = Kachelzeilen 3..5 von Screen 3, MACRO = tile_draw/tile_draw_color/tile_fill,
; PUT = drei tile_put-Aufrufe.  _SCR = CRC des Bildschirms, _COL = CRC des Farb-RAMs.  Die Werte rechnet das Python-Skript unabhängig nach.
.const EXP_S1_SCR = $596a
.const EXP_S1_COL = $1664
.const EXP_ROWS_SCR = $7e4e
.const EXP_ROWS_COL = $5aa0
.const EXP_MACRO_SCR = $d3a3
.const EXP_MACRO_COL = $2a20
.const EXP_PUT_SCR = $4586
.const EXP_PUT_COL = $48c6
.const MAP_TILES_W = 12                 ; Kartenbreite in Kacheln (wie levels_tiles_w)
.const DEMO_FX = 6                      ; Spalte des tile_fill-Rechtecks (Kachelposition)
.const DEMO_FY = 6                      ; Zeile des tile_fill-Rechtecks (Kachelposition)
