; =====================================================================================================================
; main.asm - TROXX fuer den Plus/4: Konstanten, Speicherplan, Start, Zustandsautomat
; =====================================================================================================================
; Herkunft: examples/c64/troxx/main.asm (Stand Commit d173cb4a) und examples/vic20/troxx/main.asm (Referenzfassung, Stand Commit 4697d074).
;   Diese Datei ist Plus/4-spezifisch (Zustandsautomat mit zwei Spielern, Bestenliste, Figuren als Zeichen); die gemeinsame Logik steht in grid.asm,
;   enemies.asm, hud.asm (siehe KOPIEN.txt).
;
; ZWECK
;   Troxx ist ein eigenstaendiges Arcade-Spiel: Eine Figur faehrt die Linien eines festen Gitters (8 x 5 Rechtecke) ab und faerbt sie; sind alle
;   vier Kanten eines Rechtecks abgefahren, wird es gefuellt.  Gegner laufen auf denselben Linien und kosten ein Leben.  Sind alle 40 Rechtecke
;   gefuellt, beginnt die naechste Stufe.  Regeln wie die Referenzfassung (VC-20); der Plus/4 bringt: TED mit 121 Farben (Helligkeitsstufen,
;   Fuellungen mit Verlauf und Abschraegung), Zeichensatz im RAM, Figuren aus Zeichen mit vier Animationsbildern, Rasterinterrupt (Randbalken,
;   Hintergrundbaender, Bildlauf im Titelbild), zwei Stimmen (TED), und Zusatzfunktionen: Bonusstern, Eisblock, Waechter, ein zweites Feldlayout
;   ab Stufe 5, Bestenliste auf Diskette, zwei Spieler abwechselnd mit Joystickwahl.
;
; BEDIENUNG
;   Titelbild: Joystick hoch/runter waehlt die Zeile (Spieler, Joystick, Tempo), links/rechts aendert, Feuer startet.
;   Spiel: Joystick; die Figur laeuft nur, solange eine Richtung gehalten wird (an Knoten abbiegen, auf Linien nur vor und zurueck).  Feuer = Pause.
;   Punkte: 1 je neu gefaerbtes Zeichen, 10 je abgeschlossenes Segment, 100 je gefuelltes Rechteck (zwei Rechtecke in einem Zug: zusammen 400),
;   500 je Stufe, Bonusstern 50.  Extraleben je 10000 Punkte, hoechstens 9 Leben.
;   Test ohne Joystick: -D AUTOPLAY=1 (der Rechner spielt selbst, das Titelbild entfaellt); -D AUTO_PLAYERS=2 (zwei Spieler), -D AUTOGOD=1
;   (unverwundbar), -D TESTDEMO=1 (Zaehler in Zeile 1 und im RAM ab $5020).  Zusatzfunktionen abschalten: -D EXTRA_STAR=0, EXTRA_ICE=0,
;   EXTRA_GUARD=0, EXTRA_HISCORE=0, EXTRA_2P=0, EXTRA_LAYOUT=0 (dann gelten genau die Regeln der Referenzfassung); Effekte: FX_RASTER=0, FX_TITLE=0,
;   FX_FLASH=0, FX_SHADE=0.
;
; ABLAUF (Zustandsautomat, ein Aufruf je Bild)
;   TITLE -> LEVELINIT -> PLAYING <-> FILL;  PLAYING -> HIT -> PLAYING / LEVELINIT (anderer Spieler) / GAMEOVER -> TITLE;
;   PLAYING/FILL -> LEVELCLEAR -> LEVELINIT.   Hauptschleife: tick (wait_frame, figs.update, sound.update) -> Zustandsprozedur (STATES).
;
; WAS DER CODE AUF DER HARDWARE TUT
;   TED: Bildschirm $0C00, Farbspeicher $0800, Zeichensatz-RAM $4000 (256 Zeichen); Rasterinterrupt ueber den KERNAL-Vektor $0314 (KERNAL-ROM
;   bleibt eingeblendet, damit die Diskette erreichbar ist; die Tastaturabfrage des KERNAL laeuft nicht); Joystick ueber $FF08 (Tastaturmatrix);
;   Ton in $FF0E-$FF12.  Der NMI der RESTORE-Taste zeigt auf einen RTI-Stumpf.
;
; SPEICHERPLAN
;   $0002-$007F  Zeropage (Plan unten; $90 und hoeher bleibt dem KERNAL fuer die Diskette)
;   $0800-$0BE7  Farbspeicher      $0C00-$0FE7  Bildschirm
;   $1001-$3FFF  Programm (Segment PROGRAM): BASIC-Zeile, Code, Tabellen, Zeichenvorlagen
;   $4000-$47FF  Zeichensatz-RAM (256 Zeichen: 0..127 Kopie des Zeichen-ROMs, 128..255 eigene Grafik)
;   $4800-$4AB4  Schattenkopie des Gitters, Zeichen (33 x 21)     $4B00-$4DB4  Schattenkopie, Farben
;   $5000-$5FFF  Spielvariablen (Plan unten), Bestenliste, Spielstand des zweiten Spielers
;
; SPEICHERPLAN DER ZEROPAGE
;   $02 STATE  $03 FLAGS  $04 FRAME  $05 STEPLEN  $06 RND  $07 CUR (aktiver Spieler 0/1)
;   $08-$37 Spielerblock PBLK (48 Byte, wird beim Spielerwechsel gesichert): $08 SPEED  $09 LEVEL (BCD)  $0A LIVES  $0B-$0D SCORE (BCD)
;           $0E Ueberlauf  $0F BOXES_DONE  $10-$37 BOXCNT (40 Byte)
;   $38-$3A HISCORE   $3B-$40 PX PY PDIR PWANT PANIM PINV   $43 FILLS_NOW  $44 NENEMY   $45-$5C EN (4 Gegner zu 6 Byte)
;   $5D-$64 T0..T7   $65-$6A PTR_A PTR_B PTR_C   $6B-$75 Gegner- und Hilfswerte   $76-$79 FILLQ   $7A-$7B SPTR   $7C-$7F PTR_D PTR_E
; =====================================================================================================================
; ---------------------------------------- English ----------------------------------------
; =====================================================================================================================
; main.asm - TROXX for the Plus/4: constants, memory map, start, state machine
; =====================================================================================================================
; Origin: examples/c64/troxx/main.asm (as of commit d173cb4a) and examples/vic20/troxx/main.asm (reference version, as of commit 4697d074).
;   This file is Plus/4-specific (state machine with two players, high score list, figures as characters); the shared logic is in grid.asm,
;   enemies.asm, hud.asm (see KOPIEN.txt).
;
; PURPOSE
;   Troxx is a self-contained arcade game: a figure travels along the lines of a fixed grid (8 x 5 rectangles) and colors them; when all
;   four edges of a rectangle have been traveled, it is filled.  Enemies walk on the same lines and cost a life.  When all 40 rectangles are
;   filled, the next level begins.  Rules as in the reference version (VIC-20); the Plus/4 brings: TED with 121 colors (brightness levels,
;   fills with gradient and bevel), character set in RAM, figures made of characters with four animation frames, raster interrupt (border bars,
;   background bands, scrolling on the title screen), two voices (TED), and extra features: bonus star, ice block, Guard, a second field layout
;   from level 5, high score list on disk, two players in turns with joystick choice.
;
; OPERATION
;   Title screen: joystick up/down selects the row (players, joystick, speed), left/right changes, fire starts.
;   Game: joystick; the figure runs only while a direction is held (turn at nodes, on lines only forward and back).  Fire = pause.
;   Points: 1 per newly colored character, 10 per completed segment, 100 per filled rectangle (two rectangles in one move: 400 together),
;   500 per level, bonus star 50.  Extra life every 10000 points, at most 9 lives.
;   Test without joystick: -D AUTOPLAY=1 (the computer plays by itself, no title screen); -D AUTO_PLAYERS=2 (two players), -D AUTOGOD=1
;   (invulnerable), -D TESTDEMO=1 (counters in row 1 and in RAM from $5020).  Switch off extra features: -D EXTRA_STAR=0, EXTRA_ICE=0,
;   EXTRA_GUARD=0, EXTRA_HISCORE=0, EXTRA_2P=0, EXTRA_LAYOUT=0 (then exactly the rules of the reference version apply); effects: FX_RASTER=0, FX_TITLE=0,
;   FX_FLASH=0, FX_SHADE=0.
;
; FLOW (state machine, one call per frame)
;   TITLE -> LEVELINIT -> PLAYING <-> FILL;  PLAYING -> HIT -> PLAYING / LEVELINIT (other player) / GAMEOVER -> TITLE;
;   PLAYING/FILL -> LEVELCLEAR -> LEVELINIT.   Main loop: tick (wait_frame, figs.update, sound.update) -> state procedure (STATES).
;
; WHAT THE CODE DOES ON THE HARDWARE
;   TED: screen $0C00, color RAM $0800, character set RAM $4000 (256 characters); raster interrupt via the KERNAL vector $0314 (KERNAL ROM
;   stays switched in so the disk can be reached; the KERNAL keyboard scan does not run); joystick via $FF08 (keyboard matrix);
;   sound in $FF0E-$FF12.  The NMI of the RESTORE key points to an RTI stub.
;
; MEMORY MAP
;   $0002-$007F  Zero page (map below; $90 and higher stays with the KERNAL for the disk)
;   $0800-$0BE7  Color RAM         $0C00-$0FE7  Screen
;   $1001-$3FFF  Program (segment PROGRAM): BASIC line, code, tables, character templates
;   $4000-$47FF  Character set RAM (256 characters: 0..127 copy of the character ROM, 128..255 own graphics)
;   $4800-$4AB4  Shadow copy of the grid, characters (33 x 21)     $4B00-$4DB4  Shadow copy, colors
;   $5000-$5FFF  Game variables (map below), high score list, state of the second player
;
; ZERO PAGE MAP
;   $02 STATE  $03 FLAGS  $04 FRAME  $05 STEPLEN  $06 RND  $07 CUR (active player 0/1)
;   $08-$37 player block PBLK (48 bytes, saved when the player changes): $08 SPEED  $09 LEVEL (BCD)  $0A LIVES  $0B-$0D SCORE (BCD)
;           $0E overflow  $0F BOXES_DONE  $10-$37 BOXCNT (40 bytes)
;   $38-$3A HISCORE   $3B-$40 PX PY PDIR PWANT PANIM PINV   $43 FILLS_NOW  $44 NENEMY   $45-$5C EN (4 enemies of 6 bytes)
;   $5D-$64 T0..T7   $65-$6A PTR_A PTR_B PTR_C   $6B-$75 enemy and helper values   $76-$79 FILLQ   $7A-$7B SPTR   $7C-$7F PTR_D PTR_E

