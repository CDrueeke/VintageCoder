# SPDX-License-Identifier: GPL-3.0-or-later
# Texte der neuen Oberfläche (ui/): Navigation, Startansicht, Befehlspalette, Komponentenbibliothek, Messung (Etappe E0)

# Kontext: Bedienhilfe-Name der oberen Navigation
nav-main-label = Hauptbereiche

# Kontext: Bedienhilfe-Name der zweiten Reihe
nav-sub-label = Unterbereiche

# Kontext: Hauptbereich
nav-main-code = Code

# Kontext: Hauptbereich
nav-main-grafik = Grafik

# Kontext: Hauptbereich
nav-main-musik = Musik

# Kontext: Hauptbereich
nav-main-daten = Daten

# Kontext: Hauptbereich
nav-main-debug = Debug

# Kontext: zweite Reihe Grafik
nav-grafik-sprites = Sprites

# Kontext: zweite Reihe Grafik
nav-grafik-zeichensatz = Zeichensatz

# Kontext: zweite Reihe Grafik
nav-grafik-kacheln = Kacheln

# Kontext: zweite Reihe Grafik
nav-grafik-bildschirm = Bildschirm

# Kontext: zweite Reihe Grafik
nav-grafik-karten = Karten

# Kontext: zweite Reihe Grafik
nav-grafik-palette = Palette

# Kontext: zweite Reihe Daten
nav-daten-disk = Disk

# Kontext: zweite Reihe Daten
nav-daten-hex = Hex

# Kontext: zweite Reihe Daten
nav-daten-speicher = Speicher

# Kontext: Titel des Bereichs
view-title-code = Code-Editor

# Kontext: Titel des Bereichs
view-title-musik = Musik-Editor

# Kontext: Titel des Bereichs
view-title-debug = Debugger

# Kontext: Titel des Bereichs
view-title-sprites = Sprite-Editor

# Kontext: Titel des Bereichs
view-title-zeichensatz = Zeichensatz-Editor

# Kontext: Titel des Bereichs
view-title-kacheln = Kachel-Editor

# Kontext: Titel des Bereichs
view-title-bildschirm = Bildschirm-Editor

# Kontext: Titel des Bereichs
view-title-karten = Karten-Editor

# Kontext: Titel des Bereichs
view-title-palette = Palette

# Kontext: Titel des Bereichs
view-title-disk = Disk-Editor

# Kontext: Titel des Bereichs
view-title-hex = Hex-Ansicht

# Kontext: Titel des Bereichs
view-title-speicher = Speicherkarte

# Kontext: Kontext: stage = Etappenkennung, z. B. E3
view-placeholder-text = Dieser Bereich folgt in Etappe { $stage } der neuen Oberfläche.

# Kontext: Platzhalter
view-placeholder-hint = Bis dahin steht er in der bisherigen Oberfläche zur Verfügung.


# Kontext: Marke in der Zuletzt-Liste

# Kontext: Bedienhilfe-Name des Dateibaums
tree-label = Dateien

# Kontext: Überschrift über dem Dateibaum
tree-heading = Projekt

# Kontext: leerer Dateibaum
tree-empty = Dieses Projekt enthält noch keine Dateien.

# Kontext: Dateibaum ohne Projekt
tree-no-project = Kein Projekt geöffnet.

# Kontext: Titelleiste ohne Projekt
titlebar-no-project = Kein Projekt

# Kontext: Knopf und Menü für die Sprache
titlebar-language = Sprache

# Kontext: Knopf für Hell und Dunkel
titlebar-theme = Farbschema

# Kontext: Sprachwahl: Systemsprache
lang-choice-auto = Automatisch

# Kontext: Sprachwahl xx
lang-choice-pseudo = Pseudo-Sprache (nur zum Prüfen)

# Kontext: Farbschema
theme-light = Hell

# Kontext: Farbschema
theme-dark = Dunkel

# Kontext: Farbschema
theme-system = System

# Kontext: Gruppe in der Befehlspalette
palette-group-commands = Befehle

# Kontext: Gruppe in der Befehlspalette
palette-group-places = Gehe zu

# Kontext: Befehlspalette ohne Treffer
palette-empty = Nichts gefunden.

# Kontext: Hinweiszeile der Befehlspalette
palette-hint = ↑ ↓ wählen, Eingabe führt aus, Esc schließt

# Kontext: Kontext: n = Zahl der Treffer (Ansage für Sprachausgabe)
palette-count = { $n ->
    [one] 1 Treffer
   *[other] { $n } Treffer
 }

# Kontext: Statuszeile rechts, neben dem Kürzel der Befehlspalette
status-keys-hint = Alle Befehle

# Kontext: Kontext: name = Befehlsname; Befehl der neuen Oberfläche noch nicht gebaut
toast-not-yet = „{ $name }“ folgt in einer späteren Etappe.

# Kontext: Knopf einer Meldung
toast-close = Meldung schließen

# Kontext: Fehlercode projekt-fehlt
err-projekt-fehlt = Der Projektordner „{ $path }“ ist nicht vorhanden.

# Kontext: Fehlercode projekt-fehler
err-projekt-fehler = Das Projekt konnte nicht gelesen werden: { $detail }

# Kontext: Fehlercode menue-fehler
err-menue-fehler = Die Menüleiste konnte nicht aufgebaut werden: { $detail }

# Kontext: Fehlercode sprache-unbekannt
err-sprache-unbekannt = Unbekannte Sprache „{ $lang }“.

# Kontext: Fehlercode befehl-unbekannt (Programmfehler)
err-befehl-unbekannt = Unbekannter Befehl „{ $cmd }“.

# Kontext: Fehlercode arg-fehlt (Programmfehler)
err-arg-fehlt = Dem Befehl fehlt die Angabe „{ $name }“.

# Kontext: Fehlercode doc-unbekannt
err-doc-unbekannt = Das Dokument { $id } ist nicht geöffnet.

# Kontext: Rückfall für Fehlercodes ohne eigenen Text
err-unbekannt = Unbekannter Fehler ({ $code }).

# Kontext: Oberfläche läuft ohne Tauri und ohne Prüfbrücke
err-keine-verbindung = Keine Verbindung zum Programmkern.

# Kontext: Knopf
dialog-ok = OK

# Kontext: Tastenfang im Ruhezustand
keycapture-idle = Taste oder Kürzel drücken

# Kontext: Tastenfang wartet
keycapture-active = Drücke jetzt die Tasten …

# Kontext: Tastenfang: Kürzel entfernen
keycapture-clear = Zurücksetzen

# Kontext: Entwicklungsansicht
gallery-title = Komponentenbibliothek

# Kontext: Entwicklungsansicht
gallery-intro = Alle Bausteine der neuen Oberfläche an einem Ort: Karte, Segment, Schalter, Schieber, Tabelle, Dialog, Menü, Tooltip, Kürzelanzeige und Tastenfang.

# Kontext: Beispielkarte
gallery-card-title = Karte

# Kontext: Beispielkarte
gallery-card-body = Eine Karte fasst zusammengehörige Bedienelemente zusammen.

# Kontext: Beispiel Segment
gallery-segment-label = Zeichenmodus

# Kontext: Beispiel Segment
gallery-segment-a = Mehrfarbe

# Kontext: Beispiel Segment
gallery-segment-b = Einfarbig

# Kontext: Beispiel Schalter
gallery-switch-label = Raster anzeigen

# Kontext: Beispiel Schieber
gallery-slider-label = Zoom

# Kontext: Beispiel Tabelle
gallery-table-label = Beispieltabelle

# Kontext: Spalte der Beispieltabelle
gallery-table-col-name = Name

# Kontext: Spalte der Beispieltabelle
gallery-table-col-size = Größe

# Kontext: Spalte der Beispieltabelle
gallery-table-col-kind = Art

# Kontext: Beispiel Dialog
gallery-dialog-open = Dialog öffnen

# Kontext: Beispiel Dialog
gallery-dialog-title = Beispieldialog

# Kontext: Beispiel Dialog
gallery-dialog-body = Der Fokus bleibt im Dialog, Esc schließt ihn und gibt den Fokus zurück.

# Kontext: Beispiel Menü
gallery-menu-open = Menü öffnen

# Kontext: Beispiel Menü
gallery-menu-a = Erster Eintrag

# Kontext: Beispiel Menü
gallery-menu-b = Zweiter Eintrag

# Kontext: Beispiel Menü
gallery-menu-c = Deaktivierter Eintrag

# Kontext: Beispiel Tooltip
gallery-tooltip-target = Mit der Maus oder dem Fokus ein Tooltip

# Kontext: Beispiel Tooltip
gallery-tooltip-text = Tooltips erscheinen auch bei Tastaturfokus.

# Kontext: Beispiel Kürzel
gallery-kbd-label = Kürzelanzeige

# Kontext: Beispiel Tastenfang
gallery-keycapture-label = Tastenfang (nach physischen Tasten)

# Kontext: Beispiel Knopf
gallery-button-primary = Hauptaktion

# Kontext: Beispiel Knopf
gallery-button-plain = Nebenaktion

# Kontext: Beispielzeile
gallery-row-asm = Quelltext

# Kontext: Beispielzeile
gallery-row-song = Musikstück

# Kontext: Beispielzeile
gallery-row-sprite = Sprite

# Kontext: Entwicklungsansicht
measure-title = Messung (Prüfpunkt G1)

# Kontext: Entwicklungsansicht
measure-intro = Misst die Rundlaufzeit der Befehle und den Weg vom Zeigerereignis bis zum gezeichneten Bild.

# Kontext: Knopf
measure-run = Messung starten

# Kontext: Zustand
measure-running = Messung läuft …

# Kontext: Spalte
measure-col-test = Messung

# Kontext: Spalte
measure-col-median = Median

# Kontext: Spalte
measure-col-max = Höchstwert

# Kontext: Spalte
measure-col-n = Anzahl

# Kontext: Einheit Millisekunden
measure-unit-ms = ms

# Kontext: Messung
measure-ping-small = Befehl mit kleiner Nutzlast

# Kontext: Messung
measure-raster-delta = Befehl mit Pixelraster-Delta

# Kontext: Messung
measure-binary = Binärantwort (64 KiB)

# Kontext: Messung
measure-pointer-local = Zeigerereignis bis Bild (vsync), nur lokaler Vorschaustrich

# Kontext: Messung
measure-pointer-bundled = Zeigerereignis bis Bild (vsync), gebündelter Vorgang mit Rust

# Kontext: Bedienhilfe-Name der Zeichenfläche der Messung
measure-canvas-label = Messfläche (24 × 21 Pixel)

# Kontext: Überschrift
measure-env = Umgebung

# Kontext: name = Sprachname (Deutsch, English, Automatisch …); Eintrag der Befehlspalette
lang-entry = Sprache: { $name }

# Kontext: Spalte
measure-col-mean = Mittelwert

# Kontext: Messung
measure-pointer-local-sync = Zeigerereignis bis Strich, nur lokal (ohne Bildwechsel)

# Kontext: Messung
measure-pointer-bundled-sync = Zeigerereignis bis Strich, gebündelt mit Rundlauf zu Rust (ohne Bildwechsel)

# Kontext: Fehlercode kein-projekt
err-kein-projekt = Es ist kein Projekt geöffnet.

# Kontext: Fehlercode datei-fehler (Text kommt fertig aus der Dateiverwaltung)
err-datei-fehler = { $detail }

# Kontext: Fehlercode datei-kein-utf8
err-datei-kein-utf8 = „{ $rel }“ ist keine UTF-8-Textdatei und kann hier nicht als Text geöffnet werden.

# Kontext: Fehlercode pfad-ungueltig
err-pfad-ungueltig = Ungültiger Pfad: { $detail }

# Kontext: Beschriftung des Eingabefelds im Code-Editor
editor-aria = Quelltext „{ $name }“

# Kontext: Meldung beim Speichern
editor-eol-vereinheitlicht = „{ $name }“ hatte gemischte Zeilenenden; sie wurden beim Speichern auf Unix-Zeilenenden vereinheitlicht.

# Kontext: Hinweis, wenn kein Reiter offen ist
editor-leer = Wähle links eine Datei aus dem Projekt, um sie zu bearbeiten.

# Kontext: Statuszeile: Cursorposition
status-cursor = Zeile { $line }, Spalte { $col }

# Kontext: Statuszeile: Anzahl markierter Zeichen
status-selected = { $n } markiert

# Kontext: Beschriftung der Reiterleiste
tabs-label = Offene Dateien

# Kontext: Punkt am Reiter
tab-ungespeichert = Ungespeicherte Änderungen

# Kontext: Knopf am Reiter
tab-schliessen = „{ $name }“ schließen

# Kontext: Dialogtitel beim Schließen eines Reiters
close-title = Ungespeicherte Änderungen

# Kontext: Dialogtext
close-frage = „{ $name }“ hat ungespeicherte Änderungen. Speichern, bevor der Reiter geschlossen wird?

# Kontext: Knopf im Dialog Reiter schließen
close-speichern = Speichern

# Kontext: Knopf im Dialog Reiter schließen
close-verwerfen = Verwerfen

# Kontext: Knopf in Dialogen
dialog-abbrechen = Abbrechen

# Kontext: Knopf im Dialog Name eingeben
dialog-anlegen = Anlegen

# Kontext: Knopf im Dialog Name eingeben
dialog-umbenennen = Umbenennen

# Kontext: Knopf im Dialog Löschen
dialog-loeschen = Löschen

# Kontext: Knopf, Menü und Dialogtitel im Projektbaum
tree-neue-datei = Neue Datei

# Kontext: Knopf, Menü und Dialogtitel im Projektbaum
tree-neuer-ordner = Neuer Ordner

# Kontext: Beschriftung des Namensfelds
tree-name-label = Name (Pfad relativ zum gewählten Ordner)

# Kontext: Menü und Dialogtitel im Projektbaum
tree-umbenennen = Umbenennen

