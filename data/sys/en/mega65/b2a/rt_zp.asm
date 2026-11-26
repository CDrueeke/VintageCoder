// BASIC to assembler (basic2asm), runtime for MEGA65 (BASIC 65, program at $2001): zero page, variables and memory limits. Stage 1 and 3 only (integer and
// strings): the ROM offers no usable floating-point routines (see concept, section 10.4 and rom-disasm).
// Include with: #include <mega65/b2a/rt_zp.asm> (at the start of the program, before any code)
//
// Checked in Xemu (Windows), NOT on the real machine: the test `probe_zeropage` in crates/vc-ide/tests/b2a_mega65_live.rs measures that KERNAL and interrupt change only
// $C4/$C5, $DA, $E0/$E1 and $EA/$EB while running; the chosen block ($40-$53, as in the example hello.asm) and the variables ($1F00, below the BASIC text) are free. The program's block is saved at start and written back at the end (as on C128 and Plus/4).

.const RT_ZSAVE = 1
.const RT_ZLO   = $40
.const RT_ZLEN  = 20
.const rt_t0   = $40     // pointer/temporary (2 bytes)
.const rt_ap   = $42     // pointer to array header (2 bytes)
.const rt_dp   = $44     // DATA pointer (2 bytes)
.const rt_s1   = $46     // strings: source/auxiliary pointer (2 bytes)
.const rt_s2   = $48     // strings: destination pointer (2 bytes)
.const rt_gd   = $4A     // garbage collection: descriptor being checked (2 bytes)
.const rt_gq   = $4C     // garbage collection: table pointer (2 bytes)
.const MATH_ZP = $4E     // 6 bytes for mul16/div16 (common/math.asm): $4E-$53

// Variables in RAM below the BASIC text ($2001)
.const RT_VARS = $1F00
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

// Upper limit of the string heap: below the I/O ($D000) and the ROM window
.const RT_HEAPTOP = $7F00
