---
id: editor
titel: The code editor
reihenfolge: 5
stufe: Beginner
zeit: 25 minutes
siehe_auch: [assembler, basic, projekte, debugger]
referenz: [editor, shortcuts, asm-text]
i18n_status: maschinell
i18n_quelle: 4ca3b277f0ee
---

# The code editor

**Goal:** You write source code in the VintageCoder editor quickly and with help: you work with several tabs and a split view, use completion, fold long sections, find and replace text, jump to a line or a definition, set bookmarks and see errors while you type.

## What you need

- An open project with a `.asm` file. Use the program from [Your first project](kapitel:erstes-projekt) or create a new file. For the exercises you can use this small source code (it builds without errors):

```asm
; Color the border and the background
.target c64
.basicstub start

.const FARBE = 6            ; constant: 6 = blue

start:  lda #FARBE
        sta vic2.border     ; chip register as a name
        sta $d021           ; same effect, but with an address
        rts
```

## The editor at a glance

You find the editor in the main section **Code**. At the top is the **tab bar** with one tab for each open file, below it the text with the line numbers. On the left is the sidebar with the project tree, below the text the **tray** with the pages Problems, Output, Tests, Search and Outline. The status bar at the bottom names the kind of file (Assembler, BASIC, Text or Markdown), the line ending (LF or CRLF), the state of the problems and the line and column of the cursor. With a running debug session, the program counter is shown there too.

![The Code area with the file jumprun.asm: the sidebar with the project tree on the left, and in the middle the assembler source with line numbers and colored highlighting for comments, instructions, numbers, and names. The status bar at the bottom shows line and column.](bild:editor-syntax "The editor colors instructions, numbers, names, and comments differently.")

The editor colors **assembler** and **BASIC**. Commands, directives (the instructions to the assembler with a dot in front, like `.target`), numbers, strings, labels and comments have different colors. This way you see at a glance whether a word counts as what you meant. Text stays uncolored; you see Markdown files as a preview.

Next to the line numbers is a narrow strip, the **gutter**. There you see breakpoints, bookmarks, error marks and the arrows for folding. If you have built the program, the gutter also shows the **cycle count** for each line (how many clock cycles the command needs).

## Working with tabs

Every open file has a tab. A dot on the tab means: the file has unsaved changes.

- **Open:** Click a file in the project tree, or choose **File > Open file ...** (Cmd/Ctrl+O). A file that is not text is opened by VintageCoder in the default program of your system.
- **New tab without a file:** **New tab (untitled)** with Cmd/Ctrl+N. When you save, VintageCoder asks for a name. **New note** (Cmd/Ctrl+Shift+T) creates a tab for Markdown text.
- **Switch:** With Cmd/Ctrl+Alt+Right and Cmd/Ctrl+Alt+Left you jump to the next and the previous tab. You can also switch with the arrow keys when the tab bar has the focus.
- **Arrange:** Drag a tab with the mouse to another place, or use the commands **Move tab left** and **Move tab right** in the menu **View > Tabs and splitting**.
- **Close:** Cmd/Ctrl+W, the small x, the middle click or the Del key on the tab. If the file has unsaved changes, VintageCoder asks whether to save or discard.
- **Save:** Cmd/Ctrl+S saves the file in the tab, Cmd/Ctrl+Alt+S saves all.
- **Right click on a tab** opens a menu with Close, Close others, Close all, To the left, To the right, Copy path and Show in folder.

If another program changes an open file, VintageCoder reads it again and tells you. If you have unsaved changes in the file itself, you get a warning instead: saving would overwrite the file. VintageCoder remembers the open tabs for each project and restores them the next time you open it.

If you want to write and see nothing else, switch on **Zen mode** with Cmd/Ctrl+Alt+Z: tabs and text stay. The same command brings everything back.

## Step by step: Write with completion

While you type, the editor suggests words: commands, directives, chip registers (for example `vic2.border`), labels, constants and macros, also from included files. This applies to assembler files.

1. Put the cursor in a new line and type `st`. After a short moment a list of suggestions appears. (In `vic2.` the dot counts: the list then shows the registers of the chip right away.)
2. Choose an entry with the arrow keys **Up/Down**.
3. **Tab** or **Enter** accepts the marked entry. **Esc** closes the list without inserting anything.
4. Without typing, you open the list with the command **Open completion** (menu **Edit**, command palette). It deliberately has no shortcut.

![The code editor with the file list on the left and the source jumprun.asm. The last line contains 'lda' and a suggestion list is open below it with lda (load accumulator), constants such as TILE_CYC_LDA and macros such as tile_draw.](bild:editor-vervollstaendigung "The suggestion list appears while you type.")

