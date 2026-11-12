# Assembler: Direktiven

Eine **Direktive** ist eine Anweisung an den *Assembler* selbst – kein Befehl für den Prozessor. Sie sagt zum Beispiel, wo das Programm im Speicher liegen soll (`.org`), legt Daten ab (`.byte`) oder wiederholt einen Block (`.repeat`). Direktiven beginnen mit einem Punkt, stehen allein auf einer Zeile (davor darf ein Label stehen) und erzeugen entweder Bytes oder keine.

```
label:  .direktive  argument, argument     ; Kommentar
```

- Schreibweise: Groß-/Kleinschreibung ist egal (`.BYTE` = `.byte`).
- Kommentare beginnen mit `;` (oder `//`).
- Argumente trennst du mit Kommas; jedes Argument darf ein [Ausdruck](help:asm-expr) sein.
- Eine unbekannte Direktive meldet **E208**.

Zeilen der Form `;= 01 02` in den Beispielen zeigen die vom Assembler erzeugten Bytes. Alle Beispiele sind mit dem Assembler geprüft; Beispiele, die nur mit Zusatzdateien laufen, nennen die Dateien. Mit **F1** auf einer Direktive im Editor öffnet sich die Kurzbeschreibung samt Link hierher. Als Übersicht dient der [Spickzettel](help:asm-index).

| Gruppe | Direktiven |
|---|---|
| Adresse und Speicher | `.org` `*=` `.page` `.align` `.fill` `.lohifill` `.pseudopc` `.endpseudopc` `.realpc` |
| Daten ablegen | `.byte` `.word` `.rta` `.be16` `.dword` `.hex` `.incbin` (Texte: [Texte und Zeichensätze](help:asm-text)) |
| Konstanten und Variablen | `=` `.const` `.equ` `.var` `.set` |
| Bedingte Übersetzung | `.if` `.elseif` `.else` `.endif` `.ifdef` `.ifndef` `.assert` `.error` `.warning` |
| Wiederholen | `.repeat` `.for` `.next` `.break` `.continue` |
| Dateien | `.include` `#include` `.incbin` |
| Programm starten | `.basicstub` `.run` |
| Zyklen zählen | `.cycles` `.endcycles` |
| Sprünge | `.branchlong` |
| Zielsystem | `.target` `.cpu` |
| Makros, Scopes, Strukturen | [eigene Seite](help:asm-macros) |
| Segmente, Bänke, Module | [eigene Seite](help:cartridges) |

## Adresse und Speicher

### `.org` und `*=` – Adresse festlegen

```
.org adresse
*= adresse          ; gleichbedeutend (auch  * = adresse)
```

Setzt den **Programmzähler** (die Adresse, an der der nächste Befehl oder das nächste Datenbyte liegt) und beginnt ein neues **Segment**. Ein Segment ist ein zusammenhängender Block Bytes mit fester Startadresse.

- Vor dem ersten `.org` darf **kein Code und kein Label** stehen, sonst **E300** („Code vor .org“).
- Du darfst `.org` mehrfach benutzen. Bei der Ausgabe sortiert der Assembler die Segmente nach Adresse.
- Beim PRG-Format (C64 & Co.) wird alles zu **einem** Block ab der niedrigsten Adresse; Lücken dazwischen füllt der Assembler mit Nullbytes. Beim Atari-XEX bleibt jedes Segment ein eigener Block.
- Überlappen sich zwei Segmente, meldet der Assembler **E301**.
- `adresse` muss 0 bis 65535 sein, sonst **E203**.

```asm
*= $c000
start:  lda #1
        rts
;= a9 01 60
```

```asm fehler roh
 nop
```

```text
error[E300]: Code vor .org
```

Zwei getrennte Bereiche:

```asm
.org $c000
        .byte 1
.org $c010
        .byte 2
;=* 01
```

`.org next(1024)` springt zur nächsten durch 1024 teilbaren Adresse (siehe [next()](help:asm-expr)). Der Unterschied zu `.align`: `.org` lässt eine Lücke, `.align` füllt sie mit Bytes.

### `.page` – an die nächste Seitengrenze

```
.page          ; nächste Seite (Vielfaches von 256)
.page n        ; nächstes Vielfaches von n, z. B. .page 1024
```

Eine **Seite** (Page) sind 256 Bytes, also die Adressen `$xx00`–`$xxFF`. Der 6502 ist an Seitengrenzen empfindlich: ein Sprung oder ein indizierter Zugriff über die Grenze kostet einen Zyklus mehr. Tabellen und Schleifen, die das nicht dürfen, legst du deshalb auf den Seitenanfang.

`.page` beginnt ein neues Segment an der nächsten Grenze, **ohne Füllbytes zu erzeugen** (wie `.org next(256)`). Ist die Adresse schon passend, bleibt sie.

```asm
*= $c001
        nop
        .page
tab:    .byte 7
        .word tab
;=* ea
```

