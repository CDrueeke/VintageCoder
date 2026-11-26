---
id: bauen-starten
titel: Build and run
reihenfolge: 8
stufe: Beginner
zeit: 25 minutes
siehe_auch: [erstes-projekt, assembler, basic, debugger]
referenz: [build, settings, mega65-hardware, memmap, turbo-disk]
i18n_status: maschinell
i18n_quelle: 6f00707aa121
---

# Build and run

**Goal:** You know exactly what happens when you build, where to find the result and the messages, and how to run a program in the emulator. You read the output and fix errors with the "Problems" tab.

## What you need

- A project with a source file, for example the one from [Your first project](kapitel:erstes-projekt).
- An emulator that VintageCoder has found. The start page shows it in the box **Toolchain and hardware** as "ready" (see [Installation and first setup](kapitel:installation)).

## Build and run

There are three ways from source code to a running program. You find **Build** and **Run** in the header (icons **Build** and **Run**) and in the **Project** menu, **Debug** in the **Project** menu and in the **Debug** area:

| Action | Key | What happens |
|---|---|---|
| **Build** | **F7** | builds the project: all programs of the project file in build order. If the project has no sources in the project file, the active file is built. |
| **Run** | **F5** | builds the **start program** and runs it on the run target in the emulator. It does not matter which file is active. The icon **Run** in the header does the same. |
| **Debug** | **F6** | builds the start program, runs it in the emulator with a **debug session** and opens the **Debug** area. There you can pause, go step by step and look at registers and memory (see [The debugger](kapitel:debugger)). |
| **Stop** | **Shift+F5** | stops the running emulator or the debug session (in the **Project** > **Debug** menu: "Stop debugging"). While the emulator runs, the icon **Stop** appears in the header. |

With **Alt+Shift** (**Alt+Shift+F7**, **Alt+Shift+F5**, **Alt+Shift+F6**) the **active file** (the one open in the editor) is used instead of the start program. In the **Project** menu the entries are called "Build project", "Run start program", "Debug start program" and, in the submenu **Active file**, "Build active file", "Run active file" and "Debug active file". On the Mac, Alt is the Option key. Before building, VintageCoder saves all open files.

> **Info:** There is no icon in the header for debugging. You start it with **F6**, through the **Project** menu or in the **Debug** area with the button **Start debugging**. If the last build failed, it does not start: Fix the errors first.

*[Bild folgt in E13: bauen-knopf]*

While a build is running, the icon **Build** is locked, and the status bar shows "building …". Afterwards it shows "built: N bytes" (with errors, "Build: N errors"). The results are in the `build/` folder of your project:

| File | Content |
|---|---|
| `....prg` (Atari: `.xex`, modules: `.crt`, `.car`) | the finished program |
| `.lst` | the **listing**: address, bytes and source code side by side |
| `.vs` | the labels for the emulator (VICE) |
| `.vcdbg` | debug information |

For assembler sources there are more files (for example `.sym.txt` with the symbol table and `.map.txt` with the memory usage); see the notes about the intermediate files in the output.

### The area "Project", page "Build and run"

The area **Project** (**Cmd/Ctrl+Alt+0**) has a page **Build and run**. It shows the **last build** of this session ("Built at ..., no errors." or the number of errors) and has the buttons **Build** and **Run**. Below it is **Emulator per target**: Only overrides that this project sets are shown here. The card **Packages (registry)** names the loaded packages with their source.

You set the **start program** on the page **Sources**. There it is shown as "Start program" with a star in front. For the other programs, you choose **Make start program**. Without a choice, the first program of the sources applies.

### The machine panel

On the right in the area **Code**, the panel **Machine** (show and hide it with **Cmd/Ctrl+Alt+B**) shows the **System**, the **Run target** and the **State** ("ready", "building …", "running"). Below it is the **Last build**: source, program, size, start (SYS) and a table of the segments with address from, to and length. The buttons **Build**, **Run** and **Stop** in the panel do the same as the header. If a debug session is running, the panel also shows the section **Debug session** with PC, A X Y, SP and the flags. If the emulator provides a picture, it is shown there with the button **Refresh picture**. All details are in the **Debug** area.

![The 'Maschine' (machine) panel with system Commodore 64, start target VICE x64sc (cycle exact) and state 'bereit' (ready), and the buttons 'Bauen', 'Starten' and 'Beenden'. Under 'Letzter Bau' (last build) are the source jumprun.asm, the program build/jumprun.prg, the size 16,605 bytes and a table with four segments.](bild:panel-maschine "The 'Maschine' panel with the last build and its segments.")

## Reading the output

The **tray** below the editor has the tabs **Problems**, **Output**, **Tests**, **Search** and **Outline**. In the **Output** tab, you see line by line what happened when building and running.

