# Editor

Der Editor hebt Assembler und BASIC farbig hervor.

## Vervollständigung
Beim Tippen erscheint nach zwei Zeichen (nach `.` oder `@` sofort) eine Liste: Befehle, Direktiven, Chip-Register (`vic2.border`), Labels, Konstanten und Makros – auch aus eingebundenen Dateien. **Tab** übernimmt den Eintrag, mit **↑/↓** und **Enter** wählst du einen anderen, **Esc** schließt. **Strg/Cmd+Leertaste** öffnet die Liste von Hand.

## Hilfe zum Wort
**F1** zeigt die Hilfe zum Wort unter dem Cursor: bei einem Befehl alle Adressierungsarten mit Opcode, Bytes und Zyklen, bei Direktiven die Syntax, bei Registern Adresse und Bedeutung. Das Panel „Referenz“ zeigt dasselbe live beim Bewegen des Cursors.

## Falten
Pfeile in der Zeilennummernspalte falten Blöcke (`.macro`, `.if`, `.repeat`, `while`, `for` …), Label-Abschnitte und Kommentarblöcke. **Strg/Cmd+Alt+[** faltet, mit Shift alles; **Strg/Cmd+Alt+]** entfaltet alles. Zum Bearbeiten wird automatisch entfaltet.

## Weitere Bedienung
- Klick auf eine Zeilennummer setzt einen Haltepunkt (nach dem Bauen). 
- **Strg/Cmd+Klick** auf ein Symbol springt zur Definition; Mauszeiger darüber zeigt die Beschreibung.
- **Werte im Hover:** Bleibt der Mauszeiger kurz über einer **Konstante** (Assembler: `.const`, Label, Zuweisung; BASIC: `const NAME = zahl`) oder einem **Zahlenliteral** (`$d020`, `%1010`, `255`, `'A'`), erscheint der Wert als Tabelle: dezimal, hexadezimal, binär, mit Vorzeichen (8/16 Bit), als Zeichen sowie Low- und High-Byte. Bei einer Adresse, die ein Chip-Register ist (z. B. `$d020`), steht zusätzlich der Registername dabei. Im Assembler zeigt der Hover außerdem Beschreibungen zu Befehlen, Direktiven und Registern.
- **Strg/Cmd+S** speichert.

## Dateien ins Projekt ziehen (Text, Quelltext, Binärdaten)

**Konzept:** Jede Datei im Projekt hat eine *Art* (was sie ist) und eine *Kategorie* (wozu sie dient). Die Art erkennt VintageCoder an Endung und Inhalt (Assembler- oder BASIC-Quelltext, Text, Binärdaten, PRG, Zeichensatz-Rohdaten, Grafik-Rohdaten, Bild, Diskettenimage, Studio-Datei). Die Kategorie – *Quelltext*, *Text / Doku*, *Binärdaten*, *Grafikrohdaten* oder *Zeichensatz (roh)* – wählst du; der Vorschlag kommt aus der Art. Sie steht mit einer Beschreibung im Abschnitt `assets` der `.vcprj` und bestimmt die Marke im Projektbaum (`BIN`, `TXT`). Dateien werden immer ins Projekt **kopiert**, damit es in sich geschlossen bleibt und sich verschieben lässt; `vcc` baut sie ohne die IDE.

**So geht es:** Dateien aus Finder oder Explorer auf das Fenster ziehen (mehrere auf einmal). Über dem **Projektbaum** ist der Ordner unter dem Mauszeiger das Ziel (auf freier Fläche die Projektwurzel); sonst schlägt der Dialog je Kategorie einen Ordner vor (`lib/`, `doc/`, `data/`, `graphics/`). Über einem **Assembler-Quelltext** fügt VintageCoder nach dem Übernehmen zusätzlich `.include` bzw. `.incbin` an der Cursorposition ein.

Der Dialog zeigt eine Zeile je Datei: Haken (übernehmen), Name, erkannte Art, Kategorie, Ordner und Ziel. Gibt es den Namen schon (mit anderem Inhalt), wählst du *Umbenennen* (`_2`, `_3` …), *Ersetzen* oder *Überspringen*; gleicher Inhalt wird nicht noch einmal kopiert. **Für alle übernehmen** gibt Kategorie, Ordner und Konfliktlösung der ersten Datei an alle weiter. Liegt eine Datei schon im Projektordner, wird sie nur eingetragen.

