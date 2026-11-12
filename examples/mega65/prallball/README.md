# PRALLBALL (MEGA65)

Port des C64-Spiels `examples/c64/prallball` auf den MEGA65 (kein gemeinsamer Code mit dem C64, der Quelltext ist kopiert und angepasst).
Spielregeln, Level, Extras und Bedienung wie am C64 (Joystick Port 2 oder Tastatur: Z links, X rechts, Leertaste Feuer).

- **Bauen:** `vcc build .` (oder IDE F7), Start in Xemu (`xmega65 -prg build/prallball.prg -prgmode 65`, in der IDE F5).
  Testfassung, in der der Rechner spielt: `vcc asm prallball.asm --target mega65 --cpu 45gs02 -D AUTOPLAY=1 -o autoplay.prg`.
- **Aufbau des Quelltexts:** Der Kopf von `prallball.asm` enthaelt Ablauf, Speicherplan (CPU-RAM, Chip-RAM, Palette, Nullseite), Einheiten und die Liste aller
  Testfassungen (`-D ...`); jede Routine hat einen Kopfkommentar mit Eingabe/Ausgabe.  Die folgenden Punkte beschreiben die Entwicklungsstufen und Funktionen.
- **Stand (Stufe 2a, 2b und 2c; Grundlage, spaeter auf Vollfarbzeichen und 640 x 400 erweitert, siehe unten):** Das Spiel begann im VIC-II-Modus des VIC-IV (40 x 25 Mehrfarbtext, 8 Sprites) und nutzt davon die **VIC-IV-Palette**:
  16 eigene Farben (`pal_r/g/b`) und ein **weicher Hintergrundverlauf** statt 22 Farbbaendern: 50 Baender zu je 4 Rasterzeilen setzen
  den Paletteneintrag 0 (Hintergrund und Rahmen) auf Farbwerte aus `lib/gradient.inc` (4 Verlaeufe, erzeugt von `tools/gen_gradient.py`).
- **Klang (Stufe 2b), vier SIDs:** SID 1 die Titelmusik wie am C64, SID 2 ein verstimmter Chor derselben Stimmen (+0,6 %), SID 3 Oktavverdopplung
  (Melodie und Arpeggio +12 Halbtoene als Dreieck, Bass -12 als Rechteck), SID 4 die Klangeffekte dreistimmig geschichtet (Hauptton, Oktave darueber,
  Rauschen bei Stein, hartem Stein und Verlust). Die Musik wird so nie von Effekten unterbrochen. Nur in Xemu gestartet, die SID-Register lassen sich
  nicht zuruecklesen: **nicht gehoert, nur der Programmlauf geprueft**. Bibliothek: `examples/mega65/prallball/lib/sound.asm`.
- **16-Farb-Sprites (Stufe 2c):** Ball (Sprites 3, 4, 5) und Schlaeger (Sprites 1, 2) sind VIC-IV-Sprites im Full-Colour-Modus (16 Pixel breit,
  4 Bit je Pixel, `$D06B`): eine schattierte Kugel mit Glanzlicht (blau, bei Durchschlag golden durch Umschalten der Sprite-Palette) und ein
  metallisch schattierter Schlaeger mit runden Enden. Die Sprites haben eine eigene Palettenbank (`$D070`, Bank 1, Eintrag Sprite * 16 + Farbwert);
  Farbwert 0 ist durchsichtig. Der breite Schlaeger (48 px) nutzt ein X-verdoppeltes zweites Sprite. Die **Kapsel** (Sprite 0) ist eine schattierte 16-Farb-Muenze mit dunklem Buchstaben;
  die Palette (Eintraege 0-15 der Sprite-Bank) wird je Kapselart umgeschaltet (`set_cap_pal`), die Drehphasen sind vier Muenzenbilder. Die **Laserschuesse**
  (Sprites 6 und 7) sind 16-Farb-Sprites (weiss/gelb). Damit sind alle acht Sprites im 16-Farb-Modus.
  Die Daten erzeugt `tools/gen_sprites16.py` nach `lib/sprites16.inc` (ab `$B000`). In Xemu gesehen: Ball, Schlaeger (32 und 48 px), goldene Kugel (Testfassung `-D TESTWIDE=1`).
