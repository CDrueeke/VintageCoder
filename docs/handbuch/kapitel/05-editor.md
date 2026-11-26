---
id: editor
titel: Der Code-Editor
reihenfolge: 5
stufe: Einsteiger
zeit: 25 Minuten
siehe_auch: [assembler, basic, projekte, debugger]
referenz: [editor, shortcuts, asm-text]
---

# Der Code-Editor

**Ziel:** Du schreibst Quelltext im Editor von VintageCoder zügig und mit Hilfe: Du arbeitest mit mehreren Reitern und einer geteilten Ansicht, nutzt die Vervollständigung, faltest lange Abschnitte zusammen, suchst und ersetzt Text, springst zu einer Zeile oder Definition, setzt Lesezeichen und erkennst Fehler schon beim Tippen.

## Das brauchst du

- Ein geöffnetes Projekt mit einer `.asm`-Datei. Nimm dafür das Programm aus [Dein erstes Projekt](kapitel:erstes-projekt) oder lege eine neue Datei an. Für die Übungen kannst du diesen kleinen Quelltext verwenden (er baut ohne Fehler):

```asm
; Rahmen und Hintergrund färben
.target c64
.basicstub start

.const FARBE = 6            ; Konstante: 6 = Blau

start:  lda #FARBE
        sta vic2.border     ; Chip-Register als Name
        sta $d021           ; dieselbe Wirkung, aber mit Adresse
        rts
```

## Der Editor im Überblick

Den Editor findest du im Hauptbereich **Code**. Oben liegt die **Reiterleiste** mit einem Reiter je geöffneter Datei, darunter der Text mit den Zeilennummern. Links steht die Seitenleiste mit dem Projektbaum, unter dem Text die **Ablage** mit den Seiten Probleme, Ausgabe, Tests, Suche und Gliederung. Die Statuszeile unten nennt die Art der Datei (Assembler, BASIC, Text oder Markdown), das Zeilenende (LF oder CRLF), den Zustand der Probleme sowie Zeile und Spalte des Cursors. Bei laufender Debug-Sitzung steht dort auch der Programmzähler.

![Der Code-Bereich mit der Datei jumprun.asm: links die Seitenleiste mit dem Projektbaum, in der Mitte der Assembler-Quelltext mit Zeilennummern und farbiger Hervorhebung für Kommentare, Befehle, Zahlen und Namen. Unten die Statusleiste mit Zeile und Spalte.](bild:editor-syntax "Der Editor färbt Befehle, Zahlen, Namen und Kommentare unterschiedlich.")

Der Editor färbt **Assembler** und **BASIC** ein. Befehle, Direktiven (die Anweisungen an den Assembler mit einem Punkt davor, wie `.target`), Zahlen, Zeichenketten, Labels und Kommentare haben verschiedene Farben. So erkennst du auf einen Blick, ob ein Wort als das gilt, was du gemeint hast. Texte bleiben ungefärbt; Markdown-Dateien siehst du als Vorschau.

Neben den Zeilennummern liegt ein schmaler Streifen, der **Rand**. Dort siehst du Haltepunkte, Lesezeichen, Fehlermarken und die Pfeile zum Falten. Hast du das Programm gebaut, steht im Rand außerdem die **Zyklenzahl** je Zeile (wie viele Taktzyklen der Befehl braucht).

## Mit Reitern arbeiten

Jede geöffnete Datei hat einen Reiter. Ein Punkt am Reiter heißt: Die Datei hat ungespeicherte Änderungen.

