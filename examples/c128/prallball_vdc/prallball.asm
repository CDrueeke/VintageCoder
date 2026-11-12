; =====================================================================================================================
; prallball.asm - PRALLBALL fuer den Commodore 128, VDC-Fassung (80 Spalten): 24 Levels, Zeichen-Sprites, Joystick/Tastatur, Bestenliste
; =====================================================================================================================
;
; ZWECK
;   Ein Ball-und-Schlaeger-Spiel: Der Ball prallt von Waenden, Schlaeger und Steinen ab; sind alle zerstoerbaren Steine weg,
;   beginnt das naechste Level. Faellt der Ball unten heraus, geht ein Leben verloren; nach dem letzten Leben endet das Spiel.
;  Kein Code wird mit anderen Systemen geteilt.
;   Die Spiellogik entspricht der C64- und Plus/4-Fassung; neu ist die Anzeige: der VDC (8563/8568) kann nur Text, hat keine Sprites,
;   keinen Rasterinterrupt und keine Mehrfarbzeichen - dafuer 80 Spalten, zwei Zeichensaetze und eine Farbe je Zelle.
;
; BEDIENUNG
;   Joystick Port 2 oder Tastatur: Z = links, X = rechts, Leertaste = Feuer, P = Pause, RUN/STOP = Abbruch zum Titel.
;   Der Abprallwinkel haengt davon ab, wo der Ball den Schlaeger trifft (8 Zonen: -60 bis +60 Grad).
;   WICHTIG: Im 80-Spalten-Modus starten (Taste 40/80 DISPLAY gedrueckt, im Emulator x128 -80col), denn nur dann zeigt der Monitor den VDC.
;
; ABLAUF (Zustandsautomat im Hauptprogramm)
;   title      Titelbild, wartet auf Feuer          ->  new_game
;   new_game   Punkte 0, 3 Leben, Level 1           ->  load_level
;   load_level Steinkarte laden und zeichnen, Ball auf den Schlaeger  ->  play
;   play       ein Durchlauf je Bildwechsel (50 Hz PAL): Eingabe, Schlaeger, Ball, Kollisionen, Anzeige
;              Level geschafft -> naechstes Level;  Ball verloren -> Leben - 1;  Leben 0 -> game_over
;   game_over  Bestenliste pruefen, Namenseingabe, Speichern  ->  title
;   Ohne Eingabe beginnt der Titel nach 30 s die Demo (der Rechner spielt die Levels 1-5); jede Eingabe beendet sie.
;
; SPIELELEMENTE
;   Steintypen (Steinkarte MAP, 12 x 18, siehe brick_color/brick_pts): 1..8 Farbsteine, 9 Silber (2 Treffer), 10 Gold (3 Treffer), 11 unzerstoerbar,
;   12 "explosiv" (im Code wie ein normaler Stein behandelt), 13 Ueberraschung (laesst immer eine Kapsel fallen), 16..23 Boss-Kerne (mehrere Treffer).
;   Levelarten (level_kind in lib/levels.inc): Level 11 und 22 sind Bonuslevel (45 s, ein verlorener Ball kostet kein Leben, 100 Punkte je Stein
;   zusaetzlich), Level 23 und 24 Bosslevel (alle Kerne zerstoeren; der Boss wirft alle 4 s eine boese Kapsel S oder R).
;   Aus zerstoerten Steinen faellt mit etwa 1/12 Wahrscheinlichkeit eine Kapsel (Muenze mit Buchstaben): B breit, S schmal, R umgekehrte Steuerung,
;   L Ball langsamer, F schneller, K Ball klebt, + Extraball, P 500 Punkte, M Mehrball, G Laser, D Durchschlag.
;   Alle 8 Schlaegertreffer steigt die Ballgeschwindigkeit um eine Stufe (16 Stufen, Richtungstabelle lib/dirtable.inc).
;
; BAUEN
;   vcc build .   (Projekt prallball.vcprj: Ziel c128, CPU 8502, Diskettenabbild d64)   oder einzeln:
;   vcc asm prallball.asm --target c128 --cpu 8502 -o prallball.prg
;   Start im 80-Spalten-Modus, z. B. x128 -80col -autostart prallball.prg (Beschreibung und Testlauf: README.md).
;
; TESTFASSUNGEN (Schalter  -D NAME=1  beim Bauen; im normalen Spiel nicht gesetzt)
;   AUTOPLAY    der Rechner spielt von Anfang an und ohne Ende (Demo-Modus, Titel ohne Wartezeit, keine Soundeffekte)
;   STARTLEVEL=n  das Spiel beginnt mit Level n (0-basiert: 0 = Level 1)
;   TESTMULTI   jeder zerstoerte Stein ergibt sofort Mehrball (spawn_cap ruft multiball; Kapselart immer M)
;   TESTLASER   Laser bei jedem Zuruecksetzen der Extras (reset_extras) fuer 240 Halbsekunden = 2 Minuten aktiv
;   TESTLOSE    Eingabe ist nur Feuer, der Schlaeger bewegt sich nie (Ball geht verloren); der Titel wird ohne Wartezeit verlassen
;   TESTCAP     jeder zerstoerte Stein laesst eine Kapsel fallen
;   TESTFPS     zaehlt Bilder (Hauptschleife $C200/$C201, Rasterinterrupt $C202/$C203, verpasste Bildwechsel $C204) und zeigt sie in Zeile 24
;   NOTICKER    laesst den Ticker in der Demo (Zeile 0) weg
;   Die Emulator-Tests (prallball.vctest, erzeugt von tools/gen_tests.py) springen einzelne Routinen direkt an (Trap-Modus von vcc test).
;
; BILDSCHIRM (VDC)
;   Die Spiellogik denkt weiter in einem Raster aus 40 x 25 logischen Zeichen zu 8 x 8 Pixeln (320 x 200).  Jede logische Zelle besteht auf dem
;   VDC aus ZWEI Zellen: das Zeichen wird auf 16 x 8 Pixel gedehnt (jedes Pixel doppelt).  Die linken 4 Pixel (Bit 7..4 des Bitmusters, je doppelt)
;   liegen im ersten Zeichensatz (linke VDC-Zelle), die rechten 4 Pixel (Bit 3..0) im zweiten Zeichensatz (rechte VDC-Zelle, Attributbit 7).
;   Die Dehnungstabellen dbl_l und dbl_r (lib/vdcfont.inc) liefern fuer jedes Bitmuster-Byte das gedehnte Byte der linken bzw. rechten Haelfte.
;   Der Bildschirm lebt im VDC-Speicher (nicht im Hauptspeicher), siehe VDC-RAM-PLAN.  Im Hauptspeicher stehen Spiegel (SCR/COLORRAM: was der VDC
;   zeigt), Schattenkopien (SHC/SHF: Hintergrund ohne Sprites) und der Zeichensatz CHARSET (1 Bit/Pixel, 256 Zeichen zu 8 Byte).
;   Ball (bis 3), Schlaeger, Muenze und Laserschuesse sind ZEICHEN-SPRITES (Abschnitt "ZEICHEN-SPRITES"); jedes Pool-Zeichen wird nach dem
;   Zusammensetzen im Hauptspeicher gedehnt in den VDC geladen.  Statt der Rasterbalken bekommt der Hintergrund je Level eine dunkle Themenfarbe
;   (VDC-Register 26).  Das Lauftext-Feinscrollen entfaellt (der VDC kann nur den ganzen Bildschirm verschieben): der Text rutscht alle 6 Bilder
;   um ein Zeichen; der Ticker laeuft wie gehabt ueber Zeichen-Bitmuster.
;   Der VIC-II ist dunkel (Bild aus) und liefert nur den Bildtakt (Rasterinterrupt, Zeile 251); der Prozessor laeuft mit 2 MHz ($D030), fuer die
;   Bestenliste mit 1 MHz.
;   Farben: Im Programm stehen C64-Farbnummern (0..7; 8..15 = 8 + Farbe wie in der C64-Fassung); die Tabelle vcol setzt sie in die
;   RGBI-Attributfarben des VDC um.
;
; VDC-REGISTER, die dieses Programm benutzt (Zugriff: Registernummer nach $D600, dann Wert nach $D601; Bit 7 von $D600 lesen = bereit)
;   R10        Cursoranfang: $20 = Cursor aus
;   R12/R13    Anfang des Bildschirmspeichers ($0000)         R20/R21  Anfang der Attribute ($0800)
;   R18/R19    Adresse im VDC-RAM (hoch/niedrig), setzt vdc_addr; danach R31 lesen/schreiben, die Adresse zaehlt selbst um 1 weiter
;   R25        Bit 6 = Attribute an, Bit 7 = 0 (Textmodus), Bits 3..0 = 0 (kein waagerechter Versatz); Bits 5/4 bleiben wie vom Betriebssystem
;   R26        Bit 3..0 = Hintergrundfarbe (Themenfarbe vbg bzw. Blitzfarbe), Bit 7..4 = Vordergrund (immer $F; bei eingeschalteten Attributen
;              vermutlich ohne Wirkung)
;   R28        Bits 7..5 = Anfang des Zeichensatzes (001 = $2000), Bit 4 (RAM-Art 16 K/64 K) bleibt
;   R31        Datenbyte mit selbstzaehlender Adresse
;   Die Zeit- und Bildregister R0..R9 (80 x 25, Zeichenhoehe) hat das Betriebssystem beim Einschalten gesetzt; sie bleiben unberuehrt.
;   Attributbyte je Zelle: Bit 7 = zweiter Zeichensatz, Bits 3..0 = Vordergrundfarbe (Bit 6 invers, 5 unterstrichen, 4 blinkend: nicht benutzt).
;
; VDC-RAM-PLAN (nur 16 KB belegt; beschrieben wird alles mit vdc_addr/R31, zurueckgelesen nur in den Tests mit vdc_get)
;   $0000-$07CF  Zeichencodes, 25 Zeilen zu 80 Zellen (= 40 logische Zellen zu 2 VDC-Zellen)   VDC_SCR
;   $0800-$0FCF  Attribute (Farbe, Bit 7 = zweiter Zeichensatz), gleicher Aufbau              VDC_ATTR
;   $2000-$2FFF  Zeichensatz 1 (linke Haelften): 256 Zeichen zu 16 Byte (Zeilen 8..15 sind 0)  VDC_CHR
;   $3000-$3FFF  Zeichensatz 2 (rechte Haelften), gleicher Aufbau                              VDC_ALT
;   Logische Zelle (Zeile z, Spalte s) = VDC-Zellen z * 80 + 2 * s und z * 80 + 2 * s + 1.
;
; SPEICHERPLAN Hauptspeicher (MMU-Konfiguration $3E: alles RAM der Bank 0 plus E/A; nach dem Start sind die Systemroms aus,
;   nur die Bestenliste schaltet sie kurz ein)
;   $1C01-ca. $8D80  BASIC-Starter, Bestenliste (unter $4000!), Programmcode, Tabellen, Zeichen-Sprite-Streifen, Levels, Lied
;                    (Stand ohne -D; weit unter $AC00)
;   $AC00 Attributspiegel (1000 Byte)   $B000 Zeichenspiegel (1000 Byte)   $B400 Schattencodes   $B800 Schattenfarben
;   $C000 Steinkarte (216 Byte)   $C100 Ballspeicher (3 Baelle)   $C120 Laserschuesse   $C130 Titelbaelle   $C140/$C160 Ticker-Texte
;   $C200 Testzaehler (TESTFPS)   $CC00 Sprite-Schlitze, Poolliste ($CC40/$CC80), Liste des vorigen Bildes ($CD00/$CD40)
;   $E000-$E7FF Zeichensatz CHARSET (aus dem Zeichen-ROM kopiert; 64-67, 127 eigene Zeichen, 128..191 Pool der Zeichen-Sprites, 224..255 Ticker)
;   Das Zeichen-ROM wird einmal in copy_font gelesen; $E000 liegt im RAM unter dem KERNAL-ROM (Schreiben geht immer ins RAM).
;
; NULLSEITE (Zeropage)
;   $02-$03 Zufall (rand.asm)   $04-$07 Musik (sound.asm)   $08-$0D Bestenliste (hiscore.asm)
;   $10-$8B Spielzustand (siehe .const unten; $3C..$3F frei)   $8C-$8F Eingabe (input.asm)   $90-$B3 Zeichen-Sprites, VDC-Hilfen, Ticker-Upload
;   (der KERNAL nutzt $90 und aufwaerts nur waehrend der Bestenliste)
;
; KOORDINATEN UND EINHEITEN (wichtig fuer das Verstaendnis der Physik!)
;   Bildpunkte (px) zaehlen ab der linken oberen Ecke des logischen Bildschirms (Zeichen 0/0 = px 0/0).  Ein logisches Zeichen ist 8 x 8 px.
;   Spielfeld: x = 16..303 (Spalten 2..37), y = 16.. (Zeile 2 abwaerts); Waende in Spalte 1 und 38 sowie Zeile 1.
;   Steinfeld: 18 Steine je 16 px Breite (2 Zeichen) ab x = 16, 12 Zeilen je 8 px Hoehe ab y = 32 (Bildschirmzeile 4).
;   Ball und Schlaeger-Positionen: BALL in "Festkomma 1/64": Wert = Bildpunkte * 64 (16 Bit).  So reichen 16 Bit fuer x bis 1023 px
;   und die Geschwindigkeit kann in 1/64 px je Bild feiner als 1 px sein.  Die obere Haelfte (High-Byte) = Bildpunkte / 4.
;   Zeichen-Sprites werden auf 1 logisches Pixel genau gesetzt (= 2 Pixel des VDC).
; =====================================================================================================================
.target c128
.cpu 8502 { illegals = deny }
.branchlong auto                        ; zu weite bedingte Spruenge verlaengert der Assembler selbst (Warnung W203)

