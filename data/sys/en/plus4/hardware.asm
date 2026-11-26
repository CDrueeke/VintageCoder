// Plus/4 and C16: TED, ACIA 6551, 6529 ports, ROM bank select
// Constants for the assembler: #include <plus4/hardware.asm>

// ---- Processor port
.const CPU_DDR                 = $0000   // Data direction of the processor port
.const CPU_PORT                = $0001   // Cassette, serial bus

// ---- Memory areas
.const COLOR_RAM               = $0800   // Color memory
.const SCREEN_RAM              = $0C00   // Default screen memory
.const BASIC_START             = $1001   // BASIC program start

// ---- TED
.const TED_TIMER1_LO           = $FF00   // Timer 1 low
.const TED_TIMER1_HI           = $FF01   // Timer 1 high
.const TED_TIMER2_LO           = $FF02   // Timer 2 low
.const TED_TIMER2_HI           = $FF03   // Timer 2 high
.const TED_TIMER3_LO           = $FF04   // Timer 3 low
.const TED_TIMER3_HI           = $FF05   // Timer 3 high
.const TED_CTRL1               = $FF06   // Control register 1 (Y scroll, 24/25 rows, bitmap, ECM)
.const TED_CTRL2               = $FF07   // Control register 2 (X scroll, 38/40 columns, multicolor, PAL/NTSC)
.const TED_KEYLATCH            = $FF08   // Keyboard/joystick scan
.const TED_IRQ_FLAGS           = $FF09   // IRQ sources (writing 1 clears)
.const TED_IRQ_ENABLE          = $FF0A   // Enable IRQ sources, raster line bit 8
.const TED_RASTER              = $FF0B   // Raster IRQ line (bits 0-7)
.const TED_CURSOR_HI           = $FF0C   // Cursor position high
.const TED_CURSOR_LO           = $FF0D   // Cursor position low
.const TED_VOICE1_LO           = $FF0E   // Voice 1: frequency low
.const TED_VOICE2_LO           = $FF0F   // Voice 2: frequency low
.const TED_VOICE2_HI           = $FF10   // Voice 2: frequency high
.const TED_SOUND               = $FF11   // Volume and voice selection
.const TED_BITMAP_BASE         = $FF12   // Bitmap address, voice 1 high, character set RAM/ROM
.const TED_CHARSET_BASE        = $FF13   // Character set address, fast mode
.const TED_SCREEN_BASE         = $FF14   // Screen memory address
.const TED_BGCOLOR             = $FF15   // Background color 0
.const TED_COLOR1              = $FF16   // Background color 1
.const TED_COLOR2              = $FF17   // Background color 2
.const TED_COLOR3              = $FF18   // Background color 3
.const TED_BORDERCOLOR         = $FF19   // Border color
.const TED_CHARROW_HI          = $FF1A   // Character row high
.const TED_CHARROW_LO          = $FF1B   // Character row low
.const TED_RASTER_HI           = $FF1C   // Current raster line bit 8
.const TED_RASTER_LO           = $FF1D   // Current raster line bits 0-7
.const TED_HCOUNT              = $FF1E   // Horizontal counter
.const TED_BLINK               = $FF1F   // Blink counter, character row 0-7
.const TED_ROM_ON              = $FF3E   // Switch ROM in (write)
.const TED_RAM_ON              = $FF3F   // Switch RAM in (write)

// ---- TED: hues (color value = hue + 16 * luminance 0-7)
.const TED_COL_BLACK           = $00
.const TED_COL_WHITE           = $01
.const TED_COL_RED             = $02
.const TED_COL_CYAN            = $03
.const TED_COL_PURPLE          = $04
.const TED_COL_GREEN           = $05
.const TED_COL_BLUE            = $06
.const TED_COL_YELLOW          = $07
.const TED_COL_ORANGE          = $08
.const TED_COL_BROWN           = $09
.const TED_COL_YELLOWGREEN     = $0A
.const TED_COL_PINK            = $0B
.const TED_COL_BLUEGREEN       = $0C
.const TED_COL_LIGHTBLUE       = $0D
.const TED_COL_DARKBLUE        = $0E
.const TED_COL_LIGHTGREEN      = $0F

// ---- ACIA 6551 (Plus/4 only)
.const ACIA_DATA               = $FD00   // Data
.const ACIA_STATUS             = $FD01   // Status (read) / reset (write)
.const ACIA_COMMAND            = $FD02   // Command register
.const ACIA_CONTROL            = $FD03   // Control register

// ---- 6529 ports
.const PIO_PORT                = $FD10   // Port 6529 (user port, cassette status)
.const PIO_KEYBOARD            = $FD30   // Keyboard matrix and joystick column select

// ---- ROM bank select
.const ROM_BANK                = $FDD0   // Select function ROM (write)
