// BASIC→Assembler (basic2asm), Atari 8 Bit: Grafik, Ton und Joystick wie in Atari-BASIC, über CIO (Kanal 6 = S:) und die Hardwareregister.
//   GRAPHICS n     Kanal 6 schließen und S: mit Modus n öffnen (n + 16 ohne Textfenster, n + 32 ohne Löschen)
//   POSITION x,y   Cursor setzen (COLCRS, ROWCRS)       PLOT x,y   Cursor setzen, Punkt mit der Farbe von COLOR setzen
//   DRAWTO x,y     Linie vom letzten Punkt zum neuen    COLOR n    Farbwert für PLOT und DRAWTO     LOCATE x,y,v  Farbwert eines Punktes
//   SETCOLOR r,f,h Farbregister r (0-4) = f * 16 + h (Schattenregister COLOR0)
//   SOUND v,t,v,l  Stimme 0-3: Frequenz, Verzerrung * 16 + Lautstärke; AUDCTL = 0, SKCTL = 3
//   STICK(n), STRIG(n) Schattenregister STICK0 und STRIG0
// Argumente: alle bis auf das letzte auf dem Wortestapel (rt_push), das letzte in A/Y.
// Verlangt: rt_core.asm, rt_os.asm (CIOV). Fehler des OS (Y >= $80 nach CIO) werden zu ILLEGAL QUANTITY.

.const RT_IOCB6 = $60
.const RT_AUDF1 = $D200   // POKEY (eigene Namen: die Laufzeit bindet pokey.asm nicht ein)
.const RT_AUDC1 = $D201
.const RT_AUDCTL = $D208
.const RT_SKCTL = $D20F
.const CIO_OPEN   = $03
.const CIO_DRAW   = $11
.const ATACHR     = $02FB

// Fehler des OS (A = Y >= $80): "?DEVICE ERROR <Nummer> IN <Zeile>" mit der Atari-Fehlernummer (z. B. 170 Datei nicht gefunden, 136 Dateiende)
rt_cio_err: bpl .ok
          sta rt_cv_n
          lda #13
          jsr CHROUT
          lda #'?'
          jsr CHROUT
          lda #<rt_s_dev
          ldy #>rt_s_dev
          jsr rt_print_str
          lda rt_cv_n
          ldy #0
          jsr rt_print_dec
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
.ok:      rts
rt_s_dev: .petscii "device error "
          .byte 0

// GRAPHICS n (A/Y)
rt_gr:    sta rt_gv
          ldx #RT_IOCB6
          lda #CIO_CLOSE
          sta ICCOM,x
          jsr CIOV
          ldx #RT_IOCB6
          lda #CIO_OPEN
          sta ICCOM,x
          lda #<rt_gdev
          sta ICBAL,x
          lda #>rt_gdev
          sta ICBAL+1,x
          lda rt_gv
          and #$30
          ora #$0C
          sta ICAX1,x
          lda rt_gv
          and #$0F
          sta ICAX2,x
          jsr CIOV
          tya
          jmp rt_cio_err

// POSITION x,y: x auf dem Stapel, y in A/Y
rt_pos:   sta ROWCRS
          jsr rt_pop0
          lda rt_t0
          sta COLCRS
          lda rt_t0+1
          sta COLCRS+1
          rts

// COLOR n
rt_color: sta rt_gcol
          rts

// ein Byte mit der Farbe nach Kanal 6 schreiben
rt_plot:  jsr rt_pos
          ldx #RT_IOCB6
          lda #CIO_PUTCHR
          sta ICCOM,x
          lda #0
          sta ICBLL,x
          sta ICBLL+1,x
          lda rt_gcol
          jsr CIOV
          tya
          jmp rt_cio_err

rt_drawto: jsr rt_pos
          lda rt_gcol
          sta ATACHR
          ldx #RT_IOCB6
          lda #CIO_DRAW
          sta ICCOM,x
          lda #$0C
          sta ICAX1,x
          lda #0
          sta ICAX2,x
          jsr CIOV
          tya
          jmp rt_cio_err

// Turbo BASIC XL: FCOLOR n (Füllfarbe, FILDAT $02FD) und FILLTO x,y (Linie vom letzten Punkt nach x,y füllen, XIO 18)
rt_fcolor: sta rt_gfc
          sta $02FD
          rts
rt_fillto: jsr rt_pos
          lda rt_gcol
          sta ATACHR
          lda rt_gfc
          sta $02FD
          ldx #RT_IOCB6
          lda #$12
          sta ICCOM,x
          lda #$0C
          sta ICAX1,x
          lda #0
          sta ICAX2,x
          jsr CIOV
          tya
          jmp rt_cio_err
rt_gfc:   .byte 0

// LOCATE x,y,v: Farbwert des Punktes -> A
rt_locate: jsr rt_pos
          ldx #RT_IOCB6
          lda #CIO_GETCHR
          sta ICCOM,x
          lda #0
          sta ICBLL,x
          sta ICBLL+1,x
          jsr CIOV
          sta rt_gv
          tya
          jsr rt_cio_err
          lda rt_gv
          rts

