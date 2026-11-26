<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# Codes und Meldungen von Assembler, BASIC und Werkzeugen

Erzeugt von `tools/diag_codes_doc.py` aus `crates/vc-i18n/catalog/de|en/diag-*.ftl`; nicht von Hand ändern (Änderungen im Katalog, dann Werkzeug erneut ausführen).

**Maschinenlesbar** liefert jede Meldung (`Diagnostic`, LSP `data`, `vcc --json`):

| Feld | Bedeutung |
|---|---|
| `code` | stabiler Code (E1xx–E3xx Fehler, W1xx/W2xx Warnungen, L… Lizenz, B1xx BASIC); gilt für Assembler **und** BASIC, einzelne Codes kommen in beiden vor |
| `key` | Katalogschlüssel `diag-<crate>-<code>[-<variante>]` (eindeutig, nie umbenannt); die Oberfläche formatiert damit |
| `args` | benannte Parameter (Zahl oder Text; ein Parameter kann selbst `{ key, args }` sein = verschachtelte Meldung) |
| `range` | `{ line, col, end_line, end_col }`, 1-basiert; Spalte 0 = ganze Zeile |
| `message` | fertiger Text in der gewählten Sprache (Anzeige, wenn die Oberfläche nicht selbst formatiert) |

Formatieren in der Oberfläche: Vorlage `key` im Katalog der Sprache nachschlagen, `args` als Fluent-Variablen übergeben; verschachtelte Parameter zuerst formatieren und als Text einsetzen. Fehlt `key`, den `message`-Text anzeigen.

450 Meldungen.

## B100

### `diag-basic-b100-bad-name` (Fehler): Ungültiger Name mit Punkt (Struktursprache)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `word` | der Name | a.b |

- de: ungültiger Name 'a.b' (Punkt nur in Konstanten wie asm.label)
- en: invalid name 'a.b' (a dot is only allowed in constants such as asm.label)

### `diag-basic-b100-bin-too-big` (Fehler): Binärzahl zu groß

- de: Binärzahl zu groß
- en: binary number is too big

### `diag-basic-b100-brace-missing` (Fehler): Schließende geschweifte Klammer eines Steuercodes fehlt

- de: '{ "}" }' fehlt
- en: '{ "}" }' is missing

### `diag-basic-b100-char-ref` (Fehler): Steuercode {…} in einer Zeichenkette fehlerhaft (Meldung der Zeichentabelle)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `text` | Meldung der Zeichentabelle | unbekannter Steuercode {nix} |

- de: unbekannter Steuercode {nix}
- en: unbekannter Steuercode {nix}

### `diag-basic-b100-hex-too-big` (Fehler): Hexzahl zu groß (Atari)

- de: Hexzahl zu groß
- en: hex number is too big

### `diag-basic-b100-label-name-missing` (Fehler): Name nach '@' fehlt

- de: Label-Name nach '@' fehlt
- en: label name after '@' is missing

### `diag-basic-b100-line-number-auto-max` (Fehler): Automatisch vergebene Zeilennummer größer als das Maximum

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `num` | die Zeilennummer | 64000 |
| `max` | das Maximum | 63999 |

- de: Zeilennummer 64000 größer als 63999 (zu viele Zeilen oder zu großer Abstand)
- en: line number 64000 is greater than 63999 (too many lines or step too large)

### `diag-basic-b100-line-number-max` (Fehler): Zeilennummer größer als das Maximum des Dialekts

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `digits` | die Zeilennummer | 70000 |
| `max` | das Maximum | 63999 |

- de: Zeilennummer 70000 größer als 63999
- en: line number 70000 is greater than 63999

### `diag-basic-b100-line-number-max-cbm` (Fehler): Zeilennummer größer als 63999 (CBM)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `num` | die Zeilennummer | 70000 |

- de: Zeilennummer 70000 größer als 63999
- en: line number 70000 is greater than 63999

### `diag-basic-b100-line-number-order` (Fehler): Zeilennummer nicht größer als die vorige

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `num` | die Zeilennummer | 10 |
| `last` | die vorige Zeilennummer | 20 |

- de: Zeilennummer 10 nicht größer als die vorige (20)
- en: line number 10 is not greater than the previous one (20)

### `diag-basic-b100-no-line-number` (Fehler): Zeile beginnt nicht mit einer Zeilennummer

- de: Zeile beginnt nicht mit einer Zeilennummer
- en: line does not start with a line number

### `diag-basic-b100-petscii-char` (Fehler): Zeichen ohne PETSCII-Entsprechung

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `c` | das Zeichen | é |

- de: Zeichen 'é' hat keine PETSCII-Entsprechung
- en: character 'é' has no PETSCII equivalent

### `diag-basic-b100-petscii-char-hint` (Fehler): Zeichen ohne PETSCII-Entsprechung in einer Zeichenkette mit Steuercodes

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `c` | das Zeichen | é |

- de: Zeichen 'é' hat keine PETSCII-Entsprechung (nutze { "{" }{ "$" }xx{ "}" })
- en: character 'é' has no PETSCII equivalent (use { "{" }{ "$" }xx{ "}" })

### `diag-basic-b100-unexpected-char` (Fehler): Unerwartetes Zeichen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `c` | das Zeichen | # |

- de: unerwartetes Zeichen '#'
- en: unexpected character '#'

## B101

### `diag-basic-b101-line-long` (Warnung): Zeile länger als die Zeilenlänge des Editors (nicht mehr bearbeitbar)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `max` | Zeichen, die der Editor des Dialekts zulässt | 80 |

- de: Zeile länger als 80 Zeichen: im Editor nicht mehr bearbeitbar
- en: line is longer than 80 characters: it can no longer be edited in the editor

## B102

### `diag-basic-b102-line-too-long` (Fehler): Zeile länger als 250 Bytes (CBM)

- de: Zeile zu lang (mehr als 250 Bytes)
- en: line is too long (more than 250 bytes)

### `diag-basic-b102-line-too-long-split` (Fehler): Zeile länger als 250 Bytes (Struktursprache)

- de: Zeile zu lang (mehr als 250 Bytes) – Zeile teilen
- en: line is too long (more than 250 bytes) - split the line

## B103

### `diag-basic-b103-reserved-word` (Fehler): Reserviertes Wort der Struktursprache als Variablenname

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | das Wort | loop |

- de: 'loop' ist ein reserviertes Wort der Struktursprache
- en: 'loop' is a reserved word of the structured language

## B104

### `diag-basic-b104-label-duplicate` (Fehler): Label doppelt definiert (Struktursprache)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name des Labels | schleife |

- de: Label 'schleife' doppelt definiert
- en: label 'schleife' is defined twice

## B105

### `diag-basic-b105-line-numbers-atari` (Fehler): Zeilennummern über 32767 (Atari)

- de: Zeilennummern über 32767 – step verkleinern
- en: line numbers above 32767 - reduce step

### `diag-basic-b105-line-numbers-cbm` (Fehler): Zeilennummern über 63999

- de: Zeilennummern über 63999 – step verkleinern oder pack nutzen
- en: line numbers above 63999 - reduce step or use pack

## B106

### `diag-basic-b106-block-not-closed` (Fehler): Block nicht geschlossen

- de: Block nicht geschlossen
- en: block is not closed

### `diag-basic-b106-do-syntax` (Fehler): do mit unzulässigem Zusatz

- de: do erwartet nichts, 'while Bedingung' oder 'until Bedingung'
- en: do expects nothing, 'while condition' or 'until condition'

### `diag-basic-b106-else-without-if` (Fehler): else ohne mehrzeiliges if

- de: else ohne passendes mehrzeiliges if
- en: else without a matching multi-line if

### `diag-basic-b106-elseif-without-if` (Fehler): elseif ohne mehrzeiliges if oder nach else

- de: elseif ohne passendes mehrzeiliges if (oder nach else)
- en: elseif without a matching multi-line if (or after else)

### `diag-basic-b106-endif-without-if` (Fehler): endif ohne if

- de: endif ohne if
- en: endif without if

### `diag-basic-b106-endproc-without-proc` (Fehler): endproc ohne proc

- de: endproc ohne proc
- en: endproc without proc

### `diag-basic-b106-exit-outside` (Fehler): exit außerhalb einer Schleife

- de: exit außerhalb von while, repeat oder do
- en: exit outside of while, repeat or do

### `diag-basic-b106-loop-syntax` (Fehler): loop mit unzulässigem Zusatz

- de: loop erwartet nichts, 'while Bedingung' oder 'until Bedingung'
- en: loop expects nothing, 'while condition' or 'until condition'

### `diag-basic-b106-loop-without-do` (Fehler): loop ohne do

- de: loop ohne do
- en: loop without do

### `diag-basic-b106-until-without-repeat` (Fehler): until ohne repeat

- de: until ohne repeat
- en: until without repeat

### `diag-basic-b106-wend-without-while` (Fehler): wend ohne while

- de: wend ohne while
- en: wend without while

## B107

### `diag-basic-b107-label-undefined` (Fehler): Label nicht definiert (Struktursprache)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name des Labels | ende |

- de: Label '@ende' nicht definiert
- en: label '@ende' is not defined

## B108

### `diag-basic-b108-const-syntax` (Fehler): const ohne NAME = Wert

- de: const erwartet NAME = Wert
- en: const expects NAME = value

### `diag-basic-b108-const-value` (Fehler): const mit unzulässigem Wert

- de: const erwartet eine einzelne Zahl oder Zeichenkette
- en: const expects a single number or string

## B109

### `diag-basic-b109-proc-arg-count` (Fehler): Falsche Zahl von Argumenten beim Aufruf einer Prozedur

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Prozedur | zeichne |
| `expected` | erwartete Zahl | 2 |
| `given` | übergebene Zahl | 1 |

- de: Prozedur 'zeichne' erwartet 2 Argument(e), übergeben: 1
- en: (mit Mehrzahl/Auswahl) `procedure '{ $name }' expects { $expected ->         [one] 1 argument        *[other] { $expected } arguments     }, given: { $given }`

### `diag-basic-b109-proc-head-syntax` (Fehler): Prozedurkopf ohne Klammern um die Parameter

- de: erwartet name(parameter, …)
- en: expected name(parameter, ...)

### `diag-basic-b109-proc-name-missing` (Fehler): Name der Prozedur fehlt

- de: Prozedurname fehlt
- en: procedure name is missing

### `diag-basic-b109-proc-nested` (Fehler): proc innerhalb einer proc

- de: proc darf nicht in einer anderen proc stehen
- en: proc must not be inside another proc

### `diag-basic-b109-proc-param-name` (Fehler): Ungültiger Parametername einer Prozedur

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `p` | der Name | 1x |

- de: ungültiger Parametername '1x'
- en: invalid parameter name '1x'

### `diag-basic-b109-proc-undefined` (Fehler): Prozedur nicht definiert

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Prozedur | zeichne |

- de: Prozedur 'zeichne' ist nicht definiert
- en: procedure 'zeichne' is not defined

## B110

### `diag-basic-b110-include-depth` (Fehler): include zu tief verschachtelt

- de: include zu tief verschachtelt
- en: include is nested too deeply

### `diag-basic-b110-include-not-found` (Fehler): Include-Datei nicht gefunden

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Dateiname | fehlt.bas |

- de: Include-Datei 'fehlt.bas' nicht gefunden
- en: include file 'fehlt.bas' not found

### `diag-basic-b110-include-read` (Fehler): Include-Datei nicht lesbar

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `path` | Pfad der Datei | lib/d.bas |
| `error` | Meldung des Systems | permission denied |

- de: lib/d.bas: permission denied
- en: lib/d.bas: permission denied

### `diag-basic-b110-label-undefined` (Fehler): Label nicht definiert

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name des Labels | schleife |

- de: Label @schleife ist nicht definiert
- en: label @schleife is not defined

## B111

### `diag-basic-b111-cond-char` (Fehler): Unzulässiges Zeichen in der #if-Bedingung

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `c` | das Zeichen | + |

- de: Zeichen '+' in der Bedingung nicht erlaubt
- en: character '+' is not allowed in the condition

### `diag-basic-b111-cond-incomplete` (Fehler): #if-Bedingung unvollständig

- de: Ausdruck unvollständig
- en: expression is incomplete

### `diag-basic-b111-cond-rparen` (Fehler): Schließende Klammer in der #if-Bedingung fehlt

- de: ')' fehlt
- en: ')' is missing

### `diag-basic-b111-cond-unexpected` (Fehler): Unerwartetes Wort in der #if-Bedingung

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `t` | das Wort | and |

- de: unerwartet 'and'
- en: unexpected 'and'

### `diag-basic-b111-cond-unknown-name` (Fehler): Unbekannter Name in der #if-Bedingung

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `t` | der Name | system_c65 |

- de: unbekannter Name 'system_c65' (bekannt: SYSTEM_C64, SYSTEM_C128, SYSTEM_PLUS4, SYSTEM_VIC20, SYSTEM_MEGA65, SYSTEM_ATARI und -D-Konstanten)
- en: unknown name 'system_c65' (known: SYSTEM_C64, SYSTEM_C128, SYSTEM_PLUS4, SYSTEM_VIC20, SYSTEM_MEGA65, SYSTEM_ATARI and -D constants)

### `diag-basic-b111-directive-without-if` (Fehler): #elseif, #else oder #endif ohne #if

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `word` | die Direktive | #endif |

- de: #endif ohne #if
- en: #endif without #if

### `diag-basic-b111-else-after-else` (Fehler): #elseif oder #else nach #else

- de: #elseif/#else nach #else
- en: #elseif/#else after #else

### `diag-basic-b111-if-without-endif` (Fehler): #if ohne #endif

- de: #if ohne #endif
- en: #if without #endif

### `diag-basic-b111-include-system` (Fehler): {system} im include-Pfad, Zielsystem unbekannt

- de: { "{" }system{ "}" } im include-Pfad, aber das Zielsystem ist unbekannt
- en: { "{" }system{ "}" } in the include path, but the target system is unknown

### `diag-basic-b111-label-duplicate` (Fehler): Label doppelt definiert

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name des Labels | schleife |

- de: Label @schleife ist doppelt definiert
- en: label @schleife is defined twice

## B112

### `diag-basic-b112-label-at-end` (Fehler): Label am Ende ohne Anweisung dahinter

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name des Labels | ende |

- de: Label @ende steht am Ende ohne Anweisung dahinter
- en: label @ende is at the end with no statement after it

### `diag-basic-b112-restore-label` (Fehler): RESTORE mit Marke im BASIC V2

- de: restore mit Marke gibt es im BASIC V2 nicht (erst ab BASIC 3.5): ohne Marke zurücksetzen und der Reihe nach lesen
- en: restore with a label does not exist in BASIC V2 (only from BASIC 3.5): reset without a label and read in order

