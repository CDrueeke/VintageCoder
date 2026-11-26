# SPDX-License-Identifier: GPL-3.0-or-later
# Texte der Oberfläche: diskdoc (aus crates/vc-ide-model/src/diskdoc.rs extrahiert, Wortlaut unverändert)

# Kontext: diskdoc.rs, Text
diskdoc-n-b = { $n } B

# Kontext: diskdoc.rs, Text
diskdoc-p1-kb = { $p1 } KB

# Kontext: diskdoc.rs, Text
diskdoc-1541-35-spuren-683-sektoren = 1541 · 35 Spuren · 683 Sektoren

# Kontext: diskdoc.rs, Text
diskdoc-1571-70-spuren-2-seiten-1366 = 1571 · 70 Spuren (2 Seiten) · 1366 Sektoren

# Kontext: diskdoc.rs, Text
diskdoc-1581-80-spuren-zu-40-sektoren = 1581 · 80 Spuren zu 40 Sektoren · 3200 Sektoren

# Kontext: diskdoc.rs, Text
diskdoc-boot-vtoc = Boot / VTOC

# Kontext: diskdoc.rs, Text
diskdoc-kopf-bam = Kopf / BAM

# Kontext: diskdoc.rs, Text
diskdoc-verzeichnis = Verzeichnis

# Kontext: diskdoc.rs, Text
diskdoc-belegt-aber-ohne-datei = belegt, aber ohne Datei

# Kontext: diskdoc.rs, Text
diskdoc-von-mehreren-ketten-erreicht = von mehreren Ketten erreicht

# Kontext: diskdoc.rs, Text
diskdoc-neu-nicht-gespeichert = (neu, nicht gespeichert)

# Kontext: diskdoc.rs, Text
diskdoc-sektor-p1 = Sektor { $p1 }

# Kontext: diskdoc.rs, Text
diskdoc-sektor-p1-p2 = Sektor { $p1 }/{ $p2 }

# Kontext: diskdoc.rs, Text
diskdoc-gehoert-zu-p1 = gehört zu { $p1 }

# Kontext: diskdoc.rs, Text
diskdoc-boot-bzw-vtoc = Boot bzw. VTOC

# Kontext: diskdoc.rs, Text
diskdoc-frei-laut-bam = frei laut BAM

# Kontext: diskdoc.rs, Text
diskdoc-belegt-ohne-datei = belegt ohne Datei

# Kontext: diskdoc.rs, Text
diskdoc-konflikt-mehrfach-belegt = Konflikt: mehrfach belegt

# Kontext: diskdoc.rs, Text
diskdoc-datei-nr-p1-folgesektor-p2-p3 = Datei-Nr. { $p1 } · Folgesektor { $p2 } · { $p3 } Datenbytes

# Kontext: diskdoc.rs, Text
diskdoc-kettenende-letztes-byte-p1 = Kettenende · letztes Byte { $p1 }

# Kontext: diskdoc.rs, Text
diskdoc-zeiger-p1-p2 = Zeiger { $p1 }/{ $p2 }

# Kontext: diskdoc.rs, Text
diskdoc-gespeichert-p1 = gespeichert: { $p1 }

# Kontext: diskdoc.rs, Text
diskdoc-boot-diskette-ohne-dateisystem-dateien-lassen = Boot-Diskette ohne Dateisystem: Dateien lassen sich hier nicht ablegen.

# Kontext: diskdoc.rs, Text
diskdoc-n-hinzugefuegt = »{ $n }« hinzugefügt

# Kontext: manuell
diskdoc-frei = frei

# Kontext: Disk-Editor, Diskettenspalte: Atari-Abbild mit erkanntem Dateisystem
diskdoc-atari-geometrie = { $dos } · { $n } Sektoren zu { $ssz } Byte

# Kontext: Disk-Editor, Diskettenspalte: Atari-Abbild ohne Dateisystem (Boot-Diskette)
diskdoc-atari-ohne-dos = Atari-Boot-Diskette (ohne Dateisystem) · { $n } Sektoren zu { $ssz } Byte

# Kontext: Disk-Editor, Diskettenspalte: Zusatz, wenn die Sektoren 1 bis 3 kürzer sind als die übrigen
diskdoc-atari-boot-128 =  (Sektoren 1–3 zu 128 Byte)
