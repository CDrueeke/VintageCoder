; Kachel-Bibliothek, Variante 3x2_smc (3 x 2 Zeichen je Kachel, Karte 12 x 10 Kacheln), siehe demo.inc.
; SMC-Version: screen_draw ändert seinen eigenen Code, schneller, nur im RAM.
; Dieses Programm legt nur die Schalter fest und bindet demo.inc ein.  Bauen: `vcc build`, Prüfen: `vcc test` (tiles_demo_3x2_smc.vctest).
.const DEMO_SET = 3                    ; Kachelgröße: 3 = 3x2
.const TILE_SMC = 1                     ; selbständernder Code (SMC)
.const TILE_MAPRAM = $3000     ; Karte beim Wählen in den RAM-Puffer kopieren
.include "demo.inc"
