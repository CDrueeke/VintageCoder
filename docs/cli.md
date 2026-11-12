<!-- Erzeugt von "vcc help --markdown" aus data/cli/*.json; nicht von Hand ändern. -->
# Kommandozeile: vcc

vcc ist das Kommandozeilenwerkzeug von VintageCoder. Es übersetzt Assembler und BASIC für C64, C128, Plus/4, VC-20, Atari 8 Bit und MEGA65, legt Projekte an, baut Disketten und führt Tests aus.

Hilfe im Programm: `vcc --help`, `vcc help <befehl>`, `vcc <befehl> --help`. Handbuchseiten: `vcc man <befehl>`; Vervollständigung: `vcc completion <shell>`.

**Programme bauen:** [`asm`](#vcc-asm), [`basic`](#vcc-basic), [`basic2asm`](#vcc-basic2asm), [`build`](#vcc-build)

**Projekte:** [`add`](#vcc-add), [`migrate`](#vcc-migrate), [`new`](#vcc-new), [`organize`](#vcc-organize)

**Medien und Daten:** [`dis`](#vcc-dis), [`disk`](#vcc-disk), [`import`](#vcc-import), [`inc`](#vcc-inc), [`media`](#vcc-media), [`roms`](#vcc-roms)

**Testen und Messen:** [`emulator`](#vcc-emulator), [`streams`](#vcc-streams), [`test`](#vcc-test)

**Hilfe und Rückmeldung:** [`debuglog`](#vcc-debuglog), [`feedback`](#vcc-feedback)

## `vcc add`

Dateien und Ordner ins Projekt übernehmen

Übernimmt Dateien wie das Ablegen in der IDE: erkennt die Art, wählt Kategorie und Zielordner aus rules und layout der Projektdatei, kopiert die Datei und trägt sie in assets ein. Liegt die Quelle schon im Projektordner, bleibt sie dort und wird nur eingetragen.

Ordner werden rekursiv übernommen und behalten ihre Unterordner; versteckte Dateien, target und node_modules bleiben draußen, höchstens 5000 Dateien je Ordner.

```
vcc add <datei|ordner>… [optionen]
```

| Argument | Bedeutung |
|---|---|
| `datei|ordner` | Dateien oder Ordner, die übernommen werden. Mehrfach möglich. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--into` `<projekt>` | Projektordner oder .vcprj-Datei. | `.` |
| `--dir` `<ordner>` | Zielordner im Projekt (relativ, ohne ..). Überschreibt den Ordner der Kategorie; bei Ordnern wird die Unterordnerstruktur angehängt. | Ordner der Kategorie aus layout |
| `--flat` | Dateien aus Ordnern ohne Unterordner ablegen, jede in den Ordner ihrer Kategorie (oder --dir). |  |
| `--category` `<kategorie>` | Kategorie für alle Dateien (Quelltext, Text/Doku, Binärdaten, Grafikrohdaten, Zeichensatz roh). Werte: `source`, `text`, `binary`, `graphics`, `charset`. | aus rules, Dateiname und Inhalt erkannt |
| `--description` `<text>` | Beschreibung, die in assets mit eingetragen wird. |  |
| `--conflict` `<regel>` | Verhalten, wenn am Ziel schon eine andere Datei mit diesem Namen liegt: unter neuem Namen ablegen (name_2), ersetzen oder überspringen. Werte: `rename`, `replace`, `skip`. | `rename` |
| `--dry-run` | Nur anzeigen, was geschehen würde (Zielzustand, Ziel, Art, Kategorie); nichts kopieren, Projektdatei unverändert. |  |

Beispiele:

```bash
# Datei übernehmen
vcc add sprites.bin --into spiel
```

```bash
# Ordner mit Struktur übernehmen
vcc add bilder --category graphics --into spiel
```

```bash
# Ordner flach in einen Zielordner
vcc add musik --flat --dir data/musik --description "Titelmusik"
```

```bash
# nur ansehen
vcc add bilder --into spiel --dry-run
```

> **Hinweis:** Fehlt im Projektordner eine .vcprj, bricht der Befehl ab. Fehler bei einzelnen Dateien ergeben Exit-Code 1; die übrigen werden trotzdem übernommen.

> **Hinweis:** Disketten-Images, PRG und XEX nimmt vcc import auf.

## `vcc asm`

Assembler-Quelltext übersetzen

Übersetzt eine einzelne Assembler-Datei und schreibt das Ergebnis im gewählten Ausgabeformat. Das Zielsystem kommt aus --target (Standard c64); ohne --cpu gilt die CPU des Ziels.

Steht im Quelltext .cartridge, entsteht ohne --format ein Modul (crt beim C64, car beim Atari, vic20cart beim VC-20), sonst das Standardformat des Ziels. Ohne -o heißt die Ausgabe wie der Quelltext, mit der Endung des Formats. Ein ganzes Projekt baut vcc build.

```
vcc asm <datei.asm> [optionen]
```

| Argument | Bedeutung |
|---|---|
| `datei.asm` | Assembler-Quelltext. |

| Option | Bedeutung | Standard |
|---|---|---|
| `-o` `<datei>` | Ausgabedatei. | Quelltextname mit der Endung des Formats |
| `--target` `<system>` | Zielsystem: bestimmt CPU, Chip-Symbole, Textcode und Standardformat. Werte: `atari8`, `c128`, `c64`, `cbm2`, `mega65`, `pet`, `plus4`, `vic20`. | `c64` |
| `--cpu` `<cpu>` | CPU-Befehlssatz, wenn er vom Ziel abweicht. Werte: `45gs02`, `6510`, `7501`, `8502`, `nmos6502`, `sally`. | CPU des Zielsystems |
| `--format` `<format>` | Ausgabeformat. d64, d71 und d81 packen das Programm als einzige Datei in ein neues Diskettenabbild; a0, 20, 40 und 60 erzeugen ein VC-20-Modul für den jeweiligen Speicherblock. Werte: `prg`, `bin`, `xex`, `crt`, `car`, `rom`, `d64`, `d71`, `d81`, `vic20cart`, `a0`, `20`, `40`, `60`. | Standardformat des Zielsystems; mit .cartridge im Quelltext ein Modul |
| `--memory` `<ausbau>` | Speicherausbau des VC-20; bestimmt BASIC-Start, Bildschirm- und Farbspeicher. Bei anderen Zielen ohne Wirkung. Werte: `unexpanded`, `3k`, `8k`, `16k`, `24k`, `35k`. | unexpanded (nur VC-20) |
| `--cart` `<typ>` | Modultyp für crt, car und rom; ersetzt die Angabe von .cartridge im Quelltext. Werte: `normal`, `8k`, `16k`, `ocean`, `magicdesk`, `easyflash`, `actionreplay`, `ar`, `ultimax`, `xegs`. | C64: normal; Atari: aus dem Code abgeleitet |
| `--illegals` `<regel>` | Umgang mit undokumentierten Befehlen: deny meldet Fehler, warn eine Warnung, allow nichts. Werte: `deny`, `warn`, `allow`. | `warn` |
| `-I` `<ordner>` | Zusätzlicher Suchordner für Include-Dateien; data/sys wird immer zuletzt durchsucht. Mehrfach möglich. |  |
| `-D` `<name[=zahl]>` | Definiert eine Konstante für .if und .ifdef. Ohne =zahl gilt 1; Zahlen als 42, $2a, 0x2a oder %101010. Mehrfach möglich. |  |
| `--labels` `<datei.vs>` | Schreibt die Labels als VICE-Labeldatei. |  |
| `--debug` `<datei.vcdbg>` | Schreibt die Debug-Information (für Debugger und Tests). |  |
| `--listing` `<datei.lst>` | Schreibt das Listing mit Adressen und Bytes. |  |
| `--data-dir` `<ordner>` | Ordner der Hardware-Datenbank (data); ersetzt VC_DATA und die automatische Suche. |  |

Beispiele:

```bash
# übersetzen (Ziel C64)
vcc asm examples/c64/border/border.asm -o border.prg
```

```bash
# MEGA65-Programm mit Testkonstante
vcc asm examples/mega65/prallball/prallball.asm -o prallball.prg --target mega65 --cpu 45gs02 --illegals deny -D START_HIRES=1
```

```bash
# Atari-Programm als XEX
vcc asm examples/atari/colorbars/colorbars.asm -o colorbars.xex --target atari8 --format xex
```

```bash
# C64-Modul mit mehreren Bänken
vcc asm examples/c64/cart_demo/cart_demo.asm -o cart_oc.crt --format crt --cart ocean -D OCEAN=1 --labels cart_oc.vs
```

> **Hinweis:** Ein anderer Wert bei --format erzeugt eine PRG-Datei mit dieser Endung. --cart und --cpu werden erst beim Übersetzen geprüft und melden dann einen Fehler.

> **Hinweis:** Bei Modulen füllt der Assembler ungenutzte Bereiche mit $FF auf; die Bänke kommen aus .bank und .org im Quelltext.

> **Hinweis:** Ein VC-20-Modul bei $A000 braucht im Quelltext die Kennung A0CBM bei $A004.

Rückgabe: `0` = übersetzt (Warnungen möglich) · `1` = Fehler im Quelltext, beim Schreiben oder in einem Optionswert · `2` = falscher Aufruf

## `vcc basic`

BASIC tokenisieren, zurücklesen, als Syntaxbaum ausgeben

Ohne weitere Angabe übersetzt vcc basic ein natives BASIC-Programm (Zeilennummern optional, Sprungmarken @name) in das Speicherformat des Zielsystems: PRG bei Commodore, SAVE-Format (.bas) beim Atari. Die Formen --tokenize und --detokenize arbeiten nur auf der Ebene der Token; vcc basic ast gibt den Syntaxbaum aus.

Die Dateien liegen im Ordner data; er wird über --data-dir, VC_DATA, den Ort des Programms oder den Arbeitsordner gefunden.

### Grundform

natives BASIC zu PRG (Commodore) oder BAS (Atari) übersetzen

```
vcc basic <datei.bas|datei.lst|datei.tur> [optionen]
```

| Argument | Bedeutung |
|---|---|
| `datei` | Quelltext. Die Endungen .bas, .lst und .tur (oder --plain) wählen das native BASIC; .sbas ist abgeschaltet. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--target` `<system>` | Zielsystem (Datei in data/targets), z. B. c64, c128, plus4, mega65, vic20, pet, cbm2 oder atari8. | `c64` |
| `--dialect` `<id>` | BASIC-Dialekt (Datei in data/basic): cbm_v2, basic7, basic35, basic65, atari_basic oder turbo_basic_xl. | Dialekt des Zielsystems |
| `--memory` `<ausbau>` | Speicherausbau des Ziels; bestimmt die BASIC-Startadresse. Gibt es nur beim VC-20 (unexpanded, 3k, 8k, 16k, 24k, 35k); andere Ziele ignorieren die Angabe. | Vorgabe des Ziels (VC-20: unexpanded) |
| `--plain` | Quelltext als natives BASIC lesen, auch bei einer anderen Endung als .bas, .lst und .tur. |  |
| `--data-dir` `<ordner>` | Ordner mit der Hardware-Datenbank (data). | VC_DATA oder neben dem Programm |
| `-o` `<ausgabe>` | Ausgabedatei. Bei natives BASIC darf sie nicht die Quelldatei sein. | Quellname mit .prg (Commodore) bzw. .bas (Atari) |
| `--start` `<n>` | Nummer der ersten Zeile ohne eigene Zeilennummer. Eine ungültige Zahl wird still durch den Standard ersetzt. | `10` |
| `--step` `<n>` | Schrittweite der automatischen Zeilennummern. Eine ungültige Zahl wird still durch den Standard ersetzt. | `10` |
| `-I` `<ordner>` | Suchordner für eingebundene Dateien; der erste ist auch die Wurzel für Maschinencode-Einbindungen. Mehrfach möglich. | Arbeitsordner (Wurzel) |
| `-D` `<NAME=zahl>` | Konstante für den Quelltext setzen. Mehrfach möglich. |  |
| `--pack` | Zeilen zusammenfassen (nur Commodore). |  |
| `--map` `<datei>` | Zuordnung Quellname zu BASIC-Name und Marke zu Zeile in eine Textdatei schreiben (nur Commodore). |  |
| `--labels` `<datei>` | VICE-Labeldatei lesen; ihre Adressen stehen im Quelltext als Konstanten asm.name (nur Commodore). |  |

Beispiele:

```bash
# C64-Programm mit Sprungmarken übersetzen
vcc basic examples/c64/basic_native/sterne.bas -o sterne.prg
```

```bash
# Atari-BASIC-Programm übersetzen
vcc basic examples/atari/basic_native/farben.bas --target atari8 -o farben.bas
```

```bash
# Turbo BASIC XL übersetzen
vcc basic examples/atari/turbo_basic/balken.tur --target atari8 --dialect turbo_basic_xl -o balken.bas
```

```bash
# VC-20 mit 8-KB-Erweiterung
vcc basic examples/c64/basic_native/sterne.bas --target vic20 --memory 8k -o sterne.prg
```

> **Hinweis:** Es werden die Marken (@name = Zeile n) und bei #longnames die Zuordnung der langen Namen ausgegeben. Eine .sbas-Datei wird abgelehnt.

> **Hinweis:** Die Optionen -I, -D, --pack, --map und --labels wirken nur beim Übersetzen von Quelltext mit anderer Endung als .bas, .lst und .tur ohne --plain; beim nativen BASIC werden sie angenommen, aber nicht benutzt.

### Form `tokenize`

klassisch nummeriertes Listing tokenisieren

```
vcc basic --tokenize <liste.lst> [optionen]
```

| Argument | Bedeutung |
|---|---|
| `liste` | Listing mit Zeilennummern in jeder Zeile, ohne Marken und ohne Direktiven. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--tokenize` | Schaltet auf diese Form um. Erforderlich. |  |
| `--target` `<system>` | Zielsystem (Datei in data/targets); bestimmt Dialekt und Ladeadresse. | `c64` |
| `--dialect` `<id>` | BASIC-Dialekt (Datei in data/basic). | Dialekt des Zielsystems |
| `--memory` `<ausbau>` | Speicherausbau des Ziels; bestimmt die Ladeadresse (VC-20: unexpanded, 3k, 8k, 16k, 24k, 35k). | Vorgabe des Ziels |
| `--data-dir` `<ordner>` | Ordner mit der Hardware-Datenbank (data). | VC_DATA oder neben dem Programm |
| `-o` `<ausgabe>` | Ausgabedatei. | Quellname mit .prg (Commodore) bzw. .bas (Atari) |

Beispiele:

```bash
# Listing zu PRG tokenisieren
vcc basic --tokenize liste.lst --target c64 -o liste.prg
```

> **Hinweis:** Eine Zeile ohne Zeilennummer ist ein Fehler (B100). Für Quelltext mit Marken und Direktiven die Form ohne --tokenize verwenden.

### Form `detokenize`

PRG oder Atari-SAVE-Datei als Listing ausgeben

```
vcc basic --detokenize <datei.prg> [optionen]
```

| Argument | Bedeutung |
|---|---|
| `datei` | Commodore-PRG oder Atari-BASIC-Datei im SAVE-Format. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--detokenize` | Schaltet auf diese Form um. Erforderlich. |  |
| `--target` `<system>` | Zielsystem (Datei in data/targets). Für Atari-Dateien atari8 angeben; ohne Angabe gilt das Commodore-PRG-Format. | `c64` |
| `--dialect` `<id>` | BASIC-Dialekt (Datei in data/basic); die Wortliste für die Rückübersetzung der Token. | Dialekt des Zielsystems |
| `--data-dir` `<ordner>` | Ordner mit der Hardware-Datenbank (data). | VC_DATA oder neben dem Programm |
| `--structured` | Statt des klassischen Listings Text mit Marken (@L10) ausgeben; nur Commodore. |  |
| `-o` `<ausgabe>` | Listing in eine Datei schreiben. | Ausgabe auf stdout |

Beispiele:

```bash
# PRG als Listing lesen
vcc basic --detokenize sterne.prg
```

```bash
# Atari-Datei in eine Textdatei zurückwandeln
vcc basic --detokenize farben.bas --target atari8 -o farben.lst
```

> **Hinweis:** Schlüsselwörter erscheinen bei Commodore klein, beim Atari groß; Steuerzeichen stehen als {clr} usw. Beginnt die Datei mit zwei Nullbytes, weist die Fehlermeldung auf --target atari8 hin.

### Form `ast`

Syntaxbaum eines BASIC-Programms ausgeben

```
vcc basic ast <datei.bas> [optionen]
```

| Argument | Bedeutung |
|---|---|
| `datei.bas` | BASIC-Quelltext (derselbe Leser wie bei basic2asm). |

| Option | Bedeutung | Standard |
|---|---|---|
| `--target` `<system>` | Zielsystem; bestimmt den BASIC-Dialekt. Werte: `c64`, `c128`, `plus4`, `mega65`, `atari8`. | `c64` |
| `--turbo` | Quelltext als Turbo BASIC XL lesen (nur Ziel atari8). |  |
| `--data-dir` `<ordner>` | Ordner mit der Hardware-Datenbank (data). | VC_DATA oder neben dem Programm |
| `--json` | Baum als JSON ausgeben (Zeilen, Anweisungen, Ausdrücke; Zeichenketten als Bytelisten). | Rust-Debug-Ausgabe der Zeilen |
| `--roundtrip` | Baum als kanonischen Text ausgeben, wieder einlesen und E191 melden, wenn nicht derselbe Baum entsteht. Hat Vorrang vor --json. |  |

Beispiele:

```bash
# Syntaxbaum als JSON
vcc basic ast examples/c64/basic_native/sterne.bas --json
```

```bash
# Rundreise prüfen
vcc basic ast examples/c64/basic_native/sterne.bas --roundtrip
```

> **Hinweis:** Der Aufruf nutzt den Parser von basic2asm; dessen übrige Optionen (-o, --map, --stage, --float, --int-div, --style, --runtime, --no-track-line, --no-rem, --check) werden angenommen, haben hier aber keine Wirkung.

> **Hinweis:** Anweisungen, die der Konverter nicht übersetzt (OPEN, LOAD …), lassen sich nicht zurückschreiben (E190).

Rückgabe: `0` = Baum ausgegeben bzw. Rundreise in Ordnung · `1` = Fehler im Quelltext oder Rundreise nicht in Ordnung

## `vcc basic2asm`

BASIC-Programm in Assembler-Quelltext umwandeln

Übersetzt ein BASIC-Programm in lesbaren Assembler-Quelltext, der als Maschinenprogramm läuft. Jede BASIC-Zeile wird ein Abschnitt mit dem Original als Kommentar und dem Label L<Zeile>. Neben der .asm-Datei entsteht eine Zeilenkarte (JSON) für den Debugger.

Der Konverter wählt die kleinste Stufe, die das Programm braucht: 1 = Ganzzahl, 2 = Fließkomma, 3 = Zeichenketten. --stage setzt die Obergrenze.

```
vcc basic2asm <datei.bas> [optionen]
```

| Argument | Bedeutung |
|---|---|
| `datei.bas` | BASIC-Quelltext. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--target` `<system>` | Zielsystem; legt Startadresse, Prozessor und BASIC-Dialekt fest. Auf dem MEGA65 gibt es kein Fließkomma (E331). Werte: `c64`, `c128`, `plus4`, `mega65`, `atari8`. | `c64` |
| `--turbo` | Quelltext als Turbo BASIC XL lesen (nur Ziel atari8, sonst E344); Blöcke wie WHILE/WEND, PROC/EXEC werden zu Sprüngen. |  |
| `--data-dir` `<ordner>` | Ordner mit der Hardware-Datenbank (data); bei --runtime embed auch die Quelle der Laufzeit. | VC_DATA oder neben dem Programm |
| `--stage` `<1\|2\|3>` | Höchste zugelassene Stufe; gewählt wird die kleinste, die das Programm braucht. Andere Zahlen als 1 bis 3 melden E300. | `3` |
| `--float` | Fließkomma-Laufzeit auch dann einbinden, wenn das Programm sie nicht braucht. |  |
| `--int-div` | / als ganzzahlige Division übersetzen; sonst braucht es Stufe 2. |  |
| `--style` `<stil>` | compact: jede Operation ruft eine Laufzeitroutine. inline: einfache Ganzzahlausdrücke stehen als direkte 6502-Befehle da (länger im Quelltext, kürzer in der Laufzeit). Werte: `compact`, `inline`. | `compact` |
| `--runtime` `<art>` | include: die Laufzeit kommt per #include aus data/sys. embed: sie wird in die Ergebnisdatei kopiert, die dann ohne Include-Ordner übersetzbar ist. Werte: `include`, `embed`. | `include` |
| `--no-track-line` | Keine Zeilennummer zur Laufzeit mitführen: kleiner, aber Fehlermeldungen ohne Zeile. |  |
| `--no-rem` | Zeilen, die nur aus REM bestehen, nicht übernehmen. |  |
| `-o` `<datei.asm>` | Ausgabedatei mit dem Assembler-Quelltext. | Quellname mit .asm |
| `--map` `<datei.json>` | Zeilenkarte: BASIC-Zeile, Quelltextzeile und Label im Assembler. | Quellname mit .b2a.json |
| `--check` | Nur prüfen und Stufe und Zeilenzahl melden; es werden keine Dateien geschrieben. |  |

Beispiele:

```bash
# einfach umwandeln
vcc basic2asm examples/c64/basic_native/sterne.bas -o sterne.asm
```

```bash
# für den Atari umwandeln
vcc basic2asm examples/atari/basic_native/farben.bas --target atari8 -o farben.asm
```

```bash
# Turbo BASIC XL mit Stufenobergrenze
vcc basic2asm examples/atari/turbo_basic/balken.tur --target atari8 --turbo --stage 2 -o balken.asm
```

```bash
# nur prüfen
vcc basic2asm examples/c64/basic_native/sterne.bas --check
```

> **Hinweis:** Befehle, die der Konverter nicht übersetzt (OPEN, LOAD, LIST, NEW …), werden gemeldet (E311), nicht still ersetzt. Beim Atari kommt immer der Hinweis I150 zu den Zahlen.

> **Hinweis:** Der Syntaxbaum lässt sich mit vcc basic ast ausgeben.

## `vcc build`

Projekt laut .vcprj bauen

Baut alle Quellen der Projektdatei in Baureihenfolge (Assembler und BASIC, einschließlich Umwandlung von BASIC in Assembler und Export von Medien). Beschreibt die Projektdatei eine Diskette, legt der Befehl sie danach an. Die Ergebnisse liegen im Ordner build/ des Projekts.

```
vcc build [projekt] [optionen]
```

| Argument | Bedeutung |
|---|---|
| `projekt` | Projektordner (die erste .vcprj darin) oder Projektdatei. Ohne Angabe gilt der aktuelle Ordner. (optional) |

| Option | Bedeutung | Standard |
|---|---|---|
| `--main` | Baut nur das Startprogramm (main der Projektdatei); die Diskette entsteht dann nicht. |  |
| `--data-dir` `<ordner>` | Ordner der Hardware-Datenbank (data); ersetzt VC_DATA und die automatische Suche. |  |

Beispiele:

```bash
# Projekt im aktuellen Ordner bauen
vcc build
```

```bash
# Projektordner bauen
vcc build examples/c64/cart_demo
```

```bash
# nur das Startprogramm
vcc build examples/mega65/prallball/prallball.vcprj --main
```

> **Hinweis:** Je Quelle erscheint eine Zeile OK oder FEHLER, das Startprogramm ist markiert; Meldungen stehen auf der Fehlerausgabe.

> **Hinweis:** Warnungen zur Projektdatei (z. B. fehlende Dateien aus assets) kommen vorab. Ein Projekt ohne Quellen ist ein Fehler.

Rückgabe: `0` = alles gebaut · `1` = eine Quelle oder die Diskette fehlgeschlagen, oder keine Projektdatei gefunden · `2` = falscher Aufruf

## `vcc debuglog`

Debug-Protokoll der IDE auswerten

Fasst die Protokolldatei zusammen, die VintageCoder schreibt, wenn das Debug-Protokoll eingeschaltet ist (Einstellungen ▸ Allgemein oder Umgebungsvariable VC_DEBUG_LOG). Die Datei besteht aus JSON-Zeilen (ts, seq, kind, msg); die erste Zeile enthält Version und Installation. Die Ausgabe nennt Programm, Zahl und Art der Einträge, Abstürze, Fehler und Warnungen sowie die letzten Ereignisse vor dem Ende.

Ohne Datei gilt die Standarddatei logs/vintagecoder.log im Konfigurationsordner. Der Befehl liest nur und sendet nichts.

```
vcc debuglog [datei.log] [optionen]
```

| Argument | Bedeutung |
|---|---|
| `datei.log` | Protokolldatei; Standard ist die Datei im Konfigurationsordner (oder VC_DEBUG_LOG, falls ein Pfad). (optional) |

| Option | Bedeutung | Standard |
|---|---|---|
| `--json` | Zusammenfassung maschinenlesbar als JSON ausgeben. |  |
| `--tail` `<n>` | Anzahl der letzten Ereignisse, die gezeigt werden. | 20 |
| `--path` | Nur den Pfad der Standarddatei ausgeben. |  |

Beispiele:

```bash
# Standardprotokoll zusammenfassen
vcc debuglog
```

```bash
# Datei aus einer Rückmeldung auswerten, maschinenlesbar
vcc debuglog fehler.log --json
```

```bash
# Pfad der Standarddatei anzeigen
vcc debuglog --path
```

> **Hinweis:** Zeilen, die kein gültiger Eintrag sind (etwa eine beim Absturz abgeschnittene), werden gezählt und übersprungen.

> **Hinweis:** Beim Wechsel der Datei (5 MB) bleibt eine Vorgängerdatei mit der Endung .log.1.

## `vcc dis`

Maschinenprogramm disassemblieren

Gibt das Disassemblat auf der Standardausgabe aus: Adresse, Bytes und Befehl je Zeile; undokumentierte Befehle tragen den Zusatz „undokumentiert“.

Bei .prg steht die Startadresse im Dateikopf. Atari-XEX-Dateien (Endung .xex oder .com, oder Kopf $FFFF) werden Segment für Segment mit ihren Adressen ausgegeben. Bei anderen Dateien gilt Adresse 0, wenn --start fehlt.

```
vcc dis <datei> [optionen]
```

| Argument | Bedeutung |
|---|---|
| `datei` | Programmdatei (.prg, .xex, .bin). |

| Option | Bedeutung | Standard |
|---|---|---|
| `--start` `<adresse>` | Startadresse des Abbilds. Die Datei wird dann als rohe Bytes gelesen: bei .prg zählt der Kopf mit, bei XEX entfällt die Segmentaufteilung. | Ladeadresse bei .prg; Segmentadressen bei XEX; sonst 0 |
| `--cpu` `<cpu>` | CPU-Befehlssatz. Werte: `45gs02`, `6510`, `7501`, `8502`, `nmos6502`, `sally`. | `6510` |
| `--data-dir` `<ordner>` | Ordner der Hardware-Datenbank (data); ersetzt VC_DATA und die automatische Suche. |  |

Beispiele:

```bash
# C64-Programm disassemblieren
vcc dis border.prg
```

```bash
# MEGA65-Programm mit 45GS02
vcc dis prallball.prg --cpu 45gs02
```

```bash
# rohes Abbild ab Adresse
vcc dis kernal.bin --start $e000
```

> **Hinweis:** Adressen sind Zahlen wie 49152, $c000, 0xc000 oder %1100; eine ungültige Zahl bei --start wird übergangen.

> **Hinweis:** Die Erkennung von .prg beachtet die Groß- und Kleinschreibung der Endung.

## `vcc disk`

Diskettenimages anlegen, auflisten und Dateien herausholen

Arbeitet mit den Formaten D64 (1541), D71 (1571), D81 (1581) und ATR (Atari, DOS 2). Die Art des Images folgt beim Anlegen aus der Unterform, beim Auflisten und Extrahieren aus der Dateiendung.

Bei `--add` trennt der erste Doppelpunkt Datei und Diskettenname. Ein Pfad mit Laufwerksbuchstaben (C:\...) lässt sich deshalb nicht zusammen mit einem Namen angeben; dann relative Pfade verwenden.

### Form `d64`

D64-Image (1541) mit Programmen anlegen

```
vcc disk d64 -o <out.d64> [--name <name>] [--id <id>] [--add <datei[:NAME]>]...
```

| Option | Bedeutung | Standard |
|---|---|---|
| `-o` `<datei>` | Ausgabedatei des Images. Erforderlich. |  |
| `--name` `<name>` | Name der Diskette im Verzeichniskopf. | `VINTAGECODER` |
| `--id` `<id>` | Disketten-ID im Verzeichniskopf. | `VC` |
| `--add` `<datei[:NAME]>` | Programmdatei (PRG) auf die Diskette schreiben, optional unter anderem Namen. Mehrfach möglich. | Dateiname ohne Endung, in Großbuchstaben |
| `--keep` | Wird angenommen und ignoriert. |  |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# Diskette mit einem Programm
vcc disk d64 -o spiel.d64 --name SPIEL --id 01 --add jumprun.prg:JUMPRUN
```

> **Hinweis:** Die Ausgabe zeigt Größe und freie Blöcke.

### Form `d71`

D71-Image (1571) mit Programmen anlegen

```
vcc disk d71 -o <out.d71> [--name <name>] [--id <id>] [--add <datei[:NAME]>]...
```

| Option | Bedeutung | Standard |
|---|---|---|
| `-o` `<datei>` | Ausgabedatei des Images. Erforderlich. |  |
| `--name` `<name>` | Name der Diskette im Verzeichniskopf. | `VINTAGECODER` |
| `--id` `<id>` | Disketten-ID im Verzeichniskopf. | `VC` |
| `--add` `<datei[:NAME]>` | Programmdatei (PRG) auf die Diskette schreiben, optional unter anderem Namen. Mehrfach möglich. | Dateiname ohne Endung, in Großbuchstaben |
| `--keep` | Wird angenommen und ignoriert. |  |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# D71 mit einem Programm
vcc disk d71 -o spiel.d71 --add jumprun.prg
```

### Form `d81`

D81-Image (1581) mit Programmen anlegen

```
vcc disk d81 -o <out.d81> [--name <name>] [--id <id>] [--add <datei[:NAME]>]...
```

| Option | Bedeutung | Standard |
|---|---|---|
| `-o` `<datei>` | Ausgabedatei des Images. Erforderlich. |  |
| `--name` `<name>` | Name der Diskette im Verzeichniskopf. | `VINTAGECODER` |
| `--id` `<id>` | Disketten-ID im Verzeichniskopf. | `VC` |
| `--add` `<datei[:NAME]>` | Programmdatei (PRG) auf die Diskette schreiben, optional unter anderem Namen. Mehrfach möglich. | Dateiname ohne Endung, in Großbuchstaben |
| `--keep` | Wird angenommen und ignoriert. |  |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# D81 für MEGA65 oder C128
vcc disk d81 -o spiel.d81 --name SPIEL --add jumprun.prg
```

### Form `atr`

ATR-Image (Atari DOS 2) mit Dateien anlegen

```
vcc disk atr -o <out.atr> [--add <datei[:NAME.EXT]>]...
```

| Option | Bedeutung | Standard |
|---|---|---|
| `-o` `<datei>` | Ausgabedatei des Images. Erforderlich. |  |
| `--add` `<datei[:NAME.EXT]>` | Datei in das DOS-2-Dateisystem schreiben, optional unter anderem Namen. Mehrfach möglich. | Dateiname in Großbuchstaben, mit Endung |
| `--name` `<name>` | Wird angenommen, hat bei ATR-Images keine Wirkung. | `VINTAGECODER` |
| `--id` `<id>` | Wird angenommen, hat bei ATR-Images keine Wirkung. | `VC` |
| `--keep` | Wird angenommen und ignoriert. |  |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# ATR mit einer Datei als AUTORUN.SYS
vcc disk atr -o spiel.atr --add jumprun.xex:AUTORUN.SYS
```

> **Hinweis:** Das Image ist 90 KB groß (720 Sektoren zu 128 Byte). Eine bootbare Diskette ohne Dateisystem erzeugt die Form `atr-boot`.

### Form `atr-boot`

bootbare ATR-Diskette aus einem XEX-Programm

```
vcc disk atr -o <out.atr> --boot <prog.xex>
```

| Option | Bedeutung | Standard |
|---|---|---|
| `-o` `<datei>` | Ausgabedatei des Images. Erforderlich. |  |
| `--boot` `<prog.xex>` | XEX-Programm, das die Diskette beim Einschalten lädt (Boot-Lader aus der Hardware-Datenbank). Erforderlich. |  |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sie liefert den Boot-Lader. |  |

Beispiele:

```bash
# bootbare Diskette
vcc disk atr -o jumprun.atr --boot jumprun.xex
```

> **Hinweis:** `--boot` lässt sich nicht mit `--add` verbinden: die Boot-Diskette hat kein Dateisystem. `vcc disk list` meldet das entsprechend.

> **Hinweis:** Eine Datei ohne XEX-Kennung ($FFFF am Anfang) führt zu einem Fehler.

### Form `list`

Verzeichnis eines Images zeigen und prüfen

```
vcc disk list <image.d64|d71|d81|atr>
```

| Argument | Bedeutung |
|---|---|
| `image` | Diskettenimage; die Endung (d64, d71, d81, atr) bestimmt das Format. |

Beispiele:

```bash
# Verzeichnis anzeigen
vcc disk list spiel.d64
```

```bash
# Atari-Diskette anzeigen
vcc disk list spiel.atr
```

> **Hinweis:** Commodore: Blockzahl, Name, Dateiart und freie Blöcke. Atari: Name, Sektoren, Startsektor, `*` für geschützte Dateien und freie Sektoren.

> **Hinweis:** Die Prüfung (BAM bzw. VTOC, Verzeichnis, Dateiketten) meldet Abweichungen als „Problem:“ auf stderr.

Rückgabe: `0` = fehlerfrei · `1` = Datei nicht lesbar oder Problem im Image gefunden · `2` = falscher Aufruf

### Form `extract`

eine Datei aus einem Image holen

```
vcc disk extract <image.d64|d71|d81|atr> <NAME> [-o <datei>]
```

| Argument | Bedeutung |
|---|---|
| `image` | Diskettenimage; die Endung bestimmt das Format. |
| `NAME` | Dateiname auf der Diskette. |

| Option | Bedeutung | Standard |
|---|---|---|
| `-o` `<datei>` | Zieldatei. | Commodore: NAME klein mit Endung .prg; Atari: NAME klein |

Beispiele:

```bash
# Programm aus D64 holen
vcc disk extract spiel.d64 JUMPRUN -o jumprun.prg
```

```bash
# Datei aus ATR holen
vcc disk extract spiel.atr AUTORUN.SYS -o autorun.sys
```

## `vcc emulator`

Emulatoren anzeigen, finden und portabel installieren

Zeigt die Emulatoren, die VintageCoder nutzt (VICE, atari800, Altirra, Xemu, Fujisan), mit Homepage, Lizenz und Rolle. `install` lädt auf ausdrücklichen Aufruf eine feste Version von der Seite des Projekts, prüft die SHA-256-Summe und packt sie unterhalb von VintageCoder aus (`emulators/<Emulator>/<Version>` im Konfigurationsordner). Es wird nichts mitgeliefert und nichts ohne diesen Aufruf geladen.

### Form `list`

die Emulatoren des Katalogs mit Homepage und Rolle zeigen

```
vcc emulator list
```

Beispiele:

```bash
# alle Emulatoren auflisten
vcc emulator list
```

### Form `status`

zeigen, was gefunden wird (alle Quellen) und was sich installieren lässt

```
vcc emulator status [emulator]
```

| Argument | Bedeutung |
|---|---|
| `emulator` | Kennung (vice, atari800, altirra, xemu, fujisan) oder Sorte (x64sc, x128 …); ohne Angabe alle. (optional) |

Beispiele:

```bash
# alle Emulatoren prüfen
vcc emulator status
```

```bash
# einen Emulator prüfen
vcc emulator status atari800
```

> **Hinweis:** Automatisch gilt: Pfad aus den Einstellungen, sonst das, was PATH und die üblichen Programmordner bieten, zuletzt die neueste portabel installierte Version.

### Form `install`

eine feste Version laden, prüfen und portabel installieren

```
vcc emulator install <emulator[@version]> [optionen]
```

| Argument | Bedeutung |
|---|---|
| `emulator[@version]` | Kennung (vice, atari800, altirra, xemu, fujisan) oder Sorte (x64sc, x128 …), optional mit Version (vice@3.9). Ohne Version gilt die Vorgabe (die neueste). |

| Option | Bedeutung | Standard |
|---|---|---|
| `--dry-run` | Zeigt nur Paket, Adresse, Prüfsumme und Zielordner an, ohne etwas zu laden. |  |

Beispiele:

```bash
# anzeigen, was geladen würde
vcc emulator install atari800 --dry-run
```

```bash
# Fujisan laden und installieren
vcc emulator install fujisan
```

```bash
# eine ältere Version daneben installieren
vcc emulator install vice@3.9
```

> **Hinweis:** Nutzt `curl` und `tar` des Systems (Windows 10/11 und Linux bringen sie mit). Wo es kein portables Paket gibt (z. B. macOS, VICE unter Linux), nennt der Befehl die Download-Seite.

> **Hinweis:** Ist die Version schon installiert, geschieht nichts. Stimmt die Prüfsumme nicht, wird die Datei gelöscht und nichts ausgepackt.

> **Hinweis:** Bei der automatischen Suche kommen die portabel installierten Programme nach PATH und den üblichen Programmordnern; ein Pfad in den Einstellungen und eine mit `@version` gewählte Version gehen vor.

> **Hinweis:** Mehrere Versionen eines Emulators liegen nebeneinander. `vcc emulator status` zeigt alle gefundenen Quellen (portabel und System) und die ladbaren Versionen; `vcc test --backend vice@3.9` nutzt eine bestimmte.

Rückgabe: `0` = installiert oder schon vorhanden · `1` = Fehler (Netz, Prüfsumme, Auspacken, kein Paket) · `2` = unbekannter Emulator

## `vcc feedback`

Rückmeldung per E-Mail mit Angaben zur Installation vorbereiten

Stellt Betreff und Text einer Mail an vintagecoder@drueeke.de zusammen: Version, Commit, Betriebssystem, Architektur, Installationsart und -ordner, Datenordner, Konfigurationsordner, Release-Auswahl und – mit --target – das Zielsystem. Pfade im persönlichen Ordner stehen als ~, Projektnamen kommen nicht vor.

Ohne --open wird nur ausgegeben, was in der Mail stünde. Mit --open öffnet das Betriebssystem das Standard-Mailprogramm mit vorausgefülltem Betreff und Text; gesendet wird erst, wenn du dort auf Senden klickst. Fehlerbeschreibung, Screenshots und Anhänge ergänzt du dort selbst. Es gibt keinen Netzzugriff und keine Telemetrie.

```
vcc feedback [optionen]
```

| Option | Bedeutung | Standard |
|---|---|---|
| `--open` | Die Mail im Standard-Mailprogramm öffnen (macOS open, Windows rundll32, Linux xdg-open). | nur ausgeben |
| `--json` | Alles maschinenlesbar als JSON ausgeben (Betreff, Text, Link, Einzelangaben). |  |
| `--target` `<system>` | Zielsystem des Projekts in die Angaben aufnehmen, z. B. c64. | kein Zielsystem |

Beispiele:

```bash
# ansehen, was in der Mail stünde
vcc feedback
```

```bash
# Mail im Mailprogramm öffnen
vcc feedback --open
```

```bash
# als JSON, mit Zielsystem
vcc feedback --json --target c64
```

> **Hinweis:** Der mailto-Link ist auf rund 1900 Zeichen begrenzt (Windows verarbeitet etwa 2000). Passt die Mail nicht hinein, fehlen die entbehrlichen Angaben, und der Text sagt, welche; die Ausgabe mit --json enthält den vollständigen Text als body_full.

> **Hinweis:** Mit --open ohne Mailprogramm endet der Befehl mit Fehlercode 1; Betreff und Text stehen dann trotzdem in der Ausgabe.

## `vcc import`

Programme und Dateien von Disketten oder einzeln ins Projekt holen

Übernimmt Dateien aus Diskettenimages (D64, D71, D81, ATR) oder einzelne PRG-, XEX- und BASIC-Dateien in einen Projektordner. Ohne Namen und ohne `--all` zeigt der Befehl nur das Verzeichnis.

### Form `verzeichnis`

Inhaltsverzeichnis eines Images zeigen

```
vcc import <image.d64|d71|d81|atr> [--data-dir <pfad>]
```

| Argument | Bedeutung |
|---|---|
| `image` | Diskettenimage (Endung d64, d71, d81 oder atr). |

| Option | Bedeutung | Standard |
|---|---|---|
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# Verzeichnis anzeigen
vcc import spiel.d64
```

> **Hinweis:** Zeigt je Datei Name, Art, Größe, Ladeadresse und bei erkannten BASIC-Programmen Dialekt und Zeilenzahl (nur mit Hardware-Datenbank). Es wird nichts geschrieben.

### Form `image`

Dateien aus einem Image ins Projekt übernehmen

```
vcc import <image.d64|d71|d81|atr> (<NAME>... | --all) [--into <projektordner>] [--dir <ordner>] [--asm [--cpu <id>]] [--raw] [--no-basic]
```

| Argument | Bedeutung |
|---|---|
| `image` | Diskettenimage (Endung d64, d71, d81 oder atr). |
| `NAME` | Dateiname auf der Diskette (Groß-/Kleinschreibung egal); statt dessen --all. (optional) Mehrfach möglich. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--all` | Übernimmt alle Dateien des Images. |  |
| `--into` `<projektordner>` | Projektordner, in den die Dateien übernommen werden. | `.` |
| `--dir` `<ordner>` | Unterordner im Projekt, in den die Dateien geschrieben werden. | `import` |
| `--asm` | Legt für Maschinenprogramme zusätzlich einen disassemblierten Quelltext (.asm) an. Das ist rein mechanisch, Daten erscheinen als Befehle. |  |
| `--cpu` `<id>` | Prozessor für das Disassemblieren mit --asm (Name einer Datei in data/cpus). | 6510, bei Atari-Programmen sally |
| `--raw` | Behält zusätzlich die unveränderte Originaldatei, wenn ein BASIC-Programm als Quelltext übernommen wird. |  |
| `--no-basic` | Erkennt BASIC nicht und kopiert die Dateien unverändert. |  |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# eine Datei übernehmen
vcc import spiel.d64 JUMPRUN --into meinprojekt
```

```bash
# alle Dateien mit Quelltext
vcc import spiel.atr --all --asm --dir quellen
```

> **Hinweis:** BASIC-Programme werden als lesbarer .bas-Quelltext übernommen. Vorhandene Dateien im Zielordner bleiben erhalten (der neue Dateiname bekommt ein Suffix _2, _3 ...).

> **Hinweis:** Der Exit-Code ist 1, wenn eine Datei nicht übernommen werden konnte oder ein Name nicht auf der Diskette steht.

### Form `datei`

einzelnes Programm ins Projekt übernehmen

```
vcc import <datei.prg|datei.xex|datei.bas> [--into <projektordner>] [--dir <ordner>] [--asm [--cpu <id>]] [--raw] [--no-basic]
```

| Argument | Bedeutung |
|---|---|
| `datei` | PRG-, XEX- oder BASIC-Datei (kein Diskettenimage). |

| Option | Bedeutung | Standard |
|---|---|---|
| `--into` `<projektordner>` | Projektordner, in den die Dateien übernommen werden. | `.` |
| `--dir` `<ordner>` | Unterordner im Projekt, in den die Dateien geschrieben werden. | `import` |
| `--asm` | Legt für Maschinenprogramme zusätzlich einen disassemblierten Quelltext (.asm) an. Das ist rein mechanisch, Daten erscheinen als Befehle. |  |
| `--cpu` `<id>` | Prozessor für das Disassemblieren mit --asm (Name einer Datei in data/cpus). | 6510, bei Atari-Programmen sally |
| `--raw` | Behält zusätzlich die unveränderte Originaldatei, wenn ein BASIC-Programm als Quelltext übernommen wird. |  |
| `--no-basic` | Erkennt BASIC nicht und kopiert die Dateien unverändert. |  |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# Atari-Programm mit Quelltext
vcc import jumprun.xex --asm --into meinprojekt
```

```bash
# BASIC-Programm übernehmen
vcc import sterne.prg --into meinprojekt
```

> **Hinweis:** Eine Datei gilt als Atari-Programm, wenn sie eine XEX-Kennung hat oder eine tokenisierte Atari-BASIC-Datei (.bas) ist. Als Programm gilt sie außerdem bei der Endung .prg.

## `vcc inc`

Include-Datei mit Hardwarekonstanten ausgeben

Erzeugt aus der Hardware-Datenbank (data/chips) eine Include-Datei mit den Registern der Chips des Zielsystems als .const-Zeilen. Ohne -o geht der Text auf die Standardausgabe.

```
vcc inc --target <system> [optionen]
```

| Option | Bedeutung | Standard |
|---|---|---|
| `--target` `<system>` | Zielsystem, dessen Chips ausgegeben werden. Werte: `atari8`, `c128`, `c64`, `cbm2`, `mega65`, `pet`, `plus4`, `vic20`. Erforderlich. |  |
| `-o` `<datei.inc>` | Ausgabedatei; ohne die Option erscheint der Text auf dem Bildschirm. |  |
| `--data-dir` `<ordner>` | Ordner der Hardware-Datenbank (data); ersetzt VC_DATA und die automatische Suche. |  |

Beispiele:

```bash
# C64-Konstanten ausgeben
vcc inc --target c64
```

```bash
# Atari-Konstanten in eine Datei
vcc inc --target atari8 -o atari.inc
```

## `vcc media`

Sprites, Zeichensätze, Songs und andere Mediendateien bearbeiten

Auch als: `vcc asset`.

Arbeitet mit den Mediendateien eines Projekts (graphics/NAME.vcsprite, .vcpmg, .vcchar, .vcscreen, .vctiles, .vcpal; music/NAME.vcsong, .vcplay). Der Projektordner ist der aktuelle Ordner.

### Form `new`

neue Mediendatei nach einem Profil anlegen

```
vcc media new <name> --profile <id> [--dir <ordner>] [--blank]
```

| Argument | Bedeutung |
|---|---|
| `name` | Name der Datei ohne Endung; erlaubt sind Buchstaben, Ziffern, _ und -. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--profile` `<id>` | Profil, das Art und Aufbau bestimmt (Sprite, Zeichensatz, Screen, Kacheln, Song, Palette ...). Erforderlich. |  |
| `--dir` `<ordner>` | Zielordner im Projekt (relativ zum aktuellen Ordner). | Standardordner der Art: music für Songs, sonst graphics |
| `--blank` | Legt einen Zeichensatz leer an, statt mit einem Verweis auf den Standardzeichensatz des Systems. |  |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# Zeichensatz anlegen
vcc media new font --profile c64.char.hires
```

```bash
# leeren Zeichensatz in Unterordner
vcc media new spiel --profile c64.char.hires --blank --dir graphics/fonts
```

```bash
# Multicolor-Sprite
vcc media new hero --profile c64.sprite.multicolor
```

> **Hinweis:** Ein neuer Zeichensatz beginnt mit einem Verweis auf den Standardzeichensatz des Systems in deinem ROM; --blank legt ihn leer an.

> **Hinweis:** Hat der Ordner eine .vcprj, muss das Zielsystem des Profils zu den Systemen des Projekts passen.

> **Hinweis:** Der Projektordner ist der aktuelle Ordner; Pfade zu Projektdateien sind relativ dazu. Eine vorhandene Zieldatei wird nie überschrieben (Fehler).

> **Hinweis:** Profil-IDs (z. B. c64.char.hires, c64.sprite.multicolor, atari.pmg.single, c64.sid.song) stehen in data/profile/profiles.json.

### Form `import-sprite`

Sprites aus SPD, Rohdaten oder anderem Sprite-Asset übernehmen

```
vcc media import-sprite <quelle> --name <name> [--profile <id>] [--offset <n>] [--count <n>] [--mode hires|multicolor] [--frame <n>] [--dir <ordner>]
```

| Argument | Bedeutung |
|---|---|
| `quelle` | SPD-Datei (.spd), Sprite-Datei (.vcsprite, .vcpmg) oder Rohdaten in 64-Byte-Blöcken (.bin, .prg ...). |

| Option | Bedeutung | Standard |
|---|---|---|
| `--name` `<name>` | Name der neuen Datei ohne Endung; erlaubt sind Buchstaben, Ziffern, _ und -. Erforderlich. |  |
| `--profile` `<id>` | Zielprofil (ein Sprite- oder PMG-Profil). | `c64.sprite.hires` |
| `--dir` `<ordner>` | Zielordner im Projekt (relativ zum aktuellen Ordner). | Standardordner der Art: music für Songs, sonst graphics |
| `--offset` `<n>` | Rohdaten: Byte-Offset des ersten Sprites (dezimal, $hex, 0xhex oder %binär). Bei .prg kommen die zwei Bytes der Ladeadresse dazu. | `0` |
| `--count` `<n>` | Rohdaten: Zahl der Sprites; 0 heißt alle vollständigen Blöcke ab dem Offset (höchstens 256). | `0` |
| `--mode` `<modus>` | Rohdaten: Modus der Sprites; multicolor kürzer mc. Werte: `hires`, `multicolor`, `mc`. | Modus des Zielprofils |
| `--frame` `<n>` | Beim Wechsel zu Atari-PMG: Nummer des Quell-Sprites (0-basiert), das übernommen wird. | `0` |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# SPD-Datei übernehmen
vcc media import-sprite figuren.spd --name figuren
```

```bash
# Rohdaten, fünf Multicolor-Sprites
vcc media import-sprite sprites.bin --name raw --offset 0 --count 5 --mode multicolor
```

```bash
# C64-Sprite als Atari-PMG
vcc media import-sprite graphics/hero.vcsprite --name hero_pmg --profile atari.pmg.single --frame 0
```

> **Hinweis:** Sprite-Import in Profile anderer Systeme als C64 wird nur für PMG (Atari) unterstützt. Die Ausgabe nennt Pfad und Zahl der Frames, Hinweise stehen auf stderr.

> **Hinweis:** Der Projektordner ist der aktuelle Ordner; Pfade zu Projektdateien sind relativ dazu. Eine vorhandene Zieldatei wird nie überschrieben (Fehler).

> **Hinweis:** Profil-IDs (z. B. c64.char.hires, c64.sprite.multicolor, atari.pmg.single, c64.sid.song) stehen in data/profile/profiles.json.

### Form `import-song`

Song aus GoatTracker, SID oder Song-Datei übernehmen

```
vcc media import-song <quelle> --name <name> [--profile <id>] [--song <n>] [--seconds <n>] [--dir <ordner>]
```

| Argument | Bedeutung |
|---|---|
| `quelle` | GoatTracker-Song (.sng), SID-Datei (.sid) oder Song-Datei (.vcsong) eines Projekts. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--name` `<name>` | Name der neuen Datei ohne Endung; erlaubt sind Buchstaben, Ziffern, _ und -. Erforderlich. |  |
| `--profile` `<id>` | Zielprofil des Songs (anderer Chip: Umwandlung). | `c64.sid.song` |
| `--dir` `<ordner>` | Zielordner im Projekt (relativ zum aktuellen Ordner). | Standardordner der Art: music für Songs, sonst graphics |
| `--song` `<n>` | SID-Datei: Nummer des Subtunes (ab 1); 0 nimmt den Startsong der Datei. | `0` |
| `--seconds` `<n>` | SID-Datei: Sekunden, die von der Spielroutine aufgenommen werden. | `30` |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# GoatTracker-Song
vcc media import-song lied.sng --name lied
```

```bash
# SID aufnehmen (20 Sekunden, Subtune 2)
vcc media import-song musik.sid --name musik --song 2 --seconds 20
```

```bash
# Song für den MEGA65 übernehmen
vcc media import-song music/tune.vcsong --name tune4 --profile mega65.sid4.song
```

> **Hinweis:** Bei gleichem Chip wird der Song übernommen (Stimmen werden aufgefüllt, wenn das Ziel mehr hat), sonst umgewandelt. Hinweise stehen auf stderr.

> **Hinweis:** Der Projektordner ist der aktuelle Ordner; Pfade zu Projektdateien sind relativ dazu. Eine vorhandene Zieldatei wird nie überschrieben (Fehler).

> **Hinweis:** Profil-IDs (z. B. c64.char.hires, c64.sprite.multicolor, atari.pmg.single, c64.sid.song) stehen in data/profile/profiles.json.

### Form `convert`

Song in das Profil eines anderen Chips umwandeln

```
vcc media convert <song.vcsong> --profile <id> --name <name> [--dir <ordner>]
```

| Argument | Bedeutung |
|---|---|
| `song.vcsong` | Song-Datei im Projekt. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--profile` `<id>` | Zielprofil mit anderem Chip (z. B. atari.pokey.song). Erforderlich. |  |
| `--name` `<name>` | Name der neuen Datei ohne Endung; erlaubt sind Buchstaben, Ziffern, _ und -. Erforderlich. |  |
| `--dir` `<ordner>` | Zielordner im Projekt (relativ zum aktuellen Ordner). | Standardordner der Art: music für Songs, sonst graphics |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# C64-Song für POKEY
vcc media convert music/tune.vcsong --profile atari.pokey.song --name tune_pokey
```

> **Hinweis:** Haben Quelle und Ziel denselben Chip, bricht der Befehl ab; dafür gibt es `import-song`. Hinweise zur Umwandlung (gekürzte Hüllkurven, verschobene Oktaven) stehen auf stderr.

> **Hinweis:** Der Projektordner ist der aktuelle Ordner; Pfade zu Projektdateien sind relativ dazu. Eine vorhandene Zieldatei wird nie überschrieben (Fehler).

> **Hinweis:** Profil-IDs (z. B. c64.char.hires, c64.sprite.multicolor, atari.pmg.single, c64.sid.song) stehen in data/profile/profiles.json.

### Form `import-rmt`

RMT-Modul als POKEY-Song übernehmen

```
vcc media import-rmt <modul.rmt> --name <name> [--dir <ordner>]
```

| Argument | Bedeutung |
|---|---|
| `modul.rmt` | RMT-Modul (Raster Music Tracker) im Projekt. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--name` `<name>` | Name der neuen Datei ohne Endung; erlaubt sind Buchstaben, Ziffern, _ und -. Erforderlich. |  |
| `--dir` `<ordner>` | Zielordner im Projekt (relativ zum aktuellen Ordner). | Standardordner der Art: music für Songs, sonst graphics |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# RMT-Modul übernehmen
vcc media import-rmt tune_pokey.rmt --name pokey
```

> **Hinweis:** Das Ergebnis hat immer das Profil atari.pokey.song. Hinweise stehen auf stderr.

> **Hinweis:** Der Projektordner ist der aktuelle Ordner; Pfade zu Projektdateien sind relativ dazu. Eine vorhandene Zieldatei wird nie überschrieben (Fehler).

### Form `import-rom`

Zeichensatz aus einem ROM-Abbild übernehmen

```
vcc media import-rom <rom> --profile <id> --name <name> [--offset <n>] [--dir <ordner>]
```

| Argument | Bedeutung |
|---|---|
| `rom` | ROM-Abbild (beliebiger Pfad). |

| Option | Bedeutung | Standard |
|---|---|---|
| `--profile` `<id>` | Zeichensatzprofil des Ziels. Erforderlich. |  |
| `--name` `<name>` | Name der neuen Datei ohne Endung; erlaubt sind Buchstaben, Ziffern, _ und -. Erforderlich. |  |
| `--offset` `<n>` | Byte-Offset des Zeichensatzes im Abbild (dezimal, $hex, 0xhex oder %binär). | `0` |
| `--dir` `<ordner>` | Zielordner im Projekt (relativ zum aktuellen Ordner). | Standardordner der Art: music für Songs, sonst graphics |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# zweiten C64-Zeichensatz aus chargen.bin
vcc media import-rom chargen.bin --profile c64.char.hires --offset 0x800 --name klein
```

> **Hinweis:** Fehlen --profile oder --name, nennt der Befehl auf stderr, was erkannt wurde (Größe, Prüfsumme), und listet passende Profile und Offsets (Exit-Code 2).

> **Hinweis:** ROM-Abbilder von Originalgeräten sind urheberrechtlich geschützt; die Kopie enthält die Original-Zeichen. Für ein Projekt, das veröffentlicht wird, besser `std-charset` (Verweis).

> **Hinweis:** Der Zeichensatz liegt als .vcchar im Zielordner (Standard graphics).

> **Hinweis:** Der Projektordner ist der aktuelle Ordner; Pfade zu Projektdateien sind relativ dazu. Eine vorhandene Zieldatei wird nie überschrieben (Fehler).

> **Hinweis:** Profil-IDs (z. B. c64.char.hires, c64.sprite.multicolor, atari.pmg.single, c64.sid.song) stehen in data/profile/profiles.json.

Rückgabe: `0` = fehlerfrei · `1` = Fehler · `2` = falscher Aufruf (auch: --profile oder --name fehlt)

### Form `std-charset-liste`

Quellen für Standardzeichensätze mit Status auflisten

```
vcc media std-charset
```

| Option | Bedeutung | Standard |
|---|---|---|
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# Quellen anzeigen
vcc media std-charset
```

> **Hinweis:** Je Zeile, durch Tabulator getrennt: Verweis, Profil, System, Status des ROMs, Beschreibung.

### Form `std-charset`

Standardzeichensatz als Verweis oder Kopie ins Projekt legen

```
vcc media std-charset <verweis> [--profile <id>] [--copy] [--dir <ordner>]
```

| Argument | Bedeutung |
|---|---|
| `verweis` | ROM-Verweis der Form rom:system/rolle#satz, z. B. rom:c64/chargen#upper. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--profile` `<id>` | Zeichensatzprofil. | das Profil, das die Liste für den Verweis nennt |
| `--copy` | Bettet die Zeichen in die Datei ein (Format 1) und schreibt nach graphics/standard/, statt nur einen Verweis zu speichern. |  |
| `--dir` `<ordner>` | Zielordner im Projekt; ohne --copy. | `graphics` |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# Verweis auf den C64-Zeichensatz
vcc media std-charset rom:c64/chargen#upper --profile c64.char.hires
```

```bash
# Zeichen einbetten
vcc media std-charset rom:c64/chargen#upper --profile c64.char.hires --copy
```

> **Hinweis:** Die Datei heißt <system>_<satz>.vcchar. Ohne --copy steht nur der Verweis in der Datei, die Originalzeichen bleiben im ROM des Benutzers. Eingebettete Kopien sind nicht zum Einchecken oder Weitergeben gedacht.

> **Hinweis:** Fehlt das ROM, wird die Datei trotzdem angelegt; ein Hinweis steht auf stderr.

> **Hinweis:** Der Projektordner ist der aktuelle Ordner; Pfade zu Projektdateien sind relativ dazu. Eine vorhandene Zieldatei wird nie überschrieben (Fehler).

> **Hinweis:** Profil-IDs (z. B. c64.char.hires, c64.sprite.multicolor, atari.pmg.single, c64.sid.song) stehen in data/profile/profiles.json.

### Form `import-sheet`

Zeichensatz aus einem PNG-Raster übernehmen

```
vcc media import-sheet <raster.png> --profile <id> --name <name> [--invert] [--dir <ordner>]
```

| Argument | Bedeutung |
|---|---|
| `raster.png` | PNG mit 16 Zeichen je Zeile in Leserichtung (ganzzahlig vergrößert erlaubt). |

| Option | Bedeutung | Standard |
|---|---|---|
| `--profile` `<id>` | Zeichensatzprofil mit 1 Bit je Pixel oder MEGA65-Vollfarbe. Erforderlich. |  |
| `--name` `<name>` | Name der neuen Datei ohne Endung; erlaubt sind Buchstaben, Ziffern, _ und -. Erforderlich. |  |
| `--invert` | Dunkle Pixel gelten als gesetzt statt heller. |  |
| `--dir` `<ordner>` | Zielordner im Projekt (relativ zum aktuellen Ordner). | Standardordner der Art: music für Songs, sonst graphics |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# Raster als Zeichensatz
vcc media import-sheet font.png --profile c64.char.hires --name font
```

```bash
# dunkle Zeichen auf hellem Grund
vcc media import-sheet font.png --profile c64.char.hires --name font2 --invert
```

> **Hinweis:** Das Bild muss zu 16 Spalten passen und ein ganzzahliges Vielfaches der Zeichengröße haben (128 x 128 Pixel bei 256 Zeichen zu 8 x 8). Transparente Pixel zählen als nicht gesetzt.

> **Hinweis:** Der Projektordner ist der aktuelle Ordner; Pfade zu Projektdateien sind relativ dazu. Eine vorhandene Zieldatei wird nie überschrieben (Fehler).

> **Hinweis:** Profil-IDs (z. B. c64.char.hires, c64.sprite.multicolor, atari.pmg.single, c64.sid.song) stehen in data/profile/profiles.json.

### Form `export-sheet`

Zeichensatz als PNG-Raster schreiben

```
vcc media export-sheet <zeichensatz.vcchar> <ausgabe.png>
```

| Argument | Bedeutung |
|---|---|
| `zeichensatz.vcchar` | Zeichensatz im Projekt. |
| `ausgabe.png` | Zieldatei. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# Zeichensatz als Bild
vcc media export-sheet graphics/font.vcchar font.png
```

> **Hinweis:** Das Raster hat 16 Zeichen je Zeile, weiß auf schwarz, 1:1. Nur Zeichensätze mit 1 Bit je Pixel und MEGA65-Vollfarbe.

> **Hinweis:** Der Projektordner ist der aktuelle Ordner; Pfade zu Projektdateien sind relativ dazu. Eine vorhandene Zieldatei wird nie überschrieben (Fehler).

### Form `import`

PNG-Bild in einen Frame von Sprite, PMG oder Zeichensatz einlesen

```
vcc media import <datei> <bild.png> [--frame <n>] [--system <pal|ntsc>] [--keep-colors]
```

| Argument | Bedeutung |
|---|---|
| `datei` | Mediendatei im Projekt (.vcsprite, .vcpmg oder .vcchar) mit eingebetteten Frames. |
| `bild.png` | PNG; wird auf die Frame-Größe skaliert, Pixel unter 50 % Deckkraft sind transparent. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--frame` `<n>` | Nummer des Frames, der überschrieben wird (0-basiert). | `0` |
| `--system` `<norm>` | Videonorm der Palette, pal oder ntsc; wirkt nur beim Atari. | `pal` |
| `--keep-colors` | Behält die Farben der Datei und rundet die Bildfarben darauf; sonst werden die Farben der Datei aus dem Bild bestimmt. |  |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# Bild in Sprite-Frame 0
vcc media import graphics/hero.vcsprite hero.png
```

```bash
# mit festen Farben, Frame 2
vcc media import graphics/hero.vcsprite hero.png --frame 2 --keep-colors
```

> **Hinweis:** Nur für Profile mit höchstens 16 Farben. Passt das Bild nicht in die darstellbaren Farben, meldet der Befehl einen Farbkonflikt und rundet auf die nächstliegenden Farben; die ersetzten Farben stehen in der Ausgabe.

> **Hinweis:** Die Datei wird an Ort und Stelle geändert.

> **Hinweis:** Der Projektordner ist der aktuelle Ordner; Pfade zu Projektdateien sind relativ dazu. Eine vorhandene Zieldatei wird nie überschrieben (Fehler).

### Form `export`

Mediendatei als Assembler, Binärdaten oder BASIC-DATA ausgeben

```
vcc media export <datei> [--format <asm|inc|bin|prg|data|sbas|rmt|bas>] [--label <name>] [--address <adresse>]
```

| Argument | Bedeutung |
|---|---|
| `datei` | Mediendatei im Projekt (.vcsprite, .vcpmg, .vcchar, .vcscreen, .vctiles, .vcpal, .vcsong, .vcplay). |

| Option | Bedeutung | Standard |
|---|---|---|
| `--format` `<format>` | Ausgabeformat: asm (.byte-Daten und .const), inc (nur .const für Länge und, mit --address, Adresse), bin, prg (mit Ladeadresse, braucht --address), data (BASIC-DATA mit @label), sbas (DATA für strukturiertes BASIC). POKEY-Songs: asm oder rmt; PLAY-Songs: bas oder data. Werte: `asm`, `inc`, `bin`, `prg`, `data`, `sbas`, `rmt`, `bas`. | `asm` |
| `--label` `<name>` | Marke der Daten im Quelltext; Teile bekommen ein Suffix. | Name der Datei, - durch _ ersetzt |
| `--address` `<adresse>` | Ladeadresse für prg, Adresse für inc, bei rmt Ladeadresse des Moduls ($hex, 0xhex oder dezimal). | bei rmt: $4000 |
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# Zeichensatz als Assembler
vcc media export graphics/font.vcchar --format asm
```

```bash
# Sprite als PRG bei $2000
vcc media export graphics/hero.vcsprite --format prg --address 0x2000
```

```bash
# Song als BASIC-DATA
vcc media export music/tune.vcsong --format data --label tune
```

```bash
# POKEY-Song als RMT-Modul
vcc media export music/tune_pokey.vcsong --format rmt --address 0x4000
```

> **Hinweis:** Die Dateien entstehen in build/export/; der Befehl gibt Pfad und Größe je Datei aus. Bei der Ausgabe in mehreren Teilen (Screens: chars/attrs, Kacheln: tiles/map/colors) gibt es je Teil eine Datei bzw. einen Abschnitt.

> **Hinweis:** Beim Bauen werden veraltete Mediendateien ohnehin als asm und data nach build/export/ geschrieben.

> **Hinweis:** Der Projektordner ist der aktuelle Ordner; Pfade zu Projektdateien sind relativ dazu. Eine vorhandene Zieldatei wird nie überschrieben (Fehler).

### Form `migrate-tiles`

alte Kachelkarten in Kachelsatz und Screen-Bank umwandeln

```
vcc media migrate-tiles
```

| Option | Bedeutung | Standard |
|---|---|---|
| `--data-dir` `<pfad>` | Ordner der Hardware-Datenbank; sonst VC_DATA oder neben dem Programm. |  |

Beispiele:

```bash
# Projekt umstellen
vcc media migrate-tiles
```

> **Hinweis:** Sucht im aktuellen Projektordner nach alten Kachelkarten (*.vctiles der Art tilemap). Dateien eines Ordners mit gleichem Kachelsatz ergeben einen Kachelsatz und eine Screen-Bank. Die alten Dateien werden entfernt.

> **Hinweis:** Der Projektordner ist der aktuelle Ordner; Pfade zu Projektdateien sind relativ dazu. Eine vorhandene Zieldatei wird nie überschrieben (Fehler).

## `vcc migrate`

Projekt vom alten Dateischema auf das neue umstellen

Benennt Dateien im alten Schema (*.asset.json) in <name>.vc<art> um, verschiebt assets/ nach graphics/ bzw. music/ und build/assets/ nach build/export/. Verweise in Quelltexten, Includes und Assets werden mitgezogen. Ausgegeben wird jede Umbenennung; build/export/ entsteht beim nächsten Bauen neu.

```
vcc migrate [projektordner]
```

| Argument | Bedeutung |
|---|---|
| `projektordner` | Wurzel des Projekts; Standard ist der Arbeitsordner. (optional) |

Beispiele:

```bash
# Projekt im Arbeitsordner umstellen
vcc migrate
```

```bash
# bestimmtes Projekt umstellen
vcc migrate examples/c64/tests_demo
```

> **Hinweis:** Gibt es nichts umzustellen, lautet die Ausgabe: nichts umzustellen.

> **Hinweis:** Der Befehl kennt keine Optionen; das erste Argument gilt als Ordner.

## `vcc new`

Neues Projekt aus einer Vorlage anlegen

Legt den Ordner an (falls nötig), schreibt die Projektdatei <name>.vcprj und die Quelltexte der Vorlage, je Zielsystem unter src/<system>/. Der Ordner darf noch keine .vcprj enthalten.

Mit --structure kommen die Ordner und das Layout einer Projektstruktur dazu. vcc new --list zeigt Vorlagen, Varianten und Projektstrukturen mit den möglichen Zielsystemen.

```
vcc new <ordner> [optionen]
```

| Argument | Bedeutung |
|---|---|
| `ordner` | Projektordner; sein Name ist der Projektname, wenn --name fehlt. Entfällt bei --list. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--target` `<system[,system…]>` | Zielsysteme, durch Komma getrennt. Jedes bekommt eigene Quellen unter src/<system>/. Die Vorlage muss das System kennen. | `c64` |
| `--template` `<vorlage>` | Dateivorlage: Assemblerprogramm, BASIC ohne Zeilennummern oder nur Projektdatei mit src/. Werte: `asm-hello`, `basic-hello`, `empty`. | `asm-hello` |
| `--variant` `<variante>` | Variante der Vorlage. cartridge erzeugt ein 8-KB-Modul statt eines Programms (nur asm-hello; c64, atari8, vic20). Werte: `cartridge`. | keine |
| `--structure` `<struktur>` | Projektstruktur: legt Ordner an und trägt das layout in die Projektdatei ein. Auch eigene Vorlagen aus structures.json sind möglich. Werte: `einfach`, `standard`, `gross`, `mehrsystem`, `bibliothek`, `spiel`. | keine |
| `--name` `<name>` | Projektname (Buchstaben, Ziffern, _ und -, höchstens 48 Zeichen); bestimmt den Namen der .vcprj. | Name des Ordners |
| `--list` | Projektstrukturen, Vorlagen und Varianten mit ihren Zielsystemen auflisten; legt nichts an. |  |
| `--data-dir` `<ordner>` | Ordner der Hardware-Datenbank (data). | VC_DATA, sonst neben dem Programm oder im Arbeitsordner |

Beispiele:

```bash
# einfaches C64-Projekt
vcc new spiel --target c64
```

```bash
# ein Projekt für mehrere Ziele
vcc new multi --target c64,atari8
```

```bash
# BASIC-Projekt mit Projektstruktur
vcc new demo --target c64,vic20 --template basic-hello --structure mehrsystem
```

```bash
# Steckmodul für den VC-20
vcc new modul --target vic20 --variant cartridge
```

```bash
# Vorlagen und Strukturen auflisten
vcc new --list
```

> **Hinweis:** Die Vorlage empty gibt es für c64, c128, plus4, vic20, atari8 und mega65; asm-hello zusätzlich für pet, basic-hello für pet und cbm2.

> **Hinweis:** Ausgegeben werden die angelegten Ordner und Dateien.

## `vcc organize`

Projektdateien nach der Projektstruktur umsortieren

Verschiebt Dateien des Projekts in die Ordner, die rules, layout und assets der Projektdatei vorgeben. Pfade in .include/.incbin der Quelltexte und die Projektdatei werden nachgeführt. Vorher entsteht eine Sicherung in .vcbackup/.

Es bleiben: Programme der Quellgruppen, Startprogramm, Disketten-Dateien, Studio-Dateien (ohne Ordner für studio in layout), .vctest-Dateien (außer in einem Ordner mit Rolle Tests), die Projektdatei, README, LICENSE, CHANGELOG und Dateien im Zielordner. Ist das Ziel belegt, wird die Datei übersprungen.

Nach dem Verschieben baut ein Prüfbau das Projekt neu und vergleicht die Ergebnisdateien mit denen von vorher; weichen sie ab, endet der Befehl mit Exit-Code 1.

```
vcc organize [projekt] [optionen]
```

| Argument | Bedeutung |
|---|---|
| `projekt` | Projektordner oder .vcprj-Datei; Standard ist der Arbeitsordner. (optional) |

| Option | Bedeutung | Standard |
|---|---|---|
| `--dry-run` | Geplante Verschiebungen und anzupassende Quelltexte anzeigen; nichts ändern. |  |
| `-y`, `--yes` | Ohne Rückfrage ausführen. Ohne Terminal ist --yes nötig, sonst ändert der Befehl nichts. | Rückfrage im Terminal |
| `--subdirs` | Auch Dateien in Unterordnern umordnen, nicht nur die im Projektordner. | nur Projektordner |
| `--no-rewrite` | Pfade in .include/.incbin der Quelltexte nicht anpassen. | Pfade werden angepasst |
| `--no-check` | Prüfbau vor und nach dem Verschieben auslassen. | Prüfbau läuft |
| `--data-dir` `<ordner>` | Ordner der Hardware-Datenbank (data) für den Prüfbau. Wird sie nicht gefunden, entfällt der Prüfbau mit Hinweis. | VC_DATA, sonst neben dem Programm oder im Arbeitsordner |

Beispiele:

```bash
# ansehen, was verschoben würde
vcc organize spiel --dry-run
```

```bash
# mit Rückfrage ausführen
vcc organize spiel
```

```bash
# ohne Rückfrage, auch Unterordner
vcc organize spiel --subdirs --yes
```

> **Hinweis:** Bei Abweichung im Prüfbau: Dateien aus .vcbackup/<Zeit>/ zurückkopieren und die in moves.txt genannten Dateien zurückverschieben. Die letzten fünf Sicherungen bleiben erhalten.

> **Hinweis:** Ohne zu verschiebende Dateien meldet der Befehl „Nichts zu ordnen.“

Rückgabe: `0` = fehlerfrei oder abgebrochen · `1` = Fehler, fehlende Bestätigung ohne Terminal oder abweichender Prüfbau

## `vcc roms`

ROM-Dateien anzeigen, prüfen und importieren

Verwaltet die ROM-Abbilder, die Emulatoren und Prüfungen brauchen. Systeme sind c64, c128, vic20, plus4, atari8 und mega65; eine Rolle ist eine einzelne ROM eines Systems (z. B. c64/kernal). Der ROM-Ordner lässt sich mit VC_ROMS ersetzen.

### Form `list`

Status je System und Rolle anzeigen

```
vcc roms list [system] [optionen]
```

| Argument | Bedeutung |
|---|---|
| `system` | Nur dieses System anzeigen. (optional) |

| Option | Bedeutung | Standard |
|---|---|---|
| `--json` | Gibt die Liste als JSON aus. |  |
| `--foreign` | Sucht auch in ROM-Ordnern anderer Programme. |  |
| `--project` `<ordner>` | Projektordner, in dem zuerst nach ROMs gesucht wird. |  |
| `--data-dir` `<ordner>` | Ordner der Hardware-Datenbank. | VC_DATA, sonst neben dem Programm oder im Arbeitsordner |

Beispiele:

```bash
# alle Systeme
vcc roms list
```

```bash
# nur C64 als JSON
vcc roms list c64 --json
```

> **Hinweis:** Ein Status ist Erkannt, Unbekannte Revision, Ersatz (frei), Fehlt oder Nicht benutzbar.

### Form `check`

prüfen, ob die erforderlichen ROMs vorhanden sind

```
vcc roms check [system ...] [optionen]
```

| Argument | Bedeutung |
|---|---|
| `system` | Zu prüfende Systeme; ohne Angabe alle. (optional) Mehrfach möglich. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--foreign` | Sucht auch in ROM-Ordnern anderer Programme. |  |
| `--project` `<ordner>` | Projektordner, in dem zuerst nach ROMs gesucht wird. |  |
| `--data-dir` `<ordner>` | Ordner der Hardware-Datenbank. | VC_DATA, sonst neben dem Programm oder im Arbeitsordner |

Beispiele:

```bash
# alle Systeme prüfen
vcc roms check
```

```bash
# C64 und Atari prüfen
vcc roms check c64 atari8
```

> **Hinweis:** Ersatz-ROMs gelten als vorhanden. Fehlt etwas und liegen passende Dateien in anderen Programmen, nennt die Ausgabe vcc roms import --from-others.

Rückgabe: `0` = alle erforderlichen ROMs vorhanden · `1` = eine erforderliche ROM fehlt · `2` = unbekanntes System

### Form `path`

Pfad der benutzten ROM-Datei ausgeben

```
vcc roms path <system/rolle> [optionen]
```

| Argument | Bedeutung |
|---|---|
| `system/rolle` | System und Rolle, z. B. c64/kernal. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--foreign` | Sucht auch in ROM-Ordnern anderer Programme. |  |
| `--project` `<ordner>` | Projektordner, in dem zuerst nach ROMs gesucht wird. |  |
| `--data-dir` `<ordner>` | Ordner der Hardware-Datenbank. | VC_DATA, sonst neben dem Programm oder im Arbeitsordner |

Beispiele:

```bash
# Pfad des C64-KERNAL
vcc roms path c64/kernal
```

Rückgabe: `0` = Pfad ausgegeben · `1` = die Datei fehlt oder ist nicht benutzbar · `2` = Angabe nicht in der Form system/rolle

### Form `where`

ROM-Ordner, Suchorte und portablen Modus anzeigen

```
vcc roms where [optionen]
```

| Option | Bedeutung | Standard |
|---|---|---|
| `--foreign` | Sucht auch in ROM-Ordnern anderer Programme. |  |
| `--project` `<ordner>` | Projektordner, in dem zuerst nach ROMs gesucht wird. |  |
| `--data-dir` `<ordner>` | Ordner der Hardware-Datenbank. | VC_DATA, sonst neben dem Programm oder im Arbeitsordner |

Beispiele:

```bash
# Suchorte anzeigen
vcc roms where
```

```bash
# mit ROMs anderer Programme
vcc roms where --foreign
```

> **Hinweis:** Zeigt Schreibziel, Benutzerordner, portablen Modus, Zwischenspeicher und die gesuchten Orte in der Reihenfolge der Suche.

### Form `import`

erkannte ROMs in den ROM-Ordner kopieren

```
vcc roms import <datei|ordner> ... [optionen]
```

| Argument | Bedeutung |
|---|---|
| `datei|ordner` | Dateien oder Ordner mit ROM-Abbildern; entfällt bei --from-others. Mehrfach möglich. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--system` `<system>` | Ordnet die Dateien diesem System zu, statt es zu erkennen. |  |
| `--from-others` | Übernimmt passende ROMs aus Ordnern anderer Programme statt aus Dateien. |  |
| `--yes` | Kopiert wirklich; ohne --yes zeigt der Befehl nur die Vorschau. |  |
| `--foreign` | Sucht auch in ROM-Ordnern anderer Programme. |  |
| `--project` `<ordner>` | Projektordner, in dem zuerst nach ROMs gesucht wird. |  |
| `--data-dir` `<ordner>` | Ordner der Hardware-Datenbank. | VC_DATA, sonst neben dem Programm oder im Arbeitsordner |

Beispiele:

```bash
# Vorschau für einen Ordner
vcc roms import ~/Downloads/c64roms
```

```bash
# ROMs anderer Programme übernehmen
vcc roms import --from-others --yes
```

> **Hinweis:** Ohne --yes wird nichts kopiert.

Rückgabe: `0` = Vorschau ausgegeben oder kopiert · `1` = Kopieren fehlgeschlagen · `2` = keine Dateien angegeben

### Form `emit-known`

Einträge für data/roms/known.json aus echten Dateien erzeugen

```
vcc roms emit-known [optionen]
```

| Option | Bedeutung | Standard |
|---|---|---|
| `--into` `<datei.json>` | Schreibt in diese Tabelle (wird gelesen und ergänzt, falls sie existiert); ohne Angabe nach stdout. |  |
| `--vice` `<ordner>` | Übernimmt die ROMs aus einem VICE-Datenordner. |  |
| `--merge-json` `<datei.json>` | Fügt Einträge aus einer JSON-Liste im Format von known.json hinzu. Mehrfach möglich. |  |
| `--file` `<datei>` | Beginnt einen Eintrag für diese ROM-Datei; die folgenden Optionen gehören dazu. Mehrfach möglich. |  |
| `--as` `<system/rolle>` | System und Rolle des Eintrags. Mehrfach möglich. |  |
| `--rev` `<rev>` | Revision des Eintrags. Mehrfach möglich. |  |
| `--free` | Markiert die ROM als frei (Ersatz-ROM). Mehrfach möglich. |  |
| `--default` | Markiert die Revision als Standard der Rolle. Mehrfach möglich. |  |
| `--note` `<text>` | Anmerkung zum Eintrag. Mehrfach möglich. |  |
| `--set` `<name:offset:size:profile[:text]>` | Benennt einen Bereich der ROM (Offset, Größe, Profil). Mehrfach möglich. |  |

Beispiele:

```bash
# aus VICE-Daten in eine Tabelle
vcc roms emit-known --vice /usr/lib/vice --into known.json
```

```bash
# eine Datei eintragen
vcc roms emit-known --file kernal.bin --as c64/kernal --rev 901227-03 --into known.json
```

> **Hinweis:** Werkzeug für die Pflege der Datenbank. --as, --rev, --free, --default, --note und --set gelten für das davor stehende --file; --as ist je --file Pflicht.

Rückgabe: `0` = Tabelle ausgegeben oder geschrieben · `1` = mindestens ein Eintrag fehlgeschlagen · `2` = Option ohne vorangehendes --file oder unbekannte Option

## `vcc streams`

Bild-, Ton- und Debug-Ströme der Ultimate 64 empfangen

Arbeitet mit den Datenströmen (Data Streams) einer Ultimate 64 im Netz. Die Unterbefehle starten die Ströme, werten sie aus und stoppen sie wieder. Sie ändern weder die Konfiguration des Geräts noch den Flash.

### Form `shot`

ein Bild der Ultimate als PNG speichern

```
vcc streams shot --host <host> --out <datei.png> [optionen]
```

| Option | Bedeutung | Standard |
|---|---|---|
| `--host` `<host>` | Adresse der Ultimate 64. Erforderlich. |  |
| `--out` `<datei.png>` | Zieldatei des Bildes. Erforderlich. |  |
| `--password` `<pw>` | Netzwerkpasswort; sonst das der Maschine mit gleicher Adresse aus den Einstellungen. |  |
| `--multicast` | Empfängt die Ströme per Multicast. |  |

Beispiele:

```bash
# Bild speichern
vcc streams shot --host 192.168.1.64 --out bild.png
```

> **Hinweis:** Wartet höchstens 5 Sekunden auf ein vollständiges Bild.

Rückgabe: `0` = Bild gespeichert · `1` = kein Bild empfangen oder Fehler · `2` = unbekannte Option

### Form `listen`

Lied über den echten SID der Ultimate anhören

```
vcc streams listen --host <host> <lied.vcsong> [optionen]
```

| Argument | Bedeutung |
|---|---|
| `lied.vcsong` | Lied, das als Abspielprogramm geladen wird. |

| Option | Bedeutung | Standard |
|---|---|---|
| `--host` `<host>` | Adresse der Ultimate 64. Erforderlich. |  |
| `--password` `<pw>` | Netzwerkpasswort; sonst das der Maschine mit gleicher Adresse aus den Einstellungen. |  |
| `--multicast` | Empfängt die Ströme per Multicast. |  |
| `--secs` `<sekunden>` | Dauer der Wiedergabe in Sekunden; Werte außerhalb von 2 bis 120 werden darauf begrenzt. | `6` |
| `--wav` `<datei.wav>` | Schreibt den empfangenen Ton als WAV-Datei (Quellrate). |  |
| `--ntsc` | Baut das Abspielprogramm und rechnet die Rate für NTSC statt PAL. |  |
| `--keep` | Lässt die Ultimate nach dem Hören laufen, statt sie zurückzusetzen. |  |

Beispiele:

```bash
# sechs Sekunden hören und als WAV sichern
vcc streams listen --host 192.168.1.64 lied.vcsong --wav lied.wav
```

> **Hinweis:** Lädt das Abspielprogramm auf die Ultimate und setzt sie dabei zurück; ihr laufender Zustand geht verloren.

> **Hinweis:** Gibt zuletzt Pakete, Verluste und Pufferstatistik aus.

Rückgabe: `0` = Ton empfangen · `1` = Fehler oder keine Pakete empfangen · `2` = unbekannte Option

### Form `probe`

Ströme starten, Pakete zählen, Rate und Verluste ausgeben

```
vcc streams probe --host <host> [optionen]
```

| Option | Bedeutung | Standard |
|---|---|---|
| `--host` `<host>` | Adresse der Ultimate 64. Erforderlich. |  |
| `--password` `<pw>` | Netzwerkpasswort; sonst das der Maschine mit gleicher Adresse aus den Einstellungen. |  |
| `--multicast` | Empfängt die Ströme per Multicast. |  |
| `--video` | Prüft den Bildstrom. Ohne --video, --audio und --debug werden Bild und Ton geprüft. |  |
| `--audio` | Prüft den Tonstrom. |  |
| `--debug` | Prüft den Debug-Strom. Läuft nie zugleich mit dem Bildstrom. |  |
| `--secs` `<sekunden>` | Dauer der Messung in Sekunden; Werte außerhalb von 2 bis 60 werden darauf begrenzt. | `5` |
| `--record` `<ordner>` | Schreibt die empfangenen Pakete je Strom in eine .bin-Datei in diesem Ordner. |  |
| `--count` `<anzahl>` | Höchstzahl der Pakete je Strom bei --record. | `400` |

Beispiele:

```bash
# Bild und Ton fünf Sekunden prüfen
vcc streams probe --host 192.168.1.64
```

```bash
# Debug-Strom prüfen
vcc streams probe --host 192.168.1.64 --debug --secs 10
```

> **Hinweis:** --video zusammen mit --debug ist ein Fehler.

> **Hinweis:** Je Strom erscheinen Zustand, Pakete (je Sekunde, Mbit/s), fehlende und verworfene Pakete sowie Neustarts.

Rückgabe: `0` = Ströme liefen ohne Fehler · `1` = Fehler bei einem Strom, bei --host oder bei der Aufzeichnung · `2` = unbekannte Option oder Unterbefehl

### Form `trace`

Debug-Strom (6510 Only) auswerten und zusammenfassen

```
vcc streams trace --host <host> [optionen]
```

| Option | Bedeutung | Standard |
|---|---|---|
| `--host` `<host>` | Adresse der Ultimate 64. Am Gerät muss der Debug-Strom-Modus 6510 Only stehen. Erforderlich. |  |
| `--password` `<pw>` | Netzwerkpasswort; sonst das der Maschine mit gleicher Adresse aus den Einstellungen. |  |
| `--multicast` | Empfängt die Ströme per Multicast. |  |
| `--secs` `<sekunden>` | Dauer der Aufzeichnung in Sekunden; Werte außerhalb von 1 bis 300 werden darauf begrenzt. | `5` |
| `--debug` `<datei.vcdbg>` | Debug-Datei des Programms: ordnet Befehle Quellzeilen zu, nennt nicht erreichte Zeilen und macht Symbole in --measure nutzbar. |  |
| `--measure` `<A:B>` | Misst die Zyklen von Adresse A bis Adresse B. Adressen als $1234, 0x1234, Dezimalzahl oder Symbol der .vcdbg. Mehrfach möglich. |  |
| `--cpu` `<id>` | CPU-Beschreibung für die Befehlstabelle. Der 45GS02 wird nicht ausgewertet. | `6510` |
| `--out` `<datei>` | Schreibt alle aufgezeichneten Befehle in diese Datei. |  |
| `--tail` `<anzahl>` | Anzahl der zuletzt ausgeführten Befehle in der Ausgabe. | `10` |
| `--data-dir` `<ordner>` | Ordner der Hardware-Datenbank. | VC_DATA, sonst neben dem Programm oder im Arbeitsordner |

Beispiele:

```bash
# fünf Sekunden aufzeichnen
vcc streams trace --host 192.168.1.64
```

```bash
# Zyklen zwischen zwei Adressen messen
vcc streams trace --host 192.168.1.64 --measure $0810:$0820 --secs 3
```

```bash
# mit Quellzeilen und Befehlsprotokoll
vcc streams trace --host 192.168.1.64 --debug build/prog.vcdbg --out ablauf.txt
```

> **Hinweis:** Hält den Bildstrom an, solange der Debug-Strom läuft.

> **Hinweis:** Die Ausgabe nennt Zyklen, Befehle (IRQ, NMI), Befehlserkennung, Paketverluste, Abdeckung, SID/VIC-Schreibzugriffe und die letzten Befehle.

Rückgabe: `0` = Auswertung ausgegeben · `1` = Fehler im Strom, nicht unterstützter Modus, Datei nicht lesbar oder schreibbar, ungültige Adresse · `2` = unbekannte Option

## `vcc test`

Tests aus *.vctest prüfen und im Emulator oder auf Hardware ausführen

Sucht unter dem Pfad alle Dateien *.vctest und führt zuerst ihre statischen Prüfungen aus. Mit --backend laufen zusätzlich die Einträge unter "tests": im Emulator (vice, atari800, fujisan, xemu) oder auf echter Hardware (u64, m65). Ohne --backend erscheinen diese Tests als übersprungen.

Vorher bauen (vcc build): der Läufer lädt das gebaute Programm. Testdateien mit "mode": "mailbox" prüfen sich selbst und werden über das Postfach im RAM gelesen, solche mit "mode": "measure" messen Zyklen (nur auf der Ultimate 64).

Ergebnisse je Test: OK, FEHLER, HINWEIS, ZEIT (Zeitüberschreitung), ÜBERSPR. Am Ende steht eine Zusammenfassung.

```
vcc test [ordner|datei.vctest] [optionen]
```

| Argument | Bedeutung |
|---|---|
| `ordner|datei.vctest` | Ordner (wird nach *.vctest durchsucht) oder einzelne Testdatei; Standard ist der Arbeitsordner. (optional) |

| Option | Bedeutung | Standard |
|---|---|---|
| `--backend` `<name>` | Führt die Tests zusätzlich dynamisch aus: vice, atari800 und xemu als Emulator, u64 und m65 auf echter Hardware (mit Adresse bzw. Schnittstelle und Sicherheitsabfrage). | keiner (nur statische Prüfungen) |
| `--filter` `<text>` | Führt nur die dynamischen Tests aus, die zum Text passen. |  |
| `--prologue` | Erzwingt den Vorspann in der Falle auch bei VICE, um den Weg der Adapter ohne Flag-Zugriff zu prüfen (Trap-Modus). |  |
| `--poll` | Postfach-Tests: liest das Postfach alle 100 ms, statt an der Endlosschleife von test_end anzuhalten. |  |
| `--password` `<pw>` | Netzwerkpasswort der Ultimate 64; sonst das der Maschine mit gleicher Adresse aus den Einstellungen. |  |
| `--yes` | Beantwortet die Sicherheitsabfrage vor einem Lauf auf Hardware mit Ja. Ohne Terminal ist --yes Pflicht. |  |
| `--strict` | Macht jeden Hinweis zum Fehler. |  |
| `--junit` `<datei.xml>` | Schreibt das Ergebnis zusätzlich als JUnit-XML. |  |
| `--data-dir` `<ordner>` | Ordner der Hardware-Datenbank. | VC_DATA, sonst neben dem Programm oder im Arbeitsordner |

Beispiele:

```bash
# statische Prüfungen eines Projekts
vcc test examples/c64/tests_demo
```

```bash
# zusätzlich in VICE ausführen
vcc test examples/c64/tests_demo --backend vice
```

```bash
# nur passende Tests im atari800
vcc test examples/atari/prallball --backend atari800 --filter ball
```

```bash
# Ergebnis als JUnit-XML schreiben
vcc test examples/mega65/prallball --junit ergebnis.xml
```

> **Hinweis:** --backend nimmt vice, atari800, fujisan, xemu (auch mit Version einer portabel installierten Ausgabe, z. B. vice@3.9), u64:HOST[:PORT] oder m65:SCHNITTSTELLE[@BAUD]; jeder andere Wert ist ein Fehler. u64 und m65 ohne Adresse werden abgelehnt.

> **Hinweis:** atari800 kennt nur das Ziel atari8, Xemu nur mega65. VICE läuft für c64 (x64sc), c128 (x128), plus4 (xplus4) und vic20 (xvic).

> **Hinweis:** Auf Hardware gehen Reset, Programm laden und Speicher beschreiben den laufenden Zustand des Geräts verloren. Dort laufen nur Postfach-Tests ("mode": "mailbox") und Zyklenmessungen; Trap-Tests werden übersprungen. Zyklenmessungen brauchen u64:HOST und am Gerät den Debug-Stream-Modus 6510 Only.

> **Hinweis:** Ohne gefundene *.vctest endet der Befehl fehlerfrei.

Rückgabe: `0` = alle bestanden (Hinweise und übersprungene Tests zählen nicht) · `1` = ein Test fehlgeschlagen, eine Testdatei fehlerhaft oder Zeitüberschreitung; auch Pfad oder Datenordner nicht gefunden, ungültiges --backend, Hardware-Abfrage abgelehnt · `2` = unbekannte Option

## Umgebungsvariablen

| Variable | Bedeutung |
|---|---|
| `VC_DATA` | Ordner der Hardware-Datenbank (data); sonst neben dem Programm oder im Arbeitsordner. |
| `VC_ROMS` | ROM-Ordner des Benutzers (statt roms/ im Konfigurationsordner). |
| `VC_XEMU` | Pfad zum Xemu-Programm (MEGA65-Emulator). |
| `VC_XEMU_SD` | Ordner mit MEGA65.ROM für Xemu. |
| `VC_XEMU_ARGS` | zusätzliche Argumente beim Start von Xemu, durch Leerzeichen getrennt. |
| `VC_STRUCTURES` | Datei mit eigenen Projektvorlagen (sonst structures.json im Konfigurationsordner). |
| `VC_DEBUG_LOG` | Debug-Protokoll einschalten: 1 (Standarddatei logs/vintagecoder.log im Konfigurationsordner) oder ein Dateipfad; 0 schaltet aus. Gilt für vintagecoder und vcc. |
| `VC_WIN` | Fenstergröße der IDE, z. B. 1400x860 (nur vintagecoder). |
| `COLUMNS` | Breite für den Zeilenumbruch der Hilfe (Standard: 100). |
| `NO_COLOR` | keine Farben in der Ausgabe der Hilfe. |

## Rückgabewerte

| Wert | Bedeutung |
|---|---|
| `0` | fehlerfrei |
| `1` | Fehler (im Quelltext, in einer Prüfung oder bei der Ausführung) |
| `2` | falscher Aufruf (unbekannter Befehl oder unbekannte Option) |
