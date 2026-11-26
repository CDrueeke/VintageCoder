# BASIC

In VintageCoder you write BASIC **in the original dialect of the target system**: BASIC V2 on the C64 and VIC-20, BASIC 7.0 on the C128, BASIC 3.5 on the Plus/4, BASIC 65 on the MEGA65, Atari BASIC (or Turbo BASIC XL) on the Atari. There is no language of its own and no extension of the commands: you may write whatever the system understands, and the built program is, byte for byte, what the system itself would have made from your text.

The conveniences affect only the **surroundings**. You do not have to assign line numbers, you jump with names instead of numbers, you may use long variable names, and you can include sprites, character sets and songs from the editors as `DATA` lines by naming their media file with `#include`.

This manual consists of several pages:

| Page | Content |
|---|---|
| BASIC (this page) | Introduction: your first program line by line, overview, basic rules |
| [Labels and line numbers](help:basic-labels) | `@name:`, `GOTO @name`, how the line numbers are created, converting and renumbering |
| [Long variable names](help:basic-names) | `#longnames`, short names, mapping table, pitfalls |
| [Directives, files and assets](help:basic-directives) | `#include` (also for media), `#define`, `#if`, `#numbering`, `#dialect`, `#asm` |
| [Dialects and target systems](help:basic-dialects) | C64, VIC-20, C128, Plus/4, MEGA65, Atari, Turbo BASIC XL |
| [Tools](help:basic-tools) | help for words, editor, tokenizing, converting, `basic2asm`, troubleshooting |
| [Cheat sheet and error codes](help:basic-index) | everything important for looking up |

Lines in the pattern `main.bas:3: error B110: ...` in the examples are the real messages of the translator. All examples on these pages were built with the translator; where an example is called "Listing", the listing shown is what you get when you read the built program back.

## My first BASIC program

Create a file `hallo.bas` in the project tree (or start a new project with the "BASIC" template) and write:

```basic liste
; My first program
print "{clr}hallo welt"
summe = 0
for i = 1 to 10
  summe = summe + i
next i
print "summe ="; summe
@ende:
goto @ende
```

```text
10 print "{clr}hallo welt"
20 summe = 0
30 for i = 1 to 10
40 summe = summe + i
50 next i
60 print "summe ="; summe
70 goto 70
```

This is a complete program for the C64. Line by line:

- `; My first program` - A semicolon at the start of a line makes the line a **comment** for you. It does not go into the program and costs neither memory nor time. (The BASIC word `rem` exists too, but it stays in the program.)
- `print "{clr}hallo welt"` - Prints text. **Control characters** are in curly braces: `{clr}` clears the screen. Text written in lowercase appears on the C64 as uppercase, as the machine shows it in its normal state (more on the page [Dialects](help:basic-dialects)).
- `summe = 0` - A **variable** (a named memory location for a number) gets the value 0. The name `summe` is **not** a problem here: the C64 sees only the first two characters, so `SU`. This is no problem as long as no second name begins with `su`. More on the page [Long variable names](help:basic-names).
- `for i = 1 to 10` ... `next i` - A counting loop: `i` runs from 1 to 10. You may indent the lines in between; the translator removes the indentation. Spaces around the `=` also stay as you write them.
- `print "summe ="; summe` - Text and number in one output. The semicolon attaches the number directly.
- `@ende:` - A **jump label** (*label*): a name for the place where it stands. It begins with `@` and ends with a colon. The line contains nothing else; the label applies to the next statement.
- `goto @ende` - Jumps to this label. Here the program jumps to itself and stops, so that the output stays on the screen. When building, the translator replaces `@ende` with the line number (70).

The listing shown is created like this: every line **without a number** automatically gets the next number (10, 20, 30 ...). Comment lines and empty lines get none. Everything else about this is on the page [Labels and line numbers](help:basic-labels).

### Build and run

The program is built and run like any project (see [Build](help:build) and [Run](help:start)). The file must be in the project as a source of the kind BASIC; the template already does this. The result of the build on the Commodore systems is a `.prg` file in the folder `build`, on the Atari a `.bas` file in SAVE format.

On the command line it also works without a project:

```bash
vcc basic hallo.bas --target c64 -o hallo.prg
```

## What stays the same and what is different

| Topic | Classic BASIC | Here |
|---|---|---|
| Line numbers | Required, every line | Optional. Lines without a number are numbered when building. You may write your own numbers in between at any time. |
| Jumps | `GOTO 100` | `GOTO @name` or still `GOTO 100` |
| Variable names | Only the first 2 characters count (Commodore) | With `#longnames` any length, the build maps to short names |
| Comments | `REM` | `REM` and, at the start of a line, `;` |
| Commands | Those of the dialect | Exactly those of the dialect, nothing added |
| Including files | not possible | `#include "datei.bas"`, also media files (sprites, character sets, songs) |
| Conditional translation, constants | not available | `#if`, `#else`, `#endif` and `#define NAME value` (see [Directives](help:basic-directives)); macros with parameters do not exist |
| Machine code | not possible | `#asm "datei.asm"` (see [Directives](help:basic-directives)) |
| Syntax check | when typing on the machine | The translator turns the text into tokens and **warns** about suspicious places (see below); on the Atari it checks the syntax and reports errors |

### Syntax check: warnings instead of errors

The translator recognizes keywords, numbers, strings and your labels, but it does not fully understand the sentences. A program with a typo therefore usually builds without a message, and the machine reports the error only when it runs:

```basic liste
x = 5 +
print x
```

```text
10 x = 5 +
20 print x
```

On the C64, line 10 gives `?SYNTAX ERROR IN 10` at start. In addition there is a **syntax check** that runs when building and reports suspicious places as a **warning**. It never stops the build:

| Code | Warns about |
|---|---|
| B150 | a keyword in the middle of a name (`score` contains `or`) |
| B151 | `FOR` without `NEXT` and the other way round |
| B152 | `GOSUB` without any `RETURN` and the other way round |
| B153 | a jump target with a number that is not a program line |
| B154 | two names that are the same variable for the machine (only the first 2 characters count) |
| B155 | a string that is not closed at the end of the line (only a note, this is allowed) |
| B156 | a wrong number of arguments (`poke 53280`) |
| B157 | a command that the dialect does not know (`graphic 1,1` in a C64 project) |
| B159 | a function without parentheses (`rnd`) |
| B167 | a `'` that is not a comment in this dialect |

The messages appear in the tab **Problems** in the tray below the editor and as a mark on the line. You turn off the check or single warnings in the area **Project**, page **Sources**, on the card "BASIC" ("Syntax check when building", "Disabled messages", for example `B152, B154`). On the command line, `vcc basic file.bas --check` checks without writing anything; `--no-check` and `--disable B152,B154` turn checks off. Errors cannot be turned off.

*[Bild folgt in E13: basic-karte-pruefung]*

Also check that the **source group has the right dialect** (page [Dialects](help:basic-dialects)), and try the program in the emulator. Only where the translator clearly cannot map something does it report an error: a character with no equivalent in the character set of the system, an unknown label, a line that is too long.

```basic fehler
print "a\b"
```

```text
main.bas:1: error B100: character '\' has no PETSCII equivalent
```

## Basic rules

- **File extensions:** `.bas` and `.lst` are treated the same (`.lst` for listings that you have from elsewhere). Both may contain line numbers, labels and directives.
- **Uppercase/lowercase:** you may write commands and names in lowercase, uppercase or mixed. In quotation marks it counts: on the C64 and the other Commodore machines **lowercase letters become uppercase** on the screen, **uppercase letters become graphics characters**. If you want uppercase on the screen, write the text in lowercase.
- **Question mark:** `?` is the abbreviation for `print` (on all systems).
- **Control characters** you write in curly braces: `{clr}`, `{home}`, `{down*3}`, `{$41}`. The names differ per system (page [Dialects](help:basic-dialects)).
- **Line length:** after tokenizing, a line may be at most 250 bytes (Commodore) or 255 bytes (Atari) long. The build reports **B102** if it becomes longer. The screen limit of the machine (C64: 80 characters when typing) does not apply to loaded programs, only to later editing on the machine.
- **Several statements** per line are separated by `:` (Atari: also `:`).
- **Highest line number:** 63999 (Commodore), 32767 (Atari).

## Hex and binary numbers

In the source you may write numbers in hexadecimal (`$D020`) and binary (`%00001110`):

```basic
poke $d020,%00001110
```

When building, VintageCoder turns them into the numbers the dialect understands: `poke 53280,14`. The notation in the source does not change the program, `$D020` and `53280` give the same bytes.

- **Where it applies:** in expressions, arguments and `DATA` elements (`data $ff,%1010`). Text in quotation marks and `rem` stay unchanged.
- **Type characters:** `a$` and `a%` stay variable names. After a keyword, `$` begins a number (`print $ff` prints 255).
- **Separator:** `_` may group digits (`$d0_20`, `%0000_1110`); a switch for this is still missing in the user interface.
- **Size:** up to 32 bits (`$ffffffff`). Addresses above 65535 and `poke` values above 255 are reported by the build as a warning (**B134**, **B135**), as an error if you wish.
- **Jump targets:** `goto $0100` does not work (**B138**): line numbers are created when building, use a label (`goto @name`).
- **Dialects:** Turbo BASIC XL keeps `$` (the machine reads it itself) and converts `%`; BASIC 65 gets decimal numbers or, if set so, the numbers as typed (this setting does not exist in the user interface yet). All other dialects get decimal numbers.
- **Errors:** **B130** invalid hex digit, **B131** invalid binary digit, **B132** `$` or `%` without digits, **B133** above 32 bits. **B136** warns if a hex number touches a keyword (`$ffand`: put a space before it), **B137** if `a%=40000` does not fit in an integer variable, **B139** if you wrote `0xFF`.
- **Display:** hex, binary and decimal have their own colors in the editor; over a number the hover shows all notations, the generated number and, for addresses in `poke`, `peek`, `sys`, `wait`, the chip register (`vic2.border - border color`).

### Number tools and hex suggestions

The commands **Number tools ...** (conversion and bit editor, see [Number tools](help:numbers)) and **BASIC: Hex suggestions for addresses ...** are in the menu **Edit**; **Number tools ...** exists in this version, **BASIC: Hex suggestions for addresses ...** does not yet (the command is hidden in the menu). The hex suggestions exist on the command line: `vcc basic --suggest-hex file.bas` lists decimal addresses that are a chip register of the target system (only complete addresses in `poke`, `peek`, `sys`, `wait`, `usr`), and `--apply 1,3 -o new.bas` writes the chosen suggestions into a new file. Without `--apply` nothing changes.

## Examples that come with VintageCoder

- `examples/c64/basic_native` - C64, labels, without line numbers.
- `examples/c64/basic_langenamen` - C64 with `#longnames`.
- `examples/atari/basic_native` - Atari BASIC with labels.
- `examples/atari/turbo_basic` - Turbo BASIC XL with `PROC`, `WHILE`, `REPEAT`.
- More small programs are in the templates (`vcc new ... --template basic-hello`) for every system.

See also: [Load graphics](help:graphics), [Editor](help:editor), [Assembler](help:assembler).
