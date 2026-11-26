# Paritätsprüfung 0.3.0: Iced-Oberfläche gegen neue Tauri-Oberfläche (Stand 2026-11-25, `claude/ui-e5`)

Grundlage ist die Matrix [paritaet-oberflaeche.md](paritaet-oberflaeche.md) (jede Funktion der Iced-Oberfläche, Ansicht oder Befehl der neuen Oberfläche, Etappe, Status). Diese Datei hält das Ergebnis der Gegenprüfung gegen den Code fest: Alle Zeilen, deren Status nicht „übernommen“, „fertig“ oder „neu“ war, wurden einzeln geprüft; bei „übernommen“ gab es Stichproben (jede fünfte Zeile) und eine Prüfung aller Befehlskennungen. Nichts ist gestrichen worden. Die zehn nicht machbaren Handbuchbilder stehen in [handbuchbilder-neue-oberflaeche.md](handbuchbilder-neue-oberflaeche.md). Schreibweise: Zeropage.

## Zahlen

| Bereich | geprüft (Status nicht übernommen) | inzwischen vorhanden | offen oder teilweise | unklar |
|---|---|---|---|---|
| Abschnitte 1–12 (Rahmen bis Startseite) | rund 80 | 33 | 40 | 7 |
| Abschnitte 13–22 (Disk bis Hex/Speicher) | 41 | 7 (+3 teilweise) | 5 echt offen, 3 teilweise, 3 bewusst 0.3.1 | – |
| Abschnitte 23–Ende (Einstellungen, Hilfe, Befehlstabelle, Projekt) | rund 52 | rund 20 | rund 12 (+14 Befehle ohne Ausführung) | 1 |

Befehlskennungen der Spalte „neu“: keine fehlt in der Befehlstabelle (`vc-commands`). Echt sind 155 Einträge (Datei 18, Bearbeiten 52, Ansicht 37, Projekt 26, Hilfe 11, Baum 11), 18 davon neu gegenüber Iced; alle 137 Iced-Befehle sind enthalten. Die Tabelle in Abschnitt 25 der Matrix nennt noch 137 und ist zu aktualisieren.

## Gefundene Fehler in der Matrix

- Merge-Marken (`<<<<<<< HEAD` … `>>>>>>> claude/ui-w1`) in `paritaet-oberflaeche.md` (Abschnitt Registry): **bereinigt**, die W1-Zeile „Dateien ziehen“ steht wieder in ihrer Tabelle.
- Zeile „Haltepunkt auf Sektoradresse“ (Disk) meint den Disassembler-Haltepunkt; der ist vorhanden (`DebugView.svelte`). Zeile umformulieren.
- Karten-Editor (Abschnitt 21) steht auf „fertig“, die Entscheidung D6 nennt ihn „bewusst nach 0.3.1“; er existiert (`KartenView.svelte`, `karten.rs`). D6 angleichen.
- „Menüleiste `Menu.svelte`“: Die Menüleiste ist nativ (`menu.rs`), `Menu.svelte` ist nur ein Aufklappmenü. Zeile berichtigen.

## Entscheidungen nötig (Vorschläge)

**A. Befehle in Menü und Palette ohne Ausführung** (beim Aufruf erscheint „folgt in einer späteren Etappe“): `kodierung` (BOM, ISO-Warnung), `zahlenwerkzeuge`, `basic_nach_asm`, `als_include`, `hex_vorschlaege`, `projekt_ordner_oeffnen`, `std_zeichensatz`, `zeichensatz_rom`, `zeichensatz_png`, `studio_vergroessern`, `studio_verkleinern`, `studio_einpassen`, `studio_raster`, `studio_zwiebel` (nur interner Name) sowie `kopieren`, `ausschneiden`, `einfuegen` außerhalb des Sprite-Editors. Vorschlag je Befehl: entweder in 0.3.0 anbinden (klein bis mittel) oder aus Menü und Palette ausblenden und nach 0.3.1 schieben; D6 sagt „Include-Pfade bleiben in 0.3.0“, das ist für `als_include` klarzustellen. Ob die Editoren die `studio_*`-Befehle über einen anderen Weg bedienen, ist offen.

**B. Zeichensatz-Import** (alle drei Dialoge fehlen, der Kern `std_charset_list` liegt fertig): Standard-Zeichensatz-Dialog (Weg zu den freigegebenen eigenen CC0-Sätzen, mittel) in 0.3.0 bauen; ROM und PNG-Raster bauen oder nach 0.3.1 schieben (hängt am Handbuchbild `zeichensatz-png-import`).

**C. Klein, in 0.3.0 nachziehbar:** Grundtext in der Palette für ausgegraute Befehle; Schalter für die Absturzsicherung; Zeichenpanel; Zwischenablage im Code-Editor über die Palette; ROM-Import „Alle wählen/abwählen“; ROM-Seite „Portabler Modus“; Dateidialog an den Emulatorpfaden; Projekt ▸ Ziele: Standard-Emulator und Speicherausbau; Projekt ▸ Quellen: Reihenfolge der Gruppen; Info: Tabelle der eigenen Zeichensätze (Katalogschlüssel da, nirgends benutzt); Diskette: Programmauswahl (`disk.files`) und Ausgabedatei; Zeichensatz: Anzeige „ROM fehlt“ (Kerntext `view.status` wird nirgends gezeigt).

**D. Entscheidung des Inhabers:** BASIC nach Assembler als Bauoption (Kern kann es, 0.3.0 mittel oder 0.3.1); Menüleisten-Schalter (nur Hinweistext; entfällt, da die Menüleiste nativ ist?).

**E. Bewusst nach 0.3.1 oder entfällt (mit Begründung in den Detailtabellen):** Datei-Symbole und Gruppen im Baum, exportierte Dateien markieren, Hinweis „Definition außerhalb“, Diagnosen ein/aus, Setup-Zeile der Tests, Takt und Norm in der Statusleiste, Export beim Bauen, Hardware-Probe, Startseite „neue Datei“, Import-Dialog (Ablegen kopiert und benennt bei Namensgleichheit selbst um), Projektbild in den Projekteinstellungen, Takt-Anzeige, Editorwünsche (Einrückungslinien, sichtbare Leerzeichen, Signaturhilfe, Snippets, Farbvorschau), MEGA65-Diskettenübertragung, VIC-IV-Sprites, grafisches Chip-Fenster im Debug; entfallen: Menüleiste im Fenster ein/aus, „ausgegraut mit Grund“ im nativen Menü (technisch nicht möglich).

**F. Unklar, nur am Gerät klärbar:** Bildschirmleser (VoiceOver-Handtest zeigt Namen, Rollen, Zustände in 14 Bereichen ✔); Aktionen je Editor in der Palette, Kontextmenü Zeichenfläche, Kontextmenü Aktionen, Kopfzeile in Breitenstufen, Tests parallel zur Debug-Sitzung, Kachelkarte umwandeln, Rückfrage beim Debug-Beenden, Assistent „Erstes Projekt“.

## Detailtabellen

### Abschnitte 1–12


Nachweise: UI = `ui/src/...`, Kern = `crates/vc-shell/src/...`, Tabelle = `crates/vc-commands/src/lib.rs`. Befehle ohne Zweig in `ui/src/lib/commands.ts` fallen auf `default:` und zeigen nur die Meldung „noch nicht gebaut“ (commands.ts, letzte Zeile von `runCommand`).

## A. Zeilen mit Abweichung oder Offenem

