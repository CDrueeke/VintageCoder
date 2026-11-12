; tiles_demo, Variante 3x2_ptr: Kacheln aus 3 x 2 Zeichen (Karten 7 x 11 Kacheln), Zeigerversion der Kachel-Bibliothek.
; Gemeinsamer Teil (Programm, Prüfroutinen, Speicherplan) in demo.inc; Ziel: VC-20 ohne Erweiterung (tiles_demo.vcprj), Test: tiles_demo_3x2_ptr.vctest.
.const DEMO_SET = 3                 ; Kacheln 3 x 2 Zeichen (data_3x2.inc)
.const TILE_SMC = 0                 ; Zeigerversion: screen_draw arbeitet über Zeropage-Zeiger (ROM-tauglich)
.include "demo.inc"                    ; Programm, Daten, Bibliothek und Prüfroutinen
