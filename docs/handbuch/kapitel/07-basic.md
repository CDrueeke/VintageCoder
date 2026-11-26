---
id: basic
titel: BASIC in der IDE
reihenfolge: 7
stufe: Einsteiger
zeit: 30 Minuten
siehe_auch: [assembler, editor, bauen-starten]
referenz: [basic, basic-labels, basic-names, basic-dialects, basic-directives, basic-tools, turbo-disk]
---

# BASIC in der IDE

**Ziel:** Du schreibst ein BASIC-Programm für deinen Retro-Computer, ohne Zeilennummern zu vergeben. Du springst mit Namen statt mit Nummern, benutzt lange Variablennamen, wählst den richtigen BASIC-Dialekt und verwandelst ein BASIC-Programm bei Bedarf in Assembler.

## Das brauchst du

- Ein Projekt mit der Vorlage **BASIC**, siehe [Dein erstes Projekt](kapitel:erstes-projekt), oder eine neue Datei mit der Endung `.bas` in einem bestehenden Projekt (Rechtsklick im Projektbaum oder der Knopf **Neue Datei** über dem Baum, im Dialog „Neue Datei“ die Art **BASIC (.bas)**).
- Ein BASIC-Projekt hat keinen eigenen Dialekt-Zwang: Das BASIC folgt automatisch dem **Zielsystem** deines Projekts.

## BASIC bleibt BASIC

In VintageCoder schreibst du BASIC **im Originaldialekt des Zielsystems**: BASIC V2 auf C64 und VC-20, BASIC 7.0 auf dem C128, BASIC 3.5 auf dem Plus/4, BASIC 65 auf dem MEGA65, Atari BASIC (oder Turbo BASIC XL) auf dem Atari. Es gibt keine eigene Sprache und keine neuen Befehle. Was der Computer versteht, darfst du schreiben, und das gebaute Programm ist das, was der Computer selbst aus deinem Text gemacht hätte.

Die Erleichterungen betreffen nur das **Drumherum**:

- Du musst **keine Zeilennummern** vergeben.
- Du springst mit **Namen** statt mit Nummern.
- Du darfst **lange Variablennamen** benutzen.
- Du kannst **Sprites, Zeichensätze und Songs** aus den Editoren als `DATA`-Zeilen einbinden.
- Du wandelst ein Programm per Dialog zwischen Zeilennummern und Labels um und nummerierst neu.

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

2. Drücke **F5** (Startprogramm starten; ist die BASIC-Datei das Startprogramm des Projekts, wie bei der Vorlage). VintageCoder baut das Programm und startet es im Emulator. Auf dem C64 erscheint „HALLO WELT“ und „SUMME = 55“.

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

Welche Zeile eine Marke bekommen hat (`@schleife = Zeile 30`), siehst du im gebauten Programm; es liegt als Text in `build/<name>.classic.txt`. Willst du doch eine Zeile selbst nummerieren, schreibst du die Nummer wie gewohnt davor; beides lässt sich mischen.

*[Bild folgt in E13: basic-marken]*

Besitzt du schon ein Programm mit Zeilennummern, oder willst du für eine Weitergabe wieder Nummern haben, wandelst du es um. Wie das geht, steht weiter unten im Abschnitt „Umwandeln: Labels, Nummern, neu nummerieren“.

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

Beim Bauen meldet VintageCoder eine Warnung (auf der Seite **Probleme** der Ablage unter dem Editor) und legt die Zuordnung ab:

```text
Lange Namen → Kurznamen:
  punkte                     →  PU         (Zahl)
  punkt                      →  PN         (Zahl)
```

Die Warnung (W313) sagt nur: Die beiden Namen unterscheiden sich erst nach den ersten zwei Zeichen, und VintageCoder hat sie für dich getrennt. Das gebaute Programm benutzt `PU` und `PN`.

*[Bild folgt in E13: basic-langenamen]*

Die Zuordnungstabelle liegt außerdem als Datei im Ordner `build/` (`….vars.txt`). Ein ausführliches Beispiel liefert das Projekt `examples/c64/basic_langenamen`. Beim **Atari** brauchst du `#longnames` nicht: Dort zählen alle Zeichen eines Namens.

## Hex- und Binärzahlen

