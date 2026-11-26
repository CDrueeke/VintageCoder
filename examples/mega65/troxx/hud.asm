; =====================================================================================================================
; hud.asm - Anzeigen: Tafeln (Zeilen 43..48), Meldungszeile (Zeile 49), Texte, BCD-Punkte
; =====================================================================================================================
; Herkunft: examples/vic20/troxx/hud.asm und examples/c64/troxx/hud.asm, Stand Commit 4697d074 der Referenz.  Kopien (Pruefsumme, siehe KOPIEN.txt):
;   bcd2.  Angepasst: bcd_add, bcd_sub (zwei Spieler: Punkte und Leben nach WHO), print, print_all, update, draw, msg.  Neu: Tafeln, Fortschrittsbalken.
;
; ZWECK     Texte stehen als Saetze in hud_texts: Spalte, Zeile, Thema der Schrift (0 Tafel Spieler 1, 1 Tafel Spieler 2, 2 Tafel Stufe, 3 ohne Tafel),
;           Farbe (Palettenindex), Zeichen als Bildschirmcodes, abgeschlossen mit 0.  Quelltext in KLEINBUCHSTABEN: so entstehen die Codes 1..26 (A-Z).
;           Bildschirmcode 0 darf in Texten nicht vorkommen.  Texte bleiben kurz und ohne Umlaute.
;           Punkte stehen als BCD (drei Byte und ein Ueberlaufbyte je Spieler, niedrigstes zuerst), die Stufe auch als BCD.
; BEDIENUNG hud.print mit X = Abstand des Satzes in hud_texts (HT_*); hud.msg gibt eine Meldung in der Meldungszeile aus; hud.draw zeichnet die Tafeln mit
;           ihren Beschriftungen; hud.update gibt die Zahlen aus (je Bild); hud.bcd_add und hud.bcd_sub buchen Punkte des Spielers WHO.
; ANZEIGE   Tafel Spieler 1 (Spalten 0..25), Spieler 2 (27..52), Stufe (54..79); je Tafel sechs Zeilen; Meldungen in Zeile 49 (Mitte).
; =====================================================================================================================

