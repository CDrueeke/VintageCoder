---
id: paletten
titel: Palettes
reihenfolge: 15
stufe: Beginner
zeit: 15 minutes
siehe_auch: [zeichensatz, sprites]
referenz: [mega65, export, graphics]
i18n_status: maschinell
i18n_quelle: bc5a16fa5b8e
---

# Palettes

**Goal:** You understand what a palette is, edit the 256 colors of the MEGA65 in the palette editor and load them in a program. You also know how colors come about on the C64, Atari and the other systems.

## What you need

- for the palette editor, a **MEGA65 project** (only this system has a freely changeable palette),
- knowledge from [Graphics and sound at a glance](kapitel:grafik-ueberblick).

## What is the editor for?

The palette editor decides **which colors your graphics really have**. On the MEGA65 these are 256 colors that you mix yourself; on the other systems the colors are fixed, and you choose them in the other editors.

Typical uses from real retro projects:

- **Your own color gradient** for a sky or water (full color characters and bitmaps of the MEGA65).
- **A color scheme** for a whole game, so that all figures match.
- **Color change effects:** the same graphics with another palette show day and night.

In the finished program, these are three tables of 256 bytes each (red, green, blue) that your program copies into the palette registers of the chip.

## Basic terms without previous knowledge

- **Palette:** table "number -> color".
- **Entry:** one row of the table (a number with its color).
- **RGB:** a color made of red, green and blue; on the MEGA65 8 bits each (0 to 255).
- **Hex value:** the color as `#rrggbb`, for example `#ff8800` for orange.
- **Palette bank:** the MEGA65 has four palettes that you can switch between.
- **PAL / NTSC:** European and American video standard; Atari colors look different depending on the standard.
- **VIC-IV:** the graphics chip of the MEGA65.

## What is a palette?

A dot does not store a color, but a **number**. The **palette** is the table that assigns a color to each number. If you change the table entry, the color changes everywhere this number is used.

- **C64, VIC-20, Plus/4:** The palette is hard-wired (on the C64 16 colors). You only choose numbers. The color tones you see in the editors come from files under `data/palettes`.
- **Atari:** The chip knows 128 colors (16 hues times 8 brightness levels). They are fixed as well. Whether the colors look like PAL or NTSC, you set in every Atari editor on the right at the **Palette view**.
- **MEGA65:** The graphics chip VIC-IV has a palette with **256 entries**, each made of red, green and blue with **8 bits** each (so 16.7 million possible colors). You may change these entries yourself. This becomes important in full color mode: there each dot of a character points to an entry of the palette (see [Character set editor](kapitel:zeichensatz)).

## My first palette (MEGA65)

In this version you do not create a new palette file in the user interface. To try it out, use the palette of the example `examples/mega65/pal_demo`.

1. **Open the example:** On the start page, open `pal_demo` from the example projects (to change it, use **Copy**, then you have your own version). *Expected:* The project is open; in the file tree there is `graphics/pal65.vcpal`.
2. **Open the palette:** At the top, choose the main section **Graphics** and in the second row **Palette**. If nothing is open yet, the editor shows the **Palette files** of the project as buttons; click `graphics/pal65.vcpal`. A double-click on the file in the file tree also opens it in the palette editor. *Expected:* on the left a grid with 256 color fields, on the right the card **Color 0**.
3. **Choose an entry:** click a field (16 fields per row) or use the arrow keys. *Expected:* On the right the card **Color n** appears with the color field, the footer shows number and hex value, for example "Color 3 ($03) of 256".
4. **Change the color:** drag the sliders **Red**, **Green** and **Blue** (0 to 255 each), or enter a color as `#rrggbb` in the field **Hex value** and confirm with the Enter key. *Expected:* The field in the grid shows the new color, the footer says "● unsaved".
5. **Back:** **Cmd/Ctrl+Z**; **Cmd/Ctrl+Shift+Z** brings the step back. **Default palette** (button above the grid) sets all entries to the supplied palette.
6. **Save:** **Cmd/Ctrl+S** writes the `.vcpal` file.

## Layout of the user interface

The palette editor has two areas side by side:

| Area | Content |
|---|---|
| Left | the bar with Undo, Redo, the name of the palette and "n colors", plus **Default palette**, **Export …** and **Save**; below it the **Color grid** with 16 fields per row and, if the project has several palettes, the list of palette files |
| Right | the card **Color n** with the color field, the sliders **Red**, **Green**, **Blue** and the **Hex value**, below it **In use** and **Output**; at the very bottom the footer with name, number and hex value of the chosen color |

In the grid you move with the arrow keys (Page Up and Page Down jump four rows, Home and End to the first and last entry). The field **Hex value** accepts exactly six hex digits (with or without `#`); otherwise the note "That is not a color value; use #rrggbb." appears and nothing is applied.

