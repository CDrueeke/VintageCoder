---
id: basic
titel: BASIC in the IDE
reihenfolge: 7
stufe: Beginner
zeit: 30 minutes
siehe_auch: [assembler, editor, bauen-starten]
referenz: [basic, basic-labels, basic-names, basic-dialects, basic-directives, basic-tools, turbo-disk]
i18n_status: maschinell
i18n_quelle: 4acf880d0711
---

# BASIC in the IDE

**Goal:** You write a BASIC program for your retro computer without assigning line numbers. You jump with names instead of numbers, use long variable names, choose the right BASIC dialect and, if needed, turn a BASIC program into assembler.

## What you need

- A project with the **BASIC** template, see [Your first project](kapitel:erstes-projekt), or a new file with the extension `.bas` in an existing project (right-click in the project tree or the button **New file** above the tree, in the dialog "New file" the type **BASIC (.bas)**).
- A BASIC project has no dialect of its own to set: The BASIC automatically follows the **target system** of your project.

## BASIC stays BASIC

In VintageCoder, you write BASIC **in the original dialect of the target system**: BASIC V2 on the C64 and VIC-20, BASIC 7.0 on the C128, BASIC 3.5 on the Plus/4, BASIC 65 on the MEGA65, Atari BASIC (or Turbo BASIC XL) on the Atari. There is no language of its own and there are no new commands. What the computer understands, you may write, and the built program is what the computer itself would have made of your text.

The conveniences concern only the **surroundings**:

- You do **not** have to assign **line numbers**.
- You jump with **names** instead of numbers.
- You may use **long variable names**.
- You can include **sprites, character sets and songs** from the editors as `DATA` lines.
- You convert a program between line numbers and labels with a dialog and renumber it.

## Step by step: my first BASIC program

1. In your project, open the BASIC file (with the template, `src/c64/main.bas`). Replace the content with:

```basic
; My first program
print "{clr}hello world"
summe = 0
for i = 1 to 10
  summe = summe + i
next i
print "sum ="; summe
@ende:
goto @ende
```

2. Press **F5** (Run start program; this applies if the BASIC file is the start program of the project, as with the template). VintageCoder builds the program and runs it in the emulator. On the C64, "HELLO WORLD" and "SUM = 55" appear.

Line by line:

- `; My first program` is a **comment** for you. A semicolon at the start of a line makes the line a comment. It does not go into the program and costs neither memory nor time.
- `print "{clr}hello world"` prints text. In curly brackets are **control characters**: `{clr}` clears the screen. Text written in lowercase appears as uppercase on the C64; this is the normal display.
- `summe = 0` assigns the value 0 to a **variable** (a named place in memory for a number).
- `for i = 1 to 10` ... `next i` is a counting loop: `i` runs from 1 to 10. You may indent the lines in between; the translator removes the indentation.
- `print "sum ="; summe` prints text and number together.
- `@ende:` is a **jump label**. `goto @ende` jumps there; here the program jumps to itself and stops, so that the output stays on the screen.

> **Info:** In the source texts of this chapter, the lines have **no number**. When building, each line automatically gets the next number (10, 20, 30 ...); comment lines and empty lines get none. The built program looks like this:

```text
10 print "{clr}hello world"
20 summe = 0
30 for i = 1 to 10
40 summe = summe + i
50 next i
60 print "sum ="; summe
70 goto 70
```

> **Caution:** The translator does **not check syntax**. It turns your text into the commands of the dialect, but it does not understand the sentences. A typing error like `x = 5 +` builds without a message, and the computer reports the error only when running (`?SYNTAX ERROR IN 10`). So always try your program in the emulator.

## Jump labels instead of line numbers

In classic BASIC, you jump with `GOTO 100`. Then you have to remember line numbers and cannot insert anything without changing numbers. In VintageCoder, you give the target line a **name**:

```basic
; Counts from 1 to 5, with a jump label instead of a line number
print "{clr}"
n = 1
@schleife:
print "number"; n
n = n + 1
if n <= 5 then goto @schleife
@ende:
goto @ende
```

- A label begins with `@` and, in its definition, ends with a **colon** (`@schleife:`). The line contains nothing else; the label applies to the next statement.
- `goto @schleife` (and likewise `gosub @name`, `then @name`) jumps to the label.
- When building, VintageCoder replaces `@schleife` with the line number. The built program looks like this:

```text
10 print "{clr}"
20 n = 1
30 print "number"; n
40 n = n + 1
50 if n <= 5 then goto 30
60 goto 60
```

Which line a label got (`@schleife = line 30`), you can see in the built program; it is stored as text in `build/<name>.classic.txt`. If you do want to number a line yourself, write the number in front as usual; you can mix both.

*[Bild folgt in E13: basic-marken]*

If you already have a program with line numbers, or want line numbers again to pass the program on, you convert it. How this works is described further down in the section "Converting: labels, numbers, renumbering".

## Long variable names

On the C64, VIC-20, C128, Plus/4 and MEGA65, the BASIC tells variable names apart only by the **first two characters**. There, `punkte` and `punkt` are the **same** variable, namely `PU`. This error is sneaky, because the program runs and only calculates wrongly. In addition: The BASIC recognizes commands in the middle of a word. In `total` there is `TO`, in `score` there is `OR`, in `form` there is `FOR`; the device then reports `?SYNTAX ERROR`.

With the line `#longnames` at the start of the file, you get long, meaningful names. The translator converts them to short names when building and makes sure that no names collide and no keywords appear:

```basic
#longnames
; Two different variables, although they begin the same
punkte = 10
punkt = 5
print "{clr}punkte ="; punkte
print "punkt ="; punkt
@ende:
goto @ende
```

When building, VintageCoder reports a warning (in the **Problems** tab of the tray below the editor) and stores the assignment:

```text
Long names -> short names:
  punkte                     ->  PU         (number)
  punkt                      ->  PN         (number)
```

The warning (W313) only says: The two names differ only after the first two characters, and VintageCoder has separated them for you. The built program uses `PU` and `PN`.

*[Bild folgt in E13: basic-langenamen]*

The assignment table is also stored as a file in the folder `build/` (`....vars.txt`). A detailed example is in the project `examples/c64/basic_langenamen`. On the **Atari**, you do not need `#longnames`: There, all characters of a name count.

## Hex and binary numbers

You may write numbers in the source text in hexadecimal (`$D020`) and binary (`%00001110`): `poke $d020,%00001110`. When building, this becomes what the dialect understands (`poke 53280,14`); the notation does not change the program. In strings and after `rem`, everything stays as it is; in `data` lines, complete numbers are converted. `a$` and `a%` are still variable names. As a jump target (`goto $0100`), they are not allowed: use a label.

In the editor, hex, binary and decimal have their own colors. If you rest the mouse over a number for a moment, a card shows all notations and, for addresses, the chip register with name, access and meaning of the bits. If the number stands after `poke`, the card also splits the value into its bit fields. The number tools (a dialog for converting, with a bit editor) and the hex suggestions for imported programs are not yet available in this version. The help page [BASIC](hilfe:basic) describes the BASIC as a whole.

## Directives in BASIC

A **directive** is an instruction to the translator, not to the device. It stands alone on a line, begins with `#` and has no line number. It costs neither memory nor run time.

| Line | Effect |
|---|---|
| `#include "path"` | inserts a file or a medium at this place (see below) |
| `#longnames` | allows long variable names |
| `#define NAME value` | from there on, replaces every whole word `NAME` with `value` (a constant without a variable, without parameters) |
| `#if SYSTEM_C64` ... `#else` ... `#endif` | conditional translation by target system (`SYSTEM_` and the identifier in capital letters), by a defined name or negated with `!` |
| `#dialect identifier` | chooses the dialect of this file |
| `#numbering start=10 step=10` | start and step of the line numbers for this file |
| `#asm "file.asm"` | includes machine code (see below) |

```basic
#define RAHMEN 53280
#if SYSTEM_C64
poke RAHMEN, 2
#endif
```

After building, `poke 53280, 2` stands there. The translator does not know other `#` lines; it treats them as an ordinary BASIC line, and the device reports a syntax error when running.

## Converting: labels, numbers, renumbering

A BASIC program has either line numbers or labels, and you can switch from one to the other at any time. Three commands do this, each with a dialog and a preview:

- **BASIC: Convert to labels ...** replaces line numbers and the jump targets behind them with labels (`@name`). Names that you assigned yourself are kept. VintageCoder derives new names from comments and marks them with a marker line.
- **BASIC: Convert to line numbers ...** assigns line numbers by start and step and replaces the labels in the jump targets with numbers. The names stay in marker lines (`;@name`), so you can convert back to labels later.
- **BASIC: Renumber ...** assigns the line numbers anew by start and step and adjusts all jump targets. With a range, only the lines from one number to another are renumbered.