### `.align` – mit Nullbytes auffüllen

```
.align n
```

Füllt mit **Nullbytes** auf das nächste Vielfache von `n` auf. Das Ziel ist wie bei `.page`, nur für beliebiges `n` und mit Füllbytes im selben Segment – nützlich, wenn der Bereich zusammenhängend im Modul-Abbild stehen muss.

```asm
*= $c001
        nop
        .align 4
        .byte 9
;= ea 00 00 09
```

### `.fill` – Bytes füllen und Tabellen erzeugen

```
.fill anzahl
.fill anzahl, ausdruck
.fill anzahl, ausdruck, laufvariable
```

Legt `anzahl` Bytes ab (0 bis 65536). Im `ausdruck` steht die **Laufvariable** (Vorgabe `i`, zählt ab 0) für die Nummer des Bytes. Ohne Ausdruck sind es Nullen. Die Laufvariable gibt es nur dort.

```asm
.fill 3, $ea                 ; drei Bytes $EA
.fill 4, i * 2               ; 0, 2, 4, 6
.fill 3, k + 10, k           ; anderer Name für die Laufvariable: 10, 11, 12
;= ea ea ea 00 02 04 06 0a 0b 0c
```

Praktisch für Platzhalter (`puffer: .fill 256`) und kleine Tabellen. Für Kurven nimm `.repeat` oder `.for` mit Kommazahlen (siehe [Ausdrücke](help:asm-expr)).

### `.lohifill` – Adresstabellen für den 6502

```
.lohifill anzahl, ausdruck [, laufvariable]
```

Legt eine Tabelle von 16-Bit-Werten so ab, wie der 6502 sie am bequemsten liest: **erst alle Low-Bytes, dann alle High-Bytes**. Der Zugriff ist dann `lda tab,x` und `lda tab+anzahl,x`. Werte außerhalb von 16 Bit melden **E203**.

```asm
zeilen:  .lohifill 25, $0400 + 40 * i    ; Startadressen der 25 Bildschirmzeilen
;=* 00 28 50 78
        lda zeilen,x                     ; Low-Byte der Zeile x
        sta $fb
        lda zeilen+25,x                  ; High-Byte
        sta $fc
```

### `.pseudopc`, `.endpseudopc`, `.realpc` – Code für eine andere Adresse

Der **Pseudo-PC** ist ein „falscher“ Programmzähler: Der Assembler legt die Bytes dort ab, wo sie in der Datei stehen, rechnet aber alle Adressen und Labels so, als lägen sie woanders. Das braucht man, wenn Code zur Laufzeit an eine andere Stelle **kopiert** wird – etwa eine Routine aus einem Steckmodul (ROM) ins RAM, ein Programm in die Zeropage oder ein Overlay.

```
.pseudopc adresse
...
.endpseudopc          ; auch  .realpc
```

- Zwischen den beiden Direktiven gelten Labels und `*` mit der **Laufzeitadresse** `adresse`.
- Die Bytes werden trotzdem an der **echten** Stelle abgelegt; hinter `.endpseudopc` (oder `.realpc`) geht es dort weiter.
- Ein Label **vor** `.pseudopc` (auch in derselben Zeile) hat die echte Adresse.
- Vor `.org` ist es ein Fehler (**E300**). Verschachtelte Pseudo-PC-Abschnitte gibt es nicht: `.endpseudopc` kehrt immer zur echten Adresse zurück.

```asm
start:  ldx #0
copy:   lda image,x          ; image = echte Adresse im Programm
        sta $0200,x          ; ... kopiert an die Laufadresse
        inx
        cpx #image_end - image
        bne copy
        jmp $0200
image:
        .pseudopc $0200
lauf:   inc $d020
        jmp lauf             ; lauf = $0200, nicht die echte Adresse
        .endpseudopc
image_end:
;=* a2 00 bd 10 c0
```

Das Beispiel kopiert die Routine ab `image` nach `$0200` und springt hin. Die Länge `image_end - image` rechnet mit echten Adressen; `lauf` ist `$0200`. Ein Beispiel für Module steht in [Segmente, Bänke, Module](help:cartridges).

## Daten ablegen

### `.byte` / `.db` – Bytes

```
.byte wert, wert, "text", …
```

Jedes Argument ist ein Ausdruck (−128 bis 255, sonst **E203**) oder eine Zeichenkette in Anführungszeichen. Eine Zeichenkette ergibt die Zeichen **unverändert als ASCII**, bei `ä`, `ß` & Co. als UTF-8-Bytes. Für C64-Bildschirm oder PETSCII nimm [.petscii, .screen oder .text mit .encoding](help:asm-text). Eine Liste liefert alle ihre Elemente.

```asm
.byte 1, $ff, -1, "AB", 'C', 2 + 3
;= 01 ff ff 41 42 43 05
```

