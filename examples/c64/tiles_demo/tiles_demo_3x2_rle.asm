; tiles_demo_3x2_rle.asm – Kachel-Bibliothek (C64), Variante 3x2_rle
;
; Kacheln zu 3 x 2 Zeichen; die Karten der Bank liegen lauflängenkodiert (RLE) vor (levels_rle_*, Paare Kachelnummer, Anzahl) und werden von
; screen_select nach TILE_MAPRAM entpackt (TILE_RLE = 1, Entpacker screen_load_rle). Zeigerversion von screen_draw, also ROM-tauglich.
; Zugehörige Testdatei: tiles_demo_3x2_rle.vctest.
.const DEMO_SET = 3
.const DEMO_RLE = 1
.const TILE_SMC = 0
.const TILE_RLE = 1
.const TILE_MAPRAM = $c800       ; Ziel des Entpackens
.include "demo.inc"
