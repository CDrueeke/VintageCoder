---
id: grafik-ueberblick
titel: Graphics and sound overview
reihenfolge: 11
stufe: Beginner
zeit: 15 minutes
siehe_auch: [zeichensatz, sprites, bildschirm-kacheln, paletten, tracker]
referenz: [graphics, export]
i18n_status: maschinell
i18n_quelle: 918fbe4ba87f
---

# Graphics and sound overview

**Goal:** You know which drawing tools (the "editors") VintageCoder has, how you open and create a graphics file and how its content gets from the editor into your program. With this chapter, you understand the way that all following chapters (12 to 17) each go into in detail.

## What you need

- an open project (see [Your first project](kapitel:erstes-projekt)),
- for the last example, the emulator of the target system.

## What are assets?

Everything that is not program text, we call **assets**: a sprite, a character set, a screen, a palette, a song. Each asset is a **small text file** in your project, for example `graphics/hero.vcsprite`. The content is hex lines that describe pixels. This has two advantages: You can compare the files in a meaningful way with version control such as Git, and the editor and the build read the same source.

| Extension | Type | Editor | Chapter |
|---|---|---|---|
| `.vcchar` | Character set | Character set editor | [12](kapitel:zeichensatz) |
| `.vcsprite` | Sprites (C64, MEGA65) | Sprite editor | [13](kapitel:sprites) |
| `.vcpmg` | Player/missiles (Atari) | PMG editor | [13](kapitel:sprites) |
| `.vcscreen` | Text screen or screen bank | Screen editor | [14](kapitel:bildschirm-kacheln) |
| `.vctiles` | Tile set | Tile editor | [14](kapitel:bildschirm-kacheln) |
| `.vcpal` | Palette (MEGA65) | Palette editor | [15](kapitel:paletten) |
| `.vcsong` | Song (SID, POKEY and others) | Tracker (main section **Music**) | [16](kapitel:tracker) |
| `.vcplay` | PLAY song (MEGA65) | PLAY tracker (main section **Music**) | [17](kapitel:play-tracker) |

> **Info:** Each asset belongs to a **profile**. The profile defines how the asset is built, for example `c64.sprite.multicolor` (C64 sprite, multicolor, 12x21 dots) or `atari.char.antic2` (Atari character set with 128 characters). You choose the profile when you create the asset.

Built are the sprite editor (with PMG), the character set editor, the palette editor, the tile editor, the screen editor, the map editor and the tracker (with the PLAY tracker).

## Step by step: create and open an asset

1. In the header, choose the main section **Graphics**. In the second row are **Sprites**, **Character set**, **Tiles**, **Screen** and **Palette**. You edit songs in the main section **Music**.
2. If no file is open yet, the editor shows the matching files of the project as buttons; a click opens the file. With **Open ...** you choose another file from the project. A **double-click** on the file in the project tree opens it the same way in the matching editor.
3. You create a new file in the editor itself: in the sprite editor with **New sprite file ...**, in the character set editor with **New charset ...**. In the dialog, you choose the **profile** (the matching variant) under **Type and system**, enter the **Name** (letters, digits, `_` and `-`), check the **Folder in the project** (default `graphics`) and click **Create**. The file opens right away. The dialog **New file** (sidebar or **File > New file ...**), in contrast, creates only source texts and notes. There is no separate create dialog for tile sets, screens, palettes and songs; you open them from the project, and they are already in the example projects, for example.

![The Grafik tab with the Sprites page and the file hero.vcsprite: on the left the sprite files and two frames, in the middle the 12x21 pixel multicolor sprite of a figure in yellow, brown-red and blue, on the right the cards for colors, sprite (multicolor on), preview in the game and move. Below the drawing area a note says there is no animation.](bild:sprite-c64 "The sprite editor with a multicolor sprite.")

## The common layout of the editors

The sprite, character set, palette, tile and screen editors look similar on purpose. If you know one, you find your way in the others. They are in the main section **Graphics** and always show one file. If it is changed, the footer shows "● unsaved".

1. **On the left** the list: in the sprite editor the sprite files and the pictures (frames), in the character set editor the character overview, in the palette editor the color grid, in the tile editor the tile set, in the screen editor the character table.
2. **In the middle** the drawing area with the **toolbar** above it: drawing tools, undo and redo, flip, **Grid**, zoom and, in the sprite editor, **Onion Skin**. On the right in the bar are the file buttons **Export ...** and **Save** (in the sprite and character set editors also **Load PNG ...**). The tile and screen editors arrange their tools in their own way; see [chapter 14](kapitel:bildschirm-kacheln).
3. **On the right** the cards with the pixel values, colors and properties.
4. **Footer:** name, dimensions, cell position and messages.

In the sprite and character set editors, the drawing area adjusts its zoom to the free space automatically. If you want to decide yourself, use **-** and **+** in the toolbar or **Cmd/Ctrl + mouse wheel**. The keys **+** and **-** work the same way, **0** fits the area again (button **Fit**).

Useful basic rules for the editors:

- **Cmd/Ctrl+S** saves, **Cmd/Ctrl+Z** undoes, **Cmd/Ctrl+Shift+Z** redoes.
- With the **right mouse button**, you erase on the drawing area (sprite and character set editors).
- The colors depend on the target system. On the Atari, you also choose the **Palette view** (Project, PAL or NTSC) on the right.
- The single keys of the tools (in the sprite and character set editors D, E, I, F, L, R, S, X, Y and the digits for the pixel value; in the screen editor D, T, F, R, I, E) apply when the drawing area has the focus. The tables are in the chapters on the [Sprite editor](kapitel:sprites) and the [Character set editor](kapitel:zeichensatz) and in [Keyboard shortcuts, FAQ, troubleshooting](kapitel:kuerzel-faq).

## From the editor into the program: the export

Your program cannot read the text file, it needs **bytes**. The **export** converts an asset, for example into an assembler file with a label and `.byte` lines. You do not need the export to include an asset: the assembler converts the file itself at the `.include`.

There are two ways:

- **Automatic:** You include the asset's file in the source text, for example `.include "graphics/hero.vcsprite"`. When you **build** (F7), VintageCoder converts it by itself; a cache (`build/.media-cache/`) keeps the result up to date.
- **By hand:** The button **Export ...** in the toolbar of the editor (or **Cmd/Ctrl+Shift+E**, also in the **File** menu) opens the export dialog.

### The export dialog

You choose the **Format** at the top:

| Format | File | Used for |
|---|---|---|
| Assembler | `<name>.asm` | data as `.byte` lines with labels and constants; the normal case |
| Constants | `<name>.inc` | only the sizes (`<name>_len`) and optionally the address |
| Binary | `<name>.bin` | raw data, for example for loading later |
| PRG | `<name>.prg` | raw data with load address (you give it in the field **Load address**) |
| BASIC DATA | `<name>.data.bas` | `DATA` lines for BASIC programs, if you want with **Include a read routine** |

In the field **Name** is the label that serves as the prefix of all names in the export. The **Preview** shows the current state, also unsaved changes, without creating anything in the project (tabs **Text** and **Hex**). Below it, you see how many files and bytes are created, and under **Target in the source** you choose the open source text that the output should be inserted into. At the bottom are the actions:

- **Test in emulator:** saves, exports, builds your project and runs it like F5.
- **Save in project:** writes the files to `build/export/`.
- **Insert as include** (only for Assembler and BASIC DATA): saves, exports and puts an `.include` line into the chosen source text.
- **Copy into source** (only for Assembler and BASIC DATA): writes the generated text itself at the cursor line of the chosen source text, without an include.

![The 'Exportieren' (export) dialog over the sprite editor for graphics/hero.vcsprite. At the top are the formats Assembler, Konstanten, Binaer, PRG and BASIC DATA (Assembler selected), below that the name field with 'hero' and the generated assembler text. At the bottom are the ready files, a yellow note under 'Ziel im Quelltext' and the buttons 'Abbrechen', 'Im Emulator testen', 'In Projekt speichern', 'Als Include einfuegen' and 'In Quelltext kopieren'.](bild:grafik-exportieren "The export dialog with format, name, generated text and target choice.")

> **Tip:** If you only need the data in the program, let the automatic export do the work. The dialog is worth using if you need a different format or want to look at the bytes once.

## Example to follow: a sprite on the screen

The project `examples/c64/sprite_demo` contains a multicolor sprite `graphics/hero.vcsprite` and this program:

```asm
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

start:  sei
        lda #$80                ; sprite pointer 0: data at $2000 ($2000 / 64 = $80)
        sta $07f8
        lda #1
        sta vic2.spena          ; sprite 0 on
        sta vic2.spmc           ; multicolor
        lda #100
        sta vic2.sp0x
        sta vic2.sp0y
        lda #7
        sta vic2.sp0col
        lda #2
        sta vic2.spmc0
        lda #6
        sta vic2.spmc1
loop:   jmp loop

.org $2000
.include "graphics/hero.vcsprite"
```

1. Open the project `examples/c64/sprite_demo` (on the start page, under the examples).
2. Press **F7** (Build project). VintageCoder converts `graphics/hero.vcsprite` automatically at the `.include`.
3. Press **F5** (Run start program). In the emulator, the little hero appears.

The last two lines are the whole "installation": `.org $2000` sets the address, `.include` brings the exported bytes there. You learn how the sprite is made in [chapter 13](kapitel:sprites).

## Common stumbling blocks

- **"Label not found" when building:** The name after the export (`hero`, `font`, ...) depends on the name in the asset, not on the file name. In the export dialog, you see the name in the field **Name**.
- **Changes do not appear in the program:** Then it was not exported again after your change. Build with F7 or use **Export ...**. Save first with Cmd/Ctrl+S.
- **An editor shows "No ... open":** It needs an open project and a file of the matching type. Choose a file from the list or open it with a double-click in the project tree.

## What next?

Next you get to know the simplest editor: the [Character set editor](kapitel:zeichensatz). Detailed reference texts: [Editors overview](hilfe:graphics) and [Export](hilfe:export).
