# BASIC: Direktiven, Dateien und Assets

Eine **Direktive** ist eine Anweisung an den Übersetzer, keine an das Gerät. Sie steht in einer eigenen Zeile und beginnt mit `#`. Im nativen BASIC gibt es sieben: `#include`, `#longnames`, `#define`, `#if` mit `#else` und `#endif`, `#numbering`, `#dialect` und `#asm`. Die Seite erklärt sie, dazu das Einbinden von Sprites, Zeichensätzen und Songs aus den Editoren, Maschinencode im BASIC-Programm und das, was es **nicht** gibt. Sie gehört zum [BASIC-Handbuch](help:basic).

## Übersicht

| Zeile | Wirkung |
|---|---|
| `#include "datei.bas"` | Fügt den Inhalt der Datei an dieser Stelle ein |
| `#include "graphics/hero.vcsprite"` | Bindet eine Mediendatei (Sprite, Zeichensatz, Bildschirm, Song) als DATA-Zeilen ein |
| `#longnames` | Erlaubt lange Variablennamen (siehe [Lange Namen](help:basic-names)) |
| `#define NAME wert` | Konstante: ersetzt `NAME` im Programm durch `wert` |
| `#if SYSTEM_C64` … `#else` … `#endif` | Übersetzt Zeilen nur für bestimmte Systeme oder Bedingungen |
| `#numbering start=N step=N` | Start und Schrittweite der Zeilennummern dieser Datei |
| `#dialect id` | BASIC-Dialekt dieser Datei (siehe [Dialekte](help:basic-dialects)) |
| `#asm "datei.asm"` | Übersetzt eine Assembler-Datei und legt den Code zum Programm |
| `; text` | Kommentar für dich, kommt nicht ins Programm |

Direktiven schreibt man klein oder groß (`#INCLUDE` geht auch), sie stehen allein in der Zeile und haben **keine Zeilennummer**. Sie kosten weder Speicher noch Laufzeit. Eine Zeile, die mit `#` beginnt, aber keine dieser Direktiven ist, erkennt der Übersetzer nicht; er behandelt sie als gewöhnliche BASIC-Zeile.

### Konstanten: `#define`

```
#define NAME wert
```

Von der Zeile an ersetzt der Übersetzer jedes **ganze Wort** `NAME` durch `wert`, bevor er nummeriert. Das kostet weder eine Variable noch Laufzeit.

- Groß- und Kleinschreibung zählen: `RAHMEN` und `rahmen` sind zwei Namen.
- Nicht ersetzt wird in Zeichenketten, hinter `REM`, in `DATA`, hinter `@` (Marken) und in Zahlen wie `$FF`.
- Der Wert darf andere Namen enthalten, höchstens acht Stufen tief. Verweist ein Name auf sich selbst, meldet der Übersetzer **B176**.
- Es gibt **keine Makros mit Parametern**.
- Ein Schlüsselwort des Dialekts als Name meldet **B172**; ein Name aus anderen Zeichen als Buchstaben, Ziffern und `_` sowie ein zweites `#define` mit demselben Namen melden **B174**.
- Auf der Kommandozeile setzt `-D NAME=wert` eine Konstante, als stünde `#define NAME wert` am Anfang.

### Bedingte Übersetzung: `#if`

```
#if SYSTEM_C64
poke 53280, 0
#else
print "nur für den C64"
#endif
```

- Die Bedingung ist `SYSTEM_` plus die Kennung des Zielsystems in Großbuchstaben (`SYSTEM_C64`, `SYSTEM_PLUS4`, `SYSTEM_ATARI8` …), ein mit `#define` festgelegter Name (wahr, wenn sein Wert nicht `0` ist) oder eines von beiden mit vorangestelltem `!` (nicht).
- `#else` ist wahlfrei, `#endif` schließt ab. Verschachteln ist erlaubt.
- Ausgeschlossene Zeilen kommen nicht ins Programm. Meldungen zeigen trotzdem auf die richtige Zeile der Quelle.
- Fehler im Aufbau (`#else` oder `#endif` ohne `#if`, zweites `#else`, `#if` ohne `#endif`, nicht lesbare Bedingung) melden **B175**.

