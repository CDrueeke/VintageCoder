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

## Wofür sind die Editoren gedacht?

Zwei Editoren arbeiten hier zusammen, ein dritter kommt später:

- Der **Bildschirm-Editor** gestaltet **einen** Textbildschirm: Titelbilder, Menüs, Anzeigen, Textabenteuer-Seiten.
- Der **Kachel-Editor** baut **Bausteine** (Kacheln) aus mehreren Zeichen: Mauer, Gras, Kiste, Leiter.
- Der **Karten-Editor** (die Screen-Bank) setzt Kacheln zu **Spielfeldern** zusammen: Level, Räume, Weltkarten.

Typische Einsätze aus echten Retro-Projekten:

- **Ein Titelbild** aus Text und Farben, das beim Start erscheint.
- **Ein Plattformspiel:** wenige Kacheln, viele Level; jeder Level ist eine kleine Karte.
- **Ein Rollenspiel mit Weltkarte:** Räume als Screens einer Bank; das Programm wählt den Raum per Nummer.
- **Ein Menüsystem:** mehrere Bildschirme für Hauptmenü, Einstellungen, Highscores.

Im fertigen Programm sind das Datentabellen (Zeichencodes, Farben, Kartenzeilen) plus ein Zeichensatz, den dein Programm in den Speicher kopiert und dem Grafikchip meldet.

## Grundbegriffe ohne Vorwissen

- **Zelle:** ein Platz auf dem Textbildschirm (ein Zeichen groß).
- **Zeichen:** das Bildmuster einer Zelle, aus dem [Zeichensatz](kapitel:zeichensatz).
- **Farb-RAM:** der Speicherbereich, der die Farbe jeder Zelle enthält (C64).
- **Kachel (Tile):** ein Baustein aus Zellen, zum Beispiel 2×2.
- **Kachelsatz:** alle Kacheln eines Projekts, gleich groß.
- **Karte / Screen:** ein Spielfeld aus Kacheln; **Bank:** Datei mit mehreren Screens.
- **Eigenschaft:** ein frei benanntes Merkmal einer Kachel („solide“, „Leiter“), das dein Programm abfragen kann.
- **Zielzeichensatz:** der Zeichensatz, den der Export aus allen benutzten Zeichen der Kacheln baut.
- **Videonorm:** PAL (Europa, 50 Hz) oder NTSC (USA, 60 Hz); beeinflusst Farben und Takt.

## Weitere Begriffe

- Ein **Textbildschirm** ist ein Raster aus **Zellen**. Beim C64 sind es 40×25, beim VC-20 22×23 und beim Atari (ANTIC 2) 40×24; die Größe steht im Profil des Zielsystems. Jede Zelle enthält ein Zeichen aus dem Zeichensatz. Beim C64 bekommt jede Zelle außerdem eine Farbe aus dem **Farb-RAM**.
- Eine **Kachel** (englisch *Tile*) ist ein Baustein aus mehreren Zellen, zum Beispiel 2×2 Zeichen, die zusammen eine Mauer ergeben. Ein **Kachelsatz** (`.vctiles`) enthält viele gleich große Kacheln.
- Eine **Screen-Bank** (`.vcscreen`) ist eine Datei mit einem oder mehreren **Screens**, die aus Kacheln eines Kachelsatzes aufgebaut sind. Sie ist die **Karte**: Sie sagt, welche Kachel wo steht.

Warum Kacheln? Ein ganzer Bildschirm sind 1000 Bytes. Eine Karte aus 2×2-Kacheln (20×12 = 240 Bytes) ist viel kleiner. Du zeichnest wenige Bausteine einmal und setzt sie beliebig oft ein. Das ist die klassische Art, Spielfelder zu bauen.

## Mein erster Bildschirm (am Beispiel `gfx_demo`)

