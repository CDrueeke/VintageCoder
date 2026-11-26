# PRALLBALL (Commodore 128, VIC-II-Fassung) – Prototyp

Ball-und-Schlaeger-Spiel in Assembler fuer den C128 im **40-Spalten-Modus (VIC-II)**, PAL. Die Technik entspricht der C64-Fassung (Mehrfarb-Textmodus mit eigenem Zeichensatz,
Hardware-Sprites, Raster-Farbbaender, SID); eigener Code, weil Speicherverwaltung (MMU), Start und Dateizugriff anders sind. Eine zusaetzliche **VDC-Fassung (80 Spalten)** folgt in
`examples/c128/prallball_vdc`. Der Quelltext ist ausfuehrlich kommentiert.

- **Bauen:** `vcc build .` (oder IDE F7); Start: `x128 -40col -autostart build/prallball.prg` (oder `build/disk.d64`: Bestenliste auf dasselbe Laufwerk). Im 40-Spalten-Modus starten
  (Taste 40/80 DISPLAY bzw. `-40col`), sonst zeigt der Monitor nur den VDC-Schirm.
- **Bedienung:** Joystick Port 2 oder Tastatur (Z links, X rechts, Leertaste Feuer, RUN/STOP Abbruch; die Taste P meldet die Eingabebibliothek als Pause, das Spiel wertet sie nicht aus); Namenseingabe: Tippen + RETURN (oder Joystick hoch/runter/links/rechts bzw. Q/A/Z/X, Feuer bestaetigt).
- **Speicher:** Das Spiel stellt die MMU-Konfiguration `$3E` ein (**alles RAM der Bank 0 plus E/A**, keine Systemroms; eigener Interruptvektor in `$FFFE`). Programm ab `$1C01` (bis
  `$47DF`, danach Luecke bis `$7FFF`), **VIC-Bank 2** (`$8000-$BFFF`): Sprite-Daten `$8000` (Zeiger `$00-$0A`), Bildschirm `$8400`, Zeichensatz `$8800` (aus dem Zeichen-ROM kopiert, Konfiguration `$01`),
  Levels/Lied ab `$9400` (der VIC sieht bei `$9000-$9FFF` nur das Zeichen-ROM, dort darf nichts Grafisches liegen), Steinkarte `$C000-$C0D7`, Ball-/Ticker-Variablen `$C100-$C17F`. 1 MHz (die 2-MHz-Betriebsart
  stoert den VIC-II und bleibt aus).
- **Bestenliste:** Datei `PRALL.HS` (127 Byte, Format wie C64) auf dem Laufwerk, von dem gestartet wurde (`$BA`, beim Start gemerkt). Fuer die KERNAL-Aufrufe schaltet `hs_io_on` kurz auf die
  Standardkonfiguration `$00` (Systemroms), schaltet den VIC-Interrupt ab, unterdrueckt die Systemmeldungen und waehlt per `SETBNK` die Bank 0; **die Umschaltroutinen und `copy_font`
  liegen darum unter `$4000`** (dort ist immer RAM).
- **Stand:** wie C64: Titel (Logo, Top-5, Musik, Ticker, Lauftext, huepfende Baelle, Demo nach 30 s), 24 Levels (Bonus/Boss), Abprallwinkel nach Trefferstelle mit Bewegungseinfluss und
  Seitenaufprall, Extras als rotierende Muenzen (Mehrball, Laser, Durchschlag), Bestenliste mit Namenseingabe. Geprueft in `x128` (Titel, Demo, Spiel, Speichern/Laden der Bestenliste).
- **Noch nicht / ungeprueft:** 2-MHz-Betrieb ausserhalb des sichtbaren Bildes (nicht noetig, bewusst weggelassen); Klang ungehoert (SID wie am C64, nur Aufbau geprueft); Tastatur und
  Joystick nicht im Emulator getippt (Abfrage wie C64: CIA1); die zusaetzlichen C128-Tasten (`$D02F`) werden nicht benutzt; NTSC nicht getestet.
