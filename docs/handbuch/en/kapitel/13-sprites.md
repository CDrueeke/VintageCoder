---
id: sprites
titel: Sprite and PMG editor
reihenfolge: 13
stufe: Beginner
zeit: 30 minutes
siehe_auch: [grafik-ueberblick, zeichensatz, paletten]
referenz: [sprite, export, graphics]
i18n_status: maschinell
i18n_quelle: a1104aefc942
---

# Sprite and PMG editor

**Goal:** You draw a sprite (C64) or a player (Atari), make it move, export it and show it in a program.

## What is the sprite editor for?

In the sprite editor you paint the **moving figures** of your program: the player, enemies, shots, a mouse pointer, a spaceship. Everything that should move over a background without destroying it is a candidate for a sprite.

Typical uses in real retro projects:

- **A space game:** Your ship is a sprite, and the shots are more sprites. You move them by only changing their position. The picture behind them (stars) stays untouched.
- **A mouse pointer or selection frame** in a demo or a tool.
- **A figure with a walking animation:** four to eight frames that are shown one after the other.
- **Large figures:** several sprites next to each other, for example an end boss made of 2x2 sprites.

In the finished program, a sprite ends up as a **data block** in memory (63 bytes per sprite on the C64). Your program tells the graphics chip where this block is, and the chip draws it. The editor creates the data and the color constants for you. How you include them is described below in the section "From the editor to the program".

## Basic terms, no prior knowledge needed

- **Pixel / dot:** the smallest point of an image. In the sprite editor you see it as a small square.
- **Sprite:** a small image that the graphics chip puts over the screen **by itself** (C64: VIC-II, Atari: GTIA).
- **Frame:** a single image of a sprite. Several frames one after the other make an animation, like a flip book.
- **Hires:** high resolution. Each dot is on or off, and the sprite has **one** color.
- **Multicolor:** many colors. The dots are twice as wide, but there are several colors. The resolution goes down, the colors go up.
- **Pixel value:** the "color" that a dot has, as a number (for example 0 = transparent). A register of the chip decides which real color is behind it.
- **Register:** a memory location where you write numbers to control the chip (for example `$D027` for the color of sprite 0).
- **Sprite pointer:** a byte that tells the chip in which 64-byte block the data of a sprite is.
- **PMG (Player/Missile Graphics):** this is what sprites are called on the Atari.
- **Onion Skin:** the previous frame shines through faintly, so you can place movements cleanly. The switch is called **Onion Skin**.

## What you need

- a C64 or Atari project (for MEGA65, the C64 sprites apply),
- the basics from [Graphics and sound at a glance](kapitel:grafik-ueberblick): create an asset, export it, include it.

## What is a sprite?

A **sprite** is a small image that the graphics chip **puts over the screen by itself**. You do not have to paint it into the picture. You only say where it should be. This way, figures move fast and smoothly without destroying the background.

- **C64 (VIC-II):** eight sprites, each **24x21 dots** in size (63 bytes, padded to 64 bytes in memory). In **Hires** mode, a sprite has one color. In **Multicolor** mode, the dots are twice as wide (so 12x21), but there are four values: transparent, the own **sprite color** and two colors that **all** sprites share (**Multicolor 1** and **Multicolor 2**).
- **Atari (GTIA):** The Atari calls them **players/missiles** ("PMG", Player/Missile Graphics). There are four **players**, each **8 dots wide** and as tall as the screen. Each player has its own color. You can place several players next to each other to build larger figures. There are also missiles (narrow shots), which this manual does not cover further.

## My first sprite (using `sprite_demo`)

You do not need your own project for this: The example `examples/c64/sprite_demo` already contains a sprite that you can look at and change.