#include <plus4/hardware.asm>
#include <plus4/kernal.asm>

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
.ifndef EXTRA_LAYOUT
.const EXTRA_LAYOUT = 1                 ; zweites Feldlayout ab Stufe 5 (sechs vorgefuellte Bloecke als Hindernisse)
.endif
.ifndef FX_RASTER
.const FX_RASTER = 1                    ; Rasterinterrupt: Farbbalken im Rand, Hintergrundbaender, Bildlauf im Titelbild
.endif
.ifndef FX_TITLE
.const FX_TITLE = 1                     ; Titelbild mit grossem Mehrfarben-Logo (Farbzyklus) und weichem Bildlauf (braucht FX_RASTER = 1)
.endif
.ifndef FX_FLASH
.const FX_FLASH = 1                     ; Fuellanimation: die neueste Zeile blitzt weiss/gelb auf
.endif
.ifndef FX_SHADE
.const FX_SHADE = 1                     ; Farbverlauf der Fuellungen je Zeile und Farbband je Rechteckzeile (Hintergrundfarben 1 und 2 per Raster)
.endif
.ifndef ENEMY_DESTROYS_LINES
.const ENEMY_DESTROYS_LINES = 1         ; Spielregel (Konzept 16.2): Gegner zerstoeren Linien, die an kein gefuelltes Rechteck grenzen (0 = alte Regel)
.endif
.ifndef FLASH_FRAMES
.const FLASH_FRAMES = 2                 ; Bilder des Aufblitzens beim Zerstoeren einer Linie (laenger nur fuer Bildschirmfotos)
.endif
.ifndef AUTO_PLAYERS
.const AUTO_PLAYERS = 1
.endif
.const EXTRA_ITEMS = EXTRA_STAR | EXTRA_ICE

