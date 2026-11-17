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
- am besten die **ROM-Dateien** deines Systems (siehe [Willkommen](kapitel:willkommen)), denn ein neuer Zeichensatz beginnt mit der Schrift deines Rechners. Ohne ROM nimmt VintageCoder seinen **eigenen Standardsatz** („VC-Standard“, frei und weitergebbar, siehe unten).

## Wofür ist das Studio gedacht?

Im Zeichensatz-Studio malst du die **Buchstaben und Grafikzeichen** deines Programms. Weil ein Textbildschirm nur aus Zeichen besteht, ist ein eigener Zeichensatz der einfachste Weg zu einer eigenen Optik, ohne Sprites oder Bitmap-Grafik.

Typische Einsätze aus echten Retro-Projekten:

- **Eine eigene Schrift** für ein Menü oder Textabenteuer (schmal, kursiv, im Stil des Spiels).
- **Spielgrafik aus Zeichen:** Mauern, Leitern, Münzen und Hintergründe eines Plattformspiels sind oft umdefinierte Zeichen. Das ist schnell und spart Speicher.
- **Ein Titelbild aus Text:** ein großes Logo, aus 2×2 oder 3×3 Zeichen zusammengesetzt.
- **Übersetzte oder erweiterte Zeichen:** Umlaute für einen Rechner, der sie nicht kennt.

Im fertigen Programm landet der Zeichensatz als **2048 Bytes** (C64: 256 Zeichen mal 8 Bytes) an einer Adresse im Speicher; dein Programm sagt dem Grafikchip, wo er die Zeichenformen holen soll.

## Grundbegriffe ohne Vorwissen

- **Zeichen:** ein Bildpunktmuster von 8×8 Punkten, das der Chip für einen Bildschirmplatz zeichnet.
- **Bildschirmcode:** die Nummer des Zeichens (0 bis 255). Sie ist **nicht** der ASCII- oder PETSCII-Code.
- **Zeichensatz:** die Tabelle aller Zeichenformen. Er liegt normalerweise im ROM des Rechners; du kannst ihn durch einen eigenen im RAM ersetzen.
- **ROM:** Festspeicher des Rechners. Die Original-ROM-Dateien sind urheberrechtlich geschützt und gehören deshalb nicht in dein Projekt.
- **Hires / Multicolor / Vollfarbe:** Zeichenarten, siehe unten.
- **Profil:** legt fest, welche Zeichenart und welches System eine Datei hat (zum Beispiel `c64.char.hires`).
- **Vorlage:** der Satz, mit dem ein neuer Zeichensatz beginnt: dein ROM oder der eigene Standardsatz „VC-Standard“.
- **VC-Standard:** die eigenen Zeichensätze von VintageCoder, frei unter CC0 und weitergebbar.

## Was ist ein Zeichensatz?

Ein Textbildschirm kennt keine Pixel, sondern **Zeichen**: Jede Zelle des Bildschirms enthält eine Zahl (den **Bildschirmcode**), und der Grafikchip schlägt in einer Tabelle nach, wie dieses Zeichen aussieht. Diese Tabelle ist der **Zeichensatz**. Jedes Zeichen besteht aus **8×8 Punkten**, also acht Bytes. Der C64 hat 256 Zeichen (2048 Bytes), der Atari im üblichen Textmodus 128.

Weil Spiele den Zeichensatz gern **umdefinieren**, ist das ein mächtiger Trick: Du malst eigene Kacheln (Mauern, Leitern, Münzen) als „Zeichen“ und baust damit ganze Bilder aus Text.

> **Achtung:** Die Reihenfolge im Zeichensatz ist die der **Bildschirmcodes**, nicht von ASCII oder PETSCII. Beim C64 ist `@` Zeichen 0, `A` Zeichen 1, usw. Beim Atari ist das Leerzeichen 0, die Ziffern beginnen bei 16, die Großbuchstaben bei 33.

Zwei Zeichenarten gibt es:

- **Hires** (hochauflösend): 8×8 Punkte, jeder Punkt ist an oder aus. Die Farbe kommt aus dem Farbspeicher.
- **Multicolor** (mehrfarbig): Die Punkte sind doppelt so breit, ein Zeichen hat also 4×8 Punkte, dafür darf jeder Punkt eine von vier Farben sein. Die Auflösung halbiert sich, dafür gewinnst du Farbe.

Welche Art ein Zeichensatz ist, legt sein **Profil** fest (zum Beispiel `c64.char.hires` oder `c64.char.multicolor`). Beim MEGA65 gibt es zusätzlich den **Vollfarbmodus** (`mega65.char.fcm`): 8×8 Punkte, jeder Punkt eine von 256 Palettenfarben.

