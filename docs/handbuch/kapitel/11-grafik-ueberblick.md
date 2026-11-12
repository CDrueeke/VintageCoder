---
id: grafik-ueberblick
titel: Grafik und Klang im Überblick
reihenfolge: 11
stufe: Einsteiger
zeit: 15 Minuten
siehe_auch: [zeichensatz, sprites, bildschirm-kacheln, paletten, tracker]
referenz: [graphics, export]
---

# Grafik und Klang im Überblick

**Ziel:** Du weißt, welche Zeichen-Werkzeuge („Studios“) VintageCoder hat, wie du eine neue Grafik- oder Klangdatei anlegst und wie ihr Inhalt vom Editor in dein Programm kommt. Mit diesem Kapitel verstehst du den Weg, den alle folgenden Kapitel (12 bis 17) jeweils im Detail gehen.

## Das brauchst du

- ein geöffnetes Projekt (siehe [Dein erstes Projekt](kapitel:erstes-projekt)),
- für das letzte Beispiel den Emulator des Zielsystems.

## Was sind Assets?

Alles, was nicht Programmtext ist, nennen wir **Assets** (englisch für „Bestände“): ein Sprite, ein Zeichensatz, ein Bildschirm, eine Palette, ein Song. Jedes Asset liegt als **kleine Textdatei** in deinem Projekt, zum Beispiel `graphics/hero.vcsprite`. Der Inhalt sind Hex-Zeilen, die Pixel beschreiben. Das hat zwei Vorteile: Du kannst die Dateien mit Versionsverwaltung wie Git sinnvoll vergleichen, und der Editor und der Bau lesen dieselbe Quelle.

| Endung | Art | Studio | Kapitel |
|---|---|---|---|
| `.vcchar` | Zeichensatz | Zeichensatz-Editor | [12](kapitel:zeichensatz) |
| `.vcsprite` | Sprites (C64, MEGA65) | Sprite-Editor | [13](kapitel:sprites) |
| `.vcpmg` | Player/Missiles (Atari) | PMG-Editor | [13](kapitel:sprites) |
| `.vcscreen` | Textbildschirm oder Screen-Bank | Bildschirm-Editor | [14](kapitel:bildschirm-kacheln) |
| `.vctiles` | Kachelsatz | Kachel-Editor | [14](kapitel:bildschirm-kacheln) |
| `.vcpal` | Palette (MEGA65) | Paletten-Editor | [15](kapitel:paletten) |
| `.vcsong` | Song (SID, POKEY und andere) | Tracker | [16](kapitel:tracker) |
| `.vcplay` | PLAY-Song (MEGA65) | PLAY-Tracker | [17](kapitel:play-tracker) |

> **Info:** Jedes Asset gehört zu einem **Profil**. Das Profil legt fest, wie das Asset aufgebaut ist, etwa `c64.sprite.multicolor` (C64-Sprite, mehrfarbig, 12×21 Punkte) oder `atari.char.antic2` (Atari-Zeichensatz mit 128 Zeichen). Das Profil wählst du beim Anlegen.

## Schritt für Schritt: ein Asset anlegen und öffnen

1. Klicke im Projektbaum (linke Seitenleiste) auf das **+** in der Kopfzeile. Das Menü bietet neben „Quelltext …“ und „Ordner …“ die Einträge **Sprite …**, **Zeichensatz …**, **Bildschirm …**, **Kacheln …**, **Palette …**, **Song …** und **PLAY …**. Dasselbe erreichst du über das Menü **Datei ▸ Neu: Sprite, Zeichensatz, Song …**.
2. Im Dialog **Neu anlegen** gibst du einen **Namen** ein und wählst unter **Profil** die passende Variante. Mit **Anlegen** entsteht die Datei.
3. Die neue Datei öffnet sich im passenden Studio. Ein **Doppelklick** auf eine vorhandene Datei im Projektbaum öffnet sie genauso.

## Der gemeinsame Aufbau der Studios

Sprite-, Zeichensatz-, Bildschirm- und Kachel-Editor sehen sich bewusst ähnlich. Wenn du einen kennst, findest du dich in den anderen zurecht:

1. **Kopfzeile:** Dateiname (ein Punkt dahinter bedeutet „ungespeicherte Änderungen“) und farbige Chips für Zielsystem und Profil.
2. **Werkzeugleiste:** Speichern, Undo und Redo, Zeichenwerkzeuge, Importe, Zoom.
3. **Links** die Liste (Frames, Zeichen oder Kacheln), **in der Mitte** die Zeichenfläche, **rechts** Pixelwerte, Farben und Vorschau.
4. **Fuß:** der **Export-Streifen** mit Format, Ausgabedatei und dem Knopf **Export …**.

Die Studios öffnen als Reiter im Editorbereich. Wenn du mehr Platz oder einen zweiten Bildschirm hast, kannst du ein Studio in ein **eigenes Fenster lösen** (Rechtsklick auf den Reiter ▸ „In eigenem Fenster lösen“) und es mit **Fenster ▸ Gelöste Studios zurückholen** wieder einbetten.

Die Zeichenfläche passt ihren Zoom automatisch an den freien Platz an. Willst du selbst bestimmen, schaltest du in der Werkzeugleiste **Anpassen** aus und stellst den Zoom mit dem Regler oder mit **Strg/Cmd + Mausrad** ein.

Sinnvolle Grundregeln für alle Studios:

- **Strg/Cmd+S** speichert, **Strg/Cmd+Z** macht rückgängig, **Strg/Cmd+Umschalt+Z** wiederholt.
- Mit der **rechten Maustaste** radierst du auf der Zeichenfläche (Sprite-, Zeichensatz-Editor).
- Die Farben richten sich nach dem Zielsystem. Beim Atari wählst du rechts außerdem die **Videonorm** (PAL oder NTSC).

## Vom Editor ins Programm: der Export

Dein Programm kann die Textdatei nicht lesen, es braucht **Bytes**. Der **Export** wandelt ein Asset um, zum Beispiel in eine Assembler-Datei mit einer Marke und `.byte`-Zeilen. Das Ergebnis landet in `build/export/`.

Es gibt zwei Wege:

- **Automatisch:** Beim **Bauen** (F7) schreibt VintageCoder veraltete Assets selbst nach `build/export/<name>.asm`. Du musst nichts tun, außer die Datei im Quelltext einzubinden.
- **Von Hand:** Der Knopf **Export …** im Fuß des Studios öffnet den Export-Dialog.

Der Fuß zeigt neben dem Namen der Ausgabedatei einen Chip: **aktuell** (der Export ist neuer als dein Asset) oder **veraltet** (du hast seither geändert).

### Der Export-Dialog

Das **Ausgabeformat** wählst du oben:

| Format | Datei | Wofür |
|---|---|---|
| Assembler | `<name>.asm` | Daten als `.byte`-Zeilen mit Marken und Konstanten; der Normalfall |
| Konstanten | `<name>.inc` | nur die Größen (`<name>_len`) und optional die Adresse |
| Binär | `<name>.bin` | Rohdaten, zum Beispiel zum Nachladen |
| PRG | `<name>.prg` | Rohdaten mit Ladeadresse (die gibst du im Feld **Adresse** an) |
| BASIC DATA | `<name>.data.bas` | `DATA`-Zeilen für BASIC-Programme |

Im Feld **Name** steht die Marke, die als Vorsilbe aller Namen im Export dient. Die **Vorschau** zeigt den aktuellen Stand, auch ungespeicherte Änderungen, ohne etwas im Projekt anzulegen. Unten stehen drei Aktionen:

- **Im Emulator testen:** speichert, exportiert, baut dein Projekt und startet es wie F5.
- **In Projekt speichern:** schreibt die Dateien nach `build/export/`.
- **In aktiven Code einfügen:** speichert, exportiert als Assembler und setzt eine `.include`-Zeile in den zuletzt benutzten Assembler-Quelltext. Das gelingt auch, wenn du die Datei aus dem Projektbaum in den Editor ziehst.

![Export-Dialog mit Formatwahl, Vorschau und den drei Aktionen](bild:grafik-exportieren "Der Export-Dialog: ① Format, ② Name und Adresse, ③ Vorschau, ④ Aktionen")

> **Tipp:** Brauchst du nur die Daten im Programm, lass den automatischen Export arbeiten. Der Dialog lohnt sich, wenn du ein anderes Format brauchst oder die Bytes einmal ansehen möchtest.

## Beispiel zum Nachmachen: ein Sprite auf den Bildschirm

Das Projekt `examples/c64/sprite_demo` enthält ein mehrfarbiges Sprite `graphics/hero.vcsprite` und dieses Programm:

```asm
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

start:  sei
        lda #$80                ; Sprite-Zeiger 0: Daten bei $2000 ($2000 / 64 = $80)
        sta $07f8
        lda #1
        sta vic2.spena          ; Sprite 0 an
        sta vic2.spmc           ; Multicolor
        lda #100
        sta vic2.sp0x
        sta vic2.sp0y
        lda #7
        sta vic2.sp0col
        lda #2
        sta vic2.spmc0
        lda #6
        sta vic2.spmc1
loop:   jmp loop

.org $2000
.include "build/export/hero.asm"
```

1. Öffne das Projekt `examples/c64/sprite_demo`.
2. Drücke **F7** (Projekt bauen). Dabei entsteht `build/export/hero.asm` automatisch, weil das Sprite „veraltet“ war.
3. Drücke **F5** (Startprogramm starten). Im Emulator erscheint der kleine Held.

Die letzten beiden Zeilen sind der ganze „Einbau“: `.org $2000` legt die Adresse fest, `.include` holt die exportierten Bytes dorthin. Wie das Sprite entsteht, lernst du in [Kapitel 13](kapitel:sprites).

## Häufige Stolpersteine

- **„Label nicht gefunden“ beim Bauen:** Der Name nach dem Export (`hero`, `font`, …) richtet sich nach dem Namen im Asset, nicht nach dem Dateinamen. Im Export-Dialog siehst du den Namen im Feld **Name**.
- **Änderungen erscheinen nicht im Programm:** Zeigt der Fuß **veraltet**, wurde nicht neu exportiert. Bauen (F7) oder **Export …** nutzen. Vorher mit Strg/Cmd+S speichern.
- **Ein Studio öffnet nicht:** Das Asset braucht ein geöffnetes Projekt. Ohne Projekt gibt es keinen Projektbaum.

## Wie geht es weiter?

Als Nächstes lernst du das einfachste Studio kennen: den [Zeichensatz-Editor](kapitel:zeichensatz). Ausführliche Nachschlagetexte: [Studios im Überblick](hilfe:graphics) und [Export](hilfe:export).
