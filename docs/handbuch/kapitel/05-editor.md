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

**Ziel:** Du schreibst Quelltext im Editor von VintageCoder zügig und mit Hilfe: Du nutzt die Vervollständigung, bekommst zu jedem Wort Hilfe, faltest lange Abschnitte zusammen, suchst und ersetzt Text, springst zu einer Zeile, fügst Sonderzeichen über das Zeichen-Panel ein und erkennst Fehler schon beim Tippen.

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

![Ein Assembler-Quelltext im Editor mit Zeilennummern, Kommentaren und farbig hervorgehobenen Befehlen, Konstanten und Zahlen; links der Dateibaum](bild:editor-syntax "Syntaxfarben im Assembler-Quelltext")

Links neben dem Text stehen die **Zeilennummern**. In diesem Streifen (der „Rinne“) klickst du Haltepunkte an und faltest Abschnitte.

## Schritt für Schritt: Mit Vervollständigung schreiben

Beim Tippen schlägt der Editor Wörter vor: Befehle, Direktiven, Chip-Register (zum Beispiel `vic2.border`), Labels, Konstanten und Makros, auch aus eingebundenen Dateien.

1. Setze den Cursor in eine neue Zeile und tippe `st`. Nach zwei Zeichen erscheint eine Liste mit Vorschlägen. (In `vic2.` zählt der Punkt mit: Die Liste zeigt dann gleich die Register des Chips.)
2. Mit den Pfeiltasten **↑ ↓** wählst du einen Eintrag aus.
3. **Tab** oder **Enter** übernimmt den markierten Eintrag. **Esc** schließt die Liste, ohne etwas einzufügen.
4. Die Liste erscheint nur beim Tippen. Ein Kürzel, das sie von Hand öffnet, gibt es nicht: Tippe ein weiteres Zeichen oder lösche eines und tippe es neu.

![Die Vervollständigung im Editor: nach der Eingabe von „sta vic2.b“ zeigt eine Liste vic2.border und vic2.bg0 bis vic2.bg3.](bild:editor-vervollstaendigung "Die Vorschlagsliste beim Tippen")

Ohne zu tippen öffnest du die Liste mit dem Befehl **Vervollständigung öffnen** (Bearbeiten ▸ …, Befehlspalette); er hat bewusst kein Tastenkürzel.

> **Tipp:** Probiere es mit `sta vic2.` – nach dem Punkt zeigt die Liste alle Register des VIC-II, des Grafikchips des C64, zum Beispiel `border` (Rahmenfarbe), `bg0` (Hintergrundfarbe) und `raster` (aktuelle Rasterzeile).

## Schritt für Schritt: Hilfe zum Wort

Du weißt nicht, was `cpx` macht oder was `vic2.border` bedeutet?

1. Setze den Cursor auf das Wort.
2. Drücke **F1**. Der Reiter **Hilfe** zeigt den Eintrag: bei einem Befehl alle Adressierungsarten mit Opcode, Bytes und Zyklen, bei einer Direktive die Schreibweise, bei einem Chip-Register die Adresse und die Bedeutung.

![Der Editor mit dem Cursor auf dem Befehl sta; rechts im Panel der Reiter Hilfe mit der Beschreibung des Befehls und seinen Adressierungsarten.](bild:editor-hilfe-f1 "F1 öffnet die Hilfe zum Wort unter dem Cursor")

Die **Hilfe** im rechten Panel zeigt den Eintrag zum Wort; ein eigener Reiter „Referenz“, der live mit dem Cursor mitläuft, ist in dieser Version noch nicht enthalten. Du drückst dafür F1 oder wählst die Hilfe im Panel.

Dasselbe erreichst du mit einem Rechtsklick auf das Wort ▸ **Hilfe zum Wort** (F1). Und wenn du die Maus kurz über einem Befehl, einer Direktive oder einem Register ruhen lässt, erscheint ein kleiner Hinweis mit der Beschreibung.

### Werte im Hover