; ---------------------------------------------------------------------------------------------------------------------
; Nullseiten-Variablen des Spiels ($10-$8B; die Zeiger und Hilfszellen der Zeichen-Sprites folgen weiter unten ab $90)
;   Hinweis: tbu..tbdy (Titelbaelle) sind keine Nullseite, sondern RAM-Adressen ab $C130.
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
.const sp      = $2d                    ; Zeiger auf Bildschirm-Ziel (2 Byte, fuer (sp),y); zeigt in den Zeichenspiegel SCR
.const cp      = $2f                    ; Zeiger auf das Attribut-Ziel im Attributspiegel COLORRAM (2 Byte)
.const tp      = $31                    ; Zeiger auf Text (2 Byte)
.const txtcol  = $33                    ; Textfarbe fuer put_cell/puts
.const atgt    = $34                    ; Autoplay: Zielpunkt auf dem Schlaeger (8..23 px von der linken Kante)
.const ptA     = $35                    ; UNGENUTZT (probe benutzt nur ptB)
.const ptB     = $36                    ;   probe: Y-Versatz-Index (0/1) wird hier zwischengespeichert
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
.const captype = $4c                    ; Art 0..10 (0 B breit, 1 S schmal, 2 R umgekehrt, 3 L langsam, 4 F schnell, 5 K klebt, 6 + Leben, 7 P
                                        ;   Punkte, 8 M Mehrball, 9 G Laser, 10 D Durchschlag)
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
.const sprmsb  = $5c                    ; UNGENUTZT (Rest der VIC-II-Fassung: gesammeltes 9. X-Bit der Hardware-Sprites)
.const sprena  = $5d                    ; UNGENUTZT (Rest der VIC-II-Fassung: Sprite-Freigabebits)
.const tbu     = $c130                  ; Titel-Baelle (3): X in Einheiten zu 2 px
.const tby     = $c133                  ;                    Y in px
.const tbdu    = $c136                  ;                    Schritt X (1 oder $FF)
.const tbdy    = $c139                  ;                    Schritt Y (1, 2 oder negativ)
.const bp      = $61                    ; UNGENUTZT (Rest der VIC-II-Fassung: Farbtabelle der Rasterbalken; hier ersetzt vtheme/vbg)
.const bflash  = $63                    ; Rest-Bilder des Hintergrundblitzes (Register 26; 0 = kein Blitz), fx_frame zaehlt herunter
.const glg     = $64                    ; Glanzlicht: Position der laufenden Diagonale (0..30, danach Pause bis 99)
.const scx     = $65                    ; Lauftext: Bilder bis zum naechsten Zeichenschritt (5..0, also alle 6 Bilder; der VDC kann keine einzelne
                                        ;   Zeile fein verschieben)
.const stp     = $66                    ; Lauftext: Leseposition im Text (UNGENUTZT, ersetzt durch den 16-Bit-Zeiger stq)
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
.const tkc     = $80                    ;         laufendes Zeichen 0..31 der Ticker-Zeile
.const tkr     = $81                    ;         laufende Pixelzeile 0..7 eines Zeichens
.const tkd     = $82                    ;         senkrechter Versatz d der Effekte 0/1 (+1..+8 nach oben, -1..-8 nach unten)
.const tkt     = $83                    ;         Hilfsspeicher (Zielindex im Puffer, Zaehler, Spaltengruppe j)
.const tks     = $84                    ;         Hilfsspeicher (Quellzeile s = r + d, neue Pixel, LFSR-Index)
.const tklf    = $85                    ;         Schieberegister (LFSR) fuer den Aufloese-Effekt, nie 0
.const tkdiv   = $86                    ;         Teilschritt innerhalb eines Effektschritts (Viertel 0..3 bzw. Haelfte 0..1)
.const tkd2    = $87                    ;         Hilfsspeicher von tk_hright
.const tkce    = $89                    ; tk_vert: Endzeichen (exklusiv) dieses Durchlaufs
.const tkre    = $8a                    ; tk_hleft/tk_hright: Endzeile (exklusiv) dieses Durchlaufs
.const tkrs    = $8b                    ; Startzeile
.const wfc     = $60                    ; wait_frames: Zaehler
.const shi     = $5f                    ; update_shots: Schussnummer (ueberlebt hit_brick)
.const tlas    = $5e                    ; Rest-Halbsekunden: Laser aktiv
.const dxo     = $3b                    ; Abtastversatz der Ballkante (X: 0 = links, 1 = rechts; in move_ball bei der Y-Bewegung: 0 = oben, 1 = unten)

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
.const SCR      = $b000                 ; SPIEGEL des Bildschirms: die logischen Zeichencodes (40 x 25), die der VDC gerade zeigt (je logische Zelle zwei VDC-Zellen)
.const COLORRAM = $ac00                 ; SPIEGEL der Attribute je logischer Zelle (VDC-Farbe 0..15; Bit 7 fuer die rechte VDC-Zelle wird erst beim
                                        ;   Schreiben in den VDC dazugesetzt)
.const CHARSET  = $e000                 ; Zeichensatz der logischen Zeichen im Hauptspeicher (256 * 8 Byte, 1 Bit je Pixel): Quelle fuer die Hintergruende der Zeichen-Sprites
                                        ; und fuer den Upload in den VDC (jedes Zeichen wird dort auf zwei Zellen gedehnt)
.const SHC      = $b400                 ; Schattenkopie der Zeichencodes (1000 Byte, Aufbau wie SCR): der Hintergrund ohne Zeichen-Sprites
.const SHF      = $b800                 ; Schattenkopie der Zellfarben (VDC-Attribute)
.const PADY    = 184                    ; obere Kante des Schlaegers in px (Bildschirmzeile 23)
.const PADW    = 32                     ; UNGENUTZT (die Schlaegerbreite steht in der Variablen padw: 16, 32 oder 48 px)
.const LIVES0  = 3                      ; Anzahl Baelle zu Spielbeginn

; Zeichencodes (Bildschirmcode) der eigenen Zeichen
.const CH_BRICK_L = 64
.const CH_BRICK_R = 65
.const CH_WALL    = 66
.const CH_BLOCK   = 67                    ; Block fuer das Titellogo
.const CH_SOLID   = 127                   ; voller Block
.const CH_SPACE   = 32
.const POOL0      = 128                   ; erstes Zeichen des Pools der Zeichen-Sprites (Codes 128..191; 224..255 gehoeren dem Ticker)
.const NPOOL      = 64                    ; Anzahl Pool-Zeichen

; VDC (8563/8568, 80 Spalten): Statusregister $D600 (Bit 7 = bereit), Datenregister $D601 (in c128/hardware.asm heissen sie VDC_ADDR/VDC_DATA).
;   VDC-Speicher: Bildschirm $0000, Attribute $0800, Zeichensatz $2000
; (Zeichen 0..255, je 16 Byte, nur die ersten 8 Zeilen werden benutzt), zweiter Zeichensatz $3000 (Attributbit 7).
.const VDC_ST   = $d600
.const VDC_DT   = $d601
.const VDC_SCR  = $0000
.const VDC_ATTR = $0800
.const VDC_CHR  = $2000
.const VDC_ALT  = $3000

; Zeichen-Sprites (Compositor): Nullseite $90-$AD (der KERNAL benutzt $90 und aufwaerts nur waehrend der Bestenliste, danach ist alles neu)
.const shp     = $90                    ; Zeiger auf die Schattenkopie der Zeichencodes (2 Byte)
.const shf     = $92                    ; Zeiger auf die Schattenkopie der Farben
.const ip      = $94                    ; Zeiger auf den Datenstreifen des Bildes (2 Byte)
.const mp      = $96                    ; Zeiger auf den Maskenstreifen
.const dp      = $98                    ; Zeiger auf die Pixeldaten des Pool-Zeichens (Ziel)
.const bq      = $9a                    ; Zeiger auf die Pixeldaten des Hintergrundzeichens (Quelle)
.const ip0     = $9c                    ; Anfang der Streifen des Bildes fuer die gewaehlte Feinlage (Daten)
.const mp0     = $9e                    ;                                                              (Maske)
.const cx      = $a0                    ; Zeichenspalte der linken Kante des Sprites
.const cy      = $a1                    ; Zeichenzeile der Oberkante
.const cs      = $a2                    ; Feinlage 0..7 (Pixel innerhalb der Zeichenspalte)
.const cv      = $a3                    ; senkrechte Feinlage 0..7 (Pixelzeilen)
.const cn      = $a4                    ; Anzahl Zeichenspalten, die das Sprite belegt
.const cr      = $a5                    ; Anzahl Zeichenzeilen
.const cimg    = $a6                    ; Bildnummer
.const cslot   = $a7                    ; laufender Sprite-Schlitz
.const ccol    = $a8                    ; Zellfarbe des Sprites
.const crow    = $a9                    ; laufende Zeichenzeile (Absolutwert)
.const ccnt    = $aa                    ; laufende Zeichenspalte im Sprite
.const rcnt    = $ab                    ; laufende Zeichenzeile im Sprite
.const ctmp    = $ac                    ; Hilfsspeicher
.const vt0     = $ad                    ; Hilfsspeicher der VDC-Routinen
.const vt1     = $ae
.const dbx     = $af                    ; Hilfsspeicher fuer draw_brick, Glanzlicht und Ticker
.const dby     = $b0
.const tku0    = $b1                    ; Ticker-Upload: Startindex im Puffer, Zeichennummer, Zaehler
.const tku1    = $b2
.const tku2    = $b3

; Zeichen-Sprite-Schlitze (RAM): 0..2 Baelle, 3 Schlaeger, 4 Muenze, 5..6 Laserschuesse (8 Byte je Tabelle, Schlitz 7 frei). Je Schlitz: aktiv, X
;   (Bildpunkte 0..319:
; niedrig sl_x und hoch sl_xh = 0/1), Y (Pixel, Oberkante), Bildnummer (IMG_*), Zellfarbe (VDC-Farbe 0..15). Die Wiedergabe erfolgt in tsp_render
;   (einmal je Bild).
.const sl_act  = $cc00
.const sl_x    = $cc08
.const sl_xh   = $cc28
.const sl_y    = $cc10
.const sl_img  = $cc18
.const sl_col  = $cc20
.const dl_lo   = $cc40                  ; Liste der belegten Pool-Zellen (Adresse im Spiegel SCR), Index = Pool-Nummer
.const dl_hi   = $cc80
.const pool_n  = $ccc0                  ; Anzahl bisher vergebener Pool-Zeichen in diesem Bild
.const boot_dev = $ccc1                 ; UNGENUTZT (das Startgeraet merkt sich hs_init in hs_bootdev, hiscore.asm)
.const ol_n    = $ccc2                  ; Anzahl Pool-Zellen des vorigen Bildes (tsp_render)
.const ol_lo   = $cd00                  ; ihre Spiegeladressen
.const ol_hi   = $cd40

; Vorab: Bibliotheken (Hardware-Konstanten, Eingabe, Zufall, Klang, Bestenliste) aus data/sys/c128/
#include <c128/kernal.asm>      ; (enthaelt hardware.asm und eine Konstante SCREEN; unsere Bildschirmadresse heisst deshalb SCR)

.org $1c01                              ; Start des BASIC-Speichers des C128
.basicstub start                        ; erzeugt "10 SYS <start>", damit RUN das Programm startet

; Bestenliste ZUERST einbinden: ihre Umschalt-Routinen (hs_io_on/off) muessen unter $4000 liegen (dort ist immer RAM; die Systemroms beginnen bei
; $4000 und wuerden bei der kurzen Umschaltung auf Konfiguration $00 den eigenen Code verdecken).  Auch copy_font (Zeichen-ROM lesen) steht darum vorn.
.include "../prallball/lib/hiscore.asm"

; ---------------------------------------------------------------------------------------------------------------------
; Programmstart
;   Dies ist die VDC-Fassung (80 Spalten): Das Bild liefert der VDC, der VIC-II ist dunkel (Bild aus) und liefert nur den Bildtakt (Rasterinterrupt).
;   Die MMU-Konfiguration $3E (alles RAM der Bank 0 plus E/A) gilt wie in der VIC-II-Fassung; der Prozessor laeuft mit 2 MHz.
; ---------------------------------------------------------------------------------------------------------------------
start:  sei                             ; Interrupts aus: Wir brauchen weder die KERNAL-Uhr noch deren Tastaturabfrage; cli erst nach der Einrichtung
        jsr hs_init                     ; Startgeraet fuer die Bestenliste merken ($BA)
        lda #$7f                        ; Interrupt-Maskenregister der CIAs: Bit 7 = 0 heisst, die gesetzten Bits 0..6 SPERREN diese Quellen
        sta CIA1_ICR                    ; CIA 1 (Tastatur-/Zeitgeber-Interrupt): aus
        sta $dd0d                       ; CIA 2 (ICR bei $DD0D): aus
        lda CIA1_ICR                    ; Lesen quittiert eine eventuell anstehende Anforderung
        lda $dd0d
        jsr copy_font                   ; Zeichen-ROM -> CHARSET (+ eigene Zeichen); danach MMU: alles RAM
        jsr hs_load                     ; Bestenliste von der Diskette (die Systemroms werden kurz eingeschaltet; Takt und VIC noch im Zustand des Betriebssystems); bei Fehlern bleibt die Standardliste
        jsr vic_init                    ; VIC-II: Bild aus, Rasterinterrupt (Bildtakt)
        jsr vdc_init                    ; VDC einrichten
        jsr vdc_font                    ; alle 256 Zeichen gedehnt in den VDC laden
        jsr sfx_init                    ; SID: Lautstaerke 15, Stimme 3 fuer Effekte
        jsr input_init                  ; Eingabezustand loeschen
.ifdef AUTOPLAY
        lda #1
        sta sfx_mute                    ; Demo-Modus: keine Soundeffekte (Vorgabe: nur die Musik spielt weiter)
