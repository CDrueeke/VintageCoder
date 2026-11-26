# Paritätsmatrix: Iced-Oberfläche gegen neue Tauri-Oberfläche

> Ergebnis der Gegenprüfung gegen den Code (Stand 2026-11-25): [paritaetsmatrix-0.3.0.md](paritaetsmatrix-0.3.0.md) mit Zahlen, Entscheidungsliste und Detailtabellen; die Statusspalte unten ist dort teils überholt.

Diese Matrix führt jede Einzelfunktion der bisherigen Iced-Oberfläche auf und ordnet ihr die Stelle in der neuen Tauri-Oberfläche (Svelte, Ordner `ui/`) und die Etappe zu, in der sie gebaut wird. Sie dient als Abnahmeliste: Eine Etappe ist erst fertig, wenn alle ihre Zeilen auf `übernommen` stehen (oder bewusst auf `gestrichen`). Stand: 2026-11-20 (E1.1 Teil 1: Editor-Kern, Reiter, Dateibaum-Verwaltung). Quelle: `crates/vc-gui/src/*.rs`, `crates/vc-gui-ui/src/*.rs`, `crates/vc-gui-editor/src/*.rs`.

Legende. Spalte „Iced (Datei)“: die Datei im Altbestand, in der die Funktion steht. Spalte „neu“: Ansicht oder Befehlskennung (Tabelle `vc-commands`) in der neuen Oberfläche. Etappe: E0 bis E13 laut Konzept. Status: `offen` (noch nicht gebaut), `übernommen` (gebaut und geprüft), `neu` (gibt es nur in der neuen Oberfläche), `gestrichen` (entfällt laut Konzept bewusst).

## 1. Rahmen, Navigation und Befehlspalette (E0)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Navigation Code / Grafik / Musik / Daten / Debug (Hauptreihe) | `main.rs` (Seitenleiste, Reiter, Kopfzeile) | Navigation `nav:code` … `nav:debug` | E0 | übernommen |
| Zweite Reihe bei Grafik (Sprites, Zeichensatz, Kacheln, Bildschirm, Karten, Palette) | `main.rs` (Dokumentenfeld je Dateiart) | Navigation `nav:grafik:*` | E0 | übernommen |
| Zweite Reihe bei Daten (Disk, Hex, Speicher) | `main.rs`, `hexansicht.rs`, `diskstudio.rs` | Navigation `nav:daten:*` | E0 | übernommen |
| Befehlspalette öffnen und schließen (Cmd+K), Gerüst | `main.rs` (`palette_ansicht`) | Befehlspalette (`palette`) | E0 | übernommen |
| Palette: unscharfe Suche, alle Wörter müssen vorkommen, Wortanfang zuerst | `befehle.rs` (`filtere`) | Befehlspalette | E0 | offen |
| Palette: Pfeiltasten wählen, Enter führt aus, Esc schließt | `main.rs` | Befehlspalette | E0 | offen |
| Palette: nicht nutzbare Befehle ausgegraut mit Grund (Projekt oder Editor nötig) | `main.rs` (`menue_grund`, `nutzbar`) | Befehlspalette | E1.3 | offen |
| Palette: Dateien des Projekts als Treffer | `main.rs` (`Eintrag::Datei`) | Befehlspalette | E1.1 | übernommen |
| Palette: zuletzt geöffnete Projekte als Treffer | `main.rs` (`Eintrag::Projekt`) | Befehlspalette | E1.4 | übernommen |
| Palette: Aktionen des gerade offenen Editors (Werkzeuge, Muster, Sektoren …) | `studioaktionen.rs` (`Eintrag`-Variante für Editor-Aktionen) | Befehlspalette, Aktionen je Ansicht | E3 | offen |
| Hell / Dunkel umschalten | `einstellungen.rs` (`Schema`), `main.rs` | `hell_dunkel` | E0 | übernommen |
| Farbschema „System“ folgt dem Betriebssystem | `main.rs` (`SystemSchema`) | Einstellungen ▸ Aussehen | E12 | übernommen (E12.1) |
| Farbschema „Hoher Kontrast“ | `theme.rs`, `main.rs` | `hoher_kontrast` | E12 | übernommen (E12.1) |
| Sprachwechsel Deutsch / Englisch / Pseudo-Sprache xx | `sprache.rs`, `setui.rs` (`Sprache`) | Sprachwahl in der Kopfzeile, `lang:*` | E0 | übernommen |
| Sprache „automatisch“ nach Betriebssystem | `sprache.rs` | `lang:auto` | E0 | offen |
| Oberflächen-Zoom (Cmd+Plus, Cmd+Minus, Cmd+0) | `main.rs` (`SetZoom`) | `zoom_plus`, `zoom_minus`, `zoom_null` | E12 | übernommen (E12.1) |
| Editor-Schrift größer und kleiner | `main.rs` (`SetSchrift`) | `editor_schrift_groesser`, `editor_schrift_kleiner` | E1.1 | übernommen |
| Darstellung zurücksetzen (Zoom, Schrift, Layout) | `main.rs` | `darstellung_zuruecksetzen` | E12 | übernommen (E12.1) |
| Komponentengalerie (Entwicklungsansicht) | – | `ui:gallery` | E0 | neu |
| Messansicht der Layouts (Entwicklungsansicht) | – | `ui:measure` | E0 | neu |
| Meldungen als Toasts („noch nicht gebaut“, Fehler) | `main.rs` (`hinweis`, Protokollzeilen) | Toasts | E0 | neu |
| Anzeige der Tastenkürzel je Plattform (⌘⇧⌥ oder Strg+Umschalt+Alt) | `befehle.rs` (`fuer_anzeige`) | Kürzelanzeige `Kbd` | E0 | offen |
| Startbild beim Programmstart (Klick oder Taste schließt) | `main.rs` (`splash`) | – | E1.4 | übernommen: kurzes Startbild (etwa 1 s, Klick oder Taste schließt), abschaltbar unter Einstellungen ▸ Allgemein |
| Fenstergröße und Position merken, kleiner Bildschirm maximiert | `main.rs` (`fenster_einstellungen`, `Bildschirm`) | Tauri-Fenster | E1.4 | übernommen |
| Hinweis „Update verfügbar“ in der Statusleiste | `main.rs`, `infodlg.rs` | Statusleiste | E12 | übernommen (E12.4) |
| Bedienung per Bildschirmleser (Namen und Beschreibungen der Flächen) | `main.rs` (`a11y`), `studio.rs` (Flächenbeschreibung) | Barrierefreiheit der Ansichten | E1.4 | teilweise (Namen, Sprache, Bereiche; Gerätetest offen) |

## 2. Menüs, Kürzel und Kontextmenüs (E0, E1.3)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Befehlstabelle als einzige Quelle für Menü, Palette und Kürzel | `befehle.rs` | Crate `vc-commands` | E0 | übernommen |
| Menüleiste aus der Tabelle (Datei, Bearbeiten, Ansicht, Projekt, Hilfe) | `main.rs` (`menueleiste`), `befehle.rs` | Menüleiste `Menu.svelte` | E0 | übernommen |
| Untermenüs der langen Menüs (Bearbeiten, Ansicht, Projekt) | `befehle.rs` (`UNTERMENUES`) | Menüleiste | E0 | offen |
| Menüleiste im Fenster ein- und ausschalten (Windows, Linux) | `main.rs` (`SetMenueleiste`) | Einstellungen ▸ Aussehen | E12 | offen |
| Systemmenü des Macs (Name, Über, Ausblenden) | `native.rs` | natives Menü aus `vc-commands` | E0 | offen |
| Alt+Buchstabe öffnet das Menü, Pfeile wechseln Menü und Untermenü | `main.rs` (`menue_oeffnen`), `befehle.rs` (`menue_buchstaben`) | Menüleiste, Tastatur | E1.3 | offen |
| Menüeinträge ausgegraut mit Grund | `main.rs` (`menue_grund`) | Menüleiste | E1.3 | offen |
| Kürzel auslösen (Cmd/Strg, Umschalt, Alt, F-Tasten) | `main.rs` (`Taste`), `befehle.rs` (`zerlege`) | `keys.ts` | E1.3 | offen |
| Doppelereignis eines Kürzels verwerfen (Menü und Taste gleichzeitig) | `main.rs` (`doppelt`) | Befehlsausführung | E1.3 | offen |
| Kürzel nach physischer Taste (Klaviertasten, AltGr bleibt Text) | `tastatur.rs` | Tastatur im Tracker | E5 | offen |
| Kürzel-Diagnose (Protokollzeile je Taste) | `main.rs`, `setui.rs` (`kuerzel_diagnose`) | Einstellungen ▸ Kürzel | E12 | kommt 0.3.1 (Entwicklerhilfe, kein Alltagswunsch) |
| Kontextmenü Dateibaum (Datei) | `kontext.rs` (`Art::Baum`) | Kontextmenü Dateibaum | E1.1 | übernommen |
| Kontextmenü Dateibaum (leere Fläche) | `kontext.rs` (`Art::BaumFlaeche`) | Kontextmenü Dateibaum | E1.1 | übernommen |
| Kontextmenü Reiter | `kontext.rs` (`Art::Reiter`) | Kontextmenü Reiter | E1.1 | übernommen |
| Kontextmenü Editor (Ausschneiden, Kopieren, Einfügen, Hilfe zum Wort, Definition, Falten, Haltepunkt) | `kontext.rs` (`Art::Editor`) | Kontextmenü Code-Editor | E1.2 | offen (alle Einträge bis auf „Hilfe zum Wort“; die kommt mit der Hilfe) |
| Kontextmenü Zeichenfläche (Rückgängig, Zwischenablage, Exportieren) | `kontext.rs` (`Art::Flaeche`) | Kontextmenü der Grafik-Editoren | E3 | offen |
| Kontextmenü Aktionen des Editors (Menütaste oder Umschalt+F10) | `kontext.rs` (Art der Editor-Aktionen), `studioaktionen.rs` | Kontextmenü je Ansicht | E3 | offen |
| Kontextmenü Problem (Springen, Kopieren, Alle kopieren) | `kontext.rs` (`Art::Problem`) | Kontextmenü Probleme | E1.3 | übernommen |
| Kontextmenü Protokoll (Kopieren, Leeren) | `kontext.rs` (`Art::Protokoll`) | Kontextmenü Protokoll | E1.3 | übernommen |
| Kontextmenü: Pfeiltasten überspringen Trenner und Ausgegrautes, Enter wählt | `kontext.rs` (`naechste`), `main.rs` | alle Kontextmenüs | E1.3 | übernommen |
| Tastenkürzel-Übersicht (Suche über Befehle und Kürzel) | `setui.rs` (`Seite::Kuerzel`), `startui.rs` (Schnellreferenz) | Einstellungen ▸ Tastenkürzel | E12 | übernommen (E12.1) |

## 3. Code-Editor: Reiter und Dokumentenfeld (E1.1)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Datei öffnen aus dem Dateibaum (Quelltext, Medien, Hex, Hilfe) | `main.rs` (`oeffne`) | Code: Editor-Reiter | E1.1 | offen |
| Datei über Dialog öffnen (Cmd+O) | `main.rs`, `native.rs` | `datei_oeffnen` | E1.1 | übernommen |
| Reiterleiste mit Ungespeichert-Marke | `main.rs` (`gruppe_ansicht`) | Code: Reiterleiste | E1.1 | übernommen |
| Reiter schließen (Cmd+W) mit Rückfrage bei Änderungen | `main.rs` (`Schliesse`) | `datei_schliessen` | E1.1 | übernommen |
| Nächster und voriger Reiter (Cmd+Alt+Rechts und Links) | `main.rs` | `naechster_reiter`, `voriger_reiter` | E1.1 | übernommen |
| Reiter per Maus ziehen und neu ordnen | `main.rs` (`TabDruck`, `tab_unter`) | Reiterleiste | E1.1 | übernommen |
| Reiter nach links und rechts verschieben | `main.rs` (`reiter_verschieben`) | `reiter_links`, `reiter_rechts` | E1.1 | übernommen |
| Überlange Reiterleiste: sichtbare Reiter wählen | `main.rs` (`sichtbare_reiter`) | Reiterleiste | E1.1 | übernommen |
| Teilen: zweite Hälfte rechts (Cmd+Alt+T) | `main.rs` (`teilen_umschalten`), `gruppen.rs` | `teilen` | E1.2 | übernommen |
| Reiter in die andere Hälfte schieben | `main.rs` (`tab_verschieben`) | `reiter_andere_haelfte` | E1.2 | übernommen |
| Reiter per Ziehen an den rechten Rand ablegen (Teilen) | `main.rs` (`ablegen_rechts`) | Dokumentenfeld | E1.2 | übernommen |
| Fokus zwischen beiden Hälften, Teilung aufheben | `main.rs` (`gruppe_fokus`) | Dokumentenfeld | E1.2 | übernommen |
| Neuer unbenannter Reiter (Cmd+N) | `main.rs` (`scratch_neu`) | `neuer_reiter` | E1.1 | übernommen |
| Neue Notiz (Markdown, Cmd+Umschalt+T) | `main.rs` | `neue_notiz` | E1.1 | übernommen |
| Speichern (Cmd+S), Speichern unter für Unbenannte | `main.rs` (`speichere`, `speichern_unter_dialog`) | `speichern` | E1.1 | übernommen |
| Alle speichern (Cmd+Alt+S) | `main.rs` | `alle_speichern` | E1.1 | übernommen |
| Nur-Text-Dateien im Editor, andere im Standardprogramm öffnen | `project.rs` (`ist_nichttext`), `main.rs` | Code: Editor-Reiter | E1.1 | übernommen |
| Datei außerhalb geändert: neu einlesen | `main.rs` (`editor_neu_lesen`) | Code: Editor-Reiter | E1.1 | übernommen |
| Ablegen einer Datei aus dem Betriebssystem in das Fenster (Import) | `main.rs` (`Abgelegt`), `importdlg.rs` | Import | E12 | offen |
| Zen-Modus (nur Reiter und Fläche, Cmd+Alt+Z; Cmd+Umschalt+B kam auf dem Rechner des Inhabers nicht an) | `main.rs`, `muster.rs` (`zen_setzen`) | `zen` | E1.1 | übernommen |
| Fokus-Maximieren des Bereichs unter dem Fokus (Cmd+Umschalt+Enter) | `main.rs` | `maximiere_fokus` | E1.4 | übernommen |

