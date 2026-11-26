// BASIC to assembler (basic2asm), runtime for Atari 8-bit: zero page, variables and memory limits.
// Include with: #include <atari8bit/b2a/rt_zp.asm> (at the start of the program, before any code); #include <atari8bit/os.asm> has already been done by then.
//
// The runtime uses no routine of the Atari BASIC module; floating point (stage 2) computes with the operating system's floating-point package (FR0 $D4-$D9, FR1 $E0-$E5 and the auxiliary cells $DA-$DF, $E6-$FF). The zero page from $80 on belongs
// to the BASIC module; when the program is started as XEX, nobody needs these cells any more. The block is saved at start anyway and
// written back at the end (RT_ZSAVE), so that a return to DOS or BASIC finds its pointers again.

.const B2A_ATARI = 1
.const RT_ZSAVE = 1
.const RT_ZLO   = $80
.const RT_ZLEN  = 20
.const rt_t0   = $80     // pointer/temporary (2 bytes)
.const rt_ap   = $82     // pointer to array header (2 bytes)
.const rt_dp   = $84     // DATA pointer (2 bytes)
.const rt_s1   = $86     // strings: source/auxiliary pointer (2 bytes)
.const rt_s2   = $88     // strings: destination pointer (2 bytes)
.const rt_gd   = $8A     // garbage collection: descriptor being checked (2 bytes)
.const rt_gq   = $8C     // garbage collection: table pointer (2 bytes)
.const RT_FS   = 6     // bytes per number (6-byte BCD of the OS package)
.const RT_SGN  = 0     // sign bit in byte 0
.const F_FACEXP = FR0  // byte 0 of the FAC: 0 = number is 0
.const MATH_ZP = $8E     // 6 bytes for mul16/div16 (common/math.asm): $8E-$93

// Variables in page 6 ($0600-$06FF is free for user programs)
.const RT_VARS = $0600
.const rt_t1   = RT_VARS+0    // temporary (2 bytes)
.const rt_fv   = RT_VARS+2    // loop variable (2 bytes)
.const rt_bp   = RT_VARS+4    // jump target for FOR/GOSUB (2 bytes)
.const rt_v    = RT_VARS+6    // temporary (2 bytes)
.const rt_esp  = RT_VARS+8    // expression stack pointer
.const rt_ssp  = RT_VARS+9    // stack pointer for GOSUB/FOR
.const rt_sp0  = RT_VARS+10   // CPU stack pointer at start (for END)
.const rt_tmp  = RT_VARS+11   // temporary (1 byte)
.const rt_line = RT_VARS+12   // current BASIC line (2 bytes, for error messages)
.const rt_txt  = RT_VARS+14   // temporary (2 bytes, VAL)

// Upper limit of the program and the string heap: above it lie the display list and screen memory (GRAPHICS 0 at the end of RAM)
.const RT_HEAPTOP = $9800
