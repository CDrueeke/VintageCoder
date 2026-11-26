# Character set editor

Edits 256 characters (C64) or 128 characters (Atari ANTIC 2), each 8x8 pixels; multicolor as 4x8 with double-width pixels. You find the editor in the main section **Graphics**, in the second row under **Character set**. A double-click on a `.vcchar` file in the project tree also opens it there.

- Select a character on the left in the **Character overview** (the arrow keys move through the grid), set pixels in the middle.
- The order is that of the **screen codes** (not ASCII): C64 `@`=0, `A`=1 ...; Atari: space=0, digits from 16, capital letters from 33.
- Loading in a program: `gfx_load_charset` (assembler) or `gosub @gfx_charset` (BASIC).

## Layout
The editor has three areas side by side. In narrow windows, the right one moves below the other two. It follows the [common layout](help:graphics).

| Area | Content |
|---|---|
| Left | the **Character overview** (16 characters per row), below it the **Ranges** (All, Letters, Digits, Punctuation, Custom) and, if the project has several character sets, the **Character set files** |
| Middle | toolbar, number of the character, the drawing area with the **Neighbors** (the character before and after) and the footer |
| Right | **Preview in the picture**, **Template** with **Origin**, **Character** (number and position), **Colors** and **Shift** |

![The character set editor in the area 'Grafik' with the file font.vcchar. On the left is the character matrix 'FONT' with 256 characters and the ranges 'Alle', 'Buchstaben', 'Ziffern', 'Satzzeichen' and 'Frei'. In the middle is the large drawing area with a toolbar. On the right are the cards 'VORSCHAU IM BILD', 'VORLAGE', 'ZEICHEN', 'FARBEN' and 'VERSCHIEBEN'.](bild:zeichensatz-ueberblick "The character set editor with the character matrix, drawing area and the cards on the right.")

The toolbar contains **Pen** (D), **Eraser** (E), **Eyedropper** (I), **Fill** (F), **Line** (L), **Rectangle** (R), **Selection** (S), Undo, Redo, **Flip horizontally** (X), **Flip vertically** (Y), **Grid** (G), the zoom and the file buttons **New charset …**, **Load PNG …**, **Export ...** and **Save**. **Export ...** (Cmd/Ctrl+Shift+E) opens the [export dialog](help:export). The single keys are listed under [Keyboard shortcuts](help:shortcuts).

On the right, the card **Colors** shows the pixel values with registers (on the C64, for example, background `$D021`, in multicolor mode `$D022` and `$D023`, on the Atari ANTIC 2 `COLPF2` and `COLPF1`). On the Atari, the **Palette view** switches between **Project**, **PAL** and **NTSC**. **Shift** moves the whole character with four arrow buttons; **Clear character** deletes all dots. The **Ranges** jump to the first character of a group; with **Custom**, you enter **from** and **to** as character numbers. The **Preview in the picture** shows a text (field **Text for the preview**) in the characters of your set.

## New character set and template
A **new character set** (button **New charset …** in the editor, or `vcc media new`) does not start empty, but with a **reference to the standard character set of the system** in your ROM, as it appears after power-on: on the C64, C128 (40 columns) and MEGA65 "Uppercase and graphics", on the Atari the standard set, on the Plus/4 its own set. With `vcc media new ... --blank` it starts empty.
The file then contains only `"base": "rom:c64/chargen#upper"` and **your changes** (`overrides`, format 2); the original characters are not in the project. If the system has several character set sources (C64: two sets per ROM revision, Atari: standard and international, C128 VDC), you choose another one in the card **Template** under **Template to load** and click **Load template**; this replaces all characters (undo with Cmd/Ctrl+Z). An entry whose ROM is not found is gray. If the ROM is missing, the editor shows the **own standard set** of VintageCoder (entry **Standard (own)**, label OWN, free under CC0); your changes are kept and apply on top of the real ROM as soon as it is there.

The card **Origin** says in words where the characters come from, with a label (**OWN**, **OWN, CHANGED**, **FROM YOUR ROM** or **ROM MISSING**). Changed characters have a small dot in the overview; **Reset character to template** brings back the original of the chosen character.

*[Bild folgt in E13: zeichensatz-standard]*

## Standard character sets
The command line creates the standard character sets for the target systems of the project (with a reference to your ROM, `VC-Standard` as own copy): `vcc media std-charset` (list of sources with status) and `vcc media std-charset rom:c64/chargen#upper [--profile ID] [--copy]`. The dialog **Add character set** in the user interface is not yet available. In the dialog **Create new project**, you choose the **Character set of the project** (*Automatic*, *Standard only (own)* or *None*), see [Start page, setup assistant and new project](help:wizard).

VintageCoder does not ship the character sets of the systems; references to them can be passed on freely. See the help page [ROM files](help:roms).
