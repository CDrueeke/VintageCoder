; =====================================================================================================================
; prallball.asm - PRALLBALL fuer den Atari 8 Bit (XL/XE, PAL): Ball-und-Schlaeger-Spiel
; =====================================================================================================================
;
; ZWECK
;   Dasselbe Spiel wie auf dem C64, aber mit eigener Umsetzung fuer die Atari-Hardware.  Die reine Spiellogik
;   (Ball, Steine, Extras, Bonus-/Bosslevel) ist ein 6502-Programm und aehnelt deshalb der C64-Fassung; alles an Anzeige, Farben, Figuren,
;   Klang und Eingabe ist Atari-spezifisch.  Gemeinsam mit dem C64 sind nur die Datentabellen lib/dirtable.inc, lib/levels.inc und
;   lib/music.inc (unveraenderte Kopien).  Zwei Namen der C64-Fassung (VIC_SPR_ENABLE, VIC_BORDERCOLOR) blieben als Platzhalter-
;   Speicherzellen stehen: sie werden nur beschrieben, nirgends gelesen und steuern keine Hardware.
;
; BEDIENUNG
;   Joystick Port 1 (links/rechts, Feuer) ODER Tastatur: Z links, X rechts, Leertaste Feuer, ESC Abbruch.  Namenseingabe: tippen + RETURN.
;   Titel: Feuer startet; ohne Eingabe beginnt nach 30 s die Demo (der Rechner spielt, jede Eingabe beendet sie).
;
; ABLAUF
;   start: hal_init (Chips, Zeichensatz, Spieler/Missiles, Display-Listen, DLI und Bildruecklauf-Interrupt), dann Klang, Eingabe, Zufall und
;   Bestenliste (Ladelaufwerk, Standardliste, Laden) einrichten.  Danach endlos: title (Titelbild mit Logo, Bestenliste, Ticker, Lauftext,
;   Musik) -> new_game -> je Level load_level und play (Rueckgabe: 0 Level geschafft, 1 Ball verloren, 2 Abbruch); nach dem letzten Level
;   beginnt wieder das erste -> game_over (Bestenliste mit Namenseingabe) -> title.  Die Demo kehrt nach dem fuenften Level (Index 4) zum
;   Titel zurueck.  24 Level: Nr. 11 und 22 sind Bonuslevel (45 s, ein verlorener Ball kostet kein Leben), Nr. 23 und 24 Bosslevel (Kerne
;   zerstoeren, der Boss laesst boese Kapseln fallen).  Die Ballgeschwindigkeit steigt alle 8 Schlaegertreffer um eine Stufe (0..15).
;   Start mit LIVES0 = 3 Baellen.
;
; ANZEIGE (ANTIC/GTIA)
;   * DISPLAY-LISTE: 24 Textzeilen.  Kopf- und Textzeilen im Modus 2 (Hires-Text, 40 Zeichen zu 8 x 8 Punkten), das Spielfeld im Modus 4
;     (Mehrfarbtext 40 x 24: jedes Zeichen 4 breite Pixel (je 1 Farbtakt = 2 px) x 8 Zeilen zu 2 Bit).  Beide Modi teilen sich EINEN
;     Zeichensatz (CHBAS): Codes 0-63 = Schrift (aus dem ROM kopiert, fuer Modus 2), Codes 64-95 = Tickerzeichen (Modus 2),
;     Codes 96-105 = Spielfeldzeichen (Modus 4, aus lib/gfx.inc.chars; Bit 7 des Codes waehlt im Modus 4 die Farbe PF3 statt PF2).
;     Drei Listen: DLG (Spiel), DLT (Titel, mit Feinscroll-Zeilen), DLO (Spielende); use_mode schaltet zwischen ihnen um.
;   * FARBEN JE ZEILE (Display-List-Interrupt): Im Modus 4 sind die vier Farben (COLPF0..3 + COLBK) fuer den ganzen Bildschirm gleich.  Ein DLI
;     am Ende jeder Textzeile (im Spiel Zeilen 0..17, im Titel 0..22) setzt deshalb die fuenf Farbregister der FOLGENDEN Zeile aus einer
;     Tabelle (RC_BK, RC_0..RC_3, je 24 Byte): so hat jede Zeile ihren eigenen Himmel (PF0), Steinfarbe (PF1), Lichtkante (PF2) und
;     Metallfarbe (PF3).  Der Himmel steckt in den Zeichen selbst (Pixelwert 01), COLBK bleibt schwarz: so reichen die Farben nicht ueber
;     das Spielfeld hinaus in den Rand.  Der Bildruecklauf-Interrupt (vbi) setzt die Farben der Zeile 0 und den Zeilenzaehler dli_i zurueck.
;   * RASTERZEILEN-KERNEL: Im Spiel setzt der DLI am Ende der Zeile 17 die Farben der Zeile 18 und ruft dann kernel auf, das die CPU fuer
;     die Zeilen 18..23 (47 Rasterzeilen) belegt: je Rasterzeile (WSYNC) ein neuer Himmelswert (kgrad: weicher Verlauf) und eine neue
;     Farbe der Spieler 0-2 (kpm: Metallglanz im Schlaeger).  Waehrend dieser Zeit rechnet die CPU nichts anderes.
;   * SPIELER/MISSILES (Player-Missile-Grafik, Einzelzeile, PMBASE = $90, GPRIOR = 1: vor dem Spielfeld): Schlaeger = Spieler 0-2
;     (je 16 Punkte breit: 16, 32 oder 48 Punkte), Kapsel = Spieler 3, Baelle = Missiles 0-2, Laserschuss = Missile 3.  Eine Zeile im
;     Spielerspeicher = eine Rasterzeile; Index = 32 + y.
; KLANG (POKEY): Effekte auf Kanal 4, Titelmusik auf allen vier Kanaelen (examples/atari/prallball/lib/sound.asm; Kanal 3+4 als 16-Bit-Bass).
; BENUTZTE HARDWARE: ANTIC (DMACTL und DLISTL ueber die OS-Schatten SDMCTL/SDLSTL, CHBASE ueber CHBAS, HSCROL, PMBASE, WSYNC, NMIEN),
;   GTIA (HPOSP/HPOSM, SIZEM, COLPF0-3, COLBK, COLPM0-2 und der Schatten PCOLR0-3, GRACTL, PRIOR ueber GPRIOR), POKEY (RANDOM, Klang,
;   Tastatur ueber die Bibliotheken), PIA (Joystick), OS (SETVBV/XITVBV, CIO fuer die Bestenliste).  NMIEN = $C0 (DLI + VBI).
;
; KOORDINATEN: wie auf dem C64 in Bildpunkten (px, 320 breit), Ball in 1/64 px.  Ein Farbtakt des Atari = 2 px.  Horizontale Spritelage (HPOS) =
;   48 + px / 2 (linker Rand des Normalfeldes ist Farbtakt 48), senkrecht Index = 32 + y im Spielerspeicher (y = 0 ist die erste Textzeile).
;   Spielfeld: x = 16..303 px (Waende in den Spalten 1 und 38), Steine ab y = 32 px (Zeile 4), Schlaeger bei PADY = 184 px.
;
; SPEICHERPLAN
;   $2000-$7FFF  Programm (zuerst die Spielfeldzeichen gfx_chars), Daten, Level, Lied, Bibliotheken
;   $8000-$83BF  Bildschirm SCR (24 x 40 = 960 Byte, liegt in einem 4-KB-Block)
;   $8400 / $8440 / $8480  Display-Listen Spiel (DLG) / Titel (DLT) / Spielende (DLO)     $8500  Lauftextzeile SCRL (48 Zeichen)
;   $8600-$86EF  Logozeilen LOGOBUF (5 x 48 Zeichen)     $8800-$8BFF  Zeichensatz CHARSET (1 KB, auf der 1-KB-Grenze)
;   $9000-$97FF  Spielerspeicher (PMBASE $90; $9000-$92FF bleibt ungenutzt): Missiles $9300, Spieler 0-3 ab $9400 / $9500 / $9600 / $9700
;   $9800  Steinkarte MAP (12 x 18 = 216 Byte)     $9900  Ballspeicher (3 Baelle)     $9920  Laserschuesse     $9940 / $9960  Tickertexte
;   $9980  Titelbaelle     $9A00  Tickervariablen     $9B00-$9B77  Farbtabellen je Zeile (RC_BK, RC_0..RC_3)     $9B80-$9BAF  Variablen
;   des DLI/VBI (darin RC_H = Feinscroll-Wert je Zeile)
; NULLSEITE: $80-$8B Zeiger (sp, tp, tka, tkb, stq, mdp), $8C-$EF Spielzustand und Hilfsspeicher, $F0-$FE Bibliotheken (Eingabe $F0, Musik $F4,
;   Bestenliste $F8; die Vorgaben der Bibliotheken liegen anders und werden unten ueberschrieben).
;
; TESTFASSUNGEN (vcc asm prallball.asm --target atari8 --cpu sally -D NAME[=wert] -o ...; ohne -D ist es das normale Spiel):
;   AUTOPLAY       der Rechner spielt von Anfang an, der Titel wird sofort verlassen (DEMOFLAG = 1: wie die Demo, Klang stumm; 0: mit Kopfzeile)
;   TESTLOSE       Titel sofort verlassen, nur Feuer wird gedrueckt: der Ball geht verloren (prueft Spielende und Namenseingabe)
;   NOHS           Bestenliste beim Start nicht von der Diskette laden (Standardliste)
;   HS_HOSTTEST    Bestenliste ueber das Host-Geraet H1: des Emulators; zeigt nach dem Speichern den CIO-Statuscode (debug_dec)
;   TESTHS         der Titel schreibt und liest die Bestenliste sofort, zeigt beide Statuscodes (1 = Erfolg) und bleibt stehen
;   STARTLEVEL=n   mit Level n beginnen (0-basiert)
;   TESTMULTI      sobald eine Kapsel faellt (spawn_cap), wird sofort Mehrball ausgeloest; die Kapsel ist immer Mehrball
;   TESTCAP        jeder zerstoerte Stein laesst eine Kapsel fallen
;   TESTLASER      Laserzeit bei jedem reset_extras auf 240 Halbsekunden (120 s) statt 20 s
;   TESTKEY        spielt in der Namenseingabe Tastencodes in den OS-Tastaturpuffer ein (testkey)
;   TESTPM         der Ball haftet bei y = 96 px statt auf dem Schlaeger (Vergleich von Sprite- und Spielfeldzeilen)
;   NOTICKER       kein Ticker in der Demo (ticker_tick wird in play nicht aufgerufen)
;
; BEKANNTE GRENZEN: nur ein Laserschuss gleichzeitig (ein Missile); Paddle-Eingabe ungetestet; die Bestenliste wurde nur mit DOS und mit
;   dem Host-Geraet des Emulators geprueft; einige Konstanten und Tabellen stammen aus der C64-Fassung und sind ungenutzt (am Ort vermerkt).
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }
.branchlong auto                        ; zu weite bedingte Spruenge verlaengert der Assembler selbst (Warnung W203)

; Bibliotheks-Nullseite (Vorgaben der Bibliotheken ueberschreiben)
.const IN_ZP  = $f0                     ; 4 Byte
.const MUS_ZP = $f4                     ; 4 Byte
.const HS_ZP  = $f8                     ; 7 Byte