- **Bestenliste auf dem Geraet (Datei `PRALL.HS`):** Beim Start laedt `hs_load_kernal` die Liste, solange das KERNAL noch sichtbar ist (vor `mega_init`); beim
  Spielende schreibt `hs_save` sie: `hs_env_in` stellt den Zustand von SYS wieder her (`$01`/`$D030` vom Programmstart, Rasterinterrupt des VIC aus, `map` mit
  A = $00, X = $00, Y = $00, Z = $83 = Block `$E000` auf das KERNAL; im Leerlauf von BASIC zeigt der Monitor MAPL = $E300 / MAPH = $B300, das gilt aber nicht fuer
  ein SYS-Programm: mit dieser Abbildung wurde der Code bei `$2000` ausgeblendet), `hs_env_out` stellt die Spielumgebung her und ruft `hs_after` (= `vic_init`, die
  Grafikchip-Register koennen sich geaendert haben). Gemessen in Xemu (Geraet 8 = Host-Verzeichnis "XEMU EXTERNAL"): (1) `SETBNK(0,0)`, `SETMSG(0)`; (2) `LOAD` und
  `SAVE` ueber `SETNAM`/`SETLFS`/`SAVE` hingen, sequenzielle Dateien (`OPEN 2,8,2,"PRALL.HS,P,W"`, `CHKOUT`, `CHROUT`, `CLRCHN`, `CLOSE`, `CLALL`) laufen sauber; (3) ein
  `OPEN` zum Schreiben auf eine vorhandene Datei haengt (auch mit `@0:`): die alte Datei wird zuvor ueber den Befehlskanal geloescht (`OPEN 15,8,15,"S:PRALL.HS"`);
  (4) die DMA-Audio-Samples duerfen nicht in `$10000-$107FF` und `$11800-$11FFF` liegen (dort arbeitet das DOS des KERNALs): jetzt `$10800-$117FF`, `$1C800`, `$1F000`.
  In Xemu geprueft: Laden, Speichern, erneutes Laden im laufenden Spiel (Testfassung `-D TESTHS=1 -D TESTHS_A=1`, Ergebnisse ab `$7000`), die Liste bleibt ueber mehrere
  Programmlaeufe erhalten; auch das echte Spielende (Namenseingabe mit Auto-Bestaetigung, `hs_insert`, `hs_save`) ist in Xemu mit der Testfassung `-D TESTLOSE=1` durchlaufen (Punktestand 500 vorgegeben); beim naechsten Start zeigt das Titelbild die gespeicherten Eintraege. **Nicht** auf einem echten MEGA65 geprueft.
- **Palettenanimation (nur MEGA65):** (1) Die blaue Kugel faerbt sich mit dem Tempo zunehmend rot (Stufen 4-7, 8-11, 12-15; Tabellen `heat_r/g/b`, erzeugt von
  `tools/gen_sprites16.py`; `set_ball_pal` schreibt die Sprite-Palette).  (2) Der Randblitz (Ball verloren, Treffer, Kern) blendet weich aus: Palettenfarbe
  `FLASHPAL` ($FB) wird je Bild um ein Viertel abgedunkelt (`flash`, `flash_fade`).  (3) Bei der Namenseingabe pulsiert der Rand gelb.  Testfassung
  `-D TESTSPEED=n` startet mit Stufe n.
- **Truemmerteilchen (nur MEGA65):** Jeder zerstoerte Stein spritzt 4 Splitter in seiner Farbe (bis 8 gleichzeitig, `particle_*` in `prallball.asm`). Ein Splitter
  ist ein Vollfarbzeichen (8 x 8, Pixelwert 255 = Farbe aus dem Farb-RAM) mit einem 2 x 2- oder 3 x 3-Fleck in einem der vier Viertel; die 8 Zeichen liegen ab
  `$1CE00` (Zeichennummer `$738`) und werden mit dem **DMA-Controller** (Fuellauftrag, Format F018A) geloescht und danach mit Punkten gefuellt.  Sie fliegen mit
  Schwerkraft ueber leere Zellen (vor Steinen, Wand und Text sind sie unsichtbar; ein Fleck ersetzt immer eine ganze Zelle, ueber einem Stein wuerde er dessen
  Haelfte verschwinden lassen) und gehen in 320 x 200 und 640 x 400.  Testfassungen: `-D CRUMBTEST=1` (8 Steine auf einmal), `-D PTFREEZE=n` (Bild einfrieren,
  sobald ein Splitter nur noch n Bilder lebt).  In Xemu gesehen.
