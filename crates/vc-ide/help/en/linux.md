# VintageCoder on Linux

Linux is a development target (Debian, Ubuntu, Fedora, openSUSE; X11 and Wayland). Ready-made packages (`.deb` for Debian 13 and Ubuntu 24.04, `.rpm` for Fedora 45 and openSUSE Tumbleweed, plus a `tar.gz`, each x86_64 only) are described in the manual in the chapter "Installation"; there is no AppImage. This text is mainly about building from the source tree. The script `tools/linux-vm-test.sh` sets up a fresh machine, builds and checks the start.

## Requirements for building
- Rust (stable, through [rustup](https://rustup.rs)), a C compiler and `pkg-config`.
- Development packages: ALSA (sound through `cpal`), `xkbcommon`, Wayland, X11 (`X11`, `Xcursor`, `Xrandr`, `Xi`), OpenGL/EGL.

| Distribution | Packages |
| --- | --- |
| Debian, Ubuntu | `build-essential pkg-config libasound2-dev libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev libegl1-mesa-dev` |
| Fedora | `gcc pkgconf-pkg-config alsa-lib-devel libxkbcommon-devel wayland-devel libX11-devel libXcursor-devel libXrandr-devel libXi-devel mesa-libGL-devel mesa-libEGL-devel` |
| openSUSE | `gcc pkg-config alsa-devel libxkbcommon-devel wayland-devel libX11-devel libXcursor-devel libXrandr-devel libXi-devel Mesa-libGL-devel Mesa-libEGL-devel` |

```bash
cargo build --release -p vc-gui -p vc-cli      # programs: target/release/vintagecoder and vcc
```

## Requirements for running
- A graphical session (X11 or Wayland) with OpenGL (Mesa is enough, also software rendering).
- The window system loads these libraries only at run time; if they are missing, the start crashes ("Library libxkbcommon-x11.so could not be loaded") and `ldd` shows nothing: `libxkbcommon-x11`, `libGL`, `libEGL`, `libwayland-client`, `libXcursor`, `libXrandr`, `libXi` (Debian/Ubuntu: `libxkbcommon-x11-0 libgl1 libegl1 libwayland-client0 libxcursor1 libxrandr2 libxi6`). On normal desktops they are present, in minimal installations and containers they are not.
- **File dialogs** go through the desktop portal: `xdg-desktop-portal` with `-gtk` or `-gnome`/`-kde` must be running (present on common desktops). In lean window managers it is often missing.
- **Sound** through ALSA; with PipeWire or PulseAudio, `pipewire-alsa` or `pulseaudio-alsa` is enough.
- **Trash**: files are moved to the trash with `gio trash` (package `glib2`/`libglib2.0-bin`) or `trash-put` (trash-cli); if both are missing, the dialog asks before deleting permanently.
- **Show folder** ("Show in file manager", open ROM folder) uses `xdg-open` (package `xdg-utils`).

## Emulators
VintageCoder searches the `PATH` and also `/usr/games`, `/usr/local/games`, `/usr/local/bin`, `/snap/bin`, `/opt/vice/bin`, `/opt/xemu`, `~/.local/bin`, `~/bin`. Other places: enter the path in the settings under **Targets and hardware** (card "Configured emulators"). Leaving it empty means search automatically. Where a fixed package exists (on Linux atari800 as an AppImage and Fujisan as an archive), **Install portable ...** downloads it after a confirmation and checks the SHA-256 sum. The manual has the section "Emulators on Linux" with steps per distribution in the chapter "Target systems compared".

State of the package sources (query of the default sources in fresh containers, Debian 13, Ubuntu 24.04, Fedora 45, openSUSE Tumbleweed): Debian 13 has VICE 3.9 and atari800 5.2.0 only in the section `contrib`; Ubuntu 24.04 has VICE **3.7.1** and atari800 5.0.0 only in `multiverse`; Fedora 45 has atari800 7.1.2 but no VICE; openSUSE Tumbleweed has neither VICE nor atari800 in oss, non-oss and update. Xemu is in none of these sources (the openSUSE package `xemu` is an Xbox emulator). For `contrib` and `multiverse` you change the package sources of your system.
- **VICE** (`x64sc`, `x128`, `xplus4`, `xvic`): Debian `sudo apt install vice` (section `contrib`), Ubuntu `sudo apt install vice` (section `multiverse`; VICE there is 3.7.1, too old for the debugger). Fedora and openSUSE do not have VICE in the default sources. Whether a package exists for Fedora through RPM Fusion (`sudo dnf install vice`) and for openSUSE in an additional repository is **unchecked**. For the debugger you need VICE 3.9 or newer; otherwise build it yourself or get it from the VICE website. The ROMs are in `/usr/lib/vice` or `/usr/share/vice`.
- **atari800**: `sudo apt install atari800` (Debian: `contrib`, Ubuntu: `multiverse`; program in `/usr/games`), `sudo dnf install atari800` (Fedora). On openSUSE Tumbleweed it is not in the default sources; `sudo zypper install atari800` works only when an additional repository provides it (**unchecked**). Alternatively the AppImage from the project (catalog: 7.2.1 and 7.1.2, x86_64).
- **Xemu** (MEGA65): get `xmega65` from the maker (according to the catalog a `.deb` for Debian/Ubuntu or build it yourself, [github.com/lgblgblgb/xemu](https://github.com/lgblgblgb/xemu/releases)), enter the path or set `VC_XEMU`. Whether the emulators exist as Flatpak or Snap is **unchecked**. Flatpak programs are named after their ID and are not found automatically: enter the path (for example `flatpak run ...` through a script).
- **Altirra** exists only on Windows.

## Locations
- Settings and layout: `$XDG_CONFIG_HOME/vintagecoder` (absolute paths only), otherwise `~/.config/vintagecoder`. If only the old folder under `~/.config` exists, it stays in use.
- ROMs: `<configuration folder>/roms`; the own ROM of Xemu under `~/.local/share/xemu-lgb/mega65` is searched too if "Use ROMs of other programs" is on.
- Window icon and application ID `vintagecoder` (`installer/linux/vintagecoder.desktop`) for the task bar and launcher under X11/Wayland.

## MEGA65 hardware (USB serial)
The device appears as `/dev/ttyUSB0` (or `ttyACM0`). The user needs access: group `dialout` (Debian/Ubuntu/Fedora) or `uucp` (openSUSE, Arch), then log in again. The interface is set with `stty -F` to 2 Mbit/s, raw.
