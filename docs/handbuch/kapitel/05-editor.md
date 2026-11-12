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

**Ziel:** Du schreibst Quelltext im Editor von VintageCoder zügig und mit Hilfe: Du nutzt die Vervollständigung, bekommst zu jedem Wort Hilfe, faltest lange Abschnitte zusammen, suchst und ersetzt Text, springst zu einer Zeile, fügst Sonderzeichen ein und erkennst Fehler schon beim Tippen.

## Das brauchst du

- Ein Projekt mit einer geöffneten `.asm`-Datei. Nimm dafür das Programm aus [Dein erstes Projekt](kapitel:erstes-projekt) oder lege eine neue Datei an. Für die Übungen kannst du diesen kleinen Quelltext verwenden (er baut ohne Fehler):

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

## Farben im Quelltext

Der Editor färbt **Assembler** und **BASIC** ein. Befehle, Direktiven (die Anweisungen an den Assembler mit einem Punkt davor, wie `.target`), Zahlen, Zeichenketten, Labels und Kommentare haben verschiedene Farben. So erkennst du auf einen Blick, ob ein Wort als das gilt, was du gemeint hast.

![Ein Assembler-Quelltext im Editor mit Zeilennummern, farbig hervorgehobenen Befehlen, Konstanten, Zahlen und Kommentaren](bild:editor-syntax "Syntaxfarben im Assembler-Quelltext")

Links neben dem Text stehen die **Zeilennummern**. In diesem Streifen (der „Rinne“) klickst du später Haltepunkte an und faltest Abschnitte.

## Schritt für Schritt: Mit Vervollständigung schreiben

Beim Tippen schlägt der Editor Wörter vor: Befehle, Direktiven, Chip-Register (zum Beispiel `vic2.border`), Labels, Konstanten und Makros, auch aus eingebundenen Dateien.

1. Setze den Cursor in eine neue Zeile und tippe `st`. Nach zwei Zeichen erscheint eine Liste mit Vorschlägen. (Nach einem Punkt oder einem `@` erscheint sie sofort.)
2. Mit den Pfeiltasten **↑ ↓** wählst du einen Eintrag aus.
3. **Tab** oder **Enter** übernimmt den markierten Eintrag. **Esc** schließt die Liste, ohne etwas einzufügen.
4. Brauchst du die Liste ohne vorher zu tippen, drücke **Strg+Leertaste** (auf dem Mac ebenfalls die Control-Taste).

![Die Vervollständigung im Editor: nach der Eingabe von „sta vic2.b“ zeigt eine Liste den Eintrag vic2.border und weitere Chip-Register](bild:editor-vervollstaendigung "Die Vorschlagsliste beim Tippen")

> **Tipp:** Probiere es mit `sta vic2.` – nach dem Punkt zeigt die Liste alle Register des VIC-II, des Grafikchips des C64, zum Beispiel `border` (Rahmenfarbe), `bg0` (Hintergrundfarbe) und `raster` (aktuelle Rasterzeile).

## Schritt für Schritt: Hilfe zum Wort

Du weißt nicht, was `cpx` macht oder was `vic2.border` bedeutet?

1. Setze den Cursor auf das Wort.
2. Drücke **F1**. Der Reiter **Hilfe** zeigt den Eintrag: bei einem Befehl alle Adressierungsarten mit Opcode, Bytes und Zyklen, bei einer Direktive die Schreibweise, bei einem Chip-Register die Adresse und die Bedeutung.

![Der Editor mit dem Cursor auf einem Befehl; rechts im Inspektor der Reiter Hilfe mit der Beschreibung des Befehls](bild:editor-hilfe-f1 "F1 öffnet die Hilfe zum Wort unter dem Cursor")

Noch bequemer ist der Reiter **Referenz** (Symbol „Referenz“ in der Aktivitätsleiste oder Reiter im Inspektor): Er zeigt denselben Inhalt **live**, während du den Cursor durch den Text bewegst. Du musst nicht jedes Mal F1 drücken.

![Der Reiter Referenz im Inspektor zeigt die Tabelle des Befehls am Cursor mit Adressierungsart, Opcode, Bytes und Zyklen](bild:editor-referenz "Der Reiter „Referenz“ folgt dem Cursor")