**Beispiel Zeichensatz:** `font.bin` (2 KiB) ablegen, Kategorie *Zeichensatz (roh)*, Ordner `data` – im Quelltext dann `.incbin "../data/font.bin"` (Pfad relativ zur Quelle). Schneller: Rechtsklick auf die Datei im Projektbaum ▸ **Als Include einfügen** (unter dem Cursor des aktiven Quelltexts) oder die Datei aus dem Baum in den Editor ziehen (siehe unten).

**Ganze Ordner:** Ein Ordner lässt sich wie eine Datei ablegen. VintageCoder legt ihn samt Unterordnern im Dialog an (versteckte Dateien, `target` und `node_modules` bleiben draußen, höchstens 5000 Dateien auf einmal). Ab 40 Dateien zeigt der Dialog zuerst eine Zusammenfassung (Dateien je Kategorie, Namenskonflikte mit einer gemeinsamen Lösung); *Einzelne Dateien anzeigen* öffnet die Liste (die ersten 300). „Ordnerstruktur abgelegter Ordner beibehalten“ (Vorgabe) übernimmt die Unterordner unverändert unter dem Zielordner; ohne den Haken gilt der Ordner der Kategorie. Kategorie und Art bestimmt der Dialog weiter je Datei.

**Projektstruktur:** Wo Dateien je Kategorie landen, steht im Abschnitt `layout` der `.vcprj` (Kategorie → Ordner), z. B. `"layout": {"binary": "res/bin", "source": "src/lib", "charset": "res/fonts"}`. Fehlt ein Eintrag, gilt `lib/`, `doc/`, `data/` bzw. `graphics/`. Im Projektbaum legen *+ ▸ Ordner …* und *Neuer Ordner hier …* (auch verschachtelt wie `res/bin/fonts`) Ordner an; leere Ordner bleiben sichtbar. Der Baum zeigt bis zu 16 Ebenen.

**Einstellen:** In den Projekt-Einstellungen (Doppelklick auf die `.vcprj`) gibt es den Abschnitt **Projektstruktur**: eine Tabelle Kategorie → Zielordner mit der Zahl der Dateien je Kategorie, darunter **Regeln** nach Dateimuster (`*` und `?`, ohne Groß-/Kleinschreibung), z. B. `*.sid` → Binärdaten in `res/music`. Die erste passende Regel gilt vor der Kategorie; ohne Kategorie in der Regel bleibt die erkannte Art. „Vorlage laden …“ übernimmt die Zielordner einer Strukturvorlage, „Ordner anlegen“ erzeugt fehlende Zielordner, „Als Vorlage speichern“ legt Ordner, Zielordner und Rollen des Projekts als eigene Vorlage ab (Datei `structures.json` im Konfigurationsordner neben `settings.json`; im Wizard erscheint sie hinter den mitgelieferten). Darunter zeigt die *Vorschau* die vorhandenen und vorgesehenen Ordner mit Rolle und Zuordnung. Ungültige Ordner (absolut oder mit `..`) erscheinen rot und als Achtung oben; in der Projektdatei stehen sie als `layout` und `rules`.

**Ordner im Projektbaum:** Neben jedem Ordnernamen steht, falls vergeben, seine **Rolle**; zugeklappte Ordner zeigen zusätzlich die Zahl der Dateien darin (mit Unterordnern). Beim Ziehen von Dateien über einen zugeklappten Ordner klappt er nach einer knappen Sekunde auf. Die Rolle – *Quelltext*, *Ressourcen*, *Daten*, *Doku*, *Tests* oder *Werkzeuge*. Rechtsklick auf den Ordner: **Ordner-Rolle** setzt oder entfernt sie, **Als Ziel festlegen für …** macht den Ordner zum Zielordner einer Kategorie (trägt `layout` ein). Rollen stehen im Abschnitt `folders` der `.vcprj`, die Strukturvorlagen des Wizards setzen sie mit. Sie dienen nur der Anzeige und den Vorschlägen; der Build ignoriert sie.

**Include-Pfade bleiben ausdrücklich:** Der Assembler sucht eine eingebundene Datei immer relativ zur Quelle, den Ordner schreibst du hin (`.include \"hallo/x.asm\"`); es gibt keine Suchpfade. Zieht eine eingebundene Datei um oder wird sie umbenannt, fragt VintageCoder, ob die Pfade in den Quelltexten angepasst werden sollen: beim *Umbenennen* mit Häkchen „Code anpassen“ (samt Liste der Quelltexte), bei *Dateien neu ordnen* mit demselben Häkchen im Dialog. Ohne Anpassung zeigen die Pfade ins Leere.

