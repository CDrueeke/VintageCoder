#!/bin/zsh
# SPDX-License-Identifier: GPL-3.0-or-later
# Läuft alle Ablauftests nacheinander gegen den echten Kern (Prüfbrücke, kein sichtbares Fenster) und frische Projektkopien aus examples/.
# VC_E2E_SLOW=5 verlängert alle Wartezeiten der Tests (langsame Rechner, Container). Voraussetzung: `cargo build -p vc-shell --features testbridge` ist gelaufen und Vite (VC_CSP=release) läuft auf Port 1420. Aufruf: zsh ui/tools/e2e-alle.sh
cd "$(dirname "$0")/.." || exit 1
pkill -f "target/debug/vintagecoder"; sleep 1
rm -rf /tmp/vc-bef /tmp/vc-home /tmp/vc-karten /tmp/vc-en3 /tmp/vc-en4 /tmp/vc-en5 /tmp/vc-en /tmp/vc-en2 /tmp/vc-enneu /tmp/vc-rfr /tmp/vc-md /tmp/vc-inc /tmp/vc-konfig /tmp/vc-dat /tmp/vc-dbg /tmp/vc-abl /tmp/vc-abl-ext /tmp/vc-spr /tmp/vc-disk /tmp/vc-home /tmp/vc-e2e-proj2 /tmp/vc-bau /tmp/vc-lsp /tmp/vc-sitz /tmp/vc-lay /tmp/vc-split /tmp/vc-proj /tmp/vc-pal /tmp/vc-kach /tmp/vc-bild /tmp/vc-trk /tmp/vc-trk-a /tmp/vc-np /tmp/vc-st
printf '{"format":1,"sequence":4,"channel":"stable","published":"2026-12-02T09:00:00Z","latest":"9.9.9","releases":[{"version":"9.9.9","title":"Testversion","notes":"- Neues Zeug\\n- Noch mehr"}]}' > /tmp/vc-manifest.json; mkdir -p /tmp/vc-konfig /tmp/vc-disk /tmp/vc-home /tmp/vc-bau/c64 /tmp/vc-sitz /tmp/vc-np /tmp/vc-st/zielordner
rm -rf /tmp/vc-st; mkdir -p /tmp/vc-st/zielordner   # Zielordner des Kopierdialogs leer: sonst schlägt er nach früheren Läufen „jumprun Kopie 2“ vor
for d in /tmp/vc-bef /tmp/vc-abl /tmp/vc-e2e-proj2 /tmp/vc-lsp /tmp/vc-lay /tmp/vc-split /tmp/vc-proj /tmp/vc-st/A_jumprun /tmp/vc-st/B_jumprun; do cp -R ../examples/c64/jumprun $d; done
cp -R ../examples/c64/jumprun /tmp/vc-dbg; cp -R ../examples/c64/jumprun /tmp/vc-dat; cp -R ../examples/c64/music_demo /tmp/vc-trk; cp -R ../examples/atari/music_demo /tmp/vc-trk-a
cp -R ../examples/c64/gfx_demo /tmp/vc-bild
cp -R ../examples/c64/gfx_demo /tmp/vc-karten
cp -R ../examples/c64/gfx_demo /tmp/vc-kach
cp -R ../examples/mega65/pal_demo /tmp/vc-pal
cp -R ../examples/c64/sprite_demo /tmp/vc-inc; cp -R ../examples/c64/jumprun /tmp/vc-md; cp -R ../examples/c64/jumprun /tmp/vc-en; cp -R ../examples/c64/sprite_demo /tmp/vc-en2; cp -R ../examples/c64/gfx_demo /tmp/vc-en3; cp -R ../examples/mega65/pal_demo /tmp/vc-en4; cp -R ../examples/c64/music_demo /tmp/vc-en5; mkdir -p /tmp/vc-enneu; cp -R ../examples/c64/jumprun /tmp/vc-rfr; cp -R ../examples/c64/sprite_demo /tmp/vc-spr; cp ../examples/c64/sprite_demo/graphics/hero.vcsprite /tmp/vc-spr/graphics/hero2.vcsprite; cp ../examples/atari/pmg_ship/graphics/ship.vcpmg /tmp/vc-spr/graphics/ship.vcpmg; cp ../examples/c64/gfx_demo/graphics/font.vcchar /tmp/vc-spr/graphics/font.vcchar; printf '10 REM DATEN\n20 END\n' > /tmp/vc-spr/daten.bas; printf '.target c64\n.org $0801\nstart: rts\n' > /tmp/vc-spr/leer.asm
cp -R ../examples/c64/jumprun /tmp/vc-sitz/; cp -R ../examples/c64/jumprun /tmp/vc-bau/c64/; [ -d ../examples/common ] && cp -R ../examples/common /tmp/vc-bau/
# Disketten für den Disk-Editor (Fremddateien als Programme auf Abbilder gelegt)
for i in 1 2 3; do head -c $((300 * i * 7)) /dev/urandom > /tmp/vc-disk/f$i.bin; done
cp -R ../examples/c64/jumprun /tmp/vc-disk/proj; [ -d ../examples/common ] && cp -R ../examples/common /tmp/vc-disk/
../target/debug/vcc disk d64 -o /tmp/vc-disk/proj/test.d64 --name TEST --id 01 --add /tmp/vc-disk/f1.bin:EINS --add /tmp/vc-disk/f2.bin:ZWEI --add /tmp/vc-disk/f3.bin:DREI > /dev/null
../target/debug/vcc disk d81 -o /tmp/vc-disk/proj/test.d81 --name GROSS --id 02 --add /tmp/vc-disk/f1.bin:EINS > /dev/null
cp /tmp/vc-disk/proj/test.d64 /tmp/vc-disk/pick.d64; echo kaputt > /tmp/vc-disk/proj/kaputt.d64
[ -n "$VC_ATR_ECHT" ] && [ -f "$VC_ATR_ECHT/MYDOS45D.ATR" ] && cp "$VC_ATR_ECHT/MYDOS45D.ATR" /tmp/vc-disk/proj/mydos.atr   # echtes MyDOS-Abbild (nicht im Repository), wenn der Ordner angegeben ist
(cd .. && HOME=/tmp/vc-home VC_TEST_SONG_SRC=/tmp/vc-trk-a/music/tune_pokey.vcsong VC_NO_WIZARD=1 VC_UPDATE_URL=/tmp/vc-manifest.json VC_TEST_FAKE_DEBUG=1 VC_TEST_KONFIG_OUT=/tmp/vc-konfig/out.json VC_TEST_KONFIG_IN=/tmp/vc-konfig/in.json VC_TEST_PNG=/tmp/vc-spr-a.png VC_TEST_DISK=/tmp/vc-disk/pick.d64 VC_TEST_ADD=/tmp/vc-disk/f1.bin VC_TEST_SAVEAS=/tmp/vc-disk/proj/saved.d64 VC_TEST_DIR=/tmp/vc-st/zielordner VC_TEST_PICK_FILE=/tmp/vc-st/B_jumprun/jumprun.vcprj VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_TEST_FAKE_EMU=6 VC_NO_SAVE=1 nohup target/debug/vintagecoder > /tmp/app.txt 2>&1 &)
sleep 6
run() { n=$1; shift; timeout $((290 * ${VC_E2E_SLOW:-1})) node --import ./tools/e2e-zeit.mjs tools/e2e-$n.mjs "$@" > /tmp/e2e-$n.txt 2>&1; echo "$n: $(grep -E 'Prüfungen' /tmp/e2e-$n.txt | tail -1) $(grep -cE '^FEHLER|Error:' /tmp/e2e-$n.txt) Fehlerzeilen"; }
run start /tmp/vc-st/A_jumprun /tmp/vc-st/B_jumprun /tmp/vc-st/zielordner
run editor /tmp/vc-e2e-proj2
run befehle /tmp/vc-bef
run lsp /tmp/vc-lsp
VC_TEST_FAKE_EMU=6 run bau /tmp/vc-bau
run layout /tmp/vc-lay
run split /tmp/vc-split
run sitzung /tmp/vc-sitz/jumprun /tmp/vc-home
rm -f /tmp/vc-home/.config/vintagecoder/scratch.json   # der Sitzungstest lässt gesicherte Reiter zurück: sie würden den folgenden Tests die Wiederherstellen-Frage zeigen
run disk /tmp/vc-disk/proj
run sprite /tmp/vc-spr
run zeichensatz /tmp/vc-spr
run palette /tmp/vc-pal
run kacheln /tmp/vc-kach
run bildschirm /tmp/vc-bild
run karten /tmp/vc-karten
run include /tmp/vc-inc
run markdown /tmp/vc-md
run rueckfragen /tmp/vc-rfr
VC_E2E_LANG=en run en /tmp/vc-en /tmp/vc-en2 /tmp/vc-enneu /tmp/vc-en3 /tmp/vc-en4 /tmp/vc-en5   # englische Oberfläche: Beschriftungen aus dem Katalog
run tracker /tmp/vc-trk /tmp/vc-trk-a
run debug /tmp/vc-dbg
run daten /tmp/vc-dat
run einstellungen /tmp/vc-konfig
# Einrichtungsassistent (E12.3): eigener Kernstart ohne Einstellungen, ohne VC_NO_WIZARD
pkill -f "target/debug/vintagecoder"; sleep 1; rm -rf /tmp/vc-wiz-home /tmp/vc-wiz-proj; mkdir -p /tmp/vc-wiz-home /tmp/vc-wiz-proj
(cd .. && HOME=/tmp/vc-wiz-home VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 nohup target/debug/vintagecoder > /tmp/app.txt 2>&1 &)
sleep 7
run assistent /tmp/vc-wiz-home /tmp/vc-wiz-proj
# Erststart-Fehlerauftrag: frischer Start, „Später“, Karte „Einrichtung starten“, Rückgängig setzt wizard.json zurück, Start ohne Projekt = Startseite
pkill -f "target/debug/vintagecoder"; sleep 1; rm -rf /tmp/vc-es /tmp/vc-esb /tmp/vc-esp; mkdir -p /tmp/vc-es; cp -R ../examples/c64/jumprun /tmp/vc-esp
(cd .. && HOME=/tmp/vc-es VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 nohup target/debug/vintagecoder > /tmp/app.txt 2>&1 &)
sleep 7
run erststart /tmp/vc-es /tmp/vc-esb /tmp/vc-esp
# Haltepunkte über Neustart: eigener Kernstart (Sitzungsdatei im eigenen HOME)
pkill -f "target/debug/vintagecoder"; sleep 1; rm -rf /tmp/vc-hp /tmp/vc-hph; mkdir -p /tmp/vc-hph; cp -R ../examples/c64/jumprun /tmp/vc-hp
(cd .. && HOME=/tmp/vc-hph VC_NO_WIZARD=1 VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 nohup target/debug/vintagecoder > /tmp/app.txt 2>&1 &)
sleep 7
run haltepunkte /tmp/vc-hp /tmp/vc-hph
# Hilfe (E13): Handbuchbilder nur aus dem Zwischenspeicher (Adresse nie erreichbar)
pkill -f "target/debug/vintagecoder"; sleep 1; rm -rf /tmp/vc-hlp /tmp/vc-hcache /tmp/vc-hhome; mkdir -p /tmp/vc-hhome /tmp/vc-hcache; cp -R ../examples/c64/jumprun /tmp/vc-hlp
(cd .. && HOME=/tmp/vc-hhome VC_NO_WIZARD=1 VC_HELP_IMAGES=https://example.invalid/bilder VC_HELP_CACHE=/tmp/vc-hcache VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 nohup target/debug/vintagecoder > /tmp/app.txt 2>&1 &)
sleep 7
run hilfe /tmp/vc-hlp /tmp/vc-hcache
# Neues Projekt zuletzt (braucht VC_TEST_DIR=/tmp/vc-np: eigener Kernstart)
pkill -f "target/debug/vintagecoder"; sleep 1; rm -rf /tmp/vc-home; mkdir -p /tmp/vc-home
(cd .. && HOME=/tmp/vc-home VC_NO_WIZARD=1 VC_TEST_DIR=/tmp/vc-np VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 nohup target/debug/vintagecoder > /tmp/app.txt 2>&1 &)
sleep 6
run ablegen /tmp/vc-abl /tmp/vc-abl-ext /tmp/vc-disk/pick.d64
run projekt /tmp/vc-proj
run neuprojekt /tmp/vc-np
# ROM-Import (E12): eigener Kernstart mit eigenem ROM-Ordner und festen Auswahldateien
pkill -f "target/debug/vintagecoder"; sleep 1; rm -rf /tmp/vc-rh /tmp/vc-rom-src /tmp/vc-rom-d; mkdir -p /tmp/vc-rh /tmp/vc-rom-src /tmp/vc-rom-d
(cd .. && HOME=/tmp/vc-rh VC_ROMS=/tmp/vc-rom-d VC_NO_WIZARD=1 VC_TEST_ROM_PICK="/tmp/vc-rom-src/zz.bin|/tmp/vc-rom-src/pack.zip" VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 nohup target/debug/vintagecoder > /tmp/app.txt 2>&1 &)
sleep 6
run roms /tmp/vc-rom-src /tmp/vc-rom-d
# Emulator-Installation (E12.2b): Attrappe aus lokaler Zip-Datei, VC_EMU_DIR im Temp-Ordner, nie echtes Netz
pkill -f "target/debug/vintagecoder"; sleep 1; rm -rf /tmp/vc-emu /tmp/vc-fake /tmp/vc-eh; mkdir -p /tmp/vc-emu /tmp/vc-fake /tmp/vc-eh
python3 -c "import zipfile; z=zipfile.ZipFile('/tmp/vc-fake/fake.zip','w'); i=zipfile.ZipInfo('atari800'); i.external_attr=0o755<<16; z.writestr(i,'#!/bin/sh\\necho fake\\n'); z.close()"
SHA=$(shasum -a 256 /tmp/vc-fake/fake.zip | cut -d' ' -f1)
(cd .. && HOME=/tmp/vc-eh VC_EMU_DIR=/tmp/vc-emu VC_NO_WIZARD=1 VC_TEST_EMU_PKG="atari800|9.9.9|file:///tmp/vc-fake/fake.zip|$SHA|zip|atari800" VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 nohup target/debug/vintagecoder > /tmp/app.txt 2>&1 &)
sleep 6
run emuinstall /tmp/vc-emu /tmp/vc-fake
pkill -f "target/debug/vintagecoder"
# Registry (R0): zweite App mit Erweiterungspaket (VC_EXTENSIONS_DEV) und Release-Auswahl (VC_RELEASE); die erste App wird dafür beendet
pkill -f "target/debug/vintagecoder"; sleep 1
rm -rf /tmp/vc-reg /tmp/vc-reg-proj /tmp/vc-reg-home; mkdir -p /tmp/vc-reg/autor.demo /tmp/vc-reg/kaputt /tmp/vc-reg/falsch /tmp/vc-reg-home
printf '{"id":"autor.demo","version":"1.2.3","name":"Demo","commands":[{"id":"hallo","title":{"de":"Hallo sagen","en":"Say hello"},"needs":"none"},{"id":"sprite_nur","title":{"en":"Sprite only"},"feature":"sprite"},{"id":"ohne_titel"}]}' > /tmp/vc-reg/autor.demo/extension.json
mkdir -p /tmp/vc-reg/autor.demo/examples/c64/demobsp /tmp/vc-reg/autor.demo/data/templates; printf '{"project":{"name":"Demo-Beispiel","description":"aus dem Paket"}}' > /tmp/vc-reg/autor.demo/examples/c64/demobsp/demobsp.vcprj; printf '{"structures":[{"id":"demo-struktur","name":"Demo-Struktur","description":"aus dem Paket","dirs":["x"],"layout":{},"roles":{}}]}' > /tmp/vc-reg/autor.demo/data/templates/structures.json
echo '{ kein json' > /tmp/vc-reg/kaputt/extension.json; printf '{"id":"anders","version":"1"}' > /tmp/vc-reg/falsch/extension.json
printf '{"format":1,"studios":["tracker","disk"]}' > /tmp/vc-reg-release.json
cp -R ../examples/c64/jumprun /tmp/vc-reg-proj
(cd .. && HOME=/tmp/vc-reg-home VC_EXTENSIONS_DEV=/tmp/vc-reg VC_RELEASE=/tmp/vc-reg-release.json VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 nohup target/debug/vintagecoder /tmp/vc-reg-proj --open hal_c64.asm --view disk > /tmp/app-reg.txt 2>&1 &)
sleep 6
run registry /tmp/vc-reg-proj
pkill -f "target/debug/vintagecoder"
# BASIC: Projekt-Einstellungen/Bau (e2e-basic) und Umwandeln im Editor (e2e-basicconv); je eine Projektkopie, eigener Kernstart
for t in basic basicconv; do
  pkill -f "target/debug/vintagecoder"; sleep 1; rm -rf /tmp/vc-$t-proj /tmp/vc-$t-home; mkdir -p /tmp/vc-$t-home; cp -R ../examples/c64/basic_native /tmp/vc-$t-proj
  (cd .. && HOME=/tmp/vc-$t-home VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 nohup target/debug/vintagecoder > /tmp/app-$t.txt 2>&1 &)
  sleep 6
  VC_SPRITE="$PWD/../examples/c64/sprite_demo/graphics/hero.vcsprite" run $t /tmp/vc-$t-proj
done
pkill -f "target/debug/vintagecoder"
