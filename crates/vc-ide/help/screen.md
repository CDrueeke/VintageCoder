# Bildschirm-Editor

Textbildschirm; die Größe kommt aus dem Profil: 40×25 mit Farb-RAM (C64), 22×23 (VC-20), 40×24 (Plus/4, Atari ANTIC 2).

- Werkzeuge: Zeichen setzen, Text tippen, Füllen, Rahmen, Pipette, Löschen.
- Rechts den **Zeichencode** aus dem Zeichensatz wählen und die **Farbe** (C64: Farbe je Zelle).
- Als Zeichenquelle dient der verknüpfte Zeichensatz; ohne ihn der Systemzeichensatz.
- Export: `<name>_chars` (und `<name>_attrs` mit Farbwerten beim C64) plus Konstanten `_width`, `_height`, `_border`, `_bg`.
- Laden: `gfx_load_screen` (Assembler) / `gosub @gfx_screen` (BASIC).

## Aufbau
Der Editor hat drei Bereiche nebeneinander (in schmalen Fenstern rutscht der rechte nach unten).

![Der Bildschirm-Editor im Bereich Grafik mit der Datei title.vcscreen: links die Zeichenauswahl und die Liste der Bildschirm-Dateien, in der Mitte die 40 mal 25 große Textbildschirm-Fläche mit einem türkisen Rahmen aus Zeichen und weißen und gelben Zeichen, rechts die Karten für Größe, Farben des Systems, Zeichensatz und Ausgabe.](bild:bildschirm-editor "Der Bildschirm-Editor mit Zeichenauswahl, Bildschirm-Fläche und Einstellungen.")

- **Links** die **Zeichentafel** mit dem Namen des Zeichensatzes (Pfeiltasten wählen ein Zeichen) und, wo es Farb-RAM gibt (C64), die **Zeichenfarbe**; bei mehreren Bildschirm-Dateien im Projekt darunter die **Bildschirm-Dateien**.
- **In der Mitte** die Werkzeugleiste (Zeichen setzen **D**, Rahmen **R**, Füllen **F**, Text tippen **T**, Pipette **I**, Löschen **E**, Rückgängig, Wiederholen, **In Kacheln zerlegen …**, **Exportieren …**, **Speichern**), eine Hinweiszeile zum gewählten Werkzeug und der Bildschirm. Klick oder Ziehen wendet das Werkzeug an, die rechte Maustaste löscht; Pfeiltasten bewegen den Cursor, **Enter** wendet das Werkzeug an der Cursorzelle an.
- **Rechts** der Abschnitt **Bildschirm**: Größe, **Farben des Systems** (Rahmen, Hintergrund, Zeichen), **Zeichensatz** (Auswahl oder **Ohne Zeichensatz**, dann gilt der Systemzeichensatz), beim Atari die Videonorm (Projekt, PAL, NTSC) und **Ausgabe** mit **Exportieren …**. Das öffnet den [Export-Dialog](help:export).

Die Rastergröße kommt aus dem Profil des Zielsystems. Einen neuen Bildschirm legt der Dialog **Neue Datei** in dieser Version noch nicht an.

## Screen-Banken mit Kacheln
Eine `.vcscreen`-Datei mit `screens` ist eine **Screen-Bank**: ein oder mehrere Screens, die aus Kacheln eines [Kachelsatzes](help:tiles) aufgebaut sind. Sie bearbeitest du im **Karten-Editor** (Grafik ▸ Karten, siehe [Bildschirm- und Kachel-Editor](kapitel:bildschirm-kacheln)). Eine Bank lässt sich wie jeder Bildschirm exportieren; die Bank liefert je Screen `<name>_map<i>`, die Patch-Listen und den gemeinsamen Zielzeichensatz.

### In Kacheln zerlegen
Im Bildschirm-Editor (Textbildschirm) wandelt der Knopf **In Kacheln zerlegen …** in der Werkzeugleiste einen Zeichen-Screen um (Dialog „In Kacheln zerlegen“): **Kachelgröße** wählen (2 × 2, 3 × 3 oder 4 × 4), die Vorschau nennt das Ergebnis. **Zerlegen** legt `<name>_tiles.vctiles` und `<name>_bank.vcscreen` neben den alten Screen und öffnet den neuen Kachelsatz; der alte Screen bleibt.