| Abschnitt | Funktion | bisheriger Status | Befund | Nachweis | Vorschlag |
|---|---|---|---|---|---|
| 1 | Palette: unscharfe Suche, alle Wörter, Wortanfang zuerst | offen | vorhanden (alle Wörter per `includes`, Rang: Wortanfang des ersten Worts zuerst, nutzbare vor ausgegrauten) | CommandPalette.svelte:40-42 | übernommen |
| 1 | Palette: Pfeile, Enter, Esc | offen | vorhanden | CommandPalette.svelte `key()` | übernommen |
| 1 | Palette: ausgegraut mit Grund | offen | halb: ausgegraut ja (`enabled`), Grund nur bei Turbo/Debug; für „Projekt/Editor nötig“ kein Grundtext | CommandPalette.svelte:27-31, 71 | noch in 0.3.0 bauen (klein): Grundtext „Projekt nötig“/„Editor nötig“ |
| 1 | Palette: Aktionen des offenen Editors (Werkzeuge, Muster, Sektoren) | offen | fehlt. Palette zeigt nur Befehlstabelle, Dateien, Projekte, Orte. Die Tabelle hat Editor-Befehle (`studio_*`, `tracker_*`), aber `studio_*` haben keinen Zweig in commands.ts (nur Meldung) | CommandPalette.svelte:26-38; commands.ts | unklar; für 0.3.0 nur `studio_*` verdrahten (klein), Aktionsverzeichnis je Ansicht bewusst nach 0.3.1 |
| 1 | `lang:auto` | offen | vorhanden | CommandPalette.svelte:36; commands.ts `lang:`; Kern cmds.rs `lang_set` (Auto) | übernommen |
| 1 | Kürzelanzeige `Kbd` | offen | vorhanden | Kbd.svelte; Palette `keysDisplay`; Kern `fuer_anzeige` | übernommen |
| 1 | Bildschirmleser | teilweise | nicht prüfbar ohne Gerät | – | unklar, Handpunkt bleibt |
| 2 | Untermenüs der langen Menüs | offen | vorhanden (native Untermenüs aus `untermenues_alle`) | menu.rs `bauen` | übernommen |
| 2 | Menüleiste im Fenster ein/aus (Win/Linux) | offen | entfällt de facto: native Menüleiste immer an; Einstellungen zeigen nur Hinweistext (Seite Editor, nicht Aussehen) | SettingsView.svelte:68; ui.ftl:1820-1822 | entfällt (nativ, keine Umschaltung nötig); Matrix-Ort korrigieren |
| 2 | Systemmenü des Macs | offen | vorhanden (Über, Ausblenden) | menu.rs `bauen` (appm) | übernommen |
| 2 | Alt+Buchstabe, Pfeile im Menü | offen | vorhanden über natives Menü (`&`-Buchstabe, nur Win/Linux; Mac hat das nicht); Test auf Eindeutigkeit | menu.rs `mit_buchstabe`; lib.rs Test `alt_buchstaben…` | übernommen (Handprüfung Win/Linux) |
| 2 | Menüeinträge ausgegraut mit Grund | offen | ausgegraut ja (Kontext), Grund nicht (native Menüs können keinen anzeigen) | menu.rs `Kontext` | übernommen ohne Grund; Grundteil entfällt (Plattform) |
| 2 | Kürzel auslösen | offen | vorhanden (nativ plus Rückfall) | keys.ts `matchesSpec`; App.svelte:131-146 | übernommen |
| 2 | Doppelereignis verwerfen | offen | vorhanden (`once`, 150 ms) | App.svelte:128-130 | übernommen |
| 2 | Kürzel nach physischer Taste (Tracker) | offen | vorhanden (`KLAVIER[e.code]`, Alt bleibt Text) | TrackerView.svelte:55; keys.ts | übernommen |
| 2 | Kürzel-Diagnose | 0.3.1 | bekannt | – | bleibt 0.3.1 |
| 2 | Kontextmenü Editor (Ausschneiden … Falten, Haltepunkt) | offen | unklar/prüfen: Befehle Zwischenablage laufen im Editor über Webview; in der Palette zeigen `kopieren/ausschneiden/einfuegen` außerhalb des Sprite-Editors nur „noch nicht gebaut“ (commands.ts, Zweig `kopieren`) | commands.ts | noch in 0.3.0 (klein): Zwischenablage im Code-Editor per CodeMirror anbinden; Kontextmenü Code-Editor Eintragsliste vor Abnahme gegenprüfen |
| 2 | Kontextmenü Zeichenfläche | offen | nicht belegt in diesem Lauf (nicht gesucht) | – | unklar |
| 2 | Kontextmenü Aktionen des Editors | offen | nicht belegt | – | unklar |
| 3 | Datei öffnen aus dem Baum | offen | vorhanden (Routing nach Art: Disk, Zeichensatz, Song, Bildschirm, Kacheln, Palette, Sprite, Text, Hex, extern) | editor.svelte.ts:161-195 | übernommen |
| 3 | Ablegen aus dem Betriebssystem (Import) | offen | vorhanden (Routing, Rückfrage, Disk) | ablegen.svelte.ts; DropLayer.svelte; Kern `drop_plan` | übernommen |
| 4 | Zeile hervorheben (Debug) | offen | vorhanden | cm.ts:24-27 `setDebugLine`; editor.svelte.ts:127,184 | übernommen |
| 4 | Einfügemarke beim Ziehen aus dem Baum | offen | fehlt (kein Ziel für `kind:"tree"` im Editor, nur Tab-Ziele) | CodeEditor.svelte:78-79; dnd.svelte.ts | Entscheidung Inhaber nötig: Im D6 stehen „Include-Pfade“ in 0.3.0; Zeilen 113-116 hängen daran. Vorschlag: noch in 0.3.0 (mittel) oder bewusst 0.3.1 klar benennen |
| 4 | Include-Zeile beim Ablegen im Quelltext | offen | fehlt (nur Export-Dialog „Include“ erzeugt Zeile; Absprung per Doppelklick ist da) | export.svelte.ts:45; include-sprung.ts | wie oben |
| 4 | „Als Include einfügen“ (`als_include`) | offen | Befehl in Tabelle und Baum-Menü, aber kein Zweig: nur Meldung | lib.rs:87; commands.ts | wie oben; mindestens klein bauen (Einfügen der Zeile an Cursor über Kern `dropinc::plan`, der in export.rs:66 schon benutzt wird) |
| 4 | Hinweis „schon eingebunden“ | offen | fehlt (Teil von dropinc, nur im Export-Pfad als Fehler `export-abgelehnt`) | export.rs:66 | wie oben |
| 6 | Haltepunkte bleiben beim Neuöffnen | offen | vorhanden (auch in der Sitzung gespeichert) | editor.svelte.ts:286-327 | übernommen |
| 7 | Kodierung UTF-8/ISO-8859-1 (`kodierung`) | offen | fehlt: Befehl in Tabelle, kein Zweig, keine Kodierung in Kern/UI | lib.rs `kodierung`; Suche nach Kodierung/BOM ohne Treffer | noch in 0.3.0 (klein bis mittel) oder bewusst 0.3.1; Entscheidung nötig |
| 7 | BOM erkennen und erhalten | offen | fehlt (kein Treffer) | – | zusammen mit Kodierung |
| 7 | Warnung bei Zeichen ohne ISO-8859-1 | offen | fehlt | – | zusammen mit Kodierung |
| 7 | Pretty beim Verlassen: Schalter | übernommen (Schalter folgt E12) | Schalter vorhanden | SettingsView.svelte (`einst-pretty`) | Statusnotiz „Schalter folgt“ streichen |
| 8 | Zahlenwerkzeuge (4 Zeilen) | offen | fehlt: Befehl in Tabelle, kein Zweig, keine Ansicht | lib.rs:114; commands.ts | noch in 0.3.0 (mittel) oder bewusst 0.3.1: Entscheidung; Zahlen-Hover im Editor ist da |
| 8 | Zahlenanzeige-Voreinstellung | offen | fehlt | – | mit Zahlenwerkzeugen |
| 8 | BASIC nach Assembler (`basic_nach_asm`) | offen | fehlt: Befehl in Tabelle, kein Zweig; vorhanden sind nur die BASIC-Umwandlungen Labels/Nummern/Neu nummerieren | lib.rs:131; basicconv.svelte.ts | Handbuchbild „basic-nach-asm“ ist schon als nicht machbar gelistet; Entscheidung: 0.3.0 (mittel, Kern `basic2asm` vorhanden?) oder 0.3.1 |
| 8 | BASIC: Hex-Vorschläge (`hex_vorschlaege`) | offen | fehlt, kein Zweig | lib.rs:57 | noch in 0.3.0 (klein) oder 0.3.1 |
| 8 | Markdown umschalten, als Markdown, Format, Link/Bild, Überschriften, Listen, Tabellen (6 Zeilen) | offen | vorhanden | markdown.svelte.ts:14-19 (`OPS`, `mdUmschalten`, `mdAls`); commands.ts:58-60 | übernommen |
| 8 | Markdown-Vorschau | offen | vorhanden (Komponente) | MdPreview.svelte; markdown.svelte.ts `vorschauAn`, `blocks` | übernommen (Blockweise nicht einzeln geprüft) |
| 8 | Zeichenpanel (Steuerzeichen) | offen | fehlt als Panel; Kern-Befehl `char_names` existiert, Treffer für Panel Zeichen nicht gefunden | cmds.rs:19 `char_names` | noch in 0.3.0 (klein) oder 0.3.1; unklar |
| 9 | Exportierte Dateien im Baum markieren | offen | fehlt | FileTree.svelte (nur Symbole folder/file) | bewusst 0.3.1 (Zierde) |
| 9 | Dateiart-Symbole, Gruppenüberschriften | offen | fehlt (ein Symbol für Ordner, eins für Datei) | FileTree.svelte:36 | bewusst 0.3.1 oder klein bauen |
| 9 | Neue Datei Grafik/Musik/Daten-Arten | teilweise | Dialog `neue_datei` kennt nur asm, basic, text, notiz, keine; Sprites/Zeichensatz/Disk legt man in den Editoren an (NewSpriteDialog in SpriteView/ZeichensatzView) | editor.svelte.ts:448 | teilweise lassen; Song/Palette/Kacheln/Bildschirm neu anlegen: unklar |
| 9 | Neue leere Diskette | offen | vorhanden im Disk-Editor (nicht über `neue_datei`) | DiskView.svelte:5,134; disk.svelte.ts:48 | übernommen, Ort „Daten: Disk“ in Matrix eintragen |
| 9 | Seitenleiste ein/aus (`seitenleiste`) | offen | vorhanden | commands.ts `case "seitenleiste"` | übernommen |
| 10 | Diagnosen in „Probleme“ | offen | vorhanden | ProblemsPanel.svelte; bau.svelte.ts `problemLines(lsp.diags)` | übernommen |
| 10 | Hinweis „Definition außerhalb des Projekts“ | offen | unverändert fehlt (nur „Keine Definition gefunden“) | commands.ts `gehe_zu_definition` | bewusst 0.3.1 (Randfall) |
| 10 | Diagnosen ein/aus | offen | fehlt (kein Schalter) | SettingsView.svelte | bewusst 0.3.1 oder klein |
| 11 | Debuggen (F6) | offen (E6) | vorhanden | commands.ts `debuggen`, debug.svelte.ts `debugStarten` | übernommen |
| 11 | „Ab hier starten“ (`debug_hierher`) | offen | vorhanden | commands.ts `debug_hierher` | übernommen |
| 11 | Debuggen aus der Kopfzeile | offen | siehe Debuggen | App.svelte (Debug-Zustand) | übernommen |
| 11 | Kopfzeile in Breitenstufen | offen | nicht gefunden | – | unklar; Breitenklassen der Ansichten sind übernommen, Kopfzeile separat prüfen |
| 11 | Schublade minimieren/maximieren | übernommen (Max offen) | `schublade_max` vorhanden | commands.ts `schublade_max` | Statusnotiz streichen |
| 11 | Test-Detail Setup-Zeile | offen | fehlt (kein Treffer) | TestsPanel.svelte | bewusst 0.3.1 |
| 11 | Tests parallel zur Debug-Sitzung (Hinweis) | offen | nicht gefunden | – | bewusst 0.3.1 oder unklar |
| 11 | Statusleiste Takt und Norm | offen (E6) | nicht gefunden; Statusleiste zeigt PC, Sprache, Probleme | App.svelte:274-278 | bewusst 0.3.1 |
| 11 | Panel-Reiter Gliederung, Hilfe, Zeichen, Karte, Emulator | offen | andere Aufteilung: Panel „Maschine“; Gliederung/Suche/Probleme/Ausgabe/Tests in der Ablage | OutlinePanel, SearchPanel, MachinePanel | Matrix anpassen: „übernommen in neuer Aufteilung“; Zeichen/Karte fehlen wie oben |
| 11 | Panel-Breite je Inhalt, Überlagerung | offen | Überlagerung bei schmal vorhanden (`overlay()` in App.svelte:233) | App.svelte:233 | übernommen (Breite je Inhalt nicht geprüft) |
| 11 | Export beim Bauen | offen | fehlt (keine Einstellung) | – | bewusst 0.3.1 oder klein |
| 12 | Startseite wieder öffnen (`startseite`) | offen | vorhanden | commands.ts:`projekt_schliessen`/`startseite` | übernommen |
| 12 | Startseite: Banner „Einrichtung unvollständig“, Assistent, Feedback | zurückgestellt | Banner (`news`), Willkommen mit Assistent vorhanden; Feedback als Befehl/Dialog vorhanden | StartView.svelte:88-98; FeedbackDialog.svelte; commands.ts `feedback` | übernommen (Feedback nur über Menü) |
| 12 | Startseite: Hardware-Übersicht, Probe Hardware | teilweise | nur Emulatoren | StartView.svelte:101-108 | bewusst 0.3.1 (Hardware-Probe) |
| 12 | Startseite: neue Datei/Editor-Art | zurückgestellt | fehlt | StartView.svelte:57-59 | bewusst 0.3.1 oder entfällt |
| 12 | Sitzung je Projekt abschaltbar | teilweise | Schalter im Projekt-Bereich vorhanden | projekt/Allgemein.svelte:9,27 | übernommen |
| 12 | Absturzsicherung ein/aus | teilweise | Schalter nicht gefunden | SettingsView.svelte (kein „scratch“) | noch in 0.3.0 (klein) oder 0.3.1 |
| 12 | Ältere Kachelkarte umwandeln | offen (E8) | nicht gefunden | – | unklar; wenn Kachel-Editor „Karten“ nach 0.3.1 (D6) ist, entfällt hier |
| 12 | Rückfragen (Debug beenden, Einstellungen zurücksetzen, Konfiguration importieren) | E12 | Konfig-Import/-Export vorhanden; Debug-Beenden-Rückfrage nicht geprüft | SettingsView.svelte:108 (`konfig-*`); settings.svelte.ts:40 | übernommen (Debug-Rückfrage unklar) |

