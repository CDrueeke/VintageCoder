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

## Wofür ist das Studio gedacht?

Der Paletten-Editor legt fest, **welche Farben deine Grafik wirklich hat**. Beim MEGA65 sind das 256 Farben, die du selbst mischst; bei den anderen Systemen sind die Farben fest, und du wählst sie in den anderen Studios aus.

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
- **Atari:** Der Chip kennt 128 Farben (16 Farbtöne mal 8 Helligkeiten). Auch sie sind fest. Ob die Farben nach PAL oder NTSC aussehen, stellst du in jedem Atari-Studio rechts bei **Videonorm** ein.
- **MEGA65:** Der Grafikchip VIC-IV hat eine Palette mit **256 Einträgen**, jeder aus Rot, Grün und Blau mit je **8 Bit** (also 16,7 Millionen mögliche Farben). Diese Einträge darfst du selbst ändern. Wichtig wird das im Vollfarbmodus: Dort zeigt jeder Punkt eines Zeichens auf einen Eintrag der Palette (siehe [Zeichensatz-Editor](kapitel:zeichensatz)).

## Meine erste Palette (MEGA65)

1. **＋** in der linken Spalte ▸ im Dialog **Neue Datei** die Art **Paletten**, Profil **MEGA65 Palette (256 Farben, 8 Bit je Kanal)**, Namen `pal65` eingeben, **OK**. *Erwartet:* ein neuer Reiter mit 256 Farbfeldern.
2. **Eintrag wählen:** Klick auf ein Feld (16 Felder je Zeile). *Erwartet:* Rechts erscheint die Karte **Eintrag n ($xx)**, die Fußzeile nennt Nummer und Hexwert.
3. **Farbe ändern:** Die Regler **R**, **G** und **B** oder ein Hexwert (`#rrggbb`) im Feld **Hex**, dann Eingabetaste. *Erwartet:* Das Feld zeigt die neue Farbe, **●** erscheint.
4. **Zurück:** **Cmd/Strg+Z**; **Standardpalette** setzt alle 256 Einträge auf die mitgelieferte Palette.
5. **Speichern:** **Cmd/Strg+S** schreibt `.vcpal`.

![Das Paletten-Studio des MEGA65: 256 Farbfelder, rechts die Karte „Eintrag 0 ($00)“ mit Farbfeld, den Reglern R, G, B und dem Feld Hex; oben die Leiste mit „Standardpalette“.](bild:palette-mega65 "Der Paletten-Editor des MEGA65: 256 Farbfelder und die Karte für den gewählten Eintrag")

## Aufbau der Oberfläche

| Zone | Inhalt |
|---|---|
| Kopfzeile | Art **PAL**, Name, **●**, Zahl der Farben, **Exportieren …** (**Cmd/Strg+Umschalt+E**), **Speichern** |
| Leiste | **Standardpalette**, Rückgängig, Wiederholen |
| Liste | (beim Paletten-Editor nicht belegt) |
| Fläche | die 256 Farbfelder |
| Karten | Karte **Eintrag n ($xx)**: Farbfeld, Regler **R**, **G**, **B** und Feld **Hex** |
| Fußzeile | Nummer und Hexwert des gewählten Eintrags |

Tasten: **Cmd/Strg+Z** und **Cmd/Strg+Umschalt+Z**, **Cmd/Strg+S**. Weitere Aktionen findest du in der Befehlspalette (**Cmd/Strg+K**, „Studio: Standardpalette laden“ …).


> **Info:** Die Palette der ersten 16 Einträge entspricht den C64-Farben. Die weiteren Einträge bilden eine Farbtabelle, mit der du Verläufe und eigene Töne bauen kannst. Bevor du alles überschreibst, lege dir eine Kopie an.

## Export und Einbinden

