---
id: zeichensatz
titel: Character set editor
reihenfolge: 12
stufe: Beginner
zeit: 25 minutes
siehe_auch: [grafik-ueberblick, bildschirm-kacheln, sprites]
referenz: [charset, graphics, roms, export]
i18n_status: maschinell
i18n_quelle: f85b27eaf0d5
---

# Character set editor

**Goal:** You draw your own letters and graphic characters, import and export character sets and show them in a program.

## What you need

- a project for C64, Atari, MEGA65 or another system with a text screen,
- preferably the **ROM files** of your system (see [Welcome to VintageCoder](kapitel:willkommen)), because a new character set starts with the font of your computer. Without a ROM, VintageCoder uses its **own default set** ("VC Standard", free and can be passed on, see below).

## What is the editor for?

In the character set editor you draw the **letters and graphic characters** of your program. Because a text screen consists only of characters, your own character set is the simplest way to get your own look, without sprites or bitmap graphics.

Typical uses from real retro projects:

- **Your own font** for a menu or a text adventure (narrow, italic, in the style of the game).
- **Game graphics made of characters:** walls, ladders, coins and backgrounds of a platform game are often redefined characters. This is fast and saves memory.
- **A title picture made of text:** a large logo, put together from 2x2 or 3x3 characters.
- **Translated or extended characters:** umlauts for a computer that does not know them.

In the finished program, the character set ends up as **2048 bytes** (C64: 256 characters times 8 bytes) at an address in memory; your program tells the graphics chip where to fetch the character shapes.

## Basic terms without previous knowledge

- **Character:** a pixel pattern of 8x8 dots that the chip draws for one place on the screen.
- **Screen code:** the number of the character (0 to 255). It is **not** the ASCII or PETSCII code.
- **Character set:** the table of all character shapes. It is normally in the ROM of the computer; you can replace it with your own in RAM.
- **ROM:** read-only memory of the computer. The original ROM files do not belong in your project.
- **Hires / Multicolor / Full color:** kinds of characters, see below.
- **Profile:** says which kind of character and which system a file has (for example `c64.char.hires`).
- **Template:** the set with which a new character set starts: your ROM or the own default set "VC Standard".
- **VC Standard:** the own character sets of VintageCoder, free under CC0 and can be passed on.

## What is a character set?

A text screen does not know pixels, but **characters**: each cell of the screen contains a number (the **screen code**), and the graphics chip looks up in a table what this character looks like. This table is the **character set**. Each character consists of **8x8 dots**, that is eight bytes. The C64 has 256 characters (2048 bytes), the Atari in the usual text mode 128.

Because games like to **redefine** the character set, this is a powerful trick: you draw your own tiles (walls, ladders, coins) as "characters" and build whole pictures from text with them.

> **Caution:** The order in the character set is that of the **screen codes**, not of ASCII or PETSCII. On the C64, `@` is character 0, `A` is character 1, and so on. On the Atari, the space is 0, the digits start at 16, the capital letters at 33.

There are two kinds of characters:

- **Hires** (high resolution): 8x8 dots, each dot is on or off. The color comes from the color memory.
- **Multicolor**: the dots are twice as wide, so a character has 4x8 dots, but each dot may have one of four colors. The resolution is halved, but you gain color.

Which kind a character set is, is decided by its **profile** (for example `c64.char.hires` or `c64.char.multicolor`). On the MEGA65 there is also the **full color mode** (`mega65.char.fcm`): 8x8 dots, each dot one of 256 palette colors.

## My first character set (using the example `gfx_demo`)

1. **Open the example:** On the start page, open `examples/c64/gfx_demo` from the example projects (to change it, use **Copy**, then you have your own version). *Expected:* The project is open; in the file tree there is `graphics/font.vcchar`.
2. **Open the character set:** At the top, choose the main section **Graphics** and in the second row **Character set**. If nothing is open yet, the editor shows the **Character set files** of the project as buttons; click `graphics/font.vcchar`. A double-click on the file in the file tree works the same way. *Expected:* On the left the **Character overview** with all 256 characters, in the middle the chosen character, greatly enlarged, on the right the preview and the cards.
3. **Choose a character:** click in the overview. *Expected:* Above the canvas you see the number, for example "Character 1 · $01", and the matching character next to it. With **Page Up** and **Page Down** (or **Alt+Left/Right arrow**) you go to the previous and next character.
4. **Draw:** choose the **Pen** (key **D**). The left mouse button sets a dot, the right one erases it. *Expected:* The small tile in the overview and the **Preview in the picture** show your character right away, and **●** ("unsaved") appears above the name.
5. **Undo:** **Cmd/Ctrl+Z**; **Cmd/Ctrl+Shift+Z** brings the step back.
6. **Save:** **Cmd/Ctrl+S**. *Expected:* The **●** disappears.
7. **Build and run:** **F7**, **F5**. *Expected:* The changed character appears in the emulator on the title picture.

