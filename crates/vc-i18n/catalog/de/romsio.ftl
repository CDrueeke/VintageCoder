# SPDX-License-Identifier: GPL-3.0-or-later
# ROM-Import, ZIP-Leser und Eintragswerkzeug (vc-roms)

romsio-import-not-recognized = nicht erkannt
# $candidates: passende Rollen, mit Komma getrennt | Beispiel: kernal, basic
romsio-import-role-ambiguous = Rolle unklar (passt zu { $candidates })
romsio-import-checksum-unknown = Prüfsumme unbekannt (Größe passt zu { $candidates })
# $sizes: erwartete Größen, mit „ oder “ verbunden | Beispiel: 8192 oder 16384
romsio-import-wrong-size = falsche Größe (erwartet { $sizes } Byte)
romsio-import-or = { " oder " }
romsio-import-unsupported-system = gehört zu einem System, das VintageCoder nicht unterstützt (z. B. C65)
romsio-import-in-app-bundle = Zielordner liegt in einem App-Bundle
romsio-import-no-rom-dir = kein ROM-Ordner bekannt (kein Konfigurationsordner)
romsio-import-zip-no-roms = Archiv enthält keine erkannten ROM-Dateien
romsio-import-part-label = { $label }: { $role }
romsio-import-zip-too-large = Archiv zu groß ({ $mb } MB), wird nicht gelesen
romsio-import-never-bundle = nie in ein App-Bundle schreiben
romsio-import-no-target = kein Ziel
# $name: Dateiname des Archivs | Beispiel: roms.zip
romsio-import-archive-label = Archiv { $name }

romsio-zip-not-zip = kein ZIP-Archiv
romsio-zip-dir-damaged = ZIP: Verzeichnis beschädigt
# $n: Anzahl der Einträge | Beispiel: 5000
romsio-zip-too-many = ZIP: zu viele Einträge ({ $n })
romsio-zip-name-missing = ZIP: Name fehlt
romsio-zip-data-missing = ZIP: Daten fehlen

# $system: Systemkennung | Beispiel: c64
romsio-emit-unknown-system = unbekanntes System { $system }
# $system: Systemkennung | Beispiel: c64
# $role: Rollenkennung | Beispiel: kernal
romsio-emit-unknown-role = unbekannte Rolle { $system }/{ $role }
# $path: Dateipfad | Beispiel: kernal.bin
# $size: tatsächliche Größe in Byte | Beispiel: 100
# $expected: erwartete Größen | Beispiel: [8192]
# $system: Systemkennung | Beispiel: c64
# $role: Rollenkennung | Beispiel: kernal
romsio-emit-bad-size = { $path }: { $size } Byte, erwartet { $expected } für { $system }/{ $role }
# $t: der fehlerhafte Text | Beispiel: abc
romsio-emit-bad-number = Zahl '{ $t }' ungültig
# $spec: die Angabe nach --set | Beispiel: x:1
romsio-emit-set-format = --set { $spec }: name:offset:size:profile[:Text] erwartet
# $spec: die Angabe nach --set | Beispiel: x:1:2:p
# $len: Dateigröße in Byte | Beispiel: 100
romsio-emit-set-beyond-end = --set { $spec }: liegt hinter dem Dateiende ({ $len } Byte)