; ---- Gitter (Abschnitt 2.1 des Konzepts, Fassung Plus/4): 8 x 5 Rechtecke, Innenraum 3 x 3 Zeichen
.const BOX_COLS  = 8
.const BOX_ROWS  = 5
.const BOX_W     = 3
.const BOX_H     = 3
.const GRID_W    = BOX_COLS * (BOX_W + 1) + 1   ; 33 Spalten (gx 0..32); senkrechte Linien bei gx = 0, 4, 8, ... 32
.const GRID_H    = BOX_ROWS * (BOX_H + 1) + 1   ; 21 Zeilen (gy 0..20); waagerechte Linien bei gy = 0, 4, 8, 12, 16, 20
.const NBOX      = BOX_COLS * BOX_ROWS          ; 40 Rechtecke
.const SCR_COLS  = 40
.const SCR_ROWS  = 25
.const GRID_ROW0 = 2                            ; Bildschirmzeile von gy = 0 (Zeilen 2..22); Anzeigen oben (Zeile 0), Meldungen (1), Leben und Tempo (23, 24)
.const GRID_COL0 = 3                            ; Bildschirmspalte von gx = 0 (Spalten 3..35)
.const COLOR_DELTA = ((>COLOR_RAM) - (>SCREEN_RAM)) & 255   ; High-Byte-Abstand Farbspeicher - Bildschirm ($FC = -4); die Low-Bytes sind gleich
.const CHARSET   = $4000
.const SHCH      = $4800                        ; Schattenkopie des Gitters: Zeichen (GRID_W * GRID_H Byte)
.const SHCOL     = $4b00                        ; Schattenkopie: Farben; der Abstand muss ein Vielfaches von 256 sein
.const SH_DELTA  = (>SHCOL) - (>SHCH)
.const SHLEN     = GRID_W * GRID_H              ; 693

