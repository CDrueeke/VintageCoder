// BASIC to assembler (basic2asm), runtime for Plus/4 (BASIC 3.5): zero page, variables, memory limits and BASIC ROM entry points.
// Include with: #include <plus4/b2a/rt_zp.asm> (at the start of the program, before any code)
//
// The program calls KERNAL routines (CHROUT, CHRIN, GETIN, PLOT, RDTIM, SETTIM) and - only with floating point (RT_FLOAT) - arithmetic routines of the BASIC ROM.
// Measured in VICE: KERNAL and interrupt do not change the zero page from $02 to $8F. The program's block is saved at start
// and written back at the end (and on a ROM error, via the error vector $0300), so that BASIC finds its pointers again.

.const RT_ZSAVE = 1

.ifdef RT_FLOAT
// ---- Stage 2: the ROM's arithmetic routines use $22-$29, $47, $49-$4A, $56, $5D-$72, $78 (measured from the ROM) - the block lies below that
.const RT_ZLO   = $30
.const RT_ZLEN  = 14
.const rt_t0   = $30     // pointer/temporary (2 bytes)
.const rt_ap   = $32     // pointer to array header (2 bytes)
.const rt_dp   = $34     // DATA pointer (2 bytes)
.const rt_s1   = $36     // strings: source/auxiliary pointer (2 bytes)
.const rt_s2   = $38     // strings: destination pointer (2 bytes)
.const rt_gd   = $3A     // garbage collection: descriptor being checked (2 bytes)
.const rt_gq   = $3C     // garbage collection: table pointer (2 bytes)
.else
// ---- Stage 1 and 3: no ROM calls, block $60-$73
.const RT_ZLO   = $60
.const RT_ZLEN  = 20
.const rt_t0   = $60     // pointer/temporary (2 bytes)
.const rt_ap   = $62     // pointer to array header (2 bytes)
.const rt_dp   = $64     // DATA pointer (2 bytes)
.const rt_s1   = $66     // strings: source/auxiliary pointer (2 bytes)
.const rt_s2   = $68     // strings: destination pointer (2 bytes)
.const rt_gd   = $6A     // garbage collection: descriptor being checked (2 bytes)
.const rt_gq   = $6C     // garbage collection: table pointer (2 bytes)
.const MATH_ZP = $6E     // 6 bytes for mul16/div16 (common/math.asm): $6E-$73
.endif

// Variables in the cassette buffer (free as long as no tape is used)
.const RT_VARS = $0333
.const rt_t1   = RT_VARS+0    // temporary (2 bytes)
.const rt_fv   = RT_VARS+2    // loop variable (2 bytes)
.const rt_bp   = RT_VARS+4    // jump target for FOR/GOSUB (2 bytes)
.const rt_v    = RT_VARS+6    // temporary (2 bytes)
.const rt_esp  = RT_VARS+8    // expression stack pointer
.const rt_ssp  = RT_VARS+9    // stack pointer for GOSUB/FOR
.const rt_sp0  = RT_VARS+10   // CPU stack pointer at start (for END)
.const rt_tmp  = RT_VARS+11   // temporary (1 byte)
.const rt_line = RT_VARS+12   // current BASIC line (2 bytes, for error messages)
.const rt_txt  = RT_VARS+14   // saved BASIC text pointer (2 bytes, INPUT/VAL)

// Upper limit of the string heap and of the program: the BASIC ROM lies above it
.const RT_HEAPTOP = $8000

.ifdef RT_FLOAT
// ---- BASIC ROM routines for stage 2 (BASIC 3.5 (ROM from $8000)). Arithmetic: first load ARG from memory (A/Y) (F_CONUPK), then the "T" entry points.
.const F_FSUBT    = $9E87   // FAC = ARG - FAC
.const F_FADDT    = $9E9E   // FAC = ARG + FAC
.const F_FMULTT   = $A07B   // FAC = ARG * FAC
.const F_FDIVT    = $A197   // FAC = ARG / FAC
.const F_CONUPK   = $A0DC   // ARG = (A/Y)
.const F_FPWRT    = $A5EE   // FAC = ARG ^ FAC
.const F_MOVFM    = $A222   // FAC = (A/Y)
.const F_MOVMF    = $A259   // (X/Y) = FAC (rounded)
.const F_FCOMP    = $A2E0   // compares FAC with (A/Y): 0 equal, 1 FAC greater, $FF FAC smaller
.const F_ROUND    = $A2A0
.const F_NEGOP    = $A627
.const F_ABS      = $A2DD
.const F_SGN      = $A2BE
.const F_INT      = $A358
.const F_SQR      = $A5E4
.const F_LOG      = $A01E
.const F_EXP      = $A660
.const F_SIN      = $AA77
.const F_COS      = $AA70
.const F_TAN      = $AAC0
.const F_ATN      = $AB1A
.const F_RND      = $A707
.const F_FOUT     = $A46F   // FAC -> text, A/Y = pointer
.const F_FIN      = $A37F   // text from TXTPTR -> FAC (A = first character)
.const F_FACEXP  = $61     // exponent of FAC (0 = number is 0)
.const F_CHRGOT  = $0479   // CHRGOT (get text character, flags as CHRGET)
.const F_TXTPTR  = $3B     // BASIC text pointer
.endif
