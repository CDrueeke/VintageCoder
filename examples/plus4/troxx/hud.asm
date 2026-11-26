; =====================================================================================================================
; hud.asm - Anzeigen: Kopfzeile (Zeile 0), Meldungszeile (Zeile 1), Leben und Tempo (Zeilen 23, 24), Titelbild, BCD-Punkte
; =====================================================================================================================
; Herkunft: examples/vic20/troxx/hud.asm, Stand Commit 4697d074.  Kopien (Pruefsumme, siehe KOPIEN.txt): print_all, bcd2, bcd3, bcd_add.  Angepasst an den
;   40-Spalten-Bildschirm des Plus/4 (Anzeigen oben und unten wie im Mockup): print, clrmsg, msg, draw, update; neu: das Titelbild (title_draw,
;   title_values) wie im C64-Projekt (examples/c64/troxx/hud.asm, Stand d173cb4a).
; ZWECK     Texte stehen als Saetze in hud_texts: Bildschirmadresse (Wort), Farbe (Byte), Zeichen als Bildschirmcodes, abgeschlossen mit 0.
;           Bildschirmcode 0 (das "@") darf in Texten nicht vorkommen.  Quelltext in KLEINBUCHSTABEN: so entstehen die Codes 1..26 (A-Z).
;           Texte bleiben kurz und ohne Umlaute.  Punkte stehen als BCD (drei Byte, niedrigstes zuerst), Stufe auch als BCD.
; BEDIENUNG hud.print mit X = Abstand des Satzes in hud_texts (HT_*); hud.msg gibt eine Meldung in Zeile 1 aus (Spalten 3..26);
;           hud.draw die festen Anzeigen, hud.update die Zahlen (je Bild); hud.bcd_add bucht Punkte.
; ANZEIGE   Zeile 0: SCORE (6 Ziffern), LEVEL, HI (beste Punkte).  Zeile 23: Leben als Herzen (links), PLAYER n (rechts, bei zwei Spielern).
;           Zeile 24: SPEED, BONUS mit dem Gegenstand auf dem Feld (Symbol und Text), PLUS/4.
; =====================================================================================================================

.const MSG_ROW  = 1
.const MSG_COL  = GRID_COL0
.const ROW_TOP  = 0
.const ROW_LIVES = 23
.const ROW_BOT  = 24
.const X_SCORE = 9                    ; Zahlen: Punkte ab Spalte 9, Stufe ab 23, beste Punkte ab 31, Tempo in Spalte 9 (Zeile 24), Spieler in Spalte 36
.const X_LEVEL = 23
.const X_HI    = 31
.const X_ITEM  = 20                   ; Gegenstand: Symbol in Spalte 20, Text ab Spalte 22 (acht Zeichen)

.macro print_from(tbl)
        lda tbl,x
        sta PTR_A
        sta PTR_B                       ; Bildschirm und Farbspeicher haben dasselbe Low-Byte
        lda tbl + 1,x
        sta PTR_A + 1
        clc
        adc #COLOR_DELTA
        sta PTR_B + 1
        lda tbl + 2,x
        sta T2
        inx
        inx
        inx
        ldy #0
@chr:   lda tbl,x
        beq @done
        sta (PTR_A),y
        lda T2
        sta (PTR_B),y
        inx
        iny
        bne @chr
@done:  rts
.endmacro

