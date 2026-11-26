; =====================================================================================================================
; main.asm - TROXX fuer den MEGA65 (VIC-IV): Konstanten, Speicherplan, Start, Zustandsautomat
; =====================================================================================================================
; Herkunft: examples/c64/troxx/main.asm (Stand Commit 4697d074 der Referenz).  Diese Datei ist MEGA65-spezifisch (Zustandsautomat fuer ein oder zwei Spieler,
;   abwechselnd oder gleichzeitig; Titelbild mit Logo; Bestenliste); die gemeinsame Logik steht in grid.asm, enemies.asm, hud.asm (siehe KOPIEN.txt).
;
; ZWECK     Troxx ist ein eigenstaendiges Arcade-Spiel: Eine Figur faehrt die Linien eines festen Gitters (14 x 7 Rechtecke) ab und faerbt sie; sind alle
;           vier Kanten eines Rechtecks abgefahren, wird es gefuellt.  Gegner laufen auf denselben Linien und kosten ein Leben; sie zerstoeren Linien, die
;           nicht durch ein gefuelltes Nachbarrechteck geschuetzt sind (Regel 16.2).  Sind alle 98 Rechtecke gefuellt, beginnt die naechste Stufe.
;           Regeln wie die Referenzfassung (VC-20); der MEGA65 bringt mehr Leistung: 80 x 50 Zeichen mit Vollfarb-Kacheln (VIC-IV), 256-Farben-Palette,
;           16-Farben-Sprites mit Pixelbewegung, Fuellanimation per DMA, 40 MHz, mehrstimmige Musik ueber vier SID, Zusatzfunktionen (Bonusstern,
;           Eisblock, Waechter, Bestenliste) und zwei Spieler, abwechselnd oder gleichzeitig.
;
; BEDIENUNG
;   Titelbild: Joystick hoch/runter waehlt die Zeile (Spieler, Ablauf, Joystick, Tempo), links/rechts aendert, Feuer startet.
;   Spiel: Joystick; die Figur laeuft nur, solange eine Richtung gehalten wird (an Knoten abbiegen, auf Linien nur vor und zurueck).  Feuer = Pause.
;   Punkte: 1 je neu gefaerbtes Zeichen, 10 je abgeschlossenes Segment, 100 je gefuelltes Rechteck (zwei Rechtecke in einem Zug: zusammen 400),
;   500 je Stufe, Bonusstern 50.  Gegner zerstoerte Linien kosten die gebuchten Punkte.  Extraleben je 10000 Punkte, hoechstens 9 Leben.
;   Test ohne Joystick: -D AUTOPLAY=1 (der Rechner spielt selbst, das Titelbild entfaellt); -D AUTO_PLAYERS=2 (zwei Spieler), -D AUTO_SIMUL=1
;   (gleichzeitig), -D AUTOGOD=1 (unverwundbar), -D TESTDEMO=1 (Zaehler im RAM), -D HOLD_DESTROY=n (nur Test: Aufblitzen zerstoerter Segmente n Bilder lang halten, n = 1..255).  Zusatzfunktionen abschalten: -D EXTRA_STAR=0, EXTRA_ICE=0,
;   EXTRA_GUARD=0, EXTRA_HISCORE=0, EXTRA_2P=0, ENEMY_DESTROYS_LINES=0 (dann gelten genau die Regeln der Referenzfassung ohne Regel 16.2).
;
; ABLAUF (Zustandsautomat, ein Aufruf je Bild)
;   TITLE -> LEVELINIT -> PLAYING <-> FILL;  PLAYING -> HIT -> PLAYING / LEVELINIT (anderer Spieler) / GAMEOVER -> TITLE;
;   PLAYING/FILL -> LEVELCLEAR -> LEVELINIT.   Hauptschleife: tick (wait_frame, Sprites, Klang) -> Zustandsprozedur (STATES).
;
; SPEICHERPLAN (Adressraum der CPU, ROMs ausgeblendet; Chip-RAM ueber 32-Bit-Zeiger und DMA)
;   $0002-$00BF  Zeropage (Plan unten)
;   $2001-$B7FF  Programm (BASIC-Zeile, Code, Tabellen, Kacheln, Sprite-Vorlagen, Palette)
;   $B800-$BFFF  Sprite-Daten ($B800 ff., 11 Bilder zu 128 Byte) und Sprite-Zeiger ($BFF8) in der VIC-Bank 2
;   $C000-$CD6F  Schattengitter CELL (43 Zeilen zu 80 Byte, ein Byte je Gitterzelle: Zustand), $CE00-$CFFF Spielvariablen
;   $E000-$FFEF  gesicherte Gitter der beiden Spieler (abwechselndes Spiel)
;   $0001D000    Bildschirm (80 x 50 Zellen zu 2 Byte), $0FF80000 Farb-RAM, $00012000 Kacheln, $00014000 Schrift
;
; SPEICHERPLAN DER ZEROPAGE
;   $02 STATE  $03 FLAGS  $04 FRAME  $05 STEPLEN  $06 RND  $07 CUR (Spieler am Zug)  $08 NFIG (Figuren)  $09 WHO (Spieler fuer Punkte)  $0A SPEED  $0B LEVEL (BCD)
;   $0C BOXES_DONE  $0D FILLS_NOW  $0E NENEMY  $0F ELIM  $10 HITF  $11 STEPCNT  $12 EPH  $13 ECUR  $14 FIG
;   $15-$28 Figuren (je zwei Byte): PX PY PDIR PWANT PANIM PINV POX POY PACT OWN   $29-$30 SCORE (2 * 3 + 2 Ueberlauf)  $31-$32 LIVES  $33-$35 HISCORE
;   $36-$53 EN (5 Gegner zu 6 Byte)  $54-$5B Gegnerhilfen  $5D-$64 T0..T7  $65-$6A PTR_A PTR_B PTR_C  $6B-$72 Segment (SDX..NB1)  $73-$7A RT0..RT7
;   $7B-$7E FILLQ  $7F-$80 SPTR (Klang)  $81-$88 Sprites  $89-$8A Knoten-Durchlauf  $A0-$A7 32-Bit-Zeiger sq, cq  $E0-$E3 32-Bit-Zeiger rp (Zeichen-ROM)
; =====================================================================================================================
; ---------------------------------------- English ----------------------------------------
; =====================================================================================================================
; main.asm - TROXX for the MEGA65 (VIC-IV): constants, memory map, start, state machine
; =====================================================================================================================
; Origin: examples/c64/troxx/main.asm (as of commit 4697d074 of the reference).  This file is MEGA65-specific (state machine for one or two players,
;   taking turns or simultaneously; title screen with logo; high score list); the shared logic is in grid.asm, enemies.asm, hud.asm (see KOPIEN.txt).
;
; PURPOSE   Troxx is a self-contained arcade game: a figure travels along the lines of a fixed grid (14 x 7 rectangles) and colors them; when all
;           four edges of a rectangle have been traveled, it is filled.  Enemies walk on the same lines and cost a life; they destroy lines that are
;           not protected by a filled neighbor rectangle (rule 16.2).  When all 98 rectangles are filled, the next level begins.
;           Rules as in the reference version (VIC-20); the MEGA65 offers more power: 80 x 50 characters with full-color tiles (VIC-IV), a 256-color
;           palette, 16-color sprites with pixel movement, fill animation by DMA, 40 MHz, multi-voice music over four SIDs, extra features (bonus star,
;           ice block, guard, high score list) and two players, taking turns or simultaneously.
;
; CONTROLS
;   Title screen: joystick up/down selects the row (players, flow, joystick, speed), left/right changes it, fire starts.
;   Game: joystick; the figure only moves while a direction is held (turn at nodes, on lines only forward and back).  Fire = pause.
;   Points: 1 per newly colored character, 10 per completed segment, 100 per filled rectangle (two rectangles in one move: 400 together),
;   500 per level, bonus star 50.  Lines destroyed by enemies cost the points booked for them.  Extra life per 10000 points, at most 9 lives.
;   Test without joystick: -D AUTOPLAY=1 (the computer plays itself, no title screen); -D AUTO_PLAYERS=2 (two players), -D AUTO_SIMUL=1
;   (simultaneous), -D AUTOGOD=1 (invulnerable), -D TESTDEMO=1 (counters in RAM), -D HOLD_DESTROY=n (test only: hold the flash of destroyed segments
;   for n frames, n = 1..255).  Switch off extra features: -D EXTRA_STAR=0, EXTRA_ICE=0, EXTRA_GUARD=0, EXTRA_HISCORE=0, EXTRA_2P=0,
;   ENEMY_DESTROYS_LINES=0 (then exactly the rules of the reference version without rule 16.2 apply).
;
; FLOW (state machine, one call per frame)
;   TITLE -> LEVELINIT -> PLAYING <-> FILL;  PLAYING -> HIT -> PLAYING / LEVELINIT (other player) / GAMEOVER -> TITLE;
;   PLAYING/FILL -> LEVELCLEAR -> LEVELINIT.   Main loop: tick (wait_frame, sprites, sound) -> state procedure (STATES).
;
; MEMORY MAP (CPU address space, ROMs hidden; chip RAM via 32-bit pointers and DMA)
;   $0002-$00BF  Zero page (map above)
;   $2001-$B7FF  Program (BASIC line, code, tables, tiles, sprite templates, palette)
;   $B800-$BFFF  Sprite data ($B800 ff., 11 images of 128 bytes) and sprite pointers ($BFF8) in VIC bank 2
;   $C000-$CD6F  Shadow grid CELL (43 rows of 80 bytes, one byte per grid cell: state), $CE00-$CFFF game variables
;   $E000-$FFEF  saved grids of the two players (turn-taking game)
;   $0001D000    Screen (80 x 50 cells of 2 bytes), $0FF80000 color RAM, $00012000 tiles, $00014000 font
; =====================================================================================================================

