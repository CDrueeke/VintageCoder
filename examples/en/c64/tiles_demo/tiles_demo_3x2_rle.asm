; tiles_demo_3x2_rle.asm - tile library (C64), variant 3x2_rle
;
; Tiles of 3 x 2 characters; the maps of the bank are run-length encoded (RLE) (levels_rle_*, pairs of tile number, count) and are
; unpacked by screen_select to TILE_MAPRAM (TILE_RLE = 1, unpacker screen_load_rle). Pointer version of screen_draw, so ROM-capable.
; Matching test file: tiles_demo_3x2_rle.vctest.
.const DEMO_SET = 3
.const DEMO_RLE = 1
.const TILE_SMC = 0
.const TILE_RLE = 1
.const TILE_MAPRAM = $c800       ; target of the unpacking
.include "demo.inc"