## Mein erster Zeichensatz (am Beispiel `gfx_demo`)

1. **Beispiel öffnen:** Startseite ▸ Beispiele ▸ `examples/c64/gfx_demo`. *Erwartet:* Im Baum liegt `graphics/font.vcchar`.
2. **Zeichensatz öffnen:** Doppelklick auf `font.vcchar`. *Erwartet:* Das Studio öffnet. Links die Liste aller 256 Zeichen, in der Mitte das gewählte Zeichen stark vergrößert.
3. **Zeichen wählen:** Klick in die Liste oder Pfeiltasten. *Erwartet:* Über der Liste steht Nummer und Hexwert, zum Beispiel „Zeichen 1 ($01)“.
4. **Malen:** Stift (**D**), linke Maustaste setzt Punkte, rechte löscht. *Erwartet:* Die kleine Kachel in der Liste zeigt dein Zeichen sofort mit.
5. **Speichern:** **Cmd/Strg+S**.
6. **Bauen und starten:** **F7**, **F5**. *Erwartet:* Das geänderte Zeichen erscheint im Emulator auf dem Titelbild.

Ein ganz neuer Zeichensatz: **＋** in der linken Spalte ▸ im Dialog **Neue Datei** die Art **Zeichensätze** wählen, Profil wählen (für den Einstieg `C64 Zeichensatz hires 8×8`), Namen eingeben, **OK**.

## Aufbau der Oberfläche

| Zone | Inhalt im Zeichensatz-Studio |
|---|---|
| Kopfzeile | Art **CHR**, Name, **●**, Maße, **PNG laden …**, **Exportieren …**, **Speichern** |
| Werkzeugleiste | Stift, Radierer, Füllen, Linie, Rahmen, Auswahl, Pipette, Rückgängig/Wiederholen, **Transformieren ▾**, Zoom, Raster |
| Liste | alle Zeichen als Kacheln, 16 je Zeile |
| Fläche | das gewählte Zeichen, vergrößert |
| Karten | **Werkzeug** (die Zeichenfarbe, also die Pixelwerte), **Dokument** (Farben und **Vorlage**), **Herkunft** (woher die Zeichen stammen) |
| Fußzeile | Zelle unter der Maus, Zoom, Meldungen, rechts **? Tasten** |

Werkzeuge: **Stift** (D), **Radierer** (E), **Füllen** (F), **Linie** (L), **Rahmen** (R), **Auswahl** (S), **Pipette** (I). Dazu unter **Transformieren ▾** **Spiegeln H/V** (X/Y), **Links/Rechts/Hoch/Runter** und **Leeren**. Pixelwerte wählst du mit **1** bis **4**.

![Zeichensatz-Studio: links das Raster der 256 Zeichen, in der Mitte das vergrößerte Zeichen, rechts die Karten Werkzeug und Dokument mit den Farben](bild:zeichensatz-ueberblick "Das Zeichensatz-Studio: Zeichenraster links, das gewählte Zeichen in der Mitte, Farben rechts")

Wenn der Mauszeiger über dem Editor steht, gibt es Einzeltasten (sie lassen sich unter **Einstellungen ▸ Editor** mit dem Schalter **Einzeltasten der Studios** abschalten):

| Taste | Wirkung |
|---|---|
| D, E, F, L, R, S, I | Stift, Radierer, Füllen, Linie, Rahmen, Auswahl, Pipette |
| X, Y | Spiegeln waagerecht, senkrecht |
| 1 bis 4 | Pixelwert wählen |
| Pfeil links/rechts | voriges/nächstes Zeichen |

### Die Vorlage

Ein neuer Zeichensatz beginnt **nicht leer**: Er enthält einen Verweis auf die Schrift deines Systems, wie sie nach dem Einschalten erscheint (beim C64 „Großbuchstaben und Grafik“). In der Datei stehen nur **deine Änderungen**. Die Original-Zeichen sind urheberrechtlich geschützt, deshalb liegen sie nicht in deinem Projekt, sondern bleiben im ROM. Hast du kein ROM, steht dahinter der eigene Satz von VintageCoder (Etikett **EIGEN**).

In der Karte **Dokument** wählst du unter **Vorlage** per Knopf eine andere Grundschrift (beim C64 etwa die mit Klein- und Großbuchstaben). Neben den Schriften deiner ROMs steht dort immer auch **Standard (eigen)**: der mitgelieferte Satz von VintageCoder, als **Kopie** in deiner Datei. Das ersetzt alle Zeichen; **Strg/Cmd+Z** macht es rückgängig. Soll dein Zeichensatz bei Null beginnen, lege ihn auf der Kommandozeile mit `vcc media new … --blank` an.

### Multicolor malen

