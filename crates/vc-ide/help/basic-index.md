# BASIC: Spickzettel und Fehlercodes

Kurzfassung zum Nachschlagen. Die ausführliche Erklärung steht auf den Seiten [BASIC](help:basic), [Sprungmarken und Zeilennummern](help:basic-labels), [Lange Variablennamen](help:basic-names), [Direktiven, Dateien und Assets](help:basic-directives), [Dialekte](help:basic-dialects) und [Werkzeuge](help:basic-tools).

## Aufbau einer Zeile

```
100 @name: anweisung : anweisung     ; Zeilennummer, Marke, mehrere Anweisungen
@name:                               ; Marke für die nächste Anweisung
anweisung                            ; ohne Nummer: wird beim Bauen nummeriert
; kommentar                          ; Kommentar, kommt nicht ins Programm
#include "datei.bas"                 ; Datei oder Mediendatei einfügen
#longnames                           ; lange Variablennamen
#define NAME wert                    ; Konstante
#if SYSTEM_C64 ... #endif            ; bedingte Übersetzung
#numbering start=10 step=10          ; Nummerierung dieser Datei
#dialect atari_basic                 ; Dialekt dieser Datei
#asm "datei.asm"                     ; Maschinencode zum Programm
```

Der Teil hinter `;` in diesem Kasten ist nur Erklärung; im Programm ist `;` mitten in einer Zeile **kein** Kommentar.

## Schreibweisen im Überblick

| Aufgabe | Schreibweisen (alle gleichwertig) |
|---|---|
| Sprungziel | `goto @name`, `goto 100` (feste Nummer), auf Turbo BASIC XL zusätzlich `go# name` |
| Marke festlegen | `@name:` allein in der Zeile, `@name: anweisung`, `100 @name: anweisung` |
| Mehrere Marken | `@a: @b: anweisung` |
| Datei einbinden | `#include "datei.bas"`, `#INCLUDE "datei.bas"` |
| Medium einbinden | `#include "graphics/hero.vcsprite"` (DATA-Zeilen mit Marke `@hero:`, Zeilennummer in `hero_line`) |
| Konstante | `#define RAHMEN 53280`, dann `poke RAHMEN, 0` |
| Maschinencode | `#asm "player.asm"`, dann `sys asm.start` |
| Lange Namen | `#longnames`, `#LONGNAMES` |
| Kommentar | `; text` (Zeilenanfang, kommt nicht ins Programm), `rem text` (bleibt im Programm), Turbo BASIC XL: `' text` |
| Ausgabe | `print`, `?` |
| Bildschirm löschen (C64) | `print "{clr}"`, `print "{$93}"`, `print chr$(147)` |
| Steuerzeichen wiederholen | `{down*3}` oder `{down}{down}{down}` |
| `rvs on` | `{rvs on}`, `{rvon}` |

## Direktiven

| Zeile | Wirkung | Seite |
|---|---|---|
| `#include "pfad"` | Datei oder Mediendatei einfügen (relativ zur einbindenden Datei, bis 8 Ebenen) | [Direktiven](help:basic-directives) |
| `#longnames` | lange Namen erlauben | [Lange Namen](help:basic-names) |
| `#define NAME wert` | Konstante, ersetzt `NAME` vor dem Nummerieren | [Direktiven](help:basic-directives) |
| `#if BEDINGUNG`, `#else`, `#endif` | bedingte Übersetzung (`SYSTEM_C64`, ein `#define`-Name, `!` davor) | [Direktiven](help:basic-directives) |
| `#numbering start=N step=N` | Start und Schrittweite der Zeilennummern | [Direktiven](help:basic-directives), [Sprungmarken](help:basic-labels) |
| `#dialect id` | Dialekt dieser Datei | [Dialekte](help:basic-dialects) |
| `#asm "datei.asm" [with "song.asm"]` | Maschinencode übersetzen und zum Programm legen; Marken als `asm.name` | [Direktiven](help:basic-directives) |