- **Öffnen:** Klicke eine Datei im Projektbaum an, oder wähle **Datei ▸ Datei öffnen …** (Cmd/Strg+O). Eine Datei, die kein Text ist, öffnet VintageCoder im Standardprogramm deines Systems.
- **Neuer Reiter ohne Datei:** **Neuer Reiter (unbenannt)** mit Cmd/Strg+N. Beim Speichern fragt VintageCoder nach einem Namen. **Neue Notiz** (Cmd/Strg+Umschalt+T) legt einen Reiter für Markdown-Text an.
- **Wechseln:** Mit Cmd/Strg+Alt+Rechts und Cmd/Strg+Alt+Links springst du zum nächsten und zum vorherigen Reiter. Mit den Pfeiltasten wechselst du auch, wenn die Reiterleiste den Fokus hat.
- **Ordnen:** Ziehe einen Reiter mit der Maus an eine andere Stelle, oder nimm im Menü **Ansicht ▸ Reiter und Teilen** die Befehle **Reiter nach links verschieben** und **Reiter nach rechts verschieben**.
- **Schließen:** Cmd/Strg+W, das kleine ×, der Mittelklick oder die Entf-Taste auf dem Reiter. Hat die Datei ungespeicherte Änderungen, fragt VintageCoder, ob er speichern oder verwerfen soll.
- **Speichern:** Cmd/Strg+S speichert die Datei im Reiter, Cmd/Strg+Alt+S speichert alle.
- **Rechtsklick auf einen Reiter** öffnet ein Menü mit Schließen, Andere schließen, Alle schließen, Nach links, Nach rechts, Pfad kopieren und Im Ordner zeigen.

Ändert ein anderes Programm eine offene Datei, liest VintageCoder sie neu ein und sagt es dir. Hast du in der Datei selbst ungespeicherte Änderungen, bekommst du stattdessen eine Warnung: Beim Speichern würde die Datei überschrieben. Die offenen Reiter merkt sich VintageCoder pro Projekt und stellt sie beim nächsten Öffnen wieder her.

Willst du nur schreiben und nichts anderes sehen, schaltest du mit Cmd/Strg+Alt+Z den **Zen-Modus** ein: Es bleiben Reiter und Text. Derselbe Befehl bringt alles zurück.

## Schritt für Schritt: Mit Vervollständigung schreiben

Beim Tippen schlägt der Editor Wörter vor: Befehle, Direktiven, Chip-Register (zum Beispiel `vic2.border`), Labels, Konstanten und Makros, auch aus eingebundenen Dateien. Das gilt für Assembler-Dateien.

1. Setze den Cursor in eine neue Zeile und tippe `st`. Nach einem kurzen Moment erscheint eine Liste mit Vorschlägen. (In `vic2.` zählt der Punkt mit: Die Liste zeigt dann gleich die Register des Chips.)
2. Mit den Pfeiltasten **↑ ↓** wählst du einen Eintrag aus.
3. **Tab** oder **Enter** übernimmt den markierten Eintrag. **Esc** schließt die Liste, ohne etwas einzufügen.
4. Ohne zu tippen öffnest du die Liste mit dem Befehl **Vervollständigung öffnen** (Menü **Bearbeiten**, Befehlspalette). Er hat bewusst kein Tastenkürzel.

![Der Code-Editor mit der Dateiliste links und dem Quelltext jumprun.asm. In der letzten Zeile steht „lda“ und darunter ist eine Vorschlagsliste offen mit „lda“ (Akkumulator laden), Konstanten wie TILE_CYC_LDA und Makros wie tile_draw.](bild:editor-vervollstaendigung "Die Vorschlagsliste erscheint beim Tippen.")

> **Tipp:** Probiere es mit `sta vic2.` : nach dem Punkt zeigt die Liste alle Register des VIC-II, des Grafikchips des C64, zum Beispiel `border` (Rahmenfarbe), `bg0` (Hintergrundfarbe) und `raster` (aktuelle Rasterzeile).

Tippst du in einer Zeichenkette ein `{`, bietet der Editor die Namen der Steuerzeichen und Farben an, zum Beispiel `{clr}` oder `{red}`. Die Schreibweise `{name}` in Texten beschreibt die Hilfeseite [Texte und Zeichensätze](hilfe:asm-text). Ein eigenes Zeichen-Panel zum Einfügen von Sonderzeichen gibt es in dieser Version noch nicht.

## Hinweise beim Daraufzeigen