Zahlen darfst du im Quelltext hexadezimal (`$D020`) und binär (`%00001110`) schreiben: `poke $d020,%00001110`. Beim Bauen entsteht daraus das, was der Dialekt versteht (`poke 53280,14`); die Schreibweise ändert das Programm nicht. In Zeichenketten und nach `rem` bleibt alles, wie es ist; in `data`-Zeilen werden vollständige Zahlen umgewandelt. `a$` und `a%` sind weiter Variablennamen. Als Sprungziel (`goto $0100`) sind sie nicht erlaubt: nimm eine Marke.

Im Editor haben Hex, Binär und Dezimal eigene Farben. Lässt du die Maus kurz über einer Zahl ruhen, zeigt eine Karte alle Schreibweisen und bei Adressen das Chipregister mit Name, Zugriff und Bedeutung der Bits. Steht die Zahl hinter `poke`, zerlegt die Karte auch den Wert in seine Bitfelder. Die Zahlenwerkzeuge (ein Dialog zum Umrechnen mit Bit-Editor) und die Hex-Vorschläge für importierte Programme stehen in dieser Version noch nicht zur Verfügung. Die Hilfeseite [BASIC](hilfe:basic) beschreibt das BASIC im Ganzen.

## Direktiven im BASIC

Eine **Direktive** ist eine Anweisung an den Übersetzer, keine an das Gerät. Sie steht allein in einer Zeile, beginnt mit `#` und hat keine Zeilennummer. Sie kostet weder Speicher noch Laufzeit.

| Zeile | Wirkung |
|---|---|
| `#include "pfad"` | fügt eine Datei oder ein Medium an dieser Stelle ein (siehe unten) |
| `#longnames` | erlaubt lange Variablennamen |
| `#define NAME wert` | ersetzt von da an jedes ganze Wort `NAME` durch `wert` (Konstante ohne Variable, ohne Parameter) |
| `#if SYSTEM_C64` … `#else` … `#endif` | bedingte Übersetzung nach Zielsystem (`SYSTEM_` und die Kennung in Großbuchstaben), nach einem definierten Namen oder mit `!` verneint |
| `#dialect kennung` | wählt den Dialekt dieser Datei |
| `#numbering start=10 step=10` | Start und Schrittweite der Zeilennummern für diese Datei |
| `#asm "datei.asm"` | bindet Maschinencode ein (siehe unten) |

```basic
#define RAHMEN 53280
#if SYSTEM_C64
poke RAHMEN, 2
#endif
```

Nach dem Bauen steht dort `poke 53280, 2`. Andere `#`-Zeilen kennt der Übersetzer nicht; er behandelt sie als gewöhnliche BASIC-Zeile, und das Gerät meldet beim Ausführen einen Syntaxfehler.

## Umwandeln: Labels, Nummern, neu nummerieren

Ein BASIC-Programm hat entweder Zeilennummern oder Labels, und du kannst jederzeit von einem zum anderen wechseln. Drei Befehle erledigen das, jeweils mit einem Dialog und einer Vorschau:

- **BASIC: In Labels umwandeln …** ersetzt Zeilennummern und die Sprungziele dahinter durch Labels (`@name`). Namen, die du selbst vergeben hast, bleiben erhalten. Neue Namen leitet VintageCoder aus Kommentaren ab und kennzeichnet sie mit einer Markerzeile.
- **BASIC: In Nummern umwandeln …** vergibt Zeilennummern nach Start und Schrittweite und ersetzt die Labels in den Sprungzielen durch Nummern. Die Namen bleiben in Markerzeilen (`;@name`) erhalten, damit du später wieder in Labels umwandeln kannst.
- **BASIC: Neu nummerieren …** vergibt die Zeilennummern neu nach Start und Schrittweite und passt alle Sprungziele an. Mit einem Bereich werden nur die Zeilen von einer Nummer bis zu einer anderen neu nummeriert.

Du findest die Befehle im Menü **Bearbeiten ▸ Zahlen und BASIC**, im Rechtsklickmenü einer BASIC-Datei und in der Befehlspalette. Sie arbeiten in einer BASIC-Datei des Projekts; in einem unbenannten Reiter meldet VintageCoder „Das geht nur in einer BASIC-Datei des Projekts.“

### Schritt für Schritt: Ein Programm in Nummern umwandeln