; ---- Zeichencodes: 0..127 Kopie des Zeichen-ROMs (Bildschirmcodes: A-Z = 1..26, Leerzeichen 32, Ziffern 48..57); 128..255 eigene Grafik (siehe charset.asm)
.const CH_SPACE   = 32
.const CH_HLINE   = 128
.const CH_VLINE   = 129
.const CH_CROSS   = 130
.const CH_TL      = 131                 ; Ecke oben links, oben rechts, unten links, unten rechts
.const CH_TR      = 132
.const CH_BL      = 133
.const CH_BR      = 134
.const CH_TDOWN   = 135                 ; T-Stuecke: nach unten, oben, rechts, links offen
.const CH_TUP     = 136
.const CH_TRIGHT  = 137
.const CH_TLEFT   = 138
.const CH_HEART   = 139
.const CH_STAR    = 140
.const CH_ICE     = 141
.const CH_ARROW   = 142                 ; Auswahlzeiger im Titelbild
.const CH_TILE    = 143                 ; abgeschraegter Mehrfarben-Block (Logo im Titelbild)
.const CH_HIT     = 144                 ; Treffer-Bild der Figur
.const CH_PLAYER  = 145                 ; 145..148 Figur (vier Bilder)
.const CH_ENEMY0  = 149                 ; 149..152 Laeufer, 153..156 Streuner, 157..160 Waechter (je vier Bilder)
.const CH_FILL0   = 161                 ; 161..196: vier Fuellmuster zu je 9 Zeichen (Mehrfarben, Lage im Block: Reihe * 3 + Spalte)
.const CH_ARROWUP = 30                  ; Pfeil nach oben (aus dem ROM-Zeichensatz, Namenseingabe)
.const NGFX       = 69                  ; Anzahl eigener Zeichen ab Code 128 (siehe charset.asm)

; ---- TED-Farben: Farbspeicher-Byte = Helligkeit (Bit 4-6) * 16 + Farbton (Bit 0-3); Bit 3 = 1 schaltet im Mehrfarbenmodus das Zeichen auf Mehrfarben
;      Farbtoene: 0 schwarz, 1 weiss, 2 rot, 3 cyan, 4 violett, 5 gruen, 6 blau, 7 gelb, 8 orange, 9 braun, 10 gelbgruen, 11 rosa, 12 blaugruen,
;      13 hellblau, 14 dunkelblau, 15 hellgruen.  Zellen (Text, Linien, Figuren) nutzen nur die Farbtoene 0..7: Bit 3 des Farbtons waere im Mehrfarbenmodus das
;      Mehrfarben-Bit.  Hintergrund, Rahmen und die Mehrfarben-Hintergruende 1 und 2 duerfen alle 121 Farben haben.
.const COL_BLACK  = $00
.const COL_WHITE  = $71
.const COL_GRAY   = $61
.const COL_RED    = $52
.const COL_CYAN   = $63
.const COL_PURPLE = $64
.const COL_GREEN  = $55
.const COL_BLUE   = $56
.const COL_YELLOW = $77
.const COL_ORANGE = $47
.const COL_PINK   = $72
.const COL_LTBLUE = $76
.const COL_LINE   = $46                 ; unberuehrte Linie: blau
.const COL_DONE   = $67                 ; abgefahrene Linie: hellgelb
.const COL_MULTI  = 8                   ; Bit 3: Mehrfarben-Zeichen (Fuellungen, Logo)
.const BG0_COLOR  = $06                 ; Hintergrund: sehr dunkles Blau
.const BORDER_COLOR = $36               ; Rahmen: mittleres Blau

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

