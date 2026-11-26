# BASIC: directives, files and assets

A **directive** is an instruction to the translator, not to the machine. It stands on a line of its own and begins with `#`. In native BASIC there are seven: `#include`, `#longnames`, `#define`, `#if` with `#else` and `#endif`, `#numbering`, `#dialect` and `#asm`. This page explains them, plus including sprites, character sets and songs from the editors, machine code in a BASIC program, and what does **not** exist. It belongs to the [BASIC manual](help:basic).

## Overview

| Line | Effect |
|---|---|
| `#include "datei.bas"` | Inserts the content of the file at this place |
| `#include "graphics/hero.vcsprite"` | Includes a media file (sprite, character set, screen, song) as DATA lines |
| `#longnames` | Allows long variable names (see [Long names](help:basic-names)) |
| `#define NAME wert` | Constant: replaces `NAME` in the program with `wert` |
| `#if SYSTEM_C64` ... `#else` ... `#endif` | Translates lines only for certain systems or conditions |
| `#numbering start=N step=N` | Start and step of the line numbers of this file |
| `#dialect id` | BASIC dialect of this file (see [Dialects](help:basic-dialects)) |
| `#asm "datei.asm"` | Translates an assembler file and adds the code to the program |
| `; text` | Comment for you, does not go into the program |

You write directives in lowercase or uppercase (`#INCLUDE` works too), they stand alone on the line and have **no line number**. They cost neither memory nor run time. A line that begins with `#` but is not one of these directives is not recognized by the translator; it treats it as an ordinary BASIC line.

### Constants: `#define`

```
#define NAME wert
```

From this line on, the translator replaces every **whole word** `NAME` with `wert` before it numbers the lines. This costs neither a variable nor run time.

- Upper and lower case count: `RAHMEN` and `rahmen` are two names.
- Nothing is replaced in strings, after `REM`, in `DATA`, after `@` (labels) and in numbers such as `$FF`.
- The value may contain other names, at most eight levels deep. If a name refers to itself, the translator reports **B176**.
- There are **no macros with parameters**.
- A keyword of the dialect as a name reports **B172**; a name made of characters other than letters, digits and `_`, and a second `#define` with the same name, report **B174**.
- On the command line, `-D NAME=wert` sets a constant, as if `#define NAME wert` stood at the start.

### Conditional translation: `#if`

```
#if SYSTEM_C64
poke 53280, 0
#else
print "only for the C64"
#endif
```

- The condition is `SYSTEM_` plus the identifier of the target system in uppercase (`SYSTEM_C64`, `SYSTEM_PLUS4`, `SYSTEM_ATARI8` ...), a name set with `#define` (true if its value is not `0`), or one of the two with a leading `!` (not).
- `#else` is optional, `#endif` closes the block. Nesting is allowed.
- Excluded lines do not go into the program. Messages still point to the correct line of the source.
- Errors in the structure (`#else` or `#endif` without `#if`, a second `#else`, `#if` without `#endif`, an unreadable condition) report **B175**.

The commands of the dialects differ. So an `#if` helps mainly where only single lines differ per system. For larger differences, create one main file per system (`main_c64.bas`, `main_plus4.bas`), each in its own source group in the project, and include shared parts with `#include`.

### Numbering and dialect per file

- `#numbering start=N step=N` sets the number with which lines without a number of their own begin and the distance between them (default 10 and 10). The line in the file takes precedence over the values of the project and the command line (`--start`, `--step`). If you write only `start=` or only `step=`, you change only that value. A faulty line reports **B190**, a step below 1 or a start above the highest number of the dialect reports **B191**, and if the numbers are not enough for all lines, **B192** names a smaller step.
- `#dialect id` sets the dialect of this one file, for example `#dialect turbo_basic_xl` for a file with Turbo BASIC XL in an Atari project. The line in the file takes precedence over the setting of the project. An unknown identifier reports **B178**, an identifier that does not fit the target system reports **B170**.

## Including: `#include`

```
#include "path/datei.bas"
```

- When building, the line is **replaced by the content of the file**, as if you had typed it there. Labels, line numbers and `#longnames` of the file apply to the whole program.
- The **path** is in quotation marks and is **relative to the file that contains the line**. Included files may include further files in turn, up to 8 levels deep.
- The line must not carry a line number and must stand alone.
- Lines of the included file without a number are **numbered along**, as if they stood at the place of the `#include` line. With a number they stay as they are.
- Errors point to the **original file and line**, not to the assembled one.

