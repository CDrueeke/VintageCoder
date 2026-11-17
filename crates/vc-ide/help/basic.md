# BASIC

In VintageCoder schreibst du BASIC **im Originaldialekt des Zielsystems**: BASIC V2 auf dem C64 und VC-20, BASIC 7.0 auf dem C128, BASIC 3.5 auf dem Plus/4, BASIC 65 auf dem MEGA65, Atari BASIC (oder Turbo BASIC XL) auf dem Atari. Es gibt keine eigene Sprache und keine Erweiterung der Befehle: Was das System versteht, darfst du schreiben, und das gebaute Programm ist Byte für Byte das, was auch das System selbst aus deinem Text gemacht hätte.

Die Erleichterungen betreffen nur das **Drumherum**. Du musst keine Zeilennummern vergeben, du springst mit Namen statt mit Nummern, du darfst lange Variablennamen benutzen, und du kannst Sprites, Zeichensätze und Songs aus den Studios als `DATA`-Zeilen einbinden.

Dieses Handbuch besteht aus mehreren Seiten:

| Seite | Inhalt |
|---|---|
| BASIC (diese Seite) | Einstieg: dein erstes Programm Zeile für Zeile, Überblick, Grundregeln |
| [Sprungmarken und Zeilennummern](help:basic-labels) | `@name:`, `GOTO @name`, wie die Zeilennummern entstehen |
| [Lange Variablennamen](help:basic-names) | `#longnames`, Kurznamen, Zuordnungstabelle, Fallen |
| [Direktiven, Dateien und Assets](help:basic-directives) | `#include`, Sprites und Songs einbinden, Maschinencode |
| [Dialekte und Zielsysteme](help:basic-dialects) | C64, VC-20, C128, Plus/4, MEGA65, Atari, Turbo BASIC XL |
| [Werkzeuge](help:basic-tools) | F1-Hilfe, Editor, Tokenisieren, `basic2asm`, Fehlersuche |
| [Spickzettel und Fehlercodes](help:basic-index) | alles Wichtige zum Nachschlagen |

Zeilen im Muster `main.bas:3: error B110: …` in den Beispielen sind die echten Meldungen des Übersetzers. Alle Beispiele auf diesen Seiten wurden mit dem Übersetzer gebaut; wo ein Beispiel „Liste“ heißt, ist die gezeigte Liste das, was beim Zurücklesen des gebauten Programms herauskommt.

## Mein erstes BASIC-Programm

Lege im Projektbaum eine Datei `hallo.bas` an (oder starte ein neues Projekt mit der Vorlage „BASIC (ohne Zeilennummern)“) und schreibe:

```basic liste
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

```text
10 print "{clr}hallo welt"
20 summe = 0
30 for i = 1 to 10
40 summe = summe + i
50 next i
60 print "summe ="; summe
70 goto 70
```

Das ist ein vollständiges Programm für den C64. Zeile für Zeile:

- `; Mein erstes Programm` – Ein Semikolon am Zeilenanfang macht die Zeile zum **Kommentar** für dich. Sie kommt nicht ins Programm und kostet weder Speicher noch Zeit. (Das BASIC-Wort `rem` gibt es auch, es bleibt aber im Programm.)
- `print "{clr}hallo welt"` – Gibt Text aus. In geschweiften Klammern stehen **Steuerzeichen**: `{clr}` löscht den Bildschirm. Klein geschriebener Text erscheint auf dem C64 als Großschrift, wie es das Gerät im Normalzustand zeigt (Näheres auf der Seite [Dialekte](help:basic-dialects)).
- `summe = 0` – Eine **Variable** (ein benannter Speicherplatz für eine Zahl) bekommt den Wert 0. Der Name `summe` ist hier **kein** Problem: Der C64 sieht nur die ersten zwei Zeichen, also `SU`. Das ist unproblematisch, solange kein zweiter Name mit `su` beginnt. Mehr dazu auf der Seite [Lange Variablennamen](help:basic-names).
- `for i = 1 to 10` … `next i` – Eine Zählschleife: `i` läuft von 1 bis 10. Die Zeilen dazwischen darfst du einrücken, der Übersetzer entfernt die Einrückung. Auch Leerzeichen um das `=` bleiben, wie du sie schreibst.
- `print "summe ="; summe` – Text und Zahl in einer Ausgabe. Das Semikolon hängt die Zahl direkt an.
- `@ende:` – Eine **Sprungmarke** (englisch *label*): ein Name für die Stelle, an der sie steht. Sie beginnt mit `@` und endet mit einem Doppelpunkt. Die Zeile enthält sonst nichts; die Marke gilt für die nächste Anweisung.
- `goto @ende` – Springt zu dieser Marke. Hier springt das Programm auf sich selbst und bleibt stehen, damit die Ausgabe auf dem Bildschirm bleibt. Beim Bauen ersetzt der Übersetzer `@ende` durch die Zeilennummer (70).

Die gezeigte Liste entsteht so: Jede Zeile **ohne Nummer** bekommt automatisch die nächste Nummer (10, 20, 30 …). Kommentarzeilen und leere Zeilen bekommen keine. Alles weitere dazu steht auf der Seite [Sprungmarken und Zeilennummern](help:basic-labels).

### Bauen und starten

Das Programm wird wie jedes Projekt gebaut und gestartet (siehe [Bauen](help:build) und [Start](help:start)). Die Datei muss im Projekt als Quelle der Art BASIC stehen; die Vorlage macht das schon. Ergebnis des Baus ist bei den Commodore-Systemen eine `.prg`-Datei im Ordner `build`, beim Atari eine `.bas`-Datei im SAVE-Format.

Auf der Kommandozeile geht es auch ohne Projekt:

```bash
vcc basic hallo.bas --target c64 -o hallo.prg
```

## Was gleich bleibt und was anders ist

| Thema | Klassisches BASIC | Hier |
|---|---|---|
| Zeilennummern | Pflicht, jede Zeile | Optional. Zeilen ohne Nummer werden beim Bauen nummeriert. Eigene Nummern darfst du jederzeit dazwischen schreiben. |
| Sprünge | `GOTO 100` | `GOTO @name` oder weiterhin `GOTO 100` |
| Variablennamen | Nur die ersten 2 Zeichen zählen (Commodore) | Mit `#longnames` beliebig lang, der Bau setzt auf Kurznamen um |
| Kommentare | `REM` | `REM` und, am Zeilenanfang, `;` |
| Befehle | Die des Dialekts | Genau die des Dialekts, nichts dazu |
| Dateien einbinden | nicht möglich | `#include "datei.bas"` |
| Bedingte Übersetzung, Konstanten, Makros | nicht vorhanden | **nicht vorhanden** (siehe [Direktiven](help:basic-directives)) |
| Syntaxprüfung | beim Eingeben auf dem Gerät | **keine**: Der Übersetzer macht aus dem Text Token, er prüft nicht, ob der Befehl sinnvoll ist (siehe unten) |