**Dateien neu ordnen:** Menü „⋯“ der Seitenleiste ▸ *Dateien neu ordnen …* (oder `vcc organize`) sortiert ein bestehendes Projekt nach der Projektstruktur. Der Dialog zeigt je Datei Quelle und Ziel (abwählbar); Dateien im Projektordner werden verschoben, mit *Auch Dateien in Unterordnern* auch die darin. Ziel ist der Ordner aus `rules`, sonst `layout` für die Kategorie (aus `assets` oder erkannt). **Bleiben:** Programme aus den Quellgruppen, Startprogramm, Disketten-Dateien, Studio-Dateien, `.vctest`, die Projektdatei und README/LICENSE/CHANGELOG sowie Dateien, die schon im Zielordner liegen. Ist das Ziel belegt, wird die Datei übersprungen. *Studio-Dateien* wandern nur, wenn `layout` einen Ordner für `studio` nennt (Zeile „Studio-Dateien“ in der Tabelle), `.vctest`-Dateien nur in einen Ordner mit der Rolle *Tests* (die Testläufer suchen das Programm von der Testdatei aus nach oben, `build/…` bleibt also gültig). Beim Verschieben werden Pfade in Zeichenketten der Quelltexte (`.include`, `.incbin`, `#include`), die Projektdatei und `assets` nachgeführt – auch die Pfade in Quelltexten, die selbst umziehen. Vorher entsteht eine Sicherung in `.vcbackup/<Zeit>/` (geänderte Texte, Projektdatei, `moves.txt`). Es bleiben die letzten fünf Sicherungen. Der **Prüfbau** läuft im Hintergrund (mit Fortschrittsanzeige) – er baut vor und nach dem Verschieben und vergleicht die Ergebnisdateien Byte für Byte; weichen sie ab, bietet der Dialog **Rückgängig** an. Auf der Kommandozeile: `vcc organize [ordner] [--yes] [--dry-run] [--subdirs] [--no-rewrite] [--no-check]` – ohne `--yes` fragt es im Terminal nach (erst „Verschieben?“, dann „Pfade anpassen?“) und ändert ohne Terminal nichts.

**Mehrere Systeme:** Ein Ordner lässt sich auf ein Target beschränken (Rechtsklick ▸ *Nur für Target …*; die Vorlage „Mehrere Systeme“ setzt das für `src/<system>` und `res/<system>`). Der Baum zeigt „nur C64“ am Ordner, und *Dateien neu ordnen* lässt Dateien darin an ihrem Platz. Welche Quellen für welches System gebaut werden, bestimmen weiter die Quellgruppen (`sources`); `folders` mit `targets` ist Anzeige und Schutz vor dem Umordnen. Die Prüfung meldet Targets, die das Projekt nicht kennt.

**Auf der Kommandozeile:** `vcc add <datei|ordner>… [--into projekt] [--category …] [--dir ordner] [--description TEXT] [--conflict rename|replace|skip] [--flat] [--dry-run]` übernimmt Dateien wie das Ablegen in der IDE: Art erkennen, nach `rules`/`layout` einsortieren, in `assets` eintragen; Ordner rekursiv mit ihrer Struktur (`--flat`: jede Datei in den Ordner ihrer Kategorie). `vcc import` bleibt für Diskettenimages und PRG/XEX.

**Öffnen:** Textdateien öffnen im Editor, Binärdateien in einem schreibgeschützten Hexdump. Umbenennen und Löschen im Projektbaum führen den Eintrag in `assets` nach. `vcc build` warnt, wenn eine Datei aus `assets` fehlt.

## Include per Drag & Drop

Eine Datei aus dem **Projektbaum** an eine Stelle im Quelltext ziehen: Beim Ziehen zeigt eine Linie, wo die Zeile entsteht (zwischen den Zeilen unter dem Mauszeiger), und am Zeiger steht die Zeile selbst – oder in Rot, warum es nicht geht. Beim Loslassen wird sie eingefügt; Cmd/Strg+Z nimmt sie als eine Aktion zurück. Dasselbe, ohne Maus: Rechtsklick auf die Datei ▸ **Als Include einfügen** (unter dem Cursor des aktiven Quelltexts).

| Datei | im Assembler | in BASIC (`.bas`) |
|---|---|---|
| Assembler-Quelltext (`.asm`, `.inc`, `.s`) | `.include "pfad"` (Hinweisleiste: auf `#include` umschalten) | nicht möglich |
| BASIC-Quelltext mit derselben Endung | nicht möglich | `#include "pfad"` |
| Sprite, Zeichensatz, Bildschirm, Song … | `.include "…/build/export/name.asm"` | `#include "…/build/export/name.data.bas"` |
| Sonstige Dateien (`.bin`, `.prg`, `.txt` …) | `.incbin "pfad"` (bei `.prg` auf „ohne Ladeadresse“ umschaltbar) | nicht möglich |

