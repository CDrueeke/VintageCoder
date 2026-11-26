# SPDX-License-Identifier: GPL-3.0-or-later
# User interface: window, menus, status bar, log. Key scheme <area>-<place>-<purpose>; keys are never renamed.

# Title of the window and of the program
window-title = VintageCoder

# Top menus of the menu bar (identifiers in the code stay German and stable)
# Application menu of the macOS menu bar
menu-app-about = About VintageCoder
menu-app-hide = Hide VintageCoder

menu-file = File
menu-edit = Edit
menu-view = View
menu-project = Project
menu-tree = Tree
menu-help = Help

# Foot of the file tree. Context: n = number of files, dirs = number of folders
status-tree-summary = { $n ->
    [one] 1 file
   *[other] { $n } files
 } · { $dirs ->
    [one] 1 folder
   *[other] { $dirs } folders
 }

# Addition to the foot of the file tree while a filter is active
status-tree-filtered = (filtered)

# Log line after a failed build. Context: n = number of problems
build-failed = Build failed: { $n ->
    [one] 1 problem
   *[other] { $n } problems
 }

# ---- Main window (extracted from main.rs) ----

gui-a-p1-x-p2-y-p3 = A { $p1 }  X { $p2 }  Y { $p3 }  SP { $p4 }  PC { $p5 }
gui-aa = Aa ✓
gui-aa-2 = Aa
gui-abbrechen = Cancel
gui-abbrechen-2 = ■ Cancel
gui-aeltere-kachelkarte-umwandeln = Convert an older tile map?
gui-alle = All
gui-alle-ausfuehren = ▶ Run all
gui-alle-einstellungen-der-ide-zielsysteme-emulatoren = All settings of the IDE (target systems, emulators, machines including passwords, paths, parameters) go back to the defaults. This cannot be undone; you can export the configuration first.
gui-alle-ordner-zuklappen = Collapse all folders
gui-angelegt-rel = Created: { $rel }
gui-atari = Atari
gui-bauen = Build
gui-bauen-rel = Building: { $rel } ...
gui-baum-neu-einlesen = Reload the tree
gui-baut = building ...
gui-beim-letzten-beenden-waren-unbenannte-reiter = When you last quit, untitled tabs were unsaved. The backup copy exists only on this computer, unencrypted, and expires after 14 days.
gui-bildschirm-kachel-und-tracker-dateien-oeffnen = Screen, tile and tracker files open with their editors (coming).
gui-braucht-ein-geoeffnetes-studio = needs an open editor
gui-braucht-ein-projekt = needs a project
gui-braucht-eine-debug-sitzung = needs a debug session
gui-braucht-eine-debug-sitzung-und-einen = needs a debug session and an open source file
gui-braucht-eine-markdown-datei = needs a Markdown file
gui-braucht-einen-offenen-quelltext = needs an open source file
gui-code-code2 = ${ $code } · { $code2 }
gui-das-projekt-ist-leer = The project is empty.
gui-datei = File
gui-datei-ist-schon-formatiert-oder-hier = The file is already formatted (or there is no formatting here)
gui-datei-oeffnen = Open file
gui-dateien-filtern = Filter files
gui-debug-sitzung-beendet = Debug session ended
gui-debug-sitzung-fuer-den-test-name = Debug session for the test "{ $name }" is starting ...
gui-debug-sitzung-startet = Debug session is starting ...
gui-debuggen = ● Debug
gui-debuggen-2 = Debug
gui-debuggen-e = Debug: { $e }
gui-definition-ausserhalb-des-projekts-uri = Definition outside the project: { $uri }
gui-die-datei-p1-enthaelt-z-die = The file "{ $p1 }" contains: { $z }. The current settings for target systems, emulators, machines and parameters will be replaced; known machines keep their password. The interface settings (color scheme, zoom) stay.
gui-die-tests-laufen-parallel-zur-debug = The tests run in parallel to the debug session (own emulator).
gui-diese-datei-ist-schon-eingebunden-zeile = This file is already included (line { $p1 }) – nothing inserted
gui-diese-reiter-haben-ungespeicherte-aenderungen = These tabs have unsaved changes:
gui-dieses-thema-im-dokumentenfeld-oeffnen-mehr = Open this topic in the document area (more room)
gui-disk-p1-p2-bytes-p3 = Disk { $p1 } ({ $p2 } bytes): { $p3 }
gui-diskette-atr = Disk ATR
gui-diskette-d64 = Disk D64
gui-diskette-d71 = Disk D71
gui-diskette-d81 = Disk D81
gui-diskette-e = Disk: { $e }
gui-e-wird-nicht-ueberschrieben = { $e } (will not be overwritten)
gui-editor-n = Editor { $n }
gui-editor-teilen-zweite-haelfte-rechts-cmd = Split editor: second half on the right (Cmd+Alt+T)
gui-ein-ordner-laesst-sich-nicht-in = A folder cannot be moved into itself.
gui-eine-debug-sitzung-laeuft-beenden-und = A debug session is running. End it and debug the test "{ $n }"?
gui-eine-debug-sitzung-laeuft-beenden-und-2 = A debug session is running. End it, rebuild and debug?
gui-eingebunden-p1-p2-p3-z-nimmt = Included: { $p1 }{ $p2 } ({ $p3 }+Z takes it back)
gui-einstellungen-zuruecksetzen = Reset settings?
gui-einzelheit-e = Detail: { $e }
gui-einzelheit-r = Detail: { $r }
gui-emulator = Emulator
gui-emulator-p1 = Emulator ({ $p1 })
gui-enthaelt-geoeffnete-dateien = contains open files
gui-es-gibt-nichts-rueckgaengig-zu-machen = There is nothing to undo.
gui-f-a-p3 = { $f }:{ $a }{ $p3 }
gui-fehler = Error
gui-fehler-e = Error: { $e }
gui-fehlgeschlagene = Failed
gui-fertig-p1-bytes-p2 = Done: { $p1 } bytes → { $p2 }
gui-filter = Filter
gui-fuer-dieses-projekt-gibt-es-dafuer = There is no profile for this in this project.
gui-gehe-zu-definition-nur-in-assembler = Go to definition: assembler files only.
gui-gehe-zu-zeile = Go to line
gui-gemessen-m = measured: { $m }
gui-gerade-nicht-moeglich = not possible right now
gui-gespeichert-n = Saved: { $n }
gui-gespeichert-rel = Saved: { $rel }
gui-gestartet-c = Started: { $c }
gui-hilfe = Help
gui-hinweis = Note
gui-im-reiter-oeffnen = Open in a tab
gui-kachelkarte-umgewandelt-sicherung-der-alten-datei = Tile map converted; backup of the old file: { $p1 }
gui-kein-dokument = No document
gui-kein-projekt = No project
gui-kein-quelltext-offen-hier-gibt-es = No source file open: nothing to include here
gui-keine-definition-gefunden = No definition found.
gui-keine-probleme = No problems
gui-keine-probleme-2 = No problems.
gui-keine-quelldatei-zum-bauen = No source file to build.
gui-keine-vorschlaege-an-dieser-stelle = No suggestions here
gui-klick-oder-taste-schliesst-das-startbild = Click or key closes the start screen
gui-kodierung-beim-speichern-utf-8-iso = Encoding when saving: UTF-8 ↔ ISO-8859-1 (click to switch)
gui-konfiguration-uebernehmen = Import the configuration?
gui-kuerzel-id-als-doppelereignis-verworfen = Shortcut: { $id } discarded as a duplicate event
gui-kuerzel-menue-ausloesung-id-id = Shortcut: menu trigger id={ $id }
gui-kurz = { $kurz } ▾
gui-laeuft = running ...
gui-laeuft-auf-p1 = Runs on: { $p1 } ▾
gui-lang-spruenge-automatisch-verlaengert-branchlong-auto = { $lang ->
    [one] 1 jump
   *[other] { $lang } jumps
 } extended automatically (.branchlong auto)