1. Build your project with **F7**.
2. Open the tray with **Cmd/Ctrl+J** and choose the **Output** tab. Or click "built: N bytes" in the status bar: This opens the tray directly on the output.
3. On success you see, for example, "Done: 34 bytes in ..." with the path of the program. If your project has a disk, a line with its name and size is added.
4. You open the generated files from the sidebar. Folders like `build/` are visible there. For example, the **listing** (`.lst`) shows you what your lines turned into.

The buttons **Copy** and **Clear** in the tab and the context menu (right-click) copy the output or clear it. The same commands are in the **Project** menu as "Copy log" and "Clear log (clear output)". A click on a line with a file name jumps to the place in the editor.

> **Info:** The output divides the build into stages (lines with "▸"). A table with duration and file list does not exist in the user interface. On the command line, `vcc build` shows the process.

If there are errors, the output has one line per message, and a note names "Build failed" with the number of errors.

*[Bild folgt in E13: ausgabe-panel]*

## Step by step: fix errors with the "Problems" tab

1. Build your program. If errors occur, the tray switches to the **Problems** tab by itself.
2. Each line names the severity (**Error**, **Warning**, **Note**), a code like `E202`, the message and the place. The messages of the build are under "Build result", the messages of the language services are below it, sorted by file.
3. A **click** on the message jumps to the line in the editor. A **right-click** on a message offers "Problem: Go to line", "Problem: Copy message" and "Copy all problems".
4. Fix the error and build again. The status bar counts the messages of the file in the tab ("No problems" or the number of errors and warnings). A click on it opens the **Problems** tab. In the editor, you jump with **F8** and **Shift+F8** to the next and to the previous problem.

![The Code tab with the file jumprun.asm and the tray below the editor open, Probleme (Problems) tab active (next to Output, Tests, Search, Outline). It shows 2 messages in 2 files, 0 of them errors, and two hints for lib/core.asm and lib/music.inc about automatically extended branches.](bild:probleme-panel "The tray below the editor with the Problems tab (only hints here, no errors).")

If the build fails and no message belongs to a line (for example because the project file is faulty), the **Output** tab shows the reason.

## Run in the emulator

With **F5** you build and run the start program. The header shows where:

1. Look at the field **Run target** in the header, for example "C64 · VICE x64sc". On the left is the target system, on the right the run target.
2. A click on the field opens a list of the possible run targets. The check mark is in front of the current choice.
3. Press **F5**. The emulator window opens. Your program is loaded and started automatically.

If your project has a **disk** (see [Projects and files](kapitel:projekte)) and the start program is first, VICE starts the disk. This way your program can load more files.

**Turbo BASIC XL (Atari):** here F5 builds a disk (`build/<name>.atr`) from the Turbo disk you provided and boots it in the emulator. If the disk is missing, a banner with the button **Provide ...** appears above the editor. See [Run Turbo BASIC XL](kapitel:basic) and the help page [Turbo BASIC XL: disk](hilfe:turbo-disk). Turbo BASIC programs cannot be debugged step by step yet; F6 reports this.

> **Tip:** The run target only chooses *where* the program runs. Which target system is built is set in the source code (`.target`) or in the project settings (area **Project**, page **Targets**).

> **Info:** Emulator paths, start parameters for the emulator and real hardware (Ultimate 64 / II+, MEGA65) are entered under **Settings > Targets and hardware** (cards **Emulators** and **Real hardware**; menu **View > Settings** or **Cmd/Ctrl+Comma**). For hardware, see the help pages [MEGA65 on the device](hilfe:mega65-hardware) and [Settings](hilfe:settings).

## Common pitfalls

- **F5 does not start anything.** Check the field **Run target** in the header and the emulator on the start page (box **Toolchain and hardware**: "not found" means it is missing). The message is in the **Output** tab.
- **I see nothing after the build.** The tray is collapsed. Show it with **Cmd/Ctrl+J**. If there are errors, it switches to the **Problems** tab by itself.
- **Error, but no line is marked in the editor.** Look at the **Output** tab; it gives the reason.
- **The wrong program starts.** **F5** and the icon **Run** start the **start program** (star on the page **Sources** in the area **Project**). **Alt+Shift+F5** starts the active file. You change the start program on the page **Sources** with **Make start program**.
- **The program runs, but the emulator shows "READY."** For assembler programs with `.basicstub`, type `RUN` in the emulator if the autostart did not start the program.
- **F6 does not start a debug session.** The last build failed, or a session is already running (end it first with **Shift+F5**). With Turbo BASIC XL, debugging does not exist yet.
- **Two emulator windows.** Close the old window or click **Stop** before you start again.

## What next?

The chapter [The debugger](kapitel:debugger) shows how you pause and examine a running program. The chapter [Automatic testing](kapitel:tests) shows how VintageCoder checks your program. For reference: the help pages [Build and run](hilfe:build) and [Settings](hilfe:settings).
