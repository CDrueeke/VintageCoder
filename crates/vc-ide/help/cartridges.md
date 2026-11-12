# Segmente, Bänke und Module

Normalerweise erzeugt der Assembler ein **Programm**, das von einer Diskette oder Kassette geladen wird. Ein **Modul** (englisch *Cartridge*) ist dagegen ein Steckmodul mit Festwertspeicher (ROM): Der Rechner sieht seinen Inhalt sofort im Adressraum, und das Programm startet beim Einschalten. Die drei Direktiven dieser Seite beschreiben, wo Bytes im Modul landen: `.segment` benennt und prüft Bereiche, `.bank` wählt eine Speicherbank, `.cartridge` macht aus allem ein Modulabbild.

Die Seite gehört zum [Assembler-Handbuch](help:assembler). Zeilen der Form `;= 01 02` in den Beispielen zeigen die erzeugten Bytes; alle Beispiele sind mit dem Assembler geprüft.

## Segmente: `.segment`

Ein **Segment** ist ein zusammenhängender Block erzeugter Bytes. Jedes `.org` beginnt ein neues. Mit `.segment` gibst du den folgenden Segmenten einen Namen (er erscheint in der Speicherbelegung, im Bauprotokoll und im Debug-Info) und kannst den **erlaubten Adressbereich** festlegen.

```
.segment "NAME"                 ; nur benennen
.segment "NAME", von, bis       ; benennen und Bereich prüfen
.segment                        ; Namen und Bereich aufheben
```

Verlässt ein Byte den Bereich `von`–`bis` (beide Grenzen gehören dazu), meldet der Assembler **E301**. So fängst du früh Fehler wie „Code läuft in die Daten hinein“:

```asm
.segment "CODE", $c000, $c0ff
*= $c000
        nop
.segment "DATA", $c100, $c1ff
*= $c100
        .byte 1
;=* ea
```

```asm fehler
.segment "CODE", $c000, $c003
*= $c000
        .fill 8, 0
```

```text
error[E301]: Segment 'CODE' verlässt seinen Bereich $C000-$C003 ($C000..$C007)
```

Überlappen sich zwei Segmente (auch ohne Namen), meldet der Assembler **E301** („Segmente überlappen“).

## Bänke: `.bank`

Module sind oft größer als der Adressraum, den der Rechner dafür bereithält (8 KB bei `$8000`). Der Rest steckt in weiteren **Bänken**; zur Laufzeit schaltet das Programm eine Bank ein, und ihr Inhalt erscheint an derselben Adresse.

```
.bank n            ; n = 0 bis 1023
```

- Jede Bank hat ihren **eigenen Adressraum und Programmzähler**. Nach `.bank 1` setzt du mit `.org` die Adresse innerhalb dieser Bank. Schaltest du später zurück, geht es dort weiter, wo Bank 0 aufgehört hat.
- Bank 0 ist die Vorgabe.
- Bänke gibt es nur in den Modulformaten (`crt`, `car`, `rom`). Für ein normales Programm (`prg`, `xex`) meldet der Bau „mehrere Bänke (.bank) gibt es nur in den Modulformaten crt/car/rom“.

## Module: `.cartridge`

```
.cartridge "typ", "NAME"
```

Der Typ legt das Format fest; `NAME` ist der Modulname (bis 32 Zeichen, groß geschrieben; Vorgabe `VINTAGECODER`). Steht `.cartridge` im Quelltext, erzeugt der Bau automatisch ein Modul statt eines Programms (`.crt` beim C64, `.car` beim Atari). Auf der Kommandozeile erzwingt `--cart typ` den Typ, `--format crt|car|rom` das Format. Ungenutzte Bereiche füllt der Assembler mit `$FF`.

| System | Typen | Ausgabe |
|---|---|---|
| C64 `.crt` | `normal` (auch `8k`, `16k`), `ocean`, `magicdesk`, `easyflash`, `actionreplay` (auch `ar`), `ultimax` | `.crt` (VICE-Modulformat) |
| Atari `.car` / `.rom` | `8k`, `16k`, `xegs` | `.car` mit Kopf oder `.rom` roh |
| VC-20 | `a0`, `20`, `40`, `60` | rohes 8-KB-Abbild, Endung = Typ |

Ein unbekannter Typ meldet erst beim Erzeugen des Moduls eine Meldung, etwa „unbekannter Modultyp 'gibts' (normal, 16k, ocean, magicdesk, easyflash, actionreplay, ultimax)“.

### C64-Module

Der C64 blendet ein Modul bei `$8000` (und bei manchen Typen `$A000`) ein. Der Rechner prüft beim Einschalten, ob bei `$8004` die Kennung **CBM80** steht (`$C3 $C2 $CD $38 $30`); davor liegen zwei Wörter: der Kaltstart- und der Warmstartvektor, also die Adressen, an denen das Programm beginnt. Fehlt die Kennung, startet das Modul nicht.

```asm fmt=crt
.cartridge "normal", "MEIN MODUL"
*= $8000
        .word kalt, kalt
        .byte $c3, $c2, $cd, $38, $30      ; CBM80
kalt:   inc $d020
        jmp kalt
```

(Die Bibliothek `lib/cart.inc` der Beispiele enthält das Makro `cbm80_header kalt, warm`, das diese neun Bytes anlegt; siehe `examples/c64/cart_demo`.)