.ifndef DEMOFLAG
.const DEMOFLAG = 1
.endif
.const bxl      = $8c  ; Ball-X, Festkomma 1/64 px, Low-Byte
.const bxh      = $8d  ;                           High-Byte
.const byl      = $8e  ; Ball-Y, Festkomma 1/64 px
.const byh      = $8f
.const vxl      = $90  ; Ball-Geschwindigkeit X (vorzeichenbehaftet, 1/64 px je Bild)
.const vxh      = $91
.const vyl      = $92  ; Ball-Geschwindigkeit Y (negativ = nach oben)
.const vyh      = $93
.const padl     = $94  ; Schlaeger: linke Kante in px (16 Bit, 16..272)
.const padh     = $95
.const speed    = $96  ; Geschwindigkeitsstufe 0..15 (Index in die Richtungstabelle)
.const hits     = $97  ; Schlaegertreffer seit dem Start; alle 8 steigt die Stufe um 1
.const lives    = $98  ; verbleibende Baelle
.const level    = $99  ; aktuelles Level 0.. (Index in die Levelliste)
.const bricks   = $9a  ; noch zu zerstoerende Steine (unzerstoerbare zaehlen nicht)
.const stuck    = $9b  ; 1 = Ball haftet noch am Schlaeger (wartet auf Feuer)
.const sc0      = $9c  ; Punktestand BCD, niedrigste zwei Ziffern (Dezimalmodus!)
.const sc1      = $9d  ;                  mittlere zwei Ziffern
.const sc2      = $9e  ;                  hoechste zwei Ziffern  -> 6 Stellen
.const mask     = $9f  ; Eingabemaske dieses Bildes (IN_*)
.const t0       = $a0  ; Hilfsspeicher (kurzlebig, jede Routine beschreibt ihren Gebrauch)
.const t1       = $a1
.const t2       = $a2
.const t3       = $a3
.const bcol     = $a4  ; zuletzt getroffener Stein: Spalte 0..17
.const brow     = $a5  ;                             Zeile 0..11
.const bidx     = $a6  ;                             Index in die Steinkarte (Zeile * 18 + Spalte)
.const pxl      = $a7  ; Ball-Position in ganzen px (aus get_px), Low-Byte
.const pxh      = $a8  ;                                          High-Byte
.const txtcol   = $a9  ; Textfarbe (Rest der C64-Fassung: wird nur beschrieben; die Farbe je Zeile kommt aus den Tabellen RC_*)
.const atgt     = $aa  ; Autoplay: Zielpunkt auf dem Schlaeger (8..23 px von der linken Kante)
.const ptA      = $ab  ; (ungenutzt, Rest der C64-Fassung)
.const ptB      = $ac  ; probe: gemerkter Y-Versatz (Eingabe X); Hilfsspeicher
.const oxl      = $ad  ; Ball-X vor der Bewegung (zum Zuruecksetzen beim Abprall)
.const oxh      = $ae
.const oyl      = $af  ; Ball-Y vor der Bewegung
.const oyh      = $b0
.const dxo      = $b1  ; Abtastversatz X der Ballkante (0 = links, 1 = rechts)
.const npos     = $b2  ; Namenseingabe: Cursorposition 0..7
.const nprev    = $b3  ;   Eingabemaske des vorigen Bildes (Flankenerkennung)
.const nrep     = $b4  ;   Bilder bis zur naechsten Tastenwiederholung
.const ntl      = $b5  ;   Zaehler bis zur Auto-Bestaetigung (16 Bit, in Bildern), Low-Byte
.const nth      = $b6  ;   High-Byte
.const padw     = $b7  ; aktuelle Schlaegerbreite in px (16, 32 oder 48)
.const zstep    = $b8  ; Breite einer Abprallzone = padw / 8 (2, 4 oder 6 px)
.const pad2o    = $b9  ; (ungenutzt; C64: Abstand des zweiten Schlaegersprites. Hier liegen die Spieler 0-2 immer 8 Farbtakte auseinander)
.const capact   = $ba  ; 1 = eine Kapsel faellt
.const capxl    = $bb  ; Kapsel: linke Kante in px (16 Bit)
.const capxh    = $bc
.const capy     = $bd  ;          obere Kante in px
.const captype  = $be  ;          Art 0..10: B, S, R, L, F, K, +, P, Mehrball, Laser, Durchschlag (siehe cap_pick, activate)
.const tpad     = $bf  ; Rest-Halbsekunden: Schlaegerbreite geaendert (breit/schmal)
.const trev     = $c0  ;                    Steuerung umgekehrt
.const tspd     = $c1  ;                    Geschwindigkeit geaendert
.const spdmod   = $c2  ; 0 = normal, 1 = schneller (+4 Stufen), 2 = langsamer (-4 Stufen)
.const tick     = $c3  ; Bilder bis zur naechsten halben Sekunde (zaehlt die Timer)
.const sticky   = $c4  ; 1 = naechster Schlaegertreffer faengt den Ball (Klebe-Extra)
.const demo     = $c5  ; 1 = Demo-Modus (Rechner spielt)
.const idl      = $c6  ; Titel: Bilder ohne Eingabe bis zur Demo (16 Bit)
.const idh      = $c7
.const lt       = $c8  ; Titellogo: Buchstabe 0..8
.const lr       = $c9  ;             Zeile 0..4
.const lbits    = $ca  ;             Bitmuster der Zeile
.const lcol     = $cb  ;             Spalte des ersten Buchstabenpixels
.const cb       = $cc  ; aktueller Ball (Schlitz 0..2), dessen Werte in bxl..vyh stehen
.const nact     = $cd  ; Anzahl aktiver Baelle in diesem Bild
.const sprmsb   = $ce  ; (ungenutzt; C64: gesammeltes 9. X-Bit der Sprites)
.const sprena   = $cf  ; (ungenutzt; C64: gesammelte Sprite-Freigabebits)
.const tlas     = $d0  ; Rest-Halbsekunden: Laser aktiv
.const shi      = $d1  ; update_shots: Schussnummer (ueberlebt hit_brick)
.const wfc      = $d2  ; wait_frames: Zaehler
.const bflash   = $d3  ; Rest-Bilder des Randblitzens (0 = kein Blitz)
.const glg      = $d4  ; (ungenutzt; das Glanzlicht laeuft ueber glt/glr)
.const scx      = $d5  ; (ungenutzt; der Lauftext benutzt hsc)
.const stp      = $d6  ; (ungenutzt; die Leseposition im Lauftext steht im Zeiger stq)
.const lph      = $d7  ; Titellogo: Farbphase (title_fx zaehlt sie hoch, color_rows wertet sie nicht aus)
.const lcnt     = $d8  ; Titellogo: Bildzaehler fuer den Farbwechsel (alle 6 Bilder)
.const nch      = $d9  ; Namenseingabe: getipptes Zeichen dieses Bildes (0 = keins)
.const ntyped   = $da  ;   1 = es wurde schon getippt (die Vorgabe ist geloescht)
.const capfr    = $db  ; Kapsel: Drehphase (Bildzaehler 0..23, je 4 Bilder eine Phase)
.const lkind    = $dc  ; Levelart: 0 normal, 1 Bonus, 2 Boss
.const btime    = $dd  ; Bonuslevel: Restsekunden
.const bsub     = $de  ; Halbsekundenzaehler fuer die Sekunden
.const cores    = $df  ; Bosslevel: verbliebene Kerne
.const batk     = $e0  ; Bosslevel: Halbsekunden bis zum naechsten Angriff
.const capforce = $e1  ; spawn_cap: erzwungene Kapselart + 1 (0 = zufaellig)
.const tkrow    = $e2  ; Ticker: Bildschirmzeile
.const tkst     = $e3  ;         0 = warten, 1 = Effekt laeuft
.const tkw      = $e4  ;         Wartebilder
.const tkeff    = $e5  ;         Effekt 0..6
.const tkstep   = $e6  ;         Schritt
.const tkmsg    = $e7  ;         Index in der Textfolge
.const tkseq    = $e8  ;         Folge 0 = Titel, 1 = Demo
.const tkph     = $e9  ;         Farbphase (zaehlt Bilder)
.const tkc      = $ea
.const tkr      = $eb
.const tkd      = $ec
.const tkt      = $ed
.const tks      = $ee
.const tklf     = $ef
.const tkdiv    = $9a00                  ; Ticker: Teilschritt je Bild (Viertel bzw. Haelfte der Arbeit eines Effektschritts)
.const tkd2     = $9a01                  ; Ticker: Hilfsspeicher von tk_hright
.const tpierce  = $9a02  ; Rest-Halbsekunden: Durchschlag aktiv (Ball zerstoert jeden zerstoerbaren Stein und prallt nicht ab)
.const tkce     = $9a03  ; tk_vert: Endzeichen (exklusiv) dieses Durchlaufs
.const tkre     = $9a04  ; tk_hleft/tk_hright: Endzeile (exklusiv) dieses Durchlaufs
.const tkrs     = $9a05  ; Startzeile
.const sp       = $80                    ; Zeiger (2 Byte, Nullseite)
.const tp       = $82                    ; Zeiger (2 Byte, Nullseite)
.const tka      = $84                    ; Zeiger (2 Byte, Nullseite)
.const tkb      = $86                    ; Zeiger (2 Byte, Nullseite)
.const stq      = $88                    ; Zeiger (2 Byte, Nullseite)
.const mdp      = $8a                    ; Zeiger (2 Byte, Nullseite)
; ---------------------------------------------------------------------------------------------------------------------
; Atari-Register und Speicherplan
; ---------------------------------------------------------------------------------------------------------------------
.const SDMCTL  = $022f                  ; Schattenregister (vom OS im Bildruecklauf in die Chips kopiert): DMA-Steuerung
.const SDLSTL  = $0230                  ; Adresse der Display-Liste (2 Byte)
.const GPRIOR  = $026f                  ; Prioritaet
.const PCOLR0  = $02c0                  ; Spielerfarben 0..3 ($02C0-$02C3)
.const COLOR0  = $02c4                  ; COLPF0..3 ($02C4-$02C7), COLBK = $02C8
.const COLOR4  = $02c8                  ; COLBK-Schatten (hier nicht benutzt; COLBK schreibt der DLI)
.const CHBAS   = $02f4                  ; Seite des Zeichensatzes
.const VDSLST  = $0200                  ; Vektor des Display-List-Interrupts
.const RTCLOK  = $14                    ; niedrigstes Byte der OS-Systemuhr ($12-$14): zaehlt im Bildruecklauf hoch (50 Hz PAL)
.const SETVBV  = $e45c                  ; OS: Vektor setzen (A = 7: verzoegerter Bildruecklauf)
.const XITVBV  = $e462                  ; Ende eines verzoegerten Bildruecklauf-Interrupts
.const ATRACT  = $4d                    ; Bildschirmschoner (Attract-Modus) des OS: zaehlt hoch, dimmt die Farben; wir setzen ihn laufend auf 0
.const HPOSP0  = $d000                  ; horizontale Lage Spieler 0..3
.const HPOSP1  = $d001
.const HPOSP2  = $d002
.const HPOSM0  = $d004                  ; horizontale Lage Missiles 0..3
.const SIZEP0  = $d008                  ; Breite Spieler 0..3 (0 = normal)
.const SIZEM   = $d00c                  ; Breite Missiles (2 Bit je Missile)
.const COLPF0  = $d016
.const COLPF1  = $d017
.const COLPF2  = $d018
.const COLPF3  = $d019
.const COLBK   = $d01a
.const GRACTL  = $d01d
.const HSCROL  = $d404
.const PMBASE  = $d407
.const WSYNC   = $d40a
.const NMIEN   = $d40e

.const SCR     = $8000                  ; Bildschirmspeicher 24 * 40 Byte (liegt komplett in einem 4-KB-Block)
.const DLG     = $8400                  ; Display-Liste Spiel
.const DLT     = $8440                  ; Display-Liste Titel
.const DLO     = $8480                  ; Display-Liste Spielende/Namenseingabe
.const CHARSET = $8800                  ; Zeichensatz (1 KB, auf 1-KB-Grenze)
.const PM_M    = $9300                  ; Missiles (Einzelzeile): 1 Byte je Zeile, 4 Missiles zu je 2 Bit
.const PM_P0   = $9400                  ; Spieler 0..3: je 256 Byte
.const PM_P1   = $9500
.const PM_P2   = $9600
.const PM_P3   = $9700
.const MAP     = $9800                  ; Steinkarte (216 Byte)
.const bact    = $9900                  ; Ballspeicher: 3 Byte je Eigenschaft
.const b_xl    = $9903
.const b_xh    = $9906
.const b_yl    = $9909
.const b_yh    = $990c
.const b_vxl   = $990f
.const b_vxh   = $9912
.const b_vyl   = $9915
.const b_vyh   = $9918
.const shact   = $9920                  ; Laserschuesse: 1 = aktiv
.const shu     = $9922                  ;   X in Einheiten zu 2 px
.const shy     = $9924                  ;   Y (Oberkante)
.const tk_cur  = $9940                  ; Ticker: angezeigter Text (32 Zeichen)
.const tk_new  = $9960                  ; Ticker: Zieltext
.const tby     = $9983                  ;                    Y in Zeilen
.const tbdu    = $9986                  ;                    Schritt X (1 oder $FF)
.const tbdy    = $9989                  ;                    Schritt Y
.const hsc     = $9b90                  ; Lauftext: Feinversatz (HSCROL) 3..0
.const dl_t2   = $9b95                  ; DLI: Zwischenspeicher fuer RC_H (Feinscroll-Wert)
.const RC_H    = $9b98                  ; Feinscroll-Wert (HSCROL) je Textzeile (24 Byte)
.const LOGOBUF = $8600                  ; Logozeilen 3..7 mit Feinscrollen: je 48 Zeichen
.const dl_lim  = $9b94                  ; build_dl: Anzahl Zeilen mit DLI-Bit (Zeilen 0 .. dl_lim - 1)
.const COLPM0  = $d012
.const COLPM1  = $d013
.const COLPM2  = $d014
.const dl_sr   = $9b91                  ; build_dl: Zeile mit Feinscrollen (255 = keine)
.const glt     = $9b92                  ; Glanzlicht: Bildteiler
.const glr     = $9b93                  ; Glanzlicht: laufende Steinzeile
.const SCRL    = $8500                  ; Lauftextzeile (48 Zeichen: Feinscrollen holt eine breitere Zeile)
.const tbu     = $9980                  ; Titelbaelle: X in Einheiten zu 2 px (3 Byte), danach tby, tbdu, tbdy je 3 Byte
.const RC_BK   = $9b00                  ; Farbtabellen je Textzeile (24 Byte je Register)
.const RC_0    = $9b18
.const RC_1    = $9b30
.const RC_2    = $9b48
.const RC_3    = $9b60
.const old_py  = $9b80                  ; Ballsprites: zuletzt gezeichnete Zeile (3 Byte)
.const old_cy  = $9b83                  ; Kapsel: zuletzt gezeichnete Zeile
.const old_cf  = $9b84                  ; (ungenutzt) Kapsel: zuletzt gezeichnete Phase
.const old_sy  = $9b85                  ; Schuss: zuletzt gezeichnete Zeile
.const dli_i   = $9b86                  ; naechste Tabellenzeile des DLI
.const dl_t0   = $9b87                  ; DLI: Zwischenspeicher fuer RC_0 (COLPF0 der folgenden Zeile)
.const dl_t1   = $9b88                  ; DLI: Zwischenspeicher fuer RC_1; build_dl benutzt ihn als Listenversatz (nie gleichzeitig mit einem DLI)
.const cur_mode = $9b89                 ; 0 Spiel, 1 Titel, 2 Spielende
.const vblank  = $9b8a                  ; vbi setzt 1 (wird nirgends ausgewertet)
.const band    = $9b8b                  ; (ungenutzt)
.const hs_top  = $9b8c                  ; (ungenutzt)

