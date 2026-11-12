// BASIC→Assembler (basic2asm), Atari 8 Bit, Stufe 3: Zeichenketten nach den Regeln von Atari-BASIC (gemessen am echten BASIC-Modul in atari800).
// Jede Zeichenkettenvariable ist ein fester Puffer: Kopf (Länge, Kapazität; je 16 Bit), danach die Zeichen. DIM A$(n) setzt die Kapazität (zweimal: Fehler 9),
// ohne DIM ist die Kapazität 0 und jede Zuweisung ein Fehler 9. Zuweisungen schneiden still auf die Kapazität ab. Teilzeichenketten A$(n) und A$(n,m)
// zählen ab 1; Lesen verlangt 1 <= n <= m <= LEN, Schreiben A$(n,m)=X ersetzt höchstens m-n+1 Zeichen (die Länge wächst nur bis n-1+Anzahl), A$(n)=X setzt
// die Länge auf n-1+Anzahl. Es gibt kein +; Vergleiche sind lexikographisch nach Bytewert, das Ergebnis (wahr) ist -1 wie bei den Zahlen.
// Der Akkumulator (rt_sal = Länge, rt_sap = Zeiger) zeigt immer in einen Puffer, ein Literal oder rt_tbuf; Zwischenwerte liegen auf dem Wortestapel (rt_push).
// Verlangt: rt_core.asm, rt_fp.asm (Fließkomma ist bei Zeichenketten immer an), rt_iof.asm (rt_buf)

// Kopf der Variablen (A/Y) -> Akkumulator
rt_sload: sta rt_s1
          sty rt_s1+1
          ldy #0
          lda (rt_s1),y
          sta rt_sal
          iny
          lda (rt_s1),y
          sta rt_sal+1
          clc
          lda rt_s1
          adc #4
          sta rt_sap
          lda rt_s1+1
          adc #0
          sta rt_sap+1
          rts

// Akkumulator auf den Wortestapel (Länge, Zeiger)
rt_spush: lda rt_sal
          ldy rt_sal+1
          jsr rt_push
          lda rt_sap
          ldy rt_sap+1
          jmp rt_push

// Zeiger und Länge vom Stapel -> rt_pp, rt_pl
rt_spop2: jsr rt_pop0
          lda rt_t0
          sta rt_pp
          lda rt_t0+1
          sta rt_pp+1
          jsr rt_pop0
          lda rt_t0
          sta rt_pl
          lda rt_t0+1
          sta rt_pl+1
          rts

// rt_cn Bytes von (rt_s1) nach (rt_s2) kopieren; liegt das Ziel hinter der Quelle, rückwärts (überlappende Teilzeichenketten)
rt_copyn: lda rt_cn
          ora rt_cn+1
          beq .z
          lda rt_s1
          cmp rt_s2
          lda rt_s1+1
          sbc rt_s2+1
          bcc .back
          ldy #0
.f1:      lda (rt_s1),y
          sta (rt_s2),y
          inc rt_s1
          bne .f2
          inc rt_s1+1
.f2:      inc rt_s2
          bne .f3
          inc rt_s2+1
.f3:      lda rt_cn
          bne .f4
          dec rt_cn+1
.f4:      dec rt_cn
          lda rt_cn
          ora rt_cn+1
          bne .f1
.z:       rts
.back:    clc
          lda rt_s1
          adc rt_cn
          sta rt_s1
          lda rt_s1+1
          adc rt_cn+1
          sta rt_s1+1
          clc
          lda rt_s2
          adc rt_cn
          sta rt_s2
          lda rt_s2+1
          adc rt_cn+1
          sta rt_s2+1
.b1:      lda rt_s1
          bne .b2
          dec rt_s1+1
.b2:      dec rt_s1
          lda rt_s2
          bne .b3
          dec rt_s2+1
.b3:      dec rt_s2
          ldy #0
          lda (rt_s1),y
          sta (rt_s2),y
          lda rt_cn
          bne .b4
          dec rt_cn+1
.b4:      dec rt_cn
          lda rt_cn
          ora rt_cn+1
          bne .b1
          rts

// DIM A$(n): Kopf in A/Y, Kapazität in rt_t0 (nur einmal)
rt_adim:  sta rt_s1
          sty rt_s1+1
          ldy #2
          lda (rt_s1),y
          iny
          ora (rt_s1),y
          bne rt_e_adim
          lda #0
          ldy #0
          sta (rt_s1),y
          iny
          sta (rt_s1),y
          iny
          lda rt_t0
          sta (rt_s1),y
          iny
          lda rt_t0+1
          sta (rt_s1),y
          rts