hud: {

; print: Satz X (Nummer HT_*) ausgeben.  Veraendert A, X, Y, T4, T5, T6, T7, RT6, RT7, Z, PTR_A, sq, cq.
print:  lda TXT_LO,x
        sta PTR_A
        lda TXT_HI,x
        sta PTR_A + 1
        ldy #0
        lda (PTR_A),y
        sta T6                          ; Spalte
        iny
        lda (PTR_A),y
        sta T7                          ; Zeile
        iny
        lda (PTR_A),y
        sta T5                          ; Thema
        iny
        lda (PTR_A),y
        sta T4                          ; Farbe
        iny
.chr:   lda (PTR_A),y
        beq .done
        sty RT6
        ldx T6
        ldy T7
        jsr vic.put_char
        ldy RT6
        iny
        inc T6
        bne .chr
.done:  rts
; clrmsg: Meldungszeile (Zeile 49) leeren.  Veraendert A, X, Y, Z, T4, T5, sq, cq.
clrmsg: lda #3
        sta T5
        lda #COL_WHITE
        sta T4
        ldx #0
.c:     phx
        ldy #49
        lda #CH_SPACE
        jsr vic.put_char
        plx
        inx
        cpx #SCR_COLS
        bne .c
        rts

; msg: Meldung X (Satz in hud_texts) in der Meldungszeile ausgeben (Zeile vorher leeren).  Veraendert A, X, Y, T4..T7, Z, sq, cq.
msg:    txa
        pha
        jsr clrmsg
        pla
        tax
        jmp print

; panel: Tafel zeichnen.  A = Thema (0..2), X = Spalte links, Y = Zeile oben, T6 = Breite (Zellen), T7 = Hoehe (Zellen).  Veraendert A, X, Y, RT0..RT5, Z, sq, cq.
panel:  sta RT0
        asl
        asl
        asl
        clc
        adc RT0                         ; 9 * Thema
        clc
        adc #T_PANEL
        sta RT0                         ; erste Tafelkachel des Themas
        stx RT1                         ; Spalte links
        sty RT2                         ; Zeile oben
        lda #0
        sta RT3                         ; Zeile in der Tafel
.row:   ldy RT3                         ; Zeilenart * 3: 0 oben, 3 Mitte, 6 unten
        beq .r0
        iny
        cpy T7
        beq .r2
        lda #3
        bne .rk
.r0:    lda #0
        beq .rk
.r2:    lda #6
.rk:    sta RT4
        lda #0
        sta RT5                         ; Spalte in der Tafel
.col:   ldx RT5                         ; Spaltenart: 0 links, 1 Mitte, 2 rechts
        beq .c0
        inx
        cpx T6
        beq .c2
        lda #1
        bne .ck
.c0:    lda #0
        beq .ck
.c2:    lda #2
.ck:    clc
        adc RT4
        clc
        adc RT0
        pha
        lda RT1
        clc
        adc RT5
        tax
        lda RT2
        clc
        adc RT3
        tay
        pla
        jsr vic.put_tile
        inc RT5
        lda RT5
        cmp T6
        bne .col
        inc RT3
        lda RT3
        cmp T7
        bne .row
        rts

; draw: Tafeln mit Beschriftungen zeichnen; leert die Meldungszeile.  Veraendert A, X, Y, T0..T7, RT0..RT5, Z, sq, cq.
draw:   lda #26
        sta T6
        lda #6
        sta T7
        lda #0
        ldx #0
        ldy #43
        jsr panel
        lda #1
        ldx #27
        ldy #43
        jsr panel
        lda #2
        ldx #54
        ldy #43
        jsr panel
        jsr clrmsg
        ldx #HT_P1
        jsr print
        ldx #HT_P2
        jsr print
        ldx #HT_LEVEL
        jsr print
        ldx #HT_BEST
        jsr print
        ldx #HT_FIELDS
        jsr print
        ldx #HT_OF
        jmp print
; bcd2: A = BCD-Byte: zwei Ziffern ausgeben (Spalte T6, Zeile T7, Thema T5, Farbe T4); T6 waechst um 2.  Veraendert A, X, Y, Z, T6.
bcd2:   pha
        lsr
        lsr
        lsr
        lsr
        ora #48
        ldx T6
        ldy T7
        jsr vic.put_char
        inc T6
        pla
        and #15
        ora #48
        ldx T6
        ldy T7
        jsr vic.put_char
        inc T6
        rts

; bcd3: drei BCD-Bytes ab Zeropage-Adresse X abwaerts als sechs Ziffern (wie bcd2).  Veraendert A, X, Y, Z, T6, T3.
bcd3:   stx T3
        lda 0,x
        jsr bcd2
        ldx T3
        dex
        stx T3
        lda 0,x
        jsr bcd2
        ldx T3
        dex
        lda 0,x
        jmp bcd2

; update: Zahlen (Punkte, Leben, Stufe, Bestpunkte, Felder, Balken) ausgeben.  Veraendert A, X, Y, Z, T0..T7, sq, cq.
update: lda #0                          ; Spieler 1: Punkte
        sta T5
        lda #COL_YELLOW
        sta T4
        lda #2
        sta T6
        lda #45
        sta T7
        ldx #SCORE + 2
        jsr bcd3
        lda #1                          ; Spieler 2
        sta T5
        lda #COL_CYAN
        sta T4
        lda #29
        sta T6
        lda #45
        sta T7
        ldx #SCORE + 6
        jsr bcd3
        ; Leben: Herzen
        lda #2
        sta T6
        lda #46
        sta T7
        lda LIVES
        jsr hearts
        lda #29
        sta T6
        lda #46
        sta T7
        lda LIVES + 1
        jsr hearts
        lda #2                          ; Stufe
        sta T5
        lda #COL_WHITE
        sta T4
        lda #62
        sta T6
        lda #44
        sta T7
        lda LEVEL
        jsr bcd2
        lda #COL_LTBLUE                 ; Rekord
        sta T4
        lda #63
        sta T6
        lda #45
        sta T7
        ldx #HISCORE + 2
        jsr bcd3
        jmp fields

; hearts: A = Anzahl Leben (0..9): neun Herzfelder ab Spalte T6, Zeile T7 (volle und leere Kachel).  Veraendert A, X, Y, RT0, RT1, Z, sq, cq.
hearts: sta RT0
        ldx #0
.h:     stx RT1
        txa
        cmp RT0
        lda #T_HEART_E
        bcs .p                          ; Nummer >= Leben: leeres Herz
        lda #T_HEART
.p:     pha
        lda RT1
        clc
        adc T6
        tax
        ldy T7
        pla
        jsr vic.put_tile
        ldx RT1
        inx
        cpx #9
        bne .h
        rts

; fields: Zahl der gefuellten Rechtecke (zwei Ziffern) und Fortschrittsbalken (22 Zellen).  Veraendert A, X, Y, T0..T7, RT0, RT1, Z, sq, cq.
fields: lda #2
        sta T5
        lda #COL_WHITE
        sta T4
        lda #63
        sta T6
        lda #46
        sta T7
        lda BOXES_DONE
        ldx #0
.t:     cmp #10
        bcc .u
        sbc #10
        inx
        bne .t
.u:     sta T0                          ; Einer
        txa
        ora #48
        ldx T6
        ldy T7
        jsr vic.put_char
        inc T6
        lda T0
        ora #48
        ldx T6
        ldy T7
        jsr vic.put_char
        ldy BOXES_DONE
        lda BAR_N,y                     ; volle Zellen
        sta RT0
        ldx #0
.bar:   stx RT1
        txa
        cmp RT0
        lda #T_BAR_EMPTY
        bcs .bp
        lda #T_BAR_FULL
.bp:    pha
        lda RT1
        clc
        adc #56
        tax
        ldy #47
        pla
        jsr vic.put_tile
        ldx RT1
        inx
        cpx #22
        bne .bar
        rts
; bcd_sub: Punkte des Spielers WHO abziehen.  A = BCD-Betrag (0..99); der Punktestand wird nie kleiner als 0.  Veraendert A, X, Y, T7.
.if ENEMY_DESTROYS_LINES
bcd_sub:
        sta T7
        ldy WHO
        ldx SC_OFF,y
        sed
        lda SCORE,x
        sec
        sbc T7
        sta SCORE,x
        lda SCORE + 1,x
        sbc #0
        sta SCORE + 1,x
        lda SCORE + 2,x
        sbc #0
        sta SCORE + 2,x
        cld
        bcs .out
        lda #0                          ; unter 0: auf 0 setzen
        sta SCORE,x
        sta SCORE + 1,x
        sta SCORE + 2,x
.out:   rts
.endif

; bcd_add: Punkte des Spielers WHO buchen.  A = BCD-Betrag (0..99), X = 0 (Einer/Zehner) oder 1 (Hunderter/Tausender).  Ein Extraleben, wenn die
;   Zehntausender weiterzaehlen (hoechstens 9 Leben).  Veraendert A, X, Y, T5..T7.
bcd_add:
        sta T7
        stx T6
        ldy WHO
        lda SC_OFF,y
        sta T5                          ; Anfang des Punktestands des Spielers
        tax
        lda SCORE + 2,x
        pha                             ; Zehntausender vorher
        lda T5
        clc
        adc T6
        tax
        lda T7
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
        ldx T5
        pla
        cmp SCORE + 2,x
        beq .out
        lda LIVES,y
        cmp #9
        bcs .out
        clc
        adc #1
        sta LIVES,y
.out:   rts
}