Lässt du die Maus kurz über einem Wort ruhen, erscheint eine Karte mit einer Beschreibung: bei einem Befehl die Adressierungsarten mit Opcode, Bytes und Zyklen, bei einer Direktive die Schreibweise, bei einem Chip-Register die Adresse und die Bedeutung. Über einer Zahl zeigt die Karte die Werte in anderen Schreibweisen (dezimal, hexadezimal, binär), im BASIC bei Adressen auch, was an dieser Adresse im Rechner liegt. In Zeichenketten zeigt sie Code und Wirkung eines Steuerzeichens.

Mit **F1** (Befehl **Hilfe**) schlägst du das Wort unter dem Cursor nach, oder den markierten Text. Es öffnet sich die Hilfe mit dem passenden Thema. Gibt es zu dem Wort keinen Eintrag, steht dort „Zu … gibt es keinen Hilfeeintrag.“ und du kannst in der Hilfe suchen. Ohne Wort öffnet F1 die Hilfe mit der Themenliste.

## Schritt für Schritt: Abschnitte falten

Lange Quelltexte werden unübersichtlich. Mit dem **Falten** blendest du Abschnitte aus.

1. Neben der Zeilennummer erscheinen bei faltbaren Blöcken kleine Pfeile. Gefaltet werden Blöcke wie `.macro`, `.if`, `.repeat`, Label-Abschnitte und Kommentarblöcke.
2. Klicke auf den Pfeil neben dem Blockanfang, um den Abschnitt einzuklappen. Seine Zeilen sind dann versteckt, und an der Stelle steht ein Platzhalter. Ein Klick auf den Pfeil oder den Platzhalter klappt ihn wieder auf.
3. **Alles falten** und **Alles entfalten** (Menü **Bearbeiten ▸ Falten**, Befehlspalette und Rechtsklick im Editor) klappen alle Abschnitte auf einmal zu oder auf; sie haben kein Tastenkürzel.

*[Bild folgt in E13: editor-falten]*

## Schritt für Schritt: Suchen und Ersetzen

1. Drücke **Cmd/Strg+F** (Menü **Bearbeiten ▸ Suchen und Ersetzen ▸ Suchen …**). Über dem Text erscheint die Suchleiste.
2. Tippe den Suchbegriff. Die Treffer sind im Text hervorgehoben. **Enter** springt zum nächsten Treffer, **Umschalt+Enter** zum vorigen; dafür gibt es auch die Knöpfe **Weiter** und **Zurück**. **Esc** schließt die Leiste.
3. Mit den Schaltern **Groß/Klein**, **Ganzes Wort** und **Regulärer Ausdruck** engst du die Suche ein.
4. Zum **Ersetzen** drückst du **Cmd/Strg+Alt+F** (Menü **Bearbeiten ▸ Suchen und Ersetzen ▸ Ersetzen …**). Die Leiste hat das zweite Feld **Ersetzen durch**. **Ersetzen** tauscht den aktuellen Treffer, **Alle ersetzen** tauscht alle.
5. Hast du dich verklickt, hilft **Cmd/Strg+Z** (Rückgängig); **Cmd/Strg+Umschalt+Z** wiederholt.

![Die Code-Ansicht mit jumprun.asm und geöffneter Suchleiste über dem Text. Das Suchfeld enthält „jsr“, daneben stehen „Weiter“, „Zurück“, „Alle“ und die Optionen „Groß/Klein“, „Regulärer Ausdruck“ und „Ganzes Wort“. Darunter liegt das Feld „Ersetzen durch“ mit „Ersetzen“ und „Alle ersetzen“.](bild:editor-suchen "Die Suchleiste mit Ersetzen über dem Quelltext.")

Möchtest du in **allen** Dateien des Projekts suchen? Drücke **Cmd/Strg+Umschalt+F**. In der Ablage öffnet sich die Seite **Suche** („In allen Dateien suchen …“). Die Treffer erscheinen nach Datei gruppiert, ein Klick springt zur Zeile im Editor.

## Schritt für Schritt: Springen

Eine Fehlermeldung nennt oft eine Zeilennummer, zum Beispiel „Zeile 42“.

