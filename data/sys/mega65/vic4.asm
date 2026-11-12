// MEGA65: VIC-III/IV, DMAgic (QDMA) und Hypervisor/System-Register (aus data/chips)
// Konstanten für den Assembler: #include <mega65/vic4.asm>

// ---- VIC-III/IV ($D02F ff.)
// Der VIC-IV ist der erweiterte Grafikchip des MEGA65 (zusätzlich zum VIC-II-Kern des C64). Damit sich der MEGA65 wie ein C64 verhält,
// sind seine erweiterten Register nach dem Einschalten gesperrt. Wer sie nutzt (VIC4_*), muss sie zuerst freischalten:
// $47, dann $53 in dieses Schlüsselregister $D02F schreiben, am einfachsten mit dem Makro mega65_enable_vic4 aus <mega65/gs4502.asm>.
// Ohne Freischaltung bleiben Schreibzugriffe auf die VIC4_*-Register wirkungslos; reine VIC-II-Register ($D000-$D02E) gehen immer.
.const VIC4_KEY                = $D02F   // Schlüsselregister: erst $47, dann $53 schreiben schaltet die erweiterten VIC-IV-Register frei (vorher wirkungslos); Makro mega65_enable_vic4
.const VIC4_BPENABLE           = $D032   // Bitplane enable bits
.const VIC4_BPCOMP             = $D03B   // Complement bitplane flags
.const VIC4_BPX                = $D03C   // Bitplane X
.const VIC4_BPY                = $D03D   // Bitplane Y
.const VIC4_HPOS               = $D03E   // Bitplane X Offset
.const VIC4_VPOS               = $D03F   // Bitplane Y Offset
.const VIC4_B0PIX              = $D040   // Display Address Translater (DAT) Bitplane N port
.const VIC4_TBDRPOS            = $D048   // top border position
.const VIC4_BBDRPOS            = $D04A   // bottom border position
.const VIC4_TEXTXPOS           = $D04C   // character generator horizontal position
.const VIC4_TEXTYPOS           = $D04E   // Character generator vertical position
.const VIC4_XPOSLSB            = $D050   // Read horizontal raster scan position LSB
.const VIC4_FNRASTERLSB        = $D052   // Read physical raster position
.const VIC4_SPRHGTEN           = $D055   // sprite extended height enable (one bit per sprite)
.const VIC4_SPRHGHT            = $D056   // Sprite extended height size (sprite pixels high)
.const VIC4_SPRX64EN           = $D057   // Sprite extended width enables (8 bytes per sprite row = 64…
.const VIC4_LINESTEPLSB        = $D058   // number of bytes to advance between each text row (LSB)
.const VIC4_LINESTEPMSB        = $D059   // number of bytes to advance between each text row (MSB)
.const VIC4_CHRXSCL            = $D05A   // Horizontal hardware scale of text mode (pixel 120ths per pi…
.const VIC4_CHRYSCL            = $D05B   // Vertical scaling of text mode (number of physical rasters p…
.const VIC4_CHRCOUNT           = $D05E   // Number of characters to display per row (LSB)
.const VIC4_SPRXSMSBS          = $D05F   // Sprite H640 X Super-MSBs
.const VIC4_SCRNPTRLSB         = $D060   // screen RAM precise base address (bits 7 - 0)
.const VIC4_SCRNPTRMSB         = $D061   // screen RAM precise base address (bits 15 - 8)
.const VIC4_SCRNPTRBNK         = $D062   // screen RAM precise base address (bits 23 - 16)
.const VIC4_COLPTRLSB          = $D064   // colour RAM base address (bits 0 - 7)
.const VIC4_COLPTRMSB          = $D065   // colour RAM base address (bits 15 - 8)
.const VIC4_CHARPTRLSB         = $D068   // Character set precise base address (bits 0 - 7)
.const VIC4_CHARPTRMSB         = $D069   // Character set precise base address (bits 15 - 8)
.const VIC4_CHARPTRBNK         = $D06A   // Character set precise base address (bits 23 - 16)
.const VIC4_SPR16EN            = $D06B   // sprite 16-colour mode enables
.const VIC4_SPRPTRADRLSB       = $D06C   // sprite pointer address (bits 7 - 0)
.const VIC4_SPRPTRADRMSB       = $D06D   // sprite pointer address (bits 15 - 8)
.const VIC4_BP16ENS            = $D071   // VIC-IV 16-colour bitplane enable flags
.const VIC4_SPRYADJ            = $D072   // Sprite Y position adjustment
.const VIC4_SPRENALPHA         = $D074   // Sprite alpha-blend enable
.const VIC4_SPRALPHAVAL        = $D075   // Sprite alpha-blend value
.const VIC4_SPRENV400          = $D076   // Sprite V400 enables
.const VIC4_SPRYMSBS           = $D077   // Sprite V400 Y position MSBs
.const VIC4_SPRYSMSBS          = $D078   // Sprite V400 Y position super MSBs
.const VIC4_RASCMP             = $D079   // Physical raster compare value to be used if FNRSTCMP is cle…