Hältst du den Mauszeiger über eine **Konstante** (zum Beispiel `FARBE`) oder ein **Zahlenliteral** (`$d020`, `%1010`, `255`, `'A'`), erscheint eine Tabelle mit dem Wert dezimal, hexadezimal und binär, mit Vorzeichen, als Zeichen sowie Low- und High-Byte. Ist die Zahl die Adresse eines Chip-Registers, steht auch dessen Name dabei. Das ist praktisch, wenn du nicht im Kopf zwischen `$d020` und 53280 umrechnen willst.

## Schritt für Schritt: Abschnitte falten

Lange Quelltexte werden unübersichtlich. Mit dem **Falten** blendest du Abschnitte aus.

1. Neben der Zeilennummer erscheinen bei faltbaren Blöcken kleine Pfeile **▾** und **▸**. Gefaltet werden Blöcke wie `.macro`, `.if`, `.repeat`, Label-Abschnitte und Kommentarblöcke.
2. Klicke auf **▾**, um den Abschnitt einzuklappen. Seine Zeilen sind dann versteckt, in der Rinne steht **▸**. Ein Klick auf **▸** klappt ihn wieder auf.
3. **Alles falten** und **Alles entfalten** (Menü Bearbeiten, Befehlspalette und Rechtsklick im Editor) klappen alle Abschnitte auf einmal zu oder auf; sie haben kein Tastenkürzel.
4. Sobald du in einem gefalteten Abschnitt etwas bearbeitest, klappt er automatisch auf.

![Ein Quelltext mit einem eingeklappten Block: neben der Zeilennummer steht der Pfeil zum Falten.](bild:editor-falten "Ein gefalteter Abschnitt: seine Zeilen sind versteckt, der Pfeil ▸ in der Rinne klappt sie wieder auf")

## Schritt für Schritt: Suchen und Ersetzen

1. Drücke **Cmd/Strg+F** (Menü **Bearbeiten ▸ Suchen und Ersetzen ▸ Suchen …**). Über dem Text erscheint die Suchleiste.
2. Tippe den Suchbegriff. **Enter** springt zum nächsten Treffer, **Umschalt+Enter** zum vorigen. Mit **Aa** beachtest du Groß- und Kleinschreibung. **Esc** schließt die Leiste.
3. Zum **Ersetzen** drückst du **Cmd/Strg+Alt+F** (Menü **Bearbeiten ▸ Suchen und Ersetzen ▸ Ersetzen …**). Es erscheint ein zweites Feld. **Ersetzen** tauscht den aktuellen Treffer, **Alle** tauscht alle.
4. Hast du dich verklickt, hilft **Cmd/Strg+Z** (Rückgängig); **Cmd/Strg+Umschalt+Z** wiederholt.

![Die Suchleiste über dem Text mit dem Suchbegriff „sta“, der Anzeige „1 von 15“ und den Knöpfen für vorigen und nächsten Treffer; die Treffer sind im Text hervorgehoben.](bild:editor-suchen "Suchen mit Cmd/Strg+F")

![Die Suchleiste mit einem zweiten Feld zum Ersetzen und den Knöpfen „Ersetzen“ und „Alle“.](bild:editor-ersetzen "Ersetzen mit Cmd/Strg+Alt+F")

Möchtest du in **allen** Dateien des Projekts suchen? Drücke **Cmd/Strg+Umschalt+F**. In der linken Spalte erscheint die **Suche** mit den Treffern, nach Datei gruppiert. Ein Klick springt zur Zeile im Editor.

## Schritt für Schritt: Zu einer Zeile springen

Eine Fehlermeldung nennt oft eine Zeilennummer, zum Beispiel „Zeile 42“.

1. Drücke **Cmd/Strg+L** (Menü **Bearbeiten ▸ Suchen und Ersetzen ▸ Gehe zu Zeile …**).
2. Tippe die Zeilennummer und bestätige mit Enter. Der Cursor springt dorthin.

![Die Leiste „Gehe zu Zeile“ über dem Text mit der eingetippten Zeilennummer.](bild:editor-gehe-zu "Gehe zu Zeile mit Cmd/Strg+L")

