; =====================================================================================================================
; colorbars.asm - Rasterbalken für den Atari 800XL (PAL): Hintergrundfarbe = Rasterzeile
; =====================================================================================================================
;
; WAS ZU SEHEN IST
;   Ein Farbverlauf über die ganze Bildhöhe: die Hintergrundfarbe (COLBK) ändert sich fortlaufend von Zeile zu Zeile.  Weil
;   ein Farbwert aus Farbton (Bits 7-4) und Helligkeit (Bits 3-1, Bit 0 unbenutzt) besteht, laufen sichtbar Helligkeitsstufen
;   durch, und alle 32 Rasterzeilen wechselt der Farbton.  Sichtbar ist das vermutlich vor allem im Rand und überall dort, wo
;   der OS-Textschirm den Hintergrund nicht selbst mit COLPF2 deckt.  Die Datei enthält keinen Display-List-Interrupt:
;   die Farbe wird in einer Schleife von der CPU gesetzt (siehe unten).
;
; WAS DER CODE TUT
;   1. NMIEN = 0 schaltet die Vertical-Blank- und Display-List-NMIs ab.  Damit läuft die OS-Bildschirmaktualisierung nicht
;      mehr (sie kopiert sonst den Schatten COLOR4 = $02C8 nach COLBK und würde die Balken überschreiben).
;   2. Endlosschleife, genau ein Durchlauf je Rasterzeile:
;        - VCOUNT lesen (ANTIC-Zeilenzähler, Zeile / 2, bei PAL 0..155),
;        - STA WSYNC: die CPU wird bis zum Zeilenende angehalten, danach beginnt die Schleife immer an derselben
;          Stelle der nächsten Zeile (kein Flimmern am Zeilenanfang),
;        - den gelesenen VCOUNT-Wert als Farbwert nach COLBK schreiben.
;      VCOUNT wächst nur alle 2 Zeilen, deshalb ist jeder Farbwert 2 Rasterzeilen breit.  Der Wert wird direkt als Farbe
;      benutzt (Farbton = obere vier Bit, Helligkeit = Bits 3-1).
;
; REGISTER:  ANTIC NMIEN $D40E, VCOUNT $D40B, WSYNC $D40A;  GTIA COLBK $D01A (Namen aus dem Target atari8).
; SPEICHER:  Programm ab $2000 (17 Bytes), keine Daten, keine Nullseite.
; BAUEN:     vcc asm examples/atari/colorbars/colorbars.asm --target atari8 --cpu sally -o colorbars.xex
; GRENZEN:   Kein Rückweg ins Betriebssystem (Endlosschleife); Reset beendet das Programm.
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }

.org $2000
start:  lda #0
        sta antic.nmien         ; VBI/DLI aus: das OS überschreibt COLBK nicht mehr
.line:  ldx antic.vcount        ; X = Zeile / 2 (ändert sich nur alle 2 Rasterzeilen)
        stx antic.wsync         ; bis zum Zeilenende warten (der geschriebene Wert ist egal)
        stx gtia.colbk          ; Farbe = VCOUNT (Zeile / 2), gilt ab dem Anfang der nächsten Zeile
        jmp .line
.run start
