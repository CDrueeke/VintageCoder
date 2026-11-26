; =====================================================================================================================
; cputest.asm - self-test for the instruction set of the 45GS02 (MEGA65)
; =====================================================================================================================
; The 45GS02 is the processor of the MEGA65: a 65CE02 with a Z register, 16-bit instructions, a 32-bit Q register and 32-bit pointers.
; This program runs one or a few instructions of each group, stores the result in the zero page and then stays in an
; endless loop (label `done`).  It prints nothing on the screen: a test program (or you, in the monitor) afterwards compares
; the zero page with the expected values, which are given as a comment behind each store instruction.
;
; Usage / flow
;   * Build:  vcc asm cputest.asm --target mega65 --cpu 45gs02 -o cputest.prg   (PRG with load address $2000, no BASIC starter;
;     with --format bin the raw image without load address, as crates/vc-ide/tests/debug_vice.rs loads it for Xemu).
;   * Start: without BASIC/KERNAL, e.g. in the Xemu emulator, or on the hardware load it through the UART monitor and start with `g 2000`.
;   * Then read the memory: the zero page $60-$D4 (list below) must contain the stated values.  Breakpoint: label `done`.
;   * Requirement: base page register B = 0 (the "zero page" of the 45GS02 lies at B * 256; this program does not set B),
;     and $3001 as well as chip RAM at $10000/$10001 may be written (this is harmless without KERNAL/DOS).
;
; Tested instruction groups (results in the zero page)
;   Z register: LDZ, TZA, INZ, DEZ                       $60-$62        16 bit: INW, DEW, ASW                         $63-$68
;   NEG, ASR (accumulator)                               $69, $6A       long jumps LBRA, BSR/RTS                      $6B-$6D
;   Bit instructions BBS/BBR, RMB/SMB                    $6E, $6F, $D3, $D4
;   Q register (32 bit): LDQ, STQ, INQ, ADCQ, NEGQ, ASLQ Q, LSRQ Q    $84-$97
;   32-bit pointers [$nn],z and ($nn),z                  $A4, $D2       PHW                                           $C0, $C1
;   CPZ and flags                                        $C2
; Inputs/operands in the zero page: $70-$73 (16-bit values), $80-$83 (Q operand), $A0-$A3 (32-bit pointer), $D0-$D1 (16-bit pointer).
; Other places written: $3001 (target byte of ($D0),z), $10000/$10001 in chip RAM bank 1 (target bytes of [$A0],z).
; Memory map: program from $2000 (.org), about 290 bytes; otherwise only the places named.
; =====================================================================================================================
.target mega65
.cpu 45gs02 { illegals = deny }

.org $2000
start:  sei                         ; no interrupts: after the start only this program runs
        cle                         ; CLE clears the E flag = 16-bit stack pointer (45GS02) ...
        see                         ; ... SEE sets it again = 8-bit stack pointer in page $01 as on the 6502 (BSR and PHW use the stack)
        ; --- Z register: additional index register, also index for ($nn),z and [$nn],z ---
        ldz #$12
        tza                         ; get Z into the accumulator so that it can be stored
        sta $60                     ; $12
        inz
        tza
        sta $61                     ; $13
        dez
        dez
        tza
        sta $62                     ; $11
        ; --- 16-bit instructions: work on two consecutive memory bytes (low byte first) ---
        lda #$ff
        sta $70
        lda #$00
        sta $71
        inw $70                     ; $00FF + 1 = $0100 (carry into the high byte)
        lda $70
        sta $63                     ; $00
        lda $71
        sta $64                     ; $01
        dew $70                     ; $0100 - 1 = $00FF
        lda $70
        sta $65                     ; $FF
        lda $71
        sta $66                     ; $00
        lda #$80
        sta $72
        lda #$01
        sta $73
        asw $0072                   ; $0180 << 1 = $0300 (ASW exists only with an absolute address, hence $0072 instead of $72)
        lda $72
        sta $67                     ; $00
        lda $73
        sta $68                     ; $03
        ; --- NEG / ASR on the accumulator (not present on the 6502) ---
        lda #$01
        neg                         ; two's complement: -1
        sta $69                     ; $FF
        lda #$81
        asr                         ; arithmetic shift right, the sign bit stays: %10000001 -> %11000000
        sta $6a                     ; $C0
        ; --- long jumps (16-bit distance), BSR (subroutine call with relative distance) ---
        lda #$01
        sta $6b
        lbra skip                   ; unconditional long jump over the next two instructions
        lda #$02
        sta $6b                     ; is skipped