.const PADY    = 184                    ; obere Kante des Schlaegers in px
.const PADW    = 32
.const LIVES0  = 3
.const CH_SPACE = 0                     ; Schrift: Bildschirmcodes des Atari: 0 = Leerzeichen, 16 = '0', 33 = 'A' (= ASCII - 32)
.const CH_DIGIT0 = 16
.const CH_SKY    = 96                   ; Spielfeldzeichen (Modus 4): Himmel (Pixelwert 01 = COLPF0)
.const CH_BRICK_L = 97
.const CH_BRICK_R = 98
.const CH_METAL_L = 99                  ; +$80: Pixelwert 11 = COLPF3 statt COLPF2
.const CH_METAL_R = 100
.const CH_HARD_L  = 101
.const CH_HARD_R  = 102
.const CH_WALL    = 103
.const CH_BLOCK   = 104
.const CH_BLACK   = 105                 ; ganz schwarz (Pixelwert 00 = COLBK)
.const TK_BUF = CHARSET + 64 * 8        ; Bitmuster der Zeichen 64..95 (Ticker) im Zeichensatz
.const TK_CHR = 64                      ; erstes Ticker-Zeichen (Modus-2-Zeilen)

; Platzhalter, damit die uebernommene Spiellogik die C64-Namen unveraendert benutzen kann (alle Anzeigeroutinen sind unten neu fuer den Atari).
; Die beiden Zellen sind gewoehnliche Speicherstellen: sie werden beschrieben, aber nie gelesen, und haben keinen Einfluss auf die Hardware.
.const VIC_SPR_ENABLE = $9b8d
.const VIC_BORDERCOLOR = $9b8e

.org $2000
.include "lib/gfx.inc.chars"            ; gfx_chars (Spielfeldzeichen, 8 * 10 Byte)

; ---------------------------------------------------------------------------------------------------------------------
; Programmstart
; ---------------------------------------------------------------------------------------------------------------------
; start: einmalige Einrichtung (Chips, Klang, Eingabe, Zufall, Bestenliste), dann die Hauptschleife main -> title -> Spiel.
start:  jsr hal_init
        jsr sfx_init
        jsr input_init
        lda $d20a                       ; POKEY RANDOM lesen (rand_seed ignoriert den Wert, die Bibliothek liest VCOUNT)
        jsr rand_seed
        jsr hs_dev_init                 ; Bestenliste auf das Laufwerk, von dem das Spiel gestartet wurde
        jsr hs_default
.ifndef NOHS
        jsr hs_load                     ; Bestenliste (Diskette), bei Fehlern bleibt die Standardliste
.endif
main:   jsr title                       ; kehrt zurueck, wenn Feuer gedrueckt oder die Demo gestartet wird
        jsr new_game
.level: jsr load_level
.ball:  jsr play                        ; A: 0 = Level geschafft, 1 = Ball verloren, 2 = Abbruch
        cmp #2
        beq main
        cmp #1
        beq .lost
        lda demo
        beq +
        lda level
        cmp #4
        bcs main
+       inc level                       ; naechstes Level; nach dem letzten wieder von vorn (Spiel und Demo)
        lda level
        cmp #level_count
        bcc .level
        lda #0
        sta level
        beq .level
.lost:  lda lkind
        cmp #1
        bne .loselife
        jsr place_ball                  ; Bonuslevel: ein verlorener Ball kostet kein Leben
        jmp .ball
.loselife:
        dec lives
        beq .over
        jsr reset_extras
        jsr place_ball
        jsr draw_hud
        jmp .ball
.over:  jsr game_over
        jmp main

; hal_init: Chips einrichten.  Der Bildschirm bleibt aus (SDMCTL = 0), bis die erste Display-Liste steht.
hal_init:
        lda #0
        sta SDMCTL
        sta NMIEN                       ; keine DLIs/VBIs, solange wir umbauen
        sta vblank
        ; Zeichensatz: die ersten 64 Zeichen (Schrift) aus dem ROM ($E000), Spielfeldzeichen ab Code 96
        ldx #0
.hi1:   lda $e000,x
        sta CHARSET,x
        lda $e100,x
        sta CHARSET + $100,x
        inx
        bne .hi1                        ; 2 * 256 Byte = Codes 0..63
        ldx #0
.hi2:   lda gfx_chars,x
        sta CHARSET + 96 * 8,x          ; Codes 96..: 10 Zeichen = 80 Byte
        inx
        cpx #80
        bne .hi2
        lda #>CHARSET
        sta CHBAS
        lda #>(PM_M - $300)
        sta PMBASE                      ; Spielerspeicher: Seite $90
        lda #3
        sta GRACTL                      ; Spieler und Missiles freigeben
        lda #$01
        sta GPRIOR                      ; Spieler und Missiles VOR dem Spielfeld (Bit 0); Missiles haben die Farbe ihres Spielers
        lda #$0e
        sta PCOLR0                      ; Schlaeger weiss
        sta PCOLR0 + 1
        sta PCOLR0 + 2
        lda #$1e
        sta PCOLR0 + 3                  ; Kapsel: Farbe je Art (siehe spawn_cap)
        lda #1
        sta SIZEM                       ; Missile 0 doppelt breit (4 Farbtakte) fuer den Ball; die anderen normal
        jsr clear_pm
        jsr build_dls
        lda #<dli
        sta VDSLST
        lda #>dli
        sta VDSLST + 1
        lda #7
        ldx #>vbi
        ldy #<vbi
        jsr SETVBV                      ; verzoegerter Bildruecklauf-Interrupt
        lda #$c0
        sta NMIEN                       ; DLI und VBI an
        rts

; clear_pm: Spielerspeicher (Missiles + 4 Spieler) loeschen.
clear_pm:
        lda #0
        tax
.cp1:   sta PM_M,x
        sta PM_P0,x
        sta PM_P1,x
        sta PM_P2,x
        sta PM_P3,x
        inx
        bne .cp1
        sta old_py
        sta old_py + 1
        sta old_py + 2
        sta old_cy
        sta old_sy
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Display-Listen.  ANTIC-Befehle: $70 = 8 leere Zeilen, $42/$44 = Textmodus 2/4 mit Adresse (LMS), $02/$04 = Textmodus 2/4, +$80 = DLI am Zeilenende,
; $41 = Sprung zum Anfang und auf das Bildende warten.  Jede Liste: 3 x 8 leere Zeilen (24 Zeilen Oberrand), 24 Textzeilen, Sprung.
; Modus je Zeile steht in den Tabellen modes_game / modes_title / modes_over (2 oder 4).
; ---------------------------------------------------------------------------------------------------------------------
modes_game:  .byte 2, 4, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4
modes_title: .byte 2, 2, 2, 4, 4, 4, 4, 4, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2
modes_over:  .byte 2, 4, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 2, 2, 2, 2

; Display-Liste des Titels (fertig als Daten): 3 x 8 leere Zeilen; Zeile 0: Modus 2 + LMS + DLI; Zeilen 1, 2: Modus 2 + DLI; Zeilen 3..7 (Logo): Modus 4 + LMS + Feinscrollen + DLI,
;   je Zeile eigene Adresse in LOGOBUF (48 Zeichen, damit das Feinscrollen eine breitere Zeile holen kann); Zeilen 8..22: Modus 2 + DLI; Zeile 23: Modus 2 + LMS + Feinscrollen
;   (Lauftext, Adresse SCRL), kein DLI; Sprung zum Listenanfang.
dlt_data:
        .byte $70, $70, $70
        .byte $c2, <SCR, >SCR
        .byte $82, $82
        .byte $d4, <LOGOBUF, >LOGOBUF
        .byte $d4, <(LOGOBUF + 48), >(LOGOBUF + 48)
        .byte $d4, <(LOGOBUF + 96), >(LOGOBUF + 96)
        .byte $d4, <(LOGOBUF + 144), >(LOGOBUF + 144)
        .byte $d4, <(LOGOBUF + 192), >(LOGOBUF + 192)
        .byte $c2, <(SCR + 320), >(SCR + 320)   ; Zeile 8: Textzeilen wieder aus dem Bildschirmspeicher (der Abtastzaehler laeuft sonst hinter LOGOBUF weiter)
        .fill 14, $82
        .byte $52, <SCRL, >SCRL
        .byte $41, <DLT, >DLT
dlt_end:                                ; Ende der Titel-Liste (die Laenge dlt_end - dlt_data wird nach DLT kopiert)

; build_dls: baut die drei Display-Listen: DLG (Spiel) und DLO (Spielende) mit build_dl, DLT (Titel) aus den fertigen Daten dlt_data.
;   Setzt dl_sr (Feinscroll-Zeile, 255 = keine) und dl_lim (Anzahl Zeilen mit DLI-Bit) fuer build_dl.  Veraendert A, X, Y, sp, mdp.
build_dls:
        lda #255
        sta dl_sr                       ; Spiel und Spielende: keine Feinscroll-Zeile
        lda #18
        sta dl_lim                      ; Spiel: DLI in den Zeilen 0..17; ab Zeile 18 uebernimmt der Rasterzeilen-Kernel (siehe dli)
        lda #<modes_game
        sta mdp
        lda #>modes_game
        sta mdp + 1
        lda #<DLG
        sta sp
        lda #>DLG
        sta sp + 1
        jsr build_dl
        lda #<modes_title               ; (Modustabelle fuer clear_screen/color_rows; die Display-Liste selbst steht fertig in dlt_data)
        sta mdp
        lda #>modes_title
        sta mdp + 1
        ldx #dlt_end - dlt_data - 1
.bt1:   lda dlt_data,x
        sta DLT,x
        dex
        bpl .bt1
        lda #255
        sta dl_sr
        lda #18
        sta dl_lim
        lda #<modes_over
        sta mdp
        lda #>modes_over
        sta mdp + 1
        lda #<DLO
        sta sp
        lda #>DLO
        sta sp + 1
        jmp build_dl

; build_dl: Display-Liste nach (sp) aus der Modustabelle (mdp).  Zeile 0 bekommt LMS (Bildschirmadresse), die Zeilen 0 .. dl_lim - 1 das DLI-Bit,
;   Zeile dl_sr (falls < 24) wird zur Lauftextzeile mit Feinscrollen und eigener Adresse SCRL.  Veraendert A, X, Y, dl_t1.
build_dl:
        ldy #0
        lda #$70
        sta (sp),y
        iny
        sta (sp),y
        iny
        sta (sp),y
        iny                             ; Y = Listenversatz (3)
        ldx #0                          ; X = Textzeile
.bd1:   sty dl_t1
        txa
        tay
        lda (mdp),y                     ; Modus 2 oder 4
        ldy dl_t1
        cpx dl_sr
        bne .bd0
        lda #$52                        ; Lauftextzeile: Modus 2 + LMS ($40) + Feinscrollen ($10), eigene Adresse SCRL (48 Zeichen)
        sta (sp),y
        iny
        lda #<SCRL
        sta (sp),y
        iny
        lda #>SCRL
        sta (sp),y
        iny
        jmp .bd3
