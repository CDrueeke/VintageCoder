#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Prueft gebaute Linux-Pakete in einem FRISCHEN Container der Zieldistribution (Konzept Release-Auswahl, Abschnitt 8.4 und 9).
#
#   tools/linux-package-test.sh <distro> <paketordner>        distro: debian | ubuntu | fedora | opensuse
#
# Der Paketordner enthaelt die Ausgabe von `release.py package` (*.deb oder *.rpm, dazu <paket>.files). Schritte:
#   1. Dateiliste jedes Pakets (dpkg-deb -c / rpm -qlp) gegen <paket>.files: nichts Fremdes, nichts fehlt; keine ROMs, keine Emulator-Programme
#   2. Installation ueber den Paketmanager (Abhaengigkeiten werden aufgeloest)
#   3. vcc --version, vc-lsp vorhanden, vcc build eines mitgelieferten Beispiels, vcc new
#   4. mit IDE-Paket: keine fehlenden Bibliotheken (ldd), zeitbegrenzter Start unter xvfb (IDE_WARTEN Sekunden, Standard 10): sie laeuft danach noch
#      (kein Absturz) und ein Fenster erscheint (xwininfo). Bilder oder ein Selbstbeenden gibt es in der neuen Oberflaeche nicht; Ton und Wayland sind nicht Teil der Pruefung.
#      Fuer Vulkan/Software-Darstellung, D-Bus und Portal gibt es das Test-Image tools/docker/Dockerfile.test-ide (nicht Teil der Paket-Build-Images).
#   5. Entfernen: /usr/share/vintagecoder und die Programme sind danach weg
# Werkzeuge: nur Shell und Coreutils (sort, comm, sed, grep); kein awk, diff, find noetig, damit schlanke Basis-Images frisch bleiben.
# Umgebung: NO_RECOMMENDS=1 installiert ohne empfohlene Pakete (schneller). Exit-Code 0 = alles gut.
# Aufruf im Container:  docker run --rm --platform linux/amd64 -v "$PWD/tools":/tools:ro -v "$PWD/build/pakete/debian":/pakete:ro debian:stable bash /tools/linux-package-test.sh debian /pakete
set -uo pipefail
distro="${1:?Aufruf: tools/linux-package-test.sh <debian|ubuntu|fedora|opensuse> <paketordner>}"
dir="$(cd "${2:?Paketordner fehlt}" && pwd)"
export LC_ALL=C.UTF-8 DEBIAN_FRONTEND=noninteractive
fails=0
step() { printf '\n== %s\n' "$*"; }
ok()   { printf 'OK:      %s\n' "$*"; }
bad()  { printf 'FEHLER:  %s\n' "$*"; fails=$((fails + 1)); }

