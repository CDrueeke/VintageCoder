// BASIC->assembler, C64 runtime, level 3: strings (own management, independent of the BASIC ROM)
//
// A string is a 3-byte descriptor: length, address low, address high. The characters are located
//  - as constants in the program (statement text, DATA), or
//  - in the heap, which grows downward from $A000 and extends to the end of the program (b2a_end).
// Strings are immutable; assignment copies only the descriptor. When the heap is full, the garbage collection
// (rt_sgc) moves all strings in use together toward the top. Roots: variables (b2a_sroots), arrays (b2a_aroots), string stack, accumulator.
//
// Results are in the accumulator rt_sacc. Binary operations: left operand on the string stack, right one in the accumulator.
// Requires: rt_core.asm, rt_zp.asm; the program defines b2a_end, b2a_sroots, b2a_aroots (and b2a_data_end with RT_SDATA)

rt_sacc:  .byte 0, 0, 0
rt_hp:    .word 0
rt_sstp:  .byte 0
rt_sst:   .fill 48, 0
rt_dst:   .word 0
rt_msrc:  .word 0
rt_tl:    .byte 0
rt_alen:  .byte 0
rt_n:     .byte 0
rt_m:     .byte 0
rt_off:   .byte 0
rt_st:    .byte 0
rt_bi:    .byte 0
rt_gl:    .byte 0
rt_gp:    .word 0
rt_gbest: .word 0
rt_gbl:   .byte 0
rt_glim:  .word 0
rt_gold:  .word 0
rt_gnew:  .word 0
rt_gnt:   .word 0
rt_gmode: .byte 0
rt_gx:    .byte 0
rt_ghd:   .word 0
rt_gcn:   .word 0
rt_vbuf:  .fill 258, 0

// Reset heap and stack (start, CLR)
rt_sinit: lda #0
          sta rt_sstp
          sta rt_sacc
          lda #<RT_HEAPTOP
          sta rt_hp
          lda #>RT_HEAPTOP
          sta rt_hp+1
          rts

// A/Y = address of a descriptor -> accumulator
rt_sload: sta rt_s1
          sty rt_s1+1
          ldy #0
          lda (rt_s1),y
          sta rt_sacc
          iny
          lda (rt_s1),y
          sta rt_sacc+1
          iny
          lda (rt_s1),y
          sta rt_sacc+2
          rts

// accumulator -> descriptor at A/Y
rt_sstor: sta rt_s1
          sty rt_s1+1
          ldy #0
          lda rt_sacc
          sta (rt_s1),y
          iny
          lda rt_sacc+1
          sta (rt_s1),y
          iny
          lda rt_sacc+2
          sta (rt_s1),y
          rts

// Accumulator onto the string stack (16 entries)
rt_spush: ldx rt_sstp
          cpx #48
          bcs rt_e_formula
          lda rt_sacc
          sta rt_sst,x
          lda rt_sacc+1
          sta rt_sst+1,x
          lda rt_sacc+2
          sta rt_sst+2,x
          inx
          inx
          inx
          stx rt_sstp
          rts

rt_spop:  lda rt_sstp
          sec
          sbc #3
          sta rt_sstp
          rts

// Copy A bytes from (rt_s1) to (rt_s2) (A = 0 is allowed)
rt_cp:    tax
          beq .d
          ldy #0
.l:       lda (rt_s1),y
          sta (rt_s2),y
          iny
          dex
          bne .l
.d:       rts

// ---- Memory in the heap: A = length (1..255) -> rt_s2 = address
rt_salloc: sta rt_alen
          jsr .try
          bcc .ok
          jsr rt_sgc
          jsr .try
          bcc .ok
          jmp rt_e_mem
.ok:      rts
.try:     sec
          lda rt_hp
          sbc rt_alen
          sta rt_s2
          lda rt_hp+1
          sbc #0
          sta rt_s2+1
          lda rt_s2
          cmp #<b2a_end
          lda rt_s2+1
          sbc #>b2a_end
          bcc .fail
          lda rt_s2
          sta rt_hp
          lda rt_s2+1
          sta rt_hp+1
          clc
          rts
