; =====================================================================================================================
; tiles_demo_2x2_smc.asm - Kachel-Bibliothek, Variante 2x2-Kacheln mit SMC-Zeichenroutine (C128, VIC-IIe, 40 Spalten)
; =====================================================================================================================
; Diese Datei legt nur zwei Schalter fest und bindet dann den gemeinsamen Teil demo.inc ein (dort stehen Programm, Speicherplan,
; Bibliotheksaufrufe und Prüfroutinen; siehe dessen Kopf).  Gebaut wird sie mit "vcc build ." (alle vier Varianten) bzw.
; "vcc asm tiles_demo_2x2_smc.asm --target c128 --cpu 8502"; getestet mit tiles_demo_2x2_smc.vctest ("vcc test").
;   DEMO_SET = 2   Kachelsatz mit 2x2-Zeichenzellen je Kachel (data_2x2.inc, expect_2x2.inc); 3 wäre 3x2 (data_3x2.inc)
;   TILE_SMC = 1   selbstverändernde (SMC-)Version von screen_draw: schneller, läuft aber nur im RAM (nicht im ROM/Modul)
.const DEMO_SET = 2
.const TILE_SMC = 1
.include "demo.inc"
