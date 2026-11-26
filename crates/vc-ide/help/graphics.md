# Sprites, Zeichensätze, Bildschirme und Songs

Sprites, Zeichensätze, Bildschirme, Kacheln, Paletten und Songs liegen als Textdateien `<name>.vc<art>` (z. B. `font.vcchar`) im Projekt (Pixel als Hex-Zeilen, gut diff-bar). Du findest die Editoren im Hauptbereich **Grafik** (zweite Reihe: Sprites, Zeichensatz, Kacheln, Bildschirm, Palette); den Tracker für Songs findest du im Hauptbereich **Musik**. Ein Doppelklick auf die Datei im Projektbaum öffnet den passenden Editor. Neue Dateien legst du für Sprites und Zeichensätze im Editor selbst an (**Neue Sprite-Datei …**, **Neuer Zeichensatz …**): Im Dialog wählst du unter **Art und System** das **Profil** (z. B. `c64.sprite.multicolor`), den **Namen** und den **Ordner im Projekt**. Kachelsätze, Bildschirme, Paletten und Songs öffnest du aus dem Projekt; einen eigenen Dialog zum Anlegen gibt es dafür nicht.

| Editor | Inhalt | In der neuen Oberfläche |
|---|---|---|
| [Sprite/PMG](help:sprite) | C64-Sprites, Atari Player/Missiles | gebaut |
| [Zeichensatz](help:charset) | 256 bzw. 128 Zeichen | gebaut |
| Palette | Paletten (MEGA65) | gebaut |
| [Bildschirm](help:screen) | Textbildschirme 40×25/24 und Screen-Banken aus Kacheln | gebaut |
| [Kacheln](help:tiles) | 2×2-Kacheln | gebaut |
| [Karten](help:screen) | Screen-Banken als Karten aus Kacheln (Levels) | gebaut |
| [Tracker](help:tracker) | Songs (SID, POKEY), Hauptbereich **Musik** | gebaut |
| [PLAY-Tracker](help:play) | Songs für den PLAY-Befehl des MEGA65, ebenfalls im Hauptbereich **Musik** | gebaut |

## Aufbau der Editoren
Sprite-, Zeichensatz-, Palette-, Kachel- und Bildschirm-Editor haben einen ähnlichen Aufbau und zeigen immer eine Datei. Ist sie geändert, steht in der Fußzeile „● ungespeichert“.

- **Links** die Liste: im Sprite-Editor die Sprite-Dateien und die Bilder (Frames), im Zeichensatz-Editor die Zeichenübersicht, im Palette-Editor das Farbraster, im Kachel-Editor der Kachelsatz, im Bildschirm-Editor die Zeichentafel mit der Zeichenfarbe.
- **In der Mitte** die **Werkzeugleiste** mit den Zeichenwerkzeugen, Rückgängig, Wiederholen, Spiegeln, **Raster**, Zoom und (nur Sprites) **Onion Skin**, darunter die Zeichenfläche. Rechts in der Leiste stehen die Dateiknöpfe **Exportieren …** und **Speichern** (im Sprite- und Zeichensatz-Editor auch **PNG laden …**). Kachel- und Bildschirm-Editor haben ihre eigenen Werkzeuge, siehe [Kacheln](help:tiles) und [Bildschirm](help:screen).
- **Rechts** die Karten mit Farben, Pixelwerten und Eigenschaften.
- **Fußzeile:** Name, Größe, Zellposition und Meldungen.

![Der Reiter Grafik mit der Seite „Sprites“ und der Datei hero.vcsprite: links die Sprite-Dateien und zwei Bilder, in der Mitte das 12x21 Pixel große Multicolor-Sprite einer Figur in Gelb, Rotbraun und Blau, rechts die Karten „Farben“, „Sprite“ (Mehrfarbig an), „Vorschau im Spiel“ und „Verschieben“. Unter der Zeichenfläche steht der Hinweis „Keine Animation“.](bild:sprite-c64 "Der Sprite-Editor mit einem Multicolor-Sprite.")

