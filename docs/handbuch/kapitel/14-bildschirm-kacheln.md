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

## Wofür sind die Studios gedacht?

Drei Studios arbeiten hier zusammen:

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
- **Restzelle:** eine Zelle, die nicht in das Kachelraster passt (zum Beispiel die 25. Zeile bei 2×2).
- **Videonorm:** PAL (Europa, 50 Hz) oder NTSC (USA, 60 Hz); beeinflusst Farben und Takt.

## Weitere Begriffe

- Ein **Textbildschirm** ist ein Raster aus **Zellen**. Beim C64 sind es 40×25, beim VC-20 22×23 und beim Atari (ANTIC 2) 40×24; die Größe steht im Profil des Zielsystems. Jede Zelle enthält ein Zeichen aus dem Zeichensatz. Beim C64 bekommt jede Zelle außerdem eine Farbe aus dem **Farb-RAM**.
- Eine **Kachel** (englisch *Tile*) ist ein Baustein aus mehreren Zellen, zum Beispiel 2×2 Zeichen, die zusammen eine Mauer ergeben. Ein **Kachelsatz** (`.vctiles`) enthält viele gleich große Kacheln.
- Eine **Screen-Bank** (`.vcscreen`) ist eine Datei mit einem oder mehreren **Screens**, die aus Kacheln eines Kachelsatzes aufgebaut sind. Sie ist die **Karte**: Sie sagt, welche Kachel wo steht.

Warum Kacheln? Ein ganzer Bildschirm sind 1000 Bytes. Eine Karte aus 2×2-Kacheln (20×12 = 240 Bytes) ist viel kleiner. Du zeichnest wenige Bausteine einmal und setzt sie beliebig oft ein. Das ist die klassische Art, Spielfelder zu bauen.

## Mein erster Bildschirm (am Beispiel `gfx_demo`)

1. **Beispiel öffnen:** Startseite ▸ Beispiele ▸ `examples/c64/gfx_demo`. *Erwartet:* im Baum `graphics/title.vcscreen`, `level_tiles.vctiles`, `level.vcscreen`, `font.vcchar`.
2. **Bildschirm öffnen:** Doppelklick auf `title.vcscreen`. *Erwartet:* Das Titelbild erscheint als 40×25-Raster, rechts Karten für Zeichen, Farben, Hintergrund und Rahmen.
3. **Zeichen ändern:** Werkzeug **Zeichen** (**D**), in der Zeichenauswahl ein Zeichen wählen, in die Fläche klicken. *Erwartet:* Das Zeichen steht in der Zelle, **●** erscheint.
4. **Text tippen:** Werkzeug **Text** (**T**), in die Fläche klicken und schreiben. *Erwartet:* Deine Buchstaben erscheinen Zelle für Zelle.
5. **Speichern, bauen, starten:** **Cmd/Strg+S**, **F7**, **F5** (mit dem Programm aus Kapitel 12). *Erwartet:* Das geänderte Titelbild im Emulator.

## Aufbau der Oberfläche

Alle drei Studios folgen den **sechs Zonen** (Kopfzeile, Werkzeugleiste, Liste, Fläche, Karten, Fußzeile):

| Zone | Bildschirm (**SCR**) | Kachelsatz (**MAP**) | Karte/Bank (**BNK**) |
|---|---|---|---|
| Kopfzeile | Name, **●**, Maße, Speichern, Exportieren, „Zerlegen …“ | Name, **●**, Größe, Speichern, Exportieren | Name, **●**, Screen-Nummer, Speichern, Exportieren |
| Leiste | Zeichen, Text, Füllen, Rahmen, Pipette, Löschen, Zoom | Spiegeln, Drehen, Rollen, Leeren, Kopieren, Einfügen | Werkzeuge, Modus Kacheln/Zeichen (**Tab**), Ansicht |
| Liste | Zeichenauswahl | Kachelliste mit Miniaturen und Benutzung | Screens der Bank |
| Fläche | der Bildschirm | die große Kachel | die Karte |
| Karten | Zeichensatz, Farben, Hintergrund, Rahmen | Zeichenwähler, Farbe, Eigenschaften, Zielzeichensatz | Kachelpalette, Größe, Ursprung, Ebenen |
| Fußzeile | Zelle, Zeichencode, Farbe | Kachel x von y, Füllstand | Zelle, Kachel, Zeichensatz-Füllstand |

