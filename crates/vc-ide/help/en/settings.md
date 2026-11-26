# Settings

The gear ⚙ in the header, **Cmd/Ctrl+Comma** or the menu **View > Settings** opens the settings. They apply to **all projects** and are stored in `settings.json` in the configuration folder (`~/.config/vintagecoder`, on Windows `%APPDATA%\vintagecoder`). The file is readable only by you (permissions 0600) because it contains the passwords of the hardware. With the environment variable `VC_SETTINGS` you can choose another file. The appearance of the user interface (color scheme, size, font, arrangement of the areas) is stored in a separate file, `oberflaeche.json`.

![The settings on the Allgemein (General) page: on the left the page list, on the right the cards for language (German selected), updates, splash screen, help, project folder and setup. The buttons Save and Discard are at the bottom.](bild:einstellungen "The settings, General page.")

On the left are the pages, on the right the content as cards. The pages are called **General**, **Targets and hardware**, **ROMs**, **Appearance**, **Editor**, **Toolchain**, **Shortcuts**, **Backup** and **Info and update**.

There are two kinds of values:

- **Effective at once and saved right away:** language, color scheme, zoom, editor font, word wrap, **Format a line when leaving it**, splash screen, the two switches under **Shortcuts** and the debug log.
- **Draft with Save:** everything on the pages **General** (updates, manual pictures, projects folder), **Targets and hardware**, **ROMs** and **Toolchain**. At the bottom of the page are **Save** and **Discard** and, as long as something is open, the note "Unsaved changes". **Discard** takes the draft back.

## General
The page consists of cards:

- **Language:** *Automatic*, *Deutsch* or *English*. The choice takes effect at once. English is machine-translated and not reviewed yet.
- **Updates:** whether VintageCoder looks for a new version (*Not asked yet*, *On request*, *Daily*, *Weekly*) and the **Address of the version list** (empty = default). Nothing is sent without your consent. See [Check for updates](help:updates).
- **Splash screen:** **Show the splash screen at startup** (On or Off).
- **Help:** **Load manual pictures online** (*Ask*, *Always load*, *Never load*) and the **Location of the manual pictures** (empty = default address). The pictures of the manual are on the internet, not in the program file. When loading, only the address of the picture is requested; the pictures stay in the cache. A folder instead of a web address is read instead of downloaded. The environment variable `VC_HELP_IMAGES` takes priority.
- **Projects folder:** the place where **New project** suggests new projects (with **Choose …**). Empty = default. Changing it does not move anything.
- **Setup:** starts the setup assistant: **Quick setup (7 steps)**, **Set everything myself (12 steps)** or one part (**ROMs …**, **Emulators …**, **Hardware …**, **Appearance and editor …**). See [Setup assistant](help:wizard).
- **Troubleshooting:** the switch **Write a debug log to a file** (see [Send feedback](help:feedback)).
- **Settings file:** the location of the file, with the button **Show in folder**.

## Targets and hardware
On the left are the **target systems** you have entered. Below them you choose **Add a system …** and **Add** to add another of the six systems: C64, C128, C16/Plus/4, VIC-20, Atari 800 XL/XE, MEGA65. **Remove system** (in the header of the chosen system) takes it out.

On the right is the chosen system, with manufacturer and processor in the header. The card **Start target** chooses where F5 and the Run button start for this system: **Emulator (default of the system)** or a machine of the system.

### Emulators
A system can have **several emulators**, exactly one of them is the **default** (selection button **Default**). The default emulator starts on F5 and when debugging, if the start target is set to emulator. With **Add emulator …** and **Add** you add another of the chosen kind, **Remove** takes one out. The kinds:

| System | Kinds |
|---|---|
| C64 | VICE x64sc (cycle exact), VICE x64 (fast) |
| C128 | VICE x128 |
| C16 / Plus/4 | VICE xplus4 |
| VIC-20 | VICE xvic |
| Atari 800 XL/XE | Altirra (Windows only), atari800; debugger: Fujisan (if found), atari800 |
| MEGA65 | Xemu (xmega65) |

For each emulator the card has these fields:

- **Name** and **Kind of emulator.**
- **Program (empty = search automatically):** an empty path means it is searched automatically (`PATH` and the usual folders). **Search** enters the path that was found. A line below shows whether the emulator was found ("Found: …") or not ("Not found. Enter the path by hand."). An entered path is checked for existence and whether it can be run (for `.app` bundles the program inside counts).
- **Test:** starts the emulator with exactly these parameters and ends it again after a short time. The message shows whether the start worked.
- **Install:** if the program is missing and a fixed package exists, the card offers **Install portable (version) …**. It first names the name, version, size, source and destination; nothing is downloaded until you click **Download now** (see [Emulators](help:emulators)). If there is no package, the card names the download page or the way through the package manager.
- **ROM** (only atari800, Xemu and MEGA65): for atari800 the operating system ROM, for Xemu the folder with `MEGA65.ROM`. These are single-case entries and take priority over the ROM folder; normally you put the ROMs on the page **ROMs** (help "ROM files").
- **Start parameters** (the card **Start parameters**): are appended to the command line and apply to F5 and to debugging. Spaces separate them; with `"..."` or `'...'` you can give paths with spaces.

Without an emulator, a system can only be built, and it runs only on a machine.

### Real hardware
Under **Real hardware** are the machines of the system. With **Connect …** you enter one. Which ones exist depends on the system and on the package:

- **Ultimate 64 / II+** (C64): **Address** (`192.168.1.50` or `host:port`) and, if set in the firmware, the **Network password** (hidden, **Show** reveals it). The switch **Reset the machine before the transfer** resets the machine before the program arrives. `.prg` files are started through the REST interface (reset, load, RUN). When debugging, memory access, pause and continue are possible, but no registers, no single step and no breakpoints.
- **MEGA65** (hardware; hidden in the packages of this version, present in the source): the **Connection** is **USB** (**Serial port**, list of the ports found, **Baud rate**; empty: 2000000; the program is loaded and started through the monitor) or **Ethernet** (`etherload`, `mega65_ftp`, **Network interface of the computer**, **Folder on the SD card**; unchecked: not tested on a device yet). See the help page "MEGA65 on the device".

The switch **Run RUN after the transfer** applies to every machine. **Test connection** checks whether the machine answers; the message shows the result. No network access happens when the application starts, the connection is checked only on a click. For the other systems there is no hardware you can connect.

## ROMs
Where your ROM files are and what was detected; see [ROM files](help:roms).

## Appearance
- **Color scheme:** *System*, *Dark*, *Light* or *High contrast*. At the very first start, color scheme, size and font are taken over from the `view.json` of an older installation (only read, the file stays unchanged).
- **Zoom:** the size of the whole user interface with **-**, **+** and **100 %**; also with Cmd/Ctrl+Plus, Cmd/Ctrl+Minus and Cmd/Ctrl+0.
- **Reset appearance (color scheme, zoom, editor):** restores the defaults. The command **Reset appearance (size, font, layout)** in the command palette and in the menu **View** also resets the layout.

## Editor
- **Editor font size:** **-** and **+**, size in points.
- **Word wrap** and **Format a line when leaving it** (for assembler and BASIC, the edited line is aligned as soon as you leave it).
- **Menu bar in the window:** on macOS the menu bar of the system is used, otherwise the menu bar in the window is on. This card only shows the state.

Markdown has no settings of its own; the Markdown commands are in the menu **Edit** and in the command palette (see [Markdown](help:markdown)).

## Toolchain
The page shows what is built in:
- **Assembler:** the built-in `vcasm` ("Built in (vc-asm). No other program needed."). The card **Assembler options when building** takes additional options, for example `-I` for search folders.
- **Include search:** first the folder of the project, then additional folders (`-I`), last the system files of VintageCoder.
- **BASIC dialect per target system:** the BASIC of a project follows the dialect of its target system. Line numbers are optional (area Project, page Sources).

Whether a project restores the tabs of the last session when it opens is set in the settings of the project (area Project > General), not here.

## Shortcuts
- **Single keys in the editors:** letters and digits choose tools and enter notes. Off: only shortcuts with Cmd/Ctrl or Alt work.
- **Show computer keys on the keyboard:** shows the keys on the keyboard in the tracker. The keys apply by their position on the keyboard, so no choice of keyboard layout is needed.
- **Keyboard shortcuts:** a list of all commands with shortcut and a search field **Search a command or shortcut**. The shortcuts are fixed; customizing is not possible. See also [Keyboard shortcuts](help:shortcuts).

## Backup
Here you export, import and reset the configuration; see [Back up configuration](help:config).

## Info and update
Name, version, license, website, source code, feedback, the check for updates and the third-party notices. See [About VintageCoder](help:about) and [Check for updates](help:updates).
