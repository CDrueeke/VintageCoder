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

## Wofür ist das Studio gedacht?

Im Sprite-Studio malst du die **beweglichen Figuren** deines Programms: den Spieler, Gegner, Schüsse, einen Mauszeiger, ein Raumschiff. Alles, was sich über einen Hintergrund bewegen soll, ohne ihn zu zerstören, ist ein Kandidat für ein Sprite.

Typische Einsätze aus echten Retro-Projekten:

- **Ein Weltraumspiel:** Dein Schiff ist ein Sprite, die Schüsse sind weitere. Du bewegst sie, indem du nur ihre Position änderst. Das Bild dahinter (Sterne) bleibt unberührt.
- **Ein Mauszeiger oder Auswahlrahmen** in einer Demo oder einem Werkzeug.
- **Eine Figur mit Gehanimation:** vier bis acht Frames, die der Reihe nach gezeigt werden.
- **Große Figuren:** mehrere Sprites nebeneinander, zum Beispiel ein Endgegner aus 2×2 Sprites.

Im fertigen Programm landet ein Sprite als **Datenblock** im Speicher (63 Bytes je Sprite beim C64). Dein Programm sagt dem Grafikchip, wo dieser Block liegt, und der Chip malt ihn. Das Studio erzeugt die Daten und die Farbkonstanten für dich; wie du sie einbindest, steht unten im Abschnitt „Vom Studio ins Programm“.

## Grundbegriffe ohne Vorwissen

- **Pixel / Punkt:** der kleinste Bildpunkt. Im Sprite-Studio siehst du ihn als Kästchen.
- **Sprite:** ein kleines Bild, das der Grafikchip **selbst** über den Bildschirm legt (C64: VIC-II, Atari: GTIA).
- **Frame:** ein Einzelbild eines Sprites. Mehrere Frames hintereinander ergeben eine Animation, wie bei einem Daumenkino.
- **Hires:** hochauflösend. Jeder Punkt ist an oder aus, das Sprite hat **eine** Farbe.
- **Multicolor:** mehrfarbig. Die Punkte sind doppelt so breit, dafür gibt es mehrere Farben. Die Auflösung sinkt, die Farbe steigt.
- **Pixelwert:** die „Farbe“, die ein Punkt trägt, als Nummer (zum Beispiel 0 = durchsichtig). Welche echte Farbe dahintersteckt, bestimmt ein Register des Chips.
- **Register:** eine Speicherstelle, in die du Zahlen schreibst, um den Chip zu steuern (zum Beispiel `$D027` für die Farbe von Sprite 0).
- **Sprite-Zeiger:** ein Byte, das dem Chip sagt, in welchem 64-Byte-Block die Daten eines Sprites liegen.
- **PMG (Player/Missile-Grafik):** so heißen die Sprites beim Atari.
- **Zwiebelhaut:** das vorige Frame schimmert blass durch, damit du Bewegungen sauber setzt. Der Schalter heißt **Zwiebel**.

## Das brauchst du

- ein C64- oder Atari-Projekt (für MEGA65 gelten die C64-Sprites),
- Grundkenntnisse aus [Grafik und Klang im Überblick](kapitel:grafik-ueberblick): Asset anlegen, exportieren, einbinden.

## Was ist ein Sprite?

Ein **Sprite** ist ein kleines Bild, das der Grafikchip **selbst über den Bildschirm legt**. Du musst es nicht ins Bild hineinmalen, sondern gibst nur an, wo es stehen soll. Dadurch lassen sich Figuren schnell und ruckelfrei bewegen, ohne den Hintergrund zu zerstören.

- **C64 (VIC-II):** acht Sprites, jedes **24×21 Punkte** groß (63 Bytes, im Speicher auf 64 Bytes aufgefüllt). Im **Hires**-Modus hat ein Sprite eine Farbe. Im **Multicolor**-Modus sind die Punkte doppelt so breit (also 12×21), dafür gibt es vier Werte: durchsichtig, die eigene **Sprite-Farbe** und zwei Farben, die sich **alle** Sprites teilen (**Multicolor 1** und **Multicolor 2**).
- **Atari (GTIA):** Der Atari nennt sie **Player/Missiles** („PMG“, Player-Missile-Grafik). Es gibt vier **Player**, jeder **8 Punkte breit** und so hoch wie der Bildschirm. Jeder Player hat eine eigene Farbe. Mehrere Player kannst du nebeneinander stellen, um größere Figuren zu bauen. Dazu kommen Missiles (schmale Geschosse), die dieses Handbuch nicht weiter behandelt.

## Mein erstes Sprite (am Beispiel `sprite_demo`)

Du brauchst dafür kein eigenes Projekt: Das Beispiel `examples/c64/sprite_demo` bringt schon ein Sprite mit, das du ansehen und ändern kannst.