Die Befehle der Dialekte unterscheiden sich. Ein `#if` hilft darum vor allem dort, wo sich nur einzelne Zeilen je System unterscheiden. Für größere Unterschiede legst du je System eine Hauptdatei an (`main_c64.bas`, `main_plus4.bas`), jede in ihrer eigenen Quellgruppe im Projekt, und bindest gemeinsame Teile mit `#include` ein.

### Nummerierung und Dialekt je Datei

- `#numbering start=N step=N` legt fest, mit welcher Nummer die Zeilen ohne eigene Nummer beginnen und in welchem Abstand sie folgen (Standard 10 und 10). Die Zeile in der Datei geht den Werten des Projekts und der Kommandozeile (`--start`, `--step`) vor. Wer nur `start=` oder nur `step=` schreibt, ändert nur diesen Wert. Eine fehlerhafte Zeile meldet **B190**, eine Schrittweite unter 1 oder ein Start über der höchsten Nummer des Dialekts **B191**, und reichen die Nummern nicht für alle Zeilen, nennt **B192** eine kleinere Schrittweite.
- `#dialect id` stellt den Dialekt dieser einen Datei ein, zum Beispiel `#dialect turbo_basic_xl` für eine Datei mit Turbo BASIC XL in einem Atari-Projekt. Die Zeile in der Datei geht der Einstellung des Projekts vor. Eine unbekannte Kennung meldet **B178**, eine Kennung, die nicht zum Zielsystem passt, **B170**.

## Einbinden: `#include`

```
#include "pfad/datei.bas"
```

- Die Zeile wird beim Bauen **durch den Inhalt der Datei ersetzt**, als hättest du ihn dort getippt. Marken, Zeilennummern und `#longnames` der Datei gelten für das ganze Programm.
- Der **Pfad** steht in Anführungszeichen und gilt **relativ zur Datei, die die Zeile enthält**. Eingebundene Dateien dürfen ihrerseits weitere einbinden, bis zu 8 Ebenen tief.
- Die Zeile darf keine Zeilennummer tragen und muss allein stehen.
- Zeilen der eingebundenen Datei ohne Nummer werden **mitnummeriert**, als ständen sie an der Stelle der `#include`-Zeile. Mit Nummer bleiben sie, wie sie sind.
- Fehler zeigen auf die **ursprüngliche Datei und Zeile**, nicht auf die zusammengesetzte.

Beispiel mit zwei Dateien (der Block mit `datei=` ist die Hilfsdatei, der zweite die Hauptdatei):

```basic datei=teile/ausgabe.bas
@zeig:
print "wert ="; w
return
```

```basic liste
w = 5
gosub @zeig
end
#include "teile/ausgabe.bas"
```

```text
10 w = 5
20 gosub 40
30 end
40 print "wert ="; w
50 return
```

Marken sind **für das ganze Programm eine einzige Namensliste**: `@zeig` aus der eingebundenen Datei ist auch in der Hauptdatei bekannt. Zwei Dateien dürfen deshalb nicht dieselbe Marke festlegen (B111).

### Fehler beim Einbinden

| Code | Meldung | Ursache |
|---|---|---|
| B120 | `#include "x": Datei nicht gefunden …` | Pfad falsch, oder die Datei entsteht erst beim Bauen (alter Exportpfad, siehe unten) |
| B121 | `#include "x": zu tief verschachtelt oder zyklisch` | Eine Datei bindet sich (über andere) selbst ein, oder mehr als 8 Ebenen |

```basic fehler
print "a"
#include "gibtsnicht.bas"
```

```text
main.bas:2: error B120: #include "gibtsnicht.bas": Datei nicht gefunden (bei exportierten Dateien: erst bauen bzw. exportieren)
```