1. **Beispiel öffnen:** Öffne auf der Startseite aus den Beispielprojekten `examples/c64/gfx_demo` (zum Ändern nimmst du **Kopieren**). *Erwartet:* Im Projektbaum liegen `graphics/title.vcscreen`, `level_tiles.vctiles`, `level.vcscreen` und `font.vcchar`.
2. **Bildschirm öffnen:** Wähle oben den Hauptbereich **Grafik** und in der zweiten Reihe **Bildschirm**. Ist noch nichts offen, zeigt der Editor die **Bildschirm-Dateien** als Knöpfe; klicke auf `graphics/title.vcscreen`. Ein Doppelklick im Projektbaum geht genauso. *Erwartet:* Links die Zeichentafel (und beim C64 die Zeichenfarbe), in der Mitte das Titelbild als 40×25-Raster, rechts der Abschnitt **Bildschirm**.
3. **Zeichen ändern:** Werkzeug **Zeichen setzen** (Taste **D**), in der Zeichentafel ein Zeichen wählen, in die Fläche klicken. *Erwartet:* Das Zeichen steht in der Zelle, **●** erscheint in der Fußzeile.
4. **Text tippen:** Werkzeug **Text tippen** (**T**), eine Zelle anklicken und schreiben. *Erwartet:* Deine Buchstaben erscheinen Zelle für Zelle.
5. **Speichern, bauen, starten:** **Cmd/Strg+S**, **F7**, **F5** (mit dem Programm aus Kapitel 12). *Erwartet:* Das geänderte Titelbild im Emulator.

## Aufbau der Oberfläche

Beide Editoren haben drei Bereiche nebeneinander. In schmalen Fenstern rutscht der rechte Bereich unter die beiden anderen.

| Bereich | Bildschirm-Editor | Kachel-Editor |
|---|---|---|
| Links | **Zeichen**: die Zeichentafel mit dem Namen des Zeichensatzes, beim C64 darunter die **Zeichenfarbe**; bei mehreren Bildschirmen die **Bildschirm-Dateien** | der **Kachelsatz**: alle Kacheln als Miniaturen mit Name und Zahl der Verwendungen, **Neue Kachel** (Plus); bei mehreren Sätzen die **Kachelsätze** |
| Mitte | Werkzeugleiste, Hinweiszeile, der Bildschirm, Fußzeile | Werkzeugleiste, die große Kachel mit der Zeichentafel daneben, Fußzeile |
| Rechts | Karte **Bildschirm**: Größe, Farben des Systems (Rahmen, Hintergrund, Zeichen), Zeichensatz, Ausgabe | Karte **Kachel**: Name, Farbe, Hintergrundfarbe, Eigenschaften, Größe der Kacheln, Zielzeichensatz, Kachel-Aktionen |

**Tasten im Bildschirm-Editor:** **D** Zeichen setzen, **R** Rahmen, **F** Füllen, **T** Text tippen, **I** Pipette, **E** Löschen. Im Fenster gelten außerdem **Cmd/Strg+Z** (rückgängig), **Cmd/Strg+Umschalt+Z** (wiederholen) und **Cmd/Strg+S** (speichern). Alle Aktionen findest du auch in der Befehlspalette (**Cmd/Strg+K**). Die Einzeltasten lassen sich in den Einstellungen abschalten („Einzeltasten der Editoren“).

## Schritt für Schritt: einen Textbildschirm gestalten

1. Öffne einen Bildschirm (siehe oben). Einen neuen Bildschirm legt der Dialog **Neue Datei** in dieser Version noch nicht an; bearbeite einen vorhandenen oder lege dir eine Kopie des Beispiels an.
2. In der **Mitte** liegt der Bildschirm. **Links** wählst du das Zeichen aus der **Zeichentafel** (Pfeiltasten gehen von Zeichen zu Zeichen) und beim C64 die **Zeichenfarbe**. **Rechts** stellst du in **Farben des Systems** **Rahmen**, **Hintergrund** und **Zeichen** ein (bei Atari wählst du zusätzlich die Videonorm).
3. Die Werkzeuge: **Zeichen setzen** (Klick setzt das gewählte Zeichen, die rechte Maustaste löscht, **Enter** setzt es an der Cursorzelle), **Rahmen** (ziehen, oder mit **Enter** Ecke und Gegenecke setzen, **Esc** bricht ab), **Füllen**, **Text tippen** (Zelle anklicken, dann schreiben; Pfeiltasten bewegen den Cursor, **Esc** beendet), **Pipette** und **Löschen**.
4. Soll der Bildschirm **deinen** Zeichensatz benutzen, wählst du ihn rechts unter **Zeichensatz**. Mit **Ohne Zeichensatz** erscheint der Systemzeichensatz (oder, ohne ROM, der eigene Satz von VintageCoder).
5. **Cmd/Strg+S** speichert.

