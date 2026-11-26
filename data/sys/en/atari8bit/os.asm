// Atari 8-bit: operating system (zero page, shadow registers, IOCB, vectors), PIA
// Constants for the assembler: #include <atari8bit/os.asm>

// ---- PIA ($D300)
.const PORTA                 = $D300   // Port A (joysticks 1/2)
.const PORTB                 = $D301   // Port B (XL/XE: memory configuration)
.const PACTL                 = $D302   // Port A control
.const PBCTL                 = $D303   // Port B control

// ---- Zero page
.const WARMST                = $0008   // Warm start flag
.const BOOT                  = $0009   // Boot status
.const DOSVEC                = $000A   // DOS start vector
.const DOSINI                = $000C   // DOS initialization vector
.const APPMHI                = $000E   // Upper limit of user memory
.const POKMSK                = $0010   // IRQEN shadow
.const BRKKEY                = $0011   // BREAK flag
.const RTCLOK                = $0012   // Real-time clock (3 bytes, 1/50 or 1/60 s)
.const SOUNDR                = $0041   // Sound during input/output
.const CRITIC                = $0042   // Critical section
.const ATRACT                = $004D   // Screen saver counter
.const DRKMSK                = $004E   // Darkening mask
.const COLRSH                = $004F   // Color shift
.const LMARGN                = $0052   // Left margin
.const RMARGN                = $0053   // Right margin
.const ROWCRS                = $0054   // Cursor row
.const COLCRS                = $0055   // Cursor column (2 bytes)
.const DINDEX                = $0057   // Current graphics mode
.const SAVMSC                = $0058   // Pointer to screen memory
.const RAMTOP                = $006A   // RAM top (pages)

// ---- Interrupt vectors and shadows
.const VDSLST                = $0200   // DLI vector
.const VPRCED                = $0202   // Serial proceed vector
.const VINTER                = $0204   // Serial interrupt vector
.const VBREAK                = $0206   // BRK vector
.const VKEYBD                = $0208   // Keyboard IRQ vector
.const VSERIN                = $020A   // Serial input ready
.const VSEROR                = $020C   // Serial output ready
.const VSEROC                = $020E   // Serial output complete
.const VTIMR1                = $0210   // POKEY timer 1
.const VTIMR2                = $0212   // POKEY timer 2
.const VTIMR4                = $0214   // POKEY timer 4
.const VIMIRQ                = $0216   // IRQ vector
.const CDTMV1                = $0218   // System timer 1
.const CDTMV2                = $021A   // System timer 2
.const CDTMV3                = $021C   // System timer 3
.const CDTMV4                = $021E   // System timer 4
.const CDTMV5                = $0220   // System timer 5
.const VVBLKI                = $0222   // Immediate VBI vector
.const VVBLKD                = $0224   // Deferred VBI vector
.const SDMCTL                = $022F   // DMACTL shadow
.const SDLSTL                = $0230   // Display list shadow (low)
.const SDLSTH                = $0231   // Display list shadow (high)
.const SSKCTL                = $0232   // SKCTL shadow
.const BRKKY                 = $0236   // BREAK vector
.const GPRIOR                = $026F   // PRIOR shadow
.const PADDL0                = $0270   // Paddles 0-7 (8 bytes)
.const STICK0                = $0278   // Joysticks 0-3 (4 bytes)
.const PTRIG0                = $027C   // Paddle buttons 0-7 (8 bytes)
.const STRIG0                = $0284   // Joystick buttons 0-3 (4 bytes)
.const PCOLR0                = $02C0   // COLPM0 shadow
.const PCOLR1                = $02C1   // COLPM1 shadow
.const PCOLR2                = $02C2   // COLPM2 shadow
.const PCOLR3                = $02C3   // COLPM3 shadow
.const COLOR0                = $02C4   // COLPF0 shadow
.const COLOR1                = $02C5   // COLPF1 shadow
.const COLOR2                = $02C6   // COLPF2 shadow
.const COLOR3                = $02C7   // COLPF3 shadow
.const COLOR4                = $02C8   // COLBK shadow
.const MEMTOP                = $02E5   // Upper limit of free memory
.const MEMLO                 = $02E7   // Lower limit of free memory
.const CHACT                 = $02F3   // CHACTL shadow
.const CHBAS                 = $02F4   // CHBASE shadow
.const CH                    = $02FC   // Last keyboard code ($FF = none)

