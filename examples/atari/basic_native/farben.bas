; =====================================================================================================================
; farben.bas - Natives Atari BASIC ohne Zeilennummern: Labels statt GOTO-Nummern
; =====================================================================================================================
; Beim Bauen wird nummeriert (10, 20, 30 ...) und "@name" durch die Zeilennummer ersetzt.  Strichpunkt-Zeilen sind
; Kommentare des Quelltextes und kommen nicht ins Programm.
;
; WAS ZU SEHEN IST
;   Ein Textbildschirm (GRAPHICS 0) mit schwarzem Hintergrund und Rand.  Der Hintergrund durchläuft in einer Endlosschleife
;   die 16 Farbtöne (0-15) bei Helligkeit 4; mittig (Spalte 8, Zeile 10) steht der Text "ATARI BASIC MIT LABELS".
;   Zeigt: Label-Syntax (@neu: / GOTO @neu), POKE auf Betriebssystem-Farbschatten, SETCOLOR, POSITION, FOR-Schleifen.
;
; BAUEN UND STARTEN
;   vcc build .  -> tokenisiertes Atari-BASIC-Programm (Ziel atari8, Startwerte der Nummerierung 10, 10); es läuft im
;   Atari BASIC (eingebaut im 800XL) und wird dort mit RUN gestartet.
;
; VARIABLEN: C = Farbton 0..15 (Schleifenzähler), W = Zähler der Warteschleife.
GRAPHICS 0
; Adresse 710 = COLOR2 (Schatten von COLPF2, Hintergrund des Textfeldes), 712 = COLOR4 (Schatten von COLBK, Rand): beides schwarz
POKE 710,0 : POKE 712,0
@neu:
FOR C=0 TO 15
  ; SETCOLOR Register,Farbton,Helligkeit: Register 2 = Hintergrund (COLOR2), Helligkeit 4 (0..14, gerade Werte)
  SETCOLOR 2,C,4
  ; leere Warteschleife (kurze Pause je Farbe), damit der Farbwechsel sichtbar bleibt
  FOR W=1 TO 20 : NEXT W
NEXT C
; Text in Spalte 8, Zeile 10 des Textschirms; "?" ist die Kurzform von PRINT
POSITION 8,10 : ? "ATARI BASIC MIT LABELS"
; Endlosschleife: zurück zum Label, das Programm läuft bis BREAK oder RESET
GOTO @neu
