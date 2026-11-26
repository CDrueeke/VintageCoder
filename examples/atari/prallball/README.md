# PRALLBALL (Atari 8 Bit) – Prototyp

Ball-und-Schlaeger-Spiel in Assembler fuer den Atari 800XL/130XE (PAL), eigene Umsetzung fuer die Atari-Hardware.
Die reine Spiellogik (Ball, Steine, Extras, Bonus-/Bosslevel) ist ein 6502-Programm und der C64-Fassung aehnlich, die Anzeige (ANTIC/GTIA), Figuren (Player/Missile),
Klang (POKEY) und Eingabe sind Atari-spezifisch. Der Quelltext ist ausfuehrlich kommentiert.

- **Bauen:** `vcc build .` (oder IDE F7); Start: `atari800 -xl -pal -nobasic -run build/prallball.xex`.
- **Bedienung:** Joystick Port 1 oder Tastatur (Z links, X rechts, Leertaste Feuer, ESC Abbruch); Namenseingabe: Tippen + RETURN.
- **Anzeige:** Display-Liste mit Textmodus 2 (Schrift) und 4 (Spielfeld); **Display-List-Interrupt setzt die Farben je Textzeile** (Himmelsverlauf, Regenbogen-Steine,
  Lichtkanten, Metall); Schlaeger = Spieler 0-2 (16/32/48 Punkte), Kapsel = Spieler 3 (Drehung), Baelle = Missiles 0-2, Laserschuss = Missile 3.
- **Stand:** Titel (Blocklogo mit Farbwechsel, Top-5, Musik, **Ticker** mit pixelweisen Uebergaengen, **Lauftext** mit Feinscrollen (HSCROL), huepfende Missile-Baelle, Demo nach 30 s mit Ticker in der Kopfzeile), Glanzlicht (Lichtkante der Steinzeilen blitzt von oben nach unten auf), 24 Levels (inkl. Bonus/Boss), Abprallwinkel nach Trefferstelle mit Bewegungseinfluss und Seitenaufprall,
  Extras als Muenzen (B, S, R, L, F, K, +, P, Mehrball, Laser, Durchschlag; an der Farbe erkennbar), Bestenliste mit Namenseingabe.
- **Atari-Besonderheiten (ausgereizt):** **Rasterzeilen-Kernel** (der DLI vor Zeile 18 setzt je Rasterzeile die Himmelsfarbe und die Spielerfarben: weicher Himmelsverlauf und Metallglanz im Schlaeger), **Wellen-Logo** (Feinscrollen HSCROL je Textzeile ueber die DLI-Tabelle, eigene Display-Liste mit LMS je Logozeile), Bestenliste auf dem Ladelaufwerk (DUNIT).
- **Noch nicht:** zweiter Laserschuss (nur ein Missile frei), Paddle (Abfrage vorbereitet, ungetestet), Bestenliste auf Diskette nur mit DOS getestet werden (ohne DOS meldet D: einen Fehler, die Standardliste bleibt), Klang ungehoert (nur Aufbau und Tabellen geprueft); die Pixelzeilen-Ausrichtung des Schlaeger-Glanzes ist nach Augenmass auf den Emulator abgestimmt (`kpm`).
- **Bibliotheken:** `lib/{input,hiscore,rand,sound}.asm` (Vertrag: siehe die Kopfkommentare der Dateien; Zufall aus dem POKEY-Hardwareregister, Musik auf allen 4 Kanaelen
  mit 16-Bit-Bass, Hiscore ueber CIO `D:PRALL.HSC`).
- **Tests:** `python3 tools/gen_tests.py && vcc build . && vcc test . --backend atari800` (68 Emulator-Tests + 3 statische Pruefungen: BCD-Punkte, Richtungstabelle, Schlaegerabprall,
  Steinkollision, Durchschlag, alle 24 Levels, Bestenliste mit CRC).