In einer Zeichenkette gibt es keine Fluchtfolgen wie `\n`; ein Anführungszeichen schreibst du als Zahl: `.byte "sag ", 34, "hallo", 34`.

### `.word` / `.dw` – 16-Bit-Werte

```
.word wert, wert, …
```

Legt jeden Wert als zwei Bytes ab, **Low-Byte zuerst** („little endian“ – so liest der 6502 Adressen). Erlaubt: −32768 bis 65535, sonst **E203**. Typisch für Tabellen mit Adressen.

```asm
start:  .word start, $1234, -1
;= 00 c0 34 12 ff ff
```

### `.rta` – Adresse minus eins (für `rts`-Sprünge)

```
.rta ziel, ziel, …
```

Legt ein Wort mit `ziel − 1` ab. Der 6502-Befehl `rts` springt zu der Adresse, die oben auf dem Stack liegt, **plus eins**. Legst du eine Adresse minus eins auf den Stack (Low und High per `pha`) und führst `rts` aus, springt der Prozessor dorthin – das ist die klassische Sprungtabelle. `.rta` erledigt das „minus eins“.

```asm
*= $c000
tabelle: .rta eins, zwei
eins:   nop
zwei:   nop
;= 03 c0 04 c0 ea ea
```

`eins` liegt bei `$c004`, im Wort steht deshalb `$c003` (Bytes `03 c0`); `zwei` liegt bei `$c005`, im Wort steht `$c004`.

### `.be16` / `.dbyt` – 16 Bit, High-Byte zuerst

Wie `.word`, aber in der Reihenfolge „big endian“, wie sie manche Dateiformate und Hardware (z. B. der Atari-SIO-Bus oder Netzwerkdaten) erwarten.

```asm
.be16 $1234
;= 12 34
```

### `.dword` / `.long` – 32 Bit

Vier Bytes, Low-Byte zuerst. Wertebereich −2147483648 bis 4294967295 (sonst **E203**). `.long` ist hier ein Synonym und **auch 32 Bit** breit (in manchen anderen Assemblern ist `.long` 24 Bit).

```asm
.dword $12345678
;= 78 56 34 12
```

### `.hex` – Bytes aus Hexziffern

```
.hex "aa bb 0c", …
```

Paare von Hexziffern, Leerzeichen sind egal. Eine ungerade Anzahl Ziffern meldet **E208**. Praktisch für eingefügte Daten aus einem Hexdump.

```asm
.hex "de ad be ef"
;= de ad be ef
```

### `.incbin` – Binärdatei einbinden

```
.incbin "datei" [, offset [, länge]]
```

Fügt den Inhalt einer Datei unverändert ein. `offset` überspringt Bytes am Anfang, `länge` begrenzt die Zahl (sonst bis zum Dateiende). Gesucht wird zuerst im Ordner der aufrufenden Quelldatei, dann in den Include-Ordnern (`-I`).

- Datei nicht gefunden: **E212**. Bereich größer als die Datei: **E203**.
- Eine `.prg`-Datei beginnt mit der **Ladeadresse** (2 Bytes). Willst du nur die Daten, schreibe `.incbin "bild.prg", 2`.
- Sprites, Zeichensätze, Bildschirme und Songs aus dem Projekt exportiert der Build nach `build/export/` – als `.asm` zum Einbinden mit `.include`, nicht als Binärdatei.

Beispiel mit einer Datei `daten.bin` (16 Bytes: 00 01 … 0f):

```asm
.incbin "daten.bin", 4, 3          ; drei Bytes ab Offset 4
;= 04 05 06
.incbin "daten.bin", 14            ; ab Offset 14 bis zum Ende
;= 0e 0f
```

## Konstanten und Variablen

Es gibt zwei Arten benannter Zahlen, die man unterscheiden sollte:

| | Konstante | Variable |
|---|---|---|
| Schreibweise | `NAME = ausdruck` oder `.const NAME = ausdruck` (`.equ` ist dasselbe) | `.var name = ausdruck` |
| ändern | nein – ein zweites Setzen mit **anderem** Wert ist **E204** | ja: `name = …`, `.set name = …`, `name += …` |
| Vorwärtsreferenz | ja (darf **vor** der Definition benutzt werden) | nein (gilt ab der Zeile, in der sie gesetzt wird) |
| kann halten | ganze Zahlen (und Adressen) | Zahlen, Kommazahlen, Listen, Adressen |
| Zweck | Namen für feste Werte (Adressen, Farben, Größen) | Zähler und Zwischenwerte beim Übersetzen |

### `NAME = ausdruck`, `.const`, `.equ`

```asm
BORDER = $d020                       ; Konstante
.const FARBE = 5
        lda #FARBE
        sta BORDER
;= a9 05 8d 20 d0
```

Ob du `.const` oder die kurze Form nimmst, ist Geschmack – `.const` fällt beim Lesen stärker auf, die kurze Form entspricht der üblichen Schreibweise anderer Assembler. Wird `NAME` vorher mit `.var` angelegt, ist `NAME = …` eine Zuweisung an die Variable.