1. Drücke **Cmd/Strg+L** (Menü **Bearbeiten ▸ Suchen und Ersetzen ▸ Gehe zu Zeile …**). Ein kleiner Dialog fragt nach der Zeilennummer.
2. Tippe die Zahl und bestätige mit Enter (**Gehe zu**). Der Cursor springt dorthin.

Zu einer **Definition** springst du mit **F12** (Rechtsklick ▸ **Gehe zu Definition**), wenn der Cursor auf einem Symbol steht, zum Beispiel einem Label oder einer Konstante; die Definition darf auch in einer anderen Datei liegen. Das gilt nur für Assembler-Dateien. Eine eingebundene Datei öffnest du im Projektbaum.

Die Ablage hat außerdem die Seite **Gliederung**: Sie listet Marken, Konstanten und Makros der Assembler-Datei im Reiter, ein Klick springt zur Stelle. Der Befehl **Zur passenden Klammer springen** (Menü **Bearbeiten ▸ Suchen und Ersetzen**) hat kein Kürzel.

## Lesezeichen und Haltepunkte

Beides setzt du im Rand:

- **Haltepunkt:** Ein Klick in den Rand setzt einen roten Punkt, ein zweiter Klick nimmt ihn weg. Dasselbe tut **Haltepunkt umschalten** (F9). Der Haltepunkt wirkt, wenn du das Programm mit dem Debugger startest (siehe [Der Debugger](kapitel:debugger)).
- **Lesezeichen:** Ein Klick mit gedrückter Umschalt-Taste setzt eine Marke, oder du drückst **Cmd/Strg+Alt+K** (Lesezeichen umschalten). **Cmd/Strg+Alt+J** springt zum nächsten, **Cmd/Strg+Alt+H** zum vorigen Lesezeichen, rundherum. Die Befehle stehen im Menü **Bearbeiten ▸ Suchen und Ersetzen**.

Haltepunkte und Lesezeichen wandern mit dem Text mit, wenn du Zeilen davor einfügst oder löschst. Schließt du den Reiter, gehen sie verloren.

## Formatieren und Aufräumen

Verlässt du eine **bearbeitete** Zeile (der Cursor wechselt in eine andere Zeile, Enter, Mausklick), richtet VintageCoder sie aus. Beim Assembler steht ein Label in Spalte 0, der Befehl danach, und Kommentare sitzen in einer Spalte. Bei BASIC gibt es ein Leerzeichen hinter der Zeilennummer. Gefällt dir eine Ausrichtung nicht, machst du sie mit **Cmd/Strg+Z** rückgängig wie jede andere Eingabe. Abstellen kannst du das Ausrichten unter **Einstellungen ▸ Editor** mit dem Schalter **Zeile beim Verlassen formatieren**.

Auf Befehl formatierst du auch von Hand. Im Menü **Bearbeiten ▸ Formatieren** (und in der Befehlspalette) stehen:

- **Zeile formatieren** (die Zeile oder die markierten Zeilen) und **Datei formatieren** (die ganze Datei). Beide lassen sich in einem Schritt rückgängig machen.
- **Leerzeichen am Zeilenende entfernen** (Auswahl oder ganze Datei).
- **Zeilen sortieren** (mehrere Zeilen markieren).
- **Kommentar umschalten** (Cmd/Strg+Alt+C) setzt oder entfernt das Kommentarzeichen vor den markierten Zeilen.

Formatieren geht nur in Assembler- und BASIC-Reitern; sonst meldet VintageCoder, dass der Reiter nicht formatiert werden kann.

## Art, Zeilenende und Umbruch

Welche Sprache ein Reiter hat, bestimmt die Dateiendung. Willst du sie ändern (zum Beispiel bei einem unbenannten Reiter), wähle **Bearbeiten ▸ Art und Kodierung ▸ Art: Text**, **Art: Assembler** oder **Art: BASIC**. Im selben Untermenü schaltet **Zeilenende umschalten (LF/CRLF)** das Zeilenende der Datei um; die Statuszeile zeigt, welches gilt. Den Befehl **Kodierung umschalten (UTF-8/ISO-8859-1)** gibt es in dieser Version noch nicht.

