---
id: paletten
titel: Paletten
reihenfolge: 15
stufe: Einsteiger
zeit: 15 Minuten
siehe_auch: [zeichensatz, sprites]
referenz: [mega65, export, graphics]
---

# Paletten

**Ziel:** Du verstehst, was eine Palette ist, bearbeitest die 256 Farben des MEGA65 im Paletten-Editor und lädst sie in einem Programm. Außerdem weißt du, wie die Farben beim C64, Atari und den anderen Systemen zustande kommen.

## Das brauchst du

- für den Paletten-Editor ein **MEGA65-Projekt** (nur dieses System hat eine frei veränderbare Palette),
- Kenntnisse aus [Grafik und Klang im Überblick](kapitel:grafik-ueberblick).

## Wofür ist der Editor gedacht?

Der Paletten-Editor legt fest, **welche Farben deine Grafik wirklich hat**. Beim MEGA65 sind das 256 Farben, die du selbst mischst; bei den anderen Systemen sind die Farben fest, und du wählst sie in den anderen Editoren aus.

Typische Einsätze aus echten Retro-Projekten:

- **Ein eigener Farbverlauf** für einen Himmel oder Wasser (Vollfarbzeichen und Bitmaps des MEGA65).
- **Ein Farbschema** für ein ganzes Spiel, damit alle Figuren zusammenpassen.
- **Farbwechsel-Effekte:** dieselbe Grafik mit anderer Palette zeigt Tag und Nacht.

Im fertigen Programm sind das drei Tabellen zu je 256 Bytes (Rot, Grün, Blau), die dein Programm in die Palettenregister des Chips kopiert.

## Grundbegriffe ohne Vorwissen

- **Palette:** Tabelle „Nummer → Farbe“.
- **Eintrag:** eine Zeile der Tabelle (eine Nummer mit ihrer Farbe).
- **RGB:** eine Farbe aus Rot, Grün und Blau; beim MEGA65 je 8 Bit (0 bis 255).
- **Hexwert:** die Farbe als `#rrggbb`, zum Beispiel `#ff8800` für Orange.
- **Palettenbank:** der MEGA65 hat vier Paletten, zwischen denen du umschalten kannst.
- **PAL / NTSC:** europäische und amerikanische Videonorm; Atari-Farben sehen je nach Norm anders aus.
- **VIC-IV:** der Grafikchip des MEGA65.

## Was ist eine Palette?

Ein Bildpunkt speichert keine Farbe, sondern eine **Nummer**. Die **Palette** ist die Tabelle, die jeder Nummer eine Farbe zuordnet. Ändert man den Tabelleneintrag, ändert sich die Farbe überall, wo diese Nummer vorkommt.

- **C64, VC-20, Plus/4:** Die Palette ist fest verdrahtet (beim C64 16 Farben). Du wählst nur Nummern. Die Farbtöne, die du in den Editoren siehst, stammen aus Dateien unter `data/palettes`.
- **Atari:** Der Chip kennt 128 Farben (16 Farbtöne mal 8 Helligkeiten). Auch sie sind fest. Ob die Farben nach PAL oder NTSC aussehen, stellst du in jedem Atari-Editor rechts bei der **Palettenansicht** ein.
- **MEGA65:** Der Grafikchip VIC-IV hat eine Palette mit **256 Einträgen**, jeder aus Rot, Grün und Blau mit je **8 Bit** (also 16,7 Millionen mögliche Farben). Diese Einträge darfst du selbst ändern. Wichtig wird das im Vollfarbmodus: Dort zeigt jeder Punkt eines Zeichens auf einen Eintrag der Palette (siehe [Zeichensatz-Editor](kapitel:zeichensatz)).

## Meine erste Palette (MEGA65)

Eine neue Palettendatei legst du in dieser Version nicht in der Oberfläche an. Zum Ausprobieren nimmst du die Palette des Beispiels `examples/mega65/pal_demo`.

1. **Beispiel öffnen:** Öffne auf der Startseite aus den Beispielprojekten `pal_demo` (zum Ändern nimmst du **Kopieren**, dann hast du eine eigene Fassung). *Erwartet:* Das Projekt ist offen; im Projektbaum liegt `graphics/pal65.vcpal`.
2. **Palette öffnen:** Wähle oben den Hauptbereich **Grafik** und in der zweiten Reihe **Palette**. Ist noch nichts offen, zeigt der Editor die **Palettendateien** des Projekts als Knöpfe; klicke auf `graphics/pal65.vcpal`. Ein Doppelklick auf die Datei im Projektbaum öffnet sie ebenfalls im Palette-Editor. *Erwartet:* links ein Raster mit 256 Farbfeldern, rechts die Karte **Farbe 0**.
3. **Eintrag wählen:** Klick auf ein Feld (16 Felder je Zeile) oder Pfeiltasten. *Erwartet:* Rechts erscheint die Karte **Farbe n** mit dem Farbfeld, die Fußzeile nennt Nummer und Hexwert, zum Beispiel „Farbe 3 ($03) von 256“.
4. **Farbe ändern:** Ziehe die Regler **Rot**, **Grün** und **Blau** (je 0 bis 255), oder gib im Feld **Hexwert** eine Farbe als `#rrggbb` ein und bestätige mit der Eingabetaste. *Erwartet:* Das Feld im Raster zeigt die neue Farbe, in der Fußzeile steht „● ungespeichert“.
5. **Zurück:** **Cmd/Strg+Z**; **Cmd/Strg+Umschalt+Z** holt den Schritt zurück. **Standardpalette** (Knopf über dem Raster) setzt alle Einträge auf die mitgelieferte Palette.
6. **Speichern:** **Cmd/Strg+S** schreibt die `.vcpal`-Datei.

