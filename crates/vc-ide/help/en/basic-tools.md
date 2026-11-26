# BASIC: Tools, Conversion and Troubleshooting

This page shows what the editor does for BASIC, how you tokenize programs and read them back, how `basic2asm` turns a BASIC program into assembler, and how you find errors. It belongs to the [BASIC manual](help:basic).

## Help for a word

Point the mouse at a BASIC word (`poke`, `for`, `restore` ...) and wait a moment: a card appears with a short explanation, if there is one for the word. If there is a problem at that place, its message is on the same card, with a suggestion for correcting it if needed. The texts come from the database `data/help/basic_keywords.json`; it covers the most common words of BASIC V2 and Atari BASIC. A key shortcut "Help for word" (formerly F1) does not yet exist in this interface.

The search in the help finds commands, pages and error codes: type `B110` or `longnames` and you land on the matching page.

## The editor

- **Line numbers:** The editor does not yet show the number that a line without its own number gets when you build (10, 20, ...) in this version; it is in `build/<name>.classic.txt`. The margin shows the line numbers of the source file. See [Labels](help:basic-labels).
- **Hover:** If you hold the mouse over a number, a tooltip shows its value in the usual notations (for addresses also the chip register). Over a **long variable name** it shows the short name that the build makes from it (see [Long names](help:basic-names)).
- **Format line when leaving it:** An edited line is tidied up when you leave it: one space after the line number, and multiple spaces outside of strings become one. The text after `REM` stays as it is. On command: **Edit > Formatting > Format line** and **Format file**.
- **`#include` lines:** You open the included file in the file tree. This version has no menu entry "Open file" and no double-click on the file name; there is also no menu command to comment out whole lines.
- **Converting:** The menu **Edit > Numbers and BASIC** and the right-click menu of a BASIC file offer **Convert to labels ...**, **Convert to line numbers ...** and **Renumber ...** (see [Labels](help:basic-labels)).
- **Errors:** Messages from the build and from the syntax check appear in the **Problems** tab in the tray below the editor and as a mark on the line. For included files, the message points to the right file and line (see [Directives](help:basic-directives)).

## What the build produces

| File in `build/` | Content |
|---|---|
| `<name>.prg` or `<name>.bas` | the program (Commodore: PRG with load address, Atari: SAVE format) |
| `<name>.classic.txt` | the finished program as text with line numbers, read back from the program |
| `<name>.labels.txt` | label and line number (only if there are labels) |
| `<name>.vars.txt` | long names and short names (only with `#longnames`) |

Included media (sprites, character sets, songs) are cached in the folder `build/.media-cache/`; you can delete it at any time (see [Directives](help:basic-directives)).

`<name>.classic.txt` is the best tool for checking: this is how the device sees your program. The keywords appear there in lowercase (Commodore) or uppercase (Atari).

## Tokenizing and reading back

The word **tokenizing** means: the text of a BASIC program is translated into the format in which the device stores it. `PRINT` becomes a single byte (a *token*), and numbers become their digits or the number format of the dialect. **Detokenizing** is the reverse.

```bash
vcc basic hallo.bas --target c64 -o hallo.prg
vcc basic --detokenize hallo.prg
vcc basic --tokenize liste.lst --target c64 -o liste.prg
```

