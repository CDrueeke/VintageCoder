#!/bin/sh
# Stufe 1: baut ein macOS-.app der neuen Oberflaeche (Dev-Stand, nicht der Release-Installer) (ohne Installer, ad hoc signiert). Ausgabe: build/gui/VintageCoder.app
set -e
cd "$(dirname "$0")/../.."
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/vintagecoder-target-wunschspalte}" CARGO_INCREMENTAL=0
cargo build --release -p vc-gui
APP=build/gui/VintageCoder.app
rm -rf "$APP"; mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp installer/macos/VintageCoder.icns "$APP/Contents/Resources/VintageCoder.icns"   # App-Icon unverändert aus main
cp "$CARGO_TARGET_DIR/release/vintagecoder" "$APP/Contents/MacOS/vintagecoder"
cat > "$APP/Contents/Info.plist" <<'P'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>VintageCoder</string>
<key>CFBundleIdentifier</key><string>de.drueeke.vintagecoder.neu</string>
<key>CFBundleIconFile</key><string>VintageCoder</string>
<key>CFBundleExecutable</key><string>vintagecoder</string>
<key>CFBundleVersion</key><string>0.0.0</string><key>CFBundleShortVersionString</key><string>0.2.0-dev</string>
<key>CFBundlePackageType</key><string>APPL</string><key>NSHighResolutionCapable</key><true/>
</dict></plist>
P
codesign --force --sign - "$APP" >/dev/null 2>&1 || true
echo "$PWD/$APP"
