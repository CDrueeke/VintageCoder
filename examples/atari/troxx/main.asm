; =====================================================================================================================
; main.asm - TROXX fuer den Atari 8 Bit (XL/XE): Konstanten, Speicherplan, Start, Zustandsautomat
; =====================================================================================================================
; Herkunft: examples/vic20/troxx/main.asm (Referenzfassung, Stand Commit 4697d074) und examples/c64/troxx/main.asm (Stand Commit d173cb4a).  Diese Datei ist
;   Atari-spezifisch (Zustandsautomat mit zwei Spielern, Bestenliste, Player/Missile-Figuren); die gemeinsame Logik steht in grid.asm, enemies.asm, hud.asm
;   (siehe KOPIEN.txt).
;
; ZWECK
;   Troxx ist ein eigenstaendiges Arcade-Spiel: Eine Figur faehrt die Linien eines festen Gitters (8 x 5 Rechtecke) ab und faerbt sie; sind alle
;   vier Kanten eines Rechtecks abgefahren, wird es gefuellt.  Gegner laufen auf denselben Linien und kosten ein Leben.  Sind alle 40 Rechtecke
;   gefuellt, beginnt die naechste Stufe.  Regeln wie die Referenzfassung (VC-20); der Atari bringt mehr Leistung: Player/Missile-Grafik mit Pixelbewegung,
;   Zeichensatz mit vier Farben je Zeichen, Farbverlaeufe ueber Display-List-Interrupts, vierstimmiger POKEY-Klang und Zusatzfunktionen (Bonusstern, Eisblock,
;   Waechter, Bestenliste, zwei Spieler).
;
; BEDIENUNG
;   Titelbild: Joystick hoch/runter waehlt die Zeile (Spieler, Joystick, Tempo), links/rechts aendert, Feuer (oder die Taste START) startet.
;   Spiel: Joystick; die Figur laeuft nur, solange eine Richtung gehalten wird (an Knoten abbiegen, auf Linien nur vor und zurueck).  Feuer = Pause.
;   Punkte: 1 je neu gefaerbtes Zeichen, 10 je abgeschlossenes Segment, 100 je gefuelltes Rechteck (zwei Rechtecke in einem Zug: zusammen 400),
;   500 je Stufe, Bonusstern 50.  Extraleben je 10000 Punkte, hoechstens 9 Leben.
;   Test ohne Joystick: -D STARTLEVEL=$07 (Startstufe, BCD), -D AUTOPLAY=1 (der Rechner spielt selbst, das Titelbild entfaellt); -D AUTO_PLAYERS=2 (zwei Spieler), -D AUTOGOD=1
;   (unverwundbar), -D TESTDEMO=1 (Zaehler im RAM ab $8520 und hexadezimal in Zeile 23: Bilder, Spiele, hoechste Stufe).  Zusatzfunktionen abschalten:
;   -D EXTRA_STAR=0, EXTRA_ICE=0, EXTRA_GUARD=0, EXTRA_HISCORE=0, EXTRA_2P=0 (dann gelten genau die Regeln der Referenzfassung).
;
; ABLAUF (Zustandsautomat, ein Aufruf je Bild)
;   TITLE -> LEVELINIT -> PLAYING <-> FILL;  PLAYING -> HIT -> PLAYING / LEVELINIT (anderer Spieler) / GAMEOVER -> TITLE;
;   PLAYING/FILL -> LEVELCLEAR -> LEVELINIT.   Hauptschleife: tick (wait_frame, sprites.update, fx_fill, sound.update) -> Zustandsprozedur (STATES).
;
; WAS DER CODE AUF DER HARDWARE TUT
;   ANTIC: Anzeigelisten fuer Spiel- und Titelbild (antic.asm), Zeichensatz-RAM an CHARSET, Bildschirm SCR; DLI und Bildruecklauf-Interrupt (dli.asm) setzen
;   Farben je Textzeile und Farbbalken im Rand; Player/Missiles an PM_BASE (pmg.asm); POKEY (sound.asm); PIA und TRIG fuer die Joysticks (input.asm).
;   Das Betriebssystem bleibt eingeschaltet (Zeitgeber, Schatten, CIO fuer die Bestenliste); BASIC ist aus.
;
; SPEICHERPLAN
;   $0080-$00FF  Zeropage (Plan unten; $F8-$FF Zeiger des Klangs und des DLI)
;   $2000-$6FFF  Programm (Segment PROGRAM): Code, Tabellen, Grafik- und Figurenvorlagen
;   $7000-$73FF  Zeichensatz-RAM (128 Zeichen: 0..63 Kopie der ROM-Schrift fuer Modus 2, 64..127 eigene Grafik und Seitenschrift fuer Modus 4)
;   $7800-$7FFF  Player/Missile-Speicher (PMBASE $78, Einzelzeilen: Missiles +$300, Spieler 0..3 ab +$400)
;   $8000-$83BF  Bildschirm (40 x 24), $83C0-$83EF Lauftextpuffer, $8400 Anzeigelisten, $8480 Farbtabellen je Zeile
;   $8500-$86FF  Spielvariablen, Klang, Bestenliste;  $8700-$89B4 Spielfeldzustand GRIDST;  $8A00-$94D3 gesicherte Gitter der Spieler
;
; SPEICHERPLAN DER ZEROPAGE (wie die C64-Fassung, um $7E nach oben verschoben)
;   $80 STATE  $81 FLAGS  $82 FRAME  $83 STEPLEN  $84 RND  $85 CUR (aktiver Spieler 0/1)
;   $86-$B5 Spielerblock PBLK (48 Byte, wird beim Spielerwechsel gesichert): $86 SPEED  $87 LEVEL (BCD)  $88 LIVES  $89-$8B SCORE (BCD)
;           $8C Ueberlauf  $8D BOXES_DONE  $8E-$B5 BOXCNT (40 Byte)
;   $B6-$B8 HISCORE   $B9-$C0 PX PY PDIR PWANT PANIM PINV POX POY   $C1 FILLS_NOW  $C2 NENEMY  $C3-$DA EN (4 Gegner zu 6 Byte)
;   $DB-$E2 T0..T7   $E3-$E8 PTR_A PTR_B PTR_C   $E9-$F3 Gegner- und Hilfswerte   $F4-$F7 FILLQ   $F8-$F9 SPTR (Klang)   $FA-$FB DLI   $FC-$FD PMP
; =====================================================================================================================
; ---------------------------------------- English ----------------------------------------
; =====================================================================================================================
; main.asm - TROXX for the Atari 8 bit (XL/XE): constants, memory map, start, state machine
; =====================================================================================================================
; Origin: examples/vic20/troxx/main.asm (reference version, as of commit 4697d074) and examples/c64/troxx/main.asm (as of commit d173cb4a).  This file is
;   Atari-specific (state machine with two players, high score list, player/missile figures); the shared logic is in grid.asm, enemies.asm, hud.asm
;   (see KOPIEN.txt).
;
; PURPOSE
;   Troxx is a self-contained arcade game: a figure travels along the lines of a fixed grid (8 x 5 rectangles) and colors them; when all
;   four edges of a rectangle have been traveled, it is filled.  Enemies walk on the same lines and cost a life.  When all 40 rectangles are
;   filled, the next level begins.  Rules as in the reference version (VIC-20); the Atari brings more power: player/missile graphics with pixel movement,
;   character set with four colors per character, color gradients via display list interrupts, four-voice POKEY sound and extra features (bonus star, ice block,
;   Guard, high score list, two players).
;
; OPERATION
;   Title screen: joystick up/down selects the row (players, joystick, speed), left/right changes, fire (or the START key) starts.
;   Game: joystick; the figure runs only while a direction is held (turn at nodes, on lines only forward and back).  Fire = pause.
;   Points: 1 per newly colored character, 10 per completed segment, 100 per filled rectangle (two rectangles in one move: 400 together),
;   500 per level, bonus star 50.  Extra life every 10000 points, at most 9 lives.
;   Test without joystick: -D STARTLEVEL=$07 (start level, BCD), -D AUTOPLAY=1 (the computer plays by itself, no title screen); -D AUTO_PLAYERS=2 (two players), -D AUTOGOD=1
;   (invulnerable), -D TESTDEMO=1 (counters in RAM from $8520 and in hexadecimal in row 23: frames, games, highest level).  Switch off extra features:
;   -D EXTRA_STAR=0, EXTRA_ICE=0, EXTRA_GUARD=0, EXTRA_HISCORE=0, EXTRA_2P=0 (then exactly the rules of the reference version apply).
;
; FLOW (state machine, one call per frame)
;   TITLE -> LEVELINIT -> PLAYING <-> FILL;  PLAYING -> HIT -> PLAYING / LEVELINIT (other player) / GAMEOVER -> TITLE;
;   PLAYING/FILL -> LEVELCLEAR -> LEVELINIT.   Main loop: tick (wait_frame, sprites.update, fx_fill, sound.update) -> state procedure (STATES).
;
; WHAT THE CODE DOES ON THE HARDWARE
;   ANTIC: display lists for game and title screen (antic.asm), character set RAM at CHARSET, screen SCR; DLI and vertical blank interrupt (dli.asm) set
;   colors per text row and color bars in the border; player/missiles at PM_BASE (pmg.asm); POKEY (sound.asm); PIA and TRIG for the joysticks (input.asm).
;   The operating system stays switched on (timer, shadows, CIO for the high score list); BASIC is off.
;
; MEMORY MAP
;   $0080-$00FF  Zero page (map below; $F8-$FF pointers of the sound and the DLI)
;   $2000-$6FFF  Program (segment PROGRAM): code, tables, graphics and figure templates
;   $7000-$73FF  Character set RAM (128 characters: 0..63 copy of the ROM font for mode 2, 64..127 own graphics and side lettering for mode 4)
;   $7800-$7FFF  Player/missile memory (PMBASE $78, single lines: missiles +$300, players 0..3 from +$400)
;   $8000-$83BF  Screen (40 x 24), $83C0-$83EF scrolling text buffer, $8400 display lists, $8480 color tables per row
;   $8500-$86FF  Game variables, sound, high score list;  $8700-$89B4 field state GRIDST;  $8A00-$94D3 saved grids of the players
;
; ZERO PAGE MAP (like the C64 version, shifted up by $7E)
;   $80 STATE  $81 FLAGS  $82 FRAME  $83 STEPLEN  $84 RND  $85 CUR (active player 0/1)
;   $86-$B5 player block PBLK (48 bytes, saved when the player changes): $86 SPEED  $87 LEVEL (BCD)  $88 LIVES  $89-$8B SCORE (BCD)
;           $8C overflow  $8D BOXES_DONE  $8E-$B5 BOXCNT (40 bytes)
;   $B6-$B8 HISCORE   $B9-$C0 PX PY PDIR PWANT PANIM PINV POX POY   $C1 FILLS_NOW  $C2 NENEMY  $C3-$DA EN (4 enemies of 6 bytes)
;   $DB-$E2 T0..T7   $E3-$E8 PTR_A PTR_B PTR_C   $E9-$F3 enemy and helper values   $F4-$F7 FILLQ   $F8-$F9 SPTR (sound)   $FA-$FB DLI   $FC-$FD PMP

