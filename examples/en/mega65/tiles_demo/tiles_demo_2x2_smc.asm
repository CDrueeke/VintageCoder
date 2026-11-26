; Tile library, variant 2x2_smc (2 x 2 characters per tile, map 15 x 10 tiles), see demo.inc.
; SMC version: screen_draw modifies its own code, faster, RAM only.
; This program only sets the switches and includes demo.inc.  Build: `vcc build`, check: `vcc test` (tiles_demo_2x2_smc.vctest).
.const DEMO_SET = 2                    ; tile size: 2 = 2x2
.const TILE_SMC = 1                     ; self-modifying code (SMC)
.include "demo.inc"
