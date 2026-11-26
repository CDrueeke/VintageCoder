# SPDX-License-Identifier: GPL-3.0-or-later
# English catalog; comments for translators are in the German file of the same name.

disk-name-unbekanntes-imageformat-erwartet-d64-d71 = { $name }: unknown image format (expected .d64, .d71, .d81 or .atr)
disk-verzeichniskette-ist-ein-kreis = Directory chain is a loop
disk-ungueltiger-dateiname-name-1-8-zeichen = Invalid file name '{ $name }' (1-8 characters A-Z/0-9/_, first one a letter, extension up to 3 characters)
disk-name-leere-dateien-werden-nicht-unterstuetzt = '{ $name }': empty files are not supported
disk-name-existiert-bereits = '{ $name }' already exists
disk-diskette-voll-name-braucht-need-sektoren = Disk full: '{ $name }' needs { $need } sectors, { $p3 } are free
disk-verzeichnis-voll-64-dateien = Directory full (64 files)
disk-name-nicht-gefunden = '{ $name }' not found
disk-sektor-s-ausserhalb-der-diskette = Sector { $s } is outside the disk
disk-sektor-s-gehoert-nicht-zu-name = Sector { $s } does not belong to '{ $name }' (file number { $p3 })
disk-sektor-s-ungueltige-byteanzahl-p2 = Sector { $s }: invalid byte count { $p2 }
disk-sektorkette-laenger-als-im-verzeichnis = Sector chain is longer than in the directory
disk-sektorkette-hat-count-statt-expect-sektoren = Sector chain has { $count } sectors instead of { $expect }
disk-kein-atr-kennung-0296-fehlt = Not an ATR (signature $0296 missing)
disk-p1-sektor-s-doppelt-belegt = { $p1 }: sector { $s } is used twice
disk-sektor-s-ist-reserviert-aber-als = Sector { $s } is reserved but marked as free
disk-sektor-s-ist-belegt-aber-als = Sector { $s } is in use but marked as free
disk-sektor-s-ist-als-belegt-markiert = Sector { $s } is marked as used but belongs to no file
disk-vtoc-zaehler-p1-stimmt-nicht-mit = VTOC counter { $p1 } does not match the bitmap ({ $free })
disk-abgeschnittener-segmentkopf = Truncated segment header
disk-segment-s-e-ende-vor-start = Segment ${ $s }-${ $e }: end before start
disk-segment-s-e-ist-abgeschnitten = Segment ${ $s }-${ $e } is truncated
disk-keine-segmente = No segments
disk-keine-xex-datei-ffff-kennung-fehlt = Not an XEX file ($FFFF signature missing)
disk-segment-s-e-ueberlappt-den-boot = Segment ${ $s }-${ $e } overlaps the boot loader ($0700-$08FF)
disk-xex-p1-bytes-passt-nicht-auf = XEX ({ $p1 } bytes) does not fit on the disk (max. { $max } bytes)
disk-boot-loader-ist-p1-bytes-gross = Boot loader is { $p1 } bytes (max. 384)
disk-unerwartetes-loader-layout = Unexpected loader layout
disk-id-muss-aus-genau-2-ascii = ID must consist of exactly 2 ASCII characters
disk-diskette-voll-name-braucht-need-bloecke = Disk full: '{ $name }' needs { $need } blocks, { $p3 } are free
disk-verzeichnis-voll = Directory full
disk-keine-freien-sektoren = No free sectors
disk-ungueltiger-sektor-t-s = Invalid sector { $t }/{ $s }
disk-sektorkette-zu-lang-schleife = Sector chain too long (loop?)
disk-ungueltiges-ende-in-t-s = Invalid end in { $t }/{ $s }
disk-kette-hat-n-bloecke-verzeichnis-meldet = Chain has { $n } blocks, directory reports { $p2 }
disk-endung-d71-oder-d81-erwartet = Extension .d71 or .d81 expected
disk-p1-ungueltiger-sektor-t-s = { $p1 }: invalid sector { $t }/{ $s }
disk-p1-sektor-t-s-doppelt-belegt = { $p1 }: sector { $t }/{ $s } is used twice
disk-t-s-belegt-aber-in-der = { $t }/{ $s }: in use but free in the BAM
disk-t-s-in-der-bam-belegt = { $t }/{ $s }: used in the BAM but belongs to no file
disk-spur-t-bam-zaehler-p2-bitmap = Track { $t }: BAM counter { $p2 } != bitmap { $free }
disk-name-s-1-16-zeichen-erforderlich = Name '{ $s }': 1-16 characters required
disk-name-s-zeichen-c-nicht-erlaubt = Name '{ $s }': character '{ $c }' not allowed
disk-nur-d64-mit-35-spuren-ohne = Only D64 with 35 tracks and no error info ({ $size } bytes) is supported, file has { $p2 }
disk-und-p1-weitere-probleme = ... and { $p1 } more problems
disk-kind-ohne-fehlerinfo-hat-bytes-datei = { $kind } without error info has { $size } bytes, file has { $len }
disk-atr-sektorgroesse-unbekannt = Sector size { $ssz } is not supported (128 and 256 bytes are possible)
disk-atr-groesse-passt-nicht = The size in the ATR header ({ $size } bytes) does not match the sector data ({ $len } bytes)
disk-schreiben-nur-dos2 = Adding files currently works only on Atari DOS 2 disks (this image: { $dos })
disk-atr-ketten-ueber-1023-sektoren = Files on images with more than 1023 sectors cannot be read yet

disk-verzeichnis-zeiger-ungueltig = Directory chain: pointer to invalid sector { $t }/{ $s }

# Reordering directory entries
disk-eintrag-nummer-ungueltig = There is no directory entry with this number
disk-verschieben-kette-defekt = The sector chain of a file is damaged; the directory was not reordered
disk-verschieben-nur-dos2 = Reordering only works with Atari DOS 2 and 2.5 and 128-byte sectors
disk-verschieben-nur-hauptverzeichnis = Reordering only works between entries of the main directory
