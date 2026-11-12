// C128: Hardware-Register (VIC-IIe-Erweiterungen, MMU, VDC); VIC-II, SID und CIA stehen in c64/hardware.asm
// Konstanten für den Assembler: #include <c128/hardware.asm>

#include <c64/hardware.asm>

// ---- VIC-IIe
.const VIC_KEYSCAN             = $D02F   // Zusätzliche Tastaturzeilen (Bit 0-2)
.const VIC_CLOCK               = $D030   // Bit 0: 2-MHz-Betrieb

// ---- MMU ($D500, immer im E/A-Bereich)
.const MMU_CR                  = $D500   // Konfigurationsregister (Bank/ROM-Sicht)
.const MMU_PCRA                = $D501   // Vorkonfiguration A
.const MMU_PCRB                = $D502   // Vorkonfiguration B
.const MMU_PCRC                = $D503   // Vorkonfiguration C
.const MMU_PCRD                = $D504   // Vorkonfiguration D
.const MMU_MCR                 = $D505   // Betriebsart (40/80, C64-Modus, Prozessor)
.const MMU_RCR                 = $D506   // RAM-Konfiguration (gemeinsamer Bereich, VIC-Bank)
.const MMU_P0L                 = $D507   // Nullseiten-Zeiger niedrig
.const MMU_P0H                 = $D508   // Nullseiten-Zeiger hoch (Bank)
.const MMU_P1L                 = $D509   // Stapelseiten-Zeiger niedrig
.const MMU_P1H                 = $D50A   // Stapelseiten-Zeiger hoch (Bank)
.const MMU_VR                  = $D50B   // MMU-Version

// ---- MMU-Spiegel ($FF00, immer sichtbar)
.const MMU_CR_FF00             = $FF00   // Konfigurationsregister (immer sichtbar)
.const MMU_LCRA                = $FF01   // Laden der Konfiguration A (Schreiben genügt)
.const MMU_LCRB                = $FF02   // Laden der Konfiguration B
.const MMU_LCRC                = $FF03   // Laden der Konfiguration C
.const MMU_LCRD                = $FF04   // Laden der Konfiguration D

// ---- Standardkonfigurationen für MMU_CR_FF00
.const MMUCFG_ALL_ROM          = $00   // Bank 15: Systemroms und E/A (Standard)
.const MMUCFG_RAM0_IO          = $3E   // Bank 0 RAM mit E/A
.const MMUCFG_RAM0             = $3F   // Bank 0 RAM, kein E/A
.const MMUCFG_RAM1_IO          = $7E   // Bank 1 RAM mit E/A
.const MMUCFG_RAM1             = $7F   // Bank 1 RAM, kein E/A

// ---- VDC 8563/8568 (80 Spalten)
.const VDC_ADDR                = $D600   // Registerauswahl (schreiben) / Status (lesen)
.const VDC_DATA                = $D601   // Registerdaten

// ---- VDC-Registernummern (in VDC_ADDR schreiben)
.const VDCR_HTOTAL             = $00   // VDC-Register 0
.const VDCR_HDISPLAYED         = $01   // VDC-Register 1
.const VDCR_HSYNC_POS          = $02   // VDC-Register 2
.const VDCR_SYNC_WIDTH         = $03   // VDC-Register 3
.const VDCR_VTOTAL             = $04   // VDC-Register 4
.const VDCR_VTOTAL_ADJ         = $05   // VDC-Register 5
.const VDCR_VDISPLAYED         = $06   // VDC-Register 6
.const VDCR_VSYNC_POS          = $07   // VDC-Register 7
.const VDCR_INTERLACE          = $08   // VDC-Register 8
.const VDCR_CHAR_HEIGHT        = $09   // VDC-Register 9
.const VDCR_CURSOR_START       = $0A   // VDC-Register 10
.const VDCR_CURSOR_END         = $0B   // VDC-Register 11
.const VDCR_DISPLAY_HI         = $0C   // VDC-Register 12
.const VDCR_DISPLAY_LO         = $0D   // VDC-Register 13
.const VDCR_CURSOR_HI          = $0E   // VDC-Register 14
.const VDCR_CURSOR_LO          = $0F   // VDC-Register 15
.const VDCR_LIGHTPEN_V         = $10   // VDC-Register 16
.const VDCR_LIGHTPEN_H         = $11   // VDC-Register 17
.const VDCR_UPDATE_HI          = $12   // VDC-Register 18
.const VDCR_UPDATE_LO          = $13   // VDC-Register 19
.const VDCR_ATTR_HI            = $14   // VDC-Register 20
.const VDCR_ATTR_LO            = $15   // VDC-Register 21
.const VDCR_CHAR_WIDTH         = $16   // VDC-Register 22
.const VDCR_CHAR_DISPLAYED     = $17   // VDC-Register 23
.const VDCR_VSCROLL            = $18   // VDC-Register 24
.const VDCR_HSCROLL            = $19   // VDC-Register 25
.const VDCR_COLOR              = $1A   // VDC-Register 26
.const VDCR_ROW_INCREMENT      = $1B   // VDC-Register 27
.const VDCR_CHARSET_BASE       = $1C   // VDC-Register 28
.const VDCR_UNDERLINE          = $1D   // VDC-Register 29
.const VDCR_WORD_COUNT         = $1E   // VDC-Register 30
.const VDCR_DATA               = $1F   // VDC-Register 31
.const VDCR_COPY_SRC_HI        = $20   // VDC-Register 32
.const VDCR_COPY_SRC_LO        = $21   // VDC-Register 33
.const VDCR_DISPLAY_ENABLE_BEGIN= $22   // VDC-Register 34
.const VDCR_DISPLAY_ENABLE_END = $23   // VDC-Register 35
.const VDCR_REFRESH            = $24   // VDC-Register 36

// ---- VDC-Farben (RGBI)
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
