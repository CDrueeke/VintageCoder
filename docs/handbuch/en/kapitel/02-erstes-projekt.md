---
id: erstes-projekt
titel: Your first project
reihenfolge: 2
stufe: Beginner
zeit: 15 minutes
siehe_auch: [oberflaeche, assembler, bauen-starten]
referenz: [wizard, start, build]
i18n_status: maschinell
i18n_quelle: 7fe2f0a3578c
---

# Your first project

**Goal:** You use the dialog **Create new project** to create a project for the C64, look at the finished program, build it and run it in the emulator. Then you change one number and see the effect.

## What you need

- VintageCoder, started (see [Installation and first-run setup](kapitel:installation)),
- the emulator **VICE** for the C64 (`x64sc`), which VintageCoder shows as "ready" on the start page in the box **Toolchain and hardware** (otherwise you set it up with the setup assistant or under **Settings > Targets and hardware**),
- a folder where the new project will be created, for example a folder "Retro" in your documents.

You do **not** need any previous knowledge of the C64 or of assembler.

## Step by step: Create a project

1. On the start page, click **New project**. You can also open the dialog with the menu **File > New project ...**, with the command palette (**Cmd/Ctrl+K**) or with **Cmd/Ctrl+Alt+N**. The dialog is called **Create new project**.
2. The dialog has three columns. In the first column you choose the **target system**. Click **Commodore 64**. Below it you see the **hardware profile** with the display "valid", the 6510 processor and the chips of the C64.

![The dialog 'Neues Projekt erstellen' above the start page. On the left is the list 'ZIELSYSTEM' from Commodore 64 (selected) to MEGA65 with the hardware profile 'gueltig'. In the middle is the 'VORLAGE' with 'Assembler-Programm' (selected), 'Steckmodul' and 'Leeres Projekt'. On the right are the 'PROJEKT-DETAILS' with project name, location, video standard, default emulator, start target, disk and character set. At the bottom are the buttons 'Abbrechen' and 'Projekt erstellen und oeffnen'.](bild:neues-projekt-dialog "The dialog 'Neues Projekt erstellen' with target system, template and project details.")

3. In the second column you choose the **template**. At the top right of the column you choose the language: **Assembler** or **BASIC**. For this chapter, choose **Assembler** and then the template **Assembler program** ("Small assembler program (color change), can be run with F5"). The card shows which file the template creates: `src/c64/main.asm`. The other templates are called **Cartridge** and **Empty project**.

4. The third column contains the **project details**:
   - **Project name:** for example `hallo`. Letters, digits, `_` and `-` are allowed. If you type other characters, the note turns red.
   - **Location:** the folder in which VintageCoder creates a subfolder with the project name. Use **Choose ...** to pick it.
   - **Video standard:** **PAL** (Europe) or **NTSC**. Leave the default.
   - **Default emulator** and **Start target (F5)**: an emulator is preselected here. Leave them as they are.
   - **Disk:** for your first project, **none**.
   - You can leave the two switches for **README.md** and **Git repository** switched off.
   - The block **File structure** shows in advance what will be created.

5. At the bottom, click **Create and open project**. The button is active only when the name and the template are valid. If a red message appears instead, it names the reason; read it and correct the entry.

The project is created and opens in the section **Code**. You see the result in the editor: the file `main.asm` is open, and in the **sidebar** (heading **Project**) you see the folder `src/c64/`.

*[Bild folgt in E13: projekt-neu-ergebnis]*

> **Tip:** On the command line you do the same with `vcc new hallo --target c64 --template asm-hello`. You can find more about the dialog in the help page [Start page, setup assistant and new project](hilfe:wizard).

## Read the program

The template creates this small program (the project name is in the first line):

```asm
; hallo - C64: border color follows the raster line
.target c64
.cpu 6510 { illegals = deny }
.org $0801
.basicstub start

start:  sei
.frame: lda #$80                ; wait for raster line $80
.wait:  cmp $d012
        bne .wait
        inc $d020               ; advance border color
        jmp .frame
```

You do not have to understand this yet; the assembler chapter explains everything. Here is only the meaning of the lines:

- Lines with `;` are **comments** for people. The assembler skips them.
- `.target c64` says: This program is for the C64.
- `.basicstub start` makes sure that you can start the program on the C64 with `RUN`, like any other program.
- `start:` and `.frame:` are **labels**: names for a place in the program.
- `inc $d020` counts the border color up. The program always waits for a certain screen line and then advances the color, so the border flickers in stripes.

## Step by step: Build and run

1. In the header bar, click the icon **Build** or press **F7**. VintageCoder saves the file and translates it. Afterwards the status bar at the bottom says "built: 28 bytes". A click on it opens the **tray** below the editor with the tab **Output**. There a line "Done: 28 bytes in ..." shows that the program was built.
2. Click the icon **Run** or press **F5**. If needed, VintageCoder builds once more and starts VICE with your program. After a short moment you see the emulator window: the border of the C64 screen runs through the colors in colorful stripes.

*[Bild folgt in E13: bauen-knopf]*

3. Close the emulator window when you are done, or click **Stop** in the header bar (the icon appears as long as the emulator runs). End the emulator before you run again.

> **Info:** **F7** only builds, **F5** builds and runs. The run target (for example "C64 · VICE x64sc") is to the left of the icons in the header bar and shows where F5 runs: here the emulator VICE x64sc for the C64. The **Machine** panel on the right also shows the state and the result of the last build (Cmd/Ctrl+Alt+B shows and hides it).

## Step by step: Change a number

Now you turn it into your own first program: the border should get **one** fixed color.

1. In the editor, select the whole text (Cmd/Ctrl+A) and replace it with:

```asm
; Color the border red and return to BASIC
.target c64
.basicstub start

start:  lda #2              ; 2 = color red
        sta $d020           ; set the border color
        rts                 ; return to BASIC
```

2. Press **F5**. In the emulator, the C64 start screen appears with a **red** border. Type `RUN` and Enter in the emulator if the program has not already run: the border turns red, and the C64 reports "READY.".
3. Change the `2` to another number between 0 and 15 (for example `5` for green or `6` for blue), press **F5** again and look at the effect.

> **Info:** The C64 has 16 colors, numbered from 0 (black) to 15 (light gray): 0 black, 1 white, 2 red, 3 cyan, 4 purple, 5 green, 6 blue, 7 yellow, 8 orange, 9 brown, 10 light red, 11 dark gray, 12 medium gray, 13 light green, 14 light blue, 15 light gray.

You can find out how the individual lines work in [Assembler (VCASM) for beginners](kapitel:assembler).

## Step by step: The same in BASIC

You can also start the same project in BASIC. In the dialog, in column 2, choose the language **BASIC** instead of "Assembler", and the template **BASIC**. The project then contains `src/c64/main.bas`:

```basic
REM hallo - native BASIC V2 (C64), without line numbers
PRINT CHR$(147);"HELLO FROM hallo"
T=0
FOR I=1 TO 10
T=T+I
NEXT I
PRINT "SUM=";T
@ende:
GOTO @ende
```

F5 builds and runs this program too. You can find more about it in the chapter [BASIC in the IDE](kapitel:basic).

## Common stumbling blocks

- **"Create and open project" is gray.** The project name contains a character that is not allowed (allowed: letters, digits, `_`, `-`), or there is no template for the combination of target system and language.
- **F5 starts nothing.** The header bar shows the run target. If the emulator is "not found" on the start page in the box **Toolchain and hardware**, install it (see [Installation and first-run setup](kapitel:installation)) or enter its path under **Settings > Targets and hardware**. The reason for a failed run is given by the tab **Output** in the tray.
- **The border does not turn red.** The program must run on the C64: it starts by itself after loading; if the screen stays at "READY.", type `RUN`.
- **Two emulator windows.** Close the old window before you run again.
- **The border flickers too wildly.** That is the template program: it advances the color in every frame. Replace it with the red border program as shown above.

## What next?

Now you know the way from the empty project to the running program. Next you learn about the windows: [The user interface at a glance](kapitel:oberflaeche). After that come [Projects and files](kapitel:projekte) and the [code editor](kapitel:editor).
