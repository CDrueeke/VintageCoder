# Editor

Der Editor hebt Assembler und BASIC farbig hervor.

## Vervollständigung
Beim Tippen erscheint, sobald das Wort zwei Zeichen lang ist (ein Punkt zählt mit, etwa in `vic2.`), eine Liste: Befehle, Direktiven, Chip-Register (`vic2.border`), Labels, Konstanten und Makros – auch aus eingebundenen Dateien. **Tab** übernimmt den Eintrag, mit **↑/↓** und **Enter** wählst du einen anderen, **Esc** schließt. Ein Kürzel, das die Liste von Hand öffnet, gibt es nicht.

## Hilfe zum Wort
**F1** zeigt die Hilfe zum Wort unter dem Cursor: bei einem Befehl alle Adressierungsarten mit Opcode, Bytes und Zyklen, bei Direktiven die Syntax, bei Registern Adresse und Bedeutung. Ein eigener Reiter „Referenz“, der live mit dem Cursor mitläuft, ist noch nicht enthalten.

## Markdown-Dateien
Dateien mit der Endung `.md` oder `.markdown` (Notizen, `README.md`, Handbuchtexte) zeigt der Editor mit Markdown-Farben: Überschriften, **Fett**, *Kursiv*, `Code`, Listen, Zitate, Links und Tabellen. Die Zeichen `#`, `**` und `` ` `` bleiben sichtbar (nur gedämpft), der Text ändert sich nicht.

**Vorschau.** Eine `.md`-Datei mit Inhalt öffnet als gerenderte Seite; über ihr steht eine Leiste mit dem Knopf **Quelltext**, im Quelltext führt der Knopf **Vorschau** zurück. Dasselbe tut **Ansicht ▸ Markdown: Vorschau / Quelltext umschalten** (ohne Kürzel, auch in der Befehlspalette). Eine Ansicht mit Quelltext und Vorschau nebeneinander ist noch nicht enthalten. Die Vorschau ist nur zum Lesen. Eine leere Datei öffnet gleich im Editor.

**Was die Vorschau kann:** CommonMark plus Tabellen, Aufgabenlisten (`- [x]`) und Durchgestrichen (`~~text~~`). Rohes HTML wird nie ausgeführt, sondern als Text gezeigt; Front Matter (`---` … `---` am Dateianfang) ist ausgeblendet. Fußnoten, Mathe, Diagramme und Emoji-Kürzel gibt es nicht.

**Links und Bilder.** Links in der Vorschau sind farbig und unterstrichen; Links auf Hilfethemen (`hilfe:…`) öffnen das Thema, alle anderen öffnen nichts von allein. Bilder erscheinen als Platzhaltertext.

**Schreibhilfen** (Menü Bearbeiten, Einträge „Markdown: …“, und Befehlspalette): Fett (ohne Kürzel; Strg/Cmd+Umschalt+B ist der Zen-Modus), Kursiv (Strg/Cmd+I), Code (Strg/Cmd+E), Durchgestrichen (Strg/Cmd+Umschalt+X), Link (Strg/Cmd+Umschalt+K; die Adresse ist markiert), Bild einfügen, Überschriftsebene ändern, Aufzählung, Nummerierung, Aufgabenliste, Zitat, Aufgabe abhaken, Tabelle einfügen und ausrichten. Jeder Befehl ist ein Schritt für Rückgängig. **Enter** am Ende eines Listenpunkts legt den nächsten an, auf einem leeren Punkt beendet es die Liste; **Tab** und **Umschalt+Tab** rücken Listenpunkte ein und aus; in einer Tabelle springt **Tab** zur nächsten Zelle (am Ende entsteht eine neue Zeile). Das Fortsetzen von Listen schaltest du in den Einstellungen ab.

**Gliederung.** Der Reiter „Gliederung“ im rechten Panel zeigt die Überschriften mit Ebenen; Klick springt zur Zeile, der Pfeil klappt Unterpunkte ein. Menü Bearbeiten ▸ Markdown ▸ „Datei als Markdown behandeln / als Text“ macht aus einer `.txt` oder einem unbenannten Reiter (nur für diesen Reiter) eine Markdown-Datei.

## Falten
Pfeile in der Zeilennummernspalte falten Blöcke (`.macro`, `.if`, `.repeat`, `while`, `for` …), Label-Abschnitte und Kommentarblöcke; ein Klick auf **▾** klappt zu, ein Klick auf **▸** wieder auf. Zum Bearbeiten wird automatisch entfaltet. **Alles falten** und **Alles entfalten** (Bearbeiten, Befehlspalette, Rechtsklick) klappen alle Abschnitte auf einmal zu oder auf (ohne Kürzel).

## Weitere Bedienung
- Klick auf eine Zeilennummer setzt einen Haltepunkt (nach dem Bauen); **F9** schaltet ihn in der Cursorzeile um.
- **F12** (Bearbeiten ▸ Suchen und Ersetzen ▸ Gehe zu Definition) springt zur Definition des Symbols unter dem Cursor, auch in andere Dateien (nur Assembler). Ein Klick mit Strg/Cmd und ein Doppelklick auf `.include`-Namen sind noch nicht belegt. Der Mauszeiger über einem Wort zeigt die Beschreibung.
- **Werte im Hover:** Bleibt der Mauszeiger kurz über einer **Konstante** (Assembler: `.const`, Label, Zuweisung; BASIC: `const NAME = zahl`) oder einem **Zahlenliteral** (`$d020`, `%1010`, `255`, `'A'`), erscheint der Wert als Tabelle: dezimal, hexadezimal, binär, mit Vorzeichen (8/16 Bit), als Zeichen sowie Low- und High-Byte. Bei einer Adresse, die ein Chip-Register ist (z. B. `$d020`), steht zusätzlich der Registername dabei. Im Assembler zeigt der Hover außerdem Beschreibungen zu Befehlen, Direktiven und Registern.
- **Strg/Cmd+S** speichert.

## Dateien ins Projekt ziehen (Text, Quelltext, Binärdaten)

**Konzept:** Jede Datei im Projekt hat eine *Art* (was sie ist) und eine *Kategorie* (wozu sie dient). Die Art erkennt VintageCoder an Endung und Inhalt (Assembler- oder BASIC-Quelltext, Text, Binärdaten, PRG, Zeichensatz-Rohdaten, Grafik-Rohdaten, Bild, Diskettenimage, Studio-Datei). Die Kategorie – *Quelltext*, *Text / Doku*, *Binärdaten*, *Grafikrohdaten*, *Zeichensatz (roh)* oder *Studio* – wählst du; der Vorschlag kommt aus der Art. Sie steht mit einer Beschreibung im Abschnitt `assets` der `.vcprj`. Dateien werden immer ins Projekt **kopiert**, damit es in sich geschlossen bleibt und sich verschieben lässt; `vcc` baut sie ohne die Oberfläche.

**So geht es:** Dateien aus Finder oder Explorer auf das Fenster ziehen (mehrere auf einmal, auch ganze Ordner). Ist im Projektbaum ein Ordner gewählt, ist er das Ziel, sonst der Projektordner; im Dialog kannst du den Ordner ändern.

Der Dialog zeigt eine Zeile je Datei: Haken (übernehmen), Name, erkannte Art, Kategorie, Ordner und Ziel. Gibt es den Namen schon (mit anderem Inhalt), wählst du *Umbenennen*, *Ersetzen* oder *Überspringen*; gleicher Inhalt wird nicht noch einmal kopiert. Abbrechen übernimmt nichts. Liegt eine Datei schon im Projektordner, wird sie nur eingetragen. Wo Dateien je Kategorie landen, steht im Abschnitt `layout` der `.vcprj`.

**Beispiel Zeichensatz:** `font.bin` (2 KiB) ablegen, Kategorie *Zeichensatz (roh)*, Ordner `data` – im Quelltext dann `.incbin "../data/font.bin"` (Pfad relativ zur Quelle). Schneller: Rechtsklick auf die Datei im Projektbaum ▸ **Als Include einfügen** (unter dem Cursor des aktiven Quelltexts) oder die Datei aus dem Baum in den Editor ziehen (siehe unten).

**Include-Pfade bleiben ausdrücklich:** Der Assembler sucht eine eingebundene Datei immer relativ zur Quelle, den Ordner schreibst du hin (`.include "hallo/x.asm"`); es gibt keine Suchpfade. Zieht eine eingebundene Datei um, zeigen die Pfade in den Quelltexten ins Leere, bis du sie anpasst. Auf der Kommandozeile gibt es `vcc organize` zum Neuordnen mit Pfadnachführung.

**Auf der Kommandozeile:** `vcc add <datei|ordner>… [--into projekt] [--category …] [--dir ordner] [--description TEXT] [--conflict rename|replace|skip] [--flat] [--dry-run]` übernimmt Dateien wie das Ablegen in der Oberfläche: Art erkennen, nach `rules`/`layout` einsortieren, in `assets` eintragen; Ordner rekursiv mit ihrer Struktur (`--flat`: jede Datei in den Ordner ihrer Kategorie). `vcc import` bleibt für Diskettenimages und PRG/XEX.

**Öffnen:** Textdateien öffnen im Editor. Binärdateien siehst du über Rechtsklick ▸ **Als Hex ansehen (nur Lesen)**.

## Include per Drag & Drop

Eine Datei aus dem **Projektbaum** an eine Stelle im Quelltext ziehen: Beim Ziehen zeigt eine farbige Linie, wo die Zeile entsteht (obere Hälfte einer Zeile: davor, untere Hälfte: dahinter). Beim Loslassen wird die passende Zeile eingefügt; die Statusleiste nennt sie, Cmd/Strg+Z nimmt sie als eine Aktion zurück. Geht es nicht (Ordner, die Datei selbst, Projektdatei, Diskettenimage, falscher Quelltext), steht der Grund in der Statusleiste und es wird nichts eingefügt. Ist die Datei schon eingebunden, springt der Cursor zur vorhandenen Zeile. Dasselbe ohne Maus: Rechtsklick auf die Datei ▸ **Als Include einfügen** (unter die Cursorzeile des aktiven Quelltexts).

| Datei | im Assembler | in BASIC (`.bas`) |
|---|---|---|
| Assembler-Quelltext (`.asm`, `.inc`, `.s`) | `.include "pfad"` (für „nur einmal einbinden“ schreibst du `#include` von Hand) | nicht möglich |
| BASIC-Quelltext mit derselben Endung | nicht möglich | `#include "pfad"` |
| Sprite, Zeichensatz, Bildschirm, Song … | `.include "…/build/export/name.asm"` | `#include "…/build/export/name.data.bas"` |
| Sonstige Dateien (`.bin`, `.prg`, `.txt` …) | `.incbin "pfad"` (bei `.prg` ohne Ladeadresse von Hand `, 2` anhängen) | nicht möglich |

