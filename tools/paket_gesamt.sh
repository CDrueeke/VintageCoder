#!/bin/zsh
# SPDX-License-Identifier: GPL-3.0-or-later
# Baut das Gesamtpaket gesamt-0.3.0-vorab aus dem aktuellen Stand (eine App, je Startskript eigenes HOME)
set -e
R="$(cd "$(dirname "$0")/.." && pwd)"
P="${PAKET_DIR:?Zielordner des Pakets in PAKET_DIR angeben}"
cd $R/crates/vc-shell && CARGO_INCREMENTAL=0 ../../ui/node_modules/.bin/tauri build --config tauri.release.conf.json 2>&1 | tail -2
cd $R
mkdir -p $P; rm -rf $P/VintageCoder.app $P/beispiele; cp -R target/release/bundle/macos/VintageCoder.app $P/VintageCoder.app
mkdir -p $P/beispiele
for d in c64/jumprun c64/music_demo c64/sprite_demo c64/gfx_demo c64/tiles_demo atari/music_demo atari/pmg_ship mega65/pal_demo c64/standard_charsets; do mkdir -p $P/beispiele/$(dirname $d); cp -R examples/$d $P/beispiele/$d; done
cp -R examples/common $P/beispiele/ 2>/dev/null || true
cp -R tools/paket-vorlagen/c64/basic_test $P/beispiele/c64/basic_test   # Testprojekt für Rundgang 47 bis 49 (nur im Paket, nicht in examples/)
cp docs/rundgang-0.3.0.md $P/RUNDGANG.md; mkdir -p $P/zusatz; cp docs/rundgang-zusatz-0.3.0.md $P/ZUSATZLISTE.md; cp docs/handpruefung-startseite.md docs/handpruefung-e12.md docs/handpruefung-projekt.md docs/handpruefung-e6.md docs/handpruefung-e13.md docs/handpruefung-basic.md $P/zusatz/ 2>/dev/null; for b in e12-reste projekt-p4 e6-reste e13-markdown; do [ -d "$HOME/Development/abnahme-ui/$b/bilder" ] && { rm -rf "$P/bilder-$b"; cp -R "$HOME/Development/abnahme-ui/$b/bilder" "$P/bilder-$b"; }; done
sed -i "" "s/44 Punkte, etwa 90 Minuten/65 Punkte, etwa 130 Minuten/" $P/LIESMICH.md 2>/dev/null || true
cp tools/paket_pruefen.sh tools/check_app_start.py $P/
grep -q "CBM-II (CBM2)" $P/LIESMICH.md 2>/dev/null || printf "\n## Hinweis\nCBM-II (CBM2) ist als Zielsystem entfallen: Beispiele, Vorlage und Chips gibt es nicht mehr; alte Projekte mit Ziel cbm2 melden beim Bauen „target-cbm2-entfernt“.\n" >> $P/LIESMICH.md
cat > $P/_start.sh <<'EOT'
#!/bin/zsh
# Gemeinsamer Start: eigener Einstellungs- und Projektordner je Variante (home/<name>), echte Einstellungen bleiben unberührt.
# Aufruf: _start.sh <name> <leer|kopie> [Projekt relativ zu beispiele/] [--open Datei] [--view Ansicht]
cd "$(dirname "$0")" || exit 1
name=$1; seed=$2; proj=$3; shift 3
H="$PWD/home/$name"
if [ ! -d "$H" ]; then
  mkdir -p "$H/config" "$H/VintageCoder"
  [ "$seed" = kopie ] && [ -f "$HOME/.config/vintagecoder/settings.json" ] && cp "$HOME/.config/vintagecoder/settings.json" "$H/config/settings.json" \
    && sed -i.bak 's/"language": *"[a-z-]*"/"language": "de"/' "$H/config/settings.json" && rm -f "$H/config/settings.json.bak"   # Prüfung auf Deutsch, auch wenn die echte Einstellung Englisch ist