.bd0:   cpx dl_lim
        bcs +
        ora #$80                        ; DLI-Bit nur in den Zeilen 0 .. dl_lim - 1
+       cpx #0
        bne .bd2
        ora #$40                        ; Zeile 0: LMS, es folgt die Bildschirmadresse
        sta (sp),y
        iny
        lda #<SCR
        sta (sp),y
        iny
        lda #>SCR
        sta (sp),y
        iny
        jmp .bd3
.bd2:   sta (sp),y
        iny
.bd3:   inx
        cpx #24
        bne .bd1
        lda #$41                        ; Sprung zum Anfang der Liste und auf das Bildende warten
        sta (sp),y
        iny
        lda sp
        sta (sp),y
        iny
        lda sp + 1
        sta (sp),y
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Interrupts
; ---------------------------------------------------------------------------------------------------------------------
; DLI (am Ende jeder Textzeile mit DLI-Bit): Farbregister der FOLGENDEN Zeile aus den Tabellen setzen.  Sichert A, X, Y; dli_i = Index dieser Zeile.  WSYNC wartet bis zum Zeilenende; danach nur noch die
;   Schreibzugriffe (Werte vorher geholt), damit die neue Zeile gleich mit den neuen Farben beginnt.
dli:    pha
        txa
        pha
        tya
        pha
        ldx dli_i
        lda RC_H,x
        sta dl_t2
        lda RC_0,x
        sta dl_t0
        lda RC_1,x
        sta dl_t1
        lda RC_2,x
        tay
        cpx #23                         ; Zeile 23 = Lauftext mit Feinscrollen: ihren HSCROL-Wert SCHON JETZT setzen (in der letzten Rasterzeile der Zeile 22,
        bne .dl1                        ; die selbst nicht scrollt): ANTIC liest HSCROL zu Beginn jeder Rasterzeile; ein Schreiben erst nach WSYNC kommt fuer die
        lda dl_t2                       ; ersten Rasterzeilen der Zeile 23 zu spaet (sie scrollten dann nicht weich)
        sta HSCROL
.dl1:   lda RC_3,x
        sta WSYNC
        sta COLPF3
        sty COLPF2
        lda dl_t1
        sta COLPF1
        lda dl_t0
        sta COLPF0
        lda RC_BK,x
        sta COLBK
        lda dl_t2
        sta HSCROL                      ; Feinscroll-Wert der folgenden Zeile (nur Zeilen mit Scroll-Bit in der Display-Liste)
        inc dli_i
        cpx #18                         ; Farben der Zeile 18 sind gesetzt: im Spiel laeuft ab hier der Rasterzeilen-Kernel
        bne .dlx
        lda cur_mode
        bne .dlx
        jsr kernel
.dlx:   pla
        tay
        pla
        tax
        pla
        rti

; kernel: Rasterzeilen-Kernel fuer die Zeilen 18..23 (47 Rasterzeilen).  Je Rasterzeile (nach WSYNC) wird die Himmelsfarbe COLPF0 aus kgrad gesetzt (weicher Verlauf
;   statt einer Farbe je Textzeile) und die Farbe der Spieler 0-2 aus kpm: der Schlaeger (Rasterzeilen 40..45 ab Zeile 18) bekommt einen Metallglanz (oben hell, unten dunkel);
;   die Baelle (Missiles) tragen die Farbe ihres Spielers und blitzen beim Durchfliegen mit.  Die CPU ist waehrend dieser Zeilen voll beschaeftigt (rund 47 * 114 Takte).
;   Die Tabelle fuer das aktuelle Thema holt color_rows in kgrad (47 Byte).
kernel:
        ldx #0
.kn1:   lda kgrad,x
        sta WSYNC
        sta COLPF0
        lda kpm,x
        sta COLPM0
        sta COLPM1
        sta COLPM2
        inx
        cpx #47
        bne .kn1
        rts
; Spielerfarben je Rasterzeile im Kernel: weiss, in den Schlaegerzeilen ein blauer Metallverlauf (Zeile x + 1 des Kernels = Rasterzeile; Schlaeger = Zeilen 40..45)
; (47 Eintraege: 32 weiss, 6 Schlaegerzeilen blau, 9 weiss)
kpm:    .fill 32, $0e
        .byte $9e, $9c, $98, $96, $94, $92
        .fill 9, $0e

; VBI (verzoegert): Zeile 0 direkt in die Chips, Zaehler zuruecksetzen, Bildschirmschoner abstellen, Bildtakt.
vbi:    lda #1
        sta dli_i
        lda RC_H
        sta HSCROL                      ; Feinscroll-Wert der Zeile 0
        lda #0
        sta ATRACT
        lda RC_0
        sta COLPF0
        lda RC_1
        sta COLPF1
        lda RC_2
        sta COLPF2
        lda RC_3
        sta COLPF3
        lda RC_BK
        sta COLBK
        lda #1
        sta vblank
        jmp XITVBV

; wait_frame: auf das naechste Bild warten (RTCLOK zaehlt in jedem Bildruecklauf).
wait_frame:
        lda RTCLOK
.wf1:   cmp RTCLOK
        beq .wf1
        rts

; wait_frames: X Bilder warten; dabei laeuft der Klangeffekt weiter (Zaehler im Speicher, weil sfx_update X veraendert).
wait_frames:
        stx wfc
.wfl:   jsr wait_frame
        jsr sfx_update
        dec wfc
        bne .wfl
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Bildschirm-Hilfen
; ---------------------------------------------------------------------------------------------------------------------
; Zeilenadressen: SCR + 40 * Zeile
rowlo:  .repeat 24, i
        .byte <(SCR + 40 * i)
        .endrepeat
rowhi:  .repeat 24, i
        .byte >(SCR + 40 * i)
        .endrepeat

; set_rowptr: sp = Adresse der Zelle (Spalte X, Zeile Y).   Veraendert A.
set_rowptr:
        txa
        clc
        adc rowlo,y
        sta sp
        lda rowhi,y
        adc #0
        sta sp + 1
        rts

; put_cell: Zeichen A an Spalte X, Zeile Y.   Veraendert A, X, Y.
put_cell:
        pha
        jsr set_rowptr
        ldy #0
        pla
        sta (sp),y
        rts

; puts: Text ab (tp), mit $FF abgeschlossen, an Spalte X, Zeile Y (Modus-2-Zeilen).
puts:   stx t2
        sty t3
        ldy #0
        sty t1
.pl:    ldy t1
        lda (tp),y
        cmp #$ff                        ; Ende (Zeichen 0 ist das Leerzeichen, deshalb $FF)
        beq .pd
        ldx t2
        ldy t3
        jsr put_cell
        inc t2
        inc t1
        bne .pl
.pd:    rts

; put_bcd: BCD-Byte als zwei Ziffern an Spalte X, Zeile Y (Ziffer n = Zeichen 16 + n).
put_bcd:
        sta t0
        stx t2
        sty t3
        lsr
        lsr
        lsr
        lsr
        clc
        adc #CH_DIGIT0
        jsr put_cell
        lda t0
        and #$0f
        clc
        adc #CH_DIGIT0
        ldx t2
        inx
        ldy t3
        jmp put_cell

; clear_screen: alle 24 Zeilen fuellen: Modus-4-Zeilen mit dem Himmelszeichen, Modus-2-Zeilen mit Leerzeichen (Tabelle in mdp).
clear_screen:
        lda #0
        sta t1                          ; Zeile
.cs1:   ldy t1
        lda (mdp),y
        cmp #4
        beq .cs2
        lda #CH_SPACE
        jmp .cs3                        ; (kein bne: Zeichen 0 setzt das Z-Flag)
.cs2:   lda #CH_SKY
.cs3:   pha
        ldx #0
        jsr set_rowptr                  ; X = 0, Y = Zeile
        pla
        ldy #39
.cs4:   sta (sp),y
        dey
        bpl .cs4
        inc t1
        lda t1
        cmp #24
        bne .cs1
        rts

; use_mode: A = 0 Spiel, 1 Titel, 2 Spielende: Display-Liste und Modustabelle waehlen.  (Bildschirm kurz aus, damit kein halbes Bild zu sehen ist.)
use_mode:
        sta cur_mode
        tax
        lda mode_lo,x
        sta mdp
        lda mode_hi,x
        sta mdp + 1
        lda dl_lo,x
        sta SDLSTL
        lda dl_hi,x
        sta SDLSTL + 1
        lda #$3e
        sta SDMCTL                      ; Display-Liste, Spieler, Missiles, Einzelzeile, normale Breite
        rts
mode_lo: .byte <modes_game, <modes_title, <modes_over     ; Modustabelle je Anzeigeart (Index = cur_mode), Low-/High-Byte
mode_hi: .byte >modes_game, >modes_title, >modes_over
dl_lo:   .byte <DLG, <DLT, <DLO                           ; Display-Liste je Anzeigeart, Low-/High-Byte
dl_hi:   .byte >DLG, >DLT, >DLO

; set_theme: A = Verlauf 0..3 fuer die Himmelsfarbe (PF0) je Zeile; ausserdem Steinfarben je Zeile aus dem Level (siehe color_rows).
set_theme:
        and #3
        sta theme_n
        jmp color_rows
theme_n: .byte 0                        ; aktuelles Thema 0..3 (Level and 3)

; color_rows: Farbtabellen RC_* fuer alle 24 Zeilen (Modus in mdp):
;   Modus-2-Zeilen: PF2 = Hintergrund schwarz, PF1 = Schrift weiss.
;   Modus-4-Zeilen: PF0 = Himmel (Verlauf), PF1 = Steinfarbe je Zeile (Regenbogen, verschoben um das Level), PF2 = Lichtkante (heller), PF3 = Metall
;   (Silber; Gold in Zeilen mit Goldsteinen).  COLBK bleibt schwarz.
color_rows:
        jsr kgrad_set
        lda theme_n
        asl
        asl
        asl
        sta t0
        asl
        clc
        adc t0                          ; theme * 24 = theme * 8 * 3
        sta t0
        lda #0
        sta t1                          ; Zeile
.cr1:   ldy t1
        lda #0
        sta RC_BK,y
        sta RC_H,y
        lda (mdp),y
        cmp #4
        beq .cr4
        lda #$00                        ; Modus 2: PF2 (Grund) schwarz, PF1 (Schrift) weiss
        sta RC_2,y
        sta RC_0,y
        sta RC_3,y
        lda #$0e
        sta RC_1,y
        jmp .crn
.cr4:   lda t0
        clc
        adc t1
        tax
        lda sky_tab,x                   ; Himmelsfarbe dieser Zeile
        sta RC_0,y
        lda #$0a
        sta RC_3,y                      ; Metall: Silber
        cpy #4
        bcc .cr5
        cpy #16
        bcs .cr5
        tya                             ; Steinzeilen 4..15: Regenbogenfarbe
        clc
        adc level
        and #7
        tax
        lda hue_body,x
        sta RC_1,y
        lda hue_hl,x
        sta RC_2,y
        jmp .crn
.cr5:   lda #$0c                        ; andere Zeilen (Wand, Himmel): graue Steine
        sta RC_1,y
        lda #$0e
        sta RC_2,y
.crn:   inc t1
        lda t1
        cmp #24
        bne .cr1
        lda kgrad                       ; Zeile 18 (erste Zeile des Rasterzeilen-Kernels) beginnt mit dem ersten Wert des Kernel-Verlaufs
        sta RC_0 + 18
        rts

; kgrad_set: Himmelsverlauf der Rasterzeilen-Zone fuer das Thema theme_n nach kgrad kopieren.
kgrad_set:
        lda theme_n
        tax
        lda #0
        cpx #0
        beq .kg2
.kg1:   clc
        adc #47
        dex
        bne .kg1
.kg2:   tax
        ldy #0
.kg3:   lda kgrad_tab,x
        sta kgrad,y
        inx
        iny
        cpy #47
        bne .kg3
        rts
