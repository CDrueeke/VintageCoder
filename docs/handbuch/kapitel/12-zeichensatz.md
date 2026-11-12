---
id: zeichensatz
titel: Zeichensatz-Editor
reihenfolge: 12
stufe: Einsteiger
zeit: 25 Minuten
siehe_auch: [grafik-ueberblick, bildschirm-kacheln, sprites]
referenz: [charset, graphics, roms, export]
---

# Zeichensatz-Editor

**Ziel:** Du malst eigene Buchstaben und Grafikzeichen, importierst und exportierst Zeichensätze und zeigst sie in einem Programm an.

## Das brauchst du

- ein Projekt für C64, Atari, MEGA65 oder ein anderes System mit Textbildschirm,
- am besten die **ROM-Dateien** deines Systems (siehe [Willkommen](kapitel:willkommen)), denn ein neuer Zeichensatz beginnt mit der Schrift deines Rechners. Ohne ROM erscheint eine eingebaute Ersatzschrift.

## Was ist ein Zeichensatz?

Ein Textbildschirm kennt keine Pixel, sondern **Zeichen**: Jede Zelle des Bildschirms enthält eine Zahl (den **Bildschirmcode**), und der Grafikchip schlägt in einer Tabelle nach, wie dieses Zeichen aussieht. Diese Tabelle ist der **Zeichensatz**. Jedes Zeichen besteht aus **8×8 Punkten**, also acht Bytes. Der C64 hat 256 Zeichen (2048 Bytes), der Atari im üblichen Textmodus 128.

Weil Spiele den Zeichensatz gern **umdefinieren**, ist das ein mächtiger Trick: Du malst eigene Kacheln (Mauern, Leitern, Münzen) als „Zeichen“ und baust damit ganze Bilder aus Text.

> **Achtung:** Die Reihenfolge im Zeichensatz ist die der **Bildschirmcodes**, nicht von ASCII oder PETSCII. Beim C64 ist `@` Zeichen 0, `A` Zeichen 1, usw. Beim Atari ist das Leerzeichen 0, die Ziffern beginnen bei 16, die Großbuchstaben bei 33.

Zwei Zeichenarten gibt es:

- **Hires** (hochauflösend): 8×8 Punkte, jeder Punkt ist an oder aus. Die Farbe kommt aus dem Farbspeicher.
- **Multicolor** (mehrfarbig): Die Punkte sind doppelt so breit, ein Zeichen hat also 4×8 Punkte, dafür darf jeder Punkt eine von vier Farben sein. Die Auflösung halbiert sich, dafür gewinnst du Farbe.

Welche Art ein Zeichensatz ist, legt sein **Profil** fest (zum Beispiel `c64.char.hires` oder `c64.char.multicolor`). Beim MEGA65 gibt es zusätzlich den **Vollfarbmodus** (`mega65.char.fcm`): 8×8 Punkte, jeder Punkt eine von 256 Palettenfarben.

## Schritt für Schritt: einen Zeichensatz anlegen und malen

1. Lege eine Datei an: **+** im Projektbaum ▸ **Zeichensatz …**, Namen vergeben, Profil wählen (für den Einstieg `C64 Zeichensatz hires 8×8`), **Anlegen**.
2. Der Editor öffnet. **Links** siehst du alle Zeichen als kleine Kacheln, 16 pro Zeile. Ein Klick wählt ein Zeichen, die Pfeiltasten wandern durch das Raster. Der Name über dem Raster zeigt Nummer und Hexwert, zum Beispiel „Zeichen 1 ($ 01)“.
3. **In der Mitte** ist das gewählte Zeichen stark vergrößert. Mit der **linken Maustaste** setzt du Punkte, mit der **rechten** löschst du. Wähle in der Werkzeugleiste ein Werkzeug: **Stift**, **Radierer**, **Füllen**, **Linie**, **Rahmen**, **Auswahl** oder **Pipette**.
4. **Rechts** stehen die **Pixelwerte** (bei Hires zwei: Hintergrund und Vordergrund), darunter Farbfelder und eine **Vorschau** in Originalgröße. Klick auf einen Wert wählt, was der Stift setzt; die Tasten **1** bis **4** tun dasselbe.
5. Mit den Knöpfen **Spiegeln H**, **Spiegeln V**, **Links**, **Rechts**, **Hoch**, **Runter** und **Leeren** bearbeitest du das ganze Zeichen auf einmal.
6. **Strg/Cmd+S** speichert.

![Zeichensatz-Editor: ① Zeichenmatrix, ② Zeichenfläche, ③ Pixelwerte und Farben, ④ Vorschau, ⑤ Export-Streifen](bild:zeichensatz-ueberblick "Der Zeichensatz-Editor mit Matrix links, vergrößertem Zeichen in der Mitte und Farben rechts")

