; tiles_demo, variant 2x2_ptr: tiles of 2 x 2 characters (maps 11 x 11 tiles), pointer version of the tile library.
; Common part (program, check routines, memory map) in demo.inc; target: unexpanded VIC-20 (tiles_demo.vcprj), test: tiles_demo_2x2_ptr.vctest.
.const DEMO_SET = 2                 ; tiles of 2 x 2 characters (data_2x2.inc)
.const TILE_SMC = 0                 ; pointer version: screen_draw works via zero page pointers (ROM-capable)
.include "demo.inc"                    ; program, data, library and check routines