## B113

### `diag-basic-b113-blob-syntax` (Fehler): blob mit falscher Schreibweise

- de: blob erwartet: blob "datei.asm" [with "song.asm"]
- en: blob expects: blob "file.asm" [with "song.asm"]

## B114

### `diag-ide-b114-address-unstable` (Fehler): Adresse des Maschinencodes konvergiert nicht

- de: Adresse des Maschinencodes wird nicht stabil
- en: The address of the machine code does not settle

### `diag-ide-b114-code-too-large` (Fehler): Maschinencode zu groß

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `bytes` | Größe in Bytes | 20000 |

- de: Maschinencode zu groß (20000 Bytes)
- en: Machine code is too large (20000 bytes)

### `diag-ide-b114-file-not-found` (Fehler): Blob-Datei nicht gefunden

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Dateiname des Blobs | player.asm |

- de: #asm: Datei 'player.asm' nicht gefunden
- en: #asm: file 'player.asm' not found

### `diag-ide-b114-no-code` (Fehler): Blob erzeugt keinen Code

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Dateiname des Blobs | player.asm |

- de: #asm 'player.asm' erzeugt keinen Code
- en: #asm 'player.asm' produces no code

### `diag-ide-b114-org-mismatch` (Fehler): Blob beginnt nicht an der erwarteten Adresse

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Dateiname des Blobs | player.asm |
| `start` | tatsächliche Startadresse (hex, ohne $) | C100 |
| `expected` | erwartete Adresse (hex, ohne $) | C000 |

- de: Code 'player.asm' muss mit `.org asm_base` beginnen (beginnt bei $C100, erwartet $C000)
- en: code 'player.asm' must start with `.org asm_base` (starts at $C100, expected $C000)

### `diag-ide-b114-overlap` (Fehler): Segmente eines Blobs überlappen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Dateiname des Blobs | player.asm |

- de: #asm 'player.asm': überlappende Segmente
- en: #asm 'player.asm': overlapping segments

### `diag-ide-b114-program-too-large` (Fehler): BASIC-Programm samt Maschinencode zu groß

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `end` | Endadresse (hex, ohne $) | 10A00 |

- de: Programm samt Maschinencode zu groß (endet bei $10A00)
- en: Program including machine code is too large (ends at $10A00)

### `diag-ide-b114-read-error` (Fehler): Blob-Datei nicht lesbar

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `path` | Pfad der Datei | /home/u/p/player.asm |
| `error` | Fehlertext des Betriebssystems | Permission denied |

- de: /home/u/p/player.asm: Permission denied
- en: /home/u/p/player.asm: Permission denied

## B120

### `diag-basic-b120-include-not-found` (Fehler): #include-Datei nicht gefunden

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Dateiname | fehlt.bas |

- de: #include "fehlt.bas": Datei nicht gefunden (bei exportierten Dateien: erst bauen bzw. exportieren)
- en: #include "fehlt.bas": file not found (for exported files: build or export first)

## B121

### `diag-basic-b121-include-nesting` (Fehler): #include zu tief verschachtelt oder zyklisch

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Dateiname | lib/d.bas |

- de: #include "lib/d.bas": zu tief verschachtelt oder zyklisch
- en: #include "lib/d.bas": nested too deeply or circular

## B130

### `diag-basic-b130-not-digit-after` (Fehler): Zeichen hinter einer Hexzahl ist keine Hexziffer

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `c` | das Zeichen | G |

- de: 'G' ist keine Hexziffer (vor einem Schlüsselwort ein Leerzeichen setzen)
- en: 'G' is not a hex digit (put a space before a keyword)

### `diag-basic-b130-not-hex-digit` (Fehler): Zeichen ist keine Hexziffer

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `c` | das Zeichen | G |

- de: 'G' ist keine Hexziffer (0–9, A–F)
- en: 'G' is not a hex digit (0-9, A-F)

### `diag-basic-b130-separator` (Fehler): Trenner _ falsch gesetzt (Hexzahl)

- de: Trenner _ nur einzeln zwischen zwei Ziffern
- en: separator _ is only allowed singly between two digits

## B131

### `diag-basic-b131-no-short-name` (Fehler): Kein freier Kurzname mehr für einen langen Variablennamen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `key` | der lange Name | punktestand |

- de: Für den Namen 'punktestand' ist kein freier Kurzname mehr übrig (zu viele Variablen)
- en: no free short name is left for the name 'punktestand' (too many variables)

### `diag-basic-b131-not-binary-digit` (Fehler): Zeichen ist keine Binärziffer

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `c` | das Zeichen | 2 |

- de: '2' ist keine Binärziffer (nur 0 und 1)
- en: '2' is not a binary digit (only 0 and 1)

### `diag-basic-b131-not-digit-after` (Fehler): Zeichen hinter einer Binärzahl ist keine Binärziffer

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `c` | das Zeichen | x |

- de: 'x' ist keine Binärziffer
- en: 'x' is not a binary digit

### `diag-basic-b131-separator` (Fehler): Trenner _ falsch gesetzt (Binärzahl)

- de: Trenner _ nur einzeln zwischen zwei Ziffern
- en: separator _ is only allowed singly between two digits

## B132

### `diag-basic-b132-no-digits` (Fehler): Hex- oder Binärzahl ohne Ziffern

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `prefix` | Zeichen $ oder % | $ |

- de: $ ohne Ziffern
- en: $ without digits

## B133

### `diag-basic-b133-bin-bits` (Fehler): Binärzahl zu groß für die Bitbreite des Dialekts

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `bits` | Bitbreite | 16 |
| `max` | größter Wert | 65535 |

- de: Binärzahl über 16 Bit (größter Wert 65535)
- en: binary number above 16 bits (largest value 65535)

### `diag-basic-b133-hex-bits` (Fehler): Hexzahl zu groß für die Bitbreite des Dialekts

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `bits` | Bitbreite | 16 |
| `max` | größter Wert | 65535 |

- de: Hexzahl über 16 Bit (größter Wert 65535)
- en: hex number above 16 bits (largest value 65535)

## B134

### `diag-basic-b134-address-range` (Fehler): Adresse außerhalb 0–65535 (Warnung, wenn die Prüfung auf Warnung steht)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `v` | der Wert | 70000 |

- de: Adresse 70000 außerhalb 0–65535
- en: address 70000 is outside 0-65535

## B135

### `diag-basic-b135-value-range` (Fehler): Wert außerhalb 0–255 (Warnung, wenn die Prüfung auf Warnung steht)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `v` | der Wert | 300 |
| `kw` | das Schlüsselwort | POKE |

- de: Wert 300 außerhalb 0–255 (POKE schreibt ein Byte)
- en: value 300 is outside 0-255 (POKE writes one byte)

## B136

### `diag-basic-b136-hex-keyword` (Warnung): Hexzahl grenzt an ein Schlüsselwort

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `kw` | das Schlüsselwort | OR |

- de: Hexzahl grenzt an das Schlüsselwort OR: Leerzeichen setzen (Ziffern A–F könnten zum Wort gehören)
- en: hex number is next to the keyword OR: add a space (digits A-F could belong to the word)

## B137

### `diag-basic-b137-int-range` (Warnung): Wert passt nicht in eine Ganzzahlvariable

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `v` | der Wert | 40000 |

- de: 40000 passt nicht in eine Ganzzahlvariable (−32768 … 32767)
- en: 40000 does not fit in an integer variable (-32768 ... 32767)

## B138

### `diag-basic-b138-jump-target` (Fehler): Hex- oder Binärzahl als Sprungziel

- de: Hex/Binär als Sprungziel: Zeilennummern entstehen beim Bauen, Marke verwenden (@name)
- en: hex/binary as a jump target: line numbers are created when building, use a label (@name)

## B139

### `diag-basic-b139-c-hex` (Warnung): Hexzahl in C-Schreibweise

- de: 0x… ist kein BASIC: Hexzahlen schreibt man { "$" }FF
- en: 0x... is not BASIC: write hex numbers as { "$" }FF

## B140

### `diag-basic-b140-classic-literal` (Warnung): Hex- oder Binärzahl wird im klassischen Weg nicht umgewandelt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `t` | das Literal | $FF |
| `file` | Name der Quelldatei | spiel.bas |

- de: $FF wird im klassischen Weg nicht umgewandelt, der Dialekt kennt keine Hex-/Binärzahlen (als natives BASIC bauen: vcc basic spiel.bas, oder Dezimalzahl schreiben)
- en: $FF is not converted in the classic path, the dialect has no hex/binary numbers (build as native BASIC: vcc basic spiel.bas, or write a decimal number)

## B150

### `diag-basic-b150-keyword-in-name` (Warnung): Ein Schlüsselwort steckt mitten in einem Namen; der Interpreter zerlegt den Namen beim Tokenisieren

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | der Name | SCORE |
| `keyword` | das darin enthaltene Schlüsselwort | OR |

- de: 'SCORE' enthält das Schlüsselwort OR und wird zerlegt
- en: 'SCORE' contains the keyword OR and is split up

### `diag-basic-b150-keyword-in-name-more` (Warnung): derselbe Name kommt in weiteren Zeilen vor (eine Meldung je Name und Datei)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | der Name | SCORE |
| `keyword` | das darin enthaltene Schlüsselwort | OR |
| `lines` | weitere Zeilen | 5, 7 |

- de: 'SCORE' enthält das Schlüsselwort OR und wird zerlegt (auch in Zeile 5, 7)
- en: 'SCORE' contains the keyword OR and is split up (also on line 5, 7)

## B151

### `diag-basic-b151-for-without-next` (Warnung): FOR ohne passendes NEXT

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `var` | Schleifenvariable | I |

- de: FOR I hat kein passendes NEXT
- en: FOR I has no matching NEXT

### `diag-basic-b151-next-without-for` (Warnung): NEXT ohne passendes FOR

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `var` | Schleifenvariable (leer ohne Angabe) | I |

- de: NEXT I hat kein passendes FOR
- en: NEXT I has no matching FOR

## B152

### `diag-basic-b152-gosub-without-return` (Warnung): GOSUB, aber nirgends ein RETURN

- de: GOSUB ohne irgendein RETURN im Programm
- en: GOSUB without any RETURN in the program

### `diag-basic-b152-return-without-gosub` (Warnung): RETURN, aber nirgends ein GOSUB

- de: RETURN ohne irgendein GOSUB im Programm
- en: RETURN without any GOSUB in the program

## B153

### `diag-basic-b153-target-missing` (Warnung): Sprungziel (Nummer) nicht vorhanden

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `num` | die Nummer | 500 |

- de: Sprungziel 500 ist keine Programmzeile
- en: jump target 500 is not a program line

## B154

### `diag-basic-b154-variable-clash` (Warnung): Zwei Variablennamen unterscheiden sich erst hinter den Zeichen, die der Interpreter beachtet

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | der zweite Name | COUNTER |
| `other` | der erste Name | COUNT |
| `chars` | Zahl der beachteten Zeichen | 2 |

- de: COUNTER und COUNT sind für den Interpreter dieselbe Variable (nur die ersten 2 Zeichen zählen)
- en: COUNTER and COUNT are the same variable to the interpreter (only the first 2 characters count)

## B155

### `diag-basic-b155-string-open` (Hinweis): Zeichenkette am Zeilenende nicht geschlossen (im BASIC erlaubt)

- de: Zeichenkette am Zeilenende nicht geschlossen
- en: string not closed at the end of the line

## B156

### `diag-basic-b156-arg-count` (Warnung): Anweisung oder Funktion mit falscher Zahl von Argumenten

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `word` | das Wort | POKE |
| `count` | gefundene Zahl | 1 |
| `expected` | erlaubte Zahl oder Bereich | 2 |

- de: POKE mit 1 Argument(en); erwartet: 2
- en: POKE with 1 argument(s); expected: 2

## B157

### `diag-basic-b157-keyword-unknown` (Warnung): Anweisung gehört zu einem anderen BASIC-Dialekt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `word` | das Wort | GRAPHIC |
| `dialect` | gewählter Dialekt | cbm_v2 |

- de: GRAPHIC gehört nicht zum Dialekt cbm_v2
- en: GRAPHIC is not part of dialect cbm_v2

## B159

### `diag-basic-b159-function-paren` (Warnung): Funktion ohne Klammer

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `word` | die Funktion | RND |

- de: RND braucht ein Argument in Klammern
- en: RND needs an argument in parentheses

## B160

### `diag-basic-b160-mixed-mode` (Warnung): Die Datei mischt Zeilennummern und Labels (Schnellkorrektur: in Labels oder in Zeilennummern umwandeln)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `numbers` | Zahl der Zeilen mit Nummer | 12 |
| `labels` | Zahl der Zeilen mit Label oder Verweis | 3 |

- de: (mit Mehrzahl/Auswahl) `Zeilennummern und Labels sind gemischt ({ $numbers ->         [one] 1 Zeile        *[other] { $numbers } Zeilen     } mit Nummer, { $labels } mit Label)`
- en: (mit Mehrzahl/Auswahl) `line numbers and labels are mixed ({ $numbers ->         [one] 1 line        *[other] { $numbers } lines     } with a number, { $labels } with a label)`

## B161

### `diag-basic-b161-convert-failed` (Fehler): Umwandlung nicht möglich, weil Sprungziele berechnet werden (Zusammenfassung; die Stellen stehen als B162)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `count` | Zahl der Stellen | 2 |

- de: Umwandlung in Labels nicht möglich: 2 berechnete(s) Sprungziel(e)
- en: conversion to labels is not possible: 2 computed jump target(s)

### `diag-basic-b161-range-labels` (Fehler): Neu-Nummerieren eines Bereichs nicht möglich, weil Zeilen ohne Nummer vorkommen

- de: Bereich neu nummerieren geht nur im Nummernmodus (es gibt Zeilen ohne Nummer)
- en: renumbering a range only works in line-number mode (some lines have no number)

### `diag-basic-b161-range-order` (Fehler): Neu-Nummerieren eines Bereichs würde die Reihenfolge der Zeilen ändern

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `num` | neue Nummer | 120 |
| `last` | davor stehende Nummer | 150 |

- de: Bereich neu nummerieren würde die Reihenfolge ändern (Zeile 120 nach 150)
- en: renumbering the range would change the order of the lines (line 120 after 150)

## B162

### `diag-basic-b162-computed-target` (Fehler): Sprungziel wird berechnet und lässt sich nicht in ein Label umwandeln (Warnung beim Neu-Nummerieren: das Ziel kann danach falsch zeigen)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `word` | Sprungwort | GOTO |
| `text` | Anweisung | GOTO A*10 |

- de: berechnetes Sprungziel hinter GOTO: GOTO A*10
- en: computed jump target after GOTO: GOTO A*10

## B163