## Medien aus den Editoren einbinden

Sprites, Zeichensätze, Bildschirme und Songs entstehen in den Editoren (siehe [Sprite-Editor](help:sprite), [Zeichensatz-Editor](help:charset), [Bildschirm-Editor](help:screen)). Für BASIC gibt es sie als **DATA-Zeilen**. Du bindest die **Mediendatei selbst** mit `#include` ein und liest sie mit `READ`:

```
#include "graphics/hero.vcsprite"
```

Du musst nichts exportieren: Beim Bauen wandelt VintageCoder das Medium selbst in DATA-Zeilen um und setzt sie an die Stelle der `#include`-Zeile. Das Ergebnis liegt zur Beschleunigung im Zwischenspeicher `build/.media-cache/` (eine lesbare Kopie heißt dort `<name>.data.bas`); du kannst den Ordner jederzeit löschen. Die Mediendatei bleibt unverändert, auch wenn du das Programm in Labels oder Nummern umwandelst (siehe [Werkzeuge](help:basic-tools)).

- Ein Medium wird je Programm nur **einmal** eingebunden. Ein zweites `#include` desselben Mediums überspringt der Übersetzer ohne Meldung.
- Das Medium muss zum Zielsystem des Baus passen. Sonst, oder wenn die Datei fehlt oder kein Medium ist, meldet der Bau **B193** mit dem Grund.
- Mehrere Medien nebeneinander: Jedes bekommt seine Marke `@<name>:`. Wie du sie liest, hängt vom Dialekt ab (siehe unten).
- **Zeilennummer als Konstante:** Zu jeder Marke `<name>` gibt es eine Konstante `<name>_line`, die beim Nummerieren durch die Zeilennummer der Marke ersetzt wird. Sie gilt nur, wenn mindestens ein Medium eingebunden ist, und nicht in Zeichenketten, `REM` und `DATA`. Bei Medien aus mehreren Teilen heißt sie `<medium>_<teil>_line` (zum Beispiel `font_chars_line`).
- Im reinen Nummernmodus muss die Zeile, die auf das Medium folgt, eine größere Nummer tragen als die letzte Zeile des Mediums; sonst meldet der Bau **B100**.
- Ist das Programm samt Medien größer als der freie Speicher, warnt **B195** (geprüft für C64: 38911 Byte, VC-20 ohne Erweiterung: 3583 Byte, Plus/4: 60671 Byte; andere Systeme sind nicht geprüft). Dann hilft Maschinencode mit `#asm` oder weniger DATA-Zeilen.

Den `#include`-Eintrag musst du nicht tippen. Im Export-Dialog (Cmd/Strg+Umschalt+E) fügt **Als Include einfügen** die Zeile in den geöffneten Quelltext ein. Ist das Format dort **BASIC DATA**, kopiert **In Quelltext kopieren** die Daten samt einer Einleseroutine in den Quelltext.

*[Bild folgt in E13: basic-medium-include]*

### Der alte Weg über `build/export`

Früher hat der Bau jedes Medium nach `build/export/<name>.data.bas` exportiert, und du hast diese Datei eingebunden. Das geht noch, ist aber veraltet: Der Bau warnt mit **B194** und nennt die Mediendatei, die du stattdessen einbinden sollst. Die Kommandozeile stellt alte Projekte um: `vcc migrate includes` zeigt, was sich ändern würde, und `vcc migrate includes --apply` schreibt es.

### Aufbau der erzeugten DATA-Zeilen

- Der Text beginnt mit einer Kommentarzeile `REM erzeugt von VintageCoder: n Bytes` und der Marke `@<name>:`. Der Name ist der Name des Mediums (`hero`, `font`, `title`).
- Danach folgen `DATA`-Zeilen mit je 12 Werten von 0 bis 255.
- Besteht ein Medium aus mehreren Teilen, hat jeder Teil seine eigene Marke: Bildschirme `@<name>_chars:` und `@<name>_attrs:` (Zeichen und Farben), Songs `@<name>_order:`, `@<name>_patterns:`, `@<name>_ins:`.
- Das Einbinden macht daraus normale Programmzeilen: Sie sind nummeriert wie alle anderen, die Marken gelten im ganzen Programm.

