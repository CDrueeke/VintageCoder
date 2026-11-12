; Erwartete CRC-16 (XMODEM) über Bildschirm (1000 Byte) und Farb-RAM (1000 Byte, Maske $0f) je Szenario; erzeugt von tools/gen_tiles_demo.py
; Szenarien (siehe t_* in demo.inc): S1 = ganzer Screen 1, ROWS = Kachelzeilen 3..5 von Screen 3, MACRO = tile_draw/tile_draw_color/tile_fill,
; PUT = drei tile_put-Aufrufe.  _SCR = CRC des Bildschirms, _COL = CRC des Farb-RAMs.  Die Werte rechnet das Python-Skript unabhängig nach.
.const EXP_S1_SCR = $6519
.const EXP_S1_COL = $e6da
.const EXP_ROWS_SCR = $804f
.const EXP_ROWS_COL = $3921
.const EXP_MACRO_SCR = $3e13
.const EXP_MACRO_COL = $850b
.const EXP_PUT_SCR = $d855
.const EXP_PUT_COL = $7e03
.const MAP_TILES_W = 15                 ; Kartenbreite in Kacheln (wie levels_tiles_w)
.const DEMO_FX = 6                      ; Spalte des tile_fill-Rechtecks (Kachelposition)
.const DEMO_FY = 6                      ; Zeile des tile_fill-Rechtecks (Kachelposition)
