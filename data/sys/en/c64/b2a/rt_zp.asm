// BASIC->assembler (basic2asm), C64 runtime: zero page and constants.
// The result calls no BASIC ROM; the BASIC pointers ($2B-$38, $73-$8A) stay untouched so that "END" returns cleanly to BASIC.
// Include with: #include <c64/b2a/rt_zp.asm> (at the start of the program, before any code)

.ifdef RT_FLOAT
// ---- Level 2 (floating point): the BASIC ROM calculates with $22-$2A and $57-$70 (FAC, ARG, work areas) - these areas stay free.
// Only pointers that are used indirectly are in the zero page ($F7-$FE); everything else in the cassette buffer from $0334.
.const rt_t0   = $FB     // pointer/scratch (2 bytes, zero page)
.const rt_ap   = $FD     // pointer to array header (2 bytes, zero page)
.const rt_dp   = $F7     // DATA pointer (2 bytes, zero page)
.const rt_line = $39     // = CURLIN of BASIC: ROM error messages thus name the correct line
.const rt_t1   = $0334   // scratch (2 bytes)
.const rt_fv   = $0336   // pointer to loop variable (2 bytes)
.const rt_bp   = $0338   // jump target for FOR/GOSUB (2 bytes)
.const rt_v    = $033A   // scratch (2 bytes)
.const rt_esp  = $033C   // expression stack pointer (bytes)
.const rt_ssp  = $033D   // stack pointer for GOSUB/FOR
.const rt_sp0  = $033E   // CPU stack pointer at start (for END)
.const rt_tmp  = $033F   // scratch (1 byte)
.const rt_txt  = $0340   // saved BASIC text pointer (2 bytes, INPUT)
.else
.const rt_t0   = $22     // scratch 0 (2 bytes): left operand, pointer
.const rt_t1   = $24     // scratch 1 (2 bytes): right operand
.const rt_ap   = $26     // pointer to array header (2 bytes)
.const rt_fv   = $28     // pointer to loop variable (2 bytes)
.const MATH_ZP = $57     // 6 bytes for mul16/div16 (common/math.asm): $57-$5C
.const rt_esp  = $5D     // expression stack pointer
.const rt_ssp  = $5E     // stack pointer for GOSUB/FOR
.const rt_sp0  = $5F     // CPU stack pointer at start (for END)
.const rt_tmp  = $60     // scratch (1 byte)
.const rt_line = $F7     // current BASIC line (2 bytes, for error messages)
.const rt_v    = $F9     // scratch (2 bytes)
.const rt_bp   = $FB     // jump target for FOR/GOSUB (2 bytes)
.const rt_dp   = $FD     // DATA pointer (2 bytes)

.endif

// Strings (level 3, rt_str.asm): pointers from the BASIC area of the zero page that neither level 1 nor the floating-point ROM needs
.const rt_s1   = $3F     // source/work pointer (2 bytes)
.const rt_s2   = $41     // destination pointer (2 bytes)
.const rt_gd   = $43     // garbage collection: descriptor under examination (2 bytes)
.const rt_gq   = $45     // garbage collection: table pointer (2 bytes)

// Upper limit of the string heap (level 3) and the limit up to which the program may extend: the heap grows downward from here
.const RT_HEAPTOP = $A000

// ---- BASIC ROM routines for level 2 (floating point, FAC = floating-point accumulator 1 at $61)
// Arithmetic: first load ARG from memory (A/Y) (F_CONUPK), then the "T" entry points: FAC = ARG op FAC. On the C128 the memory forms
// (FADD etc.) read from bank 1; CONUPK reads from the current memory, so this pattern works on all targets.
.const F_FSUBT   = $B853   // FAC = ARG - FAC
.const F_FADDT   = $B86A   // FAC = ARG + FAC
.const F_FMULTT  = $BA2B   // FAC = ARG * FAC
.const F_FDIVT   = $BB12   // FAC = ARG / FAC
.const F_CONUPK  = $BA8C   // ARG = (A/Y)
.const F_FPWRT   = $BF7B   // FAC = ARG ^ FAC
.const F_MOVFM   = $BBA2   // FAC = (A/Y)
.const F_MOVMF   = $BBD4   // (X/Y) = FAC (rounded)
.const F_FCOMP   = $BC5B   // compares FAC with (A/Y): 0 equal, 1 FAC greater, $FF FAC smaller
.const F_ROUND   = $BC1B
.const F_NEGOP   = $BFB4
.const F_ABS     = $BC58
.const F_SGN     = $BC39
.const F_INT     = $BCCC
.const F_SQR     = $BF71
.const F_LOG     = $B9EA
.const F_EXP     = $BFED
.const F_SIN     = $E26B
.const F_COS     = $E264
.const F_TAN     = $E2B4
.const F_ATN     = $E30E
.const F_RND     = $E097
.const F_FACEXP  = $61     // FAC exponent (0 = number is 0)
.const F_FOUT    = $BDDD   // FAC -> text at $0100, A/Y = pointer
.const F_FIN     = $BCF3   // text from TXTPTR -> FAC (A = first character)
.const F_CHRGOT  = $0079
.const F_TXTPTR  = $007A