#include "hw.inc"

; ---- Zusatzfunktionen (je abschaltbar; 0 = aus, dann gelten genau die Regeln der Referenzfassung)
.ifndef EXTRA_STAR
.const EXTRA_STAR = 1                   ; Bonusstern auf einer Linie (Zusatzpunkte, nach zehn Sekunden weg)
.endif
.ifndef EXTRA_ICE
.const EXTRA_ICE = 1                    ; Eisblock: Gegner stehen kurz still
.endif
.ifndef EXTRA_GUARD
.const EXTRA_GUARD = 1                  ; dritter Gegnertyp "Waechter" mit festem Rundgang
.endif
.ifndef EXTRA_HISCORE
.const EXTRA_HISCORE = 1                ; Bestenliste (zehn Eintraege) auf Diskette
.endif
.ifndef EXTRA_2P
.const EXTRA_2P = 1                     ; zwei Spieler abwechselnd, Joystickwahl im Titelbild
.endif
; ---- Spielregel (Konzept 16.2): 1 = Gegner zerstoeren nicht geschlossene Linien (ein Segment, das an kein gefuelltes Rechteck grenzt), 0 = alte Regel
.ifndef ENEMY_DESTROYS_LINES
.const ENEMY_DESTROYS_LINES = 1
.endif
.ifndef FX_DLI
.const FX_DLI = 1                       ; Display-List-Interrupts: Farbverlaeufe je Zeile (0: eine Farbe je Bild; dann ist das Spielfeld einfarbig)
.endif
.ifndef FX_TITLE
.const FX_TITLE = 1                     ; Titelbild mit grossem Logo (Farbzyklus) und weichem Bildlauf
.endif
.ifndef FX_FLASH
.const FX_FLASH = 1                     ; Fuellanimation: die neueste Zeile blitzt weiss/gelb auf
.endif
.ifndef FX_VIB
.const FX_VIB = 1                       ; Titelmusik: Vibrato der Melodie
.endif
.ifndef AUTO_PLAYERS
.const AUTO_PLAYERS = 1
.endif
; ---- Player/Missile-Einstellungen (Konzept 15.3): Breite je Spieler (SIZEPn: 0 normal = 8 Farbtakte, 1 doppelt, 3 vierfach), Breite der Missiles (2 Bit je Missile,
;      0 normal = 2 Farbtakte), Prioritaet PRIOR (Bit 0..3: Spieler vor/hinter dem Spielfeld, Bit 4: Missiles als fuenfter Spieler mit COLPF3, Bit 5: Mehrfarben-Spieler)
.ifndef PM_PRIOR
.const PM_PRIOR = $11                   ; Spieler und Missiles vor dem Spielfeld; Missiles als fuenfter Spieler (Gegner 4 in der Farbe COLPF3)
.endif
.ifndef PM_WIDTH_P
.const PM_WIDTH_P = 0                   ; SIZEP0..3 (die Figuren sind nur sechs Pixel breit; grosse Werte verdoppeln sie)
.endif
.ifndef PM_WIDTH_M
.const PM_WIDTH_M = 0                   ; SIZEM
.endif
.const EXTRA_ITEMS = EXTRA_STAR | EXTRA_ICE

