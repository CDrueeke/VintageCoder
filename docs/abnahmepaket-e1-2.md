# Abnahmepaket E1.2: Sprachdienste und Editor-Intelligenz

Stand: 2026-11-20 · Branch `claude/ui-e1-2` (aus `claude/ui-e1-1`). Nur privat gepusht, nichts in `main`. E1.1 (Teil 2) wird getrennt vom Inhaber geprüft; Befunde daraus kommen auf `claude/ui-e1-1` und werden per Merge hierher geholt.

## Inhalt (`~/Development/abnahme-ui/e1-2/`)
| Teil | Ort |
|---|---|
| App (macOS arm64, unsigniert, Rechtsklick ▸ Öffnen; enthält den Ordner `data/` im Bündel) | `VintageCoder.app` |
| Beispielprojekte (Kopien) | `beispiele/{c64,atari,mega65}/jumprun` |
| Handprüfliste (Abschnitt 0 mit E1.1-Rest, Wiederholung und neuen Punkten, dann E1.2; 80 Punkte) | `handpruefung-e1-2.md` |
| Bilder je Ansicht (de, en, xx, hell, dunkel) | `bilder/` |
| Ablauftest der Sprachdienste (Ausgabe und Bilder) gegen die Release-Richtlinie | `ci/e2e-lsp.txt`, `bilder/e2e/` |
| Ablauftest des Editors aus E1.1 (Rückfallprüfung) | `ci/e2e-editor.txt` |
| Selbstmessung im echten Fenster (Stile, Richtlinie) | `ci/selftest-csp-echtes-fenster.json` |
| Ausgabe der lokalen CI | `ci/lokale-ci.txt` |
| Paritätsmatrix, Mindestliste Editorfunktionen, Textdokumente (Brücke) | `paritaet-oberflaeche.md`, `editor-mindestliste.md`, `text-dokumente.md` |

## Was gebaut ist
- **Brücke zu den Sprachdiensten** (`vc-shell/src/lsp.rs`): `vc_lsp::Server` im Prozess; die Oberfläche meldet Assembler-Reiter an (`doc_open`, `doc_change` gebündelt nach 220 ms, `doc_close`), mit Version, UTF-16-Spannen und Neuabgleich bei Versionssprung. Antworten zu älterem Text werden verworfen. Festlegung: `docs/text-dokumente.md`.
- **Diagnosen:** Ereignis `lsp:diagnostics` mit `key` und `args` (I2), formatiert in der Oberfläche; Wellenlinie, roter Punkt im Rand, Zähler in der Statuszeile, Ablage unter dem Editor mit „Probleme“ (Klick springt).
- **Hover:** Karte mit Befehlshilfe (Tabelle der Adressierungsarten, gewählte Zeile ▶), Zahlen und Chipregister, Marken mit Fundstelle, Fehlerkarte (Meldung, Code, „Korrigieren“), BASIC-Karten (Adresse, Register, Wertzerlegung), `{clr}`-Zeichenangaben.
- **Vervollständigung:** Befehle, Marken, Konstanten, Makros, Chipregister, Direktiven (kanonische Namen zuerst, alte Namen erst beim Tippen mit Hinweis), Zeichennamen nach `{`; von Hand mit Strg+Leertaste.
- **Gehe zu Definition** (F12, ⌘-Klick, Befehl), **Gliederung**, **Projektsuche** im Hintergrund (abbrechbar, ungespeicherte Reiter zählen mit), **Quick-Fix** (⌘., Hover-Schaltfläche) für alte Direktivennamen, **Prettyprint** (Zeile beim Verlassen, Zeile/Auswahl, Datei; je ein Rückgängig-Schritt; BASIC nach Blocktiefe), **Kommentar umschalten** (⌘/), **Zeilenumbruch**, **Zeilenende LF/CRLF**, **Kontextmenü im Editor**.
- **Bau:** `data/` kommt über `tauri.release.conf.json` ins App-Bündel (siehe `docs/ui-aufbau.md`); neue Abhängigkeit `@codemirror/lint` 6.9.8 (MIT, in `THIRD-PARTY-NOTICES.md`).

## Entscheidungen, die ich getroffen habe (bitte bestätigen oder ändern)
1. **Nur Assembler-Reiter** sind beim Sprachdienst angemeldet (`vc-lsp` versteht nur Assembler). BASIC bekommt Hover und Zeichennamen aus dem Kern ohne Sprachdienst, Formatierung und Kommentar.
2. **Diagnosen zählen nur Fehler und Warnungen** in Liste und Statuszeile; Hinweise (Schweregrad 3) erscheinen als dezente gepunktete Unterstreichung.
3. **Prettyprint beim Verlassen der Zeile ist standardmäßig an** (wie im Iced-Editor); ein Schalter kommt mit Einstellungen ▸ Editor (E12), bis dahin nur über den Browserspeicher.
4. **Quick-Fix steht nicht als Diagnose da**: alte Direktivennamen bleiben gleichwertig und stumm (Konzept VCASM); das Angebot kommt per ⌘. und an der Hover-Karte.
5. **Neuer Befehl** `korrigieren` (Bearbeiten, ⌘.) in der Befehlstabelle.
6. **Ablage unter dem Editor** (Probleme, Suche, Gliederung) statt Seitenspalte; die Seitenspalte bleibt dem Baum.
7. **Mehrfachcursor, Zeilen verschieben/duplizieren, Auswahl erweitern** kommen als CodeMirror-Standardtasten ohne Menüeinträge (Menüeinträge nach Ihrer Entscheidung zur Mindestliste).

## Bekannte Lücken
- Signaturhilfe, Snippets, Einrückungslinien, unsichtbare Zeichen anzeigen, Kodierung ISO-8859-1/BOM (Matrix: offen, E1.2 Rückstand), Diagnosen ein- und ausschalten, „Hilfe zum Wort“ im Kontextmenü, Hinweis „Definition außerhalb des Projekts“ mit eigenem Text.
- Zahlenwerkzeuge, BASIC in Assembler umwandeln, Hex-Vorschläge, Zeichenpanel, Zeilenadressen und Zyklen am Rand (E1.3, E13): getrennte Etappen.
- Windows und Linux nicht geprüft (nur macOS; Ablauftest in WebKit gegen den echten Kern).
- Im Ablauftest sind die Diagnosen des Assemblers nur so gut wie `vc-lsp`; der Kontext einbindender Dateien (Include) wird vom Sprachdienst bestimmt (Projektdatei, `.include`).

## Lokale CI
Siehe `ci/lokale-ci.txt` (cargo build/test des Arbeitsbereichs, `release.py check`, `check_schreibweise`, `check_spdx`, `check_i18n`, `gui_notices --check`, `ui_notices`, `test_gui_notices`, `npm run check`, `npm test`, `npm run build`).

## Eichung (Konzept Abschnitt 12)
E1.2 deckt die Matrixzeilen der Abschnitte 5 bis 7 und 10 weitgehend ab (über 50 Zeilen); der Rückstand (Zahlenwerkzeuge, Zeichenpanel, Signaturhilfe, Snippets) ist in der Matrix offen. Eine PT-Angabe habe ich nicht erfasst.