// SETCOLOR r,f,h: r und f auf dem Stapel, h in A/Y
rt_setcolor: sta rt_t1
          jsr rt_pop0              // f
          lda rt_t0
          asl
          asl
          asl
          asl
          ora rt_t1
          sta rt_gv
          jsr rt_pop0              // r
          lda rt_t0+1
          bne .bad
          ldx rt_t0
          cpx #5
          bcs .bad
          lda rt_gv
          sta COLOR0,x
          rts
.bad:     jmp rt_e_illegal

// SOUND v,t,d,l: v, t und d auf dem Stapel, l in A/Y
rt_sound: sta rt_t1                // Lautstärke
          jsr rt_pop0              // Verzerrung
          lda rt_t0
          asl
          asl
          asl
          asl
          ora rt_t1
          sta rt_gv
          jsr rt_pop0              // Frequenz
          lda rt_t0
          sta rt_gf
          jsr rt_pop0              // Stimme
          lda rt_t0+1
          bne .bad
          lda rt_t0
          cmp #4
          bcs .bad
          asl
          tax
          lda rt_gf
          sta RT_AUDF1,x
          lda rt_gv
          sta RT_AUDC1,x
          lda #0
          sta RT_AUDCTL
          lda #3
          sta RT_SKCTL
          rts
.bad:     jmp rt_e_illegal

// STICK(n) / STRIG(n): n in A/Y (0-3)
rt_stick: cpy #0
          bne .bad
          cmp #4
          bcs .bad
          tax
          lda STICK0,x
          and #$0F
          ldy #0
          rts
.bad:     jmp rt_e_illegal
rt_strig: cpy #0
          bne .bad
          cmp #4
          bcs .bad
          tax
          lda STRIG0,x
          and #$01
          ldy #0
          rts
.bad:     jmp rt_e_illegal

rt_gdev:  .byte 'S', ':', $9B
rt_gv:    .byte 0
rt_gf:    .byte 0
rt_gcol:  .byte 0

// ---- Kanäle (CIO): OPEN #n,aux1,aux2,"name"; CLOSE #n; PUT #n,b; GET #n,v; Turbo BASIC XL: BPUT/BGET #n,adresse,länge
// Kanalnummer 0-7 -> X = Kanalnummer * 16; Fehler des OS (Y >= $80) werden zu ILLEGAL QUANTITY
.const CIO_CLOSE2 = $0C
.const CIO_GETCH  = $07
.const CIO_PUTCH  = $0B

// A (0-7) -> X = A * 16
rt_chx:   and #$07
          asl
          asl
          asl
          asl
          tax
          rts

// CLOSE #n: A = Kanal (Fehler werden wie in Atari-BASIC ignoriert)
rt_close: jsr rt_chx
          lda #CIO_CLOSE2
          sta ICCOM,x
          jmp CIOV

// PUT #n,b: Kanal auf dem Stapel, Byte in A
rt_put:   sta rt_cv_n
          jsr rt_pop0
          lda rt_t0
          jsr rt_chx
          lda #CIO_PUTCH
          sta ICCOM,x
          lda #0
          sta ICBLL,x
          sta ICBLL+1,x
          lda rt_cv_n
          jsr CIOV
          tya
          jmp rt_cio_err

// GET #n,v: A = Kanal -> A = Byte
rt_getc:  jsr rt_chx
          lda #CIO_GETCH
          sta ICCOM,x
          lda #0
          sta ICBLL,x
          sta ICBLL+1,x
          jsr CIOV
          sta rt_cv_n
          tya
          jsr rt_cio_err
          lda rt_cv_n
          rts

// BPUT/BGET #n,adresse,länge: Kanal und Adresse auf dem Stapel (Adresse oben), Länge in A/Y
rt_bput:  ldx #CIO_PUTCH
          bne rt_blk
rt_bget:  ldx #CIO_GETCH
rt_blk:   stx rt_cv_c
          sta rt_cv_l
          sty rt_cv_l+1
          jsr rt_pop0
          lda rt_t0
          sta rt_cv_a1
          lda rt_t0+1
          sta rt_cv_a2
          jsr rt_pop0
          lda rt_t0
          jsr rt_chx
          lda rt_cv_c
          sta ICCOM,x
          lda rt_cv_a1
          sta ICBAL,x
          lda rt_cv_a2
          sta ICBAL+1,x
          lda rt_cv_l
          sta ICBLL,x
          lda rt_cv_l+1
          sta ICBLL+1,x
          jsr CIOV
          tya
          jmp rt_cio_err

rt_cv_n:  .byte 0
rt_cv_c:  .byte 0
rt_cv_a1: .byte 0
rt_cv_a2: .byte 0
rt_cv_l:  .word 0
rt_fnbuf: .fill 64, 0