1. Öffne die `.bas`-Datei und wähle **Bearbeiten ▸ Zahlen und BASIC ▸ In Nummern umwandeln …**. Der Dialog öffnet sich und rechnet gleich eine Vorschau.
2. Oben steht eine kurze Erklärung und der **Modus** des Programms: Zeilennummern, Labels, gemischt oder noch unbestimmt. Daneben stehen Start und Schrittweite, zum Beispiel „Start 10, Schrittweite 10“.
3. Bindet das Programm andere BASIC-Dateien ein, gibt es das Kästchen **Eingebundene Dateien mitumwandeln** mit der Zahl der Dateien. Eingebundene Medien zeigt der Dialog nur an („werden nicht verändert“).
4. Bei **Neu nummerieren** kannst du unter **Bereich** die Zeilen **Von** und **Bis** angeben. Lässt du beide Felder leer, wird das ganze Programm neu nummeriert. Gibst du nur eines an, bittet der Dialog dich, beide auszufüllen.
5. Unter **Meldungen** stehen Warnungen und Fehler des Umwandlers mit Code und Zeile. Die **Vorschau** listet jede geänderte Zeile mit alter und neuer Fassung und fasst zusammen: „n Zeilen ändern sich, k neue Labels“, oder „Nichts zu ändern.“
6. Klicke auf **Anwenden**. Die Änderung ist ein einziger Schritt für Rückgängig (Cmd/Strg+Z) in dieser Datei. Mit **Abbrechen** schließt du den Dialog, ohne etwas zu ändern.

![Der Reiter „Code“ mit test.bas und darüber der Dialog „In Nummern umwandeln“. Er erklärt den Vorgang, nennt Modus „Labels“, Start 10 und Schrittweite 10 und zeigt eine Vorschau mit sieben Zeilen, links alt, rechts neu. Unten die Knöpfe „Abbrechen“ und „Anwenden“.](bild:basic-umwandeln-dialog "Der Dialog „In Nummern umwandeln“ mit Vorschau der geänderten Zeilen.")

> **Achtung:** Hat eine eingebundene Datei, die mitumgewandelt wird, ungespeicherte Änderungen, bricht VintageCoder ab und nennt die Datei: Speichere sie zuerst, oder schalte **Eingebundene Dateien mitumwandeln** aus. Eingebundene Dateien schreibt VintageCoder direkt; ihr Rückgängig gilt dann nicht über Reiter hinweg. Ändert sich der Text, während der Dialog offen ist, rechnet er die Vorschau neu.

Start und Schrittweite für das Bauen stellst du auf der Seite **Quellen** im Bereich **Projekt** in der Karte **BASIC** ein (Abschnitt **Zeilennummern**, Knopf **Speichern**). Dort liegen auch **Kompakt bauen**, die **Syntaxprüfung beim Bauen** und die **abgeschalteten Meldungen**. Eine Zeile `#numbering start=… step=…` in der Datei geht diesen Werten vor.

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

Pro **Quellgruppe** lässt sich der Dialekt ändern. Er steht in der Projektdatei (`.vcprj`), bei der Gruppe im Eintrag `dialect`; ohne Eintrag gilt der Standard des Zielsystems. So wählst du auf dem Atari zum Beispiel Turbo BASIC XL:

```text
{ "targets": ["atari8"], "kind": "basic", "dialect": "turbo_basic_xl", "files": ["balken.tur"] }
```

Die Seite **Quellen** im Bereich **Projekt** zeigt den Dialekt jeder BASIC-Gruppe in ihrer Überschrift; eine Auswahl zum Ändern gibt es dort in dieser Version nicht, du bearbeitest die Projektdatei als Text. Für eine einzelne Datei gilt außerdem eine Zeile `#dialect <kennung>` am Anfang der Datei.

> **Achtung:** Wählst du den falschen Dialekt, bleibt der Bau **ohne Meldung**. Ein Befehl, den der Dialekt nicht kennt, wird einfach als Buchstabenfolge übersetzt, also wie ein Variablenname. Ein Beispiel: `GRAPHIC 1,1` baut im C64-Projekt, weil `GRAPHIC` für BASIC V2 kein Befehl ist (andere Dialekte wie BASIC 7.0 kennen ihn). Du merkst es erst im Emulator. Prüfe deshalb, ob die Quellgruppe den richtigen Dialekt hat (Seite **Quellen** im Bereich **Projekt**).

Einige Regeln, die du dir merken solltest:

- **Groß- und Kleinschreibung:** Befehle und Namen darfst du klein, groß oder gemischt schreiben. In Anführungszeichen zählt sie: Auf dem C64 erscheinen **Kleinbuchstaben als Großschrift**, **Großbuchstaben als Grafikzeichen**. Willst du Großschrift auf dem Bildschirm, schreibe den Text **klein**.
- **Umlaute** wie `ö` und `ß` gibt es im Zeichensatz des C64 nicht. `print "größe"` führt zum Fehler B100: *Zeichen 'ö' hat keine PETSCII-Entsprechung*.
- **Steuerzeichen** schreibst du in geschweiften Klammern: `{clr}`, `{home}`, `{wht}` (weiß), `{down*3}` (dreimal Cursor runter). Auch `{$93}` für ein beliebiges Byte geht. Die Namen unterscheiden sich je System; sie stehen in der Hilfeseite [Dialekte und Zielsysteme](hilfe:basic-dialects).
- **Mehrere Anweisungen** in einer Zeile trennt `:`.
- **Das Fragezeichen** `?` ist die Abkürzung für `print`.

## Turbo BASIC XL starten

Turbo BASIC XL läuft auf dem Atari von einer **Diskette**, die du selbst besitzen musst. Turbo BASIC XL ist kein Bestandteil von VintageCoder und muss vom Anwender beigesteuert werden. Wähle hier eine Diskette aus einer Quelle, die du verwenden darfst. Es gibt keine Telemetrie: Die Diskette wird nirgends hin übertragen und liegt nur lokal.

### Schritt für Schritt: Diskette bereitstellen und starten

1. Öffne ein Projekt mit dem BASIC-Dialekt **Turbo BASIC XL** (zum Beispiel das Beispiel `examples/atari/turbo_basic`).
2. Drücke **F5**. Ohne Diskette erscheint über dem Editor das Banner „Dieses Projekt braucht eine Turbo-BASIC-XL-Diskette. Sie gehört nicht zum Lieferumfang.“ Klicke auf **Bereitstellen …**. Dasselbe erreichst du über **Projekt ▸ Turbo-BASIC-Diskette bereitstellen …**, über die Karte **Turbo BASIC XL** auf der Seite **Diskette** im Bereich **Projekt** oder über die Befehlspalette.
3. Im Assistenten klickst du bei Schritt **1 · Datei wählen** auf **Datei wählen …** und suchst deine Diskette (ein Diskettenabbild, `.atr`). Arbeite am besten mit einer **Kopie**; das Original bleibt auf jeden Fall unverändert.
4. Schritt **2 · Prüfung** zeigt zum Beispiel „Turbo BASIC XL 1.5 erkannt“ und die freien Sektoren. Sieht die Datei nicht nach Turbo BASIC XL aus, fragt der Assistent „Sieht nicht nach Turbo BASIC XL aus. Trotzdem verwenden?“ Du wählst **Ja, trotzdem verwenden** oder **Nein, andere Datei wählen**.
5. Bei Schritt **3 · Kopieren** entscheidest du über das **Verkleinern**: **Ja, in der Projektkopie verkleinern** oder **Nicht verkleinern**. Auf der Originaldiskette sind nur etwa 12,9 KB frei; verkleinert (nur die Kopie im Projekt, auf `DOS.SYS`, `DUP.SYS`, `AUTORUN.SYS`) sind es etwa 60 KB. Zu große Programme melden beim Nichtverkleinern den Fehler erst beim Bauen.
6. Klicke auf **Kopieren**. Die Kopie liegt dann in `turbo/`, der Projektbaum markiert sie mit „Diskette (Anwender)“, und `turbo/*.atr` steht in der `.gitignore`. Das Fenster meldet „Fertig“ mit der Zahl der freien Sektoren.
7. Drücke **F5**. VintageCoder baut `build/<name>.atr` (dein Programm als `AUTORUN.BAS`) und bootet sie im Emulator atari800. Am Programmende zeigt Turbo BASIC XL `READY` und wartet auf Eingaben.

*[Bild folgt in E13: turbo-diskette-assistent]*

Die Karte **Turbo BASIC XL** (Bereich **Projekt**, Seite **Diskette**) zeigt den Zustand („Diskette bereit“ oder „Diskette fehlt“), die Kopie im Projekt, die freien Sektoren und die Dateien auf der Diskette. Mit den Knöpfen **Ersetzen …**, **Prüfen**, **Verkleinern …** und **Im Disk-Editor öffnen** pflegst du die Kopie. Das Feld **Programmname auf der Diskette** ist ein Name wie `AUTORUN.BAS`; nur diesen Namen startet Turbo BASIC XL von selbst. Unter **Zusatzdateien auf der Diskette** trägst du je Zeile eine Quelle aus dem Projekt ein, optional mit Namen auf der Diskette (`daten/hallo.dat:HALLO.DAT`). **Speichern** übernimmt die Änderungen. Die Statuszeile zeigt „Turbo-Diskette: …“ mit den freien Sektoren.

