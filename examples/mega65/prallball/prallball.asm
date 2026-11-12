; =====================================================================================================================
; prallball.asm - PRALLBALL fuer den MEGA65 (C65-Modus, VIC-IV, 4 SIDs, DMA-Audio, 40,5 MHz)
; =====================================================================================================================
;
; ZWECK
;   Ein Ball-und-Schlaeger-Spiel: Der Ball prallt von Waenden, Schlaeger und Steinen ab; sind alle zerstoerbaren Steine weg,
;   beginnt das naechste Level. Faellt der Ball unten heraus, geht ein Leben verloren; nach dem letzten Leben endet das Spiel
;   (Bestenliste mit Namenseingabe, gespeichert in der Datei PRALL.HS).  Dies ist die Fassung, die die Faehigkeiten des MEGA65
;   ausnutzt; die Spiellogik (Physik, Levels, Extras, Boss) stammt vom C64-Spiel, der Code ist aber eigenstaendig.
;   Beschreibung fuer Anwender: README.md.
;
; WAS NUR DER MEGA65 KANN (und wo es im Code steht)
;   * 16-Bit-Zeichennummern, Vollfarbzeichen (FCM, 256 Farben, Pixelwert 255 = Farbe aus dem Farb-RAM) im Super-Extended-Attribute-Mode:
;     Steine, Wand, Logo, Schrift (seam_regs, put_fcm, put_brick; Bildschirm im Chip-RAM bei $1D000, 32-Bit-Zeiger sq/cq)
;   * umschaltbare Aufloesung 320 x 200 / 640 x 400 (Taste R; mode_set, toggle_mode), in 640 x 400 echte Sprite-Pixel (update_sprites_h)
;   * eigene Palette: Hintergrundverlauf per Rasterinterrupt (irq), 16-Farb-Sprites, Tempo-Glut der Kugel, weich ausblendender Randblitz
;     (set_ball_pal, flash/flash_fade, FLASHPAL)
;   * Klang: 4 SIDs (examples/mega65/prallball/lib/sound.asm) plus digitale Effekte ueber den Audio-DMA mit Stereo-Lage (sfx_pan)
;   * DMA-Controller (Fuellauftrag): Zeichensaetze der Splitter und der wandernden Steine (particle_init, w_draw)
;   * Truemmerteilchen (particle_*) und wandernde Steine (wander_*, Kollision nach der sichtbaren Lage: wcoll, wshot)
;   * Bestenliste als Datei ueber das KERNAL des C65-Modus (examples/mega65/prallball/lib/hiscore.asm; hs_after stellt danach den VIC wieder her)
;
; BEDIENUNG
;   Joystick in Port 2 (links/rechts, Feuer) ODER Tastatur: Z = links, X = rechts, Leertaste = Feuer, R = Aufloesung umschalten (Titel und
;   Spiel; mode_key), RUN/STOP = Abbruch zum Titel.  In der Namenseingabe: Buchstaben tippen oder mit hoch/runter (Q/A, Joystick) waehlen, links/rechts = Stelle, Feuer = fertig.
;   Feuer startet den Ball vom Schlaeger (und im Titel das Spiel).  P (Pause) liefert input.asm zwar, das Spiel wertet sie aber nicht aus.
;   Im Demo-Modus beendet jede Eingabe die Demo (get_input liefert dann IN_ABORT).
;   Der Abprallwinkel haengt davon ab, wo der Ball den Schlaeger trifft (8 Zonen: -60 bis +60 Grad).
;
; ABLAUF (Zustandsautomat im Hauptprogramm)
;   start      Zustand des BASIC-65-Starts merken, Bestenliste laden (solange das KERNAL sichtbar ist), mega_init, fcm_init ...
;   title      Titelbild (Logo, Bestenliste, Lauftext), nach Wartezeit Demo-Modus (der Rechner spielt) ->  new_game
;   new_game   Punkte 0, 3 Leben, Level 1           ->  load_level
;   load_level Steinkarte laden und zeichnen, Ball auf den Schlaeger  ->  play
;   play       ein Durchlauf je Bildwechsel (50 Hz): Eingabe, Schlaeger, Baelle (move_all/move_ball), Kollisionen, Extras, Bruch-Animation,
;              Splitter, wandernde Steine, Anzeige.  Level geschafft -> naechstes Level;  Ball verloren -> Leben - 1;  Leben 0 -> game_over
;   game_over  Bestenliste pruefen, Namenseingabe (name_entry), eintragen, speichern  ->  title
;
; TESTFASSUNGEN (vcc asm prallball.asm --target mega65 --cpu 45gs02 -D NAME=wert ...)
;   AUTOPLAY    der Rechner spielt selbst (Demo)           TESTLOSE   Ball geht immer verloren, Startpunktestand 500 (Spielende pruefen)
;   START_HIRES Start in 640 x 400                         STARTLEVEL=n  mit Level n beginnen          TESTSPEED=n  mit Tempostufe n beginnen
;   CRUMBTEST   8 Steine auf einmal zerstoeren             CRUMBSLOW  langsame Bruch-Animation         PTFREEZE=n   Splitter-Bild einfrieren
;   WTEST, WTEST2, WTEST3, WFREEZE=n, WVERT, WHORZ, WUP    wandernde Steine (haeufig / Zeile 9 / stehen bleiben / einfrieren / Richtung)
;   TESTTOG     alle 256 Bilder die Aufloesung umschalten  TESTHS (+ _A/_M/_V)  Bestenliste im Spiel pruefen, Ergebnisse ab $7000
;   TESTMULTI   sofort Mehrball                            TESTWIDE   Durchschlag-Kugel und breiter Schlaeger (48 px)
;   TESTLASER   Laser immer an                             TESTCAP    jeder Stein laesst eine Kapsel fallen
;   TESTDMA     beim Start den Knall einmal per Audio-DMA abspielen                TESTSTRIPES  Hintergrundverlauf 3 (Farbbaender) statt Verlauf 0
;   TESTFPS     16-Bit-Bildzaehler bei $C200 (zeigt ausgelassene Bilder)           NOTICKER     Ticker (Lauftext oben) im Spiel aus
;   Alle Schalter werden mit .ifdef/.ifndef abgefragt (Wert nur bei den mit =n genannten); sie wirken nur an der im Quelltext markierten Stelle.
;
; SPEICHERPLAN
;   CPU-RAM (Bank 0, "RAM ueberall" nach mega_init; $01 = $35, Ein-/Ausgabe bei $D000-$DFFF):
;     $2001-$6E09  BASIC-Starter (SYS), Programmcode, Bibliotheken (input, rand, sound, hiscore), Richtungstabelle, Zeichen gfx_chars
;     $6E0A-$7FFF  frei (nur -D TESTHS: Ergebnisse von hs_test ab $7000)
;     $8000-$852F  16-Farb-Sprites fuer 640 x 400 (lib/sprites640.inc; VIC-Bank 2)   $8800-$8FFF  Zeichensatz (Kopie des Zeichen-ROMs, 2 KB)
;     $9000-$92BF  Sprite-Bloecke der C64-Fassung (lib/gfx.inc.sprites; vermutlich nicht mehr benutzt)   $9400-$A887  Levelkarten (lib/levels.inc, 24 Level)
;     $B000-$BC8F  16-Farb-Sprites (lib/sprites16.inc: Kugel, Schlaeger, Kapsel, Schuss; Sprite-Zeiger $C0 ff.)
;     $C000-$C0D7  Steinkarte (216 Byte = 12 Zeilen x 18 Spalten, 0 = leer, 1-8 Stein, 9 Silber, 10 Gold, 11 unzerstoerbar, 12 explosiv,
;                  13 Ueberraschung, 16-23 Boss-Kern)   $C100-$C13B  Ballspeicher (3 Baelle), Laserschuesse, Titelbaelle   $C140-$C17F  Ticker-Texte
;     $C200/$C201  Bildzaehler (nur TESTFPS)   $C300/$C400  Puffer der Schrift-Glaettung (EPX, 100 bzw. 256 Byte)
;     $E000-$E879  Lieder (song, song3)   $E900-$F5C3  Vollfarb-Steinmuster und Palette (lib/bricks.inc, hinter dem Programm)
;     $F700-$FFFF  System des C65-Modus: wird nicht benutzt, nur der IRQ-Vektor $FFFE/$FFFF wird auf irq gesetzt (vic_init)
;   VIC-Bank 2 ($8000-$BFFF, gewaehlt ueber CIA 2 $DD00): Der Text-Bildschirm der VIC-II-Sicht ($D018 = $12 -> $8400) wird NICHT benutzt;
;     der Bildschirm liegt im Chip-RAM (SEAM, siehe unten).  Von der VIC-II-Sicht gebraucht werden nur die Sprite-Zeiger bei SPTR = $87F8
;     (ueber $D06C-$D06E eingestellt) und die Sprite-Daten (Zeiger * 64 + $8000).  Der Zeichensatz bei $8800 dient als Vorlage fuer die
;     Schrift (put_fcm, hires_init) und die Ticker-Effekte.
;   Chip-RAM Bank 1 ($10000-$1FFFF; Adresse = Zeichennummer * 64):
;     $10000-$107FF und $11800-$11FFF   DOS des KERNALs: NICHT beschreiben (sonst "device not present" beim Speichern der Bestenliste)
;     $10800-$115FF  DMA-Samples Riss, Klang, Zap (belegt, bis $117FF reserviert)   $1C800 Pling   $1F000 Knall   (erzeugt von dma_init in sound.asm)
;     $12000-$1447F  Steine 320 x 200 (Vollfarbzeichen ab Nummer FCM_BASE = $0480: 24 Typen, Wand/Logo, 2 Bruchphasen)
;     $14800-$183FF  Steine 640 x 400 (3 Fassungen x 10 Farbgruppen x 512 Byte)     $18400 Wand, Logo, Block     $18800-$1C7FF  Schrift (Viertelzeichen)
;     $1CE00-$1CFFF  Zeichen der Splitter (8)     $1D000-$1EFFF  Bildschirm (SCRQ, 8192 Byte, 2 Byte je Zelle; Farb-RAM bei $FF80000)
;     Streifen der wandernden Steine: 320 x 200 ab $14800 (hier liegen sonst die 640 x 400-Steine, die beim Umschalten neu erzeugt werden), 640 x 400 ab $12000
;   Palette (VIC-IV, Werte mit vertauschten Halbbytes in $D100/$D200/$D300): Bank 0 = Text/Bitmap: 0 Hintergrundverlauf/Rand (vom irq je Band gesetzt),
;            1-15 Grundfarben, 32-223 Steine (24 Typen x 8 Stufen), 224-231 Wand, 232-233 Logo-Rand, 234 schwarz (Rand 640 x 400), 251 Randblitz/pulsierender Rand;
;            Bank 1 = Sprites ($D070 Bits 2-3): 0-15 Grundfarben, 16-47 Schlaeger (Sprites 1 und 2), 48-95 Kugeln (Sprites 3-5), 96-127 Schuesse (Sprites 6, 7)
;
; NULLSEITE (Zeropage)
;   $01  CPU-Port (= $35)   $10-$3B  Spielzustand: Ball, Schlaeger, Punkte, Hilfsspeicher t0-t3, Zeiger sp/cp/tp (siehe .const unten)
;   $40-$5D  Namenseingabe, Schlaegerbreite, Kapsel, Extras, Demo/Titel, Logo, Sprite-Sammelbits   $5E-$8B  Laser, Interrupt/Verlauf (bp, bflash), Lauftext,
;   Titellogo, Bonus-/Bosslevel, Ticker   $8C-$8F  W_SRC (Zeiger der wandernden Steine)   $90-$97  Klangerzeugung (SYN_ZP)
;   $A0-$A7  sq/sr (32-Bit-Zeiger Bildschirm)   $E0-$E3  Hilfszeiger (32 Bit; nur kurz benutzt: Zeichensatz kopieren, Vollfarbdaten erzeugen, Klang-Samples)
;   $E4-$E7  cq (Farb-RAM)   $E8-$EB  Lied (MUS_ZP: Zeiger + Liedkopf)   $F0-$F5  Bestenliste (HS_ZP)   $F6-$F7  Zufall (RAND_ZP)   $F8-$FB  Eingabe (IN_ZP)
;
; KOORDINATEN UND EINHEITEN (wichtig fuer das Verstaendnis der Physik!)
;   Spielfeldpunkte (px) zaehlen ab der linken oberen Ecke des BILDSCHIRMS in 320 x 200 (Zeichen 0/0 = px 0/0); ein Zeichen ist 8 x 8 px.  In
;   640 x 400 rechnet das Spiel unveraendert in diesen Punkten (ein Spielfeldpunkt = 2 x 2 Bildpunkte); nur Zeichnen und Sprites rechnen um.
;   Spielfeld: x = 16..303 (Spalten 2..37), y = 16.. (Zeile 2 abwaerts); Waende in Spalte 1 und 38 sowie Zeile 1.
;   Steinfeld: 18 Steine je 16 px Breite (2 Zeichen) ab x = 16, 12 Zeilen je 8 px Hoehe ab y = 32 (Bildschirmzeile 4).
;   Ball und Schlaeger-Positionen: BALL in "Festkomma 1/64": Wert = Bildpunkte * 64 (16 Bit).  So reichen 16 Bit fuer x bis 1023 px
;   und die Geschwindigkeit kann in 1/64 px je Bild feiner als 1 px sein.  Die obere Haelfte (High-Byte) = Bildpunkte / 4.
;   Splitter: 1/16 px.  Wandernde Steine: Fortschritt 1/16 px, sichtbare Lage in ganzen px.
;   Sprite-Koordinaten des VIC-II beginnen bei x = 24, y = 50 (linker/oberer Rand des Textfensters): sprite_x = px + 24, sprite_y = py + 50.
;   In 640 x 400 zaehlen die Sprite-Koordinaten echte Bildpunkte (SPRH640/SPRENV400, update_sprites_h): X' = 2 * px + 79, Y' = 2 * py + 107
;   (die 12 Punkte breite Kugel liegt 2 Punkte eingerueckt in ihrer 16 x 16 Punkte grossen Flaeche, daher dort 77 bzw. 105).
; =====================================================================================================================
.target mega65
.cpu 45gs02 { illegals = deny }
.branchlong auto                        ; zu weite bedingte Spruenge verlaengert der Assembler selbst (Warnung W203)

; ---------------------------------------------------------------------------------------------------------------------
; Nullseiten-Variablen des Spiels ($10-$3F)
; ---------------------------------------------------------------------------------------------------------------------
.const bxl     = $10                    ; Ball-X, Festkomma 1/64 px, Low-Byte
.const bxh     = $11                    ;                           High-Byte
.const byl     = $12                    ; Ball-Y, Festkomma 1/64 px
.const byh     = $13
.const vxl     = $14                    ; Ball-Geschwindigkeit X (vorzeichenbehaftet, 1/64 px je Bild)
.const vxh     = $15
.const vyl     = $16                    ; Ball-Geschwindigkeit Y (negativ = nach oben)
.const vyh     = $17
.const padl    = $18                    ; Schlaeger: linke Kante in px (16 Bit, 16..272)
.const padh    = $19
.const speed   = $1a                    ; Geschwindigkeitsstufe 0..15 (Index in die Richtungstabelle)
.const hits    = $1b                    ; Schlaegertreffer seit dem Start; alle 8 steigt die Stufe um 1
.const lives   = $1c                    ; verbleibende Baelle
.const level   = $1d                    ; aktuelles Level 0.. (Index in die Levelliste)
.const bricks  = $1e                    ; noch zu zerstoerende Steine (unzerstoerbare zaehlen nicht)
.const stuck   = $1f                    ; 1 = Ball haftet noch am Schlaeger (wartet auf Feuer)
.const sc0     = $20                    ; Punktestand BCD, niedrigste zwei Ziffern (Dezimalmodus!)
.const sc1     = $21                    ;                  mittlere zwei Ziffern
.const sc2     = $22                    ;                  hoechste zwei Ziffern  -> 6 Stellen
.const mask    = $23                    ; Eingabemaske dieses Bildes (IN_*)
.const t0      = $24                    ; Hilfsspeicher (kurzlebig, jede Routine beschreibt ihren Gebrauch)
.const t1      = $25
.const t2      = $26
.const t3      = $27
.const bcol    = $28                    ; zuletzt getroffener Stein: Spalte 0..17
.const brow    = $29                    ;                             Zeile 0..11
.const bidx    = $2a                    ;                             Index in die Steinkarte (Zeile * 18 + Spalte)
.const pxl     = $2b                    ; Ball-Position in ganzen px (aus get_px), Low-Byte
.const pxh     = $2c                    ;                                          High-Byte
.const sp      = $2d                    ; Zeiger auf Bildschirm-Ziel (2 Byte, fuer (sp),y)
.const cp      = $2f                    ; Hilfszeiger (2 Byte; nur copy_font)
.const cq      = $e4                    ; 32-Bit-Zeiger auf das Farb-RAM ($FF80000 + Zeile * 80 + 2 * Spalte), benutzt mit [cq],z (4 Byte)
.const tp      = $31                    ; Zeiger auf Text (2 Byte)
.const txtcol  = $33                    ; Textfarbe fuer put_cell/puts
.const atgt    = $34                    ; Autoplay: Zielpunkt auf dem Schlaeger (8..23 px von der linken Kante)
.const ptA     = $35                    ; Hilfsspeicher von probe
.const ptB     = $36
.const oxl     = $37                    ; Ball-X vor der Bewegung (zum Zuruecksetzen beim Abprall)
.const oxh     = $38
.const oyl     = $39                    ; Ball-Y vor der Bewegung
.const oyh     = $3a
.const npos    = $40                    ; Namenseingabe: Cursorposition 0..7
.const nprev   = $41                    ;   Eingabemaske des vorigen Bildes (Flankenerkennung)
.const nrep    = $42                    ;   Bilder bis zur naechsten Tastenwiederholung
.const ntl     = $43                    ;   Zaehler bis zur Auto-Bestaetigung (16 Bit, in Bildern), Low-Byte
.const nth     = $44                    ;   High-Byte
.const padw    = $45                    ; aktuelle Schlaegerbreite in px (16, 32 oder 48)
.const zstep   = $46                    ; Breite einer Abprallzone = padw / 8 (2, 4 oder 6 px)
.const pad2o   = $47                    ; Abstand des zweiten Schlaegersprites vom ersten (16 oder 24 px; 0 = nur ein Sprite)
.const capact  = $48                    ; 1 = eine Kapsel faellt
.const capxl   = $49                    ; Kapsel: linke Kante in px (16 Bit)
.const capxh   = $4a
.const capy    = $4b                    ;          obere Kante in px
.const captype = $4c                    ;          Art 0..7 (siehe Tabelle cap_letter)
.const tpad    = $4d                    ; Rest-Halbsekunden: Schlaegerbreite geaendert (breit/schmal)
.const trev    = $4e                    ;                    Steuerung umgekehrt
.const tspd    = $4f                    ;                    Geschwindigkeit geaendert
.const spdmod  = $50                    ; 0 = normal, 1 = schneller (+4 Stufen), 2 = langsamer (-4 Stufen)
.const tick    = $51                    ; Bilder bis zur naechsten halben Sekunde (zaehlt die Timer)
.const sticky  = $52                    ; 1 = naechster Schlaegertreffer faengt den Ball (Klebe-Extra)
.const demo    = $53                    ; 1 = Demo-Modus (Rechner spielt)
.const idl     = $54                    ; Titel: Bilder ohne Eingabe bis zur Demo (16 Bit)
.const idh     = $55
.const lt      = $56                    ; Titellogo: Buchstabe 0..8
.const lr      = $57                    ;             Zeile 0..4
.const lbits   = $58                    ;             Bitmuster der Zeile
.const lcol    = $59                    ;             Spalte des ersten Buchstabenpixels
.const cb      = $5a                    ; aktueller Ball (Schlitz 0..2), dessen Werte in bxl..vyh stehen
.const nact    = $5b                    ; Anzahl aktiver Baelle in diesem Bild
.const sprmsb  = $5c                    ; update_sprites: gesammeltes 9. X-Bit der Sprites
.const sprena  = $5d                    ; update_sprites: gesammelte Sprite-Freigabebits
.const tbu     = $c130                  ; Titel-Baelle (3): X in Einheiten zu 2 px
.const tby     = $c133                  ;                    Y in px
.const tbdu    = $c136                  ;                    Schritt X (1 oder $FF)
.const tbdy    = $c139                  ;                    Schritt Y (1, 2 oder negativ)
.const bp      = $61                    ; Zeiger auf die Farbtabelle des aktuellen Hintergrund-Verlaufs (2 Byte, im Interrupt benutzt)
.const bflash  = $63                    ; Rest-Bilder des Randblitzens (0 = kein Blitz)
.const glg     = $64                    ; Glanzlicht: Position der laufenden Diagonale (0..30, danach Pause bis 99)
.const scx     = $65                    ; Lauftext: Feinversatz 7..0 (Bildpunkte)
.const stp     = $66                    ; Lauftext: Leseposition im Text
.const lph     = $67                    ; Titellogo: Farbphase
.const lcnt    = $68                    ; Titellogo: Bildzaehler fuer den Farbwechsel
.const nch     = $69                    ; Namenseingabe: getipptes Zeichen dieses Bildes (0 = keins)
.const ntyped  = $6a                    ;   1 = es wurde schon getippt (die Vorgabe ist geloescht)
.const capfr   = $6b                    ; Kapsel: Drehphase (Bildzaehler 0..23, je 4 Bilder eine Phase)
.const lkind   = $6c                    ; Levelart: 0 normal, 1 Bonus, 2 Boss
.const btime   = $6d                    ; Bonuslevel: Restsekunden
.const bsub    = $6e                    ; Halbsekundenzaehler fuer die Sekunden
.const cores   = $6f                    ; Bosslevel: verbliebene Kerne
.const batk    = $70                    ; Bosslevel: Halbsekunden bis zum naechsten Angriff
.const capforce = $71                   ; spawn_cap: erzwungene Kapselart + 1 (0 = zufaellig)
.const stq     = $72                    ; Lauftext: Leseposition als 16-Bit-Zeiger (der Text ist laenger als 256 Zeichen)
.const tpierce = $88                    ; Rest-Halbsekunden: Durchschlag aktiv (Ball zerstoert jeden zerstoerbaren Stein und prallt nicht ab)
.const tkrow   = $74                    ; Ticker: Bildschirmzeile
.const tkst    = $75                    ;         0 = warten, 1 = Effekt laeuft
.const tkw     = $76                    ;         Wartebilder
.const tkeff   = $77                    ;         Effekt 0..6
.const tkstep  = $78                    ;         Schritt
.const tkmsg   = $79                    ;         Index in der Textfolge
.const tkseq   = $7a                    ;         Folge 0 = Titel, 1 = Demo
.const tkph    = $7b                    ;         Farbphase (zaehlt Bilder)
.const tka     = $7c                    ;         Zeiger Bitmuster altes Zeichen (2 Byte)
.const tkb     = $7e                    ;         Zeiger Bitmuster neues Zeichen (2 Byte)
.const tkc     = $80
.const tkr     = $81
.const tkd     = $82
.const tkt     = $83
.const tks     = $84
.const tklf    = $85
.const tkdiv   = $86
.const tkd2    = $87
.const tkce    = $89                    ; tk_vert: Endzeichen (exklusiv) dieses Durchlaufs
.const tkre    = $8a                    ; tk_hleft/tk_hright: Endzeile (exklusiv) dieses Durchlaufs
.const tkrs    = $8b                    ; Startzeile
.const wfc     = $60                    ; wait_frames: Zaehler
.const shi     = $5f                    ; update_shots: Schussnummer (ueberlebt hit_brick)
.const tlas    = $5e                    ; Rest-Halbsekunden: Laser aktiv
.const dxo     = $3b                    ; Abtastversatz X der Ballkante (0 = links, 1 = rechts)

.const MAP     = $c000                  ; Steinkarte im RAM
; Ballspeicher: bis zu 3 Baelle (Mehrball).  Je Eigenschaft 3 Byte (Schlitz 0..2).  load_ball/save_ball kopieren einen Ball in die
; Nullseite (bxl..vyh), mit der die Bewegungs- und Kollisionsroutinen arbeiten.
.const shact   = $c120                  ; Laserschuesse (2 Stueck): 1 = aktiv
.const shu     = $c122                  ;   X-Lage in Einheiten zu 2 px (px / 2, damit 8 Bit reichen: max. 150)
.const shy     = $c124                  ;   Y-Lage (Oberkante) in px
.const bact    = $c100                  ; 1 = Ball aktiv
.const b_xl    = $c103
.const b_xh    = $c106
.const b_yl    = $c109
.const b_yh    = $c10c
.const b_vxl   = $c10f
.const b_vxh   = $c112
.const b_vyl   = $c115
.const b_vyh   = $c118
.const SCRQ = $d000                     ; Bildschirmspeicher im Chip-RAM: $0001_D000 (8192 Byte; 320 x 200: 40 x 25 Zellen, 640 x 400: 80 x 50 Zellen, je 2 Byte)
.const sq      = $a0                    ; 32-Bit-Zeiger auf die Bildschirmzelle (sq + 2 = 1, sq + 3 = 0), benutzt mit [sq],z
.const sr      = $a4                    ; zweiter 32-Bit-Zeiger (Lauftext schieben)
.const HB_BASE = $0520                  ; Zeichennummern der hohen Aufloesung: Steine (3 Fassungen * 10 Gruppen * 8 Zeichen, Chip-RAM $14800)
.const HS_BASE = $0610                  ;   Wand (4 Zeichen), Logo-Block (4), voller Block (1), Chip-RAM $18400
.const HF_BASE = $0620                  ;   Schrift: 64 Zeichen zu je 4 Viertelzeichen, Chip-RAM $18800
.const SPTR = $87f8                     ; Sprite-Zeiger (Ende des Bildschirmspeichers)
.const COLORRAM = $d800
.const PADY    = 184                    ; obere Kante des Schlaegers in px (Bildschirmzeile 23)
.const PADW    = 32                     ; Schlaegerbreite in px (Prototyp fest)
.const LIVES0  = 3                      ; Anzahl Baelle zu Spielbeginn

; Zeichencodes (Bildschirmcode) der eigenen Zeichen
.const FCM_BASE   = $0480                 ; Nummer des ersten Vollfarbzeichens (Steine): Daten bei Nummer * 64 = $12000 (Chip-RAM, hinter den Samples)
.const CH_WALL    = 48                    ; Vollfarbzeichen (Nummer FCM_BASE + n): 48 = Wand (Stahl), 49 = glaenzender Block fuer das Titellogo
.const CH_BLOCK   = 49
.const CH_SPACE   = 32

; Vorab: Bibliotheken (Hardware-Konstanten, Eingabe, Zufall, Klang, Bestenliste) aus data/sys/c64/
#include <mega65/hardware.asm>
#include <mega65/gs4502.asm>   ; Makro mega65_enable_vic4
#include <mega65/kernal.asm>   ; (enthaelt eine Konstante SCREEN; unsere Bildschirmadresse heisst deshalb SCR)

.org $2001                              ; Start des BASIC-65-Speichers (am C64 $0801)
.basicstub start                        ; erzeugt "10 SYS <start>", damit RUN das Programm startet

; ---------------------------------------------------------------------------------------------------------------------
; Programmstart
; ---------------------------------------------------------------------------------------------------------------------
start:  lda $01                         ; Zustand des BASIC-65-Starts merken (die Bestenliste blendet das KERNAL spaeter wieder ein, siehe hiscore.asm)
        sta hs_k01
        lda $d030
        sta hs_kd030
        jsr hs_load_kernal              ; Bestenliste laden, solange das KERNAL noch sichtbar ist; bei Fehlern bleibt die Standardliste
        sei                             ; Interrupts aus: Wir brauchen weder die KERNAL-Uhr noch deren Tastaturabfrage; wir lesen
                                        ; die Tastatur selbst (input.asm) und takten uns ueber die Rasterzeile (wait_frame).
        jsr mega_init                   ; MEGA65: BASIC-65-Umgebung wegraeumen, VIC-IV im VIC-II-Modus einstellen (siehe unten)
.ifdef TESTHS_M
        jsr hs_test
.endif
        lda #$7f                        ; Bit 7 = 0: die gesetzten Bits 0..6 LOESCHEN Interruptquellen der CIA
        sta CIA1_ICR                    ; CIA 1: alle Interruptquellen aus (Timer, Tastatur, ...)
        sta $dd0d                       ; CIA 2 (Port $DD0D) ebenso
        lda CIA1_ICR                    ; Lesen der Statusregister quittiert anstehende Meldungen
        lda $dd0d
        jsr copy_font                   ; Zeichensatz aus dem ROM ins RAM kopieren, eigene Zeichen einsetzen
        jsr fcm_init                    ; Vollfarb-Steine (FCM): Zeichendaten ins Chip-RAM, Palette
        jsr vic_init                    ; VIC-II einrichten
.ifdef TESTHS_V
        jsr hs_test
.endif
        jsr sfx_init                    ; SID: Lautstaerke 15, Stimme 3 fuer Effekte
.ifdef TESTDMA
        ldx #4                          ; Testfassung: den Knall (Verlust) einmal ueber DMA-Audio abspielen
        jsr dma_play
.endif
        jsr input_init                  ; Eingabezustand loeschen
.ifdef TESTHS_A
        jsr hs_test
.endif
.ifdef AUTOPLAY
        lda #1
        sta sfx_mute                    ; Demo-Modus: keine Soundeffekte (Vorgabe: nur die Musik spielt weiter)
.endif
        lda VIC_RASTER                  ; die aktuelle Rasterzeile ist als Startwert fuer den Zufall gut genug
        jsr rand_seed
        cli                             ; jetzt laeuft der Rasterinterrupt (Farbbaender, Bildtakt)
        ; weiter in die Hauptschleife

; ---------------------------------------------------------------------------------------------------------------------
; Hauptschleife: Titel -> Spiel -> Spielende -> Titel
; ---------------------------------------------------------------------------------------------------------------------
main_jump:
        jmp main
main:   jsr title                       ; Titelbild; kehrt zurueck, sobald Feuer gedrueckt wurde
        jsr new_game                    ; Punkte, Leben, Level, Geschwindigkeit zuruecksetzen
.level: jsr load_level                  ; Steine/Waende/Anzeige aufbauen, Ball auf den Schlaeger legen
.ball:  jsr play                        ; Rueckgabe in A: 0 = Level geschafft, 1 = Ball verloren, 2 = Abbruch (RUN/STOP)
        cmp #2
        beq main                        ; Abbruch: zurueck zum Titel
        cmp #1
        beq .lost
        lda demo
        beq +
        lda level
        cmp #4
        bcs main_jump                   ; Demo zeigt nur die Levels 1-5
+       inc level                       ; Level geschafft: naechstes Level, nach dem letzten wieder das erste
        lda level                       ; (die Geschwindigkeitsstufe bleibt erhalten: das Spiel wird immer schneller)
        cmp #level_count
        bcc .level
        lda #0
        sta level
        beq .level                      ; (immer: A = 0)
.lost:  lda lkind
        cmp #1
        bne .loselife
        jsr place_ball                  ; Bonuslevel: ein verlorener Ball kostet kein Leben, ein neuer haftet am Schlaeger
        jmp .ball
.loselife:
        dec lives
        beq .over                       ; keine Baelle mehr -> Spielende
        jsr reset_extras                ; Extras enden mit dem Ball
        jsr place_ball                  ; neuer Ball haftet auf dem Schlaeger
        jsr draw_hud                    ; Ballanzeige aktualisieren
        jmp .ball
.over:  jsr game_over
        jmp main

