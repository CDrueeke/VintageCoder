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
cargo build --release -p vc-gui -p vc-cli      # Programme: target/release/vintagecoder und vcc
```

## Voraussetzungen zum Laufen
- Eine grafische Sitzung (X11 oder Wayland) mit OpenGL (Mesa genügt, auch Software-Darstellung).
- Diese Bibliotheken lädt das Fenstersystem erst zur Laufzeit; fehlen sie, stürzt der Start ab („Library libxkbcommon-x11.so could not be loaded“) und `ldd` zeigt nichts: `libxkbcommon-x11`, `libGL`, `libEGL`, `libwayland-client`, `libXcursor`, `libXrandr`, `libXi` (Debian/Ubuntu: `libxkbcommon-x11-0 libgl1 libegl1 libwayland-client0 libxcursor1 libxrandr2 libxi6`). Auf normalen Desktops sind sie vorhanden, in Minimal-Installationen und Containern nicht.
- **Datei-Dialoge** gehen über das Desktop-Portal: `xdg-desktop-portal` mit `-gtk` oder `-gnome`/`-kde` muss laufen (auf üblichen Desktops vorhanden). In schlanken Fenstermanagern fehlt es oft.
- **Klang** über ALSA; mit PipeWire oder PulseAudio genügt `pipewire-alsa` bzw. `pulseaudio-alsa`.
- **Papierkorb**: Dateien werden mit `gio trash` (Paket `glib2`/`libglib2.0-bin`) oder `trash-put` (trash-cli) in den Papierkorb gelegt; fehlt beides, fragt der Dialog vor dem endgültigen Löschen.
- **Ordner zeigen** („Im Dateimanager zeigen“, ROM-Ordner öffnen) nutzt `xdg-open` (Paket `xdg-utils`).

## Emulatoren
VintageCoder sucht im `PATH` und zusätzlich in `/usr/games`, `/usr/local/games`, `/usr/local/bin`, `/snap/bin`, `/opt/vice/bin`, `/opt/xemu`, `~/.local/bin`, `~/bin`. Andere Orte: Pfad in den Einstellungen unter **Zielsysteme und Hardware** (Karte „Konfigurierte Emulatoren“) eintragen. Leer lassen heißt automatisch suchen. Wo es ein festes Paket gibt (unter Linux atari800 als AppImage und Fujisan als Archiv), lädt **Portabel installieren …** es nach einer Rückfrage und prüft die SHA-256-Summe. Das Handbuch hat im Kapitel „Zielsysteme im Vergleich“ den Abschnitt „Emulatoren unter Linux“ mit Schritten je Distribution.

Stand der Paketquellen (Abfrage der Standardquellen in frischen Containern, Debian 13, Ubuntu 24.04, Fedora 45, openSUSE Tumbleweed): Debian 13 hat VICE 3.9 und atari800 5.2.0 nur in der Sektion `contrib`; Ubuntu 24.04 hat VICE **3.7.1** und atari800 5.0.0 nur in `multiverse`; Fedora 45 hat atari800 7.1.2, aber kein VICE; openSUSE Tumbleweed hat in oss, non-oss und update weder VICE noch atari800. Xemu gibt es in keiner dieser Quellen (das openSUSE-Paket `xemu` ist ein Xbox-Emulator). Für `contrib` und `multiverse` änderst du die Paketquellen deines Systems.
- **VICE** (`x64sc`, `x128`, `xplus4`, `xvic`): Debian `sudo apt install vice` (Sektion `contrib`), Ubuntu `sudo apt install vice` (Sektion `multiverse`; dort ist VICE 3.7.1, für den Debugger zu alt). Fedora und openSUSE haben VICE nicht in den Standardquellen. Ob es bei Fedora über RPM Fusion (`sudo dnf install vice`) und bei openSUSE in einem Zusatzrepository ein Paket gibt, ist **ungeprüft**. Für den Debugger VICE 3.9 oder neuer; sonst selbst bauen oder von der VICE-Seite besorgen. Die ROMs liegen unter `/usr/lib/vice` bzw. `/usr/share/vice`.
- **atari800**: `sudo apt install atari800` (Debian: `contrib`, Ubuntu: `multiverse`; Programm unter `/usr/games`), `sudo dnf install atari800` (Fedora). Bei openSUSE Tumbleweed steht es nicht in den Standardquellen; `sudo zypper install atari800` gilt erst, wenn ein Zusatzrepository es liefert (**ungeprüft**). Alternativ die AppImage vom Projekt (Katalog: 7.2.1 und 7.1.2, x86_64).
- **Xemu** (MEGA65): `xmega65` beim Hersteller besorgen (laut Katalog ein `.deb` für Debian/Ubuntu oder Selbstbauen, [github.com/lgblgblgb/xemu](https://github.com/lgblgblgb/xemu/releases)), Pfad eintragen oder `VC_XEMU` setzen. Ob es die Emulatoren als Flatpak oder Snap gibt, ist **ungeprüft**. Flatpak-Programme heißen nach ihrer Kennung und werden nicht automatisch gefunden: Pfad eintragen (z. B. `flatpak run …` über ein Skript).
- **Altirra** gibt es nur unter Windows.

## Orte
- Einstellungen und Layout: `$XDG_CONFIG_HOME/vintagecoder` (nur absolute Pfade), sonst `~/.config/vintagecoder`. Gibt es nur den alten Ordner unter `~/.config`, bleibt er im Gebrauch.
- ROMs: `<Konfigurationsordner>/roms`; Xemus eigene ROM unter `~/.local/share/xemu-lgb/mega65` wird mitgesucht, wenn „ROMs anderer Programme mitbenutzen“ an ist.
- Fenstersymbol und Anwendungskennung `vintagecoder` (`installer/linux/vintagecoder.desktop`) für Taskleiste und Starter unter X11/Wayland.

## MEGA65-Hardware (USB-seriell)
Das Gerät erscheint als `/dev/ttyUSB0` (oder `ttyACM0`). Der Benutzer braucht Zugriff: Gruppe `dialout` (Debian/Ubuntu/Fedora) bzw. `uucp` (openSUSE, Arch), danach neu anmelden. Die Schnittstelle wird mit `stty -F` auf 2 Mbit/s, roh, eingestellt.