**Tasten:** Bildschirm: **D** Zeichen, **T** Text, **F** Füllen, **R** Rahmen, **I** Pipette, **E** Löschen. Alle Studios: **0** einpassen, **+** / **−** zoomen (auch Cmd/Strg+Mausrad), **Cmd/Strg+Z** zurück, **Cmd/Strg+Umschalt+Z** wiederholen. Bank: **Tab** wechselt Kacheln ↔ Zeichen, **Entf** leert die Auswahl. Alle Aktionen findest du auch in der Befehlspalette (**Cmd/Strg+K**, „Studio: …“).

## Schritt für Schritt: einen Textbildschirm gestalten

1. **＋** in der linken Spalte ▸ im Dialog **Neue Datei** die Art **Bildschirme**, Profil **C64 Textbildschirm 40×25**, Namen `title` eingeben, **OK**.
2. Im Editor liegt in der **Mitte** der Bildschirm. **Rechts** wählst du das Zeichen aus dem **Zeichensatz** und die **Farbe**. Außerdem stellst du dort **Hintergrund** und **Rahmen** ein (bei Atari die **Videonorm**).
3. Die Werkzeuge: **Zeichen setzen**, **Text tippen** (du schreibst einfach mit der Tastatur), **Füllen**, **Rahmen**, **Pipette** und **Löschen**.
4. Soll der Bildschirm **deinen** Zeichensatz benutzen, wählst du ihn rechts unter **Zeichensatz**. Ohne Auswahl erscheint der Systemzeichensatz (oder, ohne ROM, der eigene Satz von VintageCoder).
5. **Strg/Cmd+S** speichert.

![Bildschirm-Studio mit einem Titelbild: Werkzeugleiste oben, der 40×25-Bildschirm in der Mitte, rechts die Karten Werkzeug (Zeichenauswahl und Farbe) und Dokument (Zeichensatz)](bild:bildschirm-editor "Das Bildschirm-Studio: mittig der 40×25-Bildschirm, rechts Zeichen, Farbe und Zeichensatz")

Das Beispielprojekt `examples/c64/gfx_demo` hat so einen Bildschirm (`graphics/title.vcscreen`, mit dem Zeichensatz `graphics/font.vcchar`). Wie du ihn anzeigst, steht schon in [Kapitel 12](kapitel:zeichensatz).

## Schritt für Schritt: Kacheln zeichnen

1. **＋** ▸ im Dialog **Neue Datei** die Art **Kacheln**, Profil **C64 Kachelsatz**, Namen `level_tiles`, **OK**.
2. Rechts musst du dem Satz mindestens einen **Zeichensatz** geben (Reiter **Zeichensatz**, unten **+ Zeichensatz**). Ohne Zeichensatz gibt es nichts, womit du Kacheln bauen könntest.
3. Links steht die **Kachelliste**: Miniatur, Nummer, Name und wie oft die Kachel in Karten vorkommt („unbenutzt“, wenn nie). **Neu** legt eine Kachel an, **Dupl.** kopiert, **Löschen** entfernt (gibt es Karten, die den Satz benutzen, fragt der Editor vorher). Ein Suchfeld filtert nach Name oder Nummer.
4. In der **Mitte** ist die große Kachel. **Linke Maustaste** setzt das gewählte Zeichen in die Zelle, **rechte Maustaste** nimmt das Zeichen einer Zelle als Pinsel auf. Das Zeichen wählst du im **Zeichenwähler** rechts.
5. In der Werkzeugleiste stellst du die **Größe** des ganzen Satzes ein (Schnellwahl 2×2, 3×3, 4×4, 3×2 oder frei, 1 bis 16 Zeichen je Seite). Dazu **Spiegeln**, **Drehen** (nur bei quadratischen Kacheln), **Rollen**, **Leeren**, **Kopieren** und **Einfügen** von Zellen.
6. Die **Farbe** stellst du für den ganzen Satz ein: **je Kachel** (ein Farbwert für die ganze Kachel, am kompaktesten), **je Zelle** (jede Zelle eigene Farbe) oder **keine**.

