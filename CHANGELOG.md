# Änderungsprotokoll

Die Versionsnummer hat die Form `MAJOR.MINOR.PATCH`. Solange sie unter 1.0 liegt, steht `0.MINOR` für ein neues Funktionspaket oder
einen Bruch bei Projektdateien oder Einstellungen und `PATCH` für Fehlerkorrekturen. Die Nummer und die Tags (`vMAJOR.MINOR.PATCH`) legt
der Projektinhaber fest. Änderungen für die nächste Version stehen unter „Unveröffentlicht“.

## Unveröffentlicht

## 0.2.0 (Alpha)

Die größte Änderung ist die neue Oberfläche: Sie ist neu aufgebaut, zeigt sich in sechs festen Bereichen und lässt sich vollständig mit Tastatur und Befehlspalette bedienen. Assembler, BASIC und Kommandozeile bleiben im Kern, wie sie sind. VintageCoder ist weiter in Entwicklung und enthält mit Sicherheit noch Fehler; Stände und Einschränkungen stehen in STATUS.md.

### Zielsysteme

- Commodore 64, Atari 8 Bit (400/800/XL/XE), MEGA65, Commodore Plus/4 und C16/C116, Commodore 128 und VC-20 sind in den Paketen enthalten.
- PET/CBM und CBM-II sind nicht in den Paketen.

### Neue Oberfläche

- Sechs feste Bereiche um das Dokumentenfeld: Kopfzeile, Dateien und Suche links, Schublade unten (Probleme, Protokoll, Tests), Panel rechts (Gliederung, Hilfe, Zeichen, Speicherkarte, Emulator) und Statusleiste. Bereiche lassen sich ziehen, einklappen und groß machen; „Layout zurücksetzen“ stellt alles wieder her.
- Zen-Modus (Cmd/Strg+Umschalt+B) blendet alles außer der Arbeitsfläche aus.
- Befehlspalette (Cmd/Strg+K) findet Befehle, Dateien und Aktionen des offenen Studios; ausgegraute Befehle nennen den Grund. Alle Befehle stehen auch in den Menüs, mit Tastenkürzel.
- Kontextmenü auf der rechten Maustaste im Dateibaum, im Editor, auf Reitern und in den Studios.
- Farbschemata Dunkel, Hell und Hoher Kontrast; Größe der Oberfläche (Cmd/Strg+Plus, Minus, 0); Befehle „Editor-Schrift größer/kleiner“ und „Darstellung zurücksetzen“.
- Dateibaum mit Tastatur (Pfeile, Enter, F2, Entf), nach Dateiarten gruppiert; für Bildschirmleser sind Zeilen mit Namen, Art und Ebene gebaut.
- Startseite mit zuletzt geöffneten Projekten und Beispielen, Startbild, geführte Ersteinrichtung.
- Takt des Zielsystems in der Statusleiste (auf Wunsch).
- Konfiguration sichern: Einstellungen exportieren, importieren, zurücksetzen.
- Hilfe und Handbuch erscheinen als gesetzter Text (Überschriften, Tabellen, Listen, Links) im Panel oder in einem Reiter.

### Editor

- Prettyprint: Beim Verlassen einer bearbeiteten Zeile wird sie ausgerichtet (Assembler und BASIC); dazu „Zeile formatieren“ und „Datei formatieren“. Einstellung zum Abschalten.
- Dateien einbinden per Ziehen: Datei aus dem Projektbaum in den Quelltext ziehen setzt die passende `.include`-, `.incbin`- oder `#include`-Zeile; auch per Rechtsklick „Als Include einfügen“.
- Markdown: Dateien mit `.md` zeigen eine gerenderte Vorschau, dazu 16 Schreibhilfen und eine Gliederung.
- Zahlenwerkzeuge: Zahlen und Ausdrücke in allen Schreibweisen; Hex-Vorschläge für BASIC-Adressen.
- BASIC in Assembler umwandeln (experimentell).
- Aktive Datei bauen, starten, debuggen mit eigenen Kürzeln; Art des Reiters (Text, Assembler, BASIC) wählbar.
- Eingebundene Assembler-Dateien werden im Zusammenhang der einbindenden Datei geprüft.

### Studios

- Alle Studios haben denselben Aufbau; Aktionen stehen in der Befehlspalette und im Kontextmenü.
- Disk-Studio: Verzeichnis als Tabelle über die volle Breite, darunter eine Belegungskarte mit Kette der gewählten Datei.
- Bank-Studio: Miniaturen der Screens. Kachel-Export mit wählbarem Layout (Tabellen planar oder verschränkt, Karte zeilen- oder spaltenweise, 8- oder 16-Bit-Index, Lauflängenkodierung); mehrere Kachelsätze je Bank lassen sich exportieren.
- Palette (MEGA65) als Studio; TED-Palette für den Plus/4.
- Pixel-Studio für Sprites, Player/Missile und Zeichensätze mit Auswahl, PNG-Import und Vorlagen.

### Geändert

- Die gesamte Oberfläche wurde neu überarbeitet; Menüs und Kürzel sind in einer Befehlstabelle gesammelt.
- Die Befehlspalette findet Synonyme („petscii“, „kompilieren“ …).
- Handbuch und Hilfe beschreiben die neue Oberfläche, mit Zweck, Grundbegriffen und „Mein erstes …“ je Studio.

### Fehlerkorrekturen

- Tippen in Suchen, Ersetzen, Gehe zu, Palette und Dialogen änderte auch den Code.
- Kürzel mit Enter und Pfeilen kamen im Editor nicht an; dasselbe Kürzel als Menü- und Tastenereignis (macOS) wirkte doppelt.
- „Tests ausführen“ beendete eine laufende Debug-Sitzung.
- Verbindungsabbrüche zum Emulator werden verständlich gemeldet.
- Das Beobachten-Feld im Debug-Panel zeigt ein einzelnes Byte an einer Adresse.
- Eingebundene Dateien meldeten Fehler, die im Zusammenhang der einbindenden Datei keine waren.
- Die Gliederung zeigt Labels im BASIC ohne Zeilennummern.
- Dazu viele kleinere Korrekturen und Verbesserungen.

### Bekannte Einschränkungen

- Die Pakete wurden in frischen Systemen geprüft, aber nicht in allen Umgebungen: echter Desktop-Betrieb unter Linux, Installation auf fremden Rechnern, andere Windows- und Linux-Versionen.
- Der Extraktor (Zeichensätze und Bildschirme aus Programmen holen) und die Anbindung an ein echtes MEGA65 sind noch nicht enthalten.
- Die Umwandlung von BASIC nach Assembler ist experimentell.

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