; ---------------------------------------------------------------------------------------------------------------------
; mega_init: aus der BASIC-65-Umgebung in ein schlichtes 64-KB-RAM mit Ein-/Ausgabe bei $D000 wechseln und den VIC-IV so
;   einstellen, dass er sich wie ein VIC-II verhaelt (wie hal_init im Beispiel jumprun).
;   map mit A = X = Y = Z = 0 hebt alle Umblendungen auf; eom beendet die Interruptsperre des map.  $01 = $35: kein ROM, RAM bis
;   $FFFF, Ein-/Ausgabe bei $D000.  Der VIC-IV wird mit $47, $53 nach $D02F freigeschaltet; danach: keine C65-ROM-Fenster ($D030),
;   40 Spalten ($D031, $D058, $D05E), normale Sprites ($D054).
; ---------------------------------------------------------------------------------------------------------------------
mega_init:
        lda #$00
        tax
        tay
        taz
        map
        eom
        lda #$35
        sta $01
        mega65_enable_vic4
        lda #0
        sta $d030
        lda #$40
        sta $d031                       ; FAST (C65): wird bei jedem Schreiben auf $D011/$D016/$D018 aus $D031 neu gesetzt; ohne dieses Bit fiele die Geschwindigkeit zurueck
        sta $d054                       ; VFAST: 40,5 MHz (der DMA-Audio braucht freie Prozessorzyklen); alle anderen Bits 0
        lda #0
        sta $d059
        lda #40
        sta $d058
        sta $d05e
        ; ---- VIC-IV-Palette: Bit 2 von $D030 (PAL) schaltet auf die Registerpalette um; $D070 = 0: alle Palettenbaenke 0.  Eintraege 0-15 (im
        ; VIC-II-Modus sichtbar) aus lib/gradient.inc; die Register $D100/$D200/$D300 (Rot/Gruen/Blau) nehmen die Werte mit vertauschten Halbbytes.
        lda #0
        sta $d070
        lda $d030
        ora #$04
        sta $d030
        ldx #15
.pal:   lda pal_r,x
        sta $d100,x
        lda pal_g,x
        sta $d200,x
        lda pal_b,x
        sta $d300,x
        dex
        bpl .pal
        ; ---- Sprite-Palette = Bank 1 (MEGA65 Chipset Reference, "Sprite Palette Bank"): $D070 Bits 6-7 = Bank, der in $D100-$D3FF erscheint,
        ; Bits 2-3 = Bank der Sprites.  Eintraege 0-15: dieselben Farben wie das Bild (fuer normale Sprites); dazu je Sprite n ab Eintrag
        ; n * 16 die Rampe der 16-Farb-Kugel (Sprites 3, 4, 5 = Baelle).  Danach wird Bank 0 wieder in $D100-$D3FF abgebildet (der Interrupt braucht sie).
        lda #$44
        sta $d070
        ldx #15
.pal1:  lda pal_r,x
        sta $d100,x
        lda pal_g,x
        sta $d200,x
        lda pal_b,x
        sta $d300,x
        lda spal_r,x
        sta $d130,x
        sta $d140,x
        sta $d150,x
        lda spal_g,x
        sta $d230,x
        sta $d240,x
        sta $d250,x
        lda spal_b,x
        sta $d330,x
        sta $d340,x
        sta $d350,x
        lda shot_r,x                    ; Laserschuesse, Sprites 6 und 7 (Eintraege 96-127)
        sta $d160,x
        sta $d170,x
        lda shot_g,x
        sta $d260,x
        sta $d270,x
        lda shot_b,x
        sta $d360,x
        sta $d370,x
        lda ppal_r,x                    ; Schlaegersprites 1 und 2 (Eintraege 16-47)
        sta $d110,x
        sta $d120,x
        lda ppal_g,x
        sta $d210,x
        sta $d220,x
        lda ppal_b,x
        sta $d310,x
        sta $d320,x
        dex
        bpl .pal1
        lda #$04
        sta $d070                       ; Bank 0 wieder in $D100-$D3FF; Sprites benutzen Bank 1
        lda $dd02
        ora #$03
        sta $dd02                       ; CIA 2 Port A, Bits 0-1 als Ausgang (VIC-Bankwahl): am C64 macht das das KERNAL, hier nicht
        rts

; ---------------------------------------------------------------------------------------------------------------------
; copy_font: kopiert den Zeichensatz (ROM, 2 KB) nach $4800 und setzt die eigenen Zeichen 64..66 ein.
;   Das Zeichen-ROM liegt bei $D000-$DFFF, ist aber nur sichtbar, wenn im Prozessorport ($01) das Bit CHAREN (Bit 2) geloescht
;   ist (statt der Ein-/Ausgabechips).  Der VIC sieht das ROM unabhaengig davon (Bank 0: $1000-$1FFF).
;   Zeichen n beginnt bei Zeichensatz + n * 8.  Nach der Kopie gilt: 0..26 = Buchstaben, 48..57 = Ziffern, 32 = Leerzeichen.
;   Eigene Zeichen (aus lib/gfx.inc.chars, alle MULTICOLOR): 64 = Stein links, 65 = Stein rechts, 66 = Wand, 67 = Block (Logo).
; Veraendert A, X, Y, sp, cp.
; ---------------------------------------------------------------------------------------------------------------------
copy_font:
        ; MEGA65: Das C64-Zeichen-ROM liegt im Chip-Speicher bei $2D000 (nicht im Adressraum der CPU).  Wir lesen es ueber einen
        ; 32-Bit-Zeiger (Nullseite $E0-$E3, Z = 0) und kopieren die ersten 2 KB (Grossbuchstaben und Grafik) nach $8800.
        lda #$00
        sta $e0                         ; Zeiger $0002_D000: Low-Byte 0 ...
        lda #$d0
        sta $e1                         ; ... $D0
        lda #$02
        sta $e2                         ; ... Bank 2
        lda #$00
        sta $e3
        taz                             ; Z = 0: Index der 32-Bit-Zugriffe
        lda #<$8800
        sta cp
        lda #>$8800
        sta cp + 1
        ldx #8                          ; 8 Seiten a 256 Byte = 2048 Byte
        ldy #0
.cf:    tya
        taz                             ; Z = Y: Index im 32-Bit-Zeiger
        lda [$e0],z                     ; Zeichen-ROM-Byte
        sta (cp),y
        iny
        bne .cf                         ; eine Seite
        inc $e1                         ; naechste Seite im ROM
        inc cp + 1                      ; ... und im Ziel
        dex
        bne .cf
        ldz #0
        ldx #31                         ; 4 Zeichen a 8 Byte (64..67), rueckwaerts kopieren
.cc:    lda gfx_chars,x
        sta $8a00,x                     ; $4800 + 64 * 8 = $4A00: Zeichen 64
        dex
        bpl .cc
        rts

; ---------------------------------------------------------------------------------------------------------------------
; vic_init: Grafikchip einrichten.
;   $DD00 Bit 0..1 waehlen die VIC-Bank, INVERTIERT: %11 = Bank 0 ($0000-$3FFF).  Die uebrigen Bits (Seriellbus) bleiben.
;   $D018: Bits 4..7 = Bildschirmspeicher / 1 KB (innerhalb der Bank, hier 1 -> $4400), Bits 1..3 = Zeichensatz / 2 KB (hier 1 -> $4800):
;          Wert = (1 << 4) | (1 << 1) = $12.
;          Wert = (1 << 4) | (4 << 1) = $18.
;   Sprites: Die Zeiger stehen am Ende des Bildschirmspeichers ($07F8..); Zeiger * 64 = Adresse der Sprite-Daten.
; ---------------------------------------------------------------------------------------------------------------------
vic_init:
        lda $dd00
        and #$fc
        ora #$01                        ; Bits 0..1 = %01: VIC-Bank 2 ($8000-$BFFF)
        sta $dd00
        lda #$12                        ; Bildschirm $4400 = Bank + $0400 (1 << 4), Zeichensatz $4800 = Bank + $0800 (1 << 1)
        sta VIC_MEMORY
        lda #$1b                        ; Textmodus, Bildschirm an, 25 Zeilen, Y-Scroll 3 (Standardwert)
        sta VIC_CTRL1
        lda #$18                        ; 40 Spalten, MEHRFARB-Textmodus (Bit 4), X-Scroll 0.  Zeichen mit Farb-RAM-Wert >= 8 sind
        sta VIC_CTRL2                   ; Mehrfarbzeichen (4 breite Pixel zu 2 Bit), Werte 0..7 bleiben normale Hires-Zeichen (Text)
        lda #15                         ; Mehrfarbe 1 ($D022): hellgrau = Lichtkante der Steine
        sta VIC_BGCOLOR1
        lda #11                         ; Mehrfarbe 2 ($D023): dunkelgrau = Schattenkante
        sta VIC_BGCOLOR2
        lda #0
        sta VIC_BGCOLOR0
        jsr border_black
        sta VIC_SPR_PRIORITY            ; Sprites vor dem Hintergrund
        sta VIC_SPR_ENABLE              ; erst im Spiel einschalten
        lda #$d4                        ; Sprite 0 = Kapsel (16-Farb-Sprite, $B500 = Block $D4); die Baelle sind die 16-Farb-Sprites 3, 4, 5
        sta SPTR
        lda #$c6                        ; Schlaeger links (16-Farb-Sprite, $B180 = Block $C6)
        sta SPTR + 1
        lda #$c9                        ; Schlaeger rechts ($B240 = Block $C9)
        sta SPTR + 2
        lda #$e3                        ; Sprites 6 und 7: Laserschuesse (16-Farb-Sprite, $B8C0 = Block $E3)
        sta SPTR + 6
        sta SPTR + 7
        lda #0                          ; Spritefarbe 0 = durchsichtig (16-Farb-Modus)
        sta VIC_SPR0_COLOR
        sta VIC_SPR6_COLOR
        sta VIC_SPR7_COLOR
        lda #$c0                        ; Sprites 3, 4, 5 = Baelle als 16-Farb-Kugel ($B000 = Block $C0 in Bank 2)
        sta SPTR + 3
        sta SPTR + 4
        sta SPTR + 5
        lda #0                          ; Spritefarbe 0: im 16-Farb-Modus ist dieser Farbwert DURCHSICHTIG (die Kugel faerbt die Palette)
        sta VIC_SPR1_COLOR
        sta VIC_SPR2_COLOR
        sta VIC_SPR3_COLOR
        sta VIC_SPR4_COLOR
        sta VIC_SPR5_COLOR
        lda #$ff
        sta $d06b                       ; VIC-IV: alle acht Sprites im 16-Farb-Modus (16 Pixel breit, 4 Bit je Pixel)
        lda #0                          ; keine Mehrfarb- oder Hires-Sprites mehr
        sta VIC_SPR_MULTICOLOR
        lda #15                         ; Sprite-Mehrfarbe 1 ($D025): hell (Lichtkante)
        sta VIC_SPR_MCOLOR0
        lda #6                          ; Sprite-Mehrfarbe 2 ($D026): dunkelblau (Schatten)
        sta VIC_SPR_MCOLOR1
        ; Rasterinterrupt: Hintergrundfarbe in Baendern wechseln (siehe irq).  Die KERNAL-Interruptroutine ($FF48) ruft ueber den
        ; Vektor $0314/$0315 unsere Routine; CIA-Interrupts sind aus, nur der Rasterinterrupt des VIC-II ist aktiv.
        lda #<irq                       ; MEGA65: kein KERNAL-Einsprung ($01 = $35); der Prozessor liest den Vektor direkt aus $FFFE/$FFFF (RAM)
        sta $fffe
        lda #>irq
        sta $ffff
.ifdef TESTSTRIPES
        lda #3
.else
        lda #0
.endif
        jsr set_theme                   ; Verlauf 0
        jsr mode_set                    ; Betriebsart (320 x 200 oder 640 x 400), 16-Bit-Zeichennummern, Bildschirm bei $1D000
        jsr irq_first                   ; erste Interruptzeile
        lda #1
        sta VIC_IRQ_ENABLE              ; Rasterinterrupt freigeben
        lda #0
        sta band
        sta band3
        sta vblank
        rts

; ---------------------------------------------------------------------------------------------------------------------
; seam_regs: VIC-IV auf "Super-Extended Attribute Mode" (SEAM) stellen: jede Textzelle hat 2 Byte Bildschirm und 2 Byte Farb-RAM, die
;   Zeichennummer ist 16 Bit.  Nummern ab 256 sind Vollfarbzeichen (FCM, 64 Byte je Zeichen bei Nummer * 64 im Chip-RAM), kleinere
;   bleiben normale VIC-II-Zeichen.  Jedes Schreiben auf $D011/$D016/$D018/$D031/$DD00 (heisse Register) setzt diese Register
;   zurueck; deshalb wird diese Routine danach jedes Mal aufgerufen.  Veraendert A.
;   $D054: Bit 0 CHR16, Bit 2 FCLRHI, Bit 6 VFAST (40,5 MHz).  $D058/$D059 = 80 Byte je Zeile, $D05E = 40 Zeichen je Zeile.
;   $D060-$D062 = Bildschirm $00C800, $D064/$D065 = Farb-RAM-Versatz 0, $D06C-$D06E = Sprite-Zeiger bei $87F8 (Bank 2).
; ---------------------------------------------------------------------------------------------------------------------
seam_regs:
        lda #$45
        ora spr_h                       ; Bit 4 = SPRH640: Sprites in der Breite der echten Bildpunkte
        sta $d054
        lda hires
        bne .sg1
        lda #80
        sta $d058
        lda #40
        sta $d05e
        bne .sg2
.sg1:   lda #160                        ; hohe Aufloesung: 160 Byte je Zeile, 80 Zeichen je Zeile
        sta $d058
        lda #80
        sta $d05e
.sg2:   lda #0
        sta $d059
        sta $d060
        lda #>SCRQ
        sta $d061
        lda #1
        sta $d062                       ; Bildschirm bei $01_D000
        lda #0
        sta $d064
        sta $d065
        lda #<SPTR
        sta $d06c
        lda #>SPTR
        sta $d06d
        lda #0
        sta $d06e
        rts

; mode_set: Betriebsart einstellen: hires = 0: 320 x 200, hires = 1: 640 x 400 ($D031: Bit 7 H640, Bit 3 V400, Bit 6 FAST); danach die
;   VIC-IV-Register (heisses Register $D031).
mode_set:
        lda hires
        beq .ms0
        lda #49                         ; Lauftext in der letzten Zeichenzeile (Rasterzeile 2 * 247 = 494)
        sta scrband
        lda #$53                        ; Seitenrand 3 Punkte breiter: der Zeichenbereich der hohen Aufloesung sitzt 3 Punkte links vom
        sta $d05c                       ; Bildrand, rechts blieb ein Streifen des Hintergrundverlaufs sichtbar
        lda #$c8
        bne .ms1
.ms0:   lda #BAND_SCROLL
        sta scrband
        lda #$50                        ; Standardbreite des Seitenrands
        sta $d05c
        lda #$40
.ms1:   sta $d031
        jsr spr_mode
        jmp seam_regs

; irq_first: Verlauf von vorn: Band 0, erste Interruptzeile (640 x 400: doppelte Zeile, Bit 8 aus).
irq_first:
        lda #0
        sta band
        sta band3
        sta rhi
        lda bar_line
        ldx hires
        beq .if0
        asl a
.if0:   sta VIC_RASTER
        lda #$1b
        sta VIC_CTRL1
        jmp seam_regs
rhi:    .byte 0                         ; Bit 8 der eingestellten Interruptzeile
scrband: .byte 47                       ; Band, in dem der Feinversatz des Lauftextes gesetzt wird
.ifdef START_HIRES
hires:  .byte 1                         ; Testfassung -D START_HIRES=1: Start in 640 x 400
.else
hires:  .byte 0
.endif
spr_h:  .byte 0

; ---------------------------------------------------------------------------------------------------------------------
; Rasterinterrupt: Hintergrund-Farbbaender (Sonnenuntergang hinter dem Spielfeld) und Bildtakt.
;   Der Interrupt feuert an den Zeilen bar_line[0..7]; jedes Mal wird die Hintergrundfarbe $D021 auf die naechste Bandfarbe gesetzt
;   (sie wirkt bei allen Pixeln mit Bitmuster 00: Luecken zwischen den Steinen und der leere Spielraum).  Beim letzten Interrupt
;   (Zeile 251, unterhalb des sichtbaren Bildes) kommt die Farbe wieder auf Schwarz fuer die Kopfzeile des naechsten Bildes
;   und vblank wird gesetzt: das Hauptprogramm darf das naechste Bild berechnen (wait_frame).
;   Quittieren: Schreiben einer 1 nach $D019 loescht das Interruptflag.  Ende ueber $EA81 (Register zurueck, rti).
; ---------------------------------------------------------------------------------------------------------------------
irq:    pha                             ; Register sichern (ohne KERNAL muss die Routine das selbst tun)
        txa
        pha
        tya
        pha
        lda #1
        sta VIC_IRQ_FLAGS
        ldy band
        cpy #NB
        beq .ifin                       ; Band NB = Bildende (Zeile 251): kein Farbwechsel mehr
        lda bar_line,y                  ; X = Zeile T des Bandes, in der der Interrupt ausgeloest wurde
        ldx hires
        beq .iwx
        asl a                           ; 640 x 400: Rasterzeilen zaehlen die echten Bildzeilen: L = 2 * T (immer gerade, nie 255)
.iwx:   tax
.iw:    cpx VIC_RASTER                  ; STABILE FARBKANTE: nicht sofort schreiben (der Interrupt kommt je nach laufendem Befehl um bis zu
        bcs .iw                         ; 7 Takte versetzt), sondern warten, bis die Zeile T+1 beginnt (Schleife: 7 Takte; solange X >= Zeile).
                                        ; Dann liegt der Schreibzugriff in den ersten Takten der Zeile: die Kante zwischen zwei Baendern ist
                                        ; eine saubere Waagerechte.  Die Schreibzeilen T+1 sind 0 oder 4 (mod 8), nie eine "Bad Line" (3 mod 8).
        ldy band3                       ; MEGA65: Palettenfarbe 0 (Hintergrund und Rahmen) auf den RGB-Wert dieses Bandes setzen (je 3 Byte)
        lda (bp),y
        sta $d100                       ; Rot (Halbbytes vertauscht, siehe tools/gen_gradient.py)
        iny
        lda (bp),y
        sta $d200                       ; Gruen
        iny
        lda (bp),y
        sta $d300                       ; Blau
        ldy band
        cpy scrband
        bne .i4
        lda scrollreg                   ; Band bei T = 239: Feinversatz des Lauftextes ab Zeile 240 (die Textzeile 24 beginnt bei 243;
        sta VIC_CTRL2                   ; die Zeilen 240-242 gehoeren noch zur leeren Bildschirmzeile 23)
        jsr seam_regs                   ; (ein Schreiben auf $D016 setzt die VIC-IV-Register zurueck: Zeilenbreite usw. neu einstellen)
        jmp .i4
.ifin:  lda #$18                        ; Zeile 251: Versatz wieder 0 (Mehrfarbtext, 40 Spalten), Hintergrund schwarz, Bildtakt
        sta VIC_CTRL2
        jsr seam_regs
        ldy band3                       ; letzter Tabelleneintrag des Verlaufs = Schwarz
        lda (bp),y
        sta $d100
        iny
        lda (bp),y
        sta $d200
        iny
        lda (bp),y
        sta $d300
        ldy band
        lda #1
        sta vblank
.i4:    iny
        cpy #NB + 1
        bcc +
        ldy #0
+       sty band
        tya
        asl a
        clc
        adc band
        sta band3                       ; band3 = 3 * band: Index in die RGB-Tabelle
        lda bar_line,y
        ldx hires
        beq .nl0
        asl a                           ; 640 x 400: Zeile L = 2 * T, Bit 8 im Uebertrag
        sta VIC_RASTER                  ; naechste Interruptzeile (Bit 0..7)
        lda #0
        rol a                           ; A = Bit 8 der Zeile
        cmp rhi
        beq .nl1                        ; Bit 8 unveraendert: $D011 (heisses Register) nicht anfassen
        sta rhi
        lda rhi                         ; (sta setzt keine Flags)
        beq .nl2
        lda #$9b                        ; Bit 7 von $D011 = Bit 8 der Rasterzeile
        bne .nl3
.nl2:   lda #$1b
.nl3:   sta VIC_CTRL1
        jsr seam_regs
        jmp .nl1
.nl0:   sta VIC_RASTER                  ; naechste Interruptzeile
.nl1:   pla                             ; Register zurueckholen
        tay
        pla
        tax
        pla
        rti

; Interruptzeilen und Farbtabellen: lib/gradient.inc (erzeugt von tools/gen_gradient.py): NB = 50 Baender zu je 4 Rasterzeilen von T = 51
; bis 247, danach Bildende bei 251.  Je Verlauf (NB + 1) * 3 Byte (Rot, Gruen, Blau je Band; die ersten 4 und die letzten Baender schwarz).
;   0 Daemmerung (blau - violett - rot), 1 Wald (grau - gruen - braun), 2 Glut (braun - rot - orange), 3 Nacht (Streifen)
.const BAND_SCROLL = 47                 ; das Band mit T = 239 stellt den Feinversatz des Lauftextes ein
.include "lib/gradient.inc"
scrollreg: .byte $18

; set_theme: Verlauf A (0..3) waehlen: bp = bar_themes + 23 * A.   Veraendert A, X.
set_theme:
        tax
        lda #<grad_themes
        sta bp
        lda #>grad_themes
        sta bp + 1
        cpx #0
        beq .st9
.st1:   clc
        lda bp
        adc #(NB + 1) * 3
        sta bp
        bcc +
        inc bp + 1
+       dex
        bne .st1
.st9:   rts

; flash: Rand kurz in Farbe A faerben fuer X Bilder (fx_frame zaehlt herunter und stellt danach Schwarz her).
; hs_after: nach einem Dateizugriff (hiscore.asm) den Grafikchip neu einrichten: das KERNAL kann Register des VIC veraendert haben.
hs_after:
        jsr vic_init                    ; Bank, Speicher, Sprites, Betriebsart, Rasterinterrupt (setzt auch den Interruptzaehler zurueck)
        rts

; mode_key: Taste R (Spalte 2, Zeile 1 der Tastaturmatrix) abfragen; Carry = 1 genau beim Druck (Flanke).  Veraendert A, X.
mode_key:
        lda #$ff
        sta CIA1_DDRA
        lda #$00
        sta CIA1_DDRB
        lda #%11111011                  ; Spalte 2 waehlen
        sta CIA1_PRA
        lda CIA1_PRB
        and #%00000010                  ; Zeile 1: R (0 = gedrueckt)
        tax
        lda #$ff
        sta CIA1_PRA                    ; Tastaturspalten wieder frei
        txa
        bne .mk0
        lda mk_prev
        bne .mk9                        ; noch gehalten: keine neue Flanke
        lda #1
        sta mk_prev
        sec
        rts
.mk0:   lda #0
        sta mk_prev
.mk9:   clc
        rts
mk_prev: .byte 0

; toggle_mode: Aufloesung umschalten: Betriebsart, Rand, Interruptzeilen; der Bildschirm muss danach neu gezeichnet werden.
toggle_mode:
        php
        sei
        lda hires
        eor #1
        sta hires
        jsr mode_set
        lda hires                       ; Steindaten der neuen Betriebsart erzeugen (die der anderen dienen als Streifenspeicher fuer wandernde Steine)
        beq .tm1
        jsr hires_init
        jmp .tm2
.tm1:   jsr fcm_init
.tm2:   jsr border_black
        jsr irq_first
        plp
        rts

; toggle_game: im Spiel umschalten und das Bild neu aufbauen (Wand, Steine, Kopfzeile, Statuszeile).
toggle_game:
        jsr wander_settle               ; wandernde Steine zurueck an ihre Quelle (die Streifen-Zeichen gehen mit der Umschaltung verloren)
        jsr toggle_mode
        jsr crumble_reset
        jsr clear_screen
        jsr draw_walls
        jsr draw_map
        jsr draw_hud
        jsr draw_status
        lda demo
        beq .tg1
        lda #0                          ; Demo: Ticker in der obersten Zeile
        ldx #1
        jsr ticker_init
.tg1:   rts

; border_black: Rand schwarz.  Palettenfarbe 0 ist der Hintergrundverlauf; in der hohen Aufloesung (Rand bis an das Spielfeld) wird dafuer der
;   schwarze Palettenwert 234 benutzt.  Veraendert A.
border_black:
        lda #0
        ldx hires
        beq .bb1
        lda #234
.bb1:   sta VIC_BORDERCOLOR
        rts

; Weiches Ausblenden: Palettenfarbe A wird in Eintrag FLASHPAL kopiert (Rand = dieser Eintrag), fx_frame blendet sie je Bild um ein Viertel ab.
;   Die Palettenregister haben vertauschte Halbbytes (nswap); fu[0..2] = aktueller Farbwert (Rot, Gruen, Blau) mit richtiger Reihenfolge.
.const FLASHPAL = 251
flash:  stx bflash
        tay
        lda pal_r,y
        jsr nswap
        sta fu
        lda pal_g,y
        jsr nswap
        sta fu + 1
        lda pal_b,y
        jsr nswap
        sta fu + 2
        jsr flash_write
        lda #FLASHPAL
        sta VIC_BORDERCOLOR
        rts

; nswap: Halbbytes von A vertauschen.  Veraendert A.
nswap:  sta fnt
        asl a
        asl a
        asl a
        asl a
        pha
        lda fnt
        lsr a
        lsr a
        lsr a
        lsr a
        sta fnt
        pla
        ora fnt
        rts

; flash_write: fu[0..2] in die Palette (Eintrag FLASHPAL) schreiben.   Veraendert A.
flash_write:
        lda fu
        jsr nswap
        sta $d100 + FLASHPAL
        lda fu + 1
        jsr nswap
        sta $d200 + FLASHPAL
        lda fu + 2
        jsr nswap
        sta $d300 + FLASHPAL
        rts

; flash_fade: jeden Wert um ein Viertel (mindestens 1) verringern und schreiben.  Veraendert A, X.
flash_fade:
        ldx #2
.ffl:   lda fu,x
        beq .ffn
        lsr a
        lsr a
        bne +
        lda #1
+       sta fnt
        lda fu,x
        sec
        sbc fnt
        sta fu,x
.ffn:   dex
        bpl .ffl
        jmp flash_write
fu:     .byte 0, 0, 0
fnt:    .byte 0

band:     .byte 0
band3:    .byte 0                       ; 3 * band (Index in die RGB-Tabelle des Verlaufs)
vblank:   .byte 0


; ---------------------------------------------------------------------------------------------------------------------
; Bildschirm-Hilfen
; ---------------------------------------------------------------------------------------------------------------------
; clear_screen: Bildschirm (8192 Byte im Chip-RAM ab $1D000) und Farb-RAM (8192 Byte) loeschen: gerade Stellen = Leerzeichen bzw. Eigenschaften 0,
;   ungerade = 0 bzw. Farbe 1 (weiss).  Reicht fuer beide Betriebsarten (40 x 25 und 80 x 50 Zellen).
clear_screen:
        lda #0
        sta sq
        lda #>SCRQ
        sta sq + 1
        lda #1
        sta sq + 2
        lda #0
        sta sq + 3
        ldx #32
.cs1:   ldz #0
.cs2:   lda #CH_SPACE
        sta [sq],z
        inz
        lda #0
        sta [sq],z
        inz
        bne .cs2
        inc sq + 1
        dex
        bne .cs1
        lda #0
        sta cq
        lda #$f8
        sta cq + 2
        lda #$0f
        sta cq + 3
        ldx #0
.cs3:   stx cq + 1
        ldz #0
.cs4:   lda #0
        sta [cq],z
        inz
        lda #1
        sta [cq],z
        inz
        bne .cs4
        inx
        cpx #32
        bne .cs3
        ldz #0
        rts

; set_rowptr: sq (Bildschirm) und cq (Farb-RAM) auf die logische Zelle Spalte X, Zeile Y setzen (40 x 25 logische Zellen).
;   320 x 200: eine Zelle = 1 Zeichen, Versatz = Zeile * 80 + Spalte * 2.   640 x 400: eine logische Zelle = 2 x 2 Zeichen, Versatz = Zeile * 320 +
;   Spalte * 4 (Zeichenzelle links oben).  Das Farb-RAM hat denselben Versatz ab $FF80000; dort ist Byte 0 = Eigenschaften, Byte 1 = Farbe.
; Eingabe: X = Spalte 0..39, Y = Zeile 0..24.  Veraendert A.
set_rowptr:
        lda hires
        bne .srh
        txa
        asl
        clc
        adc rowlo,y
        sta cq
        lda rowhi,y
        jmp .srx
.srh:   txa
        asl
        asl
        clc
        adc rowlo2,y
        sta cq
        lda rowhi2,y
.srx:   adc #0                          ; Uebertrag aus dem Low-Byte
        sta cq + 1
        clc
        adc #>SCRQ
        sta sq + 1
        lda cq
        sta sq
        lda #1
        sta sq + 2
        lda #$f8
        sta cq + 2
        lda #0
        sta sq + 3
        lda #$0f
        sta cq + 3
        rts

; row_next: sq und cq um eine Zeichenzeile der hohen Aufloesung (160 Byte) weiter.
row_next:
        lda sq
        clc
        adc #160
        sta sq
        bcc .rn1
        inc sq + 1
.rn1:   lda cq
        clc
        adc #160
        sta cq
        bcc .rn2
        inc cq + 1
.rn2:   rts

; put_cell: Zeichen A mit Farbe txtcol an Spalte X, Zeile Y ausgeben.   Veraendert A, X, Y, Z, t0, t1.
;   Hohe Aufloesung: Zeichen 0..63 (Bildschirmcodes) als Schrift aus 2 x 2 Vollfarbzeichen (HF_BASE + 4 * Code); Leerzeichen und 160 (voller Block)
;   als 2 x 2 Zellen.
put_cell:
        pha
        jsr set_rowptr
        pla
        ldx hires
        bne .pch
        ldz #0
        sta [sq],z
        lda txtcol
        inz
        sta [cq],z
        rts
.pch:   cmp #CH_SPACE
        beq .pcs
        cmp #160
        beq .pcb
        sta qn0
        lda #0
        sta qn1
        asl qn0
        rol qn1
        asl qn0
        rol qn1                          ; qn1:qn0 = 4 * Code
        lda qn0
        clc
        adc #<HF_BASE
        sta qn0
        lda qn1
        adc #>HF_BASE
        sta qn1
        jmp put_quad
.pcs:   lda #CH_SPACE
        sta qn0
        lda #0
        sta qn1
        jmp put_quad_same
.pcb:   lda #<(HS_BASE + 8)
        sta qn0
        lda #>(HS_BASE + 8)
        sta qn1
        jmp put_quad_same

; put_quad: 2 x 2 Zellen ab (sq) mit den Zeichennummern qn1:qn0 + 0..3 (links oben, rechts oben, links unten, rechts unten); Farbbyte = txtcol.
put_quad:
        ldx #0
.pq1:   lda q_off,x
        taz
        txa
        clc
        adc qn0
        sta [sq],z
        lda qn1
        adc #0
        inz
        sta [sq],z
        lda txtcol
        sta [cq],z
        inx
        cpx #4
        bne .pq1
        rts

; put_quad_same: wie put_quad, aber alle vier Zellen mit der Zeichennummer qn1:qn0.
put_quad_same:
        ldx #3
.pq2:   lda q_off,x
        taz
        lda qn0
        sta [sq],z
        lda qn1
        inz
        sta [sq],z
        lda txtcol
        sta [cq],z
        dex
        bpl .pq2
        rts

qn0:    .byte 0                         ; Zwischenwerte der Zeichenausgabe (nicht t0/t1: puts benutzt t1)
qn1:    .byte 0
q_off:  .byte 0, 2, 160, 162             ; Byteversatz der vier Zellen eines Vierecks (Zeile = 160 Byte)
b_off:  .byte 0, 2, 4, 6, 160, 162, 164, 166   ; die acht Zellen eines hochaufgeloesten Steins (4 x 2)

; puts: Text ab (tp), mit 0 abgeschlossen, an Spalte X, Zeile Y in Farbe txtcol ausgeben.   Veraendert A, X, Y, t1..t3.
puts:   stx t2                          ; t2 = aktuelle Spalte
        sty t3                          ; t3 = Zeile
        ldy #0
        sty t1                          ; t1 = Index im Text
.pl:    ldy t1
        lda (tp),y
        beq .pd                         ; 0 = Ende
        ldx t2
        ldy t3
        jsr put_cell
        inc t2
        inc t1
        bne .pl                         ; (Text kuerzer als 256 Zeichen)
.pd:    rts

; put_bcd: ein BCD-Byte als zwei Ziffern an Spalte X, Zeile Y (Ziffernzeichen = $30 + Wert).   Veraendert A, X, Y, t0, t2, t3.
put_bcd:
        sta t0
        stx t2
        sty t3
        lsr                             ; obere Ziffer: High-Nibble nach unten
        lsr
        lsr
        lsr
        clc
        adc #$30
        jsr put_cell                    ; (X und Y sind noch gueltig)
        lda t0
        and #$0f                        ; untere Ziffer
        clc
        adc #$30
        ldx t2
        inx                             ; eine Spalte weiter rechts
        ldy t3
        jmp put_cell