; ---- Gitter (Abschnitt 2.1 des Konzepts, Fassung Atari): 8 x 5 Rechtecke, Innenraum 3 x 3 Zeichen
.const BOX_COLS  = 8
.const BOX_ROWS  = 5
.const BOX_W     = 3
.const BOX_H     = 3
.const GRID_W    = BOX_COLS * (BOX_W + 1) + 1   ; 33 Spalten (gx 0..32); senkrechte Linien bei gx = 0, 4, 8, ... 32
.const GRID_H    = BOX_ROWS * (BOX_H + 1) + 1   ; 21 Zeilen (gy 0..20); waagerechte Linien bei gy = 0, 4, 8, 12, 16, 20
.const NBOX      = BOX_COLS * BOX_ROWS          ; 40 Rechtecke
.const SCR_COLS  = 40
.const SCR_ROWS  = 24
.const GRID_ROW0 = 1                            ; Bildschirmzeile von gy = 0 (Zeilen 1..21); Kopfzeile 0, Meldungen in Zeile 23
.const GRID_COL0 = 1                            ; Bildschirmspalte von gx = 0 (Spalten 1..33; Spalte 0 bleibt frei)
.const SIDE_COL  = 34                           ; Seitenleiste: Spalten 34..39

; ---- Speicherplan (siehe Kopf)
.const CHARSET   = $7000                        ; Zeichensatz-RAM, 1-KB-Grenze
.const PM_BASE   = $7800                        ; Player/Missile-Speicher, 2-KB-Grenze (Einzelzeilen)
.const PM_M      = PM_BASE + $300               ; Missiles
.const PM_P0     = PM_BASE + $400               ; Spieler 0; Spieler 1..3 im Abstand $100
.const SCR       = $8000                        ; Bildschirmspeicher (960 Byte, in einem 4-KB-Block)
.const SCRL      = $83c0                        ; Lauftextpuffer (48 Zeichen)
.const RC_BK     = $8480                        ; Farbtabellen je Textzeile (24 Byte): COLBK, COLPF1, COLPF2, HSCROL
.const RC_P1     = $84a0
.const RC_P2     = $84c0
.const RC_H      = $84e0
.const BARRAM    = $9600                        ; Farbbalken-Tabelle (128 Byte, aus BAR je Norm angepasst)
.const GRIDST    = $8700                        ; Spielfeldzustand 33 x 21 Byte

; ---- Zeichencodes (Modus 2: Schrift Codes 0..63 wie das ROM; Modus 4: Grafik und Seitenschrift, siehe gfx.inc)
.const CH_SPACE   = 0
.const DIG2       = 16                          ; Ziffer 0 im Modus 2
.const DIG4       = 80                          ; Ziffer 0 der Seitenschrift im Modus 4 (TXT_BASE + 16)

; ---- Zustandscodes der Linien (GRIDST): beliebige Werte unter 8 (grid.color_at maskiert mit 7)
.const COL_LINE   = 4                           ; unberuehrte Linie
.const COL_DONE   = 7                           ; abgefahrene Linie (und abgefahrener Knoten)
.const COL_FILL   = 5                           ; Zelle im Innern eines gefuellten Rechtecks

; ---- Farben (Atari-Farbwerte: Farbton * 16 + Helligkeit)
.const COL_PLAYER = $0e                         ; Figur: weiss
.const COL_PLAYER2 = $9e                        ; Figur beim Blinken (Unverwundbarkeit): hellblau
.const COL_HIT    = $3a                         ; Figur beim Treffer: rot
.const COL_RUNNER = $3a                         ; Laeufer: rot
.const COL_WANDER = $2a                         ; Streuner: orange
.const COL_GUARD  = $9a                         ; Waechter: tuerkis
.const COL_ICE    = $96                         ; Gegner im Eisblock-Effekt: hellblau
.const COL_LINE_PF0 = $86                       ; unberuehrte Linie (COLPF0), PAL
.const COL_LINE_PF0_N = $96                     ; dieselbe Farbe im NTSC-Farbkreis
.const COL_DONE_LINE = $1e                      ; abgefahrene Linie (COLPF1): Gold

; ---- Zelltypen (grid.cell_type)
.const CT_NONE = 0
.const CT_H    = 1                      ; Innenzelle eines waagerechten Segments
.const CT_V    = 2                      ; Innenzelle eines senkrechten Segments
.const CT_NODE = 3                      ; Knoten

; ---- Richtungen (Reihenfolge wichtig: Umkehr = Richtung xor 1) und Eingabemaske (Bit = Richtung, wie BITMASK)
.enum Dir
    up, down, left, right
.endenum
.const IN_UP    = $01
.const IN_DOWN  = $02
.const IN_LEFT  = $04
.const IN_RIGHT = $08
.const IN_FIRE  = $10

; ---- FLAGS
.const F_PAL    = $01                   ; Bit 0: PAL erkannt (0 = NTSC)
.const SM_GAME  = 0                     ; Bildschirmart (SCRMODE)
.const SM_TITLE = 1

; ---- Spieleinstellungen
.ifndef LIVES0
.const LIVES0      = 3
.endif
.const START_SPEED = 5
.const INVULN      = 12                 ; Spielschritte Unverwundbarkeit nach einem Treffer
.const KIND_RUNNER = 0
.const KIND_WANDER = 1
.const KIND_GUARD  = 2

