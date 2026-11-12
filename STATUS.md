# VintageCoder: Entwicklungsstand

Stand: 2026-11-12 · Version: 0.1.0 (Alpha), Pakete fertig, Veröffentlichung als GitHub-Pre-release `v0.1.0` vorbereitet

Was in ein Release kommt, wird **nur in `FEATURES.md`** angehakt. Diese Datei hat keine Häkchen: sie führt die Stände der wählbaren und der festen Bestandteile, die Release-Kriterien und die bekannten Lücken. Die Versionsnummer vergibt ausschließlich der Projektinhaber; Änderungen folgen dem Entwicklungsstand (`CHANGELOG.md`, Abschnitt „Unveröffentlicht“).

## Stand der Version 0.1.0 (Alpha)

VintageCoder 0.1.0 ist ein **Alpha-Release**: Es enthält mit Sicherheit noch Fehler, und große Teile sind nur durch automatische Tests und einzelne praktische Läufe abgesichert, nicht durch eine vollständige manuelle Abnahme.

**Enthalten** sind laut Release-Auswahl (`FEATURES.md`) C64, Atari 8 Bit, MEGA65 (Emulator Xemu) und Plus/4 mit allen Studios, der Kommandozeile `vcc` und dem Sprachserver `vc-lsp`. **Abgewählt** (im Quelltext vorhanden, in den Paketen ausgeblendet): C128, VC-20, PET, CBM-II, echte MEGA65-Hardware, Extraktor.

**Pakete** (elf Dateien, alle unsigniert; Windows und Linux nur x86_64, macOS nur arm64; Intel-Macs sowie ARM unter Windows und Linux werden nicht unterstützt):

| Paket | Quellcommit | Geprüft | Nicht geprüft |
|---|---|---|---|
| Windows `Setup.exe` | `3dae7a4` | Staging-Prüfung, stille Installation und Deinstallation, `vcc --version`, `vcc build`, Start der IDE mit Bildschirmfoto-Selbsttest (Windows 11) | Aufgaben „Beispiele“, „Desktop“, „PATH“; Installation für alle Benutzer; Update über eine vorhandene Installation; interaktiver Dialog; Windows 10, 32 Bit, ARM; Reaktion von SmartScreen; Bedienung der IDE von Hand |
| Debian 13 `.deb` | `f44038a` | frischer Container: Dateiliste, Installation mit `apt`, `vcc`, Beispielbau, IDE-Start unter `xvfb` mit Software-Darstellung, Entfernen | echter Desktop (GPU, Sitzung, Ton, Dateidialoge); Debian 12 und älter; Emulator-Anbindung |
| Ubuntu 24.04 `.deb` | `f44038a` | wie Debian | Ubuntu 22.04 und älter, 26.04; Rest wie Debian |
| Fedora 45 `.rpm` | `f44038a` | wie Debian, mit `dnf` | ältere Fedora-Versionen (glibc 2.44 nötig); Rest wie Debian |
| openSUSE Tumbleweed `.rpm` | `f44038a` | wie Debian, mit `zypper` | Leap; Rest wie Debian |
| Linux `.tar.gz` | `f44038a` | Entpacken, `vcc --version`, `ldd` für `vcc` (Ubuntu 24.04) | die IDE aus dem Archiv; andere Distributionen |
| macOS `.dmg` (arm64) | `a10337b` | kein Prüfstandsbericht vorhanden | alles (Beglaubigung, Gatekeeper, Start auf frischem Mac) |

Die Linux-Pakete wurden aus `f44038a` gebaut; der Quelltext unterscheidet sich vom Veröffentlichungsstand nur in der Hilfeseite „Release-Auswahl“ (`crates/vc-ide/src/help.rs`, `crates/vc-ide/help/release-auswahl.md`): Sie wird in den Linux-Paketen zusätzlich angezeigt.

**Bekannte Einschränkungen:** keine Signierung und Beglaubigung (Windows SmartScreen und macOS Gatekeeper warnen); Hardware-Anbindungen (Ultimate, MEGA65) sind nur teilweise an echten Geräten geprüft; die Dokumentation ist deutsch.

## Entwicklungsstände

