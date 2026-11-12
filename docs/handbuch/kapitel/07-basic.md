---
id: basic
titel: BASIC in der IDE
reihenfolge: 7
stufe: Einsteiger
zeit: 30 Minuten
siehe_auch: [assembler, editor, bauen-starten]
referenz: [basic, basic-labels, basic-names, basic-dialects, basic-directives, basic-tools]
---

# BASIC in der IDE

**Ziel:** Du schreibst ein BASIC-Programm für deinen Retro-Computer, ohne Zeilennummern zu vergeben. Du springst mit Namen statt mit Nummern, benutzt lange Variablennamen, wählst den richtigen BASIC-Dialekt und verwandelst ein BASIC-Programm bei Bedarf in Assembler.

## Das brauchst du

- Ein Projekt mit der Vorlage **BASIC (ohne Zeilennummern)**, siehe [Dein erstes Projekt](kapitel:erstes-projekt), oder eine neue Datei mit der Endung `.bas` in einem bestehenden Projekt (Seitenleiste **+ ▸ Quelltext …**, Name mit Endung `.bas`).
- Ein BASIC-Projekt hat keinen eigenen Dialekt-Zwang: Das BASIC folgt automatisch dem **Zielsystem** deines Projekts.

## BASIC bleibt BASIC

In VintageCoder schreibst du BASIC **im Originaldialekt des Zielsystems**: BASIC V2 auf C64 und VC-20, BASIC 7.0 auf dem C128, BASIC 3.5 auf dem Plus/4, BASIC 65 auf dem MEGA65, Atari BASIC (oder Turbo BASIC XL) auf dem Atari. Es gibt keine eigene Sprache und keine neuen Befehle. Was der Computer versteht, darfst du schreiben, und das gebaute Programm ist das, was der Computer selbst aus deinem Text gemacht hätte.

Die Erleichterungen betreffen nur das **Drumherum**:

- Du musst **keine Zeilennummern** vergeben.
- Du springst mit **Namen** statt mit Nummern.
- Du darfst **lange Variablennamen** benutzen.
- Du kannst **Sprites, Zeichensätze und Songs** aus den Studios als `DATA`-Zeilen einbinden.

## Schritt für Schritt: Mein erstes BASIC-Programm

1. Öffne in deinem Projekt die BASIC-Datei (bei der Vorlage `src/c64/main.bas`). Ersetze den Inhalt durch:

```basic
; Mein erstes Programm
print "{clr}hallo welt"
summe = 0
for i = 1 to 10
  summe = summe + i
next i
print "summe ="; summe
@ende:
goto @ende
```

2. Drücke **F5**. VintageCoder baut das Programm und startet es im Emulator. Auf dem C64 erscheint „HALLO WELT“ und „SUMME = 55“.

Zeile für Zeile:

- `; Mein erstes Programm` ist ein **Kommentar** für dich. Ein Semikolon am Zeilenanfang macht die Zeile zum Kommentar. Sie kommt nicht ins Programm und kostet weder Speicher noch Zeit.
- `print "{clr}hallo welt"` gibt Text aus. In geschweiften Klammern stehen **Steuerzeichen**: `{clr}` löscht den Bildschirm. Klein geschriebener Text erscheint auf dem C64 als Großschrift; das ist die normale Darstellung.
- `summe = 0` weist einer **Variable** (einem benannten Speicherplatz für eine Zahl) den Wert 0 zu.
- `for i = 1 to 10` … `next i` ist eine Zählschleife: `i` läuft von 1 bis 10. Du darfst die Zeilen dazwischen einrücken; der Übersetzer entfernt die Einrückung.
- `print "summe ="; summe` gibt Text und Zahl zusammen aus.
- `@ende:` ist eine **Sprungmarke**. `goto @ende` springt dorthin; hier springt das Programm auf sich selbst und bleibt stehen, damit die Ausgabe auf dem Bildschirm bleibt.

> **Info:** In den Quelltexten dieses Kapitels sind die Zeilen **ohne Nummer**. Beim Bauen bekommt jede Zeile automatisch die nächste Nummer (10, 20, 30 …); Kommentarzeilen und leere Zeilen bekommen keine. Das gebaute Programm sieht so aus:

```text
10 print "{clr}hallo welt"
20 summe = 0
30 for i = 1 to 10
40 summe = summe + i
50 next i
60 print "summe ="; summe
70 goto 70
```

