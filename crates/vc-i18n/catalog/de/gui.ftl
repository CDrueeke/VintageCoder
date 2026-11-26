# SPDX-License-Identifier: GPL-3.0-or-later
# Oberfläche: Fenster, Menüs, Statusleiste, Protokoll. Schlüsselschema <bereich>-<ort>-<zweck>, Schlüssel werden nie umbenannt.

# Titel des Fensters und des Programms
window-title = VintageCoder

# Obere Menüs der Menüleiste (Kennungen im Code bleiben deutsch und stabil)
# Anwendungsmenü der macOS-Menüleiste
menu-app-about = Über VintageCoder
menu-app-hide = VintageCoder ausblenden

menu-file = Datei
menu-edit = Bearbeiten
menu-view = Ansicht
menu-project = Projekt
menu-tree = Baum
menu-help = Hilfe

# Fuß des Dateibaums. Kontext: n = Zahl der Dateien, dirs = Zahl der Ordner
status-tree-summary = { $n } Datei(en) · { $dirs } Ordner

# Zusatz am Fuß des Dateibaums bei aktivem Filter
status-tree-filtered = (gefiltert)

# Protokollzeile nach einem fehlgeschlagenen Bau. Kontext: n = Zahl der Probleme
build-failed = Bauen fehlgeschlagen: { $n } Problem(e)

# Kontext: main.rs, Text
gui-diskette-d71 = Diskette D71

# Kontext: main.rs, Text
gui-diskette-d81 = Diskette D81

# Kontext: main.rs, Text
gui-diskette-atr = Diskette ATR

# Kontext: main.rs, Text
gui-diskette-d64 = Diskette D64

# Kontext: main.rs, Err
gui-rel-gibt-es-schon = { $rel } gibt es schon

# Kontext: main.rs, Err
gui-fuer-dieses-projekt-gibt-es-dafuer = Für dieses Projekt gibt es dafür kein Profil.

# Kontext: main.rs, Text
gui-angelegt-rel = Angelegt: { $rel }

# Kontext: main.rs, log.push
gui-bildschirm-kachel-und-tracker-dateien-oeffnen = Bildschirm-, Kachel- und Tracker-Dateien öffnen sich mit ihren Editoren (folgt).

# Kontext: main.rs, Text
gui-ordner-angelegt-r = Ordner angelegt: { $r }/

# Kontext: main.rs, Text
gui-nur-der-name-kein-pfad-verschieben = Nur der Name, kein Pfad: Verschieben geht per Ziehen im Baum.

# Kontext: main.rs, Text
gui-umbenannt-alt-neu = Umbenannt: { $alt } → { $neu }

# Kontext: main.rs, Text
gui-p1-rel = { $p1 } { $rel }

# Kontext: main.rs, log.push
gui-projekt-wechseln-erst-die-debug-sitzung = Projekt wechseln: erst die Debug-Sitzung beziehungsweise den Bau beenden.

# Kontext: main.rs, Text
gui-projekt-geoeffnet-p1 = Projekt geöffnet: { $p1 }

# Kontext: main.rs, Text
gui-rel-sicherung-fehlgeschlagen-nichts-umgewandelt-e = { $rel }: Sicherung fehlgeschlagen, nichts umgewandelt: { $e }

# Kontext: main.rs, Text
gui-kachelkarte-umgewandelt-sicherung-der-alten-datei = Kachelkarte umgewandelt; Sicherung der alten Datei: { $p1 }

# Kontext: main.rs, Text
gui-rel-umwandlung-fehlgeschlagen-e = { $rel }: Umwandlung fehlgeschlagen: { $e }

# Kontext: main.rs, Text
gui-pc-p1 = · PC ${ $p1 }

# Kontext: main.rs, log.push
gui-ein-ordner-laesst-sich-nicht-in = Ein Ordner lässt sich nicht in sich selbst verschieben.

# Kontext: main.rs, log.push
gui-verschieben-erst-alle-geaenderten-dateien-speichern = Verschieben: erst alle geänderten Dateien speichern (Pfade in Quelltexten werden angepasst).

