---
id: tracker
titel: Der Tracker
reihenfolge: 16
stufe: Einsteiger
zeit: 40 Minuten
siehe_auch: [play-tracker, grafik-ueberblick, systeme]
referenz: [tracker, envelope, export]
---

# Der Tracker

**Ziel:** Du komponierst ein kurzes Musikstück im Tracker, hörst es dir an, exportierst es und spielst es in einem Programm ab.

## Das brauchst du

- ein Projekt für ein System mit Klangchip, hier vor allem **C64** (Chip **SID**, 3 Stimmen) oder **Atari** (Chip **POKEY**, 4 Stimmen),
- Lautsprecher oder Kopfhörer. Die Hörprobe läuft in der IDE selbst, du brauchst dafür keinen Emulator.

## Was ist ein Tracker?

Ein **Tracker** ist ein Musikprogramm, in dem du Noten nicht auf Notenlinien, sondern in **Tabellen** setzt. Jede **Spalte** ist eine **Stimme** (ein Kanal des Klangchips), jede **Zeile** ein Zeitschritt. Er stammt aus der Computermusik der 1980er und passt perfekt zu den Chips: Jeder Kanal spielt nur einen Ton zugleich.

Die wichtigsten Begriffe:

- **Zelle:** ein Eintrag aus Note und Instrument, zum Beispiel `C-4 01` (Note C in Oktave 4, Instrument 01). `---` bedeutet „nichts“, `===` „Note aus“.
- **Muster** (englisch *Pattern*): eine Tabelle mit einer festen Zahl von Zeilen, zum Beispiel 32. Ein Muster ist ein Baustein, etwa „vier Takte Bass“.
- **Reihenfolge** (englisch *Order*): die Liste, in welcher Reihenfolge die Muster gespielt werden. Weil du Muster wiederverwenden kannst, bleiben Songs klein.
- **Instrument:** die Klangfarbe. Ein Instrument bestimmt Wellenform und **Hüllkurve**, also wie ein Ton laut wird und wieder verklingt (siehe unten).
- **Takt** (im Tracker die Geschwindigkeit): die Zahl der **Bilder** je Zeile. Ein Bild ist ein Bildschirmdurchlauf, bei PAL 1/50 Sekunde. Takt 6 heißt: Jede Zeile dauert sechs Bilder.

## Schritt für Schritt: einen Song anlegen

1. **+** im Projektbaum ▸ **Song …**, Namen `tune`, Profil **C64 SID-Song (3 Stimmen)** (oder **Atari POKEY-Song (4 Stimmen)**), **Anlegen**. Der Tracker öffnet.
2. **Mitte:** das Raster des gewählten Musters. Jede Spalte heißt **Stimme 1**, **Stimme 2** usw. Die laufende Zeile ist blau hinterlegt.
3. **Links:** die **Reihenfolge** und die **Patterns**. **Rechts:** die **Instrumente** mit Klangeinstellungen.
4. **Oben** in der Werkzeugleiste: **Speichern**, **Exportieren …**, **In Code einfügen**, **Konvertieren von …**, **▶ Song**, **▶ Pattern**, **Takt**, **Zeilen**, die aktuelle **Oktave** und die **Bildrate**.

![Tracker mit drei Stimmen: ① Reihenfolge und Patterns, ② Raster, ③ Instrumente, ④ Werkzeugleiste, ⑤ Export-Streifen](bild:tracker-ueberblick "Der Tracker: links Reihenfolge und Muster, in der Mitte das Raster, rechts die Instrumente")

### Noten eingeben

Klicke eine Zelle an oder bewege dich mit den **Pfeiltasten** (**Bild auf/ab** springt acht Zeilen, **Pos1/Ende** zur ersten/letzten Zeile). Dann spielst du mit dem Computer-Keyboard wie auf einem Klavier:

```text
obere Reihe:   Q  2  W  3  E  R  5  T  6  Y  7  U  I      (eine Oktave höher)
untere Reihe:  Z  S  X  D  C  V  G  B  H  N  J  M         (aktuelle Oktave)
```

Nach jeder Note rückt der Cursor eine Zeile weiter. Weitere Tasten:

| Taste | Wirkung |
|---|---|
| Punkt, Entf oder Rücktaste | Zelle leeren |
| **1** | „Note aus“ (`===`) setzen |
| **[** und **]** | voriges/nächstes **Instrument** wählen |
| **F9** und **F10** | Oktave tiefer/höher |
| Leertaste | Wiedergabe starten/stoppen |
| Strg/Cmd+S | speichern |
| Strg/Cmd+Z, Strg/Cmd+Umschalt+Z | rückgängig, wiederholen |

Unter dem Raster zeigt eine **Klaviertastatur**, welche Töne gerade klingen. Ein Mausklick auf eine Taste spielt den Ton mit dem gewählten Instrument.

![Ein Muster mit Noten in drei Stimmen: ① Cursor, ② Stimmenköpfe](bild:tracker-muster "Ein Muster: jede Spalte eine Stimme, jede Zeile ein Zeitschritt")

