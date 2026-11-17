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
- Lautsprecher oder Kopfhörer. Das Anhören läuft in VintageCoder selbst, du brauchst dafür keinen Emulator.

## Wofür ist das Studio gedacht?

Im Tracker komponierst du die **Musik und die Klänge** deines Programms: Titelmelodie, Spielmusik, Soundeffekte (Schuss, Explosion, Sprung).

Typische Einsätze aus echten Retro-Projekten:

- **Eine Titelmelodie** mit drei Stimmen: Bass, Begleitung, Melodie.
- **Spielmusik in Schleife** aus wenigen Mustern, die sich wiederholen.
- **Soundeffekte** als kurze Muster mit Rauschen oder schnellen Tonsprüngen.

Im fertigen Programm landet der Song als **Datentabelle plus kleines Abspielprogramm**, das du bei jedem Bild (50- oder 60-mal pro Sekunde) aufrufst und das den Klangchip ansteuert. Das Studio erzeugt beides.

## Grundbegriffe ohne Vorwissen

- **Klangchip:** der Baustein, der Töne erzeugt (C64: **SID**, drei Stimmen; Atari: **POKEY**, vier Stimmen; MEGA65: **PLAY**-Befehl des BASIC 65).
- **Stimme / Kanal:** eine Tonspur, die der Chip gleichzeitig spielen kann.
- **Note, Oktave:** Tonhöhe; jede Oktave verdoppelt die Frequenz.
- **Muster (Pattern), Reihenfolge (Ablauf):** Bausteine eines Songs und die Liste, in der sie gespielt werden.
- **Instrument:** Wellenform und Hüllkurve eines Klangs.
- **Wellenform:** Dreieck (weich), Sägezahn (hell), Puls (kräftig, mit Pulsbreite), Rauschen (für Schlagwerk).
- **Hüllkurve (ADSR):** Verlauf der Lautstärke eines Tons: Anschwellen (Attack), Abfallen (Decay), Halten (Sustain), Ausklingen (Release).
- **Takt:** hier die Zahl der Bilder je Zeile (Tempo).
- **Bildrate:** 50 Hz (PAL) oder 60 Hz (NTSC); das Abspielprogramm läuft einmal je Bild.

## Was ist ein Tracker?

Ein **Tracker** ist ein Musikprogramm, in dem du Noten nicht auf Notenlinien, sondern in **Tabellen** setzt. Jede **Spalte** ist eine **Stimme** (ein Kanal des Klangchips), jede **Zeile** ein Zeitschritt. Er stammt aus der Computermusik der 1980er und passt perfekt zu den Chips: Jeder Kanal spielt nur einen Ton zugleich.

Die wichtigsten Begriffe:

- **Zelle:** ein Eintrag aus Note und Instrument, zum Beispiel `C-4 01` (Note C in Oktave 4, Instrument 01). `---` bedeutet „nichts“, `===` „Note aus“.
- **Muster** (englisch *Pattern*): eine Tabelle mit einer festen Zahl von Zeilen, zum Beispiel 32. Ein Muster ist ein Baustein, etwa „vier Takte Bass“.
- **Reihenfolge** (englisch *Order*): die Liste, in welcher Reihenfolge die Muster gespielt werden. Weil du Muster wiederverwenden kannst, bleiben Songs klein.
- **Instrument:** die Klangfarbe. Ein Instrument bestimmt Wellenform und **Hüllkurve**, also wie ein Ton laut wird und wieder verklingt (siehe unten).
- **Takt** (im Tracker die Geschwindigkeit): die Zahl der **Bilder** je Zeile. Ein Bild ist ein Bildschirmdurchlauf, bei PAL 1/50 Sekunde. Takt 6 heißt: Jede Zeile dauert sechs Bilder.

## Mein erster Song (am Beispiel `music_demo`)

1. **Beispiel öffnen:** Startseite ▸ Beispiele ▸ `examples/c64/music_demo`. *Erwartet:* Im Baum liegt `music/tune.vcsong`.
2. **Song öffnen:** Doppelklick. *Erwartet:* Der Tracker öffnet im Modus **Noten**: Raster mit drei Stimmen, rechts Karten.
3. **Anhören:** **Leertaste**. *Erwartet:* Der Song spielt (eingebauter Synthesizer), die laufende Zeile bewegt sich, die Pegel schlagen aus. Noch einmal **Leertaste** stoppt.
4. **Eine Note setzen:** Zelle anklicken, die Taste links unten drücken (deutsche Tastatur: **Y**). *Erwartet:* In der Zelle steht eine Note, der Cursor rückt eine Zeile weiter, der Ton erklingt kurz.
5. **Klang ändern:** **Cmd/Strg+2** (Modus **Instrumente**), anderes Instrument wählen, Wellenform wechseln; **Leertaste** spielt das Instrument probe. *Erwartet:* Der Klang ändert sich hörbar.
6. **Zurück zu den Noten:** **Cmd/Strg+1**; **Speichern** mit **Cmd/Strg+S**.
7. **Bauen und starten:** **F7**, **F5**. *Erwartet:* Das Programm spielt den Song im Emulator.