## 4. Code-Editor: Bearbeiten (E1.1)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Text eingeben, Löschen rückwärts und vorwärts | `core.rs` (`insert`, `backspace`, `delete`) | Code: Editor | E1.1 | übernommen |
| Enter übernimmt die Einrückung der Zeile | `core.rs` (`enter`) | Code: Editor | E1.1 | übernommen |
| Tab fügt ein Tabulatorzeichen ein | `widget.rs`, `main.rs` (`EdMsg::Tab`) | Code: Editor | E1.1 | übernommen |
| Einrückung der Auswahl erhöhen und verringern (Umschalt+Tab) | – (im Iced-Editor nicht vorhanden) | Code: Editor | E1.2 | übernommen |
| Klammern: passende Klammer hervorheben, Klammern automatisch schließen | – (im Iced-Editor nicht vorhanden) | Code: Editor | E1.2 | übernommen |
| Rückgängig und Wiederholen (Cmd+Z, Cmd+Umschalt+Z, Cmd+Y) | `core.rs` (`undo`, `redo`), `widget.rs` | `rueckgaengig`, `wiederholen` | E1.1 | übernommen |
| Ausschneiden, Kopieren, Einfügen, Alles auswählen | `widget.rs`, `main.rs` | `ausschneiden`, `kopieren`, `einfuegen`, `alle_auswaehlen` | E1.1 | übernommen |
| Cursor bewegen mit Pfeilen, Seite auf und ab, Pos1 und Ende | `core.rs` (`move_h`, `move_v`, `home`, `end`), `widget.rs` | Code: Editor | E1.1 | übernommen |
| Pos1 springt erst auf das erste Zeichen, dann an den Zeilenanfang | `core.rs` (`home`) | Code: Editor | E1.1 | übernommen |
| Cmd+Links und Cmd+Rechts als Zeilenanfang und Zeilenende (macOS) | `widget.rs` | Code: Editor | E1.1 | übernommen |
| Auswahl mit Umschalt+Pfeil, Umschalt+Klick, Ziehen mit der Maus | `core.rs` (`set_cursor`), `widget.rs` | Code: Editor | E1.1 | übernommen |
| Doppelklick wählt das Wort | `main.rs` (`EdMsg::Click`) | Code: Editor | E1.1 | übernommen |
| Scrollen senkrecht und waagerecht, Bildlaufleisten | `core.rs` (`scroll_by`, `set_hscroll`), `widget.rs` | Code: Editor | E1.1 | übernommen |
| Eingabemethoden (IME-Vorbearbeitung, Umlaute, Dead Keys) | `widget.rs` (`Preedit`) | Code: Editor | E1.1 | übernommen |
| Zeilennummern und Zeilen-Rinne | `widget.rs` | Code: Editor | E1.1 | übernommen |
| Syntaxfärbung Assembler (Mnemonics, Direktiven, Marken, Zahlen, Kommentare, Zeichenketten) | `highlight.rs`, `vc-ide-model` (`syntax`) | Code: Editor | E1.1 | übernommen |
| Syntaxfärbung BASIC | `highlight.rs`, `vc-ide-model` (`syntax`) | Code: Editor | E1.1 | übernommen |
| Art des Reiters umschalten (Text, Assembler, BASIC) | `main.rs` (`ArtText`, `ArtAsm`, `ArtBasic`) | `art_text`, `art_asm`, `art_basic` | E1.1 | übernommen |
| Zeile hervorheben, in der das Programm angehalten hat | `core.rs` (`debug_line`), `widget.rs` | Code: Editor, Debug | E6 | offen |
| Einfügemarke beim Ziehen einer Datei aus dem Baum | `core.rs` (`einfuegemarke`), `widget.rs` | Code: Editor | E1.2 | offen |
| Include-Zeile erzeugen beim Ablegen einer Datei im Quelltext | `main.rs` (`quelltext_ablegen`, `include_ablegen`), `dropinc.rs` | Code: Editor | E1.2 | offen |
| „Als Include einfügen“ aus dem Baum (Befehl) | `main.rs` (`AlsInclude`) | `als_include` | E1.2 | offen |
| Diese Datei ist schon eingebunden: Hinweis mit Zeile | `main.rs` | Code: Editor | E1.2 | offen |

## 5. Suchen, Ersetzen, Gehe zu (E1.2)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Suchleiste öffnen (Cmd+F), Auswahl übernehmen | `main.rs` (`find_leiste`) | `suchen` | E1.2 | übernommen |
| Treffer hervorheben, Zähler „n von m“ | `core.rs` (`set_search`, `match_lines`), `main.rs` (`find_info`) | Code: Suchleiste | E1.2 | übernommen |
| Nächster und voriger Treffer | `core.rs` (`step_match`), `main.rs` (`FindSchritt`) | Code: Suchleiste | E1.2 | übernommen |
| Groß-/Kleinschreibung umschalten | `main.rs` (`FindGrossKlein`) | Code: Suchleiste | E1.2 | übernommen |
| Ersetzen (Cmd+Alt+F): eins ersetzen | `core.rs` (`replace_current`), `main.rs` (`ErsetzeEins`) | `ersetzen` | E1.2 | übernommen |
| Alle ersetzen mit Anzahl | `core.rs` (`replace_all`), `main.rs` (`ErsetzeAlle`) | Code: Suchleiste | E1.2 | übernommen |
| Suchleiste schließen (Esc) | `main.rs` (`FindZu`) | Code: Suchleiste | E1.2 | übernommen |
| Suche aktualisiert sich bei jeder Textänderung | `main.rs` (`find_aktualisieren`) | Code: Suchleiste | E1.2 | übernommen |
| Gehe zu Zeile (Cmd+L) | `main.rs` (`goto_leiste`), `befehle.rs` | `gehe_zu_zeile` | E1.2 | übernommen |
| Gehe zu Definition (F12) | `main.rs` (`GeheZuDefinition`), `lsp.rs` | `gehe_zu_definition` | E1.2 | übernommen |
| Zur Zeile springen und mittig zeigen | `core.rs` (`center_on`), `main.rs` (`zeige`) | Code: Editor | E1.2 | übernommen |

## 6. Falten, Haltepunkte, Lesezeichen (E1.2)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Falten einzelner Bereiche per Klick in der Rinne | `core.rs` (`toggle_fold`, `fold_range`), `widget.rs` | Code: Editor | E1.2 | übernommen |
| Faltregeln je Sprache (Assembler, BASIC, Text) | `core.rs` (`lang`), `vc-ide-model` (`fold`) | Code: Editor | E1.2 | übernommen |
| Alles falten und alles entfalten | `main.rs` (`AllesFalten`, `AllesEntfalten`) | `alles_falten`, `alles_entfalten` | E1.2 | übernommen |
| Haltepunkt per Klick in der Rinne setzen und löschen | `widget.rs` (`EdMsg::Breakpoint`), `core.rs` (`breakpoints`) | Code: Rinne | E1.2 | übernommen |
| Haltepunkt am Cursor umschalten (F9) | `main.rs` (`HaltepunktUmschalten`) | `haltepunkt_umschalten` | E1.2 | übernommen |
| Haltepunkte der Datei bleiben beim Neuöffnen erhalten | `main.rs` (`marken_sichern`) | Code: Editor | E1.2 | offen |
| Lesezeichen setzen und zwischen Lesezeichen springen | – (im Iced nicht vorhanden) | Code: Rinne, Befehle | E1.2 | übernommen |

## 7. Formatieren, Zeilenende, Kodierung, Umbruch (E1.2)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Zeile formatieren (Assembler und BASIC) | `prettyzeile.rs`, `main.rs` | `zeile_formatieren` | E1.2 | übernommen |
| Datei formatieren als ein Rückgängig-Schritt | `prettyzeile.rs` | `datei_formatieren` | E1.2 | übernommen |
| Zeile beim Verlassen automatisch formatieren (Option) | `main.rs` (`pretty_pruefen`), `einstellungen.rs` (`pretty_zeile`) | Einstellungen ▸ Editor | E1.2 | übernommen (Schalter in Einstellungen ▸ Editor kommt mit E12; bis dahin an) |
| Zeilenende LF oder CRLF je Datei umschalten | `textdatei.rs` (`Meta`), `main.rs` (`Zeilenende`) | `zeilenende` | E1.2 | übernommen |
| Kodierung UTF-8 oder ISO-8859-1 je Datei umschalten | `textdatei.rs`, `main.rs` (`Kodierung`) | `kodierung` | E1.2 | offen |
| UTF-8 mit BOM erkennen und beim Speichern erhalten | `textdatei.rs` | Code: Editor | E1.2 | offen |
| Warnung bei Zeichen, die es in ISO-8859-1 nicht gibt | `textdatei.rs` | Code: Editor | E1.2 | offen |
| Zeilenumbruch umschalten (Ansicht) | `core.rs` (`wrap`), `main.rs` (`Umbruch`) | `umbruch` | E1.2 | übernommen |
| Umbruch als Voreinstellung speichern | `einstellungen.rs` (`umbruch`), `setui.rs` | Einstellungen ▸ Editor | E1.2 | übernommen |

## 8. Zahlenwerkzeuge, BASIC nach ASM, Markdown, Zeichenpanel (E1.2)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Zahlenwerkzeuge: Eingabe als Zahl oder Ausdruck ($D020, 0xFF, 1010) | `zahlen.rs` | `zahlenwerkzeuge` | E1.2 | offen |
| Zahlenwerkzeuge: Dezimal, Hex, Binär, Zeichen, Lo/Hi-Byte, mit Vorzeichen | `zahlen.rs` | `zahlenwerkzeuge` | E1.2 | offen |
| Zahlenwerkzeuge: Breite 8 oder 16 Bit, Bit-Editor (Bits umschalten) | `zahlen.rs` (`Breite`, `Bit`) | `zahlenwerkzeuge` | E1.2 | offen |
| Zahlenwerkzeuge: Wert kopieren oder an der Schreibmarke einsetzen | `zahlen.rs` (`Kopieren`, `Einsetzen`) | `zahlenwerkzeuge` | E1.2 | offen |
| Zahlenanzeige-Voreinstellung (Hex oder Dezimal) | `vc-ide-model` (`numpref`) | Einstellungen ▸ Editor | E1.2 | offen |
| BASIC in Assembler umwandeln (neue Datei, Hinweise unter Probleme) | `basic2asm.rs`, `main.rs` | `basic_nach_asm` | E1.2 | offen |
| BASIC: Hex-Vorschläge für Adressen (Dezimaladressen als Hex) | `hexdlg.rs` | `hex_vorschlaege` | E1.2 | offen |
| BASIC: Hover mit Adressbedeutung | `main.rs` (`basic_hover`), `vc-ide-model` (`basichover`) | Code: Editor | E1.2 | übernommen |
| Markdown: Vorschau und Quelltext umschalten | `main.rs` (`MdUmschalten`), `mdansicht.rs` | `md_umschalten` | E13 | offen |
| Markdown: Datei als Markdown anzeigen (auch ohne Endung .md) | `main.rs` (`md_als`) | `md_als` | E13 | offen |
| Markdown-Format: Fett, Kursiv, Code, Durchgestrichen | `mdhilfen.rs`, `befehle.rs` | `md_fett`, `md_kursiv`, `md_code`, `md_durchgestrichen` | E13 | offen |
| Markdown: Link, Bild einfügen (PNG im Projekt) | `mdhilfen.rs` | `md_link`, `md_bild` | E13 | offen |
| Markdown: Überschrift tiefer und höher | `mdhilfen.rs` | `md_tiefer`, `md_hoeher` | E13 | offen |
| Markdown: Aufzählung, Nummerierung, Aufgaben, Zitat, Aufgabe abhaken | `mdhilfen.rs` | `md_aufzaehlung`, `md_nummerierung`, `md_aufgaben`, `md_zitat`, `md_aufgabe` | E13 | offen |
| Markdown: Tabelle einfügen und ausrichten | `mdhilfen.rs` | `md_tabelle`, `md_ausrichten` | E13 | offen |
| Markdown-Vorschau: Überschriften, Listen, Tabellen, Code, Bilder, lange Dokumente blockweise | `mdansicht.rs`, `vc-md` | Markdown-Ansicht | E13 | offen |
| Zeichenpanel: Steuerzeichen und Zeichencodes des Zielsystems suchen und einfügen | `main.rs` (`panel_zeichen`), `vc-ide-model` (`charpanel`) | Panel Zeichen | E1.2 | offen |

## 9. Dateibaum und Projektsuche (E1.1, E1.2)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Dateibaum anzeigen (Ordner vor Dateien, alphabetisch, Gruppen) | `project.rs` (`baum_zeilen`), `main.rs` (`seitenleiste`) | Dateibaum `FileTree.svelte` | E0 | übernommen |
| Ordner auf- und zuklappen, Zustand merken | `main.rs` (`BaumKlapp`), `vc-ide-model` (`treestate`) | Dateibaum | E1.1 | übernommen |
| Alle Ordner zuklappen | `main.rs` (`BaumAlleZu`) | Dateibaum | E1.1 | übernommen |
| Baum neu einlesen | `main.rs` (`BaumNeuLaden`) | Dateibaum | E1.1 | übernommen |
| Dateien filtern (Treffer mit ihren Ordnern) | `main.rs` (`BaumFilter`), `project.rs` | Dateibaum | E1.1 | übernommen |
| Tastatur im Baum (Pfeile, Enter, F2, Entf) | `main.rs` (`baum_taste`) | Dateibaum | E1.1 | übernommen |
| Exportierte Dateien im Baum markieren | `project.rs` (`exportierte`) | Dateibaum | E2 | offen |
| Dateiart-Symbole und Gruppenüberschriften | `project.rs` (`gruppiere`) | Dateibaum | E1.1 | offen |
| Neue Datei (Art Assembler, BASIC, Text, Notiz) im gewählten Ordner | `main.rs` (`BaumNeu`, `NeuArt`) | `neue_datei` | E1.1 | übernommen |
| Neue Datei der Grafik-, Musik- und Datenarten mit Profil (Sprite, Zeichensatz, Kacheln, Bildschirm, Song, Palette) | `main.rs` (`NeuArt`, `BaumProfil`) | `neue_datei` | E3 | teilweise (Sprites und Player/Missile in E3.4; die übrigen Arten mit ihren Editoren) |
| Neue leere Diskette (D64, D71, D81, ATR) | `main.rs` (`NeuArt::Diskette`) | `neue_datei` | E2 | offen |
| Neuer Ordner | `main.rs` (`BaumOrdner`) | `neuer_ordner` | E1.1 | übernommen |
| Neue Notiz über den Baum | `main.rs` | `neue_notiz_dialog` | E1.1 | übernommen |
| Umbenennen (F2) | `main.rs` (`BaumUmbenennen`) | `datei_umbenennen` | E1.1 | übernommen |
| Löschen in den Papierkorb (Entf), endgültig mit Rückfrage | `main.rs` (`BaumLoeschen`), `fileops.rs` | `datei_loeschen` | E1.1 | übernommen |
| Baumaktionen rückgängig machen | `main.rs` (`baum_rueckgaengig`) | `baum_rueckgaengig` | E1.1 | übernommen |
| Dateien im Baum per Ziehen in andere Ordner verschieben | `main.rs` (`BaumZugStart`, `baum_verschieben`) | Dateibaum | E1.1 | übernommen |
| Verschieben passt Pfade in Quelltexten an (Include, Projektdatei) | `main.rs` (`datei_verschoben`), `fileops.rs` | Dateibaum | E1.1 | übernommen |
| Pfad kopieren | `main.rs` (`PfadKopieren`) | `pfad_kopieren` | E1.1 | übernommen |
| Im Ordner zeigen (Finder, Explorer) | `aussen.rs` (`ImOrdnerZeigen`) | `im_ordner_zeigen` | E1.1 | übernommen |
| Hex-Ansicht für eine Datei öffnen | `main.rs` (`HexAnsicht`) | `hex_ansicht` | E11 | übernommen (E11) |
| Ziel öffnen aus dem Kontextmenü | `main.rs` (`ZielOeffnen`) | `ziel_oeffnen` | E1.1 | übernommen |
| Projektsuche (Cmd+Umschalt+F) über alle Dateien im Hintergrund | `project.rs` (`suche`), `vc-ide-model` (`projsearch`) | Dateibaum ▸ Suche | E1.2 | übernommen |
| Projektsuche berücksichtigt ungespeicherte Änderungen | `project.rs` (`suche`) | Dateibaum ▸ Suche | E1.2 | übernommen |
| Treffer anklicken springt zur Zeile | `main.rs` (`GotoFund`) | Dateibaum ▸ Suche | E1.2 | übernommen |
| Seitenleiste auf Streifen minimieren, Symbole für Dateien und Suche | `main.rs` (`spalten_streifen`, `SpalteAuf`) | Seitenleiste | E1.4 | übernommen |
| Seitenleiste ein- und ausblenden (Cmd+B) | `main.rs` | `seitenleiste` | E0 | offen |