; Zeilenversatz: 80 * Zeile (Low- und High-Byte), Zeile 0..24 (2 Byte je Zelle, 40 Zellen)
rowlo:  .byte $00, $50, $a0, $f0, $40, $90, $e0, $30, $80, $d0, $20, $70, $c0, $10, $60, $b0, $00, $50, $a0, $f0, $40, $90, $e0, $30, $80
rowlo2: .byte $00, $40, $80, $c0, $00, $40, $80, $c0, $00, $40, $80, $c0, $00, $40, $80, $c0, $00, $40, $80, $c0, $00, $40, $80, $c0, $00
rowhi2: .byte $00, $01, $02, $03, $05, $06, $07, $08, $0a, $0b, $0c, $0d, $0f, $10, $11, $12, $14, $15, $16, $17, $19, $1a, $1b, $1c, $1e
rowhi:  .byte $00, $00, $00, $00, $01, $01, $01, $02, $02, $02, $03, $03, $03, $04, $04, $04, $05, $05, $05, $05, $06, $06, $06, $07, $07

; wait_frame: wartet auf den Anfang des Bildwechsels (Rasterzeile 251, unterhalb des sichtbaren Bildes).  So laeuft das Spiel
;   mit 50 Hz (PAL) und Sprites/Bildschirm werden aktualisiert, wenn der Strahl sie nicht gerade zeichnet.
wait_frame:
.w1:    lda vblank
        beq .w1                         ; der Rasterinterrupt setzt vblank in Zeile 251
        lda #0
        sta vblank
        rts

; wait_frames: X Bilder warten; dabei laeuft der Klangeffekt weiter.   Veraendert A, X.
wait_frames:
        stx wfc                         ; Zaehler im Speicher: sfx_update veraendert X (Zugriff auf sfx_id)
.wf:    jsr wait_frame
        jsr sfx_update
        dec wfc
        bne .wf
        rts

; HINWEIS ZU .screen: Der Assembler wandelt KLEINBUCHSTABEN in die Bildschirmcodes 1..26 (= Grossbuchstaben im Zeichensatz, den wir
; aus dem ROM kopieren); Grossbuchstaben ergaeben 65..90 (Grafikzeichen bzw. unsere eigenen Zeichen).  Deshalb stehen alle
; Anzeigetexte hier in Kleinbuchstaben und erscheinen auf dem Bildschirm in GROSSBUCHSTABEN.
; ---------------------------------------------------------------------------------------------------------------------
; Titelbild
; ---------------------------------------------------------------------------------------------------------------------
title:  jsr title_screen
        jmp title_cont
; title_screen: das Titelbild (neu) aufbauen (auch nach dem Umschalten der Aufloesung)
title_screen:
        jsr border_black
        jsr title_balls_init            ; drei Baelle huepfen ueber den Titel (Sprites 0, 4, 5)
        jsr clear_screen
        jsr draw_logo                   ; "PRALLBALL" aus glaenzenden Bloecken
        jsr title_scroll_init           ; Lauftext in der letzten Bildschirmzeile
        lda #10                         ; Untertitel: Ticker in Zeile 10 mit zwei wechselnden Texten
        ldx #0
        jsr ticker_init
        lda #7                          ; gelb
        sta txtcol
        lda #<t_best
        sta tp
        lda #>t_best
        sta tp + 1
        ldx #14
        ldy #12
        jsr puts
        jsr draw_top5                   ; die besten fuenf Eintraege der Liste
        lda #3                          ; tuerkis
        sta txtcol
        lda #<t_ctrl
        sta tp
        lda #>t_ctrl
        sta tp + 1
        ldx #4
        ldy #20
        jsr puts
        lda #<t_keys
        sta tp
        lda #>t_keys
        sta tp + 1
        ldx #7
        ldy #21
        jsr puts
        lda #1
        sta txtcol
        lda #<t_start
        sta tp
        lda #>t_start
        sta tp + 1
        ldx #10
        ldy #22
        jsr puts
        lda #5                          ; gruen: Hinweis auf die Umschaltung der Aufloesung
        sta txtcol
        lda #<t_res
        sta tp
        lda #>t_res
        sta tp + 1
        ldx #5
        ldy #18
        jsr puts
        rts
t_res:  .screen "taste r: aufloesung 320 / 640"
        .byte 0
; title_cont: Rest des Titelbilds nach dem Aufbau (title_screen): Demo/Effekte zuruecksetzen, Ticker und Lauftext starten, auf Feuer, R oder die Demo warten.
title_cont:
        lda #0                          ; normaler Titel: kein Demo-Modus, Effekte an
        sta demo
        sta sfx_mute
        ldx #<song                      ; Titelmusik von vorn starten (Liedformat: examples/c64/prallball/lib/sound.asm)
        ldy #>song
        jsr mus_init
        ldx #<song3                     ; zweite Spur (Stimmen 4..6 auf SID 3): Gegenstimme und Akkordtöne
        ldy #>song3
        jsr mus_init3
.ifdef AUTOPLAY
        lda #1                          ; Testfassung: der Rechner spielt von Anfang an und ohne Ende
        sta demo
        sta sfx_mute
.else
.ifndef TESTLOSE
        jsr input_wait_release          ; ein noch gedrueckter Knopf (vom vorigen Spiel) zaehlt nicht
        lda #40                         ; Schonfrist: in den ersten 0,8 s nach dem Erscheinen des Titels zaehlt keine Eingabe
        sta idl                         ; (Zaehler im Speicher, NICHT in X: mus_tick und title_balls_tick veraendern X!)
.gr:    jsr wait_frame                  ; (ein vom Ende der letzten Runde noch nachwirkender Tastendruck startet kein neues Spiel)
        jsr mus_tick
        jsr title_balls_tick
        jsr title_fx
        jsr ticker_tick
        dec idl
        bne .gr
        jsr input_wait_release
        lda #<1500
        sta idl                         ; 30 s Leerlauf (1500 Bilder bei 50 Hz) -> Demo-Modus
        lda #>1500
        sta idh
.tw:    jsr wait_frame
        jsr mus_tick                    ; Musik einen Bildschritt weiter
        jsr title_balls_tick
        jsr title_fx
        jsr ticker_tick
        jsr mode_key                    ; Taste R: Aufloesung umschalten (320 x 200 <-> 640 x 400)
        bcc .tw1
        jsr toggle_mode
        jsr title_screen
.tw1:   jsr input_poll
        sta mask
        and #IN_FIRE
        bne .start                      ; Feuer: Spiel starten
        lda mask
        beq .idle
        lda #<1500                      ; andere Eingabe: Wartezeit neu starten
        sta idl
        lda #>1500
        sta idh
        bne .tw
.idle:  lda idl
        ora idh
        beq .demo                       ; Zeit abgelaufen
        lda idl
        bne +
        dec idh
+       dec idl
        jmp .tw
.demo:  lda #1                          ; Demo-Modus: der Rechner spielt, die Soundeffekte schweigen (die Musik laeuft weiter)
        sta demo
        sta sfx_mute
        jmp .go                         ; die Musik laeuft in der Demo weiter
.start: jsr mus_stop                    ; Spielbeginn: die Musik schweigt (nur auf dem Titel und in der Demo)
.go:
.endif
.endif
        rts

; title_balls_init / title_balls_tick: drei Mehrfarb-Baelle (Sprites 0, 4, 5) prallen auf dem Titel von den Raendern ab (reine Zierde).
;   Position X in Einheiten zu 2 px (u, 8 Bit reicht fuer das ganze Bild), Y in px; Sprite-X = 2 * u + 24, Sprite-Y = y + 50.
; title_fx: Zierde des Titels, einmal je Bild: Farbwechsel im Logo (alle 4 Bilder ein Schritt) und der Lauftext.
title_fx:
        jsr title_scroll_tick
        inc lcnt
        lda lcnt
        and #3
        bne .tf9
        inc lph                         ; Farbphase weiter
        lda #0
        sta t1                          ; Logozeile 0..4
.tf1:   lda lph
        clc
        adc t1
        and #7
        tax
        lda rainbow,x                   ; Farb-RAM-Wert (8 + Farbe) dieser Zeile
        sta txtcol
        ldx #2
        lda t1
        clc
        adc #3
        tay                             ; Bildschirmzeile = 3 + Logozeile
        jsr set_rowptr                  ; cq zeigt auf das Farb-RAM (Zeile, Spalte 2)
        lda txtcol
        and #7                          ; Farbe 0..7 (Logo-Bloecke sind Vollfarbzeichen: Pixelwert 255 = Farbe aus dem Farb-RAM)
        sta tf_c
        ldx #36                         ; Spalten 2..37 einfaerben (nur die Bloecke sind sichtbar)
        lda hires
        beq .tfa
        ldx #72                         ; hohe Aufloesung: doppelt so viele Zellen in zwei Zeichenzeilen
.tfa:   stx tf_n
        jsr col_row
        lda hires
        beq .tfb
        lda cq
        clc
        adc #160
        sta cq
        bcc .tfc
        inc cq + 1
.tfc:   ldx tf_n
        jsr col_row
.tfb:
        inc t1
        lda t1
        cmp #5
        bne .tf1
.tf9:   rts
; col_row: X Zellen ab (cq) mit der Farbe tf_c versehen (Farbbyte jeder Zelle).
col_row:
        ldz #1
        lda tf_c
.cr0:   sta [cq],z
        inz
        inz
        dex
        bne .cr0
        rts
tf_c:   .byte 0
tf_n:   .byte 0
rainbow: .byte 10, 15, 13, 11, 14, 12, 9, 15            ; rot, gelb, gruen, tuerkis, blau, violett, weiss, gelb (je 8 + Farbe)

; Lauftext in Bildschirmzeile 24: weicher Versatz ueber $D016 (Rasterinterrupt in Zeile 242 setzt ihn, siehe irq), nach 8 Bildpunkten
;   rutscht die Zeile um ein Zeichen nach links und rechts kommt das naechste Zeichen des Textes nach.
title_scroll_init:
        lda #7
        sta scx
        lda #$1f
        sta scrollreg
        ldx #0
        ldy #24
        jsr set_rowptr                  ; Zeile 24 (hohe Aufloesung: die obere der zwei Zeichenzeilen)
        lda hires
        beq .si0
        jsr row_next                    ; hohe Aufloesung: Lauftext in der unteren Zeichenzeile (Zeile 49), 80 Zeichen
        lda #158
        sta sc_n
        ldx #79
        ldy #158                        ; Y = 2 * X
        bne .si1
.si0:   lda #78
        sta sc_n
        ldx #39
        ldy #78
.si1:   tya
        taz
        lda scroll_text,x               ; die ersten Zeichen des Textes (hinten beginnend)
        sta [sq],z
        inz                             ; Z = Farbbyte der Zelle
        lda #7                          ; gelb (Farbe < 8: normaler Text, kein Mehrfarbzeichen)
        sta [cq],z
        dey
        dey
        dex
        bpl .si1
        lda hires
        beq .si2
        lda #<(scroll_text + 80)        ; Leseposition hinter den ersten 80 Zeichen
        sta stq
        lda #>(scroll_text + 80)
        sta stq + 1
        rts
.si2:   lda #<(scroll_text + 40)        ; Leseposition hinter den ersten 40 Zeichen
        sta stq
        lda #>(scroll_text + 40)
        sta stq + 1
        rts
sc_n:   .byte 78

; title_scroll_tick: einmal je Bild den Lauftext der untersten Zeile um einen Punkt weiterschieben (Feinversatz scx, bei 0 ein Zeichen weiter).
title_scroll_tick:
        dec scx
        lda hires
        beq .st0
        dec scx                         ; hohe Aufloesung: 2 Bildpunkte je Bild (ein Zeichen ist nur 8 Bildpunkte breit)
.st0:   lda scx
        bpl .sk1                        ; noch im Feinversatz
        lda #7
        sta scx
        ldx #0
        ldy #24
        jsr set_rowptr
        lda hires
        beq .sh0
        jsr row_next
.sh0:   lda sq                          ; sr = sq + 2 (Quelle: Zelle rechts daneben)
        clc
        adc #2
        sta sr
        lda sq + 1
        adc #0
        sta sr + 1
        lda #1
        sta sr + 2
        lda #0
        sta sr + 3
        ldz #0
        ldx sc_n
.sh1:   lda [sr],z                      ; Zeile um ein Zeichen nach links schieben
        sta [sq],z
        inz
        dex
        bne .sh1
        ldy #0
        lda (stq),y                     ; naechstes Zeichen
        bne .sk3
        lda #<scroll_text               ; Textende (0): von vorn
        sta stq
        lda #>scroll_text
        sta stq + 1
        lda scroll_text
.sk3:   sta [sq],z                      ; Z = sc_n: letzte Zelle
        inc stq                         ; 16-Bit-Zeiger weiter
        bne .sk1
        inc stq + 1
.sk1:   lda scx
        ora #$18                        ; Mehrfarbtext, 40 Spalten, plus Feinversatz
        sta scrollreg
        rts


; scroll_text: der Lauftext des Titels (Bildschirmcodes, mit 0 abgeschlossen).
scroll_text:
        .screen "                                        prallball - zerstoere alle steine mit ball und schlaeger!   fange die muenzen, die aus zerstoerten steinen fallen:  hellblau = breiter schlaeger,  hellrot = schmaler schlaeger,  rot = umgekehrte steuerung,  gruen = ball langsamer,  gelb = ball schneller,  orange = ball klebt,  weiss = extraball,  tuerkis = bonuspunkte,  violett = mehrball,  hellgruen = laser,  grau = durchschlag.   viel spass!                "
        .byte 0

; title_balls_init: die drei Titelbaelle (Sprites 3-5) auf ihre Anfangslage setzen und einschalten.
title_balls_init:
        ldx #2
.tbi:   lda tb_u0,x
        sta tbu,x
        lda tb_y0,x
        sta tby,x
        lda tb_du,x
        sta tbdu,x
        lda tb_dy,x
        sta tbdy,x
        dex
        bpl .tbi
        lda #8 | 16 | 32
        sta VIC_SPR_ENABLE              ; nur die drei Baelle (Schlaeger/Kapsel aus)
        rts

; title_balls_tick: die Titelbaelle einen Schritt weiter huepfen lassen (Wandabpraller, in beiden Aufloesungen).
title_balls_tick:
        ldx #0
        lda hires
        beq .tbl0
        jmp title_balls_tick_h
.tbl0:  stx t3                          ; gesammeltes 9. X-Bit
.tb:    lda tbu,x                       ; X bewegen, an den Raendern (u 6 und 148) umkehren
        clc
        adc tbdu,x
        sta tbu,x
        cmp #6
        bcc .tbf1
        cmp #148
        bcc .tbx
.tbf1:  sec                             ; Schritt umkehren: 0 - Schritt
        lda #0
        sbc tbdu,x
        sta tbdu,x
.tbx:   lda tby,x                       ; Y bewegen, Grenzen 60 und 205
        clc
        adc tbdy,x
        sta tby,x
        cmp #60
        bcc .tbf2
        cmp #205
        bcc .tby
.tbf2:  sec
        lda #0
        sbc tbdy,x
        sta tbdy,x
.tby:   lda tbu,x                       ; Sprite-X = 2 * u + 24 (9 Bit)
        asl
        sta t0
        lda #0
        rol
        sta t1
        clc
        lda t0
        adc #24
        sta t0
        bcc +
        inc t1
+       ldy ball_reg,x
        lda t0
        sta VIC_SPR0_X,y
        lda t1
        beq +
        lda ball_bit,x
        ora t3
        sta t3
+       lda tby,x
        clc
        adc #50
        sta VIC_SPR0_Y,y
        inx
        cpx #3
        bne .tb
        lda t3
        sta VIC_SPR_X_MSB
        rts

tb_u0: .byte 30, 80, 120
tb_y0: .byte 90, 140, 70
tb_du: .byte 1, $ff, 1
tb_dy: .byte 1, 2, $ff

t_sub:  .screen "pralle action fuer den mega65"
        .byte 0
t_best: .screen "bestenliste"
        .byte 0
t_ctrl: .screen "joystick port 2: links / rechts"
        .byte 0
t_keys: .screen "tasten z x q a, leertaste"
        .byte 0
t_start: .screen "feuer = start"
        .byte 0

; draw_logo: "PRALLBALL" in einer 3 x 5 Pixel grossen Blockschrift; jeder Pixel ist ein Bildschirmzeichen (CH_BLOCK).
;   Die Zeilen 3..7 des Bildschirms bekommen je eine Regenbogenfarbe (Farb-RAM = 8 + Farbe, Mehrfarbzeichen).  Buchstabe i beginnt in
;   Spalte 2 + 4 * i (3 Spalten Buchstabe + 1 Spalte Abstand) - zusammen 36 Spalten, mittig im 40-Spalten-Bild.
draw_logo:
        lda #0
        sta lt
.dl:    ldx lt
        lda logo_seq,x                  ; Buchstabennummer
        sta t0
        asl
        asl
        clc
        adc t0                          ; * 5 = Index der ersten Zeile in logo_font
        sta t0
        lda lt
        asl
        asl
        clc
        adc #2
        sta lcol                        ; Spalte des Buchstabenanfangs
        lda #0
        sta lr
.dr:    lda lr
        clc
        adc t0
        tax
        lda logo_font,x                 ; 3 Bit: Bit 2 = linker Pixel
        sta lbits
        ldx lr
        lda logo_color,x
        sta txtcol                      ; Farbe dieser Bildschirmzeile
        ldy #0                          ; Y = Pixel 0..2
.dp:    asl lbits                       ; (das 3-Bit-Muster liegt in Bit 2..0; durch Schieben ueber Bit 3 laufen)
        lda lbits
        and #8
        beq .dn
        tya
        clc
        adc lcol
        tax                             ; Spalte
        stx t2
        sty t3
        lda lr
        clc
        adc #3
        tay                             ; Zeile = 3 + lr
        lda #CH_BLOCK
        jsr put_big
        ldy t3
.dn:    iny
        cpy #3
        bne .dp
        inc lr
        lda lr
        cmp #5
        bne .dr
        inc lt
        lda lt
        cmp #9
        bne .dl
        rts

logo_seq:   .byte 0, 1, 2, 3, 3, 4, 2, 3, 3          ; P R A L L B A L L (Buchstaben 0..4 = P, R, A, L, B)
logo_color: .byte 10, 15, 13, 11, 14                  ; Zeilenfarben: rot, gelb, gruen, tuerkis, blau (je 8 + Farbe)
; Blockschrift 3 x 5: je Buchstabe 5 Byte (eine Zeile je Byte, Bit 2 = links)
logo_font:  .byte %111, %101, %111, %100, %100        ; P
            .byte %111, %101, %110, %101, %101        ; R
            .byte %111, %101, %111, %101, %101        ; A
            .byte %100, %100, %100, %100, %111        ; L
            .byte %110, %101, %110, %101, %110        ; B

; draw_top5: die ersten fuenf Bestenlisten-Eintraege (Platz, Name, Punkte) in den Zeilen 13..17.
;   Eintrag k liegt bei hs_buf + HS_FIRST + 12 * k: 8 Zeichen Name (ASCII), dann Punkte lo/mid/hi (BCD), dann Level.
draw_top5:
        lda #0
        sta lt                          ; lt = Rang 0..4
.t5:    lda #1
        sta txtcol
        lda lt
        clc
        adc #13
        sta lr                          ; lr = Bildschirmzeile
        lda lt
        clc
        adc #$31                        ; Platzziffer 1..5 (Bildschirmcode der Ziffer = $30 + n)
        ldx #9
        ldy lr
        jsr put_cell
        lda lt                          ; Offset des Eintrags in hs_buf: 5 + 12 * lt
        asl
        asl
        sta t0                          ; 4 * lt
        asl
        clc
        adc t0                          ; 8 * lt + 4 * lt = 12 * lt
        clc
        adc #HS_FIRST
        sta t1
        ldx #0
.t5n:   stx t2                          ; Name: 8 Zeichen
        lda t1
        clc
        adc t2
        tay
        lda hs_buf,y                    ; ASCII
        cmp #64
        bcc +
        sec
        sbc #64                         ; Buchstabe -> Bildschirmcode
+       pha
        txa
        clc
        adc #12                         ; Spalte 12..19
        tax
        ldy lr
        pla
        jsr put_cell
        ldx t2
        inx
        cpx #8
        bne .t5n
        lda t1                          ; Punkte: BCD-Bytes hi, mid, lo an den Spalten 22, 24, 26
        clc
        adc #10
        tay
        lda hs_buf,y
        ldx #22
        ldy lr
        jsr put_bcd
        lda t1
        clc
        adc #9
        tay
        lda hs_buf,y
        ldx #24
        ldy lr
        jsr put_bcd
        lda t1
        clc
        adc #8
        tay
        lda hs_buf,y
        ldx #26
        ldy lr
        jsr put_bcd
        inc lt
        lda lt
        cmp #5
        bne .t5
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Spielstart und Level aufbauen
; ---------------------------------------------------------------------------------------------------------------------
; new_game: alles auf Anfang.
new_game:
        lda #0
        sta sc0
        sta sc1
        sta sc2
.ifdef TESTLOSE
        lda #$05
        sta sc1                         ; Testpunktestand 500, damit das Spielende einen Rekord ergibt
        lda #0
.endif
        sta level
        sta speed
        sta hits
.ifdef TESTSPEED
        lda #TESTSPEED
        sta speed                       ; Testfassung: mit dieser Geschwindigkeitsstufe beginnen
.endif
        sta padh
        lda #144                        ; Schlaeger mittig: 16 + (288 - 32) / 2
        sta padl
        lda #$18
        sta scrollreg                   ; kein Feinversatz im Spiel
.ifdef TESTFPS
        lda #0
        sta $c200
        sta $c201
.endif
.ifdef STARTLEVEL
        lda #STARTLEVEL                 ; Testfassung: mit diesem Level (0-basiert) beginnen
        sta level
.endif
        lda #0
        sta glg
        sta bflash
        lda #LIVES0
        sta lives
        lda #8
        sta atgt
        jmp reset_extras

; reset_extras: alle Extras aus (neues Spiel, neues Level, Ball verloren): Kapsel weg, Timer 0, Schlaeger 32 px.
reset_extras:
        lda #0
        sta capact
        sta tpad
        sta trev
        sta tspd
        sta spdmod
        sta sticky
        sta tlas
        sta tpierce
        sta shact
        sta shact + 1
        lda #25
        sta tick
.ifdef TESTLASER
        lda #240                        ; Testfassung: Laser immer an
        sta tlas
.endif
.ifdef TESTWIDE
        lda #250                        ; Testfassung: Durchschlag (goldene Kugel) und breiter Schlaeger
        sta tpierce
        lda #48
        jmp set_padw
.endif
        lda #32                         ; weiter mit set_padw

; set_padw: Schlaegerbreite A (16, 32 oder 48) setzen und die Sprites passend einrichten.
;   MEGA65: 32 px = zwei 16-Farb-Sprites zu 16 px; 48 px = links 16 px und rechts ein X-verdoppeltes Sprite (32 px); 16 px = ein Sprite, das zweite aus.
;   zstep = Breite / 8 ist die Zonenbreite fuer den Abprallwinkel (8 Zonen ueber die ganze Breite).
set_padw:
        sta padw
        lsr
        lsr
        lsr
        sta zstep
        lda hires
        beq .spw0
        rts                             ; 640 x 400: Zeiger und X-Verdopplung setzt update_sprites_h je Bild
.spw0:
        lda padw
        cmp #48
        beq .sw48
        cmp #16
        beq .sw16
        lda #$c6                        ; 32 px: zwei 16-Farb-Sprites zu je 16 px ($B180 links, $B240 rechts)
        sta SPTR + 1
        lda #$c9
        sta SPTR + 2
        lda #16
        sta pad2o
        lda #0
        sta VIC_SPR_EXPAND_X            ; keine X-Verdopplung
        rts
.sw48:  lda #$c6                        ; 48 px: links 16 px, rechts ein X-verdoppeltes Sprite (32 px, $B300)
        sta SPTR + 1
        lda #$cc
        sta SPTR + 2
        lda #16
        sta pad2o
        lda #4
        sta VIC_SPR_EXPAND_X            ; Sprite 2 doppelt breit
        rts
.sw16:  lda #$c3                        ; 16 px: ein Sprite mit zwei runden Enden ($B0C0)
        sta SPTR + 1
        lda #0
        sta pad2o                       ; 0 = zweites Sprite aus (siehe update_sprites)
        sta VIC_SPR_EXPAND_X
        rts

; load_level: Steinkarte des Levels `level` ins RAM kopieren, zerstoerbare Steine zaehlen, Bild aufbauen.
load_level:
        ldx level
        lda level_tab_lo,x
        sta tp
        lda level_tab_hi,x
        sta tp + 1
        jsr clear_screen
        jsr wander_clear
        lda #0
        sta bricks
        sta cores
        ldy #0
.ll:    lda (tp),y
        sta MAP,y
        beq .lz                         ; 0 = leer: nicht zaehlen
        cmp #11
        beq .lz                         ; 11 = unzerstoerbar: nicht zaehlen
        inc bricks
        cmp #16
        bcc .lz
        inc cores                       ; Typ 16..23 = Boss-Kern (Treffer-Punkte = Typ - 15)
.lz:    iny
        cpy #216                        ; 12 * 18 Steine
        bne .ll
.ifdef WTEST2
        lda #1
        sta MAP + 9 * 18 + 3            ; Testfassung: vier Steine in Zeile 9
        sta MAP + 9 * 18 + 7
        sta MAP + 9 * 18 + 11
        sta MAP + 9 * 18 + 15
.endif
        lda level
        and #3
        jsr set_theme                   ; jedes Level hat einen eigenen Hintergrundverlauf (4 Verlaeufe im Wechsel)
        ldx level
        lda level_kind,x
        sta lkind                       ; Levelart (Bonus/Boss) aus der Tabelle
        lda #45
        sta btime                       ; Bonuslevel: 45 Sekunden
        lda #0
        sta bsub
        lda #8
        sta batk                        ; Boss: erster Angriff nach 4 s
        jsr draw_walls
        jsr draw_map
        jsr draw_hud
        jsr reset_extras
        jsr place_ball
        jsr draw_status                 ; Bonus-/Bosszeile unten
        lda demo
        beq .nodt
        lda #0                          ; Demo: Ticker in der obersten Zeile (Folge 1: Spielname, Untertitel, Demo-Hinweis)
        ldx #1
        jsr ticker_init
.nodt:
        lda #3
        sta VIC_SPR_ENABLE              ; Ball und Schlaeger an (update_sprites setzt die Bits fuer Sprite 2 und die Kapsel)
        rts

; draw_walls: Oberkante (Zeile 1) und beide Seiten (Spalten 1 und 38), grau.
draw_walls:
        lda #14                         ; 8 + 6: Mehrfarbzeichen, Farbe 6 (blau) als Flaechenfarbe
        sta txtcol
        lda #1
        sta t1
.wt:    ldx t1
        ldy #1
        lda #CH_WALL
        jsr put_big
        inc t1
        lda t1
        cmp #39
        bne .wt                         ; Spalten 1..38
        lda #2
        sta t1
.ws:    ldx #1
        ldy t1
        lda #CH_WALL
        jsr put_big
        ldx #38
        ldy t1
        lda #CH_WALL
        jsr put_big
        inc t1
        lda t1
        cmp #25
        bne .ws                         ; Zeilen 2..24
        lda #0                          ; schwarz: Spalte 0 und 39 mit vollen Bloecken (Zeichen 160 = inverses Leerzeichen) fuellen, damit die
        sta txtcol                      ; Hintergrundbaender nur im Spielfeld zwischen den Waenden sichtbar sind
        lda #1
        sta t1
.wb:    ldx #0
        ldy t1
        lda #160
        jsr put_cell
        ldx #39
        ldy t1
        lda #160
        jsr put_cell
        inc t1
        lda t1
        cmp #25
        bne .wb                         ; Zeilen 1..24
        rts

; draw_map: die ganze Steinkarte zeichnen (12 Zeilen x 18 Spalten).
draw_map:
        jsr crumble_reset               ; laufende Bruch-Animationen des vorigen Levels verwerfen
        lda #0
        sta bidx
        sta brow
.dr:    lda #0
        sta bcol
.dc:    ldx bidx
        lda MAP,x
        jsr draw_brick
        inc bidx
        inc bcol
        lda bcol
        cmp #18
        bne .dc
        inc brow
        lda brow
        cmp #12
        bne .dr
        rts

; draw_brick: Stein vom Typ A an (bcol, brow) zeichnen (Typ 0 = Leerraum).  Ein Stein sind zwei Zeichen nebeneinander.
;   Bildschirmspalte = 2 + 2 * bcol; Bildschirmzeile = 4 + brow.   Veraendert A, X, Y, t0.
draw_brick:
        sta t0
        lda bcol
        asl
        clc
        adc #2
        tax
        lda brow
        clc
        adc #4
        tay
        jsr set_rowptr
        lda t0
        beq .db0
        ldx #0
        stx bph
        jmp put_brick
.db0:   jmp clear_cells

; put_brick: Stein vom Typ A (1..24; 24 = weisse Glanzfassung) in der Fassung bph (0 Stein, 1 und 2 Bruchphasen) an die Zelle (sq) schreiben.
;   320 x 200: 2 Zeichen (put_fcm).  640 x 400: 4 x 2 Zeichen der Farbgruppe des Typs (brick_grp), Zeichennummer = HB_BASE + 80 * bph + 8 * Gruppe.
put_brick:
        ldx hires
        bne .pbh
        sec
        sbc #1
        asl                             ; Zeichenpaar des Typs: 2 * (Typ - 1)
        ldx bph
        clc
        adc lphase,x                    ; Bruchphasen liegen 50 bzw. 98 Zeichen weiter
        jmp put_fcm
.pbh:   tax
        lda brick_grp,x
        asl
        asl
        asl                             ; 8 * Gruppe
        ldx bph
        clc
        adc hphase_lo,x
        sta qn0
        lda #>HB_BASE
        adc #0
        sta qn1
        ldx #0
.pb1:   lda b_off,x
        taz
        txa
        clc
        adc qn0
        sta [sq],z
        lda qn1
        adc #0
        inz
        sta [sq],z
        inx
        cpx #8
        bne .pb1
        rts
lphase:    .byte 0, 50, 98
hphase_lo: .byte <HB_BASE, <(HB_BASE + 80), <(HB_BASE + 160)
bph:       .byte 0

; clear_cells: die Zellen eines Steins ab (sq) loeschen (Zeichencode = Leerzeichen, obere Bits 0): 2 bzw. 4 x 2 Zellen.
clear_cells:
        ldx hires
        bne .cch
        ldz #0
        lda #CH_SPACE
        sta [sq],z
        ldz #2
        sta [sq],z
        lda #0
        ldz #1
        sta [sq],z
        ldz #3
        sta [sq],z
        rts
.cch:   ldx #7
.cc1:   lda b_off,x
        taz
        lda #CH_SPACE
        sta [sq],z
        inz
        lda #0
        sta [sq],z
        dex
        bpl .cc1
        rts

; put_fcm: Vollfarb-Stein an die Zelle (sp) schreiben: A = 2 * Nummer des Steins (linkes Zeichen); das rechte ist das naechste.
;   Zeichennummer = FCM_BASE + A (16 Bit: Low-Byte in die gerade, High-Byte in die ungerade Stelle der Zelle).
put_fcm:
        clc
        adc #<FCM_BASE
        pha
        lda #>FCM_BASE
        adc #0                          ; Uebertrag (Zeichennummern ab 256 hinter FCM_BASE)
        ldz #1
        sta [sq],z
        ldz #3
        sta [sq],z
        pla
        ldz #0
        sta [sq],z
        clc
        adc #1                          ; rechtes Zeichen: Nummer + 1 (A ist gerade, kein Uebertrag)
        ldz #2
        sta [sq],z
        rts

; put_big: Vollfarbzeichen A (Nummer FCM_BASE + A, z. B. CH_WALL) an Spalte X, Zeile Y; Farbbyte = Farbe 0..7 aus txtcol (Pixelwert 255).
put_big:
        pha
        jsr set_rowptr
        pla
        ldx hires
        bne .pbg
        clc
        adc #<FCM_BASE
        ldz #0
        sta [sq],z
        lda #>FCM_BASE
        inz
        sta [sq],z
        lda txtcol
        and #7
        sta [cq],z
        rts
