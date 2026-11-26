---
id: installation
titel: Installation and first-time setup
reihenfolge: 0
stufe: Beginner
zeit: 30 minutes
siehe_auch: [willkommen, erstes-projekt, oberflaeche]
referenz: [start, settings, roms, mega65-hardware]
i18n_status: maschinell
i18n_quelle: bf1dda76812a
---

# Installation and first-time setup

**Goal:** You install VintageCoder on macOS, Windows or Linux (Debian, Ubuntu, Fedora, openSUSE), start it for the first time, set it up with the **setup assistant** (emulator, systems, project folder) and check with a Hello World program and **F5** that everything works.


## Requirements

- a computer with **macOS on an Apple chip (arm64)**, **Windows (64 bit, x86_64)** or **Linux (x86_64)**; Intel Macs and ARM computers under Windows and Linux are **not** supported, there are no packages for them,
- the installation package for your system,
- an **emulator** for the system you are interested in (see below); you install it yourself, the assistant only shows whether it was found,
- optionally the **ROM files** of your target systems.

> **Info:** The **first-run setup** is done by the setup assistant. It opens by itself at the very first start, checks what is already on your computer, and writes something to your settings only after you confirm. This chapter takes you through it in order: download, installation, first start, assistant, first program. Everything the assistant sets, you can also change by hand in the **Settings** (section "Set up by hand").

## Download

