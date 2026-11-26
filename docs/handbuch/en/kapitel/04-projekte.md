---
id: projekte
titel: Projects and files
reihenfolge: 4
stufe: Beginner
zeit: 20 minutes
siehe_auch: [oberflaeche, editor, bauen-starten]
referenz: [overview, editor, wizard, layout]
i18n_status: maschinell
i18n_quelle: 1bc6c64f7191
---

# Projects and files

**Goal:** You understand what a VintageCoder project is and how the file tree is built. You create a new project, open examples, create files and folders, rename them, drag files from outside into the project, set the start program and the sources in the main area **Project** and copy a whole project.

## What you need

- An open project, for example the one from [Your first project](kapitel:erstes-projekt). For exercises with several files, an example also works: on the start page, **Open example** opens a list of all examples from the folder `examples/` of the installation.

## What is a project?

A **project** is a folder on your hard disk. The most important content is the **project file** with the extension `.vcprj`. It says which target system you program for, which source files belong to the project, which of them is the **start program** and whether a disk should be created. Next to it are your source files, graphics and songs. When you build, a subfolder `build/` is created with the finished programs.

A project is **self-contained**. You can move the whole folder, copy it or put it into a Git repository, and everything keeps working. All paths in the project file are **relative** to the project folder.

A fresh project from the wizard typically looks like this (here for the C64 with the assembler template):

```
hallo/
  hallo.vcprj          the project file
  src/c64/main.asm     your source code
  build/               created when building (program, listing, ...)
```

> **Info:** If a target system has several file structures to choose from, the dialog **New project** offers them. On the command line you do this with `vcc new ... --structure <id>` (`vcc new --list` names the structures).

## The start page

If no project is open, the main area **Code** shows the **start page**. You can also reach it at any time through **File > Start page** (this closes the current project; if there are unsaved changes, VintageCoder asks first).

![The start page with no project open: the title VintageCoder, the cards Neues Projekt, Projekt oeffnen, and Beispiel oeffnen, the link Projektdatei (.vcprj) waehlen, the section Zuletzt geoeffnete Projekte (still empty), and the sample projects with filters per system and cards with Oeffnen and Kopieren.](bild:willkommen-seite "The start page appears as long as no project is open.")

On the left are:

- the buttons **New project**, **Open project**, **Choose project file (.vcprj) ...** and **Open example**,
- the list **Recently opened projects**. Each row shows the system, the name, the shortened path and when you last opened the project. The star pins a project (pinned ones are at the top), the menu **...** offers **Open**, **Copy ...**, **Show in folder** and **Remove from list**. If the folder of a project is missing, "Folder missing" is shown next to it,
- the **Example projects**: a selection of recommended examples, each with **Open** and **Copy ...**. **All examples ...** opens a dialog with a search field in which the examples are sorted by target system.

On the right are the detected emulators ("ready" or "not found") and a **Quick reference** with the most important keyboard shortcuts.

> **Tip:** Some examples can only be copied, not opened directly (no button **Open**). With **Copy ...** you create a copy of your own (see below) and change the copy.

## Step by step: Create a new project

1. Choose **File > New project ...** (**Cmd/Ctrl+Alt+N**) or click **New project** on the start page. The dialog **Create new project** opens with three columns.
2. On the left, choose the **target system**. Below it you see whether the hardware profile is valid.
3. In the middle, choose the **language** (**Assembler** or **BASIC**) and a **template**. Below each template are the files that will be created.
4. On the right, type a name at **Project name** (letters, digits, `_` and `-` only) and choose the **Location** with **Choose ...**. Depending on the system, **BASIC dialect**, **Video standard**, **Memory expansion**, **Default emulator**, **Run target (F5)** and **Disk** are added. Further down you decide which **Character set of the project** you want (**Automatic**, **Standard only (own)** or **None**), whether a **README.md** and a **Git repository** are created and whether **More target systems** are added. The **File structure** shows what will be created.
5. At the bottom, click **Create and open project**. *Expected:* VintageCoder creates the project and opens it in the main area **Code**.

![The dialog 'Neues Projekt erstellen' above the start page. On the left is the list 'ZIELSYSTEM' from Commodore 64 (selected) to MEGA65 with the hardware profile 'gueltig'. In the middle is the 'VORLAGE' with 'Assembler-Programm' (selected), 'Steckmodul' and 'Leeres Projekt'. On the right are the 'PROJEKT-DETAILS' with project name, location, video standard, default emulator, start target, disk and character set. At the bottom are the buttons 'Abbrechen' and 'Projekt erstellen und oeffnen'.](bild:neues-projekt-dialog "The dialog 'Neues Projekt erstellen' with target system, template and project details.")

## The file tree

The **file tree** is in the sidebar of the main area **Code** (heading "Project"). It shows the folders and files of the project as they are on the disk. For some files, a tag after the name says their type. You show and hide the sidebar with **Cmd/Ctrl+B**.

*[Bild folgt in E13: projektbaum]*

