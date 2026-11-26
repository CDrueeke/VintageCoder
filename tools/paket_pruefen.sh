#!/bin/zsh
# SPDX-License-Identifier: GPL-3.0-or-later
# Prüft eine gebaute Anwendung vor der Weitergabe als Paket: das Frontend muss in der Programmdatei stecken (Bau mit `tauri build`, also mit custom-protocol).
# Fehlt es, lädt die App die Oberfläche vom Entwicklungsserver (localhost:1420) und ignoriert Startparameter. Aufruf: zsh tools/paket_pruefen.sh <Programmdatei oder .app> [--start]
# Mit --start läuft zusätzlich tools/check_app_start.py gegen die Programmdatei (eigenes HOME).
exe="$1"; [ -d "$exe" ] && exe="$exe/Contents/MacOS/vintagecoder"
[ -f "$exe" ] || { echo "Programmdatei nicht gefunden: $1"; exit 2; }
n=$(strings -a "$exe" | grep -c "assets/index-")
if [ "$n" -lt 1 ]; then echo "FEHLER: Das Frontend steckt nicht in $exe (kein assets/index-…): mit 'tauri build' bauen, nicht mit 'cargo build'."; exit 1; fi
echo "ok: Frontend ist eingebettet ($n Fundstellen)"
[ "$2" = "--start" ] && exec python3 "$(dirname "$0")/check_app_start.py" "$exe"
exit 0
