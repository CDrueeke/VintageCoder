---
id: debugger
titel: The debugger
reihenfolge: 9
stufe: Advanced
zeit: 30 minutes
siehe_auch: [bauen-starten, editor, tests, systeme]
referenz: [debugger, breakpoints, memmap]
i18n_status: maschinell
i18n_quelle: fefa2fe009e3
---

# The debugger

**Goal:** You learn to stop a program in the emulator, run it step by step and watch what happens in the registers and in memory. This helps you find errors that you cannot see in the source code.

This chapter is at the "Advanced" level: it assumes that you can already build and run a first assembler program (see [Assembler (VCASM) for beginners](kapitel:assembler) and [Build and run](kapitel:bauen-starten)). What a register or a flag is, you can find in the glossary (`GLOSSAR.md`).

## What you need

- An open project that builds without errors.
- An emulator for your target system. For C64, C128, Plus/4 and VIC-20 this is VICE, for the Atari 8-bit it is atari800, for the MEGA65 it is Xemu (for setup, see [Welcome to VintageCoder](kapitel:willkommen) and [Target systems compared](kapitel:systeme)).
- The practice program below. You can also use the finished example `examples/c64/border`; the steps are the same.

## The practice program

Create a new C64 project named `farbwechsel` (see [Your first project](kapitel:erstes-projekt)) and replace the content of the source file with this program. In an endless loop, it switches the border color to the next one:

```asm
; Switch the border color - small practice program for debugger and tests (C64)
.target c64
.cpu 6510 { illegals = deny }
.const BORDER  = $d020
.const ZAEHLER = $fb            ; zero page: how often it has switched already

.org $0801
.basicstub start

start:  lda #0
        sta ZAEHLER
schleife:
        jsr naechste_farbe
        jmp schleife

; increases the counter and shows its lower nibble as the border color
naechste_farbe:
        inc ZAEHLER
        lda ZAEHLER
        and #15
        sta BORDER
        rts
```

The program is written without raster synchronization on purpose, so it runs as fast as the CPU can. Build it with **F7** (Project > Build project); this creates 34 bytes. Line numbers in this chapter refer to exactly this source text: `inc ZAEHLER` is in line 18, `sta BORDER` in line 21, `jsr naechste_farbe` in line 13.

## The Debug area

The debugger has its **own main section** in VintageCoder: **Debug** in the main row at the top (next to Project, Code, Graphics, Music and Data). Without a running session, it shows the card **No debug session** with the button **Start debugging**. During a session, the status bar at the bottom has an entry (for example "Debug: halted at $0817"); a click on it takes you back to the Debug area.

This is how you start a session:

1. Press **F6** (or choose **Project > Debug start program**, or click **Start debugging** in the Debug area).
2. VintageCoder builds the project, starts the emulator and stops the program **at the entry point**.
3. The user interface switches to the **Debug** area by itself.

With **Alt+Shift+F6** (menu **Project > Active file > Debug active file**), you debug the file that is active in the editor instead of the start program. If the last build failed, the Debug area shows the note "The last build failed. Fix the errors first." Then look in the **Problems** tab.

![The debug view with the Debug tab active. On the left are breakpoints, watch, snapshot and chips. In the middle are a toolbar, the disassembler starting at $0801 and the memory from $0400. On the right are the registers showing the paused state, the stack and the screen.](bild:debug-ueberblick "Example view, no emulator is running: the debug view with breakpoints, disassembler, memory and registers.")

The area is divided into three columns:

- **Left:** **Breakpoints**, **Watch**, **Snapshot** and **Chips**.
- **In the middle:** the toolbar with the step buttons and the state of the session, below it the **Disassembly** and the **Memory**.
- **Right:** **Registers** with the flags, the **Stack** and the **Screen** of the emulator.

In a narrow window, the columns move below each other. The machine panel (**Cmd/Ctrl+Alt+B**) also shows the registers and flags in a session, as well as a small image of the emulator; you have the full view in the Debug area.

> **Info:** While the emulator is running, registers, disassembly and memory show nothing: You can only read the state of the machine when it is paused. You then see the note "The program is running. Stopping shows registers, disassembly and memory."

### What the emulator can do

Not every emulator can do everything. The overview:

| Emulator | Registers | Single step | Breakpoints | Data breakpoints |
|---|---|---|---|---|
| VICE (C64, C128, Plus/4, VIC-20) | yes | yes | several | yes |
| atari800 (Atari 8-bit) | yes | yes | only one at a time | no |
| MEGA65 (Xemu or hardware) | yes | yes | one | only as a slow write watch by single-stepping |
| Ultimate 64 / II+ (network) | no | no | no | no |