| Typ | Aufbau |
|---|---|
| `normal` / `8k` | nur Bank 0, Code in `$8000`–`$BFFF`. Liegt ein Byte bei `$A000` oder höher (oder steht `16k`), entsteht ein 16-KB-Modul, sonst ein 8-KB-Modul |
| `ocean` | 16 bis 64 Bänke zu 8 KB (die Zahl wird auf eine Zweierpotenz aufgerundet). Bänke 0–15 liegen bei `$8000`–`$9FFF`, Bänke ab 16 bei `$A000`–`$BFFF` |
| `magicdesk` | 4 bis 16 Bänke zu 8 KB bei `$8000`–`$9FFF` |
| `easyflash` | bis 64 Bänke; je Bank ROML bei `$8000`–`$9FFF` und ROMH bei `$A000`–`$BFFF` **oder** `$E000`–`$FFFF` (nicht beides). Bank 0 braucht ROMH-Code mit den Startvektoren bei `$FFFC` |
| `actionreplay` | genau vier Bänke zu 8 KB bei `$8000`–`$9FFF` |
| `ultimax` | nur Bank 0: ROML `$8000`–`$9FFF` (freiwillig) und ROMH `$E000`–`$FFFF` mit Reset-Vektor bei `$FFFC` |

Bankumschaltung zur Laufzeit (aus `lib/cart.inc`): bei Ocean `lda #$80 | n` / `sta $de00`, bei Magic Desk `lda #n` / `sta $de00`. Der umschaltende Code muss in jeder Bank gleich liegen oder im RAM laufen; dazu dient oft [`.pseudopc`](help:asm-directives):

```asm fmt=crt
.cartridge "magicdesk", "BANKEN"
.segment "BANK0", $8000, $9fff
.bank 0
*= $8000
        .word start, start
        .byte $c3, $c2, $cd, $38, $30
start:  ldx #0
copy:   lda image,x             ; Umschaltcode ins RAM kopieren
        sta $c000,x
        inx
        cpx #image_end - image
        bne copy
        jsr $c000
        jmp *
image:
        .pseudopc $c000        ; läuft im RAM bei $C000
        lda #1                  ; Bank 1 einschalten
        sta $de00
        lda $8100               ; Kennung dort lesen
        sta $0400
        lda #0                  ; zurück zu Bank 0
        sta $de00
        rts
        .endpseudopc
image_end:
.segment "BANK1", $8000, $9fff
.bank 1
*= $8100
        .byte $b1               ; Kennung von Bank 1
```

### Atari-Module

Ein Atari-Modul liegt bei `$A000`–`$BFFF` (8 KB) oder `$8000`–`$BFFF` (16 KB). Die Startvektoren bei `$BFFA`–`$BFFF` (Startadresse, Flag, Option, Initialisierungsroutine) ergänzt der Assembler selbst, wenn dein Programm sie nicht anlegt: die Startadresse stammt aus `.run`, die Option heißt „Modul starten“, die Initialisierung ist ein einzelnes `rts`. Fehlt `.run`, meldet der Bau „Atari-Modul: Startadresse fehlt (.run oder eigene Vektoren bei $BFFA)“.

```asm fmt=car target=atari8
.cartridge "8k"
*= $a000
start:  jmp start
.run start
```

- `8k` / `16k` / ohne Angabe: Größe ergibt sich aus dem Code (liegt etwas unter `$A000`, wird es ein 16-KB-Modul).
- `xegs`: Bankmodul, 32, 64 oder 128 KB (4, 8 oder 16 Bänke zu 8 KB). Bänke 0 bis n−2 erscheinen bei `$8000`–`$9FFF`, die **letzte Bank** liegt fest bei `$A000`–`$BFFF`. Umgeschaltet wird mit einem Schreibzugriff auf `$D500`.
- `.rom` ist dasselbe Abbild ohne den 16-Byte-Kopf der `.car`-Datei.

### VC-20-Module

Rohe 8-KB-Abbilder für einen Speicherblock: `a0` (`$A000`–`$BFFF`), `20`, `40` oder `60` (`$2000`, `$4000`, `$6000`). Bänke gibt es nicht. Ein Modul bei `$A000` startet das System selbst, wenn bei `$A004` die Kennung `A0CBM` steht (`$41 $30 $C3 $C2 $CD`) und davor Kalt- und Warmstartadresse liegen. Mehr dazu unter [VC-20](help:vic20).

## Typische Fehler

| Meldung | Ursache |
|---|---|
| **E301** Segment verlässt seinen Bereich / Segmente überlappen | Code zu lang für den Bereich; zwei `.org` bei überlappenden Adressen |
| „Standardmodul: Code muss in $8000-$BFFF liegen“ | `.org` bei `.cartridge "normal"` falsch gesetzt |
| „Bank n: Segment … liegt nicht im Fenster …“ | Segment überschreitet das 8-KB-Fenster einer Bank |
| „ultimax: ROMH-Code im Fenster $E000-$FFFF mit Reset-Vektor bei $FFFC fehlt“ | Reset-Vektor vergessen |
| „mehrere Bänke (.bank) gibt es nur in den Modulformaten crt/car/rom“ | `.bank` in einem normalen Programm |

Siehe auch: [Direktiven](help:asm-directives) (`.org`, `.pseudopc`), [Bauen und Starten](help:build). Beispiele: `examples/c64/cart_demo/cart_demo.asm`, `examples/atari/cart_demo/cart_demo.asm`.