# Kontext: main.rs, Text
gui-quelle-nichts-zu-verschieben = { $quelle }: nichts zu verschieben

# Kontext: main.rs, log.push
gui-verschieben-braucht-eine-projektdatei-vcprj = Verschieben braucht eine Projektdatei (.vcprj).

# Kontext: main.rs, Text
gui-verschoben-quelle-neu-p3 = Verschoben: { $quelle } → { $neu }{ $p3 }

# Kontext: main.rs, Text
gui-verschieben-fehlgeschlagen-e = Verschieben fehlgeschlagen: { $e }

# Kontext: main.rs, log.push
gui-es-gibt-nichts-rueckgaengig-zu-machen = Es gibt nichts rückgängig zu machen.

# Kontext: main.rs, log.push
gui-rueckgaengig-erst-alle-geaenderten-dateien-speichern = Rückgängig: erst alle geänderten Dateien speichern.

# Kontext: main.rs, Text
gui-verschieben-rueckgaengig-gemacht-n-datei-en = Verschieben rückgängig gemacht ({ $n } Datei(en))

# Kontext: main.rs, Text
gui-rel-keine-textdatei-oeffnet-im-standardprogramm = { $rel }: keine Textdatei, öffnet im Standardprogramm

# Kontext: main.rs, Text
gui-e-wird-nicht-ueberschrieben = { $e } (wird nicht überschrieben)

# Kontext: main.rs, Text
gui-rel-nicht-lesbar = ; { $rel }: nicht lesbar

# Kontext: main.rs, Text
gui-braucht-eine-debug-sitzung-und-einen = braucht eine Debug-Sitzung und einen offenen Quelltext

# Kontext: main.rs, Text
gui-braucht-eine-debug-sitzung = braucht eine Debug-Sitzung

# Kontext: main.rs, Text
gui-braucht-ein-geoeffnetes-studio = braucht einen geöffneten Editor

# Kontext: main.rs, Text
gui-braucht-eine-markdown-datei = braucht eine Markdown-Datei

# Kontext: main.rs, Text
gui-braucht-ein-projekt = braucht ein Projekt

# Kontext: main.rs, Text
gui-braucht-einen-offenen-quelltext = braucht einen offenen Quelltext

# Kontext: main.rs, Text
gui-gerade-nicht-moeglich = gerade nicht möglich

# Kontext: main.rs, titel
gui-menue = ☰ Menü

# Kontext: main.rs, Text
gui-start = Start

# Kontext: main.rs, Text
gui-projekt = Projekt

# Kontext: main.rs, Text
gui-p1-ordner-p2-ebene-p3 = { $p1 }, Ordner, { $p2 }, Ebene { $p3 }

# Kontext: main.rs, Text
gui-p1-datei-ebene-p2 = { $p1 }, Datei, Ebene { $p2 }

# Kontext: main.rs, Text
gui-editor-n = Editor { $n }

# Kontext: main.rs, Text
gui-kein-dokument = Kein Dokument

# Kontext: main.rs, Text
gui-zeile-p1-p2 = Zeile { $p1 }: { $p2 }

# Kontext: main.rs, log.push
gui-keine-definition-gefunden = Keine Definition gefunden.

# Kontext: main.rs, Text
gui-definition-ausserhalb-des-projekts-uri = Definition außerhalb des Projekts: { $uri }

# Kontext: main.rs, Text
gui-kuerzel-menue-ausloesung-id-id = Kürzel: Menü-Auslösung id={ $id }

# Kontext: main.rs, Text
gui-kuerzel-id-als-doppelereignis-verworfen = Kürzel: { $id } als Doppelereignis verworfen

# Kontext: main.rs, set_title
gui-rmt-modul-waehlen = RMT-Modul wählen

# Kontext: main.rs, set_title
gui-song-aus-anderem-system-waehlen = Song aus anderem System wählen

