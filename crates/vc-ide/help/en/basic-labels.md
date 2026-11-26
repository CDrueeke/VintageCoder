# BASIC: Labels and Line Numbers

In classic BASIC you jump with `GOTO 100` to a **line number**. This has two drawbacks: you have to number every line, and if you insert a line, you have to change numbers or have left space. Here you use **labels** instead: names for places in the program. When building, the translator assigns the line numbers itself and puts them in place of the labels. The result is an ordinary numbered BASIC program. The page belongs to the [BASIC manual](help:basic).

## Define a label: `@name:`

```
@name:
@name: statement
100 @name: statement
```

- The label begins with `@` and ends with a **colon**. It is at the **start of the line**, and in a line with a number directly after the number.
- The name consists of letters, digits and `_` and does not begin with a digit. **Uppercase and lowercase do not matter:** `@Ende` and `@ende` are the same label.
- If nothing follows the label, **it applies to the next statement** below. This is the usual way to write it, because it makes the label easy to see.
- If a statement follows the label, the label belongs to this line.
- Several labels in a row (`@a: @b: print 1`) name the same line.
- The same place may also have a number of its own (`100 @start: print "x"`); then the label has this number.

The example shows all three ways to write it. They give the same program:

```basic liste
@a:
print "x"
@b: print "y"
100 @c: print "z"
goto @a : goto @b : goto @c
```

```text
10 print "x"
20 print "y"
100 print "z"
110 goto 10 : goto 20 : goto 100
```

### Use a label: `@name`

Everywhere that classic BASIC would have a line number as a **jump target**, you write `@name`:

| Statement | Example |
|---|---|
| `GOTO` | `goto @start` |
| `GOSUB` | `gosub @zeichnen` |
| `THEN` and `ELSE` | `if x = 1 then @gewonnen` or `if x = 1 then @ja else @nein` |
| `ON ... GOTO` and `ON ... GOSUB` | `on wahl goto @a, @b, @c` |
| `RESTORE` with a line (C128, Plus/4, MEGA65, Atari; not C64 and VIC-20) | `restore @daten` |
| `TRAP`, `RUN`, `LIST`, `RESUME` and everything else with a line number | `trap @fehler` |

The translator replaces **every** `@name` in the BASIC text with the line number, everywhere except inside a string (`"..."`), after `REM` or in a `DATA` line. There the text stays unchanged, so an `@` may appear in your texts.

Example with several kinds of jumps:

```basic liste
input "zahl"; n
on n goto @eins, @zwei
if n > 2 then @gross else @klein
@eins: print "eins" : end
@zwei: print "zwei" : end
@gross: print "gross" : end
@klein: print "kleiner als eins" : end
```

```text
10 input "zahl"; n
20 on n goto 40, 50
30 if n > 2 then 60 else 70
40 print "eins" : end
50 print "zwei" : end
60 print "gross" : end
70 print "kleiner als eins" : end
```

In `DATA`, in strings and after `REM`, nothing is replaced:

```basic liste
print "mail@host"
rem goto @nirgends
data @nein, 1
@wirklich:
goto @wirklich
```

```text
10 print "mail@host"
20 rem goto @nirgends
30 data @nein, 1
40 goto 40
```

### Lines that must not have a label

A label must be **at the start of the line**. In the middle of a line, `@name:` is not a definition. The message is then misleading, because the translator reads the word as a jump target that does not exist:

```basic fehler
a = 1 : @mitte: print a
goto @mitte
```

```text
main.bas:1: error B110: label @mitte is not defined
```

Write the label on a line of its own or at the start of the line.

## How the line numbers are created

When building, the translator goes through the file from top to bottom:

- A line **without a number** gets the next number. It starts at **10**, the step is **10** (10, 20, 30, ...).
- A line **with a number of its own** keeps it. After that it continues with this number plus the step. You can mix numbered and unnumbered lines.
- Empty lines, comment lines (`;` at the start), lines with only a label and all directive lines (`#include`, `#longnames`, `#define`, `#if` ...) get **no** number and cost nothing. `REM` lines, however, are real lines and are numbered.
- The numbers must **ascend**. A number of your own that is not greater than the previous one is reported by the build as error **B100**.
- The highest line number is 63999 (Commodore) or 32767 (Atari). If it is exceeded, the build reports B100 (Atari: B185).

