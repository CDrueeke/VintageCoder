# Release-Auswahl (für Entwickler)

Diese Seite richtet sich an alle, die aus dem Quellbaum ein Release zusammenstellen. Wer VintageCoder nur benutzt, braucht sie nicht; sie erscheint deshalb nur im Quellbau (ohne `data/release.json`), nicht in einem Release. Ein Release enthält nicht alles, was der Quellbaum kann: Der Projektinhaber wählt in einer einzigen Datei aus, welche Teile ausgeliefert werden. Ein Skript prüft die Auswahl, stellt die Dateien zusammen, und das Programm blendet zur Laufzeit aus, was nicht gewählt wurde.

## Die Auswahl: `FEATURES.md`

`FEATURES.md` im Hauptordner ist der **einzige Ort mit Häkchen**. Sie hat fünf feste Abschnitte, jeder mit einer Tabelle:

| Abschnitt | Inhalt | Kennungen |
|---|---|---|
| 1. Programm | die Entwicklungsumgebung `vintagecoder` | `IDE-APP` |
| 2. Studios | Editoren für Grafik, Klang und Medien (nur wirksam mit `IDE-APP`) | `STU-SPRITE`, `STU-CHAR`, `STU-SCREEN`, `STU-TILES`, `STU-PALETTE`, `STU-TRACKER`, `STU-PLAY`, `STU-DISK`, `STU-MEMMAP`, `STU-EXTRACT` |
| 3. Anbindungen | Emulatoren und echte Geräte | `EMU-VICE`, `EMU-ATARI800`, `EMU-ALTIRRA`, `EMU-FUJISAN`, `EMU-XEMU`, `HW-ULTIMATE`, `HW-MEGA65` |
| 4. Zielsysteme | C64, Atari 8 Bit, MEGA65, Plus/4, C16, C128, VC-20, PET/CBM, CBM-II | `SYS-C64`, `SYS-ATARI`, `SYS-MEGA65`, `SYS-PLUS4`, `SYS-C16`, `SYS-C128`, `SYS-VC20`, `SYS-PET`, `SYS-CBM2` |
| 5. Beispielprojekte | Pakete von Beispielen | `EX-HELLO`, `EX-JUMPRUN`, `EX-PRALLBALL`, `EX-DEMOS` |

Jede Zeile hat vier Spalten: **Kennung** (in Backticks), Beschreibung, **Stand** (Stabil, Beta, In Arbeit, Spike, Konzept) und **Release**. Angehakt ist eine Zeile, wenn das letzte Feld `[x]` enthält; `[X]`, `[ x]`, `[x ]` und `[ X]` zählen ebenso, `[ ]` und ein leeres Feld nicht. Text außerhalb der Tabellen und HTML-Kommentare liest das Skript nicht. Die Kennung muss genau einmal in `FEATURES.md` und genau einmal in der Registry stehen.

Immer dabei, ohne Häkchen: die Kommandozeile `vcc` mit Assembler, BASIC, Bau und Test sowie der Sprachserver `vc-lsp`. Die Stände dieser festen Bestandteile, die Release-Kriterien und die bekannten Lücken stehen in `STATUS.md`. Das Betriebssystem ist keine Auswahl: Eine Auswahl gilt für Windows, macOS und Linux, nur die Verpackung unterscheidet sich.

## Registry und `release.json`

Drei Dateien hängen zusammen:

1. **`FEATURES.md`** (Repository): was gewählt ist.
2. **`release/registry.json`** (Repository, wird nicht ausgeliefert): die Technik zu jeder Kennung. Sie nennt die Art (`studio`, `link`, `target`, `examples`), die Kurzkennung für das Programm (`sprite`, `vice`, `atari8` …), die Abhängigkeiten (`needs`: alle nötig, `needs_any`: mindestens eine), weiche Hinweise (`soft`), die Dateien, die zur Zeile gehören (`files`, mit `**` und `{a,b}`), die Hilfethemen (`help`) sowie die Listen `core` (immer dabei) und `never` (nie dabei, z. B. `data/sys/vcs2600/**`).
3. **`data/release.json`** (nur im Paket, vom Skript erzeugt): die gewählte Positivliste, die das Programm beim Start liest.

```json
{ "format": 1, "ide": true,
  "studios": ["sprite", "char", "screen", "tiles", "palette", "tracker", "play", "disk", "memmap"],
  "links": ["vice", "atari800", "altirra", "fujisan", "xemu", "ultimate"],
  "targets": ["c64", "atari8", "mega65", "plus4"],
  "examples": ["EX-HELLO", "EX-JUMPRUN", "EX-PRALLBALL", "EX-DEMOS"] }
```

**Regeln beim Lesen** (Modul `vc_ide::features`, von `vintagecoder` und `vcc` gleich verwendet):

