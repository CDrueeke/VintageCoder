// VC-20 (VIC-20): VIC-I (6560/6561), zwei VIA 6522, Speicherbereiche
// Konstanten für den Assembler: #include <vic20/hardware.asm>
//
// Bildschirm-, Farb-RAM und BASIC-Anfang hängen vom Speicherausbau ab (Projekteinstellung `memory`): Beim Bauen definiert VintageCoder
// MEM_SCREEN_RAM, MEM_COLOR_RAM und MEM_BASIC_START; ohne Angabe gilt das Grundgerät.

// ---- Speicherbereiche
.ifdef MEM_SCREEN_RAM
.const SCREEN_RAM              = MEM_SCREEN_RAM   // Bildschirmspeicher (Grundgerät und +3 KB: $1E00, ab +8 KB: $1000)
.else
.const SCREEN_RAM              = $1E00   // Bildschirmspeicher (Grundgerät und +3 KB: $1E00, ab +8 KB: $1000)
.endif
.ifdef MEM_COLOR_RAM
.const COLOR_RAM               = MEM_COLOR_RAM   // Farb-RAM (nur Bit 0-3; Grundgerät und +3 KB: $9600, ab +8 KB: $9400)
.else
.const COLOR_RAM               = $9600   // Farb-RAM (nur Bit 0-3; Grundgerät und +3 KB: $9600, ab +8 KB: $9400)
.endif
.ifdef MEM_BASIC_START
.const BASIC_START             = MEM_BASIC_START   // BASIC-Programmanfang (Grundgerät $1001, +3 KB $0401, ab +8 KB $1201)
.else
.const BASIC_START             = $1001   // BASIC-Programmanfang (Grundgerät $1001, +3 KB $0401, ab +8 KB $1201)
.endif
.const CHAR_ROM                = $8000   // Zeichen-ROM (Groß/Grafik, dann Klein/Groß)
.const RAM_BLOCK1              = $2000   // Erweiterungsblock 1 (8 KB)
.const RAM_BLOCK2              = $4000   // Erweiterungsblock 2 (8 KB)
.const RAM_BLOCK3              = $6000   // Erweiterungsblock 3 (8 KB)
.const CART_BLOCK5             = $A000   // Steckmodul (Autostart-Kennung A0CBM bei $A004)

// ---- VIC-I
.const VIC_ORIGIN_X               = $9000   // Horizontaler Bildursprung (Bit 0-6), Bit 7 = Interlace
.const VIC_ORIGIN_Y               = $9001   // Vertikaler Bildursprung
.const VIC_COLS                   = $9002   // Spaltenzahl (Bit 0-6), Bit 7 = Bildschirmadresse Bit 9
.const VIC_ROWS                   = $9003   // Zeilenzahl (Bit 1-6), Bit 0 = 8×16-Zeichen, Bit 7 = Rasterzeile Bit 0
.const VIC_RASTER                 = $9004   // Rasterzeile (Bit 8-1, nur lesen)
.const VIC_MEMPTR                 = $9005   // Speicherzeiger: Bit 4-7 Bildschirm, Bit 0-3 Zeichensatz
.const VIC_LPX                    = $9006   // Lichtgriffel X
.const VIC_LPY                    = $9007   // Lichtgriffel Y
.const VIC_POTX                   = $9008   // Paddle X
.const VIC_POTY                   = $9009   // Paddle Y
.const VIC_BASS                   = $900A   // Bassstimme (Bit 7 = ein)
.const VIC_ALTO                   = $900B   // Altstimme (Bit 7 = ein)
.const VIC_SOPRANO                = $900C   // Sopranstimme (Bit 7 = ein)
.const VIC_NOISE                  = $900D   // Rauschen (Bit 7 = ein)
.const VIC_VOLUME                 = $900E   // Lautstärke (Bit 0-3), Hilfsfarbe (Bit 4-7)
.const VIC_COLOR                  = $900F   // Hintergrund (Bit 4-7), Normal/Invers (Bit 3), Rahmen (Bit 0-2)

// ---- VIA 1 (NMI, Userport, Joystick, serieller Bus)
.const VIA1_ORB                   = $9110   // Port B
.const VIA1_ORA                   = $9111   // Port A
.const VIA1_DDRB                  = $9112   // Datenrichtung B
.const VIA1_DDRA                  = $9113   // Datenrichtung A
.const VIA1_T1C_LO                = $9114   // Timer 1 Zähler niedrig
.const VIA1_T1C_HI                = $9115   // Timer 1 Zähler hoch
.const VIA1_T1L_LO                = $9116   // Timer 1 Latch niedrig
.const VIA1_T1L_HI                = $9117   // Timer 1 Latch hoch
.const VIA1_T2C_LO                = $9118   // Timer 2 Zähler niedrig
.const VIA1_T2C_HI                = $9119   // Timer 2 Zähler hoch
.const VIA1_SR                    = $911A   // Schieberegister
.const VIA1_ACR                   = $911B   // Hilfssteuerregister
.const VIA1_PCR                   = $911C   // Peripherie-Steuerregister
.const VIA1_IFR                   = $911D   // Interrupt-Flags
.const VIA1_IER                   = $911E   // Interrupt-Freigabe
.const VIA1_ORA_NH                = $911F   // Port A ohne Handshake

// ---- VIA 2 (IRQ-Timer, Tastatur)
.const VIA2_ORB                   = $9120   // Port B
.const VIA2_ORA                   = $9121   // Port A
.const VIA2_DDRB                  = $9122   // Datenrichtung B
.const VIA2_DDRA                  = $9123   // Datenrichtung A
.const VIA2_T1C_LO                = $9124   // Timer 1 Zähler niedrig
.const VIA2_T1C_HI                = $9125   // Timer 1 Zähler hoch
.const VIA2_T1L_LO                = $9126   // Timer 1 Latch niedrig
.const VIA2_T1L_HI                = $9127   // Timer 1 Latch hoch
.const VIA2_T2C_LO                = $9128   // Timer 2 Zähler niedrig
.const VIA2_T2C_HI                = $9129   // Timer 2 Zähler hoch
.const VIA2_SR                    = $912A   // Schieberegister
.const VIA2_ACR                   = $912B   // Hilfssteuerregister
.const VIA2_PCR                   = $912C   // Peripherie-Steuerregister
.const VIA2_IFR                   = $912D   // Interrupt-Flags
.const VIA2_IER                   = $912E   // Interrupt-Freigabe
.const VIA2_ORA_NH                = $912F   // Port A ohne Handshake

// ---- Farben (Zeichenfarbe 0-7; Hintergrund und Hilfsfarbe 0-15)
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
