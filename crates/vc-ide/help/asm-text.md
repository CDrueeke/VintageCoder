# Assembler: Texte und Zeichensätze

Computer speichern Buchstaben als Zahlen. Welche Zahl zu welchem Buchstaben gehört, legt eine **Zeichenkodierung** fest – und die ist bei jedem Rechner anders. Ein „A“ ist beim PC `$41`, im C64-Bildschirmspeicher `$01`, und im C64-Betriebssystem gibt es zwei verschiedene Arten, ein „a“ zu schreiben. Der Assembler wandelt deshalb deinen Text beim Übersetzen in die Kodierung des Zielsystems um.

Die Seite gehört zum [Assembler-Handbuch](help:assembler). Zeilen der Form `;= 01 02` in den Beispielen zeigen die erzeugten Bytes; alle Beispiele sind geprüft.

## Welche Direktive nehme ich?

| Ich möchte … | Nimm | Ergebnis für „A“ |
|---|---|---|
| Text, der mit `jsr $ffd2` (CHROUT) ausgegeben wird | `.petscii "…"` | `$C1` |
| Text, den ich **direkt in den Bildschirmspeicher** (`$0400`) schreibe | `.screen "…"` | `$41` |
| Text für das Atari-Betriebssystem (CIO) | `.atascii "…"` | `$41` |
| Text unverändert als ASCII (Dateien, Protokolle, Fremdformate) | `.byte "…"` oder `.text "…"` | `$41` |
| viele Texte in der **gleichen** Kodierung | `.encoding "name"` und dann `.text "…"` | je nach Kodierung |
| Text mit **Nullbyte** am Ende, **Längenbyte** davor oder **gesetztem Bit 7** im letzten Zeichen | `.null`, `.ptext`, `.shift` | siehe unten |
| eine ganz eigene Kodierung | `.encoding "tabelle.tbl"` | siehe unten |

Alle Textdirektiven nehmen eine oder mehrere Zeichenketten in Anführungszeichen. Ein Anführungszeichen **in** der Kette gibt es nicht (keine Fluchtfolgen wie `\"`); schreibe es als Zahl: `.byte "sag ", 34, "hallo", 34`. Kommas und Zahlen mischen kannst du nur mit `.byte` (zum Beispiel `.byte "ABC", 13`), nicht mit `.text` & Co. Zeichen außerhalb von ASCII (`ä`, `ß`, `€`) liegen als UTF-8 in der Quelldatei und werden als **mehrere** Bytes abgelegt (`ä` ergibt `$C3 $A4`); setze sie bei Bedarf über Zahlen (`{$..}`, siehe unten) oder eine eigene Tabelle.

## `.petscii` – für die C64-Textausgabe

```
.petscii "text", "text", …
```

**PETSCII** ist der Zeichensatz des C64 (und VC-20, C128, Plus/4 und MEGA65) für Eingabe und Textausgabe über das Betriebssystem. Der Assembler wandelt so um:

| Quelltext | PETSCII | Wirkung auf dem C64 |
|---|---|---|
| `a`–`z` | `$41`–`$5A` | erscheint als **Großbuchstabe** im Standardmodus (Großbuchstaben/Grafik) |
| `A`–`Z` | `$C1`–`$DA` | erscheint als Großbuchstabe im Modus Klein-/Großschrift; im Standardmodus als Grafikzeichen |
| Ziffern, Satzzeichen, Leerzeichen | unverändert | |

Wenn dein Programm im **Standardmodus** startet (so schaltet der C64 ein), schreibe Text deshalb **klein**: `.petscii "hallo"` erscheint als `HALLO`. Willst du Groß- *und* Kleinbuchstaben, schalte vorher mit `{lower}` auf Klein-/Großschrift um (siehe Sonderzeichen unten).

```asm
.petscii "hallo"
;= 48 41 4c 4c 4f
.petscii "Hi"
;= c8 49
```

## `.screen` – für den Bildschirmspeicher

```
.screen "text", …
```

Das Betriebssystem übersetzt PETSCII beim Ausgeben in **Bildschirmcodes**, die der Video-Chip kennt. Schreibst du selbst in den Bildschirmspeicher (`sta $0400,x`), musst du direkt Bildschirmcodes ablegen – dafür ist `.screen`.

| Quelltext | Bildschirmcode |
|---|---|
| `@` | `$00` |
| `a`–`z` | `$01`–`$1A` (Großbuchstaben im Standardmodus) |
| `A`–`Z` | `$41`–`$5A` |
| Ziffern, Satzzeichen, Leerzeichen | wie ASCII (`$20`–`$3F`) |

Auf Zielsystemen mit ANTIC (Atari) wandelt `.screen` in den Atari-Bildschirmcode (`A` wird `$21`, Leerzeichen `$00`).

```asm
.screen "abc @"
;= 01 02 03 20 00
```

Beispiel – Text in den Bildschirm kopieren:

```asm
*= $c000
        ldx #0
loop:   lda meldung,x
        sta $0400,x
        inx
        cpx #meldung_ende - meldung
        bne loop
        rts
meldung:
        .screen "hallo welt"
meldung_ende:
;=* a2 00 bd 0e c0
```

