# Hinweise auf fremde Inhalte

VintageCoder steht unter der **GNU General Public License, Version 3 oder später** (siehe `LICENSE`, SPDX: `GPL-3.0-or-later`).
Diese Datei nennt Herkunft und Lizenz von Inhalten, die nicht aus eigener Hand stammen oder aus fremden Quellen erzeugt wurden.
Sie ist eine technische Zusammenstellung, keine Rechtsberatung.

## 1. Fremder Code im Repository

| Inhalt | Herkunft | Lizenz |
|---|---|---|
| `examples/atari/music_demo/lib/rmtplayr.asm`, `rmt.inc` sowie `examples/atari/jumprun/lib/rmtplayr.asm`, `rmt.inc` | RMT-Player (Raster Music Tracker 1.28, `rmtplayr.a65`), © Radek Štěrba / Raster / C.P.U. 2002–2009, http://raster.atari.org. In VintageCoder-Syntax übertragen | laut Dateikopf **GPLv3**; der Kopf sagt nicht, ob „nur v3“ oder „v3 oder später“ – die Dateien behalten ihren eigenen Vermerk |

## 2. Aus fremden Quellen erzeugte Daten

Registeradressen, Opcode-Tabellen und Ähnliches sind technische Fakten; hier steht, woraus sie erzeugt oder übernommen wurden.

| Datei | Quelle | Lizenz der Quelle |
|---|---|---|
| `data/cpus/45gs02.cpu.json` | Opcode-Namen, Adressierungsarten und 40-MHz-Zyklen aus den Tabellen des Xemu-Projekts (lgblgblgb/xemu), Generator `tools/gen_cpu_mega65.py` | GPL-2.0-or-later |
| `data/chips/vic4`, `dmagic`, `mega65` (`*.chip.json`) | E/A-Karte `iomap.txt` aus MEGA65/mega65-core, Generator `tools/gen_chips_mega65.py` | GPL-3.0 |
| `data/basic/*.dialect.json` (BASIC 7.0, 3.5, 65) | Schlüsselwörter und Tokencodes mit `petcat` aus VICE ermittelt, Generator `tools/gen_dialect.py` | VICE: GPL-2.0-or-later |
| `data/palettes/atari.json` | mit atari800 7.2.1 gemessen | atari800: GPL-2.0-or-later |
| `data/palettes/vic20.json` | VIC-I-Palette nach Colodore (PAL), aus `VIC20/colodore_vic.vpl` von VICE | VICE: GPL-2.0-or-later; Palette nach dem Colodore-Verfahren von Pepto |
| `data/palettes/c64.json` | C64-Palette nach Colodore (Pepto) | – |
| `data/sys/**/*.asm` (Registernamen, KERNAL-Tabellen, Beschreibungen) | zusammengestellt aus den Handbüchern und Speicherplänen der Systeme, KERNAL-Adressen am ROM von VICE geprüft; die Beschreibungen sind eigene Formulierungen | – |
| `data/targets/{c64,c128,atari8}.target.json`, Abschnitt `banking` (Bank-Presets) | Technische Fakten aus Handbüchern: C64 Programmer's Reference Guide (Prozessorport `$01`), C128 Programmer's Reference Guide (MMU-Konfigurationsregister), Atari XL/XE Hardware Manual (PIA PORTB). Bitnamen und Registeradressen gegen `data/sys/**` und `data/chips/*` geprüft; die Beschreibungen sind eigene Formulierungen | – |

**Original-ROMs sind nicht Teil von VintageCoder.** KERNAL, BASIC, Zeichensätze und Betriebssysteme der Systeme (Commodore, Atari, MEGA65 u. a.) werden weder mitgeliefert noch heruntergeladen; der Anwender legt eigene Abbilder in den ROM-Ordner (Hilfe „ROM-Dateien“). `data/roms/known.json` enthält nur Größen und Prüfsummen (CRC-32, SHA-1), keine ROM-Daten. Die Rechte an den Zeichenformen und den ROMs liegen bei den Herstellern bzw. deren Rechtsnachfolgern.

## 3. Programmbibliotheken (Rust)

Ausgewertet wurde `cargo metadata --format-version 1` für den Stand der `Cargo.lock` vom 2026-11-12, alle Plattformen (macOS, Windows, Linux) zusammen: **489 Pakete** außer den zwölf eigenen. Jedes Paket nennt eine Lizenz im Feld `license`; **keines fehlt**. Die Lizenzausdrücke enthalten diese SPDX-Kennungen (Anzahl der Pakete, in deren Ausdruck sie vorkommen; bei „OR“ genügt eine der Alternativen):

