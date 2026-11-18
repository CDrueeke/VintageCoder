# Sprites, Zeichensätze, Bildschirme und Songs

Sprites, Zeichensätze, Bildschirme, Kacheln, Paletten und Songs liegen als Textdateien `<name>.vc<art>` (z. B. `font.vcchar`) im Projekt (Pixel als Hex-Zeilen, gut diff-bar). Ein Doppelklick in der linken Spalte öffnet den passenden Editor. **Datei ▸ Neue Datei …** (oder der Knopf **＋** in der linken Spalte) öffnet den Dialog „Neue Datei“: Dort wählst du die **Art** und bei Grafik und Klang das **Profil** (z. B. `c64.sprite.multicolor`).

| Editor | Inhalt |
|---|---|
| [Sprite/PMG](help:sprite) | C64-Sprites, Atari Player/Missiles |
| [Zeichensatz](help:charset) | 256 bzw. 128 Zeichen |
| [Bildschirm](help:screen) | Textbildschirme 40×25/24 und Screen-Banken aus Kacheln |
| [Kacheln](help:tiles) | 2×2-Kacheln und Karte |
| [Tracker](help:tracker) | Songs (SID, POKEY) |
| [PLAY-Tracker](help:play) | Songs für den PLAY-Befehl des MEGA65 |

## Aufbau der Studios
Sprites, Zeichensätze, Bildschirme und Kacheln haben denselben Aufbau. Jede Art öffnet als Reiter im Editorbereich (Doppelklick auf die Datei in der linken Spalte, *Datei ▸ Neue Datei …* legt eine an).

**Zeichenfläche und Platz:** Sprite-, Zeichensatz-, Bildschirm-, Kachel- und Screen-Studio passen den Zoom ihrer Zeichenfläche an den freien Platz im Reiter an (ganze Pixelvielfache, mit Mindest- und Höchstgröße). **−** und **+** in der Werkzeugleiste, die Tasten **+** und **−** oder **Strg/Cmd + Mausrad** legen den Zoom fest; **Einpassen** (Taste **0**) kehrt zur automatischen Anpassung zurück. Schrift und Knöpfe bleiben dabei unverändert. Wird der Reiter schmal (etwa im halbierten Editorbereich), steht die Liste als Karte in der Seitenspalte, und die Werkzeugleiste zeigt weniger Knöpfe; der Rest steckt im Menü **⋯**.

- **Kopfzeile:** ein Kürzel für die Art (z. B. SPR, CHR, SCR, PAL), Dateiname (ein Punkt zeigt ungespeicherte Änderungen), Maße (bei Sprites auch der Modus des Frames) und die Knöpfe **Exportieren …** und **Speichern** (im Sprite- und Zeichensatz-Editor auch **PNG laden …**). **Exportieren …** öffnet den [Export-Dialog](help:export).
- **Werkzeugleiste:** eine Reihe mit den Zeichenwerkzeugen, Rückgängig, Wiederholen, **Transformieren**, Zoom, Raster und Onion Skin.
- **Links** die Liste oder Matrix (Frames, Zeichen, Kacheln), **in der Mitte** die Zeichenfläche, **rechts** die Seitenspalte aus Karten (Pixelwerte, Farben, Eigenschaften).
- **Fußzeile:** Zelle unter der Maus, Zoom und Meldungen; rechts **? Tasten** mit den Einzeltasten.

## Gemeinsame Bedienung
- Strg/Cmd+S speichert, Strg/Cmd+Z und Umschalt+Z machen rückgängig und wiederholen. **Exportieren …** (Kopfzeile, Strg/Cmd+Umschalt+E) öffnet den Export-Dialog; er schreibt nach `build/export/` (Assembler, Konstanten, Binär, PRG mit Adresse, BASIC DATA), siehe [Export](help:export).
- Beim Bauen werden veraltete Sprites, Zeichensätze, Bildschirme und Songs automatisch exportiert.
- Die Farben kommen je Zielsystem aus `data/palettes`. Beim Atari wählt **Videonorm** in der rechten Spalte PAL, NTSC oder die Norm des Projekts; das ändert nur die Ansicht der Farben, nicht die Datei.
- **PNG laden …** importiert ein Bild: Skalieren, Quantisieren auf die Zielpalette, Meldung bei Farbkonflikten.

## Laden im Programm
Bibliotheken zum Laden von Zeichensatz, Bildschirm und Kacheln: `examples/c64/gfx_demo/lib/gfx.inc`, `examples/atari/gfx_demo/lib/gfx.inc` ; für BASIC bindest du die exportierten `DATA`-Zeilen mit `#include` ein (siehe [BASIC](help:basic-directives)).

## Zeichensätze aus dem ROM übernehmen
„Datei ▸ Zeichensatz aus ROM importieren …“ (Dialog „Zeichensatz hinzufügen“, Reiter *Aus ROM*; oder `vcc media import-rom <datei> --profile ID --offset N --name NAME`) liest den eingebauten Zeichensatz eines Systems aus einem ROM-Abbild und legt ihn als Zeichensatz unter `graphics/` an (diese Kopie enthält die Original-Zeichen und ist nicht zum Einchecken gedacht). Die App **erkennt das Abbild an Größe und Prüfsumme** und bietet dessen Zeichensätze an; nur unbekannte Dateien werden nach der Dateigröße geraten, so:

| Datei | Größe | Zeichensatz |
|---|---|---|
| C64-Zeichensatz-Abbild (z. B. `chargen.bin`) | 4096 | Satz 1 ab `$0` (Groß/Grafik), Satz 2 ab `$800` (Groß/Klein) |
| Atari XL/XE-Betriebssystem (`ATARIXL.ROM`) | 16384 | ab `$2000` (= `$E000` im Speicher) |
| Atari 400/800 (`ATARIOSB.ROM`) | 10240 | ab `$800` |
| Open-ROMs-Zeichensatz (`chargen_openroms.rom`) | 4096 | C64- und MEGA65-Profil (Vollfarbe, Vordergrund = Farbe 1) |

Hinweis: ROM-Abbilder der Originalgeräte sind urheberrechtlich geschützt. Die importierten Zeichensätze bleiben in deinem Projekt; vor einer Veröffentlichung prüfen. Frei verwendbar sind die Open-ROMs-Zeichensätze.

## Zeichensätze als PNG-Raster
Ein Zeichensatz lässt sich auch als Bild austauschen: **16 Zeichen je Zeile in Leserichtung**, gesetzte Pixel hell auf dunklem Grund (128×128 Pixel für 256 Zeichen zu 8×8; ganzzahlig vergrößerte Bilder gehen auch).
- Import: „Datei ▸ Zeichensatz aus PNG-Raster importieren …“ (Dialog „Zeichensatz hinzufügen“, Reiter *Aus PNG-Raster*) oder `vcc media import-sheet raster.png --profile c64.char.hires --name font [--invert]`.
- Export: noch nicht in der Oberfläche; auf der Kommandozeile `vcc media export-sheet graphics/font.vcchar font.png`.
Das Bild wird beim Import in einen Zeichensatz umgewandelt; im Programm landen wie immer die Bytes aus dem Export (`build/export/…`). Die Dateiformate von VintageCoder (`.vcsprite`, `.vcchar`, `.vcscreen`, `.vctiles`, `.vcpal`, `.vcsong` und weitere) sind frei nutzbar; jeder darf sie lesen und schreiben. Hinweis zum Urheberrecht: Das Dateiformat ändert nichts daran, ob Zeichen eines Originalgeräts geschützt sind.