> **Tip:** Try `sta vic2.` - after the dot the list shows all registers of the VIC-II, the graphics chip of the C64, for example `border` (border color), `bg0` (background color) and `raster` (current raster line).

If you type a `{` in a string, the editor offers the names of the control characters and colors, for example `{clr}` or `{red}`. The notation `{name}` in texts is described on the help page [Text and character sets](hilfe:asm-text). A panel of its own for inserting special characters does not exist in this version yet.

## Hints on hover

If you let the mouse rest briefly over a word, a card with a description appears: for a command the addressing modes with opcode, bytes and cycles, for a directive the syntax, for a chip register the address and the meaning. Over a number, the card shows the values in other notations (decimal, hexadecimal, binary); in BASIC, for addresses it also shows what is at this address in the computer. In strings it shows the code and effect of a control character.

With **F1** (command **Help**) you look up the word under the cursor, or the marked text. The help opens with the matching topic. If there is no entry for the word, it says "There is no help entry for ..." and you can search in the help. Without a word, F1 opens the help with the list of topics.

## Step by step: Fold sections

Long source codes become hard to read. With **folding** you hide sections.

1. Next to the line number, foldable blocks show small arrows. Blocks like `.macro`, `.if`, `.repeat`, label sections and comment blocks can be folded.
2. Click the arrow next to the start of the block to fold the section. Its lines are then hidden, and a placeholder is in its place. A click on the arrow or the placeholder unfolds it again.
3. **Fold all** and **Unfold all** (menu **Edit > Folding**, command palette and right click in the editor) fold or unfold all sections at once; they have no shortcut.

*[Bild folgt in E13: editor-falten]*

## Step by step: Find and replace

1. Press **Cmd/Ctrl+F** (menu **Edit > Find and replace > Find ...**). The search bar appears above the text.
2. Type the search term. The matches are highlighted in the text. **Enter** jumps to the next match, **Shift+Enter** to the previous one; there are also the buttons **next** and **previous** for this. **Esc** closes the bar.
3. With the switches **match case**, **by word** and **regexp** you narrow the search.
4. To **replace**, press **Cmd/Ctrl+Alt+F** (menu **Edit > Find and replace > Replace ...**). The bar has the second field **Replace with**. **replace** changes the current match, **replace all** changes all matches.
5. If you made a wrong click, **Cmd/Ctrl+Z** (Undo) helps; **Cmd/Ctrl+Shift+Z** redoes.

![The code view with jumprun.asm and the search bar open above the text. The search field contains jsr, next to it are the buttons Weiter, Zurueck and Alle and the options Gross/Klein, Regulaerer Ausdruck and Ganzes Wort. Below is the field Ersetzen durch with the buttons Ersetzen and Alle ersetzen.](bild:editor-suchen "The search bar with replace, above the source text.")

Do you want to search in **all** files of the project? Press **Cmd/Ctrl+Shift+F**. In the tray, the page **Search** opens ("Search all files …"). The matches appear grouped by file, a click jumps to the line in the editor.

## Step by step: Jump

An error message often names a line number, for example "Line 42".

1. Press **Cmd/Ctrl+L** (menu **Edit > Find and replace > Go to line ...**). A small dialog asks for the line number.
2. Type the number and confirm with Enter (**Go to**). The cursor jumps there.

You jump to a **definition** with **F12** (right click > **Go to definition**) when the cursor is on a symbol, for example a label or a constant; the definition may be in another file. This works only for assembler files. You open an included file in the project tree.

The tray also has the page **Outline**: it lists the labels, constants and macros of the assembler file in the tab, a click jumps to the place. The command **Jump to matching bracket** (menu **Edit > Find and replace**) has no shortcut.

## Bookmarks and breakpoints

You set both in the gutter:

- **Breakpoint:** A click in the gutter sets a red dot, a second click removes it. **Toggle breakpoint** (F9) does the same. The breakpoint takes effect when you start the program with the debugger (see [The debugger](kapitel:debugger)).
- **Bookmark:** A click with the Shift key held down sets a mark, or you press **Cmd/Ctrl+Alt+K** (Toggle bookmark). **Cmd/Ctrl+Alt+J** jumps to the next, **Cmd/Ctrl+Alt+H** to the previous bookmark, around in a circle. The commands are in the menu **Edit > Find and replace**.

Breakpoints and bookmarks move with the text when you insert or delete lines before them. If you close the tab, they are lost.

## Formatting and tidying up

