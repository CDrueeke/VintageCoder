# VintageCoder unter Linux

Linux ist ein Entwicklungsziel (Debian, Ubuntu, Fedora, openSUSE; X11 und Wayland). Fertige Pakete (`.deb` für Debian 13 und Ubuntu 24.04, `.rpm` für Fedora 45 und openSUSE Tumbleweed, dazu ein `tar.gz`, jeweils nur x86_64) beschreibt das Handbuch im Kapitel „Installation“; ein AppImage gibt es nicht. Dieser Text betrifft vor allem das Bauen aus dem Quellbaum. Das Skript `tools/linux-vm-test.sh` richtet eine frische Maschine ein, baut und prüft den Start.

## Voraussetzungen zum Bauen
- Rust (stabil, über [rustup](https://rustup.rs)), ein C-Compiler und `pkg-config`.
- Entwicklungspakete: ALSA (Klang über `cpal`), `xkbcommon`, Wayland, X11 (`X11`, `Xcursor`, `Xrandr`, `Xi`), OpenGL/EGL.

| Distribution | Pakete |
| --- | --- |
| Debian, Ubuntu | `build-essential pkg-config libasound2-dev libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev libegl1-mesa-dev` |
| Fedora | `gcc pkgconf-pkg-config alsa-lib-devel libxkbcommon-devel wayland-devel libX11-devel libXcursor-devel libXrandr-devel libXi-devel mesa-libGL-devel mesa-libEGL-devel` |
| openSUSE | `gcc pkg-config alsa-devel libxkbcommon-devel wayland-devel libX11-devel libXcursor-devel libXrandr-devel libXi-devel Mesa-libGL-devel Mesa-libEGL-devel` |

```bash
cargo build --release -p vc-app -p vc-cli      # Programme: target/release/vintagecoder und vcc
```

## Voraussetzungen zum Laufen
- Eine grafische Sitzung (X11 oder Wayland) mit OpenGL (Mesa genügt, auch Software-Darstellung).
- Diese Bibliotheken lädt das Fenstersystem erst zur Laufzeit; fehlen sie, stürzt der Start ab („Library libxkbcommon-x11.so could not be loaded“) und `ldd` zeigt nichts: `libxkbcommon-x11`, `libGL`, `libEGL`, `libwayland-client`, `libXcursor`, `libXrandr`, `libXi` (Debian/Ubuntu: `libxkbcommon-x11-0 libgl1 libegl1 libwayland-client0 libxcursor1 libxrandr2 libxi6`). Auf normalen Desktops sind sie vorhanden, in Minimal-Installationen und Containern nicht.
- **Datei-Dialoge** gehen über das Desktop-Portal: `xdg-desktop-portal` mit `-gtk` oder `-gnome`/`-kde` muss laufen (auf üblichen Desktops vorhanden). In schlanken Fenstermanagern fehlt es oft.
- **Klang** über ALSA; mit PipeWire oder PulseAudio genügt `pipewire-alsa` bzw. `pulseaudio-alsa`.
- **Papierkorb**: Dateien werden mit `gio trash` (Paket `glib2`/`libglib2.0-bin`) oder `trash-put` (trash-cli) in den Papierkorb gelegt; fehlt beides, fragt der Dialog vor dem endgültigen Löschen.
- **Ordner zeigen** („Im Dateimanager zeigen“, ROM-Ordner öffnen) nutzt `xdg-open` (Paket `xdg-utils`).
- **Fensterknöpfe**: Fehlt dem Fenstermanager die Titelleiste (Tiling, Minimal-Setups), schaltet das Kontrollkästchen „Eigene Fensterknöpfe“ im Menü „Fenster“ Knöpfe und Griffzonen zu.

## Emulatoren
VintageCoder sucht im `PATH` und zusätzlich in `/usr/games`, `/usr/local/games`, `/usr/local/bin`, `/snap/bin`, `/opt/vice/bin`, `/opt/xemu`, `~/.local/bin`, `~/bin`. Andere Orte: Pfad in den Einstellungen beim Zielsystem (Emulatoren und Hardware) eintragen.
- **VICE** (`x64sc`, `x128`, `xplus4`, `xvic`): Debian/Ubuntu `sudo apt install vice` (Ubuntu: Repository „universe“), Fedora über RPM Fusion (`sudo dnf install vice`), openSUSE aus dem Emulator-Repository. Für den Debugger VICE 3.9 oder neuer. Die ROMs liegen unter `/usr/lib/vice` bzw. `/usr/share/vice`.
- **atari800**: `sudo apt install atari800` (Programm unter `/usr/games`), `sudo dnf install atari800`, `sudo zypper install atari800`.
- **Xemu** (MEGA65): `xmega65` aus [xemu-project.github.io](https://xemu-project.github.io/) bauen oder entpacken und den Pfad eintragen oder `VC_XEMU` setzen. Flatpak-Programme heißen nach ihrer Kennung und werden nicht automatisch gefunden: Pfad eintragen (z. B. `flatpak run …` über ein Skript).
- **Altirra** gibt es nur unter Windows.

## Orte
- Einstellungen und Layout: `$XDG_CONFIG_HOME/vintagecoder` (nur absolute Pfade), sonst `~/.config/vintagecoder`. Gibt es nur den alten Ordner unter `~/.config`, bleibt er im Gebrauch.
- ROMs: `<Konfigurationsordner>/roms`; Xemus eigene ROM unter `~/.local/share/xemu-lgb/mega65` wird mitgesucht, wenn „ROMs anderer Programme mitbenutzen“ an ist.
- Fenstersymbol und Anwendungskennung `vintagecoder` (`installer/linux/vintagecoder.desktop`) für Taskleiste und Starter unter X11/Wayland.

## MEGA65-Hardware (USB-seriell)
Das Gerät erscheint als `/dev/ttyUSB0` (oder `ttyACM0`). Der Benutzer braucht Zugriff: Gruppe `dialout` (Debian/Ubuntu/Fedora) bzw. `uucp` (openSUSE, Arch), danach neu anmelden. Die Schnittstelle wird mit `stty -F` auf 2 Mbit/s, roh, eingestellt.
