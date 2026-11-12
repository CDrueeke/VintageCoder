; =====================================================================================================================
; prallball.asm - PRALLBALL fuer den Commodore Plus/4 (24 Levels, Zeichen-Sprites, Joystick/Tastatur, Bestenliste)
; =====================================================================================================================
;
; ZWECK
;   Ein Ball-und-Schlaeger-Spiel: Der Ball prallt von Waenden, Schlaeger und Steinen ab; sind alle zerstoerbaren Steine weg,
;   beginnt das naechste Level. Faellt der Ball unten heraus, geht ein Leben verloren; nach dem letzten Leben endet das Spiel.
;  Dieser Quelltext teilt keinen Code mit den Fassungen fuer
;   andere Systeme; nur die Bibliotheken unter data/sys/plus4/ (input, rand, sound, hiscore) werden eingebunden.
;   Die Spiellogik entspricht der C64-Fassung (examples/c64/prallball); neu ist die Anzeigeschicht, denn der Plus/4 hat KEINE Sprites.
;
; BEDIENUNG
;   Joystick (beide Anschluesse, links/rechts, Feuer) ODER Tastatur: Z = links, X = rechts, Leertaste = Feuer, RUN/STOP = Abbruch
;   zum Titel.  Feuer startet den Ball vom Schlaeger.  (Die Eingabe-Bibliothek meldet auch P als IN_PAUSE; das Spiel wertet
;   dieses Bit nicht aus, es gibt keine Pause.)  Namenseingabe der Bestenliste: Zeichen tippen, RETURN/Feuer bestaetigt.
;   Der Abprallwinkel haengt davon ab, wo der Ball den Schlaeger trifft (8 Zonen: -60 bis +60 Grad, Tabelle dir_xl..dir_yh).
;   Extras fallen als drehende Muenzen aus zerstoerten Steinen (Buchstaben B S R L F K + P M G D, siehe cap_pick/activate).
;
; ABLAUF (Zustandsautomat im Hauptprogramm, Marke main)
;   title      Titelbild, wartet auf Feuer (oder startet nach 30 s ohne Eingabe die Demo)    ->  new_game
;   new_game   Punkte 0, 3 Leben, Level 1                                                      ->  load_level
;   load_level Steinkarte laden und zeichnen, Ball auf den Schlaeger                           ->  play
;   play       ein Durchlauf je Bildwechsel (50 Hz PAL): Eingabe, Schlaeger, Ball, Kollisionen, Anzeige
;              Level geschafft -> naechstes Level;  Ball verloren -> Leben - 1;  Leben 0 -> game_over
;   game_over  Bestenliste pruefen, Namenseingabe, Speichern  ->  title
;   Die Zeit im Spiel richtet sich nach dem Rasterinterrupt (irq): er setzt einmal je Bild (Zeile 204) vblank, wait_frame wartet darauf.
;
; TESTHILFE (Schalter beim Bau: vcc asm prallball.asm --target plus4 --cpu 7501 -D NAME=1 -o x.prg; Voreinstellung: alles aus)
;   AUTOPLAY    der Rechner spielt selbst (Schlaeger folgt dem Ball, Feuer automatisch), ohne Soundeffekte; daraus ist der Demo-Modus entstanden
;   STARTLEVEL=n  das Spiel beginnt mit Level n (0-basiert)
;   TESTLOSE    es wird nur geschossen, der Schlaeger bleibt stehen: Ball geht verloren (Spielende, Namenseingabe pruefen)
;   TESTLASER   Laser ist von Anfang an aktiv        TESTMULTI  jede Kapsel wirkt als Mehrball (und spawn_cap teilt sofort den Ball)
;   TESTCAP     jeder zerstoerte Stein laesst eine Kapsel fallen
;   TESTFPS     Bildzaehler in Zeile 24 (siehe play) und Zaehler der Bildwechsel im Interrupt; zeigt, ob Bilder verpasst werden
;   NOTICKER    der Ticker wird im Demo-Modus nicht aufgerufen
;
; BILDSCHIRM (TED)
;   Mehrfarb-Textmodus 40 x 25 mit eigenem Zeichensatz im RAM ($FF07 Bit 4, siehe ted_init).  Ball, Schlaeger, Muenze und Laserschuesse
;   sind ZEICHEN-SPRITES: Zellen unter den Sprites werden mit Pool-Zeichen belegt, deren Bitmuster aus Hintergrund und Spritebild
;   zusammengesetzt wird (Abschnitt "ZEICHEN-SPRITES").  Der Hintergrund (Steine, Waende, Text) liegt dazu zusaetzlich in
;   Schattenkopien SHC/SHF.
;   Hintergrundbaender: ein Rasterinterrupt setzt die Hintergrundfarbe $FF15 22 mal je Bild um.  Farben: TED-Farbwert = Helligkeit * 16 +
;   Farbton; die Spielcode-Tabellen enthalten C64-Farbnummern 0..15, die tcol (Zellen) und tedbg (Hintergrund) umsetzen.
;   Benutzte TED-Register: $FF06/$FF07 (Modus, Versatz), $FF09/$FF0A/$FF0B (Rasterinterrupt), $FF12/$FF13/$FF14 (Zeichensatz und
;   Bildschirmadressen), $FF15-$FF17 (Hintergrundfarben 0-2; $FF16/$FF17 sind die Kantenfarben der Mehrfarbzeichen), $FF19 (Rand),
;   $FF1D (Rasterzeile lesen), $FF3E/$FF3F (ROM ein/aus); dazu Klang $FF0E-$FF11 (in sound.asm) und Tastatur/Joystick $FF08/$FD30 (input.asm).
;
; SPEICHERPLAN (der Plus/4 hat 64 KB RAM; nach dem Start ist das ROM aus, nur die Bestenliste blendet es kurz ein)
;   $0800-$0BFF Farb-RAM   $0C00-$0FFF Bildschirm   $1001-... BASIC-Starter ("10 SYS 4109"), Programmcode, Tabellen, Bibliotheken,
;   Zeichen-Sprite-Streifen, Levels (24 * 216 Byte), Lied; das Ganze endet heute bei rund $6C40 (die .vctest-Pruefung verlangt: Puffer
;   der Bestenliste unter $8000, Programm und Lied unter $C000).  Alles, was das KERNAL bei der Bestenliste anfasst oder ausfuehrt, liegt
;   unter $8000, weil dort das RAM auch bei eingeblendetem ROM sichtbar bleibt.
;   $C000-$C0D7 Steinkarte (MAP)   $C100-$C11A Ballspeicher   $C120-$C125 Laserschuesse   $C130-$C13B Titel-Baelle   $C140-$C17F Ticker-Texte
;   ($C200-$C204 Zaehler der Testfassung TESTFPS)   $C400 Schattenzeichen (SHC)   $C800 Schattenfarben (SHF)   $CC00-$CCC1 Sprite-Schlitze
;   $F000-$F7FF Zeichensatz (aus dem ROM kopiert; 64-67 und 127 eigene Zeichen, 128..191 Pool der Zeichen-Sprites, 224..255 Ticker)
;   Der TED liest den Zeichensatz ($F000) und den Bildschirm direkt aus dem RAM; die Ein-/Ausgabebereiche $FD00-$FEFF und $FF00-$FF3F
;   bleiben immer sichtbar.
;
; NULLSEITE (Zeropage)
;   $02-$03 Zufall (rand.asm)   $04-$07 Musik (sound.asm)   $08-$0D Bestenliste (hiscore.asm)   $10-$8B Spielzustand (siehe .const unten,
;   nicht alle Adressen dazwischen sind belegt)   $8C-$8F Eingabe (input.asm)   $90-$AC Zeichen-Sprites   ($AE wird einmal gelesen: KERNAL-
;   Zelle FA = Geraet des Ladens).  Der KERNAL nutzt $90 und aufwaerts nur waehrend der Bestenliste; danach ist alles neu aufgebaut.
;
; KOORDINATEN UND EINHEITEN (wichtig fuer das Verstaendnis der Physik!)
;   Bildpunkte (px) zaehlen ab der linken oberen Ecke des TEXTBILDSCHIRMS (Zeichen 0/0 = px 0/0).  Ein Zeichen ist 8 x 8 px.
;   Spielfeld: x = 16..303 (Spalten 2..37), y = 16.. (Zeile 2 abwaerts); Waende in Spalte 1 und 38 sowie Zeile 1.
;   Steinfeld: 18 Steine je 16 px Breite (2 Zeichen) ab x = 16, 12 Zeilen je 8 px Hoehe ab y = 32 (Bildschirmzeile 4).
;   Ball und Schlaeger-Positionen: BALL in "Festkomma 1/64": Wert = Bildpunkte * 64 (16 Bit).  So reichen 16 Bit fuer x bis 1023 px
;   und die Geschwindigkeit kann in 1/64 px je Bild feiner als 1 px sein.  Die obere Haelfte (High-Byte) = Bildpunkte / 4.
;   Der TED zeigt waagerecht 160 Mehrfarbpixel (je 2 Bildpunkte): Zeichen-Sprites werden auf 2 Bildpunkte genau gesetzt (x / 2).
;   Zeiten: ein Bild = 20 ms (50 Hz PAL); Extra-Timer zaehlen in Halbsekunden (25 Bilder).
;
; BEKANNTE GRENZEN
;   Nur PAL ausgelegt (Rasterzeilen von Baendern und Bildwechsel, Frequenztabelle der Musik; NTSC ungeprueft); 64 KB noetig (kein C16/116).  Hoechstens 3 Baelle,
;   2 Laserschuesse, 1 Kapsel.  In Zeile 24 und darunter gibt es keine Zeichen-Sprites.  Pool: 64 Zeichen, was darueber hinausgeht, wird nicht gezeichnet.
; =====================================================================================================================
.target plus4
.cpu 7501 { illegals = deny }
.branchlong auto                        ; zu weite bedingte Spruenge verlaengert der Assembler selbst (Warnung W203)