Ein neuer Song: **＋** in der linken Spalte ▸ im Dialog **Neue Datei** die Art **Tracker**, Profil **C64 SID-Song (3 Stimmen)** oder **Atari POKEY-Song (4 Stimmen)**, Namen `tune`, **OK**.

## Aufbau der Oberfläche

Der Tracker hat drei **Modi** (Umschalter in der Leiste oder **Cmd/Strg+1/2/3**): **Noten**, **Instrumente**, **Ablauf**.

| Zone | Inhalt |
|---|---|
| Kopfzeile | Art **TRK**, Name, **●**, Chip und Stimmen, aktuelles Muster, Rückgängig/Wiederholen, **Exportieren …**, **Speichern** |
| Leiste | genau eine Reihe: Modus, Abspielen (**▶ Song**, **▶ Muster**), **Oktave**, **Schritt** (wie viele Zeilen der Cursor nach einer Note weiterrückt), **Mitscrollen** |
| Liste | die **Muster** (Nummer, wie oft im Ablauf); Instrumente im Modus Instrumente |
| Fläche | im Modus Noten das Raster: jede Spalte eine Stimme, jede Zeile ein Zeitschritt; die laufende Zeile ist markiert |
| Karten | **Werkzeug**, **Klaviatur** (zeigt klingende Töne, Mausklick spielt), **Kanäle** (je Stimme **Stumm**, **Solo** und **Zeigen**), **Pegel**, **Block** (Auswahl transponieren, Instrument setzen, Kopieren/Einfügen), **Dokument** (Chip, Zeilenzahl, Takt) |
| Fußzeile | Muster, Zeile, Kanal und Zellinhalt; rechts **? Tasten** |

Im Modus **Ablauf** liegen die Muster als **Kartenkette** nebeneinander: Du hängst Muster an (**+**), schiebst Einträge nach vorn oder hinten (**←**, **→**), setzt den gewählten Eintrag auf das gewählte Muster (**Setzen**) oder entfernst ihn (**−**). PLAY-Dateien kennen keinen Ablauf: dort wird ein Muster einmal gespielt.

### Noten eingeben

Klicke eine Zelle an oder bewege dich mit den **Pfeiltasten** (**Bild auf/ab** springt acht Zeilen, **Pos1/Ende** zur ersten/letzten Zeile). Dann spielst du mit dem Computer-Keyboard wie auf einem Klavier:

```text
obere Reihe:   Q  2  W  3  E  R  5  T  6  Z  7  U  I      (eine Oktave höher, deutsche Tastatur)
untere Reihe:  Y  S  X  D  C  V  G  B  H  N  J  M         (aktuelle Oktave)
```

Die Tasten gelten nach der **Position**, nicht nach der Beschriftung: Auf einer amerikanischen oder französischen Tastatur liegen dieselben Töne an denselben Stellen. Das Tastaturlayout stellst du in den Einstellungen ein („Automatisch“ erkennt es).

Nach jeder Note rückt der Cursor eine Zeile weiter. Weitere Tasten:

| Taste | Wirkung |
|---|---|
| Entf oder Rücktaste | Zelle leeren |
| **1** | „Note aus“ (`===`) setzen |
| **[** und **]** | voriges/nächstes **Instrument** wählen |
| **,** und **.** | Oktave tiefer/höher (Punkt = höher) |
| Alt+↑ / Alt+↓ | Instrument der Zelle (oder der Auswahl) wechseln, die Note spielt zur Probe |
| Umschalt+Pfeile | mehrere Zellen auswählen |
| Leertaste | Song abspielen/stoppen |
| Umschalt+Leertaste | nur das gewählte Muster abspielen |
| Cmd/Strg+C, X, V | Block kopieren, ausschneiden, einfügen |
| Cmd/Strg+1, 2, 3 | Modus Noten, Instrumente, Ablauf |
| Strg/Cmd+S | speichern |
| Strg/Cmd+Z, Strg/Cmd+Umschalt+Z | rückgängig, wiederholen |

In der Karte **Klaviatur** zeigt eine Klaviertastatur, welche Töne gerade klingen. Ein Mausklick auf eine Taste trägt den Ton mit dem gewählten Instrument an der Cursorzelle ein und rückt den Cursor vor.

![Tracker-Studio im Modus Noten: links die Muster, in der Mitte das Muster mit drei Stimmen, rechts die Karten Werkzeug (Instrumente), Klaviatur und Kanäle](bild:tracker-muster "Ein Muster mit Noten in drei Stimmen: jede Spalte eine Stimme, jede Zeile ein Zeitschritt")