Der Pfad ist relativ zur Quelldatei, mit `/`, auch bei Leerzeichen und Umlauten. Nach dem Einfügen nennt die Statusleiste die Zeile, wo es eine gibt mit der Alternative von Hand; Cmd/Strg+Z nimmt sie zurück. Ist die Datei schon eingebunden, fügt VintageCoder nichts ein und markiert die vorhandene Zeile. Abgelehnt (mit Hinweis) werden: eine Datei in sich selbst, Ordner, Projektdatei, Diskettenimages, Dateien außerhalb des Projekts, Namen mit `"` und alles, was die Sprache nicht kennt (BASIC hat kein `.incbin`).

## Zeichen einfügen (PETSCII und ATASCII)

Zeichen und Steuerzeichen des Zielsystems (aus `.target`, sonst Projekt/Target) fügst du über das **Zeichen-Panel** ein: im rechten Panel auf das Symbol **Ω** klicken. Die Zeichen stammen aus den ROMs des Anwenders (Hilfe „ROM-Dateien“); ohne ROM nimmt VintageCoder den eigenen Standardsatz. Ein eigener Zeichenwähler-Dialog mit Raster ist noch nicht enthalten.

Das Panel arbeitet auch im **BASIC-Editor**. Es fügt dort die Namen des BASIC-Dialekts ein (`{clr}`, `{down*3}`).