gui-letzte-sitzung-wiederherstellen = Restore the last session?
gui-markdown-quelltext = Markdown source
gui-markdown-vorschau = Markdown preview
gui-mehr = More
gui-n-a = { $n }  ${ $a }
gui-n-ersetzt = { $n } replaced
gui-n-hat-inhalt-und-ist-nicht = "{ $n }" has content and is not saved (Cmd+S saves it as a file). Discard?
gui-n-ist-eine-aeltere-kachelkarte-kind = "{ $n }" is an older tile map (kind "tilemap"). The tile set editor reads a tile set and a screen: converting creates both next to the file and opens the screen. The old file is first backed up under .vcbackup/ and then replaced by the new ones.
gui-n-nicht-gespeichert-die-datei-war = { $n }: not saved, the file was not readable and would be overwritten
gui-n-p2-p3-gilt-beim-speichern = { $n }: { $p2 } · { $p3 } (applies when saving)
gui-nach-neuem-gesucht-rom-verzeichnis-neu = Looked for new items: ROM folder read again, emulator search in the assistant.
gui-name-p2 = { $name }{ $p2 }
gui-name-tasten = { $name }   { $tasten }
gui-neue-datei-im-gewaehlten-ordner = New file in the chosen folder
gui-nicht-jetzt = Not now
gui-nichts-zum-debuggen-erst-bauen = Nothing to debug: build first.
gui-nichts-zum-starten-erst-bauen = Nothing to run: build first.
gui-notiz = Note
gui-nur-der-name-kein-pfad-verschieben = Name only, no path: moving is done by dragging in the tree.
gui-nur-fehler = errors only
gui-ordner = Folder
gui-ordner-angelegt-r = Folder created: { $r }/
gui-p1-64-kb = { $p1 } · 64 KB
gui-p1-bestanden-fehler-fehlgeschlagen-p3-uebersprungen = { $p1 } passed · { $fehler } failed · { $p3 } skipped{ $p4 }
gui-p1-datei-ebene-p2 = { $p1 }, file, level { $p2 }
gui-p1-debuggen-geht-nur-ueber-usb = { $p1 }: debugging only works over USB (Ethernet offers no monitor).
gui-p1-name = { $p1 }{ $name }
gui-p1-name-2 = { $p1 } { $name }
gui-p1-nicht-lesbar = { $p1 }: not readable
gui-p1-ordner-p2-ebene-p3 = { $p1 }, folder, { $p2 }, level { $p3 }
gui-p1-p2-zeichen-p3 = • { $p1 } · { $p2 } characters · { $p3 }
gui-p1-rel = { $p1 } { $rel }
gui-p1-von-n = { $p1 } of { $n }
gui-pc-p1 = · PC ${ $p1 }
gui-pfad-kopiert-p = Path copied: { $p }
gui-plus-4 = Plus/4
gui-png-bild-laden = Load PNG image
gui-programm = Program
gui-programm-p1-von-256-byte = Program: { $p1 } of 256 bytes
gui-projekt = Project
gui-projekt-2 = Project ▾
gui-projekt-einstellungen = Project settings
gui-projekt-geoeffnet-p1 = Project opened: { $p1 }
gui-projekt-n = Project: { $n }
gui-projekt-oeffnen-projektdatei-vcprj = Open project (project file .vcprj)
gui-projekt-wechseln = Switch project?
gui-projekt-wechseln-erst-die-debug-sitzung = Switch project: first end the debug session or the build.
gui-projektordner-oeffnen = Open project folder
gui-pruefung = CHECK
gui-quelle-nichts-zu-verschieben = { $quelle }: nothing to move
gui-quelltext = Source code
gui-r-000 = ${ $r }000
gui-reiter-nicht-gefunden = Tab not found
gui-reiter-schliessen = Close tab
gui-rel-gibt-es-schon = { $rel } already exists
gui-rel-keine-textdatei-oeffnet-im-standardprogramm = { $rel }: not a text file, opens in the default program
gui-rel-nicht-lesbar = ; { $rel }: not readable
gui-rel-p2 = { $rel } { $p2 }
gui-rel-sicherung-fehlgeschlagen-nichts-umgewandelt-e = { $rel }: backup failed, nothing converted: { $e }
gui-rel-umwandlung-fehlgeschlagen-e = { $rel }: conversion failed: { $e }
gui-rmt-modul-waehlen = Choose RMT module
gui-rueckgaengig-erst-alle-geaenderten-dateien-speichern = Undo: first save all changed files.
gui-seite-p-p2-p3 = PAGE ${ $p } · ${ $p2 }–${ $p3 }
gui-sname-p2 = { $sname } · { $p2 } ▾
gui-song-aus-anderem-system-waehlen = Choose a song from another system
gui-speichern = Save
gui-speichern-fehlgeschlagen-er = Saving failed: { $er }
gui-speichern-unter = Save as
gui-start = Start
gui-start-fehlgeschlagen-p1 = Start failed: { $p1 }
gui-starten = Run
gui-starten-auf-p1 = Running on { $p1 } ...
gui-starten-im-emulator = Running in the emulator ...
gui-studio = Editor
gui-studio-n = Editor: { $n }
gui-studio-p1 = editor { $p1 }
gui-suche-laeuft = Searching ...
gui-suchen = Search
gui-teilung-aufheben-cmd-alt-t = Remove split (Cmd+Alt+T)
gui-testlauf-kein-emulator-gestartet = Test run: no emulator started
gui-testlauf-keine-debug-sitzung-gestartet = Test run: no debug session started
gui-uebernehmen = Apply
gui-umbenannt-alt-neu = Renamed: { $alt } → { $neu }
gui-umwandeln = Convert
gui-unbenannt = Untitled
gui-unbenannt-wird-in-der-zwischenkopie-gesichert = (untitled, saved in the backup copy)
gui-update-v-verfuegbar = Update { $v } available
gui-verschieben-braucht-eine-projektdatei-vcprj = Moving needs a project file (.vcprj).
gui-verschieben-erst-alle-geaenderten-dateien-speichern = Move: first save all changed files (paths in source files are adjusted).
gui-verschieben-fehlgeschlagen-e = Move failed: { $e }
gui-verschieben-rueckgaengig-gemacht-n-datei-en = Move undone ({ $n ->
    [one] 1 file
   *[other] { $n } files
 })
