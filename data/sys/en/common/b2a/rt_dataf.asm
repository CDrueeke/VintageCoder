// BASIC->assembler, C64 runtime, level 2: DATA/READ (RT_FS-byte numbers) and arrays
// Requires: rt_core.asm, rt_fp.asm; the program defines b2a_data_end

// READ: next DATA value -> FAC
rt_read:  lda rt_dp
          cmp #<b2a_data_end
          lda rt_dp+1
          sbc #>b2a_data_end
          bcs rt_e_data
          lda rt_dp
          ldy rt_dp+1
          jsr rt_movfm
          clc
          lda rt_dp
          adc #RT_FS
          sta rt_dp
          bcc .n
          inc rt_dp+1
.n:       rts

// Array element: rt_ap = array header (element count as a word, then the elements), A/Y = index, X = element size (2, 5 or 6)
// Result: rt_t0 = address of the element
rt_elem:  sta rt_t0
          sty rt_t0+1
          stx rt_tmp
          ldy #0
          lda rt_t0
          cmp (rt_ap),y
          iny
          lda rt_t0+1
          sbc (rt_ap),y
          bcs rt_e_sub
          lda rt_t0                // index * size
          sta rt_v
          lda rt_t0+1
          sta rt_v+1
          asl rt_t0
          rol rt_t0+1
          lda rt_tmp
          cmp #2
          beq .sz
          asl rt_t0                // * 4
          rol rt_t0+1
          lda rt_tmp
          cmp #6
          bne .f5
          asl rt_v                 // 6 bytes: * 4 + 2 * index
          rol rt_v+1
.f5:      clc                      // 5 bytes: + index = * 5
          lda rt_t0
          adc rt_v
          sta rt_t0
          lda rt_t0+1
          adc rt_v+1
          sta rt_t0+1
          jmp .add
.sz:      nop                      // * 2 is already done
.add:     clc
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

// Read element: A/Y = index -> FAC
rt_ldelemf: ldx #RT_FS
          jsr rt_elem
          lda rt_t0
          ldy rt_t0+1
          jmp rt_movfm

rt_ldelemi: ldx #2
          jsr rt_elem
          ldy #1
          lda (rt_t0),y
          sta rt_tmp
          dey
          lda (rt_t0),y
          ldy rt_tmp
          jmp rt_i2f

// Write element: value on the floating-point stack, A/Y = index
rt_stelemf: ldx #RT_FS
          jsr rt_elem
          jsr rt_fpop
          jsr rt_movfm
          ldx rt_t0
          ldy rt_t0+1
          jmp F_MOVMF

rt_stelemi: ldx #2
          jsr rt_elem
          jsr rt_fpop
          jsr rt_movfm
          jsr rt_f2i
          sty rt_v
          ldy #0
          sta (rt_t0),y
          iny
          lda rt_v
          sta (rt_t0),y
          rts

// Clear array (CLR): rt_ap = array header, X = element size
rt_clrarr: stx rt_tmp
          ldy #0
          lda (rt_ap),y
          sta rt_t1
          iny
          lda (rt_ap),y
          sta rt_t1+1
          lda #0                   // v = element count * size
          sta rt_v
          sta rt_v+1
.m:       lda rt_t1
          ora rt_t1+1
          beq .z
          clc
          lda rt_v
          adc rt_tmp
          sta rt_v
          bcc .a
          inc rt_v+1
.a:       lda rt_t1
          bne .b
          dec rt_t1+1
.b:       dec rt_t1
          jmp .m
.z:       lda rt_ap
          clc
          adc #2
          sta rt_t0
          lda rt_ap+1
          adc #0
          sta rt_t0+1
.l:       lda rt_v
          ora rt_v+1
          beq .d
          ldy #0
          tya
          sta (rt_t0),y
          inc rt_t0
          bne .c
          inc rt_t0+1
.c:       lda rt_v
          bne .e
          dec rt_v+1
.e:       dec rt_v
          jmp .l
.d:       rts