; ---- Zeropage (siehe Plan oben)
.const ZB       = $7e                   ; Verschiebung gegenueber der C64-Fassung
.const STATE    = $02 + ZB
.const FLAGS    = $03 + ZB
.const FRAME    = $04 + ZB              ; Bilder seit dem letzten Spielschritt
.const STEPLEN  = $05 + ZB              ; Bilder je Spielschritt (aus der Tempotabelle)
.const RND      = $06 + ZB              ; Zustand des Zufallsgenerators (nie 0)
.const CUR      = $07 + ZB              ; aktiver Spieler (0 oder 1)
.const PBLK     = $08 + ZB              ; Spielerblock: wird beim Spielerwechsel als Ganzes gesichert und geladen
.const SPEED    = $08 + ZB              ; Tempo 1..9
.const LEVEL    = $09 + ZB              ; Stufe, BCD
.const LIVES    = $0a + ZB
.const SCORE    = $0b + ZB              ; Punkte, BCD, 3 Byte (niedrigstes zuerst)
.const SCORE_OVF = $0e + ZB             ; Ueberlauf des Punktestands (nur Rechenhilfe)
.const BOXES_DONE = $0f + ZB            ; gefuellte Rechtecke
.const BOXCNT   = $10 + ZB              ; je Rechteck abgeschlossene Kanten (40 Byte)
.const PBLK_LEN = 3 + 4 + 1 + NBOX      ; SPEED, LEVEL, LIVES, SCORE (3), Ueberlauf, BOXES_DONE, BOXCNT
.const HISCORE  = $38 + ZB              ; beste Punkte (Bestenliste oder Sitzung), BCD, 3 Byte
.const PX       = $3b + ZB              ; Figur: Gitterzelle (gx, gy)
.const PY       = $3c + ZB
.const PDIR     = $3d + ZB              ; letzte Richtung (Dir)
.const PWANT    = $3e + ZB              ; gehaltene Richtungen (Maske IN_*)
.const PANIM    = $3f + ZB              ; Bild 0 oder 1
.const PINV     = $40 + ZB              ; Rest der Unverwundbarkeit in Spielschritten
.const POX      = $41 + ZB              ; Zelle vor dem letzten Schritt (die Figur laeuft von dort zur Zelle PX, PY)
.const POY      = $42 + ZB
.const FILLS_NOW  = $43 + ZB            ; in diesem Spielschritt abgeschlossene Rechtecke (0..2)
.const NENEMY   = $44 + ZB
.const EN       = $45 + ZB              ; Gegner: 4 * 6 Byte
.const E_X      = 0                     ; Feldabstaende je Gegner
.const E_Y      = 1
.const E_DIR    = 2
.const E_KIND   = 3                     ; 0 Laeufer, 1 Streuner, 2 Waechter
.const E_OX     = 4                     ; Zelle vor dem letzten Schritt (fuer die Bewegung)
.const E_OY     = 5
.const E_SIZE   = 6
.const T0       = $5d + ZB              ; Zwischenwerte T0..T7
.const T1       = $5e + ZB
.const T2       = $5f + ZB
.const T3       = $60 + ZB
.const T4       = $61 + ZB
.const T5       = $62 + ZB
.const T6       = $63 + ZB
.const T7       = $64 + ZB
.const PTR_A    = $65 + ZB              ; Zeiger Bildschirm
.const PTR_B    = $67 + ZB              ; Zeiger Spielfeldzustand
.const PTR_C    = $69 + ZB              ; Zeiger Puffer (Gitter sichern und laden, Plattenzugriff)
.const HITF     = $6c + ZB              ; 1 = Zusammenstoss in diesem Spielschritt
.const EPH      = $6d + ZB              ; Bild der Gegner
.const ECUR     = $6e + ZB              ; laufender Gegner (Abstand in EN)
.const ELIM     = $6f + ZB              ; Ende der Gegnerliste (Gegnerzahl * E_SIZE)
.const EBEST    = $70 + ZB
.const EDIR     = $71 + ZB
.const EDIST    = $72 + ZB
.const EQX      = $73 + ZB
.const EQY      = $74 + ZB
.const ATMP     = $75 + ZB
.const FILLQ    = $76 + ZB              ; Fuellwarteschlange: +0, +1 Rechtecke, +2 Zeile, +3 Bildzaehler
.const DLT_BK   = $fa                   ; DLI: Farben der naechsten Zeile (COLBK, COLPF1, COLPF2)
.const DLT_P1   = $fb
.const DLT_P2   = $fe
.const PMP      = $fc                   ; Zeiger in den Player/Missile-Speicher (2 Byte)
.assert BOXCNT + NBOX == PBLK + PBLK_LEN, "Spielerblock lueckenlos"
.assert PBLK + PBLK_LEN <= HISCORE, "Zeropage-Plan"
.assert EN + 4 * E_SIZE <= T0, "Zeropage-Plan"

; ---- Spielvariablen im RAM (ab $8500, ausserhalb des Programms)
.const VB        = $8500
.const JOYMODE   = VB + $00             ; 0 ein Joystick Anschluss 1, 1 ein Joystick Anschluss 2, 2 zwei Joysticks
.const NPLAYERS  = VB + $01             ; 1 oder 2
.const SELROW    = VB + $02             ; Titelbild: gewaehlte Zeile
.const RESTORE   = VB + $03             ; 1 = LEVELINIT stellt das gesicherte Gitter wieder her
.const PFRESH    = VB + $04             ; 2 Byte: 1 = Spieler hat noch kein Gitter gespielt
.const SPPROG    = VB + $06             ; Fortschritt des Schritts in Punkten 0..8
.const SPHID     = VB + $07             ; 1 = Player/Missiles verborgen
.const SPHIT     = VB + $08             ; 1 = Figur als Treffer-Bild zeigen
.const SPT0      = VB + $09             ; Rechenhilfen von sprites.update: SPT0..SPT7 (VB + $09 .. VB + $10)
.const SPT1      = VB + $0a
.const SPT2      = VB + $0b
.const SPT3      = VB + $0c
.const SPT4      = VB + $0d
.const SPT5      = VB + $0e
.const SPT6      = VB + $0f
.const SPT7      = VB + $10
.const IT_KIND   = VB + $11             ; Gegenstand auf dem Feld: 0 keiner, 1 Stern, 2 Eisblock
.const IT_X      = VB + $12
.const IT_Y      = VB + $13
.const IT_TIME   = VB + $14             ; 2 Byte: Rest der Sichtzeit in Bildern
.const IT_WAIT   = VB + $16             ; 2 Byte: Wartezeit bis zum naechsten Gegenstand
.const FREEZE    = VB + $18             ; 2 Byte: Rest der Gegner-Starre in Bildern
.const FPS       = VB + $1a             ; Bilder je Sekunde (50 oder 60)
.const FSI       = VB + $1b             ; Schleifenzaehler von final_scores
.const SPN       = VB + $1c             ; laufender Platz in sprites.update
.const SPOFF     = VB + $1d             ; 1 = Figuren verbergen (Namenseingabe)
.const FXC       = VB + $1e             ; Zaehler fuer Farbzyklus (Titelbild)
.const DLYCNT    = VB + $1f
.const TD_BASE   = VB + $20             ; TESTDEMO: +0,+1 Bilder, +2 Spiele, +3 hoechste Stufe, +4 Treffer, +5 Rechtecke gesamt
.const SCR_IDX   = VB + $30             ; Testbetrieb SCRIPT: Eintrag, Rest der Dauer, Masken der Anschluesse 1 und 2
.const SCR_REM   = VB + $31
.const SCR_M1    = VB + $32
.const SCR_M2    = VB + $33
.const SND       = VB + $40             ; Klangmodul (siehe sound.asm), 64 Byte
.const HS_BUF    = VB + $80             ; Bestenliste (siehe hiscore.asm), 96 Byte
.const HS_TMP    = VB + $100            ; Ladepuffer der Bestenliste, 80 Byte
.const HS_ST     = VB + $150            ; Ergebnis des letzten Plattenzugriffs (DK_*)
.const HS_SC     = VB + $151            ; Endstand (3 Byte Punkte, 1 Byte Stufe) fuer die Bestenliste
.const HS_NAME   = VB + $155            ; Name (3 Zeichen)
.const HS_S      = VB + $158            ; Offset des neuen Eintrags
.const HS_PL     = VB + $159            ; Spielernummer
.const HS_CPOS   = VB + $15a            ; Eingabeposition
.const HS_I      = VB + $15b
.const SIDX      = VB + $15c            ; Lauftext: Leseposition
.const SSUB      = VB + $15d
.const FXK       = VB + $15e            ; Hilfswerte der Fuellanimation
.const FXR       = VB + $15f
.const FXG       = VB + $160
.const FXQ       = VB + $161
.const SCRMODE   = VB + $162            ; Bildschirmart (SM_*)
.const SCRROW    = VB + $163            ; Zeile mit Feinscrollen im DLI (255 = keine)
.const dli_i     = VB + $164            ; Zeilenzaehler des DLI
.const DLT_H     = VB + $167            ; DLI: HSCROL der naechsten Zeile
.const BPHASE    = VB + $165            ; Phase der Farbbalken
.const PF3COL    = VB + $166            ; COLPF3 (Laeuferfarbe, bei Starre hellblau)
.const OLDY      = VB + $168            ; alte Zeile je Platz 0..4 (Player/Missile-Speicher)
.const SPX       = VB + $170
.const SPY       = VB + $171
.const SDX       = VB + $172            ; Segment-Durchlauf (grid.seg_first / seg_next / seg_destroy): laufende Zelle, Rest, gefaerbte Zeichen
.const SDY       = VB + $173
.const SDN       = VB + $174
.const SDP       = VB + $175
.const NB0       = VB + $176            ; angrenzende Rechtecke eines Segments ($FF = keins): unten/rechts
.const NB1       = VB + $177            ;                                                    oben/links
.const NBL       = VB + $178            ; Grenzwert der Koordinate in grid.nb_boxes
.const FLT       = VB + $179            ; Aufblitzen zerstoerter Linien: Rest in Bildern (0 = aus)
.const FLN       = VB + $17a            ; Zahl der vorgemerkten Segmente (bis 4); je Segment FLX, FLY, FLTY (Warteschlange ab VB + $1E0)
.const NODEX     = VB + $17b            ; Knoten-Durchlauf (grid.nodes_free)
.const NODEY     = VB + $17c
.const SDM       = VB + $17d            ; Hilfswert der Knotenpruefung (dx)
.const FLX       = VB + $1e0
.const FLY       = VB + $1e4
.const FLTY      = VB + $1e8
.const PSAVE     = VB + $180            ; gesicherter Spielerblock je Spieler (2 * PBLK_LEN)
.const GSAVE     = $8a00                ; gesichertes Gitter je Spieler (2 * 2 * GRID_W * GRID_H)
.const GSAVE_LEN = 2 * GRID_W * GRID_H
.assert GSAVE + 2 * GSAVE_LEN <= $9800, "gesicherte Gitter im RAM-Block $8A00-$97FF"
.assert PSAVE + 2 * PBLK_LEN <= VB + $1e0, "Spielvariablen im Block $8500-$86FF"

