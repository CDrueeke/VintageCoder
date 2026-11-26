# Abnahmepaket E1.1 (Teil 1): Editor-Kern

Stand: 2026-11-20 · Branch `claude/ui-e1-1` (aus `claude/tauri-ui`, enthält E0, I2, VCASM-Direktiven, K1). Nur privat gepusht, nichts in `main`.

## Inhalt (`~/Development/abnahme-ui/e1-1/`)
| Teil | Ort |
|---|---|
| App (macOS arm64, unsigniert, Rechtsklick ▸ Öffnen) | `VintageCoder.app` |
| Beispielprojekte (Kopien) | `beispiele/{c64,atari,mega65}/jumprun` |
| Handprüfliste (37 Punkte) | `handpruefung-e1-1.md` |
| Bilder je Ansicht in de, en, xx, hell und dunkel (jetzt mit Editor und Suchleiste) | `bilder/<de\|en\|xx>/` und `bilder/layout-befunde.txt` |
| Ablauftest des Editors (Ausgabe und Bilder) | `ci/e2e-editor.txt`, `bilder/e2e/` |
| Ausgabe der lokalen CI | `ci/` |
| Paritätsmatrix (26 Zeilen von „offen“ auf „übernommen“) | `paritaet-oberflaeche.md` |
| Messwerte E0 samt Bildtakt (Frage e) | `messwerte-e0.md` |

## Was gebaut ist
- **Rust (`vc-shell`):** Befehle `file_read`, `file_write` (atomar, Zeilenenden `lf`/`crlf` bleiben, Endzeile bleibt, Rundlauf byte-gleich getestet), `file_create`, `dir_create`, `file_rename`, `dir_rename`, `file_delete` (Papierkorb) über `vc-ide-model::fileops` samt Nachführen der `.vcprj`, `syntax_color` (aus `vc-ide-model::syntax`, Spalten in UTF-16 für CodeMirror), `fold_ranges` (aus `fold::ranges`, mit der Direktivenliste `data/asm/directives.json`).
- **Oberfläche (`ui/`):** CodeMirror 6 (Pakete festgenagelt, Lizenzliste nachgeführt), Reiter (Ziehen, Pfeiltasten, ungespeichert-Punkt, Nachfrage beim Schließen), Projektbaum mit Neu/Umbenennen/Löschen/Ziehen/Kontextmenü/Tastatur, Färbung, Faltung, Rand mit Haltepunkten und Lesezeichen, Suchen/Ersetzen (Texte aus dem Katalog), Gehe zu Zeile, Rückgängig, Statuszeile mit Cursorposition, Befehle aus der Tabelle verdrahtet (Speichern, Alle speichern, Schließen, Neu, Umbenennen, Löschen, Rückgängig, Wiederholen, Suchen, Ersetzen, Gehe zu Zeile, Falten, Alles auswählen, Haltepunkt).
- **Nachbesserungen aus E0:** Doppelklick Titelleiste (Ursache: eigenes Umschalten zusätzlich zu dem der Tauri-Ziehfläche, entfernt), Zuletzt-Liste höchstens 5, Streifen für die eingeklappte Spalte mit Tooltip ⌘B, sortierte Prüfsumme in der Handprüfliste.
- **Frage (e) Bildtakt:** geklärt, kein Drosseln: der Wert ist zwei Bildintervalle der Anzeige (macOS 100 Hz → 20 ms; Windows per Remotesitzung 32 Hz → 62,5 ms). Siehe `messwerte-e0.md`.
- **Linux** der E0-Messung ist nachgetragen (VM, Software-Rendering).

## Entscheidungen, die ich getroffen habe (bitte bestätigen oder ändern)
1. **Textdokumente leben in der Oberfläche** (CodeMirror-Zustand samt Verlauf); Rust liest, schreibt, färbt, faltet. Das weicht vom Muster „Dokument samt Verlauf in Rust“ ab (Konzept 4.2), weil CodeMirror den Verlauf, die Auswahl und die Eingabemethoden selbst verwaltet; für Grafik und Musik gilt das Muster weiter. Der DocStore wird für Text nicht benutzt.
2. **Gehe zu Zeile** als eigener Dialog (nicht die CodeMirror-Leiste), weil deren Leiste beim Reiterwechsel einen Fehler auslöste.
3. **Tab** fügt ein Tabulatorzeichen am Cursor ein (wie im Iced-Editor), Tabbreite 4.
4. **Lesezeichen** (neu): Umschalt+Klick im Rand, ⌘F2, F2/Umschalt+F2. Sie stehen nicht in der Befehlstabelle (Menüeinträge kommen mit E1.3).
5. **Fenster-Ziehen und Ablegen von Dateien aus dem System** ist abgeschaltet (`dragDropEnabled: false`), damit das Ziehen innerhalb der Oberfläche unter Windows funktioniert.

## Bekannte Lücken (alle in der Paritätsmatrix als „offen“ mit Etappe E1.1)
- Art des Reiters umschalten (Text/Assembler/BASIC), neue Notiz, Zen-Modus, Editor-Schrift größer/kleiner, nächster/voriger Reiter per Tastatur (Befehle nicht in der Tabelle), Reiter verschieben per Befehl, überlange Reiterleiste.
- Baum: Filter, „Alle zuklappen“, „Neu einlesen“, Zustand der Ordner merken, Pfad kopieren, „Im Ordner zeigen“, Baumaktionen rückgängig, Neue Datei mit Art, Dateiart-Symbole/Gruppenüberschriften, Löschen „endgültig mit Rückfrage“, Include-Pfade beim Verschieben, „Ziel öffnen“.
- Datei über Dialog öffnen (⌘O), Speichern unter, unbenannte Reiter, Änderung von außen erkennen, Medien/Hex/Hilfe öffnen (kommt mit den jeweiligen Editoren), Nicht-Text im Standardprogramm öffnen; Palette: Dateien als Treffer.
- **Ungespeicherte Änderungen beim Projektwechsel und Beenden** werden noch nicht abgefragt (Teil von E1.4: „Beenden mit ungesicherten Änderungen“).
- Befehlsnamen für die Färbung: nur 6502, 65C02 und 45GS02 in einer Auszugsliste; die volle Liste je Prozessor kommt mit den Sprachdiensten (E1.2). Markdown wird ungefärbt gezeigt.
- Auf Windows und Linux ist dieser Stand nicht geprüft (nur macOS; Ablauftest läuft in WebKit gegen den echten Kern).
- Keine Prüfung mit Eingabemethoden (IME, chinesisch usw.) außer toten Tasten von Hand.

## Lokale CI (Stand der Übergabe)
Cargo build und test des Arbeitsbereichs (1478 Tests, 0 Fehler), `release.py check`, `check_schreibweise`, `check_spdx`, `check_i18n`, `gui_notices --check`, `test_gui_notices`, `npm run check`, `npm test`, `npm run build`: alle grün. Ablauftest `ui/tools/e2e-editor.mjs`: 20 Prüfungen, 0 Fehler.

## Einordnung des Aufwands (Eichung, Konzept Abschnitt 12)
Von rund 60 Matrixzeilen der Etappe E1.1 sind 26 übernommen, darunter alle tragenden (Editor, Reiter, Färbung, Speichern, Dateiverwaltung); der Rest sind viele kleinere Zeilen. Eine PT-Eichung kann ich seriös erst geben, wenn der Rest erledigt ist; ich habe die Zeit dieses Teils nicht in PT erfasst.
