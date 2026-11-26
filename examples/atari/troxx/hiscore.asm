; =====================================================================================================================
; hiscore.asm - Bestenliste: zehn Eintraege, Namenseingabe mit dem Joystick, Anzeige im Titelbild
; =====================================================================================================================
; Herkunft: examples/c64/troxx/hiscore.asm (Stand Commit d173cb4a), fuer den Atari angepasst (Titelbild in Modus 2, zwei Spalten, Plattenzugriff ueber CIO).
; ZWECK     Die Liste steht in HS_BUF (HS_SIZE = 77 Byte) und wird als Datei TROXX.HS auf Diskette gehalten (disk.asm), im selben Format wie bei der
;           C64-Fassung:  0..3  Kennung "TXHS", 4 Version (1), 5..74  zehn Eintraege zu 7 Byte: Name (3 Zeichen im C64-Bildschirmcode: A-Z = 1..26,
;           Leerzeichen 32), Punkte BCD (niedrig, mittel, hoch), Stufe BCD;  75, 76  Pruefsumme (Summe und Summe der mit $5A verknuepften Bytes ueber 0..74).
;           Fehlt das Laufwerk oder die Datei oder stimmt die Pruefsumme nicht, gilt die Standardliste (zehn Eintraege "TRX", 10000 bis 1000 Punkte).
; BEDIENUNG hiscore.load beim Start (setzt auch HISCORE = Punkte des ersten Platzes); hiscore.show zeichnet die Liste im Titelbild (Zeilen 18..22, zwei
;           Spalten) mit dem Ergebnis des letzten Plattenzugriffs in Zeile 23; hiscore.enter prueft einen Endstand (T0..T2 Punkte BCD niedrigstes zuerst,
;           T3 Stufe, A = Spielernummer): kommt er in die Liste, fragt es den Namen ab (Joystick: hoch/runter Buchstabe, links/rechts Stelle, Feuer fertig),
;           sortiert ein und speichert.  Mit AUTOPLAY entfaellt die Eingabe (Name "CPU").
; =====================================================================================================================

.const HS_SIZE    = 77
.const HS_ENTRIES = 10
.const HS_ENTRY   = 7
.const HS_FIRST   = 5
.const HS_LIST_ROW = 18
.const HS_LIST_COL = 2

