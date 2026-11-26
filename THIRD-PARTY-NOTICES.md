# Hinweise auf fremde Inhalte

VintageCoder steht unter der **GNU General Public License, Version 3 oder später** (siehe `LICENSE`, SPDX: `GPL-3.0-or-later`).
Diese Datei nennt Herkunft und Lizenz von Inhalten, die nicht aus eigener Hand stammen oder aus fremden Quellen erzeugt wurden.
Sie ist eine technische Zusammenstellung, keine Rechtsberatung.

## 1. Fremder Code im Repository

| Inhalt | Herkunft | Lizenz |
|---|---|---|
| `examples/atari/music_demo/lib/rmtplayr.asm`, `rmt.inc` sowie `examples/atari/jumprun/lib/rmtplayr.asm`, `rmt.inc` | RMT-Player (Raster Music Tracker 1.28, `rmtplayr.a65`), © Radek Štěrba / Raster / C.P.U. 2002–2009, http://raster.atari.org. In VintageCoder-Syntax übertragen | Lizenz laut Quell-Repository (VinsCool/RASTER-Music-Tracker, abgerufen 2026-11-18): **GNU GPL v3.0** (nur Version 3, kein „oder später“ vermerkt), SPDX `GPL-3.0-only`; Urheber Radek Štěrba / Raster / C.P.U. 2002–2009. Das Repository nennt sich „Unofficial version by VinsCool“ (Patch 16); unsere Dateien sind aus RMT 1.28 übertragen (siehe Dateikopf), nicht aus dieser Fassung. Die Dateien behalten ihren eigenen Vermerk Weitergeführt wird RMT seit 2026 im Repository https://github.com/raster-atari-org/RASTER-Music-Tracker (GPL v3; Credits: Radek Štěrba, Vin Samuel, Peter Dell); das oben genannte Repository von VinsCool ist archiviert. |

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
- `cpal` 0.18.2 (Apache-2.0) gibt in der neuen Oberfläche (`vc-gui`) die Hörprobe des Trackers über das Audiogerät aus; seine Abhängigkeiten (u. a. `coreaudio-rs` unter macOS, `alsa`/`jack` unter Linux, `windows` unter Windows) stehen unter erlaubenden Lizenzen. Die vollständige Nachführung aller Pakete der neuen Oberfläche folgt mit dem Lizenzpaket.
- Seit 2026-11-14 kommen für die Markdown-Vorschau und -Farben `pulldown-cmark` 0.13.6 (MIT) und `unicase` 2.10 (MIT oder Apache-2.0) hinzu; die Zahlen oben gelten für den Stand vom 2026-11-12. Lizenzprüfung am 2026-11-14 gegen crates.io; `pulldown-cmark` wird ohne die optionalen Pakete `getopts`, `serde` und `pulldown-cmark-escape` gebaut.

Die neue Oberfläche (Iced, wgpu, winit, cpal, rfd, muda, accesskit, tokio u. a.) hat ihre eigene, maschinell erzeugte Liste (Abschnitt „Pakete der neuen Oberfläche“ unten in diesem Abschnitt; Prüfung `tools/test_gui_notices.py` in der lokalen CI).

Für die Weitergabe von Binärpaketen verlangen MIT, Apache-2.0 und verwandte Lizenzen, dass Urheberrechtsvermerke und Lizenztext erhalten bleiben. Die vollständigen Lizenztexte der Pakete stehen in deren Quellpaketen (Cargo-Verzeichnis, `cargo vendor` oder crates.io). Die Liste lässt sich jederzeit neu erzeugen: `cargo metadata --format-version 1` (Felder `name`, `version`, `license`, `repository`) oder, wenn installiert, `cargo deny list` / `cargo license`. **Zu prüfen:** ob die Pakete (Windows-Installer, `.dmg`, `.deb`/`.rpm`/`tar.gz`) die Urheberrechtsvermerke der Rust-Bibliotheken selbst mitliefern; diese Datei nennt sie nur zusammengefasst, nicht je Paket.

### Eingebettete Schriften (eigene Auswahl, `crates/vc-gui-ui/assets/fonts/`)

| Schrift | Version | Quelle | Lizenz |
|---|---|---|---|
| Inter (Regular, SemiBold) | 3.19 | github.com/rsms/inter, © The Inter Project Authors | SIL Open Font License 1.1 |
| JetBrains Mono (Regular, Bold) | 2.304 | github.com/JetBrains/JetBrainsMono, © The JetBrains Mono Project Authors | SIL Open Font License 1.1 |

Die neue Web-Oberfläche (`ui/`) bündelt dieselben Dateien JetBrains Mono Regular und Bold unverändert (`ui/src/assets/fonts/`, Lizenztext `JetBrainsMono-OFL.txt` liegt dabei); Ligaturen sind per CSS ausgeschaltet.

Die Schriften sind unverändert eingebettet. Die OFL erlaubt Einbetten und Weitergabe zusammen mit Software, auch kommerziell, solange die Schrift nicht für sich allein verkauft wird und der Lizenztext beiliegt (`Inter-OFL.txt`, `JetBrainsMono-OFL.txt`). Weder Inter noch JetBrains Mono tragen einen reservierten Schriftnamen. Die Schriften sind ein eigenes, getrenntes Werk und werden nicht Teil des GPL-Programmcodes.

<!-- GUI-PAKETE:BEGIN -->

### Pakete der neuen Oberfläche (`vc-gui`, `vc-gui-ui`, `vc-gui-editor`, `vc-i18n`)

Erzeugt mit `python3 tools/gui_notices.py` aus `cargo metadata` (macOS, Windows, Linux, ohne Dev-Abhängigkeiten): **663 Pakete** außer den eigenen. Direkte Abhängigkeiten der drei Crates: `accesskit`, `accesskit_macos`, `cpal`, `fluent-bundle`, `iced`, `muda`, `objc2`, `objc2-foundation`, `raw-window-handle`, `rfd`, `serde`, `serde_json`, `tauri`, `tauri-build`, `tauri-plugin-dialog`, `tauri-plugin-single-instance`, `tokio`, `unic-langid`. Die Liste ist ein Auszug des Gesamtbestands oben und wird bei jeder Änderung der Abhängigkeiten neu erzeugt (Prüfung: `python3 -m unittest tools.test_gui_notices`).

