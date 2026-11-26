#!/bin/zsh
# SPDX-License-Identifier: GPL-3.0-or-later
# Erzeugt die Handbuchbilder der neuen Oberfläche (siehe handbuchbilder.mjs). Voraussetzung: `cargo build -p vc-shell --features testbridge`.
# Aufruf: zsh ui/tools/handbuchbilder.sh <Ausgabeordner> [--lang de|en] [kennung …]   – eigener HOME unter /tmp, nie die echten Einstellungen
cd "$(dirname "$0")/.." || exit 1
out=${1:?Ausgabeordner angeben}; shift
if [ "$1" = "--lang" ]; then export HB_SPRACHE=${2:?de oder en}; shift 2; fi   # Sprache der Bilder (Vorgabe de); Beispiel: ... <Ordner> --lang en
pkill -f "target/debug/vintagecoder"; pkill -f "vite --port 1420"; sleep 1
rm -rf /tmp/vc-hb-home; mkdir -p /tmp/vc-hb-home
(VC_CSP=release nohup npx vite --port 1420 --strictPort > /tmp/vite-hb.txt 2>&1 &)
(cd .. && HOME=/tmp/vc-hb-home VC_TEST_SONG_SRC=/tmp/demo/Retro/amusic/music/tune_pokey.vcsong VC_NO_WIZARD=1 VC_TEST_FAKE_DEBUG=1 VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 nohup target/debug/vintagecoder > /tmp/app-hb.txt 2>&1 &)
sleep 7
node tools/handbuchbilder.mjs "$out" /tmp/demo/Retro "$@"   # neutraler Pfad: erscheint in der Kopfzeile der Bilder
# Zweiter Lauf: Erstkonfiguration in einem Kern ohne Einstellungen (Startbild, Einrichtungsassistent)
if [ $# -eq 0 ]; then
  pkill -f "target/debug/vintagecoder"; sleep 1; rm -rf /tmp/vc-hb-home2; mkdir -p /tmp/vc-hb-home2
  (cd .. && HOME=/tmp/vc-hb-home2 VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 nohup target/debug/vintagecoder > /tmp/app-hb2.txt 2>&1 &)
  sleep 7
  HB_MODUS=assistent node tools/handbuchbilder.mjs "$out" /tmp/demo/Retro2
fi
pkill -f "target/debug/vintagecoder"; pkill -f "vite --port 1420"