1. **Open the example:** On the start page, open `sprite_demo` from the example projects (to change it, use **Copy**, then you have your own version). *Expected:* The project is open; in the file tree there is the folder `graphics` with `hero.vcsprite`.
2. **Open the sprite:** At the top, choose the main section **Graphics** and in the second row **Sprites**. On the left is the list of **Sprite files**; click `hero.vcsprite`. (A double-click on the file in the file tree also opens it in the sprite editor.) *Expected:* In the middle you see the sprite, greatly enlarged, on the left the images (frames), on the right the cards **Colors**, **Sprite** and **Preview in the game**.
3. **Paint:** Choose the **Pen** (key **D**) and click on the canvas. *Expected:* A wide dot (multicolor) appears in the chosen color; the footer says "● unsaved".
4. **Erase:** Right-click a dot. *Expected:* The dot becomes transparent.
5. **Change the color:** In the **Colors** card, click another color value (or press **1** to **4**) and keep painting. *Expected:* The pen now sets the new value.
6. **Undo:** **Cmd/Ctrl+Z**. *Expected:* The last stroke disappears; **Cmd/Ctrl+Shift+Z** brings it back.
7. **Save:** **Cmd/Ctrl+S**. *Expected:* The note "unsaved" disappears.
8. **Build and run:** **F7**, then **F5**. *Expected:* The emulator shows your changed sprite on the screen.

You create a new sprite file with **New sprite file …** (on the left below the file list, also on the empty page of the editor). In the dialog, choose the profile under **Type and system**, enter the **Name** (letters, digits, `_` and `-`), check the **Folder in the project** (default `graphics`) and click **Create**. **Open sprite file …** fetches an existing file from the project.

## Layout of the user interface

The sprite editor has three areas side by side:

| Area | What it shows |
|---|---|
| Left | the **Sprite files** of the project, **Open sprite file …**, **New sprite file …** and below them the tabs **Frames** and **Animations** |
| Middle | the toolbar, the **Drawing area** with the chosen frame (greatly enlarged, fitted) and below it the timeline of the animation; at the very bottom the footer |
| Right | the cards **Colors**, **Palette**, on the Atari **Player/missile**, **Sprite**, **Preview in the game** and **Shift** |

The **toolbar** contains, from left to right: **Pen** (D), **Eraser** (E), **Eyedropper** (I), **Fill** (F), **Line** (L), **Rectangle** (R), **Selection** (S), **Flip horizontally** (X), **Flip vertically** (Y), Undo and Redo, **Onion Skin** (O), **Grid** (G), zoom (**Zoom out**, display in percent, **Zoom in**, **Fit**) and on the right the file buttons **Load PNG …**, **Export …** and **Save**. When you point at the buttons, they show their key. The **footer** shows name, size and "frame i of n"; unsaved changes are marked by a **●**, and a selection shows its size there.

![The Grafik tab with the Sprites page and the file hero.vcsprite: on the left the sprite files and two frames, in the middle the 12x21 pixel multicolor sprite of a figure in yellow, brown-red and blue, on the right the cards for colors, sprite (multicolor on), preview in the game and move. Below the drawing area a note says there is no animation.](bild:sprite-c64 "The sprite editor with a multicolor sprite.")

**Keys** (the single keys apply when the canvas has the focus):

| Key | Effect |
|---|---|
| D / E / I | Pen / Eraser / Eyedropper |
| F / L / R / S | Fill / Line / Rectangle / Selection |
| X / Y | flip horizontally / vertically |
| 1 to 4 | choose color value (digits up to 9, as far as the sprite has that many values) |
| O / G | Onion Skin / Grid |
| 0 / + / - | Fit / Zoom in / Zoom out (also **Cmd/Ctrl** and mouse wheel) |
| Arrow keys | move the cell marker; **Enter** or **Space** sets a dot, **Delete** or Backspace removes it |
| Page Up / Page Down, Alt+Left/Right arrow | previous / next frame |
| Cmd/Ctrl+C / X / V | copy / cut / paste selection |
| Esc | clear selection |
| Cmd/Ctrl+Z | Undo (add Shift: Redo) |
| Cmd/Ctrl+S | Save |
| Cmd/Ctrl+Shift+E | Export dialog |

You can also reach all tools, zoom, grid and Onion Skin through the **command palette** (Cmd/Ctrl+K). With the **Selection** tool, a right-click on the canvas opens a menu with Copy, Cut, Paste, Select all, Delete selection and Clear selection.