### Sprite einlesen

Auf dem C64 gehört ein Sprite 64 Byte in einen Speicherblock. Der Block 13 liegt bei 832 (`$0340`), der Zeiger dafür bei 2040:

```basic ausgabe
#include "graphics/hero.vcsprite"
for i = 0 to 63 : read a : poke 832 + i, a : next i
poke 2040, 13
poke 53269, 1
poke 53248, 100 : poke 53249, 100
poke 53287, 1
@ende:
goto @ende
```

```text
hero                     Zeile 20
ende                     Zeile 180
```

Die Mediendatei `hero.vcsprite` stammt hier aus dem Beispiel `examples/c64/sprite_demo`.

### Die Reihenfolge der DATA-Zeilen: C64 und VC-20

Das BASIC V2 liest `DATA` **streng der Reihe nach** durch das ganze Programm. Es hat kein `RESTORE` mit Zeilennummer. Auf dem C64 und VC-20 bindest du darum die Dateien **in der Reihenfolge ein, in der du sie liest**:

```basic
#include "graphics/hero.vcsprite"
#include "graphics/font.vcchar"
for i = 0 to 63 : read a : poke 832 + i, a : next i
for i = 0 to 2047 : read a : poke 12288 + i, a : next i
```

`restore` ohne Zeile setzt den Lesezeiger auf das **erste** `DATA` zurück. Mehr braucht es bei einem einzigen Asset nicht.

### Auf C128, Plus/4, MEGA65 und Atari: `restore @name`

Diese Dialekte kennen `RESTORE zeile`, also geht auch die Marke. Die Reihenfolge der Dateien ist dann egal:

```basic ausgabe system=c128
#include "graphics/hero.vcsprite"
#include "graphics/font.vcchar"
restore @font
for i = 0 to 7 : read a : print a; : next i
restore @hero
read a
```

```text
font
hero
```

Auch Atari BASIC kennt `restore` mit Zeile, also geht dort `restore @name` ebenso:

```basic system=atari8
restore @werte
read a
print a
end
@werte:
data 1, 2, 3
```

Wohin du die Werte schreibst (Sprite-Block, Zeichensatz), hängt vom System ab und steht in der Dokumentation des Zielsystems.

## Maschinencode im BASIC-Programm: `#asm`

Mit `#asm` legst du Maschinencode zum BASIC-Programm. Die Zeile nennt eine Assembler-Datei des Projekts (früher hieß die Direktive `#blob`; der alte Name gilt nicht mehr und meldet **B177** mit dem Hinweis auf `#asm`):

```
#asm "player.asm"
#asm "player.asm" with "music/tune.vcsong"
```

Beim Bauen übersetzt VintageCoder die Datei mit dem Assembler und legt den Code so ab:

- **Commodore (C64, VC-20, C128, Plus/4):** hinter den BASIC-Text im Programm. Seine Adresse hängt von der Länge des Programms ab; der Bau rechnet sie aus und wiederholt das, bis sie stabil ist (höchstens acht Durchgänge, sonst **B177**).
- **MEGA65 und Atari:** als **DATA-Zeilen** ab der Marke `@asm_data`. Das Programm lädt sie selbst an die feste Adresse (`RESTORE @asm_data`, dann lesen). Auf dem Atari liegt der Code unter `$A000` an einer Seitengrenze.

Im Programmtext nennst du die Marken des Assembler-Codes mit dem Vorsatz `asm.`, zum Beispiel `SYS asm.start`. Der Bau setzt die Dezimaladresse ein (nicht in Zeichenketten, hinter `REM` und in `DATA`). Dazu gibt es `asm.base_addr` (Anfang des Codes) und `asm.base_page` (die Seite, zum Beispiel für `POKE 106` auf dem Atari).