### Muster und Reihenfolge

1. In der **Liste** links (Modus Noten) legt das **+** über der Liste ein leeres Muster an, **Dupl.** kopiert das gewählte, **Löschen** entfernt es. Ein Klick auf ein Muster schaltet das Raster um.
2. Im Modus **Ablauf** (**Cmd/Strg+3**) stehen die Muster in der Abspielreihenfolge als Kartenkette. **+** hängt das gewählte Muster hinten an, **←** und **→** sortieren um, **Setzen** ersetzt einen Eintrag durch das gewählte Muster, **−** entfernt den gewählten Eintrag.
3. Die Wiederholung (ab welcher Position der Song nach dem Ende wieder beginnt) stellst du im Modus **Ablauf** neben den Knöpfen ein (**Wiederholung ab**, mit **−** und **+**). In der Kette markiert **↺** die Stelle.
4. Die **Zeilenzahl** des Musters (mit **−** und **+** in Achterschritten, höchstens 128) und der **Takt** stehen in der Karte **Dokument**.

### Instrumente und Hüllkurve

Im Modus **Instrumente** (**Cmd/Strg+2**) wählst du ein **Instrument** aus der Liste (Nummer und Name, Hexzahl wie in der Zelle). **Neu** legt ein Instrument an (beim PLAY-Tracker höchstens 10, sonst 32), **Dupl.** kopiert das gewählte; gelöscht werden kann nur das letzte, unbenutzte. Der Knopf **▶** in jeder Zeile spielt das Instrument vor (Vorhören), ebenso die **Leertaste**. Die Einstellungen des Instruments siehst du in der Mitte.

**SID (C64):**

- **Wellenform:** Dreieck (weich), Sägezahn (hell und schneidend), Rechteck/Puls (kräftig, die Breite stellst du mit **Pulsbreite** ein) und Rauschen (für Trommeln und Explosionen).
- **Hüllkurve** aus vier Werten, **A**ttack, **D**ecay, **S**ustain, **R**elease (englisch ADSR). Sie beschreibt den Lautstärkeverlauf eines Tons: *Attack* ist die Zeit, in der er vom Nichts auf volle Lautstärke steigt; *Decay* die Zeit, in der er auf die Haltelautstärke *Sustain* fällt; *Release* die Zeit, in der er nach „Note aus“ verklingt. Jeder Wert läuft von 0 bis 15; für Sustain ist es der **Pegel**, kein Zeitwert.

![Tracker-Studio im Modus Instrumente: links die Instrumentliste, in der Mitte die Hüllkurve, darunter Wellenform und Regler für Pulsbreite, Attack, Decay, Sustain und Release](bild:tracker-instrument "Ein SID-Instrument: Wellenform, Pulsbreite und die vier Hüllkurvenwerte")

Über den Reglern zeichnet der Editor die **Hüllkurve** als Kurve. **▶** (Vorhören) spielt das Instrument einmal, und zwar die Note C der gewählten Oktave.

![Hüllkurve eines Instruments im Modus Instrumente: die Kurve mit Attack-Spitze, Decay und Release über den Reglern.](bild:tracker-huellkurve "Die Hüllkurve: ansteigend bis zur Spitze (Attack), fallend auf das Haltepegel (Decay), nach Note aus ausklingend (Release)")

**POKEY (Atari):** Hier gibt es keine ADSR-Hüllkurve, sondern eine **Verzerrung** (Reiner Ton, Rauschen, Buzz, Gritty oder Bass) und **Lautstärke je Bild**: 16 Werte von 0 bis 15, ein Wert je Bild. Das Feld **Wiederholung ab** sagt, ab welchem Schritt die Kurve sich wiederholt.

![Ein POKEY-Instrument: die Verzerrung (Reiner Ton, Rauschen, Buzz, Gritty, Bass) und 16 Lautstärkewerte je Bild als Regler; der Strich „ab hier“ zeigt die Wiederholung.](bild:tracker-pokey "POKEY-Instrumente haben Verzerrung und eine 16-Schritte-Lautstärkekurve")

### Hören