- `vcc basic file.bas` (also `.lst`) builds a program in the native dialect of the target with everything described on these pages (labels, directives, long names). It prints `@name = line n` for each label and, with `#longnames`, the mapping table.
- `--target c64|c128|plus4|mega65|vic20|atari8` chooses the system, `--dialect ID` the dialect (e.g. `turbo_basic_xl`), `--memory 8k` the VIC-20 memory expansion, `--start` and `--step` the start number and step, `-D NAME=wert` sets a constant like `#define`, and `--compact` removes superfluous spaces (Commodore only). `vcc basic` resolves `#include`, also media files (the project folder is the nearest folder with a project file).
- `--check` only checks and prints the warnings of the syntax check (B150 to B159 and B167) without writing a file; `--no-check` switches the check off when translating, `--disable B152,B154` single warnings. Errors cannot be switched off.
- `--to-labels`, `--to-numbers` and `--renumber` convert numbers and labels (see [Labels](help:basic-labels)).
- `--detokenize` reads a program (PRG or Atari SAVE file) and prints the listing. For Atari files, give `--target atari8`.
- `--tokenize` translates a **classically numbered** listing, without labels and without directives. Lines without a number are errors there (**B100**, `line does not start with a line number`). This path does not convert hex and binary numbers (only native BASIC does that); if `$FF` or `%1010` appears in a dialect without such numbers, it warns (**B140**).
- `--suggest-hex` lists decimal addresses that are a chip register of the target system and would be more readable as hex (only arguments of `POKE`, `PEEK`, `SYS`, `WAIT`, `USR`). With `--apply 1,3 -o neu.bas` it writes the chosen suggestions to a new file; without `--apply` it changes nothing.

When reading back, the keywords appear in lowercase (Commodore) or uppercase (Atari), and control characters as `{clr}`. With `vcc import`, a BASIC program from a disk or from a `.prg` file becomes a readable `.bas` source that you can edit further.

## Converting BASIC to assembler: basic2asm

`basic2asm` translates a BASIC program into readable **assembler source**. The program then no longer runs in the BASIC interpreter but as a machine program: faster, and you can look at every line and change it. Each BASIC line becomes a section with the original as a comment and its own label `L<line>`.

In the user interface, the command **File > Convert BASIC to assembler** is in the command list but is not yet implemented (see [Converting BASIC to assembler](help:basic2asm)). On the command line it already works:

```bash
vcc basic2asm programm.bas --target c64 -o programm.asm
vcc basic2asm programm.bas --check
```

```text
programm.asm: stage 1, 5 lines, modules: rt_core, rt_arith, rt_io
programm.bas: OK (stage 1, 5 lines)
```

| Switch | Effect |
|---|---|
| `--target c64`, `c128`, `plus4`, `atari8` | target system (default c64). Floating point exists for all four targets; `atari8` knows all three stages (see below) |
| `-o file.asm` | output (default: same name with `.asm`) |
| `--map file.json` | line map (default: same name with `.b2a.json`) |
| `--stage 1` to `3` | highest allowed stage; the smallest stage that the program needs is chosen |
| `--float` | force the floating-point runtime |
| `--int-div` | translate `/` as integer division (otherwise it needs stage 2) |
| `--style compact\|inline` | expression style. `compact` (default): each operation calls a runtime routine. `inline`: simple integer expressions (variable +/- constant, variable +/- variable, constant - variable, `=` and `<>`, `X=X+/-c`) appear as direct 6502 instructions - longer in the source, shorter at run time. Everything else stays `compact` |
| `--runtime include\|embed` | `include` (default): the runtime comes via `#include` from `data/sys/`. `embed`: the sources of the runtime (and of system files such as `os.asm`) are copied into the result file; it can then be assembled and passed on without an include folder |
| `--no-track-line` | do not carry a line number at run time (smaller, but error messages without a line) |
| `--no-rem` | do not copy `REM` lines and `;` comment lines |
| `--check` | check only, write no files |

You can check and print the syntax tree: `vcc basic ast file.bas` shows it, `--json` writes it as JSON (lines, statements, expressions; strings as byte lists of the target character set), `--roundtrip` writes it back as canonical text (parentheses only where the precedence needs them), reads this text again and reports **E191** if the same tree does not come out. Statements that the converter does not translate (`OPEN`, `LOAD` ...) cannot be written back (**E190**). Both settings (`inline`, `embed`) also exist in the project file (`convert`).

### Atari (`--target atari8`)

