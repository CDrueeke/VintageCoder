# SPDX-License-Identifier: GPL-3.0-or-later
# Meldungen der IDE-Schicht zu Maschinencode-Blobs (B114) und BASIC-Konvertierung (E330); Schlüssel diag-ide-<code>-<variante>
# Fehler B114: Blob-Datei nicht gefunden
# $name: Dateiname des Blobs | Beispiel: player.asm
diag-ide-b114-file-not-found = #asm: Datei '{ $name }' nicht gefunden
# Fehler B114: Blob-Datei nicht lesbar
# $path: Pfad der Datei | Beispiel: /home/u/p/player.asm
# $error: Fehlertext des Betriebssystems | Beispiel: Permission denied
diag-ide-b114-read-error = { $path }: { $error }
# Fehler B114: Blob erzeugt keinen Code
# $name: Dateiname des Blobs | Beispiel: player.asm
diag-ide-b114-no-code = #asm '{ $name }' erzeugt keinen Code
# Fehler B114: Blob beginnt nicht an der erwarteten Adresse
# $name: Dateiname des Blobs | Beispiel: player.asm
# $start: tatsächliche Startadresse (hex, ohne $) | Beispiel: C100
# $expected: erwartete Adresse (hex, ohne $) | Beispiel: C000
diag-ide-b114-org-mismatch = Code '{ $name }' muss mit `.org asm_base` beginnen (beginnt bei ${ $start }, erwartet ${ $expected })
# Fehler B114: Segmente eines Blobs überlappen
# $name: Dateiname des Blobs | Beispiel: player.asm
diag-ide-b114-overlap = #asm '{ $name }': überlappende Segmente
# Fehler B114: BASIC-Programm samt Maschinencode zu groß
# $end: Endadresse (hex, ohne $) | Beispiel: 10A00
diag-ide-b114-program-too-large = Programm samt Maschinencode zu groß (endet bei ${ $end })
# Fehler B114: Adresse des Maschinencodes konvergiert nicht
diag-ide-b114-address-unstable = Adresse des Maschinencodes wird nicht stabil
# Fehler B114: Maschinencode zu groß
# $bytes: Größe in Bytes | Beispiel: 20000
diag-ide-b114-code-too-large = Maschinencode zu groß ({ $bytes } Bytes)
# Fehler E330: Zielsystem der Konvertierung nicht umgesetzt
# $target: gewünschtes Ziel | Beispiel: c128
# $available: verfügbare Ziele, kommagetrennt | Beispiel: c64, atari
diag-ide-e330-target-unsupported = Ziel '{ $target }' ist noch nicht umgesetzt (verfügbar: { $available })
# Fehler E330: falsche Dateiendung der Quelle
diag-ide-e330-wrong-extension = Nur .bas-, .lst- oder .tur-Dateien (natives BASIC) lassen sich konvertieren
# Fehler E330: Datei nicht les- oder schreibbar
# $path: Pfad | Beispiel: spiel.bas
# $error: Fehlertext | Beispiel: Permission denied
diag-ide-e330-io = { $path }: { $error }
# Fehler E330: Ausgabedatei hat nicht die Endung .asm
# $file: Name der Ausgabedatei | Beispiel: spiel.txt
diag-ide-e330-output-extension = Ausgabedatei '{ $file }' muss auf .asm enden
# Fehler E330: Ausgabedatei existiert und ist nicht erzeugt
# $file: Name der Ausgabedatei | Beispiel: spiel.asm
diag-ide-e330-not-generated = '{ $file }' existiert und wurde nicht von basic2asm erzeugt – wird nicht überschrieben (anderen Namen wählen)
