; =====================================================================================================================
; main.asm - TROXX fuer den VC-20 (VIC-20), unerweitert (3,5 KB): Konstanten, Speicherplan, Start, Zustandsautomat
; =====================================================================================================================
;
; ZWECK
;   Troxx ist ein eigenstaendiges Arcade-Spiel: Eine Figur faehrt die Linien eines festen Gitters (5 x 4 Rechtecke) ab und faerbt sie; sind alle
;   vier Kanten eines Rechtecks abgefahren, wird es gefuellt. Gegner laufen auf denselben Linien und kosten ein Leben. Sind alle 20 Rechtecke
;   gefuellt, beginnt die naechste Stufe. Die Referenzfassung (VC-20, unerweitert) legt die Regeln fest; andere Systeme folgen ihnen.
;
; BEDIENUNG
;   Titelbild: Feuer startet. Spiel: Joystick im Anschluss; die Figur laeuft nur, solange eine Richtung gehalten wird (an Knoten abbiegen, auf
;   Linien nur vor und zurueck). Punkte: 1 je neu gefaerbtes Zeichen, 10 je abgeschlossenes Segment, 100 je gefuelltes Rechteck (zwei Rechtecke in
;   einem Zug: zusammen 400), 500 je Stufe. Extraleben je 10000 Punkte, hoechstens 9 Leben.
;   Gegner zerstoeren Linien: betritt ein Gegner ein Innenzeichen oder einen Endknoten eines Segments, das an kein gefuelltes Rechteck grenzt, wird das ganze Segment
;   zurueckgesetzt (Zeichen blau, Punkte dafuer abgezogen, bei abgeschlossenem Segment BOXCNT der Nachbarn - 1), mit Aufblitzen und eigenem Ton.
;   Test ohne Joystick: -D AUTOPLAY=1 (der Rechner spielt selbst, das Titelbild entfaellt).
;
; BEDIENUNG (E5): Titelbild: Joystick links/rechts waehlt das Tempo 1..9, Feuer startet. Im Spiel Feuer = Pause, nochmal Feuer = weiter.
;
; ABLAUF (Zustandsautomat, ein Aufruf je Bild)
;   TITLE -> LEVELINIT -> PLAYING <-> FILL;  PLAYING -> HIT -> PLAYING / GAMEOVER -> TITLE;  PLAYING/FILL -> LEVELCLEAR -> LEVELINIT
;   Hauptschleife: vic.wait_frame -> sound.update -> Zustandsprozedur (Sprungtabelle STATES, .retaddr).
;   PLAYING: alle STEPLEN Bilder ein Spielschritt: Spieler bewegen und faerben, Gegner bewegen; Zusammenstoss = HIT.
;
; WAS DER CODE AUF DER HARDWARE TUT
;   VIC-I $9000-$900F (Bild, Zeichenbasis, Farben, Ton), VIA 1 $9111 (Joystick hoch/runter/links/Feuer), VIA 2 $9120 (Joystick rechts).
;   Der KERNAL-Interrupt ist aus (SEI am Start, nie CLI); der NMI der RESTORE-Taste zeigt auf einen RTI-Stumpf und die VIA-Interrupts sind gesperrt.
;
; SPEICHERPLAN
;   $0002-$0067  Zeropage (Plan unten; bis $7F verfuegbar, hoeher nicht benutzt)
;   $1001-$1BFF  Programm: BASIC-Zeile, Code, Tabellen (Segment PROGRAM, vom Assembler auf $1BFF begrenzt; hartes Limit 3071 Byte, Messwerte siehe README)
;   $1C00-$1DFF  Zeichensatz-RAM, 64 Zeichen zu 8 Byte (Segment CHARSET; Zeichenbasis des VIC-I)
;   $1E00-$1FF9  Bildschirm 22 x 23      $9600-$97F9  Farbspeicher
;
; SPEICHERPLAN DER ZEROPAGE
;   $02 STATE  $03 FLAGS  $04 FRAME  $05 STEPLEN  $06 SPEED  $07 LEVEL (BCD)  $08 LIVES  $09 RND  $0A-$0C SCORE (BCD)  $0D Ueberlauf  $0E-$10 HISCORE
;   $11-$16 PX PY PDIR PWANT PANIM PINV   $17 BOXES_DONE  $18 FILLS_NOW  $19 NENEMY  $1A-$2D EN (4 Gegner zu 5 Byte: x y dir kind unter)
;   $2E-$34 frei (frueher SEGBITS)  $35-$48 BOXCNT (20 Byte)  $49-$50 T0..T7  $51-$52 PTR_A  $53-$54 PTR_B  $55-$58 SND  $59-$5C FILLQ
;   $5E HITF  $5F EPH  $60 ECUR  $61 ELIM  $62-$67 Gegnerhilfswerte  $68-$69 NODEX NODEY
; =====================================================================================================================
; ---------------------------------------- English ----------------------------------------
; =====================================================================================================================
; main.asm - TROXX for the VC-20 (VIC-20), unexpanded (3.5 KB): constants, memory map, start, state machine
; =====================================================================================================================
;
; PURPOSE
;   Troxx is a self-contained arcade game: a figure travels along the lines of a fixed grid (5 x 4 rectangles) and colors them; when all
;   four edges of a rectangle have been traveled, it is filled. Enemies walk on the same lines and cost a life. When all 20 rectangles
;   are filled, the next level begins. The reference version (VIC-20, unexpanded) defines the rules; other systems follow them.
;
; CONTROLS
;   Title screen: fire starts. Game: joystick in the port; the figure only moves while a direction is held (turn at nodes, on
;   lines only forward and back). Points: 1 per newly colored character, 10 per completed segment, 100 per filled rectangle (two rectangles in
;   one move: 400 together), 500 per level. Extra life per 10000 points, at most 9 lives.
;   Enemies destroy lines: if an enemy enters an inner character or an end node of a segment that borders no filled rectangle, the whole segment is
;   reset (characters blue, points for it deducted, for a completed segment BOXCNT of the neighbors - 1), with a flash and its own tone.
;   Test without joystick: -D AUTOPLAY=1 (the computer plays itself, no title screen).
;
; CONTROLS (E5): Title screen: joystick left/right selects the speed 1..9, fire starts. In the game fire = pause, fire again = resume.
;
; FLOW (state machine, one call per frame)
;   TITLE -> LEVELINIT -> PLAYING <-> FILL;  PLAYING -> HIT -> PLAYING / GAMEOVER -> TITLE;  PLAYING/FILL -> LEVELCLEAR -> LEVELINIT
;   Main loop: vic.wait_frame -> sound.update -> state procedure (jump table STATES, .retaddr).
;   PLAYING: every STEPLEN frames one game step: move and color the player, move the enemies; collision = HIT.
;
; WHAT THE CODE DOES ON THE HARDWARE
;   VIC-I $9000-$900F (screen, character base, colors, sound), VIA 1 $9111 (joystick up/down/left/fire), VIA 2 $9120 (joystick right).
;   The KERNAL interrupt is off (SEI at start, never CLI); the NMI of the RESTORE key points to an RTI stub and the VIA interrupts are disabled.
;
; MEMORY MAP
;   $0002-$0067  Zero page (map below; available up to $7F, not used above)
;   $1001-$1BFF  Program: BASIC line, code, tables (segment PROGRAM, limited by the assembler to $1BFF; hard limit 3071 bytes, measured values see README)
;   $1C00-$1DFF  Character set RAM, 64 characters of 8 bytes (segment CHARSET; character base of the VIC-I)
;   $1E00-$1FF9  Screen 22 x 23      $9600-$97F9  Color memory
;
; ZERO PAGE MAP
;   $02 STATE  $03 FLAGS  $04 FRAME  $05 STEPLEN  $06 SPEED  $07 LEVEL (BCD)  $08 LIVES  $09 RND  $0A-$0C SCORE (BCD)  $0D overflow  $0E-$10 HISCORE
;   $11-$16 PX PY PDIR PWANT PANIM PINV   $17 BOXES_DONE  $18 FILLS_NOW  $19 NENEMY  $1A-$2D EN (4 enemies of 5 bytes: x y dir kind under)
;   $2E-$34 free (formerly SEGBITS)  $35-$48 BOXCNT (20 bytes)  $49-$50 T0..T7  $51-$52 PTR_A  $53-$54 PTR_B  $55-$58 SND  $59-$5C FILLQ
;   $5E HITF  $5F EPH  $60 ECUR  $61 ELIM  $62-$67 enemy helper values  $68-$69 NODEX NODEY
; =====================================================================================================================

