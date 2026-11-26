; Tile library, variant 3x2_rle, see demo.inc
.const DEMO_SET = 3
.const TILE_SMC = 0
.const DEMO_RLE = 1                ; maps run-length encoded (pointer version, ROM-safe)
.const TILE_RLE = 1
.const TILE_MAPRAM = $3000     ; target of unpacking
.include "demo.inc"