```basic liste
print "a"
print "b"
200 print "c"
print "d"
```

```text
10 print "a"
20 print "b"
200 print "c"
210 print "d"
```

Error with a wrong order:

```basic fehler
100 print "a"
50 print "b"
```

```text
main.bas:2: error B100: line number 50 is not greater than the previous one (100)
```

A number that coincides with an automatically assigned one is also an error: if the line `10 print "b"` follows `print "a"` (number 10), the build reports B100.

### Step and start number

The default is start 10 and step 10. You change it in three places. The most specific one applies: a line `#numbering start=N step=N` in the file takes precedence over the values of the project, and these take precedence over the default.

- **In the file:** `#numbering start=100 step=5` (see [Directives](help:basic-directives)). If you write only `start=` or only `step=`, only that value changes.
- **In the project:** area **Project**, page **Sources**, card "BASIC", section "Line numbers" with the fields "Start" and "Step" (in the project this is the object `basic` of the `.vcprj`). There it applies to all BASIC files of the project.
- **On the command line:** `vcc basic` knows the switches `--start` and `--step`; they replace the default of the project.

```bash
vcc basic hallo.bas --start 100 --step 5 -o hallo.prg
```

```text
@a = line 100
hallo.prg: 24 bytes
```

The step must be at least 1, the start at most the highest number of the dialect (63999, Atari 32767); otherwise the build reports **B191**. If the numbers are not enough for all lines, it reports **B192** and names a smaller step.

*[Bild folgt in E13: basic-karte-nummerierung]*

Use a number of your own to get a different start in the project: `1000 print "start"` continues the count from 1000. This is also useful if a routine must have a **fixed line number**, for example because machine code or another program knows it:

```basic liste
print "haupt"
gosub 9000
end
9000 print "unterprogramm"
return
```

```text
10 print "haupt"
20 gosub 9000
30 end
9000 print "unterprogramm"
9010 return
```

How these line number jumps work together with `@name`: a classic `GOTO 9000` stays as it is. The translator changes only labels, never existing numbers.

## Convert between numbers and labels

A BASIC file is either in **number mode** (the lines have numbers, you jump with `GOTO 100`) or in **label mode** (labels `@name`, few or no numbers). The mode follows from the content. If a file mixes both, the build warns with **B160**. Three commands convert; they are in the menu **Edit ▸ Numbers and BASIC**, in the command palette and in the right-click menu of a BASIC file (without a shortcut):

| Command | Effect |
|---|---|
| **BASIC: Convert to labels ...** | Replaces line numbers and the jump targets behind them with labels. Only lines that are jumped to get a label. Names of your own stay; new names are made from the comment before the line (otherwise from the old number, for example `z120`) and carry a marker line `;@name auto`. |
| **BASIC: Convert to line numbers ...** | Assigns line numbers by start and step and replaces the labels in the jump targets with numbers. The names stay as a marker line `;@name` before the line, so that you can convert back to labels later. |
| **BASIC: Renumber ...** | Assigns the numbers anew and adjusts all jump targets. With a range ("From" and "To", fill in both or leave both empty), only lines from one number to another are renumbered. |

Each command opens a dialog (title **Convert to labels**, **Convert to line numbers** or **Renumber**) with a short explanation, the **Mode** of the file, the numbering (start and step), a **Preview** of the lines that change (old > new) and messages. Only **Apply** changes the text, in a single step that **Undo** takes back. **Cancel** changes nothing.

![The 'Code' tab with test.bas and above it the dialog 'In Nummern umwandeln'. It explains the step, shows mode 'Labels', start 10 and step 10, and a preview of seven lines, old on the left and new on the right. At the bottom the buttons 'Abbrechen' and 'Anwenden'.](bild:basic-umwandeln-dialog "The 'In Nummern umwandeln' dialog with a preview of the changed lines.")