// ---- IOCB (8 blocks of 16 bytes from $0340)
.const IOCB0                 = $0340   // Channel 0 (editor)
.const IOCB1                 = $0350   // Channel 1
.const IOCB2                 = $0360   // Channel 2
.const IOCB3                 = $0370   // Channel 3
.const IOCB4                 = $0380   // Channel 4
.const IOCB5                 = $0390   // Channel 5
.const IOCB6                 = $03A0   // Channel 6
.const IOCB7                 = $03B0   // Channel 7
.const ICHID                 = $0340   // Offset: device identifier
.const ICDNO                 = $0341   // Offset: device number
.const ICCOM                 = $0342   // Offset: command
.const ICSTA                 = $0343   // Offset: status
.const ICBAL                 = $0344   // Offset: buffer address low
.const ICBAH                 = $0345   // Offset: buffer address high
.const ICPTL                 = $0346   // Offset: PUT routine low
.const ICPTH                 = $0347   // Offset: PUT routine high
.const ICBLL                 = $0348   // Offset: buffer length low
.const ICBLH                 = $0349   // Offset: buffer length high
.const ICAX1                 = $034A   // Offset: auxiliary 1 (open mode)
.const ICAX2                 = $034B   // Offset: auxiliary 2
.const ICAX3                 = $034C   // Offset: auxiliary 3
.const ICAX4                 = $034D   // Offset: auxiliary 4

// ---- CIO commands and modes
.const CIO_OPEN              = $0003   // OPEN
.const CIO_GETREC            = $0005   // Read line
.const CIO_GETCHR            = $0007   // Read bytes
.const CIO_PUTREC            = $0009   // Write line
.const CIO_PUTCHR            = $000B   // Write bytes
.const CIO_CLOSE             = $000C   // CLOSE
.const CIO_STATUS            = $000D   // STATUS
.const CIO_SPECIAL           = $000E   // Device-specific
.const OPNIN                 = $0004   // ICAX1: read
.const OPNOT                 = $0008   // ICAX1: write
.const OPNINOT               = $000C   // ICAX1: read and write

// ---- Operating system entry points
.const EDITRV                = $E400   // Editor device table
.const SCRENV                = $E410   // Screen device table
.const KEYBDV                = $E420   // Keyboard device table
.const PRINTV                = $E430   // Printer device table
.const CASETV                = $E440   // Cassette device table
.const CIOV                  = $E456   // Central input/output
.const SIOV                  = $E459   // Serial input/output
.const SETVBV                = $E45C   // Set VBI vectors and timers
.const SYSVBV                = $E45F   // Exit jump for immediate VBI
.const XITVBV                = $E462   // Exit jump for deferred VBI
.const WARMSV                = $E474   // Warm start
.const COLDSV                = $E477   // Cold start

// ---- Floating-point package of the operating system (6-byte BCD: byte 0 = sign + exponent to base 100 (excess $40), bytes 1-5 = digit pairs)
.const FR0                   = $00D4   // Floating-point register 0 (6 bytes); integer input/output: FR0 low, FR0+1 high
.const FR1                   = $00E0   // Floating-point register 1 (6 bytes)
.const CIX                   = $00F2   // Index into the input buffer (AFP)
.const INBUFF                = $00F3   // Pointer to the text (AFP reads, FASC delivers)
.const FLPTR                 = $00FC   // Pointer for FLD0P/FLD1P/FST0P
.const LBUFF                 = $0580   // Line buffer of the package (FASC writes here)
.const AFP                   = $D800   // Text from (INBUFF)+CIX -> FR0; C set: not a valid number format
.const FASC                  = $D8E6   // FR0 -> text at (INBUFF); last character has bit 7 set
.const IFP                   = $D9AA   // Integer 0..65535 in FR0/FR0+1 -> FR0
.const FPI                   = $D9D2   // FR0 -> integer (rounded) in FR0/FR0+1; C set: outside 0..65535
.const ZFR0                  = $DA44   // FR0 = 0
.const ZF1                   = $DA46   // Set 6 bytes starting at (X) to zero
.const FSUB                  = $DA60   // FR0 = FR0 - FR1; C set: overflow
.const FADD                  = $DA66   // FR0 = FR0 + FR1
.const FMUL                  = $DADB   // FR0 = FR0 * FR1
.const FDIV                  = $DB28   // FR0 = FR0 / FR1; C set: division by 0 or overflow
.const FLD0R                 = $DD89   // FR0 = (X low/Y high)
.const FLD0P                 = $DD8D   // FR0 = (FLPTR)
.const FLD1R                 = $DD98   // FR1 = (X/Y)
.const FLD1P                 = $DD9C   // FR1 = (FLPTR)
.const FST0R                 = $DDA7   // (X/Y) = FR0
.const FST0P                 = $DDAB   // (FLPTR) = FR0
.const FMOVE                 = $DDB6   // FR1 = FR0
.const EXP                   = $DDC0   // FR0 = e^FR0; C set: overflow
.const EXP10                 = $DDCC   // FR0 = 10^FR0
.const LOG                   = $DECD   // FR0 = ln(FR0); C set: FR0 <= 0
.const LOG10                 = $DED1   // FR0 = log10(FR0)
.const PLYEVL                = $DD40   // Evaluate polynomial: A = number of coefficients, X/Y = table (6-byte BCD, highest power first), FR0 = argument
