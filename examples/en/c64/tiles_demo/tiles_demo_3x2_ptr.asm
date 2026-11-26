; tiles_demo_3x2_ptr.asm - tile library (C64), variant 3x2_ptr
;
; Tiles of 3 x 2 characters; pointer version of screen_draw (TILE_SMC = 0: zero page pointer, ROM-capable).
; This program only sets the switches of the variant (DEMO_SET: 3 = 3x2 tiles, 2 = 2x2 tiles; TILE_SMC)
; and then includes demo.inc; that file holds the program, data, check routines and the detailed description.
; Matching test file: tiles_demo_3x2_ptr.vctest.
; Build: `vcc build examples/c64/tiles_demo`, test: `vcc test examples/c64/tiles_demo [--backend vice]`.
.const DEMO_SET = 3              ; 3x2 tiles: selects the data data_3x2.inc and expect_3x2.inc in demo.inc
.const TILE_SMC = 0               ; 0 = pointer version in screen_draw
.include "demo.inc"
