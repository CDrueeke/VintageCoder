# Änderungsprotokoll

Die Versionsnummer hat die Form `MAJOR.MINOR.PATCH`. Solange sie unter 1.0 liegt, steht `0.MINOR` für ein neues Funktionspaket oder
einen Bruch bei Projektdateien oder Einstellungen und `PATCH` für Fehlerkorrekturen. Die Nummer und die Tags (`vMAJOR.MINOR.PATCH`) legt
der Projektinhaber fest. Änderungen für die nächste Version stehen unter „Unveröffentlicht“.

## Unveröffentlicht

## 0.1.0 (Alpha)

Erste öffentliche Version. VintageCoder ist in Entwicklung und enthält mit Sicherheit noch Fehler; Stände und Einschränkungen stehen in
[STATUS.md](STATUS.md).

### Zielsysteme

- Commodore 64, Atari 8 Bit (400/800/XL/XE, PAL und NTSC), Commodore Plus/4 (auch C16) und MEGA65 sind in den Paketen enthalten.
- Commodore 128, VC-20, C16/C116 als eigenes Ziel sowie PET/CBM und CBM-II (die beiden letzten als Vorschau, nur BASIC) sind im Quelltext
  vorhanden und kommen mit einem späteren Paketstand.

### Entwicklungsumgebung

- Quelltext-Editor mit Syntaxfarben, Code-Faltung, Vervollständigung, Hilfe zum Wort am Cursor und Fehlern beim Tippen.
- Projektverwaltung mit Projektbaum, Übersicht, Einstellungen und geführter Ersteinrichtung mit Wiederaufruf; Arbeitsmodi „Code“ und „Debug“.
- Handbuch und Hilfe in der Anwendung (F1), Befehlspalette, Feedback per E-Mail aus der Anwendung (Hilfe ▸ Feedback senden …).
- Studios: Sprites (auch Atari Player/Missile), Zeichensätze, Bildschirme, Kachelkarten, Paletten, Tracker für SID und POKEY,
  PLAY-Tracker für den MEGA65, Disk-Studio (D64, D71, D81, ATR) und Memory Map.
- Debugger mit Haltepunkten (auch bedingt), Datenhaltepunkten (VICE), Registern, Speicher und Disassembler.
- Emulator-Anbindungen: VICE, atari800, Altirra, Fujisan und Xemu; Ultimate 64 / II+ über das Netzwerk.

### Assembler und BASIC

- Eigener 6502-Assembler mit Makros, Scopes, bedingter Übersetzung, Zyklenzählung und Zyklenbudgets, Segmenten und Bänken; Ausgabe als
  PRG, BIN, XEX und Steckmodule.
- BASIC im Originaldialekt des Zielsystems, auf Wunsch ohne Zeilennummern mit Sprungmarken und langen Variablennamen.
- Übersetzung von BASIC nach Assembler (in Arbeit).

### Kommandozeile und Werkzeuge

- Kommandozeile `vcc` für Bauen, Tests, Disk-Images und Medien, mit Hilfe im Terminal, man-Seiten und Vervollständigung.
- Automatische Tests für Assembler-Projekte: statische Prüfungen und Unterprogramm-Tests im Emulator.
- Sprachserver `vc-lsp` für andere Editoren ([Anleitung](docs/sprachserver.md)).
- Auswahl, was ein Paket enthält, über `FEATURES.md` und `tools/release.py` ([Anleitung](docs/release-auswahl.md)).

### Pakete

- macOS (Apple-Chip, `.dmg`), Windows (x86_64, Installer), Linux (x86_64: `.deb` für Debian 13 und Ubuntu 24.04, `.rpm` für Fedora 45
  und openSUSE Tumbleweed, `.tar.gz`). Intel-Macs sowie ARM unter Windows und Linux werden nicht unterstützt.
- Die Pakete sind nicht signiert; Hinweise stehen im README und in den Release-Notizen.

### Bekannte Einschränkungen

- Die Pakete wurden in frischen Systemen geprüft, aber nicht in allen Umgebungen: echter Desktop-Betrieb unter Linux, Installation auf
  fremden Rechnern, andere Windows- und Linux-Versionen.
- Der Extraktor (Zeichensätze und Bildschirme aus Programmen holen) und die Anbindung an ein echtes MEGA65 sind noch nicht enthalten.