Eine andere Zeile, die mit `#` beginnt, erkennt der Übersetzer nicht; er übersetzt sie als BASIC-Text. Der frühere Name `#blob` gilt nicht mehr: Er meldet **B177** mit dem Hinweis auf `#asm`.

## Dialekte

| System | Dialekt | Namen | Zeilen bis |
|---|---|---|---|
| C64 | `cbm_v2` | 2 Zeichen | 63999 |
| VC-20 | `vic_basic_2` (baut auf `cbm_v2` auf) | 2 Zeichen | 63999 |
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
| `vcc basic datei.bas` | eine `.bas`- oder `.lst`-Datei bauen (`--target`, `--dialect`, `--memory`, `--start`, `--step`, `-D`, `--compact`, `-o`) |
| `vcc basic datei.bas --check` | nur prüfen: Warnungen B150 bis B167 ausgeben, nichts schreiben (`--no-check` schaltet die Prüfung aus, `--disable B152,B154` einzelne Warnungen) |
| `vcc basic --to-labels datei.bas` | Zeilennummern in Labels umwandeln (`--to-numbers`: Labels in Nummern, `--renumber`: neu nummerieren; `--preview`, `--write`, `--range von-bis`) |
| `vcc basic --tokenize datei.lst` | klassisches Listing mit Zeilennummern tokenisieren |
| `vcc basic --detokenize datei.prg` | Programm als Listing ausgeben |
| `vcc basic2asm datei.bas` | in Assembler umwandeln (`--target c64`, `c128`, `plus4`; `--check`, `--stage`, `--float`, `--int-div`, `--map`, `--no-rem`, `--no-track-line`) |
| `vcc import datei.d64 NAME` | BASIC-Programm von einer Diskette als `.bas`-Quelltext übernehmen |
| `vcc new ordner --template basic-hello` | neues Projekt mit BASIC-Vorlage (mit `--structure standard` auch mit Ordnerstruktur) |
| `vcc migrate includes` | alte Include-Pfade `build/export/…` auf die Mediendatei umstellen (`--apply` schreibt) |

## Fehlercodes

Meldungen haben die Form `datei.bas:ZEILE: error CODE: text`. Bei eingebundenen Dateien zeigt `datei.bas` auf die eingebundene Datei.