A click on a file opens it as a tab in the editor, a click on a folder expands or collapses it. With the arrow keys, the **Right arrow** and **Left arrow** keys, **Enter**, **F2** (rename) and **Del** (delete) you can also use the tree without a mouse. If you drag a file or a folder onto another folder in the tree, it is moved there; if you drag it onto the empty area, it goes into the project folder.

At the top of the sidebar are four buttons: **Filter files** (shows a search field that filters the tree by name), **New file**, **New folder** and **Close project**.

Which file belongs to the project and which is the start program is not shown in the tree, but in the main area **Project** (see below).

## Step by step: Create a new file

1. In the sidebar, click **New file**. The folder in which the file is created is the folder you clicked or the folder of the file you clicked last; otherwise the project folder. (You reach the same thing with a right-click on a folder or on the empty area > **New file**, with **File > New file ...** or through the command palette.)
2. The dialog **New file** opens. Under **Type**, choose the type: **Assembler (.asm)**, **BASIC (.bas)**, **Text (.txt)**, **Note (.md)** or **No extension**.
3. Under **Name (path relative to the chosen folder)**, type the name, for example `hilfsroutinen`. Without an extension, VintageCoder adds the extension of the type. With `/` in the name you create subfolders as well.
4. Click **Create**. If the file already exists, the dialog says so.

> **Tip:** **Cmd/Ctrl+N** opens an **untitled tab** instead, in which you can write right away; you give the name when you save (Cmd/Ctrl+S). **Cmd/Ctrl+Shift+T** opens a note. You create graphics files (sprites, character sets, palettes) in their own main areas (**Graphics**).

### Create a folder and a note

Right-click a folder in the tree: **New folder** creates a subfolder, **New file** a file in this folder, **New note** a note (Markdown). A right-click on the empty area offers the same entries in the project folder. In the dialog **New folder** you type the name; with `/` in the name, several folders are created at once, for example `res/bin/fonts`.

## Step by step: Rename and delete

Right-click a file in the file tree. The context menu offers:

- **Open**
- **New file**, **New note**, **New folder**
- **Rename**, **Delete**
- **Copy path** and **Show in folder** (opens the folder in the file manager of your system)

On the empty area, the context menu also offers **Collapse all**, **Reload** (if you created files outside VintageCoder) and **Undo move**.

**Rename:**

1. Choose **Rename** (or press **F2**). The dialog shows the old name.
2. Type the new name. If it already exists, the dialog says so.
3. Click **Rename**. VintageCoder updates the project file and adjusts `.include` and `.incbin` paths in the source files. It briefly reports how many paths were changed.

> **Caution:** Renaming and moving work only when all changed files are saved; VintageCoder tells you. Check the `.include` and `.incbin` lines after renaming anyway. **Undo move** takes back the last move in the tree.

**Delete:** **Delete** (or **Del**) asks first and moves the file to the **trash**. On systems without a trash, the dialog warns that the file will be deleted permanently.

## Step by step: Bring files into the project from outside

Do you have a graphic, a character set or a finished source file in another folder? Simply drag the file onto the VintageCoder window.

1. Drag the file(s) from the Finder or Explorer onto the window. If you drop a project file (`.vcprj`) or a project folder, VintageCoder opens that project instead. Files that are already in the project are simply opened. A disk image opens in the Disk editor.
2. If a file is outside the project, VintageCoder asks: **Copy into the project and open?** The files are listed below.
3. Click **Copy and open**. VintageCoder **copies** the files into the project (nothing is overwritten) and opens them; your originals stay where they are. This keeps the project self-contained.

Without an open project, a single file cannot be dropped; open a project first.

How to use a dropped binary file in the assembler is shown by the example in the help page [Editor](hilfe:editor), section "Dragging files into the project": you drop a character set file `font.bin` into the folder `data` and include it in the source code with `.incbin "../data/font.bin"`.

> **Caution:** Paths in `.include` and `.incbin` are always **relative to the source file**. If you move an included file in the tree or rename it, VintageCoder adjusts the paths. If you change files outside VintageCoder, you have to correct the lines yourself, otherwise the paths point to nothing.

## Step by step: Copy a whole project

You need a copy when you want to try a variant without touching the original, or when you want to use a project or an example as the starting point for a new one.

1. Open the project that you want to copy, or choose **Copy ...** for a project or example on the start page. When copying an open project, VintageCoder saves all open files, so that the copy has the current state.
2. Choose **File > Copy project ...** (or, in the main area **Project**, the subarea **Share** > **Copy project ...**).
3. Enter the **Name of the copy**. The suggestion is "<name> copy". The name becomes both the folder name and the name of the project file, so characters such as `/ \ : * ? " < > |` are not allowed.
4. Use **Choose ...** to pick the **Location**. The suggestion is the folder where the original is.
5. Decide what is copied too. By default, the build folder `build/` (the build results can be created again) and the folder `.git/` (the version history; the copy starts as a new project) are left behind. With **Also copy build folder (build/)** and **Also copy version history (.git/)** you take them along. The dialog says how many files and how much space it is.
6. Click **Copy**. With the check mark **Open the copy afterwards**, VintageCoder switches to the copy right away.

