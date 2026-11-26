; =====================================================================================================================
; tiles_demo_2x2_smc.asm - tile library, 2x2 tile variant with SMC draw routine (C128, VIC-IIe, 40 columns)
; =====================================================================================================================
; This file only sets two switches and then includes the shared part demo.inc (it holds the program, memory map,
; library calls and test routines; see its header).  Build it with "vcc build ." (all four variants) or
; "vcc asm tiles_demo_2x2_smc.asm --target c128 --cpu 8502"; tested with tiles_demo_2x2_smc.vctest ("vcc test").
;   DEMO_SET = 2   tile set with 2x2 character cells per tile (data_2x2.inc, expect_2x2.inc); 3 would be 3x2 (data_3x2.inc)
;   TILE_SMC = 1   self-modifying (SMC) version of screen_draw: faster, but runs only in RAM (not in ROM/cartridge)
.const DEMO_SET = 2
.const TILE_SMC = 1
.include "demo.inc"