- **Scope:** Stages 1 to 3 - integers, floating point and strings, loops, subroutines, `DATA/READ`, `DIM`, `POKE/PEEK`, `INPUT`, `GET`, `PRINT` with `;`, `,`, `TAB`. The result is an `.xex` file (`.org $2000`, `.run`); input and output go through CIO (editor `E:`), and the layer for this is in `data/sys/atari8bit/b2a/rt_os.asm`.
- **Numbers:** If the program needs only integers, it calculates with 16-bit cells (stage 1, notes I110/I150). As soon as fractions, decimal numbers, `^`, `SQR`, `RND`, `LOG`, `EXP`, `SIN`, `COS`, `ATN` or large numbers appear, the converter chooses stage 2: all numbers are then **6-byte BCD** as in Atari BASIC (10 digits, range 1E-98 to 9.99999999E+97), calculated with the **floating-point package of the operating system** (`$D800`-`$DFFF`, without the BASIC module). Rounding, `0.1+0.2 = 0.3` and the output (`PRINT` shows at most 9 digits if the first digit pair is 10 or more; exponent notation such as `1E+10`) therefore match the device.
- **Differences from Atari BASIC:** `PRINT` of a number prints no sign position, as in Atari BASIC; comparisons return 1, and `AND`, `OR` and `NOT` are logical (1 or 0) as in the original. Error messages are text as in CBM (`ILLEGAL QUANTITY`, `DIVISION BY ZERO`, `OVERFLOW`, `STRING LENGTH`, `DIMENSION` ...), not a number.
- **Accuracy:** `SQR` calculates with `LOG10`/`EXP10` of the package and refines once by Newton's method (square numbers become exact). The converter calculates integer powers up to 255 (`2^10`, `(-2)^3`, `2^-2`) by repeated multiplication, so they are exact; other powers go through `EXP10(b*LOG10(a))` and are accurate to about 8 digits. That is the accuracy of the OS package, not of the program. `RND` reads `RANDOM` (POKEY); the argument does not matter. `SIN`, `COS` and `ATN` are the approximations of the BASIC module (rebuilt with `PLYEVL` of the OS package and the same coefficient tables), and the digits match the original. `DEG` and `RAD` set the byte `$FB` as there (6 = degrees); the converter rewrites them to `POKE 251,6` and `POKE 251,0`.
- **Not translatable:** `TAN` does not exist in Atari BASIC (error E334); `XIO`, `TRAP`, `POP`, `NOTE`/`POINT`, `PUT`, `PADDLE`/`PTRIG`, `OPEN`/`CLOSE`/`INPUT #` and others are reported as "not translatable" (E311), not silently replaced. Long names are always mapped to short names (the mapping is in the line map).

Labels and `#longnames` also work here: the conversion starts like the build, with the numbering and the resolution of the labels. For each BASIC line, the line map contains the line in the source and the label in the assembler, plus `names` with the mapping of the long names.