## 10. Sprachdienst (LSP) (E1.2)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Diagnosen: Fehler und Warnungen als Markierung in der Zeile | `lsp.rs`, `main.rs` (`lsp_antwort`), `core.rs` (`errors`, `warns`) | Code: Editor | E1.2 | übernommen |
| Hinweise (Schweregrad 3) als dezente Markierung | `core.rs` (`hints`) | Code: Editor | E1.2 | übernommen |
| Diagnosen in „Probleme“ gesammelt | `main.rs` (`probleme`) | Schublade Probleme | E1.3 | offen |
| Hover: Text zur Stelle unter der Maus nach Verzögerung | `lsp.rs` (`Hover`), `widget.rs`, `main.rs` (`hover_karte`) | Code: Editor | E1.2 | übernommen |
| Hover auf Fehlerzeile zeigt die Meldung sofort | `main.rs` (`ed_nachricht`) | Code: Editor | E1.2 | übernommen |
| Vervollständigung des Sprachdienstes mit Beschreibung | `lsp.rs` (`Complete`), `core.rs` (`lsp_items`) | Code: Editor | E1.2 | übernommen |
| Vervollständigung aus eingebautem Wortschatz (Mnemonics, Direktiven) | `highlight.rs`, `core.rs` (`update_completion`) | Code: Editor | E1.2 | übernommen |
| Vervollständigung manuell auslösen | `main.rs` (`Vervollstaendigen`) | `vervollstaendigen` | E1.2 | übernommen |
| Vervollständigung: Pfeile wählen, Enter oder Tab übernimmt, Esc schließt | `widget.rs` | Code: Editor | E1.2 | übernommen |
| Definition suchen (Gehe zu Definition, auch in anderer Datei) | `lsp.rs` (`Definition`), `main.rs` | `gehe_zu_definition` | E1.2 | übernommen |
| Hinweis „Definition außerhalb des Projekts“ | `main.rs` | Code: Editor | E1.2 | offen (Meldung „Keine Definition gefunden“, ohne eigenen Hinweis für Dateien außerhalb) |
| Gliederung des Dokuments (Marken) im Panel | `main.rs` (`panel_gliederung`, `gliederung_holen`) | Panel Gliederung | E1.2 | übernommen |
| Sprachdienst-Start und Abgleich je geöffneter Datei (Open, Change, Close) | `lsp.rs`, `main.rs` (`lsp_abgleich`) | Dienst im Kern | E1.2 | übernommen |
| Diagnosen ein- und ausschalten | `main.rs` (`SetDiagnose`), `einstellungen.rs` | Einstellungen ▸ Editor | E1.2 | offen |

## 11. Bauen, Starten, Probleme, Protokoll, Schublade, Statusleiste, Tests (E1.3)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Bauen (F7) | `main.rs` (`bauen`, `bau_ende`) | `bauen` | E1.3 | übernommen |
| Starten (F5): bauen und im Ziel starten | `main.rs` (`starte`) | `starten` | E1.3 | übernommen |
| Debuggen (F6): bauen und Debug-Sitzung | `main.rs` (`debug_start`) | `debuggen` | E1.3 | offen (E6, Debug-Sitzung; Menüeintrag meldet das) |
| Bauen, Starten, Debuggen der aktiven Datei (Alt+Umschalt+F7, F5, F6) | `main.rs` (`bauen_wahl`) | `bauen_aktiv`, `starten_aktiv`, `debuggen_aktiv` | E1.3 | übernommen |
| Bau läuft: Anzeige „baut“ und Knopf gesperrt | `main.rs` (`baut`) | Kopfzeile | E1.3 | übernommen |
| Bauergebnis: Fehler und Warnungen in Probleme, Größe und Segmente | `main.rs` (`bau_ende`, `BauInfo`) | Schublade Probleme | E1.3 | übernommen |
| Hauptquelle des Projekts wählen (ohne offenen Quelltext) | `main.rs` (`hauptquelle`) | Bauen | E1.3 | übernommen |
| Hinweise „nichts zu starten, erst bauen“ | `main.rs` | Kopfzeile | E1.3 | übernommen |
| Zielwahl-Chip (Emulator oder Hardware, Startziel F5) | `main.rs` (`ziel_chip`, `ziel_eintraege`) | Kopfzeile Zielwahl | E1.3 | übernommen |
| Kopfzeile in Breitenstufen (voll, Symbole, Lupe, Mehr-Menü) | `main.rs` (`kopf_stufe`, `kopfzeile`) | Kopfzeile | E1.3 | offen |
| Schublade ein- und ausblenden (Cmd+J), minimieren, maximieren | `main.rs`, `bereich.rs` | `schublade`, `schublade_max` | E1.3 | übernommen (Ein-/Ausblenden ⌘J; Minimieren/Maximieren offen) |
| Schublade: Reiter Protokoll | `main.rs` (`schublade_ansicht`) | Schublade Protokoll | E1.3 | übernommen |
| Protokoll kopieren und leeren | `main.rs` | `protokoll_kopieren`, `protokoll_leeren` | E1.3 | übernommen |
| Schublade: Reiter Probleme mit Zähler | `main.rs` | Schublade Probleme | E1.3 | übernommen |
| Problem anklicken springt zur Quelle | `main.rs` (`GotoProblem`) | `problem_springen` | E1.3 | übernommen |
| Problem kopieren, alle Probleme kopieren | `main.rs` | `problem_kopieren`, `probleme_alle_kopieren` | E1.3 | übernommen |
| Nächstes und voriges Problem (F8, Umschalt+F8) | `main.rs` | `naechstes_problem`, `voriges_problem` | E1.3 | übernommen |
| Schublade: Reiter Tests mit Statustext | `main.rs` (`tests_ansicht`) | Schublade Tests | E1.3 | übernommen |
| Tests ausführen (Alt+F8), alle oder nur fehlgeschlagene | `main.rs` (`tests_starten`), `testlauf.rs` | `tests_ausfuehren` | E1.3 | übernommen |
| Tests: Backend wählen (Emulator oder statisch), Abbrechen | `main.rs` (`TestBackend`, `TestStopp`) | Schublade Tests | E1.3 | übernommen |
| Tests: Filter und „nur Fehler“ | `main.rs` (`TestFilter`, `TestNurFehler`) | Schublade Tests | E1.3 | übernommen |
| Tests: Liste nach Gruppen mit Zuständen, Dauer, gemessener Wert | `testlauf.rs` | Schublade Tests | E1.3 | übernommen |
| Tests: Detail mit Erwartung, Register und Zyklen, Setup-Zeile | `main.rs`, `testlauf.rs` (`setup_zeile`) | Schublade Tests | E1.3 | übernommen bis auf die Setup-Zeile (offen) |
| Test: zur Quelle springen | `main.rs` (`TestQuelle`) | Schublade Tests | E1.3 | übernommen |
| Test im Debugger öffnen | `main.rs` (`TestDebug`, `test_debug_start`) | Schublade Tests, Debug | E6 | übernommen (E6-Rest: Knopf „Diesen Test debuggen“ im Tests-Panel; Lauf mit echtem Emulator nur als Handpunkt) |
| Tests laufen parallel zur Debug-Sitzung (Hinweis) | `main.rs` | Schublade Tests | E6 | offen |
| Statusleiste: Hinweis, „bereit“, Cursor Zeile und Spalte | `main.rs` (`view`) | Statusleiste | E1.3 | übernommen |
| Statusleiste: Zielsystem und Takt, Norm, Zeilen, Zyklen (Option) | `telemetrie.rs`, `main.rs` (`takt_anzeige`) | Statusleiste | E1.3 | übernommen: Größe und Speicherstreifen der Segmente, Zyklen je Zeile im Rand; Takt und Norm offen (E6) |
| Statusleiste: PC bei laufender Debug-Sitzung | `main.rs` (`takt_anzeige`) | Statusleiste | E6 | übernommen (Statusleiste: „Debug: angehalten bei $PC“) |
| Panel ein- und ausblenden (Cmd+Alt+B), minimieren, maximieren | `main.rs` (`panel`), `bereich.rs` | `panel`, `panel_max` | E1.3 | übernommen |
| Panel: Reiter Gliederung, Hilfe, Zeichen, Karte, Emulator | `main.rs` (`PanelSeite`) | Panel (Maschine-Panel) | E1.3 | offen (Panel zeigt „Maschine“; Gliederung, Suche, Probleme, Ausgabe und Tests stehen in der Ablage unter dem Editor) |
| Panel-Breite je Inhalt, Panel überlagert bei schmalem Fenster | `main.rs` (`panel_breite`, `schmal`) | Panel | E1.3 | offen |
| Export beim Bauen (Medien automatisch exportieren) | `setui.rs` (Export beim Bauen), `main.rs` | Einstellungen ▸ Projekt | E12 | offen |
| Bauen mit Fehlerhinweis „keine Quelldatei zum Bauen“ | `main.rs` | Bauen | E1.3 | übernommen |
| Maschine-Panel: Livebild der Maschine, Register und Flags (Bildstrom `vc_emu::framestream`) | `main.rs` (`PanelSeite::Emulator`) | Panel Maschine | E6 | übernommen (Maschine-Panel: Register, Flags und Einzelbild der Debug-Sitzung; kein Dauerstrom) |
| „Ab hier starten“ (Debug „Bis hierher“) | `main.rs` (`DebugHierher`, `run_to`) | `debug_hierher` | E6 | offen (von E1.3 nach E6 verschoben) |
| Debuggen (F6) und Debug-Sitzung aus der Kopfzeile | `main.rs` (`debug_start`) | `debuggen` | E6 | offen (siehe Zeile Debuggen oben) |

## 12. Rahmen: Startseite, Projekt, Sitzung, Beenden (E1.4)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Projekt öffnen: Zuletzt-Liste | `startui.rs`, `vc-ide-model` (`recent`) | Startansicht | E0 | übernommen |
| Projektordner öffnen (Ordnerdialog) | `main.rs` (`ProjektOrdnerOeffnen`) | `projekt_oeffnen` | E0 | übernommen |
| Projekt über Projektdatei (.vcprj) wählen | `main.rs` (`ProjektOeffnen`), `project.rs` (`ziel_fuer`) | `projekt_oeffnen` | E1.4 | übernommen |
| Zuletzt geöffnete Projekte anheften und entfernen | `startui.rs` (`Pin`, `Weg`) | Startansicht | E1.4 | übernommen |
| Zuletzt-Liste: Projekt nicht gefunden, Ordner fehlt (Hinweis) | `startui.rs` (`Probe`) | Startansicht | E1.4 | übernommen |
| Startseite mit Schnellreferenz und Überblick | `startui.rs` | Startansicht | E1.4 | teilweise (Tastenkürzel; Hilfeartikel folgen mit E12) |
| Startseite: Beispielprojekte öffnen oder kopieren | `startui.rs` (`Beispiele`, `Kopieren`) | Startansicht | E1.4 | übernommen |
| Startseite: neue Datei oder Editor-Art direkt anlegen | `startui.rs` (`StartAkt`, Neu-Knöpfe je Editor-Art) | Startansicht | E1.4 | zurückgestellt (die Editor-Arten folgen mit E3 bis E5) |
| Startseite: erkannte Emulatoren mit Probe (bereit, nicht gefunden, ungeprüft) | `startui.rs` (`Testen`, `ProbeErg`) | Startansicht | E1.4 | teilweise (erkannt; Probe echter Hardware folgt mit E12) |
| Startseite: Hardware- und Toolchain-Übersicht | `startui.rs` | Startansicht | E1.4 | teilweise (Emulatoren; Hardware folgt mit E12) |
| Startseite: Banner „Einrichtung unvollständig“ und Angebot | `startui.rs` (`Hinweis`, `BannerWeg`) | Startansicht | E1.4 | zurückgestellt (mit dem Assistenten in E12) |
| Startseite: Feedback senden, Assistent öffnen | `startui.rs` | Startansicht | E1.4 | zurückgestellt (E12) |
| Startseite wieder öffnen (Befehl) | `main.rs` (`Startseite`) | `startseite` | E0 | offen |
| Neues Projekt (Cmd+Alt+N): Zielsystem, Vorlage, Sprache Assembler oder BASIC | `neuprojekt.rs` | Dialog Neues Projekt | E1.4 | übernommen |
| Neues Projekt: Vorlagenkarten aus `data/templates/index.json` | `neuprojekt.rs` (`Karte`) | Dialog Neues Projekt | E1.4 | übernommen |
| Neues Projekt: Name prüfen, Speicherort wählen | `neuprojekt.rs` (`Name`, `Ort`) | Dialog Neues Projekt | E1.4 | übernommen |
| Neues Projekt: BASIC-Dialekt, Videonorm, Speicherausbau | `neuprojekt.rs` | Dialog Neues Projekt | E1.4 | übernommen |
| Neues Projekt: Emulator und Startziel F5 | `neuprojekt.rs` (`Emu`, `Start`) | Dialog Neues Projekt | E1.4 | übernommen |
| Neues Projekt: Diskette, Zeichensatz des Projekts, Dateistruktur | `neuprojekt.rs` (`Disk`, `Zeichensatz`, `Struktur`) | Dialog Neues Projekt | E1.4 | übernommen |
| Neues Projekt: README anlegen, Git-Repository anlegen | `neuprojekt.rs` (`Readme`, `Git`) | Dialog Neues Projekt | E1.4 | übernommen |
| Neues Projekt: weitere Zielsysteme zusätzlich | `neuprojekt.rs` (`Extra`) | Dialog Neues Projekt | E1.4 | übernommen |
| Projekt anlegen und öffnen, Fehlermeldung bei Git-Fehler | `neuprojekt.rs` (`Ok`) | Dialog Neues Projekt | E1.4 | übernommen |
| Projekt wechseln mit Rückfrage bei ungespeicherten Änderungen | `main.rs` (`wechsel_noetig`, `Folge::Projekt`) | Rahmen | E1.4 | übernommen |
| Projekt wechseln gesperrt während Debug-Sitzung | `main.rs` | Rahmen | E1.4 | E6 (Debug-Sitzung) |
| Sitzung wiederherstellen: zuletzt offene Reiter und Reihenfolge | `main.rs` (`offene_dateien`), `vc-ide-model` (`session`) | Rahmen | E1.4 | übernommen |
| Sitzung je Projekt abschaltbar („beim Öffnen die zuletzt geöffneten Dateien“) | `projeinst.rs` (`RestoreTabs`) | Projekteinstellungen | E1.4 | teilweise (Schalter in der .vcprj; Dialog folgt mit E12) |
| Layout speichern (Seitenleiste, Schublade, Panel, Höhen, Panel-Reiter) | `main.rs` (`layout_merken`), `vc-ide-model` (`layout`) | Rahmen | E1.4 | übernommen |
| Layout zurücksetzen | `main.rs` (`LayoutZuruecksetzen`) | `layout_zuruecksetzen` | E1.4 | übernommen |
| Bereiche minimieren, maximieren, wiederherstellen (Kopfzeile, Doppelklick) | `bereich.rs`, `main.rs` (`BereichAkt`) | Bereichskopf | E1.4 | übernommen |
| Spalte, Dokument, Schublade, Panel maximieren (Befehle) | `main.rs` | `spalte_max`, `dokument_max`, `schublade_max`, `panel_max` | E1.4 | übernommen |
| Trennlinie zwischen Dokumentenfeld und Debug-Fläche verstellen und merken | `main.rs` (`TrennerStart`, `debug_hoehe`) | Rahmen | E6 | entfällt (Debug ist ein eigener Bereich, kein geteiltes Fenster) |
| Breitenklassen der Ansichten (breit, mittel, schmal) | `muster.rs` (`klasse`) | Layout aller Ansichten | E1.4 | übernommen |
| Beenden (Cmd+Q) mit Liste ungespeicherter Änderungen: Speichern, Verwerfen, Abbrechen | `main.rs` (`beenden`, `beenden_dialog`) | `beenden` | E1.4 | übernommen |
| Fenster schließen wie Beenden | `main.rs` (`FensterZu`) | Rahmen | E1.4 | übernommen |
| Absturzsicherung: unbenannte Reiter in Zwischenkopie sichern | `main.rs` (`scratch_sichern`), `vc-ide-model` (`scratchsave`) | Rahmen | E1.4 | übernommen |
| Wiederherstellen unbenannter Reiter beim nächsten Start (Dialog) | `main.rs` (`scratch_dialog`, `scratch_wiederherstellen`) | Rahmen | E1.4 | übernommen |
| Absturzsicherung ein- und ausschalten | `setui.rs` (`ScratchSicherung`) | Einstellungen ▸ Allgemein | E1.4 | teilweise (Kernbefehl; Schalter folgt mit E12) |
| Unbenannten Reiter verwerfen mit Rückfrage | `main.rs` (`scratch_verwerfen`) | Rahmen | E1.4 | übernommen |
| Ältere Kachelkarte beim Öffnen umwandeln (mit Sicherung) | `main.rs` (`kachelkarte_umwandeln`) | Kachel-Editor | E8 | offen |
| Rückfragen (Debug-Sitzung beenden, Einstellungen zurücksetzen, Konfiguration importieren) | `main.rs` (`Rueckfrage`) | Dialoge | E1.4 | E12 (Einstellungen) |