#include <mega65/hardware.asm>
#include <mega65/gs4502.asm>
#include <mega65/kernal.asm>

; ---- Testszenarien (nur Testfassungen, Beschreibung bei "scenario"): SC_ANY muss vor der ersten Abfrage stehen (der Assembler liest in einem Durchgang)
.ifdef SCENARIO_DESTROY
.const SC_ANY = 1
.endif
.ifdef SCENARIO_NODE
.const SC_ANY = 1
.endif

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
.const EXTRA_HISCORE = 1                ; Bestenliste (zehn Eintraege) auf der SD-Karte
.endif
.ifndef EXTRA_2P
.const EXTRA_2P = 1                     ; zwei Spieler (abwechselnd oder gleichzeitig), Joystickwahl im Titelbild
.endif
.ifndef ENEMY_DESTROYS_LINES
.const ENEMY_DESTROYS_LINES = 1         ; Regel 16.2: Gegner zerstoeren nicht geschlossene Linien
.endif
.ifndef FX_FILL
.const FX_FILL = 1                      ; Fuellanimation: Glanzzeile, Verlauf (per DMA)
.endif
.ifndef FX_TITLE
.const FX_TITLE = 1                     ; Titelbild mit grossem Logo und Farbzyklus der Palette, Pulsbreitenmodulation der Titelmusik
.endif
.ifndef AUTO_PLAYERS
.const AUTO_PLAYERS = 1
.endif
.ifndef AUTO_SIMUL
.const AUTO_SIMUL = 0
.endif
.const EXTRA_ITEMS = EXTRA_STAR | EXTRA_ICE

; ---- Gitter (Abschnitt 15.3 des Konzepts, Fassung MEGA65): 14 x 7 Rechtecke, Innenraum 4 x 5 Zeichen
.const BOX_COLS  = 14
.const BOX_ROWS  = 7
.const BOX_W     = 4
.const BOX_H     = 5
.const GRID_W    = BOX_COLS * (BOX_W + 1) + 1   ; 71 Spalten (gx 0..70); senkrechte Linien bei gx = 0, 5, 10, ... 70
.const GRID_H    = BOX_ROWS * (BOX_H + 1) + 1   ; 43 Zeilen (gy 0..42); waagerechte Linien bei gy = 0, 6, 12, ... 42
.const NBOX      = BOX_COLS * BOX_ROWS          ; 98 Rechtecke
.const SCR_COLS  = 80
.const SCR_ROWS  = 50
.const GRID_COL0 = 4                            ; Bildschirmspalte von gx = 0 (Spalten 4..74)
.const GRID_ROW0 = 0                            ; Bildschirmzeile von gy = 0 (Zeilen 0..42); Tafeln in den Zeilen 43..49

