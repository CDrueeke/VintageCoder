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

## Wofür ist der Editor gedacht?

Im Zeichensatz-Editor malst du die **Buchstaben und Grafikzeichen** deines Programms. Weil ein Textbildschirm nur aus Zeichen besteht, ist ein eigener Zeichensatz der einfachste Weg zu einer eigenen Optik, ohne Sprites oder Bitmap-Grafik.

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
- **ROM:** Festspeicher des Rechners. Die Original-ROM-Dateien gehören nicht in dein Projekt.
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

1. **Beispiel öffnen:** Öffne auf der Startseite aus den Beispielprojekten `examples/c64/gfx_demo` (zum Ändern nimmst du **Kopieren**, dann hast du eine eigene Fassung). *Erwartet:* Das Projekt ist offen; im Projektbaum liegt `graphics/font.vcchar`.
2. **Zeichensatz öffnen:** Wähle oben den Hauptbereich **Grafik** und in der zweiten Reihe **Zeichensatz**. Ist noch nichts offen, zeigt der Editor die **Zeichensatz-Dateien** des Projekts als Knöpfe; klicke auf `graphics/font.vcchar`. Ein Doppelklick auf die Datei im Projektbaum geht genauso. *Erwartet:* Links die **Zeichenübersicht** mit allen 256 Zeichen, in der Mitte das gewählte Zeichen stark vergrößert, rechts die Vorschau und die Karten.
3. **Zeichen wählen:** Klick in die Übersicht. *Erwartet:* Über der Fläche steht die Nummer, zum Beispiel „Zeichen 1 · $01“, daneben das passende Zeichen. Mit **Bild auf** und **Bild ab** (oder **Alt+Pfeil links/rechts**) gehst du zum vorigen und nächsten Zeichen.
4. **Malen:** Wähle den **Stift** (Taste **D**). Die linke Maustaste setzt einen Punkt, die rechte löscht ihn. *Erwartet:* Die kleine Kachel in der Übersicht und die **Vorschau im Bild** zeigen dein Zeichen sofort mit, über dem Namen erscheint **●** („ungespeichert“).
5. **Rückgängig:** **Cmd/Strg+Z**; **Cmd/Strg+Umschalt+Z** holt den Schritt zurück.
6. **Speichern:** **Cmd/Strg+S**. *Erwartet:* Das **●** verschwindet.
7. **Bauen und starten:** **F7**, **F5**. *Erwartet:* Das geänderte Zeichen erscheint im Emulator auf dem Titelbild.

Einen ganz neuen Zeichensatz legst du mit **Neuer Zeichensatz …** an (Knopf in der Werkzeugleiste, auch auf der leeren Seite des Editors). Im Dialog **Neuer Zeichensatz** wählst du unter **Art und System** das Profil (für den Einstieg den C64-Zeichensatz hires 8×8), gibst den **Namen** ein (Buchstaben, Ziffern, `_` und `-`), prüfst den **Ordner im Projekt** (Vorgabe `graphics`) und klickst auf **Anlegen**. Die Datei öffnet gleich im Editor.

## Aufbau der Oberfläche

Der Zeichensatz-Editor hat drei Bereiche nebeneinander. In schmalen Fenstern rutscht der rechte Bereich unter die beiden anderen.

| Bereich | Inhalt |
|---|---|
| Links: **Zeichenübersicht** | alle Zeichen als Kacheln, 16 je Zeile; darunter **Bereiche** (Alle, Buchstaben, Ziffern, Satzzeichen, Frei) und, wenn das Projekt mehrere Zeichensätze hat, die **Zeichensatz-Dateien** |
| Mitte | Werkzeugleiste, Nummer des Zeichens, die Zeichenfläche mit der **Umgebung** (das Zeichen davor und danach), Fußzeile mit Name, Zeichen und ungespeicherten Änderungen |
| Rechts | **Vorschau im Bild**, **Vorlage** mit **Herkunft**, **Zeichen** (Nummer und Position), **Farben** und **Verschieben** |

Die Werkzeugleiste enthält, von links nach rechts: **Stift** (D), **Radierer** (E), **Pipette** (I), **Füllen** (F), **Linie** (L), **Rechteck** (R), **Auswahl** (S), Rückgängig und Wiederholen, **Waagerecht spiegeln** (X) und **Senkrecht spiegeln** (Y), dann **Raster** (G), **Verkleinern** (−), die Zoomanzeige, **Vergrößern** (+), **Einpassen** (0) und rechts davon die Dateiknöpfe **Neuer Zeichensatz …**, **PNG laden …**, **Exportieren …** und **Speichern**. Die Knöpfe zeigen beim Daraufzeigen ihre Taste.

