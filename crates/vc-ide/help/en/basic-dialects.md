# BASIC: dialects and target systems

Each system has its own BASIC, and VintageCoder builds each one in its original dialect. The project's target system decides the **dialect**. You can change it per source group in the project file (`"dialect"` in the `.vcprj`); the **Project** area, page **Sources**, shows it for each source group. For a single file, the line `#dialect id` applies (see [Directives](help:basic-directives)). This page shows what each dialect can do, how text and control characters are written, and what is special about each one. It belongs to the [BASIC manual](help:basic).

## Overview

| System | Dialect (identifier) | Program start | Result | Variable names |
|---|---|---|---|---|
| C64 | BASIC V2 (`cbm_v2`) | `$0801` | `.prg` | 2 characters count |
| VIC-20 | VIC-BASIC 2.0 (`vic_basic_2`, builds on `cbm_v2`) | depends on the memory expansion (`$1001`, `$0401` or `$1201`) | `.prg` | 2 characters count |
| C128 | BASIC 7.0 (`basic7`) | `$1C01` | `.prg` | 2 characters count |
| Plus/4 | BASIC 3.5 (`basic35`) | `$1001` | `.prg` | 2 characters count |
| MEGA65 | BASIC 65 (`basic65`) | `$2001` | `.prg` | 2 characters count |
| Atari 8-bit | Atari BASIC (`atari_basic`) | in SAVE format | `.bas` | all characters count |
| Atari 8-bit | Turbo BASIC XL (`turbo_basic_xl`) | in SAVE format | `.bas` | all characters count |

- The **program start** address is the load address of the `.prg` file. It is stored in the first two bytes.
- On the VIC-20 it depends on the **memory expansion** (project setting, on the command line `--memory 8k`).
- The dialect applies to the **source group**. If one file needs a different dialect (for example Turbo BASIC XL instead of Atari BASIC), write `#dialect turbo_basic_xl` at the start of the file. This line takes precedence over the setting of the project. An unknown identifier reports **B178**, an identifier that does not fit the target system reports **B170** (as a warning).

If you choose the wrong dialect, the build gives no message. A command that the dialect does not know is translated as a sequence of letters, like a variable name. You notice this only on the machine.

## Text, characters and control characters (Commodore)

On the C64 and its relatives, these rules apply in strings:

