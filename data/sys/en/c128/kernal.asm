// C128: KERNAL jump table (C128 extensions and C64 part), vectors, bank helpers
// Constants for the assembler: #include <c128/kernal.asm>

#include <c128/hardware.asm>

// ---- C128 extensions of the jump table
.const SPIN_SPOUT              = $FF47   // Set up fast serial ports for input/output
.const CLOSE_ALL               = $FF4A   // Close all files of a device
.const C64MODE                 = $FF4D   // Switch to C64 mode
.const DMA_CALL                = $FF50   // Call DMA service
.const BOOT_CALL               = $FF53   // Load and start boot sector
.const PHOENIX                 = $FF56   // Start function modules
.const LKUPLA                  = $FF59   // Look up logical file number
.const LKUPSA                  = $FF5C   // Look up secondary address
.const SWAPPER                 = $FF5F   // Switch screen 40/80
.const DLCHR                   = $FF62   // Load character set into the VDC
.const PFKEY                   = $FF65   // Define function key
.const SETBNK                  = $FF68   // Set bank for LOAD/SAVE/OPEN
.const GETCFG                  = $FF6B   // Get configuration value for a bank
.const JSRFAR                  = $FF6E   // Call subroutine in another bank
.const JMPFAR                  = $FF71   // Jump to another bank
.const INDFET                  = $FF74   // Fetch byte from another bank (LDA)
.const INDSTA                  = $FF77   // Write byte to another bank (STA)
.const INDCMP                  = $FF7A   // Compare byte from another bank (CMP)
.const PRIMM                   = $FF7D   // Print text following the JSR

// ---- C64-compatible part of the jump table
.const CINT                    = $FF81   // Initialize screen editor
.const IOINIT                  = $FF84   // Initialize I/O chips
.const RAMTAS                  = $FF87   // RAM test and set pointers
.const RESTOR                  = $FF8A   // Vectors to default
.const VECTOR                  = $FF8D   // Read/set vectors
.const SETMSG                  = $FF90   // Control system messages
.const SECOND                  = $FF93   // Secondary address after LISTEN
.const TKSA                    = $FF96   // Secondary address after TALK
.const MEMTOP                  = $FF99   // Read/set top of memory
.const MEMBOT                  = $FF9C   // Read/set bottom of memory
.const SCNKEY                  = $FF9F   // Scan keyboard
.const SETTMO                  = $FFA2   // IEEE timeout
.const ACPTR                   = $FFA5   // Get byte from IEC bus
.const CIOUT                   = $FFA8   // Send byte to IEC bus
.const UNTLK                   = $FFAB   // Send UNTALK
.const UNLSN                   = $FFAE   // Send UNLISTEN
.const LISTEN                  = $FFB1   // Command device to listen
.const TALK                    = $FFB4   // Command device to talk
.const READST                  = $FFB7   // Read status word
.const SETLFS                  = $FFBA   // Set logical file, device, secondary address
.const SETNAM                  = $FFBD   // Set file name
.const OPEN                    = $FFC0   // Open file
.const CLOSE                   = $FFC3   // Close file
.const CHKIN                   = $FFC6   // Set input channel
.const CHKOUT                  = $FFC9   // Set output channel
.const CLRCHN                  = $FFCC   // Reset channels
.const CHRIN                   = $FFCF   // Get character
.const CHROUT                  = $FFD2   // Output character
.const LOAD                    = $FFD5   // Load file
.const SAVE                    = $FFD8   // Save memory
.const SETTIM                  = $FFDB   // Set system clock
.const RDTIM                   = $FFDE   // Read system clock
.const STOP                    = $FFE1   // Check STOP key
.const GETIN                   = $FFE4   // Get key from buffer
.const CLALL                   = $FFE7   // Close all files
.const UDTIM                   = $FFEA   // Advance system clock
.const SCREEN                  = $FFED   // Get screen size
.const PLOT                    = $FFF0   // Read/set cursor position
.const IOBASE                  = $FFF3   // Address of the I/O chips (CIA 1)

// ---- Hardware vectors
.const VEC_NMI                 = $FFFA   // NMI vector
.const VEC_RESET               = $FFFC   // RESET vector
.const VEC_IRQ                 = $FFFE   // IRQ/BRK vector

// ---- System variables
.const STATUS                  = $0090   // Status word (ST)
.const FA                      = $00BA   // Current device number
.const MODE                    = $00D7   // Screen mode (bit 7: 80 columns)

// ---- RAM vectors
.const CINV                    = $0314   // IRQ vector
.const CBINV                   = $0316   // BRK vector
.const NMINV                   = $0318   // NMI vector
.const IOPEN                   = $031A   // OPEN
.const ICLOSE                  = $031C   // CLOSE
.const ICHKIN                  = $031E   // CHKIN
.const ICKOUT                  = $0320   // CHKOUT
.const ICLRCH                  = $0322   // CLRCHN
.const IBASIN                  = $0324   // CHRIN
.const IBSOUT                  = $0326   // CHROUT
.const ISTOP                   = $0328   // STOP
.const IGETIN                  = $032A   // GETIN
.const ICLALL                  = $032C   // CLALL
.const ILOAD                   = $0330   // LOAD
.const ISAVE                   = $0332   // SAVE

// ---- Routines for access to other banks (in common RAM)
.const FETCH                   = $02A2   // Fetch byte
.const FETVEC                  = $02AA   // Address pointer for FETCH
.const STASH                   = $02AF   // Write byte
.const STAVEC                  = $02B9   // Address pointer for STASH
.const CMPARE                  = $02BE   // Compare byte
.const CMPVEC                  = $02C8   // Address pointer for CMPARE

// ---- Macros
// Set the configuration directly via the always-visible register $FF00 (e.g. mmu_config MMUCFG_RAM0_IO)
.macro mmu_config(cfg)
 lda #cfg
 sta MMU_CR_FF00
.endmacro

// Fetch a byte from another bank: ptr = zero-page address of the 16-bit pointer, bank = bank value; index in Y, result in A
.macro fetch_far(ptr, bank)
 lda #ptr
 ldx #bank
 jsr INDFET
.endmacro