.enum St
    title, levelinit, playing, fill, hit, levelclear, gameover
.endenum

; ---------------------------------------------------------------------------------------------------------------------
; Programm
; ---------------------------------------------------------------------------------------------------------------------
.target atari8
.cpu sally { illegals = deny }
.org $2000

; start: Einstieg (XEX, Startadresse).  Das Betriebssystem bleibt eingeschaltet; der Bildschirm bleibt aus, bis antic.init fertig ist.
start:  cld
        ldx #$ff
        txs
        lda #0
        tax
.zv:    sta VB,x                        ; Spielvariablen loeschen
        sta VB + $100,x
        inx
        bne .zv
        sta FLAGS
        sta HISCORE
        sta HISCORE + 1
        sta HISCORE + 2
        sta JOYMODE                     ; Standard: ein Spieler, ein Joystick an Anschluss 2
        sta SELROW
        sta RESTORE
        lda #1
        sta NPLAYERS
        lda #START_SPEED
        sta SPEED
        jsr antic.init                  ; Norm erkennen, Zeichensatz, Player/Missiles, Interrupts, Bildschirm einrichten
        jsr setfps
        jsr antic.clear
        jsr sound.init
        jsr input.setup
.if EXTRA_HISCORE
        jsr disk.init                   ; Laufwerk der Bestenliste = Ladelaufwerk
        jsr hiscore.load                ; Bestenliste von Diskette (fehlt das Laufwerk oder DOS, bleibt die Standardliste)
.endif
.ifdef SCRIPT
        jsr input.script_step           ; Testbetrieb: erster Skripteintrag
.endif
        lda #St.title
        sta STATE
.loop:  jsr tick
        jsr dispatch
.ifdef TESTDEMO
        jsr testdemo
.endif
        jmp .loop

; setfps: Bilder je Sekunde (FPS) aus der Norm.  Veraendert A.
setfps: lda FLAGS
        and #F_PAL
        beq .n
        lda #50
        bne .s
.n:     lda #60
.s:     sta FPS
        rts

; tick: ein Bild: auf den Bildanfang warten, Figuren setzen, Klang weiterschalten.  Veraendert A, X, Y.
tick:   jsr antic.wait_frame
        jsr sprites.update
.if ENEMY_DESTROYS_LINES
.ifndef FLASH_HOLD                      ; (Testbetrieb FLASH_HOLD=1: das Aufblitzen bleibt stehen, fuer ein Bildschirmfoto)
        lda FLT                         ; zerstoerte Linie: Aufblitzen nach zwei Bildern beenden
        beq .nfl
        dec FLT
        bne .nfl
        jsr grid.flash_end
.nfl:
.endif
.endif
.if FX_TITLE
        lda STATE
        cmp #St.title
        bne .nt
        jsr title_fx                    ; Titelbild: Farbzyklus und Lauftext
.nt:
.endif
.if FX_FLASH
        jsr fx_fill
.endif
.ifdef SCRIPT
        jsr input.script_step
.endif
        jmp sound.update

; dispatch: Zustandsprozedur STATE aufrufen (Sprungtabelle mit Adresse - 1, Sprung ueber RTS).  Veraendert A, X.
dispatch:
        lda STATE
        asl
        tax
        lda STATES + 1,x
        pha
        lda STATES,x
        pha
        rts

STATES: .retaddr st_title, st_levelinit, st_playing, st_fill, st_hit, st_levelclear, st_gameover

; set_tempo: STEPLEN aus der Tempotabelle (Tempo SPEED, je Norm eine Reihe zu 9 Werten).  Veraendert A, X.
set_tempo:
        ldx SPEED
        dex
        lda FLAGS
        and #F_PAL
        bne .pal
        txa                             ; NTSC: zweite Reihe
        clc
        adc #9
        tax
.pal:   lda TEMPO,x
        sta STEPLEN
        rts

; delay: X Bilder warten (Klang und Sprites laufen weiter).  Veraendert A, X, Y.
delay:  stx DLYCNT
.d:     jsr tick
        dec DLYCNT
        bne .d
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Zustaende
; ---------------------------------------------------------------------------------------------------------------------

; fire_wait: wartet, bis der Zustand der Feuertaste X entspricht (0 = losgelassen, IN_FIRE = gedrueckt); gilt der Joystick des aktiven
;   Spielers.  Veraendert A, Y, T7.
fire_wait:
        stx T7
.w:     jsr tick
        jsr input.read_joy
        and #IN_FIRE
        cmp T7
        bne .w
        rts

; fire_wait_any: wie fire_wait, mit beiden Anschluessen (Titelbild und Zwischenbilder).  Veraendert A, Y, T7.
fire_wait_any:
        stx T7
