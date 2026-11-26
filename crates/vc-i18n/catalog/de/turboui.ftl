# SPDX-License-Identifier: GPL-3.0-or-later
# Texte der Oberfläche zu Turbo BASIC XL (Konzept „Turbo BASIC XL starten“, Etappe E3): Karte in den Projekt-Einstellungen, Assistent „Diskette bereitstellen“, Banner.
# Die Rechtstexte (Schlüssel rechtlich-…) stehen wörtlich nach „Rechtstexte-Turbo-und-Erweiterungen“; Änderungen nur nach Rücksprache mit dem Inhaber.

# Kontext: Fehlercode turbo-fehler (Text kommt fertig aus dem Kern, vc_ide::turbo)
err-turbo-fehler = { $detail }

turboui-name-unbekannt = Turbo BASIC XL
turboui-stufe-titel = Turbo BASIC XL
turboui-datei-waehlen-titel = Turbo-BASIC-Diskette wählen
turboui-filter-diskabbild = Diskettenabbild

# Rechtstexte (Entwurf, vom Inhaber freigegeben; der Anwalt prüft später)
rechtlich-turbo-assistent = Turbo BASIC XL ist kein Bestandteil von VintageCoder und muss vom Anwender beigesteuert werden. Besorge dir eine Diskette mit Turbo BASIC XL aus einer Quelle, die du nutzen darfst, und wähle sie hier.
rechtlich-turbo-export-nachher = Das Projekt enthält die Turbo-BASIC-XL-Diskette.