# Kontext: main.rs, set_title
gui-png-bild-laden = PNG-Bild laden

# Kontext: main.rs, Text
gui-zu-a-gibt-es-keine-quellzeile = Zu ${ $a } gibt es keine Quellzeile: dort lässt sich kein Haltepunkt setzen.

# Kontext: main.rs, Text
gui-ziel-p1 = Ziel: { $p1 }

# Kontext: main.rs, Text
gui-gestartet-c = Gestartet: { $c }

# Kontext: main.rs, Text
gui-start-fehlgeschlagen-p1 = Start fehlgeschlagen: { $p1 }

# Kontext: main.rs, Text
gui-einzelheit-e = Einzelheit: { $e }

# Kontext: main.rs, Text
gui-n-ersetzt = { $n } ersetzt

# Kontext: main.rs, Text
gui-p1-nicht-lesbar = { $p1 }: nicht lesbar

# Kontext: main.rs, Text
gui-p1-von-n = { $p1 } von { $n }

# Kontext: main.rs, Text
gui-fehler = Fehler

# Kontext: main.rs, hinweis
gui-hinweis = Hinweis

# Kontext: main.rs, Text
gui-warnung = Warnung

# Kontext: main.rs, Text
gui-kein-quelltext-offen-hier-gibt-es = Kein Quelltext offen: hier gibt es nichts zum Einbinden

# Kontext: main.rs, Text
gui-eingebunden-p1-p2-p3-z-nimmt = Eingebunden: { $p1 }{ $p2 } ({ $p3 }+Z nimmt es zurück)

# Kontext: main.rs, Text
gui-diese-datei-ist-schon-eingebunden-zeile = Diese Datei ist schon eingebunden (Zeile { $p1 }) – nichts eingefügt

# Kontext: main.rs, Text
gui-zeile-zeile-hat-keinen-code-dort = Zeile { $zeile } hat keinen Code: dort kann nicht angehalten werden.

# Kontext: main.rs, set_title
gui-datei-oeffnen = Datei öffnen

# Kontext: main.rs, set_title
gui-projekt-oeffnen-projektdatei-vcprj = Projekt öffnen (Projektdatei .vcprj)

# Kontext: main.rs, set_title
gui-projektordner-oeffnen = Projektordner öffnen

# Kontext: main.rs, Text
gui-zeile-ist-schon-formatiert-oder-hier = Zeile ist schon formatiert (oder hier gibt es keine Formatierung)

# Kontext: main.rs, Text
gui-datei-ist-schon-formatiert-oder-hier = Datei ist schon formatiert (oder hier gibt es keine Formatierung)

# Kontext: main.rs, Text
gui-n-p2-p3-gilt-beim-speichern = { $n }: { $p2 } · { $p3 } (gilt beim Speichern)

# Kontext: main.rs, log.push
gui-nach-neuem-gesucht-rom-verzeichnis-neu = Nach Neuem gesucht: ROM-Verzeichnis neu eingelesen, Emulatorsuche im Assistenten.

# Kontext: main.rs, Text
gui-keine-vorschlaege-an-dieser-stelle = Keine Vorschläge an dieser Stelle

# Kontext: main.rs, Text
gui-keine-probleme = Keine Probleme

# Kontext: main.rs, Text
gui-pfad-kopiert-p = Pfad kopiert: { $p }

# Kontext: main.rs, log.push
gui-gehe-zu-definition-nur-in-assembler = Gehe zu Definition: nur in Assembler-Dateien.

# Kontext: main.rs, Text
gui-projekt-einstellungen = Projekt-Einstellungen

# Kontext: main.rs, Text
gui-vintagecoder-beenden = VintageCoder beenden?

# Kontext: main.rs, Text
gui-projekt-wechseln = Projekt wechseln?

# Kontext: main.rs, text
gui-diese-reiter-haben-ungespeicherte-aenderungen = Diese Reiter haben ungespeicherte Änderungen:

# Kontext: main.rs, text
gui-unbenannt-wird-in-der-zwischenkopie-gesichert = (unbenannt, wird in der Zwischenkopie gesichert)