; ---------------------------------------------------------------------------------------------------------------------
; Nullseiten-Variablen des Spiels ($10-$8B, mit Luecken; die Adressen sind nach und nach dazugekommen und nicht nach Themen geordnet)
;   Kurzbezeichnungen: b = Ball, pad = Schlaeger, cap = Kapsel (Extra-Muenze), t... = Rest-Halbsekunden eines Extras, tk = Ticker,
;   n... = Namenseingabe, l... = Titellogo.  t0..t3 sind kurzlebige Hilfszellen, die jede Routine fuer sich benutzt (nicht ueber Aufrufe retten!).
; ---------------------------------------------------------------------------------------------------------------------
.const bxl     = $10                    ; Ball-X, Festkomma 1/64 px, Low-Byte
.const bxh     = $11                    ;                           High-Byte
.const byl     = $12                    ; Ball-Y, Festkomma 1/64 px
.const byh     = $13
.const vxl     = $14                    ; Ball-Geschwindigkeit X (vorzeichenbehaftet, 1/64 px je Bild)
.const vxh     = $15
.const vyl     = $16                    ; Ball-Geschwindigkeit Y (negativ = nach oben)
.const vyh     = $17
.const padl    = $18                    ; Schlaeger: linke Kante in px (16 Bit, 16 .. 304 - Breite, siehe move_paddle)
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
.const mask    = $23                    ; Eingabemaske dieses Bildes (IN_*, siehe input.asm; nach get_input mit evtl. vertauschter Richtung)
.const t0      = $24                    ; Hilfsspeicher (kurzlebig, jede Routine beschreibt ihren Gebrauch)
.const t1      = $25
.const t2      = $26
.const t3      = $27
.const bcol    = $28                    ; zuletzt getroffener Stein: Spalte 0..17
.const brow    = $29                    ;                             Zeile 0..11
.const bidx    = $2a                    ;                             Index in die Steinkarte (Zeile * 18 + Spalte)
.const pxl     = $2b                    ; Ball-Position in ganzen px (aus get_px), Low-Byte
.const pxh     = $2c                    ;                                          High-Byte
.const sp      = $2d                    ; Zeiger auf Bildschirm-Ziel (2 Byte, fuer (sp),y; von set_rowptr/tsp_* gesetzt)
.const cp      = $2f                    ; Zeiger auf Farb-RAM-Ziel (2 Byte)
.const tp      = $31                    ; Zeiger auf Text (2 Byte)
.const txtcol  = $33                    ; Textfarbe fuer put_cell/puts
.const atgt    = $34                    ; Autoplay: Zielpunkt auf dem Schlaeger (8..23 px von der linken Kante)
.const ptA     = $35                    ; (unbenutzt)
.const ptB     = $36                    ; probe: Y-Versatz der Abtastung (0/1), weil das Y-Register fuer den X-Versatz gebraucht wird
.const oxl     = $37                    ; Ball-X vor der Bewegung (zum Zuruecksetzen beim Abprall; oyl/oyh: ebenso Y, von paddle_hit gelesen)
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
.const pad2o   = $47                    ; Breitenkennung des Schlaegers: 0 = 16 px, 16 = 32 px, 24 = 48 px (Rest der C64-Fassung, dort Abstand des 2. Sprites);
                                        ; update_sprites waehlt danach IMG_PAD16/32/48
.const capact  = $48                    ; 1 = eine Kapsel faellt
.const capxl   = $49                    ; Kapsel: linke Kante in px (16 Bit)
.const capxh   = $4a
.const capy    = $4b                    ;          obere Kante in px
.const captype = $4c                    ;          Art 0..10 (siehe Kapselarten bei cap_img)
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
.const sprmsb  = $5c                    ; (unbenutzt; Rest der C64-Fassung)
.const sprena  = $5d                    ; (unbenutzt; Rest der C64-Fassung)
; Achtung: die vier folgenden Namen sind KEINE Nullseite, sondern RAM-Zellen bei $C130..$C13B (je 3 Byte, ein Byte je Titel-Ball)
.const tbu     = $c130                  ; Titel-Baelle (3): X in Einheiten zu 2 px
.const tby     = $c133                  ;                    Y in px
.const tbdu    = $c136                  ;                    Schritt X (1 oder $FF)
.const tbdy    = $c139                  ;                    Schritt Y (1, 2 oder negativ)
.const bp      = $61                    ; Zeiger auf die Farbtabelle des aktuellen Hintergrund-Verlaufs (2 Byte, im Interrupt benutzt)
.const bflash  = $63                    ; Rest-Bilder des Randblitzens (0 = kein Blitz)
.const glg     = $64                    ; Glanzlicht: Position der laufenden Diagonale (0..30, danach Pause bis 99)
.const scx     = $65                    ; Lauftext: Feinversatz 7..0 (Bildpunkte)
.const stp     = $66                    ; (unbenutzt; ersetzt durch den 16-Bit-Zeiger stq)
.const lph     = $67                    ; Titellogo: Farbphase
.const lcnt    = $68                    ; Titellogo: Bildzaehler fuer den Farbwechsel (alle 4 Bilder eine Phase)
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
.const tkc     = $80                    ; Ticker-Effekte: laufendes Zeichen 0..31 (auch Start eines Viertels in tk_vert)
.const tkr     = $81                    ;   laufende Pixelzeile 0..7
.const tkd     = $82                    ;   senkrechter Versatz d (tk_vert, vorzeichenbehaftet)
.const tkt     = $83                    ;   Hilfszelle (Zielindex im Puffer, Schiebebyte, Schrittzaehler, je nach Effekt)
.const tks     = $84                    ;   Hilfszelle (Quellzeile s = r + d; neue Pixel; LFSR-Index)
.const tklf    = $85                    ;   Schieberegister (LFSR, Galois, Polynom $B8) des Effekts Aufloesen
.const tkdiv   = $86                    ;   Teilschritt innerhalb eines Effektschritts (Viertel bzw. Haelfte), siehe ticker_tick
.const tkd2    = $87                    ;   Hilfszelle von tk_hright
.const tkce    = $89                    ; tk_vert: Endzeichen (exklusiv) dieses Durchlaufs
.const tkre    = $8a                    ; tk_hleft/tk_hright: Endzeile (exklusiv) dieses Durchlaufs
.const tkrs    = $8b                    ; Startzeile
.const wfc     = $60                    ; wait_frames: Zaehler
.const shi     = $5f                    ; update_shots: Schussnummer (ueberlebt hit_brick)
.const tlas    = $5e                    ; Rest-Halbsekunden: Laser aktiv
.const dxo     = $3b                    ; Abtastversatz X der Ballkante (0 = links, 1 = rechts); in move_ball (Y-Bewegung) als Y-Versatz 0/1

.const MAP     = $c000                  ; Steinkarte im RAM: 12 Zeilen x 18 Spalten = 216 Byte, Index = Zeile * 18 + Spalte, Wert = Steintyp (brick_color)
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
.const SCR      = $0c00                 ; Bildschirmspeicher (40 x 25 Zeichencodes); $FF14 = $08 legt ihn fest, das Farb-RAM liegt $0400 davor
.const COLORRAM = $0800                 ; Farb-RAM (Farbwert je Zelle, mit Helligkeit)
.const CHARSET  = $f000                 ; Zeichensatz im RAM (256 Zeichen * 8 Byte = 2 KB; der TED liest RAM auch unter dem ROM)
.const SHC      = $c400                 ; Schattenkopie der Zeichencodes (1000 Byte, Aufbau wie der Bildschirm): der Hintergrund ohne Zeichen-Sprites
.const SHF      = $c800                 ; Schattenkopie der Zellfarben (TED-Farbwerte)
.const PADY    = 184                    ; obere Kante des Schlaegers in px (Bildschirmzeile 23)
.const PADW    = 32                     ; (unbenutzt) normale Schlaegerbreite in px; wirksam ist die Variable padw (16/32/48, set_padw)
.const LIVES0  = 3                      ; Anzahl Baelle zu Spielbeginn

; Zeichencodes (Bildschirmcode) der eigenen Zeichen
.const CH_BRICK_L = 64
.const CH_BRICK_R = 65
.const CH_WALL    = 66
.const CH_BLOCK   = 67                    ; glaenzender Block fuer das Titellogo
.const CH_SOLID   = 127                   ; voller Block (Pixelmuster %11 / alle Punkte in Zellfarbe) fuer die schwarzen Randspalten 0 und 39
.const CH_SPACE   = 32
.const POOL0      = 128                   ; erstes Zeichen des Pools der Zeichen-Sprites (Codes 128..191; 224..255 gehoeren dem Ticker)
.const NPOOL      = 64                    ; Anzahl Pool-Zeichen

; Zeichen-Sprites (Compositor): Nullseite $90-$AC (der KERNAL benutzt $90 und aufwaerts nur waehrend der Bestenliste, danach ist alles neu)
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
.const cs      = $a2                    ; Feinlage 0..3 (Mehrfarbpixel innerhalb der Zeichenspalte)
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

; Zeichen-Sprite-Schlitze (RAM): 0..2 Baelle, 3 Schlaeger, 4 Muenze, 5..6 Laserschuesse.  Je Schlitz: aktiv, X (Mehrfarbpixel 0..159), Y (Pixel),
; Bildnummer (IMG_*), Zellfarbe.  Die Wiedergabe erfolgt in tsp_render (einmal je Bild, im Bildwechsel).
.const sl_act  = $cc00
.const sl_x    = $cc08
.const sl_y    = $cc10
.const sl_img  = $cc18
.const sl_col  = $cc20
.const dl_lo   = $cc40                  ; Liste der belegten Pool-Zellen (Adresse im Bildschirmspeicher), Index = Pool-Nummer
.const dl_hi   = $cc80
.const pool_n  = $ccc0                  ; Anzahl bisher vergebener Pool-Zeichen in diesem Bild

; Vorab: Bibliotheken (Hardware-Konstanten, Eingabe, Zufall, Klang, Bestenliste) aus data/sys/plus4/
#include <plus4/kernal.asm>     ; (enthaelt hardware.asm und eine Konstante SCREEN; unsere Bildschirmadresse heisst deshalb SCR)

.org $1001                              ; Start des BASIC-Speichers beim Plus/4 (Ladeadresse der .prg-Datei)
.basicstub start                        ; erzeugt "10 SYS <start>" (hier SYS 4109 = $100D), damit RUN das Programm startet

; ---------------------------------------------------------------------------------------------------------------------
; Programmstart
;   Der Plus/4 laeuft nach dem Start OHNE Betriebssystem: das ROM wird abgeschaltet (alles RAM sichtbar), Interruptvektor $FFFE im RAM.
;   Nur fuer die Bestenliste wird das KERNAL-ROM kurz eingeblendet (hs_io_on / hs_io_off).
;   Reihenfolge: Interrupts aus -> Zeichensatz kopieren (braucht noch das ROM!) -> TED, Klang, Eingabe einrichten ->
;   Zufall mischen -> Bestenliste laden -> Vektoren setzen -> Rasterinterrupt freigeben.  Danach laeuft alles ueber die Hauptschleife.
; ---------------------------------------------------------------------------------------------------------------------
start:  sei                             ; Interrupts aus: wir brauchen weder die KERNAL-Uhr noch deren Tastaturabfrage
                                        ; (Das Ladegeraet steht in der KERNAL-Zelle FA = $AE; hs_device liest sie bei Bedarf selbst, siehe hiscore.asm)
        lda #0
        sta TED_IRQ_ENABLE              ; alle TED-Interruptquellen aus
        jsr copy_font                   ; Zeichensatz aus dem ROM ins RAM kopieren, eigene Zeichen einsetzen
        jsr ted_init                    ; TED einrichten
        jsr sfx_init                    ; TED-Klang: Lautstaerke 8, beide Stimmen still
        jsr input_init                  ; Eingabezustand loeschen
.ifdef AUTOPLAY
        lda #1
        sta sfx_mute                    ; Demo-Modus: keine Soundeffekte (Vorgabe: nur die Musik spielt weiter)