### Hires or multicolor, per frame

You can mix frames of a sprite: in the **Sprite** card you switch **Multicolor** on or off (the switch applies to the chosen frame). When you switch, the editor converts the dots (from hires to multicolor, two dots become one). **Cmd/Ctrl+Z** undoes the change. The two shared colors **Multicolor 1** and **Multicolor 2** apply to all frames of the file.

*[Bild folgt in E13: sprite-c64-multicolor]*

### More sprite properties (C64)

In the **Sprite** card, C64 sprites have three switches besides **Multicolor** (`$D01C`):

| Switch | Meaning | Register |
|---|---|---|
| Double width | twice as wide (the editor shows the size as "In the game: n dots") | `$D01D` |
| Double height | twice as high | `$D017` |
| Behind the background | behind the background | `$D01B` |

The switches apply to the whole file. They end up in the export as constants (`<name>_expand_x`, `<name>_expand_y`, `<name>_behind`). The **Preview in the game** shows the chosen frame in front of a fixed background of sky and meadow, in the colors of the target system and with the switches you set, once enlarged and once in original size. Below it you see how many bytes a frame and the whole file use.

### Colors and palette

In the **Colors** card, the color values of the sprite are listed (for example transparent, **Sprite color**, **Multicolor 1**, **Multicolor 2**). You choose a value with a click or a digit key; the pen then paints with it. Which real color a value gets, you choose below in the color grid: click a color, and it applies to the chosen value. The heading **Palette** names the video standard. Where there are PAL and NTSC colors (Atari), you switch the **Palette view** between **Project**, **PAL** and **NTSC**. A transparent value has no color.

## Step by step: an Atari player

1. Create a file with **New sprite file …** and choose the profile **Atari PMG single line (8 pixels wide)** under **Type and system**. The file is called `.vcpmg`.
2. On the left under **Frames** you find the **four players P0 to P3**. Each is 8 dots wide and 32 lines high by default. You choose the color of each player on the right in the **Colors** card. The Atari has 128 tones for this (palette view can be switched to PAL or NTSC).
3. Paint as on the C64.

*[Bild folgt in E13: sprite-atari-pmg]*

The **Player/missile** card sets two things that end up as constants in the export:

- **Width of player 0 to 3** (`SIZEP0` to `SIZEP3`): **Normal**, **Double** or **Quadruple**.
- **Priority** (`PRIOR`, `$D01B`): **Not set**, **Players before playfield**, **P0/P1, playfield, P2/P3**, **PF0/PF1, players, PF2/PF3** or **Playfield before players**. A priority that is not set is exported as "Players before playfield".

> **Info:** A player shows the memory line by line from top to bottom. The first 32 or so lines lie above the visible picture: draw the artwork further down, or increase the height of the image.

## Frames, animation and Onion Skin

An **animation** is made of several frames (images) that are shown one after the other in quick succession (like a flip book).

1. On the left in the **Frames** tab, **New** creates a new empty frame, **Duplicate** a copy of the chosen one, and **Delete** removes one. A click on a frame chooses it; **Page Up/Down** or **Alt+Left/Right arrow** change the frame.
2. Switch on **Onion Skin** (toolbar or key **O**): the previous frame appears faintly as a shadow under your frame. This way you get the spacing right. For the first frame the switch is grayed out, because there is no previous one.
3. For a sequence, switch to the **Animations** tab on the left. **New** creates an animation with the chosen frame; you choose between several animations with their buttons, you name them in the field **Animation name**, and **Delete** removes the whole animation. The playback mode is **Loop**, **Back and forth** or **Once**. The steps are shown as a film strip: each shows a thumbnail, number and **Duration in ticks (1/50 s)**. You move a step with the arrow buttons or by dragging, remove it, or use **Append selected frame** to append the chosen frame as a new step. The field **Speed (frames per second)** sets the duration of all steps at once.
4. **Play:** below the canvas, the **Timeline** shows the chosen animation. The button **Play** starts it in real time, **Stop** stops it. You can switch the playback mode there with a click, and **Enlarge preview** opens the animation in a window of its own. A click on a step chooses its frame for editing.