You find the commands in the menu **Edit > Numbers and BASIC**, in the right-click menu of a BASIC file and in the command palette. They work in a BASIC file of the project; in an unnamed tab, VintageCoder reports "This only works in a BASIC file of the project."

### Step by step: convert a program to numbers

1. Open the `.bas` file and choose **Edit > Numbers and BASIC > BASIC: Convert to line numbers ...**. The dialog opens and calculates a preview right away.
2. At the top are a short explanation and the **mode** of the program: line numbers, labels, mixed or not yet determined. Next to it are start and step, for example "Start 10, step 10".
3. If the program includes other BASIC files, there is the check box **Convert included files too** with the number of files. Included media are only shown in the dialog ("they are not changed").
4. For **Renumber**, you can enter the lines **From** and **To** under **Range**. If you leave both fields empty, the whole program is renumbered. If you enter only one, the dialog asks you to fill in both.
5. Under **Messages** are warnings and errors of the converter with code and line. The **Preview** lists each changed line with its old and new version and sums up: "n lines change, k new labels", or "Nothing to change."
6. Click **Apply**. The change is a single step for Undo (Cmd/Ctrl+Z) in this file. With **Cancel**, you close the dialog without changing anything.

![The 'Code' tab with test.bas and above it the dialog 'In Nummern umwandeln'. It explains the step, shows mode 'Labels', start 10 and step 10, and a preview of seven lines, old on the left and new on the right. At the bottom the buttons 'Abbrechen' and 'Anwenden'.](bild:basic-umwandeln-dialog "The 'In Nummern umwandeln' dialog with a preview of the changed lines.")

> **Caution:** If an included file that is converted too has unsaved changes, VintageCoder stops and names the file: Save it first, or switch off **Convert included files too**. VintageCoder writes included files directly; their Undo then does not apply across tabs. If the text changes while the dialog is open, it calculates the preview again.

You set start and step for building on the page **Sources** in the area **Project**, in the card **BASIC** (section **Line numbers**, button **Save**). There you also find **Build compact**, the **Syntax check when building** and the **Disabled messages**. A line `#numbering start=... step=...` in the file takes precedence over these values.

## Dialects: which BASIC is it?

The target system of the project decides the dialect:

| System | Dialect | Program start | Result |
|---|---|---|---|
| C64 | BASIC V2 | `$0801` | `.prg` |
| VIC-20 | BASIC V2 | depends on the memory expansion | `.prg` |
| C128 | BASIC 7.0 | `$1C01` | `.prg` |
| Plus/4 | BASIC 3.5 | `$1001` | `.prg` |
| MEGA65 | BASIC 65 | `$2001` | `.prg` |
| Atari 8-bit | Atari BASIC or Turbo BASIC XL | in SAVE format | `.bas` |

You can change the dialect for each **source group**. It is in the project file (`.vcprj`), for a group in the entry `dialect`; without an entry, the default of the target system applies. This way, on the Atari you choose Turbo BASIC XL, for example:

```text
{ "targets": ["atari8"], "kind": "basic", "dialect": "turbo_basic_xl", "files": ["balken.tur"] }
```

The page **Sources** in the area **Project** shows the dialect of each BASIC group in its heading; there is no choice to change it there in this version, you edit the project file as text. For a single file, a line `#dialect <identifier>` at the start of the file also applies.

> **Caution:** If you choose the wrong dialect, the build stays **without a message**. A command that the dialect does not know is simply translated as a string of letters, that is, like a variable name. An example: `GRAPHIC 1,1` builds in a C64 project, because `GRAPHIC` is not a command for BASIC V2 (other dialects like BASIC 7.0 know it). You notice it only in the emulator. So check whether the source group has the right dialect (page **Sources** in the area **Project**).

Some rules that you should remember:

- **Uppercase and lowercase:** You may write commands and names in lowercase, uppercase or mixed. Inside quotation marks, it matters: On the C64, **lowercase letters appear as uppercase**, **uppercase letters as graphics characters**. If you want uppercase on the screen, write the text in **lowercase**.
- **German umlauts** (a, o, u with two dots) and the sharp s do not exist in the character set of the C64. A `print` with such a letter in the text leads to the error B100: *character '...' has no PETSCII equivalent* (the message names the letter).
- **Control characters** you write in curly brackets: `{clr}`, `{home}`, `{wht}` (white), `{down*3}` (cursor down three times). `{$93}` for any byte also works. The names differ for each system; they are in the help page [BASIC: dialects and target systems](hilfe:basic-dialects).
- **Several statements** in one line are separated by `:`.
- **The question mark** `?` is the abbreviation for `print`.

