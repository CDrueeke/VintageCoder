; tiles_demo, Variante 2x2_smc: Kacheln aus 2 x 2 Zeichen (Karten 11 x 11 Kacheln), Kachel-Bibliothek mit selbstmodifizierendem Code.
; Gemeinsamer Teil (Programm, Prüfroutinen, Speicherplan) in demo.inc; Ziel: VC-20 ohne Erweiterung (tiles_demo.vcprj), Test: tiles_demo_2x2_smc.vctest.
.const DEMO_SET = 2                 ; Kacheln 2 x 2 Zeichen (data_2x2.inc)
.const TILE_SMC = 1                 ; selbstmodifizierender Code: screen_draw ändert seine eigenen Befehle (schneller, nur im RAM)
.include "demo.inc"                    ; Programm, Daten, Bibliothek und Prüfroutinen