- **Wandernde Steine (nur MEGA65):** Gelegentlich (im Mittel etwa jede Sekunde, sofern es eine freie Nachbarzelle gibt) loest sich ein Stein (Typ 1-10, nicht in Bonus-
  und Boss-Levels) und gleitet weich um eine Position nach links, rechts, oben oder unten (0,4 s); hoechstens 2 gleichzeitig (`MAXW`, `wander_*` in `prallball.asm`).
  Zeichnen: das Steinmuster (Vollfarbzeichen) wird um 0..7 Bildpunkte verschoben in einen Zeichenstreifen im Chip-RAM gesetzt (vorher per DMA geloescht) und der
  Streifen innerhalb der beiden Zellen auf den Bildschirm geschrieben; funktioniert in 320 x 200 und 640 x 400.  Der Streifenspeicher liegt in der jeweils anderen
  Betriebsart ungenutzt ($14800 bzw. $12000); deshalb erzeugt das Umschalten (`toggle_mode`) die Steindaten der neuen Betriebsart neu (`fcm_init` / `hires_init`) und
  `toggle_game` stellt wandernde Steine vorher an ihrer Quelle ab (`wander_settle`).  **Kollision nach der sichtbaren Lage:** waehrend der Bewegung ist der Stein in der
  Karte nirgends eingetragen; Ball (`wcoll`, in beiden Achsen von `move_ball`) und Laserschuesse (`wshot`) pruefen gegen seine Lage in Spielfeldpunkten.  Ein
  getroffener Stein bleibt an der naechstliegenden Zelle stehen und wird wie jeder Stein behandelt (Punkte, Splitter, Bruch).  Ein wandernder Stein wartet, wenn der
  Ball im Weg ist.  Testfassungen: `-D WTEST=1` (haeufig, zaehlt Treffer/Enden in `wdbg`), `-D WTEST2=1` (immer Steine in Zeile 9), `-D WTEST3=1` (Steine bleiben
  mitten in der Bewegung stehen), `-D WFREEZE=n` (Bild einfrieren bei Fortschritt n/16 Punkt), `-D WVERT`/`WHORZ`/`WUP` (Richtung), `-D TESTTOG=1` (alle 256 Bilder
  umschalten).  In Xemu gesehen (beide Auflösungen, alle Richtungen, Umschalten im Spiel); Treffer auf wandernde Steine mit `WTEST3` gezaehlt.  Geschwindigkeit,
  Haeufigkeit und Anzahl noch nicht im Spiel abgestimmt.
- **Stereo-Lage (nur MEGA65):** Die DMA-Samples (Stein, Hartstein, Kern, Laser) klingen dort, wo sie passieren: `sfx_pan` (0 links .. 255 rechts, Mitte 128) setzt
  die Lautstaerken links/rechts des DMA-Kanals 0 (`$D729`/`$D71C`); `hit_brick` nimmt die Spalte des Steins, der Laser die Schlaegerposition. Die Register wurden in
  Xemu ueber den Monitor gelesen (Werte wechseln je nach Stein), gehoert wurde es nicht.
- **DMA-Audio und 40 MHz (Stufe 2e):** Der Prozessor laeuft mit 40,5 MHz (`$D031` FAST und `$D054` VFAST; das FAST-Bit in `$D031` muss gesetzt sein, weil
  jedes Schreiben auf `$D011`/`$D016`/`$D018` es aus `$D031` neu setzt). Die Spiellogik ist bildgebunden und laeuft unveraendert (AUTOPLAY liefert dieselben
  Werte wie bei 1 MHz). Der **Audio-DMA** (`$D711 = $80`, Kanal 0, `$D720-$D72F`) spielt zusaetzlich zu den SID-Effekten digitale Samples: Riss (Stein), Klang
  (harter Stein), Knall (Verlust), Pling (Extra), Zap (Laser). Die Samples werden **beim Start im Programm erzeugt** (Rauschen, Rechtecke, Tiefpass, Dreieck,
  fallender Ton; 8 Bit mit Vorzeichen, 6 kHz) und liegen im Chip-RAM Bank 1 (`$10800-$117FF`, `$1C800`, `$1F000`; `dma_init`/`dma_play` in `examples/mega65/prallball/lib/sound.asm`). In Xemu
  geprueft: die Sample-Daten (Inhalt und Abklang je Seite) und dass der Kanal bis zum Ende laeuft (`CHxSTP` gesetzt, Adresse am Ende); **nicht gehoert**.
  Testfassung `-D TESTDMA=1` spielt den Knall beim Start.
- **Mehrstimmige Musik:** Der Spieler (`examples/mega65/prallball/lib/sound.asm`) liest zwei Lieder gleichen Formats: Spur 1 (Melodie, Bass, Schlagzeug/Arpeggio) spielt auf
  SID 1 und verstimmt als Chor auf SID 2, Spur 2 (`mus_init3`: Gegenstimme in diatonischen Terzen, Echo, zwei Akkordtoene) auf SID 3. `tools/compose.py` erzeugt
  beide (`song`, `song3` in `lib/music.inc`, ab `$E000`). Ohne `mus_init3` spielt SID 3 wie bisher die Oktavverdopplung.
