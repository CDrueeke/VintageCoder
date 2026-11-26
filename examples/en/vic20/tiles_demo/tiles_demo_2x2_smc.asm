; tiles_demo, variant 2x2_smc: tiles of 2 x 2 characters (maps 11 x 11 tiles), tile library with self-modifying code.
; Common part (program, check routines, memory map) in demo.inc; target: unexpanded VIC-20 (tiles_demo.vcprj), test: tiles_demo_2x2_smc.vctest.
.const DEMO_SET = 2                 ; tiles of 2 x 2 characters (data_2x2.inc)
.const TILE_SMC = 1                 ; self-modifying code: screen_draw changes its own instructions (faster, RAM only)
.include "demo.inc"                    ; program, data, library and check routines
