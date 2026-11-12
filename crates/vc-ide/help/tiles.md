# Kachel-Editor

Ein **Kachelsatz** (`.vctiles`, Art `tileset`) besteht aus Kacheln gleicher Größe **B×H** (1 bis 16 Zeichen je Seite). Jede Zelle einer Kachel nennt ein
Zeichen und den Zeichensatz, aus dem es stammt; ein Satz darf mehrere Zeichensätze benutzen. Die Karte (welche Kachel wo steht) liegt nicht im Satz,
sondern in einer Screen-Bank (`.vcscreen`), die den Satz in `tilesets` nennt.

## Aufbau
Das Studio folgt dem [gemeinsamen Aufbau](help:graphics).

- **Links** die Kachelliste: Miniatur, Nummer, Name und wie oft die Kachel in den Screen-Bänken des Projekts benutzt wird („unbenutzt“, „gleich wie 3“ für Doppelte).
  *Neu*, *Dupl.* und *Löschen*; gibt es Screen-Bänke, die den Satz benutzen, fragt der Editor vor dem Löschen. Dann werden die Bänke angepasst (Verweise auf die gelöschte Kachel werden zu Kachel 0, höhere Nummern rücken nach), der Satz wird gespeichert und Rückgängig geleert.
Über der Liste filtert ein Suchfeld nach Name oder Nummer. Ist eine Kachel gleich einer früheren, bietet die rechte Spalte „Zusammenführen“ an: die Kachel entfällt, die Bänke zeigen danach auf die gleiche frühere Kachel.
- **Mitte** die große Kachel. Linke Taste (auch ziehen) setzt das gewählte Zeichen in die Zelle, rechte Taste nimmt das Zeichen der Zelle als Pinsel auf, Pfeiltasten wandern.
- **Werkzeugleiste** oben: Größe des ganzen Satzes (Schnellwahl 2×2, 3×3, 4×4, 3×2 oder frei; beim Verkleinern wird beschnitten, beim Vergrößern mit Zeichen 0 aufgefüllt),
  Spiegeln (die Zellreihenfolge, nicht die Zeichenbilder), Drehen (nur quadratisch), Rollen, Leeren, Kopieren und Einfügen von Zellen.
- **Rechts** Name, Farbe, Eigenschaften, Zeichensatz-Reiter mit Zeichenwähler, Füllstand und Hintergrundfarbe.

## Farbe
Die Farbart gilt für den ganzen Satz: *je Kachel* (ein Farbbyte je Kachel, am kompaktesten), *je Zelle* (jede Zelle hat eine eigene Farbe, sonst gilt die Kachelfarbe) oder *keine*.
Beim Atari setzt „Farbwahl“ das Zeichenbit 7 (zweite Farbgruppe, Zielzeichensatz höchstens 128 Zeichen).

## Zeichensätze und Füllstand
Die Reiter zeigen die Zeichensätze des Satzes (Kürzel A, B, …); „+ Zeichensatz“ fügt einen aus dem Projekt hinzu (nur mit gleichem Profil wie der erste), „Entfernen“
nimmt einen unbenutzten wieder heraus. Beim Export entsteht aus allen benutzten Glyphen **ein** Zielzeichensatz; gleiche Bilder zählen einmal. Der Balken „Zielzeichensatz“
zeigt, wie voll er wird (z. B. 187 von 256) und meldet einen Überlauf.

## Eigenschaften
Frei benannte Merkmale wie „solide“ oder „Leiter“ (höchstens acht je Satz) mit einem Häkchen je Kachel. Der Export schreibt sie als Byte-Tabelle `<name>_props` und die Konstanten `<name>_p_<eigenschaft>`.

## Export
Export-Streifen und [Export-Dialog](help:export) wie in den anderen Studios. Ergebnis: je Zelle eine Tabelle `<name>_c0 …` (Index = Kachelnummer), `<name>_col` (je Kachel) oder `<name>_col0 …` (je Zelle),
`<name>_props`, der zusammengeführte Zeichensatz `<name>_chars` und die Konstanten `_tw`, `_th`, `_count`, `_glyphs`. Der Satz braucht mindestens einen Zeichensatz.

## Beispiele
- `examples/c64/jumprun`: ein Kachelsatz `hg` (21 Kacheln zu 2×2, Eigenschaften „solid“ und „ladder“) und eine Screen-Bank `levels` mit den vier Bildern; Zeichnen, Kartenkopie ins RAM und Eigenschaften kommen aus der Bibliothek (`#include <c64/tiles.asm>`). Alle 256 Zeichen des Zeichensatzes sind angepinnt, der Zielzeichensatz `hg_chars` ist deshalb der Spielzeichensatz. Prüfung der Bilder im Emulator: `python3 tools/check_jumprun_c64.py`.
- `examples/c64/gfx_demo`: Kachelsatz `level_tiles` und Bank `level`; `gfx_draw_map` in `lib/gfx.inc` ist nur noch ein Mantel um `screen_draw`.
- `examples/c64/tiles_demo`: alle Größen und Varianten der Bibliothek mit Tests.

## Alte Kachelkarten
Dateien der früheren Art `tilemap` (Kacheln und Karte in einer Datei) öffnet der Editor mit einem Hinweis. „Umwandeln“ macht aus allen alten Karten des Projekts je Ordner einen Kachelsatz
und eine Screen-Bank (ein Screen je Karte); auf Wunsch werden die alten Dateien entfernt. Dasselbe leistet `vcc` auf der Kommandozeile (`vcc media migrate-tiles`, mit `--keep` bleiben die alten Dateien).
