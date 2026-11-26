# SPDX-License-Identifier: GPL-3.0-or-later
# Texte der Oberfläche: disk (aus crates/vc-disk/src/any.rs extrahiert, Wortlaut unverändert)

# Kontext: any.rs, Err
disk-name-unbekanntes-imageformat-erwartet-d64-d71 = { $name }: unbekanntes Imageformat (erwartet .d64, .d71, .d81 oder .atr)

# Kontext: any.rs, Text
disk-verzeichniskette-ist-ein-kreis = Verzeichniskette ist ein Kreis

# Kontext: atr.rs, Err
disk-ungueltiger-dateiname-name-1-8-zeichen = ungültiger Dateiname '{ $name }' (1–8 Zeichen A–Z/0–9/_, erstes ein Buchstabe, Erweiterung bis 3 Zeichen)

# Kontext: atr.rs, Err
disk-name-leere-dateien-werden-nicht-unterstuetzt = '{ $name }': leere Dateien werden nicht unterstützt

# Kontext: atr.rs, Err
disk-name-existiert-bereits = '{ $name }' existiert bereits

# Kontext: atr.rs, Err
disk-diskette-voll-name-braucht-need-sektoren = Diskette voll: '{ $name }' braucht { $need } Sektoren, frei sind { $p3 }

# Kontext: atr.rs, ok_or
disk-verzeichnis-voll-64-dateien = Verzeichnis voll (64 Dateien)

# Kontext: atr.rs, ok_or
disk-name-nicht-gefunden = '{ $name }' nicht gefunden

# Kontext: atr.rs, Err
disk-sektor-s-ausserhalb-der-diskette = Sektor { $s } außerhalb der Diskette

# Kontext: atr.rs, Err
disk-sektor-s-gehoert-nicht-zu-name = Sektor { $s } gehört nicht zu '{ $name }' (Dateinummer { $p3 })

# Kontext: atr.rs, Err
disk-sektor-s-ungueltige-byteanzahl-p2 = Sektor { $s }: ungültige Byteanzahl { $p2 }

# Kontext: atr.rs, Err
disk-sektorkette-laenger-als-im-verzeichnis = Sektorkette länger als im Verzeichnis

# Kontext: atr.rs, Err
disk-sektorkette-hat-count-statt-expect-sektoren = Sektorkette hat { $count } statt { $expect } Sektoren

# Kontext: atr.rs, Err
disk-kein-atr-kennung-0296-fehlt = kein ATR (Kennung $0296 fehlt)

# Kontext: atr.rs, Text
disk-p1-sektor-s-doppelt-belegt = { $p1 }: Sektor { $s } doppelt belegt

# Kontext: atr.rs, Text
disk-sektor-s-ist-reserviert-aber-als = Sektor { $s } ist reserviert, aber als frei markiert

# Kontext: atr.rs, Text
disk-sektor-s-ist-belegt-aber-als = Sektor { $s } ist belegt, aber als frei markiert

# Kontext: atr.rs, Text
disk-sektor-s-ist-als-belegt-markiert = Sektor { $s } ist als belegt markiert, gehört aber zu keiner Datei

# Kontext: atr.rs, Text
disk-vtoc-zaehler-p1-stimmt-nicht-mit = VTOC-Zähler { $p1 } stimmt nicht mit der Bitmap ({ $free }) überein

# Kontext: boot.rs, Err
disk-abgeschnittener-segmentkopf = abgeschnittener Segmentkopf

# Kontext: boot.rs, Err
disk-segment-s-e-ende-vor-start = Segment ${ $s }-${ $e }: Ende vor Start

# Kontext: boot.rs, Err
disk-segment-s-e-ist-abgeschnitten = Segment ${ $s }-${ $e } ist abgeschnitten

# Kontext: boot.rs, Err
disk-keine-segmente = keine Segmente

# Kontext: boot.rs, Err
disk-keine-xex-datei-ffff-kennung-fehlt = keine XEX-Datei ($FFFF-Kennung fehlt)

# Kontext: boot.rs, Err
disk-segment-s-e-ueberlappt-den-boot = Segment ${ $s }-${ $e } überlappt den Boot-Loader ($0700-$08FF)

# Kontext: boot.rs, Err
disk-xex-p1-bytes-passt-nicht-auf = XEX ({ $p1 } Bytes) passt nicht auf die Diskette (max. { $max } Bytes)

# Kontext: boot.rs, Err
disk-boot-loader-ist-p1-bytes-gross = Boot-Loader ist { $p1 } Bytes groß (max. 384)

# Kontext: boot.rs, Err
disk-unerwartetes-loader-layout = unerwartetes Loader-Layout

# Kontext: cbm.rs, Err
disk-id-muss-aus-genau-2-ascii = ID muss aus genau 2 ASCII-Zeichen bestehen