; ---- Spieleinstellungen
.ifndef LIVES0
.const LIVES0      = 3
.endif
.ifndef START_SPEED
.const START_SPEED = 5
.endif
.const INVULN      = 12                 ; Spielschritte Unverwundbarkeit nach einem Treffer
.const KIND_RUNNER = 0
.const KIND_WANDER = 1
.const KIND_GUARD  = 2
.ifndef LAYOUT2_LEVEL
.const LAYOUT2_LEVEL = 5                ; ab dieser Stufe gilt das zweite Feldlayout (EXTRA_LAYOUT)
.endif

; ---- Zeropage (siehe Plan oben)
.const STATE    = $02
.const FLAGS    = $03
.const FRAME    = $04                   ; Bilder seit dem letzten Spielschritt
.const STEPLEN  = $05                   ; Bilder je Spielschritt (aus der Tempotabelle)
.const RND      = $06                   ; Zustand des Zufallsgenerators (nie 0)
.const CUR      = $07                   ; aktiver Spieler (0 oder 1)
.const PBLK     = $08                   ; Spielerblock: wird beim Spielerwechsel als Ganzes gesichert und geladen
.const SPEED    = $08                   ; Tempo 1..9
.const LEVEL    = $09                   ; Stufe, BCD
.const LIVES    = $0a
.const SCORE    = $0b                   ; Punkte, BCD, 3 Byte (niedrigstes zuerst)
.const SCORE_OVF = $0e                  ; Ueberlauf des Punktestands (nur Rechenhilfe)
.const BOXES_DONE = $0f                 ; gefuellte Rechtecke
.const BOXCNT   = $10                   ; je Rechteck abgeschlossene Kanten (40 Byte)
.const PBLK_LEN = 3 + 4 + 1 + NBOX      ; SPEED, LEVEL, LIVES, SCORE (3), Ueberlauf, BOXES_DONE, BOXCNT
.const HISCORE  = $38                   ; beste Punkte (Bestenliste oder Sitzung), BCD, 3 Byte
.const PX       = $3b                   ; Figur: Gitterzelle (gx, gy)
.const PY       = $3c
.const PDIR     = $3d                   ; letzte Richtung (Dir)
.const PWANT    = $3e                   ; gehaltene Richtungen (Maske IN_*)
.const PANIM    = $3f                   ; Schrittzaehler der Figur (nur Anzeige)
.const PINV     = $40                   ; Rest der Unverwundbarkeit in Spielschritten
.const FILLS_NOW  = $43                 ; in diesem Spielschritt abgeschlossene Rechtecke (0..2)
.const NENEMY   = $44
.const EN       = $45                   ; Gegner: 4 * 6 Byte
.const E_X      = 0                     ; Feldabstaende je Gegner
.const E_Y      = 1
.const E_DIR    = 2
.const E_KIND   = 3                     ; 0 Laeufer, 1 Streuner, 2 Waechter
.const E_SIZE   = 6                     ; (zwei Byte unbenutzt: der Abstand 6 bleibt wie im C64-Projekt)
.const T0       = $5d                   ; Zwischenwerte T0..T7
.const T1       = $5e
.const T2       = $5f
.const T3       = $60
.const T4       = $61
.const T5       = $62
.const T6       = $63
.const T7       = $64
.const PTR_A    = $65                   ; Zeiger Bildschirm
.const PTR_B    = $67                   ; Zeiger Farbspeicher
.const PTR_C    = $69                   ; Zeiger Puffer (Gitter sichern und laden)
.const HITF     = $6c                   ; 1 = Zusammenstoss in diesem Spielschritt
.const EPH      = $6d                   ; Bild der Gegner
.const ECUR     = $6e                   ; laufender Gegner (Abstand in EN)
.const ELIM     = $6f                   ; Ende der Gegnerliste (Gegnerzahl * E_SIZE)
.const EBEST    = $70
.const EDIR     = $71
.const EDIST    = $72
.const EQX      = $73
.const EQY      = $74
.const ATMP     = $75
.const FILLQ    = $76                   ; Fuellwarteschlange: +0, +1 Rechtecke, +2 Zeile, +3 Bildzaehler
.const SPTR     = $7a                   ; Klangmodul: Zeiger in den Strom (2 Byte)
.const PTR_D    = $7c                   ; Zeiger Schattenkopie Zeichen (Zeile des Gitters)
.const PTR_E    = $7e                   ; Zeiger Schattenkopie Farben (Zeile des Gitters)
.assert BOXCNT + NBOX == PBLK + PBLK_LEN, "Spielerblock lueckenlos"
.assert PBLK + PBLK_LEN <= HISCORE, "Zeropage-Plan"
.assert EN + 4 * E_SIZE <= T0, "Zeropage-Plan"
.assert SHCOL - SHCH == SH_DELTA * 256, "Schattenkopien: Abstand ein Vielfaches von 256"