kgrad:  .fill 47, 0                     ; Himmelsverlauf der Rasterzeilen-Zone (COLPF0 je Rasterzeile), von kgrad_set gefuellt
; kgrad_tab: Himmelsverlaeufe des Kernels, 4 Themen (Daemmerung, Wald, Glut, Nacht) zu je 47 Byte (eine Farbe je Rasterzeile der Zeilen 18..23)
kgrad_tab:
        .byte $52, $52, $52, $52, $52, $52, $52, $52, $52, $52, $52, $52, $44, $44, $44, $44
        .byte $44, $44, $44, $44, $44, $44, $44, $44, $44, $44, $44, $44, $44, $44, $44, $44
        .byte $44, $44, $44, $36, $36, $36, $36, $36, $36, $36, $36, $36, $36, $36, $36
        .byte $22, $22, $22, $22, $22, $22, $22, $22, $22, $22, $22, $22, $24, $24, $24, $24
        .byte $24, $24, $24, $24, $24, $24, $24, $24, $24, $24, $24, $24, $24, $24, $24, $24
        .byte $24, $24, $24, $26, $26, $26, $26, $26, $26, $26, $26, $26, $26, $26, $26
        .byte $12, $12, $12, $12, $12, $12, $14, $14, $14, $14, $14, $14, $14, $14, $14, $14
        .byte $14, $14, $16, $16, $16, $16, $16, $16, $16, $16, $16, $16, $16, $18, $18, $18
        .byte $18, $18, $18, $18, $18, $18, $18, $18, $18, $1a, $1a, $1a, $1a, $1a, $1a
        .byte $02, $02, $02, $02, $00, $00, $00, $00, $02, $02, $02, $02, $00, $00, $00, $00
        .byte $02, $02, $02, $02, $00, $00, $00, $00, $02, $02, $02, $02, $00, $00, $00, $00
        .byte $02, $02, $02, $02, $00, $00, $00, $00, $02, $02, $02, $02, $00, $00, $00

; Regenbogen: Farbtoene rot, orange, gelb, gruen, tuerkis, blau, violett, rosa (Atari-Farbwert = Farbton * 16 + Helligkeit)
hue_body: .byte $38, $28, $1a, $c8, $a8, $88, $58, $48
hue_hl:   .byte $3e, $2e, $1e, $ce, $ae, $8e, $5e, $4e
; Himmelsverlaeufe (4 Themen * 24 Zeilen): Daemmerung, Wald, Glut, Nacht; Helligkeit niedrig, damit die Steine hervortreten
sky_tab:
        .byte $00,$00,$00,$00,$00,$82,$82,$82,$82,$82,$82,$82,$52,$52,$52,$52,$52,$52,$32,$32,$32,$32,$32,$32
        .byte $00,$00,$00,$00,$00,$02,$02,$02,$c2,$c2,$c2,$c2,$c2,$c2,$c2,$22,$22,$22,$22,$22,$22,$22,$22,$00
        .byte $00,$00,$00,$00,$00,$22,$22,$22,$32,$32,$32,$32,$32,$32,$12,$12,$12,$12,$12,$12,$12,$12,$12,$12
        .byte $00,$00,$00,$00,$00,$02,$02,$00,$00,$02,$02,$00,$00,$02,$02,$00,$00,$02,$02,$00,$00,$02,$02,$00

; flash: Randblitz (Atari: das ganze Hintergrundschwarz/die Raender werden kurz farbig).  A = COLBK-Wert (Atari-Farbwert!), X = Bilder.
;   Die Aufrufer uebergeben Atari-Farbwerte ($1E gelb, $34 rot, $0E weiss).  Veraendert A, X, bflash, RC_BK.
flash:  sta flash_col
        stx bflash
        ldx #23
.fl1:   sta RC_BK,x
        dex
        bpl .fl1
        rts
flash_col: .byte 0                      ; zuletzt gewaehlte Blitzfarbe (nur geschrieben)

; fx_frame: Randblitz herunterzaehlen und danach COLBK wieder auf Schwarz (Spielfeld: Himmel steckt in den Zeichen).
fx_frame:
        jsr glint
        lda bflash
        beq .fx9
        dec bflash
        bne .fx9
        ldx #23
        lda #0
.fx1:   sta RC_BK,x
        dex
        bpl .fx1
.fx9:   rts

; glint: Glanzlicht - die Lichtkante (COLPF2) je Steinzeile blitzt nacheinander von oben nach unten weiss auf (alle 3 Bilder ein Schritt, dann Pause).
glint:
        lda cur_mode
        bne .gl9                        ; nur im Spiel
        inc glt
        lda glt
        cmp #3
        bcc .gl9
        lda #0
        sta glt
        ldx glr                         ; vorige Zeile in ihre Farbe zurueck
        cpx #4
        bcc .gl1
        cpx #16
        bcs .gl1
        txa
        clc
        adc level
        and #7
        tay
        lda hue_hl,y
        sta RC_2,x
.gl1:   inc glr
        lda glr
        cmp #34
        bcc .gl2
        lda #0
        sta glr
.gl2:   ldx glr
        cpx #4
        bcc .gl9
        cpx #16
        bcs .gl9
        lda #$0f
        sta RC_2,x
.gl9:   rts

; draw_walls: obere Wand (Zeile 1) und Seitenwaende (Spalten 1 und 38); Spalten 0 und 39 schwarz (Rand), Metallfarbe PF3 = Bit 7 des Zeichencodes.
draw_walls:
        lda #1
        sta t1
.dw1:   ldx t1
        ldy #1
        lda #CH_WALL | $80
        jsr put_cell
        inc t1
        lda t1
        cmp #39
        bne .dw1
        lda #3
        sta t1
.dw2:   ldx #1
        ldy t1
        lda #CH_WALL | $80
        jsr put_cell
        ldx #38
        ldy t1
        lda #CH_WALL | $80
        jsr put_cell
        inc t1
        lda t1
        cmp #24
        bne .dw2
        lda #1
        sta t1
.dw3:   ldx #0
        ldy t1
        lda #CH_BLACK
        jsr put_cell
        ldx #39
        ldy t1
        lda #CH_BLACK
        jsr put_cell
        inc t1
        lda t1
        cmp #24
        bne .dw3
        rts

; draw_brick: Stein vom Typ A an (bcol, brow) zeichnen.  Zeichenpaar je nach Typ: leer = Himmel; 1..8, 12, 13 = Stein; 9, 10 = Metall (Bit 7 = PF3);
;   11 = unzerstoerbar (schwarz mit heller Kante); 16..23 = Boss-Kern (Metall).
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
        ldy #0
        lda t0
        beq .db0
        cmp #9
        beq .dm
        cmp #10
        beq .dm
        cmp #11
        beq .dh
        cmp #16
        bcs .dm
        lda #CH_BRICK_L
        sta (sp),y
        iny
        lda #CH_BRICK_R
        sta (sp),y
        rts
.dm:    lda #CH_METAL_L | $80
        sta (sp),y
        iny
        lda #CH_METAL_R | $80
        sta (sp),y
        rts
.dh:    lda #CH_HARD_L
        sta (sp),y
        iny
        lda #CH_HARD_R
        sta (sp),y
        rts
.db0:   lda #CH_SKY
        sta (sp),y
        iny
        sta (sp),y
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Schlaeger (Spieler 0-2): Formen schreiben.  16 px = Spieler 0 (beide Enden rund), 32 px = Spieler 0 + 1, 48 px = Spieler 0 + 1 + 2 (je 16 px = 8 Farbtakte).
;   Ein Bit eines Spielerbytes ist ein Farbtakt (2 px); gerundet wird durch ein weggelassenes Bit an der Aussenseite.
; ---------------------------------------------------------------------------------------------------------------------
set_padw:
        sta padw
        lsr
        lsr
        lsr
        sta zstep
        ldx #5
        lda #0
.sp0:   sta PM_P0 + 32 + PADY,x         ; alte Formen loeschen (6 Zeilen)
        sta PM_P1 + 32 + PADY,x
        sta PM_P2 + 32 + PADY,x
        dex
        bpl .sp0
        lda padw
        cmp #16
        beq .p16
        cmp #48
        beq .p48
        ldx #5                          ; 32 px: P0 links (links rund), P1 rechts (rechts rund)
.sp1:   lda pad_l,x
        sta PM_P0 + 32 + PADY,x
        lda pad_r,x
        sta PM_P1 + 32 + PADY,x
        dex
        bpl .sp1
        rts
.p16:   ldx #5                          ; 16 px: beide Enden rund in P0
.sp2:   lda pad_s,x
        sta PM_P0 + 32 + PADY,x
        dex
        bpl .sp2
        rts
.p48:   ldx #5                          ; 48 px: P0 links, P1 Mitte, P2 rechts
.sp3:   lda pad_l,x
        sta PM_P0 + 32 + PADY,x
        lda #$ff
        sta PM_P1 + 32 + PADY,x
        lda pad_r,x
        sta PM_P2 + 32 + PADY,x
        dex
        bpl .sp3
        rts
pad_l: .byte $7f, $ff, $ff, $ff, $ff, $7f            ; linkes Ende (Bit 7 = links)
pad_r: .byte $fe, $ff, $ff, $ff, $ff, $fe            ; rechtes Ende
pad_s: .byte $3c, $7e, $ff, $ff, $7e, $3c            ; schmal, beide Enden rund

; ---------------------------------------------------------------------------------------------------------------------
; update_sprites: Positionen der Figuren aus dem Spielzustand.
;   Schlaeger: HPOS = 48 + padx / 2 (Spieler 1 und 2 je 8 Farbtakte weiter).  Baelle: Missile i (Bits 2i..2i+1 des Missile-Bytes), 4 Zeilen hoch, HPOS = 49 + px / 2.
;   Kapsel: Spieler 3.  Laserschuss: Missile 3.
; ---------------------------------------------------------------------------------------------------------------------
update_sprites:
        lda padh
        lsr
        lda padl
        ror
        clc
        adc #48
        sta HPOSP0
        adc #8
        sta HPOSP1
        adc #8
        sta HPOSP2
        lda #0
        sta cb
.us1:   ldx cb
        lda bact,x
        beq .usoff
        jsr load_ball
        jsr get_px                      ; pxl/pxh
        lda pxh
        lsr
        lda pxl
        ror
        clc
        adc #49
        ldx cb
        sta HPOSM0,x
        lda byl                         ; py = Y / 64 (obere Kante); gezeichnet werden die Zeilen py + 1 .. py + 4
        sta t0
        lda byh
        sta t1
        asl t0
        rol t1
        asl t0
        rol t1
        lda t1
        ldx cb
        jsr ball_draw
        jmp .usn
.usoff: ldx cb
        lda #0
        sta HPOSM0,x
        jsr ball_hide
.usn:   inc cb
        lda cb
        cmp #3
        bne .us1
        jsr coin_draw
        jmp shot_draw

; ball_draw: Ball X (0..2) mit oberer Kante A zeichnen (alte Lage loeschen).  ball_hide: nur loeschen.
ball_draw:
        sta t3
        jsr ball_hide
        lda t3
        sta old_py,x
        tay
        lda ball_mask,x
        sta t0
        lda #4
        sta t2
.bd1:   lda PM_M + 33,y
        ora t0
        sta PM_M + 33,y
        iny
        dec t2
        bne .bd1
        rts
ball_hide:                              ; Eingabe X = Ball; loescht die Maskenbits des Balls in den 4 Zeilen ab old_py.  Veraendert A, Y, t0, t2.
        lda old_py,x
        beq .bh9
        tay
        lda ball_mask,x
        eor #$ff
        sta t0
        lda #4
        sta t2
.bh1:   lda PM_M + 33,y
        and t0
        sta PM_M + 33,y
        iny
        dec t2
        bne .bh1
        lda #0
        sta old_py,x
.bh9:   rts
ball_mask: .byte $03, $0c, $30, $c0     ; Bits des Balls im Missile-Byte: Missile i = Bits 2i und 2i+1

; coin_draw: Kapsel (Spieler 3): Hoehe je Drehphase 12, 10, 7, 3, 7, 10 Zeilen (Drehung um die waagerechte Achse), 12 Punkte breit; Farbe je Art (spawn_cap).
coin_draw:
        lda old_cy                      ; alte Zeilen loeschen (12)
        beq .cd1
        tay
        lda #0
        ldx #12
.cd0:   sta PM_P3 + 32,y
        iny
        dex
        bne .cd0
        sta old_cy
.cd1:   lda capact
        beq .cdoff
        lda capfr
        lsr
        lsr
        tax
        lda coin_h,x
        sta t2                          ; Hoehe
        sec
        sbc #1
        sta t1                          ; letzte Zeile
        lda #12
        sec
        sbc t2
        lsr                             ; obere Leerzeilen = (12 - Hoehe) / 2
        clc
        adc capy
        sta t3
        tay
        ldx #0
.cd2:   lda #$7e
        cpx #0
        beq .cdE
        cpx t1
        bne .cdS
.cdE:   lda #$3c                        ; erste und letzte Zeile schmaler (rund)
.cdS:   sta PM_P3 + 32,y
        iny
        inx
        cpx t2
        bne .cd2
        lda t3
        sta old_cy
        lda capxh
        lsr
        lda capxl
        ror
        clc
        adc #48 + 1
        sta HPOSP0 + 3
        rts