### Muster und Reihenfolge

1. Links unter **Patterns** legt **+ Neu** ein leeres Muster an, **Dupl.** kopiert das gewählte. Ein Klick auf einen Eintrag („Pattern 01“) schaltet das Raster um.
2. In der Liste **Reihenfolge** stehen die Muster in der Abspielreihenfolge. **+** hängt das aktuelle Muster an, **−** entfernt einen Eintrag, **Setzen** ersetzt den gewählten Eintrag durch das aktuelle Muster.
3. Das Feld **Wiederholung ab** legt fest, bei welcher Position der Song nach dem Ende wieder beginnt. Das Zeichen ↺ in der Liste zeigt die Stelle.
4. Die **Zeilenzahl** des Musters (1 bis 128) änderst du in der Werkzeugleiste unter **Zeilen**, die Geschwindigkeit unter **Takt**.

### Instrumente und Hüllkurve

Rechts wählst du ein **Instrument** aus der Liste (Nummer und Name, Hexzahl wie in der Zelle). **+** legt ein neues an, **Dupl.** kopiert. Darunter stellst du es ein.

**SID (C64):**

- **Wellenform:** Dreieck (weich), Sägezahn (hell und schneidend), Puls (kräftig, die Breite stellst du mit **Pulsbreite** ein) und Rauschen (für Trommeln und Explosionen).
- **Hüllkurve** aus vier Werten, **A**ttack, **D**ecay, **S**ustain, **R**elease (englisch ADSR). Sie beschreibt den Lautstärkeverlauf eines Tons: *Attack* ist die Zeit, in der er vom Nichts auf volle Lautstärke steigt; *Decay* die Zeit, in der er auf die Haltelautstärke *Sustain* fällt; *Release* die Zeit, in der er nach „Note aus“ verklingt. Jeder Wert läuft von 0 bis 15; für Sustain ist es der **Pegel**, kein Zeitwert.

![Instrument mit Wellenform, Pulsbreite und ADSR-Reglern: ① Instrumentliste, ② ADSR-Regler](bild:tracker-instrument "Ein SID-Instrument: Wellenform, Pulsbreite und die vier Hüllkurvenwerte")

Unter den Reglern zeichnet der Editor die **Hüllkurve** als Kurve. **Hörprobe (C-4)** spielt das Instrument einmal.

![Hüllkurve eines Instruments mit Attack-Spitze, Decay und Release: ① Kurvenplot](bild:tracker-huellkurve "Die Hüllkurve: ansteigend bis zur Spitze (Attack), fallend auf das Haltepegel (Decay), nach Note aus ausklingend (Release)")

**POKEY (Atari):** Hier gibt es keine ADSR-Hüllkurve, sondern eine **Verzerrung** (Reiner Ton, Rauschen, Buzz, Gritty oder Bass) und **Lautstärke je Bild**: 16 Werte von 0 bis 15, ein Wert je Bild. Das Feld **Wiederholung ab** sagt, ab welchem Schritt die Kurve sich wiederholt.

![POKEY-Instrument mit Verzerrung und 16 Lautstärkewerten](bild:tracker-pokey "POKEY-Instrumente haben Verzerrung und eine 16-Schritte-Lautstärkekurve")

### Hören

- **▶ Song** spielt ab der aktuellen Position, **▶ Pattern** nur das aktuelle Muster. Der Knopf wird zu **■ Stopp**. Die Leertaste macht dasselbe.
- Die Hörprobe verwendet einen **eingebauten Synthesizer**, der die Chips nachahmt. Sie klingt ähnlich, aber nicht identisch.
- Die **Bildrate** (50 Hz für PAL, 60 Hz für NTSC) bestimmt das Tempo der Hörprobe. Stelle sie so ein, wie dein Programm später laufen soll.
- Über den Kanälen siehst du ein kleines **Oszilloskop** und einen **Pegel** je Stimme sowie die Schalter **M** (stumm) und **S** (solo). In der Seitenleiste sind ein Spektrum und ein Summen-Oszilloskop. Unter **Anzeige** kannst du alles abschalten.

![Tracker während der Wiedergabe mit Pegeln, Oszilloskopen und leuchtenden Noten](bild:tracker-hoerprobe "Während der Hörprobe leuchten die gerade einsetzenden Töne orange; über jeder Stimme laufen Oszilloskop und Pegel")

## Songs von anderswo holen

- **Konvertieren von …** holt einen Song aus einem anderen System in dieses Projekt (**SID ↔ POKEY**), eine **GoatTracker-Datei** (`.sng`) oder eine **SID-Datei** (`.sid`). Bei SID-Dateien läuft das Abspielprogramm in einem eingebauten 6502-Kern; der Editor zeichnet die Registerwerte auf und baut daraus Noten und Instrumente (Dialog: **Subtune** und **Länge in Sekunden**). Das Ergebnis ist eine Annäherung; Vibrato, Filter und Ähnliches fallen weg. Die Ausgabe nennt, was nicht übernommen wurde.
- **RMT importieren …** (nur POKEY) lädt ein Modul des **Raster Music Trackers**, des klassischen Atari-Trackers.

