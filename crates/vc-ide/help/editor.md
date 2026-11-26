# Editor

Der Editor hebt Assembler und BASIC farbig hervor. Du findest ihn im Bereich **Code**: links der Projektbaum, in der Mitte die Dateien als Reiter, unten die Ablage mit Problemen, Ausgabe, Suche und Gliederung (siehe [Oberfläche anpassen](help:layout)).

## Vervollständigung
Beim Tippen erscheint, sobald das Wort zwei Zeichen lang ist (ein Punkt zählt mit, etwa in `vic2.`), eine Liste: Befehle, Direktiven, Chip-Register (`vic2.border`), Labels, Konstanten und Makros – auch aus eingebundenen Dateien. **Tab** übernimmt den Eintrag, mit **↑/↓** und **Enter** wählst du einen anderen, **Esc** schließt.

## Beschreibung zum Wort
Der Mauszeiger über einem Wort zeigt die Beschreibung: bei einem Befehl die Adressierungsarten mit Opcode, Bytes und Zyklen, bei Direktiven die Syntax, bei Registern Adresse und Bedeutung. **F1** schlägt das Wort unter dem Cursor (oder den markierten Text) in der Hilfe nach und öffnet das passende Thema.

## Markdown-Dateien
Dateien mit der Endung `.md` oder `.markdown` (Notizen, `README.md`) zeigt der Editor als **Vorschau** (nur zum Lesen); **Markdown: Vorschau / Quelltext umschalten** wechselt zum Quelltext. Schreibhilfen (Fett, Kursiv, Code, Link, Listen, Aufgaben, Zitat, Tabellen) stehen im Menü **Bearbeiten ▸ Markdown**, siehe [Markdown](help:markdown). Neue Notizen legst du über das Kontextmenü des Projektbaums (**Neue Notiz**) oder mit Cmd/Strg+Umschalt+T (eine unbenannte Notiz) an.

## Falten
Pfeile neben den Zeilennummern falten Blöcke (`.macro`, `.if`, `.repeat`, `while`, `for` …); ein Klick klappt zu oder wieder auf. **Alles falten** und **Alles entfalten** klappen alle Abschnitte auf einmal zu oder auf (Befehlspalette und Rechtsklick, ohne Kürzel).

## Weitere Bedienung
- Klick auf die schmale Spalte neben den Zeilennummern setzt einen Haltepunkt; **F9** schaltet ihn in der Cursorzeile um. Der Haltepunkt wirkt beim Debuggen, siehe [Haltepunkte](help:breakpoints).
- **F12** (Bearbeiten ▸ Suchen und Ersetzen ▸ Gehe zu Definition) springt zur Definition des Symbols unter dem Cursor, auch in andere Dateien (nur Assembler).
- **Cmd/Strg+.** bietet Korrekturen zu einer Meldung an.
- **Lesezeichen** setzt du mit Cmd/Strg+Alt+K; Cmd/Strg+Alt+J springt zum nächsten, Cmd/Strg+Alt+H zum vorigen.
- **Kommentar umschalten:** Cmd/Strg+Alt+C.
- **Cmd/Strg+S** speichert die Datei, **Cmd/Strg+Alt+S** alle geänderten Dateien.

## Dateien ins Projekt ziehen
Dateien aus Finder oder Explorer auf das Fenster ziehen: Eine Fläche „Dateien hier ablegen, um sie zu öffnen“ erscheint. Textdateien aus dem Projekt öffnen gleich; liegt eine Datei außerhalb des Projekts, fragt VintageCoder „Ins Projekt kopieren und öffnen?“ (**Kopieren und öffnen**). Eine Projektdatei (`.vcprj`) öffnet das Projekt, ein Diskettenabbild öffnet im [Disk-Editor](help:disks). Ohne geöffnetes Projekt lassen sich Dateien nicht ablegen.

