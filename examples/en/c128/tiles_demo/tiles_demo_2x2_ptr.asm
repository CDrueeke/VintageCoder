; =====================================================================================================================
; tiles_demo_2x2_ptr.asm - tile library, 2x2 tile variant with POINTER draw routine (C128, VIC-IIe, 40 columns)
; =====================================================================================================================
; This file only sets two switches and then includes the shared part demo.inc (it holds the program, memory map,
; library calls and test routines; see its header).  Build it with "vcc build ." (all four variants) or
; "vcc asm tiles_demo_2x2_ptr.asm --target c128 --cpu 8502"; tested with tiles_demo_2x2_ptr.vctest ("vcc test").
;   DEMO_SET = 2   tile set with 2x2 character cells per tile (data_2x2.inc, expect_2x2.inc); 3 would be 3x2 (data_3x2.inc)
;   TILE_SMC = 0   pointer version of screen_draw (zero page pointer, ROM-safe); 1 would be the self-modifying (SMC) version
.const DEMO_SET = 2
.const TILE_SMC = 0
.include "demo.inc"