## Aufbau der Oberfläche

Der Palette-Editor hat zwei Bereiche nebeneinander:

| Bereich | Inhalt |
|---|---|
| Links | die Leiste mit Rückgängig, Wiederholen, dem Namen der Palette und „n Farben“, dazu **Standardpalette**, **Exportieren …** und **Speichern**; darunter das **Farbraster** mit 16 Feldern je Zeile und, wenn das Projekt mehrere Paletten hat, die Liste der Palettendateien |
| Rechts | die Karte **Farbe n** mit dem Farbfeld, den Reglern **Rot**, **Grün**, **Blau** und dem **Hexwert**, darunter **Im Einsatz** und **Ausgabe**; ganz unten die Fußzeile mit Name, Nummer und Hexwert der gewählten Farbe |

Im Raster bewegst du dich mit den Pfeiltasten (Bild auf und Bild ab springen vier Zeilen, Pos1 und Ende zum ersten und letzten Eintrag). Das Feld **Hexwert** nimmt genau sechs Hexziffern an (mit oder ohne `#`); sonst erscheint der Hinweis „Das ist kein Farbwert; erlaubt ist #rrggbb.“ und nichts wird übernommen.

![Der Paletten-Editor im Bereich „Grafik“ für die MEGA65-Palette pal65 mit 256 Farben. In der Mitte ein Raster aus 16 mal 16 Farbfeldern, das erste Feld ist gewählt. Rechts die Karten „FARBE 0“ mit Vorschau, Reglern „ROT“, „GRÜN“, „BLAU“, dem Feld „HEXWERT“ mit #000000, „IM EINSATZ“ und „AUSGABE“.](bild:palette-mega65 "Der Paletten-Editor des MEGA65 mit 256 Farbfeldern und der Karte für den gewählten Eintrag.")

**Im Einsatz** listet die Dateien deines Projekts, die den gewählten Farbindex verwenden (Sprite, Player/Missile, Zeichensatz, Kacheln oder Bildschirm); ein Klick auf den Dateinamen öffnet die Datei. Wird die Farbe nirgends benutzt, steht dort „Diese Farbe wird in keiner Mediendatei des Projekts verwendet.“ Unter **Ausgabe** steht, wie groß die Daten als Assembler-Text sind, mit einem zweiten Knopf **Exportieren …**.

Tasten: **Cmd/Strg+Z** und **Cmd/Strg+Umschalt+Z**, **Cmd/Strg+S**, **Cmd/Strg+Umschalt+E** (Export). Die Befehlspalette (**Cmd/Strg+K**) bietet dieselben Befehle an.

> **Info:** Die Palette der ersten 16 Einträge entspricht den C64-Farben. Die weiteren Einträge bilden eine Farbtabelle, mit der du Verläufe und eigene Töne bauen kannst. Bevor du alles überschreibst, lege dir eine Kopie an.

## Export und Einbinden

**Exportieren …** öffnet den Export-Dialog. Dort wählst du das **Format** (**Assembler**, **Konstanten**, **Binär**, **PRG** oder **BASIC DATA**), den **Namen** und je nach Format eine **Adresse**; die **Vorschau** zeigt Text und Bytes. **In Projekt speichern** schreibt die Palette nach `build/export/<name>.asm`; **Als Include einfügen** setzt zusätzlich die `.include`-Zeile in den unter **Ziel im Quelltext** gewählten Quelltext. Zum Einbinden brauchst du den Export nicht: `.include "graphics/<name>.vcpal"` wandelt die Palette beim Bauen von selbst um, wie bei den anderen Editoren.

Die Datei enthält drei Tabellen zu je 256 Bytes: `<name>_red`, `<name>_green` und `<name>_blue`. Der Export kümmert sich dabei um eine Eigenheit der Hardware: Die Palettenregister `$D100`, `$D200` und `$D300` erwarten die beiden Halbbytes (vier Bit) jedes Werts **vertauscht**. Das musst du nicht selbst umrechnen.

### Beispiel: eigene Palette laden

Das Projekt `examples/mega65/pal_demo` lädt `graphics/pal65.vcpal` in die Palettenbank 1 des VIC-IV:

```asm
.target mega65
.cpu 45gs02 { illegals = deny }

.org $2000
.include "lib/vic4.inc"

start:  sei
        vic4_unlock
        vic4_load_palette pal65_red, pal65_green, pal65_blue, 1
        vic4_use_palette 1
        lda #$40
        sta $d070                   ; Bank 1 bei $D100-$D3FF eingeblendet lassen (zum Zurücklesen)
done:   bra done

vic4_palette_runtime
.include "graphics/pal65.vcpal"
```

- `vic4_unlock` schaltet die Zusatzregister des VIC-IV frei. Ohne diese Zeile ignoriert der Chip alle weiteren Befehle.
- `vic4_load_palette` kopiert die 256 Einträge aus den drei Tabellen in **Palettenbank 1**. Der MEGA65 hat vier Bänke; die vierte Zahl wählt eine.
- `vic4_use_palette 1` lässt Text und Bitmaps diese Bank benutzen.
- `vic4_palette_runtime` legt das Unterprogramm zum Laden ab, **hinter** dem Programm.

1. Öffne `examples/mega65/pal_demo`, baue mit **F7**, starte mit **F5** (Xemu).
2. Das Programm zeigt nichts Eigenes an, es lädt nur die Palette.
3. Ändere im Palette-Editor einen Eintrag, speichere, baue und starte neu. Im Export-Dialog (Vorschau, Reiter **Hex**) siehst du die Bytes der Tabellen. Denk daran: Sie enthalten die Halbbytes vertauscht (aus `$8A` wird `$A8`).

> **Achtung:** Eine geladene Palette färbt nur das, was ihre Einträge benutzt: Vollfarbzeichen (Kapitel [Zeichensatz-Editor](kapitel:zeichensatz)) und Sprites oder Bildschirmfarben, die auf diese Palettenbank zeigen. Ohne solche Anwender ändert sich auf dem Bildschirm nichts.

## Atari und die anderen: Farben im Editor wählen

Bei den anderen Systemen gibt es **keine** frei veränderbare Palette, denn die Farben sind Hardware. Der Sprite- und der Zeichensatz-Editor zeigen rechts in der Karte **Farben** das Farbraster des Systems: beim C64 16 Felder, beim Atari 128 (mit der **Palettenansicht** **Projekt**, **PAL** oder **NTSC**). Du wählst erst einen Farbwert (zum Beispiel die Sprite-Farbe), dann klickst du ein Feld an und weist es dem Wert zu. Gespeichert wird die Nummer, nicht der Farbton.

*[Bild folgt in E13: palette-atari]*

Wie die Atari-Farben nach PAL oder NTSC wirklich aussehen, zeigen die kleinen Messprogramme `examples/atari/palette0` und `examples/atari/palette1`: Sie malen jeden Farbwert als Streifen auf den Bildschirm.

## Hardware-Besonderheiten und Grenzen

| System | Farben | Veränderbar? |
|---|---|---|
| C64, C128 (40 Spalten) | 16 | nein (Emulatoren zeigen sie unterschiedlich) |
| VC-20 | 16 | nein |
| Plus/4 | 121 | nein (TED-Palette im Editor noch unvollständig) |
| Atari | 128 (16 Farbtöne × 8 Helligkeiten) | nein; PAL und NTSC sehen anders aus |
| MEGA65 | 256 aus 16,7 Millionen | **ja**, 4 Palettenbänke |

Auf dem MEGA65 wirkt eine Palette nur auf Vollfarbzeichen, Bitmaps und Sprites, die diese Bank benutzen; den Vorgaben „C64-Farben“ der ersten 16 Einträge solltest du treu bleiben, wenn du die Standardgrafik weiter benutzen willst.

## Nachbar-Editoren und Glossar

- Die Farben benutzen der [Zeichensatz-Editor](kapitel:zeichensatz) (Vollfarbzeichen), der [Sprite-Editor](kapitel:sprites) und der [Bildschirm-Editor](kapitel:bildschirm-kacheln).
- Begriffe: Glossar (Datei `GLOSSAR.md`): Palette, RGB, PAL, NTSC.

## Häufige Stolpersteine

- **Die Farben im Programm weichen vom Editor ab:** Beim Atari kommt es auf die Videonorm an, beim C64 auf den Emulator (VICE zeigt die Farben nach seiner eigenen Palette).
- **Die Palette hat keine Wirkung:** Auf dem MEGA65 wirkt sie nur, wenn `vic4_use_palette` die geladene Bank wählt und etwas ihre Einträge benutzt.
- **Ein Hexwert wird nicht übernommen:** Das Feld **Hexwert** akzeptiert genau sechs Hexziffern (`#rrggbb`); bei allem anderen erscheint der Hinweis „Das ist kein Farbwert; erlaubt ist #rrggbb.“

## Wie geht es weiter?

Jetzt kommt der Klang: [Der Tracker](kapitel:tracker). Nachschlagen: [MEGA65](hilfe:mega65), [Export](hilfe:export).