Example with two files (the block with `datei=` is the helper file, the second is the main file):

```basic datei=teile/ausgabe.bas
@zeig:
print "value ="; w
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
40 print "value ="; w
50 return
```

Labels are **a single list of names for the whole program**: `@zeig` from the included file is also known in the main file. So two files must not define the same label (B111).

### Errors when including

| Code | Message | Cause |
|---|---|---|
| B120 | `#include "x": file not found ...` | wrong path, or the file is created only when building (old export path, see below) |
| B121 | `#include "x": nested too deeply or circular` | a file includes itself (through others), or more than 8 levels |

```basic fehler
print "a"
#include "gibtsnicht.bas"
```

```text
main.bas:2: error B120: #include "gibtsnicht.bas": file not found (for exported files: build or export first)
```

## Including media from the editors

Sprites, character sets, screens and songs are created in the editors (see [Sprite editor](help:sprite), [Character set editor](help:charset), [Screen editor](help:screen)). For BASIC they exist as **DATA lines**. You include the **media file itself** with `#include` and read it with `READ`:

```
#include "graphics/hero.vcsprite"
```

You do not have to export anything: when building, VintageCoder converts the medium to DATA lines itself and puts them in place of the `#include` line. For speed, the result is kept in the cache `build/.media-cache/` (a readable copy is called `<name>.data.bas` there); you can delete the folder at any time. The media file stays unchanged, even if you convert the program to labels or numbers (see [Tools](help:basic-tools)).

- A medium is included only **once** per program. A second `#include` of the same medium is skipped by the translator without a message.
- The medium must fit the target system of the build. Otherwise, or if the file is missing or is not a medium, the build reports **B193** with the reason.
- Several media side by side: each gets its label `@<name>:`. How you read them depends on the dialect (see below).
- **Line number as a constant:** For every label `<name>` there is a constant `<name>_line` that is replaced by the line number of the label when numbering. It applies only if at least one medium is included, and not in strings, `REM` and `DATA`. For media made of several parts it is called `<medium>_<part>_line` (for example `font_chars_line`).
- In pure number mode, the line that follows the medium must carry a larger number than the last line of the medium; otherwise the build reports **B100**.
- If the program together with the media is larger than the free memory, **B195** warns (checked for C64: 38911 bytes, VIC-20 without expansion: 3583 bytes, Plus/4: 60671 bytes; other systems are not checked). Then machine code with `#asm` or fewer DATA lines helps.

You do not have to type the `#include` entry. In the export dialog (Cmd/Ctrl+Shift+E), **Insert as include** inserts the line into the open source. If the format there is **BASIC DATA**, **Copy into source** copies the data, together with a read-in routine, into the source.

*[Bild folgt in E13: basic-medium-include]*

### The old way through `build/export`

Formerly the build exported every medium to `build/export/<name>.data.bas`, and you included that file. This still works but is outdated: the build warns with **B194** and names the media file that you should include instead. The command line converts old projects: `vcc migrate includes` shows what would change, and `vcc migrate includes --apply` writes it.

### Structure of the generated DATA lines

- The text begins with a comment line `REM erzeugt von VintageCoder: n Bytes` and the label `@<name>:`. The name is the name of the medium (`hero`, `font`, `title`).
- After that come `DATA` lines with 12 values each, from 0 to 255.
- If a medium consists of several parts, each part has its own label: screens `@<name>_chars:` and `@<name>_attrs:` (characters and colors), songs `@<name>_order:`, `@<name>_patterns:`, `@<name>_ins:`.
- Including turns this into normal program lines: they are numbered like all others, the labels are valid in the whole program.

### Read a sprite

On the C64, a sprite belongs to a 64-byte memory block. Block 13 is at 832 (`$0340`), the pointer for it at 2040:

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

Here the media file `hero.vcsprite` comes from the example `examples/c64/sprite_demo`.

### The order of the DATA lines: C64 and VIC-20

BASIC V2 reads `DATA` **strictly in order** through the whole program. It has no `RESTORE` with a line number. So on the C64 and VIC-20 you include the files **in the order in which you read them**:

```basic
#include "graphics/hero.vcsprite"
#include "graphics/font.vcchar"
for i = 0 to 63 : read a : poke 832 + i, a : next i
for i = 0 to 2047 : read a : poke 12288 + i, a : next i
```

`restore` without a line sets the read pointer back to the **first** `DATA`. With a single asset you need nothing more.

### On C128, Plus/4, MEGA65 and Atari: `restore @name`

These dialects know `RESTORE line`, so the label works too. The order of the files then does not matter:

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

Atari BASIC also knows `restore` with a line, so `restore @name` works there as well:

```basic system=atari8
restore @werte
read a
print a
end
@werte:
data 1, 2, 3
```

Where you write the values (sprite block, character set) depends on the system and is described in the documentation of the target system.

## Machine code in the BASIC program: `#asm`

With `#asm` you add machine code to the BASIC program. The line names an assembler file of the project (formerly the directive was called `#blob`; the old name no longer works and reports **B177** with a pointer to `#asm`):

```
#asm "player.asm"
#asm "player.asm" with "music/tune.vcsong"
```

When building, VintageCoder translates the file with the assembler and puts the code like this:

- **Commodore (C64, VIC-20, C128, Plus/4):** behind the BASIC text in the program. Its address depends on the length of the program; the build calculates it and repeats this until it is stable (at most eight passes, otherwise **B177**).
- **MEGA65 and Atari:** as **DATA lines** from the label `@asm_data`. The program loads them itself to the fixed address (`RESTORE @asm_data`, then read). On the Atari, the code sits below `$A000` at a page boundary.

In the program text you name the labels of the assembler code with the prefix `asm.`, for example `SYS asm.start`. The build inserts the decimal address (not in strings, after `REM` and in `DATA`). There are also `asm.base_addr` (start of the code) and `asm.base_page` (the page, for example for `POKE 106` on the Atari).

```
#asm "player.asm"
sys asm.start
```

With `with` you pass further files, for example a song; the names go unchanged to the assembler, which resolves a medium itself (see [Assembler directives](help:asm-directives)).

Errors: **B177** reports a wrong spelling (expected is `#asm "datei.asm" [with "song.asm"]`), a label `asm.name` that the code does not define, a program together with code that is too large (it ends above the memory) and code that does not fit below `$A000` on the Atari. **B173** reports that the line numbers in number mode are not enough for the DATA lines of the code. **B195** warns if the program does not fit into the free memory.

![The code editor with the BASIC file test.bas, which contains lines with #define, #if, #else and #asm with the file code.asm. Above the text is a help card for the '#asm' directive that explains that machine code is included and that asm.name gives the address of a label.](bild:basic-asm-direktive "The help card for the #asm directive in the BASIC editor.")

There are also two older ways:

1. **`DATA` + `POKE` + `SYS`.** The bytes of the code are `DATA`, the program writes them to a free address and calls it. An example for the C64: the code `lda #0 : sta $d020 : rts` consists of the bytes `A9 00 8D 20 D0 60` (169, 0, 141, 32, 208, 96) and sits at 49152 (`$C000`):

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

   You can get the bytes with the assembler (`vcc asm code.asm --format bin`) and transfer them into `DATA` lines. On the Atari you call with `USR(address)`.

2. **Two programs.** The machine code comes as a program of its own (assembler source in the project) on the same disk; the BASIC program loads it with `LOAD "name",8,1` and starts it with `SYS`. See [Disks](help:disks).

And the other way round: the assembler program itself carries a BASIC line `SYS` (`.basicstub`, see [Assembler directives](help:asm-directives)), and BASIC parts are converted to assembler with [basic2asm](help:basic-tools).

With the older ways, make sure the code does **not** sit in the BASIC program area or in the variables (C64: `$0801` to the end of the program plus variables). Free areas on the C64 are, for example, `$C000`-`$CFFF` or the cassette buffer from `$033C` (828).

## Comments and empty lines

- `; text` at the **start of a line** is a comment that does not go into the program. A `;` in the middle of a line is not a comment (in `PRINT` it is the attach character).
- `rem text` stays in the program and uses memory. The translator leaves the text unchanged, so `rem` may also contain special characters.
- Empty lines are skipped.
- The apostrophe `'` is a comment character only in Turbo BASIC XL (see [Dialects](help:basic-dialects)).