With the Ultimate, you can read and write memory, and pause and continue, nothing more. atari800 cannot be paused with a button: There, you press **F8 in the emulator window** or set a breakpoint first. If the emulator cannot do a single step at the moment, the reason is shown as a note below the toolbar. At the top right of the toolbar is the name of the emulator that the session runs with.

## Breakpoints

A **breakpoint** tells the emulator: "Stop before you run this line."

### A breakpoint on a line

1. Switch to the area **Code** and **click in the margin to the left of the line number** of line 18 (`inc ZAEHLER`). A red dot appears there. Or put the cursor in the line and press **F9** (menu **Edit > Toggle breakpoint**).
2. A second click (or **F9**) removes the breakpoint again.
3. Press **F6**. The program starts and stops as soon as it reaches line 18. The toolbar then shows, for example, `Stopped at $0817`, and next to it the source line.

*[Bild folgt in E13: debug-haltepunkt-gutter]*

> **Info:** You can set breakpoints in the editor even before the start. VintageCoder assigns them to an address through the line mapping of the last build. If you changed the source text after building, build again (F7, or simply F6, which builds as well).

In the **Disassembly**, a click on the round button in front of a line also sets a breakpoint (see below).

The list **Breakpoints** on the left in the Debug area shows all points of the session: the kind (Execute, Read, Write or Access), the address or the address range, a condition (`if ...`), the number of hits to ignore (`x3`) and the number of hits so far (`2x`). The **x** at the end of the row deletes the point. Breakpoints from the editor are also in the list, faded and with the addition **Editor** and `file:line`. If there are none, it says "No breakpoints. A click left of a line sets one."

### Create breakpoints by hand: address, condition, data access

Below the list is a small form for breakpoints that you do not want to set through a source line:

1. Choose the **Kind of point**: **Execute**, **Read**, **Write** or **Access**. The last three are **data breakpoints** (the emulator stops when a memory location is accessed) and can be chosen only if the emulator can do this, that is, with VICE.
2. Enter the **Address**, for example `$D020`, `0x0400` or `2049`. A symbol name from your program does not work in this field; for an invalid entry, "That is not an address. Examples: $D020, 0x0400, 2049." appears.
3. For data breakpoints, the field **End (for a range)** is added. If you leave it empty, the point applies to the one address; otherwise to the range from address to end.
4. Under **Condition**, you can write an expression, for example `a == 5`. The point then stops only if the expression is true. The notation is in the section "Watch" below.
5. Under **Ignore**, you enter how many hits are skipped before the point stops.
6. Click **Add**.

*[Bild folgt in E13: debug-bedingung]*

If something is wrong with a condition, a red message below the list shows the error. You also know the expressions themselves from the chapter [Automatic testing](kapitel:tests).

## Single step

When the program is paused, you control it with the buttons of the toolbar or with the keys. The buttons have icons; if you move the mouse over them, the name appears. All commands are also in the menu **Project > Debug** and in the command palette (**Cmd/Ctrl+K**):

| Name (button and menu) | Key | What happens |
|---|---|---|
| **Debug: Continue** | Alt+F5 | runs to the next breakpoint |
| **Debug: Step into** | F11 | runs exactly one instruction; a `JSR` jumps *into* the subroutine |
| **Debug: Step over** | F10 | runs one instruction; a `JSR` runs *completely* and stops after it |
| **Debug: Step out** | Shift+F11 | runs to the return (`RTS` or `RTI`) of the current subroutine |
| **Debug: Pause** | Alt+F6 | pauses the running emulator (with atari800: F8 in the emulator window) |
| **Stop debugging** | Shift+F5 | ends the debug session |
| **Debug: Run to here** | none | runs to the line where the cursor is in the editor |

> **Info:** Step over, step into and step out work only if the program is paused; otherwise the buttons are gray. **Pause** can be chosen only while the program is running.

**Try it.** Before that, remove all breakpoints and set one on line 13 (`jsr naechste_farbe`) before you start the session:

1. Press **F6**. The program stops in line 13.
2. Press **F11** (Step into). You are now *in* `naechste_farbe`, on `inc ZAEHLER` (line 18).
3. Press **F11** four more times: `inc`, `lda`, `and` and `sta` are run, then the program counter is on the `rts`. After each step, watch the registers (see below).
4. Press **Shift+F11** (Step out). The `rts` is run, and the program stops after the call, in line 14 (`jmp schleife`).
5. Press **F10** (Step over) twice. Now the debugger stops in line 13 again, then it stands after the `jsr`. **Step over** ran the whole subroutine in one go.