hud: {

; print: Satz X (Abstand in hud_texts) ausgeben.  Veraendert A, X, Y, T2, PTR_A, PTR_B.
print:  print_from hud_texts

; mprint: Satz X (Abstand in msg_texts) ausgeben.  Veraendert wie print.
mprint: print_from msg_texts

; clrmsg: Spalten 3..26 der Meldungszeile leeren.  Veraendert A, X.
clrmsg: ldx #23
        lda #CH_SPACE
.clr:   sta SCREEN_RAM + MSG_ROW * SCR_COLS + MSG_COL,x
        dex
        bpl .clr
        rts

; msg: Meldung X (Satz in msg_texts) in der Meldungszeile ausgeben (vorher leeren).  Veraendert A, X, Y, T2, PTR_A, PTR_B.
msg:    txa
        pha
        jsr clrmsg
        pla
        tax
        jmp mprint

; draw: feste Anzeigen (Beschriftungen); loescht eine Meldung.  Veraendert A, X, Y, T2, PTR_A, PTR_B.
draw:   jsr clrmsg
        ldx #HT_STATIC
        jsr print_all
        lda NPLAYERS                    ; die Beschriftung "player" nur bei zwei Spielern
        cmp #2
        beq .out
        ldx #5
        lda #CH_SPACE
.np:    sta SCREEN_RAM + ROW_LIVES * SCR_COLS + 29,x
        dex
        bpl .np
.out:   rts

; print_all: alle Saetze ab X bis zur Endemarke $FF ausgeben.  Veraendert A, X, Y, T2, PTR_A, PTR_B.
print_all:
        jsr print
        inx                             ; hinter die 0 des Satzes
        lda hud_texts,x
        cmp #$ff
        bne print_all
        rts

; bcd2: A = BCD-Byte: zwei Ziffern nach (PTR_A),Y; Y wird um 2 groesser.  Veraendert A, Y.
bcd2:   pha
        lsr
        lsr
        lsr
        lsr
        ora #48
        sta (PTR_A),y
        iny
        pla
        and #15
        ora #48
        sta (PTR_A),y
        iny
        rts

; bcd3: drei BCD-Bytes ab Zeropage-Adresse X abwaerts als sechs Ziffern nach (PTR_A),Y.  Veraendert A, X, Y.
bcd3:   lda 0,x
        jsr bcd2
        dex
        lda 0,x
        jsr bcd2
        dex
        lda 0,x
        jmp bcd2

; setp: PTR_A = Bildschirmadresse Zeile A, Spalte X.  Veraendert A, X.
setp:   pha
        txa
        sta PTR_A
        pla
        tax
        lda PTR_A
        clc
        adc ROW_LO,x
        sta PTR_A
        lda ROW_HI,x
        adc #0
        sta PTR_A + 1
        rts

; update: Zahlen (Punkte, beste Punkte, Stufe, Leben, Tempo, Spieler, Gegenstand) ausgeben.  Veraendert A, X, Y, PTR_A.
update: lda #ROW_TOP
        ldx #X_SCORE
        jsr setp
        ldy #0
        ldx #SCORE + 2
        jsr bcd3
        lda #ROW_TOP
        ldx #X_HI
        jsr setp
        ldy #0
        ldx #HISCORE + 2
        jsr bcd3
        lda #ROW_TOP
        ldx #X_LEVEL
        jsr setp
        ldy #0
        lda LEVEL
        jsr bcd2
        ldx #8                          ; Leben als Herzen (neun Plaetze: 9 Leben sind der Hoechstwert)
.h:     cpx LIVES
        bcs .nh
        lda #CH_HEART
        bne .hs
.nh:    lda #CH_SPACE
.hs:    sta SCREEN_RAM + ROW_LIVES * SCR_COLS + GRID_COL0,x
        lda #COL_RED
        sta COLOR_RAM + ROW_LIVES * SCR_COLS + GRID_COL0,x
        dex
        bpl .h
        lda SPEED
        ora #48
        sta SCREEN_RAM + ROW_BOT * SCR_COLS + X_SCORE
        lda #CH_SPACE
        ldx NPLAYERS
        cpx #2
        bne .p1
        lda CUR
        clc
        adc #49
.p1:    sta SCREEN_RAM + ROW_LIVES * SCR_COLS + 36
.if EXTRA_ITEMS
        ldx IT_KIND                     ; Gegenstand auf dem Feld: Symbol und Text neben "bonus"
        lda ITEM_SYM,x
        sta SCREEN_RAM + ROW_BOT * SCR_COLS + X_ITEM
        lda ITEM_SYMCOL,x
        sta COLOR_RAM + ROW_BOT * SCR_COLS + X_ITEM
        txa
        asl
        asl
        asl
        tax
        ldy #0
.it:    lda ITEM_TXT,x
        sta SCREEN_RAM + ROW_BOT * SCR_COLS + X_ITEM + 2,y
        inx
        iny
        cpy #8
        bne .it
.endif
        rts

; bcd_sub: Punkte abziehen.  A = BCD-Betrag (0..99); der Punktestand wird nie kleiner als 0.  Veraendert A, T7.
.if ENEMY_DESTROYS_LINES
bcd_sub:
        sta T7
        sed
        lda SCORE
        sec
        sbc T7
        sta SCORE
        lda SCORE + 1
        sbc #0
        sta SCORE + 1
        lda SCORE + 2
        sbc #0
        sta SCORE + 2
        cld
        bcs .out
        lda #0                          ; unter 0: auf 0 setzen
        sta SCORE
        sta SCORE + 1
        sta SCORE + 2
.out:   rts
.endif

; bcd_add: Punkte buchen.  A = BCD-Betrag (0..99), X = 0 (Einer/Zehner) oder 1 (Hunderter/Tausender).  Ein Extraleben, wenn die Zehntausender
;   weiterzaehlen (hoechstens 9 Leben).  Veraendert A, X.
bcd_add:
        pha
        lda SCORE + 2
        tay                             ; (Y nur als Zwischenspeicher)
        pla
        sed
        clc
        adc SCORE,x
        sta SCORE,x
        lda #0
        adc SCORE + 1,x
        sta SCORE + 1,x
        lda #0
        adc SCORE + 2,x
        sta SCORE + 2,x
        cld
        cpy SCORE + 2
        beq .out
        lda LIVES
        cmp #9
        bcs .out
        inc LIVES
.out:   rts
}