## 13. Disk-Editor (E2)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Diskettenabbild öffnen (D64, D71, D81, ATR, Boot-Diskette ohne Dateisystem) | `diskstudio.rs` | Daten: Disk | E2 | übernommen (E2.1–E2.4) |
| Belegungskarte der Spuren und Sektoren (BAM bzw. VTOC) | `diskstudio.rs` (`KartenGeo`) | Daten: Disk | E2 | übernommen (E2.1–E2.4) |
| Große Disketten (ab 41 Spuren) in zwei Spurblöcken | `diskstudio.rs` (`KartenGeo`) | Daten: Disk | E2 | übernommen (E2.1–E2.4) |
| Verzeichnis mit Dateiname, Typ, Größe, Start, Blöcke | `diskstudio.rs` | Daten: Disk | E2 | übernommen (E2.1–E2.4) |
| Verzeichnisnamen in PETSCII oder ATASCII anzeigen | `diskstudio.rs` | Daten: Disk | E2 | übernommen (E2.1–E2.4) |
| Datei im Verzeichnis wählen: ihre Sektorkette in der Karte | `diskstudio.rs` (`Datei`) | Daten: Disk | E2 | übernommen (E2.1–E2.4) |
| Sektor wählen: Inhalt als Hex und Zeichen | `diskstudio.rs` (`Sektor`) | Daten: Disk | E2 | übernommen (E2.1–E2.4) |
| Sektorkette folgen (Folgesektor) | `diskstudio.rs` (`Folgen`) | Aktion Sektorkette folgen | E2 | übernommen (E2.1–E2.4) |
| Zurück zum vorigen Sektor (Verlauf) | `diskstudio.rs` (`Zurueck`) | Aktion Zurück | E2 | übernommen (E2.1–E2.4) |
| Kette anzeigen oder ausblenden (Start, Ende, Richtung, Interleave) | `diskstudio.rs` (`KetteAn`) | Daten: Disk | E2 | übernommen (E2.1–E2.4) |
| Validieren (BAM, Verzeichnis, Dateiketten), Bericht mit Problemen | `diskstudio.rs` (`Validieren`, `ReportZu`) | Aktion Validieren | E2 | übernommen (E2.1–E2.4) |
| Haltepunkt auf Sektoradresse, Sprung in Debug | `main.rs` (`DisHaltepunkt`) | Daten: Disk | E6 | offen |
| Dateien vormerken (einzeln, alle, keine, nur Programme, nur BASIC) | `diskstudio.rs` (`Vormerken`, `Alle`, `Keine`, `Programme`, `Basic`) | Daten: Disk | E2 | übernommen (E2.1–E2.4) |
| Vorgemerkte Dateien in das Projekt übernehmen (Zielordner) | `diskstudio.rs` (`Uebernehmen`, `Zielordner`) | Daten: Disk | E2 | übernommen (E2.1–E2.4) |
| Übernehmen: BASIC als Quelltext, auch als Assembler, rohe Bytes | `diskstudio.rs` (`OptBasic`, `OptAsm`, `OptRaw`) | Daten: Disk | E2 | übernommen (E2.1–E2.4) |
| Datei hinzufügen (Dateisystem-Diskette) | `diskstudio.rs` (`Hinzufuegen`, `DiskDateien`) | Aktion Datei hinzufügen | E2 | übernommen (E2.1–E2.4) |
| Image speichern, Speichern unter (als Kopie) | `diskstudio.rs` (`Speichern`, `SpeichernUnter`) | Daten: Disk | E2 | übernommen (E2.1–E2.4) |
| Schreibschutz und „Image ungespeichert“ | `diskstudio.rs` | Daten: Disk | E2 | teilweise („Image ungespeichert“ ja, Schreibschutz der Datei nicht) |
| Diskette einlegen (Standard-Emulator, Laufwerk 8) | `diskstudio.rs` (`Einlegen`) | Aktion Einlegen | E2 | übernommen (E2.1–E2.4) |
| Diskette auf MEGA65 übertragen (Ethernet) | `diskstudio.rs` (`Mega65`) | Daten: Disk | E2 | zurückgestellt (Hardwareprüfung, Inhaber entscheidet) |
| Erst speichern, dann einlegen oder übertragen (Sperrhinweis) | `diskstudio.rs` | Daten: Disk | E2 | teilweise (Einlegen ja, MEGA65-Übertragung zurückgestellt) |
| Diskette in Projekteinstellungen: Gruppen und Programme der Diskette | `projeinst.rs` (`PeReiter::Diskette`) | Projekteinstellungen | E12 | offen |

## 14. Sprite-Editor (E3)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Sprite-Datei öffnen (Sprites C64, C128, MEGA65, Atari PMG) | `studio.rs` (Öffnen), `studios.rs` | Grafik: Sprites | E3 | übernommen (E3.1) |
| Zeichenfläche mit Raster der Pixel, Zellmarke und Achsenmarken | `studio.rs` (`Flaeche`) | Grafik: Sprites | E3 | übernommen (E3.1) |
| Werkzeug Stift (D), rechte Taste löscht | `studio.rs` (`Tool::Pen`) | Werkzeug Stift | E3 | übernommen (E3.1) |
| Werkzeug Radierer (E) | `studio.rs` (`Tool::Erase`) | Werkzeug Radierer | E3 | übernommen (E3.1) |
| Werkzeug Füllen (F) | `studio.rs` (`Tool::Fill`) | Werkzeug Füllen | E3 | übernommen (E3.2) |
| Werkzeug Linie (L) | `studio.rs` (`Tool::Line`) | Werkzeug Linie | E3 | übernommen (E3.2) |
| Werkzeug Rahmen (R) | `studio.rs` (`Tool::Rect`) | Werkzeug Rahmen | E3 | übernommen (E3.2) |
| Werkzeug Auswahl (S), Rechtsklick öffnet Menü | `studio.rs` (`Tool::Select`) | Werkzeug Auswahl | E3 | übernommen (E3.2) |
| Werkzeug Pipette (I) | `studio.rs` (`Tool::Pick`) | Werkzeug Pipette | E3 | übernommen (E3.1) |
| Farbwert 1 bis 4 mit den Zifferntasten wählen | `studio.rs` (Tastenbehandlung der Zeichenfläche) | Sprite: Farbwahl | E3 | übernommen (E3.1) |
| Farbwahl: Palette je Zielsystem und Videonorm | `studio.rs` (`Farben`, `FarbSlot`, `Farbe`) | Sprite: Farbwahl | E3 | übernommen (E3.1) |
| Ansicht PAL, NTSC oder Projekt-Norm der Palette | `studio.rs` (`Ansicht`, `ansicht_wahl`) | Sprite: Ansicht | E3 | übernommen (E3.1) |
| Rückgängig und Wiederholen | `studio.rs` (`Undo`, `Redo`) | `rueckgaengig`, `wiederholen` | E3 | übernommen (E3.1) |
| Horizontal spiegeln (X), vertikal spiegeln (Y) | `studio.rs` (`Spiegeln`) | Aktion Spiegeln | E3 | übernommen (E3.2) |
| Frame verschieben nach links, rechts, oben, unten | `studio.rs` (`Verschieben`) | Aktion Verschieben | E3 | übernommen (E3.2) |
| Frame leeren | `studio.rs` (`Leeren`) | Aktion Leeren | E3 | übernommen (E3.2) |
| Zoom plus und minus, Einpassen (0), Zoom per Cmd+Mausrad | `studio.rs`, `flaeche.rs` | Befehle für Editor-Zoom (Ansicht, Untermenü der Grafik-Editoren) | E3 | übernommen (E3.1) |
| Verschieben der Fläche mit Leertaste und Ziehen | `flaeche.rs` (`Gesten`) | Grafik-Editoren | E3 | übernommen (E3.1) |
| Raster ein- und ausschalten | `studio.rs` (`Raster`) | Editor-Raster | E3 | übernommen (E3.1) |
| Onion Skin: vorheriges Frame durchscheinen lassen | `studio.rs` (Schalter der Vorschau) | Onion Skin | E3 | übernommen (E3.3) |
| Frames: Liste mit Vorschau, anhängen, duplizieren, löschen | `studio.rs` (`FrameNeu`, `FrameDup`, `FrameLoeschen`, `Frame`) | Sprite: Frame-Liste | E3 | übernommen (E3.3) |
| Frame wechseln mit Pfeil links und rechts | `studio.rs` (Tastenbehandlung der Zeichenfläche) | Sprite: Frame-Liste | E3 | übernommen (E3.3; Alt+Pfeil und Bild-auf/ab, weil die Pfeiltasten die Zellmarke bewegen) |
| Modus Hires oder Multicolor je Frame (C) | `studio.rs` (`Modus`) | Sprite: Modus | E3 | übernommen (E3.3) |
| Hardware-Flags: doppelt breit, doppelt hoch, hinter Hintergrund, Multicolor | `studio.rs` (`Flag`), `vc-ide-model` (`pixel`) | Sprite: Hardware-Flags | E3 | übernommen (E3.3; C64, C128 und MEGA65 im VIC-II-Modus; weitere Eigenschaften nach Entscheidung des Inhabers) |
| Animationen: neu, löschen, wählen, benennen | `studio.rs` (`Anim`, `AnimWahl`) | Sprite: Animationen | E3 | übernommen (E3.4) |
| Animation: Schritte anhängen, entfernen, verschieben, Dauer je Schritt | `studio.rs` (`AnimOp`) | Sprite: Animationen | E3 | übernommen (E3.4; Bild je Schritt nicht änderbar, wie in der alten Oberfläche) |
| Animation: Ablauf (Schleife, Hin und zurück, einmal) und Vorschau | `studio.rs` (`AnimOp::Mode`, Takt der Vorschau) | Sprite: Animationen | E3 | übernommen (E3.4; Zeitleiste unter der Fläche) |
| Vorlage laden (Sprite, Zeichensatz) | `studio.rs` (`Vorlage`) | Sprite: Vorlagen | E3 | entfällt für Sprites (Vorlagen gibt es nur für Zeichensätze, E4) |
| PNG laden in das gewählte Frame | `studio.rs` (`PngLaden`, `png_laden`) | Sprite: PNG laden | E3 | übernommen (E3.4) |
| Auswahl kopieren, ausschneiden, einfügen, alles, löschen, aufheben | `studio.rs` (`Kopieren`, `Ausschneiden`, `Einfuegen`, `AuswahlAlles`) | Befehle Zwischenablage | E3 | übernommen (E3.2) |
| Speichern der Mediendatei (Cmd+S) | `studio.rs` (`Speichern`) | `speichern` | E3 | übernommen (E3.1) |
| Exportieren (Cmd+Umschalt+E) | `studio.rs`, `exportdlg.rs` | `exportieren` | E3 | übernommen (E3.4; zusätzlich „In Quelltext kopieren“ ohne Include und BASIC DATA mit Einleseroutine) |
| Einzeltasten der Editoren ein- und ausschalten | `einstellungen.rs` (Schalter für Einzeltasten) | Einstellungen ▸ Tastenkürzel | E12 | übernommen (Schalter „Einzeltasten der Editoren“: Tracker-Tasten, Bildschirm-Werkzeuge, Kacheln-Pipette) |
| Statuszeile: Frame n von m, Maße, Modus, Zelle x/y | `studio.rs` | Statuszeile Sprite | E3 | übernommen (E3.1; in der Fußzeile des Editors) |
| Karten der Seitenspalte ein- und zuklappen, Weitere Aktionen (⋯) | `studio.rs` (`Karte`, `Mehr`), `muster.rs` | Editor-Muster | E3 | entfällt (die Karten der Seitenspalte stehen offen, die Spalte scrollt) |

| MEGA65 VIC-IV-Sprites: 16 oder 64 Punkte breit, 16-Farben-Modus (`SPR16EN`, `SPRX64EN`) | neu (in der Iced-Oberfläche nicht vorhanden) | Grafik: Sprites | nach E3 (eigenes Paket) | vorgemerkt (Entscheidung des Inhabers: nicht in E3; braucht neue Profile, Export, Vorlagen, Beispiele) |
| Atari Player/Missile: Breite je Spieler (`SIZEP0–3`) und Priorität (`PRIOR`), Vorschau skaliert, Export schreibt die Werte | neu (in der Iced-Oberfläche nicht vorhanden) | Sprite: Player/Missile | E3 | übernommen (E3.4; Kernänderung additiv: `hw.sizep`, `hw.prior`, Konstanten `…_sizepN`, `…_prior`) |

