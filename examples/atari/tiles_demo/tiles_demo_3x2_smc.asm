; Kachel-Bibliothek, Variante 3x2_smc, siehe demo.inc: Kachelgröße 3 x 2 Zeichen, selbstmodifizierende Version (TILE_SMC = 1, nur im RAM).
; Baut mit: vcc asm tiles_demo_3x2_smc.asm --target atari8 --cpu sally (oder vcc build .); Test: tiles_demo_3x2_smc.vctest.
; Zusätzlich TILE_MAPRAM = $5000: screen_select kopiert die Karte in diesen RAM-Puffer, gezeichnet wird aus dem Puffer.
.const DEMO_SET = 3
.const TILE_SMC = 1
.const TILE_MAPRAM = $5000     ; Karte beim Wählen in den RAM-Puffer kopieren
.include "demo.inc"