.if EXTRA_ITEMS
ITEM_SYM:    .byte CH_SPACE, CH_STAR, CH_ICE
ITEM_SYMCOL: .byte COL_WHITE, COL_YELLOW, COL_LTBLUE
ITEM_TXT:
        .encoding "screen" {
        .text "        "
        .text "star +50"
        .text "ice stop"
        }
.endif

; ---------------------------------------------------------------------------------------------------------------------
; Titelbild
; ---------------------------------------------------------------------------------------------------------------------
.const TITLE_COL  = 22                  ; Spalte der Werte im Titelbild
.const TITLE_ROW  = 7                   ; Zeile der ersten Einstellung (die naechsten im Abstand von zwei Zeilen)

; title_draw: Titelbild aufbauen (Rahmen aus Linienzeichen, Titel, Einstellungen, Bestenliste).  Veraendert A, X, Y, T0..T2, PTR_A, PTR_B.
title_draw:
.if FX_TITLE
        jsr draw_logo
        ldx #7                          ; Bildlaufzeile (Zeile 6) in Cyan
        lda #COL_CYAN
.sc:    sta COLOR_RAM + 6 * SCR_COLS,x
        dex
        bpl .sc
        ldx #39
.sc2:   sta COLOR_RAM + 6 * SCR_COLS,x
        dex
        cpx #7
        bne .sc2
        lda #0
        sta SIDX
        ldx #HT_TPLAYERS
        jsr hud.print_all
.else
        ldx #21                         ; Rahmen: Zeilen 1 und 5, Spalten 9..30, gelb
.fr:    lda #CH_HLINE
        sta SCREEN_RAM + 1 * SCR_COLS + 9,x
        sta SCREEN_RAM + 5 * SCR_COLS + 9,x
        lda #COL_YELLOW
        sta COLOR_RAM + 1 * SCR_COLS + 9,x
        sta COLOR_RAM + 5 * SCR_COLS + 9,x
        dex
        bpl .fr
        lda #CH_TL
        sta SCREEN_RAM + 1 * SCR_COLS + 9
        lda #CH_TR
        sta SCREEN_RAM + 1 * SCR_COLS + 30
        lda #CH_BL
        sta SCREEN_RAM + 5 * SCR_COLS + 9
        lda #CH_BR
        sta SCREEN_RAM + 5 * SCR_COLS + 30