; ---- Speicherplan (siehe Kopf)
.const SCRQ       = $d000                       ; Bildschirm: $0001D000 (Bank 1)
.const TILE_RAM   = $12000                      ; Kachelspeicher (Chip-RAM)
.const FONT_RAM   = $14000                      ; Schrift (4 Themen zu 64 Zeichen zu 64 Byte)
.const TILE0      = TILE_RAM / 64               ; Zeichennummer der ersten Kachel ($0480)
.const FONT0      = FONT_RAM / 64               ; Zeichennummer des ersten Schriftzeichens ($0500)
.const SPR_DATA   = $b800                       ; Sprite-Daten in der VIC-Bank 2: Zeiger-Wert = (Adresse - $8000) / 64
.const SPR_PTR    = $bff8                       ; Sprite-Zeiger (8 Byte)
.const SPR_BLK    = (SPR_DATA - $8000) / 64     ; Zeigerwert des ersten Bildes ($E0)
.const CELL       = $c000                       ; Schattengitter (siehe grid.asm)
.const RASTER_LINE = 100                        ; Bildtakt: physische Rasterzeile (im oberen Rand)
.const CELL_LEN   = 80 * GRID_H                 ; Schattengitter: 3440 Byte
.const BSAVE0     = $e000                       ; gesichertes Gitter des Spielers 1 (CELL, BOXCNT, SPEED, LEVEL, BOXES_DONE)
.const BSAVE1     = $f000
.const BSAVE_LEN  = CELL_LEN + NBOX + 3

; ---- Palette: Eintraege aus assets.inc (siehe tools/gen_assets.py)
.const COL_BLACK  = 0
.const COL_WHITE  = 1
.const COL_RED    = 2
.const COL_CYAN   = 3
.const COL_PURPLE = 4
.const COL_GREEN  = 5
.const COL_BLUE   = 6
.const COL_YELLOW = 7
.const COL_ORANGE = 8
.const COL_BROWN  = 9
.const COL_LTRED  = 10
.const COL_DGREY  = 11
.const COL_GREY   = 12
.const COL_LTGREEN = 13
.const COL_LTBLUE = 14
.const COL_LTGREY = 15
.const PAL_PAN0   = 96                          ; Hintergrund der Tafeln (Thema 0 Spieler 1, 1 Spieler 2, 2 Stufe)
.const PAL_RAINBOW = 128                        ; 64 Regenbogenfarben (Titel)
.const CH_SPACE   = 32                          ; Bildschirmcode Leerzeichen (Zeichen unter 256: ROM-Schrift)

; ---- Zustand der Gitterzellen im Schattengitter CELL (ein Byte je Zelle)
.const CS_NONE  = 0                             ; Innenraum (nichts)
.const CS_LINE  = 1                             ; Linienzelle, unberuehrt (blau)
.const CS_DONE1 = 2                             ; abgefahren von Spieler 1 (gelb)
.const CS_DONE2 = 3                             ; abgefahren von Spieler 2 (cyan)
.const CS_FLASH = 4                             ; zerstoert, blitzt kurz weiss auf
                                                ; ab 16: gefuellt, der Wert ist die Kachelnummer (T_FILL + 5 * Farbe + Zeile im Rechteck)

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

; ---- Spieleinstellungen
.ifndef LIVES0
.const LIVES0      = 3
.endif
.const START_SPEED = 5
.const INVULN      = 12                 ; Spielschritte Unverwundbarkeit nach einem Treffer
.const KIND_RUNNER = 0
.const KIND_WANDER = 1
.const KIND_GUARD  = 2

; ---- FLAGS
.const F_PAL    = $01                   ; Bit 0: PAL erkannt (0 = NTSC)

; ---- Zeropage (siehe Plan oben)
.const STATE    = $02
.const FLAGS    = $03
.const FRAME    = $04                   ; Bilder seit dem letzten Spielschritt
.const STEPLEN  = $05                   ; Bilder je Spielschritt (aus der Tempotabelle)
.const RND      = $06                   ; Zustand des Zufallsgenerators (nie 0)
.const CUR      = $07                   ; Spieler am Zug (0 oder 1; bei einem Spieler und beim gleichzeitigen Spiel 0)
.const NFIG     = $08                   ; Anzahl der Figuren (1; beim gleichzeitigen Spiel 2)
.const WHO      = $09                   ; Spieler, dem Punkte, Farbe und Leben gehoeren (0 oder 1)
.const SPEED    = $0a                   ; Tempo 1..9
.const LEVEL    = $0b                   ; Stufe, BCD
.const BOXES_DONE = $0c                 ; gefuellte Rechtecke
.const FILLS_NOW  = $0d                 ; in diesem Spielschritt abgeschlossene Rechtecke (0..2)
.const NENEMY   = $0e
.const ELIM     = $0f                   ; Ende der Gegnerliste (Gegnerzahl * E_SIZE)
.const HITF     = $10                   ; Maske der in diesem Spielschritt getroffenen Figuren (Bit n = Figur n)
.const STEPCNT  = $11                   ; Spielschritte (Takt der Streuner)
.const EPH      = $12                   ; Bild der Gegner
.const ECUR     = $13                   ; laufender Gegner (Abstand in EN)
.const FIG      = $14                   ; laufende Figur
.const PX       = $15                   ; Figuren: je zwei Byte (Figur 0, 1)
.const PY       = $17
.const PDIR     = $19                   ; letzte Richtung (Dir)
.const PWANT    = $1b                   ; gehaltene Richtungen (Maske IN_*)
.const PANIM    = $1d                   ; Bild 0 oder 1
.const PINV     = $1f                   ; Rest der Unverwundbarkeit in Spielschritten
.const POX      = $21                   ; Zelle vor dem letzten Schritt (das Sprite laeuft von dort zur Zelle PX, PY)
.const POY      = $23
.const PACT     = $25                   ; 1 = Figur im Spiel
.const OWN      = $27                   ; Spieler (0 oder 1), dem die Figur gehoert
.const SCORE    = $29                   ; Punkte, BCD, 3 Byte + Ueberlaufbyte je Spieler (Spieler 2 ab SCORE + 4)
.const LIVES    = $31                   ; Leben je Spieler (zwei Byte)
.const HISCORE  = $33                   ; beste Punkte (Bestenliste oder Sitzung), BCD, 3 Byte
.const EN       = $36                   ; Gegner: 5 * 6 Byte
.const E_X      = 0                     ; Feldabstaende je Gegner
.const E_Y      = 1
.const E_DIR    = 2
.const E_KIND   = 3                     ; 0 Laeufer, 1 Streuner, 2 Waechter
.const E_OX     = 4                     ; Zelle vor dem letzten Schritt (fuer die Sprite-Bewegung)
.const E_OY     = 5
.const E_SIZE   = 6
.const EBEST    = $54
.const EDIR     = $55
.const EDIST    = $56
.const EQX      = $57
.const EQY      = $58
.const ATMP     = $59
.const ETX      = $5a                   ; Ziel der Gegner (Zelle der naechsten Figur)
.const ETY      = $5b
.const T0       = $5d                   ; Zwischenwerte T0..T7
.const T1       = $5e
.const T2       = $5f
.const T3       = $60
.const T4       = $61
.const T5       = $62
.const T6       = $63
.const T7       = $64
.const PTR_A    = $65                   ; Zeiger auf Satztexte (hud.print)
.const PTR_B    = $67                   ; Zeiger in das Schattengitter
.const SDX      = $6b                   ; Segment-Durchlauf (grid.seg_first/seg_next/seg_destroy): laufende Zelle, Rest, Zaehler, Modus
.const SDY      = $6c
.const SDN      = $6d
.const SDM      = $6e
.const SDP      = $6f                   ; abgefahrene Zeichen von Spieler 1
.const SDQ      = $70                   ; abgefahrene Zeichen von Spieler 2
.const NB0      = $71                   ; angrenzende Rechtecke eines Segments ($FF = keins)
.const NB1      = $72
.const RT0      = $73                   ; Hilfswerte der Anzeigeroutinen (render, hud, sprites)
.const RT1      = $74
.const RT2      = $75
.const RT3      = $76
.const RT4      = $77
.const RT5      = $78
.const RT6      = $79
.const RT7      = $7a
.const FILLQ    = $7b                   ; Fuellwarteschlange: +0, +1 Rechtecke, +2 Zeile, +3 Bildzaehler
.const SPPROG   = $81                   ; Fortschritt des Schritts in Punkten 0..7
.const SPN      = $82
.const SPMASK   = $83
.const SPX8     = $84
.const SPX9     = $85
.const SPY8     = $86
.const SPHIT    = $87                   ; 1 = Figuren als Treffer-Bild zeigen
.const SPOFF    = $88                   ; 1 = Sprites verbergen
.const NODEX    = $89                   ; Knoten-Durchlauf (grid.nodes_free)
.const NODEY    = $8a
.const sq       = $a0                   ; 32-Bit-Zeiger auf die Bildschirmzelle ($0001D000 + ...), benutzt mit [sq],z
.const cq       = $a4                   ; 32-Bit-Zeiger auf das Farb-RAM ($0FF80000 + ...)
.const rp       = $e0                   ; 32-Bit-Zeiger (Zeichen-ROM, Logo)

