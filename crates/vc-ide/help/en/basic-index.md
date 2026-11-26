# BASIC: cheat sheet and error codes

A short version for looking things up. The detailed explanation is on the pages [BASIC](help:basic), [Labels and line numbers](help:basic-labels), [Long variable names](help:basic-names), [Directives, files and assets](help:basic-directives), [Dialects](help:basic-dialects) and [Tools](help:basic-tools).

## Structure of a line

```
100 @name: statement : statement     ; line number, label, several statements
@name:                               ; label for the next statement
statement                            ; without a number: numbered when building
; comment                            ; comment, does not go into the program
#include "datei.bas"                 ; insert a file or media file
#longnames                           ; long variable names
#define NAME wert                    ; constant
#if SYSTEM_C64 ... #endif            ; conditional translation
#numbering start=10 step=10          ; numbering of this file
#dialect atari_basic                 ; dialect of this file
#asm "datei.asm"                     ; machine code added to the program
```

The part after `;` in this box is only explanation; in the program, a `;` in the middle of a line is **not** a comment.

## Notations at a glance

| Task | Notations (all equivalent) |
|---|---|
| Jump target | `goto @name`, `goto 100` (fixed number), on Turbo BASIC XL also `go# name` |
| Define a label | `@name:` alone on the line, `@name: statement`, `100 @name: statement` |
| Several labels | `@a: @b: statement` |
| Include a file | `#include "datei.bas"`, `#INCLUDE "datei.bas"` |
| Include a medium | `#include "graphics/hero.vcsprite"` (DATA lines with label `@hero:`, line number in `hero_line`) |
| Constant | `#define RAHMEN 53280`, then `poke RAHMEN, 0` |
| Machine code | `#asm "player.asm"`, then `sys asm.start` |
| Long names | `#longnames`, `#LONGNAMES` |
| Comment | `; text` (start of line, does not go into the program), `rem text` (stays in the program), Turbo BASIC XL: `' text` |
| Output | `print`, `?` |
| Clear the screen (C64) | `print "{clr}"`, `print "{$93}"`, `print chr$(147)` |
| Repeat control characters | `{down*3}` or `{down}{down}{down}` |
| `rvs on` | `{rvs on}`, `{rvon}` |

## Directives

| Line | Effect | Page |
|---|---|---|
| `#include "path"` | insert a file or media file (relative to the including file, up to 8 levels) | [Directives](help:basic-directives) |
| `#longnames` | allow long names | [Long names](help:basic-names) |
| `#define NAME wert` | constant, replaces `NAME` before numbering | [Directives](help:basic-directives) |
| `#if CONDITION`, `#else`, `#endif` | conditional translation (`SYSTEM_C64`, a `#define` name, `!` in front) | [Directives](help:basic-directives) |
| `#numbering start=N step=N` | start and step of the line numbers | [Directives](help:basic-directives), [Labels](help:basic-labels) |
| `#dialect id` | dialect of this file | [Dialects](help:basic-dialects) |
| `#asm "datei.asm" [with "song.asm"]` | translate machine code and add it to the program; labels as `asm.name` | [Directives](help:basic-directives) |

The translator does not recognize any other line that begins with `#`; it translates it as BASIC text. The former name `#blob` no longer works: it reports **B177** with a pointer to `#asm`.

## Dialects

| System | Dialect | Names | Lines up to |
|---|---|---|---|
| C64 | `cbm_v2` | 2 characters | 63999 |
| VIC-20 | `vic_basic_2` (builds on `cbm_v2`) | 2 characters | 63999 |
| C128 | `basic7` | 2 characters | 63999 |
| Plus/4 | `basic35` | 2 characters | 63999 |
| MEGA65 | `basic65` | 2 characters | 63999 |
| Atari | `atari_basic` | all characters, up to 128 variables | 32767 |
| Atari | `turbo_basic_xl` | all characters, up to 256 variables | 32767 |

Reserved variables (Commodore): `ST`, `TI`, `DS`; from BASIC 3.5 also `ER`, `EL`. A line byte limit applies: 250 bytes (Commodore), 255 bytes (Atari).

## Command line

| Command | Effect |
|---|---|
| `vcc build` | build the project, including BASIC sources (`vcc build project`, `--main`) |
| `vcc basic datei.bas` | build a `.bas` or `.lst` file (`--target`, `--dialect`, `--memory`, `--start`, `--step`, `-D`, `--compact`, `-o`) |
| `vcc basic datei.bas --check` | only check: print warnings B150 to B167, write nothing (`--no-check` switches the check off, `--disable B152,B154` single warnings) |
| `vcc basic --to-labels datei.bas` | convert line numbers to labels (`--to-numbers`: labels to numbers, `--renumber`: renumber; `--preview`, `--write`, `--range from-to`) |
| `vcc basic --tokenize datei.lst` | tokenize a classic listing with line numbers |
| `vcc basic --detokenize datei.prg` | output a program as a listing |
| `vcc basic2asm datei.bas` | convert to assembler (`--target c64`, `c128`, `plus4`; `--check`, `--stage`, `--float`, `--int-div`, `--map`, `--no-rem`, `--no-track-line`) |
| `vcc import datei.d64 NAME` | take over a BASIC program from a disk as `.bas` source |
| `vcc new folder --template basic-hello` | new project with the BASIC template (with `--structure standard` also with a folder structure) |
| `vcc migrate includes` | change old include paths `build/export/...` to the media file (`--apply` writes) |