Lange Zeilen bricht der Editor um (Voreinstellung: an). Mit **Ansicht ▸ Zeilenumbruch im Editor** schaltest du das ab und wieder an. Die Schriftgröße änderst du mit **Editor-Schrift größer** und **Editor-Schrift kleiner** im Menü **Ansicht ▸ Zoom und Schrift**.

## Fehler siehst du sofort

Du musst nicht bauen, um Fehler zu finden. Nach jeder Änderung prüft VintageCoder den Assembler-Quelltext im Hintergrund. Hat eine Zeile einen Fehler, wird die Stelle mit einer **Wellenlinie** unterstrichen und im Rand markiert. Fährst du mit der Maus darüber, zeigt die Karte die Meldung mit dem Fehlercode. Die Ablage zeigt auf der Seite **Probleme** alle Meldungen (siehe [Bauen und Starten](kapitel:bauen-starten)), die Statuszeile zählt Fehler, Warnungen und Hinweise. Mit **F8** springst du zum nächsten, mit **Umschalt+F8** zum vorigen Problem im Text.

Probiere es: Ändere `sta vic2.border` in `sta vic2.bordr`. Die Stelle wird unterstrichen, und die Karte meldet ein unbekanntes Symbol. Stelle den alten Text mit Cmd/Strg+Z wieder her.

