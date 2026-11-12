# Assembler: Ausdrücke und Funktionen

Ein **Ausdruck** ist eine Rechnung, die der Assembler beim Übersetzen selbst ausführt – nicht der C64 beim Laufen. Das Ergebnis ist immer eine feste Zahl, die ins Programm eingesetzt wird. Ausdrücke dürfen überall stehen, wo eine Zahl gebraucht wird: als Operand (`lda #WIDTH * 2`), in Datendirektiven (`.byte 1 + 2`), in `.if`, `.org` und so weiter.

Die Seite gehört zum [Assembler-Handbuch](help:assembler). Zeilen der Form `;= 01 02` in den Beispielen zeigen, welche Bytes der Assembler erzeugt; alle Beispiele sind mit dem Assembler geprüft.

## Zahlen schreiben

| Schreibweise | Beispiel | Wert | Hinweis |
|---|---|---|---|
| dezimal | `255` | 255 | |
| hexadezimal mit `$` | `$ff`, `$D020` | 255, 53280 | Groß-/Kleinschreibung egal |
| hexadezimal mit `0x` | `0x1f` | 31 | wie in C |
| binär mit `%` | `%1010` | 10 | nur Ziffern 0 und 1 |
| Zeichen | `'a'` | 97 | der **ASCII**-Wert, genau ein Zeichen; für PETSCII siehe [Texte](help:asm-text) |
| Kommazahl | `2.5`, `1e3`, `1.5e-2` | siehe unten | nur zum Rechnen |

Nicht möglich: `0b1010` (binär nur mit `%`), Unterstriche in Zahlen (`1_000`) und Zeichenketten in Ausdrücken (`"AB"` gibt es nur in Datendirektiven).

```asm
.byte 255, $ff, 0xff, %11111111, 'A'
;= ff ff ff ff 41
```

Nach `%` muss gleich eine 0 oder 1 folgen, sonst ist das `%` der Rest-Operator (`7 % 3` ist 1).

## Wie groß dürfen Zahlen sein?

Der Assembler rechnet intern mit großen ganzen Zahlen (64 Bit mit Vorzeichen). Erst beim **Einsetzen** wird geprüft, ob der Wert passt:

| Stelle | erlaubt | sonst |
|---|---|---|
| `.byte`, `#wert` (unmittelbar) | −128 bis 255 | Fehler **E203** |
| `.word`, Adressen, `.org` | −32768 bis 65535 (Adressen 0 bis 65535) | Fehler **E203** |
| `.dword` | −2147483648 bis 4294967295 | Fehler **E203** |

Negative Werte werden als Zweierkomplement abgelegt: `.byte -1` ergibt `$FF`.

```asm fehler
 lda #256
```

```text
error[E203]: Wert 256 außerhalb -128..255
```

Mit `<` und `>` (siehe unten) holst du Low- und High-Byte eines 16-Bit-Wertes heraus, wenn ein Byte gebraucht wird.

## Symbole in Ausdrücken

| Was | Beispiel | Bedeutung |
|---|---|---|
| Label | `start`, `.schleife` | Adresse an dieser Stelle (`.name` = lokales Label, siehe [Labels und Scopes](help:asm-macros)) |
| Konstante | `BREITE` | Wert aus `BREITE = 40` oder `.const` |
| Variable | `i`, `x` | Wert aus `.var` (ändert sich beim Übersetzen) |
| Programmzähler | `*` | Adresse des aktuellen Befehls bzw. der aktuellen Datenzeile |
| Chip-Register | `vic2.border` | Adresse laut Hardware-Datenbank des Zielsystems |
| Scope-Name | `init.loop`, `music.play` | Symbol in einem Namensraum |
| `pi` | `pi` | die Kreiszahl 3.14159… |

Das Sternchen steht an zwei Stellen mit verschiedener Bedeutung: am **Zeilenanfang** vor `=` ist es der Programmzähler setzen (`*= $c000`), im **Ausdruck** ist es der Programmzähler lesen, und zwischen zwei Zahlen ist es das Malzeichen.