1. **Beispiel öffnen:** Auf der Startseite „Beispiele“ ▸ `sprite_demo`. *Erwartet:* Links im Projektbaum erscheint der Ordner `graphics` mit `hero.vcsprite`.
2. **Sprite öffnen:** Doppelklick auf `hero.vcsprite`. *Erwartet:* Das Sprite-Studio öffnet in einem neuen Reiter. In der Mitte siehst du das Sprite stark vergrößert, links die Liste der Frames, rechts Karten.
3. **Malen:** Wähle den **Stift** (Taste **D**) und klicke in die Fläche. *Erwartet:* Ein breiter Punkt (Multicolor) erscheint in der gewählten Farbe, in der Kopfzeile erscheint ein **●** („ungespeichert“).
4. **Löschen:** Rechte Maustaste auf einen Punkt. *Erwartet:* Der Punkt wird durchsichtig.
5. **Farbe wechseln:** In der Karte **Werkzeug** klickst du auf einen anderen Pixelwert (oder drückst **1** bis **4**) und malst weiter. *Erwartet:* Der Stift setzt jetzt den neuen Wert.
6. **Rückgängig:** **Cmd/Strg+Z**. *Erwartet:* Der letzte Strich verschwindet; **Cmd/Strg+Umschalt+Z** holt ihn zurück.
7. **Speichern:** **Cmd/Strg+S**. *Erwartet:* Das **●** verschwindet.
8. **Bauen und starten:** **F7**, dann **F5**. *Erwartet:* Der Emulator zeigt dein geändertes Sprite auf dem Bildschirm.

## Aufbau der Oberfläche

Das Studio hat **sechs Zonen**, wie alle Studios in VintageCoder:

| Zone | Was sie zeigt |
|---|---|
| Kopfzeile | Art der Datei (**SPR** für C64, **PMG** für Atari), Name, **●** bei ungespeicherten Änderungen, Maße; rechts die Dateiaktionen (**PNG laden …**, **Exportieren …**, **Speichern**). Bei wenig Platz stehen weniger Knöpfe da, der Rest steckt im **⋯**-Menü. |
| Werkzeugleiste | genau eine Reihe: Stift, Radierer, Füllen, Linie, Rahmen, Auswahl, Pipette; Rückgängig/Wiederholen, **Transformieren ▾**, Zoom (− + Einpassen), Raster, Zwiebel. |
| Liste | die **Frames** des Sprites (bei Atari die vier Player). Klick wählt, die Pfeiltasten links/rechts wechseln das Frame. |
| Fläche | das gewählte Frame, stark vergrößert und immer eingepasst; Mausrad mit **Cmd/Strg** zoomt. |
| Karten (Seitenspalte) | **Werkzeug** (die Zeichenfarbe, also die Pixelwerte), **Dokument** (Modus dieses Frames, Farben, Palette) und **Sprite** (Hardware-Flags und Animationen). In schmalen Fenstern steht auch die Liste der **Frames** als Karte hier. |
| Fußzeile | Position der Maus (Zelle), Zoom und Meldungen; rechts der Hinweis **? Tasten**. Frame x von y und die Größe stehen in der Kopfzeile. |

**Werkzeuge und Tasten** (die Einzeltasten gelten, wenn der Mauszeiger über der Fläche steht; sie lassen sich unter **Einstellungen ▸ Editor** mit **Einzeltasten der Studios** abschalten):

| Taste | Wirkung |
|---|---|
| D / E / F | Stift / Radierer / Füllen |
| L / R | Linie / Rahmen |
| S / I | Auswahl / Pipette |
| X / Y | horizontal / vertikal spiegeln |
| 1 bis 4 | Pixelwert wählen |
| 0 / + / − | Einpassen / Vergrößern / Verkleinern |
| Cmd/Strg+C / X / V | Auswahl kopieren / ausschneiden / einfügen |
| Entf oder Rücktaste | Auswahl löschen |
| Cmd/Strg+Z | Rückgängig (Umschalt dazu: Wiederholen) |
| Cmd/Strg+S | Speichern |
| Cmd/Strg+Umschalt+E | Export-Dialog |

Zoom, Raster und Zwiebelhaut findest du auch in der **Befehlspalette** (Cmd/Strg+K, Einträge „Studio: …“). Das Kontextmenü der Fläche (Rechtsklick mit dem Auswahlwerkzeug, Menü-Taste oder Umschalt+F10) bietet Rückgängig, Wiederholen, Ausschneiden, Kopieren, Einfügen und Exportieren.

### Hires oder Multicolor – je Frame