# Kontext: main.rs, knopf
gui-speichern = Speichern

# Kontext: main.rs, knopf
gui-verwerfen = Verwerfen

# Kontext: main.rs, knopf
gui-abbrechen = Abbrechen

# Kontext: main.rs, Text
gui-notiz = Notiz

# Kontext: main.rs, Text
gui-unbenannt = Unbenannt

# Kontext: main.rs, ok_or
gui-reiter-nicht-gefunden = Reiter nicht gefunden

# Kontext: main.rs, Text
gui-gespeichert-rel = Gespeichert: { $rel }

# Kontext: main.rs, text
gui-letzte-sitzung-wiederherstellen = Letzte Sitzung wiederherstellen?

# Kontext: main.rs, text
gui-beim-letzten-beenden-waren-unbenannte-reiter = Beim letzten Beenden waren unbenannte Reiter ungespeichert. Die Zwischenkopie liegt nur auf diesem Rechner, unverschlüsselt, und verfällt nach 14 Tagen.

# Kontext: main.rs, text
gui-p1-p2-zeichen-p3 = • { $p1 } · { $p2 } Zeichen · { $p3 }

# Kontext: main.rs, knopf
gui-wiederherstellen = Wiederherstellen

# Kontext: main.rs, knopf
gui-nicht-jetzt = Nicht jetzt

# Kontext: main.rs, Text
gui-gespeichert-n = Gespeichert: { $n }

# Kontext: main.rs, Text
gui-n-nicht-gespeichert-die-datei-war = { $n }: nicht gespeichert, die Datei war nicht lesbar und würde überschrieben

# Kontext: main.rs, Text
gui-speichern-fehlgeschlagen-er = Speichern fehlgeschlagen: { $er }

# Kontext: main.rs, log.push
gui-keine-quelldatei-zum-bauen = Keine Quelldatei zum Bauen.

# Kontext: main.rs, Text
gui-bauen-rel = Bauen: { $rel } …

# Kontext: main.rs, Text
gui-fehler-e = Fehler: { $e }

# Kontext: main.rs, Text
gui-lang-spruenge-automatisch-verlaengert-branchlong-auto = { $lang } Sprünge automatisch verlängert (.branchlong auto)

# Kontext: main.rs, Text
gui-fertig-p1-bytes-p2 = Fertig: { $p1 } Bytes → { $p2 }

# Kontext: main.rs, Text
gui-disk-p1-p2-bytes-p3 = Disk { $p1 } ({ $p2 } Bytes): { $p3 }

# Kontext: main.rs, Text
gui-diskette-e = Diskette: { $e }

# Kontext: main.rs, log.push
gui-nichts-zum-starten-erst-bauen = Nichts zum Starten: erst bauen.

# Kontext: main.rs, Text
gui-starten-auf-p1 = Starten auf { $p1 } …

# Kontext: main.rs, Text
gui-starten-im-emulator = Starten im Emulator …

# Kontext: main.rs, Err
gui-testlauf-kein-emulator-gestartet = Testlauf: kein Emulator gestartet

# Kontext: main.rs, Text
gui-emulator = Emulator

# Kontext: main.rs, Text
gui-emulator-p1 = Emulator ({ $p1 })

# Kontext: main.rs, text
gui-sname-p2 = { $sname } · { $p2 } ▾

# Kontext: main.rs, log.push
gui-nichts-zum-debuggen-erst-bauen = Nichts zum Debuggen: erst bauen.

# Kontext: main.rs, log.push
gui-testlauf-keine-debug-sitzung-gestartet = Testlauf: keine Debug-Sitzung gestartet

# Kontext: main.rs, Text
gui-debuggen-e = Debuggen: { $e }

# Kontext: main.rs, Text
gui-p1-debuggen-geht-nur-ueber-usb = { $p1 }: Debuggen geht nur über USB (Ethernet bietet keinen Monitor).