Dasselbe erreichst du mit einem Rechtsklick auf das Wort ▸ **Hilfe zu »Wort« (F1)**. Und wenn du die Maus kurz über einem Befehl, einer Direktive oder einem Register ruhen lässt, erscheint ein kleiner Hinweis mit der Beschreibung.

### Werte im Hover

Hältst du den Mauszeiger über eine **Konstante** (zum Beispiel `FARBE`) oder ein **Zahlenliteral** (`$d020`, `%1010`, `255`, `'A'`), erscheint eine Tabelle mit dem Wert dezimal, hexadezimal und binär, mit Vorzeichen, als Zeichen sowie Low- und High-Byte. Ist die Zahl die Adresse eines Chip-Registers, steht auch dessen Name dabei. Das ist praktisch, wenn du nicht im Kopf zwischen `$d020` und 53280 umrechnen willst.

## Schritt für Schritt: Abschnitte falten

Lange Quelltexte werden unübersichtlich. Mit dem **Falten** blendest du Abschnitte aus.

1. Neben der Zeilennummer erscheinen bei faltbaren Blöcken kleine Pfeile **▾** und **▸**. Gefaltet werden Blöcke wie `.macro`, `.if`, `.repeat`, Label-Abschnitte und Kommentarblöcke.
2. Klicke auf **▾**, um den Abschnitt einzuklappen. An seiner Stelle steht eine Marke. Ein Klick auf **▸** klappt ihn wieder auf.
3. Mit einem Rechtsklick in den Text und **Alles falten** bzw. **Alles entfalten** machst du das für die ganze Datei auf einmal.
4. Sobald du in einem gefalteten Abschnitt etwas bearbeitest, klappt er automatisch auf.

![Ein Quelltext mit einem eingeklappten Makro; neben der Zeilennummer stehen die Pfeile zum Falten und eine Marke zeigt, wie viele Zeilen versteckt sind](bild:editor-falten "Ein gefalteter Abschnitt hinterlässt eine Marke mit der Zahl der versteckten Zeilen")

## Schritt für Schritt: Suchen und Ersetzen

1. Drücke **Cmd/Strg+F** (Menü **Bearbeiten ▸ Suchen …**). Über dem Text erscheint die Suchleiste.
2. Tippe den Suchbegriff. **Enter** springt zum nächsten Treffer, **Umschalt+Enter** zum vorigen. Mit **Aa** beachtest du Groß- und Kleinschreibung. **Esc** schließt die Leiste.
3. Zum **Ersetzen** drückst du **Cmd/Strg+Alt+F** (Menü **Bearbeiten ▸ Ersetzen …**). Es erscheint ein zweites Feld. **Ersetzen** tauscht den aktuellen Treffer, **Alle** tauscht alle.
4. Hast du dich verklickt, hilft **Cmd/Strg+Z** (Rückgängig); **Cmd/Strg+Umschalt+Z** wiederholt.

![Die Suchleiste über dem Text mit dem Suchbegriff „sta“ und der Anzeige des Treffers; die Treffer sind im Text hervorgehoben](bild:editor-suchen "Suchen mit Cmd/Strg+F")

![Die Suchleiste mit einem zweiten Feld zum Ersetzen und den Knöpfen „Ersetzen“ und „Alle“](bild:editor-ersetzen "Ersetzen mit Cmd/Strg+Alt+F")

Möchtest du in **allen** Dateien des Projekts suchen? Drücke **Cmd/Strg+Umschalt+F**. In der Seitenleiste erscheint die **Suche** mit den Treffern, nach Datei gruppiert. Ein Klick springt zur Zeile im Editor.

## Schritt für Schritt: Zu einer Zeile springen

Eine Fehlermeldung nennt oft eine Zeilennummer, zum Beispiel „Zeile 42“.

1. Drücke **Cmd/Strg+L** (Menü **Bearbeiten ▸ Gehe zu Zeile …**).
2. Tippe die Zeilennummer und bestätige mit Enter. Der Cursor springt dorthin.

![Die Leiste „Gehe zu Zeile“ über dem Text mit der eingetippten Zeilennummer](bild:editor-gehe-zu "Gehe zu Zeile mit Cmd/Strg+L")

Zu einer **Definition** springst du mit **Cmd/Strg+Klick** auf ein Symbol, zum Beispiel auf ein Label oder eine Konstante, auch in eine andere Datei. Zu einer `.include`-Zeile findest du mit einem Doppelklick auf den Dateinamen die eingebundene Datei; es gibt sie noch nicht, bietet VintageCoder an, sie anzulegen.

