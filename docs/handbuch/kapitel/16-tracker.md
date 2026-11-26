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

## Wofür ist der Editor gedacht?

Im Tracker komponierst du die **Musik und die Klänge** deines Programms: Titelmelodie, Spielmusik, Soundeffekte (Schuss, Explosion, Sprung).

Typische Einsätze aus echten Retro-Projekten:

- **Eine Titelmelodie** mit drei Stimmen: Bass, Begleitung, Melodie.
- **Spielmusik in Schleife** aus wenigen Mustern, die sich wiederholen.
- **Soundeffekte** als kurze Muster mit Rauschen oder schnellen Tonsprüngen.

Im fertigen Programm landet der Song als **Datentabelle plus kleines Abspielprogramm**, das du bei jedem Bild (50- oder 60-mal pro Sekunde) aufrufst und das den Klangchip ansteuert. Der Editor erzeugt beides.

## Grundbegriffe ohne Vorwissen

- **Klangchip:** der Baustein, der Töne erzeugt (C64: **SID**, drei Stimmen; Atari: **POKEY**, vier Stimmen; MEGA65: **PLAY**-Befehl des BASIC 65).
- **Stimme / Kanal:** eine Tonspur, die der Chip gleichzeitig spielen kann.
- **Note, Oktave:** Tonhöhe; jede Oktave verdoppelt die Frequenz.
- **Muster (Pattern), Reihenfolge (Ablauf):** Bausteine eines Songs und die Liste, in der sie gespielt werden.
- **Instrument:** Wellenform und Hüllkurve eines Klangs.
- **Wellenform:** Dreieck (weich), Sägezahn (hell), Puls (kräftig, mit Pulsbreite), Rauschen (für Schlagwerk).
- **Hüllkurve (ADSR):** Verlauf der Lautstärke eines Tons: Anschwellen (Attack), Abfallen (Decay), Halten (Sustain), Ausklingen (Release).
- **Takt / Tempo:** hier die Zahl der Bilder je Zeile; das Feld in der Werkzeugleiste heißt **Tempo**.
- **Bildrate:** 50 Hz (PAL) oder 60 Hz (NTSC); das Abspielprogramm läuft einmal je Bild.

## Was ist ein Tracker?

Ein **Tracker** ist ein Musikprogramm, in dem du Noten nicht auf Notenlinien, sondern in **Tabellen** setzt. Jede **Spalte** ist eine **Stimme** (ein Kanal des Klangchips), jede **Zeile** ein Zeitschritt. Er stammt aus der Computermusik der 1980er und passt perfekt zu den Chips: Jeder Kanal spielt nur einen Ton zugleich.

Die wichtigsten Begriffe:

- **Zelle:** ein Eintrag aus Note und Instrument, zum Beispiel `C-4 01` (Note C in Oktave 4, Instrument 01). `---` bedeutet „nichts“ (im Raster als Punkte gezeigt), `===` „Note aus“.
- **Muster** (englisch *Pattern*): eine Tabelle mit einer festen Zahl von Zeilen, zum Beispiel 32. Ein Muster ist ein Baustein, etwa „vier Takte Bass“.
- **Reihenfolge** (englisch *Order*): die Liste, in welcher Reihenfolge die Muster gespielt werden. Weil du Muster wiederverwenden kannst, bleiben Songs klein.
- **Instrument:** die Klangfarbe. Ein Instrument bestimmt Wellenform und **Hüllkurve**, also wie ein Ton laut wird und wieder verklingt (siehe unten).
- **Tempo** (auch Takt genannt): die Zahl der **Bilder** je Zeile. Ein Bild ist ein Bildschirmdurchlauf, bei PAL 1/50 Sekunde. Tempo 6 heißt: Jede Zeile dauert sechs Bilder.

## Mein erster Song (am Beispiel `music_demo`)