### Der Übersetzer prüft keine Syntax

Das ist die wichtigste Eigenschaft, die man kennen muss. Der Übersetzer erkennt Schlüsselwörter, Zahlen, Zeichenketten und deine Marken, aber er versteht die Sätze nicht. Ein Programm mit Tippfehler baut ohne Meldung, und das Gerät meldet den Fehler erst beim Ausführen:

```basic liste
x = 5 +
print x
```

```text
10 x = 5 +
20 print x
```

Auf dem C64 ergibt Zeile 10 beim Start `?SYNTAX ERROR IN 10`. Dasselbe gilt für Befehle eines anderen Dialekts: `graphic 1,1` baut im C64-Projekt ohne Meldung, weil `GRAPHIC` für den Übersetzer dort einfach drei Variablennamen und kein Befehl ist. Prüfe deshalb, ob die **Quellgruppe den richtigen Dialekt** hat (Seite [Dialekte](help:basic-dialects)), und probiere das Programm im Emulator aus. Nur dort, wo der Übersetzer eindeutig etwas nicht abbilden kann, meldet er einen Fehler: ein Zeichen ohne Entsprechung im Zeichensatz des Systems, eine unbekannte Marke, eine zu lange Zeile.

```basic fehler
print "größe"
```

```text
main.bas:1: error B100: Zeichen 'ö' hat keine PETSCII-Entsprechung
```

## Grundregeln

- **Dateiendungen:** `.bas` und `.lst` werden gleich behandelt (`.lst` für Listings, die du von woanders hast). Beide dürfen Zeilennummern, Marken und Direktiven enthalten.
- **Groß-/Kleinschreibung:** Befehle und Namen darfst du klein, groß oder gemischt schreiben. In Anführungszeichen zählt sie: Auf dem C64 und den anderen Commodore-Geräten werden **Kleinbuchstaben zu Großschrift** auf dem Bildschirm, **Großbuchstaben zu Grafikzeichen**. Willst du Großschrift auf dem Bildschirm, schreibe den Text klein.
- **Fragezeichen:** `?` ist die Abkürzung für `print` (auf allen Systemen).
- **Steuerzeichen** schreibst du in geschweiften Klammern: `{clr}`, `{home}`, `{down*3}`, `{$41}`. Die Namen unterscheiden sich je System (Seite [Dialekte](help:basic-dialects)).
- **Zeilenlänge:** Eine Zeile darf nach dem Tokenisieren höchstens 250 Byte (Commodore) bzw. 255 Byte (Atari) lang sein. Der Bau meldet **B102**, wenn sie länger wird. Das Bildschirm-Limit des Geräts (C64: 80 Zeichen beim Eintippen) gilt für geladene Programme nicht, nur beim späteren Bearbeiten auf dem Gerät.
- **Mehrere Anweisungen** pro Zeile trennt `:` (Atari: ebenfalls `:`).
- **Höchste Zeilennummer:** 63999 (Commodore), 32767 (Atari).

