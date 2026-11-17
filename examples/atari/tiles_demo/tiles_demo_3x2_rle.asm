; Kachel-Bibliothek, Variante 3x2_rle, siehe demo.inc
.const DEMO_SET = 3
.const TILE_SMC = 0
.const DEMO_RLE = 1                ; Karten lauflängenkodiert (Zeigerversion, ROM-tauglich)
.const TILE_RLE = 1
.const TILE_MAPRAM = $5000     ; Ziel des Entpackens
.include "demo.inc"