> **Info:** **Debuggen** ist für Turbo-BASIC-Programme noch nicht möglich; der Befehl meldet das. Mehr, auch zu Platz, Zusatzdateien und Fehlerbildern, steht in der Hilfeseite [Turbo BASIC XL: Diskette](hilfe:turbo-disk).

## Medien aus den Editoren einbinden

Sprites, Zeichensätze, Bildschirme und Songs zeichnest du in den Editoren (siehe [Grafik und Klang im Überblick](kapitel:grafik-ueberblick)). Für BASIC bindest du die **Mediendatei** selbst ein: VintageCoder macht daraus beim Bauen `DATA`-Zeilen, die du mit `READ` liest.

```
#include "graphics/hero.vcsprite"
```

- Die erzeugten Zeilen beginnen mit einer Marke `@name:` (der Name des Mediums). Besteht ein Medium aus mehreren Teilen, hat jeder Teil seine Marke.
- Die Zeilen haben keine eigenen Nummern; sie werden an der Stelle der `#include`-Zeile mitnummeriert.
- Ein Medium wird je Programm nur einmal eingebunden; ein zweites `#include` desselben Mediums überspringt VintageCoder ohne Meldung.
- Findet der Übersetzer das Medium nicht oder lässt es sich nicht als `DATA` ausgeben, meldet er **B193**. Ein alter Pfad `build/export/…` läuft weiter, wird aber mit **B194** beanstandet. Wird das Programm samt Daten größer als der freie Speicher, meldet er **B195** (C64: 38911 Byte, VC-20 ohne Erweiterung: 3583 Byte, Plus/4: 60671 Byte).
- Beim Umwandeln (siehe oben) liest VintageCoder eingebundene Medien mit, schreibt sie aber nie zurück.
- Ein `.incbin` gibt es in BASIC nicht; Binärdateien lehnt VintageCoder dort ab.

Ein vollständiges Beispiel mit Sprite steht in der Hilfeseite [Direktiven, Dateien und Assets](hilfe:basic-directives). Wichtig auf dem C64: Das BASIC V2 liest `DATA` **streng der Reihe nach**. Binde die Medien deshalb in der Reihenfolge ein, in der du sie liest. Mehrere Medien zusammen mit `RESTORE @name` meldet VintageCoder auf dem C64 und VC-20 mit **B196**; auf C128, Plus/4, MEGA65 und Atari geht `RESTORE @name`.

## Maschinencode im BASIC: `#asm`

Willst du den schnellen Teil eines Programms in Assembler schreiben, bindest du ihn mit `#asm` ein. Der Assembler übersetzt die Datei beim Bauen, und du rufst den Code über seine Marken auf:

```basic
#asm "player.asm"
sys asm.start
```

- `#asm "datei.asm"` nennt die Assembler-Datei, optional mit `with "song.asm"` für einen Song dazu (auch ein Medium wie `music/tune.vcsong` ist möglich).
- Jede Marke des Codes steht im BASIC als `asm.name` zur Verfügung. Beim Bauen ersetzt VintageCoder sie durch die Adresse, zum Beispiel `sys asm.start`. Zusätzlich gibt es `asm.base_addr` (Anfang des Codes) und `asm.base_page` (die Seite, für `POKE 106`).
- Auf **Commodore-Systemen** liegt der Code **hinter dem BASIC-Text** im Programm; er beginnt mit `.org asm_base`. Auf **MEGA65 und Atari** liefert VintageCoder den Code als `DATA`-Zeilen ab der Marke `@asm_data`, die das Programm selbst an die feste Adresse lädt; auf dem Atari liegt sie unter `$A000` an einer Seitengrenze.
- Die frühere Schreibweise `#blob` gibt es nicht mehr. Der Übersetzer meldet **B177** mit dem Hinweis, dass es jetzt `#asm` heißt.

Der Weg ist gedacht für ein Programm, das im Kern BASIC bleibt. Willst du das ganze Programm als Maschinencode, wandelst du es um (nächster Abschnitt).

