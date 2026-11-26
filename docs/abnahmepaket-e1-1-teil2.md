# Abnahmepaket E1.1 (Teil 2)

Stand: 2026-11-20 · Branch `claude/ui-e1-1` (privat gepusht, nicht in `main`). Der feste Stand von Teil 1 unter `~/Development/abnahme-ui/e1-1/` ist unverändert.

## Inhalt (`~/Development/abnahme-ui/e1-1-teil2/`)
`VintageCoder.app` (macOS arm64, unsigniert), `beispiele/` (Kopien), Bilder je Ansicht in de/en/xx hell/dunkel mit Editor und Suchleiste (`bilder/`, Layoutprüfung: 0 Befunde), Ablauftest (`ci/e2e-editor.txt`, `bilder/e2e/`). Dazu in `docs/`: `handpruefung-e1-1-teil2.md` (28 Punkte), `paritaet-oberflaeche.md`, `text-dokumente.md`, `messwerte-e0.md`.

## Befunde aus dem Handtest Teil 1 (Stand dieser Lieferung)
| Befund | Ursache | Stand |
|---|---|---|
| A Streifen ohne Symbol | – | Pfeil darauf, Tooltip, Klick |
| B Tab ohne Wirkung | Tab rückte die Zeile ein statt am Cursor einzufügen; im Handtest zusätzlich verdeckt durch D | `insertTab`; im Ablauftest |
| C, D, F, N, O, P Scrollen, Zeilennummern/Inhalt fehlen, Suche ungestaltet, leerer Editor beim Verkleinern | **Inhaltsrichtlinie des Auslieferungsbaus (`style-src 'self'`) blockierte die zur Laufzeit eingefügten `<style>`-Elemente von CodeMirror.** Ohne sie: kein Scroller, Editor 17000 px hoch, Seite scrollt, Zeilennummern und Suchleiste ungestaltet. Vite-Entwicklung hat eine lockere Richtlinie, deshalb im Browser-Test nie aufgefallen. | `style-src 'self' 'unsafe-inline'` (nur Stile) und `dangerousDisableAssetCspModification: ["style-src"]` (sonst setzt Tauri einen Nonce, der `unsafe-inline` aufhebt). **Im echten Release-Fenster geprüft:** `ci/selftest-csp-echtes-fenster.json` (`style_inline_ok: ok`). Ablauftest läuft jetzt mit der Release-Richtlinie (`VC_CSP=release`) und schlägt mit der alten nachweislich an. |
| E Klammer nur bei der ersten | Standard von CodeMirror ergänzt nur vor Leerraum und bestimmten Zeichen | `before` erweitert; im Ablauftest |
| G „kein Projekt geöffnet“ bei `lib/neu.asm` | **nicht reproduzierbar**: gegen eine Kopie von `~/Documents/Handtest/neustand/10-jumprun` meldet die App „lib/neu.asm gibt es schon.“ (die Datei liegt dort) bzw. legt `lib/neu7.asm` korrekt an. Die Meldung „Es ist kein Projekt geöffnet“ erscheint nur, wenn der Kern kein Projekt kennt. Bitte Schritte nennen (Menü oder Knopf, Name, Zustand davor). | offen, Rückfrage |
| G `.sbas` | Ich hatte die Endung des strukturierten BASIC (`.sbas`, im Bau vorerst abgeschaltet) statt `.bas` (CBM-BASIC, wie im Iced-Stand) eingetragen | `.bas` |
| G Wunsch Verzeichnisauswahl im Dialog | – | **nicht gebaut**, offen (Entwurf nötig) |
| H App schloss sich | nicht ermittelt. Gefunden: das native Menü beendete die App bei „Beenden“ **ohne** Abfrage (jetzt über die Oberfläche mit Abfrage); Panik in einem Befehl bricht das Programm nicht mehr ab (Fehler „intern“, Eintrag in `vintagecoder-panik.log` im Temp-Ordner). Meine Aufräumbefehle treffen nur `target/debug/vintagecoder` und Vite, nie die App im Abnahmeordner. | teilweise |
| I, J, K Umbenennen, Löschen, Kontextmenü ohne Wirkung | Das Menü wurde vor der Aktion geschlossen; die Aktion las danach das schon gelöschte Ziel (Fehler in der Oberfläche, nicht sichtbar). Gleiches am Reiter-Menü. | behoben; Ablauftest klickt die Einträge mit der Maus |
| L Verweis in `.vcprj` | folgt aus I | der Kern zog ihn schon mit (Test), jetzt auch bedienbar |
| M Projekt schließen fehlt | „Startseite“ stand im Menü, war aber leicht zu übersehen | neuer Befehl „Projekt schließen“ (Menü Projekt, ⌘⇧W, Palette) mit Abfrage |
| Testdatei `.prg` für Punkt 34 | – | liegt in den Kopien (`graphics/test.prg`) |
| Fehler ohne sichtbare Meldung | – | alle Befehlsfehler laufen über die Meldungszeile; Panik wird zu „intern“ |

