---
id: kuerzel-faq
titel: Keyboard shortcuts, FAQ, troubleshooting
reihenfolge: 20
stufe: Beginner
zeit: 15 minutes
siehe_auch: [oberflaeche, editor, debugger, bauen-starten, systeme]
referenz: [shortcuts, roms, settings]
i18n_status: maschinell
i18n_quelle: 49a06ee39ea9
---

# Keyboard shortcuts, FAQ, troubleshooting

**Goal:** Here you find all keyboard shortcuts of VintageCoder at a glance, and answers to the questions that come up most often.

All shortcuts apply on the Mac **and** on Windows and Linux. The **Cmd** key on the Mac (the command key) is called **Ctrl** on Windows and Linux; **Shift** is the Shift key, **Alt** is the Option key on the Mac. In the tables it therefore says **Cmd/Ctrl+S**: on the Mac you use Cmd, otherwise Ctrl. Behind each command in the menus and in the command palette (**Cmd/Ctrl+K**) is its shortcut.

> **Info:** Some commands do not exist in this version yet; the menus and the command palette do not show them: zoom, grid and Onion Skin of the editors through the menu (the buttons and keys in the editor exist), Convert BASIC to assembler, Switch encoding, BASIC: Hex suggestions and the character set imports (standard set, ROM, PNG grid). The keys of the editors (tables below) are not affected.


## Build, run, debug

| Shortcut | Effect | Menu |
|---|---|---|
| F7 | Build project | Project > Build project |
| F5 | Run start program (builds first) | Project > Run start program |
| F6 | Debug start program (builds, starts the emulator with a debug session, opens the area "Debug") | Project > Debug start program |
| Alt+Shift+F7 | build active file | Project > Active file > Build active file |
| Alt+Shift+F5 | run active file | Project > Active file > Run active file |
| Alt+Shift+F6 | debug active file | Project > Active file > Debug active file |
| Alt+F8 | Run tests | Project > Run tests |
| F8 / Shift+F8 | next / previous problem | Project > Next problem, Previous problem |
| Cmd/Ctrl+Shift+E | Export ... (export dialog) | File > Export ... |

You set the **start program** in the main section **Project** under **Sources** (star on the file). The shortcuts with Alt+Shift apply to the file in the active tab. You can only build and run when a project is open and no build is running. If a debug session is already running, F6 only takes you back to the area **Debug**.

## In the debugger (only when the program is paused)

| Shortcut | Effect | Menu |
|---|---|---|
| Alt+F5 | Continue to the next breakpoint | Project > Debug > Continue |
| Alt+F6 | Pause | Project > Debug > Pause |
| F11 | Step: one instruction, a `JSR` is entered | Project > Debug > Step into |
| F10 | Over: one instruction, a `JSR` runs through | Project > Debug > Step over |
| Shift+F11 | Out: until the return of the subroutine | Project > Debug > Step out |
| Shift+F5 | Stop debugging | Project > Debug > Stop debugging |
| - | Run to here (to the line with the cursor) | Project > Debug > Run to here |
| F9 | Toggle breakpoint (line with the cursor) | Edit > Toggle breakpoint |
| Click on the line number | set or delete breakpoint | - |

With atari800, the program cannot be paused with a button: there you press **F8 in the emulator window**.

On the Mac, the menu bar of the system applies; on Windows and Linux, the menu bar in the window. Everything that is in the menu is also in the command palette (**Cmd/Ctrl+K**).

## Files and projects

| Shortcut | Effect |
|---|---|
| Cmd/Ctrl+Alt+N | New project ... |
| Cmd/Ctrl+Shift+O | Open project ... (project file) |
| Cmd/Ctrl+Shift+W | Close project |
| Cmd/Ctrl+Alt+0 | Project area (settings and overview of the project) |
| Cmd/Ctrl+O | Open file ... |
| Cmd/Ctrl+N | New untitled tab (also without a project); name when saving |
| Cmd/Ctrl+Shift+T | New note |
| Cmd/Ctrl+Alt+Right / Cmd/Ctrl+Alt+Left | Next / previous tab |
| Cmd/Ctrl+S | Save (also in the sprite, character set and palette editor) |
| Cmd/Ctrl+Alt+S | Save all |
| Cmd/Ctrl+W | Close file |
| Cmd/Ctrl+Comma | Settings |
| Cmd/Ctrl+Q | Quit VintageCoder |

