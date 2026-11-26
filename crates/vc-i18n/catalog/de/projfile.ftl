# SPDX-License-Identifier: GPL-3.0-or-later
# Texte der Oberfläche: projfile (aus crates/vc-ide/src/projfile.rs extrahiert, Wortlaut unverändert)

# Kontext: projfile.rs, Text
projfile-projektname-fehlt-bitte-einen-namen-eingeben = Projektname fehlt: bitte einen Namen eingeben

# Kontext: projfile.rs, Text
projfile-kein-target-mindestens-ein-zielsystem-eintragen = Kein Target: mindestens ein Zielsystem eintragen

# Kontext: projfile.rs, Text
projfile-f-datei-fehlt = { $f }: Datei fehlt

# Kontext: projfile.rs, Text
projfile-f-mehrfach-als-quelle-eingetragen = { $f }: mehrfach als Quelle eingetragen

# Kontext: projfile.rs, Text
projfile-f-unbekannte-art-kind-asm-oder = { $f }: unbekannte Art '{ $kind }' (asm oder basic)

# Kontext: projfile.rs, Text
projfile-p1-datei-fehlt-abschnitt-assets = { $p1 }: Datei fehlt (Abschnitt assets)

# Kontext: projfile.rs, Text
projfile-layout-unbekannte-kategorie-k = layout: unbekannte Kategorie '{ $k }'

# Kontext: projfile.rs, Text
projfile-layout-d-ist-kein-gueltiger-ordner = layout: '{ $d }' ist kein gültiger Ordner (relativ, ohne ..)

# Kontext: projfile.rs, Text
projfile-folders-p1-ist-kein-gueltiger-ordner = folders: '{ $p1 }' ist kein gültiger Ordner

# Kontext: projfile.rs, Text
projfile-folders-unbekannte-rolle-p1-bei-p2 = folders: unbekannte Rolle '{ $p1 }' bei '{ $p2 }'

# Kontext: projfile.rs, Text
projfile-folders-target-t-bei-p2-ist = folders: Target '{ $t }' bei '{ $p2 }' ist nicht definiert

# Kontext: projfile.rs, Text
projfile-rules-muster-fehlt = rules: Muster fehlt

# Kontext: projfile.rs, Text
projfile-rules-unbekannte-kategorie-p1-bei-p2 = rules: unbekannte Kategorie '{ $p1 }' bei '{ $p2 }'

# Kontext: projfile.rs, Text
projfile-rules-p1-ist-kein-gueltiger-ordner = rules: '{ $p1 }' ist kein gültiger Ordner (bei '{ $p2 }')

# Kontext: projfile.rs, Text
projfile-main-m-ist-keine-quelldatei-des = main: '{ $m }' ist keine Quelldatei des Projekts

# Kontext: projfile.rs, Text
projfile-o-und-f-erzeugen-dieselbe-ausgabedatei = { $o } und { $f } erzeugen dieselbe Ausgabedatei (gleicher Dateiname)

# Kontext: projfile.rs, Text
projfile-disk-unbekanntes-format-p1-d64-d71 = disk: unbekanntes Format '{ $p1 }' (d64, d71, d81, atr)

# Kontext: projfile.rs, Text
projfile-disk-f-ist-keine-quelldatei-des = disk: '{ $f }' ist keine Quelldatei des Projekts

# Kontext: projfile.rs, Text
projfile-p-b = P{ $b }

# Kontext: projfile.rs, Err
projfile-diskette-keine-programme-ausgewaehlt = Diskette: keine Programme ausgewählt

# Kontext: projfile.rs, Text
projfile-art-e = { $art }: { $e }

# Kontext: projfile.rs, Err
projfile-diskettenformat-f-unbekannt-d64-d71-d81 = Diskettenformat '{ $f }' unbekannt (d64, d71, d81, atr)

# Kontext: projfile.rs, Text
projfile-out-rel-e = { $out_rel }: { $e }

# Kontext: manuell
projfile-quellgruppe-f-target-t-ist-nicht-definiert = Quellgruppe { $f }: Target '{ $t }' ist nicht definiert

# Kontext: manuell
projfile-rolle-quelltext = Quelltext

# Kontext: manuell
projfile-rolle-ressourcen = Ressourcen

# Kontext: manuell
projfile-rolle-daten = Daten

# Kontext: manuell
projfile-rolle-doku = Doku

# Kontext: manuell
projfile-rolle-tests = Tests

# Kontext: manuell
projfile-rolle-werkzeuge = Werkzeuge

# Kontext: manuell
projfile-diskette-nicht-erstellt-nicht-alle-programme = Diskette nicht erstellt: nicht alle Programme des Projekts wurden gebaut

# Kontext: safepath.rs, Text (Pfad aus der Projektdatei liegt nicht im Projektordner)
projfile-pfad-unzulaessig = Unzulässiger Pfad '{ $pfad }' in der Projektdatei: erlaubt sind nur relative Pfade innerhalb des Projektordners (kein '..', kein absoluter Pfad, kein Laufwerk, kein Link nach draußen)