.w:     jsr tick
        jsr input.read_any
        and #IN_FIRE
        cmp T7
        bne .w
        rts

; TITLE: Titelbild mit Spielerzahl, Joystickwahl und Tempo, Bestenliste, Titelmusik; Feuer startet ein neues Spiel
st_title:
        jsr antic.use_title
        jsr antic.clear
        jsr sprites.hide
        jsr title_draw
.ifndef AUTOPLAY
        jsr sound.music_start
.sel:   jsr title_values
        jsr tick
        jsr input.read_any
        sta T7
        and #IN_FIRE
        bne .go
        lda T7
        and #IN_UP
        beq .nu
        lda SELROW
        beq .rep
        dec SELROW
        bpl .rep
.nu:    lda T7
        and #IN_DOWN
        beq .nd
        lda SELROW
        cmp #2
        bcs .rep
        inc SELROW
        bne .rep
.nd:    lda T7
        and #IN_LEFT
        beq .nl
        lda #$ff
        jsr title_change
        jmp .rep
.nl:    lda T7
        and #IN_RIGHT
        beq .sel
        lda #1
        jsr title_change
.rep:   ldx #10                         ; Wiederholung bremsen
        jsr delay
        jmp .sel
.go:    jsr sound.music_stop
        ldx #0
        jsr fire_wait_any               ; Feuer loslassen, sonst gilt es gleich als Pause
.else
        lda #AUTO_PLAYERS
        sta NPLAYERS
.endif
        jsr new_game
        lda #St.levelinit
        sta STATE
        rts

; new_game: Spielerbloecke fuer ein neues Spiel setzen (beide Spieler: 3 Leben, Stufe 1, 0 Punkte, Tempo wie gewaehlt).  Veraendert A, X, Y, PTR_C.
new_game:
        lda #0
        sta CUR
        sta RESTORE
        lda #1
        sta PFRESH
        sta PFRESH + 1
        ldy #PBLK_LEN - 1
        lda #0
.z:     sta PBLK,y                      ; Block leeren (Punkte, Rechtecke)
        dey
        bpl .z
        lda #LIVES0
        sta LIVES
        lda #1
        sta LEVEL
.ifdef STARTLEVEL
        lda #STARTLEVEL                 ; Testbetrieb: Startstufe (BCD), z. B. $07 fuer vier Gegner
        sta LEVEL
.endif
.ifdef TESTSCORE
        lda #TESTSCORE                  ; Testbetrieb: Startpunkte (BCD, Tausender/Hunderter), damit die Bestenliste erreichbar ist
        sta SCORE + 1
.endif
        lda title_speed
        sta SPEED
        ldy #PBLK_LEN - 1               ; Block auch fuer den zweiten Spieler
.c:     lda PBLK,y
        sta PSAVE + PBLK_LEN,y
        dey
        bpl .c
        rts

; title_speed: gewaehltes Tempo (wird beim Spielstart nach SPEED kopiert)
title_speed: .byte START_SPEED

; title_change: Wert der gewaehlten Zeile aendern.  A = +1 oder -1 ($FF).  Veraendert A, X.
title_change:
        sta T6
        ldx SELROW
        bne .notp
        lda NPLAYERS                    ; Zeile 0: Spielerzahl 1 oder 2 (nur mit EXTRA_2P)
.if EXTRA_2P
        eor #3                          ; 1 <-> 2
        sta NPLAYERS
        cmp #2
        beq .out
        lda JOYMODE
        cmp #2
        bcc .out
        lda #0                          ; mit einem Spieler gibt es keine zwei Joysticks
        sta JOYMODE
.endif
.out:   rts
.notp:  dex
        bne .speed
        lda JOYMODE                     ; Zeile 1: Joystick 0 = Anschluss 2, 1 = Anschluss 1, 2 = zwei Joysticks
        clc
        adc T6
        bmi .out
        ldx NPLAYERS
        cpx #2
        beq .m2
        cmp #2
        bcs .out
        bcc .setj
.m2:    cmp #3
        bcs .out
.setj:  sta JOYMODE
        rts
.speed: lda title_speed                 ; Zeile 2: Tempo 1..9
        clc
        adc T6
        beq .out
        cmp #10
        bcs .out
        sta title_speed
        rts

; LEVELINIT: Gitter und Farben neu (oder das gesicherte Gitter), Figuren auf die Startknoten, kurzer Hinweis, dann Spiel
st_levelinit:
        lda #0
        sta SPOFF
        sta FLT                         ; kein Aufblitzen ueber den Bildaufbau hinaus
        sta FLN
        jsr antic.use_game
        jsr antic.clear
        jsr hud.draw
        lda RESTORE
        bne .restore
        jsr grid.draw
        lda #0
        ldx #NBOX - 1
.clr:   sta BOXCNT,x                    ; BOXCNT leeren
        dex
        bpl .clr
        sta BOXES_DONE
        jmp .go
.restore:
.if EXTRA_2P
        jsr gbuf_ptr
        jsr grid.load                   ; Gitter des Spielers wiederherstellen (BOXCNT und BOXES_DONE stehen schon im Spielerblock)
.endif
.go:    lda #0
        sta FILLS_NOW
        sta HITF
        sta RESTORE
        ldx CUR
        sta PFRESH,x
        lda LEVEL
        ora #$80
        sta RND                         ; fester Startwert je Stufe (nie 0)
        jsr set_tempo
        jsr player.init
        jsr enemies.place_all
.if EXTRA_ITEMS
        jsr items.reset
.endif
        jsr hud.update
        ldx #HT_READY
        lda NPLAYERS
        cmp #2
        bne .msg
        ldx #HT_PLAYER                  ; "player n ready": Ziffer eintragen (bei einem Joystick wartet das Spiel danach auf Feuer)
.msg:   jsr hud.msg
        lda NPLAYERS
        cmp #2
        bne .nopl
        lda CUR
        clc
        adc #DIG2 + 1
        sta SCR + MSG_ROW * SCR_COLS + 11
.nopl:
        ldx #75
        jsr delay
.ifndef AUTOPLAY
        lda NPLAYERS
        cmp #2
        bne .start
        lda JOYMODE
        cmp #2
        beq .start
        ldx #HT_FIRE                    ; ein Joystick: den Joystick weiterreichen, Feuer startet
        jsr hud.msg
        ldx #0
        jsr fire_wait_any
        ldx #IN_FIRE
        jsr fire_wait_any
        ldx #0
        jsr fire_wait_any
.start:
.endif
        jsr hud.draw
        lda #0
        sta FRAME
        lda #St.playing
        sta STATE
        rts

; PLAYING: alle STEPLEN Bilder ein Spielschritt
st_playing:
.ifndef AUTOPLAY
        jsr input.read_joy
        sta PWANT
        and #IN_FIRE
        beq .run
        jsr sound.silence               ; Pause: Feuer, nochmal Feuer = weiter
        ldx #HT_PAUSE
        jsr hud.msg
        ldx #0
        jsr fire_wait
        ldx #IN_FIRE
        jsr fire_wait
        ldx #0
        jsr fire_wait
        jsr hud.draw
        jmp st_playing