#include <vic20/hardware.asm>

; ---- Gitter (Abschnitt 2.1 des Konzepts): 5 x 4 Rechtecke, Innenraum 3 x 4 Zeichen
.const BOX_COLS  = 5
.const BOX_ROWS  = 4
.const BOX_W     = 3
.const BOX_H     = 4
.const GRID_W    = BOX_COLS * (BOX_W + 1) + 1   ; 21 Spalten (gx 0..20); senkrechte Linien bei gx = 0, 4, 8, 12, 16, 20
.const GRID_H    = BOX_ROWS * (BOX_H + 1) + 1   ; 21 Zeilen (gy 0..20); waagerechte Linien bei gy = 0, 5, 10, 15, 20
.const NBOX      = BOX_COLS * BOX_ROWS          ; 20 Rechtecke
.const SCR_COLS  = 22
.const SCR_ROWS  = 23
.const COLOR_DELTA = (>COLOR_RAM) - (>SCREEN_RAM)   ; High-Byte-Abstand Farbspeicher - Bildschirm ($78); die Low-Bytes sind gleich
.const CHARSET   = $1c00

; ---- Zeichencodes (Abschnitt 3 des Konzepts); 1..26 A-Z, 32 Leerzeichen, 45 "-", 48..57 Ziffern, 58 ":" kommen aus dem Zeichen-ROM
.const CH_HLINE   = 0
.const CH_VLINE   = 27
.const CH_CROSS   = 28
.const CH_SPACE   = 32
.const CH_FILL0   = 38                  ; 38..41 vier Fuellmuster
.const CH_PLAYER  = 42                  ; 42, 43 Spieler (zwei Bilder)
.const CH_ENEMY_A = 44                  ; 44, 46 Gegner (zwei Bilder: 44 + 2 * Bild)
.const CH_HEART   = 47
.const CH_FLASH   = 59

