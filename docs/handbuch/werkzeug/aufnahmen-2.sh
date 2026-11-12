#!/bin/zsh
# Gruppe 2: Debugger (Kapitel 09), Tests (10), Emulator-Reiter. Beispiel: beispiele/farbwechsel (Kopie nach /tmp/demo/Retro, vorgebaut mit vcc).
D=${0:A:h}; O=${HANDBUCH_ROH:-$D/../bilder/roh}; S=$D/shot.sh; V=${VC_VCC:-$HOME/.cache/vintagecoder-target-bilder/debug/vcc}
export VC_DATA=/tmp/demo/VintageCoder/data   # vcc findet die Hardware-Datenbank sonst nicht (neutrales HOME)
for dir in /tmp/demo/Retro /tmp/demo/Retro2; do rm -rf $dir/farbwechsel; mkdir -p $dir; cp -R $D/beispiele/farbwechsel $dir/farbwechsel; done
sed -i '' 's/== 4", "a == 4"/== 5", "a == 4"/' /tmp/demo/Retro2/farbwechsel/farbwechsel.vctest   # absichtlich falsche Erwartung für tests-ergebnis
for dir in /tmp/demo/Retro /tmp/demo/Retro2; do (cd $dir/farbwechsel && HOME=/tmp/demo/home $V build >/dev/null 2>&1); done
P=/tmp/demo/Retro/farbwechsel; F=farbwechsel.asm; export VC_BP_LINE=17
VC_DBG_TAB=disasm $S $O/debug-ueberblick.png $P $F bp,debug,dbgtab 960
VC_DBG_TAB=memory $S $O/debug-speicher.png $P $F bp,debug,dbgtab 960
VC_DBG_TAB=watch VC_WATCH='a;[ZAEHLER];[$d020]' $S $O/debug-beobachten.png $P $F bp,debug,dbgtab 960
VC_DBG_TAB=breaks VC_POINT='exec,0817,0817,[$fb] == 9' $S $O/debug-bedingung.png $P $F bp,debug,dbgtab 960
VC_DBG_TAB=breaks VC_POINT_LATE=1 VC_POINT='write,00fb,00fb' $S $O/debug-datenhaltepunkt.png $P $F bp,debug,dbgtab 960
VC_DBG_TAB=emulator $S $O/emulator-reiter.png $P $F bp,debug,dbgtab 960
VC_DBG_TAB=live $S $O/live-bild.png $P $F bp,debug,dbgtab 960
VC_DBG_TAB=snapshot $S $O/debug-momentaufnahme.png c64/sprite_demo sprite_demo.asm bp,debug,dbgtab 960
$S $O/tests-panel.png $P $F tests 500
$S $O/tests-ergebnis.png /tmp/demo/Retro2/farbwechsel $F tests 500
$S $O/memory-map.png c64/sprite_demo sprite_demo.asm projbuild,showmm 200