## 15. Zeichensatz-Editor (E4)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Zeichensatz-Datei öffnen (256 Zeichen, Raster 16 × 16) | `studio.rs` (`kind == "charset"`) | Grafik: Zeichensatz | E4 | offen |
| Zeichen bearbeiten mit allen Werkzeugen des Pixel-Editors | `studio.rs` | Grafik: Zeichensatz | E4 | offen |
| Zeichenwahl im Raster mit Code in Dezimal und Hex | `studio.rs` | Grafik: Zeichensatz | E4 | offen |
| Herkunft des Zeichensatzes anzeigen (ROM-Verweis, eigener Satz) | `studio.rs` (`herkunft`) | Grafik: Zeichensatz | E4 | offen |
| Hinweis „ROM fehlt“ bei Verweis ohne vorhandenes ROM | `studio.rs` | Grafik: Zeichensatz | E4 | offen |
| Zeichensatz-Vorlage laden (rückgängig machbar) | `studio.rs` (`Vorlage`) | Grafik: Zeichensatz | E4 | offen |
| Standardzeichensatz hinzufügen (VC-Standard, eigene Sätze CC0) | `zeichendlg.rs` (`ZTab::Standard`) | `std_zeichensatz` | E4 | offen |
| Zeichensatz aus ROM lesen (Verweis oder einbetten, Profil) | `zeichendlg.rs` (`ZTab::Rom`) | `zeichensatz_rom` | E4 | offen |
| Zeichensatz aus PNG-Raster (16 Zeichen je Zeile, Invertierung) | `zeichendlg.rs` (`ZTab::Png`) | `zeichensatz_png` | E4 | offen |
| Zeichensatzprofil und Name prüfen | `zeichendlg.rs` (`Profil`, `Name`) | Dialog Zeichensatz | E4 | offen |
| ROM-Datei erkannt oder nicht erkannt (Vorschläge nach Dateigröße) | `zeichendlg.rs` | Dialog Zeichensatz | E4 | offen |
| Zeichensatz im Debug-Fenster anzeigen (Chip-Fenster) | `dbgview.rs` | Debug | E6 | übernommen (Chip-Fenster: Register der gewählten Chips mit Werten im Halt) |

## 16. Musik-Editor, Tracker und Audio (E5)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Song-Datei öffnen (SID, POKEY, MEGA65-Chip, PLAY) | `songstudio.rs` | Musik | E5 | übernommen (E5.1) |
| Ansicht Noten (Cmd+1): Noten eingeben und bearbeiten | `songstudio.rs` (`SongModus::Noten`) | `tracker_noten` | E5 | übernommen (E5.1) |
| Ansicht Instrumente (Cmd+2): Klang und Probespielen | `songstudio.rs` (`SongModus::Instrumente`) | `tracker_instrumente` | E5 | übernommen (E5.1) |
| Ansicht Ablauf (Cmd+3): Reihenfolge und Muster | `songstudio.rs` (`SongModus::Ablauf`) | `tracker_ablauf` | E5 | übernommen (E5.1) |
| Tracker: Zellen bewegen mit Pfeilen | `songstudio.rs` (`song_taste`) | Musik: Tracker | E5 | übernommen (E5.1) |
| Tracker: Auswahl mit Umschalt+Pfeil | `songstudio.rs` (`bewege`) | Musik: Tracker | E5 | übernommen (E5.1) |
| Tracker: Seite auf und ab (8 Zeilen), Pos1 und Ende | `songstudio.rs` | Musik: Tracker | E5 | übernommen (E5.1) |
| Tracker: Maus wählt Zellen und zieht Auswahl | `songstudio.rs` (`GMaus`) | Musik: Tracker | E5 | übernommen (E5.1) |
| Klaviertasten tragen Noten ein (nach physischer Tastenposition) | `songstudio.rs` (`klaviertaste`), `tastatur.rs` | Musik: Tracker | E5 | übernommen (E5.1) |
| Tastaturlayout für die Beschriftung der Klaviertasten (Auto, QWERTZ, QWERTY, AZERTY) | `einstellungen.rs` (`Tastaturlayout`), `setui.rs` | Einstellungen ▸ Tastenkürzel | E12 | entfällt: die Tasten gelten nach ihrer Lage auf der Tastatur (Tastencode), eine Layoutwahl ist nicht nötig; Hinweis auf der Seite |
| Klaviatur-Karte mit Computertasten und klingenden Noten | `songstudio.rs`, `einstellungen.rs` (`tasten_auf_klaviatur`) | Musik: Klaviatur | E5 | übernommen (E5.3) |
| Oktave tiefer und höher (Komma und Punkt) | `songstudio.rs` (`Oktave`) | Musik: Tracker | E5 | übernommen (E5.1) |
| Instrument der Zelle wechseln ([ und ], Alt+Pfeil) | `songstudio.rs` (`zell_instrument_aendern`) | Musik: Tracker | E5 | übernommen (E5.1) |
| Notenende eintragen (1) | `songstudio.rs` (`notenende`) | Musik: Tracker | E5 | übernommen (E5.1) |
| Löschen der Zelle oder Auswahl (Entf, Rücktaste) | `songstudio.rs` (`Loeschen`) | Musik: Tracker | E5 | übernommen (E5.1) |
| Schrittweite nach Eingabe größer und kleiner | `songstudio.rs` (`Schritt`) | Musik: Tracker | E5 | übernommen (E5.1) |
| Zwischenablage: Zellen kopieren, ausschneiden, einfügen | `songstudio.rs` (`Kopieren`, `Ausschneiden`, `Einfuegen`) | Musik: Tracker | E5 | übernommen (E5.1) |
| Block transponieren (Halbton, Oktave) | `songstudio.rs` (`Transponieren`) | Musik: Tracker | E5 | übernommen (E5.1) |
| Gewähltes Instrument auf alle Noten des Blocks anwenden | `songstudio.rs` (`InstrumentAnwenden`) | Musik: Tracker | E5 | übernommen (E5.1) |
| Kanäle einblenden und ausblenden, alle einblenden | `songstudio.rs` (`Sichtbar`, `AlleEinblenden`) | Musik: Kanäle | E5 | übernommen (E5.3) |
| Kanal stumm und Solo | `songstudio.rs` (`Stumm`, `Solo`) | Musik: Kanäle | E5 | übernommen (E5.2) |
| Pegelanzeige je Kanal und Summe | `songstudio.rs` (`pegel`) | Musik: Pegel | E5 | übernommen (E5.2) |
| Wiedergabe Song (Leertaste) und Stopp | `songstudio.rs` (`SpielSong`, `Stopp`), `audio.rs` | Musik: Wiedergabe | E5 | übernommen (E5.2) |
| Wiedergabe nur Muster (Umschalt+Leertaste) | `songstudio.rs` (`SpielMuster`) | Musik: Wiedergabe | E5 | übernommen (E5.2) |
| Wiedergabe ab gewähltem Eintrag der Reihenfolge | `songstudio.rs` | Musik: Wiedergabe | E5 | übernommen (E5.2) |
| Mitscrollen beim Abspielen, gespielte Zeile mittig | `songstudio.rs` (`Mitlaufen`, `SongScroll`) | Musik: Wiedergabe | E5 | übernommen (E5.2) |
| Änderungen im Raster gelten sofort während der Wiedergabe | `songstudio.rs` (`song_tick`, `live_stand`) | Musik: Wiedergabe | E5 | übernommen (E5.2) |
| Lautstärke der Wiedergabe | `songstudio.rs` (`Lautstaerke`) | Musik: Wiedergabe | E5 | übernommen (E5.2) |
| Hinweis „kein Audio-Ausgabegerät gefunden“ | `audio.rs` | Musik: Wiedergabe | E5 | übernommen (E5.2) |
| Tempo (Bilder je Zeile) schneller und langsamer | `songstudio.rs` (`Tempo`) | Musik: Muster | E5 | übernommen (E5.1) |
| Zeilen je Muster 8 mehr und weniger | `songstudio.rs` (`Zeilen`) | Musik: Muster | E5 | übernommen (E5.1) |
| Muster neu, duplizieren, löschen, wählen | `songstudio.rs` (`MusterNeu`, `MusterDupl`, `MusterLoeschen`, `Muster`) | Musik: Ablauf | E5 | übernommen (E5.1) |
| Reihenfolge: Eintrag setzen, anhängen, entfernen, verschieben | `songstudio.rs` (`Ord`, `OrdPlus`, `OrdMinus`, `OrdSetzen`, `OrdVerschieben`) | Musik: Ablauf | E5 | übernommen (E5.1) |
| Wiederholung ab Eintrag (Loop) früher und später | `songstudio.rs` (`Schleife`) | Musik: Ablauf | E5 | übernommen (E5.1) |
| PLAY-Dateien ohne Reihenfolge (Hinweis) | `songstudio.rs` | Musik: Ablauf | E5 | übernommen (E5.1) |
| Instrument: neu, duplizieren, löschen, benennen | `songstudio.rs` (`InsNeu`, `InsDupl`, `InsLoeschen`, `InsName`) | Musik: Instrumente | E5 | übernommen (E5.1) |
| Instrument: Wellenform (Dreieck, Sägezahn, Rechteck, Rauschen) | `songstudio.rs` (`InsWelle`) | Musik: Instrumente | E5 | übernommen (E5.1) |
| Instrument: Pulsbreite und ADSR | `songstudio.rs` (`InsZahl`) | Musik: Instrumente | E5 | übernommen (E5.1) |
| Instrument: POKEY-Verzerrung (reiner Ton, Bass, Buzz, Gritty) | `songstudio.rs` (`InsDist`) | Musik: Instrumente | E5 | übernommen (E5.1) |
| Instrument: Hüllkurve je Bild und Lautstärke je Bild (PLAY) | `songstudio.rs` (`Huell`, `ReglerLos`) | Musik: Instrumente | E5 | übernommen (E5.1) |
| Instrument: Höchstzahl je System, letzte unbenutzte löschbar | `songstudio.rs` | Musik: Instrumente | E5 | übernommen (E5.1) |
| Vorhören der Note mit dem Instrument (Klick, Leertaste im Modus Instrumente) | `songstudio.rs` (`Vorhoeren`, `Probe`) | Musik: Instrumente | E5 | übernommen (E5.3) |
| Song aus anderem System übernehmen (Konvertieren) | `songstudio.rs` (`Konvertieren`, `SongKonv`) | Musik: Konvertieren | E5 | übernommen (E5.3) |
| Import: SID, GoatTracker (SNG) und weitere Songformate | `songstudio.rs`, `vc-ide-model` (`songimport`) | Musik: Import | E5 | übernommen (E5.3) |
| Import: RMT-Modul (Raster Music Tracker, Subtune, Sekunden) | `songstudio.rs` (`RmtImport`, `SongImpSub`) | Musik: Import | E5 | übernommen (E5.3) |
| Rückgängig, Wiederholen, Speichern, Exportieren | `songstudio.rs` (`Undo`, `Redo`, `Speichern`, `Exportieren`) | `speichern`, `exportieren` | E5 | übernommen (E5.1) |
| Spielroutine für den 6502 als Export-Ergebnis | `exportdlg.rs` | Export | E5 | kommt 0.3.1 (Wunsch offen): die bisherige Oberfläche erzeugte keine Routine; der Export liefert die Songdaten (asm, inc, bin, prg, DATA), die Spielroutine liegt dem Beispiel als lib/music.inc bei |

## 17. Debug und Maschine (E6)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Debug-Sitzung starten (Emulator oder Hardware) und beenden | `debug.rs`, `main.rs` (`debug_start`, `debug_ende`) | Debug | E6 | übernommen (E6) |
| Weiter (Alt+F5), Pause (Alt+F6) | `dbgview.rs`, `main.rs` | `debug_weiter`, `debug_pause` | E6 | übernommen (E6) |
| Schritt über (F10), hinein (F11), hinaus (Umschalt+F11) | `main.rs` | `debug_ueber`, `debug_hinein`, `debug_raus` | E6 | übernommen (E6) |
| Debug beenden (Umschalt+F5) | `main.rs` (`debug_ende`) | `debug_ende` | E6 | übernommen (E6) |
| Bis hierher laufen (Cursorzeile) | `main.rs` (`DebugHierher`, `run_to`) | `debug_hierher` | E6 | übernommen (E6) |
| Debug-Leiste mit Symbolen, kompakte Form bei schmalem Fenster | `dbgview.rs` (`debug_leiste`, `debug_leiste_kompakt`) | Debug: Leiste | E6 | übernommen (E6) |
| Statustext der Sitzung (startet, läuft, angehalten bei PC, wartet) | `dbgview.rs` (`debug_status`) | Debug: Status | E6 | übernommen (E6) |
| Debug-Fläche in der Schublade mit verstellbarer Höhe | `main.rs` (`debug_flaeche`) | Debug | E6 | ersetzt: Debug ist ein eigener Bereich (E6) |
| Register (PC, A, X, Y, SP, Flags), Änderungen markiert | `dbgview.rs` (`dbg_register`) | Debug: Register | E6 | übernommen (E6) |
| Zusatzregister je Zielsystem | `debug.rs` (`Snap::extra`) | Debug: Register | E6 | übernommen (E6) |
| Disassembler mit Adressen, Bytes, Marken, Zeilenbezug zum Quelltext | `dbgview.rs` (`dbg_disassembler`), `main.rs` (`BauInfo`) | Debug: Disassembler | E6 | übernommen (E6) |
| Disassembler: Taktzyklen je Befehl, Zusatzzyklen, illegale Opcodes | `debug.rs` (`DisLine`) | Debug: Disassembler | E6 | übernommen (E6) |
| Haltepunkt im Disassembler per Klick | `main.rs` (`DisHaltepunkt`) | Debug: Disassembler | E6 | übernommen (E6) |
| Haltepunkte aus dem Editor in die Sitzung übernehmen | `main.rs` (`gewuenschte_haltepunkte`, `debug_abgleich`) | Debug | E6 | übernommen (E6) |
| Haltepunkte und Wachpunkte (Ausführen, Lesen, Schreiben) mit Bereich | `debug.rs` (`AddPoint`), `dbgview.rs` (`dbg_haltepunkte`) | Debug: Haltepunkte | E6 | übernommen (E6) |
| Bedingung und Ignorierzähler je Punkt, Treffer zählen | `debug.rs` (`PointInfo`) | Debug: Haltepunkte | E6 | übernommen (E6) |
| Punkt löschen | `main.rs` (`PunktDel`) | Debug: Haltepunkte | E6 | übernommen (E6) |
| Hinweis bei fehlerhafter Bedingung | `debug.rs` (`cond_error`) | Debug: Haltepunkte | E6 | übernommen (E6) |
| Speicherabzug (Hex und ASCII, Dezimal), Änderungen markiert | `dbgview.rs` (`dbg_speicher`) | Debug: Speicher | E6 | übernommen (E6) |
| Speicher: Adresse eingeben, Ausdruck, Symbol | `main.rs` (`MemText`, `MemGo`), `debug.rs` | Debug: Speicher | E6 | übernommen (E6) |
| Beobachten: Ausdrücke (Adresse, Register, Symbole, peek) | `dbgview.rs` (`dbg_beobachten`), `debug.rs` | Debug: Beobachten | E6 | übernommen (E6) |
| Beobachten: Eingabe prüfen mit freundlicher Meldung („meintest du …“) | `debug.rs` | Debug: Beobachten | E6 | übernommen (E6) |
| Beobachten: Zahl in Dezimal und Hex, Eintrag löschen | `main.rs` (`BeobAdd`, `BeobDel`) | Debug: Beobachten | E6 | übernommen (E6) |
| Chip-Fenster: Sprites, Bildschirmcode, Zeichensatz im ROM (mehrfarbig) | `dbgview.rs`, `debug.rs` (`Chips`) | Debug: Chips | E6 | bisher nicht angeschlossen; Erkennen in der Momentaufnahme zeigt Bildschirm, Zeichensatz, Sprites (E6) |
| Momentaufnahme: erkennen, was im Speicher liegt | `dbgview.rs` (`dbg_momentaufnahme`), `main.rs` (`SnapErkennen`) | Debug: Momentaufnahme | E6 | übernommen (E6) |
| Momentaufnahme: Name, Sprites, Kacheln, ROM-Anteil wählen, speichern | `main.rs` (`SnapName`, `SnapSprites`, `SnapKacheln`, `SnapRom`, `SnapSpeichern`) | Debug: Momentaufnahme | E6 | übernommen (E6) |
| Emulator-Bild im Panel anzeigen (angehalten oder Zeitlupe) | `main.rs` (`panel_emulator`) | Panel Emulator | E6 | übernommen (E6) |
| Emulator-Bild: aktualisieren, automatisch, live (Zeitlupe) | `main.rs` (`EmuBild`, `EmuAuto`, `EmuLive`) | Panel Emulator | E6 | übernommen (E6) |
| Emulator-Bild: Bildquelle wählen (C128: VIC-II oder VDC) | `main.rs` (`EmuQuelle`) | Panel Emulator | E6 | übernommen (E6) |
| Emulator: Text an die Tastatur des Emulators senden | `main.rs` (`EmuText`, `EmuSenden`) | Panel Emulator | E6 | übernommen (E6) |
| Fehlermeldungen der Verbindung (Zeitüberschreitung, Verbindung verloren) freundlich | `debug.rs` (`freundlich`) | Debug | E6 | übernommen (E6) |
| Debuggen auf echter Hardware nur über USB (Hinweis) | `main.rs` | Debug | E6 | übernommen (E6) |
| Debuggen nur mit Fähigkeiten des Adapters (Register, Schritt, Haltepunkte, Wachpunkte) | `debug.rs` (`CapsInfo`) | Debug | E6 | übernommen (E6) |
| Zielwahl Emulator oder Hardware, Start ohne Debug | `main.rs` (`ZielWahl`, `starte`) | Kopfzeile Zielwahl | E1.3 | übernommen |
| Start auf Hardware: MEGA65 per Ethernet, Ultimate, serielle Geräte | `aussen.rs`, `zielui.rs` | Start | E6 | übernommen (E1.3, Maschinen); Debuggen s. Zeile oben |
| Testlauf ohne Emulator (nur statische Prüfungen) | `aussen.rs`, `testlauf.rs` | Schublade Tests | E1.3 | übernommen |