.for r in range(2, 5)
        lda #CH_VLINE
        sta SCREEN_RAM + r * SCR_COLS + 9
        sta SCREEN_RAM + r * SCR_COLS + 30
        lda #COL_YELLOW
        sta COLOR_RAM + r * SCR_COLS + 9
        sta COLOR_RAM + r * SCR_COLS + 30
.endfor
        ldx #HT_TITLE
        jsr hud.print_all
.endif
.if EXTRA_HISCORE
        jmp hiscore.show
.else
        rts
.endif

.if FX_TITLE
; draw_logo: grosses Logo "TROXX" aus 5 x 5 Bloecken je Buchstabe (abgeschraegte Mehrfarben-Bloecke, Zeilen 1..5, ab Spalte 5).  Veraendert A, X, Y, T0..T3, PTR_A, PTR_B.
draw_logo:
        ldy #0
.r:     sty T0                          ; Zeile 0..4
        lda ROW_LO + 1,y
        clc
        adc #5
        sta PTR_A
        sta PTR_B
        lda ROW_HI + 1,y
        sta PTR_A + 1
        clc
        adc #COLOR_DELTA
        sta PTR_B + 1
        lda #0
        sta T1                          ; Spalte
        ldx #0                          ; Buchstabe 0..4
.l:     stx T3
        lda LOGO_ORDER,x
        sta T2
        asl
        asl
        clc
        adc T2                          ; * 5
        clc
        adc T0
        tay
        lda LOGO_BITS,y
        asl
        asl
        asl
        sta T2
        ldy T1
        ldx #5
.b:     asl T2
        bcc .blank
        lda #CH_TILE
        sta (PTR_A),y
        lda #COL_MULTI + COL_YELLOW
        sta (PTR_B),y
.blank: iny
        dex
        bne .b
        iny                             ; Luecke zwischen den Buchstaben
        sty T1
        ldx T3
        inx
        cpx #5
        bne .l
        ldy T0
        iny
        cpy #5
        bne .r
        rts

; title_fx: je Bild im Titelbild: Farbzyklus im Logo und weicher Bildlauf in Zeile 6 (Versatz RB_VAR + 4 wird vom Rasterinterrupt ab Zeile 97 gesetzt;
;   alle acht Bilder rueckt der Text um ein Zeichen).  Veraendert A, X, Y, PTR_A, PTR_B, T0.
title_fx:
        inc FXC
        ldy #0                          ; Logo: Farbe nach Zeile, Spalte und Zaehler
.row:   sty T0
        lda ROW_LO + 1,y
        clc
        adc #5
        sta PTR_A
        sta PTR_B
        lda ROW_HI + 1,y
        sta PTR_A + 1
        clc
        adc #COLOR_DELTA
        sta PTR_B + 1
        ldy #28
.col:   lda (PTR_A),y
        cmp #CH_TILE
        bne .skip
        lda FXC                         ; Farbband laeuft ueber die Zeilen: ein Buchstabe ist je Zeile einfarbig
        lsr
        lsr
        clc
        adc T0
        and #7
        tax
        lda LOGO_CYCLE,x
        ora #COL_MULTI
        sta (PTR_B),y
.skip:  dey
        bpl .col
        ldy T0
        iny
        cpy #5
        bne .row
        dec RB_VAR + 4                  ; Bildlauf
        bpl .out
        lda #7
        sta RB_VAR + 4
        ldx #0
.sh:    lda SCREEN_RAM + 6 * SCR_COLS + 1,x
        sta SCREEN_RAM + 6 * SCR_COLS,x
        inx
        cpx #39
        bne .sh
        ldx SIDX
        lda SCROLL_TEXT,x
        bne .ch
        ldx #0
        stx SIDX
        lda SCROLL_TEXT
.ch:    sta SCREEN_RAM + 6 * SCR_COLS + 39
        inc SIDX
.out:   rts

