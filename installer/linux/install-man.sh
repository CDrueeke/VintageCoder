#!/bin/sh
# Installiert die man-Seiten von VintageCoder (vcc(1), vcc-asm(1) ...).
#
#   installer/linux/install-man.sh            nach ~/.local/share/man/man1 (ohne Rechte), danach "man vcc"
#   installer/linux/install-man.sh --system   nach /usr/local/share/man/man1 (mit sudo)
#   installer/linux/install-man.sh --remove   die Seiten wieder entfernen (mit oder ohne --system)
#
# Die Seiten liegen fertig unter installer/linux/man/; sie werden aus data/cli/*.json erzeugt (vcc man --all installer/linux/man).
# Ohne Installation: vcc man asm | man -l -
set -eu

here=$(cd "$(dirname "$0")" && pwd)
src="$here/man"
dest="${XDG_DATA_HOME:-$HOME/.local/share}/man/man1"
remove=0

for a in "$@"; do
  case "$a" in
    --system) dest=/usr/local/share/man/man1 ;;
    --remove) remove=1 ;;
    -h|--help) sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "install-man.sh: unbekannte Option $a (siehe --help)" >&2; exit 2 ;;
  esac
done

if [ "$remove" -eq 1 ]; then
  n=0
  for f in "$src"/*.1; do
    b=$(basename "$f")
    if [ -e "$dest/$b.gz" ]; then rm -f "$dest/$b.gz"; n=$((n + 1)); fi
  done
  echo "$n Seiten aus $dest entfernt."
else
  [ -d "$src" ] || { echo "install-man.sh: $src fehlt (vcc man --all installer/linux/man)" >&2; exit 1; }
  mkdir -p "$dest"
  n=0
  for f in "$src"/*.1; do
    gzip -9n -c "$f" > "$dest/$(basename "$f").gz"
    n=$((n + 1))
  done
  echo "$n Seiten installiert nach $dest."
  echo "Mit \"man vcc\" lesen."
  case "$dest" in
    */.local/share/man/man1) echo "Falls man die Seiten nicht findet: MANPATH um $(dirname "$dest") erweitern." ;;
  esac
fi
if command -v mandb >/dev/null 2>&1; then mandb -q "$(dirname "$dest")" 2>/dev/null || true; fi
