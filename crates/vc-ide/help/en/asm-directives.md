# Assembler: Directives

A **directive** is an instruction to the *assembler* itself - not a command for the processor. For example, it says where the program should be in memory (`.org`), stores data (`.byte`) or repeats a block (`.repeat`). Directives start with a dot, stand alone on a line (a label may come before them) and either produce bytes or not.

```
label:  .directive  argument, argument     ; comment
```

- Spelling: upper and lower case do not matter (`.BYTE` = `.byte`).
- Comments start with `;` (or `//`).
- You separate arguments with commas; each argument may be an [expression](help:asm-expr).
- An unknown directive reports **E208**.

Lines of the form `;= 01 02` in the examples show the bytes the assembler produces. All examples are checked with the assembler; examples that run only with additional files name the files. If you point the mouse at a directive in the editor, the short description appears. The [cheat sheet](help:asm-index) serves as an overview.

| Group | Directives |
|---|---|
| Address and memory | `.org` `*=` `.align` `.fill` `.lohitable` `.relocate` `.endrelocate` |
| Storing data | `.byte` `.word` `.retaddr` `.be16` `.be32` `.dword` `.hex` `.incbin` (text: [Text and character sets](help:asm-text)) |
| Constants and variables | `=` `.const` `.equ` `.var` `.set` |
| Conditional assembly | `.if` `.elseif` `.else` `.endif` `.ifdef` `.ifndef` `.assert` `.error` `.warning` |
| Repeating | `.repeat` `.for` `.next` `.break` `.continue` |
| Files | `.include` `#include` `.incbin` |
| Running the program | `.basicstub` `.run` |
| Counting cycles | `.cycles` `.endcycles` |
| Jumps | `.branchlong` |
| Target system | `.target` `.cpu` |
| Macros, scopes, structures | [own page](help:asm-macros) |
| Segments, banks, cartridges | [own page](help:cartridges) |

## Address and memory

### `.org` and `*=` - set the address

```
.org address
*= address          ; same meaning (also  * = address)
```

Sets the **program counter** (the address where the next command or the next data byte is located) and starts a new **segment**. A segment is a contiguous block of bytes with a fixed start address.

- Before the first `.org` there must be **no code and no label**, otherwise **E300** ("code before .org").
- You may use `.org` several times. When writing the output, the assembler sorts the segments by address.
- With the PRG format (C64 and similar), everything becomes **one** block from the lowest address; the assembler fills gaps in between with zero bytes. With Atari XEX, each segment stays a block of its own.
- If two segments overlap, the assembler reports **E301**.
- `address` must be 0 to 65535, otherwise **E203**.

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
error[E300]: code before .org
```

Two separate areas:

```asm
.org $c000
        .byte 1
.org $c010
        .byte 2
;=* 01
```

`.org next(1024)` jumps to the next address that is divisible by 1024 (see [next()](help:asm-expr)). The difference from `.align`: `.org` leaves a gap, `.align` fills it with bytes.

### `.align` - align to a multiple

```
.align n               ; fill with zero bytes
.align n, fillbyte     ; fill with a fill byte, for example .align 4, $ea
.align n, skip         ; skip without fill bytes, new segment
```

All three forms move the program counter to the next multiple of `n`. If the address already fits, it stays.

A **page** is 256 bytes, that is, the addresses `$xx00`-`$xxFF`. The 6502 is sensitive to page boundaries: a jump or an indexed access across the boundary costs one more cycle. You therefore place tables and loops that must not have this at the start of a page: `.align 256, skip`.

**`.align n`** fills with **zero bytes**. The fill bytes are in the same segment - useful if the area must be contiguous in the cartridge image.

```asm
*= $c001
        nop
        .align 4
        .byte 9
;= ea 00 00 09
```

**`.align n, fillbyte`** fills with the given byte (an expression from -128 to 255, otherwise **E203**). With `$ea` you pad with `nop`:

```asm
*= $c001
        rts
        .align 4, $ea
        .byte 9
;= 60 ea ea 09
```

**`.align n, skip`** starts a new segment at the next boundary, **without producing fill bytes** (like `.org next(n)`). `n` must be at least 1 (otherwise **E203**).

```asm
*= $c001
        nop
        .align 256, skip
tab:    .byte 7
        .word tab