.endif
        lda VIC_RASTER                  ; die aktuelle Rasterzeile ist als Startwert fuer den Zufall gut genug
        jsr rand_seed
        lda #<rti_only                  ; NMI- und Reset-Vektor (RAM) zeigen auf ein blosses rti
        sta $fffa
        sta $fffc
        lda #>rti_only
        sta $fffb
        sta $fffd
        cli                             ; jetzt laeuft der Rasterinterrupt (Bildtakt)
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

rti_only: rti                           ; Interruptvektor fuer NMI/RESET: tut nichts

; ---------------------------------------------------------------------------------------------------------------------
; copy_font: kopiert den Zeichensatz (Zeichen-ROM, 2 KB) nach CHARSET und setzt die eigenen Zeichen 64..67 und 127 ein.
;   Das Zeichen-ROM liegt bei $D000; sichtbar ist es bei MMU-Konfiguration $01 (Systemroms, E/A AUS; der Code hier steht unter $4000 und bleibt sichtbar).
;   Zeichen n beginnt bei CHARSET + n * 8, 1 Bit je Pixel (Bit 7 = links).  1..26 = Buchstaben, 48..57 = Ziffern, 32 = Leerzeichen.
; Veraendert A, X, Y, sp, cp.
; ---------------------------------------------------------------------------------------------------------------------
copy_font:
        lda #$01
        sta MMU_CR_FF00                 ; Zeichen-ROM bei $D000 sichtbar (E/A abgeschaltet; die Schreibzugriffe nach CHARSET gehen in das RAM unter
                                        ;   dem ROM)
        lda #<$d000
        sta sp
        lda #>$d000
        sta sp + 1
        lda #<CHARSET
        sta cp
        lda #>CHARSET
        sta cp + 1
        ldx #8                          ; 8 Seiten a 256 Byte = 2048 Byte
        ldy #0
.cf:    lda (sp),y
        sta (cp),y
        iny
        bne .cf
        inc sp + 1
        inc cp + 1
        dex
        bne .cf
        lda #MMUCFG_RAM0_IO             ; $3E: alles RAM der Bank 0 plus E/A (ab hier laeuft das Spiel ohne Systemroms)
        sta MMU_CR_FF00
        ldx #31                         ; 4 Zeichen a 8 Byte (64..67), rueckwaerts kopieren
.cc:    lda gfx_chars,x
        sta CHARSET + 64 * 8,x
        dex
        bpl .cc
        ldx #7                          ; Zeichen 127: voller Block
.cs:    lda gfx_chars + 32,x
        sta CHARSET + CH_SOLID * 8,x
        dex
        bpl .cs
        rts

; ---------------------------------------------------------------------------------------------------------------------
; vic_init: der VIC-II zeigt in der VDC-Fassung nichts (Bild aus, Bit 4 von $D011 = 0) und arbeitet nur als Taktgeber: ein Rasterinterrupt je Bild in Zeile 251
;   setzt vblank (wait_frame).  Der Prozessor laeuft mit 2 MHz ($D030 Bit 0); bei ausgeschaltetem VIC-II-Bild stoert das nichts, und der VDC-Zugriff
;   profitiert davon.  Der Interrupt-Vektor steht im RAM ($FFFE), weil das Spiel ohne Systemroms laeuft.
; ---------------------------------------------------------------------------------------------------------------------
vic_init:
        lda #$0b
        sta VIC_CTRL1                   ; Bild aus (Bit 4 = 0), 25 Zeilen (Bit 3), Y-Versatz 3, Bit 7 = 0: die Rasterzeile hat nur 8 Bit (Zeile 251
                                        ;   genuegt)
        lda #0
        sta VIC_SPR_ENABLE
        lda VIC_CLOCK
        ora #$01
        sta VIC_CLOCK                   ; 2 MHz
        lda #<irq
        sta $fffe                       ; Interruptvektor im RAM (alles RAM sichtbar)
        lda #>irq
        sta $ffff
        lda #251
        sta VIC_RASTER                  ; Interruptzeile (unterhalb des sichtbaren Bildes)
        lda #$ff
        sta VIC_IRQ_FLAGS               ; alte Anforderungen loeschen
        lda #1
        sta VIC_IRQ_ENABLE              ; Rasterinterrupt freigeben
        lda #0
        sta vblank
        sta bflash
        rts

; irq: Rasterinterrupt, einmal je Bild (Zeile 251): setzt nur das Flag vblank (Bildtakt fuer wait_frame). Die Systemroms sind aus, also rettet der
;   Interrupt A, X, Y selbst
;   (Vektor $FFFE im RAM); es gibt keine anderen Interruptquellen (CIAs gesperrt).
irq:    pha
        txa
        pha
        tya
        pha
        lda #1
        sta VIC_IRQ_FLAGS               ; Anforderung quittieren
        sta vblank
.ifdef TESTFPS
        inc $c202                       ; Testfassung: Bildwechsel (16 Bit) zaehlen
        bne .ifp
        inc $c203
.ifp:
.endif
        pla
        tay
        pla
        tax
        pla
        rti

; ---------------------------------------------------------------------------------------------------------------------
; VDC-Zugriff. Jeder Zugriff wartet, bis Bit 7 des Statusregisters ($D600) gesetzt ist (bit/bpl-Schleife); danach darf das Datenregister ($D601)
;   gelesen/beschrieben werden.
; Der VDC hat 37 Register: Nummer nach $D600, dann Wert nach $D601.  Register 18/19 = Speicheradresse, Register 31 = Datenbyte (die Adresse zaehlt danach hoch).
; ---------------------------------------------------------------------------------------------------------------------
vdc_reg:                                ; X = Registernummer, A = Wert; A, X, Y bleiben erhalten
        stx VDC_ST
.w:     bit VDC_ST
        bpl .w
        sta VDC_DT
        rts
vdc_rd:                                 ; X = Registernummer -> A = Wert; X, Y bleiben
        stx VDC_ST
.w:     bit VDC_ST
        bpl .w
        lda VDC_DT
        rts
vdc_addr:                               ; A = VDC-Adresse hoch, Y = niedrig (Register 18 und 19).  A und Y bleiben, X = 19.
        ldx #18
        stx VDC_ST
.w1:    bit VDC_ST
        bpl .w1
        sta VDC_DT
        inx
        stx VDC_ST
.w2:    bit VDC_ST
        bpl .w2
        sty VDC_DT
        rts
vdc_get:                                ; A = VDC-Adresse hoch, Y = niedrig -> A = Byte an dieser Adresse (Register 31); X = 31
        jsr vdc_addr
        ldx #31
        jmp vdc_rd
vdc_put:                                ; A = Byte in den VDC-Speicher an die aktuelle Adresse (Register 31, Adresse zaehlt danach hoch); A und Y
                                        ;   bleiben, X = 31
        ldx #31
        stx VDC_ST
.w:     bit VDC_ST
        bpl .w
        sta VDC_DT
        rts

; vdc_init: Textmodus 80 x 25 einrichten (die Zeitregister 0..7 und die Zeichenhoehe hat das Betriebssystem beim Einschalten gesetzt und bleiben unberuehrt).
;   Veraendert A, X (vdc_bgrestore am Ende); schreibt nur R10, R12/13, R20/21, R25, R26, R28 (R25 und R28 mit Lesen-Aendern-Schreiben).
;   Cursor aus, Bildschirm bei $0000, Attribute bei $0800, Zeichensatz bei $2000 (zweiter bei $3000), Attribute an (Bit 6 von Register 25), keine Bitmap,
;   kein waagerechter Versatz, Hintergrundfarbe schwarz.
vdc_init:
        ldx #10
        lda #$20
        jsr vdc_reg                     ; Cursor aus
        ldx #12
        lda #>VDC_SCR
        jsr vdc_reg
        ldx #13
        lda #<VDC_SCR
        jsr vdc_reg                     ; Bildschirmanfang
        ldx #20
        lda #>VDC_ATTR
        jsr vdc_reg
        ldx #21
        lda #<VDC_ATTR
        jsr vdc_reg                     ; Attributanfang
        ldx #28
        jsr vdc_rd
        and #$1f
        ora #$20                        ; Bits 5..7 = Zeichensatzadresse Bit 15..13 = 001 ($2000); Bit 4 (DRAM-Art) bleibt
        ldx #28
        jsr vdc_reg
        ldx #25
        jsr vdc_rd
        and #$30
        ora #$40                        ; Attribute an, Textmodus, Versatz 0
        ldx #25
        jsr vdc_reg
        lda #0
        sta vbg                         ; Hintergrund schwarz (Titelthema)
        jmp vdc_bgrestore

; vdc_font: alle 256 logischen Zeichen aus CHARSET gedehnt in den VDC laden. Das linke Halbzeichen (die linken 4 Pixel = Bit 7..4, je doppelt, ueber
;   dbl_l) kommt in
;   den ersten Zeichensatz ($2000), das rechte (Bit 3..0, ueber dbl_r) in den zweiten ($3000); eine logische Zelle besteht auf dem VDC aus zwei
;   Zellen: links
;   Zeichen c, rechts Zeichen c mit Attributbit 7 (zweiter Zeichensatz). Je Zeichen 16 Byte; die letzten 8 (Zeilen 8..15) bleiben 0. Veraendert A,
;   X, Y, dp, vt0.
;   Schnell geschrieben: R31 bleibt gewaehlt, die VDC-Adresse zaehlt selbst weiter (jeder Satz = 256 Zeichen * 16 Byte = 4096 Byte).
vdc_font:
        lda #>VDC_CHR
        ldy #<VDC_CHR
        jsr vdc_addr
        lda #<CHARSET
        sta dp
        lda #>CHARSET
        sta dp + 1
        lda #0
        sta vt0
        ldx #31
        stx VDC_ST                      ; Register 31 bleibt gewaehlt
.l1:    ldy #0                          ; Schleife ueber die 256 Zeichen (vt0 zaehlt 0..255), je Zeichen 8 Pixelzeilen
.l2:    lda (dp),y
        tax
        lda dbl_l,x                     ; linke 4 Pixel (Bit 7..4) auf 8 Pixel gedehnt
.l3:    bit VDC_ST
        bpl .l3
        sta VDC_DT
        iny
        cpy #8
        bne .l2
        ldy #8                          ; die Zeilen 8..15 des 16-Byte-Zeichens (VDC-Zeichen sind 16 Zeilen hoch) mit 0 auffuellen
        lda #0
.l4:    bit VDC_ST
        bpl .l4
        sta VDC_DT
        dey
        bne .l4
        clc
        lda dp
        adc #8
        sta dp
        bcc +
        inc dp + 1
+       inc vt0
        bne .l1
        lda #>VDC_ALT
        ldy #<VDC_ALT
        jsr vdc_addr
        lda #<CHARSET
        sta dp
        lda #>CHARSET
        sta dp + 1
        ldx #31
        stx VDC_ST
.m1:    ldy #0                          ; dasselbe fuer den zweiten Zeichensatz
.m2:    lda (dp),y
        tax
        lda dbl_r,x                     ; rechte 4 Pixel (Bit 3..0) auf 8 Pixel gedehnt
.m3:    bit VDC_ST
        bpl .m3
        sta VDC_DT
        iny
        cpy #8
        bne .m2
        ldy #8                          ; Zeilen 8..15 wieder mit 0 auffuellen
        lda #0
.m4:    bit VDC_ST
        bpl .m4
        sta VDC_DT
        dey
        bne .m4
        clc
        lda dp
        adc #8
        sta dp
        bcc +
        inc dp + 1
+       inc vt0
        bne .m1
        rts

; vdc_idx: Adresse der Zelle (sp zeigt in den Spiegel SCR) im VDC: (sp - SCR) * 2 -> A = hoch, Y = niedrig (Zeichenbereich ab $0000). Veraendert
;   vt0. Weil eine logische Zelle zwei VDC-Zellen belegt,
;   liegt die Zelle Zeile * 40 + Spalte bei Zeile * 80 + Spalte * 2 = Index * 2.
vdc_idx:
        lda sp + 1
        sec
        sbc #>SCR
        sta vt0                         ; Hochbyte des Zellindex (Index = sp - SCR, 0..999)
        lda sp
        asl                             ; Index * 2: Low-Byte, Bit 7 wandert in den Uebertrag ...
        tay                             ; Y = Low-Byte der VDC-Adresse
        rol vt0                         ; ... und von dort in das Hochbyte
        lda vt0                         ; A = Hochbyte der VDC-Adresse
        rts

; vdc_cell_sp: die Zelle (sp) mit Zeichencode (sp) und Attribut (cp) in den VDC schreiben (zwei VDC-Zellen: links Zeichen c, rechts c mit Attributbit 7).
;   Eingabe: sp = Zeiger in SCR, cp = Zeiger in COLORRAM (gleiche Zelle).  Veraendert A, X, Y, vt0, vt1.
vdc_cell_sp:
        jsr vdc_idx
        sta vt0
        sty vt1
        jsr vdc_addr
        ldy #0
        lda (sp),y
        jsr vdc_put
        lda (sp),y
        jsr vdc_put
        jmp vdc_attr2

; vdc_attr_sp: nur das Attribut der Zelle in den VDC schreiben. Eingabe: sp (Zeiger in SCR, fuer die Adresse) und cp (Zeiger in COLORRAM, fuer den
;   Wert). Veraendert A, X, Y, vt0, vt1.
vdc_attr_sp:
        jsr vdc_idx
        sta vt0
        sty vt1
vdc_attr2:
        lda vt0
        clc
        adc #>VDC_ATTR
        ldy vt1
        jsr vdc_addr
        ldy #0
        lda (cp),y
        jsr vdc_put
        ora #$80                        ; rechte Zelle: zweiter Zeichensatz
        jmp vdc_put

