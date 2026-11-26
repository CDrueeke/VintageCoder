<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# Mindestliste Editorfunktionen (Vorschlag, bitte ergänzen)

Stand: 2026-11-20 · Branch `claude/ui-e1-2`. Der Code-Editor soll alle Funktionen haben, die ein guter Code-Editor haben muss. Diese Liste ist ein **Vorschlag**: bitte Zeilen streichen, ergänzen oder die Etappe ändern. Ergänzungen kommen als Rückstand dazu; die geplanten Teile von E1.2 werden schon gebaut.

Status: `übernommen` = gebaut und im Ablauftest, `offen` = in der genannten Etappe.

Die gleiche Liste steht als Abschnitt 14 in `docs/paritaet-oberflaeche.md`; diese Datei ist die zum Ergänzen gedachte Fassung.

| Funktion | Etappe | Status |
|---|---|---|
| Blockfaltung (Marken, Befehle Alles falten/entfalten, Tastatur) | E1.1 | übernommen |
| Suchen und Ersetzen mit Groß/Klein, ganzem Wort, **regulären Ausdrücken** (Erfassungsgruppen im Ersetzen) | E1.1 | übernommen (CodeMirror-Suchleiste; Erfassungsgruppen `$1` im Handtest prüfen) |
| Gehe zu Zeile | E1.1 | übernommen |
| Rückgängig/Wiederholen, Zeilennummern, Klammerhervorhebung, Klammern ergänzen, Einrückung übernehmen | E1.1 | übernommen |
| Tab/Umschalt+Tab (Einrücken), Auswahl (Wort, Zeile, alles), Zwischenablage | E1.1 | übernommen |
| Lesezeichen, Haltepunkte am Rand, Schriftgröße | E1.1 | übernommen |
| Zeilen verschieben, duplizieren, löschen | E1.2 | offen (CodeMirror-Befehle, Tasten klären) |
| Kommentar umschalten (Zeile, Auswahl) | E1.2 | offen (`vc-ide-model::editor_text::toggle_comment` vorhanden) |
| Mehrfachcursor, Auswahl erweitern, Wort- und Zeilenoperationen | E1.2 | offen |
| Einrückungslinien, Leerzeichen und Zeilenenden anzeigen, Zeilenumbruch umschalten | E1.2 | offen |
| **Autovervollständigung** (Vokabular, Marken, Direktiven aus `data/asm/directives.json`, Sprachdienst) | E1.2 | offen |
| **Kontextbezogenes Prettyprint** (`vc-ide-model::pretty`; beim Zeilenwechsel, für Auswahl und Datei, als ein Rückgängig-Schritt) | E1.2 | offen |
| Hover/Befehlshilfe, Signaturhilfe, Diagnosen (Wellenlinie, Fehlerkarte), Quick-Fix | E1.2 | offen |
| Gehe zu Definition, Gliederung, Projektsuche (mit Abbruch) | E1.2 | offen |
| Snippets (Textbausteine) | E1.2 | offen, Vorschlag |
| Bauen, Probleme anklicken, „Ab hier starten“, Zyklenzahl je Zeile | E1.3 | offen |

## Entscheidungen des Inhabers zu den Ergänzungen (2026-11-20)

| Funktion | Etappe | Status |
|---|---|---|
| Fehlersprung F8 / Umschalt+F8 (Befehle „Nächstes/Voriges Problem“ in Menü und Palette; auf dem Mac sind F-Tasten oft Mediatasten: Palette ⌘K bleibt der sichere Weg) | E1.2 | übernommen (im Editor; über Dateien hinweg mit dem Bauergebnis in E1.3) |
| Umbenennen eines Namens über Dateien hinweg | nach 0.3.0 | zurückgestellt (eigene Kernarbeit mit Symbolindex) |
| Zur passenden Klammer springen (Ctrl+M, auf der deutschen Tastatur erreichbar; Menü Bearbeiten ▸ Suchen und Ersetzen) | E1.2 | übernommen |
| Zeilen sortieren (markierte Zeilen), Leerzeichen am Zeilenende entfernen (Auswahl oder Datei); je ein Rückgängig-Schritt, **keine** Automatik beim Speichern (Byte-Gleichheit); Einstellung frühestens E12 | E1.2 | übernommen |
| Zahlen-Hover (Hex, Dezimal, Binär, Low-/High-Byte, Registername aus den Chip-Daten) | E1.2 | übernommen; Farbvorschau bei Farbregistern offen (E1.2 Rückstand) |
| Geteilte Ansicht (zwei Editor-Hälften, Parität zu Iced); Matrixzeilen Teilen | E1.4 | offen (zusammen mit Sitzung und Layout; vom Inhaber bestätigt) |
| Randleiste mit Markierungen wie im Mockup | E1.2 | übernommen (Rand mit Haltepunkten, Lesezeichen, Diagnosen) |
| Minimap | – | abgelehnt (Inhaber) |
| Sticky-Kopfzeile (aktuelle Marke oben anheften) | später | Wunschliste, bei Bedarf |

Keine weiteren Wünsche.
