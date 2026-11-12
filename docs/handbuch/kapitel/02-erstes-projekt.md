---
id: erstes-projekt
titel: Dein erstes Projekt
reihenfolge: 2
stufe: Einsteiger
zeit: 15 Minuten
siehe_auch: [oberflaeche, assembler, bauen-starten]
referenz: [wizard, start, build]
---

# Dein erstes Projekt

**Ziel:** Du legst mit dem Projekt-Assistenten ein Projekt für den C64 an, siehst dir das fertige Programm an, baust es und startest es im Emulator. Danach änderst du eine Zahl und siehst die Wirkung.

## Das brauchst du

- VintageCoder, gestartet (siehe [Installation und Ersteinrichtung](kapitel:installation)),
- den Emulator **VICE** für den C64 (`x64sc`), den VintageCoder in den Einstellungen mit einem grünen Punkt zeigt,
- einen Ordner, in dem das neue Projekt entstehen soll, zum Beispiel einen Ordner „Retro“ in deinen Dokumenten.

Vorwissen über den C64 oder über Assembler brauchst du **nicht**.

## Schritt für Schritt: Projekt anlegen

1. Klicke auf der Willkommensseite auf **Neues Projekt**. Du erreichst den Assistenten auch über das Menü **Datei ▸ Neues Projekt …** oder mit **Cmd/Strg+Umschalt+N**.
2. Der Assistent hat drei Spalten. In der ersten Spalte wählst du das **Zielsystem**. Klicke auf **Commodore 64**. Darunter siehst du das Hardware-Profil: den Prozessor 6510 und die Chips des C64.

![Der Projekt-Assistent: links die Liste der sechs Zielsysteme, der C64 ist gewählt, darunter das Hardware-Profil mit der Anzeige „gültig“](bild:wizard-system "Spalte 1 des Assistenten: das Zielsystem wählen")

3. In der zweiten Spalte wählst du die **Vorlage**. Zuerst die Sprache: **Assembler (vcasm)** oder **BASIC**. Nimm für dieses Kapitel **Assembler (vcasm)** und dann die Vorlage **Assembler-Programm** („Kleines Assemblerprogramm (Farbwechsel), startbar mit F5“). Unter dem Namen steht, welche Datei die Vorlage anlegt: `src/c64/main.asm`.

![Der Assistent, Spalte 2: oben die Wahl zwischen Assembler und BASIC, darunter die Vorlagenkarte „Assembler-Programm“ mit der Dateiangabe src/c64/main.asm](bild:wizard-sprache "Spalte 2: erst die Sprache, dann die Vorlage")

4. In der dritten Spalte stehen die **Projekt-Details**:
   - **Projektname:** zum Beispiel `hallo`. Erlaubt sind Buchstaben, Ziffern, `_` und `-`. Bei anderen Zeichen färbt sich der Hinweis rot.
   - **Speicherort:** der Ordner, in dem VintageCoder einen Unterordner mit dem Projektnamen anlegt. Mit **Wählen …** suchst du ihn aus.
   - **Videonorm:** **PAL** (Europa) oder **NTSC**. Lass die Vorgabe stehen.
   - **Standard-Emulator** und **Startziel (F5)**: Hier steht, was du in den Einstellungen gewählt hast. Lass es stehen.
   - **Diskette:** für dein erstes Projekt **keine**.
   - Die beiden Schalter für **README.md** und **Git-Repository** kannst du ausgeschaltet lassen.
   - Der Block **Dateistruktur** zeigt vorab, was angelegt wird.

   ![Der Projekt-Assistent, Spalte 3 „Projekt-Details“ mit Projektname, Speicherort, Videonorm, Emulator, Startziel und Diskette](bild:wizard-details "Spalte 3: Name, Speicherort und die Vorschau der Dateien")

5. Klicke unten auf **Projekt erstellen und öffnen**. Der Knopf ist erst aktiv, wenn Name und Vorlage gültig sind. Erscheint stattdessen eine rote Meldung, nennt sie den Grund; lies sie und korrigiere die Angabe.

Das Projekt ist angelegt und öffnet sich. Das Ergebnis siehst du im Editor: die Datei `main.asm` ist offen, links im Projektbaum steht der Ordner `src/c64/`.

![Das frisch angelegte Projekt hallo: links der Projektbaum mit hallo.vcprj und src/c64/main.asm, in der Mitte der geöffnete Quelltext, oben die Befehlsleiste mit den Knöpfen Starten, Debuggen und Bauen](bild:projekt-neu-ergebnis "Das neue Projekt direkt nach dem Anlegen")

> **Tipp:** Auf der Kommandozeile erreichst du dasselbe mit `vcc new hallo --target c64 --template asm-hello`. Mehr zum Assistenten steht in der Hilfeseite [Willkommen und Projekt-Wizard](hilfe:wizard).

## Das Programm lesen

Die Vorlage legt dieses kleine Programm an (der Projektname steht in der ersten Zeile):

```asm
; hallo – C64: Rahmenfarbe läuft im Takt der Rasterzeile
.target c64
.cpu 6510 { illegals = deny }
.org $0801
.basicstub start

start:  sei
.frame: lda #$80                ; auf Rasterzeile $80 warten
.wait:  cmp $d012
        bne .wait
        inc $d020               ; Rahmenfarbe weiterschalten
        jmp .frame
```

Du musst das noch nicht verstehen; das Assembler-Kapitel erklärt alles. Hier nur der Sinn der Zeilen:

- Zeilen mit `;` sind **Kommentare** für Menschen. Der Assembler überliest sie.
- `.target c64` sagt: Dieses Programm ist für den C64.
- `.basicstub start` sorgt dafür, dass du das Programm auf dem C64 wie jedes andere mit `RUN` starten kannst.
- `start:` und `.frame:` sind **Marken** (*Labels*): Namen für eine Stelle im Programm.
- `inc $d020` zählt die Rahmenfarbe hoch. Das Programm wartet dabei immer auf eine bestimmte Bildschirmzeile und schaltet dann die Farbe weiter, deshalb flackert der Rahmen in Streifen.

## Schritt für Schritt: Bauen und starten

1. Klicke in der Befehlsleiste auf **Bauen** (blau) oder drücke **F7**. VintageCoder speichert die Datei und übersetzt sie. Unten öffnet sich das Fußpanel mit dem **Bauprotokoll**. Eine Zeile zeigt, dass das Programm gebaut wurde; es ist 28 Byte klein.
2. Klicke auf **Starten** (grün) oder drücke **F5**. VintageCoder baut bei Bedarf noch einmal und startet VICE mit deinem Programm. Nach einem kurzen Moment siehst du das Emulator-Fenster: Der Rahmen des C64-Bildschirms läuft in bunten Streifen durch die Farben.

![Die Befehlsleiste mit dem Feld „Auf: C64 PAL → VICE x64sc“ und den Knöpfen Starten (grün), Debuggen (bernstein) und Bauen (blau), darunter F5, F6 und F7 als Kürzel](bild:bauen-knopf "Starten (F5), Debuggen (F6) und Bauen (F7) in der Befehlsleiste")

3. Schließe das Emulator-Fenster, wenn du fertig bist, und beende den Emulator, bevor du erneut startest.

> **Info:** **F7** baut nur, **F5** baut und startet, **F6** baut und startet im Debugger (siehe [Der Debugger](kapitel:debugger)). Das Feld **Auf:** links neben den Knöpfen zeigt, wohin F5 startet: hier der Emulator VICE x64sc für den C64 im PAL-Modus.

## Schritt für Schritt: Eine Zahl ändern

Jetzt machst du dein erstes eigenes Programm daraus: Der Rahmen soll **eine** feste Farbe bekommen.

1. Markiere im Editor den gesamten Text (Cmd/Strg+A) und ersetze ihn durch:

```asm
; Rahmen rot färben und zurück zu BASIC
.target c64
.basicstub start

start:  lda #2              ; 2 = Farbe Rot
        sta $d020           ; Rahmenfarbe setzen
        rts                 ; zurück zu BASIC
```

2. Drücke **F5**. Im Emulator erscheint der C64-Startbildschirm mit **rotem** Rahmen. Tippe im Emulator `RUN` und Enter, falls das Programm nicht schon gelaufen ist: Der Rahmen wird rot, und der C64 meldet „READY.“.
3. Ändere die `2` in eine andere Zahl zwischen 0 und 15 (zum Beispiel `5` für Grün oder `6` für Blau), drücke wieder **F5** und sieh dir die Wirkung an.

> **Info:** Der C64 kennt 16 Farben, nummeriert von 0 (Schwarz) bis 15 (Hellgrau): 0 Schwarz, 1 Weiß, 2 Rot, 3 Cyan, 4 Purpur, 5 Grün, 6 Blau, 7 Gelb, 8 Orange, 9 Braun, 10 Hellrot, 11 Dunkelgrau, 12 Mittelgrau, 13 Hellgrün, 14 Hellblau, 15 Hellgrau.

Wie die einzelnen Zeilen arbeiten, erfährst du in [Assembler (VCASM) für Einsteiger](kapitel:assembler).

## Schritt für Schritt: Dasselbe in BASIC

Du kannst das gleiche Projekt auch in BASIC beginnen. Wähle im Assistenten in Spalte 2 statt „Assembler“ die Sprache **BASIC** und die Vorlage **BASIC (ohne Zeilennummern)**. Im Projekt liegt dann `src/c64/main.bas`:

```basic
REM hallo - natives BASIC V2 (C64), ohne Zeilennummern
PRINT CHR$(147);"HALLO VON hallo"
T=0
FOR I=1 TO 10
T=T+I
NEXT I
PRINT "SUMME=";T
@ende:
GOTO @ende
```

F5 baut und startet auch dieses Programm. Mehr dazu im Kapitel [BASIC in der IDE](kapitel:basic).

## Häufige Stolpersteine

- **„Projekt erstellen und öffnen“ ist grau.** Der Projektname enthält ein unerlaubtes Zeichen (erlaubt: Buchstaben, Ziffern, `_`, `-`), oder zur Kombination aus Zielsystem und Sprache gibt es keine Vorlage.
- **F5 startet nichts.** Die Befehlsleiste zeigt links den Emulator. Ist er in den Einstellungen rot, installiere ihn oder trage seinen Pfad ein (siehe [Installation und Ersteinrichtung](kapitel:installation)).
- **Der Rahmen wird nicht rot.** Auf dem C64 muss das Programm laufen: Es startet nach dem Laden von selbst; bleibt der Bildschirm bei „READY.“, tippe `RUN`.
- **Zwei Emulator-Fenster.** Schließe das alte Fenster, bevor du erneut startest.
- **Der Rahmen flackert zu wild.** Das ist das Vorlagenprogramm: Es schaltet die Farbe in jedem Bild weiter. Ersetze es wie oben durch das rote Rahmenprogramm.

## Wie geht es weiter?

Jetzt kennst du den Weg vom leeren Projekt zum laufenden Programm. Als Nächstes lernst du die Fenster kennen: [Die Oberfläche im Überblick](kapitel:oberflaeche). Danach folgen [Projekte und Dateien](kapitel:projekte) und der [Code-Editor](kapitel:editor).