.endif
        lda TED_RASTER_LO               ; die aktuelle Rasterzeile ist als Startwert fuer den Zufall gut genug (rand_seed mischt sie noch einmal ein)
        jsr rand_seed
        jsr hs_load                     ; Bestenliste von der Diskette (KERNAL wird kurz eingeblendet); bei Fehlern bleibt die Standardliste
        lda #<irq                       ; Interruptvektoren im RAM (das ROM ist jetzt aus): $FFFE = IRQ/BRK, $FFFA = NMI, $FFFC = RESET
        sta $fffe
        lda #>irq
        sta $ffff
        lda #<rti_only
        sta $fffa
        sta $fffc
        lda #>rti_only
        sta $fffb
        sta $fffd
        lda TED_IRQ_FLAGS
        sta TED_IRQ_FLAGS               ; alte Anforderungen loeschen
        lda #2
        sta TED_IRQ_ENABLE              ; Rasterinterrupt freigeben (Bit 0 = 0: Rasterzeile hat nur 8 Bit, alle Zeilen < 256)
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

rti_only: rti                           ; Ziel der Vektoren $FFFA (NMI) und $FFFC (RESET), solange das ROM aus ist: tut nichts

; ---------------------------------------------------------------------------------------------------------------------
; copy_font: kopiert den Zeichensatz (ROM, 2 KB) nach CHARSET ($F000) und setzt die eigenen Zeichen 64..67 und 127 ein.
;   Das Zeichen-ROM liegt bei $D000-$D7FF (im KERNAL-ROM-Bild), ist fuer den Prozessor aber nur sichtbar, solange das ROM eingeblendet ist ($FF3E).
;   Die Zeichen 0..127 sind der Grossschrift-/Grafiksatz (Bildschirmcodes wie beim C64: 1..26 = A..Z, 48..57 = Ziffern, 32 = Leerzeichen).
;   Was das ROM in 128..255 enthaelt, wird hier nicht benutzt: die Plaetze 128..191 gehoeren dem Pool der Zeichen-Sprites, 224..255 dem Ticker
;   (Achtung: name_draw setzt fuer die Cursorstelle Bit 7 des Bildschirmcodes; das waehlt die Zeichen 128.., nicht automatisch eine Umkehrschrift).
;   Eigene Zeichen (aus lib/gfx.inc.chars, alle MULTICOLOR): 64 = Stein links, 65 = Stein rechts, 66 = Wand, 67 = Block (Logo), 127 = voller Block.
; Veraendert A, X, Y, sp, cp.  Muss mit gesetztem I-Flag laufen (das ROM ist kurz sichtbar).  Danach ist das ROM aus ($FF3F).
; ---------------------------------------------------------------------------------------------------------------------
copy_font:
        sta TED_ROM_ON                  ; ROM einblenden (Schreiben genuegt)
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
        bne .cf                         ; eine Seite
        inc sp + 1                      ; naechste Seite quelle und ziel
        inc cp + 1
        dex
        bne .cf
        sta TED_RAM_ON                  ; ROM wieder aus
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
; ted_init: Grafikchip einrichten.
;   $FF06: Bit 4 Bild an, Bit 3 25 Zeilen, Bits 0-2 senkrechter Versatz 3 (Standard).   $FF07: Bit 7 = 1 (KEINE automatische Umkehrung, alle
;   256 Zeichencodes frei waehlbar), Bit 4 Mehrfarbtext, Bit 3 40 Spalten, Bits 0-2 waagerechter Versatz (Lauftext).
;   $FF12 Bit 2 = 0: Zeichensatz im RAM (die uebrigen Bits gehoeren dem Klang von Stimme 1 und bleiben).
;   $FF13: Bits 2-7 = Zeichensatzadresse / 1 KB ($F000 -> $3C), Bit 1 = schneller Takt ausserhalb des Bildes.
;   $FF14: Bits 3-7 = Farb-RAM-Adresse (der Bildschirmspeicher folgt 1 KB dahinter): $08 -> Farb-RAM $0800, Bildschirm $0C00.
;   Mehrfarbtext: Pixelpaar 00 = Hintergrund ($FF15), 01 = $FF16 (hell: Lichtkante), 10 = $FF17 (dunkel: Schattenkante), 11 = Zellfarbe;
;   mehrfarbig ist eine Zelle nur, wenn im Farb-RAM-Byte Bit 3 gesetzt ist (Farbton dann 0..7).
; ---------------------------------------------------------------------------------------------------------------------
ted_init:
        lda #$98                        ; %1001 1000: Bit 7 keine Umkehrung, Bit 4 Mehrfarbtext, Bit 3 40 Spalten, X-Versatz 0 (Bit 6 = 0: laut
        sta TED_CTRL2                   ; ted.chip.json PAL/NTSC-Bit; seine Wirkung ist hier nicht geprueft)
        lda #$1b                        ; %0001 1011: Bild an (Bit 4), 25 Zeilen (Bit 3), Y-Versatz 3, Rasterzeilenbit 8 = 0
        sta TED_CTRL1
        lda #$08                        ; Farb-RAM $0800, Bildschirm darauf folgend $0C00 (Standardlage des Plus/4)
        sta TED_SCREEN_BASE
        lda TED_BITMAP_BASE
        and #$fb                        ; Bit 2 loeschen: Zeichensatz aus dem RAM; die uebrigen Bits (Stimme 1 hoch, Bitmapadresse) bleiben
        sta TED_BITMAP_BASE
        lda #$f2                        ; %1111 0010: Zeichensatz bei $F000; Bit 1 gesetzt (hardware.asm: Schnellmodus; ted.chip.json nennt Bit 1
        sta TED_CHARSET_BASE            ; "ROM-Status" - die Wirkung dieses Bits ist hier nicht geprueft)
        lda #$61
        sta TED_COLOR1                  ; Mehrfarbe 1 ($FF16): Weiss, Helligkeit 6 = Lichtkante der Steine
        lda #$31
        sta TED_COLOR2                  ; Mehrfarbe 2 ($FF17): Weiss, Helligkeit 3 = Schattenkante (dunkelgrau)
        lda #0
        sta TED_BORDERCOLOR             ; Rand schwarz
        sta TED_BGCOLOR                 ; Hintergrund schwarz (die Baender setzen ihn danach im Interrupt)
        jsr tsp_init
        lda #0
        jsr set_theme                   ; Verlauf 0
        lda bar_line
        sta TED_RASTER                  ; erste Interruptzeile ($FF0B, Bit 0-7; Bit 8 ist ueber $FF0A = 2 auf 0)
        lda #0
        sta band                        ; naechstes Band = 0
        sta vblank
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Rasterinterrupt: Hintergrund-Farbbaender (Sonnenuntergang hinter dem Spielfeld) und Bildtakt.
;   Der Interrupt feuert an den Zeilen bar_line[0..22]; jedes Mal wird die Hintergrundfarbe $FF15 auf die naechste Bandfarbe gesetzt
;   (sie wirkt bei allen Pixeln mit Bitmuster 00: Luecken zwischen den Steinen und der leere Spielraum).  Beim letzten Interrupt
;   (Zeile 204, unterhalb des sichtbaren Bildes) kommt die Farbe wieder auf Schwarz fuer die Kopfzeile des naechsten Bildes
;   und vblank wird gesetzt: das Hauptprogramm darf das naechste Bild berechnen (wait_frame).
;   Die Bandfarben der Tabellen sind C64-Farbnummern 0..15; tedbg setzt sie in TED-Farbwerte (Farbton + 16 * Helligkeit) um.
;   Quittieren: Schreiben der gelesenen Bits nach $FF09 loescht die Anforderung.  Weil das ROM aus ist, rettet der Interrupt die Register selbst.
;   Zustand: band (0..22, naechstes Band), bp (Farbtabelle des Levels), scrollreg (Wert fuer $FF07), vblank (Meldung an wait_frame).
;   Schreibt: $FF09, $FF15, im Band 21 auch $FF07, bei Bildende $FF07 = $98, und immer $FF0B (naechste Zeile).  Der Interrupt
;   wartet aktiv bis zum Beginn der naechsten Rasterzeile (Schleife .iw); danach folgt der Farbwechsel.
; ---------------------------------------------------------------------------------------------------------------------
irq:    pha
        txa
        pha
        tya
        pha
        lda TED_IRQ_FLAGS
        sta TED_IRQ_FLAGS               ; Anforderung quittieren (nur die gelesenen Bits zurueckschreiben)
        ldy band
        cpy #22
        beq .ifin                       ; Band 22 = Bildende (Zeile 204): kein Farbwechsel mehr
        lda (bp),y                      ; Farbe dieses Bandes aus der Verlaufstabelle des Levels (C64-Farbnummer) ...
        tax
        lda tedbg,x
        tax                             ; ... als TED-Farbwert in X (vorab umgesetzt: nach dem Warten zaehlt jeder Takt)
        lda bar_line,y
        tay                             ; Y = Zeile T des Bandes, in der der Interrupt ausgeloest wurde
.iw:    cpy TED_RASTER_LO               ; STABILE FARBKANTE: nicht sofort schreiben (der Interrupt kommt je nach laufendem Befehl um bis zu
        bcs .iw                         ; 7 Takte versetzt), sondern warten, bis die Zeile T+1 beginnt (Schleife: solange Y >= Zeile).
                                        ; Dann liegt der Schreibzugriff in den ersten Takten der Zeile, noch im Randbereich links: die Kante
                                        ; zwischen zwei Baendern ist eine saubere Waagerechte.  Darum steht zwischen Schleife und Schreiben nur ein Befehl.
        stx TED_BGCOLOR
        ldy band
        cpy #21
        bne .i4
        lda scrollreg                   ; letztes Band: Feinversatz des Lauftextes (Zeile 24), $FF07 = $98 | Versatz.  Hinweis: Band 21 liegt bei
        sta TED_CTRL2                   ; Rasterzeile 191, der Schreibzugriff trifft also Zeile 192 (noch Textzeile 23, wenn Textzeile 0 bei Zeile 4 beginnt)
        jmp .i4
.ifin:  lda #$98                        ; Bildende: Versatz wieder 0 (Mehrfarbtext, 40 Spalten), Hintergrund schwarz, Bildtakt
        sta TED_CTRL2
        lda (bp),y                      ; Y = 22: Index 22 der Verlaufstabelle = Schwarz
        tax
        lda tedbg,x
        sta TED_BGCOLOR
        lda #1
        sta vblank                      ; Bildwechsel melden (wait_frame wartet darauf)
.ifdef TESTFPS
        inc $c202                       ; Testfassung: Bildwechsel (16 Bit, $C202/$C203) zaehlen
        bne .i4
        inc $c203
.endif
.i4:    iny                             ; naechstes Band; nach Band 22 wieder von vorn
        cpy #23
        bcc +
        ldy #0
+       sty band
        lda bar_line,y
        sta TED_RASTER                  ; naechste Interruptzeile
        pla
        tay
        pla
        tax
        pla
        rti