.pbg:   sec                             ; hohe Aufloesung: Viereck aus 4 Zeichen (Wand 0, Logo-Block 1)
        sbc #CH_WALL
        asl
        asl
        clc
        adc #<HS_BASE
        sta qn0
        lda #>HS_BASE
        adc #0
        sta qn1
        lda txtcol
        pha
        and #7
        sta txtcol
        jsr put_quad
        pla
        sta txtcol
        rts

; fcm_init: Vollfarb-Steine einrichten (einmal beim Start).  brick_pat enthaelt EIN Steinmuster (2 Zeichen * 64 Byte, Werte 32 + Stufe);
;   daraus entstehen die 24 Typen: Typ n bekommt 8 * n zu jedem Bildpunkt (ausser 0 = durchsichtig) addiert, benutzt also seinen eigenen
;   Palettenbereich.  Ziel: Chip-RAM ab $12000 (Zeichennummer FCM_BASE, 32-Bit-Zeiger $E0-$E3), je Typ 128 Byte.  Danach die Palette
;   (Eintraege 32..223) schreiben ($D100/$D200/$D300 zeigen die Palettenbank 0).  Veraendert A, X, Y, Z, t1, t2.
fcm_init:
        lda #$00
        sta $e0
        lda #$20
        sta $e1
        lda #$01
        sta $e2                         ; Ziel $0001_2000 = FCM_BASE * 64
        lda #$00
        sta $e3
        lda #<brick_pat                 ; 24 Steine
        sta cp
        lda #>brick_pat
        sta cp + 1
        jsr fcm_types
        ldx #0                          ; Wand und Logo-Block (ohne Addition): 128 Byte hinter den Steinen
.fi7:   lda brick_pat + 128,x
        tay                             ; (Y = Wert, X = Index)
        txa
        taz
        tya
        sta [$e0],z
        inx
        cpx #128
        bne .fi7
        lda $e0                         ; Zeiger hinter die 128 Byte (Wand, Logo): +128
        clc
        adc #$80
        sta $e0
        bcc .fi8
        inc $e1
.fi8:   lda #<crumb_pat                 ; Bruchphase 1 und 2: je 24 Typen
        sta cp
        lda #>crumb_pat
        sta cp + 1
        jsr fcm_types
        lda #<(crumb_pat + 128)
        sta cp
        lda #>(crumb_pat + 128)
        sta cp + 1
        jsr fcm_types
        ldx #202                        ; 24 Typen * 8 Stufen + 8 Stufen Wand + 2 Eintraege Logo-Rand + Schwarz
.fi5:   lda bpal_r,x
        sta $d120,x
        lda bpal_g,x
        sta $d220,x
        lda bpal_b,x
        sta $d320,x
        dex
        cpx #$ff
        bne .fi5
        lda hires
        beq .fi6
        jsr hires_init                  ; (320 x 200: nicht noetig, der Speicher dient den wandernden Steinen)
.fi6:   jsr particle_init
        ldz #0
        rts

; hires_init: Daten der hohen Aufloesung ins Chip-RAM: Steine (3 Fassungen * 10 Farbgruppen * 512 Byte ab $14800, je Gruppe 8 * (Beispieltyp - 1)
;   zu den Werten addiert), Wand/Logo/voller Block ab $18400 und die Schrift (Bildschirmcodes 0..63 als 16 x 16 Bildpunkte aus dem 8 x 8-Zeichensatz bei
;   $8800, jedes Viertel ein Vollfarbzeichen: gesetzter Punkt = Pixelwert 255 = Farbe aus dem Farb-RAM) ab $18800.
hires_init:
        lda #$00
        sta $e0
        lda #$48
        sta $e1
        lda #$01
        sta $e2
        lda #$00
        sta $e3
        sta hi_p
        lda #<hb_pat
        sta hsrc
        lda #>hb_pat
        sta hsrc + 1
.hp:    lda #0
        sta hi_g
.hg:    ldx hi_g
        lda hb_off,x
        sta hi_o
        lda hsrc
        sta cp
        lda hsrc + 1
        sta cp + 1
        ldx #2
.hy:    ldy #0
.hz:    lda (cp),y
        beq .hq
        clc
        adc hi_o
.hq:    sta t0
        tya
        taz
        lda t0
        sta [$e0],z
        iny
        bne .hz
        inc cp + 1
        inc $e1
        dex
        bne .hy
        inc hi_g
        lda hi_g
        cmp #10
        bne .hg
        lda hsrc + 1
        clc
        adc #2                          ; naechste Fassung: Muster + 512
        sta hsrc + 1
        inc hi_p
        lda hi_p
        cmp #3
        bne .hp
        lda #<hs_pat                    ; Wand, Logo-Block, voller Block (3 Seiten kopieren: der Rest ist unbenutzt)
        sta cp
        lda #>hs_pat
        sta cp + 1
        ldx #3
.hs1:   ldy #0
.hs2:   lda (cp),y
        sta t0
        tya
        taz
        lda t0
        sta [$e0],z
        iny
        bne .hs2
        inc cp + 1
        inc $e1
        dex
        bne .hs1
        lda #$00                        ; Schrift ab $0001_8800
        sta $e0
        lda #$88
        sta $e1
        lda #0
        sta hi_c
.fg1:   lda hi_c                        ; Quellzeiger cp = $8800 + 8 * Code
        sta cp
        lda #0
        sta cp + 1
        asl cp
        rol cp + 1
        asl cp
        rol cp + 1
        asl cp
        rol cp + 1
        lda cp + 1
        clc
        adc #$88
        sta cp + 1
        jsr font_epx                    ; OB = geglaettete 16 x 16 Punkte (0/1)
        lda #0
        sta hi_q
.fg2:   ldz #0                          ; Viertel q: Zeile 8 * (q >> 1), Spalte 8 * (q & 1)
        lda hi_q
        and #2
        asl
        asl
        asl
        asl
        asl
        asl                             ; 128 * (q >> 1)
        sta t0
        lda hi_q
        and #1
        asl
        asl
        asl
        clc
        adc t0
        tay                             ; Y = Index in OB
        ldx #8
.fg3:   lda #8
        sta t1
.fg4:   lda OB,y
        beq .fg5
        lda #$ff                        ; gesetzter Punkt = Pixelwert 255 = Farbe aus dem Farb-RAM
.fg5:   sta [$e0],z
        inz
        iny
        dec t1
        bne .fg4
        tya
        clc
        adc #8                          ; naechste Zeile der 16 breiten Vorlage
        tay
        dex
        bne .fg3
        lda $e0                         ; naechstes Zeichen: Ziel + 64
        clc
        adc #64
        sta $e0
        bcc .fg7
        inc $e1
.fg7:   inc hi_q
        lda hi_q
        cmp #4
        bne .fg2
        inc hi_c
        lda hi_c
        cmp #64
        bne .fg1
        ldz #0
        rts

; font_epx: 8 x 8-Zeichen (8 Byte ab (cp)) auf 16 x 16 Punkte vergroessern und glaetten (EPX / Scale2x): jeder Punkt P wird zu 2 x 2 Punkten; mit den
;   Nachbarn A (oben), B (rechts), C (links), D (unten) gilt: links oben = A, wenn C = A, C <> D und A <> B (sonst P); rechts oben = B, wenn A = B, A <> C und B <> D;
;   links unten = C, wenn D = C, D <> B und C <> A; rechts unten = D, wenn B = D, B <> A und D <> C.  So werden Diagonalen weich statt treppig.
;   SB = 10 x 10 Vorlage mit Rand (Byte 0/1), OB = 16 x 16 Ergebnis (je 0/1).  Veraendert A, X, Y, t1, t3.
.const SB = $c300
.const OB = $c400
font_epx:
        ldx #99
        lda #0
.fe0:   sta SB,x
        dex
        bpl .fe0
        ldy #0
.fe1:   lda (cp),y                      ; Zeile y des Zeichens in die Vorlage
        sta t1
        ldx sbrow,y
        lda #8
        sta t3
.fe2:   asl t1
        lda #0
        rol
        sta SB,x
        inx
        dec t3
        bne .fe2
        iny
        cpy #8
        bne .fe1
        lda #0
        sta fe_r
.fe3:   ldy fe_r
        lda sbrow,y
        sta fe_s
        tya
        asl
        asl
        asl
        asl
        asl
        sta fe_o                        ; 32 * Zeile
        lda #0
        sta fe_x
.fe4:   ldx fe_s
        lda SB,x
        sta fe_p
        lda SB - 10,x
        sta fe_a
        lda SB + 10,x
        sta fe_d
        lda SB - 1,x
        sta fe_c
        lda SB + 1,x
        sta fe_b
        lda fe_p                        ; links oben
        sta fe_1
        lda fe_c
        cmp fe_a
        bne .e1
        cmp fe_d
        beq .e1
        lda fe_a
        cmp fe_b
        beq .e1
        sta fe_1
.e1:    lda fe_p                        ; rechts oben
        sta fe_2
        lda fe_a
        cmp fe_b
        bne .e2
        cmp fe_c
        beq .e2
        lda fe_b
        cmp fe_d
        beq .e2
        sta fe_2
.e2:    lda fe_p                        ; links unten
        sta fe_3
        lda fe_d
        cmp fe_c
        bne .e3
        cmp fe_b
        beq .e3
        lda fe_c
        cmp fe_a
        beq .e3
        sta fe_3
.e3:    lda fe_p                        ; rechts unten
        sta fe_4
        lda fe_b
        cmp fe_d
        bne .e4
        cmp fe_a
        beq .e4
        lda fe_d
        cmp fe_c
        beq .e4
        sta fe_4
.e4:    ldy fe_o
        lda fe_1
        sta OB,y
        lda fe_2
        sta OB + 1,y
        lda fe_3
        sta OB + 16,y
        lda fe_4
        sta OB + 17,y
        inc fe_s
        inc fe_o
        inc fe_o
        inc fe_x
        lda fe_x
        cmp #8
        bne .fe4
        inc fe_r
        lda fe_r
        cmp #8
        bne .fe3
        rts
sbrow:  .byte 11, 21, 31, 41, 51, 61, 71, 81
fe_r:   .byte 0
fe_x:   .byte 0
fe_s:   .byte 0
fe_o:   .byte 0
fe_p:   .byte 0
fe_a:   .byte 0
fe_b:   .byte 0
fe_c:   .byte 0
fe_d:   .byte 0
fe_1:   .byte 0
fe_2:   .byte 0
fe_3:   .byte 0
fe_4:   .byte 0
hsrc:   .byte 0, 0
hi_g:   .byte 0
hi_p:   .byte 0
hi_o:   .byte 0
hi_c:   .byte 0
hi_q:   .byte 0

; fcm_types: 24 Typen eines 128-Byte-Musters (cp) ab dem 32-Bit-Ziel $E0..: Typ n bekommt 8 * n zu jedem Wert ausser 0 addiert.
fcm_types:
        lda #0
        sta t1                          ; t1 = 8 * Typ (0, 8, ... 184)
        lda #24
        sta t2
.ft1:   ldy #0
.ft2:   lda (cp),y
        beq .ft3                        ; 0 bleibt 0 (durchsichtig)
        clc
        adc t1
.ft3:   tax
        tya
        taz
        txa
        sta [$e0],z
        iny
        cpy #128
        bne .ft2
        lda $e0                         ; naechster Typ: Ziel + 128
        clc
        adc #$80
        sta $e0
        bcc .ft4
        inc $e1
.ft4:   lda t1
        clc
        adc #8
        sta t1
        dec t2
        bne .ft1
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Bruch-Animation: ein zerstoerter Stein blitzt weiss auf (2 Bilder), zerbroeselt in zwei Phasen (je 3 Bilder) und verschwindet.  Bis zu 8
;   Steine gleichzeitig.  Eintrag k: acol/arow = Steinspalte/-zeile, atyp = Typ, ast = naechster Schritt (0 = frei, 1 Blitz, 2 Phase 1,
;   3 Phase 2, 4 loeschen), atm = Bilder bis zum Schritt.  crumble_start (A = Typ, bcol/brow = Lage) legt einen Eintrag an; crumble_tick
;   einmal je Bild; crumble_reset leert die Liste (beim Zeichnen eines neuen Levels).
; ---------------------------------------------------------------------------------------------------------------------
; Dauer der Schritte in Bildern (Testfassung -D CRUMBSLOW=1: langsam, damit man die Phasen im Screenshot sieht)
.ifdef CRUMBSLOW
.const CR_FLASH = 1
.const CR_PHASE = 250
.else
.const CR_FLASH = 2
.const CR_PHASE = 4
.endif
crumble_reset:
        ldx #7
        lda #0
.cr1:   sta ast,x
        dex
        bpl .cr1
        jmp particle_reset

crumble_start:
        cmp #16
        bcs .csx                        ; Boss-Kerne bleiben ohne Bruch
        sta cut
        jsr particle_spawn              ; Splitter
        ldx #7
.cs1:   lda ast,x
        beq .cs2
        dex
        bpl .cs1
.csx:   rts                             ; Liste voll: dieser Stein verschwindet ohne Bruch
.cs2:   lda bcol
        sta acol,x
        lda brow
        sta arow,x
        lda cut
        sta atyp,x
        lda #1
        sta ast,x
        sta atm,x
        rts

crumble_tick:
.ifdef CRUMBTEST
        inc ctc                         ; Testfassung: nach 100 Bildern 8 Steine der Zeile 3 auf einmal "zerstoeren"
        lda ctc
        cmp #100
        bne .nct
        lda #3
        sta brow
        lda #2
        sta bcol
.ctl:   ldy brow
        lda rowmul,y
        clc
        adc bcol
        tax
        lda MAP,x
        beq .ctn2
        pha
        lda #0
        sta MAP,x
        pla
        jsr crumble_start
        lda #0
        jsr draw_brick
.ctn2:  inc bcol
        inc bcol
        lda bcol
        cmp #18
        bcc .ctl
.nct:
.endif
        ldx #7
.cu1:   lda ast,x
        beq .cun
        dec atm,x
        bne .cun
        stx cux                         ; Eintrag merken (X und Y gehen an set_rowptr)
        lda acol,x
        asl
        clc
        adc #2
        sta cut                         ; Bildschirmspalte
        lda arow,x
        clc
        adc #4
        tay                             ; Bildschirmzeile
        ldx cut
        jsr set_rowptr
        ldx cux
        lda ast,x
        cmp #4
        bcs .cuc                        ; 4: weg
        sta cut                         ; Schritt 1..3
        lda #CR_PHASE
        sta cuw
        lda atyp,x
        ldy cut
        cpy #1
        bne .cu2
        lda #24                         ; 1: Blitz = weisse Fassung (Typ 24), Phase 0
        ldy #0
        sty bph
        ldy #CR_FLASH
        sty cuw
        bne .cus
.cu2:   ldy cut
        dey
        sty bph                         ; 2: Phase 1, 3: Phase 2
.cus:   jsr put_brick
        ldx cux
        lda cuw
        sta atm,x
        inc ast,x
        bne .cun
.cuc:   jsr clear_cells
        ldx cux
        lda #0
        sta ast,x
.cun:   dex
        bpl .cu1
        rts

cuw:    .byte 0
acol:   .byte 0, 0, 0, 0, 0, 0, 0, 0
arow:   .byte 0, 0, 0, 0, 0, 0, 0, 0
atyp:   .byte 0, 0, 0, 0, 0, 0, 0, 0
ast:    .byte 0, 0, 0, 0, 0, 0, 0, 0
atm:    .byte 0, 0, 0, 0, 0, 0, 0, 0
cux:    .byte 0
ctc:    .byte 0
cut:    .byte 0


; ---------------------------------------------------------------------------------------------------------------------
; Truemmerteilchen (nur MEGA65): Aus jedem zerstoerten Stein spritzen 4 Splitter in der Farbe des Steins, fliegen mit Schwerkraft und verschwinden
;   am Ende ihrer Lebenszeit, an Wand, Stein oder Text.  Jeder Splitter ist ein Vollfarbzeichen (8 x 8, Pixelwert 255 = Farbe aus dem Farb-RAM) mit
;   einem Fleck in einem der vier Viertel; 8 Zeichen (klein/gross * 4 Viertel) liegen im Chip-RAM ab PT_ADDR (Zeichennummer PT_BASE) und werden mit
;   dem DMA-Controller (Fuellen) geloescht und dann mit Punkten gefuellt.  Position in 1/16 px (320 x 200-Bildpunkte; in der hohen Aufloesung gibt es
;   doppelt so viele Zeichen je Richtung), Geschwindigkeit in 1/16 px je Bild (8 Bit mit Vorzeichen), Schwerkraft + 2 je Bild.
;   Die Zeichen auf dem Bildschirm werden nur gesetzt, wenn dort ein Leerzeichen steht, und nur geloescht, wenn dort noch der eigene Splitter steht.
; ---------------------------------------------------------------------------------------------------------------------
.const PT_ADDR = $1ce00                 ; 8 Zeichen zu 64 Byte, hinter dem Pling-Sample (Seiten $C8-$CD) und vor dem Bildschirm ($1D000)
.const PT_BASE = $0738                  ; Zeichennummer = PT_ADDR / 64
.const PT_N    = 8                      ; Splitter gleichzeitig
particle_init:
        lda #0                          ; DMA-Auftrag (Fuellen): 512 Byte ab $1CE00 mit 0 loeschen
        sta $d702                       ; Bank der Auftragsliste: 0
        sta $d704                       ; Erweiterung (MB) 0
        lda #>pt_dma
        sta $d701
        lda #<pt_dma
        sta $d700                       ; Schreiben nach $D700 startet den Auftrag (Liste im Format F018A)
        lda #$00
        sta $e0
        lda #$ce
        sta $e1
        lda #$01
        sta $e2
        lda #$00
        sta $e3                         ; 32-Bit-Zeiger $E0-$E3 = $0001CE00
        ldx #0                          ; Zeichen 0..7: Viertel (x, y) = (Bit 0, Bit 1), Groesse = Bit 2 (0: 2 x 2, 1: 3 x 3 Punkte)
.pti:   stx pt_i
        txa
        and #1
        asl
        asl
        clc
        adc #1
        sta pt_x                        ; erster Punkt: x = 4 * qx + 1
        txa
        and #2
        asl
        clc
        adc #1
        sta pt_y                        ; y = 4 * qy + 1
        txa
        and #4
        lsr
        lsr
        clc
        adc #2
        sta pt_s                        ; Kantenlaenge 2 oder 3
        lda pt_i
        asl
        asl
        asl
        asl
        asl
        asl
        sta pt_o                        ; 64 * Zeichen, Low-Byte
        lda pt_i
        lsr
        lsr
        sta pt_oh                       ; High-Byte (Zeichen 4..7)
        ldy pt_s
.ptr:   dey                             ; naechste Zeile (Zeile Y des Flecks)
        bmi .ptn
        tya
        clc
        adc pt_y
        asl
        asl
        asl                             ; (y + Zeile) * 8
        clc
        adc pt_x
        clc
        adc pt_o
        sta pt_a
        lda pt_oh
        adc #0
        sta pt_ah
        ldx pt_s
.ptc:   dex                             ; naechste Spalte
        bmi .ptr
        txa
        clc
        adc pt_a
        sta $e0
        lda pt_ah
        adc #0
        clc
        adc #$ce
        sta $e1
        ldz #0
        lda #255
        sta [$e0],z
        bra .ptc
.ptn:   ldx pt_i
        inx
        cpx #8
        bne .pti
        ldz #0
        rts
pt_dma: .byte $03                       ; Befehl: Fuellen
        .word 512                       ; Anzahl
        .word 0                         ; Quelle: Fuellwert 0
        .byte 0                         ; Quellbank
        .word PT_ADDR & $ffff           ; Ziel
        .byte PT_ADDR >> 16             ; Zielbank
        .byte 0, 0                      ; Modulo
pt_i:   .byte 0
pt_x:   .byte 0
pt_y:   .byte 0
pt_s:   .byte 0
pt_o:   .byte 0
pt_oh:  .byte 0
pt_a:   .byte 0
pt_ah:  .byte 0

; particle_reset: alle Splitter vergessen (die Zeichen verschwinden mit dem naechsten Bildaufbau).
particle_reset:
        ldx #PT_N - 1
        lda #0
.pr1:   sta qpst,x
        dex
        bpl .pr1
        rts

; particle_spawn: 4 Splitter fuer den Stein vom Typ cut (bcol/brow = Lage).
particle_spawn:
        ldx cut
        lda brick_grp,x
        tax
        lda deb_col,x
        sta qpcn                         ; Farbe
        lda #4
        sta qpk
.ps1:   ldx #PT_N - 1                   ; freien Platz suchen
.ps2:   lda qpst,x
        beq .ps3
        dex
        bpl .ps2
        rts                             ; alle belegt
.ps3:   lda qpcn
        sta qpcol,x
        jsr rand_byte
        and #15
        clc
        adc #44
        sta qpst,x                       ; Lebensdauer 44..59 Bilder
        lda qpk
        and #1
        asl
        asl
        sta qpvar,x                      ; abwechselnd kleiner und groesserer Splitter
        lda bcol                        ; x = 16 * (16 * Spalte + 24) = 256 * Spalte + $180
        clc
        adc #1
        sta qpxh,x
        lda #$80
        sta qpxl,x
        lda brow                        ; y = 16 * (8 * Zeile + 36) = 128 * Zeile + $240
        sta qpyl,x
        lda #0
        sta qpyh,x
        ldy #7
.ps4:   asl qpyl,x
        rol qpyh,x
        dey
        bne .ps4
        lda qpyl,x
        clc
        adc #$40
        sta qpyl,x
        lda qpyh,x
        adc #2
        sta qpyh,x
        jsr rand_byte
        and #31
        sec
        sbc #16
        sta qpvx,x                       ; -16..15
        jsr rand_byte
        and #7
        eor #$ff
        sec
        sbc #3
        sta qpvy,x                       ; -4..-11 (kurz nach oben)
        lda #$ff
        sta qpofh,x                      ; noch nichts gezeichnet
        dec qpk
        bne .ps1
        rts

; particle_tick: einmal je Bild alle Splitter bewegen und zeichnen.
particle_tick:
        ldx #PT_N - 1
.pt1:   lda qpst,x
        bne .pt2
.ptd:   dex
        bpl .pt1
        ldz #0
        rts
.pt2:   stx qpcx
        jsr pt_erase                    ; altes Zeichen weg
        ldx qpcx
        dec qpst,x
        beq .ptk                        ; Lebensdauer um
        lda qpvx,x                       ; x += vx (Vorzeichen erweitert)
        pha
        clc
        adc qpxl,x
        sta qpxl,x
        pla
        and #$80
        beq +
        lda #$ff
+       adc qpxh,x
        sta qpxh,x
        lda qpvy,x
        pha
        clc
        adc qpyl,x
        sta qpyl,x
        pla
        and #$80
        beq +
        lda #$ff
+       adc qpyh,x
        sta qpyh,x
        inc qpvy,x                       ; Schwerkraft (2 je Bild)
        inc qpvy,x
        lda qpxh,x
        cmp #20                         ; ausserhalb (x < 0 zaehlt als sehr gross)
        bcs .ptk
        lda qpyh,x
        cmp #12
        bcs .ptk
        jsr pt_draw
        ldx qpcx
.ifdef PTFREEZE
        lda qpst,x                      ; Testfassung: Bild einfrieren, damit man die Splitter im Screenshot sieht
        cmp #PTFREEZE
        bne +
.ptfz:  jmp .ptfz
+
.endif
        jmp .ptd
.ptk:   lda #0
        sta qpst,x
        jmp .ptd

; pt_erase: das zuletzt gezeichnete Zeichen von Splitter pcx wieder zum Leerzeichen machen, wenn dort noch der Splitter steht.
pt_erase:
        ldx qpcx
        lda qpofh,x
        cmp #$ff
        beq .pe9
        sta pt_ofh
        lda qpofl,x
        sta pt_ofl
        lda #$ff
        sta qpofh,x
        jsr pt_ptr
        ldz #0
        lda [sq],z
        and #$f8
        cmp #<PT_BASE
        bne .pe9
        ldz #1
        lda [sq],z
        cmp #>PT_BASE
        bne .pe9
        ldz #0
        lda #CH_SPACE
        sta [sq],z
        inz
        lda #0
        sta [sq],z
.pe9:   ldz #0
        rts

; pt_draw: Splitter pcx in seine Zelle zeichnen, aber nur auf ein Leerzeichen (Steine, Wand, Text und andere Splitter bleiben unberuehrt: dort ist der
;   Splitter in diesem Bild unsichtbar).
pt_draw:
        jsr pt_cell
        ldz #0
        lda [sq],z
        cmp #CH_SPACE
        bne .pd9
        ldz #1
        lda [sq],z
        bne .pd9
        ldx qpcx
        lda pt_q
        clc
        adc qpvar,x
        clc
        adc #<PT_BASE
        ldz #0
        sta [sq],z
        lda #>PT_BASE
        inz
        sta [sq],z
        lda qpcol,x
        sta [cq],z
        lda #0
        ldz #0
        sta [cq],z
        lda pt_ofl
        sta qpofl,x
        lda pt_ofh
        sta qpofh,x
.pd9:   ldz #0
        rts

; pt_cell: Zelle des Splitters qpcx bestimmen: pt_ofl/pt_ofh = Versatz im Bildschirmspeicher, pt_q = Viertel 0..3.  Setzt sq und cq.
pt_cell:
        ldx qpcx
        ldy #7                          ; 320 x 200: Zeichen = 8 px = 128 Einheiten (2^7)
        lda hires
        beq .pc1
        ldy #6                          ; 640 x 400: Zeichen = 4 px (der Spielfeldpunkt ist 2 Bildpunkte gross) = 64 Einheiten
.pc1:   sty pt_sh
        lda qpxl,x                       ; Spalte und Viertelbit (x)
        sta pt_l
        lda qpxh,x
        sta pt_h
        dey
.pc2:   lsr pt_h
        ror pt_l
        dey
        bne .pc2
        lda pt_l
        and #1
        sta pt_q                        ; Viertelbit x
        lsr pt_h
        ror pt_l
        lda pt_l
        sta pt_col
        lda qpyl,x
        sta pt_l
        lda qpyh,x
        sta pt_h
        ldy pt_sh
        dey
.pc4:   lsr pt_h
        ror pt_l
        dey
        bne .pc4
        lda pt_l
        and #1
        asl
        ora pt_q
        sta pt_q                        ; Viertel = x + 2 * y
        lsr pt_h
        ror pt_l
        lda pt_l
        sta pt_row                      ; Zeichenzeile 0..49
rc_ptr:                                 ; (auch einzeln: pt_row/pt_col gesetzt -> sq/cq)
        lda pt_row
        asl
        asl
        clc
        adc pt_row                      ; 5 * Zeile
        sta pt_ofl
        lda #0
        sta pt_ofh
        ldy #4                          ; * 16 (Zeilenbreite 80) bzw. * 32 (160)
        lda hires
        beq .pc5
        iny
.pc5:   asl pt_ofl
        rol pt_ofh
        dey
        bne .pc5
        lda pt_col
        asl
        clc
        adc pt_ofl
        sta pt_ofl
        bcc pt_ptr
        inc pt_ofh                      ; (weiter in pt_ptr)

; pt_ptr: sq und cq aus pt_ofl/pt_ofh setzen.
pt_ptr:
        lda pt_ofl
        sta sq
        sta cq
        lda pt_ofh
        sta cq + 1
        clc
        adc #>SCRQ
        sta sq + 1
        lda #1
        sta sq + 2
        lda #$f8
        sta cq + 2
        lda #0
        sta sq + 3
        lda #$0f
        sta cq + 3
        rts

qpst:    .byte 0, 0, 0, 0, 0, 0, 0, 0    ; Lebensdauer (0 = frei)
qpcol:   .byte 0, 0, 0, 0, 0, 0, 0, 0
qpvar:   .byte 0, 0, 0, 0, 0, 0, 0, 0    ; 0 klein, 4 gross
qpxl:    .byte 0, 0, 0, 0, 0, 0, 0, 0
qpxh:    .byte 0, 0, 0, 0, 0, 0, 0, 0
qpyl:    .byte 0, 0, 0, 0, 0, 0, 0, 0
qpyh:    .byte 0, 0, 0, 0, 0, 0, 0, 0
qpvx:    .byte 0, 0, 0, 0, 0, 0, 0, 0
qpvy:    .byte 0, 0, 0, 0, 0, 0, 0, 0
qpofl:   .byte 0, 0, 0, 0, 0, 0, 0, 0
qpofh:   .byte $ff, $ff, $ff, $ff, $ff, $ff, $ff, $ff
qpcn:    .byte 0
qpk:     .byte 0
qpcx:    .byte 0
pt_sh:  .byte 0
pt_l:   .byte 0
pt_h:   .byte 0
pt_q:   .byte 0
pt_col: .byte 0
pt_row: .byte 0
pt_ofl: .byte 0
pt_ofh: .byte 0
deb_col: .byte 2, 7, 5, 3, 6, 4, 15, 8, 11, 1   ; Palettenfarbe der Splitter je Steingruppe (rot, gelb, gruen, tuerkis, blau, violett, silber, gold, stahl, weiss)


; ---------------------------------------------------------------------------------------------------------------------
; Wandernde Steine (nur MEGA65): Gelegentlich loest sich ein Stein und gleitet weich um eine Position nach links, rechts, oben oder unten, wenn dort frei ist.
;   Waehrend der Bewegung steht der Stein in der Karte nirgends (MAP[Quelle] = MAP[Ziel] = 0); er gehoert dem Eintrag w*[k] (k = 0..MAXW-1): Typ, Quell- und
;   Zielzelle, Richtung (-1/0/+1), Fortschritt in 1/16 Bildpunkt (waagerecht 0..255 = 16 px, senkrecht 0..127 = 8 px, jeweils in Spielfeldpunkten) und die
;   sichtbare Lage wxl/wxh/wyp (Spielfeldpunkte).  Der Ball und die Schuesse pruefen gegen diese Lage (wcoll, wshot), nicht gegen die Karte: getroffen wird
;   der Stein genau dort, wo man ihn sieht.  Ein getroffener Stein bleibt an der naechstliegenden Zelle stehen (Quelle oder Ziel) und wird wie jeder Stein
;   behandelt.  Zeichnen: Das Steinmuster (Vollfarbzeichen, 320 x 200: 2 x 1 Zeichen, 640 x 400: 4 x 2) wird um 0..7 Bildpunkte verschoben in einen kleinen
;   Streifen aus Zeichen im Chip-RAM gesetzt (320 x 200: ab $14800, 640 x 400: ab $12000; dort liegen in der jeweils anderen Betriebsart Steindaten, die beim
;   Umschalten neu erzeugt werden) und der Streifen auf den Bildschirm geschrieben.  Der Streifen wird vorher mit dem DMA-Controller geloescht.
; ---------------------------------------------------------------------------------------------------------------------
.const MAXW   = 2                       ; gleichzeitig wandernde Steine
.ifdef WTEST
.const WRATE  = $01                     ; Testfassung: sehr haeufig
.else
.const WRATE  = $3f                     ; je Bild beginnt mit Wahrscheinlichkeit 1 / 64 eine Wanderung (wenn ein Platz frei ist)
.endif
.const W_SRC  = $8c                     ; 32-Bit-Zeiger auf das Steinmuster ($8C-$8F)

; wander_clear: alle Eintraege verwerfen (neues Level).
wander_clear:
        lda #0
        ldx #MAXW - 1
.wcl:   sta wst,x
        dex
        bpl .wcl
        rts

; wander_settle: wandernde Steine an ihre Quellzelle zurueckstellen (Karte), z. B. vor dem Umschalten der Aufloesung.
wander_settle:
        ldx #MAXW - 1
.wse:   lda wst,x
        beq .wsn
        ldy wsr,x
        lda rowmul,y
        clc
        adc wsc,x
        tay
        lda wtp,x
        sta MAP,y
        lda #0
        sta wst,x
.wsn:   dex
        bpl .wse
        rts

; wander_tick: einmal je Bild: laufende Wanderungen weiterfuehren, gelegentlich eine neue beginnen.
wander_tick:
        ldx #MAXW - 1
.wt1:   lda wst,x
        beq .wtn
        stx wcx
        jsr w_step
        ldx wcx
.wtn:   dex
        bpl .wt1
        lda lkind
        bne .wtx                        ; nur in normalen Levels (nicht Bonus, nicht Boss)
        lda cores
        bne .wtx
        jsr rand_byte
        and #WRATE
        bne .wtx
        jsr w_spawn