- **Bestenliste testen ohne DOS:** `-D HS_HOSTTEST=1` nutzt das Host-Geraet `H1:` des Emulators: `atari800 -xl -pal -nobasic -hreadwrite -H1 <Ordner> -run ...`; mit `-D TESTHS=1` schreibt und liest der Titel die Liste sofort und zeigt die CIO-Statuscodes (1 = Erfolg). Geprueft: `PRALL.HSC` hat 127 Byte im selben Format wie auf dem C64.
- **Testfassungen (`-D`):** `AUTOPLAY` (Rechner spielt; `DEMOFLAG=0` mit Kopfzeile), `TESTLOSE` (verliert absichtlich: Spielende/Namenseingabe), `NOHS` (ohne Diskettenzugriff beim Start), `STARTLEVEL=n` (mit Level n, 0-basiert, beginnen), `TESTMULTI` (sofort Mehrball), `TESTCAP` (jeder Stein laesst eine Kapsel fallen), `TESTLASER` (Laser immer an), `TESTKEY` (spielt Tastendruecke D, A, Z in den OS-Puffer ein), `TESTPM` (Ball-Y fest, zum Vergleich von Sprite- und Spielfeldzeilen), `NOTICKER` (ohne Ticker); Bestenliste: `HS_HOSTTEST`, `TESTHS` (siehe oben).
- **Werkzeuge:** `tools/make_levels.py` (Levelentwuerfe), `tools/compose.py` (Titelmusik, gleiche Noten wie C64, Atari-Spieler in `sound.asm`), `tools/gen.py` (Spielfeldzeichen `lib/gfx.inc.chars`), `tools/gen_tests.py` (erzeugt `prallball.vctest`). `lib/dirtable.inc` und `lib/levels.inc` sind unveraenderte Kopien der C64-Fassung (`examples/c64/prallball`, dort erzeugt von `tools/gen.py` aus `levels/levels.txt`); eine eigene Erzeugung fuer den Atari gibt es hier nicht.

## Einstellungen und Demo-Spieler (neu)

Am Anfang von `prallball.asm` steht der Block **EINSTELLUNGEN** (auch per `-D NAME=wert` beim Uebersetzen setzbar):
`EXTRA_RATE` (40: Chance in 256 je zerstoertem Stein auf Kapseln; etwa doppelt so viele wie vorher), `DEMO_REAKTION_MIN`/`DEMO_REAKTION_MAX` (4/12 Bilder Reaktionszeit nach einem Richtungswechsel des Balls), `DEMO_FEHLER_RATE`, `DEMO_FEHLER_FERN`, `DEMO_UNGENAU`, `DEMO_VERFEHLEN_RATE`, `DEMO_EXTRA_LUST`, `DEMO_EXTRA_FERN`.
Der Demo-Spieler wirkt dadurch menschlicher (Reaktionszeit, kurze Fehler, ungenaues Treffen, seltenes absichtliches Verfehlen) und faengt gute fallende Kapseln, solange der Ball weit weg ist.
Testfassung `-D AUTOPLAY=1 -D TESTDEMO=1` zaehlt Ereignisse des Demo-Spielers im RAM (siehe Kopfkommentar).

**Winkelzufall (neu):** Beim Abschlag vom Schlaeger wird der Abprallwinkel mit der Chance `WINKEL_ZUFALL` (Standard 64 von 256, also etwa jeder vierte Treffer) um genau eine Zone (17 Grad) nach links oder rechts verschoben. Das gilt fuer den Menschen und den Demo-Spieler; der Ball bleibt immer in den acht Zonen, an den Raendern wird nicht darueber hinaus geschoben. Es verhindert endlose, immer gleiche Bahnen. `0` schaltet den Zufall ab (`-D WINKEL_ZUFALL=0`). Mit `-D AUTOPLAY=1` spielt der Rechner jetzt alle Levels ohne Ende (bisher nur die ersten fuenf); `-D TESTDEMO=1` zaehlt zusaetzlich die Winkelverschiebungen, die Schlaegertreffer, die geschafften Levels, die zerstoerten Steine und die durch das Zeitlimit beendeten Levels (Adressen im Kopf von `prallball.asm`).

**Demo-Zeitlimit (neu):** Trifft der Demo-Spieler `DEMO_STEIN_LIMIT` Sekunden (Standard 40) lang keinen Stein, gilt das Level als geschafft und das naechste beginnt; so haelt ein unerreichbarer oder zaeher letzter Stein die Demo nicht endlos auf. Nur in der Demo (und mit `AUTOPLAY`), nie im Spiel fuer Menschen; `0` schaltet es ab.