### `diag-basic-b163-runtime-line` (Warnung): Das Programm wertet die Zeilennummer zur Laufzeit aus (z. B. EL); Umwandlung oder Neu-Nummerieren ändert den Wert

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `word` | das Wort | EL |

- de: EL liefert die Zeilennummer zur Laufzeit; nach Umwandlung oder Neu-Nummerieren stimmen die Nummern nicht mehr
- en: EL returns the line number at run time; after a conversion or renumbering the numbers no longer match

## B165

### `diag-basic-b165-marker-orphan` (Warnung): Markerzeile ohne folgende Anweisung (verwaist; bleibt erhalten)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name | schleife |

- de: Marker ;@schleife steht ohne folgende Anweisung
- en: marker ;@schleife has no statement after it

## B166

### `diag-basic-b166-include-mode` (Fehler): Dateien einer Einbindung haben verschiedene Modi

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `file` | die Datei | lib/daten.bas |
| `mode` | numbers oder labels | labels |

- de: lib/daten.bas: Modus labels passt nicht zu den übrigen Dateien der Einbindung
- en: lib/daten.bas: mode labels does not match the other files of the include set

## B167

### `diag-basic-b167-quote-not-comment` (Warnung): Das Zeichen ' ist in diesem Dialekt kein Kommentar

- de: ' ist in diesem BASIC kein Kommentarzeichen (Kommentare: REM)
- en: ' is not a comment character in this BASIC (comments: REM)

## B168

### `diag-basic-b168-marker-invalid` (Warnung): Markerzeile ungültig oder Name doppelt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `text` | die Zeile | ;@1x |

- de: ungültige oder doppelte Markerzeile: ;@1x
- en: invalid or duplicate marker line: ;@1x

## B170

### `diag-basic-b170-dialect-system` (Warnung): Der gewählte BASIC-Dialekt passt nicht zum Zielsystem

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `id` | Kennung des Dialekts | vic_basic_2 |
| `system` | Kennung des Zielsystems | c64 |

- de: Dialekt 'vic_basic_2' ist nicht für das Zielsystem 'c64' vorgesehen
- en: dialect 'vic_basic_2' is not intended for target system 'c64'

## B172

### `diag-basic-b172-define-keyword` (Fehler): Der Name in #define ist ein Schlüsselwort des Dialekts

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | der Name | PRINT |

- de: #define: 'PRINT' ist ein Schlüsselwort und darf nicht als Name dienen
- en: #define: 'PRINT' is a keyword and cannot be used as a name

## B173

### `diag-basic-b173-blob-range` (Fehler): Die DATA-Zeilen des Maschinencodes passen nicht mehr hinter die letzte Zeilennummer

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `last` | höchste Zeilennummer | 63000 |
| `lines` | Zahl der DATA-Zeilen | 200 |
| `max` | größte erlaubte Nummer | 63999 |

- de: Zeilennummern reichen nicht für die DATA-Zeilen des Maschinencodes (200 Zeilen nach 63000, höchstens 63999)
- en: line numbers are not enough for the DATA lines of the machine code (200 lines after 63000, at most 63999)

## B174

### `diag-basic-b174-define-syntax` (Fehler): #define hat keinen gültigen Namen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `text` | die Zeile | #define 1X 5 |

- de: #define erwartet einen Namen aus Buchstaben, Ziffern und _ : #define 1X 5
- en: #define expects a name made of letters, digits and _ : #define 1X 5

### `diag-basic-b174-define-twice` (Fehler): Name zweimal mit #define festgelegt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | der Name | SCR |

- de: #define: 'SCR' ist schon festgelegt
- en: #define: 'SCR' is already defined

## B175

### `diag-basic-b175-else-twice` (Fehler): zweites #else im selben #if

- de: zweites #else im selben #if
- en: second #else in the same #if

### `diag-basic-b175-else-without-if` (Fehler): #else ohne #if

- de: #else ohne #if
- en: #else without #if

### `diag-basic-b175-endif-without-if` (Fehler): #endif ohne #if

- de: #endif ohne #if
- en: #endif without #if

### `diag-basic-b175-if-condition` (Fehler): Bedingung von #if nicht lesbar

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `text` | die Bedingung | SYSTEM-C64 |

- de: #if: Bedingung 'SYSTEM-C64' nicht lesbar (erlaubt: SYSTEM_C64, ein definierter Name, vorangestelltes !)
- en: #if: condition 'SYSTEM-C64' not readable (allowed: SYSTEM_C64, a defined name, a leading !)

### `diag-basic-b175-if-not-closed` (Fehler): #if ohne #endif

- de: #if ohne #endif
- en: #if without #endif

## B176

### `diag-basic-b176-define-cycle` (Fehler): Ersetzung eines #define-Namens ist zyklisch oder zu tief verschachtelt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | der Name | A |

- de: #define: 'A' verweist auf sich selbst (zyklisch oder mehr als acht Stufen)
- en: #define: 'A' refers to itself (cyclic or more than eight levels)

## B177

### `diag-basic-b177-address-unstable` (Fehler): Die Adresse des Maschinencodes wird nicht stabil

- de: Adresse des Maschinencodes wird nicht stabil (BASIC-Text und Zahlen beeinflussen sich)
- en: address of the machine code does not settle (BASIC text and numbers influence each other)

### `diag-basic-b177-blob-here` (Fehler): #asm ist hier nicht möglich (Übersetzen ohne Anbindung des Assemblers)

- de: #asm braucht den Bau mit Maschinencode (vcc basic bzw. Bauen in der IDE mit Assembler-Anbindung)
- en: #asm needs the build with machine code (vcc basic, or building in the IDE with the assembler attached)

### `diag-basic-b177-blob-renamed` (Fehler): die frühere Schreibweise #blob gibt es nicht mehr

- de: #blob heißt jetzt #asm: #asm "datei.asm" [with "song.asm"]
- en: #blob is now called #asm: #asm "file.asm" [with "song.asm"]

### `diag-basic-b177-blob-syntax` (Fehler): #asm hat keine gültige Form

- de: #asm erwartet: #asm "datei.asm" [with "song.asm"]
- en: #asm expects: #asm "file.asm" [with "song.asm"]

### `diag-basic-b177-code-too-large` (Fehler): Der Maschinencode ist zu groß für den Speicher unter dem BASIC-ROM

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `bytes` | Größe des Codes | 30000 |

- de: Maschinencode (30000 Bytes) passt nicht unter { "$" }A000
- en: machine code (30000 bytes) does not fit below { "$" }A000

### `diag-basic-b177-symbol-missing` (Fehler): Der Programmtext braucht eine Marke, die der Maschinencode nicht festlegt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | die Marke (klein) | start |

- de: asm.start: der Maschinencode legt keine Marke 'start' fest
- en: asm.start: the machine code does not define a symbol 'start'

### `diag-basic-b177-too-large` (Fehler): Code und BASIC-Text passen nicht in den Speicher

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `end` | Endadresse (hexadezimal) | 10A00 |

- de: Programm samt Maschinencode endet bei $10A00: zu groß
- en: program with machine code ends at $10A00: too large

## B178

### `diag-basic-b178-dialect-unknown` (Fehler): Unbekannter BASIC-Dialekt (Projekteinstellung, Parameter oder `#dialect`)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `id` | angegebene Kennung | basic99 |
| `known` | bekannte Kennungen, durch Komma getrennt | atari_basic, basic35, basic65 |

- de: unbekannter BASIC-Dialekt 'basic99' (bekannt: atari_basic, basic35, basic65)
- en: unknown BASIC dialect 'basic99' (known: atari_basic, basic35, basic65)

## B180

### `diag-basic-b180-assign-expected` (Fehler): Zuweisung ohne '=' (Atari)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `id` | Name der Variablen | A |

- de: '=' nach 'A' erwartet
- en: '=' expected after 'A'

### `diag-basic-b180-equals-expected` (Fehler): '=' fehlt (Atari)

- de: '=' erwartet
- en: '=' expected

### `diag-basic-b180-flag-statement-missing` (Fehler): Flag-Anweisung fehlt in der Dialektdatei (Turbo BASIC XL)

- de: Flag-Anweisung fehlt
- en: flag statement is missing

### `diag-basic-b180-for-var-expected` (Fehler): FOR ohne Laufvariable (Atari)

- de: Laufvariable erwartet
- en: loop variable expected

### `diag-basic-b180-goto-gosub-expected` (Fehler): ON ohne GOTO oder GOSUB (Atari)

- de: GOTO oder GOSUB erwartet
- en: GOTO or GOSUB expected

### `diag-basic-b180-proc-params` (Fehler): Parameter bei PROC/EXEC (Turbo BASIC XL)

- de: Parameter bei PROC/EXEC sind in Turbo BASIC XL nicht vorgesehen (Variablen sind global)
- en: PROC/EXEC parameters are not supported in Turbo BASIC XL (variables are global)

### `diag-basic-b180-statement-expected` (Fehler): Anweisung erwartet (Turbo BASIC XL)

- de: Anweisung erwartet
- en: statement expected

### `diag-basic-b180-string-compare-expected` (Fehler): Nach dem Vergleichsoperator fehlt eine Zeichenkette (Atari)

- de: Zeichenkette nach dem Vergleichsoperator erwartet
- en: string expected after the comparison operator

### `diag-basic-b180-then-expected` (Fehler): THEN fehlt (Atari)

- de: THEN erwartet
- en: THEN expected

### `diag-basic-b180-then-target-expected` (Fehler): Nach THEN fehlt Anweisung oder Zeilennummer (Atari)

- de: Anweisung oder Zeilennummer nach THEN erwartet
- en: statement or line number expected after THEN

### `diag-basic-b180-to-expected` (Fehler): TO fehlt (Atari)

- de: TO erwartet
- en: TO expected

## B181

### `diag-basic-b181-expr-incomplete` (Fehler): Ausdruck unvollständig (Atari)

- de: Ausdruck unvollständig
- en: expression is incomplete

### `diag-basic-b181-lparen-expected` (Fehler): '(' fehlt (DIM, Atari)

- de: '(' erwartet
- en: '(' expected

### `diag-basic-b181-rparen-missing` (Fehler): Schließende Klammer fehlt

- de: ')' fehlt
- en: ')' is missing

## B182

### `diag-basic-b182-label-expected` (Fehler): Label erwartet (Turbo BASIC XL)

- de: Label erwartet
- en: label expected

### `diag-basic-b182-label-name` (Fehler): Ungültiger Labelname (Turbo BASIC XL)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `id` | der Name | A$ |

- de: 'A$' ist kein gültiger Labelname
- en: 'A$' is not a valid label name

### `diag-basic-b182-name-expected` (Fehler): Name erwartet (DIM, Atari)

- de: Name erwartet
- en: name expected

### `diag-basic-b182-reserved-word` (Fehler): Reserviertes Wort an einer Stelle, an der ein Name stehen muss (Atari)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `id` | das Wort | TO |

- de: reserviertes Wort 'TO' an dieser Stelle
- en: reserved word 'TO' at this position

### `diag-basic-b182-variable-expected` (Fehler): Variable erwartet (Atari)

- de: Variable erwartet
- en: variable expected

## B183

### `diag-basic-b183-bad-exponent` (Fehler): Exponent einer Zahl ungültig (Atari)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `lit` | das Zahlenliteral | 1E+X |

- de: ungültiger Exponent in '1E+X'
- en: invalid exponent in '1E+X'

### `diag-basic-b183-bad-number` (Fehler): Zahlenliteral ungültig (Atari)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `lit` | das Zahlenliteral | 1.2.3 |

- de: ungültige Zahl '1.2.3'
- en: invalid number '1.2.3'

### `diag-basic-b183-number-range` (Fehler): Zahl außerhalb des Bereichs der BCD-Darstellung

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `lit` | das Zahlenliteral | 1E-99 |

- de: Zahl '1E-99' außerhalb des Bereichs
- en: number '1E-99' is out of range

### `diag-basic-b183-number-range-max` (Fehler): Zahl außerhalb des Bereichs der BCD-Darstellung (mit Höchstwert)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `lit` | das Zahlenliteral | 1E99 |

- de: Zahl '1E99' außerhalb des Bereichs (max. 9,99…E97)
- en: number '1E99' is out of range (max. 9.99...E97)

## B184

### `diag-basic-b184-atascii-range` (Fehler): Zeichen außerhalb des ATASCII-Bereichs in einer Zeichenkette

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `c` | das Zeichen | é |

- de: Zeichen 'é' nicht im ATASCII-Bereich (nutze { "{" }{ "$" }xx{ "}" })
- en: character 'é' is not in the ATASCII range (use { "{" }{ "$" }xx{ "}" })

### `diag-basic-b184-string-too-long` (Fehler): Zeichenkette länger als 255 Zeichen (Atari)

- de: Zeichenkette länger als 255 Zeichen
- en: string is longer than 255 characters

## B185

### `diag-basic-b185-dialect-missing` (Fehler): Eintrag fehlt in der Dialektdatei (Atari)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name des Eintrags | RND0 |

- de: RND0 fehlt
- en: RND0 is missing

### `diag-basic-b185-empty-line` (Fehler): Leere Zeile (Atari)

- de: leere Zeile
- en: empty line

### `diag-basic-b185-line-number-max-atari` (Fehler): Zeilennummer größer als 32767 (Atari)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `num` | die Zeilennummer | 40000 |

- de: Zeilennummer 40000 größer als 32767
- en: line number 40000 is greater than 32767

### `diag-basic-b185-line-too-long-atari` (Fehler): Zeile länger als 255 Bytes (Atari)

- de: Zeile zu lang (mehr als 255 Bytes)
- en: line is too long (more than 255 bytes)

### `diag-basic-b185-too-many-vars` (Fehler): Zu viele Variablen im Programm (Atari)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `n` | erlaubte Höchstzahl | 128 |

- de: mehr als 128 Variablen
- en: more than 128 variables

## B190

### `diag-basic-b190-numbering-syntax` (Fehler): Die Zeile #numbering ist ungültig

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `text` | die Zeile | #numbering step=x |