| Paket | Version | Lizenz | Quelle |
|---|---|---|---|
| `ab_glyph` | 0.2.32 | Apache-2.0 | https://github.com/alexheretic/ab-glyph |
| `ab_glyph_rasterizer` | 0.1.10 | Apache-2.0 | https://github.com/alexheretic/ab-glyph |
| `accesskit` | 0.25.2 | MIT OR Apache-2.0 | https://github.com/AccessKit/accesskit |
| `accesskit_consumer` | 0.39.2 | MIT OR Apache-2.0 | https://github.com/AccessKit/accesskit |
| `accesskit_macos` | 0.27.2 | MIT OR Apache-2.0 | https://github.com/AccessKit/accesskit |
| `adler2` | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | https://github.com/oyvindln/adler2 |
| `ahash` | 0.8.12 | MIT OR Apache-2.0 | https://github.com/tkaitchuck/ahash |
| `aho-corasick` | 1.1.5 | Unlicense OR MIT | https://github.com/BurntSushi/aho-corasick |
| `aligned` | 0.4.3 | MIT OR Apache-2.0 | https://github.com/rust-embedded-community/aligned |
| `aligned-vec` | 0.6.4 | MIT | https://github.com/sarah-ek/aligned-vec/ |
| `alloc-no-stdlib` | 3.0.0 | BSD-3-Clause | https://github.com/dropbox/rust-alloc-no-stdlib |
| `alloc-stdlib` | 0.3.0 | BSD-3-Clause | https://github.com/dropbox/rust-alloc-no-stdlib |
| `alsa` | 0.11.0 | Apache-2.0/MIT | https://github.com/diwic/alsa-rs |
| `alsa-sys` | 0.4.0 | MIT | https://github.com/diwic/alsa-sys |
| `android-build` | 0.1.4 | MIT | https://github.com/project-robius/android-build |
| `anyhow` | 1.0.104 | MIT OR Apache-2.0 | https://github.com/dtolnay/anyhow |
| `arg_enum_proc_macro` | 0.3.4 | MIT | https://github.com/lu-zero/arg_enum_proc_macro |
| `arrayref` | 0.3.9 | BSD-2-Clause | https://github.com/droundy/arrayref |
| `arrayvec` | 0.7.8 | MIT OR Apache-2.0 | https://github.com/bluss/arrayvec |
| `as-raw-xcb-connection` | 1.0.1 | MIT OR Apache-2.0 | https://github.com/psychon/as-raw-xcb-connection |
| `as-slice` | 0.2.1 | MIT OR Apache-2.0 | https://github.com/japaric/as-slice |
| `ash` | 0.38.0+1.3.281 | MIT OR Apache-2.0 | https://github.com/ash-rs/ash |
| `async-broadcast` | 0.7.2 | MIT OR Apache-2.0 | https://github.com/smol-rs/async-broadcast |
| `async-channel` | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-channel |
| `async-executor` | 1.14.1 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-executor |
| `async-io` | 2.6.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-io |
| `async-lock` | 3.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-lock |
| `async-process` | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-process |
| `async-recursion` | 1.2.0 | MIT OR Apache-2.0 | https://github.com/dcchut/async-recursion |
| `async-signal` | 0.2.14 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-signal |
| `async-task` | 4.7.1 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-task |
| `async-trait` | 0.1.92 | MIT OR Apache-2.0 | https://github.com/dtolnay/async-trait |
| `atk` | 0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs |
| `atk-sys` | 0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs |
| `atomic-waker` | 1.1.2 | Apache-2.0 OR MIT | https://github.com/smol-rs/atomic-waker |
| `autocfg` | 1.5.1 | Apache-2.0 OR MIT | https://github.com/cuviper/autocfg |
| `av-scenechange` | 0.14.1 | MIT | https://github.com/rust-av/av-scenechange |
| `av1-grain` | 0.2.5 | BSD-2-Clause | https://github.com/rust-av/av1-grain |
| `avif-serialize` | 0.8.9 | BSD-3-Clause | https://github.com/kornelski/avif-serialize |
| `base64` | 0.21.7 | MIT OR Apache-2.0 | https://github.com/marshallpierce/rust-base64 |
| `base64` | 0.23.1 | MIT OR Apache-2.0 | https://github.com/marshallpierce/rust-base64 |
| `bit-set` | 0.8.0 | Apache-2.0 OR MIT | https://github.com/contain-rs/bit-set |
| `bit-vec` | 0.8.0 | Apache-2.0 OR MIT | https://github.com/contain-rs/bit-vec |
| `bit_field` | 0.10.4 | MIT OR Apache-2.0 | https://github.com/phil-opp/rust-bit-field |
| `bitflags` | 1.3.2 | MIT/Apache-2.0 | https://github.com/bitflags/bitflags |
| `bitflags` | 2.13.2 | MIT OR Apache-2.0 | https://github.com/bitflags/bitflags |
| `bitstream-io` | 4.10.0 | MIT/Apache-2.0 | https://github.com/tuffy/bitstream-io |
| `block` | 0.1.6 | MIT | http://github.com/SSheldon/rust-block |
| `block-buffer` | 0.12.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| `block2` | 0.5.1 | MIT | https://github.com/madsmtm/objc2 |
| `block2` | 0.6.2 | MIT | https://github.com/madsmtm/objc2 |
| `blocking` | 1.7.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/blocking |
| `brotli` | 9.1.3 | BSD-3-Clause AND MIT | https://github.com/dropbox/rust-brotli |
| `brotli-decompressor` | 6.3.2 | BSD-3-Clause/MIT | https://github.com/dropbox/rust-brotli-decompressor |
| `bs58` | 0.5.1 | MIT/Apache-2.0 | https://github.com/Nullus157/bs58-rs |
| `built` | 0.8.1 | MIT | https://github.com/lukaslueg/built |
| `bumpalo` | 3.20.4 | MIT OR Apache-2.0 | https://github.com/fitzgen/bumpalo |
| `bytemuck` | 1.25.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| `bytemuck_derive` | 1.12.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| `byteorder` | 1.5.0 | Unlicense OR MIT | https://github.com/BurntSushi/byteorder |
| `byteorder-lite` | 0.1.0 | Unlicense OR MIT | https://github.com/image-rs/byteorder-lite |
| `bytes` | 1.12.1 | MIT | https://github.com/tokio-rs/bytes |
| `cairo-rs` | 0.18.5 | MIT | https://github.com/gtk-rs/gtk-rs-core |
| `cairo-sys-rs` | 0.18.2 | MIT | https://github.com/gtk-rs/gtk-rs-core |
| `calloop` | 0.13.0 | MIT | https://github.com/Smithay/calloop |
| `calloop` | 0.14.5 | MIT | https://github.com/Smithay/calloop |
| `calloop-wayland-source` | 0.3.0 | MIT | https://github.com/smithay/calloop-wayland-source |
| `calloop-wayland-source` | 0.4.1 | MIT | https://github.com/smithay/calloop-wayland-source |
| `camino` | 1.2.6 | MIT OR Apache-2.0 | https://github.com/camino-rs/camino |
| `cargo-platform` | 0.1.9 | MIT OR Apache-2.0 | https://github.com/rust-lang/cargo |
| `cargo_metadata` | 0.19.2 | MIT | https://github.com/oli-obk/cargo_metadata |
| `cargo_toml` | 1.0.1 | Apache-2.0 OR MIT | https://gitlab.com/lib.rs/cargo_toml |
| `cc` | 1.11.2 | MIT OR Apache-2.0 | https://github.com/rust-lang/cc-rs |
| `cfb` | 0.14.0 | MIT | https://github.com/mdsteele/rust-cfb |
| `cfg-expr` | 0.15.8 | MIT OR Apache-2.0 | https://github.com/EmbarkStudios/cfg-expr |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/cfg-if |
| `cfg_aliases` | 0.2.2 | MIT | https://github.com/katharostech/cfg_aliases |
| `chrono` | 0.4.45 | MIT OR Apache-2.0 | https://github.com/chronotope/chrono |
| `clipboard-win` | 5.4.3 | BSL-1.0 | https://github.com/DoumanAsh/clipboard-win |
| `clipboard_macos` | 0.1.1 | Apache-2.0 | https://github.com/hecrj/window_clipboard |
| `clipboard_wayland` | 0.2.2 | Apache-2.0 | https://github.com/hecrj/window_clipboard |
| `clipboard_x11` | 0.4.3 | MIT | https://github.com/hecrj/window_clipboard |
| `codespan-reporting` | 0.12.0 | Apache-2.0 | https://github.com/brendanzab/codespan |
| `color_quant` | 1.1.0 | MIT | https://github.com/image-rs/color_quant.git |
| `concurrent-queue` | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/concurrent-queue |
| `const-oid` | 0.10.3 | Apache-2.0 OR MIT | https://github.com/RustCrypto/formats |
| `cookie` | 0.18.3 | MIT OR Apache-2.0 | https://github.com/SergioBenitez/cookie-rs |
| `core-foundation` | 0.10.1 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| `core-foundation` | 0.9.4 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| `core-foundation-sys` | 0.8.7 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| `core-graphics` | 0.23.2 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| `core-graphics` | 0.25.0 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| `core-graphics-types` | 0.1.3 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| `core-graphics-types` | 0.2.0 | MIT OR Apache-2.0 | https://github.com/servo/core-foundation-rs |
| `core_maths` | 0.1.1 | MIT | https://github.com/robertbastian/core_maths |
| `coreaudio-rs` | 0.14.2 | MIT/Apache-2.0 | https://github.com/RustAudio/coreaudio-rs.git |
| `cosmic-text` | 0.15.0 | MIT OR Apache-2.0 | https://github.com/pop-os/cosmic-text |
| `cpal` | 0.18.2 | Apache-2.0 | https://github.com/RustAudio/cpal |
| `cpufeatures` | 0.3.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| `crc32fast` | 1.5.4 | MIT OR Apache-2.0 | https://github.com/srijs/rust-crc32fast |
| `crossbeam-channel` | 0.5.17 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |
| `crossbeam-deque` | 0.8.8 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |
| `crossbeam-epoch` | 0.9.21 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |
| `crossbeam-utils` | 0.8.23 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |
| `cryoglyph` | 0.1.0 | MIT OR Apache-2.0 OR Zlib | https://github.com/iced-rs/cryoglyph |
| `crypto-common` | 0.2.2 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| `cssparser` | 0.37.0 | MPL-2.0 | https://github.com/servo/rust-cssparser |
| `cssparser-macros` | 0.7.1 | MPL-2.0 | https://github.com/servo/rust-cssparser |
| `ctor` | 0.10.1 | Apache-2.0 OR MIT | https://github.com/mmastrac/rust-ctor |
| `ctor` | 1.0.14 | Apache-2.0 OR MIT | https://github.com/mmastrac/linktime |
| `cursor-icon` | 1.2.0 | MIT OR Apache-2.0 OR Zlib | https://github.com/rust-windowing/cursor-icon |
| `darling` | 0.24.1 | MIT | https://github.com/TedDriggs/darling |
| `darling_core` | 0.24.1 | MIT | https://github.com/TedDriggs/darling |
| `darling_macro` | 0.24.1 | MIT | https://github.com/TedDriggs/darling |
| `dasp_sample` | 0.11.0 | MIT OR Apache-2.0 | https://github.com/rustaudio/sample.git |
| `dbus` | 0.9.12 | Apache-2.0/MIT | https://github.com/diwic/dbus-rs |
| `deranged` | 0.5.9 | MIT OR Apache-2.0 | https://github.com/jhpratt/deranged |
| `derive_more` | 2.1.1 | MIT | https://github.com/JelteF/derive_more |
| `derive_more-impl` | 2.1.1 | MIT | https://github.com/JelteF/derive_more |
| `digest` | 0.11.3 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| `dirs` | 7.0.0 | MIT OR Apache-2.0 | https://codeberg.org/dirs/dirs-rs |
| `dirs-sys` | 0.5.0 | MIT OR Apache-2.0 | https://github.com/dirs-dev/dirs-sys-rs |
| `dispatch` | 0.2.0 | MIT | http://github.com/SSheldon/rust-dispatch |
| `dispatch2` | 0.3.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `displaydoc` | 0.2.7 | MIT OR Apache-2.0 | https://github.com/yaahc/displaydoc |
| `dlib` | 0.5.3 | MIT | https://github.com/elinorbgr/dlib |
| `dlopen2` | 0.8.2 | MIT | https://github.com/OpenByteDev/dlopen2 |
| `dlopen2_derive` | 0.4.3 | MIT | https://github.com/OpenByteDev/dlopen2 |
| `document-features` | 0.2.12 | MIT OR Apache-2.0 | https://github.com/slint-ui/document-features |
| `dom_query` | 0.28.0 | MIT | https://github.com/niklak/dom_query |
| `downcast-rs` | 1.2.1 | MIT/Apache-2.0 | https://github.com/marcianx/downcast-rs |
| `dpi` | 0.1.2 | Apache-2.0 AND MIT | https://github.com/rust-windowing/winit |
| `dtoa` | 1.0.11 | MIT OR Apache-2.0 | https://github.com/dtolnay/dtoa |
| `dtoa-short` | 0.3.5 | MPL-2.0 | https://github.com/upsuper/dtoa-short |
| `dtor` | 0.8.1 | Apache-2.0 OR MIT | https://github.com/mmastrac/rust-ctor |
| `dunce` | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 | https://gitlab.com/kornelski/dunce |
| `dyn-clone` | 1.0.20 | MIT OR Apache-2.0 | https://github.com/dtolnay/dyn-clone |
| `either` | 1.19.0 | MIT OR Apache-2.0 | https://github.com/rayon-rs/either |
| `embed-resource` | 3.0.12 | MIT | https://github.com/nabijaczleweli/rust-embed-resource |
| `embed_plist` | 1.2.2 | MIT OR Apache-2.0 | https://github.com/nvzqz/embed-plist-rs |
| `endi` | 1.1.1 | MIT | https://github.com/zeenix/endi |
| `enumflags2` | 0.7.12 | MIT OR Apache-2.0 | https://github.com/meithecatte/enumflags2 |
| `enumflags2_derive` | 0.7.12 | MIT OR Apache-2.0 | https://github.com/meithecatte/enumflags2 |
| `equator` | 0.4.2 | MIT | https://github.com/sarah-ek/equator/ |
| `equator-macro` | 0.4.2 | MIT | https://github.com/sarah-ek/equator/ |
| `equivalent` | 1.0.2 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/equivalent |
| `erased-serde` | 0.4.11 | MIT OR Apache-2.0 | https://github.com/dtolnay/erased-serde |
| `errno` | 0.3.15 | MIT OR Apache-2.0 | https://github.com/lambda-fairy/rust-errno |
| `error-code` | 3.4.0 | BSL-1.0 | https://github.com/DoumanAsh/error-code |
| `etagere` | 0.2.15 | MIT/Apache-2.0 | https://github.com/nical/etagere |
| `euclid` | 0.22.14 | MIT OR Apache-2.0 | https://github.com/servo/euclid |
| `event-listener` | 5.4.2 | Apache-2.0 OR MIT | https://github.com/smol-rs/event-listener |
| `event-listener-strategy` | 0.5.4 | Apache-2.0 OR MIT | https://github.com/smol-rs/event-listener-strategy |
| `exr` | 1.74.3 | BSD-3-Clause | https://github.com/johannesvollmer/exrs |
| `fastrand` | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/fastrand |
| `fax` | 0.2.7 | MIT | https://github.com/pdf-rs/fax |
| `fdeflate` | 0.3.7 | MIT OR Apache-2.0 | https://github.com/image-rs/fdeflate |
| `field-offset` | 0.3.6 | MIT OR Apache-2.0 | https://github.com/Diggsey/rust-field-offset |
| `find-msvc-tools` | 0.1.15 | MIT OR Apache-2.0 | https://github.com/rust-lang/cc-rs |
| `flate2` | 1.1.10 | MIT OR Apache-2.0 | https://github.com/rust-lang/flate2-rs |
| `float_next_after` | 1.0.0 | MIT | https://gitlab.com/bronsonbdevost/next_afterf |
| `fluent-bundle` | 0.16.0 | Apache-2.0 OR MIT | https://github.com/projectfluent/fluent-rs |
| `fluent-langneg` | 0.13.1 | Apache-2.0 OR MIT | https://github.com/projectfluent/fluent-langneg-rs |
| `fluent-syntax` | 0.12.0 | Apache-2.0 OR MIT | https://github.com/projectfluent/fluent-rs |
| `fnv` | 1.0.7 | Apache-2.0 / MIT | https://github.com/servo/rust-fnv |
| `foldhash` | 0.1.5 | Zlib | https://github.com/orlp/foldhash |
| `foldhash` | 0.2.0 | Zlib | https://github.com/orlp/foldhash |
| `font-types` | 0.10.1 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| `font-types` | 0.12.7 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| `fontconfig-parser` | 0.5.8 | MIT | https://github.com/Riey/fontconfig-parser |
| `fontdb` | 0.23.0 | MIT | https://github.com/RazrFalcon/fontdb |
| `foreign-types` | 0.5.0 | MIT/Apache-2.0 | https://github.com/sfackler/foreign-types |
| `foreign-types-macros` | 0.2.4 | MIT/Apache-2.0 | https://github.com/sfackler/foreign-types |
| `foreign-types-shared` | 0.3.1 | MIT/Apache-2.0 | https://github.com/sfackler/foreign-types |
| `form_urlencoded` | 1.2.3 | MIT OR Apache-2.0 | https://github.com/servo/rust-url |
| `futures` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `futures-channel` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `futures-core` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `futures-executor` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `futures-io` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `futures-lite` | 2.6.1 | Apache-2.0 OR MIT | https://github.com/smol-rs/futures-lite |
| `futures-macro` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `futures-sink` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `futures-task` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `futures-util` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| `gdk` | 0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs |
| `gdk-pixbuf` | 0.18.5 | MIT | https://github.com/gtk-rs/gtk-rs-core |
| `gdk-pixbuf-sys` | 0.18.0 | MIT | https://github.com/gtk-rs/gtk-rs-core |
| `gdk-sys` | 0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs |
| `gdkwayland-sys` | 0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs |
| `gdkx11` | 0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs |
| `gdkx11-sys` | 0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs |
| `gethostname` | 1.1.0 | Apache-2.0 | https://codeberg.org/swsnr/gethostname.rs.git |
| `getrandom` | 0.3.4 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom |
| `getrandom` | 0.4.3 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom |
| `gif` | 0.14.2 | MIT OR Apache-2.0 | https://github.com/image-rs/image-gif |
| `gio` | 0.18.4 | MIT | https://github.com/gtk-rs/gtk-rs-core |
| `gio-sys` | 0.18.1 | MIT | https://github.com/gtk-rs/gtk-rs-core |
| `gl_generator` | 0.14.0 | Apache-2.0 | https://github.com/brendanzab/gl-rs/ |
| `glam` | 0.25.0 | MIT OR Apache-2.0 | https://github.com/bitshifter/glam-rs |
| `glib` | 0.18.5 | MIT | https://github.com/gtk-rs/gtk-rs-core |
| `glib-macros` | 0.18.5 | MIT | https://github.com/gtk-rs/gtk-rs-core |
| `glib-sys` | 0.18.1 | MIT | https://github.com/gtk-rs/gtk-rs-core |
| `glob` | 0.3.4 | MIT OR Apache-2.0 | https://github.com/rust-lang/glob |
| `glow` | 0.16.0 | MIT OR Apache-2.0 OR Zlib | https://github.com/grovesNL/glow |
| `glutin_wgl_sys` | 0.6.1 | Apache-2.0 | https://github.com/rust-windowing/glutin |
| `gobject-sys` | 0.18.0 | MIT | https://github.com/gtk-rs/gtk-rs-core |
| `gpu-alloc` | 0.6.2 | MIT OR Apache-2.0 | https://github.com/zakarumych/gpu-alloc |
| `gpu-alloc-types` | 0.3.1 | MIT OR Apache-2.0 | https://github.com/zakarumych/gpu-alloc |
| `gpu-allocator` | 0.27.0 | MIT OR Apache-2.0 | https://github.com/Traverse-Research/gpu-allocator |
| `gpu-descriptor` | 0.3.2 | MIT OR Apache-2.0 | https://github.com/zakarumych/gpu-descriptor |
| `gpu-descriptor-types` | 0.2.0 | MIT OR Apache-2.0 | https://github.com/zakarumych/gpu-descriptor |
| `gtk` | 0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs |
| `gtk-sys` | 0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs |
| `gtk3-macros` | 0.18.2 | MIT | https://github.com/gtk-rs/gtk3-rs |
| `guillotiere` | 0.6.2 | MIT/Apache-2.0 | https://github.com/nical/guillotiere |
| `half` | 2.7.1 | MIT OR Apache-2.0 | https://github.com/VoidStarKat/half-rs |
| `harfrust` | 0.3.2 | MIT | https://github.com/harfbuzz/harfrust |
| `hashbrown` | 0.12.3 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| `hashbrown` | 0.15.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| `hashbrown` | 0.16.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| `hashbrown` | 0.17.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| `heck` | 0.4.1 | MIT OR Apache-2.0 | https://github.com/withoutboats/heck |
| `heck` | 0.5.0 | MIT OR Apache-2.0 | https://github.com/withoutboats/heck |
| `hex` | 0.4.3 | MIT OR Apache-2.0 | https://github.com/KokaKiwi/rust-hex |
| `hexf-parse` | 0.2.1 | CC0-1.0 | https://github.com/lifthrasiir/hexf |
| `html5ever` | 0.39.0 | MIT OR Apache-2.0 | https://github.com/servo/html5ever |
| `http` | 1.5.0 | MIT OR Apache-2.0 | https://github.com/hyperium/http |
| `hybrid-array` | 0.4.15 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hybrid-array |
| `iana-time-zone` | 0.1.65 | MIT OR Apache-2.0 | https://github.com/strawlab/iana-time-zone |
| `iced` | 0.14.0 | MIT | https://github.com/iced-rs/iced |
| `iced_core` | 0.14.0 | MIT | https://github.com/iced-rs/iced |
| `iced_debug` | 0.14.0 | MIT | https://github.com/iced-rs/iced |
| `iced_futures` | 0.14.0 | MIT | https://github.com/iced-rs/iced |
| `iced_graphics` | 0.14.0 | MIT | https://github.com/iced-rs/iced |
| `iced_program` | 0.14.0 | MIT | https://github.com/iced-rs/iced |
| `iced_renderer` | 0.14.0 | MIT | https://github.com/iced-rs/iced |
| `iced_runtime` | 0.14.0 | MIT | https://github.com/iced-rs/iced |
| `iced_tiny_skia` | 0.14.1 | MIT | https://github.com/iced-rs/iced |
| `iced_wgpu` | 0.14.0 | MIT | https://github.com/iced-rs/iced |
| `iced_widget` | 0.14.2 | MIT | https://github.com/iced-rs/iced |
| `iced_winit` | 0.14.1 | MIT | https://github.com/iced-rs/iced |
| `ico` | 0.6.0 | MIT | https://github.com/mdsteele/rust-ico |
| `icu_collections` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `icu_locale_core` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `icu_normalizer` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `icu_normalizer_data` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `icu_properties` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `icu_properties_data` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `icu_provider` | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `ident_case` | 1.0.1 | MIT/Apache-2.0 | https://github.com/TedDriggs/ident_case |
| `idna` | 1.1.0 | MIT OR Apache-2.0 | https://github.com/servo/rust-url/ |
| `idna_adapter` | 1.2.2 | Apache-2.0 OR MIT | https://github.com/hsivonen/idna_adapter |
| `image` | 0.25.10 | MIT OR Apache-2.0 | https://github.com/image-rs/image |
| `image-webp` | 0.2.4 | MIT OR Apache-2.0 | https://github.com/image-rs/image-webp |
| `imgref` | 1.12.3 | CC0-1.0 OR Apache-2.0 | https://github.com/kornelski/imgref |
| `indexmap` | 1.9.3 | Apache-2.0 OR MIT | https://github.com/bluss/indexmap |
| `indexmap` | 2.14.2 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/indexmap |
| `infer` | 0.22.0 | MIT | https://github.com/bojand/infer |
| `intl-memoizer` | 0.5.3 | Apache-2.0 OR MIT | https://github.com/projectfluent/fluent-rs |
| `intl_pluralrules` | 7.0.2 | Apache-2.0/MIT | https://github.com/zbraniecki/pluralrules |
| `itertools` | 0.14.0 | MIT OR Apache-2.0 | https://github.com/rust-itertools/itertools |
| `itoa` | 1.0.18 | MIT OR Apache-2.0 | https://github.com/dtolnay/itoa |
| `javascriptcore-rs` | 1.1.2 | MIT | https://github.com/tauri-apps/javascriptcore-rs |
| `javascriptcore-rs-sys` | 1.1.1 | MIT | https://github.com/tauri-apps/javascriptcore-rs |
| `jiff` | 0.2.46 | Unlicense OR MIT | https://github.com/BurntSushi/jiff |
| `jiff` | 1.0.1 | Unlicense OR MIT | https://github.com/BurntSushi/jiff |
| `jiff-core` | 0.1.8 | Unlicense OR MIT | https://github.com/BurntSushi/jiff |
| `jiff-core` | 1.0.1 | Unlicense OR MIT | https://github.com/BurntSushi/jiff |
| `jiff-tzdb` | 0.1.10 | Unlicense OR MIT | https://github.com/BurntSushi/jiff |
| `jiff-tzdb` | 1.0.0 | Unlicense OR MIT | https://github.com/BurntSushi/jiff |
| `jiff-tzdb-platform` | 0.1.3 | Unlicense OR MIT | https://github.com/BurntSushi/jiff |
| `jiff-tzdb-platform` | 1.0.0 | Unlicense OR MIT | https://github.com/BurntSushi/jiff |
| `jobserver` | 0.1.35 | MIT OR Apache-2.0 | https://github.com/rust-lang/jobserver-rs |
| `json-patch` | 4.2.0 | MIT/Apache-2.0 | https://github.com/idubrov/json-patch |
| `jsonptr` | 0.7.1 | MIT OR Apache-2.0 | https://github.com/chanced/jsonptr |
| `kamadak-exif` | 0.6.2 | BSD-2-Clause | https://github.com/kamadak/exif-rs |
| `keyboard-types` | 0.7.0 | MIT OR Apache-2.0 | https://github.com/pyfisch/keyboard-types |
| `keyboard-types` | 0.8.3 | MIT OR Apache-2.0 | https://github.com/rust-windowing/keyboard-types |
| `khronos-egl` | 6.0.0 | MIT/Apache-2.0 | https://github.com/timothee-haudebourg/khronos-egl |
| `khronos_api` | 3.1.0 | Apache-2.0 | https://github.com/brendanzab/gl-rs/ |
| `kurbo` | 0.10.4 | MIT OR Apache-2.0 | https://github.com/linebender/kurbo |
| `lebe` | 0.5.3 | BSD-3-Clause | https://github.com/johannesvollmer/lebe |
| `libappindicator` | 0.9.0 | Apache-2.0 OR MIT | https://crates.io/crates/libappindicator |
| `libappindicator-sys` | 0.9.0 | Apache-2.0 OR MIT | https://crates.io/crates/libappindicator-sys |
| `libc` | 0.2.191 | MIT OR Apache-2.0 | https://github.com/rust-lang/libc |
| `libdbus-sys` | 0.2.7 | Apache-2.0/MIT | https://github.com/diwic/dbus-rs |
| `libloading` | 0.7.4 | ISC | https://github.com/nagisa/rust_libloading/ |
| `libloading` | 0.8.9 | ISC | https://github.com/nagisa/rust_libloading/ |
| `libm` | 0.2.16 | MIT | https://github.com/rust-lang/compiler-builtins |
| `libxdo` | 0.6.0 | MIT | https://github.com/crumblingstatue/rust-libxdo |
| `libxdo-sys` | 0.11.0 | MIT | https://github.com/crumblingstatue/rust-libxdo-sys |
| `lilt` | 0.8.2 | MIT | https://github.com/cyypherus/lilt |
| `linebender_resource_handle` | 0.1.1 | Apache-2.0 OR MIT | https://github.com/linebender/raw_resource_handle |
| `linux-raw-sys` | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/sunfishcode/linux-raw-sys |
| `linux-raw-sys` | 0.4.15 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/sunfishcode/linux-raw-sys |
| `litemap` | 0.8.3 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `litrs` | 1.0.1 | MIT OR Apache-2.0 | https://github.com/LukasKalbertodt/litrs |
| `lock_api` | 0.4.14 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| `log` | 0.4.35 | MIT OR Apache-2.0 | https://github.com/rust-lang/log |
| `loop9` | 0.1.5 | MIT | https://gitlab.com/kornelski/loop9.git |
| `lru` | 0.16.4 | MIT | https://github.com/jeromefroe/lru-rs.git |
| `lyon` | 1.0.19 | MIT OR Apache-2.0 | https://github.com/nical/lyon |
| `lyon_algorithms` | 1.0.22 | MIT OR Apache-2.0 | https://github.com/nical/lyon |
| `lyon_geom` | 1.0.20 | MIT OR Apache-2.0 | https://github.com/nical/lyon |
| `lyon_path` | 1.0.19 | MIT OR Apache-2.0 | https://github.com/nical/lyon |
| `lyon_tessellation` | 1.0.23 | MIT OR Apache-2.0 | https://github.com/nical/lyon |
| `mach2` | 0.6.0 | BSD-2-Clause OR MIT OR Apache-2.0 | https://github.com/JohnTitor/mach2 |
| `malloc_buf` | 0.0.6 | MIT | https://github.com/SSheldon/malloc_buf |
| `markup5ever` | 0.39.0 | MIT OR Apache-2.0 | https://github.com/servo/html5ever |
| `maybe-rayon` | 0.1.1 | MIT | https://github.com/shssoichiro/maybe-rayon |
| `memchr` | 2.8.3 | Unlicense OR MIT | https://github.com/BurntSushi/memchr |
| `memmap2` | 0.9.12 | MIT OR Apache-2.0 | https://github.com/RazrFalcon/memmap2-rs |
| `memoffset` | 0.9.1 | MIT | https://github.com/Gilnaa/memoffset |
| `metal` | 0.32.0 | MIT OR Apache-2.0 | https://github.com/gfx-rs/metal-rs |
| `mime` | 0.3.18 | MIT OR Apache-2.0 | https://github.com/hyperium/mime |
| `miniz_oxide` | 0.8.9 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| `miniz_oxide` | 0.9.1 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| `mio` | 1.2.5 | MIT | https://github.com/tokio-rs/mio |
| `moxcms` | 0.8.1 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/moxcms.git |
| `muda` | 0.17.2 | Apache-2.0 OR MIT | https://github.com/tauri-apps/muda |
| `muda` | 0.20.0 | Apache-2.0 OR MIT | https://github.com/tauri-apps/muda |
| `multiversion` | 0.8.1 | MIT OR Apache-2.0 | https://github.com/calebzulawski/multiversion |
| `multiversion-macros` | 0.8.1 | MIT OR Apache-2.0 | https://github.com/calebzulawski/multiversion |
| `mundy` | 0.2.3 | Apache-2.0 | https://github.com/tautropfli/mundy |
| `mutate_once` | 0.1.2 | BSD-2-Clause | https://github.com/kamadak/mutate_once-rs |
| `naga` | 27.0.3 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| `new_debug_unreachable` | 1.0.6 | MIT | https://github.com/mbrubeck/rust-debug-unreachable |
| `no_std_io2` | 0.9.4 | Apache-2.0 OR MIT | https://github.com/wcampbell0x2a/no-std-io2 |
| `nom` | 8.0.0 | MIT | https://github.com/rust-bakery/nom |
| `noop_proc_macro` | 0.3.0 | MIT | https://github.com/lu-zero/noop_proc_macro |
| `num-bigint` | 0.4.8 | MIT OR Apache-2.0 | https://github.com/rust-num/num-bigint |
| `num-conv` | 0.2.2 | MIT OR Apache-2.0 | https://github.com/jhpratt/num-conv |
| `num-derive` | 0.4.2 | MIT OR Apache-2.0 | https://github.com/rust-num/num-derive |
| `num-integer` | 0.1.47 | MIT OR Apache-2.0 | https://github.com/rust-num/num-integer |
| `num-rational` | 0.4.2 | MIT OR Apache-2.0 | https://github.com/rust-num/num-rational |
| `num-traits` | 0.2.19 | MIT OR Apache-2.0 | https://github.com/rust-num/num-traits |
| `objc` | 0.2.7 | MIT | http://github.com/SSheldon/rust-objc |
| `objc-sys` | 0.3.5 | MIT | https://github.com/madsmtm/objc2 |
| `objc2` | 0.5.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2` | 0.6.5 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-app-kit` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-app-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-audio-toolbox` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-cloud-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-audio` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-audio-types` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-data` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-data` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-foundation` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-graphics` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-image` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-image` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-text` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-core-video` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-encode` | 4.1.0 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-exception-helper` | 0.1.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-foundation` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-foundation` | 0.3.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-io-surface` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-metal` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-quartz-core` | 0.2.2 | MIT | https://github.com/madsmtm/objc2 |
| `objc2-quartz-core` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `objc2-web-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/madsmtm/objc2 |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 | https://github.com/matklad/once_cell |
| `option-ext` | 0.2.0 | MPL-2.0 | https://github.com/soc/option-ext.git |
| `ordered-float` | 5.5.1 | MIT | https://github.com/reem/rust-ordered-float |
| `ordered-stream` | 0.2.0 | MIT OR Apache-2.0 | https://github.com/danieldg/ordered-stream |
| `owned_ttf_parser` | 0.25.1 | Apache-2.0 | https://github.com/alexheretic/owned-ttf-parser |
| `pango` | 0.18.3 | MIT | https://github.com/gtk-rs/gtk-rs-core |
| `pango-sys` | 0.18.0 | MIT | https://github.com/gtk-rs/gtk-rs-core |
| `parking` | 2.2.1 | Apache-2.0 OR MIT | https://github.com/smol-rs/parking |
| `parking_lot` | 0.12.5 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| `parking_lot_core` | 0.9.12 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| `paste` | 1.0.15 | MIT OR Apache-2.0 | https://github.com/dtolnay/paste |
| `pastey` | 0.1.1 | MIT OR Apache-2.0 | https://github.com/as1100k/pastey |
| `percent-encoding` | 2.3.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-url/ |
| `phf` | 0.13.1 | MIT | https://github.com/rust-phf/rust-phf |
| `phf_codegen` | 0.13.1 | MIT | https://github.com/rust-phf/rust-phf |
| `phf_generator` | 0.13.1 | MIT | https://github.com/rust-phf/rust-phf |
| `phf_macros` | 0.13.1 | MIT | https://github.com/rust-phf/rust-phf |
| `phf_shared` | 0.13.1 | MIT | https://github.com/rust-phf/rust-phf |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT | https://github.com/taiki-e/pin-project-lite |
| `piper` | 0.2.5 | MIT OR Apache-2.0 | https://github.com/smol-rs/piper |
| `pkg-config` | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/pkg-config-rs |
| `plist` | 1.10.4 | MIT | https://github.com/ebarnard/rust-plist/ |
| `png` | 0.17.16 | MIT OR Apache-2.0 | https://github.com/image-rs/image-png |
| `png` | 0.18.1 | MIT OR Apache-2.0 | https://github.com/image-rs/image-png |
| `polling` | 3.11.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/polling |
| `pollster` | 0.4.0 | Apache-2.0/MIT | https://github.com/zesterer/pollster |
| `potential_utf` | 0.1.6 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `powerfmt` | 0.2.1 | MIT OR Apache-2.0 | https://github.com/jhpratt/powerfmt |
| `precomputed-hash` | 0.1.1 | MIT | https://github.com/emilio/precomputed-hash |
| `presser` | 0.3.1 | MIT OR Apache-2.0 | https://github.com/EmbarkStudios/presser |
| `proc-macro-crate` | 1.3.1 | MIT OR Apache-2.0 | https://github.com/bkchr/proc-macro-crate |
| `proc-macro-crate` | 2.0.2 | MIT OR Apache-2.0 | https://github.com/bkchr/proc-macro-crate |
| `proc-macro-crate` | 3.5.0 | MIT OR Apache-2.0 | https://github.com/bkchr/proc-macro-crate |
| `proc-macro-error` | 1.0.4 | MIT OR Apache-2.0 | https://gitlab.com/CreepySkeleton/proc-macro-error |
| `proc-macro-error-attr` | 1.0.4 | MIT OR Apache-2.0 | https://gitlab.com/CreepySkeleton/proc-macro-error |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 | https://github.com/dtolnay/proc-macro2 |
| `profiling` | 1.0.18 | MIT OR Apache-2.0 | https://github.com/aclysma/profiling |
| `profiling-procmacros` | 1.0.18 | MIT OR Apache-2.0 | https://github.com/aclysma/profiling |
| `pulldown-cmark` | 0.13.6 | MIT | https://github.com/raphlinus/pulldown-cmark |
| `pxfm` | 0.1.30 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/pxfm |
| `qoi` | 0.4.1 | MIT/Apache-2.0 | https://github.com/aldanor/qoi-rust |
| `quick-error` | 2.0.1 | MIT/Apache-2.0 | http://github.com/tailhook/quick-error |
| `quick-xml` | 0.41.0 | MIT | https://github.com/tafia/quick-xml |
| `quick-xml` | 0.42.0 | MIT | https://github.com/tafia/quick-xml |
| `quote` | 1.0.47 | MIT OR Apache-2.0 | https://github.com/dtolnay/quote |
| `range-alloc` | 0.1.5 | MIT OR Apache-2.0 | https://github.com/gfx-rs/range-alloc |
| `rangemap` | 1.8.0 | MIT/Apache-2.0 | https://github.com/jeffparsons/rangemap |
| `rav1e` | 0.8.1 | BSD-2-Clause | https://github.com/xiph/rav1e/ |
| `ravif` | 0.13.0 | BSD-3-Clause | https://github.com/kornelski/cavif-rs |
| `raw-window-handle` | 0.6.2 | MIT OR Apache-2.0 OR Zlib | https://github.com/rust-windowing/raw-window-handle |
| `rayon` | 1.12.0 | MIT OR Apache-2.0 | https://github.com/rayon-rs/rayon |
| `rayon-core` | 1.13.0 | MIT OR Apache-2.0 | https://github.com/rayon-rs/rayon |
| `read-fonts` | 0.35.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| `read-fonts` | 0.41.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| `ref-cast` | 1.0.27 | MIT OR Apache-2.0 | https://github.com/dtolnay/ref-cast |
| `ref-cast-impl` | 1.0.27 | MIT OR Apache-2.0 | https://github.com/dtolnay/ref-cast |
| `regex` | 1.13.2 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| `regex-automata` | 0.4.19 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| `regex-syntax` | 0.8.12 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| `renderdoc-sys` | 1.1.0 | MIT OR Apache-2.0 | https://github.com/ebkalderon/renderdoc-rs |
| `rfd` | 0.17.2 | MIT | https://github.com/PolyMeilex/rfd |
| `rgb` | 0.8.53 | MIT | https://github.com/kornelski/rust-rgb |
| `roxmltree` | 0.20.0 | MIT OR Apache-2.0 | https://github.com/RazrFalcon/roxmltree |
| `rustc-hash` | 1.1.0 | Apache-2.0/MIT | https://github.com/rust-lang-nursery/rustc-hash |
| `rustc-hash` | 2.1.3 | Apache-2.0 OR MIT | https://github.com/rust-lang/rustc-hash |
| `rustc_version` | 0.4.1 | MIT OR Apache-2.0 | https://github.com/djc/rustc-version-rs |
| `rustix` | 0.38.44 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/bytecodealliance/rustix |
| `rustix` | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/bytecodealliance/rustix |
| `rustversion` | 1.0.23 | MIT OR Apache-2.0 | https://github.com/dtolnay/rustversion |
| `same-file` | 1.0.6 | Unlicense/MIT | https://github.com/BurntSushi/same-file |
| `schemars` | 0.8.22 | MIT | https://github.com/GREsau/schemars |
| `schemars` | 0.9.0 | MIT | https://github.com/GREsau/schemars |
| `schemars` | 1.2.3 | MIT | https://github.com/GREsau/schemars |
| `schemars_derive` | 0.8.22 | MIT | https://github.com/GREsau/schemars |
| `scoped-tls` | 1.0.1 | MIT/Apache-2.0 | https://github.com/alexcrichton/scoped-tls |
| `scopeguard` | 1.2.0 | MIT OR Apache-2.0 | https://github.com/bluss/scopeguard |
| `sctk-adwaita` | 0.10.1 | MIT | https://github.com/PolyMeilex/sctk-adwaita |
| `selectors` | 0.38.0 | MPL-2.0 | https://github.com/servo/stylo |
| `self_cell` | 1.3.0 | Apache-2.0 OR GPL-2.0-only | https://github.com/Voultapher/self_cell |
| `semver` | 1.0.28 | MIT OR Apache-2.0 | https://github.com/dtolnay/semver |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde-untagged` | 0.1.9 | MIT OR Apache-2.0 | https://github.com/dtolnay/serde-untagged |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_derive` | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_derive_internals` | 0.29.1 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 | https://github.com/serde-rs/json |
| `serde_repr` | 0.1.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/serde-repr |
| `serde_spanned` | 0.6.9 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `serde_spanned` | 1.1.3 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `serde_with` | 3.25.0 | MIT OR Apache-2.0 | https://github.com/jonasbb/serde_with/ |
| `serde_with_macros` | 3.25.0 | MIT OR Apache-2.0 | https://github.com/jonasbb/serde_with/ |
| `serialize-to-javascript` | 0.1.2 | MIT OR Apache-2.0 | https://github.com/chippers/serialize-to-javascript |
| `serialize-to-javascript-impl` | 0.1.2 | MIT OR Apache-2.0 | https://github.com/chippers/serialize-to-javascript |
| `servo_arc` | 0.4.3 | MIT OR Apache-2.0 | https://github.com/servo/stylo |
| `sha2` | 0.11.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes |
| `shlex` | 2.0.1 | MIT OR Apache-2.0 | https://github.com/comex/rust-shlex |
| `signal-hook-registry` | 1.4.8 | MIT OR Apache-2.0 | https://github.com/vorner/signal-hook |
| `simd-adler32` | 0.3.10 | MIT | https://github.com/mcountryman/simd-adler32 |
| `simd_helpers` | 0.1.0 | MIT | https://github.com/lu-zero/simd_helpers |
| `siphasher` | 1.0.4 | MIT OR Apache-2.0 | https://github.com/jedisct1/rust-siphash |
| `skrifa` | 0.37.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| `skrifa` | 0.44.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| `slab` | 0.4.12 | MIT | https://github.com/tokio-rs/slab |
| `slotmap` | 1.1.1 | Zlib | https://github.com/orlp/slotmap |
| `smallvec` | 1.16.5 | MIT OR Apache-2.0 | https://github.com/servo/rust-smallvec |
| `smithay-client-toolkit` | 0.19.2 | MIT | https://github.com/smithay/client-toolkit |
| `smithay-client-toolkit` | 0.20.0 | MIT | https://github.com/smithay/client-toolkit |
| `smithay-clipboard` | 0.7.3 | MIT | https://github.com/smithay/smithay-clipboard |
| `smol_str` | 0.2.2 | MIT OR Apache-2.0 | https://github.com/rust-analyzer/smol_str |
| `socket2` | 0.6.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/socket2 |
| `softbuffer` | 0.4.8 | MIT OR Apache-2.0 | https://github.com/rust-windowing/softbuffer |
| `soup3` | 0.5.0 | MIT | https://gitlab.gnome.org/World/Rust/soup3-rs |
| `soup3-sys` | 0.5.0 | MIT | https://gitlab.gnome.org/World/Rust/soup3-rs |
| `spirv` | 0.3.0+sdk-1.3.268.0 | Apache-2.0 | https://github.com/gfx-rs/rspirv |
| `stable_deref_trait` | 1.2.1 | MIT OR Apache-2.0 | https://github.com/storyyeller/stable_deref_trait |
| `static_assertions` | 1.1.0 | MIT OR Apache-2.0 | https://github.com/nvzqz/static-assertions-rs |
| `static_vcruntime` | 3.0.0 | MIT OR Apache-2.0 OR Zlib | https://github.com/ChrisDenton/static_vcruntime |
| `strict-num` | 0.1.1 | MIT | https://github.com/RazrFalcon/strict-num |
| `string_cache` | 0.9.0 | MIT OR Apache-2.0 | https://github.com/servo/string-cache |
| `string_cache_codegen` | 0.6.1 | MIT OR Apache-2.0 | https://github.com/servo/string-cache |
| `strsim` | 0.11.1 | MIT | https://github.com/rapidfuzz/strsim-rs |
| `svg_fmt` | 0.4.5 | MIT/Apache-2.0 | https://github.com/nical/rust_debug |
| `swash` | 0.2.10 | Apache-2.0 OR MIT | https://github.com/dfrg/swash |
| `swift-rs` | 1.0.9 | MIT OR Apache-2.0 | https://github.com/Brendonovich/swift-rs |
| `syn` | 1.0.109 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| `syn` | 2.0.119 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| `syn` | 3.0.9 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| `synstructure` | 0.14.0 | MIT | https://github.com/mystor/synstructure |
| `sys-locale` | 0.3.2 | MIT OR Apache-2.0 | https://github.com/1Password/sys-locale |
| `system-deps` | 6.2.2 | MIT OR Apache-2.0 | https://github.com/gdesmott/system-deps |
| `tao` | 0.37.3 | Apache-2.0 | https://github.com/tauri-apps/tao |
| `target-features` | 0.1.6 | MIT OR Apache-2.0 | https://github.com/calebzulawski/target-features |
| `target-lexicon` | 0.12.16 | Apache-2.0 WITH LLVM-exception | https://github.com/bytecodealliance/target-lexicon |
| `tauri` | 2.12.7 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri |
| `tauri-build` | 2.7.6 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri |
| `tauri-codegen` | 2.7.6 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri |
| `tauri-macros` | 2.7.6 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri |
| `tauri-plugin` | 2.7.4 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri |
| `tauri-plugin-dialog` | 2.8.4 | Apache-2.0 OR MIT | https://github.com/tauri-apps/plugins-workspace |
| `tauri-plugin-fs` | 2.7.1 | Apache-2.0 OR MIT | https://github.com/tauri-apps/plugins-workspace |
| `tauri-plugin-single-instance` | 2.5.5 | Apache-2.0 OR MIT | https://github.com/tauri-apps/plugins-workspace |
| `tauri-runtime` | 2.12.4 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri |
| `tauri-runtime-wry` | 2.12.5 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri |
| `tauri-utils` | 2.10.4 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri |
| `tauri-winres` | 0.3.7 | MIT | https://github.com/tauri-apps/winres |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | https://github.com/Stebalien/tempfile |
| `tendril` | 0.5.1 | MIT OR Apache-2.0 | https://github.com/servo/html5ever |
| `termcolor` | 1.4.1 | Unlicense OR MIT | https://github.com/BurntSushi/termcolor |
| `thiserror` | 1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| `thiserror-impl` | 1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| `thiserror-impl` | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| `tiff` | 0.11.3 | MIT | https://github.com/image-rs/image-tiff |
| `time` | 0.3.55 | MIT OR Apache-2.0 | https://github.com/time-rs/time |
| `time-core` | 0.1.9 | MIT OR Apache-2.0 | https://github.com/time-rs/time |
| `time-macros` | 0.2.32 | MIT OR Apache-2.0 | https://github.com/time-rs/time |
| `tiny-skia` | 0.11.4 | BSD-3-Clause | https://github.com/RazrFalcon/tiny-skia |
| `tiny-skia-path` | 0.11.4 | BSD-3-Clause | https://github.com/RazrFalcon/tiny-skia/tree/master/path |
| `tiny-xlib` | 0.2.5 | MIT OR Apache-2.0 OR Zlib | https://github.com/rust-windowing/tiny-xlib |
| `tinystr` | 0.8.5 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `tinyvec` | 1.13.3 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/tinyvec |
| `tokio` | 1.54.1 | MIT | https://github.com/tokio-rs/tokio |
| `toml` | 0.8.2 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `toml` | 1.2.1+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `toml_datetime` | 0.6.3 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `toml_datetime` | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `toml_edit` | 0.19.15 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `toml_edit` | 0.20.2 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `toml_edit` | 0.25.19+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `toml_parser` | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `toml_writer` | 1.2.0+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| `tracing` | 0.1.44 | MIT | https://github.com/tokio-rs/tracing |
| `tracing-attributes` | 0.1.31 | MIT | https://github.com/tokio-rs/tracing |
| `tracing-core` | 0.1.36 | MIT | https://github.com/tokio-rs/tracing |
| `tray-icon` | 0.25.1 | MIT OR Apache-2.0 | https://github.com/tauri-apps/tray-icon |
| `ttf-parser` | 0.25.1 | MIT OR Apache-2.0 | https://github.com/harfbuzz/ttf-parser |
| `type-map` | 0.5.1 | MIT/Apache-2.0 | https://github.com/kardeiz/type-map |
| `typeid` | 1.0.3 | MIT OR Apache-2.0 | https://github.com/dtolnay/typeid |
| `typenum` | 1.20.1 | MIT OR Apache-2.0 | https://github.com/paholg/typenum |
| `uds_windows` | 1.2.1 | MIT | https://github.com/haraldh/rust_uds_windows |
| `unic-langid` | 0.9.6 | MIT OR Apache-2.0 | https://github.com/zbraniecki/unic-locale |
| `unic-langid-impl` | 0.9.6 | MIT OR Apache-2.0 | https://github.com/zbraniecki/unic-locale |
| `unicase` | 2.10.0 | MIT OR Apache-2.0 | https://github.com/seanmonstar/unicase |
| `unicode-bidi` | 0.3.18 | MIT OR Apache-2.0 | https://github.com/servo/unicode-bidi |
| `unicode-ident` | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/dtolnay/unicode-ident |
| `unicode-linebreak` | 0.1.5 | Apache-2.0 | https://github.com/axelf4/unicode-linebreak |
| `unicode-script` | 0.5.9 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-script |
| `unicode-segmentation` | 1.13.3 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-segmentation |
| `unicode-width` | 0.2.3 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-width |
| `url` | 2.5.9 | MIT OR Apache-2.0 | https://github.com/servo/rust-url |
| `urlpattern` | 0.6.0 | MIT | https://github.com/denoland/rust-urlpattern |
| `utf8_iter` | 1.1.0 | Apache-2.0 OR MIT | https://github.com/hsivonen/utf8_iter |
| `utf_types` | 1.0.0 | Apache-2.0 OR MIT | https://github.com/hsivonen/utf_types |
| `uuid` | 1.29.2 | Apache-2.0 OR MIT | https://github.com/uuid-rs/uuid |
| `v_frame` | 0.3.9 | BSD-2-Clause | https://github.com/rust-av/v_frame |
| `version-compare` | 0.2.1 | MIT | https://gitlab.com/timvisee/version-compare |
| `version_check` | 0.9.5 | MIT/Apache-2.0 | https://github.com/SergioBenitez/version_check |
| `vswhom` | 0.1.0 | MIT | https://github.com/nabijaczleweli/vswhom.rs |
| `vswhom-sys` | 0.1.3 | MIT | https://github.com/nabijaczleweli/vswhom-sys.rs |
| `walkdir` | 2.5.0 | Unlicense/MIT | https://github.com/BurntSushi/walkdir |
| `wasm-bindgen` | 0.2.130 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen |
| `wasm-bindgen-macro` | 0.2.130 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro |
| `wasm-bindgen-macro-support` | 0.2.130 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/main/crates/macro-support |
| `wasm-bindgen-shared` | 0.2.130 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared |
| `wayland-backend` | 0.3.17 | MIT | https://github.com/smithay/wayland-rs |
| `wayland-client` | 0.31.15 | MIT | https://github.com/smithay/wayland-rs |
| `wayland-csd-frame` | 0.3.0 | MIT | https://github.com/rust-windowing/wayland-csd-frame |
| `wayland-cursor` | 0.31.14 | MIT | https://github.com/smithay/wayland-rs |
| `wayland-protocols` | 0.32.13 | MIT | https://github.com/smithay/wayland-rs |
| `wayland-protocols-experimental` | 20250721.0.1 | MIT | https://github.com/smithay/wayland-rs |
| `wayland-protocols-misc` | 0.3.12 | MIT | https://github.com/smithay/wayland-rs |
| `wayland-protocols-plasma` | 0.3.12 | MIT | https://github.com/smithay/wayland-rs |
| `wayland-protocols-wlr` | 0.3.12 | MIT | https://github.com/smithay/wayland-rs |
| `wayland-scanner` | 0.31.11 | MIT | https://github.com/smithay/wayland-rs |
| `wayland-sys` | 0.31.11 | MIT | https://github.com/smithay/wayland-rs |
| `web-time` | 1.1.0 | MIT OR Apache-2.0 | https://github.com/daxpedda/web-time |
| `web_atoms` | 0.2.6 | MIT OR Apache-2.0 | https://github.com/servo/html5ever |
| `webkit2gtk` | 2.0.2 | MIT | https://github.com/tauri-apps/webkit2gtk-rs |
| `webkit2gtk-sys` | 2.0.2 | MIT | https://github.com/tauri-apps/webkit2gtk-rs |
| `webview2-com` | 0.39.1 | MIT | https://github.com/wravery/webview2-rs |
| `webview2-com-macros` | 0.8.1 | MIT | https://github.com/wravery/webview2-rs |
| `webview2-com-sys` | 0.39.1 | MIT | https://github.com/wravery/webview2-rs |
| `weezl` | 0.1.12 | MIT OR Apache-2.0 | https://github.com/image-rs/weezl |
| `wgpu` | 27.0.1 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| `wgpu-core` | 27.0.3 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| `wgpu-core-deps-apple` | 27.0.0 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| `wgpu-core-deps-windows-linux-android` | 27.0.0 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| `wgpu-hal` | 27.0.4 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| `wgpu-types` | 27.0.1 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| `winapi` | 0.3.9 | MIT/Apache-2.0 | https://github.com/retep998/winapi-rs |
| `winapi-util` | 0.1.11 | Unlicense OR MIT | https://github.com/BurntSushi/winapi-util |
| `window-vibrancy` | 0.8.1 | Apache-2.0 OR MIT | https://github.com/tauri-apps/tauri-plugin-vibrancy |
| `window_clipboard` | 0.5.1 | MIT | https://github.com/hecrj/window_clipboard |
| `windows` | 0.58.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows` | 0.62.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-collections` | 0.3.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-core` | 0.58.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-core` | 0.62.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-future` | 0.3.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-implement` | 0.58.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-implement` | 0.60.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-interface` | 0.58.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-interface` | 0.59.3 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-link` | 0.2.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-numerics` | 0.3.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-result` | 0.2.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-result` | 0.4.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-strings` | 0.1.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-strings` | 0.5.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-sys` | 0.52.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-sys` | 0.59.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-sys` | 0.60.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-sys` | 0.61.2 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-targets` | 0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-targets` | 0.53.5 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-threading` | 0.2.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows-version` | 0.1.7 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows_x86_64_gnu` | 0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows_x86_64_gnu` | 0.53.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows_x86_64_msvc` | 0.52.6 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `windows_x86_64_msvc` | 0.53.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |
| `winit` | 0.30.13 | Apache-2.0 | https://github.com/rust-windowing/winit |
| `winnow` | 0.5.40 | MIT | https://github.com/winnow-rs/winnow |
| `winnow` | 1.0.5 | MIT | https://github.com/winnow-rs/winnow |
| `winreg` | 0.56.0 | MIT | https://github.com/gentoo90/winreg-rs |
| `writeable` | 0.6.4 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `wry` | 0.57.3 | Apache-2.0 OR MIT | https://github.com/tauri-apps/wry |
| `x11` | 2.21.0 | MIT | https://github.com/AltF02/x11-rs.git |
| `x11-dl` | 2.21.0 | MIT | https://github.com/AltF02/x11-rs.git |
| `x11rb` | 0.13.2 | MIT OR Apache-2.0 | https://github.com/psychon/x11rb |
| `x11rb-protocol` | 0.13.2 | MIT OR Apache-2.0 | https://github.com/psychon/x11rb |
| `xcursor` | 0.3.11 | MIT | https://github.com/esposm03/xcursor-rs |
| `xkbcommon-dl` | 0.4.2 | MIT | https://github.com/rust-windowing/xkbcommon-dl |
| `xkeysym` | 0.2.1 | MIT OR Apache-2.0 OR Zlib | https://github.com/notgull/xkeysym |
| `xml-rs` | 0.8.29 | MIT | https://github.com/kornelski/xml-rs |
| `y4m` | 0.8.0 | MIT | https://github.com/image-rs/y4m.git |
| `yazi` | 0.2.2 | Apache-2.0 OR MIT | https://github.com/dfrg/yazi |
| `yoke` | 0.8.4 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `yoke-derive` | 0.8.4 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `zbus` | 5.19.0 | MIT | https://github.com/z-galaxy/zbus/ |
| `zbus_macros` | 5.19.0 | MIT | https://github.com/z-galaxy/zbus/ |
| `zbus_names` | 4.3.5 | MIT | https://github.com/z-galaxy/zbus/ |
| `zcheapstr` | 1.2.0 | MIT | https://github.com/z-galaxy/zcheapstr/ |
| `zeno` | 0.3.3 | Apache-2.0 OR MIT | https://github.com/dfrg/zeno |
| `zerocopy` | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | https://github.com/google/zerocopy |
| `zerocopy-derive` | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | https://github.com/google/zerocopy |
| `zerofrom` | 0.1.8 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `zerofrom-derive` | 0.1.8 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `zerotrie` | 0.2.5 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `zerovec` | 0.11.9 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `zerovec-derive` | 0.11.7 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| `zlib-rs` | 0.6.8 | Zlib | https://github.com/trifectatechfoundation/zlib-rs |
| `zmij` | 1.0.23 | MIT | https://github.com/dtolnay/zmij |
| `zune-core` | 0.5.3 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image |
| `zune-inflate` | 0.2.54 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image/tree/main/zune-inflate |
| `zune-jpeg` | 0.5.15 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg |
| `zvariant` | 5.15.1 | MIT | https://github.com/z-galaxy/zbus/ |
| `zvariant_derive` | 5.15.1 | MIT | https://github.com/z-galaxy/zbus/ |
| `zvariant_utils` | 4.2.0 | MIT | https://github.com/z-galaxy/zbus/ |