- **Included files:** If the program has `#include` lines with BASIC files, the dialog offers "Convert included files too"; otherwise it converts only the open file. If included files have unsaved changes, the dialog asks you to save first. Media files (`.vcsprite` and others) are read but never changed.
- **Not possible** is the conversion to labels for computed jump targets such as `GOTO A*10` (**B162**, summary **B161**); the dialog names the places. A range can be renumbered only in number mode and must not change the order of the lines (**B161**).
- **Warnings:** `ER`, `EL` and similar words that give the line number at run time are reported as **B163**: after the conversion the numbers are no longer correct. A marker line with no statement behind it is reported as **B165**, an invalid or duplicate one as **B168**; if included files have a different mode from the others, the dialog reports **B166**.
- **Command line:** `vcc basic --to-labels file.bas`, `--to-numbers` and `--renumber` (also `--range from-to`). `--preview` lists only the changed lines, `--write` overwrites all changed files, `-o` writes the result of the main file to another file, without either it goes to the standard output.

```bash
vcc basic --to-labels spiel.bas --preview
vcc basic --renumber spiel.bas --start 100 --step 5 -o neu.bas
```

## See the mapping of label to line number

After building, there are several places where you can look up which line a label got:

- **Build log:** The step "Line numbers and labels" gives the number of lines and labels.
- **`build/<name>.labels.txt`:** one line per label, for example `ende                     line 70`.
- **`build/<name>.classic.txt`:** the finished program as text with line numbers, read back from the built program. This way you see exactly what the machine gets.
- **Editor:** The number that a line without a number of its own gets when building is not shown by the editor in this version yet.
- **Command line:** `vcc basic file.bas` prints `@name = line n` for every label.

You need this mapping mainly for **error messages of the machine**. If the C64 reports `?SYNTAX ERROR IN 40`, 40 is the generated number; a look into `.classic.txt` shows the matching source line.

## Errors and notes

| Code | Message | Cause |
|---|---|---|
| B110 | `label @x is not defined` | There is no `@x:` at the start of a line. A typo in the name? The label is in the middle of a line? |
| B111 | `label @x is defined twice` | Two lines carry the same label (uppercase and lowercase do not matter; the message names it in lowercase) |
| B112 | `label @x is at the end with no statement after it` | Nothing comes after the last label, so the label would have no line |
| B100 | `line number N is not greater than the previous one (M)` | Own numbers are not ascending, or they collide with the automatic number |
| B100 | `line number N is greater than 63999` | Too many lines or too large a number of your own (Atari: 32767, code B185) |

Examples:

```basic fehler
print "a"
goto @weg
```

```text
main.bas:2: error B110: label @weg is not defined
```

```basic fehler
@x:
print 1
@X:
print 2
```

```text
main.bas:3: error B111: label @x is defined twice
```

```basic fehler
print 1
@ende:
```

```text
main.bas:2: error B112: label @ende is at the end with no statement after it
```

The translator collects several messages and shows them all in the tab **Problems** in the tray below the editor.

## Pitfalls

- **Computed jumps:** `goto 100 + x * 10` or `on x goto 100, 200` with fixed numbers work only if you number the lines yourself. With automatic numbers you do not know the numbers in advance. Use `on x goto @a, @b, @c`.
- **`RESTORE @name` on C64 and VIC-20:** BASIC V2 has no `RESTORE` with a line number. The translator does not check this; the finished program stops on the machine with `?SYNTAX ERROR`. There, read the `DATA` lines in order (see [Include assets](help:basic-directives)).
- **Error handling lines:** `ER`, `EL`, `ERL`, `RESUME` and `TRAP` work with the **generated** numbers. The program works; you have to interpret the numbers in `EL` through `.classic.txt`.
- **Atari and Turbo BASIC XL:** `GOTO @name` works for both. Turbo BASIC XL also has its own names for jump targets (`#name`, `GO#`), see [Dialects](help:basic-dialects). You can mix both.
