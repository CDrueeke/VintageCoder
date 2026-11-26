# Assembler: Expressions and Functions

An **expression** is a calculation that the assembler does itself while it translates - not the C64 while it runs. The result is always a fixed number that is put into the program. Expressions can be used wherever a number is needed: as an operand (`lda #WIDTH * 2`), in data directives (`.byte 1 + 2`), in `.if`, `.org` and so on.

This page belongs to the [assembler manual](help:assembler). Lines of the form `;= 01 02` in the examples show which bytes the assembler produces; all examples have been checked with the assembler.

## Writing numbers

| Notation | Example | Value | Note |
|---|---|---|---|
| decimal | `255` | 255 | |
| hexadecimal with `$` | `$ff`, `$D020` | 255, 53280 | upper or lower case |
| hexadecimal with `0x` | `0x1f` | 31 | as in C |
| binary with `%` | `%1010` | 10 | only the digits 0 and 1 |
| character | `'a'` | 97 | the **ASCII** value, exactly one character; for PETSCII see [Text](help:asm-text) |
| decimal fraction | `2.5`, `1e3`, `1.5e-2` | see below | for calculating only |

Not possible: `0b1010` (binary only with `%`), underscores in numbers (`1_000`) and strings in expressions (`"AB"` exists only in data directives).

```asm
.byte 255, $ff, 0xff, %11111111, 'A'
;= ff ff ff ff 41
```

After `%` a 0 or 1 must follow right away. Otherwise the `%` is the remainder operator (`7 % 3` is 1).

## How big can numbers be?

The assembler calculates internally with large integers (64 bits with sign). Only when the value is **put in** does it check whether the value fits:

| Place | allowed | otherwise |
|---|---|---|
| `.byte`, `#value` (immediate) | -128 to 255 | error **E203** |
| `.word`, addresses, `.org` | -32768 to 65535 (addresses 0 to 65535) | error **E203** |
| `.dword` | -2147483648 to 4294967295 | error **E203** |

Negative values are stored as two's complement: `.byte -1` gives `$FF`.

```asm fehler
 lda #256
```

```text
error[E203]: value 256 outside -128..255
```

With `<` and `>` (see below) you take the low byte and the high byte out of a 16-bit value when a byte is needed.

## Symbols in expressions

| What | Example | Meaning |
|---|---|---|
| label | `start`, `.loop` | address at this place (`.name` = local label, see [Labels and scopes](help:asm-macros)) |
| constant | `WIDTH` | value from `WIDTH = 40` or `.const` |
| variable | `i`, `x` | value from `.var` (changes during translation) |
| program counter | `*` | address of the current instruction or of the current data line |
| chip register | `vic2.border` | address according to the hardware database of the target system |
| scope name | `init.loop`, `music.play` | symbol in a namespace |
| `pi` | `pi` | the circle constant 3.14159... |

The asterisk has different meanings in different places: at the **start of a line** before `=` it means set the program counter (`*= $c000`), in an **expression** it means read the program counter, and between two numbers it is the multiplication sign.

```asm
anfang: nop
        nop
laenge = * - anfang        ; 2 bytes between anfang and here
        .byte laenge
;= ea ea 02
```

On lines with an instruction, the program counter `*` is the address **before** the instruction. In a [pseudo PC](help:asm-directives) section (`.relocate`) it is the run-time address.

### Forward references

You can use symbols before they are defined: `jmp spaeter` before `spaeter:` is normal. For this the assembler translates several times (passes) until all addresses are fixed. Two consequences:

- With a forward reference, the assembler does not know the size of the value in the first pass. For `lda spaeter` it plans the long (absolute) address form to be safe. Where the length matters, you force it with `lda.z` or `lda.a` (see [addressing modes](help:assembler)).
- `.ifdef` and `.ifndef` count only symbols that were defined **before** the line (otherwise the query could change between the passes).

A symbol that does not exist anywhere reports **E202** (`unknown symbol`).

## Operators

From the highest to the lowest precedence. Within one row of the table, the calculation goes from left to right; parentheses `( )` always come first.

| Precedence | Operator | Meaning | Example -> result |
|---|---|---|---|
| 1 (highest) | `-` `+` `!` `~` `<` `>` | sign, logical not, bitwise not, low byte, high byte (only **before** a value) | `-3`, `!0` -> 1, `~0` -> -1, `<$1234` -> $34, `>$1234` -> $12 |
| 2 | `*` `/` `%` | times, integer division, remainder | `7 / 2` -> 3, `7 % 3` -> 1 |
| 3 | `+` `-` | plus, minus | `10 - 2 - 3` -> 5 |
| 4 | `<<` `>>` | shift bits left / right | `1 << 3` -> 8, `$80 >> 7` -> 1 |
| 5 | `&` | bitwise and | `$ff & $0f` -> 15 |
| 6 | `^` | bitwise exclusive or | `1 ^ 3` -> 2 |
| 7 | vertical bar | bitwise or | `$30` or `1` -> $31 (notation see below) |
| 8 | `<` `<=` `>` `>=` | comparisons (result 1 or 0) | `3 > 2` -> 1 |
| 9 | `==` `!=` | equal / not equal (result 1 or 0) | `3 == 3` -> 1 |
| 10 | `&&` | logical and | `1 && 0` -> 0 |
| 11 (lowest) | two vertical bars | logical or | `0` or `2` -> 1 (notation see below) |