## 18. Palette-Editor (E7)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Palette-Datei öffnen (Paletten der Zielsysteme, z. B. MEGA65, Atari) | `palettestudio.rs`, `studios.rs` | Grafik: Palette | E7 | übernommen (E7) |
| Eintrag wählen, Farbe je Kanal (R, G, B) einstellen | `palettestudio.rs` (`Wahl`, `Kanal`, `Fertig`) | Grafik: Palette | E7 | übernommen (E7) |
| Farbe als Hex-Wert eingeben | `palettestudio.rs` (`Hex`, `HexOk`) | Grafik: Palette | E7 | übernommen (E7) |
| Standardpalette laden | `palettestudio.rs` (`Standard`) | Aktion Standardpalette | E7 | übernommen (E7) |
| Rückgängig und Wiederholen | `palettestudio.rs` (`Undo`, `Redo`) | `rueckgaengig`, `wiederholen` | E7 | übernommen (E7) |
| Speichern und Exportieren | `palettestudio.rs` (`Speichern`, `Exportieren`) | `speichern`, `exportieren` | E7 | übernommen (E7) |
| Anzeige „n Farben“ und Eintragsnummer | `palettestudio.rs` | Grafik: Palette | E7 | übernommen (E7) |
| „Im Einsatz“: Dateien, die den Farbindex verwenden (neu) | – | Grafik: Palette, Karte Im Einsatz | E7 | neu (E7) |

## 19. Kachel-Editor (E8)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Kachelsatz öffnen | `tilestudio.rs`, `tilesetdoc.rs` | Grafik: Kacheln | E8 | übernommen (E8) |
| Kachelliste mit Vorschau, Kachel wählen | `tilestudio.rs` (`Kachel`) | Grafik: Kacheln | E8 | übernommen (E8) |
| Kachel neu, duplizieren, löschen (benutzte Kacheln fragen) | `tilestudio.rs` (`Neu`, `Dupl`, `Loeschen`, `FrageJa`) | Grafik: Kacheln | E8 | übernommen (E8) |
| Kacheln zusammenführen (gleich wie Kachel j) | `tilestudio.rs` (`Zusammen`) | Grafik: Kacheln | E8 | übernommen (E8) |
| Kachel benennen | `tilestudio.rs` (`Name`) | Grafik: Kacheln | E8 | übernommen (E8) |
| Kachelgröße (Zellen waagerecht und senkrecht) ändern, Beschneiden warnen | `tilestudio.rs` (`Groesse`) | Grafik: Kacheln | E8 | übernommen (E8) |
| Pinselmodus Zeichen, Farbe oder beides | `tilestudio.rs` (`Modus`) | Grafik: Kacheln | E8 | übernommen (E8) |
| Zeichenwahl aus dem Zielzeichensatz (Zeichenbit 7) | `tilestudio.rs` (`Zeichen`, `Bit7`) | Grafik: Kacheln | E8 | übernommen (E8) |
| Pinselfarbe und Kachelfarbe (Farbe je Zelle oder je Kachel) | `tilestudio.rs` (`Pinselfarbe`, `Kachelfarbe`, `Farbmodus`) | Grafik: Kacheln | E8 | übernommen (E8) |
| Hintergrundfarbe | `tilestudio.rs` (`Bg`) | Grafik: Kacheln | E8 | übernommen (E8) |
| Zeichensatz zum Satz hinzufügen und entfernen | `tilestudio.rs` (`ZeichensatzPlus`, `ZeichensatzMinus`) | Grafik: Kacheln | E8 | übernommen (E8) |
| Füllstand des Zeichensatzes (n von 256 belegt) | `tilestudio.rs` | Grafik: Kacheln | E8 | übernommen (E8) |
| Zellen spiegeln waagerecht und senkrecht | `tilestudio.rs` (`SpiegelH`, `SpiegelV`) | Aktion Spiegeln | E8 | übernommen (E8) |
| Zellen um 90 Grad drehen (nur quadratisch) | `tilestudio.rs` (`Drehen`) | Aktion Drehen | E8 | übernommen (E8) |
| Zellen rollen (links, rechts, oben, unten) | `tilestudio.rs` (`Rollen`) | Aktion Rollen | E8 | übernommen (E8) |
| Alle Zellen leeren | `tilestudio.rs` (`Leeren`) | Aktion Leeren | E8 | übernommen (E8) |
| Zellen der Kachel merken und einfügen | `tilestudio.rs` (`Kopieren`, `Einfuegen`) | Aktion Kopieren, Einfügen | E8 | übernommen (E8) |
| Eigenschaften (Merkmale wie solid): anlegen, je Kachel setzen | `tilestudio.rs` (`Prop`, `PropText`, `PropNeu`) | Grafik: Kacheln | E8 | übernommen (E8) |
| Zoom, Einpassen, Verschieben der Fläche | `tilestudio.rs` (`Zoom`, `Einpassen`, `Schieben`) | Editor-Zoom | E8 | entfällt (die große Kachel passt sich der Breite an) |
| Rückgängig, Wiederholen, Speichern, Exportieren | `tilestudio.rs` | `rueckgaengig`, `speichern`, `exportieren` | E8 | übernommen (E8) |
| Export der Kachelbibliothek (Tabellen, Karte, Index) | `exportdlg.rs` (`KachelAkt`) | Export | E8 | übernommen (E8) |
| Tasten der Fläche (Pfeile bewegen den Zellcursor) | `tilestudio.rs` (`tile_taste`) | Grafik: Kacheln | E8 | übernommen (E8) |

## 20. Bildschirm-Editor (E9)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Bildschirm-Datei öffnen (Textbildschirm des Zielsystems) | `screenstudio.rs`, `screendoc.rs` | Grafik: Bildschirm | E9 | übernommen (E9) |
| Werkzeug Zeichen setzen (D), rechte Taste löscht | `screenstudio.rs` (`Werkzeug::Zeichen`) | Grafik: Bildschirm | E9 | übernommen (E9) |
| Werkzeug Text tippen (T), Pfeile bewegen den Cursor, Esc beendet | `screenstudio.rs` (`screen_taste`, `tippe`) | Grafik: Bildschirm | E9 | übernommen (E9) |
| Werkzeug Füllen (F): gleiche Zeichen füllen | `screenstudio.rs` | Grafik: Bildschirm | E9 | übernommen (E3.2) |
| Werkzeug Rahmen (R): aufziehen mit Klick, Ziehen, Loslassen | `screenstudio.rs` | Grafik: Bildschirm | E9 | übernommen (E3.2) |
| Werkzeug Pipette (I): Zeichen und Farbe aufnehmen | `screenstudio.rs` | Grafik: Bildschirm | E9 | übernommen (E9) |
| Werkzeug Löschen (E) | `screenstudio.rs` | Grafik: Bildschirm | E9 | übernommen (E9) |
| Zeichenwahl aus dem Zeichensatz | `screenstudio.rs` (`Zeichen`) | Grafik: Bildschirm | E9 | übernommen (E9) |
| Zeichensatz des Bildschirms wählen | `screenstudio.rs` (`Zeichensatz`) | Grafik: Bildschirm | E9 | übernommen (E9) |
| Farbe für neue Zeichen, Rollen Hintergrund und Vordergrund (Helligkeit) | `screenstudio.rs` (`Farbe`, `Rolle`) | Grafik: Bildschirm | E9 | übernommen (E9) |
| Ansicht PAL, NTSC oder Projekt-Norm | `screenstudio.rs` (`Ansicht`) | Grafik: Bildschirm | E9 | übernommen (E9) |
| Zerlegen in Kacheln (Vorschau, Kachelgröße, Kachelsatz und Bank anlegen) | `screenstudio.rs` (`ZerlegenDialog`, `Zerlegen`) | Aktion In Kacheln zerlegen | E9 | übernommen (E9) |
| Zoom, Einpassen, Verschieben | `screenstudio.rs` | Editor-Zoom | E9 | entfällt (der Bildschirm passt sich der Breite an) |
| Cursor-Anzeige Spalte, Zeile und Code | `screenstudio.rs` (`Cursor`) | Statuszeile Bildschirm | E9 | übernommen (E9) |
| Rückgängig, Wiederholen, Speichern, Exportieren | `screenstudio.rs` | `rueckgaengig`, `speichern`, `exportieren` | E9 | übernommen (E9) |

## 21. Karten-Editor und Bildschirmbank (E10)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Bank-Datei öffnen (Screens aus Kacheln und Zellen) | `bankstudio.rs`, `bankdoc.rs` | Grafik: Karten | E10 | fertig |
| Screens: neu, duplizieren, löschen (rückgängig stellt wieder her) | `bankstudio.rs` (`ScreenNeu`, `ScreenDupl`, `ScreenLoeschen`) | Grafik: Karten | E10 | fertig |
| Screens ordnen (nach vorn und hinten), benennen | `bankstudio.rs` (`ScreenVor`, `ScreenName`) | Grafik: Karten | E10 | fertig |
| Screen-Größe ändern (Breite, Höhe, Ursprung) | `bankstudio.rs` (`Groesse`) | Grafik: Karten | E10 | fertig |
| Modus Kacheln und Zeichen (Tab wechselt) | `bankstudio.rs` (`Modus`) | Grafik: Karten | E10 | fertig |
| Werkzeuge: Stift, Linie, Rechteck, Füllen, Pipette, Auswahl, Stempel, Text, Löschen | `bankstudio.rs` (`Werkzeug`, `werkzeug_taste`) | Grafik: Karten | E10 | fertig |
| Rechteck gefüllt oder als Rahmen | `bankstudio.rs` (`Gefuellt`) | Grafik: Karten | E10 | fertig |
| Kachelwahl aus dem Kachelsatz, Kachelsatz wählen | `bankstudio.rs` (`Kachel`, `Satz`) | Grafik: Karten | E10 | fertig |
| Zeichenebene: freie Zellen über den Kacheln (Patches) | `bankstudio.rs` (`Zeichen`, `Zellebene`) | Grafik: Karten | E10 | fertig |
| Zeichenfarbe und Zeichenbit 7 wählen | `bankstudio.rs` (`Pinselfarbe`, `Bit7`) | Grafik: Karten | E10 | fertig |
| Ebene ausblenden | `bankstudio.rs` | Grafik: Karten | E10 | fertig |
| Auswahl kopieren, als Stempel merken, leeren (Entf) | `bankstudio.rs` (`AuswahlKopieren`, `AuswahlLeeren`) | Grafik: Karten | E10 | fertig |
| Auswahl als neue Kachel anlegen (ersetzen oder neu) | `bankstudio.rs` (`AlsKachel`, `NKName`, `NKErsetzen`, `NKOk`) | Grafik: Karten | E10 | fertig |
| Eigenschaft einer Kachel einfärben | `bankstudio.rs` (`Eigenschaft`) | Grafik: Karten | E10 | fertig |
| Zellgitter und Kachelgrenzen zeigen | `bankstudio.rs` (`Gitter`, `Kachelgrenzen`) | Grafik: Karten | E10 | fertig |
| Zoom, Einpassen, Verschieben, Cursor mit Tasten | `bankstudio.rs` (`Zoom`, `Einpassen`, `Cursor`, `bank_taste`) | Editor-Zoom | E10 | fertig |
| Ansicht PAL, NTSC oder Projekt-Norm | `bankstudio.rs` (`Ansicht`) | Grafik: Karten | E10 | fertig |
| Nummer: Auswahlnummer für Screen-Wechsel im Programm | `bankstudio.rs` | Grafik: Karten | E10 | fertig |
| Kein Kachelsatz für dieses System: Hinweis | `bankstudio.rs` | Grafik: Karten | E10 | fertig |
| Rückgängig, Wiederholen, Speichern, Exportieren | `bankstudio.rs` | `rueckgaengig`, `speichern`, `exportieren` | E10 | fertig |

