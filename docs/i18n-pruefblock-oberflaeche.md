<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
# Prüfblock Oberfläche: Deutsch und Englisch

Erzeugt von `tools/i18n_pruefblock.py` aus den Katalogen (nicht von Hand ändern). Die englischen Texte sind ein **Erstentwurf**; maßgeblich für Begriffe ist
[docs/glossar-i18n.md](glossar-i18n.md), für den Stil: einfache Sätze, „you“, Imperativ in Anleitungen. `{$name}` ist ein Platzhalter, der im Programm
durch einen Wert ersetzt wird; `⏎` ist ein Zeilenumbruch im Text. „Zeropage“ heißt auf Englisch „zero page“, „Onion Skin“ bleibt unübersetzt.

**So prüfen:** je Tabelle links lesen, rechts den englischen Text beurteilen; Korrekturen mit Schlüssel (erste Spalte) notieren oder direkt in
`crates/vc-i18n/catalog/en/<bereich>.ftl` ändern. Länge: 1228 Zeilen, Lesezeit etwa 1–2 Stunden. Die übrigen Bereiche (Studios, Debugger,
Protokolltexte, Bauprotokoll …) stehen in den Katalogen und werden stichprobenartig geprüft.

**Inhalt:** Befehle (Menüs, Befehlspalette) (155) · Untermenü-Titel (16) · Tastennamen (4) · Hauptfenster, Menüleiste, Statuszeile, Meldungen (235) · Gemeinsame Beschriftungen der Bereiche (128) · Kontextmenüs (10) · Startseite (40) · Einrichtungsassistent (66) · Assistent: Änderungsliste und Zusammenfassung (84) · Einstellungen (118) · Neues Projekt (47) · Projekt-Einstellungen (90) · Info-Fenster (57) · Export (22) · Import (18) · Projekt kopieren (24) · Studio-Aktionen (Befehlspalette) (62) · Studio-Namen (10) · Update-Prüfung (42)

## Befehle (Menüs, Befehlspalette)

