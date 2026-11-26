; =====================================================================================================================
; tiles_demo_3x2_smc.asm - tile library, 3x2 tile variant with SMC draw routine and map buffer (C128, VIC-IIe, 40 columns)
; =====================================================================================================================
; This file sets the switches and then includes the shared part demo.inc (it holds the program, memory map,
; library calls and test routines; see its header).  Build it with "vcc build ." (all four variants) or
; "vcc asm tiles_demo_3x2_smc.asm --target c128 --cpu 8502"; tested with tiles_demo_3x2_smc.vctest ("vcc test").
;   DEMO_SET = 3        tile set with 3x2 character cells per tile (data_3x2.inc, expect_3x2.inc); 2 would be 2x2 (data_2x2.inc)
;   TILE_SMC = 1        self-modifying (SMC) version of screen_draw: faster, but runs only in RAM (not in ROM/cartridge)
;   TILE_MAPRAM = $3000 RAM buffer for the map: screen_select copies the map of the chosen screen there, screen_draw draws
;                       from the buffer.  So a game may change the map without altering the map array in the program.
;                       $3000 lies behind the program (upper limit $3000, see .assert at the end of demo.inc), still in free
;                       RAM below $4000 (the system ROMs start there in the default configuration).
.const DEMO_SET = 3
.const TILE_SMC = 1
.const TILE_MAPRAM = $3000     ; copy the map to the RAM buffer when selecting
.include "demo.inc"
