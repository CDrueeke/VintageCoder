# MEGA65 on the device (USB and Ethernet)

> **Unchecked:** The Ethernet way (run, copy, mount) has so far been tested **only against dummy commands**, not on a real MEGA65. Command lines and error messages come from the MEGA65 wiki and GitHub posts. The USB way (own monitor) is covered through Xemu, but has not been checked again on the real device either.

## Two connections
A machine of the type MEGA65 (Settings > **Targets and hardware** > MEGA65 > **Real hardware**, button **Connect MEGA65**; this connection is hidden in the packages of this version) is entered there. The card has the field **Connection** with **USB** and **Ethernet**. The interface sets one connection for each machine; the table shows what it means for each action.

![Settings, page Ziele und Hardware (targets and hardware) with the Commodore 64 selected. The Emulatoren card shows VICE x64sc as the default, the program field set to automatisch, the buttons Suchen and Testen, and a green Gefunden (found) message with the path. Below are the collapsed Startparameter, the button Entfernen, Emulator hinzufuegen ... and, under ECHTE HARDWARE, the button to connect an Ultimate 64 / II+ over the network.](bild:einstellungen-emulator "The emulator card: find the program and check it with Testen.")

| Action | USB | Ethernet |
|---|---|---|
| Run (F5) | yes, own monitor, no third-party tool | `etherload -r` (third-party tool) |
| Copy and mount disks | no (postponed) | `mega65_ftp` (`put`), then `etherload -m` |
| Debug (F6) | yes | no |

The IDE does **not** ship the tools and does not download anything. `etherload` and `mega65_ftp` belong to the package `mega65-tools` (github.com/MEGA65/mega65-tools). Paths (the fields `etherload` and `mega65_ftp`) are set at the machine as soon as **Ethernet** is chosen; empty means: search in the `PATH`.

## Setting up Ethernet (on the device)
1. Core **0.96 or newer**; `etherload` is still considered BETA.
2. DIP switch 2 to ON (newer devices have it set already).
3. Valid **MAC address** (not `00:00:00:00:00:00`): ALT at power on, `1`, section NETWORK, `R`.
4. **Turn on remote access with Shift + £** (the LED blinks green/yellow). This lasts only until the next reset or power off.
5. PC and MEGA65 in the same network, safest directly by cable or through a simple switch (IPv6 link-local). Wi-Fi and company networks often block this.
6. In the field **Network interface of the computer**, enter the interface of the PC (macOS for example `en0`, Linux `eth0`, Windows the index).

**Security:** Remote access is open to everyone in the same network (control and SD card). Do not turn it on in networks you do not trust.

## Running a program
F5 with the machine as run target. Over Ethernet, `etherload -r <program>` runs (loads and starts; without the switch **Run RUN after the transfer** it only loads). The output of the tool is in the log. If the device does not answer, the message reminds you of remote access (Shift+£), DIP switch, MAC address and network.

## Disk to the SD card, mount, run
- **Disk editor:** for a `.d81`, "Copy to MEGA65" appears: copying with `mega65_ftp` into the **folder on the SD card** (field of the machine; empty = top level; the folder is created) and mounting with `etherload -m`. The button is not yet available in the Disk editor of the new interface.
- **F5 with a disk:** If the project has a `.d81` disk with the start program first and Run **and** Copy are set to Ethernet, the disk is copied, mounted and the program is started.
- Use file names without spaces; `mega65_ftp` gets the commands as lines.

## Testing the connection
The button **Test connection** of the machine checks the chosen connection. USB checks whether the monitor answers; Ethernet checks whether both tools are found and `mega65_ftp` can read the directory.

## Adjusting command lines
The options of the tools are in a data file (`data/mega65/tools.json`, built in). A file `mega65-tools.json` in the configuration folder (`~/.config/vintagecoder`, Windows `%APPDATA%\vintagecoder`) replaces it, in case options of the tools change or turn out different on the device. Placeholders: `{iface}`, `{file}`, `{image}`, `{dir}`, `{localdir}`, `{name}`; `{iface_args}` is dropped if no interface is entered.

## Open until the first test on the device
Exact options and interface notation of `etherload` and `mega65_ftp`, behavior when the SD folder already exists, whether `etherload -m ... -r ...` does mounting and starting in one call, messages of the tools on errors. Own transfer over USB: postponed.
