// BASIC->assembler, C64 runtime: GOSUB/RETURN and FOR/NEXT on a shared software stack (as in the original)
// Frames (index grows upward, the type is on top):
//   GOSUB: return address lo, hi, type 2                     (3 bytes)
//   FOR:   variable lo, hi, limit lo, hi, step lo, hi, body lo, hi, type 1   (9 bytes)
// Requires: rt_core.asm

// GOSUB: rt_bp = return address
rt_gosub: ldx rt_ssp
          cpx #240
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

// RETURN: removes FOR frames down to the next GOSUB (called with jmp)
rt_return: ldx rt_ssp
.l:       cpx #0
          beq rt_e_ret
          lda rt_ss-1,x
          cmp #2
          beq .g
          txa
          sec
          sbc #9
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

// FOR: rt_fv = variable, rt_bp = start of body, limit and step on the expression stack (step on top)
rt_for:   ldx rt_ssp
.s:       cpx #0
          beq .push
          lda rt_ss-1,x
          cmp #1
          bne .push
          lda rt_ss-9,x
          cmp rt_fv
          bne .nxt
          lda rt_ss-8,x
          cmp rt_fv+1
          bne .nxt
          txa
          sec
          sbc #9
          sta rt_ssp
          jmp .push
.nxt:     txa
          sec
          sbc #9
          tax
          jmp .s
.push:    ldx rt_ssp
          cpx #240
          bcs rt_e_mem
          jsr rt_pop0
          lda rt_t0
          sta rt_v
          lda rt_t0+1
          sta rt_v+1
          jsr rt_pop0
          ldx rt_ssp
          lda rt_fv
          sta rt_ss,x
          inx
          lda rt_fv+1
          sta rt_ss,x
          inx
          lda rt_t0
          sta rt_ss,x
          inx
          lda rt_t0+1
          sta rt_ss,x
          inx
          lda rt_v
          sta rt_ss,x
          inx
          lda rt_v+1
          sta rt_ss,x
          inx
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

// NEXT: rt_fv = variable (0 = innermost FOR loop). Continue: jump to the body; finished: return
rt_next:  ldx rt_ssp
.f:       cpx #0
          beq rt_e_next
          lda rt_ss-1,x
          cmp #1
          bne rt_e_next
          lda rt_fv
          ora rt_fv+1
          beq .found
          lda rt_ss-9,x
          cmp rt_fv
          bne .no
          lda rt_ss-8,x
          cmp rt_fv+1
          beq .found
.no:      txa
          sec
          sbc #9
          tax
          jmp .f
.found:   stx rt_ssp
          lda rt_ss-9,x
          sta rt_t0
          lda rt_ss-8,x
          sta rt_t0+1
          ldy #0
          clc
          lda (rt_t0),y
          adc rt_ss-5,x
          sta (rt_t0),y
          sta rt_t1
          iny
          lda (rt_t0),y
          adc rt_ss-4,x
          sta (rt_t0),y
          sta rt_t1+1
          lda rt_ss-4,x
          bmi .neg
          sec                      // Step >= 0: continue while limit - variable >= 0
          lda rt_ss-7,x
          sbc rt_t1
          lda rt_ss-6,x
          sbc rt_t1+1
          bvc .p1
          eor #$80
.p1:      bmi .done
          bpl .loop
.neg:     sec                      // Step < 0: continue while variable - limit >= 0
          lda rt_t1
          sbc rt_ss-7,x
          lda rt_t1+1
          sbc rt_ss-6,x
          bvc .n1
          eor #$80
.n1:      bmi .done
.loop:    lda rt_ss-3,x
          sta rt_bp
          lda rt_ss-2,x
          sta rt_bp+1
          pla
          pla
          jmp (rt_bp)
.done:    txa
          sec
          sbc #9
          sta rt_ssp
          rts

rt_ss:    .fill 256, 0
