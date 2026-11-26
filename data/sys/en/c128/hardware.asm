// C128: hardware registers (VIC-IIe extensions, MMU, VDC); VIC-II, SID and CIA are in c64/hardware.asm
// Constants for the assembler: #include <c128/hardware.asm>

#include <c64/hardware.asm>

// ---- VIC-IIe
.const VIC_KEYSCAN             = $D02F   // Additional keyboard rows (bits 0-2)
.const VIC_CLOCK               = $D030   // Bit 0: 2 MHz mode

// ---- MMU ($D500, always in the I/O area)
.const MMU_CR                  = $D500   // Configuration register (bank/ROM view)
.const MMU_PCRA                = $D501   // Preconfiguration A
.const MMU_PCRB                = $D502   // Preconfiguration B
.const MMU_PCRC                = $D503   // Preconfiguration C
.const MMU_PCRD                = $D504   // Preconfiguration D
.const MMU_MCR                 = $D505   // Operating mode (40/80, C64 mode, processor)
.const MMU_RCR                 = $D506   // RAM configuration (common area, VIC bank)
.const MMU_P0L                 = $D507   // Zero page pointer low
.const MMU_P0H                 = $D508   // Zero page pointer high (bank)
.const MMU_P1L                 = $D509   // Stack page pointer low
.const MMU_P1H                 = $D50A   // Stack page pointer high (bank)
.const MMU_VR                  = $D50B   // MMU version

// ---- MMU mirror ($FF00, always visible)
.const MMU_CR_FF00             = $FF00   // Configuration register (always visible)
.const MMU_LCRA                = $FF01   // Load configuration A (writing is enough)
.const MMU_LCRB                = $FF02   // Load configuration B
.const MMU_LCRC                = $FF03   // Load configuration C
.const MMU_LCRD                = $FF04   // Load configuration D

// ---- Default configurations for MMU_CR_FF00
.const MMUCFG_ALL_ROM          = $00   // Bank 15: system ROMs and I/O (default)
.const MMUCFG_RAM0_IO          = $3E   // Bank 0 RAM with I/O
.const MMUCFG_RAM0             = $3F   // Bank 0 RAM, no I/O
.const MMUCFG_RAM1_IO          = $7E   // Bank 1 RAM with I/O
.const MMUCFG_RAM1             = $7F   // Bank 1 RAM, no I/O

// ---- VDC 8563/8568 (80 columns)
.const VDC_ADDR                = $D600   // Register select (write) / status (read)
.const VDC_DATA                = $D601   // Register data

// ---- VDC register numbers (write to VDC_ADDR)
.const VDCR_HTOTAL             = $00   // VDC register 0
.const VDCR_HDISPLAYED         = $01   // VDC register 1
.const VDCR_HSYNC_POS          = $02   // VDC register 2
.const VDCR_SYNC_WIDTH         = $03   // VDC register 3
.const VDCR_VTOTAL             = $04   // VDC register 4
.const VDCR_VTOTAL_ADJ         = $05   // VDC register 5
.const VDCR_VDISPLAYED         = $06   // VDC register 6
.const VDCR_VSYNC_POS          = $07   // VDC register 7
.const VDCR_INTERLACE          = $08   // VDC register 8
.const VDCR_CHAR_HEIGHT        = $09   // VDC register 9
.const VDCR_CURSOR_START       = $0A   // VDC register 10
.const VDCR_CURSOR_END         = $0B   // VDC register 11
.const VDCR_DISPLAY_HI         = $0C   // VDC register 12
.const VDCR_DISPLAY_LO         = $0D   // VDC register 13
.const VDCR_CURSOR_HI          = $0E   // VDC register 14
.const VDCR_CURSOR_LO          = $0F   // VDC register 15
.const VDCR_LIGHTPEN_V         = $10   // VDC register 16
.const VDCR_LIGHTPEN_H         = $11   // VDC register 17
.const VDCR_UPDATE_HI          = $12   // VDC register 18
.const VDCR_UPDATE_LO          = $13   // VDC register 19
.const VDCR_ATTR_HI            = $14   // VDC register 20
.const VDCR_ATTR_LO            = $15   // VDC register 21
.const VDCR_CHAR_WIDTH         = $16   // VDC register 22
.const VDCR_CHAR_DISPLAYED     = $17   // VDC register 23
.const VDCR_VSCROLL            = $18   // VDC register 24
.const VDCR_HSCROLL            = $19   // VDC register 25
.const VDCR_COLOR              = $1A   // VDC register 26
.const VDCR_ROW_INCREMENT      = $1B   // VDC register 27
.const VDCR_CHARSET_BASE       = $1C   // VDC register 28
.const VDCR_UNDERLINE          = $1D   // VDC register 29
.const VDCR_WORD_COUNT         = $1E   // VDC register 30
.const VDCR_DATA               = $1F   // VDC register 31
.const VDCR_COPY_SRC_HI        = $20   // VDC register 32
.const VDCR_COPY_SRC_LO        = $21   // VDC register 33
.const VDCR_DISPLAY_ENABLE_BEGIN= $22   // VDC register 34
.const VDCR_DISPLAY_ENABLE_END = $23   // VDC register 35
.const VDCR_REFRESH            = $24   // VDC register 36

// ---- VDC colors (RGBI)
.const VDC_COL_BLACK           = $00
.const VDC_COL_DARKGRAY        = $01
.const VDC_COL_DARKBLUE        = $02
.const VDC_COL_LIGHTBLUE       = $03
.const VDC_COL_DARKGREEN       = $04
.const VDC_COL_LIGHTGREEN      = $05
.const VDC_COL_DARKCYAN        = $06
.const VDC_COL_LIGHTCYAN       = $07
.const VDC_COL_DARKRED         = $08
.const VDC_COL_LIGHTRED        = $09
.const VDC_COL_DARKPURPLE      = $0A
.const VDC_COL_LIGHTPURPLE     = $0B
.const VDC_COL_DARKYELLOW      = $0C
.const VDC_COL_LIGHTYELLOW     = $0D
.const VDC_COL_LIGHTGRAY       = $0E
.const VDC_COL_WHITE           = $0F