Zu einer **Definition** springst du mit **F12** (Rechtsklick ▸ **Gehe zu Definition**, Menü **Bearbeiten**), wenn der Cursor auf einem Symbol steht, zum Beispiel einem Label oder einer Konstante; die Definition darf auch in einer anderen Datei liegen. Das gilt nur für Assembler-Dateien. Ein Klick mit gedrückter Cmd/Strg-Taste und ein Doppelklick auf einen `.include`-Dateinamen sind in dieser Version noch nicht belegt; einen Doppelklick gibt es nur zum Markieren eines Wortes. Eine eingebundene Datei öffnest du im Projektbaum.

## Schritt für Schritt: Sonderzeichen mit dem Zeichen-Panel einfügen

Die alten Computer haben eigene Zeichensätze mit Grafikzeichen, Farbcodes und Cursorzeichen, die du auf deiner Tastatur nicht findest. Das **Zeichen-Panel** listet sie auf.

1. Setze den Cursor an die Stelle, an der das Zeichen stehen soll, zum Beispiel in einen `.petscii "…"`-Text.
2. Klicke im rechten Panel auf das Symbol **Ω** (ist das Panel zugeklappt, öffnet der Klick es; Cmd/Strg+Alt+B klappt das Panel ein und aus).
3. Das Panel listet die Steuerzeichen des Zielsystems mit Namen, Code und Wirkung, zum Beispiel `{clr}`, die Farben, Cursor- und Funktionstasten. Im Suchfeld findest du Namen, Zahlen (`$93`, `147`) und getippte Zeichen; bei einer Suche erscheinen auch gewöhnliche Zeichen mit ihrem Code.
4. Klicke einen Eintrag an. VintageCoder fügt ihn an der Cursorstelle ein, und zwar in der Form, die dort passt: in einer Zeichenkette in geschweiften Klammern (`{clr}HALLO`), im BASIC nach `PRINT` mit Anführungszeichen, in `CHR$(` als Zahl, sonst als Zahlenwert. Passt an der Stelle nichts, steht ein Hinweis im Protokoll und es wird nichts eingefügt.

Ein eigenes Zeichenwähler-Fenster mit einem Raster aller 256 Zeichen, eine Ansicht „Meine“ mit zuletzt benutzten Zeichen und das Ziehen von Zeichen in den Editor sind in dieser Version noch nicht enthalten. Das Panel arbeitet auch im BASIC-Editor.

> **Info:** Die Zeichen stammen aus deinen ROMs. Ohne ROM nimmt VintageCoder einen eigenen Standardsatz (siehe [Installation und Ersteinrichtung](kapitel:installation)). Die Schreibweise `{name}` in Texten beschreibt die Hilfeseite [Texte und Zeichensätze](hilfe:asm-text).

Steht ein Steuerzeichen in einer Zeichenkette, zum Beispiel `print "{clr}{red}hallo"`, hebt der Editor es hervor: Farbcodes erscheinen als Farbfeld, ein unbekannter Name wird rot unterschlängelt, und der Hover zeigt Code und Wirkung. Tippst du ein `{` in der Zeichenkette, bietet der Editor die Namen an. Assembler und BASIC verstehen dieselben Namen, jede Sprache hat ihre eigene Hauptschreibweise (`{orange}` im Assembler, `{orng}` im BASIC), und beide Schreibweisen gelten überall.

## Schritt für Schritt: Dateien per Drag & Drop einbinden

Statt eine `.include`- oder `.incbin`-Zeile selbst zu tippen, ziehst du die Datei aus dem **Projektbaum** in den Quelltext. VintageCoder wählt die passende Zeile und den richtigen Pfad.

1. Öffne einen **Assembler-** oder **BASIC-Quelltext**.
2. Ziehe eine Datei aus dem Projektbaum an eine Stelle im Quelltext. Eine **farbige Linie** zeigt zwischen zwei Zeilen, wo die Zeile entsteht: obere Hälfte einer Zeile heißt davor, untere Hälfte dahinter.

![Eine Datei wird aus dem Projektbaum in den Quelltext gezogen: eine blaue Linie zwischen zwei Zeilen zeigt, wo die Zeile entsteht.](bild:include-ziehen "Beim Ziehen zeigt die Linie, wo die Zeile entsteht")