;=* ea
```

`skip` is a keyword only at this position. This form used to be called `.page` (`.page` = `.align 256, skip`, `.page n` = `.align n, skip`); the old name still works.

### `.fill` - fill bytes and create tables

```
.fill count
.fill count, expression
.fill count, expression, loopvariable
```

Stores `count` bytes (0 to 65536). In the `expression`, the **loop variable** (default `i`, counts from 0) stands for the number of the byte. Without an expression, the bytes are zeros. The loop variable exists only there.

```asm
.fill 3, $ea                 ; three bytes $EA
.fill 4, i * 2               ; 0, 2, 4, 6
.fill 3, k + 10, k           ; another name for the loop variable: 10, 11, 12
;= ea ea ea 00 02 04 06 0a 0b 0c
```

Useful for placeholders (`puffer: .fill 256`) and small tables. For curves, use `.repeat` or `.for` with decimal numbers (see [Expressions](help:asm-expr)).

### `.lohitable` - address tables for the 6502

```
.lohitable count, expression [, loopvariable]
```

Stores a table of 16-bit values the way the 6502 reads them most conveniently: **first all low bytes, then all high bytes**. The access is then `lda tab,x` and `lda tab+count,x`. Values outside 16 bits report **E203**.

```asm
zeilen:  .lohitable 25, $0400 + 40 * i    ; start addresses of the 25 screen rows
;=* 00 28 50 78
        lda zeilen,x                     ; low byte of row x
        sta $fb
        lda zeilen+25,x                  ; high byte
        sta $fc
```

### `.relocate` / `.endrelocate` - code for another address

The **pseudo PC** is a "false" program counter: the assembler stores the bytes where they are in the file, but calculates all addresses and labels as if they were somewhere else. You need this if code is **copied** to another place at run time - for example a routine from a cartridge (ROM) into RAM, a program into the zero page or an overlay.

```
.relocate address
...
.endrelocate
```

- Between the two directives, labels and `*` use the **run-time address** `address`.
- The bytes are still stored at the **real** location; after `.endrelocate` it continues there.
- A label **before** `.relocate` (also in the same line) has the real address.
- Before `.org` it is an error (**E300**). Nested pseudo-PC sections do not exist: `.endrelocate` always returns to the real address.

```asm
start:  ldx #0
copy:   lda image,x          ; image = real address in the program
        sta $0200,x          ; ... copies to the run address
        inx
        cpx #image_end - image
        bne copy
        jmp $0200
image:
        .relocate $0200
lauf:   inc $d020
        jmp lauf             ; lauf = $0200, not the real address
        .endrelocate
