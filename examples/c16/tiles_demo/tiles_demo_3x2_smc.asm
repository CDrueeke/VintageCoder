; Kachel-Bibliothek, Variante 3x2_smc, siehe demo.inc
; Kacheln 3x2 Zeichen, selbstverändernder Code (TILE_SMC = 1: schneller, nur im RAM).
; Bauen: vcc asm tiles_demo_3x2_smc.asm --target c16 --cpu 7501 -o <ausgabe>.prg, oder vcc build im Ordner; testen: vcc test --backend vice.
; Dateien dieser Gruppe werden von tools/gen_tiles_demo.py erzeugt.
.const DEMO_SET = 3                 ; Kachelgröße/Datensatz: 3 = 3x2 (data_3x2.inc), 2 = 2x2 (data_2x2.inc)
.const TILE_SMC = 1                 ; 0 = Zeigerversion, 1 = selbstveränderter Code
.const TILE_MAPRAM = $3000     ; RAM-Puffer der Karte: screen_select kopiert die Karte dorthin (Spiellogik dürfte sie ändern)
.include "demo.inc"