The bitwise or is a single vertical bar, the logical or is two (`$30 | 1` gives $31, `0 || 2` gives 1). Comparisons and logical operators always give **1** (true) or **0** (false). As a condition (in `.if`, `.assert`, with `&&`, `||` and `!`), every value other than 0 counts as true.

```asm
.byte 2 + 3 * 4, (2 + 3) * 4, 10 - 2 - 3, 7 / 2, 7 % 3
;= 0e 14 05 03 01
.byte 1 << 3, $ff & $0f, 1 ^ 3, $30 | 1
;= 08 0f 02 31
.byte 3 > 2, 3 == 4, !0, !7, 1 || 0 && 0
;= 01 00 01 00 01
```

### Like Python, not like C

The bit operators `& ^ |` bind **more strongly** than the comparisons. So `x & 3 == 2` does what you mean - `(x & 3) == 2` - while in C it would be read as `x & (3 == 2)`:

```asm
.byte 6 & 3 == 2          ; (6 & 3) == 2  ->  2 == 2  ->  1
;= 01
```

There is no `**`, no `? :`, no `and`/`or`/`not`, and no single `=` as a comparison (write `==`). Both sides of `&&` and `||` are always evaluated (no short cut). You only notice this with a division by 0 on the right side.

### Integer division and remainder

`/` and `%` truncate toward zero, as in C: `-7 / 2` is -3, `-5 % 3` is -2. Dividing by 0 reports **E202** ("division by 0").

```asm fehler
 .byte 1 / 0
```

```text
error[E202]: division by 0 in '1 / 0'
```

## Low byte and high byte: `<` and `>`

The C64 has an 8-bit processor, but addresses have 16 bits. An address consists of a **low byte** (the lower 8 bits) and a **high byte** (the upper 8 bits): `$1234` has the low byte `$34` and the high byte `$12`. Two signs split an address:

| Notation | Value |
|---|---|
| `<value` | `value & $FF` (low byte) |
| `>value` | `(value >> 8) & $FF` (high byte) |

Typical use: put an address into two zero page cells.

```asm
ziel = $1234
        lda #<ziel          ; $34
        sta $fb
        lda #>ziel          ; $12
        sta $fc
;=* a9 34 85 fb a9 12 85 fc
```

**Caution, precedence:** `<` and `>` bind more strongly than `+`. So `>ziel + 1` means "high byte, then plus 1" (= $13), **not** "high byte of ziel+1". If you want the latter, use parentheses:

```asm
ziel = $12ff
        .byte >ziel + 1        ; ($12) + 1
        .byte >(ziel + 1)      ; high byte of $1300
;= 13 13
```

With `<` the difference is especially unpleasant: `<ziel + 1` gives `$ff + 1 = $100` here, and that does not fit into a byte (**E203**). When in doubt, always use the parentheses: `<(ziel + 1)` gives `$00`.

You can build a table of addresses with [.lohitable](help:asm-directives) or with two lists `<`/`>` (see the lists below).

## Comparisons and logic

```asm
BREITE = 80
.if BREITE > 40
    .byte 1
.else
    .byte 2
.endif
;= 01
```

Truth values are just numbers: `5 > 3` is 1, `5 < 3` is 0, `!5` is 0, `!0` is 1. This allows tricks like `farbe = (x > 10) * 2`, but it is mainly useful for `.if`, `.assert` and `.function`.

## Integers and decimal fractions

Integers stay integers: `7 / 2` is 3. As soon as a decimal fraction is involved, the calculation is exact: `7 / 2.0` is 3.5.

Where the end result is a byte or a word, the fractional part is cut off and the assembler warns with **W202**. With the functions `int()`, `round()`, `floor()` or `ceil()` you choose how to round, and then there is no warning.

```asm warnung
 .byte 7 / 2.0
```

```text
warning[W202]: floating-point number 3.5 truncated to 3 (use int(), round() or floor()) in '7 / 2.0'
```

```asm
.byte int(7 / 2.0), round(2.5), floor(-2.5), ceil(2.1)
;= 03 03 fd 03
```

The operators `% & | ^ << >> ~` and `<` and `>` (low/high byte) work only with integers. With a decimal fraction they report **E202**.

## Built-in functions

You write functions with parentheses, even without arguments, for the sake of the parentheses: `sin(x)`. Upper or lower case does not matter (`ABS(-1)`). A **label** named `max` stays a label - it is the function only when a parenthesis follows directly.