- **Vollfarb-Grafik (FCM, Super-Extended-Attribute-Mode):** `seam_regs` schaltet den VIC-IV auf 16-Bit-Zeichennummern (`$D054` = `$45`): jede Textzelle hat 2 Byte
  Bildschirm (jetzt bei `$C800`, 25 * 80 Byte) und 2 Byte Farb-RAM (Zugriff ueber den 32-Bit-Zeiger `cq` auf `$FF80000`, Byte 1 = Farbe). Heisse Register
  (`$D011`, `$D016`, `$D018`, `$D031`, `$DD00`) setzen diese Einstellung zurueck; der Rasterinterrupt ruft `seam_regs` deshalb nach jedem Schreiben auf `$D016` und `$D011`.
  Zeichennummern ab 256 sind Vollfarbzeichen: Daten bei Nummer * 64 im Chip-RAM (`FCM_BASE` = `$480`, also `$12000`), 8 Bit je Bildpunkt, Pixelwert = Palettenindex.
  Die Steine (24 Typen und die weisse Glanzfassung) haben abgerundete Ecken, Licht- und Schattenkante, Glanzstreifen und 8 Helligkeitsstufen je Typ (Palette
  32 bis 223); Wand (Stahl mit Niete) und Logo-Block (fester Rand, Flaeche mit Pixelwert 255 = Farbe aus dem Farb-RAM) ebenfalls. `tools/gen_bricks.py` erzeugt nur
  EIN Steinmuster; `fcm_init` bildet daraus beim Start die 24 Typen im Chip-RAM (Palettenbereich je Typ addieren). Daten: `lib/bricks.inc` ab `$E900`
  (`$F700-$FFFF` nicht benutzen: dort schreibt das C65-System).
- **Bruch-Animation:** Ein zerstoerter Stein blitzt weiss auf (Typ 24), zerbroeselt in zwei Phasen (`crumble_pat`: 2 x 2 grosse Stuecke, die um einige Zeilen
  fallen) und verschwindet; bis zu 8 gleichzeitig (`crumble_start`, `crumble_tick`, `crumble_reset`). Testfassungen: `-D CRUMBSLOW=1` (Phase 1 bleibt 5 s) und
  `-D CRUMBTEST=1` (zerstoert nach 100 Bildern 8 Steine der Zeile 3); in Xemu gesehen.
- **Umschaltbare Aufloesung (Taste R, auf dem Titel und im Spiel):** das Spiel laeuft in 320 x 200 oder 640 x 400 (`$D031`: Bit 7 H640 und Bit 3 V400). Die Spiellogik
  rechnet immer in 40 x 25 logischen Zellen; nur der Bildaufbau unterscheidet sich (`hires` = 0/1, `mode_set`, `toggle_mode`, `toggle_game`). **320 x 200:** wie bisher.
  **640 x 400:** eine logische Zelle sind 2 x 2 Zeichen (80 x 50 Zellen zu 8 x 8 Bildpunkten, 160 Byte je Zeile). Die Steine (32 x 16 Bildpunkte, 4 x 2 Vollfarbzeichen)
  haben gerundete Ecken, Lichtkante, Glanzbaender, Lichtfleck und geordnetes Raster zwischen den 8 Palettenstufen; Wand (Stahl mit Niete) und Logo-Block sind 16 x 16
  Bildpunkte gross; die Schrift entsteht beim Start aus dem 8 x 8-Zeichensatz (jedes Zeichen aus vier Vollfarbzeichen, `hires_init`), vergroessert und mit dem EPX-Verfahren (Scale2x, `font_epx`) geglaettet: Diagonalen sind weich statt treppig; Ticker und Lauftext sind
  8 x 8-Zeichen der hohen Aufloesung (halb so gross, dafuer feiner). Steine gibt es in 10 Farbgruppen mit je drei Fassungen (Stein, Bruch 1, Bruch 2; `tools/gen_bricks.py`,
  `brick_grp` ordnet jedem Typ eine Gruppe zu). **Sprites in 640 x 400** (`tools/gen_sprites640.py`, `lib/sprites640.inc` ab `$8000`, `spr_mode`, `update_sprites_h`): `SPRH640` (`$D054` Bit 4) und `SPRENV400` (`$D076`) machen einen Spritepunkt zu
  einem echten Bildpunkt, ein 16-Farb-Sprite ist 16 Punkte breit, die Hoehe steht in `$D055`/`$D056` (16 Zeilen; die Bloecke sind 128 Byte gross). Koordinaten: X' = x + 79 (10 Bit: `$D010` Bit 8, `$D05F` Bit 9),
  Y' = y + 107 (9 Bit: `$D077` Bit 8). Kugel 12 x 12, Schlaeger 12 Zeilen hoch (schmal: links + rechts; normal: links, Mitte X-verdoppelt, rechts = 64 Punkte; breit: alle drei verdoppelt = 96 Punkte; ist
  Sprite 7 durch einen Schuss belegt, bleiben zwei verdoppelte Enden), Muenze 16 x 16 mit Buchstabe, Schuss 4 x 16; Sprite 7 ist dritter Schlaegerteil, wenn kein Schuss ihn braucht; die Palette der
  Sprites 6 und 7 enthaelt in 640 x 400 Schlaegerrampe und Schussfarben (`set_s67_pal`). Beim Umschalten setzt `spr_mode` Zeiger, Hoehe, Verdopplung und Palette. Der Rasterverlauf zaehlt in der hohen Aufloesung echte
  Bildzeilen: die Interruptzeile ist 2 * T (9 Bit, Bit 8 in `$D011` Bit 7 nur bei Aenderung gesetzt, weil `$D011` ein heisses Register ist), der Lauftext wird im Band 49
  fein verschoben. Der Bildschirm (8192 Byte) liegt im Chip-RAM bei `$1D000` und wird ueber den 32-Bit-Zeiger `sq` beschrieben (im 64-KB-Raum wuerde er `$D000` ueberlaufen),
  die Schrift ab `$18800`, die Steine der hohen Aufloesung ab `$14800`. Start in 640 x 400 fuer Tests: `-D START_HIRES=1`. In Xemu gesehen (Titel und Spiel, beide
  Richtungen der Umschaltung). **Noch nicht:** die Sprites wurden nur in Xemu gesehen (Ball, Schlaeger in drei Breiten, Kapsel, Schuesse, Titelkugeln); die Lage ist gegen das Zeichenraster auf etwa einen Bildpunkt abgeglichen, nicht gegen die echte Hardware.