## Run Turbo BASIC XL

Turbo BASIC XL runs on the Atari from a **disk** that you must own yourself. Turbo BASIC XL is not part of VintageCoder and must be provided by the user. Select a disk from a source you may use. There is no telemetry: the disk is not transmitted anywhere and stays only on your computer.

### Step by step: provide the disk and run

1. Open a project with the BASIC dialect **Turbo BASIC XL** (for example the sample `examples/atari/turbo_basic`).
2. Press **F5**. Without a disk, the banner "This project needs a Turbo BASIC XL disk. It is not part of VintageCoder." appears above the editor. Click **Provide ...**. You get the same through **Project > Provide Turbo BASIC XL disk ...**, through the card **Turbo BASIC XL** on the page **Disk** in the area **Project** or through the command palette.
3. In the assistant, at step **1 · Choose file**, click **Choose file ...** and look for your disk (a disk image, `.atr`). It is best to work with a **copy**; the original stays unchanged in any case.
4. Step **2 · Check** shows, for example, "Turbo BASIC XL 1.5 recognized" and the free sectors. If the file does not look like Turbo BASIC XL, the assistant asks "This does not look like Turbo BASIC XL. Use it anyway?" You choose **Yes, use it anyway** or **No, choose another file**.
5. At step **3 · Copy**, you decide about **reducing**: **Yes, reduce in the project copy** or **Do not reduce**. On the original disk only about 12.9 KB are free; reduced (only the copy in the project, to `DOS.SYS`, `DUP.SYS`, `AUTORUN.SYS`) it is about 60 KB. If you do not reduce, programs that are too large report the error only when building.
6. Click **Copy**. The copy is then in `turbo/`, the project tree marks it as "Disk (user)", and `turbo/*.atr` is listed in the `.gitignore`. The window reports "Done" with the number of free sectors.
7. Press **F5**. VintageCoder builds `build/<name>.atr` (your program as `AUTORUN.BAS`) and boots it in the atari800 emulator. At the end of the program, Turbo BASIC XL shows `READY` and waits for input.

*[Bild folgt in E13: turbo-diskette-assistent]*

The card **Turbo BASIC XL** (area **Project**, page **Disk**) shows the state ("Disk ready" or "Disk missing"), the copy in the project, the free sectors and the files on the disk. With the buttons **Replace ...**, **Check**, **Reduce ...** and **Open in the Disk Editor**, you maintain the copy. The field **Program name on the disk** is a name like `AUTORUN.BAS`; Turbo BASIC XL starts only this name by itself. Under **Additional files on the disk**, you enter one source from the project per line, optionally with a name on the disk (`daten/hallo.dat:HALLO.DAT`). **Save** applies the changes. The status bar shows "Turbo disk: ..." with the free sectors.

> **Info:** **Debugging** is not yet possible for Turbo BASIC programs; the command reports this. More, including space, additional files and error messages, is on the help page [Turbo BASIC XL: disk](hilfe:turbo-disk).

## Including media from the editors

You draw sprites, character sets, screens and songs in the editors (see [Graphics and sound overview](kapitel:grafik-ueberblick)). For BASIC, you include the **media file** itself: When building, VintageCoder turns it into `DATA` lines that you read with `READ`.

```
#include "graphics/hero.vcsprite"
```

- The generated lines begin with a label `@name:` (the name of the medium). If a medium has several parts, each part has its label.
- The lines have no numbers of their own; they are numbered along at the place of the `#include` line.
- A medium is included only once per program; VintageCoder skips a second `#include` of the same medium without a message.
- If the translator does not find the medium or cannot output it as `DATA`, it reports **B193**. An old path `build/export/...` keeps working but is objected to with **B194**. If the program with its data becomes larger than the free memory, it reports **B195** (C64: 38911 bytes, VIC-20 without expansion: 3583 bytes, Plus/4: 60671 bytes).
- When converting (see above), VintageCoder reads included media along but never writes them back.
- There is no `.incbin` in BASIC; VintageCoder refuses binary files there.

