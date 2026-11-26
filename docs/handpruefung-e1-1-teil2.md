# Handprüfung E1.1 (Teil 2): Rest des Editor-Kerns

Stand: 2026-11-20 · Branch `claude/ui-e1-1` · App: `~/Development/abnahme-ui/e1-1-teil2/VintageCoder.app` (unsigniert: Rechtsklick ▸ Öffnen) · Beispielprojekte (nur Kopien) in `…/e1-1-teil2/beispiele/`.

**Zu wissen:** Teil 1 (`handpruefung-e1-1.md`) bleibt gültig; wiederholt bitte nur die Punkte, bei denen ihr Befunde hattet. Hier stehen die **neuen** Funktionen. Ausdrücklich nicht dabei: Sprachdienste (E1.2), Bauen/Starten (E1.3), Einstellungen (E12), Gruppenüberschriften im Baum.

Prüfsumme der Kopien vorher merken:
```bash
cd ~/Development/abnahme-ui/e1-1-teil2/beispiele && find . -type f -exec shasum {} + | sort -k2 | shasum
```
Projekt: `…/beispiele/c64/jumprun`. Tastatur deutsch.

| Nr | Handlung | Erwartet | Ergebnis |
|---|---|---|---|
| 1 | Datei in `lib/` mit `.include`-Zeilen suchen (oder selbst anlegen: `lib/t2.asm` mit `.include "t1.inc"`, daneben `lib/t1.inc`), `t1.inc` per Ziehen in die Wurzel (leere Fläche unter dem Baum) | Datei zieht um; Meldung „verschoben (n Pfad(e) angepasst)“; die Include-Zeile in `t2.asm` lautet jetzt `"../t1.inc"`; ein offener Reiter von `t2.asm` zeigt den neuen Text | |
| 2 | Rechtsklick auf die leere Fläche unter dem Baum ▸ „Verschieben rückgängig“ (oder Palette ⌘K ▸ „Baumaktionen rückgängig“) | Datei liegt wieder in `lib/`, Include-Zeile wie vorher | |
| 3 | Beim Verschieben eine geänderte (ungespeicherte) Datei offen haben | Meldung „erst alle geänderten Dateien speichern“, nichts wird verschoben | |
| 4 | ⌘N (Neuer Reiter); Text tippen; ⌘S | Reiter „Unbenannt 1“; beim Speichern Dialog „Speichern unter“ mit Namensvorschlag `neu.asm`; danach trägt der Reiter den Dateinamen, die Datei liegt im Projekt, der Baum zeigt sie | |
| 5 | ⌘⇧T (Neue Notiz) | Reiter „Notiz 1“, ungefärbt; Speichern unter schlägt `notiz.md` vor | |
| 6 | Rechtsklick auf Baumordner ▸ „Neue Datei“; Art wählen (Assembler, BASIC, Text, Notiz); Name ohne Endung eingeben | Datei bekommt die Endung der Art (.asm, .bas, .txt, .md); mit eigener Endung bleibt diese | |
| 7 | Rechtsklick auf Baumordner ▸ „Neue Notiz“ | Dialog mit Vorschlag `notiz.md`; legt die Datei im Ordner an | |
| 8 | Menü Bearbeiten ▸ Art Text / Assembler / BASIC bei einem offenen Reiter | Färbung und Faltung schalten sich entsprechend; Statuszeile nennt die Art | |
| 9 | Menü Ansicht: Editor-Schrift größer / kleiner (ohne Tastenkürzel) | Schrift ändert sich schrittweise (9 bis 28), bleibt nach Neustart erhalten | |
| 10 | Menü Ansicht ▸ Zen (⌘⇧B) | Baum und Statuszeile verschwinden, Reiter und Fläche bleiben; erneut: zurück | |
| 11 | Mehrere Reiter offen: Menü/Palette „Nächster Reiter“, „Voriger Reiter“, „Reiter nach links/rechts“ | Wechsel bzw. Verschieben der Reihenfolge, rundum | |
| 12 | Rechtsklick auf einen Reiter (oder Menütaste) | Menü: Schließen, Andere schließen, Alle schließen, Nach links, Nach rechts, Pfad kopieren, Im Ordner zeigen | |
| 13 | „Pfad kopieren“ (Reiter oder Baum), danach in ein anderes Programm einfügen | Absoluter Pfad der Datei | |
| 14 | „Im Ordner zeigen“ | Finder öffnet mit der Datei markiert | |
| 15 | Palette ⌘K, Teil eines Dateinamens tippen | Treffer in der Gruppe „Dateien“; Enter öffnet die Datei; Befehle wie Speichern sind jetzt auswählbar, wenn ein Reiter offen ist | |
| 16 | Baumfilter: Lupe über dem Baum, Text eingeben | Nur Treffer (mit ihren Ordnern, aufgeklappt); Esc leert und schließt; „Keine Treffer.“ bei nichts | |
| 17 | Ordner aufklappen, App beenden und neu starten, Projekt wieder öffnen | Aufgeklappte Ordner sind wieder aufgeklappt | |
| 18 | Rechtsklick auf leere Baumfläche ▸ „Alle zuklappen“, „Neu einlesen“ | Alle Ordner zu; Baum liest neu (Datei im Finder anlegen, dann „Neu einlesen“: erscheint) | |
| 19 | Eine offene, **ungeänderte** Datei in einem anderen Programm ändern, zurück in die App | Die App liest sie neu ein (Meldung), Cursor und Marken bleiben | |
| 20 | Dieselbe Datei im Editor ändern (nicht speichern) **und** außerhalb ändern, zurück in die App | Meldung, dass beim Speichern überschrieben wird | |
| 21 | Datei im Finder löschen, zurück in die App | Meldung „außerhalb gelöscht“, Reiter bleibt als ungespeichert stehen | |
| 22 | Nichttext (z. B. aus `graphics/` ein PNG, falls vorhanden) im Baum anklicken | Öffnet im Standardprogramm des Systems, Meldung; kein Reiter | |
| 23 | ⌘O (Datei öffnen) | Systemdialog im Projektordner; Datei im Projekt öffnet als Reiter; Datei außerhalb: Meldung „liegt außerhalb des Projekts“ | |
| 24 | Ungespeicherte Änderung, dann Menü „Startseite“ bzw. „Projekt öffnen“ | Dialog „Ungespeicherte Änderungen“ mit Alle speichern / Verwerfen / Abbrechen; jede Wahl tut, was sie sagt | |
| 25 | Ungespeicherte Änderung, dann ⌘Q bzw. Fensterknopf Schließen | Derselbe Dialog; „Abbrechen“ hält die App offen, „Verwerfen“ beendet | |
| 26 | Datei anklicken, Entf | Nachfrage; Papierkorb (auf macOS immer vorhanden) | |
| 27 | Färbung von Befehlen prüfen: in `.asm` mit 65C02/45GS02-Befehlen (z. B. `bra`, `inz`, `ldq`) | Auch diese sind als Befehle gefärbt (nicht nur die 56 der 6502) | |
| 28 | **Befunde aus Teil 1, bitte zuerst:** Datei öffnen, Seite auf/ab, ⌘↑/⌘↓, Pos1/Ende, Mausrad; Fenster auf 900×600 verkleinern; Reiter wechseln | Zeilennummern, Reiterleiste, Rand und Text bleiben immer an ihrem Platz, nur der Text scrollt; Mausrad geht; nichts wird leer (Ursache war die Inhaltsrichtlinie, die CodeMirrors Stile blockierte) | |
| 29 | ⌘F, Ersetzen (⌘⌥F) | Suchleiste gestaltet (Felder, Knöpfe, Kontrollkästchen), Text bleibt sichtbar; Block 4 (Teil 1, Nr. 19–23) komplett wiederholen | |
| 30 | Tab, Umschalt+Tab (mit und ohne Auswahl) | Tabulatorzeichen am Cursor; mit Auswahl rückt Tab ein, Umschalt+Tab aus | |
| 31 | `lda (a),y` mit mehreren Klammern in einer Zeile tippen | Jede öffnende Klammer bekommt ihre schließende (auch die zweite und dritte) | |
| 32 | Linke Spalte einklappen | Der Streifen am Rand trägt einen Pfeil; Tooltip mit ⌘B; Klick klappt aus | |
| 33 | Rechtsklick im Baum: Umbenennen, Löschen, Neue Datei (je mit der Maus anklicken); auch F2 und Entf | Jeder Eintrag tut, was er sagt (früher: ohne Wirkung) | |
| 34 | Menü Projekt ▸ „Projekt schließen“ (⌘⇧W) | Zurück zur Startansicht; bei ungespeicherten Änderungen erst die Abfrage | |
| 35 | Neue Datei, Art BASIC | Endung `.bas` (nicht `.sbas`) | |
| 36 | `graphics/test.prg` im Baum anklicken (liegt in den Kopien dieses Pakets) | Öffnet im Standardprogramm, Meldung, kein Reiter, kein Absturz | |
| 37 | Alle Punkte aus Teil 1, bei denen es Befunde gab | Wie dort erwartet | |

Bekannte Lücken: `docs/abnahmepaket-e1-1-teil2.md`.