3. Lass die Maustaste los. Die Zeile wird eingefügt, und die Statusleiste unten nennt sie („Eingebunden: .incbin "data/level.bin"“). Geht es nicht, steht dort der Grund, und es wird nichts eingefügt.
4. **Cmd/Strg+Z** im Editor nimmt die Zeile als **eine** Aktion wieder heraus.

**Ohne Maus:** Rechtsklick auf die Datei im Projektbaum ▸ **Als Include einfügen**. Die Zeile entsteht unter der Cursorzeile des aktiven Quelltexts. Ist keiner offen, steht ein Hinweis in der Statusleiste.

Welche Zeile entsteht, hängt von Datei und Ziel ab (Pfade relativ zur Quelldatei, mit `/`, auch bei Leerzeichen und Umlauten):

| Datei | im Assembler | in BASIC (`.bas`) |
|---|---|---|
| Assembler-Quelltext (`.asm`, `.inc`, `.s`) | `.include "pfad"`; für „nur einmal einbinden“ tippst du `#include` selbst | nicht möglich |
| BASIC-Quelltext mit derselben Endung | nicht möglich | `#include "pfad"` |
| Sprite, Zeichensatz, Bildschirm, Song … | `.include "…/build/export/name.asm"` | `#include "…/build/export/name.data.bas"` |
| Sonstige Dateien (`.bin`, `.prg`, `.txt` …) | `.incbin "pfad"`; bei `.prg` ohne Ladeadresse hängst du `, 2` selbst an | nicht möglich |

Bei Sprites, Zeichensätzen und Co. bindest du den **Export** ein; den erzeugt der Build bei Bedarf neu.

### Zwei Möglichkeiten, die du von Hand wählst

Bei zwei Dateiarten gibt es eine zweite, ebenfalls sinnvolle Zeile. VintageCoder fügt die Standardzeile ein; die andere tippst du selbst, wenn du sie brauchst.

**`.include` oder `#include` (Assembler-Quelltext in Assembler).** Beide setzen den Inhalt der anderen Datei an dieser Stelle ein. Der Unterschied liegt im Mehrfach-Einbinden:

- `.include "datei"` setzt die Datei **jedes Mal** neu ein. Bindest du dieselbe Datei an zwei Stellen ein, meldet der Assembler die Namen als doppelt (Fehler **E204**).
- `#include "datei"` bindet sie **nur einmal** ein, wie ein `#pragma once` in C. Sie kommt beim zweiten Mal nicht mehr dazu.

Wähle `.include`, wenn du den Inhalt wirklich an dieser Stelle haben willst (zum Beispiel Code, der an einer festen Adresse stehen soll). Schreibe `#include`, wenn die Datei eine **Bibliothek** ist, die auch andere Dateien einbinden könnten: Dann dürfen sich Quelltexte gegenseitig einbinden, ohne dass Namen doppelt definiert werden.

**`.prg` mit oder ohne Ladeadresse (`.incbin` einer `.prg`-Datei).** Eine `.prg`-Datei beginnt mit einer **Ladeadresse** aus 2 Bytes; sie sagt dem Rechner, wohin das Programm geladen wird, und gehört nicht zu den eigentlichen Daten.

- `.incbin "x.prg"` (Standard) bindet die Datei **komplett** ein, also einschließlich dieser zwei Bytes.
- `.incbin "x.prg", 2` überspringt die ersten zwei Bytes. Du bekommst nur die Daten, zum Beispiel ein Bild oder eine Musik, die dein Programm selbst an die richtige Stelle kopiert.

Weißt du nicht, welche Fassung du brauchst: Brauchst du die Datei als reine Daten im Speicher, nimm „ohne Ladeadresse“. Brauchst du sie genau so, wie sie auf Diskette läge, nimm den Standard. Genaueres zu `.incbin` mit Versatz und Länge steht in der Hilfeseite [Assembler: Direktiven](hilfe:asm-directives).