**Step out** counts along: Calls inside the subroutine are skipped, otherwise VintageCoder runs single steps. After ten seconds without a return, it stops with a message. Then set a breakpoint directly at the return.

> **Tip:** If you only want to run to a certain place without setting a permanent breakpoint: Put the cursor in the line in the editor and choose **Project > Debug > Debug: Run to here**. This is a one-time breakpoint that disappears after the next stop. If the cursor is in no source file, the note "Put the cursor in a source file, then "Run to here"." appears.

## The disassembly

The **Disassembly** in the middle shows the **machine code from the program counter on**, as the CPU sees it: breakpoint button, address, bytes, instruction and, on the right, the number of cycles (a `+` after it means: possibly one cycle more).

![The disassembler as a list: each line has a breakpoint circle, address, bytes, instruction and the cycle count on the right. The first line at $0801 is highlighted and links to jumprun.asm, and the label start: appears further down.](bild:debug-disassembler "Example view: the disassembler with address, bytes, instruction and cycles on each line.")

Useful things:

- The line where the program stands is highlighted. On the right in this line is a link `file:line`; a click on it jumps to this source line in the editor (the area switches to **Code**).
- The **round button** at the left edge of each line is a breakpoint: A click sets it (it turns red), a second click removes it. The breakpoint applies to the address of this line.
- If labels from your source text are at an address, you see them in front of the instruction (`naechste_farbe:`).
- Instructions that are not part of the official instruction set at all are shown in a different color.

## The registers

The column on the top right shows the state of the CPU. Next to it, it says whether the session is **PAUSED** or **Running**.

![A narrow side column with the registers PC $0801, A $00, X $00, Y $00 and SP $F8, the flags N, V, B, D, I, Z and C with I highlighted, the stack, an empty screen area and some switches. At the top right it says 'ANGEHALTEN' (halted).](bild:debug-register "Example view: the debug column with registers, flags, stack and screen.")

- **PC**: address of the next instruction (the one that is *not yet* run).
- **A**, **X**, **Y**: the calculation registers.
- **SP**: the stack pointer (the stack is on page 1, so from `$0100`).
- The **flags** N V B D I Z C as a bar of boxes. A flag that is set is highlighted. The bar is display only; a click changes nothing in the emulator.
- Each register is shown as a hex value in a field. If a value changed in the last step, its field is highlighted; a changed flag has an outline.
- With some emulators, further values are added, for example the raster position (with VICE).

Below it, the **Stack** shows the bytes that are on the stack, each with its address (`$01FF` downwards). If it is empty, it says "Stack empty".

### The chips

On the left in the Debug area is the section **Chips**. For your target system, it lists the components (for example the video chip or the sound chip) as check boxes. If you tick one, a window with the **registers of this chip** appears below it: address, name and current value (`$..`). If you move the mouse over a register, you see its description. The values are updated at every stop; while the program is running, the values read last are shown. If your target system has no chip description, it says "There is no chip description for this target system."

## Memory

Below the disassembly, the **Memory** shows the main memory of the computer. The heading names the start address ("Memory from $00FB"). Each line has the address, 16 bytes in hex and, on the right, the same bytes as characters.

![At the top the disassembler with addresses, bytes and instructions, below it the 'Speicher ab $0400' area with a hex dump: addresses 0400 to 04F0 on the left, 16 bytes per row in the middle, all 00, and the characters as dots on the right.](bild:debug-speicher "Example view: address, 16 bytes in hex and the same bytes as characters on the right.")

1. In the field **Address or symbol**, enter a hex number, for example `00FB`, or a name from your program (`ZAEHLER`), and press Enter (or click **Show**).
2. Run a step or press **Continue** to the next stop: Bytes that have changed *since the previous stop* are colored and bold. This way you see at a glance what your program has changed.

There is no button for scrolling and no right-click on the lines; just enter the next address. You see the content of *your program file* (not of the running computer) further below in the hex view, and the layout of your built program in the memory map.

## Watch

In the section **Watch**, you create **expressions** that VintageCoder calculates again at *every* stop. This way you do not have to look into memory all the time.

1. Type an expression in the field **Expression, address or symbol**, for example `a`, `x + 1`, `[$d020]` or `[ZAEHLER]`.
2. Press Enter or click **Add**.
3. Step by step (F11), you see how the value changes. It is shown in hexadecimal and in decimal, for example `$05 · 5`.

![Narrow side column of the debug view with the sections 'Haltepunkte', 'Beobachten', 'Momentaufnahme' and 'Chips'. Under 'Beobachten' there are three expressions, a, x and $d020, each with the value '$00 / 0' and a button to remove it. Below them are an input field and the button 'Hinzufuegen'.](bild:debug-beobachten "Example view, no emulator is running: the watch area with three expressions and their values in hexadecimal and decimal.")

