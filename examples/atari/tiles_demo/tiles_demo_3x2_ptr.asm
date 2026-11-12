; Kachel-Bibliothek, Variante 3x2_ptr, siehe demo.inc: Kachelgröße 3 x 2 Zeichen, Zeigerversion (TILE_SMC = 0, ROM-tauglich).
; Baut mit: vcc asm tiles_demo_3x2_ptr.asm --target atari8 --cpu sally (oder vcc build .); Test: tiles_demo_3x2_ptr.vctest.
.const DEMO_SET = 3
.const TILE_SMC = 0
.include "demo.inc"
