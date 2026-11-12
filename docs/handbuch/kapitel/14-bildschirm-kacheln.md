---
id: bildschirm-kacheln
titel: Bildschirm- und Kachel-Editor
reihenfolge: 14
stufe: Einsteiger
zeit: 35 Minuten
siehe_auch: [zeichensatz, sprites, paletten]
referenz: [screen, tiles, export]
---

# Bildschirm- und Kachel-Editor

**Ziel:** Du gestaltest einen Textbildschirm (zum Beispiel ein Titelbild), baust daraus mit **Kacheln** ein Spielfeld und zeigst beides in deinem Programm an.

## Das brauchst du

- einen Zeichensatz aus [Kapitel 12](kapitel:zeichensatz) (oder die Schrift deines Systems),
- ein C64-, VC-20-, Plus/4-, Atari- oder MEGA65-Projekt. Die Beispiele laufen auf dem C64.

## Begriffe

- Ein **Textbildschirm** ist ein Raster aus **Zellen**. Beim C64 sind es 40×25, beim VC-20 22×23 und beim Atari (ANTIC 2) 40×24; die Größe steht im Profil des Zielsystems. Jede Zelle enthält ein Zeichen aus dem Zeichensatz. Beim C64 bekommt jede Zelle außerdem eine Farbe aus dem **Farb-RAM**.
- Eine **Kachel** (englisch *Tile*) ist ein Baustein aus mehreren Zellen, zum Beispiel 2×2 Zeichen, die zusammen eine Mauer ergeben. Ein **Kachelsatz** (`.vctiles`) enthält viele gleich große Kacheln.
- Eine **Screen-Bank** (`.vcscreen`) ist eine Datei mit einem oder mehreren **Screens**, die aus Kacheln eines Kachelsatzes aufgebaut sind. Sie ist die **Karte**: Sie sagt, welche Kachel wo steht.

Warum Kacheln? Ein ganzer Bildschirm sind 1000 Bytes. Eine Karte aus 2×2-Kacheln (20×12 = 240 Bytes) ist viel kleiner. Du zeichnest wenige Bausteine einmal und setzt sie beliebig oft ein. Das ist die klassische Art, Spielfelder zu bauen.

## Schritt für Schritt: einen Textbildschirm gestalten

1. **+** im Projektbaum ▸ **Bildschirm …**, Namen `title` eingeben, Profil **C64 Textbildschirm 40×25**, **Anlegen**.
2. Im Editor liegt in der **Mitte** der Bildschirm. **Rechts** wählst du das Zeichen aus dem **Zeichensatz** und die **Farbe**. Außerdem stellst du dort **Hintergrund** und **Rahmen** ein (bei Atari die **Videonorm**).
3. Die Werkzeuge: **Zeichen setzen**, **Text tippen** (du schreibst einfach mit der Tastatur), **Füllen**, **Rahmen**, **Pipette** und **Löschen**.
4. Soll der Bildschirm **deinen** Zeichensatz benutzen, wählst du ihn rechts unter **Zeichensatz**. Ohne Auswahl erscheint der Systemzeichensatz (oder die **Ersatzschrift**).
5. **Strg/Cmd+S** speichert.

![Bildschirm-Editor mit Titelbild: ① Bildschirm, ② Zeichenauswahl, ③ Farben, ④ Hintergrund und Rahmen, ⑤ Export-Streifen](bild:bildschirm-editor "Der Bildschirm-Editor: mittig der 40×25-Bildschirm, rechts Zeichen, Zeichensatz und Farben")

Das Beispielprojekt `examples/c64/gfx_demo` hat so einen Bildschirm (`graphics/title.vcscreen`, mit dem Zeichensatz `graphics/font.vcchar`). Wie du ihn anzeigst, steht schon in [Kapitel 12](kapitel:zeichensatz).

## Schritt für Schritt: Kacheln zeichnen

1. **+** ▸ **Kacheln …**, Namen `level_tiles`, Profil **C64 Kachelsatz**, **Anlegen**.
2. Rechts musst du dem Satz mindestens einen **Zeichensatz** geben (Reiter **Zeichensatz**, unten **+ Zeichensatz**). Ohne Zeichensatz gibt es nichts, womit du Kacheln bauen könntest.
3. Links steht die **Kachelliste**: Miniatur, Nummer, Name und wie oft die Kachel in Karten vorkommt („unbenutzt“, wenn nie). **Neu** legt eine Kachel an, **Dupl.** kopiert, **Löschen** entfernt (gibt es Karten, die den Satz benutzen, fragt der Editor vorher). Ein Suchfeld filtert nach Name oder Nummer.
4. In der **Mitte** ist die große Kachel. **Linke Maustaste** setzt das gewählte Zeichen in die Zelle, **rechte Maustaste** nimmt das Zeichen einer Zelle als Pinsel auf. Das Zeichen wählst du im **Zeichenwähler** rechts.
5. In der Werkzeugleiste stellst du die **Größe** des ganzen Satzes ein (Schnellwahl 2×2, 3×3, 4×4, 3×2 oder frei, 1 bis 16 Zeichen je Seite). Dazu **Spiegeln**, **Drehen** (nur bei quadratischen Kacheln), **Rollen**, **Leeren**, **Kopieren** und **Einfügen** von Zellen.
6. Die **Farbe** stellst du für den ganzen Satz ein: **je Kachel** (ein Farbwert für die ganze Kachel, am kompaktesten), **je Zelle** (jede Zelle eigene Farbe) oder **keine**.