SC_OFF: .byte 0, 4                      ; Anfang des Punktestands (SCORE) von Spieler 1 und 2: je 3 Byte plus ein Ueberlaufbyte

; Saetze: Spalte, Zeile, Thema, Farbe, Text.
hud_texts:
t_p1:   .byte 2, 44, 0, COL_YELLOW
        .encoding "screen" {
        .textnull "spieler 1"
        }
t_p2:   .byte 29, 44, 1, COL_CYAN
        .encoding "screen" {
        .textnull "spieler 2"
        }
t_level: .byte 56, 44, 2, COL_WHITE
        .encoding "screen" {
        .textnull "stufe"
        }
t_best: .byte 56, 45, 2, COL_LTBLUE
        .encoding "screen" {
        .textnull "rekord"
        }
t_fields: .byte 56, 46, 2, COL_WHITE
        .encoding "screen" {
        .textnull "felder"
        }
t_of:   .byte 65, 46, 2, COL_WHITE
        .encoding "screen" {
        .textnull "/98"
        }
t_pause: .byte 37, 49, 3, COL_YELLOW
        .encoding "screen" {
        .textnull "pause"
        }
t_ready: .byte 36, 49, 3, COL_WHITE
        .encoding "screen" {
        .textnull "bereit"
        }
t_hit:  .byte 36, 49, 3, COL_RED
        .encoding "screen" {
        .textnull "treffer"
        }
t_clear: .byte 35, 49, 3, COL_GREEN
        .encoding "screen" {
        .textnull "stufe ok"
        }
t_over: .byte 34, 49, 3, COL_RED
        .encoding "screen" {
        .textnull "spiel ende"
        }
t_player: .byte 32, 49, 3, COL_WHITE
        .encoding "screen" {
        .textnull "spieler   bereit"
        }
t_fire: .byte 29, 49, 3, COL_WHITE
        .encoding "screen" {
        .textnull "joystick weitergeben, feuer"
        }

t_title_a: .byte 25, 21, 3, COL_LTBLUE
        .encoding "screen" {
        .textnull "ein arcade-spiel fuer mega65"
        }
t_m_players: .byte 24, 24, 3, COL_YELLOW
        .encoding "screen" {
        .textnull "spieler"
        }
t_m_mode: .byte 24, 27, 3, COL_YELLOW
        .encoding "screen" {
        .textnull "ablauf"
        }
