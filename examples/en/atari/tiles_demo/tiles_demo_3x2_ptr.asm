; Tile library, variant 3x2_ptr, see demo.inc: tile size 3 x 2 characters, pointer version (TILE_SMC = 0, ROM-safe).
; Build with: vcc asm tiles_demo_3x2_ptr.asm --target atari8 --cpu sally (or vcc build .); test: tiles_demo_3x2_ptr.vctest.
.const DEMO_SET = 3
.const TILE_SMC = 0
.include "demo.inc"