Wenn der Mauszeiger über dem Editor steht, gibt es Einzeltasten (sie lassen sich in den Einstellungen unter „Grafik-Editoren ▸ Einzeltasten“ abschalten):

| Taste | Wirkung |
|---|---|
| D, E, F, L, R, S, I | Stift, Radierer, Füllen, Linie, Rahmen, Auswahl, Pipette |
| X, Y | Spiegeln waagerecht, senkrecht |
| 1 bis 4 | Pixelwert wählen |
| Pfeil links/rechts | voriges/nächstes Zeichen |

### Die Vorlage

Ein neuer Zeichensatz beginnt **nicht leer**: Er enthält einen Verweis auf die Schrift deines Systems, wie sie nach dem Einschalten erscheint (beim C64 „Großbuchstaben und Grafik“). In der Datei stehen nur **deine Änderungen**. Die Original-Zeichen sind urheberrechtlich geschützt, deshalb liegen sie nicht in deinem Projekt, sondern bleiben im ROM.

Über das Auswahlfeld **Vorlage** in der Werkzeugleiste wählst du eine andere Grundschrift (beim C64 etwa die mit Klein- und Großbuchstaben). Das ersetzt alle Zeichen; **Strg/Cmd+Z** macht es rückgängig. Soll dein Zeichensatz bei Null beginnen, lege ihn auf der Kommandozeile mit `vcc media new … --blank` an.

### Multicolor malen

Bei einem Multicolor-Profil bekommt jeder Punkt einen von vier **Pixelwerten**. Rechts zeigt die Tabelle zu jedem Wert die Farbe und das **Register**, das sie bestimmt (beim C64 Hintergrund `$D021`, dazu `$D022` und `$D023`). Den Wert wählst du per Klick oder Taste 1 bis 4. Drei der Farben (Hintergrund und zwei weitere) gelten für den ganzen Bildschirm, die vierte kommt aus dem Farbspeicher der jeweiligen Bildschirmzelle.

![Multicolor-Zeichen mit vier Pixelwerten und ihren Registern](bild:zeichensatz-multicolor "Im Multicolor-Modus sind die Punkte doppelt so breit; rechts steht, welches Register welche Farbe liefert")

### MEGA65: Vollfarbzeichen

Bei `mega65.char.fcm` zeigt die rechte Spalte statt einiger Farbfelder ein Raster mit allen **256 Palettenfarben**. Du wählst eine Farbe und malst damit. Welche Farben das sind, bestimmt die Palette, siehe [Paletten](kapitel:paletten).

![Vollfarbzeichen des MEGA65 mit der 256-Farben-Auswahl](bild:zeichensatz-mega65-farbe "Beim MEGA65 darf jeder Punkt eine von 256 Farben sein")

## Standardzeichensätze hinzufügen

Brauchst du die System-Schrift als Ausgangspunkt für mehrere Systeme, nimmst du **Datei ▸ Standardzeichensatz hinzufügen …**. Der Dialog legt für die Zielsysteme deines Projekts Zeichensätze in `graphics/` an, die auf dein ROM verweisen (C64, C128, Plus/4, VC-20, Atari, MEGA65). Du kannst sie danach wie jeden anderen Zeichensatz bearbeiten. Die Option „Zeichen stattdessen einbetten“ kopiert die Glyphen in `graphics/standard/` (nicht einchecken, die Rechte liegen bei den Herstellern).

Das Beispielprojekt `examples/c64/standard_charsets` enthält die **freien** Open-ROMs-Zeichensätze (`openroms_c64_gross.vcchar`, `openroms_c64_klein.vcchar`). Die darfst du weitergeben.

![Dialog „Standardzeichensatz hinzufügen“ mit der Liste der Zeichensatz-Quellen](bild:zeichensatz-standard "Standardzeichensätze verweisen auf deine ROMs; fehlt ein ROM, ist der Eintrag grau")

> **Info:** VintageCoder liefert keine Original-ROMs mit und lädt auch keine aus dem Netz. Wo du deine eigenen Abbilder ablegst, steht im Kapitel [Disk-Studio, Segmente, ROMs](kapitel:disk-studio).

## Zeichensätze als PNG austauschen

Du kannst einen Zeichensatz auch in einem Bildprogramm malen. Das Bild ist ein Raster mit **16 Zeichen je Zeile** in Leserichtung, gesetzte Punkte hell auf dunklem Grund. 128×128 Pixel ergeben 256 Zeichen zu 8×8; ganzzahlig vergrößerte Bilder gehen auch.