## Schritt für Schritt: Sonderzeichen mit dem Zeichenwähler einfügen

Die alten Computer haben eigene Zeichensätze mit Grafikzeichen, Farbcodes und Cursorzeichen, die du auf deiner Tastatur nicht findest. Der **Zeichenwähler** zeigt sie dir.

1. Setze den Cursor an die Stelle, an der das Zeichen stehen soll, zum Beispiel in einen `.petscii "…"`-Text.
2. Drücke **Cmd/Strg+Umschalt+Y** (Rechtsklick ▸ **Zeichen einfügen …** oder Menü **Bearbeiten ▸ Zeichen einfügen …**).
3. Es öffnet sich ein Raster mit 16 × 16 Feldern. Beim C64, C128, Plus/4 und MEGA65 gibt es die Reiter **PETSCII** und **Bildschirmcode**, beim Atari **ATASCII** und **ANTIC-Code**. Blaue Kürzel sind Steuerzeichen (`CLR`, `HOME`, Farben, Cursor).
4. Klicke ein Zeichen an. Mit **Umschalt+Klick** wählst du einen Bereich; ein **Doppelklick** fügt das Zeichen sofort ein. Mit **Enter** übernimmst du die Auswahl, **Esc** schließt.
5. Steht der Cursor in einer Zeichenkette, schreibt VintageCoder Steuerzeichen in geschweiften Klammern, zum Beispiel `{clr}HALLO`. Sonst fügt es Zahlenwerte ein (`$c1, $93`).

![Der Zeichenwähler mit dem 16×16-Raster der PETSCII-Zeichen, rechts die Angaben zum gewählten Zeichen und unten die Auswahl „Einfügen als“](bild:editor-zeichenwaehler "Der Zeichenwähler (Cmd/Strg+Umschalt+Y)")

> **Info:** Die Zeichen stammen aus deinen ROMs. Ohne ROM zeigt der Wähler eine eingebaute Ersatzschrift und einen Hinweis (siehe [Installation und Ersteinrichtung](kapitel:installation)). Die Schreibweise `{name}` in Texten beschreibt die Hilfeseite [Texte und Zeichensätze](hilfe:asm-text).

## Schritt für Schritt: Dateien per Drag & Drop einbinden

Statt eine `.include`- oder `.incbin`-Zeile selbst zu tippen, ziehst du die Datei aus dem **Projektbaum** in den Quelltext. VintageCoder wählt die passende Zeile und den richtigen Pfad.

1. Öffne einen **Assembler-** oder **BASIC-Quelltext**.
2. Ziehe eine Datei aus dem Projektbaum an eine Stelle im Quelltext. Eine **orange Linie** zeigt zwischen zwei Zeilen, wo die Zeile entsteht, und am Mauszeiger steht die Zeile selbst. Geht es nicht, steht dort in Rot der Grund.

![Eine Datei wird aus dem Projektbaum in den Quelltext gezogen: Linie zwischen zwei Zeilen und die Vorschau .incbin "data/level.bin" am Mauszeiger](bild:include-ziehen "Beim Ziehen zeigen Linie und Vorschau, was entsteht")

3. Lass die Maustaste los. Die Zeile wird eingefügt. Unten erscheint eine kleine Leiste mit der Zeile, dem Knopf **Rückgängig** und, wo es eine Alternative gibt, einem Knopf zum Umschalten.

![Die eingefügte Zeile .incbin "data/level.bin" im Quelltext, darunter die Leiste mit „Rückgängig“](bild:include-eingefuegt "Die eingefügte Zeile und die Leiste mit „Rückgängig“")

4. **Rückgängig** in der Leiste oder **Cmd/Strg+Z** im Editor nimmt die Zeile als **eine** Aktion wieder heraus.

**Ohne Maus:** Rechtsklick auf die Datei im Projektbaum ▸ **Als Include einfügen**. Die Zeile entsteht unter dem Cursor des aktiven Quelltexts. Ist keiner offen, erscheint ein Hinweis im Ausgabe-Panel.

Welche Zeile entsteht, hängt von Datei und Ziel ab (Pfade relativ zur Quelldatei, mit `/`, auch bei Leerzeichen und Umlauten):