// A$ = Akkumulator: Kopf in A/Y; auf die Kapazität abschneiden
rt_sstor: sta rt_s2
          sty rt_s2+1
          ldy #2
          lda (rt_s2),y
          sta rt_t0
          iny
          lda (rt_s2),y
          sta rt_t0+1
          ora rt_t0
          beq rt_e_adim
          lda rt_sal+1
          cmp rt_t0+1
          bcc .keep
          bne .cap
          lda rt_sal
          cmp rt_t0
          bcc .keep
.cap:     lda rt_t0
          sta rt_sal
          lda rt_t0+1
          sta rt_sal+1
.keep:    ldy #0
          lda rt_sal
          sta (rt_s2),y
          iny
          lda rt_sal+1
          sta (rt_s2),y
          lda rt_sal
          sta rt_cn
          lda rt_sal+1
          sta rt_cn+1
          clc
          lda rt_s2
          adc #4
          sta rt_s2
          lda rt_s2+1
          adc #0
          sta rt_s2+1
          lda rt_sap
          sta rt_s1
          lda rt_sap+1
          sta rt_s1+1
          jmp rt_copyn

// Teilzeichenkette lesen A$(n) / A$(n,m): n auf dem Stapel, m in A/Y ($FFFF = nicht angegeben), rt_ap = Kopf
rt_ssub:  sta rt_am
          sty rt_am+1
          jsr rt_pop0
          lda rt_t0
          sta rt_an
          lda rt_t0+1
          sta rt_an+1
          ldy #0
          lda (rt_ap),y
          sta rt_al
          iny
          lda (rt_ap),y
          sta rt_al+1
          lda rt_am
          and rt_am+1
          cmp #$FF
          bne .have
          lda rt_al
          sta rt_am
          lda rt_al+1
          sta rt_am+1
.have:    lda rt_an
          ora rt_an+1
          beq rt_e_alen            // n >= 1
          lda rt_am
          cmp rt_an
          lda rt_am+1
          sbc rt_an+1
          bcc rt_e_alen            // n <= m
          lda rt_al
          cmp rt_am
          lda rt_al+1
          sbc rt_am+1
          bcc rt_e_alen            // m <= LEN
          sec
          lda rt_am
          sbc rt_an
          sta rt_sal
          lda rt_am+1
          sbc rt_an+1
          sta rt_sal+1
          inc rt_sal
          bne .a
          inc rt_sal+1
.a:       clc                      // Zeiger = Kopf + 4 + n - 1
          lda rt_ap
          adc rt_an
          sta rt_sap
          lda rt_ap+1
          adc rt_an+1
          sta rt_sap+1
          clc
          lda rt_sap
          adc #3
          sta rt_sap
          bcc .b
          inc rt_sap+1
.b:       rts

// A$(n) = X / A$(n,m) = X: X liegt auf dem Wortestapel (unter n), n danach, m in A/Y ($FFFF = nicht angegeben), rt_ap = Kopf
rt_sstsub: sta rt_am
          sty rt_am+1
          jsr rt_pop0
          lda rt_t0
          sta rt_an
          lda rt_t0+1
          sta rt_an+1
          jsr rt_spop2             // Quelle -> rt_pp (Zeiger), rt_pl (Länge)
          ldy #0
          lda (rt_ap),y
          sta rt_al
          iny
          lda (rt_ap),y
          sta rt_al+1
          iny
          lda (rt_ap),y
          sta rt_ac
          iny
          lda (rt_ap),y
          sta rt_ac+1
          lda rt_an
          ora rt_an+1
          beq rt_e_alen            // n >= 1
          lda rt_am
          and rt_am+1
          cmp #$FF
          bne .rng
          lda rt_ac                // A$(n) = X: m = Kapazität, n <= Kapazität
          sta rt_am
          lda rt_ac+1
          sta rt_am+1
          lda #0
          sta rt_aflag
          beq .chk
.rng:     lda #1
          sta rt_aflag
.chk:     lda rt_am
          cmp rt_an
          lda rt_am+1
          sbc rt_an+1
          bcc rt_e_alen            // n <= m
          lda rt_ac
          cmp rt_am
          lda rt_ac+1
          sbc rt_am+1
          bcc rt_e_alen            // m <= Kapazität
          sec                      // Anzahl = min(LEN(X), m-n+1)
          lda rt_am
          sbc rt_an
          sta rt_cn
          lda rt_am+1
          sbc rt_an+1
          sta rt_cn+1
          inc rt_cn
          bne .m
          inc rt_cn+1