fi
# Beispielprojekte als Arbeitskopien (nur wenn noch nicht vorhanden)
for d in beispiele/*/*; do n=${d:h:t}-${d:t}; [ -d "$H/VintageCoder/$n" ] || cp -R "$d" "$H/VintageCoder/$n"; done
args=()
[ -n "$proj" ] && [ "$proj" != - ] && args+=("$H/VintageCoder/${proj:h:t}-${proj:t}")
while [ $# -gt 0 ]; do
  case "$1" in --open) args+=(--open "$2"); shift 2;; --view) args+=(--view "$2"); shift 2;; *) shift;; esac
done
export HOME="$H" VC_CONFIG_DIR="$H/config"
exec "$PWD/VintageCoder.app/Contents/MacOS/vintagecoder" "${args[@]}"
EOT
chmod +x $P/_start.sh

mk() { f=$P/start-$1.command; printf '#!/bin/zsh\n# %s\nexec "$(dirname "$0")/_start.sh" %s\n' "$2" "$3" > $f; chmod +x $f; }
mk rundgang "Startseite, eigene Einstellungen (Kopie der echten)" "rundgang kopie -"
mk assistent "Leere Konfiguration: der Einrichtungsassistent muss von selbst kommen" "assistent leer -"
mk assistent-mit-projekt "Leere Konfiguration und Projekt: erst Projekt, dann Assistent" "assistent-proj leer c64/jumprun"
mk editor "Editor, Bauen, Starten" "editor kopie c64/jumprun --open jumprun.asm"
mk tracker-c64 "Tracker (SID)" "tracker kopie c64/music_demo --open music/tune.vcsong"
mk tracker-atari "Tracker (POKEY)" "tracker kopie atari/music_demo --open music/tune_pokey.vcsong"
mk include "Include-Absprung im Code-Editor" "include kopie c64/sprite_demo --open sprite_demo.asm"
mk markdown "Markdown: Vorschau und Schreibhilfen" "markdown kopie c64/jumprun --open README.md"
mk karten "Karten-Editor (Bank mit Levels)" "grafik kopie c64/gfx_demo --open graphics/level.vcscreen"
mk basic "BASIC: #define, #if, #asm (Rundgang 47 bis 49)" "basic kopie c64/basic_test --open test.bas"
mk debug "Debuggen" "debug kopie c64/jumprun --view debug"
mk sprites "Sprite-Editor" "grafik kopie c64/sprite_demo --open graphics/hero.vcsprite"
mk zeichensatz "Zeichensatz" "grafik kopie c64/gfx_demo --open graphics/font.vcchar"
mk palette "Palette" "grafik kopie mega65/pal_demo --open graphics/pal65.vcpal"
mk kacheln "Kacheln" "grafik kopie c64/gfx_demo --open graphics/level_tiles.vctiles"
mk bildschirm "Bildschirm" "grafik kopie c64/gfx_demo --open graphics/title.vcscreen"
mk hex "Hex" "daten kopie c64/jumprun --view daten:hex"
mk speicher "Speicherkarte" "daten kopie c64/jumprun --view daten:speicher"
mk disk "Disk" "daten kopie c64/jumprun --view daten:disk"
mk projekt "Projekt" "projekt kopie c64/jumprun --view projekt"
mk einstellungen "Einstellungen" "einst kopie - "
mk hilfe "Hilfe" "hilfe kopie c64/jumprun --open jumprun.asm"
mk englisch "Englische Oberfläche (Stichprobe)" "englisch leer c64/gfx_demo --open graphics/level.vcscreen"
mkdir -p $P/home/englisch/config $P/home/englisch/VintageCoder
printf '{"schema_version":2,"language":"en","updates":{"check":"never"}}\n' > $P/home/englisch/config/settings.json
cd $R && zsh tools/paket_pruefen.sh $P/VintageCoder.app --start