image_end:
;=* a2 00 bd 10 c0
```

The example copies the routine from `image` to `$0200` and jumps there. The length `image_end - image` is calculated with real addresses; `lauf` is `$0200`. An example for cartridges is in [Segments, banks, cartridges](help:cartridges).

## Storing data

### `.byte` / `.db` - bytes

```
.byte value, value, "text", ...
```

Each argument is an expression (-128 to 255, otherwise **E203**) or a string in quotation marks. A string gives the characters **unchanged as ASCII**; characters outside ASCII (such as German umlauts) become UTF-8 bytes. For the C64 screen or PETSCII, use [.petscii, .screencode or .text with .encoding](help:asm-text). A list supplies all of its elements.

```asm
.byte 1, $ff, -1, "AB", 'C', 2 + 3
;= 01 ff ff 41 42 43 05
```

A string has no escape sequences such as `\n`; you write a quotation mark as a number: `.byte "say ", 34, "hello", 34`.

### `.word` / `.dw` - 16-bit values

```
.word value, value, ...
```

Stores each value as two bytes, **low byte first** ("little endian" - this is how the 6502 reads addresses). Allowed: -32768 to 65535, otherwise **E203**. Typical for tables with addresses.

```asm
start:  .word start, $1234, -1
;= 00 c0 34 12 ff ff
```

### `.retaddr` - address minus one (for `rts` jumps)

```
.retaddr target, target, ...
```

Stores a word with `target - 1`. The 6502 command `rts` jumps to the address on top of the stack **plus one**. If you put an address minus one on the stack (low and high with `pha`) and execute `rts`, the processor jumps there - this is the classic jump table. `.retaddr` does the "minus one" for you.

```asm
*= $c000
tabelle: .retaddr eins, zwei
eins:   nop
zwei:   nop
;= 03 c0 04 c0 ea ea
```

`eins` is at `$c004`, so the word contains `$c003` (bytes `03 c0`); `zwei` is at `$c005`, so the word contains `$c004`.

### `.be16` / `.be32` - 16 and 32 bits, high byte first

Like `.word`, but in "big endian" order, as some file formats and hardware (for example the Atari SIO bus or network data) expect it.

```asm
.be16 $1234
;= 12 34
```

`.be32` stores four bytes, also high byte first (value range -2147483648 to 4294967295, otherwise **E203**) - for tables and file formats in big-endian order.

```asm
.be32 $12345678
;= 12 34 56 78
```

### `.dword` - 32 bits

Four bytes, low byte first. Value range -2147483648 to 4294967295 (otherwise **E203**). The old name `.long` still works, but **stores four bytes** (in some other assemblers `.long` is 24 bits wide); so it is better to write `.dword`.

```asm
.dword $12345678
;= 78 56 34 12
```

### `.hex` - bytes from hex digits

```
.hex "aa bb 0c", ...
```

Pairs of hex digits; spaces do not matter. An odd number of digits reports **E208**. Useful for data pasted from a hex dump.

```asm
.hex "de ad be ef"
;= de ad be ef
```

### `.incbin` - include a binary file

```
.incbin "file" [, offset [, length]]
```

Inserts the content of a file unchanged. `offset` skips bytes at the start, `length` limits the number (otherwise up to the end of the file). The assembler searches first in the folder of the calling source file, then in the include folders (`-I`).

- File not found: **E212**. Range larger than the file: **E203**.
- A `.prg` file starts with the **load address** (2 bytes). If you want only the data, write `.incbin "bild.prg", 2`.
- You include sprites, character sets, screens and songs from the project with their file (`.include "graphics/hero.vcsprite"`); the assembler converts them into assembler text, not into a binary file (see below: including media files). With `as name` the labels live in a namespace of their own.

Example with a file `daten.bin` (16 bytes: 00 01 ... 0f):

```asm
.incbin "daten.bin", 4, 3          ; three bytes from offset 4
;= 04 05 06
.incbin "daten.bin", 14            ; from offset 14 to the end
;= 0e 0f
```

## Constants and variables

There are two kinds of named numbers that you should tell apart:

| | Constant | Variable |
|---|---|---|
| Syntax | `NAME = expression` or `.const NAME = expression` (`.equ` is the same) | `.var name = expression` |
| Change | no - setting it a second time with a **different** value is **E204** | yes: `name = ...`, `.set name = ...`, `name += ...` |
| Forward reference | yes (may be used **before** the definition) | no (valid from the line where it is set) |
| Can hold | integers (and addresses) | numbers, decimal numbers, lists, addresses |
| Purpose | names for fixed values (addresses, colors, sizes) | counters and intermediate values during assembly |

### `NAME = expression`, `.const`, `.equ`

```asm
BORDER = $d020                       ; constant
.const FARBE = 5
        lda #FARBE
        sta BORDER
;= a9 05 8d 20 d0
```

Whether you use `.const` or the short form is a matter of taste - `.const` stands out more when reading, the short form matches the usual syntax of other assemblers. If `NAME` was created with `.var` before, `NAME = ...` is an assignment to the variable.

```asm fehler
K = 1
K = 2
```

```text
error[E204]: symbol 'K' defined twice
```

### `.var` and `.set` - variables

```
.var name = expression      ; create
name = expression           ; change
name += expression          ; also -=  *=  /=  %=  &=  |=  ^=  <<=  >>=
.set name = expression      ; change (same syntax as .var)
```

`.set` on a name that was not created with `.var` is **E216**. Variables follow the [scopes](help:asm-macros): a variable with the same name in a `{ ... }` block hides the outer one.

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
error[E216]: variable 'K' was not created with .var
```

Variables are mainly useful with loops, see Repeating.

## Conditional assembly

With conditions you decide **during assembly** which source text becomes part of the program at all - for example debug output or variants for different systems. Lines that are not selected produce no bytes and define no labels.

### `.if`, `.elseif`, `.else`, `.endif`

```
.if expression
    ...
.elseif expression
    ...
.else
    ...
.endif
```

