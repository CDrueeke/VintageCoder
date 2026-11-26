---
id: oberflaeche
titel: The user interface at a glance
reihenfolge: 3
stufe: Beginner
zeit: 20 minutes
siehe_auch: [erstes-projekt, projekte, editor]
referenz: [layout, settings, shortcuts, overview]
i18n_status: maschinell
i18n_quelle: 8003da3a6f41
---

# The user interface at a glance

**Goal:** You know the layout of the VintageCoder window and what the main sections, the sidebar and the tray below the editor are for. You can show and hide areas, make one area large, use **Zen mode**, open the command palette and change the color scheme and the language.

## What you need

- An open project, for example the one from the chapter [Your first project](kapitel:erstes-projekt). If no project is open, the main section **Code** shows the **start page** (new project, recently opened projects, examples). You can bring it back at any time with the command **Start page** (menu **File**); this closes the project.

## The frame of the window

At the top are the **header bar** and, where there is one, a **second row** below it. Below that is the work area, and at the very bottom the **status bar**. You switch the main sections in the header bar; what the work area shows depends on the main section.

![The main window in the area 'Code'. At the top is the header with the project name and the main areas from 'Startseite' to 'Debug'. On the left is the sidebar 'PROJEKT' with the file tree, in the middle the editor with jumprun.asm, below it the drawer with the tab 'Ausgabe', on the right the panel 'Maschine', and at the bottom the status bar.](bild:ui-gesamt "The main window at a glance: header, sidebar, editor, drawer, machine panel and status bar.")

| Zone | What it is for |
|---|---|
| **Menu bar** | File, Edit, View, Project, Help. On the Mac it is in the menu bar of the system. Long menus (Edit, View, Project) have submenus. Entries that are not possible are gray (for example without an open project). |
| **Header bar** | button for the sidebar, project name with path, the **main sections**, the buttons **Build**, **Run** and (as long as a program is running) **Stop**, the **Target** button, the menus for **Language** and **Color scheme** and the gear for the settings and the search icon for the command palette |
| **Second row** | subsections of the chosen main section (for **Project**, **Graphics** and **Data**) |
| **Sidebar** (left, only in the main section **Code**) | the file tree of the project |
| **Editor** (middle) | the open files as **tabs** |
| **Tray** (below the editor) | **Problems**, **Output**, **Tests**, **Search**, **Outline** |
| **Machine panel** (right, only in the main section **Code**) | target, state and result of the last build; in a debug session also registers, flags and the picture of the emulator |
| **Status bar** | project name, kind of file and line ending, number of problems, line and column, state of the build, with a running debug session the program counter, with a new version the update note, and the note about the command palette |

## The main sections

In the header bar the main sections stand next to each other. A click switches the section; the chosen one is highlighted.

| Main section | What it is for |
|---|---|
| **Project** | Settings and overview for the whole project. Second row: **Overview**, **General**, **Targets**, **Sources**, **Build and run**, **Disk**, **Share**. More in the chapter [Projects and files](kapitel:projekte). Shortcut: **Cmd/Ctrl+Alt+0** |
| **Code** | Write source code (sidebar, editor, tray, machine panel) and, without a project, the start page. More in the chapter [The code editor](kapitel:editor) |
| **Graphics** | Second row: **Sprites**, **Character set**, **Tiles**, **Screen**, **Maps**, **Palette**. This is where the Sprite editor, the Character set editor, the Tile editor, the Screen editor, the Map editor and the Palette editor work. More in the chapter [Graphics and sound: overview](kapitel:grafik-ueberblick) |
| **Music** | the tracker for songs (files `.vcsong` and `.vcplay`). More in the chapter [The tracker](kapitel:tracker) |
| **Data** | Second row: **Disk**, **Hex**, **Memory**. **Disk** is the Disk editor (see [Disk editor, segments, ROMs](kapitel:disk-studio)), **Hex** shows binary files of the project for viewing only, **Memory** is the graphical memory map of the last build |
| **Debug** | the debug session with breakpoints, single step, registers, memory and the picture of the emulator. More in the chapter [The debugger](kapitel:debugger) |

A section or subsection can be missing if the build of VintageCoder that you use does not contain the matching editor.

## The header bar