```asm fehler
K = 1
K = 2
```

```text
error[E204]: Symbol 'K' doppelt definiert
```

### `.var` und `.set` – Variablen

```
.var name = ausdruck        ; anlegen
name = ausdruck             ; ändern
name += ausdruck            ; auch -=  *=  /=  %=  &=  |=  ^=  <<=  >>=
.set name = ausdruck        ; ändern (Schreibweise wie .var)
```

`.set` auf einen Namen, der nicht mit `.var` angelegt wurde, ist **E216**. Variablen folgen den [Scopes](help:asm-macros): eine gleichnamige Variable in einem `{ … }`-Block verdeckt die äußere.

```asm
.var x = 0
x = x + 1
x += 2
.byte x
;= 03
```

```asm fehler
K = 1
.set K = 2
```

```text
error[E216]: Variable 'K' ist nicht mit .var angelegt
```

Variablen sind vor allem mit Schleifen nützlich, siehe Wiederholen.

## Bedingte Übersetzung

Mit Bedingungen entscheidest du **beim Übersetzen**, welcher Quelltext überhaupt in das Programm kommt – etwa eine Debug-Ausgabe oder Varianten für verschiedene Systeme. Nicht ausgewählte Zeilen erzeugen keine Bytes und definieren keine Labels.

### `.if`, `.elseif`, `.else`, `.endif`

```
.if ausdruck
    …
.elseif ausdruck
    …
.else
    …
.endif
```

Ist der Ausdruck ungleich 0, wird der Block übersetzt. Es gilt der erste Zweig mit wahrer Bedingung. `.if` darf verschachtelt werden und innerhalb von Makros und Schleifen stehen. Ein `.if` ohne `.endif` meldet **E210**.

```asm
MODUS = 2
.if MODUS == 1
        .byte 10
.elseif MODUS == 2
        .byte 20
.else
        .byte 99
.endif
;= 14
```

Die Bedingung sollte nur Werte benutzen, die schon vor der Zeile feststehen (Konstanten, Variablen, `-D`-Werte). Hängt sie von einer Adresse weiter unten im Quelltext ab, kann das Ergebnis zwischen den Durchläufen schwanken.

### `.ifdef` und `.ifndef` – ist ein Name definiert?

```
.ifdef name
.ifndef name
```

Prüft, ob `name` (Konstante, Label, Variable) **in diesem Durchlauf schon definiert ist**. Ein Name, der erst weiter unten definiert wird, zählt als nicht definiert. Zusammen mit `-D` auf der Kommandozeile (`vcc asm … -D DEBUG=1`) oder `"defines"` in der Projektdatei schaltet man Varianten:

```asm D=DEBUG=1
.ifdef DEBUG
        inc $d020          ; nur in der Debug-Variante
.endif
;= ee 20 d0
```

```asm
.ifndef DEBUG
        nop                ; ohne -D DEBUG
.endif
;= ea
```

Willst du auch den *Wert* prüfen (`-D DEBUG=0`), nimm `.if DEBUG`. `.ifdef` fragt nur nach der Existenz.

### `.assert`, `.error`, `.warning` – eigene Prüfungen und Meldungen

```
.assert bedingung [, "meldung"]
.error "text"
.warning "text"
```

- `.assert` bricht mit **E207** ab, wenn die Bedingung 0 ist. Gut, um Annahmen festzuschreiben („Tabelle passt in eine Seite“).
- `.error` meldet immer **E207**; gedacht für unmögliche Zweige in `.if`/`.else`.
- `.warning` erzeugt nur die Warnung **W200**; das Programm wird trotzdem gebaut.

```asm fehler
tab:    .fill 300
        .assert * - tab <= 256, "Tabelle ist größer als eine Seite"
```

```text
error[E207]: Assertion fehlgeschlagen: "Tabelle ist größer als eine Seite"
```

```asm warnung
.warning "noch nicht fertig"
        nop
```

```text
warning[W200]: noch nicht fertig
```

```asm fehler
SYSTEM = 3
.if SYSTEM == 1
        nop
.elseif SYSTEM == 2
        nop
.else
        .error "SYSTEM muss 1 oder 2 sein"
.endif
```

```text
error[E207]: SYSTEM muss 1 oder 2 sein
```

## Wiederholen

### `.repeat` – einen Block n-mal einsetzen

```
.repeat anzahl [, variable]
    …
.endrepeat
```

Der Block wird `anzahl`-mal übersetzt (0 bis 65536, sonst **E203**). Mit `variable` bekommt jeder Durchlauf seine Nummer (ab 0); die Variable gilt nur im Block. So „entrollt“ man Schleifen: Der Prozessor muss sie nicht mehr durchlaufen.

```asm
.repeat 3
        nop
.endrepeat
;= ea ea ea
```