- Die Datei fehlt: **alles ist an**. So verhalten sich der Quellbaum, `cargo run` und `cargo test`.
- Ein Schlüssel fehlt: dafür ist alles an. Ein Schlüssel ist da: nur die genannten Einträge. Neue Studios in einer alten `release.json` bleiben also aus.
- Ist die Datei kaputt oder ihr `format` neuer, als das Programm kennt, gilt die **Mindestauswahl** (nichts Wählbares an) mit einer Warnung, nie „alles an“.
- `VC_RELEASE=all` erzwingt „alles an“, `VC_RELEASE=<Pfad>` lädt eine andere `release.json`. Das ist der Weg, ein Release im Quellbau auszuprobieren, ohne zu paketieren.

```bash
VC_RELEASE=build/stage-linux/data/release.json cargo run -p vc-app
VC_RELEASE=all vcc help                         # alles an, auch im Paket
```

## Das Skript `tools/release.py`

Nur Python-Standardbibliothek, Aufruf vom Hauptordner (der Arbeitsordner ist sonst egal).

| Unterbefehl | Zweck |
|---|---|
| `check` | `FEATURES.md` gegen Registry und Dateisystem prüfen: Kennungen, Stände, Abhängigkeiten, Dateizuordnung, Beispiele, `#include`-Abschluss. Rückgabewert 1 bei Fehlern. |
| `plan` | zeigen, was ausgeliefert würde (Zeilen, Dateien, Größen). Schreibt nichts. |
| `stage` | ein Staging-Verzeichnis für ein Betriebssystem erzeugen, das die Installer nur noch einpacken. |
| `package` | Linux-Pakete (`.deb`, `.rpm`, `tar.gz`) aus einem Linux-Staging bauen (`dpkg-deb`, `rpmbuild`; im Bau-Container, siehe `Makefile`). |
| `bootstrap` | einmaliger Entwurf von `registry.json` aus dem Bestand (`data/targets`, `data/args`, `examples`); danach von Hand pflegen. |

```bash
python3 tools/release.py check                           # Auswahl und Registry prüfen
python3 tools/release.py plan -v                         # was würde ausgeliefert, mit Hinweisen
python3 tools/release.py check --explain STU-TILES       # wer braucht die Zeile, was hängt daran
python3 tools/release.py stage --os macos --out build/stage-macos --build
python3 tools/release.py stage --os windows --out build/stage-windows --bin-dir target/release --force
python3 tools/release.py stage --os linux --out build/stage-linux --build --archive --reproducible
python3 tools/release.py package --os linux --format deb --distro debian --stage build/stage-linux --out build/pakete
python3 tools/release.py check --features tools/testdata/nur-cli-c64.md
python3 tools/release.py bootstrap --out /tmp/registry-entwurf.json
```

Wichtige Optionen: `--features DATEI` (andere Auswahldatei, etwa eine Testauswahl), `--allow-unfinished` (gewählte Zeilen mit Stand unter Beta zulassen, der Bericht markiert sie), `--auto-deps` (fehlende harte Abhängigkeiten ergänzen und melden, nur für Experimente), `--explain KENNUNG`, `-v`. Bei `stage`: `--os windows|macos|linux`, `--out ORDNER`, `--build` (vorher `cargo build --release`, ein Bau je Betriebssystem), `--bin-dir` (fertige Programme), `--force`, `--dry-run`, `--reproducible` (keine Zeitstempel, gleiche Prüfsummen), `--archive` (Linux: zusätzlich `tar.gz`), `--verify-build` (jedes ausgelieferte Beispiel mit dem Staging-`vcc` bauen). Das Skript liest die Versionsnummer nur aus `Cargo.toml`; vergeben wird sie ausschließlich vom Projektinhaber.

Zeilen unter Beta, die ausdrücklich mitgeliefert werden sollen, stehen in `release/registry.json` unter `preview_ok` (derzeit PET/CBM und CBM-II). `check` und `stage` liefern sie dann ohne `--allow-unfinished` aus und markieren sie im Bericht als Vorschau.

**Was `stage` tut:** Es prüft die Auswahl (bei Fehlern Abbruch), kopiert die Programme (`vcc` und `vc-lsp` immer, `vintagecoder` nur mit `IDE-APP`), `data/` nach Registry, die Beispiele, schreibt `data/release.json` und den Betriebssystemteil (Windows `installer.inc`, macOS Ordnerstruktur, Linux ein FHS-Baum mit Programmen, Daten, Manpages), führt den **Verbotsscan** aus (keine ROM-Abbilder, keine Emulator-Programme, keine Namen anderer Entwicklungsumgebungen in `data/` und `examples/`) und schreibt `release-report.md` mit Auswahl, Warnungen und den **Hinweisen für die Release-Notizen**.

## Abhängigkeiten

Ein Studio braucht `IDE-APP`. `EMU-ATARI800`, `EMU-ALTIRRA` und `EMU-FUJISAN` brauchen `SYS-ATARI`, `EMU-XEMU` und `HW-MEGA65` brauchen `SYS-MEGA65`, `HW-ULTIMATE` braucht `SYS-C64`, `EMU-VICE` mindestens eines der VICE-Systeme. `STU-TRACKER` und `STU-PLAY` brauchen ein Ziel mit Tonchip. Ein Beispiel wird nur ausgeliefert, wenn sein **Paket und das Zielsystem seines Ordners** gewählt sind. Genaue Regeln: `check --explain <Kennung>`.

