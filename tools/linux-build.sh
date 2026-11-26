#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Im Bau-Container (Image vc-build-<distro>, tools/docker/): Programme bauen, Linux-Staging erzeugen und Pakete bauen.
#
#   tools/linux-build.sh <distro> [ausgabeordner]      distro: debian | ubuntu | fedora | opensuse (Standard Ausgabe: /out)
#
# Umgebung:  FEATURES  Auswahldatei (Standard FEATURES.md)      CARGO_TARGET_DIR  Build-Ordner (z. B. /cache/target)
#            OBERFLAECHE  tauri (Standard) oder egui      npm_config_cache  npm-Zwischenspeicher (z. B. /cache/npm)
# Ablauf:    release.py check -> stage --os linux --build --verify-build -> package --format auto (Paket der Distribution plus tar.gz)
# Alles Weitere steht in tools/release.py (--help); hier steckt keine eigene Logik.
set -euo pipefail
cd "$(dirname "$0")/.."
# Quellbaum schreibgeschuetzt gemountet (Makefile: -v ...:/src:ro)? Dann arbeitet der Bau in einer Kopie (ui/dist und node_modules brauchen einen beschreibbaren Ordner).
if ! ( : > .vc-schreibtest ) 2>/dev/null; then
  work="$(mktemp -d)/src"; mkdir -p "$work"
  tar -c --exclude=./.git --exclude=./target --exclude=./build --exclude=node_modules --exclude=./ui/dist . | tar -x -C "$work"
  cd "$work"
  # release.py check/stage fragen git (git ls-files): die Kopie bekommt ein frisches Repository (die .git des Quellbaums kann ein Worktree-Verweis sein)
  git config --global --add safe.directory '*' 2>/dev/null || true
  git init -q . && git add -A >/dev/null 2>&1
else
  rm -f .vc-schreibtest
fi
distro="${1:?Aufruf: tools/linux-build.sh <debian|ubuntu|fedora|opensuse> [ausgabeordner]}"
out="${2:-/out}"
features="${FEATURES:-FEATURES.md}"
oberflaeche="${OBERFLAECHE:-tauri}"      # tauri (ui/ + vc-shell) oder egui (vc-gui, bis zum Wegfall)
stage="$(mktemp -d)/stage"
python3 tools/release.py check --features "$features"
python3 tools/release.py stage --os linux --out "$stage" --build --verify-build --reproducible --oberflaeche "$oberflaeche" --features "$features"
python3 tools/release.py package --os linux --format auto --distro "$distro" --stage "$stage" --out "$out" --oberflaeche "$oberflaeche" --reproducible
cp "$stage/release-report.md" "$out/release-report-$distro.md"
ls -l "$out"