## B. Stichproben bei „übernommen“

- Alle Befehlskennungen in Backticks der Spalte „neu“ (Abschnitte 1 bis 12) gegen die Tabelle geprüft: **alle vorhanden**. Nicht in der Tabelle, aber gewollt anders gelöst: `lang:*`, `nav:*` (Präfix-Zweige in commands.ts), `ui:gallery`, `ui:measure` (commands.ts, GalleryView/MeasureView vorhanden). Dateinamen `FileTree.svelte`, `Kbd`, `keys.ts` existieren. **`Menu.svelte` ist kein Menüleiste, sondern ein Aufklappmenü** (Menüleiste ist nativ in `menu.rs`): Matrixzeile 43 sachlich korrigieren.
- Kennungen, die es nirgends gibt: **keine**.
- Kennungen in der Tabelle ohne Zweig in commands.ts (zeigen nur „noch nicht gebaut“): `als_include`, `basic_nach_asm`, `hex_vorschlaege`, `kodierung`, `zahlenwerkzeuge`, `projekt_ordner_oeffnen`, `std_zeichensatz`, `zeichensatz_rom`, `zeichensatz_png`, `studio_vergroessern`, `studio_verkleinern`, `studio_einpassen`, `studio_raster`, `studio_zwiebel`; außerdem `kopieren`/`ausschneiden`/`einfuegen` außerhalb des Sprite-Editors. Auffällig: Matrix Zeile 100 (Abschnitt 4, „Zwischenablage“) und Abschnitt 14/E3 (Zoom `studio_*`, Zeile 367, außerhalb meiner Abschnitte) stehen auf „übernommen“; über die Befehlspalette bzw. unter Windows/Linux-Menü bewirken sie nichts. Native Zwischenablage-Einträge im Mac-Menü laufen über das Webview (menu.rs), dort funktioniert es. `projekt_ordner_oeffnen` (Zeile 280 nennt `projekt_oeffnen`) ohne Zweig: prüfen, ob Menüeintrag doppelt oder tot ist; `studio_*` und Zeichensatz-Importe gehören den Abschnitten 14ff, nur der Hinweis.
- Stichprobe weiterer „übernommen“-Zeilen: Zoom (`zoom_*`), `hoher_kontrast`, `darstellung_zuruecksetzen`, Reiterbefehle, Teilen, Lesezeichen, Haltepunkt F9, Formatieren, Zeilenende, Umbruch, Bauen/Starten, Tests, Protokoll, Probleme, Layout, Beenden: Zweige in commands.ts vorhanden. Nicht inhaltlich geprüft: Verhalten im laufenden Programm.

## C. Sonstiges

- In docs/paritaet-oberflaeche.md stehen im Abschnitt „Registry (R0)“ (um Zeile 746) noch Merge-Marken `<<<<<<< HEAD`, `=======`, `>>>>>>> claude/ui-w1`. Bitte bereinigen.
- Kopf der Datei sagt „Stand: 2026-11-20“, viele Zeilen sind überholt.

### Abschnitte 13–22


Stand: Worktree tauri-sammel (HEAD b67b3aa7, losgelöst). Nur gelesen, nichts gestartet.
Hinweis: Die Datei docs/paritaet-oberflaeche.md enthält im Bereich Zeilen 735 bis 750 noch Merge-Marken (`<<<<<<< HEAD`, `=======`, `>>>>>>> claude/ui-w1`). Das liegt außerhalb von 13 bis 22, muss aber vor dem Merge bereinigt werden.

## Abweichungen und Offenes