![Kachel-Editor: ① Kachelliste, ② große Kachel, ③ Zeichenwähler, ④ Farbe und Eigenschaften](bild:kachel-editor "Der Kachel-Editor: links die Kachelliste, in der Mitte die Kachel mit ihren vier Zellen")

### Eigenschaften und Füllstand

Rechts unter **Eigenschaften** vergibst du frei benannte Merkmale, zum Beispiel „solide“ oder „Leiter“ (höchstens acht je Satz). Pro Kachel setzt du ein Häkchen. Ein Spiel kann später abfragen, ob man auf einer Kachel stehen darf. Der Export schreibt sie als Tabelle `<name>_props` und als Konstanten `<name>_p_<eigenschaft>`.

Alle benutzten Zeichen aller Kacheln ergeben beim Export **einen** gemeinsamen **Zielzeichensatz**. Gleiche Bilder zählen nur einmal. Der Balken **Zielzeichensatz** zeigt, wie voll er ist (zum Beispiel 187 von 256) und meldet einen Überlauf.

![Rechte Spalte des Kachel-Editors: ① Zeichensatz-Reiter mit „+ Zeichensatz“, ② Füllstand des Zielzeichensatzes](bild:kachel-bank "Der Zielzeichensatz fasst alle benutzten Glyphen zusammen; die Fußzeile nennt den Füllstand, zum Beispiel „Zeichensatz 256 von 256“")

## Die Karte: der Screen-Editor für Kacheln

Öffnest du eine Screen-Bank (zum Beispiel `graphics/level.vcscreen` aus `gfx_demo`), erscheint der Karten-Editor automatisch.

- **Reiter** oben: ein Reiter pro Screen. Die Nummer ist die Auswahlnummer, mit der dein Programm den Screen wählt. **Neu**, **Duplizieren**, **Löschen** und Pfeile zum Umsortieren stehen daneben.
- **Kachelmodus:** Links liegt die **Palette** des Kachelsatzes. Ein Klick wählt den Pinsel; mit **Umschalt-Klick** wählst du einen ganzen Block, der dann als **Stempel** dient. Werkzeuge: Stift, Linie, Rechteck (gefüllt oder als Rahmen), Füllen, Pipette (auch rechte Maustaste), Auswahl, Stempel, Löschen (setzt Kachel 0).
- **Zeichenmodus:** Mit der Taste **Tab** (oder dem Schalter **Kacheln/Zeichen**) wechselst du auf die **Zellebene**: Hier setzt du einzelne Zeichen **über** die Kacheln (höchstens 255 solche Ausnahmen je Screen). Das Werkzeug **Löschen** lässt die Kachel darunter wieder durchscheinen.
- Rechts stellst du **Größe** und **Ursprung** des Kachelrasters ein. Was nicht aufgeht, zum Beispiel die 25. Zeile beim C64 mit 2×2-Kacheln, bleibt **Restzelle** (schraffiert, nur der Zellebene vorbehalten).
- **Ansicht:** Schalter für **Kachelgrenzen** und **Zellgitter**; die **Eigenschaften-Ebene** blendet eine gewählte Eigenschaft farbig ein, damit du siehst, wo „solide“ Kacheln stehen; die **Vorschau** zeigt den Screen mit Rahmen und den echten Systemfarben.
- **Kachelsatz:** Unter **Kachelsatz** wählst du, welcher Satz zur Bank gehört.

![Karten-Editor mit Kachelpalette, Karte und Vorschau](bild:kachel-karte "Die Karte: links die Kachelpalette, in der Mitte das Spielfeld, rechts Ebenen und Vorschau")

### Aus einem Textbildschirm wird eine Karte

Hast du einen Textbildschirm gemalt, der sich wiederholende Blöcke enthält, wandelt **In Kacheln zerlegen …** (Werkzeugleiste des Bildschirm-Editors) ihn um. Du wählst Kachelgröße und Ursprung, die Vorschau nennt Zahl der Kacheln und Speichergröße vorher und nachher. **Umwandeln** legt `<name>_tiles.vctiles` und `<name>_bank.vcscreen` neben den alten Screen; der alte bleibt bestehen.

