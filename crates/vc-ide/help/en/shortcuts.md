# Keyboard shortcuts

The list follows the command table of the user interface: What is listed here is also shown in the menu next to the command and in the command palette (**Cmd/Ctrl+K**). The shortcuts are fixed. You cannot customize them yet.

**Cmd** is the command key of the Mac. On Windows and Linux, use **Ctrl**. **Shift** is the Shift key, **Alt** is the Option key on the Mac.

Some commands do not exist in this version yet; the menus and the command palette do not show them: zoom, grid and Onion Skin of the editors through the menu (the buttons and keys in the editor exist), Convert BASIC to assembler, Switch encoding, BASIC: Hex suggestions and the character set imports (standard set, ROM, PNG grid). They are not listed here.

## Build, run, tests
| Key | Effect |
|---|---|
| F7 | Build project |
| F5 | Run start program (builds first) |
| F6 | Debug start program (builds first, opens the area **Debug**) |
| Alt+Shift+F7 | Build active file |
| Alt+Shift+F5 | Run active file |
| Alt+Shift+F6 | Debug active file |
| Shift+F5 | Stop the running emulator or the debug session |
| Alt+F8 | Run tests |
| F8 / Shift+F8 | Next / previous problem |
| Cmd/Ctrl+Shift+E | Export ... |

You set the start program in the area **Project** on the page **Sources**.

## Debug
The commands are in the menu **Project > Debug**. The steps work when a debug session is running and the program is paused (see [Debugger](help:debugger)).

| Key | Effect |
|---|---|
| Alt+F5 | Continue (to the next breakpoint); without a session it starts one |
| Alt+F6 | Pause |
| F10 | Step over (a `JSR` runs through) |
| F11 | Step into (a `JSR` is entered) |
| Shift+F11 | Step out (to the return of the subroutine) |
| Shift+F5 | Stop debugging |
| F9 | Toggle breakpoint on the line with the cursor |

**Run to here** (to the cursor line) has no shortcut.

## Files and projects
| Key | Effect |
|---|---|
| Cmd/Ctrl+Alt+N | New project ... |
| Cmd/Ctrl+Shift+O | Open project ... (project file) |
| Cmd/Ctrl+Shift+W | Close project |
| Cmd/Ctrl+Alt+0 | Project area |
| Cmd/Ctrl+O | Open file ... |
| Cmd/Ctrl+N | New tab (untitled) |
| Cmd/Ctrl+Shift+T | New note |
| Cmd/Ctrl+S | Save (also in the sprite editor, character set editor and palette editor) |
| Cmd/Ctrl+Alt+S | Save all |
| Cmd/Ctrl+W | Close file |
| Cmd/Ctrl+Q | Quit VintageCoder |

## Edit and search
| Key | Effect |
|---|---|
| Cmd/Ctrl+Z / Cmd/Ctrl+Shift+Z | Undo / Redo (source code, sprite editor, character set editor and palette editor) |
| Cmd/Ctrl+F | Find ... |
| Cmd/Ctrl+Alt+F | Replace ... |
| Cmd/Ctrl+L | Go to line ... |
| Cmd/Ctrl+Shift+F | Find in project ... (opens the tab **Search** in the tray below the editor) |
| F12 | Go to definition |
| Cmd/Ctrl+. | Quick fix |
| Cmd/Ctrl+Alt+C | Toggle comment |
| Cmd/Ctrl+Alt+K | Toggle bookmark |
| Cmd/Ctrl+Alt+J / Cmd/Ctrl+Alt+H | Next / previous bookmark |
| F9 | Toggle breakpoint on the line with the cursor |

Suggestions appear by themselves while you type (after typing, after "." and after "@"). There is no separate key to open them, but there is the command **Open completion** in the command palette. The Markdown commands in the Edit menu have the shortcuts Cmd/Ctrl+I (Italic), Cmd/Ctrl+E (Code), Cmd/Ctrl+Shift+X (Strikethrough) and Cmd/Ctrl+Shift+K (Insert link). **Bold** has no shortcut.

## View and user interface
| Key | Effect |
|---|---|
| Cmd/Ctrl+K | Search for a command or file ... (command palette) |
| Cmd/Ctrl+B | Show / hide sidebar |
| Cmd/Ctrl+J | Show / hide tray below the editor |
| Cmd/Ctrl+Alt+B | Show / hide machine panel |
| Cmd/Ctrl+Alt+Z | Zen mode: only the header, the second row and the work area (press again: back) |
| Cmd/Ctrl+Shift+Enter | Maximize / restore the area with focus |
| Cmd/Ctrl+Alt+T | Split editor / remove split |
| Cmd/Ctrl+Alt+Right / Cmd/Ctrl+Alt+Left | Next / previous tab |
| Cmd/Ctrl+Plus / Cmd/Ctrl+Minus / Cmd/Ctrl+0 | Make UI larger / smaller / original size |
| Cmd/Ctrl+Comma | Settings |
| F1 | Help for the word under the cursor (opens the help) |

Without a shortcut, but in the menu **View**: Switch light / dark, High contrast, Reset layout, Editor font larger and smaller, Reset appearance, Word wrap and the commands to maximize single areas.

## Single keys in the editors
As soon as the canvas of an editor has the focus and no text field is active, single keys (letters, digits, plus and minus) apply. They do not work when **Cmd**, **Ctrl** or **Alt** is held down.

| Editor | Keys |
|---|---|
| Sprites and character sets | D pen, E eraser, I eyedropper, F fill, L line, R rectangle, S selection, X / Y mirror horizontally / vertically, 1 to 4 pixel value, G grid, O Onion Skin (sprites only), arrow keys move the cell marker (Enter or Space sets a dot, Del clears it), Page Up / Page Down or Alt+Left/Right previous / next frame or character, Esc clears the selection, + / - / 0 zoom (also Cmd/Ctrl and mouse wheel) |
| Palette | Arrow keys choose a color field, Page Up / Page Down jump four rows, Home / End to the first and last entry |

In the editors, Cmd/Ctrl+C, +X, +V and +A also apply. The single keys of the other editors are listed below. You can turn the single keys off under **Settings > Shortcuts** (switch **Single keys in the editors**).

| Editor | Keys |
|---|---|
| Screen | D character, T text, F fill, R frame, I eyedropper, E erase; arrow keys move the cursor, Enter or Space sets the character, Esc finishes the text input or cancels a frame |
| Tiles | Arrow keys move the cell marker, Enter or Space sets the chosen character, I eyedropper |

## Tracker
The keys apply when the pattern grid has the focus.

| Key | Effect |
|---|---|
| Arrow keys | Move the cursor (with Shift: selection) |
| Page Up / Page Down | 8 rows up / down |
| Home / End | First / last row |
| Space / Shift+Space | Play and stop the whole song / only the pattern |
| Comma / Period | One octave lower / higher |
| [ / ] | Previous / next instrument |
| Cmd/Ctrl+1 / Cmd/Ctrl+2 / Cmd/Ctrl+3 | Mode Notes / Instruments / Order |
| Piano keys Z S X D C V G B H N J M and Q 2 W 3 E R 5 T 6 Y 7 U I | Enter a note (by key position) |
| 1 | Enter the note-off sign |
| Del or Backspace | Clear the cell |
| Alt+Up / Alt+Down | Change the instrument of the cell |

## Mouse
| Action | Effect |
|---|---|
| Click on the line number | Set or delete a breakpoint |
| Click on the arrow in the margin | Fold or unfold a section |