; vcol: Farbumsetzung C64-Farbnummer (Index 0..15) -> VDC-Attributfarbe (RGBI: 0 schwarz, 1 dunkelgrau, 2 dunkelblau, 3 hellblau, 4 dunkelgruen, 5
;   hellgruen, 6 dunkelcyan,
; 7 hellcyan, 8 dunkelrot, 9 hellrot, 10 dunkelviolett, 11 hellviolett, 12 braun/dunkelgelb, 13 hellgelb, 14 hellgrau, 15 weiss).
;   C64: 0 schwarz 1 weiss 2 rot 3 tuerkis 4 violett 5 gruen 6 blau 7 gelb, 8..15 = 8 + Farbe (Mehrfarbzelle der C64-Fassung; hier wie Farbe 0..7, nur 8 =
;   "schwarz" der Mehrfarbzelle wird dunkelgrau, damit unzerstoerbare Steine sichtbar bleiben)
vcol:   .byte 0, 15, 9, 7, 11, 5, 3, 13, 1, 15, 9, 7, 11, 5, 3, 13
vbg:    .byte 0                         ; aktuelle Hintergrundfarbe (Thema), Tiefnibble fuer Register 26
vtheme: .byte 2, 4, 8, 0                ; Hintergrund je Verlauf 0..3: dunkelblau, dunkelgruen, dunkelrot, schwarz

; vdc_bgrestore: Hintergrundfarbe auf das Thema vbg setzen (Register 26: Hoch-Nibble Vordergrund = $F, Tief-Nibble Hintergrund). Veraendert A, X
;   (ein Eingabewert in A wird ignoriert).
vdc_bgrestore:
        lda vbg
        ora #$f0
        ldx #26
        jmp vdc_reg

; set_theme: Verlauf A (0..3) waehlen: der VDC kennt keine Rasterbalken; stattdessen bekommt der Hintergrund eine dunkle Themenfarbe (Tabelle
;   vtheme). Veraendert A, X.
set_theme:
        tax
        lda vtheme,x
        sta vbg
        jmp vdc_bgrestore

; flash: Hintergrund kurz in Farbe A (C64-Farbnummer 0..15) faerben fuer X Bilder (fx_frame zaehlt bflash herunter und stellt danach das Thema her).
;   X < 4 wird ignoriert (siehe unten).  Veraendert A, X, Y.
flash:  cpx #4
        bcc .fl0                        ; ganz kurze Blitze (weisser Blitz bei jedem Steintreffer) lassen wir weg: auf dem VDC faerbt er den ganzen Bildschirm
        stx bflash
        tay
        lda vcol,y
        ora #$f0
        ldx #26
        jmp vdc_reg
.fl0:   rts

band:     .byte 0                       ; UNGENUTZT (Rest der VIC-II-Fassung)
vblank:   .byte 0                       ; 1 = Bildwechsel seit dem letzten wait_frame (setzt irq)

; ---------------------------------------------------------------------------------------------------------------------
; Bildschirm-Hilfen
;   Der Bildschirm lebt im VDC (nicht im Hauptspeicher); im Hauptspeicher stehen Spiegel (SCR, COLORRAM: was der VDC zeigt) und Schattenkopien
;   (SHC, SHF: der Hintergrund ohne Zeichen-Sprites).  put_cell und die uebrigen Schreiber halten alle vier aktuell und schreiben die Zelle in den VDC.
; ---------------------------------------------------------------------------------------------------------------------
; clear_screen: Bildschirm = Leerzeichen, Farbe weiss; Spiegel, Schattenkopien und VDC.  Danach gibt es keine Pool-Zellen mehr.  Veraendert A, X, Y.
;   Die Schleife .cs fuellt je Bereich vier Teile (Offset $000, $100, $200 und $2E8); der letzte ueberlappt, damit genau 1000 Byte gefuellt werden.
clear_screen:
        ldx #0
.cs:    lda #CH_SPACE
        sta SCR,x
        sta SCR + $100,x
        sta SCR + $200,x
        sta SCR + $2e8,x
        sta SHC,x
        sta SHC + $100,x
        sta SHC + $200,x
        sta SHC + $2e8,x
        lda #15                         ; weiss (VDC-Farbe 15)
        sta COLORRAM,x
        sta COLORRAM + $100,x
        sta COLORRAM + $200,x
        sta COLORRAM + $2e8,x
        sta SHF,x
        sta SHF + $100,x
        sta SHF + $200,x
        sta SHF + $2e8,x
        inx
        bne .cs
        lda #0
        sta pool_n
        lda #>VDC_SCR                   ; VDC: 2000 Bytes Leerzeichen ...
        ldy #<VDC_SCR
        jsr vdc_addr
        ldx #31
        stx VDC_ST
        lda #CH_SPACE
        ldx #8
        ldy #0
.v1:    bit VDC_ST                      ; 8 * 250 = 2000 Bytes (Y zaehlt 250 herunter, X die Blocks)
        bpl .v1
        sta VDC_DT
        iny
        cpy #250
        bne .v1
        ldy #0
        dex
        bne .v1
        lda #>VDC_ATTR                  ; ... und 2000 Attribute: abwechselnd weiss und weiss mit Bit 7 (rechte Zellen: zweiter Zeichensatz)
        ldy #<VDC_ATTR
        jsr vdc_addr
        ldx #31
        stx VDC_ST
        ldx #0
        ldy #0
.v2:    lda #15
.v3:    bit VDC_ST
        bpl .v3
        sta VDC_DT
        lda #15 | $80
.v4:    bit VDC_ST
        bpl .v4
        sta VDC_DT
        inx
        bne .v2
        iny
        cpy #4
        bne .v2                         ; 4 * 256 = 1024 Paare (mehr als die 1000 noetigen: die letzten 24 liegen hinter dem sichtbaren Bild)
        rts

; set_rowptr: sp (Spiegel SCR), cp (Spiegel COLORRAM), shp (Schattencodes) und shf (Schattenfarben) auf Zeile Y, Spalte X setzen.
;   Adresse = SCR + Zeile * 40 + Spalte; alle vier Bereiche haben denselben Aufbau und unterscheiden sich nur im oberen Adressbyte.
; Eingabe: X = Spalte 0..39, Y = Zeile 0..24.  Veraendert A.
set_rowptr:
        txa
        clc
        adc rowlo,y
        sta sp
        sta cp
        sta shp
        sta shf
        lda rowhi,y
        adc #0                          ; Uebertrag aus dem Low-Byte (Zeilenanfang + Spalte)
        sta sp + 1
        clc
        adc #(>SHC) - (>SCR)            ; Schattencodes
        sta shp + 1
        lda sp + 1
        clc
        adc #(>SHF) - (>SCR)            ; Schattenfarben
        sta shf + 1
        lda sp + 1
        sec
        sbc #(>SCR) - (>COLORRAM)       ; Attributspiegel liegt 4 Seiten tiefer
        sta cp + 1
        rts

; put_cell: Zeichen A mit Farbe txtcol (C64-Farbnummer, siehe vcol) an Spalte X, Zeile Y ausgeben (Spiegel, Schattenkopie und VDC).   Veraendert A, X, Y.
put_cell:
        pha
        jsr set_rowptr
        ldy #0
        pla
        sta (sp),y
        sta (shp),y
        ldx txtcol
        lda vcol,x
        sta (cp),y
        sta (shf),y
        jmp vdc_cell_sp

; wr_attr: nur die Farbe (VDC-Attribut A, 0..15) der Zelle Spalte X, Zeile Y setzen (Spiegel, Schattenkopie, VDC).   Veraendert A, X, Y.
wr_attr:
        pha
        jsr set_rowptr
        ldy #0
        pla
        sta (cp),y
        sta (shf),y
        jmp vdc_attr_sp

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

; ---------------------------------------------------------------------------------------------------------------------
; ZEICHEN-SPRITES (Compositor)
;   Der VDC hat KEINE Sprites.  Ball, Schlaeger, Muenze und Laserschuesse sind deshalb Zeichen: vor jedem Bild werden die Zellen unter den
;   Sprites neu mit "Pool-Zeichen" (Codes 128..191) belegt, deren Bitmuster aus dem Hintergrundzeichen und dem Bild des Sprites zusammengesetzt wird.
;   Ablauf von tsp_render (einmal je Bild, gleich nach wait_frame):
;     1. alle Zellen des letzten Bildes aus den Schattenkopien SHC/SHF im SPIEGEL wiederherstellen (der VDC bleibt vorerst unveraendert: Zellen, die
;     auch jetzt
;        wieder belegt werden, sollen nicht kurz leer erscheinen); die alte Liste der Pool-Zellen wird nach ol_* gerettet,
;     2. die aktiven Schlitze (sl_*) in der Reihenfolge Schlaeger, Muenze, Schuesse, Baelle (zuletzt = oben) zeichnen: je Zelle ein Pool-Zeichen vergeben
;        (oder das schon vergebene weiterverwenden) und dessen 8 Zeilen im Hauptspeicher (CHARSET) setzen als
;            Ergebnis = (Hintergrund UND Maske) ODER Bild,
;     3. Zellen des letzten Bildes, die jetzt nicht mehr von einem Sprite belegt sind, im VDC wiederherstellen,
;     4. tsp_flush: die neuen Pool-Zeichen gedehnt in den VDC laden und die Zellen in den VDC schreiben.
;   Grenzen: 64 Pool-Zeichen (NPOOL); ist der Pool voll, entfaellt die Zelle.  Zeile 24 und tiefer sowie Spalten ab 40 bekommen keine Sprites.
;   Bilder: lib/sprites.inc (erzeugt von tools/gen.py): einfarbig, 1 Bit je Pixel, 8 Feinlagen (0..7 Pixel nach rechts) und je Zeichenspalte ein Streifen mit
;   8 Leerzeilen davor und danach (so kann jede senkrechte Verschiebung v = 0..7 ohne Pruefungen gelesen werden: Zeile L des Zeichens liest den Streifen bei
;   Index 8 - v + 8 * (Zeichenzeile im Sprite) + L).
;   Schlitz: sl_x/sl_xh = X in Bildpunkten (0..319), sl_y = Y in Bildpunkten.
; ---------------------------------------------------------------------------------------------------------------------
; tsp_init: alle Schlitze aus, Pool leer.  Veraendert A, X.
tsp_init:
        ldx #7
        lda #0
.ti:    sta sl_act,x
        dex
        bpl .ti
        sta pool_n
        rts

; tsp_hide: alle Schlitze aus (die Zellen werden beim naechsten tsp_render wiederhergestellt).  Veraendert A, X.
tsp_hide:
        ldx #7
        lda #0
.th:    sta sl_act,x
        dex
        bpl .th
        rts

; tsp_render: die Schlitze sl_* als Pool-Zeichen in Spiegel und VDC bringen (Ablauf siehe Abschnittskopf).  Einmal je Bild, NICHT im Interrupt.
;   Veraendert A, X, Y und die Hilfszellen des Compositors (sp, cp, shp, shf, ip, mp, dp, bq, c*, vt*).
tsp_render:
        ; --- 1. Zellen des letzten Bildes im SPIEGEL wiederherstellen (noch nicht im VDC: Zellen, die auch jetzt wieder von einem Sprite belegt werden, sollen
        ;        nicht kurz leer erscheinen); die alte Liste wird nach ol_* gerettet
        ldx pool_n
        stx ol_n
        beq .r2
.r1:    dex
        lda dl_lo,x
        sta ol_lo,x
        sta sp
        sta shp
        sta shf
        sta cp
        lda dl_hi,x
        sta ol_hi,x
        sta sp + 1
        clc
        adc #(>SHC) - (>SCR)
        sta shp + 1
        lda sp + 1
        clc
        adc #(>SHF) - (>SCR)
        sta shf + 1
        lda sp + 1
        sec
        sbc #(>SCR) - (>COLORRAM)
        sta cp + 1
        ldy #0
        lda (shp),y
        sta (sp),y
        lda (shf),y
        sta (cp),y
        cpx #0
        bne .r1
.r2:    lda #0
        sta pool_n
        ; --- 2. Sprites zeichnen
        ldx #0
.r3:    ldy sl_order,x
        lda sl_act,y
        beq .r4
        stx ctmp                        ; Position in der Reihenfolge merken (tsp_draw verbraucht X)
        sty cslot
        jsr tsp_draw
        ldx ctmp
.r4:    inx
        cpx #7
        bne .r3
        ; --- 3. Zellen des letzten Bildes, die jetzt NICHT mehr von einem Sprite belegt sind, im VDC wiederherstellen
        ldx ol_n
        beq tsp_flush
.o1:    dex
        stx ctmp
        lda ol_lo,x
        sta sp
        sta cp
        lda ol_hi,x
        sta sp + 1
        sec
        sbc #(>SCR) - (>COLORRAM)
        sta cp + 1
        ldy #0
        lda (sp),y
        cmp #POOL0
        bcc .o2                         ; kein Pool-Zeichen mehr: Hintergrund zurueck in den VDC
        cmp #POOL0 + NPOOL
        bcc .o3                         ; wieder von einem Sprite belegt: der VDC bekommt es unten im tsp_flush
.o2:    jsr vdc_cell_sp
.o3:    ldx ctmp
        bne .o1
        ; --- 4. neue Pool-Zeichen und Zellen in den VDC
; tsp_flush: alle Pool-Zellen dieses Bildes (dl_lo/dl_hi, pool_n Stueck) mit Bitmuster (vdc_pool) und Zeichencode/Attribut (vdc_cell_sp) in den VDC
;   schreiben.
tsp_flush:
        lda #0
        sta ctmp                        ; Pool-Nummer
.fl:    lda ctmp
        cmp pool_n
        bcs .fl9
        tax
        lda dl_lo,x
        sta sp
        sta cp
        lda dl_hi,x
        sta sp + 1
        sec
        sbc #(>SCR) - (>COLORRAM)
        sta cp + 1
        jsr vdc_pool                    ; Bitmuster des Pool-Zeichens (gedehnt) in den VDC
        jsr vdc_cell_sp                 ; Zelle (Zeichencode und Attribut) in den VDC
        inc ctmp
        jmp .fl