gui-verschoben-quelle-neu-p3 = Moved: { $quelle } → { $neu }{ $p3 }
gui-version-p1 = Version { $p1 }
gui-verwerfen = Discard
gui-vintagecoder = VintageCoder
gui-vintagecoder-beenden = Quit VintageCoder?
gui-vorschau = Preview
gui-w-hh-bild-p3-aufbau-p4 = { $w }×{ $hh } · frame { $p3 } · build { $p4 } ms
gui-warnung = Warning
gui-wiederherstellen = Restore
gui-wird-angelegt-rel = Being created: { $rel }
gui-z-z-s-s-p3 = L { $z } · C { $s }   { $p3 }
gui-zeile-ist-schon-formatiert-oder-hier = The line is already formatted (or there is no formatting here)
gui-zeile-p1-p2 = Line { $p1 }: { $p2 }
gui-zeile-zeile-hat-keinen-code-dort = Line { $zeile } has no code: execution cannot stop there.
gui-zeilenende-beim-speichern-lf-crlf-klick = Line ending when saving: LF ↔ CRLF (click to switch)
gui-zeilennummer = Line number
gui-ziel-p1 = Target: { $p1 }
gui-ziel-p1-p2 = Target: { $p1 } · { $p2 }
gui-zu-a-gibt-es-keine-quellzeile = There is no source line for ${ $a }: a breakpoint cannot be set there.
gui-zuletzt-geoeffnet = recently opened
gui-zuletzt-geoeffnet-2 = Recently opened
gui-zum-editor-wechseln-befehlspalette-markdown-vorschau = Switch to the editor (command palette: Markdown: Switch between preview and source)
gui-zur-gerenderten-ansicht-wechseln = Switch to the rendered view
gui-zur-quelle-rel-z = To the source ({ $rel }:{ $z })
gui-zuruecksetzen = Reset

gui-menue = ☰ Menu

gui-alternative-von-hand-a = Alternative by hand: { $a }

gui-angepasst-pfad-e-in-quelltexten-angepasst = ({ $angepasst } path(s) adjusted in source code)

gui-geloescht = Deleted:

gui-in-den-papierkorb-gelegt = Moved to the trash:

gui-sid-datei-sid-goattracker-sng-oder-song = SID file (.sid), GoatTracker (.sng) or song (.vcsong)

gui-vintagecoder-projekt = VintageCoder project

gui-aufgeklappt = expanded

gui-zugeklappt = collapsed
