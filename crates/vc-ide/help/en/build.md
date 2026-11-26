# Build and run

- **Build project (F7):** builds the project, that is, all programs in the project file in build order; if the project has no sources, it builds the active file. With **Alt+Shift+F7** the active file is always used ("Build active file"). Output goes to `build/`: the program (`.prg`, `.xex`, `.crt`, `.car`), `.vs` (labels for VICE), `.lst` (listing), `.vcdbg` (debug information).
- **Run start program (F5):** builds and runs the start program (star in the **Project** section on the **Sources** page), whatever file is open. The header button **Run** does the same. With **Alt+Shift+F5** you run the **file in the active tab** instead ("Run active file"). Both run on the run target of the target system, see below. If the project has a disk and the start program is first, VICE runs the disk (so that programs can load more files).
- **Stop (Shift+F5):** stops the running emulator. The header button **Stop** appears as long as the emulator is running.
- **Debug (F6):** builds the start program, runs it in the emulator with a debug session and opens the main section **Debug** (see [Debugger](help:debugger)). **Alt+Shift+F6** debugs the active file. There is no icon for it in the header; in the **Debug** section, the button **Start debugging** starts the session. **Shift+F5** ends it. Debugging is not available yet for Turbo BASIC XL.
- Build and run are in the **Project** menu. Before building, VintageCoder saves all open files.
- **Run target:** the button **Run target** in the header ("C64 · VICE x64sc ▾") shows where the program runs. A click opens the choice. The run target only selects *where* the program runs. The target system is set by `.target` in the source, otherwise in the **Project** section on the **Targets** page. You enter emulator paths and real hardware under **Settings > Targets and hardware**, see [Settings](help:settings).
- **Sprites, character sets, screens and songs** (`font.vcchar`, `hero.vcsprite` ...) are included with the file itself, for example `.include "graphics/hero.vcsprite"`; they are converted when you build (message in the output: "Media file converted").
- **Format:** the default format of the target (PRG, XEX); if `.cartridge` is in the source, the result is a cartridge (`.crt` / `.car`).

On the command line: `vcc asm file.asm -o output --target c64 --format prg`.

## The output
The tray below the editor (**Cmd/Ctrl+J**) has the tabs **Problems**, **Output**, **Tests**, **Search** and **Outline**. When you build and run, the **Output** tab shows line by line what happened ("Done: 34 bytes" with the path of the program, for a disk its name). The status bar shows "building …", then "built: N bytes" or "Build: N errors"; a click on it opens the tray. The output divides the build into stages (lines with "▸"); a table with the duration and the file list does not exist in the user interface, on the command line `vcc build` shows the process. The generated files (listing, symbol table, memory usage ...) are in `build/` and can be opened from the sidebar.

## Problems
If errors occur, the tray switches to the **Problems** tab; a click on a message jumps to the line, a right-click offers "Problem: Go to line", "Problem: Copy message" and "Copy all problems". In the **Output** tab, the buttons **Copy** and **Clear** (and the right-click menu) offer "Copy log" and "Clear log (clear output)". **F8** and **Shift+F8** jump to the next and the previous problem. If the build fails without a message for a line, the **Output** tab shows the reason. If the tray is collapsed, you show it with **Cmd/Ctrl+J**.

## The machine panel
The **Machine** panel (Code section, **Cmd/Ctrl+Alt+B**) shows the system, the run target and the state ("ready", "building …", "running") and below it the last build: source, program, size, start (SYS) and a table of the segments. For the exact memory usage, see [Memory map](help:memmap). While a debug session is running, the panel also shows the section **Debug session** with PC, A X Y, SP, flags and, if the emulator provides one, its picture (button **Refresh picture**).

## On the command line
The same works without the user interface with `vcc build` and `vcc asm`; all commands and options are listed under [Command line](help:cli).