.cdoff: lda #0
        sta HPOSP0 + 3
        rts
coin_h:    .byte 12, 10, 7, 3, 7, 10    ; Hoehe der Kapsel in Zeilen je Drehphase (capfr / 4)

; shot_draw: Laserschuss (Missile 3, 4 Punkte breit): 8 Zeilen hoch an (shu * 2, shy).
shot_draw:
        lda old_sy
        beq .sd1
        tay
        ldx #8
.sd0:   lda PM_M + 32,y
        and #$3f
        sta PM_M + 32,y
        iny
        dex
        bne .sd0
        lda #0
        sta old_sy
.sd1:   lda shact
        beq .sdoff
        lda shu
        clc
        adc #48 + 1
        sta HPOSM0 + 3
        ldy shy
        sty old_sy
        ldx #8
.sd2:   lda PM_M + 32,y
        ora #$c0
        sta PM_M + 32,y
        iny
        dex
        bne .sd2
        rts
.sdoff: lda #0
        sta HPOSM0 + 3
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Titelbild
; ---------------------------------------------------------------------------------------------------------------------
; title: baut das Titelbild auf (Logo, Ticker, Bestenliste, Hinweise, Musik) und wartet auf Feuer oder 30 s Leerlauf (dann Demo).
;   Rueckgabe: demo = 0 (Spielstart) oder 1 (Demo); die Musik ist beim Spielstart beendet (mus_stop).  Testfassungen kehren sofort zurueck.
title:  lda #1
        jsr use_mode                    ; Titel-Display-Liste und Modustabelle
        lda #3
        sta theme_n
        jsr clear_screen
        lda #0
        sta lph
        sta lcnt
        sta demo
        sta sfx_mute
        jsr color_rows
        jsr clear_pm
        jsr draw_logo
        jsr logo_copy
        jsr title_scroll_init
        jsr title_balls_init
        lda #9                          ; Untertitel: Ticker in Zeile 9 (zwei wechselnde Texte)
        ldx #0
        jsr ticker_init
        lda #<t_best
        sta tp
        lda #>t_best
        sta tp + 1
        ldx #14
        ldy #11
        jsr puts
        jsr draw_top5
        lda #<t_ctrl
        sta tp
        lda #>t_ctrl
        sta tp + 1
        ldx #4
        ldy #19
        jsr puts
        lda #<t_keys
        sta tp
        lda #>t_keys
        sta tp + 1
        ldx #5
        ldy #20
        jsr puts
        lda #<t_start
        sta tp
        lda #>t_start
        sta tp + 1
        ldx #13
        ldy #22
        jsr puts
        ldx #<song
        ldy #>song
        jsr mus_init
.ifdef TESTHS
        jsr hs_save                     ; Testfassung: Bestenliste sofort schreiben und wieder laden, Fehlercodes zeigen (mit HS_HOSTTEST + atari800 -H1)
        lda hs_st
        ldx #2
        ldy #1
        jsr debug_dec
        jsr hs_load
        lda hs_st
        ldx #6
        ldy #1
        jsr debug_dec
        jmp *
.endif
.ifdef TESTLOSE
        rts                             ; Testfassung: ohne Warten starten (spielt dann nur Feuer = verliert)
.endif
.ifdef AUTOPLAY
        lda #DEMOFLAG                   ; Testfassung: der Rechner spielt (DEMOFLAG 1 = wie Demo, 0 = mit Kopfzeile)
        sta demo
        sta sfx_mute
        rts
.else
        jsr input_wait_release
        lda #40
        sta idl                         ; Schonfrist 0,8 s (Zaehler im Speicher: mus_tick veraendert X)
.gr:    jsr wait_frame
        jsr mus_tick
        jsr title_fx
        jsr ticker_tick
        jsr title_scroll_tick
        jsr title_wave
        jsr title_balls_tick
        dec idl
        bne .gr
        jsr input_wait_release
        lda #<1500
        sta idl                         ; 30 s Leerlauf -> Demo
        lda #>1500
        sta idh
.tw:    jsr wait_frame
        jsr mus_tick
        jsr title_fx
        jsr ticker_tick
        jsr title_scroll_tick
        jsr title_wave
        jsr title_balls_tick
        jsr input_poll
        sta mask
        and #IN_FIRE
        bne .start
        lda mask
        beq .idle
        lda #<1500
        sta idl
        lda #>1500
        sta idh
        bne .tw
.idle:  lda idl
        ora idh
        beq .demo
        lda idl
        bne +
        dec idh
+       dec idl
        jmp .tw
.demo:  lda #1
        sta demo
        sta sfx_mute
        jmp .go
.start: jsr mus_stop
.go:    lda #0
        sta VIC_BORDERCOLOR             ; (C64-Platzhalter ohne Wirkung)
        rts
.endif

; title_scroll_init / title_scroll_tick: Lauftext in Zeile 23 mit Feinscrollen (HSCROL, 1 Farbtakt = 2 Punkte je Bild).  Der Versatz zaehlt 3..0; dann rutscht die
;   Zeile um ein Zeichen (4 Farbtakte) nach links und rechts kommt das naechste Zeichen des Textes nach (der Text ist laenger als 256 Zeichen: 16-Bit-Zeiger stq).
title_scroll_init:
        lda #3
        sta hsc
        ldx #47
.si1:   lda scroll_text,x
        sta SCRL,x
        dex
        bpl .si1
        lda #<(scroll_text + 48)
        sta stq
        lda #>(scroll_text + 48)
        sta stq + 1
        rts

title_scroll_tick:                      ; einmal je Bild: Feinversatz hsc herunterzaehlen, bei Unterlauf die Zeile um ein Zeichen schieben
        dec hsc
        bpl .st1
        lda #3
        sta hsc
        ldx #0
.st2:   lda SCRL + 1,x
        sta SCRL,x
        inx
        cpx #47
        bne .st2
        ldy #0
        lda (stq),y
        cmp #$ff
        bne .st3
        lda #<scroll_text               ; Textende: von vorn
        sta stq
        lda #>scroll_text
        sta stq + 1
        lda scroll_text
.st3:   sta SCRL + 47
        inc stq
        bne .st1
        inc stq + 1
.st1:   lda tkph                        ; Farbe der Lauftextzeile: Farbton aus der Tickerfolge, langsam wechselnd
        lsr
        lsr
        lsr
        clc
        adc #3
        and #7
        tax
        lda tk_pal,x
        sta RC_2 + 23
        lda hsc                         ; Feinversatz erst NACH dem eventuellen Zeichenschritt uebergeben: Zeilenpuffer und HSCROL muessen im selben
        sta RC_H + 23                   ; Bild zusammenpassen (vorher sprang der Text beim Zeichenschritt kurz um 3 Takte nach links)
        rts

scroll_text:                            ; Lauftext in Bildschirmcodes, $FF = Ende; vorn 40 Leerzeichen, damit der Text von rechts hereinlaeuft
        .screen "                                        PRALLBALL - ZERSTOERE ALLE STEINE MIT BALL UND SCHLAEGER!   FANGE DIE MUENZEN, DIE AUS ZERSTOERTEN STEINEN FALLEN:  HELLBLAU = BREITER SCHLAEGER,  ROSA = SCHMALER SCHLAEGER,  ROT = UMGEKEHRTE STEUERUNG,  GRUEN = BALL LANGSAMER,  GELB = BALL SCHNELLER,  ORANGE = BALL KLEBT,  WEISS = EXTRABALL,  TUERKIS = BONUSPUNKTE,  VIOLETT = MEHRBALL,  HELLGRUEN = LASER,  GRAU = DURCHSCHLAG.   VIEL SPASS!                "
        .byte $ff

; title_balls_init / title_balls_tick: drei Missile-Baelle (4 Punkte breit) prallen ueber dem Titel von den Raendern ab (Zierde).  X in Farbtakten ab 49, Y in Zeilen.
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
        rts

title_balls_tick:                       ; einmal je Bild: Baelle 0..2 bewegen, an den Raendern umkehren, zeichnen (ball_draw)
        ldx #0
.tb:    lda tbu,x
        clc
        adc tbdu,x
        sta tbu,x
        cmp #4
        bcc .tbf1
        cmp #146
        bcc .tbx
.tbf1:  sec
        lda #0
        sbc tbdu,x
        sta tbdu,x
.tbx:   lda tby,x
        clc
        adc tbdy,x
        sta tby,x
        cmp #20
        bcc .tbf2
        cmp #168
        bcc .tby
.tbf2:  sec
        lda #0
        sbc tbdy,x
        sta tbdy,x
.tby:   lda tbu,x
        clc
        adc #49
        sta HPOSM0,x
        lda tby,x
        jsr ball_draw                   ; X = Ball 0..2, A = obere Zeile
        inx
        cpx #3
        bne .tb
        rts
tb_u0: .byte 30, 80, 120                ; Startwerte der Titelbaelle: X (Farbtakte ab 49), Y (Zeile), Schritt X, Schritt Y ($FF = -1)
tb_y0: .byte 50, 100, 140
tb_du: .byte 1, $ff, 1
tb_dy: .byte 1, 2, $ff

; logo_copy: die Logozeilen 3..7 aus dem Bildschirmspeicher in die 48-Zeichen-Zeilen LOGOBUF kopieren (je 40 Zeichen, links 5 und rechts 3 Himmelszeichen als Rand).
logo_copy:
        lda #0
        sta t1                          ; Logozeile 0..4
.lc1:   lda t1
        clc
        adc #3
        tay                             ; Y = Bildschirmzeile
        ldx #0
        jsr set_rowptr                  ; sp = Zeile
        lda t1
        asl
        asl
        asl
        asl
        sta t2                          ; 16 * Logozeile
        asl
        clc
        adc t2                          ; 48 * Logozeile = 16 * z + 32 * z
        sta t2
        ldx t2
        lda #CH_SKY
        ldy #5
.lc0:   sta LOGOBUF,x                   ; fuenf Himmelszeichen als linker Rand: ANTIC zeigt bei Feinscrollen die Zeile erst ab etwa dem 4. Zeichen (gemessen), so steht
        inx                             ; das Logo mittig und der Anfang wird nicht abgeschnitten
        dey
        bne .lc0
        stx t2
        ldy #0
.lc2:   lda (sp),y
        ldx t2
        sta LOGOBUF,x
        inc t2
        iny
        cpy #40
        bne .lc2
        lda #CH_SKY
        ldx t2
        ldy #3
.lc3:   sta LOGOBUF,x
        inx
        dey
        bne .lc3
        inc t1
        lda t1
        cmp #5
        bne .lc1
        rts

; title_wave: das Logo schwingt: je Logozeile ein anderer Feinscroll-Wert (0..3 Farbtakte nach rechts, benachbarte Zeilen um eine Stufe versetzt) aus einer Sinustabelle; wird ueber RC_H vom DLI gesetzt.
title_wave:
        inc wph
        lda wph
        lsr
        sta t2
        ldx #0
.tw1:   txa
        clc
        adc t2
        and #15
        tay
        lda wave_tab,y
        sta RC_H + 3,x
        inx
        cpx #5
        bne .tw1
        rts
wph:    .byte 0                         ; Phase der Logo-Welle (zaehlt je Bild, Index = wph / 2)
wave_tab: .byte 0, 1, 2, 3, 3, 2, 1, 0, 0, 1, 2, 3, 3, 2, 1, 0   ; Feinscroll-Werte (HSCROL 0..3), 16 Schritte = zwei Perioden

; title_fx: Farbwechsel im Logo (alle 6 Bilder eine Stufe weiter): die Zeilenfarben der Logozeilen rotieren.
title_fx:
        inc lcnt
        lda lcnt
        cmp #6
        bcc .tf9
        lda #0
        sta lcnt
        inc lph
        jmp color_rows
.tf9:   rts

; Titeltexte in Bildschirmcodes (.screen: ASCII - 32), mit $FF abgeschlossen (fuer puts).  t_sub ist ungenutzt (der Ticker zeigt den Untertitel).
t_sub:   .screen "PRALLE ACTION FUER DEN ATARI"
         .byte $ff
t_best:  .screen "BESTENLISTE"
         .byte $ff
t_ctrl:  .screen "JOYSTICK PORT 1: LINKS / RECHTS"
         .byte $ff
t_keys:  .screen "TASTEN Z / X, LEERTASTE = FEUER"
         .byte $ff
t_start: .screen "FEUER = START"
         .byte $ff

; draw_logo: "PRALLBALL" in einer 3 x 5 Pixel grossen Blockschrift aus Bloecken (CH_BLOCK, Modus-4-Zeilen 3..7); Farbe je Zeile aus den Tabellen.
draw_logo:
        lda #0
        sta lt
