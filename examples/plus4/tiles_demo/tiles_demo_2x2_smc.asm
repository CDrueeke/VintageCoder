; Kachel-Bibliothek, Variante 2x2_smc, siehe demo.inc
; Kacheln 2x2 Zeichen, selbstveraendernder Code (TILE_SMC = 1: schneller, nur im RAM).
; Bauen: vcc asm tiles_demo_2x2_smc.asm --target plus4 --cpu 7501 -o <ausgabe>.prg, oder vcc build im Ordner; testen: vcc test --backend vice.
; Dateien dieser Gruppe werden von tools/gen_tiles_demo.py erzeugt.
.const DEMO_SET = 2                 ; Kachelgroesse/Datensatz: 3 = 3x2 (data_3x2.inc), 2 = 2x2 (data_2x2.inc)
.const TILE_SMC = 1                 ; 0 = Zeigerversion, 1 = selbstveraendernder Code
.include "demo.inc"
