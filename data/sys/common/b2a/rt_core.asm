// BASIC→Assembler, Laufzeit C64: Kern (immer eingebunden)
// Start, Ende, Fehler, Ausdrucksstapel, Textausgabe.
// Konvention: Werte stehen als 16-Bit-Zahl in A (niedrig) und Y (hoch).
// Zweistellige Operatoren: linker Operand liegt auf dem Ausdrucksstapel, rechter in A/Y; Ergebnis in A/Y.

// Fehlernummern für rt_error (Index in die Meldungstabelle in rt_core.asm)
.const ERR_ILLEGAL = 0
.const ERR_DIV0    = 1
.const ERR_DATA    = 2
.const ERR_SUBSCR  = 3
.const ERR_NEXT    = 4
.const ERR_RETURN  = 5
.const ERR_MEM     = 6
.const ERR_FORMULA = 7
.const ERR_SYNTAX  = 8
.const ERR_STRLONG = 9
.const ERR_UNDEF   = 10
.const ERR_OVERFLOW = 11   // nur Atari (Fließkomma-Paket meldet Überlauf)
.const ERR_ALEN = 12       // nur Atari: Teilzeichenkette außerhalb (Fehler 5)
.const ERR_ADIM = 13       // nur Atari: DIM fehlt oder doppelt (Fehler 9)
.const ERR_AVAL = 14       // nur Atari: VAL ohne Zahl (Fehler 18)
.const ERR_AINP = 15       // nur Atari: kein Zahlenwert beim Lesen (Fehler 8)

.ifdef RT_ZSAVE
.ifdef RT_FLOAT
// Fehler der ROM-Routinen (DIVISION BY ZERO usw.) springen nicht zu uns zurück, sondern ins BASIC: der Fehlervektor $0300 wird für die
// Dauer des Programms umgebogen, damit zuvor der Nullseitenblock zurückgeschrieben wird (X = Fehlernummer bleibt erhalten).
rt_oldierr: .word 0
rt_errhook: stx rt_tmp
          ldx #RT_ZLEN-1
.z:       lda rt_zsave,x
          sta RT_ZLO,x
          dex
          bpl .z
          lda rt_oldierr
          sta $0300
          lda rt_oldierr+1
          sta $0301
          ldx rt_tmp
          jmp (rt_oldierr)
.endif
// Ziele, deren Nullseite das BASIC nach dem Programm noch braucht (C128, Plus/4): der benutzte Block wird beim Start gesichert und bei Ende zurückgeschrieben
rt_zsave: .fill RT_ZLEN, 0
.endif

rt_init:  tsx
          inx                  // JSR-Rücksprung von rt_init abziehen: SP wie beim SYS-Aufruf
          inx
          stx rt_sp0
.ifdef RT_ZSAVE
          ldx #RT_ZLEN-1
.z:       lda RT_ZLO,x
          sta rt_zsave,x
          dex
          bpl .z
.ifdef RT_FLOAT
          lda $0300
          sta rt_oldierr
          lda $0301
          sta rt_oldierr+1
          lda #<rt_errhook
          sta $0300
          lda #>rt_errhook
          sta $0301
.endif
.endif
          lda #0
          sta rt_esp
          sta rt_ssp
          sta rt_line
          sta rt_line+1
          rts

// Programmende (END, Programmende): zurück ins BASIC
rt_end:
.ifdef RT_ZSAVE
.ifdef RT_FLOAT
          lda rt_oldierr
          sta $0300
          lda rt_oldierr+1
          sta $0301
.endif
          ldx #RT_ZLEN-1
.z:       lda rt_zsave,x
          sta RT_ZLO,x
          dex
          bpl .z
.endif
          ldx rt_sp0
          txs
          rts

// STOP: A/Y = Zeilennummer
rt_stop:  sta rt_t0
          sty rt_t0+1
          lda #13
          jsr CHROUT
          lda #<rt_s_break
          ldy #>rt_s_break
          jsr rt_print_str
          lda rt_t0
          ldy rt_t0+1
          jsr rt_print_dec
          lda #13
          jsr CHROUT
          jmp rt_end

// Fehler: X = Fehlernummer. Ausgabe wie der Interpreter: ?TEXT  ERROR IN zeile
rt_error: lda #<rt_msgs
          sta rt_t0
          lda #>rt_msgs
          sta rt_t0+1
.skip:    cpx #0
          beq .found
          ldy #0
.sk1:     lda (rt_t0),y
          iny
          cmp #0
          bne .sk1
          tya
          clc
          adc rt_t0
          sta rt_t0
          bcc .sk2
          inc rt_t0+1
.sk2:     dex
          jmp .skip
.found:   lda #13
          jsr CHROUT
          lda #'?'
          jsr CHROUT
          lda rt_t0
          ldy rt_t0+1
          jsr rt_print_str
          lda #<rt_s_err
          ldy #>rt_s_err
          jsr rt_print_str
          lda rt_line
          ora rt_line+1
          beq .nl
          lda #<rt_s_in
          ldy #>rt_s_in
          jsr rt_print_str
          lda rt_line
          ldy rt_line+1
          jsr rt_print_dec
.nl:      lda #13
          jsr CHROUT
          jmp rt_end

rt_e_illegal: ldx #ERR_ILLEGAL
          jmp rt_error
rt_e_div0:    ldx #ERR_DIV0
          jmp rt_error
rt_e_data:    ldx #ERR_DATA
          jmp rt_error