LOGO_ORDER: .byte 0, 1, 2, 3, 3         ; T R O X X
LOGO_BITS:                              ; je Buchstabe fuenf Zeilen zu fuenf Bit
        .byte %11111, %00100, %00100, %00100, %00100   ; T
        .byte %11110, %10001, %11110, %10100, %10011   ; R
        .byte %01110, %10001, %10001, %10001, %01110   ; O
        .byte %10001, %01010, %00100, %01010, %10001   ; X
LOGO_CYCLE: .byte $47, $57, $67, $77, $75, $73, $63, $62     ; Verlauf: Gelb heller, Gruen, Cyan, dann zurueck (Farbton und Helligkeit)
SCROLL_TEXT:
        .encoding "screen" {
        .textnull "      troxx for the plus/4 - drive along the lines, close the boxes, dodge the enemies, grab the star and the ice block -      "
        }
.endif

; title_values: veraenderliche Werte des Titelbilds (Spielerzahl, Joystick, Tempo) und den Auswahlzeiger ausgeben.  Veraendert A, X, Y, PTR_A.
title_values:
        lda NPLAYERS
        ora #48
        sta SCREEN_RAM + TITLE_ROW * SCR_COLS + TITLE_COL
        ldx JOYMODE
        lda JOY_OFF,x
        tax
        ldy #0
.j:     lda JOY_TEXT,x
        sta SCREEN_RAM + (TITLE_ROW + 2) * SCR_COLS + TITLE_COL,y
        inx
        iny
        cpy #10
        bne .j
        lda title_speed
        ora #48
        sta SCREEN_RAM + (TITLE_ROW + 4) * SCR_COLS + TITLE_COL
        ldx #2                          ; Zeiger (Spalte 9) an der gewaehlten Zeile
.a:     txa
        asl
        tay                             ; Y = 2 * Zeile
        lda ROW_LO + TITLE_ROW,y
        sta PTR_A
        lda ROW_HI + TITLE_ROW,y
        sta PTR_A + 1
        lda #CH_SPACE
        cpx SELROW
        bne .sa
        lda #CH_ARROW
.sa:    ldy #9
        sta (PTR_A),y
        dex
        bpl .a
        rts

TITLE_COLS: .byte COL_YELLOW, COL_GREEN, COL_CYAN, COL_PURPLE, COL_RED
JOY_OFF:  .byte 0, 10, 20
JOY_TEXT:
        .encoding "screen" {
        .text "port 2    "
        .text "port 1    "
        .text "two sticks"
        }

; Saetze: Adresse, Farbe, Text.  Zwei Tabellen: hud_texts (Seitenleiste, Titelbild, Namenseingabe; Gruppen enden mit $FF) und msg_texts (Meldungen).
hud_texts:
t_score:
        .word SCREEN_RAM + ROW_TOP * SCR_COLS + 3
        .byte COL_CYAN
        .encoding "screen" {
        .textnull "score"
        }
t_level:
        .word SCREEN_RAM + ROW_TOP * SCR_COLS + 17
        .byte COL_CYAN
        .encoding "screen" {
        .textnull "level"
        }
t_hi:
        .word SCREEN_RAM + ROW_TOP * SCR_COLS + 28
        .byte COL_CYAN
        .encoding "screen" {
        .textnull "hi"
        }
t_speed:
        .word SCREEN_RAM + ROW_BOT * SCR_COLS + 3
        .byte COL_CYAN
        .encoding "screen" {
        .textnull "speed"
        }
t_bonus:
        .word SCREEN_RAM + ROW_BOT * SCR_COLS + 14
        .byte COL_CYAN
        .encoding "screen" {
        .textnull "bonus"
        }
t_sys:
        .word SCREEN_RAM + ROW_BOT * SCR_COLS + 33
        .byte COL_YELLOW
        .encoding "screen" {
        .textnull "plus/4"
        }
t_player:
        .word SCREEN_RAM + ROW_LIVES * SCR_COLS + 29
        .byte COL_CYAN
        .encoding "screen" {
        .textnull "player"
        }
        .byte $ff                       ; Ende der Gruppe fuer print_all