### Steuerzeichen im Quelltext

`{clr}`, `{rvs on}`, `{$93}` in Zeichenketten (BASIC, und im Assembler bei `.petscii`, `.atascii`, `.screen`, `.text`, `.null`, `.ptext`, `.shift`) erkennt der Editor:

- **Farbe:** gültige Angaben in einer eigenen Farbe, Farbcodes (`{red}`, `{orng}`) als Farbfeld, unbekannte Namen rot und unterschlängelt (Meldung `VC-CHAR` mit dem Namen).
- **Hover:** Name, Wirkung, Code (hex und dezimal), weitere Schreibweisen.
- **Vervollständigung:** Nach `{` in einer Zeichenkette erscheint die Liste der Namen (mit Code und Wirkung; `_` und `-` gelten wie Leerzeichen). Pfeile wählen, Enter oder Tab übernimmt und schließt die Klammer.

### Panel „Zeichen“ (rechtes Panel)

Im rechten Panel listet der Reiter **Zeichen** (Symbol Ω) die Steuerzeichen des Zielsystems mit Name, Code und Wirkung: Farben, Cursor, Bildschirm und Bearbeiten, Zeichensatz, Funktionstasten. Er klappt mit dem rechten Panel ein und aus (Cmd/Strg+Alt+B); ein eigener Menübefehl oder ein Kürzel nur für das Zeichen-Panel ist nicht vorhanden.