```asm
.repeat 4, n
        .byte n * n
.endrepeat
;= 00 01 04 09
```

**Fallstrick Labels:** Steht in der Schleife ein normales Label, entsteht es bei jedem Durchlauf neu → **E204** („doppelt definiert“). Löse das mit einem Scope `{ … }` im Block (siehe [Scopes](help:asm-macros)):

```asm
.repeat 2, n
{
lbl:    dex
        bne lbl
        .byte n
}
.endrepeat
;= ca d0 fd 00 ca d0 fd 01
```

`.break` und `.continue` gelten auch in `.repeat`.

### `.for` / `.next` – Schleife mit Zähler oder Liste

Es gibt drei Schreibweisen:

```
.for i = 0, i < 10, i += 3          ; Anfang, Bedingung, Schritt
.for i in range(3)                  ; Zahlen 0, 1, 2  (auch range(a, b) und range(a, b, schritt))
.for c in [2, 5, 7]                 ; Elemente einer Liste
    …
.next
```

```asm
.for i = 0, i < 10, i += 3
        .byte i                     ; 0, 3, 6, 9
.next
;= 00 03 06 09
.for i in range(3)
        .byte i
.next
;= 00 01 02
.for c in [2, 5, 7]
        .byte c * 2
.next
;= 04 0a 0e
```

- `.break` verlässt die innerste Schleife, `.continue` springt zum nächsten Durchlauf. Beide gelten nicht über ein Makro hinweg, das innerhalb der Schleife aufgerufen wird; außerhalb einer Schleife melden sie **E210**.
- Die Laufvariable gilt nur in der Schleife. Hatte sie vorher einen Wert, kommt er danach wieder.
- Höchstens 65536 Durchläufe, sonst **E217**. `range` zählt rückwärts, wenn der Schritt negativ ist; Schritt 0 ist ein Fehler.

```asm
.for i in range(10)
        .if i == 2
        .continue                   ; 2 überspringen
        .endif
        .if i == 5
        .break                      ; bei 5 aufhören
        .endif
        .byte i
.next
;= 00 01 03 04
```

**`.repeat` oder `.for`?** `.repeat 8, i` ist die kürzeste Form für „n-mal“. `.for` brauchst du, wenn der Zähler anders läuft (Schrittweite, rückwärts, Elemente einer Liste) oder wenn du `.break` / `.continue` mit eigener Variable willst. Beide erzeugen dasselbe.

## Dateien einbinden

### `.include` und `#include`

```
.include "datei"
.include "datei" as name
#include <pfad>
```

Setzt den Inhalt einer anderen Quelldatei an dieser Stelle ein, als stünde er hier.

| Schreibweise | Gesucht wird | Mehrfach |
|---|---|---|
| `.include "datei"` | im Ordner der aufrufenden Datei, dann in den Include-Ordnern (`-I`) | jedes Mal neu eingesetzt |
| `#include "datei"` | wie oben | **nur einmal** je Datei und Namensraum |
| `#include <pfad>` | nur in den Include-Ordnern: die [Systembibliothek](help:asm-library) | nur einmal |

`#include` verhält sich wie `#pragma once`; so dürfen Bibliotheken einander einbinden, ohne dass ihre Namen doppelt definiert werden. Wird dieselbe Datei zweimal mit `.include` geholt, meldet der Assembler die Namen als doppelt (**E204**).

Mit `as name` landet der Inhalt der Datei in einem eigenen **Namensraum** (siehe [Scopes](help:asm-macros)): Von außen heißen die Namen `name.symbol`. So lässt sich dieselbe Bibliothek mehrfach unter verschiedenen Namen einbinden (zwei Musikstücke, zwei Treiberinstanzen).

Beispiel mit der Datei `lib.inc`:

```
; lib.inc
SPEED = 3
spiele: lda #SPEED
        rts
```

```asm
.include "lib.inc"                 ; Namen direkt sichtbar
        jsr spiele
        .byte SPEED
;= a9 03 60 20 00 c0 03
```

```asm
.include "lib.inc" as m1           ; zwei Kopien unter verschiedenen Namen
.include "lib.inc" as m2
        jsr m1.spiele
        jsr m2.spiele
        .byte m1.SPEED
;= a9 03 60 a9 03 60 20 00 c0 20 03 c0 03
```

- Datei nicht gefunden: **E212**. Mehr als 16 Ebenen tief (oft ein Zyklus): **E211**.
- Ein ungültiger Name hinter `as` meldet **E208**.
- **Makros** bleiben auch bei `as name` global: ein Makro aus der Datei rufst du weiter einfach mit `makroname` auf.
- Steht `.include` in einem `.if`-Zweig, der nicht gilt, wird auch die Datei nicht übersetzt.

## Programm starten

### `.basicstub` – die BASIC-Startzeile