; ---- Spielvariablen im RAM ($CE00 ...)
.const BOXCNT    = $ce00                ; je Rechteck abgeschlossene Kanten (98 Byte)
.const JOYMODE   = $ce70                ; 0 ein Joystick Anschluss 2, 1 ein Joystick Anschluss 1, 2 zwei Joysticks
.const NPLAYERS  = $ce71                ; 1 oder 2
.const PLAYMODE  = $ce72                ; bei zwei Spielern: 0 abwechselnd, 1 gleichzeitig
.const SELROW    = $ce73                ; Titelbild: gewaehlte Zeile
.const RESTORE   = $ce74                ; 1 = LEVELINIT stellt das gesicherte Gitter wieder her
.const PFRESH    = $ce75                ; 2 Byte: 1 = Spieler hat noch kein Gitter gespielt
.const FPS       = $ce77                ; Bilder je Sekunde (50 oder 60)
.const DLYCNT    = $ce78                ; Zaehler von delay
.const IT_KIND   = $ce79                ; Gegenstand auf dem Feld: 0 keiner, 1 Stern, 2 Eisblock
.const IT_X      = $ce7a
.const IT_Y      = $ce7b
.const IT_TIME   = $ce7c                ; 2 Byte: Rest der Sichtzeit in Bildern
.const IT_WAIT   = $ce7e                ; 2 Byte: Wartezeit bis zum naechsten Gegenstand
.const FREEZE    = $ce80                ; 2 Byte: Rest der Gegner-Starre in Bildern
.const FXC       = $ce82                ; Zaehler fuer den Farbzyklus (Titelbild)
.const TITLE_SPEED = $ce83              ; gewaehltes Tempo
.const TD_BASE   = $ce84                ; TESTDEMO: +0,+1 Bilder, +2 Spiele, +3 hoechste Stufe, +4 Treffer, +5 Rechtecke gesamt (nur Low-Byte, laeuft nach 256 ueber)
.const SND       = $cec0                ; Klangmodul (siehe sound.asm), 96 Byte
.const FILLBUF   = $cf70                ; 8 Byte: vier Zeichennummern der Fuellzeile fuer den DMA-Auftrag
.const DMA_JOB   = $cfc0                ; Auftrag im Format F018A (11 Byte)
.const DMA_CMD   = DMA_JOB
.const DMA_LEN   = DMA_JOB + 1
.const DMA_SRC   = DMA_JOB + 3
.const DMA_SRCB  = DMA_JOB + 5
.const DMA_DST   = DMA_JOB + 6
.const DMA_DSTB  = DMA_JOB + 8
.const DMA_MOD   = DMA_JOB + 9
.const SCR_IDX   = $ce90                ; Testbetrieb SCRIPT: Eintrag, Rest der Dauer, Masken der Anschluesse 1 und 2
.const SCR_REM   = $ce91
.const SCR_M1    = $ce92
.const SCR_M2    = $ce93

.enum St
    title, levelinit, playing, fill, hit, levelclear, gameover
.endenum

; ---------------------------------------------------------------------------------------------------------------------
; Programm
; ---------------------------------------------------------------------------------------------------------------------
.segment "PROGRAM", $2001, $b7ff
.basicstub start                        ; "10 SYS <start>"

; start: Einstieg ueber SYS.  Interrupts bleiben fuer immer aus (Abfrage der Eingabe und Bildtakt ohne Interrupt).
start:  lda $01                         ; Zustand des BASIC-65-Starts merken (die Bestenliste blendet das KERNAL spaeter wieder ein, siehe hiscore.asm)
.if EXTRA_HISCORE
        sta hiscore.hs_k01
        lda $d030
        sta hiscore.hs_kd030
.endif
        lda #0
        tax
.zv:    sta BOXCNT,x                    ; Variablen loeschen
        sta $ce70,x
        inx
        cpx #$80
        bne .zv
        sta FLAGS
        sta HISCORE
        sta HISCORE + 1
        sta HISCORE + 2
.if EXTRA_HISCORE
        jsr hiscore.load                ; Bestenliste laden, solange das KERNAL noch sichtbar ist; bei Fehlern bleibt die Standardliste
