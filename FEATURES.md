# VintageCoder: Release-Auswahl

Hier wird festgelegt, was ein Release enthält. **Häkchen setzen** (`[x]`), dann prüfen und bauen:

```
python3 tools/release.py check                         # Auswahl, Abhängigkeiten, Dateien prüfen
python3 tools/release.py plan                          # zeigt, was ausgeliefert würde
python3 tools/release.py stage --os windows|macos|linux --out build/stage-<os> --build
```

Die Kommandozeile `vcc` mit Assembler, BASIC, Bau, Test und dem Sprachserver `vc-lsp` ist **immer** dabei. Entwicklungsstände, Release-Kriterien und Lücken stehen in `STATUS.md`; die Technik (Abhängigkeiten, Dateien je Zeile) in `release/registry.json`. Eine Auswahl gilt für alle drei Betriebssysteme. Die Versionsnummer vergibt nur der Projektinhaber (`CHANGELOG.md`).

**Stand 2026-11-12:** Gewählt (`[x]`) ist der Umfang der Alpha-Version 0.1.0 (C64, Atari 8 Bit, MEGA65, Plus/4 mit Studios und Beispielen). Abgewählt sind C128, VC-20, PET, CBM-II, echte MEGA65-Hardware und der Extraktor; sie sind im Quelltext vorhanden, aber in den Paketen ausgeblendet. Die Stände sind keine vollständige manuelle Abnahme; was je Paket geprüft wurde, steht in `STATUS.md`.

Ausführlich: IDE-Hilfe „Release-Auswahl“ und `docs/release-auswahl.md`; Vorlage für die Release-Notiz: `release/notes-template.md`.

Wählbar sind nur die Zeilen der Tabellen unten (Kennung in Backticks, am Ende das Feld „Release“). Fehlt eine Abhängigkeit, bricht `check` ab und nennt, was anzukreuzen ist.

<!--
Muster einer Zeile (Kommentare werden vom Skript ignoriert):

| Kennung | Inhalt | Stand | Release |
|---|---|---|---|
| `STU-BEISPIEL` | Beschreibung in einer Zeile | Beta | [x] |

- Die Kennung steht in Backticks und genau einmal in dieser Datei und in release/registry.json.
- Angehakt gilt: [x] [X] [ x] [x ] [ X] [X ]. Nicht angehakt: [ ] oder leer.
- Stand: Stabil, Beta, In Arbeit, Spike, Konzept. Gewählte Zeilen unter Beta bricht das Skript ab (Option --allow-unfinished).
- Text außerhalb der Tabellen ist frei.
-->

## 1. Programm

| Kennung | Inhalt | Stand | Release |
|---|---|---|---|
| `IDE-APP` | **Entwicklungsumgebung** `vintagecoder` (Editor, Projektbaum, Debugger, Einstellungen). Ohne sie entsteht ein reines Kommandozeilenpaket. | Stabil | [x] |

## 2. Studios (nur wirksam mit `IDE-APP`)

| Kennung | Studio | Stand | Release |
|---|---|---|---|
| `STU-SPRITE` | Sprites und Player/Missile | Stabil | [x] |
| `STU-CHAR` | Zeichensätze | Stabil | [x] |
| `STU-SCREEN` | Bildschirme | Stabil | [x] |
| `STU-TILES` | Kacheln und Screen-Bank | Beta | [x] |
| `STU-PALETTE` | Paletten | Stabil | [x] |
| `STU-TRACKER` | Tracker (SID, POKEY) | Beta | [x] |
| `STU-PLAY` | PLAY-Tracker | Beta | [x] |
| `STU-DISK` | Disk-Studio (D64, D71, D81, ATR) | Stabil | [x] |
| `STU-MEMMAP` | Memory Map | Stabil | [x] |
| `STU-EXTRACT` | Extraktor (PRG/XEX/Speicherabbild → Zeichensätze, Bildschirme) | Konzept | [ ] |

## 3. Anbindungen (Emulatoren und echte Geräte)

Gemeint ist die **Anbindung im Programm** (Erkennung, Parameterkatalog, Einstellungen, Debugger/Test), nicht das Mitliefern der Emulatoren. Emulatoren und ROMs werden nie ausgeliefert.

| Kennung | Anbindung | Stand | Release |
|---|---|---|---|
| `EMU-VICE` | VICE (`x64sc`, `x128`, `xplus4`, `xvic`, `xpet`, `xcbm2`) | Stabil | [x] |
| `EMU-ATARI800` | atari800 | Stabil | [x] |
| `EMU-ALTIRRA` | Altirra | Beta | [x] |
| `EMU-FUJISAN` | Fujisan | Beta | [x] |
| `EMU-XEMU` | Xemu (MEGA65) | Beta | [x] |
| `HW-ULTIMATE` | Ultimate 64/II+ (REST, Datenströme) | Beta | [x] |
| `HW-MEGA65` | MEGA65 echtes Gerät (serieller Monitor) und Netz | In Arbeit | [ ] |

## 4. Zielsysteme

| Kennung | Zielsystem | Stand | Release |
|---|---|---|---|
| `SYS-C64` | Commodore 64 | Stabil | [x] |
| `SYS-ATARI` | Atari 8 Bit | Stabil | [x] |
| `SYS-MEGA65` | MEGA65 | Beta | [x] |
| `SYS-PLUS4` | Commodore Plus/4 | Beta | [x] |
| `SYS-C16` | Commodore 16 / 116 (16 KB, braucht `SYS-PLUS4`) | Beta | [x] |
| `SYS-C128` | Commodore 128 | Beta | [x] |
| `SYS-VC20` | VC-20 | Beta | [x] |
| `SYS-PET` | PET/CBM (BASIC) | Spike | [x] |
| `SYS-CBM2` | CBM-II (nur BASIC) | Spike | [x] |

## 5. Beispielprojekte (je gewähltem Zielsystem)

Ein Beispiel wird nur ausgeliefert, wenn sein Paket **und** das Zielsystem seines Ordners gewählt sind.

| Kennung | Beispiele | Stand | Release |
|---|---|---|---|
| `EX-HELLO` | Kleinstbeispiele je System (`hello`, `basic_hello`, `basic_native`, `border` …) | Stabil | [x] |
| `EX-JUMPRUN` | Hugo & Gumbo | Stabil | [x] |
| `EX-PRALLBALL` | Prallball für alle sechs Systeme | Stabil | [x] |
| `EX-DEMOS` | Rasterbalken, Sprites, Musik, Kacheln, Module, Tests | Stabil | [x] |