.dl:    ldx lt
        lda logo_seq,x
        sta t0
        asl
        asl
        clc
        adc t0
        sta t0
        lda lt
        asl
        asl
        clc
        adc #2
        sta lcol
        lda #0
        sta lr
.dr:    lda lr
        clc
        adc t0
        tax
        lda logo_font,x
        sta lbits
        ldy #0
.dp:    asl lbits
        lda lbits
        and #8
        beq .dn
        tya
        clc
        adc lcol
        tax
        sty t3
        lda lr
        clc
        adc #3
        tay
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
logo_seq:   .byte 0, 1, 2, 3, 3, 4, 2, 3, 3             ; Buchstabenfolge P R A L L B A L L (Index in logo_font)
; logo_font: je Buchstabe 5 Zeilen zu 3 Bit (Bit 1 = Block): P, R, A, L, B
logo_font:  .byte %111, %101, %111, %100, %100
            .byte %111, %101, %110, %101, %101
            .byte %111, %101, %111, %101, %101
            .byte %100, %100, %100, %100, %111
            .byte %110, %101, %110, %101, %110

; draw_top5: die ersten fuenf Bestenlisten-Eintraege (Platz, Name, Punkte) in den Zeilen 12..16.  Benutzt lt als Rang, lr als Zeile (Logo-Variablen).
;   Eintrag k liegt bei hs_buf + HS_FIRST + 12 * k: 8 Zeichen Name (ASCII), dann Punkte lo/mid/hi (BCD), dann Level.
draw_top5:
        lda #0
        sta lt                          ; lt = Rang 0..4
.t5:    lda #1
        sta txtcol
        lda lt
        clc
        adc #12
        sta lr                          ; lr = Bildschirmzeile
        lda lt
        clc
        adc #CH_DIGIT0 + 1              ; Platzziffer 1..5 (Bildschirmcode der Ziffer = 16 + n)
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
        sec
        sbc #32                         ; ASCII -> Atari-Bildschirmcode (ASCII - 32)
        pha
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
        sta level
        sta speed
        sta hits
        sta padh
        lda #144                        ; Schlaeger mittig: 16 + (288 - 32) / 2
        sta padl
.ifdef STARTLEVEL
        lda #STARTLEVEL                 ; Testfassung: mit diesem Level (0-basiert) beginnen
        sta level
.endif
        lda #0
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
        lda #32
        jmp set_padw                    ; (set_padw steht in der Atari-Sprite-Schicht)

; load_level: Steinkarte des Levels `level` ins RAM kopieren, zerstoerbare Steine zaehlen, Bild aufbauen.
load_level:
        lda #0
        jsr use_mode                    ; Spiel-Display-Liste
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
        inc cores                       ; Typ 16..23 = Boss-Kern (Lebenspunkte = Typ - 15, jeder Treffer zieht 1 ab)
.lz:    iny
        cpy #216                        ; 12 * 18 Steine
        bne .ll
        lda level
        and #3
        sta theme_n
        ldx level
        lda level_kind,x
        sta lkind                       ; Levelart (Bonus/Boss) aus der Tabelle
        lda #45
        sta btime                       ; Bonuslevel: 45 Sekunden
        lda #0
        sta bsub
        lda #8
        sta batk                        ; Boss: erster Angriff nach 4 s
        jsr color_rows                  ; Farbtabellen je Zeile (Himmel, Steinfarben)
        jsr draw_walls
        jsr draw_map
        jsr draw_hud
        jsr reset_extras
        jsr place_ball
        jsr draw_status                 ; Bonus-/Bosszeile (Bildschirmzeile 2)
        lda demo
        beq .nodt
        lda #0                          ; Demo: Ticker in der obersten Zeile (Folge 1: Spielname, Untertitel, Demo-Hinweis)
        ldx #1
        jsr ticker_init
.nodt:
        lda #3
        sta VIC_SPR_ENABLE              ; (C64-Platzhalter ohne Wirkung; die Figuren zeigt update_sprites)
        rts

; draw_map: die ganze Steinkarte zeichnen (12 Zeilen x 18 Spalten).
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
        adc #CH_DIGIT0
        bne .tput
.notens: lda #CH_SPACE
.tput:  ldx #22
        ldy #0
        jsr put_cell
        pla
        clc
        adc #CH_DIGIT0
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
        adc #CH_DIGIT0
        ldx #33
        ldy #0
        jmp put_cell

; Texte der Kopfzeile (Bildschirmcodes, $FF = Ende)
h_score: .screen "PUNKTE"
         .byte $ff
h_level: .screen "LEVEL"
         .byte $ff
h_balls: .screen "BAELLE"
         .byte $ff

; brick_color: UNGENUTZT (Rest der C64-Fassung: dort Farb-RAM-Wert je Steintyp = 8 + Farbe; auf dem Atari kommen die Steinfarben aus color_rows/hue_body).
; Steintypen: 0 leer, 1..8 Farbsteine, 9 Silber, 10 Gold, 11 unzerstoerbar, 12 explosiv, 13 Ueberraschung, 16..23 Boss-Kern.
brick_color: .byte 8, 10, 15, 13, 11, 14, 12, 9, 10, 9, 15, 8, 10, 9, 8, 8, 10, 12, 14, 11, 13, 15, 9, 9
; brick_pts: Punkte je Treffer (BCD, ein Byte, Index = Steintyp): 10 * Typ fuer 1..8, Silber 50, Gold 90, unzerstoerbar 0, explosiv/Ueberraschung 10,
;   Typ 14 und 15 (nicht benutzt) 0, Boss-Kerne 16..23 je 20 je Treffer
brick_pts:   .byte 0, $10, $20, $30, $40, $50, $60, $70, $80, $50, $90, 0, $10, $10, 0, 0, $20, $20, $20, $20, $20, $20, $20, $20

; ---------------------------------------------------------------------------------------------------------------------
; Spielschleife
; ---------------------------------------------------------------------------------------------------------------------
; play: spielt bis zum Ende eines Levels oder Balls.
;   Rueckgabe A: 0 = Level geschafft, 1 = Ball verloren, 2 = Abbruch.
play:   jsr wait_frame                  ; ab hier ~ 300 Rasterzeilen Zeit bis zum naechsten Bild
        lda demo
        beq +
        jsr mus_tick                    ; in der Demo spielt die Musik weiter (Effekte sind dort stumm)
.ifndef NOTICKER
        jsr ticker_tick                 ; Ticker oben in der Kopfzeile (Spielname, Untertitel, Hinweis im Wechsel; nur in der Demo)
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
        jsr laser_fire                  ; Laser aktiv und Feuer gedrueckt: einen Schuss starten (nur ein Missile frei)
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
        lda #$1e                        ; Atari-Farbwert Gelb (die C64-Fassung uebergab die Farbnummer 7)
        ldx #25
        jsr flash                       ; Level geschafft: gelber Randblitz (25 Bilder)
        lda #SFX_EXTRA                  ; Level geschafft
        jsr sfx_play
        ldx #60
        jsr wait_frames
        lda #0
        rts
.gone:  lda #$34                        ; Atari-Farbwert Rot (die C64-Fassung uebergab die Farbnummer 2)
        ldx #12
        jsr flash                       ; Ball verloren: roter Randblitz (12 Bilder)
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
;   damit unterschiedliche Winkel vorkommen.  Differenz = (Schlaegerkante + atgt) - (Ballmitte); >= 6 -> links, <= -7 -> rechts, dazwischen
;   Totzone.  Setzt mask (Feuer immer an).  Veraendert A, X, t0..t3, cb und die Ballwerte (pick_ball laedt den tiefsten Ball).
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

; move_paddle: Schlaeger nach mask um 4 px je Bild bewegen und auf 16 .. 304 - Breite begrenzen (272 bei 32 px; 16 Bit: padh:padl).
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

; place_ball: Ball haftet in der Schlaegermitte auf dem Schlaeger (stuck = 1); die anderen Ballplaetze werden frei.
;   X = (padl + padw / 2 - 3) * 64 (Ballkante = Schlaegermitte - Ballbreite / 2; bei 32 px also padl + 13), Y = (184 - 6) * 64 = $2C80.
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
.ifdef TESTPM
        lda #0
        sta byl
        lda #$18                        ; Testfassung: y = 96 px fest (Oberkante der Textzeile 12), zum Vergleich von Sprite- und Spielfeldzeilen
        sta byh
.else
        lda #$80
        sta byl
        lda #$2c
        sta byh
.endif
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

; hit_brick: Stein vom Typ A getroffen (bidx/bcol/brow gesetzt).
;   Typ 11 (unzerstoerbar): nur Klang.  Gold (10) wird zu Silber (9), Silber zu Typ 1; alle anderen verschwinden.
;   Fuer jeden Treffer gibt es Punkte nach brick_pts.
hit_brick:
        pha
        lda #$0e                        ; Atari-Farbwert Weiss (die C64-Fassung uebergab die Farbnummer 1)
        ldx #2
        jsr flash                       ; kurzer weisser Randblitz (2 Bilder)
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
        lda #$1e                        ; Atari-Farbwert Gelb (C64: Farbnummer 7)
        ldx #8
        jsr flash                       ; gelber Randblitz fuer den zerstoerten Kern
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
; Boss, Statuszeile, Extras (Kapseln), Laser
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

; draw_status: Statuszeile (Bildschirmzeile 2, Modus 2): Bonus = verbleibende Sekunden, Boss = verbliebene Kerne; sonst nichts.
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
        ldy #2
        jsr puts
        lda btime
        jmp .dsnum
.dsb:   lda #<s_boss
        sta tp
        lda #>s_boss
        sta tp + 1
        ldx #13
        ldy #2
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
        adc #CH_DIGIT0
        pha
        ldx #25
        ldy #2
        jsr set_rowptr
        pla
        ldy #0
        sta (sp),y
        pla
        clc
        adc #CH_DIGIT0
        iny
        sta (sp),y
        rts
.ds0:   rts
; Texte der Statuszeile (draw_status): Bonus-Restzeit bzw. Boss-Kerne
s_bonus: .screen "BONUS   "
        .byte $ff
s_boss: .screen "BOSS: KERNE "
        .byte $ff

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
;   Die Kapselart wird zufaellig gewaehlt (cap_pick; capforce erzwingt eine Art); ihre Farbe (Spieler 3) kommt aus cap_color.  Auf dem Atari
;   traegt die Kapsel keinen Buchstaben: die Art erkennt man nur an der Farbe.  Position capxl/capxh/capy in px (linke/obere Kante).
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
        lda captype                     ; Art (A enthaelt hier nur capforce bzw. 0: ohne dieses Laden waere die Farbe immer die der Art 0)
        tax
        lda cap_color,x
        sta PCOLR0 + 3                  ; Kapselfarbe (Spieler 3)
        lda #0
        sta capfr
        lda #1
        sta capact
.sc9:   rts

; Kapselarten: 0 B breit, 1 S schmal, 2 R umgekehrt, 3 L langsam, 4 F schnell, 5 K klebrig, 6 + Leben, 7 P Punkte, 8 M Mehrball, 9 G Laser, 10 D Durchschlag
; cap_frames und cap_letter sind UNGENUTZT (C64-Rest: Sprite-Bloecke und Buchstaben der Kapsel; cap_letter enthaelt C64-, keine Atari-Bildschirmcodes).
cap_frames: .byte $46, $48, $49, $4a, $49, $48           ; Sprite-Bloecke der Drehphasen (vorn, 3/4, halb, Kante, halb, 3/4)
cap_letter: .byte 2, 19, 18, 12, 6, 11, $2b, 16, 13, 7, 4        ; Bildschirmcodes: B=2, S=19, R=18, L=12, F=6, K=11, + = $2B, P=16, M=13
; Farbe der Muenze je Art (Sprite-Farbe): B hellblau, S hellrot, R rot, L gruen, F gelb, K orange, + weiss, P tuerkis, M violett, G hellgruen,
; D (Durchschlag) grau
cap_color:  .byte $9a, $4a, $34, $c6, $1e, $2a, $0e, $aa, $5a, $dc, $0a
; Gewichtung der Kapselarten bei der Auswahl: 16 Eintraege (Zufallszahl and 15); M kommt oefter als Leben (+)
cap_pick:   .byte 0, 0, 1, 2, 3, 4, 5, 6, 7, 7, 8, 8, 8, 9, 9, 10