; ---- Farben im Farbspeicher
.const COL_LINE   = COL_BLUE            ; unberuehrte Linie
.const COL_DONE   = COL_YELLOW          ; abgefahrene Linie

; ---- Zelltypen (grid.cell_type)
.const CT_NONE = 0
.const CT_H    = 1                      ; Innenzelle eines waagerechten Segments
.const CT_V    = 2                      ; Innenzelle eines senkrechten Segments
.const CT_NODE = 3                      ; Knoten

; ---- Richtungen (Reihenfolge wichtig: Umkehr = Richtung xor 1) und Eingabemaske
.enum Dir
    up, down, left, right
.endenum
.const IN_UP    = $01
.const IN_DOWN  = $02
.const IN_LEFT  = $04
.const IN_FIRE  = $08
.const IN_RIGHT = $10

; ---- FLAGS
.const F_PAL    = $01                   ; Bit 0: PAL erkannt (0 = NTSC)

; ---- Spieleinstellungen
.const LIVES0     = 3
.const START_SPEED = 5
; Spielregel (Konzept 16.2): 1 = Gegner zerstoeren nicht geschlossene Linien (ein Segment, das an kein gefuelltes Rechteck grenzt), 0 = alte Regel
.ifndef ENEMY_DESTROYS_LINES
.const ENEMY_DESTROYS_LINES = 1
.endif
.const INVULN     = 12                  ; Spielschritte Unverwundbarkeit nach einem Treffer