Der Pfad ist relativ zur Quelldatei, mit `/`, auch bei Leerzeichen und Umlauten. Nach dem Einfügen bietet eine kleine Leiste **Rückgängig** und, wo es eine gibt, die Alternative an. Ist die Datei schon eingebunden, fügt VintageCoder nichts ein und markiert die vorhandene Zeile. Abgelehnt (mit Hinweis) werden: eine Datei in sich selbst, Ordner, Projektdatei, Diskettenimages, Dateien außerhalb des Projekts, Namen mit `"` und alles, was die Sprache nicht kennt (BASIC hat kein `.incbin`).

## Zeichen einfügen (PETSCII und ATASCII)

**Strg/Cmd+Umschalt+Y**, Rechtsklick → *Zeichen einfügen …* oder Menü *Bearbeiten* öffnet den Zeichenwähler für das Ziel der Datei (aus `.target`, sonst Projekt/Target). Die Zeichen stammen aus den ROMs des Anwenders (Hilfe „ROM-Dateien“); ohne ROM erscheint eine eingebaute Ersatzschrift. Beim C64, C128, Plus/4 und MEGA65 gibt es die Reiter **PETSCII** und **Bildschirmcode**, beim Atari **ATASCII** und **ANTIC-Code**.

- **Raster:** 16×16 Felder in Codereihenfolge. Blaue Kürzel sind Steuerzeichen (`CLR`, `HOME`, `RVS`, Farben, Cursor, F-Tasten; beim Atari `EOL`, `ESC` …). Rechts stehen Code, Bildschirmcode bzw. ANTIC-Code, Name und die Taste auf dem Original.
- **Auswählen:** Klick legt das Zeichen in die *Auswahl*, Umschalt+Klick einen Bereich, ein Doppelklick fügt es sofort ein. Entf nimmt das letzte Zeichen aus der Auswahl, Enter fügt ein, Esc schließt. Die Suche findet Codes (`$c1`, `193`) und Namen.
- **Einfügen als:**

| Art | Beispiel | Wann |
|---|---|---|
| In Zeichenkette | `{clr}HALLO{down*2}{$a1}` | Vorgabe, wenn der Cursor in einer Zeichenkette von `.petscii`, `.atascii`, `.screen` oder `.text` … steht. Buchstaben und Ziffern erscheinen direkt, Steuerzeichen als `{name}`, Grafikzeichen als `{$hh}`, gleiche Zeichen hintereinander als `{name*n}`. |
| Wert | `$c1, $93` | Vorgabe sonst. |
| Byte-Direktive | `.byte $c1, $93` | fügt eine ganze Zeile ein. |
| Dezimalzahl | `193, 147` | |
| Bildschirmcode / ANTIC-Code | `$41` | Code im Bildschirmspeicher. |

- **Dialog offen lassen** (Vorgabe): nach dem Einfügen bleibt der Dialog stehen und man kann weitere Zeichen holen. Die zuletzt benutzten Zeichen stehen unten.
- Die Schreibweise `{name}` ist unter [Texte und Zeichensätze](help:asm-text) beschrieben. Sie funktioniert nur im Assembler-Editor.

## Zeile beim Verlassen formatieren
Verlässt du eine **bearbeitete** Zeile (Cursor in eine andere Zeile, Enter, Mausklick), wird sie ausgerichtet. Das gilt für Assembler und BASIC. Rückgängig machst du es wie jede Eingabe mit Strg/Cmd+Z. Abschalten: **Fenster > Zeile beim Verlassen formatieren**.

**Assembler**
- Label (`name:`) steht in Spalte 0, dahinter die Mnemonik in Spalte 8 (bei langen Labels nach einem Leerzeichen); Befehle ohne Label werden auf Spalte 8 eingerückt.
- Mnemonik und bekannte Direktiven werden klein geschrieben (`LDA` → `lda`, `.ORG` → `.org`); Makroaufrufe und andere Namen bleiben unverändert.
- Zwischen Mnemonik und Operand steht ein Leerzeichen, Mehrfachleerzeichen werden zusammengefasst (Strings und Zeichen wie `' '` bleiben).
- Ein Kommentar hinter dem Code beginnt in Spalte 40 (ist der Code länger, nach einem Leerzeichen). Reine Kommentar- und Leerzeilen sowie `{` und `}` ändern nur den rechten Rand. Direktiven ohne Label und Zuweisungen behalten ihre Einrückung.