## Error codes

Messages have the form `datei.bas:LINE: error CODE: text`. For included files, `datei.bas` points to the included file.

| Code | Message | Explanation |
|---|---|---|
| B100 | `line number N is greater than 63999` | too many lines or own number too large (Atari: B185, limit 32767) |
| B100 | `line number N is not greater than the previous one (M)` | own numbers not ascending or identical with the automatic number |
| B100 | `character 'x' has no PETSCII equivalent` | character (umlaut, `\` ...) does not exist in Commodore text: use `{$xx}` |
| B184 | `character 'x' is not in the ATASCII range (use {$xx})` | the same for the Atari |
| B100 | `unknown control code {name}`, `'}' is missing`, `invalid code ...`, `invalid count in ...` | wrong control characters in `{...}` |
| B180, B181, B182, B183 | `'=' expected after 'X'`, `expression is incomplete`, `'x' is not a valid label name`, `invalid number` | Atari: syntax error in statement, expression, name or number |
| B100 | `unexpected character 'c'` | character that does not exist in the dialect (e.g. `_` in an Atari name) |
| B185 | `more than 128 variables` | Atari BASIC: too many variables (Turbo BASIC XL: 256) |
| B185 | `line is too long (more than 255 bytes)` | Atari: too much in one line |
| B100 | `line does not start with a line number` | only with `vcc basic --tokenize` (classic listing) |
| B102 | `line is too long (more than 250 bytes)` | Commodore: split the line |
| B110 | `label @x is not defined` | label is missing or stands in the middle of a line |
| B111 | `label @x is defined twice` | two identical labels |
| B112 | `label @x is at the end with no statement after it` | label has no following line |
| B120 | `#include "x": file not found ...` | wrong path or old export path without a file |
| B121 | `#include "x": nested too deeply or circular` | files include each other in a circle, more than 8 levels |
| B131 | `no free short name is left for the name 'x' (too many variables)` | only with `#longnames` |
| B150 to B159 | warnings of the syntax check | suspicious places, see [BASIC](help:basic) (section syntax check); can be switched off |
| B160, B161, B162, B166 | line numbers and labels mixed; conversion not possible; computed jump target; mode of an included file does not fit | when converting, see [Labels](help:basic-labels) |
| B167, B168 | `'` is not a comment character; invalid marker line `;@name` | warning |
| B170, B178 | dialect does not fit the target system; unknown dialect | `#dialect` or project setting |
| B172, B174, B175, B176 | `#define` with a keyword, wrong spelling, `#if` built wrongly, `#define` circular | see [Directives](help:basic-directives) |
| B173, B177 | `#asm`: line numbers not enough for the DATA lines; spelling, missing label, code too large, old name `#blob` | see [Directives](help:basic-directives) |
| B190, B191, B192 | `#numbering` written wrongly; value range; numbers not enough (with a suggestion for the step) | see [Labels](help:basic-labels) |
| B193, B194, B195, B196 | medium cannot be included; old path `build/export/...`; program larger than the free memory; several media and `RESTORE @label` in BASIC V2 | see [Directives](help:basic-directives) |
| W313 | `'a', 'b' differ only after the first 2 characters ...` | warning: long names with the same beginning, mapped separately |

The codes **E...** and **I...** belong to [basic2asm](help:basic-tools) (E3xx: constructs that cannot be translated, I110-I113: notes on the integer stage). Codes **E2...** and **W2...** come from the [assembler](help:asm-index).

## Common misunderstandings

| Expectation | Actually |
|---|---|
| The translator reports syntax errors | Only on the Atari as errors. For the other dialects, the syntax check gives warnings (B150 to B159); the machine still checks only when running. |
| `#if SYSTEM_C64 ... #endif` selects code per system | It does exist: the translator leaves out the lines of the other branch. |
| `restore @name` works everywhere | Not on C64 and VIC-20 (BASIC V2). |
| `@name:` may stand in the middle of a line | Only at the start of a line (otherwise B110). |
| Long names are always on | Only with `#longnames`. |
| `'` is a comment | Only in Turbo BASIC XL. Otherwise `;` at the start of a line or `rem`. |
| Lowercase letters in C64 text are lowercase | They appear as uppercase; uppercase letters become graphics characters. |
