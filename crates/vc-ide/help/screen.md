# Bildschirm-Editor

Textbildschirm; die Größe kommt aus dem Profil: 40×25 mit Farb-RAM (C64), 22×23 (VC-20), 40×24 (Plus/4, Atari ANTIC 2).

- Werkzeuge: Zeichen setzen, Text tippen, Füllen, Rahmen, Pipette, Löschen.
- Rechts den **Zeichencode** aus dem Zeichensatz wählen und die **Farbe** (C64: Farbe je Zelle).
- Als Zeichenquelle dient der verknüpfte Zeichensatz; ohne ihn der Systemzeichensatz.
- Export: `<name>_chars` (und `<name>_attrs` mit Farbwerten beim C64) plus Konstanten `_width`, `_height`, `_border`, `_bg`.
- Laden: `gfx_load_screen` (Assembler) / `gosub @gfx_screen` (BASIC).

## Aufbau
Das Studio folgt dem [gemeinsamen Aufbau](help:graphics): oben Kopfzeile und Werkzeuge, rechts die Zeichenauswahl und die Farben (Zeichensatz, Hintergrund, Rahmen beim C64 und VC-20, Videonorm PAL oder NTSC beim Atari), in der Mitte der Bildschirm, **Exportieren …** in der Kopfzeile öffnet den [Export-Dialog](help:export). Die Rastergröße kommt aus dem Profil des Zielsystems.

## Screen-Banken mit Kacheln
Eine `.vcscreen`-Datei mit `screens` ist eine **Screen-Bank**: ein oder mehrere Screens, die aus Kacheln eines [Kachelsatzes](help:tiles) aufgebaut sind. Der Editor dafür öffnet sich automatisch; alte Textbildschirme (`chars`/`attrs`) behalten die Oberfläche oben.

- **Liste links:** ein Eintrag je Screen, die Nummer ist die Auswahlnummer für `screen_select`. *Neu* (Größe und Farben des aktuellen Screens), *Dupl.*, *Löschen*, *Nach vorn* und *Nach hinten* ändern die Reihenfolge, ein Namensfeld benennt um.
- **Kachelmodus:** links die Palette des Kachelsatzes (Klick wählt den Pinsel, **Umschalt-Klick** wählt einen Block, der als *Stempel* dient; darüber die zuletzt benutzten Kacheln). Werkzeuge: Stift, Linie, Rechteck (gefüllt oder Rahmen), Füllen, Pipette (auch rechte Maustaste), Auswahl (mit Kopieren als Stempel und Leeren), Stempel, Löschen (setzt Kachel 0). Die Zellebene erscheint abgeblendet.
- **Zeichenmodus** (Taste Tab wechselt): zeichnet **Zellen** über die Kacheln (Patches, höchstens 255 je Screen). Zeichen und Zeichensatz-Reiter rechts, Werkzeuge Zeichen, Linie, Rechteck, Pipette, Auswahl, Text, Löschen (die Kachel scheint wieder durch).
- **Auswahl → Kachel:** im Zeichenmodus ein Rechteck wählen, *Als Kachel anlegen …*. Die Kachel nimmt die Zellen ab der linken oberen Ecke in Kachelgröße (mit Farbe). Gibt es schon eine gleiche Kachel, wird sie benutzt. Liegen die Zellen auf dem Kachelraster, ersetzt der Editor sie im Screen durch den Kachelverweis. Der Kachelsatz wird beim Speichern mitgeschrieben.
- **Größe und Ursprung:** rechts. Das Kachelraster beginnt am *Ursprung*; was nicht aufgeht (z. B. die 25. Zeile beim C64 mit 2×2-Kacheln), sind **Restzellen**, schraffiert und der Zellebene vorbehalten.
- **Ansicht:** Kachelgrenzen und Zellgitter schaltbar, Zoom mit **−**, **+** oder Strg/Cmd + Rad, **Eigenschaften-Ebene** (rechts) blendet die gewählte Eigenschaft der Kacheln farbig ein, die **Vorschau** zeigt den Screen mit Rahmen, echtem Zeichensatz und Systemfarben.
- **Atari:** im Zeichenmodus der Schalter *Farbwahl (Zeichenbit 7)*. **Plus/4:** Farbe mit Helligkeit als Palettenindex.
- Export: wie jeder Screen über **Exportieren …** in der Kopfzeile; die Bank liefert je Screen `<name>_map<i>`, die Patch-Listen und den gemeinsamen Zielzeichensatz.

### In Kacheln zerlegen
Im Bildschirm-Editor (Textbildschirm) wandelt der Knopf **Zerlegen …** in der Kopfzeile einen Zeichen-Screen um (Dialog „In Kacheln zerlegen“): Kachelgröße wählen (2×2, 3×3 oder 4×4), die Vorschau nennt das Ergebnis. *Zerlegen* legt `<name>_tiles.vctiles` und `<name>_bank.vcscreen` neben den alten Screen; der alte Screen bleibt.