.fl9:   rts
sl_order: .byte 3, 4, 5, 6, 0, 1, 2     ; Zeichenreihenfolge: spaeter = weiter oben

; vdc_pool: das Pool-Zeichen ctmp (Zeichencode POOL0 + ctmp) gedehnt in beide Zeichensaetze des VDC schreiben (je 8 Byte: links die linken 4 Pixel
;   ueber dbl_l,
;   rechts die rechten 4 Pixel ueber dbl_r). VDC-Adresse = Satzanfang ($2000 bzw. $3000) + Zeichencode * 16. Eingabe: ctmp. Veraendert A, X, Y, dp,
;   vt0, vt1.
vdc_pool:
        lda ctmp
        clc
        adc #POOL0
        sta vt1                         ; Zeichencode
        tax
        lda chr_lo,x
        sta dp
        lda chr_hi,x
        sta dp + 1
        lda vt1                         ; VDC-Adresse links = VDC_CHR + Code * 16
        lsr
        lsr
        lsr
        lsr
        clc
        adc #>VDC_CHR
        sta vt0
        lda vt1
        asl
        asl
        asl
        asl
        tay
        lda vt0
        jsr vdc_addr
        ldx #31
        stx VDC_ST
        ldy #0
.p1:    lda (dp),y
        tax
        lda dbl_l,x
.p2:    bit VDC_ST
        bpl .p2
        sta VDC_DT
        iny
        cpy #8
        bne .p1
        lda vt1                         ; rechte Haelfte im zweiten Zeichensatz
        lsr
        lsr
        lsr
        lsr
        clc
        adc #>VDC_ALT
        sta vt0
        lda vt1
        asl
        asl
        asl
        asl
        tay
        lda vt0
        jsr vdc_addr
        ldx #31
        stx VDC_ST
        ldy #0
.p3:    lda (dp),y
        tax
        lda dbl_r,x
.p4:    bit VDC_ST
        bpl .p4
        sta VDC_DT
        iny
        cpy #8
        bne .p3
        rts

; tsp_draw: Sprite des Schlitzes cslot zeichnen. Aus sl_x/sl_xh/sl_y entstehen Zeichenspalte cx = X / 8, Feinlage cs = X mod 8 (0 bei Bildern mit
;   nur einer Feinlage),
;   Zeichenzeile cy = Y / 8 und senkrechte Feinlage cv = Y mod 8; cn Spalten und cr Zeilen werden mit tsp_cell gesetzt (Zeilen ab 24 und Spalten ab
;   40 entfallen).
tsp_draw:
        ldy cslot
        lda sl_img,y
        sta cimg
        lda sl_col,y
        sta ccol
        lda sl_x,y
        sta vt0                         ; X niedrig (nicht ctmp: dort steht die Position in der Zeichenreihenfolge)
        lda sl_xh,y
        asl
        asl
        asl
        asl
        asl                             ; 0 oder 32 (X-Bit 8 = 256 px = 32 Zeichenspalten)
        sta cx
        lda vt0
        lsr
        lsr
        lsr
        clc
        adc cx
        sta cx                          ; Zeichenspalte = X / 8
        lda vt0
        and #7
        ldx cimg
        and im_smask,x                  ; Bilder mit nur einer Feinlage (Muenzen) liegen immer auf Zeichengrenze
        sta cs
        lda sl_y,y
        tax
        lsr
        lsr
        lsr
        sta cy                          ; Zeichenzeile
        txa
        and #7
        sta cv                          ; senkrechte Feinlage
        lda cimg
        asl
        asl
        asl
        clc
        adc cs
        tax
        lda im_used,x
        sta cn                          ; Zeichenspalten, die das Bild bei dieser Feinlage belegt
        ldx cimg
        lda im_h,x
        clc
        adc cv
        adc #7
        lsr
        lsr
        lsr
        sta cr                          ; Zeichenzeilen
        ; Anfang der Streifen dieser Feinlage: Bildanfang + cs * (nc * stride)
        lda im_dlo,x
        sta ip0
        lda im_dhi,x
        sta ip0 + 1
        lda im_mlo,x
        sta mp0
        lda im_mhi,x
        sta mp0 + 1
        ldy cs
        beq .o9
.o1:    clc
        lda ip0
        adc im_sset,x
        sta ip0
        bcc +
        inc ip0 + 1
+       clc
        lda mp0
        adc im_sset,x
        sta mp0
        bcc +
        inc mp0 + 1
+       dey
        bne .o1
.o9:    ; Streifenanfang fuer Zeichenzeile r: + 8 - cv + 8 * r
        sec
        lda ip0
        sbc cv
        sta ip0
        bcs +
        dec ip0 + 1
+       clc
        lda ip0
        adc #8
        sta ip0
        bcc +
        inc ip0 + 1
+       sec
        lda mp0
        sbc cv
        sta mp0
        bcs +
        dec mp0 + 1
+       clc
        lda mp0
        adc #8
        sta mp0
        bcc +
        inc mp0 + 1
+       lda #0
        sta rcnt
.rw:    lda cy
        clc
        adc rcnt
        sta crow
        cmp #24
        bcs .rwn                        ; Zeile 24 (Lauftext/Statuszeile) und tiefer: keine Sprites
        tax
        lda cx
        clc
        adc rowlo,x
        sta sp
        lda rowhi,x
        adc #0
        sta sp + 1
        lda ip0                         ; Streifen dieser Zeichenzeile, Zeichenspalte 0
        sta ip
        lda ip0 + 1
        sta ip + 1
        lda mp0
        sta mp
        lda mp0 + 1
        sta mp + 1
        lda #0
        sta ccnt
.cw:    lda cx
        clc
        adc ccnt
        cmp #40
        bcs .cwn                        ; rechts aus dem Bild
        jsr tsp_cell
.cwn:   inc sp                          ; naechste Zeichenspalte
        bne +
        inc sp + 1
+       ldx cimg
        clc
        lda ip
        adc im_stride,x
        sta ip
        bcc +
        inc ip + 1
+       clc
        lda mp
        adc im_stride,x
        sta mp
        bcc +
        inc mp + 1
+       inc ccnt
        lda ccnt
        cmp cn
        bne .cw
.rwn:   clc                             ; naechste Zeichenzeile des Sprites: Streifenanfang + 8
        lda ip0
        adc #8
        sta ip0
        bcc +
        inc ip0 + 1
+       clc
        lda mp0
        adc #8
        sta mp0
        bcc +
        inc mp0 + 1
+       inc rcnt
        lda rcnt
        cmp cr
        bne .rw
        rts

; tsp_cell: eine Zelle (Spiegeladresse sp, Streifen ip/mp, Farbe ccol) im Hauptspeicher setzen (Spiegel, Attribut, Bitmuster); der VDC folgt in tsp_flush.
;   Steht dort schon ein Pool-Zeichen dieses Bildes (anderes Sprite), wird darauf weitergezeichnet; sonst wird ein neues vergeben und mit dem
;   Hintergrundzeichen aus SHC vorbelegt.
tsp_cell:
        ldy #0
        lda (sp),y                      ; Zeichen, das jetzt dort steht
        cmp #POOL0
        bcc .new
        cmp #POOL0 + NPOOL
        bcs .new
        tax                             ; schon ein Pool-Zeichen dieses Bildes (anderes Sprite hier): darauf weiterzeichnen
        lda chr_lo,x
        sta dp
        sta bq
        lda chr_hi,x
        sta dp + 1
        sta bq + 1
        jmp .cmp
.new:   ldx pool_n
        cpx #NPOOL
        bcc +
        rts                             ; Pool voll: diese Zelle entfaellt
+       lda sp                          ; Zelle in die Liste eintragen (zum Wiederherstellen)
        sta dl_lo,x
        sta shp
        lda sp + 1
        sta dl_hi,x
        clc
        adc #(>SHC) - (>SCR)
        sta shp + 1
        lda (shp),y                     ; Hintergrundzeichen dieser Zelle
        tay
        lda chr_lo,y
        sta bq                          ; Quelle: Bitmuster des Hintergrundzeichens
        lda chr_hi,y
        sta bq + 1
        txa
        clc
        adc #POOL0
        ldy #0
        sta (sp),y                      ; Pool-Zeichen in den Spiegel
        tax
        lda chr_lo,x
        sta dp                          ; Ziel: Bitmuster des Pool-Zeichens
        lda chr_hi,x
        sta dp + 1
        inc pool_n
.cmp:   lda sp                          ; Zellfarbe (Attributspiegel = Spiegeladresse - $0400)
        sta cp
        lda sp + 1
        sec
        sbc #(>SCR) - (>COLORRAM)
        sta cp + 1
        lda ccol
        ldy #0
        sta (cp),y
        ldy #7                          ; acht Zeilen: (Hintergrund UND Maske) ODER Bild
.px:    lda (bq),y
        and (mp),y
        ora (ip),y
        sta (dp),y
        dey
        bpl .px
        rts

; Adresse des Bitmusters von Zeichen n im Zeichensatz (n * 8 + CHARSET)
chr_lo:
    .repeat 256, i
        .byte <(CHARSET + i * 8)
    .endrepeat
chr_hi:
    .repeat 256, i
        .byte >(CHARSET + i * 8)
    .endrepeat
; Zeilenanfaenge des Spiegels: SCR + 40 * Zeile
rowlo:
    .repeat 25, i
        .byte <(SCR + i * 40)
    .endrepeat
rowhi:
    .repeat 25, i
        .byte >(SCR + i * 40)
    .endrepeat

; wait_frame: wartet auf den Bildtakt (Rasterinterrupt in Zeile 251; 50 Hz PAL) und quittiert vblank. War der Bildwechsel schon da, kehrt die
;   Routine sofort zurueck.
;   Der Aufrufer setzt danach als Erstes die Zeichen-Sprites (update_sprites bzw. tsp_render).  Veraendert A.
wait_frame:
.w1:    lda vblank
        beq .w1                         ; der Rasterinterrupt setzt vblank
        lda #0
        sta vblank
        rts

; wait_frames: X Bilder warten (X = 1..255; 0 waere 256); dabei laeuft der Klangeffekt weiter (sfx_update).   Veraendert A, X.
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
; title: Titelbild aufbauen und auf Feuer warten (oder 30 s lang ohne Eingabe, dann beginnt die Demo). Rueckkehr mit demo = 0 (Feuer) oder 1
;   (Demo-Modus).
;   Zeilen: 3..7 Logo, 10 Ticker (Untertitel), 12 "Bestenliste", 13..17 Top 5, 20/21 Bedienhinweise, 22 "Feuer = Start", 24 Lauftext. Die Titelmusik
;   laeuft (mus_tick je Bild).
;   Die ersten 40 Bilder (0,8 s) zaehlt keine Eingabe, damit ein noch gedrueckter Knopf aus dem letzten Spiel nicht sofort wieder startet.
title:  lda #0                          ; Thema schwarz (vbg = 0)
        jsr vdc_bgrestore
        jsr title_balls_init            ; drei Baelle huepfen ueber den Titel (Zeichen-Sprites 0..2)
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
        ldx #5
        ldy #20
        jsr puts
        lda #<t_keys
        sta tp
        lda #>t_keys
        sta tp + 1
        ldx #9
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
        lda #0                          ; normaler Titel: kein Demo-Modus, Effekte an
        sta demo
        sta sfx_mute
        ldx #<song                      ; Titelmusik von vorn starten (Liedformat: examples/c128/prallball/lib/sound.asm)
        ldy #>song
        jsr mus_init
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
        jsr input_poll
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

; title_fx: Zierde des Titels, einmal je Bild: Lauftext (title_scroll_tick) und Farbwechsel im Logo (alle 4 Bilder ein Schritt: die Regenbogenfarben
;   der fuenf
;   Logozeilen wandern weiter; nur Zellen mit CH_BLOCK werden umgefaerbt, in Spiegel, Schattenkopie und VDC).  Veraendert A, X, Y, t0..t2.
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
        lda rainbow,x                   ; C64-Farbnummer (8 + Farbe) dieser Zeile
        sta txtcol
        ldx txtcol
        lda vcol,x
        sta t2                          ; VDC-Farbe dieser Logozeile
        ldx #2
.tf2:   stx t0                          ; Spalte 2..37: nur die Logobloecke einfaerben (auch in der Schattenkopie: die Titelbaelle laufen darueber)
        lda t1
        clc
        adc #3
        tay                             ; Bildschirmzeile = 3 + Logozeile
        jsr set_rowptr
        ldy #0
        lda (shp),y
        cmp #CH_BLOCK
        bne .tf3
        lda t2
        sta (cp),y
        sta (shf),y
        jsr vdc_attr_sp
.tf3:   ldx t0
        inx
        cpx #38
        bne .tf2
        inc t1
        lda t1
        cmp #5
        bne .tf1
.tf9:   rts
rainbow: .byte 10, 15, 13, 11, 14, 12, 9, 15            ; rot, gelb, gruen, tuerkis, blau, violett, weiss, gelb (je 8 + Farbe)

; Lauftext in Bildschirmzeile 24 (nur Titel): Der VDC kann nur den ganzen Bildschirm fein verschieben, darum rutscht die Zeile alle 6 Bilder um ein
;   ganzes Zeichen
;   nach links, und rechts kommt das naechste Zeichen des Textes nach (title_scroll_tick). Die Zeile steht im Spiegel (SCR/COLORRAM) und wird mit
;   scroll_flush in den VDC
;   geschrieben (gelb).  Es gibt keine Sprites in Zeile 24 und keine Schattenkopie fuer diese Zeile.
; title_scroll_init: Zeile 24 mit den ersten 40 Zeichen des Textes fuellen, Leseposition stq dahinter, in den VDC schreiben.  Veraendert A, X, Y.
title_scroll_init:
        lda #5
        sta scx
        ldx #39
