; Kachel-Bibliothek, Variante 2x2_ptr (2 x 2 Zeichen je Kachel, Karte 15 x 10 Kacheln), siehe demo.inc.
; Zeigerversion: screen_draw arbeitet über Zeropage-Zeiger, ROM-tauglich.
; Dieses Programm legt nur die Schalter fest und bindet demo.inc ein.  Bauen: `vcc build`, Prüfen: `vcc test` (tiles_demo_2x2_ptr.vctest).
.const DEMO_SET = 2                    ; Kachelgröße: 2 = 2x2
.const TILE_SMC = 0                     ; Zeigerversion
.include "demo.inc"