**Was VintageCoder ablehnt** (mit Hinweis in der Statusleiste, es wird nichts eingefügt): eine Datei in sich selbst, Ordner, die Projektdatei und Diskettenimages, Dateien außerhalb des Projekts, Namen mit einem Anführungszeichen `"` (benenne die Datei um), `.sbas`, andere BASIC-Dialekte (andere Endung als das Ziel) und alles, was die Sprache nicht kennt, zum Beispiel `.incbin` in BASIC. Ziele, die weder Assembler- noch BASIC-Quelltext sind, nehmen keine Includes an.


**Doppelte Einträge:** Ist die Datei schon eingebunden, fügt VintageCoder nichts ein und setzt den Cursor auf die vorhandene Zeile.

> **Info:** Etwas anderes ist das Ziehen von Dateien aus dem Finder oder Explorer **auf das Fenster**; das legt Dateien ins Projekt (siehe [Projekte und Dateien](kapitel:projekte)).

## Das Kontextmenü

Ein **Rechtsklick** im Editor setzt den Cursor an die Klickstelle und öffnet ein Menü mit:

- **Ausschneiden**, **Kopieren**, **Einfügen**, **Alles auswählen** (auch Cmd/Strg+A),
- **Hilfe zum Wort** (F1),
- **Gehe zu Definition** (F12; siehe oben),
- **Alles falten** und **Alles entfalten**,
- **Haltepunkt umschalten** (F9; siehe [Der Debugger](kapitel:debugger)),
- **Debuggen: Bis hierher ausführen** (nur in einer Debug-Sitzung wählbar).

Das Auskommentieren ganzer Zeilen ist in dieser Version noch nicht enthalten.

![Das Kontextmenü des Editors mit Ausschneiden, Kopieren, Einfügen, Alles auswählen, Hilfe zum Wort, Gehe zu Definition, Alles falten, Alles entfalten, Haltepunkt umschalten und Debuggen: Bis hier ausführen.](bild:editor-kontextmenue "Das Kontextmenü im Editor (Rechtsklick)")

## Markdown-Dateien: Notizen, README und Handbuchtexte