skip:   bsr sub                     ; call like JSR, but with relative distance; return with a normal RTS
        lda #$09
        sta $6d                     ; $09 (after the return)
        bra bits
sub:    lda #$07
        sta $6c                     ; $07
        rts
        ; --- Bit instructions: BBS/BBR (branch if bit set/clear), RMB/SMB (clear/set bit), all on zero page bytes ---
bits:   lda #$04                    ; %00000100: only bit 2 set
        sta $6e
        lda #$00
        sta $6f                     ; counter for wrong branch decisions
        bbs2 $6e,t1                 ; bit 2 set -> branch
        inc $6f
t1:     bbr3 $6e,t2                 ; bit 3 clear -> branch
        inc $6f
t2:     bbs0 $6e,bad                ; bit 0 clear -> no branch
        bra t3
bad:    inc $6f                     ; $6F stays $00
t3:     lda #$ff
        sta $d3
        rmb3 $d3                    ; clear bit 3: $FF -> $F7
        lda #$00
        sta $d4
        smb7 $d4                    ; set bit 7: $00 -> $80
        ; --- Q register (32 bit, A is the least significant byte): instructions with the prefix NEG NEG ($42 $42) ---
        ; Q instructions load/store four consecutive bytes, least significant first.
        lda #$78
        sta $80
        lda #$56
        sta $81
        lda #$34
        sta $82
        lda #$12
        sta $83                     ; $12345678
        ldq $80
        inq
        stq $84                     ; $12345679 -> 79 56 34 12
        ldq $80
        clc
        adcq $80
        stq $88                     ; $2468ACF0 -> F0 AC 68 24
        ldq $80
        negq
        stq $8c                     ; $EDCBA988 -> 88 A9 CB ED
        ldq $80
        aslq q
        stq $90                     ; $2468ACF0 -> F0 AC 68 24
        ldq $80
        lsrq q
        stq $94                     ; $091A2B3C -> 3C 2B 1A 09
        ; --- 32-bit pointer [$nn],z (prefix $EA): four pointer bytes in the zero page, here to $0001_0000 (chip RAM bank 1) ---
        ; Bank 1 of the chip RAM lies outside the 64 KB address space of the CPU and can only be reached through such a pointer.
        lda #$00
        sta $a0
        sta $a1
        sta $a3
        lda #$01
        sta $a2                     ; pointer = $00010000
        ldz #$00
        lda #$5a
        sta [$a0],z                 ; $10000 = $5A
        ldz #$01
        lda #$5b
        sta [$a0],z                 ; $10001 = $5B (Z is added to the pointer)
        lda [$a0],z
        sta $a4                     ; $5B
        ; --- ($nn),z: 16-bit pointer with Z as index (like ($nn),y, but with the Z register) ---
        lda #$99
        sta $3001
        lda #$00
        sta $d0
        lda #$30
        sta $d1                     ; pointer = $3000
        ldz #$01
        lda ($d0),z
        sta $d2                     ; $99 (read from $3000 + Z = $3001)
        ; --- PHW: push a 16-bit value onto the stack ---
        phw #$1234
        pla
        sta $c0                     ; $12 (PHW puts the high byte on top of the stack)
        pla
        sta $c1                     ; $34
        ; --- CPZ / flags ---
        ldz #$05
        lda #$00
        sta $c2
        cpz #$05
        bne nz
        lda #$01
        sta $c2                     ; $01 (equal)
nz:
done:   bra done                    ; endless loop: set a breakpoint here and read the zero page