; Interruptzeilen (TED-Rasterzeilen; die Textzeile 0 beginnt in Zeile 4, jede Textzeile ist 8 Zeilen hoch; C64-Zeile - 47):
; 22 Baender alle 8 Zeilen von 23 bis 191 und 204 (Bildende).
bar_line: .byte 23, 31, 39, 47, 55, 63, 71, 79, 87, 95, 103, 111, 119, 127, 135, 143, 151, 159, 167, 175, 183, 191, 204
; Verlaufstabellen (je 23 Farben als C64-Farbnummer, Index 21 = letztes Band, Index 22 = Schwarz fuer die Kopfzeile des naechsten Bildes):
;   0 Daemmerung (blau - violett - rot), 1 Wald (grau - gruen - braun), 2 Glut (braun - rot - orange), 3 Nacht (graue Streifen)
bar_themes:
        .byte 0, 6, 6, 6, 6, 6, 6, 6, 6, 6, 4, 4, 4, 4, 4, 2, 2, 2, 2, 2, 2, 0, 0
        .byte 0, 11, 11, 11, 5, 5, 5, 5, 5, 5, 5, 9, 9, 9, 9, 9, 9, 9, 0, 0, 0, 0, 0
        .byte 0, 9, 9, 9, 2, 2, 2, 2, 2, 2, 8, 8, 8, 8, 8, 8, 2, 2, 2, 2, 2, 0, 0
        .byte 0, 11, 11, 0, 0, 11, 11, 0, 0, 11, 11, 0, 0, 11, 11, 0, 0, 11, 11, 0, 0, 0, 0
scrollreg: .byte $98                    ; Wert fuer $FF07 im letzten Band: $98 + Feinversatz 0..7 (title_scroll_tick); im Spiel immer $98

; Farbumsetzung C64-Farbnummer -> TED-Farbwert.
;   tedbg: Hintergrund- und Randfarben (dunkel gehalten, damit Steine und Zeichen davor leuchten).  Werte: Helligkeit * 16 + Farbton; die
;          Farbtoene 8..15 des TED sind orange, braun, gelbgruen, rosa, blaugruen, hellblau, dunkelblau, hellgruen.
;   tcol:  Zellfarben im Farb-RAM.  Index 0..7 = C64-Farbnummer 0..7: normaler Text (Hires-Zelle, Bit 3 = 0).  Index 8..15 = 8 + Farbe 0..7:
;          Mehrfarbzelle (Bit 3 gesetzt, damit nur die Farbtoene 0..7).  Die Zeichenfarbe %11 im Mehrfarbmodus kommt aus dem Farb-RAM.
;          Beispiel: tcol[10] = $5A = 8 + 2 (rot), Helligkeit 5.  Die C64-Namen 8..15 (siehe unten) gelten nur fuer tedbg, nicht fuer tcol.
;            C64:  0=schwarz 1=weiss 2=rot 3=tuerkis 4=violett 5=gruen 6=blau 7=gelb 8=orange 9=braun 10=hellrot 11=dunkelgrau 12=grau
;                  13=hellgruen 14=hellblau 15=hellgrau
tedbg:  .byte $00, $71, $32, $43, $34, $25, $26, $47, $38, $29, $4b, $21, $31, $4f, $4d, $61
tcol:   .byte $00, $71, $52, $63, $64, $55, $46, $77, $08, $79, $5a, $6b, $6c, $5d, $4e, $7f

band:     .byte 0                       ; Nummer des naechsten Interruptbandes 0..22 (siehe irq)
vblank:   .byte 0                       ; 1 = Bildwechsel erreicht (vom irq gesetzt, von wait_frame geloescht)

; flash: Rand kurz in Farbe A (C64-Farbnummer) faerben fuer X Bilder (fx_frame zaehlt herunter und stellt danach Schwarz her).
;   Eingabe: A = Farbnummer 0..15, X = Anzahl Bilder.  Veraendert A, Y.
flash:  tay
        lda tedbg,y
        sta TED_BORDERCOLOR
        stx bflash
        rts

; set_theme: Verlauf A (0..3) waehlen: bp = bar_themes + 23 * A.   Veraendert A, X.
set_theme:
        tax
        lda #<bar_themes
        sta bp
        lda #>bar_themes
        sta bp + 1
        cpx #0
        beq .st9
.st1:   clc
        lda bp
        adc #23
        sta bp
        bcc +
        inc bp + 1
+       dex
        bne .st1
.st9:   rts

; ---------------------------------------------------------------------------------------------------------------------
; Bildschirm-Hilfen
;   WICHTIG (Zeichen-Sprites): Alles, was zum HINTERGRUND gehoert (Waende, Steine, Texte), wird nicht nur auf den Bildschirm, sondern auch in
;   die Schattenkopien SHC (Zeichencodes) und SHF (Zellfarben) geschrieben.  Der Compositor (tsp_render) legt die Zeichen-Sprites ueber den
;   Bildschirm und stellt vor jedem Bild die Zellen aus den Schattenkopien wieder her.
; ---------------------------------------------------------------------------------------------------------------------
; clear_screen: 1000 Zeichen Bildschirmspeicher = Leerzeichen, Farb-RAM = weiss; ebenso die Schattenkopien.  Danach gibt es keine Pool-Zellen mehr.
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
        lda #$71                        ; weiss (TED-Farbwert)
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
        sta pool_n                      ; (die Pool-Zellen sind mit dem Bildschirm verschwunden)
        rts

; set_rowptr: sp (Bildschirm), cp (Farb-RAM), shp (Schattencodes) und shf (Schattenfarben) auf Zeile Y, Spalte X setzen.
;   Adresse = $0C00 + Zeile * 40 + Spalte; alle vier Bereiche haben denselben Aufbau und unterscheiden sich nur im oberen Adressbyte.
; Eingabe: X = Spalte 0..39, Y = Zeile 0..24.  Veraendert A, sp, cp, shp, shf (X und Y bleiben).
set_rowptr:
        txa
        clc
        adc rowlo,y
        sta sp
        sta cp
        sta shp
        sta shf
        lda rowhi,y
        adc #0                          ; Uebertrag aus dem Low-Byte
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
        sbc #(>SCR) - (>COLORRAM)       ; Farb-RAM liegt 4 Seiten tiefer
        sta cp + 1
        rts

; put_cell: Zeichen A (Bildschirmcode) mit Farbe txtcol (C64-Farbnummer, siehe tcol) an Spalte X, Zeile Y ausgeben; schreibt Bildschirm UND
;   Schattenkopie (Zeichen) sowie Farb-RAM UND Schattenfarbe.   Veraendert A, X, Y, sp, cp, shp, shf.  Das Y-Register ist danach 0.
put_cell:
        pha
        jsr set_rowptr
        ldy #0
        pla
        sta (sp),y
        sta (shp),y
        ldx txtcol
        lda tcol,x
        sta (cp),y
        sta (shf),y
        rts

; puts: Text ab (tp), mit 0 abgeschlossen (Bildschirmcodes, siehe Hinweis zu .screen), an Spalte X, Zeile Y in Farbe txtcol ausgeben.
;   Kein Umbruch, keine Pruefung auf Zeilenende.   Veraendert A, X, Y, t1..t3 und alles, was put_cell veraendert.
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

; put_bcd: Byte A (BCD, zwei Dezimalziffern) als zwei Ziffern an Spalte X und X + 1, Zeile Y ausgeben (Ziffernzeichen = $30 + Wert, Farbe txtcol).
;   Nibbles ueber 9 ergeben Sonderzeichen ($3A..$3F).   Veraendert A, X, Y, t0, t2, t3 und alles, was put_cell veraendert.
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
;   Der TED hat KEINE Sprites.  Ball, Schlaeger, Muenze und Laserschuesse sind deshalb Zeichen: vor jedem Bild werden die Zellen unter den
;   Sprites neu mit "Pool-Zeichen" (Codes 128..191) belegt, deren Bitmuster aus dem Hintergrundzeichen und dem Bild des Sprites zusammengesetzt wird.
;   Ablauf von tsp_render (einmal je Bild, im Bildwechsel - nur dann darf der Zeichensatz geaendert werden, ohne dass es flackert):
;     1. alle Zellen des letzten Bildes aus den Schattenkopien SHC/SHF wiederherstellen,
;     2. die aktiven Schlitze (sl_*) in der Reihenfolge Schlaeger, Muenze, Schuesse, Baelle (zuletzt = oben) zeichnen:
;        je Zelle ein Pool-Zeichen vergeben (oder das schon vergebene weiterverwenden) und dessen 8 Zeilen setzen als
;            Ergebnis = (Hintergrund UND Maske) ODER Bild.
;   Bilder: lib/sprites.inc (erzeugt von tools/gen.py).  Ein Bild liegt als Streifen vor, je Feinlage s (0..3 Mehrfarbpixel nach rechts verschoben)
;   und Zeichenspalte c ein Streifen mit 8 Leerzeilen davor und danach (so kann jede senkrechte Verschiebung v = 0..7 ohne Pruefungen gelesen werden:
;   Zeile L des Zeichens liest den Streifen bei Index 8 - v + 8 * (Zeichenzeile im Sprite) + L).
;   Schlitz: sl_x in MEHRFARBPIXELN (2 Bildpunkte), sl_y in Bildpunkten.
; ---------------------------------------------------------------------------------------------------------------------
; tsp_init: alle 8 Schlitzeintraege aus und keine Pool-Zellen (nur beim Start; die Zellen-Liste ist leer).   Veraendert A, X.
tsp_init:
        ldx #7
        lda #0
.ti:    sta sl_act,x
        dex
        bpl .ti
        sta pool_n
        rts

; tsp_hide: alle Schlitze aus (die Zellen werden beim naechsten tsp_render wiederhergestellt).   Veraendert A, X.
tsp_hide:
        ldx #7
        lda #0
.th:    sta sl_act,x
        dex
        bpl .th
        rts

; tsp_render: der Compositor, einmal je Bild im Bildwechsel aufrufen (siehe oben): Zellen des letzten Bildes aus SHC/SHF wiederherstellen,
;   dann alle aktiven Schlitze in der Reihenfolge sl_order zeichnen.   Veraendert A, X, Y und alle Zeiger/Zellen des Compositors.
;   Die Laufzeit waechst mit der Zahl und Groesse der Sprites; der Aufruf soll fertig sein, bevor der Strahl die Spielfeldzeilen erreicht
;   (siehe wait_frame; die Testfassung TESTFPS zeigt verpasste Bildwechsel).
tsp_render:
        ; --- 1. Zellen des letzten Bildes wiederherstellen
        ldx pool_n
        beq .r2
.r1:    dex
        lda dl_lo,x
        sta sp
        sta shp
        sta shf
        sta cp
        lda dl_hi,x
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
        rts
sl_order: .byte 3, 4, 5, 6, 0, 1, 2     ; Zeichenreihenfolge der Schlitze: Schlaeger, Muenze, Schuesse, Baelle; spaeter = weiter oben

