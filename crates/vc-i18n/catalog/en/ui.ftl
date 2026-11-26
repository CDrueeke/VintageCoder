# SPDX-License-Identifier: GPL-3.0-or-later
# New interface (ui/): navigation, start view, command palette, component library, measurement (English)

nav-main-label = Main sections
nav-sub-label = Subsections
nav-main-code = Code
nav-main-grafik = Graphics
nav-main-musik = Music
nav-main-daten = Data
nav-main-debug = Debug
nav-grafik-sprites = Sprites
nav-grafik-zeichensatz = Character set
nav-grafik-kacheln = Tiles
nav-grafik-bildschirm = Screen
nav-grafik-karten = Maps
nav-grafik-palette = Palette
nav-daten-disk = Disk
nav-daten-hex = Hex
nav-daten-speicher = Memory
view-title-code = Code editor
view-title-musik = Music editor
view-title-debug = Debugger
view-title-sprites = Sprite editor
view-title-zeichensatz = Character set editor
view-title-kacheln = Tile editor
view-title-bildschirm = Screen editor
view-title-karten = Map editor
view-title-palette = Palette
view-title-disk = Disk editor
view-title-hex = Hex view
view-title-speicher = Memory map
view-placeholder-text = This section follows in stage { $stage } of the new interface.
view-placeholder-hint = Until then it is available in the previous interface.
tree-label = Files
tree-heading = Project
tree-empty = This project contains no files yet.
tree-no-project = No project open.
titlebar-no-project = No project
titlebar-language = Language
titlebar-theme = Color scheme
lang-choice-auto = Automatic
lang-choice-pseudo = Pseudo language (for testing only)
theme-light = Light
theme-dark = Dark
theme-system = System
palette-group-commands = Commands
palette-group-places = Go to
palette-empty = Nothing found.
palette-hint = ↑ ↓ to choose, Enter to run, Esc to close
palette-count = { $n ->
    [one] 1 result
   *[other] { $n } results
 }
status-keys-hint = All commands
toast-not-yet = “{ $name }” follows in a later stage.
toast-close = Dismiss message
err-projekt-fehlt = The project folder “{ $path }” does not exist.
err-projekt-fehler = The project could not be read: { $detail }
err-menue-fehler = The menu bar could not be built: { $detail }
err-sprache-unbekannt = Unknown language “{ $lang }”.
err-befehl-unbekannt = Unknown command “{ $cmd }”.
err-arg-fehlt = The command is missing the argument “{ $name }”.
err-doc-unbekannt = Document { $id } is not open.
err-unbekannt = Unknown error ({ $code }).
err-keine-verbindung = No connection to the program core.
dialog-ok = OK
keycapture-idle = Press a key or shortcut
keycapture-active = Press the keys now …
keycapture-clear = Reset
gallery-title = Component library
gallery-intro = All building blocks of the new interface in one place: card, segmented control, switch, slider, table, dialog, menu, tooltip, shortcut display and key capture.
gallery-card-title = Card
gallery-card-body = A card groups controls that belong together.
gallery-segment-label = Drawing mode
gallery-segment-a = Multicolor
gallery-segment-b = Single color
gallery-switch-label = Show grid
gallery-slider-label = Zoom
gallery-table-label = Example table
gallery-table-col-name = Name
gallery-table-col-size = Size
gallery-table-col-kind = Kind
gallery-dialog-open = Open dialog
gallery-dialog-title = Example dialog
gallery-dialog-body = Focus stays inside the dialog; Esc closes it and returns focus.
gallery-menu-open = Open menu
gallery-menu-a = First item
gallery-menu-b = Second item
gallery-menu-c = Disabled item
gallery-tooltip-target = Hover or focus for a tooltip
gallery-tooltip-text = Tooltips also appear on keyboard focus.
gallery-kbd-label = Shortcut display
gallery-keycapture-label = Key capture (by physical keys)
gallery-button-primary = Primary action
gallery-button-plain = Secondary action
gallery-row-asm = Source
gallery-row-song = Song
gallery-row-sprite = Sprite
measure-title = Measurement (checkpoint G1)
measure-intro = Measures the command round trip and the time from pointer event to drawn frame.
measure-run = Run measurement
measure-running = Measuring …
measure-col-test = Measurement
measure-col-median = Median
measure-col-max = Maximum
measure-col-n = Count
measure-unit-ms = ms
measure-ping-small = Command with small payload
measure-raster-delta = Command with pixel raster delta
measure-binary = Binary reply (64 KiB)
measure-pointer-local = Pointer event to frame (vsync), local preview stroke only
measure-pointer-bundled = Pointer event to frame (vsync), bundled operation with Rust
measure-canvas-label = Test canvas (24 × 21 pixels)
measure-env = Environment
lang-entry = Language: { $name }

measure-col-mean = Mean

measure-pointer-local-sync = Pointer event to stroke, local only (no frame wait)