.sc1:   lda scroll_text,x               ; die ersten 40 Zeichen des Textes
        sta SCR + 24 * 40,x
        lda #13                         ; gelb (VDC-Farbe)
        sta COLORRAM + 24 * 40,x
        dex
        bpl .sc1
        lda #<(scroll_text + 40)        ; Leseposition hinter den ersten 40 Zeichen
        sta stq
        lda #>(scroll_text + 40)
        sta stq + 1
        jmp scroll_flush

; title_scroll_tick: einmal je Bild; zaehlt scx herunter (5..0) und schiebt dann die Zeile um ein Zeichen nach links (neues Zeichen aus (stq), bei 0
;   von vorn).
;   Die Farben der Zeile bleiben unveraendert.  Veraendert A, X, Y.
title_scroll_tick:
        dec scx
        bpl .sk9
        lda #5
        sta scx
        ldx #0
.sk2:   lda SCR + 24 * 40 + 1,x         ; Zeile 24 im Spiegel um ein Zeichen nach links schieben (die Farben bleiben)
        sta SCR + 24 * 40,x
        inx
        cpx #39
        bne .sk2
        ldy #0
        lda (stq),y                     ; naechstes Zeichen
        bne .sk3
        lda #<scroll_text               ; Textende (0): von vorn
        sta stq
        lda #>scroll_text
        sta stq + 1
        lda scroll_text
.sk3:   sta SCR + 24 * 40 + 39
        inc stq                         ; 16-Bit-Zeiger weiter
        bne +
        inc stq + 1
+       jsr scroll_flush
.sk9:   rts

; scroll_flush: Zeile 24 (Spiegel) in den VDC schreiben: 40 logische Zellen = 80 VDC-Zellen (Zeichen doppelt, Attribute mit/ohne Bit 7). Veraendert
;   A, X, Y.
scroll_flush:
        lda #>(VDC_SCR + 24 * 80)
        ldy #<(VDC_SCR + 24 * 80)
        jsr vdc_addr
        ldx #31
        stx VDC_ST
        ldx #0
.sf1:   lda SCR + 24 * 40,x
.sf2:   bit VDC_ST
        bpl .sf2
        sta VDC_DT
.sf3:   bit VDC_ST
        bpl .sf3
        sta VDC_DT
        inx
        cpx #40
        bne .sf1
        lda #>(VDC_ATTR + 24 * 80)
        ldy #<(VDC_ATTR + 24 * 80)
        jsr vdc_addr
        ldx #31
        stx VDC_ST
        ldx #0
.sf4:   lda COLORRAM + 24 * 40,x
.sf5:   bit VDC_ST
        bpl .sf5
        sta VDC_DT
        ora #$80
.sf6:   bit VDC_ST
        bpl .sf6
        sta VDC_DT
        inx
        cpx #40
        bne .sf4
        rts
scrollreg: .byte 0                      ; UNGENUTZT (Rest der VIC-II-Fassung: Wert fuer $D016)

; Lauftext: 40 Leerzeichen vorweg (der Text laeuft von rechts herein), dann die Beschreibung der Kapselfarben (Kapseln: siehe cap_tcol); mit 0
;   abgeschlossen, laenger als
; 256 Zeichen (darum der 16-Bit-Zeiger stq).  Kleinbuchstaben erscheinen als Grossbuchstaben (siehe Hinweis zu .screen).
scroll_text:
        .screen "                                        prallball - zerstoere alle steine mit ball und schlaeger!   fange die muenzen, die aus zerstoerten steinen fallen:  hellblau = breiter schlaeger,  hellrot = schmaler schlaeger,  rot = umgekehrte steuerung,  gruen = ball langsamer,  gelb = ball schneller,  dunkelgelb = ball klebt,  weiss = extraball,  tuerkis = bonuspunkte,  violett = mehrball,  hellgruen = laser,  grau = durchschlag.   viel spass!                "
        .byte 0

; title_balls_init: die drei Titelbaelle (Zeichen-Sprites 0..2, weiss) auf ihre Startwerte (tb_u0, tb_y0, tb_du, tb_dy) setzen,
;   Schlaeger/Muenze/Schuesse aus.
;   Position X in Einheiten zu 2 px (u, 8 Bit reicht fuer das ganze Bild), Y in px; Zeichen-Sprite-X = 2 * u, Y = y.  Veraendert A, X.
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
        lda #1
        sta sl_act,x                    ; Zeichen-Sprites 0..2: die drei Baelle
        lda #IMG_BALL
        sta sl_img,x
        lda #15                         ; weiss (VDC-Farbe)
        sta sl_col,x
        dex
        bpl .tbi
        lda #0
        sta sl_act + 3                  ; Schlaeger, Muenze, Schuesse aus
        sta sl_act + 4
        sta sl_act + 5
        sta sl_act + 6
        rts

; title_balls_tick: die Titelbaelle um (tbdu, tbdy) weiterbewegen (Umkehr bei u = 6/148 und y = 60/205), in die Schlitze 0..2 eintragen und die
;   Sprites setzen.
;   Einmal je Bild, direkt nach wait_frame.  Veraendert A, X (und alles, was tsp_render veraendert).
title_balls_tick:
        ldx #0
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
.tby:   lda tbu,x                       ; u (Einheiten zu 2 px) -> Bildpunkte
        asl
        sta sl_x,x
        lda #0
        rol
        sta sl_xh,x
        lda tby,x
        sta sl_y,x
        inx
        cpx #3
        bne .tb
        jmp tsp_render                  ; Zeichen-Sprites setzen (im Bildwechsel: der Aufrufer hat gerade wait_frame verlassen)

tb_u0: .byte 30, 80, 120
tb_y0: .byte 90, 140, 70
tb_du: .byte 1, $ff, 1
tb_dy: .byte 1, 2, $ff

t_sub:  .screen "pralle action fuer den c128"   ; UNGENUTZT (derselbe Text steht in tk_msgs)
        .byte 0
t_best: .screen "bestenliste"
        .byte 0
t_ctrl: .screen "joystick: links / rechts, feuer"
        .byte 0
t_keys: .screen "tasten: z x, leertaste"
        .byte 0
t_start: .screen "feuer = start"
        .byte 0

; draw_logo: "PRALLBALL" in einer 3 x 5 Pixel grossen Blockschrift; jeder Pixel ist ein Bildschirmzeichen (CH_BLOCK). Veraendert A, X, Y, t0, t2,
;   t3, lt, lr, lbits, lcol, txtcol.
;   Die Zeilen 3..7 des Bildschirms bekommen je eine Regenbogenfarbe (C64-Farbnummer 8 + Farbe aus logo_color, ueber vcol in VDC-Farben umgesetzt).
;   Buchstabe i beginnt in
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
        jsr put_cell
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
; new_game: alles auf Anfang: Punkte 0, Level 1 (oder STARTLEVEL), Geschwindigkeitsstufe 0, 3 Baelle, Schlaeger mittig, keine Extras.  Veraendert A.
new_game:
        lda #0
        sta sc0
        sta sc1
        sta sc2
        sta level
        sta speed
        sta hits
        sta padh
        lda #144                        ; Schlaeger mittig: 16 + (288 - 32) / 2
        sta padl
        lda #$98
        sta scrollreg                   ; (Rest der VIC-II-Fassung, ohne Wirkung)
.ifdef TESTFPS
        lda #0
        sta $c200                       ; Testzaehler der Hauptschleife (16 Bit) loeschen
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
        sta atgt                        ; Autoplay: Zielpunkt auf dem Schlaeger (8 = linke Zone)
        jmp reset_extras

; reset_extras: alle Extras aus (neues Spiel, neues Level, Ball verloren): Kapsel weg, Timer 0, Laserschuesse weg, Schlaeger 32 px. Veraendert A, X,
;   padw, zstep, pad2o.
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
        lda #32                         ; weiter mit set_padw

; set_padw: Schlaegerbreite A (16, 32 oder 48) setzen und pad2o passend einrichten.  Veraendert A.
;   update_sprites waehlt daraus das Bild des Schlaegers: 16 px = IMG_PAD16 (pad2o = 0), 32 px = IMG_PAD32 (pad2o = 16), 48 px = IMG_PAD48 (pad2o =
;   24).
;   zstep = Breite / 8 ist die Zonenbreite fuer den Abprallwinkel (8 Zonen ueber die ganze Breite).
set_padw:
        sta padw
        lsr
        lsr
        lsr
        sta zstep
        lda padw
        cmp #48
        beq .sw48
        cmp #16
        beq .sw16
        lda #16
        sta pad2o
        rts
.sw48:  lda #24
        sta pad2o
        rts
.sw16:  lda #0
        sta pad2o                       ; 0 = schmaler Schlaeger (Bild IMG_PAD16, siehe update_sprites)
        rts

; load_level: Steinkarte des Levels `level` ins RAM (MAP) kopieren, zerstoerbare Steine (bricks) und Boss-Kerne (cores) zaehlen, Bild aufbauen, Ball
;   auf den Schlaeger.
;   Veraendert A, X, Y, tp und alles, was die gerufenen Zeichenroutinen verwenden.
load_level:
        ldx level
        lda level_tab_lo,x
        sta tp
        lda level_tab_hi,x
        sta tp + 1
        jsr clear_screen
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
        lda level
        and #3
        jsr set_theme                   ; jedes Level hat eine eigene Hintergrund-Themenfarbe (4 Themen im Wechsel, siehe vtheme)
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
        jsr tsp_hide                    ; alle Zeichen-Sprites aus (update_sprites setzt sie im naechsten Bild)
        rts

; draw_walls: Oberkante (Zeile 1) und beide Seiten (Spalten 1 und 38) mit CH_WALL zeichnen; die Aussenspalten 0 und 39 werden schwarz gefuellt.
;   Veraendert A, X, Y, t1, txtcol.
draw_walls:
        lda #14                         ; C64-Farbnummer 14 = 8 + 6 (blau) -> VDC hellblau
        sta txtcol
        lda #1
        sta t1
.wt:    ldx t1
        ldy #1
        lda #CH_WALL
        jsr put_cell
        inc t1
        lda t1
        cmp #39
        bne .wt                         ; Spalten 1..38
        lda #2
        sta t1
.ws:    ldx #1
        ldy t1
        lda #CH_WALL
        jsr put_cell
        ldx #38
        ldy t1
        lda #CH_WALL
        jsr put_cell
        inc t1
        lda t1
        cmp #25
        bne .ws                         ; Zeilen 2..24
        lda #0                          ; schwarz: Spalte 0 und 39 mit vollen Bloecken (CH_SOLID = Zeichen 127) fuellen, damit die
        sta txtcol                      ; Hintergrundfarbe (Thema, Blitz) nur im Spielfeld zwischen den Waenden sichtbar ist
        lda #1
        sta t1
.wb:    ldx #0
        ldy t1
        lda #CH_SOLID
        jsr put_cell
        ldx #39
        ldy t1
        lda #CH_SOLID
        jsr put_cell
        inc t1
        lda t1
        cmp #25
        bne .wb                         ; Zeilen 1..24
        rts

; draw_map: die ganze Steinkarte MAP zeichnen (12 Zeilen x 18 Spalten).  Veraendert A, X, Y, bidx, brow, bcol und die von draw_brick benutzten Zellen.
draw_map:
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
        tay
        lda brick_color,y
        sta txtcol
        lda bcol
        asl
        clc
        adc #2
        sta dbx                         ; Bildschirmspalte
        lda brow
        clc
        adc #4
        sta dby                         ; Bildschirmzeile
        lda t0
        beq .db0
        ldx dbx
        ldy dby
        lda #CH_BRICK_L
        jsr put_cell
        ldx dbx
        inx
        ldy dby
        lda #CH_BRICK_R
        jmp put_cell
.db0:   ldx dbx                         ; leerer Platz: beide Zeichen loeschen
        ldy dby
        lda #CH_SPACE
        jsr put_cell
        ldx dbx
        inx
        ldy dby
        lda #CH_SPACE
        jmp put_cell

; draw_hud: Kopfzeile "PUNKTE 000000 LEVEL n BAELLE n" (Spalten 0, 7..12, 16, 22/23, 26, 33; im Demo-Modus nichts, die Zeile gehoert dem Ticker).
;   Veraendert A, X, Y, t0, t2, t3, tp.
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

; Farbe je Steintyp als C64-Farbnummer = 8 + Farbe (vcol macht daraus die VDC-Farbe; 8 = schwarz wird dunkelgrau): 1 rot, 2 gelb, 3 gruen, 4 tuerkis,
; 5 blau, 6 violett, 7 weiss, 8 rot; 9 Silber = weiss, 10 Gold = gelb, 11 unzerstoerbar = dunkelgrau, 12 explosiv = rot, 13 Ueberraschung = weiss;
; 14/15 unbenutzt; 16..23 = Boss-Kerne (wechselnde Farben nach Lebenspunkten).  Index 0 (leer) wird nicht gezeichnet.
brick_color: .byte 8, 10, 15, 13, 11, 14, 12, 9, 10, 9, 15, 8, 10, 9, 8, 8, 10, 12, 14, 11, 13, 15, 9, 9
; Punkte je Treffer (BCD, ein Byte): 10 * Typ fuer 1..8, Silber 50, Gold 90, unzerstoerbar 0, explosiv/Ueberraschung 10, Boss-Kern 20 (Typ 16..23)
brick_pts:   .byte 0, $10, $20, $30, $40, $50, $60, $70, $80, $50, $90, 0, $10, $10, 0, 0, $20, $20, $20, $20, $20, $20, $20, $20

; ---------------------------------------------------------------------------------------------------------------------
; Spielschleife
; ---------------------------------------------------------------------------------------------------------------------
; play: spielt bis zum Ende eines Levels oder Balls.
;   Rueckgabe A: 0 = Level geschafft, 1 = Ball verloren, 2 = Abbruch.
play:
.ifdef TESTFPS
        lda vblank
        beq .nomiss
        inc $c204                       ; Testfassung: Bildwechsel war schon da = ein Bild zu langsam (verpasst)