| Abschnitt | Funktion (Kurztext) | bisheriger Status | Befund | Nachweis | Vorschlag |
|---|---|---|---|---|---|
| 13 Disk | Haltepunkt auf Sektoradresse, Sprung in Debug | offen | Zeile ist ein Zuordnungsfehler: `DisHaltepunkt` im Iced-Code ist der Haltepunkt-Klick im Disassembler (dbgview.rs:122, main.rs:987), keine Sektorfunktion. Eine Sektor-Haltepunkt-Funktion gab es nie. Der Disassembler-Haltepunkt ist neu vorhanden. | ui/src/views/DebugView.svelte:132 (`data-dbg="bp"`, `punktWeg`/`punktHinzu`); Abschnitt 17 führt ihn schon als übernommen | Zeile streichen oder umformulieren: „entfällt als Disk-Funktion (Zuordnungsfehler, siehe Abschnitt 17)“ |
| 13 Disk | Schreibschutz und „Image ungespeichert“ | teilweise (Schreibschutz der Datei nicht) | Jetzt vorhanden: Schloss-Symbol bei geschützten Dateien, dazu „ungespeichert“-Hinweis | ui/src/views/DiskView.svelte:103 (`f.locked`, `disk-geschuetzt`), :54 (`disk-ungespeichert`); crates/vc-shell/src/disk.rs:31 (`locked`) | Status → übernommen. Einschränkung: Schreibschutz-Anzeige der Dateien im Verzeichnis (Dateisystem-Flag); ein Schreibschutz der Image-Datei selbst im Dateisystem wird nicht geprüft (war in Iced auch nicht nachweisbar) |
| 13 Disk | Diskette auf MEGA65 übertragen (Ethernet) | zurückgestellt | Fehlt vollständig (kein Befehl in disk.rs, keine Schaltfläche). Iced-Text selbst: „UNGEPRÜFT, nicht an einem Gerät getestet“ | grep nach mega65net/Ethernet in vc-shell: nur einst.rs/dbgsitzung.rs (Ziele), nicht disk.rs | bewusst nach 0.3.1 (Hardwareprüfung steht aus, Inhaber entscheidet; ungeprüfte Funktion nicht ausliefern) |
| 13 Disk | Erst speichern, dann einlegen oder übertragen (Sperrhinweis) | teilweise | Einlegen: vorhanden (Sperre und Text „Erst speichern, dann einlegen“). Übertragen entfällt mit der MEGA65-Zeile | disk.rs:69 bis 71 (`disk-erst-speichern`), ui.ftl:1026 `disk-einlegen-erst`, :1028 | Status → übernommen für Einlegen; Rest hängt an der MEGA65-Zeile (nach 0.3.1) |
| 13 Disk | Diskette in Projekteinstellungen: Gruppen und Programme der Diskette | offen | Teilweise da: Abbild an/aus, Format (D64/D71/D81/ATR), Name auf der Diskette, Hinweis zur Ausgabedatei, Quellgruppen im Bereich Projekt, Quellen. Es fehlt: Auswahl, welche Programme auf die Diskette kommen (Häkchenliste `disk.files`) und das Feld „Ausgabedatei“. Kern kann es schon: `projekt_set` mit `disk.files` (api.ts:412), `ProjektInfo.disk.files`/`file` werden geliefert (projekt.rs:28) | ui/src/views/projekt/Bauen.svelte:33 bis 40 (nur Format/Name/Weg); Uebersicht.svelte:16 liest `disk.files.length` | noch in 0.3.0 bauen (klein): Häkchenliste der Programme und Ausgabedatei-Feld in Bauen.svelte, Kern fertig. Ohne Auswahl kommen laut Hinweistext „alle Programme der Quellen“ auf das Abbild, also kein Datenverlust |
| 14 Sprite | MEGA65 VIC-IV-Sprites (16/64 breit, 16-Farben-Modus) | vorgemerkt | Nicht vorhanden (keine Treffer für SPR16EN/SPRX64EN in crates und ui/src) | grep ohne Treffer | bewusst nach 0.3.1 (Entscheidung des Inhabers steht in der Zeile; braucht Profile, Export, Vorlagen, Beispiele) |
| 15 Zeichensatz | Zeichensatz-Datei öffnen (256 Zeichen, 16 × 16) | offen | Vorhanden | ui/src/views/ZeichensatzView.svelte (Dateiliste, Öffnen), sprite.svelte.ts:23 bis 47 (`istZeichensatzDatei`, `openSprite`), CharGrid.svelte | Status → übernommen |
| 15 Zeichensatz | Zeichen bearbeiten mit allen Werkzeugen | offen | Vorhanden (Stift, Radierer, Pipette, Füllen, Linie, Rechteck, Auswahl, Rückgängig, Spiegeln, Verschieben, Raster, Zoom, Speichern, Export) | ZeichensatzView.svelte:23 (`tools`), Werkzeugleiste Zeilen 87 bis 105 | Status → übernommen |
| 15 Zeichensatz | Zeichenwahl im Raster mit Code in Dezimal und Hex | offen | Vorhanden: Raster mit Auswahl, Chip „Code dez/hex“, Karte „Nummer“ und „Position“ | ZeichensatzView.svelte:76 (CharGrid), `zeichensatz-chip`, Karte Zeichen | Status → übernommen (neu dazu: Bereiche und Vorschautext) |
| 15 Zeichensatz | Herkunft des Zeichensatzes anzeigen | offen | Vorhanden | ZeichensatzView.svelte (`data-cs="herkunft"`), pixel.rs:86 bis 99 (`herkunft`) | Status → übernommen |
| 15 Zeichensatz | Hinweis „ROM fehlt“ bei Verweis ohne ROM | offen | Teilweise: In der Vorlagenliste erscheint „ROM nicht gefunden“ und der Eintrag ist gesperrt. Der Kern liefert beim Öffnen außerdem den Text „Zeichensatz-ROM fehlt – leeres Raster“ in `view.status`, aber keine Ansicht zeigt `status` an (ZeichensatzView hat keinen Verweis auf `v.status`; nur `note` wird als Toast gezeigt). Unklar, ob das Feld an anderer Stelle gezeigt wird: grep nach `view.status`/`v.status` in ui/src ergab keinen Treffer | ZeichensatzView.svelte:131 (`zeichensatz-vorlage-fehlt`); vc-ide-model/src/pixel.rs:72 | noch in 0.3.0 bauen (klein): `v.status` als Hinweiszeile im Zeichensatz-Editor anzeigen (ein Absatz). Bis dahin Status „teilweise“ |
| 15 Zeichensatz | Zeichensatz-Vorlage laden (rückgängig machbar) | offen | Vorhanden, Laden ist ein Verlaufsschritt; dazu „Original zurück“ je Zeichen | ZeichensatzView.svelte:127 bis 148, pixel.rs:243 bis 253 (`vorlage`, `zeichen_original`), Test pixel.rs:279 | Status → übernommen |
| 15 Zeichensatz | Standardzeichensatz hinzufügen (`std_zeichensatz`) | offen | Befehlskennung steht in vc-commands (lib.rs:58) und im Katalog (cmd.ftl), aber `commands.ts` hat keinen `case` dafür, der Aufruf endet im `default` mit Toast „noch nicht“ (commands.ts:191). Kein Dialog, keine Kernfunktion (kein `std_charset_list` im vc-shell). Teilersatz: Neuer Zeichensatz über „Neu“ (media_new) plus Vorlage laden deckt „Satz aus ROM/VC-Standard anlegen“ halb ab | commands.ts (kein case), grep nach `std_zeichensatz` in ui/src und vc-shell: kein Treffer | noch in 0.3.0 bauen. Größe: mittel, nicht klein (Dialog mit Liste der Standardsätze je Zielsystem, Verweis oder Einbetten, Name; Kern `vc_media::project::std_charset_list`/`eigene_saetze` liegt fertig vor). Falls nicht: Matrix ehrlich auf „offen, 0.3.1“ setzen und den Befehl ausblenden. Empfehlung: bauen, weil das ohne ROM-Pflicht der Weg zu den freigegebenen eigenen Sätzen (CC0) ist |
| 15 Zeichensatz | Zeichensatz aus ROM lesen (`zeichensatz_rom`) | offen | Wie oben: keine Behandlung in commands.ts, kein Dialog, kein Kernbefehl | wie oben | Empfehlung: zusammen mit dem Standard-Dialog (zweiter Reiter) in 0.3.0; alternativ bewusst nach 0.3.1. Rechtlich nur ROM-Dateien des Anwenders (Memory: Anwaltsangabe zu ROM-Sätzen ungeprüft), deshalb vertretbar zu verschieben, wenn der Zeitplan eng ist |
| 15 Zeichensatz | Zeichensatz aus PNG-Raster (`zeichensatz_png`) | offen | Befehl ohne Behandlung. Es gibt nur „PNG laden“ in das gewählte Zeichen/Frame (`png`-Op, pixel.rs:221, sprite.svelte.ts:130), nicht das 16-Zeichen-je-Zeile-Raster mit Invertierung und neuer Datei | pixel.rs:221 bis 224 (`png_importieren`, ein Frame) | noch in 0.3.0 bauen (mittel) oder bewusst nach 0.3.1. Handbuchbild `zeichensatz-png-import` ist bereits in der Liste der nicht machbaren Bilder, hängt also an dieser Entscheidung |
| 15 Zeichensatz | Zeichensatzprofil und Name prüfen | offen | Teilweise: Profil und Name werden im Dialog „Neu“ geprüft (`NewSpriteDialog`, Namensregel, Profil). Der Prüfteil des Iced-Dialogs „Hinzufügen“ fehlt mit diesem Dialog | NewSpriteDialog.svelte:12 bis 16 | Status → teilweise; folgt dem Dialog zu `std_zeichensatz` |
| 15 Zeichensatz | ROM-Datei erkannt/nicht erkannt (Vorschläge nach Größe) | offen | Fehlt (gehört zum ROM-Reiter). Die Einstellungsseite Roms (einst/Roms.svelte, `roms_import_plan`) erkennt ROMs, aber nicht für diesen Zweck | views/einst/Roms.svelte | folgt der Entscheidung zu `zeichensatz_rom` |
| 17 Debug | Chip-Fenster: Sprites, Bildschirmcode, Zeichensatz im ROM (mehrfarbig) | „bisher nicht angeschlossen; Erkennen … zeigt Bildschirm, Zeichensatz, Sprites“ | Grafisches Chip-Fenster fehlt. Vorhanden: Chip-Register mit Werten im Halt (DebugView.svelte:101 bis 107) und die Momentaufnahme-Erkennung mit Adresse von Bildschirm, Zeichensatz, Spritezahl (DebugView.svelte:86 bis 92). In der alten Oberfläche war es nach der Matrix selbst nicht angeschlossen | DebugView.svelte:86 bis 107 | bewusst nach 0.3.1 (war auch in Iced nicht angeschlossen, Ersatz durch Momentaufnahme und Chip-Register vorhanden). Matrixtext entsprechend auf „nach 0.3.1, kein Altbestand“ ändern |

