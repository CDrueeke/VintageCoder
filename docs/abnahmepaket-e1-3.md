# Abnahmepaket E1.3: Bauen und Starten

Stand: 2026-11-20 · Branch `claude/ui-e1-3` (aus `claude/ui-e1-2`). Nur privat gepusht, nichts in `main`.

## Inhalt (`~/Development/abnahme-ui/e1-3/`)
| Teil | Ort |
|---|---|
| App (macOS arm64, unsigniert, Auslieferungsbau mit eingebettetem Frontend, `data/` im Bündel) | `VintageCoder.app` |
| Beispielprojekte (Kopien, `common/` daneben) | `beispiele/` |
| Handprüfliste (23 Punkte) | `handpruefung-e1-3.md` |
| Ablauftest Bauen/Starten/Tests (Ausgabe, Bilder) gegen die Release-Richtlinie | `ci/e2e-bau.txt`, `bilder/e2e/` |
| Rückfallprüfung Editor und Sprachdienste | `ci/e2e-editor.txt`, `ci/e2e-lsp.txt` |
| Ausgabe der lokalen CI | `ci/lokale-ci.txt` |
| Paritätsmatrix | `paritaet-oberflaeche.md` |

## Was gebaut ist
- **Bauen** im Hintergrund (`build_start`, Ereignisse `build:started`/`build:done`), ein Bau zur Zeit, vorher werden alle Reiter gespeichert; Hauptquelle des Projekts oder aktive Datei; Bauergebnis (Ziel, Artefakt, Größe, Segmente, Zeilenadressen, Zyklen, Marken) bleibt für Statuszeile und Panel.
- **Starten/Beenden** (`run_start`, `run_stop`, Ereignisse `run:started`/`run:failed`/`run:stopped`) im Emulator des Systems oder auf einer eingerichteten Maschine; frühes Ende des Emulators wird als Fehler gemeldet.
- **Oberfläche:** Kopfzeile (Bauen F7, Starten F5, Beenden, Startziel-Chip mit Auswahl), Ablage unter dem Editor mit Probleme (Bauergebnis plus Sprachdienste), **Ausgabe** (Protokoll, Kopieren/Leeren), Tests; Statuszeile (baut/gebaut/Fehler, Speicherstreifen), **Zyklenzahl je Zeile** im Rand, Panel „Maschine“ (⌘⌥B), Schublade (⌘J), Kontextmenüs Probleme und Ausgabe, Befehle `problem_*`, `protokoll_*`.
- **Tests** über `*.vctest` (`tests_start/stop/state`, Ereignis `tests:update`): alle oder fehlgeschlagene, statisch oder im Emulator, Filter, „nur Fehler“, Detail mit Messwert und Registern, Sprung zur Quelle.
- **Prettyprint** beim Verlassen der Zeile jetzt auch beim Verlassen mit Enter (Ursache des „nur bei fehlerfreien Zeilen“: Enter ersetzt die bearbeitete Zeile im selben Schritt und löste nichts aus; mit Fehler hatte es nichts zu tun) und ohne erneutes Formatieren nach Rückgängig.
- Kernänderung (klein, nur ergänzend): `vc-ide::build` liefert je Zeile `cyc` (kleinste/größte Zyklenzahl).

## Abweichung vom Umfang (vom Inhaber entschieden: wie vorgeschlagen)
**Nicht in E1.3, sondern mit der Debug-Sitzung (E6):** Debuggen (F6), „Ab hier starten“ (Bis hierher), Register, Flags und das Bild der laufenden Maschine (Bildstrom `vc_emu::framestream`). Gründe: Sie brauchen eine Sitzung mit Emulator-Anbindung (Haltepunkte setzen, anhalten, Register lesen), die im Konzept zu E6 gehört. Das Panel „Maschine“ zeigt in E1.3 Ziel, Zustand und das Bauergebnis.

## Entscheidungen, die ich getroffen habe
1. **Ablage unter dem Editor** (Probleme, Ausgabe, Tests, Suche, Gliederung) statt „Schublade“ plus Panel mit Reitern; das rechte Panel ist „Maschine“.
2. **Probleme** zeigen das Bauergebnis und die Meldungen der Sprachdienste getrennt (Gruppe „Bauergebnis“).
3. **Tests in der Prüfbrücke** starten nie einen Emulator (nur statisch), echte Läufe nur im gebauten Programm.

## Bekannte Lücken
Kopfzeile in Breitenstufen (voll, Symbole, Mehr-Menü), Minimieren/Maximieren der Ablage, Panel-Breite, Takt und Norm in der Statuszeile, Setup-Zeile im Testdetail, Start auf Hardware (MEGA65/Ultimate), Alt+Buchstabe für Menüs, ausgegraute Palettenbefehle mit Grund; Windows/Linux ungeprüft.
