#!/usr/bin/env bash
# Im Bau-Container (Image vc-build-<distro>, tools/docker/): Programme bauen, Linux-Staging erzeugen und Pakete bauen.
#
#   tools/linux-build.sh <distro> [ausgabeordner]      distro: debian | ubuntu | fedora | opensuse (Standard Ausgabe: /out)
#
# Umgebung:  FEATURES  Auswahldatei (Standard FEATURES.md)      CARGO_TARGET_DIR  Build-Ordner (z. B. /cache/target)
# Ablauf:    release.py check -> stage --os linux --build --verify-build -> package --format auto (Paket der Distribution plus tar.gz)
# Alles Weitere steht in tools/release.py (--help); hier steckt keine eigene Logik.
set -euo pipefail
cd "$(dirname "$0")/.."
distro="${1:?Aufruf: tools/linux-build.sh <debian|ubuntu|fedora|opensuse> [ausgabeordner]}"
out="${2:-/out}"
features="${FEATURES:-FEATURES.md}"
stage="$(mktemp -d)/stage"
python3 tools/release.py check --features "$features"
python3 tools/release.py stage --os linux --out "$stage" --build --verify-build --reproducible --features "$features"
python3 tools/release.py package --os linux --format auto --distro "$distro" --stage "$stage" --out "$out" --reproducible
cp "$stage/release-report.md" "$out/release-report-$distro.md"
ls -l "$out"
