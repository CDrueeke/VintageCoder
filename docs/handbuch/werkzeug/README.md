# Werkzeug für die Handbuch-Bilder

**Rohbilder liegen nicht im Repository.** `bilder/roh/` (unbeschriftete Aufnahmen, rund 58 MB) steht in `.gitignore`; im Repository liegen nur die fertigen, beschrifteten Bilder (`bilder/<id>.png`, verlustfrei optimiert), das Manifest und diese Werkzeuge. Eine Sicherung der Rohbilder liegt lokal unter `~/Development/handbuch-roh/`.

## Rohbilder neu erzeugen

Die Aufnahmeskripte reproduzieren sie. Rohbilder werden standardmäßig in `docs/handbuch/bilder/roh/` geschrieben, abweichend in den Ordner aus der Umgebungsvariable `HANDBUCH_ROH` (z. B. `HANDBUCH_ROH=~/Development/handbuch-roh`).

1. IDE und `vcc` in einen eigenen Ordner bauen (`CARGO_TARGET_DIR=~/.cache/vintagecoder-target-bilder cargo build -p vc-app --bin vintagecoder && cargo build --bin vcc`); `shot.sh` und die Skripte erwarten sie dort (`VC_BIN`, `VC_VCC` überschreiben das).
2. Nacheinander ausführen (nur eine IDE-Instanz gleichzeitig, jedes Bild braucht etwa 20 bis 30 s): `aufnahmen-1.sh` (ruft 1b und `ausschnitte.sh` mit auf), `aufnahmen-2.sh` und `ausschnitte2.sh`, `aufnahmen-3.sh` und `ausschnitte3.sh`, `aufnahmen-4.sh`, `aufnahmen-5.sh`, `aufnahmen-6.sh`, `aufnahmen-7.sh` (Startbildschirm und Konfigurationsassistent; legt für den Wiederaufruf eine Demo-`settings.json` an und löscht sie wieder).
3. Beschriften: `swift werkzeug/annotate.swift docs/handbuch [id …]` liest `bilder/manifest.json` (Marken in Pixeln der Rohaufnahme) und schreibt `bilder/<id>.png` (1600 px breit).
4. Verkleinern: `cc -O2 -o /tmp/pngopt werkzeug/pngopt.c -lz && /tmp/pngopt docs/handbuch/bilder/*.png` (verlustfrei, prüft jede Ausgabe pixelgenau gegen das Original).

## Dateien

- `shot.sh`: eine IDE-Aufnahme über den Selbsttest-Modus der App (`VC_SHOT`, `VC_DEMO`, …); Kopf des Skripts erklärt die Werte. Läuft mit eigenem `HOME` unter `/tmp/demo/home` und neutralen Pfaden unter `/tmp/demo/Retro`, damit keine Nutzerpfade im Bild stehen.
- `aufnahmen-*.sh`, `ausschnitte*.sh`: die Aufnahmen und Zuschnitte (nach Kapitelgruppen).
- `annotate.swift`: Marken (Rahmen, Pfeil, nummerierte Kreise in Orange `#FF9F1C`) aus dem Manifest.
- `kuerzel-karte.swift`: rendert die Tastenkürzel-Karte (kein Screenshot).
- `split.swift`: Dunkel/Hell-Montage; `pngopt.c`: verlustfreier PNG-Optimierer.
- `beispiele/farbwechsel/`: Übungsprogramm für Debugger und Tests.