; ---- Zeropage (siehe Plan oben)
.const STATE    = $02
.const FLAGS    = $03
.const FRAME    = $04                   ; Bilder seit dem letzten Spielschritt
.const STEPLEN  = $05                   ; Bilder je Spielschritt (aus der Tempotabelle)
.const SPEED    = $06                   ; Tempo 1..9
.const LEVEL    = $07                   ; Stufe, BCD
.const LIVES    = $08
.const RND      = $09                   ; Zustand des Zufallsgenerators (nie 0)
.const SCORE    = $0a                   ; Punkte, BCD, 3 Byte (niedrigstes zuerst)
.const SCORE_OVF = $0d                  ; Ueberlauf des Punktestands (nur Rechenhilfe)
.const HISCORE  = $0e                   ; beste Punkte der Sitzung, BCD, 3 Byte
.const PX       = $11                   ; Figur: Gitterzelle (gx, gy)
.const PY       = $12
.const PDIR     = $13                   ; letzte Richtung (Dir)
.const PWANT    = $14                   ; gehaltene Richtungen (Maske IN_*)
.const PANIM    = $15                   ; Bild 0 oder 1
.const PINV     = $16                   ; Rest der Unverwundbarkeit in Spielschritten
.const BOXES_DONE = $17                 ; gefuellte Rechtecke
.const FILLS_NOW  = $18                 ; in diesem Spielschritt abgeschlossene Rechtecke (0..2)
.const EN       = $1a                   ; Gegner: 4 * 5 Byte
.const E_X      = 0                     ; Feldabstaende je Gegner
.const E_Y      = 1
.const E_DIR    = 2
.const E_KIND   = 3                     ; 0 Laeufer, 1 Streuner
.const E_UNDER  = 4                     ; Farbe der Zelle unter dem Gegner (Zustand der Linie)
.const E_SIZE   = 5
.const SDX      = $2e                   ; Segment-Durchlauf (grid.seg_first/seg_next/seg_destroy): laufende Zelle, Rest, Zaehler, Modus
.const SDY      = $2f
.const SDN      = $30
.const SDP      = $31                   ; gefaerbte Zeichen des Segments
.const SDM      = $32                   ; Durchgang 0 zaehlen, 1 aufblitzen, 2 zuruecksetzen
.const NB1      = $33                   ; angrenzende Rechtecke eines Segments ($FF = keins): oben/links
.const NB0      = $34                   ;                                                    unten/rechts
.const BOXCNT   = $35                   ; je Rechteck abgeschlossene Kanten
.const T0       = $49                   ; Zwischenwerte T0..T7
.const T1       = $4a
.const T2       = $4b
.const T3       = $4c
.const T4       = $4d
.const T5       = $4e
.const T6       = $4f
.const T7       = $50
.const PTR_A    = $51                   ; Zeiger Bildschirm
.const PTR_B    = $53                   ; Zeiger Farbspeicher
.const SND      = $55                   ; Klangmodul: +0 Lesestand, +1 Zeit, +2 Notenversatz (PAL 0, NTSC 8), +3 Prioritaet (0 = ruhig)
.const FILLQ    = $59                   ; Fuellwarteschlange: +0, +1 Rechtecke, +2 Zeile, +3 Bildzaehler
.const HITF     = $5e                   ; 1 = Zusammenstoss in diesem Spielschritt
.const EPH      = $5f                   ; Bild der Gegner
.const ECUR     = $60                   ; laufender Gegner (Abstand in EN)
.const ELIM     = $61                   ; Ende der Gegnerliste (Gegnerzahl * E_SIZE)
.const EBEST    = $62
.const EDIR     = $63
.const EDIST    = $64
.const EQX      = $65
.const EQY      = $66
.const ATMP     = $67
.const NODEX    = $68                   ; Knoten-Durchlauf (grid.nodes_free)
.const NODEY    = $69
.assert BOXCNT + NBOX <= T0, "Zeropage-Plan"

.enum St
    title, levelinit, playing, fill, hit, levelclear, gameover
.endenum

; ---------------------------------------------------------------------------------------------------------------------
; Programm
; ---------------------------------------------------------------------------------------------------------------------
.segment "PROGRAM", $1001, $1bff
.basicstub start                        ; "10 SYS <start>", Programm beginnt bei $100D

; start: Einstieg ueber SYS.  Interrupts bleiben fuer immer aus (Abfrage der Eingabe und Bildtakt ohne Interrupt).
start:  sei
        cld
        lda #<nmi_stub                  ; NMI der RESTORE-Taste: Vektor auf RTI
        sta $0318
        lda #>nmi_stub
        sta $0319
        lda #$7f                        ; alle VIA-Interrupts sperren (VIA 1: RESTORE-Leitung CA1; VIA 2: Systemtakt)
        sta VIA1_IER
        sta VIA2_IER
        lda #START_SPEED
        sta SPEED
        lda #0
        sta FLAGS
        ldx #2
.hs:    sta HISCORE,x                   ; beste Punkte der Sitzung: 0
        dex
        bpl .hs
        jsr vic.init                    ; Norm erkennen, Bildregister, Zeichensatz, Bildschirm loeschen
        jsr sound.init
        jsr player.setup
        lda #St.title
        sta STATE
.loop:  jsr vic.wait_frame
        jsr sound.update
        jsr dispatch
        jmp .loop

nmi_stub:
        rti

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

; delay: X Bilder warten (Klang laeuft weiter).  Veraendert A, X.
delay:  jsr vic.wait_frame
        jsr sound.update
        dex
        bne delay
        rts

; ---------------------------------------------------------------------------------------------------------------------
; Zustaende
; ---------------------------------------------------------------------------------------------------------------------

; fire_wait: wartet, bis der Zustand der Feuertaste X entspricht (0 = losgelassen, IN_FIRE = gedrueckt).  Veraendert A, T7.
fire_wait:
        stx T7