.endif
        sei
        cld
        ldx #$ff
        txs
        jsr vic.init                    ; MEGA65 einrichten (VIC-IV, Palette, Kacheln, Sprites), Norm erkennen
        lda #50
        ldx FLAGS
        txa
        and #F_PAL
        bne .fps
        lda #60
        bne .fs
.fps:   lda #50
.fs:    sta FPS
        lda #0
        sta JOYMODE                     ; Standard: ein Spieler, ein Joystick an Anschluss 2
        sta SELROW
        sta RESTORE
        sta PLAYMODE
        lda #1
        sta NPLAYERS
        lda #START_SPEED
        sta TITLE_SPEED
        sta SPEED
        jsr sound.init
        jsr input.setup
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

; tick: ein Bild: auf den Bildanfang warten, Sprites setzen, Klang weiterschalten.  Veraendert A, X, Y.
tick:   jsr vic.wait_frame
        jsr sprites.update
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

; fire_wait: wartet, bis der Zustand der Feuertaste X entspricht (0 = losgelassen, IN_FIRE = gedrueckt); gilt der Joystick der Figur FIG.  Veraendert A, Y, T7.
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
        stx RT6
.w:     jsr tick
        jsr input.read_any
        and #IN_FIRE
        cmp RT6
        bne .w
        rts

; TITLE: Titelbild mit Spielerzahl, Ablauf, Joystickwahl und Tempo, Bestenliste, Titelmusik; Feuer startet ein neues Spiel
st_title:
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
        cmp #3
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
        lda #AUTO_SIMUL
        sta PLAYMODE
        beq .j
        lda #2
        sta JOYMODE
.j:
.endif
        jsr new_game
        lda #St.levelinit
        sta STATE
        rts

; new_game: Spieler fuer ein neues Spiel setzen (Leben, Punkte, Stufe 1, Tempo wie gewaehlt; bei einem Spieler hat Spieler 2 keine Leben).  Veraendert A, X, Y.
new_game:
        lda #0
        sta CUR
        sta RESTORE
        lda #1
        sta PFRESH
        sta PFRESH + 1
        lda #0
        ldx #7
.z:     sta SCORE,x
        dex
        bpl .z
        lda #LIVES0
        sta LIVES
        lda #0
        sta LIVES + 1
        lda NPLAYERS
        cmp #2
        bne .one
        lda #LIVES0
        sta LIVES + 1
.one:   lda #1
        sta LEVEL
.ifdef TESTSCORE
        lda #TESTSCORE                  ; Testbetrieb: Startpunkte (BCD, Tausender/Hunderter), damit die Bestenliste erreichbar ist
        sta SCORE + 1
.endif
        lda TITLE_SPEED
        sta SPEED
        lda #0
        sta BOXES_DONE
        lda NPLAYERS
        cmp #2
        bne .nf
        lda PLAYMODE
        beq .nf
        lda #2
        sta NFIG
        lda #2
        sta JOYMODE                     ; gleichzeitig: zwei Joysticks
        rts
.nf:    lda #1
        sta NFIG
        rts

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
        lda #0
        sta PLAYMODE                    ; mit einem Spieler gibt es nur "abwechselnd"
        lda JOYMODE
        cmp #2
        bcc .out
        lda #0                          ; ... und keine zwei Joysticks
        sta JOYMODE
.endif
.out:   rts
.notp:  dex
        bne .notj
        lda NPLAYERS                    ; Zeile 1: Ablauf abwechselnd / gleichzeitig (nur bei zwei Spielern)
        cmp #2
        bne .out
        lda PLAYMODE
        eor #1
        sta PLAYMODE
        beq .out
        lda #2
        sta JOYMODE                     ; gleichzeitig verlangt zwei Joysticks
        rts
.notj:  dex
        bne .speed
        lda PLAYMODE                    ; Zeile 2: Joystick 0 = Anschluss 2, 1 = Anschluss 1, 2 = zwei Joysticks
        bne .out                        ; gleichzeitig: fest zwei Joysticks
        lda JOYMODE
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
.speed: lda TITLE_SPEED                 ; Zeile 3: Tempo 1..9
        clc
        adc T6
        beq .out
        cmp #10
        bcs .out
        sta TITLE_SPEED
        rts

; LEVELINIT: Gitter und Farben neu (oder das gesicherte Gitter), Figuren auf die Startknoten, kurzer Hinweis, dann Spiel
st_levelinit:
        lda #0
        sta SPOFF
        sta SPHIT
        jsr vic.clear
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
        jsr board_load                  ; Gitter des Spielers wiederherstellen (BOXCNT, LEVEL, SPEED, BOXES_DONE gehoeren dazu)
        jsr grid.redraw
.endif
.go:    lda #0
        sta FILLS_NOW
        sta STEPCNT
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
.ifdef SC_ANY
        jsr scenario                    ; nur Testfassungen: festes Szenario fuer Bildschirmaufnahmen
.endif
.if EXTRA_ITEMS
        jsr items.reset
.endif
        jsr hud.update
        ldx #HT_READY
        lda NPLAYERS
        cmp #2
        bne .msg
        lda NFIG
        cmp #2
        beq .msg
        ldx #HT_PLAYER                  ; "spieler n bereit": Ziffer eintragen
.msg:   jsr hud.msg
        lda NPLAYERS
        cmp #2
        bne .nopl
        lda NFIG
        cmp #2
        beq .nopl
        lda CUR
        clc
        adc #49
        sta T6
        lda #3
        sta T5
        lda #COL_WHITE
        sta T4
        lda CUR
        clc
        adc #49
        ldx #40
        ldy #49
        jsr vic.put_char
.nopl:  ldx #75
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
        jsr hud.clrmsg
        lda #0
        sta FRAME
        lda #St.playing
        sta STATE
        rts

; PLAYING: alle STEPLEN Bilder ein Spielschritt
st_playing:
        ldx #0
        stx FIG
.rd:
.ifndef AUTOPLAY
        ldx FIG
        lda PACT,x
        beq .nr
        jsr input.read_joy
        ldx FIG
        sta PWANT,x
        and #IN_FIRE
        bne .pause
.nr:    inc FIG
        lda FIG
        cmp NFIG
        bcc .rd
        jmp .run
.pause: jsr sound.silence               ; Pause: Feuer, nochmal Feuer = weiter
        ldx #HT_PAUSE
        jsr hud.msg
        ldx #0
        jsr fire_wait
        ldx #IN_FIRE
        jsr fire_wait
        ldx #0
        jsr fire_wait
        jsr hud.clrmsg
        lda #0
        sta FIG
        beq st_playing
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
        inc STEPCNT
        ldx #0
        stx FIG