## Edit and search

| Shortcut | Effect |
|---|---|
| Cmd/Ctrl+Z | Undo (source, sprite, character set and palette editor) |
| Cmd/Ctrl+Shift+Z | Redo |
| Cmd/Ctrl+F | Find ... |
| Cmd/Ctrl+Alt+F | Replace ... |
| Cmd/Ctrl+L | Go to line ... |
| Cmd/Ctrl+Shift+F | Find in project ... (opens the page "Search" in the tray) |
| Cmd/Ctrl+. | Quick fix ... |
| Cmd/Ctrl+Alt+C | Toggle comment |
| Cmd/Ctrl+Alt+K | Toggle bookmark |
| Cmd/Ctrl+Alt+J / Cmd/Ctrl+Alt+H | next / previous bookmark |
| F1 | Help for the word under the cursor (opens the help and looks up the word) |
| F12 | Go to definition |
| Cmd/Ctrl+I | Markdown: Italic |
| Cmd/Ctrl+E | Markdown: Code |
| Cmd/Ctrl+Shift+X | Markdown: Strikethrough |
| Cmd/Ctrl+Shift+K | Markdown: Insert link |

Suggestions appear by themselves while you write (after typing, after "." and after "@"); there is no key to open them, but there is the command **Open completion** in the command palette. **Bold** has no shortcut (command "Markdown: Bold" in the command palette).

In the input fields of the search and of "Go to line", **Enter** runs the search (in the search with **Shift+Enter** to the previous match), **Esc** closes the field.

## View and user interface

| Shortcut | Effect |
|---|---|
| Cmd/Ctrl+K | Search for a command or file ... (command palette) |
| Cmd/Ctrl+B | Show / hide sidebar |
| Cmd/Ctrl+Alt+Z | Zen mode: only header and editor (hide sidebar, tray, machine panel and status bar); once more restores the layout |
| Cmd/Ctrl+Alt+B | Show / hide machine panel |
| Cmd/Ctrl+J | Show / hide tray below the editor |
| Cmd/Ctrl+Shift+Enter | Maximize and restore the area with the focus |
| Cmd/Ctrl+Alt+T | Split editor / remove split |
| Cmd/Ctrl+Plus / Cmd/Ctrl+Minus / Cmd/Ctrl+0 | Make UI larger, smaller, original size (menu View > Zoom and font) |
| Cmd/Ctrl+Comma | Settings |
| Cmd/Ctrl+Alt+0 | Project area |

In the **View** menu there are also, without a shortcut, **Switch light / dark**, **High contrast on / off**, **Reset layout**, **Editor font larger/smaller**, **Reset appearance (size, font, layout)**, **Word wrap in the editor** and the commands for maximizing single areas (sidebar, document area, tray, machine panel). You also choose the color scheme in the menu **Color scheme** of the header.

## Help

| Shortcut | Effect |
|---|---|
| F1 | Open help; in the editor, F1 looks up the word under the cursor |

The **Help** menu also contains **Open help in a tab**, the **Setup assistant** (whole, or ROMs only, emulators only, real hardware only, design and editor only, Look for new items), **Send feedback ...**, **Check for updates ...** and **About VintageCoder**. You find export, import and reset of the configuration under Settings > Backup.

## Single keys in the editors

In the editors there are single keys as soon as the canvas has the focus and no text field is active. You can switch them off under **Settings > Shortcuts** in the section **Keys** with the switch **Single keys in the editors** (default: on). Arrows, Space, Esc and Delete apply even then.

| Editor | Keys |
|---|---|
| Sprites and character sets | **D** Pen, **E** Eraser, **I** Eyedropper, **F** Fill, **L** Line, **R** Rectangle, **S** Selection, **X** / **Y** flip horizontally / vertically, **1** to **4** pixel value (color), **G** Grid, **O** Onion Skin (sprites only), **Arrow keys** move the cell marker (**Enter** or **Space** sets a dot, **Delete** removes it), **Page Up** / **Page Down** or **Alt+Left/Right arrow** previous / next frame or character, **Esc** clears the selection |
| Palette | **Arrow keys** choose a color field, **Page Up** / **Page Down** jump four rows, **Home** / **End** to the first and last entry |
| Screen | **D** Character, **T** Text, **F** Fill, **R** Frame, **I** Eyedropper, **E** Erase; **Arrow keys** move the cursor, **Enter** or **Space** sets the character (with the frame: first and second corner). In the text tool you type characters, **Esc** ends the input or cancels a frame that was started. |
| Tiles | **Arrow keys** move the cell marker, **Enter** or **Space** sets the chosen character in the cell, **I** Eyedropper. |
| Sprites and character sets (zoom) | **+** and **-** zoom, **0** Fit (also Cmd/Ctrl+mouse wheel) |