## Hex- und Binärzahlen

Im Quelltext darfst du Zahlen hexadezimal (`$D020`) und binär (`%00001110`) schreiben:

```basic
poke $d020,%00001110
```

Beim Bauen macht VintageCoder daraus die Zahlen, die der Dialekt versteht: `poke 53280,14`. Die Schreibweise im Quelltext ändert das Programm nicht, `$D020` und `53280` ergeben dieselben Bytes.

- **Wo es gilt:** in Ausdrücken, Argumenten und `DATA`-Elementen (`data $ff,%1010`). Text in Anführungszeichen und `rem` bleiben unverändert.
- **Typkennzeichen:** `a$` und `a%` bleiben Variablennamen. Nach einem Schlüsselwort beginnt `$` eine Zahl (`print $ff` gibt 255 aus).
- **Trenner:** `_` darf Ziffern gruppieren (`$d0_20`, `%0000_1110`); ein Schalter dafür fehlt in der Oberfläche noch.
- **Größe:** bis 32 Bit (`$ffffffff`). Adressen über 65535 und `poke`-Werte über 255 meldet der Bau als Warnung (**B134**, **B135**), auf Wunsch als Fehler.
- **Sprungziele:** `goto $0100` geht nicht (**B138**): Zeilennummern entstehen beim Bauen, nimm eine Marke (`goto @name`).
- **Dialekte:** Turbo BASIC XL behält `$` (das Gerät liest es selbst) und wandelt `%` um; BASIC 65 bekommt Dezimalzahlen oder, wenn so eingestellt, die Zahlen wie getippt (diese Einstellung gibt es in der Oberfläche noch nicht). Alle anderen Dialekte bekommen Dezimalzahlen.
- **Fehler:** **B130** ungültige Hexziffer, **B131** ungültige Binärziffer, **B132** `$` oder `%` ohne Ziffern, **B133** über 32 Bit. **B136** warnt, wenn eine Hexzahl an ein Schlüsselwort stößt (`$ffand` – ein Leerzeichen davor), **B137**, wenn `a%=40000` nicht in eine Ganzzahlvariable passt, **B139**, wenn du `0xFF` geschrieben hast.
- **Anzeigen:** Hex, Binär und Dezimal haben im Editor eigene Farben; über einer Zahl zeigt der Hover alle Schreibweisen, die erzeugte Zahl und bei Adressen in `poke`, `peek`, `sys`, `wait` das Chipregister (`vic2.border – Rahmenfarbe`). 

### Zahlen im Editor bearbeiten

Zwei Werkzeuge im Menü **Bearbeiten** (auch in der Befehlspalette, beide ohne Kürzel):

- **Zahlenwerkzeuge …:** ein Dialog mit Umrechnung und Bit-Editor. Gib eine Zahl oder einen Ausdruck ein (`$D020`, `%1010`, `255`, `$D000+$20`), und der Dialog zeigt sie dezimal, hexadezimal und binär, als Lo-/Hi-Byte, als Zeichen und mit Vorzeichen. Der Bit-Editor schaltet einzelne Bits (8 oder 16 Bit) per Klick; **Kopieren** und **Einsetzen** bringen den Wert in die Zwischenablage bzw. an die Schreibmarke im Editor. Siehe [Zahlenwerkzeuge](help:numbers).
- **BASIC: Hex-Vorschläge für Adressen …:** für importierte Programme. Dezimale Adressen, die ein Chipregister des Zielsystems sind, lassen sich als Hex schreiben. Nur vollständige Adressen in `poke`, `peek`, `sys`, `wait`, `usr` werden vorgeschlagen, jede Zeile ist einzeln wählbar (mit **Alle** und **Keine**), ohne deine Auswahl ändert sich nichts. Auf der Kommandozeile: `vcc basic --suggest-hex datei.bas` listet, `--apply 1,3 -o neu.bas` schreibt die gewählten.

Noch nicht in der Oberfläche: das Umwandeln aller Zahlen einer Zeile oder Datei, ein Rechtsklick-Menü „Zahlen“, das Suchen nach dem Zahlenwert (`#` in der Suchleiste) und eine Eingabehilfe nach `$` und `%`.

## Beispiele im Lieferumfang

- `examples/c64/basic_native` – C64, Marken, ohne Zeilennummern.
- `examples/c64/basic_langenamen` – C64 mit `#longnames`.
- `examples/atari/basic_native` – Atari BASIC mit Marken.
- `examples/atari/turbo_basic` – Turbo BASIC XL mit `PROC`, `WHILE`, `REPEAT`.
- Weitere kleine Programme liegen in den Vorlagen (`vcc new … --template basic-hello`) für jedes System.

Siehe auch: [Grafik laden](help:graphics), [Editor](help:editor), [Assembler](help:assembler).
