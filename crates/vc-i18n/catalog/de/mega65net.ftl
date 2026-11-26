# SPDX-License-Identifier: GPL-3.0-or-later
# Texte der Oberfläche: mega65net (aus crates/vc-ide/src/mega65net.rs extrahiert, Wortlaut unverändert)

# Kontext: mega65net.rs, Text
mega65net-ethernet = Ethernet

# Kontext: mega65net.rs, Err
mega65net-p-nicht-gefunden-pfad-unter-einstellungen = { $p }: nicht gefunden (Pfad unter Einstellungen → Hardware prüfen)

# Kontext: mega65net.rs, ok_or_else
mega65net-exe-nicht-gefunden-pfad-unter-einstellungen = { $exe } nicht gefunden – Pfad unter Einstellungen → Hardware eintragen (Werkzeug aus dem Paket mega65-tools, Quelle: github.com/MEGA65/mega65-tools; die IDE liefert es nicht mit)

# Kontext: mega65net.rs, Text
mega65net-exe-e = { $exe }: { $e }

# Kontext: mega65net.rs, Err
mega65net-keine-antwort-innerhalb-von-p1-s = keine Antwort innerhalb von { $p1 } s; Ausgabe: { $p2 }

# Kontext: mega65net.rs, Err
mega65net-what-endete-mit-p2-p3-unchecked = { $what } endete mit { $p2 } ({ $p3 }) [{ $UNCHECKED }]
  { $p5 }

# Kontext: mega65net.rs, Err
mega65net-what-e-p3-unchecked = { $what }: { $e }
  { $p3 } [{ $UNCHECKED }]

# Kontext: mega65net.rs, Err
mega65net-ethernet-uebertragung-gibt-es-nur-fuer = Ethernet-Übertragung gibt es nur für den MEGA65

# Kontext: mega65net.rs, ok_or_else
mega65net-p1-kein-dateiname = { $p1 }: kein Dateiname

# Kontext: mega65net.rs, Err
mega65net-n-dateiname-mit-leerzeichen-oder-anfuehrungszeichen = { $n }: Dateiname mit Leerzeichen oder Anführungszeichen lässt sich nicht an mega65_ftp übergeben – bitte umbenennen

# Kontext: mega65net.rs, Text
mega65net-p1-programm-ueber-ethernet-p2-unchecked = { $p1 }: Programm über Ethernet { $p2 } ({ $UNCHECKED })
  { $exe } { $p5 }{ $p6 }

# Kontext: mega65net.rs, Err
mega65net-kopieren-auf-die-sd-karte-geht = Kopieren auf die SD-Karte geht nur über Ethernet (eine eigene USB-Übertragung ist zurückgestellt). Unter Einstellungen → Hardware die Verbindung „Kopieren“ auf Ethernet stellen.

# Kontext: mega65net.rs, Err
mega65net-p1-datei-nicht-gefunden-erst-bauen = { $p1 }: Datei nicht gefunden (erst bauen)

# Kontext: mega65net.rs, Err
mega65net-sd-ordner-dir-enthaelt-leerzeichen-bitte = SD-Ordner „{ $dir }“ enthält Leerzeichen – bitte ändern

# Kontext: mega65net.rs, Text
mega65net-p1-sd-image-eingebunden-unchecked = { $p1 }: { $sd_image } eingebunden ({ $UNCHECKED })

# Kontext: mega65net.rs, Text
mega65net-p1-p2-kopiert-nach-sd-r = { $p1 }: { $p2 } kopiert nach { $sd }; { $r }

# Kontext: mega65net.rs, Text
mega65net-p1-diskette-nach-sd-kopiert-eingebunden = { $p1 }: Diskette nach { $sd } kopiert, eingebunden und Programm gestartet ({ $UNCHECKED })

# Kontext: mega65net.rs, Text
mega65net-ethernet-werkzeuge-gefunden-el-ftp-mega65 = Ethernet: Werkzeuge gefunden ({ $el }, { $ftp }), MEGA65 antwortet ({ $UNCHECKED })

# Kontext: mega65net.rs, Err
mega65net-signal = Signal

# Kontext: mega65net.rs, Err
mega65net-code-c = Code { $c }

# Kontext: manuell
mega65net-ungeprueft-noch-nicht-an-einem-mega65-getestet = ungeprüft: noch nicht an einem MEGA65 getestet

# Kontext: manuell
mega65net-gestartet = gestartet

# Kontext: manuell
mega65net-geladen = geladen