.run:
.endif
        jsr hud.update
.if EXTRA_ITEMS
        jsr items.tick
.endif
        inc FRAME
        lda FRAME
        cmp STEPLEN
        bcc .out
        lda #0
        sta FRAME
        jsr player.step
        lda HITF
        bne .after                      ; Treffer: Gegner nicht mehr bewegen
        jsr enemies.step
.after: lda FILLS_NOW
        beq .nofill
        lda #St.fill                    ; ein Treffer im selben Schritt wartet, bis das Fuellen fertig ist
        sta STATE
        lda #0
        sta FILLQ + 2
        sta FILLQ + 3
        rts
.nofill:
        lda HITF
        beq .out
        lda #St.hit
        sta STATE
.out:   rts

; FILL: vorgemerkte Rechtecke fuellen, je 3 Bilder eine Zeile (beide Rechtecke zugleich)
st_fill:
        jsr hud.update
        inc FILLQ + 3
        lda FILLQ + 3
        cmp #3
        bcc .out
        lda #0
        sta FILLQ + 3
        tax                             ; X = Nummer in der Warteschlange
.box:   stx ATMP
        lda FILLQ,x
        ldx FILLQ + 2
        jsr grid.fill_row_b
        ldx ATMP
        inx
        cpx FILLS_NOW
        bcc .box
        inc FILLQ + 2
        lda FILLQ + 2
        cmp #BOX_H
        bcc .out
        lda #0
        sta FILLS_NOW
        lda HITF
        bne .tohit
        lda #St.playing
        ldx BOXES_DONE
        cpx #NBOX
        bne .chg
        lda #St.levelclear
        bne .chg
.tohit: lda #St.hit
.chg:   sta STATE
.out:   rts

; HIT: Treffer anzeigen, Leben - 1, Figuren zurueck auf die Startknoten (Farben und Rechtecke bleiben); bei zwei Spielern wechselt der Spieler
st_hit:
        lda #0
        sta HITF
        lda #1
        sta SPHIT                       ; Spieler als Treffer-Bild
        ldx #HT_HIT
        jsr hud.msg
        lda #Fx.hit
        jsr sound.play
        ldx #75
        jsr delay
        lda #0
        sta SPHIT
.ifdef TESTDEMO
        inc TD_BASE + 4
.endif
        dec LIVES
        jsr player.init
        lda #INVULN
        sta PINV
        jsr enemies.place_all
.if EXTRA_ITEMS
        jsr items.reset
.endif
        jsr hud.draw
        lda #0
        sta FRAME
        lda LIVES
        beq .dead
        ldx BOXES_DONE
        cpx #NBOX
        bne .turn
        lda #St.levelclear              ; die Stufe war im selben Zug geschafft
        bne .chg
.turn:
.if EXTRA_2P
        jsr switch_player               ; zwei Spieler: nach jedem Treffer ist der andere dran (C = 1: gewechselt, STATE gesetzt)
        bcs .done
.endif
        lda #St.playing
        bne .chg
.dead:
.if EXTRA_2P
        jsr switch_player
        bcs .done
.endif
        lda #St.gameover
.chg:   sta STATE
.done:  rts

.if EXTRA_2P
; switch_player: zum anderen Spieler wechseln, wenn zwei Spieler spielen und dieser noch Leben hat.  Sichert Block und Gitter des bisherigen
;   Spielers, laedt den Block des anderen; setzt STATE = LEVELINIT und RESTORE.  C = 1: gewechselt.  Veraendert A, X, Y, PTR_A..PTR_C, T0, T1.
switch_player:
        lda NPLAYERS
        cmp #2
        bne .no
        lda CUR
        eor #1
        tax
        ldy PSAVE_OFF,x
        lda PSAVE + (LIVES - PBLK),y    ; Leben des anderen
        beq .no
        jsr save_player
        lda CUR
        eor #1
        sta CUR
        jsr load_player
        ldx CUR
        lda PFRESH,x
        eor #1
        sta RESTORE                     ; 1, wenn der Spieler schon ein Gitter hat
        lda #St.levelinit
        sta STATE
        sec
        rts
.no:    clc
        rts

; save_player: Spielerblock und Gitter des aktiven Spielers sichern (PSAVE, GSAVE).  Veraendert A, X, Y, PTR_A..PTR_C, T0, T1.
save_player:
        ldx CUR
        ldy PSAVE_OFF,x
        ldx #0
.b:     lda PBLK,x
        sta PSAVE,y
        iny
        inx
        cpx #PBLK_LEN
        bne .b
        jsr gbuf_ptr
        jmp grid.save

; load_player: Spielerblock des aktiven Spielers aus PSAVE laden.  Veraendert A, X, Y.
load_player:
        ldx CUR
        ldy PSAVE_OFF,x
        ldx #0
.b:     lda PSAVE,y
        sta PBLK,x
        iny
        inx
        cpx #PBLK_LEN
        bne .b
        rts

; gbuf_ptr: PTR_C = Gitterpuffer des aktiven Spielers.  Veraendert A, X.
gbuf_ptr:
        ldx CUR
        lda GBUF_LO,x
        sta PTR_C
        lda GBUF_HI,x
        sta PTR_C + 1
        rts

GBUF_LO:   .byte <GSAVE, <(GSAVE + GSAVE_LEN)
GBUF_HI:   .byte >GSAVE, >(GSAVE + GSAVE_LEN)
.endif

PSAVE_OFF: .byte 0, PBLK_LEN            ; Abstand des Spielerblocks von Spieler 0 und 1 in PSAVE

; LEVELCLEAR: Bonus 500, naechste Stufe; alle 2 Stufen Tempo + 1 (bis 9)
st_levelclear:
        ldx #HT_CLEAR
        jsr hud.msg
        lda #Fx.clear
        jsr sound.play
        lda #5
        ldx #1
        jsr hud.bcd_add                 ; 500 Punkte
        lda LEVEL
        cmp #$99                        ; Stufe 99 ist die letzte (BCD)
        beq .keep
        sed
        clc
        adc #1
        sta LEVEL
        cld
        and #1
        bne .keep
        lda SPEED
        cmp #9
        bcs .keep
        inc SPEED
.keep:  jsr hud.update
        ldx #100
        jsr delay
        lda #0
        sta RESTORE
        lda #St.levelinit
        sta STATE
        rts

; GAMEOVER: Meldung, Bestenliste und beste Punkte merken, zurueck zum Titelbild
st_gameover:
.ifdef TESTDEMO
        inc TD_BASE + 2
.endif
        ldx #HT_OVER
        jsr hud.msg
        lda #Fx.over
        jsr sound.play
        ldx #150
        jsr delay
        jsr final_scores
        lda #St.title
        sta STATE
        rts

; final_scores: Punkte jedes Spielers mit der besten Punktzahl und der Bestenliste abgleichen.  Veraendert A, X, Y und mehr.
final_scores:
.if EXTRA_2P
        jsr save_player                 ; beide Spielerbloecke liegen danach in PSAVE
.else
        ldy #PBLK_LEN - 1
.sv:    lda PBLK,y
        sta PSAVE,y
        dey
        bpl .sv
