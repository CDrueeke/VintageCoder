// C64: hardware registers (VIC-II, SID, CIA 1/2), memory areas
// Constants for the assembler: #include <c64/hardware.asm>

// ---- Processor port
.const CPU_DDR               = $0000   // Data direction of the processor port
.const CPU_PORT              = $0001   // Memory configuration (LORAM/HIRAM/CHAREN) and cassette

// ---- Memory areas
.const SCREEN_RAM            = $0400   // Default screen memory
.const SPRITE_PTRS           = $07F8   // Sprite pointers (with default screen)
.const COLOR_RAM             = $D800   // Color RAM (bits 0-3 only)

// ---- VIC-II
.const VIC_SPR0_X            = $D000   // Sprite 0: X (bits 0-7)
.const VIC_SPR0_Y            = $D001   // Sprite 0: Y
.const VIC_SPR1_X            = $D002   // Sprite 1: X (bits 0-7)
.const VIC_SPR1_Y            = $D003   // Sprite 1: Y
.const VIC_SPR2_X            = $D004   // Sprite 2: X (bits 0-7)
.const VIC_SPR2_Y            = $D005   // Sprite 2: Y
.const VIC_SPR3_X            = $D006   // Sprite 3: X (bits 0-7)
.const VIC_SPR3_Y            = $D007   // Sprite 3: Y
.const VIC_SPR4_X            = $D008   // Sprite 4: X (bits 0-7)
.const VIC_SPR4_Y            = $D009   // Sprite 4: Y
.const VIC_SPR5_X            = $D00A   // Sprite 5: X (bits 0-7)
.const VIC_SPR5_Y            = $D00B   // Sprite 5: Y
.const VIC_SPR6_X            = $D00C   // Sprite 6: X (bits 0-7)
.const VIC_SPR6_Y            = $D00D   // Sprite 6: Y
.const VIC_SPR7_X            = $D00E   // Sprite 7: X (bits 0-7)
.const VIC_SPR7_Y            = $D00F   // Sprite 7: Y
.const VIC_SPR_X_MSB         = $D010   // X bit 8 of all sprites
.const VIC_CTRL1             = $D011   // Control register 1 (Y scroll, 24/25 rows, bitmap, ECM, raster line bit 8)
.const VIC_RASTER            = $D012   // Raster line (read) / raster interrupt line (write)
.const VIC_LIGHTPEN_X        = $D013   // Light pen X
.const VIC_LIGHTPEN_Y        = $D014   // Light pen Y
.const VIC_SPR_ENABLE        = $D015   // Enable sprites
.const VIC_CTRL2             = $D016   // Control register 2 (X scroll, 38/40 columns, multicolor)
.const VIC_SPR_EXPAND_Y      = $D017   // Sprites double height
.const VIC_MEMORY            = $D018   // Character set and screen memory pointers
.const VIC_IRQ_FLAGS         = $D019   // IRQ sources (writing 1 clears)
.const VIC_IRQ_ENABLE        = $D01A   // Enable IRQ sources
.const VIC_SPR_PRIORITY      = $D01B   // Sprites behind the background
.const VIC_SPR_MULTICOLOR    = $D01C   // Sprites multicolor
.const VIC_SPR_EXPAND_X      = $D01D   // Sprites double width
.const VIC_SPR_COLL_SPR      = $D01E   // Collision sprite-sprite (reading clears)
.const VIC_SPR_COLL_BG       = $D01F   // Collision sprite-background (reading clears)
.const VIC_BORDERCOLOR       = $D020   // Border color
.const VIC_BGCOLOR0          = $D021   // Background color 0
.const VIC_BGCOLOR1          = $D022   // Background color 1 (multicolor/ECM)
.const VIC_BGCOLOR2          = $D023   // Background color 2
.const VIC_BGCOLOR3          = $D024   // Background color 3
.const VIC_SPR_MCOLOR0       = $D025   // Sprite multicolor 0
.const VIC_SPR_MCOLOR1       = $D026   // Sprite multicolor 1
.const VIC_SPR0_COLOR        = $D027   // Color of sprite 0
.const VIC_SPR1_COLOR        = $D028   // Color of sprite 1
.const VIC_SPR2_COLOR        = $D029   // Color of sprite 2
.const VIC_SPR3_COLOR        = $D02A   // Color of sprite 3
.const VIC_SPR4_COLOR        = $D02B   // Color of sprite 4
.const VIC_SPR5_COLOR        = $D02C   // Color of sprite 5
.const VIC_SPR6_COLOR        = $D02D   // Color of sprite 6
.const VIC_SPR7_COLOR        = $D02E   // Color of sprite 7