## BASIC in Assembler umwandeln

BASIC ist bequem, aber langsam. VintageCoder kann ein BASIC-Programm in lesbaren **Assembler-Quelltext** übersetzen (der Übersetzer heißt `basic2asm`). Das Programm läuft danach nicht mehr im BASIC-Interpreter, sondern als Maschinenprogramm: schneller, und du kannst jede Zeile ansehen und anpassen.

Der Befehl **Datei ▸ BASIC in Assembler umwandeln** steht in der Oberfläche in dieser Version noch nicht zur Verfügung. Du hast zwei andere Wege:

- **Beim Bauen.** Du trägst in der Projektdatei (`.vcprj`) bei der BASIC-Quellgruppe einen Eintrag `convert` ein. Dann übersetzt VintageCoder die Quelle bei jedem Bauen zuerst nach Assembler und baut das Ergebnis; die `.bas`-Datei bleibt die Quelle:

```text
{ "targets": ["c64"], "kind": "basic", "convert": { "target": "c64" }, "files": ["main.bas"] }
```

  Ohne weitere Angaben legt der Übersetzer die Datei `<name>_asm.asm` neben der Quelle ab.
- **Auf der Kommandozeile.** `vcc basic2asm programm.bas --target c64 -o programm.asm` erzeugt die Assembler-Datei, die du dann wie jeden Quelltext baust. Alle Schalter stehen in der Hilfeseite [Werkzeuge](hilfe:basic-tools).

Der Konverter wählt die nötige Genauigkeit selbst. Braucht dein Programm nur Ganzzahlen, rechnet es mit 16-Bit-Zahlen (Stufe 1); der Konverter weist darauf hin (Hinweis I110: *Variable A ist im Original eine Fließkommazahl; Stufe 1 rechnet mit 16-Bit-Ganzzahlen*). Das ist ein Hinweis und kein Fehler. Brüche, Dezimalzahlen und Funktionen wie `SIN` führen automatisch zur Fließkomma-Stufe.

> **Info:** Der Konverter übersetzt nicht alles, vor allem keine Befehle, die ein Dialekt zusätzlich hat (zum Beispiel Sprite-Befehle des C128). Er meldet dann einen Fehler.

## Häufige Stolpersteine

- **Der Bau meldet nichts, aber das Programm macht Unsinn oder meldet `?SYNTAX ERROR`.** Der Übersetzer prüft keine Syntax. Prüfe Tippfehler und den Dialekt der Quellgruppe.
- **`print "größe"` bricht ab.** Umlaute gibt es im Zeichensatz des C64 nicht; schreibe `groesse`.
- **Die Variable hat den falschen Wert.** Auf dem C64 zählen nur die ersten zwei Zeichen des Namens. Nimm `#longnames` an den Anfang der Datei.
- **Der Text erscheint in Grafikzeichen.** In Anführungszeichen werden **Großbuchstaben** zu Grafikzeichen. Schreibe Text klein.
- **Eine unbekannte `#`-Zeile.** Der Übersetzer kennt nur die Direktiven aus der Tabelle weiter oben. Alles andere (zum Beispiel `#blob`, `#ifdef`) behandelt er als gewöhnliche BASIC-Zeile, und das Gerät meldet beim Ausführen einen Syntaxfehler.
- **Der Dialog „Umwandeln“ weigert sich.** Prüfe, ob der Reiter eine BASIC-Datei des Projekts zeigt und ob eingebundene Dateien ungespeicherte Änderungen haben.
- **Eine Marke wird nicht gefunden.** Marken beginnen mit `@` und enden in der Definition mit `:`; beim Sprung steht `@name` ohne Doppelpunkt. Doppelt vergebene Marken meldet der Übersetzer (Fehler B111).
- **Die Zeile ist zu lang.** Eine Zeile darf nach dem Übersetzen höchstens 250 Byte (Commodore) bzw. 255 Byte (Atari) haben; sonst meldet der Bau B102.

## Wie geht es weiter?

Als Nächstes lernst du, wie du dein Programm [baust und startest](kapitel:bauen-starten) und die Meldungen liest. Wenn dich der Weg über Assembler reizt, lies [Assembler (VCASM) für Einsteiger](kapitel:assembler). Zum Nachschlagen: die Hilfeseiten [BASIC](hilfe:basic), [Sprungmarken und Zeilennummern](hilfe:basic-labels) und [Lange Variablennamen](hilfe:basic-names).