## 22. Hex-Ansicht und Speicherkarte (E11)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Hex-Ansicht für Binärdateien (nur lesend) | `hexansicht.rs` | Daten: Hex | E11 | übernommen (E11) |
| Seitenweise blättern (4 KiB), zum Anfang, zum Ende | `hexansicht.rs` (`Seite`, `Anfang`, `Ende`) | Daten: Hex | E11 | übernommen (E11) |
| Umfang: nur die ersten MiB geladen (Hinweis, ganze Datei geladen) | `hexansicht.rs` | Daten: Hex | E11 | übernommen (E11) |
| Im Standardprogramm öffnen | `hexansicht.rs` (`Extern`) | Daten: Hex | E11 | übernommen (E11) |
| Speicherkarte: 256 Seiten (16 × 16) mit Kategorien (Zeropage, Stack, RAM, ROM, E/A, frei) | `main.rs` (`panel_karte`), `memmap_facts.rs` | Daten: Speicher | E11 | übernommen (E11) |
| Speicherkarte: Programmbelegung je Seite als Balken | `main.rs` (`panel_karte`) | Daten: Speicher | E11 | übernommen (E11) |
| Speicherkarte: Filter (alle, Programm, RAM, ROM, E/A) | `main.rs` (`KarteFilter`) | Daten: Speicher | E11 | übernommen (E11) |
| Speicherkarte: Bankumschaltung mit Vorgaben (Register, Preset, Hinweis) | `main.rs` (`KartePreset`) | Daten: Speicher | E11 | übernommen (E11) |
| Speicherkarte: Seite wählen, Einzelheiten (Typ, Kategorie, Symbole, Quellzeilen) | `main.rs` (`KarteSeite`) | Daten: Speicher | E11 | übernommen (E11) |
| Speicherkarte: Prüfungen (Überschneidungen, Warnungen) | `memmap_facts.rs` (`checks`) | Daten: Speicher | E11 | übernommen (E11) |
| Speicherkarte: Legende | `main.rs` | Daten: Speicher | E11 | übernommen (E11) |
| Speicherkarte aus der letzten Bauauskunft laden | `main.rs` (`karte_laden`) | Daten: Speicher | E11 | übernommen (E11) |

## 23. Einstellungen, Einrichtung, Import und Export, Info (E12)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Einstellungen öffnen (Cmd+Komma) als Reiter | `setui.rs`, `main.rs` (`Tab::Einstellungen`) | `einstellungen` | E12 | übernommen (E12.1) |
| Einstellungen speichern und verwerfen (Entwurf) | `setui.rs` (`Speichern`, `Verwerfen`) | Einstellungen | E12 | übernommen (E12.1) |
| Seite Allgemein: Sprache, Darstellung, Zoom, Startbild | `setui.rs` (`Seite::Allgemein`) | Einstellungen ▸ Allgemein | E12 | übernommen (E12.1) |
| Menüleiste im Fenster anzeigen, Takt-Anzeige in der Statusleiste | `setui.rs`, `einstellungen.rs` | Einstellungen ▸ Aussehen | E12 | offen |
| Updates: nach Updates suchen, Kanal stabil, Adresse der Versionsliste | `setui.rs` (`UpdPruefen`, `UpdUrl`) | Einstellungen ▸ Allgemein | E12 | teilweise (E12.1): Wahl und Adresse; „Jetzt prüfen“ kommt mit E12.4 |
| Hilfe-Bilder laden (Adresse oder Ordner) | `setui.rs` (`HilfeBilder`, `HilfeUrl`) | Einstellungen ▸ Allgemein | E13 | offen |
| Debug-Protokoll in Datei schreiben (Fehlersuche) | `setui.rs` (`DebugLog`) | Einstellungen ▸ Allgemein | E12 | übernommen (E12.1) |
| Einrichtung: Assistent starten, Stand der Einrichtung | `setui.rs` | Einstellungen ▸ Allgemein | E12 | offen |
| Seite Zielsysteme und Hardware: Systeme hinzufügen, entfernen, filtern | `zielui.rs` (`Hinzu`, `Weg`, `Filter`) | Einstellungen ▸ Ziele | E12 | übernommen (E12.2) |
| Ziele: Startziel F5 je System (Emulator oder Hardware) | `zielui.rs` (`StartZiel`) | Einstellungen ▸ Ziele | E12 | übernommen (E12.2) |
| Ziele: Emulator eintragen (Name, Art, Pfad, ROM, Argumente) | `zielui.rs` (`EmuPlus`, `EmuName`, `EmuArt`, `EmuPfad`, `EmuRom`, `EmuArgs`) | Einstellungen ▸ Emulatorpfade | E12 | übernommen (E12.2) |
| Ziele: Emulator automatisch suchen, als Standard setzen, entfernen | `zielui.rs` (`EmuSuche`, `EmuStandard`, `EmuWeg`) | Einstellungen ▸ Emulatorpfade | E12 | übernommen (E12.2) |
| Ziele: Emulator testen (Probe: bereit, nicht gefunden) | `zielui.rs` (`EmuTest`, `Probe`) | Einstellungen ▸ Emulatorpfade | E12 | übernommen (E12.2) |
| Ziele: Emulator portabel herunterladen und installieren | `zielui.rs` (laedt, installiert) | Einstellungen ▸ Emulatorpfade | E12 | offen (E12.2b) |
| Ziele: Hardware eintragen (Ultimate, MEGA65, Name, Host, Passwort) | `zielui.rs` (`MaschPlus`, `MaschFeld`, `PwZeigen`) | Einstellungen ▸ Ziele | E12 | übernommen (E12.2) |
| Ziele: Verbindung je Hardware (Ethernet, USB seriell, Baudrate, Gerät, Ports scannen) | `zielui.rs` (`MaschVerb`, `MaschVerbFuer`, `Feld`) | Einstellungen ▸ Ziele | E12 | übernommen (E12.2) |
| Ziele: nach Übertragung RUN ausführen, Maschine vorher zurücksetzen | `zielui.rs` (`Haken`) | Einstellungen ▸ Ziele | E12 | übernommen (E12.2) |
| Ziele: Verbindung testen | `zielui.rs` (`MaschTest`) | Einstellungen ▸ Ziele | E12 | übernommen (E12.2) |
| Ziele: Hinweis „nur gegen Attrappen getestet“ je Hardware | `zielui.rs` | Einstellungen ▸ Ziele | E12 | übernommen (E12.2) |
| Argumente: Parametereditor je Programm (Katalog, Wert, Schalter) | `argsui.rs` (`ArgAkt`) | Einstellungen ▸ Argumente | E12 | offen (E12.2b) |
| Argumente: freies Feld, Platzhalter und Variablen einfügen | `argsui.rs` (`Frei`, `Platzhalter`) | Einstellungen ▸ Argumente | E12 | offen (E12.2b) |
| Argumente: auf Standard zurücksetzen, Migration, Vorschau | `argsui.rs` (`Zurueck`, `Migrieren`) | Einstellungen ▸ Argumente | E12 | offen (E12.2b) |
| Seite ROMs: ROM-Stand je System (erkannt, Ersatz, fehlt) | `romui.rs` | Einstellungen ▸ ROMs | E12 | übernommen (E12.2) |
| ROMs: ROM-Ordner öffnen, Pfade der Suche anzeigen | `romui.rs` (`OrdnerOeffnen`) | Einstellungen ▸ ROMs | E12 | übernommen (E12.2) |
| ROMs: ROM-Datei oder Zip wählen und ablegen, per Ablegen ins Fenster | `romui.rs` (`Waehlen`, `Abgelegt`) | Einstellungen ▸ ROMs | E12 | offen (E12.2b) |
| ROMs: ROMs anderer Programme (VICE, xemu) suchen und mitbenutzen | `romui.rs` (`Suchen`, `Fremde`) | Einstellungen ▸ ROMs | E12 | übernommen (E12.2) |
| ROMs: Fundliste kopieren (nur Auswahl, Alle, Keine) | `romui.rs` (`Haken`, `Alle`, `Kopieren`) | Einstellungen ▸ ROMs | E12 | offen (E12.2b) |
| ROMs: Prüfsummen CRC-32 und SHA-1, erwartete Größe | `romui.rs` | Einstellungen ▸ ROMs | E12 | übernommen (E12.2) |
| ROMs: Hinweis „woher bekomme ich ROMs“ | `romui.rs` (`Hilfe`) | Einstellungen ▸ ROMs | E12 | übernommen (E12.2) |
| ROMs: portabler Modus (Markierungsdatei) | `romui.rs` | Einstellungen ▸ ROMs | E12 | offen (E12.2b) |
| ROMs: Auswahl in roms.json gespeichert | `romui.rs` | Einstellungen ▸ ROMs | E12 | offen (E12.2b) |
| Seite Toolchain: Assembler (eingebaut, vc-asm), Optionen, Include-Suche | `setui.rs` (`Seite::Toolchain`, `AsmArgs`) | Einstellungen ▸ Toolchain | E12 | übernommen (E12.1) |
| Toolchain: BASIC-Dialekt je Zielsystem, Zeilennummern, Leerraum und Kommentarspalte | `setui.rs` | Einstellungen ▸ Toolchain | E12 | übernommen (E12.1, Anzeige; Optionen wie bisher unter Projekt ▸ Quellen) |
| Seite Editor: Schrift, Umbruch, Prettyprint, Diagnosen | `setui.rs` (`Seite::Editor`) | Einstellungen ▸ Editor | E12 | übernommen (E12.1) |
| Seite Tastenkürzel: Suche nach Befehl oder Kürzel, Kürzel sind fest | `setui.rs` (`KuerzelSuche`) | Einstellungen ▸ Tastenkürzel | E12 | übernommen (E12.1) |
| Zustimmung und Sicherer Modus (Erlaubnis für Start externer Programme) | – (im Iced nur über Einstellungen und Rückfragen) | Einstellungen ▸ Zustimmung | E12 | kommt 0.3.1 (keine Platzhalterseite in 0.3.0) |
| Konfiguration exportieren (ohne Passwörter) | `konfig.rs` (`KArt::Export`) | Einstellungen ▸ Sicherung | E12 | übernommen (E12.1) |
| Konfiguration importieren (mit Rückfrage) | `konfig.rs` (`KArt::Import`) | Einstellungen ▸ Sicherung | E12 | übernommen (E12.1) |
| Einstellungen auf Vorgabe zurücksetzen (mit Rückfrage) | `konfig.rs`, `main.rs` (`KonfigReset`) | Einstellungen ▸ Sicherung | E12 | übernommen (E12.1) |
| Pfade im persönlichen Ordner als Abkürzung anzeigen | `setui.rs`, `pfadfeld.rs` | Einstellungen | E12 | übernommen (E12.1) |
| Pfadfeld mit Dateidialog (Datei oder Ordner wählen) | `pfadfeld.rs` | alle Pfadfelder | E12 | teilweise (E12.1): Projektordner; weitere Pfadfelder mit E12.2 |
| Einstellungsdatei im Ordner zeigen | `setui.rs` (`OrdnerZeigen`) | Einstellungen | E12 | übernommen (E12.1) |
| Einrichtungsassistent Schnelleinrichtung (7 Schritte) | `assistent.rs` | Assistent | E12 | übernommen (E12.3) |
| Assistent: alles selbst einstellen (12 Schritte), Teilassistenten (ROMs, Emulatoren, Hardware, Design) | `assistent.rs`, `befehle.rs` | `assistent`, `assistent_*` | E12 | übernommen (E12.3) |
| Assistent: Willkommen mit Sprachwahl | `assistent.rs` | Assistent | E12 | übernommen (E12.3) |
| Assistent: Farbschema, Zoom, Editor-Schrift wählen | `assistent.rs` (`Schema`, `Zoom`, `Schrift`) | Assistent | E12 | übernommen (E12.3) |
| Assistent: Zielsysteme wählen, Emulatoren, ROMs, Hardware, Startziel | `assistent.rs` (`System`) | Assistent | E12 | übernommen (E12.3) |
| Assistent: Projektordner und Beispiele kopieren, erstes Projekt | `assistent.rs` (`ProjekteOrdner`, `BeispieleKopieren`, `Erstes`) | Assistent | E12 | übernommen (E12.3; „Erstes Projekt“ folgt) |
| Assistent: Zusammenfassung, Übernehmen, Rückgängig, Sicherung | `assistent.rs` (`Uebernehmen`, `Rueckgaengig`) | Assistent | E12 | übernommen (E12.3) |
| Assistent: Später fortsetzen, Schritt überspringen, Esc bricht ab | `assistent.rs` (`Spaeter`, `Abbruch`) | Assistent | E12 | übernommen (E12.3) |
| Assistent: „Nach Neuem suchen“ (ROM-Verzeichnis neu einlesen) | `main.rs` (`AssistentNeues`) | `assistent_neues` | E12 | übernommen (E12.3: öffnet den ROM-Teil) |
| Projekteinstellungen: Name, Beschreibung, Projektbild | `projeinst.rs` | `projekt_einstellungen` | E12 | offen |
| Projekteinstellungen: Ziele (Videonorm, Speicher, Emulator, Start) | `projeinst.rs` (`PeReiter::Targets`, `Target*`) | Projekteinstellungen ▸ Ziele | E12 | offen |
| Projekteinstellungen: Quellgruppen (Art, Ziel, Dialekt, Reihenfolge) | `projeinst.rs` (`PeReiter::Quellen`, `Gruppe*`) | Projekteinstellungen ▸ Quellen | E12 | offen |
| Projekteinstellungen: Umwandlung BASIC nach Assembler, Laufzeit einbetten, Fließkomma | `projeinst.rs` (`Conv*`) | Projekteinstellungen ▸ Quellen | E12 | offen |
| Projekteinstellungen: Diskette (Format D64, D81, ATR, Programme, Name) | `projeinst.rs` (`PeReiter::Diskette`) | Projekteinstellungen ▸ Diskette | E12 | offen |
| Projekteinstellungen: Hauptdatei, Dateien je Gruppe hinzufügen und ordnen | `projeinst.rs` (`DateiMain`, `DateiPlus`, `DateiAuf`) | Projekteinstellungen ▸ Quellen | E12 | offen |
| Projekt kopieren (Name, Ort, Bauordner, Git, danach öffnen) | `kopierdlg.rs` | `projekt_kopieren` | E12 | offen |
| Import: Dateien ins Projekt übernehmen (Ordner, Name, Konflikt: ersetzen, umbenennen, überspringen) | `importdlg.rs`, `importfiles` | Import | E12 | offen |
| Export-Dialog: Format (ASM, INC, PRG, BAS, Hex, Binär) und Name | `exportdlg.rs` | `exportieren` | E12 | offen |
| Export: Ladeadresse prüfen (PRG, INC) | `exportdlg.rs` (`braucht_adresse`) | Export | E12 | offen |
| Export: Vorschau-Reiter Text und Daten, Größe in Byte | `exportdlg.rs` (`Preview`) | Export | E12 | offen |
| Export: im Projekt speichern, in offenen Quelltext einfügen, Testlauf | `exportdlg.rs` (`ExportAus`) | Export | E12 | offen |
| Info: Version, Autor, Dank, Lizenz, Quelltext | `infodlg.rs` | `ueber` | E12 | übernommen (E12.4) |
| Info: Herkunft und Lizenzen, Drittanbieter-Hinweise | `infodlg.rs` (`Herkunft`, `Hinweise`) | `ueber` | E12 | teilweise (E12.4): Drittanbieter-Hinweise; die Tabelle der eigenen Zeichensätze kommt mit E13 |
| Feedback senden (Mail vorbereiten, kopieren, Debug-Protokoll) | `infodlg.rs` (`Mail`, `Kopieren`) | `feedback` | E12 | übernommen (E12.4) |
| Update prüfen (jetzt prüfen, neue Version, Änderungen ansehen) | `infodlg.rs` (`Pruefen`, `Ansehen`) | `update_pruefen` | E12 | übernommen (E12.4) |
| Update: später erinnern, diese Version überspringen | `infodlg.rs` (`Spaeter`, `Ueberspringen`) | `update_pruefen` | E12 | übernommen (E12.4) |
| Alpha-Hinweis beim Start | `main.rs` (gui-alpha-version) | Rahmen | E12 | entfällt in 0.3.0 (Entscheidung: kein Alpha-Hinweis); Stellen merken für die Versionsstufe |
| Tastenkürzel-Übersicht und Schnellreferenz | `startui.rs`, `setui.rs` | Einstellungen ▸ Tastenkürzel | E12 | übernommen (E12.1) |