1. **Beispiel öffnen:** Öffne auf der Startseite aus den Beispielprojekten `examples/c64/music_demo` (zum Ändern nimmst du **Kopieren**, dann hast du eine eigene Fassung). *Erwartet:* Im Projektbaum liegt `music/tune.vcsong`.
2. **Song öffnen:** Wähle oben den Hauptbereich **Musik**. Ist noch nichts offen, zeigt der Editor die **Songs** des Projekts als Knöpfe; klicke auf `music/tune.vcsong`. Ein Doppelklick auf die Datei im Projektbaum geht genauso. *Erwartet:* Links die Reihenfolge und die Instrumente, in der Mitte das Raster mit drei Stimmen, rechts das gewählte Instrument.
3. **Anhören:** Klicke auf **Song abspielen** (Pfeil in der Werkzeugleiste) oder drücke die **Leertaste**, solange der Cursor im Raster steht. *Erwartet:* Der Song spielt über den eingebauten Synthesizer, die laufende Zeile ist markiert, in den Stimmenköpfen schlagen die Pegel aus. **Wiedergabe stoppen** (oder noch einmal die Leertaste) hält ihn an.
4. **Eine Note setzen:** Zelle anklicken, die Taste links unten drücken (deutsche Tastatur: **Y**). *Erwartet:* In der Zelle steht eine Note, der Cursor rückt weiter, der Ton erklingt kurz.
5. **Klang ändern:** Wähle in der Liste **Instrumente** ein anderes Instrument. Rechts wechselst du die Klangform oder verschiebst die Regler. Ein Klick auf eine Taste der **Klaviatur** spielt den Ton mit dem Instrument vor. *Erwartet:* Der Klang ändert sich hörbar.
6. **Speichern:** **Cmd/Strg+S**. *Erwartet:* Das **●** („ungespeichert“) in der Fußzeile verschwindet.
7. **Bauen und starten:** **F7**, **F5**. *Erwartet:* Das Programm spielt den Song im Emulator.

Einen neuen Song legt der Dialog **Neue Datei** in dieser Version noch nicht an. Beginne mit einem Beispielprojekt oder einem vorhandenen Song.

## Aufbau der Oberfläche

Der Musik-Editor hat drei Bereiche nebeneinander. In schmalen Fenstern rutschen die Bereiche untereinander.

| Bereich | Inhalt |
|---|---|
| Links: **Song** | Name des Songs mit **Neues Muster** (Plus), die **Reihenfolge** der Muster (Anhängen, Setzen, Entfernen, Pfeile, **Wiederholung ab Eintrag**), die Liste der **Instrumente** und, wenn das Projekt mehrere Songs hat, die **Songs** |
| Mitte | Werkzeugleiste, Auswahl der sichtbaren **Stimmen**, die Muster als Knöpfe, das **Raster** (jede Spalte eine Stimme, jede Zeile ein Zeitschritt) und die Fußzeile |
| Rechts: **Instrument** | Name, Klangform und Regler des gewählten Instruments, die **Klaviatur** und die **Lautstärke** |

Die Werkzeugleiste enthält von links nach rechts: **Rückgängig**, **Wiederholen**, **Song abspielen**, **Muster ▶**, **Wiedergabe stoppen**, den Schalter **Mitscrollen**, die Felder **Tempo**, **Zeilen**, **Oktave** und **Schritt** (wie viele Zeilen der Cursor nach einer Note weiterrückt), **Halbton ↑**, **Halbton ↓**, **Instrument anwenden**, **Übernehmen …**, bei POKEY **RMT-Modul importieren …**, **Exportieren …** und **Speichern**.

![Der Reiter Musik mit der Datei tune.vcsong: links Reihenfolge (Muster 1, 2, 1, 2), Instrumente Puls, Bass und Snare, in der Mitte die Werkzeugleiste mit Mitscrollen, Tempo, Zeilen, Oktave und Schritt und das Raster mit drei Stimmen, rechts die Karten „Instrument 00 · Puls“, „Klaviatur“ und „Lautstärke“. Der Cursor steht auf Zeile 02, unten steht „ungespeichert“.](bild:tracker-ueberblick "Der Tracker mit Reihenfolge, Raster mit drei Stimmen und Instrumenten.")

Die Fußzeile zeigt den Namen der Datei, das aktuelle Muster, die Reihe und die Stimme des Cursors sowie einen Tastenhinweis; **●** meldet ungespeicherte Änderungen, bei laufender Wiedergabe steht dort der Eintrag und die Zeile.