Du kannst Frames eines Sprites mischen: Rechts unter **Modus dieses Frames** wählst du **Hires** oder **Multicolor** (oder drückst die Taste **C**, die zwischen beiden umschaltet). Beim Umschalten rechnet der Editor die Punkte um (bei Hires → Multicolor werden zwei Punkte zu einem). **Strg/Cmd+Z** macht den Wechsel rückgängig. Die zwei gemeinsamen Farben **Multicolor 1** und **2** gelten für alle Frames der Datei.

![Sprite-Studio mit einem Multicolor-Sprite: Frame-Liste links, Zeichenfläche in der Mitte, rechts die Karten Werkzeug, Dokument (Modus Hires/Multicolor und die drei Farben) und Sprite](bild:sprite-c64-multicolor "Das Sprite-Studio mit einem Multicolor-Sprite: vier Pixelwerte, davon teilen sich alle Sprites zwei Farben ($D025 und $D026)")")

### Weitere Sprite-Eigenschaften (C64)

In der Karte **Sprite** stehen bei C64-Sprites drei Schalter. Multicolor (`$D01C`) folgt dem Modus des Frames und hat deshalb keinen eigenen Schalter.

| Schalter | Bedeutung | Register |
|---|---|---|
| Doppelt breit (X2) | doppelt breit | `$D01D` |
| Doppelt hoch (Y2) | doppelt hoch | `$D017` |
| Hinter dem Hintergrund (BG) | hinter dem Hintergrund | `$D01B` |

Die drei Schalter gelten für die ganze Datei. Sie landen im Export als Konstanten (`<name>_expand_x`, `<name>_expand_y`, `<name>_behind`).

## Schritt für Schritt: ein Atari-Player

1. **＋** in der linken Spalte, im Dialog **Neue Datei** die Art **Sprites / PMG**, Profil **Atari PMG Einzelzeile (8 Pixel breit)**. Die Datei heißt `.vcpmg`.
2. In der linken Liste stehen die **vier Player P0 bis P3**. Jeder ist 8 Punkte breit und standardmäßig 32 Zeilen hoch. Die Farbe jedes Players wählst du rechts in der Karte **Dokument**. Beim Atari gibt es dafür 128 Töne (Videonorm PAL oder NTSC umschaltbar).
3. Mal wie beim C64.

![Der PMG-Editor für Atari: links die vier Player P0 bis P3, in der Mitte das Frame mit 8×32 Punkten, rechts Werkzeug und Dokument mit der Atari-Palette (PAL, NTSC oder Projekt).](bild:sprite-atari-pmg "Der PMG-Editor: links die vier Player, rechts die Farben")

> **Info:** Breite (`SIZEP0` bis `SIZEP3`) und Priorität der Player stellst du im Programm selbst ein (siehe das Beispiel unten). Dafür gibt es in der Oberfläche noch keine Schalter.

## Frames, Animation und Zwiebelhaut

Eine **Animation** entsteht aus mehreren Frames, die kurz nacheinander gezeigt werden (wie ein Daumenkino).

1. Links unter **Frames** legst du mit **+** ein neues leeres Frame an, mit **Dupl.** eine Kopie des gewählten, mit **Löschen** entfernst du eines. Die Pfeiltasten links/rechts wechseln das Frame.
2. Schalte **Zwiebel** ein (Werkzeugleiste): Das vorige Frame erscheint blass als Schatten unter deinem Bild. So triffst du die Abstände. Beim ersten Frame ist der Schalter ausgegraut, denn es gibt kein voriges.
3. Für eine Folge benutzt du den Abschnitt **Animationen** in der Karte **Sprite**. Die Vorschau dort läuft in Echtzeit: **+ Neu** legt eine Animation an (mehrere Animationen wählst du über ihre Knöpfe), **Frame anhängen** fügt das gewählte Frame als Schritt hinzu. Jeder Schritt hat eine **Dauer** (in „B“ = Bildern; 1 Bild = 1/50 Sekunde), du kannst ihn mit den Pfeilen verschieben oder mit × entfernen. Der **Ablauf** ist **Schleife**, **Hin und zurück** oder **Einmal** (ein Klick auf den Knopf schaltet weiter). **Löschen** entfernt die ganze Animation.

![Das Sprite-Studio mit der Karte „Sprite“: unter „Animationen“ die Animation „laufen“ mit Vorschau, Schrittzahl, Schleife und den Schritten mit Dauer in Bildern.](bild:sprite-animation "Das Animationsfeld: Schritte mit Dauer in Bildern, darüber Ablauf und Name")

Wenn es Animationen gibt, erzeugt der Export zusätzlich eine Datei `<name>.anim.asm` mit den Sprite-Zeigern (`_ptr`), den Dauern (`_delay`), der Länge (`_len`) und dem Modus (`_mode`). Ein Raster-Interrupt zählt die Dauer herunter und schreibt den nächsten Zeiger nach `$07F8` plus Sprite-Nummer.

