#!/usr/bin/env bash
# VintageCoder auf einer frischen Linux-Maschine (VM) einrichten, bauen und prüfen.
# Getestet werden soll auf: Debian (12/13), Ubuntu (24.04 LTS oder neuer), Fedora (aktuell), openSUSE (Leap 16 / Tumbleweed).
# Stand: in Docker-Containern (arm64, ohne Fenstersystem, unter xvfb) gelaufen: Debian 13 (deps, build, test, smoke), Ubuntu 24.04, Fedora 45 und
# openSUSE Tumbleweed (deps, build, smoke). Auf echten VMs mit Desktop-Sitzung noch nicht; x86_64 ebenfalls nicht.
#
#   tools/linux-vm-test.sh deps      Pakete installieren (sudo; erkennt die Distribution über /etc/os-release)
#   tools/linux-vm-test.sh rust      Rust über rustup installieren, falls `cargo` fehlt
#   tools/linux-vm-test.sh build     Release-Build von vintagecoder und vcc
#   tools/linux-vm-test.sh test      cargo test --workspace (Emulator-Tests laufen nur mit VC_EMU_TESTS=1)
#   tools/linux-vm-test.sh smoke     Start prüfen: vcc baut ein Beispiel; die App rendert ein Bild und beendet sich (unter xvfb, falls kein DISPLAY)
#   tools/linux-vm-test.sh emus      Emulatoren (optional) installieren: VICE, atari800 und zeigen, was VintageCoder findet
#   tools/linux-vm-test.sh all       deps, rust, build, test, smoke
#
# Danach von Hand (siehe das Hilfethema „Linux“): Datei-Dialoge, Klang, Papierkorb, Emulatorstart, Fensterverhalten.
set -euo pipefail
cd "$(dirname "$0")/.."

. /etc/os-release 2>/dev/null || { echo "/etc/os-release fehlt – keine Linux-Distribution?" >&2; exit 1; }
FAMILY="${ID_LIKE:-} ${ID}"
case "$FAMILY" in
  *debian*|*ubuntu*) DIST=debian ;;
  *fedora*|*rhel*)   DIST=fedora ;;
  *suse*)            DIST=suse ;;
  *) echo "Unbekannte Distribution: $PRETTY_NAME" >&2; exit 1 ;;
esac
echo "== $PRETTY_NAME ($DIST), $(uname -m), Sitzung: ${XDG_SESSION_TYPE:-keine}"

deps() {
  case "$DIST" in
    debian)
      sudo apt-get update
      sudo apt-get install -y build-essential pkg-config curl git ca-certificates \
        libasound2-dev libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev libxrandr-dev libxi-dev \
        libgl1-mesa-dev libegl1-mesa-dev \
        libxkbcommon-x11-0 libgl1 libegl1 libwayland-client0 libwayland-cursor0 libxcursor1 libxrandr2 libxi6 libx11-xcb1 \
        xdg-utils libglib2.0-bin xvfb xauth alsa-utils \
        xdg-desktop-portal xdg-desktop-portal-gtk ;;
    fedora)
      sudo dnf install -y gcc gcc-c++ make pkgconf-pkg-config curl git \
        alsa-lib-devel libxkbcommon-devel wayland-devel libX11-devel libXcursor-devel libXrandr-devel libXi-devel \
        mesa-libGL-devel mesa-libEGL-devel libxkbcommon-x11 mesa-libGL mesa-libEGL libwayland-client libwayland-cursor libXcursor libXrandr libXi \
        xdg-utils glib2 xorg-x11-server-Xvfb xorg-x11-xauth alsa-utils \
        xdg-desktop-portal xdg-desktop-portal-gtk ;;
    suse)
      sudo zypper --non-interactive install -y gcc gcc-c++ make pkg-config curl git \
        alsa-devel libxkbcommon-devel wayland-devel libX11-devel libXcursor-devel libXrandr-devel libXi-devel \
        Mesa-libGL-devel Mesa-libEGL-devel libxkbcommon-x11-0 Mesa-libGL1 Mesa-libEGL1 libwayland-client0 libwayland-cursor0 libXcursor1 libXrandr2 libXi6 \
        xdg-utils glib2-tools xvfb-run xauth alsa-utils \
        xdg-desktop-portal xdg-desktop-portal-gtk ;;
  esac
}

rust() {
  [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
  if ! command -v rustup >/dev/null 2>&1 && ! command -v cargo >/dev/null 2>&1; then
    curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal
    . "$HOME/.cargo/env"
  fi
  # rustup vorhanden, aber keine Toolchain gewählt (z. B. halb eingerichtete Maschine)
  if command -v rustup >/dev/null 2>&1 && ! cargo --version >/dev/null 2>&1; then rustup default stable; fi
  cargo --version
}
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
BIN="${CARGO_TARGET_DIR:-target}/release"

build() { cargo build --release -p vc-app -p vc-cli; ls -l "$BIN/vintagecoder" "$BIN/vcc"; }

tests() { cargo test --workspace; }

smoke() {
  local out; out="$(mktemp -d)"
  echo "-- Fehlende Laufzeitbibliotheken (ldd; zur Laufzeit geladene wie libxkbcommon-x11 sieht ldd nicht, dafür gibt der Start unten den Fehler):"
  ldd "$BIN/vintagecoder" | grep 'not found' || echo "keine fehlenden"
  echo "-- vcc assembliert ein Beispiel:"
  "$BIN/vcc" asm examples/c64/border/border.asm --target c64 -o "$out/border.prg" && ls -l "$out/border.prg"
  echo "-- App rendert ein Bild und beendet sich (VC_SHOT):"
  local run=(env VC_SHOT="$out/shot.png" "$BIN/vintagecoder" examples/c64/border)
  if [ -z "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]; then run=(xvfb-run -a -s "-screen 0 1600x1000x24" "${run[@]}"); fi
  timeout 60 "${run[@]}" && ls -l "$out/shot.png" || { echo "FEHLER: App startet nicht oder rendert kein Bild"; return 1; }
  echo "-- Audio-Geräte (ALSA):"; (aplay -l 2>&1 || true) | head -5
  echo "Ergebnis in $out"
}

emus() {
  case "$DIST" in
    debian) sudo apt-get install -y vice atari800 ;;
    fedora) sudo dnf install -y atari800 || true; sudo dnf install -y vice || echo "VICE: RPM Fusion nötig (https://rpmfusion.org)" ;;
    suse)   sudo zypper --non-interactive install -y atari800 || true; sudo zypper --non-interactive install -y vice || echo "VICE: Emulator-Repository nötig (zypper ar … games/emulators)" ;;
  esac
  echo "-- Gefunden (PATH und die Zusatzorte von VintageCoder):"
  for p in x64sc x128 xplus4 xvic atari800 xmega65; do
    f="$(command -v "$p" 2>/dev/null || ls /usr/games/$p /usr/local/games/$p /opt/vice/bin/$p /opt/xemu/$p "$HOME/.local/bin/$p" 2>/dev/null | head -1 || true)"
    printf '%-10s %s\n' "$p" "${f:-nicht gefunden}"
  done
  ls -d /usr/lib/vice /usr/share/vice 2>/dev/null || echo "VICE-ROM-Ordner: keiner unter /usr/lib/vice oder /usr/share/vice"
}

case "${1:-}" in
  deps) deps ;; rust) rust ;; build) build ;; test) tests ;; smoke) smoke ;; emus) emus ;;
  all) deps; rust; build; tests; smoke ;;
  *) sed -n '2,14p' "$0"; exit 2 ;;
esac