![Kachelsatz-Studio mit der Kachelliste und der Kachel in der Mitte](bild:kachel-editor "Das Kachelsatz-Studio: die Kacheln des Satzes mit ihren Zellen")

### Eigenschaften und Füllstand

Rechts unter **Eigenschaften** vergibst du frei benannte Merkmale, zum Beispiel „solide“ oder „Leiter“ (höchstens acht je Satz). Pro Kachel setzt du ein Häkchen. Ein Spiel kann später abfragen, ob man auf einer Kachel stehen darf. Der Export schreibt sie als Tabelle `<name>_props` und als Konstanten `<name>_p_<eigenschaft>`.

Alle benutzten Zeichen aller Kacheln ergeben beim Export **einen** gemeinsamen **Zielzeichensatz**. Gleiche Bilder zählen nur einmal. Der Balken **Zielzeichensatz** zeigt, wie voll er ist (zum Beispiel 187 von 256) und meldet einen Überlauf.

![Screen-Bank-Studio: links die Liste der Screens mit Vorschau, in der Mitte der gewählte Screen im Kachelmodus, rechts die Karten Werkzeug (Kachelpalette) und Dokument](bild:kachel-bank "Die Screen-Bank: Screens links, der gewählte Screen mit Kachelgrenzen in der Mitte, Kachelsatz und Farben rechts")

## Die Karte: der Screen-Editor für Kacheln

Öffnest du eine Screen-Bank (zum Beispiel `graphics/level.vcscreen` aus `gfx_demo`), erscheint der Karten-Editor automatisch.

- **Reiter** oben: ein Reiter pro Screen. Die Nummer ist die Auswahlnummer, mit der dein Programm den Screen wählt. **Neu**, **Duplizieren**, **Löschen** und Pfeile zum Umsortieren stehen daneben.
- **Kachelmodus:** Links liegt die **Palette** des Kachelsatzes. Ein Klick wählt den Pinsel; mit **Umschalt-Klick** wählst du einen ganzen Block, der dann als **Stempel** dient. Werkzeuge: Stift, Linie, Rechteck (gefüllt oder als Rahmen), Füllen, Pipette (auch rechte Maustaste), Auswahl, Stempel, Löschen (setzt Kachel 0).
- **Zeichenmodus:** Mit der Taste **Tab** (oder dem Schalter **Kacheln/Zeichen**) wechselst du auf die **Zellebene**: Hier setzt du einzelne Zeichen **über** die Kacheln (höchstens 255 solche Ausnahmen je Screen). Das Werkzeug **Löschen** lässt die Kachel darunter wieder durchscheinen.
- Rechts stellst du **Größe** und **Ursprung** des Kachelrasters ein. Was nicht aufgeht, zum Beispiel die 25. Zeile beim C64 mit 2×2-Kacheln, bleibt **Restzelle** (schraffiert, nur der Zellebene vorbehalten).
- **Ansicht:** Schalter für **Kachelgrenzen** und **Zellgitter**; die **Eigenschaften-Ebene** blendet eine gewählte Eigenschaft farbig ein, damit du siehst, wo „solide“ Kacheln stehen; die **Vorschau** zeigt den Screen mit Rahmen und den echten Systemfarben.
- **Kachelsatz:** Unter **Kachelsatz** wählst du, welcher Satz zur Bank gehört.

![Screen-Bank-Studio im Kachelmodus: die Karte des Spielfelds mit Kachelgrenzen und die Kachelpalette rechts](bild:kachel-karte "Die Karte eines Screens im Kachelmodus: Kachel wählen, auf das Spielfeld setzen")

### Aus einem Textbildschirm wird eine Karte

