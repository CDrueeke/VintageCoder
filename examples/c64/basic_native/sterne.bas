; sterne.bas – Natives C64-BASIC V2 ohne Zeilennummern: Marken (Labels) statt GOTO-Nummern
;
; Zweck: Zeigt den "nativen" Quelltext: Die Zeilen werden beim Bauen nummeriert (10, 20, 30 …), "@name" wird durch die Zeilennummer
;   ersetzt. Wer will, schreibt Zeilennummern wie gewohnt davor – beides lässt sich mischen. Das Ergebnis ist ein gewöhnliches BASIC-V2-
;   Programm des C64 (siehe build/sterne.classic.txt und build/sterne.labels.txt für Listing und Zuordnung der Marken).
;
; Ablauf: Mit `vcc build examples/c64/basic_native` bauen, build/sterne.prg laden und mit RUN starten. Das Programm setzt Rahmen und
;   Hintergrund auf Schwarz, löscht den Bildschirm und streut 200 Sterne ("*") in zufälligen Farben und an zufälligen Stellen
;   (Spalten 1-38, Zeilen 2-23; Zeile 0 trägt die Überschrift). Dann schreibt es "fertig" in die letzte Bildschirmzeile und bleibt in einer
;   Endlosschleife stehen (die STOP-Taste beendet es). Zufallszahlen liefert RND(1); ohne eigenen Startwert beginnt die Folge nach jedem
;   Reset gleich (ein zweites RUN ohne Reset setzt sie fort). Ein Stern kann einen früheren überschreiben; es erscheinen deshalb
;   höchstens 200 Sterne.
;
; Hardware und Speicher, die direkt mit POKE angesprochen werden:
;   53280 ($D020) Rahmenfarbe, 53281 ($D021) Hintergrundfarbe
;   1024 ($0400) Bildschirmspeicher, 40 Zeichen je Zeile: Adresse = 1024 + Zeile*40 + Spalte (Werte sind Bildschirmcodes, 42 = "*")
;   55296 ($D800) Farb-RAM mit gleicher Aufteilung: Adresse = 55296 + Zeile*40 + Spalte (Wert = Farbe 0-15)
; Variablen: x = Spalte, y = Zeile, n = Zähler der gesetzten Sterne (beginnt ohne Zuweisung bei 0).
; Hinweis: `;` ist nur am Zeilenanfang ein Kommentar (mitten in der Zeile wäre es in PRINT das Anhängezeichen); Erklärungen stehen
;   deshalb auf eigenen Zeilen. `rem` dagegen bleibt im Programm und kostet Speicher und Zeit.
poke 53280,0 : poke 53281,0
print "{clr}{wht}sterne fuer den c64"

@neu:
; Spalte 1..38 (int(rnd*38) liefert 0..37) und Zeile 2..23 (int(rnd*22) liefert 0..21) auswählen
x = int(rnd(1)*38) + 1
y = int(rnd(1)*22) + 2
poke 1024 + y*40 + x, 42          : rem stern (bildschirmcode 42)
; Farbe 1..15 (Schwarz, 0, ist ausgeschlossen, damit der Stern auf dem schwarzen Hintergrund sichtbar bleibt)
poke 55296 + y*40 + x, int(rnd(1)*15) + 1
n = n + 1
if n < 200 then @neu

; Cursor nach links oben setzen und 24 Zeilen nach unten, also in die letzte Zeile (Zeile 24 des 25-zeiligen Bildschirms)
print "{home}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}{down}fertig"
@ende:
; Endlosschleife, damit "READY." den Bildschirm nicht überschreibt
goto @ende