The copy is an independent project with a name of its own; all other settings stay the same. The original stays unchanged. If the destination is already taken, or would be inside the source folder, VintageCoder copies nothing and tells you why. Symbolic links in the project folder are not copied.

## Text files: character encoding and line endings

All text files in the project (source files, notes, project files) are saved as **UTF-8 without BOM** with **LF** as the line ending, the same on every system. If you open a file that is built differently, VintageCoder still reads it and shows a note:

- **BOM removed**: a UTF-8 mark at the start of the file is no longer written.
- **Line endings CRLF to LF**: Windows line endings are converted.
- **Read as Windows-1252**: files that are not valid UTF-8 (for example older texts with umlauts) are read as Windows-1252 and saved as UTF-8 when you save.

The file on the disk changes only when you save. Files with null bytes count as binary files and are not shown in the editor. The assembler and the BASIC translator also build source files with BOM or CRLF without errors.

## The main area Project

Everything that applies to **this** project is in the main area **Project**. You reach it with the button **Project** in the header bar, with **Cmd/Ctrl+Alt+0** or through **View > Areas > Project area**. In the second row below it are seven subareas. Without an open project, the page asks you to open one first.

![The Projekt area with the Uebersicht subpage. Four cards with a green or gray dot: Allgemein, Ziele (C64, PAL), Quellen (Programme: 1, Quellgruppen: 1), and Diskette (not switched on), each with the button Aendern.](bild:projekt-uebersicht "The project overview shows Allgemein, Ziele, Quellen, and Diskette as cards.")

Changes are **checked and saved at once**; there is no Save button for this. If a value is rejected, the page says so.

### Overview

For each subarea there is a card (**General**, **Targets**, **Sources**, **Disk**) with a state in words and color (fine, note, entry missing, not switched on) and a short entry: name and start program, the target systems, the number of programs and source groups, whether a disk is set up. **Change >** jumps to the matching subarea. Problems of the project file are listed below.

### General

- **Name** and **Description** of the project. The name must not be empty.
- **Start program:** the program that **F5** runs and that is built first when building. "Automatic" means: the first program of the sources.
- **Restore the most recently opened files when opening:** switch; **Apply** saves it.

### Targets

The target systems of the project, each with its **variant** (for example PAL or NTSC), which determines clock and number of lines for the emulator, the preview and timing. If the system has no variants, it says so. Adding or removing target systems is not yet possible here.

### Sources

The programs of the project **in build order**, sorted into **source groups** (each with type, target system and, for BASIC, the dialect). A click on a file name opens the file in the code editor. The start program has a star (★); **As start program** makes another file the start program. With the arrows up and down you change the order within a group. If a file is missing on the disk, it is shown in red with the word "missing". If the project contains BASIC sources, there is also the card **BASIC** here with **Start** and **Step** of the line numbers, **Build compact** and **Syntax check when building**.

### Build and run

The card **Last build** shows the result of the last build in this session (time, without errors or number of errors) with the buttons **Build** and **Run**. The card **Emulator per target** names the emulator and the run target for each target system, as far as this project defines them. The card **Packages (registry)** lists the loaded packages with their origin. More about building is in the chapter [Build and run](kapitel:bauen-starten).

### Disk

If the project has a Turbo BASIC XL disk (Atari, BASIC dialect "Turbo BASIC XL"), the card **Turbo BASIC XL** appears here with the state of the disk and the buttons **Provide ...**, **Check**, **Shrink ...** and **Open in the Disk editor**. For other projects, a note says that there are no special settings. Having a disk created when building is not set in this user interface in this version; the **Overview** shows whether one is set up. More about disks is in the chapter [Disk editor, segments, ROMs](kapitel:disk-studio).

### Share

Here you copy the project (**Copy project ...**, procedure see above), for example for passing it on or as a backup. Below it, a note says that the export of graphics, character sets and music is in the respective editor (button **Export ...**; the files created may be used freely). The list **Recently opened** shows the last projects with a button **Open**.

### Step by step: Change the start program

1. Open the main area **Project** and choose **Sources**.
2. For the file that F5 should run, click **As start program**. *Expected:* The star ★ moves to this file.
3. Alternatively, in the subarea **General**, choose the file from the list in the field **Start program**.

## Common stumbling blocks

- **The file does not appear in the file tree.** If you created it outside VintageCoder, right-click the empty area > **Reload**.
- **F5 runs the wrong file.** The start program is in the main area **Project**, subarea **Sources** (star ★) and in the subarea **General**.
- **After renaming, the assembler does not find a file any more.** Check the `.include` and `.incbin` lines; if files changed outside VintageCoder, you have to correct them by hand.
- **A file in the sources is red with "missing".** The project file points to a file that is no longer on the disk. Create it again or change the start program.
- **Renaming or moving does not work.** Save all changed files first (**Cmd/Ctrl+Alt+S**).

## What next?

Now you know where everything is. The next chapter is about how you edit text comfortably in the [code editor](kapitel:editor). After that you learn about [assembler](kapitel:assembler) and [BASIC](kapitel:basic).