```asm
anfang: nop
        nop
laenge = * - anfang        ; 2 Bytes zwischen anfang und hier
        .byte laenge
;= ea ea 02
```

Der Programmzähler `*` ist bei Zeilen mit einem Befehl die Adresse **vor** dem Befehl. In einem [Pseudo-PC](help:asm-directives)-Abschnitt (`.pseudopc`) ist es die Laufzeitadresse.

### Vorwärtsreferenzen

Du darfst Symbole benutzen, bevor sie definiert sind: `jmp spaeter` vor `spaeter:` ist normal. Der Assembler übersetzt dazu mehrmals (Durchläufe), bis alle Adressen feststehen. Zwei Folgen davon:

- Bei einer Vorwärtsreferenz kennt der Assembler im ersten Durchlauf die Größe des Wertes noch nicht und plant für `lda spaeter` vorsichtshalber die lange (absolute) Adressform ein. Wo es auf die Länge ankommt, erzwingst du sie mit `lda.z` oder `lda.a` (siehe [Adressierungsarten](help:assembler)).
- `.ifdef` und `.ifndef` zählen nur Symbole, die **vor** der Zeile definiert wurden (sonst könnte die Abfrage zwischen den Durchläufen schwanken).

Ein Symbol, das es nirgends gibt, meldet **E202** (`unbekanntes Symbol`).

## Operatoren

Vom stärksten zum schwächsten Vorrang. Innerhalb einer Zeile der Tabelle wird von links nach rechts gerechnet; Klammern `( )` gehen immer vor.

| Vorrang | Operator | Bedeutung | Beispiel → Ergebnis |
|---|---|---|---|
| 1 (stärkster) | `-` `+` `!` `~` `<` `>` | Vorzeichen, logisches Nicht, bitweises Nicht, Low-Byte, High-Byte (nur **vor** einem Wert) | `-3`, `!0` → 1, `~0` → −1, `<$1234` → $34, `>$1234` → $12 |
| 2 | `*` `/` `%` | Mal, ganzzahlig geteilt, Rest | `7 / 2` → 3, `7 % 3` → 1 |
| 3 | `+` `-` | Plus, Minus | `10 - 2 - 3` → 5 |
| 4 | `<<` `>>` | Bits nach links / rechts schieben | `1 << 3` → 8, `$80 >> 7` → 1 |
| 5 | `&` | bitweises Und | `$ff & $0f` → 15 |
| 6 | `^` | bitweises Exklusiv-Oder | `1 ^ 3` → 2 |
| 7 | senkrechter Strich | bitweises Oder | `$30` oder `1` → $31 (Schreibweise siehe unten) |
| 8 | `<` `<=` `>` `>=` | Vergleiche (Ergebnis 1 oder 0) | `3 > 2` → 1 |
| 9 | `==` `!=` | gleich / ungleich (Ergebnis 1 oder 0) | `3 == 3` → 1 |
| 10 | `&&` | logisches Und | `1 && 0` → 0 |
| 11 (schwächster) | zwei senkrechte Striche | logisches Oder | `0` oder `2` → 1 (Schreibweise siehe unten) |

Das bitweise Oder ist ein einzelner senkrechter Strich, das logische Oder zwei (`$30 | 1` ergibt $31, `0 || 2` ergibt 1). Vergleiche und logische Operatoren liefern immer **1** (wahr) oder **0** (falsch). Als Bedingung (in `.if`, `.assert`, bei `&&`, `||` und `!`) gilt jeder Wert ungleich 0 als wahr.

```asm
.byte 2 + 3 * 4, (2 + 3) * 4, 10 - 2 - 3, 7 / 2, 7 % 3
;= 0e 14 05 03 01
.byte 1 << 3, $ff & $0f, 1 ^ 3, $30 | 1
;= 08 0f 02 31
.byte 3 > 2, 3 == 4, !0, !7, 1 || 0 && 0
;= 01 00 01 00 01
```

