# Start page, setup assistant and new project

## Start page
If no project is open, the main area **Code** shows the **start page**. With the command **Start page** (menu **File**, also in the command palette) you bring it back at any time; this closes the project (if there are unsaved changes, VintageCoder asks first).

![The start page with no project open: the title VintageCoder, the cards Neues Projekt, Projekt oeffnen, and Beispiel oeffnen, the link Projektdatei (.vcprj) waehlen, the section Zuletzt geoeffnete Projekte (still empty), and the sample projects with filters per system and cards with Oeffnen and Kopieren.](bild:willkommen-seite "The start page appears as long as no project is open.")

At the top are the logo, the name and the version number. Below them follow:
- the cards **New project**, **Open project** (Cmd/Ctrl+Shift+O) and **Open example**, and the link **Choose project file (.vcprj) ...**,
- the list **Recently opened projects** (pinned ones first; the start page shows at most five, **More ...** opens the rest). A click opens the project, the star pins it to the top, the menu **...** offers **Open**, **Copy ...**, **Show in folder** and **Remove from list**. If the folder is missing, it says "Folder missing",
- the **Example projects**: a selection of recommended examples from the folder `examples/`, with a filter for each system and, for each example, **Open** and **Copy ...**. **All examples ...** opens a dialog with a search field.

At the bottom are two boxes. The box **Toolchain and hardware** shows the **emulators that were found** ("ready" or "not found"); the quick reference next to it names the most important keyboard shortcuts. As long as you have not set up VintageCoder yet, or the setup is incomplete, the card **Welcome to VintageCoder** takes the place of the box **Toolchain and hardware**, with the buttons **Start setup** and **Later** (**Later** hides the card). If VintageCoder has found new emulators since the last setup, the card **New finds** appears with **Set up emulators**, **Remember as known** and **Do not show again**.

## Setup assistant
The setup assistant sets up target systems, emulators, ROMs, hardware and appearance. It opens by itself at the very first start (unless the environment variable `VC_NO_WIZARD=1` is set) and can be repeated at any time: with **Start setup** on the start page (as long as the card **Welcome to VintageCoder** is visible), in the menu **Help > Setup assistant ...**, in **Settings > General > Setup** or through the command palette (Cmd/Ctrl+K, the words "Setup assistant").

![First step, 1. Willkommen (Welcome), of the setup wizard: on the left the step list, on the right the language choice Automatisch, Deutsch, English and the choice between the quick setup with 7 steps (selected) and setting everything yourself with 12 steps.](bild:assistent-weg-waehlen "The welcome step: pick the language and choose between the quick setup and setting everything yourself.")

It follows three rules:
- **It installs nothing and downloads nothing.** It only shows what was found on your computer. In the step **Emulators** it shows the same page as the settings; an emulator is downloaded portably only after you click **Download now**.
- **It works on a draft.** Nothing is written until **Apply** in the **Summary**, and a backup of the settings is made first. Only the **Design** (color scheme, zoom, font) takes effect at once; **Cancel** restores the old state.
- **You can stop at any time.** **Cancel** ends the assistant without any change. **Continue later** does the same and remembers the step; the start page then reminds you of the open setup.

On the left is the list of **steps**, at the bottom **Cancel**, **Continue later**, **Back** and **Next**. The first step, **Welcome**, has the **Language** (*Automatic*, *Deutsch*, *English*) and two ways:
- **Quick setup (7 steps):** Welcome, Design, Target systems, Toolchain, Projects, Summary, Done. Emulators, ROMs and hardware are found automatically and refined later in the settings.
- **Set everything myself (12 steps):** in addition **Emulators**, **ROMs**, **Real hardware**, **Start target F5** and **Editor**.

The partial assistants set up only one part: **Setup assistant: ROMs only ...**, **emulators only ...**, **real hardware only ...** and **design and editor only ...** (menu **Help**, command palette, **Settings > General > Setup**). **Setup assistant: Look for new items ...** opens the ROM part.

The steps:
- **Design:** color scheme (*System*, *Dark*, *Light*), zoom and editor font.
- **Target systems:** Which systems do you want to program for? Systems you untick disappear from the interface; their entries are kept.
- **Toolchain:** names the built-in assembler and shows the page of the emulators.
- **Emulators, Real hardware, Start target F5:** the same cards as under **Settings > Targets and hardware** (see [Settings](help:settings)). Machines are optional, connections are tested only on a click.
- **ROMs:** the same page as **Settings > ROMs**, see [ROM files](help:roms).
- **Editor:** font, word wrap, format line when leaving it.
- **Projects:** the **Project folder** and, if examples are available, the switch **Copy example projects of the chosen systems** with the list of examples (each can be unticked). Existing files stay untouched.
- **Summary:** every change with a check mark, area, kind (+ new, ~ changed, - removed) and the old and new value; an unticked check mark means "do not apply". **Apply** writes. After that, the step shows the place of the backup, the number of copied examples and the button **Undo**. If there is nothing to change, it says so there.
- **Done:** says that the setup is complete. Under **First project**, **Create a new project ...** opens the dialog **Create new project** right away, and **Open example: ...** opens a copied example. **Close** ends the assistant.

