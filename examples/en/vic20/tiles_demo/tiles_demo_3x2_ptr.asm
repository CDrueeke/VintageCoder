; tiles_demo, variant 3x2_ptr: tiles of 3 x 2 characters (maps 7 x 11 tiles), pointer version of the tile library.
; Common part (program, check routines, memory map) in demo.inc; target: unexpanded VIC-20 (tiles_demo.vcprj), test: tiles_demo_3x2_ptr.vctest.
.const DEMO_SET = 3                 ; tiles of 3 x 2 characters (data_3x2.inc)
.const TILE_SMC = 0                 ; pointer version: screen_draw works via zero page pointers (ROM-capable)
.include "demo.inc"                    ; program, data, library and check routines