// ---- SID
.const SID_V1_FREQ_LO        = $D400   // Voice 1: frequency low
.const SID_V1_FREQ_HI        = $D401   // Voice 1: frequency high
.const SID_V1_PW_LO          = $D402   // Voice 1: pulse width low
.const SID_V1_PW_HI          = $D403   // Voice 1: pulse width high (4 bits)
.const SID_V1_CTRL           = $D404   // Voice 1: waveform and gate
.const SID_V1_AD             = $D405   // Voice 1: attack/decay
.const SID_V1_SR             = $D406   // Voice 1: sustain/release
.const SID_V2_FREQ_LO        = $D407   // Voice 2: frequency low
.const SID_V2_FREQ_HI        = $D408   // Voice 2: frequency high
.const SID_V2_PW_LO          = $D409   // Voice 2: pulse width low
.const SID_V2_PW_HI          = $D40A   // Voice 2: pulse width high (4 bits)
.const SID_V2_CTRL           = $D40B   // Voice 2: waveform and gate
.const SID_V2_AD             = $D40C   // Voice 2: attack/decay
.const SID_V2_SR             = $D40D   // Voice 2: sustain/release
.const SID_V3_FREQ_LO        = $D40E   // Voice 3: frequency low
.const SID_V3_FREQ_HI        = $D40F   // Voice 3: frequency high
.const SID_V3_PW_LO          = $D410   // Voice 3: pulse width low
.const SID_V3_PW_HI          = $D411   // Voice 3: pulse width high (4 bits)
.const SID_V3_CTRL           = $D412   // Voice 3: waveform and gate
.const SID_V3_AD             = $D413   // Voice 3: attack/decay
.const SID_V3_SR             = $D414   // Voice 3: sustain/release
.const SID_FILTER_LO         = $D415   // Filter cutoff frequency (bits 0-2)
.const SID_FILTER_HI         = $D416   // Filter cutoff frequency (bits 3-10)
.const SID_RES_FILT          = $D417   // Resonance and filter routing
.const SID_MODE_VOL          = $D418   // Filter mode and volume
.const SID_POTX              = $D419   // Paddle X (read)
.const SID_POTY              = $D41A   // Paddle Y (read)
.const SID_OSC3              = $D41B   // Oscillator 3 (read)
.const SID_ENV3              = $D41C   // Envelope 3 (read)

// ---- SID: bits in the control register
.const SID_GATE              = $0001   // Ctrl: gate
.const SID_SYNC              = $0002   // Ctrl: sync
.const SID_RING              = $0004   // Ctrl: ring modulation
.const SID_TEST              = $0008   // Ctrl: test
.const SID_TRIANGLE          = $0010   // Ctrl: triangle
.const SID_SAW               = $0020   // Ctrl: sawtooth
.const SID_PULSE             = $0040   // Ctrl: pulse
.const SID_NOISE             = $0080   // Ctrl: noise

// ---- CIA 1 (keyboard, joystick, IRQ)
.const CIA1_PRA              = $DC00   // Port A
.const CIA1_PRB              = $DC01   // Port B
.const CIA1_DDRA             = $DC02   // Data direction A
.const CIA1_DDRB             = $DC03   // Data direction B
.const CIA1_TA_LO            = $DC04   // Timer A low
.const CIA1_TA_HI            = $DC05   // Timer A high
.const CIA1_TB_LO            = $DC06   // Timer B low
.const CIA1_TB_HI            = $DC07   // Timer B high
.const CIA1_TOD_10THS        = $DC08   // Clock: tenths of seconds
.const CIA1_TOD_SEC          = $DC09   // Clock: seconds
.const CIA1_TOD_MIN          = $DC0A   // Clock: minutes
.const CIA1_TOD_HRS          = $DC0B   // Clock: hours
.const CIA1_SDR              = $DC0C   // Serial shift register
.const CIA1_ICR              = $DC0D   // Interrupt control/status
.const CIA1_CRA              = $DC0E   // Control register A
.const CIA1_CRB              = $DC0F   // Control register B

// ---- CIA 2 (IEC, user port, VIC bank, NMI)
.const CIA2_PRA              = $DD00   // Port A
.const CIA2_PRB              = $DD01   // Port B
.const CIA2_DDRA             = $DD02   // Data direction A
.const CIA2_DDRB             = $DD03   // Data direction B
.const CIA2_TA_LO            = $DD04   // Timer A low
.const CIA2_TA_HI            = $DD05   // Timer A high
.const CIA2_TB_LO            = $DD06   // Timer B low
.const CIA2_TB_HI            = $DD07   // Timer B high
.const CIA2_TOD_10THS        = $DD08   // Clock: tenths of seconds
.const CIA2_TOD_SEC          = $DD09   // Clock: seconds
.const CIA2_TOD_MIN          = $DD0A   // Clock: minutes
.const CIA2_TOD_HRS          = $DD0B   // Clock: hours
.const CIA2_SDR              = $DD0C   // Serial shift register
.const CIA2_ICR              = $DD0D   // Interrupt control/status
.const CIA2_CRA              = $DD0E   // Control register A
.const CIA2_CRB              = $DD0F   // Control register B

// ---- Colors
.const COL_BLACK             = $0000
.const COL_WHITE             = $0001
.const COL_RED               = $0002
.const COL_CYAN              = $0003
.const COL_PURPLE            = $0004
.const COL_GREEN             = $0005
.const COL_BLUE              = $0006
.const COL_YELLOW            = $0007
.const COL_ORANGE            = $0008
.const COL_BROWN             = $0009
.const COL_LIGHTRED          = $000A
.const COL_DARKGRAY          = $000B
.const COL_MIDGRAY           = $000C
.const COL_LIGHTGREEN        = $000D
.const COL_LIGHTBLUE         = $000E
.const COL_LIGHTGRAY         = $000F