Ein **Stub** (englisch für „Stummel“) ist ein winziges Anfangsstück. Auf dem C64 & Co. startest du Maschinenprogramme normalerweise mit `RUN`. `RUN` führt aber BASIC aus – also braucht dein Programm ganz vorn eine kleine BASIC-Zeile, die in das Maschinenprogramm springt:

```
10 SYS 2061
```

Genau das legt `.basicstub` an: eine fertig kodierte BASIC-Zeile mit dem `SYS`-Befehl (Aufruf eines Maschinenprogramms an einer Adresse). Ohne diese Zeile müsste der Nutzer `SYS 2061` selbst eintippen.

```
.basicstub ziel [, zeilennummer] [, "kommentar"]
```

| Argument | Bedeutung |
|---|---|
| `ziel` | Adresse, an der das Maschinenprogramm beginnt; fast immer ein Label (`start`) |
| `zeilennummer` | BASIC-Zeilennummer 0 bis 63999, Vorgabe **10** |
| `"kommentar"` | wird als `REM`-Zeile hinter einem `:` angehängt (in Großbuchstaben); die ganze Zeile darf höchstens 200 Bytes lang sein (sonst **E203**) |

Der BASIC-Interpreter sucht das Programm an einer festen Adresse, die vom System abhängt. `.basicstub` kennt sie aus den Zieldaten des Projekts und setzt den Stub **ohne vorheriges `.org` selbst dorthin**. Du schreibst also einfach `.basicstub start` an den Anfang der Datei:

| System | BASIC-Startadresse |
|---|---|
| C64 | `$0801` |
| C128 | `$1C01` |
| C16/Plus/4 | `$1001` |
| VC-20 | `$1001` (Grundgerät), `$1201` (mit 8 KB oder mehr Erweiterung), `$0401` (3 KB) – je nach Speicherausbau des Projekts |
| MEGA65 | `$2001` |

Die Startadresse steht auch als Ladeadresse am Anfang der PRG-Datei. Ohne Zieldaten (zum Beispiel beim Aufruf des Assemblers ohne Projekt und ohne `--target`) gilt `$0801`. Der Stub belegt 12 Bytes (mit Kommentar entsprechend mehr); dein Programm beginnt gleich dahinter. Das Ziel wird als Dezimalzahl in die Zeile geschrieben; kennt der Assembler das Label im ersten Durchlauf noch nicht, setzt er vorläufig 2061 ein und korrigiert beim nächsten Durchlauf.

```asm roh fmt=prg
.basicstub start
start:  rts
;= 01 08 0b 08 0a 00 9e 32 30 36 31 00 00 00 60
```

Die ersten zwei Bytes (`01 08`) sind die Ladeadresse `$0801` der PRG-Datei, danach folgt der Stub. Dessen Bytes: `0b 08` Zeiger auf die nächste BASIC-Zeile (hier: Programmende), `0a 00` Zeilennummer 10, `9e` das Token (Kürzel) für `SYS`, `32 30 36 31` der Text „2061“, danach drei Nullbytes (Zeilen- und Programmende). Das Programm beginnt bei `$080D` = 2061.

Mit Zeilennummer und Kommentar:

```asm roh
*= $0801
.basicstub start, 1000, "test"
start:  rts
;= 12 08 e8 03 9e 32 30 36 38 3a 8f 20 54 45 53 54 00 00 00 60
```

Das ergibt `1000 SYS2068:REM TEST`. Im Programm steht zwischen dem Token `9e` (SYS) und den Ziffern kein Leerzeichen; `LIST` zeigt deshalb ebenfalls `SYS2068`. Ein Leerzeichen steht nur hinter `REM` (`20` vor dem Text). Die Schreibweise `10 SYS 2061` mit Leerzeichen in dieser Hilfe ist nur die übliche Lesefassung. `8f` ist das Token für `REM`. Der Kommentar steht für Bildschirmtext und Credits; er verlängert den Stub, deshalb ist `start` hier `$0814` = 2068.

Auf einem anderen System nur die Adresse tauschen:

```asm roh fmt=prg target=c128
.basicstub start
start:  rts
;= 01 1c 0b 1c 0a 00 9e 37 31 38 31 00 00 00 60
```

Ein `.basicstub` mit einer festen Adresse statt eines Labels geht auch (`.basicstub $0810`), dann muss dort dein Code beginnen. Eine ungültige Zeilennummer oder zu lange Zeile meldet **E203**.

**Mit eigenem `.org`:** Steht vor `.basicstub` ein `.org`, gilt diese Adresse. Stimmt sie nicht mit der BASIC-Startadresse des Zielsystems überein, warnt der Assembler mit **W204**, denn BASIC findet die Zeile dort nicht:

```asm warnung roh target=c128
.org $0801
.basicstub start
start:  rts
```

```text
warning[W204]: .basicstub steht bei $0801, die BASIC-Startadresse des Zielsystems ist $1C01 (BASIC findet die Zeile dort nicht)
```