# Kontext: main.rs, log.push
gui-debug-sitzung-startet = Debug-Sitzung startet …

# Kontext: main.rs, log.push
gui-debug-sitzung-beendet = Debug-Sitzung beendet

# Kontext: main.rs, Text
gui-einzelheit-r = Einzelheit: { $r }

# Kontext: main.rs, log.push
gui-die-tests-laufen-parallel-zur-debug = Die Tests laufen parallel zur Debug-Sitzung (eigener Emulator).

# Kontext: main.rs, Text
gui-debug-sitzung-fuer-den-test-name = Debug-Sitzung für den Test „{ $name }“ startet …

# Kontext: main.rs, Text
gui-studio-p1 = editor { $p1 }

# Kontext: main.rs, Text
gui-zuletzt-geoeffnet = zuletzt geöffnet

# Kontext: main.rs, Text
gui-debuggen = ● Debuggen

# Kontext: main.rs, text
gui-name-tasten = { $name }   { $tasten }

# Kontext: main.rs, Text
gui-kein-projekt = Kein Projekt

# Kontext: main.rs, text
gui-projekt-2 = Projekt ▾

# Kontext: main.rs, Text
gui-starten = Starten

# Kontext: main.rs, text
gui-debuggen-2 = Debuggen

# Kontext: main.rs, Text
gui-bauen = Bauen

# Kontext: main.rs, Text
gui-mehr = Mehr

# Kontext: main.rs, text
gui-suchen = Suchen

# Kontext: main.rs, Text
gui-ziel-p1-p2 = Ziel: { $p1 } · { $p2 }

# Kontext: main.rs, Text
gui-plus-4 = Plus/4

# Kontext: main.rs, Text
gui-atari = Atari

# Kontext: main.rs, text
gui-kurz = { $kurz } ▾

# Kontext: main.rs, text
gui-update-v-verfuegbar = Update { $v } verfügbar

# Kontext: main.rs, text
gui-baut = baut …

# Kontext: main.rs, text
gui-z-z-s-s-p3 = Z { $z } · S { $s }   { $p3 }

# Kontext: main.rs, Text
gui-kodierung-beim-speichern-utf-8-iso = Kodierung beim Speichern: UTF-8 ↔ ISO-8859-1 (Klick wechselt)

# Kontext: main.rs, Text
gui-zeilenende-beim-speichern-lf-crlf-klick = Zeilenende beim Speichern: LF ↔ CRLF (Klick wechselt)

# Kontext: main.rs, text
gui-p1-name = { $p1 }{ $name }

# Kontext: main.rs, text
gui-vintagecoder = VintageCoder

# Kontext: main.rs, text
gui-version-p1 = Version { $p1 }

# Kontext: main.rs, text
gui-klick-oder-taste-schliesst-das-startbild = Klick oder Taste schließt das Startbild

# Kontext: main.rs, text
gui-name-p2 = { $name }{ $p2 }

# Kontext: main.rs, text
gui-teilung-aufheben-cmd-alt-t = Teilung aufheben (Cmd+Alt+T)

# Kontext: main.rs, text
gui-editor-teilen-zweite-haelfte-rechts-cmd = Editor teilen: zweite Hälfte rechts (Cmd+Alt+T)

# Kontext: main.rs, text
gui-markdown-vorschau = Markdown-Vorschau

# Kontext: main.rs, knopf
gui-quelltext = Quelltext

# Kontext: main.rs, knopf
gui-zum-editor-wechseln-befehlspalette-markdown-vorschau = Zum Editor wechseln (Befehlspalette: Markdown: Vorschau/Quelltext)

# Kontext: main.rs, text
gui-markdown-quelltext = Markdown-Quelltext

# Kontext: main.rs, knopf
gui-vorschau = Vorschau

# Kontext: main.rs, knopf
gui-zur-gerenderten-ansicht-wechseln = Zur gerenderten Ansicht wechseln

# Kontext: main.rs, knopf
gui-aa = Aa ✓