You create a completely new character set with **New charset …** (button in the toolbar, also on the empty page of the editor). In the dialog **New charset**, choose the profile under **Type and system** (to start, the C64 character set hires 8x8), enter the **Name** (letters, digits, `_` and `-`), check the **Folder in the project** (default `graphics`) and click **Create**. The file opens in the editor right away.

## Layout of the user interface

The character set editor has three areas side by side. In narrow windows, the right area moves below the other two.

| Area | Content |
|---|---|
| Left: **Character overview** | all characters as tiles, 16 per row; below it **Ranges** (All, Letters, Digits, Punctuation, Custom) and, if the project has several character sets, the **Character set files** |
| Middle | toolbar, number of the character, the canvas with the **Neighbors** (the character before and after), footer with name, character and unsaved changes |
| Right | **Preview in the picture**, **Template** with **Origin**, **Character** (number and position), **Colors** and **Shift** |

The toolbar contains, from left to right: **Pen** (D), **Eraser** (E), **Eyedropper** (I), **Fill** (F), **Line** (L), **Rectangle** (R), **Selection** (S), Undo and Redo, **Flip horizontally** (X) and **Flip vertically** (Y), then **Grid** (G), **Zoom out** (-), the zoom display, **Zoom in** (+), **Fit** (0) and to the right of that the file buttons **New charset …**, **Load PNG …**, **Export …** and **Save**. When you point at the buttons, they show their key.

![The character set editor in the area 'Grafik' with the file font.vcchar. On the left is the character matrix 'FONT' with 256 characters and the ranges 'Alle', 'Buchstaben', 'Ziffern', 'Satzzeichen' and 'Frei'. In the middle is the large drawing area with a toolbar. On the right are the cards 'VORSCHAU IM BILD', 'VORLAGE', 'ZEICHEN', 'FARBEN' and 'VERSCHIEBEN'.](bild:zeichensatz-ueberblick "The character set editor with the character matrix, drawing area and the cards on the right.")

The single keys apply when the canvas has the focus:

| Key | Effect |
|---|---|
| D, E, I, F, L, R, S | Pen, Eraser, Eyedropper, Fill, Line, Rectangle, Selection |
| X, Y | flip horizontally, vertically |
| 1 to 4 | choose pixel value (digits up to 9, as far as the set has that many values) |
| G | grid on or off |
| + / - / 0 | zoom in / zoom out / fit (also **Cmd/Ctrl** and mouse wheel) |
| Arrow keys | move the cell marker; **Enter** or **Space** sets a dot, **Delete** removes it |
| Page Up / Page Down, Alt+Left/Right arrow | previous / next character |
| Cmd/Ctrl+Z, Cmd/Ctrl+Shift+Z | Undo, Redo |
| Cmd/Ctrl+S | Save |
| Cmd/Ctrl+Shift+E | Export dialog |

With the **Selection** tool you drag a frame; **Cmd/Ctrl+C**, **X** and **V** copy, cut and paste, **Delete** deletes the selection, **Esc** clears it. A right-click with the selection tool opens a menu with the same actions.

To shift the whole character, there are four arrow buttons on the right under **Shift** (left, right, up, down). **Clear character** deletes all dots of the chosen character.

### Ranges and preview

The **Ranges** jump to the first character of a group and mark the group in the overview. With **Custom** you enter **from** and **to** as character numbers. This way you quickly find all digits, for example.

The **Preview in the picture** shows a text in the characters of your set, as it would look on the screen. You change the text in the field below it (**Text for the preview**). The preview follows every change right away.

### The template and the origin

A new character set does **not** start empty: it contains a reference to the font of your system, as it appears after power-on (on the C64 "Uppercase and graphics"). The file contains only **your changes**. The original characters are protected by copyright, so they are not in your project, but stay in the ROM. If you have no ROM, the own set of VintageCoder is behind it.