- **Graphics, sound and joystick:** `GRAPHICS n` (also `+16` and `+32`), `COLOR`, `PLOT`, `DRAWTO`, `POSITION`, `LOCATE x,y,v`, `SETCOLOR`, `SOUND`, `STICK(n)`, `STRIG(n)` and `PRINT #6;...` run as in the original through operating system channel 6 (`S:`) and the hardware registers (runtime `data/sys/atari8bit/b2a/rt_gfx.asm`). For the converter, these statements are keywords of their own, so `COLOR` (contains `OR`) or `POSITION` (starts with `POS`) are not variables. `PRINT #n` writes through channel n; only channel 6 is opened (by `GRAPHICS`). Screen memory, color shadows and cursor match the original in a comparison (mode 0, 1, 2, 3, 5 and 8); `SOUND` is checked in the model (register values), not against the device.
- **Turbo BASIC XL (extension `.tur` as on the Atari, `--turbo` and `turbo` in the project file):** The converter additionally reads the constructs of Turbo BASIC XL and translates them into simple jumps (source without line numbers such as `examples/atari/turbo_basic/balken.tur`): `IF` without `THEN` with `ELSE` and `ENDIF`, `WHILE`/`WEND`, `REPEAT`/`UNTIL`, `DO`/`LOOP` with `EXIT`, `PROC name`/`ENDPROC`/`EXEC name` (the original has no parameters either; if the flow runs into a procedure, Turbo BASIC XL reports error 27, while the translated program simply skips it - procedures belong after `END`), the operators `DIV` and `MOD`, `DPOKE`, `DPEEK`, `PAUSE n` (n frames), `MOVE from,to,n`, `-MOVE from,to,n` (copy downward), `CLS`, `HEX$(n)` (two digits below 256, otherwise four), `DEC(a$)` (hex number at the start, 16 bits with overflow as in the original), `INSTR(a$,b$[,n])` (n = number of characters to skip), `RAND(n)` (= `INT(RND(0)*n)`), `INKEY$` (without waiting), `TIME$` (`hhmmss` from the clock), `ON n EXEC a,b`, `FCOLOR n` and `FILLTO x,y` (fill like XIO 18, after `PLOT`), `BPUT #n,address,length` and `BGET #n,address,length` (block through CIO). Blocks may be nested and may span lines; a block that is not closed or closed wrongly reports **E341**/**E342**, `EXEC` of a missing procedure **E343**, a duplicate procedure **E340**. Without `--turbo`, `WHILE`, `PROC` and so on are ordinary names as in Atari BASIC. Not translated: `TRACE`, `DUMP`, `RENUM` and other commands of the programming environment. Channels (for Atari programs, with and without `--turbo`): `OPEN #n,aux1,aux2,"name"`, `CLOSE #n`, `PUT #n,byte`, `GET #n,v` and `PRINT #n` go through CIO. The devices E: and S: (channel 6 after `GRAPHICS`) and files on `D:` (writing and reading with `PUT`, `GET #`, `PRINT #`, `BPUT`, `BGET`, compared with the real Turbo BASIC XL) are checked. `D:` exists only if the program runs from a DOS disk (as `AUTORUN.SYS`); a program with `OPEN` always includes the string runtime. An error of the operating system reports itself as `?DEVICE ERROR <number> IN <line>` with the Atari error number (170 = file not found, 136 = end of file) and ends the program; `TRAP` does not exist. Everything is checked against the real Turbo BASIC XL (disk image `assets/disks/atari8/Programmiersprachen/Turbo_Basic_XL.atr`, not in Git; the tests type the programs into atari800): block statements, `DIV`/`MOD` also with negative and fractional numbers, number output (Turbo BASIC XL prints up to ten digits, Atari BASIC nine) and the words above. `RAND(n)` for n <= 0 returns n in the original, here `INT(RND(0)*n)`; a negative skip number for `INSTR` is not checked.
- **Strings (stage 3):** by the rules of Atari BASIC, measured on the real BASIC module. `DIM A$(n)` with a number creates a fixed buffer (without `DIM` the converter reports E321, and `DIM` twice is error 9 at run time); assignments silently cut to the size. Substrings `A$(n)` and `A$(n,m)` count from 1, also as a target (`A$(3,4)="XY"` replaces at most two characters, `A$(LEN(A$)+1)="Z"` appends); reading outside 1...LEN reports `STRING LENGTH`. There are `LEN`, `ASC`, `VAL`, `STR$`, `CHR$`, comparisons (true = 1, by byte value), `INPUT A$` (whole line) and `READ A$`. `DATA` is raw text up to the comma, with quotation marks and spaces, and extends to the end of the line (also across `:`); each entry becomes a number (otherwise error 8) or a string only when it is read. The program then always calculates with floating point (`VAL` and `STR$` need the OS package). Not translated (and not present in the original either): `+` for strings, `LEFT$`, `RIGHT$`, `MID$` (E311), `GET` into a string. Texts in quotation marks, `DATA` and `REM` stay ASCII (characters such as `|` or `_` work); the size of `DIM` must be a number.
- **Parser rules of Atari BASIC:** The converter reads Atari source with its own rules: unary `-`, `+` and `NOT` bind more strongly than `^` (`-2^2` is 4, `NOT 0=1` is `(NOT 0)=1`), `DATA` extends to the end of the line, texts stay ASCII.
- **Comparison with the original:** `crates/vc-ide/tests/atari_basic_oracle.rs` runs the same programs in the real Atari BASIC (`ATARIBAS.ROM` in atari800) and as a translated program and compares the screen: numbers, functions, `^`, operator precedence, loops, arrays, strings, logic, comma tab, joystick and graphics memory match (only with `VC_EMU_TESTS=1`).

### The three stages

The converter chooses the **stage** itself:

1. **Integer.** All numbers are 16-bit integers. Variables such as `A` then become 16-bit cells. The converter points this out (**I110**: `variable A is a floating-point number in the original; stage 1 uses 16-bit integers`). This is a note, not an error.
2. **Floating point** (C64, C128, Plus/4, Atari) through the arithmetic routines of the BASIC ROM or (Atari) of the OS floating-point package.
3. **Strings.**

**MEGA65 (`--target mega65`, dialect `basic65`, program at `$2001`):** only stages 1 and 3 (integer and strings). Floating point reports **E331**, because the ROM of the MEGA65 has no usable floating-point routines. The result stays 6502-compatible (long jumps as a reversed branch plus `JMP`, `.branchlong jmp`). In the model it runs with the same programs as C64, C128 and Plus/4. **Checked in Xemu on Windows** (live tests in `crates/vc-ide/tests/b2a_mega65_live.rs`): KERNAL and interrupt leave the zero-page block `$40`-`$53` untouched (`data/sys/mega65/b2a/rt_zp.asm`), and integer and string programs run and write the expected text. **Not yet checked on a real MEGA65 device.**

```basic nocheck
a = 1
print a
b = a + 2
print b
@x:
goto @x
```

This program gives:

```text
warning[I110]: variable A is a floating-point number in the original; stage 1 uses 16-bit integers
```

### What the converter does not (yet) translate

The errors start with **E**. The most important ones:

| Code | Meaning |
|---|---|
| E100, E101, E102, E103 | line or character not readable, line number duplicated, statement not readable (`line 10: '=' expected`) |
| E303 | number too large for 16 bits |
| E304 | only one-dimensional arrays |
| E306 | function is not translated |
| E311 | command is not translated (files, `LIST`, `NEW`, `CONT`, `DEF FN` in stage 1) |
| E320 | types do not match (`TYPE MISMATCH`: string and number mixed) |
| E330, E331 | target or floating point not implemented for the target |

The converter does not translate commands that a dialect has in addition (graphics, sprites). It then reports E103 or E311.

```basic fehler nocheck
sprite 1, 1
```

```text
error[E103]: line 10: '=' expected
```

### Debugging with the line map

Every conversion comes with a line map that assigns each BASIC line to the assembler code. Debugging with it (breakpoints in the `.bas` file, showing the BASIC line when stopped) is not yet available in this version.

## Finding errors

1. **Read the messages of the build.** They start with `main.bas:LINE:` and a code. The tables on the pages [Labels](help:basic-labels), [Names](help:basic-names), [Directives](help:basic-directives) and the [Cheat sheet](help:basic-index) explain every code.
2. **Test the program on the device.** The build warns about suspicious places (syntax check, B150 to B159) but really checks the syntax only on the Atari. If the device reports `?SYNTAX ERROR IN 40`, 40 is the **generated** line number. In `build/<name>.classic.txt`, line 40 appears as the device sees it; you find the source line through the line map or through `.classic.txt`.
3. **Check the right dialect.** A command that the dialect does not know becomes letters and causes `?SYNTAX ERROR` on the device. The listing in `.classic.txt` does not show it. Therefore check the command list on the page [Dialects](help:basic-dialects).
4. **Suspicious names.** Strange errors with names such as `total` or `score` come from keywords inside the name. Switch on `#longnames`.
5. **Trace what happens.** On C128, Plus/4 and MEGA65, `tron` switches on line tracing (`troff` off); `trap @label` catches errors, and `print er, el, err$(er)` reports them. On the C64 there are only `print` outputs and `stop`/`cont`. On the Atari, `trap @label` catches errors, and `? err` reports the code.
6. **Does an assembler part not run?** For the assembler, see the page [Assembler](help:assembler).
