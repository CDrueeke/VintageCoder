<!--
Vorlage für die Release-Notiz (Text für GitHub, Website, Weitergabe). Nicht Teil des Pakets.
Ergänzt den Bericht `release-report.md`, den `tools/release.py stage` im Staging schreibt: Der Bericht liefert die Auswahl,
die Warnungen und die „Hinweise für die Release-Notizen“; diese Vorlage fasst sie in Worte für Anwender.
Spitze Klammern <…> ersetzen, nicht Zutreffendes löschen. Die Versionsnummer (Beispiel `0.1.0`, Tag `v0.1.0`) und das Datum trägt der Projektinhaber ein.
-->

# VintageCoder <VERSION, z. B. 0.1.0>

Veröffentlicht am <DATUM>. Entwicklungsstand: <z. B. Alpha, siehe unten>.

## Was ist dabei

- **Programme:** `vcc` (Kommandozeile mit Assembler, BASIC, Bau und Test) und `vc-lsp` (Sprachserver) <und `vintagecoder` (Entwicklungsumgebung)>.
- **Zielsysteme:** <aus „Auswahl“ im Bericht: C64, Atari 8 Bit, MEGA65, Plus/4 …>
- **Studios:** <Sprites, Zeichensätze, Bildschirme, Kacheln, Paletten, Tracker, PLAY-Tracker, Disk-Studio, Memory Map> / keine (nur Kommandozeile)
- **Anbindungen:** <VICE, atari800, Altirra, Fujisan, Xemu, Ultimate 64> / keine
- **Beispiele:** <Pakete aus dem Bericht> / keine
- **Betriebssysteme:** <Windows x86_64 (Installer), macOS arm64 (`.dmg`), Linux x86_64 (`.deb` Debian 13+/Ubuntu 24.04+, `.rpm` Fedora 45+/openSUSE Tumbleweed, `.tar.gz`)>. Intel-Macs sowie ARM unter Windows und Linux werden nicht unterstützt.

Emulatoren und ROM-Dateien gehören nicht zum Paket. VintageCoder erkennt vorhandene Emulatoren oder lädt sie auf Wunsch portabel nach (Einstellungen ▸ Emulatoren).

## Hinweise zum Stand

Aus dem Abschnitt „Hinweise für die Release-Notizen“ des Berichts:

- <Zeilen mit Stand Beta: `KENNUNG` – was noch nicht praktisch geprüft ist>
- <feste Bestandteile mit Hinweis: `BAS-B2A` (basic2asm, In Arbeit), `ASM-CART` (Module, wenig auf echter Hardware geprüft), `ASM-45GS02` (MEGA65, Beta)>
- <Zeilen unter Beta, nur bei `--allow-unfinished`: Kennung und Stand>

## Bekannte Einschränkungen

- Abgewählte Teile sind ausgeblendet, nicht aus dem Programm entfernt.
- Geteilte Datendateien (Vorlagen, Profile, ROM-Liste, Paletten) nennen auch Systeme, die dieses Release nicht enthält; das Programm ignoriert sie.
- <Warnungen aus dem Bericht, z. B. „System ohne Emulator oder Gerät: nur Bau, kein Start“>
- <Plattformen, die nicht praktisch geprüft wurden (siehe `STATUS.md`)>

## Änderungen

Zusammenfassung aus dem Abschnitt der Version im `CHANGELOG.md`:

- <Neues>
- <Verbessertes>
- <Behobenes>

## Installation

- **Windows:** Installer ausführen, Optionen nach Wunsch.
- **macOS:** Programmpaket in „Programme“ ziehen <oder Kommandozeilen-Archiv entpacken und „Kommandozeile installieren“ ausführen>.
- **Linux:** beide Pakete der Distribution (`vintagecoder-cli` und `vintagecoder`) mit `apt`, `dnf` oder `zypper` installieren (Befehle im Handbuch, Kapitel „Installation“), oder das Archiv `tar.gz` entpacken.

Prüfsummen aller Dateien: `SHA256SUMS` (Linux: `sha256sum -c SHA256SUMS`, macOS: `shasum -a 256 -c SHA256SUMS`).

## Pakete sind nicht signiert

Kein Paket ist signiert oder beglaubigt. Deshalb warnen Windows und macOS beim ersten Start:

- **macOS:** Rechtsklick auf VintageCoder ▸ **Öffnen**, im Dialog noch einmal **Öffnen** (neuere macOS-Versionen: *Systemeinstellungen ▸ Datenschutz & Sicherheit ▸ Trotzdem öffnen*).
- **Windows:** SmartScreen ▸ **Weitere Informationen ▸ Trotzdem ausführen**.
- **Linux:** die `.deb`- und `.rpm`-Pakete sind nicht signiert (openSUSE: `zypper` fragt nach oder `--allow-unsigned-rpm`).

Lade nur aus diesem Release oder von der Website des Projekts und prüfe die Summen.

## Quelltext und Herkunft der Pakete

VintageCoder steht unter der GNU General Public License, Version 3 oder später. Der vollständige Quelltext des hier veröffentlichten Standes steht im Tag `<TAG, z. B. v0.1.0>` dieses Repositorys. Die Pakete dieses Releases wurden **vor** der Veröffentlichung dieses Repositorys aus privaten Entwicklungsständen gebaut (<Linux `f44038a`, macOS `a10337b`, Windows `3dae7a4`>); diese Stände sind in diesem Repository nicht als Commits enthalten. Damit du den Quelltext der Binärdateien hast, hängen wir ihn als Archive an: `<vintagecoder-0.1.0-quelle-linux-f44038a.tar.gz>` (Linux-Pakete), `<…-macos-a10337b.tar.gz>` (macOS), `<…-windows-3dae7a4.tar.gz>` (Windows). Der Programmcode ist bis auf folgende Unterschiede derselbe wie im veröffentlichten Stand: <**Linux** zeigt in der Hilfe zusätzlich die Entwicklerseite „Release-Auswahl“; **macOS** sucht die Programmdaten noch nicht unter `<Präfix>/share/vintagecoder/data` (Daten-Suchpfad für Linux) und hat dieselbe Hilfeseite wie Linux; Windows entspricht dem Tag>. Die Archive und die Datei `SHA256SUMS` (Prüfsummen aller Anhänge) liegen neben den Paketen. Der Quelltext ist für alle Empfänger kostenlos und ohne weitere Bedingungen verfügbar, solange diese Release-Seite besteht.

Logo und Name „VintageCoder“ sind von der GPL ausgenommen (siehe `THIRD-PARTY-NOTICES.md`).

Fehler melden: in der IDE unter Hilfe ▸ „Feedback senden …“ oder mit `vcc feedback`.