The assistant builds and starts nothing by itself. Not part of the configuration are: keyboard shortcuts (they are fixed) and storing passwords in the keychain (still missing). Backing up and resetting the settings is under **Settings > Backup**, see [Back up and reset the configuration](help:config).

## New project
**File > New project ...** (Cmd/Ctrl+Alt+N), the button on the start page or `vcc new` creates a project: the project file `.vcprj`, source files from a template and the folder `src/<system>/`. The dialog **Create new project** has three columns:

1. **Target system:** C64, C128, C16/Plus/4, VIC-20, Atari 800 XL/XE or MEGA65, plus the **Hardware profile** (CPU and chips).
2. **Template:** first the language, **Assembler** or **BASIC**, then the template. If there is none for the combination, it says so. BASIC is written in the dialect of the target system, without line numbers and with jump labels (`@name`); building creates the native program.
3. **Project details:** **Project name** (letters, digits, `_` and `-`), **Location** (with **Choose ...**), **BASIC dialect** (where there are several), **Video standard** PAL or NTSC (not on the MEGA65), on the VIC-20 the **Memory expansion**, the **Default emulator** and the **Run target (F5)** (preset from the settings; a difference is saved in the project file) and the **Disk**: none or a format that fits the system (C64 D64, C128 D64, D71 or D81, Atari ATR, MEGA65 D81). The **Character set of the project** is *Automatic* (your ROM has priority, otherwise the own set VC Standard as a copy), *Standard only (own)* or *None*. There are also two switches: **Create README.md with build notes** and **Create a Git repository** (with `.gitignore` for `build/`). Under **More target systems** you can tick additional systems of the template; the **File structure** shows what is created.

![The dialog 'Neues Projekt erstellen' above the start page. On the left is the list 'ZIELSYSTEM' from Commodore 64 (selected) to MEGA65 with the hardware profile 'gueltig'. In the middle is the 'VORLAGE' with 'Assembler-Programm' (selected), 'Steckmodul' and 'Leeres Projekt'. On the right are the 'PROJEKT-DETAILS' with project name, location, video standard, default emulator, start target, disk and character set. At the bottom are the buttons 'Abbrechen' and 'Projekt erstellen und oeffnen'.](bild:neues-projekt-dialog "The dialog 'Neues Projekt erstellen' with target system, template and project details.")

**Create and open project** creates everything and opens the project in the area **Code**. If the button is gray, the column gives the reason (for example a forbidden character in the name). See also the manual chapter [Your first project](kapitel:erstes-projekt).

## Project structure
You choose additional folders (*Simple*, *Standard*, *Large, modular*, *Multiple systems*, *Library*, *Game with levels*) currently **on the command line** with `vcc new ... --structure <id>` (`vcc new --list` lists the structures). The dialog **Create new project** has no choice for this yet; it only creates the folder `src/<system>/`. The structure creates the folders and enters, in the `layout` section of the project file, where dropped files go for each category. With *Multiple systems*, one folder in `src/` and `res/` is created for each chosen target system. With the Git repository ticked, new empty folders get a `.gitkeep`. The bundled templates are in `data/templates/structures.json`, your own in `structures.json` in the configuration folder.

## Command line and templates
- Systems: `c64`, `c128`, `plus4`, `vic20`, `atari8`, `mega65` (also several, comma-separated).
- Templates: `asm-hello`, `basic-hello`, `empty`. `asm-hello` is also available as a **cartridge** variant (`--variant cartridge`; C64, Atari, VIC-20): the same project file, the start source contains `.cartridge`, the address of the cartridge area and the identifier of the system. `--template cartridge` is kept as a short form.
- Folder: relative to the open folder; `.` only if there is no project there yet.
- Graphics and sound: the editors offer only profiles of the target systems of the project; sprites, character sets and songs of other systems cannot be opened or created.

        vcc new mygame --target c64,mega65 --template asm-hello
        vcc new mymodule --target c64 --template asm-hello --variant cartridge
        vcc new --list

Your own templates: a folder and an entry in `data/templates/index.json` (`{system}` in the path, `{{name}}` in the text). A template can have variants under `variants` with their own systems, files and sources; they replace the files of the template.

Constants for the assembler are in the target of the project file (like `vcc asm -D NAME=number`), for example `"defines": { "DEBUG": 1 }` for `.if DEBUG`; `true` and `false` count as 1 and 0. They apply when building and in the live errors of the editor.

With `--structure <id>` (`vcc new --list` lists the structures), `vcc new` creates the same folders and the `layout`, for example `vcc new game --target c64 --structure large`.
