# BASIC: Direktiven, Dateien und Assets

Eine **Direktive** ist eine Anweisung an den Übersetzer, keine an das Gerät. Sie steht in einer eigenen Zeile und beginnt mit `#`. Im nativen BASIC gibt es genau zwei: `#include` und `#longnames`. Die Seite erklärt sie, dazu das Einbinden von Sprites, Zeichensätzen und Songs aus den Studios, Maschinencode im BASIC-Programm und das, was es **nicht** gibt. Sie gehört zum [BASIC-Handbuch](help:basic).

## Übersicht

| Zeile | Wirkung |
|---|---|
| `#include "datei.bas"` | Fügt den Inhalt der Datei an dieser Stelle ein |
| `#longnames` | Erlaubt lange Variablennamen (siehe [Lange Namen](help:basic-names)) |
| `; text` | Kommentar für dich, kommt nicht ins Programm |

Direktiven schreibt man klein oder groß (`#INCLUDE` geht auch), sie stehen allein in der Zeile und haben **keine Zeilennummer**. Sie kosten weder Speicher noch Laufzeit.

### Was es nicht gibt

Dies kommt oft vor und ist wichtig: Im nativen BASIC gibt es **keine bedingte Übersetzung** (`#if`, `#else`, `#endif`), keine Konstanten oder Makros (`#define`, `const`) und keinen Platzhalter `{system}` im Include-Pfad. Der Übersetzer erkennt solche Zeilen nicht als Direktiven. Er behandelt sie als gewöhnliche BASIC-Zeilen und baut Müll, ohne Fehler zu melden:

```basic liste
#if SYSTEM_C64
print "c64"
#endif
```

```text
10 #if system_c64
20 print "c64"
30 #endif
```

Das Gerät meldet dann zur Laufzeit einen Syntaxfehler. Schreibe also kein `#if` und kein `#define` in `.bas`-Dateien.

Für ein Programm, das auf mehreren Systemen laufen soll, bleibt der Weg über **mehrere Quelldateien**: je System eine Hauptdatei (`main_c64.bas`, `main_plus4.bas`), jede in ihrer eigenen Quellgruppe im Projekt (Zielsystem und Dialekt wählst du je Gruppe), und gemeinsame Teile in Dateien, die du mit `#include` einbindest. Das geht nur, soweit die Befehle des gemeinsamen Teils auf allen Systemen gleich sind; sonst braucht jede Hauptdatei ihren eigenen Teil.

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
| B120 | `#include "x": Datei nicht gefunden …` | Pfad falsch, oder die Datei entsteht erst beim Bauen (Export, siehe unten) |
| B121 | `#include "x": zu tief verschachtelt oder zyklisch` | Eine Datei bindet sich (über andere) selbst ein, oder mehr als 8 Ebenen |

```basic fehler
print "a"
#include "gibtsnicht.bas"
```

```text
main.bas:2: error B120: #include "gibtsnicht.bas": Datei nicht gefunden (bei exportierten Dateien: erst bauen bzw. exportieren)
```

## Assets aus den Studios einbinden

Sprites, Zeichensätze, Bildschirme und Songs entstehen in den Studios (siehe [Sprite-Editor](help:sprite), [Zeichensatz-Editor](help:charset), [Bildschirm-Editor](help:screen), [Tracker](help:tracker)). Für BASIC gibt es sie als **DATA-Zeilen**: Beim Bauen exportiert VintageCoder jedes Medium nach `build/export/<name>.data.bas`. Du bindest die Datei mit `#include` ein und liest sie mit `READ`.

```
#include "build/export/hero.data.bas"
```

Gibt es in der Quelle ein `#include`, prüft der Bau vorher, ob die Exporte aktuell sind, und erzeugt veraltete neu (Stufe „Medien exportieren“ im Bauprotokoll). Eine Datei, die erst durch diesen Export entsteht, kann also schon beim ersten Bauen eingebunden werden.

