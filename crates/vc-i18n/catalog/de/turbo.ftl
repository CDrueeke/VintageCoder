# SPDX-License-Identifier: GPL-3.0-or-later
# Texte von Turbo BASIC XL starten (Diskette des Anwenders, Bauen, Starten): crates/vc-ide/src/turbo.rs, crates/vc-disk/src/atr.rs, crates/vc-cli/src/main.rs

# Kontext: Turbo-Diskette prüfen: Ergebnis
turbo-erkannt-label = { $label } erkannt.

# Kontext: Turbo-Diskette prüfen: Ergebnis
turbo-unbekanntes-abbild-version = Unbekanntes Abbild, aber mit der Kennung von Turbo BASIC XL (Version laut Text: { $version }).

# Kontext: Turbo-Diskette prüfen: Ergebnis
turbo-nicht-erkannt = Das sieht nicht nach Turbo BASIC XL aus (keine AUTORUN.SYS mit Kennung).

# Kontext: Turbo-Diskette prüfen: Ergebnis
turbo-sektoren-frei = { $count } Sektoren frei (≈ { $kb } KB).

# Kontext: Turbo-Diskette prüfen: Ergebnis
turbo-pruefsumme = SHA-256: { $sha }

# Kontext: Turbo-Diskette prüfen: Ergebnis; ein Stern kennzeichnet schreibgeschützte Dateien
turbo-dateien = { $count } Dateien: { $names } (* = schreibgeschützt)

# Kontext: Turbo-Diskette prüfen: Ergebnis
turbo-programm-vorhanden-gesperrt = { $name } ist schon vorhanden und schreibgeschützt (Bauen ist dann nicht möglich).

# Kontext: Turbo-Diskette prüfen: Ergebnis
turbo-programm-vorhanden = { $name } ist schon vorhanden (wird beim Bauen ersetzt).

# Kontext: Turbo-Diskette prüfen: Ergebnis
turbo-kein-dos2 = Kein DOS-2-Dateisystem auf der Diskette.

# Kontext: Turbo-Diskette prüfen: Problem im Abbild
turbo-problem = Problem: { $m }

# Kontext: Verkleinern der Kopie
turbo-reduzieren-systemdatei-fehlt = Verkleinern nicht möglich: { $name } fehlt auf der Diskette.

# Kontext: Verkleinern der Kopie
turbo-reduzieren-abbild-fehlerhaft = Verkleinern abgebrochen, das Ergebnis wäre fehlerhaft: { $m }

# Kontext: Kommentarzeile in der .gitignore des Projekts
turbo-gitignore-kommentar = Turbo-BASIC-XL-Diskette des Anwenders: nicht einchecken

# Kontext: Bereitstellen
turbo-keine-projektdatei = Keine Projektdatei in { $path }.

# Kontext: Bereitstellen
turbo-nicht-erkannt-trotzdem = Das sieht nicht nach Turbo BASIC XL aus (keine AUTORUN.SYS mit Kennung). Mit der Option für „trotzdem verwenden“ geht es auf eigene Verantwortung.

# Kontext: Bereitstellen
turbo-bereitstellen-fehler-in-der-diskette = Die Diskette hat Fehler im Dateisystem und wird nicht bereitgestellt: { $m }

# Kontext: Bereitstellen
turbo-ungueltiger-pfad = Ungültiger Pfad: { $path }

# Kontext: Bereitstellen
turbo-quelle-ist-die-kopie = Die Quelle ist schon die Kopie im Projekt ({ $disk }).

# Kontext: Bereitstellen
turbo-kopie-existiert-bereits = Die Kopie { $disk } gibt es schon (zum Überschreiben die Option zum Ersetzen angeben).

# Kontext: Bauen und Starten: Diskette fehlt
turbo-diskette-fehlt = Dieses Projekt braucht eine Turbo-BASIC-XL-Diskette. Sie gehört nicht zum Lieferumfang; stelle sie mit „vcc turbo provide“ bereit.

# Kontext: Bauen und Starten: Diskette fehlt
turbo-diskette-datei-fehlt = Die Turbo-BASIC-XL-Diskette { $disk } fehlt im Projekt. Sie gehört nicht zum Lieferumfang; stelle sie mit „vcc turbo provide“ neu bereit.

# Kontext: Projektdatei: Abschnitt turbo
turbo-ungueltiger-name = turbo: ungültiger Dateiname „{ $name }“ (1–8 Zeichen A–Z/0–9/_, erstes ein Buchstabe, Erweiterung bis 3 Zeichen).

# Kontext: Projektdatei: Abschnitt turbo
turbo-name-doppelt = turbo: Der Name { $name } kommt auf der Diskette doppelt vor.