| Stand | Bedeutung |
|---|---|
| **Stabil** | Umgesetzt, durch Tests abgedeckt und praktisch geprüft (Emulator oder Gerät). Für ein Release geeignet. |
| **Beta** | Umgesetzt und getestet, aber nur teilweise praktisch geprüft oder mit bekannten Lücken. Release mit Hinweis möglich. |
| **In Arbeit** | Teilweise umgesetzt oder auf einem Entwicklungszweig, noch nicht im Hauptzweig vollständig. |
| **Spike** | Machbarkeitsversuch, nicht für Anwender gedacht. |
| **Konzept** | Nur geplant (Idee oder Entwurf), noch keine Umsetzung. |
| **Zurückgestellt / verworfen** | Bewusst nicht verfolgt. |

Wo „geprüft“ steht, ist gemeint, was in der Entwicklung wirklich ausprobiert wurde. Alles andere ist nur durch automatische Tests abgedeckt.

**Hinweis zur Einstufung:** Die Stände beruhen auf den Berichten der Entwicklung, den Tests und dem Quellstand. Sie sind keine vollständige manuelle Abnahme jeder Funktion. Der Projektinhaber korrigiert sie, wo sein eigener Eindruck abweicht. Die Spalte „Stand“ in `FEATURES.md` ist maßgeblich für die Prüfung durch `tools/release.py`.

## Release-Kriterien

| # | Kriterium | Geprüft von |
|---|---|---|
| 1 | `cargo build --workspace` ohne Fehler und ohne Warnungen, `cargo test --workspace` grün. | CI, von Hand |
| 2 | Feature hat eine Hilfeseite oder einen Abschnitt in der Hilfe und einen Eintrag im `CHANGELOG.md`. | von Hand |
| 3 | Mindestens ein Beispielprojekt oder Test zeigt das Feature. | `release.py check` (warnt, wenn ein Zielsystem ohne Beispiel gewählt ist) |
| 4 | Stand mindestens **Beta**; gewählte Zeilen unter Beta werden nicht ausgeliefert (Ausnahme: `--allow-unfinished`, im Bericht markiert). | `release.py check` und `stage` |
| 5 | Installer/Paket für das Betriebssystem (Windows: Inno Setup, macOS: Programmpaket (`.dmg`), Linux: Archiv `tar.gz`, `.deb`, `.rpm`). | `release.py stage` und Installer |

## So wird ein Release zusammengestellt

1. Stände prüfen und in `FEATURES.md` aktualisieren; in der Spalte „Release“ anhaken.
2. `python3 tools/release.py check` (Fehler beheben), `plan` ansehen.
3. `python3 tools/release.py stage --os <system> --out build/stage-<system> --build`; der Installer packt nur noch dieses Staging ein (`installer/vintagecoder.iss`, `installer/build-macos.sh`, `installer/linux/`).
4. `release-report.md` im Staging lesen: Warnungen und „Hinweise für die Release-Notizen“ gehören in die Release-Notiz (zusammen mit `CHANGELOG.md`).
5. Installer je Betriebssystem einmal von Hand starten (Prüfschritt, den das Skript nicht ersetzt). Versionsnummer und Tag legt der Projektinhaber fest.

## Hinweise zu den wählbaren Bestandteilen

Die Stände dazu stehen in `FEATURES.md`.