If the expression is not 0, the block is assembled. The first branch with a true condition applies. `.if` may be nested and may appear inside macros and loops. An `.if` without `.endif` reports **E210**.

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

The condition should use only values that are already known before the line (constants, variables, `-D` values). If it depends on an address further down in the source, the result can vary between passes.

### `.ifdef` and `.ifndef` - is a name defined?

```
.ifdef name
.ifndef name
```

Checks whether `name` (constant, label, variable) **is already defined in this pass**. A name that is defined only further down counts as not defined. Together with `-D` on the command line (`vcc asm ... -D DEBUG=1`) or `"defines"` in the project file, you switch between variants:

```asm D=DEBUG=1
.ifdef DEBUG
        inc $d020          ; only in the debug variant
.endif
;= ee 20 d0
```

```asm
.ifndef DEBUG
        nop                ; without -D DEBUG
.endif
;= ea
```

If you also want to check the *value* (`-D DEBUG=0`), use `.if DEBUG`. `.ifdef` asks only whether the name exists.

### `.assert`, `.error`, `.warning` - your own checks and messages

```
.assert condition [, "message"]
.error "text"
.warning "text"
```

- `.assert` stops with **E207** if the condition is 0. Good for fixing assumptions in place ("table fits in one page").
- `.error` always reports **E207**; meant for impossible branches in `.if`/`.else`.
- `.warning` only produces the warning **W200**; the program is still built.

```asm fehler
tab:    .fill 300
        .assert * - tab <= 256, "Table is larger than one page"
```

```text
error[E207]: assertion failed: "Table is larger than one page"
```

```asm warnung
.warning "not finished yet"
        nop
```

```text
warning[W200]: not finished yet
```

```asm fehler
SYSTEM = 3
.if SYSTEM == 1
        nop
.elseif SYSTEM == 2
        nop
.else
        .error "SYSTEM must be 1 or 2"
.endif
```

```text
error[E207]: SYSTEM must be 1 or 2
```

## Repeating

### `.repeat` - insert a block n times

```
.repeat count [, variable]
    ...
.endrepeat
```

The block is assembled `count` times (0 to 65536, otherwise **E203**). With `variable`, each pass gets its number (from 0); the variable is valid only in the block. This is how you "unroll" loops: the processor no longer has to run through them.

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

**Pitfall labels:** if a normal label is in the loop, it is created anew in every pass -> **E204** ("defined twice"). Solve this with a scope `{ ... }` in the block (see [Scopes](help:asm-macros)):

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

`.break` and `.continue` also apply in `.repeat`.

### `.for` / `.endfor` - loop with counter or list

The end is called `.endfor`; the old name `.next` works as well (also mixed). There are three forms:

```
.for i = 0, i < 10, i += 3          ; start, condition, step
.for i in range(3)                  ; numbers 0, 1, 2  (also range(a, b) and range(a, b, step))
.for c in [2, 5, 7]                 ; elements of a list
    ...
.endfor
```

```asm
.for i = 0, i < 10, i += 3
        .byte i                     ; 0, 3, 6, 9
.endfor
;= 00 03 06 09
.for i in range(3)
        .byte i
.endfor
;= 00 01 02
.for c in [2, 5, 7]
        .byte c * 2
.endfor
;= 04 0a 0e
```

- `.break` leaves the innermost loop, `.continue` jumps to the next pass. Neither applies across a macro that is called inside the loop; outside a loop they report **E210**.
- The loop variable is valid only in the loop. If it had a value before, that value returns afterwards.
- At most 65536 passes, otherwise **E217**. `range` counts backwards if the step is negative; step 0 is an error.

```asm
.for i in range(10)
        .if i == 2
        .continue                   ; skip 2
        .endif
        .if i == 5
        .break                      ; stop at 5
        .endif
        .byte i
.endfor
;= 00 01 03 04
```

**`.repeat` or `.for`?** `.repeat 8, i` is the shortest form for "n times". You need `.for` if the counter runs differently (step size, backwards, elements of a list) or if you want `.break` / `.continue` with your own variable. Both produce the same result.

## Including files

### `.include` and `#include`

```
.include "file"
.include "file" as name
#include <path>
```

Inserts the content of another source file at this place, as if it were written here.