With the Cmd, Ctrl or Alt key held down, the single keys do not act. In the editors, Cmd/Ctrl+C, +X, +V and +A also apply.

## In the tracker

| Key | Effect |
|---|---|
| Arrow keys | move the cursor in the pattern (with Shift: selection) |
| Page Up / Page Down | 8 rows up / down |
| Home / End | to the first / last row |
| Space | start and stop playback |
| Comma / Period | octave lower / higher |
| Shift+Space | play only the chosen pattern |
| Cmd/Ctrl+1 / Cmd/Ctrl+2 / Cmd/Ctrl+3 | mode Notes / Instruments / Order |
| [ / ] | previous / next instrument |
| Z S X D C V G B H N J M | piano keys, lower octave (Z = C, S = C#, X = D ...); by key position, so on a German QWERTZ keyboard a Y is where the Z is |
| Q 2 W 3 E R 5 T 6 Y 7 U I | piano keys, next octave |
| Delete or Backspace | clear cell |
| Alt+Up / Alt+Down | change the instrument of the cell |
| 1 | enter the note-off character ("=== --") |
| Cmd/Ctrl+S, Cmd/Ctrl+Z | Save, Undo (with Shift: Redo) |
| Cmd/Ctrl+C / X / V / A | Copy, Cut, Paste, Select all |

The piano keys enter a note and move on one row. In the Instruments mode, Up and Down choose the instrument; piano keys and Space only play it as a preview. The tracker is in the main section **Music**; details in the chapter [The tracker](kapitel:tracker).

> **Info:** The reference [Keyboard shortcuts](hilfe:shortcuts) has a shorter table. The shortcuts shown next to the commands in the menu are the ones that count.

## Frequently asked questions

### Installation and first start

**How do I set up VintageCoder the first time?** At the very first start, the **setup assistant** opens. Choose the **Quick setup** (7 steps, about 2 minutes). You can call it up again at any time through **Help > Setup assistant ...**. More about this in [Installation and first-run setup](kapitel:installation).

**Where is the splash screen?** You switch the splash screen at program start on or off under **Settings > General** in the section **Splash screen**. Without a project, the main section **Code** shows the **start page** with new project, recently opened projects and examples; with the command **Start page** (menu **File**) you get there at any time.

**Where do I get the emulators?** VintageCoder does not ship emulators. Install VICE (C64, C128, Plus/4, VIC-20), atari800 or Altirra (Atari) and Xemu (MEGA65) yourself and enter their path under **Settings > Targets and hardware** in the section **Emulators**. If the path stays empty, the IDE searches by itself. Where there is a fixed package for your system, the emulator card offers **Install portable …**; nothing is downloaded until you confirm. More about this in the chapter [Installation and first-run setup](kapitel:installation), for Linux in the section [Emulators on Linux](kapitel:systeme).

**Where do I get ROMs?** VintageCoder does not ship original ROMs and does not download any from the network. If you need ROMs, you put your own images in the ROM folder: **Settings > ROMs** (button **Choose ROM file or zip …**, or drop the file onto the window), or **Help > Setup assistant: ROMs only ...**. VICE and atari800 run without ROM with their own replacement ROMs; **Xemu needs a MEGA65 ROM file of 131072 bytes**. The file is recognized by its content, the name does not matter. Whether you may use an image is for you to clarify.

### Build and run

**F5 or F6 does nothing.** The program was not built because there are errors: look in the tray in the tab **Problems**; a click on a message jumps to the line. If a build is already running, F5 is locked; if a debug session is already running, F6 only takes you to the area **Debug** (end it first: **Stop** in the header or Shift+F5).

**The emulator does not start.** Check the path under **Settings > Targets and hardware**; the emulator card shows "Found: ..." or "Not found", and **Test** tries out the emulator. Only **one** emulator runs at a time.

**My program starts, but nothing happens.** Build again (F7) and check the target in the **Run target** button in the header. Check that the start address is right: on the C64 a program needs `.basicstub` at `.org $0801`, on the Atari `.run start`.

**The program runs on the wrong device.** The **Run target** button in the header (system and run target) is set to a device you did not want. A click opens the list with the emulator and the devices you entered. The system itself is decided by the project (or `.target` in the source).

**What does the warning W104 mean?** The assembler reports that an instruction does not belong to the official instruction set of the CPU (an undocumented opcode such as `lax`). The example `examples/c64/border` does this on purpose. With `illegals = deny` in the `.cpu` command it becomes an error, with `illegals = warn` it stays a warning.

### Debugger and tests

**Breakpoints are ignored.** Build again; the mapping from line to address comes from the last build. With atari800 and MEGA65 there is also: only one breakpoint at a time.

**Registers and memory are empty.** The program is running. Press **Pause** (Alt+F6) or stop at a breakpoint.

**I cannot change the flags and chip registers.** They are display only, so that read accesses do not falsify anything. Change the program in the source and start again.

**All my tests are skipped.** The selection **Target** in the tab **Tests** of the tray is set to "Static checks only", or the tests need the emulator (VICE). See [Testing automatically](kapitel:tests).

### Graphics and sound

**The character set editor shows the own set of VintageCoder.** The system ROM with the font is missing; the font appears as a built-in font without graphic characters instead. Put the ROM in place as described above or import the font on the command line with `vcc media import-rom` (see [Editors at a glance](hilfe:graphics)).

**There is no sprite editor for the Plus/4.** The TED has no sprites, and for the Plus/4 there are no profiles yet for sprites, character sets or songs.

**The emulator picture is missing.** It appears in the area **Debug** in the section **Screen** and in the machine panel, only in a debug session; **Refresh picture** fetches it. Only VICE delivers a picture (C64, C128, Plus/4, VIC-20). Atari and MEGA65 deliver none.

### Files

**Where are my graphics and songs?** As text files in the project, for example `font.vcchar` or `hero.vcsprite`. To include them, you give the file itself, for example `.include "graphics/hero.vcsprite"`; when building, VintageCoder converts it.

**Which files can I delete without worry?** The folder `build/` you rebuild at any time with F7.

### Markdown

**Why does the preview not show my HTML?** Raw HTML is never run; it appears in the preview as text. Rewrite it in Markdown.

**Why does the preview not load my image from the internet?** Web images are not loaded (privacy). Put a PNG file in the project and refer to it relatively: `![Alt](pics/bild.png)`.

**Why is a link red?** The target file is missing or the path leads out of the project. Paths apply relative to the Markdown file.

**Cmd+B and Cmd+K do something different than in other editors.** They show/hide the sidebar and open the command palette. Bold therefore has no shortcut; the link is on Cmd/Ctrl+Shift+K.

## Troubleshooting: the first three steps

1. **Look in the Problems tab** (tray). There you find errors and warnings with file and line; a click jumps there.
2. **Look in the Output tab** (tray). There you see the course of the build; if a build fails without a message at a line, the reason is there. See [Build and run](kapitel:bauen-starten).
3. **Check the setup:** emulator (**Settings > Targets and hardware**) and ROM (**Settings > ROMs**).

If that does not help, narrow the problem down: build the smallest example of the target system (`examples/c64/border`, `examples/atari/colorbars`, `examples/mega65/hello`). If it builds, the problem is in your project; if it does not build, it is the setup (ROM, emulator, path).

## What next?

- Look up technical terms: Glossary (`GLOSSAR.md`).
- Get to know the interface: [The interface at a glance](kapitel:oberflaeche).
- Find an error in the running program: [The debugger](kapitel:debugger).
- For looking things up: [Keyboard shortcuts](hilfe:shortcuts), [ROM files](hilfe:roms), [Settings](hilfe:settings).

## Linux

For Linux there are packages for Debian, Ubuntu, Fedora and openSUSE as well as an archive (see [Installation and first-run setup](kapitel:installation)); X11 and Wayland are supported. All shortcuts apply unchanged with **Ctrl** instead of **Cmd**. How you install the emulators on Linux is in the section [Emulators on Linux](kapitel:systeme) in the chapter "Target systems compared".