### Wie in Python, nicht wie in C

Die Bit-Operatoren `& ^ |` binden **stärker** als die Vergleiche. Dadurch tut `x & 3 == 2` das, was man meint – `(x & 3) == 2` –, während es in C als `x & (3 == 2)` gelesen würde:

```asm
.byte 6 & 3 == 2          ; (6 & 3) == 2  →  2 == 2  →  1
;= 01
```

Es gibt kein `**`, kein `? :`, kein `and`/`or`/`not` und kein einfaches `=` als Vergleich (`==` schreiben). Beide Seiten von `&&` und `||` werden immer ausgewertet (keine Abkürzung); das ist nur bei einer Division durch 0 auf der rechten Seite zu merken.

### Ganzzahl-Division und Rest

`/` und `%` schneiden zur Null hin ab, wie in C: `-7 / 2` ist −3, `-5 % 3` ist −2. Durch 0 teilen meldet **E202** („Division durch 0“).

```asm fehler
 .byte 1 / 0
```

```text
error[E202]: Division durch 0 in '1 / 0'
```

## Low-Byte und High-Byte: `<` und `>`

Der C64 hat einen 8-Bit-Prozessor, Adressen haben aber 16 Bit. Eine Adresse besteht aus **Low-Byte** (die unteren 8 Bit) und **High-Byte** (die oberen 8 Bit): `$1234` hat Low-Byte `$34` und High-Byte `$12`. Zum Aufteilen gibt es zwei Vorzeichen:

| Schreibweise | Wert |
|---|---|
| `<wert` | `wert & $FF` (Low-Byte) |
| `>wert` | `(wert >> 8) & $FF` (High-Byte) |

Typisch: eine Adresse in zwei Zeropage-Zellen legen.

```asm
ziel = $1234
        lda #<ziel          ; $34
        sta $fb
        lda #>ziel          ; $12
        sta $fc
;=* a9 34 85 fb a9 12 85 fc
```

**Achtung, Vorrang:** `<` und `>` binden stärker als `+`. `>ziel + 1` heißt also „High-Byte, dann plus 1“ (= $13), **nicht** „High-Byte von ziel+1“. Wer das Letzte will, setzt Klammern:

```asm
ziel = $12ff
        .byte >ziel + 1        ; ($12) + 1
        .byte >(ziel + 1)      ; High-Byte von $1300
;= 13 13
```

Bei `<` ist der Unterschied besonders unangenehm: `<ziel + 1` ergibt hier `$ff + 1 = $100`, und das passt nicht in ein Byte (**E203**). Setze deshalb im Zweifel immer die Klammer: `<(ziel + 1)` ergibt `$00`.

Eine Tabelle aus Adressen legst du bequem mit [.lohifill](help:asm-directives) oder zwei Listen `<`/`>` an (siehe unten bei Listen).

## Vergleiche und Logik

```asm
BREITE = 80
.if BREITE > 40
    .byte 1
.else
    .byte 2
.endif
;= 01
```

Wahrheitswerte sind einfach Zahlen: `5 > 3` ist 1, `5 < 3` ist 0, `!5` ist 0, `!0` ist 1. Das erlaubt Rechenkunststücke wie `farbe = (x > 10) * 2`, ist aber vor allem für `.if`, `.assert` und `.function` nützlich.

## Ganze Zahlen und Kommazahlen

Ganze Zahlen bleiben ganze Zahlen: `7 / 2` ist 3. Sobald eine Kommazahl im Spiel ist, wird genau gerechnet: `7 / 2.0` ist 3.5.

Dort, wo am Ende ein Byte oder Wort herauskommt, wird der Nachkommaanteil abgeschnitten und der Assembler warnt mit **W202**. Mit den Funktionen `int()`, `round()`, `floor()` oder `ceil()` wählst du selbst, wie gerundet wird, dann gibt es keine Warnung.