![The palette editor in the area 'Grafik' for the MEGA65 palette pal65 with 256 colors. In the middle is a grid of 16 by 16 color cells with the first cell selected. On the right are the cards 'FARBE 0' with a preview, sliders 'ROT', 'GRUEN' and 'BLAU', the field 'HEXWERT' with #000000, 'IM EINSATZ' and 'AUSGABE'.](bild:palette-mega65 "The MEGA65 palette editor with 256 color cells and the card for the selected entry.")

**In use** lists the files of your project that use the chosen color index (sprite, player/missile, character set, tiles or screen); a click on the file name opens the file. If the color is not used anywhere, it says "No media file in the project uses this color." Under **Output** it says how large the data is as assembler text, with a second button **Export …**.

Keys: **Cmd/Ctrl+Z** and **Cmd/Ctrl+Shift+Z**, **Cmd/Ctrl+S**, **Cmd/Ctrl+Shift+E** (export). The command palette (**Cmd/Ctrl+K**) offers the same commands.

> **Info:** The palette of the first 16 entries matches the C64 colors. The further entries form a color table with which you can build gradients and your own tones. Before you overwrite everything, make a copy.

## Export and including

**Export …** opens the export dialog. There you choose the **Format** (**Assembler**, **Constants**, **Binary**, **PRG** or **BASIC DATA**), the **Name** and, depending on the format, an **Address**; the **Preview** shows text and bytes. **Save in project** writes the palette to `build/export/<name>.asm`; **Insert as include** also puts the `.include` line into the source you chose under **Target in the source**. You do not need the export for including: `.include "graphics/<name>.vcpal"` converts the palette by itself when you build, as with the other editors.

The file contains three tables of 256 bytes each: `<name>_red`, `<name>_green` and `<name>_blue`. The export takes care of a hardware quirk: the palette registers `$D100`, `$D200` and `$D300` expect the two halves (four bits) of each value **swapped**. You do not have to convert this yourself.

### Example: load your own palette

The project `examples/mega65/pal_demo` loads `graphics/pal65.vcpal` into palette bank 1 of the VIC-IV:

```asm
.target mega65
.cpu 45gs02 { illegals = deny }

.org $2000
.include "lib/vic4.inc"

start:  sei
        vic4_unlock
        vic4_load_palette pal65_red, pal65_green, pal65_blue, 1
        vic4_use_palette 1
        lda #$40
        sta $d070                   ; leave bank 1 mapped in at $D100-$D3FF (for reading back)
done:   bra done

vic4_palette_runtime
.include "graphics/pal65.vcpal"
```

- `vic4_unlock` unlocks the extra registers of the VIC-IV. Without this line, the chip ignores all further commands.
- `vic4_load_palette` copies the 256 entries from the three tables into **palette bank 1**. The MEGA65 has four banks; the fourth number chooses one.
- `vic4_use_palette 1` makes text and bitmaps use this bank.
- `vic4_palette_runtime` stores the loading subroutine, **after** the program.

1. Open `examples/mega65/pal_demo`, build with **F7**, run with **F5** (Xemu).
2. The program shows nothing of its own, it only loads the palette.
3. In the palette editor, change an entry, save, build and run again. In the export dialog (preview, tab **Hex**) you see the bytes of the tables. Remember: they contain the halves swapped (`$8A` becomes `$A8`).

> **Caution:** A loaded palette only colors what uses its entries: full color characters (chapter [Character set editor](kapitel:zeichensatz)) and sprites or screen colors that point to this palette bank. Without such users, nothing changes on the screen.

## Atari and the others: choose colors in the editor

The other systems have **no** freely changeable palette, because the colors are hardware. The sprite editor and the character set editor show the color grid of the system on the right in the **Colors** card: on the C64 16 fields, on the Atari 128 (with the **Palette view** **Project**, **PAL** or **NTSC**). You first choose a color value (for example the sprite color), then you click a field and assign it to the value. The number is saved, not the color tone.

*[Bild folgt in E13: palette-atari]*

How the Atari colors really look in PAL or NTSC is shown by the small measuring programs `examples/atari/palette0` and `examples/atari/palette1`: they paint each color value as a stripe on the screen.

## Hardware features and limits

| System | Colors | Changeable? |
|---|---|---|
| C64, C128 (40 columns) | 16 | no (emulators show them differently) |
| VIC-20 | 16 | no |
| Plus/4 | 121 | no (TED palette in the editor still incomplete) |
| Atari | 128 (16 hues x 8 brightness levels) | no; PAL and NTSC look different |
| MEGA65 | 256 out of 16.7 million | **yes**, 4 palette banks |

On the MEGA65, a palette only affects full color characters, bitmaps and sprites that use this bank; you should stay true to the "C64 colors" default of the first 16 entries if you want to keep using the standard graphics.

## Neighboring editors and glossary

- The colors are used by the [Character set editor](kapitel:zeichensatz) (full color characters), the [Sprite editor](kapitel:sprites) and the [Screen editor](kapitel:bildschirm-kacheln).
- Terms: Glossary (file `GLOSSAR.md`): palette, RGB, PAL, NTSC.

## Common stumbling blocks

- **The colors in the program differ from the editor:** On the Atari the video standard matters, on the C64 the emulator (VICE shows the colors by its own palette).
- **The palette has no effect:** On the MEGA65 it only works if `vic4_use_palette` chooses the loaded bank and something uses its entries.
- **A hex value is not applied:** The field **Hex value** accepts exactly six hex digits (`#rrggbb`); for anything else the note "That is not a color value; use #rrggbb." appears.

## What next?

Now comes sound: [The tracker](kapitel:tracker). For looking things up: [MEGA65](hilfe:mega65), [Export](hilfe:export).