| Syntax | Searched in | Several times |
|---|---|---|
| `.include "file"` | the folder of the calling file, then the include folders (`-I`) | inserted anew each time |
| `#include "file"` | as above | **only once** per file and namespace |
| `#include <path>` | only the include folders: the [system library](help:asm-library) | only once |

`#include` behaves like `#pragma once`; this way libraries may include each other without their names being defined twice. If the same file is fetched twice with `.include`, the assembler reports the names as defined twice (**E204**).

**Included files in the editor:** a file that is included anywhere in the project is not checked by the editor on its own, but in the context of the including file - with its marks, constants, macros, `.target` and `.org`, and with the **unsaved** text of all open files. This way there are no follow-up errors such as "unknown symbol" just because the mark is only in the main file. Which file is the context:

| Case | Context |
|---|---|
| Start program (`main` of the project file) or not included by any file | the file itself, as always |
| included, and the start program reaches it (also through several levels) | the start program |
| included, the start program does not reach it | the topmost including file with the shortest include chain, in a tie the first by path |
| only a cycle of files that include each other, without an entry point | the file itself; the assembler reports the cycle (**E211**) |

Errors of the including file appear in its own file, even if only the included file is open. Line addresses, hover, go to definition and completion come from the same run. A file **without** `.org`, `.target` and `.basicstub` that includes nothing and is not in the project file (an "orphan", probably a library) shows missing symbols, macros and `.org` only as a **hint** ("probably defined in the including file") instead of as an error; all other errors stay errors.

With `as name`, the content of the file goes into a **namespace** of its own (see [Scopes](help:asm-macros)): from outside, the names are called `name.symbol`. This way you can include the same library several times under different names (two pieces of music, two driver instances).

Example with the file `lib.inc`:

```
; lib.inc
SPEED = 3
spiele: lda #SPEED
        rts
```

```asm
.include "lib.inc"                 ; names directly visible
        jsr spiele
        .byte SPEED
;= a9 03 60 20 00 c0 03
```

```asm
.include "lib.inc" as m1           ; two copies under different names
.include "lib.inc" as m2
        jsr m1.spiele
        jsr m2.spiele
        .byte m1.SPEED
;= a9 03 60 a9 03 60 20 00 c0 20 03 c0 03
```

- File not found: **E212**. More than 16 levels deep (often a cycle): **E211**.
- An invalid name after `as` reports **E208**.
- **Macros** stay global even with `as name`: you still call a macro from the file simply with `macroname`.
- If `.include` is in an `.if` branch that does not apply, the file is not assembled either.

### Including media files

You include a media file from the editors (extension `.vcsprite`, `.vcpmg`, `.vcchar`, `.vcscreen`, `.vctiles`, `.vcpal`, `.vcsong` or `.vcplay`) by its name, just like a source file:

```
.org $2000
.include "graphics/hero.vcsprite"
```

You do not have to export anything. When building, the assembler converts the medium into assembler text itself (data, labels and constants such as `hero_sprite`; for sprites with animations, the tables are in the same text) and puts it in place of the line. `#include` and `as name` work in the same way. With `.org` in front, you set where the data goes (for example a multiple of 64 for a C64 sprite).

- For speed, the result is kept in the cache `build/.media-cache/`; there is also a readable copy `<name>.asm` there. You can delete the folder at any time.
- If the medium belongs to a different target system than that of the build, the file is not a medium or it cannot be converted, the assembler reports **E219** with the reason. If the file is missing, it reports **E212**.
- The **old way** through `build/export/hero.asm` (and `hero.anim.asm`) still works but warns with **W208** and names the media file that you should include instead. A line with `hero.anim.asm` after the include is no longer needed: the media file supplies data and tables together. `vcc migrate includes` converts old projects (without `--apply` it only shows what would change).
- In the export dialog (Cmd/Ctrl+Shift+E), **Insert as include** inserts the line into the open source (see [Export](help:export)).

*[Bild folgt in E13: include-eingefuegt]*

## Running the program

### `.basicstub` - the BASIC start line

A **stub** is a tiny piece at the start. On the C64 and similar systems, you normally start machine programs with `RUN`. But `RUN` runs BASIC - so your program needs a small BASIC line at the very start that jumps into the machine program:

```
10 SYS 2061
```