A complete example with a sprite is in the help page [BASIC: directives, files and assets](hilfe:basic-directives). Important on the C64: BASIC V2 reads `DATA` **strictly in order**. So include the media in the order in which you read them. VintageCoder reports several media together with `RESTORE @name` on the C64 and VIC-20 with **B196**; on C128, Plus/4, MEGA65 and Atari, `RESTORE @name` works.

## Machine code in BASIC: `#asm`

If you want to write the fast part of a program in assembler, you include it with `#asm`. The assembler translates the file when building, and you call the code through its labels:

```basic
#asm "player.asm"
sys asm.start
```

- `#asm "file.asm"` names the assembler file, optionally with `with "song.asm"` for a song as well (a medium like `music/tune.vcsong` is also possible).
- Every label of the code is available in BASIC as `asm.name`. When building, VintageCoder replaces it with the address, for example `sys asm.start`. In addition, there are `asm.base_addr` (start of the code) and `asm.base_page` (the page, for `POKE 106`).
- On **Commodore systems**, the code is **behind the BASIC text** in the program; it begins with `.org asm_base`. On **MEGA65 and Atari**, VintageCoder delivers the code as `DATA` lines from the label `@asm_data`, which the program itself loads to the fixed address; on the Atari it is below `$A000` at a page boundary.
- The earlier spelling `#blob` does not exist any more. The translator reports **B177** with the note that it is now called `#asm`.

This way is meant for a program that stays BASIC at its core. If you want the whole program as machine code, you convert it (next section).

## Turning BASIC into assembler

BASIC is convenient but slow. VintageCoder can translate a BASIC program into readable **assembler source text** (the translator is called `basic2asm`). The program then no longer runs in the BASIC interpreter but as a machine program: faster, and you can look at every line and change it.

The command **File > Convert BASIC to assembler** is not yet available in the user interface in this version. You have two other ways:

- **When building.** In the project file (`.vcprj`), you enter an entry `convert` for the BASIC source group. VintageCoder then translates the source to assembler first at every build and builds the result; the `.bas` file stays the source:

```text
{ "targets": ["c64"], "kind": "basic", "convert": { "target": "c64" }, "files": ["main.bas"] }
```

  Without further settings, the translator stores the file `<name>_asm.asm` next to the source.
- **On the command line.** `vcc basic2asm programm.bas --target c64 -o programm.asm` creates the assembler file, which you then build like any source text. All options are in the help page [BASIC: tools and debugging](hilfe:basic-tools).

The converter chooses the needed precision itself. If your program needs only whole numbers, it calculates with 16-bit numbers (stage 1); the converter points this out (note I110: *variable A is a floating-point number in the original; stage 1 uses 16-bit integers*). This is a note and not an error. Fractions, decimal numbers and functions like `SIN` lead to the floating-point stage automatically.

> **Info:** The converter does not translate everything, above all no commands that a dialect has in addition (for example sprite commands of the C128). It then reports an error.

## Common stumbling blocks

- **The build reports nothing, but the program does nonsense or reports `?SYNTAX ERROR`.** The translator does not check syntax. Check for typing errors and the dialect of the source group.
- **A `print` with an umlaut in the text stops with an error.** Umlauts do not exist in the character set of the C64; write `groesse` instead of the word with the umlaut.
- **The variable has the wrong value.** On the C64, only the first two characters of the name count. Put `#longnames` at the start of the file.
- **The text appears in graphics characters.** In quotation marks, **uppercase letters** become graphics characters. Write text in lowercase.
- **An unknown `#` line.** The translator knows only the directives from the table further up. It treats everything else (for example `#blob`, `#ifdef`) as an ordinary BASIC line, and the device reports a syntax error when running.
- **The dialog "Convert" refuses.** Check whether the tab shows a BASIC file of the project and whether included files have unsaved changes.
- **A label is not found.** Labels begin with `@` and, in their definition, end with `:`; at the jump, `@name` stands without a colon. The translator reports labels defined twice (error B111).
- **The line is too long.** After translation, a line may have at most 250 bytes (Commodore) or 255 bytes (Atari); otherwise the build reports B102.

## What next?

Next you learn how to [build and run](kapitel:bauen-starten) your program and read the messages. If the way through assembler appeals to you, read [Assembler (VCASM) for beginners](kapitel:assembler). For looking things up: the help pages [BASIC](hilfe:basic), [BASIC: labels and line numbers](hilfe:basic-labels) and [BASIC: long variable names](hilfe:basic-names).