- de: ungültige Zeile '#numbering step=x' (erwartet: #numbering start=100 step=5)
- en: invalid line '#numbering step=x' (expected: #numbering start=100 step=5)

## B191

### `diag-basic-b191-numbering-range` (Fehler): Start oder Schrittweite der Zeilennummern außerhalb des erlaubten Bereichs

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `start` | Start | 100 |
| `step` | Schrittweite | 0 |
| `max` | größte Zeilennummer des Dialekts | 63999 |

- de: Zeilennummern: Start 100 und Schrittweite 0 ungültig (Schrittweite mindestens 1, Start höchstens 63999)
- en: line numbers: start 100 and step 0 are invalid (step at least 1, start at most 63999)

## B192

### `diag-basic-b192-numbering-overflow` (Fehler): Die Zeilennummern reichen bei dieser Schrittweite nicht für alle Zeilen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `count` | Zahl der Zeilen | 900 |
| `last` | letzte Nummer bei dieser Schrittweite | 90000 |
| `max` | größte Zeilennummer des Dialekts | 63999 |
| `suggestion` | Vorschlag für die Schrittweite | 7 |

- de: 900 Zeilen brauchen bei dieser Schrittweite Nummern bis 90000 (größte erlaubte: 63999); kleinere Schrittweite, z. B. 7
- en: 900 lines need numbers up to 90000 with this step (largest allowed: 63999); use a smaller step, e.g. 7

## B193

### `diag-basic-b193-media-failed` (Fehler): Medium beim Einbinden nicht gefunden oder nicht als BASIC-DATA umwandelbar

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Pfad im #include | graphics/hero.vcsprite |
| `reason` | Grund des Auflösers | Medium nicht gefunden |

- de: #include "graphics/hero.vcsprite": Medium nicht gefunden
- en: #include "graphics/hero.vcsprite": Medium nicht gefunden

## B194

### `diag-basic-b194-legacy-path` (Warnung): Altpfad build/export im #include

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Pfad im #include | build/export/hero.data.bas |
| `source` | Medium, das den Text erzeugt | graphics/hero.vcsprite |

- de: #include "build/export/hero.data.bas": veralteter Pfad; besser direkt das Medium einbinden: #include "graphics/hero.vcsprite"
- en: #include "build/export/hero.data.bas": outdated path; better include the medium directly: #include "graphics/hero.vcsprite"

## B195

### `diag-basic-b195-program-too-big` (Warnung): Programm größer als der freie BASIC-Speicher

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `size` | Größe des übersetzten Programms in Byte | 41200 |
| `free` | freier BASIC-Speicher in Byte | 38911 |
| `system` | Zielsystem | c64 |

- de: Das Programm belegt 41200 Byte, frei sind 38911 Byte (c64); es passt nicht in den Speicher. Maschinencode mit #asm oder weniger DATA-Zeilen verwenden
- en: The program takes 41200 bytes, 38911 bytes are free (c64); it does not fit in memory. Use machine code with #asm or fewer DATA lines

## B196

### `diag-basic-b196-media-restore-order` (Warnung): RESTORE @marke bei mehreren Medien in einem BASIC ohne RESTORE-Ziel

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `count` | Zahl der eingebundenen Medien | 2 |
| `dialect` | Dialekt | cbm_v2 |

- de: 2 Medien eingebunden, aber cbm_v2 kennt kein RESTORE auf eine Zeile: READ liest streng der Reihe nach; die Medien in Lesereihenfolge einbinden
- en: 2 media included, but cbm_v2 has no RESTORE to a line: READ reads strictly in order; include the media in reading order

## E100

### `diag-basic-e100-no-line-number` (Fehler): Zeile beginnt nicht mit einer Zeilennummer

- de: Zeile beginnt nicht mit einer Zeilennummer
- en: line does not start with a line number

## E101

### `diag-basic-e101-unexpected-byte` (Fehler): Unerwartetes Zeichen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `b` | Byte (Hex) | 7C |

- de: unerwartetes Zeichen $7C
- en: unexpected character $7C

### `diag-basic-e101-unknown-token` (Fehler): Unbekanntes Token

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `b` | Byte (Hex) | FF |

- de: unbekanntes Token $FF
- en: unknown token $FF

## E102

### `diag-basic-e102-line-twice` (Fehler): Zeilennummer doppelt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `num` | die Zeilennummer | 10 |

- de: Zeilennummer 10 doppelt
- en: line number 10 appears twice

## E103

### `diag-basic-e103-array-name-expected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: Feldname erwartet
- en: array name expected

### `diag-basic-e103-equals-expected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: '=' erwartet
- en: '=' expected

### `diag-basic-e103-expr-expected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: Ausdruck erwartet
- en: expression expected

### `diag-basic-e103-expr-expected-found` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `found` | das gefundene Element | Sym(')') |

- de: Ausdruck erwartet, gefunden Sym(')')
- en: expression expected, found Sym(')')

### `diag-basic-e103-fn-after-def` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: FN nach DEF erwartet
- en: FN expected after DEF

### `diag-basic-e103-fn-name-after-fn` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: Funktionsname nach FN erwartet
- en: function name expected after FN

### `diag-basic-e103-fn-name-expected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: Name nach FN erwartet
- en: name expected after FN

### `diag-basic-e103-fn-one-arg` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: FN erwartet genau ein Argument
- en: FN expects exactly one argument

### `diag-basic-e103-goto-gosub-after-on` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: GOTO oder GOSUB nach ON erwartet
- en: GOTO or GOSUB expected after ON

### `diag-basic-e103-kw-not-allowed` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `k` | das Schlüsselwort | GOTO |

- de: GOTO ist hier nicht erlaubt
- en: GOTO is not allowed here

### `diag-basic-e103-line` (Fehler): Syntaxfehler in einer Zeile

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `num` | Zeilennummer | 10 |
| `e` | innere Meldung | (Meldung) |

- de: Zeile 10: (Meldung)
- en: line 10: (Meldung)

### `diag-basic-e103-line-no-expected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: Zeilennummer erwartet
- en: line number expected

### `diag-basic-e103-line-no-invalid` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `s` | der Text | 70000 |

- de: Zeilennummer 70000 ungültig
- en: line number 70000 is invalid

### `diag-basic-e103-loop-var-expected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: Schleifenvariable erwartet
- en: loop variable expected

### `diag-basic-e103-move-three` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: -MOVE erwartet 3 Zahlen
- en: -MOVE expects 3 numbers

### `diag-basic-e103-name-after-expected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `k` | das Schlüsselwort | NEXT |

- de: Name nach NEXT erwartet
- en: name expected after NEXT

### `diag-basic-e103-not-a-statement` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `k` | das Schlüsselwort | TO |

- de: TO ist hier keine Anweisung
- en: TO is not a statement here

### `diag-basic-e103-not-sign` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: NOT mit einem Vorzeichen davor (NOT -1) ist in Atari-BASIC Fehler 11; Klammern setzen: NOT (-1)
- en: NOT followed by a sign (NOT -1) is error 11 in Atari BASIC; use parentheses: NOT (-1)

### `diag-basic-e103-numbers-expected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `k` | das Schlüsselwort | POSITION |
| `want` | erwartete Zahl | 2 |

- de: POSITION erwartet 2 Zahlen
- en: POSITION expects 2 numbers

### `diag-basic-e103-param-name-expected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: Parametername erwartet
- en: parameter name expected

### `diag-basic-e103-proc-name-on-exec` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: Prozedurname nach ON … EXEC erwartet
- en: procedure name expected after ON ... EXEC

### `diag-basic-e103-rand-one-arg` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: RAND erwartet genau ein Argument
- en: RAND expects exactly one argument

### `diag-basic-e103-semicolon-channel` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: ';' nach der Kanalnummer erwartet
- en: ';' expected after the channel number

### `diag-basic-e103-semicolon-prompt` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: ';' nach dem Eingabetext erwartet
- en: ';' expected after the input prompt

### `diag-basic-e103-stmt-expected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: Anweisung erwartet
- en: statement expected

### `diag-basic-e103-stmt-expected-found` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `found` | das gefundene Element | Num("5") |

- de: Anweisung erwartet, gefunden Num("5")
- en: statement expected, found Num("5")

### `diag-basic-e103-sym-expected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `c` | das Zeichen | ( |

- de: '(' erwartet
- en: '(' expected

### `diag-basic-e103-then-or-goto` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: THEN oder GOTO nach IF erwartet
- en: THEN or GOTO expected after IF

### `diag-basic-e103-to-after-go` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: TO nach GO erwartet
- en: TO expected after GO

### `diag-basic-e103-to-expected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: TO erwartet
- en: TO expected

### `diag-basic-e103-unexpected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `found` | das gefundene Element | Sym(')') |

- de: unerwartet: Sym(')')
- en: unexpected: Sym(')')

### `diag-basic-e103-variable-expected` (Fehler): Syntaxfehler im BASIC-Text (basic2asm)

- de: Variable erwartet
- en: variable expected

## E190

### `diag-basic-e190-line` (Fehler): Rundreise: Fehler in einer Zeile

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `n` | Zeilennummer | 10 |
| `e` | innere Meldung | (Meldung) |

- de: Zeile 10: (Meldung)
- en: line 10: (Meldung)

### `diag-basic-e190-unsupported` (Fehler): Anweisung lässt sich nicht zurückschreiben

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `k` | die Anweisung | LOAD |

- de: LOAD lässt sich nicht zurückschreiben (nicht übersetzte Anweisung)
- en: LOAD cannot be written back (statement is not translated)

## E191

### `diag-basic-e191-roundtrip` (Fehler): Rundreise ergibt anderen Syntaxbaum

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `at` | erste abweichende Zeilennummer | 20 |

- de: Rundreise ergibt einen anderen Syntaxbaum (erste Abweichung in Zeile 20)
- en: round trip gives a different syntax tree (first difference in line 20)

## E192

### `diag-basic-e192-embed-data-dir` (Fehler): embed ohne Ordner data

- de: embed braucht den Ordner data (B2aOptions::data_dir)
- en: embed needs the data folder (B2aOptions::data_dir)

### `diag-basic-e192-embed-loop` (Fehler): embed: Include-Schleife

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Datei | rt_core.asm |

- de: embed: Include-Schleife bei rt_core.asm
- en: embed: include loop at rt_core.asm

### `diag-basic-e192-embed-read` (Fehler): embed: Laufzeitdatei nicht lesbar

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `path` | Pfad | data/sys/c64/b2a/rt_core.asm |
| `error` | Meldung des Systems | not found |

- de: embed: data/sys/c64/b2a/rt_core.asm: not found
- en: embed: data/sys/c64/b2a/rt_core.asm: not found

## E201

### `diag-asm-e201-cpu` (Fehler): Befehl gibt es auf dieser CPU nicht

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `mn` | Befehl (Mnemonic), bei Q-Befehlen mit Q | SLO |
| `cpu` | Kennung der CPU | 6510 |

- de: SLO existiert auf CPU 6510 nicht
- en: SLO does not exist on CPU 6510

### `diag-asm-e201-illegal-denied` (Fehler): undokumentierter Opcode bei illegals=deny

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `mn` | Befehl (Mnemonic) | SLO |

- de: undokumentierter Opcode SLO (illegals=deny)
- en: undocumented opcode SLO (illegals=deny)

### `diag-basic-e201-jump-target` (Fehler): Sprungziel existiert nicht

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `n` | die Zeilennummer | 100 |

- de: Sprungziel: Zeile 100 existiert nicht
- en: jump target: line 100 does not exist

## E202

### `diag-asm-e202-bad-char-literal` (Fehler): ungültiges Zeichenliteral (Teil von E202)

- de: ungültiges Zeichenliteral
- en: invalid character literal

### `diag-asm-e202-bad-float` (Fehler): ungültige Kommazahl (Teil von E202)

- de: ungültige Kommazahl
- en: invalid floating-point number

### `diag-asm-e202-bad-hex` (Fehler): ungültige Hex-Zahl (Teil von E202)

- de: ungültige Hex-Zahl
- en: invalid hex number

### `diag-asm-e202-bracket-missing` (Fehler): eckige Klammer zu fehlt (Teil von E202)

- de: ']' fehlt
- en: ']' missing

### `diag-asm-e202-call-paren-missing` (Fehler): Klammer zu fehlt bei Funktionsaufruf (Teil von E202)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Funktionsname | min |

- de: ')' fehlt bei min(…)
- en: ')' missing after min(...)

### `diag-asm-e202-div-zero` (Fehler): Division durch 0 (Teil von E202)

- de: Division durch 0
- en: division by 0

### `diag-asm-e202-empty` (Fehler): Ausdruck fehlt (Teil von E202)

- de: Ausdruck fehlt
- en: expression missing

### `diag-asm-e202-expression` (Fehler): Fehler in einem Ausdruck (mit dem Ausdruck)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `inner` | Fehlermeldung des Ausdrucks | (Meldung) |
| `expr` | Ausdruck | 1 + |

- de: (Meldung) in '1 +'
- en: (Meldung) in '1 +'

### `diag-asm-e202-fn-arity` (Fehler): falsche Anzahl Argumente (Teil von E202)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Funktionsname | atan2 |
| `expected` | erwartete Anzahl der Argumente | 2 |
| `got` | Anzahl der übergebenen Argumente | 1 |

- de: atan2() erwartet 2 Argument(e), erhalten 1
- en: atan2() expects 2 argument(s), got 1

### `diag-asm-e202-fn-arity-min` (Fehler): zu wenige Argumente (Teil von E202)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Funktionsname | min |
| `expected` | Mindestzahl der Argumente | 1 |
| `got` | Anzahl der übergebenen Argumente | 1 |

- de: min() erwartet mindestens 1 Argument(e), erhalten 1
- en: min() expects at least 1 argument(s), got 1

### `diag-asm-e202-fn-needs-numbers` (Fehler): Funktion mit Liste statt Zahlen (Teil von E202)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Funktionsname | sin |

- de: sin() erwartet Zahlen, keine Liste
- en: sin() expects numbers, not a list

### `diag-asm-e202-fn-recursion` (Fehler): Funktion ruft sich selbst auf (Teil von E202)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Funktionsname | f |

- de: Funktion 'f' ruft sich selbst auf
- en: function 'f' calls itself

### `diag-asm-e202-fn-too-deep` (Fehler): Funktionen zu tief verschachtelt (Teil von E202)

- de: Funktionen zu tief verschachtelt
- en: functions nested too deeply

### `diag-asm-e202-for-in-not-list` (Fehler): .for … in ohne Liste

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `expr` | Ausdruck | 5 |

- de: .for … in erwartet eine Liste oder range(…), nicht '5'
- en: .for ... in expects a list or range(...), not '5'

### `diag-asm-e202-in-function` (Fehler): Fehler innerhalb einer Funktion (Teil von E202)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `inner` | eingebettete Meldung | (Meldung) |
| `name` | Funktionsname | f |

- de: (Meldung) (in Funktion f)
- en: (Meldung) (in function f)

### `diag-asm-e202-incomplete` (Fehler): Ausdruck unvollständig (Teil von E202)

- de: Ausdruck unvollständig
- en: incomplete expression

### `diag-asm-e202-index-missing` (Fehler): Index fehlt (Teil von E202)

- de: Index fehlt
- en: index missing

### `diag-asm-e202-index-not-list` (Fehler): Index auf Nicht-Liste (Teil von E202)

- de: Index nur auf Listen erlaubt
- en: indexing is only allowed on lists

### `diag-asm-e202-index-out-of-range` (Fehler): Index außerhalb der Liste (Teil von E202)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `index` | Index | 5 |
| `len` | Länge der Liste | 3 |

- de: Index 5 außerhalb der Liste (Länge 3)
- en: index 5 outside the list (length 3)

### `diag-asm-e202-integer-only` (Fehler): Operator nur für ganze Zahlen (Teil von E202)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `op` | Operator | % |

- de: '%' ist nur für ganze Zahlen erlaubt
- en: '%' is only allowed for integers

### `diag-asm-e202-len-needs-list` (Fehler): len() ohne Liste (Teil von E202)

- de: len() erwartet eine Liste
- en: len() expects a list

### `diag-asm-e202-list-for-number` (Fehler): Liste, wo eine Zahl erwartet wird

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `expr` | Ausdruck | tabelle |

- de: Liste statt Zahl in 'tabelle' (Elemente mit [i] wählen oder .byte/.word für die ganze Liste)
- en: list instead of number in 'tabelle' (pick elements with [i] or use .byte/.word for the whole list)

### `diag-asm-e202-list-not-allowed` (Fehler): Operator für Liste nicht erlaubt (Teil von E202)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `op` | Operator | + |

- de: '+' ist für Listen nicht erlaubt
- en: '+' is not allowed for lists

### `diag-asm-e202-list-op` (Fehler): Operator für Listen nicht erlaubt (Teil von E202)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `op` | Operator | - |

- de: '-' ist für Listen nicht erlaubt (nur Liste + Liste und Liste * Zahl)
- en: '-' is not allowed for lists (only list + list and list * number)

### `diag-asm-e202-list-too-long` (Fehler): Liste zu lang (Teil von E202)

- de: Liste zu lang (höchstens 65536 Elemente)
- en: list too long (at most 65536 elements)

### `diag-asm-e202-next-zero` (Fehler): next(n) mit n kleiner 1 (Teil von E202)

- de: next(n): n muss größer als 0 sein
- en: next(n): n must be greater than 0

### `diag-asm-e202-not-finite` (Fehler): Ergebnis ist keine endliche Zahl (Teil von E202)

- de: Ergebnis ist keine endliche Zahl
- en: result is not a finite number

### `diag-asm-e202-number-too-large` (Fehler): Zahl zu groß (Teil von E202)

- de: Zahl zu groß
- en: number too large

### `diag-asm-e202-paren-missing` (Fehler): runde Klammer zu fehlt (Teil von E202)

- de: ')' fehlt
- en: ')' missing

### `diag-asm-e202-range-step-zero` (Fehler): range mit Schrittweite 0 (Teil von E202)

- de: range: Schrittweite darf nicht 0 sein
- en: range: step must not be 0

### `diag-asm-e202-range-too-long` (Fehler): range zu lang (Teil von E202)

- de: range: mehr als 65536 Elemente
- en: range: more than 65536 elements

### `diag-asm-e202-trailing` (Fehler): Zeichen nach dem Ausdruck (Teil von E202)

- de: unerwartete Zeichen am Ausdrucksende
- en: unexpected characters at end of expression

### `diag-asm-e202-unexpected-char` (Fehler): unerwartetes Zeichen (Teil von E202)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `ch` | Zeichen | # |

- de: unerwartetes Zeichen '#'
- en: unexpected character '#'

### `diag-asm-e202-unexpected-operator` (Fehler): unerwarteter Operator (Teil von E202)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `op` | Operator | ) |

- de: unerwarteter Operator ')'
- en: unexpected operator ')'

### `diag-asm-e202-unknown-symbol` (Fehler): unbekanntes Symbol (Teil von E202)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Symbolname | schleife |

- de: unbekanntes Symbol 'schleife'
- en: unknown symbol 'schleife'

## E203

### `diag-asm-e203-address-range` (Fehler): Adresse außerhalb 0..65535

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `value` | Adresse | 70000 |

- de: Adresse 70000 außerhalb 0..65535
- en: address 70000 outside 0..65535

### `diag-asm-e203-bank-range` (Fehler): .bank außerhalb 0..1023

- de: .bank außerhalb 0..1023
- en: .bank outside 0..1023

### `diag-asm-e203-basicstub-line` (Fehler): .basicstub: Zeilennummer außerhalb

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `line` | Zeilennummer | 70000 |

- de: .basicstub: Zeilennummer 70000 außerhalb 0–63999
- en: .basicstub: line number 70000 outside 0–63999

### `diag-asm-e203-basicstub-too-long` (Fehler): .basicstub: BASIC-Zeile zu lang

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `len` | Länge der Zeile in Bytes | 230 |

- de: .basicstub: BASIC-Zeile zu lang (230 Bytes, höchstens 200)
- en: .basicstub: BASIC line too long (230 bytes, at most 200)

### `diag-asm-e203-branch-range` (Fehler): Sprung zu weit

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `off` | Sprungweite in Bytes | 200 |

- de: Sprung zu weit (200 Bytes)
- en: branch too far (200 bytes)

### `diag-asm-e203-fill-length` (Fehler): .fill: ungültige Länge

- de: .fill: ungültige Länge
- en: .fill: invalid length

### `diag-asm-e203-incbin-range` (Fehler): .incbin: Bereich außerhalb der Datei

- de: .incbin: Bereich außerhalb der Datei
- en: .incbin: range outside the file

### `diag-asm-e203-lohifill-length` (Fehler): .lohitable: ungültige Länge

- de: .lohitable: ungültige Länge
- en: .lohitable: invalid length

### `diag-asm-e203-org-range` (Fehler): .org außerhalb 0..65535

- de: .org außerhalb 0..65535
- en: .org outside 0..65535

### `diag-asm-e203-page-address` (Fehler): .align: Adresse außerhalb

- de: .align: Adresse außerhalb 0..65535
- en: .align: address outside 0..65535

### `diag-asm-e203-page-size` (Fehler): .align: Größe kleiner 1

- de: .align: Größe muss größer als 0 sein
- en: .align: size must be greater than 0

### `diag-asm-e203-repeat-count` (Fehler): .repeat: ungültige Anzahl

- de: .repeat: ungültige Anzahl
- en: .repeat: invalid count

### `diag-asm-e203-segment-order` (Fehler): .segment: Anfang hinter dem Ende

- de: .segment: Anfang liegt hinter dem Ende
- en: .segment: start is after the end

### `diag-asm-e203-struct-count` (Fehler): .struct: ungültige Anzahl

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `field` | Feldname | x |

- de: .struct: ungültige Anzahl bei 'x'
- en: .struct: invalid count for 'x'

### `diag-asm-e203-text` (Fehler): Fehler in einer Textdirektive (mit Direktive)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `dir` | Direktive | .ptext |
| `reason` | eingebettete Meldung | (Meldung) |

- de: .ptext: (Meldung)
- en: .ptext: (Meldung)

### `diag-asm-e203-text-empty` (Fehler): leerer Text (Teil von E203)

- de: leerer Text
- en: empty text

### `diag-asm-e203-text-too-long` (Fehler): Text länger als 255 Zeichen (Teil von E203)

- de: Text länger als 255 Zeichen
- en: text longer than 255 characters

### `diag-asm-e203-value-16bit` (Fehler): Wert außerhalb 16 Bit

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `value` | Wert | 70000 |

- de: Wert 70000 außerhalb 16 Bit
- en: value 70000 outside 16 bits

### `diag-asm-e203-value-32bit` (Fehler): Wert außerhalb 32 Bit

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `value` | Wert | 5000000000 |

- de: Wert 5000000000 außerhalb 32 Bit
- en: value 5000000000 outside 32 bits

### `diag-asm-e203-value-range` (Fehler): Wert außerhalb des Bereichs

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `value` | Wert | 300 |
| `lo` | untere Grenze | -128 |
| `hi` | obere Grenze | 255 |

- de: Wert 300 außerhalb -128..255
- en: value 300 outside -128..255

## E204

### `diag-asm-e204-builtin-function` (Fehler): Name einer eingebauten Funktion

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Funktionsname | sin |

- de: 'sin' ist eine eingebaute Funktion
- en: 'sin' is a built-in function

### `diag-asm-e204-function` (Fehler): Funktion mehrfach definiert

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Funktionsname | quadrat |

- de: Funktion 'quadrat' doppelt definiert
- en: function 'quadrat' defined twice

### `diag-asm-e204-struct` (Fehler): .struct/.enum mehrfach definiert

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Struktur | punkt |

- de: 'punkt' doppelt definiert
- en: 'punkt' defined twice

### `diag-asm-e204-symbol` (Fehler): Symbol mehrfach definiert

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name des Symbols | schleife |

- de: Symbol 'schleife' doppelt definiert
- en: symbol 'schleife' defined twice

## E205

### `diag-asm-e205` (Fehler): unbekannter Befehl

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `mn` | Befehl (Mnemonic) | LDA |

- de: unbekannter Befehl 'LDA'
- en: unknown instruction 'LDA'

## E206

### `diag-asm-e206-indirect` (Fehler): ungültiger indirekter Operand

- de: ungültiger indirekter Operand
- en: invalid indirect operand

### `diag-asm-e206-mode-unsupported` (Fehler): Adressierungsart nicht unterstützt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `mn` | Befehl (Mnemonic) | LDA |
| `mode` | Adressierungsart | Abs |

- de: LDA unterstützt die Adressierungsart Abs nicht
- en: LDA does not support addressing mode Abs

### `diag-asm-e206-operand` (Fehler): ungültiger Operand

- de: ungültiger Operand
- en: invalid operand

### `diag-asm-e206-operand-needed` (Fehler): Befehl braucht einen Operanden

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `mn` | Befehl (Mnemonic) | LDA |

- de: LDA braucht einen Operanden
- en: LDA needs an operand

### `diag-asm-e206-ptr32` (Fehler): ungültiger 32-Bit-Zeiger

- de: ungültiger 32-Bit-Zeiger
- en: invalid 32-bit pointer

### `diag-asm-e206-ptr32-cpu` (Fehler): 32-Bit-Zeiger auf dieser CPU nicht möglich

- de: 32-Bit-Zeiger [..] gibt es auf dieser CPU nicht
- en: 32-bit pointers [..] do not exist on this CPU

### `diag-asm-e206-quad-mode` (Fehler): Adressierungsart beim Q-Befehl nicht unterstützt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `mn` | Befehl (Mnemonic) mit Q | LDAQ |
| `mode` | Adressierungsart | Abs |

- de: LDAQ unterstützt die Adressierungsart Abs nicht
- en: LDAQ does not support addressing mode Abs

### `diag-asm-e206-sp-needs-y` (Fehler): ($nn,sp) ohne ,y

- de: ({ "$" }nn,sp) braucht ,y dahinter
- en: ({ "$" }nn,sp) needs ,y after it

## E207

### `diag-asm-e207-assert` (Fehler): Assertion fehlgeschlagen (ohne Text)

- de: Assertion fehlgeschlagen
- en: assertion failed

### `diag-asm-e207-assert-message` (Fehler): Assertion fehlgeschlagen (mit Text)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `message` | Text der Assertion | x muss kleiner 10 sein |

- de: Assertion fehlgeschlagen: x muss kleiner 10 sein
- en: assertion failed: x muss kleiner 10 sein

### `diag-asm-e207-user` (Fehler): Fehler aus .error (Text vom Programmierer)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `text` | Text hinter .error | Speicher voll |

- de: Speicher voll
- en: Speicher voll

## E208

### `diag-asm-e208-assignment-expected` (Fehler): Zuweisung erwartet

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `text` | Text | i < 8 |

- de: Zuweisung erwartet statt 'i < 8'
- en: assignment expected instead of 'i < 8'

### `diag-asm-e208-branchlong` (Fehler): .branchlong mit falschem Wert

- de: .branchlong erwartet auto, jmp oder off
- en: .branchlong expects auto, jmp or off

### `diag-asm-e208-cartridge-type` (Fehler): .cartridge ohne Typ

- de: .cartridge erwartet den Typ in Anführungszeichen (z. B. "ocean")
- en: .cartridge expects the type in quotation marks (e.g. "ocean")

### `diag-asm-e208-const-syntax` (Fehler): .const ohne NAME = Ausdruck

- de: .const erwartet NAME = Ausdruck
- en: .const expects NAME = expression

### `diag-asm-e208-encoding-name` (Fehler): .encoding ohne Namen

- de: .encoding erwartet einen Namen in Anführungszeichen
- en: .encoding expects a name in quotation marks

### `diag-asm-e208-encoding-size` (Fehler): Kodiertabelle hat nicht 256 Byte

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Dateiname der Tabelle | tabelle.bin |

- de: .encoding: 'tabelle.bin' muss genau 256 Byte enthalten
- en: .encoding: 'tabelle.bin' must contain exactly 256 bytes

### `diag-asm-e208-encoding-unknown` (Fehler): unbekannte Kodierung

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Kodierung oder Datei | latin1 |

- de: .encoding: unbekannte Kodierung oder Datei 'latin1'
- en: .encoding: unknown encoding or file 'latin1'

### `diag-asm-e208-enum-item-name` (Fehler): .enum: ungültiger Name

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name des Eintrags | 9x |

- de: .enum: ungültiger Name '9x'
- en: .enum: invalid name '9x'

### `diag-asm-e208-for-in-list` (Fehler): .for … in ohne Liste

- de: .for … in erwartet eine Liste oder range(…)
- en: .for ... in expects a list or range(...)

### `diag-asm-e208-for-syntax` (Fehler): .for falsch aufgebaut

- de: .for erwartet: .for anfang, bedingung, schritt (z. B. .for i = 0, i < 8, i += 1)
- en: .for expects: .for start, condition, step (e.g. .for i = 0, i < 8, i += 1)

### `diag-asm-e208-function-name` (Fehler): .function: ungültiger Name

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Funktionsname | 9f |

- de: .function: ungültiger Name '9f'
- en: .function: invalid name '9f'

### `diag-asm-e208-function-syntax` (Fehler): .function falsch aufgebaut

- de: .function erwartet: .function name(param, …) = ausdruck
- en: .function expects: .function name(param, ...) = expression

### `diag-asm-e208-hex-even` (Fehler): .hex mit ungerader Ziffernzahl

- de: .hex: gerade Anzahl Hexziffern erwartet
- en: .hex: even number of hex digits expected

### `diag-asm-e208-hex-quoted` (Fehler): .hex ohne Zeichenkette

- de: .hex erwartet Hexziffern in Anführungszeichen
- en: .hex expects hex digits in quotation marks

### `diag-asm-e208-illegals-value` (Fehler): unbekannter Wert für illegals

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `value` | Wert | viel |

- de: illegals: unbekannter Wert 'viel'
- en: illegals: unknown value 'viel'

### `diag-asm-e208-incbin-name` (Fehler): .incbin ohne Dateinamen

- de: .incbin erwartet einen Dateinamen
- en: .incbin expects a file name

### `diag-asm-e208-include-alias` (Fehler): ungültiger Name hinter as beim Include

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `first` | Direktive | .include |
| `name` | Name hinter as | 9x |

- de: .include ... as: ungültiger Name '9x' (nur Buchstaben, Ziffern und _)
- en: .include ... as: invalid name '9x' (letters, digits and _ only)

### `diag-asm-e208-include-name` (Fehler): Include ohne Dateinamen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `first` | Direktive | .include |

- de: .include erwartet einen Dateinamen in Anführungszeichen oder <spitzen Klammern>
- en: .include expects a file name in quotation marks or <angle brackets>

### `diag-asm-e208-lohifill-syntax` (Fehler): .lohitable ohne Ausdruck

- de: .lohitable erwartet: .lohitable anzahl, ausdruck(i)
- en: .lohitable expects: .lohitable count, expression(i)

### `diag-asm-e208-segment-name` (Fehler): .segment ohne Namen

- de: .segment erwartet einen Namen in Anführungszeichen
- en: .segment expects a name in quotation marks

### `diag-asm-e208-string-expected` (Fehler): Zeichenkette erwartet

- de: Zeichenkette in Anführungszeichen erwartet
- en: string in quotation marks expected

### `diag-asm-e208-strings-expected` (Fehler): Zeichenketten erwartet

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `dir` | Direktive | .text |

- de: .text erwartet Zeichenketten in Anführungszeichen
- en: .text expects strings in quotation marks

### `diag-asm-e208-struct-enum-field` (Fehler): .struct: Feldtyp ist eine Aufzählung

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `ty` | Name der Aufzählung | farbe |

- de: .struct: 'farbe' ist eine Aufzählung; für das Feld .byte (oder .word) verwenden
- en: .struct: 'farbe' is an enumeration; use .byte (or .word) for the field

### `diag-asm-e208-struct-field-name` (Fehler): .struct: ungültiger Feldname

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `field` | Feldname | 9x |

- de: .struct: ungültiger Feldname '9x'
- en: .struct: invalid field name '9x'

### `diag-asm-e208-struct-field-syntax` (Fehler): .struct: Feld ohne Typ

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `text` | Text der Zeile | x |

- de: .struct: Feld erwartet 'name typ [anzahl]', gefunden 'x'
- en: .struct: field expects 'name type [count]', found 'x'

### `diag-asm-e208-struct-field-type` (Fehler): .struct: unbekannter Feldtyp

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `ty` | Feldtyp | float |

- de: .struct: unbekannter Feldtyp 'float' (.byte, .word, .dword oder eine früher definierte Struktur)
- en: .struct: unknown field type 'float' (.byte, .word, .dword or a previously defined structure)

### `diag-asm-e208-struct-name` (Fehler): ungültiger Name bei .struct/.enum

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `dir` | Direktive | .struct |
| `name` | Name | 9x |

- de: .struct: ungültiger Name '9x'
- en: .struct: invalid name '9x'

### `diag-asm-e208-token-hex` (Fehler): ungültiger Hexwert im Sonderzeichen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `token` | Sonderzeichen in Klammern | {$zz} |

- de: {$zz}: ungültiger Hexwert (00 bis ff)
- en: {$zz}: invalid hex value (00 to ff)

### `diag-asm-e208-token-names` (Fehler): Name im Sonderzeichen nicht erlaubt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `token` | Sonderzeichen in Klammern | {down} |

- de: {down}: Namen gibt es nur mit .petscii/.atascii oder .encoding "petscii"/"atascii" (hier nur { "{" }{ "$" }hh{ "}" } und { "{" }nnn{ "}" })
- en: {down}: names are only available with .petscii/.atascii or .encoding "petscii"/"atascii" (only { "{" }{ "$" }hh{ "}" } and { "{" }nnn{ "}" } here)

### `diag-asm-e208-token-number` (Fehler): ungültige Zahl im Sonderzeichen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `token` | Sonderzeichen in Klammern | {300} |

- de: {300}: Zahl 0 bis 255 erwartet
- en: {300}: number 0 to 255 expected

### `diag-asm-e208-token-repeat` (Fehler): Wiederholung im Sonderzeichen ungültig

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `token` | Sonderzeichen in Klammern | {down*0} |

- de: {down*0}: Wiederholung 1 bis 255 erwartet
- en: {down*0}: repeat count 1 to 255 expected

### `diag-asm-e208-token-unclosed` (Fehler): Klammer in Zeichenkette nicht geschlossen

- de: '{ "{" }' ohne schließendes '{ "}" }' (für ein einzelnes '{ "{" }' schreibe '{ "{{" }')
- en: '{ "{" }' without closing '{ "}" }' (to write a single '{ "{" }', type '{ "{{" }')

### `diag-asm-e208-token-unknown` (Fehler): unbekanntes Sonderzeichen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `token` | Sonderzeichen in Klammern | {nix} |

- de: unbekanntes Sonderzeichen {nix}
- en: unknown special character {nix}

### `diag-asm-e208-unknown-directive` (Fehler): unbekannte Direktive

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Direktive | .foo |

- de: unbekannte Direktive '.foo'
- en: unknown directive '.foo'

### `diag-asm-e208-var-name` (Fehler): ungültiger Variablenname

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `dir` | Direktive | .var |
| `name` | Variablenname | 9x |

- de: .var: ungültiger Variablenname '9x'
- en: .var: invalid variable name '9x'

### `diag-asm-e208-var-syntax` (Fehler): .var/.set ohne NAME = Ausdruck

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `dir` | Direktive | .var |

- de: .var erwartet NAME = Ausdruck
- en: .var expects NAME = expression

## E209

### `diag-asm-e209-macro-args` (Fehler): falsche Anzahl Argumente beim Makroaufruf

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Makroname | FARBE |
| `expected` | erwartete Anzahl der Argumente | 2 |
| `got` | Anzahl der übergebenen Argumente | 1 |

- de: Makro 'FARBE' erwartet 2 Argument(e), erhalten 1
- en: macro 'FARBE' expects 2 argument(s), got 1

### `diag-asm-e209-macro-args-range` (Fehler): Anzahl Argumente beim Makroaufruf außerhalb des Bereichs

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Makroname | FARBE |
| `min` | Mindestwert | 1 |
| `max` | größte Anzahl der Argumente | 3 |
| `got` | Anzahl der übergebenen Argumente | 1 |

- de: Makro 'FARBE' erwartet 1 bis 3 Argument(e), erhalten 1
- en: macro 'FARBE' expects 1 to 3 argument(s), got 1

### `diag-asm-e209-macro-name` (Fehler): Makroname wie ein Befehl

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Makroname | LDA |

- de: Makroname 'LDA' kollidiert mit einem Befehl
- en: macro name 'LDA' conflicts with an instruction

## E210

### `diag-asm-e210-block-unclosed` (Fehler): .struct/.enum ohne Ende

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `directive` | Direktive | .struct |
| `end` | erwartetes Ende | .endstruct |

- de: .struct ohne .endstruct
- en: .struct without .endstruct

### `diag-asm-e210-cycles-unclosed` (Fehler): .cycles ohne .endcycles

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name des Blocks | irq |

- de: .cycles 'irq' ohne .endcycles
- en: .cycles 'irq' without .endcycles

### `diag-asm-e210-endcycles` (Fehler): .endcycles ohne .cycles

- de: .endcycles ohne .cycles
- en: .endcycles without .cycles

### `diag-asm-e210-exitmacro` (Fehler): .exitmacro außerhalb eines Makros

- de: .exitmacro außerhalb eines Makros
- en: .exitmacro outside a macro

### `diag-asm-e210-for-unclosed` (Fehler): .for ohne .endfor

- de: .for ohne .endfor
- en: .for without .endfor

### `diag-asm-e210-if-unclosed` (Fehler): .if ohne .endif

- de: .if ohne .endif
- en: .if without .endif

### `diag-asm-e210-macro-name-missing` (Fehler): Makroname fehlt

- de: Makroname fehlt
- en: macro name missing

### `diag-asm-e210-macro-param-default` (Fehler): Makroparameter ohne Standardwert hinter einem mit

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `param` | Parametername | farbe |

- de: Parameter 'farbe' ohne Standardwert steht hinter einem Parameter mit Standardwert
- en: parameter 'farbe' without a default value follows a parameter with a default value

### `diag-asm-e210-macro-unclosed` (Fehler): .macro ohne .endmacro

- de: .macro ohne .endmacro
- en: .macro without .endmacro

### `diag-asm-e210-outside-loop` (Fehler): .break/.continue außerhalb einer Schleife

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `directive` | Direktive | .break |

- de: .break außerhalb einer Schleife
- en: .break outside a loop

### `diag-asm-e210-repeat-unclosed` (Fehler): .repeat ohne .endrepeat

- de: .repeat ohne .endrepeat
- en: .repeat without .endrepeat

### `diag-asm-e210-unexpected` (Fehler): Schlussdirektive ohne Beginn

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `directive` | Direktive | .endif |

- de: unerwartetes .endif
- en: unexpected .endif

## E211

### `diag-asm-e211-include-depth` (Fehler): Include zu tief verschachtelt

- de: Include zu tief verschachtelt (Zyklus?)
- en: include nested too deeply (cycle?)

### `diag-asm-e211-macro-depth` (Fehler): Makro zu tief verschachtelt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Makroname | FARBE |

- de: Makro 'FARBE' zu tief verschachtelt
- en: macro 'FARBE' nested too deeply

## E212

### `diag-asm-e212-file-error` (Fehler): Datei nicht lesbar (Systemfehler)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `path` | Dateipfad | daten/bild.bin |
| `error` | Fehlertext des Betriebssystems | (Systemtext) |

- de: daten/bild.bin: (Systemtext)
- en: daten/bild.bin: (Systemtext)

### `diag-asm-e212-incbin-missing` (Fehler): .incbin-Datei nicht gefunden

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Dateiname | bild.bin |

- de: Datei 'bild.bin' nicht gefunden
- en: file 'bild.bin' not found

### `diag-asm-e212-include-missing` (Fehler): Include-Datei nicht gefunden

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Dateiname | lib.inc |

- de: Include-Datei 'lib.inc' nicht gefunden
- en: include file 'lib.inc' not found

## E213

### `diag-asm-e213-after-open` (Fehler): Text hinter {

- de: nach '{ "{" }' darf nichts in derselben Zeile stehen
- en: nothing may follow '{ "{" }' on the same line

### `diag-asm-e213-namespace-syntax` (Fehler): .namespace falsch aufgebaut

- de: .namespace erwartet: .namespace NAME { "{" }
- en: .namespace expects: .namespace NAME { "{" }

### `diag-asm-e213-scope-name-dot` (Fehler): Scope-Name mit Punkt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name des Scopes | a.b |

- de: Scope-Name 'a.b' darf keinen Punkt enthalten
- en: scope name 'a.b' must not contain a dot

### `diag-asm-e213-scope-unclosed` (Fehler): Scope nicht geschlossen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Pfad des Scopes | init.schleife |

- de: Scope 'init.schleife' wird nicht mit '{ "}" }' geschlossen
- en: scope 'init.schleife' is not closed with '{ "}" }'

### `diag-asm-e213-unmatched-close` (Fehler): } ohne {

- de: '{ "}" }' ohne öffnendes '{ "{" }'
- en: '{ "}" }' without opening '{ "{" }'

## E214

### `diag-asm-e214-after` (Fehler): kein passendes anonymes Label dahinter

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `label` | Label-Schreibweise | -- |

- de: kein passendes '--'-Label dahinter
- en: no matching '--' label after this

### `diag-asm-e214-before` (Fehler): kein passendes anonymes Label davor

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `label` | Label-Schreibweise | -- |

- de: kein passendes '--'-Label davor
- en: no matching '--' label before this

## E215

### `diag-asm-e215-form-absolute` (Fehler): Befehl hat keine absolute Form

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `mn` | Befehl (Mnemonic) | LDA |
| `suffix` | Suffix | a |

- de: LDA.a: keine absolute Form
- en: LDA.a: no absolute form

### `diag-asm-e215-form-branch` (Fehler): Suffix an Sprungbefehl

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `mn` | Befehl (Mnemonic) | BNE |
| `suffix` | Suffix | z |

- de: BNE.z: Sprungbefehle haben nur eine Form
- en: BNE.z: branch instructions have only one form

### `diag-asm-e215-form-zeropage` (Fehler): Befehl hat keine Zeropage-Form

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `mn` | Befehl (Mnemonic) | LDA |
| `suffix` | Suffix | z |

- de: LDA.z: keine Zeropage-Form
- en: LDA.z: no zero-page form

### `diag-asm-e215-suffix` (Fehler): Suffix .z/.a an falscher Operandenform

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `suffix` | Suffix | z |

- de: Suffix .z gilt nur für Zeropage- und absolute Adressierung
- en: suffix .z only applies to zero-page and absolute addressing

## E216

### `diag-asm-e216` (Fehler): Variable nicht mit .var angelegt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Variablenname | i |

- de: Variable 'i' ist nicht mit .var angelegt
- en: variable 'i' was not created with .var

## E217

### `diag-asm-e217` (Fehler): zu viele Schleifendurchläufe

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `limit` | Grenze | 65536 |

- de: .for: mehr als 65536 Durchläufe
- en: .for: more than 65536 iterations

## E218

### `diag-asm-e218` (Fehler): .basicstub nicht am Programmanfang

- de: .basicstub muss am Anfang des Programms stehen (davor wurden schon Bytes ausgegeben)
- en: .basicstub must be at the start of the program (bytes were already output before it)

## E219

### `diag-asm-e219-media` (Fehler): Medium (Sprite, Zeichensatz, Song …) lässt sich nicht einbinden

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `error` | Meldung des Exporters | Medium graphics/hero.vcsprite ist für c64, das Ziel des Baus ist atari8 |

- de: Medium graphics/hero.vcsprite ist für c64, das Ziel des Baus ist atari8
- en: Medium graphics/hero.vcsprite ist für c64, das Ziel des Baus ist atari8

## E300

### `diag-asm-e300-code-before-org` (Fehler): Code vor .org

- de: Code vor .org
- en: code before .org

### `diag-asm-e300-label-before-org` (Fehler): Label vor .org

- de: Label vor .org
- en: label before .org

### `diag-asm-e300-no-convergence` (Fehler): Symbole konvergieren nicht

- de: Symbole konvergieren nicht (zu viele Durchläufe)
- en: symbols do not converge (too many passes)

### `diag-asm-e300-pseudopc-before-org` (Fehler): .relocate vor .org

- de: .relocate vor .org
- en: .relocate before .org

### `diag-basic-e300-stage` (Fehler): Stufe nicht umgesetzt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `stage` | die Stufe | 4 |

- de: Stufe 4 ist nicht umgesetzt; verfügbar sind Stufe 1 (Ganzzahl), 2 (Fließkomma) und 3 (Zeichenketten)
- en: stage 4 is not implemented; stages 1 (integer), 2 (floating point) and 3 (strings) are available

## E301

### `diag-asm-e301-beyond-ffff` (Fehler): Segment überschreitet $FFFF

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `start` | Anfang des Segments (hex ohne $) | FFF0 |

- de: Segment bei $FFF0 überschreitet { "$" }FFFF
- en: segment at $FFF0 exceeds { "$" }FFFF

### `diag-asm-e301-overlap` (Fehler): Segmente überlappen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `a` | Anfang des ersten Segments (hex ohne $) | 8000 |
| `b` | Anfang des zweiten Segments (hex ohne $) | 8001 |

- de: Segmente überlappen: $8000 und $8001
- en: segments overlap: $8000 and $8001

### `diag-asm-e301-overlap-bank` (Fehler): Segmente überlappen in einer Bank

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `bank` | Banknummer | 2 |
| `a` | Anfang des ersten Segments (hex ohne $) | 8000 |
| `b` | Anfang des zweiten Segments (hex ohne $) | 8001 |

- de: Segmente überlappen (Bank 2): $8000 und $8001
- en: segments overlap (bank 2): $8000 and $8001

### `diag-asm-e301-segment-range` (Fehler): Segment verlässt seinen Bereich

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name des Segments | CODE |
| `lo` | Anfang des Bereichs (hex ohne $) | 8000 |
| `hi` | Ende des Bereichs (hex ohne $) | 8003 |
| `from` | Anfangsadresse (hex ohne $) | 8000 |
| `to` | Endadresse (hex ohne $) | 8004 |

- de: Segment 'CODE' verlässt seinen Bereich $8000-$8003 ($8000..$8004)
- en: segment 'CODE' leaves its range $8000-$8003 ($8000..$8004)

### `diag-basic-e301-needs-stage` (Fehler): Konstrukt braucht Stufe 2 (Fließkomma), die Einstellung erlaubt weniger

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `what` | das Konstrukt (Meldung) | Potenz '^' |
| `max` | höchste erlaubte Stufe | 1 |

- de: Potenz '^' braucht Stufe 2 (Fließkomma); erlaubt ist höchstens Stufe 1
- en: Potenz '^' needs stage 2 (floating point); the highest allowed stage is 1

## E302

### `diag-basic-e302-needs-stage` (Fehler): Konstrukt braucht Stufe 3 (Zeichenketten), die Einstellung erlaubt weniger

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `what` | das Konstrukt (Meldung) | Zeichenkettenfeld |
| `max` | höchste erlaubte Stufe | 2 |

- de: Zeichenkettenfeld braucht Stufe 3 (Zeichenketten); erlaubt ist höchstens Stufe 2
- en: Zeichenkettenfeld needs stage 3 (strings); the highest allowed stage is 2

## E303

### `diag-basic-e303-bcd-too-big` (Fehler): Zahl zu groß für das BASIC-Zahlenformat (Atari)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `v` | die Zahl | 1e100 |

- de: Zahl 1e100 ist zu groß für das BASIC-Zahlenformat (höchstens 9,99999999E+97)
- en: number 1e100 is too big for the BASIC number format (at most 9.99999999E+97)

### `diag-basic-e303-data-too-big-16` (Fehler): DATA-Zahl zu groß für 16 Bit

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `n` | die Zahl | 70000 |

- de: DATA-Zahl 70000 ist zu groß für 16 Bit
- en: DATA number 70000 is too big for 16 bits

### `diag-basic-e303-mbf-too-big` (Fehler): Zahl zu groß für das BASIC-Zahlenformat (CBM)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `v` | die Zahl | 1e40 |

- de: Zahl 1e40 ist zu groß für das BASIC-Zahlenformat
- en: number 1e40 is too big for the BASIC number format

### `diag-basic-e303-number-invalid` (Fehler): Zahl ungültig

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `s` | der Text | 1.2.3 |

- de: Zahl 1.2.3 ungültig
- en: number 1.2.3 is invalid

### `diag-basic-e303-too-big-16` (Fehler): Zahl zu groß für 16 Bit

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `n` | die Zahl | 70000 |

- de: Zahl 70000 ist zu groß für 16 Bit
- en: number 70000 is too big for 16 bits

## E304

### `diag-basic-e304-dim-string-one-size` (Fehler): DIM einer Zeichenkette braucht genau eine Größe

- de: DIM einer Zeichenkette braucht genau eine Größe: DIM A$(n)
- en: DIM of a string needs exactly one size: DIM A$(n)

### `diag-basic-e304-one-dim` (Fehler): Nur eindimensionale Felder umgesetzt

- de: Es sind nur eindimensionale Felder umgesetzt
- en: only one-dimensional arrays are implemented

### `diag-basic-e304-one-dim-stage1` (Fehler): Stufe 1 kennt nur eindimensionale Felder

- de: Stufe 1 kennt nur eindimensionale Felder
- en: stage 1 supports one-dimensional arrays only

### `diag-basic-e304-substring` (Fehler): Teilzeichenkette falsch geschrieben

- de: Teilzeichenkette: A$(n) oder A$(n,m)
- en: substring: A$(n) or A$(n,m)

## E305

### `diag-basic-e305-arg-count` (Fehler): Falsche Argumentzahl

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `n` | Name der Funktion | MID$ |

- de: MID$: falsche Argumentzahl
- en: MID$: wrong number of arguments

### `diag-basic-e305-one-arg` (Fehler): Funktion braucht genau ein Argument

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Funktion | ABS |

- de: ABS erwartet genau ein Argument
- en: ABS expects exactly one argument

### `diag-basic-e305-open-args` (Fehler): OPEN mit falschen Argumenten

- de: OPEN erwartet #Kanal, Zahl, Zahl, Name
- en: OPEN expects #channel, number, number, name

## E306

### `diag-basic-e306-function` (Fehler): Funktion wird nicht übersetzt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Funktion | FRE |

- de: Funktion FRE wird nicht übersetzt
- en: function FRE is not translated

## E307

### `diag-basic-e307-assign` (Fehler): Wert kann nicht zugewiesen werden

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Variablen | TI |

- de: TI kann nicht zugewiesen werden
- en: TI cannot be assigned

## E308

### `diag-basic-e308-input-multi` (Fehler): INPUT mit mehreren Variablen in Stufe 1

- de: INPUT mit mehreren Variablen wird in Stufe 1 nicht übersetzt (je Variable ein INPUT)
- en: INPUT with several variables is not translated in stage 1 (use one INPUT per variable)

## E309

### `diag-basic-e309-poke-range` (Fehler): POKE-Wert außerhalb 0..255

- de: POKE-Wert muss 0..255 sein
- en: POKE value must be 0..255

## E310

### `diag-basic-e310-run-line` (Fehler): RUN mit Zeilennummer

- de: RUN mit Zeilennummer wird nicht übersetzt
- en: RUN with a line number is not translated

## E311

### `diag-basic-e311-atari-only` (Fehler): Atari-Wort wird nicht übersetzt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `w` | das Wort | XIO |

- de: 'XIO' ist Atari-BASIC und wird nicht übersetzt (Kanal- und Dateibefehle, XIO, TRAP, NOTE/POINT, PADDLE u. a.)
- en: 'XIO' is Atari BASIC and is not translated (channel and file commands, XIO, TRAP, NOTE/POINT, PADDLE and others)

### `diag-basic-e311-get-string` (Fehler): GET in eine Zeichenkette (Atari)

- de: GET in eine Zeichenkette gibt es in Atari-BASIC nicht (GET #n,A liest ein Byte in eine Zahl)
- en: GET into a string does not exist in Atari BASIC (GET #n,A reads one byte into a number)

### `diag-basic-e311-not-in-atari` (Fehler): Funktion gibt es in Atari-BASIC nicht

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `n` | Name der Funktion | LEFT$ |

- de: LEFT$ gibt es in Atari-BASIC nicht (Teilzeichenkette A$(n,m) benutzen)
- en: LEFT$ does not exist in Atari BASIC (use the substring A$(n,m))

### `diag-basic-e311-plus-strings` (Fehler): + für Zeichenketten (Atari)

- de: Atari-BASIC kennt kein + für Zeichenketten (A$(LEN(A$)+1) = B$ hängt an)
- en: Atari BASIC has no + for strings (A$(LEN(A$)+1) = B$ appends)

### `diag-basic-e311-unsupported` (Fehler): Anweisung wird nicht übersetzt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `k` | die Anweisung | LIST |

- de: LIST wird nicht übersetzt (Dateien, LIST/NEW/CONT und DEF FN sind nicht Teil der Umsetzung)
- en: LIST is not translated (files, LIST/NEW/CONT and DEF FN are not part of the conversion)

## E312

### `diag-basic-e312-dim-number` (Fehler): DIM: Größe keine Zahl (Stufe 1)

- de: DIM braucht in Stufe 1 eine Zahl als Größe
- en: DIM needs a number as the size in stage 1

### `diag-basic-e312-dim-size-number` (Fehler): DIM A$(n): Größe keine Zahl

- de: DIM A$(n) braucht in basic2asm eine Zahl als Größe
- en: DIM A$(n) needs a number as the size in basic2asm

### `diag-basic-e312-dim-size-range` (Fehler): DIM A$(n): Größe außerhalb des Bereichs (Atari)

- de: DIM A$(n): n muss 1 bis 32767 sein (Atari-BASIC: sonst Fehler 9)
- en: DIM A$(n): n must be 1 to 32767 (Atari BASIC: otherwise error 9)

## E313

### `diag-basic-e313-redim` (Fehler): Feld zweimal dimensioniert

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name des Feldes | A |

- de: Feld A wird zweimal dimensioniert (REDIM'D ARRAY)
- en: array A is dimensioned twice (REDIM'D ARRAY)

## E314

### `diag-basic-e314-for-int` (Fehler): FOR-Variable ganzzahlig

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Variablen | I |

- de: FOR-Variable I% darf nicht ganzzahlig sein (im BASIC V2 ein SYNTAX ERROR)
- en: FOR variable I% must not be an integer variable (a SYNTAX ERROR in BASIC V2)

## E315

### `diag-basic-e315-fn-string` (Fehler): FN mit Zeichenketten-Name oder -Parameter

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Funktion | A$ |

- de: FNA$: Zeichenkettenfunktionen und -parameter werden nicht übersetzt
- en: FNA$: string functions and parameters are not translated

### `diag-basic-e315-fn-string-plain` (Fehler): Aufruf einer Zeichenkettenfunktion

- de: Zeichenkettenfunktionen werden nicht übersetzt
- en: string functions are not translated

## E316

### `diag-basic-e316-fn-twice` (Fehler): FN zweimal definiert

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Funktion | A |

- de: FNA wird zweimal definiert
- en: FNA is defined twice

## E317

### `diag-basic-e317-fn-undefined` (Fehler): FN nirgends definiert

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Funktion | A |

- de: FNA ist nirgends mit DEF FN definiert (UNDEF'D FUNCTION)
- en: FNA is not defined anywhere with DEF FN (UNDEF'D FUNCTION)

## E320

### `diag-basic-e320-arith` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

- de: Mit einer Zeichenkette kann man nicht rechnen (TYPE MISMATCH)
- en: you cannot calculate with a string (TYPE MISMATCH)

### `diag-basic-e320-assign` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

- de: Typen passen nicht (TYPE MISMATCH): Zeichenkette und Zahl
- en: types do not match (TYPE MISMATCH): string and number

### `diag-basic-e320-compare` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

- de: Zeichenkette und Zahl lassen sich nicht vergleichen (TYPE MISMATCH)
- en: a string and a number cannot be compared (TYPE MISMATCH)

### `diag-basic-e320-concat` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

- de: Zeichenkette und Zahl lassen sich nicht verketten (TYPE MISMATCH)
- en: a string and a number cannot be joined (TYPE MISMATCH)

### `diag-basic-e320-dec-string` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

- de: DEC erwartet eine Zeichenkette (TYPE MISMATCH)
- en: DEC expects a string (TYPE MISMATCH)

### `diag-basic-e320-expects-string` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `n` | Name der Funktion | LEFT$ |

- de: LEFT$ erwartet eine Zeichenkette (TYPE MISMATCH)
- en: LEFT$ expects a string (TYPE MISMATCH)

### `diag-basic-e320-fn-number` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Funktion | A |

- de: FNA erwartet eine Zahl (TYPE MISMATCH)
- en: FNA expects a number (TYPE MISMATCH)

### `diag-basic-e320-instr` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

- de: INSTR(a$,b$) oder INSTR(a$,b$,n) mit Zeichenketten (TYPE MISMATCH)
- en: INSTR(a$,b$) or INSTR(a$,b$,n) with strings (TYPE MISMATCH)

### `diag-basic-e320-number-expected` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

- de: Zahl erwartet, aber eine Zeichenkette (TYPE MISMATCH)
- en: number expected, but got a string (TYPE MISMATCH)

### `diag-basic-e320-open-name` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

- de: OPEN: der Dateiname muss eine Zeichenkette sein (TYPE MISMATCH)
- en: OPEN: the file name must be a string (TYPE MISMATCH)

### `diag-basic-e320-returns-string` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `n` | Name der Funktion | CHR$ |

- de: CHR$ liefert eine Zeichenkette, hier wird eine Zahl erwartet (TYPE MISMATCH)
- en: CHR$ returns a string, but a number is expected here (TYPE MISMATCH)

### `diag-basic-e320-str-number` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

- de: STR$ erwartet eine Zahl (TYPE MISMATCH)
- en: STR$ expects a number (TYPE MISMATCH)

### `diag-basic-e320-string-expected` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

- de: Zeichenkette, aber eine Zahl erwartet (TYPE MISMATCH)
- en: string, but a number was expected (TYPE MISMATCH)

### `diag-basic-e320-string-expr` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

- de: Zeichenkettenausdruck erwartet (TYPE MISMATCH)
- en: string expression expected (TYPE MISMATCH)

### `diag-basic-e320-string-ops` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

- de: Mit Zeichenketten sind nur + und Vergleiche erlaubt (TYPE MISMATCH)
- en: only + and comparisons are allowed with strings (TYPE MISMATCH)

### `diag-basic-e320-string-var-number` (Fehler): Typfehler (TYPE MISMATCH) in einem Ausdruck

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Variablen | A |

- de: Zeichenkettenvariable A$, aber eine Zahl erwartet (TYPE MISMATCH)
- en: string variable A$, but a number was expected (TYPE MISMATCH)

## E321

### `diag-basic-e321-string-dim` (Fehler): Zeichenkettenvariable ohne DIM (Atari)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Variablen | A$ |

- de: Zeichenkettenvariable A$ braucht ein DIM mit fester Größe (Atari-BASIC: DIM A$(n), sonst Fehler 9)
- en: string variable A$ needs a DIM with a fixed size (Atari BASIC: DIM A$(n), otherwise error 9)

## E322

### `diag-basic-e322-data-entry` (Fehler): DATA-Eintrag keine Zahl (Atari)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `entry` | der Eintrag | abc |

- de: DATA-Eintrag 'abc' ist keine Zahl (Atari-BASIC: Fehler 8 beim Lesen)
- en: DATA entry 'abc' is not a number (Atari BASIC: error 8 when reading)

## E330

### `diag-basic-e330-target` (Fehler): Ziel nicht umgesetzt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `target` | das Ziel | vic20 |
| `available` | die verfügbaren Ziele | c64, c128, plus4, mega65, atari8 |

- de: Ziel 'vic20' ist nicht umgesetzt (verfügbar: c64, c128, plus4, mega65, atari8)
- en: target 'vic20' is not implemented (available: c64, c128, plus4, mega65, atari8)

### `diag-ide-e330-io` (Fehler): Datei nicht les- oder schreibbar

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `path` | Pfad | spiel.bas |
| `error` | Fehlertext | Permission denied |

- de: spiel.bas: Permission denied
- en: spiel.bas: Permission denied

### `diag-ide-e330-not-generated` (Fehler): Ausgabedatei existiert und ist nicht erzeugt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `file` | Name der Ausgabedatei | spiel.asm |

- de: 'spiel.asm' existiert und wurde nicht von basic2asm erzeugt – wird nicht überschrieben (anderen Namen wählen)
- en: 'spiel.asm' exists and was not created by basic2asm – it will not be overwritten (choose another name)

### `diag-ide-e330-output-extension` (Fehler): Ausgabedatei hat nicht die Endung .asm

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `file` | Name der Ausgabedatei | spiel.txt |

- de: Ausgabedatei 'spiel.txt' muss auf .asm enden
- en: Output file 'spiel.txt' must end in .asm

### `diag-ide-e330-target-unsupported` (Fehler): Zielsystem der Konvertierung nicht umgesetzt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `target` | gewünschtes Ziel | c128 |
| `available` | verfügbare Ziele, kommagetrennt | c64, atari |

- de: Ziel 'c128' ist noch nicht umgesetzt (verfügbar: c64, atari)
- en: Target 'c128' is not supported yet (available: c64, atari)

### `diag-ide-e330-wrong-extension` (Fehler): falsche Dateiendung der Quelle

- de: Nur .bas-, .lst- oder .tur-Dateien (natives BASIC) lassen sich konvertieren
- en: Only .bas, .lst, or .tur files (native BASIC) can be converted

## E331

### `diag-basic-e331-float-target` (Fehler): Fließkomma für das Ziel nicht umgesetzt

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `t` | das Ziel | mega65 |
| `available` | Ziele mit Fließkomma | c64,c128,plus4,atari8 |

- de: Fließkomma (Stufe 2) ist für das Ziel mega65 noch nicht umgesetzt (verfügbar: c64,c128,plus4,atari8); das Programm braucht es
- en: floating point (stage 2) is not yet implemented for the target mega65 (available: c64,c128,plus4,atari8); the program needs it

## E334

### `diag-basic-e334-tan` (Fehler): TAN gibt es in Atari-BASIC nicht

- de: TAN gibt es in Atari-BASIC nicht (SIN(x)/COS(x) benutzen)
- en: TAN does not exist in Atari BASIC (use SIN(x)/COS(x))

## E340

### `diag-basic-e340-proc-twice` (Fehler): Prozedur zweimal definiert (Turbo BASIC XL)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `n` | Name der Prozedur | zeichne |

- de: Prozedur zeichne wird zweimal definiert
- en: procedure zeichne is defined twice

## E341

### `diag-basic-e341-open-blocks` (Fehler): Turbo-BASIC-Blöcke am Ende nicht abgeschlossen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `n` | Zahl der offenen Blöcke | 2 |

- de: 2 Block(s) von Turbo BASIC XL am Programmende nicht abgeschlossen (ENDIF, WEND, UNTIL, LOOP oder ENDPROC fehlt)
- en: (mit Mehrzahl/Auswahl) `{ $n ->         [one] 1 block        *[other] { $n } blocks     } of Turbo BASIC XL not closed at the end of the program (ENDIF, WEND, UNTIL, LOOP or ENDPROC is missing)`

## E342

### `diag-basic-e342-else` (Fehler): Turbo-BASIC-Block passt nicht

- de: ELSE ohne passendes IF (oder zweites ELSE)
- en: ELSE without a matching IF (or a second ELSE)

### `diag-basic-e342-endif` (Fehler): Turbo-BASIC-Block passt nicht

- de: ENDIF ohne passendes IF
- en: ENDIF without a matching IF

### `diag-basic-e342-endproc` (Fehler): Turbo-BASIC-Block passt nicht

- de: ENDPROC ohne passendes PROC
- en: ENDPROC without a matching PROC

### `diag-basic-e342-exit` (Fehler): Turbo-BASIC-Block passt nicht

- de: EXIT außerhalb von WHILE, REPEAT und DO
- en: EXIT outside of WHILE, REPEAT and DO

### `diag-basic-e342-loop` (Fehler): Turbo-BASIC-Block passt nicht

- de: LOOP ohne passendes DO
- en: LOOP without a matching DO

### `diag-basic-e342-proc-nested` (Fehler): Turbo-BASIC-Block passt nicht

- de: PROC innerhalb einer Prozedur
- en: PROC inside a procedure

### `diag-basic-e342-until` (Fehler): Turbo-BASIC-Block passt nicht

- de: UNTIL ohne passendes REPEAT
- en: UNTIL without a matching REPEAT

### `diag-basic-e342-wend` (Fehler): Turbo-BASIC-Block passt nicht

- de: WEND ohne passendes WHILE
- en: WEND without a matching WHILE

## E343

### `diag-basic-e343-exec-undefined` (Fehler): Prozedur in EXEC nicht definiert

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `n` | Name der Prozedur | zeichne |

- de: EXEC: Prozedur zeichne ist nicht definiert
- en: EXEC: procedure zeichne is not defined

### `diag-basic-e343-on-exec-undefined` (Fehler): Prozedur in ON … EXEC nicht definiert

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `n` | Name der Prozedur | zeichne |

- de: ON … EXEC: Prozedur zeichne ist nicht definiert
- en: ON ... EXEC: procedure zeichne is not defined

## E344

### `diag-basic-e344-turbo-target` (Fehler): Turbo BASIC XL nur für atari8

- de: Turbo BASIC XL gibt es nur für das Ziel atari8
- en: Turbo BASIC XL exists only for the target atari8

## I110

### `diag-basic-i110-float-var` (Warnung): Fließkommavariable, Stufe 1 rechnet mit Ganzzahlen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name der Variablen | X |

- de: Variable X ist im Original eine Fließkommazahl; Stufe 1 rechnet mit 16-Bit-Ganzzahlen
- en: variable X is a floating-point number in the original; stage 1 uses 16-bit integers

## I111

### `diag-basic-i111-big-numbers` (Warnung): Zahlen über 32767 gelten als 16-Bit-Muster

- de: Zahlen über 32767 gelten als 16-Bit-Muster (Adressen für PEEK/POKE/SYS); im Rechnen und im PRINT erscheinen sie negativ
- en: numbers above 32767 are treated as 16-bit patterns (addresses for PEEK/POKE/SYS); in calculations and in PRINT they appear negative

## I112

### `diag-basic-i112-ti-16bit` (Warnung): TI liefert nur 16 Bit

- de: TI liefert nur die unteren 16 Bit der Systemuhr
- en: TI returns only the lower 16 bits of the system clock

## I113

### `diag-basic-i113-int-noop` (Warnung): INT in Stufe 1 ohne Wirkung

- de: INT ist in Stufe 1 ohne Wirkung (nur Ganzzahlen)
- en: INT has no effect in stage 1 (integers only)

## I150

### `diag-basic-i150-atari-bcd` (Warnung): Hinweis zu Zahlen beim Atari-Ziel (Fließkomma)

- de: Atari: Zahlen sind 6-Byte-BCD des OS-Fließkommapakets (wie Atari-BASIC); TAN gibt es nicht; PRINT gibt Zahlen wie Atari-BASIC ohne Vorzeichenstelle aus; Fehlermeldungen heißen wie bei CBM (z. B. ILLEGAL QUANTITY)
- en: Atari: numbers are 6-byte BCD of the OS floating-point package (as in Atari BASIC); TAN does not exist; PRINT prints numbers like Atari BASIC without a sign position; error messages are named as on CBM (e.g. ILLEGAL QUANTITY)

### `diag-basic-i150-atari-int` (Warnung): Hinweis zu Zahlen beim Atari-Ziel (Ganzzahl)

- de: Atari: Zahlen sind 16-Bit-Ganzzahlen statt BCD mit 10 Stellen; PRINT gibt Zahlen wie Atari-BASIC ohne Vorzeichenstelle aus; Fehlermeldungen heißen wie bei CBM (z. B. ILLEGAL QUANTITY)
- en: Atari: numbers are 16-bit integers instead of BCD with 10 digits; PRINT prints numbers like Atari BASIC without a sign position; error messages are named as on CBM (e.g. ILLEGAL QUANTITY)

## W104

### `diag-asm-w104` (Warnung): undokumentierter Opcode (illegals=warn)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `mn` | Befehl (Mnemonic) | LDA |

- de: LDA ist ein undokumentierter Opcode (illegals=warn)
- en: LDA is an undocumented opcode (illegals=warn)

## W105

### `diag-asm-w105` (Warnung): instabiler undokumentierter Opcode

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `mn` | Befehl (Mnemonic) | LDA |

- de: LDA ist ein instabiler undokumentierter Opcode
- en: LDA is an unstable undocumented opcode

## W200

### `diag-asm-w200` (Warnung): Warnung aus .warning (Text vom Programmierer)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `text` | Text hinter .warning | Tabelle fast voll |

- de: Tabelle fast voll
- en: Tabelle fast voll

## W201

### `diag-asm-w201` (Warnung): Zyklenbudget überschritten

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `name` | Name des Blocks | irq |
| `max` | Zyklen im ungünstigsten Fall | 120 |
| `limit` | Budget in Zyklen | 100 |

- de: Zyklenbudget 'irq' überschritten: worst-case 120 > 100 Zyklen
- en: cycle budget 'irq' exceeded: worst case 120 > 100 cycles

## W202

### `diag-asm-w202` (Warnung): Kommazahl abgeschnitten

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `value` | Kommazahl | 3.5 |
| `to` | ganze Zahl nach dem Abschneiden | 3 |
| `expr` | Ausdruck | 7 / 2 |

- de: Kommazahl 3.5 auf 3 abgeschnitten (int(), round() oder floor() verwenden) in '7 / 2'
- en: floating-point number 3.5 truncated to 3 (use int(), round() or floor()) in '7 / 2'

## W203

### `diag-asm-w203` (Warnung): Sprung zu weit, lange Form verwendet

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `mn` | Befehl (Mnemonic) | LDA |
| `len` | Länge der langen Form in Bytes | 5 |

- de: Sprung zu weit für LDA: lange Form (5 Bytes) verwendet
- en: branch too far for LDA: long form (5 bytes) used

## W204

### `diag-asm-w204` (Warnung): .basicstub nicht an der BASIC-Startadresse

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `pc` | Adresse (hex ohne $) | 1000 |
| `start` | BASIC-Startadresse (hex ohne $) | 0801 |

- de: .basicstub steht bei $1000, die BASIC-Startadresse des Zielsystems ist $0801 (BASIC findet die Zeile dort nicht)
- en: .basicstub is at $1000, the BASIC start address of the target system is $0801 (BASIC will not find the line there)

## W205

### `diag-asm-w205` (Warnung): .basicstub auf einem Ziel ohne BASIC-Zeile

- de: .basicstub hat auf diesem Ziel keine Wirkung (kein BASIC mit SYS); für den Start .run label verwenden
- en: .basicstub has no effect on this target (no BASIC with SYS); use .run label to set the start

## W206

### `diag-asm-w206` (Warnung): .run auf einem Ziel ohne Startvektor

- de: .run hat auf diesem Ziel keine Wirkung (PRG hat keinen Startvektor); für den Start .basicstub label verwenden
- en: .run has no effect on this target (a PRG has no start vector); use .basicstub label to set the start

## W207

### `diag-asm-w207` (Warnung): Programm ohne Startvektor

- de: Kein Startvektor festgelegt; mit .run label ergänzen
- en: No start vector set; add one with .run label

## W208

### `diag-asm-w208-legacy-export` (Warnung): alter Include-Pfad auf eine Exportdatei im Build-Ordner

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `path` | Pfad im Include | build/export/hero.asm |
| `media` | Mediendatei im Projekt | graphics/hero.vcsprite |

- de: Veralteter Pfad 'build/export/hero.asm': das Medium direkt einbinden, z. B. .include "graphics/hero.vcsprite"
- en: Outdated path 'build/export/hero.asm': include the media file directly, e.g. .include "graphics/hero.vcsprite"

## W312

### `diag-basic-w312-same-variable` (Warnung): Zwei Variablennamen sind im BASIC dieselbe Variable (nur 2 Zeichen zählen)

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `a` | erster Name | PUNKTE |
| `b` | zweiter Name | PUNKTZAHL |

- de: PUNKTE und PUNKTZAHL sind im BASIC dieselbe Variable (nur die ersten 2 Zeichen zählen)
- en: PUNKTE and PUNKTZAHL are the same variable in BASIC (only the first 2 characters count)

## W313

### `diag-basic-w313-names-collide` (Warnung): Lange Namen unterscheiden sich erst nach den signifikanten Zeichen

| Parameter | Bedeutung | Beispiel |
|---|---|---|
| `names` | die langen Namen, in Anführungszeichen, mit Komma getrennt | 'punktestand', 'punktezahl' |
| `sig` | Zahl der unterschiedenen Zeichen | 2 |
| `nat` | gemeinsamer Anfang | PU |
| `shorts` | die vergebenen Kurznamen | PU, PA |

- de: 'punktestand', 'punktezahl' unterscheiden sich erst nach den ersten 2 Zeichen ('PU'); sie werden hier getrennt (PU, PA). Ein BASIC ohne diese Umsetzung sähe nur eine Variable
- en: 'punktestand', 'punktezahl' differ only after the first 2 characters ('PU'); they are kept apart here (PU, PA). A BASIC without this conversion would see only one variable
