// BASIC->assembler (basic2asm), C128 runtime (BASIC 7, C128 mode): zero page, variables, memory limits and BASIC ROM entry points.
// Include with: #include <c128/b2a/rt_zp.asm> (at the start of the program, before any code)
//
// The program calls KERNAL routines (CHROUT, CHRIN, GETIN, PLOT, RDTIM, SETTIM) and - only with floating point (RT_FLOAT) - arithmetic routines of the BASIC ROM.
// Measured in VICE: KERNAL and interrupt do not change the zero page from $02 to $8F. The program's block is saved at start
// and written back at the end (and on a ROM error, via the error vector $0300) so that BASIC finds its pointers again.

.const RT_ZSAVE = 1

.ifdef RT_FLOAT
// ---- Level 2: the ROM arithmetic routines use $24-$2B, $49, $58, $5F-$73, $79 (measured from the ROM) - the block lies below them
.const RT_ZLO   = $30
.const RT_ZLEN  = 14
.const rt_t0   = $30     // pointer/scratch (2 bytes)
.const rt_ap   = $32     // pointer to array header (2 bytes)
.const rt_dp   = $34     // DATA pointer (2 bytes)
.const rt_s1   = $36     // strings: source/work pointer (2 bytes)
.const rt_s2   = $38     // strings: destination pointer (2 bytes)
.const rt_gd   = $3A     // garbage collection: descriptor under examination (2 bytes)
.const rt_gq   = $3C     // garbage collection: table pointer (2 bytes)
.else
// ---- Levels 1 and 3: no ROM call, block $60-$73
.const RT_ZLO   = $60
.const RT_ZLEN  = 20
.const rt_t0   = $60     // pointer/scratch (2 bytes)
.const rt_ap   = $62     // pointer to array header (2 bytes)
.const rt_dp   = $64     // DATA pointer (2 bytes)
.const rt_s1   = $66     // strings: source/work pointer (2 bytes)
.const rt_s2   = $68     // strings: destination pointer (2 bytes)
.const rt_gd   = $6A     // garbage collection: descriptor under examination (2 bytes)
.const rt_gq   = $6C     // garbage collection: table pointer (2 bytes)
.const MATH_ZP = $6E     // 6 bytes for mul16/div16 (common/math.asm): $6E-$73
.endif

// Variables in the cassette buffer (free as long as no tape is used)
.const RT_VARS = $0B00
.const rt_t1   = RT_VARS+0    // scratch (2 bytes)
.const rt_fv   = RT_VARS+2    // loop variable (2 bytes)
.const rt_bp   = RT_VARS+4    // jump target for FOR/GOSUB (2 bytes)
.const rt_v    = RT_VARS+6    // scratch (2 bytes)
.const rt_esp  = RT_VARS+8    // expression stack pointer
.const rt_ssp  = RT_VARS+9    // stack pointer for GOSUB/FOR
.const rt_sp0  = RT_VARS+10   // CPU stack pointer at start (for END)
.const rt_tmp  = RT_VARS+11   // scratch (1 byte)
.const rt_line = RT_VARS+12   // current BASIC line (2 bytes, for error messages)
.const rt_txt  = RT_VARS+14   // saved BASIC text pointer (2 bytes, INPUT/VAL)

// Upper limit of the string heap and of the program: the BASIC ROM lies above it
.const RT_HEAPTOP = $4000

.ifdef RT_FLOAT
// ---- BASIC ROM routines for level 2 (BASIC 7 (ROM from $4000, bank 0)). Arithmetic: first load ARG from memory (A/Y) (F_CONUPK), then the "T" entry points.
.const F_FSUBT    = $8831   // FAC = ARG - FAC
.const F_FADDT    = $8848   // FAC = ARG + FAC
.const F_FMULTT   = $8A27   // FAC = ARG * FAC
.const F_FDIVT    = $8B4C   // FAC = ARG / FAC
.const F_CONUPK   = $8A89   // ARG = (A/Y)
.const F_FPWRT    = $8FC1   // FAC = ARG ^ FAC
.const F_MOVFM    = $8BD4   // FAC = (A/Y)
.const F_MOVMF    = $8C00   // (X/Y) = FAC (rounded)
.const F_FCOMP    = $8C87   // compares FAC with (A/Y): 0 equal, 1 FAC greater, $FF FAC smaller
.const F_ROUND    = $8C47
.const F_NEGOP    = $8FFA
.const F_ABS      = $8C84
.const F_SGN      = $8C65
.const F_INT      = $8CFB
.const F_SQR      = $8FB7
.const F_LOG      = $89CA
.const F_EXP      = $9033
.const F_SIN      = $9410
.const F_COS      = $9409
.const F_TAN      = $9459
.const F_ATN      = $94B3
.const F_RND      = $8434
.const F_FOUT     = $8E42   // FAC -> text, A/Y = pointer
.const F_FIN      = $8D25   // text from TXTPTR -> FAC (A = first character)
.const F_FACEXP  = $63     // FAC exponent (0 = number is 0)
.const F_CHRGOT  = $0386   // CHRGOT (get text character, flags as CHRGET)
.const F_TXTPTR  = $3D     // BASIC text pointer
.endif
