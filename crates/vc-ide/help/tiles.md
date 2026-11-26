# Kachel-Editor

Ein **Kachelsatz** (`.vctiles`, Art `tileset`) besteht aus Kacheln gleicher Größe **B×H** (1 bis 16 Zeichen je Seite). Jede Zelle einer Kachel nennt ein
Zeichen und den Zeichensatz, aus dem es stammt; ein Satz darf mehrere Zeichensätze benutzen. Die Karte (welche Kachel wo steht) liegt nicht im Satz,
sondern in einer Screen-Bank (`.vcscreen`), die den Satz in `tilesets` nennt.

## Aufbau
Der Editor hat drei Bereiche nebeneinander (in schmalen Fenstern rutscht der rechte nach unten).

![Der Kachel-Editor im Bereich Grafik mit der Datei level_tiles.vctiles: links der Kachelsatz mit den Kacheln leer, stein, gras und kiste, in der Mitte die große 2 mal 2 Kachel und der Zeichenwähler mit einem Raster, rechts die Karten Kachel, Farbe, Eigenschaften, Größe der Kacheln, Zielzeichensatz und Kachel.](bild:kachel-editor "Der Kachel-Editor mit Kachelsatz, Kachelfläche und Zeichenwähler.")

- **Links** der **Kachelsatz**: Miniatur, Name und wie oft die Kachel in den Screen-Bänken des Projekts benutzt wird („0×“ heißt unbenutzt); ein **=** zeigt eine Kachel an, die gleich einer früheren ist. **Neue Kachel** (Plus) legt eine an. Bei mehreren Kachelsätzen im Projekt stehen darunter die **Kachelsätze**.
- **In der Mitte** die Werkzeugleiste (Rückgängig, Wiederholen, Spiegeln waagerecht und senkrecht, **Kachel drehen** 90° nur bei quadratischen Kacheln, Zellen rollen in vier Richtungen, **Leeren**, **Kopieren**, **Einfügen**, **Exportieren …**, **Speichern**) und die große Kachel. Linke Taste (auch ziehen) setzt das gewählte Zeichen in die Zelle, rechte Taste oder Alt+Klick nimmt das Zeichen der Zelle als Pinsel auf, Pfeiltasten wandern, **Enter** setzt das Zeichen, **I** nimmt es auf. Daneben die **Zeichentafel** (bei mehreren Zeichensätzen mit Reitern).
- **Rechts** der Abschnitt **Kachel**: Name, **Farbe**, **Hintergrundfarbe**, **Eigenschaften**, **Größe der Kacheln** (Breite und Höhe, 1 bis 16, dann **Übernehmen**; beim Verkleinern geht Belegtes verloren, der Editor fragt vorher), **Zielzeichensatz** und die Kachel-Aktionen **Duplizieren**, **Zusammenführen** (bei einer Kachel, die gleich einer früheren ist: die Kachel entfällt, die Bänke zeigen danach auf die frühere) und **Löschen**. Gibt es Screen-Bänke, die den Satz benutzen, fragt der Editor vor dem Löschen: Verweise auf die gelöschte Kachel werden zu Kachel 0, höhere Nummern rücken nach, der Satz wird gespeichert.

Einen neuen Kachelsatz legt der Dialog **Neue Datei** in dieser Version noch nicht an.

## Farbe
Die Farbart gilt für den ganzen Satz (Abschnitt **Kachel**, **Farbe gilt**): *Je Kachel* (ein Farbbyte je Kachel, am kompaktesten), *Je Zelle* (jede Zelle hat eine eigene Farbe; über der Kachel wählst du, ob ein Klick Zeichen und Farbe, nur das Zeichen oder nur die Farbe setzt) oder *Ohne*.
Beim Atari setzt der Schalter „Zeichenbit 7 setzen (Farbwahl des Zeichens)“ das Zeichenbit 7 (zweite Farbgruppe, Zielzeichensatz höchstens 128 Zeichen).

## Zeichensätze und Füllstand
Die Reiter über der Zeichentafel zeigen die Zeichensätze des Satzes. Unter **Zielzeichensatz** fügt **Hinzufügen** einen aus dem Projekt hinzu (nur mit gleichem Profil wie der erste), **<Name> entfernen** nimmt einen unbenutzten wieder heraus. Beim Export entsteht aus allen benutzten Glyphen **ein** Zielzeichensatz; gleiche Bilder zählen einmal. Der Balken zeigt, wie voll er wird (z. B. „187 von 256 Zeichen belegt“) und meldet einen Überlauf.

## Eigenschaften
Frei benannte Merkmale wie „solide“ oder „Leiter“ (höchstens acht je Satz): Name bei **Neue Eigenschaft** eintippen, **Hinzufügen**, dann je Kachel den Schalter setzen. Der Export schreibt sie als Byte-Tabelle `<name>_props` und die Konstanten `<name>_p_<eigenschaft>`.

## Export
**Exportieren …** in der Werkzeugleiste öffnet den [Export-Dialog](help:export) wie in den anderen Editoren. Ergebnis: je Zelle eine Tabelle `<name>_c0 …` (Index = Kachelnummer), `<name>_col` (je Kachel) oder `<name>_col0 …` (je Zelle),
`<name>_props`, der zusammengeführte Zeichensatz `<name>_chars` und die Konstanten `_tw`, `_th`, `_count`, `_glyphs`. Der Satz braucht mindestens einen Zeichensatz.

## Beispiele
- `examples/c64/jumprun`: ein Kachelsatz `hg` (21 Kacheln zu 2×2, Eigenschaften „solid“ und „ladder“) und eine Screen-Bank `levels` mit den vier Bildern; Zeichnen, Kartenkopie ins RAM und Eigenschaften kommen aus der Bibliothek (`#include <c64/tiles.asm>`). Alle 256 Zeichen des Zeichensatzes sind angepinnt, der Zielzeichensatz `hg_chars` ist deshalb der Spielzeichensatz. Prüfung der Bilder im Emulator: `python3 tools/check_jumprun_c64.py`.
- `examples/c64/gfx_demo`: Kachelsatz `level_tiles` und Bank `level`; `gfx_draw_map` in `lib/gfx.inc` ist nur noch ein Mantel um `screen_draw`.
- `examples/c64/tiles_demo`: alle Größen und Varianten der Bibliothek mit Tests.

## Alte Kachelkarten
Dateien der früheren Art `tilemap` (Kacheln und Karte in einer Datei) wandelt `vcc` auf der Kommandozeile um: `vcc media migrate-tiles` macht aus allen alten Karten des Projekts je Ordner einen Kachelsatz und eine Screen-Bank (ein Screen je Karte); mit `--keep` bleiben die alten Dateien.