# Kontext: main.rs, knopf
gui-aa-2 = Aa

# Kontext: main.rs, text
gui-gehe-zu-zeile = Gehe zu Zeile

# Kontext: main.rs, text_input
gui-zeilennummer = Zeilennummer

# Kontext: main.rs, text
gui-keine-probleme-2 = Keine Probleme.

# Kontext: main.rs, knopf_k
gui-abbrechen-2 = ■ Abbrechen

# Kontext: main.rs, knopf_k
gui-alle-ausfuehren = ▶ Alle ausführen

# Kontext: main.rs, knopf_k
gui-fehlgeschlagene = Fehlgeschlagene

# Kontext: main.rs, knopf_k
gui-laeuft-auf-p1 = Läuft auf: { $p1 } ▾

# Kontext: main.rs, text_input
gui-filter = Filter

# Kontext: main.rs, label
gui-nur-fehler = nur Fehler

# Kontext: main.rs, text
gui-laeuft = läuft …

# Kontext: main.rs, text
gui-p1-bestanden-fehler-fehlgeschlagen-p3-uebersprungen = { $p1 } bestanden · { $fehler } fehlgeschlagen · { $p3 } übersprungen{ $p4 }

# Kontext: main.rs, text
gui-gemessen-m = gemessen: { $m }

# Kontext: main.rs, text
gui-a-p1-x-p2-y-p3 = A { $p1 }  X { $p2 }  Y { $p3 }  SP { $p4 }  PC { $p5 }

# Kontext: main.rs, knopf_k
gui-zur-quelle-rel-z = Zur Quelle ({ $rel }:{ $z })

# Kontext: main.rs, text
gui-enthaelt-geoeffnete-dateien = enthält geöffnete Dateien

# Kontext: main.rs, text
gui-das-projekt-ist-leer = Das Projekt ist leer.

# Kontext: main.rs, text_input
gui-dateien-filtern = Dateien filtern

# Kontext: main.rs, Text
gui-neue-datei-im-gewaehlten-ordner = Neue Datei im gewählten Ordner

# Kontext: main.rs, Text
gui-baum-neu-einlesen = Baum neu einlesen

# Kontext: main.rs, Text
gui-alle-ordner-zuklappen = Alle Ordner zuklappen

# Kontext: main.rs, text
gui-suche-laeuft = Suche läuft …

# Kontext: main.rs, Text
gui-eine-debug-sitzung-laeuft-beenden-und = Eine Debug-Sitzung läuft. Beenden und den Test „{ $n }“ debuggen?

# Kontext: main.rs, Text
gui-eine-debug-sitzung-laeuft-beenden-und-2 = Eine Debug-Sitzung läuft. Beenden und neu bauen und debuggen?

# Kontext: main.rs, Text
gui-reiter-schliessen = Reiter schließen

# Kontext: main.rs, Text
gui-n-hat-inhalt-und-ist-nicht = „{ $n }“ hat Inhalt und ist nicht gespeichert (Cmd+S legt ihn als Datei an). Verwerfen?

# Kontext: main.rs, Text
gui-aeltere-kachelkarte-umwandeln = Ältere Kachelkarte umwandeln?

# Kontext: main.rs, Text
gui-n-ist-eine-aeltere-kachelkarte-kind = „{ $n }“ ist eine ältere Kachelkarte (kind „tilemap“). Das Kachelsatz-Editor liest Kachelsatz und Bildschirm: Umwandeln legt beides neben der Datei an und öffnet den Bildschirm. Die alte Datei wird vorher unter .vcbackup/ gesichert und dann durch die neuen ersetzt.

# Kontext: main.rs, Text
gui-umwandeln = Umwandeln

# Kontext: main.rs, Text
gui-konfiguration-uebernehmen = Konfiguration übernehmen?

# Kontext: main.rs, Text
gui-die-datei-p1-enthaelt-z-die = Die Datei „{ $p1 }“ enthält: { $z }. Die jetzigen Einstellungen für Zielsysteme, Emulatoren, Maschinen und Parameter werden ersetzt; bekannte Maschinen behalten ihr Passwort. Die Oberflächeneinstellungen (Farbschema, Zoom) bleiben.