You can also create expressions before the start of the session; they then apply at the next start. As long as there is nothing to show, "…" is there. These are allowed in expressions:

| What | Notation |
|---|---|
| Registers | `a`, `x`, `y`, `sp`, `pc`, `p` |
| Flags | `fn`, `fv`, `fb`, `fd`, `fi`, `fz`, `fc` |
| Memory | `[$d020]` or `peek($d020)`; a word with `peekw($fffe)` |
| Raster position | `lin` (line), `cyc` (cycle) |
| Names from your program | for example `[ZAEHLER]` (labels and constants of the last build) |

Operators work as in C (`==`, `!=`, `<`, `>`, `&&`, `||`, `&`, `|`). Comparisons bind more strongly than `&` and `|`; if in doubt, use parentheses, for example `(x & 15) == 5 && fc`. If an expression is invalid, the error message appears in its place. With the **x** next to the expression, you remove it.

## Snapshot

You have a program that puts something nice on the screen, and you want to take a character set or sprites from it into your project? The **Snapshot** (on the left in the Debug area) reads the current screen, the character set and the active sprites from the paused machine and stores them as files in the project (under `graphics/snapshot/`). This works for the C64 and the Atari 8-bit.

1. Pause the program at the place you want to capture (breakpoint or **Pause**). If the program is running, it says "A snapshot works only while the program is stopped."
2. Click **Detect what is in memory**. VintageCoder detects from the hardware registers where the screen and the character set are, and shows the mode, the screen address, the character set (address, in ROM or unknown, "multicolor" if applicable) and the number of sprites. Notes on the detection are below it.
3. Switch on what should be saved: **Include sprites** and **Build tiles from the screen**. If the character set is in ROM, there is also **Fetch charset from ROM**.
4. Enter a **Name of the snapshot** (only letters, digits, `_` and `-`) and click **Save as files**.

*[Bild folgt in E13: debug-momentaufnahme]*

Depending on your choice, this creates a screen (`.vcscreen`), a character set (`.vcchar`), sprites (`.vcsprite`) and a tile set (`_tiles.vctiles`). Bitmap modes and ANTIC modes other than mode 2 are not possible.

> **Info:** A preview and changing the detected addresses do not exist.

> **Caution:** The standard fonts of the computers belong to the manufacturers. Do not check a character set saved this way into a public repository.

## The image of the emulator

On the right below the stack, the section **Screen** shows the **image of the emulator directly in VintageCoder**. This works for VICE systems (C64, C128, Plus/4, VIC-20), not for Atari and MEGA65; there it says "This emulator does not provide a picture." The machine panel also shows a smaller single image.

*[Bild folgt in E13: live-bild]*

- **Refresh picture** gets the image immediately. As long as there is none yet, it says "No picture yet. "Refresh picture" fetches one."
- **Refresh at every stop**: With this switch, VintageCoder gets the image by itself at every stop (breakpoint, single step, pause). So you see exactly the state in which the machine stands.
- **Live (slow motion)**: VintageCoder stops the machine briefly every few milliseconds, gets the image and continues. The machine runs at about 40 to 80 percent of its speed and without sound. This is not meant for playing, but it is enough for watching changes.
- **Picture source**: If there are several sources (with the C128 **VIC-II** with 40 columns and **VDC** with 80 columns), you choose them in a selection list.
- **Send text to the emulator** writes text into the keyboard buffer of the emulator, for example `RUN`. Enter or **Send** sends it. Nothing but text is passed on: keyboard and joystick do not go to the emulator.

> **Info:** The user interface cannot save the image as PNG, copy it to the clipboard or set it as a project image.

## Examine a test in the debugger

If a test fails, you can examine it in the debugger: In the panel **Tests**, select the test and click **Debug this test** (see [Automatic testing](kapitel:tests)). VintageCoder prepares the test by itself (without building again) and opens a session that is paused at the start of the test. The toolbar then also shows **Test session**.

## The memory map

The **memory map** has nothing to do with the running program; it shows you the memory of *your built program*. You find it under **Data > Memory**. Important: It works with the result of the **last build**. Without a build, it says "Nothing built yet. The map comes from the last build. Build first (F7)."

![The Daten area with the Speicher page: the ranges (zero page, Stack, BASIC, ROM, E/A, frei) on the left, a matrix of 16 by 16 squares colored by type in the middle, and on the right the details of the selected page $08 with checks and the choice of processor port.](bild:memory-map "Example view: the memory map with 256 pages colored by type and the details of the selected page.")