## Zeilen ohne Befund (offen/teilweise, bleiben so)

Bewusst entfallen oder vorgemerkt, Begründung steht in der Matrix und trifft zu:
- 14: „Vorlage laden (Sprite, Zeichensatz)“ entfällt für Sprites: stimmt, `loadTemplate` kommt nur im Zeichensatz-Editor vor.
- 14: „Karten der Seitenspalte ein-/zuklappen“ entfällt: Karten stehen offen (Muster).
- 15: Tastaturlayout-Wahl entfällt (Tastencode): kein Widerspruch gefunden.
- 16: „Spielroutine für den 6502“ kommt 0.3.1: kein Altbestand, passt zu „bewusst nach 0.3.1“. Keine Änderung.
- 17: „Debug-Fläche in der Schublade“ ersetzt, „Trennlinie“ entfällt: stimmig.
- 19: „Zoom, Einpassen, Verschieben“ entfällt; 20: „Zoom, Einpassen, Verschieben“ entfällt (passt sich der Breite an): nicht widerlegt. Zu prüfen im Handtest, ob bei sehr großen Karten/Bildschirmen die Breitenanpassung reicht. Im Abschnitt 21 steht dagegen „Zoom, Einpassen, Verschieben“ als fertig (Editor-Zoom).
- 21: Karten-Editor steht auf „fertig“, obwohl D6 ihn „bewusst nach 0.3.1“ nennt. Widerspruch in den Texten: Die Ansicht existiert (ui/src/views/KartenView.svelte, crates/vc-shell/src/karten.rs, Navigation vorhanden). Klären, ob D6 für den Karten-Editor gestrichen wurde (dann Eintrag in D6 aktualisieren) oder ob 0.3.1 nur die Veröffentlichung meint. Status in der Matrix selbst nicht ändern, aber den Satz in „Entscheidung D6“ prüfen.

## Stichproben und Befehlskennungen

Alle in den Abschnitten 13 bis 22 genannten Kennungen in Backticks in Spalte „neu“ (17 Stück) geprüft gegen crates/vc-commands/src/lib.rs, `case` in ui/src/lib/commands.ts:

| Kennung | vc-commands | commands.ts-case |
|---|---|---|
| debug_weiter, debug_pause, debug_ueber, debug_hinein, debug_raus, debug_ende, debug_hierher | ja | ja |
| tracker_noten, tracker_instrumente, tracker_ablauf | ja | ja |
| rueckgaengig, wiederholen, speichern, exportieren | ja | ja |
| std_zeichensatz, zeichensatz_rom, zeichensatz_png | ja | NEIN (Toast „noch nicht“) |

Nicht existierende Kennungen: keine. Aber drei Kennungen existieren nur als Tabelleneintrag und Menütext, ohne Handler (siehe Zeilen oben). Das ist bei „offen“ konsistent, sollte aber im Katalog nicht sichtbar bleiben, wenn nicht gebaut wird (Menü/Palette zeigt sie, Aufruf liefert nur „noch nicht“).

Hinweis zur Schreibweise: Die Matrix nennt für Editor-Zoom/Raster/Onion Skin keine Kennung. Vorhanden sind `studio_vergroessern`, `studio_verkleinern`, `studio_einpassen`, `studio_raster`, `studio_zwiebel` (lib.rs, Gruppe „Ansicht“). Der Kennungsname `studio_zwiebel` enthält das Wort „Zwiebel“ (Memory: überall „Onion Skin“). Nur interner Bezeichner, kein Anzeigetext; zur Kenntnis.

Stichproben „übernommen“ (jede 5. Zeile, plus Auswahl) bestätigt ohne Abweichung:
- 13: Diskettenabbild öffnen, Belegungskarte (DiskMap.svelte), Verzeichnis, Sektorkette folgen/zurück (`follow`/`back`, disk.rs:62 bis 63), Validieren (`validate`), Vormerken-Funktionen (`check_all`, `check_none`, `check_prg`, `check_basic`), Übernehmen (`import`), Einlegen (`insert`).
- 14: Werkzeuge, Frames, Animationen (AnimSteps, AnimTimeline, AnimPreviewDialog), PNG laden (`png`), Onion Skin, Player/Missile (`pmg_size`, `pmg_prior`, pixel.rs:226).
- 16: Tracker mit Konvertieren (3 Treffer in TrackerView), RMT-Import (TrackerView 4, tracker.svelte.ts 7), `song_quelle_pick`/`song_import_run` in api.ts und cmds.rs, Kanäle/Pegel/Wiedergabe in tracker.rs/audio.rs vorhanden.
- 17: DebugView mit Haltepunkten, Beobachten, Momentaufnahme, Chip-Register, Emulator-Bild (MachinePanel.svelte), `debug_start`, `debug_cmd`, `debug_frame`.
- 18: Palette „Im Einsatz“ (PaletteView.svelte:87).
- 19 bis 22: KachelView, BildschirmView (Zerlegen: `zerlegen_ende`, BildschirmView.svelte:76), KartenView, HexView (Im Standardprogramm öffnen `data-hx="extern"`, HexView.svelte:91), SpeicherView (Filter/Preset: 8 Treffer).
- Nicht untersucht in der Tiefe: Zeilen 18 bis 22 mit „übernommen (E…)“ außer den oben genannten; keine Unstimmigkeit sichtbar, aber nicht jede Zeile einzeln im Code geprüft (Stichprobe wie beauftragt).

## Zahlen

- Zeilen in 13 bis 22: rund 260 Tabellenzeilen (13: 23, 14: 38, 15: 12, 16: 51, 17: 36, 18: 8, 19: 22, 20: 15, 21: 20, 22: 12).
- Zeilen mit Status ungleich „übernommen/fertig/neu“ (inkl. entfällt/zurückgestellt/vorgemerkt/ersetzt/teilweise/offen): 41. Davon „offen“: 14 (13: 2, 15: 11, 17: 0 offen), „teilweise“: 3, „zurückgestellt/vorgemerkt/kommt 0.3.1“: 3, „entfällt/ersetzt“: 8 (nicht widerlegt), Rest übernommen mit Zusatztext.
- Davon inzwischen vorhanden (Status → übernommen): 7 (Disk Schreibschutz; Disk Sperrhinweis Einlegen; Zeichensatz: Datei öffnen, bearbeiten, Zeichenwahl, Herkunft, Vorlage laden). Disk-Haltepunkt zählt als Zuordnungsfehler (nicht „fehlend“).
- Teilweise vorhanden: 3 (Disk in Projekteinstellungen; Zeichensatz „ROM fehlt“; Profil/Name prüfen).
- Wirklich offen: 5 Zeilen Zeichensatz-Import (`std_zeichensatz`, `zeichensatz_rom`, `zeichensatz_png`, ROM-Erkennung, Name/Profil im Dialog), dazu MEGA65-Übertragung (zurückgestellt), VIC-IV-Sprites (vorgemerkt), Chip-Fenster grafisch (nach 0.3.1).
- Nicht existierende Befehlskennungen: 0. Kennungen ohne Handler: 3.