From left to right: the button for the **sidebar** (the tooltip shows the shortcut), the **project name** with the shortened path below it, the **main sections**, then, with an open project, the buttons **Build** (F7) and **Run** (F5). If a program runs in the emulator, the button **Stop** is added. The **Target** button shows the target system and the name of the run target; a click opens a list with the emulator and the machines you entered, and a check mark is at the current run target. Then follow two menus with an icon, **Language** (Automatic, Deutsch, English) and **Color scheme** (System, Light, Dark), then the gear for the **Settings** (**Cmd/Ctrl+Comma**) and last the search icon for the command palette (**Cmd/Ctrl+K**).

There is no Save button of its own in the window: **Cmd/Ctrl+S** saves. Before every build, VintageCoder saves anyway.

## The sidebar

In the main section **Code**, the sidebar shows the **file tree** of the project (heading "Project"). At the top of it are the buttons **Filter files**, **New file**, **New folder** and **Close project**. The arrow keys choose entries, the **Right arrow** and **Left arrow** keys open and close folders, **Enter** opens, **F2** renames, **Del** deletes. Everything else about files is in the chapter [Projects and files](kapitel:projekte).

**Cmd/Ctrl+B** folds the sidebar in and out. If it is folded in, a narrow strip with icons (files, search) stays, through which you open it again.

## The editor

In the editor, files open as **tabs**. **Cmd/Ctrl+W** closes a tab (if there are unsaved changes, VintageCoder asks), **Cmd/Ctrl+N** opens an **untitled tab** in which you can write at once. **Cmd/Ctrl+Shift+T** opens a **note**; VintageCoder saves untitled tabs and notes locally every few seconds. With **Cmd/Ctrl+Alt+T** you split the editor into two halves and can send tabs to the other half. **Cmd/Ctrl+Alt+Right** and **Cmd/Ctrl+Alt+Left** switch to the next and the previous tab.

The editors for graphics, music and disks are main sections of their own (**Graphics**, **Music**, **Data**). You open their files there, not in the main section **Code**.

## The tray below the editor

The **tray** is below the editor and has five tabs:

- **Problems:** errors and warnings with file and line (a click jumps to the place).
- **Output:** what happened during building and running, with the buttons **Copy** and **Clear**.
- **Tests:** the test results (see [Tests](kapitel:tests)).
- **Search:** searches text in all text files of the project (**Cmd/Ctrl+Shift+F** opens it).
- **Outline:** the labels, constants and macros of the file in the tab; a click jumps there.

![The Code tab with the file jumprun.asm and the tray below the editor open, Probleme (Problems) tab active (next to Output, Tests, Search, Outline). It shows 2 messages in 2 files, 0 of them errors, and two hints for lib/core.asm and lib/music.inc about automatically extended branches.](bild:probleme-panel "The tray below the editor with the Problems tab (only hints here, no errors).")

The tray opens by itself when there is something to show (build errors, output). **Cmd/Ctrl+J** folds it in and out; with **x** at the top right you close it. You switch between the tabs with the arrow keys.

## The machine panel

On request, the **Machine** panel is on the right. It shows the system and the run target, the state (ready, building, running) and the result of the last build (source, program, size, start address and the segments), and the buttons **Build**, **Run** and **Stop**. During a debug session it also shows the registers, the flags and, where the emulator supplies it, the picture of the machine. **Cmd/Ctrl+Alt+B** shows and hides the panel.

## Step by step: Show and hide areas

1. **Cmd/Ctrl+B**: sidebar. *Expected:* The file tree disappears, the editor becomes wider.
2. **Cmd/Ctrl+J**: tray. *Expected:* It folds out or in below the editor.
3. **Cmd/Ctrl+Alt+B**: machine panel. *Expected:* It appears on the right or disappears.
4. **Zen mode (Cmd/Ctrl+Alt+Z):** hides the sidebar, the tray, the panel and the status bar. *Expected:* The header bar and the editor stay. The same command brings everything back.
5. You change the sizes by dragging the edge between two areas with the mouse. It also works with the arrow keys (Shift for large steps); a double click on the edge restores the default.

## Make an area large