## 24. Hilfe, Handbuch, Handbuchbilder (E13)

| Funktion | Iced (Datei) | neu (Ansicht/Befehl) | Etappe | Status |
|---|---|---|---|---|
| Hilfe im Panel: Themenliste nach Gruppen, Suche | `main.rs` (`panel_hilfe`, `HilfeSuche`) | Panel Hilfe | E13 | übernommen (E13) |
| Hilfe: Thema öffnen, Zurück, Links zwischen Themen | `main.rs` (`HilfeOeffne`, `HilfeZurueck`) | Panel Hilfe | E13 | übernommen (E13) |
| Hilfe zum Wort unter dem Cursor (F1) | `main.rs` (`Hilfe`, `wort_unter_cursor`) | `hilfe` | E13 | übernommen (E13) |
| Hilfe im Reiter öffnen (Dokumentenfeld, lesbare Breite) | `main.rs` (`HilfeImReiter`, `hilfe_reiter`) | `hilfe_reiter` | E13 | übernommen (E13: Hilfe ist ein eigener Bereich im Dokumentenfeld) |
| Hilfe je Zielsystem und Sprache | `main.rs` (`hilfe_ziel`, `hilfe_sprache`), `helpstate.rs` | Hilfe | E13 | übernommen (E13) |
| Markdown-Rendering der Hilfe (Überschriften, Listen, Tabellen, Code, Bilder) | `mdansicht.rs`, `vc-md` | Markdown-Ansicht | E13 | übernommen (E13) |
| Handbuchbilder erzeugen (Ansichten als PNG für das Handbuch) | `handbuchbilder.rs` | Prüfbrücke und Bildskript | E13 | offen (Dokumentation: Bildskript, nicht Teil der Oberfläche) |
| Handbuchbilder laden (Adresse oder Ordner) | `setui.rs` (`HilfeBilder`) | Einstellungen | E13 | übernommen (E13) |
| Handbuch (Kapitel unter `docs/handbuch`) | – | Hilfe ▸ Handbuch | E13 | übernommen (E13) |

## 25. Befehlstabelle

Anzahl der Befehle je Menü laut `crates/vc-gui-ui/src/befehle.rs` (Tabelle `BEFEHLE`, insgesamt 137 Einträge). Das Menü „Baum“ steht in keiner Menüleiste: Seine 10 Befehle liegen nur im Kontextmenü des Dateibaums und in der Befehlspalette. Die Befehlstabelle der neuen Oberfläche liegt im Crate `vc-commands` und speist Menüleiste, Palette und Kürzel (E0 übernommen); die Zeilen unten prüfen je Menü, dass jeder Befehl in seiner Etappe wirkt.

| Gruppe (Menü) | Iced (Datei) | neu (Menü und Befehle) | Etappe | Status |
|---|---|---|---|---|
| Datei: 18 Befehle (Projekt neu, öffnen, Ordner, kopieren; Reiter, Datei öffnen, Speichern, Alle speichern, Schließen, Neue Datei, Notiz; Zeichensatz-Befehle; BASIC nach ASM; Exportieren; Startseite; Beenden) | `befehle.rs` | Menü Datei | E1.1 bis E1.4 | offen |
| Bearbeiten: 40 Befehle (Rückgängig, Wiederholen, Zwischenablage, Suchen, Ersetzen, Gehe zu, Projektsuche, Definition, Haltepunkt, Falten, Vervollständigen, Formatieren, Zeilenende, Kodierung, Art, Zahlenwerkzeuge, Hex-Vorschläge, 16 Markdown-Befehle) | `befehle.rs` | Menü Bearbeiten | E1.1 bis E1.2, E13 | offen |
| Ansicht: 36 Befehle (Palette, Bereiche, Zen, Maximieren, Zoom, Schrift, Farbschema, Umbruch, Reiter und Teilen, Zoom und Raster der Grafik-Editoren, Tracker-Ansichten, Markdown umschalten, Einstellungen, Layout zurücksetzen) | `befehle.rs` | Menü Ansicht | E0 bis E1.4, E3, E5, E12 | offen |
| Projekt: 22 Befehle (Bauen, Starten, Debuggen, aktive Datei, Debug-Schritte, Bis hierher, Tests, Probleme, Protokoll, Projekteinstellungen) | `befehle.rs` | Menü Projekt | E1.3, E6, E12 | offen |
| Hilfe: 11 Befehle (Hilfe F1, Hilfe im Reiter, Über, Feedback, Update prüfen, Assistent und fünf Teilassistenten) | `befehle.rs` | Menü Hilfe | E12, E13 | offen |
| Baum (kein Menü): 10 Befehle nur im Kontextmenü des Dateibaums und in der Palette | `befehle.rs`, `kontext.rs` | Kontextmenü Dateibaum, Palette | E1.1 | offen |

## 14. Mindestliste Editorfunktionen (Vorschlag zur Ergänzung durch den Inhaber, vor E1.2)

Anforderung des Inhabers: Der Code-Editor bietet alle Funktionen, die ein guter Code-Editor haben muss. Diese Liste ist ein **Vorschlag**; bitte streichen oder ergänzen. Status: `übernommen` (gebaut und im Ablauftest), `offen` (Etappe).

| Funktion | Etappe | Status |
|---|---|---|
| Blockfaltung (Marken, Befehle Alles falten/entfalten, Tastatur) | E1.1 | übernommen |
| Suchen und Ersetzen mit Groß/Klein, ganzem Wort, **regulären Ausdrücken** (Erfassungsgruppen im Ersetzen) | E1.1 | übernommen (CodeMirror-Suchleiste; Erfassungsgruppen `$1` im Handtest prüfen) |
| Gehe zu Zeile | E1.1 | übernommen |
| Rückgängig/Wiederholen, Zeilennummern, Klammerhervorhebung, Klammern ergänzen, Einrückung übernehmen | E1.1 | übernommen |
| Tab/Umschalt+Tab (Einrücken), Auswahl (Wort, Zeile, alles), Zwischenablage | E1.1 | übernommen |
| Lesezeichen, Haltepunkte am Rand, Schriftgröße | E1.1 | übernommen |
| Zeilen verschieben, duplizieren, löschen | E1.2 | übernommen (CodeMirror-Standardtasten: Alt+↑/↓, Umschalt+Alt+↑/↓, ⌘⇧K; Menüeinträge nach Entscheidung des Inhabers) |
| Kommentar umschalten (Zeile, Auswahl) | E1.2 | übernommen (⌘/; Assembler `;`, strukturiertes BASIC `'`) |
| Mehrfachcursor, Auswahl erweitern, Wort- und Zeilenoperationen | E1.2 | übernommen (CodeMirror-Standard: ⌘D nächstes Vorkommen, ⌘⌥↑/↓ Cursor darüber/darunter, Alt+Klick) |
| Einrückungslinien, Leerzeichen und Zeilenenden anzeigen, Zeilenumbruch umschalten | E1.2 | Zeilenumbruch übernommen (Befehl `umbruch`); Einrückungslinien und unsichtbare Zeichen offen |
| **Autovervollständigung** (Vokabular, Marken, Direktiven aus `data/asm/directives.json`, Sprachdienst) | E1.2 | übernommen (Befehle, Marken, Konstanten, Makros, Chipregister, Direktiven kanonisch zuerst, alte Namen erst beim Tippen mit Hinweis, Zeichennamen nach `{`) |
| **Kontextbezogenes Prettyprint** (`vc-ide-model::pretty`; beim Zeilenwechsel, für Auswahl und Datei, als ein Rückgängig-Schritt) | E1.2 | übernommen (BASIC rückt nach Blocktiefe der Zeilen davor ein) |
| Hover/Befehlshilfe, Signaturhilfe, Diagnosen (Wellenlinie, Fehlerkarte), Quick-Fix | E1.2 | übernommen bis auf Signaturhilfe (offen) |
| Gehe zu Definition, Gliederung, Projektsuche (mit Abbruch) | E1.2 | übernommen |
| Snippets (Textbausteine) | E1.2 | offen, Vorschlag |
| Fehlersprung F8 / Umschalt+F8 (Befehle „Nächstes/Voriges Problem“ in Menü und Palette; auf dem Mac sind F-Tasten oft Mediatasten: Palette ⌘K bleibt der sichere Weg) | E1.2 | übernommen (im Editor; über Dateien hinweg mit dem Bauergebnis in E1.3) |
| Umbenennen eines Namens über Dateien hinweg | nach 0.3.0 | zurückgestellt (eigene Kernarbeit mit Symbolindex) |
| Zur passenden Klammer springen (Ctrl+M, auf der deutschen Tastatur erreichbar; Menü Bearbeiten ▸ Suchen und Ersetzen) | E1.2 | übernommen |
| Zeilen sortieren (markierte Zeilen), Leerzeichen am Zeilenende entfernen (Auswahl oder Datei); je ein Rückgängig-Schritt, **keine** Automatik beim Speichern (Byte-Gleichheit); Einstellung frühestens E12 | E1.2 | übernommen |
| Zahlen-Hover (Hex, Dezimal, Binär, Low-/High-Byte, Registername aus den Chip-Daten) | E1.2 | übernommen; Farbvorschau bei Farbregistern offen (E1.2 Rückstand) |
| Geteilte Ansicht (zwei Editor-Hälften, Parität zu Iced); Matrixzeilen Teilen | E1.4 | übernommen |
| Randleiste mit Markierungen wie im Mockup | E1.2 | übernommen (Rand mit Haltepunkten, Lesezeichen, Diagnosen) |
| Minimap | – | abgelehnt (Inhaber) |
| Sticky-Kopfzeile (aktuelle Marke oben anheften) | später | Wunschliste, bei Bedarf |
| Bauen, Probleme anklicken, „Ab hier starten“, Zyklenzahl je Zeile | E1.3 | offen |
| Dateien aus dem Dateimanager in das Fenster ziehen: Routing nach Endung, Rückfrage „Ins Projekt kopieren und öffnen?“, Hinweisfläche | neu (in der Iced-Oberfläche nicht vorhanden) | Gesamte Oberfläche | W1 | übernommen (Handprüfung auf allen Systemen, siehe `handpruefung-w1.md`; Ziehen im Fenster mit Zeigerereignissen) |

## Registry (R0)
| Funktion | Iced-Oberfläche | Neue Oberfläche | Etappe | Stand |
|---|---|---|---|---|
| Menü und Palette richten sich nach der Release-Auswahl (`release.json`): abgewählte Editoren fehlen, ihre Dateien öffnen als Text mit Hinweis | nein (Menü zeigte alles) | Menü, Palette, Navigation | R0 | übernommen (neu) |
| Befehle aus Erweiterungen (`ext:<paket>:<befehl>`) mit Herkunft in Menü und Palette; Untermenü „Projekt ▸ Erweiterungen“ nur wenn vorhanden | nein | Menü, Palette | R0 | übernommen (neu; Ausführen folgt mit E2) |
| Kürzel-Konflikte werden gemeldet, nichts wird überschrieben | – | Kern | R0 | übernommen |
| Beispiele und Projektstrukturen aus allen Quellen mit Herkunft (aus Paketen nur Kopieren) | nein | Startseite, Neues Projekt | R0 | übernommen (neu) |
| Hilfe-Themenregister mit Quellen (Themen aus Paketen, Namensraum `<paket>:<id>`) | nein | Kern (Oberfläche mit der Hilfe-Etappe) | R0 | Kern übernommen |
| Vorlagen, Zielsysteme, Paletten, Zeichensätze über Suchpfade der Registry | Datenordner | Datenordner | E1 | zurückgestellt: bleiben bis E1 am Datenordner (kein Bedarf vor Paketen dieser Arten; die Registry liefert Suchpfade und zusammengeführte Listen dafür schon) |
| Benutzer- und Projektquellen im Programm | – | nur Entwicklerschalter `VC_EXTENSIONS_DEV` (Feature `dev-extensions`) | E1 | wie entschieden: erst mit Prüfung und Zustimmung (E1) |


## Entscheidung D6 (2026-11-23): bewusst nach 0.3.1

Erweiterungs-Oberfläche (Etappen E1 und E2 des Registry-Plans), Karten-Editor (E10), Englisch in Hilfe und Handbuch nur für Kernbereiche, Handbuchbilder in Englisch. In 0.3.0 bleiben: Registry R0 (Anzeige der Pakete), E4–E9, E11–E13, BASIC-Anbindung, Include-Pfade. In den Einstellungen (E12) entsteht keine Erweiterungsverwaltung; die Seite „Erweiterungen“ entfällt oder ist ein Platzhalter („kommt später“). Alle zugehörigen Zeilen dieser Matrix gelten als „bewusst nach 0.3.1“.

## Bereich „Projekt“ (Wunsch des Inhabers 2026-11-23, Etappen P1–P4)

| Funktion | Quelle (alte IDE) | Ort in der neuen Oberfläche | Etappe | Stand |
|---|---|---|---|---|
| Projekt-Einstellungen als Seiten (Übersicht, Allgemein, Ziele, Quellen, Bauen und Starten, Diskette, Weitergeben) | `projekt_einstellungen` (Reiter) | Hauptbereich Projekt, zweite Reihe | P1 | übernommen (Rahmen; Inhalt P2/P3) |
| Turbo-Karte | Projekt-Einstellungen | Projekt ▸ Diskette | P1 | übernommen |
| Übersicht mit Zustandspunkten und „Ändern ›“ | Hilfe overview.md | Projekt ▸ Übersicht | P2 | übernommen |
| Name, Beschreibung, Haupt-/Startdatei, Reiter wiederherstellen | Projekt-Einstellungen ▸ Allgemein | Projekt ▸ Allgemein | P2 | übernommen |
| Zielsysteme und Varianten | Projekt-Einstellungen ▸ Targets | Projekt ▸ Ziele | P2 | teilweise (Varianten ja; Ziele hinzufügen/entfernen offen) |
| Quellen in Baureihenfolge, fehlende Dateien rot | Projekt-Einstellungen ▸ Quellen | Projekt ▸ Quellen | P3 | offen |
| Bau-/Emulator-Einstellungen, Pakete (Registry) mit Herkunft | Einstellungen, Registry R0 | Projekt ▸ Bauen und Starten | P3 | offen |
| Export/Kopieren, Zuletzt-Verlauf | Dateimenü, Startseite | Projekt ▸ Weitergeben | P3 | offen |
