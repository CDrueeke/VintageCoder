; tiles_demo, variant 3x2_smc: tiles of 3 x 2 characters (maps 7 x 11 tiles), tile library with self-modifying code and map in RAM.
; Common part (program, check routines, memory map) in demo.inc; target: unexpanded VIC-20 (tiles_demo.vcprj), test: tiles_demo_3x2_smc.vctest.
.const DEMO_SET = 3                 ; tiles of 3 x 2 characters (data_3x2.inc)
.const TILE_SMC = 1                 ; self-modifying code: screen_draw changes its own instructions (faster, RAM only)
.const TILE_MAPRAM = $0360     ; copy the map into the RAM buffer when selecting it (cassette buffer $0360-$03AC, 7 x 11 = 77 bytes; $0340 is the trap of the test)
.include "demo.inc"                    ; program, data, library and check routines
