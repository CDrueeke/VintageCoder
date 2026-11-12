; Erwartete CRC-16 (XMODEM) über Bildschirm (1000 Byte) und Farb-RAM (1000 Byte, Maske $0f) je Szenario; erzeugt von tools/gen_tiles_demo.py
.const EXP_S1_SCR = $596a
.const EXP_S1_COL = $1664
.const EXP_ROWS_SCR = $7e4e
.const EXP_ROWS_COL = $5aa0
.const EXP_MACRO_SCR = $d3a3
.const EXP_MACRO_COL = $2a20
.const EXP_PUT_SCR = $4586
.const EXP_PUT_COL = $48c6
; MAP_TILES_W = Kartenbreite in Kacheln; DEMO_FX/FY = Kachelposition des 2 x 2-Blocks von tile_fill (muss zur Referenz passen)
.const MAP_TILES_W = 12
.const DEMO_FX = 6
.const DEMO_FY = 6