## Neu seit Teil 1
- **Abfrage ungespeicherter Änderungen** bei Projektwechsel, Beenden (Menü, ⌘Q) und Fensterschließen.
- **Mnemonics der Färbung** aus den Prozessortabellen (`data/cpus/*.cpu.json`, 6502 samt illegalen, 45GS02 samt Quad), 149 Namen statt 56.
- **Verschieben im Baum** über die Fachschicht (Include-Pfade und Projektverweise ziehen mit, Sicherung unter `.vcbackup/`, Rückgängig); offene Reiter folgen und lesen neu.
- Unbenannte Reiter und Notizen mit „Speichern unter“; Neue Datei mit Art; Neue Notiz im Baum.
- Art des Reiters umschalten, Editor-Schrift, Zen-Modus, Reiter wechseln und verschieben, Kontextmenü am Reiter.
- Baum: Filter, Alle zuklappen, Neu einlesen, Ordnerzustand je Projekt merken, Pfad kopieren, Im Ordner zeigen, Löschen mit Nachfrage ohne Papierkorb.
- Datei öffnen (⌘O), Nichttext im Standardprogramm, Änderungen von außen erkennen (beim Zurückkehren ins Fenster), Palette mit Dateien und freigeschalteten Editorbefehlen.
- **Konzeptnachtrag** `docs/text-dokumente.md`: Zuständigkeiten Oberfläche/Rust für Text, Dirty-Zustand, Speichern und die festgelegte Brücke zu den Sprachdiensten (`doc_open`/`doc_change`/`doc_close`, Versionen, UTF-16-Spannen). Bitte in Konzept 4.2 übernehmen; die Konzeptdatei selbst habe ich nicht angefasst.

## Matrix
Von den E1.1-Zeilen stehen 3 auf „offen“: (68) Medien, Hex und Hilfe aus dem Baum öffnen (kommt mit den jeweiligen Editoren, hier nur Text und Nichttext), (193) Gruppenüberschriften und Dateiart-Symbole im Baum, (695) Kontextmenü mit allen 10 Baumbefehlen (es fehlen „Als Include“ → E1.2, „Hex-Ansicht“ → E11). Alle übrigen E1.1-Zeilen sind „übernommen“.

## Bekannte Lücken und Hinweise
- Schriftgröße, Ordnerzustand und Ansicht Zen liegen nur in der Oberfläche (nicht in den gemeinsamen Einstellungen mit dem Iced-Stand; kommt mit E12/E1.4).
- „Alle schließen/Andere schließen“ fragt bei mehreren ungespeicherten Reitern nur für einen nach.
- Konflikt bei Änderung von außen: Hinweis, kein Dialog; Speichern überschreibt.
- „Datei außerhalb des Projekts“ lässt sich nicht öffnen (Meldung).
- Nur macOS von Hand prüfbar; Windows/Linux nicht getestet. Ablauftest läuft in WebKit gegen den echten Kern.
- **Auffällig:** Ein erster Lauf von `cargo test --workspace` meldete einmal 1 Fehler (parallel zu einem laufenden Bau), der zweite und dritte Lauf 1483/0; Ursache nicht ermittelt (vermutlich Wettlauf um Dateien in `/tmp` bei gleichzeitigem Bau). Wiederholbar nicht.

## Lokale CI
`ci/lokale-ci.txt`. Ablauftest `ui/tools/e2e-editor.mjs`: 55 Prüfungen, 0 Fehler.