> **Achtung:** Der Übersetzer prüft **keine Syntax**. Er macht aus deinem Text die Befehle des Dialekts, versteht aber die Sätze nicht. Ein Tippfehler wie `x = 5 +` baut ohne Meldung und der Computer meldet den Fehler erst beim Ausführen (`?SYNTAX ERROR IN 10`). Probiere dein Programm deshalb immer im Emulator aus.

## Sprungmarken statt Zeilennummern

Im klassischen BASIC springst du mit `GOTO 100`. Dann musst du dir Zeilennummern merken und kannst nichts mehr einfügen, ohne Nummern zu ändern. In VintageCoder gibst du der Zielzeile einen **Namen**:

```basic
; Zählt von 1 bis 5, mit Sprungmarke statt Zeilennummer
print "{clr}"
n = 1
@schleife:
print "zahl"; n
n = n + 1
if n <= 5 then goto @schleife
@ende:
goto @ende
```

- Eine Marke beginnt mit `@` und endet in der Definition mit einem **Doppelpunkt** (`@schleife:`). Die Zeile enthält sonst nichts; die Marke gilt für die nächste Anweisung.
- `goto @schleife` (und genauso `gosub @name`, `then @name`) springt zur Marke.
- Beim Bauen ersetzt VintageCoder `@schleife` durch die Zeilennummer. Das gebaute Programm sieht so aus:

```text
10 print "{clr}"
20 n = 1
30 print "zahl"; n
40 n = n + 1
50 if n <= 5 then goto 30
60 goto 60
```

Das Bauprotokoll listet alle Marken mit der Zeile, die sie bekommen haben (`@schleife = Zeile 30`). Willst du doch eine Zeile selbst nummerieren, schreibst du die Nummer wie gewohnt davor; beides lässt sich mischen.

![Ein BASIC-Quelltext ohne Zeilennummern mit der Marke @schleife, daneben das Bauprotokoll mit der Zuordnung „@schleife = Zeile 30“](bild:basic-marken "Marken im Quelltext und ihre Zeilennummern im Bauprotokoll")

> **Tipp:** Der **Inspektor** zeigt im Reiter **Gliederung** bei BASIC die Zeilen, die von `GOTO`, `GOSUB`, `THEN` oder `ON … GOTO` angesprungen werden, und deine Kommentar-Überschriften. Ein Klick springt zur Stelle.

## Lange Variablennamen

Auf dem C64, VC-20, C128, Plus/4 und MEGA65 unterscheidet das BASIC an einem Variablennamen nur die **ersten zwei Zeichen**. Dort sind `punkte` und `punkt` **dieselbe** Variable, nämlich `PU`. Dieser Fehler ist hinterhältig, weil das Programm läuft und nur falsch rechnet. Dazu kommt: Das BASIC erkennt Befehle mitten im Wort. In `total` steckt `TO`, in `score` steckt `OR`, in `form` steckt `FOR`; das Gerät meldet dann `?SYNTAX ERROR`.

Mit der Zeile `#longnames` am Anfang der Datei bekommst du lange, sprechende Namen. Der Übersetzer setzt sie beim Bauen auf Kurznamen um und achtet darauf, dass keine Namen kollidieren und keine Schlüsselwörter entstehen:

```basic
#longnames
; Zwei verschiedene Variablen, obwohl sie gleich beginnen
punkte = 10
punkt = 5
print "{clr}punkte ="; punkte
print "punkt ="; punkt
@ende:
goto @ende
```

Beim Bauen meldet VintageCoder eine Warnung und zeigt die Zuordnung:

```text
Lange Namen → Kurznamen:
  punkte                     →  PU         (Zahl)
  punkt                      →  PN         (Zahl)
```

Die Warnung (W313) sagt nur: Die beiden Namen unterscheiden sich erst nach den ersten zwei Zeichen, und VintageCoder hat sie für dich getrennt. Das gebaute Programm benutzt `PU` und `PN`.

![Das Bauprotokoll mit der Zuordnung „Lange Namen → Kurznamen“ für die Variablen punkte und punkt](bild:basic-langenamen "Aus langen Namen werden beim Bauen eindeutige Kurznamen")

Die Zuordnungstabelle liegt außerdem als Datei im Ordner `build/` (`….vars.txt`). Ein ausführliches Beispiel liefert das Projekt `examples/c64/basic_langenamen`. Beim **Atari** brauchst du `#longnames` nicht: Dort zählen alle Zeichen eines Namens.