t_title:
        .word SCREEN_RAM + 3 * SCR_COLS + 15
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "t r o x x"
        }
t_tplayers:
        .word SCREEN_RAM + TITLE_ROW * SCR_COLS + 11
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "players"
        }
t_tjoy:
        .word SCREEN_RAM + (TITLE_ROW + 2) * SCR_COLS + 11
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "joystick"
        }
t_tspeed:
        .word SCREEN_RAM + (TITLE_ROW + 4) * SCR_COLS + 11
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "speed"
        }
t_tfire:
        .word SCREEN_RAM + 13 * SCR_COLS + 14
        .byte COL_YELLOW
        .encoding "screen" {
        .textnull "fire = start"
        }
t_thi:
        .word SCREEN_RAM + 14 * SCR_COLS + 14
        .byte COL_CYAN
        .encoding "screen" {
        .textnull "high scores"
        }
        .byte $ff
t_newhs:
        .word SCREEN_RAM + 8 * SCR_COLS + 13
        .byte COL_YELLOW
        .encoding "screen" {
        .textnull "new high score"
        }
t_enter:
        .word SCREEN_RAM + 12 * SCR_COLS + 15
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "enter name"
        }
t_fok:
        .word SCREEN_RAM + 18 * SCR_COLS + 14
        .byte COL_CYAN
        .encoding "screen" {
        .textnull "fire = done"
        }
        .byte $ff
t_plname:
        .word SCREEN_RAM + 10 * SCR_COLS + 13
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "player "
        }

msg_texts:
t_pause:
        .word SCREEN_RAM + MSG_ROW * SCR_COLS + MSG_COL
        .byte COL_YELLOW
        .encoding "screen" {
        .textnull "pause"
        }
t_ready:
        .word SCREEN_RAM + MSG_ROW * SCR_COLS + MSG_COL
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "ready"
        }
t_hit:
        .word SCREEN_RAM + MSG_ROW * SCR_COLS + MSG_COL
        .byte COL_RED
        .encoding "screen" {
        .textnull "hit"
        }
t_clear:
        .word SCREEN_RAM + MSG_ROW * SCR_COLS + MSG_COL
        .byte COL_GREEN
        .encoding "screen" {
        .textnull "level clear"
        }
t_over:
        .word SCREEN_RAM + MSG_ROW * SCR_COLS + MSG_COL
        .byte COL_RED
        .encoding "screen" {
        .textnull "game over"
        }
t_pl:
        .word SCREEN_RAM + MSG_ROW * SCR_COLS + MSG_COL
        .byte COL_CYAN
        .encoding "screen" {
        .textnull "player 1 ready"
        }
t_fire:
        .word SCREEN_RAM + MSG_ROW * SCR_COLS + MSG_COL
        .byte COL_CYAN
        .encoding "screen" {
        .textnull "pass joystick - fire"
        }
t_saving:
        .word SCREEN_RAM + MSG_ROW * SCR_COLS + MSG_COL
        .byte COL_CYAN
        .encoding "screen" {
        .textnull "saving..."
        }
msg_end:

.const HT_STATIC = t_score - hud_texts
.const HT_TITLE  = t_title - hud_texts
.const HT_TPLAYERS = t_tplayers - hud_texts
.const HT_NEWHS  = t_newhs - hud_texts
.const HT_PLNAME = t_plname - hud_texts
.const HT_PAUSE  = t_pause - msg_texts
.const HT_READY  = t_ready - msg_texts
.const HT_HIT    = t_hit - msg_texts
.const HT_CLEAR  = t_clear - msg_texts
.const HT_OVER   = t_over - msg_texts
.const HT_PLAYER = t_pl - msg_texts
.const HT_FIRE   = t_fire - msg_texts
.const HT_SAVING = t_saving - msg_texts
.assert msg_texts - hud_texts < 256, "hud_texts muss mit 8-Bit-Abstaenden erreichbar sein"
.assert msg_end - msg_texts < 256, "msg_texts muss mit 8-Bit-Abstaenden erreichbar sein"