measure-pointer-bundled-sync = Pointer event to stroke, bundled with Rust round trip (no frame wait)
err-kein-projekt = No project is open.
err-datei-fehler = { $detail }
err-datei-kein-utf8 = "{ $rel }" is not a UTF-8 text file and cannot be opened as text here.
err-pfad-ungueltig = Invalid path: { $detail }
editor-aria = Source text "{ $name }"
editor-eol-vereinheitlicht = "{ $name }" had mixed line endings; they were unified to Unix line endings on save.
editor-leer = Pick a file from the project on the left to edit it.
status-cursor = Line { $line }, column { $col }
status-selected = { $n } selected
tabs-label = Open files
tab-ungespeichert = Unsaved changes
tab-schliessen = Close "{ $name }"
close-title = Unsaved changes
close-frage = "{ $name }" has unsaved changes. Save before closing the tab?
close-speichern = Save
close-verwerfen = Discard
dialog-abbrechen = Cancel
dialog-anlegen = Create
dialog-umbenennen = Rename
dialog-loeschen = Delete
tree-neue-datei = New file
tree-neuer-ordner = New folder
tree-name-label = Name (path relative to the chosen folder)
tree-umbenennen = Rename
tree-loeschen = Delete
tree-loeschen-frage = Move "{ $name }" to the trash?
tree-oeffnen = Open
tree-menu = Actions for the selection
tree-name-leer = The name must not be empty.
tree-name-zeichen = The name contains an invalid character (\ : * ? " < > |).
tree-geloescht-papierkorb = "{ $name }" is now in the trash.
tree-geloescht-endgueltig = "{ $name }" was deleted permanently (no trash available).
cm-find = Find
cm-replace = Replace with
cm-next = next
cm-previous = previous
cm-all = all
cm-match-case = match case
cm-regexp = regexp
cm-by-word = by word
cm-replace-one = replace
cm-replace-all = replace all
cm-close = close
cm-current-match = current match
cm-on-line = on line
cm-replaced-n = replaced $ matches
cm-replaced-one = replaced match on line $
cm-goto-line = Go to line
cm-go = go
cm-folded-lines = Folded lines
cm-unfolded-lines = Unfolded lines
cm-to = to
cm-folded-code = folded code
cm-unfold = unfold
cm-fold-line = Fold line
cm-unfold-line = Unfold line
cm-selection-deleted = Selection deleted
cm-control-char = Control character
editor-goto-title = Go to line
editor-goto-label = Line number (1 to { $max })
editor-goto-ok = Go to
editor-goto-invalid = Please enter a number from 1 to { $max }.
leave-title = Unsaved changes
leave-alle-speichern = Save all
err-datei-nichttext = "{ $rel }" is not a text file.
err-ordner-in-sich = A folder cannot be moved into itself.
err-nichts-zu-verschieben = "{ $rel }": there is nothing to move.
err-verschieben-braucht-projektdatei = Moving needs a project file (.vcprj).
err-verschieben-fehlgeschlagen = Move failed: { $detail }
err-nichts-rueckgaengig = There is nothing to undo.
err-rueckgaengig-fehlgeschlagen = { $detail }
err-extern-fehlgeschlagen = The program could not be started: { $detail }
tree-verschieben-erst-speichern = Moving: save all changed files first.
tree-verschoben = { $n ->
    [one] "{ $name }" moved (1 path in sources adjusted).
   *[other] "{ $name }" moved ({ $n } paths in sources adjusted).
 }
tree-rueckgaengig-gemacht = { $n ->
    [one] Move undone (1 file).
   *[other] Move undone ({ $n } files).
 }
tree-alle-zuklappen = Collapse all
tree-neu-einlesen = Reload
tree-als-include = Insert as include
tree-pfad-kopieren = Copy path
tree-im-ordner-zeigen = Show in folder
tree-pfad-kopiert = Path copied: { $path }
tree-filter-label = Filter files
tree-filter-keine = No matches.
tree-rueckgaengig = Undo move
tree-loeschen-endgueltig-frage = Delete "{ $name }" permanently? There is no trash on this system.
tree-neue-notiz = New note
tree-art-label = Type
tree-art-asm = Assembler (.asm)
tree-art-basic = BASIC (.bas)
tree-art-text = Text (.txt)
tree-art-notiz = Note (.md)
editor-extern-neu = "{ $name }" was changed outside and has been reloaded.
editor-extern-konflikt = "{ $name }" was changed outside and has unsaved changes here. Saving will overwrite the file.
editor-extern-geloescht = "{ $name }" was deleted outside.
editor-unbenannt = Untitled { $n }
editor-notiz = Note { $n }
editor-speichern-unter = Save as
editor-datei-oeffnen = Open file
editor-draussen = "{ $path }" is outside the project.
editor-nichttext-extern = "{ $name }" is not a text file and opens in the default program.
tab-andere-schliessen = Close others
tab-alle-schliessen = Close all
tab-nach-links = Move left
tab-nach-rechts = Move right
status-lang-asm = Assembler
status-lang-basic = BASIC
status-lang-plain = Text
status-lang-markdown = Markdown
palette-dateien = Files
err-intern = Internal error in command "{ $cmd }". Details are in the panic log (vintagecoder-panik.log in the temp folder).

# Kontext: Schweregrad einer Meldung der Sprachdienste (Karte beim Daraufzeigen, Liste Probleme)
diag-sev-1 = Error
diag-sev-2 = Warning
diag-sev-3 = Note
diag-sev-4 = Tip

# Kontext: Meldungen der Sprachdienste im Editor
lsp-keine-dienste = This tab has no language services.
lsp-keine-korrektur = There is no quick fix for this line.
lsp-keine-definition = No definition found.

# Kontext: Liste Probleme unter dem Editor und Zähler in der Statuszeile
problems-none = No problems.
problems-line = Line { $line }
status-problems = { $e ->
    [0] { $w ->
        [0] No problems
        [one] 1 warning
       *[other] { $w } warnings
    }
    [one] { $w ->
        [0] 1 error
        [one] 1 error, 1 warning
       *[other] 1 error, { $w } warnings
    }
   *[other] { $w ->
        [0] { $e } errors
        [one] { $e } errors, 1 warning
       *[other] { $e } errors, { $w } warnings
    }
}

# Kontext: Meldungen nach dem Formatieren (Prettyprint) im Code-Editor
format-fertig = Formatted.
format-nichts = Already formatted, nothing to change.
format-sortieren-auswahl = Select several lines to sort.
format-nicht-moeglich = This tab cannot be formatted (assembler and BASIC only).

# Kontext: Ablage unter dem Editor (Reiter Probleme, Suche, Gliederung)
dock-label = Tray below the editor
dock-problems = Problems
dock-search = Search
dock-outline = Outline
dock-close = Close tray

# Kontext: Suche im Projekt (Ablage unter dem Editor)
search-label = Search in project
search-placeholder = Search all files …
search-abbrechen = Cancel
search-laeuft = { $n ->
    [one] Searching … 1 hit
   *[other] Searching … { $n } hits
 }
search-abgebrochen = { $n ->
    [one] Cancelled after 1 hit
   *[other] Cancelled after { $n } hits
 }
search-gekuerzt = More than { $n } hits, the list is cut off
search-nichts = No hits
search-fertig = { $n ->
    [one] 1 hit
   *[other] { $n } hits
} in { $files ->
    [one] 1 file
   *[other] { $files } files
}

# Kontext: Gliederung der Datei (Ablage unter dem Editor)
outline-leer = No labels, constants or macros in this file.
outline-label = Label
outline-const = Constant
outline-macro = Macro
# Kontext: Hinweis beim Umbenennen im Projektbaum
tree-umbenennen-erst-speichern = Renaming: save all changed files first.
# Kontext: Meldung nach dem Umbenennen, wenn Include-Pfade in Quelltexten angepasst wurden
tree-includes-angepasst = { $n ->
    [one] 1 include path in sources adjusted.
   *[other] { $n } include paths in sources adjusted.
 }

# Kontext: Beschriftung des Kontextmenüs im Code-Editor
editor-kontextmenu = Editor menu

# Kontext: Art-Auswahl im Dialog „Neue Datei“ und „Speichern unter“: keine Endung anhängen
tree-art-keine = No extension

# Kontext: Statuszeile: Zähler der Hinweise (Schweregrad 3) der Sprachdienste
status-hinweise = { $h ->
    [one] 1 note
   *[other] { $h } notes
}

# Kontext: Statuszeile: weitere Meldungen (Fehler, Warnungen, Hinweise) in anderen Dateien des Projekts als die im Reiter
status-andere-dateien = { $n ->
    [one] 1 message in other files
   *[other] { $n } messages in other files
}

# Kontext: Probleme-Liste: Kurzzeile für das ganze Projekt
problems-projekt = Project: { $n } messages in { $m ->
    [one] 1 file
   *[other] { $m } files
}, { $e } of them errors in { $f ->
    [one] 1 file
   *[other] { $f } files
}

# Kontext: Bauen und Starten (Kopfzeile, Ausgabe, Meldungen)
bau-erst-speichern = Build: save all changed files first.
bau-log-bauen = Building: { $rel } …
bau-log-spruenge = { $n } branches too far, extended automatically (`.branchlong auto`).
bau-log-fehler = Error: { $e }
bau-log-disk = Disk { $file } ({ $size } bytes): { $programs }
bau-log-disk-fehler = Disk: { $e }
bau-log-fertig = Done: { $size } bytes in { $artifact }
bau-fehlgeschlagen = { $n ->
    [one] Build failed (1 error). See Problems.
   *[other] Build failed ({ $n } errors). See Problems.
 }
run-log-gestartet = Started: { $line }
run-log-fehler = Start failed: { $e }
run-log-beendet = Program ended.
run-fehlgeschlagen = Start failed: { $e }
run-beendet = Emulator stopped.
run-nichts-laeuft = Nothing is running.
err-bau-laeuft = A build is already running.
err-nichts-zu-starten = Nothing to start: build first.
err-keine-quelle = No source file to build: the project has no .asm or BASIC file (or the active file is not one).
err-daten-fehlen = The data folder with the target tables was not found.
bar-bauen = Build
bar-starten = Run
bar-stopp = Stop
bar-baut = building …
bar-gebaut = built: { $size } bytes
bar-fehler = { $n ->
    [one] Build: 1 error
   *[other] Build: { $n } errors
 }
bar-laeuft = running
bar-ziel = Run target
dock-output = Output
output-leer = No output yet. Build with F7.
output-leeren = Clear
output-kopieren = Copy
output-kopiert = Output copied.
problems-bau = Build result

# Kontext: Statuszeile: Speicherstreifen des gebauten Programms
bar-speicher = Memory used: { $segs }

# Kontext: Probleme: Meldungen nach dem Kopieren und Beschriftung des Kontextmenüs
problems-kopiert = Copied to the clipboard.
problems-keine-wahl = No problem selected.
problems-title-menu = Problem

# Kontext: Maschine-Panel (rechts): Zustand und Bauergebnis
machine-title = Machine
machine-system = System
machine-ziel = Run target
machine-zustand = State
machine-laeuft = running
machine-bereit = ready
machine-bau = Last build
machine-quelle = Source
machine-datei = Program
machine-groesse = Size
machine-bytes = { $n } bytes
machine-start = Start (SYS)
machine-segment = No
machine-von = from
machine-bis = to
machine-laenge = Length
machine-kein-bau = Not built yet. Build with F7.
machine-spaeter = During a debug session this panel shows the registers, flags and the picture of the machine.

# Kontext: Tests (Ablage unter dem Editor)
dock-tests = Tests
tests-alle = Run all
tests-fehlgeschlagene = Failed
tests-abbrechen = Cancel
tests-ziel = Target
tests-nur-statisch = Static checks only
tests-filter = Filter tests …
tests-nur-fehler = errors only
tests-laeuft = Running …
tests-stand = { $p } of { $n } passed
tests-keine = No run yet, or no tests (`*.vctest`) in the project.
tests-state-passed = passed
tests-state-failed = failed
tests-state-timeout = timed out
tests-state-skipped = skipped
tests-hinweis = note
tests-gemessen = Measured
tests-quelle = Go to source
tests-testdatei = Open test file
tests-waehlen = Select a test to see expectation and registers.

# Kontext: Probleme: zusammengefasste Warnungen zu weiter Sprünge (W203)
problems-spruenge = { $n } branches too far, extended automatically (`.branchlong auto`)
problems-aufklappen = show each

# Kontext: Ende des Emulators (vom Nutzer beendet oder abgestürzt)
run-log-ende = Emulator ended.
run-log-ende-code = Emulator ended (exit code { $code }).
run-ende-code = The emulator ended with exit code { $code }. See Output.

# Kontext: Beenden mit Liste und Wiederherstellen unbenannter Reiter (E1.4)
leave-frage = { $n ->
    [one] 1 tab with unsaved changes. Save before continuing?
   *[other] { $n } tabs with unsaved changes. Save before continuing?
 }
leave-unbenannt = (unnamed, lost unless saved)
scratch-title = Restore unnamed tabs
scratch-frage = The program was not closed cleanly last time. { $n } unnamed tab(s) with content were backed up. Restore them?
scratch-zeichen = { $n } characters
scratch-hinweis = The backup is stored unencrypted on this computer only and expires after 14 days.
scratch-wiederherstellen = Restore

# Kontext: Layout der Arbeitsfläche (E1.4)
splitter-hilfe = Adjust with the arrow keys, Shift for large steps, Home and End for smallest and largest size, double-click restores the default.
splitter-dock = Height of the drawer
splitter-panel = Width of the machine panel
splitter-seite = Width of the sidebar
bereich-maximieren = Maximize area
bereich-wiederherstellen = Restore area
layout-zurueck = Layout reset.

# Kontext: Geteilte Ansicht (E1.4)
split-zu-wenig = Splitting needs at least two tabs.
split-hierher = Drop here
split-rechts = Drop on the right (splits)

# Kontext: Neues Projekt (E1.4)
err-neuprojekt-fehler = The project could not be created: { $detail }
np-zielsystem = Target system
np-vorlage = Template
np-sprache = Language
np-struktur = Folder structure
np-kein-zeichensatz = No character set; you create one later in the character set editor.
neuprojekt-angelegt = Project created: { $dir }
err-kopieren-fehler = The project could not be copied: { $detail }

# Kontext: Startseite (E1.4)
start-projektdatei = Choose project file (.vcprj) …
start-anheften = Pin project
start-loesen = Unpin project
start-entfernen = Remove from list

# Kontext: Bereichsnamen für Bildschirmleser (E1.4)
split-gruppe-links = Document area, left
split-gruppe-rechts = Document area, right
split-gruppe-dokument = Document area

# Kontext: Zuletzt geöffnete Projekte in der Befehlspalette (E1.4)
palette-projekte = Projects
palette-projekt = Open project: { $name } ({ $path })
# Kontext: Startseite, Beispielprojekte (E1.4 Rest)
start-beispiele-alle = All examples …
start-beispiele-suche = Search examples
start-beispiele-keine = No examples found.
# veraltet seit E-Start: Empfehlung wird nicht mehr als Beschriftung gezeigt
start-beispiele-empfohlen = Recommended
# Kontext: Startseite, Zuletzt-Liste (E1.4 Rest 2)
start-angeheftet-kopf = Pinned
start-zuletzt-kopf = Recent
start-weitere = More … ({ $n })
start-weniger = Show fewer
start-aktionen = More actions for { $name }
# Kontext: Disk-Editor (E2)
disk-oeffnen = Open disk image …
disk-abbilder = Disk images
disk-leer-titel = No disk image open
disk-leer-text = Open a disk image (D64, D71, D81 or ATR) or double-click such a file in the project tree.
err-disk-unlesbar = The disk image cannot be read: { $detail }
disk-karte-diskette = Disk
disk-belegung = Usage
disk-belegt-frei = { $used } blocks used · { $free } free
disk-aktionen = Actions
disk-schliessen = Close disk image
disk-verzeichnis = Directory
disk-n-dateien = { $n ->
    [one] 1 file
   *[other] { $n } files
 }
disk-spalte-name = Name
disk-spalte-typ = Type
disk-spalte-bloecke = Blocks
disk-spalte-groesse = Size
disk-spalte-start = Start
disk-autostart = Autostart
disk-geschuetzt = File locked
disk-belegungskarte = Allocation map
disk-legende-datei = Selected file
disk-legende-belegt = Used
disk-legende-verz = Directory
disk-legende-kopf = Header/BAM
disk-kette-zeigen = Show chain
disk-karte-label = Allocation map, arrow keys select the sector
disk-karte-ansage = Track { $t }, sector { $s }: { $was }
disk-folgen-zu = Follow ▶ { $a }
disk-kettenende = End of chain
disk-zurueck = Back
disk-datei-block = { $name } · block { $n } of { $m }
disk-naechster-sektor = Next sector
disk-bytes-im-sektor = Bytes in this sector
disk-vormerken = Mark
disk-vormerken-datei = Mark: { $name }
disk-vm-alle = All
disk-vm-keine = None
disk-vm-prg = Programs
disk-vm-basic = BASIC
disk-ablegen = Add files …
disk-pruefen = Check
disk-uebernehmen = Import into project
disk-speichern = Save
disk-speichern-unter = Save as …
disk-neu = New disk …
disk-ungespeichert = Image not saved
disk-bericht = Check report
disk-bericht-ok = No problems found.
disk-uebernahme = Import
disk-ziel = Target folder in the project
disk-opt-basic = BASIC as source text
disk-opt-raw = Keep original file
disk-opt-asm = Also as assembler
disk-ziel-ungueltig = The target folder must be inside the project.
disk-projekt-noetig = A project must be open to import.
disk-ohne-dateisystem = This disk has no file system.
disk-neu-titel = New disk
disk-neu-format = Format
disk-neu-name = Disk name
disk-neu-id = ID (2 characters)
disk-neu-anlegen = Create
disk-frage-titel = Unsaved disk
disk-frage-text = The disk "{ $name }" has unsaved changes.
err-disk-ohne-pfad = The disk has no location yet.
err-disk-zielordner = The target folder must be inside the project.
err-disk-verschieben = { $detail }
disk-datei-hoch = Move up
disk-datei-runter = Move down
disk-ordnen-hinweis = Order in the directory: drag a row or press Alt+Arrow up/down
disk-ordnen-zeile = Move entry { $name }
err-disk-nichts-vorgemerkt = No file is marked.
err-disk-neu = The disk cannot be created: { $detail }
disk-einlegen = Insert in emulator
disk-einlegen-erst = Save first, then insert
disk-eingelegt = Inserted: { $c }
err-disk-erst-speichern = Save first, then insert.

sprite-leer-titel = No sprite open
sprite-leer-text = Choose a sprite file from the project or open another one.
sprite-leer-kein-projekt = A project must be open to edit sprites.
sprite-keine-dateien = This project has no sprite files yet.
sprite-oeffnen = Open sprite file …
sprite-dateien = Sprite files
sprite-werkzeuge = Tools
sprite-flaeche-label = Drawing area
sprite-flaeche-hilfe = Arrow keys move the cell marker, Enter or Space sets a dot, Delete removes it. Digits choose the color value, D selects the pen, E the eraser, I the picker. Plus and minus zoom, G toggles the grid.
sprite-zelle = x { $x } y { $y }
sprite-zelle-ansage = Cell { $x }, { $y }: { $wert }
sprite-wert = Value { $n }
sprite-wert-gewaehlt = Color value { $n }: { $name }
sprite-karte-farben = Colors
sprite-farbe-fuer = Color for "{ $name }"
sprite-durchsichtig = This value is transparent and has no color.
sprite-palette = Palette: { $name }
sprite-ansicht = Palette view
sprite-ansicht-projekt = Project
sprite-raster = Grid
sprite-zoom-plus = Zoom in
sprite-zoom-minus = Zoom out
sprite-zoom-einpassen = Fit
sprite-zoom-text = { $z } %
sprite-fuss = { $name } · { $w } × { $h } · frame { $i } of { $n }
sprite-kurz-wert = Color value
sprite-kurz-zoom = Zoom
sprite-frage-titel = Unsaved sprite file
sprite-frage-text = The file "{ $name }" has unsaved changes.
sprite-ungespeichert = unsaved
err-pixel-unlesbar = The file "{ $rel }" cannot be opened: { $detail }
err-pixel-speichern = The file cannot be saved: { $detail }

sprite-kopiert = Copied: { $w } x { $h } dots
sprite-ausgeschnitten = Cut: { $w } x { $h } dots
err-pixel-keine-auswahl = Nothing is selected.
err-pixel-zwischenablage-leer = The sprite editor clipboard is empty.

sprite-spiegeln-h = Flip horizontally
sprite-spiegeln-v = Flip vertically
sprite-verschieben = Shift
sprite-links = Left
sprite-rechts = Right
sprite-oben = Up
sprite-unten = Down
sprite-leeren = Clear frame
sprite-auswahl-menue = Selection
sprite-auswahl-loeschen = Delete selection
sprite-auswahl-aufheben = Clear selection
sprite-auswahl-ansage = Selection { $w } x { $h }
sprite-anfang-gesetzt = Start set at { $x }, { $y }. Move the cell marker, Enter sets the end, Escape cancels.
sprite-abgebrochen = Cancelled

sprite-bilder = Frames
sprite-bilder-n = { $n ->
    [one] 1 frame
   *[other] { $n } frames
 }
sprite-bild-neu = New
sprite-bild-dupl = Duplicate
sprite-bild-loeschen = Delete
sprite-bild-ansage = Frame { $i } of { $n }
sprite-onion = Onion Skin
sprite-karte-sprite = Sprite
sprite-multicolor = Multicolor
sprite-doppelt-breit = Double width
sprite-doppelt-hoch = Double height
sprite-hinter-hintergrund = Behind the background
sprite-im-spiel = { $n ->
    [one] In the game: 1 dot
   *[other] In the game: { $n } dots
 }
sprite-karte-vorschau = Preview in the game
sprite-speicher = { $bytes } bytes per frame · { $n } frames · { $total } bytes

export-routine-rem = Load data: GOSUB @{ $label }_load reads the DATA lines from the current pointer into memory at { $addr }
export-format-unbekannt = Unknown export format.
export-routine-braucht-adresse = The read routine needs a valid address, e.g. $2000.
err-export-fehler = The export failed: { $detail }
err-export-abgelehnt = Including is not possible here: { $detail }
err-export-adresse = This format needs a valid address.
err-export-routine-braucht-adresse = The read routine needs a valid address, e.g. $2000.

export-titel = Export
export-format = Format
export-format-asm = Assembler
export-format-inc = Constants
export-format-bin = Binary
export-format-prg = PRG
export-format-data = BASIC DATA
export-name = Name
export-adresse = Address
export-ladeadresse = Load address
export-routine = Include a read routine
export-routine-hinweis = The routine reads the DATA lines into memory with POKE, starting at the address. Call it with GOSUB.
export-vorschau = Preview
export-tab-text = Text
export-tab-hex = Hex
export-keine-binaer = No binary data.
export-bereit = { $n ->
    [one] Ready: 1 file, { $bytes } bytes
   *[other] Ready: { $n } files, { $bytes } bytes
 }
export-ziel = Target in the source
export-kein-ziel = No matching source file is open. Open the file you want to insert into first.
export-testen = Test in emulator
export-speichern = Save in project
export-include = Insert as include
export-kopieren = Copy into source
export-gespeichert = Saved in the project: { $files }
export-eingefuegt = Inserted in { $doc }: { $line }
export-kopiert = Copied into { $doc }
export-nichts = To export, open an editor with a media file first.

err-pixel-anim = The animation cannot be changed: { $detail }
err-pixel-png = The image cannot be loaded: { $detail }
sprite-png-laden = Load PNG image

sprite-zeitleiste = Timeline
sprite-keine-animation = No animation. Create one in the Animations tab.
sprite-abspielen = Play
sprite-stopp = Stop
sprite-ablauf = Change playback mode
sprite-ablauf-loop = Loop
sprite-ablauf-pingpong = Back and forth
sprite-ablauf-once = Once
sprite-anim-dauer = { $ticks } frames · { $s } s
sprite-anim-schritte = Animation steps
sprite-schritt-titel = Step { $i }: frame { $f }, { $d } ticks long
sprite-reiter-bilder = Frames
sprite-reiter-anim = Animations
sprite-anim-neu = New
sprite-anim-loeschen = Delete
sprite-anim-name = Animation name
sprite-anim-keine = No animation yet. "New" creates one with the selected frame.
sprite-anim-anhaengen = Append selected frame
sprite-anim-hoch = Move step up
sprite-anim-runter = Move step down
sprite-anim-entfernen = Remove step
sprite-anim-dauer-label = Duration in ticks (1/50 s)
sprite-anim-fehler = The animations are invalid: { $e }
sprite-png = Load PNG …
sprite-png-tipp = Loads a PNG image into the selected frame; the file is saved first
sprite-png-nichts = To load a PNG image, open a file in the sprite editor first.

err-media-fehler = The file cannot be created: { $detail }
sprite-neu = New sprite file …
sprite-neu-titel = New sprite file
sprite-neu-profil = Type and system
sprite-neu-name = Name (letters, digits, _ and -)
sprite-neu-ordner = Folder in the project
sprite-neu-anlegen = Create
sprite-neu-name-ungueltig = The name may only contain letters, digits, _ and -.
system-name-vic20 = VIC-20

sprite-karte-pmg = Player/missile
sprite-pmg-breite = Width of player { $n }
sprite-pmg-breite-0 = Normal
sprite-pmg-breite-1 = Double
sprite-pmg-breite-3 = Quadruple
sprite-pmg-prior = Priority
sprite-pmg-prior-0 = Not set
sprite-pmg-prior-1 = Players before playfield
sprite-pmg-prior-2 = P0/P1, playfield, P2/P3
sprite-pmg-prior-4 = PF0/PF1, players, PF2/PF3
sprite-pmg-prior-8 = Playfield before players
sprite-pmg-sichtbar = A player shows its memory line by line from top to bottom. The first 32 or so lines lie above the visible picture: draw the artwork further down, or increase the image height. A priority that is not set is exported as 1 (players before playfield).
sprite-bilder-markiert = { $n ->
    [one] 1 frame selected
   *[other] { $n } frames selected
}
sprite-anim-aus-auswahl = Animation from selection
sprite-anim-aus-auswahl-punkte = Animation from selection …
sprite-alle-markieren = Select all frames
sprite-markierung-aufheben = Clear selection
sprite-markier-hinweis = Shift- or Ctrl-click to select several frames and make an animation from them.
sprite-anim-aus-titel = { $n ->
    [one] Animation from 1 frame
   *[other] Animation from { $n } frames
 }
sprite-anim-aus-ablauf = Playback
sprite-anim-aus-tempo = Speed (frames per second)
sprite-anim-aus-ticks = equals { $t } ticks (1/50 s) per frame
sprite-anim-aus-hinweis = The frames play in the order of the frame list.
sprite-anim-aus-erzeugen = Create animation
sprite-anim-aus-vergeben = This name is already taken.
sprite-anim-aus-ungueltig = The name may only contain letters, digits and _ and must not start with a digit.
sprite-anim-schritte-liste = Animation steps
sprite-anim-ziehen-hinweis = Drag steps or use the arrows to reorder them; click a picture to edit that frame.
sprite-anim-tempo-gemischt = different duration per frame
ext-werkzeuge-folgen = Commands from extensions run with the tools that follow in a later version.
release-ohne-editor = “{ $name }” opens as text: the { $editor } is not included in this release.
release-editor-sprite = Sprite editor
release-editor-disk = Disk editor
palette-herkunft = Command from the extension { $paket }
beispiele-herkunft = Example from the extension { $paket }

ablegen-hier = Drop files here to open them
ablegen-kopieren-titel = Copy into the project and open?
ablegen-kopieren-text = { $n ->
    [one] This file is outside the project. It will be copied into the project (nothing is overwritten) and opened there:
   *[other] These { $n } files are outside the project. They will be copied into the project (nothing is overwritten) and opened there:
}
ablegen-kopieren-ja = Copy and open
ablegen-kopiert = { $n ->
    [one] 1 file copied into the project.
   *[other] { $n } files copied into the project.
}
ablegen-kein-projekt = “{ $name }” can only be dropped when a project is open. Open a project first.
ablegen-ordner = “{ $name }” is a folder without a project file and is not opened.
ablegen-fehlt = “{ $name }” no longer exists.
ablegen-typ = “{ $name }” has a file type VintageCoder does not open.
err-ablegen-keine-datei = “{ $name }” is not a file.
err-ablegen-zu-gross = “{ $name }” is too large to copy (over 32 MB).
sprite-vorschau-gross = Enlarge preview
sprite-vorschau-gross-titel = Animation “{ $name }”
sprite-zoom-label = Zoom
sprite-vorschau-schritt = Step { $i } of { $n }
dialog-schliessen = Close
release-editor-char = Character set editor
zeichensatz-leer-titel = No character set open
zeichensatz-leer-text = Choose a character set file (.vcchar) from the project or open another one.
zeichensatz-leer-kein-projekt = A project must be open to edit character sets.
zeichensatz-keine-dateien = The project does not contain a character set file yet.
zeichensatz-dateien = Character set files
zeichensatz-uebersicht-titel = Character overview
zeichensatz-anzahl = { $n } characters
zeichensatz-ansage = Character { $n }, { $hex }
zeichensatz-flaeche-label = Edit character
zeichensatz-umgebung = Neighbors
zeichensatz-chip = Character { $n } · { $hex }
zeichensatz-fuss = { $name } · character { $n } ({ $hex })
zeichensatz-vorschau-titel = Preview in the picture
zeichensatz-vorschau-label = Preview of a text in this character set
zeichensatz-text-label = Text for the preview
zeichensatz-karte-zeichen = Character
zeichensatz-nummer = Number
zeichensatz-nummer-hinweis = Screen code
zeichensatz-position = Position in the set
zeichensatz-position-hinweis = Byte offset from the start ({ $n } bytes per set)
zeichensatz-zeichen-leeren = Clear character
zeichensatz-vorschau-text = HUGO AND GUMBO\nSCORE 00420
zeichensatz-bereiche = Ranges
zeichensatz-bereich-alle = All
zeichensatz-bereich-buchstaben = Letters
zeichensatz-bereich-ziffern = Digits
zeichensatz-bereich-zeichen = Punctuation
zeichensatz-bereich-frei = Custom
zeichensatz-bereich-von = from
zeichensatz-bereich-bis = to
zeichensatz-bereich-ansage = Range { $name }: characters { $von } to { $bis }
zeichensatz-vorlage-titel = Template
zeichensatz-vorlage-label = Template to load
zeichensatz-vorlage-keine = Designed by you, no template loaded
zeichensatz-vorlage-laden = Load template
zeichensatz-vorlage-hinweis = Replaces all characters; Undo brings them back. Sets from a ROM are stored only as a reference in the file.
zeichensatz-vorlage-fehlt = ROM not found
zeichensatz-herkunft = Origin
zeichensatz-wie-vorlage = Same as the template
zeichensatz-geaendert = Changed from the template
zeichensatz-original-zurueck = Reset character to template
zeichensatz-aenderungen = { $n ->
    [one] 1 character differs from the template
   *[other] { $n } characters differ from the template
 }
zeichensatz-neu = New charset …
zeichensatz-neu-titel = New charset
zeichensatz-neu-name = charset
err-pixel-vorlage = The template could not be loaded: { $detail }
zeichensatz-uebersicht-kurz = Characters
nav-main-projekt = Project
nav-projekt-uebersicht = Overview
nav-projekt-allgemein = General
nav-projekt-ziele = Targets
nav-projekt-quellen = Sources
nav-projekt-bauen = Build and run
nav-projekt-diskette = Disk
nav-projekt-weitergeben = Share
view-title-projekt-uebersicht = Project: Overview
view-title-projekt-allgemein = Project: General
view-title-projekt-ziele = Project: Targets
view-title-projekt-quellen = Project: Sources
view-title-projekt-bauen = Project: Build and run
view-title-projekt-diskette = Project: Disk
view-title-projekt-weitergeben = Project: Share
projekt-ohne-projekt = Open a project first (via "Home" at the top left or the command "Open project …"); its overview and settings then appear here.
projekt-seite-folgt = This page comes in one of the next stages of the Project area. You can already switch to the other pages here.
projekt-name-fehlt = Project name missing
projekt-start = Start: { $f }
projekt-kein-programm = no program
projekt-kein-ziel = No target system entered
projekt-quellen-anzahl = Programs: { $n } · source groups: { $g }
projekt-fehlende-dateien = { $n ->
    [one] 1 file is missing on disk
   *[other] { $n } files are missing on disk
 }
projekt-diskette-an = { $n ->
    [one] Disk { $f }, 1 program
   *[other] Disk { $f }, { $n } programs
 }
projekt-turbo-an = Your Turbo BASIC XL disk
projekt-diskette-aus = No disk set up
projekt-zustand-ok = OK
projekt-zustand-warn = Note
projekt-zustand-err = Missing
projekt-zustand-off = Not switched on
projekt-aendern = Change ›
projekt-probleme = Problems in the project file
projekt-name = Name
projekt-beschreibung = Description
projekt-startdatei = Start program
projekt-startdatei-auto = Automatic: first program ({ $f })
projekt-startdatei-hinweis = This program starts with F5 and is built first. Without a choice the first program of the sources is used.
projekt-reiter-wiederherstellen = Restore the last opened files when opening the project
projekt-uebernehmen = Apply
projekt-gespeichert = Saved.
projekt-variante-von = Variant of { $s }
projekt-variante-hinweis = The variant sets clock and line count (for example PAL or NTSC) for emulator, preview and timing.
projekt-keine-variante = This target system has no variants.
projekt-emulator-dieses-ziels = Emulator for this target only: { $e }
projekt-quellen-hinweis = The programs in the order they are built. You can move files within a source group. Missing files are shown in red.
projekt-gruppe = Source group { $n }
projekt-datei-oeffnen = Open the file in the code editor
projekt-start-programm = Start program
projekt-fehlt = missing
projekt-als-start = Make start program
projekt-nach-oben = Move { $f } up
projekt-nach-unten = Move { $f } down
projekt-keine-quellen = The project has no sources yet.
projekt-letzter-bau = Last build
projekt-bau-ok = Built at { $w }, no errors.
projekt-bau-fehler = { $n ->
    [one] Built at { $w }: 1 error.
   *[other] Built at { $w }: { $n } errors.
 }
projekt-noch-nicht-gebaut = Not built in this session yet.
projekt-emulator-je-ziel = Emulator per target
projekt-emulator-projekt = Emulator of the project: { $e }
projekt-emulator-global = Emulator from the settings
projekt-start-auf = starts on { $s }
projekt-emulator-hinweis = Only overrides set by this project appear here. The general choice of emulator and start target is in the settings.
projekt-pakete = Packages (registry)
projekt-keine-pakete = No packages loaded.
projekt-paket-herkunft = Source: { $k } · { $o } · { $p }
projekt-kopieren-titel = Copy project
projekt-kopieren-text = Makes a copy of the project somewhere else, for example to hand it on or as a backup.
projekt-kopieren-knopf = Copy project …
projekt-export-titel = Export of graphics, charsets and music
projekt-export-text = You find the export (as source text, DATA lines or PRG) in each editor with the "Export …" button. The files it creates may be used freely.
projekt-verlauf = Recently opened
err-palette-unlesbar = The palette "{ $rel }" cannot be opened: { $detail }
err-palette-hex = "{ $hex }" is not a color value; use #rrggbb.
palette-rot = Red
palette-gruen = Green
palette-blau = Blue
palette-art-sprite = Sprite
palette-art-pmg = Player/Missile
palette-art-zeichensatz = Charset
palette-art-kacheln = Tiles
palette-art-bildschirm = Screen
palette-leer-titel = No palette open
palette-leer-kein-projekt = Open a project to edit palettes.
palette-leer-text = Choose a palette file (.vcpal) from the project or open another one.
palette-pal-dateien = Palette files
palette-keine-dateien = The project has no palette file yet.
palette-raster-titel = Color grid
palette-werkzeuge = Palette tools
palette-anzahl = { $n ->
    [one] 1 color
   *[other] { $n } colors
 }
palette-standard = Default palette
palette-raster-label = { $n ->
    [one] Color grid with 1 color; arrow keys choose a color
   *[other] Color grid with { $n } colors; arrow keys choose a color
 }
palette-farbe-ansage = Color { $n }, { $hex }, { $farbe }
palette-farbe-titel = Color { $n }
palette-farbfeld = Chosen color { $hex }
palette-regler = Color channels
palette-hexwert = Hex value
palette-hex-ungueltig = That is not a color value; use #rrggbb.
palette-im-einsatz = In use
palette-nicht-im-einsatz = No media file in the project uses this color.
palette-ausgabe = Output
palette-ausgabe-text = As assembler data, { $n } bytes
palette-fuss = { $name } · color { $n } ({ $hex }) of { $count }
kacheln-leer-titel = No tile set open
kacheln-leer-kein-projekt = Open a project to edit tiles.
kacheln-leer-text = Choose a tile set (.vctiles) from the project or open another one.
kacheln-dateien = Tile sets
kacheln-keine-dateien = The project has no tile set yet.
kacheln-satz-titel = Tile set · { $name }
kacheln-neu = New tile
kacheln-liste-label = { $n ->
    [one] Tile set with 1 tile; arrow keys choose a tile
   *[other] Tile set with { $n } tiles; arrow keys choose a tile
 }
kacheln-ansage = Tile { $n }, { $name }, used { $uses } times
kacheln-doppelt = Same as tile { $j }
kacheln-flaeche-label = Edit tile
kacheln-werkzeuge = Tile tools
kacheln-drehen = Rotate tile
kacheln-rollen-links = Roll cells left
kacheln-rollen-rechts = Roll cells right
kacheln-rollen-oben = Roll cells up
kacheln-rollen-unten = Roll cells down
kacheln-leeren = Clear
kacheln-pinsel-label = A click sets
kacheln-pinsel-beides = Character and color
kacheln-pinsel-zeichen = Character only
kacheln-pinsel-farbe = Color only
kacheln-chip = { $w }×{ $h } cells · charset { $cs }
kacheln-zellen-label = Cells of the tile ({ $w } × { $h }); arrow keys move, Enter sets the character, I picks it up
kacheln-zelle-ansage = Cell { $x }, { $y }: character { $code }, { $hex }
kacheln-zeichen-waehlen = Choose character
kacheln-zeichensaetze = Charsets of the tile set
kacheln-tafel-label = Character table with { $n } characters; arrow keys choose a character
kacheln-tafel-hinweis = A click puts the character into the marked cell.
kacheln-bit7 = Set character bit 7 (color selection of the character)
kacheln-fuss = { $name } · tile no. { $n } · { $count } tiles
kacheln-karte-titel = Tile
kacheln-name = Name of the tile
kacheln-farbe-titel = Color
kacheln-farbmodus = Color applies
kacheln-farbe-je-kachel = Per tile
kacheln-farbe-je-zelle = Per cell
kacheln-farbe-ohne = None
kacheln-kachelfarbe = Color of the tile
kacheln-pinselfarbe = Brush color
kacheln-hintergrund = Background color
kacheln-eigenschaften = Properties
kacheln-keine-eigenschaften = The set has no properties yet.
kacheln-eigenschaft-neu = New property
kacheln-hinzufuegen = Add
kacheln-groesse = Size of the tiles
kacheln-breite = Width
kacheln-hoehe = Height
kacheln-uebernehmen = Apply
kacheln-zielzeichensatz = Target charset
kacheln-fuellstand-label = Characters used in the target charset
kacheln-fuellstand = of { $max } characters used · { $frei } free
kacheln-fuellstand-hinweis = Duplicate characters are merged on export. Pinned characters keep their number.
kacheln-zeichensatz-hinzu = Add a charset to the tile set
kacheln-zeichensatz-benutzt = Used by cells
kacheln-entfernen = Remove { $name }
kacheln-banken = { $n ->
    [one] The set is used in 1 screen bank.
   *[other] The set is used in { $n } screen banks.
 }
kacheln-satz-aktionen = Tile
kacheln-duplizieren = Duplicate
kacheln-zusammenfuehren = Merge
kacheln-loeschen = Delete
kacheln-loeschen-titel = Delete tile?
kacheln-loeschen-text = Tile { $i } is used { $uses } times in screen banks. After deleting, those places point to tile 0, higher tiles move up, and the set is saved.
kacheln-groesse-titel = Change size?
kacheln-groesse-text = At { $w } × { $h } cells, filled cells are lost. Undo brings them back.
err-kacheln-unlesbar = The tile set "{ $rel }" cannot be opened: { $detail }
err-kacheln-zwischenablage = Nothing to paste: copy a tile first (same size).
err-kacheln-eigenschaft = The property "{ $name }" cannot be created: name empty or taken, or all eight places are used.
palette-frage-titel = Unsaved palette
kacheln-frage-titel = Unsaved tile set
zeichensatz-frage-titel = Unsaved charset
bildschirm-werkzeug-zeichen = Set character
bildschirm-werkzeug-rahmen = Frame
bildschirm-werkzeug-fuellen = Fill
bildschirm-werkzeug-text = Type text
bildschirm-werkzeug-pipette = Eyedropper
bildschirm-werkzeug-loeschen = Erase
bildschirm-leer-titel = No screen open
bildschirm-leer-kein-projekt = Open a project to edit screens.
bildschirm-leer-text = Choose a screen file (.vcscreen) from the project or open another one.
bildschirm-dateien = Screen files
bildschirm-keine-dateien = The project has no screen file yet.
bildschirm-zeichen-titel = Characters
bildschirm-ohne-zeichensatz = No charset
bildschirm-tafel-label = Character table with { $n } characters; arrow keys choose a character
bildschirm-farbe-titel = Character color
bildschirm-zeichenfarbe = Color of new characters
bildschirm-flaeche-label = Edit screen
bildschirm-werkzeuge = Screen tools
bildschirm-zerlegen = Split into tiles …
bildschirm-text-hinweis = Type text: click a cell, then write; arrow keys move the cursor, Esc ends.
bildschirm-rahmen-hinweis = Frame: press Enter to set the second corner, Esc cancels.
bildschirm-zeichen-hinweis = A click sets the chosen character, the right mouse button erases; Enter sets it at the cursor cell.
bildschirm-zellen-label = Cells of the screen ({ $w } × { $h }); arrow keys move the cursor, Enter applies the tool
bildschirm-zelle-ansage = Column { $x }, row { $y }: character { $code }, { $hex }
bildschirm-zelle-info = Column { $x } row { $y } · character { $code } ({ $hex })
bildschirm-karte-titel = Screen
bildschirm-groesse = Size
bildschirm-groesse-farbram = Text screen with color per cell
bildschirm-groesse-einfarbig = Text screen, one character color
bildschirm-farben-system = System colors
bildschirm-rahmen = Border
bildschirm-hintergrund = Background
bildschirm-zeichen-rolle = Characters
bildschirm-zeichensatz = Charset
bildschirm-ausgabe = Output
bildschirm-ausgabe-text = As assembler data, { $n } characters
bildschirm-zerlegen-titel = Split into tiles
bildschirm-zerlegen-text = Equal blocks become tiles. The tile set and screen bank are created next to the screen; the screen itself stays unchanged.
bildschirm-kachelgroesse = Tile size
bildschirm-zerlegen-los = Split
bildschirm-frage-titel = Unsaved screen
err-bildschirm-unlesbar = The screen "{ $rel }" cannot be opened: { $detail }
err-bildschirm-zerlegen = Splitting into tiles is not possible: { $detail }
tracker-leer-titel = No song open
tracker-leer-kein-projekt = Open a project to edit songs.
tracker-leer-text = Choose a song (.vcsong or .vcplay) from the project or open another one.
tracker-dateien = Songs
tracker-keine-dateien = The project has no song yet.
tracker-liste-titel = Song
tracker-muster-neu = New pattern
tracker-muster-nr = Pattern { $n }
tracker-reihenfolge = Order
tracker-reihenfolge-label = { $n ->
    [one] Order of the patterns with 1 entry; arrow keys choose an entry
   *[other] Order of the patterns with { $n } entries; arrow keys choose an entry
 }
tracker-eintrag-ansage = Entry { $i }: pattern { $p }
tracker-schleife = The repeat starts here
tracker-schleife-ab = Repeat from entry
tracker-anhaengen = Append
tracker-setzen = Set
tracker-entfernen = Remove
tracker-nach-oben = Move entry up
tracker-nach-unten = Move entry down
tracker-muster-dup = Duplicate
tracker-muster-loeschen = Delete
tracker-play-hinweis = PLAY files have one pattern and no order.
tracker-instrumente = Instruments
tracker-instrumente-label = { $n ->
    [one] Instruments with 1 entry; arrow keys choose an instrument
   *[other] Instruments with { $n } entries; arrow keys choose an instrument
 }
tracker-instrument-ansage = Instrument { $i }: { $name }
tracker-instrument-neu = New instrument
tracker-instrument-loeschen-hinweis = Only the last, unused instrument can be deleted
tracker-raster-label = Edit pattern
tracker-werkzeuge = Tracker tools
tracker-tempo = Tempo
tracker-zeilen = Rows
tracker-oktave = Octave
tracker-schritt = Step
tracker-halbton-hoch = Semitone ↑
tracker-halbton-runter = Semitone ↓
tracker-instrument-anwenden = Apply instrument
tracker-muster-wahl = Choose pattern
tracker-stimme = Voice { $n }
tracker-raster-liste = Grid of the pattern ({ $r } rows, { $c } voices); arrow keys move, piano keys enter notes
tracker-zelle-ansage = Row { $r }, voice { $c }: { $note }, instrument { $ins }
tracker-reihe = Row { $r }
tracker-kanal = Voice { $c }
tracker-tasten-hinweis = Z–M, Q–I: notes · 1: note end · Del: clear · , .: octave · [ ]: instrument
tracker-instrument-titel = Instrument
tracker-instrument-titel-n = Instrument { $n } · { $name }
tracker-instrument-name = Name of the instrument
tracker-klangform = Waveform
tracker-welle-dreieck = Triangle
tracker-welle-saege = Saw
tracker-welle-rechteck = Pulse
tracker-welle-rauschen = Noise
tracker-anschlag = Attack
tracker-abfall = Decay
tracker-halten = Sustain
tracker-ausklang = Release
tracker-pulsbreite = Pulse width
tracker-verzerrung = Distortion
tracker-dist-rein = Pure tone
tracker-dist-bass = Bass
tracker-dist-buzz = Buzz
tracker-dist-gritty = Gritty
tracker-huellkurve = Envelope per frame
tracker-huellkurve-label = Volume of the 16 frames of the envelope
tracker-env-bild = Frame { $k }: volume { $w }
tracker-env-schleife = Envelope repeats from frame
tracker-lautstaerke = Volume
tracker-frage-titel = Unsaved song
err-song-unlesbar = The song "{ $rel }" cannot be opened: { $detail }
err-song-zwischenablage = Nothing to paste: copy cells first.
err-song-instrument-loeschen = Only the last instrument can be deleted, and only if no pattern uses it.
err-audio-fehler = Playback not possible: { $detail }
tracker-abspielen-song = Play song (Space)
tracker-abspielen-muster = Play pattern (Shift+Space)
tracker-muster-kurz = Pattern ▶
tracker-stopp = Stop playback
tracker-mitscrollen = Follow playback
tracker-stumm = Mute voice { $n }
tracker-solo = Solo voice { $n }
tracker-spielt = Playing: entry { $i }, row { $r }
err-song-import-fehler = { $detail }
tracker-quelle-filter = Songs (SID, GoatTracker, VintageCoder)
tracker-konvertieren = Convert
tracker-konvertieren-titel = Convert a song from another system
tracker-rmt-import = Import RMT module
tracker-import-name = Name of the new file
tracker-import-ziel = Target system
tracker-import-subtune = Subtune (1 to { $n })
tracker-import-sekunden = Recording length in seconds
tracker-import-laeuft = Working …
tracker-stimmen = Voices
tracker-stimmen-sichtbar = Visible voices
tracker-alle-einblenden = Show all
tracker-klaviatur = Keyboard
tracker-klaviatur-label = Keyboard from octave { $o }
tracker-taste-ansage = Key { $note }
tracker-klaviatur-hinweis = A click previews the note with the selected instrument but does not enter anything. The label is the computer key.
err-debug-kein-bau = Build the program first to debug it.
err-debug-fehler = Debugging is not possible: { $detail }
err-debug-testlauf = No debug session is started in a test run.
err-debug-nur-usb = { $name }: debugging works over USB only, not over Ethernet.
err-debug-keine-sitzung = No debug session is running.
err-debug-keine-erkennung = Detect the snapshot first.
err-debug-zeile-ohne-code = Line { $zeile } has no code: cannot run to it.
dbgui-adresse = Address ($D020 or symbol)
dbgui-angehalten = PAUSED
dbgui-art = Kind of point
dbgui-art-access = Access
dbgui-art-exec = Execute
dbgui-art-read = Read
dbgui-art-write = Write
dbgui-bau-fehlt = The last build failed. Fix the errors first.
dbgui-bedingung = Condition (e.g. a == 5)
dbgui-beendet = Debug session ended.
dbgui-beob-eingabe = Expression, address or symbol
dbgui-beob-weg = Remove watch
dbgui-beobachten = Watch
dbgui-bild-aktualisieren = Refresh picture
dbgui-bild-alt = Emulator picture
dbgui-bild-auto = Refresh at every stop
dbgui-bild-leer = No picture yet. “Refresh picture” fetches one.
dbgui-bild-live = Live (slow motion)
dbgui-bild-nicht = This emulator does not provide a picture.
dbgui-bild-quelle = Picture source
dbgui-bildschirm = Screen
dbgui-bildschirm-bei = screen at
dbgui-disasm = Disassembly
dbgui-editor = Editor
dbgui-ende = End (for a range)
dbgui-erkennen = Detect what is in memory
dbgui-flags = Flags
dbgui-haltepunkt-bei = Breakpoint at ${ $a }
dbgui-haltepunkte = Breakpoints
dbgui-hierher-hinweis = Put the cursor in a source file, then “Run to here”.
dbgui-hinzufuegen = Add
dbgui-ignorieren = Ignore
dbgui-keine-beobachtungen = Nothing watched. An expression shows its value here while the program is stopped.
dbgui-keine-punkte = No breakpoints. A click left of a line sets one.
dbgui-laeuft-hinweis = The program is running. Stopping shows registers, disassembly and memory.
dbgui-leer-hinweis = Set breakpoints in the editor margin (or F9) and in the disassembly.
dbgui-leer-text = Builds the program and starts it in the emulator with a debug session: breakpoints, single step, registers, memory.
dbgui-leer-titel = No debug session
dbgui-links = Breakpoints, watch, snapshot
dbgui-mehrfarbig = multicolor
dbgui-mem-eingabe = Address or symbol
dbgui-mem-los = Show
dbgui-mitte = Control, disassembly and memory
dbgui-moment = Snapshot
dbgui-moment-halt = A snapshot works only while the program is stopped.
dbgui-moment-kacheln = Build tiles from the screen
dbgui-moment-name = Name of the snapshot
dbgui-moment-rom = Fetch charset from ROM
dbgui-moment-sichern = Save as files
dbgui-moment-sprites = Include sprites
dbgui-punkt-adresse = That is not an address. Examples: $D020, 0x0400, 2049.
dbgui-punkt-loeschen = Delete point
dbgui-rechts = Registers, stack and picture
dbgui-register = Registers
dbgui-speicher = Memory from
dbgui-sprites-n = { $n ->
    [one] 1 sprite
   *[other] { $n } sprites
 }
dbgui-stapel = Stack
dbgui-stapel-leer = Stack empty
dbgui-start = Start debugging
dbgui-startet = Debug session starting …
dbgui-status-halt = Stopped at ${ $pc } { $ort }
dbgui-status-laeuft = Running
dbgui-status-startet = Starting …
dbgui-status-wartet = Waiting for the emulator
dbgui-text-eingabe = Send text to the emulator
dbgui-text-senden = Send
dbgui-treffer = Hits
dbgui-werkzeuge = Debug tools
dbgui-zeichensatz-bei = Charset at ${ $a }
dbgui-zeichensatz-rom = Charset in ROM
dbgui-zeichensatz-unbekannt = Charset unknown
hexui-16 = 16 bytes
hexui-4kb = 4 KB per page
hexui-8 = 8 bytes
hexui-adr = ADR
hexui-anfang = To the start
hexui-ansicht = Hex view
hexui-auswahl = Selection
hexui-bereich = Range
hexui-breite = Bytes per row
hexui-byte = Byte
hexui-byte-ansage = Address { $off }, value { $v }
hexui-bytes = bytes
hexui-bytes-ab = { $n } bytes from ${ $off }
hexui-datei = File
hexui-dateien = Binary files
hexui-ende = To the end
hexui-extern = Open in the default program
hexui-ganze-datei = The whole file is loaded.
hexui-gedeutet = Interpreted as
hexui-gefunden = Found at { $off }
hexui-kein-projekt = No project open.
hexui-keine-dateien = No binary files in the project.
hexui-ladeadresse = Load address ${ $a }
hexui-leer-text = Choose a file on the left. The hex view only shows files and never changes them.
hexui-leer-titel = No file chosen
hexui-liste = Hex view, { $n } bytes
hexui-naechste = Next page
hexui-nichts-gefunden = Nothing found.
hexui-nichts-gewaehlt = Click a byte or choose one with the arrow keys.
hexui-nur-erste = Showing the first { $mib } MiB of { $n } bytes.
hexui-nur-lesen = Read only
hexui-seite-label = Page selection
hexui-speicheradresse = Memory address ${ $a }
hexui-suche = Search: bytes ($A9 00) or text
hexui-text = TEXT
hexui-text-deutung = Text
hexui-vorige = Previous page
hexui-weiter-suchen = Search
hexui-wort = Word (little endian)
spk-adresse = Address
spk-alle = All
spk-art = Kind
spk-belegt = Used by the program
spk-bereiche = Areas
spk-byte = bytes
spk-ea = I/O
spk-filter = Filter
spk-frei = Free RAM
spk-hinweis = Arrow keys choose a page. The bar in a cell shows how much of it the program uses.
spk-kaestchen = Each cell = 256 bytes
spk-karte = Memory map
spk-karte-label = Memory map, 256 pages
spk-leer-text = The map comes from the last build. Build first ({ $taste }).
spk-leer-titel = Nothing built yet
spk-programm = Program
spk-programm-belegt = Used by the program
spk-programm-titel = Program
spk-pruefungen = Checks
spk-seite = Page ${ $p }
spk-seite-ansage = Page { $p }, { $kat }, { $used } bytes used
spk-seite-titel = Selected page
spk-umschaltung = Bank switching
err-einst-ungueltig = The settings are invalid: { $detail }
err-einst-schreiben = The settings could not be saved: { $detail }
err-einst-import = The settings file could not be read: { $detail }
einst-assembler = Assembler
einst-assembler-eingebaut = Built in (vc-asm). No other program needed.
einst-basic-dialekt = BASIC dialect per target system
einst-basic-hinweis = A project's BASIC follows its target system. Line numbers are optional (Project ▸ Sources).
einst-bilder-fragen = Ask
einst-bilder-immer = Always load
einst-bilder-nie = Never load
einst-darstellung-zurueck = Appearance reset.
einst-darstellung-zuruecksetzen = Reset appearance (color scheme, zoom, editor)
einst-datei = Settings file
einst-datei-zeigen = Show in folder
einst-debuglog = Write a debug log to a file
einst-debuglog-hinweis = Helps with troubleshooting. The file stays on this computer and is never sent.
einst-fehlersuche = Troubleshooting
einst-hilfe = Help
einst-hilfe-bilder = Load manual pictures online
einst-hilfe-url = Location of the manual pictures
einst-hilfe-url-leer = empty = default address
einst-include-suche = Include search: 1. the project folder, 2. extra folders (-I), 3. VintageCoder system files.
einst-kuerzel = Keyboard shortcuts
einst-kuerzel-fest = The shortcuts are fixed. This list helps you find them.
einst-kuerzel-keine = Nothing found.
einst-kuerzel-suche = Search a command or shortcut
einst-laedt = Loading …
einst-menueleiste = Menu bar in the window
einst-menueleiste-mac = On macOS the system menu bar is used.
einst-menueleiste-win = The menu bar in the window is on.
einst-pretty = Format a line when leaving it
einst-pretty-hinweis = Assembler and BASIC: the edited line is aligned as soon as you leave it.
einst-projektordner = Projects folder
einst-projektordner-hinweis = “New project” suggests this folder for new projects. Empty = default.
einst-schema = Color scheme
einst-schema-dunkel = Dark
einst-schema-hell = Light
einst-schema-kontrast = High contrast
einst-schema-system = System
einst-schrift = Editor font size
einst-schrift-groesser = Larger font
einst-schrift-kleiner = Smaller font
einst-seite-allgemein = General
einst-seite-aussehen = Appearance
einst-seite-editor = Editor
einst-seite-kuerzel = Shortcuts
einst-seite-sicherung = Backup
einst-seite-toolchain = Toolchain
einst-seiten = Settings pages
einst-speichern = Save
einst-sprache = Language
einst-sprache-auto = Automatic
einst-sprache-hinweis = English is machine-translated and not reviewed yet.
einst-titel = Settings
einst-umbruch = Word wrap
einst-ungespeichert = Unsaved changes
einst-upd-fragen = Not asked yet
einst-upd-hinweis = Nothing is sent without your consent. “On request” checks only when you ask.
einst-upd-nie = On request
einst-upd-taeglich = Daily
einst-upd-url = Address of the version list
einst-upd-url-leer = empty = default
einst-upd-woechentlich = Weekly
einst-updates = Updates
einst-verwerfen = Discard
einst-waehlen = Choose …
einst-zoom = Zoom
einst-zoom-groesser = Larger
einst-zoom-hinweis = Cmd/Ctrl + Plus, Minus and 0 set the zoom.
einst-zoom-kleiner = Smaller
konfig-exportieren = Export configuration …
konfig-exportiert = Exported to { $path } (without passwords).
konfig-hinweis = Target systems, emulators, machines, help and update choice can be saved and taken over on another computer. Machine passwords are never included.
konfig-import-frage = The file contains { $z } target systems and { $m } machines. The current settings are replaced.
konfig-importieren = Import configuration …
konfig-passwoerter-bleiben = Passwords of machines you already have are kept.
konfig-reset-frage = All settings go back to the default, including target systems, emulators, machines and their passwords. This cannot be undone.
konfig-titel = Backup
konfig-uebernehmen = Apply
konfig-uebernommen = Applied from { $path }.
konfig-zurueckgesetzt = Settings reset to the default.
konfig-zuruecksetzen = Reset to default …
einst-seite-ziele = Targets and hardware
einst-seite-roms = ROMs
roms-byte = bytes
roms-dateien = { $n ->
    [one] 1 file
   *[other] { $n } files
 }
roms-erwartet = Expected size in bytes: { $n }
roms-fehlt-optional = Missing (optional)
roms-fehlt-ort = not present
roms-fremde = Use ROMs of other programs (VICE, Xemu, atari800)
roms-fremde-hinweis = Read only; nothing is copied or changed. Default: off.
roms-neu-pruefen = Check again
roms-ordner-oeffnen = Open ROM folder
roms-orte = Places searched
roms-systeme = Systems
roms-woher = VintageCoder ships no ROMs. Put your ROM files into the ROM folder; they are recognized by checksum.
ziele-art = Kind of emulator
ziele-baud = Baud rate (empty = 2000000)
ziele-emu-hinzu = Add emulator …
ziele-emulatoren = Emulators
ziele-entfernen = Remove
ziele-eth-ungeprueft = The Ethernet path is only tested against stand-ins, not on real hardware.
ziele-ethernet = Ethernet
ziele-gefunden-auto = Found: { $p }
ziele-geraet = Serial port
ziele-hardware = Real hardware
ziele-hinzu = Add
ziele-host = Address (192.168.1.50 or host:port)
ziele-hw-hinzu = Connect { $name }
ziele-kein-system = No target system chosen.
ziele-keine-emulatoren = No emulator entered yet.
ziele-name = Name
ziele-nicht-gefunden = Not found. Enter the path by hand.
ziele-passwort = Network password (empty = none)
ziele-pfad = Program (empty = search automatically)
ziele-pfad-auto = automatic
ziele-pw-verbergen = Hide
ziele-pw-zeigen = Show
ziele-reset-vor = Reset the machine before the transfer
ziele-rom = ROM (atari800: operating system, Xemu: folder with MEGA65.ROM)
ziele-run-nach = Run RUN after the transfer
ziele-schnittstelle = Network interface of the computer
ziele-sd-ordner = Folder on the SD card
ziele-standard = Default
ziele-start-emulator = Emulator (default of the system)
ziele-start-hinweis = The start target applies to F5 and the Start button of this system.
ziele-startziel = Start target
ziele-suchen = Search
ziele-system-hinzu = Add a system …
ziele-system-weg = Remove system
ziele-systeme = Target systems
ziele-testen = Test
ziele-testet = Testing …
ziele-usb = USB
ziele-verbindung = Connection
ziele-verbindung-testen = Test connection
einst-seite-info = Info and update
info-aenderungen = View changes
info-aktuell = You have the latest version.
info-betreff = Subject
info-drittanbieter = Third parties
info-feedback = Feedback
info-feedback-hinweis = The feedback is prepared as an email. It is only sent in your mail program; the text contains technical details only, no files and no names.
info-feedback-titel = Send feedback
info-format-warnung = Note: the new version writes a higher project format. Older versions can no longer read the projects afterwards.
info-gekuerzt = The text was shortened for the mail link; “Copy” takes the full text.
info-hinweise-auf = Show notices
info-hinweise-zu = Hide notices
info-jetzt-pruefen = Check for updates now
info-kein-netz = This page never fetches anything from the network on its own.
info-kopieren = To the clipboard
info-kopieren-fehlt = The clipboard is not available here. Select the text and copy it by hand.
info-kopiert = Copied to the clipboard.
info-letzte-pruefung = Last check { $t }: { $r }
info-mail-geoeffnet = Mail program opened. It is only sent there.
info-mail-oeffnen = Prepare mail
info-neue-version = New version { $v } available
info-nie-geprueft = Never checked for updates.
info-prueft = Checking …
info-quelltext = Source code
info-spaeter = Remind me later
info-text = Text
info-ueber = About VintageCoder
info-ueberspringen = Skip this version
info-update-hinweis = Update { $v } available
info-update-hinweis-text = Only the version list is fetched; nothing is downloaded or installed. You set the frequency under General.
info-webseite = Website
info-zurueckgezogen = The installed version was withdrawn. Please update.
err-wiz-fehler = The assistant reports: { $detail }
err-wiz-kein-ordner = There is no folder for the settings.
err-wiz-keine-sitzung = The assistant is not open.
wiz-abbrechen = Cancel
wiz-aenderungen-text = This will be changed. Untick to leave an item out. A backup of your settings is made before anything is written.
wiz-beispiele = Examples
wiz-beispiele-fehlen = This installation contains no examples.
wiz-beispiele-kopieren = Copy example projects of the chosen systems
wiz-beispiele-kopiert = { $n ->
    [one] 1 example copied.
   *[other] { $n } examples copied.
 }
wiz-beispiele-ziel = Destination: { $p }
wiz-einrichtung = Setup
wiz-einrichtung-text = The assistant sets up target systems, emulators, ROMs and hardware. It installs nothing and downloads nothing.
wiz-emulatoren-text = Emulators are searched for; enter the path by hand if one is missing.
wiz-fertig-spaeter = You can change everything later under Settings or via Help ▸ Assistant.
wiz-fertig-text = Setup is complete.
wiz-hardware-text = Real machines (Ultimate, MEGA65) are optional. Connections can be tested here.
wiz-hinweise = Notes
wiz-keine-installation = The assistant installs nothing and downloads nothing. You can cancel at any time; nothing is written until the end.
wiz-nichts-zu-aendern = Nothing to change. Everything is already set.
wiz-rueckgaengig = Undo
wiz-schliessen = Close
wiz-schnell = Quick setup (7 steps)
wiz-schritte = Assistant steps
wiz-sicherung = Backup of the old settings: { $p }
wiz-spaeter = Continue later
wiz-systeme-text = Which systems do you want to program for? Systems you untick disappear from the interface; their entries are kept.
wiz-teil-design = Appearance and editor …
wiz-teil-emulatoren = Emulators …
wiz-teil-hardware = Hardware …
wiz-teil-roms = ROMs …
wiz-titel = Setup
wiz-uebernehmen = Apply
wiz-uebernommen = Applied.
wiz-unvollstaendig = Setup is not finished yet.
wiz-voll = Set everything myself (12 steps)
wiz-weiter = Next
wiz-willkommen-text = Welcome to VintageCoder. Quick setup asks only what is needed; if you want to set everything yourself, take the long way.
wiz-zurueck = Back
view-title-hilfe = Help
hilfe-bilder-einmal = Just this time
hilfe-bilder-frage = Load manual pictures
hilfe-bilder-immer = Always load
hilfe-bilder-nie = Never load
hilfe-bilder-text = The manual's pictures live online. Load them? Only the picture is requested; nothing of yours is sent.
hilfe-kein-eintrag = There is no help entry for “{ $w }”.
hilfe-nichts-gefunden = Nothing found.
hilfe-suche = Search help: topics, commands, directives, registers …
hilfe-themen = Help topics
hilfe-willkommen = Help and manual
hilfe-willkommen-text = Choose a topic on the left or search. F1 in the editor looks up the word under the cursor.
hilfe-zurueck = Back
startui-alle = All
startui-beispiel-sub = Playable games and demos
startui-beispiele-sub = runnable on real hardware and in the emulator
startui-frei-verwendbar = Files created with VintageCoder and the VCASM syntax may be used freely.
startui-n-projekte = { $n ->
    [one] 1 project
   *[other] { $n } projects
 }
startui-neues-sub = From a template for a target system
startui-oeffnen-sub = Choose a folder or project file
startui-system-filter = Filter examples by system
einst-assembler-optionen = Assembler options when building
ziele-args-titel = Start parameters
args-an = on
args-aus = off
args-auf-standard = Reset to default
args-beispiel = Example: { $x }
args-bereich = Range: { $a } to { $b }
args-braucht = Needs: { $x }
args-katalog = Catalog: { $l }. Parameters you leave unset keep the program's default.
args-kein-katalog = There is no parameter catalog for this kind; the free field is enough.
args-migrierbar = { $n } parameters in the free field are known to the catalog.
args-n-gesetzt = { $n } set
args-nicht-geprueft = Note: this catalog has not been checked against the program.
args-nicht-zusammen = Not together with: { $x }
args-standard = Default
args-standard-des-programms = Program default: { $d }
args-uebernehmen = Move to the selection
args-variable-einfuegen = Insert variable:
args-vorschau = Preview (example project “demo”):
args-weitere = More parameters
args-zuruecksetzen-eins = Reset
einst-startbild = Splash screen
einst-startbild-an = Show the splash screen at startup
einst-tasten = Keys
einst-einzeltasten = Single keys in the editors
einst-einzeltasten-hinweis = Letters and digits choose tools and enter notes. Off: only shortcuts with Cmd/Ctrl or Alt work.
einst-tasten-klaviatur = Show computer keys on the keyboard
einst-tastenlayout-hinweis = Keys apply by their position on the keyboard, so no keyboard layout choice is needed.
splash-text = Assembler development for 8-bit computers
startui-willkommen = Welcome to VintageCoder
startui-willkommen-text = The setup finds assemblers and emulators, picks your target systems and adds examples. It takes a few minutes and can be repeated at any time.
startui-willkommen-einrichten = Start setup
startui-willkommen-spaeter = Later
kopierdlg-system = Target system of the copy
kopierdlg-system-wie-original = Same as the original
kopierdlg-system-hinweis = Copy created for another system. The code was not converted: please adapt the sources yourself.
kopierdlg-system-hinweis-lang = The chosen system becomes the default target of the copy. The code is not converted; the sources stay unchanged. Target-dependent are, for example, zero page and memory addresses, sprite and character set formats, and the palette.
include-oeffnen = Open included file
include-im-baum-zeigen = Show in project tree
include-nicht-gefunden = File not found: {$path}
include-altpfad = The old path {$path} (build/export) no longer exists: please include the media file itself (W214).
roms-import = Import ROMs
roms-datei-waehlen = Choose ROM file or zip …
roms-fremde-suchen = Search ROMs of other programs
roms-liste-kopieren = Copy list of finds
roms-import-hinweis = Nothing is copied without confirmation. Files can also be dropped on the window. ROMs are never shipped and never downloaded.
roms-import-nichts = Nothing recognizable was found in the selection.
roms-import-nichts-fremd = No recognized ROMs were found in the known places of other programs.
roms-import-haken = Import {$name}
roms-import-kopieren = will be copied
roms-import-da = already present
roms-import-ausfuehren = Import {$n}
roms-import-fertig = {$n} ROM(s) copied to the ROM folder.
roms-fest-gespeichert = Choice saved to roms.json.
roms-liste-kopiert = List of finds copied to the clipboard.
roms-fest-label = Pinned file for {$rolle}
roms-fest-auto = Automatic
ziele-installiert = Installed: {$pfad}
ziele-laedt-und-installiert = Downloading and installing …
ziele-laedt-frage = {$name} {$version} ({$mb} MB) will be downloaded from {$host}, verified and unpacked to {$nach}. Nothing is downloaded until you click “Download now”.
ziele-jetzt-laden = Download now
ziele-portabel-installieren = Install portable ({$version}) …
err-emu-install-fehler = The installation failed: {$detail}
err-roms-fehler = The ROM choice could not be saved: {$detail}
wiz-erstes-text = A project is the quickest way to try VintageCoder.
wiz-erstes-neu = Create a new project …
wiz-erstes-beispiel = Open example: {$name}
startui-neues-erkannt = New finds
startui-neues-text = Found since the last setup: {$liste}.
startui-neues-einrichten = Set up emulators
startui-neues-merken = Remember as known
startui-neues-aus = Do not show again
projekt-gruppe-art = Kind
projekt-art-asm = Assembler
projekt-art-basic = BASIC
projekt-gruppe-dialekt = Dialect
projekt-gruppe-ziele = Targets
projekt-gruppe-entfernen = Remove group
projekt-quelle-entfernen = Remove {$f} from the sources
projekt-datei-hinzu = Add file (path in the project)
projekt-hinzufuegen = Add
projekt-gruppe-neu-asm = New group: assembler
projekt-gruppe-neu-basic = New group: BASIC
projekt-illegale = Undocumented opcodes
projekt-illegale-auto = Default (warning)
projekt-illegale-deny = Error
projekt-illegale-warn = Warning
projekt-illegale-allow = Allowed
projekt-startziel = Start target (F5)
projekt-startziel-global = As in the settings
projekt-startziel-emulator = Emulator
projekt-abbild = Disk image when building
projekt-abbild-format = Format
projekt-abbild-name = Name on the disk
projekt-abbild-weg = No image
projekt-abbild-hinweis = Output: {$datei}. The programs of the sources go onto the image.
projekt-abbild-aus = No disk image is created.
projekt-abbild-neu = Create {$f} image
dbgui-chips = Chips
dbgui-chips-keine = There is no chip description for this target system.
dbgui-test-startet = Test “{$name}” is being prepared in the debugger.
tests-debuggen = Debug this test
dbgui-test-sitzung = Test session
status-debug-pc = Debug: halted at ${$pc}
machine-debug = Debug session
machine-flags = Flags
md-keine-markdown-datei = This is not a Markdown file.
md-hier-nichts = Nothing to do here.
md-link-geoeffnet = Address opened in the browser: {$url}
md-bild-fehlt = Image not found or too large
md-bild = Image
md-aufgabe = Tick task
md-vorschau-label = Markdown preview
md-leer = The file is empty.
md-mehr = Show {$n} more blocks
md-vorschau = Preview
md-quelltext = Source
md-leiste = Markdown
md-ansicht = View
bau-abgestuerzt = The build crashed internally ({ $was }). The program keeps running; please build again.
err-extern-ausfuehrbar = { $rel } is an executable program. Opening it would run it.
rf-papierkorb-titel = Trash not available
rf-papierkorb-text = “{$name}” could not be moved to the trash and was not deleted. Delete permanently? This cannot be undone.
rf-endgueltig-loeschen = Delete permanently
rf-programm-titel = This file is a program
rf-programm-text = “{$name}” is a program. Open anyway? It would be started.
rf-trotzdem-oeffnen = Open anyway
err-fileops-papierkorb-fehlgeschlagen = “{$rel}” could not be moved to the trash. The file was not deleted.
