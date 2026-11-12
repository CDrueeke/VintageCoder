; =====================================================================================================================
; tiles_demo_3x2_ptr.asm - Kachel-Bibliothek, Variante 3x2-Kacheln mit ZEIGER-Zeichenroutine (C128, VIC-IIe, 40 Spalten)
; =====================================================================================================================
; Diese Datei legt nur zwei Schalter fest und bindet dann den gemeinsamen Teil demo.inc ein (dort stehen Programm, Speicherplan,
; Bibliotheksaufrufe und Prüfroutinen; siehe dessen Kopf).  Gebaut wird sie mit "vcc build ." (alle vier Varianten) bzw.
; "vcc asm tiles_demo_3x2_ptr.asm --target c128 --cpu 8502"; getestet mit tiles_demo_3x2_ptr.vctest ("vcc test").
;   DEMO_SET = 3   Kachelsatz mit 3x2-Zeichenzellen je Kachel (data_3x2.inc, expect_3x2.inc); 2 wäre 2x2 (data_2x2.inc)
;   TILE_SMC = 0   Zeigerversion von screen_draw (Zeropage-Zeiger, ROM-tauglich); 1 wäre die selbstverändernde (SMC-)Version
; Dies ist die Hauptdatei des Projekts (main in tiles_demo.vcprj).  Die Karte wird direkt aus dem Kartenfeld gelesen (kein TILE_MAPRAM).
.const DEMO_SET = 3
.const TILE_SMC = 0
.include "demo.inc"
