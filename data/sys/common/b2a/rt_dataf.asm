// BASIC→Assembler, Laufzeit C64, Stufe 2: DATA/READ (RT_FS-Byte-Zahlen) und Felder
// Verlangt: rt_core.asm, rt_fp.asm; das Programm definiert b2a_data_end

// READ: nächster DATA-Wert -> FAC
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

// Feldelement: rt_ap = Feldkopf (Elementzahl als Wort, danach die Elemente), A/Y = Index, X = Elementgröße (2, 5 oder 6)
// Ergebnis: rt_t0 = Adresse des Elements
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
          lda rt_t0                // Index * Größe
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
          asl rt_v                 // 6 Byte: * 4 + 2 * Index
          rol rt_v+1
.f5:      clc                      // 5 Byte: + Index = * 5
          lda rt_t0
          adc rt_v
          sta rt_t0
          lda rt_t0+1
          adc rt_v+1
          sta rt_t0+1
          jmp .add
.sz:      nop                      // * 2 ist schon erledigt
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

// Element lesen: A/Y = Index -> FAC
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

// Element schreiben: Wert auf dem Fließkomma-Stapel, A/Y = Index
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

// Feld löschen (CLR): rt_ap = Feldkopf, X = Elementgröße
rt_clrarr: stx rt_tmp
          ldy #0
          lda (rt_ap),y
          sta rt_t1
          iny
          lda (rt_ap),y
          sta rt_t1+1
          lda #0                   // v = Elementzahl * Größe
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