| Kennung | Hinweise |
|---|---|
| `IDE-APP` | Fasst die früheren Bestandteile `IDE-*` zusammen (siehe unten). |
| `SYS-C64` | Referenzsystem, am längsten in Arbeit. Hugo & Gumbo läuft. Enthält Assembler, BASIC V2, VICE, Ultimate 64, Studios, Beispiele. |
| `SYS-ATARI` | 400/800/XL/XE, PAL/NTSC: Assembler (`xex`), Atari BASIC, Turbo-BASIC XL, ANTIC/GTIA/POKEY, Player/Missile, RMT. atari800 und Altirra (Altirra-Katalog 32 Parameter, nur 5 praktisch geprüft); Debugging nur mit atari800 und eingeschränkt (ein Haltepunkt). |
| `SYS-MEGA65` | Assembler (45GS02), BASIC 65, Xemu, Zeichensätze, Hugo & Gumbo (Stufe 1, VIC-II-Modus). Echte Hardware per UART-Monitor angelegt, Gerät derzeit offline (nicht geprüft). VIC-IV-Vollfarb-Modus (FCM, „Stufe 2“) nicht begonnen. |
| `SYS-PLUS4` | TED, BASIC 3.5, `xplus4`. Hugo & Gumbo in `xplus4` geprüft. Zeichen-Sprites farblich ungenau. |
| `SYS-C128` | BASIC 7.0, VDC-Zeichensätze, `x128`. Nur Beispiele `border`, `tiles_demo`, `prallball`. Wenig praktisch geprüft. |
| `SYS-VC20` | Wählbarer Speicherausbau, BASIC V2, `xvic`. Tabellen laut Zieldaten „aus der Doku übernommen, unverifiziert“. |
| `SYS-PET` | Zieldaten, Vorlagen und BASIC-Beispiele (`pet/hello`, `pet/basic_hello`); nur als Daten angelegt. |
| `SYS-CBM2` | Zieldaten und BASIC-Beispiel (`cbm2/basic_hello`); nur als Daten angelegt, Hilfeseite gemeinsam mit PET. |
| `STU-SPRITE` | Import: SPD-Format, Rohdaten. |
| `STU-TILES` | Neu; optische Mängel der Studio-Fenster in Behebung (Standardgröße zu klein). |
| `STU-TRACKER` | Tracker-Knistern noch nicht abschließend geprüft; TED-Export nicht umgesetzt. Chips: SID, POKEY, 4× SID MEGA65. |
| `STU-DISK` | Abgleich mit `c1541`. |
| `STU-EXTRACT` | Geplant, noch nicht umgesetzt. |
| `EMU-VICE` | VICE vollständig (Debugger, Datenhaltepunkte); auf macOS geprüft. |
| `EMU-ATARI800` | Debugging eingeschränkt. |
| `EMU-ALTIRRA` | Parameterkatalog nur teilweise gegen das Programm geprüft. |
| `EMU-XEMU` | Parameterkatalog; ROM-Schalter nicht überall praktisch geprüft. |
| `HW-ULTIMATE` | Am echten Gerät geprüft (Firmware ≥ 3.15); ohne Register/Einzelschritt/Haltepunkte. Datenströme (früher `BLD-STREAMS`, In Arbeit): Grundlage umgesetzt; Windows-Firewall-Abfrage beim UDP-Empfang noch offen. |
| `HW-MEGA65` | Serieller Monitor (früher `BLD-M65HW`) und Ethernet (früher `BLD-NET65`): In Arbeit, Gerät offline, Entwicklungszweig, Attrappen. |
| `EX-JUMPRUN` | Alle vier in Emulator geprüft (Atari: Altirra; MEGA65: Xemu). |
| `EX-DEMOS` | Enthält die Daten-Beispiele ohne Quellen (`pmg_ship`, `standard_charsets` ×3, `play_demo`); sie bauen nicht. |

## Bestandteile ohne Auswahl (immer dabei)

Diese Bestandteile gehören zur IDE oder zur Kommandozeile und sind nicht wählbar. Das Betriebssystem ist keine Auswahl: es ergibt sich aus `stage --os`.