Keine Warnung gibt es, wenn `.cartridge` davor steht (ein Modul startet nicht über BASIC) oder der Stub in einem `.pseudopc`-Abschnitt liegt. Der Stub muss am Anfang des Programms stehen: Wurden davor schon Bytes ausgegeben, meldet der Assembler **E218**.

```asm fehler roh
.org $0801
        nop
.basicstub start
start:  rts
```

```text
error[E218]: .basicstub muss am Anfang des Programms stehen (davor wurden schon Bytes ausgegeben)
```

Wann nehme ich was?

- Das Programm soll mit `RUN` laufen → `.basicstub start` am Anfang der Datei, ohne `.org` (das Zielsystem bestimmt die Adresse).
- Du brauchst keinen Start über BASIC (Modul, Atari, Programm wird von einem Lader aufgerufen) → kein Stub.
- Zeilennummer ändern, wenn das Programm weitere BASIC-Zeilen nachlädt oder der Nutzer `LIST` sehen soll.

### `.run` – Startadresse (Atari)

```
.run adresse
```

Merkt sich die Startadresse des Programms. Der Assembler schreibt sie beim Atari-XEX als eigenes Segment nach `$02E0` (RUNAD: dorthin springt der Atari nach dem Laden), beim Atari-Modul in die Startvektoren bei `$BFFA`. Beim C64 hat `.run` keine Wirkung – nimm dort `.basicstub`.

```asm fmt=xex target=atari8
*= $2000
start:  jmp start
.run start
```

## Zyklen zählen: `.cycles`

Der 6502 arbeitet im Takt. Jeder Befehl braucht eine feste Zahl von **Zyklen** (Takten): `nop` 2, `lda $d020` 4, `jsr` 6. Auf dem PAL-C64 hat eine Rasterzeile (eine Bildschirmzeile) 63 Zyklen. Wer Rasterbalken, Musikroutinen im Interrupt oder Tricks mit exaktem Timing programmiert, muss wissen, ob sein Code in diese Zeit passt.

`.cycles` und `.endcycles` umschließen einen Teil des Quelltextes und lassen den Assembler die Zyklen der darin enthaltenen Befehle **zusammenzählen**. Die Direktive erzeugt keinen Code und ändert nichts am Programm.

```
.cycles [name] [max=budget]
    …
.endcycles
```

| Teil | Bedeutung |
|---|---|
| `name` | frei gewählter Name des Blocks (ein Wort); erscheint in Meldungen und im Test. Ohne Namen ist er leer |
| `max=budget` | **Budget** (erlaubte Höchstzahl) in Zyklen, ein Ausdruck. Überschreitet der ungünstigste Fall das Budget, gibt es die Warnung **W201** – der Bau gelingt trotzdem |

**Was wird gezählt?** Der Assembler addiert die Zyklen jedes Befehls, der **zwischen den beiden Zeilen im Quelltext steht**, genau einmal – in der Reihenfolge des Quelltextes. Er führt das Programm nicht aus und folgt keinen Sprüngen. Das hat Folgen:

- Eine Schleife (`dex` / `bne`) zählt nur *einen* Durchlauf, nicht acht.
- Makros und `.repeat` zählen so oft, wie sie übersetzt werden (3 × `nop` aus `.repeat 3` sind 6 Zyklen).
- Zeilen in nicht gewählten `.if`-Zweigen zählen nicht; Daten (`.byte`) zählen nicht.
- `jsr` zählt 6 – die Zyklen der aufgerufenen Routine nicht.
- Ein Sprung nach vorn (`bne weiter`) zählt trotzdem alle Befehle dazwischen.

Es entstehen **zwei Werte**:

| Wert | Bedeutung |
|---|---|
| **min** | Summe der Grundzyklen aller Befehle. Entspricht dem günstigsten Fall, wenn kein Sprung genommen wird und keine Seitengrenze überschritten wird |
| **max** | jeder bedingte Sprung als *genommen* **und** über eine Seitengrenze (+2), jeder indizierte Lesezugriff mit Seitenüberschreitung (+1) – der ungünstigste Fall |

Eine **Seitengrenze** (Page) liegt alle 256 Bytes: Ein Sprung oder ein Zugriff wie `lda tab,x`, der über sie hinwegreicht, dauert einen Zyklus länger.

Ergebnis ansehen:

- im [Bauprotokoll](help:build) steht je Block eine Zeile `Zyklen „name“: min–max (Budget n)`;
- die Debug-Datei `.vcdbg` enthält die Blöcke (`cycleBlocks`);
- ein Test mit `"check": "cycles", "block": "name"` prüft den Block bei `vcc test` (siehe [Tests](help:tests)). Dort wird der gebaute Code *wirklich verfolgt* (alle Wege bis `rts`/`rti`); Schleifen und `jsr` meldet er als „nicht bestimmbar“. `.cycles` im Quelltext ist dagegen die einfache Summe.

Beispiel: ein Interrupt-Block mit Budget. Das Budget passt, es gibt keine Meldung.