| Datei | im Assembler | in BASIC (`.bas`) |
|---|---|---|
| Assembler-Quelltext (`.asm`, `.inc`, `.s`) | `.include "pfad"`; die Leiste kann auf `#include` umschalten | nicht möglich |
| BASIC-Quelltext mit derselben Endung | nicht möglich | `#include "pfad"` |
| Sprite, Zeichensatz, Bildschirm, Song … | `.include "…/build/export/name.asm"` | `#include "…/build/export/name.data.bas"` |
| Sonstige Dateien (`.bin`, `.prg`, `.txt` …) | `.incbin "pfad"`; bei `.prg` umschaltbar auf „ohne Ladeadresse“ | nicht möglich |

Bei Sprites, Zeichensätzen und Co. bindest du den **Export** ein; den erzeugt der Build bei Bedarf neu.

**Was VintageCoder ablehnt** (mit Hinweis, es wird nichts eingefügt): eine Datei in sich selbst, Ordner, die Projektdatei und Diskettenimages, Dateien außerhalb des Projekts, Namen mit einem Anführungszeichen `"` (benenne die Datei um), `.sbas`, andere BASIC-Dialekte (andere Endung als das Ziel) und alles, was die Sprache nicht kennt, zum Beispiel `.incbin` in BASIC. Ziele, die weder Assembler- noch BASIC-Quelltext sind, nehmen keine Includes an.

![Beim Ziehen der Projektdatei steht am Mauszeiger in Rot, dass sie sich nicht einbinden lässt](bild:include-abgelehnt "Abgelehnt: Projektdateien lassen sich nicht einbinden")

**Doppelte Einträge:** Ist die Datei schon eingebunden, fügt VintageCoder nichts ein, springt zu der vorhandenen Zeile und hebt sie kurz hervor.

> **Info:** Etwas anderes ist das Ziehen von Dateien aus dem Finder oder Explorer **auf das Fenster**; das legt Dateien ins Projekt (siehe [Projekte und Dateien](kapitel:projekte)).

## Das Kontextmenü

Ein **Rechtsklick** im Editor setzt den Cursor an die Klickstelle und öffnet ein Menü mit:

- **Ausschneiden**, **Kopieren**, **Einfügen**, **Alles auswählen**,
- **Haltepunkt setzen / entfernen** (bei Assembler-Quelltext; siehe [Der Debugger](kapitel:debugger)),
- **Zur Definition von »Wort«**,
- **Hilfe zu »Wort« (F1)**,
- bei `.include`-Zeilen **Datei öffnen**,
- **Zeile(n) auskommentieren / Kommentar entfernen**,
- **Zeichen einfügen …** (Cmd/Strg+Umschalt+Y),
- **Alles falten** und **Alles entfalten**.

![Das Kontextmenü des Editors mit den Einträgen Ausschneiden, Kopieren, Einfügen, Haltepunkt setzen, Zur Definition, Hilfe zum Wort, Zeile auskommentieren, Zeichen einfügen und Alles falten](bild:editor-kontextmenue "Das Kontextmenü im Editor (Rechtsklick)")

> **Tipp:** **Zeile auskommentieren** ist praktisch zum Ausprobieren: Markiere mehrere Zeilen, rechter Mausklick, **Zeilen auskommentieren**. VintageCoder setzt ein `;` vor jede Zeile. Dasselbe Menü nimmt die Zeichen wieder weg.

## Der Editor räumt auf

Verlässt du eine **bearbeitete** Zeile (Cursor in eine andere Zeile, Enter, Mausklick), richtet VintageCoder sie aus. Beim Assembler steht ein Label in Spalte 0, der Befehl danach, Befehle und Direktiven werden klein geschrieben, und Kommentare sitzen in einer Spalte. Bei BASIC gibt es ein Leerzeichen hinter der Zeilennummer. Gefällt dir eine Ausrichtung nicht, machst du sie mit **Cmd/Strg+Z** rückgängig wie jede andere Eingabe.

## Fehler siehst du sofort

Du musst nicht bauen, um Fehler zu finden. Nach jeder Änderung prüft VintageCoder den Assembler-Quelltext im Hintergrund. Hat eine Zeile einen Fehler, wird sie **rot unterstrichen** und ihre **Zeilennummer wird rot**. Fährst du mit der Maus über die Zeile, zeigt ein Hinweis die Meldung mit dem Fehlercode. Im Fußpanel zeigt der Reiter **Probleme** alle Meldungen (siehe [Bauen und Starten](kapitel:bauen-starten)).

