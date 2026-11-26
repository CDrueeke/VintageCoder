; Tile library, variant 2x2_ptr (2 x 2 characters per tile, map 15 x 10 tiles), see demo.inc.
; Pointer version: screen_draw works through zero page pointers, ROM-capable.
; This program only sets the switches and includes demo.inc.  Build: `vcc build`, check: `vcc test` (tiles_demo_2x2_ptr.vctest).
.const DEMO_SET = 2                    ; tile size: 2 = 2x2
.const TILE_SMC = 0                     ; pointer version
.include "demo.inc"