Die Befehle **Tracker: Noten**, **Tracker: Instrumente** und **Tracker: Ablauf** (**Cmd/Strg+1**, **Cmd/Strg+2**, **Cmd/Strg+3**) setzen den Fokus auf das Raster, die Instrumentliste oder die Reihenfolge. Die Reihenfolge der Muster hat bei PLAY-Dateien keinen Platz: Dort steht links der Hinweis „PLAY-Dateien haben ein Muster und keine Reihenfolge.“

### Noten eingeben

Klicke eine Zelle an oder bewege dich mit den **Pfeiltasten** (**Bild auf/ab** springt acht Zeilen, **Pos1/Ende** zur ersten/letzten Zeile). Mit gedrückter Maustaste ziehst du eine Auswahl auf. Dann spielst du mit dem Computer-Keyboard wie auf einem Klavier:

```text
obere Reihe:   Q  2  W  3  E  R  5  T  6  Z  7  U  I      (eine Oktave höher, deutsche Tastatur)
untere Reihe:  Y  S  X  D  C  V  G  B  H  N  J  M         (aktuelle Oktave)
```

Die Tasten gelten nach der **Position**, nicht nach der Beschriftung: Auf einer amerikanischen oder französischen Tastatur liegen dieselben Töne an denselben Stellen. Das Tastaturlayout stellst du in den Einstellungen ein („Automatisch“ erkennt es).

Nach jeder Note rückt der Cursor um den **Schritt** weiter. Weitere Tasten, solange das Raster den Fokus hat:

| Taste | Wirkung |
|---|---|
| Entf oder Rücktaste | Zelle leeren |
| **1** | „Note aus“ (`===`) setzen |
| **[** und **]** | voriges/nächstes **Instrument** wählen |
| **,** und **.** | Oktave tiefer/höher (Punkt = höher) |
| Alt+↑ / Alt+↓ | Instrument der Zelle (oder der Auswahl) wechseln |
| Umschalt+Pfeile | mehrere Zellen auswählen |
| Leertaste | Song abspielen/stoppen |
| Umschalt+Leertaste | nur das gewählte Muster abspielen |
| Cmd/Strg+C, X, V, A | Block kopieren, ausschneiden, einfügen, alles auswählen |
| Cmd/Strg+S | speichern |
| Cmd/Strg+Z, Cmd/Strg+Umschalt+Z | rückgängig, wiederholen |

Die Einzeltasten (Noten, **1**, **[**, **]**, **,**, **.**) lassen sich in den Einstellungen abschalten („Einzeltasten der Editoren“); dann bleiben Pfeile, Leertaste, Entf und alle Kürzel mit Cmd/Strg.

**Halbton ↑** und **Halbton ↓** verschieben die ausgewählten Noten um einen Halbton. **Instrument anwenden** setzt das gewählte Instrument auf alle Noten der Auswahl.

In der **Klaviatur** rechts leuchten die Tasten der gerade klingenden Töne. Ein Klick auf eine Taste spielt den Ton mit dem gewählten Instrument vor, trägt aber nichts ein. Beschriftet ist die Computertaste, wenn die Tastenhinweise eingeschaltet sind.

![Der Reiter „Musik“ mit tune.vcsong. Links Reihenfolge und Instrumente, in der Mitte die Werkzeugleiste, die Stimmen 1 bis 3 und das Muster-Raster mit Noten wie C-4 00 und C-2 01; der Cursor steht in Stimme 3, Reihe 01. Rechts das Instrument „Puls“ mit Klangform, Reglern und der Klaviatur mit Computertasten.](bild:tracker-muster "Das Muster-Raster mit drei Stimmen und rechts die Klaviatur.")

### Muster und Reihenfolge

1. Das **Plus** neben dem Namen des Songs legt ein leeres **Neues Muster** an. Die Muster erscheinen über dem Raster als Knöpfe („Muster 1“, „Muster 2“ …); ein Klick schaltet das Raster um.
2. Die **Reihenfolge** links ist die Liste, in der die Muster gespielt werden. **Anhängen** fügt das gewählte Muster hinten an, **Setzen** ersetzt den gewählten Eintrag durch das gewählte Muster, **Entfernen** löscht den Eintrag, die Pfeile **Eintrag nach oben** und **Eintrag nach unten** sortieren um.
3. **Wiederholung ab Eintrag** nennt die Stelle, an der der Song nach dem Ende wieder beginnt. In der Liste markiert **↻** diese Stelle.
4. **Duplizieren** kopiert das gewählte Muster, **Löschen** entfernt es.
5. **Zeilen** (Zeilenzahl des Musters, 1 bis 128) und **Tempo** (1 bis 255) stehen in der Werkzeugleiste.

