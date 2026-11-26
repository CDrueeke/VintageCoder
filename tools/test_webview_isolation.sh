#!/bin/zsh
# SPDX-License-Identifier: GPL-3.0-or-later
# Prüft (macOS ab 14): zwei Konfigurationsordner (VC_CONFIG_DIR) bekommen je einen eigenen Webview-Datenspeicher, derselbe Ordner denselben.
# Startet das Debug-Programm mit Prüfbrücke (kein sichtbares Fenster) und räumt die dabei neu entstandenen Speicher wieder weg.
# Aufruf: zsh tools/test_webview_isolation.sh   (vorher: cargo build -p vc-shell --features testbridge)
cd "$(dirname "$0")/.." || exit 1
[ "$(uname)" = Darwin ] || { echo "übersprungen (nur macOS)"; exit 0; }
(( $(sw_vers -productVersion | cut -d. -f1) >= 14 )) || { echo "übersprungen (macOS 14 nötig)"; exit 0; }
S=~/Library/WebKit/vintagecoder/WebsiteDataStore; mkdir -p "$S"
vorher=$(ls "$S" | sort)
start() { rm -rf /tmp/wvt-$1; mkdir -p /tmp/wvt-$1/cfg; (HOME=/tmp/wvt-$1 VC_CONFIG_DIR=/tmp/wvt-$1/cfg VC_NO_WIZARD=1 VC_TESTBRIDGE=17421 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 target/debug/vintagecoder > /tmp/wvt-$1.log 2>&1 &); sleep 7; pkill -f "target/debug/vintagecoder"; sleep 1; }
start A; nachA=$(ls "$S" | sort); neuA=$(comm -13 <(echo "$vorher") <(echo "$nachA"))
start B; nachB=$(ls "$S" | sort); neuB=$(comm -13 <(echo "$nachA") <(echo "$nachB"))
# A noch einmal: kein weiterer Speicher
start A; nachA2=$(ls "$S" | sort)
fehler=0
[ -n "$neuA" ] && [ -n "$neuB" ] && [ "$neuA" != "$neuB" ] && echo "ok: A und B haben verschiedene Speicher ($neuA | $neuB)" || { echo "FEHLER: A und B teilen sich den Speicher (A=$neuA B=$neuB)"; fehler=1; }
[ "$nachA2" = "$nachB" ] && echo "ok: derselbe Konfigurationsordner nutzt seinen Speicher wieder" || { echo "FEHLER: ein weiterer Speicher entstand"; fehler=1; }
for d in $neuA $neuB; do rm -rf "$S/$d"; done; rm -rf /tmp/wvt-A /tmp/wvt-B
exit $fehler