t_m_joy: .byte 24, 30, 3, COL_YELLOW
        .encoding "screen" {
        .textnull "joystick"
        }
t_m_speed: .byte 24, 33, 3, COL_YELLOW
        .encoding "screen" {
        .textnull "tempo"
        }
t_m_fire: .byte 13, 37, 3, COL_WHITE
        .encoding "screen" {
        .textnull "hoch/runter waehlt, links/rechts aendert, feuer startet"
        }
t_m_best: .byte 24, 40, 3, COL_LTBLUE
        .encoding "screen" {
        .textnull "rekord"
        }
t_v_1:  .byte 36, 24, 3, COL_WHITE
        .encoding "screen" {
        .textnull "1"
        }
t_v_2:  .byte 36, 24, 3, COL_WHITE
        .encoding "screen" {
        .textnull "2"
        }
t_v_none: .byte 36, 27, 3, COL_DGREY
        .encoding "screen" {
        .textnull "-"
        }
t_v_alt: .byte 36, 27, 3, COL_WHITE
        .encoding "screen" {
        .textnull "abwechselnd"
        }
t_v_sim: .byte 36, 27, 3, COL_GREEN
        .encoding "screen" {
        .textnull "gleichzeitig"
        }
t_v_j1: .byte 36, 30, 3, COL_WHITE
        .encoding "screen" {
        .textnull "ein joystick, anschluss 1"
        }
t_v_j2: .byte 36, 30, 3, COL_WHITE
        .encoding "screen" {
        .textnull "ein joystick, anschluss 2"
        }
t_v_jj: .byte 36, 30, 3, COL_GREEN
        .encoding "screen" {
        .textnull "zwei joysticks"
        }
t_hs_new: .byte 33, 20, 2, COL_YELLOW
        .encoding "screen" {
        .textnull "neuer rekord!"
        }
t_hs_name: .byte 28, 23, 2, COL_WHITE
        .encoding "screen" {
        .textnull "name:"
        }
t_hs_help: .byte 22, 26, 2, COL_LTBLUE
        .encoding "screen" {
        .textnull "hoch/runter: zeichen, feuer: fertig"
        }
t_hs_saved: .byte 35, 49, 3, COL_GREEN
        .encoding "screen" {
        .textnull "gespeichert"
        }
t_hs_nosave: .byte 31, 49, 3, COL_RED
        .encoding "screen" {
        .textnull "nicht gespeichert"
        }
.const HT_P1 = 0
.const HT_P2 = 1
.const HT_LEVEL = 2
.const HT_BEST = 3
.const HT_FIELDS = 4
.const HT_OF = 5
.const HT_PAUSE = 6
.const HT_READY = 7
.const HT_HIT = 8
.const HT_CLEAR = 9
.const HT_OVER = 10
.const HT_PLAYER = 11
.const HT_FIRE = 12
.const HT_TITLE_A = 13
.const HT_M_PLAYERS = 14
.const HT_M_MODE = 15
.const HT_M_JOY = 16
.const HT_M_SPEED = 17
.const HT_M_FIRE = 18
.const HT_M_BEST = 19
.const HT_V_1 = 20
.const HT_V_2 = 21
.const HT_V_NONE = 22
.const HT_V_ALT = 23
.const HT_V_SIM = 24
.const HT_V_J1 = 25
.const HT_V_J2 = 26
.const HT_V_JJ = 27
.const HT_HS_NEW = 28
.const HT_HS_NAME = 29
.const HT_HS_HELP = 30
.const HT_HS_SAVED = 31
.const HT_HS_NOSAVE = 32
TXT_LO: .byte <t_p1, <t_p2, <t_level, <t_best, <t_fields, <t_of, <t_pause, <t_ready, <t_hit, <t_clear, <t_over, <t_player, <t_fire, <t_title_a, <t_m_players, <t_m_mode, <t_m_joy, <t_m_speed, <t_m_fire, <t_m_best, <t_v_1, <t_v_2, <t_v_none, <t_v_alt, <t_v_sim, <t_v_j1, <t_v_j2, <t_v_jj, <t_hs_new, <t_hs_name, <t_hs_help, <t_hs_saved, <t_hs_nosave
TXT_HI: .byte >t_p1, >t_p2, >t_level, >t_best, >t_fields, >t_of, >t_pause, >t_ready, >t_hit, >t_clear, >t_over, >t_player, >t_fire, >t_title_a, >t_m_players, >t_m_mode, >t_m_joy, >t_m_speed, >t_m_fire, >t_m_best, >t_v_1, >t_v_2, >t_v_none, >t_v_alt, >t_v_sim, >t_v_j1, >t_v_j2, >t_v_jj, >t_hs_new, >t_hs_name, >t_hs_help, >t_hs_saved, >t_hs_nosave