### Instrumente und Hüllkurve

Rechts siehst du das in der Liste **Instrumente** gewählte Instrument (Nummer und Name, Hexzahl wie in der Zelle). **Neues Instrument** legt eins an (beim PLAY-Tracker höchstens 10, sonst 32), **Duplizieren** kopiert das gewählte. Gelöscht werden kann nur das letzte, unbenutzte Instrument. Das Namensfeld über den Reglern benennt es um.

**SID (C64):**

- **Klangform:** Dreieck (weich), Säge (hell und schneidend), Rechteck (kräftig, die Breite stellst du mit **Pulsbreite** ein) und Rauschen (für Trommeln und Explosionen).
- **Hüllkurve** aus vier Reglern: **Anschlag** (Attack), **Abfall** (Decay), **Halten** (Sustain) und **Ausklang** (Release), zusammen ADSR. Sie beschreibt den Lautstärkeverlauf eines Tons: *Attack* ist die Zeit, in der er vom Nichts auf volle Lautstärke steigt; *Decay* die Zeit, in der er auf die Haltelautstärke *Sustain* fällt; *Release* die Zeit, in der er nach „Note aus“ verklingt. Jeder Wert läuft von 0 bis 15; für Sustain ist es der **Pegel**, kein Zeitwert.

![Der Reiter „Musik“ mit tune.vcsong. Rechts die Spalte „Instrument 00 · Puls“ mit den Klangformen (Rechteck gewählt), den Reglern Anschlag, Abfall, Halten, Ausklang und Pulsbreite, der Klaviatur und der Lautstärke. Links Reihenfolge und Instrumente, in der Mitte das Muster mit drei Stimmen.](bild:tracker-instrument "Die rechte Spalte des Trackers mit Klangform, Hüllkurve, Pulsbreite und Klaviatur.")

**POKEY (Atari):** Hier gibt es keine ADSR-Hüllkurve, sondern eine **Verzerrung** (Reiner Ton, Bass, Buzz oder Gritty) und die **Hüllkurve je Bild**: 16 senkrechte Regler von 0 bis 15, ein Wert je Bild. Das Feld **Hüllkurve wiederholt ab Bild** sagt, ab welchem Bild sich die Kurve wiederholt.

*[Bild folgt in E13: tracker-pokey]*

Die **Lautstärke** (0 bis 15) unter der Klaviatur gilt für den ganzen Song.

### Hören

- **Song abspielen** spielt ab dem gewählten Eintrag der Reihenfolge, **Muster ▶** nur das aktuelle Muster. **Wiedergabe stoppen** hält an. Die Leertaste macht dasselbe (mit Umschalt für das Muster).
- Das Anhören verwendet einen **eingebauten Synthesizer**, der die Chips nachahmt. Es klingt ähnlich, aber nicht identisch.
- Änderungen im Raster hörst du sofort, auch während der Song spielt. **Mitscrollen** lässt das Raster und die Reihenfolge mitlaufen.
- In jedem Stimmenkopf stehen **M** (stumm), **S** (solo) und ein Pegelbalken. Stumm und Solo wirken auch auf den Klang.
- **Viele Stimmen** (beim MEGA65 zwölf): Das Raster hat dann eine waagerechte Bildlaufleiste. Über dem Raster stehen die Knöpfe der **Stimmen**; mit ihnen blendest du Stimmen aus, die du gerade nicht brauchst. Sie verschwinden nur aus dem Raster, **spielen aber weiter** (wenn sie nicht stumm sind), und die Datei bleibt unverändert. **Alle einblenden** holt sie zurück. Eine Stimme bleibt immer sichtbar.