**Zeichenfläche und Platz (Sprites, Zeichensätze):** Die Zeichenfläche passt ihren Zoom an den freien Platz an. **−** und **+** in der Werkzeugleiste, die Tasten **+** und **−** oder **Cmd/Strg + Mausrad** legen den Zoom fest; **Einpassen** (Taste **0**) kehrt zur automatischen Anpassung zurück. Wird das Fenster schmal, ordnen die Editoren ihre Karten untereinander an.

## Gemeinsame Bedienung
- Cmd/Strg+S speichert, Cmd/Strg+Z und Cmd/Strg+Umschalt+Z machen rückgängig und wiederholen. **Exportieren …** (Werkzeugleiste, Cmd/Strg+Umschalt+E) öffnet den Export-Dialog, siehe [Export](help:export).
- Mit der rechten Maustaste löschst du auf der Zeichenfläche (Sprite-, Zeichensatz-Editor).
- Die Farben kommen je Zielsystem aus `data/palettes`. Beim Atari wählt die **Palettenansicht** in der Karte **Farben** *Projekt*, *PAL* oder *NTSC*; das ändert nur die Ansicht der Farben, nicht die Datei.
- **PNG laden …** importiert ein Bild in das gewählte Bild oder Zeichen: Skalieren, Quantisieren auf die Zielpalette, Meldung bei Farbkonflikten.
- Die Einzeltasten der Werkzeuge stehen unter [Tastenkürzel](help:shortcuts).

## Laden im Programm
Bibliotheken zum Laden von Zeichensatz, Bildschirm und Kacheln: `examples/c64/gfx_demo/lib/gfx.inc`, `examples/atari/gfx_demo/lib/gfx.inc` ; für BASIC bindest du die exportierten `DATA`-Zeilen mit `#include` ein (siehe [BASIC](help:basic-directives)).

## Zeichensätze aus dem ROM übernehmen
Der Befehl `vcc media import-rom <datei> --profile ID --offset N --name NAME` liest den eingebauten Zeichensatz eines Systems aus einem ROM-Abbild und legt ihn als Zeichensatz unter `graphics/` an (diese Kopie enthält die Original-Zeichen und ist nicht zum Einchecken gedacht). Er **erkennt das Abbild an Größe und Prüfsumme**; nur unbekannte Dateien werden nach der Dateigröße geraten, so:

| Datei | Größe | Zeichensatz |
|---|---|---|
| C64-Zeichensatz-Abbild (z. B. `chargen.bin`) | 4096 | Satz 1 ab `$0` (Groß/Grafik), Satz 2 ab `$800` (Groß/Klein) |
| Atari XL/XE-Betriebssystem (`ATARIXL.ROM`) | 16384 | ab `$2000` (= `$E000` im Speicher) |
| Atari 400/800 (`ATARIOSB.ROM`) | 10240 | ab `$800` |
| Open-ROMs-Zeichensatz (`chargen_openroms.rom`) | 4096 | C64- und MEGA65-Profil (Vollfarbe, Vordergrund = Farbe 1) |

Hinweis: Die importierten Zeichensätze bleiben in deinem Projekt. Frei verwendbar sind die Open-ROMs-Zeichensätze.

## Zeichensätze als PNG-Raster
Ein Zeichensatz lässt sich auch als Bild austauschen: **16 Zeichen je Zeile in Leserichtung**, gesetzte Pixel hell auf dunklem Grund (128×128 Pixel für 256 Zeichen zu 8×8; ganzzahlig vergrößerte Bilder gehen auch).
- Import: `vcc media import-sheet raster.png --profile c64.char.hires --name font [--invert]`.
- Export: `vcc media export-sheet graphics/font.vcchar font.png`.
Das Bild wird beim Import in einen Zeichensatz umgewandelt; im Programm landen wie immer die Bytes der Mediendatei (beim Einbinden umgewandelt). Die Dateiformate von VintageCoder (`.vcsprite`, `.vcchar`, `.vcscreen`, `.vctiles`, `.vcpal`, `.vcsong` und weitere) sind frei nutzbar; jeder darf sie lesen und schreiben. Hinweis zum Urheberrecht: Das Dateiformat ändert nichts daran, ob Zeichen eines Originalgeräts geschützt sind.