Bei einer verletzten **harten** Abhängigkeit bricht `check` (und damit `stage`) mit einer Meldung wie „`HW-MEGA65` braucht `SYS-MEGA65`“ ab und nennt, was anzukreuzen oder abzuwählen ist. Das Skript ändert die Auswahl nie stillschweigend (Ausnahme: `--auto-deps`). **Weiche** Abhängigkeiten ergeben nur eine Warnung im Bericht, etwa „Zielsystem ohne Emulator oder Gerät: nur Bau, kein Start“. Eine gewählte Zeile mit Stand unter Beta bricht ebenfalls ab (Ausnahme: `--allow-unfinished`).

## Was abgewählte Teile bewirken

**Versteckt, nicht entfernt.** Der Code bleibt im Programm; die Oberfläche zeigt abgewählte Teile nirgends und lässt sich nicht dorthin führen.

- **Studios:** keine Menüpunkte, keine Befehle in der Befehlspalette, keine Schaltflächen (Willkommensseite, Übersicht, „+“ im Projektbaum), kein Reiter. Die Datei eines abgewählten Studios (etwa `*.sprite.json`) öffnet im Text-Editor mit einem Hinweis. **Dateiformate, `vcc media` und `vcc import` bleiben immer**; Projekte bleiben baubar. Gespeicherte Fensterlagen mit abgewählten Studios bleiben in `layout.json`, wirken aber nicht.
- **Anbindungen:** Erkennung, F5, Debugger, `vcc test --backend`, `vcc emulator` und `vcc streams` melden „nicht im Paket enthalten“ (Rückgabewert 1). Einstellungen zeigen nur Karten der gewählten Anbindungen; bereits gespeicherte Einträge bleiben in der Datei. Debugger-Befehle, Reiter „Live-Bild“ und die Hilfeseite „Emulatoren“ richten sich nach den gewählten Anbindungen. Emulatoren und ROMs werden nie ausgeliefert.
- **Zielsysteme:** Wizard, Willkommensseite, Einstellungen, Vorlagen und `vcc new` führen nur gelieferte Systeme. Ein Projekt mit fehlendem Ziel lässt sich öffnen und bearbeiten; Bau und Start melden „Zielsystem ‚x‘ ist in diesem Release nicht enthalten“, `vcc` endet mit Rückgabewert 2. Die Projektdatei wird nie verändert. Das Vorgabe-Target ist der C64, wenn geliefert, sonst das erste gelieferte.
- **Hilfe:** Themen, die nur zu abgewählten Teilen gehören, fehlen in Liste, Suche und Verweisen (ein Link wird zu Text). Die Zuordnung steht in `vc_ide::help::needs` und im Feld `help` der Registry; ein Test hält beide gleich.
- **Beispiele:** ohne Beispielpakete ist „Beispiel öffnen“ auf der Willkommensseite grau. Verweise in der Hilfe auf `examples/…` bleiben Text und können auf nicht gelieferte Beispiele zeigen.

## Was die Auswahl nicht kann

- Sie **entfernt keinen Code**. Wer `data/release.json` ändert oder löscht oder `VC_RELEASE=all` setzt, bekommt die ausgeblendeten Teile zurück. Wer Unfertiges sicher nicht ausliefern will, bräuchte Cargo-Features; das ist nicht vorgesehen. Der Zweck ist ein überschaubares Angebot ohne Hinweise auf Unfertiges, nicht Platzgewinn.
- **Geteilte Datendateien** (`data/templates/*.json`, `data/profile/profiles.json`, `data/roms/known.json`, `data/palettes/index.json`) werden unverändert ausgeliefert und nennen auch nicht gewählte Systeme; das Programm ignoriert diese Einträge.
- Die **festen Bestandteile** (Assembler, BASIC, `basic2asm`, Module, `vcc`) sind nicht abwählbar. Unfertiges darin (`BAS-B2A`, `ASM-CART`, `ASM-45GS02`) erscheint als Hinweis im Release-Bericht.
- **Keine benannten Voreinstellungen** und keine eigene Auswahl je Betriebssystem. Die Testauswahlen unter `tools/testdata/` sind nur für die Prüfung gedacht.
- Es ersetzt **keinen Installer-Lauf**: Windows-Installer und macOS-Paket müssen je Release einmal von Hand gestartet werden.
- **Linux-Pakete** (`.deb`, `.rpm`): `package` baut sie aus dem Linux-Staging (zwei Pakete `vintagecoder-cli` und `vintagecoder`); gebaut und geprüft werden sie in Docker-Containern je Distribution, der Stand steht in `STATUS.md`. `stage --os linux --archive` liefert weiterhin ein Archiv `tar.gz`.

## Weiterführend

Die Zusammenstellung Schritt für Schritt samt Checkliste beschreibt `docs/release-auswahl.md`. Prüfung der Pakete: `python3 tools/ci_staging.py` (ein Staging je Testauswahl, gebaute Beispiele, Verbotsscan, Prüfung mit dem Staging-`vcc`). Die Kommandozeile siehe [Kommandozeile (vcc)](help:cli), Emulatoren [hier](help:emulators), Zielsysteme im [Wizard](help:wizard).
