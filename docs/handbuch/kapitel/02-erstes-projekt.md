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

**Ziel:** Du legst mit dem Dialog **Neues Projekt erstellen** ein Projekt für den C64 an, siehst dir das fertige Programm an, baust es und startest es im Emulator. Danach änderst du eine Zahl und siehst die Wirkung.

## Das brauchst du

- VintageCoder, gestartet (siehe [Installation und Ersteinrichtung](kapitel:installation)),
- den Emulator **VICE** für den C64 (`x64sc`), den VintageCoder auf der Startseite im Kasten **Toolchain und Hardware** als „bereit“ zeigt (sonst richtest du ihn mit dem Konfigurationsassistenten oder unter **Einstellungen ▸ Ziele und Hardware** ein),
- einen Ordner, in dem das neue Projekt entstehen soll, zum Beispiel einen Ordner „Retro“ in deinen Dokumenten.

Vorwissen über den C64 oder über Assembler brauchst du **nicht**.

## Schritt für Schritt: Projekt anlegen

1. Klicke auf der Startseite auf **Neues Projekt**. Du erreichst den Dialog auch über das Menü **Datei ▸ Neues Projekt …**, die Befehlspalette (**Cmd/Strg+K**) oder mit **Cmd/Strg+Alt+N**. Der Dialog heißt **Neues Projekt erstellen**.
2. Der Dialog hat drei Spalten. In der ersten Spalte wählst du das **Zielsystem**. Klicke auf **Commodore 64**. Darunter siehst du das **Hardware-Profil** mit der Anzeige „gültig“, den Prozessor 6510 und die Chips des C64.

![Der Dialog „Neues Projekt erstellen“ über der Startseite. Links die Liste „ZIELSYSTEM“ mit Commodore 64 (gewählt) bis MEGA65 und dem Hardware-Profil „gültig“, in der Mitte die „VORLAGE“ mit „Assembler-Programm“ (gewählt), „Steckmodul“ und „Leeres Projekt“, rechts die „PROJEKT-DETAILS“ mit Projektname, Speicherort, Videonorm, Standard-Emulator, Startziel, Diskette und Zeichensatz. Unten die Knöpfe „Abbrechen“ und „Projekt erstellen und öffnen“.](bild:neues-projekt-dialog "Der Dialog „Neues Projekt erstellen“ mit Zielsystem, Vorlage und Projekt-Details.")

3. In der zweiten Spalte wählst du die **Vorlage**. Oben rechts in der Spalte wählst du die Sprache: **Assembler** oder **BASIC**. Nimm für dieses Kapitel **Assembler** und dann die Vorlage **Assembler-Programm** („Kleines Assemblerprogramm (Farbwechsel), startbar mit F5“). Auf der Karte steht, welche Datei die Vorlage anlegt: `src/c64/main.asm`. Die weiteren Vorlagen heißen **Steckmodul** und **Leeres Projekt**.

4. In der dritten Spalte stehen die **Projekt-Details**:
   - **Projektname:** zum Beispiel `hallo`. Erlaubt sind Buchstaben, Ziffern, `_` und `-`. Bei anderen Zeichen färbt sich der Hinweis rot.
   - **Speicherort:** der Ordner, in dem VintageCoder einen Unterordner mit dem Projektnamen anlegt. Mit **Wählen …** suchst du ihn aus.
   - **Videonorm:** **PAL** (Europa) oder **NTSC**. Lass die Vorgabe stehen.
   - **Standard-Emulator** und **Startziel (F5)**: Hier ist ein Emulator vorgewählt. Lass es stehen.
   - **Diskette:** für dein erstes Projekt **keine**.
   - Die beiden Schalter für **README.md** und **Git-Repository** kannst du ausgeschaltet lassen.
   - Der Block **Dateistruktur** zeigt vorab, was angelegt wird.

