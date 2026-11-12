; tiles_demo_3x2_ptr.asm – Kachel-Bibliothek (C64), Variante 3x2_ptr
;
; Kacheln zu 3 x 2 Zeichen; Zeigerversion von screen_draw (TILE_SMC = 0: Zeropage-Zeiger, ROM-tauglich).
; Dieses Programm legt nur die Schalter der Variante fest (DEMO_SET: 3 = 3x2-Kacheln, 2 = 2x2-Kacheln; TILE_SMC)
; und bindet dann demo.inc ein; dort stehen Programm, Daten, Prüfroutinen und die ausführliche Beschreibung.
; Zugehörige Testdatei: tiles_demo_3x2_ptr.vctest.
; Bauen: `vcc build examples/c64/tiles_demo`, testen: `vcc test examples/c64/tiles_demo [--backend vice]`.
.const DEMO_SET = 3              ; 3x2-Kacheln: wählt in demo.inc die Daten data_3x2.inc und expect_3x2.inc
.const TILE_SMC = 0               ; 0 = Zeigerversion in screen_draw
.include "demo.inc"