### Aufbau der Exportdatei

- Die Datei beginnt mit einer Kommentarzeile `REM erzeugt von VintageCoder: n Bytes` und der Marke `@<name>:`. Der Name ist der Name des Mediums (`hero`, `font`, `title`).
- Danach folgen `DATA`-Zeilen mit je 12 Werten von 0 bis 255.
- Besteht ein Medium aus mehreren Teilen, hat jeder Teil seine eigene Marke: Bildschirme `@<name>_chars:` und `@<name>_attrs:` (Zeichen und Farben), Songs `@<name>_order:`, `@<name>_patterns:`, `@<name>_ins:`.
- Das Einbinden macht aus der Datei normale Programmzeilen: Die Zeilen sind nummeriert wie alle anderen, die Marken sind im ganzen Programm gültig.

### Sprite einlesen

Auf dem C64 gehört ein Sprite 64 Byte in einen Speicherblock. Der Block 13 liegt bei 832 (`$0340`), der Zeiger dafür bei 2040:

```basic ausgabe
#include "build/export/hero.data.bas"
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

Die Datei `hero.data.bas` stammt hier aus dem Beispiel `examples/c64/sprite_demo`. Im Projektbaum ziehst du die Mediendatei in den Quelltext oder wählst im Asset-Editor „In Code einfügen“: Die `#include`-Zeile entsteht an der Cursorposition. Ein Doppelklick auf den Dateinamen in der Zeile öffnet das Asset.

### Die Reihenfolge der DATA-Zeilen: C64 und VC-20

Das BASIC V2 liest `DATA` **streng der Reihe nach** durch das ganze Programm. Es hat kein `RESTORE` mit Zeilennummer. Auf dem C64 und VC-20 bindest du darum die Dateien **in der Reihenfolge ein, in der du sie liest**:

```basic
#include "build/export/hero.data.bas"
#include "build/export/font.data.bas"
for i = 0 to 63 : read a : poke 832 + i, a : next i
for i = 0 to 2047 : read a : poke 12288 + i, a : next i
```

`restore` ohne Zeile setzt den Lesezeiger auf das **erste** `DATA` zurück. Mehr braucht es bei einem einzigen Asset nicht.

### Auf C128, Plus/4, MEGA65 und Atari: `restore @name`

Diese Dialekte kennen `RESTORE zeile`, also geht auch die Marke. Die Reihenfolge der Dateien ist dann egal:

```basic ausgabe system=c128
#include "build/export/hero.data.bas"
#include "build/export/font.data.bas"
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

## Maschinencode im BASIC-Programm

Natives BASIC hat keinen Befehl, der Maschinencode hinter den BASIC-Text legt. Es gibt drei Wege:

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
3. **Umgekehrt.** Das Assembler-Programm trägt selbst eine BASIC-Zeile `SYS` (`.basicstub`, siehe [Direktiven des Assemblers](help:asm-directives)), und BASIC-Teile werden mit [basic2asm](help:basic-tools) in Assembler umgewandelt.

Achte darauf, dass der Code **nicht** im BASIC-Programmbereich oder in den Variablen liegt (C64: `$0801` bis zum Ende des Programms plus Variablen). Freie Bereiche auf dem C64 sind z. B. `$C000`–`$CFFF` oder der Kassettenpuffer ab `$033C` (828).

## Kommentare und leere Zeilen

- `; text` am **Zeilenanfang** ist ein Kommentar, der nicht ins Programm kommt. Ein `;` mitten in der Zeile ist kein Kommentar (in `PRINT` ist es das Anhängezeichen).
- `rem text` bleibt im Programm und belegt Speicher. Der Übersetzer lässt dabei den Text unverändert, `rem` darf also auch Sonderzeichen enthalten.
- Leere Zeilen werden übersprungen.
- Der Hochstrich `'` ist nur in Turbo BASIC XL ein Kommentarzeichen (siehe [Dialekte](help:basic-dialects)).