![Der Zeichensatz-Editor im Bereich „Grafik“ mit der Datei font.vcchar. Links die Zeichenmatrix „FONT“ mit 256 Zeichen und den Bereichen „Alle“, „Buchstaben“, „Ziffern“, „Satzzeichen“ und „Frei“. In der Mitte die große Zeichenfläche mit Werkzeugleiste, rechts die Karten „VORSCHAU IM BILD“, „VORLAGE“, „ZEICHEN“, „FARBEN“ und „VERSCHIEBEN“.](bild:zeichensatz-ueberblick "Der Zeichensatz-Editor mit Zeichenmatrix, Zeichenfläche und den Karten rechts.")

Die Einzeltasten gelten, wenn die Zeichenfläche den Fokus hat:

| Taste | Wirkung |
|---|---|
| D, E, I, F, L, R, S | Stift, Radierer, Pipette, Füllen, Linie, Rechteck, Auswahl |
| X, Y | waagerecht, senkrecht spiegeln |
| 1 bis 4 | Pixelwert wählen (Ziffern bis 9, soweit der Satz so viele Werte hat) |
| G | Raster ein oder aus |
| + / − / 0 | Vergrößern / Verkleinern / Einpassen (auch **Cmd/Strg** und Mausrad) |
| Pfeiltasten | Zellmarke bewegen; **Eingabe** oder **Leertaste** setzt einen Punkt, **Entf** löscht ihn |
| Bild auf / Bild ab, Alt+Pfeil links/rechts | voriges / nächstes Zeichen |
| Cmd/Strg+Z, Cmd/Strg+Umschalt+Z | Rückgängig, Wiederholen |
| Cmd/Strg+S | Speichern |
| Cmd/Strg+Umschalt+E | Export-Dialog |

Mit dem Werkzeug **Auswahl** ziehst du einen Rahmen auf; **Cmd/Strg+C**, **X** und **V** kopieren, schneiden aus und fügen ein, **Entf** löscht die Auswahl, **Esc** hebt sie auf. Ein Rechtsklick mit dem Auswahlwerkzeug öffnet ein Menü mit denselben Aktionen.

Zum Verschieben des ganzen Zeichens gibt es rechts unter **Verschieben** vier Pfeilknöpfe (links, rechts, oben, unten). **Zeichen leeren** löscht alle Punkte des gewählten Zeichens.

### Bereiche und Vorschau

Die **Bereiche** springen zum ersten Zeichen einer Gruppe und markieren die Gruppe in der Übersicht. Bei **Frei** gibst du **von** und **bis** als Zeichennummern ein. So findest du zum Beispiel schnell alle Ziffern.

Die **Vorschau im Bild** zeigt einen Text in den Zeichen deines Satzes, so wie er auf dem Bildschirm aussähe. Den Text änderst du im Feld darunter (**Text für die Vorschau**). Die Vorschau folgt jeder Änderung sofort.

### Die Vorlage und die Herkunft

Ein neuer Zeichensatz beginnt **nicht leer**: Er enthält einen Verweis auf die Schrift deines Systems, wie sie nach dem Einschalten erscheint (beim C64 „Großbuchstaben und Grafik“). In der Datei stehen nur **deine Änderungen**. Die Original-Zeichen liegen nicht in deinem Projekt, sondern bleiben im ROM. Hast du kein ROM, steht dahinter der eigene Satz von VintageCoder.

Rechts unter **Vorlage** wählst du in der Liste **Vorlage zum Laden** eine andere Grundschrift (beim C64 etwa die mit Klein- und Großbuchstaben) und klickst auf **Vorlage laden**. Ein Eintrag, dessen ROM nicht gefunden wird, ist grau und trägt den Zusatz „ROM nicht gefunden“. Neben den Schriften deiner ROMs steht immer auch **Standard (eigen)**: der mitgelieferte Satz von VintageCoder, als **Kopie** in deiner Datei. Das Laden ersetzt alle Zeichen; **Cmd/Strg+Z** macht es rückgängig. Soll dein Zeichensatz bei Null beginnen, lege ihn auf der Kommandozeile mit `vcc media new … --blank` an.

