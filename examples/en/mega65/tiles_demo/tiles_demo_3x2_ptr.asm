; Tile library, variant 3x2_ptr (3 x 2 characters per tile, map 12 x 10 tiles), see demo.inc.
; Pointer version: screen_draw works through zero page pointers, ROM-capable.
; This program only sets the switches and includes demo.inc.  Build: `vcc build`, check: `vcc test` (tiles_demo_3x2_ptr.vctest).
.const DEMO_SET = 3                    ; tile size: 3 = 3x2
.const TILE_SMC = 0                     ; pointer version
.include "demo.inc"