; tsp_draw: Sprite des Schlitzes cslot zeichnen.   Eingabe: cslot (0..6), sl_x/sl_y/sl_img/sl_col dieses Schlitzes.
;   Rechnet zuerst die Lage aus: cx = x / 4 (Zeichenspalte), cs = x mod 4 (Feinlage, Mehrfarbpixel), cy = y / 8, cv = y mod 8 (senkrechte Feinlage),
;   cn = belegte Zeichenspalten, cr = belegte Zeichenzeilen; waehlt dann die Streifen (ip0/mp0) der Feinlage und ruft je Zelle tsp_cell.
;   Zellen ausserhalb (rechts von Spalte 39, ab Zeile 24) werden ausgelassen.   Veraendert A, X, Y und die Zeiger des Compositors.
tsp_draw:
        ldy cslot
        lda sl_img,y
        sta cimg
        lda sl_col,y
        sta ccol
        lda sl_x,y
        tax
        lsr
        lsr
        sta cx                          ; Zeichenspalte
        txa
        and #3
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

; tsp_cell: eine Zelle (Bildschirmadresse sp, Streifen ip/mp, Farbe ccol) setzen.
;   Steht dort schon ein Pool-Zeichen (anderes Sprite in diesem Bild), wird darauf weitergezeichnet; sonst wird das naechste freie Pool-Zeichen
;   vergeben, in dl_lo/dl_hi eingetragen (fuer die Wiederherstellung) und das Hintergrundzeichen aus SHC/SHF als Quelle gemerkt.
;   Zellfarbe im Farb-RAM = ccol (das zuletzt gezeichnete Sprite bestimmt sie).  Pool voll: die Zelle bleibt unveraendert.   Veraendert A, X, Y.
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
        lda sp + 1
        clc
        adc #(>SHF) - (>SCR)
        sta shf + 1
        lda sp
        sta shf
        lda (shf),y                     ; Zellfarbe der Schattenkopie: Bit 3 = 0 bedeutet Text (Hires-Zelle); dessen Bitmuster waere als
        and #8                          ; Mehrfarbmuster Unsinn, also nimmt das Sprite dort ein leeres Zeichen als Hintergrund
        bne +
        ldy #CH_SPACE
        bne .bs                         ; (immer: CH_SPACE ist nicht 0)
+       lda (shp),y                     ; Hintergrundzeichen dieser Zelle
        tay
.bs:
        lda chr_lo,y
        sta bq                          ; Quelle: Bitmuster des Hintergrundzeichens
        lda chr_hi,y
        sta bq + 1
        txa
        clc
        adc #POOL0
        ldy #0
        sta (sp),y                      ; Pool-Zeichen auf den Bildschirm
        tax
        lda chr_lo,x
        sta dp                          ; Ziel: Bitmuster des Pool-Zeichens
        lda chr_hi,x
        sta dp + 1
        inc pool_n
.cmp:   lda sp                          ; Zellfarbe (Farb-RAM = Bildschirmadresse - $0400)
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

; Adresse des Bitmusters von Zeichen n im Zeichensatz (n * 8 + CHARSET), Tabellen mit 256 Eintraegen (Low-/High-Byte), erzeugt vom Assembler
chr_lo:
    .repeat 256, i
        .byte <(CHARSET + i * 8)
    .endrepeat
chr_hi:
    .repeat 256, i
        .byte >(CHARSET + i * 8)
    .endrepeat
; Zeilenanfaenge des Bildschirms: $0C00 + 40 * Zeile (25 Eintraege, Low-/High-Byte); fuer den Farb-RAM und die Schattenkopien kommt ein fester Seitenversatz dazu
rowlo:
    .repeat 25, i
        .byte <(SCR + i * 40)
    .endrepeat
rowhi:
    .repeat 25, i
        .byte >(SCR + i * 40)
    .endrepeat

; wait_frame: wartet auf den Anfang des Bildwechsels (Rasterzeile 204, unterhalb des sichtbaren Bildes).  So laeuft das Spiel
;   mit 50 Hz (PAL), und im Bildwechsel werden die Zeichen-Sprites neu gesetzt (kein Flackern, solange der Strahl nicht im Spielfeld ist).
;   Wartet auf das vblank-Flag des Rasterinterrupts und loescht es.  War das Flag schon gesetzt (das letzte Bild dauerte zu lange), kehrt die
;   Routine sofort zurueck: ein verpasstes Bild wird nicht nachgeholt, das Spiel laeuft dann kurz langsamer.  Veraendert A.
wait_frame:
.w1:    lda vblank
        beq .w1                         ; der Rasterinterrupt setzt vblank in Zeile 204
        lda #0
        sta vblank
        rts

; wait_frames: X Bilder warten (X = 1..255; 0 waere 256 Bilder); dabei laeuft der Klangeffekt weiter (sfx_update).   Veraendert A, X.
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
; title: baut den Titel auf und wartet darauf, dass das Spiel beginnt.  Kehrt zurueck, wenn Feuer gedrueckt wurde (demo = 0, Musik aus) oder
;   wenn 30 s lang nichts gedrueckt wurde (demo = 1, Musik laeuft weiter): dann spielt der Rechner (get_input/auto_input).
;   Bildaufbau: Logo in den Zeilen 3..7, Untertitel-Ticker in Zeile 10, Bestenliste "Top 5" in Zeile 12..17, Bedienhinweise in 20..22, Lauftext
;   in Zeile 24; dazu drei huepfende Baelle (Zeichen-Sprites 0..2).  Jede Wiederholung der Warteschleife entspricht einem Bild (wait_frame).
;   Veraendert alle Register.  Testfassungen: AUTOPLAY startet sofort die Demo, TESTLOSE ueberspringt die ganze Warteschleife.
title:  lda #0
        sta TED_BORDERCOLOR
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
        ldx #<song                      ; Titelmusik von vorn starten (Liedformat: examples/plus4/prallball/lib/sound.asm)
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

; title_fx: Zierde des Titels, einmal je Bild: Farbwechsel im Logo (alle 4 Bilder ein Schritt) und der Lauftext.   Veraendert A, X, Y, t1, txtcol, sp, cp, shp, shf.
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
        jsr set_rowptr                  ; cp zeigt auf das Farb-RAM (Zeile, Spalte 2)
        ldy #0
        ldx txtcol
        lda tcol,x
.tf2:   sta (cp),y                      ; Spalten 2..37 einfaerben (nur die Bloecke sind sichtbar)
        sta (shf),y                     ; (und in der Schattenkopie: die Titelbaelle laufen darueber)
        iny
        cpy #36
        bne .tf2
        inc t1
        lda t1
        cmp #5
        bne .tf1
.tf9:   rts
rainbow: .byte 10, 15, 13, 11, 14, 12, 9, 15            ; rot, gelb, gruen, tuerkis, blau, violett, weiss, gelb (je 8 + Farbe)

; Lauftext in Bildschirmzeile 24: weicher Versatz ueber $FF07 (Bits 0-2; der Rasterinterrupt im letzten Band schreibt scrollreg dorthin, siehe
;   irq), nach 8 Bildpunkten rutscht die Zeile um ein Zeichen nach links und rechts kommt das naechste Zeichen des Textes nach.
;   Die Zeile steht als Hires-Text (Farb-RAM-Bit 3 = 0) in Gelb; sie wird nicht in die Schattenkopien geschrieben, weil Sprites dort nie zeichnen.
; title_scroll_init: Lauftext starten: Feinversatz 7, scrollreg = $9F ($98 + 7), die ersten 40 Zeichen in Zeile 24, stq = Leseposition 40.   Veraendert A, X.
title_scroll_init:
        lda #7
        sta scx
        lda #$9f
        sta scrollreg
        ldx #39
.sc1:   lda scroll_text,x               ; die ersten 40 Zeichen des Textes
        sta SCR + 24 * 40,x
        lda #$77                        ; gelb (Hires-Zelle: Bit 3 = 0, normaler Text, kein Mehrfarbzeichen)
        sta COLORRAM + 24 * 40,x
        dex
        bpl .sc1
        lda #<(scroll_text + 40)        ; Leseposition hinter den ersten 40 Zeichen
        sta stq
        lda #>(scroll_text + 40)
        sta stq + 1
        rts

; title_scroll_tick: einmal je Bild: Feinversatz scx um 1 Bildpunkt verringern (7..0), bei Unterlauf die Zeile um ein Zeichen schieben und ein neues
;   Zeichen nachladen (am Textende von vorn); scrollreg = $98 | scx fuer den naechsten Interrupt.   Veraendert A, X, Y.
title_scroll_tick:
        dec scx
        bpl .sk1                        ; noch im Feinversatz
        lda #7
        sta scx
        ldx #0
.sk2:   lda SCR + 24 * 40 + 1,x         ; Zeile um ein Zeichen nach links schieben
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
        bne .sk1
        inc stq + 1
.sk1:   lda scx
        ora #$98                        ; Mehrfarbtext, 40 Spalten (Bit 7: keine Umkehrung), plus Feinversatz
        sta scrollreg
        rts

; Lauftext (Bildschirmcodes): beginnt mit 40 Leerzeichen, damit der Text von rechts hereinlaeuft, und endet mit einigen Leerzeichen als Pause; Ende = 0.
;   Die Laenge liegt ueber 256 Zeichen, deshalb der 16-Bit-Zeiger stq.
scroll_text:
        .screen "                                        prallball - zerstoere alle steine mit ball und schlaeger!   fange die muenzen, die aus zerstoerten steinen fallen:  hellblau = breiter schlaeger,  hellrot = schmaler schlaeger,  rot = umgekehrte steuerung,  gruen = ball langsamer,  gelb = ball schneller,  dunkelgelb = ball klebt,  weiss = extraball,  tuerkis = bonuspunkte,  violett = mehrball,  hellgruen = laser,  grau = durchschlag.   viel spass!                "
        .byte 0

; title_balls_init / title_balls_tick: drei Mehrfarb-Baelle (Zeichen-Sprites 0..2) prallen auf dem Titel von den Raendern ab (reine Zierde).
;   Position X in Einheiten zu 2 px (u, 8 Bit reicht fuer das ganze Bild), Y in px; Zeichen-Sprite-X = u (Mehrfarbpixel), Y = y.
;   Zustand in tbu/tby (Position) und tbdu/tbdy (Schritt je Bild) bei $C130; Startwerte aus tb_u0.. unten.   title_balls_init veraendert A, X; title_balls_tick (ruft tsp_render) alle Register.
;   title_balls_tick endet mit tsp_render und gehoert daher direkt hinter wait_frame (Bildwechsel).
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
        lda #$79                        ; weiss (Mehrfarbzelle)
        sta sl_col,x
        dex
        bpl .tbi
        lda #0
        sta sl_act + 3                  ; Schlaeger, Muenze, Schuesse aus
        sta sl_act + 4
        sta sl_act + 5
        sta sl_act + 6
        rts

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
.tby:   lda tbu,x                       ; u (Einheiten zu 2 px) ist schon die Mehrfarbpixel-Koordinate der Zeichen-Sprites
        sta sl_x,x
        lda tby,x
        sta sl_y,x
        inx
        cpx #3
        bne .tb
        jmp tsp_render                  ; Zeichen-Sprites setzen (im Bildwechsel: der Aufrufer hat gerade wait_frame verlassen)

