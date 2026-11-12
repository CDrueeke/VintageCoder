---
id: assembler
titel: Assembler (VCASM) für Einsteiger
reihenfolge: 6
stufe: Einsteiger
zeit: 40 Minuten
siehe_auch: [editor, basic, bauen-starten, debugger]
referenz: [assembler, asm-directives, asm-macros, asm-library, asm-index]
---

# Assembler (VCASM) für Einsteiger

**Ziel:** Du schreibst dein erstes eigenes Maschinenprogramm. Du verstehst, aus welchen Teilen eine Assembler-Zeile besteht, benutzt Labels, Konstanten, Datentabellen, ein Makro und die Systembibliothek und kannst eine Fehlermeldung des Assemblers lesen.

## Das brauchst du

- Ein C64-Projekt mit Assembler-Vorlage (siehe [Dein erstes Projekt](kapitel:erstes-projekt)). Du ersetzt den Inhalt von `src/c64/main.asm` nach und nach durch die Beispiele in diesem Kapitel.
- Keine Vorkenntnisse im Assembler. Du brauchst nur die Bereitschaft, ein paar Dinge über den Prozessor zu lernen. Sie stehen im nächsten Abschnitt.

## Was ist ein Assembler?

Ein Prozessor versteht nur Zahlen. Ein **Maschinenprogramm** ist eine lange Folge solcher Zahlen. Weil niemand Zahlenreihen tippen möchte, schreibt man **Assembler**: lesbare Kürzel wie `lda #2`, die der **Assembler** (das Übersetzerprogramm) in die Zahlen verwandelt. VintageCoder hat dafür einen eigenen Assembler namens **VCASM**. Er beherrscht die 6502-Familie: den 6510 des C64, den 8502 des C128, den 7501 des Plus/4, den „Sally“ des Atari und den 45GS02 des MEGA65.

Drei Dinge musst du über den Prozessor wissen:

- **Register:** Der Prozessor hat nur wenige eigene Merkzellen mit je einem Byte (Zahl von 0 bis 255). Die drei wichtigsten heißen **A** (Akkumulator, das Arbeitsregister), **X** und **Y** (Indexregister, gern als Zähler benutzt).
- **Speicher:** Der Rechner hat 65536 Speicherzellen (Bytes), jede mit einer **Adresse**. Adressen schreibt man **hexadezimal** (Zahlen zur Basis 16, Ziffern 0–9 und a–f) mit einem `$` davor: `$d020` ist die Adresse der Rahmenfarbe des C64. Ein `$` vor einer Zahl heißt also „hexadezimal“, `%` bedeutet binär, nichts davor bedeutet dezimal.
- **Chips:** Bild, Ton und Eingabe steuerst du, indem du Zahlen an bestimmte Adressen schreibst. Das Grundmuster ist: `lda #zahl` (Zahl in A laden), dann `sta adresse` (A an die Adresse speichern).

## Mein erstes Programm

Im Kapitel [Dein erstes Projekt](kapitel:erstes-projekt) hast du ein Programm gebaut, das den Rahmen rot färbt. Jetzt erweitern wir es um einen Gruß auf dem Bildschirm.

1. Öffne `src/c64/main.asm` und ersetze den Inhalt durch:

```asm
; Mein erstes Programm: roter Rahmen und ein Gruß auf dem Bildschirm
.target c64
.basicstub start            ; BASIC-Zeile "10 SYS 2061", damit RUN das Programm startet

start:  lda #2              ; 2 = Farbe Rot
        sta $d020           ; Rahmenfarbe setzen
        ldx #0
loop:   lda text,x          ; ein Zeichen holen ...
        sta $0400,x         ; ... und in den Bildschirmspeicher schreiben
        inx
        cpx #5              ; fünf Zeichen kopiert?
        bne loop            ; nein: weiter
        rts                 ; zurück zu BASIC

text:   .screen "hallo"     ; Text als Bildschirmcodes
```

2. Drücke **F5**. Der Emulator startet, der Rahmen wird rot, und oben links steht „HALLO“.

![Der Quelltext im Editor und rechts daneben der C64-Bildschirm mit rotem Rahmen und dem Wort HALLO in der oberen linken Ecke](bild:asm-erstes-programm "Das erste Programm: roter Rahmen und ein Gruß")

Zeile für Zeile:

| Zeile | Bedeutung |
|---|---|
| `; …` | **Kommentar** bis zum Zeilenende. Der Assembler ignoriert ihn. Auch `//` geht. |
| `.target c64` | Dieses Programm ist für den C64. Die IDE erkennt daran das Zielsystem. |
| `.basicstub start` | legt die BASIC-Zeile `10 SYS 2061` an. Der C64 führt sie bei `RUN` aus und springt dadurch zu `start`. Der Stub beginnt automatisch an der BASIC-Startadresse `$0801`. |
| `start:` | ein **Label**: ein Name für die Adresse an dieser Stelle. |
| `lda #2` | lade die Zahl 2 in A. Das `#` heißt: „die Zahl selbst“, nicht „den Inhalt der Adresse 2“. |
| `sta $d020` | speichere A an Adresse `$d020`, dem Register der Rahmenfarbe. |
| `ldx #0` | lade 0 in X. |
| `lda text,x` | lade das Byte an der Adresse `text + X`. |
| `sta $0400,x` | schreibe es an `$0400 + X`. Dort beginnt der Bildschirmspeicher. |
| `inx` | erhöhe X um 1. |
| `cpx #5` | vergleiche X mit 5. |
| `bne loop` | wenn X ungleich 5 ist, springe zurück zu `loop`. |
| `rts` | beende das Programm und kehre zu BASIC zurück. |
| `.screen "hallo"` | legt die fünf Buchstaben als **Bildschirmcodes** ab. Das sind die Zahlen, die der Bildschirmspeicher erwartet. |

> **Info:** `.screen` und `.petscii` wandeln Text in die passenden Zeichencodes um. Der Bildschirm des C64 kennt andere Codes als die Tastatur (PETSCII). Näheres steht in der Hilfeseite [Texte und Zeichensätze](hilfe:asm-text).

### Das Listing lesen

Beim Bauen entstehen im Ordner `build/` mehrere Dateien. Interessant ist das **Listing** (`.lst`): links die Adresse, dann die erzeugten Bytes, rechts dein Text.

```
080d  a9 02             5  start:  lda #2
080f  8d 20 d0          6          sta $d020
0812  a2 00             7          ldx #0
0814  bd 20 08          8  loop:   lda text,x
```

Du siehst, dass `lda #2` zu den zwei Bytes `a9 02` wurde. `sta $d020` wurde zu `8d 20 d0`: der Befehl `8d`, danach die Adresse mit dem niedrigen Byte zuerst. Und `text` wurde durch die Adresse `$0820` ersetzt, die der Assembler selbst ausgerechnet hat. Wie du das Listing im Editor öffnest, steht im Kapitel [Bauen und Starten](kapitel:bauen-starten).

## Aufbau einer Zeile

Eine Zeile besteht aus höchstens vier Teilen: **Label**, **Befehl** (oder **Direktive**), **Operand** und **Kommentar**.

```asm
.target c64
.basicstub start

start:  lda #$00        ; Label mit Doppelpunkt
        sta vic2.border ; Chip-Register als Namen
.loop:  dex             ; lokales Label gilt bis zum nächsten globalen
        bne .loop
        rts
```

- **Label:** ein Name mit **Doppelpunkt** am Zeilenanfang. Ohne Doppelpunkt hält der Assembler das Wort für einen Befehl und meldet einen Fehler.
- **Befehl:** ein Kürzel des Prozessors (`lda`, `sta`, `jmp` …). Groß- oder Kleinschreibung ist egal.
- **Direktive:** beginnt mit einem Punkt und ist eine Anweisung **an den Assembler**, kein Befehl für den Prozessor (zum Beispiel `.target`, `.org`, `.byte`).
- **Kommentar:** `;` oder `//` bis zum Zeilenende.
- Auf einer Zeile steht nur **ein** Befehl.
- Chip-Register stehen als `chip.register` zur Verfügung, zum Beispiel `vic2.border` statt `$d020`. Mit **F1** auf dem Wort bekommst du die Beschreibung (siehe [Der Code-Editor](kapitel:editor)).

### Labels

Ein Label ist ein Name für eine Adresse. Statt `jmp $c012` schreibst du `jmp start`.

- Ein **globales** Label (`start:`) gibt es nur einmal im Programm.
- Ein **lokales** Label beginnt mit einem Punkt (`.loop:`) und gehört zum letzten globalen Label davor. So darf jede Routine ihre eigene `.loop` haben, ohne dass sie sich in die Quere kommen.
- **Anonyme** Labels sind ein einzelnes `-` oder `+` am Zeilenanfang. Im Operanden springt `-` zum vorigen `-`, `+` zum nächsten `+`:

```asm
.target c64
.basicstub start

start:  ldx #3
-       dex
        bne -               ; zurück zum "-" darüber
        rts
```

Welche Art nimmst du? Anonyme Labels für Sprünge über zwei, drei Zeilen. Lokale Labels für Schleifen innerhalb einer Routine. Globale Labels für Einsprungpunkte, die von anderswo angesprungen werden.

## Adressierung kurz erklärt

Beim Operanden kommt es auf die Schreibweise an:

| Schreibweise | Name | Bedeutung |
|---|---|---|
| `#$12` | unmittelbar | die Zahl selbst |
| `$12` | Zeropage | Adresse in den ersten 256 Bytes (kürzer und schneller) |
| `$1234` | absolut | beliebige 16-Bit-Adresse |
| `$1234,x` und `$1234,y` | indiziert | Adresse plus Register X oder Y |

Weitere Arten stehen im Spickzettel der Hilfe [Assembler](hilfe:assembler). Du brauchst sie erst später.

## Konstanten und Datentabellen

Mit `.const` gibst du einer Zahl einen Namen. So steht die Bedeutung im Quelltext und nicht nur die Zahl:

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

Mit `.byte` legst du eine **Tabelle** fester Zahlen im Programm ab. Dieses Programm holt vier Farben nacheinander aus einer Tabelle und färbt den Rahmen damit:

```asm
; Rahmenfarbe nacheinander aus einer Tabelle holen (nur das letzte Ergebnis bleibt sichtbar)
.target c64
.basicstub start

start:  ldx #0
.next:  lda farben,x        ; Farbe Nr. X aus der Tabelle
        sta $d020
        inx
        cpx #4              ; vier Farben durch?
        bne .next
        rts

farben: .byte 2, 5, 6, 7    ; Rot, Grün, Blau, Gelb
```

> **Info:** Der Prozessor ist so schnell, dass du die ersten drei Farben nicht siehst. Nur die letzte (Gelb) bleibt sichtbar. Der Sinn des Beispiels ist das Muster: ein Zähler in X und eine Tabelle, die mit `lda tabelle,x` gelesen wird.

Die wichtigsten Direktiven auf einen Blick:

| Direktive | Wirkung |
|---|---|
| `.target c64` | Zielsystem des Programms |
| `.org $0801` | die folgenden Bytes beginnen bei dieser Adresse |
| `.basicstub label` | BASIC-Zeile `SYS …`, damit `RUN` das Programm startet |
| `.const NAME = wert` | benennt eine Zahl |
| `.byte a, b, c` | legt einzelne Bytes ab |
| `.word a, b` | legt 16-Bit-Zahlen ab (niedriges Byte zuerst) |
| `.screen "text"` | Text als Bildschirmcodes |
| `.petscii "text"` | Text als PETSCII (mit Steuerzeichen wie `{clr}`) |
| `.include "datei.asm"` | bindet eine Quelldatei ein |
| `.incbin "datei.bin"` | bindet eine Binärdatei ein |

> **Tipp:** Die Zeile musst du nicht tippen: Ziehe die Datei aus dem Projektbaum in den Quelltext oder wähle im Projektbaum **Als Include einfügen**. VintageCoder wählt `.include` oder `.incbin` und den Pfad (siehe [Der Code-Editor](kapitel:editor)).

Alle Direktiven stehen in den Hilfeseiten [Direktiven](hilfe:asm-directives) und [Spickzettel und Fehlercodes](hilfe:asm-index).

## Makros: Abkürzungen für mehrere Zeilen

Ein **Makro** ist eine benannte Abkürzung für mehrere Zeilen. Der Assembler setzt bei jedem Aufruf den Text des Makros ein, als hättest du ihn selbst getippt:

```asm
; Ein Makro setzt Rahmen- und Hintergrundfarbe
.target c64
.basicstub start

.macro farbe(c)
        lda #c
        sta $d020
        sta $d021
.endmacro

start:  farbe(0)
        farbe(6)
        rts
```

Das Makro heißt `farbe` und hat einen **Parameter** `c`. Beim Aufruf `farbe(6)` wird überall `c` durch `6` ersetzt. Das Listing zeigt, was daraus wird: bei jedem Aufruf stehen die drei Befehle neu im Programm.

![Der Quelltext mit dem Makro farbe und rechts das Listing, das zeigt, wie jeder Aufruf zu drei Befehlen wird](bild:asm-makro "Ein Makro wird bei jedem Aufruf eingesetzt")

> **Achtung:** Ein Makro ist **kein Unterprogramm**. Es gibt weder `jsr` noch `rts` dafür: Der Code steht bei jedem Aufruf neu im Programm. Das ist schnell, kostet aber Speicher. Mehr über Makros, Scopes und Strukturen steht in der Hilfeseite [Labels, Makros, Scopes, Strukturen](hilfe:asm-macros).

## Die Systembibliothek: Namen statt Adressen

Statt Adressen wie `$d020` zu merken, bindest du fertige Namenslisten ein. Mit `#include <pfad>` holst du sie aus der **Systembibliothek** der IDE (Ordner `data/sys/`):