.fail:    sec
          rts

// Take over the string from rt_msrc (A characters, not in the heap) into the heap -> accumulator
rt_smk:   sta rt_tl
          bne .some
          lda #0
          sta rt_sacc
          rts
.some:    jsr rt_salloc
          lda rt_msrc
          sta rt_s1
          lda rt_msrc+1
          sta rt_s1+1
          lda rt_tl
          jsr rt_cp
          lda rt_tl
          sta rt_sacc
          lda rt_s2
          sta rt_sacc+1
          lda rt_s2+1
          sta rt_sacc+2
          rts

// ---- Garbage collection
rt_sgc:   lda #<RT_HEAPTOP
          sta rt_gnt
          sta rt_glim
          lda #>RT_HEAPTOP
          sta rt_gnt+1
          sta rt_glim+1
.next:    lda #0
          sta rt_gbest
          sta rt_gbest+1
          sta rt_gmode
          jsr rt_gscan
          lda rt_gbest
          ora rt_gbest+1
          beq .done
          sec                      // new upper limit = upper limit - length
          lda rt_gnt
          sbc rt_gbl
          sta rt_gnt
          lda rt_gnt+1
          sbc #0
          sta rt_gnt+1
          lda rt_gbest             // copy backward (destination lies above the source)
          sta rt_s1
          lda rt_gbest+1
          sta rt_s1+1
          lda rt_gnt
          sta rt_s2
          lda rt_gnt+1
          sta rt_s2+1
          ldy rt_gbl
.cp:      dey
          lda (rt_s1),y
          sta (rt_s2),y
          cpy #0
          bne .cp
          lda rt_gbest             // switch all descriptors with the old address
          sta rt_gold
          lda rt_gbest+1
          sta rt_gold+1
          lda rt_gnt
          sta rt_gnew
          lda rt_gnt+1
          sta rt_gnew+1
          lda #1
          sta rt_gmode
          jsr rt_gscan
          lda rt_gold
          sta rt_glim
          lda rt_gold+1
          sta rt_glim+1
          jmp .next
.done:    lda rt_gnt
          sta rt_hp
          lda rt_gnt+1
          sta rt_hp+1
          rts

// visit all descriptors (accumulator, string stack, variables, arrays)
rt_gscan: lda #<rt_sacc
          sta rt_gd
          lda #>rt_sacc
          sta rt_gd+1
          jsr rt_gvisit
          ldx #0
.st:      cpx rt_sstp
          bcs .sc
          txa
          clc
          adc #<rt_sst
          sta rt_gd
          lda #>rt_sst
          adc #0
          sta rt_gd+1
          stx rt_gx
          jsr rt_gvisit
          ldx rt_gx
          inx
          inx
          inx
          jmp .st
.sc:      lda #<b2a_sroots
          sta rt_gq
          lda #>b2a_sroots
          sta rt_gq+1
.s1:      ldy #0
          lda (rt_gq),y
          sta rt_gd
          iny
          lda (rt_gq),y
          sta rt_gd+1
          ora rt_gd
          beq .ar
          jsr rt_gvisit
          clc
          lda rt_gq
          adc #2
          sta rt_gq
          bcc .s1
          inc rt_gq+1
          jmp .s1
.ar:      lda #<b2a_aroots
          sta rt_gq
          lda #>b2a_aroots
          sta rt_gq+1
.a1:      ldy #0
          lda (rt_gq),y
          sta rt_ghd
          iny
          lda (rt_gq),y
          sta rt_ghd+1
          ora rt_ghd
          beq .end
          lda rt_ghd
          sta rt_gd
          lda rt_ghd+1
          sta rt_gd+1
          ldy #0
          lda (rt_gd),y
          sta rt_gcn
          iny
          lda (rt_gd),y
          sta rt_gcn+1
          clc
          lda rt_ghd
          adc #2
          sta rt_gd
          lda rt_ghd+1
          adc #0
          sta rt_gd+1
