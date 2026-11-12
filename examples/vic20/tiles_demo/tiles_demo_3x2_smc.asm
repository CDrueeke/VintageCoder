; tiles_demo, Variante 3x2_smc: Kacheln aus 3 x 2 Zeichen (Karten 7 x 11 Kacheln), Kachel-Bibliothek mit selbstmodifizierendem Code und Karte im RAM.
; Gemeinsamer Teil (Programm, Prüfroutinen, Speicherplan) in demo.inc; Ziel: VC-20 ohne Erweiterung (tiles_demo.vcprj), Test: tiles_demo_3x2_smc.vctest.
.const DEMO_SET = 3                 ; Kacheln 3 x 2 Zeichen (data_3x2.inc)
.const TILE_SMC = 1                 ; selbstmodifizierender Code: screen_draw ändert seine eigenen Befehle (schneller, nur im RAM)
.const TILE_MAPRAM = $0360     ; Karte beim Wählen in den RAM-Puffer kopieren (Kassettenpuffer $0360-$03AC, 7 x 11 = 77 Byte; $0340 ist die Falle des Tests)
.include "demo.inc"                    ; Programm, Daten, Bibliothek und Prüfroutinen