; ---- Spielvariablen im RAM ($5000, ausserhalb des Programms)
.const VARS      = $5000
.const JOYMODE   = VARS                 ; 0 ein Joystick Anschluss 2, 1 ein Joystick Anschluss 1, 2 zwei Joysticks
.const NPLAYERS  = VARS + $01           ; 1 oder 2
.const SELROW    = VARS + $02           ; Titelbild: gewaehlte Zeile
.const RESTORE   = VARS + $03           ; 1 = LEVELINIT stellt das gesicherte Gitter wieder her
.const PFRESH    = VARS + $04           ; 2 Byte: 1 = Spieler hat noch kein Gitter gespielt
.const FRAMEC    = VARS + $06           ; Bildzaehler (vom Rasterinterrupt erhoeht)
.const FRAMEW    = VARS + $07           ; letzter von wait_frame gesehener Stand
.const SPHIT     = VARS + $08           ; 1 = Figur als Treffer-Bild zeigen
.const FIGOFF    = VARS + $09           ; 1 = Figuren verbergen (Titelbild, Namenseingabe)
.const FIG_OX    = VARS + $0a           ; 6 Plaetze (0 Figur, 1..4 Gegner, 5 Gegenstand): gezeichnete Zelle x ($FF = nichts)
.const FIG_OY    = VARS + $10           ; ... y (6 Byte)
.const FIG_T     = VARS + $16           ; Hilfswerte von figs.update (4 Byte)
.const IT_KIND   = VARS + $20           ; Gegenstand auf dem Feld: 0 keiner, 1 Stern, 2 Eisblock
.const IT_X      = VARS + $21
.const IT_Y      = VARS + $22
.const IT_TIME   = VARS + $23           ; 2 Byte: Rest der Sichtzeit in Bildern
.const IT_WAIT   = VARS + $25           ; 2 Byte: Wartezeit bis zum naechsten Gegenstand
.const FREEZE    = VARS + $27           ; 2 Byte: Rest der Gegner-Starre in Bildern
.const FPS       = VARS + $29           ; Bilder je Sekunde (50 oder 60)
.const FSI       = VARS + $2a           ; Schleifenzaehler von final_scores
.const FXC       = VARS + $2b           ; Zaehler fuer Farbzyklus (Titelbild)
.const DLYCNT    = VARS + $2c           ; Zaehler von delay
.const LAYOUT    = VARS + $2d           ; 1 oder 2: Feldlayout der laufenden Stufe
.const FILLGRAY  = VARS + $2e           ; 1 = fill_row fuellt grau (vorgefuellte Hindernisbloecke)
.const TD_BASE   = VARS + $30           ; TESTDEMO: +0,+1 Bilder, +2 Spiele, +3 hoechste Stufe, +4 Treffer, +5 Rechtecke gesamt
.const SCR_IDX   = VARS + $38           ; Testbetrieb SCRIPT: Eintrag, Rest der Dauer, Masken der Anschluesse 1 und 2
.const SCR_REM   = VARS + $39
.const SCR_M1    = VARS + $3a
.const SCR_M2    = VARS + $3b
.const SND       = VARS + $40           ; Klangmodul (siehe sound.asm), 32 Byte
.const RB_VAR    = VARS + $60           ; Rasterinterrupt (siehe raster.asm): +0 Eintrag, +1 Phase, +2 Anzahl, +3 Modus (0 Spiel, 1 Titelbild), +4 Versatz, +5 Start
.const SIDX      = VARS + $68           ; Bildlauf: Leseposition im Text
.const FXK       = VARS + $69           ; Hilfswerte der Fuellanimation
.const FXR       = VARS + $6a
.const FXG       = VARS + $6b
.const FXQ       = VARS + $6c
.const SDX       = VARS + $72           ; Segment-Durchlauf (grid.seg_first/seg_next/seg_destroy): laufende Zelle, Rest, gefaerbte Zeichen, Modus
.const SDY       = VARS + $73
.const SDN       = VARS + $74
.const SDP       = VARS + $75
.const SDM       = VARS + $76           ; Durchgang 0 zaehlen und aufblitzen, 1 zuruecksetzen
.const NB0       = VARS + $77           ; angrenzende Rechtecke eines Segments ($FF = keins): unten/rechts
.const NB1       = VARS + $78           ;                                                        oben/links
.const NODEX     = VARS + $79           ; Knotenpruefung (grid.nodes_free): laufender Knoten
.const NODEY     = VARS + $7a
.const OLDIRQ    = VARS + $70           ; Vektor $0314 des KERNALs (2 Byte), gesichert beim Start, zurueckgesetzt waehrend der Diskettenzugriffe
.const HS_BUF    = VARS + $80           ; Bestenliste (siehe hiscore.asm), 96 Byte
.const HS_TMP    = VARS + $100          ; Ladepuffer der Bestenliste (siehe hiscore.asm), 80 Byte
.const HS_ST     = VARS + $150          ; Ergebnis des letzten Plattenzugriffs (DK_OK ...)
.const HS_SC     = VARS + $151          ; Endstand (3 Byte Punkte, 1 Byte Stufe) fuer die Bestenliste
.const HS_NAME   = VARS + $155          ; Name (3 Zeichen)
.const HS_S      = VARS + $158          ; Offset des neuen Eintrags
.const HS_PL     = VARS + $159          ; Spielernummer
.const HS_CPOS   = VARS + $15a          ; Eingabeposition
.const HS_I      = VARS + $15b
.const PSAVE     = VARS + $180          ; gesicherter Spielerblock je Spieler (2 * PBLK_LEN)
.const GSAVE     = VARS + $200          ; gesichertes Gitter je Spieler (GSAVE_LEN Byte)
.const GSAVE_LEN = 2 * 768              ; je Spieler: Zeichen- und Farbkopie zu je drei Seiten (siehe grid.save)
.assert GSAVE + 2 * GSAVE_LEN <= VARS + $1000, "Spielvariablen im RAM-Block $5000-$5FFF"

