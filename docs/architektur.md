# Aufbau des Quelltexts

Dieses Blatt ist die Landkarte für alle, die im Quelltext suchen. Die Regeln für Beiträge stehen in [CONTRIBUTING.md](../CONTRIBUTING.md), die Gestaltung der Oberfläche in [DESIGN.md](../DESIGN.md), Prüfung und Bildvergleich in [pruefen.md](pruefen.md).

## Crates (16 Mitglieder des Workspace)

| Schicht | Crate | Aufgabe |
|---|---|---|
| Fachlogik ohne Oberfläche | `vc-core` | gemeinsame Grundtypen (Meldungen) |
| | `vc-target` | Hardware-Datenbank: Zielsysteme, CPUs, Chips aus `data/` |
| | `vc-asm` | 6502-Assembler (Mehrpass, Makros, Zyklen, Segmente) |
| | `vc-basic` | BASIC-Dialekte, Tokenizer, Umsetzung BASIC → Assembler |
| | `vc-disk` | Disk-Images (ATR, D64, D71, D81) |
| | `vc-emu` | Emulator-Anbindung (VICE, atari800 und weitere Adapter) |
| | `vc-media` | Sprites, Zeichensätze, Bildschirme, Kacheln, Paletten: Dateiformate und Export |
| | `vc-roms` | ROM-Verzeichnis, Erkennung, Rollen je System |
| | `vc-md` | Markdown für den Editor und die Hilfe (Auszeichnung, Schreibhilfen) |
| | `vc-lsp` | Sprachdienst als Bibliothek (Diagnosen, Hover, Vervollständigung, Gliederung) |
| | `vc-ide` | Dienste der Entwicklungsumgebung: Bauen, Debuggen, Start im Emulator, Hilfe, Einstellungen, Release-Auswahl zur Laufzeit |
| Modell | `vc-ide-model` | Dokumentmodelle der Studios, Layout, Dateioperationen, Hilfe-Zustand: alles, was eine Oberfläche braucht, ohne Fenster |
| Iced-Bausteine | `vc-gui-ui` | Befehlstabelle (`befehle`), Muster und Studio-Rahmen (`muster`), Bereiche (`bereich`), Texte (`texte`), Farben (`theme`), Schriften, Kontextmenü, Zeichenfläche (`flaeche`) |
| | `vc-gui-editor` | Quelltext-Editor: Modell (`core`) und Iced-Widget (`widget`) |
| Anwendung | `vc-gui` | die Oberfläche `vintagecoder` (Programm `target/release/vintagecoder`) |
| Kommandozeile | `vc-cli` | das Programm `vcc` |

Die frühere egui-Oberfläche `vc-app` und die frühere Web-Oberfläche gibt es nur noch in der Git-Historie.

## Wegweiser durch `crates/vc-gui/src`

| Datei | Zweck |
|---|---|
| `main.rs` | Zustand (`App`), Nachrichten (`Msg`), `update` und `view`, Befehle; dazu die Tests (Galerie, Bildvergleich, Bedienung mit simulierten Ereignissen) |
| `*studio.rs` | die Studios: `studio.rs` (Sprite, Player/Missile, Zeichensatz), `screenstudio.rs`, `tilestudio.rs`, `bankstudio.rs`, `palettestudio.rs`, `songstudio.rs` (Tracker), `diskstudio.rs` |
| `studioaktionen.rs` | Aktionen der Studios als Liste für Befehlspalette und Kontextmenü |
| `*dlg.rs` | Dialoge: `exportdlg.rs`, `importdlg.rs`, `infodlg.rs` (Über, Feedback, Update), `kopierdlg.rs`, `hexdlg.rs`, `zeichendlg.rs` |
| `neuprojekt.rs`, `startui.rs`, `assistent.rs` | Neues Projekt, Startseite, Konfigurationsassistent |
| `setui.rs`, `zielui.rs`, `romui.rs`, `argsui.rs`, `konfig.rs`, `einstellungen.rs` | Einstellungen: Seiten, Zielsysteme und Hardware, ROMs, Startparameter, Konfiguration sichern, Oberflächen-Einstellungen |
| `projeinst.rs`, `project.rs`, `textdatei.rs`, `gruppen.rs` | Projekteinstellungen, Projektordner, Textdateien (Kodierung, Zeilenende), Reitergruppen |
| `debug.rs`, `dbgview.rs`, `testlauf.rs` | Debug-Sitzung (Arbeits-Thread) und ihre Ansicht, Test-Explorer |
| `lsp.rs`, `prettyzeile.rs`, `mdansicht.rs`, `mdhilfen.rs`, `zahlen.rs`, `basic2asm.rs`, `hexansicht.rs` | Editor-Zubehör: Sprachdienst, Prettyprint, Markdown-Ansicht und Schreibhilfen, Zahlenwerkzeuge, BASIC → Assembler, Hex-Ansicht |
| `tastatur.rs`, `telemetrie.rs`, `audio.rs`, `bildcache.rs`, `pfadfeld.rs` | Tastaturlayout für die Tracker-Klaviertasten, Takt-Anzeige der Statusleiste (rein lokal), Audioausgabe, Bild-Handles, Pfadfeld mit „Wählen …“ |
| `aussen.rs`, `native.rs` | alles, was Prozesse startet oder das System anfasst: **nur** über `aussen` (im Testlauf wird nichts gestartet); `native` ist die macOS-Menüleiste und die Bedienungshilfen |

## Die Oberfläche bauen und prüfen

- Entwicklungsstand als macOS-App: `crates/vc-gui/make-app.sh` baut mit `cargo build --release -p vc-gui` und legt `build/gui/VintageCoder.app` an (ad hoc signiert; nur für die Entwicklung, nicht der Release-Installer). Das Zielverzeichnis von Cargo ist dort ein eigener Ordner unter `~/.cache`, damit sich die Läufe nicht in die Quellen schreiben; mit `CARGO_TARGET_DIR` änderbar.
- Die Iced-Pakete sind mit `=` genau gepinnt (`iced` und `iced_*` 0.14, `iced_tiny_skia` 0.14.1, `tiny-skia` 0.11.4). Grund: Die Tests und der Bildvergleich rechnen mit dem Software-Renderer; andere Versionen können Pixel und damit die Referenzbilder verschieben. Ein Wechsel ist eine eigene Änderung mit neuen Referenzen.
- Tests der Oberfläche, Galerie und Bildvergleich: [pruefen.md](pruefen.md). Umgebungsvariablen der Galerie: `VC_ZUSTAENDE` (Auswahl der Zustände, durch Kommas getrennt), `VC_VERGLEICH` (Namenszusatz der Ausgabedateien unter `build/vergleich/`), `VC_UPDATE_REFS=1` (Referenzen neu schreiben, nur nach Freigabe).

## Weitere Wege durch die Dokumentation

[Handbuch](handbuch/kapitel/) (auch in der Anwendung), Hilfeseiten unter `crates/vc-ide/help/` (Tests dazu in `crates/vc-ide/src/help.rs`), Kommandozeile [cli.md](cli.md), Release [release-auswahl.md](release-auswahl.md), Update-Feed [update-feed.md](update-feed.md), Sprachserver [sprachserver.md](sprachserver.md).
