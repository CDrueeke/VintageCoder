# PRALLBALL (Commodore 128, VDC-Fassung, 80 Spalten) – Prototyp

Zweite C128-Fassung neben der 40-Spalten-Fassung `examples/c128/prallball` (VIC-II). Der **VDC** (8563/8568) kann nur Text: keine Sprites, keinen Rasterinterrupt, keine
Mehrfarbzeichen; dafuer 80 Spalten, zwei Zeichensaetze und eine Farbe je Zelle. Die Spiellogik ist die der C64-/Plus/4-Fassung; die Anzeige ist neu. Quelltext ausfuehrlich kommentiert.

- **Bauen:** `vcc build .`; Start im **80-Spalten-Modus**: `x128 -80col -autostart build/prallball.prg` (oder `build/disk.d64`). Am echten C128 vor dem Start die Taste 40/80 DISPLAY druecken
  (der Monitor muss am VDC-Ausgang haengen).
- **Bedienung:** Joystick Port 2 oder Tastatur (Z links, X rechts, Leertaste Feuer, P Pause, RUN/STOP Abbruch); Namenseingabe: Tippen + RETURN.
- **Anzeige:** Jede logische Zelle (40 x 25 wie beim C64) besteht aus **zwei VDC-Zellen**: das Zeichen wird auf 16 x 8 Pixel gedehnt (links die linken, rechts die rechten vier Pixel, jeweils doppelt; links
  Zeichensatz 1, rechts Zeichensatz 2 ueber Attributbit 7). Alle Texte und das Layout bleiben dadurch unveraendert. Der Bildschirm liegt im VDC ($0000 Zeichen, $0800 Attribute, $2000/$3000
  Zeichensaetze); im Hauptspeicher stehen Spiegel, Schattenkopien des Hintergrunds und der Zeichensatz (1 Bit/Pixel).
- **Zeichen-Sprites** (Ball bis 3, Schlaeger, Muenze, Laserschuesse): wie am Plus/4 setzt `tsp_render` je Bild Pool-Zeichen (128..191) aus Hintergrund UND Maske ODER Bild zusammen
  (einfarbig, 8 Feinlagen, 1 logisches Pixel genau) und laedt sie **gedehnt in den VDC**. Zellen, die auch im naechsten Bild belegt sind, werden nicht kurz geleert (kein Flackern).
- **Unterschiede zur VIC-II-Fassung:** einfarbige Steine (Farbe je Zelle statt Lichtkante/Schatten), statt der Rasterbalken eine dunkle Hintergrundfarbe je Level (VDC-Register 26),
  Ticker laeuft ueber geaenderte Zeichen-Bitmuster (nur Unterschiede werden geladen), Lauftext rutscht alle 6 Bilder um ein ganzes Zeichen (Feinscrollen einer Zeile geht am VDC nicht),
  kurze Hintergrundblitze (Steintreffer) entfallen. Der VIC-II ist dunkel und liefert nur den Bildtakt (Rasterinterrupt Zeile 251); der Prozessor laeuft mit **2 MHz**, fuer die Bestenliste
  mit 1 MHz.
- **Speicher / MMU:** wie die VIC-II-Fassung (Konfiguration `$3E`, alles RAM, eigener Interruptvektor); Bestenliste und `copy_font` unter `$4000`. Die Bestenliste wird **vor** der
  Umstellung auf 2 MHz und dunklen VIC gelesen (danach blieb der KERNAL in der schnellen seriellen Uebertragung haengen); das Speichern am Spielende geht (1 MHz waehrenddessen).
- **Stand:** Titel (Logo, Top-5, Ticker, Lauftext, Demo nach 30 s), 24 Levels, Extras, Mehrball/Laser, Bonus-/Bosslevel, Bestenliste auf dem Startlaufwerk. Geprueft in `x128 -80col`.
- **Noch nicht / ungeprueft:** echter C128 (nur Emulator; VDC-Zugriffszeiten und Flackern am Geraet unbekannt), Klang ungehoert, Tastatur/Joystick nicht getippt, 64-KB-VDC nicht noetig (16 KB genuegen),
  NTSC nicht getestet. Im ungünstigsten Fall (3 Baelle + Laser + Muenze) verpasst die Hauptschleife rund 2-3 % der Bildwechsel (`TESTFPS`).
- **Tests:** `python3 tools/gen_tests.py && vcc build . && vcc test . --backend vice` (x128, Trap `$BF00`, `"$ff00": 62`): 103 Emulator-Tests; der VDC-Inhalt wird mit `vdc_get` zurueckgelesen
  (Bildschirm, Attribute, gedehnte Pool-Zeichen, Ticker), dazu Logiktests, Compositor mit unabhaengig berechneten Bitmustern, Bestenliste, Klang-Schritte uebernommen wie am C64.
- **Testfassungen (`-D`):** `AUTOPLAY`, `STARTLEVEL=n`, `TESTMULTI`, `TESTLASER`, `TESTLOSE`, `TESTFPS`, `TESTCAP`; Bauen: `vcc asm prallball.asm --target c128 --cpu 8502 -D AUTOPLAY=1 -o x.prg`.
- **Werkzeuge:** `tools/gen.py` (Richtungstabelle, Levels, Zeichen, Sprite-Streifen, Dehnungstabellen), `tools/make_levels.py` (Levelentwuerfe 4-24 nach `levels/levels.txt`),
  `tools/compose.py` (Titelmusik nach `lib/music.inc`), `tools/gen_tests.py` (erzeugt `prallball.vctest`). Weiterer Schalter: `NOTICKER` (ohne Demo-Ticker); Ueberblick im Kopf von `prallball.asm`.