| Kennung | Pakete | Kennung | Pakete |
|---|---|---|---|
| MIT | 442 | BSD-2-Clause | 4 |
| Apache-2.0 (auch mit LLVM-Ausnahme) | 339 | BSL-1.0 | 2 |
| Zlib | 25 | LGPL-2.1-or-later (nur als Alternative) | 2 |
| Unicode-3.0 | 19 | 0BSD, ISC | je 1 |
| Unlicense | 6 | OFL-1.1, Ubuntu-font-1.0 (Schriften, siehe unten) | je 1 |
| BSD-3-Clause | 6 | GPL-2.0-only (nur als Alternative) | 1 |

Alle Pakete sind unter mindestens einer erlaubenden Lizenz nutzbar, die mit der GPL-3.0 vereinbar ist. Besonderheiten:

- `self_cell` steht unter „Apache-2.0 OR GPL-2.0-only“ und wird unter Apache-2.0 verwendet.
- `r-efi` (zwei Versionen) steht unter „MIT OR Apache-2.0 OR LGPL-2.1-or-later“ und wird unter MIT oder Apache-2.0 verwendet (nur bei UEFI-Zielen gebraucht).
- `clipboard-win` und `error-code` (nur Windows) stehen unter BSL-1.0 (Boost Software License).
- `epaint_default_fonts` bringt Schriften mit eigenen Lizenzen mit (siehe unten).

Für die Weitergabe von Binärpaketen verlangen MIT, Apache-2.0 und verwandte Lizenzen, dass Urheberrechtsvermerke und Lizenztext erhalten bleiben. Die vollständigen Lizenztexte der Pakete stehen in deren Quellpaketen (Cargo-Verzeichnis, `cargo vendor` oder crates.io). Die Liste lässt sich jederzeit neu erzeugen: `cargo metadata --format-version 1` (Felder `name`, `version`, `license`, `repository`) oder, wenn installiert, `cargo deny list` / `cargo license`. **Zu prüfen:** ob die Pakete (Windows-Installer, `.dmg`, `.deb`/`.rpm`/`tar.gz`) die Urheberrechtsvermerke der Rust-Bibliotheken selbst mitliefern; diese Datei nennt sie nur zusammengefasst, nicht je Paket.

### Eingebettete Schriften (eigene Auswahl, `crates/vc-app/assets/fonts/`)

| Schrift | Version | Quelle | Lizenz |
|---|---|---|---|
| Inter (Regular, SemiBold) | 3.19 | github.com/rsms/inter, © The Inter Project Authors | SIL Open Font License 1.1 |
| JetBrains Mono (Regular, Bold) | 2.304 | github.com/JetBrains/JetBrainsMono, © The JetBrains Mono Project Authors | SIL Open Font License 1.1 |

Die Schriften sind unverändert eingebettet. Die OFL erlaubt Einbetten und Weitergabe zusammen mit Software, auch kommerziell, solange die Schrift nicht für sich allein verkauft wird und der Lizenztext beiliegt (`Inter-OFL.txt`, `JetBrainsMono-OFL.txt`). Weder Inter noch JetBrains Mono tragen einen reservierten Schriftnamen. Die Schriften sind ein eigenes, getrenntes Werk und werden nicht Teil des GPL-Programmcodes.

### Eingebettete Schriften (Paket `epaint_default_fonts`, von egui)

| Schrift | Lizenz |
|---|---|
| Hack (Hack-Regular) | MIT-artige Hack-Lizenz |
| Ubuntu Light | Ubuntu Font Licence 1.0 |
| Noto Emoji | SIL Open Font License 1.1 |
| emoji-icon-font | MIT |

Die Lizenztexte liegen im Quellpaket (`fonts/*.txt` in `epaint_default_fonts`) und sind bei der Weitergabe des Programms beizulegen.

## 4a. Logo, Symbole und Namen

- `crates/vc-app/assets/drueeke-logo-light.png`: Logo von drueeke.de, Eigenwerk des Projektinhabers (Christian Drüeke), in der Seite „Über VintageCoder“ angezeigt. Es liegt im Repository, damit sich das Programm bauen lässt; das **Logo** und der **Name „VintageCoder“** sind von der GPL **ausgenommen**: Sie dürfen nicht frei verwendet werden (die GPL gilt für den Programmcode, nicht für Logo und Namen). Fragen dazu an vintagecoder@drueeke.de.
- Das Programmsymbol (`installer/macos/VintageCoder.icns`, `VintageCoder-1024.png`, `installer/windows/VintageCoder.ico`, das Linux-Symbol) wird von `tools/gen_icon.py` erzeugt und ist ein Eigenwerk (Standardbibliothek, keine Vorlage).
- „VintageCoder“ ist der Name des Projekts (siehe oben); „vcc“ und „vc-lsp“ sind Namen der zugehörigen Programme.

## 4b. Beispiele und Testdaten