tb_u0: .byte 30, 80, 120                ; Startwerte der drei Titel-Baelle: X in Einheiten zu 2 px
tb_y0: .byte 90, 140, 70               ;   Y in px
tb_du: .byte 1, $ff, 1                  ;   Schritt X je Bild (1 = rechts, $FF = -1 links)
tb_dy: .byte 1, 2, $ff                  ;   Schritt Y je Bild (1, 2 oder -1)

t_sub:  .screen "pralle action fuer den plus/4"      ; (unbenutzt; der Untertitel kommt aus tk_msgs)
        .byte 0
t_best: .screen "bestenliste"
        .byte 0
t_ctrl: .screen "joystick: links / rechts, feuer"
        .byte 0
t_keys: .screen "tasten: z x, leertaste"
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

; draw_top5: die ersten fuenf Bestenlisten-Eintraege (Platz, Name, Punkte) in den Zeilen 13..17: Platzziffer Spalte 9, Name Spalte 12..19,
;   Punkte (6 Ziffern) Spalte 22..27, Farbe weiss.   Veraendert alle Register, lt, lr, t0..t3.
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
; new_game: alles auf Anfang: Punkte 0, Level 0 (bzw. STARTLEVEL), Geschwindigkeitsstufe 0, Trefferzaehler 0, Schlaeger mittig, 3 Baelle;
;   springt am Ende nach reset_extras (kehrt von dort zurueck).   Veraendert A und die von reset_extras/set_padw veraenderten Zellen.
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

; reset_extras: alle Extras aus (neues Spiel, neues Level, Ball verloren): Kapsel weg, Laserschuesse weg, alle Timer 0, normale Geschwindigkeit,
;   Halbsekundenzaehler tick = 25, Schlaeger 32 px (faellt in set_padw durch).   Veraendert A, padw, zstep, pad2o.
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

; set_padw: Schlaegerbreite A (16, 32 oder 48) setzen und die Bildkennung pad2o passend einrichten (die Breite waehlt in update_sprites
;   das Zeichen-Sprite-Bild IMG_PAD16/32/48).  Eingabe: A = Breite in px.   Veraendert A, padw, zstep, pad2o.
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

; load_level: Steinkarte des Levels `level` ins RAM (MAP) kopieren, zerstoerbare Steine zaehlen (bricks, Boss-Kerne cores), Bild aufbauen
;   (Waende, Steine, Kopfzeile, Statuszeile), Extras zuruecksetzen, Ball auf den Schlaeger legen, alle Zeichen-Sprites ausblenden.
;   Eingabe: level (0..23).   Veraendert alle Register.
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
        inc cores                       ; Typ 16..23 = Boss-Kern (Lebenspunkte = Typ - 15; jeder Treffer zieht 1 ab, bei 15 ist der Kern zerstoert)
.lz:    iny
        cpy #216                        ; 12 * 18 Steine
        bne .ll
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
        jsr tsp_hide                    ; alle Zeichen-Sprites aus (update_sprites setzt sie im naechsten Bild)
        rts

; draw_walls: Oberkante (Zeile 1) und beide Seiten (Spalten 1 und 38) aus dem Mehrfarbzeichen CH_WALL (Kanten grau aus $FF16/$FF17, Flaeche blau),
;   dazu die schwarzen Randspalten 0 und 39.   Veraendert alle Register, t1, txtcol.
draw_walls:
        lda #14                         ; 8 + 6: Mehrfarbzeichen, Farbe 6 (blau) als Flaechenfarbe
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
        lda #0                          ; schwarz: Spalte 0 und 39 mit vollen Bloecken (CH_SOLID, Zeichen 127) fuellen, damit die
        sta txtcol                      ; Hintergrundbaender nur im Spielfeld zwischen den Waenden sichtbar sind
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

; draw_map: die ganze Steinkarte MAP zeichnen (12 Zeilen x 18 Spalten) mit draw_brick.   Veraendert alle Register, bidx, bcol, brow.
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

; draw_brick: Stein vom Typ A an (bcol, brow) zeichnen (Typ 0 = Leerraum) in Bildschirm und Schattenkopien.  Ein Stein sind zwei Zeichen
;   nebeneinander (CH_BRICK_L, CH_BRICK_R); die Farbe kommt aus brick_color.
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
        tax
        lda brow
        clc
        adc #4
        tay
        jsr set_rowptr
        ldy #0
        lda t0
        beq .db0
        ldx txtcol
        lda tcol,x
        sta t0                          ; Typ wird nicht mehr gebraucht: t0 = TED-Zellfarbe
        lda #CH_BRICK_L
        sta (sp),y
        sta (shp),y
        lda t0
        sta (cp),y
        sta (shf),y
        iny
        lda #CH_BRICK_R
        sta (sp),y
        sta (shp),y
        lda t0
        sta (cp),y
        sta (shf),y
        rts
.db0:   lda #CH_SPACE                   ; leerer Platz: beide Zeichen loeschen
        sta (sp),y
        sta (shp),y
        iny
        sta (sp),y
        sta (shp),y
        rts

; draw_hud: Kopfzeile "PUNKTE 000000  LEVEL n  BAELLE n" (Zeile 0; Punkte ab Spalte 7, Level Spalte 22/23, Baelle Spalte 33).  In der Demo
;   passiert nichts (dort gehoert die Zeile dem Ticker).   Veraendert alle Register, tp, txtcol.
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

; Farbe je Steintyp als Index in tcol (8 + Farbe 0..7 = Mehrfarbzeichen): 1 rot, 2 gelb, 3 gruen, 4 tuerkis, 5 blau, 6 violett, 7 weiss, 8 rot;
; 9 Silber = weiss, 10 Gold = gelb, 11 unzerstoerbar = schwarz (dunkles Metall), 12 explosiv = rot, 13 Ueberraschung = weiss;
; 14, 15 (unbelegt) = schwarz; 16..23 Boss-Kerne (Lebenspunkte 1..8) in wechselnden Farben.  Typ 0 (leer) wird nicht gefaerbt.
brick_color: .byte 8, 10, 15, 13, 11, 14, 12, 9, 10, 9, 15, 8, 10, 9, 8, 8, 10, 12, 14, 11, 13, 15, 9, 9
; Punkte je Treffer (BCD, ein Byte, Index = Steintyp): 10 * Typ fuer 1..8, Silber 50, Gold 90, unzerstoerbar 0, explosiv/Ueberraschung 10,
; Typ 14, 15 unbelegt (0), Boss-Kerne 16..23: 20 je Treffer.  "Explosiv" (12) hat in diesem Programm keine besondere Wirkung (wie Typ 1 mit 10 Punkten).
brick_pts:   .byte 0, $10, $20, $30, $40, $50, $60, $70, $80, $50, $90, 0, $10, $10, 0, 0, $20, $20, $20, $20, $20, $20, $20, $20

; ---------------------------------------------------------------------------------------------------------------------
; Spielschleife
; ---------------------------------------------------------------------------------------------------------------------
; play: spielt bis zum Ende eines Levels oder Balls.
;   Rueckgabe A: 0 = Level geschafft, 1 = Ball verloren, 2 = Abbruch (RUN/STOP oder in der Demo jede echte Eingabe).
;   Ein Durchlauf (Marke play, wird per jmp/beq wiederholt) entspricht einem Bild:
;     wait_frame -> Sprites setzen (update_sprites, im Bildwechsel) -> Klang -> Eingabe -> Effekte, Timer, Schlaeger, Kapsel, Laserschuesse ->
;     haftet der Ball, folgt er dem Schlaeger; sonst alle Baelle bewegen (move_all) und Ende des Levels/Balls pruefen.
;   Endbedingungen: normales Level = keine zerstoerbaren Steine mehr (bricks); Bonus = Zeit um (btime) oder keine Steine; Boss = alle Kerne weg.
;   Veraendert alle Register.
play:
.ifdef TESTFPS
        lda vblank
        beq .nomiss
        inc $c204                       ; Testfassung: Bildwechsel war schon da = ein Bild zu langsam (verpasst)
.nomiss:
.endif
        jsr wait_frame                  ; ab hier ~ 300 Rasterzeilen Zeit bis zum naechsten Bild
.ifdef TESTFPS
        inc $c200                       ; Testfassung: Bildzaehler (16 Bit) - zeigt, ob Bilder ausgelassen werden (ruckelt)
        bne .fp1
        inc $c201
.fp1:   lda $c200
        and #15
        bne .fp2
        lda #1
        sta txtcol                      ; weiss (Farbe < 8: normale Zeichen)
        lda $c201                       ; alle 16 Bilder: die Zaehler ($C200/$C201 Bilder, $C202/$C203 Bildwechsel des Interrupts, $C204 verpasst)
                                        ; mit put_bcd in Zeile 24 anzeigen (Spalten 6..17; Nibbles A-F erscheinen als Sonderzeichen)
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
+       jsr update_sprites              ; zuerst die Sprites setzen (Zustand des vorigen Durchlaufs), solange der Strahl unten steht
        jsr sfx_update
        jsr get_input                   ; mask = Eingabe dieses Bildes
        lda mask
        and #IN_ABORT
        beq +
        lda #2
        rts
+       jsr fx_frame                    ; Randblitz und Glanzlicht
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

; get_input: mask aus Joystick/Tastatur (input_poll); im Demo-Modus vom Rechner (auto_input).  Dann beendet jede echte Eingabe die Demo
;   (mask = IN_ABORT).  Danach wirkt das Extra "umgekehrte Steuerung" (trev <> 0: links <-> rechts vertauscht).   Veraendert A, X, Y, mask, t0..t3.
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
;   damit unterschiedliche Winkel vorkommen.  Differenz = (Schlaegerkante + atgt) - (Ballmitte); >= 6 -> links, <= -6 -> rechts, sonst
;   stehen bleiben (Totzone, siehe unten).  Feuer ist immer gesetzt.  Ergebnis in mask.   Veraendert A, X, Y, t0..t3 und den geladenen Ball (pick_ball).
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

; move_paddle: Schlaeger nach mask (IN_LEFT/IN_RIGHT) um 4 px je Bild bewegen und auf 16 .. 304 - padw begrenzen (16 Bit: padh:padl);
;   rechter Anschlag: 256 + (48 - padw), also 272 bei 32 px.   Veraendert A, t0.
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

; multiball: aus dem ersten aktiven Ball zwei weitere Baelle erzeugen (Zonen 1 und 6 = -43 / +43 Grad), soweit Schlitze frei sind.
;   Die neuen Baelle starten an der Stelle des Quellballs mit der Geschwindigkeit ihrer Zone.   Veraendert A, X, t1, t2, cb und die Ball-Zellen.
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

; launch: Ball abschiessen, leicht schraeg (Zone 3 oder 4 = -9 / +9 Grad, zufaellig).  Der haftende Ball ist immer Ball 0.   Veraendert A, X, cb, t0.
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

