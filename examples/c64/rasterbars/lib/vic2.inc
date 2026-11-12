; vic2.inc – VIC-II-Register (Auszug) und ein Warte-Makro für das Beispiel rasterbars.asm
;
; Die Konstanten tragen hier absichtlich eigene Namen (vic_*), um das Einbinden einer Datei mit .include zu zeigen. Dieselben
; Register stehen auch unter vic2.raster, vic2.border, vic2.bg0 usw. aus den Zieldaten (data/chips/vic2.chip.json) bereit.
.const vic_raster = $d012       ; aktuelle Rasterzeile, Bits 0-7 (lesen); Bit 8 steht in Bit 7 von $D011
.const vic_border = $d020       ; Rahmenfarbe (Farbnummer 0-15)
.const vic_bg0    = $d021       ; Hintergrundfarbe 0 (Farbnummer 0-15)

; wait_line(line) – wartet in einer Schleife, bis der VIC-II die Rasterzeile `line` erreicht hat.
;   Eingabe:  line = Rasterzeile 0-255 (nur das untere Byte wird verglichen; Zeilen ab 256 sind so nicht erreichbar)
;   Ausgabe:  keine; verändert A (= line) und die Flags N, Z, C. X und Y bleiben unberührt.
;   Besonderheit: reine Warteschleife (aktives Warten, kein Interrupt). Trifft der Aufruf genau in der Zeile ein, kehrt das Makro
;   sofort zurück; wird die Zeile verpasst (z. B. durch einen langen Interrupt), wartet es bis zum nächsten Bild.
;   `@w` ist eine nur im Makro gültige lokale Marke, so dass es mehrfach verwendet werden kann.
.macro wait_line(line)
        lda #line
@w:     cmp vic_raster
        bne @w
.endmacro