**BASIC (`.bas`, `.lst`)**
- Ein Leerzeichen hinter der Zeilennummer, Mehrfachleerzeichen außerhalb von Strings zusammengefasst; der Text hinter `REM` bleibt unverändert. Keine Einrückung.

## Kontextmenü (rechte Maustaste)
**Im Editor** setzt der Rechtsklick den Cursor an die Klickstelle (außer innerhalb einer Auswahl) und bietet: Ausschneiden, Kopieren, Einfügen, Alles auswählen; beim Assembler außerdem **Haltepunkt setzen/entfernen** (Zeile), im Debugger **Bis hierher ausführen** (setzt einen Haltepunkt und läuft weiter), **Zur Definition von »Wort«** (springt auch in andere Dateien); bei Assembler und BASIC **Hilfe zu »Wort«** (wie F1), bei `.include`-Zeilen **Datei öffnen**, **Zeile(n) auskommentieren / Kommentar entfernen** (Assembler und BASIC: `;`) sowie **Alles falten / entfalten**.
**Im Projektbaum**: bei Dateien Öffnen, Als Startprogramm festlegen, Nur diese Datei bauen, Umbenennen (die Projektdatei wird nachgeführt), Duplizieren, Löschen (in den Papierkorb; Rückfrage), Im Finder zeigen und Pfad kopieren; bei Ordnern Neue Datei hier, Im Finder zeigen und Pfad kopieren. In den Pixel- und Bildschirmeditoren löscht die rechte Maustaste weiterhin.
**Diskettenimages öffnen:** Rechtsklick auf eine `.d64`-, `.d71`-, `.d81`- oder `.atr`-Datei im Projektbaum → „Diskettenimage öffnen …“, oder der Knopf „Image …“ oben im Projektfenster für ein Image irgendwo auf der Platte. Der Dialog zeigt das Inhaltsverzeichnis (Name, Art, Größe, Ladeadresse); Dateien ankreuzen („Alle“, „Nur Programme“, „Nur BASIC“) und mit „Übernehmen“ in den Zielordner (Vorgabe `import/`) kopieren. Vorhandene Dateien werden nie überschrieben (der Name bekommt `_2`). **BASIC-Programme** (C64, C128, Plus/4, MEGA65 und Atari-BASIC) werden erkannt und als lesbarer `.bas`-Quelltext übernommen; das Original bleibt nur mit „Original zusätzlich behalten“ erhalten oder wenn hinter dem BASIC-Teil Maschinencode folgt (typisch: `SYS`-Zeile). Maschinenprogramme lassen sich zusätzlich als `.asm` disassemblieren (rein mechanisch, Daten erscheinen als Befehle). Dasselbe auf der Kommandozeile: `vcc import`.
**Auf einem Reiter** (Quelltext): Speichern, Änderungen verwerfen (lädt die Datei neu), Andere schließen und Alle Quelltexte schließen (Dateien mit ungespeicherten Änderungen bleiben offen und werden gemeldet), Editor teilen bzw. Teilung aufheben, Pfad kopieren, Im Finder zeigen, Schließen. Ein kursiver Reiter ist eine Vorschau (Einfachklick im Projektbaum); Bearbeiten oder Doppelklick macht ihn fest.
**Ausgabe:** Zeile kopieren, Alles kopieren, Ausgabe leeren. **Probleme:** Zur Zeile springen, Meldung kopieren, Alle Probleme kopieren.


## Suchen, Ersetzen, Gehe zu Zeile
**Bearbeiten ▸ Suchen …** (Strg/Cmd+F) öffnet über dem Text eine Leiste: Enter springt zum nächsten Treffer, Umschalt+Enter zum vorigen, „Aa“ beachtet die Groß-/Kleinschreibung, Esc schließt. **Ersetzen …** (Strg/Cmd+Alt+F) ergänzt ein Ersetzen-Feld mit „Ersetzen“ (aktuellen Treffer) und „Alle“. **Gehe zu Zeile …** (Strg/Cmd+L) springt zu einer Zeilennummer. **Rückgängig** und **Wiederholen** (Strg/Cmd+Z, Umschalt+Z) stehen auch im Menü. **Datei ▸ Exportieren …** öffnet den Export-Dialog des gerade aktiven Studios; **Datei ▸ Zuletzt geöffnet** listet die letzten Projekte.