.el:      lda rt_gcn
          ora rt_gcn+1
          beq .na
          jsr rt_gvisit
          clc
          lda rt_gd
          adc #3
          sta rt_gd
          bcc .e1
          inc rt_gd+1
.e1:      lda rt_gcn
          bne .e2
          dec rt_gcn+1
.e2:      dec rt_gcn
          jmp .el
.na:      clc
          lda rt_gq
          adc #2
          sta rt_gq
          bcc .a1
          inc rt_gq+1
          jmp .a1
.end:     rts

// one descriptor (rt_gd): mode 0 = remember the highest address below rt_glim, mode 1 = switch address rt_gold to rt_gnew
rt_gvisit: ldy #0
          lda (rt_gd),y
          beq .ret
          sta rt_gl
          iny
          lda (rt_gd),y
          sta rt_gp
          iny
          lda (rt_gd),y
          sta rt_gp+1
          lda rt_gmode
          bne .upd
          lda rt_gp                // heap strings only (address >= b2a_end)
          cmp #<b2a_end
          lda rt_gp+1
          sbc #>b2a_end
          bcc .ret
          lda rt_gp                // below the limit
          cmp rt_glim
          lda rt_gp+1
          sbc rt_glim+1
          bcs .ret
          lda rt_gbest             // higher than the best so far
          cmp rt_gp
          lda rt_gbest+1
          sbc rt_gp+1
          bcs .ret
          lda rt_gp
          sta rt_gbest
          lda rt_gp+1
          sta rt_gbest+1
          lda rt_gl
          sta rt_gbl
.ret:     rts
.upd:     lda rt_gp
          cmp rt_gold
          bne .ret
          lda rt_gp+1
          cmp rt_gold+1
          bne .ret
          ldy #1
          lda rt_gnew
          sta (rt_gd),y
          iny
          lda rt_gnew+1
          sta (rt_gd),y
          rts

// ---- Concatenation: left on the string stack, right in the accumulator
rt_scat:  ldx rt_sstp
          lda rt_sst-3,x
          clc
          adc rt_sacc
          bcs rt_e_strlong
          sta rt_tl
          bne .some
          lda #0
          sta rt_sacc
          jmp rt_spop
.some:    jsr rt_salloc
          lda rt_s2
          sta rt_dst
          lda rt_s2+1
          sta rt_dst+1
          ldx rt_sstp
          lda rt_sst-2,x
          sta rt_s1
          lda rt_sst-1,x
          sta rt_s1+1
          lda rt_sst-3,x
          sta rt_n
          jsr rt_cp
          clc
          lda rt_s2
          adc rt_n
          sta rt_s2
          bcc .r
          inc rt_s2+1
.r:       lda rt_sacc+1
          sta rt_s1
          lda rt_sacc+2
          sta rt_s1+1
          lda rt_sacc
          jsr rt_cp
          lda rt_tl
          sta rt_sacc
          lda rt_dst
          sta rt_sacc+1
          lda rt_dst+1
          sta rt_sacc+2
          jmp rt_spop

// ---- Comparison: left on the string stack, right in the accumulator. A = 0 equal, 1 left greater, $FF left smaller
rt_scmp:  ldx rt_sstp
          lda rt_sst-2,x
          sta rt_s1
          lda rt_sst-1,x
          sta rt_s1+1
          lda rt_sst-3,x
          sta rt_n
          lda rt_sacc+1
          sta rt_s2
          lda rt_sacc+2
          sta rt_s2+1
          lda rt_sacc
          sta rt_m
          ldy #0
.l:       cpy rt_n
          beq .endl
          cpy rt_m
          beq .gt
          lda (rt_s1),y
          cmp (rt_s2),y
          bne .diff
          iny
          jmp .l
.endl:    cpy rt_m
          beq .eq
          lda #$ff
          jmp .res
.diff:    bcs .gt
          lda #$ff
          jmp .res
.gt:      lda #1
          jmp .res
.eq:      lda #0
.res:     pha
          jsr rt_spop
          pla
          rts