![The sprite editor with hero.vcsprite, tab 'Animationen'. On the left the animation 'laufen' with loop, speed 8.3 and two frames, in the middle the enlarged figure, below it the timeline with the animation and two frames, on the right the colors and the preview in the game.](bild:sprite-animation "The 'Animationen' tab with the animation 'laufen' and the timeline below the drawing area.")

You can also do it the other way round: in the frame list, select several frames with **Shift-** or **Ctrl-click** (**Cmd/Ctrl+A** selects all) and click **Animation from selection**. The dialog **Animation from n frames** asks for the name, the **Playback** and the **Speed (frames per second)**; the frames run in the order of the list. The menu on right-click on a frame offers the same as **Animation from selection …**.

If there are animations, the export also creates a file `<name>.anim.asm` with the sprite pointers (`_ptr`), the durations (`_delay`), the length (`_len`) and the mode (`_mode`). A raster interrupt counts the duration down and writes the next pointer to `$07F8` plus the sprite number.

### Selection and rectangle

With the **Selection** tool (key **S**) you drag a frame. Inside it you can copy, cut and paste with **Cmd/Ctrl+C**, **Cmd/Ctrl+X** and **Cmd/Ctrl+V** and delete with **Delete**. **Cmd/Ctrl+A** selects the whole frame, **Esc** clears the selection. The **Rectangle** tool (key **R**) draws a rectangle as an outline. Under **Shift** in the right column you move the whole frame by one dot with four arrow buttons; **Clear frame** deletes all dots of the chosen frame.

## Sprites from other sources

**Load PNG …** (toolbar) chooses an image file and converts it into the pixel values of the chosen frame; the editor saves the file while doing so. Importing from SPD files, from raw data and from sprites of other systems is not available in this version yet.

## Example 1: sprite on the C64

Open `examples/c64/sprite_demo` (see the program in the [overview chapter](kapitel:grafik-ueberblick)). After F7 and F5 the sprite is in the emulator.

The most important lines in the program:

```asm
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
        sta vic2.spmc0          ; multicolor 1
        lda #6
        sta vic2.spmc1          ; multicolor 2
```

- The **sprite pointer** (`$07F8` for sprite 0) tells the chip in which 64-byte block the data is. If it is at `$2000`, the pointer is `$2000 / 64 = $80`. This is why the file has `.org $2000` before the `.include`: sprite data must start on a 64-byte boundary.
- The colors in the program (`7`, `2`, `6`) are the same ones that the editor shows as sprite color, Multicolor 1 and Multicolor 2. The register values are not in the sprite bytes, but the export delivers them as constants `hero_sprite`, `hero_mc1` and `hero_mc2`. If you write `lda #hero_sprite` instead of `lda #7`, your program automatically follows every color change in the editor.

> **Tip:** If several frames follow each other in memory (frame 0 at `$2000`, frame 1 at `$2040` and so on), you choose another frame by changing the pointer: `$80`, `$81`, ...

## Example 2: player on the Atari

In the project `examples/atari/pmg_ship`, four players of a spaceship are in `graphics/ship.vcpmg`. The program `pmg_ship.asm` puts them next to each other on the screen:

```asm
.target atari8
.cpu sally { illegals = deny }

.const SDMCTL = $022f           ; shadow register of DMACTL (the operating system copies it every frame)

.org $2000
start:  lda #$40
        sta antic.pmbase        ; player/missile page = $4000 (player 0 is at $4400)
        lda #3
        sta gtia.gractl         ; switch players and missiles on
        lda #$3a                ; display list on, player DMA on, single-line resolution, normal field
        sta SDMCTL
        lda #ship_p0
        sta gtia.colpm0
        lda #ship_p1
        sta gtia.colpm1
        lda #ship_p2
        sta gtia.colpm2
        lda #ship_p3
        sta gtia.colpm3
        lda #100                ; four players next to each other, 8 pixels wide each
        sta gtia.hposp0
        lda #108
        sta gtia.hposp1
        lda #116
        sta gtia.hposp2
        lda #124
        sta gtia.hposp3
done:   jmp done

.org $4400                      ; player 0; players 1 to 3 follow at a distance of $100
.include "graphics/ship.vcpmg"
.run start
```