The installation packages are at [github.com/CDrueeke/VintageCoder/releases](https://github.com/CDrueeke/VintageCoder/releases) (version 0.1.0: tag `v0.1.0`, marked as a pre-release) and on the website of the project, [vintagecoder.drueeke.de](https://vintagecoder.drueeke.de/). You need only the file(s) for your system:

| System | File(s) | Requirement |
|---|---|---|
| **Windows** | `VintageCoder-<Version>-Setup.exe` | Windows 11 tested, 64 bit (x86_64) |
| **macOS** | `VintageCoder-<Version>-macos-arm64.dmg` | Mac with Apple chip (arm64); Intel Macs not supported |
| **Debian, Ubuntu** | `vintagecoder-cli_<Version>_amd64.deb` and `vintagecoder_<Version>_amd64.deb` | Debian 13 or newer, Ubuntu 24.04 or newer |
| **Fedora, openSUSE** | `vintagecoder-cli-<Version>-1.x86_64.rpm` and `vintagecoder-<Version>-1.x86_64.rpm` | Fedora 45 or newer, openSUSE Tumbleweed (each with glibc 2.44 or newer) |
| **other Linux systems** | `vintagecoder-<Version>-linux-x86_64.tar.gz` | x86_64, glibc 2.39 or newer |

On the GitHub release pages, the files carry an addition in the name depending on the distribution (for example `...debian13.deb`, `...fedora45....rpm`) so that equal names do not collide; the content is the same. Every release includes a file `SHA256SUMS`. Use it to check the download (Linux: `sha256sum -c SHA256SUMS`, macOS: `shasum -a 256 -c SHA256SUMS`, Windows: `Get-FileHash` in PowerShell and compare the sum).

> **Caution:** The packages are **not signed** (no developer certificate, no notarization by Apple). For this reason, Windows and macOS warn you at the first start; how to deal with this is described in the sections for your system. Download packages only from the official release or from the website of the project.

How to open the file is described in the following sections.

## Windows

> **Info:** The setup program is tested on Windows 11 (x86_64): installation, start and uninstallation. Windows 10 is not tested; 32-bit Windows and Windows on ARM are not supported.

1. Start the setup program `VintageCoder-<Version>-Setup.exe`. Because it is **not signed**, Windows SmartScreen may report "Windows protected your PC" first (in some versions the text differs). In that case, click **More info** and then **Run anyway**, if you downloaded the file from the official release or from the website of the project. Then follow the questions. The setup needs administrator rights only if you want to install for all users.
2. The setup offers three choices:
   - **Create a desktop icon** (off),
   - **Add the command line program vcc to the PATH** (off; check it if you want to use `vcc` in every command prompt),
   - **Copy the example projects to "Documents\VintageCoder\Beispiele"** (on; the folder name `Beispiele` means "Examples").
3. At the end you can start VintageCoder right away. The setup also associates the extension `.vcprj`: a double click on a project file opens it in VintageCoder.

## macOS

1. Open the file `VintageCoder-<Version>-macos-<Architecture>.dmg`. In the window are **VintageCoder.app**, a shortcut to the **Applications** folder, the folder **Beispiele** ("Examples") and the file **Kommandozeile installieren.command** ("install command line").
2. Drag **VintageCoder.app** onto **Applications**. Also copy the folder **Beispiele** to a place where you may write (for example into your Documents), because building creates files in the project folder.
3. Start the app. If it is not notarized by Apple, macOS (Gatekeeper) refuses to open it. In that case, in the Applications folder, click VintageCoder with the **right mouse button** and choose **Open**, then **Open** once more in the dialog. You have to do this only at the first start. On newer macOS versions, the entry **Open** may not appear in the dialog; in that case you open **System Settings > Privacy & Security**, scroll to the note about VintageCoder and click **Open Anyway**.
4. If you want to use the command line tool `vcc` in the terminal, double-click **Kommandozeile installieren.command** (with the app already in Applications). It links `vcc` and `vc-lsp` to `/usr/local/bin`; if needed, it asks for your password. Then open a new terminal window.


## Linux (Debian, Ubuntu)

The `.deb` packages exist for **Debian 13** and **Ubuntu 24.04** (each or newer), only for x86_64. Older versions (Debian 12, Ubuntu 22.04) have a system library (glibc) that is too old and are not supported. Download both packages into the same folder. If you need only the command line, install only `vintagecoder-cli`; the user interface (`vintagecoder`) always needs the command line package in the same version.

1. Open a terminal in the folder with the packages and install both with `apt` (the `./` before the names is important, otherwise `apt` searches the package sources; `apt` fetches needed libraries by itself):

   ```bash
   sudo apt install ./vintagecoder-cli_<Version>_amd64.deb ./vintagecoder_<Version>_amd64.deb
   ```

2. Check the installation. This command shows the version:

   ```bash
   vcc --version
   ```

3. Start the user interface from the application menu (entry "VintageCoder", category Development) or in the terminal with `vintagecoder`. The example projects are in `/usr/share/vintagecoder/examples`; copy them to a place where you may write, because building creates files in the project folder.

> **Info:** At the end of the installation, `apt` may print the line "N: Download is performed unsandboxed as root as file '...' couldn't be accessed by user '_apt'". This is only a note, not an error; the installation is complete. Otherwise `apt` downloads packages with the restricted system user `_apt`, which is not allowed to look into your personal folder. If you want to avoid the line, copy the packages to `/tmp` first and install from there (`sudo apt install /tmp/vintagecoder-cli_<Version>_amd64.deb /tmp/vintagecoder_<Version>_amd64.deb`).

To remove it, `sudo apt remove vintagecoder vintagecoder-cli` is enough.

## Linux (Fedora, openSUSE)

The `.rpm` packages exist for **Fedora 45** and **openSUSE Tumbleweed** (each or newer), only for x86_64. They require glibc 2.44 and cannot be installed on older Fedora versions and on openSUSE Leap. Download both packages into the same folder, open a terminal there and install them; the package manager fetches needed libraries by itself.

Fedora:

```bash
sudo dnf install ./vintagecoder-cli-<Version>-1.x86_64.rpm ./vintagecoder-<Version>-1.x86_64.rpm
```

openSUSE Tumbleweed (the packages are not signed; `zypper` asks whether it should install them anyway, or you give `--allow-unsigned-rpm`):

```bash
sudo zypper install --allow-unsigned-rpm ./vintagecoder-cli-<Version>-1.x86_64.rpm ./vintagecoder-<Version>-1.x86_64.rpm
```

Then check with `vcc --version` and start the user interface as described above. To remove it: `sudo dnf remove vintagecoder vintagecoder-cli` or `sudo zypper remove vintagecoder vintagecoder-cli`.

## Linux (other systems, archive)

The archive `vintagecoder-<Version>-linux-x86_64.tar.gz` needs no installation: unpack it (`tar xzf vintagecoder-<Version>-linux-x86_64.tar.gz`) and start `usr/bin/vcc --version`; the program finds its data by itself. With `sudo cp -a usr/. /usr/local/` you install it to `/usr/local`. For the user interface (`usr/bin/vintagecoder`), the usual libraries must be present (ALSA, xkbcommon-x11, GL/EGL, Wayland, Xcursor, Xrandr, Xi); the list is in the help page "Linux". The requirement is glibc 2.39 or newer. For the archive, only the command line is tested, not the user interface.

> **Info:** The Linux packages were tested in fresh containers of each distribution (installation, `vcc`, start of the user interface with software rendering) and also on an Ubuntu 24.04 test machine with a desktop (virtual machine); there the menu bar, the Alt shortcuts, the file dialog and the trash worked. **Sound** and a **Wayland session** are not tested yet.

## The first start

1. Start VintageCoder. The first thing you see briefly is the **splash screen**: the logo with the name and the sentence "Assembler development for 8-bit computers". A click or any key closes it; otherwise it disappears by itself after about a second.
2. **Note:** VintageCoder probably still contains many errors. There is no warranty (see the license). Back up your projects regularly, for example with Git or a backup, especially before you transfer anything to real devices.
3. You can switch the splash screen off: **Settings > General > Splash screen**, switch **Show the splash screen at startup**. The choice applies from the next start.
4. At the **very first start** (there is neither `settings.json` nor `wizard.json` in the configuration folder), the **setup assistant** opens over the user interface. You go through it in the next section. After that you see the **start page**, the starting point as long as no project is open.

![The start page with no project open: the title VintageCoder, the cards Neues Projekt, Projekt oeffnen, and Beispiel oeffnen, the link Projektdatei (.vcprj) waehlen, the section Zuletzt geoeffnete Projekte (still empty), and the sample projects with filters per system and cards with Oeffnen and Kopieren.](bild:willkommen-seite "The start page appears as long as no project is open.")

5. At the top are the cards **New project**, **Open project** (Cmd/Ctrl+Shift+O) and **Open example** (opens a dialog with the supplied examples and a search field). Below them follow the link **Choose project file (.vcprj) …**, the list of **recently opened projects** (empty at the first start) and the **example projects** with a filter for each system.
6. At the bottom, the box **Toolchain and hardware** shows for each emulator a green dot with "ready" (found) or a gray one with "not found". Next to it is the **quick reference** with the most important keyboard shortcuts. As long as you have not set anything up, the card **Welcome to VintageCoder** with **Start setup** and **Later** is in the place of the box. At start-up there is no network access.

> **Info:** On the Mac, Cmd means what Ctrl is on Windows. In the manual you therefore see **Cmd/Ctrl**. You reach the start page later with the command **Start page** (menu **File**, also in the command palette).

> **Info:** The menu bar with **File**, **Edit**, **View**, **Project** and **Help** exists on all systems: on the Mac at the top edge of the screen, otherwise in the window. You can also reach all commands through the command palette (**Cmd/Ctrl+K**), the shortcuts, the header bar and the context menus. You open the area **Settings** with the gear icon in the header bar or with **Cmd/Ctrl+Comma**.

> **Tip:** If you hid the card **Welcome to VintageCoder** with **Later**, it stays away. You can find the setup at any time under **Settings > General > Setup** and in the menu **Help**.

## The setup assistant

The assistant takes you in a few minutes from "freshly installed" to "F5 runs my program". It works by three rules:

- **Nothing is sent.** No network access, no telemetry. It **installs nothing** and downloads **nothing**; it only shows what was found on your computer. There is one exception: in the step **Emulators** (long way), the assistant shows the same page as the settings. If an emulator is missing there, it can offer a portable installation; nothing is downloaded until you click **Download now**.
- **It works on a draft.** All changes are first kept in memory. Only in the **Summary** do you see the list and confirm with **Apply**. Only the **design** (color scheme, zoom, font) takes effect at once, so that you see what you choose; **Cancel** restores the old state.
- **You can stop at any time.** **Cancel** ends the assistant without changing any settings. **Continue later** does the same and remembers which step you were at, not your entries.

On the left are the **steps** with their groups. At the bottom are **Cancel**, **Continue later**, **Back** and **Next**. With **Esc** you cancel.

### Choosing the way

The first step, **Welcome**, has the **Language** at the top (*Automatic*, *Deutsch*, *English*; English is machine-translated and not reviewed yet) and offers two ways:

| Way | Steps | What for |
|---|---|---|
| **Quick setup (7 steps)** | 7 | design, target systems, toolchain and projects; emulators, ROMs and hardware are found automatically and refined later in the settings |
| **Set everything myself (12 steps)** | 12 | in addition emulators, ROMs, real hardware, run target F5 and editor |
| **Only one part** | 3 to 4 | the partial assistants **ROMs …**, **Emulators …**, **Hardware …** and **Appearance and editor …** set up only that part |

![First step, 1. Willkommen (Welcome), of the setup wizard: on the left the step list, on the right the language choice Automatisch, Deutsch, English and the choice between the quick setup with 7 steps (selected) and setting everything yourself with 12 steps.](bild:assistent-weg-waehlen "The welcome step: pick the language and choose between the quick setup and setting everything yourself.")

With **Next** you go one step forward, with **Back** one step back. You can click steps you have already reached directly in the list on the left. If you have already left the assistant once without finishing, this page says "Setup is not finished yet."

### Quick setup, step by step

**Step 2: Design.** Choose **System** (follows the operating system), **Dark** or **Light**. Two pairs of buttons **-** and **+** set the **Zoom** of the user interface (in percent) and the **Editor font size** (in points). Everything takes effect at once.

**Step 3: Target systems.** Check the systems you want to work with: C64, C128, C16/Plus/4, VIC-20, Atari 800 XL/XE, MEGA65. Only chosen systems appear later in the user interface; deselected ones are only hidden, nothing is lost.

> **Info:** The scope of this version is C64, C128, C16/Plus/4, VIC-20, Atari 8 bit and MEGA65. The Commodore PET/CBM is not included.

**Step 4: Toolchain.** The page names the built-in **assembler** ("Built in (vc-asm). No other program needed."; no Java, no additional installation) and below it shows the same page as **Settings > Targets and hardware**: for each emulator whether it was found ("Found: ...") or is missing ("Not found. Enter the path by hand."). The search covers the `PATH` and the usual folders. You enter missing emulators here, in the step **Emulators** (long way) or later in the settings. Without an emulator, the affected systems can be built but not run.

> **Info:** On the Mac, the detection searches the `PATH` and the usual folders (Homebrew, `/Applications`). On Windows and Linux, it searches mainly the `PATH`. If your emulator is somewhere else, you enter the path in the settings (**Targets and hardware**, field **Program**). On macOS, Gatekeeper blocks programs that are not notarized at the first start; then right click > **Open** helps.

**Step 5: Projects.** The **project folder** is the folder where your projects are; **New project** suggests it as the location. Empty means: `~/VintageCoder`. Changing it moves nothing, it only changes the default for new projects. With the switch **Copy example projects of the chosen systems**, the assistant puts the example projects for your systems into the project folder (subfolder "Beispiele"); you can deselect single examples. Copying happens only at **Apply**, existing files stay untouched. If your installation contains no examples, it says "This installation contains no examples."

**Step 6: Summary.** The list shows each change with a check mark, the area and the kind (+ new, ~ changed, - removed) and the old and the new value. You can deselect every entry; what is not checked is not written. A backup of the settings is made before writing. If there is nothing to change, it says "Nothing to change. Everything is already set." **Apply** writes.

![Setup wizard, step 6 Zusammenfassung (summary). One card lists what will change: under SYSTEM the target system Commodore 64 with VICE x64sc, and under PROJEKTE the copying of 15 examples, each with a checked box. The buttons Zurueck and Uebernehmen are at the bottom right.](bild:assistent-zusammenfassung "The summary lists every change with a checkbox; nothing is written until you click Uebernehmen.")

**Step 7: Done.** The assistant reports "Applied." and shows the location of the backup, the number of copied examples and notes. With **Undo** you restore settings and copied examples. Under **First project**, you create a project right away with **Create a new project …** (the dialog **Create new project** opens) or open a copied example with **Open example: ...**. **Close** ends the assistant.

> **Caution:** The assistant does **not build or run anything by itself**. Whether the chain of building and emulator really works, you check in the section "The test: Hello World with F5".

### "Set everything myself": the additional steps

The long way has twelve steps: Welcome, Design, Target systems, Toolchain, **Emulators**, **ROMs**, **Real hardware**, **Run target F5**, **Editor**, Projects, Summary, Done. The steps in between are:

| Step | What you set there |
|---|---|
| **Emulators** | for each system the default emulator, path and start parameters (the same page as **Targets and hardware** in the settings) |
| **ROMs** | the same page as **Settings > ROMs**: choose ROM files or ZIP archives or drag them onto the window; detection is by content, no download takes place. Files go into the ROM folder only after your confirmation. Free replacement ROMs are often enough |
| **Real hardware** | enter machines (for example Ultimate 64 / II+); connections are tested only on click |
| **Run target F5** | exactly one target per system: the default emulator or an entered machine; F5 and F6 run there |
| **Editor** | editor font size, **Word wrap** and **Format a line when leaving it** (the page **Editor** of the settings). The **keyboard shortcuts are fixed**; customizing them is not possible |

> **Caution:** The network password of an Ultimate is stored in `settings.json` (readable only by you). Storing it in the keychain of the operating system does not exist yet.

### Opening the assistant again later

You can open the assistant again at any time:

- menu **Help > Setup assistant ...**,
- in the **Settings** on the page **General** in the card **Setup** (**Quick setup (7 steps)**, **Set everything myself (12 steps)** or one part),
- on the start page with **Start setup**, as long as the card **Welcome to VintageCoder** is visible,
- in the command palette (Cmd/Ctrl+K) with the word "Setup assistant".

The **partial assistants** set up only one part: **Setup assistant: ROMs only ...**, **emulators only ...**, **real hardware only ...** and **design and editor only ...**. **Setup assistant: Look for new items ...** opens the part for the ROMs. You reach them in the **Help** menu, in the command palette and in the card **Setup** of the settings. If VintageCoder has found a new emulator since the last setup, the start page shows the card **New finds** with **Set up emulators**.

**Exporting, importing and resetting the configuration** are not part of the assistant. You find them under **Settings > Backup**: **Export configuration …** writes the settings of the IDE (target systems, emulators, machines, choice of help and updates) into a JSON file, without passwords. **Import configuration …** shows what is in it first and then replaces the current settings. **Reset to default …** restores the default. Color scheme and zoom are not included.

### What the assistant cannot do

- It **does not install programs** and downloads nothing (not even ROMs).
- It sets **no key assignment**; the shortcuts are fixed.
- The English interface is machine-translated and not reviewed yet.
- Storing passwords in the **keychain** of the operating system is still missing.
- The automatic emulator search is most thorough on the Mac; on Windows and Linux, the field **Program** in the settings helps.

> **Tip:** For scripts and test runs, the environment variable `VC_NO_WIZARD=1` suppresses the automatic start of the assistant.

## Set up by hand: emulator, ROMs and hardware

Everything the assistant sets, you also find in the **Settings**. The following sections show the manual way, for example for fine points or if you want to change the settings later.

### Step by step: Find and set an emulator

VintageCoder runs your program in an **emulator**, a separate program that you install yourself. VintageCoder does not include an emulator and never downloads anything unasked: where there is a fixed package for your system, the emulator card in the settings offers the button **Install portable (version) …**; the download happens only after your confirmation with **Download now** (with a check of the SHA-256 sum). On Linux see also [Emulators on Linux](kapitel:systeme).

| Target system | Emulator | Note |
|---|---|---|
| C64 | VICE `x64sc` (cycle exact) or `x64` (fast) | |
| C128 | VICE `x128` | |
| C16 / Plus/4 | VICE `xplus4` | |
| VIC-20 | VICE `xvic` | |
| Atari 800 XL/XE | `atari800`, on Windows also Altirra | Altirra manages its firmware by itself |
| MEGA65 | Xemu (`xmega65`) | needs a MEGA65 ROM, see below |
| C64 (hardware) | Ultimate 64 / II+ | not an emulator, see below |

1. **Install an emulator.** For the C64, C128, Plus/4 and VIC-20 you need VICE, for Atari `atari800` or (on Windows) Altirra, for the MEGA65 Xemu. For the debugger, VICE needs **version 3.9 or newer**; on Windows better 3.10 or newer (VICE 3.9 crashes there during single step, running works with 3.9 too).
2. Open the **Settings**: with the gear icon in the header bar, with **Cmd/Ctrl+Comma** or in the menu **View > Settings**.
3. Switch to the page **Targets and hardware** and choose your system on the left under **Target systems**. If it is missing in the list, add it with **Add a system …** and **Add**.
4. On the right, in the card of the emulator, the field **Program (empty = search automatically)** holds the path of the program. **Empty** means: VintageCoder searches by itself in the `PATH` and in the usual folders. The button **Search** enters the path that was found. If it finds nothing, you enter the path by hand. Below it, a line with a check mark or a cross shows whether the emulator was found ("Found: ...", with the path for the automatic search, or "Not found. Enter the path by hand.").
5. With **Test** you start the emulator briefly as a test. The message shows whether it worked.
6. Click **Save** at the bottom. **Discard** takes your changes back. Next to it it says "Unsaved changes" as long as something is open.

![Settings, page Ziele und Hardware (targets and hardware) with the Commodore 64 selected. The Emulatoren card shows VICE x64sc as the default, the program field set to automatisch, the buttons Suchen and Testen, and a green Gefunden (found) message with the path. Below are the collapsed Startparameter, the button Entfernen, Emulator hinzufuegen ... and, under ECHTE HARDWARE, the button to connect an Ultimate 64 / II+ over the network.](bild:einstellungen-emulator "The emulator card: find the program and check it with Testen.")

**Where VintageCoder searches automatically:** in the `PATH` and in addition, for example on macOS, in `/Applications/vice-arm64-gtk3-3.10/bin`, `/Applications/vice-arm64-gtk3-3.9/bin`, `/opt/homebrew/bin` and `/usr/local/bin` (Xemu also under `/Applications/xmega65.app/Contents/MacOS`). If your emulator is somewhere else, enter the path. For Xemu, the environment variable `VC_XEMU` works too.

Some special points:

- A system can have **several emulators**; one of them is the **default** (selection button **Default**). With **Add emulator …** and **Add**, another one of the chosen kind is added.
- VintageCoder appends the **start parameters** (card **Start parameters**) to the command line (see [Build and run](kapitel:bauen-starten)).
- **atari800** can get its own **OS ROM**, **Xemu** the **folder with `MEGA65.ROM`** (field **ROM**). Normally, though, you put the ROMs in the ROM folder (next section).
- The card **Start target** decides where F5 and F6 run: **Emulator (default of the system)** or an entered machine.

#### Ultimate 64 (real hardware)

Under **Real hardware**, you enter a machine with the button **Connect Ultimate 64 / II+**: **Address** (like `192.168.1.50`) and, if set in the firmware, the **Network password**. **Test connection** checks whether it answers; this happens only on click. The connection to a real MEGA65 (USB or Ethernet) is hidden in the packages of this version. Details and the procedure are in the chapter [Build and run](kapitel:bauen-starten).

### ROM files: do you need them?

A **ROM** is the built-in, fixed memory of an old computer: operating system, BASIC, character set. VintageCoder includes **no original ROMs** and downloads nothing from the network. Whoever needs them puts their **own images** (files) in the ROM folder. VintageCoder recognizes them by their **content** (size and checksum); the file name does not matter. Whether you may use an image is for you to clarify.

| What for | Without ROM | With ROM |
|---|---|---|
| VICE, atari800 | run with their own replacement ROMs | use your images |
| Xemu (MEGA65) | does **not** start: needs a `MEGA65.ROM` of exactly 131072 bytes | runs |
| Character set editor, character picker | the own default set of VintageCoder (VC Standard), with a note | original character set of your system |

For the Hello World program with VICE you need **no** ROMs.

#### Step by step: Set up ROMs

1. Start the partial assistant with **Help > Setup assistant: ROMs only ...**, through the command palette (Cmd/Ctrl+K, word "ROMs") or through **Settings > General > Setup > ROMs …**.
2. The partial assistant has three steps: **Target systems**, **ROMs** and **Summary**. In the step **ROMs**, you choose your ROM files or ZIP archives with **Choose ROM file or zip …** or drag them onto the window; a list shows what was recognized, and only **Import n** copies the checked files into the ROM folder. **Open ROM folder** shows the folder.
3. You find the same page under **Settings > ROMs**. For each system (tab with a status light) and role (for example KERNAL, BASIC, character set on the C64), it shows whether a matching file was found, and offers **Open ROM folder**, **Check again**, **Choose ROM file or zip …** and **Search ROMs of other programs**. The switch **Use ROMs of other programs (VICE, Xemu, atari800)** (default off) lets VintageCoder read the ROM folders of these programs too. At the bottom are the **Places searched**.

![Settings, ROMs page. The list of pages is on the left. At the top right are the buttons ROM-Ordner oeffnen and Neu pruefen, below them ROM-Datei oder Zip waehlen, ROMs anderer Programme suchen, and Fundliste kopieren. Tabs for each system (C64 selected) and for each role the status Fehlt with the expected size. The searched places are at the bottom.](bild:einstellungen-roms "The ROMs page shows for each system and role whether a file was found; here everything is still missing.")

**Where the ROMs are:** in the user folder `roms` (macOS `~/.config/vintagecoder/roms`, Windows `%APPDATA%\vintagecoder\roms`), with one subfolder per system (`c64`, `c128`, `vic20`, `plus4`, `atari8`, `mega65`). If you want, a folder `roms/` in the project or next to the program also applies (portable mode). The complete order is in the help page [ROM files](hilfe:roms). On the command line, `vcc roms list` shows which roles are missing.

## The command line tool vcc

Besides the user interface, the tool **`vcc`** is part of the package. You do not need it for everyday work, but it builds projects without the user interface (for example for scripts). Important commands:

```bash
vcc new hallo --target c64 --template asm-hello   # create a new project
vcc build hallo                                   # build the project
vcc new --list                                    # templates and project structures
vcc roms list                                     # ROM roles and their status
```

On Windows, the setup puts `vcc` into the `PATH` only if you check that option there; on macOS, **Kommandozeile installieren.command** links it.

## Finding new versions

VintageCoder can check whether there is a new version. **This never happens without your consent:** as long as you have chosen nothing under **Settings > General > Updates**, it says *Not asked yet* and nothing is sent. You can choose *On request*, *Daily* or *Weekly* (daily and weekly check when VintageCoder starts) and apply it with **Save**.

If there is a newer version, the status bar shows **Update X available**. A click opens **Settings > Info and update** with the **changes** since your version and these buttons: **View changes** (opens the page of the new version in your browser, from where you get it as the first time), **Remind me later** and **Skip this version**. **VintageCoder downloads and installs nothing by itself.** If the new version brings a newer project format, the page warns about it: older versions can then no longer read your projects.

Only a request for the list of versions is sent to `github.com`; GitHub sees your IP address and the program name "VintageCoder-update" and nothing else (no identifier, no information about computer or projects). The same page **Info and update** has the button **Check for updates now**; you also reach it through the menu **Help > About VintageCoder** or **Help > Check for updates ...**. On the command line, `vcc update` shows the same. Details: help "Check for updates". In packages without the update check, the button is gray.

## Example projects

VintageCoder includes finished examples. On the start page, **Open example** opens a dialog with the examples from the folder `examples/` next to the hardware database (for example "c64 / border"); **Open** opens the project, **Copy ...** makes a copy at a place of your choice. The same examples are also on the start page under **Example projects**. On Windows, the setup copies the examples to "Documents\VintageCoder\Beispiele", on the Mac the folder **Beispiele** is in the .dmg. There are examples for all systems, for example `c64/border`, `atari/colorbars` and `mega65/hello`.

## The test: Hello World with F5

Finally, you check that the whole chain works.

1. On the start page, click **New project** (or press Cmd/Ctrl+Alt+N). If you chose **Create a new project …** under "First project" in the setup assistant, the dialog opens right after the step **Done**.
2. Choose the target system **Commodore 64** and the template **Assembler (vcasm)** > **Assembler program**. Give the project a name like `hallo` and choose a location. Click **Create and open project**.

![The dialog 'Neues Projekt erstellen' above the start page. On the left is the list 'ZIELSYSTEM' from Commodore 64 (selected) to MEGA65 with the hardware profile 'gueltig'. In the middle is the 'VORLAGE' with 'Assembler-Programm' (selected), 'Steckmodul' and 'Leeres Projekt'. On the right are the 'PROJEKT-DETAILS' with project name, location, video standard, default emulator, start target, disk and character set. At the bottom are the buttons 'Abbrechen' and 'Projekt erstellen und oeffnen'.](bild:neues-projekt-dialog "The dialog 'Neues Projekt erstellen' with target system, template and project details.")

3. Press **F5**. VintageCoder builds the program and starts VICE. In the emulator, the border runs through the colors in colorful stripes.
4. Close the emulator.

If this works, the setup is finished. If you had the assistant copy the examples, it also works with an example: **Open example**, open an example in the dialog with **Open**, **F5**. A detailed guide with pictures is in [Your first project](kapitel:erstes-projekt).

## Common stumbling blocks

- **The assistant has disappeared or did not open.** It opens by itself only at the very first start. Open it through **Help > Setup assistant ...**, the command palette or **Settings > General > Setup**.
- **The splash screen is annoying.** Switch it off under **Settings > General > Splash screen**.
- **Emulator "Not found" in the settings.** It is not installed or is in an unusual place. Install it and click **Search**, or enter the path.
- **macOS refuses to open it.** Right click on the app and **Open** (Gatekeeper note above).
- **`vcc` is not found in the terminal.** Windows: add the check mark "add to the PATH" in the setup afterwards (run the setup again) and open a new command prompt. macOS: run **Kommandozeile installieren.command**.
- **The MEGA65 emulator does not start.** Xemu needs a `MEGA65.ROM` of exactly 131072 bytes.
- **The character set editor shows the own set of VintageCoder.** The character set ROM of your system is missing; set it up with the partial assistant "ROMs only" or on the page **Settings > ROMs**.
- **Windows debugger crashes.** VICE 3.9 crashes during single step; use 3.10 or newer.

## What next?

Go on with [Welcome to VintageCoder](kapitel:willkommen) if you want an overview, or directly with [Your first project](kapitel:erstes-projekt). The user interface is explained in [The user interface at a glance](kapitel:oberflaeche).