Dateien mit der Endung `.md` (oder `.markdown`) zeigt der Editor als **Markdown**: Überschriften, Fett, Code und Links sind farbig, die Zeichen `#`, `**` und `` ` `` bleiben dabei sichtbar. Du siehst immer den echten Text, es gibt kein „Rich-Text“-Bearbeiten. In der Statuszeile steht dann „Markdown“.

### Schritt für Schritt: Die Vorschau benutzen

1. Öffne eine `.md`-Datei, zum Beispiel die `README.md` deines Projekts.
2. Eine `.md`-Datei mit Inhalt öffnet zuerst als **gerenderte Seite**. Über ihr steht eine schmale Leiste „Markdown-Vorschau“ mit dem Knopf **Quelltext**. Ein Klick darauf zeigt den Text zum Bearbeiten; dort heißt der Knopf **Vorschau** und führt zurück. Dasselbe tut der Befehl **Ansicht ▸ Markdown: Vorschau / Quelltext umschalten** (er hat kein Kürzel).
3. Eine Ansicht, die Text und Vorschau nebeneinander zeigt, ist in dieser Version noch nicht enthalten. Du schaltest also um, statt zu beiden Seiten zu schauen. Eine leere Datei öffnet gleich im Editor.
4. Die Vorschau ist nur zum Lesen. Links auf Hilfethemen (`hilfe:…`) öffnen das Thema; andere Links sind farbig und unterstrichen, öffnen aber nichts von allein.

Welche Ansicht eine Datei hat, merkt sich VintageCoder mit den offenen Reitern.

### Was Markdown hier kann und was nicht

| Geht | Geht nicht |
|---|---|
| Überschriften, Absätze, Fett, Kursiv, Durchgestrichen, `Code` und Codeblöcke | Fußnoten, Definitionslisten, Mathe, Diagramme, Emoji-Kürzel wie `:smile:` |
| Aufzählungen, Nummerierungen, verschachtelt; Aufgabenlisten `- [x]` (in der Vorschau nur Anzeige) | Rohes HTML wird nicht dargestellt, nur als Text gezeigt und nie ausgeführt |
| Zitate, Trennlinien, Tabellen mit Ausrichtung | Bilder aus dem Internet (aus Datenschutzgründen nicht geladen) |
| Links werden farbig und unterstrichen gezeigt | Bilder anzeigen; Links auf Webseiten oder Projektdateien öffnen |
| Bilder `![Alt](pics/bild.png)` erscheinen in der Vorschau nur als Platzhaltertext | Front Matter (`---` … `---` am Anfang) erscheint nicht in der Vorschau |

### Schreibhilfen

Bei einer geöffneten Markdown-Datei findest du unter **Bearbeiten ▸ Markdown ▸ …** (und in der Befehlspalette) die Schreibhilfen: Fett, Kursiv, Code, Durchgestrichen, Link, Bild, Überschrift tiefer/höher, Aufzählung, Nummerierung, Aufgabenliste, Zitat, Aufgabe abhaken, Tabelle einfügen und ausrichten. Kürzel: **Cmd/Strg+I** kursiv, **Cmd/Strg+E** Code, **Cmd/Strg+Umschalt+X** durchgestrichen, **Cmd/Strg+Umschalt+K** Link; Fett hat kein Kürzel (Cmd/Strg+Umschalt+B ist der Zen-Modus). Jeder Befehl ist ein Schritt für Rückgängig. Die **Vorschau** schaltest du mit „Markdown: Vorschau / Quelltext umschalten“ (Menü **Ansicht**) um.

### Die Gliederung

Der Reiter **Gliederung** im rechten Panel zeigt die Überschriften der Markdown-Datei mit ihren Ebenen. Ein Klick springt in die Zeile, der Pfeil klappt Unterpunkte ein. Überschriften in Codeblöcken zählen nicht.

> **Info:** Eine `.txt`-Datei oder ein unbenannter Reiter wird mit Bearbeiten › Markdown › „als Markdown behandeln“ zu Markdown (nur für diesen Reiter, die Datei bleibt unverändert).

## Der Editor räumt auf

Verlässt du eine **bearbeitete** Zeile (der Cursor wechselt in eine andere Zeile, Enter, Mausklick), richtet VintageCoder sie aus. Beim Assembler steht ein Label in Spalte 0, der Befehl danach, und Kommentare sitzen in einer Spalte. Bei BASIC gibt es ein Leerzeichen hinter der Zeilennummer. Gefällt dir eine Ausrichtung nicht, machst du sie mit **Cmd/Strg+Z** rückgängig wie jede andere Eingabe. Bei einer Auswahl, bei offener Vervollständigung und in den Studios passiert nichts von selbst.

Abschalten: **Einstellungen ▸ Editor ▸ Zeile beim Verlassen formatieren**. Auf Befehl, auch bei abgeschalteter Einstellung: **Bearbeiten ▸ Formatieren ▸ Zeile formatieren** und **Datei formatieren** (die ganze Datei, als **eine** Aktion rückgängig zu machen). Beide stehen auch in der Befehlspalette.

## Fehler siehst du sofort

Du musst nicht bauen, um Fehler zu finden. Nach jeder Änderung prüft VintageCoder den Assembler-Quelltext im Hintergrund. Hat eine Zeile einen Fehler, wird sie **rot unterstrichen** und ihre **Zeilennummer wird rot**. Fährst du mit der Maus über die Zeile, zeigt ein Hinweis die Meldung mit dem Fehlercode. In der Schublade zeigt der Reiter **Probleme** alle Meldungen (siehe [Bauen und Starten](kapitel:bauen-starten)).

Probiere es: Ändere `sta vic2.border` in `sta vic2.bordr`. Die Zeile wird rot, und der Hinweis meldet ein unbekanntes Symbol. Stelle den alten Text mit Cmd/Strg+Z wieder her.

![Eine rot hinterlegte Zeile mit rot unterstrichenem Fehler: „jmp ende“ nennt ein Symbol, das es nicht gibt.](bild:editor-fehlerzeile "Ein Fehler wird rot unterstrichen, die Meldung erscheint beim Darüberfahren")

## Der zweite Editorbereich

Manchmal möchtest du zwei Dateien nebeneinander sehen, zum Beispiel einen Quelltext und seine Bibliothek.

1. Öffne mindestens zwei Reiter. Mit nur einem Reiter lässt sich der Editor nicht teilen.
2. Drücke **Cmd/Strg+Alt+T** (Menü **Ansicht ▸ Reiter und Teilen ▸ Editor teilen / Teilung aufheben**, oder der Knopf ◫ am Reiterstreifen). Der Editorbereich teilt sich in zwei Hälften.
3. Schicke einen Reiter in die andere Hälfte: Rechtsklick auf den Reiter ▸ **Reiter in die andere Hälfte schicken** (auch im Menü **Ansicht**), oder ziehe den Reiter hinüber.
4. Derselbe Befehl **Editor teilen / Teilung aufheben** hebt die Teilung wieder auf. Teilen kannst du den Editor nur einmal.

![Der Editor ist geteilt: links music.inc, rechts jumprun.asm, jeweils mit eigener Reiterleiste.](bild:editor-zweiter-bereich "Der zweite Editorbereich: zwei Dateien nebeneinander")

## Zusammenfassung der Kürzel

| Aktion | Mac | Windows |
|---|---|---|
| Speichern | Cmd+S | Strg+S |
| Rückgängig / Wiederholen | Cmd+Z / Cmd+Umschalt+Z | Strg+Z / Strg+Umschalt+Z |
| Suchen / Ersetzen | Cmd+F / Cmd+Alt+F | Strg+F / Strg+Alt+F |
| Gehe zu Zeile | Cmd+L | Strg+L |
| Include einfügen | Datei aus dem Projektbaum ziehen, oder Rechtsklick ▸ „Als Include einfügen“ | dasselbe |
| In Projekt suchen | Cmd+Umschalt+F | Strg+Umschalt+F |
| Hilfe zum Wort | F1 | F1 |
| Gehe zu Definition | F12 | F12 |
| Zeichen-Panel | Symbol Ω im rechten Panel | dasselbe |
| Editor teilen | Cmd+Alt+T | Strg+Alt+T |
| Datei schließen | Cmd+W | Strg+W |
| Alle speichern | Cmd+Alt+S | Strg+Alt+S |

Eine vollständige Tabelle steht in der Hilfeseite [Tastenkürzel](hilfe:shortcuts); die Einzelheiten zum Editor in der Hilfeseite [Editor](hilfe:editor).

## Häufige Stolpersteine

- **Die Vervollständigung erscheint nicht.** Sie öffnet sich beim Tippen, sobald das Wort zwei Zeichen lang ist (ein Punkt zählt mit). Ein Kürzel zum Öffnen von Hand gibt es nicht. Tippe weiter oder lösche das letzte Zeichen und tippe es neu.
- **Enter fügt einen Vorschlag ein, obwohl ich eine neue Zeile wollte.** Solange die Liste offen ist, übernimmt Enter den Eintrag. Drücke vorher **Esc**.
- **F1 zeigt nichts Brauchbares.** Der Cursor muss **auf** einem Wort stehen, zum Beispiel einem Befehl oder einer Direktive.
- **Die Zeile hat sich nach dem Verlassen verändert.** Das ist das Ausrichten. Das Ausrichten ist gewollt; mit Cmd/Strg+Z machst du es rückgängig.
- **Ein Klick im Zeichen-Panel fügt nichts ein.** Es ist keine Quelltext-Datei geöffnet, oder an der Cursorstelle passt das Zeichen nicht. Öffne eine Datei und setze den Cursor; ein Hinweis steht im Protokoll.
- **Ich habe einen gefalteten Abschnitt „verloren“.** Klicke in der Rinne auf den Pfeil **▸** an der Zeile, unter der die Zeilen fehlen.

## Wie geht es weiter?

Jetzt schreibst du Quelltext komfortabel. Was die einzelnen Zeilen eines Assembler-Programms bedeuten, lernst du im nächsten Kapitel: [Assembler (VCASM) für Einsteiger](kapitel:assembler). Wenn du lieber mit BASIC beginnst, geht es mit [BASIC in der IDE](kapitel:basic) weiter.