### Abschnitte 23 bis Ende


Grundlage: nur Lesen. Vorschläge: „→ übernommen“ = Matrix aktualisieren; „0.3.0 klein“ = noch bauen; „0.3.1“ = bewusst verschieben; „entfällt“ = streichen mit Grund.

## A. Zeilen mit Abweichung oder Offenem

| Abschnitt | Funktion | bisheriger Status | Befund | Nachweis | Vorschlag |
|---|---|---|---|---|---|
| 23 | Menüleiste im Fenster anzeigen | offen | Fehlt: es gibt nur einen Hinweistext, keinen Schalter. Die Menüleiste ist fest (Mac: Systemmenü). | `SettingsView.svelte:68` (nur Text `einst-menueleiste-*`); Iced: `Einstellungen.menueleiste`, `main.rs:676` | 0.3.0 klein (Schalter + Einstellung) oder entfällt (Grund: Menü fest, Zen-Modus ersetzt es). Entscheidung Inhaber nötig |
| 23 | Takt-Anzeige in der Statusleiste | offen | Fehlt. Statusleiste zeigt Projekt, Sprache, PC beim Debuggen, Bau, Speicherstreifen; keinen Takt, Norm, Zeilen oder Zyklen. Der Kern liefert die Werte nicht. Iced-Standard war aus. | `App.svelte:274-286`; Iced `einstellungen.rs:22` (`takt_anzeige`) | 0.3.1 (Anzeige, Standard aus; Zyklen je Zeile gibt es ohnehin am Editorrand) |
| 23 | Updates: nach Updates suchen, „Jetzt prüfen“ | teilweise (E12.1) | Vorhanden: Wahl, Adresse und „Jetzt prüfen“ in Einstellungen ▸ Info (E12.4 gebaut). | `SettingsView.svelte` (`info-jetzt-pruefen`, `updatePruefen`); Befehl `update_pruefen` | → übernommen (E12.4) |
| 23 | Hilfe-Bilder laden (Adresse oder Ordner) | offen | Vorhanden: Karte „Hilfe“ mit Wahl immer/nie/fragen und Adresse. Ordner-Auswahl nur über die Adresse. | `SettingsView.svelte` (`einst-hilfe-bilder`, `einst-hilfe-url`); Abschnitt 24 meldet es schon als übernommen | → übernommen |
| 23 | Einrichtung: Assistent starten, Stand | offen | Vorhanden: Karte mit Schnell, Alles selbst und den vier Teilassistenten; Stand als Hinweis in der Statusleiste. | `SettingsView.svelte` (`wizStarten`); `App.svelte` (`status-einrichtung-hinweis`) | → übernommen (E12.3) |
| 23 | Emulator portabel herunterladen und installieren | offen (E12.2b) | Vorhanden, mit Rückfrage vor dem Laden (Version, Größe, Quelle, Ziel). | `EmuInstall.svelte`; Kern `emu_pakete`, `emu_install` (`cmds.rs:391-392`, `einst.rs:152`) | → übernommen (E12.2b) |
| 23 | Argumente: Parametereditor (Katalog, Wert, Schalter) | offen (E12.2b) | Vorhanden für Emulatoren und Assembler (Toolchain-Seite). | `ArgsEditor.svelte`; `args_info` (`cmds.rs:381`); Einbindung `SettingsView.svelte` (Toolchain) und `Ziele.svelte` | → übernommen (E12.2b) |
| 23 | Argumente: freies Feld, Platzhalter, Variablen | offen (E12.2b) | Vorhanden (Feld „weitere“, Knöpfe `{name}`). | `ArgsEditor.svelte` (`data-args=free`, `ph`) | → übernommen |
| 23 | Argumente: Standard, Migration, Vorschau | offen (E12.2b) | Vorhanden. | `ArgsEditor.svelte` (`zurueck`, `migrieren`, `vorschau`); `args_migrate` (`cmds.rs:382`) | → übernommen |
| 23 | ROMs: Datei oder Zip wählen, Ablegen ins Fenster | offen (E12.2b) | Vorhanden: Dateidialog und Ablegen auf der ROM-Seite, mit Plan vor dem Kopieren. | `settings.svelte.ts:80-81` (`romsWaehlen`, `romsAbgelegt`); `ablegen.svelte.ts:25`; `roms_pick`, `roms_import_plan/run` | → übernommen |
| 23 | ROMs: Fundliste kopieren (nur Auswahl, Alle, Keine) | offen (E12.2b) | Teilweise. Der Import-Plan hat Häkchen je Eintrag und „Ausführen (n)“. Es fehlen die Knöpfe „Alle wählen“ und „Alle abwählen“ (Iced: `RomAkt::Alle`). „Liste kopieren“ kopiert den ROM-Stand in die Zwischenablage. | `Roms.svelte` (`data-rm=haken`, `copylist`); Iced `romui.rs:108,189` | 0.3.0 klein (zwei Knöpfe) |
| 23 | ROMs: portabler Modus (Markierungsdatei) | offen (E12.2b) | Kern wertet ihn aus (Stufe „Portable“ in der Suchliste sichtbar). Die Statuszeile „Portabler Modus: aktiv/aus“ der Iced-Seite fehlt; `roms_info` liefert sie nicht. | `crates/vc-roms/src/env.rs:4`; `einst.rs:105` (`searched`); Iced `romui.rs:133-138` | 0.3.0 klein (ein Feld und eine Zeile) |
| 23 | ROMs: Auswahl in roms.json gespeichert | offen (E12.2b) | Vorhanden: Auswahlfeld „festnageln“ je Rolle. | `Roms.svelte` (`data-rm=pin`); `roms_pin` (`einst.rs:129`, `cmds.rs:406`) | → übernommen |
| 23 | Pfadfeld mit Dateidialog (alle Pfadfelder) | teilweise (E12.1) | Nur Projektordner hat Ordnerdialog (`dir_pick`). Die Emulator-Pfade in `Ziele.svelte` haben keinen Dateidialog („waehlen/pick“ kommt dort nicht vor). | `settings.svelte.ts:38`; `Ziele.svelte` (0 Treffer) | 0.3.0 klein (Datei-Dialog am Emulatorpfad) |
| 23 | Assistent: Projektordner, Beispiele, „Erstes Projekt“ | übernommen (E12.3; „Erstes Projekt“ folgt) | Zusatz „folgt“ ungeklärt. Im Quellcode nicht geprüft, ob der Schritt „Erstes Projekt“ vorhanden ist. | `WizardView.svelte`, `wiz.rs` (nicht vertieft) | unklar: Stand des Teils „Erstes Projekt“ vor der Abnahme klären |
| 23 | Projekteinstellungen: Name, Beschreibung, Projektbild | offen | Name und Beschreibung vorhanden; das Projektbild (Schalter „Vorschau“, Bild wählen, löschen) fehlt. Die Startseite zeigt `projektbild.png`, die Einstellung dazu fehlt. Befehl `projekt_einstellungen` besteht. | `projekt/Allgemein.svelte`; `start.rs:11-24`; Iced `projeinst.rs` (`ProjektBild`, `BildWaehlen`, `BildLoeschen`) | 0.3.0 klein bis mittel oder 0.3.1 (Bild ist selten, Datei lässt sich von Hand ablegen). Empfehlung 0.3.1 |
| 23 | Projekteinstellungen: Ziele (Videonorm, Speicher, Emulator, Start) | offen | Teilweise. Videonorm, Startziel und illegale Befehle sind einstellbar; Ziele hinzufügen und entfernen gibt es. Es fehlen der Speicherausbau (VIC-20) und die Wahl des Standard-Emulators je Ziel. Der Kern kennt `defaultEmulator` (`projekt.rs:111`), die Oberfläche zeigt ihn nur als Text. Emulator-Art (vice, atari800, xemu) fehlt. | `projekt/Ziele.svelte`, `projekt/Bauen.svelte` (Text); `projekt.rs:71,76,111`; Iced `projeinst.rs:57-61,117-123` | 0.3.0 klein (Auswahl Standard-Emulator und Speicher); Emulator-Art 0.3.1 |
| 23 | Projekteinstellungen: Quellgruppen (Art, Ziel, Dialekt, Reihenfolge) | offen | Art, Ziele, Dialekt, Gruppe hinzufügen und entfernen sind vorhanden. Die Reihenfolge der Gruppen untereinander fehlt (Iced `GruppeAuf/Ab`); Dateien lassen sich nur innerhalb ihrer Gruppe verschieben. | `projekt/Quellen.svelte`; `projekt.rs:58,81-96` | 0.3.0 klein (Gruppe hoch/runter) |
| 23 | Projekteinstellungen: Umwandlung BASIC nach Assembler, Laufzeit, Fließkomma | offen | Fehlt. Weder Oberfläche noch `projekt_set` kennen `convert` (Ziel, Ausgabe, int_div, float, track_line); Kern `projfile` unterstützt es, ein Projekt aus der Iced-Fassung baut also weiter. Der Befehl `basic_nach_asm` ist nur der Dialog, nicht die Bauoption. | `projekt.rs` (nur Zeile 93 löscht `convert`); Iced `projeinst.rs:70-73,139` | 0.3.0 mittel (sonst nur von Hand in `.vcprj`) oder bewusst 0.3.1. Entscheidung Inhaber |
| 23 | Projekteinstellungen: Diskette (Format, Programme, Name) | offen | Teilweise. Format (D64, D71, D81, ATR), Name, an/aus unter Projekt ▸ Bauen und Starten ▸ Abbild. Die Programmauswahl (`disk.files`) und Dateiname fehlen in der Oberfläche; der Kern nimmt `files` an. Die Seite „Diskette“ zeigt nur die Turbo-Karte. | `projekt/Bauen.svelte:20-40`; `projekt.rs:113-122`; `ProjektView.svelte` | 0.3.0 klein (Programmliste) oder 0.3.1; Nachweis vorhanden für den Rest → „teilweise“ |
| 23 | Projekteinstellungen: Hauptdatei, Dateien je Gruppe hinzufügen und ordnen | offen | Vorhanden: Startdatei (Allgemein und „Als Start“), Datei hinzufügen, entfernen, hoch, runter. | `Allgemein.svelte`, `Quellen.svelte` (`data-pj=datei-hinzu`, `data-pj-hoch`, `data-pj-start`); `projekt.rs:58,97` | → übernommen |
| 23 | Projekt kopieren (Name, Ort, Bauordner, Git, danach öffnen) | offen | Vorhanden, dazu Zielsystem wählen. | `CopyDialog.svelte`; Befehl `projekt_kopieren` (`commands.ts`, `lib.rs`) | → übernommen |
| 23 | Import: Dateien ins Projekt übernehmen (Ordner, Name, Konflikt) | offen | Teilweise. Ablegen aus dem Dateimanager mit Rückfrage „Ins Projekt kopieren und öffnen?“ (`drop_copy`); bei Namensgleichheit wird automatisch umbenannt (Zähler). Ein Dialog mit Zielordner, Name und Wahl ersetzen/überspringen fehlt. Datei hinzufügen am Disk-Editor hat eigenen Weg. | `ablegen.svelte.ts:41-49`; `ablegen.rs:43-56`; Iced `importdlg.rs` | 0.3.1 (der Weg über Ablegen deckt den Normalfall) oder klein: Dialog. Entscheidung Inhaber |
| 23 | Export-Dialog: Format (ASM, INC, PRG, BAS, Hex, Binär) und Name | offen | Vorhanden mit asm, inc, bin, prg und „DATA“ (BASIC-DATA-Zeilen, auf Wunsch mit Einleseroutine). Hex als Vorschau-Reiter statt Format; reines BAS-Format fehlt (ersetzt durch `data`). Befehl `exportieren` (Cmd+Umschalt+E). | `ExportDialog.svelte:15`; `export.rs:10` (`FORMATE`) | → übernommen (E12) mit Vermerk: BAS durch DATA ersetzt; sonst unklar, ob Iced-BAS = DATA |
| 23 | Export: Ladeadresse prüfen (PRG, INC) | offen | Vorhanden: Adressfeld mit Fehlermeldung. | `ExportDialog.svelte` (`braucht`, `data-ex=addr`, `aria-invalid`) | → übernommen |
| 23 | Export: Vorschau Text und Daten, Größe in Byte | offen | Vorhanden (Text und Hex-Reiter, Dateiliste mit Byte). | `ExportDialog.svelte` (`data-ex=vorschau`, `export-bereit`) | → übernommen |
| 23 | Export: im Projekt speichern, in Quelltext einfügen, Testlauf | offen | Vorhanden: Speichern, Als Include einfügen, In Quelltext kopieren, „Testen“ im Emulator. | `ExportDialog.svelte` (`save`, `include`, `copy`, `test`) | → übernommen |
| 23 | Info: Herkunft und Lizenzen (Tabelle eigener Zeichensätze) | teilweise (E12.4) | Drittanbieter-Hinweise und Danksagungen vorhanden; die Tabelle „Eigene Zeichensätze (VC-Standard)“ fehlt (Katalogschlüssel `infodlg-eigene-zeichensaetze-vc-standard` ist da, im Kern `info.rs` und in der Oberfläche unbenutzt). | `SettingsView.svelte` (Info); `crates/vc-i18n/catalog/de/infodlg.ftl:20,26` | 0.3.0 klein (Rechtsstand: Zeichensatzherkunft sichtbar machen) |
| 24 | Handbuchbilder erzeugen | offen (Bildskript) | Wie angegeben: Dokumentation, nicht Oberfläche; 10 nicht machbare Bilder bekannt. | `docs/handbuch/BILDER-OFFEN.md`, `docs/handbuchbilder-neue-oberflaeche.md` | unverändert (nicht neu berichtet) |
| 25 | Datei: 18 Befehle | offen | Zahl stimmt: 18. Alle in `vc-commands`. Ausführung: s. Abschnitt B (`std_zeichensatz`, `zeichensatz_rom`, `zeichensatz_png`, `basic_nach_asm`, `projekt_ordner_oeffnen` ohne Ausführung). | `lib.rs` `BEFEHLE`; `commands.ts` | Status „teilweise“, bis B erledigt |
| 25 | Bearbeiten: 40 Befehle | offen | Echt: 52 (+12 neu gegenüber Iced: `basic_in_labels`, `basic_in_nummern`, `basic_neu_nummerieren`, `kommentar_umschalten`, `korrigieren`, `leerzeichen_entfernen`, `lesezeichen_umschalten`, `naechstes_lesezeichen`, `voriges_lesezeichen`, `png_laden`, `zeilen_sortieren`, `zur_klammer`). Ohne Ausführung: 16 `md_*` laufen über `mdBefehle`; ohne Ausführung sind `hex_vorschlaege`, `zahlenwerkzeuge`, `kodierung`. | `lib.rs`; `commands.ts` | Zahl auf 52 korrigieren; Status „teilweise“ |
| 25 | Ansicht: 36 Befehle | offen | Echt: 37 (+`projekt_bereich`). Ohne Ausführung: `studio_vergroessern`, `studio_verkleinern`, `studio_einpassen`, `studio_raster`, `studio_zwiebel`. | `lib.rs`; `commands.ts` | Zahl auf 37; Status „teilweise“ |
| 25 | Projekt: 22 Befehle | offen | Echt: 26 (+`projekt_schliessen`, `turbo_bereitstellen`, `turbo_pruefen`, `turbo_im_disk_editor`). Alle haben Ausführung. | `lib.rs`; `commands.ts` | Zahl auf 26 → übernommen |
| 25 | Hilfe: 11 Befehle | offen | 11, stimmt. Alle ausführbar (`hilfe`, `hilfe_reiter`, `ueber`, `feedback`, `update_pruefen`, `assistent`, `assistent_neues/roms/emulatoren/hardware/design`). | `commands.ts` | → übernommen |
| 25 | Baum: 10 Befehle | offen | Echt: 11 (+`disk_oeffnen`). Ohne Ausführung: `als_include`. | `lib.rs`; `commands.ts` | Zahl auf 11; Status „teilweise“ |
| 25 | Gesamtzahl 137 | – | Echt: 155 Einträge (18+52+37+26+11+11). Kein Iced-Befehl fehlt; 18 sind neu. | Vergleich der Kennungen `vc-gui-ui/src/befehle.rs` gegen `vc-commands/src/lib.rs` | Zahl nachtragen |
| 14 (Mindestliste) | Einrückungslinien, Leerzeichen und Zeilenenden anzeigen | Zeilenumbruch übernommen, Rest offen | Bleibt offen: keine Treffer für Einrückungslinien oder sichtbare Leerzeichen in `ui/src`. | Suche in `ui/src/lib/cm*.ts`, `CodeEditor.svelte` | 0.3.1 (Wunsch, kein Iced-Bestand) |
| 14 | Signaturhilfe | offen | Bleibt offen: keine Treffer in `cm-lsp.ts`, `lsp.rs`. | wie oben | 0.3.1 |
| 14 | Snippets | offen, Vorschlag | Bleibt offen. | wie oben | 0.3.1 |
| 14 | Farbvorschau bei Farbregistern | offen (E1.2 Rückstand) | Bleibt offen. | `cm.ts` ohne Farbvorschau | 0.3.1 |
| 14 | Bauen, Probleme anklicken, „Ab hier starten“, Zyklenzahl je Zeile | offen | Vorhanden: Bauen/Probleme-Panel, Zyklenzahl am Rand, `debug_hierher`. | `cm.ts:19,110`; `ProblemsPanel.svelte`; `commands.ts:86` | → übernommen (E1.3) |
| Registry | Zeile „Vorlagen, Zielsysteme, Paletten, Zeichensätze über Suchpfade“ | zurückgestellt | Nicht geprüft, bewusst zurückgestellt; passt zu D6. | – | unverändert |
| Registry | Datei enthält Merge-Marken | – | `<<<<<<< HEAD`, `=======`, `>>>>>>> claude/ui-w1` stehen noch in der Datei (nach Zeile „Bauen, Probleme anklicken…“ und vor „Entscheidung D6“). Die Zeile zu „Dateien ziehen“ (W1) steht außerhalb jeder Tabelle. | `docs/paritaet-oberflaeche.md`, ab Zeile ca. 735 | Auflösen: beide Teile behalten; W1-Zeile in Tabelle „Registry“ oder Abschnitt 22 verschieben |
| Projekt (P1–P4) | Zielsysteme und Varianten | teilweise (Ziele hinzufügen/entfernen offen) | Ziele hinzufügen und entfernen vorhanden. | `projekt/Ziele.svelte` (`ziel-hinzu`, `ziel-weg`); `projekt.rs:71,76` | → übernommen |
| Projekt | Quellen in Baureihenfolge, fehlende Dateien rot | offen | Vorhanden (Nummern, Wort „fehlt“, Farbe, Stern). | `projekt/Quellen.svelte` | → übernommen (P3) |
| Projekt | Bau-/Emulator-Einstellungen, Pakete mit Herkunft | offen | Vorhanden: letzter Bau, Bauen/Starten, Emulator je Ziel (nur Anzeige, Wahl siehe oben), Pakete mit Herkunft. | `projekt/Bauen.svelte` | → übernommen (P3) mit Vermerk: Emulatorwahl je Ziel nur Anzeige |
| Projekt | Export/Kopieren, Zuletzt-Verlauf | offen | Kopieren und Verlauf vorhanden; Export steht nur als Hinweistext (Export liegt in den Editoren und im Dialog `exportieren`). | `projekt/Weitergeben.svelte` | → übernommen (P3), Export-Karte ggf. mit Knopf `exportieren` ergänzen (klein) |
| 23 | „Zustimmung und Sicherer Modus“ | kommt 0.3.1 | Nicht vorhanden, wie festgelegt. | `SEITEN` in `settings.svelte.ts` | unverändert |

