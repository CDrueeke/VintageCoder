; =====================================================================================================================
; prallball.asm - PRALLBALL fuer den VC-20 (VIC-20), 8-KB-Erweiterung: 20 Levels + 2 Bonuslevel, Zeichen-Sprites, Joystick/Tastatur, Bestenliste
; =====================================================================================================================
;
; ZWECK
;   Ein Ball-und-Schlaeger-Spiel (Breakout-Art): Der Ball prallt von Waenden, Schlaeger und Steinen ab; sind alle zerstoerbaren Steine weg, beginnt
;   das naechste Level. Faellt der Ball unten heraus, geht ein Leben verloren (im Bonuslevel keines); nach dem letzten Leben endet das Spiel.
;  Kein Spielcode wird mit anderen Systemen geteilt; nur die Bibliotheken
;   data/sys/vic20/{input,rand,sound,hiscore}.asm folgen demselben Vertrag wie auf den anderen Systemen. Dies ist eine SCHLANKE Neufassung der
;   Spiellogik fuer rund 11,5 KB Speicher (3,5 KB intern + 8 KB Erweiterung). Der VIC-I hat keine Sprites: Ball, Schlaeger und Muenzen werden aus
;   Zeichen zusammengesetzt.
;
; BEDIENUNG
;   Titelbild: Feuer startet das Spiel. Nach 30 Sekunden ohne Eingabe spielt der Rechner eine Demo (Levels 1-5; Effekte stumm, die Titelmusik laeuft
;   weiter); jede Eingabe beendet sie und fuehrt zum Titel zurueck.
;   Spiel: Joystick (einziger Anschluss) und Tastatur gleichzeitig moeglich: Z = links, X = rechts, Leertaste = Feuer (Ball abschiessen), P = Pause
;   (erneut P: weiter), RUN/STOP = Abbruch zum Titel. Hoch/Runter (Q/A) liefert die Eingabebibliothek zwar auch, das Spiel wertet sie aber nicht
;   aus.
;   Zu Beginn jedes Balls haftet er mittig am Schlaeger und folgt ihm, bis Feuer gedrueckt wird. Der Abprallwinkel am Schlaeger haengt davon ab, wo
;   der Ball trifft (8 Zonen: -60 bis +60 Grad), und davon, ob sich der Schlaeger gerade bewegt (Zone -1 bei links, +1 bei rechts).
;   Spielende: kommen die Punkte unter die ersten zehn, folgt die Namenseingabe (A-Z, 0-9, Leerzeichen, "-" und "."; DEL loescht, RETURN bestaetigt,
;   nach 30 s ohne Eingabe automatisch); die Liste wird als PRALL.HS auf das Startlaufwerk geschrieben.
;
; SPIELREGELN
;   Steinarten (Typ in der Steinkarte MAP): 1..8 = a..h (ein Treffer, 10 Punkte mal Typ), 9 = Silber (zwei Treffer), 10 = Gold (drei Treffer), 11 =
;   unzerstoerbar (zaehlt nicht mit), 12 = explosiv (zerstoert auch die bis zu acht Nachbarn), 13 = Ueberraschung (gibt immer eine Muenze).
;   Muenzen (Extras) fallen aus zerstoerten Steinen (sonst mit Wahrscheinlichkeit 1/8); mit dem Schlaeger auffangen: B breiter, S schmaler, L
;   langsamer, F schneller (je 20 s), + Extraball, P 500 Punkte. Die Geschwindigkeitsstufe (0..7) steigt alle 8 Schlaegertreffer um 1 und gilt ueber
;   alle Levels hinweg (zurueckgesetzt nur im neuen Spiel). Bonuslevel (2 von 22): 45 Sekunden Zeit, ein verlorener Ball kostet kein Leben. 3 Leben,
;   hoechstens 9.
;
; ABLAUF (Zustandsautomat im Hauptprogramm, Schleife "main")
;   title -> new_game -> load_level -> play (-> naechstes Level / Ball verloren / Abbruch) -> game_over (Bestenliste) -> title
;   play kehrt mit A = 0 (Level geschafft), 1 (Ball verloren) oder 2 (Abbruch) zurueck. Nach dem letzten Level beginnt wieder Level 1.
;
; WAS DER CODE AUF DER HARDWARE TUT
;   VIC-I ($9000-$900F): video_init stellt Spalten/Zeilen ($9002/$9003), Bild- und Zeichensatzadresse ($9005) und Farben ($900F) ein; das
;   Rasterregister $9004 liefert den Bildtakt (wait_frame). Die Stimmen und die Lautstaerke ($900A-$900E) schreibt examples/vic20/prallball/lib/sound.asm.
;   VIA 1/2 ($9110-$912F): Joystick und Tastatur werden direkt abgefragt (input.asm); weder der Tastaturpuffer noch der Jiffy-IRQ des KERNALs
;   laufen.
;   KERNAL: nur fuer die Bestenliste (SETNAM, SETLFS, LOAD, SAVE, SETMSG in hiscore.asm), sonst werden keine KERNAL-Routinen gerufen.
;   Interrupts: SEI am Programmstart, danach nie CLI (der NMI der RESTORE-Taste wird von SEI nicht gesperrt).
;
; BILDSCHIRM (VIC-I, 22 x 23 Zeichen zu 8 x 8 Punkten = 176 x 184 Punkte)
;   Bildschirm $1000 (Farb-RAM $9400), eigener Zeichensatz im RAM bei $1C00 (Codes 0..127; die Textzeichen 0..63 kopiert das Programm aus dem
;   Zeichen-ROM). Der VIC-I sieht nur Speicher unterhalb von $2000 (und das Zeichen-ROM): Bildschirm und Zeichensatz muessen darum in $1000-$1FFF
;   liegen, der Programmcode kann auch in der Erweiterung ab $2000 stehen. Mehrfarbzeichen (Farb-RAM-Bit 3): Pixelpaar 01 = Rahmenfarbe (schwarz:
;   Schatten), 10 = Zeichenfarbe, 11 = Hilfsfarbe (weiss: Licht). Steine und Waende sind Mehrfarbzeichen, Text und Zeichen-Sprites sind einfarbig
;   (Hires).
;   Zeichencodes: 0..63 Text (Bildschirmcodes wie im ROM: 1..26 = A..Z, 48..57 = Ziffern, 32 = Leerzeichen), 64 Stein links, 65 Stein rechts, 66
;   Wand, 80..99 Pool-Zeichen fuer die Zeichen-Sprites; die uebrigen Codes sind unbenutzt.
;   Ball, Schlaeger und Muenze sind ZEICHEN-SPRITES: Zellen unter den Figuren werden mit Pool-Zeichen belegt (Codes 80..), deren Bitmuster aus dem
;   Bild zusammengesetzt wird (Hintergrund der Zelle ist dort immer leer oder wird verdeckt). Der Rueckbau (tsp_restore) berechnet den Hintergrund
;   aus der Steinkarte.
;   Zeitlich: Es gibt keinen Rasterinterrupt; das Programm laeuft mit gesperrten Interrupts, der Bildtakt kommt aus dem Rasterregister ($9004). Ein
;   Durchlauf der Spielschleife (play) muss in ein Bild passen (PAL, 50 Hz), sonst geht ein Bild verloren und das Spiel wird langsamer.
;
; SPEICHERAUSBAU: nur 8k oder mehr (Projekt: memory 8k; vcc asm ... --memory 8k). Der Assembler setzt dann MEM_SCREEN_RAM = $1000, MEM_COLOR_RAM =
;   $9400 und MEM_BASIC_START = $1201, woraus <vic20/hardware.asm> SCREEN_RAM, COLOR_RAM und BASIC_START macht. Mit dem Grundgeraet oder +3 KB
;   (Bildschirm $1E00, BASIC-Start $1001 bzw. $0401 laut hardware.asm) laeuft das Programm nicht: Teil 2 liegt ab $2000 und braucht dort
;   Erweiterungs-RAM.
;
; SPEICHERPLAN (8-KB-Ausbau: BASIC-Start $1201)
;   $0340-$03B7 Steinkarte MAP (120 Byte, Kassettenpuffer)   $03B8-$03CB Pool-Zeilen   $03CC-$03DF Pool-Spalten
;   $1000-$11F9 Bildschirm (22 x 23)   $1201 BASIC-Zeile "10 SYS", Programmstart bei $120D
;   $1201-$1BFF Programm Teil 1 (derzeit bis $1BDC; tools/run_tests.py prueft, dass es vor $1C00 endet)
;   $1C00-$1FFF Zeichensatz-RAM (128 Zeichen; im Programmabbild mit Nullen gefuellt, video_init belegt ihn)
;   $2000-$3FFF Programm Teil 2 und Daten (derzeit bis $34BC): blit, Zeichen-Sprites, Titel, Namenseingabe, Bibliotheken (input, rand, sound,
;       hiscore mit dem Puffer hs_buf), Tabellen (dir_tab, gfx_chars, logo, level_data, song). Reserve rund 2,9 KB; der Testlaeufer nimmt $3F00 als
;       Falle.
;   $8000-$81FF Zeichen-ROM (Quelle der Textzeichen)   $9400-$95F9 Farb-RAM (22 x 23 Zellen, nur Bit 0..3)
;
; NULLSEITE: $02 Zufall (2 Byte), $04 Musik (4), $08 Bestenliste (6), $10-$62 Spielzustand (siehe unten), $8C Eingabe (4). Das Programm kehrt nie zu
;   BASIC zurueck, darf also die BASIC-Zeiger belegen; die KERNAL-Variablen ab $90 (Status, Geraetenummer ...) bleiben unberuehrt, die Dateizugriffe
;   brauchen sie.
;
; KOORDINATEN: Bildpunkte ab links oben des Bildschirms. Spielfeld x = 8..167 (Spalten 1..20), y = 16.. (Zeile 2 abwaerts); Waende in Spalte 0 und
;   21 sowie Zeile 1, Kopfzeile 0. Steinfeld: 10 Steine je 16 Punkte Breite ab x = 8, 12 Zeilen je 8 Punkte ab y = 16 (Zeile 2..13). Der Schlaeger
;   liegt in Zeile 21 (y = 170, 5 Punkte hoch). Ball: 5 x 5 Punkte, Position in Festkomma 8.8 (Hochbyte = Bildpunkt); Geschwindigkeit 16 Bit
;   vorzeichenbehaftet, Einheit 1/256 Punkt je Bild. Zeiten zaehlen Bilder (50 je Sekunde); die Extra-Timer laufen in Halbsekunden (25 Bilder).
;
; BIBLIOTHEKEN UND ERZEUGTE DATEN (am Dateiende eingebunden)
;   <vic20/input.asm>    input_init, input_poll (Maske IN_*), input_char, input_wait_release
;   <vic20/rand.asm>     rand_seed, rand_byte
;   <vic20/sound.asm>    sfx_init/sfx_play/sfx_update (Effekte SFX_*), mus_init/mus_tick/mus_stop (Titellied); sfx_mute <> 0 stellt die Effekte stumm
;   <vic20/hiscore.asm>  hs_init, hs_load, hs_save, hs_check, hs_insert; Puffer hs_buf, hs_points, hs_level, hs_name
;   lib/*.inc            ERZEUGT von tools/gen.py (nicht von Hand aendern): dir_tab, gfx_chars, logo_bits/logo_y, level_data/level_kind/level_count, song
;
; TESTFASSUNGEN (vcc asm prallball.asm --target vic20 --memory 8k -D NAME[=wert] -o x.prg)
;   AUTOPLAY       der Rechner spielt von Anfang an, kein Titelbild, Effekte stumm (nach Level 5 beginnt es wieder bei Level 1)
;   STARTLEVEL=n   beginnt mit Level n (0-basiert, 0..21)
;   TESTLOSE       kein Titelbild; Eingabe fest "Feuer + rechts", der Ball geht verloren; 5000 Punkte zu Beginn -> Spielende und Namenseingabe
;   TESTCAP        aus jedem zerstoerten Stein faellt eine Muenze (sonst nur mit Wahrscheinlichkeit 1/8)
;
; BEKANNTE GRENZEN: nur PAL ist getestet (NTSC-Zeitverhalten unbekannt); der Klang ist nur ueber die Registerwerte geprueft; bewusst fehlen
;   Boss-Level, Mehrball, Laser, Durchschlag, Paddle und Joystick-Namenseingabe; es gibt nur NPOOL = 20 Pool-Zeichen je Bild (benoetigt werden
;   hoechstens 13).
; =====================================================================================================================
.target vic20
.cpu 6502
.branchlong auto

; ---------------------------------------------------------------------------------------------------------------------
; Nullseite: Spielzustand $10-$62 (Bibliotheken: Zufall $02, Musik $04, Bestenliste $08, Eingabe $8C).  Alles sind Bytes; 16-Bit-Werte liegen als Low/High
; nebeneinander (Low zuerst).  Die Hilfsspeicher t0..t5 gelten nur innerhalb einer Routine; jede Routine nennt, welche sie verwendet.
; ---------------------------------------------------------------------------------------------------------------------
; -- Ball: Position ist Festkomma 8.8 (Bildpunkt = High-Byte, Bruchteil = Low-Byte), Geschwindigkeit in 1/256 Punkt je Bild (vorzeichenbehaftet)
.const bxl     = $10                    ; Ball-X, Festkomma 8.8: Low = Bruchteil, High = Bildpunkt (linke Kante des 5 x 5 Punkte grossen Balls)
.const bxh     = $11
.const byl     = $12                    ; Ball-Y (Oberkante)
.const byh     = $13
.const vxl     = $14                    ; Ball-Geschwindigkeit X (vorzeichenbehaftet, 1/256 Punkt je Bild)
.const vxh     = $15
.const vyl     = $16                    ; Ball-Geschwindigkeit Y (negativ = nach oben)
.const vyh     = $17
; -- Schlaeger und Spielstand
.const padx    = $18                    ; Schlaeger: linke Kante in Bildpunkten (8 .. 168 - padw)
.const padw    = $19                    ; Schlaegerbreite (16, 24 oder 32)
.const speed   = $1a                    ; Geschwindigkeitsstufe 0..7
.const hits    = $1b                    ; Schlaegertreffer seit dem Start; alle 8 steigt die Stufe
.const lives   = $1c                    ; Leben (Baelle) 0..9, in der Kopfzeile angezeigt
.const level   = $1d                    ; aktuelles Level 0..level_count - 1 (0-basiert)
.const bricks  = $1e                    ; noch zu zerstoerende Steine (unzerstoerbare zaehlen nicht)
.const stuck   = $1f                    ; 1 = Ball haftet am Schlaeger
.const sc0     = $20                    ; Punktestand BCD, 6 Stellen: sc0 = Einer/Zehner, sc1 = Hunderter/Tausender, sc2 = Zehn-/Hunderttausender
.const sc1     = $21
.const sc2     = $22
.const mask    = $23                    ; Eingabemaske dieses Bildes (IN_*-Bits aus input_poll)
; -- Hilfsspeicher, Zeiger, Zeichenausgabe
.const t0      = $24                    ; Hilfsspeicher (kurzlebig)
.const t1      = $25
.const t2      = $26
.const t3      = $27
.const t4      = $28
.const t5      = $29
.const pmov    = $2a                    ; Schlaegerbewegung dieses Bildes: 0 ruhig, 1 links, 2 rechts
.const sp      = $2b                    ; Zeiger auf Bildschirm (2 Byte)
.const cp      = $2d                    ; Zeiger auf Farb-RAM (2 Byte)
.const tp      = $2f                    ; Zeiger auf Text (2 Byte)
.const txtcol  = $31                    ; Farbe fuer put_cell/puts: Farb-RAM-Wert (0..7 einfarbig, 8..15 = 8 + Farbe: Mehrfarbzeichen)
; -- Muenze (Extra), Zeitgeber, Bonuslevel
.const capact  = $32                    ; 1 = eine Muenze faellt
.const capx    = $33                    ; Muenze: linke Kante, Oberkante (Bildpunkte)
.const capy    = $34
.const captype = $35                    ; Art 0..5: B S L F + P
.const tpad    = $36                    ; Rest-Halbsekunden: Schlaegerbreite geaendert
.const tspd    = $37                    ; Rest-Halbsekunden: Geschwindigkeit geaendert
.const spdmod  = $38                    ; 0 normal, 1 schneller (+3 Stufen), 2 langsamer (-3 Stufen)
.const tick    = $39                    ; Bilder bis zur naechsten halben Sekunde
.const lkind   = $3a                    ; Levelart: 0 normal, 1 Bonus
.const btime   = $3b                    ; Bonuslevel: Restsekunden
.const bsub    = $3c                    ; Halbsekundenzaehler dazu (2 je Sekunde)
; -- Zeichen-Sprites: Anzahl belegter Pool-Zeichen, Glyphzeiger, Parameter und Zwischenwerte von blit/get_cell
.const pool_n  = $3d                    ; Anzahl vergebener Pool-Zeichen in diesem Bild
.const gp      = $3e                    ; Zeiger auf Glyphdaten (2 Byte)
.const bl_img  = $40                    ; blit: Zeiger auf Bildzeilen (2 Byte; je Zeile ein Byte, hoechstes Bit = linkester Punkt)
.const bl_n    = $42                    ;       Anzahl Zeilen
.const bl_x    = $43                    ;       X in Bildpunkten
.const bl_y    = $44                    ;       Y in Bildpunkten
.const bl_col  = $45                    ;       Farbe (0..7)
.const bl_i    = $46                    ;       laufende Bildzeile
.const bl_l    = $47                    ;       Bildbyte, um X mod 8 nach rechts geschoben: Anteil der linken Zelle
.const bl_r    = $48                    ;       herausgeschobener Rest: Anteil der rechten Zelle
.const bl_cr   = $49                    ;       Zellzeile
.const bl_cx   = $4a                    ;       Zellspalte
.const bl_ln   = $4b                    ;       Pixelzeile innerhalb der Zelle (0..7)
.const gc_r    = $4c                    ; get_cell: Zeile und Spalte
.const gc_c    = $4d
; -- Steintreffer, Titel, Namenseingabe, Diagnose
.const bidx    = $4e                    ; zuletzt getroffener Stein: Index in der Karte (Zeile * 10 + Spalte)
.const bcx     = $4f                    ;   Bildschirmspalte, -zeile seines linken Zeichens (von draw_brick_idx gesetzt)
.const bcy     = $50
.const frame   = $51                    ; Titel: Bildzaehler (alle 4 Bilder wechselt die Logofarbe)
.const npos    = $52                    ; Namenseingabe: Cursorposition 0..8
.const ntyped  = $53                    ;   1 = die Vorgabe "SPIELER" ist schon durch eigene Eingabe ersetzt
.const ntl     = $54                    ;   Zaehler bis zur Auto-Bestaetigung (16 Bit, zaehlt Bilder herunter)
.const nth     = $55
.const idl     = $56                    ; Titel: Leerlaufzaehler (16 Bit, Bilder bis zur Demo; idl dient anfangs als Schonfrist)
.const idh     = $57
.const lph     = $58                    ; Logo: Farbphase
.const wfc     = $59                    ; wait_frames: Restbilder
.const frame2  = $61                    ; Bildzaehler (Diagnose; wait_frame zaehlt mit, gelesen wird er nirgends)
.const demo    = $62                    ; 1 = Demo-Modus (Rechner spielt, keine Effekte)
.const zone    = $5a                    ; Abprallzone 0..7
.const lasthit = $5b                    ; probe_ball: zuletzt verarbeiteter Stein ($ff = keiner)
.const htype   = $5c                    ; hit_brick: Typ des getroffenen Steins
.const ex_i    = $5d                    ; explode: Nachbar, Zeile, Spalte der Mitte, Hilfswert
.const ex_r    = $5e
.const ex_c    = $5f
.const ex_n    = $60

; ---------------------------------------------------------------------------------------------------------------------
; Konstanten
; ---------------------------------------------------------------------------------------------------------------------
.const MAP      = $0340                 ; Steinkarte im Kassettenpuffer: 12 Zeilen x 10 Spalten (Index = Zeile * 10 + Spalte), je Byte ein Typ 0..13, 0 = leer
.const POOL_CR  = $03b8                 ; Pool-Zellenliste: Bildschirmzeile der Zelle n (NPOOL Byte, $03B8-$03CB)
.const POOL_CX  = $03cc                 ;                   Bildschirmspalte der Zelle n (NPOOL Byte, $03CC-$03DF)
.const NPOOL    = 20                    ; Anzahl Pool-Zeichen (= gleichzeitig belegbare Zellen der Zeichen-Sprites)
.const POOL0    = 80                    ; erstes Pool-Zeichen (Codes 80..99)
.const CHARSET  = $1c00                 ; Zeichensatz im RAM (128 Zeichen zu 8 Byte = 1 KB bis $1FFF)
.const PADY     = 170                   ; Oberkante des Schlaegers in Bildpunkten (Zeile 21); der Ball sitzt darauf bei byh = PADY - 5
.const LIVES0   = 3                     ; Leben zu Spielbeginn
.const CH_SPACE = 32                    ; Bildschirmcode Leerzeichen
.const CH_BRICK_L = 64                  ; Mehrfarbzeichen: Stein links, Stein rechts, Wand (Muster in gfx_chars)
.const CH_BRICK_R = 65
.const CH_WALL  = 66
.const RASTER_WAIT = 130                ; Rasterwert ($9004 = Rasterzeile / 2), ab dem das Bild unten zu Ende ist: dann werden Zeichen-Sprites umgebaut

#include <vic20/hardware.asm>
#include <vic20/kernal.asm>

.basicstub start                        ; erzeugt "10 SYS <start>" ab BASIC-Start ($1201 beim 8-KB-Ausbau; start liegt bei $120D)

; ---------------------------------------------------------------------------------------------------------------------
; Programmstart und Hauptschleife
; ---------------------------------------------------------------------------------------------------------------------
; Der BASIC-Start (SYS) landet hier.  hs_init muss vor jedem Dateizugriff laufen: das KERNAL ueberschreibt die Geraetenummer ($BA) bei Dateizugriffen,
; die Bestenliste soll aber immer vom Startlaufwerk kommen.
start:  sei                             ; Interrupts bleiben AUS: weder Uhr noch Tastaturabfrage des Betriebssystems sind gewollt (die Tastatur lesen wir selbst)
        jsr hs_init                     ; Startgeraet fuer die Bestenliste merken
        jsr video_init                  ; Zeichensatz nach $1C00, VIC-I-Register
        jsr sfx_init                    ; Lautstaerke 15, Hilfsfarbe weiss, Stimmen still
        jsr input_init                  ; Tastaturspalten auf Ausgang, Eingabezustand loeschen
        lda $9004                       ; Rasterzeile: unregelmaessiger Anteil fuer den Zufallsstartwert (rand_seed mischt $9004 selbst nochmals dazu)
        jsr rand_seed
        jsr hs_load                     ; Bestenliste von der Diskette; bei Fehlern bleibt die Standardliste
        ; weiter in die Hauptschleife

main:   jsr title                       ; Titelbild; kehrt zurueck, sobald Feuer gedrueckt wurde (oder die Demo beginnen soll: demo = 1)
        jsr new_game
.level: jsr load_level                  ; Steinkarte des Levels `level` laden und zeichnen
.ball:  jsr play                        ; Rueckgabe in A: 0 = Level geschafft, 1 = Ball verloren, 2 = Abbruch (RUN/STOP)
        cmp #2
        beq main                        ; Abbruch: zurueck zum Titel
        cmp #1
        beq .lost
        lda demo
        beq +
        lda level
        cmp #4
        bcs main                        ; Demo zeigt nur die Levels 1-5 (level ist 0-basiert: 4 = das fuenfte Level ist geschafft)
+       inc level                       ; Level geschafft: naechstes, nach dem letzten wieder das erste
        lda level
        cmp #level_count
        bcc .level
        lda #0
        sta level
        beq .level
.lost:  lda lkind
        bne .bonuslost                  ; Bonuslevel: ein verlorener Ball kostet kein Leben
        dec lives
        beq .over                       ; letztes Leben verloren
        jsr reset_extras                ; Extras (Muenze, Schlaegerbreite, Tempo) gelten nicht fuer den naechsten Ball
.bonuslost:
        jsr place_ball                  ; neuer Ball haftet am Schlaeger
        jsr draw_hud                    ; Kopfzeile mit der neuen Ballzahl
        jmp .ball
.over:  lda demo
        bne main                        ; Demo: kein Spielende-Bild, keine Bestenliste
        jsr game_over
        jmp main

; ---------------------------------------------------------------------------------------------------------------------
; video_init: Bildschirm, Zeichensatz, Farben (einmal beim Start).  Veraendert A, X.
;   Zeichensatz: Textzeichen 0..63 (512 Byte) aus dem Zeichen-ROM ($8000) nach $1C00, eigene Zeichen 64..66 aus gfx_chars, Pool-Zeichen 80..99 leer.
;   Die uebrigen Zeichen (67..79, 100..127) bleiben unberuehrt: das Programmabbild fuellt $1C00-$1FFF beim Laden mit Nullen.
;   $9005 = $CF: Bits 4..7 = Bildschirmadresse (Bit 7 = VIC-Adressbit A13 = 1: RAM unter $2000; Bits 6..4 = A12..A10 = 100 -> $1000),
;                Bits 0..3 = Zeichensatz (15 -> $1C00; laut vic1.chip.json 0 = Zeichen-ROM ab $8000, 8 = $0000, 12 = $1000 RAM: Schritte zu 1 KB).
;   $9002 = $16: Bits 0..6 = 22 Spalten, Bit 7 = 0 (Bildschirmadresse A9 = 0, dazu gehoert das Farb-RAM bei $9400).
;   $9003 = $2E: Bits 1..6 = 23 Zeilen, Bit 0 = 0 (8 x 8 statt 8 x 16 Punkte je Zeichen).  Lage des Bildes ($9000/$9001) bleibt wie vom KERNAL gesetzt.
;   $900F = $08: Hintergrund (Bits 4..7) schwarz, Bit 3 = 1 (normal, nicht invers), Rahmen (Bits 0..2) schwarz = zugleich der Schatten der Mehrfarbzeichen.
;   Das Hochnibble von $900E (Hilfsfarbe, hier weiss = Licht der Mehrfarbzeichen) setzt sfx_init.
; ---------------------------------------------------------------------------------------------------------------------
video_init:
        ldx #0                          ; Zeichen 0..63 aus dem Zeichen-ROM ($8000) nach $1C00, 512 Byte
.vi1:   lda $8000,x
        sta CHARSET,x
        lda $8100,x
        sta CHARSET + $100,x
        inx
        bne .vi1
        ldx #23                         ; eigene Zeichen 64..66 (Stein links, rechts, Wand): 3 * 8 Byte, rueckwaerts kopiert
.vi2:   lda gfx_chars,x
        sta CHARSET + 64 * 8,x
        dex
        bpl .vi2
        ldx #0                          ; Pool-Zeichen 80..99 leeren (160 Byte)
        lda #0
.vi3:   sta CHARSET + POOL0 * 8,x
        inx
        cpx #NPOOL * 8
        bne .vi3
        lda #$16                        ; 22 Spalten, Bildschirmadresse Bit 9 = 0
        sta $9002
        lda #$2e                        ; 23 Zeilen, 8 x 8 Punkte
        sta $9003
        lda #$cf                        ; Bildschirm $1000, Zeichensatz $1C00
        sta $9005
        lda #$08                        ; schwarzer Hintergrund und Rahmen, normal
        sta $900f
        rts

; clear_screen: Bildschirm = Leerzeichen, Farb-RAM = weiss (1), pool_n = 0.  Veraendert A, X.
;   Geloescht werden je 512 Byte ($1000-$11FF und $9400-$95FF), also etwas mehr als die 506 Zellen des Bildes.
clear_screen:
        ldx #0
.cs:    lda #CH_SPACE
        sta SCREEN_RAM,x
        sta SCREEN_RAM + $100,x
        lda #1
        sta COLOR_RAM,x
        sta COLOR_RAM + $100,x
        inx
        bne .cs
        lda #0
        sta pool_n
        rts

; set_rowptr: sp (Bildschirm) und cp (Farb-RAM) auf Spalte X, Zeile Y setzen.  X und Y bleiben erhalten.  Veraendert A, sp, cp.
;   Trick: Bildschirm- und Farb-RAM-Adresse haben dasselbe Low-Byte (Unterschied $8400), cp wird deshalb aus sp nur im High-Byte korrigiert.
set_rowptr:
        txa
        clc
        adc rowlo,y
        sta sp
        sta cp
        lda rowhi,y
        adc #0
        sta sp + 1
        clc
        adc #(>COLOR_RAM) - (>SCREEN_RAM)
        sta cp + 1
        rts

; put_cell: Zeichen A (Bildschirmcode) mit Farbe txtcol an Spalte X, Zeile Y.   X bleibt erhalten.  Veraendert A, Y (= 0), sp, cp.
put_cell:
        pha
        jsr set_rowptr
        ldy #0
        pla
        sta (sp),y
        lda txtcol
        sta (cp),y
        rts

; puts: Text ab (tp), mit 0 abgeschlossen (Bildschirmcodes, hoechstens 255 Zeichen), an Spalte X, Zeile Y in Farbe txtcol.   Veraendert A, X, Y, t1..t3, sp, cp.
;   t1 = Index im Text, t2/t3 = laufende Spalte/Zeile; put_cell setzt Y auf 0, darum werden X und Y je Zeichen neu geladen.
puts:   stx t2
        sty t3
        ldy #0
        sty t1
.pl:    ldy t1
        lda (tp),y
        beq .pd
        ldx t2
        ldy t3
        jsr put_cell
        inc t2
        inc t1
        bne .pl
.pd:    rts

; put_bcd: BCD-Byte A als zwei Ziffern (Zehner an Spalte X, Einer an X + 1) in Zeile Y, Farbe txtcol.   Veraendert A, X, Y, t0, t2, t3, sp, cp.
put_bcd:
        sta t0
        stx t2
        sty t3
        lsr
        lsr
        lsr
        lsr
        clc
        adc #$30
        jsr put_cell
        lda t0
        and #$0f
        clc
        adc #$30
        ldx t2
        inx
        ldy t3
        jmp put_cell

; Zeilenanfaenge des Bildschirms: SCREEN_RAM + 22 * Zeile (0..22), Low- und High-Byte getrennt (Index = Zeile)
rowlo:
    .repeat 23, i
        .byte <(SCREEN_RAM + i * 22)
    .endrepeat
rowhi:
    .repeat 23, i
        .byte >(SCREEN_RAM + i * 22)
    .endrepeat

; wait_frame: wartet auf das Bildende (Rasterwert RASTER_WAIT): das Programm laeuft mit 50 Hz; im Bildwechsel werden die Zeichen-Sprites umgebaut,
;   solange der Strahl unterhalb der Anzeige steht.  Veraendert A, frame2.
;   Zwei Schleifen: Steht der Strahl beim Aufruf schon unter der Marke, wartet die erste ihn bis in das naechste Bild ab, sonst kaeme der Aufruf sofort
;   zurueck und zwei Aufrufe hintereinander fielen in dasselbe Bild.  Braucht ein Durchlauf laenger als ein Bild, geht die Marke dieses Bildes verloren.
wait_frame:
        inc frame2
.w1:    lda $9004
        cmp #RASTER_WAIT
        bcs .w1                         ; noch unterhalb der Marke von diesem Bild: den Anfang des naechsten abwarten
.w2:    lda $9004
        cmp #RASTER_WAIT
        bcc .w2                         ; Marke erreicht
        rts

; wait_frames: X Bilder warten; dabei laeuft der Klangeffekt weiter (sfx_update je Bild).   Veraendert A, X, Y, wfc, frame2.
wait_frames:
        stx wfc
.wf:    jsr wait_frame
        jsr sfx_update
        dec wfc
        bne .wf
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Spielstart und Level aufbauen
; ---------------------------------------------------------------------------------------------------------------------
; new_game: neues Spiel: Punkte, Level, Geschwindigkeit und Trefferzahl auf 0, 3 Leben, Schlaeger mittig; danach reset_extras.  Veraendert A.
;   Testfassungen: TESTLOSE beginnt mit 5000 Punkten, STARTLEVEL=n mit Level n.
new_game:
        lda #0
        sta sc0
        sta sc1
        sta sc2
        sta level
        sta speed
        sta hits
.ifdef TESTLOSE
        lda #$50                        ; Testfassung: 5000 Punkte, damit der Eintrag in die Bestenliste kommt
        sta sc1
.endif
.ifdef STARTLEVEL
        lda #STARTLEVEL                 ; Testfassung: mit diesem Level (0-basiert) beginnen
        sta level
.endif
        lda #LIVES0
        sta lives
        lda #76                         ; Schlaeger mittig: (176 - 24) / 2 (Schlaegerbreite 24 setzt reset_extras)
        sta padx
        jmp reset_extras

; reset_extras: alle Extras aus (neues Spiel, neues Level, Ball verloren): Muenze weg, Timer 0, Schlaeger 24 Punkte breit.
reset_extras:
        lda #0
        sta capact
        sta tpad
        sta tspd
        sta spdmod
        lda #25
        sta tick
        lda #24
        sta padw
        rts

; load_level: Steinkarte des Levels `level` aus level_data entpacken (zwei Steine je Byte, linker im hohen Nibble, 60 Byte je Level) nach MAP, zaehlen
;   (bricks), Levelart (lkind) und Bonuszeit (45 s) setzen, Bild aufbauen (draw_all, draw_status), Ball auf den Schlaeger setzen.
;   Veraendert A, X, Y, tp und alles, was draw_all und place_ball veraendern.
load_level:
        lda #<level_data
        sta tp
        lda #>level_data
        sta tp + 1
        ldx level
        beq .ll2
.ll1:   clc                             ; Zeiger um 60 je Level weiter (Schleife statt Multiplikation)
        lda tp
        adc #60
        sta tp
        bcc +
        inc tp + 1
+       dex
        bne .ll1
.ll2:   ldy #0                          ; Y = Index in den Leveldaten (0..59)
        ldx #0                          ; X = Index in der Karte (0..119)
        stx bricks
.ll3:   lda (tp),y
        pha
        lsr
        lsr
        lsr
        lsr
        sta MAP,x
        jsr count_brick
        inx
        pla
        and #$0f
        sta MAP,x
        jsr count_brick
        inx
        iny
        cpy #60
        bne .ll3
        ldx level
        lda level_kind,x
        sta lkind
        lda #45                         ; Bonuslevel: 45 Sekunden
        sta btime
        lda #2
        sta bsub
        jsr draw_all
        jsr draw_status
        jmp place_ball
; count_brick: A = Steintyp: zaehlt zerstoerbare Steine (0 = leer und 11 = unzerstoerbar zaehlen nicht).   Veraendert nur bricks und die Flags (A bleibt).
count_brick:
        cmp #0
        beq +
        cmp #11
        beq +
        inc bricks
+       rts

; draw_all: Bildschirm leer, Waende (obere Zeile 1, Spalte 0 und 21 ab Zeile 2), alle 120 Steine, Kopfzeile.  Veraendert A, X, Y, t4, bidx und mehr (put_cell).
draw_all:
        jsr clear_screen
        lda #14                         ; Mehrfarbe (Bit 3) + blau (6): Wandfarbe
        sta txtcol
        ldx #0
.da1:   stx t4
        ldy #1
        lda #CH_WALL
        jsr put_cell                    ; obere Wand, Zeile 1
        ldx t4
        inx
        cpx #22
        bne .da1
        ldy #2
.da2:   sty t4
        ldx #0
        lda #CH_WALL
        jsr put_cell                    ; linke Wand
        ldx #21
        ldy t4
        lda #CH_WALL
        jsr put_cell                    ; rechte Wand
        ldy t4
        iny
        cpy #23
        bne .da2
        ldx #0
.da3:   stx bidx
        jsr draw_brick_idx
        ldx bidx
        inx
        cpx #120
        bne .da3
        jmp draw_hud

; draw_brick_idx: Stein bidx (Index in der Karte) zeichnen (zwei Zeichen: links, rechts; Typ 0 = Leerzeichen); setzt bcx/bcy = Bildschirmspalte/-zeile
;   des linken Zeichens.  Veraendert A, X, Y, bcx, bcy, txtcol, sp, cp (t0..t3 bleiben unberuehrt).
draw_brick_idx:
        ldx #0                          ; Zeile = bidx / 10, Spalte = bidx mod 10 (Schleife statt Division)
        lda bidx
.db1:   cmp #10
        bcc .db2
        sbc #10
        inx
        bne .db1
.db2:   asl                             ; Steinspalte * 2 + 1 = Bildschirmspalte
        clc
        adc #1
        sta bcx
        txa
        clc
        adc #2
        sta bcy
        ldx bidx
        ldy MAP,x
        lda brick_col,y
        sta txtcol
        lda #CH_BRICK_L
        cpy #0
        bne .db3
        lda #CH_SPACE                   ; leerer Platz: Leerzeichen in weiss
        ldy #1
        sty txtcol
.db3:   ldx bcx
        ldy bcy
        jsr put_cell
        ldx bidx
        lda MAP,x
        tay
        lda #CH_BRICK_R
        cpy #0
        bne .db4
        lda #CH_SPACE
.db4:   ldx bcx
        inx
        ldy bcy
        jmp put_cell

; Farb-RAM-Wert je Steintyp (Index = Typ 0..13): 8 + Farbe (Bit 3 = Mehrfarbzeichen): Typ 0 leer (weiss, Wert 1), a..h rot, gelb, gruen, tuerkis, blau,
; violett, weiss, rot; 9 Silber weiss, 10 Gold gelb, 11 unzerstoerbar schwarz (dunkles Metall), 12 explosiv rot, 13 Ueberraschung weiss
brick_col: .byte 1, 10, 15, 13, 11, 14, 12, 9, 10, 9, 15, 8, 10, 9
; Punkte je Treffer bzw. Zerstoerung nach dem Typ des Steins (BCD-Byte fuer sc0, also Einer/Zehner): 10 * Typ fuer 1..8, Silber 50, Gold 90, unzerstoerbar 0,
;   explosiv/Ueberraschung 10.  Silber/Gold geben bei jedem Treffer die Punkte ihres Typs (hit_brick, .hb_dmg: Gold 90, dann Silber 50, dann als Typ 1 beim
;   Zerstoeren 10); eine Explosion zaehlt den aktuellen Typ.
brick_pts: .byte 0, $10, $20, $30, $40, $50, $60, $70, $80, $50, $90, 0, $10, $10

; draw_hud: Kopfzeile (Zeile 0) in weiss neu aufbauen: Text "LEVEL" und "BALL" mit Platz fuer die Zahlen, dann draw_nums.  Veraendert A, X, Y, t1..t3 u. a.
;   Belegung der Zeile: Spalte 0-5 Punkte, 7-11 "LEVEL", 13-14 Levelnummer, 16-19 "BALL", 21 Anzahl Leben (= Baelle).
draw_hud:
        lda #1
        sta txtcol
        lda #<h_text
        sta tp
        lda #>h_text
        sta tp + 1
        ldx #0
        ldy #0
        jsr puts                        ; Text mit Platz fuer die Zahlen
        jmp draw_nums
h_text: .screen "       level    ball  "     ; Kleinbuchstaben ergeben die Bildschirmcodes 1..26, im Zeichensatz also A..Z
        .byte 0

; draw_nums: Punkte (6 Stellen aus sc2, sc1, sc0), Level (2 Stellen, level + 1) und Leben (1 Stelle) in die Kopfzeile.  Veraendert A, X, Y, t0..t3, sp, cp.
draw_nums:
        lda #1
        sta txtcol
        lda sc2
        ldx #0
        ldy #0
        jsr put_bcd
        lda sc1
        ldx #2
        ldy #0
        jsr put_bcd
        lda sc0
        ldx #4
        ldy #0
        jsr put_bcd
        lda level                       ; Level + 1 als BCD (Level < 22 -> Zehner per Schleife)
        clc
        adc #1
        ldx #0
.dn1:   cmp #10
        bcc .dn2
        sbc #10
        inx
        bne .dn1
.dn2:   sta t1
        txa
        asl
        asl
        asl
        asl
        ora t1
        ldx #13
        ldy #0
        jsr put_bcd
        lda lives
        clc
        adc #$30
        ldx #21
        ldy #0
        jmp put_cell

; ---------------------------------------------------------------------------------------------------------------------
; Spielschleife
; ---------------------------------------------------------------------------------------------------------------------
; play: spielt bis zum Ende eines Levels oder Balls.  Rueckgabe A: 0 = Level geschafft, 1 = Ball verloren, 2 = Abbruch.
;   Ein Durchlauf der Schleife = ein Bild (50 Hz):
;     1. auf die Rastermarke warten und die Zeichen-Sprites fuer den Zustand des vorigen Durchlaufs setzen (unterhalb der Anzeige, flimmerfrei)
;     2. Musik (nur Demo) und Klangeffekte weiterzaehlen; Eingabe holen (Demo: Rechner) -> Abbruch / Pause
;     3. Zeitgeber, Schlaeger, Muenze; dann haftet der Ball (folgt dem Schlaeger, Feuer schiesst ab) oder er fliegt (move_ball)
;   Veraendert alle Register und viele Zeropage-Zellen.
play:   jsr wait_frame                  ; Zeichen-Sprites zuerst setzen (Zustand des vorigen Durchlaufs), solange der Strahl unten steht
        jsr tsp_render
        lda demo
        beq +
        jsr mus_tick                    ; in der Demo spielt die Musik weiter (Effekte sind dort stumm)
+       jsr sfx_update
.ifdef TESTLOSE
        lda #IN_FIRE | IN_RIGHT         ; Testfassung: schiessen, Schlaeger nach rechts aussen -> der Ball geht verloren
.else
        lda demo
        beq .pi1
        jsr auto_input                  ; Demo-Modus: der Rechner spielt ...
        pha                             ; (seine Maske merken, input_poll ueberschreibt A)
        jsr input_poll                  ; ... bis der Spieler irgendetwas drueckt
        beq .pi0                        ; Maske 0: nichts gedrueckt
        pla
        lda #2                          ; Abbruch zum Titel
        rts
.pi0:   pla
        jmp .pi2
.pi1:   jsr input_poll                  ; Eingabe dieses Bildes
.pi2:
.endif
        sta mask
        and #IN_ABORT
        beq +
        lda #2                          ; RUN/STOP: Abbruch zum Titel
        rts
+       lda mask
        and #IN_PAUSE
        beq +
        jsr pause                       ; P: Pause bis zum naechsten Druck auf P
+       jsr tick_timers                 ; Extra-Timer und Bonuszeit
        jsr move_paddle
        jsr update_cap                  ; fallende Muenze
        lda stuck
        beq .live
        jsr place_ball                  ; Ball klebt am Schlaeger und folgt ihm
        lda mask
        and #IN_FIRE
        beq play
        jsr launch
        jmp play
.live:  jsr move_ball                   ; A = 1: Ball unten heraus
        cmp #1
        beq .gone
        lda bricks
        bne play                        ; noch Steine: naechstes Bild
        lda #SFX_EXTRA                  ; Level geschafft
        jsr sfx_play
        ldx #50                         ; eine Sekunde (50 Bilder) Klang ausklingen lassen
        jsr wait_frames
        lda #0
        rts
.gone:  lda #SFX_LOSE
        jsr sfx_play
        ldx #40                         ; 0,8 s
        jsr wait_frames
        lda #1
        rts

; auto_input: Rechnerspieler (Demo): Schlaeger folgt dem Ball, Feuer wenn der Ball haftet.  Ausgabe A = Eingabemaske (IN_FIRE, IN_LEFT, IN_RIGHT).
;   Veraendert A, X, t0..t2.  Der Schlaeger bewegt sich erst, wenn die Mitte des Balls mindestens 4 Punkte links oder 3 Punkte rechts von der
;   Schlaegermitte liegt (kleine, unsymmetrische Totzone gegen Zittern).
auto_input:
        lda #0
        ldx stuck
        beq +
        lda #IN_FIRE
+       sta t0
        lda bxh
        clc
        adc #2                          ; Ballmitte (Ball ist 5 Punkte breit)
        sta t1
        lda padw
        lsr
        clc
        adc padx                        ; Schlaegermitte
        sta t2
        lda t1
        sec
        sbc t2                          ; Ballmitte - Schlaegermitte (vorzeichenbehaftet in 8 Bit)
        bcs .ai1
        cmp #$fd                        ; Unterschied -3..-1 (Ball bis 3 Punkte links): ruhig; kleiner (>= 4 Punkte links): nach links
        bcs .ai9
        lda t0
        ora #IN_LEFT
        rts
.ai1:   cmp #3                          ; Ball ab 3 Punkte rechts der Mitte: nach rechts
        bcc .ai9
        lda t0
        ora #IN_RIGHT
        rts
.ai9:   lda t0
        rts

; pause: wartet, bis P losgelassen, dann erneut gedrueckt und wieder losgelassen wurde.  Der Bildschirm bleibt stehen (Sprites bleiben gesetzt), die
;   Spielzeit steht still, weil play nicht weiterlaeuft.  Veraendert A, X, Y.
pause:  jsr input_wait_release
.pa1:   jsr wait_frame
        jsr input_poll
        and #IN_PAUSE
        beq .pa1
        jmp input_wait_release

; draw_status: Bonuslevel: verbleibende Sekunden in Zeile 22 ("BONUS 45": Text ab Spalte 6 in gelb, Zahl ab Spalte 12); sonst nichts.
;   Veraendert A, X, Y, t1..t3, tp, txtcol, sp, cp.
draw_status:
        lda lkind
        beq .ds9
        lda #7
        sta txtcol
        lda #<s_bonus
        sta tp
        lda #>s_bonus
        sta tp + 1
        ldx #6
        ldy #22
        jsr puts
        lda btime                       ; Zahl 0..99 -> zwei BCD-Ziffern (Zehner per Abziehen)
        ldx #0
.ds1:   cmp #10
        bcc .ds2
        sbc #10
        inx
        bne .ds1
.ds2:   sta t1
        txa
        asl
        asl
        asl
        asl
        ora t1
        ldx #12
        ldy #22
        jmp put_bcd
.ds9:   rts
s_bonus: .screen "bonus  "              ; zwei Leerzeichen; die Zahl ersetzt das zweite
         .byte 0

; tick_timers: einmal je Bild; alle 25 Bilder (0,5 s) zaehlen die Extra-Timer (tpad: Schlaegerbreite, tspd: Tempo) herunter; sind sie abgelaufen, gilt wieder
;   Breite 24 bzw. spdmod = 0 (das Tempo des Balls aendert sich aber erst beim naechsten Schlaegertreffer, siehe set_vel).  Im Bonuslevel zaehlt bsub/btime
;   die Zeit herunter (laeuft auch, solange der Ball noch am Schlaeger haftet); bei 0 ist das Level zu Ende (bricks = 0).  Veraendert A, tick, tpad, ...
tick_timers:
        dec tick
        bne .tt9
        lda #25
        sta tick
        lda tpad
        beq +
        dec tpad
        bne +
        lda #24                         ; Zeit um: Schlaeger wieder normal breit
        sta padw
+       lda tspd
        beq +
        dec tspd
        bne +
        lda #0
        sta spdmod
+       lda lkind
        beq .tt9
        dec bsub
        bne .tt9
        lda #2
        sta bsub
        dec btime
        jsr draw_status
        lda btime
        bne .tt9
        lda #0
        sta bricks                      ; Zeit um: Level zu Ende
.tt9:   rts

; move_paddle: Schlaeger nach der Eingabe (mask) bewegen (3 Punkte je Bild), auf 8..(168 - padw) begrenzen; pmov = 0/1/2 (ruhig/links/rechts) fuer paddle_hit.
;   Links und rechts zugleich heben sich auf (pmov = 2).  Veraendert A, padx, pmov.
move_paddle:
        lda #0
        sta pmov
        lda mask
        and #IN_LEFT
        beq .mp1
        lda padx
        sec
        sbc #3
        cmp #8
        bcs +
        lda #8
+       sta padx
        lda #1
        sta pmov
.mp1:   lda mask
        and #IN_RIGHT
        beq .mp9
        lda padx
        clc
        adc #3
        sta padx
        lda #168
        sec
        sbc padw
        cmp padx
        bcs +
        sta padx
+       lda #2
        sta pmov
.mp9:   lda padw                        ; nach einer Breitenaenderung (Muenze B/S, Ablauf) sicher in den Grenzen bleiben: padx + padw <= 168
        clc
        adc padx
        cmp #169
        bcc +
        lda #168
        sec
        sbc padw
        sta padx
+       rts

; place_ball: Ball haftet mittig auf dem Schlaeger (Ballmitte = Schlaegermitte, Unterkante auf der Oberkante des Schlaegers), stuck = 1.  Veraendert A.
;   Die Geschwindigkeit bleibt unberuehrt; launch setzt sie neu.
place_ball:
        lda #1
        sta stuck
        lda padw
        lsr
        clc
        adc padx
        sec
        sbc #2                          ; linke Kante = Mitte - 2 (der Ball ist 5 Punkte breit)
        sta bxh
        lda #0
        sta bxl
        sta byl
        lda #PADY - 5                   ; Oberkante des Balls: Ball (5 hoch) sitzt genau auf dem Schlaeger
        sta byh
        rts

; launch: Ball abschiessen (Zone 4 = leicht nach rechts oben, +9 Grad), Klang SFX_START.  Veraendert A, X, Y, t0, stuck, zone, vx/vy.
launch:
        lda #0
        sta stuck
        lda #4
        sta zone
        jsr set_vel
        lda #SFX_START
        jmp sfx_play

; set_vel: Geschwindigkeit aus Stufe (speed, mit Extra spdmod) und Zone (zone).  dir_tab liefert vx, vy als vorzeichenbehaftetes Byte in 1/32 Punkt;
;   gespeichert wird 8.8 (1/256 Punkt): Wert * 8 als 16-Bit-Zahl (niedriges Byte = Wert << 3, hohes Byte = Wert >> 5 mit Vorzeichen).
;   Eingabe: speed 0..7, spdmod (0 normal, 1 = Stufe + 3 hoechstens 7, 2 = Stufe - 3 mindestens 0), zone 0..7.  Ausgabe: vxl..vyh.  Veraendert A, X, Y, t0.
;   dir_tab-Index = (Stufe * 8 + Zone) * 2; die Geschwindigkeit wird nur beim Abschuss und bei Schlaegertreffern neu bestimmt.
set_vel:
        lda speed
        ldx spdmod
        beq .sv3
        cpx #1
        bne .sv1
        clc
        adc #3                          ; schneller
        cmp #8
        bcc .sv3
        lda #7
        bne .sv3
.sv1:   sec
        sbc #3                          ; langsamer
        bcs .sv3
        lda #0
.sv3:   asl
        asl
        asl                             ; Stufe * 8
        clc
        adc zone
        asl                             ; * 2: je (vx, vy) zwei Byte
        tax
        lda dir_tab,x
        jsr ext8
        sta vxl
        sty vxh
        lda dir_tab + 1,x
        jsr ext8
        sta vyl
        sty vyh
        rts
; ext8: A = Wert (signed) -> A = (Wert * 8) niedrig, Y = (Wert * 8) hoch.   Veraendert t0, Y.
;   Das Hochbyte ist Wert / 32 (arithmetisch geschoben, Vorzeichen bleibt), das Tiefbyte (Wert * 8) mod 256.
ext8:   sta t0
        cmp #$80                        ; Vorzeichen in den Uebertrag, dann fuenfmal arithmetisch nach rechts (CMP #$80 setzt C = 1 bei A >= $80)
        ror
        cmp #$80
        ror
        cmp #$80
        ror
        cmp #$80
        ror
        cmp #$80
        ror
        tay
        lda t0
        asl
        asl
        asl
        rts

; ---------------------------------------------------------------------------------------------------------------------
; move_ball: Ball um einen Schritt bewegen: erst X, dann Y; bei Treffer (Wand, Stein) zurueck und Richtung der Achse umkehren.
;   Rueckgabe A: 0 = weiter, 1 = Ball unten heraus.  Veraendert A, X, Y, Ballposition und -geschwindigkeit, t0..t5, MAP u. a. (Steintreffer, Schlaegerabprall).
;   Getrennte Achsen: bei einem Treffer ist klar, welche Richtung umzukehren ist.  Ein getroffener Stein wird schon in probe_ball verarbeitet (zerstoert).
;   Der Schlaeger wird nur beim Fallen (vy >= 0) und nur, wenn in Y kein Treffer war, geprueft.
; ---------------------------------------------------------------------------------------------------------------------
move_ball:
        clc                             ; --- X-Achse
        lda bxl
        adc vxl
        sta bxl
        lda bxh
        adc vxh
        sta bxh
        jsr probe_ball                  ; A <> 0: eine Ecke liegt in einer Wand oder einem Stein
        beq .mby
        sec                             ; Treffer: zurueck, vx = -vx
        lda bxl
        sbc vxl
        sta bxl
        lda bxh
        sbc vxh
        sta bxh
        sec
        lda #0
        sbc vxl
        sta vxl
        lda #0
        sbc vxh
        sta vxh
.mby:   clc                             ; --- Y-Achse
        lda byl
        adc vyl
        sta byl
        lda byh
        adc vyh
        sta byh
        jsr probe_ball
        beq .mpad
        sec
        lda byl
        sbc vyl
        sta byl
        lda byh
        sbc vyh
        sta byh
        sec
        lda #0
        sbc vyl
        sta vyl
        lda #0
        sbc vyh
        sta vyh
        jmp .mend                       ; (nach einem Treffer in Y gibt es in diesem Bild keinen Schlaegerabprall)
.mpad:  lda vyh                         ; Schlaeger nur beim Fallen pruefen
        bmi .mend
        lda byh
        clc
        adc #4                          ; Unterkante des Balls
        cmp #PADY
        bcc .mend                       ; noch darueber
        cmp #PADY + 4
        bcs .mend                       ; schon tiefer als die Schlaegerkante: vorbei
        lda bxh                         ; waagerechte Ueberdeckung: Ball [bxh, bxh + 4] gegen Schlaeger [padx, padx + padw - 1]
        clc
        adc #4
        cmp padx
        bcc .mend
        lda padx
        clc
        adc padw
        sta t0
        lda bxh
        cmp t0
        bcs .mend
        jsr paddle_hit
.mend:  lda byh
        cmp #176                        ; Oberkante des Balls ab Punkt 176 (der Schlaeger liegt bei 170..174): unten heraus
        bcc +
        lda #1
        rts
+       lda #0
        rts

; paddle_hit: Abprall am Schlaeger.  Zone = (Ballmitte - padx) / (padw / 8), mit Einfluss der Schlaegerbewegung (links -1, rechts +1), dann neue Geschwindigkeit.
;   Die 8 Zonen sind je 2, 3 oder 4 Punkte breit (Schlaeger 16, 24, 32); Zone 0 = -60 Grad (links), 7 = +60 Grad (rechts), vgl. tools/gen.py (ANG).  Jeder 8. Treffer
;   erhoeht die Stufe (hoechstens 7).  Der Ball wird auf den Schlaeger gesetzt (byh = PADY - 5) und klingt SFX_PADDLE.  Veraendert A, X, t0, t1, zone, hits, speed, vx/vy.
paddle_hit:
        lda bxh
        clc
        adc #2                          ; Ballmitte
        sec
        sbc padx
        bcs +
        lda #0                          ; Ballmitte links vom Schlaeger (Randtreffer): zaehlt als Zone 0
+       sta t0                          ; Abstand von der linken Kante (>= 0)
        lda padw
        lsr
        lsr
        lsr
        sta t1                          ; Zonenbreite: 2, 3 oder 4
        ldx #0
.ph1:   lda t0                          ; Division durch wiederholtes Abziehen der Zonenbreite; X = Zone, hoechstens 7
        cmp t1
        bcc .ph2
        sbc t1
        sta t0
        inx
        cpx #7
        bcc .ph1
.ph2:   stx zone
        lda pmov
        cmp #1
        bne .ph3
        lda zone
        beq .ph5
        dec zone
        jmp .ph5
.ph3:   cmp #2
        bne .ph5
        lda zone
        cmp #7
        bcs .ph5
        inc zone
.ph5:   inc hits
        lda hits
        and #7
        bne +
        lda speed
        cmp #7
        bcs +
        inc speed                       ; alle 8 Treffer eine Stufe schneller
+       jsr set_vel
        lda #PADY - 5                   ; Ball sitzt genau auf dem Schlaeger
        sta byh
        lda #0
        sta byl
        lda #SFX_PADDLE
        jmp sfx_play

; probe_ball: die vier Ecken des Balls (5 x 5 Punkte: (bxh, byh) bis (bxh + 4, byh + 4)) gegen Waende und Steine pruefen.  Rueckgabe A <> 0 bei Treffer
;   (Z-Flag passend).  Getroffene Steine werden dabei schon verarbeitet (cell_test -> hit_brick).  Veraendert A, X, t0..t2, t5, lasthit, bidx u. a.
probe_ball:
        lda #0
        sta t5                          ; Trefferflag
        lda #$ff
        sta lasthit                     ; kein Stein bisher getroffen
        lda bxh
        ldx byh
        jsr cell_test
        lda bxh
        clc
        adc #4
        ldx byh
        jsr cell_test
        lda bxh
        ldx byh
        inx
        inx
        inx
        inx
        jsr cell_test
        lda bxh
        clc
        adc #4
        ldx byh
        inx
        inx
        inx
        inx
        jsr cell_test
        lda t5
        rts

; cell_test: Punkt (A = X, X = Y in Bildpunkten) gegen Wand/Stein.  Trifft er einen Stein, wird der Treffer verarbeitet (einmal je Stein und Pruefung).
;   t5 wird bei Treffer 1.  Veraendert A, X, t0..t2 und, bei einem Stein, alles was hit_brick veraendert (bidx, lasthit, Karte, Punkte, Muenze).
;   Zelle = Punkt / 8; Spalte 0 und 21 sowie Zeile 0/1 = Wand, Zeilen 2..13 = Steinfeld (Spalte 1..20 -> Steinspalte (Spalte - 1) / 2), darunter frei.
cell_test:
        lsr
        lsr
        lsr
        sta t0                          ; Spalte
        txa
        lsr
        lsr
        lsr
        sta t1                          ; Zeile
        lda t0
        beq .wall                       ; Spalte 0: linke Wand
        cmp #21
        bcs .wall                       ; Spalte 21: rechte Wand
        lda t1
        cmp #2
        bcc .wall                       ; Zeile 0/1: oben
        cmp #14
        bcs .none                       ; unter den Steinen
        sec
        sbc #2                          ; Steinzeile 0..11
        sta t2
        asl
        asl
        asl
        clc
        adc t2
        adc t2                          ; * 10
        sta t2
        lda t0
        sec
        sbc #1
        lsr                             ; Steinspalte 0..9
        clc
        adc t2
        tax                             ; X = Index in der Karte
        lda MAP,x
        beq .none
        cpx lasthit
        beq .hit                        ; derselbe Stein wie vorher in dieser Pruefung: nicht noch einmal
        stx lasthit
        stx bidx
        jsr hit_brick
.hit:   lda #1
        sta t5
.none:  rts
.wall:  lda #1
        sta t5
        rts

; hit_brick: Stein bidx wurde getroffen.  Typ 11: nichts; 10 Gold -> 9; 9 Silber -> 1; sonst zerstoert (Punkte, Karte, Bild, ggf. Muenze, explosiv: Nachbarn).
;   Klang: SFX_HARD bei unzerstoerbar/Gold/Silber, sonst SFX_BRICK.  Muenze: Typ 13 immer, sonst mit Wahrscheinlichkeit 1/8 (Testfassung TESTCAP: immer).  Gold/Silber: Punkte je Treffer.
;   Veraendert A, X, Y, htype, bricks, sc0..sc2, MAP und was kill_brick/explode/spawn_cap veraendern.
hit_brick:
        ldx bidx
        lda MAP,x
        cmp #11
        beq .hb_hard
        cmp #10
        beq .hb_dmg
        cmp #9
        beq .hb_dmg
        tay
        sty htype                       ; Typ merken (kill_brick veraendert t0..t3 und X, Y)
        jsr kill_brick
        lda htype
        cmp #12
        bne +
        jsr explode
+       lda htype
        cmp #13
        beq .hb_coin                    ; Ueberraschungsstein: immer eine Muenze
.ifndef TESTCAP
        jsr rand_byte
        and #7                          ; 1 von 8
        bne .hb_snd
.endif
.hb_coin:
        jsr spawn_cap
.hb_snd:
        lda #SFX_BRICK
        jmp sfx_play
.hb_dmg:
        tay                             ; A = Typ (9 oder 10): jeder Treffer gibt die Punkte des Typs (brick_pts 50/90), wie auf C64 und Plus/4
        lda brick_pts,y
        jsr add_score
        jsr draw_score                  ; (veraendert X)
        ldx bidx
        dec MAP,x                       ; Gold (10) -> Silber (9); Silber (9) -> 8: das wird sofort durch Typ 1 ersetzt (naechster Treffer zerstoert)
        lda MAP,x
        cmp #8
        bne +
        lda #1                          ; Silber nach einem Treffer: normaler Stein (Typ 1)
        sta MAP,x
+       jsr draw_brick_idx
.hb_hard:
        lda #SFX_HARD
        jmp sfx_play

; kill_brick: Stein bidx entfernen: Punkte nach Typ (brick_pts), Karte leeren, Zaehler (bricks), Bild, Punkteanzeige.   Veraendert A, X, Y, t0, t2, t3, sc0..sc2.
kill_brick:
        ldx bidx
        ldy MAP,x
        lda brick_pts,y
        jsr add_score
        ldx bidx
        lda #0
        sta MAP,x
        dec bricks
        jsr draw_brick_idx
        jmp draw_score

; explode: explosiver Stein: die bis zu acht Nachbarn (ausser unzerstoerbaren) zerstoeren.  Mitte = bidx (bleibt erhalten; sie ist schon leer).
;   Ruft fuer jeden Nachbarn kill_brick (auch Silber und Gold sind dann mit einem Schlag weg und geben 50/90 Punkte).  Keine Kettenreaktion: weitere explosive
;   Nachbarn werden nur zerstoert, nicht gezuendet.  Veraendert A, X, Y, t4, ex_*, bidx (am Ende wiederhergestellt) und alles aus kill_brick.
explode:
        lda bidx
        sta t4                          ; Mitte merken
        ldx #0                          ; Zeile/Spalte der Mitte aus dem Index
.ex1:   cmp #10
        bcc .ex2
        sbc #10
        inx
        bne .ex1
.ex2:   sta ex_c                        ; Spalte
        stx ex_r                        ; Zeile
        ldy #0                          ; Y = Nachbar 0..8 (Zeile -1..+1, Spalte -1..+1)
.ex3:   sty ex_i
        lda ex_dr,y
        clc
        adc ex_r
        bmi .exn                        ; Zeile < 0
        cmp #12
        bcs .exn
        sta ex_n                        ; Nachbarzeile
        asl
        asl
        asl
        clc
        adc ex_n
        adc ex_n                        ; * 10
        sta ex_n
        lda ex_dc,y
        clc
        adc ex_c
        bmi .exn
        cmp #10
        bcs .exn
        clc
        adc ex_n
        tax                             ; Index des Nachbarn
        lda MAP,x
        beq .exn
        cmp #11
        beq .exn
        stx bidx
        jsr kill_brick
.exn:   ldy ex_i
        iny
        cpy #9
        bne .ex3
        lda t4
        sta bidx
        rts
ex_dr: .byte $ff, $ff, $ff, 0, 0, 1, 1, 1, 0         ; Zeilenabstand der Nachbarn 0..8 (signed: -1, 0, +1); Nachbar 8 = die Mitte selbst
ex_dc: .byte $ff, 0, 1, $ff, 1, $ff, 0, 1, 0         ; Spaltenabstand dazu

; add_score: A = Punkte (BCD-Byte 00..99, Einer/Zehner), auf sc0/sc1/sc2 addieren (Dezimalmodus, Uebertrag in die hoeheren Bytes).  Veraendert A.
;   Das D-Flag ist nur waehrend der Addition gesetzt (die IRQs sind ohnehin gesperrt).
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

; draw_score: nur die Punkte (6 Stellen) der Kopfzeile neu zeichnen.  Veraendert A, X, Y, t0, t2, t3, txtcol, sp, cp.
draw_score:
        lda #1
        sta txtcol
        lda sc2
        ldx #0
        ldy #0
        jsr put_bcd
        lda sc1
        ldx #2
        ldy #0
        jsr put_bcd
        lda sc0
        ldx #4
        ldy #0
        jmp put_bcd

; ---------------------------------------------------------------------------------------------------------------------
; Muenzen (Extras): fallen aus zerstoerten Steinen; Art 0 B breiter Schlaeger, 1 S schmaler, 2 L langsamer, 3 F schneller, 4 + Extraball, 5 P 500 Punkte.
; ---------------------------------------------------------------------------------------------------------------------
; spawn_cap: neue Muenze am zuletzt gezeichneten Stein (bcx, bcy) starten, falls keine faellt.  Die Art ist Zufall 0..7, wobei 6 und 7 auf 0 und 1 zurueckfallen
;   (B und S sind damit doppelt so haeufig wie die anderen).  Veraendert A, capx, capy, captype, capact.
spawn_cap:
        lda capact
        bne .sc9
        lda bcx                         ; linkes Zeichen des Steins: Bildpunkt = Spalte * 8
        asl
        asl
        asl
        sta capx
        lda bcy
        asl
        asl
        asl
        sta capy
        jsr rand_byte
        and #7
        cmp #6
        bcc +
        sbc #6
+       sta captype
        lda #1
        sta capact
.sc9:   rts

; update_cap: Muenze fallen lassen (1 Punkt je Bild) und mit dem Schlaeger auffangen (Unterkante der Muenze in PADY .. PADY + 5 und waagerecht ueber dem
;   Schlaeger).  Verpasst sie den Schlaeger (capy >= 176), verschwindet sie.  Gefangen: SFX_EXTRA und die Wirkung nach Art; die Zeiten tpad/tspd zaehlen 40
;   Halbsekunden (= 20 s); Breite wirkt sofort, Tempo (spdmod) erst beim naechsten set_vel, also beim naechsten Schlaegertreffer.  Veraendert A, X, capact, ...
update_cap:
        lda capact
        beq .uc9
        inc capy
        lda capy
        cmp #176
        bcc +
        lda #0
        sta capact                      ; verpasst
        rts
+       clc
        adc #8                          ; Unterkante
        cmp #PADY
        bcc .uc9
        cmp #PADY + 6
        bcs .uc9
        lda capx                        ; waagerechte Ueberdeckung: Muenze [capx, capx + 7] gegen [padx, padx + padw - 1]
        clc
        adc #8
        cmp padx
        bcc .uc9
        beq .uc9
        lda padx
        clc
        adc padw
        cmp capx
        bcc .uc9
        beq .uc9
        lda #0
        sta capact
        ; --- Wirkung
        lda #SFX_EXTRA
        jsr sfx_play
        ldx captype
        beq .ub
        cpx #1
        beq .us
        cpx #2
        beq .ul
        cpx #3
        beq .uf
        cpx #4
        beq .ulife
        lda #$05                        ; P: 500 Punkte (BCD $05 im Hunderter/Tausender-Byte sc1)
        sed
        clc
        adc sc1
        sta sc1
        lda sc2
        adc #0
        sta sc2
        cld
        jmp draw_score
.ub:    lda #32
        sta padw
        lda #40
        sta tpad
        rts
.us:    lda #16
        sta padw
        lda #40
        sta tpad
        rts
.ul:    lda #2
        sta spdmod
        lda #40
        sta tspd
        rts
.uf:    lda #1
        sta spdmod
        lda #40
        sta tspd
        rts
.ulife: lda lives
        cmp #9
        bcs .uc9                        ; hoechstens 9 Leben
        inc lives
        jmp draw_nums
.uc9:   rts

; ---------------------------------------------------------------------------------------------------------------------
; ZEICHEN-SPRITES: tsp_render setzt Schlaeger, Muenze und Ball als Pool-Zeichen auf den Bildschirm.
;   Jede Zelle, die ein Sprite beruehrt, bekommt ein Pool-Zeichen (Code POOL0 + n, Glyph im Zeichensatz-RAM); die Bildzeilen des Sprites werden um X mod 8 nach rechts
;   geschoben und in die Glyphen der linken und rechten Zelle gesetzt (ODER).  Der Hintergrund unter dem Pool-Zeichen ist leer oder wird verdeckt.
;   Rueckbau im naechsten Bild: aus der Steinkarte wird der Hintergrund jeder belegten Zelle neu berechnet (tsp_restore).
;   Hoechstens 13 Pool-Zeichen je Bild (Schlaeger bis 5 Zellen, Ball 4, Muenze 4), NPOOL = 20.  Die Zellfarbe ist die des zuletzt dort gezeichneten Sprites.
; ---------------------------------------------------------------------------------------------------------------------
; tsp_render: alle Sprites des aktuellen Spielzustands neu setzen (erst tsp_restore, dann Schlaeger, Muenze, Ball).  Veraendert alle Register, t0, t4, t5, bl_*.
;   Wird im Bildwechsel von play aufgerufen (solange der Strahl unter der Anzeige steht), damit nichts flackert.
tsp_render:
        jsr tsp_restore
        ; --- Schlaeger: Streifen zu 8 Punkten (links gerundet, Mitte voll, rechts gerundet), Farbe tuerkis
        lda #3
        sta bl_col
        lda #5
        sta bl_n
        lda padx
        sta bl_x
        lda #PADY
        sta bl_y
        lda padw
        lsr
        lsr
        lsr
        sta t4                          ; Anzahl Streifen (2, 3 oder 4)
        ldx #0
.tr1:   stx t5                          ; Streifennummer
        lda #<pad_m                     ; Vorgabe: Mittelstueck; erster Streifen links gerundet, letzter rechts gerundet
        ldy #>pad_m
        cpx #0
        bne +
        lda #<pad_l
        ldy #>pad_l
+       ldx t5
        inx
        cpx t4
        bne +
        lda #<pad_r
        ldy #>pad_r
+       sta bl_img
        sty bl_img + 1
        jsr blit
        lda bl_x
        clc
        adc #8
        sta bl_x
        ldx t5
        inx
        cpx t4
        bne .tr1
        ; --- Muenze: Glyph des Buchstabens aus dem Zeichensatz-RAM (8 Zeilen), Farbe nach Art
        lda capact
        beq .tr3
        ldx captype
        lda cap_code,x                  ; Bildschirmcode des Buchstabens
        sta t0
        lda #0
        sta bl_img + 1
        lda t0                          ; bl_img = CHARSET + Code * 8 (Glyph des Buchstabens im Zeichensatz-RAM)
        asl
        rol bl_img + 1
        asl
        rol bl_img + 1
        asl
        rol bl_img + 1
        sta bl_img
        lda bl_img + 1
        clc
        adc #>CHARSET
        sta bl_img + 1
        lda cap_color,x
        sta bl_col
        lda #8
        sta bl_n
        lda capx
        sta bl_x
        lda capy
        sta bl_y
        jsr blit
        ; --- Ball: 5 x 5 Punkte, weiss
.tr3:   lda #<ball_img
        sta bl_img
        lda #>ball_img
        sta bl_img + 1
        lda #1
        sta bl_col
        lda #5
        sta bl_n
        lda bxh
        sta bl_x
        lda byh
        sta bl_y
        jmp blit

; Bitmuster der Sprites, je Zeile ein Byte (hoechstes Bit = linkester Punkt), 5 Zeilen: Schlaegerstreifen zu 8 Punkten (links/rechts mit abgerundeter Ecke) und Ball
pad_l:    .byte $7f, $ff, $ff, $ff, $7f
pad_m:    .byte $ff, $ff, $ff, $ff, $ff
pad_r:    .byte $fe, $ff, $ff, $ff, $fe
ball_img: .byte $70, $f8, $f8, $f8, $70         ; 5 x 5 Punkte, Ecken abgerundet
cap_code:  .byte 2, 19, 12, 6, 43, 16           ; Bildschirmcodes: B S L F + P
cap_color: .byte 6, 2, 5, 7, 1, 3               ; blau, rot, gruen, gelb, weiss, tuerkis (Farb-RAM-Werte 0..7, einfarbig)

.org $2000                              ; Teil 2 des Programms in der Erweiterung ($1C00-$1FFF gehoert dem Zeichensatz-RAM, der VIC-I sieht nur Speicher unter $2000)
; blit: Bild (bl_img, bl_n Zeilen zu 8 Punkten) an Bildpunkt (bl_x, bl_y) in Farbe bl_col in die Pool-Glyphen setzen (ODER auf den vorhandenen Inhalt).
;   Je Bildzeile: Byte um X mod 8 nach rechts schieben (bl_l = Rest in der linken Zelle, bl_r = in die rechte Zelle gelaufene Bits), Zellzeile/Pixelzeile aus
;   Y + Zeile, Zelle(n) mit get_cell belegen und beschreiben.  Zeilen ab Bildschirmzeile 23 und Zellen rechts von Spalte 21 werden uebersprungen; ist der Pool
;   voll (get_cell: Carry), fehlt die Zelle einfach.  Veraendert A, X, Y, bl_i..bl_ln, gp, sp, cp, gc_r, gc_c, pool_n.
blit:
        lda #0
        sta bl_i
.bl1:   lda bl_i
        cmp bl_n
        bcs .bl9
        tay
        lda (bl_img),y
        sta bl_l
        lda #0
        sta bl_r
        lda bl_x
        and #7
        tax
        beq .bl3
.bl2:   lsr bl_l                        ; um X mod 8 Punkte nach rechts: links bleibt der Rest, rechts laeuft in die naechste Zelle
        ror bl_r
        dex
        bne .bl2
.bl3:   lda bl_y
        clc
        adc bl_i
        tax
        and #7
        sta bl_ln                       ; Pixelzeile in der Zelle
        txa
        lsr
        lsr
        lsr
        sta bl_cr                       ; Zellzeile
        cmp #23
        bcs .bl8                        ; unter dem Bildschirm
        lda bl_x
        lsr
        lsr
        lsr
        sta bl_cx
        ldx bl_cr
        ldy bl_cx
        jsr get_cell
        bcs .bl4
        ldy bl_ln
        lda (gp),y
        ora bl_l
        sta (gp),y
.bl4:   lda bl_r
        beq .bl8
        ldx bl_cr
        ldy bl_cx
        iny
        cpy #22
        bcs .bl8
        jsr get_cell
        bcs .bl8
        ldy bl_ln
        lda (gp),y
        ora bl_r
        sta (gp),y
.bl8:   inc bl_i
        jmp .bl1
.bl9:   rts

; get_cell: Zelle (X = Zeile, Y = Spalte) fuer ein Sprite belegen: ist dort schon ein Pool-Zeichen, wird es weiterbenutzt, sonst ein neues vergeben und geleert.
;   Rueckgabe: gp = Glyph der Zelle, Carry 0; Carry 1 = Pool voll.  Setzt die Zellfarbe auf bl_col.   Veraendert A, X, Y, sp, cp, gp, gc_r, gc_c, pool_n.
;   Eine neu vergebene Zelle wird in POOL_CR/POOL_CX vermerkt (fuer tsp_restore) und bekommt den Code POOL0 + pool_n.
get_cell:
        stx gc_r
        sty gc_c
        tya
        clc
        adc rowlo,x
        sta sp
        sta cp
        lda rowhi,x
        adc #0
        sta sp + 1
        clc
        adc #(>COLOR_RAM) - (>SCREEN_RAM)
        sta cp + 1
        ldy #0
        lda (sp),y
        cmp #POOL0
        bcs .gc2                        ; schon ein Pool-Zeichen
        ldx pool_n
        cpx #NPOOL
        bcs .gcfull
        lda gc_r
        sta POOL_CR,x
        lda gc_c
        sta POOL_CX,x
        txa
        clc
        adc #POOL0
        sta (sp),y                      ; Pool-Zeichen auf den Bildschirm
        inc pool_n
        jsr glyph_ptr                   ; gp = Glyph (A = Code)
        ldy #7
        lda #0
.gc1:   sta (gp),y                      ; Glyph leeren
        dey
        bpl .gc1
        jmp .gc3
.gc2:   jsr glyph_ptr
.gc3:   ldy #0
        lda bl_col
        sta (cp),y
        clc
        rts
.gcfull: sec
        rts

; glyph_ptr: A = Zeichencode -> gp = CHARSET + Code * 8 (Zeiger auf das Glyph im Zeichensatz-RAM).   Veraendert A.
glyph_ptr:
        sta gp
        lda #0
        sta gp + 1
        asl gp
        rol gp + 1
        asl gp
        rol gp + 1
        asl gp
        rol gp + 1
        lda gp + 1
        clc
        adc #>CHARSET
        sta gp + 1
        rts

; tsp_restore: die belegten Pool-Zellen des letzten Bildes wieder auf ihren Hintergrund setzen (Stein links/rechts oder leer), danach pool_n = 0.
;   Hintergrund je Zelle: im Steinfeld (Zeile 2..13, Spalte 1..20) der Stein aus MAP (Zeichen 64 oder 65 je nach Spaltenparitaet, Farbe brick_col), sonst
;   Leerzeichen in weiss.  Die Wand wird dabei nicht neu gezeichnet, weil die Sprites innerhalb des Spielfeldes bleiben.  Veraendert A, X, Y, t0..t5, sp, cp, pool_n.
;   Die Liste wird rueckwaerts abgearbeitet (t4 = Listenindex).
tsp_restore:
        ldx pool_n
        beq .tr9
.tr1:   dex
        stx t4                          ; Listenindex
        lda POOL_CR,x
        sta t0                          ; Zeile
        lda POOL_CX,x
        sta t1                          ; Spalte
        lda #CH_SPACE
        sta t2                          ; Zeichen
        lda #1
        sta t3                          ; Farbe
        lda t0
        cmp #2
        bcc .tr3                        ; oberhalb der Steine
        cmp #14
        bcs .tr3                        ; unterhalb
        lda t1
        beq .tr3
        cmp #21
        bcs .tr3                        ; Wand
        lda t0
        sec
        sbc #2
        sta t5
        asl
        asl
        asl
        clc
        adc t5
        adc t5                          ; Zeile * 10
        sta t5
        lda t1
        sec
        sbc #1
        pha
        lsr
        clc
        adc t5
        tax
        lda MAP,x
        beq .tr2
        tay
        lda brick_col,y
        sta t3
        pla
        and #1
        clc
        adc #CH_BRICK_L                 ; links (64) oder rechts (65)
        sta t2
        jmp .tr3
.tr2:   pla
.tr3:   ldx t1
        ldy t0
        jsr set_rowptr
        ldy #0
        lda t2
        sta (sp),y
        lda t3
        sta (cp),y
        ldx t4
        bne .tr1
.tr9:   lda #0
        sta pool_n
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Titelbild
; ---------------------------------------------------------------------------------------------------------------------
; title: Titelbild mit Logo (Farbwechsel), Untertitel, Top 5 der Bestenliste und Musik.  Kehrt zurueck, wenn Feuer gedrueckt wurde (demo = 0, Musik aus) oder
;   30 s lang keine Eingabe kam (demo = 1: der Rechner spielt, die Effekte schweigen).  In den ersten 0,8 s nach dem Aufbau zaehlt keine Eingabe (Schonfrist).
;   Testfassungen AUTOPLAY/TESTLOSE ueberspringen das Bild.  Veraendert alle Register.
title:
        lda #0
        sta demo
        sta sfx_mute
.ifdef AUTOPLAY
        lda #1                          ; Testfassung: der Rechner spielt von Anfang an
        sta demo
        sta sfx_mute
        rts
.endif
.ifdef TESTLOSE
        rts
.endif
        jsr clear_screen
        lda #$08
        sta $900f                       ; schwarzer Hintergrund
        jsr draw_logo
        lda #7                          ; gelb
        sta txtcol
        lda #<t_sub
        sta tp
        lda #>t_sub
        sta tp + 1
        ldx #0
        ldy #14
        jsr puts
        lda #1
        sta txtcol
        lda #<t_best
        sta tp
        lda #>t_best
        sta tp + 1
        ldx #5
        ldy #16
        jsr puts
        jsr draw_top5
        lda #3                          ; tuerkis
        sta txtcol
        lda #<t_start
        sta tp
        lda #>t_start
        sta tp + 1
        ldx #4
        ldy #22
        jsr puts
        ldx #<song                      ; Titelmusik von vorn (X/Y = Adresse des Liedkopfes)
        ldy #>song
        jsr mus_init
        jsr input_wait_release          ; erst loslassen, dann zaehlen
        lda #40                         ; Schonfrist: in den ersten 0,8 s zaehlt keine Eingabe
        sta idl
.t1:    jsr wait_frame
        jsr mus_tick
        dec idl
        bne .t1
        jsr input_wait_release
        lda #<1500                      ; 30 s ohne Eingabe -> Demo-Modus (1500 Bilder bei 50 Hz)
        sta idl
        lda #>1500
        sta idh
.tw:    jsr wait_frame
        jsr mus_tick
        inc frame
        lda frame
        and #3
        bne +
        jsr logo_fx
+       jsr input_poll
        tax                             ; Maske merken
        and #IN_FIRE
        bne .tstart                     ; Feuer: Spiel starten
        txa
        beq .tidle                      ; keine Eingabe: Leerlaufzaehler weiter
        lda #<1500                      ; andere Eingabe: Wartezeit neu starten
        sta idl
        lda #>1500
        sta idh
        bne .tw
.tidle: lda idl
        ora idh
        beq .tdemo                      ; Zeit abgelaufen
        lda idl                         ; 16-Bit-Zaehler herunterzaehlen (Ueberlauf von idl nach idh)
        bne +
        dec idh
+       dec idl
        jmp .tw
.tdemo: lda #1                          ; Demo: der Rechner spielt, die Soundeffekte schweigen (die Musik laeuft weiter)
        sta demo
        sta sfx_mute
        rts
.tstart: jsr mus_stop                   ; Spielbeginn: die Musik schweigt (nur auf dem Titel)
        rts

; Titeltexte (Bildschirmcodes: Kleinbuchstaben erscheinen als Grossbuchstaben), mit 0 abgeschlossen
t_sub:   .screen "action fuer den vc-20"     ; Zeile 14
         .byte 0
t_best:  .screen "bestenliste"               ; Zeile 16
         .byte 0
t_start: .screen "feuer = start"             ; Zeile 22
         .byte 0

; draw_logo: "PRALL" (Zeilen 2..6) und "BALL" (Zeilen 8..12) aus 10 Bitmustern zu je 24 Bit (Spalte 0 = hoechstes Bit), erzeugt von tools/gen.py (lib/logo.inc).
;   Jeder gesetzte Punkt ist ein Wandzeichen (Mehrfarbe); die Farbe wechselt je Zeile (Regenbogen, logo_fx).  Nur die ersten 22 der 24 Bit werden benutzt.
;   Veraendert A, X, Y, t0..t5, txtcol, sp, cp.
draw_logo:
        lda #0
        sta t4                          ; Logozeile 0..9
.dl1:   lda t4
        asl
        clc
        adc t4                          ; * 3
        tax
        lda logo_bits,x
        sta t0
        lda logo_bits + 1,x
        sta t1
        lda logo_bits + 2,x
        sta t2
        ldx t4
        lda logo_y,x
        sta t5                          ; Bildschirmzeile
        lda t4
        jsr logo_color                  ; A = Farbe
        sta txtcol
        ldx #0                          ; Spalte 0..21
.dl2:   asl t2
        rol t1
        rol t0
        bcc .dl3
        stx t3
        ldy t5
        lda #CH_WALL
        jsr put_cell
        ldx t3
.dl3:   inx
        cpx #22
        bne .dl2
        inc t4
        lda t4
        cmp #10
        bne .dl1
        rts

; logo_color: A = Logozeile 0..9 -> A = Farb-RAM-Wert (8 + Farbe) nach Zeile (mod 5) und Phase lph: jede Buchstabenzeile bekommt die naechste Regenbogenfarbe,
;   bei jedem Schritt von lph wandert das Muster um eine Zeile.  Veraendert X (und A als Ergebnis).
logo_color:
.lc1:   cmp #5
        bcc .lc2
        sbc #5
        bcs .lc1
.lc2:   clc
        adc lph
        and #7
        tax
        lda rainbow,x
        rts
rainbow: .byte 10, 15, 13, 11, 14, 12, 9, 15            ; rot, gelb, gruen, tuerkis, blau, violett, weiss, gelb (je 8 + Farbe)

; logo_fx: Farbwechsel im Logo (Aufruf alle 4 Bilder): Phase weiter, Farb-RAM aller Wandzeichen der Logozeilen neu setzen (nur Farbe, nicht das Zeichen).
;   Veraendert A, X, Y, t4, txtcol, sp, cp.
logo_fx:
        inc lph
        lda #0
        sta t4
.lf1:   lda t4
        jsr logo_color
        sta txtcol
        ldx t4
        ldy logo_y,x
        ldx #0
        jsr set_rowptr                  ; sp/cp auf Spalte 0 der Logozeile
        ldy #0
.lf2:   lda (sp),y
        cmp #CH_WALL
        bne +
        lda txtcol
        sta (cp),y
+       iny
        cpy #22
        bne .lf2
        inc t4
        lda t4
        cmp #10
        bne .lf1
        rts

; draw_top5: die ersten fuenf Bestenlisten-Eintraege (Platz in Spalte 1 gelb, Name ab Spalte 3, Punkte ab Spalte 12) in den Zeilen 17..21.
;   Eintrag k liegt bei hs_buf + HS_FIRST + 12 * k: 8 Zeichen Name (ASCII), dann Punkte niedrig/mittel/hoch (BCD), dann Level (hier nicht angezeigt).
;   Veraendert A, X, Y, t0..t5, tp, txtcol, sp, cp.
draw_top5:
        lda #<(hs_buf + HS_FIRST)
        sta tp
        lda #>(hs_buf + HS_FIRST)
        sta tp + 1
        lda #0
        sta t4                          ; Eintrag 0..4
.d51:   lda #7                          ; Platz in gelb
        sta txtcol
        lda t4
        clc
        adc #$31
        ldx #1
        pha
        lda t4
        clc
        adc #17
        tay
        pla
        jsr put_cell
        lda #1
        sta txtcol
        ldy #0
.d52:   sty t5                          ; Name: 8 Zeichen ab Spalte 3
        lda (tp),y
        and #$3f                        ; ASCII -> Bildschirmcode (A = 1, 0 = 48, Leerzeichen = 32, "-" = 45, "." = 46)
        pha
        lda t5
        clc
        adc #3
        tax
        lda t4
        clc
        adc #17
        tay
        pla
        jsr put_cell
        ldy t5
        iny
        cpy #8
        bne .d52
        ldy #10                         ; Punkte hoch (Spalte 12), mittel (14), niedrig (16)
        lda (tp),y
        ldx #12
        jsr .d5n
        ldy #9
        lda (tp),y
        ldx #14
        jsr .d5n
        ldy #8
        lda (tp),y
        ldx #16
        jsr .d5n
        clc
        lda tp
        adc #12
        sta tp
        bcc +
        inc tp + 1
+       inc t4
        lda t4
        cmp #5
        bne .d51
        rts
.d5n:   pha                             ; BCD-Byte A an Spalte X in Zeile 17 + t4
        lda t4
        clc
        adc #17
        tay
        pla
        jmp put_bcd

; ---------------------------------------------------------------------------------------------------------------------
; Spielende und Namenseingabe
; ---------------------------------------------------------------------------------------------------------------------
; game_over: "SPIEL ZU ENDE" zeigen; reichen die Punkte fuer die Bestenliste (hs_check), "NEUER REKORD!", Namenseingabe, Eintrag (hs_insert) und Speichern
;   (hs_save; bei Fehler "NICHT GESPEICHERT!").  Dann 3 s (150 Bilder) stehen lassen.  hs_level = erreichtes Level (1-basiert).  Veraendert alle Register.
game_over:
        jsr tsp_restore                 ; Ball und Schlaeger vom Bildschirm
        lda #1
        sta txtcol
        lda #<g_over
        sta tp
        lda #>g_over
        sta tp + 1
        ldx #4
        ldy #10
        jsr puts
        lda sc0                         ; Punkte in die Puffer der Bestenlisten-Bibliothek
        sta hs_points
        lda sc1
        sta hs_points + 1
        lda sc2
        sta hs_points + 2
        lda level
        clc
        adc #1
        sta hs_level
        jsr hs_check                    ; A = Platz 0..9 oder $FF
        cmp #$ff
        beq .nohs
        pha                             ; Platz fuer hs_insert merken (name_entry veraendert A)
        lda #7
        sta txtcol
        lda #<g_new
        sta tp
        lda #>g_new
        sta tp + 1
        ldx #4
        ldy #12
        jsr puts
        jsr name_entry
        pla
        jsr hs_insert
        sei                             ; vorsichtshalber: die Interrupts bleiben waehrend der seriellen Dateiroutinen gesperrt (siehe hiscore.asm)
        jsr hs_save                     ; auf Diskette schreiben; A = HS_OK oder Fehlercode
        ldx #1                          ; weiss (LDX, damit das Ergebnis in A erhalten bleibt)
        stx txtcol
        cmp #HS_OK
        bne .bad
        lda #<g_saved
        sta tp
        lda #>g_saved
        sta tp + 1
        ldx #5                          ; Spalte der Meldung "gespeichert"
        bne .msg                        ; (immer: X = 5)
.bad:   lda #<g_nosave                  ; Speichern ging schief (keine Diskette, schreibgeschuetzt, ...): nur melden
        sta tp
        lda #>g_nosave
        sta tp + 1
        ldx #2                          ; Spalte der Meldung "nicht gespeichert!" (18 Zeichen)
.msg:   ldy #19
        jsr puts
.nohs:  ldx #150                        ; drei Sekunden anzeigen
        jmp wait_frames

; Texte des Spielende-Bildes (Bildschirmcodes, mit 0 abgeschlossen)
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
g_help:  .screen "tippen + return"
         .byte 0

; name_entry: 8 Zeichen, Vorgabe "SPIELER".  Tippen ersetzt die Vorgabe (erstes Zeichen loescht sie), DEL loescht, RETURN bestaetigt; ohne Eingabe nach 30 s automatisch.
;   Der Name steht als ASCII in hs_name.  Erlaubt sind A-Z, 0-9, Leerzeichen, "-" und "." (input_char liefert nur Grossbuchstaben); alles andere wird
;   ignoriert.  Pro Bild wird ein Zeichen abgeholt (input_char liefert eine Taste nur einmal, bis sie losgelassen wurde).  Veraendert alle Register.
name_entry:
        lda #1
        sta txtcol
        lda #<g_name
        sta tp
        lda #>g_name
        sta tp + 1
        ldx #3
        ldy #14
        jsr puts
        lda #<g_help
        sta tp
        lda #>g_help
        sta tp + 1
        ldx #3
        ldy #17
        jsr puts
        ldx #7
.ne0:   lda ne_default,x
        sta hs_name,x
        dex
        bpl .ne0
        lda #0
        sta npos
        sta ntyped
        lda #<1500
        sta ntl
        lda #>1500
        sta nth
        jsr input_wait_release
.ne1:   jsr wait_frame
        jsr draw_name
        jsr input_char                  ; A = ASCII einer neuen Taste oder 0
        beq .ne9
        pha
        lda #<1500                      ; jede Eingabe stellt den Zaehler zurueck
        sta ntl
        lda #>1500
        sta nth
        pla
        cmp #$0d
        beq .nedone
        cmp #$14
        beq .nedel
        cmp #' '
        beq .neok
        cmp #'-'
        beq .neok
        cmp #'.'
        beq .neok
        cmp #'0'
        bcc .ne9
        cmp #'9' + 1
        bcc .neok
        cmp #'A'
        bcc .ne9
        cmp #'Z' + 1
        bcs .ne9
.neok:  pha                             ; Zeichen merken
        lda ntyped
        bne +
        ldx #7                          ; erste Eingabe: Vorgabe loeschen (Leerzeichen)
        lda #' '
.ne2:   sta hs_name,x
        dex
        bpl .ne2
        lda #1
        sta ntyped
        lda #0
        sta npos
+       pla
        ldx npos
        cpx #8
        bcs .ne9
        sta hs_name,x
        inc npos
        jmp .ne9
.nedel: lda ntyped
        bne +
        lda #1                          ; DEL vor jeder Eingabe: Vorgabe loeschen
        sta ntyped
        ldx #7
        lda #' '
.ne3:   sta hs_name,x
        dex
        bpl .ne3
        lda #0
        sta npos
        beq .ne9
+       lda npos
        beq .ne9
        dec npos
        ldx npos
        lda #' '
        sta hs_name,x
.ne9:   lda ntl                         ; 30 s ohne Eingabe: automatisch bestaetigen (16-Bit-Zaehler ntl/nth herunter, bei Unterlauf fertig)
        bne +
        dec nth
        bmi .nedone
+       dec ntl
        jmp .ne1
.nedone: rts
ne_default: .text "SPIELER "              ; Vorgabename (8 Zeichen, aufgefuellt)

; draw_name: den Namen (8 Zeichen aus hs_name, ASCII -> Bildschirmcode per AND $3F) in Zeile 14 ab Spalte 9 zeigen; die Cursorstelle (npos) gelb.
;   Veraendert A, X, Y, t5, txtcol, sp, cp.
draw_name:
        ldy #0
.dn1:   sty t5
        lda hs_name,y
        and #$3f
        pha
        lda #1
        cpy npos
        bne +
        lda #7                          ; Cursorstelle
+       sta txtcol
        lda t5
        clc
        adc #9
        tax
        ldy #14
        pla
        jsr put_cell
        ldy t5
        iny
        cpy #8
        bne .dn1
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Bibliotheken und Daten
; ---------------------------------------------------------------------------------------------------------------------
.include "lib/input.asm"              ; Joystick/Tastatur (VIA 1/2), Zeropage $8C-$8F
.include "lib/rand.asm"               ; Zufallszahlen (LFSR), Zeropage $02-$03
.include "lib/sound.asm"              ; Effekte und Titellied (VIC-I), Zeropage $04-$07; bindet <vic20/notes.inc> ein
.include "lib/hiscore.asm"            ; Bestenliste PRALL.HS (KERNAL), Zeropage $08-$0D, Puffer hs_buf
.include "lib/dirtable.inc"             ; dir_tab: Richtungstabelle (erzeugt von tools/gen.py)
.include "lib/gfx.inc"                  ; gfx_chars: Mehrfarbzeichen 64..66
.include "lib/logo.inc"                 ; logo_bits, logo_y: Titellogo
.include "lib/levels.inc"               ; level_count, level_data, level_kind: Levels
.include "lib/music.inc"                ; song, song_patterns: Titellied