When you leave an **edited** line (the cursor moves to another line, Enter, mouse click), VintageCoder aligns it. In the assembler, a label is in column 0, the command after it, and comments sit in one column. In BASIC there is a space after the line number. If you do not like an alignment, you undo it with **Cmd/Ctrl+Z** like any other input. You can turn off the alignment under **Settings > Editor** with the switch **Format a line when leaving it**.

You can also format by hand on command. In the menu **Edit > Formatting** (and in the command palette) are:

- **Format line** (the line or the marked lines) and **Format file** (the whole file). Both can be undone in one step.
- **Remove trailing whitespace** (selection or whole file).
- **Sort lines** (mark several lines).
- **Toggle comment** (Cmd/Ctrl+Alt+C) sets or removes the comment character in front of the marked lines.

Formatting works only in assembler and BASIC tabs; otherwise VintageCoder reports that the tab cannot be formatted.

## Type, line ending and wrap

The file extension decides which language a tab has. If you want to change it (for example for an untitled tab), choose **Edit > Type and encoding > Type: Text**, **Type: Assembler** or **Type: BASIC**. In the same submenu, **Switch line ending (LF/CRLF)** switches the line ending of the file; the status bar shows which one applies. The command **Switch encoding (UTF-8/ISO-8859-1)** does not exist in this version yet.

The editor wraps long lines (default: on). With **View > Word wrap in the editor** you switch it off and on again. You change the font size with **Editor font larger** and **Editor font smaller** in the menu **View > Zoom and font**.

## You see errors immediately

You do not have to build to find errors. After every change, VintageCoder checks the assembler source code in the background. If a line has an error, the place is underlined with a **wavy line** and marked in the gutter. If you move the mouse over it, the card shows the message with the error code. The tray shows all messages on the page **Problems** (see [Build and run](kapitel:bauen-starten)), the status bar counts errors, warnings and hints. With **F8** you jump to the next problem in the text, with **Shift+F8** to the previous one.

Try it: change `sta vic2.border` to `sta vic2.bordr`. The place is underlined, and the card reports an unknown symbol. Restore the old text with Cmd/Ctrl+Z.