.enum St
    title, levelinit, playing, fill, hit, levelclear, gameover
.endenum

; ---------------------------------------------------------------------------------------------------------------------
; Programm
; ---------------------------------------------------------------------------------------------------------------------
.segment "PROGRAM", $1001, $3fff
.basicstub start                        ; "10 SYS <start>"

; start: Einstieg ueber SYS.  Interrupts bleiben fuer immer aus (Abfrage der Eingabe und Bildtakt ohne Interrupt).
start:  sei
        cld
        ldx #$ff
        txs
        lda #<nmi_stub                  ; NMI der RESTORE-Taste: Vektor auf RTI (Hardwarevektor liegt im ROM; $0318 gilt bei eingeblendetem KERNAL)
        sta NMINV
        lda #>nmi_stub
        sta NMINV + 1
        lda #0
        tax
.zv:    sta VARS,x                      ; Spielvariablen loeschen
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
        jsr ted.init                    ; Norm erkennen, Bildregister, Zeichensatz einrichten, Bildschirm loeschen
        jsr sound.init
        jsr input.setup
        jsr raster.start                ; Rasterinterrupt: Bildtakt und Randbalken (ab hier freigegeben; Diskettenzugriffe sperren ihn)
.if EXTRA_HISCORE
        jsr hiscore.load                ; Bestenliste von Diskette (fehlt das Laufwerk, bleibt die Standardliste)
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

nmi_stub:
        rti

; tick: ein Bild: auf den Bildanfang warten, Figuren zeichnen, Klang weiterschalten.  Veraendert A, X, Y und mehr.
tick:   jsr ted.wait_frame
        jsr figs.update
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

; delay: X Bilder warten (Klang und Figuren laufen weiter).  Veraendert A, X, Y.
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
        jsr ted.clear
        lda #1
        sta FIGOFF                      ; keine Figuren im Titelbild
.if FX_RASTER && FX_TITLE
        lda #1                          ; Hintergrundbaender des Titelbilds
        sta RB_VAR + 3
        lda #7
        sta RB_VAR + 4
.endif
        jsr ted.title_colors
        jsr title_draw
.ifndef AUTOPLAY
        jsr sound.music_start
.sel:   jsr title_values
        jsr tick
.if FX_TITLE
        jsr title_fx
.endif
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
        sta FIGOFF
        sta RB_VAR + 3                  ; Spiel: keine Titel-Hintergrundbaender
        lda #8
        sta RB_VAR + 4                  ; Bildlauf-Versatz 8: 40 Spalten ohne Versatz
        jsr ted.clear
        jsr ted.game_colors
        jsr hud.draw
        lda RESTORE
        bne .restore
        jsr set_layout
        jsr grid.draw
        lda #0
        ldx #NBOX - 1
