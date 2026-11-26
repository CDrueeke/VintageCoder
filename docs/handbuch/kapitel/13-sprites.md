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

## Wofür ist der Editor gedacht?

Im Sprite-Editor malst du die **beweglichen Figuren** deines Programms: den Spieler, Gegner, Schüsse, einen Mauszeiger, ein Raumschiff. Alles, was sich über einen Hintergrund bewegen soll, ohne ihn zu zerstören, ist ein Kandidat für ein Sprite.

Typische Einsätze aus echten Retro-Projekten:

- **Ein Weltraumspiel:** Dein Schiff ist ein Sprite, die Schüsse sind weitere. Du bewegst sie, indem du nur ihre Position änderst. Das Bild dahinter (Sterne) bleibt unberührt.
- **Ein Mauszeiger oder Auswahlrahmen** in einer Demo oder einem Werkzeug.
- **Eine Figur mit Gehanimation:** vier bis acht Frames, die der Reihe nach gezeigt werden.
- **Große Figuren:** mehrere Sprites nebeneinander, zum Beispiel ein Endgegner aus 2×2 Sprites.

Im fertigen Programm landet ein Sprite als **Datenblock** im Speicher (63 Bytes je Sprite beim C64). Dein Programm sagt dem Grafikchip, wo dieser Block liegt, und der Chip malt ihn. Der Editor erzeugt die Daten und die Farbkonstanten für dich; wie du sie einbindest, steht unten im Abschnitt „Vom Editor ins Programm“.

## Grundbegriffe ohne Vorwissen

- **Pixel / Punkt:** der kleinste Bildpunkt. Im Sprite-Editor siehst du ihn als Kästchen.
- **Sprite:** ein kleines Bild, das der Grafikchip **selbst** über den Bildschirm legt (C64: VIC-II, Atari: GTIA).
- **Frame:** ein Einzelbild eines Sprites. Mehrere Frames hintereinander ergeben eine Animation, wie bei einem Daumenkino.
- **Hires:** hochauflösend. Jeder Punkt ist an oder aus, das Sprite hat **eine** Farbe.
- **Multicolor:** mehrfarbig. Die Punkte sind doppelt so breit, dafür gibt es mehrere Farben. Die Auflösung sinkt, die Farbe steigt.
- **Pixelwert:** die „Farbe“, die ein Punkt trägt, als Nummer (zum Beispiel 0 = durchsichtig). Welche echte Farbe dahintersteckt, bestimmt ein Register des Chips.
- **Register:** eine Speicherstelle, in die du Zahlen schreibst, um den Chip zu steuern (zum Beispiel `$D027` für die Farbe von Sprite 0).
- **Sprite-Zeiger:** ein Byte, das dem Chip sagt, in welchem 64-Byte-Block die Daten eines Sprites liegen.
- **PMG (Player/Missile-Grafik):** so heißen die Sprites beim Atari.
- **Onion Skin:** das vorige Frame schimmert blass durch, damit du Bewegungen sauber setzt. Der Schalter heißt **Onion Skin**.

## Das brauchst du

- ein C64- oder Atari-Projekt (für MEGA65 gelten die C64-Sprites),
- Grundkenntnisse aus [Grafik und Klang im Überblick](kapitel:grafik-ueberblick): Asset anlegen, exportieren, einbinden.

## Was ist ein Sprite?

Ein **Sprite** ist ein kleines Bild, das der Grafikchip **selbst über den Bildschirm legt**. Du musst es nicht ins Bild hineinmalen, sondern gibst nur an, wo es stehen soll. Dadurch lassen sich Figuren schnell und ruckelfrei bewegen, ohne den Hintergrund zu zerstören.

- **C64 (VIC-II):** acht Sprites, jedes **24×21 Punkte** groß (63 Bytes, im Speicher auf 64 Bytes aufgefüllt). Im **Hires**-Modus hat ein Sprite eine Farbe. Im **Multicolor**-Modus sind die Punkte doppelt so breit (also 12×21), dafür gibt es vier Werte: durchsichtig, die eigene **Sprite-Farbe** und zwei Farben, die sich **alle** Sprites teilen (**Multicolor 1** und **Multicolor 2**).
- **Atari (GTIA):** Der Atari nennt sie **Player/Missiles** („PMG“, Player-Missile-Grafik). Es gibt vier **Player**, jeder **8 Punkte breit** und so hoch wie der Bildschirm. Jeder Player hat eine eigene Farbe. Mehrere Player kannst du nebeneinander stellen, um größere Figuren zu bauen. Dazu kommen Missiles (schmale Geschosse), die dieses Handbuch nicht weiter behandelt.

