# PRALLBALL (Commodore Plus/4) – Prototyp

Ball-und-Schlaeger-Spiel in Assembler fuer den Commodore Plus/4 (PAL, 64 KB; **C16/116 werden nicht unterstuetzt**, sie haben nur 16 KB). Eigene Umsetzung fuer den TED.
Die Spiellogik (Ball, Steine, Extras, Bonus-/Bosslevel) ist ein 6502-Programm und der C64-Fassung aehnlich; Anzeige, Figuren,
Klang und Eingabe sind Plus/4-spezifisch. Der Quelltext ist ausfuehrlich kommentiert.

- **Bauen:** `vcc build .` (oder IDE F7); Start: `xplus4 -autostart build/prallball.prg` (oder `build/disk.d64`: Bestenliste auf dasselbe Laufwerk).
- **Bedienung:** Joystick (beide Anschluesse) oder Tastatur (Z links, X rechts, Leertaste Feuer, RUN/STOP Abbruch; die Taste P meldet die Eingabe-Bibliothek zwar als Pause, das Spiel wertet sie aber nicht aus); Namenseingabe: Tippen + RETURN (oder Joystick: hoch/runter Zeichen, links/rechts Cursor, Feuer).
- **Anzeige (TED):** Mehrfarb-Textmodus 40 x 25 mit eigenem Zeichensatz im RAM (`$F000`). Der TED hat **keine Sprites**: Ball (bis 3), Schlaeger (16/32/48 Punkte), Muenze und
  Laserschuesse sind **Zeichen-Sprites** (Compositor `tsp_render`): im Bildwechsel werden die Zellen unter den Sprites mit Pool-Zeichen (Codes 128..191) belegt, deren Bitmuster aus
  Hintergrund und vorverschobenem Spritebild entsteht (`(Hintergrund UND Maske) ODER Bild`, Streifen je Feinlage aus `tools/gen.py`). Der Hintergrund liegt zusaetzlich in Schattenkopien
  (`$C400` Zeichen, `$C800` Farben), aus denen die Zellen vor jedem Bild wiederhergestellt werden. Sprites sitzen auf 2 Bildpunkte genau (ein Mehrfarbpixel).
  Himmelsverlauf und Hintergrundbaender: **Rasterinterrupt setzt `$FF15` 22 mal je Bild**; Farben: Tabellen mit C64-Farbnummern, `tcol`/`tedbg` setzen sie in TED-Farbwerte (Helligkeit * 16 + Farbton) um.
- **Stand:** Titel (Blocklogo mit Farbwechsel, Top-5, Musik, **Ticker** mit pixelweisen Uebergaengen, **Lauftext** mit Feinscrollen, huepfende Baelle, Demo nach 30 s), 24 Levels
  (inkl. 2 Bonus, 2 Boss), Abprallwinkel nach Trefferstelle mit Bewegungseinfluss und Seitenaufprall, Extras als rotierende farbcodierte Muenzen (B, S, R, L, F, K, +, P, Mehrball, Laser, Durchschlag),
  Bestenliste mit Namenseingabe (30 s Auto-Bestaetigung), Speichern auf das Geraet, von dem gestartet wurde (KERNAL-Zelle `FA` = `$AE`).
- **Plus/4-Besonderheiten:** Das Spiel laeuft **ohne ROM** (`$FF3F`, eigener Interruptvektor in `$FFFE`); fuer die Bestenliste blendet `hs_io_on` das KERNAL kurz ein (TED-Interrupts aus,
  Systemmeldungen aus). Alle Puffer, die das KERNAL anfasst, liegen unter `$8000`. Muenzfarben: Mehrfarbzellen kennen nur die Farbtoene 0..7, daher ist "orange" ein gedaempftes Gelb ("dunkelgelb").
- **Klang (TED, 2 Stimmen, keine Huellkurven):** Titelmusik = Melodie (Stimme 1) + Bass (Stimme 2), Schlagzeug als kurzer Rauschstoss auf Stimme 2 (der Bass setzt aus), die dritte Notenspur
  (Arpeggio) entfaellt; Klangeffekte nehmen Stimme 2. Frequenztabelle fuer PAL, tiefe Toene um Oktaven nach oben gefaltet.
- **Noch nicht / ungeprueft:** Klang ungehoert (nur Aufbau und Register-Tests); Joystick ueber `$FF08` (Auswahl `%11111011`/`%11111101`, Feuer Bit 6 oder 7, wie im cc65-Treiber `plus4-stdjoy`; beide Anschluesse werden gelesen; nicht an echter Hardware geprueft); Tastatureingabe nur gegen die KERNAL-Dekodiertabelle geprueft, nicht im Emulator getippt; Paddle (am Plus/4 nicht vorgesehen); NTSC nicht getestet. Im ungünstigsten Fall
  (3 Baelle + Laser + Muenze) verpasst die Hauptschleife einzelne Bildwechsel (rund 3 %, Testfassung `TESTFPS`).
