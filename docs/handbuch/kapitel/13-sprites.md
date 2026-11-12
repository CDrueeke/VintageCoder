---
id: sprites
titel: Sprite- und PMG-Editor
reihenfolge: 13
stufe: Einsteiger
zeit: 30 Minuten
siehe_auch: [grafik-ueberblick, zeichensatz, paletten]
referenz: [sprite, export, graphics]
---

# Sprite- und PMG-Editor

**Ziel:** Du zeichnest ein Sprite (C64) oder einen Player (Atari), lässt es sich bewegen, exportierst es und zeigst es in einem Programm an.

## Das brauchst du

- ein C64- oder Atari-Projekt (für MEGA65 gelten die C64-Sprites),
- Grundkenntnisse aus [Grafik und Klang im Überblick](kapitel:grafik-ueberblick): Asset anlegen, exportieren, einbinden.

## Was ist ein Sprite?

Ein **Sprite** ist ein kleines Bild, das der Grafikchip **selbst über den Bildschirm legt**. Du musst es nicht ins Bild hineinmalen, sondern gibst nur an, wo es stehen soll. Dadurch lassen sich Figuren schnell und ruckelfrei bewegen, ohne den Hintergrund zu zerstören.

- **C64 (VIC-II):** acht Sprites, jedes **24×21 Punkte** groß (63 Bytes, im Speicher auf 64 Bytes aufgefüllt). Im **Hires**-Modus hat ein Sprite eine Farbe. Im **Multicolor**-Modus sind die Punkte doppelt so breit (also 12×21), dafür gibt es vier Werte: durchsichtig, die eigene **Sprite-Farbe** und zwei Farben, die sich **alle** Sprites teilen (**Multicolor 1** und **Multicolor 2**).
- **Atari (GTIA):** Der Atari nennt sie **Player/Missiles** („PMG“, Player-Missile-Grafik). Es gibt vier **Player**, jeder **8 Punkte breit** und so hoch wie der Bildschirm. Jeder Player hat eine eigene Farbe. Mehrere Player kannst du nebeneinander stellen, um größere Figuren zu bauen. Dazu kommen Missiles (schmale Geschosse), die dieses Handbuch nicht weiter behandelt.

## Schritt für Schritt: ein C64-Sprite zeichnen

1. **+** im Projektbaum ▸ **Sprite …**. Name `hero`, Profil **C64 Sprite multicolor 12×21**, **Anlegen**.
2. Der Editor öffnet. **Links** steht die Liste der **Frames** (Einzelbilder), zunächst eins. **In der Mitte** ist die Zeichenfläche mit breiten Punkten. **Rechts** stehen die **Pixelwerte**, die Farben und die Vorschau.
3. Rechts wählst du einen Pixelwert (Klick oder Taste **1** bis **4**): *transparent*, *Multicolor 1*, *Sprite-Farbe* oder *Multicolor 2*. Unter der Tabelle gibst du jeder Farbe einen Palettenwert, indem du auf ein Farbfeld klickst.
4. Mal mit der linken Maustaste. Die rechte Maustaste löscht (setzt auf *transparent*).
5. Die Werkzeuge der Werkzeugleiste: **Stift**, **Radierer**, **Füllen**, **Linie**, **Rahmen**, **Auswahl**, **Pipette**. Dazu **Spiegeln H/V**, **Links/Rechts/Hoch/Runter** (schiebt das Bild) und **Leeren**.
6. **Strg/Cmd+S** speichert.

![Sprite-Editor mit Multicolor-Held: ① Frame-Liste, ② Zeichenfläche, ③ Pixelwerte, ④ Farben, ⑤ Animationsfeld](bild:sprite-c64 "Der Sprite-Editor: Frames links, Zeichenfläche in der Mitte, Pixelwerte und Farben rechts")

### Hires oder Multicolor – je Frame

Du kannst Frames eines Sprites mischen: Rechts unter **Modus dieses Frames** wählst du **Hires** oder **Multicolor**. Beim Umschalten rechnet der Editor die Punkte um (bei Hires → Multicolor werden zwei Punkte zu einem). Die Taste **C** wechselt ebenfalls. **Strg/Cmd+Z** macht den Wechsel rückgängig. Die zwei gemeinsamen Farben **Multicolor 1** und **2** gelten für alle Frames der Datei.

![Multicolor-Sprite mit den vier Pixelwerten und ihren Registern](bild:sprite-c64-multicolor "Multicolor: vier Werte, davon teilen sich alle Sprites zwei Farben ($D025 und $D026)")

### Weitere Sprite-Eigenschaften (C64)

Rechts erscheinen unter **VIC-II Sprite-Flags** vier Schalter:

| Schalter | Bedeutung | Register |
|---|---|---|
| MC | Multicolor (folgt dem Modus des Frames) | `$D01C` |
| X2 | doppelt breit | `$D01D` |
| Y2 | doppelt hoch | `$D017` |
| BG | hinter dem Hintergrund | `$D01B` |

X2, Y2 und BG gelten für die ganze Datei. Sie landen im Export als Konstanten (`<name>_expand_x`, `<name>_expand_y`, `<name>_behind`).

## Schritt für Schritt: ein Atari-Player

1. **+** ▸ **Sprite …**, Profil **Atari PMG Einzelzeile (8 Pixel breit)**. Die Datei heißt `.vcpmg`.
2. In der linken Liste stehen die **vier Player P0 bis P3**. Jeder ist 8 Punkte breit und standardmäßig 32 Zeilen hoch. Die Farbe jedes Players wählst du rechts. Beim Atari gibt es dafür 128 Töne (Videonorm PAL oder NTSC umschaltbar).
3. Rechts findest du **Breite** (einfach, doppelt oder vierfach – die Register `SIZEP0` bis `SIZEP3`) und **Priorität** (Player vor Feld oder Feld vor Player).
4. Mal wie beim C64.

![Atari-PMG-Editor mit vier Spielern P0 bis P3 und ihren Farben](bild:sprite-atari-pmg "Der PMG-Editor: links die vier Player, rechts Farbe, Breite und Priorität")

> **Info:** Breite und Priorität speichert der Editor in der Datei (`hw`), schreibt sie aber **noch nicht** in den Export. Setze sie im Programm selbst (siehe das Beispiel unten).

## Frames, Animation und Onion-Skin

Eine **Animation** entsteht aus mehreren Frames, die kurz nacheinander gezeigt werden (wie ein Daumenkino).

1. Links unter **Frames** legst du mit **+** ein neues leeres Frame an, mit **Dupl.** eine Kopie des gewählten, mit **−** löschst du eines. Die Pfeiltasten links/rechts wechseln das Frame.
2. Schalte **Onion-Skin** ein (Werkzeugleiste): Das vorige Frame erscheint blass als Schatten unter deinem Bild. So triffst du die Abstände.
3. Rechts bei der Vorschau startet **▶ Animation** eine einfache Vorschau über alle Frames.
4. Für eine richtige Folge benutzt du das Feld **Animationen** unter dem Editor: **+ Neu** legt eine Animation an, **Frame anhängen** fügt das gewählte Frame als Schritt hinzu. Jeder Schritt hat eine **Dauer** (in „B“ = Bildern; 1 Bild = 1/50 Sekunde), du kannst ihn mit den Pfeilen verschieben oder mit × entfernen. Der **Ablauf** ist **Schleife**, **Hin und zurück** oder **Einmal**.

![Animationsfeld mit drei Schritten, Dauer in Bildern und Ablaufwahl](bild:sprite-animation "Das Animationsfeld: Schritte mit Dauer in Bildern, darüber Ablauf und Name")

Wenn es Animationen gibt, erzeugt der Export zusätzlich eine Datei `<name>.anim.asm` mit den Sprite-Zeigern (`_ptr`), den Dauern (`_delay`), der Länge (`_len`) und dem Modus (`_mode`). Ein Raster-Interrupt zählt die Dauer herunter und schreibt den nächsten Zeiger nach `$07F8` plus Sprite-Nummer.

### Auswahl und Rahmen

Mit dem Werkzeug **Auswahl** (Taste **S**) ziehst du einen Rahmen auf. Darin kannst du kopieren, ausschneiden, einfügen oder löschen (zusätzliche Knöpfe erscheinen in der Werkzeugleiste: **Kopieren**, **Ausschneiden**, **Einfügen**, **Auswahl löschen**). Das Werkzeug **Rahmen** (Taste **R**) zeichnet ein Rechteck als Umriss.

## Sprites aus anderen Quellen

Der Knopf **Konvertieren von …** legt aus einer Datei ein neues Sprite-Asset an. Möglich sind:

- **SPD-Dateien** (`.spd`, Versionen 1, 3 und 5),
- **Rohdaten** (`.bin`, `.spr`, `.prg`): Blöcke zu 64 Bytes; du gibst Offset, Anzahl und Modus an,
- die **Sprite-Datei eines anderen Systems** (`.vcsprite` oder `.vcpmg`), zum Beispiel ein C64-Sprite für den Atari. Dabei gehen je nach Ziel Farben oder Breite verloren; Hinweise stehen danach in der Ausgabe.

Außerdem lädt **PNG laden …** ein Bild und wandelt es in die Pixelwerte um.

## Beispiel 1: Sprite auf dem C64

Öffne `examples/c64/sprite_demo` (siehe das Programm im [Überblickskapitel](kapitel:grafik-ueberblick)). Nach F7 und F5 steht das Sprite im Emulator.

Die wichtigsten Zeilen im Programm:

```asm
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
        sta vic2.spmc0          ; Multicolor 1
        lda #6
        sta vic2.spmc1          ; Multicolor 2
```

- Der **Sprite-Zeiger** (`$07F8` für Sprite 0) sagt dem Chip, in welchem 64-Byte-Block die Daten liegen. Liegen sie bei `$2000`, ist der Zeiger `$2000 / 64 = $80`. Deshalb steht in der Datei `.org $2000` vor dem `.include`: Sprite-Daten müssen auf einer 64-Byte-Grenze beginnen.
- Die Farben im Programm (`7`, `2`, `6`) sind dieselben, die im Editor als Sprite-Farbe, Multicolor 1 und Multicolor 2 stehen. Die Registerwerte stehen nicht in den Sprite-Bytes, aber der Export liefert sie als Konstanten `hero_sprite`, `hero_mc1` und `hero_mc2`. Schreibst du `lda #hero_sprite` statt `lda #7`, folgt dein Programm automatisch jeder Farbänderung im Editor.

> **Tipp:** Zeigen mehrere Frames hintereinander im Speicher (Frame 0 bei `$2000`, Frame 1 bei `$2040` usw.), wählst du ein anderes Frame, indem du den Zeiger änderst: `$80`, `$81`, …

## Beispiel 2: Player auf dem Atari

Im Projekt `examples/atari/pmg_ship` liegen vier Player eines Raumschiffs in `graphics/ship.vcpmg`. Das Programm `pmg_ship.asm` stellt sie nebeneinander auf den Bildschirm:

```asm
.target atari8
.cpu sally { illegals = deny }

.const SDMCTL = $022f           ; Schattenregister von DMACTL (das Betriebssystem kopiert es jedes Bild)

.org $2000
start:  lda #$40
        sta antic.pmbase        ; Player/Missile-Seite = $4000 (Spieler 0 liegt bei $4400)
        lda #3
        sta gtia.gractl         ; Player und Missiles einschalten
        lda #$3a                ; Anzeigeliste an, Player-DMA an, Einzelzeilen-Auflösung, normales Feld
        sta SDMCTL
        lda #ship_p0
        sta gtia.colpm0
        lda #ship_p1
        sta gtia.colpm1
        lda #ship_p2
        sta gtia.colpm2
        lda #ship_p3
        sta gtia.colpm3
        lda #100                ; vier Player nebeneinander, je 8 Pixel breit
        sta gtia.hposp0
        lda #108
        sta gtia.hposp1
        lda #116
        sta gtia.hposp2
        lda #124
        sta gtia.hposp3
done:   jmp done

.org $4400                      ; Player 0; Player 1 bis 3 folgen im Abstand von $100
.include "build/export/ship.asm"
.run start
```

So funktioniert es:

- Der Export legt die vier Player **hintereinander im Abstand von 256 Bytes** ab (1024 Bytes). In der Einzelzeilen-Auflösung liegt Player 0 bei **PMBASE + `$400`**, also hier bei `$4400`, wenn die Basis `$4000` ist (`$40` in `PMBASE`). Das `.org $4400` bringt die Daten an die richtige Stelle.
- `GRACTL` schaltet die Player ein, `SDMCTL` (die Schattenkopie von `DMACTL`) lässt den Chip die Daten selbst aus dem Speicher holen.
- Die Konstanten `ship_p0` bis `ship_p3` kommen aus dem Export und enthalten die im Editor gewählten Farben.

1. Öffne `examples/atari/pmg_ship`, baue mit **F7**, starte mit **F5**.
2. Ändere im Editor die Farbe eines Players, baue und starte neu.

## Häufige Stolpersteine

- **Das Sprite ist verschoben oder zeigt Müll:** Die Daten liegen nicht auf einer 64-Byte-Grenze (C64) oder nicht bei `PMBASE + $400` (Atari).
- **Die Farben im Emulator weichen ab:** Die Sprite-Farben stehen in den Registern, nicht in den Daten. Setze sie im Programm, am besten mit den Export-Konstanten (`hero_sprite`, `hero_mc1`, `hero_mc2`).
- **Das Sprite erscheint nicht:** `vic2.spena` hat das Bit nicht; oder die Position liegt außerhalb des sichtbaren Bereichs (links beginnt der sichtbare Bereich etwa bei X = 24, oben bei Y = 50).
- **Multicolor wirkt zu breit:** Das ist so: Ein Multicolor-Punkt ist zwei Hires-Punkte breit.

## Wie geht es weiter?

Im nächsten Kapitel setzt du Zeichen und Kacheln zu ganzen Bildern zusammen: [Bildschirm- und Kachel-Editor](kapitel:bildschirm-kacheln). Nachschlagen: [Sprite-/PMG-Editor](hilfe:sprite), [Export](hilfe:export).
