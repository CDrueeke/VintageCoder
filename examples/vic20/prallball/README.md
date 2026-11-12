# PRALLBALL (VC-20, 8-KB-Erweiterung) – Prototyp

Ball-und-Schlaeger-Spiel in Assembler fuer den VC-20 (PAL) mit **mindestens 8 KB Erweiterung** ($2000-$3FFF; 3,5 KB intern + 8 KB = rund 11,5 KB). Der VIC-I
hat keine Sprites und nur 22 x 23 Zeichen; die Spiellogik ist deshalb eine **schlanke Neufassung** (nicht der Code der anderen Systeme). Quelltext ausfuehrlich kommentiert.

- **Bauen:** `vcc build .` (oder IDE F7; Projekteinstellung `memory: 8k`). Start: `xvic -memory 8k -autostart build/disk.d64` (Diskette; **nicht** die PRG allein einspielen, sonst bleibt der
  Dateizugriff der Bestenliste ohne Laufwerk).
- **Bedienung:** Joystick (einziger Anschluss) oder Tastatur: Z links, X rechts, Leertaste Feuer, P Pause, RUN/STOP zurueck zum Titel. Namenseingabe: tippen, RETURN (30 s Auto-Bestaetigung).
- **Anzeige:** 22 x 23 Zeichen (176 x 184 Punkte): Bildschirm `$1000`, Farb-RAM `$9400`, **eigener Zeichensatz im RAM bei `$1C00`** (der VIC-I sieht nur Speicher unter `$2000`; die Textzeichen 0..63
  kopiert das Programm aus dem Zeichen-ROM). Steine und Waende sind **Mehrfarbzeichen** (Licht = Hilfsfarbe weiss, Schatten = Rahmenfarbe schwarz), Text und Figuren einfarbig.
  Ball (5 x 5), Schlaeger (16/24/32 Punkte breit) und Muenze sind **Zeichen-Sprites**: die Zellen unter ihnen bekommen Pool-Zeichen (Codes 80..99), deren Bitmuster das Programm je Bild
  zusammensetzt (`blit`: Bildzeilen um X mod 8 verschieben, in zwei Zellen ODERn); der Hintergrund wird beim Rueckbau aus der Steinkarte berechnet (`tsp_restore`). Kein Rasterinterrupt:
  die Interrupts bleiben aus, der Bildtakt kommt aus dem Rasterregister `$9004` (Umbau der Figuren unterhalb der Anzeige).
- **Stand:** Titel (Logo mit Farbwechsel, Top-5, Musik, Demo nach 30 s), **22 Levels** (20 regulaere + 2 Bonus mit 45-s-Zeit und ohne Ballverlust), 10 Steinspalten (aus den 18 Spalten der anderen
  Systeme abgeleitet), Steinarten a-h, Silber, Gold, unzerstoerbar, explosiv, Ueberraschung, Abprallwinkel nach Trefferstelle mit Bewegungseinfluss, 8 Geschwindigkeitsstufen, Extras als fallende
  Muenzen (B breiter, S schmaler, L langsamer, F schneller, + Extraball, P 500 Punkte), Bestenliste mit Namenseingabe auf dem Startlaufwerk, Pause.
- **Bewusst weggelassen (Speicher/Rechenzeit, soweit das Target es kann):** Boss-Level, Mehrball, Laser, Durchschlag, Ticker und Lauftext, Paddle, Joystick-Namenseingabe.
- **Speicher:** Programm derzeit `$1201-$1BDC` (Grenze `$1BFF`) und `$2000-$34BC` (zusammen rund 7,8 KB mit Levels, Lied und Tabellen; die PRG-Datei ist 8894 Byte gross, weil die Luecke `$1BDD-$1FFF` mit Nullen gefuellt ist); `$1C00-$1FFF` Zeichensatz-RAM (im Programmabbild leer); Steinkarte `$0340` (Kassettenpuffer), Pool-Zellenliste `$03B8-$03DF`.
  Reserve bis `$3FFF`: rund 2,9 KB. `.org $2000` im Quelltext trennt die beiden Programmteile; der Testlaeufer prueft, dass Teil 1 vor `$1C00` endet.
- **Bestenliste:** `PRALL.HS` (127 Byte, Format wie C64) auf dem Laufwerk, von dem gestartet wurde. Der serielle Bus gehoert an Port A von VIA 1 mit: die Eingabe darf `$9113` nicht umstellen.
- **Klang (VIC-I, 3 Toene + Rauschen, gemeinsame Lautstaerke):** Titellied (16 Takte aus dem C64-Lied, Melodie Sopran, Bass Bass, Arpeggio Alt, Schlagzeug als Rauschstoss), Effekte auf Alt/Rauschen.
- **Noch nicht / ungeprueft:** Klang ungehoert (nur Registerwerte geprueft); Tastatur und Joystick nicht im Emulator getippt (Matrix gegen die KERNAL-Dekodiertabelle `$EC5E` geprueft); NTSC nicht getestet;
  Verhalten ohne Erweiterung (laeuft nicht).
- **Bibliotheken:** `lib/{input,hiscore,rand,sound}.asm` (Vertrag: siehe die Kopfkommentare der Dateien). Nullseite: Zufall `$02`, Musik `$04`, Bestenliste `$08`, Spiel `$10-$62`, Eingabe `$8C`.
- **Tests:** `python3 tools/run_tests.py` (nach `vcc build .`): startet xvic mit `-memory 8k` und dem Text-Monitor, ruft Unterprogramme einzeln auf. Grund: `vcc test --backend vice` startet xvic ohne
  Speichererweiterung. Der Pfad zu `xvic` steht oben in `tools/run_tests.py` (`EXE`, dort ein macOS-Pfad) und muss ggf. angepasst werden. 81 Pruefungen: Ladegroessen und Grenzen, BCD-Punkte, Zufall, Geschwindigkeitstabelle, Schlaegerabprall, Steinkollision, Steinarten inkl. Explosion, Muenzen (alle 6 Arten), alle 22 Levels, Zeichen-Sprites mit
  unabhaengig berechneten Bitmustern, Bestenliste, Klangwerte.
- **Testfassungen (`-D`):** `AUTOPLAY` (Rechner spielt von Anfang an), `STARTLEVEL=n` (0-basiert), `TESTLOSE` (verliert absichtlich, 5000 Punkte: Spielende/Namenseingabe), `TESTCAP` (aus jedem zerstoerten Stein faellt eine Muenze). Bauen:
  `vcc asm prallball.asm --target vic20 --memory 8k -D AUTOPLAY=1 -o x.prg`; zum Starten eine Diskette bauen (`vcc disk d64 -o x.d64 --add x.prg:PRALLBALL`).
- **Werkzeuge:** `tools/gen.py` (erzeugt `lib/*.inc`: Richtungstabelle, Levels auf 10 Spalten aus `levels/levels.txt`, Mehrfarbzeichen, Logo, gekuerztes Lied aus dem C64-Lied `examples/c64/prallball/lib/music.inc`), `tools/make_levels.py` (Kopie des C64-Skripts, erzeugt `levels/levels.txt`), `tools/run_tests.py`.