rt_e_sub:     ldx #ERR_SUBSCR
          jmp rt_error
rt_e_next:    ldx #ERR_NEXT
          jmp rt_error
rt_e_ret:     ldx #ERR_RETURN
          jmp rt_error
rt_e_mem:     ldx #ERR_MEM
          jmp rt_error
rt_e_formula: ldx #ERR_FORMULA
          jmp rt_error
rt_e_syntax:  ldx #ERR_SYNTAX
          jmp rt_error
rt_e_strlong: ldx #ERR_STRLONG
          jmp rt_error
rt_e_undef:   ldx #ERR_UNDEF
          jmp rt_error
.ifdef B2A_ATARI
rt_e_overflow: ldx #ERR_OVERFLOW
          jmp rt_error
rt_e_alen:    ldx #ERR_ALEN
          jmp rt_error
rt_e_adim:    ldx #ERR_ADIM
          jmp rt_error
rt_e_aval:    ldx #ERR_AVAL
          jmp rt_error
rt_e_ainput:  ldx #ERR_AINP
          jmp rt_error
.endif

// ---- Ausdrucksstapel (16 Einträge à 2 Byte)
rt_push:  ldx rt_esp
          cpx #32
          bcs rt_e_formula
          sta rt_es,x
          inx
          tya
          sta rt_es,x
          inx
          stx rt_esp
          rts

// oberster Eintrag -> rt_t0
rt_pop0:  ldx rt_esp
          dex
          lda rt_es,x
          sta rt_t0+1
          dex
          lda rt_es,x
          sta rt_t0
          stx rt_esp
          rts

// ---- Textausgabe
rt_print_nl:  lda #13
          jmp CHROUT

// A/Y = Zeiger auf nullterminierten Text
rt_print_str: sta rt_t0
          sty rt_t0+1
.next:    ldy #0
          lda (rt_t0),y
          beq .done
          jsr CHROUT
          inc rt_t0
          bne .next
          inc rt_t0+1
          jmp .next
.done:    rts

// CHR$(n): A/Y = Zeichencode 0..255
rt_print_chr: cpy #0
          bne rt_e_illegal
          jmp CHROUT

// SPC(n)
rt_spc:   cpy #0
          bne rt_e_illegal
          sta rt_tmp
.l:       lda rt_tmp
          beq .d
          lda #' '
          jsr CHROUT
          dec rt_tmp
          jmp .l
.d:       rts

// TAB(n): bis Spalte n (0-basiert) mit Leerzeichen auffüllen
rt_tab:   cpy #0
          bne rt_e_illegal
          sta rt_t1
          sec
          jsr PLOT
          sty rt_tmp
          lda rt_t1
          sec
          sbc rt_tmp
          bcc .d
          sta rt_tmp
.l:       lda rt_tmp
          beq .d
          lda #' '
          jsr CHROUT
          dec rt_tmp
          jmp .l
.d:       rts

// "," im PRINT: zur nächsten Zehnerspalte
rt_comma: sec
          jsr PLOT
          tya
.m:       sec
          sbc #10
          bcs .m
          eor #$ff
          clc
          adc #1
          sta rt_tmp
.l:       lda rt_tmp
          beq .d
          lda #' '
          jsr CHROUT
          dec rt_tmp
          jmp .l
.d:       rts

// Zahl ohne Vorzeichen und ohne Leerzeichen ausgeben (für Zeilennummern in Meldungen): A/Y = Wert
rt_print_dec: sta rt_t1
          sty rt_t1+1
          lda #0
          sta rt_tmp
          ldx #0
.digit:   ldy #0
.sub:     sec
          lda rt_t1
          sbc rt_pow_lo,x
          sta rt_v
          lda rt_t1+1
          sbc rt_pow_hi,x
          bcc .done
          sta rt_t1+1
          lda rt_v
          sta rt_t1
          iny
          jmp .sub
.done:    tya
          bne .out
          cpx #4
          beq .out
          lda rt_tmp
          beq .skip
          tya
.out:     pha
          lda #1
          sta rt_tmp
          pla
          clc
          adc #'0'
          jsr CHROUT
.skip:    inx
          cpx #5
          bne .digit
          rts

rt_pow_lo: .byte <10000, <1000, <100, <10, <1
rt_pow_hi: .byte >10000, >1000, >100, >10, >1

// ---- Meldungen (Großbuchstaben im C64-Zeichensatz: .petscii mit Kleinbuchstaben)
rt_s_break: .petscii "break in "
          .byte 0
rt_s_err: .petscii "  error"
          .byte 0
rt_s_in:  .petscii " in "
          .byte 0
rt_msgs:  .petscii "illegal quantity"
          .byte 0
          .petscii "division by zero"
          .byte 0
          .petscii "out of data"
          .byte 0
          .petscii "bad subscript"
          .byte 0
          .petscii "next without for"
          .byte 0
          .petscii "return without gosub"
          .byte 0
          .petscii "out of memory"
          .byte 0
          .petscii "formula too complex"
          .byte 0
          .petscii "syntax"
          .byte 0
          .petscii "string too long"
          .byte 0
          .petscii "undef'd function"
          .byte 0
.ifdef B2A_ATARI
          .petscii "overflow"
          .byte 0
          .petscii "string length"
          .byte 0
          .petscii "dimension"
          .byte 0
          .petscii "bad value"
          .byte 0
          .petscii "input"
          .byte 0
.endif

rt_es:    .fill 32, 0