Über **Exportieren …** (Kopfzeile) öffnest du den Export-Dialog. **In Projekt speichern** schreibt die Palette nach `build/export/<name>.asm`; **In aktiven Code einfügen** setzt zusätzlich die `.include`-Zeile in den zuletzt benutzten Quelltext. Beim Bauen entsteht der Export wie bei den anderen Studios von selbst.

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
.include "build/export/pal65.asm"
```

- `vic4_unlock` schaltet die Zusatzregister des VIC-IV frei. Ohne diese Zeile ignoriert der Chip alle weiteren Befehle.
- `vic4_load_palette` kopiert die 256 Einträge aus den drei Tabellen in **Palettenbank 1**. Der MEGA65 hat vier Bänke; die vierte Zahl wählt eine.
- `vic4_use_palette 1` lässt Text und Bitmaps diese Bank benutzen.
- `vic4_palette_runtime` legt das Unterprogramm zum Laden ab, **hinter** dem Programm.

1. Öffne `examples/mega65/pal_demo`, baue mit **F7**, starte mit **F5** (Xemu).
2. Das Programm zeigt nichts Eigenes an, es lädt nur die Palette. Weil es die Bank 1 bei `$D100`–`$D3FF` eingeblendet lässt, kannst du das Ergebnis im Debugger (Speicher-Ansicht, siehe [Der Debugger](kapitel:debugger)) nachlesen.
3. Ändere im Paletten-Editor einen Eintrag, baue und starte neu und vergleiche die Bytes. Denk daran: Die Tabellen enthalten die Halbbytes vertauscht (aus `$8A` wird `$A8`).

> **Achtung:** Eine geladene Palette färbt nur das, was ihre Einträge benutzt: Vollfarbzeichen (Kapitel 12) und Sprites oder Bildschirmfarben, die auf diese Palettenbank zeigen. Ohne solche Anwender ändert sich auf dem Bildschirm nichts.

## Atari und die anderen: Farben im Studio wählen

Bei den anderen Systemen gibt es **keinen** Paletten-Editor, denn die Farben sind Hardware. Jedes Studio zeigt rechts seine Farbfelder: beim C64 16 Felder, beim Atari 128 (mit **Videonorm** PAL oder NTSC). Du klickst ein Feld an und weist es einem Pixelwert zu. Gespeichert wird die Nummer, nicht der Farbton.

![Die Karte „Dokument“ im Atari-PMG-Editor: die gewählte Spielerfarbe, darunter die Atari-Palette (128 Farbtöne) mit den Schaltern PAL, NTSC und Projekt.](bild:palette-atari "Beim Atari wählst du eine von 128 festen Farben; die Videonorm (PAL oder NTSC) ändert, wie sie aussehen")

Wie die Atari-Farben nach PAL oder NTSC wirklich aussehen, zeigen die kleinen Messprogramme `examples/atari/palette0` und `examples/atari/palette1`: Sie malen jeden Farbwert als Streifen auf den Bildschirm.

## Hardware-Besonderheiten und Grenzen

| System | Farben | Veränderbar? |
|---|---|---|
| C64, C128 (40 Spalten) | 16 | nein (Emulatoren zeigen sie unterschiedlich) |
| VC-20 | 16 | nein |
| Plus/4 | 121 | nein (TED-Palette im Studio noch unvollständig) |
| Atari | 128 (16 Farbtöne × 8 Helligkeiten) | nein; PAL und NTSC sehen anders aus |
| MEGA65 | 256 aus 16,7 Millionen | **ja**, 4 Palettenbänke |

Auf dem MEGA65 wirkt eine Palette nur auf Vollfarbzeichen, Bitmaps und Sprites, die diese Bank benutzen; den Vorgaben „C64-Farben“ der ersten 16 Einträge solltest du treu bleiben, wenn du die Standardgrafik weiter benutzen willst.

## Nachbar-Studios und Glossar

- Die Farben benutzen der [Zeichensatz-Editor](kapitel:zeichensatz) (Vollfarbzeichen), der [Sprite-Editor](kapitel:sprites) und der [Bildschirm-Editor](kapitel:bildschirm-kacheln).
- Begriffe: Glossar (Datei `GLOSSAR.md`): Palette, RGB, PAL, NTSC.

## Häufige Stolpersteine

- **Die Farben im Programm weichen vom Editor ab:** Beim Atari kommt es auf die Videonorm an, beim C64 auf den Emulator (VICE zeigt die Farben nach seiner eigenen Palette).
- **Die Palette hat keine Wirkung:** Auf dem MEGA65 wirkt sie nur, wenn `vic4_use_palette` die geladene Bank wählt und etwas ihre Einträge benutzt.
- **Ein Eintrag lässt sich nicht tippen:** Das Textfeld akzeptiert genau sechs Hexziffern (`#rrggbb`); alles andere wird beim Verlassen verworfen.

## Wie geht es weiter?

Jetzt kommt der Klang: [Der Tracker](kapitel:tracker). Nachschlagen: [MEGA65](hilfe:mega65), [Export](hilfe:export).