.ifdef RT_FLOAT
rt_strue:  jmp rt_ftrue
rt_sfalse: jmp rt_ffalse
.else
rt_strue:  jmp rt_true
rt_sfalse: jmp rt_false
.endif
rt_seq:   jsr rt_scmp
          beq rt_strue
          bne rt_sfalse
rt_sne:   jsr rt_scmp
          bne rt_strue
          beq rt_sfalse
rt_slt:   jsr rt_scmp
          cmp #$ff
          beq rt_strue
          bne rt_sfalse
rt_sgt:   jsr rt_scmp
          cmp #1
          beq rt_strue
          bne rt_sfalse
rt_sle:   jsr rt_scmp
          cmp #1
          bne rt_strue
          beq rt_sfalse
rt_sge:   jsr rt_scmp
          cmp #$ff
          bne rt_strue
          beq rt_sfalse

// ---- Substrings: source on the string stack, rt_n = count, rt_off = offset
rt_ssub:  lda rt_n
          beq .empty
          jsr rt_salloc
          ldx rt_sstp
          clc
          lda rt_sst-2,x
          adc rt_off
          sta rt_s1
          lda rt_sst-1,x
          adc #0
          sta rt_s1+1
          lda rt_n
          jsr rt_cp
          lda rt_n
          sta rt_sacc
          lda rt_s2
          sta rt_sacc+1
          lda rt_s2+1
          sta rt_sacc+2
          jmp rt_spop
.empty:   lda #0
          sta rt_sacc
          jmp rt_spop

// LEFT$(s, n): A/Y = n
rt_left:  cpy #0
          bne rt_e_illegal
          sta rt_n
          ldx rt_sstp
          lda rt_sst-3,x
          cmp rt_n
          bcs .k
          sta rt_n
.k:       lda #0
          sta rt_off
          jmp rt_ssub

// RIGHT$(s, n)
rt_right: cpy #0
          bne rt_e_illegal
          sta rt_n
          ldx rt_sstp
          lda rt_sst-3,x
          cmp rt_n
          bcs .k
          sta rt_n
.k:       lda rt_sst-3,x
          sec
          sbc rt_n
          sta rt_off
          jmp rt_ssub

// MID$(s, start, len): start (1..255) on the integer stack, A/Y = length
rt_mid:   cpy #0
          bne rt_e_illegal
          sta rt_m
          jsr rt_pop0
          lda rt_t0+1
          bne rt_e_illegal
          lda rt_t0
          beq rt_e_illegal
          sta rt_st
          ldx rt_sstp
          lda rt_sst-3,x
          sec
          sbc rt_st
          bcc .none
          clc
          adc #1
          cmp rt_m
          bcc .a
          lda rt_m
.a:       sta rt_n
          lda rt_st
          sec
          sbc #1
          sta rt_off
          jmp rt_ssub
.none:    lda #0
          sta rt_n
          jmp rt_ssub

// CHR$(n): A/Y = code 0..255
rt_chr:   cpy #0
          bne rt_e_illegal
          pha
          lda #1
          jsr rt_salloc
          pla
          ldy #0
          sta (rt_s2),y
          lda #1
          sta rt_sacc
          lda rt_s2
          sta rt_sacc+1
          lda rt_s2+1
          sta rt_sacc+2
          rts

rt_len:   lda rt_sacc
          ldy #0
          rts

rt_asc:   lda rt_sacc
          beq rt_e_illegal
          lda rt_sacc+1
          sta rt_s1
          lda rt_sacc+2
          sta rt_s1+1
          ldy #0
          lda (rt_s1),y
          ldy #0
          rts

// Copy accumulator to rt_vbuf, terminated with 0
rt_s2v:   lda rt_sacc+1
          sta rt_s1
          lda rt_sacc+2
          sta rt_s1+1
          ldy #0
.l:       cpy rt_sacc
          beq .t
          lda (rt_s1),y
          sta rt_vbuf,y
          iny
          jmp .l
.t:       lda #0
          sta rt_vbuf,y
          rts

// ---- Output
rt_sprint: lda rt_sacc
          beq .d
          sta rt_n
          lda rt_sacc+1
          sta rt_s1
          lda rt_sacc+2
          sta rt_s1+1
          ldy #0