.w:     jsr vic.wait_frame
        jsr player.read_joy
        and #IN_FIRE
        cmp T7
        bne .w
        rts

; TITLE: Titelbild mit Tempowahl (Joystick links/rechts, 1..9), Feuer startet ein neues Spiel (Punkte 0, Leben 3, Stufe 1)
st_title:
        jsr vic.clear
        ldx #HT_TITLE
        jsr hud.print_all
.ifndef AUTOPLAY
.sel:   lda SPEED
        ora #48
        sta SCREEN_RAM + 11 * SCR_COLS + 12
        jsr vic.wait_frame
        jsr player.read_joy
        sta T7
        and #IN_FIRE
        bne .go
        lda T7
        and #IN_LEFT
        beq .r
        lda SPEED
        cmp #1
        beq .r
        dec SPEED
        bne .d
.r:     lda T7
        and #IN_RIGHT
        beq .sel
        lda SPEED
        cmp #9
        bcs .sel
        inc SPEED
.d:     ldx #12                         ; Wiederholung bremsen
        jsr delay
        jmp .sel
.go:    ldx #0
        jsr fire_wait                   ; Feuer loslassen, sonst gilt es gleich als Pause
.endif
        lda #0
        sta SCORE
        sta SCORE + 1
        sta SCORE + 2
        sta SCORE_OVF
        lda #LIVES0
        sta LIVES
        lda #1
        sta LEVEL
        lda #St.levelinit
        sta STATE
        rts

; LEVELINIT: Gitter und Farben neu, Spielstand der Stufe loeschen, Figuren auf die Startknoten, kurzer Hinweis, dann Spiel
st_levelinit:
        jsr vic.clear
        jsr hud.draw
        jsr grid.draw
        lda #0
        ldx #NBOX - 1
.clr:   sta BOXCNT,x                    ; BOXCNT leeren
        dex
        bpl .clr
        sta BOXES_DONE
        sta FILLS_NOW
        sta HITF
        lda LEVEL
        ora #$80
        sta RND                         ; fester Startwert je Stufe (nie 0)
        jsr set_tempo
        jsr player.init
        jsr enemies.place_all
        jsr hud.update
        ldx #HT_READY
        jsr hud.msg
        ldx #75
        jsr delay
        jsr hud.draw
        lda #0
        sta FRAME
        lda #St.playing
        sta STATE
        rts

; PLAYING: alle STEPLEN Bilder ein Spielschritt
st_playing:
.ifndef AUTOPLAY
        jsr player.read_joy
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

; HIT: Treffer anzeigen, Leben - 1, Figuren zurueck auf die Startknoten (Farben und Rechtecke bleiben)
st_hit:
        lda #0
        sta HITF
        ldx #HT_HIT
        jsr hud.msg
        lda #Fx.hit
        jsr sound.play
        ldx #75
        jsr delay
        dec LIVES
        jsr player.erase
        jsr enemies.erase_all
        jsr player.init
        lda #INVULN
        sta PINV
        jsr enemies.place_all
        jsr hud.draw
        lda #0
        sta FRAME
        lda #St.playing
        ldx LIVES
        beq .over
        ldx BOXES_DONE
        cpx #NBOX
        bne .chg
        lda #St.levelclear              ; die Stufe war im selben Zug geschafft
        bne .chg
.over:  lda #St.gameover
.chg:   sta STATE
        rts

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
        lda #St.levelinit
        sta STATE
        rts

; GAMEOVER: Meldung, beste Punkte der Sitzung merken, zurueck zum Titelbild
st_gameover:
        ldx #HT_OVER
        jsr hud.msg
        lda #Fx.over
        jsr sound.play
        lda SCORE + 2                   ; BCD-Bytes lassen sich wie Binaerzahlen vergleichen
        cmp HISCORE + 2
        bne .cmp
        lda SCORE + 1
        cmp HISCORE + 1
        bne .cmp
        lda SCORE
        cmp HISCORE
.cmp:   bcc .keep
        ldx #2
.cp:    lda SCORE,x
        sta HISCORE,x
        dex
        bpl .cp
.keep:  ldx #150
        jsr delay
        lda #St.title
        sta STATE
        rts

.include "vic.asm"
.include "grid.asm"
.include "player.asm"
.include "enemies.asm"
.include "sound.asm"
.include "hud.asm"
.include "data.inc"

.include "charset.asm"
