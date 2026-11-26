# Abnahmepaket E3: Sprite-Editor

Stand: 2026-11-21 · Branch `claude/ui-e3` (aus `claude/ui-e2`, E2 ist noch nicht freigegeben). Nur privat gepusht, nichts in `claude/tauri-ui` oder `main` gemerged.

## Inhalt (`~/Development/abnahme-ui/e3/`)
| Teil | Ort |
|---|---|
| App (macOS arm64, unsigniert, Auslieferungsbau) | `VintageCoder.app` |
| Beispielprojekte (Sprites C64, MEGA65, Atari Player/Missile) | `beispiele/` |
| Handprüfliste (56 Punkte, nach Teilschritten E3.1 bis E3.5) | `handpruefung-e3.md` |
| Ablauftest Sprite-Editor, Ausgabe der lokalen CI, Bilder | `ci/`, `bilder/` |

## Was gebaut ist (Grafik ▸ Sprites, nach Mockup neu-4)
- **E3.1 Zeichenfläche:** Canvas mit lokalem Vorschaustrich (ein Strich ist ein Verlaufsschritt, Lücken schließt der Kern), Stift, Radierer, Pipette, Farbwerte 1 bis 4, Palette (PAL/NTSC bei Atari), Zoom 100 bis 3200 %, Raster mit Bytelinien, Verschieben mit Leertaste, Rückgängig/Wiederholen, Speichern, Rückfrage beim Dateiwechsel und Beenden. Tastaturbedienung ohne Maus mit Ansage der Zelle.
- **E3.2 Werkzeuge:** Linie, Rahmen, Füllen, Auswahl, Kopieren/Ausschneiden/Einfügen, Spiegeln, Verschieben, Leeren, Rechtsklick-Menü der Auswahl.
- **E3.3 Bilder:** Bilderliste mit Vorschau, Neu/Duplizieren/Löschen, Modus Hires/Multicolor, Hardware-Flags (C64, C128, MEGA65 im VIC-II-Modus), Onion Skin, Vorschau im Spiel mit festem Hintergrund, Größe im Speicher.
- **E3.4 Animation und Austausch:** Animationen mit Zeitleiste und Abspielen, PNG laden, Export-Dialog (Assembler, Konstanten, Binär, PRG, BASIC DATA; speichern, als Include einfügen, **in den Quelltext kopieren ohne Include**, BASIC DATA **mit optionaler Einleseroutine**, im Emulator testen), neue Sprite-Datei.
- **E3.5 Atari Player/Missile:** Breite je Spieler und Priorität, Vorschau skaliert, Export schreibt die Werte. Kernänderung additiv (siehe Handprüfliste); ohne diese Werte ist alles Byte-gleich wie vorher.
- **Terminologie:** Englisch „VIC-20“, Deutsch „VC-20“; „Studio“ ist in allen Katalogwerten durch „Editor“ ersetzt, die Schreibweisenprüfung sichert das.

Kern: `vc-shell/src/pixel.rs` (`PixelDok` im `DocStore`) und `export.rs`; die Logik kommt aus `vc_ide_model::pixel`.

## Abweichungen
- Bild wechseln mit **Alt+Pfeil** und **Bild-auf/Bild-ab** (Pfeiltasten allein bewegen die Zellmarke, das ist für die Bedienung ohne Maus nötig).
- „Alles auswählen“ im Menü Bearbeiten wirkt im Sprite-Editor nicht (Systemmenü); stattdessen Befehlspalette oder Rechtsklick-Menü der Auswahl.
- Die Karte „Speicher“ zeigt nur Größen (Zeiger und Adresse kommen mit Export-Daten später); die Beschriftung im Mockup „Vor dem Hintergrund“ heißt hier wie im Register `$D01B`: „Hinter dem Hintergrund“.
- Hintergrund der „Vorschau im Spiel“ fest; ein Screen aus dem Projekt später (E8).
- Vorlage laden: nur für Zeichensätze (E4); Karten der Seitenspalte lassen sich nicht einklappen (die Spalte scrollt).
- Neue Datei: nur Sprites und Player/Missile hier; die übrigen Arten mit ihren Editoren.
- Nicht in E3 (vorgemerkt): MEGA65 VIC-IV-Sprites (16/64 Punkte breit, 16 Farben) als eigenes Paket.
- Öffnen und Speichern von PNG: beim Laden wird die Datei zuerst gespeichert.
- Die englischen und deutschen Namen der Zielsysteme in Dateien unter `data/` (Vorlagen, Zielbeschreibungen) bleiben deutsch („VC-20“); eine Übersetzung dieser Daten ist noch offen.

## Prüfumfang (schmale Stufe, siehe `ci/lokale-ci.txt`)
Sprite-Ablauftest 132 Prüfungen ohne Fehler; alle weiteren Ablauftests (start, editor, lsp, bau, layout, split, sitzung, disk, neuprojekt) grün; Rust (vc-shell, vc-commands, vc-ide-model, vc-media), Vitest, npm check, i18n, SPDX, Schreibweise grün.
Nicht geprüft: VoiceOver an der Zeichenfläche (E13), echter Emulatorstart („Im Emulator testen“), Windows/Linux.