.endif
        lda #0
        sta FSI                         ; Spieler 0, 1
.pl:    ldx FSI
        ldy PSAVE_OFF,x
        lda PSAVE + (SCORE - PBLK),y
        sta T0
        lda PSAVE + (SCORE - PBLK) + 1,y
        sta T1
        lda PSAVE + (SCORE - PBLK) + 2,y
        sta T2
        lda PSAVE + (LEVEL - PBLK),y
        sta T3
        lda FSI
        jsr score_done
        inc FSI
        lda FSI
        cmp NPLAYERS
        bcc .pl
        rts

; score_done: ein Endstand (T0..T2 = Punkte BCD niedrigstes zuerst, T3 = Stufe, A = Spieler): beste Punkte und Bestenliste.
score_done:
        sta T5
        lda T2                          ; BCD-Bytes lassen sich wie Binaerzahlen vergleichen
        cmp HISCORE + 2
        bne .cmp
        lda T1
        cmp HISCORE + 1
        bne .cmp
        lda T0
        cmp HISCORE
.cmp:   bcc .keep
        lda T0
        sta HISCORE
        lda T1
        sta HISCORE + 1
        lda T2
        sta HISCORE + 2
.keep:
.if EXTRA_HISCORE
        lda T5
        jmp hiscore.enter               ; T0..T3, A = Spieler: Platz bestimmen, Name eingeben, speichern
.else
        rts
.endif

.ifdef TESTDEMO
; td_run: Testeinstieg fuer vcc test (Trap): Spiel einrichten (wie start, ohne Titelbild und Diskette), neues Spiel, dann X Bilder der Zustandsprozeduren.
;   Kehrt zurueck; der Zustand steht danach im RAM (STATE, Bildschirm).  Nur mit -D TESTDEMO=1 (und AUTOPLAY).
td_run: stx T7
        sei
        lda #0
        tax
.zv:    sta VB,x
        sta VB + $100,x
        inx
        bne .zv
        sta FLAGS
        lda #1
        sta NPLAYERS
        lda #START_SPEED
        sta title_speed
        jsr antic.init
        jsr setfps
        jsr sound.init
        jsr input.setup
.if EXTRA_HISCORE
        jsr hiscore.default
.endif
        jsr new_game
        lda #St.levelinit
        sta STATE
.l:     jsr tick
        jsr dispatch
        dec T7
        bne .l
        rts

; testdemo: Zaehler fuer den Dauerlauf: Bilder (16 Bit), hoechste Stufe, gefuellte Rechtecke; Anzeige in Zeile 24 (hexadezimal).  Veraendert A, X, Y.
testdemo:
        inc TD_BASE
        bne .a
        inc TD_BASE + 1
.a:     lda LEVEL
        cmp TD_BASE + 3
        bcc .b
        sta TD_BASE + 3
.b:     ldx #0
        ldy #0
.show:  lda TD_BASE,x
        pha
        lsr
        lsr
        lsr
        lsr
        jsr .hex
        sta SCR + MSG_ROW * SCR_COLS + 33,y
        iny
        pla
        and #15
        jsr .hex
        sta SCR + MSG_ROW * SCR_COLS + 33,y
        iny
        inx
        cpx #3
        bne .show
        lda FPS                         ; Bilder je Sekunde (Norm) in Spalte 28, 29
        pha
        lsr
        lsr
        lsr
        lsr
        jsr .hex
        sta SCR + MSG_ROW * SCR_COLS + 28
        pla
        and #15
        jsr .hex
        sta SCR + MSG_ROW * SCR_COLS + 29
        rts
.hex:   cmp #10
        bcc .dg
        adc #22                         ; A-F = Codes 33..38 (C = 1 nach cmp: Ziffer + 1 + 22)
        rts
.dg:    ora #DIG2
        rts
.endif

.if FX_FLASH
; fx_fill: Fuellanimation (nur im Zustand FILL, ein Aufruf je Bild aus tick): die neueste Zeile jedes vorgemerkten Rechtecks blitzt (weiss, gelb), indem die Zeilenfarbe
;   COLPF2 dieser Bildschirmzeile in RC_P2 kurz ersetzt wird; danach gilt wieder die Vorlage GAME_P2.  Die Spiellogik bleibt unberuehrt.  Veraendert A, X, Y, FXQ, FXR.
fx_fill:
        lda STATE
        cmp #St.fill
        bne .out
        lda FILLQ + 2                   ; Zeilen fertig
        beq .out
        ldx #0
.q:     stx FXQ
        lda FILLQ,x
        lsr
        lsr
        lsr                             ; Rechteckzeile r
        asl
        asl                             ; 4 r
        clc
        adc FILLQ + 2
        adc #GRID_ROW0                  ; Bildschirmzeile der neuesten Zeile: gy = 4 r + FILLQ + 2, + GRID_ROW0
        tay
        ldx FILLQ + 3                   ; Blitzfarbe nach Bildzaehler (ab dem dritten Bild wieder die Fuellfarbe)
        cpx #2
        bcs .rest
        lda FLASHC,x
        bne .set
.rest:  lda GAME_P2,y
.set:   sta RC_P2,y
        ldx FXQ
        inx
        cpx FILLS_NOW
        bcc .q
.out:   rts
FLASHC: .byte $0e, $1e
.endif

.include "antic.asm"
.include "dli.asm"
.include "pmg.asm"
.include "grid.asm"
.include "input.asm"
.include "player.asm"
.include "enemies.asm"
.if EXTRA_ITEMS
.include "items.asm"
.endif
.include "sound.asm"
.include "hud.asm"
.if EXTRA_HISCORE
.include "disk.asm"
.include "hiscore.asm"
.endif
.include "gfx.inc"
.include "data.inc"
.ifdef SCRIPT
SCRIPT_TAB:
.if SCRIPT == 1
.include "tools/script1.inc"
.endif
.if SCRIPT == 2
.include "tools/script2.inc"
.endif
.if SCRIPT == 3
.include "tools/script3.inc"
.endif
.if SCRIPT == 4
.include "tools/script4.inc"
.endif
.endif
.ifdef TESTPIA
; pia_read: Testhilfe (nur mit -D TESTPIA=1, siehe troxx_pia.vctest): A = Wert fuer die PIA-Port-A-Leitungen, X = Anschluss (1 oder 2).  Schaltet PORTA kurz
;   auf Ausgang, damit der Lesezugriff den ausgegebenen Wert liefert, ruft input.read_port auf und stellt den Eingang wieder her.  Ergebnis in A.
pia_read:
        pha
        txa
        pha
        lda #0
        sta PACTL                       ; Richtungsregister waehlen
        lda #$ff
        sta PORTA                       ; alle Leitungen Ausgang
        lda #4
        sta PACTL                       ; Datenregister waehlen
        pla
        tax
        pla
        sta PORTA
        jsr input.read_port
        pha
        lda #0
        sta PACTL
        sta PORTA                       ; wieder Eingang
        lda #4
        sta PACTL
        pla
        rts
.endif
PRG_END:                                ; Ende des Programms (Pruefung in troxx.vctest)
.run start