![Der Bildschirm-Editor im Bereich Grafik mit der Datei title.vcscreen: links die Zeichenauswahl und die Liste der Bildschirm-Dateien, in der Mitte die 40 mal 25 große Textbildschirm-Fläche mit einem türkisen Rahmen aus Zeichen und weißen und gelben Zeichen, rechts die Karten für Größe, Farben des Systems, Zeichensatz und Ausgabe.](bild:bildschirm-editor "Der Bildschirm-Editor mit Zeichenauswahl, Bildschirm-Fläche und Einstellungen.")

Das Beispielprojekt `examples/c64/gfx_demo` hat so einen Bildschirm (`graphics/title.vcscreen`, mit dem Zeichensatz `graphics/font.vcchar`). Wie du ihn anzeigst, steht schon in [Kapitel 12](kapitel:zeichensatz).

## Schritt für Schritt: Kacheln zeichnen

1. Wähle **Grafik** ▸ **Kacheln**. Ist noch nichts offen, zeigt der Editor die **Kachelsätze** des Projekts als Knöpfe; im Beispiel `gfx_demo` ist es `graphics/level_tiles.vctiles`. Einen neuen Kachelsatz legt der Dialog **Neue Datei** in dieser Version noch nicht an.
2. Der Satz braucht mindestens einen **Zeichensatz**. Rechts unter **Zielzeichensatz** wählst du einen aus der Liste und klickst auf **Hinzufügen**. Ohne Zeichensatz gibt es nichts, womit du Kacheln bauen könntest. Hat der Satz mehrere Zeichensätze, wählst du über der Zeichentafel den Reiter.
3. Links steht der **Kachelsatz**: Miniatur, Name und wie oft die Kachel in Screen-Bänken vorkommt („0×“, wenn nie). Ein **=** meldet, dass eine Kachel gleich einer früheren ist. **Neue Kachel** (Plus über der Liste) legt eine an. Rechts in der Karte **Kachel** stehen **Duplizieren**, **Zusammenführen** (bei gleichen Kacheln) und **Löschen**. Wird die Kachel in Screen-Bänken benutzt, fragt der Editor vor dem Löschen.
4. In der **Mitte** ist die große Kachel. **Linke Maustaste** (auch ziehen) setzt das gewählte Zeichen in die Zelle, die **rechte Maustaste** (oder **Alt**+Klick) nimmt das Zeichen einer Zelle als Pinsel auf. Mit den Pfeiltasten wanderst du von Zelle zu Zelle, **Enter** setzt das Zeichen, **I** nimmt es auf. Das Zeichen wählst du in der **Zeichentafel** neben der Kachel.
5. In der Werkzeugleiste stehen **Spiegeln** (waagerecht, senkrecht), **Kachel drehen** (**90°**, nur bei quadratischen Kacheln), **Zellen rollen** (vier Pfeile), **Leeren**, **Kopieren** und **Einfügen**. Die **Größe** des ganzen Satzes stellst du rechts unter **Größe der Kacheln** ein (**Breite** und **Höhe**, 1 bis 16 Zeichen je Seite, dann **Übernehmen**; beim Verkleinern gehen Zellen verloren, der Editor fragt vorher).
6. Die **Farbe** stellst du für den ganzen Satz rechts unter **Farbe** ein: **Je Kachel** (ein Farbwert für die ganze Kachel, am kompaktesten), **Je Zelle** (jede Zelle eine eigene Farbe; dann wählst du über der Kachel, ob ein Klick **Zeichen und Farbe**, **Nur Zeichen** oder **Nur Farbe** setzt) oder **Ohne**. Dazu kommt die **Hintergrundfarbe**.

