# PRALLBALL für den Commodore PET (4000er, 40 Spalten)

Breakout-Klon, Fassung für den PET mit BASIC 4 (4032) und 32 KB RAM. Schlichter als die anderen Fassungen: **keine Farbe, keine Sprites, kein Klangchip.**

## Spielen
Bauen mit `vcc build examples/pet/prallball`; starten mit der Diskette `build/disk.d64`:

```bash
xpet -model 4032 -8 build/disk.d64 -keybuf 'load"*",8\nrun\n'
```

| Taste | Aktion |
|---|---|
| Z oder , | Schläger links |
| X oder . | Schläger rechts |
| Leertaste | Ball abschießen / Titel verlassen |
| P | Pause |
| RUN/STOP | Ende (Neustart des Rechners) |

22 Levels (die Boss-Level der C64-Fassung entfallen), Steintypen wie in den anderen Fassungen: Stein a bis h (10 bis 80 Punkte, abwechselnd Vollblock und Raster), Silber (2 Treffer), Gold (3 Treffer), unzerstörbar, explosiv (`XX`, zerstört die acht Nachbarn), Überraschung (`??`: breiter Schläger, ein Leben mehr oder langsamerer Ball). Der Ball wird mit der Zeit schneller (acht Stufen). Bestenliste (10 Einträge) auf der Diskette in `PRALL.HS`.

**Nicht enthalten** (verglichen mit C64/C128): Mehrfach-Ball, Laser, fallende Kapseln, Musik, Titelanimation, Joystick.

## Technik
- **Bild:** 40 x 25 Zeichen. Gezeichnet wird mit den Viertelblock-Zeichen des PET-Zeichensatzes (2 x 2 Teilzellen je Zeichen = 80 x 50 Teilzellen); der Ball ist eine Teilzelle, der Schläger eine Reihe Teilzellen (untere Hälfte der Zeile 23). Die Zuordnung Maske → Bildschirmcode (`qtab`) ist gegen die Zeichen-ROM geprüft (`tools/run_tests.py`).
- **Zwei Schichten:** Bildschirm (Steine, Wände, Text) und Overlay (`$6C00`, Maske je Zelle für Ball und Schläger).
- **Interrupt:** Der Editor-Interrupt (Vektor `$0090`) wird eingehängt: Bildzähler, Tastaturpuffer leeren, Klang zählen; dann die alte Routine (Tastaturabfrage, Uhr).
- **Eingabe:** Der Editor (BASIC 4) legt den Code der gehaltenen Taste in `$97` ab (`$FF` = keine); der Puffer muss dafür leer sein (`$9E`). Dadurch ist die Abfrage unabhängig von der Tastaturbelegung (Grafik-/Schreibmaschinentastatur).
- **Klang:** nur Pieptöne über das Schieberegister der VIA (Leitung CB2, `$E848`/`$E84A`/`$E84B`).
- **Bestenliste:** Das KERNAL des PET hat keine `SETLFS`/`SETNAM`; die Dateiparameter stehen in der Nullseite. Außerdem sind `OPEN` (`$FFC0`) und `CLOSE` (`$FFC3`) der Sprungtabelle beim PET die BASIC-Anweisungen; benutzt werden `$F563` und `$F2E0` (BASIC 4). Dateien gibt es nur, wenn das Programm von Diskette gestartet wurde (Gerät in `$D4`).
- **Speicher:** Programm ab `$0401` (etwa 8,5 KB), Levelkarte `$6B00`, Overlay `$6C00` – 32 KB RAM nötig.

## Prüfung
`python3 tools/run_tests.py` (nach `vcc build`): einzelne Unterprogramme über den VICE-Text-Monitor (xpet, `-model 4032`), dazu Läufe der Testfassungen `AUTOPLAY` und `TESTLOSE` (Spielende, Namenseingabe, Speichern auf eine Diskettenkopie). `vcc test` kennt das Ziel `pet` noch nicht.

## Nicht geprüft
Klang (nur Register geprüft, nicht gehört), echte Tastatur (nur `$97` gesetzt), 8032 (80 Spalten, andere Zeichenbreite), PAL-/NTSC-Unterschiede (60 Hz im Test), BASIC 2 (2001, 3000er), echte Hardware und echtes Diskettenlaufwerk.
