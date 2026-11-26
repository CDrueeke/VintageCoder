# Emulators

VintageCoder runs and debugs programs in external emulators. The emulators are not part of VintageCoder: they are **not shipped**, you install them yourself or have them downloaded portably on request. Each emulator runs as a separate process; the emulator programs are under their own licenses.

## Overview

<!-- EMULATOR-TABLE -->

<!-- EMULATOR-SECTIONS -->
## Install portably

Where a package exists for your system, VintageCoder downloads a **fixed version** from the project's website on request, checks the SHA-256 sum and unpacks it below VintageCoder (`emulators/<emulator>/<version>` in the configuration folder, on Windows `%APPDATA%\VintageCoder`). Nothing is ever downloaded without your click, and nothing is changed outside this folder.

- **User interface:** Settings > **Targets and hardware** > card of the emulator > **Install portable (version) ...**. The card first names version, size, source and destination; the download happens only after **Download now**, and **Cancel** discards the question. If there are several versions in the catalog, the button offers the one that is not installed yet. The setup assistant shows the same card in the step **Emulators**.

![Settings, page Ziele und Hardware (targets and hardware) with the Commodore 64 selected. The Emulatoren card shows VICE x64sc as the default, the program field set to automatisch, the buttons Suchen and Testen, and a green Gefunden (found) message with the path. Below are the collapsed Startparameter, the button Entfernen, Emulator hinzufuegen ... and, under ECHTE HARDWARE, the button to connect an Ultimate 64 / II+ over the network.](bild:einstellungen-emulator "The emulator card: find the program and check it with Testen.")

- **Command line:** `vcc emulator list`, `vcc emulator status [emulator]`, `vcc emulator install <emulator>[@version]` (with `--dry-run` only show). See [Command line](help:cli).

Where there is no package (for example macOS, or VICE on Linux), the card names the download page or the way through the package manager. Then you enter the path or let it search automatically.

## Several versions

Several versions of an emulator can exist side by side: portably installed ones (each in its own folder) and those that your system brings itself (`PATH`, program folder). Which of them applies is decided by the **Program** in the emulator card (empty = search automatically): empty means search automatically (order see below); a path of your own is always possible. For different projects you can create several entries of the same emulator with different paths; the selection button **Default** chooses the one for F5 and F6.

On the command line, `vcc test --backend vice@3.9` chooses a specific portable version, `vcc emulator install vice@3.9` installs it. Versions are compared by numbers ("3.10" is newer than "3.9"). A new version of an emulator is not offered automatically: the catalog is in the program and grows with new releases of VintageCoder; you can install newer versions yourself and enter them as a path.

## How VintageCoder finds the emulator

1. the field **Program** of the emulator card in the settings (empty = search automatically),
2. a portable version chosen with `@version` (command line only),
3. what the **system** offers: `PATH` and the usual program folders (Windows: `Programme\...`, Linux: `/usr/games`, `~/.local/bin` and others, macOS: Homebrew),
4. last, the **newest portably installed** version.

See also [Settings](help:settings), [Debugger](help:debugger) and [ROM files](help:roms).
