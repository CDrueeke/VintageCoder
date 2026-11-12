; tiles_demo_3x2_smc.asm – Kachel-Bibliothek (C64), Variante 3x2_smc
;
; Kacheln zu 3 x 2 Zeichen; selbstverändernde Version von screen_draw (TILE_SMC = 1: der Code ändert sich selbst, schneller, nur im RAM lauffähig).
; Dieses Programm legt nur die Schalter der Variante fest (DEMO_SET: 3 = 3x2-Kacheln, 2 = 2x2-Kacheln; TILE_SMC, TILE_MAPRAM)
; und bindet dann demo.inc ein; dort stehen Programm, Daten, Prüfroutinen und die ausführliche Beschreibung.
; Zugehörige Testdatei: tiles_demo_3x2_smc.vctest.
; Bauen: `vcc build examples/c64/tiles_demo`, testen: `vcc test examples/c64/tiles_demo [--backend vice]`.
.const DEMO_SET = 3              ; 3x2-Kacheln: wählt in demo.inc die Daten data_3x2.inc und expect_3x2.inc
.const TILE_SMC = 1               ; 1 = selbstverändernder Code in screen_draw
; TILE_MAPRAM: RAM-Puffer für die Karte (120 Bytes = 12 x 10 Kacheln). screen_select kopiert die Karte des gewählten Screens hierher,
; screen_draw zeichnet daraus; die Spiellogik darf die Karte dort ändern.
.const TILE_MAPRAM = $c800
.include "demo.inc"