.nomiss:
.endif
        jsr wait_frame                  ; Bildtakt: ab hier bleibt bis zum naechsten Rasterinterrupt (20 ms) Zeit fuer den ganzen Durchlauf
.ifdef TESTFPS
        inc $c200                       ; Testfassung: Bildzaehler (16 Bit) - zeigt, ob Bilder ausgelassen werden (ruckelt)
        bne .fp1
        inc $c201
.fp1:   lda $c200
        and #15
        bne .fp2
        lda #1
        sta txtcol                      ; weiss (C64-Farbnummer 1)
        lda $c201                       ; alle 16 Bilder: Zaehler (put_bcd gibt die Nibbles als Zeichen $30 + Wert aus) in Zeile 24 anzeigen:
                                        ; Spalte 6/8 = Bilder der Hauptschleife, 11/13 = Rasterinterrupts, 16 = verpasste Bilder
        ldx #6
        ldy #24
        jsr put_bcd
        lda $c200
        ldx #8
        ldy #24
        jsr put_bcd
        lda $c203
        ldx #11
        ldy #24
        jsr put_bcd
        lda $c202
        ldx #13
        ldy #24
        jsr put_bcd
        lda $c204
        ldx #16
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
+       jsr update_sprites              ; zuerst die Zeichen-Sprites setzen (Zustand des vorigen Durchlaufs): so liegt die VDC-Aktualisierung immer
                                        ;   gleich nach dem Bildtakt
        jsr sfx_update
        jsr get_input                   ; mask = Eingabe dieses Bildes
        lda mask
        and #IN_ABORT
        beq +
        lda #2
        rts
+       jsr fx_frame                    ; Hintergrundblitz und Glanzlicht
        jsr tick_timers                 ; Extra-Zeiten herunterzaehlen
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
        jsr flash                       ; Level geschafft: Hintergrund 25 Bilder gelb
        lda #SFX_EXTRA                  ; Level geschafft
        jsr sfx_play
        ldx #60
        jsr wait_frames
        lda #0
        rts
.gone:  lda #2
        ldx #12
        jsr flash                       ; Ball verloren: roter Hintergrundblitz (12 Bilder)
        lda #SFX_LOSE
        jsr sfx_play
        ldx #50
        jsr wait_frames
        lda #1
        rts
.bplay: jmp play

; get_input: mask aus Joystick/Tastatur (input_poll); im Demo-Modus vom Rechner (auto_input).  Dann beendet jede echte Eingabe die Demo (IN_ABORT).
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

; auto_input: Der Rechner spielt (liefert mask).  Der Schlaeger soll den Ball mit dem Zielpunkt atgt (8..23 px von der linken Kante) treffen,
;   damit unterschiedliche Winkel vorkommen. Differenz = (Schlaegerkante + atgt) - (Ballmitte); >= 6 -> links, <= -7 -> rechts, dazwischen Totzone.
;   Feuer ist immer gesetzt.
;   Veraendert A, X, t0..t3 und alles, was pick_ball/get_px veraendern.
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
        bne .left                       ; >= 256: Ziel weit rechts vom Ball -> Schlaeger nach links
        cpx #6
        bcc .none                       ; |Differenz| unter 6 px: nichts tun (Totzone: der Schlaeger bewegt sich 4 px je Bild, bei einer
                                        ; Schwelle von 2 px schwingt er um den Zielpunkt hin und her und die Demo wirkt ruckelig)
.left:  lda #IN_LEFT | IN_FIRE
        bne .set
.neg:   cmp #$ff
        bne .right                      ; Ziel weit links vom Ball (<= -257) -> Schlaeger nach rechts
        cpx #$fa
        bcs .none                       ; -6 .. -1: klein (Totzone, siehe oben)
.right: lda #IN_RIGHT | IN_FIRE
        bne .set
.none:  lda #IN_FIRE
.set:   sta mask
        rts

; move_paddle: Schlaeger nach mask (IN_LEFT/IN_RIGHT) um 4 px je Bild bewegen und auf 16 .. 304 - padw begrenzen (16 Bit: padh:padl; bei 32 px also
;   16..272). Veraendert A, t0.
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

; place_ball: Ball haftet in der Schlaegermitte auf dem Schlaeger (stuck = 1); nur Ball 0 ist aktiv. Veraendert A, X, cb und die Ballwerte in der
;   Nullseite.
;   X = (padl + padw / 2 - 3) * 64 (Ballkante = Schlaegermitte - halbe Ballbreite; bei 32 px also padl + 13), Y = (184 - 6) * 64 = $2C80.
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

; pick_ball: den tiefsten aktiven Ball (groesstes High-Byte von Y) in die Nullseite laden (der Autoplay/Demo folgt ihm); cb = sein Schlitz.
;   Veraendert A, X, t2, t3.
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

; multiball: aus dem ersten aktiven Ball zwei weitere Baelle erzeugen (Zonen 1 und 6 = -43 / +43 Grad), soweit Schlitze frei sind. Veraendert A, X,
;   cb, t1, t2 und die Ballwerte.
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

; launch: Ball abschiessen, leicht schraeg (Zone 3 oder 4 = -9 / +9 Grad, zufaellig).  Hebt stuck auf.  Veraendert A, X, t0 und die Ballwerte.
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

; get_px: ganze Bildpunkte der linken Ballkante: (bxh:bxl) / 64 -> pxh:pxl (nur X; fuer Y rechnet update_sprites selbst).   Veraendert X.
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

; hit_brick: Stein vom Typ A getroffen (bidx/bcol/brow gesetzt).  Veraendert A, X, Y, t3 und was add_score/draw_brick/spawn_cap verwenden.
;   Boss-Kerne (Typ 16..23) verlieren je Treffer einen Lebenspunkt (Typ - 1) und verschwinden bei Typ 15.
;   Typ 11 (unzerstoerbar): nur Klang.  Gold (10) wird zu Silber (9), Silber zu Typ 1; alle anderen verschwinden.
;   Fuer jeden Treffer gibt es Punkte nach brick_pts.
hit_brick:                              ; (der weisse Kurzblitz der VIC-II-Fassung entfaellt: flash ignorierte auf dem VDC X < 4)
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
        jsr flash                       ; gelber Hintergrundblitz (8 Bilder) fuer den Kern
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

; draw_score: nur die Punktzahl neu zeichnen (Spalten 7..12; im Demo-Modus nichts).
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

; paddle_hit: Ball ist auf Schlaegerhoehe (178 <= Y < 192). Prueft die Ueberlappung in X und prallt ab (setzt dann Y, Geschwindigkeit, ggf. stuck,
;   hits, speed).
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
; boss_attack: der Boss laesst eine boese Kapsel fallen (S = schmaler Schlaeger oder R = umgekehrte Steuerung) aus einer zufaelligen Spalte (1..16,
;   Zeile 4).
;   Nur wenn gerade keine Kapsel faellt.  Veraendert A, bcol, brow, capforce und was spawn_cap veraendert.
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

; draw_status: Statuszeile (Bildschirmzeile 24): Bonus = verbleibende Sekunden, Boss = verbliebene Kerne; sonst leer (normales Level: nichts).
;   Veraendert A, X, Y, tp, txtcol.
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
.dso:   pha                             ; Einer merken
        txa
        clc
        adc #$30                        ; Zehnerziffer
        ldx #25
        ldy #24
        jsr put_cell
        pla
        clc
        adc #$30                        ; Einerziffer
        ldx #26
        ldy #24
        jmp put_cell
.ds0:   rts
s_bonus: .screen "bonus   "
        .byte 0
s_boss: .screen "boss: kerne "
        .byte 0

; fx_frame: Hintergrundblitz herunterzaehlen und das Glanzlicht ueber die Steine laufen lassen (je zweites Bild ein Schritt: tick ist gerade).
fx_frame:
        lda bflash
        beq .fx1
        dec bflash
        bne .fx1
        lda #0
        jsr vdc_bgrestore               ; Blitz zu Ende: Themenfarbe wieder einstellen
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
.grw:   lda bcol                        ; weiss faerben: Attribut beider Zeichen
        asl
        clc
        adc #2
        sta dbx
        lda brow
        clc
        adc #4
        sta dby
        ldx dbx
        ldy dby
        lda #15                         ; weiss (VDC-Farbe), auch in der Schattenkopie
        jsr wr_attr
        ldx dbx
        inx
        ldy dby
        lda #15
        jsr wr_attr
.gr8:   inc brow
        lda brow
        cmp #12
        bne .gr1
.gr9:   rts

; update_sprites: Zustand des Spiels in die Schlitze der Zeichen-Sprites (sl_*) uebertragen und diese zeichnen (tsp_render). Einmal je Bild, direkt
;   nach wait_frame.
;   Schlitze: 0..2 Baelle, 3 Schlaeger, 4 Muenze, 5..6 Laserschuesse.  X (16 Bit) und Y in Bildpunkten.
update_sprites:
        lda #0
        sta cb
.ub:    ldx cb                          ; Baelle 0..2 -> Schlitze 0..2
        lda bact,x
        bne .ubon
        lda #0
        sta sl_act,x                    ; Schlitz leer: Sprite aus
        beq .ubn
.ubon:  jsr load_ball
        jsr get_px                      ; pxl/pxh (bis 9 Bit)
        ldx cb
        lda pxl                         ; X in Bildpunkten (9 Bit)
        sta sl_x,x
        lda pxh
        sta sl_xh,x
        lda byl                         ; py = (Y * 4) / 256 = Y / 64
        sta t0
        lda byh
        sta t1
        asl t0
        rol t1
        asl t0
        rol t1
        lda t1
        sta sl_y,x
        lda #IMG_BALL
        sta sl_img,x
        lda #15                         ; Ballfarbe: weiss, bei Durchschlag gelb (VDC-Farben)
        ldy tpierce
        beq +
        lda #13
+       sta sl_col,x
        lda #1
        sta sl_act,x
.ubn:   inc cb
        lda cb
        cmp #3
        bne .ub
        ; --- Schlaeger (Schlitz 3): Breite ueber pad2o (0 = 16 px, 16 = 32 px, 24 = 48 px)
        lda padl
        sta sl_x + 3
        lda padh
        sta sl_xh + 3
        lda #PADY
        sta sl_y + 3
        ldx #IMG_PAD16
        lda pad2o
        beq +
        ldx #IMG_PAD32
        cmp #24
        bne +
        ldx #IMG_PAD48
+       stx sl_img + 3
        lda #3                          ; hellblau (VDC-Farbe)
        sta sl_col + 3
        lda #1
        sta sl_act + 3
        ; --- Muenze (Schlitz 4)
        lda capact
        bne +
        sta sl_act + 4
        beq .nocap
+       lda capxl                       ; Muenzenbild liegt in einem Rahmen aus 2 Zeichen (16 Bildpunkte) ab capx
        sta sl_x + 4
        lda capxh
        sta sl_xh + 4
        lda capy
        sta sl_y + 4
        lda capfr                       ; Drehphase 0..5 = capfr / 4 -> Bild aus der Folge Vorn, 3/4, halb, Kante, halb, 3/4
        lsr
        lsr
        tax
        lda cap_img,x
        bne +
        lda captype                     ; Vorderseite: Bild mit dem Buchstaben der Art
        clc
        adc #IMG_COIN_B
+       sta sl_img + 4
        ldx captype
        lda cap_tcol,x
        sta sl_col + 4
        lda #1
        sta sl_act + 4
.nocap: ; --- Laserschuesse (Schlitze 5, 6)
        ldx #0
.ss:    lda shact,x
        bne .sson
        lda #0
        sta sl_act + 5,x
        beq .ssn
.sson:  lda shu,x                       ; X-Lage in Einheiten zu 2 px -> Bildpunkte
        asl
        sta sl_x + 5,x
        lda #0
        rol
        sta sl_xh + 5,x
        lda shy,x
        sta sl_y + 5,x
        lda #IMG_SHOT
        sta sl_img + 5,x
        lda #13                         ; gelb
        sta sl_col + 5,x
        lda #1
        sta sl_act + 5,x
.ssn:   inx
        cpx #2
        bne .ss
        jmp tsp_render

; tick_timers: einmal je Bild.  Alle 25 Bilder (0,5 s) zaehlen die Extra-Timer (tpad, trev, tlas, tpierce, tspd) um 1 herunter; bei 0 endet das Extra.
;   Dazu die Bonus-Sekunden (btime) und der Boss-Angriffszaehler (batk).
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
;   Die Kapselart wird zufaellig aus cap_pick gewaehlt (oder per capforce vorgegeben); ihr Buchstabe steckt im Bild IMG_COIN_* (siehe update_sprites).
;   Veraendert A, X, capxl/capxh, capy, captype, capfr, capact.
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
        lda #0
        sta capfr
        lda #1
        sta capact
.sc9:   rts

; Kapselarten: 0 B breit, 1 S schmal, 2 R umgekehrt, 3 L langsam, 4 F schnell, 5 K klebrig, 6 + Leben, 7 P Punkte, 8 M Mehrball, 9 G Laser, 10 D
;   Durchschlag
cap_img:    .byte 0, IMG_COIN_TQ, IMG_COIN_HALF, IMG_COIN_EDGE, IMG_COIN_HALF, IMG_COIN_TQ   ; Bilder der Drehphasen (0 = Vorderseite je nach Art)
; Farbe der Muenze je Art (VDC-Attributfarbe 0..15): B hellblau, S hellrot, R dunkelrot, L dunkelgruen, F hellgelb, K dunkelgelb (braun), + weiss, P
;   hellcyan (tuerkis),
; M hellviolett, G hellgruen, D (Durchschlag) hellgrau.  Der Buchstabe steckt im Bild (IMG_COIN_*).
cap_tcol:   .byte 3, 9, 8, 4, 13, 12, 15, 7, 11, 5, 14
; Gewichtung der Kapselarten bei der Auswahl: 16 Eintraege (Zufallszahl and 15); M kommt oefter als Leben (+)
cap_pick:   .byte 0, 0, 1, 2, 3, 4, 5, 6, 7, 7, 8, 8, 8, 9, 9, 10