## Mein erstes Sprite (am Beispiel `sprite_demo`)

Du brauchst dafür kein eigenes Projekt: Das Beispiel `examples/c64/sprite_demo` bringt schon ein Sprite mit, das du ansehen und ändern kannst.

1. **Beispiel öffnen:** Öffne auf der Startseite aus den Beispielprojekten `sprite_demo` (zum Ändern nimmst du **Kopieren**, dann hast du eine eigene Fassung). *Erwartet:* Das Projekt ist offen; im Projektbaum liegt der Ordner `graphics` mit `hero.vcsprite`.
2. **Sprite öffnen:** Wähle oben den Hauptbereich **Grafik** und in der zweiten Reihe **Sprites**. Links steht die Liste der **Sprite-Dateien**; klicke auf `hero.vcsprite`. (Ein Doppelklick auf die Datei im Projektbaum öffnet sie auch im Sprite-Editor.) *Erwartet:* In der Mitte siehst du das Sprite stark vergrößert, links die Bilder (Frames), rechts die Karten **Farben**, **Sprite** und **Vorschau im Spiel**.
3. **Malen:** Wähle den **Stift** (Taste **D**) und klicke in die Fläche. *Erwartet:* Ein breiter Punkt (Multicolor) erscheint in der gewählten Farbe, in der Fußzeile steht „● ungespeichert“.
4. **Löschen:** Rechte Maustaste auf einen Punkt. *Erwartet:* Der Punkt wird durchsichtig.
5. **Farbe wechseln:** In der Karte **Farben** klickst du auf einen anderen Farbwert (oder drückst **1** bis **4**) und malst weiter. *Erwartet:* Der Stift setzt jetzt den neuen Wert.
6. **Rückgängig:** **Cmd/Strg+Z**. *Erwartet:* Der letzte Strich verschwindet; **Cmd/Strg+Umschalt+Z** holt ihn zurück.
7. **Speichern:** **Cmd/Strg+S**. *Erwartet:* Der Hinweis „ungespeichert“ verschwindet.
8. **Bauen und starten:** **F7**, dann **F5**. *Erwartet:* Der Emulator zeigt dein geändertes Sprite auf dem Bildschirm.

Eine neue Sprite-Datei legst du mit **Neue Sprite-Datei …** an (links unter der Dateiliste, auch auf der leeren Seite des Editors). Im Dialog wählst du unter **Art und System** das Profil, gibst den **Namen** ein (Buchstaben, Ziffern, `_` und `-`), prüfst den **Ordner im Projekt** (Vorgabe `graphics`) und klickst auf **Anlegen**. **Sprite-Datei öffnen …** holt eine vorhandene Datei aus dem Projekt.

## Aufbau der Oberfläche

Der Sprite-Editor hat drei Bereiche nebeneinander:

| Bereich | Was er zeigt |
|---|---|
| Links | die **Sprite-Dateien** des Projekts, **Sprite-Datei öffnen …**, **Neue Sprite-Datei …** und darunter die Reiter **Bilder** und **Animationen** |
| Mitte | die Werkzeugleiste, die **Zeichenfläche** mit dem gewählten Bild (stark vergrößert, eingepasst) und darunter die Zeitleiste der Animation; ganz unten die Fußzeile |
| Rechts | die Karten **Farben**, **Palette**, bei Atari **Player/Missile**, **Sprite**, **Vorschau im Spiel** und **Verschieben** |

Die **Werkzeugleiste** enthält, von links nach rechts: **Stift** (D), **Radierer** (E), **Pipette** (I), **Füllen** (F), **Linie** (L), **Rechteck** (R), **Auswahl** (S), **Waagerecht spiegeln** (X), **Senkrecht spiegeln** (Y), Rückgängig und Wiederholen, **Onion Skin** (O), **Raster** (G), Zoom (**Verkleinern**, Anzeige in Prozent, **Vergrößern**, **Einpassen**) und rechts die Dateiknöpfe **PNG laden …**, **Exportieren …** und **Speichern**. Die Knöpfe zeigen beim Daraufzeigen ihre Taste. Die **Fußzeile** nennt Name, Größe und „Bild i von n“; ungespeicherte Änderungen markiert ein **●**, eine Auswahl nennt dort ihre Größe.