.m:       lda rt_pl+1
          cmp rt_cn+1
          bcc .src
          bne .cnt
          lda rt_pl
          cmp rt_cn
          bcs .cnt
.src:     lda rt_pl
          sta rt_cn
          lda rt_pl+1
          sta rt_cn+1
.cnt:     lda rt_cn                // Anzahl merken, Ziel = Kopf + 4 + n - 1
          sta rt_am
          lda rt_cn+1
          sta rt_am+1
          clc
          lda rt_ap
          adc rt_an
          sta rt_s2
          lda rt_ap+1
          adc rt_an+1
          sta rt_s2+1
          clc
          lda rt_s2
          adc #3
          sta rt_s2
          bcc .s
          inc rt_s2+1
.s:       lda rt_pp
          sta rt_s1
          lda rt_pp+1
          sta rt_s1+1
          jsr rt_copyn
          clc                      // neue Länge: n-1+Anzahl; bei A$(n,m) nie kürzer als bisher
          lda rt_an
          adc rt_am
          sta rt_cn
          lda rt_an+1
          adc rt_am+1
          sta rt_cn+1
          lda rt_cn
          bne .d1
          dec rt_cn+1
.d1:      dec rt_cn
          lda rt_aflag
          beq .set
          lda rt_cn+1
          cmp rt_al+1
          bcc .keep
          bne .set
          lda rt_cn
          cmp rt_al
          bcc .keep
.set:     ldy #0
          lda rt_cn
          sta (rt_ap),y
          iny
          lda rt_cn+1
          sta (rt_ap),y
.keep:    rts

// ---- Vergleiche: a (Stapel) gegen b (Akkumulator): A = 0 gleich, 1 a > b, $FF a < b
rt_scmp:  jsr rt_spop2
          lda rt_pp
          sta rt_s1
          lda rt_pp+1
          sta rt_s1+1
          lda rt_sap
          sta rt_s2
          lda rt_sap+1
          sta rt_s2+1
          lda rt_pl+1
          cmp rt_sal+1
          bcc .a
          bne .b
          lda rt_pl
          cmp rt_sal
          bcc .a
.b:       lda rt_sal
          sta rt_cn
          lda rt_sal+1
          sta rt_cn+1
          jmp .go
.a:       lda rt_pl
          sta rt_cn
          lda rt_pl+1
          sta rt_cn+1
.go:      ldy #0
.l:       lda rt_cn
          ora rt_cn+1
          beq .len
          lda (rt_s1),y
          cmp (rt_s2),y
          bne .diff
          inc rt_s1
          bne .i1
          inc rt_s1+1
.i1:      inc rt_s2
          bne .i2
          inc rt_s2+1
.i2:      lda rt_cn
          bne .i3
          dec rt_cn+1
.i3:      dec rt_cn
          jmp .l
.diff:    bcc .less
          lda #1
          rts
.less:    lda #$ff
          rts
.len:     lda rt_pl+1
          cmp rt_sal+1
          bne .lc
          lda rt_pl
          cmp rt_sal
          beq .eq
.lc:      bcc .less
          lda #1
          rts
.eq:      lda #0
          rts

rt_seq:   jsr rt_scmp
          cmp #0
          beq .t
          jmp rt_ffalse
.t:       jmp rt_ftrue
rt_sne:   jsr rt_scmp
          cmp #0
          bne .t
          jmp rt_ffalse
.t:       jmp rt_ftrue
rt_slt:   jsr rt_scmp
          cmp #$ff
          beq .t
          jmp rt_ffalse
.t:       jmp rt_ftrue
rt_sgt:   jsr rt_scmp
          cmp #1
          beq .t
          jmp rt_ffalse
.t:       jmp rt_ftrue
rt_sle:   jsr rt_scmp
          cmp #1
          bne .t
          jmp rt_ffalse
.t:       jmp rt_ftrue
rt_sge:   jsr rt_scmp
          cmp #$ff
          bne .t
          jmp rt_ffalse
.t:       jmp rt_ftrue

// ---- Funktionen
rt_len:   lda rt_sal
          ldy rt_sal+1
          rts
rt_asc:   lda rt_sal
          ora rt_sal+1
          beq .z
          lda rt_sap
          sta rt_s1
          lda rt_sap+1
          sta rt_s1+1
          ldy #0
          lda (rt_s1),y
          ldy #0
          rts
.z:       lda #0
          tay
          rts