![Die Code-Ansicht mit jumprun.asm, gescrollt bis Zeile 269. In den Zeilen 267 und 268 stehen „lda #$zz“ und „xyz 5“, beide rot unterstrichen und mit einem roten Punkt am Rand. Die Statusleiste unten zeigt „2 Fehler · 2 Meldungen in anderen Dateien“.](bild:editor-fehlerzeile "Fehlerhafte Zeilen sind rot unterstrichen und am Rand markiert, die Statusleiste zählt die Fehler.")

Für manche Meldungen gibt es einen Vorschlag zur Korrektur. Stelle den Cursor in die Zeile und drücke **Cmd/Strg+Punkt** (**Korrigieren (Quick-Fix) …**): Ein kleines Menü bietet die Korrekturen an, **Enter** wendet eine an. Gibt es keine, sagt VintageCoder das.

> **Info:** Die Sprachdienste (Fehlerprüfung, Vervollständigung, Definition, Gliederung, Korrigieren) arbeiten für Assembler-Dateien. In BASIC- und Text-Reitern bekommst du Färbung und Formatieren, im BASIC außerdem die Karte beim Daraufzeigen.

## Das Kontextmenü

Ein **Rechtsklick** im Editor setzt den Cursor an die Klickstelle und öffnet ein Menü mit:

- **Ausschneiden**, **Kopieren**, **Einfügen**, **Alles auswählen**,
- **Gehe zu Definition** (F12) und **Korrigieren (Quick-Fix) …** (Cmd/Strg+Punkt),
- **Kommentar umschalten** (Cmd/Strg+Alt+C) und **Gehe zu Zeile …** (Cmd/Strg+L),
- **Lesezeichen umschalten** und **Haltepunkt umschalten** (F9),
- **Alles falten** und **Alles entfalten**,
- in einer BASIC-Datei zusätzlich die drei Umwandlungen **BASIC: In Labels umwandeln …**, **BASIC: In Nummern umwandeln …** und **BASIC: Neu nummerieren …** (siehe [BASIC in der IDE](kapitel:basic)).

![Der Code-Editor mit der BASIC-Datei sterne.bas und geöffnetem Kontextmenü an Zeile 4. Die Einträge sind Ausschneiden und Kopieren (ausgegraut), Einfügen (markiert), Alles auswählen, Gehe zu Definition, Korrigieren (Quick-Fix) (ausgegraut), Kommentar umschalten, Gehe zu Zeile, Lesezeichen umschalten, Alles falten, Alles entfalten, Haltepunkt umschalten und die drei BASIC-Einträge zum Umwandeln und Neu nummerieren.](bild:editor-kontextmenue "Das Kontextmenü des Editors in einer BASIC-Datei mit den drei BASIC-Einträgen am Ende.")

## Der zweite Editorbereich

Manchmal möchtest du zwei Dateien nebeneinander sehen, zum Beispiel einen Quelltext und seine Bibliothek.

1. Öffne mindestens zwei Reiter. Mit nur einem Reiter lässt sich der Editor nicht teilen; VintageCoder sagt dir das.
2. Drücke **Cmd/Strg+Alt+T** (Menü **Ansicht ▸ Reiter und Teilen ▸ Editor teilen / Teilung aufheben**). Der Editorbereich teilt sich in zwei Hälften mit je eigener Reiterleiste. Du kannst auch einen Reiter mit der Maus an den rechten Rand ziehen („Rechts ablegen (teilt)“).
3. Schicke einen Reiter in die andere Hälfte: Menü **Ansicht ▸ Reiter und Teilen ▸ Reiter in die andere Hälfte schicken**, oder ziehe den Reiter hinüber.
4. Derselbe Befehl **Editor teilen / Teilung aufheben** hebt die Teilung wieder auf. Teilen kannst du den Editor nur einmal.

*[Bild folgt in E13: editor-zweiter-bereich]*

Ist das Fenster schmal, liegen die beiden Hälften untereinander statt nebeneinander.

## Dateien einbinden

Eine Datei in den Quelltext einzubinden geht auf zwei Wegen. Wähle die Datei im Projektbaum, öffne den Assembler-Quelltext und wähle im Kontextmenü der Datei **Als Include einfügen**: VintageCoder schreibt die Zeile mit dem Pfad relativ zur Quelldatei vor die Zeile mit der Schreibmarke (Cmd/Strg+Z nimmt sie zurück; ist die Datei schon eingebunden, fügt er nichts doppelt ein; Projektdateien und Diskettenabbilder lassen sich nicht einbinden). Oder du schreibst die Zeile selbst. Ein Ziehen aus dem Projektbaum in den Quelltext gibt es noch nicht. Die gängigen Zeilen (Pfade relativ zur Quelldatei, mit `/`):

| Zeile | Wirkung |
|---|---|
| `.include "pfad/datei.asm"` | setzt den Assembler-Quelltext an dieser Stelle ein, bei jedem Aufruf neu |
| `#include "pfad/datei.asm"` | setzt ihn **nur einmal** ein (für Bibliotheken) |
| `.incbin "pfad/datei.bin"` | bindet die Datei unverändert als Bytes ein |
| `.include "graphics/hero.vcsprite"` | bindet ein Sprite, einen Zeichensatz, einen Bildschirm oder einen Song aus dem Projekt ein; der Assembler wandelt die Mediendatei dabei selbst um |

In BASIC lautet die Zeile für Quelltext und Medien `#include "pfad"`; `.incbin` gibt es dort nicht. Wie das Einbinden von Medien genau funktioniert, steht im Kapitel [Assembler (VCASM) für Einsteiger](kapitel:assembler), die Zeilen der Direktiven in der Hilfeseite [Assembler: Direktiven](hilfe:asm-directives).

> **Info:** Etwas anderes ist das Ziehen von Dateien aus dem Finder oder Explorer **auf das Fenster**; das legt Dateien ins Projekt (siehe [Projekte und Dateien](kapitel:projekte)).

## Markdown-Dateien und Notizen

Dateien mit der Endung `.md` (oder `.markdown`) zeigt der Editor als **Vorschau**: Überschriften, Listen, Tabellen und Code, nur zum Lesen. **Markdown: Vorschau / Quelltext umschalten** (Menü **Ansicht** und Befehlspalette) wechselt zum Quelltext, in dem du schreibst. Notizen für dich (zum Beispiel eine `README.md`) legst du mit **Neue Notiz** oder **Neue Notiz …** an. Beim Schreiben helfen die Befehle im Menü **Bearbeiten ▸ Markdown** (Fett, Kursiv, Code, Link, Listen, Aufgaben, Zitat, Tabelle). Die Kürzel sind Cmd/Strg+I (kursiv), Cmd/Strg+E (Code), Cmd/Strg+Umschalt+X (durchgestrichen) und Cmd/Strg+Umschalt+K (Link). Mehr dazu steht in der Hilfeseite [Markdown](hilfe:markdown). Eine Gliederung der Überschriften gibt es nicht.

## Zusammenfassung der Kürzel

| Aktion | Mac | Windows und Linux |
|---|---|---|
| Speichern | Cmd+S | Strg+S |
| Alle speichern | Cmd+Alt+S | Strg+Alt+S |
| Neuer Reiter | Cmd+N | Strg+N |
| Datei schließen | Cmd+W | Strg+W |
| Nächster / voriger Reiter | Cmd+Alt+Rechts / Links | Strg+Alt+Rechts / Links |
| Rückgängig / Wiederholen | Cmd+Z / Cmd+Umschalt+Z | Strg+Z / Strg+Umschalt+Z |
| Suchen / Ersetzen | Cmd+F / Cmd+Alt+F | Strg+F / Strg+Alt+F |
| In Projekt suchen | Cmd+Umschalt+F | Strg+Umschalt+F |
| Gehe zu Zeile | Cmd+L | Strg+L |
| Gehe zu Definition | F12 | F12 |
| Korrigieren (Quick-Fix) | Cmd+Punkt | Strg+Punkt |
| Kommentar umschalten | Cmd+Alt+C | Strg+Alt+C |
| Lesezeichen umschalten / nächstes / voriges | Cmd+Alt+K / J / H | Strg+Alt+K / J / H |
| Haltepunkt umschalten | F9 | F9 |
| Nächstes / voriges Problem | F8 / Umschalt+F8 | F8 / Umschalt+F8 |
| Editor teilen | Cmd+Alt+T | Strg+Alt+T |
| Zen-Modus | Cmd+Alt+Z | Strg+Alt+Z |

Eine vollständige Tabelle steht in der Hilfeseite [Tastenkürzel](hilfe:shortcuts); die Einzelheiten zum Editor in der Hilfeseite [Editor](hilfe:editor).

## Häufige Stolpersteine

- **Die Vervollständigung erscheint nicht.** Sie arbeitet in Assembler-Dateien und öffnet sich beim Tippen. Tippe weiter oder öffne sie mit dem Befehl **Vervollständigung öffnen**.
- **Enter fügt einen Vorschlag ein, obwohl ich eine neue Zeile wollte.** Solange die Liste offen ist, übernimmt Enter den Eintrag. Drücke vorher **Esc**.
- **Die Zeile hat sich nach dem Verlassen verändert.** Das ist das Ausrichten. Es ist gewollt; mit Cmd/Strg+Z machst du es rückgängig.
- **„Für diesen Reiter gibt es keine Sprachdienste.“** Definition und Korrigieren gehören zu Assembler-Dateien. Prüfe die Art in der Statuszeile.
- **Ich habe einen gefalteten Abschnitt „verloren“.** Klicke auf den Platzhalter oder den Pfeil neben der Zeile, unter der die Zeilen fehlen.
- **Der Editor lässt sich nicht teilen.** Öffne zuerst einen zweiten Reiter.
- **F1 zeigt „Zu … gibt es keinen Hilfeeintrag.“** Zu diesem Wort gibt es keinen Eintrag. Suche in der Hilfe nach einem verwandten Begriff; die Karte beim Daraufzeigen beschreibt oft schon das Wort.

## Wie geht es weiter?

Jetzt schreibst du Quelltext komfortabel. Was die einzelnen Zeilen eines Assembler-Programms bedeuten, lernst du im nächsten Kapitel: [Assembler (VCASM) für Einsteiger](kapitel:assembler). Wenn du lieber mit BASIC beginnst, geht es mit [BASIC in der IDE](kapitel:basic) weiter.