# Kontext: cbm.rs, Err
disk-diskette-voll-name-braucht-need-bloecke = Diskette voll: '{ $name }' braucht { $need } Blöcke, frei sind { $p3 }

# Kontext: cbm.rs, ok_or
disk-verzeichnis-voll = Verzeichnis voll

# Kontext: cbm.rs, ok_or
disk-keine-freien-sektoren = keine freien Sektoren

# Kontext: cbm.rs, Err
disk-ungueltiger-sektor-t-s = ungültiger Sektor { $t }/{ $s }

# Kontext: cbm.rs, Err
disk-sektorkette-zu-lang-schleife = Sektorkette zu lang (Schleife?)

# Kontext: cbm.rs, Err
disk-ungueltiges-ende-in-t-s = ungültiges Ende in { $t }/{ $s }

# Kontext: cbm.rs, Err
disk-kette-hat-n-bloecke-verzeichnis-meldet = Kette hat { $n } Blöcke, Verzeichnis meldet { $p2 }

# Kontext: cbm.rs, ok_or
disk-endung-d71-oder-d81-erwartet = Endung .d71 oder .d81 erwartet

# Kontext: cbm.rs, Text
disk-p1-ungueltiger-sektor-t-s = { $p1 }: ungültiger Sektor { $t }/{ $s }

# Kontext: cbm.rs, Text
disk-p1-sektor-t-s-doppelt-belegt = { $p1 }: Sektor { $t }/{ $s } doppelt belegt

# Kontext: cbm.rs, Text
disk-t-s-belegt-aber-in-der = { $t }/{ $s }: belegt, aber in der BAM frei

# Kontext: cbm.rs, Text
disk-t-s-in-der-bam-belegt = { $t }/{ $s }: in der BAM belegt, gehört aber zu keiner Datei

# Kontext: cbm.rs, Text
disk-spur-t-bam-zaehler-p2-bitmap = Spur { $t }: BAM-Zähler { $p2 } ≠ Bitmap { $free }

# Kontext: d64.rs, Err
disk-name-s-1-16-zeichen-erforderlich = Name '{ $s }': 1–16 Zeichen erforderlich

# Kontext: d64.rs, Err
disk-name-s-zeichen-c-nicht-erlaubt = Name '{ $s }': Zeichen '{ $c }' nicht erlaubt

# Kontext: d64.rs, Err
disk-nur-d64-mit-35-spuren-ohne = nur D64 mit 35 Spuren ohne Fehlerinfo ({ $size } Bytes) unterstützt, Datei hat { $p2 }

# Kontext: lib.rs, Text
disk-und-p1-weitere-probleme = … und { $p1 } weitere Probleme

# Kontext: cbm.rs, Err
disk-kind-ohne-fehlerinfo-hat-bytes-datei = { $kind } ohne Fehlerinfo hat { $size } Bytes, Datei hat { $len }

# Kontext: atr.rs, Err: Sektorgröße im ATR-Kopf
disk-atr-sektorgroesse-unbekannt = Sektorgröße { $ssz } wird nicht unterstützt (möglich sind 128 und 256 Byte)

# Kontext: atr.rs, Err: Größe im ATR-Kopf passt nicht zur Dateilänge
disk-atr-groesse-passt-nicht = Die Größe im ATR-Kopf ({ $size } Byte) passt nicht zu den Sektordaten ({ $len } Byte)

# Kontext: atr.rs, Err: Schreiben auf andere Dateisysteme als Atari DOS 2
disk-schreiben-nur-dos2 = Dateien ablegen geht zurzeit nur auf Atari-DOS-2-Disketten (dieses Abbild: { $dos })

# Kontext: atr.rs, Err: MyDOS-Abbilder mit mehr als 1023 Sektoren
disk-atr-ketten-ueber-1023-sektoren = Dateien auf Abbildern mit mehr als 1023 Sektoren lassen sich noch nicht lesen

# Kontext: d64.rs/cbm.rs, Text (Zeiger der Verzeichniskette zeigt aus der Diskette heraus)
disk-verzeichnis-zeiger-ungueltig = Verzeichniskette: Zeiger auf ungültigen Sektor { $t }/{ $s }

# Verzeichniseinträge umordnen
disk-eintrag-nummer-ungueltig = Die Nummer des Verzeichniseintrags gibt es nicht
disk-verschieben-kette-defekt = Die Sektorkette einer Datei ist beschädigt; das Verzeichnis wurde nicht umsortiert
disk-verschieben-nur-dos2 = Umordnen geht nur bei Atari DOS 2 und 2.5 mit 128-Byte-Sektoren
disk-verschieben-nur-hauptverzeichnis = Umordnen geht nur zwischen Einträgen des Hauptverzeichnisses