Bei einem Multicolor-Profil bekommt jeder Punkt einen von vier **Pixelwerten**. Rechts zeigt die Tabelle zu jedem Wert die Farbe und das **Register**, das sie bestimmt (beim C64 Hintergrund `$D021`, dazu `$D022` und `$D023`). Den Wert wählst du per Klick oder Taste 1 bis 4. Drei der Farben (Hintergrund und zwei weitere) gelten für den ganzen Bildschirm, die vierte kommt aus dem Farbspeicher der jeweiligen Bildschirmzelle.

![Das Zeichensatz-Studio mit einem Multicolor-Zeichen: die Punkte sind doppelt so breit, rechts die Karte „Werkzeug“ mit den vier Pixelwerten und die Karte „Dokument“ mit den Farben.](bild:zeichensatz-multicolor "Im Multicolor-Modus sind die Punkte doppelt so breit; rechts steht, welches Register welche Farbe liefert")

### MEGA65: Vollfarbzeichen

Bei `mega65.char.fcm` zeigt die rechte Spalte statt einiger Farbfelder ein Raster mit allen **256 Palettenfarben**. Du wählst eine Farbe und malst damit. Welche Farben das sind, bestimmt die Palette, siehe [Paletten](kapitel:paletten).


## Eigene Standardzeichensätze (VC-Standard)

VintageCoder bringt für jedes System **eigene Zeichensätze** mit (C64, C128 mit VDC, VC-20, Plus/4, Atari mit Standard und International, MEGA65). Sie sind **kein Nachbau** der Original-ROMs, sondern eigene Entwürfe der Zeichenformen, frei unter der Lizenz CC0 (gemeinfrei). Du darfst sie in eigenen Programmen und Projekten benutzen und weitergeben.

- **Neues Projekt:** Im Abschnitt **Zeichensatz des Projekts** wählst du **Automatisch** (dein ROM als Verweis, sonst der eigene Satz), **Nur Standard (eigen)** (immer der eigene Satz, mit `HERKUNFT.txt` unter `graphics/vc-standard/`: das Projekt lässt sich weitergeben) oder **Keinen**. Auf der Kommandozeile: `vcc new … --charset auto|own|none`.
- **Zeichensatz-Studio:** Die Vorlage **Standard (eigen)** kopiert den Satz in deine Datei. Die Karte **Herkunft** in der Seitenspalte sagt in Worten, woher die Zeichen stammen: **EIGEN**, **EIGEN, GEÄNDERT**, **AUS DEINEM ROM** oder **ROM FEHLT**.
- **Rangfolge:** dein ROM, dann der eigene Satz, dann nichts. Fehlt auf einem anderen Rechner das ROM, auf das ein Verweis zeigt, springt VintageCoder auf den eigenen Satz und sagt es.

> **Hinweis:** Die Freigabe der Zeichenformen steht noch aus; die Dateien tragen deshalb den Vermerk „Freigabe offen“. Die Herkunft jedes Satzes steht in der Datei (`origin`).

## Standardzeichensätze hinzufügen

Brauchst du die System-Schrift als Ausgangspunkt, nimmst du **Datei ▸ Standardzeichensatz hinzufügen …**. Der Dialog **Zeichensatz hinzufügen** hat drei Reiter: **Standardzeichensatz**, **Aus ROM** und **Aus PNG-Raster**. Im Reiter **Standardzeichensatz** wählst du einen Eintrag aus der Liste. Sie zeigt die Sätze für die Zielsysteme deines Projekts (C64, C128, Plus/4, VC-20, Atari, MEGA65). Ein Eintrag mit **(dein ROM)** legt eine Datei in `graphics/` an, die auf dein ROM verweist. Du kannst sie danach wie jeden anderen Zeichensatz bearbeiten. Das Häkchen **Zeichen stattdessen einbetten** kopiert die Glyphen in die Datei (nicht einchecken, die Rechte liegen bei den Herstellern). Die eigenen Sätze von VintageCoder (Einträge **VC-Standard …**) kommen als Kopie nach `graphics/vc-standard/` und dürfen eingecheckt werden.

Das Beispielprojekt `examples/c64/standard_charsets` enthält die **freien** Open-ROMs-Zeichensätze (`openroms_c64_gross.vcchar`, `openroms_c64_klein.vcchar`). Die darfst du weitergeben.

![Der Dialog „Zeichensatz hinzufügen“ im Reiter „Standardzeichensatz“ mit der Liste der Quellen und dem Häkchen „Zeichen stattdessen einbetten“.](bild:zeichensatz-standard "Standardzeichensätze verweisen auf deine ROMs; fehlt ein ROM, ist der Eintrag grau")

