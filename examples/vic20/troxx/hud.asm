; =====================================================================================================================
; hud.asm - Anzeigen: Statuszeile (Bildschirmzeile 0), Anzeigezeile (Zeile 22), Meldungen, BCD-Punkte
; =====================================================================================================================
; ZWECK     Texte stehen als Saetze in hud_texts: Bildschirmadresse (Wort), Farbe (Byte), Zeichen als Bildschirmcodes, abgeschlossen mit 0.
;           Bildschirmcode 0 (die waagerechte Linie) darf in Texten nicht vorkommen.  Quelltext in KLEINBUCHSTABEN: so entstehen die Codes
;           1..26 des Zeichensatzes (A-Z).  Texte bleiben kurz und ohne Umlaute.
;           Punkte stehen als BCD (drei Byte, niedrigstes zuerst), Stufe auch als BCD.
; BEDIENUNG hud.print mit X = Abstand des Satzes in hud_texts (HT_*); hud.msg gibt eine Meldung in der Anzeigezeile aus (Spalten 3..20);
;           hud.draw die festen Anzeigen, hud.update die Zahlen (je Bild); hud.bcd_add bucht Punkte.
; ANZEIGE   Zeile 0:  score 000000   lv 01      Zeile 22:  Herz Leben  spd Tempo  best Bestpunkte
; =====================================================================================================================

hud: {

; print: Satz X ausgeben.  Veraendert A, X, Y, T2, PTR_A, PTR_B.
print:  lda hud_texts,x
        sta PTR_A
        sta PTR_B                       ; Bildschirm und Farbspeicher haben dasselbe Low-Byte
        lda hud_texts + 1,x
        sta PTR_A + 1
        clc
        adc #COLOR_DELTA
        sta PTR_B + 1
        lda hud_texts + 2,x
        sta T2
        inx
        inx
        inx
        ldy #0
.chr:   lda hud_texts,x
        beq .done
        sta (PTR_A),y
        lda T2
        sta (PTR_B),y
        inx
        iny
        bne .chr
.done:  rts

; clrmsg: Spalten 3..20 der Anzeigezeile leeren.  Veraendert A, X.
clrmsg: ldx #17
        lda #CH_SPACE
.clr:   sta SCREEN_RAM + 22 * SCR_COLS + 3,x
        dex
        bpl .clr
        rts

; msg: Meldung X (Satz in hud_texts) in der Anzeigezeile ausgeben (Spalten 3..20 vorher leeren).  Veraendert A, X, Y, T2, PTR_A, PTR_B.
msg:    txa
        pha
        jsr clrmsg
        pla
        tax
        jmp print

; draw: feste Anzeigen (Beschriftungen); loescht eine Meldung.  Veraendert A, X, Y, T2, PTR_A, PTR_B.
draw:   jsr clrmsg
        ldx #HT_SCORE
        ; weiter mit print_all

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

; update: Zahlen (Punkte, Stufe, Leben, Tempo, Bestpunkte) ausgeben.  Veraendert A, Y, PTR_A.
update: lda #<SCREEN_RAM
        sta PTR_A
        lda #>SCREEN_RAM
        sta PTR_A + 1
        ldy #6
        ldx #SCORE + 2
        jsr bcd3
        ldy #17
        lda LEVEL
        jsr bcd2
        lda #<(SCREEN_RAM + 22 * SCR_COLS)
        sta PTR_A
        lda #>(SCREEN_RAM + 22 * SCR_COLS)
        sta PTR_A + 1
        ldy #1
        lda LIVES
        ora #48
        sta (PTR_A),y
        ldy #7
        lda SPEED
        ora #48
        sta (PTR_A),y
        ldy #14
        ldx #HISCORE + 2
        jmp bcd3

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

; Saetze: Adresse, Farbe, Text.  Zeile 0 und Zeile 22 liegen am Anfang und am Ende des Bildschirms (22 Zeichen je Zeile).
hud_texts:
t_score:
        .word SCREEN_RAM
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "score"
        }
t_lv:
        .word SCREEN_RAM + 14
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "lv"
        }
t_heart:
        .word SCREEN_RAM + 22 * SCR_COLS
        .byte COL_RED
        .byte CH_HEART, 0
t_spd:
        .word SCREEN_RAM + 22 * SCR_COLS + 3
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "spd"
        }
t_best:
        .word SCREEN_RAM + 22 * SCR_COLS + 9
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "best"
        }
        .byte $ff                       ; Ende der Gruppe fuer print_all
t_title:
        .word SCREEN_RAM + 7 * SCR_COLS + 8
        .byte COL_YELLOW
        .encoding "screen" {
        .textnull "troxx"
        }
t_tempo:
        .word SCREEN_RAM + 11 * SCR_COLS + 6
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "tempo"
        }
t_start:
        .word SCREEN_RAM + 13 * SCR_COLS + 4
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "feuer"
        }
        .byte $ff
t_pause:
        .word SCREEN_RAM + 22 * SCR_COLS + 3
        .byte COL_YELLOW
        .encoding "screen" {
        .textnull "pause"
        }
t_ready:
        .word SCREEN_RAM + 22 * SCR_COLS + 3
        .byte COL_WHITE
        .encoding "screen" {
        .textnull "bereit"
        }
t_hit:
        .word SCREEN_RAM + 22 * SCR_COLS + 3
        .byte COL_RED
        .encoding "screen" {
        .textnull "treffer"
        }
t_clear:
        .word SCREEN_RAM + 22 * SCR_COLS + 3
        .byte COL_GREEN
        .encoding "screen" {
        .textnull "stufe ok"
        }
t_over:
        .word SCREEN_RAM + 22 * SCR_COLS + 3
        .byte COL_RED
        .encoding "screen" {
        .textnull "spiel ende"
        }

.const HT_SCORE = t_score - hud_texts
.const HT_LV    = t_lv - hud_texts
.const HT_HEART = t_heart - hud_texts
.const HT_SPD   = t_spd - hud_texts
.const HT_BEST  = t_best - hud_texts
.const HT_TITLE = t_title - hud_texts

.const HT_TEMPO = t_tempo - hud_texts
.const HT_PAUSE = t_pause - hud_texts
.const HT_START = t_start - hud_texts
.const HT_READY = t_ready - hud_texts
.const HT_HIT   = t_hit - hud_texts
.const HT_CLEAR = t_clear - hud_texts
.const HT_OVER  = t_over - hud_texts
.assert * - hud_texts < 256, "Satztabelle muss mit 8-Bit-Abstaenden erreichbar sein"
