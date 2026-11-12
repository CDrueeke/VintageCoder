; Kachel-Bibliothek, Variante 2x2_smc, siehe demo.inc: Kachelgröße 2 x 2 Zeichen, selbstmodifizierende Version (TILE_SMC = 1, nur im RAM).
; Baut mit: vcc asm tiles_demo_2x2_smc.asm --target atari8 --cpu sally (oder vcc build .); Test: tiles_demo_2x2_smc.vctest.
.const DEMO_SET = 2
.const TILE_SMC = 1
.include "demo.inc"
