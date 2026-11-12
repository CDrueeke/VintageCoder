#!/bin/zsh
# Screenshot der IDE über den Selbsttest-Modus der App.
# Aufruf: shot.sh AUSGABE.png PROJEKT [DATEI] [DEMO] [BILD] [e]
#   PROJEKT  Ordner unter examples/ (z. B. c64/music_demo)      DATEI  im Projekt zu öffnende Datei ("" = keine)
#   DEMO     VC_DEMO-Werte, durch Komma getrennt (z. B. media, build, showmm, showhelp, dlgp, settings, projed, debug,snap, charpick, bp, fold, complete)
#   BILD     Bildnummer, bei der aufgenommen wird (VC_SHOT_AT; Standard 60; Debugger 925, Tracker/Editoren ~220)
#   e        VC_EMBED=1: Editorfenster (Sprite, Zeichensatz, Tracker …) erscheinen im Hauptfenster-Bild
# Umgebungsvariablen: VC_THEME=light|dark, VC_TRK_PLAY=1 (Tracker spielt von selbst, mit Ton), VC_BP_LINE (Zeile für Demo "bp")
# PROJEKT "-": ohne Projekt starten (Willkommensseite). Anpassbar: VC_HOME, VC_KEEP_CONFIG=1 (Einstellungen/Layout behalten), VC_BIN (Programm), VC_DATA (Datenbank). Neutraler Pfad im Ausgabe-Panel: Symlink /tmp/demo/VintageCoder/data -> data.
R=${VC_REPO:-$(cd "$(dirname "$0")/../../.." && pwd)}
B=${VC_BIN:-$HOME/.cache/vintagecoder-target-bilder/debug/vintagecoder}
# Programm aus neutralem Ordner starten (die ROM-Seite zeigt den Programmpfad)
mkdir -p /tmp/demo/bin; [ "$B" -ef /tmp/demo/bin/vintagecoder ] || cp -f "$B" /tmp/demo/bin/vintagecoder; B=/tmp/demo/bin/vintagecoder
DATA=${VC_DATA:-/tmp/demo/VintageCoder/data}
[ -e "$DATA" ] || { mkdir -p /tmp/demo/VintageCoder && ln -sfn $R/data /tmp/demo/VintageCoder/data; }
# Neutrales Zuhause: Einstellungen, Layout und „Zuletzt geöffnet“ liegen unter /tmp/demo/home (keine echten Nutzerpfade im Bild).
export HOME=${VC_HOME:-/tmp/demo/home}; mkdir -p $HOME/.config/vintagecoder
[ -n "$VC_KEEP_CONFIG" ] || rm -f $HOME/.config/vintagecoder/{settings,layout,layout_code,layout_debug,view}.json
mkdir -p /tmp/demo/Retro; for p in c64/rasterbars c64/music_demo atari/colorbars; do ln -sfn $R/examples/$p /tmp/demo/Retro/${p:t}; done
[ -e $HOME/.config/vintagecoder/recent.json ] || python3 - <<PY
import json,time
n=int(time.time()); r="/tmp/demo/Retro"
json.dump([{"path":r+"/rasterbars","name":"C64: rasterbars","pinned":False,"last":n-3600},{"path":r+"/music_demo","name":"C64: music_demo","pinned":False,"last":n-86400},{"path":r+"/colorbars","name":"Atari: colorbars","pinned":False,"last":n-3*86400}],open("$HOME/.config/vintagecoder/recent.json","w"))
PY
OUT=$1; rm -f $OUT; cd $R
PROJ=examples/$2; [ "${2[1]}" = "/" ] && PROJ=$2
[ -n "$VC_KEEP_IMG" ] || rm -f $PROJ/projektbild.png   # die IDE legt das Projektbild beim Bauen an: nie im Baum zeigen
ARG=examples/$2; [ "$2" = "-" ] && ARG=; [ "${2[1]}" = "/" ] && ARG=$2   # absoluter Projektpfad (Handbuch-Beispiele)
env ${6:+VC_EMBED=1} VC_THEME=${VC_THEME:-dark} VC_DATA=$DATA VC_SHOT=$OUT ${3:+VC_OPEN=$3} VC_DEMO="$4" VC_SHOT_AT="${5:-60}" VC_BP_LINE="${VC_BP_LINE:-13}" $B $ARG >/dev/null 2>&1 &
PID=$!
for i in {1..${VC_WAIT:-90}}; do [ -s $OUT ] && { sleep 2; break; }; kill -0 $PID 2>/dev/null || break; sleep 1; done
kill $PID 2>/dev/null; sleep 1; kill -9 $PID 2>/dev/null
[ -n "$VC_KEEP_IMG" ] || rm -f $PROJ/projektbild.png
pkill -f 'x64sc|atari800' 2>/dev/null     # nur eine Emulator-Instanz
[ -s $OUT ] && echo "$OUT OK" || echo "$OUT FEHLT"