Darunter sagt die Karte **Herkunft** in Worten, woher die Zeichen stammen, mit einem Etikett: **EIGEN**, **EIGEN, GEÄNDERT**, **AUS DEINEM ROM** oder **ROM FEHLT**. Ein Satz, den du selbst gestaltet hast, trägt den Hinweis „Selbst gestaltet, keine Vorlage geladen“. Ist eine Vorlage geladen, zeigt der Editor außerdem, wie viele Zeichen von ihr abweichen („Gegenüber der Vorlage geändert“ oder „Wie die Vorlage“ für das gewählte Zeichen). Geänderte Zeichen haben in der Übersicht einen kleinen Punkt. Der Knopf **Zeichen auf Vorlage zurücksetzen** holt das Original des gewählten Zeichens zurück.

*[Bild folgt in E13: zeichensatz-standard]*

### Multicolor malen

Bei einem Multicolor-Profil bekommt jeder Punkt einen von vier **Pixelwerten**. Die Karte **Farben** listet jeden Wert mit seiner Farbe; welches Register welche Farbe bestimmt, steht beim jeweiligen Wert (beim C64 Hintergrund `$D021`, dazu `$D022` und `$D023`; die vierte Farbe kommt aus dem **Farb-RAM `$D800`**). Den Wert wählst du per Klick oder Taste 1 bis 4. Darunter wählst du für den gewählten Wert die Farbe aus dem Farbraster des Systems. Drei der Farben (Hintergrund und zwei weitere) gelten für den ganzen Bildschirm, die vierte kommt aus dem Farbspeicher der jeweiligen Bildschirmzelle. Bei Systemen mit PAL- und NTSC-Farben (Atari) stellst du die **Palettenansicht** zwischen **Projekt**, **PAL** und **NTSC** um.

*[Bild folgt in E13: zeichensatz-multicolor]*

### MEGA65: Vollfarbzeichen

Beim Profil `mega65.char.fcm` ist jeder Punkt eine von 256 Palettenfarben. Welche Farben das sind, bestimmt die Palette, siehe [Paletten](kapitel:paletten).

## Eigene Standardzeichensätze (VC-Standard)

VintageCoder bringt für jedes System **eigene Zeichensätze** mit (C64, C128 mit VDC, VC-20, Plus/4, Atari mit Standard und International, MEGA65). Sie sind **kein Nachbau** der Original-ROMs, sondern eigene Entwürfe der Zeichenformen, frei unter der Lizenz CC0 (gemeinfrei). Du darfst sie in eigenen Programmen und Projekten benutzen und weitergeben.

- **Neues Projekt:** Im Abschnitt **Zeichensatz des Projekts** wählst du **Automatisch** (dein ROM als Verweis, sonst der eigene Satz), **Nur Standard (eigen)** (immer der eigene Satz, mit `HERKUNFT.txt` unter `graphics/vc-standard/`: das Projekt lässt sich weitergeben) oder **Keinen** (einen Zeichensatz legst du dann später im Zeichensatz-Editor an). Auf der Kommandozeile: `vcc new … --charset auto|own|none`.
- **Zeichensatz-Editor:** Die Vorlage **Standard (eigen)** kopiert den Satz in deine Datei. Die Karte **Herkunft** sagt, woher die Zeichen stammen.
- **Rangfolge:** dein ROM, dann der eigene Satz, dann nichts. Fehlt auf einem anderen Rechner das ROM, auf das ein Verweis zeigt, springt VintageCoder auf den eigenen Satz und sagt es.

> **Hinweis:** Die Herkunft jedes Satzes steht in der Datei (`origin`). Das ist ein Entstehungsvermerk.

> **Info:** VintageCoder liefert keine Original-ROMs mit und lädt auch keine aus dem Netz. Wo du deine eigenen Abbilder ablegst, steht im Kapitel [Disk-Editor, Segmente, ROMs](kapitel:disk-studio).

Das Beispielprojekt `examples/c64/standard_charsets` enthält die **freien** Open-ROMs-Zeichensätze (`openroms_c64_gross.vcchar`, `openroms_c64_klein.vcchar`). Die darfst du weitergeben.

## Zeichensätze aus anderen Quellen

In dieser Version kommst du so zu einem Zeichensatz:

- **Neuer Zeichensatz …** mit der Vorlage deines ROMs oder **Standard (eigen)** (siehe oben).
- **PNG laden …** (Werkzeugleiste): Du wählst eine Bilddatei; der Editor wandelt sie in die Pixelwerte des gewählten Zeichens um und speichert dabei die Datei. So kannst du ein einzelnes Zeichen in einem Bildprogramm malen.

Die Befehle **Standardzeichensatz hinzufügen …**, **Zeichensatz aus ROM importieren …** und **Zeichensatz aus PNG-Raster importieren …** (Menü **Datei**) und der Export als PNG-Raster stehen in dieser Version noch nicht zur Verfügung.

## Zeichensatz exportieren

**Exportieren …** (Werkzeugleiste, auch **Cmd/Strg+Umschalt+E**) öffnet den Export-Dialog. Dort wählst du das **Format**: **Assembler**, **Konstanten**, **Binär**, **PRG** oder **BASIC DATA**. Dazu kommen der **Name** und, je nach Format, eine **Adresse** (beim PRG die **Ladeadresse**). Bei **BASIC DATA** kannst du **Einleseroutine mitliefern** ankreuzen. Die **Vorschau** zeigt den Text und, außer bei BASIC DATA, die Bytes als **Hex**; darunter steht, wie viele Dateien und Bytes entstehen.

Unten wählst du, was mit dem Ergebnis geschieht:

- **In Projekt speichern** schreibt die Dateien in den Ordner `build/export`.
- **Als Include einfügen** setzt die `.include`-Zeile in den Quelltext, den du unter **Ziel im Quelltext** gewählt hast (Assembler und BASIC DATA).
- **In Quelltext kopieren** schreibt den Text selbst an die Cursorzeile dieses Quelltexts, ohne Include.
- **Im Emulator testen** speichert den Export und baut und startet das Projekt.

Zum Einbinden brauchst du den Export nicht: Beim Bauen (**F7**) wandelt VintageCoder den Zeichensatz beim `.include` selbst um (siehe das Programm unten). Das Einfügen in den Quelltext klappt nur, wenn ein passender Quelltext geöffnet ist.

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
.include "graphics/font.vcchar"
.include "graphics/title.vcscreen"
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
- Wenn du auf einem Rechner keine eigenen ROM-Dateien hast, zeigt der Editor automatisch den eigenen Satz (Etikett **EIGEN**).

## Nachbar-Editoren und Glossar

- Aus Zeichen entstehen Bilder im [Bildschirm- und Kachel-Editor](kapitel:bildschirm-kacheln).
- Bewegliche Figuren zeichnest du im [Sprite-Editor](kapitel:sprites), Farben stehen im Kapitel [Paletten](kapitel:paletten).
- Begriffe: Glossar (Datei `GLOSSAR.md`): Zeichensatz, Bildschirmcode, ROM, Hires, Multicolor.

## Häufige Stolpersteine

- **Statt meiner Zeichen erscheint die normale Schrift:** Du hast vergessen, `gfx_charset_at` aufzurufen, oder die Zieladresse liegt nicht im Bereich des VIC (in der ersten 16-KB-Bank sind `$1000`–`$1FFF` das Zeichen-ROM und daher nicht verfügbar).
- **Die Schrift sieht „falsch“ aus:** Vermutlich zeigt der Editor den eigenen Satz von VintageCoder, weil dein ROM fehlt. Der Editor sagt das in der Karte **Herkunft** (Etikett **ROM FEHLT**). ROM ablegen und neu prüfen.
- **Buchstaben stimmen nicht:** Die Reihenfolge ist Bildschirmcode, nicht ASCII (siehe Achtung oben).
- **Die Pfeiltasten wechseln das Zeichen nicht:** Auf der Zeichenfläche bewegen sie die Zellmarke. Zum nächsten Zeichen kommst du mit **Bild auf/ab**, **Alt+Pfeil links/rechts** oder per Klick in die Übersicht.

## Wie geht es weiter?

Als Nächstes zeichnest du bewegliche Figuren: [Sprite- und PMG-Editor](kapitel:sprites). Die Zeichen setzt du später im [Bildschirm-Editor](kapitel:bildschirm-kacheln) zu Bildern zusammen. Nachschlagen: [Zeichensatz-Editor](hilfe:charset), [Standardzeichensätze und ROMs](hilfe:roms).