# Karte in den Projekt-Einstellungen
# veraltet seit P1 (Extra-Reiter „Projekt-Einstellungen“ entfallen, Bereich Projekt)
turboui-projekt-titel = Projekt-Einstellungen
turboui-projekt-ohne-turbo = Dieses Projekt hat keine besonderen Einstellungen für Turbo BASIC XL. Die Karte erscheint bei Projekten mit dem BASIC-Dialekt „Turbo BASIC XL“ für den Atari.
turboui-karte-titel = Turbo BASIC XL
turboui-zustand-fehlt = Diskette fehlt
turboui-zustand-bereit = Diskette bereit
turboui-zustand-datei-fehlt = Die Diskette { $disk } ist eingetragen, aber die Datei fehlt.
turboui-zustand-name = { $name }, Version { $version }
turboui-zustand-name-ohne-version = { $name }
turboui-zustand-frei = { $count } Sektoren frei (etwa { $kb } KB)
turboui-zustand-reduziert-ja = Verkleinert auf die Systemdateien
turboui-zustand-reduziert-nein = Nicht verkleinert (alle Dateien der Diskette sind noch da)
turboui-zustand-veraendert = Die Kopie hat sich seit dem Bereitstellen verändert (andere Prüfsumme).
turboui-zustand-nicht-turbo = Die Kopie sieht nicht nach Turbo BASIC XL aus (du verwendest sie auf eigene Verantwortung).
turboui-zustand-kopie = Kopie im Projekt: { $disk }
turboui-knopf-bereitstellen = Bereitstellen …
turboui-knopf-ersetzen = Ersetzen …
turboui-knopf-pruefen = Prüfen
turboui-knopf-verkleinern = Verkleinern …
turboui-knopf-disk-editor = Im Disk-Editor öffnen
turboui-feld-programm = Programmname auf der Diskette
turboui-feld-programm-hinweis = Turbo BASIC XL startet nur AUTORUN.BAS von selbst. Ein anderer Name startet nicht automatisch.
turboui-feld-programm-fehler = „{ $name }“ ist kein gültiger Name (1–8 Zeichen A–Z, 0–9, _, erstes ein Buchstabe, Endung bis 3 Zeichen).
turboui-zusatz-titel = Zusatzdateien auf der Diskette
turboui-zusatz-hinweis = Eine Datei je Zeile als Quelle oder Quelle:NAME (z. B. daten/hallo.dat:HALLO.DAT). Die Quelle liegt im Projekt, der Name ist ein 8.3-Name auf der Diskette.
turboui-zusatz-leer = Keine Zusatzdateien.
turboui-zusatz-neu = Neue Zeile (Quelle oder Quelle:NAME)
turboui-zusatz-hinzufuegen = Hinzufügen
turboui-zusatz-entfernen = Entfernen: { $zeile }
turboui-zusatz-fehlt = Die Quelle fehlt im Projekt: { $zeile }
turboui-speichern = Speichern
turboui-gespeichert = Einstellungen für Turbo BASIC XL gespeichert.
turboui-pruefung-titel = Ergebnis der Prüfung
turboui-dateien-titel = Dateien auf der Diskette ({ $count })
turboui-datei-zeile = { $name }, { $sectors } Sektoren
turboui-datei-gesperrt = schreibgeschützt
turboui-hinweis-kein-lieferumfang = Turbo BASIC XL ist kein Bestandteil von VintageCoder und muss vom Anwender beigesteuert werden. Die Diskette bleibt auf deinem Rechner und wird nirgends hin übertragen.
turboui-hinweis-gitignore = Die Datei turbo/*.atr steht in der .gitignore des Projekts und gehört nicht in die Versionsverwaltung.
turboui-debuggen-hinweis = Debuggen: Turbo-BASIC-Programme lassen sich noch nicht schrittweise ausführen.
turboui-verkleinern-fertig = Verkleinert: { $count } Dateien entfernt, jetzt { $free } Sektoren frei.
turboui-geprueft = Diskette geprüft: { $text }

# Assistent „Turbo-BASIC-Diskette bereitstellen“
turboui-dlg-titel = Turbo-BASIC-Diskette bereitstellen
turboui-dlg-ersetzen-titel = Turbo-BASIC-Diskette ersetzen
turboui-dlg-schritt-datei = 1 · Datei wählen
turboui-dlg-schritt-pruefen = 2 · Prüfung
turboui-dlg-schritt-kopieren = 3 · Kopieren
turboui-dlg-datei-waehlen = Datei wählen …
turboui-dlg-keine-datei = Noch keine Datei gewählt.
turboui-dlg-quelle = Gewählt: { $path }
turboui-dlg-pruefung-laeuft = Prüfe …
turboui-dlg-befund-erkannt = { $name } erkannt. { $count } Sektoren frei (etwa { $kb } KB). Die Diskette wird in das Projekt kopiert ({ $dest }); dein Original bleibt unverändert.
turboui-dlg-befund-nicht-erkannt = Das sieht nicht nach Turbo BASIC XL aus (keine AUTORUN.SYS mit Kennung).
turboui-dlg-befund-kein-dos2 = Auf der Diskette gibt es kein DOS-2-Dateisystem.
turboui-dlg-befund-fehler = Fehler in der Diskette: { $m }
turboui-dlg-trotzdem-frage = Sieht nicht nach Turbo BASIC XL aus. Trotzdem verwenden?
turboui-dlg-trotzdem-ja = Ja, trotzdem verwenden
turboui-dlg-trotzdem-nein = Nein, andere Datei wählen
turboui-dlg-kopie-existiert = Im Projekt liegt schon die Kopie { $dest }. Sie wird ersetzt.
turboui-dlg-aktuelle-kopie = Aktuelle Kopie im Projekt: { $disk }. Sie wird durch die neue ersetzt.
turboui-dlg-verkleinern = Diskette auf die Systemdateien verkleinern
turboui-dlg-verkleinern-text = Nur in der Kopie im Projekt entfallen { $count } Dateien (zum Beispiel Beispielprogramme). Dann sind { $free } statt { $now } Sektoren frei, genug für große Programme. Dein Original bleibt unberührt. Du kannst das auch später auf der Karte tun.
turboui-dlg-verkleinern-bestaetigt = Ja, in der Projektkopie verkleinern
turboui-dlg-verkleinern-nein = Nicht verkleinern (die volle Kopie bleibt; zu große Programme melden den Fehler erst beim Bauen)
turboui-dlg-kopieren = Kopieren
turboui-dlg-abbrechen = Abbrechen
turboui-dlg-schliessen = Schließen
turboui-dlg-fertig = Fertig: { $disk } liegt im Projekt, { $free } Sektoren frei. Die .gitignore wurde ergänzt.
turboui-dlg-fertig-gitignore = Die .gitignore enthält jetzt turbo/*.atr.
turboui-dlg-fehler = Das Bereitstellen ist fehlgeschlagen: { $detail }

# Banner im Editor
turboui-banner-text = Dieses Projekt braucht eine Turbo-BASIC-XL-Diskette. Sie gehört nicht zum Lieferumfang.
turboui-banner-bereitstellen = Bereitstellen …
turboui-banner-schliessen = Banner schließen
turboui-banner-label = Hinweis zu Turbo BASIC XL

# Projektbaum, Statuszeile, Protokoll
turboui-marke-diskette = Diskette (Anwender)
turboui-marke-hinweis = Nicht einchecken: diese Diskette gehört dir und darf nicht jeder weitergeben.
turboui-status = Turbo-Diskette: { $name }, { $free } Sektoren frei
turboui-status-fehlt = Turbo-Diskette fehlt
turboui-log-bereitgestellt = Turbo-Diskette bereitgestellt: { $disk } ({ $free } Sektoren frei)
turboui-log-fehlt = Turbo BASIC XL: Diskette fehlt, bereitstellen über die Projekt-Einstellungen.
turboui-nur-turbo = Dieser Befehl gehört zu Projekten mit Turbo BASIC XL für den Atari.
turboui-disk-editor-bald = Der Disk-Editor wird in einer späteren Etappe gebaut. Die Diskette liegt unter { $disk }.
turboui-dlg-quelle-ist-kopie = Die gewählte Datei ist bereits die Kopie im Projekt ({ $disk }). Wähle dein Original.