case "$distro" in
  debian|ubuntu) kind=deb; pkgs=("$dir"/*.deb) ;;
  fedora|opensuse) kind=rpm; pkgs=("$dir"/*.rpm) ;;
  *) echo "Unbekannte Distribution: $distro" >&2; exit 2 ;;
esac
[ -e "${pkgs[0]}" ] || { echo "Keine .$kind-Pakete in $dir" >&2; exit 2; }
. /etc/os-release 2>/dev/null && echo "Container: ${PRETTY_NAME:-?} ($(uname -m))"
# cli zuerst: so bleibt die Reihenfolge fuer den Paketmanager stabil
cli=(); ide=()
for p in "${pkgs[@]}"; do case "$(basename "$p")" in vintagecoder-cli*) cli+=("$p") ;; *) ide+=("$p") ;; esac; done
[ ${#cli[@]} -eq 1 ] || { bad "genau ein vintagecoder-cli-Paket erwartet, gefunden ${#cli[@]}"; }
[ ${#ide[@]} -le 1 ] || bad "hoechstens ein vintagecoder-Paket erwartet, gefunden ${#ide[@]}"

# Nur Shell, sort, comm, grep, sed: auf frischen Basis-Images fehlen awk (openSUSE) und diff (Fedora); es werden keine Zusatzpakete installiert.
list_pkg() {   # absolute Pfade der Dateien (ohne Verzeichnisse und Verknuepfungen), sortiert
  if [ "$kind" = deb ]; then
    dpkg-deb -c "$1" | while read -r perm owner size date time name; do
      case "$perm" in -*) name="${name#.}"; printf '%s\n' "$name" ;; esac
    done | LC_ALL=C sort
  else
    rpm -qp --qf '[%{FILEMODES:perms}\t%{FILENAMES}\n]' "$1" | while IFS=$'\t' read -r perm path; do
      case "$perm" in -*) printf '%s\n' "$path" ;; esac
    done | LC_ALL=C sort
  fi
}

step "1. Dateilisten gegen die Staging-Liste"
for p in "${pkgs[@]}"; do
  exp="$p.files"; got="$(mktemp)"
  list_pkg "$p" > "$got"
  if [ ! -f "$exp" ]; then bad "$(basename "$p"): $(basename "$exp") fehlt"; continue; fi
  LC_ALL=C sort "$exp" > /tmp/expected.txt
  LC_ALL=C comm -23 /tmp/expected.txt "$got" | sed 's/^/  fehlt im Paket: /' > /tmp/diff.txt
  LC_ALL=C comm -13 /tmp/expected.txt "$got" | sed 's/^/  fremd im Paket: /' >> /tmp/diff.txt
  if [ ! -s /tmp/diff.txt ] && [ -s "$got" ]; then ok "$(basename "$p"): $(wc -l < "$got") Dateien wie erwartet"; else bad "$(basename "$p"): Dateiliste weicht ab (oder leer)"; head -20 /tmp/diff.txt; fi
  if grep -iE '(^|/)(x64sc|x64|x128|xplus4|xvic|xpet|xcbm2|xcbm5x0|atari800|altirra|altirra64|fujisan|xmega65|xemu)(\.exe)?$' "$got"; then bad "$(basename "$p"): Emulator-Programm im Paket"; fi
  if grep -iE '\.(rom|crt)$' "$got"; then bad "$(basename "$p"): ROM-verdaechtige Datei"; fi
  if grep -E '/data/roms/' "$got" | grep -vE '/data/roms/(known\.json|README\.md)$'; then bad "$(basename "$p"): unerwartete Datei unter data/roms"; fi
  if ! grep -qE '^/usr/' "$got" || grep -vE '^/usr/' "$got" | grep -q .; then bad "$(basename "$p"): Dateien ausserhalb von /usr"; fi
done

step "2. Installation"
install_cmd() {
  case "$distro" in
    # Die Docker-Basis-Images lassen Dokumentation weg (dpkg: path-exclude=/usr/share/doc/*, dnf: tsflags=nodocs). Das waere ein Fehlalarm bei den Lizenzdateien
    # (Schritt 3): im Wegwerf-Container wird der Ausschluss aufgehoben, damit der Test die Dateien des Pakets prueft und nicht die Einstellung des Images.
    debian|ubuntu) sed -i '\#path-exclude=/usr/share/doc/#d' /etc/dpkg/dpkg.cfg.d/* 2>/dev/null
                   apt-get update -qq && apt-get install -y ${NO_RECOMMENDS:+--no-install-recommends} "${pkgs[@]}" ;;
    fedora)        dnf install -y --setopt=tsflags= ${NO_RECOMMENDS:+--setopt=install_weak_deps=False} "${pkgs[@]}" ;;
    # openSUSE: das Basis-Image setzt rpm.install.excludedocs = yes (/usr/etc/zypp/zypp.conf.d/excludedocs.conf): ebenfalls aufheben (sonst fehlen die Lizenzdateien)
    opensuse)      rm -f /usr/etc/zypp/zypp.conf.d/excludedocs.conf
                   zypper --non-interactive --no-gpg-checks install -y ${NO_RECOMMENDS:+--no-recommends} "${pkgs[@]}" ;;
  esac
}
if install_cmd > /tmp/install.log 2>&1; then ok "Pakete installiert"; else bad "Installation fehlgeschlagen"; tail -30 /tmp/install.log; echo "Ergebnis: $fails Fehler"; exit 1; fi
tail -5 /tmp/install.log

step "3. Kommandozeile"
if [ "$kind" = deb ]; then ver="$(dpkg-deb -f "${cli[0]}" Version)"; else ver="$(rpm -qp --qf '%{VERSION}' "${cli[0]}")"; fi
core="${ver%%[~-]*}"
out="$(vcc --version 2>&1)"; case "$out" in *"$core"*) ok "vcc --version: $out" ;; *) bad "vcc --version zeigt '$out', erwartet $core" ;; esac
command -v vc-lsp > /dev/null && ok "vc-lsp: $(command -v vc-lsp)" || bad "vc-lsp fehlt"
if ldd /usr/bin/vcc 2>&1 | grep -q 'not found'; then bad "vcc: fehlende Bibliotheken"; ldd /usr/bin/vcc | grep 'not found'; else ok "vcc: ldd ohne fehlende Bibliotheken"; fi
[ -d /usr/share/vintagecoder/data ] && ok "Daten: /usr/share/vintagecoder/data" || bad "/usr/share/vintagecoder/data fehlt"
[ -f /usr/share/vintagecoder/data/release.json ] && ok "release.json vorhanden" || bad "release.json fehlt"
for f in LICENSE LICENSING.md LICENSES/MIT-0.txt; do [ -f "/usr/share/doc/vintagecoder/$f" ] && ok "Lizenzdatei /usr/share/doc/vintagecoder/$f" || bad "/usr/share/doc/vintagecoder/$f fehlt"; done
vcc --license 2>&1 | grep -q 'LICENSING.md' && ok "vcc --license nennt LICENSING.md" || bad "vcc --license fehlt oder ohne Verweis"
export HOME="$(mktemp -d)" XDG_CONFIG_HOME="$HOME/cfg" XDG_DATA_HOME="$HOME/data"
cd "$HOME"
rm -rf work && mkdir work
if [ -d /usr/share/vintagecoder/examples ]; then
  cp -a /usr/share/vintagecoder/examples work/examples
  proj=""; shopt -s globstar nullglob                       # statt find (fehlt in schlanken Basis-Images moeglicherweise)
  for f in work/examples/**/*.vcprj; do case "$f" in */common/*) continue ;; esac; proj="$f"; break; done
  if [ -n "$proj" ]; then
    if vcc build "$(dirname "$proj")" > build.log 2>&1; then ok "vcc build $(dirname "$proj")"; else bad "vcc build $(dirname "$proj")"; tail -10 build.log; fi
  else echo "Hinweis: kein Beispielprojekt im Paket (Auswahl ohne Beispiele?)"; fi
else echo "Hinweis: keine Beispiele im Paket"; fi
target="$(grep -A1 '"targets"' /usr/share/vintagecoder/data/release.json | tail -1 | tr -d ' ",')"
if [ -n "$target" ]; then
  if vcc new "$HOME/work/neu" --target "$target" > new.log 2>&1 && vcc build "$HOME/work/neu" >> new.log 2>&1; then ok "vcc new + build (Ziel $target)"; else bad "vcc new/build (Ziel $target)"; tail -10 new.log; fi
fi

step "4. IDE"
if [ ${#ide[@]} -eq 1 ]; then
  if ldd /usr/bin/vintagecoder 2>&1 | grep -q 'not found'; then bad "vintagecoder: fehlende Bibliotheken"; ldd /usr/bin/vintagecoder | grep 'not found'; else ok "vintagecoder: ldd ohne fehlende Bibliotheken"; fi
  [ -f /usr/share/applications/vintagecoder.desktop ] && ok ".desktop-Datei" || bad ".desktop-Datei fehlt"
  [ -f /usr/share/icons/hicolor/512x512/apps/vintagecoder.png ] && ok "Symbol" || bad "Symbol fehlt"
  # Xvfb und Software-Rendering (Mesa-DRI) nachinstallieren: nur fuer den Test im Container, gehoert nicht zu den Paketabhaengigkeiten
  case "$distro" in
    debian|ubuntu) apt-get install -y --no-install-recommends xvfb xauth x11-utils libgl1-mesa-dri mesa-vulkan-drivers > /tmp/xvfb.log 2>&1 ;;
    fedora)        dnf install -y xorg-x11-server-Xvfb xorg-x11-xauth xwininfo mesa-dri-drivers mesa-vulkan-drivers > /tmp/xvfb.log 2>&1 ;;
    opensuse)      zypper --non-interactive install -y xvfb-run xauth xwininfo Mesa-dri > /tmp/xvfb.log 2>&1 ;;
  esac || { bad "xvfb nicht installierbar"; tail -5 /tmp/xvfb.log; }
  if command -v xvfb-run > /dev/null; then
    proj="${proj:+$(dirname "$proj")}"; proj="${proj:-$HOME/work/neu}"
    # Die Oberflaeche beendet sich nicht von selbst: zeitbegrenzter Start unter Xvfb. Geprueft wird, dass sie nach der Wartezeit noch laeuft (kein Absturz)
    # und, wenn xwininfo da ist, dass ein Fenster "vintagecoder" auf dem Schirm steht. Wiedergabe von Ton und Wayland pruefen wir hier nicht.
    cat > /tmp/ide-start.sh <<'EOS'
#!/bin/sh
vintagecoder "$1" > "$2" 2>&1 &
pid=$!
sleep "$3"
if ! kill -0 "$pid" 2>/dev/null; then wait "$pid"; echo "ABSTURZ (Rueckgabewert $?)"; exit 1; fi
if command -v xwininfo > /dev/null; then
  if xwininfo -root -tree 2>/dev/null | grep -qi vintagecoder; then echo "FENSTER"; else echo "KEIN-FENSTER"; kill "$pid"; exit 3; fi
else echo "LAEUFT"; fi
kill "$pid"; wait "$pid" 2>/dev/null; exit 0
EOS
    res="$(env LIBGL_ALWAYS_SOFTWARE=1 timeout 90 xvfb-run -a -s "-screen 0 1600x1000x24" sh /tmp/ide-start.sh "$proj" "$HOME/ide-start.log" "${IDE_WARTEN:-10}" 2>&1)"; rc=$?
    case "$res" in
      *KEIN-FENSTER*) bad "IDE laeuft, aber kein Fenster erschienen"; tail -15 "$HOME/ide-start.log" ;;
      *FENSTER*) [ $rc -eq 0 ] && ok "IDE startet, Fenster vorhanden (nach ${IDE_WARTEN:-10} s noch am Laufen)" || { bad "IDE-Start: $res"; tail -15 "$HOME/ide-start.log"; } ;;
      *LAEUFT*)  ok "IDE startet und laeuft nach ${IDE_WARTEN:-10} s noch (Fenster nicht geprueft: xwininfo fehlt)" ;;
      *)         bad "IDE-Start fehlgeschlagen: $res"; tail -15 "$HOME/ide-start.log" ;;
    esac
  fi
else
  echo "Kein IDE-Paket in dieser Auswahl: Abschnitt uebersprungen"
  [ -e /usr/bin/vintagecoder ] && bad "/usr/bin/vintagecoder vorhanden, obwohl kein IDE-Paket" || ok "keine IDE-Dateien installiert"
  [ -e /usr/share/applications/vintagecoder.desktop ] && bad ".desktop-Datei ohne IDE" || ok "keine .desktop-Datei"
fi

step "5. Entfernen"
cd /
case "$distro" in
  debian|ubuntu) apt-get remove -y vintagecoder vintagecoder-cli > /tmp/remove.log 2>&1 || apt-get remove -y vintagecoder-cli > /tmp/remove.log 2>&1 ;;
  fedora)        dnf remove -y vintagecoder vintagecoder-cli > /tmp/remove.log 2>&1 || dnf remove -y vintagecoder-cli > /tmp/remove.log 2>&1 ;;
  opensuse)      zypper --non-interactive remove vintagecoder vintagecoder-cli > /tmp/remove.log 2>&1 || zypper --non-interactive remove vintagecoder-cli > /tmp/remove.log 2>&1 ;;
esac
left="$(ls -d /usr/share/vintagecoder /usr/bin/vcc /usr/bin/vc-lsp /usr/bin/vintagecoder /usr/share/doc/vintagecoder 2>/dev/null)"
if [ -z "$left" ]; then ok "alles entfernt"; else bad "nach dem Entfernen noch vorhanden: $left"; tail -5 /tmp/remove.log; fi

printf '\nErgebnis (%s): %d Fehler\n' "$distro" "$fails"
[ "$fails" -eq 0 ]