- **Noch nicht:** Bestenliste auf einem echten MEGA65 geprueft.
- **Speicher (MEGA65-Abweichungen):** Programm ab `$2001` (BASIC-65-Start); der Code ist laenger als zwischen `$0801` und `$3FFF`, daher
  liegt die VIC-Bank 2 (`$8000-$BFFF`): Sprite-Zeiger `$87F8`, Zeichensatz `$8800`, Sprites `$9000`, Levels `$9400`, 16-Farb-Sprites `$B000`; Steinkarte `$C000`,
  Titelmusik `$E000`, Vollfarb-Daten `$E900`; Bildschirm im Chip-RAM `$1D000` (siehe oben).
  Start (`mega_init`): `map` hebt die BASIC-65-Umblendung auf, `$01 = $35` (RAM bis `$FFFF`, Ein-/Ausgabe bei `$D000`), VIC-IV freischalten
  (`$D02F`), 40 Spalten; Interrupt-Vektor im RAM (`$FFFE`), der Interrupt rettet die Register selbst. Der Zeichensatz kommt aus dem
  Chip-ROM (`$2D000`, ueber einen 32-Bit-Zeiger).
- **Tests:** `python3 tools/gen_tests.py` erzeugt `prallball.vctest` (Falle bei `$1F00`). Die 8 statischen Pruefungen bestehen (Segmentgroessen und -lagen,
  Zyklen von `add_score`). Die Trap-Tests (`vcc test --backend xemu`) laufen inzwischen und haengen nicht mehr, liefern aber unzuverlaessige Ergebnisse (8 von 63 bestanden,
  einige Unterprogramme kehren nicht zurueck und der Emulator wird neu gestartet): der Test springt im Zustand des BASIC-65-Starts (C65-Umblendung, ROM bei `$E000`, Zeropage
  und Interrupt des Systems) in die Routine, ohne dass `mega_init` gelaufen ist; z. B. liefert `rand_byte` aus `$A5/$5A` `$43` statt `$CB`, `add_score` laesst die Ziffern
  unveraendert. Eine Loesung braucht einen Setup-Schritt im Testlaeufer (Programm starten und erst nach `mega_init` anhalten) oder eine Falle nach der Initialisierung; sie liegt
  im Testlaeufer (`crates/vc-emu`), nicht im Spiel. Als Funktionstest dient die AUTOPLAY-Fassung (in Xemu spielt der Rechner, auch in beiden Aufloesungen).
- **Bibliotheken:** `examples/mega65/prallball/lib/input.asm`, `hiscore.asm`, `rand.asm`, `sound.asm` (Kopien der C64-Fassung).
