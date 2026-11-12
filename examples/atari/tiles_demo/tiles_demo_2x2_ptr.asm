; Kachel-Bibliothek, Variante 2x2_ptr, siehe demo.inc: Kachelgröße 2 x 2 Zeichen, Zeigerversion (TILE_SMC = 0, ROM-tauglich).
; Baut mit: vcc asm tiles_demo_2x2_ptr.asm --target atari8 --cpu sally (oder vcc build .); Test: tiles_demo_2x2_ptr.vctest.
.const DEMO_SET = 2
.const TILE_SMC = 0
.include "demo.inc"
