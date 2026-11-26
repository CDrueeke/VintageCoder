; =====================================================================================================================
; hud.asm - Anzeigen: Kopfzeile, Seitenleiste (Spalten 34..39), Meldungszeile (Zeile 23), Titelbild, BCD-Punkte
; =====================================================================================================================
; Herkunft: examples/vic20/troxx/hud.asm, Stand Commit 4697d074; Atari-Fassung nach examples/c64/troxx/hud.asm (Stand Commit d173cb4a).  Kopien (Pruefsumme, siehe
;   KOPIEN.txt): bcd3, bcd_add, bcd_sub.  Angepasst: print, clrmsg, msg, draw, update, bcd2 (Ziffernzeichen des Modus 4: Code 80 + Ziffer); neu: bcd2t (Ziffern
;   des Modus 2), bcd3t, das Titelbild (Logo, Lauftext, Farbzyklus).
; ZWECK     Texte stehen als Saetze in hud_texts / msg_texts: Bildschirmadresse (Wort), Maske (Byte: 64 = Seitenschrift des Modus 4, 0 = Schrift des Modus 2),
;           Zeichen als Atari-Bildschirmcodes (Quelltext in GROSSBUCHSTABEN), das letzte Zeichen mit gesetztem Bit 7 (.textshift; der Code des Leerzeichens
;           ist 0, darum gibt es keine Endemarke 0).  Texte bleiben kurz und ohne Umlaute.  Punkte stehen als BCD (drei Byte, niedrigstes zuerst), Stufe auch als BCD.
; BEDIENUNG hud.print mit X = Abstand des Satzes in hud_texts (HT_*); hud.msg gibt eine Meldung in Zeile 23 aus (Spalten 0..39); hud.draw die festen Anzeigen,
;           hud.update die Zahlen (je Bild); hud.bcd_add bucht Punkte.
; ANZEIGE   Seitenleiste: TROXX, SCORE (6 Ziffern), HI (beste Punkte), LEVEL, LIVES (Herzen), SPEED, BONUS (Gegenstand auf dem Feld), PLAYER.  Die Farbe der
;           Seitenschrift kommt vom DLI (Zeilenfarbe COLPF2); die Herzen haben Bit 7 und damit COLPF3 (rot).
; =====================================================================================================================

.const MSG_ROW  = 23
.const SROW_SCORE = 4                    ; Zeilen der Zahlen in der Seitenleiste (Spalte SIDE_COL)
.const SROW_HI   = 7
.const SROW_LEVEL = 10
.const SROW_LIVES = 13
.const SROW_SPEED = 16
.const SROW_BONUS = 19
.const SROW_PLAYER = 22

.macro print_from(tbl)
        lda tbl,x
        sta PTR_A
        lda tbl + 1,x
        sta PTR_A + 1
        lda tbl + 2,x
        sta T2                          ; Maske: 64 fuer die Seitenschrift (Modus 4), 0 fuer die Schrift des Modus 2
        inx
        inx
        inx
        ldy #0
@chr:   lda tbl,x
        pha
        and #$7f
        ora T2
        sta (PTR_A),y
        inx
        iny
        pla
        bpl @chr                        ; das letzte Zeichen des Satzes hat Bit 7 (.textshift)
        rts
.endmacro

