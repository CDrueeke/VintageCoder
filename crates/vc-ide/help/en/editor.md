# Editor

The editor highlights assembler and BASIC in color. You find it in the main section **Code**: the project tree on the left, the files as tabs in the middle, and below it the tray with problems, output, search and outline (see [Customize the layout](help:layout)).

## Completion
While you type, a list appears as soon as the word is two characters long (a dot counts, for example in `vic2.`). It shows commands, directives, chip registers (`vic2.border`), labels, constants and macros: also from included files. **Tab** accepts the entry, **Up/Down** and **Enter** choose another one, **Esc** closes the list.

## Description for the word
The mouse pointer over a word shows the description: for a command all addressing modes with opcode, bytes and cycles, for directives the syntax, for registers the address and meaning. **F1** looks up the word under the cursor (or the selected text) in the help and opens the matching topic.

## Markdown files
Files with the extension `.md` or `.markdown` (notes, `README.md`) are shown in the editor as a **preview** (read only). **Markdown: Switch between preview and source** switches to the source text. Writing aids (bold, italic, code, link, lists, tasks, quote, tables) are in the menu **Edit > Markdown**, see [Markdown](help:markdown). You create new notes with the context menu of the project tree (**New note**) or with Cmd/Ctrl+Shift+T (an untitled note).

## Folding
Arrows next to the line numbers fold blocks (`.macro`, `.if`, `.repeat`, `while`, `for` ...); a click folds or unfolds. **Fold all** and **Unfold all** fold or unfold all sections at once (command palette and right click, no shortcut).

## More controls
- A click on the narrow column next to the line numbers sets a breakpoint; **F9** toggles it on the cursor line. The breakpoint works while debugging, see [Breakpoints](help:breakpoints).
- **F12** (Edit > Find and replace > Go to definition) jumps to the definition of the symbol under the cursor, also into other files (assembler only).
- **Cmd/Ctrl+.** offers fixes for a message.
- You set **bookmarks** with Cmd/Ctrl+Alt+K; Cmd/Ctrl+Alt+J jumps to the next one, Cmd/Ctrl+Alt+H to the previous one.
- **Toggle comment:** Cmd/Ctrl+Alt+C.
- **Cmd/Ctrl+S** saves the file, **Cmd/Ctrl+Alt+S** all changed files.

## Dragging files into the project
Drag files from Finder or Explorer onto the window: an area "Drop files here to open them" appears. Text files from the project open right away; if a file is outside the project, VintageCoder asks "Copy into the project and open?" (**Copy and open**). A project file (`.vcprj`) opens the project, a disk image opens in the [Disk editor](help:disks). Without an open project, files cannot be dropped.

## Format line when leaving it
If you leave an **edited** line (cursor to another line, Enter, mouse click), it is aligned. This applies to assembler and BASIC. You undo it like any input with Cmd/Ctrl+Z. You can turn it off under **Settings > Editor** (**Format a line when leaving it**). On command (also when formatting on leaving is off): **Edit > Formatting > Format line** and **Format file** (the whole file as one undo step). With a selection, with an open completion and in the editors, nothing is ever formatted automatically.

**Assembler**
- A label (`name:`) is in column 0, followed by the mnemonic in column 8 (for long labels after one space); commands without a label are indented to column 8.
- The mnemonic and known directives are written in lowercase (`LDA` becomes `lda`, `.ORG` becomes `.org`); macro calls and other names stay unchanged.
- Between the mnemonic and the operand there is one space, multiple spaces are merged (strings and characters like `' '` stay).
- A comment after the code starts in column 40 (if the code is longer, after one space). Pure comment lines and empty lines as well as `{` and `}` only change the right margin. Directives without a label and assignments keep their indentation.

**BASIC (`.bas`, `.lst`)**
- One space after the line number, multiple spaces outside strings merged; the text after `REM` stays unchanged. No indentation.

## Character encoding and line endings

Files are always saved as **UTF-8 without BOM** with **LF**. When you open files from elsewhere (BOM, CRLF, Windows-1252), a note appears in the status bar; the file is only rewritten when you save. Files with null bytes are treated as binary files and not shown.

## Context menu (right mouse button)
**In the editor**, a right click puts the cursor at the click position (except inside a selection) and offers: Cut, Copy, Paste, Select all, Go to definition, Quick fix, Toggle comment, Go to line, Toggle bookmark, Fold all and Unfold all, and Toggle breakpoint; in BASIC files also the switch between line numbers and labels.
**In the project tree** and on a **tab** there are menus of their own, see [File tree](help:filetree). The tab menu offers Close, Close others, Close all, Move left and right, and Copy path. **Cmd/Ctrl+Alt+T** splits the editor into two halves.

## Find, replace, go to line
**Edit > Find and replace > Find ...** (Cmd/Ctrl+F) opens a bar above the text: Enter jumps to the next match, Shift+Enter to the previous one, Esc closes. **Replace ...** (Cmd/Ctrl+Alt+F) adds a Replace field. **Go to line ...** (Cmd/Ctrl+L) jumps to a line number. **Find in project ...** (Cmd/Ctrl+Shift+F) searches all files; the hits appear in the tab **Search** of the tray. **Undo** and **Redo** (Cmd/Ctrl+Z, Cmd/Ctrl+Shift+Z) are also in the menu. **File > Start page** shows the recently opened projects.
