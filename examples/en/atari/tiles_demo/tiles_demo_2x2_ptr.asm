; Tile library, variant 2x2_ptr, see demo.inc: tile size 2 x 2 characters, pointer version (TILE_SMC = 0, ROM-safe).
; Build with: vcc asm tiles_demo_2x2_ptr.asm --target atari8 --cpu sally (or vcc build .); test: tiles_demo_2x2_ptr.vctest.
.const DEMO_SET = 2
.const TILE_SMC = 0
.include "demo.inc"