<!-- GUI-PAKETE:END -->

<!-- UI-PAKETE:BEGIN -->

### Pakete der Web-Oberfläche (`ui/`, npm)

Erzeugt mit `python3 tools/ui_notices.py` aus `ui/package-lock.json`: **124 Einträge** (alle Plattformen; davon 17 im ausgelieferten Frontend bzw. zur Laufzeit, der Rest nur zur Bauzeit und zum Prüfen). Installation mit `npm ci --ignore-scripts`. Das gebaute Frontend enthält nur den Code von `svelte`, `@fluent/bundle` und `@tauri-apps/api` (und deren Laufzeitabhängigkeiten); MPL-2.0-Pakete (`lightningcss*`) sind Werkzeuge der Bauzeit und gelangen nicht in das Programm. Die Bauwerkzeuge laden bei der Installation keine Skripte aus (`--ignore-scripts`). Nach der Wahl der Festbreitenschrift (Folgefrage F2) kommt deren Lizenz hinzu.

| Paket | Version | Lizenz | Verwendung |
|---|---|---|---|
| `@codemirror/autocomplete` | 6.20.4 | MIT | Laufzeit |
| `@codemirror/commands` | 6.11.1 | MIT | Laufzeit |
| `@codemirror/language` | 6.13.1 | MIT | Laufzeit |
| `@codemirror/lint` | 6.9.8 | MIT | Laufzeit |
| `@codemirror/search` | 6.7.2 | MIT | Laufzeit |
| `@codemirror/state` | 6.7.6 | MIT | Laufzeit |
| `@codemirror/streamparser` | 6.0.0 | MIT | Laufzeit |
| `@codemirror/view` | 6.43.15 | MIT | Laufzeit |
| `@emnapi/core` | 2.0.0-beta.2 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@emnapi/runtime` | 2.0.0-beta.2 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@emnapi/wasi-threads` | 2.2.0 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@fluent/bundle` | 0.19.1 | Apache-2.0 | Laufzeit |
| `@jridgewell/gen-mapping` | 0.3.13 | MIT | Bauzeit/Prüfung |
| `@jridgewell/remapping` | 2.3.5 | MIT | Bauzeit/Prüfung |
| `@jridgewell/resolve-uri` | 3.1.3 | MIT | Bauzeit/Prüfung |
| `@jridgewell/sourcemap-codec` | 1.6.0 | MIT | Bauzeit/Prüfung |
| `@jridgewell/trace-mapping` | 0.3.31 | MIT | Bauzeit/Prüfung |
| `@lezer/common` | 1.5.6 | MIT | Laufzeit |
| `@lezer/highlight` | 1.2.5 | MIT | Laufzeit |
| `@lezer/lr` | 1.4.12 | MIT | Laufzeit |
| `@marijn/find-cluster-break` | 1.0.4 | MIT | Laufzeit |
| `@napi-rs/wasm-runtime` | 1.2.6 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@oxc-project/types` | 0.156.0 | MIT | Bauzeit/Prüfung |
| `@rolldown/binding-android-arm-eabi` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-android-arm64` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-darwin-arm64` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-darwin-x64` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-freebsd-x64` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-linux-arm-gnueabihf` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-linux-arm64-gnu` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-linux-arm64-musl` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-linux-ppc64-gnu` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-linux-s390x-gnu` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-linux-x64-gnu` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-linux-x64-musl` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-openharmony-arm64` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-wasm32-wasi` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-win32-arm64-msvc` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/binding-win32-x64-msvc` | 1.2.19 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@rolldown/pluginutils` | 1.1.1 | MIT | Bauzeit/Prüfung |
| `@sveltejs/acorn-typescript` | 1.0.13 | MIT | Bauzeit/Prüfung |
| `@sveltejs/load-config` | 0.2.3 | MIT | Bauzeit/Prüfung |
| `@sveltejs/vite-plugin-svelte` | 7.3.1 | MIT | Bauzeit/Prüfung |
| `@tauri-apps/api` | 2.12.4 | Apache-2.0 OR MIT | Laufzeit |
| `@tauri-apps/cli` | 2.12.4 | Apache-2.0 OR MIT | Bauzeit/Prüfung |
| `@tauri-apps/cli-darwin-arm64` | 2.12.4 | Apache-2.0 OR MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@tauri-apps/cli-darwin-x64` | 2.12.4 | Apache-2.0 OR MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@tauri-apps/cli-linux-arm-gnueabihf` | 2.12.4 | Apache-2.0 OR MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@tauri-apps/cli-linux-arm64-gnu` | 2.12.4 | Apache-2.0 OR MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@tauri-apps/cli-linux-arm64-musl` | 2.12.4 | Apache-2.0 OR MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@tauri-apps/cli-linux-riscv64-gnu` | 2.12.4 | Apache-2.0 OR MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@tauri-apps/cli-linux-x64-gnu` | 2.12.4 | Apache-2.0 OR MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@tauri-apps/cli-linux-x64-musl` | 2.12.4 | Apache-2.0 OR MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@tauri-apps/cli-win32-arm64-msvc` | 2.12.4 | Apache-2.0 OR MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@tauri-apps/cli-win32-ia32-msvc` | 2.12.4 | Apache-2.0 OR MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@tauri-apps/cli-win32-x64-msvc` | 2.12.4 | Apache-2.0 OR MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@tybys/wasm-util` | 0.10.5 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `@types/chai` | 5.2.3 | MIT | Bauzeit/Prüfung |
| `@types/deep-eql` | 4.0.2 | MIT | Bauzeit/Prüfung |
| `@types/estree` | 1.0.10 | MIT | Bauzeit/Prüfung |
| `@vitest/mocker` | 5.1.0 | MIT | Bauzeit/Prüfung |
| `@vitest/spy` | 5.1.0 | MIT | Bauzeit/Prüfung |
| `acorn` | 8.19.0 | MIT | Bauzeit/Prüfung |
| `aria-query` | 5.3.1 | Apache-2.0 | Bauzeit/Prüfung |
| `assertion-error` | 2.0.1 | MIT | Bauzeit/Prüfung |
| `axobject-query` | 4.1.0 | Apache-2.0 | Bauzeit/Prüfung |
| `chai` | 6.3.0 | MIT | Bauzeit/Prüfung |
| `chokidar` | 4.0.3 | MIT | Bauzeit/Prüfung |
| `clsx` | 2.1.1 | MIT | Bauzeit/Prüfung |
| `crelt` | 1.0.7 | MIT | Laufzeit |
| `deepmerge` | 4.3.1 | MIT | Bauzeit/Prüfung |
| `detect-libc` | 2.1.2 | Apache-2.0 | Bauzeit/Prüfung |
| `devalue` | 5.9.4 | MIT | Bauzeit/Prüfung |
| `es-module-lexer` | 2.3.2 | MIT | Bauzeit/Prüfung |
| `esm-env` | 1.2.2 | MIT | Bauzeit/Prüfung |
| `esrap` | 2.4.2 | MIT | Bauzeit/Prüfung |
| `estree-walker` | 3.0.3 | MIT | Bauzeit/Prüfung |
| `expect-type` | 1.4.0 | Apache-2.0 | Bauzeit/Prüfung |
| `fdir` | 6.5.0 | MIT | Bauzeit/Prüfung |
| `fsevents` | 2.3.3 | MIT | Bauzeit/Prüfung (Plattformpaket) |
| `is-reference` | 3.0.3 | MIT | Bauzeit/Prüfung |
| `lightningcss` | 1.33.0 | MPL-2.0 | Bauzeit/Prüfung |
| `lightningcss-android-arm64` | 1.33.0 | MPL-2.0 | Bauzeit/Prüfung (Plattformpaket) |
| `lightningcss-darwin-arm64` | 1.33.0 | MPL-2.0 | Bauzeit/Prüfung (Plattformpaket) |
| `lightningcss-darwin-x64` | 1.33.0 | MPL-2.0 | Bauzeit/Prüfung (Plattformpaket) |
| `lightningcss-freebsd-x64` | 1.33.0 | MPL-2.0 | Bauzeit/Prüfung (Plattformpaket) |
| `lightningcss-linux-arm-gnueabihf` | 1.33.0 | MPL-2.0 | Bauzeit/Prüfung (Plattformpaket) |
| `lightningcss-linux-arm64-gnu` | 1.33.0 | MPL-2.0 | Bauzeit/Prüfung (Plattformpaket) |
| `lightningcss-linux-arm64-musl` | 1.33.0 | MPL-2.0 | Bauzeit/Prüfung (Plattformpaket) |
| `lightningcss-linux-x64-gnu` | 1.33.0 | MPL-2.0 | Bauzeit/Prüfung (Plattformpaket) |
| `lightningcss-linux-x64-musl` | 1.33.0 | MPL-2.0 | Bauzeit/Prüfung (Plattformpaket) |
| `lightningcss-win32-arm64-msvc` | 1.33.0 | MPL-2.0 | Bauzeit/Prüfung (Plattformpaket) |
| `lightningcss-win32-x64-msvc` | 1.33.0 | MPL-2.0 | Bauzeit/Prüfung (Plattformpaket) |
| `locate-character` | 3.0.0 | MIT | Bauzeit/Prüfung |
| `magic-string` | 1.4.5 | MIT | Bauzeit/Prüfung |
| `mri` | 1.2.0 | MIT | Bauzeit/Prüfung |
| `nanoid` | 3.3.20 | MIT | Bauzeit/Prüfung |
| `obug` | 2.2.1 | MIT | Bauzeit/Prüfung |
| `picocolors` | 1.1.1 | ISC | Bauzeit/Prüfung |
| `picomatch` | 4.0.7 | MIT | Bauzeit/Prüfung |
| `playwright` | 1.65.0 | Apache-2.0 | Bauzeit/Prüfung |
| `playwright-core` | 1.65.0 | Apache-2.0 | Bauzeit/Prüfung |
| `postcss` | 8.5.33 | MIT | Bauzeit/Prüfung |
| `readdirp` | 4.1.2 | MIT | Bauzeit/Prüfung |
| `rolldown` | 1.2.19 | MIT | Bauzeit/Prüfung |
| `sade` | 1.8.1 | MIT | Bauzeit/Prüfung |
| `source-map-js` | 1.2.4 | BSD-3-Clause | Bauzeit/Prüfung |
| `std-env` | 4.3.0 | MIT | Bauzeit/Prüfung |
| `style-mod` | 4.1.5 | MIT | Laufzeit |
| `svelte` | 5.57.3 | MIT | Bauzeit/Prüfung |
| `svelte-check` | 4.7.8 | MIT | Bauzeit/Prüfung |
| `magic-string` | 0.30.21 | MIT | Bauzeit/Prüfung |
| `tinybench` | 6.2.4 | MIT | Bauzeit/Prüfung |
| `tinyexec` | 1.3.1 | MIT | Bauzeit/Prüfung |
| `tinyglobby` | 0.2.17 | MIT | Bauzeit/Prüfung |
| `tslib` | 2.9.1 | 0BSD | Bauzeit/Prüfung (Plattformpaket) |
| `typescript` | 6.0.3 | Apache-2.0 | Bauzeit/Prüfung |
| `vite` | 8.3.6 | MIT | Bauzeit/Prüfung |
| `vitefu` | 1.1.4 | MIT | Bauzeit/Prüfung |
| `vitest` | 5.1.0 | MIT | Bauzeit/Prüfung |
| `obug` | 3.0.0 | MIT | Bauzeit/Prüfung |
| `w3c-keyname` | 2.2.8 | MIT | Laufzeit |
| `why-is-node-running` | 3.2.1 | MIT | Bauzeit/Prüfung |
| `zimmerframe` | 1.1.5 | MIT | Bauzeit/Prüfung |

<!-- UI-PAKETE:END -->

## 4a. Logo, Symbole und Namen

- `branding/` (Icon „VC.“ und Wortmarke „VintageCoder.“ in allen Varianten, erzeugt von `tools/gen_logo.py`; die Buchstaben sind als Umrisse (Pfade) aus JetBrains Mono Bold übernommen, die Schrift selbst ist nicht enthalten und steht unter der SIL Open Font License 1.1, siehe oben): Eigenwerk des Projektinhabers (Christian Drüeke), in der Seite „Über VintageCoder“ und in den Paketen verwendet. Es liegt im Repository, damit sich das Programm bauen lässt; das **Logo** und der **Name „VintageCoder“** sind von der GPL **ausgenommen**: Der Name darf nur als wahrheitsgemäße Herkunftsangabe genannt werden; das Logo darf als Referenz verwendet werden (Verweis auf VintageCoder, auch skaliert oder einfarbig), nicht aber verändert, als Zeichen eines anderen Produkts oder mit dem Eindruck einer Zugehörigkeit oder Billigung (die GPL gilt für den Programmcode, nicht für Logo und Namen). Fragen dazu an vintagecoder@drueeke.de.
- Das Programmsymbol (`installer/macos/VintageCoder.icns`, `VintageCoder-1024.png`, `installer/windows/VintageCoder.ico`, das Linux-Symbol) wird von `tools/gen_icon.py` aus dem Icon unter `branding/` erzeugt und ist ein Eigenwerk (Standardbibliothek).
- „VintageCoder“ ist der Name des Projekts (siehe oben); „vcc“ und „vc-lsp“ sind Namen der zugehörigen Programme.

## 4b. Beispiele und Testdaten

- `examples/`: Beispielprojekte des Projekts unter der **MIT-0-Lizenz** (Fremdanteile ausgenommen). In den Beispielen sind keine Urheberhinweise Dritter vermerkt; der einzige bekannte Fremdanteil ist der RMT-Player (Abschnitt 1). Einige Beispiele folgen bekannten Spielprinzipien (zum Beispiel „Prallball“ als Breakout-ähnliches Spiel).
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
- **Eingebaute Notschrift** (`crates/vc-roms/src/fallback.rs`, nur noch Notbehelf, wenn der Datenordner `data/charsets/` fehlt; sonst ersetzt der eigene Satz „VC-Standard“ ein fehlendes ROM): Eigenbau von VintageCoder (5×7-Zeichen in 8×8-Zellen, Buchstaben, Ziffern, Satzzeichen), unter der Lizenz von VintageCoder. Sie ist keine Nachbildung der Schrift eines Systems.

### Mitgelieferte Zeichensätze (Eigenwerke)

Die Sätze „VC-Standard“ unter `data/charsets/` sind **Eigenwerke des VintageCoder-Projekts** und stehen unter **CC0-1.0** (Lizenzvermerk: `data/charsets/LICENSE.txt`; Rechtstext: https://creativecommons.org/publicdomain/zero/1.0/legalcode). Es wurden **keine Original-ROMs und keine Nachbauten** (auch keine Zeichensätze anderer Emulatoren, Firmwares oder Rechner-Nachbauten) gelesen, kopiert, abgepaust oder als Vorlage angesehen; die Formen entstehen nach dem Stilblatt `data/charsets/STIL.md` („Fasen-Grotesk 8“, Stand 1) aus Regeln (Werkzeug `tools/gen_vc_standard.py`, Regelentwurf aus der Sitzung; die Gestaltung ist vom Projektinhaber freigegeben (2026-11-17)). Das ist ein Entstehungsprotokoll und **keine Rechtsaussage**; eine Rechtsauskunft wurde nicht eingeholt, nichts hier ist als „rechtlich sicher“ gemeint. Die Gleichheitsprüfung gegen die ROMs des Projektinhabers (`tools/charset_distance.py`, nur lokal, nur Zahlen) ist **offen**. Ändert sich ein Satz, ändert sich sein SHA-256 (`origin.check.set_sha256`), und ein Test schlägt an.

| id | System | Satz | Urheber | Entstanden | Werkzeug | Lizenz | Prüfstand |
|---|---|---|---|---|---|---|---|
| `vc-c64-upper` | c64 | upper | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-c64-lower` | c64 | lower | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-atari-standard` | atari8 | standard | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-atari-international` | atari8 | international | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-plus4-standard` | plus4 | standard | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-plus4-lower` | plus4 | lower | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-vic20-upper` | vic20 | upper | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-pet-upper` | pet | upper | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-pet-lower` | pet | lower | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-vic20-lower` | vic20 | lower | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-c128-upper` | c128 | upper | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-c128-lower` | c128 | lower | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-c128-vdc-upper` | c128 | vdc-upper | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-c128-vdc-lower` | c128 | vdc-lower | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-mega65-upper` | mega65 | upper | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |
| `vc-mega65-lower` | mega65 | lower | VintageCoder-Projekt | 2026-11 | `tools/gen_vc_standard.py` 1 | CC0-1.0 | Abstandsprüfung nicht bestanden (Rest vom Projektinhaber akzeptiert), Gestaltung freigegeben 2026-11-17 |

Ein fehlendes ROM wird seit der Anbindung an die Oberfläche durch diese Sätze ersetzt (Etikett „VC-Standard (eigen)“); die alte Notschrift (`crates/vc-roms/src/fallback.rs`, oben) greift nur noch, wenn der Datenordner fehlt (Konzept „Eigene Standardzeichensätze“, Entscheidung 13).