- **Bibliotheken:** `lib/{input,hiscore,rand,sound}.asm` (Vertrag: siehe die Kopfkommentare der Dateien). Zeropage: `$02-$0D` Zufall/Musik/Bestenliste, `$10-$8B` Spiel, `$8C-$8F` Eingabe,
  `$90-$AC` Zeichen-Sprites (das KERNAL benutzt `$90` und aufwaerts nur waehrend der Bestenliste).
- **Tests:** `python3 tools/gen_tests.py && vcc build . && vcc test . --backend vice` (93 Emulator-Tests in `xplus4` mit Trap `$7F00`, ROM-Abschaltung im Setup, plus statische Pruefungen:
  BCD-Punkte, Richtungstabelle, Schlaegerabprall, Steinkollision, Durchschlag, alle 24 Levels, Bestenliste mit CRC und Geraeteauswahl, Compositor mit unabhaengig berechneten Bitmustern,
  Klangeffekte und Musikschritte). Die Taktmessung (`cycles`) gibt es bei `xplus4` nicht.
- **Testfassungen (`-D`):** `AUTOPLAY` (Rechner spielt), `STARTLEVEL=n` (0-basiert), `TESTMULTI`, `TESTLASER`, `TESTLOSE` (verliert absichtlich: Spielende/Namenseingabe), `TESTCAP` (jeder Stein laesst eine Kapsel fallen), `NOTICKER` (kein Ticker in der Demo), `TESTFPS` (Zaehler
  in Zeile 24: Schleifen, Bildwechsel, verpasste Bilder). Bauen: `vcc asm prallball.asm --target plus4 --cpu 7501 -D AUTOPLAY=1 -o x.prg`.
- **Werkzeuge:** `tools/gen.py` (Richtungstabelle, Levels, Zeichen, Zeichen-Sprites), `tools/make_levels.py` (Levelentwuerfe), `tools/compose.py` (Titelmusik, gleiche Noten wie C64).

## Einstellungen und Demo-Spieler (neu)

Am Anfang von `prallball.asm` steht der Block **EINSTELLUNGEN** (auch per `-D NAME=wert` beim Uebersetzen setzbar):
`EXTRA_RATE` (40: Chance in 256 je zerstoertem Stein auf Kapseln; etwa doppelt so viele wie vorher), `DEMO_REAKTION_MIN`/`DEMO_REAKTION_MAX` (4/12 Bilder Reaktionszeit nach einem Richtungswechsel des Balls), `DEMO_FEHLER_RATE`, `DEMO_FEHLER_FERN`, `DEMO_UNGENAU`, `DEMO_VERFEHLEN_RATE`, `DEMO_EXTRA_LUST`, `DEMO_EXTRA_FERN`.
Der Demo-Spieler wirkt dadurch menschlicher (Reaktionszeit, kurze Fehler, ungenaues Treffen, seltenes absichtliches Verfehlen) und faengt gute fallende Kapseln, solange der Ball weit weg ist.
Testfassung `-D AUTOPLAY=1 -D TESTDEMO=1` zaehlt Ereignisse des Demo-Spielers im RAM (siehe Kopfkommentar).

**Winkelzufall (neu):** Beim Abschlag vom Schlaeger wird der Abprallwinkel mit der Chance `WINKEL_ZUFALL` (Standard 64 von 256, also etwa jeder vierte Treffer) um genau eine Zone (17 Grad) nach links oder rechts verschoben. Das gilt fuer den Menschen und den Demo-Spieler; der Ball bleibt immer in den acht Zonen, an den Raendern wird nicht darueber hinaus geschoben. Es verhindert endlose, immer gleiche Bahnen. `0` schaltet den Zufall ab (`-D WINKEL_ZUFALL=0`). Mit `-D AUTOPLAY=1` spielt der Rechner jetzt alle Levels ohne Ende (bisher nur die ersten fuenf); `-D TESTDEMO=1` zaehlt zusaetzlich die Winkelverschiebungen, die Schlaegertreffer, die geschafften Levels, die zerstoerten Steine und die durch das Zeitlimit beendeten Levels (Adressen im Kopf von `prallball.asm`).

**Demo-Zeitlimit (neu):** Trifft der Demo-Spieler `DEMO_STEIN_LIMIT` Sekunden (Standard 40) lang keinen Stein, gilt das Level als geschafft und das naechste beginnt; so haelt ein unerreichbarer oder zaeher letzter Stein die Demo nicht endlos auf. Nur in der Demo (und mit `AUTOPLAY`), nie im Spiel fuer Menschen; `0` schaltet es ab.
