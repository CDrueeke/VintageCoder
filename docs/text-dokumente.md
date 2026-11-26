# Textdokumente im Code-Editor: Zuständigkeiten (Ergänzung zu Konzept 4.2)

Stand: 2026-11-20 (E1.1). Gilt für Quelltext, BASIC und Text/Markdown. Für Grafik, Musik, Disk und Hex gilt weiter „Dokument samt Verlauf in Rust“.

## Entscheidung und Begründung
Der **Text und sein Verlauf (Rückgängig) liegen in der Oberfläche**, im Zustand von CodeMirror 6 (`EditorState`, je Reiter einer). Gründe: CodeMirror verwaltet Auswahl, Mehrfachcursor, Eingabemethoden (IME, tote Tasten), Faltung und den Verlauf selbst, jeder Tastendruck wäre sonst ein Rundlauf zu Rust, und die Vorschau muss ohne Wartezeit stehen. Rust bleibt zuständig für alles, was Dateien und Fachwissen betrifft.

| Aufgabe | Wer | Wie |
|---|---|---|
| Text halten, Verlauf, Auswahl | Oberfläche | `EditorState` je Reiter (`ui/src/lib/editor.svelte.ts`) |
| Datei lesen | Rust | `file_read` → `{text, eol, lang}`; Zeilenenden werden auf `\n` vereinheitlicht und als `lf`/`crlf`/`gemischt` gemeldet; Nicht-UTF-8 und Binärdateien werden abgelehnt (`datei-kein-utf8`, `datei-nichttext`) |
| Datei schreiben | Rust | `file_write {rel, text, eol}`: atomar (Nachbardatei, dann umbenennen), stellt `crlf` wieder her, Endzeile bleibt; Rundlauf byte-gleich getestet |
| Färbung, Faltung | Rust | `syntax_color`, `fold_ranges` aus `vc-ide-model`; die Oberfläche fragt 80 ms nach der letzten Änderung und verwirft Antworten zu veraltetem Text (Vergleich der Dokumentidentität) |
| Dateiverwaltung, Verschieben mit Include-Anpassung, Rückgängig | Rust | `file_*`, `dir_*`, `tree_move`, `tree_undo` (Fachschicht `fileops`, `reorganize`) |

## Dirty-Zustand
Ein Reiter ist **ungespeichert**, wenn sein Text nicht mehr gleich dem Text beim letzten Lesen oder Schreiben ist (`Text.eq` auf dem gemerkten Stand, nicht ein Zähler). Rückgängig bis zum gespeicherten Stand macht den Reiter wieder sauber. Unbenannte Reiter sind ungespeichert, sobald sie Text enthalten.

## Speichern
Nur der Oberflächenstand wird gespeichert (`file_write`). Danach gilt er als gespeicherter Stand und als Änderungsstand der Datei (Größe, Zeit) für die Erkennung von Änderungen von außen. Beim Zurückkehren ins Fenster (`focus`) werden die offenen Dateien verglichen (`file_stats`): saubere Reiter werden neu gelesen (Marken und Cursorstelle bleiben), ungespeicherte melden einen Konflikt und werden beim Speichern überschrieben. Vor Projektwechsel, Beenden und Fensterschließen fragt die Oberfläche nach, solange Reiter ungespeichert sind; Verschieben im Baum verlangt vorher Speichern.

## Brücke zu den Sprachdiensten (E1.2), gebaut
Modul `crates/vc-shell/src/lsp.rs`; `vc_lsp::Server` läuft im Prozess, ohne Rohr. Der Server arbeitet auf dem **Oberflächenstand**, nie auf der Datei.
- Rust hält je offenem Assembler-Reiter eine Spiegelung `{version, text}`. Die Oberfläche meldet an: `doc_open {rel, text}` (auch erneut, um nach einem Versionssprung den ganzen Text zu liefern), `doc_change {rel, version, changes: [{from, to, insert}]}`, `doc_close {rel}`. Reiter mit anderer Art (BASIC, Text, Notiz) und unbenannte Reiter sind nicht angemeldet.
- **Änderungen** werden nach 220 ms Tippruhe gebündelt (CodeMirror-`ChangeSet.compose`); die Spannen gelten im Text **vor** dem Bündel, in UTF-16-Einheiten, Rust wendet sie von hinten nach vorn an. `version` zählt je Bündel um eins hoch; bei einem Sprung antwortet Rust mit dem Fehler `doc-version` (unbekanntes Dokument: `doc-unbekannt`), die Oberfläche sendet dann `doc_open` mit dem ganzen Text.
- **Anfragen** (`lsp_hover`, `lsp_complete`, `lsp_definition`, `lsp_outline`, `lsp_fixes`) bekommen Zeile ab 0 und Spalte in UTF-16-Einheiten; Rust rechnet auf Zeichen um und die Antworten zurück. Vorher wartet die Oberfläche, bis ausstehende Änderungen beim Kern sind (`lspFlush`). Ohne Projekt: Fehler `kein-projekt`.
- **Diagnosen**: Ereignis `lsp:diagnostics {rel, version, items: [{line, endLine, from, to, severity, code, key, args, message}]}` je Datei (auch für Dateien, die ein offener Reiter einbindet). `key` und `args` (verschachtelt, I2) formatiert die Oberfläche mit dem Katalog (`msgText`), `message` ist nur der Rückfall. Die Oberfläche wendet eine Antwort nur an, wenn der Text seither unverändert ist; Spalte 0 bis Zeilenende heißt ganze Zeile.
- **Ohne Sprachdienst** (stateless): `text_hover` (Zeichenangaben `{clr}` in Zeichenketten; nacktes BASIC: Zahlen, Register, Wertzerlegung aus `vc-ide-model::basichover`), `char_names` (Namen nach `{`), `format_lines` (Prettyprint aus `vc-ide-model::pretty`, liefert nur geänderte Zeilen), `search_start`/`search_cancel` mit den Ereignissen `search:hits` und `search:done` (Projektsuche im Hintergrund, abbrechbar, ungespeicherte Reiter gehen der Datei vor).
- Verschieben und Umbenennen: `doc_close` mit altem, `doc_open` mit neuem Namen (`rekey` in `editor.svelte.ts`). Nach dem Neulesen von außen: `doc_close` und `doc_open`.
- Die Sprache der Texte des Servers wird bei `lang_set` nachgeführt; Meldungen mit `key` formatiert ohnehin die Oberfläche.
- Datenordner: `VC_DATA`, sonst neben dem Programm (App-Bündel: `Resources/data`, über `bundle.resources` in `tauri.release.conf.json`, siehe `docs/ui-aufbau.md`), sonst im Arbeitsordner (Entwicklung).
