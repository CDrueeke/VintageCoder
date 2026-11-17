; Erwartete CRC-16 (XMODEM) über Bildschirm (960 Byte), ohne Farbspeicher je Szenario; erzeugt von tools/gen_tiles_demo.py
.const EXP_S1_SCR = $d8a9
.const EXP_ROWS_SCR = $e125
.const EXP_MACRO_SCR = $b13d
.const EXP_PUT_SCR = $e9bf
.const EXP_ERASE_SCR = $2c05
.const EXP_RESTORE_SCR = $2efc
.const EXP_ROW_SCR = $08ff
.const MAP_TILES_W = 13
.const DEMO_FX = 6
.const DEMO_FY = 6
; Selbsttest: drei Stellen der Karte von Screen 1 (Spalte, Zeile, Kachelnummer, Eigenschaftenbyte)
.const SELF_X0 = 0
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
