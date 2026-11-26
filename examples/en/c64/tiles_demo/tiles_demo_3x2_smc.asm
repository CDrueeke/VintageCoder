; tiles_demo_3x2_smc.asm - tile library (C64), variant 3x2_smc
;
; Tiles of 3 x 2 characters; self-modifying version of screen_draw (TILE_SMC = 1: the code modifies itself, faster, runs in RAM only).
; This program only sets the switches of the variant (DEMO_SET: 3 = 3x2 tiles, 2 = 2x2 tiles; TILE_SMC, TILE_MAPRAM)
; and then includes demo.inc; that file holds the program, data, check routines and the detailed description.
; Matching test file: tiles_demo_3x2_smc.vctest.
; Build: `vcc build examples/c64/tiles_demo`, test: `vcc test examples/c64/tiles_demo [--backend vice]`.
.const DEMO_SET = 3              ; 3x2 tiles: selects the data data_3x2.inc and expect_3x2.inc in demo.inc
.const TILE_SMC = 1               ; 1 = self-modifying code in screen_draw
; TILE_MAPRAM: RAM buffer for the map (120 bytes = 12 x 10 tiles). screen_select copies the map of the selected screen here,
; screen_draw draws from it; the game logic may change the map there.
.const TILE_MAPRAM = $c800
.include "demo.inc"
