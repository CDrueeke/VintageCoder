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

## Beispiele im Lieferumfang

- `examples/c64/basic_native` – C64, Marken, ohne Zeilennummern.
- `examples/c64/basic_langenamen` – C64 mit `#longnames`.
- `examples/atari/basic_native` – Atari BASIC mit Marken.
- `examples/atari/turbo_basic` – Turbo BASIC XL mit `PROC`, `WHILE`, `REPEAT`.
- Weitere kleine Programme liegen in den Vorlagen (`vcc new … --template basic-hello`) für jedes System.

Siehe auch: [Grafik laden](help:graphics), [Editor](help:editor), [Assembler](help:assembler).