This is how it works:

- The export stores the four players **one after the other at a distance of 256 bytes** (1024 bytes). In single-line resolution, player 0 is at **PMBASE + `$400`**, so here at `$4400` if the base is `$4000` (`$40` in `PMBASE`). The `.org $4400` puts the data in the right place.
- `GRACTL` switches the players on, `SDMCTL` (the shadow copy of `DMACTL`) makes the chip fetch the data from memory by itself.
- The constants `ship_p0` to `ship_p3` come from the export and contain the colors you chose in the editor.

1. Open `examples/atari/pmg_ship`, build with **F7**, run with **F5**.
2. In the editor, change the color of a player, then build and run again.

## Hardware features and limits

| | C64 (VIC-II) | Atari (GTIA) | MEGA65 |
|---|---|---|---|
| Number | 8 sprites | 4 players + 4 missiles | like C64 |
| Size | 24x21 dots (multicolor: 12x21 wide dots) | 8 dots wide, height free up to screen height | like C64 |
| Colors | Hires: 1 color per sprite. Multicolor: 3 colors + transparent, 2 of them shared by all sprites | 1 color per player (128 tones) | like C64 |
| Change size | X2/Y2 double width/height | `SIZEP`: single, double, quadruple | like C64 |
| Memory | 64-byte block, address divisible by 64 | 256 bytes per player, base set by `PMBASE` | like C64 |

What this means: more than eight sprites at the same time are only possible with tricks (raster interrupt). A multicolor sprite looks more colorful, but is coarser. Two sprites of the same chip always share two colors in multicolor, so they change together.

## From the editor to the program

1. **Export …** (toolbar or **Cmd/Ctrl+Shift+E**) opens the export dialog. There you choose the **Format** (**Assembler**, **Constants**, **Binary**, **PRG** or **BASIC DATA**), the **Name** and, depending on the format, an **Address**. The **Preview** shows text and bytes. With **Save in project**, **Insert as include**, **Copy into source** or **Test in emulator** you decide what happens to the result. You do not need the export for including: when you build (**F7**), VintageCoder converts the sprite itself at the `.include`.
2. **Assembler:** `.include "graphics/hero.vcsprite"` at the place in memory that you set with `.org` (C64: multiple of 64). The export delivers data and the color constants `hero_sprite`, `hero_mc1`, `hero_mc2`.
3. **BASIC:** export the format **BASIC DATA** (if you like with **Include a read routine**) and include it with `#include`; copy the lines into the sprite memory with `READ` and `POKE`.
4. **Raw data:** format **Binary** for `.incbin` or for loading from disk.
5. In the program you set **pointer, colors, position and "sprite on"** (see the two examples above).

## Common stumbling blocks

- **The sprite is shifted or shows garbage:** The data is not on a 64-byte boundary (C64) or not at `PMBASE + $400` (Atari).
- **The colors in the emulator differ:** The sprite colors are in the registers, not in the data. Set them in the program, best with the export constants (`hero_sprite`, `hero_mc1`, `hero_mc2`).
- **The sprite does not appear:** `vic2.spena` does not have the bit; or the position is outside the visible area (on the left the visible area starts at about X = 24, at the top at Y = 50).
- **Multicolor looks too wide:** That is how it is: a multicolor dot is two hires dots wide.

## Neighboring editors and glossary

- The background in front of which your sprite moves is created in the [Screen and tile editor](kapitel:bildschirm-kacheln).
- You choose the colors from the [Palette](kapitel:paletten) of your system.
- Look up terms: Glossary (file `GLOSSAR.md`) (sprite, frame, multicolor, sprite pointer, PMG).

## What next?

In the next chapter, you put characters and tiles together into whole pictures: [Screen and tile editor](kapitel:bildschirm-kacheln). For looking things up: [Sprite/PMG editor](hilfe:sprite), [Export](hilfe:export).
