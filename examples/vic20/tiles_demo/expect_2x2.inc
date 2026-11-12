; Erwartete CRC-16 (XMODEM) über Bildschirm (506 Byte) und Farb-RAM (506 Byte, Maske $0f) je Szenario; erzeugt von tools/gen_tiles_demo.py
; Szenarien (Prüfroutinen in demo.inc): EXP_S1 = t_screen1 (Karte lv2_id ganz), EXP_ROWS = t_rows (Kachelzeilen 3..5 der Karte lv3_id), EXP_MACRO = t_macros,
; EXP_PUT = t_puts; _SCR = Zeichen auf dem Bildschirm, _COL = Farb-RAM. MAP_TILES_W = Kartenbreite in Kacheln, DEMO_FX/DEMO_FY = Kachelposition des 2 x 2-Blocks in main.
.const EXP_S1_SCR = $ba0f
.const EXP_S1_COL = $d3a4
.const EXP_ROWS_SCR = $1959
.const EXP_ROWS_COL = $cc4d
.const EXP_MACRO_SCR = $176f
.const EXP_MACRO_COL = $212e
.const EXP_PUT_SCR = $08dc
.const EXP_PUT_COL = $c810
.const MAP_TILES_W = 11
.const DEMO_FX = 6
.const DEMO_FY = 6