On the right under **Template**, you choose another base font in the list **Template to load** (on the C64, for example, the one with lowercase and uppercase letters) and click **Load template**. An entry whose ROM is not found is gray and has the addition "ROM not found". Next to the fonts of your ROMs, there is always also **Standard (own)**: the supplied set of VintageCoder, as a **copy** in your file. Loading replaces all characters; **Cmd/Ctrl+Z** undoes it. If your character set should start from zero, create it on the command line with `vcc media new ... --blank`.

Below it, the card **Origin** says in words where the characters come from, with a label: **OWN**, **OWN, CHANGED**, **FROM YOUR ROM** or **ROM MISSING**. A set that you designed yourself has the note "Designed by you, no template loaded". If a template is loaded, the editor also shows how many characters differ from it ("Changed from the template" or "Same as the template" for the chosen character). Changed characters have a small dot in the overview. The button **Reset character to template** brings back the original of the chosen character.

*[Bild folgt in E13: zeichensatz-standard]*

### Drawing multicolor

With a multicolor profile, each dot gets one of four **pixel values**. The card **Colors** lists each value with its color; which register sets which color is shown next to the respective value (on the C64 background `$D021`, plus `$D022` and `$D023`; the fourth color comes from the **Color RAM `$D800`**). You choose the value with a click or with the key 1 to 4. Below that, for the chosen value you choose the color from the color grid of the system. Three of the colors (background and two more) apply to the whole screen, the fourth comes from the color memory of the respective screen cell. On systems with PAL and NTSC colors (Atari), you switch the **Palette view** between **Project**, **PAL** and **NTSC**.

*[Bild folgt in E13: zeichensatz-multicolor]*

### MEGA65: full color characters

With the profile `mega65.char.fcm`, each dot is one of 256 palette colors. Which colors these are is decided by the palette, see [Palettes](kapitel:paletten).

## Own default character sets (VC Standard)

VintageCoder comes with **own character sets** for every system (C64, C128 with VDC, VIC-20, Plus/4, Atari with standard and international, MEGA65). They are **not a copy** of the original ROMs, but own designs of the character shapes, free under the license CC0 (public domain). You may use and pass them on in your own programs and projects.

- **New project:** In the section **Character set of the project**, you choose **Automatic** (your ROM as a reference, otherwise the own set), **Standard only (own)** (always the own set, with `HERKUNFT.txt` under `graphics/vc-standard/`: the project can be passed on) or **None** (you then create a character set later in the character set editor). On the command line: `vcc new ... --charset auto|own|none`.
- **Character set editor:** The template **Standard (own)** copies the set into your file. The card **Origin** says where the characters come from.
- **Order of precedence:** your ROM, then the own set, then nothing. If the ROM that a reference points to is missing on another computer, VintageCoder switches to the own set and tells you.

> **Note:** The origin of each set is in the file (`origin`). This is a note on how it came about.

> **Info:** VintageCoder does not ship original ROMs and does not download any from the network. Where you put your own images is described in the chapter [Disk editor, segments, ROMs](kapitel:disk-studio).

The example project `examples/c64/standard_charsets` contains the **free** Open ROMs character sets (`openroms_c64_gross.vcchar`, `openroms_c64_klein.vcchar`). You may pass them on.

## Character sets from other sources

In this version, you get a character set this way:

- **New charset …** with the template of your ROM or **Standard (own)** (see above).
- **Load PNG …** (toolbar): you choose an image file; the editor converts it into the pixel values of the chosen character and saves the file while doing so. This way you can draw a single character in an image program.

The commands **Add standard character set ...**, **Import character set from ROM ...** and **Import character set from PNG grid ...** (menu **File**) and the export as a PNG grid are not available in this version yet.

## Export a character set

**Export …** (toolbar, also **Cmd/Ctrl+Shift+E**) opens the export dialog. There you choose the **Format**: **Assembler**, **Constants**, **Binary**, **PRG** or **BASIC DATA**. In addition there are the **Name** and, depending on the format, an **Address** (for PRG the **Load address**). With **BASIC DATA** you can tick **Include a read routine**. The **Preview** shows the text and, except for BASIC DATA, the bytes as **Hex**; below it you see how many files and bytes are created.

At the bottom you choose what happens to the result:

- **Save in project** writes the files into the folder `build/export`.
- **Insert as include** puts the `.include` line into the source you chose under **Target in the source** (Assembler and BASIC DATA).
- **Copy into source** writes the text itself at the cursor line of that source, without include.
- **Test in emulator** saves the export and builds and starts the project.

You do not need the export for including: when you build (**F7**), VintageCoder converts the character set itself at the `.include` (see the program below). Inserting into the source only works if a matching source is open.

## Show the character set in the program

The example `examples/c64/gfx_demo` shows the whole way. This project contains a character set `graphics/font.vcchar` (all 256 characters drawn by yourself) and a screen `graphics/title.vcscreen` that uses these characters. The following program loads both:

```asm
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

.include "lib/gfx.inc"

start:  sei
        gfx_load_charset font, $2800        ; copy our character set to $2800
        gfx_charset_at $2800                ; from now on the VIC reads the characters from there
        gfx_screen_at gfx_screen
        gfx_load_screen title_chars, title_attrs
        gfx_colors title_border, title_bg
done:   jmp done

gfx_runtime
.include "graphics/font.vcchar"
.include "graphics/title.vcscreen"
```

What happens here?

- `.include "lib/gfx.inc"` brings small **macros** (reusable sequences of instructions): `gfx_load_charset` copies the 2048 bytes of the character set to an address that must be a multiple of 2 KB (`$2800` fits). `gfx_charset_at` tells the graphics chip VIC-II where to find the character shapes.
- `gfx_load_screen` writes the 1000 character codes into the screen memory and the colors into the color RAM. `gfx_colors` sets the border and background color from the export.
- `gfx_runtime` stores the copy routine. It comes **after** the main program, so that it is not run by accident.
- The two `.include` lines fetch the export files that are created automatically when you build (F7).

1. Open `examples/c64/gfx_demo`, replace the content of `gfx_demo.asm` with the program above (or create a copy).
2. Build with **F7**, run with **F5**.
3. Open `graphics/font.vcchar`, change a character (for example the A), build and run. The new character is immediately visible on the title picture.

There are counterparts for Atari and BASIC: `examples/atari/gfx_demo/lib/gfx.inc`; in BASIC you include the `DATA` lines of the export (format **BASIC DATA**) with `#include`, see [BASIC in the IDE](kapitel:basic).

## Hardware features and limits

- **C64, C128 (40 columns), VIC-20, Plus/4:** 256 characters of 8x8 dots (VIC-20 depending on the set); the set must start on a 2 KB boundary in memory and lie in the visible range of the graphics chip (on the C64 not at `$1000`-`$1FFF` of the first bank, where the VIC sees the ROM).
- **Atari:** 128 characters (the hardware does the inverse display); the set starts on a 1 KB boundary. Standard and international are two sets.
- **C128 80 columns (VDC):** 256 characters of 8x16 dots.
- **MEGA65:** in addition full color characters (256 palette colors per dot).
- **Multicolor:** per character 4x8 wide dots with four values; three colors apply to the whole screen, one comes from the color memory of the cell.
- If you have no ROM files of your own on a computer, the editor automatically shows the own set (label **OWN**).

## Neighboring editors and glossary

- From characters you build pictures in the [Screen and tile editor](kapitel:bildschirm-kacheln).
- You draw moving figures in the [Sprite and PMG editor](kapitel:sprites); colors are in the chapter [Palettes](kapitel:paletten).
- Terms: Glossary (file `GLOSSAR.md`): character set, screen code, ROM, Hires, Multicolor.

## Common stumbling blocks

- **The normal font appears instead of my characters:** You forgot to call `gfx_charset_at`, or the target address is not in the range of the VIC (in the first 16 KB bank, `$1000`-`$1FFF` is the character ROM and therefore not available).
- **The font looks "wrong":** Probably the editor shows the own set of VintageCoder because your ROM is missing. The editor says so in the card **Origin** (label **ROM MISSING**). Put the ROM in place and check again.
- **Letters are not right:** The order is screen code, not ASCII (see the caution above).
- **The arrow keys do not change the character:** On the canvas they move the cell marker. You get to the next character with **Page Up/Down**, **Alt+Left/Right arrow** or by clicking in the overview.

## What next?

Next you draw moving figures: [Sprite and PMG editor](kapitel:sprites). Later you put the characters together into pictures in the [Screen editor](kapitel:bildschirm-kacheln). For looking things up: [Character set editor](hilfe:charset), [Standard character sets and ROMs](hilfe:roms).