- The **map** in the middle consists of 16 by 16 cells, each stands for a page of 256 bytes. To the left of each row is its start address (`$0000`, `$1000` to `$F000`). The colors show the purpose (for example zero page, stack, screen, sprites, character set, music, I/O, ROM, BASIC, free); a bar in the cell shows how much of it your program uses.
- **Filter** above the map: **All**, **Program**, **RAM**, **ROM**, **I/O**. Everything that does not match is faded.
- With the **arrow keys**, you choose a page; a click works too.
- On the left are the **Areas** (a click jumps to the first page of this kind) and the **Program**: how many bytes it uses and how much RAM is free.
- On the right, **Selected page** shows the address, the type of memory, how many bytes your program uses there, the symbols on the page and the source lines (a click jumps into the editor).
- Under **Checks**, there are notes with a sign for the severity (a check mark for OK, a dot for information, a warning sign for a warning, a cross for an error): overlaps, whether the stack is free, how much zero page your program uses.
- With the C64, C128 and Atari 800 XL/XE, the **Bank switching** at the bottom right switches the view (for example on the C64 "Standard (BASIC, I/O, KERNAL)" or "All RAM"). Above the list is the register with its address.

The reference describes everything else: [Memory map](hilfe:memmap).

## The hex view

The **hex view** (**Data > Hex**) shows you the content of a binary file of your project byte by byte, for example a built program. It is **read only**; it never changes a file.

![The area 'Daten' with the page 'Hex'. On the left is the list 'BINAERDATEIEN' with jumprun.prg selected. In the middle is a hex dump of the file with addresses, byte values and a text column, with a search field, '16 Byte', '8 Byte' and 'Nur lesen' above it. On the right is the card 'AUSWAHL' with file name, size, load address $0801 and page selector 1 / 5.](bild:hex-ansicht "The hex view shows a binary file byte by byte, read only.")

1. On the left under **Binary files**, choose a file. If there is none, it says "No binary files in the project." In the file tree, you can also open a file with the entry **View as hex (read only)**.
2. In the middle are the address (ADR), the bytes and, on the right, the text. A click on a byte selects it; with the Shift key held down or by dragging, you select several. The arrow keys, Page Up, Page Down, Home and End also work.
3. On the right, **Selection** shows the chosen range and **Interpreted as** shows the first byte (decimal, hexadecimal, binary), the word of two bytes (low byte first, as on the 6502) and the text. For program files, the **Load address** is also shown, and for the selection the memory address where the byte is after loading.
4. In the field **Search**, you enter bytes (`$A9 00`) or text; **Search** jumps to the next hit, or reports "Nothing found."
5. With **16 bytes** or **8 bytes**, you set how many bytes are in one row. The view reads large files page by page (4 KB per page) and at most the first 4 MiB; the buttons under **Page selection** page through.
6. **Open in the default program** opens the file in the program that your operating system provides for it.

## The live image (Ultimate 64)

> **Info:** A live image of the screen of an Ultimate 64 does not exist in this version. Running, reading memory and tests over the network are possible (see [Target systems compared](kapitel:systeme)).

## Common problems

| Problem | Cause and solution |
|---|---|
| Nothing happens after F6 | The build failed: Look in the **Problems** tab. Or a debug session is already running: end it with **Stop debugging** (Shift+F5). Otherwise F6 takes you back to the Debug area. |
| Registers, disassembly and memory are empty | The emulator is running. Press **Debug: Pause** (Alt+F6) or set a breakpoint. |
| The breakpoint is never hit | The line is not reached, or you changed the source text after the build: build again. With atari800, only one breakpoint at a time works. |
| The single step does not work | The emulator does not support the step in this situation; the note below the toolbar gives the reason. |
| An expression shows "unknown" | Check the spelling and the names. The error message is below it. Use parentheses with `&` and `|`. |
| **Step out** stops after ten seconds | There was no return. Set a breakpoint on the `RTS` of the subroutine. |
| The Screen section shows no picture | Your system does not provide one (Atari, MEGA65), or none has been fetched yet: **Refresh picture**. |

## What next?

- Check your program automatically instead of by hand: [Automatic testing](kapitel:tests).
- Everything about the special features of your target system: [Target systems compared](kapitel:systeme).
- The layout of your program and the hex view are described earlier in this chapter; you find more about the areas in [The user interface](kapitel:oberflaeche).
- All keys at a glance: [Keyboard shortcuts, FAQ, troubleshooting](kapitel:kuerzel-faq).
- For looking things up: [Debugger](hilfe:debugger), [Breakpoints](hilfe:breakpoints), [Memory map](hilfe:memmap).
