# Mitwirken

Danke für dein Interesse an VintageCoder! Das Projekt befindet sich im Alpha-Stadium und wird von einer Einzelperson betreut. Beiträge sind willkommen, es gibt aber keine Zusage, dass sie angenommen werden, und Antworten können dauern.

## Fehler melden und Vorschläge machen

- **Fehler und Wünsche** als [Issue](https://github.com/CDrueeke/VintageCoder/issues) (es gibt Vorlagen) oder per E-Mail an **vintagecoder@drueeke.de**.
- Die Anwendung bereitet die Angaben vor: *Hilfe ▸ Feedback senden …* (oder `vcc feedback`) zeigt Version, Betriebssystem und Installationsart. Schicke sie mit, ebenso das Zielsystem und, wenn möglich, ein kleines Beispielprojekt oder den Quelltext, der den Fehler zeigt. Bei schwer beschreibbaren Fehlern hilft das Debug-Protokoll (siehe Hilfe „Feedback senden“). Hänge nichts an, was du nicht veröffentlichen darfst (zum Beispiel ROMs).
- **Sicherheitslücken** bitte nicht als Issue, sondern wie in [SECURITY.md](SECURITY.md).

## Pull Requests

- Sprich größere Änderungen vorher in einem Issue an, damit du keine Arbeit umsonst machst.
- Eine Änderung je Pull Request, kleine und sprechende Commits (deutsch oder englisch, in der Befehlsform oder als kurze Beschreibung).
- Beschreibe, was sich ändert und wie du es geprüft hast. Neue Funktionen brauchen einen Test und einen Eintrag im [CHANGELOG](CHANGELOG.md) unter „Unveröffentlicht“ sowie, wenn Anwender sie sehen, eine Hilfeseite oder einen Handbuchabschnitt.
- Die **Versionsnummer** (`Cargo.toml`) und Tags ändert nur der Projektinhaber.
- Das Repository enthält Releasestände; die frühere Entwicklungsgeschichte ist nicht öffentlich. Übernommen wird ein Beitrag deshalb eventuell als Patch in die Entwicklung und erscheint mit dem nächsten Stand.

## Bauen und testen

Voraussetzungen und Hinweise je Betriebssystem stehen in der [README](README.md#aus-dem-quelltext-bauen) (Rust 1.95 oder neuer; unter Linux zusätzlich Entwicklungsbibliotheken, siehe [Linux-Hilfe](crates/vc-ide/help/linux.md)).

```bash
cargo build --workspace --all-targets   # ohne Warnungen
cargo test --workspace                  # Hilfe- und Handbuch-Tests, Galerie- und Oberflächentests; dauert (Iced mit tiny-skia, mehrere Minuten)
python3 tools/release.py check          # Release-Auswahl, Registry und Dateien
python3 tools/check_schreibweise.py     # Schreibweisen (u. a. „Zeropage“)
python3 tools/check_kuerzel.py          # Kürzel in Handbuch und Hilfe gegen die Befehlstabelle
python3 -m unittest tools.test_release tools.test_linux_pkg tools.test_export_check tools.test_source_archive tools.test_gui_notices tools.test_check_schreibweise tools.test_charsets tools.test_check_kuerzel
python3 tools/ci_staging.py --bin-dir target/debug   # je Testauswahl ein Staging
```

Dieselben Schritte laufen in `.github/workflows/ci.yml` (GitHub) und in `ci-local.yml` (lokal, `act`); beide halten wir gleich.

Wo was liegt (Crates, Wegweiser durch die Oberfläche): [docs/architektur.md](docs/architektur.md). Prüfung der Oberfläche ohne Fenster (Galerie, Bildvergleich, Layout-Prüfung): [docs/pruefen.md](docs/pruefen.md). Hilfeseiten stehen unter `crates/vc-ide/help/` und werden von Tests in `crates/vc-ide/src/help.rs` geprüft (Verweise, Kennungen, Schreibweisen).

Tests, die einen Emulator, echte Hardware oder Original-ROMs brauchen, überspringen sich selbst, wenn diese fehlen.

## Code und Sprache

- **Bezeichner** (Funktionen, Variablen, Dateinamen im Code) sind **englisch**, **Anzeigetexte**, Hilfe, Handbuch und Kommentare zum Verständnis **deutsch**.
- Rust-Code im Stil der Umgebung halten (`cargo fmt` ohne Besonderheiten, keine neuen Compiler-Warnungen). Jede Quelldatei beginnt mit `// SPDX-License-Identifier: GPL-3.0-or-later` (Python-Skripte mit `#`); **Ausnahme:** Dateien unter `data/sys/`, `data/templates/` und `examples/` tragen `SPDX-License-Identifier: MIT-0` (Ordnervermerk in der Datei `LICENSE` des Ordners).
- Keine Abhängigkeiten ohne Not; neue Bibliotheken müssen eine mit der GPL-3.0 vereinbare Lizenz haben und stehen dann in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).
- **Textdateien** (Quelltexte, Markdown, JSON, Projektdateien) sind **UTF-8 ohne BOM** mit **LF** als Zeilenende; `.gitattributes` erzwingt LF. Die IDE schreibt beim Speichern immer so und liest fremde Dateien (BOM, CRLF, Windows-1252) mit einem Hinweis.
- **Oberfläche (neue Oberfläche `vc-gui`):** Jedes neue oder überarbeitete Studio und jede neue Seite folgt dem gemeinsamen Muster. Pflicht ist der **Studio-Rahmen** aus `crates/vc-gui-ui` (Kopfzeile mit allen Datei-Aktionen, genau eine Werkzeugleiste, Liste, eingepasste Arbeitsfläche ohne Scrollbalken, Seitenspalte aus Karten „Werkzeug“ und „Dokument“, Fußzeile), die drei Breitenklassen (breit ab 900, mittel 640 bis 899, schmal unter 640 px), das 4-px-Raster und nur Farbrollen aus `Pal`. Ein Studio ohne Studio-Rahmen wird nicht angenommen.
- **Fachbegriff „Zeropage“:** wird in allen Texten (Code, Oberfläche, Hilfe, Handbuch, Tests, Beispiele) nie übersetzt; `tools/check_schreibweise.py` prüft das in der CI.
- **Keine ROMs, Firmware oder Emulator-Programme** und keine urheberrechtlich geschützten Fremddateien ins Repository legen.
- In Texten, die ausgeliefert werden, keine Produktnamen fremder Entwicklungsumgebungen (der Verbotsscan in `tools/release.py` prüft das).

## Lizenz der Beiträge

VintageCoder steht unter der **GNU General Public License, Version 3 oder später** (`GPL-3.0-or-later`, siehe [LICENSE](LICENSE)). Mit einem Beitrag erklärst du dich einverstanden, dass er unter dieser Lizenz veröffentlicht wird, und bestätigst, dass du ihn selbst geschrieben hast oder die Rechte dafür besitzt. **Beiträge zu `data/sys/`, `data/templates/` und `examples/`** kommen unter der **MIT-0-Lizenz** (siehe `LICENSES/MIT-0.txt`), damit sie frei in eigene Programme eingesetzt werden können.