- **Bibliotheken:** `lib/{input,hiscore,rand,sound}.asm` (Vertrag: siehe die Kopfkommentare der Dateien; Zeropage: Zufall `$02`, Musik `$04`, Bestenliste `$08`, Spiel `$10-$8B`, Eingabe `$8C`, das KERNAL benutzt `$90` und aufwaerts).
- **Tests:** `python3 tools/gen_tests.py && vcc build . && vcc test . --backend vice` (x128, Trap `$BF00`, im Setup `"$ff00": 62` = MMU-Konfiguration `$3E`: 68 Emulator-Tests und statische Pruefungen:
  BCD-Punkte, Richtungstabelle, Schlaegerabprall, Steinkollision, Durchschlag, alle 24 Levels, Bestenliste, Zeichensatzkopie, VIC-Einrichtung, Bildschirm-Hilfen).
- **Testfassungen (`-D`):** `AUTOPLAY`, `STARTLEVEL=n` (0-basiert), `TESTMULTI`, `TESTLASER`, `TESTLOSE`, `TESTFPS`, `TESTCAP`, `NOTICKER` (Einzelheiten im Dateikopf von `prallball.asm`). Bauen: `vcc asm prallball.asm --target c128 --cpu 8502 -D AUTOPLAY=1 -o x.prg`.
- **Werkzeuge:** `tools/gen.py`, `tools/make_levels.py`, `tools/compose.py` (wie C64).

## Einstellungen und Demo-Spieler (neu)

Am Anfang von `prallball.asm` steht der Block **EINSTELLUNGEN** (auch per `-D NAME=wert` beim Uebersetzen setzbar):
`EXTRA_RATE` (40: Chance in 256 je zerstoertem Stein auf Kapseln; etwa doppelt so viele wie vorher), `DEMO_REAKTION_MIN`/`DEMO_REAKTION_MAX` (4/12 Bilder Reaktionszeit nach einem Richtungswechsel des Balls), `DEMO_FEHLER_RATE`, `DEMO_FEHLER_FERN`, `DEMO_UNGENAU`, `DEMO_VERFEHLEN_RATE`, `DEMO_EXTRA_LUST`, `DEMO_EXTRA_FERN`.
Der Demo-Spieler wirkt dadurch menschlicher (Reaktionszeit, kurze Fehler, ungenaues Treffen, seltenes absichtliches Verfehlen) und faengt gute fallende Kapseln, solange der Ball weit weg ist.
Testfassung `-D AUTOPLAY=1 -D TESTDEMO=1` zaehlt Ereignisse des Demo-Spielers im RAM (siehe Kopfkommentar).

**Winkelzufall (neu):** Beim Abschlag vom Schlaeger wird der Abprallwinkel mit der Chance `WINKEL_ZUFALL` (Standard 64 von 256, also etwa jeder vierte Treffer) um genau eine Zone (17 Grad) nach links oder rechts verschoben. Das gilt fuer den Menschen und den Demo-Spieler; der Ball bleibt immer in den acht Zonen, an den Raendern wird nicht darueber hinaus geschoben. Es verhindert endlose, immer gleiche Bahnen. `0` schaltet den Zufall ab (`-D WINKEL_ZUFALL=0`). Mit `-D AUTOPLAY=1` spielt der Rechner jetzt alle Levels ohne Ende (bisher nur die ersten fuenf); `-D TESTDEMO=1` zaehlt zusaetzlich die Winkelverschiebungen, die Schlaegertreffer, die geschafften Levels, die zerstoerten Steine und die durch das Zeitlimit beendeten Levels (Adressen im Kopf von `prallball.asm`).

**Demo-Zeitlimit (neu):** Trifft der Demo-Spieler `DEMO_STEIN_LIMIT` Sekunden (Standard 40) lang keinen Stein, gilt das Level als geschafft und das naechste beginnt; so haelt ein unerreichbarer oder zaeher letzter Stein die Demo nicht endlos auf. Nur in der Demo (und mit `AUTOPLAY`), nie im Spiel fuer Menschen; `0` schaltet es ab.