This is exactly what `.basicstub` creates: a ready-encoded BASIC line with the `SYS` command (a call of a machine program at an address). Without this line, the user would have to type `SYS 2061` themselves.

```
.basicstub target [, linenumber] [, "comment"]
```

| Argument | Meaning |
|---|---|
| `target` | address where the machine program starts; almost always a label (`start`) |
| `linenumber` | BASIC line number 0 to 63999, default **10** |
| `"comment"` | is appended as a `REM` line after a `:` (in uppercase); the whole line may be at most 200 bytes long (otherwise **E203**) |

The BASIC interpreter looks for the program at a fixed address that depends on the system. `.basicstub` knows it from the target data of the project and puts the stub there **by itself, without a preceding `.org`**. So you simply write `.basicstub start` at the start of the file:

| System | BASIC start address |
|---|---|
| C64 | `$0801` |
| C128 | `$1C01` |
| C16/Plus/4 | `$1001` |
| VIC-20 | `$1001` (basic machine), `$1201` (with 8 KB or more of expansion), `$0401` (3 KB) - depending on the memory configuration of the project |
| MEGA65 | `$2001` |

The start address is also the load address at the start of the PRG file. Without target data (for example when you call the assembler without a project and without `--target`), `$0801` applies. The stub takes 12 bytes (more with a comment); your program starts right after it. The target is written into the line as a decimal number; if the assembler does not yet know the label in the first pass, it temporarily uses 2061 and corrects it in the next pass.

```asm roh fmt=prg
.basicstub start
start:  rts
;= 01 08 0b 08 0a 00 9e 32 30 36 31 00 00 00 60
```

The first two bytes (`01 08`) are the load address `$0801` of the PRG file, then the stub follows. Its bytes: `0b 08` pointer to the next BASIC line (here: end of program), `0a 00` line number 10, `9e` the token (abbreviation) for `SYS`, `32 30 36 31` the text "2061", then three zero bytes (end of line and end of program). The program starts at `$080D` = 2061.

With line number and comment:

```asm roh
*= $0801
.basicstub start, 1000, "test"
start:  rts
;= 12 08 e8 03 9e 32 30 36 38 3a 8f 20 54 45 53 54 00 00 00 60
```

This gives `1000 SYS2068:REM TEST`. In the program, there is no space between the token `9e` (SYS) and the digits; `LIST` therefore also shows `SYS2068`. A space appears only after `REM` (`20` before the text). The notation `10 SYS 2061` with a space in this help is just the usual reading version. `8f` is the token for `REM`. The comment is for screen text and credits; it makes the stub longer, so `start` is `$0814` = 2068 here.

On another system, just change the address:

```asm roh fmt=prg target=c128
.basicstub start
start:  rts
;= 01 1c 0b 1c 0a 00 9e 37 31 38 31 00 00 00 60
```

A `.basicstub` with a fixed address instead of a label also works (`.basicstub $0810`); then your code must start there. An invalid line number or a line that is too long reports **E203**.

**With your own `.org`:** if there is an `.org` before `.basicstub`, that address applies. If it does not match the BASIC start address of the target system, the assembler warns with **W204**, because BASIC does not find the line there:

```asm warnung roh target=c128
.org $0801
.basicstub start
start:  rts
```

```text
warning[W204]: .basicstub is at $0801, the BASIC start address of the target system is $1C01 (BASIC will not find the line there)
```

There is no warning if `.cartridge` comes before it (a cartridge does not start through BASIC) or if the stub is in a `.relocate` section. The stub must be at the start of the program: if bytes were already output before it, the assembler reports **E218**.

```asm fehler roh
.org $0801
        nop
.basicstub start
start:  rts
```

```text
error[E218]: .basicstub must be at the start of the program (bytes were already output before it)
```

**Target without a BASIC line:** the Atari has no BASIC line with `SYS`. A `.basicstub` still puts a line into the file there, which nobody runs; the assembler reports **W205**, the output stays unchanged. To start the program, use `.run label`.

```asm warnung roh target=atari8
.org $2000
.basicstub start
start:  rts
.run start
```

```text
warning[W205]: .basicstub has no effect on this target (no BASIC with SYS); use .run label to set the start
```

When do I use what?