.l:       lda (rt_s1),y
          jsr CHROUT
          iny
          cpy rt_n
          bne .l
.d:       rts

// ---- Number <-> text
.ifdef RT_FLOAT
// STR$ (FAC): text from the ROM (FOUT)
rt_strs:  jsr F_FOUT
          sta rt_t0
          sty rt_t0+1
          sta rt_msrc
          sty rt_msrc+1
          ldy #0
.l:       lda (rt_t0),y
          beq .e
          iny
          jmp .l
.e:       tya
          jmp rt_smk

// VAL (accumulator) -> FAC
rt_val:   jsr rt_s2v
          lda F_TXTPTR
          sta rt_txt
          lda F_TXTPTR+1
          sta rt_txt+1
          lda #<rt_vbuf
          sta F_TXTPTR
          lda #>rt_vbuf
          sta F_TXTPTR+1
          jsr F_CHRGOT
          jsr F_FIN
          lda rt_txt
          sta F_TXTPTR
          lda rt_txt+1
          sta F_TXTPTR+1
          rts
.else
// STR$ (A/Y): sign position (space or -) and digits
rt_strs:  sta rt_t0
          sty rt_t0+1
          tya
          bpl .pos
          lda #'-'
          sta rt_vbuf
          neg16(rt_t0)
          jmp .go
.pos:     lda #' '
          sta rt_vbuf
.go:      lda #1
          sta rt_bi
          lda #0
          sta rt_tmp
          ldx #0
.digit:   ldy #0
.sub:     sec
          lda rt_t0
          sbc rt_pow_lo,x
          sta rt_v
          lda rt_t0+1
          sbc rt_pow_hi,x
          bcc .done
          sta rt_t0+1
          lda rt_v
          sta rt_t0
          iny
          jmp .sub
.done:    tya
          bne .out
          cpx #4
          beq .out
          lda rt_tmp
          beq .skip
          tya
.out:     clc
          adc #'0'
          pha
          lda #1
          sta rt_tmp
          ldy rt_bi
          pla
          sta rt_vbuf,y
          inc rt_bi
.skip:    inx
          cpx #5
          bne .digit
          lda #<rt_vbuf
          sta rt_msrc
          lda #>rt_vbuf
          sta rt_msrc+1
          lda rt_bi
          jmp rt_smk

// VAL (accumulator) -> A/Y: leading spaces, sign, digits up to the first other character
rt_val:   jsr rt_s2v
          lda #0
          sta rt_t0
          sta rt_t0+1
          sta rt_tmp
          ldx #0
.sp:      lda rt_vbuf,x
          cmp #' '
          bne .sg
          inx
          jmp .sp
.sg:      cmp #'-'
          bne .pl
          lda #$80
          sta rt_tmp
          inx
          jmp .dg
.pl:      cmp #'+'
          bne .dg
          inx
.dg:      lda rt_vbuf,x
          sec
          sbc #'0'
          cmp #10
          bcs .end
          pha
          asl rt_t0
          rol rt_t0+1
          lda rt_t0
          sta rt_t1
          lda rt_t0+1
          sta rt_t1+1
          asl rt_t0
          rol rt_t0+1
          asl rt_t0
          rol rt_t0+1
          clc
          lda rt_t0
          adc rt_t1
          sta rt_t0
          lda rt_t0+1
          adc rt_t1+1
          sta rt_t0+1
          pla
          clc
          adc rt_t0
          sta rt_t0
          bcc .nc
          inc rt_t0+1
.nc:      inx
          jmp .dg
.end:     lda rt_tmp
          bpl .done
          neg16(rt_t0)
.done:    lda rt_t0
          ldy rt_t0+1
          rts
.endif

// ---- Input (INPUT, GET)
// INPUT a$: print "? ", read line -> accumulator
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
          sta rt_vbuf,x
          inx
          bne .rd
.eol:     lda #13
          jsr CHROUT
          lda #<rt_vbuf
          sta rt_msrc
          lda #>rt_vbuf
          sta rt_msrc+1
          txa
          jmp rt_smk