![Der Kachel-Editor im Bereich Grafik mit der Datei level_tiles.vctiles: links der Kachelsatz mit den Kacheln leer, stein, gras und kiste, in der Mitte die große 2 mal 2 Kachel und der Zeichenwähler mit einem Raster, rechts die Karten Kachel, Farbe, Eigenschaften, Größe der Kacheln, Zielzeichensatz und Kachel.](bild:kachel-editor "Der Kachel-Editor mit Kachelsatz, Kachelfläche und Zeichenwähler.")

### Eigenschaften und Füllstand

Rechts unter **Eigenschaften** vergibst du frei benannte Merkmale, zum Beispiel „solide“ oder „Leiter“ (höchstens acht je Satz): Name in **Neue Eigenschaft** eintippen, **Hinzufügen**. Pro Kachel setzt du den Schalter. Ein Spiel kann später abfragen, ob man auf einer Kachel stehen darf. Der Export schreibt sie als Tabelle `<name>_props` und als Konstanten `<name>_p_<eigenschaft>`.

Alle benutzten Zeichen aller Kacheln ergeben beim Export **einen** gemeinsamen **Zielzeichensatz**. Gleiche Bilder zählen nur einmal. Der Balken unter **Zielzeichensatz** zeigt, wie voll er ist (zum Beispiel „187 von 256 Zeichen belegt“) und meldet einen Überlauf. Beim C64, Atari und MEGA65 gibt es den Schalter **Zeichenbit 7 setzen (Farbwahl des Zeichens)**.

## Die Karte: Screen-Bank und Karten-Editor

Öffnest du eine Screen-Bank (zum Beispiel `graphics/level.vcscreen` aus `gfx_demo`), erscheint der Karten-Editor automatisch.

- **Screens unten:** Unter der Karte liegt die Leiste mit allen Karten (Screens) der Bank, jede mit Vorschaubild und Name. Ein Klick wählt sie; **Neuer Screen** hängt eine leere Karte an, **Screen kopieren**, **Screen löschen** (Rückgängig stellt ihn wieder her) und **Nach vorn/Nach hinten** ordnen die Reihenfolge; die Reihenfolge ändern kannst du auch, indem du einen Screen in der Leiste an eine andere Stelle ziehst (ein Rückgängig-Schritt). Die Nummer („Screen 2 von 4“) ist die Auswahlnummer, mit der dein Programm die Karte wählt.
- **Kachelmodus:** Links liegt das **Kachelraster** des Kachelsatzes. Ein Klick wählt den Pinsel. Werkzeuge (Tasten in Klammern): Stift (**D**), Linie (**L**), Rechteck (**R**, gefüllt oder als Rahmen), Füllen (**F**), Pipette (**I**, auch rechte Maustaste), Auswahl (**A**), Stempel (**S**), Löschen (**E**, setzt Kachel 0). Beim Ziehen leuchten die berührten Zellen sofort auf; ein ganzer Zug ist **ein** Rückgängig-Schritt.
- **Auswahl:** Ziehe mit dem Auswahl-Werkzeug ein Rechteck. Dann kannst du es **als Stempel merken** (das Stempel-Werkzeug setzt es überall wieder ein), **leeren** (Entf), mit **Alt+Pfeiltasten** verschieben oder **als Kachel anlegen …** (ein Dialog fragt nach dem Namen und ob die Auswahl auf der Karte durch die neue Kachel ersetzt werden soll; gibt es die Zellen schon als Kachel, wird sie benutzt).
- **Zellenmodus:** Mit der Taste **Tab** (oder dem Schalter **Kacheln | Zellen**) wechselst du auf die **Zellebene**: Links erscheint die Zeichentafel; du setzt einzelne Zeichen **über** die Kacheln (höchstens 255 solche Ausnahmen je Karte). Das Werkzeug **Löschen** lässt die Kachel darunter wieder durchscheinen; das Text-Werkzeug (**T**) setzt getippte Zeichen ab dem Cursor.
- **Ebenen:** Links schaltest du **Kachelebene** und **Zellenebene** einzeln ein und aus und lässt mit **Eigenschaften zeigen** eine gewählte Eigenschaft (zum Beispiel „solid“) farbig einblenden.
- Rechts stellst du **Name**, **Kachelsatz**, **Größe** und **Ursprung** des Kachelrasters ein und wählst die Farben der Karte. Was nicht aufgeht, zum Beispiel die 25. Zeile beim C64 mit 2×2-Kacheln, bleibt Restzelle (nur der Zellebene vorbehalten). Die Ansicht lässt sich auf **PAL** oder **NTSC** stellen.
- **Ansicht:** In der Werkzeugleiste zoomst du (**−**, **+**, „Auf die Breite einpassen“) und zeigst **Zellgitter** und **Kachelgrenzen**. Mit den Pfeiltasten bewegst du den Cursor über die Karte; die Statuszeile nennt Spalte, Zeile und Kachel.