![Der Reiter „Musik“ mit tune.vcsong. Über dem Muster-Raster stehen die Knöpfe der Stimmen 1, 2 und 3 und „Alle einblenden“. In Stimme 1 steht in Reihe 01 die Note C-3, der Cursor ist in Reihe 02; rechts ist die Karte „Klaviatur“ ohne hervorgehobene Taste.](bild:tracker-hoerprobe "Der Tracker mit der Stimmen-Leiste über dem Raster und der Klaviatur rechts.")

## Hardware-Besonderheiten und Grenzen

| | C64 (SID) | Atari (POKEY) | MEGA65 (PLAY) |
|---|---|---|---|
| Stimmen | 3 | 4 | nach dem PLAY-Befehl |
| Klang | Dreieck, Sägezahn, Puls, Rauschen, ADSR | Verzerrung und Lautstärke je Bild | Hüllkurven 0 bis 9 |
| Instrumente | bis 32 | bis 32 | höchstens 10 |
| Muster | bis 128 Zeilen | wie SID | ein Muster, einmal gespielt |

Das Anhören verwendet einen **eingebauten Synthesizer** und klingt ähnlich, aber nicht identisch zum echten Chip. Prüfe wichtige Klänge im Emulator oder auf der Hardware.

## Nachbar-Editoren und Glossar

- Für Spielmusik und Effekte brauchst du die [Sprite-](kapitel:sprites) und [Bildschirm-Editoren](kapitel:bildschirm-kacheln) für das, was man sieht. Den MEGA65-PLAY-Tracker beschreibt das Kapitel [PLAY-Tracker](kapitel:play-tracker).
- Begriffe: Glossar (Datei `GLOSSAR.md`): SID, POKEY, Hüllkurve, Muster, Tracker.

## Songs von anderswo holen

- **Übernehmen …** (Werkzeugleiste) holt einen Song aus einem anderen System in dieses Projekt (**SID ↔ POKEY**), eine **GoatTracker-Datei** (`.sng`) oder eine **SID-Datei** (`.sid`). Du wählst die Datei und im Dialog **Song aus anderem System übernehmen** den **Namen der neuen Datei** und das **Zielsystem**. Bei SID-Dateien läuft das Abspielprogramm in einem eingebauten 6502-Kern; der Editor zeichnet die Registerwerte auf und baut daraus Noten und Instrumente (Felder **Subtune** und **Aufnahme in Sekunden**). Das Ergebnis ist eine Annäherung; Vibrato, Filter und Ähnliches fallen weg. Die Ausgabe nennt, was nicht übernommen wurde.
- **RMT-Modul importieren …** (nur POKEY) lädt ein Modul des **Raster Music Trackers**, des klassischen Atari-Trackers.

*[Bild folgt in E13: tracker-sid-pokey]*

## Einbinden im Programm

Wie bei allen Editoren entsteht der Export beim Bauen von selbst (Werkzeugleiste ▸ **Exportieren …** oder **Cmd/Strg+Umschalt+E** für den Dialog).

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
.include "music/tune.vcsong"
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
.include "music/atune.vcsong"
.run start
```

Der Export für POKEY ist ein **RMT-Modul**, das der Spieler direkt versteht. Der Spieler hängt sich in den **Deferred VBI** des Betriebssystems ein, den Interrupt am Ende jedes Bildes. Die tiefste Note im POKEY-Tracker ist C-3.

## Häufige Stolpersteine

- **Der Song läuft zu schnell oder zu langsam:** Die Spielroutine muss einmal je Bild laufen, am besten per Raster-Interrupt (C64).
- **Man hört nichts:** Im Tracker prüfen, ob eine Stimme auf **M** (stumm) steht und ob die Lautstärke deines Rechners nicht aus ist. Gibt es oben eine Fehlermeldung „Audio: …“, stimmt etwas mit der Audioausgabe nicht.
- **Eine Note klingt endlos:** Setze mit **1** ein „Note aus“ (`===`).
- **Der Klang weicht vom Emulator ab:** Das Anhören ist ein Synthesizer, nicht der echte Chip.

## Wie geht es weiter?

Für den MEGA65 gibt es einen eigenen, einfacheren Weg zur Musik: [PLAY-Tracker](kapitel:play-tracker). Nachschlagen: [Tracker](hilfe:tracker), [Hüllkurve](hilfe:envelope), [Export](hilfe:export).