![The code view with jumprun.asm scrolled to line 269. Lines 267 and 268 contain lda #$zz and xyz 5, both underlined in red and marked with a red dot in the margin. The status bar at the bottom shows 2 Fehler (errors) and 2 messages in other files.](bild:editor-fehlerzeile "Faulty lines are underlined in red and marked in the margin, and the status bar counts the errors.")

For some messages there is a suggestion for a correction. Put the cursor in the line and press **Cmd/Ctrl+Period** (**Quick fix …**): a small menu offers the corrections, **Enter** applies one. If there is none, VintageCoder says so.

> **Info:** The language services (error check, completion, definition, outline, quick fix) work for assembler files. In BASIC and text tabs you get coloring and formatting, in BASIC also the card on hover.

## The context menu

A **right click** in the editor puts the cursor at the click position and opens a menu with:

- **Cut**, **Copy**, **Paste**, **Select all**,
- **Go to definition** (F12) and **Quick fix …** (Cmd/Ctrl+Period),
- **Toggle comment** (Cmd/Ctrl+Alt+C) and **Go to line ...** (Cmd/Ctrl+L),
- **Toggle bookmark** and **Toggle breakpoint** (F9),
- **Fold all** and **Unfold all**,
- in a BASIC file also the three conversions **BASIC: Convert to labels …**, **BASIC: Convert to line numbers …** and **BASIC: Renumber …** (see [BASIC in the IDE](kapitel:basic)).

![The code editor with the BASIC file sterne.bas and an open context menu at line 4. The entries are Ausschneiden and Kopieren (grayed out), Einfuegen (highlighted), Alles auswaehlen, Gehe zu Definition, Korrigieren (Quick-Fix) (grayed out), Kommentar umschalten, Gehe zu Zeile, Lesezeichen umschalten, Alles falten, Alles entfalten, Haltepunkt umschalten and the three BASIC entries for converting and renumbering.](bild:editor-kontextmenue "The editor context menu in a BASIC file with the three BASIC entries at the end.")

## The second editor area

Sometimes you want to see two files side by side, for example a source code and its library.

1. Open at least two tabs. With only one tab, the editor cannot be split; VintageCoder tells you.
2. Press **Cmd/Ctrl+Alt+T** (menu **View > Tabs and splitting > Split editor / remove split**). The editor area splits into two halves, each with its own tab bar. You can also drag a tab with the mouse to the right edge ("Drop on the right (splits)").
3. Send a tab to the other half: menu **View > Tabs and splitting > Send tab to the other half**, or drag the tab over.
4. The same command **Split editor / remove split** removes the split again. You can split the editor only once.

*[Bild folgt in E13: editor-zweiter-bereich]*

If the window is narrow, the two halves are one below the other instead of side by side.

## Including files

There are two ways to include a file in the source code. Choose the file in the project tree, open the assembler source, and choose **Insert as include** in the context menu of the file: VintageCoder writes the line with the path relative to the source file before the line with the cursor (Cmd/Ctrl+Z takes it back; if the file is already included, nothing is added twice; project files and disk images cannot be included). Or you write the line yourself. Dragging from the project tree into the source code does not exist yet. The usual lines (paths relative to the source file, with `/`):

| Line | Effect |
|---|---|
| `.include "pfad/datei.asm"` | inserts the assembler source code at this place, anew on every call |
| `#include "pfad/datei.asm"` | inserts it **only once** (for libraries) |
| `.incbin "pfad/datei.bin"` | includes the file unchanged as bytes |
| `.include "graphics/hero.vcsprite"` | includes a sprite, a character set, a screen or a song from the project; the assembler converts the media file itself |

In BASIC, the line for source code and media is `#include "pfad"`; `.incbin` does not exist there. How including media works exactly is described in the chapter [Assembler (VCASM) for beginners](kapitel:assembler), and the lines of the directives on the help page [Assembler: directives](hilfe:asm-directives).

> **Info:** Something different is dragging files from Finder or Explorer **onto the window**; that adds files to the project (see [Projects and files](kapitel:projekte)).

## Markdown files and notes

Files with the extension `.md` (or `.markdown`) are shown by the editor as a **preview**: headings, lists, tables and code, for reading only. **Markdown: Switch between preview and source** (menu **View** and command palette) switches to the source, in which you write. You create notes for yourself (for example a `README.md`) with **New note** or **New note ...**. The commands in the menu **Edit > Markdown** help you write (bold, italic, code, link, lists, tasks, quote, table). The shortcuts are Cmd/Ctrl+I (italic), Cmd/Ctrl+E (code), Cmd/Ctrl+Shift+X (strikethrough) and Cmd/Ctrl+Shift+K (link). You can find more on the help page [Markdown](hilfe:markdown). An outline of the headings does not exist.

## Summary of the shortcuts

| Action | Mac | Windows and Linux |
|---|---|---|
| Save | Cmd+S | Ctrl+S |
| Save all | Cmd+Alt+S | Ctrl+Alt+S |
| New tab | Cmd+N | Ctrl+N |
| Close file | Cmd+W | Ctrl+W |
| Next / previous tab | Cmd+Alt+Right / Left | Ctrl+Alt+Right / Left |
| Undo / Redo | Cmd+Z / Cmd+Shift+Z | Ctrl+Z / Ctrl+Shift+Z |
| Find / Replace | Cmd+F / Cmd+Alt+F | Ctrl+F / Ctrl+Alt+F |
| Search in project | Cmd+Shift+F | Ctrl+Shift+F |
| Go to line | Cmd+L | Ctrl+L |
| Go to definition | F12 | F12 |
| Quick fix | Cmd+Period | Ctrl+Period |
| Toggle comment | Cmd+Alt+C | Ctrl+Alt+C |
| Toggle bookmark / next / previous | Cmd+Alt+K / J / H | Ctrl+Alt+K / J / H |
| Toggle breakpoint | F9 | F9 |
| Next / previous problem | F8 / Shift+F8 | F8 / Shift+F8 |
| Split editor | Cmd+Alt+T | Ctrl+Alt+T |
| Zen mode | Cmd+Alt+Z | Ctrl+Alt+Z |

A complete table is on the help page [Keyboard shortcuts](hilfe:shortcuts); the details about the editor are on the help page [Editor](hilfe:editor).

## Common stumbling blocks

- **Completion does not appear.** It works in assembler files and opens while you type. Keep typing, or open it with the command **Open completion**.
- **Enter inserts a suggestion although I wanted a new line.** As long as the list is open, Enter accepts the entry. Press **Esc** first.
- **The line changed after I left it.** That is the alignment. It is intended; you undo it with Cmd/Ctrl+Z.
- **"This tab has no language services."** Definition and quick fix belong to assembler files. Check the type in the status bar.
- **I "lost" a folded section.** Click the placeholder or the arrow next to the line below which the lines are missing.
- **The editor cannot be split.** Open a second tab first.
- **F1 shows "There is no help entry for ...".** There is no entry for this word. Search the help for a related term; the card on hover often describes the word already.

## What next?

Now you write source code comfortably. You learn what the individual lines of an assembler program mean in the next chapter: [Assembler (VCASM) for beginners](kapitel:assembler). If you prefer to start with BASIC, continue with [BASIC in the IDE](kapitel:basic).