![Der Bereich „Grafik“ mit dem Reiter „Karten“ im Projekt gfx_demo. Links die vier Kacheln des Kachelsatzes und die Schalter für Ebenen, in der Mitte die Karte „level“ mit Boden aus Kacheln und Grasstreifen, rechts die Felder zu Name, Größe, Farben und Ausgabe. Unter der Karte stehen die Level.](bild:kachel-karte "Der Karten-Editor mit Kachelsatz, Karte und den Einstellungen zur Karte.")

### Aus einem Textbildschirm wird eine Karte

Hast du einen Textbildschirm gemalt, der sich wiederholende Blöcke enthält, wandelt **In Kacheln zerlegen …** (Werkzeugleiste des Bildschirm-Editors) ihn um. Im Dialog **In Kacheln zerlegen** wählst du die **Kachelgröße** (2 × 2, 3 × 3 oder 4 × 4); die Vorschau nennt das Ergebnis vorher. **Zerlegen** legt `<name>_tiles.vctiles` und `<name>_bank.vcscreen` neben den alten Screen; der alte bleibt bestehen. Danach öffnet der Kachel-Editor den neuen Kachelsatz.

## Export

Wie überall: Werkzeugleiste ▸ **Exportieren …** (oder **Cmd/Strg+Umschalt+E**) oder automatisch beim Bauen. Ergebnis:

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
.include "graphics/level_tiles.vctiles"
.include "graphics/level.vcscreen"
```

- Zuerst kommt der **Zielzeichensatz** des Kachelsatzes (`level_tiles_chars`) in den Speicher, denn die Kacheln bestehen aus **seinen** Zeichen.
- `tiles_use` verbindet den Kachelsatz mit der Bank. Die langen Namen sind Marken aus den beiden Exportdateien. Es ist die Zeile, die man am häufigsten abschreibt.
- `gfx_draw_map` zeichnet den gewählten Screen (hier `level_level_id`, die Nummer 0) in den Bildschirmspeicher und das Farb-RAM.
- `tiles_runtime` und `gfx_runtime` legen die Unterprogramme ab. Beide stehen **hinter** dem Hauptprogramm.

1. Öffne `examples/c64/gfx_demo`, ersetze `gfx_demo.asm` durch das Programm oben (oder arbeite in einer Kopie).
2. Bauen mit **F7**, starten mit **F5**. Du siehst ein Spielfeld mit Steinboden, einer Grasreihe, einer Steinsäule und einer schwebenden Kiste.
3. Öffne `graphics/level_tiles.vctiles`, ändere im Kachel-Editor eine Kachel (zum Beispiel die Kiste), speichere, baue und starte neu. Die Karte zeigt die geänderte Kachel überall dort, wo sie steht.

Das Original `gfx_demo.asm` zeigt per Schalter `-D DEMO_MAP=1` die Karte statt des Titelbildes. Ein größeres Beispiel mit Eigenschaften und vier Karten ist `examples/c64/jumprun`.

## Hardware-Besonderheiten und Grenzen

| System | Bildschirm | Zeichen / Farbe | Besonderheit |
|---|---|---|---|
| C64 | 40×25 Zellen, 1000 Bytes + 1000 Farben | 256 Zeichen, 16 Farben | Farb-RAM `$D800`; Bildschirmspeicher und Zeichensatz müssen in derselben 16-KB-Bank des VIC liegen |
| VC-20 | 22×23 | 256 Zeichen | wenig Speicher; der Speicherausbau bestimmt die Adressen |
| Plus/4 | 40×25 | 256 Zeichen, 121 Farben (TED) | TED-Palette im Editor noch nicht vollständig (Hinweis „TED-Palette fehlt“) |
| Atari (ANTIC 2) | 40×24 | 128 Zeichen | Inversdarstellung über Bit 7 |
| MEGA65 | Zeilen und Spalten je nach Profil | wie C64, zusätzlich Vollfarbzeichen | 256-Farben-Palette |

Grenzen der Kacheln: höchstens **16** Zeichen je Kachelseite, höchstens **255** Zellausnahmen je Screen, Kachelsatz bis zu 256 Zeichenbildern im Zielzeichensatz. Reichen 256 Zeichen nicht, gleichst du Zeichen zusammen oder nutzt weniger verschiedene Bausteine.

## Vom Editor ins Programm

- **Assembler:** `.include "graphics/<name>.vctiles"` bzw. `.vcscreen` hinter dem Hauptprogramm, dazu `lib/gfx.inc` (`gfx_load_screen`, `gfx_draw_map` …), siehe die Programme in diesem Kapitel.
- **BASIC:** Export im Format **BASIC DATA**, mit `#include` einbinden und mit `READ`/`POKE` in den Bildschirmspeicher kopieren.
- **Rohdaten:** Format **Binär** für `.incbin` oder zum Laden von Diskette.
- **Atari und MEGA65:** gleiche Dateien, andere Zieladressen; für den Atari zeigt `examples/atari/gfx_demo` den Weg.