Probiere es: Ändere `sta vic2.border` in `sta vic2.bordr`. Die Zeile wird rot, und der Hinweis meldet ein unbekanntes Symbol. Stelle den alten Text mit Cmd/Strg+Z wieder her.

![Eine Zeile mit rot unterstrichenem Fehler und roter Zeilennummer; ein Hinweisfenster darunter zeigt die Meldung „unbekanntes Symbol“](bild:editor-fehlerzeile "Ein Fehler wird rot unterstrichen, die Meldung erscheint beim Darüberfahren")

## Der zweite Editorbereich

Manchmal möchtest du zwei Dateien nebeneinander sehen, zum Beispiel einen Quelltext und seine Bibliothek.

1. Drücke **Cmd/Strg+\\** (Menü **Ansicht ▸ Editor teilen / Teilung aufheben**). Der Editorbereich teilt sich in zwei Hälften.
2. Ziehe einen Reiter in die andere Hälfte, oder halte beim Klick im Projektbaum **Cmd/Strg** gedrückt (Rechtsklick ▸ **Im zweiten Editorbereich öffnen**).
3. Derselbe Befehl hebt die Teilung wieder auf. Teilen kannst du den Editor nur einmal.

![Der Editor ist geteilt: links main.asm, rechts die Bibliothek utils.asm, jeweils mit eigener Reiterleiste](bild:editor-zweiter-bereich "Der zweite Editorbereich: zwei Dateien nebeneinander")

## Zusammenfassung der Kürzel

| Aktion | Mac | Windows |
|---|---|---|
| Speichern | Cmd+S | Strg+S |
| Rückgängig / Wiederholen | Cmd+Z / Cmd+Umschalt+Z | Strg+Z / Strg+Umschalt+Z |
| Suchen / Ersetzen | Cmd+F / Cmd+Alt+F | Strg+F / Strg+Alt+F |
| Gehe zu Zeile | Cmd+L | Strg+L |
| Include einfügen | Datei aus dem Projektbaum ziehen, oder Rechtsklick ▸ „Als Include einfügen“ | dasselbe |
| In Projekt suchen | Cmd+Umschalt+F | Strg+Umschalt+F |
| Vervollständigung öffnen | Ctrl+Leertaste | Strg+Leertaste |
| Hilfe zum Wort | F1 | F1 |
| Zeichenwähler | Cmd+Umschalt+Y | Strg+Umschalt+Y |
| Editor teilen | Cmd+\\ | Strg+\\ |
| Datei schließen | Cmd+W | Strg+W |
| Alle speichern | Cmd+Alt+S | Strg+Alt+S |

Eine vollständige Tabelle steht in der Hilfeseite [Tastenkürzel](hilfe:shortcuts); die Einzelheiten zum Editor in der Hilfeseite [Editor](hilfe:editor).

## Häufige Stolpersteine

- **Die Vervollständigung erscheint nicht.** Sie öffnet sich nach zwei Zeichen, nach `.` und `@` sofort. Drücke sonst **Strg+Leertaste**. Beachte: Der Befehl ist **Strg**, auch auf dem Mac.
- **Enter fügt einen Vorschlag ein, obwohl ich eine neue Zeile wollte.** Solange die Liste offen ist, übernimmt Enter den Eintrag. Drücke vorher **Esc**.
- **F1 zeigt nichts Brauchbares.** Der Cursor muss **auf** einem Wort stehen, zum Beispiel einem Befehl oder einer Direktive.
- **Die Zeile hat sich nach dem Verlassen verändert.** Das ist das Ausrichten. Das Ausrichten ist gewollt; mit Cmd/Strg+Z machst du es rückgängig.
- **Der Zeichenwähler ist ausgegraut.** Es ist keine Quelltext-Datei geöffnet. Öffne erst eine.
- **Ich habe einen gefalteten Abschnitt „verloren“.** Rechtsklick ▸ **Alles entfalten**.

## Wie geht es weiter?

Jetzt schreibst du Quelltext komfortabel. Was die einzelnen Zeilen eines Assembler-Programms bedeuten, lernst du im nächsten Kapitel: [Assembler (VCASM) für Einsteiger](kapitel:assembler). Wenn du lieber mit BASIC beginnst, geht es mit [BASIC in der IDE](kapitel:basic) weiter.