## B. Befehle in der Tabelle ohne Ausführung (Befund aus Abschnitt 25)

Diese 14 Kennungen stehen in `crates/vc-commands/src/lib.rs`, kommen aber in `ui/src` weder in `commands.ts` (`runCommand`) noch sonstwo vor. Beim Aufruf erscheint die Meldung „folgt in einer späteren Etappe“ (`toast-not-yet`, `commands.ts:~202`):

`als_include`, `basic_nach_asm`, `hex_vorschlaege`, `kodierung`, `projekt_ordner_oeffnen`, `std_zeichensatz`, `zeichensatz_rom`, `zeichensatz_png`, `zahlenwerkzeuge`, `studio_vergroessern`, `studio_verkleinern`, `studio_einpassen`, `studio_raster`, `studio_zwiebel`.

Einschätzung: unklar, ob die Editoren die `studio_*`-Befehle über einen anderen Weg (Tastatur im Editor) bedienen; im Code gibt es dazu keinen Verweis auf die Kennungen. Vorschlag: je Befehl entweder anbinden (0.3.0, klein bis mittel: `projekt_ordner_oeffnen`, `studio_*`, `als_include`, `kodierung`) oder aus Menü/Palette ausblenden; `basic_nach_asm` hat einen Dialog (`openConv` in `commands.ts` importiert), nur die Kennung ist nicht verdrahtet (klein). Zeichensatz-Befehle und Zahlenwerkzeuge (Zeichensatz-Editor hat eigene Knöpfe): anbinden oder ausblenden.

