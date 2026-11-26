// BASIC->assembler, C64 runtime: DATA/READ and arrays
// Requires: rt_core.asm; the program defines b2a_data_end (end of the DATA table)

// READ: next DATA value to A/Y
rt_read:  lda rt_dp
          cmp #<b2a_data_end
          lda rt_dp+1
          sbc #>b2a_data_end
          bcs rt_e_data
          ldy #0
          lda (rt_dp),y
          pha
          iny
          lda (rt_dp),y
          pha
          clc
          lda rt_dp
          adc #2
          sta rt_dp
          bcc .n
          inc rt_dp+1
.n:       pla
          tay
          pla
          rts

// Array element: rt_ap = array header (element count as a word, then the elements), A/Y = index; result: rt_t0 = address of the element
rt_elem:  sta rt_t0
          sty rt_t0+1
          ldy #0
          lda rt_t0
          cmp (rt_ap),y
          iny
          lda rt_t0+1
          sbc (rt_ap),y
          bcs rt_e_sub
          asl rt_t0
          rol rt_t0+1
          clc
          lda rt_t0
          adc rt_ap
          sta rt_t0
          lda rt_t0+1
          adc rt_ap+1
          sta rt_t0+1
          clc
          lda rt_t0
          adc #2
          sta rt_t0
          bcc .x
          inc rt_t0+1
.x:       rts

// Read element: A/Y = index -> A/Y = value
rt_ldelem: jsr rt_elem
          ldy #1
          lda (rt_t0),y
          sta rt_tmp
          dey
          lda (rt_t0),y
          ldy rt_tmp
          rts

// Write element: value on the expression stack, A/Y = index
rt_stelem: sta rt_v
          sty rt_v+1
          jsr rt_pop0
          lda rt_t0
          sta rt_t1
          lda rt_t0+1
          sta rt_t1+1
          lda rt_v
          ldy rt_v+1
          jsr rt_elem
          ldy #0
          lda rt_t1
          sta (rt_t0),y
          iny
          lda rt_t1+1
          sta (rt_t0),y
          rts

// Clear array (CLR): rt_ap = array header
rt_clrarr: ldy #0
          lda (rt_ap),y
          sta rt_t1
          iny
          lda (rt_ap),y
          sta rt_t1+1
          asl rt_t1
          rol rt_t1+1
          lda rt_ap
          clc
          adc #2
          sta rt_t0
          lda rt_ap+1
          adc #0
          sta rt_t0+1
.l:       lda rt_t1
          ora rt_t1+1
          beq .d
          ldy #0
          tya
          sta (rt_t0),y
          inc rt_t0
          bne .a
          inc rt_t0+1
.a:       lda rt_t1
          bne .b
          dec rt_t1+1
.b:       dec rt_t1
          jmp .l
.d:       rts