# Kontext: main.rs, Text
gui-uebernehmen = Übernehmen

# Kontext: main.rs, Text
gui-einstellungen-zuruecksetzen = Einstellungen zurücksetzen?

# Kontext: main.rs, Text
gui-alle-einstellungen-der-ide-zielsysteme-emulatoren = Alle Einstellungen der IDE (Zielsysteme, Emulatoren, Maschinen samt Passwörtern, Pfade, Parameter) gehen auf die Vorgabe zurück. Das lässt sich nicht rückgängig machen; vorher kann man die Konfiguration exportieren.

# Kontext: main.rs, Text
gui-zuruecksetzen = Zurücksetzen

# Kontext: main.rs, text
gui-ordner = Ordner

# Kontext: main.rs, Text
gui-speichern-unter = Speichern unter

# Kontext: main.rs, text
gui-rel-p2 = { $rel } { $p2 }

# Kontext: main.rs, text
gui-p1-name-2 = { $p1 } { $name }

# Kontext: main.rs, text
gui-wird-angelegt-rel = Wird angelegt: { $rel }

# Kontext: main.rs, Text
gui-alle = Alle

# Kontext: main.rs, Text
gui-programm = Programm

# Kontext: main.rs, titel
gui-p1-64-kb = { $p1 } · 64 KB

# Kontext: main.rs, text
gui-r-000 = ${ $r }000

# Kontext: main.rs, titel
gui-seite-p-p2-p3 = SEITE ${ $p } · ${ $p2 }–${ $p3 }

# Kontext: main.rs, mono
gui-programm-p1-von-256-byte = Programm: { $p1 } von 256 Byte

# Kontext: main.rs, mono
gui-n-a = { $n }  ${ $a }

# Kontext: main.rs, mono
gui-f-a-p3 = { $f }:{ $a }{ $p3 }

# Kontext: main.rs, titel
gui-pruefung = PRÜFUNG

# Kontext: main.rs, text
gui-w-hh-bild-p3-aufbau-p4 = { $w }×{ $hh } · Bild { $p3 } · Aufbau { $p4 } ms

# Kontext: main.rs, Text
gui-hilfe = Hilfe

# Kontext: main.rs, knopf
gui-im-reiter-oeffnen = Im Reiter öffnen

# Kontext: main.rs, knopf
gui-dieses-thema-im-dokumentenfeld-oeffnen-mehr = Dieses Thema im Dokumentenfeld öffnen (mehr Platz)

# Kontext: main.rs, text
gui-code-code2 = ${ $code } · { $code2 }

# Kontext: main.rs, Text
gui-datei = Datei

# Kontext: main.rs, Text
gui-projekt-n = Projekt: { $n }

# Kontext: main.rs, Text
gui-zuletzt-geoeffnet-2 = Zuletzt geöffnet

# Kontext: main.rs, Text
gui-studio-n = Editor: { $n }

# Kontext: main.rs, Text
gui-studio = Editor

# Kontext: main.rs, Text
gui-in-den-papierkorb-gelegt = In den Papierkorb gelegt:

# Kontext: main.rs, Text
gui-geloescht = Gelöscht:

# Kontext: main.rs, Text
gui-angepasst-pfad-e-in-quelltexten-angepasst = ({ $angepasst } Pfad(e) in Quelltexten angepasst)

# Kontext: manuell
gui-alternative-von-hand-a = Alternative von Hand: { $a }

# Kontext: manuell
gui-sid-datei-sid-goattracker-sng-oder-song = SID-Datei (.sid), GoatTracker (.sng) oder Song (.vcsong)

# Kontext: manuell
gui-vintagecoder-projekt = VintageCoder-Projekt

# Kontext: manuell
gui-aufgeklappt = aufgeklappt

# Kontext: manuell
gui-zugeklappt = zugeklappt