```asm
; Systembibliothek: Namen statt Adressen
.target c64
#include <c64/hardware.asm>
#include <c64/kernal.asm>
.basicstub start

start:  lda #COL_GREEN
        sta VIC_BORDERCOLOR     ; Rahmen grün
        ldx #0
.next:  lda text,x              ; nächstes Zeichen
        beq .done               ; 0 = Ende des Textes
        jsr CHROUT              ; KERNAL: Zeichen ausgeben
        inx
        bne .next
.done:  rts

text:   .null "{clr}hallo welt{ret}"
```

Hier stammen `COL_GREEN` und `VIC_BORDERCOLOR` aus `c64/hardware.asm`. `CHROUT` ist eine Routine des C64-Betriebssystems (**KERNAL**), die ein Zeichen auf dem Bildschirm ausgibt; ihr Name kommt aus `c64/kernal.asm`. `jsr` ruft ein Unterprogramm auf. `.null` legt einen Text ab und hängt ein Null-Byte als Endmarke an. Die Schleife liest den Text, bis sie auf diese Null trifft.

Für andere Systeme gibt es entsprechende Dateien, zum Beispiel `atari8bit/antic.asm` oder `mega65/hardware.asm`. Die Liste aller Dateien steht in der Hilfeseite [Systembibliothek](hilfe:asm-library).

## Fehlermeldungen lesen

Der Assembler meldet Fehler im Reiter **Probleme** und im Editor als rot unterstrichene Zeile (siehe [Der Code-Editor](kapitel:editor)). Eine Meldung hat immer die gleiche Form: Fehlercode, Beschreibung, Datei und Zeile, dazu die betroffene Zeile. Drei häufige:

**Unbekanntes Symbol** (E202): Du benutzt ein Label, das es nicht gibt.

```asm
.target c64
.basicstub start
start:  lda #2
        sta $d020
        jmp ende
```

```text
error[E202]: unbekanntes Symbol 'ende' in 'ende'
  --> fehler.asm:5
```

Lösung: Das Label `ende:` fehlt oder ist falsch geschrieben. Achte auch auf Groß- und Kleinschreibung, bei Labels zählt sie.

**Unbekannter Befehl** (E205): Das Wort ist kein Befehl. Hast du einen Tippfehler gemacht (`rtz` statt `rts`) oder bei einem Label den **Doppelpunkt vergessen** (`loop jmp loop`)? Dann hält der Assembler `loop` für einen Befehl:

```text
error[E205]: unbekannter Befehl 'LOOP'
```

**Wert außerhalb** (E203): Ein Byte passt nur von 0 bis 255.

```asm
start:  lda #300
```

```text
error[E203]: Wert 300 außerhalb -128..255
```

![Der Editor mit einer rot unterstrichenen Zeile „jmp ende“, daneben der Reiter Probleme mit der Meldung „E202: unbekanntes Symbol ende“](bild:asm-fehler "Eine Fehlermeldung lesen: Code, Beschreibung und Zeile")

Ein Klick auf die Meldung im Reiter **Probleme** springt zur Zeile. Alle Fehlercodes erklärt der [Spickzettel](hilfe:asm-index).

## Häufige Stolpersteine

- **Ein Label ohne Doppelpunkt.** `loop jmp loop` geht nicht; es muss `loop: jmp loop` heißen (E205).
- **`#` vergessen.** `lda 2` lädt den **Inhalt der Adresse 2**, nicht die Zahl 2. Für die Zahl selbst schreibst du `lda #2`.
- **Groß-/Kleinschreibung bei Labels.** `Start` und `start` sind verschiedene Namen. Befehle und Direktiven sind davon frei.
- **Mehrere Befehle in einer Zeile.** Das geht nicht; jede Zeile hat genau einen Befehl.
- **Das Programm kehrt nicht zurück.** Ohne `rts` am Ende läuft der Prozessor in den nächsten Speicher und stürzt ab. Eine Endlosschleife wie `ende: jmp ende` hält das Programm kontrolliert an.
- **Text erscheint als Kauderwelsch.** Der Bildschirmspeicher erwartet Bildschirmcodes (`.screen`), die Ausgabe über `CHROUT` erwartet PETSCII (`.petscii` oder `.null`).
- **Der Rahmen bleibt unverändert.** Prüfe, ob das Programm wirklich läuft: Tippe im Emulator `RUN`, falls der C64 bei „READY.“ steht.

## Wie geht es weiter?

Du kennst jetzt die Bausteine eines Assembler-Programms. Weiter geht es mit dem anderen Weg zu einem Programm, [BASIC in der IDE](kapitel:basic). Wie du das Ergebnis baust, im Bauprotokoll liest und startest, steht im Kapitel [Bauen und Starten](kapitel:bauen-starten). Wenn dein Programm nicht tut, was du willst, hilft der [Debugger](kapitel:debugger). Zum Nachschlagen nutzt du die Hilfeseiten [Assembler](hilfe:assembler) und [Direktiven](hilfe:asm-directives).