> **Info:** VintageCoder liefert keine Original-ROMs mit und lädt auch keine aus dem Netz. Wo du deine eigenen Abbilder ablegst, steht im Kapitel [Disk-Studio, Segmente, ROMs](kapitel:disk-studio).

## Zeichensätze als PNG austauschen

Du kannst einen Zeichensatz auch in einem Bildprogramm malen. Das Bild ist ein Raster mit **16 Zeichen je Zeile** in Leserichtung, gesetzte Punkte hell auf dunklem Grund. 128×128 Pixel ergeben 256 Zeichen zu 8×8; ganzzahlig vergrößerte Bilder gehen auch.

- **Import:** **Datei ▸ Zeichensatz aus PNG-Raster importieren …** öffnet den Dialog im Reiter **Aus PNG-Raster**. Dort wählst du die Bilddatei, das Profil und den Namen und setzt das Häkchen **Dunkle Pixel sind gesetzt (statt heller)**, falls dein Bild dunkel auf hell gemalt ist.
- **Export als PNG-Raster:** noch nicht in der Oberfläche.
- **Aus einem ROM-Abbild:** **Datei ▸ Zeichensatz aus ROM importieren …** öffnet den Dialog im Reiter **Aus ROM**. Du wählst eine ROM-Datei, die du selbst besitzt, und einen erkannten Satz. VintageCoder legt ihn als Zeichensatzdatei in `graphics/` ab.

![Der Dialog „Zeichensatz hinzufügen“ im Reiter „Aus PNG-Raster“ mit Name, Dateiwahl, Profil und dem Häkchen „Dunkle Pixel sind gesetzt“.](bild:zeichensatz-png-import "Ein Zeichensatz als PNG-Raster: 16 Zeichen je Zeile")

Daneben gibt es in der Kopfzeile des Studios **PNG laden …**; es skaliert ein Bild auf die Größe des aktuellen Zeichens und wählt die nächstliegenden Farben der Palette.

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

## Hardware-Besonderheiten und Grenzen

- **C64, C128 (40 Spalten), VC-20, Plus/4:** 256 Zeichen zu 8×8 Punkten (VC-20 je nach Satz); der Satz muss im Speicher auf einer 2-KB-Grenze beginnen und im Sichtbereich des Grafikchips liegen (beim C64 nicht bei `$1000`–`$1FFF` der ersten Bank, dort sieht der VIC das ROM).
- **Atari:** 128 Zeichen (Inversdarstellung macht die Hardware); der Satz beginnt auf einer 1-KB-Grenze. Standard und International sind zwei Sätze.
- **C128 80 Spalten (VDC):** 256 Zeichen zu 8×16 Punkten.
- **MEGA65:** zusätzlich Vollfarbzeichen (256 Palettenfarben je Punkt).
- **Multicolor:** je Zeichen 4×8 breite Punkte mit vier Werten; drei Farben gelten für den ganzen Bildschirm, eine kommt aus dem Farbspeicher der Zelle.
- Wenn du auf einem Rechner keine eigenen ROM-Dateien hast, zeigt das Studio automatisch den eigenen Satz (Etikett **EIGEN**).

## Nachbar-Studios und Glossar

- Aus Zeichen entstehen Bilder im [Bildschirm- und Kachel-Editor](kapitel:bildschirm-kacheln).
- Bewegliche Figuren zeichnest du im [Sprite-Editor](kapitel:sprites), Farben stehen im Kapitel [Paletten](kapitel:paletten).
- Begriffe: Glossar (Datei `GLOSSAR.md`): Zeichensatz, Bildschirmcode, ROM, Hires, Multicolor.

## Häufige Stolpersteine

- **Statt meiner Zeichen erscheint die normale Schrift:** Du hast vergessen, `gfx_charset_at` aufzurufen, oder die Zieladresse liegt nicht im Bereich des VIC (in der ersten 16-KB-Bank sind `$1000`–`$1FFF` das Zeichen-ROM und daher nicht verfügbar).
- **Die Schrift sieht „falsch“ aus:** Vermutlich zeigt der Editor den eigenen Satz von VintageCoder, weil dein ROM fehlt. Der Editor sagt das in der Karte „Herkunft“. ROM ablegen und neu prüfen.
- **Buchstaben stimmen nicht:** Die Reihenfolge ist Bildschirmcode, nicht ASCII (siehe Achtung oben).

## Wie geht es weiter?

Als Nächstes zeichnest du bewegliche Figuren: [Sprite- und PMG-Editor](kapitel:sprites). Die Zeichen setzt du später im [Bildschirm-Editor](kapitel:bildschirm-kacheln) zu Bildern zusammen. Nachschlagen: [Zeichensatz-Editor](hilfe:charset), [Standardzeichensätze und ROMs](hilfe:roms).