.wtx:   ldz #0
        rts

; w_spawn: einen Stein zum Wandern auswaehlen (hoechstens 8 Versuche).
w_spawn:
        ldx #MAXW - 1                   ; freien Eintrag suchen
.sp1:   lda wst,x
        beq .sp2
        dex
        bpl .sp1
        rts
.sp2:   stx wcx
        lda #8
        sta wtry
.try:   dec wtry
        bmi .spx
.ifdef WTEST2
        ldx #17                         ; Testfassung: irgendein Stein in Zeile 9
.tq:    lda MAP + 9 * 18,x
        bne .tq2
        dex
        bpl .tq
        rts
.tq2:   stx wtc
        lda #9
        sta wtr
.else
        jsr rand_byte
        and #31
        cmp #18
        bcs .try
        sta wtc
        jsr rand_byte
        and #15
        cmp #11
        bcs .try                        ; Zeilen 0..10
        sta wtr
.endif
        tay
        lda rowmul,y
        clc
        adc wtc
        tay
        lda MAP,y
        beq .try
        cmp #11
        bcs .try                        ; nur Typ 1..10
        sta wtt
        jsr rand_byte
        and #3
.ifdef WVERT
        ora #2                          ; Testfassung: nur senkrecht
.endif
.ifdef WHORZ
        and #1                          ; Testfassung: nur waagerecht
.endif
.ifdef WUP
        lda #2                          ; Testfassung: nur nach oben
.endif
        tax
        lda wdxt,x
        sta wtdx
        lda wdyt,x
        sta wtdy
        lda wtc
        clc
        adc wtdx
        bmi .try
        cmp #18
        bcs .try
        sta wtdc
        lda wtr
        clc
        adc wtdy
        bmi .try
        cmp #11
        bcs .try
        sta wtdr
        tay
        lda rowmul,y
        clc
        adc wtdc
        tay
        lda MAP,y
        bne .try                        ; Ziel nicht frei
        ldx #MAXW - 1                   ; Ziel/Quelle von einem anderen wandernden Stein belegt?
.sp3:   lda wst,x
        beq .sp4
        lda wsc,x
        cmp wtdc
        bne +
        lda wsr,x
        cmp wtdr
        beq .try
+       lda wdc,x
        cmp wtdc
        bne .sp4
        lda wdr,x
        cmp wtdr
        beq .try
.sp4:   dex
        bpl .sp3
        lda wtdc                        ; Ball in der Naehe des Ziels?  (Zielfeld in Spielfeldpunkten: x = 16 + 16 * Spalte, y = 32 + 8 * Zeile)
        clc
        adc #1
        sta woxl
        lda #0
        sta woxh
        ldy #4
.sp5:   asl woxl
        rol woxh
        dey
        bne .sp5
        lda wtdr
        clc
        adc #4
        asl
        asl
        asl
        sta woy
        lda #10
        sta wm
        jsr w_ballov
        bcs .try
        ldx wcx                         ; Eintrag anlegen
        lda #1
        sta wst,x
        lda wtt
        sta wtp,x
        lda wtc
        sta wsc,x
        lda wtr
        sta wsr,x
        lda wtdc
        sta wdc,x
        lda wtdr
        sta wdr,x
        lda wtdx
        sta wdx,x
        lda wtdy
        sta wdy,x
        lda #0
.ifdef WTEST3
        ldy #100
        sty wpl,x
.else
        sta wpl,x
.endif
        sta wph,x
        lda wtc                         ; die Quellzelle leeren (Karte und Bildschirm)
        sta bcol
        lda wtr
        sta brow
        ldy wtr
        lda rowmul,y
        clc
        adc wtc
        tay
        lda #0
        sta MAP,y
        ldx wcx                         ; sichtbare Lage bei dem Anfangsfortschritt
        lda wpl,x
        sta wnl
        lda wph,x
        sta wnh
        jsr w_pos
        ldx wcx
        lda woxl
        sta wxl,x
        lda woxh
        sta wxh,x
        lda woy
        sta wyp,x
        jsr w_draw
.spx:   ldz #0
        rts
wdxt:   .byte $ff, 1, 0, 0              ; links, rechts, oben, unten
wdyt:   .byte 0, 0, $ff, 1

; w_step: ein Bild Fortschritt fuer den Eintrag wcx.
w_step:
.ifdef WTEST3
        rts                             ; Testfassung: wandernde Steine bleiben mitten in der Bewegung stehen
.endif
        ldx wcx
        lda wdx,x
        beq .wsv
        lda wpl,x                       ; waagerecht: 10 / 16 px je Bild, Ende bei 256
        clc
        adc #10
        sta wnl
        lda wph,x
        adc #0
        sta wnh
        bne .wsf
        beq .wsc
.wsv:   lda wpl,x                       ; senkrecht: 5 / 16 px je Bild, Ende bei 128
        clc
        adc #5
        sta wnl
        lda wph,x
        adc #0
        sta wnh
        bne .wsf
        lda wnl
        cmp #128
        bcs .wsf
.wsc:   jsr w_pos                       ; Lage bei dem neuen Fortschritt (woxl/woxh, woy)
        lda #1
        sta wm
        jsr w_ballov                    ; steht ein Ball im Weg?  Dann wartet der Stein
        bcs .wsh
        ldx wcx
        lda wnl
        sta wpl,x
        lda wnh
        sta wph,x
        lda woxl
        sta wxl,x
        lda woxh
        sta wxh,x
        lda woy
        sta wyp,x
.ifdef WFREEZE
        jsr w_draw
        ldx wcx
        lda wpl,x
        cmp #WFREEZE
        bcc .wfz1
.wfz:   jmp .wfz                        ; Testfassung: Bild einfrieren
.wfz1:  rts
.else
        jmp w_draw
.endif
.wsh:   rts
.wsf:   jmp w_finish

; w_pos: Lage des Eintrags wcx bei dem Fortschritt wnl/wnh in Spielfeldpunkten: woxl/woxh (16 Bit) und woy (8 Bit).
w_pos:
        ldx wcx
        lda wnl
        lsr
        lsr
        lsr
        lsr
        sta wd
        lda wnh
        asl
        asl
        asl
        asl
        ora wd
        sta wd                          ; Verschiebung in ganzen Punkten
        lda wsc,x
        clc
        adc #1
        sta woxl
        lda #0
        sta woxh
        ldy #4
.wp1:   asl woxl                        ; x = 16 * (Spalte + 1)
        rol woxh
        dey
        bne .wp1
        lda wsr,x
        clc
        adc #4
        asl
        asl
        asl
        sta woy                         ; y = 8 * (Zeile + 4)
        lda wdx,x
        beq .wp3
        bmi .wp2
        lda woxl
        clc
        adc wd
        sta woxl
        bcc .wp3
        inc woxh
        bcs .wp3
.wp2:   lda woxl
        sec
        sbc wd
        sta woxl
        bcs .wp3
        dec woxh
.wp3:   lda wdy,x
        beq .wp5
        bmi .wp4
        lda woy
        clc
        adc wd
        sta woy
        rts
.wp4:   lda woy
        sec
        sbc wd
        sta woy
.wp5:   rts

; w_ballov: ueberlappt ein aktiver Ball (6 x 6 px) das Rechteck x = woxl/woxh, y = woy, 16 x 8 px, um wm Punkte vergroessert?  C = 1 ja.
w_ballov:
        ldy #2
.bo1:   lda bact,y
        beq .bon
        lda b_xl,y                      ; Ball-X in Punkten = High-Byte * 4 + Low-Byte / 64 (9 Bit)
        lsr
        lsr
        lsr
        lsr
        lsr
        lsr
        sta wt0
        lda b_xh,y
        asl
        asl
        ora wt0
        sta wbx
        lda b_xh,y
        lsr
        lsr
        lsr
        lsr
        lsr
        lsr
        sta wbx + 1
        lda b_yl,y
        lsr
        lsr
        lsr
        lsr
        lsr
        lsr
        sta wt0
        lda b_yh,y
        asl
        asl
        ora wt0
        sta wby
        sec
        lda wbx
        sbc woxl
        sta ovdl
        lda wbx + 1
        sbc woxh
        sta ovdh
        lda wm
        clc
        adc #5
        sta ovk1
        lda wm
        clc
        adc #15
        sta ovk2
        jsr ov1
        bcc .bon
        sec
        lda wby
        sbc woy
        sta ovdl
        lda #0
        sbc #0
        sta ovdh
        lda wm
        clc
        adc #5
        sta ovk1
        lda wm
        clc
        adc #7
        sta ovk2
        jsr ov1
        bcc .bon
        rts                             ; C = 1
.bon:   dey
        bpl .bo1
        clc
        rts

; ov1: Ueberlappung auf einer Achse.  ovdl/ovdh = Unterschied (Ball - Rechteck, 16 Bit mit Vorzeichen), ovk1 = Ballbreite - 1 + Rand, ovk2 = Rechteckbreite
;   - 1 + Rand.  C = 1, wenn  -ovk1 <= Unterschied <= ovk2.
ov1:    clc
        lda ovdl
        adc ovk1
        lda ovdh
        adc #0
        bmi .ovn
        sec
        lda ovk2
        sbc ovdl
        lda #0
        sbc ovdh
        bmi .ovn
        sec
        rts
.ovn:   clc
        rts

; wcoll: trifft der Ball (Nullseite, nach der Bewegung) einen wandernden Stein?  Z = 1 nein; sonst A = Typ, bcol/brow/bidx gesetzt (wie probe).
wcoll:
        jsr get_px
        lda byl
        lsr
        lsr
        lsr
        lsr
        lsr
        lsr
        sta wt0
        lda byh
        asl
        asl
        ora wt0
        sta wby
        ldx #MAXW - 1
.wc1:   lda wst,x
        beq .wcn
        sec
        lda pxl
        sbc wxl,x
        sta ovdl
        lda pxh
        sbc wxh,x
        sta ovdh
        lda #5
        sta ovk1
        lda #15
        sta ovk2
        jsr ov1
        bcc .wcn
        sec
        lda wby
        sbc wyp,x
        sta ovdl
        lda #0
        sbc #0
        sta ovdh
        lda #5
        sta ovk1
        lda #7
        sta ovk2
        jsr ov1
        bcc .wcn
        stx wcx
        jmp w_hit
.wcn:   dex
        bpl .wc1
        lda #0
        rts

; wshot: trifft der Schuss shi (Spitze) einen wandernden Stein?  C = 1 ja (A = Typ, bcol/brow/bidx gesetzt).
wshot:
        ldx shi
        lda shu,x
        asl
        sta wbx
        lda #0
        rol
        sta wbx + 1
        lda shy,x
        sta wby
        ldx #MAXW - 1
.ws1:   lda wst,x
        beq .wsn
        stx wcx2
        sec
        lda wbx
        sbc wxl,x
        sta ovdl
        lda wbx + 1
        sbc wxh,x
        sta ovdh
        lda #1
        sta ovk1
        lda #15
        sta ovk2
        jsr ov1
        bcc .ws2
        ldx wcx2
        sec
        lda wby
        sbc wyp,x
        sta ovdl
        lda #0
        sbc #0
        sta ovdh
        lda #3
        sta ovk1
        lda #7
        sta ovk2
        jsr ov1
        bcc .ws2
        ldx wcx2
        stx wcx
        jsr w_hit
        sec
        rts
.ws2:   ldx wcx2
.wsn:   dex
        bpl .ws1
        clc
        rts

; w_hit: der Stein wcx wurde getroffen: an der naechstliegenden Zelle stehen lassen, normal zeichnen, Eintrag frei.  Rueckgabe A = Typ (<> 0), Z = 0.
w_hit:
.ifdef WTEST
        inc wdbg                        ; Testfassung: Treffer auf wandernde Steine zaehlen
.endif
        ldx wcx
        lda wdx,x
        beq .wh1
        lda wpl,x                       ; waagerecht: ab 128 (halber Weg) gilt das Ziel
        cmp #128
        jmp .wh2
.wh1:   lda wpl,x                       ; senkrecht: ab 64
        cmp #64
.wh2:   bcs .whd
        lda wsc,x
        sta bcol
        lda wsr,x
        sta brow
        bcc .whs
.whd:   lda wdc,x
        sta bcol
        lda wdr,x
        sta brow
.whs:   lda wtp,x
        sta wtt
        jsr w_free                      ; Zellen leeren, Eintrag freigeben
        ldy brow
        lda rowmul,y
        clc
        adc bcol
        sta bidx
        tay
        lda wtt
        sta MAP,y
        jsr draw_brick                  ; (A = Typ) an bcol/brow zeichnen
        ldz #0
        lda wtt
        rts

; w_finish: die Bewegung ist zu Ende: Stein in die Zielzelle setzen.
w_finish:
.ifdef WTEST
        inc wdbg + 1                    ; Testfassung: beendete Wanderungen zaehlen
.endif
        ldx wcx
        lda wdc,x
        sta bcol
        lda wdr,x
        sta brow
        lda wtp,x
        sta wtt
        jsr w_free
        ldy brow
        lda rowmul,y
        clc
        adc bcol
        tay
        lda wtt
        sta MAP,y
        jsr draw_brick
        ldz #0
        rts

; w_free: die Zellen von Quelle und Ziel des Eintrags wcx auf dem Bildschirm leeren (Streifen weg) und den Eintrag freigeben.
w_free:
        ldx wcx
        lda wsc,x
        ldy wsr,x
        jsr w_clr
        ldx wcx
        lda wdc,x
        ldy wdr,x
        jsr w_clr
        ldx wcx
        lda #0
        sta wst,x
        rts

; w_clr: Stein-Zelle (A = Spalte, Y = Zeile) auf dem Bildschirm leeren.
w_clr:  asl
        clc
        adc #2
        tax
        tya
        clc
        adc #4
        tay
        jsr set_rowptr
        jmp clear_cells

; w_draw: den Stein wcx in seiner Lage (Fortschritt wpl/wph) zeichnen.
w_draw:
        ldx wcx
        lda wsc,x
        ldy wsr,x
        jsr w_clr
        ldx wcx
        lda wdc,x
        ldy wdr,x
        jsr w_clr
        ldx wcx
        lda wpl,x                       ; Verschiebung in Bildpunkten der Betriebsart: 320 x 200: Fortschritt / 16, 640 x 400: / 8
        sta wt0
        lda wph,x
        sta wt1
        ldy #4
        lda hires
        beq .d0
        dey
.d0:    lsr wt1
        ror wt0
        dey
        bne .d0
        lda wt0
        sta wd
        lda hires                       ; Betriebsart: Streifenfelder, Steinmuster
        beq .dlo
        lda #4
        sta wbwc
        lda #2
        sta wbhc
        lda #32
        sta wbw
        lda #16
        sta wbh
        lda #0
        sta wbufl
        lda #$20
        sta wbufh                       ; Streifen ab $12000 + 768 * k
        ldy wcx
        beq .dh1
        lda #$23
        sta wbufh
.dh1:   lda #$80
        sta wcodel
        lda #4
        sta wcodeh                      ; Zeichennummer $480 + 12 * k
        ldy wcx
        beq .dh2
        lda #$8c
        sta wcodel
.dh2:   ldy wtp,x                       ; Steinmuster: Farbgruppe * 512 ab $14800
        lda brick_grp,y
        asl
        clc
        adc #$48
        sta wsbh
        lda #0
        sta wsbl
        jmp .dgo
.dlo:   lda #2
        sta wbwc
        lda #1
        sta wbhc
        lda #16
        sta wbw
        lda #8
        sta wbh
        lda #0
        sta wbufl
        lda #$48
        clc
        adc wcx
        sta wbufh                       ; Streifen ab $14800 + 256 * k
        lda #$20
        sta wcodel
        lda wcx
        asl
        asl
        clc
        adc wcodel
        sta wcodel
        lda #5
        sta wcodeh                      ; Zeichennummer $520 + 4 * k
        lda wtp,x                       ; Steinmuster: (Typ - 1) * 128 ab $12000
        sec
        sbc #1
        lsr
        clc
        adc #$20
        sta wsbh
        lda wtp,x
        sec
        sbc #1
        and #1
        lsr
        ror
        sta wsbl                        ; (Typ - 1) und 1 -> $80 / $00
.dgo:   ldx wcx                         ; Ursprung = obere linke Ecke der beiden Zellen (Quelle und Ziel): x0 = 16 * (Spalte + 1), y0 = 8 * (Zeile + 4)
        lda wsc,x                       ;   (der Stein liegt immer innerhalb dieser Zellen; so greift der Streifen nie in Nachbarzellen)
        cmp wdc,x
        bcc +
        lda wdc,x
+       clc
        adc #1
        sta wx0l
        lda #0
        sta wx0h
        ldy #4
        lda hires
        beq .g1
        iny
.g1:    asl wx0l
        rol wx0h
        dey
        bne .g1
        lda wsr,x
        cmp wdr,x
        bcc +
        lda wdr,x
+       clc
        adc #4
        sta wy0l
        lda #0
        sta wy0h
        ldy #3
        lda hires
        beq .g2
        iny
.g2:    asl wy0l
        rol wy0h
        dey
        bne .g2
        lda wdx,x                       ; Abstand vom Ursprung: nach rechts/unten = Verschiebung, nach links/oben = Laenge - Verschiebung
        clc
        adc wdy,x
        bpl .g4
        lda wdx,x
        beq .g3
        lda wbw
        bne .g3a
.g3:    lda wbh
.g3a:   sec
        sbc wd
        sta wd
.g4:    lda wdx,x
        beq .g5
        lda wx0l
        clc
        adc wd
        sta wx0l
        bcc .g8
        inc wx0h
        jmp .g8
.g5:    lda wy0l
        clc
        adc wd
        sta wy0l
        bcc .g8
        inc wy0h
.g8:    lda wx0l                        ; Zeichenspalte und Verschiebung
        and #7
        sta wsx
        lda wx0h
        sta wt1
        lda wx0l
        sta wt0
        lsr wt1
        ror wt0
        lsr wt1
        ror wt0
        lsr wt1
        ror wt0
        lda wt0
        sta wcc
        lda wy0l
        and #7
        sta wsy
        lda wy0h
        sta wt1
        lda wy0l
        sta wt0
        lsr wt1
        ror wt0
        lsr wt1
        ror wt0
        lsr wt1
        ror wt0
        lda wt0
        sta wcr
        ldx wcx                         ; Streifengroesse: waagerecht (Breite + 1 bei Verschiebung) x Hoehe, senkrecht Breite x (Hoehe + 1 bei Verschiebung)
        lda wbwc
        sta wscols
        lda wbhc
        sta wsrows
        lda wdx,x
        beq .g9
        lda wsx
        beq .ga
        inc wscols
        jmp .ga
.g9:    lda wsy
        beq .ga
        inc wsrows
.ga:    lda wscols                      ; Anzahl Zeichen * 64 Byte = Laenge des Streifens
        sta wt0
        lda #0
        sta wt1
        lda wscols
        ldy wsrows
        dey
        beq .gb
.gm:    clc
        adc wscols
        dey
        bne .gm
.gb:    sta wnc                         ; Zeichen insgesamt
        sta wt0
        lda #0
        sta wt1
        ldy #6
.gl:    asl wt0
        rol wt1
        dey
        bne .gl
        lda wt0
        sta wz_dma + 1                  ; Laenge
        lda wt1
        sta wz_dma + 2
        lda wbufl
        sta wz_dma + 6                  ; Ziel
        lda wbufh
        sta wz_dma + 7
        lda #0
        sta $d702
        sta $d704
        lda #>wz_dma
        sta $d701
        lda #<wz_dma
        sta $d700                       ; DMA: Streifen loeschen
        lda #$01
        sta W_SRC + 2
        sta $e2
        lda #0
        sta W_SRC + 3
        sta $e3
        sta wpy
.rl:    lda wpy                         ; Quellzeile: Muster + (Zeile / 8) * (Breite in Zeichen * 64) + (Zeile mod 8) * 8
        and #7
        asl
        asl
        asl
        clc
        adc wsbl
        sta W_SRC
        lda #0
        adc wsbh
        sta W_SRC + 1
        lda hires
        beq .r1
        lda wpy
        lsr
        lsr
        lsr
        clc
        adc W_SRC + 1                   ; hohe Aufloesung: +256 je Zeichenzeile
        sta W_SRC + 1
.r1:    lda wpy                         ; Zielzeile y = Zeile + wsy: Streifen + (y / 8) * Streifenbreite * 64 + (y mod 8) * 8
        clc
        adc wsy
        sta wt0
        and #7
        asl
        asl
        asl
        sta wdbl
        lda wt0
        lsr
        lsr
        lsr
        tax                             ; Zeichenzeile des Ziels
        lda #0
        sta wt0
        sta wt1
        cpx #0
        beq .r3
.r2:    lda wt0                         ; (y / 8) * Streifenbreite
        clc
        adc wscols
        sta wt0
        dex
        bne .r2
.r3:    lda wt0                         ; * 64
        sta wt1
        lda #0
        sta wt0
        lda wt1
        lsr
        ror wt0
        lsr
        ror wt0                         ; (wt1:wt0) = Wert * 256 / 4 = Wert * 64
        sta wt1
        lda wt0
        clc
        adc wdbl
        sta wdbl
        lda wt1
        adc #0
        sta wdbh
        lda wdbl
        clc
        adc wbufl
        sta wdbl
        lda wdbh
        adc wbufh
        sta wdbh
        lda #0
        sta wpx
.pl:    lda wpx                         ; Index in der Quellzeile: (x / 8) * 64 + x mod 8
        and #7
        sta wt0
        lda wpx
        lsr
        lsr
        lsr
        asl
        asl
        asl
        asl
        asl
        asl
        clc
        adc wt0
        taz
        lda [W_SRC],z
        beq .pn                         ; 0 = durchsichtig
        sta wv
        lda wpx                         ; Ziel: Zeichen (x + wsx) / 8, Spalte (x + wsx) mod 8
        clc
        adc wsx
        sta wt0
        and #7
        sta wt1
        lda wt0
        lsr
        lsr
        lsr
        tax
        lda wdbl
        clc
        adc m64l,x
        sta $e0
        lda wdbh
        adc m64h,x
        sta $e1
        lda wt1
        taz
        lda wv
        sta [$e0],z
.pn:    inc wpx
        lda wpx
        cmp wbw
        bne .pl
        inc wpy
        lda wpy
        cmp wbh
        beq +
        jmp .rl
+       lda #0                          ; Streifen auf den Bildschirm schreiben (Zeichen in Zeilenfolge)
        sta wm
        sta wj
.sj:    lda #0
        sta wi
.si:    lda wcc
        clc
        adc wi
        sta pt_col
        lda wcr
        clc
        adc wj
        sta pt_row
        jsr rc_ptr
        lda wcodel
        clc
        adc wm
        pha
        lda wcodeh
        adc #0
        sta wt0
        ldz #0
        pla
        sta [sq],z
        lda wt0
        inz
        sta [sq],z
        lda #1
        sta [cq],z
        lda #0
        ldz #0
        sta [cq],z
        inc wm
        inc wi
        lda wi
        cmp wscols
        bne .si
        inc wj
        lda wj
        cmp wsrows
        bne .sj
        ldz #0
        rts
m64l:   .byte 0, 64, 128, 192, 0
m64h:   .byte 0, 0, 0, 0, 1
wz_dma: .byte $03                       ; Befehl: Fuellen mit 0
        .word 0                         ; Anzahl (wird gesetzt)
        .word 0                         ; Quelle: Fuellwert 0
        .byte 0
        .word 0                         ; Ziel (wird gesetzt)
        .byte 1                         ; Bank 1
        .byte 0, 0

wdbg:   .byte 0, 0                      ; (Testfassung WTEST: Zaehler Treffer / Ende)
ttc:    .byte 0
wst:    .byte 0, 0                      ; 0 = frei, 1 = wandert
wtp:    .byte 0, 0                      ; Typ
wsc:    .byte 0, 0                      ; Quellspalte / -zeile
wsr:    .byte 0, 0
wdc:    .byte 0, 0                      ; Zielspalte / -zeile
wdr:    .byte 0, 0
wdx:    .byte 0, 0                      ; Richtung (-1, 0, +1)
wdy:    .byte 0, 0
wpl:    .byte 0, 0                      ; Fortschritt in 1/16 Spielfeldpunkt
wph:    .byte 0, 0
wxl:    .byte 0, 0                      ; sichtbare Lage der linken oberen Ecke in Spielfeldpunkten
wxh:    .byte 0, 0
wyp:    .byte 0, 0
wcx:    .byte 0
wcx2:   .byte 0
wtry:   .byte 0
wtc:    .byte 0
wtr:    .byte 0
wtt:    .byte 0
wtdx:   .byte 0
wtdy:   .byte 0
wtdc:   .byte 0
wtdr:   .byte 0
wnl:    .byte 0
wnh:    .byte 0
wd:     .byte 0
woxl:   .byte 0
woxh:   .byte 0
woy:    .byte 0
wm:     .byte 0
wt0:    .byte 0
wt1:    .byte 0
wbx:    .word 0
wby:    .byte 0
ovdl:   .byte 0
ovdh:   .byte 0
ovk1:   .byte 0
ovk2:   .byte 0
wbwc:   .byte 0
wbhc:   .byte 0
wbw:    .byte 0
wbh:    .byte 0
wbufl:  .byte 0
wbufh:  .byte 0
wcodel: .byte 0
wcodeh: .byte 0
wsbl:   .byte 0
wsbh:   .byte 0
wx0l:   .byte 0
wx0h:   .byte 0
wy0l:   .byte 0
wy0h:   .byte 0
wsx:    .byte 0
wsy:    .byte 0
wcc:    .byte 0
wcr:    .byte 0
wscols: .byte 0
wsrows: .byte 0
wnc:    .byte 0
wpx:    .byte 0
wpy:    .byte 0
wdbl:   .byte 0
wdbh:   .byte 0
wv:     .byte 0
wi:     .byte 0
wj:     .byte 0


; draw_hud: Kopfzeile "PUNKTE 000000  LEVEL n  BAELLE n".
draw_hud:
        lda demo
        beq .dh0
        rts                             ; Demo: kein Punktestand, die Zeile gehoert dem Ticker
.dh0:   lda #1
        sta txtcol
        lda #<h_score
        sta tp
        lda #>h_score
        sta tp + 1
        ldx #0
        ldy #0
        jsr puts
        lda sc2
        ldx #7
        ldy #0
        jsr put_bcd
        lda sc1
        ldx #9
        ldy #0
        jsr put_bcd
        lda sc0
        ldx #11
        ldy #0
        jsr put_bcd
        lda #<h_level
        sta tp
        lda #>h_level
        sta tp + 1
        ldx #16
        ldy #0
        jsr puts
        lda level                       ; Levelnummer 1..24 in zwei Ziffern (Zehner per wiederholtem Abziehen)
        clc
        adc #1
        ldx #0                          ; X = Zehnerziffer
.tens:  cmp #10
        bcc .ones
        sbc #10                         ; (Uebertrag ist gesetzt: Abziehen ohne sec)
        inx
        bne .tens
.ones:  pha                             ; Einerziffer merken
        txa
        beq .notens                     ; Zehner 0: Leerzeichen statt "0"
        clc
        adc #$30
        bne .tput
.notens: lda #CH_SPACE
.tput:  ldx #22
        ldy #0
        jsr put_cell
        pla
        clc
        adc #$30
        ldx #23
        ldy #0
        jsr put_cell
        lda #<h_balls
        sta tp
        lda #>h_balls
        sta tp + 1
        ldx #26
        ldy #0
        jsr puts
        lda lives
        clc
        adc #$30
        ldx #33
        ldy #0
        jmp put_cell

h_score: .screen "punkte"
         .byte 0
h_level: .screen "level"
         .byte 0
h_balls: .screen "baelle"
         .byte 0

; Farb-RAM-Wert je Steintyp = 8 + Farbe (Bit 3 = Mehrfarbzeichen; nur die Farben 0..7 sind moeglich): 1 rot, 2 gelb, 3 gruen, 4 tuerkis,
; 5 blau, 6 violett, 7 weiss, 8 rot; 9 Silber = weiss, 10 Gold = gelb, 11 unzerstoerbar = schwarz (dunkles Metall), 12 explosiv = rot, 13 Ueberraschung = weiss
brick_color: .byte 8, 10, 15, 13, 11, 14, 12, 9, 10, 9, 15, 8, 10, 9, 8, 8, 10, 12, 14, 11, 13, 15, 9, 9
; Punkte je Treffer (BCD, ein Byte): 10 * Typ fuer 1..8, Silber 50, Gold 90, unzerstoerbar 0, explosiv/Ueberraschung 10
brick_pts:   .byte 0, $10, $20, $30, $40, $50, $60, $70, $80, $50, $90, 0, $10, $10, 0, 0, $20, $20, $20, $20, $20, $20, $20, $20

; ---------------------------------------------------------------------------------------------------------------------
; Spielschleife
; ---------------------------------------------------------------------------------------------------------------------
; play: spielt bis zum Ende eines Levels oder Balls.
;   Rueckgabe A: 0 = Level geschafft, 1 = Ball verloren, 2 = Abbruch.
play:   jsr wait_frame                  ; ab hier ~ 300 Rasterzeilen Zeit bis zum naechsten Bild
.ifdef TESTFPS
        inc $c200                       ; Testfassung: Bildzaehler (16 Bit) - zeigt, ob Bilder ausgelassen werden (ruckelt)
        bne .fp1
        inc $c201
.fp1:   lda $c200
        and #15
        bne .fp2
        lda #1
        sta txtcol                      ; weiss (Farbe < 8: normale Zeichen)
        lda $c201                       ; alle 16 Bilder: Zaehler als Hexziffern in Zeile 24, Spalte 6..9 anzeigen
        ldx #6
        ldy #24
        jsr put_bcd
        lda $c200
        ldx #8
        ldy #24
        jsr put_bcd
.fp2:
.endif
        lda demo
        beq +
        jsr mus_tick                    ; in der Demo spielt die Musik weiter (Effekte sind dort stumm)
.ifndef NOTICKER
        jsr ticker_tick                 ; Ticker oben (Spielname und Untertitel im Wechsel)
.endif
+       jsr update_sprites              ; zuerst die Sprites setzen (Zustand des vorigen Durchlaufs), solange der Strahl unten steht
        jsr sfx_update
        jsr get_input                   ; mask = Eingabe dieses Bildes
        jsr mode_key                    ; Taste R: Aufloesung umschalten
        bcc .pk1
        jsr toggle_game
.pk1:
        lda mask
        and #IN_ABORT
        beq +
        lda #2
        rts
+       jsr fx_frame                    ; Randblitz und Glanzlicht
        jsr tick_timers                 ; Extra-Zeiten herunterzaehlen
        jsr crumble_tick                ; Bruch-Animation der zerstoerten Steine
        jsr particle_tick               ; Splitter
        jsr wander_tick                 ; wandernde Steine
.ifdef TESTTOG
        inc ttc                         ; Testfassung: alle 256 Bilder die Aufloesung umschalten
        lda ttc
        bne .ttg
        jsr toggle_game
.ttg:
.endif
        jsr move_paddle
        jsr update_cap                  ; Kapsel fallen lassen / auffangen
        jsr update_shots                ; Laserschuesse bewegen
        lda stuck
        beq .live
        jsr place_ball                  ; Ball klebt am Schlaeger und folgt ihm
        lda mask
        and #IN_FIRE
        beq play
        jsr launch                      ; Feuer: Ball abschiessen
        jmp play
.live:  lda mask
        and #IN_FIRE
        beq .nofire
        lda tlas
        beq .nofire
        jsr laser_fire                  ; Laser aktiv und Feuer gedrueckt: zwei Schuesse
.nofire:
        jsr move_all                    ; alle aktiven Baelle bewegen; A = 1 wenn keiner mehr im Spiel ist
        cmp #1
        beq .gone
        lda lkind
        beq .nk                         ; normales Level: nur die Steine zaehlen
        cmp #1
        bne .boss
        lda btime                       ; Bonuslevel: Ende, wenn die Zeit um ist (oder alle Steine weg sind)
        beq .cleared
        bne .nk
.boss:  lda cores                       ; Bosslevel: Ende, wenn alle Kerne zerstoert sind (uebrige Steine zaehlen nicht)
        bne .bplay
        beq .cleared
.nk:    lda bricks
        bne .bplay                      ; noch Steine uebrig
