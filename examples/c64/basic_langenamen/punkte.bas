#longnames
; punkte.bas – C64-BASIC V2 mit langen Variablennamen (#longnames)
;
; Zweck: Das BASIC des C64 unterscheidet Variablen nur an den ersten zwei Zeichen: `punkte` und `punkt` wären dieselbe Variable (PU), und
;   Namen wie `total` (enthält TO) oder `score` (enthält OR) zerlegt der Interpreter in Schlüsselwörter. Die Zeile `#longnames` (hier die
;   erste Zeile; sie darf überall stehen und bekommt keine Zeilennummer) erlaubt dagegen lange, sprechende Namen: der Bau bildet jeden auf
;   ein eindeutiges Namenspaar ab (hier z. B. punkte -> PU, runde -> RU, zaehler% -> ZA%, name$ -> NA$, feld() -> FE(), fn quad -> FN QU).
;   Die vollständige Zuordnung steht im Bauprotokoll und in build/punkte.vars.txt, das klassische Listing in build/punkte.classic.txt.
;
; Ablauf: Mit `vcc build examples/c64/basic_langenamen` bauen, build/punkte.prg laden und mit RUN starten. Das Programm löscht den
;   Bildschirm (weiße Schrift), berechnet in einer Schleife die Quadratzahlen 1, 4 und 9 und summiert sie in `punkte`
;   (14 je Durchgang). Es gibt drei Zeilen der Form „hero runde 1 punkte 14“ (Runde 2: 28, Runde 3: 42) aus und bleibt dann in einer
;   Endlosschleife stehen (die STOP-Taste beendet es).
;
; Sprachmittel, die das Beispiel zeigt:
;   - `@neu:` und `@ende:` sind Marken (Labels); `then @neu` und `goto @ende` springen dorthin, die Zeilennummern vergibt der Bau
;     (10, 20, 30 …). Die Marken zählen nicht als Variablen.
;   - Kommentarzeilen mit `;` am Zeilenanfang (wie diese) werden beim Bau entfernt und kosten im Programm nichts. Ein `;` mitten in
;     der Zeile ist dagegen kein Kommentar (in PRINT ist es das Anhängezeichen), deshalb stehen die Erklärungen auf eigenen Zeilen.
;   - Steuerzeichen in Anführungszeichen schreibt man in geschweiften Klammern: {clr} = Bildschirm löschen, {wht} = Schriftfarbe weiß.
;   - Namen mit Typzeichen: `zaehler%` ist eine Ganzzahlvariable, `name$` eine Zeichenkette; `feld(…)` ein Feld und `fn quad` eine
;     mit `def fn` definierte Funktion. Schlüsselwörter brauchen Leerzeichen drumherum, damit sie nicht in Namen hineinlaufen.
print "{clr}{wht}lange namen auf dem c64"
; Feld mit den Indizes 0..3; benutzt werden 1..3 (je eine Quadratzahl)
dim feld(3)
; Funktion "Quadrat"; `wert` ist nur der Platzhalter für das Argument
def fn quad(wert) = wert * wert
; zaehler% zählt die ausgegebenen Zeilen, name$ steht vor jeder Zeile; punkte beginnt ohne Zuweisung mit 0
zaehler% = 0 : name$ = "hero"

@neu:
; innere Schleife: Quadrate 1, 4, 9; die Summe wird nie zurückgesetzt und wächst so über die Durchgänge auf 14, 28, 42
for runde = 1 to 3
  feld(runde) = fn quad(runde)
  punkte = punkte + feld(runde)
next runde
zaehler% = zaehler% + 1
print name$; " runde"; zaehler%; " punkte"; punkte
; dreimal wiederholen
if zaehler% < 3 then @neu
@ende:
; Endlosschleife, damit "READY." den Bildschirm nicht überschreibt
goto @ende
