; Tile library, variant 3x2_smc, see demo.inc: tile size 3 x 2 characters, self-modifying version (TILE_SMC = 1, RAM only).
; Build with: vcc asm tiles_demo_3x2_smc.asm --target atari8 --cpu sally (or vcc build .); test: tiles_demo_3x2_smc.vctest.
; In addition TILE_MAPRAM = $5000: screen_select copies the map into this RAM buffer, drawing is done from the buffer.
.const DEMO_SET = 3
.const TILE_SMC = 1
.const TILE_MAPRAM = $5000     ; copy the map to the RAM buffer when selecting
.include "demo.inc"