.cleared:
        lda #7
        ldx #25
        jsr flash                       ; Level geschafft: gelber Rand
        lda #SFX_EXTRA                  ; Level geschafft
        jsr sfx_play
        ldx #60
        jsr wait_frames
        lda #0
        rts
.gone:  lda #2
        ldx #12
        jsr flash                       ; Ball verloren: roter Randblitz
        lda #SFX_LOSE
        jsr sfx_play
        ldx #50
        jsr wait_frames
        lda #1
        rts
.bplay: jmp play

; get_input: mask aus Joystick/Tastatur; im Demo-Modus vom Rechner.  Dann beendet jede echte Eingabe die Demo (IN_ABORT).
;   Danach wirkt das Extra "umgekehrte Steuerung" (links <-> rechts vertauscht).
get_input:
.ifdef AUTOPLAY
        jsr auto_input
.else
.ifdef TESTLOSE
        lda #IN_FIRE                    ; Testfassung: nur schiessen, den Schlaeger nie bewegen -> der Ball geht verloren
        sta mask
.else
        lda demo
        beq .real
        jsr auto_input                  ; Demo: der Rechner spielt ...
        jsr input_poll                  ; ... bis der Spieler irgendetwas drueckt
        beq .rev
        lda #IN_ABORT
        sta mask
        rts
.real:  jsr input_poll
        sta mask
.endif
.endif
.rev:   lda trev
        beq .gi
        lda mask
        and #IN_LEFT | IN_RIGHT
        beq .gi
        lda mask
        eor #IN_LEFT | IN_RIGHT         ; beide Richtungsbits kippen: links wird rechts und umgekehrt
        sta mask
.gi:    rts

; auto_input: Der Rechner spielt.  Der Schlaeger soll den Ball mit dem Zielpunkt atgt (8..23 px von der linken Kante) treffen,
;   damit unterschiedliche Winkel vorkommen.  Differenz = (Schlaegerkante + atgt) - (Ballmitte); > 2 -> links, < -2 -> rechts.
auto_input:
        jsr pick_ball                   ; dem tiefsten Ball folgen
        jsr get_px                      ; pxl/pxh = linke Ballkante in px
        lda pxl
        clc
        adc #3                          ; Ballmitte = Kante + 3 (Ball 6 px breit)
        sta t0
        lda pxh
        adc #0
        sta t1
        lda padl
        clc
        adc atgt
        sta t2
        lda padh
        adc #0
        sta t3
        sec
        lda t2
        sbc t0                          ; Differenz Ziel - Ball (16 Bit, vorzeichenbehaftet)
        tax
        lda t3
        sbc t1
        bmi .neg                        ; Ziel liegt links vom Ball -> Schlaeger nach rechts
        bne .left                       ; >= 256: weit rechts
        cpx #6
        bcc .none                       ; |Differenz| unter 6 px: nichts tun (Totzone: der Schlaeger bewegt sich 4 px je Bild, bei einer
                                        ; Schwelle von 2 px schwingt er um den Zielpunkt hin und her und die Demo wirkt ruckelig)
.left:  lda #IN_LEFT | IN_FIRE
        bne .set
.neg:   cmp #$ff
        bne .right                      ; weit links
        cpx #$fa
        bcs .none                       ; -6 .. -1: klein (Totzone, siehe oben)
.right: lda #IN_RIGHT | IN_FIRE
        bne .set
.none:  lda #IN_FIRE
.set:   sta mask
        rts

; move_paddle: Schlaeger um 4 px je Bild bewegen und auf 16..272 begrenzen (16 Bit: padh:padl).
move_paddle:
        lda mask
        and #IN_LEFT
        beq .mr
        sec
        lda padl
        sbc #4
        sta padl
        lda padh
        sbc #0
        sta padh
.mr:    lda mask
        and #IN_RIGHT
        beq .clamp
        clc
        lda padl
        adc #4
        sta padl
        lda padh
        adc #0
        sta padh
.clamp: lda padh
        bmi .low                        ; negativ: unter 0 gelaufen
        bne .high                       ; 256 und mehr
        lda padl
        cmp #16
        bcs .done                       ; 16..255: in Ordnung
.low:   lda #16                         ; links anschlagen
        sta padl
        lda #0
        sta padh
        rts
.high:  lda #48                         ; rechter Anschlag: 304 - Breite = 256 + (48 - Breite)
        sec
        sbc padw
        sta t0
        lda padl
        cmp t0
        bcc .done                       ; links davon: in Ordnung
        lda t0
        sta padl
.done:  rts

; place_ball: Ball haftet in der Schlaegermitte auf dem Schlaeger (stuck = 1).
;   X = (padl + 13) * 64 (Ballkante = Schlaegermitte padl + 16 - Ballbreite / 2 = padl + 13), Y = (184 - 6) * 64 = $2C80.
place_ball:
        lda #1
        sta stuck
        lda padw
        lsr                             ; halbe Schlaegerbreite
        sec
        sbc #3                          ; minus halbe Ballbreite -> Abstand der Ballkante von der Schlaegerkante
        clc
        adc padl
        sta bxl
        lda padh
        adc #0
        sta bxh
        ldx #6                          ; mal 64 = 6 mal links schieben
.pb:    asl bxl
        rol bxh
        dex
        bne .pb
        lda #$80
        sta byl
        lda #$2c
        sta byh
        lda #0
        sta cb
        jsr save_ball                   ; Ball 0 aktiv, die anderen aus (ein neuer Ball beginnt immer allein)
        lda #1
        sta bact
        lda #0
        sta bact + 1
        sta bact + 2
        rts

; load_ball / save_ball: Ball cb aus dem Ballspeicher in die Nullseite holen / dorthin zurueckschreiben.   Veraendert A, X.
load_ball:
        ldx cb
        lda b_xl,x
        sta bxl
        lda b_xh,x
        sta bxh
        lda b_yl,x
        sta byl
        lda b_yh,x
        sta byh
        lda b_vxl,x
        sta vxl
        lda b_vxh,x
        sta vxh
        lda b_vyl,x
        sta vyl
        lda b_vyh,x
        sta vyh
        rts
; save_ball: die Nullseitenwerte des aktuellen Balls (bxl..vyh) in den Ballspeicher (Schlitz cb) zurueckschreiben.
save_ball:
        ldx cb
        lda bxl
        sta b_xl,x
        lda bxh
        sta b_xh,x
        lda byl
        sta b_yl,x
        lda byh
        sta b_yh,x
        lda vxl
        sta b_vxl,x
        lda vxh
        sta b_vxh,x
        lda vyl
        sta b_vyl,x
        lda vyh
        sta b_vyh,x
        rts

; move_all: jeden aktiven Ball bewegen (load, move_ball, save); verlorene Baelle werden deaktiviert.
;   Rueckgabe A: 0 = mindestens ein Ball im Spiel, 1 = alle verloren.
move_all:
        lda #0
        sta nact
        sta cb
.ma:    ldx cb
        lda bact,x
        beq .man
        jsr load_ball
        jsr move_ball
        cmp #1
        bne .mok
        ldx cb
        lda #0
        sta bact,x                      ; dieser Ball ist unten herausgefallen
        beq .man
.mok:   jsr save_ball
        inc nact
.man:   inc cb
        lda cb
        cmp #3
        bne .ma
        lda nact
        bne .alive
        lda #1
        rts
.alive: lda #0
        rts

; pick_ball: den tiefsten aktiven Ball in die Nullseite laden (der Autoplay/Demo folgt ihm).
pick_ball:
        lda #0
        sta t2                          ; bester Y-Wert (High-Byte)
        sta t3                          ; bester Schlitz
        sta cb
.pk:    ldx cb
        lda bact,x
        beq .pkn
        lda b_yh,x
        cmp t2
        bcc .pkn
        sta t2
        stx t3
.pkn:   inc cb
        lda cb
        cmp #3
        bne .pk
        lda t3
        sta cb
        jmp load_ball

; multiball: aus dem ersten aktiven Ball zwei weitere Baelle erzeugen (Zonen 1 und 6 = -43 / +26 Grad), soweit Schlitze frei sind.
multiball:
        ldx #0
.mb1:   lda bact,x
        bne .mbf
        inx
        cpx #3
        bne .mb1
        rts                             ; kein aktiver Ball
.mbf:   stx cb
        jsr load_ball                   ; Quelle in die Nullseite
        lda #1
        sta t1                          ; naechste Zone: 1, dann 6
        ldx #0
.mb2:   lda bact,x
        bne .mb3                        ; Schlitz belegt
        stx t2
        lda t1
        jsr set_vel                     ; Geschwindigkeit fuer die Zone (Position bleibt die der Quelle)
        ldx t2
        stx cb
        jsr save_ball
        ldx t2
        lda #1
        sta bact,x
        lda #6
        sta t1
        ldx t2
.mb3:   inx
        cpx #3
        bne .mb2
        rts

; launch: Ball abschiessen, leicht schraeg (Zone 3 oder 4 = -9 / +9 Grad).
launch: lda #0
        sta stuck
        sta cb
        jsr load_ball                   ; Ball 0 (der haftende) in die Nullseite
        lda #SFX_START
        jsr sfx_play
        jsr rand_byte
        and #1
        clc
        adc #3                          ; Zone 3 oder 4
        jsr set_vel
        jmp save_ball                   ; Ball 0 zurueckschreiben

; set_vel: Geschwindigkeit aus der Richtungstabelle: Index = Stufe * 8 + Zone.   Eingabe: A = Zone 0..7.   Veraendert A, X, t0.
set_vel:
        sta t0
        lda speed                       ; wirksame Stufe = Stufe +/- 4 bei "schneller"/"langsamer", begrenzt auf 0..15
        ldx spdmod
        beq .sv
        cpx #1
        bne .slow
        clc
        adc #4
        cmp #16
        bcc .sv
        lda #15
        bne .sv
.slow:  sec
        sbc #4
        bcs .sv
        lda #0
.sv:    asl
        asl
        asl
        ora t0
        tax
        lda dir_xl,x
        sta vxl
        lda dir_xh,x
        sta vxh
        lda dir_yl,x
        sta vyl
        lda dir_yh,x
        sta vyh
        rts

; get_px: ganze Bildpunkte der linken/oberen Ballkante: (bxh:bxl) / 64 -> pxh:pxl.   Veraendert A, X.
get_px: lda bxl
        sta pxl
        lda bxh
        sta pxh
        ldx #6
.gp:    lsr pxh
        ror pxl
        dex
        bne .gp
        rts

; neg_vx / neg_vy: Geschwindigkeitskomponente umkehren (16 Bit: 0 - Wert).
neg_vx: sec
        lda #0
        sbc vxl
        sta vxl
        lda #0
        sbc vxh
        sta vxh
        rts
neg_vy: sec
        lda #0
        sbc vyl
        sta vyl
        lda #0
        sbc vyh
        sta vyh
        rts

; ---------------------------------------------------------------------------------------------------------------------
; move_ball: Ball um einen Schritt bewegen; Waende, Steine, Schlaeger.  Rueckgabe A: 0 = weiter, 1 = Ball unten heraus.
;   Erst die X-Bewegung (mit Wand- und Steinpruefung), dann die Y-Bewegung.  Bei Treffer wird die Bewegung dieser Achse
;   zurueckgenommen und die Geschwindigkeitskomponente umgekehrt.  Der Ball ist 6 x 6 px; abgetastet werden die beiden Ecken
;   der Seite, in die er sich bewegt (probe).
; ---------------------------------------------------------------------------------------------------------------------
move_ball:
        lda bxl                         ; alte X-Position merken
        sta oxl
        lda bxh
        sta oxh
        clc
        lda bxl
        adc vxl
        sta bxl
        lda bxh
        adc vxh
        sta bxh
        lda vxh
        bmi .mleft
        ; ---- Bewegung nach rechts: rechte Wand bei px >= 296 (Ball 6 px breit + 296 = 302 <= 303)
        lda bxh
        cmp #$4a                        ; 296 * 64 = $4A00
        bcs .xwall
        ldy #1                          ; Abtastversatz: rechte Ballkante
        bne .xprobe
.mleft: lda bxh
        cmp #4                          ; 16 * 64 = $0400
        bcc .xwall
        cmp #$c0
        bcs .xwall                      ; unter 0 gerutscht (Wert wie $FFxx)
        ldy #0                          ; linke Ballkante
.xprobe:
        sty dxo
        ldx #0                          ; obere Ballkante
        jsr probe
        bne .xbrick
        ldy dxo
        ldx #1                          ; untere Ballkante
        jsr probe
        bne .xbrick
        jsr wcoll                       ; wandernder Stein?
        bne .xbrick
        jmp .ymove
.xbrick:
        jsr hit_brick                   ; Stein getroffen: Punkte, ggf. entfernen
        lda tpierce
        beq .xback
        ldx bidx
        lda MAP,x
        bne .xback                      ; Stein steht noch (unzerstoerbar oder Boss-Kern): normal abprallen
        jmp .ymove                      ; Durchschlag: Stein ist weg, der Ball fliegt in X ohne Umkehr weiter
.xwall: lda #SFX_WALL
        jsr sfx_play
.xback: lda oxl                         ; Bewegung zuruecknehmen
        sta bxl
        lda oxh
        sta bxh
        jsr neg_vx
.ymove: lda byl
        sta oyl
        lda byh
        sta oyh
        clc
        lda byl
        adc vyl
        sta byl
        lda byh
        adc vyh
        sta byh
        lda vyh
        bmi .mup
        ; ---- Bewegung nach unten: Schlaeger oder verloren
        lda byh
        cmp #$2c
        bcc .ybrick0                    ; noch ueber der Schlaegerhoehe: nur Steine
        bne .pad                        ; > $2C00: sicher in der Schlaegerzone oder darunter
        lda byl
        cmp #$80
        bcc .ybrick0                    ; < $2C80 = 178 px: Ballunterkante noch nicht auf Schlaegerhoehe
.pad:   lda byh
        cmp #$30
        bcs .lost                       ; >= 192 px: unten heraus
        jsr paddle_hit                  ; trifft er den Schlaeger? (Rueckgabe C=1 = ja)
        bcs .padok
        lda #0                          ; verfehlt: weiter nach unten fallen
        rts
.padok: lda #0
        rts
.lost:  lda #1
        rts
.mup:   lda byh                         ; ---- Bewegung nach oben: Decke bei px < 16
        cmp #4
        bcc .ywall
        cmp #$c0
        bcs .ywall
.ybrick0:
        ldy #0
        lda vyh
        bmi +
        ldy #1                          ; nach unten: untere Ballkante abtasten
+       sty dxo                         ; (dxo dient hier als Versatz Y)
        ldy #0                          ; linke Ballkante
        ldx dxo
        jsr probe
        bne .ybrick
        ldy #1                          ; rechte Ballkante
        ldx dxo
        jsr probe
        bne .ybrick
        jsr wcoll                       ; wandernder Stein?
        bne .ybrick
        lda #0
        rts
.ybrick:
        jsr hit_brick
        lda tpierce
        beq .yback
        ldx bidx
        lda MAP,x
        bne .yback                      ; Stein steht noch: abprallen
        lda #0                          ; Durchschlag: weiterfliegen (Rueckgabe: Ball im Spiel)
        rts
.ywall: lda #SFX_WALL
        jsr sfx_play
.yback: lda oyl
        sta byl
        lda oyh
        sta byh
        jsr neg_vy
        lda #0
        rts

; probe: tastet einen Punkt der Ballkante in der Steinkarte ab.
;   Eingabe: Y = 0/1: X-Versatz (0 = linke Kante, 1 = rechte Kante: +5 px = +320 in 1/64),
;            X = 0/1: Y-Versatz (0 = obere Kante, 1 = untere Kante)
;   Rueckgabe: Z=1 (A=0) = kein Stein; sonst A = Typ, bcol/brow/bidx gesetzt.
;   Rechnung: Spalte = (X - 16 px) / 16 px = (bxh - 4) / 4 (High-Byte = px / 4);  Zeile = (Y - 32 px) / 8 px = (byh - 8) / 2.
probe:  stx ptB
        clc
        lda bxl
        adc offl,y
        lda bxh
        adc offh,y                      ; High-Byte der Abtastposition X
        sec
        sbc #4
        bcc .none                       ; links vom Steinfeld
        lsr
        lsr
        cmp #18
        bcs .none                       ; rechts davon
        sta bcol
        ldy ptB
        clc
        lda byl
        adc offl,y
        lda byh
        adc offh,y                      ; High-Byte der Abtastposition Y
        sec
        sbc #8
        bcc .none                       ; ueber dem Steinfeld
        lsr
        cmp #12
        bcs .none                       ; darunter
        sta brow
        tax
        lda rowmul,x                    ; Zeile * 18
        clc
        adc bcol
        sta bidx
        tax
        lda MAP,x                       ; Typ; setzt Z
        rts
.none:  lda #0
        rts

offl:   .byte $00, $40                  ; 5 px * 64 = 320 = $0140: Low-Byte
offh:   .byte $00, $01                  ;                          High-Byte
rowmul: .byte 0, 18, 36, 54, 72, 90, 108, 126, 144, 162, 180, 198       ; Zeile * 18

; hit_brick: Stein vom Typ A getroffen (bidx/bcol/brow gesetzt).
;   Typ 11 (unzerstoerbar): nur Klang.  Gold (10) wird zu Silber (9), Silber zu Typ 1; alle anderen verschwinden.
;   Fuer jeden Treffer gibt es Punkte nach brick_pts.
hit_brick:
        pha
        ldy bcol                        ; Stereo: der Klang kommt von der Seite des getroffenen Steins (Spalte 0..17 -> links..rechts)
        lda pan_col,y
        sta sfx_pan
        lda #1
        ldx #2
        jsr flash                       ; kurzer weisser Randblitz
        pla
        cmp #11
        bne +
        lda #SFX_HARD
        jmp sfx_play
+       cmp #16
        bcc .nocore
        sta t3                          ; Boss-Kern: jeder Treffer nimmt 1 Lebenspunkt, bei 0 ist der Kern zerstoert
        tay
        lda brick_pts,y
        jsr add_score
        ldx bidx
        lda MAP,x
        sec
        sbc #1
        cmp #15
        bne .corehit                    ; noch Lebenspunkte: Typ - 1
        lda #0                          ; zerstoert
        sta MAP,x
        dec bricks
        dec cores
        lda #SFX_BRICK
        jsr sfx_play
        lda #7
        ldx #8
        jsr flash                       ; gelber Randblitz fuer den Kern
        lda #0
        jmp .redraw
.corehit:
        sta MAP,x
        pha
        lda #SFX_HARD
        jsr sfx_play
        pla
        jmp .redraw
.nocore: pha
        sta t3                          ; Typ merken (fuer die Kapsel-Entscheidung)
        tay
        lda brick_pts,y
        jsr add_score
        pla
        ldx tpierce
        bne .rem                        ; Durchschlag: jeder Stein wird sofort zerstoert (auch Silber und Gold)
        cmp #10
        beq .gold
        cmp #9
        beq .silver
.rem:   lda #0                          ; normaler Stein: weg
        ldx bidx
        sta MAP,x
        lda t3
        jsr crumble_start               ; Bruch-Animation fuer den zerstoerten Stein (bcol/brow stehen noch)
        dec bricks
        lda #SFX_BRICK
        jsr sfx_play
        lda lkind
        cmp #1
        bne +
        sed                             ; Bonuslevel: 100 Punkte je Stein zusaetzlich
        clc
        lda sc1
        adc #1
        sta sc1
        lda sc2
        adc #0
        sta sc2
        cld
+       lda t3                          ; ein Ueberraschungsstein (Typ 13) laesst immer eine Kapsel fallen, andere in etwa 1 von 12
        cmp #13
        beq .drop
.ifdef TESTCAP
        bne .drop                       ; Testfassung: jeder Stein laesst eine Kapsel fallen
.endif
        jsr rand_byte
        cmp #21
        bcs .nodrop
.drop:  jsr spawn_cap
.nodrop:
        lda #0
        jmp .redraw
.gold:  lda #9
        bne .down
.silver: lda #1
.down:  ldx bidx
        sta MAP,x
        pha
        lda #SFX_HARD
        jsr sfx_play
        pla
.redraw:
        jsr draw_brick                  ; (bcol/brow stehen noch)
        jmp draw_score

; draw_score: nur die Punktzahl neu zeichnen.
draw_score:
        lda demo
        beq .dsc0
        rts
.dsc0:  lda #1
        sta txtcol
        lda sc2
        ldx #7
        ldy #0
        jsr put_bcd
        lda sc1
        ldx #9
        ldy #0
        jsr put_bcd
        lda sc0
        ldx #11
        ldy #0
        jmp put_bcd

; add_score: A = BCD-Punkte (00..99) zum Punktestand addieren.  Dezimalmodus: Die CPU rechnet die Bytes als zwei Dezimalziffern.
add_score:
        sed
        clc
        adc sc0
        sta sc0
        lda sc1
        adc #0
        sta sc1
        lda sc2
        adc #0
        sta sc2
        cld
        rts

; paddle_hit: Ball ist auf Schlaegerhoehe (178 <= Y < 192).  Prueft die Ueberlappung in X und prallt ab.
;   Ueberlappung: d = Ballkante - Schlaegerkante muss in -5 .. Breite-1 liegen (Ball 6 px breit).
;   Zone = (d + 3) / zstep (Zonenbreite = Breite / 8), hoechstens 7;  negative d -> Zone 0.   Rueckgabe: C=1 = abgeprallt.
paddle_hit:
        jsr get_px
        sec
        lda pxl
        sbc padl
        sta t0
        lda pxh
        sbc padh
        sta t1                          ; t1:t0 = d
        bmi .neg
        bne .miss                       ; d >= 256
        lda t0
        cmp padw
        bcs .miss                       ; d >= Breite
        clc
        adc #3                          ; Ballmitte = Kante + 3
        cmp padw
        bcc +
        lda padw                        ; hoechstens Breite - 1 (letzte Zone)
        sbc #1
+       ldx #0                          ; Zone = (Ballmitte - Schlaegerkante) / zstep, hoechstens 7
.zl:    cmp zstep
        bcc .zd
        sbc zstep
        inx
        bne .zl
.zd:    cpx #8
        bcc +
        ldx #7
+       txa
        jmp .bounce
.neg:   lda t0
        cmp #251                        ; d >= -5 ?
        bcc .miss
        lda #0
.bounce:
        sta t2                          ; Zone merken
        lda oyh                         ; kam der Ball von oben?  (alte Y-Lage + 6 px < Schlaegeroberkante: Y < 178 px = $2C80)
        cmp #$2c
        bcc .top
        bne .side
        lda oyl
        cmp #$80
        bcs .side
.top:   lda t2                          ; Einfluss der Schlaegerbewegung: nach links bewegt -> eine Zone weiter links, nach rechts -> rechts
        tax
        lda mask
        and #IN_LEFT
        beq .nl
        cpx #0
        beq .nl
        dex
.nl:    lda mask
        and #IN_RIGHT
        beq .nr
        cpx #7
        bcs .nr
        inx
.nr:    txa
        pha
        lda #$80                        ; Ball genau auf den Schlaeger setzen
        sta byl
        lda #$2c
        sta byh
        pla
        jsr set_vel
        lda sticky                      ; Klebe-Extra: Ball bleibt am Schlaeger haengen (nur bei einem einzelnen Ball in Schlitz 0)
        beq +
        lda bact + 1
        ora bact + 2
        bne +
        lda cb
        bne +
        lda #0
        sta sticky
        lda #1
        sta stuck
+       inc hits
        lda hits
        and #7
        bne +
        lda speed                       ; alle 8 Treffer eine Stufe schneller, bis 15
        cmp #15
        bcs +
        inc speed
+       lda #SFX_PADDLE
        jsr sfx_play
        jsr rand_byte                   ; neuer Zielpunkt fuer den Autoplay (8..23)
        and #15
        clc
        adc #8
        sta atgt
        sec
        rts
        ; ---- Seitenaufprall: der Ball war schon auf Schlaegerhoehe und wurde von der Seite getroffen (oder lief hinein)
.side:  lda padw                        ; links oder rechts der Schlaegermitte? Ballmitte = Kante + 3, Schlaegermitte = Kante + Breite / 2
        lsr
        sta t3
        ldx t1                          ; t1:t0 = d = Ballkante - Schlaegerkante (aus dem Anfang der Routine)
        bmi .sl                         ; d negativ: der Ball liegt links der Schlaegerkante
        lda t0
        clc
        adc #3                          ; Ballmitte relativ zur Schlaegerkante
        cmp t3
        bcs .sr                         ; rechts der Mitte
.sl:    lda vxh                         ; links: nach links wegprallen (vx negativ), Ball links neben den Schlaeger setzen
        bpl .sl1
        jmp .sl2
.sl1:   jsr neg_vx
.sl2:   sec
        lda padl
        sbc #6
        sta bxl
        lda padh
        sbc #0
        sta bxh
        jmp .sx
.sr:    lda vxh                         ; rechts: nach rechts wegprallen, Ball rechts neben den Schlaeger setzen
        bmi .sr1
        jmp .sr2
.sr1:   jsr neg_vx
.sr2:   clc
        lda padl
        adc padw
        sta bxl
        lda padh
        adc #0
        sta bxh
.sx:    ldx #6                          ; Pixel -> 1/64 px
.sxl:   asl bxl
        rol bxh
        dex
        bne .sxl
        lda #SFX_WALL
        jsr sfx_play
        clc                             ; kein Abprall vom Schlaeger: der Ball faellt weiter und geht meist verloren
        rts
.miss:  clc
        rts

; ---------------------------------------------------------------------------------------------------------------------
; update_sprites: Spritepositionen aus dem Spielzustand.
;   Ball: x = px + 24, y = py + 50.  Schlaeger: zwei Sprites nebeneinander (16 px sichtbar je Sprite) bei padl und padl + 16.
;   Das 9. Bit der X-Koordinaten steht gesammelt in $D010 (Bit n = Sprite n).
; ---------------------------------------------------------------------------------------------------------------------
; boss_attack: der Boss laesst eine boese Kapsel fallen (S = schmaler Schlaeger oder R = umgekehrte Steuerung) aus einer zufaelligen Spalte.
boss_attack:
        lda capact
        bne .ba9                        ; es faellt schon eine Kapsel
        jsr rand_byte
        and #15
        clc
        adc #1                          ; Spalte 1..16
        sta bcol
        lda #4
        sta brow                        ; Zeile 4 (mitten im Steinfeld)
        jsr rand_byte
        and #1
        clc
        adc #2                          ; erzwungene Art + 1: 2 = S, 3 = R
        sta capforce
        jmp spawn_cap
.ba9:   rts

; draw_status: Statuszeile (Bildschirmzeile 24): Bonus = verbleibende Sekunden, Boss = verbliebene Kerne; sonst leer.
draw_status:
        lda #7
        sta txtcol
        lda lkind
        beq .ds0
        cmp #1
        bne .dsb
        lda #<s_bonus
        sta tp
        lda #>s_bonus
        sta tp + 1
        ldx #17
        ldy #24
        jsr puts
        lda btime
        jmp .dsnum
.dsb:   lda #<s_boss
        sta tp
        lda #>s_boss
        sta tp + 1
        ldx #13
        ldy #24
        jsr puts
        lda cores
.dsnum: ldx #0                          ; Zahl 0..99 in zwei Ziffern (Zehner per Abziehen)
.dst:   cmp #10
        bcc .dso
        sbc #10
        inx
        bne .dst
.dso:   pha
        txa
        clc
        adc #$30
        ldx #25
        ldy #24
        jsr put_cell                    ; (Farbe: txtcol des Aufrufers)
        pla
        clc
        adc #$30
        ldx #26
        ldy #24
        jmp put_cell
.ds0:   rts
s_bonus: .screen "bonus   "
        .byte 0
s_boss: .screen "boss: kerne "
        .byte 0

; fx_frame: Randblitz herunterzaehlen und das Glanzlicht ueber die Steine laufen lassen (je zweites Bild ein Schritt).
fx_frame:
        lda bflash
        beq .fx1
        dec bflash
        bne .fx0
        jsr border_black                ; Blitz zu Ende
        jmp .fx1
.fx0:   jsr flash_fade                  ; sonst weich abblenden
.fx1:   lda tick
        and #1
        bne .fx9                        ; nur jedes zweite Bild
        lda glg
        cmp #31
        bcs .fx2                        ; Pause
        jsr glint_diag_white            ; neue Diagonale weiss
.fx2:   lda glg
        beq .fx3
        cmp #32
        bcs .fx3
        sec
        sbc #1
        jsr glint_diag_restore          ; vorige Diagonale wieder in Steinfarbe
.fx3:   inc glg
        lda glg
        cmp #100
        bcc .fx9
        lda #0
        sta glg
.fx9:   rts

; glint_diag_white / glint_diag_restore: A = Nummer der Diagonale g (Spalte + Zeile = g).  Alle vorhandenen Steine auf ihr werden weiss
;   gefaerbt bzw. in der richtigen Farbe neu gezeichnet.
glint_diag_white:
        ldx #1
        bne glint_run
glint_diag_restore:
        ldx #0
glint_run:
        stx t3                          ; 1 = weiss, 0 = zuruecksetzen
        sta t2                          ; g
        lda #0
        sta brow
.gr1:   lda t2
        sec
        sbc brow                        ; Spalte = g - Zeile
        bcc .gr9                        ; negativ: weiter unten gibt es nichts mehr auf dieser Diagonale
        cmp #18
        bcs .gr8
        sta bcol
        ldy brow
        lda rowmul,y
        clc
        adc bcol
        tax
        lda MAP,x
        beq .gr8                        ; kein Stein
        ldx t3
        bne .grw
        jsr draw_brick                  ; zuruecksetzen: normal zeichnen (A = Typ)
        jmp .gr8
.grw:   lda bcol                        ; weiss faerben: die weisse Glanzfassung des Steins (Typ 24) einsetzen
        asl
        clc
        adc #2
        tax
        lda brow
        clc
        adc #4
        tay
        jsr set_rowptr
        lda #0
        sta bph
        lda #24                         ; weisse Glanzfassung (Typ 24)
        jsr put_brick
.gr8:   inc brow
        lda brow
        cmp #12
        bne .gr1
.gr9:   rts

; update_sprites: Sprite-Lage und -Freigabe je Bild setzen (Baelle, Schlaeger, Kugelfarbe/Tempo-Glut, Kapsel, Schuesse).  320 x 200 hier,
;   640 x 400 in update_sprites_h.
update_sprites:
        lda hires
        beq .us0
        jmp update_sprites_h
.us0:   lda #0
        sta sprmsb
        sta sprena
        sta cb
.ub:    ldx cb                          ; Baelle 0..2 -> Sprites 0, 4, 5
        lda bact,x
        beq .ubn                        ; Schlitz leer: Sprite bleibt aus
        jsr load_ball
        jsr get_px                      ; pxl/pxh (bis 9 Bit)
        ldx cb
        ldy ball_reg,x                  ; Y = Registerversatz des Sprites (0, 8, 10)
        clc
        lda pxl
        adc #24
        sta VIC_SPR0_X,y
        lda pxh
        adc #0                          ; Bit 8 der Sprite-X-Koordinate
        beq +
        lda ball_bit,x
        ora sprmsb
        sta sprmsb
+       lda byl                         ; py = (Y * 4) / 256 = Y / 64
        sta t0
        lda byh
        sta t1
        asl t0
        rol t1
        asl t0
        rol t1
        lda t1
        clc
        adc #50
        sta VIC_SPR0_Y,y
        lda ball_bit,x
        ora sprena
        sta sprena
.ubn:   inc cb
        lda cb
        cmp #3
        bne .ub
        lda sprmsb
        sta t2
        clc
        lda padl
        adc #24
        sta VIC_SPR1_X
        lda padh
        adc #0
        asl                             ; Bit 1 = Sprite 1
        ora t2
        sta t2
        clc
        lda padl
        adc pad2o                       ; zweite Haelfte rechts daneben (16 oder 24 px) ...
        sta t0
        lda padh
        adc #0
        sta t1
        clc
        lda t0
        adc #24                         ; ... plus Sprite-Versatz 24 (16-Bit-Addition, damit Bit 8 stimmt)
        sta VIC_SPR2_X
        lda t1
        adc #0
        asl
        asl                             ; Bit 2 = Sprite 2
        ora t2
        sta t2
        lda #PADY + 50
        sta VIC_SPR1_Y
        sta VIC_SPR2_Y
        ldx #0                          ; Ballfarbe: blaue Kugel, bei Durchschlag goldene (Palette der Sprites 3-5 umschalten)
        lda tpierce
        beq .bc
        inx