# Kontext: Projektdatei: Abschnitt turbo
turbo-zusatzdatei-fehlt = turbo: Datei { $file } fehlt (Abschnitt files).

# Kontext: Protokollzeile nach dem Bau der Turbo-Diskette
turbo-bau-diskette = Diskette: { $file } (Turbo BASIC XL { $version }), Programm { $program } ({ $size } Bytes), { $free } Sektoren frei

# Kontext: Hinweis beim Bauen
turbo-kopie-veraendert = Die Kopie { $disk } wurde seit dem Bereitstellen verändert (Prüfsumme weicht ab).

# Kontext: Hinweis beim Bauen
turbo-hinweis-nicht-erkannt = { $disk } sieht nicht nach Turbo BASIC XL aus (Verwendung auf eigene Verantwortung).

# Kontext: Fehler beim Bauen
turbo-bauen-fehler-in-der-diskette = Die Diskette { $disk } hat Fehler im Dateisystem: { $m }

# Kontext: Fehler beim Löschen/Ersetzen einer Datei im ATR
turbo-datei-name-ist-schreibgeschuetzt = Die Datei { $name } ist schreibgeschützt.

# Kontext: Platzmangel beim Bau
turbo-platz = Das Programm samt Dateien braucht { $need } Sektoren, auf der Diskette sind { $free } frei.

# Kontext: Platzmangel beim Bau
turbo-platz-hinweis-reduzieren = Nach dem Verkleinern auf DOS.SYS, DUP.SYS und AUTORUN.SYS wären { $free } Sektoren frei: „vcc turbo reduce“ verkleinert die Kopie im Projekt (dein Original bleibt unverändert).

# Kontext: Platzmangel beim Bau
turbo-platz-hinweis-groesseres-abbild = Auch nach dem Verkleinern wäre es zu groß: ein größeres Abbild (130 KB) wird noch nicht unterstützt.

# Kontext: Starten
turbo-kein-turbo-projekt = Das Hauptprogramm des Projekts ist kein Turbo-BASIC-XL-Programm für den Atari.

# Kontext: Starten
turbo-bauen-fehlgeschlagen = Das Projekt konnte nicht gebaut werden.

# Kontext: Starten
turbo-programmende-hinweis = Nach dem Programmende (READY) wartet Turbo BASIC XL auf Eingaben; zum erneuten Start das Programm noch einmal starten.

# Kontext: Kommandozeile turbo
turbo-cli-bestaetigen-reduzieren = Die Kopie im Projekt ({ $disk }) wird auf DOS.SYS, DUP.SYS und AUTORUN.SYS verkleinert; { $count } andere Dateien fallen dort weg. Dein Original bleibt unverändert. Fortfahren? [j/N]

# Kontext: Kommandozeile turbo
turbo-cli-abgebrochen = Abgebrochen, nichts verändert (ohne Rückfrage geht es mit --yes).

# Kontext: Kommandozeile turbo
turbo-cli-bereitgestellt = Turbo-Diskette bereitgestellt: { $disk } ({ $free } Sektoren frei). Dein Original bleibt unverändert.

# Kontext: Kommandozeile turbo
turbo-cli-verkleinert = Kopie verkleinert: { $count } Dateien entfernt ({ $names }), { $free } Sektoren frei.

# Kontext: Kommandozeile turbo
turbo-cli-gitignore = turbo/*.atr steht jetzt in der .gitignore (die Diskette gehört nicht ins Repository).

# Kontext: Kommandozeile turbo
turbo-cli-gestartet = Gestartet: { $command }

# Kontext: Kommandozeile turbo
turbo-cli-nichts-zu-entfernen = Die Kopie besteht schon nur aus den Systemdateien, nichts zu entfernen.

# Kontext: Kommandozeile disk atr --base
turbo-cli-base-boot-konflikt = --base lässt sich nicht mit --boot verbinden.

# Kontext: Kommandozeile disk atr --base
turbo-cli-bearbeiten-braucht-base = --put, --delete und --force brauchen --base (das bestehende Abbild).

# Kontext: Kommandozeile disk atr --base
turbo-cli-disk-bearbeitet = { $out }: { $bytes } Bytes, { $free } Sektoren frei

# Kontext: Kommandozeile disk atr --base
turbo-cli-base-quelle-ist-ziel = Die Ausgabedatei darf nicht das Ausgangsabbild (--base) sein; das Original bleibt unverändert.

# Kontext: Prüfung eines ATR-Abbilds: Hinweis
turbo-warnung-eintrag-nicht-abgeschlossen = { $name }: Eintrag wurde nie abgeschlossen (Altlast im Verzeichnis, { $count } Sektoren in der Kette).

# Kontext: Turbo-Diskette prüfen: Hinweis auf eine Altlast (kein Fehler)
turbo-hinweis = Hinweis: { $m }

