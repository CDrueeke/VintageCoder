// Plus/4 and C16: KERNAL jump table (C64-compatible part), vectors, RAM vectors
// Constants for the assembler: #include <plus4/kernal.asm>

#include <plus4/hardware.asm>

// ---- KERNAL jump table
.const CINT                    = $FF81   // Initialize screen editor
.const IOINIT                  = $FF84   // Initialize I/O chips
.const RAMTAS                  = $FF87   // RAM test and set pointers
.const RESTOR                  = $FF8A   // Reset vectors to default
.const VECTOR                  = $FF8D   // Read/set vectors
.const SETMSG                  = $FF90   // Control system messages
.const SECOND                  = $FF93   // Secondary address after LISTEN
.const TKSA                    = $FF96   // Secondary address after TALK
.const MEMTOP                  = $FF99   // Read/set top of memory
.const MEMBOT                  = $FF9C   // Read/set bottom of memory
.const SCNKEY                  = $FF9F   // Scan keyboard
.const SETTMO                  = $FFA2   // IEEE timeout
.const ACPTR                   = $FFA5   // Read byte from IEC bus
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