- The program should run with `RUN` -> `.basicstub start` at the start of the file, without `.org` (the target system determines the address).
- You do not need a start through BASIC (cartridge, Atari, program is called by a loader) -> no stub.
- Change the line number if the program loads more BASIC lines or the user should see `LIST`.

### `.run` - start address (Atari)

```
.run address
```

Remembers the start address of the program. On the Atari XEX, the assembler writes it as a segment of its own to `$02E0` (RUNAD: the Atari jumps there after loading); on the Atari cartridge, into the start vectors at `$BFFA`. On the C64, `.run` has no effect - use `.basicstub` there; the assembler points this out with **W206** (see below).

```asm fmt=xex target=atari8
*= $2000
start:  jmp start
.run start
```

The hints about the target are warnings; the output stays as it was:

- **W206** - `.run` is on a target without a start vector (for example C64, MEGA65, Plus/4): the line has no effect; use `.basicstub label`. For a cartridge (`.cartridge`) there is no message; a cartridge starts through its own vectors.
- **W207** - an Atari program contains commands but no `.run`: the XEX file has no start vector and does not start by itself; add `.run label`. There is no message for cartridges and for pure data files without commands.

```text
warning[W206]: .run has no effect on this target (a PRG has no start vector); use .basicstub label to set the start
warning[W207]: No start vector set; add one with .run label
```

## Counting cycles: `.cycles`

The 6502 works to a clock. Each command needs a fixed number of **cycles** (clock ticks): `nop` 2, `lda $d020` 4, `jsr` 6. On the PAL C64, a raster line (one screen line) has 63 cycles. If you program raster bars, music routines in the interrupt or tricks with exact timing, you must know whether your code fits into this time.

`.cycles` and `.endcycles` enclose a part of the source text and let the assembler **add up** the cycles of the commands in it. The directive produces no code and changes nothing in the program.

```
.cycles [name] [max=budget]
    ...
.endcycles
```

| Part | Meaning |
|---|---|
| `name` | freely chosen name of the block (one word); appears in messages and in the test. Without a name it is empty |
| `max=budget` | **budget** (allowed maximum) in cycles, an expression. If the worst case exceeds the budget, there is the warning **W201** - the build still succeeds |

**What is counted?** The assembler adds the cycles of each command that is **between the two lines in the source text**, exactly once - in the order of the source text. It does not run the program and does not follow jumps. This has consequences:

- A loop (`dex` / `bne`) counts only *one* pass, not eight.
- Macros and `.repeat` count as often as they are assembled (3 x `nop` from `.repeat 3` are 6 cycles).
- Lines in `.if` branches that are not selected do not count; data (`.byte`) does not count.
- `jsr` counts 6 - the cycles of the called routine do not.
- A forward jump (`bne weiter`) still counts all commands in between.

This produces **two values**:

| Value | Meaning |
|---|---|
| **min** | sum of the base cycles of all commands. Matches the best case, if no jump is taken and no page boundary is crossed |
| **max** | every conditional jump as *taken* **and** across a page boundary (+2), every indexed read access with a page crossing (+1) - the worst case |

A **page boundary** is every 256 bytes: a jump or an access such as `lda tab,x` that reaches across it takes one cycle longer.

View the result:

- in the [build log](help:build), there is one line per block, `Cycles "name": min-max (budget n)`;
- the debug file `.vcdbg` contains the blocks (`cycleBlocks`);
- a test with `"check": "cycles", "block": "name"` checks the block with `vcc test` (see [Tests](help:tests)). There the built code is *really traced* (all paths up to `rts`/`rti`); it reports loops and `jsr` as "not determinable". `.cycles` in the source text, in contrast, is the simple sum.

Example: an interrupt block with a budget. The budget fits, there is no message.

```asm
.cycles irq max=24
        inc $d020          ; 6
        lda $d012          ; 4
        sta $d021          ; 4
        rts                ; 6  -> total 20
.endcycles
;= ee 20 d0 ad 12 d0 8d 21 d0 60
```

If the worst case exceeds the budget, the assembler warns (line of the `.endcycles`):

```asm warnung
.cycles langsam max=10
        lda $d020          ; 4
        sta $d021          ; 4
        jsr $ffd2          ; 6
.endcycles
```

```text
warning[W201]: cycle budget 'langsam' exceeded: worst case 14 > 10 cycles
```

A loop counts only once; the block `s` also shows the difference between min and max (min 16, max 19):

