; Kachel-Bibliothek, Variante 3x2_ptr (3 x 2 Zeichen je Kachel, Karte 12 x 10 Kacheln), siehe demo.inc.
; Zeigerversion: screen_draw arbeitet über Zeropage-Zeiger, ROM-tauglich.
; Dieses Programm legt nur die Schalter fest und bindet demo.inc ein.  Bauen: `vcc build`, Prüfen: `vcc test` (tiles_demo_3x2_ptr.vctest).
.const DEMO_SET = 3                    ; Kachelgröße: 3 = 3x2
.const TILE_SMC = 0                     ; Zeigerversion
.include "demo.inc"
