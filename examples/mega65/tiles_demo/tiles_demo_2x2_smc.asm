; Kachel-Bibliothek, Variante 2x2_smc (2 x 2 Zeichen je Kachel, Karte 15 x 10 Kacheln), siehe demo.inc.
; SMC-Version: screen_draw ändert seinen eigenen Code, schneller, nur im RAM.
; Dieses Programm legt nur die Schalter fest und bindet demo.inc ein.  Bauen: `vcc build`, Prüfen: `vcc test` (tiles_demo_2x2_smc.vctest).
.const DEMO_SET = 2                    ; Kachelgröße: 2 = 2x2
.const TILE_SMC = 1                     ; selbständernder Code (SMC)
.include "demo.inc"