; laser_fire: einen Schuss in der Schlaegermitte starten (nur wenn kein Schuss fliegt).  Der zweite Schlitz (shact + 1) bleibt
;   immer leer: auf dem Atari ist nur Missile 3 fuer Schuesse frei.
;   Schussposition in Einheiten zu 2 px: u = (px) / 2 (16-Bit-Wert padh:padl nach rechts schieben).  Veraendert A, t0, t1.
laser_fire:
        lda shact
        ora shact + 1
        bne .lf9                        ; es fliegt noch ein Schuss
        lda padw                        ; Schuss in der Schlaegermitte (padl + padw / 2)
        lsr
        clc
        adc padl
        sta t0
        lda padh
        adc #0
        lsr
        ror t0
        lda t0
        sta shu
        lda #PADY - 8
        sta shy
        lda #1
        sta shact
        lda #0
        sta shact + 1                   ; auf dem Atari gibt es nur ein Missile fuer Schuesse: ein Schuss in der Mitte
        lda #SFX_LASER
        jsr sfx_play
.lf9:   rts

; update_shots: Schuesse um 4 px nach oben; trifft einer einen Stein (Karte an Schussspitze), wirkt hit_brick und der Schuss endet.
;   Spalte = (u - 8) / 8 (u = px / 2, Stein 16 px = 8 Einheiten breit, Feld beginnt bei px 16 = u 8), Zeile = (y - 32) / 8.
update_shots:
        ldx #0
.us:    stx shi                         ; Schuss 0..1; X geht in hit_brick verloren, shi (Nullseite) nicht: danach aus shi neu laden
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

; activate: Kapsel wirkt.  Art in captype (0 B, 1 S, 2 R, 3 L, 4 F, 5 K, 6 +, 7 P, 8 M, 9 G, 10 D; siehe die Zeit-/Wirkungskommentare unten).
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
.a10:   lda #10                         ; D: Durchschlag, 5 s (zaehlt in Halbsekunden)
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
uc_end: rts                             ; gemeinsames Ende von update_cap und activate

; ---------------------------------------------------------------------------------------------------------------------
; Spielende
; ---------------------------------------------------------------------------------------------------------------------
game_over:
        jsr over_screen                 ; Zeilen 18..23 auf Textmodus umstellen
        lda #0
        sta VIC_SPR_ENABLE              ; (C64-Platzhalter ohne Wirkung; die Figuren verschwinden durch clear_pm in over_screen)
        sta VIC_BORDERCOLOR             ; (Platzhalter; ein laufender Randblitz endet, weil color_rows in over_screen RC_BK loescht)
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
        ldy #19
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
        ldy #22
        jsr puts
.ifdef HS_HOSTTEST
        lda hs_st
        ldx #31
        ldy #22
        jsr debug_dec                   ; Testfassung: CIO-Fehlercode dezimal neben der Meldung
.endif
.nohs:  ldx #150                        ; drei Sekunden anzeigen
        jmp wait_frames

; Texte des Spielendes und der Namenseingabe (Bildschirmcodes, $FF = Ende)
g_over:  .screen "SPIEL ZU ENDE"
         .byte $ff
g_new:   .screen "NEUER REKORD!"
         .byte $ff
g_saved: .screen "GESPEICHERT"
         .byte $ff
g_nosave: .screen "NICHT GESPEICHERT!"
         .byte $ff
g_name:  .screen "NAME:"
         .byte $ff
g_help:  .screen "TIPPEN + RETURN  ODER  JOYSTICK"
         .byte $ff

; ---------------------------------------------------------------------------------------------------------------------
; Namenseingabe: 8 Zeichen, Vorgabe "SPIELER".
;   Bedienung: Joystick Port 1 hoch/runter wechselt das Zeichen an der Cursorstelle (Leerzeichen, A-Z, 0-9, im Kreis), rechts/links bewegt den
;   Cursor, Feuer bestaetigt.  Tastatur: Buchstaben, Ziffern, Leerzeichen, - und . tippen (das erste getippte Zeichen loescht die Vorgabe), DEL
;   loescht, RETURN bestaetigt (Z/X/Q/A sind hier Buchstaben, keine Richtungen).  Haelt man eine Joystickrichtung, wiederholt sie sich
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
        ldy #21
        jsr puts
        lda #<g_help
        sta tp
        lda #>g_help
        sta tp + 1
        ldx #5
        ldy #23
        jsr puts
        jsr input_wait_release          ; Feuer vom Spiel noch gedrueckt? -> loslassen lassen
        jsr name_draw
.ne:    jsr wait_frame
.ifdef TESTKEY
        jsr testkey                     ; Testfassung: Tastendruecke D, A, Z in den OS-Puffer einspielen (alle 32 Bilder einer)
.endif
        jsr input_char                  ; Tastatur: getipptes Zeichen (A-Z, 0-9, Leerzeichen, -, .), RETURN ($0D), DEL ($14)
        sta nch
        jsr input_joy                   ; Joystick (Port 1): nur der Stick, nicht die Tasten Z/X/Q/A (hier Buchstaben und keine Richtungen)
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
        jsr name_clear                  ; erste Eingabe: Vorgabe "SPIELER" ganz loeschen
.typ:   pla
        ldy npos
        sta hs_name,y                   ; Zeichen an die Cursorstelle
        cpy #7
        bcs .tdraw                      ; achte Stelle erreicht: bleibt dort (weitere Zeichen ersetzen sie)
        inc npos
.tdraw: jsr name_draw
        jmp .ne
.del:   lda #1                          ; (A bleibt unbenutzt) DEL zaehlt auch als eigene Eingabe: beim ersten Mal die Vorgabe ganz loeschen
        ldx ntyped
        bne .dl
        jsr name_clear
        jmp .tdraw
.dl:    ldy npos
        lda hs_name,y
        cmp #$20                        ; ASCII-Leerzeichen
        beq .dprev                      ; Zeichen unter dem Cursor schon leer: stattdessen das davor loeschen
        lda #$20                        ; Zeichen unter dem Cursor loeschen
        sta hs_name,y
        jmp .tdraw
.dprev: tya
        beq .tdraw                      ; schon ganz links
        dec npos
        ldy npos
        lda #$20                        ; ASCII-Leerzeichen
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

.ifdef TESTKEY
; testkey: spielt Atari-Tastencodes D ($3A), A ($3F), Z ($17), X ($16) im Abstand von 32 Bildern in den OS-Tastaturpuffer CH ein (wie die Tastatur-Unterbrechung des OS).
testkey:
        inc tk_n
        lda tk_n
        and #31
        bne .tk9
        lda tk_n
        lsr
        lsr
        lsr
        lsr
        lsr
        tax
        cpx #4
        bcs .tk9
        lda testkey_codes,x
        sta CH_OS
.tk9:   rts
tk_n: .byte $ff                         ; Bildzaehler von testkey (Start $FF: der erste Aufruf zaehlt auf 0 und spielt sofort den Code D)
testkey_codes: .byte $3a, $3f, $17, $16   ; Atari-Tastaturcodes D, A, Z, X
.endif

; name_clear: Namenspuffer mit ASCII-Leerzeichen ($20) fuellen, Cursor an den Anfang, "es wurde getippt" merken.  (Der Name steht als ASCII im Puffer; der Atari-Bildschirmcode
;   0 = Leerzeichen waere hier falsch: er ergaebe beim Anzeigen ein Zeichen jenseits des Zeichensatzes.)
name_clear:
        lda #1
        sta ntyped
        lda #0
        sta npos
        ldx #7
        lda #$20
.nc1:   sta hs_name,x
        dex
        bpl .nc1
        rts

; cur_idx: Index des Zeichens an der Cursorstelle in der Zeichenfolge ctab (nicht gefunden: 0).  Rueckgabe: X = Index, Y = npos, A = Zeichen.
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
; ch_up / ch_down: naechstes / vorheriges Zeichen der Folge ctab (im Kreis) an der Cursorstelle in hs_name eintragen.  Veraendert A, X, Y.
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
ctab:   .text " ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"   ; Zeichenfolge fuer hoch/runter (37 Zeichen, ASCII)

; name_draw: den Namen in Zeile 21 ab Spalte 16 anzeigen; das Zeichen unter dem Cursor in Umkehrschrift (Bit 7 des Bildschirmcodes).
;   ASCII -> Atari-Bildschirmcode: ASCII - 32 (Leerzeichen 0, Ziffern 16..25, Buchstaben 33..58).
name_draw:
        lda #1
        sta txtcol
        ldx #0
.nr:    stx t1
        lda hs_name,x
        sec
        sbc #32                         ; ASCII -> Atari-Bildschirmcode (ASCII - 32)
+       cpx npos
        bne +
        ora #$80                        ; Cursorstelle invers
+       pha
        txa
        clc
        adc #16
        tax
        pla
        ldy #21
        jsr put_cell
        ldx t1
        inx
        cpx #8
        bne .nr
        rts


; ---- Ticker (Titel Zeile 9, Demo Zeile 0)
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
        jsr set_rowptr                  ; sp auf Spalte 4 der Zeile
        ldy #0
.ti2:   tya
        clc
        adc #TK_CHR
        sta (sp),y                      ; Zeichen 64 + Spalte
        iny
        cpy #32
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

; tk_ga / tk_gb: A = Bildschirmcode -> tka / tkb = Adresse des Bitmusters im Zeichensatz (CHARSET = $8800, + Code * 8).  Veraendert A.
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
        lda #CH_SPACE                   ; Bildschirmcode des Leerzeichens (Atari: 0)
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
        lda tkph                        ; Regenbogenfarbe: der Farbton steht in COLPF2 (Hintergrund bleibt schwarz, weil die Helligkeit 0 ist), die
        lsr                             ; Helligkeit der Schrift kommt aus COLPF1; eine Farbe je Zeile (Farbtabelle RC_2)
        lsr
        and #7
        tax
        lda tk_pal,x
        ldy tkrow
        sta RC_2,y
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
        sta tkd                         ; (der Wert 8 wird sofort ueberschrieben; tkd wird in den senkrechten Effekten je Schritt neu berechnet)
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

tk_pal: .byte $10, $20, $30, $40, $50, $80, $a0, $c0       ; Farbtoene: gelb, orange, rot, rosa, violett, blau, tuerkis, gruen
tk_seqs: .byte 0, 1, 0, 0                                 ; Folge Titel (Texte 0, 1; je Folge Platz fuer 4)
         .byte 2, 0, 1, 0                                 ; Folge Demo (Texte 2, 0, 1)
tk_seq:  .byte 0, 2                                       ; erster Text je Folge
tk_seqlen: .byte 2, 3                                     ; Laenge je Folge
; Texte, je 32 Zeichen, mittig aufgefuellt (Bildschirmcodes: .screen wandelt ASCII in Atari-Bildschirmcodes = ASCII - 32)
tk_msgs:
        .screen "  PRALLE ACTION FUER DEN ATARI  "
        .screen " EIN DEMOSPIEL VON VINTAGECODER "
        .screen "       *** PRALLBALL ***        "

.ifdef HS_HOSTTEST
; debug_dec: A = Wert 0..255 dezimal (3 Ziffern) an Spalte X, Zeile Y (nur Testfassung)
debug_dec:
        stx t2
        sty t3
        ldx #0
.dd1:   cmp #100
        bcc .dd2
        sbc #100
        inx
        bne .dd1
.dd2:   pha
        txa
        clc
        adc #CH_DIGIT0
        ldx t2
        ldy t3
        jsr put_cell
        pla
        ldx #0
.dd3:   cmp #10
        bcc .dd4
        sbc #10
        inx
        bne .dd3
.dd4:   pha
        txa
        clc
        adc #CH_DIGIT0
        ldx t2
        inx
        ldy t3
        jsr put_cell
        pla
        clc
        adc #CH_DIGIT0
        ldx t2
        inx
        inx
        ldy t3
        jmp put_cell
.endif

; over_screen: Spielende / Namenseingabe: Zeilen 18..23 als Textzeilen (Modus 2), vorher die Spielfeldzeichen dort loeschen.
over_screen:
        lda #18
        sta t1
.os1:   ldx #0
        ldy t1
        jsr set_rowptr
        ldy #39
        lda #CH_SPACE                   ; Bildschirmcode des Leerzeichens (Atari: 0)
.os2:   sta (sp),y
        dey
        bpl .os2
        inc t1
        lda t1
        cmp #24
        bne .os1
        lda #2
        jsr use_mode
        jsr color_rows
        jmp clear_pm                    ; Schlaeger, Baelle, Kapsel ausblenden

; ---------------------------------------------------------------------------------------------------------------------
; Bibliotheken und Daten
; ---------------------------------------------------------------------------------------------------------------------
.include "lib/input.asm"
.include "lib/rand.asm"
.include "lib/sound.asm"
.include "lib/hiscore.asm"
.include "lib/dirtable.inc"
.include "lib/levels.inc"
.include "lib/music.inc"
.run start
