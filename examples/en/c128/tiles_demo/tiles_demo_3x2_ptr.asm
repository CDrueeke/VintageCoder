; =====================================================================================================================
; tiles_demo_3x2_ptr.asm - tile library, 3x2 tile variant with POINTER draw routine (C128, VIC-IIe, 40 columns)
; =====================================================================================================================
; This file only sets two switches and then includes the shared part demo.inc (it holds the program, memory map,
; library calls and test routines; see its header).  Build it with "vcc build ." (all four variants) or
; "vcc asm tiles_demo_3x2_ptr.asm --target c128 --cpu 8502"; tested with tiles_demo_3x2_ptr.vctest ("vcc test").
;   DEMO_SET = 3   tile set with 3x2 character cells per tile (data_3x2.inc, expect_3x2.inc); 2 would be 2x2 (data_2x2.inc)
;   TILE_SMC = 0   pointer version of screen_draw (zero page pointer, ROM-safe); 1 would be the self-modifying (SMC) version
; This is the main file of the project (main in tiles_demo.vcprj).  The map is read directly from the map array (no TILE_MAPRAM).
.const DEMO_SET = 3
.const TILE_SMC = 0
.include "demo.inc"