.clr:   sta BOXCNT,x                    ; BOXCNT leeren
        dex
        bpl .clr
        sta BOXES_DONE
.if EXTRA_LAYOUT
        lda LAYOUT
        cmp #2
        bne .go
        jsr grid.prefill                ; zweites Layout: Hindernisbloecke vorfuellen
.endif
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
        adc #49
        sta SCREEN_RAM + MSG_ROW * SCR_COLS + GRID_COL0 + 7
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
.zv:    sta VARS,x
        inx
        bne .zv
        sta FLAGS
        lda #1
        sta NPLAYERS
        lda #START_SPEED
        sta title_speed
        jsr ted.init
        jsr sound.init
        jsr input.setup
        jsr raster.start
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

; testdemo: Zaehler fuer den Dauerlauf: Bilder (16 Bit), hoechste Stufe, gefuellte Rechtecke; Anzeige in Zeile 1 (hexadezimal).  Veraendert A, X, Y.
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
        sta SCREEN_RAM + 1 * SCR_COLS + 33,y
        iny
        pla
        and #15
        jsr .hex
        sta SCREEN_RAM + 1 * SCR_COLS + 33,y
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
        sta SCREEN_RAM + 1 * SCR_COLS + 28
        pla
        and #15
        jsr .hex
        sta SCREEN_RAM + 1 * SCR_COLS + 29
        rts
.hex:   cmp #10
        bcc .dg
        sbc #9                          ; A-F = Codes 1..6
        rts
.dg:    ora #48
        rts
.endif

.if FX_FLASH
; fx_fill: Fuellanimation (nur im Zustand FILL, ein Aufruf je Bild aus tick): die neueste Zeile jedes vorgemerkten Rechtecks blitzt (weiss, gelb), aeltere
;   Zeilen bekommen ihre Fuellfarbe zurueck.  Die Spiellogik und die Schattenkopie bleiben unberuehrt; es aendert sich nur der Farbspeicher.
;   Veraendert A, X, Y, T0..T3, PTR_A, PTR_B (und PTR_D, PTR_E).
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
        lsr
        sta FXR                         ; Rechteckzeile r
        lda FILLQ,x
        and #7
        asl
        asl
        clc
        adc #1
        sta FXG                         ; erste Innenspalte gx
        ldy #0
.k:     sty FXK
        lda FXR
        asl
        asl
        clc
        adc #1
        adc FXK
        tay
        jsr grid.set_row
        ldx FXQ
        lda FILLQ,x
        and #7
        tay                             ; Y = Rechteckspalte c
        ldx FXR
        lda FXK
        jsr grid.fill_colour            ; A = Fuellfarbe der Zeile
        sta T3
        ldy FXK
        iny
        cpy FILLQ + 2
        bne .fin
        ldx FILLQ + 3                   ; neueste Zeile: Blitzfarbe nach Bildzaehler (im letzten Bild wieder die Fuellfarbe, sonst bliebe sie beim Ende stehen)
        cpx #2
        bcs .fin
        lda FLASHC,x
        ora #COL_MULTI
        sta T3
.fin:   ldy FXG
        ldx #BOX_W
.c:     lda T3
        sta (PTR_B),y
        iny
        dex
        bne .c
        ldy FXK
        iny
        cpy FILLQ + 2
        bcc .k
        ldx FXQ
        inx
        cpx FILLS_NOW
        bcc .q
.out:   rts
FLASHC: .byte COL_WHITE, COL_YELLOW
.endif

; set_layout: Feldlayout der Stufe LEVEL bestimmen (LAYOUT 1 oder 2).  Veraendert A.
set_layout:
        lda #1
.if EXTRA_LAYOUT
        ldx LEVEL
        cpx #LAYOUT2_LEVEL              ; BCD 1..9: gleich der Zahl; ab 10 immer das zweite Layout
        bcc .set
        lda #2
.endif
.set:   sta LAYOUT
        rts

.include "ted.asm"
.include "figs.asm"
.include "raster.asm"
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
.include "data.inc"
.ifdef SCRIPT
SCRIPT_TAB:
.if SCRIPT == 1
.include "tools/script1.inc"
.endif
.if SCRIPT == 2
.include "tools/script2.inc"
.endif
.endif
.include "charset.asm"
.ifdef TESTPAD
.include "tools/testpad.inc"
.endif