.pl:    ldx FIG
        lda PACT,x
        beq .np
        jsr player.step
.np:    inc FIG
        lda FIG
        cmp NFIG
        bcc .pl
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

; FILL: vorgemerkte Rechtecke fuellen, je 3 Bilder eine Zeile (alle vorgemerkten Rechtecke zugleich)
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
        cmp #BOX_H + 1                  ; Schritte 0..4 fuellen eine Zeile, Schritt 5 schliesst die letzte ab
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

; HIT: Treffer anzeigen, Leben - 1 der getroffenen Figuren, Figuren zurueck auf die Startknoten (Farben und Rechtecke bleiben); bei zwei Spielern
;   abwechselnd wechselt der Spieler.
st_hit:
        lda #1
        sta SPHIT                       ; Figuren als Treffer-Bild
        ldx #HT_HIT
        jsr hud.msg
        lda #Fx.hit
        jsr sound.play
        ldx #75
        jsr delay
        lda #0
        sta SPHIT
        jsr hud.clrmsg                  ; Meldung "TREFFER" wegnehmen
.ifdef TESTDEMO
        inc TD_BASE + 4
.endif
        ldx #0                          ; je getroffener Figur ein Leben des Besitzers abziehen
.f:     lda BITMASK,x
        and HITF
        beq .nf
        ldy OWN,x
        lda LIVES,y
        beq .nf
        sec
        sbc #1
        sta LIVES,y
        bne .nf
        lda #0
        sta PACT,x                      ; letztes Leben verbraucht: Figur scheidet aus
.nf:    inx
        cpx NFIG
        bcc .f
        lda #0
        sta HITF
        jsr player.respawn
        jsr enemies.place_all
.if EXTRA_ITEMS
        jsr items.reset
.endif
        jsr hud.update
        lda #0
        sta FRAME
        lda NFIG
        cmp #2
        beq .two
        ldx CUR                         ; ein Spieler am Zug
        lda LIVES,x
        bne .alive
.if EXTRA_2P
        jsr switch_player               ; keine Leben mehr: anderer Spieler (C = 1: gewechselt, STATE gesetzt)
        bcs .done
.endif
        lda #St.gameover
        bne .chg
.alive: ldx BOXES_DONE
        cpx #NBOX
        bne .turn
        lda #St.levelclear              ; die Stufe war im selben Zug geschafft
        bne .chg
.turn:
.if EXTRA_2P
        jsr switch_player               ; zwei Spieler abwechselnd: nach jedem Treffer ist der andere dran
        bcs .done
.endif
        lda #St.playing
        bne .chg
.two:   lda PACT                        ; gleichzeitig: weiter, solange eine Figur im Spiel ist
        ora PACT + 1
        beq .over
        ldx BOXES_DONE
        cpx #NBOX
        bne .cont
        lda #St.levelclear
        bne .chg
.cont:  lda #St.playing
        bne .chg
.over:  lda #St.gameover
.chg:   sta STATE
.done:  rts

.if EXTRA_2P
; switch_player: zum anderen Spieler wechseln, wenn zwei Spieler abwechselnd spielen und dieser noch Leben hat.  Sichert das Gitter des bisherigen
;   Spielers, setzt STATE = LEVELINIT und RESTORE.  C = 1: gewechselt.  Veraendert A, X, Y, T0.. und die DMA-Liste.
switch_player:
        lda NPLAYERS
        cmp #2
        bne .no
        lda CUR
        eor #1
        tax
        lda LIVES,x
        beq .no
        jsr board_save
        lda CUR
        eor #1
        sta CUR
        tax
        lda PFRESH,x
        eor #1
        sta RESTORE                     ; 1, wenn der Spieler schon ein Gitter hat
        lda #St.levelinit
        sta STATE
        sec
        rts
.no:    clc
        rts

; board_save: Gitter des Spielers CUR (CELL, BOXCNT, SPEED, LEVEL, BOXES_DONE) nach BSAVE0 / BSAVE1 sichern.  Veraendert A, X, DMA-Liste.
board_save:
        jsr bs_ptr
        lda SPEED
        sta T4
        ; CELL
        lda #<CELL_LEN
        ldx #>CELL_LEN
        jsr vic.dma_len
        lda #<CELL
        sta DMA_SRC
        lda #>CELL
        sta DMA_SRC + 1
        lda #0
        sta DMA_SRCB
        lda T6
        sta DMA_DST
        lda T7
        sta DMA_DST + 1
        lda #0
        sta DMA_DSTB
        jsr vic.dma_copy
        ; BOXCNT
        lda #NBOX
        ldx #0
        jsr vic.dma_len
        lda #<BOXCNT
        sta DMA_SRC
        lda #>BOXCNT
        sta DMA_SRC + 1
        lda T6
        clc
        adc #<CELL_LEN
        sta DMA_DST
        lda T7
        adc #>CELL_LEN
        sta DMA_DST + 1
        jsr vic.dma_copy
        ; SPEED, LEVEL, BOXES_DONE
        lda T6
        clc
        adc #<(CELL_LEN + NBOX)
        sta T6
        lda T7
        adc #>(CELL_LEN + NBOX)
        sta T7
        ldy #0
        lda SPEED
        sta (T6),y
        iny
        lda LEVEL
        sta (T6),y
        iny
        lda BOXES_DONE
        sta (T6),y
        rts

; board_load: Gegenstueck zu board_save: Gitter des Spielers CUR wiederherstellen.  Veraendert A, X, Y, DMA-Liste.
board_load:
        jsr bs_ptr
        lda #<CELL_LEN
        ldx #>CELL_LEN
        jsr vic.dma_len
        lda T6
        sta DMA_SRC
        lda T7
        sta DMA_SRC + 1
        lda #0
        sta DMA_SRCB
        lda #<CELL
        sta DMA_DST
        lda #>CELL
        sta DMA_DST + 1
        lda #0
        sta DMA_DSTB
        jsr vic.dma_copy
        lda #NBOX
        ldx #0
        jsr vic.dma_len
        lda T6
        clc
        adc #<CELL_LEN
        sta DMA_SRC
        lda T7
        adc #>CELL_LEN
        sta DMA_SRC + 1
        lda #<BOXCNT
        sta DMA_DST
        lda #>BOXCNT
        sta DMA_DST + 1
        jsr vic.dma_copy
        lda T6
        clc
        adc #<(CELL_LEN + NBOX)
        sta T6
        lda T7
        adc #>(CELL_LEN + NBOX)
        sta T7
        ldy #0
        lda (T6),y
        sta SPEED
        iny
        lda (T6),y
        sta LEVEL
        iny
        lda (T6),y
        sta BOXES_DONE
        rts

