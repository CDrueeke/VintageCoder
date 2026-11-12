; Erwartete CRC-16 (XMODEM) über Bildschirm (1000 Byte) und Farb-RAM (1000 Byte, Maske $ff) je Szenario; erzeugt von tools/gen_tiles_demo.py
.const EXP_S1_SCR = $596a   ; Screen-Nummer 1 komplett gezeichnet (t_screen1): Bildschirm
.const EXP_S1_COL = $072f   ; dasselbe: Farb-RAM
.const EXP_ROWS_SCR = $7e4e   ; Kachelzeilen 3..5 von Screen-Nummer 2 (t_rows): Bildschirm
.const EXP_ROWS_COL = $fbe3   ; dasselbe: Farb-RAM
.const EXP_MACRO_SCR = $d3a3   ; Makros tile_draw / tile_draw_color / tile_fill (t_macros): Bildschirm
.const EXP_MACRO_COL = $0c87   ; dasselbe: Farb-RAM
.const EXP_PUT_SCR = $4586   ; tile_put-Aufrufe (t_puts): Bildschirm
.const EXP_PUT_COL = $b0f2   ; dasselbe: Farb-RAM
.const MAP_TILES_W = 12   ; Breite der Karten in Kacheln (wie levels_tiles_w)
.const DEMO_FX = 6   ; Kachelspalte des 2x2-Rechtecks in main und t_macros
.const DEMO_FY = 6   ; Kachelzeile des Rechtecks