| Kennung | Bestandteil | Stand | Hinweise |
|---|---|---|---|
| `IDE-EDITOR` | Quelltext-Editor: Syntaxfarben, Falten, Vervollständigung, Hover, Live-Fehler (`vc-lsp`), Gehe zu Definition | Stabil | Assembler und BASIC. |
| `IDE-LAYOUT` | Andockbare Panels je Arbeitsmodus, Layout wird gespeichert, Modus-Leiste, Projektbaum | Stabil | |
| `IDE-WINDOWS` | Fenstersteuerung: Studios einbetten und lösen, Bereiche maximieren und wiederherstellen | Beta | Per Selbsttest geprüft. Maus-Ziehen, Menüs, Tastenkürzel, Windows und Linux nicht geprüft. |
| `IDE-DESIGN` | Designsystem „Retro Terminal IDE“ (dunkel und hell), eingebettete Schriften | Stabil | |
| `IDE-WELCOME` | Willkommensseite, Projekt-Wizard, Befehlspalette, Projekt-Übersicht | Stabil | |
| `IDE-SETTINGS` | Einstellungen: Zielsysteme, Toolchain, Emulatoren, Echte Maschinen, Startziel für F5 | Stabil | |
| `IDE-ARGS` | Kommandozeilen-Parameter der Emulatoren als Schalter/Auswahl mit Platzhaltern (englische Namen) | Beta | Katalog `data/args/`; Altirra nur teilweise gegen das Programm geprüft. Kommt mit jeder gewählten Emulator-Anbindung. |
| `IDE-ABOUT` | Seite „Über VintageCoder“ (Version, Lizenz, Danksagung) | Beta | In der IDE vorhanden (Hilfe ▸ Über VintageCoder), nicht auf allen Systemen praktisch geprüft. |
| `IDE-WIZARD2` | Konfigurationsassistent für den Erststart (Schnell- und Vollweg, Wiederaufruf) | Beta | Ersteinrichtung, Wiederaufruf, Teil-Assistenten, Export und Import der Einstellungen; unter Linux und Windows noch nicht ausgiebig praktisch geprüft. |
| `IDE-MAC` | macOS: native Menüleiste, eigene Titelleiste | Stabil | |
| `IDE-WIN` | Windows: Installer, `%APPDATA%`-Einstellungen, Pfade | Beta | F5 mit C64/Atari/MEGA65 auf Windows geprüft; Installer mit allen Optionen geprüft. |
| `IDE-LINUX` | Linux (Debian, Ubuntu, Fedora, openSUSE; X11 und Wayland) | In Arbeit | Pakete `.deb`, `.rpm` und `tar.gz` (x86_64) gebaut und in frischen Containern getestet; echter Desktop (GPU, Wayland/X11-Sitzung, Ton, Dateidialoge) und VMs nicht geprüft. |
| `ASM-CORE` | Eigener 6502-Assembler `vc-asm`: Mehrpass, Scopes, Makros, `.if`, `.repeat`, `.include`, Pseudo-PC, Listing, Illegals | Stabil | Hilfe: 7 Assembler-Seiten; 101 Beispiele geprüft. |
| `ASM-CYCLES` | `.cycles` (statische Zyklenzählung), Zyklenbudget mit Warnung | Stabil | Messung im Emulator nur C64 (PAL). |
| `ASM-BASICSTUB` | `.basicstub` mit Startadresse des Zielsystems ohne `.org` | Stabil | W204, E218. |
| `ASM-45GS02` | 45GS02-Befehle (MEGA65) | Beta | Hinweis für die Release-Notizen. |
| `ASM-CART` | Module/Cartridges: C64 Normal, Ocean, Magic Desk, EasyFlash u. a.; Atari-Module; VC-20 | Beta | Mehrere Typen, wenig auf echter Hardware geprüft. Hinweis für die Release-Notizen. |
| `BAS-NATIVE` | Natives BASIC `.bas` ohne Zeilennummern: Sprungmarken `@name`, `#include`, Dialekte CBM V2, 3.5, 7.0, 65, Atari BASIC, Turbo-BASIC XL | Stabil | Tokenizer byte-identisch mit `petcat`; Atari BASIC am ROM abgeglichen. |
| `BAS-LONGNAMES` | Lange Variablennamen `#longnames` (automatische Kürzung je Dialekt) | Beta | Opt-in; Debugger nutzt die Zuordnung noch nicht. |
| `BAS-B2A` | `basic2asm`: BASIC in Assembler übersetzen | In Arbeit | C64, C128, Plus/4; Atari-Ziel auf Entwicklungszweig. Hinweis für die Release-Notizen. |
| `BAS-SBAS` | Strukturiertes BASIC `.sbas` | Zurückgestellt / verworfen | Bau abgeschaltet. |
| `STU-TILELIB` | Assembler-Bibliothek für Kacheln: `tile_put`, `tile_draw`, `screen_select`, `screen_draw` (C64, Atari, Plus/4, VC-20, C128, MEGA65) | Beta | Liegt unter `data/sys/`, gehört zur Kommandozeile. Tests statisch und VICE/atari800; MEGA65 ohne Emulatorlauf. |
| `STU-ROMS` | ROM-Verzeichnis und -Oberfläche: Suche, Erkennung, Import aus VICE, Assistent, Ersatzschrift | Beta | Folgt den Emulator-Anbindungen. VICE auf macOS geprüft; Xemu/Altirra/atari800 mit ROM-Schaltern nicht überall praktisch geprüft; Oberfläche optisch teils noch zu korrigieren. |
| `BLD-BUILD` | Projektbau (`.vcprj`): Quellen, Export, Disk-Image, Fehlerliste, Speicherkarte | Stabil | |
| `BLD-DEBUG` | Debugger: Register, Flags, Disassembler, Speicher, Haltepunkte (bedingt, Treffer zählen), Datenhaltepunkte (VICE) | Stabil | Fähigkeit der Emulator-Anbindungen: VICE vollständig; atari800 eingeschränkt. |
| `BLD-TEST` | `vcc test`: statische Prüfungen (Größe, Zeropage, Überlappung, Zyklen), Emulator-/Trap-Tests | Beta | Emulator-Tests nur mit VICE für C64. |
| `DOC-HELP` | Hilfe in der IDE (F1, Suche): rund 45 Seiten; Assembler-Doku und BASIC-Doku | Stabil | 101 Assembler-Beispiele, 48 von 50 BASIC-Beispielen geprüft. Folgt künftig der Auswahl. |
| `DOC-WEB` | Website `vintagecoder.drueeke.de` mit Doku und Screenshots | Stabil | Live. Außerhalb des Pakets. |