// CHR$(n): A/Y = 0..255
rt_chr:   cpy #0
          bne .bad
          sta rt_tbuf
          lda #1
          sta rt_sal
          lda #0
          sta rt_sal+1
          lda #<rt_tbuf
          sta rt_sap
          lda #>rt_tbuf
          sta rt_sap+1
          rts
.bad:     jmp rt_e_illegal

// STR$(x): FR0 -> Text ohne Maskierung der 10. Stelle, ohne abschließenden Punkt
rt_strs:  jsr FASC
          ldy #0
.l:       lda (INBUFF),y
          pha
          and #$7f
          sta rt_tbuf,y
          pla
          bmi .e
          iny
          bne .l
.e:       lda rt_tbuf,y
          cmp #'.'
          beq .nd
          iny
.nd:      sty rt_sal
          lda #0
          sta rt_sal+1
          lda #<rt_tbuf
          sta rt_sap
          lda #>rt_tbuf
          sta rt_sap+1
          rts

// Akkumulator -> rt_tbuf (nullterminiert, höchstens 62 Zeichen); INBUFF und CIX für AFP setzen
rt_totbuf: lda rt_sap
          sta rt_s1
          lda rt_sap+1
          sta rt_s1+1
          lda rt_sal+1
          bne .max
          lda rt_sal
          cmp #62
          bcc .n
.max:     lda #62
.n:       sta rt_cn
          ldy #0
.c:       cpy rt_cn
          beq .t
          lda (rt_s1),y
          sta rt_tbuf,y
          iny
          bne .c
.t:       lda #0
          sta rt_tbuf,y
          lda #<rt_tbuf
          sta INBUFF
          lda #>rt_tbuf
          sta INBUFF+1
          lda #0
          sta CIX
          rts

// VAL: führende Zahl der Zeichenkette (der Rest wird ignoriert); keine Zahl: Fehler 18
rt_val:   jsr rt_totbuf
          jsr AFP
          bcs rt_e_aval
          rts

// ---- Ausgabe und Eingabe
rt_sprint: lda rt_sal
          ora rt_sal+1
          beq .z
          lda rt_sap
          sta rt_s1
          lda rt_sap+1
          sta rt_s1+1
          lda rt_sal
          sta rt_cn
          lda rt_sal+1
          sta rt_cn+1
.l:       ldy #0
          lda (rt_s1),y
          jsr CHROUT
          inc rt_s1
          bne .a
          inc rt_s1+1
.a:       lda rt_cn
          bne .b
          dec rt_cn+1
.b:       dec rt_cn
          lda rt_cn
          ora rt_cn+1
          bne .l
.z:       rts

// INPUT A$: Prompt "? ", ganze Zeile (höchstens 80 Zeichen) -> Akkumulator
rt_input_s: lda #'?'
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
          stx rt_sal
          lda #0
          sta rt_sal+1
          lda #<rt_buf
          sta rt_sap
          lda #>rt_buf
          sta rt_sap+1
          rts

// ---- DATA: jeder Eintrag = Längenbyte + Text (roh bis zum Komma); nur wenn das Programm DATA hat (RT_ADATA)
.ifdef RT_ADATA
rt_dchk:  lda rt_dp
          cmp #<b2a_data_end
          lda rt_dp+1
          sbc #>b2a_data_end
          bcs rt_e_data
          rts

// READ in eine Zeichenkette -> Akkumulator
rt_reads: jsr rt_dchk
          ldy #0
          lda (rt_dp),y
          sta rt_sal
          sty rt_sal+1
          clc
          lda rt_dp
          adc #1
          sta rt_sap
          lda rt_dp+1
          adc #0
          sta rt_sap+1
          lda rt_dp
          sec
          adc rt_sal
          sta rt_dp
          bcc .n
          inc rt_dp+1
.n:       rts

// READ in eine Zahl -> FR0 (der ganze Eintrag muss eine Zahl sein: sonst Fehler 8)
rt_readn: jsr rt_reads
          jsr rt_totbuf
          jsr AFP
          bcs rt_e_ainput
          ldy CIX
.tr:      lda rt_tbuf,y
          beq .ok
          cmp #' '
          bne rt_e_ainput
          iny
          bne .tr
.ok:      rts

.endif

rt_sal:   .word 0
rt_sap:   .word 0
rt_pp:    .word 0
rt_pl:    .word 0
rt_cn:    .word 0
rt_an:    .word 0
rt_am:    .word 0
rt_al:    .word 0
rt_ac:    .word 0
rt_aflag: .byte 0
rt_tbuf:  .fill 64, 0