## `.atascii` – Atari-Zeichensatz

```
.atascii "text", …
```

**ATASCII** ist die Kodierung des Atari-Betriebssystems. Sie entspricht ASCII, nur das Zeilenende ist `$9B` (`{eol}`) statt `$0A`. Ein `\n`-Zeichen im Quelltext gibt es nicht; schreibe `{eol}`.

```asm
.atascii "Hi{eol}"
;= 48 69 9b
```

## `.text` und `.encoding` – Kodierung wählen

```
.encoding "name"            ; ab hier gilt "name"
.encoding "name" {          ; nur innerhalb des Blocks
    .text "…"
}
.text "text", …
```

`.text` legt Text in der **aktuellen Kodierung** ab. Ohne `.encoding` ist das `ascii` (unverändert). Praktisch, wenn viele Strings die gleiche Kodierung haben und du nicht jedes Mal die Direktive wechseln willst; auch für Tabellen und Blöcke.

| Name | Wirkung |
|---|---|
| `ascii` (Vorgabe, auch `text`) | unverändert |
| `petscii` | wie `.petscii` |
| `screen` (auch `screencode`) | wie `.screen`; auf Atari-Zielsystemen der Atari-Bildschirmcode |
| `atascii` | wie `.atascii` |
| `"datei.tbl"` | eigene Tabelle: genau 256 Byte; Byte *n* der Datei ist der Code für das Zeichen *n* |

Die Einstellung gilt bis zur nächsten Umschaltung. Mit `{ … }` hinter dem Namen gilt sie nur im Block; danach ist wieder die vorige Kodierung aktiv (Labels im Block bleiben sichtbar).

```asm
.encoding "petscii"
.text "ab"
.encoding "screen"
.text "ab"
.encoding "ascii"
.text "ab"
;= 41 42 01 02 61 62
```

```asm
.encoding "screen" {
    .text "ab"             ; Bildschirmcode, nur hier
}
.text "ab"                 ; wieder ascii
;= 01 02 61 62
```

Ein unbekannter Name oder eine Tabelle ohne 256 Byte meldet **E208**:

```asm fehler
.encoding "gibts-nicht"
```

```text
error[E208]: .encoding: unbekannte Kodierung oder Datei 'gibts-nicht'
```

### Eigene Tabellen

Eine Tabelle ist eine Datei mit **256 Bytes**. Beim Übersetzen wird jedes Textzeichen (als Byte `0`–`255`) durch das Byte an dieser Stelle der Datei ersetzt. So lassen sich Zeichensätze nachbilden, die der Assembler nicht kennt, oder Zeichen auf eigene Grafik legen. Die Datei wird relativ zum Quelltext gesucht, dann in den Include-Ordnern.

Beispiel mit einer Tabelle `zeichen.tbl`, in der jedes Byte um 1 größer ist (`A` → `B`):

```asm
.encoding "zeichen.tbl"
.text "AB"
;= 42 43
```

Nicht-ASCII-Zeichen kommen als UTF-8-Bytes bei der Tabelle an. In Tabellen und mit `screen` kannst du Codes auch direkt als `{$hh}` oder `{nnn}` angeben (siehe unten).

## `.null`, `.ptext`, `.shift` – Textende markieren

Wie hört eine Ausgaberoutine auf? Es gibt drei übliche Arten, das Ende zu markieren. Die Direktiven nehmen **eine oder mehrere** Zeichenketten und wandeln sie in der aktuellen Kodierung (`.encoding`) um; jede Kette bekommt ihre eigene Markierung.

| Direktive | Ergebnis für `"AB"` | Wann |
|---|---|---|
| `.null "AB"` | `41 42 00` | Ausgaberoutine liest bis zum Nullbyte (`lda text,x` / `beq fertig`) – die gebräuchlichste Form |
| `.ptext "AB"` | `02 41 42` | Länge steht **vorn**; die Routine kennt die Länge vorab, kann also bequem zählen. Höchstens 255 Zeichen, sonst **E203** |
| `.shift "AB"` | `41 C2` | Bit 7 des **letzten** Zeichens ist gesetzt; spart ein Byte je Text, z. B. bei Befehlslisten. Leerer Text ist **E203** |

```asm
.encoding "petscii"
.null "hi"
.ptext "hi"
.shift "hi"
;= 48 49 00 02 48 49 48 c9
```

Eine Ausgaberoutine für einen `.null`-Text:

```asm
*= $c000
        ldx #0
loop:   lda text,x
        beq fertig
        jsr $ffd2              ; CHROUT: Zeichen ausgeben
        inx
        bne loop
fertig: rts
        .encoding "petscii"
text:   .null "{clr}hallo welt{ret}"
;=* a2 00 bd 0e c0 f0 06 20 d2 ff e8 d0 f5 60
```

## Sonderzeichen in Zeichenketten: `{name}`

Steuerzeichen wie „Bildschirm löschen“ lassen sich nicht tippen. Bei `.petscii`, `.atascii`, `.screen` und bei `.text` mit gesetzter Kodierung (alles außer `ascii`) steht in **geschweiften Klammern** das fertige Byte:

| Schreibweise | Bedeutung |
|---|---|
| `{name}` | Steuerzeichen mit Namen (nur bei PETSCII und ATASCII), z. B. `{clr}`, `{down}`, `{rvs on}` |
| `{$hh}` | beliebiger Code hexadezimal, z. B. `{$c1}` – auch für Grafikzeichen; bei `.screen` die einzige Form |
| `{nnn}` | beliebiger Code dezimal, z. B. `{147}` |
| `{name*3}` | Wiederholung (1 bis 255): `{down*3}`, `{$20*40}` |
| `{{` | ein einzelnes `{` |

- Der Wert in der Klammer ist das **fertige Byte** und wird nicht noch einmal umgewandelt.
- Groß-/Kleinschreibung und `_` oder `-` statt Leerzeichen sind egal: `{RVS_ON}` = `{rvs on}`.
- Bei `.text` ohne Kodierung (`ascii`) sind Klammern gewöhnliche Zeichen.
- Ein unbekannter Name, eine offene Klammer oder eine falsche Zahl melden **E208**.

```asm
.petscii "{clr}Hi{down*2}{rvs on}X{rvs off}{ret}{$c1}{147}"
;= 93 c8 49 11 11 12 d8 92 0d c1 93
.petscii "a{{b"
;= 41 7b 42
```

```asm fehler
.petscii "{nix}"
```

```text
error[E208]: unbekanntes Sonderzeichen {nix}
```

Bei `.screen` gibt es keine Namen (der Bildschirmcode kennt keine Steuerzeichen), nur `{$hh}` und `{nnn}`:

```asm fehler
.screen "{clr}"
```

```text
error[E208]: {clr}: Namen gibt es nur mit .petscii/.atascii oder .encoding "petscii"/"atascii" (hier nur {$hh} und {nnn})
```

### Namen für PETSCII

| Name | Code | Wirkung |
|---|---|---|
| `{wht}` | `$05` | Weiß |
| `{ret}` | `$0D` | Wagenrücklauf (RETURN) |
| `{lower}` | `$0E` | auf Klein-/Großschrift umschalten |
| `{down}` | `$11` | Cursor runter |
| `{rvs on}` | `$12` | Invers an |
| `{home}` | `$13` | Cursor links oben |
| `{del}` | `$14` | Zeichen links löschen |
| `{red}` | `$1C` | Rot |
| `{right}` | `$1D` | Cursor rechts |
| `{grn}` | `$1E` | Grün |
| `{blu}` | `$1F` | Blau |
| `{space}` | `$20` | Leerzeichen |
| `{orange}` | `$81` | Orange |
| `{f1}` `{f3}` `{f5}` `{f7}` | `$85` `$86` `$87` `$88` | Funktionstasten |
| `{f2}` `{f4}` `{f6}` `{f8}` | `$89` `$8A` `$8B` `$8C` | Funktionstasten |
| `{shift ret}` | `$8D` | Zeilenvorschub (Umschalt+RETURN) |
| `{upper}` | `$8E` | auf Großschrift/Grafik umschalten |
| `{blk}` | `$90` | Schwarz |
| `{up}` | `$91` | Cursor hoch |
| `{rvs off}` | `$92` | Invers aus |
| `{clr}` | `$93` | Bildschirm löschen |
| `{inst}` | `$94` | Zeichen einfügen |
| `{brn}` | `$95` | Braun |
| `{lred}` | `$96` | Hellrot |
| `{gry1}` `{gry2}` `{gry3}` | `$97` `$98` `$9B` | Dunkel-, Mittel-, Hellgrau |
| `{lgrn}` | `$99` | Hellgrün |
| `{lblu}` | `$9A` | Hellblau |
| `{pur}` | `$9C` | Violett |
| `{left}` | `$9D` | Cursor links |
| `{yel}` | `$9E` | Gelb |
| `{cyn}` | `$9F` | Türkis |
| `{shift space}` | `$A0` | Umschalt+Leerzeichen |

### Namen für ATASCII

| Name | Code | Wirkung |
|---|---|---|
| `{esc}` | `$1B` | Escape (das nächste Zeichen wird nur dargestellt) |
| `{up}` `{down}` `{left}` `{right}` | `$1C` `$1D` `$1E` `$1F` | Cursor |
| `{clr}` | `$7D` | Bildschirm löschen |
| `{bs}` | `$7E` | Rückschritt |
| `{tab}` | `$7F` | Tabulator |
| `{eol}` | `$9B` | Zeilenende |
| `{del line}` `{ins line}` | `$9C` `$9D` | Zeile löschen / einfügen |
| `{clr tab}` `{set tab}` | `$9E` `$9F` | Tabulator löschen / setzen |
| `{bell}` | `$FD` | Klingel |
| `{del char}` `{ins char}` | `$FE` `$FF` | Zeichen löschen / einfügen |

Weiter mit: [Direktiven](help:asm-directives), [Spickzettel](help:asm-index).