- **Import:** **Datei ▸ Zeichensatz aus PNG-Raster importieren …** wählt die Bilddatei, das Profil und den Namen und das Häkchen **Dunkle Pixel sind gesetzt (statt heller)**, falls dein Bild dunkel auf hell gemalt ist.
- **Export:** In der Werkzeugleiste des Zeichensatz-Editors speichert **Als PNG-Raster …** alle Zeichen als Bild.
- **Aus einem ROM-Abbild:** **Datei ▸ Zeichensatz aus ROM importieren …** liest den eingebauten Zeichensatz aus einer ROM-Datei und legt ihn unter `graphics/standard/` ab.

![Importdialog für ein PNG-Raster mit Profilwahl](bild:zeichensatz-png-import "Ein Zeichensatz als PNG-Raster: 16 Zeichen je Zeile")

Daneben gibt es in der Werkzeugleiste **PNG laden …**; es skaliert ein Bild auf die Größe des aktuellen Zeichens und wählt die nächstliegenden Farben der Palette.

## Den Zeichensatz im Programm anzeigen

Das Beispiel `examples/c64/gfx_demo` zeigt den ganzen Weg. In diesem Projekt liegen ein Zeichensatz `graphics/font.vcchar` (alle 256 Zeichen selbst gemalt) und ein Bildschirm `graphics/title.vcscreen`, der diese Zeichen benutzt. Das folgende Programm lädt beides:

```asm
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

.include "lib/gfx.inc"

start:  sei
        gfx_load_charset font, $2800        ; unseren Zeichensatz nach $2800 kopieren
        gfx_charset_at $2800                ; der VIC liest die Zeichen ab jetzt von dort
        gfx_screen_at gfx_screen
        gfx_load_screen title_chars, title_attrs
        gfx_colors title_border, title_bg
done:   jmp done

gfx_runtime
.include "build/export/font.asm"
.include "build/export/title.asm"
```

Was passiert hier?

- `.include "lib/gfx.inc"` bringt kleine **Makros** (wiederverwendbare Befehlsfolgen) mit: `gfx_load_charset` kopiert die 2048 Bytes des Zeichensatzes an eine Adresse, die ein Vielfaches von 2 KB sein muss (`$2800` passt). `gfx_charset_at` sagt dem Grafikchip VIC-II, wo er die Zeichenformen findet.
- `gfx_load_screen` schreibt die 1000 Zeichencodes in den Bildschirmspeicher und die Farben in das Farb-RAM. `gfx_colors` setzt Rahmen- und Hintergrundfarbe aus dem Export.
- `gfx_runtime` legt die Kopierroutine ab. Es steht **hinter** dem Hauptprogramm, damit es nicht versehentlich ausgeführt wird.
- Die beiden `.include`-Zeilen holen die Export-Dateien, die beim Bauen (F7) automatisch entstehen.

1. Öffne `examples/c64/gfx_demo`, ersetze den Inhalt von `gfx_demo.asm` durch das Programm oben (oder lege eine Kopie an).
2. Bauen mit **F7**, starten mit **F5**.
3. Öffne `graphics/font.vcchar`, ändere ein Zeichen (zum Beispiel das A), bauen und starten. Das neue Zeichen ist sofort auf dem Titelbild zu sehen.

Für Atari und BASIC gibt es Gegenstücke: `examples/atari/gfx_demo/lib/gfx.inc`; in BASIC bindest du die `DATA`-Zeilen des Exports (Format **BASIC DATA**) mit `#include` ein, siehe [BASIC in der IDE](kapitel:basic).

## Häufige Stolpersteine

- **Statt meiner Zeichen erscheint die normale Schrift:** Du hast vergessen, `gfx_charset_at` aufzurufen, oder die Zieladresse liegt nicht im Bereich des VIC (in der ersten 16-KB-Bank sind `$1000`–`$1FFF` das Zeichen-ROM und daher nicht verfügbar).
- **Die Schrift sieht „falsch“ aus:** Vermutlich zeigt der Editor die Ersatzschrift, weil dein ROM fehlt. Der Editor sagt das im Fuß. ROM ablegen und neu prüfen.
- **Buchstaben stimmen nicht:** Die Reihenfolge ist Bildschirmcode, nicht ASCII (siehe Achtung oben).
- **„Als PNG-Raster …“ fehlt:** Der Knopf erscheint nur bei Zeichensätzen mit einer Farbe je Punkt (Hires) oder Vollfarbe.

## Wie geht es weiter?

Als Nächstes zeichnest du bewegliche Figuren: [Sprite- und PMG-Editor](kapitel:sprites). Die Zeichen setzt du später im [Bildschirm-Editor](kapitel:bildschirm-kacheln) zu Bildern zusammen. Nachschlagen: [Zeichensatz-Editor](hilfe:charset), [Standardzeichensätze und ROMs](hilfe:roms).