| Function | Arguments | Result |
|---|---|---|
| `sin(x)` `cos(x)` `tan(x)` | 1 | trigonometric functions, angle in **radians** (360 degrees = `2 * pi`) |
| `asin(x)` `acos(x)` `atan(x)` | 1 | inverse functions (result in radians) |
| `atan2(y, x)` | 2 | angle of the point (x, y), radians |
| `sqrt(x)` | 1 | square root |
| `pow(a, b)` | 2 | a to the power of b |
| `exp(x)` `log(x)` `log2(x)` `log10(x)` | 1 | e to the power of x, natural logarithm, logarithm to base 2 or 10 |
| `int(x)` | 1 | integer **toward zero** (2.7 -> 2, -2.7 -> -2) |
| `floor(x)` | 1 | rounded down (-2.5 -> -3) |
| `ceil(x)` | 1 | rounded up (2.1 -> 3) |
| `round(x)` | 1 | to the nearest integer, at .5 away from zero (2.5 -> 3) |
| `abs(x)` | 1 | absolute value |
| `sign(x)` | 1 | -1, 0 or 1 |
| `min(a, b, ...)` `max(a, b, ...)` | 1 or more | smallest / largest value |
| `len(list)` | 1 | number of elements in a list |
| `range(n)` `range(a, b)` `range(a, b, step)` | 1 to 3 | list of numbers (at most 65536 elements) |
| `next(n)` or `align(n)` | 1 | next address from the program counter that is divisible by `n` (stays where it is if it already fits) |

All functions except `len` and `range` need numbers, not lists. A wrong number of arguments and unknown functions report **E202**:

```asm fehler
 .byte abs(1, 2)
```

```text
error[E202]: abs() expects 1 argument(s), got 2 in 'abs(1, 2)'
```

```asm
.byte abs(-3), sign(-9), min(4, 2, 8), max(4, 2, 8)
;= 03 ff 02 08
.byte sqrt(16), pow(2, 5), log2(8), log10(1000)
;= 04 20 03 03
.byte int(sin(pi / 2)), int(cos(0)), round(atan2(1, 1) * 4)
;= 01 01 03
```

### Sine table (example for decimal fractions and `.repeat`)

A sine curve with 256 values from 1 to 255, as it is needed for sprite movements or raster bars:

```asm
sinus:
    .repeat 256, i
        .byte round(128 + 127 * sin(i * 2 * pi / 256))
    .endrepeat
;=* 80 83 86 89
```

`i * 2 * pi / 256` calculates with decimal fractions, and `round()` turns the result into the integer for the byte - without a warning.

### `next()` and `align()` as an expression

```asm
 *= $c001
tabelle = next(256)        ; next page boundary from here: $c100
 .word tabelle
;= 00 c1
```

The counterpart as a directive is `.align n, skip` (see [Directives](help:asm-directives)).

## Your own functions: `.function`

Put calculations that repeat into your own function. It consists of **one** expression:

```
.function name(parameter, ...) = expression
```

```asm
.function lerp(a, b, t) = a + (b - a) * t / 255     ; mix between a and b, t = 0..255
.function bildschirm(zeile) = $0400 + 40 * zeile

        .byte lerp(0, 200, 128)
;= 64
        .word bildschirm(3)
;= 78 04
```

Rules:

- The function applies from the line where it stands.
- It sees its parameters, constants, variables, built-in functions and your own functions defined earlier.
- **No recursion** (the function calls itself), no duplicate names and no names of the built-in functions - these are errors (**E204**, for recursion **E202**).
- Several statements in one function ("block function") do not exist. If you need this, use a [macro](help:asm-macros).
- The number of arguments must match (**E202**).

```asm fehler
.function f(a) = a * 2
 .byte f(1, 2)
```

```text
error[E202]: f() expects 1 argument(s), got 2 in 'f(1, 2)'
```

## Lists

A **list** is a sequence of numbers in square brackets. Lists exist only during translation; they end up in `.var` variables and are output with `.byte`, `.word` and so on.

| Notation | Meaning |
|---|---|
| `[1, 2, 3]` | list with three elements; `[]` is empty |
| `t[0]` | element (counting from 0); `t[-1]` is the last one |
| `t[1:3]` | slice from index 1 **up to but not including** 3; `t[:2]` the first two, `t[2:]` from index 2 |
| `a + b` | two lists one after the other |
| `[0] * 8` | repeat a list |
| `len(t)` | length |
| `range(3)` | `[0, 1, 2]`; also `range(a, b)` and `range(a, b, step)` |

```asm
.var t = [10, 20, 30]
.byte t                          ; all elements
.byte t[1], t[-1], len(t)
.byte t[1:3]
.byte [0] * 3 + [7]
;= 0a 14 1e 14 1e 03 14 1e 00 00 00 07
```

- Lists can contain lists; on output they are flattened.
- A list is **not a number**: `lda #t` or `t + 1` are errors (**E202**). Only `list + list` and `list * number` work.
- You cannot change single elements (`t[1] = 5`). Build the list again, for example with `t = t[:1] + [5] + t[2:]`.
- An index outside the list is **E202**.
- `.const` takes only numbers; lists belong in `.var`.

You build a list in a loop (here for a table that is also good for further calculation):

```asm
.var quadrate = []
.for n in range(5)
    quadrate += [n * n]
.endfor
tab:    .byte quadrate
;= 00 01 04 09 10
```

See also: [Directives](help:asm-directives) (`.var`, `.for`, `.repeat`, `.if`), [Cheat sheet](help:asm-index).