```asm warnung
 .byte 7 / 2.0
```

```text
warning[W202]: Kommazahl 3.5 auf 3 abgeschnitten (int(), round() oder floor() verwenden) in '7 / 2.0'
```

```asm
.byte int(7 / 2.0), round(2.5), floor(-2.5), ceil(2.1)
;= 03 03 fd 03
```

Die Operatoren `% & | ^ << >> ~` sowie `<` und `>` (Low-/High-Byte) gehen nur mit ganzen Zahlen; mit einer Kommazahl melden sie **E202**.

## Eingebaute Funktionen

Funktionen schreibst du mit Klammer, auch ohne Argumente den Klammern wegen: `sin(x)`. Groß-/Kleinschreibung ist egal (`ABS(-1)`). Ein **Label** namens `max` bleibt ein Label – es ist nur dann die Funktion, wenn direkt eine Klammer folgt.

| Funktion | Argumente | Ergebnis |
|---|---|---|
| `sin(x)` `cos(x)` `tan(x)` | 1 | Winkelfunktionen, Winkel im **Bogenmaß** (360° = `2 * pi`) |
| `asin(x)` `acos(x)` `atan(x)` | 1 | Umkehrfunktionen (Ergebnis im Bogenmaß) |
| `atan2(y, x)` | 2 | Winkel des Punktes (x, y), Bogenmaß |
| `sqrt(x)` | 1 | Quadratwurzel |
| `pow(a, b)` | 2 | a hoch b |
| `exp(x)` `log(x)` `log2(x)` `log10(x)` | 1 | e hoch x, natürlicher Logarithmus, Logarithmus zur Basis 2 bzw. 10 |
| `int(x)` | 1 | ganze Zahl **zur Null hin** (2.7 → 2, −2.7 → −2) |
| `floor(x)` | 1 | nach unten gerundet (−2.5 → −3) |
| `ceil(x)` | 1 | nach oben gerundet (2.1 → 3) |
| `round(x)` | 1 | auf die nächste ganze Zahl, bei .5 weg von Null (2.5 → 3) |
| `abs(x)` | 1 | Betrag |
| `sign(x)` | 1 | −1, 0 oder 1 |
| `min(a, b, …)` `max(a, b, …)` | 1 oder mehr | kleinster / größter Wert |
| `len(liste)` | 1 | Anzahl der Elemente einer Liste |
| `range(n)` `range(a, b)` `range(a, b, schritt)` | 1 bis 3 | Zahlenliste (höchstens 65536 Elemente) |
| `next(n)` oder `align(n)` | 1 | nächste durch `n` teilbare Adresse ab dem Programmzähler (bleibt stehen, wenn sie schon passt) |

Alle Funktionen außer `len` und `range` brauchen Zahlen, keine Listen. Falsche Argumentzahl und unbekannte Funktionen melden **E202**:

```asm fehler
 .byte abs(1, 2)
```

```text
error[E202]: abs() erwartet 1 Argument(e), erhalten 2 in 'abs(1, 2)'
```

```asm
.byte abs(-3), sign(-9), min(4, 2, 8), max(4, 2, 8)
;= 03 ff 02 08
.byte sqrt(16), pow(2, 5), log2(8), log10(1000)
;= 04 20 03 03
.byte int(sin(pi / 2)), int(cos(0)), round(atan2(1, 1) * 4)
;= 01 01 03
```

### Sinustabelle (Beispiel für Kommazahlen und `.repeat`)

Eine Sinuskurve mit 256 Werten von 1 bis 255, wie sie für Sprite-Bewegungen oder Rasterbalken gebraucht wird:

```asm
sinus:
    .repeat 256, i
        .byte round(128 + 127 * sin(i * 2 * pi / 256))
    .endrepeat
;=* 80 83 86 89
```