- `examples/`: Beispielprojekte des Projekts unter der Lizenz von VintageCoder. In den Beispielen sind keine Urheberhinweise Dritter vermerkt; der einzige bekannte Fremdanteil ist der RMT-Player (Abschnitt 1). Einige Beispiele folgen bekannten Spielprinzipien (zum Beispiel „Prallball“ als Breakout-ähnliches Spiel).
- `crates/vc-emu/tests/data/*.bin`: UDP-Pakete, die der Projektinhaber an einem eigenen Ultimate 64 II aufgezeichnet hat (Firmware 3.14d); sie enthalten keine ROM-Daten.
- `crates/vc-basic/tests/fixtures/`: kleine BASIC-Quellen mit zugehörigen Tokenproben (`.prg`, `.bin`) für die Tokenizer-Tests; keine ROM-Auszüge (Größen 0,2 bis 0,7 KB).
- `crates/vc-media/tests/fixtures/atari800-window.png`: Bildschirmfoto eines Fensters des Emulators atari800 (GPL-2.0-or-later) für einen Importtest.

## 4. Dateiformate und Marken

VintageCoder liest und schreibt Formate anderer Werkzeuge (u. a. RMT, GoatTracker `.sng`, SID-Dateien `.sid`, ATR, D64/D71/D81, CRT, XEX). Die Namen sind Marken ihrer jeweiligen Inhaber; die Implementierung dafür ist eigen.

## 5. Nicht mitgelieferte Programme

VICE, Xemu, atari800, Altirra, Fujisan und das MEGA65-ROM werden nicht mit ausgeliefert, sondern vom Benutzer installiert und als eigene Prozesse gestartet. Auf ausdrücklichen Aufruf (Einstellungen oder `vcc emulator install`) lädt VintageCoder eine feste Version von der Seite des jeweiligen Projekts (Pakete und SHA-256-Summen: `data/emulators/catalog.json`) in den Konfigurationsordner des Benutzers; die Lizenzen der Pakete (VICE, atari800, Altirra, Xemu: GPL-2.0-or-later) gelten unverändert, VintageCoder verteilt sie nicht.

**Fujisan** (https://github.com/pedgarcia/fujisan, Atari-8-Bit-Emulator mit TCP-Fernsteuerung): `vcc emulator install fujisan` (und die Einstellungen) laden auf ausdrücklichen Aufruf die feste Version 2.0.5 aus den GitHub-Releases des Projekts (SHA-256 geprüft) in den Konfigurationsordner des Benutzers; VintageCoder verteilt das Paket nicht. Der Fujisan-Quelltext steht unter MIT (Paulo Garcia), das gelieferte Programm enthält libatari800 (GPL-2.0-or-later, „oder später“ in den Kopfzeilen geprüft) sowie Qt 5.15.2 (LGPL) und weitere Bibliotheken; das Paket enthält selbst keine Lizenztexte. Fujisan läuft als eigener Prozess und wird nur über TCP gesteuert. Die in libatari800 eingebauten Ersatz-ROMs (Altirra-OS, Altirra-BASIC, © Avery Lee) tragen im Quelltext (`src/roms/altirraos_xl.c`, `altirra_basic.c`) den Vermerk, dass Kopieren und Weitergeben mit oder ohne Änderung ohne Gebühr erlaubt ist, sofern Copyright- und dieser Hinweis erhalten bleiben (geprüft 2026-11-10). Eine Auslieferung mit VintageCoder ist noch nicht vorgesehen; dafür wären Lizenztexte, ein Quellcode-Angebot und die Zustimmung des Autors zu Name und Logo nötig (siehe Konzept „Fujisan“).

### Freie Ersatz-ROMs (Open-ROMs, Altirra) und Ersatzschrift

- **Nicht mitgeliefert:** Open-ROMs (Projekt MEGA65/open-roms), Altirra-Firmware und die ROMs der Emulatoren. Geprüft am Original (LICENSE von Open-ROMs, abgerufen 2026-11-08): Open-ROMs steht unter der GNU LGPL, Version 3 oder später (Copyright Paul Gardner-Stephen, Roman Standzikowski), einzelne BASIC-Dateien tragen zusätzlich einen MIT-Vermerk von Microsoft ohne Angabe, welche; die Herkunft der Schrift „PXL-Font“ ist dort nicht erklärt. Wegen dieser Unklarheit werden die Abbilder nicht verteilt. Die Lizenz der Altirra-OS-Abbilder wurde im Original nicht geprüft; atari800 bringt eigene Ersatz-ROMs mit, VintageCoder liefert keine.
- Anwender können Open-ROMs oder Altirra-Abbilder selbst in den ROM-Ordner legen; sie werden als „Ersatz (frei)“ erkannt.
- **Eingebaute Ersatzschrift** (`crates/vc-roms/src/fallback.rs`): Eigenbau von VintageCoder (5×7-Zeichen in 8×8-Zellen, Buchstaben, Ziffern, Satzzeichen), unter der Lizenz von VintageCoder. Sie ist keine Nachbildung der Schrift eines Systems.