- **Lowercase letters** appear as uppercase letters on the screen (this is how the computer shows text in its normal state). **Uppercase letters** appear as graphics characters. So write text in lowercase. Outside of strings, the case of commands and names does not matter.
- Digits, punctuation and `@ [ ] ^ _` are as on the keyboard. Accented letters such as `é`, the German letters with umlauts and the sharp s, `\`, `{` and `}` (except as control characters) have no equivalent and give the error **B100** (`character 'é' has no PETSCII equivalent`).
- **Control characters** go in curly braces: `{clr}`, `{home}`, `{wht}` ...
- `{name*n}` repeats a character `n` times: `{down*3}` is three cursor-down characters.
- `{$xx}` writes any byte in hexadecimal, `{nnn}` in decimal (`{147}`).
- Uppercase or lowercase, and `_` or `-` instead of spaces, do not matter (`{rvs_on}`). The names come from the same table as in the assembler (`data/chars/`). BASIC writes `{orng}`, `{return}`, `{rght}`, and the assembler spellings (`{orange}`, `{ret}`, `{right}`) are valid too. When you list a program (`LIST`), you see the names of the dialect.
- In the editor, the entries are colored, and unknown names have a red wavy underline. A hover shows the code and its effect, and after `{` the editor offers the names.

The following lines give the same program:

```basic liste
print "{clr}hello"
print "{$93}hello"
print chr$(147);"hello"
```

```text
10 print "{clr}hello"
20 print "{clr}hello"
30 print chr$(147);"hello"
```

(The third line is a completely different program that only has the same effect: `chr$(147)` is calculated only when the program runs.)

### Names of the control characters

| Name | Code | Name | Code | Name | Code |
|---|---|---|---|---|---|
| `{blk}` | `$90` | `{wht}` | `$05` | `{red}` | `$1C` |
| `{cyn}` | `$9F` | `{pur}` | `$9C` | `{grn}` | `$1E` |
| `{blu}` | `$1F` | `{yel}` | `$9E` | `{orng}` | `$81` |
| `{brn}` | `$95` | `{lred}` | `$96` | `{gry1}` | `$97` |
| `{gry2}` | `$98` | `{lgrn}` | `$99` | `{lblu}` | `$9A` |
| `{gry3}` | `$9B` | `{clr}` | `$93` | `{home}` | `$13` |
| `{down}` | `$11` | `{up}` | `$91` | `{rght}` | `$1D` |
| `{left}` | `$9D` | `{rvs on}` or `{rvon}` | `$12` | `{rvs off}` or `{rvof}` | `$92` |
| `{inst}` | `$94` | `{del}` | `$14` | `{return}` | `$0D` |
| `{shift return}` | `$8D` | `{lower}` | `$0E` | `{upper}` | `$8E` |
| `{stop}` | `$03` | `{pound}` | `$5C` | `{pi}` | `$FF` |
| `{f1}` to `{f8}` | `$85 $89 $86 $8A $87 $8B $88 $8C` | | | | |

`{rvs on}` and `{rvon}`, `{rvs off}` and `{rvof}` are two spellings of the same character. An unknown name is an error:

```basic fehler
print "{foo}"
```

```text
main.bas:1: error B100: unknown control code {foo}
```

The table applies to all Commodore dialects. The machine decides whether it has the color or function. The Plus/4, for example, has its own color table and assigns the codes differently.

The build also reports missing closing braces:

```basic fehler
print "{clr"
```

```text
main.bas:1: error B100: '}' is missing
```

## C64 and VIC-20: BASIC V2

BASIC V2 is the smallest Commodore BASIC. Commands:

`END FOR NEXT DATA INPUT# INPUT DIM READ LET GOTO RUN IF RESTORE GOSUB RETURN REM STOP ON WAIT LOAD SAVE VERIFY DEF POKE PRINT# PRINT CONT LIST CLR CMD SYS OPEN CLOSE GET NEW`

Functions and operators: `TAB( SPC( TO FN THEN NOT STEP AND OR SGN INT ABS USR FRE POS SQR RND LOG EXP COS SIN TAN ATN PEEK LEN STR$ VAL ASC CHR$ LEFT$ RIGHT$ MID$`, plus `GO` (for `GO TO`). Variables with a special meaning: `ST`, `TI`, `TI$`, `DS`, `DS$` (you cannot assign to them).

Special features:

- **No graphics or sound commands.** On the C64, you use `POKE` and `PEEK` on the chips (VIC-II, SID). Example: border and background color:

```basic liste
poke 53280, 0
poke 53281, 0
print "{clr}{wht}hello"
```

```text
10 poke 53280, 0
20 poke 53281, 0
30 print "{clr}{wht}hello"
```

- **`RESTORE` without a line.** It resets to the first `DATA`. The dialect does not know `RESTORE @name` (with a line); see [Assets](help:basic-directives).
- **`DEF FN`**: functions with one parameter, called with `fn name(x)`.
- **Line length:** If you type on the machine, the line ends after 80 characters. Loaded programs may be longer (up to 250 bytes), but the machine can no longer edit them.
- The **choice of the keyword** follows the order of the machine's command table: the first match wins. This is why `GOTO` is not split into `GO` and `TO`.

The VIC-20 uses the same BASIC. The memory addresses (`POKE` on the VIC chip) are different:

```basic liste system=vic20
poke 36879, 8
print "{clr}hello vic-20"
```

```text
10 poke 36879, 8
20 print "{clr}hello vic-20"
```

## C128: BASIC 7.0

BASIC 7.0 extends V2 with graphics, sound, sprites, error handling, loops and disk commands. In addition to the V2 commands, it has, among others:

- **Graphics:** `GRAPHIC COLOR BOX CIRCLE DRAW PAINT CHAR LOCATE SCALE GSHAPE SSHAPE SCNCLR WINDOW WIDTH RGR RCLR RDOT`
- **Sprites:** `SPRITE MOVSPR SPRCOLOR SPRSAV SPRDEF COLLISION RSPPOS RSPRITE RSPCOLOR BUMP`
- **Sound:** `SOUND VOL PLAY TEMPO ENVELOPE FILTER`
- **Program flow:** `DO LOOP UNTIL WHILE EXIT` (`DO ... LOOP` loops), `ELSE` (for `IF ... THEN`), `BEGIN ... BEND` (command blocks), `TRAP RESUME ERR$ TRON TROFF`
- **Memory and disk:** `BANK FETCH STASH SWAP DIRECTORY CATALOG DOPEN DCLOSE APPEND DSAVE DLOAD BSAVE BLOAD DVERIFY DCLEAR HEADER SCRATCH COLLECT COPY RENAME BACKUP CONCAT BOOT RECORD DELETE`
- **Other:** `KEY MONITOR USING PUDEF HELP AUTO RENUMBER DEC HEX$ INSTR JOY POT PEN SLEEP FAST SLOW QUIT OFF XOR RWINDOW POINTER`

```basic liste system=c128
color 0, 1
color 4, 1
graphic 1, 1
circle 1, 160, 100, 60
box 1, 10, 10, 100, 60
sleep 3
graphic 0, 1
```

```text
10 color 0, 1
20 color 4, 1
30 graphic 1, 1
40 circle 1, 160, 100, 60
50 box 1, 10, 10, 100, 60
60 sleep 3
70 graphic 0, 1
```

- The translator **does not check the parameters** (syntax). The C128 manual tells you which numbers a command needs.
- You no longer need to build `do ... loop` with `exit` and `else` the way you do on the C64:

```basic liste system=c128
n = 0
do
  n = n + 1
  if n >= 10 then exit
loop
print n
```

```text
10 n = 0
20 do
30 n = n + 1
40 if n >= 10 then exit
50 loop
60 print n
```

- For **error handling**, the generated line number counts: `trap @fehler` inserts the number of the label, and `resume next` and `err$(er)` work with it. The variables `ER` and `EL` (error number, error line) are reserved.

## Plus/4: BASIC 3.5

BASIC 3.5 sits between V2 and 7.0. It has the graphics, sound, loop and error commands, but **no sprites**, no bank switching and no `PLAY`:

`RGR RCLR RLUM JOY RDOT DEC HEX$ ERR$ INSTR ELSE RESUME TRAP TRON TROFF SOUND VOL AUTO PUDEF GRAPHIC PAINT CHAR BOX CIRCLE GSHAPE SSHAPE DRAW LOCATE COLOR SCNCLR SCALE HELP DO LOOP EXIT DIRECTORY DSAVE DLOAD HEADER SCRATCH COLLECT COPY RENAME BACKUP DELETE RENUMBER KEY MONITOR USING UNTIL WHILE`

```basic liste system=plus4
graphic 1, 1
draw 1, 10, 10 to 100, 80
box 1, 20, 20, 90, 70
for t = 1 to 500 : next t
graphic 0, 1
```

```text
10 graphic 1, 1
20 draw 1, 10, 10 to 100, 80
30 box 1, 20, 20, 90, 70
40 for t = 1 to 500 : next t
50 graphic 0, 1
```

The Plus/4 has its own color table with brightness levels. `{wht}` and so on name the same codes as on the C64. The machine decides which color results.

## MEGA65: BASIC 65

BASIC 65 is the most extensive dialect. It contains almost everything from BASIC 7.0, plus commands for DMA, palette, mouse, screen modes, file system and languages. New compared to 7.0 (selection):

`BORDER BACKGROUND FOREGROUND HIGHLIGHT SCREEN PALETTE PIXEL DOT LINE POLYGON ELLIPSE VIEWPORT GCOPY CUT PASTE DMA EDMA DMODE DPAT MOUSE RMOUSE CURSOR RCURSOR FONT VSYNC SPEED FIND CHANGE ERASE DIR MKDIR CHDIR MOUNT LOADIFF SAVEIFF WPEEK WPOKE MOD LOG10`

```basic liste system=mega65
border 0
background 0
foreground 1
print "{clr}hello mega65"
vsync 100
```

```text
10 border 0
20 background 0
30 foreground 1
40 print "{clr}hello mega65"
50 vsync 100
```

A note for the translator: In BASIC 65, the **longest match** wins for the keyword (`FOREGROUND` before `FOR`). For names with `#longnames` this does not matter (see [Long names](help:basic-names)). Without `#longnames`, you must not use variables such as `foreground`. Foreign BASIC 65 programs that you import run unchanged.

## Atari BASIC

Atari BASIC is very different from the Commodore dialects: It checks each line when you enter it, stores numbers in BCD format and knows long variable names. VintageCoder creates the file in **SAVE format** (`.bas`), which Atari BASIC can load with `LOAD` or `RUN`.

Commands: `REM DATA INPUT COLOR LIST ENTER LET IF FOR NEXT GOTO GO TO GOSUB TRAP BYE CONT COM CLOSE CLR DEG DIM END NEW OPEN LOAD SAVE STATUS NOTE POINT XIO ON POKE PRINT RAD READ RESTORE RETURN RUN STOP POP ? GET PUT GRAPHICS PLOT POSITION DOS DRAWTO SETCOLOR LOCATE SOUND LPRINT CSAVE CLOAD`

Functions: `STR$ CHR$ USR ASC VAL LEN ADR ATN COS PEEK SIN RND FRE EXP LOG CLOG SQR SGN ABS INT PADDLE STICK PTRIG STRIG`

```basic liste system=atari8
graphics 0
poke 710, 0 : poke 712, 0
@neu:
position 8, 10 : ? "atari basic with labels"
goto @neu
```

```text
10 GRAPHICS 0
20 POKE 710,0:POKE 712,0
30 POSITION 8,10:? "atari basic with labels"
40 GOTO 30
```

Differences from the Commodore way:

- **Lowercase in text stays lowercase.** In strings, Atari BASIC knows ASCII/ATASCII from space to `~`. Characters above that, such as accented letters, give an error (see below).
- **Control characters** in `{...}`: `{clr}` (clear screen, `$7D`), `{up}`, `{down}`, `{left}`, `{right}`, `{esc}`, `{tab}`, `{del}`, `{eol}`, `{bell}` and any byte as `{$xx}`.
- **The translator checks the syntax of the statements** (unlike for Commodore): It reads each statement according to the grammar of Atari BASIC and reports errors such as `'=' expected after 'X'` (**B180**), `expression is incomplete` (**B181**) or `unexpected character '_'` (**B100**). The codes **B180** to **B185** belong to the Atari: B180 statement, B181 expression, B182 name, B183 number, B184 text, B185 limits (too many variables, line too long, line number too large). The `'` as a comment does not exist there either.
- The **line numbers** go up to 32767. A line may be at most 255 bytes long.
- Names can be any length, but there can be at most 128 different variables. `_` is not allowed in names (see [Long names](help:basic-names)).
- **Labels** work as everywhere: `goto @neu`, `gosub @x`, `trap @fehler`, `on a goto @a, @b`.
- The detailed tokenization was compared with the real Atari BASIC (in the atari800 emulator).

```basic fehler system=atari8
x =
```

```text
main.bas:1: error B181: expression is incomplete
```

Characters outside the ATASCII range are reported in the same way:

```basic fehler system=atari8
? "café"
```

```text
main.bas:1: error B184: character 'é' is not in the ATASCII range (use {$xx})
```

## Turbo BASIC XL (Atari)

Turbo BASIC XL is a faster, extended Atari BASIC. You choose it when you create the project (dialog "New project", BASIC dialect), as the dialect of the source group (in the `.vcprj` `"dialect": "turbo_basic_xl"`), for a single file with `#dialect turbo_basic_xl`, or on the command line with `--dialect turbo_basic_xl`. In addition to Atari BASIC, the translator knows:

- **Structures:** `PROC name ... ENDPROC`, `EXEC name`, `WHILE ... WEND`, `REPEAT ... UNTIL`, `DO ... LOOP`, `EXIT`, multi-line `IF ... ELSE ... ENDIF`.
- **Jump labels:** `GO# name` and `# name` (own label in Turbo BASIC XL); `ON x EXEC a, b` and `ON x GO# a, b`. `@name` also works as always.
- **Commands:** `DPOKE MOVE -MOVE BPUT BGET FILLTO CIRCLE PAINT FCOLOR PAUSE TEXT CLS DSOUND TRACE DUMP DEL RENUM BLOAD BRUN DIR LOCK UNLOCK RENAME DELETE TIME$= *F *B *L %PUT %GET`.
- **Functions and operators:** `DIV MOD EXOR & !`, hex numbers `$FF` (stay as they are), binary numbers from two digits (`%1010`, become decimal when you build), `%0` to `%3` alone as constants, `DPEEK INSTR UINSTR INKEY$ HEX$ DEC FRAC TRUNC RAND TIME TIME$ ERR ERL` and `RND` without parentheses.
- **Comments:** `'` and `.` at the start of a line (both give a `REM` line), and also `--`.
- **Variables:** up to 256. The spelling of the first occurrence is kept.
- **Abbreviations** such as `G.`, `PR.`, `POK.`.

Not supported: parameters for `PROC`/`EXEC` and `$define` macros (these are extensions of a third-party preprocessor, not part of Turbo BASIC XL).

```basic liste system=atari8 dialect=turbo_basic_xl
graphics 0
exec titel
spalte = 0
while spalte < 4
  exec balken
  spalte = spalte + 1
wend
repeat
  position 4, 20 : ? "press a key";
until peek(764) <> 255
end

proc titel
  position 8, 2 : ? "turbo basic xl"
endproc

proc balken
  if spalte mod 2
    setcolor 2, spalte, 4
  else
    setcolor 2, spalte, 8
  endif
  pause 3
endproc
```

```text
10 GRAPHICS 0
20 EXEC titel
30 spalte=0
40 WHILE spalte<4
50 EXEC balken
60 spalte=spalte+1
70 WEND
80 REPEAT
90 POSITION 4,20:? "press a key";
100 UNTIL PEEK(764)<>255
110 END
120 PROC titel
130 POSITION 8,2:? "turbo basic xl"
140 ENDPROC
150 PROC balken
160 IF spalte MOD 2
170 SETCOLOR 2,spalte,4
180 ELSE
190 SETCOLOR 2,spalte,8
200 ENDIF
210 PAUSE 3
220 ENDPROC
```

Own labels of Turbo BASIC XL (`#name`) and `@name` work side by side:

```basic liste system=atari8 dialect=turbo_basic_xl
exec rahmen
go# ende
proc rahmen
  ? "frame"
endproc
# ende
? "done"
```

```text
10 EXEC rahmen
20 GO# ende
30 PROC rahmen
40 ? "frame"
50 ENDPROC
60 # ende
70 ? "done"
```

**To run it**, you need Turbo BASIC XL itself; it is not part of VintageCoder. You provide your own disk (**Project** area, page **Disk**, card "Turbo BASIC XL"), VintageCoder writes your program as `AUTORUN.BAS` into a copy of it and starts it in the emulator; see [Turbo BASIC XL: disk](hilfe:turbo-disk). You can also load the built `.bas` file by hand. The output is checked byte by byte against an independent tokenizer, not against the original program in an emulator. The build rejects lines that are too long (over 255 bytes).

## The same task on every system

Black border and background, clear the screen:

| System | Lines |
|---|---|
| C64 | `poke 53280,0 : poke 53281,0 : print "{clr}"` |
| VIC-20 | `poke 36879,8 : print "{clr}"` (background and border share one register) |
| C128 | `color 4,1 : color 0,1 : print "{clr}"` |
| Plus/4 | `color 4,1 : color 0,1 : print "{clr}"` |
| MEGA65 | `border 0 : background 0 : print "{clr}"` |
| Atari | `graphics 0 : poke 710,0 : poke 712,0` |

Because the commands differ, a program for several systems needs its own file or its own part for each system (see [Directives](help:basic-directives)).
