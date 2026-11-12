// BASIC→Assembler, Laufzeit C64, Stufe 2: GOSUB/RETURN und FOR/NEXT mit Fließkomma-Schleifenvariable auf einem gemeinsamen Stapel
// Rahmen (Index wächst nach oben, der Typ liegt oben):
//   GOSUB: Rücksprung lo, hi, Typ 2                                                    (3 Byte)
//   FOR:   Variable lo, hi, Grenze (RT_FS), Schritt (RT_FS), Rumpf lo, hi, Typ 1        (RT_FRAME = 2*RT_FS+5 Byte; 15 bei 5-Byte-Zahlen)
// Verlangt: rt_core.asm, rt_fp.asm

.const RT_FRAME = 2*RT_FS+5
.const RT_STEPSIGN = RT_SGN-RT_FS-3   // Byte des Schritts mit dem Vorzeichen, vom Rahmenende aus (-7 bei MBF)

.macro ssptr(off)
 txa
 clc
 adc #<(rt_ss + off - RT_FRAME)
 pha
 lda #>(rt_ss + off - RT_FRAME)
 adc #0
 tay
 pla
.endmacro

rt_gosub: ldx rt_ssp
          cpx #230
          bcs rt_e_mem
          lda rt_bp
          sta rt_ss,x
          inx
          lda rt_bp+1
          sta rt_ss,x
          inx
          lda #2
          sta rt_ss,x
          inx
          stx rt_ssp
          rts

rt_return: ldx rt_ssp
.l:       cpx #0
          beq rt_e_ret
          lda rt_ss-1,x
          cmp #2
          beq .g
          txa
          sec
          sbc #RT_FRAME
          tax
          jmp .l
.g:       lda rt_ss-3,x
          sta rt_bp
          lda rt_ss-2,x
          sta rt_bp+1
          txa
          sec
          sbc #3
          sta rt_ssp
          jmp (rt_bp)

// FOR: rt_fv = Variable, rt_bp = Rumpfanfang, Grenze und Schritt (Schritt oben) auf dem Fließkomma-Stapel
rt_for:   ldx rt_ssp
.s:       cpx #0
          beq .push
          lda rt_ss-1,x
          cmp #1
          bne .push
          lda rt_ss-RT_FRAME,x
          cmp rt_fv
          bne .nxt
          lda rt_ss-RT_FRAME+1,x
          cmp rt_fv+1
          bne .nxt
          txa
          sec
          sbc #RT_FRAME
          sta rt_ssp
          jmp .push
.nxt:     txa
          sec
          sbc #RT_FRAME
          tax
          jmp .s
.push:    ldx rt_ssp
          cpx #230
          bcs rt_e_mem
          lda rt_fv
          sta rt_ss,x
          inx
          lda rt_fv+1
          sta rt_ss,x
          inx
          lda rt_esp
          sec
          sbc #(2*RT_FS)
          sta rt_esp
          tay                      // Quelle: rt_fes + esp (Grenze, Schritt)
          lda #(2*RT_FS)
          sta rt_tmp
.cp:      lda rt_fes,y
          sta rt_ss,x
          inx
          iny
          dec rt_tmp
          bne .cp
          lda rt_bp
          sta rt_ss,x
          inx
          lda rt_bp+1
          sta rt_ss,x
          inx
          lda #1
          sta rt_ss,x
          inx
          stx rt_ssp
          rts

// NEXT: rt_fv = Variable (0 = oberste FOR-Schleife). Weiter: Sprung zum Rumpf; fertig: Rücksprung
rt_next:  ldx rt_ssp
.f:       cpx #0
          beq rt_e_next
          lda rt_ss-1,x
          cmp #1
          bne rt_e_next
          lda rt_fv
          ora rt_fv+1
          beq .found
          lda rt_ss-RT_FRAME,x
          cmp rt_fv
          bne .no
          lda rt_ss-RT_FRAME+1,x
          cmp rt_fv+1
          beq .found
.no:      txa
          sec
          sbc #RT_FRAME
          tax
          jmp .f
.found:   stx rt_ssp
          lda rt_ss-RT_FRAME,x           // Variable laden, Schritt addieren, zurückschreiben
          sta rt_t1
          lda rt_ss-RT_FRAME+1,x
          sta rt_t1+1
          lda rt_t1
          ldy rt_t1+1
          jsr rt_movfm
          ldx rt_ssp
          ssptr(2+RT_FS)           // Schritt
          jsr F_CONUPK
          jsr F_FADDT
          ldx rt_t1
          ldy rt_t1+1
          jsr F_MOVMF
          ldx rt_ssp
          ssptr(2)                 // Grenze
          jsr F_FCOMP              // FAC ? Grenze
          sta rt_v
          ldx rt_ssp
          lda rt_ss+RT_STEPSIGN,x            // Vorzeichen des Schritts (Byte 1; Exponent 0 = Schritt 0 = positiv)
          bmi .neg
          lda rt_v                 // Schritt >= 0: weiter, solange FAC <= Grenze
          cmp #1
          beq .done
          bne .loop
.neg:     lda rt_v                 // Schritt < 0: weiter, solange FAC >= Grenze
          cmp #$ff
          beq .done
.loop:    lda rt_ss-3,x
          sta rt_bp
          lda rt_ss-2,x
          sta rt_bp+1
          pla
          pla
          jmp (rt_bp)
.done:    txa
          sec
          sbc #RT_FRAME
          sta rt_ssp
          rts

rt_ss:    .fill 256, 0