![Dialog zum Import eines RMT-Moduls](bild:tracker-import-rmt "RMT-Module aus dem Atari-Tracker lassen sich als POKEY-Song laden")

![Dialog Konvertieren von … mit Quell- und Zielchip SID und POKEY](bild:tracker-sid-pokey "Konvertieren von …: ein Song wird für den Chip dieses Trackers umgewandelt")

## Einbinden im Programm

Wie bei allen Studios entsteht der Export beim Bauen von selbst (Fuß ▸ **Export …** für den Dialog).

### C64 (SID)

Das Projekt `examples/c64/music_demo` enthält `music/tune.vcsong`. Das Programm:

```asm
.target c64
.cpu 6510 { illegals = deny }

.org $0801
.basicstub start

.include "lib/music.inc"

start:  sei
        music_init tune_order, tune_patterns, tune_ins, tune_speed, tune_rows, tune_patsize
        ; Rasterzeilen-IRQ statt KERNAL-IRQ: genau ein Takt je Bild (PAL 50 Hz, NTSC 60 Hz).
        lda #$7f
        sta $dc0d                   ; CIA-Interrupts aus
        sta $dd0d
        lda $dc0d                   ; ausstehende Meldungen quittieren
        lda $dd0d
        lda #$01
        sta $d01a                   ; Raster-IRQ an
        lda #$00
        sta $d012                   ; Rasterzeile 0
        lda $d011
        and #$7f
        sta $d011
        lda #<irq
        sta $0314
        lda #>irq
        sta $0315
        cli
loop:   jmp loop

irq:    lda #$01
        sta $d019                   ; Raster-IRQ quittieren
        jsr music_play
        jmp $ea81                   ; Register zurückholen, RTI

music_runtime
.include "build/export/tune.asm"
```

- `music_init` übergibt dem Spieler die Tabellen des Exports: Reihenfolge, Muster, Instrumente, Geschwindigkeit, Zeilenzahl und Mustergröße.
- Die **Spielroutine** `music_play` muss **einmal pro Bild** aufgerufen werden. Das erledigt ein **Raster-Interrupt**: Der Grafikchip unterbricht das Hauptprogramm in Rasterzeile 0 und springt nach `irq`.
- `music_runtime` legt die Spielroutine ab, **hinter** dem Hauptprogramm.

> **Achtung:** Nimm den **Raster**-Interrupt, nicht den KERNAL-Interrupt. Der KERNAL-Interrupt des C64 läuft auch am PAL-Gerät mit 60 Hz; der Song spielte dann ein Fünftel zu schnell.

1. Öffne `examples/c64/music_demo`, baue mit **F7**, starte mit **F5** (VICE).
2. Ändere im Tracker eine Note, speichere, baue und starte neu.

### Atari (POKEY)

Das Projekt `examples/atari/music_demo` spielt `music/atune.vcsong` mit dem Original-Spieler des Raster Music Trackers:

```asm
.target atari8
.cpu sally { illegals = deny }

.org $2000
.include "lib/rmt.inc"

start:  rmt_start atune_rmt, 0      ; Modul + Startzeile (liefert die Instrumentgeschwindigkeit in A)
        rmt_vbi
done:   jmp done

rmt_vbi_code
.include "lib/rmtplayr.asm"         ; Player: Tabellen ab $30E0, Code ab $3400
.org $4000
.include "build/export/atune.asm"
.run start
```

Der Export für POKEY ist ein **RMT-Modul**, das der Spieler direkt versteht. Der Spieler hängt sich in den **Deferred VBI** des Betriebssystems ein, den Interrupt am Ende jedes Bildes. Die tiefste Note im POKEY-Tracker ist C-3.

## Häufige Stolpersteine

- **Der Song läuft zu schnell oder zu langsam:** Die Spielroutine muss einmal je Bild laufen, am besten per Raster-Interrupt (C64). Stimmt die **Bildrate** der Hörprobe mit PAL/NTSC überein?
- **Man hört nichts:** Im Tracker prüfen, ob eine Stimme auf **M** (stumm) steht und ob die Lautstärke deines Rechners nicht aus ist. Gibt es oben eine Fehlermeldung „Audio: …“, stimmt etwas mit der Audioausgabe nicht.
- **Eine Note klingt endlos:** Setze mit **1** ein „Note aus“ (`===`).
- **Der Klang weicht vom Emulator ab:** Die Hörprobe ist ein Synthesizer, nicht der echte Chip.

## Wie geht es weiter?

Für den MEGA65 gibt es einen eigenen, einfacheren Weg zur Musik: [PLAY-Tracker](kapitel:play-tracker). Nachschlagen: [Tracker](hilfe:tracker), [Hüllkurve](hilfe:envelope), [Export](hilfe:export).
