// VIC-20: VIC-I (6560/6561), two VIA 6522, memory areas
// Constants for the assembler: #include <vic20/hardware.asm>
//
// Screen RAM, color RAM and BASIC start depend on the memory expansion (project setting `memory`): when building, VintageCoder defines
// MEM_SCREEN_RAM, MEM_COLOR_RAM and MEM_BASIC_START; without a setting the unexpanded machine applies.

// ---- Memory areas
.ifdef MEM_SCREEN_RAM
.const SCREEN_RAM              = MEM_SCREEN_RAM   // Screen memory (unexpanded and +3 KB: $1E00, +8 KB and more: $1000)
.else
.const SCREEN_RAM              = $1E00   // Screen memory (unexpanded and +3 KB: $1E00, +8 KB and more: $1000)
.endif
.ifdef MEM_COLOR_RAM
.const COLOR_RAM               = MEM_COLOR_RAM   // Color RAM (bits 0-3 only; unexpanded and +3 KB: $9600, +8 KB and more: $9400)
.else
.const COLOR_RAM               = $9600   // Color RAM (bits 0-3 only; unexpanded and +3 KB: $9600, +8 KB and more: $9400)
.endif
.ifdef MEM_BASIC_START
.const BASIC_START             = MEM_BASIC_START   // BASIC program start (unexpanded $1001, +3 KB $0401, +8 KB and more $1201)
.else
.const BASIC_START             = $1001   // BASIC program start (unexpanded $1001, +3 KB $0401, +8 KB and more $1201)
.endif
.const CHAR_ROM                = $8000   // Character ROM (uppercase/graphics, then lowercase/uppercase)
.const RAM_BLOCK1              = $2000   // Expansion block 1 (8 KB)
.const RAM_BLOCK2              = $4000   // Expansion block 2 (8 KB)
.const RAM_BLOCK3              = $6000   // Expansion block 3 (8 KB)
.const CART_BLOCK5             = $A000   // Cartridge (autostart signature A0CBM at $A004)

// ---- VIC-I
.const VIC_ORIGIN_X               = $9000   // Horizontal screen origin (bits 0-6), bit 7 = interlace
.const VIC_ORIGIN_Y               = $9001   // Vertical screen origin
.const VIC_COLS                   = $9002   // Number of columns (bits 0-6), bit 7 = screen address bit 9
.const VIC_ROWS                   = $9003   // Number of rows (bits 1-6), bit 0 = 8x16 characters, bit 7 = raster line bit 0
.const VIC_RASTER                 = $9004   // Raster line (bits 8-1, read only)
.const VIC_MEMPTR                 = $9005   // Memory pointer: bits 4-7 screen, bits 0-3 character set
.const VIC_LPX                    = $9006   // Light pen X
.const VIC_LPY                    = $9007   // Light pen Y
.const VIC_POTX                   = $9008   // Paddle X
.const VIC_POTY                   = $9009   // Paddle Y
.const VIC_BASS                   = $900A   // Bass voice (bit 7 = on)
.const VIC_ALTO                   = $900B   // Alto voice (bit 7 = on)
.const VIC_SOPRANO                = $900C   // Soprano voice (bit 7 = on)
.const VIC_NOISE                  = $900D   // Noise (bit 7 = on)
.const VIC_VOLUME                 = $900E   // Volume (bits 0-3), auxiliary color (bits 4-7)
.const VIC_COLOR                  = $900F   // Background (bits 4-7), normal/inverse (bit 3), border (bits 0-2)

// ---- VIA 1 (NMI, user port, joystick, serial bus)
.const VIA1_ORB                   = $9110   // Port B
.const VIA1_ORA                   = $9111   // Port A
.const VIA1_DDRB                  = $9112   // Data direction B
.const VIA1_DDRA                  = $9113   // Data direction A
.const VIA1_T1C_LO                = $9114   // Timer 1 counter low
.const VIA1_T1C_HI                = $9115   // Timer 1 counter high
.const VIA1_T1L_LO                = $9116   // Timer 1 latch low
.const VIA1_T1L_HI                = $9117   // Timer 1 latch high
.const VIA1_T2C_LO                = $9118   // Timer 2 counter low
.const VIA1_T2C_HI                = $9119   // Timer 2 counter high
.const VIA1_SR                    = $911A   // Shift register
.const VIA1_ACR                   = $911B   // Auxiliary control register
.const VIA1_PCR                   = $911C   // Peripheral control register
.const VIA1_IFR                   = $911D   // Interrupt flags
.const VIA1_IER                   = $911E   // Interrupt enable
.const VIA1_ORA_NH                = $911F   // Port A without handshake

// ---- VIA 2 (IRQ timer, keyboard)
.const VIA2_ORB                   = $9120   // Port B
.const VIA2_ORA                   = $9121   // Port A
.const VIA2_DDRB                  = $9122   // Data direction B
.const VIA2_DDRA                  = $9123   // Data direction A
.const VIA2_T1C_LO                = $9124   // Timer 1 counter low
.const VIA2_T1C_HI                = $9125   // Timer 1 counter high
.const VIA2_T1L_LO                = $9126   // Timer 1 latch low
.const VIA2_T1L_HI                = $9127   // Timer 1 latch high
.const VIA2_T2C_LO                = $9128   // Timer 2 counter low
.const VIA2_T2C_HI                = $9129   // Timer 2 counter high
.const VIA2_SR                    = $912A   // Shift register
.const VIA2_ACR                   = $912B   // Auxiliary control register
.const VIA2_PCR                   = $912C   // Peripheral control register
.const VIA2_IFR                   = $912D   // Interrupt flags
.const VIA2_IER                   = $912E   // Interrupt enable
.const VIA2_ORA_NH                = $912F   // Port A without handshake

// ---- Colors (character color 0-7; background and auxiliary color 0-15)
.const COL_BLACK                  = $0000
.const COL_WHITE                  = $0001
.const COL_RED                    = $0002
.const COL_CYAN                   = $0003
.const COL_PURPLE                 = $0004
.const COL_GREEN                  = $0005
.const COL_BLUE                   = $0006
.const COL_YELLOW                 = $0007
.const COL_ORANGE                 = $0008
.const COL_LIGHTORANGE            = $0009
.const COL_PINK                   = $000A
.const COL_LIGHTCYAN              = $000B
.const COL_LIGHTPURPLE            = $000C
.const COL_LIGHTGREEN             = $000D
.const COL_LIGHTBLUE              = $000E
.const COL_LIGHTYELLOW            = $000F