// GET a$: one key or the empty string
rt_get_s: jsr GETIN
          cmp #0
          bne .key
          sta rt_sacc
          rts
.key:     jmp rt_chr_a
rt_chr_a: ldy #0
          jmp rt_chr

// ---- String arrays: rt_ap = array header (element count, then 3 bytes each)
// A/Y = index -> rt_t0 = address of the descriptor
rt_selem: sta rt_t0
          sty rt_t0+1
          ldy #0
          lda rt_t0
          cmp (rt_ap),y
          iny
          lda rt_t0+1
          sbc (rt_ap),y
          bcs rt_e_sub
          lda rt_t0
          sta rt_v
          lda rt_t0+1
          sta rt_v+1
          asl rt_t0
          rol rt_t0+1
          clc
          lda rt_t0
          adc rt_v
          sta rt_t0
          lda rt_t0+1
          adc rt_v+1
          sta rt_t0+1
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

rt_sldelem: jsr rt_selem
          lda rt_t0
          ldy rt_t0+1
          jmp rt_sload

// Write element: value on the string stack, A/Y = index
rt_ststelem: jsr rt_selem
          ldx rt_sstp
          ldy #0
          lda rt_sst-3,x
          sta (rt_t0),y
          iny
          lda rt_sst-2,x
          sta (rt_t0),y
          iny
          lda rt_sst-1,x
          sta (rt_t0),y
          jmp rt_spop

// Clear array (CLR)
rt_sclrarr: ldy #0
          lda (rt_ap),y
          sta rt_gcn
          iny
          lda (rt_ap),y
          sta rt_gcn+1
          clc
          lda rt_ap
          adc #2
          sta rt_t0
          lda rt_ap+1
          adc #0
          sta rt_t0+1
.l:       lda rt_gcn
          ora rt_gcn+1
          beq .d
          ldy #0
          tya
          sta (rt_t0),y
          clc
          lda rt_t0
          adc #3
          sta rt_t0
          bcc .a
          inc rt_t0+1
.a:       lda rt_gcn
          bne .b
          dec rt_gcn+1
.b:       dec rt_gcn
          jmp .l
.d:       rts

// ---- DATA with strings: entry = length, address (2), kind (0 = text, 1 = number), number (2 bytes, 5 bytes in floating-point mode)
.ifdef RT_SDATA
.ifdef RT_FLOAT
.const RT_DSIZE = 9
.else
.const RT_DSIZE = 6
.endif

rt_dchk:  lda rt_dp
          cmp #<b2a_data_end
          lda rt_dp+1
          sbc #>b2a_data_end
          bcs rt_e_data
          rts

rt_dadv:  clc
          lda rt_dp
          adc #RT_DSIZE
          sta rt_dp
          bcc .x
          inc rt_dp+1
.x:       rts

// READ a$: the text of the entry -> accumulator
rt_reads: jsr rt_dchk
          ldy #0
          lda (rt_dp),y
          sta rt_sacc
          iny
          lda (rt_dp),y
          sta rt_sacc+1
          iny
          lda (rt_dp),y
          sta rt_sacc+2
          jmp rt_dadv

// READ x: number of the entry (A/Y or FAC); a text is a SYNTAX ERROR
rt_readn: jsr rt_dchk
          ldy #3
          lda (rt_dp),y
          beq rt_e_syntax
.ifdef RT_FLOAT
          lda rt_dp
          clc
          adc #4
          pha
          lda rt_dp+1
          adc #0
          tay
          pla
          jsr rt_movfm
          jmp rt_dadv
.else
          ldy #4
          lda (rt_dp),y
          sta rt_v
          iny
          lda (rt_dp),y
          sta rt_v+1
          jsr rt_dadv
          lda rt_v
          ldy rt_v+1
          rts
.endif
.endif

// ---- TI$ (system clock $A0-$A2, 60 steps per second): read = "hhmmss", assign = set the clock
rt_d0:    .byte 0
rt_d1:    .byte 0
rt_d2:    .byte 0
rt_m0:    .byte 0
rt_m1:    .byte 0
rt_m2:    .byte 0
rt_dv:    .byte 0
rt_dr:    .byte 0
rt_th:    .byte 0
rt_tm:    .byte 0
rt_ts:    .byte 0