```asm
.cycles s max=20
        .repeat 3
        nop                ; 3 x 2 = 6
        .endrepeat
        lda $1234,x        ; 4, max 5
        beq weiter         ; 2, max 4 (taken + page boundary)
        nop                ; 2
weiter: nop                ; 2
.endcycles
```

Blocks may be nested; the outer one also counts the commands of the inner one. Typical errors:

| Message | Cause |
|---|---|
| **E210** `.endcycles without .cycles` | closing line without a start |
| **E210** `.cycles 'x' without .endcycles` | block not closed |
| **W201** | worst case greater than `max=` |

When do I use what?

- **Quick safeguard in the source text** ("my interrupt handler may take at most 40 cycles"): `.cycles name max=40`.
- **Exact number for branching code, measuring in the emulator**: `vcc test` with `cycles` (path analysis) or cycle measurement in the emulator, see [Tests](help:tests).
- No counter takes into account deductions caused by DMA (bad lines, sprites), interrupts and wait states.

## Long jumps: `.branchlong`

Conditional jumps (`beq`, `bne`, ...) reach only -128 to +127 bytes. A jump that is too far is normally the error **E203**.

```
.branchlong auto        ; extend jumps that are too far by itself (also: on)
.branchlong off         ; back to the default
```

With `auto`, the assembler replaces a jump that is too far with a long form and reports this with the warning **W203**:

| CPU | `beq target` becomes |
|---|---|
| 6502 and related | `bne *+5` + `jmp target` (5 bytes) |
| CPU with `bra` but without a long jump | `bra target` becomes `jmp target` (3 bytes) |
| 45GS02 (MEGA65) | `lbeq target` (3 bytes, a real long jump) |

The length changes the addresses after it; the assembler repeats the passes until everything fits. Jumps that were extended once stay long. For `.cycles`, the long form counts 3 to 5 cycles (6502).

```asm warnung
.branchlong auto
        beq weit
        .fill 200, $ea
weit:   nop
```

```text
warning[W203]: branch too far for BEQ: long form (5 bytes) used
```

## Target system: `.target` and `.cpu`

### `.target`

```
.target c64
```

Writes the target system into the source text (`c64`, `c128`, `plus4`, `vic20`, `atari8`, `mega65`). The **assembler itself ignores the line**; the tools read it: if the target in the IDE is set to "automatic", the build uses this target (in the build log: "from .target"). In a project (`.vcprj`), the project file determines the target; on the command line, `--target`.

### `.cpu`

```
.cpu 6510 { illegals = warn }
```

Only the setting `illegals = deny | warn | allow` has an effect (handling of **undocumented commands** such as `lax` or `dcp`: not allowed, allowed with warning **W104**, allowed). The assembler does not read the CPU identifier before it (`6510`): the CPU follows from the target system. The default is what is set in the project or `--illegals`. Commands that are considered *unstable* (`xaa`) always warn with **W105**; a use that is not allowed reports **E201**.

```asm fehler illegals=deny
 lax $d020
```

```text
error[E201]: undocumented opcode LAX (illegals=deny)
```

## Old names

Earlier spellings stay **equivalent and silent**: they give the same bytes and the same messages, and the assembler does not warn about them. In the editor, a quick fix offers to replace them with the new name. New names need version 0.3.0 or later; older versions report **E208** for them.

| New name | Old name (equivalent) |
|---|---|
| `.relocate` ... `.endrelocate` | `.pseudopc` ... `.endpseudopc` or `.realpc` |
| `.retaddr` | `.rta` |
| `.lohitable` | `.lohifill` |
| `.screencode` | `.screen` |
| `.textnull`, `.textlen`, `.textshift` | `.null`, `.ptext`, `.shift` |
| `.align n, skip` | `.page` (= `.align 256, skip`), `.page n` |
| `.endfor` | `.next` |
| `.be16` | `.dbyt` |
| `.dword` | `.long` (**four** bytes; often 3 bytes in other assemblers) |
| `.byte`, `.word`, `.const` | `.db`, `.dw` and `.addr`, `.equ` |

Directive names are never translated; they are the same in every language.

Continue with: [Macros, scopes and structures](help:asm-macros), [Text and character sets](help:asm-text), [Cheat sheet](help:asm-index).
