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
cargo build --workspace          # ohne Warnungen
cargo test --workspace           # enthält die Tests für Handbuch und Hilfeseiten
python3 tools/release.py check   # Release-Auswahl, Registry und Dateien
python3 -m unittest tools.test_release tools.test_linux_pkg   # Python-Tests der Werkzeuge
```

Tests, die einen Emulator, echte Hardware oder Original-ROMs brauchen, überspringen sich selbst, wenn diese fehlen.

## Code und Sprache

- **Bezeichner** (Funktionen, Variablen, Dateinamen im Code) sind **englisch**, **Anzeigetexte**, Hilfe, Handbuch und Kommentare zum Verständnis **deutsch**.
- Rust-Code im Stil der Umgebung halten (`cargo fmt` ohne Besonderheiten, keine neuen Compiler-Warnungen). Jede Quelldatei beginnt mit `// SPDX-License-Identifier: GPL-3.0-or-later` (Python-Skripte mit `#`).
- Keine Abhängigkeiten ohne Not; neue Bibliotheken müssen eine mit der GPL-3.0 vereinbare Lizenz haben und stehen dann in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).
- **Keine ROMs, Firmware oder Emulator-Programme** und keine urheberrechtlich geschützten Fremddateien ins Repository legen.
- In Texten, die ausgeliefert werden, keine Produktnamen fremder Entwicklungsumgebungen (der Verbotsscan in `tools/release.py` prüft das).

## Lizenz der Beiträge

VintageCoder steht unter der **GNU General Public License, Version 3 oder später** (`GPL-3.0-or-later`, siehe [LICENSE](LICENSE)). Mit einem Beitrag erklärst du dich einverstanden, dass er unter dieser Lizenz veröffentlicht wird, und bestätigst, dass du ihn selbst geschrieben hast oder die Rechte dafür besitzt.
