# BASIC: Spickzettel und Fehlercodes

Kurzfassung zum Nachschlagen. Die ausführliche Erklärung steht auf den Seiten [BASIC](help:basic), [Sprungmarken und Zeilennummern](help:basic-labels), [Lange Variablennamen](help:basic-names), [Direktiven, Dateien und Assets](help:basic-directives), [Dialekte](help:basic-dialects) und [Werkzeuge](help:basic-tools).

## Aufbau einer Zeile

```
100 @name: anweisung : anweisung     ; Zeilennummer, Marke, mehrere Anweisungen
@name:                               ; Marke für die nächste Anweisung
anweisung                            ; ohne Nummer: wird beim Bauen nummeriert
; kommentar                          ; Kommentar, kommt nicht ins Programm
#include "datei.bas"                 ; Datei einfügen
#longnames                           ; lange Variablennamen
```

Der Teil hinter `;` in diesem Kasten ist nur Erklärung; im Programm ist `;` mitten in einer Zeile **kein** Kommentar.

## Schreibweisen im Überblick

| Aufgabe | Schreibweisen (alle gleichwertig) |
|---|---|
| Sprungziel | `goto @name`, `goto 100` (feste Nummer), auf Turbo BASIC XL zusätzlich `go# name` |
| Marke festlegen | `@name:` allein in der Zeile, `@name: anweisung`, `100 @name: anweisung` |
| Mehrere Marken | `@a: @b: anweisung` |
| Datei einbinden | `#include "datei.bas"`, `#INCLUDE "datei.bas"` |
| Lange Namen | `#longnames`, `#LONGNAMES` |
| Kommentar | `; text` (Zeilenanfang, kommt nicht ins Programm), `rem text` (bleibt im Programm), Turbo BASIC XL: `' text` |
| Ausgabe | `print`, `?` |
| Bildschirm löschen (C64) | `print "{clr}"`, `print "{$93}"`, `print chr$(147)` |
| Steuerzeichen wiederholen | `{down*3}` oder `{down}{down}{down}` |
| `rvs on` | `{rvs on}`, `{rvon}` |

## Direktiven

| Zeile | Wirkung | Seite |
|---|---|---|
| `#include "pfad"` | Datei einfügen (relativ zur einbindenden Datei, bis 8 Ebenen) | [Direktiven](help:basic-directives) |
| `#longnames` | lange Namen erlauben | [Lange Namen](help:basic-names) |

Es gibt **kein** `#if`, `#define` oder `#else` im nativen BASIC. Solche Zeilen werden nicht erkannt, sondern als BASIC-Text übersetzt.

## Dialekte

| System | Dialekt | Namen | Zeilen bis |
|---|---|---|---|
| C64, VC-20 | `cbm_v2` | 2 Zeichen | 63999 |
| C128 | `basic7` | 2 Zeichen | 63999 |
| Plus/4 | `basic35` | 2 Zeichen | 63999 |
| MEGA65 | `basic65` | 2 Zeichen | 63999 |
| Atari | `atari_basic` | alle Zeichen, ≤ 128 Variablen | 32767 |
| Atari | `turbo_basic_xl` | alle Zeichen, ≤ 256 Variablen | 32767 |

Reservierte Variablen (Commodore): `ST`, `TI`, `DS`; ab BASIC 3.5 auch `ER`, `EL`. Ein Zeilen-Byte-Limit gilt: 250 Byte (Commodore), 255 Byte (Atari).

## Kommandozeile

| Befehl | Wirkung |
|---|---|
| `vcc build` | Projekt bauen, auch BASIC-Quellen (`vcc build projekt`, `--main`) |
| `vcc basic datei.bas` | eine `.bas`- oder `.lst`-Datei bauen (`--target`, `--dialect`, `--memory`, `--start`, `--step`, `-o`) |
| `vcc basic --tokenize datei.lst` | klassisches Listing mit Zeilennummern tokenisieren |
| `vcc basic --detokenize datei.prg` | Programm als Listing ausgeben |
| `vcc basic2asm datei.bas` | in Assembler umwandeln (`--target c64`, `c128`, `plus4`; `--check`, `--stage`, `--float`, `--int-div`, `--map`, `--no-rem`, `--no-track-line`) |
| `vcc import datei.d64 NAME` | BASIC-Programm von einer Diskette als `.bas`-Quelltext übernehmen |
| `vcc new ordner --template basic-hello` | neues Projekt mit BASIC-Vorlage |