; bs_ptr: T6/T7 = Anfang des Speicherbereichs fuer das Gitter des Spielers CUR.  Veraendert A.
bs_ptr: lda CUR
        bne .p1
        lda #<BSAVE0
        sta T6
        lda #>BSAVE0
        sta T7
        rts
.p1:    lda #<BSAVE1
        sta T6
        lda #>BSAVE1
        sta T7
        rts
.endif

; LEVELCLEAR: Bonus 500 fuer jeden Spieler im Spiel, naechste Stufe; alle 2 Stufen Tempo + 1 (bis 9)
st_levelclear:
        ldx #HT_CLEAR
        jsr hud.msg
        lda #Fx.clear
        jsr sound.play
        ldx #0
.b:     lda PACT,x
        beq .nb
        stx T0
        lda OWN,x
        sta WHO
        lda #5
        ldx #1
        jsr hud.bcd_add                 ; 500 Punkte
        ldx T0
.nb:    inx
        cpx NFIG
        bcc .b
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
        lda #0
        sta RT6                         ; Spieler 0, 1
.pl:    ldx RT6
        lda SC_OFF,x
        tax
        lda SCORE,x
        sta T0
        lda SCORE + 1,x
        sta T1
        lda SCORE + 2,x
        sta T2
        lda LEVEL
        sta T3
        lda RT6
        jsr score_done
        inc RT6
        lda RT6
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

; ---- Testszenarien (nur Testfassungen, nicht im Standardbau): -D SCENARIO_DESTROY=1 oder -D SCENARIO_NODE=1 (zusammen mit AUTOPLAY, AUTOGOD, HOLD_DESTROY=n).
;   Nach dem Start von Stufe 1 sind abgefahrene Segmente vorbelegt und ein einziger Gegner steht neben einem Knoten, den er im ersten Schritt betritt:
;   DESTROY: waagerechtes Segment (20..25, 18) gelb, senkrechte Segmente (25, 12..18) gelb und (25, 18..24) cyan; der Gegner betritt den Knoten (25, 18)
;     und zerstoert alle drei nacheinander (Aufblitzen je Segment HOLD_DESTROY Bilder).
;   NODE: waagerechtes Segment (20..25, 18) wird zerstoert; vorher gelb: frei stehende Knoten (10, 12) und (40, 24) (cyan) und ein Segment (50..55, 12)
;     mit Endknoten.  Nach der Zerstoerung sind die frei stehenden Knoten und die Knoten (20, 18), (25, 18) blau; das Segment (50..55, 12) bleibt gelb.
.ifdef SC_ANY
; Laeufe: x, y, dx, dy, Laenge, Zustand ($FF beendet)
.ifdef SCENARIO_DESTROY
SC_RUNS:  .byte 20, 18, 1, 0, 6, CS_DONE1
          .byte 25, 12, 0, 1, 7, CS_DONE1
          .byte 25, 18, 0, 1, 7, CS_DONE2
          .byte $ff
SC_BOXES: .byte 32, 2, 33, 1, 46, 2, 47, 1, $ff
.endif
.ifdef SCENARIO_NODE
SC_RUNS:  .byte 20, 18, 1, 0, 6, CS_DONE1
          .byte 10, 12, 1, 0, 1, CS_DONE1
          .byte 40, 24, 1, 0, 1, CS_DONE2
          .byte 50, 12, 1, 0, 6, CS_DONE1
          .byte $ff
SC_BOXES: .byte 32, 1, 46, 1, 24, 1, 38, 1, $ff
.endif

; scenario: Stufe 1: Laeufe in das Schattengitter schreiben, BOXCNT setzen, einen Gegner auf (26, 18) nach links, Gitter neu zeichnen.  Veraendert A, X, Y, T0..T5, PTR_B.
scenario:
        lda LEVEL
        cmp #1
        bne .out
        ldx #0
.run:   lda SC_RUNS,x
        cmp #$ff
        beq .boxes
        sta T0                          ; x
        lda SC_RUNS + 1,x
        sta T1                          ; y
        lda SC_RUNS + 2,x
        sta T2
        lda SC_RUNS + 3,x
        sta T3
        lda SC_RUNS + 4,x
        sta T4                          ; Laenge
        lda SC_RUNS + 5,x
        sta T5                          ; Zustand
        txa
        pha
.cell:  ldy T1
        jsr grid.set_row
        ldy T0
        lda T5
        sta (PTR_B),y
        lda T0
        clc
        adc T2
        sta T0
        lda T1
        clc
        adc T3
        sta T1
        dec T4
        bne .cell
        pla
        clc
        adc #6
        tax
        bne .run
.boxes: ldx #0
.bx:    ldy SC_BOXES,x
        cpy #$ff
        beq .enemy
        lda SC_BOXES + 1,x
        sta BOXCNT,y
        inx
        inx
        bne .bx
.enemy: lda #1
        sta NENEMY
        lda #E_SIZE
        sta ELIM
        lda #26
        sta EN + E_X
        sta EN + E_OX
        lda #18
        sta EN + E_Y
        sta EN + E_OY
        lda #2                          ; links
        sta EN + E_DIR
        jsr grid.redraw
.out:   rts
.endif

.ifdef TESTDEMO
; testdemo: Zaehler fuer den Dauerlauf: Bilder (16 Bit), hoechste Stufe; Anzeige in Zeile 49 (hexadezimal).  Veraendert A, X, Y.
testdemo:
        inc TD_BASE
        bne .a
        inc TD_BASE + 1
.a:     lda LEVEL
        cmp TD_BASE + 3
        bcc .b
        sta TD_BASE + 3
.b:     rts
.endif

; ---------------------------------------------------------------------------------------------------------------------
; Titelbild
; ---------------------------------------------------------------------------------------------------------------------
; title_draw: Bildschirm loeschen, Logo "TROXX" (Zeichen-ROM zweifach vergroessert, Farbe aus dem Regenbogen), Menue, Bestenliste.  Veraendert alles.
title_draw:
        lda #1
        sta SPOFF
        jsr vic.clear
        lda #0
        sta FXC
        ; Logo: Buchstaben T R O X X, je 16 x 16 Zellen (Bit des Zeichens = 2 x 2 Zellen), links oben bei Spalte 8 + 13 * n, Zeile 3
        lda #0
        sta RT0                         ; Buchstabe 0..4