; set_vel: Geschwindigkeit aus der Richtungstabelle (dir_xl..dir_yh): Index = Stufe * 8 + Zone.   Eingabe: A = Zone 0..7 (0 = -60 Grad ..
;   7 = +60 Grad, 3/4 = -9/+9 Grad fast senkrecht).   Ausgabe: vxl..vyh (1/64 px je Bild).   Veraendert A, X, t0.
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

; probe: tastet einen Punkt der Ballkante in der Steinkarte ab.   Veraendert A, X, Y, ptB, bcol, brow, bidx (die drei nur bei Treffer im Feld).
;   Eingabe: Y = 0/1: X-Versatz (0 = linke Kante, 1 = rechte Kante: +5 px = +320 in 1/64),
;            X = 0/1: Y-Versatz (0 = obere Kante, 1 = untere Kante)
;   Rueckgabe: Z=1 (A=0) = kein Stein; sonst A = Typ, bcol/brow/bidx gesetzt.
;   Rechnung: Spalte = (X - 16 px) / 16 px = (bxh - 4) / 4 (High-Byte = px / 4);  Zeile = (Y - 32 px) / 8 px = (byh - 8) / 2.
probe:  stx ptB
        clc
        lda bxl
        adc offl,y                      ; nur der Uebertrag zaehlt: das Low-Byte der Abtastposition wird nicht gebraucht
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
        adc offl,y                      ; (wieder nur der Uebertrag)
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
;   Typ 11 (unzerstoerbar): nur Klang.  Gold (10) wird zu Silber (9), Silber zu Typ 1; alle anderen verschwinden.  Bei aktivem Durchschlag
;   (tpierce) verschwinden auch Gold und Silber sofort.  Boss-Kerne (16..23): jeder Treffer macht den Typ um 1 kleiner, bei 15 ist der Kern weg.
;   Fuer jeden Treffer gibt es Punkte nach brick_pts; zerstoerte normale Steine lassen mit etwa 1/12 Wahrscheinlichkeit eine Kapsel fallen
;   (Ueberraschungsstein 13: immer), im Bonuslevel gibt es 100 Punkte zusaetzlich.  Am Ende wird der Stein neu gezeichnet und die Punktzahl
;   aktualisiert (.redraw).   Veraendert alle Register, t3.
hit_brick:
        pha
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
;   Eingabe: Ball in der Nullseite (bxl.., alte Y-Lage oyl/oyh), padl/padh/padw/zstep, mask (Schlaegerbewegung beeinflusst die Zone).
;   Ausgabe: C=1 = vom Schlaeger abgeprallt (Y auf Schlaegeroberkante gesetzt, neue Geschwindigkeit, hits/Stufe, Klang), sonst C=0;
;   bei einem Seitenaufprall wird der Ball neben den Schlaeger gesetzt und C=0 gemeldet.   Veraendert A, X, t0..t3.
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
+       ldx #0                          ; Zone = (Ballmitte - Schlaegerkante) / zstep, hoechstens 7 (Division durch wiederholtes Abziehen; C=1 nach cmp)
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
; Bosslevel, Statuszeile, Effekte
; ---------------------------------------------------------------------------------------------------------------------
; boss_attack: der Boss laesst eine boese Kapsel fallen (S = schmaler Schlaeger oder R = umgekehrte Steuerung) aus einer zufaelligen Spalte
;   (Steinspalte 1..16, Zeile 4); nur wenn nicht schon eine Kapsel faellt.  Wird alle 4 s von tick_timers gerufen.   Veraendert A und die von spawn_cap veraenderten Zellen.
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

; draw_status: Statuszeile (Bildschirmzeile 24, gelb): Bonus = verbleibende Sekunden (btime), Boss = verbliebene Kerne (cores); sonst nichts.
;   Die Zahl (0..99) wird ohne put_cell direkt auf den Bildschirm geschrieben (kein Schattenspeicher noetig: Sprites zeichnen in Zeile 24 nie).
;   Veraendert A, X, Y, tp, txtcol, sp, cp, shp, shf.
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
        pha
        ldx #25
        ldy #24
        jsr set_rowptr
        pla
        ldy #0
        sta (sp),y
        pla
        clc
        adc #$30
        iny
        sta (sp),y
        rts
.ds0:   rts
s_bonus: .screen "bonus   "
        .byte 0
s_boss: .screen "boss: kerne "
        .byte 0

; fx_frame: Randblitz herunterzaehlen (bflash; bei 0 wird der Rand wieder schwarz) und das Glanzlicht ueber die Steine laufen lassen (je zweites
;   Bild ein Schritt; glg 0..30 = Diagonale weiss, danach Pause bis 99).   Veraendert A, X, Y, t2, t3 und die Zellen von glint_run.
fx_frame:
        lda bflash
        beq .fx1
        dec bflash
        bne .fx1
        lda #0
        sta TED_BORDERCOLOR             ; Blitz zu Ende
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
;   gefaerbt bzw. in der richtigen Farbe neu gezeichnet.  Veraendert alle Register, t2, t3, bcol, brow.
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
.grw:   lda bcol                        ; weiss faerben: Farb-RAM beider Zeichen = 8 + 1
        asl
        clc
        adc #2
        tax
        lda brow
        clc
        adc #4
        tay
        jsr set_rowptr
        ldy #0
        lda #$79                        ; weiss (Mehrfarbzelle), auch in der Schattenkopie
        sta (cp),y
        sta (shf),y
        iny
        sta (cp),y
        sta (shf),y
.gr8:   inc brow
        lda brow
        cmp #12
        bne .gr1
.gr9:   rts

; update_sprites: Zustand des Spiels in die Schlitze der Zeichen-Sprites (sl_*) uebertragen und diese zeichnen (tsp_render).
;   Schlitze: 0..2 Baelle, 3 Schlaeger, 4 Muenze, 5..6 Laserschuesse.  X wird in Mehrfarbpixeln (2 Bildpunkte) angegeben: X / 2, Y in Bildpunkten.
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
        lda pxh
        lsr                             ; Bit 0 des High-Bytes ins Carry ...
        lda pxl
        ror                             ; ... und von oben in das Low-Byte: X / 2 = Mehrfarbpixel (0..159)
        sta sl_x,x
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
        lda #$79                        ; Ballfarbe: weiss, bei Durchschlag gelb (Mehrfarbzellen)
        ldy tpierce
        beq +
        lda #$7f
+       sta sl_col,x
        lda #1
        sta sl_act,x
.ubn:   inc cb
        lda cb
        cmp #3
        bne .ub
        ; --- Schlaeger (Schlitz 3): Breite ueber pad2o (0 = 16 px, 16 = 32 px, 24 = 48 px)
        lda padh
        lsr
        lda padl
        ror
        sta sl_x + 3
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
        lda #$6e                        ; hellblau (Mehrfarbzelle)
        sta sl_col + 3
        lda #1
        sta sl_act + 3
        ; --- Muenze (Schlitz 4)
        lda capact
        bne +
        sta sl_act + 4
        beq .nocap
+       lda capxh                       ; Muenzenbild liegt in einem Rahmen aus 2 Zeichen (8 Mehrfarbpixel) ab capx / 2
        lsr
        lda capxl
        ror
        sta sl_x + 4
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
.sson:  lda shu,x                       ; X-Lage in Einheiten zu 2 px = Mehrfarbpixel
        sta sl_x + 5,x
        lda shy,x
        sta sl_y + 5,x
        lda #IMG_SHOT
        sta sl_img + 5,x
        lda #$7f                        ; gelb
        sta sl_col + 5,x
        lda #1
        sta sl_act + 5,x
.ssn:   inx
        cpx #2
        bne .ss
        jmp tsp_render

; tick_timers: einmal je Bild.  Alle 25 Bilder (0,5 s) zaehlen die Extra-Timer (tpad, trev, tspd, tlas, tpierce) um 1 herunter; bei 0 endet das
;   Extra.  Dazu im Bonuslevel die Restzeit (btime, jede zweite Halbsekunde) und im Bosslevel der Angriffszaehler batk.   Veraendert A (und was draw_status/boss_attack/set_padw veraendern).
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
;   Die Kapselart (captype) wird nach cap_pick zufaellig gewaehlt oder von boss_attack erzwungen (capforce).  Der Buchstabe steckt fertig im
;   Zeichen-Sprite-Bild IMG_COIN_B + Art (aus tools/gen.py); die Farbe kommt aus cap_tcol.  Startposition: capx = 16 + 16 * Spalte, capy = 32 + 8 * Zeile.
;   Veraendert A, X.
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

; Kapselarten (captype): 0 B breit, 1 S schmal, 2 R umgekehrt, 3 L langsam, 4 F schnell, 5 K klebrig, 6 + Leben, 7 P Punkte (500),
; 8 M Mehrball, 9 G Laser, 10 D Durchschlag.  Wirkung: siehe activate.
; cap_img: Bild der Drehphasen 0..5 (capfr / 4); 0 = Vorderseite mit dem Buchstaben der Art (IMG_COIN_B + captype)
cap_img:    .byte 0, IMG_COIN_TQ, IMG_COIN_HALF, IMG_COIN_EDGE, IMG_COIN_HALF, IMG_COIN_TQ   ; Bilder der Drehphasen (0 = Vorderseite je nach Art)
; cap_tcol: Farbe der Muenze je Art (TED-Zellfarbe, Mehrfarbzelle, Index = captype): B hellblau, S hellrot, R rot, L gruen, F gelb, K orange (gelb, gedaempft: Mehrfarbzellen
; haben nur die Farbtoene 0..7), + weiss, P tuerkis, M violett, G hellgruen, D (Durchschlag) grau.  Der Buchstabe steckt im Bild (IMG_COIN_*).
cap_tcol:   .byte $6e, $6a, $3a, $4d, $7f, $5f, $79, $6b, $5c, $7d, $39
; Gewichtung der Kapselarten bei der Auswahl: 16 Eintraege (Zufallszahl and 15); M kommt oefter als Leben (+)
cap_pick:   .byte 0, 0, 1, 2, 3, 4, 5, 6, 7, 7, 8, 8, 8, 9, 9, 10

; laser_fire: zwei Schuesse an den Enden des Schlaegers starten (nur wenn beide Schlitze frei sind).   Veraendert A, t0, t1.
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
;   Ein Schuss verschwindet ueber der Decke (y < 20).  Die Schussnummer steht in shi (hit_brick veraendert die Register).   Veraendert alle Register.
;   Spalte = (u - 8) / 8 (u = px / 2, Stein 16 px = 8 Einheiten breit, Feld beginnt bei px 16 = u 8), Zeile = (y - 32) / 8.
update_shots:
        ldx #0
.us:    stx shi                         ; Schuss 0..1; shi, weil hit_brick X veraendert (X wird danach aus shi neu geladen)
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

; update_cap: Kapsel um 1 px je Bild fallen lassen; faengt der Schlaeger sie (Ueberlappung in X und Y), wirkt sie (activate).  Faellt sie unten
;   (capy >= 200) heraus, verschwindet sie.  Die Muenze ist 12 x 12 px gross (8 Mehrfarbpixel breiter Rahmen, Muenze 2 px eingerueckt).   Veraendert A, t0, t1.
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