hiscore: {

; default: Standardliste nach HS_BUF.  Veraendert A, X, Y.
default:
        ldx #4
.h:     lda HEAD,x
        sta HS_BUF,x
        dex
        bpl .h
        ldx #0                          ; Eintrag
        ldy #HS_FIRST                   ; Abstand in HS_BUF
.e:     lda #20                         ; "TRX"
        sta HS_BUF,y
        lda #18
        sta HS_BUF + 1,y
        lda #24
        sta HS_BUF + 2,y
        lda #0
        sta HS_BUF + 3,y                ; Punkte niedrig
        lda DEF_MID,x
        sta HS_BUF + 4,y
        lda DEF_HI,x
        sta HS_BUF + 5,y
        lda #1
        sta HS_BUF + 6,y                ; Stufe 1
        tya
        clc
        adc #HS_ENTRY
        tay
        inx
        cpx #HS_ENTRIES
        bne .e
        jmp makesum

; makesum: Pruefsumme ueber HS_BUF[0..74] berechnen und nach HS_BUF[75..76] schreiben.  Veraendert A, X.
makesum:
        jsr calc
        lda T0
        sta HS_BUF + HS_SIZE - 2
        lda T1
        sta HS_BUF + HS_SIZE - 1
        rts

; calc: T0 = Summe, T1 = Summe der Bytes eor $5A ueber HS_BUF[0..74].  Veraendert A, X.
calc:   lda #0
        sta T0
        sta T1
        ldx #HS_SIZE - 3
.b:     lda HS_BUF,x
        clc
        adc T0
        sta T0
        lda HS_BUF,x
        eor #$5a
        clc
        adc T1
        sta T1
        dex
        bpl .b
        rts

; load: Liste von Diskette holen (oder Standardliste), HISCORE setzen.  HS_ST = Ergebnis.  Veraendert A, X, Y, T0, T1.
load:   jsr default
        jsr disk.load
        sta HS_ST
        cmp #DK_OK
        bne .done
        ldx #3                          ; Kennung pruefen
.k:     lda HS_TMP,x
        cmp HEAD,x
        bne .bad
        dex
        bpl .k
        lda HS_TMP + 4
        cmp HEAD + 4
        bne .bad
        ldx #HS_SIZE - 1                ; in den Arbeitspuffer, dann Pruefsumme
.cp:    lda HS_TMP,x
        sta HS_BUF,x
        dex
        bpl .cp
        jsr calc
        lda T0
        cmp HS_BUF + HS_SIZE - 2
        bne .bad
        lda T1
        cmp HS_BUF + HS_SIZE - 1
        beq .done
.bad:   jsr default
        lda #DK_ERR
        sta HS_ST
.done:  jmp settop

; settop: HISCORE = Punkte des ersten Platzes.  Veraendert A.
settop: lda HS_BUF + HS_FIRST + 3
        sta HISCORE
        lda HS_BUF + HS_FIRST + 4
        sta HISCORE + 1
        lda HS_BUF + HS_FIRST + 5
        sta HISCORE + 2
        rts

; show: Liste im Titelbild zeichnen (Zeilen 18..22, Spalten 2 und 21) und das Ergebnis des letzten Plattenzugriffs (Zeile 23).  Veraendert A, X, Y, PTR_A, T2.
show:   ldx #0                          ; Eintrag 0..9
        stx HS_I
.row:   txa
        cmp #5
        bcc .l
        sbc #5                          ; zweite Spalte: Eintraege 5..9
.l:     tay
        lda ROW_LO + HS_LIST_ROW,y
        sta PTR_A
        lda ROW_HI + HS_LIST_ROW,y
        sta PTR_A + 1
        lda HS_I
        cmp #5
        lda #HS_LIST_COL
        bcc .c1
        lda #HS_LIST_COL + 19
.c1:    clc
        adc PTR_A
        sta PTR_A
        bcc .c2
        inc PTR_A + 1
.c2:    ldx HS_I
        ldy #0
        cpx #9
        bcs .ten
        lda #CH_SPACE                   ; Platz 1..9: " n"
        sta (PTR_A),y
        iny
        txa
        clc
        adc #DIG2 + 1
        sta (PTR_A),y
        bne .rk                         ; (immer)
.ten:   lda #DIG2 + 1                   ; Platz 10
        sta (PTR_A),y
        iny
        lda #DIG2
        sta (PTR_A),y
.rk:    ldx HS_I                        ; Abstand des Eintrags = 7 * Nummer
        lda MUL7,x
        tax
        ldy #3
.nm:    lda HS_BUF + HS_FIRST,x         ; Name: C64-Bildschirmcode -> Atari-Bildschirmcode
        jsr toatari
        sta (PTR_A),y
        inx
        iny
        cpy #6
        bne .nm
        ldx HS_I
        lda MUL7,x
        tax
        ldy #7
        lda HS_BUF + HS_FIRST + 5,x
        jsr hud.bcd2t
        lda HS_BUF + HS_FIRST + 4,x
        jsr hud.bcd2t
        lda HS_BUF + HS_FIRST + 3,x
        jsr hud.bcd2t
        lda #44                         ; "L"
        iny
        sta (PTR_A),y
        iny
        lda HS_BUF + HS_FIRST + 6,x
        jsr hud.bcd2t
        inc HS_I
        ldx HS_I
        cpx #HS_ENTRIES
        beq .rows
        jmp .row
.rows:  lda HS_ST                       ; Ergebnis in Zeile 23, Spalte 16 (8 Zeichen)
        asl
        asl
        asl
        tax
        ldy #0
.s:     lda ST_TEXT,x
        sta SCR + 23 * SCR_COLS + 16,y
        inx
        iny
        cpy #8
        bne .s
        rts

; toatari: A = C64-Bildschirmcode eines Namenszeichens (1..26 Buchstabe, sonst Leerzeichen) -> Atari-Bildschirmcode.  Veraendert A.
toatari:
        cmp #27
        bcs .sp
        cmp #1
        bcc .sp
        clc
        adc #32                         ; A = 33
        rts
.sp:    lda #CH_SPACE
        rts

; place: T0..T2 = Punkte.  X = Platz 0..9, bei $FF kommt der Endstand nicht in die Liste.  Veraendert A, X, Y.
place:  ldx #0
        ldy #HS_FIRST + 3               ; Punkte des Eintrags
.n:     lda HS_SC + 2
        cmp HS_BUF + 2,y
        bcc .lower
        bne .found
        lda HS_SC + 1
        cmp HS_BUF + 1,y
        bcc .lower
        bne .found
        lda HS_SC
        cmp HS_BUF,y
        beq .lower                      ; gleich: dahinter
        bcs .found
.lower: tya
        clc
        adc #HS_ENTRY
        tay
        inx
        cpx #HS_ENTRIES
        bne .n
        ldx #$ff
.found: rts

; enter: Endstand T0..T2 (Punkte), T3 (Stufe), A = Spielernummer: in die Liste einsortieren, Namen abfragen, speichern.  Veraendert alles ausser HISCORE.
enter:  sta HS_PL
        lda T0
        sta HS_SC
        lda T1
        sta HS_SC + 1
        lda T2
        sta HS_SC + 2
        lda T3
        sta HS_SC + 3
        jsr place
        cpx #$ff
        beq .out
        lda MUL7,x
        clc
        adc #HS_FIRST
        sta HS_S                        ; Abstand des neuen Eintrags
.ifdef AUTOPLAY
        lda #3                          ; Name "CPU"
        sta HS_NAME
        lda #16
        sta HS_NAME + 1
        lda #21
        sta HS_NAME + 2
.else
        jsr name_entry
.endif
        ldx #HS_FIRST + (HS_ENTRIES - 1) * HS_ENTRY - 1   ; Eintraege ab dem Platz um einen nach unten schieben
.sh:    cpx HS_S
        bcc .ins
        lda HS_BUF,x
        sta HS_BUF + HS_ENTRY,x
        dex
        jmp .sh
.ins:   ldx HS_S
        lda HS_NAME
        sta HS_BUF,x
        lda HS_NAME + 1
        sta HS_BUF + 1,x
        lda HS_NAME + 2
        sta HS_BUF + 2,x
        lda HS_SC
        sta HS_BUF + 3,x
        lda HS_SC + 1
        sta HS_BUF + 4,x
        lda HS_SC + 2
        sta HS_BUF + 5,x
        lda HS_SC + 3
        sta HS_BUF + 6,x
        jsr makesum
        jsr settop
        ldx #HT_SAVING
        jsr hud.msg
        jsr disk.save
        sta HS_ST
.ifndef AUTOPLAY
        ldx #100
        jsr delay
.endif
.out:   rts

.ifndef AUTOPLAY
; name_entry: Namen (3 Zeichen) mit dem Joystick eingeben: hoch/runter Buchstabe A-Z oder Leerzeichen, links/rechts Stelle, Feuer fertig.
;   Ergebnis in HS_NAME.  Veraendert A, X, Y.
name_entry:
        lda #1
        sta SPOFF
        jsr antic.use_title             ; Anzeige mit Textzeilen (Modus 2) im Titelbild-Rahmen
        lda RC_P2 + 9                   ; ohne die Farbbaender des Titelbilds: alle Zeilen wie Zeile 9
        ldx #23
.pc:    sta RC_P2,x
        dex
        bpl .pc
        jsr antic.clear
        jsr scroll_init
        ldx #HT_NEWHS
        jsr hud.print_all
        lda NPLAYERS                    ; "player n" nur bei zwei Spielern
        cmp #2
        bne .np
        ldx #HT_PLNAME
        jsr hud.print
        lda HS_PL
        clc
        adc #DIG2 + 1
        sta SCR + 10 * SCR_COLS + 20
.np:    lda #1
        sta HS_NAME
        sta HS_NAME + 1
        sta HS_NAME + 2
        lda #0
        sta HS_CPOS
        ldx #0
        jsr fire_wait_any               ; Feuer des Spiels loslassen
.l:     ldx #2                          ; Name und Zeiger darstellen
.d:     lda HS_NAME,x
        jsr toatari
        sta SCR + 14 * SCR_COLS + 18,x
        lda #CH_SPACE
        cpx HS_CPOS
        bne .c
        lda #62                         ; Pfeil nach oben unter der Stelle
.c:     sta SCR + 15 * SCR_COLS + 18,x
        dex
        bpl .d
        jsr tick
        jsr input.read_any
        sta T7
        and #IN_FIRE
        bne .done
        lda T7
        and #IN_UP
        beq .nu
        ldx HS_CPOS
        lda HS_NAME,x
        cmp #32
        beq .toA                        ; nach dem Leerzeichen kommt A
        cmp #26
        bcs .toS                        ; nach Z das Leerzeichen
        clc
        adc #1
        bne .set
.toA:   lda #1
        bne .set
.toS:   lda #32
        bne .set
.nu:    lda T7
        and #IN_DOWN
        beq .nd
        ldx HS_CPOS
        lda HS_NAME,x
        cmp #1
        beq .toS2
        cmp #32
        beq .toZ
        sec
        sbc #1
        bne .set
.toS2:  lda #32
        bne .set
.toZ:   lda #26
.set:   sta HS_NAME,x
        jmp .rep
.nd:    lda T7
        and #IN_LEFT
        beq .nl
        lda HS_CPOS
        beq .rep
        dec HS_CPOS
        jmp .rep
.nl:    lda T7
        and #IN_RIGHT
        bne .rt
        jmp .l
.rt:    lda HS_CPOS
        cmp #2
        bcs .rep
        inc HS_CPOS
.rep:   ldx #8
        jsr delay
        jmp .l
.done:  ldx #0
        jsr fire_wait_any
        rts
.endif
}

HEAD:    .byte $54, $58, $48, $53, 1    ; "TXHS", Version 1
DEF_MID: .byte $00, $90, $80, $70, $60, $50, $40, $30, $20, $10
DEF_HI:  .byte $01, 0, 0, 0, 0, 0, 0, 0, 0, 0
MUL7:    .byte 0, 7, 14, 21, 28, 35, 42, 49, 56, 63
; Ergebnis des Plattenzugriffs (je 8 Zeichen): ok, kein Laufwerk, keine Datei, Fehler
ST_TEXT:
        .encoding "screen" {
        .text "DISK OK "
        .text "NO DRIVE"
        .text "NO FILE "
        .text "DISK ERR"
        }