hud: {

; print: Satz X (Abstand in hud_texts) ausgeben.  Veraendert A, X, Y, T2, PTR_A.
print:  print_from hud_texts

; mprint: Satz X (Abstand in msg_texts) ausgeben.  Veraendert wie print.
mprint: print_from msg_texts

; clrmsg: Meldungszeile (Zeile 23, Spalten 0..39) leeren.  Veraendert A, X.
clrmsg: ldx #SCR_COLS - 1
        lda #CH_SPACE
.clr:   sta SCR + MSG_ROW * SCR_COLS,x
        dex
        bpl .clr
        rts

; msg: Meldung X (Satz in msg_texts) in der Meldungszeile ausgeben (vorher leeren).  Veraendert A, X, Y, T2, PTR_A.
msg:    txa
        pha
        jsr clrmsg
        pla
        tax
        jmp mprint

; draw: feste Anzeigen (Kopfzeile, Beschriftungen der Seitenleiste); loescht eine Meldung.  Veraendert A, X, Y, T2, PTR_A.
draw:   jsr clrmsg
        ldx #HT_TROXX
        jsr print_all
        lda NPLAYERS                    ; die Beschriftung "PLAYER" nur bei zwei Spielern
        cmp #2
        beq .out
        ldx #5
        lda #CH_SPACE
.np:    sta SCR + (SROW_PLAYER - 1) * SCR_COLS + SIDE_COL,x
        dex
        bpl .np
.out:   rts

; print_all: alle Saetze ab X bis zur Endemarke $FF ausgeben.  Veraendert A, X, Y, T2, PTR_A.
print_all:
        jsr print                       ; X steht danach hinter dem Satz
        lda hud_texts,x
        cmp #$ff
        bne print_all
        rts

; bcd2: A = BCD-Byte: zwei Ziffern (Zeichen der Seitenschrift, Modus 4) nach (PTR_A),Y; Y wird um 2 groesser.  Veraendert A, Y.
bcd2:   pha
        lsr
        lsr
        lsr
        lsr
        ora #DIG4
        sta (PTR_A),y
        iny
        pla
        and #15
        ora #DIG4
        sta (PTR_A),y
        iny
        rts

; bcd2t: wie bcd2, mit den Ziffern des Modus 2 (Titelbild).  Veraendert A, Y.
bcd2t:  pha
        lsr
        lsr
        lsr
        lsr
        ora #DIG2
        sta (PTR_A),y
        iny
        pla
        and #15
        ora #DIG2
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

; setp: PTR_A = Bildschirmadresse der Seitenleiste, Zeile A (Spalte SIDE_COL).  Veraendert A, X.
setp:   tax
        lda ROW_LO,x
        clc
        adc #SIDE_COL
        sta PTR_A
        lda ROW_HI,x
        adc #0
        sta PTR_A + 1
        rts

; update: Zahlen (Punkte, beste Punkte, Stufe, Leben, Tempo, Spieler, Gegenstand) ausgeben.  Veraendert A, X, Y, PTR_A.
update: lda #SROW_SCORE
        jsr setp
        ldy #0
        ldx #SCORE + 2
        jsr bcd3
        lda #SROW_HI
        jsr setp
        ldy #0
        ldx #HISCORE + 2
        jsr bcd3
        lda #SROW_LEVEL
        jsr setp
        ldy #0
        lda LEVEL
        jsr bcd2
        ldx #5                          ; Leben als Herzen (hoechstens sechs Plaetze; 9 Leben zeigen sechs Herzen)
.h:     cpx LIVES
        bcs .nh
        lda #CH_HEART + $80             ; Bit 7: COLPF3 (rot)
        bne .hs
.nh:    lda #CH_SPACE
.hs:    sta SCR + SROW_LIVES * SCR_COLS + SIDE_COL,x
        dex
        bpl .h
        lda SPEED
        ora #DIG4
        sta SCR + SROW_SPEED * SCR_COLS + SIDE_COL
        lda #CH_SPACE
        ldx NPLAYERS
        cpx #2
        bne .p1
        lda CUR
        clc
        adc #DIG4 + 1
.p1:    sta SCR + SROW_PLAYER * SCR_COLS + SIDE_COL
.if EXTRA_ITEMS
        ldx IT_KIND                     ; Gegenstand auf dem Feld: Symbol neben "BONUS"
        lda ITEM_SYM,x
        sta SCR + SROW_BONUS * SCR_COLS + SIDE_COL
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
.endif

; ---------------------------------------------------------------------------------------------------------------------
; Titelbild
; ---------------------------------------------------------------------------------------------------------------------
.const TITLE_COL  = 22                  ; Spalte der Werte im Titelbild
.const TITLE_ROW  = 9                   ; Zeile der ersten Einstellung (die naechsten im Abstand von zwei Zeilen)

; title_draw: Titelbild aufbauen (Logo, Lauftext, Einstellungen, Bestenliste).  Veraendert A, X, Y, T0..T3, PTR_A.
title_draw:
        ldx #HT_TITLE
        jsr hud.print_all
.if FX_TITLE
        jsr draw_logo
        jsr scroll_init
.endif
.if EXTRA_HISCORE
        jmp hiscore.show
.else
        rts
.endif

.if FX_TITLE
; draw_logo: grosses Logo "TROXX" aus 5 x 5 Bloecken je Buchstabe (abgeschraegte Bloecke, Zeilen 1..5, ab Spalte 5).  Veraendert A, X, Y, T0..T3, PTR_A.
draw_logo:
        ldy #0
.r:     sty T0                          ; Zeile 0..4
        lda ROW_LO + 1,y
        clc
        adc #5
        sta PTR_A
        lda ROW_HI + 1,y
        adc #0
        sta PTR_A + 1
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
        lda #CH_FTL
        sta (PTR_A),y
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

; scroll_init: Lauftextpuffer SCRL (48 Zeichen) mit Leerzeichen fuellen, Leseposition 0.  Veraendert A, X.
scroll_init:
        lda #0
        sta SIDX
        sta SSUB
        ldx #47
.s:     sta SCRL,x
        dex
        bpl .s
        lda #3
        sta RC_H + 7
        rts

; title_fx: je Bild im Titelbild: Farbzyklus im Logo (Zeilenfarben COLPF2 der Logozeilen 1..5) und weicher Bildlauf der Zeile 7 (HSCROL 3..0, danach rueckt
;   der Text um ein Zeichen).  Veraendert A, X, Y.
title_fx:
        inc FXC
        ldy #0
.row:   lda FXC                         ; Farbband laeuft ueber die Zeilen
        lsr
        lsr
        clc
        adc FXROWS,y
        and #7
        tax
        lda LOGO_CYCLE,x
        sta RC_P2 + 1,y
        iny
        cpy #5
        bne .row
        lda FXC
        and #1
        bne .out                        ; Bildlauf: ein Schritt je zwei Bilder
        dec RC_H + 7
        bpl .out
        lda #3
        sta RC_H + 7
        ldx #0
.sh:    lda SCRL + 1,x
        sta SCRL,x
        inx
        cpx #47
        bne .sh
        ldx SIDX
        lda SCROLL_TEXT,x
        cmp #$ff
        bne .ch
        ldx #0
        stx SIDX
        lda SCROLL_TEXT
.ch:    sta SCRL + 47
        inc SIDX
.out:   rts

FXROWS: .byte 0, 1, 2, 3, 4
LOGO_ORDER: .byte 0, 1, 2, 3, 3         ; T R O X X
LOGO_BITS:                              ; je Buchstabe fuenf Zeilen zu fuenf Bit
        .byte %11111, %00100, %00100, %00100, %00100   ; T
        .byte %11110, %10001, %11110, %10100, %10011   ; R
        .byte %01110, %10001, %10001, %10001, %01110   ; O
        .byte %10001, %01010, %00100, %01010, %10001   ; X
LOGO_CYCLE: .byte $1e, $2c, $3c, $5c, $9c, $cc, $dc, $fc   ; Farbzyklus (Gold, Orange, Rot, Purpur, Blau, Gruen ...)
SCROLL_TEXT:
        .encoding "screen" {
        .text "       TROXX FOR THE ATARI 8 BIT - DRIVE ALONG THE LINES, CLOSE THE BOXES, DODGE THE ENEMIES, GRAB THE STAR AND THE ICE BLOCK -       "
        }
        .byte $ff
.endif

; title_values: veraenderliche Werte des Titelbilds (Spielerzahl, Joystick, Tempo) und den Auswahlzeiger ausgeben.  Veraendert A, X, Y, PTR_A.
title_values:
        lda NPLAYERS
        ora #DIG2
        sta SCR + TITLE_ROW * SCR_COLS + TITLE_COL
        ldx JOYMODE
        lda JOY_OFF,x
        tax
        ldy #0
.j:     lda JOY_TEXT,x
        sta SCR + (TITLE_ROW + 2) * SCR_COLS + TITLE_COL,y
        inx
        iny
        cpy #10
        bne .j
        lda title_speed
        ora #DIG2
        sta SCR + (TITLE_ROW + 4) * SCR_COLS + TITLE_COL
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
        lda #30                         ; ">"
.sa:    ldy #9
        sta (PTR_A),y
        dex
        bpl .a
        rts

JOY_OFF:  .byte 0, 10, 20
JOY_TEXT:
        .encoding "screen" {
        .text "PORT 1    "
        .text "PORT 2    "
        .text "TWO STICKS"
        }

; Saetze: Adresse, Maske, Text.  Zwei Tabellen: hud_texts (Seitenleiste, Titelbild, Namenseingabe; Gruppen enden mit $FF) und msg_texts (Meldungen).
hud_texts:
t_troxx:                                ; Seitenleiste (Modus 4, Maske 64)
        .word SCR + 1 * SCR_COLS + SIDE_COL
        .byte 64
        .encoding "screen" {
        .textshift "TROXX"
        }
t_score:
        .word SCR + 3 * SCR_COLS + SIDE_COL
        .byte 64
        .encoding "screen" {
        .textshift "SCORE"
        }
t_hi:
        .word SCR + 6 * SCR_COLS + SIDE_COL
        .byte 64
        .encoding "screen" {
        .textshift "HI"
        }
t_level:
        .word SCR + 9 * SCR_COLS + SIDE_COL
        .byte 64
        .encoding "screen" {
        .textshift "LEVEL"
        }
t_lives:
        .word SCR + 12 * SCR_COLS + SIDE_COL
        .byte 64
        .encoding "screen" {
        .textshift "LIVES"
        }
t_speed:
        .word SCR + 15 * SCR_COLS + SIDE_COL
        .byte 64
        .encoding "screen" {
        .textshift "SPEED"
        }
t_bonus:
        .word SCR + 18 * SCR_COLS + SIDE_COL
        .byte 64
        .encoding "screen" {
        .textshift "BONUS"
        }
t_player:
        .word SCR + 21 * SCR_COLS + SIDE_COL
        .byte 64
        .encoding "screen" {
        .textshift "PLAYER"
        }
t_head:                                 ; Kopfzeile (Modus 2)
        .word SCR + 1
        .byte 0
        .encoding "screen" {
        .textshift "TROXX - ATARI 8 BIT"
        }
        .byte $ff                       ; Ende der Gruppe fuer print_all
t_title:                                ; Titelbild (Modus 2)
        .word SCR + 0 * SCR_COLS + 12
        .byte 0
        .encoding "screen" {
        .textshift "ATARI XL / XE VERSION"
        }
t_tplayers:
        .word SCR + TITLE_ROW * SCR_COLS + 11
        .byte 0
        .encoding "screen" {
        .textshift "PLAYERS"
        }
t_tjoy:
        .word SCR + (TITLE_ROW + 2) * SCR_COLS + 11
        .byte 0
        .encoding "screen" {
        .textshift "JOYSTICK"
        }
t_tspeed:
        .word SCR + (TITLE_ROW + 4) * SCR_COLS + 11
        .byte 0
        .encoding "screen" {
        .textshift "SPEED"
        }
t_tfire:
        .word SCR + 15 * SCR_COLS + 13
        .byte 0
        .encoding "screen" {
        .textshift "FIRE OR START"
        }
t_thi:
        .word SCR + 17 * SCR_COLS + 14
        .byte 0
        .encoding "screen" {
        .textshift "HIGH SCORES"
        }
        .byte $ff
t_newhs:
        .word SCR + 8 * SCR_COLS + 13
        .byte 0
        .encoding "screen" {
        .textshift "NEW HIGH SCORE"
        }
t_enter:
        .word SCR + 12 * SCR_COLS + 15
        .byte 0
        .encoding "screen" {
        .textshift "ENTER NAME"
        }
t_fok:
        .word SCR + 18 * SCR_COLS + 14
        .byte 0
        .encoding "screen" {
        .textshift "FIRE = DONE"
        }
        .byte $ff
t_plname:
        .word SCR + 10 * SCR_COLS + 13
        .byte 0
        .encoding "screen" {
        .textshift "PLAYER "
        }

msg_texts:
t_pause:
        .word SCR + MSG_ROW * SCR_COLS + 4
        .byte 0
        .encoding "screen" {
        .textshift "PAUSE"
        }
t_ready:
        .word SCR + MSG_ROW * SCR_COLS + 4
        .byte 0
        .encoding "screen" {
        .textshift "READY"
        }
t_hit:
        .word SCR + MSG_ROW * SCR_COLS + 4
        .byte 0
        .encoding "screen" {
        .textshift "HIT"
        }
t_clear:
        .word SCR + MSG_ROW * SCR_COLS + 4
        .byte 0
        .encoding "screen" {
        .textshift "LEVEL CLEAR"
        }
t_over:
        .word SCR + MSG_ROW * SCR_COLS + 4
        .byte 0
        .encoding "screen" {
        .textshift "GAME OVER"
        }
t_pl:
        .word SCR + MSG_ROW * SCR_COLS + 4
        .byte 0
        .encoding "screen" {
        .textshift "PLAYER 1 READY"
        }
t_fire:
        .word SCR + MSG_ROW * SCR_COLS + 4
        .byte 0
        .encoding "screen" {
        .textshift "PASS JOYSTICK - FIRE"
        }
t_saving:
        .word SCR + MSG_ROW * SCR_COLS + 4
        .byte 0
        .encoding "screen" {
        .textshift "SAVING..."
        }
msg_end:

.const HT_TROXX  = t_troxx - hud_texts
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
