# Messung G1 auf Windows und Linux (Anleitung für die Messsitzungen)

Zweck (Konzept Abschnitt 9, E0, Prüfpunkt G1): Läuft die neue Oberfläche auf dem System, öffnet sich das Fenster, zeichnet das Canvas, wie schnell ist der Rundlauf der Kernbefehle? Ein Ergebnis je System genügt: die Messdatei, ein Bild und einige Angaben. **Nichts ändern, nichts committen, nichts veröffentlichen**; nur bauen und messen.

## Voraussetzungen

| | Windows 10/11 x86_64 | Linux x86_64 (Debian 13 oder Ubuntu 24.04, virtuelle Maschine) |
|---|---|---|
| Rust | stable (`rustup`), MSVC-Toolchain | stable (`rustup`) |
| Node | 22 oder neuer (`node -v`) | 22 oder neuer |
| System | WebView2-Laufzeit (Windows 11 hat sie; Windows 10: Evergreen-Bootstrapper von Microsoft) | `sudo apt install build-essential pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev libxdo-dev libasound2-dev` |

## Bauen

Branch `claude/ui-e0` (privates Repository `CDrueeke/VintageCoder-dev`) auschecken, dann:

```bash
cd ui
npm ci --ignore-scripts
cd ../crates/vc-shell
# Windows (PowerShell):  $env:VITE_SELFTEST = "1"
# Linux:                 export VITE_SELFTEST=1
../../ui/node_modules/.bin/tauri build --features selftest --no-bundle     # Windows: ..\..\ui\node_modules\.bin\tauri.cmd
```

Die ausführbare Datei liegt danach in `target/release/vintagecoder` (Windows: `vintagecoder.exe`).

## Messen

1. **Selbstmessung** (das Fenster öffnet sich, misst sich und schließt sich nach 10 bis 40 s):
   ```bash
   # Windows: $env:VC_NO_SAVE = "1"; $env:VC_SELFTEST_OUT = "$env:TEMP\selftest.json"
   # Linux:   export VC_NO_SAVE=1 VC_SELFTEST_OUT=/tmp/selftest.json
   target/release/vintagecoder
   ```
   Dreimal wiederholen (erster Lauf ist „kalt“). Die drei JSON-Dateien abgeben. Öffnet sich kein Fenster oder bleibt es leer: Fehlermeldung der Konsole abgeben; Linux: zusätzlich `WEBKIT_DISABLE_COMPOSITING_MODE=1` versuchen und den Unterschied nennen.
2. **Bild und Speicher** (ohne `VC_SELFTEST_OUT`, Variable löschen): App normal starten, `Strg+K` ▸ „Messung“ ▸ „Messung starten“, Ergebnisbild der Tabelle aufnehmen (Fenster mit Titelleiste); danach im Leerlauf nach 15 s den Speicher **aller Prozesse** der App ablesen (Windows: Task-Manager ▸ Details, `vintagecoder.exe` und alle `msedgewebview2.exe` darunter; Linux: `ps -eo pid,rss,comm | grep -E "vintagecoder|WebKit"`).
3. Angaben: Betriebssystem und Version, Prozessor, Arbeitsspeicher, WebView2-Version bzw. `libwebkit2gtk-4.1`-Version (`dpkg -l libwebkit2gtk-4.1-0`), grafische Umgebung (Wayland/X11/Treiber der virtuellen Maschine), ob die Titelleiste/Fensterrahmen normal aussehen, ob Hell/Dunkel nach System wechselt, ob Umlaute und ⌘-Zeichen (hier: Strg) richtig dargestellt werden, Auffälligkeiten (Flackern, Ruckeln, falsche Schrift).

Abgabe an die Hauptsession: die drei JSON-Dateien, das Bild, die Angaben. Aufräumen: nur die Build-Ausgabe (`target/`), sonst wurde nichts verändert.