# Kontext: Menü und Dialogtitel im Projektbaum
tree-loeschen = Löschen

# Kontext: Nachfrage beim Löschen
tree-loeschen-frage = „{ $name }“ in den Papierkorb legen?

# Kontext: Menü im Projektbaum
tree-oeffnen = Öffnen

# Kontext: Beschriftung des Kontextmenüs im Projektbaum
tree-menu = Aktionen für die Auswahl

# Kontext: Fehler im Namensfeld
tree-name-leer = Der Name darf nicht leer sein.

# Kontext: Fehler im Namensfeld
tree-name-zeichen = Der Name enthält ein unzulässiges Zeichen (\ : * ? " < > |).

# Kontext: Meldung nach dem Löschen
tree-geloescht-papierkorb = „{ $name }“ liegt jetzt im Papierkorb.

# Kontext: Meldung nach dem Löschen
tree-geloescht-endgueltig = „{ $name }“ wurde endgültig gelöscht (kein Papierkorb verfügbar).

# Kontext: Suchleiste im Editor
cm-find = Suchen

# Kontext: Suchleiste im Editor
cm-replace = Ersetzen durch

# Kontext: Suchleiste im Editor
cm-next = Weiter

# Kontext: Suchleiste im Editor
cm-previous = Zurück

# Kontext: Suchleiste im Editor
cm-all = Alle

# Kontext: Suchleiste im Editor
cm-match-case = Groß/Klein

# Kontext: Suchleiste im Editor
cm-regexp = Regulärer Ausdruck

# Kontext: Suchleiste im Editor
cm-by-word = Ganzes Wort

# Kontext: Suchleiste im Editor
cm-replace-one = Ersetzen

# Kontext: Suchleiste im Editor
cm-replace-all = Alle ersetzen

# Kontext: Suchleiste im Editor
cm-close = Schließen

# Kontext: Suchleiste im Editor
cm-current-match = aktueller Treffer

# Kontext: Suchleiste im Editor
cm-on-line = in Zeile

# Kontext: Meldung der Suchleiste
cm-replaced-n = $ Treffer ersetzt

# Kontext: Meldung der Suchleiste
cm-replaced-one = Treffer in Zeile $ ersetzt

# Kontext: Leiste Gehe zu Zeile
cm-goto-line = Gehe zu Zeile

# Kontext: Leiste Gehe zu Zeile
cm-go = Los

# Kontext: Meldung beim Falten
cm-folded-lines = Zeilen gefaltet

# Kontext: Meldung beim Falten
cm-unfolded-lines = Zeilen entfaltet

# Kontext: Meldung beim Falten
cm-to = bis

# Kontext: Faltung
cm-folded-code = gefalteter Code

# Kontext: Faltung
cm-unfold = Entfalten

# Kontext: Faltung
cm-fold-line = Zeile falten

# Kontext: Faltung
cm-unfold-line = Zeile entfalten

# Kontext: Meldung des Editors
cm-selection-deleted = Auswahl gelöscht

# Kontext: Meldung des Editors
cm-control-char = Steuerzeichen

# Kontext: Dialogtitel
editor-goto-title = Gehe zu Zeile

# Kontext: Beschriftung im Dialog Gehe zu Zeile
editor-goto-label = Zeilennummer (1 bis { $max })

# Kontext: Knopf im Dialog Gehe zu Zeile
editor-goto-ok = Gehe zu

# Kontext: Fehler im Dialog Gehe zu Zeile
editor-goto-invalid = Bitte eine Zahl von 1 bis { $max } eingeben.

# Kontext: Dialogtitel beim Wechseln oder Beenden
leave-title = Ungespeicherte Änderungen

# Kontext: Dialogtext beim Projektwechsel oder Beenden

# Kontext: Knopf im Dialog Projektwechsel oder Beenden
leave-alle-speichern = Alle speichern

# Kontext: Fehlercode datei-nichttext
err-datei-nichttext = „{ $rel }“ ist keine Textdatei.

# Kontext: Fehlercode ordner-in-sich
err-ordner-in-sich = Ein Ordner lässt sich nicht in sich selbst verschieben.

# Kontext: Fehlercode nichts-zu-verschieben
err-nichts-zu-verschieben = „{ $rel }“: Es gibt nichts zu verschieben.

# Kontext: Fehlercode verschieben-braucht-projektdatei
err-verschieben-braucht-projektdatei = Verschieben braucht eine Projektdatei (.vcprj).

# Kontext: Fehlercode verschieben-fehlgeschlagen
err-verschieben-fehlgeschlagen = Verschieben fehlgeschlagen: { $detail }

# Kontext: Fehlercode nichts-rueckgaengig
err-nichts-rueckgaengig = Es gibt nichts rückgängig zu machen.

# Kontext: Fehlercode rueckgaengig-fehlgeschlagen (Text aus der Fachschicht)
err-rueckgaengig-fehlgeschlagen = { $detail }

# Kontext: Fehlercode extern-fehlgeschlagen
err-extern-fehlgeschlagen = Das Programm konnte nicht gestartet werden: { $detail }

# Kontext: Meldung beim Verschieben im Baum
tree-verschieben-erst-speichern = Verschieben: erst alle geänderten Dateien speichern.

# Kontext: Meldung nach dem Verschieben
tree-verschoben = { $n ->
    [one] „{ $name }“ verschoben (1 Pfad in Quelltexten angepasst).
   *[other] „{ $name }“ verschoben ({ $n } Pfade in Quelltexten angepasst).
 }

# Kontext: Meldung nach Rückgängig im Baum
tree-rueckgaengig-gemacht = { $n ->
    [one] Verschieben rückgängig gemacht (1 Datei).
   *[other] Verschieben rückgängig gemacht ({ $n } Dateien).
 }

# Kontext: Menü und Knopf im Projektbaum
tree-alle-zuklappen = Alle zuklappen

# Kontext: Menü und Knopf im Projektbaum
tree-neu-einlesen = Neu einlesen

# Kontext: Menü im Projektbaum und am Reiter
tree-als-include = Als Include einfügen
tree-pfad-kopieren = Pfad kopieren

# Kontext: Menü im Projektbaum und am Reiter
tree-im-ordner-zeigen = Im Ordner zeigen

# Kontext: Meldung nach Pfad kopieren
tree-pfad-kopiert = Pfad kopiert: { $path }

# Kontext: Beschriftung des Filterfelds im Projektbaum
tree-filter-label = Dateien filtern

# Kontext: Hinweis im gefilterten Projektbaum
tree-filter-keine = Keine Treffer.

# Kontext: Menü im Projektbaum
tree-rueckgaengig = Verschieben rückgängig

# Kontext: Nachfrage beim Löschen ohne Papierkorb
tree-loeschen-endgueltig-frage = „{ $name }“ endgültig löschen? Auf diesem System gibt es keinen Papierkorb.

# Kontext: Menü und Dialogtitel im Projektbaum
tree-neue-notiz = Neue Notiz

# Kontext: Beschriftung der Auswahl im Dialog Neue Datei
tree-art-label = Art

# Kontext: Auswahl im Dialog Neue Datei
tree-art-asm = Assembler (.asm)

# Kontext: Auswahl im Dialog Neue Datei
tree-art-basic = BASIC (.bas)

# Kontext: Auswahl im Dialog Neue Datei
tree-art-text = Text (.txt)

# Kontext: Auswahl im Dialog Neue Datei
tree-art-notiz = Notiz (.md)

# Kontext: Meldung bei Änderung von außen
editor-extern-neu = „{ $name }“ wurde außerhalb geändert und neu eingelesen.

# Kontext: Meldung bei Änderung von außen
editor-extern-konflikt = „{ $name }“ wurde außerhalb geändert, hier gibt es ungespeicherte Änderungen. Beim Speichern wird die Datei überschrieben.

# Kontext: Meldung bei Änderung von außen
editor-extern-geloescht = „{ $name }“ wurde außerhalb gelöscht.

# Kontext: Name eines neuen, noch nicht gespeicherten Reiters
editor-unbenannt = Unbenannt { $n }

# Kontext: Name einer neuen, noch nicht gespeicherten Notiz
editor-notiz = Notiz { $n }

# Kontext: Dialogtitel für unbenannte Reiter
editor-speichern-unter = Speichern unter

# Kontext: Titel des Dateidialogs
editor-datei-oeffnen = Datei öffnen

# Kontext: Meldung beim Öffnen einer Datei außerhalb
editor-draussen = „{ $path }“ liegt außerhalb des Projekts.

# Kontext: Meldung beim Öffnen von Nicht-Text
editor-nichttext-extern = „{ $name }“ ist keine Textdatei und wird im Standardprogramm geöffnet.

# Kontext: Menü am Reiter
tab-andere-schliessen = Andere schließen

# Kontext: Menü am Reiter
tab-alle-schliessen = Alle schließen

# Kontext: Menü am Reiter
tab-nach-links = Nach links

# Kontext: Menü am Reiter
tab-nach-rechts = Nach rechts

# Kontext: Statuszeile: Art des Reiters
status-lang-asm = Assembler

# Kontext: Statuszeile: Art des Reiters
status-lang-basic = BASIC

# Kontext: Statuszeile: Art des Reiters
status-lang-plain = Text

# Kontext: Statuszeile: Art des Reiters
status-lang-markdown = Markdown

# Kontext: Gruppenüberschrift in der Befehlspalette
palette-dateien = Dateien

# Kontext: Fehlercode intern (Panik in einem Befehl)
err-intern = Interner Fehler im Befehl „{ $cmd }“. Details stehen im Panikprotokoll (vintagecoder-panik.log im Temp-Ordner).

# Kontext: Schweregrad einer Meldung der Sprachdienste (Karte beim Daraufzeigen, Liste Probleme)
diag-sev-1 = Fehler
diag-sev-2 = Warnung
diag-sev-3 = Hinweis
diag-sev-4 = Tipp

# Kontext: Meldungen der Sprachdienste im Editor
lsp-keine-dienste = Für diesen Reiter gibt es keine Sprachdienste.
lsp-keine-korrektur = Für diese Zeile gibt es keine Korrektur.
lsp-keine-definition = Keine Definition gefunden.

# Kontext: Liste Probleme unter dem Editor und Zähler in der Statuszeile
problems-none = Keine Probleme.
problems-line = Zeile { $line }
status-problems = { $e ->
    [0] { $w ->
        [0] Keine Probleme
        [one] 1 Warnung
       *[other] { $w } Warnungen
    }
    [one] { $w ->
        [0] 1 Fehler
        [one] 1 Fehler, 1 Warnung
       *[other] 1 Fehler, { $w } Warnungen
    }
   *[other] { $w ->
        [0] { $e } Fehler
        [one] { $e } Fehler, 1 Warnung
       *[other] { $e } Fehler, { $w } Warnungen
    }
}

# Kontext: Meldungen nach dem Formatieren (Prettyprint) im Code-Editor
format-fertig = Formatiert.
format-nichts = Schon formatiert, nichts zu ändern.
format-sortieren-auswahl = Zum Sortieren mehrere Zeilen markieren.
format-nicht-moeglich = Diesen Reiter kann man nicht formatieren (nur Assembler und BASIC).

# Kontext: Ablage unter dem Editor (Reiter Probleme, Suche, Gliederung)
dock-label = Ablage unter dem Editor
dock-problems = Probleme
dock-search = Suche
dock-outline = Gliederung
dock-close = Ablage schließen

# Kontext: Suche im Projekt (Ablage unter dem Editor)
search-label = Suche im Projekt
search-placeholder = In allen Dateien suchen …
search-abbrechen = Abbrechen
search-laeuft = { $n ->
    [one] Suche läuft … 1 Treffer
   *[other] Suche läuft … { $n } Treffer
 }
search-abgebrochen = { $n ->
    [one] Abgebrochen nach 1 Treffer
   *[other] Abgebrochen nach { $n } Treffern
 }
search-gekuerzt = Mehr als { $n } Treffer, die Anzeige ist gekürzt
search-nichts = Keine Treffer
search-fertig = { $n ->
    [one] 1 Treffer
   *[other] { $n } Treffer
} in { $files ->
    [one] 1 Datei
   *[other] { $files } Dateien
}

# Kontext: Gliederung der Datei (Ablage unter dem Editor)
outline-leer = Keine Marken, Konstanten oder Makros in dieser Datei.
outline-label = Marke
outline-const = Konstante
outline-macro = Makro
# Kontext: Hinweis beim Umbenennen im Projektbaum
tree-umbenennen-erst-speichern = Umbenennen: erst alle geänderten Dateien speichern.
# Kontext: Meldung nach dem Umbenennen, wenn Include-Pfade in Quelltexten angepasst wurden
tree-includes-angepasst = { $n ->
    [one] 1 Include-Pfad in Quelltexten angepasst.
   *[other] { $n } Include-Pfade in Quelltexten angepasst.
 }

# Kontext: Beschriftung des Kontextmenüs im Code-Editor
editor-kontextmenu = Editor-Menü

# Kontext: Art-Auswahl im Dialog „Neue Datei“ und „Speichern unter“: keine Endung anhängen
tree-art-keine = Ohne Endung

# Kontext: Statuszeile: Zähler der Hinweise (Schweregrad 3) der Sprachdienste
status-hinweise = { $h ->
    [one] 1 Hinweis
   *[other] { $h } Hinweise
}

# Kontext: Statuszeile: weitere Meldungen (Fehler, Warnungen, Hinweise) in anderen Dateien des Projekts als die im Reiter
status-andere-dateien = { $n ->
    [one] 1 Meldung in anderen Dateien
   *[other] { $n } Meldungen in anderen Dateien
}

# Kontext: Probleme-Liste: Kurzzeile für das ganze Projekt
problems-projekt = Projekt: { $n } Meldungen in { $m ->
    [one] 1 Datei
   *[other] { $m } Dateien
}, davon { $e } Fehler in { $f ->
    [one] 1 Datei
   *[other] { $f } Dateien
}

# Kontext: Bauen und Starten (Kopfzeile, Ausgabe, Meldungen)
bau-erst-speichern = Bauen: erst alle geänderten Dateien speichern.
bau-log-bauen = Bauen: { $rel } …
bau-log-spruenge = { $n } zu weite Sprünge automatisch verlängert (`.branchlong auto`).
bau-log-fehler = Fehler: { $e }
bau-log-disk = Diskette { $file } ({ $size } Bytes): { $programs }
bau-log-disk-fehler = Diskette: { $e }
bau-log-fertig = Fertig: { $size } Bytes in { $artifact }
bau-fehlgeschlagen = { $n ->
    [one] Bauen fehlgeschlagen (1 Fehler). Siehe Probleme.
   *[other] Bauen fehlgeschlagen ({ $n } Fehler). Siehe Probleme.
 }
run-log-gestartet = Gestartet: { $line }
run-log-fehler = Start fehlgeschlagen: { $e }
run-log-beendet = Programm beendet.
run-fehlgeschlagen = Starten fehlgeschlagen: { $e }
run-beendet = Emulator beendet.
run-nichts-laeuft = Es läuft nichts.
err-bau-laeuft = Es wird schon gebaut.
err-nichts-zu-starten = Nichts zu starten: erst bauen.
err-keine-quelle = Keine Quelldatei zum Bauen: im Projekt gibt es keine .asm- oder BASIC-Datei (oder die aktive Datei ist keine).
err-daten-fehlen = Der Datenordner mit den Zieltabellen wurde nicht gefunden.
bar-bauen = Bauen
bar-starten = Starten
bar-stopp = Beenden
bar-baut = baut …
bar-gebaut = gebaut: { $size } Bytes
bar-fehler = { $n ->
    [one] Bau: 1 Fehler
   *[other] Bau: { $n } Fehler
 }
bar-laeuft = läuft
bar-ziel = Startziel
dock-output = Ausgabe
output-leer = Noch keine Ausgabe. Bauen mit F7.
output-kopieren = Kopieren
output-leeren = Leeren
output-kopiert = Ausgabe kopiert.
problems-bau = Bauergebnis

# Kontext: Statuszeile: Speicherstreifen des gebauten Programms
bar-speicher = Belegter Speicher: { $segs }

# Kontext: Probleme: Meldungen nach dem Kopieren und Beschriftung des Kontextmenüs
problems-kopiert = In die Zwischenablage kopiert.
problems-keine-wahl = Kein Problem gewählt.
problems-title-menu = Problem

# Kontext: Maschine-Panel (rechts): Zustand und Bauergebnis
machine-title = Maschine
machine-system = System
machine-ziel = Startziel
machine-zustand = Zustand
machine-laeuft = läuft
machine-bereit = bereit
machine-bau = Letzter Bau
machine-quelle = Quelle
machine-datei = Programm
machine-groesse = Größe
machine-bytes = { $n } Bytes
machine-start = Start (SYS)
machine-segment = Nr
machine-von = von
machine-bis = bis
machine-laenge = Länge
machine-kein-bau = Noch nicht gebaut. Bauen mit F7.
machine-spaeter = Während einer Debug-Sitzung zeigt dieses Panel Register, Flags und das Bild der Maschine.

# Kontext: Tests (Ablage unter dem Editor)
dock-tests = Tests
tests-alle = Alle ausführen
tests-fehlgeschlagene = Fehlgeschlagene
tests-abbrechen = Abbrechen
tests-ziel = Ziel
tests-nur-statisch = Nur statische Prüfungen
tests-filter = Tests filtern …
tests-nur-fehler = nur Fehler
tests-laeuft = Läuft …
tests-stand = { $p } von { $n } bestanden
tests-keine = Noch kein Lauf oder keine Tests (`*.vctest`) im Projekt.
tests-state-passed = bestanden
tests-state-failed = fehlgeschlagen
tests-state-timeout = Zeit überschritten
tests-state-skipped = übersprungen
tests-hinweis = Hinweis
tests-gemessen = Gemessen
tests-quelle = Zur Quelle
tests-testdatei = Testdatei öffnen
tests-waehlen = Einen Test wählen, um Erwartung und Register zu sehen.

# Kontext: Probleme: zusammengefasste Warnungen zu weiter Sprünge (W203)
problems-spruenge = { $n } zu weite Sprünge automatisch verlängert (`.branchlong auto`)
problems-aufklappen = einzeln anzeigen

# Kontext: Ende des Emulators (vom Nutzer beendet oder abgestürzt)
run-log-ende = Emulator beendet.
run-log-ende-code = Emulator beendet (Rückgabewert { $code }).
run-ende-code = Der Emulator hat sich mit Rückgabewert { $code } beendet. Siehe Ausgabe.

# Kontext: Beenden mit Liste und Wiederherstellen unbenannter Reiter (E1.4)
leave-frage = { $n ->
    [one] 1 Reiter mit ungespeicherten Änderungen. Vor dem Fortfahren speichern?
   *[other] { $n } Reiter mit ungespeicherten Änderungen. Vor dem Fortfahren speichern?
 }
leave-unbenannt = (unbenannt, geht ohne Speichern verloren)
scratch-title = Unbenannte Reiter wiederherstellen
scratch-frage = Beim letzten Mal wurde das Programm nicht sauber beendet. { $n } unbenannte(r) Reiter mit Inhalt wurde(n) gesichert. Wiederherstellen?
scratch-zeichen = { $n } Zeichen
scratch-hinweis = Die Zwischenkopie liegt unverschlüsselt nur auf diesem Rechner und verfällt nach 14 Tagen.
scratch-wiederherstellen = Wiederherstellen

# Kontext: Layout der Arbeitsfläche (E1.4)
splitter-hilfe = Mit den Pfeiltasten verstellen, Umschalt für große Schritte, Pos1 und Ende für kleinste und größte Größe, Doppelklick stellt die Vorgabe her.
splitter-dock = Höhe der Ablage
splitter-panel = Breite des Maschine-Panels
splitter-seite = Breite der Seitenleiste
bereich-maximieren = Bereich maximieren
bereich-wiederherstellen = Bereich wiederherstellen
layout-zurueck = Layout zurückgesetzt.

# Kontext: Geteilte Ansicht (E1.4)
split-zu-wenig = Zum Teilen braucht es mindestens zwei Reiter.
split-hierher = Hier ablegen
split-rechts = Rechts ablegen (teilt)

# Kontext: Neues Projekt (E1.4)
err-neuprojekt-fehler = Das Projekt konnte nicht angelegt werden: { $detail }
np-zielsystem = Zielsystem
np-vorlage = Vorlage
np-sprache = Sprache
np-struktur = Dateistruktur der Ordner
np-kein-zeichensatz = Kein Zeichensatz; du legst später im Zeichensatz-Editor einen an.
neuprojekt-angelegt = Projekt angelegt: { $dir }
err-kopieren-fehler = Das Projekt konnte nicht kopiert werden: { $detail }

# Kontext: Startseite (E1.4)
start-projektdatei = Projektdatei (.vcprj) wählen …
start-anheften = Projekt anheften
start-loesen = Anheften lösen
start-entfernen = Aus der Liste entfernen

# Kontext: Bereichsnamen für Bildschirmleser (E1.4)
split-gruppe-links = Dokumentenfeld links
split-gruppe-rechts = Dokumentenfeld rechts
split-gruppe-dokument = Dokumentenfeld

# Kontext: Zuletzt geöffnete Projekte in der Befehlspalette (E1.4)
palette-projekte = Projekte
palette-projekt = Projekt öffnen: { $name } ({ $path })
# Kontext: Startseite, Beispielprojekte (E1.4 Rest)
start-beispiele-alle = Alle Beispiele …
start-beispiele-suche = Beispiele suchen
start-beispiele-keine = Keine Beispiele gefunden.
# veraltet seit E-Start: Empfehlung wird nicht mehr als Beschriftung gezeigt
start-beispiele-empfohlen = Empfohlen
# Kontext: Startseite, Zuletzt-Liste (E1.4 Rest 2)
start-angeheftet-kopf = Angeheftet
start-zuletzt-kopf = Zuletzt geöffnet
start-weitere = Weitere … ({ $n })
start-weniger = Weniger anzeigen
start-aktionen = Weitere Aktionen zu { $name }
# Kontext: Disk-Editor (E2)
disk-oeffnen = Diskette öffnen …
disk-abbilder = Diskettenabbilder
disk-leer-titel = Keine Diskette geöffnet
disk-leer-text = Öffne ein Diskettenabbild (D64, D71, D81 oder ATR) oder doppelklicke eine solche Datei im Projektbaum.
err-disk-unlesbar = Das Diskettenabbild lässt sich nicht lesen: { $detail }
disk-karte-diskette = Diskette
disk-belegung = Belegung
disk-belegt-frei = { $used } Blöcke belegt · { $free } frei
disk-aktionen = Aktionen
disk-schliessen = Diskette schließen
disk-verzeichnis = Verzeichnis
disk-n-dateien = { $n ->
    [one] 1 Datei
   *[other] { $n } Dateien
 }
disk-spalte-name = Name
disk-spalte-typ = Typ
disk-spalte-bloecke = Blöcke
disk-spalte-groesse = Größe
disk-spalte-start = Start
disk-autostart = Autostart
disk-geschuetzt = Datei geschützt
disk-belegungskarte = Belegungskarte
disk-legende-datei = Gewählte Datei
disk-legende-belegt = Belegt
disk-legende-verz = Verzeichnis
disk-legende-kopf = Kopf/BAM
disk-kette-zeigen = Kette zeigen
disk-karte-label = Belegungskarte, Pfeiltasten wählen den Sektor
disk-karte-ansage = Spur { $t }, Sektor { $s }: { $was }
disk-folgen-zu = Folgen ▶ { $a }
disk-kettenende = Kettenende
disk-zurueck = Zurück
disk-datei-block = { $name } · Block { $n } von { $m }
disk-naechster-sektor = Nächster Sektor
disk-bytes-im-sektor = Bytes in diesem Sektor
disk-vormerken = Vormerken
disk-vormerken-datei = Vormerken: { $name }
disk-vm-alle = Alle
disk-vm-keine = Keine
disk-vm-prg = Programme
disk-vm-basic = BASIC
disk-ablegen = Dateien ablegen …
disk-pruefen = Prüfen
disk-uebernehmen = Ins Projekt übernehmen
disk-speichern = Speichern
disk-speichern-unter = Speichern unter …
disk-neu = Neue Diskette …
disk-ungespeichert = Image ungespeichert
disk-bericht = Prüfbericht
disk-bericht-ok = Keine Probleme gefunden.
disk-uebernahme = Übernahme
disk-ziel = Zielordner im Projekt
disk-opt-basic = BASIC als Quelltext
disk-opt-raw = Originaldatei behalten
disk-opt-asm = Auch als Assembler
disk-ziel-ungueltig = Der Zielordner muss im Projekt liegen.
disk-projekt-noetig = Zum Übernehmen muss ein Projekt geöffnet sein.
disk-ohne-dateisystem = Diese Diskette hat kein Dateisystem.
disk-neu-titel = Neue Diskette
disk-neu-format = Format
disk-neu-name = Name der Diskette
disk-neu-id = Kennung (2 Zeichen)
disk-neu-anlegen = Anlegen
disk-frage-titel = Ungespeicherte Diskette
disk-frage-text = Die Diskette „{ $name }“ hat ungespeicherte Änderungen.
err-disk-ohne-pfad = Die Diskette hat noch keinen Speicherort.
err-disk-zielordner = Der Zielordner muss innerhalb des Projekts liegen.
err-disk-nichts-vorgemerkt = Es ist keine Datei vorgemerkt.
err-disk-verschieben = { $detail }
disk-datei-hoch = Nach oben
disk-datei-runter = Nach unten
disk-ordnen-hinweis = Reihenfolge im Verzeichnis: Zeile ziehen oder Alt+Pfeil hoch/runter
disk-ordnen-zeile = Eintrag { $name } verschieben
err-disk-neu = Die Diskette lässt sich nicht anlegen: { $detail }
disk-einlegen = Im Emulator einlegen
disk-einlegen-erst = Erst speichern, dann einlegen
disk-eingelegt = Eingelegt: { $c }
err-disk-erst-speichern = Erst speichern, dann einlegen.

sprite-leer-titel = Kein Sprite geöffnet
sprite-leer-text = Wähle eine Sprite-Datei aus dem Projekt oder öffne eine andere.
sprite-leer-kein-projekt = Zum Bearbeiten von Sprites muss ein Projekt geöffnet sein.
sprite-keine-dateien = Dieses Projekt enthält noch keine Sprite-Dateien.
sprite-oeffnen = Sprite-Datei öffnen …
sprite-dateien = Sprite-Dateien
sprite-werkzeuge = Werkzeuge
sprite-flaeche-label = Zeichenfläche
sprite-flaeche-hilfe = Pfeiltasten bewegen die Zellmarke, Eingabetaste oder Leertaste setzt einen Punkt, Entfernen löscht ihn. Ziffern wählen den Farbwert, D wählt den Stift, E den Radierer, I die Pipette. Plus und Minus zoomen, G schaltet das Raster.
sprite-zelle = x { $x } y { $y }
sprite-zelle-ansage = Zelle { $x }, { $y }: { $wert }
sprite-wert = Wert { $n }
sprite-wert-gewaehlt = Farbwert { $n }: { $name }
sprite-karte-farben = Farben
sprite-farbe-fuer = Farbe für „{ $name }“
sprite-durchsichtig = Dieser Wert ist durchsichtig und hat keine Farbe.
sprite-palette = Palette: { $name }
sprite-ansicht = Palettenansicht
sprite-ansicht-projekt = Projekt
sprite-raster = Raster
sprite-zoom-plus = Vergrößern
sprite-zoom-minus = Verkleinern
sprite-zoom-einpassen = Einpassen
sprite-zoom-text = { $z } %
sprite-fuss = { $name } · { $w } × { $h } · Bild { $i } von { $n }
sprite-kurz-wert = Farbwert
sprite-kurz-zoom = Zoom
sprite-frage-titel = Ungespeicherte Sprite-Datei
sprite-frage-text = Die Datei „{ $name }“ hat ungespeicherte Änderungen.
sprite-ungespeichert = ungespeichert
err-pixel-unlesbar = Die Datei „{ $rel }“ lässt sich nicht öffnen: { $detail }
err-pixel-speichern = Die Datei lässt sich nicht speichern: { $detail }

sprite-kopiert = Kopiert: { $w } × { $h } Punkte
sprite-ausgeschnitten = Ausgeschnitten: { $w } × { $h } Punkte
err-pixel-keine-auswahl = Es ist nichts ausgewählt.
err-pixel-zwischenablage-leer = Die Zwischenablage des Sprite-Editors ist leer.

sprite-spiegeln-h = Waagerecht spiegeln
sprite-spiegeln-v = Senkrecht spiegeln
sprite-verschieben = Verschieben
sprite-links = Nach links
sprite-rechts = Nach rechts
sprite-oben = Nach oben
sprite-unten = Nach unten
sprite-leeren = Bild leeren
sprite-auswahl-menue = Auswahl
sprite-auswahl-loeschen = Auswahl löschen
sprite-auswahl-aufheben = Auswahl aufheben
sprite-auswahl-ansage = Auswahl { $w } × { $h }
sprite-anfang-gesetzt = Anfang bei { $x }, { $y } gesetzt. Zellmarke bewegen, Eingabetaste setzt das Ende, Escape bricht ab.
sprite-abgebrochen = Abgebrochen

sprite-bilder = Bilder
sprite-bilder-n = { $n ->
    [one] 1 Bild
   *[other] { $n } Bilder
 }
sprite-bild-neu = Neu
sprite-bild-dupl = Duplizieren
sprite-bild-loeschen = Löschen
sprite-bild-ansage = Bild { $i } von { $n }
sprite-onion = Onion Skin
sprite-karte-sprite = Sprite
sprite-multicolor = Mehrfarbig
sprite-doppelt-breit = Doppelt breit
sprite-doppelt-hoch = Doppelt hoch
sprite-hinter-hintergrund = Hinter dem Hintergrund
sprite-im-spiel = { $n ->
    [one] Im Spiel auf 1 Punkt
   *[other] Im Spiel auf { $n } Punkte
 }
sprite-karte-vorschau = Vorschau im Spiel
sprite-speicher = { $bytes } Byte je Bild · { $n } Bilder · { $total } Byte

export-routine-rem = Daten laden: GOSUB @{ $label }_load liest die DATA-Zeilen ab dem aktuellen Zeiger in den Speicher ab { $addr }
export-format-unbekannt = Unbekanntes Exportformat.
export-routine-braucht-adresse = Für die Einleseroutine wird eine gültige Adresse gebraucht, z. B. $2000.
err-export-fehler = Der Export schlägt fehl: { $detail }
err-export-abgelehnt = Das Einbinden ist hier nicht möglich: { $detail }
err-export-adresse = Für dieses Format wird eine gültige Adresse gebraucht.
err-export-routine-braucht-adresse = Für die Einleseroutine wird eine gültige Adresse gebraucht, z. B. $2000.

export-titel = Exportieren
export-format = Format
export-format-asm = Assembler
export-format-inc = Konstanten
export-format-bin = Binär
export-format-prg = PRG
export-format-data = BASIC DATA
export-name = Name
export-adresse = Adresse
export-ladeadresse = Ladeadresse
export-routine = Einleseroutine mitliefern
export-routine-hinweis = Die Routine liest die DATA-Zeilen mit POKE ab der Adresse in den Speicher. Aufruf mit GOSUB.
export-vorschau = Vorschau
export-tab-text = Text
export-tab-hex = Hex
export-keine-binaer = Keine Binärdaten.
export-bereit = { $n ->
    [one] Bereit: 1 Datei, { $bytes } Byte
   *[other] Bereit: { $n } Dateien, { $bytes } Byte
 }
export-ziel = Ziel im Quelltext
export-kein-ziel = Kein passender Quelltext ist geöffnet. Öffne zuerst die Datei, in die eingefügt werden soll.
export-testen = Im Emulator testen
export-speichern = In Projekt speichern
export-include = Als Include einfügen
export-kopieren = In Quelltext kopieren
export-gespeichert = Gespeichert im Projekt: { $files }
export-eingefuegt = In { $doc } eingefügt: { $line }
export-kopiert = In { $doc } kopiert
export-nichts = Zum Exportieren muss ein Editor mit einer Mediendatei offen sein.

err-pixel-anim = Die Animation lässt sich nicht ändern: { $detail }
err-pixel-png = Das Bild lässt sich nicht laden: { $detail }
sprite-png-laden = PNG-Bild laden

sprite-zeitleiste = Zeitleiste
sprite-keine-animation = Keine Animation. Lege im Reiter Animationen eine an.
sprite-abspielen = Abspielen
sprite-stopp = Anhalten
sprite-ablauf = Ablauf umschalten
sprite-ablauf-loop = Schleife
sprite-ablauf-pingpong = Hin und zurück
sprite-ablauf-once = Einmal
sprite-anim-dauer = { $ticks } Bilder · { $s } s
sprite-anim-schritte = Schritte der Animation
sprite-schritt-titel = Schritt { $i }: Bild { $f }, { $d } Bilder lang
sprite-reiter-bilder = Bilder
sprite-reiter-anim = Animationen
sprite-anim-neu = Neu
sprite-anim-loeschen = Löschen
sprite-anim-name = Name der Animation
sprite-anim-keine = Noch keine Animation. „Neu“ legt eine mit dem gewählten Bild an.
sprite-anim-anhaengen = Gewähltes Bild anhängen
sprite-anim-hoch = Schritt nach oben
sprite-anim-runter = Schritt nach unten
sprite-anim-entfernen = Schritt entfernen
sprite-anim-dauer-label = Dauer in Bildern (1/50 s)
sprite-anim-fehler = Die Animationen sind fehlerhaft: { $e }
sprite-png = PNG laden …
sprite-png-tipp = Lädt ein PNG-Bild in das gewählte Bild; die Datei wird dabei gespeichert
sprite-png-nichts = Zum Laden eines PNG-Bildes muss der Sprite-Editor mit einer Datei offen sein.

err-media-fehler = Die Datei lässt sich nicht anlegen: { $detail }
sprite-neu = Neue Sprite-Datei …
sprite-neu-titel = Neue Sprite-Datei
sprite-neu-profil = Art und System
sprite-neu-name = Name (Buchstaben, Ziffern, _ und -)
sprite-neu-ordner = Ordner im Projekt
sprite-neu-anlegen = Anlegen
sprite-neu-name-ungueltig = Der Name darf nur Buchstaben, Ziffern, _ und - enthalten.
system-name-vic20 = VC-20

sprite-karte-pmg = Player/Missile
sprite-pmg-breite = Breite Spieler { $n }
sprite-pmg-breite-0 = Normal
sprite-pmg-breite-1 = Doppelt
sprite-pmg-breite-3 = Vierfach
sprite-pmg-prior = Priorität
sprite-pmg-prior-0 = Nicht gesetzt
sprite-pmg-prior-1 = Spieler vor Spielfeld
sprite-pmg-prior-2 = P0/P1, Spielfeld, P2/P3
sprite-pmg-prior-4 = PF0/PF1, Spieler, PF2/PF3
sprite-pmg-prior-8 = Spielfeld vor Spielern

# Kontext: Sprite-Editor, Karte Player/Missile: Hinweis zur sichtbaren Fläche
sprite-pmg-sichtbar = Ein Player zeigt Zeile für Zeile den Speicher von oben nach unten. Die ersten etwa 32 Zeilen liegen über dem sichtbaren Bild: Zeichne das Motiv weiter unten, oder erhöhe die Höhe des Bildes. Nicht gesetzte Priorität wird mit 1 (Spieler vor Spielfeld) exportiert.

# Kontext: Sprite-Editor, Bilderliste: mehrere Bilder markieren und daraus eine Animation erzeugen
sprite-bilder-markiert = { $n ->
    [one] 1 Bild markiert
   *[other] { $n } Bilder markiert
}
sprite-anim-aus-auswahl = Animation aus Auswahl
sprite-anim-aus-auswahl-punkte = Animation aus Auswahl …
sprite-alle-markieren = Alle Bilder markieren
sprite-markierung-aufheben = Auswahl aufheben
sprite-markier-hinweis = Mit Umschalt- oder Strg-Klick mehrere Bilder markieren und daraus eine Animation machen.
sprite-anim-aus-titel = { $n ->
    [one] Animation aus 1 Bild
   *[other] Animation aus { $n } Bildern
 }
sprite-anim-aus-ablauf = Ablauf
sprite-anim-aus-tempo = Tempo (Bilder pro Sekunde)
sprite-anim-aus-ticks = entspricht { $t } Ticks (1/50 s) je Bild
sprite-anim-aus-hinweis = Die Bilder laufen in der Reihenfolge der Bilderliste.
sprite-anim-aus-erzeugen = Animation erzeugen
sprite-anim-aus-vergeben = Dieser Name ist schon vergeben.
sprite-anim-aus-ungueltig = Der Name darf nur Buchstaben, Ziffern und _ enthalten und nicht mit einer Ziffer beginnen.

sprite-anim-schritte-liste = Schritte der Animation
sprite-anim-ziehen-hinweis = Schritte lassen sich ziehen oder mit den Pfeilen verschieben; Klick auf ein Bild wählt es zum Bearbeiten.
sprite-anim-tempo-gemischt = unterschiedliche Dauer je Bild

# Kontext: Release-Auswahl und Erweiterungen
ext-werkzeuge-folgen = Befehle aus Erweiterungen laufen mit den Werkzeugen, die in einer späteren Version folgen.
release-ohne-editor = „{ $name }“ öffnet als Text: Der { $editor } ist in diesem Release nicht enthalten.
release-editor-sprite = Sprite-Editor
release-editor-disk = Disk-Editor
palette-herkunft = Befehl aus der Erweiterung { $paket }
beispiele-herkunft = Beispiel aus der Erweiterung { $paket }

# Kontext: Dateien aus dem Dateimanager in das Fenster ziehen
ablegen-hier = Dateien hier ablegen, um sie zu öffnen
ablegen-kopieren-titel = Ins Projekt kopieren und öffnen?
ablegen-kopieren-text = { $n ->
    [one] Diese Datei liegt außerhalb des Projekts. Sie wird ins Projekt kopiert (nichts wird überschrieben) und dort geöffnet:
   *[other] Diese { $n } Dateien liegen außerhalb des Projekts. Sie werden ins Projekt kopiert (nichts wird überschrieben) und dort geöffnet:
}
ablegen-kopieren-ja = Kopieren und öffnen
ablegen-kopiert = { $n ->
    [one] 1 Datei ins Projekt kopiert.
   *[other] { $n } Dateien ins Projekt kopiert.
}
ablegen-kein-projekt = „{ $name }“ lässt sich nur mit geöffnetem Projekt ablegen. Öffne zuerst ein Projekt.
ablegen-ordner = „{ $name }“ ist ein Ordner ohne Projektdatei und wird nicht geöffnet.
ablegen-fehlt = „{ $name }“ gibt es nicht mehr.
ablegen-typ = „{ $name }“ hat einen Dateityp, den VintageCoder nicht öffnet.
err-ablegen-keine-datei = „{ $name }“ ist keine Datei.
err-ablegen-zu-gross = „{ $name }“ ist zu groß zum Kopieren (über 32 MB).

# Kontext: Sprite-Editor, große Vorschau der Animation
sprite-vorschau-gross = Vorschau vergrößern
sprite-vorschau-gross-titel = Animation „{ $name }“
sprite-zoom-label = Zoom
sprite-vorschau-schritt = Schritt { $i } von { $n }
dialog-schliessen = Schließen
release-editor-char = Zeichensatz-Editor

# Kontext: Zeichensatz-Editor (E4)
zeichensatz-leer-titel = Kein Zeichensatz geöffnet
zeichensatz-leer-text = Wähle eine Zeichensatz-Datei (.vcchar) aus dem Projekt oder öffne eine andere.
zeichensatz-leer-kein-projekt = Zum Bearbeiten von Zeichensätzen muss ein Projekt geöffnet sein.
zeichensatz-keine-dateien = Das Projekt enthält noch keine Zeichensatz-Datei.
zeichensatz-dateien = Zeichensatz-Dateien
zeichensatz-uebersicht-titel = Zeichenübersicht
zeichensatz-anzahl = { $n } Zeichen
zeichensatz-ansage = Zeichen { $n }, { $hex }
zeichensatz-flaeche-label = Zeichen bearbeiten
zeichensatz-umgebung = Umgebung
zeichensatz-chip = Zeichen { $n } · { $hex }
zeichensatz-fuss = { $name } · Zeichen { $n } ({ $hex })
zeichensatz-vorschau-titel = Vorschau im Bild
zeichensatz-vorschau-label = Vorschau eines Textes in diesem Zeichensatz
zeichensatz-text-label = Text für die Vorschau
zeichensatz-karte-zeichen = Zeichen
zeichensatz-nummer = Nummer
zeichensatz-nummer-hinweis = Bildschirmcode
zeichensatz-position = Position im Satz
zeichensatz-position-hinweis = Byte-Abstand zum Anfang ({ $n } Byte je Satz)
zeichensatz-zeichen-leeren = Zeichen leeren
zeichensatz-vorschau-text = HUGO UND GUMBO\nPUNKTE 00420
zeichensatz-bereiche = Bereiche
zeichensatz-bereich-alle = Alle
zeichensatz-bereich-buchstaben = Buchstaben
zeichensatz-bereich-ziffern = Ziffern
zeichensatz-bereich-zeichen = Satzzeichen
zeichensatz-bereich-frei = Frei
zeichensatz-bereich-von = von
zeichensatz-bereich-bis = bis
zeichensatz-bereich-ansage = Bereich { $name }: Zeichen { $von } bis { $bis }
zeichensatz-vorlage-titel = Vorlage
zeichensatz-vorlage-label = Vorlage zum Laden
zeichensatz-vorlage-keine = Selbst gestaltet, keine Vorlage geladen
zeichensatz-vorlage-laden = Vorlage laden
zeichensatz-vorlage-hinweis = Ersetzt alle Zeichen; mit Rückgängig kommst du zurück. Sätze aus einem ROM stehen nur als Verweis in der Datei.
zeichensatz-vorlage-fehlt = ROM nicht gefunden
zeichensatz-herkunft = Herkunft
zeichensatz-wie-vorlage = Wie die Vorlage
zeichensatz-geaendert = Gegenüber der Vorlage geändert
zeichensatz-original-zurueck = Zeichen auf Vorlage zurücksetzen
zeichensatz-aenderungen = { $n ->
    [one] 1 Zeichen weicht von der Vorlage ab
   *[other] { $n } Zeichen weichen von der Vorlage ab
 }
zeichensatz-neu = Neuer Zeichensatz …
zeichensatz-neu-titel = Neuer Zeichensatz
zeichensatz-neu-name = zeichensatz
err-pixel-vorlage = Die Vorlage lässt sich nicht laden: { $detail }
zeichensatz-uebersicht-kurz = Zeichen
nav-main-projekt = Projekt
nav-projekt-uebersicht = Übersicht
nav-projekt-allgemein = Allgemein
nav-projekt-ziele = Ziele
nav-projekt-quellen = Quellen
nav-projekt-bauen = Bauen und Starten
nav-projekt-diskette = Diskette
nav-projekt-weitergeben = Weitergeben
view-title-projekt-uebersicht = Projekt: Übersicht
view-title-projekt-allgemein = Projekt: Allgemein
view-title-projekt-ziele = Projekt: Ziele
view-title-projekt-quellen = Projekt: Quellen
view-title-projekt-bauen = Projekt: Bauen und Starten
view-title-projekt-diskette = Projekt: Diskette
view-title-projekt-weitergeben = Projekt: Weitergeben
projekt-ohne-projekt = Öffne zuerst ein Projekt (über „Startseite“ ganz links oben oder den Befehl „Projekt öffnen …“), dann siehst du hier seine Übersicht und Einstellungen.
projekt-seite-folgt = Diese Seite entsteht in einer der nächsten Etappen des Bereichs Projekt. Hier kannst du schon zu den anderen Seiten wechseln.
projekt-name-fehlt = Projektname fehlt
projekt-start = Start: { $f }
projekt-kein-programm = kein Programm
projekt-kein-ziel = Kein Zielsystem eingetragen
projekt-quellen-anzahl = Programme: { $n } · Quellgruppen: { $g }
projekt-fehlende-dateien = { $n ->
    [one] 1 Datei fehlt auf der Platte
   *[other] { $n } Dateien fehlen auf der Platte
 }
projekt-diskette-an = { $n ->
    [one] Diskette { $f }, 1 Programm
   *[other] Diskette { $f }, { $n } Programme
 }
projekt-turbo-an = Turbo-BASIC-XL-Diskette des Anwenders
projekt-diskette-aus = Keine Diskette eingerichtet
projekt-zustand-ok = in Ordnung
projekt-zustand-warn = Hinweis
projekt-zustand-err = Angabe fehlt
projekt-zustand-off = nicht eingeschaltet
projekt-aendern = Ändern ›
projekt-probleme = Probleme der Projektdatei
projekt-name = Name
projekt-beschreibung = Beschreibung
projekt-startdatei = Start-Programm
projekt-startdatei-auto = Automatisch: erstes Programm ({ $f })
projekt-startdatei-hinweis = Dieses Programm startet mit F5 und wird beim Bauen zuerst gebaut. Ohne Angabe gilt das erste Programm der Quellen.
projekt-reiter-wiederherstellen = Beim Öffnen die zuletzt geöffneten Dateien wiederherstellen
projekt-uebernehmen = Übernehmen
projekt-gespeichert = Gespeichert.
projekt-variante-von = Variante von { $s }
projekt-variante-hinweis = Die Variante bestimmt Takt und Zeilenzahl (zum Beispiel PAL oder NTSC) für Emulator, Vorschau und Zeitmessung.
projekt-keine-variante = Dieses Zielsystem hat keine Varianten.
projekt-emulator-dieses-ziels = Emulator nur für dieses Ziel: { $e }
projekt-quellen-hinweis = Die Programme in der Reihenfolge, in der sie gebaut werden. Verschieben geht innerhalb einer Quellgruppe. Fehlende Dateien stehen rot.
projekt-gruppe = Quellgruppe { $n }
projekt-datei-oeffnen = Datei im Code-Editor öffnen
projekt-start-programm = Start-Programm
projekt-fehlt = fehlt
projekt-als-start = Als Start-Programm
projekt-nach-oben = { $f } nach oben
projekt-nach-unten = { $f } nach unten
projekt-keine-quellen = Das Projekt hat noch keine Quellen.
projekt-letzter-bau = Letzter Bau
projekt-bau-ok = Gebaut um { $w }, ohne Fehler.
projekt-bau-fehler = { $n ->
    [one] Gebaut um { $w }: 1 Fehler.
   *[other] Gebaut um { $w }: { $n } Fehler.
 }
projekt-noch-nicht-gebaut = In dieser Sitzung noch nicht gebaut.
projekt-emulator-je-ziel = Emulator je Ziel
projekt-emulator-projekt = Emulator des Projekts: { $e }
projekt-emulator-global = Emulator aus den Einstellungen
projekt-start-auf = startet auf { $s }
projekt-emulator-hinweis = Hier stehen nur Überschreibungen, die dieses Projekt festlegt. Die allgemeine Wahl von Emulator und Startziel steht in den Einstellungen.
projekt-pakete = Pakete (Registry)
projekt-keine-pakete = Keine Pakete geladen.
projekt-paket-herkunft = Quelle: { $k } · { $o } · { $p }
projekt-kopieren-titel = Projekt kopieren
projekt-kopieren-text = Legt eine Kopie des Projekts an einem anderen Ort an, zum Beispiel für die Weitergabe oder als Sicherung.
projekt-kopieren-knopf = Projekt kopieren …
projekt-export-titel = Export von Grafik, Zeichensätzen und Musik
projekt-export-text = Den Export (als Quelltext, DATA-Zeilen oder PRG) findest du im jeweiligen Editor über den Knopf „Export …“. Die erzeugten Dateien dürfen frei verwendet werden.
projekt-verlauf = Zuletzt geöffnet
err-palette-unlesbar = Die Palette „{ $rel }“ lässt sich nicht öffnen: { $detail }
err-palette-hex = „{ $hex }“ ist kein Farbwert; erlaubt ist #rrggbb.
palette-rot = Rot
palette-gruen = Grün
palette-blau = Blau
palette-art-sprite = Sprite
palette-art-pmg = Player/Missile
palette-art-zeichensatz = Zeichensatz
palette-art-kacheln = Kacheln
palette-art-bildschirm = Bildschirm
palette-leer-titel = Keine Palette geöffnet
palette-leer-kein-projekt = Zum Bearbeiten von Paletten muss ein Projekt geöffnet sein.
palette-leer-text = Wähle eine Palettendatei (.vcpal) aus dem Projekt oder öffne eine andere.
palette-pal-dateien = Palettendateien
palette-keine-dateien = Das Projekt enthält noch keine Palettendatei.
palette-raster-titel = Farbraster
palette-werkzeuge = Werkzeuge der Palette
palette-anzahl = { $n ->
    [one] 1 Farbe
   *[other] { $n } Farben
 }
palette-standard = Standardpalette
palette-raster-label = { $n ->
    [one] Farbraster mit 1 Farbe; Pfeiltasten wählen eine Farbe
   *[other] Farbraster mit { $n } Farben; Pfeiltasten wählen eine Farbe
 }
palette-farbe-ansage = Farbe { $n }, { $hex }, { $farbe }
palette-farbe-titel = Farbe { $n }
palette-farbfeld = Gewählte Farbe { $hex }
palette-regler = Farbkanäle
palette-hexwert = Hexwert
palette-hex-ungueltig = Das ist kein Farbwert; erlaubt ist #rrggbb.
palette-im-einsatz = Im Einsatz
palette-nicht-im-einsatz = Diese Farbe wird in keiner Mediendatei des Projekts verwendet.
palette-ausgabe = Ausgabe
palette-ausgabe-text = Als Assembler-Daten, { $n } Byte
palette-fuss = { $name } · Farbe { $n } ({ $hex }) von { $count }
kacheln-leer-titel = Kein Kachelsatz geöffnet
kacheln-leer-kein-projekt = Zum Bearbeiten von Kacheln muss ein Projekt geöffnet sein.
kacheln-leer-text = Wähle einen Kachelsatz (.vctiles) aus dem Projekt oder öffne einen anderen.
kacheln-dateien = Kachelsätze
kacheln-keine-dateien = Das Projekt enthält noch keinen Kachelsatz.
kacheln-satz-titel = Kachelsatz · { $name }
kacheln-neu = Neue Kachel
kacheln-liste-label = { $n ->
    [one] Kachelsatz mit 1 Kachel; Pfeiltasten wählen eine Kachel
   *[other] Kachelsatz mit { $n } Kacheln; Pfeiltasten wählen eine Kachel
 }
kacheln-ansage = Kachel { $n }, { $name }, { $uses } mal verwendet
kacheln-doppelt = Gleich wie Kachel { $j }
kacheln-flaeche-label = Kachel bearbeiten
kacheln-werkzeuge = Werkzeuge der Kachel
kacheln-drehen = Kachel drehen
kacheln-rollen-links = Zellen nach links rollen
kacheln-rollen-rechts = Zellen nach rechts rollen
kacheln-rollen-oben = Zellen nach oben rollen
kacheln-rollen-unten = Zellen nach unten rollen
kacheln-leeren = Leeren
kacheln-pinsel-label = Ein Klick setzt
kacheln-pinsel-beides = Zeichen und Farbe
kacheln-pinsel-zeichen = Nur Zeichen
kacheln-pinsel-farbe = Nur Farbe
kacheln-chip = { $w }×{ $h } Zellen · Zeichensatz { $cs }
kacheln-zellen-label = Zellen der Kachel ({ $w } × { $h }); Pfeiltasten bewegen, Enter setzt das Zeichen, I nimmt es auf
kacheln-zelle-ansage = Zelle { $x }, { $y }: Zeichen { $code }, { $hex }
kacheln-zeichen-waehlen = Zeichen wählen
kacheln-zeichensaetze = Zeichensätze des Kachelsatzes
kacheln-tafel-label = Zeichentafel mit { $n } Zeichen; Pfeiltasten wählen ein Zeichen
kacheln-tafel-hinweis = Ein Klick setzt das Zeichen in die markierte Zelle.
kacheln-bit7 = Zeichenbit 7 setzen (Farbwahl des Zeichens)
kacheln-fuss = { $name } · Kachel Nr. { $n } · { $count } Kacheln
kacheln-karte-titel = Kachel
kacheln-name = Name der Kachel
kacheln-farbe-titel = Farbe
kacheln-farbmodus = Farbe gilt
kacheln-farbe-je-kachel = Je Kachel
kacheln-farbe-je-zelle = Je Zelle
kacheln-farbe-ohne = Ohne
kacheln-kachelfarbe = Farbe der Kachel
kacheln-pinselfarbe = Pinselfarbe
kacheln-hintergrund = Hintergrundfarbe
kacheln-eigenschaften = Eigenschaften
kacheln-keine-eigenschaften = Der Satz hat noch keine Eigenschaften.
kacheln-eigenschaft-neu = Neue Eigenschaft
kacheln-hinzufuegen = Hinzufügen
kacheln-groesse = Größe der Kacheln
kacheln-breite = Breite
kacheln-hoehe = Höhe
kacheln-uebernehmen = Übernehmen
kacheln-zielzeichensatz = Zielzeichensatz
kacheln-fuellstand-label = Belegte Zeichen des Zielzeichensatzes
kacheln-fuellstand = von { $max } Zeichen belegt · { $frei } frei
kacheln-fuellstand-hinweis = Doppelte Zeichen werden beim Export zusammengeführt. Angepinnte Zeichen behalten ihre Nummer.
kacheln-zeichensatz-hinzu = Zeichensatz zum Kachelsatz hinzufügen
kacheln-zeichensatz-benutzt = Wird von Zellen verwendet
kacheln-entfernen = { $name } entfernen
kacheln-banken = { $n ->
    [one] Der Satz wird in 1 Bildschirmbank verwendet.
   *[other] Der Satz wird in { $n } Bildschirmbanken verwendet.
 }
kacheln-satz-aktionen = Kachel
kacheln-duplizieren = Duplizieren
kacheln-zusammenfuehren = Zusammenführen
kacheln-loeschen = Löschen
kacheln-loeschen-titel = Kachel löschen?
kacheln-loeschen-text = Kachel { $i } wird { $uses } mal in Bildschirmbanken verwendet. Beim Löschen zeigen diese Stellen danach auf Kachel 0, höhere Kacheln rücken nach, der Satz wird gespeichert.
kacheln-groesse-titel = Größe ändern?
kacheln-groesse-text = Bei { $w } × { $h } Zellen gehen belegte Zellen verloren. Rückgängig holt sie zurück.
err-kacheln-unlesbar = Der Kachelsatz „{ $rel }“ lässt sich nicht öffnen: { $detail }
err-kacheln-zwischenablage = Es gibt nichts einzufügen: erst eine Kachel kopieren (gleiche Größe).
err-kacheln-eigenschaft = Die Eigenschaft „{ $name }“ lässt sich nicht anlegen: Name leer oder schon vorhanden, oder alle acht Plätze sind belegt.
palette-frage-titel = Ungespeicherte Palette
kacheln-frage-titel = Ungespeicherter Kachelsatz
zeichensatz-frage-titel = Ungespeicherter Zeichensatz
bildschirm-werkzeug-zeichen = Zeichen setzen
bildschirm-werkzeug-rahmen = Rahmen
bildschirm-werkzeug-fuellen = Füllen
bildschirm-werkzeug-text = Text tippen
bildschirm-werkzeug-pipette = Pipette
bildschirm-werkzeug-loeschen = Löschen
bildschirm-leer-titel = Kein Bildschirm geöffnet
bildschirm-leer-kein-projekt = Zum Bearbeiten von Bildschirmen muss ein Projekt geöffnet sein.
bildschirm-leer-text = Wähle eine Bildschirm-Datei (.vcscreen) aus dem Projekt oder öffne eine andere.
bildschirm-dateien = Bildschirm-Dateien
bildschirm-keine-dateien = Das Projekt enthält noch keine Bildschirm-Datei.
bildschirm-zeichen-titel = Zeichen
bildschirm-ohne-zeichensatz = Ohne Zeichensatz
bildschirm-tafel-label = Zeichentafel mit { $n } Zeichen; Pfeiltasten wählen ein Zeichen
bildschirm-farbe-titel = Zeichenfarbe
bildschirm-zeichenfarbe = Farbe der neuen Zeichen
bildschirm-flaeche-label = Bildschirm bearbeiten
bildschirm-werkzeuge = Werkzeuge des Bildschirms
bildschirm-zerlegen = In Kacheln zerlegen …
bildschirm-text-hinweis = Text tippen: Zelle anklicken, dann schreiben; Pfeiltasten bewegen den Cursor, Esc beendet.
bildschirm-rahmen-hinweis = Rahmen: mit Enter die zweite Ecke setzen, Esc bricht ab.
bildschirm-zeichen-hinweis = Klick setzt das gewählte Zeichen, die rechte Maustaste löscht; mit Enter setzt du es an der Cursorzelle.
bildschirm-zellen-label = Zellen des Bildschirms ({ $w } × { $h }); Pfeiltasten bewegen den Cursor, Enter wendet das Werkzeug an
bildschirm-zelle-ansage = Spalte { $x }, Zeile { $y }: Zeichen { $code }, { $hex }
bildschirm-zelle-info = Spalte { $x } Zeile { $y } · Zeichen { $code } ({ $hex })
bildschirm-karte-titel = Bildschirm
bildschirm-groesse = Größe
bildschirm-groesse-farbram = Textbildschirm mit Farbe je Zelle
bildschirm-groesse-einfarbig = Textbildschirm, eine Zeichenfarbe
bildschirm-farben-system = Farben des Systems
bildschirm-rahmen = Rahmen
bildschirm-hintergrund = Hintergrund
bildschirm-zeichen-rolle = Zeichen
bildschirm-zeichensatz = Zeichensatz
bildschirm-ausgabe = Ausgabe
bildschirm-ausgabe-text = Als Assembler-Daten, { $n } Zeichen
bildschirm-zerlegen-titel = In Kacheln zerlegen
bildschirm-zerlegen-text = Gleiche Blöcke werden Kacheln. Kachelsatz und Bildschirmbank entstehen neben dem Bildschirm, der Bildschirm bleibt unverändert.
bildschirm-kachelgroesse = Kachelgröße
bildschirm-zerlegen-los = Zerlegen
bildschirm-frage-titel = Ungespeicherter Bildschirm
err-bildschirm-unlesbar = Der Bildschirm „{ $rel }“ lässt sich nicht öffnen: { $detail }
err-bildschirm-zerlegen = Das Zerlegen in Kacheln ist nicht möglich: { $detail }
tracker-leer-titel = Kein Song geöffnet
tracker-leer-kein-projekt = Zum Bearbeiten von Songs muss ein Projekt geöffnet sein.
tracker-leer-text = Wähle einen Song (.vcsong oder .vcplay) aus dem Projekt oder öffne einen anderen.
tracker-dateien = Songs
tracker-keine-dateien = Das Projekt enthält noch keinen Song.
tracker-liste-titel = Song
tracker-muster-neu = Neues Muster
tracker-muster-nr = Muster { $n }
tracker-reihenfolge = Reihenfolge
tracker-reihenfolge-label = { $n ->
    [one] Reihenfolge der Muster mit 1 Eintrag; Pfeiltasten wählen einen Eintrag
   *[other] Reihenfolge der Muster mit { $n } Einträgen; Pfeiltasten wählen einen Eintrag
 }
tracker-eintrag-ansage = Eintrag { $i }: Muster { $p }
tracker-schleife = Hier beginnt die Wiederholung
tracker-schleife-ab = Wiederholung ab Eintrag
tracker-anhaengen = Anhängen
tracker-setzen = Setzen
tracker-entfernen = Entfernen
tracker-nach-oben = Eintrag nach oben
tracker-nach-unten = Eintrag nach unten
tracker-muster-dup = Duplizieren
tracker-muster-loeschen = Löschen
tracker-play-hinweis = PLAY-Dateien haben ein Muster und keine Reihenfolge.
tracker-instrumente = Instrumente
tracker-instrumente-label = { $n ->
    [one] Instrumente mit 1 Eintrag; Pfeiltasten wählen ein Instrument
   *[other] Instrumente mit { $n } Einträgen; Pfeiltasten wählen ein Instrument
 }
tracker-instrument-ansage = Instrument { $i }: { $name }
tracker-instrument-neu = Neues Instrument
tracker-instrument-loeschen-hinweis = Nur das letzte, unbenutzte Instrument lässt sich löschen
tracker-raster-label = Muster bearbeiten
tracker-werkzeuge = Werkzeuge des Trackers
tracker-tempo = Tempo
tracker-zeilen = Zeilen
tracker-oktave = Oktave
tracker-schritt = Schritt
tracker-halbton-hoch = Halbton ↑
tracker-halbton-runter = Halbton ↓
tracker-instrument-anwenden = Instrument anwenden
tracker-muster-wahl = Muster wählen
tracker-stimme = Stimme { $n }
tracker-raster-liste = Raster des Musters ({ $r } Zeilen, { $c } Stimmen); Pfeiltasten bewegen, Klaviertasten tragen Noten ein
tracker-zelle-ansage = Reihe { $r }, Stimme { $c }: { $note }, Instrument { $ins }
tracker-reihe = Reihe { $r }
tracker-kanal = Stimme { $c }
tracker-tasten-hinweis = Z–M, Q–I: Noten · 1: Notenende · Entf: löschen · , .: Oktave · [ ]: Instrument
tracker-instrument-titel = Instrument
tracker-instrument-titel-n = Instrument { $n } · { $name }
tracker-instrument-name = Name des Instruments
tracker-klangform = Klangform
tracker-welle-dreieck = Dreieck
tracker-welle-saege = Säge
tracker-welle-rechteck = Rechteck
tracker-welle-rauschen = Rauschen
tracker-anschlag = Anschlag
tracker-abfall = Abfall
tracker-halten = Halten
tracker-ausklang = Ausklang
tracker-pulsbreite = Pulsbreite
tracker-verzerrung = Verzerrung
tracker-dist-rein = Reiner Ton
tracker-dist-bass = Bass
tracker-dist-buzz = Buzz
tracker-dist-gritty = Gritty
tracker-huellkurve = Hüllkurve je Bild
tracker-huellkurve-label = Lautstärke der 16 Bilder der Hüllkurve
tracker-env-bild = Bild { $k }: Lautstärke { $w }
tracker-env-schleife = Hüllkurve wiederholt ab Bild
tracker-lautstaerke = Lautstärke
tracker-frage-titel = Ungespeicherter Song
err-song-unlesbar = Der Song „{ $rel }“ lässt sich nicht öffnen: { $detail }
err-song-zwischenablage = Es gibt nichts einzufügen: erst Zellen kopieren.
err-song-instrument-loeschen = Gelöscht werden kann nur das letzte Instrument, wenn kein Muster es benutzt.
err-audio-fehler = Wiedergabe nicht möglich: { $detail }
tracker-abspielen-song = Song abspielen (Leertaste)
tracker-abspielen-muster = Muster abspielen (Umschalt+Leertaste)
tracker-muster-kurz = Muster ▶
tracker-stopp = Wiedergabe stoppen
tracker-mitscrollen = Mitscrollen
tracker-stumm = Stimme { $n } stumm
tracker-solo = Stimme { $n } solo
tracker-spielt = Wiedergabe: Eintrag { $i }, Zeile { $r }
err-song-import-fehler = { $detail }
tracker-quelle-filter = Songs (SID, GoatTracker, VintageCoder)
tracker-konvertieren = Übernehmen
tracker-konvertieren-titel = Song aus anderem System übernehmen
tracker-rmt-import = RMT-Modul importieren
tracker-import-name = Name der neuen Datei
tracker-import-ziel = Zielsystem
tracker-import-subtune = Subtune (1 bis { $n })
tracker-import-sekunden = Aufnahme in Sekunden
tracker-import-laeuft = Läuft …
tracker-stimmen = Stimmen
tracker-stimmen-sichtbar = Sichtbare Stimmen
tracker-alle-einblenden = Alle einblenden
tracker-klaviatur = Klaviatur
tracker-klaviatur-label = Klaviatur ab Oktave { $o }
tracker-taste-ansage = Taste { $note }
tracker-klaviatur-hinweis = Klick spielt die Note mit dem gewählten Instrument vor, trägt aber nichts ein. Beschriftet ist die Computertaste.
err-debug-kein-bau = Zum Debuggen muss das Programm erst gebaut sein.
err-debug-fehler = Debuggen nicht möglich: { $detail }
err-debug-testlauf = Im Testlauf wird keine Debug-Sitzung gestartet.
err-debug-nur-usb = { $name }: Debuggen geht nur über USB, nicht über Ethernet.
err-debug-keine-sitzung = Es läuft keine Debug-Sitzung.
err-debug-keine-erkennung = Erst die Momentaufnahme erkennen.
err-debug-zeile-ohne-code = Die Zeile { $zeile } hat keinen Code: dorthin lässt sich nicht laufen.
dbgui-adresse = Adresse ($D020 oder Symbol)
dbgui-angehalten = ANGEHALTEN
dbgui-art = Art des Punkts
dbgui-art-access = Zugriff
dbgui-art-exec = Ausführen
dbgui-art-read = Lesen
dbgui-art-write = Schreiben
dbgui-bau-fehlt = Der letzte Bau ist fehlgeschlagen. Erst die Fehler beheben.
dbgui-bedingung = Bedingung (z. B. a == 5)
dbgui-beendet = Debug-Sitzung beendet.
dbgui-beob-eingabe = Ausdruck, Adresse oder Symbol
dbgui-beob-weg = Beobachtung entfernen
dbgui-beobachten = Beobachten
dbgui-bild-aktualisieren = Bild aktualisieren
dbgui-bild-alt = Bild des Emulators
dbgui-bild-auto = Bei jedem Halt aktualisieren
dbgui-bild-leer = Noch kein Bild. „Bild aktualisieren“ holt eines.
dbgui-bild-live = Live (Zeitlupe)
dbgui-bild-nicht = Dieser Emulator liefert kein Bild.
dbgui-bild-quelle = Bildquelle
dbgui-bildschirm = Bildschirm
dbgui-bildschirm-bei = Bildschirm bei
dbgui-disasm = Disassembler
dbgui-editor = Editor
dbgui-ende = Ende (bei Bereich)
dbgui-erkennen = Erkennen, was im Speicher liegt
dbgui-flags = Flags
dbgui-haltepunkt-bei = Haltepunkt bei ${ $a }
dbgui-haltepunkte = Haltepunkte
dbgui-hierher-hinweis = Cursor in eine Quelldatei setzen, dann „Bis hierher ausführen“.
dbgui-hinzufuegen = Hinzufügen
dbgui-ignorieren = Ignorieren
dbgui-keine-beobachtungen = Nichts beobachtet. Ein Ausdruck zeigt hier seinen Wert, wenn das Programm angehalten ist.
dbgui-keine-punkte = Keine Haltepunkte. Ein Klick links neben eine Zeile setzt einen.
dbgui-laeuft-hinweis = Das Programm läuft. Anhalten zeigt Register, Disassembler und Speicher.
dbgui-leer-hinweis = Haltepunkte setzt du im Editor am linken Rand (oder F9) und im Disassembler.
dbgui-leer-text = Baut das Programm und startet es im Emulator mit einer Debug-Sitzung: Haltepunkte, Einzelschritt, Register, Speicher.
dbgui-leer-titel = Keine Debug-Sitzung
dbgui-links = Haltepunkte, Beobachten, Momentaufnahme
dbgui-mehrfarbig = mehrfarbig
dbgui-mem-eingabe = Adresse oder Symbol
dbgui-mem-los = Anzeigen
dbgui-mitte = Ablauf, Disassembler und Speicher
dbgui-moment = Momentaufnahme
dbgui-moment-halt = Die Momentaufnahme geht nur, wenn das Programm angehalten ist.
dbgui-moment-kacheln = Kacheln aus dem Bildschirm bilden
dbgui-moment-name = Name der Momentaufnahme
dbgui-moment-rom = Zeichensatz aus dem ROM holen
dbgui-moment-sichern = Als Dateien sichern
dbgui-moment-sprites = Sprites übernehmen
dbgui-punkt-adresse = Das ist keine Adresse. Beispiele: $D020, 0x0400, 2049.
dbgui-punkt-loeschen = Punkt löschen
dbgui-rechts = Register, Stapel und Bild
dbgui-register = Register
dbgui-speicher = Speicher ab
dbgui-sprites-n = { $n ->
    [one] 1 Sprite
   *[other] { $n } Sprites
 }
dbgui-stapel = Stapel
dbgui-stapel-leer = Stapel leer
dbgui-start = Debuggen starten
dbgui-startet = Debug-Sitzung startet …
dbgui-status-halt = Angehalten bei ${ $pc } { $ort }
dbgui-status-laeuft = Läuft
dbgui-status-startet = Startet …
dbgui-status-wartet = Wartet auf den Emulator
dbgui-text-eingabe = Text an den Emulator senden
dbgui-text-senden = Senden
dbgui-treffer = Treffer
dbgui-werkzeuge = Debug-Werkzeuge
dbgui-zeichensatz-bei = Zeichensatz bei ${ $a }
dbgui-zeichensatz-rom = Zeichensatz im ROM
dbgui-zeichensatz-unbekannt = Zeichensatz unbekannt
hexui-16 = 16 Byte
hexui-4kb = 4 KB je Seite
hexui-8 = 8 Byte
hexui-adr = ADR
hexui-anfang = Zum Anfang
hexui-ansicht = Hex-Ansicht
hexui-auswahl = Auswahl
hexui-bereich = Bereich
hexui-breite = Bytes je Zeile
hexui-byte = Byte
hexui-byte-ansage = Adresse { $off }, Wert { $v }
hexui-bytes = Byte
hexui-bytes-ab = { $n } Byte ab ${ $off }
hexui-datei = Datei
hexui-dateien = Binärdateien
hexui-ende = Zum Ende
hexui-extern = Im Standardprogramm öffnen
hexui-ganze-datei = Die ganze Datei ist geladen.
hexui-gedeutet = Gedeutet als
hexui-gefunden = Gefunden bei { $off }
hexui-kein-projekt = Kein Projekt geöffnet.
hexui-keine-dateien = Keine Binärdateien im Projekt.
hexui-ladeadresse = Ladeadresse ${ $a }
hexui-leer-text = Links eine Datei wählen. Die Hex-Ansicht zeigt Dateien nur an und verändert sie nie.
hexui-leer-titel = Keine Datei gewählt
hexui-liste = Hex-Ansicht, { $n } Byte
hexui-naechste = Nächste Seite
hexui-nichts-gefunden = Nichts gefunden.
hexui-nichts-gewaehlt = Ein Byte anklicken oder mit den Pfeiltasten wählen.
hexui-nur-erste = Gezeigt werden die ersten { $mib } MiB von { $n } Byte.
hexui-nur-lesen = Nur lesen
hexui-seite-label = Seitenwahl
hexui-speicheradresse = Speicheradresse ${ $a }
hexui-suche = Suchen: Bytes ($A9 00) oder Text
hexui-text = TEXT
hexui-text-deutung = Text
hexui-vorige = Vorige Seite
hexui-weiter-suchen = Suchen
hexui-wort = Wort (klein vorn)
spk-adresse = Adresse
spk-alle = Alle
spk-art = Art
spk-belegt = Vom Programm belegt
spk-bereiche = Bereiche
spk-byte = Byte
spk-ea = E/A
spk-filter = Filter
spk-frei = Freies RAM
spk-hinweis = Pfeiltasten wählen eine Seite. Der Balken im Kästchen zeigt, wie viel das Programm dort belegt.
spk-kaestchen = Jedes Kästchen = 256 Byte
spk-karte = Speicherkarte
spk-karte-label = Speicherkarte, 256 Seiten
spk-leer-text = Die Karte entsteht aus dem letzten Bau. Erst bauen ({ $taste }).
spk-leer-titel = Noch nichts gebaut
spk-programm = Programm
spk-programm-belegt = Vom Programm belegt
spk-programm-titel = Programm
spk-pruefungen = Prüfungen
spk-seite = Seite ${ $p }
spk-seite-ansage = Seite { $p }, { $kat }, { $used } Byte belegt
spk-seite-titel = Gewählte Seite
spk-umschaltung = Bankumschaltung
err-einst-ungueltig = Die Einstellungen sind ungültig: { $detail }
err-einst-schreiben = Die Einstellungen ließen sich nicht speichern: { $detail }
err-einst-import = Die Einstellungsdatei ließ sich nicht lesen: { $detail }
einst-assembler = Assembler
einst-assembler-eingebaut = Eingebaut (vc-asm). Kein weiteres Programm nötig.
einst-basic-dialekt = BASIC-Dialekt je Zielsystem
einst-basic-hinweis = Das BASIC eines Projekts folgt dem Zielsystem. Zeilennummern sind optional (Projekt ▸ Quellen).
einst-bilder-fragen = Fragen
einst-bilder-immer = Immer laden
einst-bilder-nie = Nie laden
einst-darstellung-zurueck = Darstellung zurückgesetzt.
einst-darstellung-zuruecksetzen = Darstellung zurücksetzen (Farbschema, Zoom, Editor)
einst-datei = Einstellungsdatei
einst-datei-zeigen = Im Ordner zeigen
einst-debuglog = Debug-Protokoll in eine Datei schreiben
einst-debuglog-hinweis = Hilft bei der Fehlersuche. Die Datei bleibt auf diesem Rechner und wird nie gesendet.
einst-fehlersuche = Fehlersuche
einst-hilfe = Hilfe
einst-hilfe-bilder = Handbuchbilder online laden
einst-hilfe-url = Ort der Handbuchbilder
einst-hilfe-url-leer = leer = Standardadresse
einst-include-suche = Include-Suche: 1. Ordner des Projekts, 2. zusätzliche Ordner (-I), 3. Systemdateien von VintageCoder.
einst-kuerzel = Tastenkürzel
einst-kuerzel-fest = Die Kürzel sind fest. Diese Liste hilft beim Finden.
einst-kuerzel-keine = Nichts gefunden.
einst-kuerzel-suche = Befehl oder Kürzel suchen
einst-laedt = Lädt …
einst-menueleiste = Menüleiste im Fenster
einst-menueleiste-mac = Unter macOS steht die Menüleiste des Systems zur Verfügung.
einst-menueleiste-win = Die Menüleiste im Fenster ist eingeschaltet.
einst-pretty = Zeile beim Verlassen formatieren
einst-pretty-hinweis = Assembler und BASIC: die bearbeitete Zeile wird ausgerichtet, sobald du sie verlässt.
einst-projektordner = Projektordner
einst-projektordner-hinweis = Hier legt „Neues Projekt“ neue Projekte vorgeschlagen ab. Leer = Voreinstellung.
einst-schema = Farbschema
einst-schema-dunkel = Dunkel
einst-schema-hell = Hell
einst-schema-kontrast = Hoher Kontrast
einst-schema-system = System
einst-schrift = Editor-Schrift
einst-schrift-groesser = Schrift größer
einst-schrift-kleiner = Schrift kleiner
einst-seite-allgemein = Allgemein
einst-seite-aussehen = Aussehen
einst-seite-editor = Editor
einst-seite-kuerzel = Tastenkürzel
einst-seite-sicherung = Sicherung
einst-seite-toolchain = Toolchain
einst-seiten = Einstellungsseiten
einst-speichern = Speichern
einst-sprache = Sprache
einst-sprache-auto = Automatisch
einst-sprache-hinweis = Englisch ist maschinell übersetzt und noch nicht geprüft.
einst-titel = Einstellungen
einst-umbruch = Zeilenumbruch
einst-ungespeichert = Ungespeicherte Änderungen
einst-upd-fragen = Noch nicht gefragt
einst-upd-hinweis = Es wird nichts ohne deine Zustimmung gesendet. „Nie“ prüft nur auf Wunsch.
einst-upd-nie = Nur auf Wunsch
einst-upd-taeglich = Täglich
einst-upd-url = Adresse der Versionsliste
einst-upd-url-leer = leer = Standard
einst-upd-woechentlich = Wöchentlich
einst-updates = Updates
einst-verwerfen = Verwerfen
einst-waehlen = Wählen …
einst-zoom = Zoom
einst-zoom-groesser = Größer
einst-zoom-hinweis = Cmd/Strg + Plus, Minus und 0 stellen den Zoom ein.
einst-zoom-kleiner = Kleiner
konfig-exportieren = Konfiguration exportieren …
konfig-exportiert = Exportiert nach { $path } (ohne Passwörter).
konfig-hinweis = Zielsysteme, Emulatoren, Maschinen, Hilfe und Update-Wahl lassen sich sichern und auf einem anderen Rechner übernehmen. Passwörter der Maschinen sind nie dabei.
konfig-import-frage = Die Datei enthält { $z } Zielsysteme und { $m } Maschinen. Die aktuellen Einstellungen werden ersetzt.
konfig-importieren = Konfiguration importieren …
konfig-passwoerter-bleiben = Passwörter bereits bekannter Maschinen bleiben erhalten.
konfig-reset-frage = Alle Einstellungen gehen auf die Vorgabe zurück, auch Zielsysteme, Emulatoren, Maschinen und ihre Passwörter. Das lässt sich nicht rückgängig machen.
konfig-titel = Sicherung
konfig-uebernehmen = Übernehmen
konfig-uebernommen = Übernommen aus { $path }.
konfig-zurueckgesetzt = Einstellungen auf die Vorgabe zurückgesetzt.
konfig-zuruecksetzen = Auf Vorgabe zurücksetzen …
einst-seite-ziele = Ziele und Hardware
einst-seite-roms = ROMs
roms-byte = Byte
roms-dateien = { $n ->
    [one] 1 Datei
   *[other] { $n } Dateien
 }
roms-erwartet = Erwartete Größe in Byte: { $n }
roms-fehlt-optional = Fehlt (optional)
roms-fehlt-ort = nicht vorhanden
roms-fremde = ROMs anderer Programme mitbenutzen (VICE, Xemu, atari800)
roms-fremde-hinweis = Nur lesend; es wird nichts kopiert oder verändert. Standard: aus.
roms-neu-pruefen = Neu prüfen
roms-ordner-oeffnen = ROM-Ordner öffnen
roms-orte = Durchsuchte Orte
roms-systeme = Systeme
roms-woher = VintageCoder liefert keine ROMs mit. Lege deine ROM-Dateien in den ROM-Ordner; sie werden über Prüfsummen erkannt.
ziele-art = Art des Emulators
ziele-baud = Baudrate (leer = 2000000)
ziele-emu-hinzu = Emulator hinzufügen …
ziele-emulatoren = Emulatoren
ziele-entfernen = Entfernen
ziele-eth-ungeprueft = Der Ethernet-Weg ist nur gegen Attrappen getestet, nicht an echter Hardware.
ziele-ethernet = Ethernet
ziele-gefunden-auto = Gefunden: { $p }
ziele-geraet = Serielle Schnittstelle
ziele-hardware = Echte Hardware
ziele-hinzu = Hinzufügen
ziele-host = Adresse (192.168.1.50 oder host:port)
ziele-hw-hinzu = { $name } anbinden
ziele-kein-system = Kein Zielsystem gewählt.
ziele-keine-emulatoren = Noch kein Emulator eingetragen.
ziele-name = Name
ziele-nicht-gefunden = Nicht gefunden. Pfad von Hand eintragen.
ziele-passwort = Netzwerkpasswort (leer = keines)
ziele-pfad = Programm (leer = automatisch suchen)
ziele-pfad-auto = automatisch
ziele-pw-verbergen = Verbergen
ziele-pw-zeigen = Zeigen
ziele-reset-vor = Maschine vor der Übertragung zurücksetzen
ziele-rom = ROM (atari800: Betriebssystem, Xemu: Ordner mit MEGA65.ROM)
ziele-run-nach = Nach der Übertragung RUN ausführen
ziele-schnittstelle = Netzwerkschnittstelle des Rechners
ziele-sd-ordner = Ordner auf der SD-Karte
ziele-standard = Standard
ziele-start-emulator = Emulator (Standard des Systems)
ziele-start-hinweis = Das Startziel gilt für F5 und den Start-Knopf dieses Systems.
ziele-startziel = Startziel
ziele-suchen = Suchen
ziele-system-hinzu = System hinzufügen …
ziele-system-weg = System entfernen
ziele-systeme = Zielsysteme
ziele-testen = Testen
ziele-testet = Testet …
ziele-usb = USB
ziele-verbindung = Verbindung
ziele-verbindung-testen = Verbindung testen
einst-seite-info = Info und Update
info-aenderungen = Änderungen ansehen
info-aktuell = Du hast die neueste Version.
info-betreff = Betreff
info-drittanbieter = Drittanbieter
info-feedback = Feedback
info-feedback-hinweis = Das Feedback wird als Mail vorbereitet. Gesendet wird erst in deinem Mailprogramm; der Text enthält nur technische Angaben, keine Dateien und keine Namen.
info-feedback-titel = Feedback senden
info-format-warnung = Achtung: Die neue Version schreibt ein höheres Projektformat. Ältere Versionen können die Projekte danach nicht mehr lesen.
info-gekuerzt = Der Text wurde für den Mail-Link gekürzt; „Kopieren“ übernimmt den vollständigen Text.
info-hinweise-auf = Hinweise anzeigen
info-hinweise-zu = Hinweise ausblenden
info-jetzt-pruefen = Jetzt nach Updates suchen
info-kein-netz = Diese Seite ruft von selbst nichts aus dem Netz ab.
info-kopieren = In die Zwischenablage
info-kopieren-fehlt = Die Zwischenablage ist hier nicht erreichbar. Den Text markieren und von Hand kopieren.
info-kopiert = In die Zwischenablage kopiert.
info-letzte-pruefung = Letzte Prüfung { $t }: { $r }
info-mail-geoeffnet = Mailprogramm geöffnet. Gesendet wird erst dort.
info-mail-oeffnen = Mail vorbereiten
info-neue-version = Neue Version { $v } verfügbar
info-nie-geprueft = Noch nie nach Updates gesucht.
info-prueft = Prüft …
info-quelltext = Quelltext
info-spaeter = Später erinnern
info-text = Text
info-ueber = Über VintageCoder
info-ueberspringen = Diese Version überspringen
info-update-hinweis = Update { $v } verfügbar
info-update-hinweis-text = Es wird nur die Versionsliste abgefragt; nichts wird geladen oder installiert. Die Häufigkeit stellst du unter Allgemein ein.
info-webseite = Webseite
info-zurueckgezogen = Die installierte Version wurde zurückgezogen. Bitte aktualisieren.
err-wiz-fehler = Der Assistent meldet: { $detail }
err-wiz-kein-ordner = Es gibt keinen Ordner für die Einstellungen.
err-wiz-keine-sitzung = Der Assistent ist nicht geöffnet.
wiz-abbrechen = Abbrechen
wiz-aenderungen-text = Das wird geändert. Haken weg = nicht übernehmen. Vor dem Schreiben wird eine Sicherung der Einstellungen angelegt.
wiz-beispiele = Beispiele
wiz-beispiele-fehlen = In dieser Installation liegen keine Beispiele.
wiz-beispiele-kopieren = Beispielprojekte der gewählten Systeme kopieren
wiz-beispiele-kopiert = { $n ->
    [one] 1 Beispiel kopiert.
   *[other] { $n } Beispiele kopiert.
 }
wiz-beispiele-ziel = Ziel: { $p }
wiz-einrichtung = Einrichtung
wiz-einrichtung-text = Der Assistent richtet Zielsysteme, Emulatoren, ROMs und Hardware ein. Er installiert nichts und lädt nichts herunter.
wiz-emulatoren-text = Emulatoren werden gesucht; trag den Pfad von Hand ein, wenn einer fehlt.
wiz-fertig-spaeter = Alles lässt sich jederzeit unter Einstellungen oder über Hilfe ▸ Assistent ändern.
wiz-fertig-text = Die Einrichtung ist fertig.
wiz-hardware-text = Echte Maschinen (Ultimate, MEGA65) sind freiwillig. Verbindungen lassen sich hier testen.
wiz-hinweise = Hinweise
wiz-keine-installation = Der Assistent installiert nichts und lädt nichts herunter. Du kannst jederzeit abbrechen; geschrieben wird erst am Ende.
wiz-nichts-zu-aendern = Es gibt nichts zu ändern. Alles steht schon so.
wiz-rueckgaengig = Rückgängig
wiz-schliessen = Schließen
wiz-schnell = Schnelleinrichtung (7 Schritte)
wiz-schritte = Schritte des Assistenten
wiz-sicherung = Sicherung der alten Einstellungen: { $p }
wiz-spaeter = Später fortsetzen
wiz-systeme-text = Für welche Systeme willst du programmieren? Abgewählte Systeme verschwinden aus der Oberfläche, ihre Einträge bleiben erhalten.
wiz-teil-design = Darstellung und Editor …
wiz-teil-emulatoren = Emulatoren …
wiz-teil-hardware = Hardware …
wiz-teil-roms = ROMs …
wiz-titel = Einrichtung
wiz-uebernehmen = Übernehmen
wiz-uebernommen = Übernommen.
wiz-unvollstaendig = Die Einrichtung ist noch nicht abgeschlossen.
wiz-voll = Alles selbst einstellen (12 Schritte)
wiz-weiter = Weiter
wiz-willkommen-text = Willkommen bei VintageCoder. Die Schnelleinrichtung fragt nur das Nötigste; wer alles selbst einstellen will, nimmt den langen Weg.
wiz-zurueck = Zurück
view-title-hilfe = Hilfe
hilfe-bilder-einmal = Nur dieses Mal
hilfe-bilder-frage = Handbuchbilder laden
hilfe-bilder-immer = Immer laden
hilfe-bilder-nie = Nie laden
hilfe-bilder-text = Die Bilder des Handbuchs liegen im Netz. Sollen sie geladen werden? Dabei wird nur das Bild angefragt, nichts von dir gesendet.
hilfe-kein-eintrag = Zu „{ $w }“ gibt es keinen Hilfeeintrag.
hilfe-nichts-gefunden = Nichts gefunden.
hilfe-suche = Hilfe durchsuchen: Themen, Befehle, Direktiven, Register …
hilfe-themen = Hilfethemen
hilfe-willkommen = Hilfe und Handbuch
hilfe-willkommen-text = Links ein Thema wählen oder suchen. F1 im Editor schlägt das Wort unter dem Cursor nach.
hilfe-zurueck = Zurück
startui-alle = Alle
startui-beispiel-sub = Lauffähige Spiele und Demos
startui-beispiele-sub = lauffähig auf echter Hardware und im Emulator
startui-frei-verwendbar = Mit VintageCoder erzeugte Dateien und die Syntax von VCASM dürfen frei verwendet werden.
startui-n-projekte = { $n ->
    [one] 1 Projekt
   *[other] { $n } Projekte
 }
startui-neues-sub = Aus Vorlage für ein Zielsystem
startui-oeffnen-sub = Ordner oder Projektdatei wählen
startui-system-filter = Beispiele nach System filtern
einst-assembler-optionen = Assembler-Optionen beim Bauen
ziele-args-titel = Startparameter
args-an = an
args-aus = aus
args-auf-standard = Auf Standard zurücksetzen
args-beispiel = Beispiel: { $x }
args-bereich = Bereich: { $a } bis { $b }
args-braucht = Braucht: { $x }
args-katalog = Katalog: { $l }. Nicht gesetzte Parameter bleiben beim Programm auf ihrem Standard.
args-kein-katalog = Für diese Sorte gibt es keinen Parameterkatalog; das freie Feld genügt.
args-migrierbar = { $n } Parameter im freien Feld kennt der Katalog.
args-n-gesetzt = { $n } gesetzt
args-nicht-geprueft = Achtung: dieser Katalog ist nicht gegen das Programm geprüft.
args-nicht-zusammen = Nicht zusammen mit: { $x }
args-standard = Standard
args-standard-des-programms = Standard des Programms: { $d }
args-uebernehmen = In die Auswahl übernehmen
args-variable-einfuegen = Variable einfügen:
args-vorschau = Vorschau (Beispielprojekt „demo“):
args-weitere = Weitere Parameter
args-zuruecksetzen-eins = Zurücksetzen
einst-startbild = Startbild
einst-startbild-an = Startbild beim Programmstart zeigen
einst-tasten = Tasten
einst-einzeltasten = Einzeltasten der Editoren
einst-einzeltasten-hinweis = Buchstaben und Ziffern wählen Werkzeuge und tragen Noten ein. Aus: nur Kürzel mit Cmd/Strg oder Alt wirken.
einst-tasten-klaviatur = Computertasten auf der Klaviatur anzeigen
einst-tastenlayout-hinweis = Die Tasten gelten nach ihrer Lage auf der Tastatur, daher ist keine Wahl des Tastaturlayouts nötig.
splash-text = Assembler-Entwicklung für 8-Bit-Computer
startui-willkommen = Willkommen bei VintageCoder
startui-willkommen-text = Die Einrichtung sucht Assembler und Emulatoren, wählt deine Zielsysteme und legt Beispiele an. Sie dauert wenige Minuten und lässt sich jederzeit wiederholen.
startui-willkommen-einrichten = Einrichtung starten
startui-willkommen-spaeter = Später
kopierdlg-system = Zielsystem der Kopie
kopierdlg-system-wie-original = Wie das Original
kopierdlg-system-hinweis = Kopie für ein anderes System angelegt. Der Code wurde nicht umgesetzt: bitte Quellen selbst anpassen.
kopierdlg-system-hinweis-lang = Das gewählte System wird Normalziel der Kopie. Der Code wird nicht umgesetzt; die Quellen bleiben unverändert. Zielsystemabhängig sind zum Beispiel Zeropage- und Speicheradressen, Sprite- und Zeichensatzformate und die Palette.
include-oeffnen = Eingebundene Datei öffnen
include-im-baum-zeigen = Im Projektbaum zeigen
include-nicht-gefunden = Datei nicht gefunden: {$path}
include-altpfad = Der Altpfad {$path} (build/export) existiert nicht mehr: bitte die Mediendatei selbst einbinden (W214).
roms-import = ROMs übernehmen
roms-datei-waehlen = ROM-Datei oder Zip wählen …
roms-fremde-suchen = ROMs anderer Programme suchen
roms-liste-kopieren = Fundliste kopieren
roms-import-hinweis = Es wird nichts ohne Bestätigung kopiert. Dateien lassen sich auch auf das Fenster ziehen. ROMs werden nie mitgeliefert und nie heruntergeladen.
roms-import-nichts = In der Auswahl wurde nichts Erkennbares gefunden.
roms-import-nichts-fremd = In den bekannten Orten anderer Programme wurden keine erkannten ROMs gefunden.
roms-import-haken = {$name} übernehmen
roms-import-kopieren = wird kopiert
roms-import-da = schon vorhanden
roms-import-ausfuehren = {$n} übernehmen
roms-import-fertig = {$n} ROM(s) in den ROM-Ordner kopiert.
roms-fest-gespeichert = Auswahl in roms.json gespeichert.
roms-liste-kopiert = Fundliste in die Zwischenablage kopiert.
roms-fest-label = Festgelegte Datei für {$rolle}
roms-fest-auto = Automatisch
ziele-installiert = Installiert: {$pfad}
ziele-laedt-und-installiert = Lädt und installiert …
ziele-laedt-frage = {$name} {$version} ({$mb} MB) wird von {$host} geladen, geprüft und nach {$nach} entpackt. Es wird erst nach „Jetzt laden“ etwas geladen.
ziele-jetzt-laden = Jetzt laden
ziele-portabel-installieren = Portabel installieren ({$version}) …
err-emu-install-fehler = Die Installation ist fehlgeschlagen: {$detail}
err-roms-fehler = Die ROM-Auswahl konnte nicht gespeichert werden: {$detail}
wiz-erstes-text = Ein Projekt ist der schnellste Weg, VintageCoder auszuprobieren.
wiz-erstes-neu = Neues Projekt anlegen …
wiz-erstes-beispiel = Beispiel öffnen: {$name}
startui-neues-erkannt = Neues erkannt
startui-neues-text = Seit der letzten Einrichtung wurde gefunden: {$liste}.
startui-neues-einrichten = Emulatoren einrichten
startui-neues-merken = Als bekannt merken
startui-neues-aus = Nicht mehr zeigen
projekt-gruppe-art = Art
projekt-art-asm = Assembler
projekt-art-basic = BASIC
projekt-gruppe-dialekt = Dialekt
projekt-gruppe-ziele = Ziele
projekt-gruppe-entfernen = Gruppe entfernen
projekt-quelle-entfernen = {$f} aus den Quellen entfernen
projekt-datei-hinzu = Datei hinzufügen (Pfad im Projekt)
projekt-hinzufuegen = Hinzufügen
projekt-gruppe-neu-asm = Neue Gruppe: Assembler
projekt-gruppe-neu-basic = Neue Gruppe: BASIC
projekt-illegale = Undokumentierte Befehle
projekt-illegale-auto = Standard (Warnung)
projekt-illegale-deny = Fehler
projekt-illegale-warn = Warnung
projekt-illegale-allow = Erlaubt
projekt-startziel = Startziel (F5)
projekt-startziel-global = Wie in den Einstellungen
projekt-startziel-emulator = Emulator
projekt-abbild = Diskettenabbild beim Bauen
projekt-abbild-format = Format
projekt-abbild-name = Name auf der Diskette
projekt-abbild-weg = Kein Abbild
projekt-abbild-hinweis = Ausgabe: {$datei}. Die Programme der Quellen kommen auf das Abbild.
projekt-abbild-aus = Es wird kein Diskettenabbild erzeugt.
projekt-abbild-neu = {$f}-Abbild erzeugen
dbgui-chips = Chips
dbgui-chips-keine = Für dieses Zielsystem gibt es keine Chipbeschreibung.
dbgui-test-startet = Test „{$name}“ wird im Debugger vorbereitet.
tests-debuggen = Diesen Test debuggen
dbgui-test-sitzung = Test-Sitzung
status-debug-pc = Debug: angehalten bei ${$pc}
machine-debug = Debug-Sitzung
machine-flags = Flags
md-keine-markdown-datei = Das ist keine Markdown-Datei.
md-hier-nichts = Hier gibt es dafür nichts zu tun.
md-link-geoeffnet = Adresse im Browser geöffnet: {$url}
md-bild-fehlt = Bild nicht gefunden oder zu groß
md-bild = Bild
md-aufgabe = Aufgabe abhaken
md-vorschau-label = Markdown-Vorschau
md-leer = Die Datei ist leer.
md-mehr = {$n} weitere Blöcke anzeigen
md-vorschau = Vorschau
md-quelltext = Quelltext
md-leiste = Markdown
md-ansicht = Ansicht
bau-abgestuerzt = Der Bau ist intern abgestürzt ({ $was }). Das Programm läuft weiter; bitte noch einmal bauen.
# Kontext: Fehlercode extern-ausfuehrbar (Datei würde beim Öffnen ein Programm ausführen; die Oberfläche fragt nach)
err-extern-ausfuehrbar = { $rel } ist ein ausführbares Programm. Öffnen würde es starten.
rf-papierkorb-titel = Papierkorb nicht verfügbar
rf-papierkorb-text = „{$name}“ konnte nicht in den Papierkorb gelegt werden und wurde nicht gelöscht. Endgültig löschen? Das lässt sich nicht rückgängig machen.
rf-endgueltig-loeschen = Endgültig löschen
rf-programm-titel = Diese Datei ist ein Programm
rf-programm-text = „{$name}“ ist ein Programm. Trotzdem öffnen? Es würde gestartet.
rf-trotzdem-oeffnen = Trotzdem öffnen
err-fileops-papierkorb-fehlgeschlagen = „{$rel}“ konnte nicht in den Papierkorb gelegt werden. Die Datei wurde nicht gelöscht.
# Kontext: Fehlercode fileops-papierkorb-fehlgeschlagen (Datei bleibt; die Oberfläche fragt nach endgültigem Löschen)
