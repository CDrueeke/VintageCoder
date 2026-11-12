// BASIC→Assembler, Laufzeit C64, Stufe 2: Zahlenausgabe, INPUT, GET mit Fließkomma
// Verlangt: rt_core.asm, rt_fp.asm

.ifdef B2A_ATARI
// PRINT einer Zahl wie Atari-BASIC: Text des OS-Pakets (FASC), ohne Vorzeichenstelle und ohne abschließendes Leerzeichen; ein Punkt am Ende entfällt. Wert in FR0
// Wie im BASIC-Modul: ist das erste Ziffernpaar >= 10, fällt die zehnte Ziffer weg (höchstens 9 gültige Stellen)
// Turbo BASIC XL (RT_P10) druckt immer alle zehn Stellen
rt_print_f:
.ifndef RT_P10
          lda FR0+1
          cmp #$10
          bcc .go
          lda FR0+5
          and #$F0
          sta FR0+5
.endif
.go:      jsr FASC
          ldy #0
.l:       lda (INBUFF),y
          bmi .last
          jsr CHROUT
          iny
          bne .l
.last:    and #$7f
          cmp #'.'
          beq .e
          jmp CHROUT
.e:       rts

// INPUT einer Zahl: gibt "? " aus, liest eine Zeile. Ergebnis im FAC (AFP); Carry gesetzt = leere Eingabe (Variable bleibt)
rt_input_f:
.ask:     lda #'?'
          jsr CHROUT
          lda #' '
          jsr CHROUT
          ldx #0
.rd:      jsr CHRIN
          cmp #13
          beq .eol
          cpx #80
          bcs .rd
          sta rt_buf,x
          inx
          bne .rd
.eol:     lda #13
          jsr CHROUT
          lda #0
          sta rt_buf,x
          ldx #0                   // leere Eingabe (nur Leerzeichen)?
.bl:      lda rt_buf,x
          beq .empty
          inx
          cmp #' '
          beq .bl
          lda #<rt_buf
          sta INBUFF
          lda #>rt_buf
          sta INBUFF+1
          lda #0
          sta CIX
          jsr AFP
          bcs .redo
          ldy CIX                  // danach nur noch Leerzeichen
.tr:      lda rt_buf,y
          beq .ok
          cmp #' '
          bne .redo
          iny
          bne .tr
.ok:      clc
          rts
.empty:   sec
          rts
.redo:    lda #<rt_s_redo
          ldy #>rt_s_redo
          jsr rt_print_str
          jmp .ask
.else
// PRINT einer Zahl: Text aus dem ROM (FOUT: Vorzeichenstelle, Ziffern), danach ein Leerzeichen. Wert im FAC
rt_print_f: jsr F_FOUT
          jsr rt_print_str
          lda #' '
          jmp CHROUT

// INPUT einer Zahl: gibt "? " aus, liest eine Zeile. Ergebnis im FAC; Carry gesetzt = leere Eingabe (Variable bleibt)
rt_input_f:
.ask:     lda #'?'
          jsr CHROUT
          lda #' '
          jsr CHROUT
          ldx #0
.rd:      jsr CHRIN
          cmp #13
          beq .eol
          cpx #80
          bcs .rd
          sta rt_buf,x
          inx
          bne .rd
.eol:     lda #13
          jsr CHROUT
          lda #0
          sta rt_buf,x
          ldx #0                   // prüfen: nur Zeichen einer Zahl, mindestens eine Ziffer
          stx rt_tmp
.chk:     lda rt_buf,x
          beq .done
          inx
          cmp #' '
          beq .chk
          cmp #'+'
          beq .chk
          cmp #'-'
          beq .chk
          cmp #'.'
          beq .chk
          cmp #'E'
          beq .chk
          cmp #'0'
          bcc .redo
          cmp #'9'+1
          bcs .redo
          inc rt_tmp
          jmp .chk
.done:    lda rt_buf
          bne .some
          sec                      // leere Eingabe
          rts
.some:    lda rt_tmp
          beq .redo
          lda F_TXTPTR             // BASIC-Textzeiger sichern, auf den Puffer setzen
          sta rt_txt
          lda F_TXTPTR+1
          sta rt_txt+1
          lda #<rt_buf
          sta F_TXTPTR
          lda #>rt_buf
          sta F_TXTPTR+1
          jsr F_CHRGOT
          jsr F_FIN
          lda rt_txt
          sta F_TXTPTR
          lda rt_txt+1
          sta F_TXTPTR+1
          clc
          rts
.redo:    lda #<rt_s_redo
          ldy #>rt_s_redo
          jsr rt_print_str
          jmp .ask

.endif

// GET einer Zahl: Taste 0-9 -> Wert, keine Taste -> 0, sonst Fehler (A/Y)
rt_get_int: jsr GETIN
          cmp #0
          beq .none
          sec
          sbc #'0'
          cmp #10
          bcs rt_e_syntax
          ldy #0
          rts
.none:    ldy #0
          rts

rt_s_redo: .byte 63
          .petscii "redo from start"
          .byte 13, 0
rt_buf:   .fill 82, 0