.bc:    ldy #0                          ; Tempo-Glut: ab Stufe 4 faerbt sich die blaue Kugel zunehmend rot (Gruppe 1..3), die goldene bleibt
        cpx #0
        bne .bch
        lda speed
        lsr a
        lsr a
        tay
.bch:   cpx bpal
        bne .bcs
        cpy bheat
        beq .bcd
.bcs:   stx bpal
        sty bheat
        jsr set_ball_pal
.bcd:
        lda sprena                      ; Sprite-Freigabe: Baelle (aus der Schleife oben) + Schlaeger links
        ora #2
        ldx pad2o
        beq +
        ora #4                          ; zweite Schlaegerhaelfte (nicht beim schmalen Schlaeger)
+       ldx capact
        beq +
        ora #1                          ; Kapsel (Sprite 0)
        pha
        sec                             ; die Muenze liegt im 16-Farb-Sprite ab Spalte 0 (12 px breit, in der 16 px breiten Kachel um 2 px eingerueckt):
        lda capxl                       ; x = capx + 24 + 2 = capx + 26
        adc #25                         ; (Uebertrag gesetzt: +26)
        sta VIC_SPR0_X
        lda capxh
        adc #0                          ; Bit 0 = Sprite 0
        ora t2
        sta t2
        lda capfr                       ; Drehphase 0..5 = capfr / 4 -> Zeiger aus der Folge Vorn, 3/4, halb, Kante, halb, 3/4
        lsr
        lsr
        tax
        lda cap_frames,x
        sta SPTR
        lda capy
        clc
        adc #50
        sta VIC_SPR0_Y
        pla
+       sta t3                          ; Freigabebits bisher
        ldx #0                          ; Laserschuesse: Sprites 6 und 7
.ss:    lda shact,x
        beq .ssn
        lda shu,x                       ; x = 2 * u + 24
        asl
        sta t0
        lda #0
        rol
        sta t1                          ; (t1:t0) = 2 * u
        clc
        lda t0
        adc #24
        sta t0
        bcc +
        inc t1
+       lda shot_reg,x
        tay
        lda t0
        sta VIC_SPR0_X,y
        lda t1
        beq +
        lda shot_bit,x
        ora t2
        sta t2
+       lda shy,x
        clc
        adc #50
        sta VIC_SPR0_Y,y
        lda shot_bit,x
        ora t3
        sta t3
.ssn:   inx
        cpx #2
        bne .ss
        lda t3
        sta VIC_SPR_ENABLE
        lda t2
        sta VIC_SPR_X_MSB
        rts
; ---------------------------------------------------------------------------------------------------------------------
; Sprites in 640 x 400 (SPRH640 / SPRENV400): ein Spritepunkt = ein echter Bildpunkt.  Koordinaten: X' = x + 79, Y' = y + 107 (x, y = Bildpunkt links oben im
;   Anzeigebereich), X' hat 10 Bit ($D010 = Bit 8, $D05F = Bit 9), Y' 9 Bit ($D077 = Bit 8).  Die Spielkoordinaten sind 320er Punkte: x = 2 * px.
;   Aufteilung der Sprites: 3, 4, 5 Baelle; 0 Kapsel; 1, 2 und (wenn frei) 7 Schlaeger; 6, 7 Laserschuesse.
;   Schlaeger: schmal (16) = links + rechts zu je 16 Punkten; normal (32) = links, Mitte (verdoppelt: 32 Punkte), rechts = 64 Punkte; breit (48) = alle drei
;   verdoppelt = 96 Punkte; ist Sprite 7 durch einen Schuss belegt, bleibt nur links + rechts verdoppelt (64 Punkte).
; ---------------------------------------------------------------------------------------------------------------------
; spr_mode: Spriteeinstellungen der Betriebsart (aus mode_set): SPRH640 ($D054 Bit 4 ueber spr_h), SPRENV400 ($D076), Zeiger, Palette der Sprites 6 und 7.
spr_mode:
        lda hires
        bne .sm1
        lda #0                          ; 320 x 200
        sta spr_h
        sta $d076
        sta $d055                       ; Spritehoehe wieder 21 Zeilen
        sta $d05f
        sta $d077
        lda #$c0
        sta SPTR + 3
        sta SPTR + 4
        sta SPTR + 5
        lda #$e3
        sta SPTR + 6
        sta SPTR + 7
        lda #$d4
        sta SPTR
        lda padw
        beq .sm0
        jsr set_padw                    ; Zeiger und X-Verdopplung des Schlaegers wie in 320 x 200
.sm0:   ldx #0
        jmp set_s67_pal
.sm1:   lda #$10                        ; 640 x 400
        sta spr_h
        lda #$ff
        sta $d076
        sta $d055                       ; alle Sprites mit eigener Hoehe ...
        lda #16
        sta $d056                       ; ... 16 Zeilen (die Bloecke sind 128 Byte gross, sonst wuerden die Zeilen 16-20 den Nachbarblock zeigen)
        lda #HP_BALL
        sta SPTR + 3
        sta SPTR + 4
        sta SPTR + 5
        lda #HP_SHOT
        sta SPTR + 6
        sta SPTR + 7
        lda #0
        sta VIC_SPR_EXPAND_X
        ldx #1
        jmp set_s67_pal

; set_s67_pal: Palette der Sprites 6 und 7 (Eintraege 96-127 der Sprite-Bank 1): X = 0 Schuss (320 x 200), X = 1 Schlaegerrampe + Schuss (640 x 400).
set_s67_pal:
        php
        sei
        lda #$44
        sta $d070
        ldy #15
.s67:   cpx #0
        bne .s6h
        lda shot_r,y
        sta $d160,y
        sta $d170,y
        lda shot_g,y
        sta $d260,y
        sta $d270,y
        lda shot_b,y
        sta $d360,y
        sta $d370,y
        jmp .s6n
.s6h:   lda hpal_r,y
        sta $d160,y
        sta $d170,y
        lda hpal_g,y
        sta $d260,y
        sta $d270,y
        lda hpal_b,y
        sta $d360,y
        sta $d370,y
.s6n:   dey
        bpl .s67
        lda #$04
        sta $d070
        plp
        rts

; sph_put: Sprite A (0..7) an die Stelle X' = hx_h:hx_l, Y' = hy_h:hy_l setzen (die hohen Bits sammeln sich in hm10, hm5f, hm77).
sph_put:
        tax
        lda sbit,x
        sta hb
        txa
        asl
        tax
        lda hx_l
        sta $d000,x
        lda hy_l
        sta $d001,x
        lda hx_h
        and #1
        beq .sp1
        lda hm10
        ora hb
        sta hm10
.sp1:   lda hx_h
        and #2
        beq .sp2
        lda hm5f
        ora hb
        sta hm5f
.sp2:   lda hy_h
        and #1
        beq .sp3
        lda hm77
        ora hb
        sta hm77
.sp3:   rts
sbit:   .byte 1, 2, 4, 8, 16, 32, 64, 128
ball_spr: .byte 3, 4, 5
hx_l:   .byte 0
hx_h:   .byte 0
hy_l:   .byte 0
hy_h:   .byte 0
hb:     .byte 0
hm10:   .byte 0
hm5f:   .byte 0
hm77:   .byte 0
hena:   .byte 0
hexp:   .byte 0
pbx_l:  .byte 0                         ; Schlaeger: linke Kante X' (ohne Versatz)
pbx_h:  .byte 0
pn:     .byte 0                         ; pad_one: Spritenummer, Versatz, Zeiger, Verdopplung
po:     .byte 0
pp:     .byte 0
pe:     .byte 0

; hx_add: hx_h:hx_l += A (A = 0..255).
hx_add:
        clc
        adc hx_l
        sta hx_l
        bcc .ha1
        inc hx_h
.ha1:   rts
; hy_add: hy_h:hy_l += A.
hy_add:
        clc
        adc hy_l
        sta hy_l
        bcc .hb1
        inc hy_h
.hb1:   rts

; hsr5: (hx_h:hx_l) und (hy_h:hy_l) je 5 Stellen nach rechts: 16-Bit-Spielwert (6 Nachkommabits, Punkte der 320er Aufloesung) -> echte Bildpunkte.
hsr5:
        ldx #5
.hs5:   lsr hx_h
        ror hx_l
        lsr hy_h
        ror hy_l
        dex
        bne .hs5
        rts

; pad_one: ein Schlaegersprite: pn = Spritenummer, po = X-Versatz, pp = Zeiger, pe = 1: X-Verdopplung.
pad_one:
        lda pbx_l
        sta hx_l
        lda pbx_h
        sta hx_h
        lda po
        jsr hx_add
        ldy pn
        lda pp
        sta SPTR,y
        lda pn
        jsr sph_put
        ldy pn
        lda sbit,y
        ora hena
        sta hena
        lda pe
        beq .po1
        lda sbit,y
        ora hexp
        sta hexp
.po1:   rts

; update_sprites_h: wie update_sprites in 640 x 400: Sprite-Pixel sind echte Bildpunkte (Lage * 2, X-Super-MSB $D05F, Y-MSB $D077).
update_sprites_h:
        lda #0
        sta hm10
        sta hm5f
        sta hm77
        sta hena
        sta hexp
        sta cb
.hb0:   ldx cb                          ; Baelle: Sprites 3, 4, 5
        lda bact,x
        beq .hbn
        jsr load_ball
        lda bxl
        sta hx_l
        lda bxh
        sta hx_h
        lda byl
        sta hy_l
        lda byh
        sta hy_h
        jsr hsr5                        ; Bildpunkte = Spielwert / 32
        lda #77                         ; X' = x + 79 - 2 (die 12 Punkte breite Kugel liegt 2 Punkte eingerueckt in der 16 breiten Flaeche)
        jsr hx_add
        lda #105                        ; Y' = y + 107 - 2
        jsr hy_add
        ldx cb
        lda ball_spr,x
        jsr sph_put
        ldx cb
        lda ball_bit,x
        ora hena
        sta hena
.hbn:   inc cb
        lda cb
        cmp #3
        bne .hb0
        ; --- Schlaeger: linke Kante = 2 * pad + 79, Oberkante = 2 * 184 + 107 = 475
        lda padl
        asl
        sta pbx_l
        lda padh
        rol
        sta pbx_h
        lda pbx_l
        clc
        adc #79
        sta pbx_l
        bcc .hp0
        inc pbx_h
.hp0:   lda #<475
        sta hy_l
        lda #>475
        sta hy_h
        lda #0
        sta pe
        lda padw
        cmp #16
        beq .pm0
        ldx shact + 1                   ; Schuss 2 belegt Sprite 7: nur zwei verdoppelte Enden
        bne .pmf
        cmp #32
        beq .pm1
        lda #1                          ; breit: links, Mitte, rechts, alle verdoppelt (3 * 32 = 96 Punkte)
        sta pe
        lda #1
        sta pn
        lda #0
        sta po
        lda #HP_PADL
        sta pp
        jsr pad_one
        lda #2
        sta pn
        lda #32
        sta po
        lda #HP_PADM
        sta pp
        jsr pad_one
        lda #7
        sta pn
        lda #64
        sta po
        lda #HP_PADR
        sta pp
        jsr pad_one
        jmp .pdone
.pm1:   lda #0                          ; normal: links (16), Mitte verdoppelt (32), rechts (16) = 64 Punkte
        sta pe
        lda #1
        sta pn
        lda #0
        sta po
        lda #HP_PADL
        sta pp
        jsr pad_one
        lda #1
        sta pe
        lda #2
        sta pn
        lda #16
        sta po
        lda #HP_PADM
        sta pp
        jsr pad_one
        lda #0
        sta pe
        lda #7
        sta pn
        lda #48
        sta po
        lda #HP_PADR
        sta pp
        jsr pad_one
        jmp .pdone
.pm0:   lda #1                          ; schmal: links (16) + rechts (16) = 32 Punkte
        sta pn
        lda #0
        sta po
        sta pe
        lda #HP_PADL
        sta pp
        jsr pad_one
        lda #2
        sta pn
        lda #16
        sta po
        lda #HP_PADR
        sta pp
        jsr pad_one
        jmp .pdone
.pmf:   lda #1                          ; Ersatz: zwei verdoppelte Enden = 64 Punkte
        sta pe
        sta pn
        lda #0
        sta po
        lda #HP_PADL
        sta pp
        jsr pad_one
        lda #2
        sta pn
        lda #32
        sta po
        lda #HP_PADR
        sta pp
        jsr pad_one
.pdone:
        ; --- Kapsel (Sprite 0): X' = 2 * capx + 87 (Muenze in der Mitte der 32 Punkte breiten Steinzelle), Y' = 2 * capy + 111
        lda capact
        beq .hcn
        lda capxl
        asl
        sta hx_l
        lda capxh
        rol
        sta hx_h
        lda #87
        jsr hx_add
        lda capy
        asl
        sta hy_l
        lda #0
        rol
        sta hy_h
        lda #111
        jsr hy_add
        lda capfr                       ; Drehphase 0..5
        lsr
        lsr
        tax
        lda hcap_frames,x
        sta SPTR
        lda #0
        jsr sph_put
        lda hena
        ora #1
        sta hena
.hcn:   ; --- Laserschuesse: Sprites 6 und 7
        ldx #0
.hss:   lda shact,x
        beq .hsn
        lda shu,x                       ; X' = 4 * u + 79
        sta hx_l
        lda #0
        sta hx_h
        asl hx_l
        rol hx_h
        asl hx_l
        rol hx_h
        lda #79
        jsr hx_add
        lda shy,x                       ; Y' = 2 * y + 107
        asl
        sta hy_l
        lda #0
        rol
        sta hy_h
        lda #107
        jsr hy_add
        txa
        clc
        adc #6
        sta pn
        tay
        lda #HP_SHOT
        sta SPTR,y
        lda pn
        stx pe                          ; (X merken: sph_put veraendert X)
        jsr sph_put
        ldx pe
        ldy pn
        lda sbit,y
        ora hena
        sta hena
.hsn:   inx
        cpx #2
        bne .hss
        lda hena
        sta VIC_SPR_ENABLE
        lda hm10
        sta VIC_SPR_X_MSB
        lda hm5f
        sta $d05f
        lda hm77
        sta $d077
        lda hexp
        sta VIC_SPR_EXPAND_X
        rts
hcap_frames: .byte HP_CAP, HP_CAPA, HP_CAPB, HP_CAPC, HP_CAPB, HP_CAPA

; title_balls_tick_h: die drei Titelkugeln in 640 x 400: X' = 4 * u + 77, Y' = 2 * y + 105.
title_balls_tick_h:
        lda #0
        sta hm10
        sta hm5f
        sta hm77
        ldx #0
        stx cb
.thb:   ldx cb
        lda tbu,x                       ; X bewegen, an den Raendern (u 6 und 148) umkehren
        clc
        adc tbdu,x
        sta tbu,x
        cmp #6
        bcc .thf1
        cmp #148
        bcc .thx
.thf1:  sec
        lda #0
        sbc tbdu,x
        sta tbdu,x
.thx:   lda tby,x                       ; Y bewegen, Grenzen 60 und 205
        clc
        adc tbdy,x
        sta tby,x
        cmp #60
        bcc .thf2
        cmp #205
        bcc .thy
.thf2:  sec
        lda #0
        sbc tbdy,x
        sta tbdy,x
.thy:   lda tbu,x
        sta hx_l
        lda #0
        sta hx_h
        asl hx_l
        rol hx_h
        asl hx_l
        rol hx_h
        lda #77
        jsr hx_add
        ldx cb
        lda tby,x
        asl
        sta hy_l
        lda #0
        rol
        sta hy_h
        lda #105
        jsr hy_add
        ldx cb
        lda ball_spr,x
        jsr sph_put
        inc cb
        lda cb
        cmp #3
        bne .thb
        lda hm10
        sta VIC_SPR_X_MSB
        lda hm5f
        sta $d05f
        lda hm77
        sta $d077
        rts

; cap_compose_h: Muenze in 640 x 400: hs_capbase (16 x 16, 128 Byte) nach hs_cap kopieren und den Buchstaben (8 x 8 aus dem Zeichensatz, sp zeigt darauf)
;   in die Mitte (Zeilen und Spalten 4..11, Farbwert 11 = dunkel) eintragen.
cap_compose_h:
        ldy #127
.ch1:   lda hs_capbase,y
        sta hs_cap,y
        dey
        bpl .ch1
        lda #0
        sta t3                          ; Zeile 0..7 des Buchstabens -> Zeile 4 + Zeile, Spalte 4 + X (Byte 2 + X / 2 der Zeile)
.ch2:   ldy t3
        lda (sp),y
        sta t0
        lda t3
        clc
        adc #4
        asl
        asl
        asl
        sta t1                          ; Byteversatz der Zeile (8 Byte je Zeile)
        ldx #0
.ch3:   asl t0
        bcc .ch6
        txa
        lsr
        clc
        adc t1
        tay                             ; Y = Byteversatz der Zeile + X / 2; das Zielbyte ist Y + 2
        txa
        and #1
        bne .ch5                        ; ungerade X: unteres Halbbyte
        lda hs_cap + 2,y
        and #$0f
        ora #$b0
        sta hs_cap + 2,y
        jmp .ch6
.ch5:   lda hs_cap + 2,y
        and #$f0
        ora #$0b
        sta hs_cap + 2,y
.ch6:   inx
        cpx #8
        bne .ch3
        inc t3
        lda t3
        cmp #8
        bne .ch2
        lda #0
        sta capfr
        lda #1
        sta capact
        rts

shot_reg: .byte 12, 14                          ; Registerversatz Sprite 6, 7 (2 * Nummer)
shot_bit: .byte 64, 128

ball_reg: .byte 6, 8, 10                      ; Registerversatz der Ballsprites (Sprite 3, 4, 5: $D000 + 2 * Nummer)
ball_bit: .byte 8, 16, 32                     ; Bitmaske der Ballsprites
bpal:     .byte 0                             ; aktuelle Kugelpalette: 0 = blau, 1 = gold (Durchschlag)
bheat:    .byte 0                             ; Tempo-Gruppe der blauen Kugel: 0 = blau, 1..3 = zunehmend rot (lib/sprites16.inc: heat_r/g/b)

; set_ball_pal: Palette der Sprites 3, 4, 5 (Eintraege 48-95, Bank 1) auf die blaue (X = 0) oder goldene (X = 1) Kugel setzen.  Bank 1 wird kurz in
; $D100-$D3FF abgebildet; solange sperren wir den Interrupt, denn der schreibt Eintrag 0 der Bank 0 (Hintergrundverlauf) an dieselben Adressen.
set_ball_pal:
        php
        sei
        lda #$44
        sta $d070
        lda bheat                       ; Ansatz in den Tempo-Tabellen: (Gruppe - 1) * 16
        sec
        sbc #1
        asl a
        asl a
        asl a
        asl a
        sta t2
        ldy #15
.sbp:   cpx #0
        bne .sbg
        lda bheat
        beq .sbn0
        tya                             ; Tempo-Glut: ueberblendete Eintraege
        clc
        adc t2
        tax
        lda heat_r,x
        sta $d130,y
        sta $d140,y
        sta $d150,y
        lda heat_g,x
        sta $d230,y
        sta $d240,y
        sta $d250,y
        lda heat_b,x
        sta $d330,y
        sta $d340,y
        sta $d350,y
        ldx #0
        jmp .sbn
.sbn0:  lda spal_r,y
        sta $d130,y
        sta $d140,y
        sta $d150,y
        lda spal_g,y
        sta $d230,y
        sta $d240,y
        sta $d250,y
        lda spal_b,y
        sta $d330,y
        sta $d340,y
        sta $d350,y
        jmp .sbn
.sbg:   lda spalg_r,y
        sta $d130,y
        sta $d140,y
        sta $d150,y
        lda spalg_g,y
        sta $d230,y
        sta $d240,y
        sta $d250,y
        lda spalg_b,y
        sta $d330,y
        sta $d340,y
        sta $d350,y
.sbn:   dey
        bpl .sbp
        lda #$04
        sta $d070
        plp
        rts

; tick_timers: einmal je Bild.  Alle 25 Bilder (0,5 s) zaehlen die Extra-Timer um 1 herunter; bei 0 endet das Extra.
tick_timers:
        dec tick
        bne .tt
        lda #25
        sta tick
        lda lkind
        beq .tk0
        cmp #1
        bne .tkb
        lda bsub                        ; Bonus: alle zwei Halbsekunden eine Sekunde herunter
        eor #1
        sta bsub
        bne .tk1
        lda btime
        beq .tk1
        dec btime
.tk1:   jsr draw_status
        jmp .tk0
.tkb:   dec batk                        ; Boss: alle 4 s ein Angriff (eine boese Kapsel)
        bne .tk2
        lda #8
        sta batk
        jsr boss_attack
.tk2:   jsr draw_status
.tk0:   lda tpad
        beq .t1
        dec tpad
        bne .t1
        lda #32                         ; Breite zurueck auf normal
        jsr set_padw
.t1:    lda trev
        beq .t2
        dec trev
.t2:    lda tlas
        beq .t3
        dec tlas
.t3:    lda tpierce                     ; Durchschlag zaehlt herunter
        beq .t4
        dec tpierce
.t4:    lda tspd
        beq .tt
        dec tspd
        bne .tt
        lda #0
        sta spdmod                      ; Geschwindigkeit wieder normal (wirkt ab dem naechsten Schlaegertreffer)
.tt:    rts

; spawn_cap: laesst an der Position des zuletzt zerstoerten Steins (bcol/brow) eine Kapsel fallen (hoechstens eine zugleich).
;   Die Kapselart wird zufaellig gewaehlt; ihr Buchstabe wird aus dem Zeichensatz ins Sprite $E6 kopiert (Zeile i -> Byte 3 * (2 + i) + 1).
spawn_cap:
.ifdef TESTMULTI
        jsr multiball                   ; Testfassung: sofort Mehrball (prueft Bewegung/Anzeige mehrerer Baelle)
.endif
        lda capact
        bne .sc9
        lda bcol
        asl
        asl
        asl
        asl                             ; Spalte * 16 (Low-Byte; Spalte 16 und 17 laufen ueber 255)
        sta capxl
        lda bcol
        lsr
        lsr
        lsr
        lsr                             ; Spalte / 16 = Uebertrag in das High-Byte
        sta capxh
        clc
        lda capxl
        adc #16                         ; + 16 px (linker Rand des Steinfelds)
        sta capxl
        bcc +
        inc capxh
+       lda brow
        asl
        asl
        asl                             ; Zeile * 8
        clc
        adc #32                         ; + 32 px (Oberkante des Steinfelds)
        sta capy
        jsr rand_byte
        and #15
        tax
        lda cap_pick,x
        sta captype
        lda capforce
        beq .nf
        sec
        sbc #1
        sta captype                     ; erzwungene Art (Bossangriff)
        lda #0
        sta capforce
.nf:
.ifdef TESTMULTI
        lda #8                          ; Testfassung: immer Mehrball
        sta captype
.endif
        ldx captype                     ; (A ist hier 0 bzw. capforce, nicht die Art: daher captype laden)
        jsr set_cap_pal                 ; Palette der Muenze (Sprite 0, Bank 1) auf die Farbe dieser Art setzen (X = Art, bleibt erhalten)
        lda cap_letter,x                ; Bildschirmcode des Buchstabens
        sta t0
        lda #0
        sta t1
        asl t0                          ; Code * 8 = Adresse im Zeichensatz ($8800 + Code * 8)
        rol t1
        asl t0
        rol t1
        asl t0
        rol t1
        lda t1
        clc
        adc #$88
        sta sp + 1
        lda t0
        sta sp
        lda hires
        beq .sc0
        jmp cap_compose_h               ; 640 x 400: 16 x 16-Muenze (der Zeiger sp zeigt auf den Buchstaben im Zeichensatz)
.sc0:   ldy #95                         ; Muenze ohne Buchstaben (cap_base, 12 Zeilen zu 8 Byte) nach gfx_cap16 kopieren
.cb:    lda cap_base,y
        sta gfx_cap16,y
        dey
        bpl .cb
        lda #0
        sta t3                          ; Zeile 0..7 des Buchstabens -> Sprite-Zeile 2 + Zeile, Spalten 2..9 (Farbwert 11 = dunkel)
.cr:    ldy t3
        lda (sp),y                      ; Zeichenzeile: 8 Bildpunkte
        sta t0
        lda t3
        clc
        adc #2
        asl
        asl
        asl
        sta t1                          ; Byteversatz der Zeile (8 Byte je Zeile)
        ldx #0                          ; Spalte 0..7 des Buchstabens = Sprite-Spalte 2 + X
.cc:    asl t0                          ; naechstes Bit (Bit 7 zuerst) in den Uebertrag
        bcc .cn
        txa
        lsr                             ; (2 + X) / 2 = 1 + X / 2: Byte in der Zeile
        clc
        adc t1
        tay                             ; Y + 1 = Byteversatz
        txa
        and #1
        bne .lo                         ; X ungerade: 2 + X ungerade = unteres Halbbyte (zweites Pixel des Bytes)
        lda gfx_cap16 + 1,y
        and #$0f
        ora #$b0                        ; Pixel links (hohes Halbbyte) = 11
        sta gfx_cap16 + 1,y
        jmp .cn
.lo:    lda gfx_cap16 + 1,y
        and #$f0
        ora #$0b                        ; Pixel rechts (unteres Halbbyte) = 11
        sta gfx_cap16 + 1,y
.cn:    inx
        cpx #8
        bne .cc
        inc t3
        lda t3
        cmp #8
        bne .cr
        lda #0
        sta capfr
        lda #1
        sta capact
.sc9:   rts

; Kapselarten: 0 B breit, 1 S schmal, 2 R umgekehrt, 3 L langsam, 4 F schnell, 5 K klebrig, 6 + Leben, 7 P Punkte
cap_frames: .byte $d4, $d7, $da, $dd, $da, $d7           ; Sprite-Bloecke der Drehphasen (vorn, 10 Zeilen, 7 Zeilen, Kante, 7, 10)
cap_letter: .byte 2, 19, 18, 12, 6, 11, $2b, 16, 13, 7, 4        ; Bildschirmcodes: B=2, S=19, R=18, L=12, F=6, K=11, + = $2B, P=16, M=13
; Farbe der Muenze je Art (Sprite-Farbe): B hellblau, S hellrot, R rot, L gruen, F gelb, K orange, + weiss, P tuerkis, M violett, G hellgruen,
; D (Durchschlag) grau
cap_color:  .byte 14, 10, 2, 5, 7, 8, 1, 3, 4, 13, 12
; Gewichtung der Kapselarten bei der Auswahl: 16 Eintraege (Zufallszahl and 15); M kommt oefter als Leben (+)
cap_pick:   .byte 0, 0, 1, 2, 3, 4, 5, 6, 7, 7, 8, 8, 8, 9, 9, 10

; set_cap_pal: Palette der Muenze (Sprite 0 = Paletteneintraege 0-15 der Sprite-Bank 1) auf die Farbe der Art X setzen (aus cap_pr/pg/pb in
; lib/sprites16.inc, je Art 16 Eintraege).  Bank 1 wird kurz in $D100-$D3FF abgebildet, deshalb Interrupt gesperrt (siehe set_ball_pal).  Erhaelt X.
set_cap_pal:
        php
        sei
        stx t2                          ; Art merken
        txa
        asl
        asl
        asl
        asl
        tay                             ; Y = Art * 16: Quellindex
        lda #$44
        sta $d070
        ldx #0
.scp:   lda cap_pr,y
        sta $d100,x
        lda cap_pg,y
        sta $d200,x
        lda cap_pb,y
        sta $d300,x
        iny
        inx
        cpx #16
        bne .scp
        lda #$04
        sta $d070
        ldx t2
        plp
        rts

; laser_fire: zwei Schuesse an den Enden des Schlaegers starten (nur wenn beide Schlitze frei sind).
;   Schussposition in Einheiten zu 2 px: u = (px) / 2 (16-Bit-Wert padh:padl nach rechts schieben).
laser_fire:
        lda shact
        ora shact + 1
        bne .lf9                        ; es fliegt noch ein Schuss
        lda padl                        ; linker Schuss: Schlaegerkante + 2 px
        clc
        adc #2
        sta t0
        lda padh
        adc #0
        lsr
        ror t0
        lda t0
        sta shu
        lda padl                        ; rechter Schuss: Schlaegerkante + Breite - 4 px
        clc
        adc padw
        sta t0
        lda padh
        adc #0
        sta t1
        sec
        lda t0
        sbc #4
        sta t0
        lda t1
        sbc #0
        lsr
        ror t0
        lda t0
        sta shu + 1
        lda #PADY - 8
        sta shy
        sta shy + 1
        lda #1
        sta shact
        sta shact + 1
        lda padh                        ; Stereo: der Schuss klingt dort, wo der Schlaeger steht (links 16 .. rechts 272 px)
        lsr a
        lda padl
        ror a
        sta sfx_pan                     ; px / 2
        lsr a
        clc
        adc sfx_pan                     ; 3/4 px
        adc #20
        sta sfx_pan
        lda #SFX_LASER
        jsr sfx_play
.lf9:   rts
pan_col: .byte 10, 24, 38, 52, 66, 80, 94, 108, 122, 136, 150, 164, 178, 192, 206, 220, 234, 248   ; Stereo-Lage je Steinspalte

; update_shots: Schuesse um 4 px nach oben; trifft einer einen Stein (Karte an Schussspitze), wirkt hit_brick und der Schuss endet.
;   Spalte = (u - 8) / 8 (u = px / 2, Stein 16 px = 8 Einheiten breit, Feld beginnt bei px 16 = u 8), Zeile = (y - 32) / 8.
update_shots:
        ldx #0
.us:    stx shi                          ; Schuss 0..1 (shi bleibt ueber hit_brick erhalten? nein: nach dem Aufruf neu laden)
        lda shact,x
        beq .usn
        lda shy,x
        sec
        sbc #4
        sta shy,x
        cmp #20
        bcs .us1                        ; noch im Spielfeld
        lda #0                          ; ueber der Decke: Schuss weg
        sta shact,x
        beq .usn
.us1:   jsr wshot                      ; wandernder Stein getroffen?
        bcc .us2
        pha
        ldx shi
        lda #0
        sta shact,x
        pla
        jsr hit_brick
        jmp .usn
.us2:   ldx shi
        lda shy,x
        cmp #32
        bcc .usn                        ; ueber dem Steinfeld
        sec
        sbc #32
        lsr
        lsr
        lsr
        cmp #12
        bcs .usn                        ; unter dem Steinfeld (kommt nicht vor, Schuesse fliegen nur aufwaerts)
        sta brow
        lda shu,x
        sec
        sbc #8
        bcc .usn                        ; links vom Steinfeld
        lsr
        lsr
        lsr
        cmp #18
        bcs .usn                        ; rechts davon
        sta bcol
        ldy brow
        lda rowmul,y
        clc
        adc bcol
        sta bidx
        tay
        lda MAP,y
        beq .usn                        ; kein Stein
        ldx shi
        pha
        lda #0
        sta shact,x                     ; Schuss endet
        pla
        jsr hit_brick
.usn:   ldx shi
        inx
        cpx #2
        bne .us
        rts

; update_cap: Kapsel um 1 px fallen lassen; faengt der Schlaeger sie (Ueberlappung in X und Y), wirkt sie.
update_cap:
        lda capact
        beq uc_end
        inc capfr                       ; Drehung: 24 Bilder = ein voller Umlauf
        lda capfr
        cmp #24
        bcc +
        lda #0
        sta capfr
+       inc capy
        lda capy
        cmp #200
        bcc +
        lda #0                          ; unten heraus: weg
        sta capact
        rts
+       cmp #PADY - 12                  ; Unterkante der Kapsel (12 px hoch) erreicht die Schlaegeroberkante?
        bcc uc_end
        cmp #PADY + 6
        bcs uc_end                        ; schon unterhalb des Schlaegers
        sec
        lda capxl
        sbc padl
        sta t0
        lda capxh
        sbc padh
        bmi .un                         ; d < 0: Kapsel liegt links der Schlaegerkante
        bne uc_end                        ; d >= 256
        lda padw
        sec
        sbc #2
        sta t1                          ; die Muenze ist 12 px breit, 2 px eingerueckt: sie beruehrt den Schlaeger, wenn d < Breite - 2
        lda t0
        cmp t1
        bcs uc_end                      ; d zu gross: rechts vorbei
        jmp activate