### Auswahl und Rahmen

Mit dem Werkzeug **Auswahl** (Taste **S**) ziehst du einen Rahmen auf. Darin kannst du mit **Cmd/Strg+C**, **Cmd/Strg+X** und **Cmd/Strg+V** kopieren, ausschneiden und einfügen und mit **Entf** löschen. **Cmd/Strg+A** wählt das ganze Frame, **Esc** hebt die Auswahl auf. Das Werkzeug **Rahmen** (Taste **R**) zeichnet ein Rechteck als Umriss.

## Sprites aus anderen Quellen

**PNG laden …** lädt ein Bild und wandelt es in die Pixelwerte um. Das Übernehmen aus SPD-Dateien, aus Rohdaten und aus Sprites anderer Systeme ist in dieser Version noch nicht in der Oberfläche enthalten.

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

## Hardware-Besonderheiten und Grenzen

| | C64 (VIC-II) | Atari (GTIA) | MEGA65 |
|---|---|---|---|
| Anzahl | 8 Sprites | 4 Player + 4 Missiles | wie C64 |
| Größe | 24×21 Punkte (Multicolor: 12×21 breite Punkte) | 8 Punkte breit, Höhe frei bis Bildschirmhöhe | wie C64 |
| Farben | Hires: 1 Farbe je Sprite. Multicolor: 3 Farben + durchsichtig, davon 2 gemeinsam für alle Sprites | 1 Farbe je Player (128 Töne) | wie C64 |
| Größe ändern | X2/Y2 verdoppeln Breite/Höhe | `SIZEP`: einfach, doppelt, vierfach | wie C64 |
| Speicher | 64-Byte-Block, Adresse durch 64 teilbar | 256 Byte je Player, Basis durch `PMBASE` | wie C64 |

Was das bedeutet: Mehr als acht Sprites gleichzeitig gehen nur mit Tricks (Raster-Interrupt). Ein Multicolor-Sprite sieht farbiger aus, ist aber grober. Zwei Sprites desselben Chips teilen sich bei Multicolor immer zwei Farben, ändern sich also zusammen.

## Vom Studio ins Programm

1. **Exportieren** (Kopfzeile oder **Cmd/Strg+Umschalt+E**). Der Export-Dialog zeigt Format, Name, Adresse und eine Vorschau. Beim Bauen (**F7**) entsteht die Datei auch automatisch unter `build/export/`.
2. **Assembler:** `.include "build/export/hero.asm"` an die Stelle im Speicher, die du mit `.org` festlegst (C64: Vielfaches von 64). Der Export liefert Daten und die Farbkonstanten `hero_sprite`, `hero_mc1`, `hero_mc2`.
3. **BASIC:** Format **BASIC DATA** exportieren und mit `#include` einbinden; die Zeilen mit `READ` und `POKE` in den Sprite-Speicher kopieren.
4. **Rohdaten:** Format **Binär** für `.incbin` oder zum Laden von Diskette.
5. Im Programm setzt du **Zeiger, Farben, Position und „Sprite an“** (siehe die zwei Beispiele unten).

## Häufige Stolpersteine

- **Das Sprite ist verschoben oder zeigt Müll:** Die Daten liegen nicht auf einer 64-Byte-Grenze (C64) oder nicht bei `PMBASE + $400` (Atari).
- **Die Farben im Emulator weichen ab:** Die Sprite-Farben stehen in den Registern, nicht in den Daten. Setze sie im Programm, am besten mit den Export-Konstanten (`hero_sprite`, `hero_mc1`, `hero_mc2`).
- **Das Sprite erscheint nicht:** `vic2.spena` hat das Bit nicht; oder die Position liegt außerhalb des sichtbaren Bereichs (links beginnt der sichtbare Bereich etwa bei X = 24, oben bei Y = 50).
- **Multicolor wirkt zu breit:** Das ist so: Ein Multicolor-Punkt ist zwei Hires-Punkte breit.

## Nachbar-Studios und Glossar

- Der Hintergrund, vor dem sich dein Sprite bewegt, entsteht im [Bildschirm- und Kachel-Editor](kapitel:bildschirm-kacheln).
- Die Farben wählst du aus der [Palette](kapitel:paletten) deines Systems.
- Begriffe nachschlagen: Glossar (Datei `GLOSSAR.md`) (Sprite, Frame, Multicolor, Sprite-Zeiger, PMG).

## Wie geht es weiter?

Im nächsten Kapitel setzt du Zeichen und Kacheln zu ganzen Bildern zusammen: [Bildschirm- und Kachel-Editor](kapitel:bildschirm-kacheln). Nachschlagen: [Sprite-/PMG-Editor](hilfe:sprite), [Export](hilfe:export).
