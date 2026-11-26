# Abnahmepaket E2: Disk-Editor

Stand: 2026-11-21 · Branch `claude/ui-e2` (aus `claude/tauri-ui`). Nur privat gepusht, nichts in `main`.

## Inhalt (`~/Development/abnahme-ui/e2/`)
| Teil | Ort |
|---|---|
| App (macOS arm64, unsigniert, Auslieferungsbau) | `VintageCoder.app` |
| Beispielprojekte und Diskettenkopien | `beispiele/`, `disketten/` |
| Handprüfliste (28 Punkte) | `handpruefung-e2.md` |
| Ablauftest Disk-Editor, Ausgabe der lokalen CI, Bilder | `ci/`, `bilder/` |

## Was gebaut ist
Daten ▸ Disk nach Mockup neu-11: links Diskette (Format, Belegung, Aktionen, Übernahme, Prüfbericht), Mitte Verzeichnis und Belegungskarte (Canvas mit Textalternative, Pfeiltasten), rechts Sektor-Inspektor (Hex, Zeichen, Folgen/Zurück). D64, D71, D81, ATR öffnen (Befehl, Palette, Klick im Projektbaum), Prüfen, Vormerken und Übernehmen ins Projekt (BASIC als Quelltext, auch Assembler, roh), Dateien ablegen, Speichern, Speichern unter, Neue Diskette, Rückfrage bei ungespeicherter Diskette (auch in der Beenden-Liste), Einlegen im Standard-Emulator.
Kern: `vc-shell/src/disk.rs` (`DiskDok` als `Dokument` im `DocStore`), Befehle `disk_*`; Logik unverändert aus `vc_ide_model::diskdoc`.

## Abweichungen
- Haltepunkt auf Sektoradresse (E6) und MEGA65-Übertragung (Hardwareprüfung) fehlen.
- Schreibschutz der Datei fehlt (nur „Image ungespeichert“).
- Der Befehl „Diskette öffnen …“ steht in der Befehlspalette und nicht in einem Menü (Menü Datei ist voll); Menüordnung in E12.
- Namen mit Grafikzeichen erscheinen als „▒“ (keine C64-Zeichensatzschrift).