- **▶ Song** spielt ab dem gewählten Eintrag der Reihenfolge, **▶ Muster** nur das aktuelle Muster. Der Knopf wird zu **■ Stopp**. Die Leertaste macht dasselbe.
- Das Anhören verwendet einen **eingebauten Synthesizer**, der die Chips nachahmt. Es klingt ähnlich, aber nicht identisch.
- Ändert sich der Song im Raster, während er spielt, hörst du die Änderung sofort. **Mitscrollen** in der Leiste lässt das Raster mitlaufen; Handscrollen beendet das, ein Klick schaltet es wieder ein.
- In der Karte **Kanäle** stehen je Stimme die Schalter **Stumm**, **Solo** und **Zeigen**; die Karte **Pegel** zeigt die Lautstärke je Stimme.
- **Viele Stimmen** (beim MEGA65 zwölf): Das Raster hat dann eine waagerechte Bildlaufleiste, Kanalköpfe und Zeilennummern bleiben beim Scrollen stehen, und der Cursor scrollt von selbst ins Bild. Mit **Zeigen** blendest du Stimmen aus, die du gerade nicht brauchst: Sie verschwinden nur aus dem Raster, **spielen aber weiter** (wenn sie nicht stumm sind), und die Datei bleibt unverändert. Der Cursor springt über ausgeblendete Stimmen hinweg; eine Stimme bleibt immer sichtbar. **Alle Kanäle einblenden** (Befehlspalette und Rechtsklick auf die Fläche) holt sie zurück.

![Der Tracker im Modus Noten; in der Karte „Klaviatur“ sind die gerade klingenden Tasten hervorgehoben.](bild:tracker-hoerprobe "Die Karte „Klaviatur“ zeigt die gerade klingenden Töne (Beispielansicht)")

## Hardware-Besonderheiten und Grenzen

| | C64 (SID) | Atari (POKEY) | MEGA65 (PLAY) |
|---|---|---|---|
| Stimmen | 3 | 4 | nach dem PLAY-Befehl |
| Klang | Dreieck, Sägezahn, Puls, Rauschen, ADSR | Verzerrung und Lautstärke je Bild | Hüllkurven 0 bis 9 |
| Instrumente | bis 32 | bis 32 | höchstens 10 |
| Muster | bis 128 Zeilen | wie SID | ein Muster, einmal gespielt |

Das Anhören verwendet einen **eingebauten Synthesizer** und klingt ähnlich, aber nicht identisch zum echten Chip. Prüfe wichtige Klänge im Emulator oder auf der Hardware.

## Nachbar-Studios und Glossar

- Für Spielmusik und Effekte brauchst du die [Sprite-](kapitel:sprites) und [Bildschirm-Studios](kapitel:bildschirm-kacheln) für das, was man sieht. Den MEGA65-PLAY-Tracker beschreibt das Kapitel [PLAY-Tracker](kapitel:play-tracker).
- Begriffe: Glossar (Datei `GLOSSAR.md`): SID, POKEY, Hüllkurve, Muster, Tracker.

## Songs von anderswo holen

- **Konvertieren von …** holt einen Song aus einem anderen System in dieses Projekt (**SID ↔ POKEY**), eine **GoatTracker-Datei** (`.sng`) oder eine **SID-Datei** (`.sid`). Bei SID-Dateien läuft das Abspielprogramm in einem eingebauten 6502-Kern; der Editor zeichnet die Registerwerte auf und baut daraus Noten und Instrumente (Dialog: **Subtune** und **Länge in Sekunden**). Das Ergebnis ist eine Annäherung; Vibrato, Filter und Ähnliches fallen weg. Die Ausgabe nennt, was nicht übernommen wurde.
- **RMT importieren …** (nur POKEY) lädt ein Modul des **Raster Music Trackers**, des klassischen Atari-Trackers.


![Der Dialog „Song aus anderem System übernehmen“ mit Namensfeld, Quelle (POKEY (Atari) → SID (C64)) und den Knöpfen „OK“ und „Abbrechen“.](bild:tracker-sid-pokey "Ein Song aus einem anderen System wird für den Chip dieses Trackers umgewandelt")

## Einbinden im Programm

Wie bei allen Studios entsteht der Export beim Bauen von selbst (Kopfzeile ▸ **Exportieren …** oder **Cmd/Strg+Umschalt+E** für den Dialog).

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

- **Der Song läuft zu schnell oder zu langsam:** Die Spielroutine muss einmal je Bild laufen, am besten per Raster-Interrupt (C64).
- **Man hört nichts:** Im Tracker prüfen, ob eine Stimme auf **Stumm** steht und ob die Lautstärke deines Rechners nicht aus ist. Gibt es oben eine Fehlermeldung „Audio: …“, stimmt etwas mit der Audioausgabe nicht.
- **Eine Note klingt endlos:** Setze mit **1** ein „Note aus“ (`===`).
- **Der Klang weicht vom Emulator ab:** Das Anhören ist ein Synthesizer, nicht der echte Chip.

## Wie geht es weiter?

Für den MEGA65 gibt es einen eigenen, einfacheren Weg zur Musik: [PLAY-Tracker](kapitel:play-tracker). Nachschlagen: [Tracker](hilfe:tracker), [Hüllkurve](hilfe:envelope), [Export](hilfe:export).
