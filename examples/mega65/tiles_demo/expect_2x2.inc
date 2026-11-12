; Erwartete CRC-16 (XMODEM) über Bildschirm (1000 Byte) und Farb-RAM (1000 Byte, Maske $0f) je Szenario; erzeugt von tools/gen_tiles_demo.py
.const EXP_S1_SCR = $6519
.const EXP_S1_COL = $e6da
.const EXP_ROWS_SCR = $804f
.const EXP_ROWS_COL = $3921
.const EXP_MACRO_SCR = $3e13
.const EXP_MACRO_COL = $850b
.const EXP_PUT_SCR = $d855
.const EXP_PUT_COL = $7e03
; MAP_TILES_W = Kartenbreite in Kacheln; DEMO_FX/FY = Kachelposition des 2 x 2-Blocks von tile_fill (muss zur Referenz passen)
.const MAP_TILES_W = 15
.const DEMO_FX = 6
.const DEMO_FY = 6