![Der Reiter Grafik mit der Seite „Sprites“ und der Datei hero.vcsprite: links die Sprite-Dateien und zwei Bilder, in der Mitte das 12x21 Pixel große Multicolor-Sprite einer Figur in Gelb, Rotbraun und Blau, rechts die Karten „Farben“, „Sprite“ (Mehrfarbig an), „Vorschau im Spiel“ und „Verschieben“. Unter der Zeichenfläche steht der Hinweis „Keine Animation“.](bild:sprite-c64 "Der Sprite-Editor mit einem Multicolor-Sprite.")

**Tasten** (die Einzeltasten gelten, wenn die Zeichenfläche den Fokus hat):

| Taste | Wirkung |
|---|---|
| D / E / I | Stift / Radierer / Pipette |
| F / L / R / S | Füllen / Linie / Rechteck / Auswahl |
| X / Y | waagerecht / senkrecht spiegeln |
| 1 bis 4 | Farbwert wählen (Ziffern bis 9, soweit das Sprite so viele Werte hat) |
| O / G | Onion Skin / Raster |
| 0 / + / − | Einpassen / Vergrößern / Verkleinern (auch **Cmd/Strg** und Mausrad) |
| Pfeiltasten | Zellmarke bewegen; **Eingabe** oder **Leertaste** setzt einen Punkt, **Entf** oder Rücktaste löscht ihn |
| Bild auf / Bild ab, Alt+Pfeil links/rechts | voriges / nächstes Bild |
| Cmd/Strg+C / X / V | Auswahl kopieren / ausschneiden / einfügen |
| Esc | Auswahl aufheben |
| Cmd/Strg+Z | Rückgängig (Umschalt dazu: Wiederholen) |
| Cmd/Strg+S | Speichern |
| Cmd/Strg+Umschalt+E | Export-Dialog |

Alle Werkzeuge, Zoom, Raster und Onion Skin erreichst du auch über die **Befehlspalette** (Cmd/Strg+K). Mit dem Werkzeug **Auswahl** öffnet ein Rechtsklick auf die Fläche ein Menü mit Kopieren, Ausschneiden, Einfügen, Alles auswählen, Auswahl löschen und Auswahl aufheben.

### Hires oder Multicolor, je Bild

Du kannst Bilder eines Sprites mischen: In der Karte **Sprite** schaltest du **Mehrfarbig** ein oder aus (der Schalter gilt für das gewählte Bild). Beim Umschalten rechnet der Editor die Punkte um (bei Hires zu Multicolor werden zwei Punkte zu einem). **Cmd/Strg+Z** macht den Wechsel rückgängig. Die zwei gemeinsamen Farben **Multicolor 1** und **Multicolor 2** gelten für alle Bilder der Datei.

*[Bild folgt in E13: sprite-c64-multicolor]*

### Weitere Sprite-Eigenschaften (C64)

In der Karte **Sprite** stehen bei C64-Sprites außer **Mehrfarbig** (`$D01C`) drei Schalter:

| Schalter | Bedeutung | Register |
|---|---|---|
| Doppelt breit | doppelt breit (der Editor nennt die Größe „Im Spiel auf n Punkte“) | `$D01D` |
| Doppelt hoch | doppelt hoch | `$D017` |
| Hinter dem Hintergrund | hinter dem Hintergrund | `$D01B` |

Die Schalter gelten für die ganze Datei. Sie landen im Export als Konstanten (`<name>_expand_x`, `<name>_expand_y`, `<name>_behind`). Die **Vorschau im Spiel** zeigt das gewählte Bild vor einem festen Hintergrund aus Himmel und Wiese, in den Farben des Zielsystems und mit den gesetzten Schaltern, einmal vergrößert und einmal in Originalgröße. Darunter steht, wie viele Bytes ein Bild und die ganze Datei belegen.

### Farben und Palette

In der Karte **Farben** stehen die Farbwerte des Sprites als Liste (zum Beispiel durchsichtig, **Sprite-Farbe**, **Multicolor 1**, **Multicolor 2**). Du wählst einen Wert per Klick oder Zifferntaste; der Stift malt dann damit. Welche echte Farbe ein Wert bekommt, wählst du darunter im Farbraster: Klicke auf eine Farbe, dann gilt sie für den gewählten Wert. Die Überschrift **Palette** nennt die Videonorm. Wo es PAL- und NTSC-Farben gibt (Atari), stellst du die **Palettenansicht** zwischen **Projekt**, **PAL** und **NTSC** um. Ein durchsichtiger Wert hat keine Farbe.

## Schritt für Schritt: ein Atari-Player