The sidebar, the tray and the panel have a button **Maximize area** at the top right. Maximize folds the other areas in; **Restore area** (or a double click on the header of the tray) brings back the previous layout. With the keyboard: **Cmd/Ctrl+Shift+Enter** toggles the area that has the focus. The editor can only be maximized, never folded in.

> **Tip:** If you clicked in the wrong place and nothing looks right: **View > Areas > Reset layout** restores the default sizes.

All these commands are in the **View** menu and in the command palette.

## Narrow windows

The window may become narrow. If the editor is narrower than about 640 points, the sidebar and the panel float over the editor; a click into the editor folds them in again. If space is narrow, the two halves of the split editor are one below the other instead of side by side.

## The command palette

The **command palette** is the fastest way to find something: a command, a file, a place in the navigation or a recently opened project.

1. Press **Cmd/Ctrl+K** or click the search icon in the header bar. A window with an input field opens.
2. Type a few letters, for example `build`. All words must occur in the name. The list is sorted into groups: **Commands**, **Files**, **Projects** and **Go to** (the main sections and subsections).
3. Choose with the arrow keys and press Enter, or click an entry. Esc closes the window.

![The command palette open over the code editor. A search field at the top reads 'Befehl oder Datei suchen ...'. Below it is a list of commands, some with keyboard shortcuts. A hint line at the bottom explains arrow keys and Esc.](bild:befehlspalette "The command palette finds every command and file.")

Commands that are not possible now are grayed out (for example without an open project). After the commands you see their keyboard shortcut, so you learn the shortcuts along the way.

## The menus

The menus offer the same commands as the command palette, sorted by task:

- **File:** New project, Open project, Start page, New file, Save, Copy project, Export and more.
- **Edit:** Undo, Find and replace, Folding, Formatting, Markdown and further aids for the source code.
- **View:** Command palette, Areas (sidebar, tray, panel, Maximize, Layout), Editor font, Color scheme, Tabs and splitting, Zen mode, area Project.
- **Project:** Build, Run, Problems and log, Tests, Close project.
- **Help:** Help and further entries.

The shortcuts are shown after the entries. A list of the most important ones is in the chapter [Shortcuts and common questions](kapitel:kuerzel-faq).

## Color scheme, language and font

You choose the color scheme in the menu **Color scheme** of the header bar: **System** (follows the operating system), **Light** or **Dark**. In the menu **View > Color scheme** and in the command palette there is **Switch light / dark**. You choose the language of the user interface in the menu **Language** of the header bar.

You change the font in the source code with **Editor font larger** and **Editor font smaller** (menu **View > Zoom and font** or the command palette).

## Settings

The **Settings** apply to all projects. You open them with the gear in the header bar or with **Cmd/Ctrl+Comma**. On the left are the pages **General**, **Targets and hardware**, **ROMs**, **Appearance**, **Editor**, **Toolchain**, **Shortcuts**, **Backup** and **Info and update**, on the right the content. There you set, for example, language, color scheme, splash screen, emulators, ROM files and the update check (help page [Settings](hilfe:settings)). Everything that belongs to a single project is in the main section **Project** (see [Projects and files](kapitel:projekte)). You also set up emulators, ROMs and hardware with the setup assistant (chapter [Installation and first-time setup](kapitel:installation)).

## Your layout is remembered

You do not have to save the sizes and visibility of the areas: VintageCoder remembers them by itself. Only numbers and switches are stored, never contents. A maximized area and Zen mode are not remembered: at the next start, the remembered normal layout is there.

## Common stumbling blocks

- **Sidebar, tray and panel are missing.** You may have switched on **Zen mode** (Cmd/Ctrl+Alt+Z) or a maximized area; the same command brings them back. Also, only the main section **Code** (with an open project) shows the sidebar and the panel.
- **The panel is not there.** It is visible only on request: **Cmd/Ctrl+Alt+B**.
- **The layout is messed up.** **View > Areas > Reset layout**.
- **Cmd/Ctrl+B does not build.** This shortcut shows and hides the sidebar. You build with F7.
- **A subsection is missing in the second row.** A section is missing if the build of VintageCoder that you use does not contain the matching editor.

## What next?

The next chapter is about what is in the sidebar and what the main section **Project** shows: [Projects and files](kapitel:projekte). After that you get to know the [Code editor](kapitel:editor).