## C. Stichproben (Zeilen „übernommen“ und Befehlskennungen)

- Alle Befehlskennungen in Backticks der Spalte „neu“ (Abschnitte 23 bis 25): `einstellungen`, `assistent`, `assistent_*` (neues, roms, emulatoren, hardware, design), `projekt_einstellungen`, `projekt_kopieren`, `exportieren`, `ueber`, `feedback`, `update_pruefen`, `hilfe`, `hilfe_reiter`, `umbruch`. Alle existieren in `vc-commands`; keine nicht existierende Kennung gefunden.
- Alle Iced-Kennungen aus `befehle.rs` (137) kommen in `vc-commands` vor; zusätzlich 18 neue. Fehlende Kennungen: keine.
- Aus den Tabellen mit Kennungen in der Spalte „neu“ ist `assistent_neues` (Abschnitt 23) und `hilfe_reiter` (24) ebenfalls vorhanden und ausführbar.
- Stichprobe „übernommen“ (jede 5. Zeile und mehr): Einstellungen als Reiter, Sprache/Zoom/Startbild, Debug-Protokoll, Zielsysteme, Startziel, Emulator eintragen/suchen/testen, Hardware-Verbindungstest, ROM-Stand, Prüfsummen, Toolchain, Tastenkürzel, Export/Import/Zurücksetzen der Konfiguration, Einstellungsdatei zeigen, Assistent (Schnell, Alles, Teile), Info, Feedback, Update (später, überspringen): alle im Code nachgewiesen (`SettingsView.svelte`, `Ziele.svelte`, `Roms.svelte`, `WizardView.svelte`, `wiz.rs`, `info.rs`).
- Stichprobe Hilfe 24 (alle 9 Zeilen gelesen, Bilder ausgenommen): `hilfe`, `hilfe_reiter`, Hilfe-Panel im Bereich Hilfe vorhanden (`HelpView.svelte`, `hilfe.rs`, `commands.ts`).
- Stichprobe Mindestliste „übernommen“: Zeilenumbruch (`umbruch`), Fehlersprung (`naechstes_problem`, `voriges_problem`), Lesezeichen, Zur Klammer, Zeilen sortieren, Leerzeichen entfernen, Kommentar umschalten, Falten: Befehle und `case`-Zweige vorhanden.
- Abschnitt „Registry“: alle Zeilen „übernommen“ nur nach Aufschrift geprüft (R0), nicht im Code vertieft; `ext:`-Befehle laufen in `commands.ts` in die Meldung „Werkzeuge folgen mit E2“ (passt zur Aufschrift).
- Abschnitt „Entscheidung D6“: Text, keine Zeilen; Seite „Erweiterungen“ fehlt in `SEITEN` (passt).

## D. Zahlen

- Geprüfte Tabellenzeilen: 23: 68, 24: 9, 25: 6, 14: 33, Registry: 7, Projekt: 8; zusammen 131.
- Davon Status nicht „übernommen/fertig/neu“ (offen, teilweise, Leerstand): 52 (23: 30; 25: 6; 14: 6; Projekt: 4; Registry/Info/Sonstiges: 6).
- Davon inzwischen vorhanden („→ übernommen“): 20.
- Wirklich offen oder nur teilweise: Menüleisten-Schalter, Takt-Anzeige, Alle/Keine bei ROM-Import, portabler Modus anzeigen, Dateidialog an Emulatorpfaden, Projektbild, Speicher/Standard-Emulator je Ziel, Gruppen-Reihenfolge, Umwandlung BASIC nach Assembler (Projekt), Programmauswahl der Diskette, Import-Dialog, Tabelle eigener Zeichensätze in Info, 14 Befehle ohne Ausführung, 4 Editorwünsche der Mindestliste (Einrückungslinien/Leerzeichen, Signaturhilfe, Snippets, Farbvorschau).
- Nicht existierende Befehlskennungen in der Matrix: keine. Zahlen der Befehlstabelle in Abschnitt 25 sind veraltet (137 statt 155).