`i * 2 * pi / 256` rechnet in Kommazahlen, `round()` macht daraus die ganze Zahl für das Byte – ohne Warnung.

### `next()` und `align()` als Ausdruck

```asm
 *= $c001
tabelle = next(256)        ; nächste Seitengrenze ab hier: $c100
 .word tabelle
;= 00 c1
```

Das Gegenstück als Direktive ist `.page` (siehe [Direktiven](help:asm-directives)).

## Eigene Funktionen: `.function`

Sich wiederholende Rechnungen packst du in eine eigene Funktion. Sie besteht aus **einem** Ausdruck:

```
.function name(parameter, …) = ausdruck
```

```asm
.function lerp(a, b, t) = a + (b - a) * t / 255     ; Mischung zwischen a und b, t = 0..255
.function bildschirm(zeile) = $0400 + 40 * zeile

        .byte lerp(0, 200, 128)
;= 64
        .word bildschirm(3)
;= 78 04
```

Regeln:

- Die Funktion gilt ab der Zeile, in der sie steht.
- Sie sieht ihre Parameter, Konstanten, Variablen, eingebaute und früher definierte eigene Funktionen.
- **Keine Rekursion** (die Funktion ruft sich selbst), keine doppelten Namen und keine Namen der eingebauten Funktionen – das sind Fehler (**E204**, bei Rekursion **E202**).
- Mehrere Anweisungen in einer Funktion („Block-Funktion“) gibt es nicht. Brauchst du das, nimm ein [Makro](help:asm-macros).
- Die Argumentzahl muss stimmen (**E202**).

```asm fehler
.function f(a) = a * 2
 .byte f(1, 2)
```

```text
error[E202]: f() erwartet 1 Argument(e), erhalten 2 in 'f(1, 2)'
```

## Listen

Eine **Liste** ist eine Folge von Zahlen in eckigen Klammern. Listen gibt es nur beim Übersetzen; sie landen in `.var`-Variablen und werden mit `.byte`, `.word` & Co. ausgegeben.

| Schreibweise | Bedeutung |
|---|---|
| `[1, 2, 3]` | Liste mit drei Elementen; `[]` ist leer |
| `t[0]` | Element (Zählung ab 0); `t[-1]` ist das letzte |
| `t[1:3]` | Ausschnitt ab Index 1 **bis ausschließlich** 3; `t[:2]` die ersten zwei, `t[2:]` ab Index 2 |
| `a + b` | zwei Listen hintereinander |
| `[0] * 8` | Liste wiederholen |
| `len(t)` | Länge |
| `range(3)` | `[0, 1, 2]`; auch `range(a, b)` und `range(a, b, schritt)` |

```asm
.var t = [10, 20, 30]
.byte t                          ; alle Elemente
.byte t[1], t[-1], len(t)
.byte t[1:3]
.byte [0] * 3 + [7]
;= 0a 14 1e 14 1e 03 14 1e 00 00 00 07
```

- Listen können Listen enthalten; bei der Ausgabe werden sie flach aufgelöst.
- Eine Liste ist **keine Zahl**: `lda #t` oder `t + 1` sind Fehler (**E202**). Nur `Liste + Liste` und `Liste * Zahl` gehen.
- Einzelne Elemente ändern (`t[1] = 5`) geht nicht. Baue die Liste neu auf, zum Beispiel mit `t = t[:1] + [5] + t[2:]`.
- Ein Index außerhalb der Liste ist **E202**.
- `.const` nimmt nur Zahlen; Listen gehören in `.var`.

Eine Liste baust du in einer Schleife auf (hier für eine Tabelle, die auch zum Weiterrechnen taugt):

```asm
.var quadrate = []
.for n in range(5)
    quadrate += [n * n]
.next
tab:    .byte quadrate
;= 00 01 04 09 10
```

Siehe auch: [Direktiven](help:asm-directives) (`.var`, `.for`, `.repeat`, `.if`), [Spickzettel](help:asm-index).