// ---- DMAgic ($D700 ff.)
.const DMA_ADDRLSBTRIG         = $D700   // DMAgic DMA list address LSB (bits 0 - 7), trigger DMA when…
.const DMA_ADDRMSB             = $D701   // DMA list address high byte (bits 8 - 15).
.const DMA_ADDRMB              = $D704   // DMA list address mega-byte (bits 20 - 27). Overlaps with AD…
.const DMA_ETRIG               = $D705   // Set low-order byte of DMA list address (bits 0 - 7), and tr…
.const DMA_ETRIGMAPD           = $D706   // Set low-order byte of DMA list address (bits 0 - 7), and tr…
.const DMA_ETRIGINLINE         = $D707   // Trigger Enhanced DMAgic job inline (i.e., beginning at PC v…
.const DMA_ADDRLSB             = $D70E   // DMA list address low byte (bits 0 - 7) WITHOUT STARTING A D…

// ---- Hypervisor und System ($D640 ff.)
.const M65_HTRAP00             = $D640   // Writing triggers hypervisor trap \$XX
.const M65_REGA                = $D640   // Hypervisor A register storage
.const M65_REGX                = $D641   // Hypervisor X register storage
.const M65_REGZ                = $D643   // Hypervisor Z register storage
.const M65_REGB                = $D644   // Hypervisor B register storage
.const M65_SPL                 = $D645   // Hypervisor SPL register storage
.const M65_SPH                 = $D646   // Hypervisor SPH register storage
.const M65_PFLAGS              = $D647   // Hypervisor P register storage
.const M65_PCL                 = $D648   // Hypervisor PC-low register storage
.const M65_PCH                 = $D649   // Hypervisor PC-high register storage
.const M65_MAPLO0              = $D64A   // Hypervisor MAPLO register storage (high bits)
.const M65_MAPLO1              = $D64B   // Hypervisor MAPLO register storage (low bits)
.const M65_MAPHI0              = $D64C   // Hypervisor MAPHI register storage (high bits)
.const M65_MAPHI1              = $D64D   // Hypervisor MAPHI register storage (low bits)
.const M65_MAPLOMB             = $D64E   // Hypervisor MAPLO mega-byte number register storage
.const M65_MAPHIMB             = $D64F   // Hypervisor MAPHI mega-byte number register storage
.const M65_PORT00              = $D650   // Hypervisor CPU port \$00 value
.const M65_PORT01              = $D651   // Hypervisor CPU port \$01 value
.const M65_DMASRCMB            = $D653   // Hypervisor DMAgic source MB
.const M65_DMADSTMB            = $D654   // Hypervisor DMAgic destination MB
.const M65_DMALADDR0           = $D655   // Hypervisor DMAGic list address bits 0-7
.const M65_DMALADDR1           = $D656   // Hypervisor DMAGic list address bits 15-8
.const M65_DMALADDR2           = $D657   // Hypervisor DMAGic list address bits 23-16
.const M65_DMALADDR3           = $D658   // Hypervisor DMAGic list address bits 27-24
.const M65_VIRTPAGE1           = $D65E   // Hypervisor current virtual page number (bits 2 - 9)
.const M65_VIRTPAGE2           = $D65F   // Hypervisor current virtual page number (bits 10 - 17)
.const M65_VPG0LOG0            = $D660   // Hypervisor virtual memory page X logical page (bits 0 - 7)
.const M65_VPG0LOG1            = $D661   // Hypervisor virtual memory page X logical page (bits 8 - 15)
.const M65_VPG0PHY0            = $D662   // Hypervisor virtual memory page X physical page (bits 0 - 7)
.const M65_VPG0PHY1            = $D663   // Hypervisor virtual memory page X physical page (bits 8 - 15)
.const M65_GEORAMBASE          = $D670   // Hypervisor GeoRAM base address (x MB)
.const M65_GEORAMMASK          = $D671   // Hypervisor GeoRAM address mask (applied to GeoRAM block reg…
.const M65_WATCHDOG            = $D67D   // Hypervisor watchdog register: writing any value clears the…
.const M65_HICKED              = $D67E   // Hypervisor already-upgraded bit (writing sets permanently)
.const M65_ENTEREXIT           = $D67F   // Writing trigger return from hypervisor
.const M65_CMDANDSTAT          = $D680   // SD controller status/command
.const M65_SECTOR0             = $D681   // SD controller SD sector address (LSB)
.const M65_SECTOR1             = $D682   // SD controller SD sector address (2nd byte)
.const M65_SECTOR2             = $D683   // SD controller SD sector address (3rd byte)
.const M65_SECTOR3             = $D684   // SD controller SD sector address (MSB)
.const M65_FILLVAL             = $D686   // WRITE ONLY set fill byte for use in fill mode, instead of S…
.const M65_D0STARTSEC0         = $D68C   // F011 drive 0 disk image address on SD card (LSB)
.const M65_DISKADDR0           = $D68C   // Diskimage sector number (bits 0-7)
.const M65_D0STARTSEC1         = $D68D   // F011 drive 0 disk image address on SD card (2nd byte)
.const M65_DISKADDR1           = $D68D   // Diskimage sector number (bits 8-15)
.const M65_D0STARTSEC2         = $D68E   // F011 drive 0 disk image address on SD card (3rd byte)
.const M65_DISKADDR2           = $D68E   // Diskimage sector number (bits 16-23)
.const M65_D0STARTSEC3         = $D68F   // F011 drive 0 disk image address on SD card (MSB)
.const M65_DISKADDR3           = $D68F   // Diskimage sector number (bits 24-31)
.const M65_D1STARTSEC0         = $D690   // F011 drive 1 disk image address on SD card (LSB)
.const M65_DISK2ADDR0          = $D690   // Diskimage 2 sector number (bits 0-7)
.const M65_D1STARTSEC1         = $D691   // F011 drive 1 disk image address on SD card (2nd byte)
.const M65_DISK2ADDR1          = $D691   // Diskimage 2 sector number (bits 8-15)
.const M65_D1STARTSEC2         = $D692   // F011 drive 1 disk image address on SD card (3rd byte)
.const M65_DISK2ADDR2          = $D692   // Diskimage 2 sector number (bits 16-23)
.const M65_D1STARTSEC3         = $D693   // F011 drive 1 disk image address on SD card (MSB)
.const M65_DISK2ADDR3          = $D693   // Diskimage 2 sector number (bits 24-31)
.const M65_AUTOTUNE            = $D696   // Enable automatic track seeking for sector reads and writes
