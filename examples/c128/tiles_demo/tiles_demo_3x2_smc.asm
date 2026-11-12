; =====================================================================================================================
; tiles_demo_3x2_smc.asm - Kachel-Bibliothek, Variante 3x2-Kacheln mit SMC-Zeichenroutine und Kartenpuffer (C128, VIC-IIe, 40 Spalten)
; =====================================================================================================================
; Diese Datei legt die Schalter fest und bindet dann den gemeinsamen Teil demo.inc ein (dort stehen Programm, Speicherplan,
; Bibliotheksaufrufe und Prüfroutinen; siehe dessen Kopf).  Gebaut wird sie mit "vcc build ." (alle vier Varianten) bzw.
; "vcc asm tiles_demo_3x2_smc.asm --target c128 --cpu 8502"; getestet mit tiles_demo_3x2_smc.vctest ("vcc test").
;   DEMO_SET = 3        Kachelsatz mit 3x2-Zeichenzellen je Kachel (data_3x2.inc, expect_3x2.inc); 2 wäre 2x2 (data_2x2.inc)
;   TILE_SMC = 1        selbstverändernde (SMC-)Version von screen_draw: schneller, läuft aber nur im RAM (nicht im ROM/Modul)
;   TILE_MAPRAM = $3000 RAM-Puffer für die Karte: screen_select kopiert die Karte des gewählten Screens dorthin, screen_draw zeichnet
;                       aus dem Puffer.  So darf ein Spiel die Karte ändern, ohne das Kartenfeld im Programm zu verändern.
;                       $3000 liegt hinter dem Programm (Obergrenze $3000, siehe .assert am Ende von demo.inc), noch im freien
;                       RAM unterhalb von $4000 (dort beginnen bei der Standardkonfiguration die Systemroms).
.const DEMO_SET = 3
.const TILE_SMC = 1
.const TILE_MAPRAM = $3000     ; Karte beim Wählen in den RAM-Puffer kopieren
.include "demo.inc"
