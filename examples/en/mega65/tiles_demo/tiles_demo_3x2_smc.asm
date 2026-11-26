; Tile library, variant 3x2_smc (3 x 2 characters per tile, map 12 x 10 tiles), see demo.inc.
; SMC version: screen_draw modifies its own code, faster, RAM only.
; This program only sets the switches and includes demo.inc.  Build: `vcc build`, check: `vcc test` (tiles_demo_3x2_smc.vctest).
.const DEMO_SET = 3                    ; tile size: 3 = 3x2
.const TILE_SMC = 1                     ; self-modifying code (SMC)
.const TILE_MAPRAM = $3000     ; copy the map into the RAM buffer when selecting
.include "demo.inc"