5. Klicke unten auf **Projekt erstellen und öffnen**. Der Knopf ist erst aktiv, wenn Name und Vorlage gültig sind. Erscheint stattdessen eine rote Meldung, nennt sie den Grund; lies sie und korrigiere die Angabe.

Das Projekt ist angelegt und öffnet sich im Bereich **Code**. Das Ergebnis siehst du im Editor: die Datei `main.asm` ist offen, in der **Seitenleiste** (Überschrift **Projekt**) steht der Ordner `src/c64/`.

*[Bild folgt in E13: projekt-neu-ergebnis]*

> **Tipp:** Auf der Kommandozeile erreichst du dasselbe mit `vcc new hallo --target c64 --template asm-hello`. Mehr zum Dialog steht in der Hilfeseite [Startseite, Konfigurationsassistent und neues Projekt](hilfe:wizard).

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

1. Klicke in der Kopfzeile auf das Symbol **Bauen** oder drücke **F7**. VintageCoder speichert die Datei und übersetzt sie. In der Statusleiste unten steht danach „gebaut: 28 Bytes“. Ein Klick darauf öffnet die **Ablage** unter dem Editor mit dem Reiter **Ausgabe**. Dort zeigt eine Zeile „Fertig: 28 Bytes in …“, dass das Programm gebaut wurde.
2. Klicke auf das Symbol **Starten** oder drücke **F5**. VintageCoder baut bei Bedarf noch einmal und startet VICE mit deinem Programm. Nach einem kurzen Moment siehst du das Emulator-Fenster: Der Rahmen des C64-Bildschirms läuft in bunten Streifen durch die Farben.

*[Bild folgt in E13: bauen-knopf]*

3. Schließe das Emulator-Fenster, wenn du fertig bist, oder klicke in der Kopfzeile auf **Beenden** (das Symbol erscheint, solange der Emulator läuft). Beende den Emulator, bevor du erneut startest.

> **Info:** **F7** baut nur, **F5** baut und startet. Das Startziel (zum Beispiel „C64 · VICE x64sc“) steht links neben den Symbolen in der Kopfzeile und zeigt, wohin F5 startet: hier der Emulator VICE x64sc für den C64. Das Panel **Maschine** zeigt rechts dazu den Zustand und das Ergebnis des letzten Baus (Cmd/Strg+Alt+B blendet es ein und aus).

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

Du kannst das gleiche Projekt auch in BASIC beginnen. Wähle im Dialog in Spalte 2 statt „Assembler“ die Sprache **BASIC** und die Vorlage **BASIC**. Im Projekt liegt dann `src/c64/main.bas`:

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
- **F5 startet nichts.** Die Kopfzeile zeigt das Startziel. Steht der Emulator auf der Startseite im Kasten **Toolchain und Hardware** als „nicht gefunden“, installiere ihn (siehe [Installation und Ersteinrichtung](kapitel:installation)) oder trage seinen Pfad unter **Einstellungen ▸ Ziele und Hardware** ein. Den Grund eines fehlgeschlagenen Starts nennt der Reiter **Ausgabe** in der Ablage.
- **Der Rahmen wird nicht rot.** Auf dem C64 muss das Programm laufen: Es startet nach dem Laden von selbst; bleibt der Bildschirm bei „READY.“, tippe `RUN`.
- **Zwei Emulator-Fenster.** Schließe das alte Fenster, bevor du erneut startest.
- **Der Rahmen flackert zu wild.** Das ist das Vorlagenprogramm: Es schaltet die Farbe in jedem Bild weiter. Ersetze es wie oben durch das rote Rahmenprogramm.

## Wie geht es weiter?

Jetzt kennst du den Weg vom leeren Projekt zum laufenden Programm. Als Nächstes lernst du die Fenster kennen: [Die Oberfläche im Überblick](kapitel:oberflaeche). Danach folgen [Projekte und Dateien](kapitel:projekte) und der [Code-Editor](kapitel:editor).
