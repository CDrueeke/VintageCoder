; Erwartete CRC-16 (XMODEM) über Bildschirm (960 Byte), ohne Farbspeicher je Szenario; erzeugt von tools/gen_tiles_demo.py
.const EXP_S1_SCR = $d8a9   ; t_screen1: ganzer Screen 1 (lv2_id)
.const EXP_ROWS_SCR = $e125   ; t_rows: Kachelzeilen 3..5 von Screen 2 (lv3_id)
.const EXP_MACRO_SCR = $b13d   ; t_macros: tile_draw, tile_draw_color, tile_fill
.const EXP_PUT_SCR = $e9bf   ; t_puts: drei tile_put-Aufrufe
.const MAP_TILES_W = 13   ; Kartenbreite in Kacheln (entspricht levels_tiles_w)
.const DEMO_FX = 6   ; Spalte der 2 x 2 Kacheln, die tile_fill in main/t_macros zeichnet
.const DEMO_FY = 6   ; Zeile dazu
; Selbsttest: drei Stellen der Karte von Screen 1 (Spalte, Zeile, Kachelnummer, Eigenschaftenbyte)
.const SELF_X0 = 0   ; Spalte, Zeile, erwartete Kachelnummer (T) und Eigenschaftenbyte (P)
.const SELF_Y0 = 0
.const SELF_T0 = 3
.const SELF_P0 = 2
.const SELF_X1 = 12
.const SELF_Y1 = 11
.const SELF_T1 = 2
.const SELF_P1 = 1
.const SELF_X2 = 3
.const SELF_Y2 = 4
.const SELF_T2 = 5
.const SELF_P2 = 5