```asm
.cycles irq max=24
        inc $d020          ; 6
        lda $d012          ; 4
        sta $d021          ; 4
        rts                ; 6  -> zusammen 20
.endcycles
;= ee 20 d0 ad 12 d0 8d 21 d0 60
```

Überschreitet der ungünstigste Fall das Budget, warnt der Assembler (Zeile des `.endcycles`):

```asm warnung
.cycles langsam max=10
        lda $d020          ; 4
        sta $d021          ; 4
        jsr $ffd2          ; 6
.endcycles
```

```text
warning[W201]: Zyklenbudget 'langsam' überschritten: worst-case 14 > 10 Zyklen
```

Eine Schleife zählt nur einmal; der Block `s` zeigt auch den Unterschied von min und max (min 16, max 19):

```asm
.cycles s max=20
        .repeat 3
        nop                ; 3 x 2 = 6
        .endrepeat
        lda $1234,x        ; 4, max 5
        beq weiter         ; 2, max 4 (genommen + Seitengrenze)
        nop                ; 2
weiter: nop                ; 2
.endcycles
```

Blöcke dürfen verschachtelt sein; der äußere zählt auch die Befehle des inneren. Typische Fehler:

| Meldung | Ursache |
|---|---|
| **E210** `.endcycles ohne .cycles` | schließende Zeile ohne Beginn |
| **E210** `.cycles 'x' ohne .endcycles` | Block nicht geschlossen |
| **W201** | worst-case größer als `max=` |

Wann nehme ich was?

- **Schnell im Quelltext absichern** („mein Interrupt-Handler darf höchstens 40 Zyklen brauchen“): `.cycles name max=40`.
- **Genaue Zahl für verzweigten Code, Messen im Emulator**: `vcc test` mit `cycles` (Pfadanalyse) bzw. Zyklenmessung im Emulator, siehe [Tests](help:tests).
- Abzüge durch DMA (Bad Lines, Sprites), Interrupts und Wartezustände berücksichtigt kein Zähler.

## Weite Sprünge: `.branchlong`

Bedingte Sprünge (`beq`, `bne`, …) reichen nur −128 bis +127 Bytes weit. Ein zu weiter Sprung ist normalerweise der Fehler **E203**.

```
.branchlong auto        ; zu weite Sprünge selbst verlängern (auch: on)
.branchlong off         ; zurück zum Standard
```

Mit `auto` ersetzt der Assembler einen zu weiten Sprung durch eine lange Form und meldet das mit der Warnung **W203**:

| CPU | `beq ziel` wird zu |
|---|---|
| 6502 und Verwandte | `bne *+5` + `jmp ziel` (5 Bytes) |
| CPU mit `bra`, aber ohne langen Sprung | `bra ziel` wird zu `jmp ziel` (3 Bytes) |
| 45GS02 (MEGA65) | `lbeq ziel` (3 Bytes, ein echter langer Sprung) |

Die Länge ändert die Adressen dahinter; der Assembler wiederholt die Durchläufe, bis alles stimmt. Einmal verlängerte Sprünge bleiben lang. Für `.cycles` zählt die lange Form 3 bis 5 Zyklen (6502).

```asm warnung
.branchlong auto
        beq weit
        .fill 200, $ea
weit:   nop
```

```text
warning[W203]: Sprung zu weit für BEQ: lange Form (5 Bytes) verwendet
```

## Zielsystem: `.target` und `.cpu`

### `.target`

```
.target c64
```

Schreibt das Zielsystem in den Quelltext (`c64`, `c128`, `plus4`, `vic20`, `atari8`, `mega65`). Der **Assembler selbst ignoriert die Zeile**; sie wird von den Werkzeugen gelesen: Steht in der IDE das Ziel auf „automatisch“, nimmt der Bau dieses Ziel (im Bauprotokoll: „aus .target“). In einem Projekt (`.vcprj`) bestimmt die Projektdatei das Ziel; auf der Kommandozeile `--target`.

### `.cpu`

```
.cpu 6510 { illegals = warn }
```

Nur die Angabe `illegals = deny | warn | allow` wirkt (Behandlung **undokumentierter Befehle** wie `lax` oder `dcp`: nicht erlaubt, mit Warnung **W104**, erlaubt). Die CPU-Kennung davor (`6510`) liest der Assembler nicht: die CPU ergibt sich aus dem Zielsystem. Als Standard gilt, was in Projekt oder `--illegals` steht. Befehle, die als *instabil* gelten (`xaa`), warnen immer mit **W105**; eine nicht erlaubte Verwendung meldet **E201**.

```asm fehler illegals=deny
 lax $d020
```

```text
error[E201]: undokumentierter Opcode LAX (illegals=deny)
```

Weiter mit: [Makros, Scopes und Strukturen](help:asm-macros), [Texte und Zeichensätze](help:asm-text), [Spickzettel](help:asm-index).
