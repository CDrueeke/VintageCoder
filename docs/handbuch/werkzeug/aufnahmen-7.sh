#!/bin/zsh
# Gruppe 7: Startbildschirm und Konfigurationsassistent (Schnelleinrichtung, Zusammenfassung, Wiederaufruf).
# Ohne Projekt, neutrales HOME (shot.sh löscht settings.json: der Assistent sieht einen „frischen“ Rechner).
D=${0:A:h}; O=${HANDBUCH_ROH:-$D/../bilder/roh}; S=$D/shot.sh
export VC_DATA=/tmp/demo/VintageCoder/data
W="VC_WIN=1500x950"
# Startbildschirm (Fenster etwas höher als der Karteninhalt)
env VC_WIN=1400x860 $S $O/start-splash.png - "" splash 60
rm -f /tmp/demo/home/.config/vintagecoder/{settings,wizard}.json
ln -sfn $D/../../../examples /tmp/demo/VintageCoder/examples   # Beispiele neben der Hardware-Datenbank (Schritt „Projekte“)
# Assistent, Schnelleinrichtung: Willkommen, Design, Zielsysteme, Toolchain, Projekte, Zusammenfassung, Fertig
for st in welcome:weg-waehlen design:design systems:systeme toolchain:toolchain projects:projekte summary:zusammenfassung done:fertig; do
  env $W VC_WIZ_MODE=quick VC_WIZ_STEP=${st%%:*} $S $O/assistent-${st##*:}.png - "" "" 80
done

# Wiederaufruf mit Statusmarken: gespeicherter Stand mit einem verschwundenen Emulator-Pfad (Problem) und einem offenen Thema
C=/tmp/demo/home/.config/vintagecoder; mkdir -p $C
cat > $C/settings.json <<J
{"schema_version":1,"projects_dir":"/tmp/demo/home/VintageCoder","targets":[{"system":"c64","start_on":"","emulators":[{"id":"x64sc","kind":"x64sc","path":"/Applications/vice-alt/bin/x64sc","standard":true}]},{"system":"atari8","start_on":"","emulators":[{"id":"atari800","kind":"atari800","standard":true}]}],"machines":[]}
J
echo '{"version":1,"first_run_done":true,"last_run":"2026-11-01","mode":"full","later":["roms"]}' > $C/wizard.json
env $W VC_KEEP_CONFIG=1 VC_WIZ_MODE=recall VC_WIZ_STEP=welcome $S $O/assistent-wiederaufruf.png - "" "" 80
rm -f $C/settings.json $C/wizard.json

# Include per Drag & Drop (VC_DEMO=dropdrag): Demo-Projekt = rasterbars mit zusätzlicher data/level.bin
P=/tmp/demo/dropdemo; rm -rf $P; cp -R $D/../../../examples/c64/rasterbars $P; mkdir $P/data; head -c 32 /dev/zero > $P/data/level.bin
export VC_WIN=1400x860
env VC_DRAG_FROM=140,258 VC_DRAG_TO=700,482 VC_DRAG_HOLD=100000 $S $O/include-ziehen.png $P rasterbars.asm dropdrag 90
env VC_DRAG_FROM=140,258 VC_DRAG_TO=700,482 $S $O/include-eingefuegt.png $P rasterbars.asm dropdrag 100
env VC_DRAG_FROM=140,372 VC_DRAG_TO=700,482 VC_DRAG_HOLD=100000 $S $O/include-abgelehnt.png $P rasterbars.asm dropdrag 90