Hast du einen Textbildschirm gemalt, der sich wiederholende Blöcke enthält, wandelt **Zerlegen …** (Kopfzeile des Bildschirm-Editors, Dialog „In Kacheln zerlegen“) ihn um. Du wählst die Kachelgröße (2×2, 3×3 oder 4×4), die Vorschau nennt das Ergebnis vorher. **Zerlegen** legt `<name>_tiles.vctiles` und `<name>_bank.vcscreen` neben den alten Screen; der alte bleibt bestehen.

## Export

Wie überall: Kopfzeile ▸ **Exportieren …** oder automatisch beim Bauen. Ergebnis:

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

## Hardware-Besonderheiten und Grenzen

| System | Bildschirm | Zeichen / Farbe | Besonderheit |
|---|---|---|---|
| C64 | 40×25 Zellen, 1000 Bytes + 1000 Farben | 256 Zeichen, 16 Farben | Farb-RAM `$D800`; Bildschirmspeicher und Zeichensatz müssen in derselben 16-KB-Bank des VIC liegen |
| VC-20 | 22×23 | 256 Zeichen | wenig Speicher; der Speicherausbau bestimmt die Adressen |
| Plus/4 | 40×25 | 256 Zeichen, 121 Farben (TED) | TED-Palette im Studio noch nicht vollständig (Hinweis „TED-Palette fehlt“) |
| Atari (ANTIC 2) | 40×24 | 128 Zeichen | Inversdarstellung über Bit 7 |
| MEGA65 | Zeilen und Spalten je nach Profil | wie C64, zusätzlich Vollfarbzeichen | 256-Farben-Palette |

Grenzen der Kacheln: höchstens **16** Zeichen je Kachelseite, höchstens **255** Zellausnahmen je Screen, Kachelsatz bis zu 256 Zeichenbildern im Zielzeichensatz. Reichen 256 Zeichen nicht, gleichst du Zeichen zusammen oder nutzt weniger verschiedene Bausteine.

## Vom Studio ins Programm

- **Assembler:** `.include "build/export/<name>.asm"` hinter dem Hauptprogramm, dazu `lib/gfx.inc` (`gfx_load_screen`, `gfx_draw_map` …), siehe die Programme in diesem Kapitel.
- **BASIC:** Export im Format **BASIC DATA**, mit `#include` einbinden und mit `READ`/`POKE` in den Bildschirmspeicher kopieren.
- **Rohdaten:** Format **Binär** für `.incbin` oder zum Laden von Diskette.
- **Atari und MEGA65:** gleiche Dateien, andere Zieladressen; für den Atari zeigt `examples/atari/gfx_demo` den Weg.

## Häufige Stolpersteine

- **Karte zeigt Unsinn:** Der Zielzeichensatz wurde nicht geladen oder liegt nicht auf einer 2-KB-Grenze. Er muss mit `gfx_charset_at` angemeldet sein.
- **Die Karte wird nicht gezeichnet:** `tiles_use` fehlt oder nennt falsche Marken, oder `tiles_runtime` fehlt hinter dem Programm.
- **„Überlauf“ im Zielzeichensatz:** Du benutzt mehr als 256 verschiedene Zeichenbilder. Lösche Kacheln oder gleiche Zeichen zusammen (**Zusammenführen** bei gleichen Kacheln).
- **Der Kachel-Editor sagt „Der Satz hat keinen Zeichensatz“:** Rechts erst einen Zeichensatz hinzufügen.
- **Die letzte Zeile bleibt leer:** Die 25. Zeile ist beim C64 mit 2×2-Kacheln eine Restzelle; du malst sie im Zeichenmodus.

## Nachbar-Studios und Glossar

- Zeichen malst du im [Zeichensatz-Editor](kapitel:zeichensatz), Farben wählst du aus der [Palette](kapitel:paletten), bewegliche Figuren sind [Sprites](kapitel:sprites).
- Begriffe: Glossar (Datei `GLOSSAR.md`): Kachel, Karte, Bank, Zelle, Farb-RAM.

## Wie geht es weiter?

Wenn du eigene Farben bestimmen willst (MEGA65), geht es im nächsten Kapitel um [Paletten](kapitel:paletten). Nachschlagen: [Bildschirm-Editor](hilfe:screen), [Kachel-Editor](hilfe:tiles), [Export](hilfe:export).