; activate: Kapsel wirkt.  Art in captype (siehe die Liste der Kapselarten bei cap_img).  Zeiten in Halbsekunden (tick_timers).
;   Veraendert A (und was set_padw/multiball/draw_hud/draw_score veraendern); endet mit rts dieser Routinen.
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
.a10:   lda #10                         ; D: Durchschlag fuer 5 s
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
; game_over: Spielende anzeigen ("SPIEL ZU ENDE"), Punkte an die Bestenliste uebergeben; kommt der Spieler hinein (hs_check), Namenseingabe,
;   Eintrag (hs_insert) und Speichern (hs_save, bei gesperrten Interrupts) mit Meldung.  In der Demo keine Bestenliste.  Nach 3 s Rueckkehr.
;   Veraendert alle Register.
game_over:
        jsr tsp_hide                    ; Ball und Schlaeger ausblenden
        jsr wait_frame
        jsr tsp_render                  ; (stellt die Zellen unter den Sprites wieder her)
        lda #0
        sta TED_BORDERCOLOR             ; ein laufender Randblitz endet
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
;   Tastatur (input_char): Zeichen tippen (A-Z, 0-9, Leerzeichen, - und .), RETURN bestaetigt, DEL loescht.  Das erste getippte Zeichen
;   (oder DEL) loescht die Vorgabe.  Joystick (input_joy, nur der Joystick; die Buchstabentasten Z X Q A sind hier normale Zeichen):
;   hoch/runter wechselt das Zeichen an der Cursorstelle (Folge ctab: Leerzeichen, A-Z, 0-9, im Kreis), rechts/links bewegt den Cursor,
;   Feuer bestaetigt.  Haelt man eine Joystickrichtung, wiederholt sie sich (nach 25 Bildern, dann alle 6).  Ohne jede Eingabe wird nach 30 s
;   (1500 Bilder bei 50 Hz) automatisch bestaetigt; jede Eingabe stellt den Zaehler zurueck.  Der Name steht als ASCII in hs_name (8 Byte).
;   Veraendert alle Register, npos, nprev, nrep, ntl/nth, nch, ntyped, mask, t1.
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
        jsr input_joy                   ; Joystick: nur die Joystickkontakte, beide Anschluesse (Z/X/Q/A sind hier Buchstaben und keine Richtungen)
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

; cur_idx: Index des Zeichens an der Cursorstelle in der Zeichenfolge ctab (nicht gefunden, z. B. '-' oder '.': 0).  Rueckgabe X = Index, Y = npos.
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
ctab:   .text " ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"   ; 37 Zeichen (ASCII), Reihenfolge fuer hoch/runter am Joystick

; name_draw: den Namen in Zeile 22 ab Spalte 16 anzeigen; das Zeichen unter dem Cursor mit gesetztem Bit 7 des Bildschirmcodes (gedacht als
;   Umkehrschrift; ob das Zeichen 128+ des RAM-Zeichensatzes dann wirklich invers aussieht, haengt von seinem Inhalt ab, siehe copy_font).
;   ASCII -> Bildschirmcode: Buchstaben (65..90) minus 64, Ziffern und Leerzeichen unveraendert.   Veraendert alle Register, t1, txtcol.
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
;   $F700 = TK_BUF, je Zeichen 8 Byte, ein Byte je Pixelzeile) werden in jedem Schritt neu berechnet.  So sind Bewegungen um einzelne BILDPUNKTE moeglich,
;   obwohl nur Zeichen im Raster sichtbar sind.  Die Bitmuster der Buchstaben holen wir aus dem ins RAM kopierten ROM-Zeichensatz (CHARSET + Code * 8).
;   EFFEKTE (zufaellig gewaehlt, Effekt 0 doppelt so haeufig wie die anderen: Zufallszahl and 7, 7 wird zu 0): 0 nach oben wegscrollen (neu von unten), 1 nach unten, 2 buchstabenweise (erst rechts nach links loeschen,
;   dann links nach rechts schreiben), 3 nach links wegscrollen (neu von rechts), 4 nach rechts, 5 Wischer (Kante laeuft pixelweise von links
;   nach rechts), 6 Aufloesen (zufaellige Pixelzeilen werden ausgetauscht).  Dazu eine laufende Regenbogenfarbe ueber die Zeichen.
;   ABLAUF: warten (tkw Bilder), naechsten Text aus der Folge holen, Effekt starten, Schritte ausfuehren, fertig -> warten.
; Zustand (Nullseite): tkrow Zeile, tkst 0 = warten / 1 = Effekt, tkw Wartebilder, tkeff Effekt, tkstep Schritt, tkmsg Folgenindex, tkseq 0 Titel / 1 Demo
; =====================================================================================================================
.const TK_BUF = CHARSET + 224 * 8       ; Bitmuster der Zeichen 224..255 im RAM-Zeichensatz = $F700..$F7FF (32 Zeichen * 8 Byte)
.const TK_CHR = 224                     ; erstes Ticker-Zeichen (Bildschirmcode); Spalte 4 + k der Ticker-Zeile zeigt TK_CHR + k
.const tk_cur = $c140                   ; aktuell angezeigter Text (32 Bildschirmcodes)
.const tk_new = $c160                   ; Zieltext (32 Bildschirmcodes), das Ziel des laufenden Effekts

; ticker_init: A = Bildschirmzeile, X = Folge (0 Titel: Texte 0, 1;  1 Demo: Texte 2, 0, 1).  Zeigt den ersten Text sofort.
;   Setzt die Zeile Spalte 4..35 auf die Zeichen 224..255 (gelb, Hires) und rendert den Text in TK_BUF.   Veraendert alle Register.
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
        ldy #0
.ti2:   tya
        clc
        adc #TK_CHR
        sta (sp),y                      ; Zeichen 224 + Spalte
        sta (shp),y
        lda #$77                        ; gelb (Hires-Zelle)
        sta (cp),y
        sta (shf),y
        iny
        cpy #32
        bne .ti2
        rts

; tk_loadnew: A = Textnummer (0..2): 32 Zeichen aus tk_msgs nach tk_new kopieren.   Veraendert A, X, Y.
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

; tk_ga / tk_gb: A = Bildschirmcode -> tka / tkb = Adresse des Bitmusters im Zeichensatz (CHARSET + Code * 8).   Veraendern A (zwei gleiche Routinen,
;   weil der Aufrufer meist beide Zeiger braucht: tka = altes, tkb = neues Zeichen).
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

; tk_renderchar: Zeichen tkc (0..31) des angezeigten Textes tk_cur in den Zeichensatz (TK_BUF) kopieren (8 Pixelzeilen).   Veraendert A, X, Y, tka.
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

; tk_renderall: alle 32 Zeichen neu aus tk_cur zeichnen.   Veraendert A, X, Y, tkc, tka.
tk_renderall:
        lda #0
        sta tkc
.ra:    jsr tk_renderchar
        inc tkc
        lda tkc
        cmp #32
        bne .ra
        rts

; tk_sync: der Zieltext wird der angezeigte Text (Effekt zu Ende), sauber neu gezeichnet.   Veraendert A, X, Y, tkc, tka.
tk_sync:
        ldx #31
.sy:    lda tk_new,x
        sta tk_cur,x
        dex
        bpl .sy
        jmp tk_renderall

; ---- Effekt 0/1: senkrecht pixelweise.  tkd = Versatz d (+1..+8 nach oben, -1..-8 nach unten): Pixelzeile r zeigt Zeile r + d des alten Textes,
;   liegt r + d ausserhalb 0..7, die Zeile (r + d) mod 8 des neuen Textes.
; Eingabe: tkd = d, tkc = erstes Zeichen, tkce = Ende (exklusiv).   Veraendert A, X, Y, tkc, tkr, tks, tkt, tka, tkb.
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
;   Eingabe: tkstep.
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

; ---- Effekt 3: nach links wegscrollen, neuer Text kommt von rechts (je Schritt 4 Pixel, 64 Schritte).  Eingabe: tkstep, tkrs/tkre (Pixelzeilen).
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

; ---- Effekt 4: nach rechts wegscrollen, neuer Text kommt von links (die rechten Spaltengruppen des neuen Textes zuerst).  Eingabe wie Effekt 3.
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

; ---- ticker_tick: einmal je Bild (im Titel und in der Demo).  Laesst die Regenbogenfarbe wandern (alle 2 Bilder) und treibt den Zustandsautomaten:
;   tkst = 0: tkw herunterzaehlen (4 s), dann naechsten Text der Folge laden und einen Effekt waehlen; tkst = 1: einen Teilschritt des Effekts
;   ausfuehren (senkrechte Effekte in 4 Vierteln, waagerechte in 2 Haelften je Schritt, damit ein Bild nicht zu lang dauert); am Ende tk_sync.
;   Veraendert alle Register.
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
.tkc1:  tya
        clc
        adc tkt
        and #7
        tax
        lda tk_pal,x
        tax
        lda tcol,x                      ; C64-Farbnummer -> TED-Zellfarbe
        sta (cp),y
        sta (shf),y
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
        lda #0
        sta tkd                         ; (tk_vert-Aufrufer setzen d bei jedem Schritt selbst)
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
        .screen " pralle action fuer den plus/4  "
        .screen " ein demospiel von vintagecoder "
        .screen "       *** prallball ***        "

; ---------------------------------------------------------------------------------------------------------------------
; Bibliotheken und Daten
; ---------------------------------------------------------------------------------------------------------------------
; Die Bibliotheken aus data/sys/plus4/ liegen hier mitten im Programmbereich (alles unter $8000, damit das KERNAL bei der Bestenliste sie sieht).
.include "lib/input.asm"             ; input_init/poll/joy/char/wait_release, IN_*-Masken (Nullseite $8C-$8F)
.include "lib/rand.asm"              ; rand_seed, rand_byte (Nullseite $02-$03)
.include "lib/sound.asm"             ; sfx_init/play/update, mus_init/tick/stop, SFX_*-Nummern, Frequenztabellen (Nullseite $04-$07)
.include "lib/hiscore.asm"           ; hs_load/save/check/insert, hs_buf/hs_points/hs_level/hs_name, HS_*-Konstanten (Nullseite $08-$0D)
.include "lib/dirtable.inc"             ; dir_xl, dir_xh, dir_yl, dir_yh: 16 Stufen x 8 Zonen, 16 Bit je Wert (erzeugt von tools/gen.py)
.include "lib/gfx.inc.chars"            ; gfx_chars: 5 Zeichen zu 8 Byte (64..67, 127), siehe copy_font (erzeugt von tools/gen.py)
.include "lib/sprites.inc"              ; Zeichen-Sprites: Streifen und Tabellen im_*, Konstanten IMG_* (erzeugt von tools/gen.py)
.include "lib/levels.inc"               ; level_count, level1..level24 (je 216 Byte), level_kind, level_tab_lo/hi (erzeugt von tools/gen.py)
.include "lib/music.inc"                ; song, song_patterns: Titelmusik (erzeugt von tools/compose.py)