1. Lege mit **Neue Sprite-Datei …** eine Datei an und wähle unter **Art und System** das Profil **Atari PMG Einzelzeile (8 Pixel breit)**. Die Datei heißt `.vcpmg`.
2. Links stehen unter **Bilder** die **vier Player P0 bis P3**. Jeder ist 8 Punkte breit und standardmäßig 32 Zeilen hoch. Die Farbe jedes Players wählst du rechts in der Karte **Farben**. Beim Atari gibt es dafür 128 Töne (Palettenansicht PAL oder NTSC umschaltbar).
3. Mal wie beim C64.

*[Bild folgt in E13: sprite-atari-pmg]*

Die Karte **Player/Missile** stellt zwei Dinge ein, die im Export als Konstanten landen:

- **Breite Spieler 0 bis 3** (`SIZEP0` bis `SIZEP3`): **Normal**, **Doppelt** oder **Vierfach**.
- **Priorität** (`PRIOR`, `$D01B`): **Nicht gesetzt**, **Spieler vor Spielfeld**, **P0/P1, Spielfeld, P2/P3**, **PF0/PF1, Spieler, PF2/PF3** oder **Spielfeld vor Spielern**. Eine nicht gesetzte Priorität wird mit „Spieler vor Spielfeld“ exportiert.

> **Info:** Ein Player zeigt Zeile für Zeile den Speicher von oben nach unten. Die ersten etwa 32 Zeilen liegen über dem sichtbaren Bild: Zeichne das Motiv weiter unten, oder erhöhe die Höhe des Bildes.

## Frames, Animation und Onion Skin

Eine **Animation** entsteht aus mehreren Frames (Bildern), die kurz nacheinander gezeigt werden (wie ein Daumenkino).

1. Links im Reiter **Bilder** legst du mit **Neu** ein neues leeres Bild an, mit **Duplizieren** eine Kopie des gewählten, mit **Löschen** entfernst du eines. Ein Klick auf ein Bild wählt es; **Bild auf/ab** oder **Alt+Pfeil links/rechts** wechseln das Bild.
2. Schalte **Onion Skin** ein (Werkzeugleiste oder Taste **O**): Das vorige Bild erscheint blass als Schatten unter deinem Bild. So triffst du die Abstände. Beim ersten Bild ist der Schalter ausgegraut, denn es gibt kein voriges.
3. Für eine Folge wechselst du links zum Reiter **Animationen**. **Neu** legt eine Animation mit dem gewählten Bild an; mehrere Animationen wählst du über ihre Knöpfe, im Feld **Name der Animation** benennst du sie, **Löschen** entfernt die ganze Animation. Der **Ablauf** ist **Schleife**, **Hin und zurück** oder **Einmal**. Die Schritte stehen als Filmstreifen: Jeder zeigt Miniatur, Nummer und **Dauer in Bildern** (1 Bild = 1/50 Sekunde). Du verschiebst einen Schritt mit den Pfeilknöpfen oder durch Ziehen, entfernst ihn oder hängst mit **Gewähltes Bild anhängen** das gewählte Bild als neuen Schritt an. Das Feld **Tempo (Bilder pro Sekunde)** setzt die Dauer aller Schritte auf einmal.
4. **Abspielen:** Unter der Zeichenfläche zeigt die **Zeitleiste** die gewählte Animation. Der Knopf **Abspielen** startet sie in Echtzeit, **Anhalten** stoppt sie. Der Ablauf lässt sich dort per Klick weiterschalten, und **Vorschau vergrößern** öffnet die Animation in einem eigenen Fenster. Ein Klick auf einen Schritt wählt sein Bild zum Bearbeiten.

![Der Sprite-Editor mit hero.vcsprite, Reiter „Animationen“. Links die Animation „laufen“ mit Schleife, Tempo 8,3 und zwei Bildern, in der Mitte die vergrößerte Figur, darunter die Zeitleiste mit der Animation und zwei Bildern, rechts Farben und die Vorschau im Spiel.](bild:sprite-animation "Der Reiter „Animationen“ mit der Animation „laufen“ und der Zeitleiste unter der Zeichenfläche.")

Es geht auch umgekehrt: Markiere in der Bilderliste mit **Umschalt-** oder **Strg-Klick** mehrere Bilder (**Cmd/Strg+A** markiert alle) und klicke auf **Animation aus Auswahl**. Der Dialog **Animation aus n Bildern** fragt nach dem Namen, dem **Ablauf** und dem **Tempo (Bilder pro Sekunde)**; die Bilder laufen in der Reihenfolge der Liste. Das Menü per Rechtsklick auf ein Bild bietet dasselbe als **Animation aus Auswahl …**.