> **Tipp:** Hältst du im Editor die Maus über einen langen Namen, zeigt VintageCoder, in welchen Kurznamen er umgesetzt wird.

## Dialekte: Welches BASIC ist es?

Das Zielsystem des Projekts bestimmt den Dialekt:

| System | Dialekt | Programmstart | Ergebnis |
|---|---|---|---|
| C64 | BASIC V2 | `$0801` | `.prg` |
| VC-20 | BASIC V2 | je nach Speicherausbau | `.prg` |
| C128 | BASIC 7.0 | `$1C01` | `.prg` |
| Plus/4 | BASIC 3.5 | `$1001` | `.prg` |
| MEGA65 | BASIC 65 | `$2001` | `.prg` |
| Atari 8-Bit | Atari BASIC oder Turbo BASIC XL | im SAVE-Format | `.bas` |

Pro **Quellgruppe** lässt sich der Dialekt ändern: in den **Projekt-Einstellungen** im Abschnitt **Quellen und Startprogramm**, bei einer BASIC-Gruppe in der Auswahl **BASIC-Dialekt** (Vorgabe: „Standard des Targets“). So wählst du auf dem Atari zum Beispiel Turbo BASIC XL.

> **Achtung:** Wählst du den falschen Dialekt, bleibt der Bau **ohne Meldung**. Ein Befehl, den der Dialekt nicht kennt, wird einfach als Buchstabenfolge übersetzt, also wie ein Variablenname. Ein Beispiel: `GRAPHIC 1,1` baut im C64-Projekt, weil `GRAPHIC` für BASIC V2 kein Befehl ist (andere Dialekte wie BASIC 7.0 kennen ihn). Du merkst es erst im Emulator. Prüfe deshalb, ob die Quellgruppe den richtigen Dialekt hat.

Einige Regeln, die du dir merken solltest:

- **Groß- und Kleinschreibung:** Befehle und Namen darfst du klein, groß oder gemischt schreiben. In Anführungszeichen zählt sie: Auf dem C64 erscheinen **Kleinbuchstaben als Großschrift**, **Großbuchstaben als Grafikzeichen**. Willst du Großschrift auf dem Bildschirm, schreibe den Text **klein**.
- **Umlaute** wie `ö` und `ß` gibt es im Zeichensatz des C64 nicht. `print "größe"` führt zum Fehler B100: *Zeichen 'ö' hat keine PETSCII-Entsprechung*.
- **Steuerzeichen** schreibst du in geschweiften Klammern: `{clr}`, `{home}`, `{wht}` (weiß), `{down*3}` (dreimal Cursor runter). Auch `{$93}` für ein beliebiges Byte geht. Die Namen unterscheiden sich je System; sie stehen in der Hilfeseite [Dialekte und Zielsysteme](hilfe:basic-dialects).
- **Mehrere Anweisungen** in einer Zeile trennt `:`.
- **Das Fragezeichen** `?` ist die Abkürzung für `print`.

## Assets aus den Studios einbinden

Sprites, Zeichensätze, Bildschirme und Songs zeichnest du in den Studios (siehe [Grafik und Klang im Überblick](kapitel:grafik-ueberblick)). Für BASIC gibt es sie als **DATA-Zeilen**: Beim Bauen exportiert VintageCoder jedes Medium nach `build/export/<name>.data.bas`. Du bindest die Datei mit `#include` ein und liest sie mit `READ`:

```
#include "build/export/hero.data.bas"
```

> **Tipp:** Diese Zeile entsteht auch, wenn du das Sprite aus dem Projektbaum in den BASIC-Quelltext ziehst oder **Als Include einfügen** wählst. Ein `.incbin` gibt es in BASIC nicht; dort lehnt VintageCoder das Einbinden von Binärdateien ab (siehe [Der Code-Editor](kapitel:editor)).

Ein vollständiges Beispiel mit Sprite steht in der Hilfeseite [Direktiven, Dateien und Assets](hilfe:basic-directives). Wichtig auf dem C64: Das BASIC V2 liest `DATA` **streng der Reihe nach**. Binde die Dateien deshalb in der Reihenfolge ein, in der du sie liest.

## Schritt für Schritt: BASIC in Assembler umwandeln

BASIC ist bequem, aber langsam. VintageCoder kann ein BASIC-Programm in lesbaren **Assembler-Quelltext** übersetzen (der Übersetzer heißt `basic2asm`). Das Programm läuft danach nicht mehr im BASIC-Interpreter, sondern als Maschinenprogramm: schneller, und du kannst jede Zeile ansehen, anpassen und im Debugger Schritt für Schritt verfolgen.