## Export

Wie überall: Fuß ▸ **Export …** oder automatisch beim Bauen. Ergebnis:

- **Bildschirm:** `<name>_chars` (Zeichencodes) und `<name>_attrs` (Farben, C64) sowie die Konstanten `_width`, `_height`, `_border` und `_bg`.
- **Kachelsatz:** Tabellen `<name>_c0 …` (eine je Zelle der Kachel; Index = Kachelnummer), `<name>_col`, `<name>_props` und den Zielzeichensatz `<name>_chars`; dazu die Konstanten `_tw`, `_th`, `_count`, `_glyphs`.
- **Screen-Bank:** pro Screen eine Karte `<name>_map<i>` und die Zeigertabellen `<name>_screens_lo/hi`.

## Beispiel: Spielfeld aus Kacheln

Im Projekt `examples/c64/gfx_demo` gehören zusammen: `graphics/level_tiles.vctiles` (Kacheln 2×2, darunter leer, Stein, Gras und Kiste), `graphics/level.vcscreen` (die Karte, 20×12 Kacheln) und `graphics/font.vcchar`. Dieses Programm zeichnet die Karte:

```asm
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

.include "lib/gfx.inc"

start:  sei
        gfx_load_charset level_tiles_chars, $2800   ; Zielzeichensatz des Kachelsatzes
        gfx_charset_at $2800
        gfx_screen_at gfx_screen
        tiles_use level_tiles_c0, level_tiles_col, level_tiles_props, level_tiles_tw, level_tiles_th, level_tiles_count, level_screens_lo, level_screens_hi, level_level_tiles_w, level_level_tiles_h
        gfx_clear 32, level_level_bg
        gfx_colors level_level_border, level_level_bg
        gfx_draw_map level_level_id
done:   jmp done

gfx_runtime
tiles_runtime
.include "build/export/level_tiles.asm"
.include "build/export/level.asm"
```

- Zuerst kommt der **Zielzeichensatz** des Kachelsatzes (`level_tiles_chars`) in den Speicher, denn die Kacheln bestehen aus **seinen** Zeichen.
- `tiles_use` verbindet den Kachelsatz mit der Bank. Die langen Namen sind Marken aus den beiden Exportdateien. Es ist die Zeile, die man am häufigsten abschreibt.
- `gfx_draw_map` zeichnet den gewählten Screen (hier `level_level_id`, die Nummer 0) in den Bildschirmspeicher und das Farb-RAM.
- `tiles_runtime` und `gfx_runtime` legen die Unterprogramme ab. Beide stehen **hinter** dem Hauptprogramm.

1. Öffne `examples/c64/gfx_demo`, ersetze `gfx_demo.asm` durch das Programm oben (oder arbeite in einer Kopie).
2. Bauen mit **F7**, starten mit **F5**. Du siehst ein Spielfeld mit Steinboden, einer Grasreihe, einer Steinsäule und einer schwebenden Kiste.
3. Öffne `graphics/level.vcscreen`, setze mit dem Stift eine zweite Kiste daneben, speichere, baue und starte neu.

Das Original `gfx_demo.asm` zeigt per Schalter `-D DEMO_MAP=1` die Karte statt des Titelbildes. Ein größeres Beispiel mit Eigenschaften und vier Karten ist `examples/c64/jumprun`.

## Häufige Stolpersteine

- **Karte zeigt Unsinn:** Der Zielzeichensatz wurde nicht geladen oder liegt nicht auf einer 2-KB-Grenze. Er muss mit `gfx_charset_at` angemeldet sein.
- **Die Karte wird nicht gezeichnet:** `tiles_use` fehlt oder nennt falsche Marken, oder `tiles_runtime` fehlt hinter dem Programm.
- **„Überlauf“ im Zielzeichensatz:** Du benutzt mehr als 256 verschiedene Zeichenbilder. Lösche Kacheln oder gleiche Zeichen zusammen (**Zusammenführen** bei gleichen Kacheln).
- **Der Kachel-Editor sagt „Der Satz hat keinen Zeichensatz“:** Rechts erst einen Zeichensatz hinzufügen.
- **Die letzte Zeile bleibt leer:** Die 25. Zeile ist beim C64 mit 2×2-Kacheln eine Restzelle; du malst sie im Zeichenmodus.

## Wie geht es weiter?

Wenn du eigene Farben bestimmen willst (MEGA65), geht es im nächsten Kapitel um [Paletten](kapitel:paletten). Nachschlagen: [Bildschirm-Editor](hilfe:screen), [Kachel-Editor](hilfe:tiles), [Export](hilfe:export).
