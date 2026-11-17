#!/bin/sh
# SPIKE: baut ein macOS-.app des Iced-Spikes (ohne Installer, ad hoc signiert). Ausgabe: build/spike/VintageCoder-Spike.app
set -e
cd "$(dirname "$0")/../.."
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/vintagecoder-target-wunschspalte}" CARGO_INCREMENTAL=0
cargo build --release -p spike-iced
APP=build/spike/VintageCoder-Spike.app
rm -rf "$APP"; mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp installer/macos/VintageCoder.icns "$APP/Contents/Resources/VintageCoder.icns"   # App-Icon unverändert aus main
cp "$CARGO_TARGET_DIR/release/vc-spike-iced" "$APP/Contents/MacOS/vc-spike-iced"
cat > "$APP/Contents/Info.plist" <<'P'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>VintageCoder Spike</string>
<key>CFBundleIdentifier</key><string>de.drueeke.vintagecoder.spike</string>
<key>CFBundleIconFile</key><string>VintageCoder</string>
<key>CFBundleExecutable</key><string>vc-spike-iced</string>
<key>CFBundleVersion</key><string>0.0.0</string><key>CFBundleShortVersionString</key><string>0.0.0-spike</string>
<key>CFBundlePackageType</key><string>APPL</string><key>NSHighResolutionCapable</key><true/>
</dict></plist>
P
codesign --force --sign - "$APP" >/dev/null 2>&1 || true
echo "$PWD/$APP"