Wenn es Animationen gibt, erzeugt der Export zusätzlich eine Datei `<name>.anim.asm` mit den Sprite-Zeigern (`_ptr`), den Dauern (`_delay`), der Länge (`_len`) und dem Modus (`_mode`). Ein Raster-Interrupt zählt die Dauer herunter und schreibt den nächsten Zeiger nach `$07F8` plus Sprite-Nummer.

### Auswahl und Rechteck

Mit dem Werkzeug **Auswahl** (Taste **S**) ziehst du einen Rahmen auf. Darin kannst du mit **Cmd/Strg+C**, **Cmd/Strg+X** und **Cmd/Strg+V** kopieren, ausschneiden und einfügen und mit **Entf** löschen. **Cmd/Strg+A** wählt das ganze Bild, **Esc** hebt die Auswahl auf. Das Werkzeug **Rechteck** (Taste **R**) zeichnet ein Rechteck als Umriss. Unter **Verschieben** in der rechten Spalte schiebst du das ganze Bild mit vier Pfeilknöpfen um einen Punkt; **Bild leeren** löscht alle Punkte des gewählten Bildes.

## Sprites aus anderen Quellen

**PNG laden …** (Werkzeugleiste) wählt eine Bilddatei und wandelt sie in die Pixelwerte des gewählten Bildes um; dabei speichert der Editor die Datei. Das Übernehmen aus SPD-Dateien, aus Rohdaten und aus Sprites anderer Systeme steht in dieser Version noch nicht zur Verfügung.

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
.include "graphics/ship.vcpmg"
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

## Vom Editor ins Programm

1. **Exportieren …** (Werkzeugleiste oder **Cmd/Strg+Umschalt+E**) öffnet den Export-Dialog. Dort wählst du das **Format** (**Assembler**, **Konstanten**, **Binär**, **PRG** oder **BASIC DATA**), den **Namen** und je nach Format eine **Adresse**. Die **Vorschau** zeigt Text und Bytes. Mit **In Projekt speichern**, **Als Include einfügen**, **In Quelltext kopieren** oder **Im Emulator testen** bestimmst du, was mit dem Ergebnis geschieht. Zum Einbinden brauchst du den Export nicht: Beim Bauen (**F7**) wandelt VintageCoder das Sprite beim `.include` selbst um.
2. **Assembler:** `.include "graphics/hero.vcsprite"` an die Stelle im Speicher, die du mit `.org` festlegst (C64: Vielfaches von 64). Der Export liefert Daten und die Farbkonstanten `hero_sprite`, `hero_mc1`, `hero_mc2`.
3. **BASIC:** Format **BASIC DATA** exportieren (auf Wunsch mit **Einleseroutine mitliefern**) und mit `#include` einbinden; die Zeilen mit `READ` und `POKE` in den Sprite-Speicher kopieren.
4. **Rohdaten:** Format **Binär** für `.incbin` oder zum Laden von Diskette.
5. Im Programm setzt du **Zeiger, Farben, Position und „Sprite an“** (siehe die zwei Beispiele oben).

## Häufige Stolpersteine

- **Das Sprite ist verschoben oder zeigt Müll:** Die Daten liegen nicht auf einer 64-Byte-Grenze (C64) oder nicht bei `PMBASE + $400` (Atari).
- **Die Farben im Emulator weichen ab:** Die Sprite-Farben stehen in den Registern, nicht in den Daten. Setze sie im Programm, am besten mit den Export-Konstanten (`hero_sprite`, `hero_mc1`, `hero_mc2`).
- **Das Sprite erscheint nicht:** `vic2.spena` hat das Bit nicht; oder die Position liegt außerhalb des sichtbaren Bereichs (links beginnt der sichtbare Bereich etwa bei X = 24, oben bei Y = 50).
- **Multicolor wirkt zu breit:** Das ist so: Ein Multicolor-Punkt ist zwei Hires-Punkte breit.

## Nachbar-Editoren und Glossar

- Der Hintergrund, vor dem sich dein Sprite bewegt, entsteht im [Bildschirm- und Kachel-Editor](kapitel:bildschirm-kacheln).
- Die Farben wählst du aus der [Palette](kapitel:paletten) deines Systems.
- Begriffe nachschlagen: Glossar (Datei `GLOSSAR.md`) (Sprite, Frame, Multicolor, Sprite-Zeiger, PMG).

## Wie geht es weiter?

Im nächsten Kapitel setzt du Zeichen und Kacheln zu ganzen Bildern zusammen: [Bildschirm- und Kachel-Editor](kapitel:bildschirm-kacheln). Nachschlagen: [Sprite-/PMG-Editor](hilfe:sprite), [Export](hilfe:export).