// Divide the 24-bit number rt_d0-rt_d2 by A; quotient stays in rt_d0-rt_d2, A = remainder
rt_div24: sta rt_dv
          lda #0
          sta rt_dr
          ldx #24
.l:       asl rt_d0
          rol rt_d1
          rol rt_d2
          rol rt_dr
          lda rt_dr
          cmp rt_dv
          bcc .n
          sbc rt_dv
          sta rt_dr
          inc rt_d0
.n:       dex
          bne .l
          lda rt_dr
          rts

// A = 0..99 as two digits to rt_vbuf[X], X += 2
rt_2dig:  ldy #0
.l:       cmp #10
          bcc .d
          sec
          sbc #10
          iny
          jmp .l
.d:       clc
          adc #'0'
          pha
          tya
          clc
          adc #'0'
          sta rt_vbuf,x
          inx
          pla
          sta rt_vbuf,x
          inx
          rts

rt_tis:   jsr RDTIM              // A = low, X = middle, Y = high (KERNAL, same on all targets)
          sta rt_d0
          stx rt_d1
          sty rt_d2
          lda #60
          jsr rt_div24             // total seconds
          lda #60
          jsr rt_div24             // total minutes, remainder = seconds
          sta rt_ts
          lda #60
          jsr rt_div24             // hours, remainder = minutes
          sta rt_tm
          lda rt_d0
          sta rt_th
          ldx #0
          lda rt_th
          jsr rt_2dig
          lda rt_tm
          jsr rt_2dig
          lda rt_ts
          jsr rt_2dig
          lda #<rt_vbuf
          sta rt_msrc
          lda #>rt_vbuf
          sta rt_msrc+1
          lda #6
          jmp rt_smk

// rt_d0-rt_d2 = rt_d0-rt_d2 * 60  (= * 64 - * 4)
rt_mul60: lda rt_d0
          sta rt_m0
          lda rt_d1
          sta rt_m1
          lda rt_d2
          sta rt_m2
          asl rt_m0
          rol rt_m1
          rol rt_m2
          asl rt_m0
          rol rt_m1
          rol rt_m2
          ldx #6
.s:       asl rt_d0
          rol rt_d1
          rol rt_d2
          dex
          bne .s
          sec
          lda rt_d0
          sbc rt_m0
          sta rt_d0
          lda rt_d1
          sbc rt_m1
          sta rt_d1
          lda rt_d2
          sbc rt_m2
          sta rt_d2
          rts

// A = tens, Y = ones -> A = number
rt_dig2:  asl
          sta rt_tmp
          asl
          asl
          clc
          adc rt_tmp
          sty rt_tmp
          adc rt_tmp
          rts

rt_add24a: clc
          adc rt_d0
          sta rt_d0
          bcc .x
          inc rt_d1
          bne .x
          inc rt_d2
.x:       rts

// TI$ = "hhmmss" (accumulator)
rt_tiset: lda rt_sacc
          cmp #6
          beq .len
.bad:     jmp rt_e_illegal
.len:     jsr rt_s2v
          ldx #0
.v:       lda rt_vbuf,x
          sec
          sbc #'0'
          cmp #10
          bcs .bad
          sta rt_vbuf,x
          inx
          cpx #6
          bne .v
          lda #0
          sta rt_d1
          sta rt_d2
          lda rt_vbuf
          ldy rt_vbuf+1
          jsr rt_dig2
          sta rt_d0
          jsr rt_mul60
          lda rt_vbuf+2
          ldy rt_vbuf+3
          jsr rt_dig2
          jsr rt_add24a
          jsr rt_mul60
          lda rt_vbuf+4
          ldy rt_vbuf+5
          jsr rt_dig2
          jsr rt_add24a
          jsr rt_mul60
          lda rt_d0
          ldx rt_d1
          ldy rt_d2
          jmp SETTIM
