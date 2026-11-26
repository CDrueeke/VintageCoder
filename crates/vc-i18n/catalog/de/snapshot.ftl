# SPDX-License-Identifier: GPL-3.0-or-later
# Texte der Oberfläche: snapshot (aus crates/vc-ide/src/snapshot.rs extrahiert, Wortlaut unverändert)

# Kontext: snapshot.rs, Err
snapshot-nicht-lesbar = nicht lesbar

# Kontext: snapshot.rs, Err
snapshot-momentaufnahme-system-s-wird-nicht-unterstuetzt = Momentaufnahme: System '{ $s }' wird nicht unterstützt (C64 und Atari 8-Bit)

# Kontext: snapshot.rs, Text
snapshot-p1-p2-vic-bank-bank-p4 = { $p1 }{ $p2 } · VIC-Bank { $bank } (${ $p4 }–${ $p5 })

# Kontext: snapshot.rs, Text
snapshot-bitmap-modus-kein-textbildschirm-screen-und = Bitmap-Modus: kein Textbildschirm – Screen und Zeichensatz entfallen (Sprites sind trotzdem möglich).

# Kontext: snapshot.rs, Text
snapshot-ecm-modus-es-werden-nur-64 = ECM-Modus: es werden nur 64 Zeichen dargestellt; der Bildschirm wird unverändert gelesen.

# Kontext: snapshot.rs, Text
snapshot-der-zeichensatz-liegt-im-zeichen-rom = Der Zeichensatz liegt im Zeichen-ROM (Standardschrift) und wird von dort gelesen.

# Kontext: snapshot.rs, Text
snapshot-atari-die-farben-und-zeiger-stammen = Atari: Die Farben und Zeiger stammen aus den Schattenregistern des Betriebssystems. Programme mit eigener Display-Liste oder eigenem Interrupt müssen Adressen und Farben ggf. von Hand anpassen.

# Kontext: snapshot.rs, Text
snapshot-keine-display-liste-mit-lms-gefunden = keine Display-Liste mit LMS gefunden

# Kontext: snapshot.rs, Text
snapshot-display-liste-bei-dl-enthaelt-keinen = Display-Liste bei ${ $dl } enthält keinen LMS-Befehl.

# Kontext: snapshot.rs, Text
snapshot-antic-modus-p1-display-liste-dl = ANTIC-Modus { $p1 }, Display-Liste ${ $dl }

# Kontext: snapshot.rs, Text
snapshot-antic-modus-p1-wird-nicht-unterstuetzt = ANTIC-Modus { $p1 } wird nicht unterstützt (nur Modus 2, 40×24 Textzeichen).

# Kontext: snapshot.rs, Text
snapshot-der-zeichensatz-liegt-im-betriebssystem-rom = Der Zeichensatz liegt im Betriebssystem-ROM ($E000) und wird von dort gelesen.

# Kontext: snapshot.rs, Err
snapshot-zu-viele-verschiedene-kacheln-p1-der = Zu viele verschiedene Kacheln ({ $p1 }); der Bildschirm lässt sich nicht als Kachelkarte darstellen

# Kontext: snapshot.rs, Text
snapshot-p1-kacheln-raster-ab-spalte-ox = { $p1 } Kacheln, Raster ab Spalte { $ox }, Zeile { $oy }.

# Kontext: snapshot.rs, Text
snapshot-mixed-kachelstellen-hatten-gemischte-farben-es = { $mixed } Kachelstellen hatten gemischte Farben (es gilt die Farbe der ersten sichtbaren Zelle).

# Kontext: snapshot.rs, ok_or_else
snapshot-profil-id-fehlt = Profil '{ $id }' fehlt

# Kontext: snapshot.rs, Err
snapshot-der-aktuelle-modus-ist-kein-textbildschirm = Der aktuelle Modus ist kein Textbildschirm.

# Kontext: snapshot.rs, Err
snapshot-der-aktuelle-antic-modus-wird-nicht = Der aktuelle ANTIC-Modus wird nicht unterstützt.

# Kontext: snapshot.rs, Err
snapshot-system-s-wird-nicht-unterstuetzt = System '{ $s }' wird nicht unterstützt

# Kontext: snapshot.rs, Err
snapshot-nichts-zu-speichern-adresse-fuer-screen = Nichts zu speichern: Adresse für Screen oder Zeichensatz fehlt (Zeichensatz im ROM?).

# Kontext: snapshot.rs, Text
snapshot-bitmap = Bitmap

# Kontext: snapshot.rs, Text
snapshot-text = Text

# Kontext: snapshot.rs, Text
snapshot-ecm = (ECM)

# Kontext: snapshot.rs, Text
snapshot-multicolor = (Multicolor)
