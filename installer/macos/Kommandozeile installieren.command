#!/bin/bash
# Verlinkt vcc (und vc-lsp) aus der App nach /usr/local/bin, damit sie im Terminal überall verfügbar sind.
set -e
APP="/Applications/VintageCoder.app"
[ -d "$APP" ] || { echo "VintageCoder.app liegt nicht in /Programme (Applications). Bitte zuerst dorthin ziehen."; read -r -p "Enter zum Beenden"; exit 1; }
for t in vcc vc-lsp; do
  if ! { mkdir -p /usr/local/bin && ln -sf "$APP/Contents/MacOS/$t" "/usr/local/bin/$t"; } 2>/dev/null; then
    echo "Administratorrechte nötig für /usr/local/bin ..."
    sudo mkdir -p /usr/local/bin && sudo ln -sf "$APP/Contents/MacOS/$t" "/usr/local/bin/$t"
  fi
  echo "verlinkt: /usr/local/bin/$t"
done
echo; echo "Fertig. Neues Terminalfenster öffnen und 'vcc --version' ausprobieren."
read -r -p "Enter zum Beenden"