.un:    lda t0
        cmp #243                        ; d >= -13: noch mindestens 1 px ueberlappt
        bcc uc_end
        ; weiter mit activate

; activate: Kapsel wirkt.  Art in captype (siehe cap_letter).
activate:
        lda #0
        sta capact
        lda #SFX_EXTRA
        jsr sfx_play
        lda captype
        beq .a0
        cmp #1
        beq .a1
        cmp #2
        beq .a2
        cmp #3
        beq .a3
        cmp #4
        beq .a4
        cmp #5
        beq .a5
        cmp #6
        beq .a6
        cmp #8
        beq .a8
        cmp #9
        beq .a9
        cmp #10
        beq .a10
        sed                             ; 7 = P: 500 Punkte (Dezimalmodus, auf die mittleren Ziffern)
        clc
        lda sc1
        adc #5
        sta sc1
        lda sc2
        adc #0
        sta sc2
        cld
        jmp draw_score
.a0:    lda #30                         ; B: breit, 15 s
        sta tpad
        lda #48
        jmp set_padw
.a1:    lda #30                         ; S: schmal, 15 s
        sta tpad
        lda #16
        jmp set_padw
.a2:    lda #20                         ; R: umgekehrte Steuerung, 10 s
        sta trev
        rts
.a3:    lda #30                         ; L: Ball langsamer, 15 s
        sta tspd
        lda #2
        sta spdmod
        rts
.a4:    lda #30                         ; F: Ball schneller, 15 s
        sta tspd
        lda #1
        sta spdmod
        rts
.a5:    lda #1                          ; K: Ball klebt beim naechsten Treffer
        sta sticky
        rts
.a10: lda #10
        sta tpierce
        rts
.a9:    lda #40                         ; G: Laser fuer 20 s (Feuer schiesst, solange der Ball nicht am Schlaeger klebt)
        sta tlas
        rts
.a8:    jmp multiball                   ; M: aus dem Ball werden bis zu drei
.a6:    lda lives                       ; +: ein Ball mehr (hoechstens 9)
        cmp #9
        bcs uc_end
        inc lives
        jmp draw_hud
uc_end: rts

; ---------------------------------------------------------------------------------------------------------------------
; Spielende
; ---------------------------------------------------------------------------------------------------------------------
game_over:
        lda #0
        sta VIC_SPR_ENABLE              ; Ball und Schlaeger ausblenden
        jsr border_black                ; ein laufender Randblitz endet
        lda #0
        lda #1
        sta txtcol
        lda #<g_over
        sta tp
        lda #>g_over
        sta tp + 1
        ldx #14
        ldy #18
        jsr puts
        lda sc0                         ; Punkte (BCD, niedrig bis hoch) in die Puffer der Bestenlisten-Bibliothek
        sta hs_points
        lda sc1
        sta hs_points + 1
        lda sc2
        sta hs_points + 2
        lda level                       ; erreichtes Level (1..)
        clc
        adc #1
        sta hs_level
        lda demo
        bne .nohs                       ; Demo-Modus: keine Bestenliste
        jsr hs_check                    ; A = Platz 0..9 oder $FF (nicht in der Liste)
        cmp #$ff
        beq .nohs
        pha                             ; Platz merken
        lda #7                          ; gelb
        sta txtcol
        lda #<g_new
        sta tp
        lda #>g_new
        sta tp + 1
        ldx #13
        ldy #20
        jsr puts
        jsr name_entry                  ; Name eingeben (Vorgabe SPIELER, Auto-Bestaetigung nach 30 s)
        pla
        jsr hs_insert                   ; Eintrag an den Platz setzen, Rest rutscht nach unten
        sei                             ; die Diskettenroutinen sind zeitkritisch: ohne Rasterinterrupt arbeiten
        jsr hs_save                     ; auf Diskette schreiben; A = HS_OK oder Fehlercode
        cli
        ldx #1
        stx txtcol
        cmp #HS_OK
        bne .bad
        lda #<g_saved
        sta tp
        lda #>g_saved
        sta tp + 1
        bne .msg                        ; (immer: High-Byte der Adresse ist nicht 0)
.bad:   lda #<g_nosave                  ; Speichern ging schief (keine Diskette, schreibgeschuetzt, ...): nur melden
        sta tp
        lda #>g_nosave
        sta tp + 1
.msg:   ldx #11
        ldy #23
        jsr puts
.nohs:  ldx #150                        ; drei Sekunden anzeigen
        jmp wait_frames

g_over:  .screen "spiel zu ende"
         .byte 0
g_new:   .screen "neuer rekord!"
         .byte 0
g_saved: .screen "gespeichert"
         .byte 0
g_nosave: .screen "nicht gespeichert!"
         .byte 0
g_name:  .screen "name:"
         .byte 0
g_help:  .screen "tippen + return  oder  joystick"
         .byte 0

; ---------------------------------------------------------------------------------------------------------------------
; Namenseingabe: 8 Zeichen, Vorgabe "SPIELER".
;   Bedienung: hoch/runter (Joystick oder Q/A) wechselt das Zeichen an der Cursorstelle (Leerzeichen, A-Z, 0-9, im Kreis),
;   rechts/links (Joystick oder X/Z) bewegt den Cursor, Feuer bestaetigt.  Haelt man eine Richtung, wiederholt sie sich
;   (nach 25 Bildern, dann alle 6).  Ohne jede Eingabe wird nach 30 s (1500 Bilder bei 50 Hz) automatisch bestaetigt; jede
;   Eingabe stellt den Zaehler zurueck.  Der Name steht als ASCII in hs_name (8 Byte).
; ---------------------------------------------------------------------------------------------------------------------
name_entry:
        ldx #7
.nd:    lda hs_defname,x                ; Vorgabe "SPIELER " in den Namenspuffer
        sta hs_name,x
        dex
        bpl .nd
        lda #0
        sta npos
        sta nprev
        sta nrep
        sta ntyped
        lda #<1500
        sta ntl
        lda #>1500
        sta nth
        lda #1
        sta txtcol
        lda #<g_name
        sta tp
        lda #>g_name
        sta tp + 1
        ldx #10
        ldy #22
        jsr puts
        lda #<g_help
        sta tp
        lda #>g_help
        sta tp + 1
        ldx #5
        ldy #24
        jsr puts
        jsr input_wait_release          ; Feuer vom Spiel noch gedrueckt? -> loslassen lassen
        jsr name_draw
        lda #FLASHPAL                   ; Rekord: der Rand pulsiert gelb (Palettenanimation)
        sta VIC_BORDERCOLOR
.ne:    jsr wait_frame
        inc npul
        lda npul
        and #63                         ; Dreieckswelle 0..31..0, Helligkeit 100..224
        cmp #32
        bcc +
        eor #63
+       asl a
        asl a
        clc
        adc #100
        sta fu
        sta fu + 1
        lda #0
        sta fu + 2
        jsr flash_write
        jsr input_char                  ; Tastatur: getipptes Zeichen (A-Z, 0-9, Leerzeichen, -, .), RETURN ($0D), DEL ($14)
        sta nch
        jsr input_joy                   ; Joystick: nur Port 2 (Z/X/Q/A sind hier Buchstaben und keine Richtungen)
        sta mask
        ora nch
        bne .press
        lda ntl                         ; keine Eingabe: Zaehler herunterzaehlen (16 Bit)
        ora nth
        beq .done                       ; 0 erreicht: automatisch bestaetigen
        lda ntl
        bne +
        dec nth
+       dec ntl
        lda #0
        sta nprev                       ; Taste losgelassen: naechster Druck zaehlt wieder als neu
        jmp .ne
.press: lda #<1500                      ; irgendeine Eingabe: Auto-Bestaetigung neu starten
        sta ntl
        lda #>1500
        sta nth
        lda nch
        beq .joy                        ; kein Zeichen getippt: Joystick
        cmp #$0d
        beq .done                       ; RETURN = fertig
        cmp #$14
        beq .del                        ; DEL = zeichen loeschen
        pha                             ; ein Zeichen: bei der ERSTEN Eingabe die Vorgabe "SPIELER" ganz loeschen
        lda ntyped
        bne .typ
        lda #1
        sta ntyped
        lda #0
        sta npos
        ldx #7
        lda #CH_SPACE
.tc:    sta hs_name,x                   ; (Leerzeichen als ASCII = $20 = auch Bildschirmcode 32)
        dex
        bpl .tc
.typ:   pla
        ldy npos
        sta hs_name,y                   ; Zeichen an die Cursorstelle
        cpy #7
        bcs .tdraw                      ; achte Stelle erreicht: bleibt dort (weitere Zeichen ersetzen sie)
        inc npos
.tdraw: jsr name_draw
        jmp .ne
.del:   lda #1                          ; DEL zaehlt auch als "es wird etwas Eigenes eingegeben": Vorgabe loeschen
        ldx ntyped
        bne .dl
        sta ntyped
        lda #0
        sta npos
        ldx #7
        lda #CH_SPACE
.dc:    sta hs_name,x
        dex
        bpl .dc
        jmp .tdraw
.dl:    ldy npos
        lda hs_name,y
        cmp #CH_SPACE
        beq .dprev
        lda #CH_SPACE                   ; Zeichen unter dem Cursor (achte Stelle) loeschen
        sta hs_name,y
        jmp .tdraw
.dprev: tya
        beq .tdraw                      ; schon ganz links
        dec npos
        ldy npos
        lda #CH_SPACE
        sta hs_name,y
        jmp .tdraw
.joy:   lda mask
        cmp nprev
        bne .new                        ; andere Taste als im vorigen Bild: neuer Druck
        dec nrep                        ; gleiche Taste gehalten: erst nach der Verzoegerung wiederholen
        bne .ne
        lda #6
        sta nrep
        jmp .do
.new:   sta nprev
        lda #25
        sta nrep
.do:    lda mask
        and #IN_FIRE
        bne .done                       ; Feuer = fertig
        lda mask
        and #IN_UP
        beq +
        jsr ch_up
+       lda mask
        and #IN_DOWN
        beq +
        jsr ch_down
+       lda mask
        and #IN_RIGHT
        beq +
        lda npos
        cmp #7
        bcs +                           ; ganz rechts: bleibt
        inc npos
+       lda mask
        and #IN_LEFT
        beq +
        lda npos
        beq +                           ; ganz links: bleibt
        dec npos
+       jsr name_draw
        jmp .ne
.done:  jsr border_black
        rts
npul:   .byte 0

; cur_idx: Index des Zeichens an der Cursorstelle in der Zeichenfolge ctab (nicht gefunden: 0).  Rueckgabe X, Y = npos.
cur_idx:
        ldy npos
        lda hs_name,y
        ldx #0
.ci:    cmp ctab,x
        beq .ck
        inx
        cpx #37
        bne .ci
        ldx #0
.ck:    rts
; ch_up / ch_down: naechstes / vorheriges Zeichen der Folge (im Kreis).
ch_up:  jsr cur_idx
        inx
        cpx #37
        bcc +
        ldx #0
+       lda ctab,x
        sta hs_name,y
        rts
ch_down:
        jsr cur_idx
        dex
        bpl +
        ldx #36
+       lda ctab,x
        sta hs_name,y
        rts
ctab:   .text " ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"

; name_draw: den Namen in Zeile 22 ab Spalte 16 anzeigen; das Zeichen unter dem Cursor in Umkehrschrift (Bit 7 des Bildschirmcodes).
;   ASCII -> Bildschirmcode: Buchstaben (65..90) minus 64, Ziffern und Leerzeichen unveraendert.
name_draw:
        lda #1
        sta txtcol
        ldx #0
.nr:    stx t1
        lda hs_name,x
        cmp #64
        bcc +
        sec
        sbc #64
+       cpx npos
        bne +
        ora #$80                        ; Cursorstelle invers
+       pha
        txa
        clc
        adc #16
        tax
        pla
        ldy #22
        jsr put_cell
        ldx t1
        inx
        cpx #8
        bne .nr
        rts

; =====================================================================================================================
; Ticker: eine Textzeile (32 Zeichen breit) mit wechselnden Texten und Uebergangseffekten.
;   Der Titel zeigt darin den Untertitel (Zeile 10), die Demo die Kopfzeile (Zeile 0, statt des Punktestands).
;   TECHNIK: Die 32 Bildschirmzeichen der Zeile (Spalte 4..35) zeigen die Zeichen 224..255 des Zeichensatzes; deren Bitmuster (256 Byte bei
;   $4F00, je Zeichen 8 Byte, ein Byte je Pixelzeile) werden in jedem Schritt neu berechnet.  So sind Bewegungen um einzelne BILDPUNKTE moeglich,
;   obwohl nur Zeichen im Raster sichtbar sind.  Die Bitmuster der Buchstaben holen wir aus dem ins RAM kopierten ROM-Zeichensatz ($4800 + Code * 8).
;   EFFEKTE (zufaellig gewaehlt): 0 nach oben wegscrollen (neu von unten), 1 nach unten, 2 buchstabenweise (erst rechts nach links loeschen,
;   dann links nach rechts schreiben), 3 nach links wegscrollen (neu von rechts), 4 nach rechts, 5 Wischer (Kante laeuft pixelweise von links
;   nach rechts), 6 Aufloesen (zufaellige Pixelzeilen werden ausgetauscht).  Dazu eine laufende Regenbogenfarbe ueber die Zeichen.
;   ABLAUF: warten (tkw Bilder), naechsten Text aus der Folge holen, Effekt starten, Schritte ausfuehren, fertig -> warten.
; Zustand (Nullseite): tkrow Zeile, tkst 0 = warten / 1 = Effekt, tkw Wartebilder, tkeff Effekt, tkstep Schritt, tkmsg Folgenindex, tkseq 0 Titel / 1 Demo
; =====================================================================================================================
.const TK_BUF = $8f00                   ; Bitmuster der Zeichen 224..255 ($4800 + 224 * 8)
.const TK_CHR = 224                     ; erstes Ticker-Zeichen
.const tk_cur = $c140                   ; aktuell angezeigter Text (32 Bildschirmcodes)
.const tk_new = $c160                   ; Zieltext

; ticker_init: A = Bildschirmzeile, X = Folge (0 Titel: Texte 0, 1;  1 Demo: Texte 2, 0, 1).  Zeigt den ersten Text sofort.
ticker_init:
        sta tkrow
        stx tkseq
        lda #0
        sta tkmsg
        sta tkst
        sta tkph
        lda #200
        sta tkw                         ; erster Wechsel nach 4 s
        lda #$41                        ; Zufallsstartwert fuer das Aufloesen (nicht 0)
        sta tklf
        ldx tkseq
        lda tk_seq,x                    ; erster Text der Folge
        jsr tk_loadnew
        ldx #31
.ti1:   lda tk_new,x                    ; Zieltext wird der angezeigte Text
        sta tk_cur,x
        dex
        bpl .ti1
        jsr tk_renderall
        ldx #4
        ldy tkrow
        jsr set_rowptr                  ; sp/cp auf Spalte 4 der Zeile
        ldx #0
        ldz #0
.ti2:   txa
        clc
        adc #TK_CHR
        sta [sq],z                      ; Zeichen 224 + Spalte (hohe Aufloesung: 8 x 8-Zeichen, eine Zeichenzelle je Zeichen)
        inz
        lda #7
        sta [cq],z
        inz
        inx
        cpx #32
        bne .ti2
        rts

; tk_loadnew: A = Textnummer (0..2): 32 Zeichen aus tk_msgs nach tk_new kopieren.
tk_loadnew:
        asl
        asl
        asl
        asl
        asl                             ; * 32
        tax
        ldy #0
.tl:    lda tk_msgs,x
        sta tk_new,y
        inx
        iny
        cpy #32
        bne .tl
        rts

; tk_ga / tk_gb: A = Bildschirmcode -> tka / tkb = Adresse des Bitmusters im Zeichensatz ($4800 + Code * 8).
tk_ga:  sta tka
        lda #0
        sta tka + 1
        asl tka
        rol tka + 1
        asl tka
        rol tka + 1
        asl tka
        rol tka + 1
        lda tka + 1
        clc
        adc #$88
        sta tka + 1
        rts
tk_gb:  sta tkb
        lda #0
        sta tkb + 1
        asl tkb
        rol tkb + 1
        asl tkb
        rol tkb + 1
        asl tkb
        rol tkb + 1
        lda tkb + 1
        clc
        adc #$88
        sta tkb + 1
        rts

; tk_renderchar: Zeichen tkc (0..31) des angezeigten Textes tk_cur in den Zeichensatz kopieren (8 Pixelzeilen).
tk_renderchar:
        ldx tkc
        lda tk_cur,x
        jsr tk_ga
        lda tkc
        asl
        asl
        asl
        tax                             ; X = tkc * 8 = Anfang im Puffer
        ldy #0
.rc:    lda (tka),y
        sta TK_BUF,x
        inx
        iny
        cpy #8
        bne .rc
        rts

; tk_renderall: alle 32 Zeichen neu aus tk_cur zeichnen.
tk_renderall:
        lda #0
        sta tkc
.ra:    jsr tk_renderchar
        inc tkc
        lda tkc
        cmp #32
        bne .ra
        rts

; tk_sync: der Zieltext wird der angezeigte Text (Effekt zu Ende), sauber neu gezeichnet.
tk_sync:
        ldx #31
.sy:    lda tk_new,x
        sta tk_cur,x
        dex
        bpl .sy
        jmp tk_renderall

; ---- Effekt 0/1: senkrecht pixelweise.  tkd = Versatz d (+1..+8 nach oben, -1..-8 nach unten): Pixelzeile r zeigt Zeile r + d des alten Textes,
;   liegt r + d ausserhalb 0..7, die Zeile (r + d) mod 8 des neuen Textes.
tk_vert:                                ; verarbeitet die Zeichen tkc .. tkce - 1 (ein Viertel je Bild, damit ein Bild nicht zu lang dauert)
.vc:    ldx tkc
        lda tk_cur,x
        jsr tk_ga
        ldx tkc
        lda tk_new,x
        jsr tk_gb
        lda tkc
        asl
        asl
        asl
        sta tkt                         ; Zielindex im Puffer
        lda #0
        sta tkr
.vr:    lda tkr
        clc
        adc tkd
        sta tks                         ; s = r + d
        and #$f8
        bne .vn                         ; ausserhalb 0..7: neuer Text
        ldy tks
        lda (tka),y
        jmp .vs
.vn:    lda tks
        and #7
        tay
        lda (tkb),y
.vs:    ldy tkt
        sta TK_BUF,y
        inc tkt
        inc tkr
        lda tkr
        cmp #8
        bne .vr
        inc tkc
        lda tkc
        cmp tkce
        bne .vc
        rts

; ---- Effekt 2: buchstabenweise.  Schritt 0..31: Zeichen 31 - n loeschen (Leerzeichen), Schritt 32..63: Zeichen n - 32 aus dem neuen Text setzen.
tk_type:
        lda tkstep
        cmp #32
        bcs .ty2
        eor #$ff                        ; 31 - n  (n < 32)
        clc
        adc #32
        sta tkc
        tax
        lda #32
        sta tk_cur,x
        jmp tk_renderchar
.ty2:   sec
        sbc #32
        sta tkc
        tax
        lda tk_new,x
        sta tk_cur,x
        jmp tk_renderchar

; ---- Effekt 3: nach links wegscrollen, neuer Text kommt von rechts (je Schritt 4 Pixel, 64 Schritte).
;   Neue Pixel rechts: Spaltengruppe j = Schritt des NEUEN Textes (Zeichen j / 2, rechte oder linke Haelfte).
tk_hleft:
        lda tkstep
        lsr
        tax
        stx tkt                         ; Zeichen des neuen Textes
        lda tk_new,x
        jsr tk_gb
        lda tkrs
        sta tkr                         ; nur die Pixelzeilen tkrs .. tkre - 1 (halbe Zeichenhoehe je Bild)
.hl1:   ldy tkr
        lda (tkb),y                     ; Pixelzeile des Zeichens
        ldx tkstep
        txa
        and #1
        bne .hl2
        lda (tkb),y                     ; gerader Schritt: linke Haelfte (obere 4 Bit) nach unten
        lsr
        lsr
        lsr
        lsr
        jmp .hl3
.hl2:   lda (tkb),y
        and #$0f                        ; ungerader Schritt: rechte Haelfte
.hl3:   sta tks                         ; neue 4 Pixel
        ldx #0                          ; Zeichen 0..30
        lda tkr
        tay                             ; Y = Index (Zeichen * 8 + Zeile), laeuft in Schritten von 8
.hl4:   lda TK_BUF,y
        asl
        asl
        asl
        asl
        sta tkt
        lda TK_BUF + 8,y
        lsr
        lsr
        lsr
        lsr
        ora tkt
        sta TK_BUF,y
        tya
        clc
        adc #8
        tay
        inx
        cpx #31
        bne .hl4
        lda TK_BUF,y                    ; Zeichen 31: von rechts neue Pixel
        asl
        asl
        asl
        asl
        ora tks
        sta TK_BUF,y
        inc tkr
        lda tkr
        cmp tkre
        bne .hl1
        rts

; ---- Effekt 4: nach rechts wegscrollen, neuer Text kommt von links (die rechten Spaltengruppen des neuen Textes zuerst).
tk_hright:
        lda #63
        sec
        sbc tkstep
        sta tkt                         ; j = 63 - Schritt
        lsr
        tax
        lda tk_new,x
        jsr tk_gb
        lda tkrs
        sta tkr
.hr1:   ldy tkr
        lda tkt
        and #1
        bne .hr2
        lda (tkb),y                     ; gerades j: obere Haelfte (linke 4 Pixel des Zeichens)
        lsr
        lsr
        lsr
        lsr
        jmp .hr3
.hr2:   lda (tkb),y
        and #$0f
.hr3:   asl
        asl
        asl
        asl
        sta tks                         ; neue Pixel stehen in den oberen 4 Bit (ganz links)
        ldx #31                         ; Zeichen 31 .. 1: b[c] = (b[c] >> 4) | (b[c-1] << 4)
        lda tkr
        clc
        adc #248
        tay                             ; Y = 31 * 8 + Zeile
.hr4:   lda TK_BUF,y
        lsr
        lsr
        lsr
        lsr
        sta tkd2
        lda TK_BUF - 8,y
        asl
        asl
        asl
        asl
        ora tkd2
        sta TK_BUF,y
        tya
        sec
        sbc #8
        tay
        dex
        bne .hr4
        lda TK_BUF,y                    ; Zeichen 0: b[0] = (b[0] >> 4) | neue Pixel links
        lsr
        lsr
        lsr
        lsr
        ora tks
        sta TK_BUF,y
        inc tkr
        lda tkr
        cmp tkre
        bne .hr1
        rts

; ---- Effekt 5: Wischer.  Schritt n (0..63): die Kante liegt bei Pixel 4 * (n + 1); komplette Zeichen davor sind neu, das Zeichen an der Kante
;   ist bei ungerader Kante nur zur Haelfte neu (obere 4 Bit von links).
tk_wipe:
        lda tkstep
        clc
        adc #1                          ; k = n + 1
        lsr                             ; k / 2 = Zeichenindex der Kante, Uebertrag = Haelfte
        bcc .wf
        sta tkc                         ; ungerade: Zeichen k/2 halb neu, halb alt
        tax
        lda tk_cur,x
        jsr tk_ga
        ldx tkc
        lda tk_new,x
        jsr tk_gb
        lda tkc
        asl
        asl
        asl
        tax
        ldy #0
.wh:    lda (tkb),y
        and #$f0
        sta tks
        lda (tka),y
        and #$0f
        ora tks
        sta TK_BUF,x
        inx
        iny
        cpy #8
        bne .wh
        rts
.wf:    sec                             ; gerade: Zeichen k/2 - 1 ist jetzt ganz neu
        sbc #1
        sta tkc
        tax
        lda tk_new,x
        sta tk_cur,x
        jmp tk_renderchar

; ---- Effekt 6: Aufloesen.  Je Schritt 8 Pixelzeilen in der Reihenfolge eines 8-Bit-Schieberegisters (alle 255 Werte ausser 0) austauschen.
tk_dissolve:
        lda #8
        sta tkt
.dd:    lda tklf                        ; Galois-LFSR, Polynom $B8
        lsr
        bcc .d1
        eor #$b8
.d1:    sta tklf
        sta tks                         ; Index 1..255 = Zeichen * 8 + Zeile
        lsr
        lsr
        lsr
        tax                             ; Zeichen
        lda tk_new,x
        jsr tk_gb
        lda tks
        and #7
        tay
        lda (tkb),y
        ldy tks
        sta TK_BUF,y
        dec tkt
        bne .dd
        rts

; tk_half: A = 0 / 1 -> tkrs/tkre = Pixelzeilen 0-3 bzw. 4-7.
tk_half:
        asl
        asl
        sta tkrs
        clc
        adc #4
        sta tkre
        rts

; ---- ticker_tick: einmal je Bild.
ticker_tick:
        inc tkph
        lda tkph
        and #1
        bne .tkc9
        lda tkph                        ; Regenbogenfarbe ueber die 32 Zeichen, wandert alle 2 Bilder weiter (Farben < 8: normale Zeichen)
        lsr
        sta tkt
        ldx #4
        ldy tkrow
        jsr set_rowptr
        ldy #0
        ldz #1
.tkc1:  tya
        clc
        adc tkt
        and #7
        tax
        lda tk_pal,x
        sta [cq],z
        inz
        inz
        iny
        cpy #32
        bne .tkc1
.tkc9:  lda tkst
        bne .tkrun
        dec tkw
        bne .tkret
        lda #1                          ; Wartezeit um: naechsten Text und einen Effekt waehlen
        sta tkst
        lda #0
        sta tkstep
        sta tkdiv
        inc tkmsg
        ldx tkseq
        lda tk_seqlen,x
        cmp tkmsg
        bne .tkn1
        lda #0
        sta tkmsg
.tkn1:  lda tkseq
        asl
        asl                             ; Folge * 4 (jede Folge hat Platz fuer 4 Eintraege)
        clc
        adc tkmsg
        tax
        lda tk_seqs,x
        jsr tk_loadnew
        jsr rand_byte
        and #7
        cmp #7
        bcc .tkn2
        lda #0
.tkn2:  sta tkeff
        lda #8
        sta tkd                         ; senkrechte Effekte starten bei d = 0, wir zaehlen ab dem ersten Schritt
        lda #0
        sta tkd
.tkret: rts
.tkrun: lda tkeff                       ; die teuren Effekte werden auf mehrere Bilder verteilt (Rechenzeit je Bild: sonst ruckelt das Spiel)
        cmp #2
        bcs .tkfast
        ; ---- senkrecht (0, 1): ein Viertel der Zeichen je Bild, nach dem vierten Viertel zaehlt der Schritt weiter
        lda tkdiv
        asl
        asl
        asl
        sta tkc                         ; erstes Zeichen dieses Viertels (0, 8, 16, 24)
        clc
        adc #8
        sta tkce
        lda tkeff
        bne .vdown
        lda tkstep
        clc
        adc #1                          ; nach oben: d = Schritt + 1 (1..8)
        sta tkd
        jmp .vgo
.vdown: lda #$ff
        sec
        sbc tkstep                      ; nach unten: d = -(Schritt + 1)
        sta tkd
.vgo:   jsr tk_vert
        inc tkdiv
        lda tkdiv
        cmp #4
        bcc .tkret                      ; naechstes Viertel im naechsten Bild
        lda #0
        sta tkdiv
        inc tkstep
        lda tkstep
        cmp #8
        bcs .tkdone
        rts
.tkfast:
        lda tkeff
        cmp #2
        beq .e2
        cmp #3
        beq .e3
        cmp #4
        beq .e4
        cmp #5
        beq .e5
        jmp .e6
.e2:    jsr tk_type
        jmp .tkadv64
.e3:    lda tkdiv                       ; links: je Bild eine Haelfte der Pixelzeilen (0-3, 4-7), nach beiden Haelften ein Schritt
        jsr tk_half
        jsr tk_hleft
        jmp .hnext
.e4:    lda tkdiv
        jsr tk_half
        jsr tk_hright
.hnext: inc tkdiv
        lda tkdiv
        cmp #2
        bcc .tkret
        lda #0
        sta tkdiv
        jmp .tkadv64
.e5:    jsr tk_wipe
.tkadv64:
        inc tkstep
        lda tkstep
        cmp #64
        bcs .tkdone
        rts
.e6:    jsr tk_dissolve
        inc tkstep
        lda tkstep
        cmp #33                         ; 33 * 8 = 264 >= 255 Zeilen
        bcs .tkdone
        rts
.tkdone:
        jsr tk_sync                     ; Text endgueltig sauber setzen
        lda #0
        sta tkst
        lda #200                        ; 4 s Wartezeit bis zum naechsten Wechsel
        sta tkw
        rts

tk_pal: .byte 1, 7, 5, 3, 6, 4, 2, 7                     ; Regenbogen aus Farben < 8
tk_seqs: .byte 0, 1, 0, 0                                 ; Folge Titel (Texte 0, 1)
         .byte 2, 0, 1, 0                                 ; Folge Demo (Texte 2, 0, 1)
tk_seq:  .byte 0, 2                                       ; erster Text je Folge
tk_seqlen: .byte 2, 3                                     ; Laenge je Folge
; Texte, je 32 Zeichen, mittig aufgefuellt (Kleinbuchstaben = Bildschirmcodes 1..26, siehe Hinweis zu .screen)
tk_msgs:
        .screen "  pralle action fuer den mega65 "
        .screen " ein demospiel von vintagecoder "
        .screen "       *** prallball ***        "

; ---------------------------------------------------------------------------------------------------------------------
; Bibliotheken und Daten
; ---------------------------------------------------------------------------------------------------------------------
.include "lib/input.asm"
.include "lib/rand.asm"
.include "lib/sound.asm"
.include "lib/hiscore.asm"
.include "lib/dirtable.inc"             ; dir_xl, dir_xh, dir_yl, dir_yh (erzeugt von tools/gen.py)
.include "lib/gfx.inc.chars"            ; gfx_chars

.include "lib/sprites640.inc"             ; 16-Farb-Sprites fuer 640 x 400 ab $8000 (in der VIC-Bank 2 vor dem Zeichensatz)
.org $9000                              ; Sprite-Daten (Zeiger $40..$4A): Ball, Schlaegerteile, Kapsel, Schuss, Kapselphasen
.include "lib/gfx.inc.sprites"
.org $9400                              ; ($5200-$53FF: weitere Sprite-Bloecke bis Zeiger $4F)
.include "lib/levels.inc"               ; level_count, levelN, level_tab_lo/hi

.org $b000                              ; 16-Farb-Sprites (Bank 2: Zeiger $C0 ff.)
.include "lib/sprites16.inc"
.org $e000                              ; Titelmusik beider Spuren (song, song3), erzeugt von tools/compose.py ($C000-$CFFF gehoert der Steinkarte und den Spielvariablen)
.include "lib/music.inc"                ; song: Titelmusik (erzeugt von tools/compose.py)
.org $e900                              ; Vollfarb-Steine: Muster und Palette (erzeugt von tools/gen_bricks.py); $F700-$FFFF nicht benutzen (vom C65-System beschrieben)
.include "lib/bricks.inc"
.ifdef TESTHS
th_name: .byte $4d, $45, $47, $41, $36, $35, $20, $20      ; "MEGA65  "
.endif
.ifdef TESTHS
; hs_test (nur -D TESTHS): Bestenliste im Spielzustand pruefen (laden, eintragen, speichern, laden); Ergebnisse siehe unten.
hs_test:
        jsr hs_load                     ; Testfassung: Bestenliste im Spielzustand laden (KERNAL wird kurz eingeblendet), einen Eintrag eintragen und
        sta $7000                       ; speichern, wieder laden; Ergebnisse ab $7000 (0 = ok), erster Eintrag ab $7010; danach laeuft das Spiel
        lda hs_buf + 5 + 8
        sta hs_points
        lda hs_buf + 5 + 9
        sta hs_points + 1
        lda hs_buf + 5 + 10
        clc
        adc #1
        sta hs_points + 2
        lda #5
        sta hs_level
        ldx #7
.th1:   lda th_name,x
        sta hs_name,x
        dex
        bpl .th1
        jsr hs_check
        cmp #$ff
        beq .th3
        jsr hs_insert
.th3:   jsr hs_save
        sta $7001
        jsr hs_load
        sta $7002
        ldx #11
.th4:   lda hs_buf + 5,x
        sta $7010,x
        dex
        bpl .th4
        lda #$ee
        sta $70ff
        rts
.endif