```
#asm "player.asm"
sys asm.start
```

Mit `with` gibst du weitere Dateien mit, zum Beispiel einen Song; die Namen gehen unverändert an den Assembler, ein Medium löst er selbst auf (siehe [Direktiven des Assemblers](help:asm-directives)).

Fehler: **B177** meldet eine falsche Schreibweise (erwartet wird `#asm "datei.asm" [with "song.asm"]`), eine Marke `asm.name`, die der Code nicht festlegt, ein zu großes Programm samt Code (es endet über dem Speicher) und Code, der auf dem Atari nicht unter `$A000` passt. **B173** meldet, dass die Zeilennummern im Nummernmodus nicht für die DATA-Zeilen des Codes reichen. **B195** warnt, wenn das Programm nicht in den freien Speicher passt.

![Der Code-Editor mit der BASIC-Datei test.bas, die Zeilen mit #define, #if, #else und #asm mit der Datei code.asm enthält. Über dem Text steht eine Hilfekarte zu „#asm datei.asm“, die erklärt, dass Maschinencode eingebunden wird und asm.name die Adresse einer Marke ergibt.](bild:basic-asm-direktive "Die Hilfekarte zur Direktive #asm im BASIC-Editor.")

Daneben gibt es zwei ältere Wege:

1. **`DATA` + `POKE` + `SYS`.** Die Bytes des Codes stehen als `DATA`, das Programm schreibt sie an eine freie Adresse und ruft sie auf. Ein Beispiel für den C64: Der Code `lda #0 : sta $d020 : rts` besteht aus den Bytes `A9 00 8D 20 D0 60` (169, 0, 141, 32, 208, 96) und liegt ab 49152 (`$C000`):

```basic liste
for i = 0 to 5 : read b : poke 49152 + i, b : next i
sys 49152
data 169, 0, 141, 32, 208, 96
```

```text
10 for i = 0 to 5 : read b : poke 49152 + i, b : next i
20 sys 49152
30 data 169, 0, 141, 32, 208, 96
```

   Die Bytes lassen sich mit dem Assembler holen (`vcc asm code.asm --format bin`) und in `DATA`-Zeilen übertragen. Auf dem Atari ruft man mit `USR(adresse)` auf.

2. **Zwei Programme.** Der Maschinencode kommt als eigenes Programm (Assembler-Quelle im Projekt) auf dieselbe Diskette; das BASIC-Programm lädt ihn mit `LOAD "name",8,1` und startet ihn mit `SYS`. Siehe [Disketten](help:disks).

Und umgekehrt: Das Assembler-Programm trägt selbst eine BASIC-Zeile `SYS` (`.basicstub`, siehe [Direktiven des Assemblers](help:asm-directives)), und BASIC-Teile werden mit [basic2asm](help:basic-tools) in Assembler umgewandelt.

Achte bei den älteren Wegen darauf, dass der Code **nicht** im BASIC-Programmbereich oder in den Variablen liegt (C64: `$0801` bis zum Ende des Programms plus Variablen). Freie Bereiche auf dem C64 sind z. B. `$C000`–`$CFFF` oder der Kassettenpuffer ab `$033C` (828).

## Kommentare und leere Zeilen

- `; text` am **Zeilenanfang** ist ein Kommentar, der nicht ins Programm kommt. Ein `;` mitten in der Zeile ist kein Kommentar (in `PRINT` ist es das Anhängezeichen).
- `rem text` bleibt im Programm und belegt Speicher. Der Übersetzer lässt dabei den Text unverändert, `rem` darf also auch Sonderzeichen enthalten.
- Leere Zeilen werden übersprungen.
- Der Hochstrich `'` ist nur in Turbo BASIC XL ein Kommentarzeichen (siehe [Dialekte](help:basic-dialects)).