| Code | Meldung | Erklärung |
|---|---|---|
| B100 | `Zeilennummer N größer als 63999` | zu viele Zeilen oder zu große eigene Nummer (Atari: B185, Grenze 32767) |
| B100 | `Zeilennummer N nicht größer als die vorige (M)` | eigene Nummern nicht aufsteigend oder mit der automatischen Nummer deckungsgleich |
| B100 | `Zeichen 'x' hat keine PETSCII-Entsprechung` | Zeichen (Umlaut, `\` …) im Commodore-Text nicht vorhanden: `{$xx}` benutzen |
| B184 | `Zeichen 'x' nicht im ATASCII-Bereich (nutze {$xx})` | dasselbe für den Atari |
| B100 | `unbekannter Steuercode {name}`, `'}' fehlt`, `ungültiger Code …`, `ungültige Anzahl in …` | falsche Steuerzeichen in `{…}` |
| B180, B181, B182, B183 | `'=' nach 'X' erwartet`, `Ausdruck unvollständig`, `'x' ist kein gültiger Labelname`, `ungültige Zahl` | Atari: Syntaxfehler in Anweisung, Ausdruck, Name oder Zahl |
| B100 | `unerwartetes Zeichen 'c'` | Zeichen, das es im Dialekt nicht gibt (z. B. `_` in einem Atari-Namen) |
| B185 | `mehr als 128 Variablen` | Atari BASIC: zu viele Variablen (Turbo BASIC XL: 256) |
| B185 | `Zeile zu lang (mehr als 255 Bytes)` | Atari: zu viel in einer Zeile |
| B100 | `Zeile beginnt nicht mit einer Zeilennummer` | nur bei `vcc basic --tokenize` (klassisches Listing) |
| B102 | `Zeile zu lang (mehr als 250 Bytes)` | Commodore: Zeile teilen |
| B110 | `Label @x ist nicht definiert` | Marke fehlt oder steht mitten in einer Zeile |
| B111 | `Label @x ist doppelt definiert` | zwei gleiche Marken |
| B112 | `Label @x steht am Ende ohne Anweisung dahinter` | Marke hat keine folgende Zeile |
| B120 | `#include "x": Datei nicht gefunden …` | Pfad falsch oder alter Exportpfad ohne Datei |
| B121 | `#include "x": zu tief verschachtelt oder zyklisch` | Dateien binden sich im Kreis ein, mehr als 8 Ebenen |
| B131 | `Für den Namen 'x' ist kein freier Kurzname mehr übrig (zu viele Variablen)` | nur mit `#longnames` |
| B150 bis B159 | Warnungen der Syntaxprüfung | verdächtige Stellen, siehe [BASIC](help:basic) (Abschnitt Syntaxprüfung); abschaltbar |
| B160, B161, B162, B166 | Zeilennummern und Labels gemischt; Umwandlung nicht möglich; berechnetes Sprungziel; Modus einer eingebundenen Datei passt nicht | beim Umwandeln, siehe [Sprungmarken](help:basic-labels) |
| B167, B168 | `'` ist kein Kommentarzeichen; ungültige Markerzeile `;@name` | Warnung |
| B170, B178 | Dialekt passt nicht zum Zielsystem; unbekannter Dialekt | `#dialect` oder Projekteinstellung |
| B172, B174, B175, B176 | `#define` mit Schlüsselwort, falsche Schreibweise, `#if` falsch aufgebaut, `#define` zyklisch | siehe [Direktiven](help:basic-directives) |
| B173, B177 | `#asm`: Zeilennummern reichen nicht für die DATA-Zeilen; Schreibweise, fehlende Marke, zu großer Code, alter Name `#blob` | siehe [Direktiven](help:basic-directives) |
| B190, B191, B192 | `#numbering` falsch geschrieben; Wertebereich; Nummern reichen nicht (mit Vorschlag für die Schrittweite) | siehe [Sprungmarken](help:basic-labels) |
| B193, B194, B195, B196 | Medium nicht einbindbar; alter Pfad `build/export/…`; Programm größer als der freie Speicher; mehrere Medien und `RESTORE @marke` in BASIC V2 | siehe [Direktiven](help:basic-directives) |
| W313 | `'a', 'b' unterscheiden sich erst nach den ersten 2 Zeichen …` | Warnung: lange Namen mit gleichem Anfang, getrennt umgesetzt |

Die Codes **E…** und **I…** gehören zu [basic2asm](help:basic-tools) (E3xx: nicht übersetzbare Konstrukte, I110–I113: Hinweise zur Ganzzahl-Stufe). Codes **E2…** und **W2…** stammen vom [Assembler](help:asm-index).

## Häufige Missverständnisse

| Erwartung | Tatsächlich |
|---|---|
| Der Übersetzer meldet Syntaxfehler | Nur beim Atari als Fehler. Bei den anderen Dialekten gibt die Syntaxprüfung Warnungen aus (B150 bis B159); das Gerät prüft weiter erst beim Ausführen. |
| `#if SYSTEM_C64 … #endif` wählt Code je System | Das gibt es: Der Übersetzer lässt die Zeilen des anderen Zweigs weg. |
| `restore @name` geht überall | Nicht auf C64 und VC-20 (BASIC V2). |
| `@name:` darf mitten in der Zeile stehen | Nur am Zeilenanfang (B110 sonst). |
| Lange Namen sind immer an | Nur mit `#longnames`. |
| `'` ist ein Kommentar | Nur in Turbo BASIC XL. Sonst `;` am Zeilenanfang oder `rem`. |
| Kleinbuchstaben im C64-Text sind klein | Sie erscheinen als Großschrift; Großbuchstaben werden zu Grafikzeichen. |