1. Klicke im Projektbaum mit der rechten Maustaste auf die `.bas`-Datei und wähle **Nach Assembler konvertieren …**. Du erreichst den Dialog auch über das Menü **Datei ▸ BASIC nach Assembler konvertieren …**.
2. Der Dialog fragt nach der **Ausgabedatei** (Vorgabe `<name>_asm.asm`), dem **Zielsystem** und einigen Schaltern. Die Vorgaben passen meist.
3. Bestätige. Neben dem Quelltext entsteht die Assembler-Datei. Sie enthält deine BASIC-Zeilen als Kommentare vor den jeweiligen Anweisungen und bindet eine kleine Laufzeitbibliothek ein.
4. Du kannst die neue Datei wie jeden Assembler-Quelltext bauen und starten.

![Der Dialog „Nach Assembler konvertieren“ mit Ausgabedatei, Zielsystem und den Schaltern; im Hintergrund der Projektbaum mit der .bas-Datei](bild:basic-nach-asm "BASIC in Assembler umwandeln")

Der Konverter wählt die nötige Genauigkeit selbst. Braucht dein Programm nur Ganzzahlen, rechnet es mit 16-Bit-Zahlen (Stufe 1); der Konverter weist darauf hin (Hinweis I110: *Variable A ist im Original eine Fließkommazahl; Stufe 1 rechnet mit 16-Bit-Ganzzahlen*). Das ist ein Hinweis und kein Fehler. Brüche, Dezimalzahlen und Funktionen wie `SIN` führen automatisch zur Fließkomma-Stufe.

Möchtest du, dass immer die konvertierte Fassung gebaut wird, hakst du in den **Projekt-Einstellungen**, Abschnitt **Quellen und Startprogramm**, bei der BASIC-Gruppe **Beim Bauen nach Assembler konvertieren** an. Die `.bas`-Datei bleibt dann die Quelle.

> **Info:** Der Konverter übersetzt nicht alles, vor allem keine Befehle, die ein Dialekt zusätzlich hat (zum Beispiel Sprite-Befehle des C128). Er meldet dann einen Fehler. Auf der Kommandozeile geht es mit `vcc basic2asm programm.bas --target c64 -o programm.asm`. Alle Einzelheiten stehen in der Hilfeseite [Werkzeuge](hilfe:basic-tools).

## Häufige Stolpersteine

- **Der Bau meldet nichts, aber das Programm macht Unsinn oder meldet `?SYNTAX ERROR`.** Der Übersetzer prüft keine Syntax. Prüfe Tippfehler und den Dialekt der Quellgruppe.
- **`print "größe"` bricht ab.** Umlaute gibt es im Zeichensatz des C64 nicht; schreibe `groesse`.
- **Die Variable hat den falschen Wert.** Auf dem C64 zählen nur die ersten zwei Zeichen des Namens. Nimm `#longnames` an den Anfang der Datei.
- **Der Text erscheint in Grafikzeichen.** In Anführungszeichen werden **Großbuchstaben** zu Grafikzeichen. Schreibe Text klein.
- **`#if` oder `#define` in der BASIC-Datei.** Das gibt es im nativen BASIC nicht. Der Übersetzer macht daraus ohne Meldung Müll. Direktiven gibt es nur zwei: `#include` und `#longnames`.
- **Eine Marke wird nicht gefunden.** Marken beginnen mit `@` und enden in der Definition mit `:`; beim Sprung steht `@name` ohne Doppelpunkt. Doppelt vergebene Marken meldet der Übersetzer (Fehler B111).
- **Die Zeile ist zu lang.** Eine Zeile darf nach dem Übersetzen höchstens 250 Byte (Commodore) bzw. 255 Byte (Atari) haben; sonst meldet der Bau B102.

## Wie geht es weiter?

Als Nächstes lernst du, wie du dein Programm [baust und startest](kapitel:bauen-starten) und das Bauprotokoll liest. Wenn dich der Weg über Assembler reizt, lies [Assembler (VCASM) für Einsteiger](kapitel:assembler). Zum Nachschlagen: die Hilfeseiten [BASIC](hilfe:basic), [Sprungmarken und Zeilennummern](hilfe:basic-labels) und [Lange Variablennamen](hilfe:basic-names).