## Zeile beim Verlassen formatieren
Verlässt du eine **bearbeitete** Zeile (Cursor in eine andere Zeile, Enter, Mausklick), wird sie ausgerichtet. Das gilt für Assembler und BASIC. Rückgängig machst du es wie jede Eingabe mit Cmd/Strg+Z. Abschalten kannst du es unter **Einstellungen ▸ Editor** (**Zeile beim Verlassen formatieren**). Auf Befehl (auch bei abgeschaltetem Verlassen-Formatieren): **Bearbeiten ▸ Formatieren ▸ Zeile formatieren** und **Datei formatieren** (die ganze Datei als ein Undo-Schritt). Bei Auswahl, offener Vervollständigung und in den Editoren wird nie von selbst formatiert.

**Assembler**
- Label (`name:`) steht in Spalte 0, dahinter die Mnemonik in Spalte 8 (bei langen Labels nach einem Leerzeichen); Befehle ohne Label werden auf Spalte 8 eingerückt.
- Mnemonik und bekannte Direktiven werden klein geschrieben (`LDA` → `lda`, `.ORG` → `.org`); Makroaufrufe und andere Namen bleiben unverändert.
- Zwischen Mnemonik und Operand steht ein Leerzeichen, Mehrfachleerzeichen werden zusammengefasst (Strings und Zeichen wie `' '` bleiben).
- Ein Kommentar hinter dem Code beginnt in Spalte 40 (ist der Code länger, nach einem Leerzeichen). Reine Kommentar- und Leerzeilen sowie `{` und `}` ändern nur den rechten Rand. Direktiven ohne Label und Zuweisungen behalten ihre Einrückung.

**BASIC (`.bas`, `.lst`)**
- Ein Leerzeichen hinter der Zeilennummer, Mehrfachleerzeichen außerhalb von Strings zusammengefasst; der Text hinter `REM` bleibt unverändert. Keine Einrückung.

## Zeichenkodierung und Zeilenenden

Gespeichert wird immer **UTF-8 ohne BOM** mit **LF**. Beim Öffnen fremder Dateien (BOM, CRLF, Windows-1252) erscheint ein Hinweis in der Statusleiste; die Datei wird erst beim Speichern umgeschrieben. Dateien mit Nullbytes werden als Binärdateien nicht angezeigt.

## Kontextmenü (rechte Maustaste)
**Im Editor** setzt der Rechtsklick den Cursor an die Klickstelle (außer innerhalb einer Auswahl) und bietet: Ausschneiden, Kopieren, Einfügen, Alles auswählen, Gehe zu Definition, Korrigieren, Kommentar umschalten, Gehe zu Zeile, Lesezeichen umschalten, Alles falten und Alles entfalten und Haltepunkt umschalten; in BASIC-Dateien dazu die Umstellung zwischen Zeilennummern und Labels.
**Im Projektbaum** und auf einem **Reiter** gibt es eigene Menüs, siehe [Dateibaum](help:filetree). Das Reitermenü bietet Schließen, Andere schließen, Alle schließen, nach links und rechts verschieben und Pfad kopieren. **Cmd/Strg+Alt+T** teilt den Editor in zwei Hälften.

## Suchen, Ersetzen, Gehe zu Zeile
**Bearbeiten ▸ Suchen und Ersetzen ▸ Suchen …** (Cmd/Strg+F) öffnet über dem Text eine Leiste: Enter springt zum nächsten Treffer, Umschalt+Enter zum vorigen, Esc schließt. **Ersetzen …** (Cmd/Strg+Alt+F) ergänzt ein Ersetzen-Feld. **Gehe zu Zeile …** (Cmd/Strg+L) springt zu einer Zeilennummer. **In Projekt suchen …** (Cmd/Strg+Umschalt+F) sucht in allen Dateien; die Treffer stehen im Reiter **Suche** der Ablage. **Rückgängig** und **Wiederholen** (Cmd/Strg+Z, Cmd/Strg+Umschalt+Z) stehen auch im Menü. **Datei ▸ Startseite** zeigt die zuletzt geöffneten Projekte.