Name jedes Befehls; die Titel der Menüs stehen unter „Untermenü-Titel“ und in der Menüleiste („Hauptfenster“).

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `cmd-ueber` | Über VintageCoder | About VintageCoder |
| `cmd-feedback` | Feedback senden … | Send feedback ... |
| `cmd-update_pruefen` | Nach Updates suchen … | Check for updates ... |
| `cmd-assistent` | Konfigurationsassistent … | Setup assistant ... |
| `cmd-md_fett` | Markdown: Fett | Markdown: Bold |
| `cmd-md_kursiv` | Markdown: Kursiv | Markdown: Italic |
| `cmd-md_code` | Markdown: Code | Markdown: Code |
| `cmd-md_durchgestrichen` | Markdown: Durchgestrichen | Markdown: Strikethrough |
| `cmd-md_link` | Markdown: Link einfügen | Markdown: Insert link |
| `cmd-md_bild` | Markdown: Bild einfügen … | Markdown: Insert image ... |
| `cmd-md_tiefer` | Markdown: Überschrift eine Ebene tiefer (# ergänzen) | Markdown: Heading one level lower (add #) |
| `cmd-md_hoeher` | Markdown: Überschrift eine Ebene höher (# entfernen) | Markdown: Heading one level higher (remove #) |
| `cmd-md_aufzaehlung` | Markdown: Aufzählung | Markdown: Bulleted list |
| `cmd-md_nummerierung` | Markdown: Nummerierung | Markdown: Numbered list |
| `cmd-md_aufgaben` | Markdown: Aufgabenliste | Markdown: Task list |
| `cmd-md_zitat` | Markdown: Zitat | Markdown: Quote |
| `cmd-md_aufgabe` | Markdown: Aufgabe abhaken / wieder öffnen | Markdown: Check / reopen task |
| `cmd-md_tabelle` | Markdown: Tabelle einfügen | Markdown: Insert table |
| `cmd-md_ausrichten` | Markdown: Tabelle ausrichten | Markdown: Align table |
| `cmd-md_als` | Markdown: Datei als Markdown behandeln / als Text | Markdown: Treat file as Markdown / as text |
| `cmd-hilfe_reiter` | Hilfe im Reiter öffnen | Open help in a tab |
| `cmd-art_text` | Art: Text | Type: Text |
| `cmd-art_asm` | Art: Assembler | Type: Assembler |
| `cmd-art_basic` | Art: BASIC | Type: BASIC |
| `cmd-hex_vorschlaege` | BASIC: Hex-Vorschläge für Adressen … | BASIC: Hex suggestions for addresses ... |
| `cmd-std_zeichensatz` | Standardzeichensatz hinzufügen … | Add standard character set ... |
| `cmd-zeichensatz_rom` | Zeichensatz aus ROM importieren … | Import character set from ROM ... |
| `cmd-zeichensatz_png` | Zeichensatz aus PNG-Raster importieren … | Import character set from PNG grid ... |
| `cmd-assistent_neues` | Konfigurationsassistent: Nach Neuem suchen … | Setup assistant: Look for new items ... |
| `cmd-assistent_roms` | Konfigurationsassistent: nur ROMs … | Setup assistant: ROMs only ... |
| `cmd-assistent_emulatoren` | Konfigurationsassistent: nur Emulatoren … | Setup assistant: emulators only ... |
| `cmd-assistent_hardware` | Konfigurationsassistent: nur echte Hardware … | Setup assistant: real hardware only ... |
| `cmd-assistent_design` | Konfigurationsassistent: nur Design und Editor … | Setup assistant: design and editor only ... |
| `cmd-projekt_neu` | Neues Projekt … | New project ... |
| `cmd-projekt_einstellungen` | Projekt-Einstellungen … | Project settings ... |
| `cmd-disk_oeffnen` | Diskette öffnen … | Open disk image ... |
| `cmd-turbo_bereitstellen` | Turbo-BASIC-Diskette bereitstellen … | Provide Turbo BASIC XL disk ... |
| `cmd-turbo_pruefen` | Turbo-BASIC-Diskette prüfen | Check Turbo BASIC XL disk |
| `cmd-turbo_im_disk_editor` | Turbo-BASIC-Diskette im Disk-Editor öffnen | Open Turbo BASIC XL disk in the Disk Editor |
| `cmd-projekt_kopieren` | Projekt kopieren … | Copy project ... |
| `cmd-projekt_ordner_oeffnen` | Projektordner öffnen … | Open project folder ... |
| `cmd-projekt_oeffnen` | Projekt öffnen … (Projektdatei) | Open project ... (project file) |
| `cmd-neuer_reiter` | Neuer Reiter (unbenannt) | New tab (untitled) |
| `cmd-neue_notiz` | Neue Notiz | New note |
| `cmd-datei_oeffnen` | Datei öffnen … | Open file ... |
| `cmd-speichern` | Speichern | Save |
| `cmd-alle_speichern` | Alle speichern | Save all |
| `cmd-datei_schliessen` | Datei schließen | Close file |
| `cmd-neue_datei` | Neue Datei … | New file ... |
| `cmd-neuer_ordner` | Neuer Ordner … | New folder ... |
| `cmd-neue_notiz_dialog` | Neue Notiz … | New note ... |
| `cmd-ziel_oeffnen` | Öffnen | Open |
| `cmd-datei_umbenennen` | Umbenennen … | Rename ... |
| `cmd-datei_loeschen` | Löschen … | Delete ... |
| `cmd-als_include` | Als Include einfügen | Insert as include |
| `cmd-pfad_kopieren` | Pfad kopieren | Copy path |
| `cmd-im_ordner_zeigen` | Im Ordner zeigen | Show in folder |
| `cmd-baum_rueckgaengig` | Verschieben im Baum rückgängig | Undo move in the tree |
| `cmd-hex_ansicht` | Als Hex ansehen (nur Lesen) | View as hex (read only) |
| `cmd-ausschneiden` | Ausschneiden | Cut |
| `cmd-kopieren` | Kopieren | Copy |
| `cmd-einfuegen` | Einfügen | Paste |
| `cmd-gehe_zu_definition` | Gehe zu Definition | Go to definition |
| `cmd-haltepunkt_umschalten` | Haltepunkt umschalten | Toggle breakpoint |
| `cmd-rueckgaengig` | Rückgängig | Undo |
| `cmd-wiederholen` | Wiederholen | Redo |
| `cmd-suchen` | Suchen … | Find ... |
| `cmd-ersetzen` | Ersetzen … | Replace ... |
| `cmd-gehe_zu_zeile` | Gehe zu Zeile … | Go to line ... |
| `cmd-in_projekt_suchen` | In Projekt suchen … | Find in project ... |
| `cmd-palette` | Befehl oder Datei suchen … | Search for a command or file ... |
| `cmd-studio_vergroessern` | Editor: Fläche vergrößern | Editor: Zoom canvas in |
| `cmd-studio_verkleinern` | Editor: Fläche verkleinern | Editor: Zoom canvas out |
| `cmd-studio_einpassen` | Editor: Fläche einpassen | Editor: Fit canvas |
| `cmd-studio_raster` | Editor: Raster ein-/ausschalten | Editor: Toggle grid |
| `cmd-studio_zwiebel` | Editor: Onion Skin ein-/ausschalten | Editor: Toggle Onion Skin |
| `cmd-tracker_noten` | Tracker: Noten | Tracker: Notes |
| `cmd-tracker_instrumente` | Tracker: Instrumente | Tracker: Instruments |
| `cmd-tracker_ablauf` | Tracker: Ablauf | Tracker: Order |
| `cmd-zen` | Zen-Modus: nur der Editor (Seitenleiste, Ablage, Panel Maschine ausblenden) | Zen mode: editor only (hide sidebar, tray, machine panel) |
| `cmd-zahlenwerkzeuge` | Zahlenwerkzeuge … | Number tools ... |
| `cmd-alle_auswaehlen` | Alles auswählen | Select all |
| `cmd-alles_falten` | Alles falten | Fold all |
| `cmd-alles_entfalten` | Alles entfalten | Unfold all |
| `cmd-kommentar_umschalten` | Kommentar umschalten | Toggle comment |
| `cmd-zur_klammer` | Zur passenden Klammer springen | Jump to matching bracket |
| `cmd-leerzeichen_entfernen` | Leerzeichen am Zeilenende entfernen | Remove trailing whitespace |
| `cmd-zeilen_sortieren` | Zeilen sortieren | Sort lines |
| `cmd-korrigieren` | Korrigieren (Quick-Fix) … | Quick fix … |
| `cmd-lesezeichen_umschalten` | Lesezeichen umschalten | Toggle bookmark |
| `cmd-naechstes_lesezeichen` | Nächstes Lesezeichen | Next bookmark |
| `cmd-voriges_lesezeichen` | Voriges Lesezeichen | Previous bookmark |
| `cmd-vervollstaendigen` | Vervollständigung öffnen | Open completion |
| `cmd-problem_springen` | Problem: Zur Zeile springen | Problem: Go to line |
| `cmd-problem_kopieren` | Problem: Meldung kopieren | Problem: Copy message |
| `cmd-probleme_alle_kopieren` | Alle Probleme kopieren | Copy all problems |
| `cmd-protokoll_kopieren` | Protokoll kopieren | Copy log |
| `cmd-protokoll_leeren` | Protokoll leeren (Ausgabe leeren) | Clear log (clear output) |
| `cmd-zeile_formatieren` | Zeile formatieren | Format line |
| `cmd-datei_formatieren` | Datei formatieren | Format file |
| `cmd-basic_nach_asm` | BASIC in Assembler umwandeln | Convert BASIC to assembler |
| `cmd-md_umschalten` | Markdown: Vorschau / Quelltext umschalten | Markdown: Switch between preview and source |
| `cmd-seitenleiste` | Seitenleiste ein-/ausklappen | Show / hide sidebar |
| `cmd-schublade` | Ablage unter dem Editor ein-/ausklappen (Probleme, Ausgabe, Tests, Suche, Gliederung) | Show / hide tray below the editor (problems, output, tests, search, outline) |
| `cmd-panel` | Panel Maschine ein-/ausklappen | Show / hide machine panel |
| `cmd-zoom_plus` | Oberfläche vergrößern | Make UI larger |
| `cmd-zoom_minus` | Oberfläche verkleinern | Make UI smaller |
| `cmd-zoom_null` | Oberfläche: Originalgröße | UI size: original |
| `cmd-hell_dunkel` | Hell/Dunkel umschalten | Switch light / dark |
| `cmd-beenden` | VintageCoder beenden | Quit VintageCoder |
| `cmd-startseite` | Startseite | Start page |
| `cmd-projekt_schliessen` | Projekt schließen | Close project |
| `cmd-zeilenende` | Zeilenende umschalten (LF/CRLF) | Switch line ending (LF/CRLF) |
| `cmd-kodierung` | Kodierung umschalten (UTF-8/ISO-8859-1) | Switch encoding (UTF-8/ISO-8859-1) |
| `cmd-hoher_kontrast` | Hoher Kontrast ein/aus | High contrast on / off |
| `cmd-umbruch` | Zeilenumbruch im Editor | Word wrap in the editor |
| `cmd-naechster_reiter` | Nächster Reiter | Next tab |
| `cmd-voriger_reiter` | Vorheriger Reiter | Previous tab |
| `cmd-reiter_andere_haelfte` | Reiter in die andere Hälfte schicken | Send tab to the other half |
| `cmd-teilen` | Editor teilen / Teilung aufheben | Split editor / remove split |
| `cmd-reiter_links` | Reiter nach links verschieben | Move tab left |
| `cmd-reiter_rechts` | Reiter nach rechts verschieben | Move tab right |
| `cmd-maximiere_fokus` | Bereich mit Fokus maximieren / wiederherstellen | Maximize / restore the area with focus |
| `cmd-spalte_max` | Seitenleiste maximieren / wiederherstellen | Maximize / restore sidebar |
| `cmd-dokument_max` | Dokumentenfeld maximieren / wiederherstellen | Maximize / restore document area |
| `cmd-schublade_max` | Ablage maximieren / wiederherstellen | Maximize / restore tray |
| `cmd-panel_max` | Panel Maschine maximieren / wiederherstellen | Maximize / restore machine panel |
| `cmd-editor_schrift_groesser` | Editor-Schrift größer | Editor font larger |
| `cmd-editor_schrift_kleiner` | Editor-Schrift kleiner | Editor font smaller |
| `cmd-darstellung_zuruecksetzen` | Darstellung zurücksetzen (Größe, Schrift, Layout) | Reset appearance (size, font, layout) |
| `cmd-layout_zuruecksetzen` | Layout zurücksetzen | Reset layout |
| `cmd-einstellungen` | Einstellungen | Settings |
| `cmd-bauen` | Projekt bauen | Build project |
| `cmd-starten` | Startprogramm starten | Run start program |
| `cmd-debuggen` | Startprogramm debuggen | Debug start program |
| `cmd-bauen_aktiv` | Aktive Datei bauen | Build active file |
| `cmd-starten_aktiv` | Aktive Datei starten | Run active file |
| `cmd-debuggen_aktiv` | Aktive Datei debuggen | Debug active file |
| `cmd-debug_weiter` | Debuggen: Weiter | Debug: Continue |
| `cmd-debug_pause` | Debuggen: Anhalten | Debug: Pause |
| `cmd-debug_ueber` | Debuggen: Schritt über | Debug: Step over |
| `cmd-debug_hinein` | Debuggen: Schritt hinein | Debug: Step into |
| `cmd-debug_raus` | Debuggen: Schritt hinaus | Debug: Step out |
| `cmd-debug_ende` | Debuggen beenden | Stop debugging |
| `cmd-debug_hierher` | Debuggen: Bis hierher ausführen | Debug: Run to here |
| `cmd-tests_ausfuehren` | Tests ausführen | Run tests |
| `cmd-exportieren` | Exportieren … | Export ... |
| `cmd-png_laden` | PNG-Bild in Sprite laden … | Load PNG into sprite ... |
| `cmd-naechstes_problem` | Nächstes Problem | Next problem |
| `cmd-voriges_problem` | Voriges Problem | Previous problem |
| `cmd-hilfe` | Hilfe | Help |
| `cmd-projekt_bereich` | Bereich Projekt | Project area |
| `cmd-basic_in_labels` | BASIC: In Labels umwandeln … | BASIC: Convert to labels … |
| `cmd-basic_in_nummern` | BASIC: In Nummern umwandeln … | BASIC: Convert to line numbers … |
| `cmd-basic_neu_nummerieren` | BASIC: Neu nummerieren … | BASIC: Renumber … |

## Untermenü-Titel

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `submenu-suchen-und-ersetzen` | Suchen und Ersetzen | Find and replace |
| `submenu-markdown` | Markdown | Markdown |
| `submenu-zahlen-und-basic` | Zahlen und BASIC | Numbers and BASIC |
| `submenu-falten` | Falten | Folding |
| `submenu-art-und-kodierung` | Art und Kodierung | Type and encoding |
| `submenu-formatieren` | Formatieren | Formatting |
| `submenu-bereiche` | Bereiche | Areas |
| `submenu-zoom-und-schrift` | Zoom und Schrift | Zoom and font |
| `submenu-farbschema` | Farbschema | Color scheme |
| `submenu-studio` | Editor | Editor |
| `submenu-tracker` | Tracker | Tracker |
| `submenu-reiter-und-teilen` | Reiter und Teilen | Tabs and splitting |
| `submenu-probleme-und-protokoll` | Probleme und Protokoll | Problems and log |
| `submenu-debuggen` | Debuggen | Debug |
| `submenu-aktive-datei` | Aktive Datei | Active file |
| `submenu-erweiterungen` | Erweiterungen | Extensions |

## Tastennamen

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `key-ctrl` | Strg | Ctrl |
| `key-shift` | Umschalt | Shift |
| `key-left` | Links | Left |
| `key-right` | Rechts | Right |

## Hauptfenster, Menüleiste, Statuszeile, Meldungen

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `window-title` | VintageCoder | VintageCoder |
| `menu-app-about` | Über VintageCoder | About VintageCoder |
| `menu-app-hide` | VintageCoder ausblenden | Hide VintageCoder |
| `menu-file` | Datei | File |
| `menu-edit` | Bearbeiten | Edit |
| `menu-view` | Ansicht | View |
| `menu-project` | Projekt | Project |
| `menu-tree` | Baum | Tree |
| `menu-help` | Hilfe | Help |
| `status-tree-summary` | {$n } Datei(en) · {$dirs } Ordner | {$n -> ⏎ [one] 1 file ⏎ *[other] {$n } files ⏎ } · {$dirs -> ⏎ [one] 1 folder ⏎ *[other] {$dirs } folders ⏎ } |
| `status-tree-filtered` | (gefiltert) | (filtered) |
| `build-failed` | Bauen fehlgeschlagen: {$n } Problem(e) | Build failed: {$n -> ⏎ [one] 1 problem ⏎ *[other] {$n } problems ⏎ } |
| `gui-diskette-d71` | Diskette D71 | Disk D71 |
| `gui-diskette-d81` | Diskette D81 | Disk D81 |
| `gui-diskette-atr` | Diskette ATR | Disk ATR |
| `gui-diskette-d64` | Diskette D64 | Disk D64 |
| `gui-rel-gibt-es-schon` | {$rel } gibt es schon | {$rel } already exists |
| `gui-fuer-dieses-projekt-gibt-es-dafuer` | Für dieses Projekt gibt es dafür kein Profil. | There is no profile for this in this project. |
| `gui-angelegt-rel` | Angelegt: {$rel } | Created: {$rel } |
| `gui-bildschirm-kachel-und-tracker-dateien-oeffnen` | Bildschirm-, Kachel- und Tracker-Dateien öffnen sich mit ihren Editoren (folgt). | Screen, tile and tracker files open with their editors (coming). |
| `gui-ordner-angelegt-r` | Ordner angelegt: {$r }/ | Folder created: {$r }/ |
| `gui-nur-der-name-kein-pfad-verschieben` | Nur der Name, kein Pfad: Verschieben geht per Ziehen im Baum. | Name only, no path: moving is done by dragging in the tree. |
| `gui-umbenannt-alt-neu` | Umbenannt: {$alt } → {$neu } | Renamed: {$alt } → {$neu } |
| `gui-p1-rel` | {$p1 } {$rel } | {$p1 } {$rel } |
| `gui-projekt-wechseln-erst-die-debug-sitzung` | Projekt wechseln: erst die Debug-Sitzung beziehungsweise den Bau beenden. | Switch project: first end the debug session or the build. |
| `gui-projekt-geoeffnet-p1` | Projekt geöffnet: {$p1 } | Project opened: {$p1 } |
| `gui-rel-sicherung-fehlgeschlagen-nichts-umgewandelt-e` | {$rel }: Sicherung fehlgeschlagen, nichts umgewandelt: {$e } | {$rel }: backup failed, nothing converted: {$e } |
| `gui-kachelkarte-umgewandelt-sicherung-der-alten-datei` | Kachelkarte umgewandelt; Sicherung der alten Datei: {$p1 } | Tile map converted; backup of the old file: {$p1 } |
| `gui-rel-umwandlung-fehlgeschlagen-e` | {$rel }: Umwandlung fehlgeschlagen: {$e } | {$rel }: conversion failed: {$e } |
| `gui-pc-p1` | · PC ${$p1 } | · PC ${$p1 } |
| `gui-ein-ordner-laesst-sich-nicht-in` | Ein Ordner lässt sich nicht in sich selbst verschieben. | A folder cannot be moved into itself. |
| `gui-verschieben-erst-alle-geaenderten-dateien-speichern` | Verschieben: erst alle geänderten Dateien speichern (Pfade in Quelltexten werden angepasst). | Move: first save all changed files (paths in source files are adjusted). |
| `gui-quelle-nichts-zu-verschieben` | {$quelle }: nichts zu verschieben | {$quelle }: nothing to move |
| `gui-verschieben-braucht-eine-projektdatei-vcprj` | Verschieben braucht eine Projektdatei (.vcprj). | Moving needs a project file (.vcprj). |
| `gui-verschoben-quelle-neu-p3` | Verschoben: {$quelle } → {$neu }{$p3 } | Moved: {$quelle } → {$neu }{$p3 } |
| `gui-verschieben-fehlgeschlagen-e` | Verschieben fehlgeschlagen: {$e } | Move failed: {$e } |
| `gui-es-gibt-nichts-rueckgaengig-zu-machen` | Es gibt nichts rückgängig zu machen. | There is nothing to undo. |
| `gui-rueckgaengig-erst-alle-geaenderten-dateien-speichern` | Rückgängig: erst alle geänderten Dateien speichern. | Undo: first save all changed files. |
| `gui-verschieben-rueckgaengig-gemacht-n-datei-en` | Verschieben rückgängig gemacht ({$n } Datei(en)) | Move undone ({$n -> ⏎ [one] 1 file ⏎ *[other] {$n } files ⏎ }) |
| `gui-rel-keine-textdatei-oeffnet-im-standardprogramm` | {$rel }: keine Textdatei, öffnet im Standardprogramm | {$rel }: not a text file, opens in the default program |
| `gui-e-wird-nicht-ueberschrieben` | {$e } (wird nicht überschrieben) | {$e } (will not be overwritten) |
| `gui-rel-nicht-lesbar` | ; {$rel }: nicht lesbar | ; {$rel }: not readable |
| `gui-braucht-eine-debug-sitzung-und-einen` | braucht eine Debug-Sitzung und einen offenen Quelltext | needs a debug session and an open source file |
| `gui-braucht-eine-debug-sitzung` | braucht eine Debug-Sitzung | needs a debug session |
| `gui-braucht-ein-geoeffnetes-studio` | braucht einen geöffneten Editor | needs an open editor |
| `gui-braucht-eine-markdown-datei` | braucht eine Markdown-Datei | needs a Markdown file |
| `gui-braucht-ein-projekt` | braucht ein Projekt | needs a project |
| `gui-braucht-einen-offenen-quelltext` | braucht einen offenen Quelltext | needs an open source file |
| `gui-gerade-nicht-moeglich` | gerade nicht möglich | not possible right now |
| `gui-menue` | ☰ Menü | ☰ Menu |
| `gui-start` | Start | Start |
| `gui-projekt` | Projekt | Project |
| `gui-p1-ordner-p2-ebene-p3` | {$p1 }, Ordner, {$p2 }, Ebene {$p3 } | {$p1 }, folder, {$p2 }, level {$p3 } |
| `gui-p1-datei-ebene-p2` | {$p1 }, Datei, Ebene {$p2 } | {$p1 }, file, level {$p2 } |
| `gui-editor-n` | Editor {$n } | Editor {$n } |
| `gui-kein-dokument` | Kein Dokument | No document |
| `gui-zeile-p1-p2` | Zeile {$p1 }: {$p2 } | Line {$p1 }: {$p2 } |
| `gui-keine-definition-gefunden` | Keine Definition gefunden. | No definition found. |
| `gui-definition-ausserhalb-des-projekts-uri` | Definition außerhalb des Projekts: {$uri } | Definition outside the project: {$uri } |
| `gui-kuerzel-menue-ausloesung-id-id` | Kürzel: Menü-Auslösung id={$id } | Shortcut: menu trigger id={$id } |
| `gui-kuerzel-id-als-doppelereignis-verworfen` | Kürzel: {$id } als Doppelereignis verworfen | Shortcut: {$id } discarded as a duplicate event |
| `gui-rmt-modul-waehlen` | RMT-Modul wählen | Choose RMT module |
| `gui-song-aus-anderem-system-waehlen` | Song aus anderem System wählen | Choose a song from another system |
| `gui-png-bild-laden` | PNG-Bild laden | Load PNG image |
| `gui-zu-a-gibt-es-keine-quellzeile` | Zu ${$a } gibt es keine Quellzeile: dort lässt sich kein Haltepunkt setzen. | There is no source line for ${$a }: a breakpoint cannot be set there. |
| `gui-ziel-p1` | Ziel: {$p1 } | Target: {$p1 } |
| `gui-gestartet-c` | Gestartet: {$c } | Started: {$c } |
| `gui-start-fehlgeschlagen-p1` | Start fehlgeschlagen: {$p1 } | Start failed: {$p1 } |
| `gui-einzelheit-e` | Einzelheit: {$e } | Detail: {$e } |
| `gui-n-ersetzt` | {$n } ersetzt | {$n } replaced |
| `gui-p1-nicht-lesbar` | {$p1 }: nicht lesbar | {$p1 }: not readable |
| `gui-p1-von-n` | {$p1 } von {$n } | {$p1 } of {$n } |
| `gui-fehler` | Fehler | Error |
| `gui-hinweis` | Hinweis | Note |
| `gui-warnung` | Warnung | Warning |
| `gui-kein-quelltext-offen-hier-gibt-es` | Kein Quelltext offen: hier gibt es nichts zum Einbinden | No source file open: nothing to include here |
| `gui-eingebunden-p1-p2-p3-z-nimmt` | Eingebunden: {$p1 }{$p2 } ({$p3 }+Z nimmt es zurück) | Included: {$p1 }{$p2 } ({$p3 }+Z takes it back) |
| `gui-diese-datei-ist-schon-eingebunden-zeile` | Diese Datei ist schon eingebunden (Zeile {$p1 }) – nichts eingefügt | This file is already included (line {$p1 }) – nothing inserted |
| `gui-zeile-zeile-hat-keinen-code-dort` | Zeile {$zeile } hat keinen Code: dort kann nicht angehalten werden. | Line {$zeile } has no code: execution cannot stop there. |
| `gui-datei-oeffnen` | Datei öffnen | Open file |
| `gui-projekt-oeffnen-projektdatei-vcprj` | Projekt öffnen (Projektdatei .vcprj) | Open project (project file .vcprj) |
| `gui-projektordner-oeffnen` | Projektordner öffnen | Open project folder |
| `gui-zeile-ist-schon-formatiert-oder-hier` | Zeile ist schon formatiert (oder hier gibt es keine Formatierung) | The line is already formatted (or there is no formatting here) |
| `gui-datei-ist-schon-formatiert-oder-hier` | Datei ist schon formatiert (oder hier gibt es keine Formatierung) | The file is already formatted (or there is no formatting here) |
| `gui-n-p2-p3-gilt-beim-speichern` | {$n }: {$p2 } · {$p3 } (gilt beim Speichern) | {$n }: {$p2 } · {$p3 } (applies when saving) |
| `gui-nach-neuem-gesucht-rom-verzeichnis-neu` | Nach Neuem gesucht: ROM-Verzeichnis neu eingelesen, Emulatorsuche im Assistenten. | Looked for new items: ROM folder read again, emulator search in the assistant. |
| `gui-keine-vorschlaege-an-dieser-stelle` | Keine Vorschläge an dieser Stelle | No suggestions here |
| `gui-keine-probleme` | Keine Probleme | No problems |
| `gui-pfad-kopiert-p` | Pfad kopiert: {$p } | Path copied: {$p } |
| `gui-gehe-zu-definition-nur-in-assembler` | Gehe zu Definition: nur in Assembler-Dateien. | Go to definition: assembler files only. |
| `gui-projekt-einstellungen` | Projekt-Einstellungen | Project settings |
| `gui-vintagecoder-beenden` | VintageCoder beenden? | Quit VintageCoder? |
| `gui-projekt-wechseln` | Projekt wechseln? | Switch project? |
| `gui-diese-reiter-haben-ungespeicherte-aenderungen` | Diese Reiter haben ungespeicherte Änderungen: | These tabs have unsaved changes: |
| `gui-unbenannt-wird-in-der-zwischenkopie-gesichert` | (unbenannt, wird in der Zwischenkopie gesichert) | (untitled, saved in the backup copy) |
| `gui-speichern` | Speichern | Save |
| `gui-verwerfen` | Verwerfen | Discard |
| `gui-abbrechen` | Abbrechen | Cancel |
| `gui-notiz` | Notiz | Note |
| `gui-unbenannt` | Unbenannt | Untitled |
| `gui-reiter-nicht-gefunden` | Reiter nicht gefunden | Tab not found |
| `gui-gespeichert-rel` | Gespeichert: {$rel } | Saved: {$rel } |
| `gui-letzte-sitzung-wiederherstellen` | Letzte Sitzung wiederherstellen? | Restore the last session? |
| `gui-beim-letzten-beenden-waren-unbenannte-reiter` | Beim letzten Beenden waren unbenannte Reiter ungespeichert. Die Zwischenkopie liegt nur auf diesem Rechner, unverschlüsselt, und verfällt nach 14 Tagen. | When you last quit, untitled tabs were unsaved. The backup copy exists only on this computer, unencrypted, and expires after 14 days. |
| `gui-p1-p2-zeichen-p3` | • {$p1 } · {$p2 } Zeichen · {$p3 } | • {$p1 } · {$p2 } characters · {$p3 } |
| `gui-wiederherstellen` | Wiederherstellen | Restore |
| `gui-nicht-jetzt` | Nicht jetzt | Not now |
| `gui-gespeichert-n` | Gespeichert: {$n } | Saved: {$n } |
| `gui-n-nicht-gespeichert-die-datei-war` | {$n }: nicht gespeichert, die Datei war nicht lesbar und würde überschrieben | {$n }: not saved, the file was not readable and would be overwritten |
| `gui-speichern-fehlgeschlagen-er` | Speichern fehlgeschlagen: {$er } | Saving failed: {$er } |
| `gui-keine-quelldatei-zum-bauen` | Keine Quelldatei zum Bauen. | No source file to build. |
| `gui-bauen-rel` | Bauen: {$rel } … | Building: {$rel } ... |
| `gui-fehler-e` | Fehler: {$e } | Error: {$e } |
| `gui-lang-spruenge-automatisch-verlaengert-branchlong-auto` | {$lang } Sprünge automatisch verlängert (.branchlong auto) | {$lang -> ⏎ [one] 1 jump ⏎ *[other] {$lang } jumps ⏎ } extended automatically (.branchlong auto) |
| `gui-fertig-p1-bytes-p2` | Fertig: {$p1 } Bytes → {$p2 } | Done: {$p1 } bytes → {$p2 } |
| `gui-disk-p1-p2-bytes-p3` | Disk {$p1 } ({$p2 } Bytes): {$p3 } | Disk {$p1 } ({$p2 } bytes): {$p3 } |
| `gui-diskette-e` | Diskette: {$e } | Disk: {$e } |
| `gui-nichts-zum-starten-erst-bauen` | Nichts zum Starten: erst bauen. | Nothing to run: build first. |
| `gui-starten-auf-p1` | Starten auf {$p1 } … | Running on {$p1 } ... |
| `gui-starten-im-emulator` | Starten im Emulator … | Running in the emulator ... |
| `gui-testlauf-kein-emulator-gestartet` | Testlauf: kein Emulator gestartet | Test run: no emulator started |
| `gui-emulator` | Emulator | Emulator |
| `gui-emulator-p1` | Emulator ({$p1 }) | Emulator ({$p1 }) |
| `gui-sname-p2` | {$sname } · {$p2 } ▾ | {$sname } · {$p2 } ▾ |
| `gui-nichts-zum-debuggen-erst-bauen` | Nichts zum Debuggen: erst bauen. | Nothing to debug: build first. |
| `gui-testlauf-keine-debug-sitzung-gestartet` | Testlauf: keine Debug-Sitzung gestartet | Test run: no debug session started |
| `gui-debuggen-e` | Debuggen: {$e } | Debug: {$e } |
| `gui-p1-debuggen-geht-nur-ueber-usb` | {$p1 }: Debuggen geht nur über USB (Ethernet bietet keinen Monitor). | {$p1 }: debugging only works over USB (Ethernet offers no monitor). |
| `gui-debug-sitzung-startet` | Debug-Sitzung startet … | Debug session is starting ... |
| `gui-debug-sitzung-beendet` | Debug-Sitzung beendet | Debug session ended |
| `gui-einzelheit-r` | Einzelheit: {$r } | Detail: {$r } |
| `gui-die-tests-laufen-parallel-zur-debug` | Die Tests laufen parallel zur Debug-Sitzung (eigener Emulator). | The tests run in parallel to the debug session (own emulator). |
| `gui-debug-sitzung-fuer-den-test-name` | Debug-Sitzung für den Test „{$name }“ startet … | Debug session for the test "{$name }" is starting ... |
| `gui-studio-p1` | editor {$p1 } | editor {$p1 } |
| `gui-zuletzt-geoeffnet` | zuletzt geöffnet | recently opened |
| `gui-debuggen` | ● Debuggen | ● Debug |
| `gui-name-tasten` | {$name }   {$tasten } | {$name }   {$tasten } |
| `gui-kein-projekt` | Kein Projekt | No project |
| `gui-projekt-2` | Projekt ▾ | Project ▾ |
| `gui-starten` | Starten | Run |
| `gui-debuggen-2` | Debuggen | Debug |
| `gui-bauen` | Bauen | Build |
| `gui-mehr` | Mehr | More |
| `gui-suchen` | Suchen | Search |
| `gui-ziel-p1-p2` | Ziel: {$p1 } · {$p2 } | Target: {$p1 } · {$p2 } |
| `gui-plus-4` | Plus/4 | Plus/4 |
| `gui-atari` | Atari | Atari |
| `gui-kurz` | {$kurz } ▾ | {$kurz } ▾ |
| `gui-update-v-verfuegbar` | Update {$v } verfügbar | Update {$v } available |
| `gui-baut` | baut … | building ... |
| `gui-z-z-s-s-p3` | Z {$z } · S {$s }   {$p3 } | L {$z } · C {$s }   {$p3 } |
| `gui-kodierung-beim-speichern-utf-8-iso` | Kodierung beim Speichern: UTF-8 ↔ ISO-8859-1 (Klick wechselt) | Encoding when saving: UTF-8 ↔ ISO-8859-1 (click to switch) |
| `gui-zeilenende-beim-speichern-lf-crlf-klick` | Zeilenende beim Speichern: LF ↔ CRLF (Klick wechselt) | Line ending when saving: LF ↔ CRLF (click to switch) |
| `gui-p1-name` | {$p1 }{$name } | {$p1 }{$name } |
| `gui-vintagecoder` | VintageCoder | VintageCoder |
| `gui-version-p1` | Version {$p1 } | Version {$p1 } |
| `gui-klick-oder-taste-schliesst-das-startbild` | Klick oder Taste schließt das Startbild | Click or key closes the start screen |
| `gui-name-p2` | {$name }{$p2 } | {$name }{$p2 } |
| `gui-teilung-aufheben-cmd-alt-t` | Teilung aufheben (Cmd+Alt+T) | Remove split (Cmd+Alt+T) |
| `gui-editor-teilen-zweite-haelfte-rechts-cmd` | Editor teilen: zweite Hälfte rechts (Cmd+Alt+T) | Split editor: second half on the right (Cmd+Alt+T) |
| `gui-markdown-vorschau` | Markdown-Vorschau | Markdown preview |
| `gui-quelltext` | Quelltext | Source code |
| `gui-zum-editor-wechseln-befehlspalette-markdown-vorschau` | Zum Editor wechseln (Befehlspalette: Markdown: Vorschau/Quelltext) | Switch to the editor (command palette: Markdown: Switch between preview and source) |
| `gui-markdown-quelltext` | Markdown-Quelltext | Markdown source |
| `gui-vorschau` | Vorschau | Preview |
| `gui-zur-gerenderten-ansicht-wechseln` | Zur gerenderten Ansicht wechseln | Switch to the rendered view |
| `gui-aa` | Aa ✓ | Aa ✓ |
| `gui-aa-2` | Aa | Aa |
| `gui-gehe-zu-zeile` | Gehe zu Zeile | Go to line |
| `gui-zeilennummer` | Zeilennummer | Line number |
| `gui-keine-probleme-2` | Keine Probleme. | No problems. |
| `gui-abbrechen-2` | ■ Abbrechen | ■ Cancel |
| `gui-alle-ausfuehren` | ▶ Alle ausführen | ▶ Run all |
| `gui-fehlgeschlagene` | Fehlgeschlagene | Failed |
| `gui-laeuft-auf-p1` | Läuft auf: {$p1 } ▾ | Runs on: {$p1 } ▾ |
| `gui-filter` | Filter | Filter |
| `gui-nur-fehler` | nur Fehler | errors only |
| `gui-laeuft` | läuft … | running ... |
| `gui-p1-bestanden-fehler-fehlgeschlagen-p3-uebersprungen` | {$p1 } bestanden · {$fehler } fehlgeschlagen · {$p3 } übersprungen{$p4 } | {$p1 } passed · {$fehler } failed · {$p3 } skipped{$p4 } |
| `gui-gemessen-m` | gemessen: {$m } | measured: {$m } |
| `gui-a-p1-x-p2-y-p3` | A {$p1 }  X {$p2 }  Y {$p3 }  SP {$p4 }  PC {$p5 } | A {$p1 }  X {$p2 }  Y {$p3 }  SP {$p4 }  PC {$p5 } |
| `gui-zur-quelle-rel-z` | Zur Quelle ({$rel }:{$z }) | To the source ({$rel }:{$z }) |
| `gui-enthaelt-geoeffnete-dateien` | enthält geöffnete Dateien | contains open files |
| `gui-das-projekt-ist-leer` | Das Projekt ist leer. | The project is empty. |
| `gui-dateien-filtern` | Dateien filtern | Filter files |
| `gui-neue-datei-im-gewaehlten-ordner` | Neue Datei im gewählten Ordner | New file in the chosen folder |
| `gui-baum-neu-einlesen` | Baum neu einlesen | Reload the tree |
| `gui-alle-ordner-zuklappen` | Alle Ordner zuklappen | Collapse all folders |
| `gui-suche-laeuft` | Suche läuft … | Searching ... |
| `gui-eine-debug-sitzung-laeuft-beenden-und` | Eine Debug-Sitzung läuft. Beenden und den Test „{$n }“ debuggen? | A debug session is running. End it and debug the test "{$n }"? |
| `gui-eine-debug-sitzung-laeuft-beenden-und-2` | Eine Debug-Sitzung läuft. Beenden und neu bauen und debuggen? | A debug session is running. End it, rebuild and debug? |
| `gui-reiter-schliessen` | Reiter schließen | Close tab |
| `gui-n-hat-inhalt-und-ist-nicht` | „{$n }“ hat Inhalt und ist nicht gespeichert (Cmd+S legt ihn als Datei an). Verwerfen? | "{$n }" has content and is not saved (Cmd+S saves it as a file). Discard? |
| `gui-aeltere-kachelkarte-umwandeln` | Ältere Kachelkarte umwandeln? | Convert an older tile map? |
| `gui-n-ist-eine-aeltere-kachelkarte-kind` | „{$n }“ ist eine ältere Kachelkarte (kind „tilemap“). Das Kachelsatz-Editor liest Kachelsatz und Bildschirm: Umwandeln legt beides neben der Datei an und öffnet den Bildschirm. Die alte Datei wird vorher unter .vcbackup/ gesichert und dann durch die neuen ersetzt. | "{$n }" is an older tile map (kind "tilemap"). The tile set editor reads a tile set and a screen: converting creates both next to the file and opens the screen. The old file is first backed up under .vcbackup/ and then replaced by the new ones. |
| `gui-umwandeln` | Umwandeln | Convert |
| `gui-konfiguration-uebernehmen` | Konfiguration übernehmen? | Import the configuration? |
| `gui-die-datei-p1-enthaelt-z-die` | Die Datei „{$p1 }“ enthält: {$z }. Die jetzigen Einstellungen für Zielsysteme, Emulatoren, Maschinen und Parameter werden ersetzt; bekannte Maschinen behalten ihr Passwort. Die Oberflächeneinstellungen (Farbschema, Zoom) bleiben. | The file "{$p1 }" contains: {$z }. The current settings for target systems, emulators, machines and parameters will be replaced; known machines keep their password. The interface settings (color scheme, zoom) stay. |
| `gui-uebernehmen` | Übernehmen | Apply |
| `gui-einstellungen-zuruecksetzen` | Einstellungen zurücksetzen? | Reset settings? |
| `gui-alle-einstellungen-der-ide-zielsysteme-emulatoren` | Alle Einstellungen der IDE (Zielsysteme, Emulatoren, Maschinen samt Passwörtern, Pfade, Parameter) gehen auf die Vorgabe zurück. Das lässt sich nicht rückgängig machen; vorher kann man die Konfiguration exportieren. | All settings of the IDE (target systems, emulators, machines including passwords, paths, parameters) go back to the defaults. This cannot be undone; you can export the configuration first. |
| `gui-zuruecksetzen` | Zurücksetzen | Reset |
| `gui-ordner` | Ordner | Folder |
| `gui-speichern-unter` | Speichern unter | Save as |
| `gui-rel-p2` | {$rel } {$p2 } | {$rel } {$p2 } |
| `gui-p1-name-2` | {$p1 } {$name } | {$p1 } {$name } |
| `gui-wird-angelegt-rel` | Wird angelegt: {$rel } | Being created: {$rel } |
| `gui-alle` | Alle | All |
| `gui-programm` | Programm | Program |
| `gui-p1-64-kb` | {$p1 } · 64 KB | {$p1 } · 64 KB |
| `gui-r-000` | ${$r }000 | ${$r }000 |
| `gui-seite-p-p2-p3` | SEITE ${$p } · ${$p2 }–${$p3 } | PAGE ${$p } · ${$p2 }–${$p3 } |
| `gui-programm-p1-von-256-byte` | Programm: {$p1 } von 256 Byte | Program: {$p1 } of 256 bytes |
| `gui-n-a` | {$n }  ${$a } | {$n }  ${$a } |
| `gui-f-a-p3` | {$f }:{$a }{$p3 } | {$f }:{$a }{$p3 } |
| `gui-pruefung` | PRÜFUNG | CHECK |
| `gui-w-hh-bild-p3-aufbau-p4` | {$w }×{$hh } · Bild {$p3 } · Aufbau {$p4 } ms | {$w }×{$hh } · frame {$p3 } · build {$p4 } ms |
| `gui-hilfe` | Hilfe | Help |
| `gui-im-reiter-oeffnen` | Im Reiter öffnen | Open in a tab |
| `gui-dieses-thema-im-dokumentenfeld-oeffnen-mehr` | Dieses Thema im Dokumentenfeld öffnen (mehr Platz) | Open this topic in the document area (more room) |
| `gui-code-code2` | ${$code } · {$code2 } | ${$code } · {$code2 } |
| `gui-datei` | Datei | File |
| `gui-projekt-n` | Projekt: {$n } | Project: {$n } |
| `gui-zuletzt-geoeffnet-2` | Zuletzt geöffnet | Recently opened |
| `gui-studio-n` | Editor: {$n } | Editor: {$n } |
| `gui-studio` | Editor | Editor |
| `gui-in-den-papierkorb-gelegt` | In den Papierkorb gelegt: | Moved to the trash: |
| `gui-geloescht` | Gelöscht: | Deleted: |
| `gui-angepasst-pfad-e-in-quelltexten-angepasst` | ({$angepasst } Pfad(e) in Quelltexten angepasst) | ({$angepasst } path(s) adjusted in source code) |
| `gui-alternative-von-hand-a` | Alternative von Hand: {$a } | Alternative by hand: {$a } |
| `gui-sid-datei-sid-goattracker-sng-oder-song` | SID-Datei (.sid), GoatTracker (.sng) oder Song (.vcsong) | SID file (.sid), GoatTracker (.sng) or song (.vcsong) |
| `gui-vintagecoder-projekt` | VintageCoder-Projekt | VintageCoder project |
| `gui-aufgeklappt` | aufgeklappt | expanded |
| `gui-zugeklappt` | zugeklappt | collapsed |

## Gemeinsame Beschriftungen der Bereiche

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `texte-app-titel` | VintageCoder | VintageCoder |
| `texte-starten` | ▶ Starten | ▶ Run |
| `texte-debuggen` | ● Debuggen | ● Debug |
| `texte-bauen` | ▣ Bauen | ▣ Build |
| `texte-baut` | … Baut | ... Building |
| `texte-suchfeld` | Befehl oder Datei suchen | Search for a command or file |
| `texte-dateien` | Dateien | Files |
| `texte-suche` | Suche | Search |
| `texte-teilen-mind-zwei` | Zum Teilen müssen mindestens zwei Reiter offen sein. | To split, at least two tabs must be open. |
| `texte-ablegen-rechts` | Hier ablegen: Editor teilen | Drop here: split editor |
| `texte-streifen` | Rechte Leiste | Right bar |
| `texte-ausgeklappt` | ausgeklappt | expanded |
| `texte-eingeklappt` | eingeklappt | collapsed |
| `texte-spalte` | Linke Spalte | Left column |
| `texte-gruppe-export` | Exportiert (build/export) | Exported (build/export) |
| `texte-dokumente` | Dokumentenfeld | Document area |
| `texte-schublade` | Schublade | Drawer |
| `texte-panel` | Rechtes Panel | Right panel |
| `texte-normal` | normal | normal |
| `texte-minimiert` | minimiert | minimized |
| `texte-maximiert` | maximiert | maximized |
| `texte-hilfe-titel` | Hilfe | Help |
| `texte-zeichen` | Zeichen | Characters |
| `texte-hilfe-suche` | Suchen: Thema, Befehl, Register … | Search: topic, command, register ... |
| `texte-hilfe-zurueck` | ‹ Zurück | ‹ Back |
| `texte-hilfe-nichts` | Nichts gefunden. | Nothing found. |
| `texte-zeichen-suche` | Name, Zahl ($c1) oder Zeichen | Name, number ($c1) or character |
| `texte-zeichen-nicht-hier` | Hier lässt sich kein Zeichen einfügen. | A character cannot be inserted here. |
| `texte-art-asm` | Assembler | Assembler |
| `texte-art-basic` | BASIC | BASIC |
| `texte-art-text` | Text | Text |
| `texte-art-notiz` | Notiz | Note |
| `texte-dlg-neue-datei` | Neue Datei | New file |
| `texte-dlg-neuer-ordner` | Neuer Ordner | New folder |
| `texte-dlg-umbenennen` | Datei umbenennen | Rename file |
| `texte-dlg-loeschen` | Datei löschen | Delete file |
| `texte-dlg-art` | Art | Type |
| `texte-dlg-name` | Name (der Ordner steht davor) | Name (the folder comes first) |
| `texte-dlg-profil` | Profil | Profile |
| `texte-baum-pfad` | Pfad, z. B. lib/musik.asm | Path, e.g. lib/music.asm |
| `texte-baum-ok` | OK | OK |
| `texte-baum-abbruch` | Abbrechen | Cancel |
| `texte-baum-frage-korb` | kommt in den Papierkorb. Löschen? | will be moved to the trash. Delete? |
| `texte-baum-frage-endgueltig` | wird endgültig gelöscht (kein Papierkorb). Löschen? | will be deleted permanently (no trash). Delete? |
| `texte-baum-keine-datei` | Zuerst eine Datei öffnen. | Open a file first. |
| `texte-protokoll` | Protokoll | Log |
| `texte-probleme` | Probleme | Problems |
| `texte-bereit` | Bereit | Ready |
| `texte-keine-datei` | Keine Datei geöffnet. Cmd+K, Cmd+O oder links wählen. | No file open. Choose one with Cmd+K, Cmd+O or on the left. |
| `texte-kein-projekt` | Kein Projekt geöffnet | No project open |
| `texte-suche-leer` | Nichts gefunden. | Nothing found. |
| `texte-suche-platzhalter` | In allen Dateien suchen … | Search in all files ... |
| `texte-finden` | Suchen | Find |
| `texte-ersetzen` | Ersetzen | Replace |
| `texte-alle` | Alle | All |
| `texte-zeile` | Zeile | Line |
| `texte-treffer` | Treffer | Matches |
| `texte-kein-treffer` | kein Treffer | no match |
| `texte-gliederung` | Gliederung | Outline |
| `texte-gliederung-leer` | Keine Marken in dieser Datei. | No markers in this file. |
| `texte-einstellungen` | Einstellungen | Settings |
| `texte-farbschema` | Farbschema | Color scheme |
| `texte-dunkel` | Dunkel | Dark |
| `texte-hell` | Hell | Light |
| `texte-system` | System | System |
| `texte-kontrast` | Hoher Kontrast | High contrast |
| `texte-oberflaeche` | Oberfläche | UI size |
| `texte-editor-schrift` | Editor-Schrift | Editor font |
| `texte-umbruch` | Zeilenumbruch | Word wrap |
| `texte-debuggen-panel` | Debuggen | Debug |
| `texte-dbg-weiter` | ▶ Weiter | ▶ Continue |
| `texte-dbg-pause` | ⏸ Anhalten | ⏸ Pause |
| `texte-dbg-ende` | ■ Beenden | ■ Stop |
| `texte-dbg-ueber` | Schritt über | Step over |
| `texte-dbg-hinein` | Hinein | Step into |
| `texte-dbg-raus` | Hinaus | Step out |
| `texte-dbg-startet` | Emulator startet … | Emulator is starting ... |
| `texte-dbg-laeuft` | Programm läuft | Program running |
| `texte-dbg-angehalten` | Angehalten bei | Stopped at |
| `texte-dbg-wartet` | Warte auf den Emulator … | Waiting for the emulator ... |
| `texte-dbg-register` | REGISTER | REGISTERS |
| `texte-dbg-disassembler` | DISASSEMBLER | DISASSEMBLER |
| `texte-dbg-speicher` | SPEICHER (Adresse hexadezimal, * = geändert) | MEMORY (address in hexadecimal, * = changed) |
| `texte-dbg-beobachten` | BEOBACHTEN | WATCH |
| `texte-dbg-haltepunkte` | HALTEPUNKTE | BREAKPOINTS |
| `texte-dbg-ausdruck` | Ausdruck, z. B. $d020 | Expression, e.g. $d020 |
| `texte-karte` | Karte | Memory map |
| `texte-karte-leer` | Noch nichts gebaut: Nach dem Bauen zeigt die Karte, wo dein Programm im Speicher liegt. | Nothing built yet: after building, the map shows where your program sits in memory. |
| `texte-dbg-moment` | MOMENTAUFNAHME | SNAPSHOT |
| `texte-dbg-moment-erkennen` | Bildschirm erkennen | Detect screen |
| `texte-dbg-moment-sprites` | Sprites speichern | Save sprites |
| `texte-dbg-moment-kacheln` | Zeichen als Kacheln | Characters as tiles |
| `texte-dbg-moment-rom` | Zeichensatz aus dem ROM lesen | Read character set from ROM |
| `texte-dbg-moment-name` | Name | Name |
| `texte-dbg-moment-speichern` | Als Dateien speichern | Save as files |
| `texte-tests` | Tests | Tests |
| `texte-tests-leer` | Noch keine Tests gelaufen. „Alle ausführen“ prüft die statischen Regeln und die Tests der .vctest-Dateien. | No tests run yet. "Run all" checks the static rules and the tests of the .vctest files. |
| `texte-tests-wahl` | Wähle einen Test: Erwartung, Messwert, Register und Zyklen erscheinen hier. | Choose a test: expectation, measured value, registers and cycles appear here. |
| `texte-test-debug` | Diesen Test debuggen | Debug this test |
| `texte-emulator-bild` | Bild | Image |
| `texte-emu-aktualisieren` | Bild holen | Get image |
| `texte-emu-auto` | bei jedem Halt | at every stop |
| `texte-emu-live` | Zeitlupe live | Live slow motion |
| `texte-emu-laeuft` | Das Programm läuft: Anhalten oder „Zeitlupe live“ zeigt das Bild. | The program is running: pause it or use "Live slow motion" to see the image. |
| `texte-emu-kein-bild` | Noch kein Bild: „Bild holen“. | No image yet: "Get image". |
| `texte-emu-text` | Text an den Emulator senden (Enter) | Send text to the emulator (Enter) |
| `texte-rq-titel` | Debug-Sitzung läuft | Debug session running |
| `texte-rq-beenden` | Sitzung beenden | End session |
| `texte-tests-parallel` | Die Debug-Sitzung läuft weiter; die Tests starten einen eigenen Emulator. | The debug session keeps running; the tests start their own emulator. |
| `texte-debug-reiter` | Debug | Debug |
| `texte-dbg-reg-laeuft` | Das Programm läuft. „Anhalten“ zeigt die Werte. | The program is running. "Pause" shows the values. |
| `texte-dbg-speicher-titel` | SPEICHER | MEMORY |
| `texte-dbg-speicher-feld` | Adresse, z. B. D020 oder start | Address, e.g. D020 or start |
| `texte-dbg-keine-haltepunkte` | Keine Haltepunkte. Mit F9 im Editor oder per Klick auf den Rand des Disassemblers setzen. | No breakpoints. Set one with F9 in the editor or by clicking the margin of the disassembler. |
| `texte-export-titel` | Export | Export |
| `texte-export-kein-studio` | Kein Editor als Ziel: Öffne ein Sprite, einen Zeichensatz oder eine Palette im Editor und klicke dann auf „Exportieren …“. | No editor as target: open a sprite, a character set or a palette in its editor and then click "Export ...". |
| `texte-export-name` | Name | Name |
| `texte-export-ladeadresse` | Ladeadresse | Load address |
| `texte-export-adresse` | Adresse | Address |
| `texte-export-format` | AUSGABEFORMAT | OUTPUT FORMAT |
| `texte-export-umfang` | UMFANG | SCOPE |
| `texte-export-tab-text` | Quelltext | Source code |
| `texte-export-testen` | Im Emulator testen | Test in the emulator |
| `texte-export-speichern` | In Projekt speichern | Save to project |
| `texte-export-einfuegen` | In aktiven Code einfügen | Insert into active code |
| `texte-import-titel` | Dateien ins Projekt übernehmen | Add files to the project |
| `texte-import-hinweis` | Art, Kategorie und Zielordner sind Vorschläge; Namen lassen sich ändern. Bei einem Namenskonflikt wählst du Umbenennen, Ersetzen oder Überspringen. | Type, category and target folder are suggestions; names can be changed. In case of a name conflict you choose Rename, Replace or Skip. |
| `texte-palette-hinweis` | Die MEGA65-Palette hat 8 Bit je Kanal. Der Export kümmert sich um die vertauschten Halbbytes der Register $D100/$D200/$D300. | The MEGA65 palette has 8 bits per channel. The export takes care of the swapped nibbles of the registers $D100/$D200/$D300. |

## Kontextmenüs

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `kontext-menue` | Menü | Menu |
| `kontext-kontextmenue-problem` | Kontextmenü Problem | Context menu problem |
| `kontext-kontextmenue-protokoll` | Kontextmenü Protokoll | Context menu log |
| `kontext-kontextmenue-datei` | Kontextmenü Datei | Context menu file |
| `kontext-kontextmenue-dateibaum` | Kontextmenü Dateibaum | Context menu file tree |
| `kontext-kontextmenue-reiter` | Kontextmenü Reiter | Context menu tab |
| `kontext-kontextmenue-editor` | Kontextmenü Editor | Context menu editor |
| `kontext-kontextmenue-zeichenflaeche` | Kontextmenü Zeichenfläche | Context menu canvas |
| `kontext-kontextmenue-studio` | Kontextmenü Editor | Context menu editor |
| `kontext-hilfe-zum-wort` | Hilfe zum Wort | Help for word |

## Startseite

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `startui-vintagecoder` | VintageCoder | VintageCoder |
| `startui-entwicklungsumgebung-fuer-die-6502-familie-commodore` | Entwicklungsumgebung für die 6502-Familie: Commodore und Atari, Assembler und BASIC, Emulator und echte Hardware. | Development environment for the 6502 family: Commodore and Atari, assembler and BASIC, emulator and real hardware. |
| `startui-neues-projekt` | Neues Projekt | New project |
| `startui-projekt-oeffnen` | Projekt öffnen | Open project |
| `startui-beispiel-oeffnen` | Beispiel öffnen | Open example |
| `startui-einrichten` | Einrichten | Set up |
| `startui-einrichtung-unvollstaendig-ein-paar-schritte-sind` | Einrichtung unvollständig: ein paar Schritte sind noch offen. | Setup incomplete: a few steps are still open. |
| `startui-fertig-einrichten` | Fertig einrichten | Finish setup |
| `startui-neu-konfigurationsassistent-er-prueft-deine-einrichtung` | Neu: Konfigurationsassistent. Er prüft deine Einrichtung und ändert nur, was du bestätigst. | New: setup assistant. It checks your setup and changes only what you confirm. |
| `startui-assistent-oeffnen` | Assistent öffnen | Open assistant |
| `startui-beispielprojekte` | Beispielprojekte | Example projects |
| `startui-zuletzt-geoeffnete-projekte` | Zuletzt geöffnete Projekte | Recently opened projects |
| `startui-noch-kein-projekt-geoeffnet-neues-projekt` | Noch kein Projekt geöffnet. „Neues Projekt“ legt eines an, „Beispiel öffnen“ zeigt fertige Programme. | No project open yet. "New project" creates one, "Open example" shows finished programs. |
| `startui-ordner-fehlt` | Ordner fehlt | Folder missing |
| `startui-kopieren` | Kopieren … | Copy ... |
| `startui-oeffnen` | Öffnen | Open |
| `startui-studios` | Editoren | Editors |
| `startui-legt-eine-neue-datei-im-geoeffneten` | Legt eine neue Datei im geöffneten Projekt an. | Creates a new file in the open project. |
| `startui-studios-arbeiten-auf-dateien-eines-projekts` | Editoren arbeiten auf Dateien eines Projekts. Zuerst ein Projekt öffnen. | Editors work on the files of a project. Open a project first. |
| `startui-toolchain-und-hardware` | Toolchain und Hardware | Toolchain and hardware |
| `startui-erkannte-emulatoren` | ERKANNTE EMULATOREN | DETECTED EMULATORS |
| `startui-nur-windows` | nur Windows | Windows only |
| `startui-nicht-gefunden` | nicht gefunden | not found |
| `startui-echte-hardware` | ECHTE HARDWARE | REAL HARDWARE |
| `startui-keine-hardware-eingetragen` | Keine Hardware eingetragen | No hardware entered |
| `startui-nicht-erreichbar` | nicht erreichbar | not reachable |
| `startui-pruefe` | prüfe … | checking ... |
| `startui-ungeprueft` | ungeprüft | unchecked |
| `startui-testen` | Testen | Test |
| `startui-einstellungen-oeffnen` | Einstellungen öffnen | Open settings |
| `startui-konfigurationsassistent` | Konfigurationsassistent | Setup assistant |
| `startui-feedback-senden` | Feedback senden … | Send feedback ... |
| `startui-schnellreferenz` | Schnellreferenz | Quick reference |
| `startui-ueberblick` | Überblick | Overview |
| `startui-einstellungen` | Einstellungen | Settings |
| `startui-assembler` | Assembler | Assembler |
| `startui-debugger` | Debugger | Debugger |
| `startui-disk-studio` | Disk-Editor | Disk editor |
| `startui-tastenkuerzel` | Tastenkürzel | Keyboard shortcuts |
| `startui-bereit` | bereit | ready |

## Einrichtungsassistent

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `assistent-assistent-kein-konfigurationsordner` | Assistent: kein Konfigurationsordner. | Assistant: no configuration folder. |
| `assistent-assistent-e` | Assistent: {$e } | Assistant: {$e } |
| `assistent-einrichtung-uebernommen-p1` | Einrichtung übernommen{$p1 } | Setup applied{$p1 } |
| `assistent-einrichtung-rueckgaengig-gemacht` | Einrichtung rückgängig gemacht. | Setup undone. |
| `assistent-einrichtung` | Einrichtung | Setup |
| `assistent-schnelleinrichtung` | Schnelleinrichtung | Quick setup |
| `assistent-alles-selbst` | Alles selbst | Everything myself |
| `assistent-abbrechen` | Abbrechen | Cancel |
| `assistent-spaeter-fortsetzen` | Später fortsetzen | Continue later |
| `assistent-zurueck` | Zurück | Back |
| `assistent-uebernehmen` | Übernehmen | Apply |
| `assistent-fertig` | Fertig | Done |
| `assistent-weiter` | Weiter | Next |
| `assistent-der-assistent-richtet-zielsysteme-emulatoren-roms` | Der Assistent richtet Zielsysteme, Emulatoren, ROMs, Hardware und Aussehen ein. Er ändert nur, was du am Ende bestätigst, und sichert vorher deine bisherige Einstellungsdatei. | The assistant sets up target systems, emulators, ROMs, hardware and appearance. It changes only what you confirm at the end and first backs up your current settings file. |
| `assistent-schnelleinrichtung-7-schritte` | Schnelleinrichtung (7 Schritte) | Quick setup (7 steps) |
| `assistent-design-zielsysteme-toolchain-und-erstes-projekt` | Design, Zielsysteme, Toolchain und erstes Projekt; Emulatoren, ROMs und Hardware werden automatisch gefunden und später in den Einstellungen verfeinert. | Design, target systems, toolchain and first project; emulators, ROMs and hardware are found automatically and refined later in the settings. |
| `assistent-schnelleinrichtung-starten` | Schnelleinrichtung starten | Start quick setup |
| `assistent-alles-selbst-einstellen-12-schritte` | Alles selbst einstellen (12 Schritte) | Set up everything myself (12 steps) |
| `assistent-zusaetzlich-emulatoren-roms-echte-hardware-startziel` | Zusätzlich Emulatoren, ROMs, echte Hardware, Startziel F5 und Editor. | In addition: emulators, ROMs, real hardware, run target F5 and editor. |
| `assistent-alles-selbst-einstellen` | Alles selbst einstellen | Set up everything myself |
| `assistent-nur-einen-teil` | Nur einen Teil | Only one part |
| `assistent-roms` | ROMs | ROMs |
| `assistent-emulatoren` | Emulatoren | Emulators |
| `assistent-hardware` | Hardware | Hardware |
| `assistent-design-und-editor` | Design und Editor | Design and editor |
| `assistent-offen-von-der-letzten-einrichtung-p1` | Offen von der letzten Einrichtung: {$p1 } | Left open from the last setup: {$p1 } |
| `assistent-design` | Design | Design |
| `assistent-gilt-fuer-die-ganze-oberflaeche-und` | Gilt für die ganze Oberfläche und wirkt sofort; „Abbrechen“ stellt das Alte wieder her. | Applies to the whole interface and takes effect at once; "Cancel" restores the old state. |
| `assistent-farbschema` | Farbschema | Color scheme |
| `assistent-dunkel` | Dunkel | Dark |
| `assistent-hell` | Hell | Light |
| `assistent-system` | System | System |
| `assistent-oberflaeche` | Oberfläche | UI size |
| `assistent-editor-schrift` | Editor-Schrift | Editor font |
| `assistent-p1-pt` | {$p1 } pt | {$p1 } pt |
| `assistent-zielsysteme` | Zielsysteme | Target systems |
| `assistent-fuer-welche-systeme-willst-du-programmieren` | Für welche Systeme willst du programmieren? Abgewählte Systeme werden nur ausgeblendet, nichts geht verloren. | Which systems do you want to program for? Deselected systems are only hidden, nothing is lost. |
| `assistent-vintagecoder-hat-einen-eigenen-assembler-es` | VintageCoder hat einen eigenen Assembler; es wird kein externer Assembler benötigt. Emulatoren werden gesucht (PATH und übliche Ordner), nichts wird gestartet oder installiert. | VintageCoder has its own assembler; no external assembler is needed. Emulators are searched for (PATH and usual folders); nothing is started or installed. |
| `assistent-nicht-gefunden` | nicht gefunden | not found |
| `assistent-fehlende-emulatoren-traegst-du-im-schritt` | Fehlende Emulatoren trägst du im Schritt „Emulatoren“ (Vollweg) oder später unter Einstellungen ▸ Zielsysteme ein; dort gibt es auch die portable Installation. | Enter missing emulators in the step "Emulators" (full path) or later under Settings ▸ Target systems; the portable installation is there as well. |
| `assistent-toolchain` | Toolchain | Toolchain |
| `assistent-was-ist-auf-diesem-rechner-schon` | Was ist auf diesem Rechner schon da? | What is already on this computer? |
| `assistent-emulatoren-je-zielsystem-pfad-leer-automatisch` | Emulatoren je Zielsystem: Pfad (leer = automatisch suchen), Standard, Startparameter. | Emulators per target system: path (empty = search automatically), default, start parameters. |
| `assistent-echte-maschinen-ultimate-mega65-adresse-bzw` | Echte Maschinen (Ultimate, MEGA65): Adresse bzw. Schnittstelle, Verbindungstest nur auf Klick. | Real machines (Ultimate, MEGA65): address or interface; the connection test runs only on click. |
| `assistent-startziel-f5-je-system-emulator-oder` | Startziel F5 je System: Emulator oder eine eingetragene Maschine. | Run target F5 per system: emulator or a machine you entered. |
| `assistent-projektordner` | Projektordner | Project folder |
| `assistent-leer-voreinstellung-vintagecoder-das-ist-der` | Leer = Voreinstellung ~/VintageCoder. Das ist der vorgeschlagene Ort für neue Projekte. | Empty = default ~/VintageCoder. This is the suggested place for new projects. |
| `assistent-beispielprojekte-in-meinen-projektordner-kopieren-ordner` | Beispielprojekte in meinen Projektordner kopieren (Ordner „Beispiele“) | Copy example projects to my project folder (folder "Examples") |
| `assistent-spaeter-entscheiden` | Später entscheiden | Decide later |
| `assistent-neues-projekt-anlegen` | Neues Projekt anlegen | Create a new project |
| `assistent-beispiel-p1-oeffnen` | Beispiel „{$p1 }“ öffnen | Open example "{$p1 }" |
| `assistent-erstes-projekt` | Erstes Projekt | First project |
| `assistent-projekte` | Projekte | Projects |
| `assistent-wo-liegen-deine-projekte-und-womit` | Wo liegen deine Projekte, und womit fängst du an? | Where are your projects, and what do you start with? |
| `assistent-es-gibt-nichts-zu-aendern-alles` | Es gibt nichts zu ändern: alles ist schon so eingerichtet. | There is nothing to change: everything is already set up this way. |
| `assistent-zusammenfassung` | Zusammenfassung | Summary |
| `assistent-n-aenderung-en-werden-uebernommen-vorher` | {$n } Änderung(en) werden übernommen. Vorher entsteht eine Sicherung (settings.json.bak); danach lässt sich alles rückgängig machen. | {$n -> ⏎ [one] 1 change ⏎ *[other] {$n } changes ⏎ } will be applied. A backup is made first (settings.json.bak); afterwards everything can be undone. |
| `assistent-die-einrichtung-ist-uebernommen-alles-laesst` | Die Einrichtung ist übernommen. Alles lässt sich unter Einstellungen jederzeit ändern; der Assistent ist über die Palette („Konfigurationsassistent“) erneut erreichbar. | The setup has been applied. You can change everything at any time under Settings; the assistant can be started again from the palette ("Setup assistant"). |
| `assistent-sicherung-p1-2` | Sicherung: {$p1 } | Backup: {$p1 } |
| `assistent-beispiele-kopiert-p1` | Beispiele kopiert: {$p1 } | Examples copied: {$p1 } |
| `assistent-rueckgaengig` | Rückgängig | Undo |
| `assistent-sicherung-p1` | (Sicherung {$p1 }) | (backup {$p1 }) |
| `assistent-bereit` | bereit | ready |
| `assistent-neu` | neu | new |
| `assistent-geaendert` | geändert | changed |
| `assistent-entfernt` | entfernt | removed |

## Assistent: Änderungsliste und Zusammenfassung

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `setup-willkommen` | Willkommen | Welcome |
| `setup-design` | Design | Design |
| `setup-zielsysteme` | Zielsysteme | Target systems |
| `setup-toolchain` | Toolchain | Toolchain |
| `setup-emulatoren` | Emulatoren | Emulators |
| `setup-roms` | ROMs | ROMs |
| `setup-echte-hardware` | Echte Hardware | Real hardware |
| `setup-startziel-f5` | Startziel F5 | Start target F5 |
| `setup-editor` | Editor | Editor |
| `setup-projekte` | Projekte | Projects |
| `setup-zusammenfassung` | Zusammenfassung | Summary |
| `setup-fertig` | Fertig | Done |
| `setup-start` | Start | Start |
| `setup-systeme` | Systeme | Systems |
| `setup-maschinen` | Maschinen | Machines |
| `setup-arbeitsplatz` | Arbeitsplatz | Workspace |
| `setup-abschluss` | Abschluss | Finish |
| `setup-nicht-gesetzt` | nicht gesetzt | not set |
| `setup-pfad-automatisch` | Pfad: automatisch | Path: automatic |
| `setup-pfad-p1` | Pfad: {$p1 } | Path: {$p1 } |
| `setup-standard` | Standard | Default |
| `setup-argumente-p1` | Argumente: {$p1 } | Arguments: {$p1 } |
| `setup-p1-parameter` | {$p1 } Parameter | {$p1 } parameters |
| `setup-rom-p1` | ROM: {$p1 } | ROM: {$p1 } |
| `setup-schnittstelle-p1` | Schnittstelle: {$p1 } | Port: {$p1 } |
| `setup-baudrate-p1` | Baudrate {$p1 } | Baud rate {$p1 } |
| `setup-adresse-p1` | Adresse: {$p1 } | Address: {$p1 } |
| `setup-passwort-p1` | Passwort: {$p1 } | Password: {$p1 } |
| `setup-run-danach` | RUN danach | RUN afterwards |
| `setup-nur-laden` | nur laden | load only |
| `setup-emulator` | Emulator | Emulator |
| `setup-id-fehlt` | {$id } (fehlt) | {$id } (missing) |
| `setup-der-ordner-darf-nicht-leer-sein` | Der Ordner darf nicht leer sein | The folder must not be empty |
| `setup-bitte-einen-vollstaendigen-pfad-angeben-beginnend` | Bitte einen vollständigen Pfad angeben (beginnend mit / oder ~/) | Please enter a full path (starting with / or ~/) |
| `setup-das-ist-ein-systemordner-bitte-einen` | Das ist ein Systemordner; bitte einen Ordner in deinem Benutzerordner wählen | This is a system folder; please choose a folder in your user folder |
| `setup-der-ordner-liegt-innerhalb-der-installation` | Der Ordner liegt innerhalb der Installation von VintageCoder; sie wird bei Updates ersetzt | The folder is inside the VintageCoder installation; it is replaced on updates |
| `setup-der-ordner-ist-nicht-schreibbar` | Der Ordner ist nicht schreibbar | The folder is not writable |
| `setup-der-ordner-laesst-sich-hier-nicht` | Der Ordner lässt sich hier nicht anlegen | The folder cannot be created here |
| `setup-zielsystem-p1` | Zielsystem {$p1 } | Target system {$p1 } |
| `setup-kein-emulator` | kein Emulator | no emulator |
| `setup-ausgeblendete-eintraege-werden-wiederhergestellt` | ausgeblendete Einträge werden wiederhergestellt | hidden entries are restored |
| `setup-emulatoren-und-startziel-bleiben-erhalten-wiedereinblenden` | Emulatoren und Startziel bleiben erhalten; Wiedereinblenden stellt sie her (Sicherung: settings.json.bak) | Emulators and start target are kept; showing them again restores them (backup: settings.json.bak) |
| `setup-p1-f5-startet-auf` | {$p1 }: F5 startet auf | {$p1 }: F5 starts on |
| `setup-maschine-p1` | Maschine {$p1 } | Machine {$p1 } |
| `setup-passwort-geaendert` | (Passwort geändert) | (password changed) |
| `setup-projektordner` | Projektordner | Project folder |
| `setup-verschiebt-nichts-nur-die-voreinstellung-fuer` | verschiebt nichts; nur die Voreinstellung für neue Projekte ändert sich | moves nothing; only the default for new projects changes |
| `setup-p1-beispiele-kopieren` | {$p1 } Beispiele kopieren | Copy {$p1 } examples |
| `setup-nach-p1-missing-fehlen-noch-vorhandene` | nach {$p1 } ({$missing } fehlen noch, vorhandene Dateien bleiben unberührt) | to {$p1 } ({$missing } still missing, existing files stay untouched) |
| `setup-farbschema` | Farbschema | Color scheme |
| `setup-oberflaeche` | Oberfläche | Interface |
| `setup-editorschrift` | Editorschrift | Editor font |
| `setup-p1-pt` | {$p1 } pt | {$p1 } pt |
| `setup-roms-anderer-programme-mitbenutzen` | ROMs anderer Programme mitbenutzen | Use ROMs of other programs |
| `setup-assembler-optionen` | Assembler-Optionen | Assembler options |
| `setup-p1-optionen` | {$p1 } Optionen | {$p1 } options |
| `setup-datenstroeme-der-ultimate` | Datenströme der Ultimate | Data streams of the Ultimate |
| `setup-hell` | Hell | Light |
| `setup-system` | System | System |
| `setup-dunkel` | Dunkel | Dark |
| `setup-p1-startziel-zeigte-auf-eine-nicht` | {$p1 }: Startziel zeigte auf eine nicht übernommene Maschine; F5 startet wieder im Emulator | {$p1 }: the start target pointed to a machine that was not taken over; F5 starts in the emulator again |
| `setup-die-konfiguration-hat-sich-geaendert-seit` | Die Konfiguration hat sich geändert, seit der Assistent sie geladen hat. Bitte den Assistenten neu starten. | The configuration has changed since the assistant loaded it. Please restart the assistant. |
| `setup-sicherung-e` | Sicherung {$e } | Backup {$e } |
| `setup-rueckbau-unvollstaendig-sicherung-settings-json-bak` | (Rückbau unvollständig, Sicherung: settings.json.bak) | (undo incomplete, backup: settings.json.bak) |
| `setup-aenderungen-zurueckgenommen` | (Änderungen zurückgenommen) | (changes undone) |
| `setup-kein-ordner-fuer-die-einstellungen-home` | Kein Ordner für die Einstellungen (HOME bzw. unter Windows APPDATA fehlt) | No folder for the settings (HOME, or APPDATA on Windows, is missing) |
| `setup-sys-pfad-von-p2-existiert-nicht` | {$sys }: Pfad von {$p2 } existiert nicht mehr | {$sys }: path of {$p2 } no longer exists |
| `setup-sys-rom-ort-von-p2-existiert` | {$sys }: ROM-Ort von {$p2 } existiert nicht mehr | {$sys }: ROM location of {$p2 } no longer exists |
| `setup-p1-adresse-bzw-schnittstelle-fehlt` | {$p1 }: Adresse bzw. Schnittstelle fehlt | {$p1 }: address or port missing |
| `setup-projektordner-existiert-nicht-wird-beim-ersten` | Projektordner existiert nicht (wird beim ersten Projekt angelegt) | Project folder does not exist (it is created with the first project) |
| `setup-p1-p2-gefunden-noch-nicht-eingetragen` | {$p1 }: {$p2 } gefunden, noch nicht eingetragen | {$p1 }: {$p2 } found, not entered yet |
| `setup-keine-gueltige-exportdatei-e` | Keine gültige Exportdatei: {$e } | Not a valid export file: {$e } |
| `setup-das-ist-keine-exportdatei-von-vintagecoder` | Das ist keine Exportdatei von VintageCoder | This is not an export file of VintageCoder |
| `setup-die-datei-stammt-aus-einer-neueren` | Die Datei stammt aus einer neueren Version (Format {$ver }); bitte VintageCoder aktualisieren | The file comes from a newer version (format {$ver }); please update VintageCoder |
| `setup-einstellungen-in-der-datei-unlesbar-e` | Einstellungen in der Datei unlesbar: {$e } | Settings in the file are unreadable: {$e } |
| `setup-p1-pfad-existiert-hier-nicht-bitte` | {$p1 }: Pfad existiert hier nicht, bitte neu suchen: {$p2 } | {$p1 }: path does not exist here, please search again: {$p2 } |
| `setup-p1-rom-ort-existiert-hier-nicht` | {$p1 }: ROM-Ort existiert hier nicht: {$p2 } | {$p1 }: ROM location does not exist here: {$p2 } |
| `setup-assembler-eingebaut` | Assembler (eingebaut) | Assembler (built in) |
| `setup-von-vintagecoder-portabel-installierte-programme` | von VintageCoder portabel installierte Programme | programs installed portably by VintageCoder |
| `setup-gesetzt` | gesetzt | set |
| `setup-ja` | ja | yes |
| `setup-nein` | nein | no |
| `setup-eingerichtet` | eingerichtet | set up |
| `setup-ausgeblendet` | ausgeblendet | hidden |

## Einstellungen

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `setui-gespeichert` | Gespeichert. | Saved. |
| `setui-speichern-fehlgeschlagen-e` | Speichern fehlgeschlagen: {$e } | Saving failed: {$e } |
| `setui-ungespeicherte-aenderungen` | ● Ungespeicherte Änderungen | ● Unsaved changes |
| `setui-verwerfen` | Verwerfen | Discard |
| `setui-speichern` | Speichern | Save |
| `setui-groesse-der-ganzen-oberflaeche-auch-mit` | Größe der ganzen Oberfläche; auch mit Cmd + und −, zurück mit Cmd 0. | Size of the whole interface; also with Cmd + and -, back with Cmd 0. |
| `setui-konfiguration` | Konfiguration | Configuration |
| `setui-exportieren` | Exportieren … | Export ... |
| `setui-importieren` | Importieren … | Import ... |
| `setui-zuruecksetzen` | Zurücksetzen … | Reset ... |
| `setui-exportieren-schreibt-die-einstellungen-der-ide` | Exportieren schreibt die Einstellungen der IDE (Zielsysteme, Emulatoren, Maschinen, Parameter) in eine JSON-Datei, ohne Passwörter. Importieren zeigt vorher, was drinsteht, und ersetzt dann die jetzigen Einstellungen; Zurücksetzen stellt die Vorgabe her. Farbschema und Zoom gehören nicht dazu. | Export writes the settings of the IDE (target systems, emulators, machines, parameters) to a JSON file, without passwords. Import first shows what is in it and then replaces the current settings; Reset restores the defaults. Color scheme and zoom are not included. |
| `setui-assistent` | Assistent | Assistant |
| `setui-assistent-starten` | Assistent starten | Start assistant |
| `setui-zuletzt-ausgefuehrt-d` | zuletzt ausgeführt: {$d } | last run: {$d } |
| `setui-noch-nicht-ausgefuehrt` | noch nicht ausgeführt | not run yet |
| `setui-debug-protokoll` | Debug-Protokoll | Debug log |
| `setui-protokolldatei-schreiben` | Protokolldatei schreiben | Write log file |
| `setui-aus` | Aus | Off |
| `setui-umgebungsvariable-vc-debug-log-ueberstimmt-die` | Umgebungsvariable VC_DEBUG_LOG überstimmt die Einstellung: {$v } | The environment variable VC_DEBUG_LOG overrides the setting: {$v } |
| `setui-datei-p1` | Datei: {$p1 } | File: {$p1 } |
| `setui-pfade-im-persoenlichen-ordner-stehen-als` | Pfade im persönlichen Ordner stehen als ~, Passwörter und Tokens werden geschwärzt. Dateinamen und Fehlermeldungen deines Projekts können vorkommen: vor dem Weitergeben ansehen. Ab 5 MB beginnt eine neue Datei. | Paths in your personal folder appear as ~, passwords and tokens are blacked out. File names and error messages of your project may appear: look at it before passing it on. A new file starts at 5 MB. |
| `setui-bilder-laden` | Bilder laden | Load images |
| `setui-fragen` | Fragen | Ask |
| `setui-immer` | Immer | Always |
| `setui-nie` | Nie | Never |
| `setui-adresse` | Adresse | Address |
| `setui-umgebungsvariable-vc-help-images-ueberstimmt-die` | Umgebungsvariable VC_HELP_IMAGES überstimmt die Adresse: {$v } | The environment variable VC_HELP_IMAGES overrides the address: {$v } |
| `setui-ein-ordner-statt-einer-webadresse-wird` | Ein Ordner statt einer Webadresse wird gelesen statt geladen. Beim Laden wird nur die Adresse des Bildes abgerufen; geladene Bilder bleiben im Zwischenspeicher. | A folder instead of a web address is read instead of downloaded. When downloading, only the address of the image is requested; downloaded images stay in the cache. |
| `setui-zwischenspeicher-p1` | Zwischenspeicher: {$p1 } | Cache: {$p1 } |
| `setui-darstellung` | Darstellung | Appearance |
| `setui-gilt-fuer-die-ganze-oberflaeche-und` | Gilt für die ganze Oberfläche und wirkt sofort. | Applies to the whole interface and takes effect at once. |
| `setui-konfiguration-sichern` | Konfiguration sichern | Back up the configuration |
| `setui-einstellungen-in-eine-datei-schreiben-von` | Einstellungen in eine Datei schreiben, von dort übernehmen oder auf die Vorgabe zurücksetzen. | Write the settings to a file, import them from there, or reset them to the defaults. |
| `setui-einrichtung` | Einrichtung | Setup |
| `setui-der-konfigurationsassistent-prueft-die-einrichtung-und` | Der Konfigurationsassistent prüft die Einrichtung und ändert nur, was du bestätigst. | The setup assistant checks your setup and changes only what you confirm. |
| `setui-fehlersuche` | Fehlersuche | Troubleshooting |
| `setui-das-debug-protokoll-schreibt-befehle-meldungen` | Das Debug-Protokoll schreibt Befehle, Meldungen, Fehler und Abstürze in eine Datei auf deinem Rechner; nichts wird gesendet. | The debug log writes commands, messages, errors and crashes to a file on your computer; nothing is sent. |
| `setui-hilfe` | Hilfe | Help |
| `setui-die-bilder-des-handbuchs-liegen-im` | Die Bilder des Handbuchs liegen im Internet und werden erst nach deiner Zustimmung geladen; die Wahl wirkt sofort und wird gespeichert. | The images of the manual are on the internet and are loaded only after you agree; the choice takes effect at once and is saved. |
| `setui-nach-updates-suchen` | Nach Updates suchen | Check for updates |
| `setui-nur-auf-wunsch` | Nur auf Wunsch | Only on request |
| `setui-taeglich-beim-start` | Täglich beim Start | Daily at start |
| `setui-woechentlich-beim-start` | Wöchentlich beim Start | Weekly at start |
| `setui-noch-nicht-entschieden-es-wird-nichts` | Noch nicht entschieden: es wird nichts gesendet. | Not decided yet: nothing is sent. |
| `setui-kanal` | Kanal | Channel |
| `setui-stabil-nur-fertige-versionen` | Stabil (nur fertige Versionen) | Stable (finished versions only) |
| `setui-jetzt-pruefen` | Jetzt prüfen | Check now |
| `setui-pruefe` | Prüfe … | Checking ... |
| `setui-noch-nicht-geprueft` | noch nicht geprüft | not checked yet |
| `setui-uebersprungen` | Übersprungen | Skipped |
| `setui-version-p1` | Version {$p1 } | Version {$p1 } |
| `setui-zuruecknehmen` | Zurücknehmen | Undo skip |
| `setui-umgebungsvariable-vc-update-url-ueberstimmt-die` | Umgebungsvariable VC_UPDATE_URL überstimmt die Adresse: {$v } | The environment variable VC_UPDATE_URL overrides the address: {$v } |
| `setui-die-adresse-betrifft-nur-die-versionsliste` | Die Adresse betrifft nur die Versionsliste (https oder ein Ordner); der Link zur Releaseseite ist im Programm fest. Übertragen wird eine Anfrage nach der Versionsliste (update.json) an github.com: GitHub sieht deine IP-Adresse und den Programmnamen „VintageCoder-update“, ohne Versionsnummer. Keine Kennung, kein Konto, keine Angaben zu Rechner, Projekten oder Einstellungen. | The address only concerns the version list (https or a folder); the link to the release page is fixed in the program. One request for the version list (update.json) is sent to github.com: GitHub sees your IP address and the program name "VintageCoder-update", without a version number. No ID, no account, no information about your computer, projects or settings. |
| `setui-updates` | Updates | Updates |
| `setui-vintagecoder-kann-nachsehen-ob-es-eine` | VintageCoder kann nachsehen, ob es eine neue Version gibt, und zeigt die Änderungen mit einem Link zur Releaseseite. Es lädt und installiert nichts selbst. Ohne deine Zustimmung geht nie etwas ins Netz. | VintageCoder can check whether there is a new version and shows the changes with a link to the release page. It does not download or install anything itself. Nothing ever goes to the network without your consent. |
| `setui-dateien` | Dateien | Files |
| `setui-wo-vintagecoder-einstellungen-ablegt` | Wo VintageCoder Einstellungen ablegt. | Where VintageCoder stores its settings. |
| `setui-ordner` | Ordner | Folder |
| `setui-einstellungen` | Einstellungen | Settings |
| `setui-die-einstellungsdatei-enthaelt-die-passwoerter-der` | Die Einstellungsdatei enthält die Passwörter der Hardware und ist nur für dich lesbar. | The settings file contains the passwords of the hardware and is readable only by you. |
| `setui-im-ordner-zeigen` | Im Ordner zeigen | Show in folder |
| `setui-assembler` | Assembler | Assembler |
| `setui-eingebaut-vc-asm` | eingebaut (vc-asm) | built in (vc-asm) |
| `setui-include-suche` | Include-Suche | Include search |
| `setui-1-ordner-des-projekts` | 1. Ordner des Projekts | 1. Folder of the project |
| `setui-2-zusaetzliche-ordner-unten-weitere-parameter` | 2. Zusätzliche Ordner (unten: weitere Parameter, -I) | 2. Additional folders (below: more parameters, -I) |
| `setui-3-systemdateien-p1` | 3. Systemdateien: {$p1 } | 3. System files: {$p1 } |
| `setui-assembler-optionen-beim-bauen` | Assembler-Optionen beim Bauen | Assembler options when building |
| `setui-vcc-asm` | vcc asm | vcc asm |
| `setui-z-b-i-pfad-zu-includes` | z. B. -I /pfad/zu/includes -D DEBUG=1 | e.g. -I /path/to/includes -D DEBUG=1 |
| `setui-gilt-fuer-das-bauen-in-der` | Gilt für das Bauen in der IDE (nicht für vcc auf der Kommandozeile). Konstanten aus der Projektdatei kommen zuerst, diese danach. | Applies to building in the IDE (not to vcc on the command line). Constants from the project file come first, these after. |
| `setui-basic-bas-zeilennummern-optional-gibt-es` | BASIC (.bas, Zeilennummern optional) gibt es für alle Dialekte. | BASIC (.bas, line numbers optional) is available for all dialects. |
| `setui-projekt` | Projekt | Project |
| `setui-kein-projekt-geoeffnet` | kein Projekt geöffnet | no project open |
| `setui-p1-auto-export-in-den-projekt` | {$p1 } (Auto-Export in den Projekt-Einstellungen) | {$p1 } (auto export in the project settings) |
| `setui-gilt-je-projekt-und-steht-in` | Gilt je Projekt und steht in dessen Einstellungen. | Applies per project and is stored in its settings. |
| `setui-vintagecoder-hat-einen-eigenen-assembler-es` | VintageCoder hat einen eigenen Assembler; es wird kein externer Assembler benötigt. | VintageCoder has its own assembler; no external assembler is needed. |
| `setui-basic-dialekt-je-zielsystem` | BASIC-Dialekt je Zielsystem | BASIC dialect per target system |
| `setui-das-basic-eines-projekts-folgt-dem` | Das BASIC eines Projekts folgt dem Dialekt seines Zielsystems; je Quelle lässt er sich in der Projektdatei ändern. | The BASIC of a project follows the dialect of its target system; it can be changed per source in the project file. |
| `setui-export-beim-bauen` | Export beim Bauen | Export when building |
| `setui-sprites-zeichensaetze-und-songs-werden-vor` | Sprites, Zeichensätze und Songs werden vor dem Assemblieren nach build/export/ geschrieben, wenn sie veraltet sind. | Sprites, character sets and songs are written to build/export/ before assembling if they are out of date. |
| `setui-an` | An | On |
| `setui-menueleiste-anzeigen` | Menüleiste anzeigen | Show menu bar |
| `setui-unter-macos-steht-die-menueleiste-des` | Unter macOS steht die Menüleiste des Systems oben am Bildschirm. | On macOS, the menu bar of the system is at the top of the screen. |
| `setui-zeile-beim-verlassen-formatieren` | Zeile beim Verlassen formatieren | Format line when leaving it |
| `setui-assembler-und-basic-leerraeume-und-kommentarspalte` | Assembler und BASIC: Leerräume und Kommentarspalte werden aufgeräumt, sobald du die Zeile verlässt (nie bei Auswahl). Auf Befehl: Bearbeiten ▸ Zeile / Datei formatieren. | Assembler and BASIC: white space and the comment column are tidied up as soon as you leave the line (never while a selection exists). On demand: Edit ▸ Format line / file. |
| `setui-einzeltasten-der-studios` | Einzeltasten der Editoren | Single keys of the editors |
| `setui-kuerzel-diagnose` | Kürzel-Diagnose | Shortcut diagnostics |
| `setui-schreibt-zu-jedem-kuerzel-taste-umschalttasten` | Schreibt zu jedem Kürzel (Taste, Umschalttasten, gefundener Befehl, Menü-Auslösung) eine Zeile ins Protokoll. Nur lokal, nichts wird übertragen. | Writes one line to the log for every shortcut (key, modifier keys, command found, menu trigger). Local only, nothing is transmitted. |
| `setui-tastaturlayout` | Tastaturlayout | Keyboard layout |
| `setui-automatisch` | Automatisch | Automatic |
| `setui-die-klaviertasten-des-trackers-gelten-nach` | Die Klaviertasten des Trackers gelten nach der Position der Taste (wie bei US-QWERTY); das Layout bestimmt nur, welche Buchstaben die Hinweise nennen. Jetzt erkannt: {$p1 }. | The piano keys of the tracker apply by the position of the key (as on a US QWERTY keyboard); the layout only decides which letters the hints show. Detected now: {$p1 }. |
| `setui-tasten-auf-der-klaviatur` | Tasten auf der Klaviatur | Keys on the keyboard |
| `setui-buchstaben-und-zifferntasten-in-den-studios` | Buchstaben- und Zifferntasten in den Editoren (D, E, F … für Werkzeuge, Z S X … für Noten, 1–4 für Zeichenfarben). Aus: Sie tippen nichts mehr versehentlich; Pfeile, Leertaste, Entf, Esc und alle Kürzel mit Cmd oder Alt bleiben, die Werkzeuge wählen Sie dann mit der Maus oder über die Befehlspalette. | Letter and digit keys in the editors (D, E, F ... for tools, Z S X ... for notes, 1–4 for character colors). Off: you no longer type anything by accident; arrows, space, Del, Esc and all shortcuts with Cmd or Alt remain, and you then choose tools with the mouse or the command palette. |
| `setui-startbild` | Startbild | Start screen |
| `setui-takt-in-der-statusleiste` | Takt in der Statusleiste | Clock in the status bar |
| `setui-zeigt-unten-takt-norm-pal-ntsc` | Zeigt unten Takt, Norm (PAL/NTSC), Zeilen je Bild und Zyklen je Zeile des Zielsystems, beim Debuggen auch den Programmzähler. Nur eine Anzeige aus den Systemdaten: nichts wird erhoben oder gesendet. | Shows at the bottom the clock, standard (PAL/NTSC), lines per frame and cycles per line of the target system, and the program counter while debugging. Only a display from the system data: nothing is collected or sent. |
| `setui-sicherung` | Sicherung | Backup |
| `setui-unbenannte-reiter-nach-einem-absturz-wiederherstellen` | Unbenannte Reiter nach einem Absturz wiederherstellen | Restore untitled tabs after a crash |
| `setui-der-inhalt-unbenannter-reiter-wird-alle` | Der Inhalt unbenannter Reiter wird alle paar Sekunden in scratch.json im Konfigurationsordner gesichert: nur auf diesem Rechner, nie im Projekt, unverschlüsselt, nach 14 Tagen gelöscht. Beim Abschalten verschwindet die Sicherung sofort. | The content of untitled tabs is saved every few seconds in scratch.json in the configuration folder: only on this computer, never in the project, unencrypted, deleted after 14 days. Turning it off removes the backup immediately. |
| `setui-lange-zeilen-werden-an-der-sichtbaren` | Lange Zeilen werden an der sichtbaren Breite umbrochen (nur Darstellung). Der Text bleibt unverändert; Zeilennummern und Haltepunkte gehören zur ersten Zeile. | Long lines are wrapped at the visible width (display only). The text stays unchanged; line numbers and breakpoints belong to the first line. |
| `setui-editor` | Editor | Editor |
| `setui-gilt-fuer-alle-quelltexte-und-wirkt` | Gilt für alle Quelltexte und wirkt sofort. | Applies to all source files and takes effect at once. |
| `setui-tastenkuerzel` | Tastenkürzel | Keyboard shortcuts |
| `setui-die-kuerzel-sind-fest-anpassen-ist` | Die Kürzel sind fest; Anpassen ist noch nicht möglich. Die Liste kommt aus der Befehlstabelle und gilt auch für Menü und Palette. | The shortcuts are fixed; customizing is not possible yet. The list comes from the command table and also applies to the menu and the palette. |
| `setui-befehl-oder-kuerzel-suchen` | Befehl oder Kürzel suchen | Search for a command or shortcut |
| `setui-nichts-gefunden` | Nichts gefunden. | Nothing found. |
| `setui-p1-pt` | {$p1 } pt | {$p1 } pt |
| `setui-seite-allgemein` | Allgemein | General |
| `setui-seite-zielsysteme-und-hardware` | Zielsysteme und Hardware | Target systems and hardware |
| `setui-seite-roms` | ROMs | ROMs |
| `setui-seite-toolchain` | Toolchain | Toolchain |
| `setui-seite-editor` | Editor | Editor |
| `setui-seite-tastenkuerzel` | Tastenkürzel | Keyboard shortcuts |
| `setui-auch-turbo-basic-xl` | auch Turbo BASIC XL | also Turbo BASIC XL |
| `setui-an-klein` | an | on |
| `setui-aus-klein` | aus | off |

## Neues Projekt

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `neuprojekt-keine-vorlage-gewaehlt` | keine Vorlage gewählt | no template chosen |
| `neuprojekt-zeichensatz-nicht-angelegt-e` | Zeichensatz nicht angelegt: {$e } | Character set not created: {$e } |
| `neuprojekt-git-init-ist-fehlgeschlagen-ist-git` | git init ist fehlgeschlagen (ist Git installiert?). .gitignore wurde trotzdem angelegt. | git init failed (is Git installed?). .gitignore was created anyway. |
| `neuprojekt-neues-projekt-data-templates-index-json` | Neues Projekt: data/templates/index.json fehlt. | New project: data/templates/index.json is missing. |
| `neuprojekt-projekt-angelegt-p1` | Projekt angelegt: {$p1 } | Project created: {$p1 } |
| `neuprojekt-hardware-profil` | Hardware-Profil | Hardware profile |
| `neuprojekt-gueltig` | gültig | valid |
| `neuprojekt-cpu-p1-p2` | CPU {$p1 } · {$p2 } | CPU {$p1 } · {$p2 } |
| `neuprojekt-assembler` | Assembler | Assembler |
| `neuprojekt-fuer-diese-kombination-aus-zielsystem-und` | Für diese Kombination aus Zielsystem und Sprache gibt es keine Vorlage. | There is no template for this combination of target system and language. |
| `neuprojekt-basic-wird-im-dialekt-des-zielsystems` | BASIC wird im Dialekt des Zielsystems geschrieben, ohne Zeilennummern und mit Sprungmarken (@name). Beim Bauen entsteht daraus das native Programm. | BASIC is written in the dialect of the target system, without line numbers and with jump labels (@name). Building turns it into the native program. |
| `neuprojekt-projekt-details` | PROJEKT-DETAILS | PROJECT DETAILS |
| `neuprojekt-projektname` | Projektname | Project name |
| `neuprojekt-nur-buchstaben-ziffern-und` | Nur Buchstaben, Ziffern, _ und - | Letters, digits, _ and - only |
| `neuprojekt-speicherort` | Speicherort | Location |
| `neuprojekt-waehlen` | Wählen … | Choose ... |
| `neuprojekt-basic-dialekt` | BASIC-Dialekt | BASIC dialect |
| `neuprojekt-videonorm` | Videonorm | Video standard |
| `neuprojekt-speicherausbau` | Speicherausbau | Memory expansion |
| `neuprojekt-standard-emulator` | Standard-Emulator | Default emulator |
| `neuprojekt-p1-einstellungen` | {$p1 } (Einstellungen) | {$p1 } (settings) |
| `neuprojekt-weicht-von-den-einstellungen-ab-wird` | weicht von den Einstellungen ab: wird in der Projektdatei gespeichert | differs from the settings: stored in the project file |
| `neuprojekt-aus-den-einstellungen` | aus den Einstellungen | from the settings |
| `neuprojekt-emulator` | Emulator | Emulator |
| `neuprojekt-p1-hardware` | {$p1 } (Hardware) | {$p1 } (hardware) |
| `neuprojekt-startziel-f5` | Startziel (F5) | Run target (F5) |
| `neuprojekt-diskette` | Diskette | Disk |
| `neuprojekt-dein-rom-hat-vorrang-nur-ein` | Dein ROM hat Vorrang (nur ein Verweis in der Datei, nichts Fremdes im Projekt); sonst der eigene Satz VC-Standard als Kopie. | Your ROM takes precedence (only a reference in the file, nothing foreign in the project); otherwise our own VC Standard set as a copy. |
| `neuprojekt-immer-der-eigene-satz-vc-standard` | Immer der eigene Satz VC-Standard (CC0) als Kopie mit HERKUNFT.txt: das Projekt lässt sich weitergeben. | Always our own VC Standard set (CC0) as a copy with HERKUNFT.txt: the project can be passed on. |
| `neuprojekt-kein-zeichensatz-du-legst-spaeter-im` | Kein Zeichensatz; du legst später im Zeichensatz-Editor einen an. | No character set; you create one later in the character set editor. |
| `neuprojekt-zeichensatz-des-projekts` | Zeichensatz des Projekts | Character set of the project |
| `neuprojekt-automatisch` | Automatisch | Automatic |
| `neuprojekt-nur-standard-eigen` | Nur Standard (eigen) | Standard only (own) |
| `neuprojekt-keinen` | Keinen | None |
| `neuprojekt-readme-md-mit-bauhinweisen-anlegen` | README.md mit Bauhinweisen anlegen | Create README.md with build notes |
| `neuprojekt-git-repository-anlegen-gitignore-fuer-build` | Git-Repository anlegen (.gitignore für build/ und .vcbackup/) | Create a Git repository (.gitignore for build/ and .vcbackup/) |
| `neuprojekt-weitere-zielsysteme` | Weitere Zielsysteme | More target systems |
| `neuprojekt-dateistruktur` | Dateistruktur | File structure |
| `neuprojekt-n-dateien` | {$n } Dateien | {$n } files |
| `neuprojekt-entsteht-beim-bauen` | entsteht beim Bauen | created when building |
| `neuprojekt-ziel-p1-norm-p3-p4` | Ziel: {$p1 }{$norm } · {$p3 }{$p4 } | Target: {$p1 }{$norm } · {$p3 }{$p4 } |
| `neuprojekt-abbrechen` | Abbrechen | Cancel |
| `neuprojekt-projekt-erstellen-und-oeffnen` | Projekt erstellen und öffnen | Create and open project |
| `neuprojekt-neues-projekt-erstellen` | Neues Projekt erstellen | Create new project |
| `neuprojekt-zielsystem-vorlage-und-details-waehlen` | Zielsystem, Vorlage und Details wählen | Choose target system, template and details |
| `neuprojekt-fehlt` | fehlt | missing |
| `neuprojekt-keine` | keine | none |

## Projekt-Einstellungen

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `projeinst-projekt-einstellungen-erst-ein-projekt-oeffnen` | Projekt-Einstellungen: erst ein Projekt öffnen. | Project settings: open a project first. |
| `projeinst-projektbild-waehlen` | Projektbild wählen | Choose project image |
| `projeinst-projektbild-ersetzt` | Projektbild ersetzt. | Project image replaced. |
| `projeinst-fehlgeschlagen-e` | Fehlgeschlagen: {$e } | Failed: {$e } |
| `projeinst-projektbild-geloescht` | Projektbild gelöscht. | Project image deleted. |
| `projeinst-nicht-gespeichert-p1` | Nicht gespeichert: {$p1 } | Not saved: {$p1 } |
| `projeinst-gespeichert` | Gespeichert. | Saved. |
| `projeinst-projektdatei-gespeichert` | Projektdatei gespeichert | Project file saved |
| `projeinst-speichern-fehlgeschlagen-e` | Speichern fehlgeschlagen: {$e } | Saving failed: {$e } |
| `projeinst-kein-projekt-geoeffnet` | Kein Projekt geöffnet. | No project open. |
| `projeinst-projekt-einstellungen` | Projekt-Einstellungen | Project settings |
| `projeinst-ungespeichert` | ● ungespeichert | ● unsaved |
| `projeinst-zuruecksetzen` | Zurücksetzen | Reset |
| `projeinst-speichern` | Speichern | Save |
| `projeinst-speichern-projekt-bauen` | Speichern & Projekt bauen | Save & build project |
| `projeinst-achtung-x` | Achtung: {$x } | Warning: {$x } |
| `projeinst-hinweis-x` | Hinweis: {$x } | Note: {$x } |
| `projeinst-allgemein` | Allgemein | General |
| `projeinst-name` | Name | Name |
| `projeinst-projektname` | Projektname | Project name |
| `projeinst-beschreibung` | Beschreibung | Description |
| `projeinst-beim-oeffnen-die-zuletzt-geoeffneten-dateien` | Beim Öffnen die zuletzt geöffneten Dateien wiederherstellen | Restore the most recently opened files when opening |
| `projeinst-projektbild-verwenden-projektbild-png` | Projektbild verwenden (projektbild.png) | Use a project image (projektbild.png) |
| `projeinst-projektbild-png-vorhanden` | projektbild.png vorhanden | projektbild.png present |
| `projeinst-noch-kein-projektbild` | noch kein Projektbild | no project image yet |
| `projeinst-bild-waehlen` | Bild wählen … | Choose image ... |
| `projeinst-loeschen` | Löschen | Delete |
| `projeinst-cpu-p1` | CPU {$p1 } | CPU {$p1 } |
| `projeinst-entfernen` | Entfernen | Remove |
| `projeinst-videonorm` | Videonorm | Video standard |
| `projeinst-standard` | Standard | Default |
| `projeinst-speicherausbau` | Speicherausbau | Memory expansion |
| `projeinst-wie-in-den-einstellungen` | wie in den Einstellungen | as in the settings |
| `projeinst-standard-emulator` | Standard-Emulator | Default emulator |
| `projeinst-emulator` | Emulator | Emulator |
| `projeinst-p1-hardware` | {$p1 } (Hardware) | {$p1 } (hardware) |
| `projeinst-startziel-f5` | Startziel (F5) | Run target (F5) |
| `projeinst-undokumentierte-befehle` | Undokumentierte Befehle | Undocumented instructions |
| `projeinst-targets-p1` | Targets ({$p1 }) | Targets ({$p1 }) |
| `projeinst-target-hinzufuegen` | Target hinzufügen: | Add target: |
| `projeinst-art` | Art | Type |
| `projeinst-assembler` | Assembler | Assembler |
| `projeinst-target` | Target | Target |
| `projeinst-gruppe-auf` | Gruppe auf | Group up |
| `projeinst-gruppe-ab` | Gruppe ab | Group down |
| `projeinst-gruppe-loeschen` | Gruppe löschen | Delete group |
| `projeinst-basic-dialekt` | BASIC-Dialekt | BASIC dialect |
| `projeinst-beim-bauen-nach-assembler-konvertieren` | Beim Bauen nach Assembler konvertieren | Convert to assembler when building |
| `projeinst-laufzeit-fuer` | Laufzeit für | Runtime for |
| `projeinst-ausgabe` | Ausgabe | Output |
| `projeinst-standard-name-asm-asm` | Standard: <name>_asm.asm | Default: <name>_asm.asm |
| `projeinst-auf` | Auf | Up |
| `projeinst-ab` | Ab | Down |
| `projeinst-entf` | Entf. | Del |
| `projeinst-noch-keine-dateien-unten-eine-hinzufuegen` | Noch keine Dateien: unten eine hinzufügen. | No files yet: add one below. |
| `projeinst-datei-hinzufuegen` | Datei hinzufügen: | Add file: |
| `projeinst-quellen-und-startprogramm-p1-programme` | Quellen und Startprogramm ({$p1 } Programme) | Sources and start program ({$p1 } programs) |
| `projeinst-projekt-bauen-baut-die-gruppen-von` | „Projekt bauen“ baut die Gruppen von oben nach unten, innerhalb einer Gruppe die Dateien von oben nach unten; in dieser Reihenfolge erscheinen auch die Ergebnisse. Ist kein ★ gesetzt, gilt das erste Programm als Startprogramm. ★ = Startprogramm (F5 / F6). | "Build project" builds the groups from top to bottom and the files within a group from top to bottom; the results appear in the same order. If no ★ is set, the first program is the start program. ★ = start program (F5 / F6). |
| `projeinst-quellgruppe-hinzufuegen` | Quellgruppe hinzufügen | Add source group |
| `projeinst-beim-projektbau-ein-diskettenabbild-erzeugen` | Beim Projektbau ein Diskettenabbild erzeugen | Create a disk image when building the project |
| `projeinst-format` | Format | Format |
| `projeinst-d64-c64-1541` | D64 (C64/1541) | D64 (C64/1541) |
| `projeinst-d81-1581-mega65` | D81 (1581, MEGA65) | D81 (1581, MEGA65) |
| `projeinst-atr-atari-dos-2` | ATR (Atari, DOS 2) | ATR (Atari, DOS 2) |
| `projeinst-diskettenname` | Diskettenname | Disk name |
| `projeinst-ausgabedatei` | Ausgabedatei | Output file |
| `projeinst-programme-auf-der-diskette-nichts-angehakt` | Programme auf der Diskette (nichts angehakt = alle): | Programs on the disk (nothing checked = all): |
| `projeinst-diskette-datentraeger` | Diskette / Datenträger | Disk / data carrier |
| `projeinst-targets-p1-2` | Targets {$p1 } | Targets {$p1 } |
| `projeinst-quellen-p1` | Quellen {$p1 } | Sources {$p1 } |
| `projeinst-diskette-p1` | Diskette {$p1 } | Disk {$p1 } |
| `projeinst-aendern` | Ändern › | Change › |
| `projeinst-uebersicht` | Übersicht | Overview |
| `projeinst-zustand-des-projekts` | Zustand des Projekts | State of the project |
| `projeinst-name-fehlt` | Name fehlt | Name missing |
| `projeinst-targets` | Targets | Targets |
| `projeinst-quellen` | Quellen | Sources |
| `projeinst-p1-programm-e-in-p2-gruppe` | {$p1 } Programm(e) in {$p2 } Gruppe(n){$p3 } | {$p1 -> ⏎ [one] 1 program ⏎ *[other] {$p1 } programs ⏎ } in {$p2 -> ⏎ [one] 1 group ⏎ *[other] {$p2 } groups ⏎ }{$p3 } |
| `projeinst-diskette` | Diskette | Disk |
| `projeinst-alle-programme` | alle Programme | all programs |
| `projeinst-p1-programm-e` | {$p1 } Programm(e) | {$p1 -> ⏎ [one] 1 program ⏎ *[other] {$p1 } programs ⏎ } |
| `projeinst-start-m` | · Start: {$m } | · Start: {$m } |
| `projeinst-standard-des-targets` | Standard des Targets | Default of the target |
| `projeinst-target-der-gruppe` | Target der Gruppe | Target of the group |
| `projeinst-ganzzahl-division` | Ganzzahl-Division | Integer division |
| `projeinst-fliesskomma-erzwingen` | Fließkomma erzwingen | Force floating point |
| `projeinst-zeilenverfolgung` | Zeilenverfolgung | Line tracking |
| `projeinst-rem-weglassen` | REM weglassen | Leave out REM |
| `projeinst-stil-inline` | Stil inline | Inline style |
| `projeinst-laufzeit-einbetten` | Laufzeit einbetten | Embed runtime |

## Info-Fenster

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `infodlg-mailprogramm-geoeffnet-gesendet-wird-erst-dort` | Mailprogramm geöffnet. Gesendet wird erst dort. | Mail program opened. The mail is sent only there. |
| `infodlg-mailprogramm-nicht-gestartet-e-nimm-in` | Mailprogramm nicht gestartet ({$e }). Nimm „In Zwischenablage kopieren“. | Mail program not started ({$e }). Use "Copy to clipboard". |
| `infodlg-in-die-zwischenablage-kopiert` | In die Zwischenablage kopiert. | Copied to the clipboard. |
| `infodlg-version-p1-verfuegbar` | Version {$p1 } verfügbar | Version {$p1 } available |
| `infodlg-fehler-e` | Fehler: {$e } | Error: {$e } |
| `infodlg-eigene-zeichensaetze-vc-standard` | Eigene Zeichensätze (VC-Standard) | Own character sets (VC Standard) |
| `infodlg-entwuerfe-des-vintagecoder-projekts-keine-nachbildung` | Entwürfe des VintageCoder-Projekts, keine Nachbildung von Original-ROMs. Lizenz CC0-1.0: frei nutzbar und weitergebbar. Jede Datei trägt ihre Herkunft im Block „origin“. | Designs of the VintageCoder project, not a reproduction of original ROMs. License CC0-1.0: free to use and pass on. Each file states its origin in the block "origin". |
| `infodlg-keine-eigenen-zeichensaetze-gefunden-data-charsets` | Keine eigenen Zeichensätze gefunden (data/charsets/). | No own character sets found (data/charsets/). |
| `infodlg-p1-rev-p2` | {$p1 } · Rev. {$p2 } | {$p1 } · rev. {$p2 } |
| `infodlg-pruefung-pruef-freigabe-frei-p3` | Prüfung: {$pruef } · Freigabe: {$frei }{$p3 } | Check: {$pruef } · Approval: {$frei }{$p3 } |
| `infodlg-feedback-senden` | Feedback senden | Send feedback |
| `infodlg-die-mail-geht-an-p1-sie` | Die Mail geht an {$p1 }. Sie wird nur vorbereitet: gesendet wird erst, wenn du in deinem Mailprogramm auf „Senden“ klickst; hier wird nichts ins Netz geschickt. | The mail goes to {$p1 }. It is only prepared: it is sent only when you click "Send" in your mail program; nothing is sent over the network from here. |
| `infodlg-beschreibe-den-fehler-in-deinem-mailprogramm` | Beschreibe den Fehler in deinem Mailprogramm in der leeren Zeile über den Angaben; Screenshots und Dateien hängst du dort selbst an. | Describe the problem in your mail program in the empty line above the details; you attach screenshots and files there yourself. |
| `infodlg-betreff` | Betreff | Subject |
| `infodlg-text` | Text | Text |
| `infodlg-gekuerzt-wegen-der-laengengrenze-von-mail` | Gekürzt: wegen der Längengrenze von Mail-Links fehlen {$p1 }. „In Zwischenablage kopieren“ enthält alles. | Shortened: because of the length limit of mail links, {$p1 } is missing. "Copy to clipboard" contains everything. |
| `infodlg-der-link-ist-trotz-kuerzung-sehr` | Der Link ist trotz Kürzung sehr lang; manche Mailprogramme schneiden ihn ab. Nimm dann die Zwischenablage. | Even after shortening, the link is very long; some mail programs cut it off. Use the clipboard then. |
| `infodlg-pfade-in-deinem-persoenlichen-ordner-stehen` | Pfade in deinem persönlichen Ordner stehen als ~; Projekt- und Dateinamen kommen nicht vor. | Paths in your personal folder appear as ~; project and file names are not included. |
| `infodlg-fehler-schwer-zu-beschreiben-schalte-das` | Fehler schwer zu beschreiben? Schalte das Debug-Protokoll ein, wiederhole den Fehler und hänge die Datei an: | Problem hard to describe? Turn on the debug log, repeat the problem and attach the file: |
| `infodlg-debug-protokoll-schreiben` | Debug-Protokoll schreiben | Write debug log |
| `infodlg-im-mailprogramm-oeffnen` | Im Mailprogramm öffnen | Open in mail program |
| `infodlg-in-zwischenablage-kopieren` | In Zwischenablage kopieren | Copy to clipboard |
| `infodlg-abbrechen` | Abbrechen | Cancel |
| `infodlg-neue-version-p1` | Neue Version {$p1 } | New version {$p1 } |
| `infodlg-installiert-ist-p1-vintagecoder-laedt-und` | Installiert ist {$p1 }. VintageCoder lädt und installiert nichts selbst: „Ansehen“ öffnet die Releaseseite im Browser. | Installed is {$p1 }. VintageCoder does not download or install anything itself: "View" opens the release page in the browser. |
| `infodlg-achtung-die-neue-version-schreibt-ein` | Achtung: Die neue Version schreibt ein neueres Projektformat; ältere Versionen können solche Projekte danach nicht mehr lesen. | Warning: the new version writes a newer project format; older versions can no longer read such projects afterwards. |
| `infodlg-ansehen` | Ansehen … | View ... |
| `infodlg-spaeter-erinnern` | Später erinnern | Remind me later |
| `infodlg-diese-version-ueberspringen` | Diese Version überspringen | Skip this version |
| `infodlg-schliessen` | Schließen | Close |
| `infodlg-updates` | Updates | Updates |
| `infodlg-installiert-inst-p2` | Installiert: {$inst }{$p2 } | Installed: {$inst }{$p2 } |
| `infodlg-zuletzt-geprueft-p1-p2` | Zuletzt geprüft: {$p1 }{$p2 } | Last checked: {$p1 }{$p2 } |
| `infodlg-pruefe` | Prüfe … | Checking ... |
| `infodlg-jetzt-pruefen` | Jetzt prüfen | Check now |
| `infodlg-aenderungen-ansehen` | Änderungen ansehen | View changes |
| `infodlg-jetzt-pruefen-fragt-die-versionsliste-update` | „Jetzt prüfen“ fragt die Versionsliste (update.json) bei github.com ab: GitHub sieht deine IP-Adresse und den Programmnamen „VintageCoder-update“, ohne Versionsnummer. Ohne deine Zustimmung geht nie etwas ins Netz. | "Check now" asks github.com for the version list (update.json): GitHub sees your IP address and the program name "VintageCoder-update", without a version number. Nothing ever goes to the network without your consent. |
| `infodlg-vintagecoder-p1` | VintageCoder {$p1 } | VintageCoder {$p1 } |
| `infodlg-entwicklungsumgebung-fuer-retro-computer` | Entwicklungsumgebung für Retro-Computer | Development environment for retro computers |
| `infodlg-assembler-basic-grafik-und-klang-editoren` | Assembler, BASIC, Grafik- und Klang-Editoren, Debugger und Emulator-Anbindung für C64, Atari, MEGA65 und weitere Systeme. | Assembler, BASIC, graphics and sound editors, debugger and emulator connection for C64, Atari, MEGA65 and more systems. |
| `infodlg-enthaelt-wahrscheinlich-noch-viele-fehler` | Enthält wahrscheinlich noch viele Fehler. Keine Gewährleistung, keine Haftung für Datenverlust (siehe Lizenz). | Probably still contains many errors. No warranty, no liability for data loss (see license). |
| `infodlg-lizenz` | Lizenz | License |
| `infodlg-autor` | Autor | Author |
| `infodlg-christian-drueeke` | Christian Drüeke | Christian Drüeke |
| `infodlg-dank` | Dank | Thanks |
| `infodlg-an-die-community-n3rdroom-discord` | an die Community „N3rdroom“ (Discord) | to the community "N3rdroom" (Discord) |
| `infodlg-webseite` | Webseite | Website |
| `infodlg-quelltext` | Quelltext | Source code |
| `infodlg-feedback-senden-2` | Feedback senden … | Send feedback ... |
| `infodlg-herkunft-und-lizenzen-ausblenden` | Herkunft und Lizenzen ausblenden | Hide origin and licenses |
| `infodlg-herkunft-und-lizenzen` | Herkunft und Lizenzen | Origin and licenses |
| `infodlg-drittanbieter-hinweise-ausblenden` | Drittanbieter-Hinweise ausblenden | Hide third-party notices |
| `infodlg-drittanbieter-hinweise` | Drittanbieter-Hinweise | Third-party notices |
| `infodlg-abstandspruefung-nicht-bestanden-vom-inhaber-akzeptiert` | (Abstandsprüfung nicht bestanden, vom Inhaber akzeptiert) | (spacing check failed, accepted by the owner) |
| `infodlg-neuere-version-p1` | ·   Neuere Version: {$p1 } | ·   Newer version: {$p1 } |
| `infodlg-aktuell` | ·   aktuell | ·   up to date |
| `infodlg-noch-nie` | noch nie | never |

## Export

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `exportdlg-prg-braucht-eine-gueltige-ladeadresse-z` | PRG braucht eine gültige Ladeadresse (z. B. $2000) | PRG needs a valid load address (e.g. $2000) |
| `exportdlg-export-rel-e` | Export {$rel }: {$e } | Export {$rel }: {$e } |
| `exportdlg-exportiert-p1` | Exportiert: {$p1 } | Exported: {$p1 } |
| `exportdlg-gespeichert-im-projekt-p1-im-baum` | Gespeichert im Projekt: {$p1 } – im Baum unter „{$p2 }“ | Saved in the project: {$p1 } – in the tree under "{$p2 }" |
| `exportdlg-der-export-hat-keine-datei-erzeugt` | Der Export hat keine Datei erzeugt – nichts eingefügt | The export produced no file – nothing inserted |
| `exportdlg-kein-quelltext-zum-einfuegen-offen-oeffne` | Kein Quelltext zum Einfügen offen – öffne zuerst eine .asm-Datei und setze den Cursor an die Einfügestelle | No source file open to insert into – open an .asm file first and put the cursor where the data should go |
| `exportdlg-doc-e` | {$doc }: {$e } | {$doc }: {$e } |
| `exportdlg-in-doc-eingefuegt-p2` | In {$doc } eingefügt: {$p2 } | Inserted into {$doc }: {$p2 } |
| `exportdlg-g-byte` | {$g } Byte | {$g } bytes |
| `exportdlg-hex` | Hex | Hex |
| `exportdlg-keine-binaerdaten` | (keine Binärdaten) | (no binary data) |
| `exportdlg-adresse-ungueltig` | ● Adresse ungültig | ● Address invalid |
| `exportdlg-bereit-gesamt-byte` | ● bereit · {$gesamt } Byte | ● ready · {$gesamt } bytes |
| `exportdlg-kachel-layout` | Kachel-Layout | Tile layout |
| `exportdlg-kachel-tabellen` | Kachel-Tabellen | Tile tables |
| `exportdlg-karte` | Karte | Map |
| `exportdlg-kachelindex` | Kachelindex | Tile index |
| `exportdlg-8-bit` | 8 Bit | 8 bit |
| `exportdlg-16-bit` | 16 Bit | 16 bit |
| `exportdlg-speicher-p1-byte-kacheln-p2-farben` | Speicher {$p1 } Byte (Kacheln {$p2 }, Farben {$p3 }, Eigenschaften {$p4 }, Zeichensatz {$p5 }, Karten {$p6 }, Zellen {$p7 }) · mit {$p8 }-Bit-Index {$p9 } Byte | Memory {$p1 } bytes (tiles {$p2 }, colors {$p3 }, properties {$p4 }, character set {$p5 }, maps {$p6 }, cells {$p7 }) · with {$p8 }-bit index {$p9 } bytes |
| `exportdlg-hinweis-die-kachel-bibliothek-tiles-use` | Hinweis: die Kachel-Bibliothek (tiles_use) kennt nur planar, zeilenweise, 8 Bit; andere Layouts brauchen eigene Routinen. | Note: the tile library (tiles_use) only knows planar, row by row, 8 bit; other layouts need their own routines. |
| `exportdlg-basic-programm-play` | BASIC-Programm (PLAY) | BASIC program (PLAY) |

## Import

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `importdlg-nichts-zu-uebernehmen-alle-dateien-liegen` | Nichts zu übernehmen: alle Dateien liegen schon im Projekt | Nothing to add: all files are already in the project |
| `importdlg-uebernommen-r` | Übernommen: {$r } | Added: {$r } |
| `importdlg-uebersprungen-r` | Übersprungen: {$r } | Skipped: {$r } |
| `importdlg-ok-datei-en-uebernommen-p2` | {$ok } Datei(en) übernommen{$p2 } | {$ok -> ⏎ [one] 1 file added ⏎ *[other] {$ok } files added ⏎ }{$p2 } |
| `importdlg-name-belegt-anderer-inhalt` | Name belegt (anderer Inhalt) | Name taken (different content) |
| `importdlg-schon-vorhanden-gleicher-inhalt-wird-nicht` | schon vorhanden (gleicher Inhalt) – wird nicht übernommen | already there (same content) – will not be added |
| `importdlg-liegt-schon-dort-wird-nicht-uebernommen` | liegt schon dort – wird nicht übernommen | is already there – will not be added |
| `importdlg-name` | Name | Name |
| `importdlg-ordner-im-projekt` | Ordner im Projekt | Folder in the project |
| `importdlg-pfad` | → {$pfad } | → {$pfad } |
| `importdlg-umbenennen` | Umbenennen | Rename |
| `importdlg-ersetzen` | Ersetzen | Replace |
| `importdlg-ueberspringen` | Überspringen | Skip |
| `importdlg-als-kopie-unter-neuem-namen` | Als Kopie unter neuem Namen | As a copy under a new name |
| `importdlg-und-mehr-weitere-datei-en-die` | … und {$mehr } weitere Datei(en), die mit denselben Vorschlägen übernommen werden | ... and {$mehr } more file(s), which are added with the same suggestions |
| `importdlg-n-uebernehmen` | {$n } übernehmen | Add {$n } |
| `importdlg-fehler-fehler` | , {$fehler } Fehler | , {$fehler } error(s) |
| `importdlg-neu` | neu | new |

## Projekt kopieren

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `kopierdlg-projekt` | Projekt | Project |
| `kopierdlg-basis-kopie` | {$basis } Kopie | {$basis } copy |
| `kopierdlg-erst-i` | {$erst } {$i } | {$erst } {$i } |
| `kopierdlg-b-byte` | {$b } Byte | {$b } bytes |
| `kopierdlg-p1-kb` | {$p1 } KB | {$p1 } KB |
| `kopierdlg-p1-mb` | {$p1 } MB | {$p1 } MB |
| `kopierdlg-p1-gibt-es-schon` | „{$p1 }“ gibt es schon. | "{$p1 }" already exists. |
| `kopierdlg-das-ziel-laege-im-quellordner` | Das Ziel läge im Quellordner. | The target would be inside the source folder. |
| `kopierdlg-projekt-kopieren-erst-ein-projekt-oeffnen` | Projekt kopieren: erst ein Projekt öffnen. | Copy project: open a project first. |
| `kopierdlg-projekt-kopiert-p1-p2-dateien-p3` | Projekt kopiert: {$p1 } ({$p2 } Dateien, {$p3 }) | Project copied: {$p1 } ({$p2 } files, {$p3 }) |
| `kopierdlg-uebersprungen-s` | übersprungen: {$s } | skipped: {$s } |
| `kopierdlg-projekt-kopieren` | Projekt kopieren | Copy project |
| `kopierdlg-quelle-p1` | Quelle: {$p1 } | Source: {$p1 } |
| `kopierdlg-name-der-kopie` | Name der Kopie | Name of the copy |
| `kopierdlg-name` | Name | Name |
| `kopierdlg-speicherort` | Speicherort | Location |
| `kopierdlg-waehlen` | Wählen … | Choose ... |
| `kopierdlg-bauordner-build-mitkopieren` | Bauordner (build/) mitkopieren | Copy the build folder (build/) too |
| `kopierdlg-versionsgeschichte-git-mitkopieren` | Versionsgeschichte (.git/) mitkopieren | Copy the version history (.git/) too |
| `kopierdlg-danach-die-kopie-oeffnen` | Danach die Kopie öffnen | Open the copy afterwards |
| `kopierdlg-n-dateien-p2-werden-kopiert` | {$n } Dateien, {$p2 } werden kopiert. | {$n } files, {$p2 } will be copied. |
| `kopierdlg-kopiert` | kopiert … | copied ... |
| `kopierdlg-abbrechen` | Abbrechen | Cancel |
| `kopierdlg-kopieren` | Kopieren | Copy |

## Studio-Aktionen (Befehlspalette)

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `studioaktionen-stift` | Stift | Pen |
| `studioaktionen-radierer` | Radierer | Eraser |
| `studioaktionen-fuellen` | Füllen | Fill |
| `studioaktionen-linie` | Linie | Line |
| `studioaktionen-rahmen` | Rahmen | Frame |
| `studioaktionen-auswahl` | Auswahl | Selection |
| `studioaktionen-pipette` | Pipette | Eyedropper |
| `studioaktionen-werkzeug-n` | Werkzeug: {$n } | Tool: {$n } |
| `studioaktionen-horizontal-spiegeln` | Horizontal spiegeln | Flip horizontally |
| `studioaktionen-vertikal-spiegeln` | Vertikal spiegeln | Flip vertically |
| `studioaktionen-frame-leeren` | Frame leeren | Clear frame |
| `studioaktionen-modus-hires` | Modus: Hires | Mode: Hires |
| `studioaktionen-modus-multicolor` | Modus: Multicolor | Mode: Multicolor |
| `studioaktionen-einpassen` | Einpassen | Fit |
| `studioaktionen-vergroessern` | Vergrößern | Zoom in |
| `studioaktionen-verkleinern` | Verkleinern | Zoom out |
| `studioaktionen-zeichen` | Zeichen | Character |
| `studioaktionen-text` | Text | Text |
| `studioaktionen-loeschen` | Löschen | Delete |
| `studioaktionen-in-kacheln-zerlegen` | In Kacheln zerlegen … | Split into tiles ... |
| `studioaktionen-zellen-waagerecht-spiegeln` | Zellen waagerecht spiegeln | Flip cells horizontally |
| `studioaktionen-zellen-senkrecht-spiegeln` | Zellen senkrecht spiegeln | Flip cells vertically |
| `studioaktionen-um-90-drehen` | Um 90° drehen | Rotate by 90° |
| `studioaktionen-alle-zellen-leeren` | Alle Zellen leeren | Clear all cells |
| `studioaktionen-zellen-der-kachel-kopieren` | Zellen der Kachel kopieren | Copy the cells of the tile |
| `studioaktionen-zellen-einfuegen` | Zellen einfügen | Paste cells |
| `studioaktionen-neue-kachel` | Neue Kachel | New tile |
| `studioaktionen-kachel-duplizieren` | Kachel duplizieren | Duplicate tile |
| `studioaktionen-kachel-loeschen` | Kachel löschen | Delete tile |
| `studioaktionen-modus-kacheln` | Modus: Kacheln | Mode: Tiles |
| `studioaktionen-tab` | Tab | Tab |
| `studioaktionen-modus-zeichen` | Modus: Zeichen | Mode: Characters |
| `studioaktionen-neuer-screen` | Neuer Screen | New screen |
| `studioaktionen-screen-duplizieren` | Screen duplizieren | Duplicate screen |
| `studioaktionen-screen-loeschen` | Screen löschen | Delete screen |
| `studioaktionen-auswahl-kopieren` | Auswahl kopieren | Copy selection |
| `studioaktionen-auswahl-leeren` | Auswahl leeren | Clear selection |
| `studioaktionen-entf` | Entf | Del |
| `studioaktionen-standardpalette-laden` | Standardpalette laden | Load standard palette |
| `studioaktionen-rueckgaengig` | Rückgängig | Undo |
| `studioaktionen-wiederholen` | Wiederholen | Redo |
| `studioaktionen-validieren` | Validieren | Validate |
| `studioaktionen-einlegen` | Einlegen | Insert |
| `studioaktionen-datei-hinzufuegen` | Datei hinzufügen … | Add file ... |
| `studioaktionen-dem-folgesektor-folgen` | Dem Folgesektor folgen | Follow the next sector |
| `studioaktionen-zurueck-zum-vorigen-sektor` | Zurück zum vorigen Sektor | Back to the previous sector |
| `studioaktionen-kette-ausblenden` | Kette ausblenden | Hide chain |
| `studioaktionen-kette-anzeigen` | Kette anzeigen | Show chain |
| `studioaktionen-stopp` | Stopp | Stop |
| `studioaktionen-song-abspielen` | Song abspielen | Play song |
| `studioaktionen-leertaste` | Leertaste | Space |
| `studioaktionen-muster-abspielen` | Muster abspielen | Play pattern |
| `studioaktionen-oktave-tiefer` | Oktave tiefer | Octave down |
| `studioaktionen-oktave-hoeher` | Oktave höher | Octave up |
| `studioaktionen-neues-muster` | Neues Muster | New pattern |
| `studioaktionen-muster-duplizieren` | Muster duplizieren | Duplicate pattern |
| `studioaktionen-muster-loeschen` | Muster löschen | Delete pattern |
| `studioaktionen-neues-instrument` | Neues Instrument | New instrument |
| `studioaktionen-alle-kanaele-einblenden` | Alle Kanäle einblenden | Show all channels |
| `studioaktionen-frame-anhaengen` | Frame anhängen | Append frame |
| `studioaktionen-frame-duplizieren` | Frame duplizieren | Duplicate frame |
| `studioaktionen-frame-loeschen` | Frame löschen | Delete frame |

## Studio-Namen

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `studios-sprites-pmg` | Sprites / PMG | Sprites / PMG |
| `studios-zeichensaetze` | Zeichensätze | Character sets |
| `studios-bildschirme` | Bildschirme | Screens |
| `studios-kacheln` | Kacheln | Tiles |
| `studios-paletten` | Paletten | Palettes |
| `studios-tracker` | Tracker | Tracker |
| `studios-play-tracker` | PLAY-Tracker | PLAY tracker |
| `studios-disk-studio` | Disk-Editor | Disk Editor |
| `studios-memory-map` | Memory Map | Memory Map |
| `studios-extraktor` | Extraktor | Extractor |

## Update-Prüfung

| Schlüssel | Deutsch | Englisch |
|---|---|---|
| `update-kein-gueltiges-json-e` | kein gültiges JSON ({$e }) | not valid JSON ({$e }) |
| `update-kein-objekt` | kein Objekt | not an object |
| `update-format-fehlt` | format fehlt | format missing |
| `update-format-0` | format 0 | format 0 |
| `update-channel-ist-nicht-stable` | channel ist nicht „stable“ | channel is not “stable” |
| `update-latest-fehlt-oder-ist-keine-version` | latest fehlt oder ist keine Version MAJOR.MINOR.PATCH | latest is missing or is not a version MAJOR.MINOR.PATCH |
| `update-yanked-ist-keine-liste` | yanked ist keine Liste | yanked is not a list |
| `update-yanked-ist-zu-lang` | yanked ist zu lang | yanked is too long |
| `update-yanked-ungueltige-version` | yanked: ungültige Version | yanked: invalid version |
| `update-releases-fehlt` | releases fehlt | releases missing |
| `update-releases-1-bis-max-releases-eintraege` | releases: 1 bis {$MAX_RELEASES } Einträge erwartet | releases: 1 to {$MAX_RELEASES } entries expected |
| `update-releases-eintrag-ist-kein-objekt` | releases: Eintrag ist kein Objekt | releases: entry is not an object |
| `update-releases-ungueltige-version` | releases: ungültige Version | releases: invalid version |
| `update-text-zu-version-ist-zu-lang` | Text zu {$version } ist zu lang | Text for {$version } is too long |
| `update-texte-insgesamt-zu-lang` | Texte insgesamt zu lang | Texts are too long in total |
| `update-version-version-kommt-doppelt-vor` | Version {$version } kommt doppelt vor | Version {$version } occurs twice |
| `update-latest-steht-nicht-in-releases` | latest steht nicht in releases | latest is not in releases |
| `update-notes-kopf` | Version {$p1 }{$p2 } | Version {$p1 }{$p2 } |
| `update-shown` | ({$shown }) | ({$shown }) |
| `update-kein-ordner-fuer-die-update-pruefung` | Kein Ordner für die Update-Prüfung | No folder for the update check |
| `update-y-m-d-p4-p5` | {$y }-{$m }-{$d } {$p4 }:{$p5 } | {$y }-{$m }-{$d } {$p4 }:{$p5 } |
| `update-keine-verbindung-kein-netz-dns-oder` | keine Verbindung (kein Netz, DNS oder Proxy) | no connection (no network, DNS or proxy) |
| `update-zeitueberschreitung` | Zeitüberschreitung | Timeout |
| `update-keine-versionsliste-unter-dieser-adresse-404` | keine Versionsliste unter dieser Adresse (404) | no version list at this address (404) |
| `update-github-lehnt-die-anfrage-voruebergehend-ab` | GitHub lehnt die Anfrage vorübergehend ab ({$p1 }) | GitHub temporarily rejects the request ({$p1 }) |
| `update-der-server-antwortet-mit-fehler-c` | Der Server antwortet mit Fehler {$c } | The server answers with error {$c } |
| `update-sichere-verbindung-nicht-moeglich-zertifikat-oder` | sichere Verbindung nicht möglich (Zertifikat oder TLS) | secure connection not possible (certificate or TLS) |
| `update-das-programm-curl-fehlt` | das Programm curl fehlt | the curl program is missing |
| `update-nur-https-ist-erlaubt` | nur https:// ist erlaubt | only https:// is allowed |
| `update-die-versionsliste-ist-zu-gross` | die Versionsliste ist zu groß | the version list is too large |
| `update-keine-gueltige-utf-8-datei` | keine gültige UTF-8-Datei | not a valid UTF-8 file |
| `update-curl-nicht-ausfuehrbar-e` | curl nicht ausführbar: {$e } | curl cannot be run: {$e } |
| `update-http-fehler` | HTTP-Fehler | HTTP error |
| `update-abruf-fehlgeschlagen` | Abruf fehlgeschlagen | Fetching failed |
| `update-version-p1-verfuegbar` | Version {$p1 } verfügbar | Version {$p1 } available |
| `update-fehler-e` | Fehler: {$e } | Error: {$e } |
| `update-p1-utc-source-p3` | {$p1 } UTC  {$source }  {$p3 } | {$p1 } UTC  {$source }  {$p3 } |
| `update-diese-installation-stammt-aus-einem-linux` | Diese Installation stammt aus einem Linux-Paket. Neue Versionen installierst du mit deiner Paketverwaltung (beide Pakete, vintagecoder und vintagecoder-cli, in gleicher Version); die Pakete stehen auf der Releaseseite. | This installation comes from a Linux package. Install new versions with your package manager (both packages, vintagecoder and vintagecoder-cli, in the same version); the packages are on the releases page. |
| `update-diese-installation-wurde-aus-dem-quelltext` | Diese Installation wurde aus dem Quelltext gebaut. Hole den neuen Stand mit git und baue ihn neu. | This installation was built from the source code. Get the new state with git and build it again. |
| `update-die-versionsliste-hat-das-format-n` | Die Versionsliste hat das Format {$n }; diese Version von VintageCoder kennt höchstens {$max } | The version list has format {$n }; this version of VintageCoder knows {$max } at most |
| `update-die-versionsliste-ist-ungueltig-m` | Die Versionsliste ist ungültig: {$m } | The version list is invalid: {$m } |
| `update-aktuell` | aktuell | up to date |

