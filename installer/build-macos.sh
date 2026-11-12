#!/bin/bash
# Erzeugt VintageCoder.app und ein .dmg (macOS) oder, ohne Entwicklungsumgebung, ein Kommandozeilen-Archiv (.tar.gz).
# Der Inhalt kommt aus dem Staging von tools/release.py (Auswahl: FEATURES.md); dieses Skript packt es nur ein.
# Aufruf im Projektordner:
#   installer/build-macos.sh                  Staging erzeugen (mit Bau der Programme) und einpacken
#   installer/build-macos.sh --stage ORDNER   vorhandenes Staging einpacken (python3 tools/release.py stage --os macos --out ORDNER ...)
# Signieren (optional): SIGN_ID="Developer ID Application: Name (TEAMID)" installer/build-macos.sh
#   Notarisieren (optional, nach dem Signieren): NOTARY_PROFILE=<xcrun notarytool store-credentials-Name>
# Ohne SIGN_ID wird ad-hoc signiert (läuft lokal; auf fremden Rechnern: Rechtsklick -> Öffnen).
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT=$PWD
VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
ARCH=$(uname -m)

SRC=""
while [ $# -gt 0 ]; do
  case "$1" in
    --stage) SRC=${2:?--stage braucht einen Ordner}; shift 2 ;;
    -h|--help) sed -n '2,10p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "build-macos.sh: unbekannte Option $1 (siehe --help)" >&2; exit 2 ;;
  esac
done
if [ -z "$SRC" ]; then
  SRC=$(mktemp -d)/stage   # lokal anlegen: auf Netzlaufwerken (SMB) entstehen ._*-Dateien und Attribute, die das Signieren verbieten
  python3 tools/release.py stage --os macos --out "$SRC" --build --force
fi
[ -f "$SRC/data/release.json" ] || { echo "$SRC ist kein Staging (data/release.json fehlt)" >&2; exit 1; }
SRC=$(cd "$SRC" && pwd)

OUT=$ROOT/installer/output; mkdir -p "$OUT"
WORK=$(mktemp -d)

if [ -x "$SRC/bin/vintagecoder" ]; then
  # ---- mit Entwicklungsumgebung: Programmpaket und .dmg
  APP=$WORK/VintageCoder.app
  mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
  cp "$SRC"/bin/* "$APP/Contents/MacOS/"
  sed "s/@VERSION@/$VERSION/g" "$SRC/macos/Info.plist.in" > "$APP/Contents/Info.plist"
  [ -f "$SRC/macos/VintageCoder.icns" ] && cp "$SRC/macos/VintageCoder.icns" "$APP/Contents/Resources/" \
    && /usr/libexec/PlistBuddy -c "Add :CFBundleIconFile string VintageCoder" "$APP/Contents/Info.plist"
  cp "$SRC/LICENSE" "$SRC/THIRD-PARTY-NOTICES.md" "$SRC/README.md" "$APP/Contents/Resources/"
  cp -RX "$SRC/data" "$APP/Contents/Resources/data"
  for d in docs editors; do if [ -d "$SRC/$d" ]; then cp -RX "$SRC/$d" "$APP/Contents/Resources/$d"; fi; done
  find "$APP" \( -name .DS_Store -o -name '._*' \) -delete
  xattr -cr "$APP"

  codesign --force --deep --options runtime --timestamp=none -s "${SIGN_ID:--}" "$APP" 2>&1 | tail -2
  [ -n "${SIGN_ID:-}" ] && codesign --force --options runtime --timestamp -s "$SIGN_ID" "$APP"/Contents/MacOS/* || true

  DMGSRC=$(mktemp -d)
  cp -RX "$APP" "$DMGSRC/"; ln -s /Applications "$DMGSRC/Applications"
  cp "$SRC/macos/Kommandozeile installieren.command" "$DMGSRC/"
  cp "$SRC/LICENSE" "$SRC/THIRD-PARTY-NOTICES.md" "$DMGSRC/"
  if [ -d "$SRC/examples" ]; then cp -RX "$SRC/examples" "$DMGSRC/Beispiele"; fi
  find "$DMGSRC" -name '._*' -delete; xattr -cr "$DMGSRC" 2>/dev/null || true
  RESULT=$OUT/VintageCoder-$VERSION-macos-$ARCH.dmg
  rm -f "$RESULT"; hdiutil create -quiet -volname "VintageCoder $VERSION" -srcfolder "$DMGSRC" -ov -format UDZO "$RESULT"
  rm -rf "$DMGSRC"
  [ -d "$OUT/VintageCoder.app" ] && rm -rf "$OUT/VintageCoder.app"; cp -RX "$APP" "$OUT/VintageCoder.app"
  if [ -n "${SIGN_ID:-}" ] && [ -n "${NOTARY_PROFILE:-}" ]; then
    codesign -s "$SIGN_ID" "$RESULT"
    xcrun notarytool submit "$RESULT" --keychain-profile "$NOTARY_PROFILE" --wait && xcrun stapler staple "$RESULT"
  fi
else
  # ---- ohne Entwicklungsumgebung: Kommandozeilen-Archiv (bin/, data/, Beispiele); die Programme werden ad-hoc signiert
  TOP=VintageCoder-CLI-$VERSION-macos-$ARCH
  PKG=$WORK/$TOP
  mkdir -p "$PKG"
  cp -RX "$SRC/bin" "$PKG/bin"
  for f in data docs editors examples LICENSE THIRD-PARTY-NOTICES.md README.md release-report.md; do if [ -e "$SRC/$f" ]; then cp -RX "$SRC/$f" "$PKG/$f"; fi; done
  cat > "$PKG/Kommandozeile installieren.command" <<'EOS'
#!/bin/bash
# Verlinkt vcc und vc-lsp aus diesem Ordner nach /usr/local/bin (bin/ und data/ müssen nebeneinander bleiben).
set -e
HERE=$(cd "$(dirname "$0")" && pwd)
for t in vcc vc-lsp; do
  if ! { mkdir -p /usr/local/bin && ln -sf "$HERE/bin/$t" "/usr/local/bin/$t"; } 2>/dev/null; then
    echo "Administratorrechte nötig für /usr/local/bin ..."
    sudo mkdir -p /usr/local/bin && sudo ln -sf "$HERE/bin/$t" "/usr/local/bin/$t"
  fi
  echo "verlinkt: /usr/local/bin/$t"
done
echo; echo "Fertig. Neues Terminalfenster öffnen und 'vcc --version' ausprobieren."
read -r -p "Enter zum Beenden"
EOS
  chmod +x "$PKG/Kommandozeile installieren.command"
  find "$PKG" \( -name .DS_Store -o -name '._*' \) -delete
  for b in "$PKG"/bin/*; do codesign --force --options runtime --timestamp=none -s "${SIGN_ID:--}" "$b" 2>&1 | tail -1; done
  RESULT=$OUT/$TOP.tar.gz
  rm -f "$RESULT"; COPYFILE_DISABLE=1 tar -C "$WORK" -czf "$RESULT" "$TOP"
fi
rm -rf "$WORK"
echo "Fertig: $RESULT"