- **Klick** fügt den Code an der Cursorstelle der zuletzt bearbeiteten Datei ein; der Editor behält den Fokus, **Cmd/Strg+Z** nimmt die Einfügung zurück.
- **Form nach Cursorstelle:** in einer Zeichenkette `{clr}`; im BASIC nach `PRINT` oder `?` mit Anführungszeichen (`"{clr}"`), in `CHR$(` die Zahl (`147`), sonst die Zahl; im Assembler außerhalb von Zeichenketten der Wert (`$93`). Passt an der Stelle nichts, steht ein Hinweis im Protokoll und nichts wird eingefügt.
- Im BASIC stehen die Namen des BASIC-Dialekts (`orng`, `rvon`), im Assembler die des Assemblers (`orange`, `rvs on`); beide Schreibweisen führen zum selben Code.
- **Suche:** Das Feld über der Liste findet Namen und Beschreibungen (`rvs`, `türkis`), Zahlen (`$93`, `147`) und getippte Zeichen. Bei einer Suche erscheinen auch gewöhnliche Zeichen mit ihrem Code.
- **Noch nicht enthalten:** Formenmenü per Rechtsklick oder Alt+Klick, Umschalt+Klick für die Zahl, Ziehen von Zeichen in den Editor, die Ansicht *Meine* (zuletzt benutzt, Favoriten) und die Tabelle mit allen 256 Zeichen.

## Zeile beim Verlassen formatieren
Verlässt du eine **bearbeitete** Zeile (Cursor in eine andere Zeile, Enter, Mausklick), wird sie ausgerichtet. Das gilt für Assembler und BASIC. Rückgängig machst du es wie jede Eingabe mit Strg/Cmd+Z. Abschalten: **Einstellungen ▸ Editor ▸ Zeile beim Verlassen formatieren**. Auf Befehl (auch bei abgeschaltetem Verlassen-Formatieren): **Bearbeiten ▸ Formatieren ▸ Zeile formatieren** und **Datei formatieren** (die ganze Datei als ein Undo-Schritt). Bei Auswahl, offener Vervollständigung und in den Studios wird nie von selbst formatiert.

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
**Im Editor** setzt der Rechtsklick den Cursor an die Klickstelle (außer innerhalb einer Auswahl) und bietet: Ausschneiden, Kopieren, Einfügen, **Alles auswählen** (auch Cmd/Strg+A), **Hilfe zum Wort** (F1), **Gehe zu Definition** (F12), **Alles falten** und **Alles entfalten**, **Haltepunkt umschalten** (F9) und **Debuggen: Bis hierher ausführen** (nur in einer Debug-Sitzung). Auskommentieren ganzer Zeilen ist noch nicht enthalten.
**Im Projektbaum** (nur hier und in der Befehlspalette, nicht im Menü Datei): bei Dateien Öffnen, Umbenennen …, Löschen …, Als Include einfügen, Pfad kopieren, Im Ordner zeigen, Als Hex ansehen (nur Lesen), dazu Neue Datei …, Neuer Ordner … und Neue Notiz …; auf freier Fläche des Baums die drei Neu-Einträge. Ein Knopf **＋** über dem Baum legt eine Datei im gewählten Ordner an (Dialog „Neue Datei“ mit Art, Profil und Name).
**Diskettenimages:** `.d64`, `.d71`, `.d81` und `.atr` öffnen im Disketten-Studio (Verzeichnis ansehen, prüfen, Dateien ablegen und ins Projekt übernehmen). Eine leere Diskette legst du mit dem Knopf **＋** über dem Baum an (Art „Diskette“). Auf der Kommandozeile gibt es `vcc import`.
**Auf einem Reiter:** Datei schließen, Reiter nach links oder rechts verschieben, Reiter in die andere Hälfte schicken, Editor teilen bzw. Teilung aufheben.

## Suchen, Ersetzen, Gehe zu Zeile
**Bearbeiten ▸ Suchen und Ersetzen ▸ Suchen …** (Strg/Cmd+F) öffnet über dem Text eine Leiste: Enter springt zum nächsten Treffer, Umschalt+Enter zum vorigen, „Aa“ beachtet die Groß-/Kleinschreibung, Esc schließt. **Ersetzen …** (Strg/Cmd+Alt+F) ergänzt ein Ersetzen-Feld mit „Ersetzen“ (aktuellen Treffer) und „Alle“. **Gehe zu Zeile …** (Strg/Cmd+L) springt zu einer Zeilennummer. **Rückgängig** und **Wiederholen** (Strg/Cmd+Z, Umschalt+Z) stehen auch im Menü. **Datei ▸ Exportieren …** öffnet den Export-Dialog des gerade aktiven Studios; Die **Startseite** (Datei ▸ Startseite) listet die zuletzt geöffneten Projekte.
