// MEGA65: KERNAL jump table (at the same addresses in all ROM versions)
// Constants for the assembler: #include <mega65/kernal.asm>

// The jump table is the same in all MEGA65 ROMs (closed 9xxxxx and OpenROMs); ROM-dependent addresses: rom_920418.asm, rom_open.asm.

// ---- Extensions (C65/C128 style)
.const SPIN_SPOUT              = $FF4D   // Fast serial ports
.const CLOSE_ALL               = $FF50   // Close all files of a device
.const C64MODE                 = $FF53   // Switch to C64 mode
.const MONITOR_CALL            = $FF56   // Call the monitor
.const BOOTSYS                 = $FF59   // Load and start boot sector
.const PHOENIX                 = $FF5C   // Start function modules
.const LKUPLA                  = $FF5F   // Look up logical file number
.const LKUPSA                  = $FF62   // Look up secondary address
.const SWAPPER                 = $FF65   // Switch screen width
.const PFKEY                   = $FF68   // Define function key
.const SETBNK                  = $FF6B   // Set bank for LOAD/SAVE
.const JSR_FAR                 = $FF6E   // Call subroutine in another bank
.const JMP_FAR                 = $FF71   // Jump to another bank
.const LDA_FAR                 = $FF74   // Fetch byte from another bank
.const STA_FAR                 = $FF77   // Write byte to another bank
.const CMP_FAR                 = $FF7A   // Compare byte from another bank
.const PRIMM                   = $FF7D   // Print text following the JSR

// ---- C64-compatible part
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