.let:   ldx RT0
        lda LOGO_CODES,x
        sta RT1
        lda #0
        sta rp
        sta rp + 3
        lda #$d0
        sta rp + 1
        lda #$02
        sta rp + 2                      ; Zeichen-ROM
        lda RT1
        asl
        asl
        asl
        sta RT2                         ; Zeichen * 8 (Zeichen 0..31)
        lda #0
        sta RT3                         ; Zeile 0..7
.row:   lda RT2
        clc
        adc RT3
        taz
        lda [rp],z
        sta RT4                         ; Bitmuster
        ldz #0
        lda #0
        sta RT5                         ; Spalte 0..7
.bit:   asl RT4
        bcc .nb
        lda RT5                         ; Zielzelle: Spalte 8 + 13 * Buchstabe + 2 * Spalte, Zeile 3 + 2 * Zeile
        asl
        sta T0
        lda RT0
        asl
        clc
        adc RT0
        asl
        asl
        clc
        adc RT0                         ; 13 * Buchstabe = 12 * Buchstabe + Buchstabe: (3 * b) * 4 + b
        clc
        adc #8
        clc
        adc T0
        sta T1                          ; Spalte der Zelle
        lda RT3
        asl
        clc
        adc #3
        sta T2
        jsr logo_block
        lda RT2
        and #$f8
        sta RT2
.nb:    inc RT5
        lda RT5
        cmp #8
        bne .bit
        inc RT3
        lda RT3
        cmp #8
        bne .row
        inc RT0
        lda RT0
        cmp #5
        bne .let
        ldx #HT_TITLE_A
        jsr hud.print
        jmp title_menu_static

; logo_block: 2 x 2 Zellen mit der Kachel T_SOLID an Spalte T1, Zeile T2; Farbe (Regenbogen) nach Spalte und Zeile.  Veraendert A, X, Y, Z, T4, RT2.
logo_block:
        lda RT2
        pha
        lda #0
        sta T6                          ; Zeilenhaelfte
.lr:    lda #0
        sta T7                          ; Spaltenhaelfte
.lc:    lda T1
        clc
        adc T7
        tax
        lda T2
        clc
        adc T6
        tay
        sec
        sbc #3
        sta T3
        txa
        clc
        adc T3
        and #63
        clc
        adc #PAL_RAINBOW
        sta T4
        lda #T_SOLID
        jsr vic.put_tile_c
        inc T7
        lda T7
        cmp #2
        bne .lc
        inc T6
        lda T6
        cmp #2
        bne .lr
        pla
        sta RT2
        rts

LOGO_CODES: .byte 20, 18, 15, 24, 24    ; Bildschirmcodes T R O X X

; title_menu_static: feste Beschriftungen des Menues.  Veraendert A, X, Y und mehr.
title_menu_static:
        ldx #HT_M_PLAYERS
        jsr hud.print
        ldx #HT_M_MODE
        jsr hud.print
        ldx #HT_M_JOY
        jsr hud.print
        ldx #HT_M_SPEED
        jsr hud.print
        ldx #HT_M_FIRE
        jsr hud.print
        ldx #HT_M_BEST
        jmp hud.print

; title_values: veraenderliche Teile des Menues (Werte, Auswahlmarke, Bestpunkte).  Veraendert A, X, Y und mehr.
title_values:
        ; Auswahlmarke ">" links der Zeile SELROW (Zeilen 24, 27, 30, 33 mit Abstand 3)
        ldx #0
.mk:    stx RT6
        lda #3
        sta T5
        lda #COL_YELLOW
        sta T4
        lda ROWTAB,x
        tay
        lda #CH_SPACE
        cpx SELROW
        bne .sp
        lda #62                         ; ">" ist Bildschirmcode 62
.sp:    ldx #20
        jsr vic.put_char
        ldx RT6
        inx
        cpx #4
        bne .mk
        ; Werte: erst die Wertbereiche leeren, dann beschreiben
        ldx #0
.cl:    stx RT6
        lda ROWTAB,x
        sta RT5
        ldx #35
.cc:    stx T6
        lda #3
        sta T5
        lda #COL_WHITE
        sta T4
        ldx T6
        ldy RT5
        lda #CH_SPACE
        jsr vic.put_char
        ldx T6
        inx
        cpx #70
        bne .cc
        ldx RT6
        inx
        cpx #4
        bne .cl
        ; Spielerzahl
        ldx #HT_V_1
        lda NPLAYERS
        cmp #2
        bne .v0
        ldx #HT_V_2
.v0:    jsr hud.print
        ; Ablauf
        ldx #HT_V_NONE
        lda NPLAYERS
        cmp #2
        bne .v1
        ldx #HT_V_ALT
        lda PLAYMODE
        beq .v1
        ldx #HT_V_SIM
.v1:    jsr hud.print
        ; Joystick
        ldx #HT_V_J2
        lda JOYMODE
        beq .v2
        ldx #HT_V_J1
        cmp #1
        beq .v2
        ldx #HT_V_JJ
.v2:    jsr hud.print
        ; Tempo (Ziffer)
        lda #3
        sta T5
        lda #COL_WHITE
        sta T4
        lda TITLE_SPEED
        ora #48
        ldx #36
        ldy #ROW_SPEED
        jsr vic.put_char
        ; Bestpunkte
        lda #3
        sta T5
        lda #COL_LTBLUE
        sta T4
        lda #41
        sta T6
        lda #ROW_BEST
        sta T7
        ldx #HISCORE + 2
        jmp hud.bcd3

ROWTAB: .byte 24, 27, 30, 33
.const ROW_SPEED = 33
.const ROW_BEST  = 40

; title_fx: Farbzyklus des Logos: die Regenbogeneintraege der Palette (128..191) alle 2 Bilder um einen Schritt drehen.  Veraendert A, X, Y.
title_fx:
.if FX_TITLE
        inc FXC
        lda FXC
        and #1
        bne .out
        lda FXC
        lsr
        and #63
        sta T0                          ; Phase 0..63
        ldx #0
.p:     txa
        clc
        adc T0
        and #63
        tay
        lda PAL_R + 128,y
        sta $d100 + 128,x
        lda PAL_G + 128,y
        sta $d200 + 128,x
        lda PAL_B + 128,y
        sta $d300 + 128,x
        inx
        cpx #64
        bne .p
.endif
.out:   rts

.include "vic4.asm"
.include "grid.asm"
.include "input.asm"
.include "player.asm"
.include "enemies.asm"
.if EXTRA_ITEMS
.include "items.asm"
.endif
.include "sprites.asm"
.include "sound.asm"
.include "hud.asm"
.if EXTRA_HISCORE
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
.include "assets.inc"





