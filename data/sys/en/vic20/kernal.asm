// VIC-20: KERNAL jump table (like C64, but without CINT/IOINIT/RAMTAS), hardware vectors, system variables and RAM vectors
// Constants for the assembler: #include <vic20/kernal.asm>

#include <vic20/hardware.asm>

// ---- Official KERNAL jump table (on the VIC-20 only from RESTOR; CINT, IOINIT and RAMTAS are missing there, see KERNAL ROM routines)
.const RESTOR                = $FF8A   // Reset vectors to default
.const VECTOR                = $FF8D   // Read/set vectors
.const SETMSG                = $FF90   // Control system messages
.const SECOND                = $FF93   // Secondary address after LISTEN
.const TKSA                  = $FF96   // Secondary address after TALK
.const MEMTOP                = $FF99   // Read/set top of memory
.const MEMBOT                = $FF9C   // Read/set bottom of memory
.const SCNKEY                = $FF9F   // Scan keyboard
.const SETTMO                = $FFA2   // IEEE timeout
.const ACPTR                 = $FFA5   // Read byte from IEC bus
.const CIOUT                 = $FFA8   // Send byte to IEC bus
.const UNTLK                 = $FFAB   // Send UNTALK
.const UNLSN                 = $FFAE   // Send UNLISTEN
.const LISTEN                = $FFB1   // Command device to listen
.const TALK                  = $FFB4   // Command device to talk
.const READST                = $FFB7   // Read status word
.const SETLFS                = $FFBA   // Set logical file, device, secondary address
.const SETNAM                = $FFBD   // Set file name
.const OPEN                  = $FFC0   // Open file
.const CLOSE                 = $FFC3   // Close file
.const CHKIN                 = $FFC6   // Set input channel
.const CHKOUT                = $FFC9   // Set output channel
.const CLRCHN                = $FFCC   // Reset channels
.const CHRIN                 = $FFCF   // Get character
.const CHROUT                = $FFD2   // Output character
.const LOAD                  = $FFD5   // Load file
.const SAVE                  = $FFD8   // Save memory
.const SETTIM                = $FFDB   // Set system clock
.const RDTIM                 = $FFDE   // Read system clock
.const STOP                  = $FFE1   // Check STOP key
.const GETIN                 = $FFE4   // Get key from buffer
.const CLALL                 = $FFE7   // Close all files
.const UDTIM                 = $FFEA   // Advance system clock
.const SCREEN                = $FFED   // Get screen size
.const PLOT                  = $FFF0   // Read/set cursor position
.const IOBASE                = $FFF3   // Address of the I/O chips (VIA 1)

// ---- Hardware vectors
.const VEC_NMI               = $FFFA   // NMI vector
.const VEC_RESET             = $FFFC   // RESET vector
.const VEC_IRQ               = $FFFE   // IRQ/BRK vector

// ---- KERNAL zero page
.const STATUS                = $0090   // Status word (ST)
.const STKEY                 = $0091   // STOP key (flag)
.const LDTND                 = $0098   // Number of open files
.const DFLTN                 = $0099   // Input device
.const DFLTO                 = $009A   // Output device
.const TIME                  = $00A0   // System clock (3 bytes, 1/60 s)
.const FNLEN                 = $00B7   // Length of file name
.const LA                    = $00B8   // Current logical file number
.const SA                    = $00B9   // Current secondary address
.const FA                    = $00BA   // Current device number
.const FNADR                 = $00BB   // Pointer to file name
.const LSTX                  = $00C5   // Last key pressed
.const NDX                   = $00C6   // Number of characters in keyboard buffer
.const RVS                   = $00C7   // Reverse flag
.const SFDX                  = $00CB   // Key currently pressed
.const BLNSW                 = $00CC   // Cursor blink off (0 = blinking)
.const PNT                   = $00D1   // Pointer to current screen line
.const PNTR                  = $00D3   // Cursor column
.const QTSW                  = $00D4   // Quote mode
.const TBLX                  = $00D6   // Cursor row
.const USER                  = $00F3   // Pointer to current color RAM line

// ---- KERNAL RAM
.const BUF                   = $0200   // BASIC input buffer (89 bytes)
.const KEYD                  = $0277   // Keyboard buffer (10 bytes)
.const MEMSTR                = $0281   // Start of RAM for the operating system
.const MEMSIZ                = $0283   // End of RAM for the operating system
.const COLOR                 = $0286   // Current character color
.const GDCOL                 = $0287   // Color under the cursor
.const HIBASE                = $0288   // Screen memory (high byte)
.const XMAX                  = $0289   // Size of keyboard buffer
.const RPTFLAG               = $028A   // Key repeat
.const SHFLAG                = $028D   // SHIFT/CTRL/C= flags
.const MODE                  = $0291   // Character set switching allowed
.const TBUFFR                = $033C   // Cassette buffer (192 bytes)

// ---- RAM vectors (redirectable)
.const CINV                  = $0314   // IRQ vector
.const CBINV                 = $0316   // BRK vector
.const NMINV                 = $0318   // NMI vector
.const IOPEN                 = $031A   // OPEN
.const ICLOSE                = $031C   // CLOSE
.const ICHKIN                = $031E   // CHKIN
.const ICKOUT                = $0320   // CHKOUT
.const ICLRCH                = $0322   // CLRCHN
.const IBASIN                = $0324   // CHRIN
.const IBSOUT                = $0326   // CHROUT
.const ISTOP                 = $0328   // STOP
.const IGETIN                = $032A   // GETIN
.const ICLALL                = $032C   // CLALL
.const ILOAD                 = $0330   // LOAD
.const ISAVE                 = $0332   // SAVE

// ---- KERNAL ROM routines (addresses of KERNAL versions 901486-06 and -07)
.const RAMTAS                = $FD8D   // Test RAM, set pointers and zero page (not in the jump table)
.const IOINIT                = $FDF9   // Initialize VIC-I and VIA (not in the jump table)
.const CINT                  = $E518   // Initialize screen editor (not in the jump table)
.const CLRSCR                = $E55F   // Clear screen
.const HOME                  = $E581   // Cursor to top left
.const IRQ_STD               = $EABF   // Default IRQ routine
.const IRQ_ACK               = $EB15   // End IRQ (acknowledge VIA 2 timer, restore registers, RTI)
.const IRQ_EXIT              = $EB18   // End IRQ (restore registers, RTI)