## Häufige Stolpersteine

- **Karte zeigt Unsinn:** Der Zielzeichensatz wurde nicht geladen oder liegt nicht auf einer 2-KB-Grenze. Er muss mit `gfx_charset_at` angemeldet sein.
- **Die Karte wird nicht gezeichnet:** `tiles_use` fehlt oder nennt falsche Marken, oder `tiles_runtime` fehlt hinter dem Programm.
- **„Überlauf“ im Zielzeichensatz:** Du benutzt mehr als 256 verschiedene Zeichenbilder. Lösche Kacheln oder gleiche Zeichen zusammen (**Zusammenführen** bei gleichen Kacheln).
- **Im Kachel-Editor lässt sich nichts zeichnen:** Dem Satz fehlt ein Zeichensatz. Rechts unter **Zielzeichensatz** erst einen hinzufügen.

## Nachbar-Editoren und Glossar

- Zeichen malst du im [Zeichensatz-Editor](kapitel:zeichensatz), Farben wählst du aus der [Palette](kapitel:paletten), bewegliche Figuren sind [Sprites](kapitel:sprites).
- Begriffe: Glossar (Datei `GLOSSAR.md`): Kachel, Karte, Bank, Zelle, Farb-RAM.

## Wie geht es weiter?

Wenn du eigene Farben bestimmen willst (MEGA65), geht es im nächsten Kapitel um [Paletten](kapitel:paletten). Nachschlagen: [Bildschirm-Editor](hilfe:screen), [Kachel-Editor](hilfe:tiles), [Export](hilfe:export).