; laser_fire: zwei Schuesse an den Enden des Schlaegers starten (nur wenn beide Schlitze frei sind).  Veraendert A, t0, t1, shact, shu, shy.
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
        lda #SFX_LASER
        jsr sfx_play
.lf9:   rts

; update_shots: Schuesse um 4 px nach oben; trifft einer einen Stein (Karte an Schussspitze), wirkt hit_brick und der Schuss endet.
;   Spalte = (u - 8) / 8 (u = px / 2, Stein 16 px = 8 Einheiten breit, Feld beginnt bei px 16 = u 8), Zeile = (y - 32) / 8.
update_shots:
        ldx #0
.us:    stx shi                         ; Schuss 0..1; hit_brick veraendert X, darum merken wir die Schussnummer in shi (Nullseite, nicht von
                                        ;   hit_brick benutzt)
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
.us1:   cmp #32
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

; update_cap: Kapsel um 1 px fallen lassen (und die Drehphase capfr weiterzaehlen); faengt der Schlaeger sie (Ueberlappung in X und Y), wirkt sie
;   (activate).
;   Unterkante 200 px: weg.  Veraendert A, t0, t1.
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

; activate: Kapsel wirkt.  Art in captype (0..10, siehe Kommentar bei cap_img).  Dauern in Halbsekunden (tick_timers).
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
.a10:   lda #10                         ; D: Durchschlag, 5 s
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
; game_over: "SPIEL ZU ENDE"; bei einem Platz in der Bestenliste Namenseingabe, Eintragen und Speichern auf Diskette (nicht im Demo-Modus); danach 3
;   s warten.
;   Veraendert A, X, Y und alles, was die gerufenen Routinen veraendern.
game_over:
        jsr tsp_hide                    ; Ball und Schlaeger ausblenden
        jsr wait_frame
        jsr tsp_render                  ; (stellt die Zellen unter den Sprites wieder her)
        lda #0
        jsr vdc_bgrestore               ; ein laufender Hintergrundblitz endet (Themenfarbe des letzten Levels bleibt)
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
        ldx #13                         ; "neuer rekord!" blinkt (VDC-Attributbit 4), solange die Namenseingabe laeuft
.blk:   stx t1
        ldy #20
        lda vcol + 7                    ; gelb in VDC-Farbe
        ora #$10
        jsr wr_attr
        ldx t1
        inx
        cpx #26
        bne .blk
        jsr name_entry                  ; Name eingeben (Vorgabe SPIELER, Auto-Bestaetigung nach 30 s)
        pla
        jsr hs_insert                   ; Eintrag an den Platz setzen, Rest rutscht nach unten
        sei                             ; Diskettenzugriff ohne Rasterinterrupt (hs_io_on schaltet den VIC-Interrupt zusaetzlich ab und den Takt auf
                                        ;   1 MHz)
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
;   Bedienung mit dem JOYSTICK (Port 2): hoch/runter wechselt das Zeichen an der Cursorstelle (Leerzeichen, A-Z, 0-9, im Kreis), rechts/links bewegt
;   den Cursor,
;   Feuer bestaetigt.  Haelt man eine Richtung, wiederholt sie sich (nach 25 Bildern, dann alle 6).
;   Bedienung mit der TASTATUR: Buchstaben, Ziffern, Leerzeichen, - und . tippen (das erste Zeichen loescht die Vorgabe), DEL loescht, RETURN
;   bestaetigt. Die Tasten
;   Z/X/Q/A bedeuten hier Buchstaben, nicht Richtungen: darum wird nur der Joystick (input_joy) und input_char gelesen.
;   Ohne jede Eingabe wird nach 30 s (1500 Bilder bei 50 Hz) automatisch bestaetigt; jede Eingabe stellt den Zaehler zurueck. Der Name steht als
;   ASCII in hs_name (8 Byte).
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
.ne:    jsr wait_frame
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
.done:  rts

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
;   Der Cursor ist NICHT das Umkehrzeichen (Code + 128): diese Codes sind hier der Pool der Zeichen-Sprites (POOL0/NPOOL) bzw. der Ticker und ihre Bitmuster
;   sind ueberschrieben.  Stattdessen bekommt die Zelle das VDC-Attributbit 6 (Umkehrschrift).
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
+       pha
        txa
        clc
        adc #16
        tax
        pla
        ldy #22
        jsr put_cell
        ldx t1
        lda #15 | $20                   ; Namensfeld: unterstrichen (VDC-Attributbit 5), Farbe weiss
        cpx npos
        bne +
        lda #15 | $40                   ; Cursorstelle: Umkehrschrift (VDC-Attributbit 6), Farbe weiss
+       pha
        txa
        clc
        adc #16
        tax
        ldy #22
        pla
        jsr wr_attr                     ; (schreibt Spiegel, Schattenkopie und beide VDC-Zellen; put_cell setzt es beim naechsten Aufruf zurueck)
        ldx t1
        inx
        cpx #8
        bne .nr
        rts

; =====================================================================================================================
; Ticker: eine Textzeile (32 Zeichen breit, Spalten 4..35) mit wechselnden Texten und Uebergangseffekten.
;   Der Titel zeigt darin den Untertitel (Zeile 10), die Demo die Kopfzeile (Zeile 0, statt des Punktestands).
;   TECHNIK: Die 32 Bildschirmzeichen der Zeile (Spalte 4..35) zeigen die Zeichen 224..255 des Zeichensatzes; deren Bitmuster (256 Byte bei
; TK_BUF = CHARSET + 224 * 8 = $E700, je Zeichen 8 Byte, ein Byte je Pixelzeile) werden in jedem Schritt neu berechnet; tk_upload laedt nur
;   geaenderte Zeichen gedehnt in den VDC. So sind Bewegungen um einzelne BILDPUNKTE moeglich,
;   obwohl nur Zeichen im Raster sichtbar sind.  Die Bitmuster der Buchstaben holen wir aus dem ins RAM kopierten ROM-Zeichensatz (CHARSET + Code * 8).
;   EFFEKTE (zufaellig gewaehlt): 0 nach oben wegscrollen (neu von unten), 1 nach unten, 2 buchstabenweise (erst rechts nach links loeschen,
;   dann links nach rechts schreiben), 3 nach links wegscrollen (neu von rechts), 4 nach rechts, 5 Wischer (Kante laeuft pixelweise von links
;   nach rechts), 6 Aufloesen (zufaellige Pixelzeilen werden ausgetauscht).  Dazu eine laufende Regenbogenfarbe ueber die Zeichen.
;   ABLAUF: warten (tkw Bilder), naechsten Text aus der Folge holen, Effekt starten, Schritte ausfuehren, fertig -> warten.
; Zustand (Nullseite, tk*): tkrow Zeile, tkst 0 = warten / 1 = Effekt, tkw Wartebilder, tkeff Effekt, tkstep Schritt, tkmsg Folgenindex, tkseq 0
;   Titel / 1 Demo
; =====================================================================================================================
.const TK_BUF = CHARSET + 224 * 8       ; Bitmuster der Zeichen 224..255 (Hauptspeicher; tk_upload kopiert Aenderungen in den VDC)
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
        lda #7                          ; gelb
        sta txtcol
        ldx #0
.ti2:   stx dbx
        txa
        clc
        adc #TK_CHR                     ; Zeichen 224 + Spalte
        ldy tkrow
        pha
        txa
        clc
        adc #4
        tax
        pla
        jsr put_cell
        ldx dbx
        inx
        cpx #32
        bne .ti2
        ldx #0                          ; Bitmuster im VDC sind unbekannt: alles neu hochladen
.ti3:   lda TK_BUF,x
        eor #$ff
        sta tk_vdc,x
        inx
        bne .ti3
        jmp tk_upload

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

; tk_ga / tk_gb: A = Bildschirmcode -> tka / tkb = Adresse des Bitmusters im Zeichensatz (CHARSET + Code * 8).
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
        adc #>CHARSET
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
        adc #>CHARSET
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

; ---- ticker_tick: einmal je Bild (Farbwechsel, Wartezeit, Effektschritt); laedt die Bitmuster in den VDC, solange ein Effekt laeuft. Veraendert A,
;   X, Y und die tk*-Zellen.
ticker_tick:
        lda tkst
        sta tkprev
        jsr ticker_tick0
        lda tkst
        ora tkprev                      ; lief ein Effekt (jetzt oder gerade eben)? dann die Bitmuster der Ticker-Zeichen in den VDC laden
        beq tk_rts
        jmp tk_upload
tk_rts: rts
tkprev: .byte 0
tk_vdc: .fill 256, 0                    ; was der VDC von den Ticker-Zeichen kennt (zum Vergleichen)

; tk_upload: geaenderte Ticker-Zeichen (TK_BUF gegen tk_vdc) gedehnt in den VDC schreiben (links die linken, rechts die rechten 4 Pixel). Veraendert
;   A, X, Y, tku0..tku2.
tk_upload:
        lda #0
        sta tku1
.tu1:   lda tku1
        asl
        asl
        asl
        sta tku0
        tax
        ldy #8
.tu2:   lda TK_BUF,x
        cmp tk_vdc,x
        bne .tu3
        inx
        dey
        bne .tu2
        beq .tu4
.tu3:   jsr tk_char
.tu4:   inc tku1
        lda tku1
        cmp #32
        bne .tu1
        rts

; tk_char: Ticker-Zeichen tku1 (Bitmuster ab TK_BUF + tku0) in beide Zeichensaetze des VDC schreiben.
tk_char:
        lda tku1                        ; VDC-Adresse links = VDC_CHR + (TK_CHR + n) * 16
        lsr
        lsr
        lsr
        lsr
        clc
        adc #>(VDC_CHR + TK_CHR * 16)
        pha
        lda tku1
        asl
        asl
        asl
        asl
        tay
        pla
        jsr vdc_addr
        ldx #31
        stx VDC_ST
        ldx tku0
        lda #8
        sta tku2
.tc1:   lda TK_BUF,x
        sta tk_vdc,x
        tay
        lda dbl_l,y
.tc2:   bit VDC_ST
        bpl .tc2
        sta VDC_DT
        inx
        dec tku2
        bne .tc1
        lda tku1                        ; rechts: zweiter Zeichensatz
        lsr
        lsr
        lsr
        lsr
        clc
        adc #>(VDC_ALT + TK_CHR * 16)
        pha
        lda tku1
        asl
        asl
        asl
        asl
        tay
        pla
        jsr vdc_addr
        ldx #31
        stx VDC_ST
        ldx tku0
        lda #8
        sta tku2
.tc3:   lda TK_BUF,x
        tay
        lda dbl_r,y
.tc4:   bit VDC_ST
        bpl .tc4
        sta VDC_DT
        inx
        dec tku2
        bne .tc3
        rts

ticker_tick0:
        inc tkph
        lda tkph
        and #1
        bne .tkc9
        lda tkph                        ; Regenbogenfarbe ueber die 32 Zeichen, wandert alle 2 Bilder weiter (Farben < 8: normale Zeichen)
        lsr
        sta tkt
        lda #0
        sta dbx
.tkc1:  lda dbx
        clc
        adc tkt
        and #7
        tax
        lda tk_pal,x
        tax
        lda vcol,x                      ; C64-Farbnummer -> VDC-Farbe
        pha
        lda dbx
        clc
        adc #4
        tax
        ldy tkrow
        pla
        jsr wr_attr
        inc dbx
        lda dbx
        cmp #32
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
.tkn0:  jsr rand_byte
        and #7
        cmp #7
        bcs .tkn0                       ; Zufallszahl 7 verwerfen: die Effekte 0..6 sind gleich wahrscheinlich
        sta tkeff
        lda #0
        sta tkd                         ; senkrechte Effekte zaehlen ab dem ersten Schritt (tk_vert-Effekte setzen tkd in jedem Schritt neu)
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

tk_pal: .byte 1, 7, 5, 3, 6, 4, 2, 7                     ; Regenbogen aus C64-Farben 0..7 (weiss, gelb, gruen, tuerkis, blau, violett, rot, gelb)
tk_seqs: .byte 0, 1, 0, 0                                 ; Folge Titel (Texte 0, 1)
         .byte 2, 0, 1, 0                                 ; Folge Demo (Texte 2, 0, 1)
tk_seq:  .byte 0, 2                                       ; erster Text je Folge
tk_seqlen: .byte 2, 3                                     ; Laenge je Folge
; Texte, je 32 Zeichen, mittig aufgefuellt (Kleinbuchstaben = Bildschirmcodes 1..26, siehe Hinweis zu .screen)
tk_msgs:
        .screen "  pralle action fuer den c128   "
        .screen " ein demospiel von vintagecoder "
        .screen "       *** prallball ***        "

; ---------------------------------------------------------------------------------------------------------------------
; Bibliotheken und Daten
; ---------------------------------------------------------------------------------------------------------------------
.include "../prallball/lib/input.asm"
.include "../prallball/lib/rand.asm"
.include "../prallball/lib/sound.asm"
.include "lib/dirtable.inc"             ; dir_xl, dir_xh, dir_yl, dir_yh (erzeugt von tools/gen.py)
.include "lib/gfx.inc.chars"            ; gfx_chars
.include "lib/vdcfont.inc"             ; dbl_l, dbl_r: Dehnungstabellen (erzeugt von tools/gen.py)
.include "lib/sprites.inc"              ; Zeichen-Sprites: Streifen und Tabellen (erzeugt von tools/gen.py)
.include "lib/levels.inc"               ; level_count, levelN, level_tab_lo/hi
.include "lib/music.inc"                ; song: Titelmusik (erzeugt von tools/compose.py)