## Fehlercodes

Meldungen haben die Form `datei.bas:ZEILE: error CODE: text`. Bei eingebundenen Dateien zeigt `datei.bas` auf die eingebundene Datei.

| Code | Meldung | Erklärung |
|---|---|---|
| B100 | `Zeilennummer N größer als 63999` (Atari: 32767) | zu viele Zeilen oder zu große eigene Nummer |
| B100 | `Zeilennummer N nicht größer als die vorige (M)` | eigene Nummern nicht aufsteigend oder mit der automatischen Nummer deckungsgleich |
| B100 | `Zeichen 'x' hat keine PETSCII-Entsprechung` | Zeichen (Umlaut, `\` …) im Commodore-Text nicht vorhanden: `{$xx}` benutzen |
| B100 | `Zeichen 'x' nicht im ATASCII-Bereich (nutze {$xx})` | dasselbe für den Atari |
| B100 | `unbekannter Steuercode {name}`, `'}' fehlt`, `ungültiger Code …`, `ungültige Anzahl in …` | falsche Steuerzeichen in `{…}` |
| B100 | `'=' nach 'X' erwartet`, `Ausdruck unvollständig`, `unerwartetes Zeichen 'c'` | Atari: Syntaxfehler in der Anweisung |
| B100 | `mehr als 128 Variablen` | Atari BASIC: zu viele Variablen (Turbo BASIC XL: 256) |
| B100 | `Zeile zu lang (mehr als 255 Bytes)` | Atari: zu viel in einer Zeile |
| B100 | `Zeile beginnt nicht mit einer Zeilennummer` | nur bei `vcc basic --tokenize` (klassisches Listing) |
| B102 | `Zeile zu lang (mehr als 250 Bytes)` | Commodore: Zeile teilen |
| B110 | `Label @x ist nicht definiert` | Marke fehlt oder steht mitten in einer Zeile |
| B111 | `Label @x ist doppelt definiert` | zwei gleiche Marken |
| B112 | `Label @x steht am Ende ohne Anweisung dahinter` | Marke hat keine folgende Zeile |
| B120 | `#include "x": Datei nicht gefunden …` | Pfad falsch oder Export fehlt |
| B121 | `#include "x": zu tief verschachtelt oder zyklisch` | Dateien binden sich im Kreis ein, mehr als 8 Ebenen |
| B131 | `Für den Namen 'x' ist kein freier Kurzname mehr übrig (zu viele Variablen)` | nur mit `#longnames` |
| W313 | `'a', 'b' unterscheiden sich erst nach den ersten 2 Zeichen …` | Warnung: lange Namen mit gleichem Anfang, getrennt umgesetzt |

Die Codes **E…** und **I…** gehören zu [basic2asm](help:basic-tools) (E3xx: nicht übersetzbare Konstrukte, I110–I113: Hinweise zur Ganzzahl-Stufe). Codes **E2…** und **W2…** stammen vom [Assembler](help:asm-index).

## Häufige Missverständnisse

| Erwartung | Tatsächlich |
|---|---|
| Der Übersetzer meldet Syntaxfehler | Nur beim Atari. Commodore-Programme prüft erst das Gerät. |
| `#if SYSTEM_C64 … #endif` wählt Code je System | Gibt es für `.bas` nicht, die Zeilen werden als BASIC-Text übersetzt. |
| `restore @name` geht überall | Nicht auf C64 und VC-20 (BASIC V2). |
| `@name:` darf mitten in der Zeile stehen | Nur am Zeilenanfang (B110 sonst). |
| Lange Namen sind immer an | Nur mit `#longnames`. |
| `'` ist ein Kommentar | Nur in Turbo BASIC XL. Sonst `;` am Zeilenanfang oder `rem`. |
| Kleinbuchstaben im C64-Text sind klein | Sie erscheinen als Großschrift; Großbuchstaben werden zu Grafikzeichen. |