## Ausblick und Verworfenes

| Kennung | Thema | Stand | Hinweise |
|---|---|---|---|
| `SYS-OTHER` | Weitere 8-Bit-Systeme (Atari 2600/5200/7800, NES, BBC Micro, Apple II …) | Konzept | Geplant, gestaffelt, noch nicht entschieden. `data/sys/vcs2600` wird nie ausgeliefert (Regel `never` in der Registry). |
| – | Z80-Systeme, Amiga, Atari ST | Zurückgestellt / verworfen | Entscheidung des Projektinhabers. |

## Release-Auswahl: Stand der Umsetzung

Bedienung: IDE-Hilfe „Release-Auswahl“ und `docs/release-auswahl.md`; Vorlage für die Release-Notiz: `release/notes-template.md`.

Umgesetzt (Stand 2026-11-12):

- `FEATURES.md` als Auswahl, `release/registry.json`, `tools/release.py` (`check`, `plan`, `stage`, `bootstrap`, `package`); die Installer packen das Staging.
- Studios und Anbindungen werden zur Laufzeit ausgeblendet (`vc_ide::features`, `data/release.json`, `VC_RELEASE`), der Emulator-Katalog ersetzt feste Abfragen.
- Zielsysteme sind robust gegen fehlende Targets; Tests und CI prüfen je Testauswahl ein Staging (`tools/ci_staging.py`).
- Hilfeseite, `docs/release-auswahl.md` und Vorlage für die Release-Notiz; Linux-Pakete `.deb` und `.rpm` (zusätzlich `tar.gz`), gebaut und in Containern getestet.

Die Auswahl wirkt in `vintagecoder` und `vcc` gleichermaßen. Nicht gewählt heißt: nicht erreichbar, aber nicht aus der Programmdatei entfernt.

## Bekannte Lücken und offene Punkte

- Die fünf Beispielprojekte ohne Quellen (`EX-DEMOS`, Daten-Beispiele).
- Linux: Test auf echten Desktops und in VMs fehlt; die Pakete sind nur in Containern geprüft (Abschnitt „Linux-Pakete“).
- macOS-Paket: kein Prüfstandsbericht.
- Keine Signierung (Windows, macOS), keine Pakete für Intel-Macs sowie ARM unter Windows und Linux.
- MEGA65: Gerät offline, FCM-Stufe 2 nicht begonnen.
- Altirra-Katalog nur teilweise gegen das Programm geprüft.
- Im Hauptzweig noch nicht eingeflossen: `basic2asm` für Atari, MEGA65 über Ethernet.
- Geplant, ohne Umsetzung: Extraktor, weitere 8-Bit-Systeme.
- Release-Auswahl: Abgewähltes ist ausgeblendet, nicht aus dem Programm entfernt (`VC_RELEASE=all` oder das Löschen von `data/release.json` bringt es zurück); geteilte Datendateien nennen auch nicht gewählte Systeme (Abschnitt „Release-Auswahl: Stand der Umsetzung“).

## Linux-Pakete

Je Distribution zwei Pakete (`vintagecoder-cli` für die Kommandozeile und `vintagecoder` für die Oberfläche) und dazu ein `tar.gz`, gebaut und in frischen Containern getestet: Debian 13, Ubuntu 24.04, Fedora 45 und openSUSE Tumbleweed (x86_64). **Nicht geprüft:** echter Desktop-Betrieb (Fenster, Dialoge, Klang). AppImage, Paketquellen und Signierung gibt es nicht; die Dateien werden zum Herunterladen angeboten.
