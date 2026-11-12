// C64: Hardware-Register (VIC-II, SID, CIA 1/2), Speicherbereiche
// Konstanten für den Assembler: #include <c64/hardware.asm>

// ---- Prozessorport
.const CPU_DDR               = $0000   // Datenrichtung des Prozessorports
.const CPU_PORT              = $0001   // Speicherbelegung (LORAM/HIRAM/CHAREN) und Kassette

// ---- Speicherbereiche
.const SCREEN_RAM            = $0400   // Standard-Bildschirmspeicher
.const SPRITE_PTRS           = $07F8   // Sprite-Zeiger (bei Standard-Bildschirm)
.const COLOR_RAM             = $D800   // Farb-RAM (nur Bit 0-3)

// ---- VIC-II
.const VIC_SPR0_X            = $D000   // Sprite 0: X (Bit 0-7)
.const VIC_SPR0_Y            = $D001   // Sprite 0: Y
.const VIC_SPR1_X            = $D002   // Sprite 1: X (Bit 0-7)
.const VIC_SPR1_Y            = $D003   // Sprite 1: Y
.const VIC_SPR2_X            = $D004   // Sprite 2: X (Bit 0-7)
.const VIC_SPR2_Y            = $D005   // Sprite 2: Y
.const VIC_SPR3_X            = $D006   // Sprite 3: X (Bit 0-7)
.const VIC_SPR3_Y            = $D007   // Sprite 3: Y
.const VIC_SPR4_X            = $D008   // Sprite 4: X (Bit 0-7)
.const VIC_SPR4_Y            = $D009   // Sprite 4: Y
.const VIC_SPR5_X            = $D00A   // Sprite 5: X (Bit 0-7)
.const VIC_SPR5_Y            = $D00B   // Sprite 5: Y
.const VIC_SPR6_X            = $D00C   // Sprite 6: X (Bit 0-7)
.const VIC_SPR6_Y            = $D00D   // Sprite 6: Y
.const VIC_SPR7_X            = $D00E   // Sprite 7: X (Bit 0-7)
.const VIC_SPR7_Y            = $D00F   // Sprite 7: Y
.const VIC_SPR_X_MSB         = $D010   // X-Bit 8 aller Sprites
.const VIC_CTRL1             = $D011   // Steuerregister 1 (Y-Scroll, 24/25 Zeilen, Bitmap, ECM, Rasterzeile Bit 8)
.const VIC_RASTER            = $D012   // Rasterzeile (lesen) / Raster-IRQ-Zeile (schreiben)
.const VIC_LIGHTPEN_X        = $D013   // Lichtgriffel X
.const VIC_LIGHTPEN_Y        = $D014   // Lichtgriffel Y
.const VIC_SPR_ENABLE        = $D015   // Sprites einschalten
.const VIC_CTRL2             = $D016   // Steuerregister 2 (X-Scroll, 38/40 Spalten, Multicolor)
.const VIC_SPR_EXPAND_Y      = $D017   // Sprites doppelt hoch
.const VIC_MEMORY            = $D018   // Zeichensatz- und Bildschirmspeicher-Zeiger
.const VIC_IRQ_FLAGS         = $D019   // IRQ-Quellen (Schreiben von 1 löscht)
.const VIC_IRQ_ENABLE        = $D01A   // IRQ-Quellen freigeben
.const VIC_SPR_PRIORITY      = $D01B   // Sprites hinter dem Hintergrund
.const VIC_SPR_MULTICOLOR    = $D01C   // Sprites mehrfarbig
.const VIC_SPR_EXPAND_X      = $D01D   // Sprites doppelt breit
.const VIC_SPR_COLL_SPR      = $D01E   // Kollision Sprite-Sprite (Lesen löscht)
.const VIC_SPR_COLL_BG       = $D01F   // Kollision Sprite-Hintergrund (Lesen löscht)
.const VIC_BORDERCOLOR       = $D020   // Rahmenfarbe
.const VIC_BGCOLOR0          = $D021   // Hintergrundfarbe 0
.const VIC_BGCOLOR1          = $D022   // Hintergrundfarbe 1 (Multicolor/ECM)
.const VIC_BGCOLOR2          = $D023   // Hintergrundfarbe 2
.const VIC_BGCOLOR3          = $D024   // Hintergrundfarbe 3
.const VIC_SPR_MCOLOR0       = $D025   // Sprite-Multicolor 0
.const VIC_SPR_MCOLOR1       = $D026   // Sprite-Multicolor 1
.const VIC_SPR0_COLOR        = $D027   // Farbe Sprite 0
.const VIC_SPR1_COLOR        = $D028   // Farbe Sprite 1
.const VIC_SPR2_COLOR        = $D029   // Farbe Sprite 2
.const VIC_SPR3_COLOR        = $D02A   // Farbe Sprite 3
.const VIC_SPR4_COLOR        = $D02B   // Farbe Sprite 4
.const VIC_SPR5_COLOR        = $D02C   // Farbe Sprite 5
.const VIC_SPR6_COLOR        = $D02D   // Farbe Sprite 6
.const VIC_SPR7_COLOR        = $D02E   // Farbe Sprite 7

// ---- SID
.const SID_V1_FREQ_LO        = $D400   // Stimme 1: Frequenz niedrig
.const SID_V1_FREQ_HI        = $D401   // Stimme 1: Frequenz hoch
.const SID_V1_PW_LO          = $D402   // Stimme 1: Pulsbreite niedrig
.const SID_V1_PW_HI          = $D403   // Stimme 1: Pulsbreite hoch (4 Bit)
.const SID_V1_CTRL           = $D404   // Stimme 1: Wellenform und Gate
.const SID_V1_AD             = $D405   // Stimme 1: Attack/Decay
.const SID_V1_SR             = $D406   // Stimme 1: Sustain/Release
.const SID_V2_FREQ_LO        = $D407   // Stimme 2: Frequenz niedrig
.const SID_V2_FREQ_HI        = $D408   // Stimme 2: Frequenz hoch
.const SID_V2_PW_LO          = $D409   // Stimme 2: Pulsbreite niedrig
.const SID_V2_PW_HI          = $D40A   // Stimme 2: Pulsbreite hoch (4 Bit)
.const SID_V2_CTRL           = $D40B   // Stimme 2: Wellenform und Gate
.const SID_V2_AD             = $D40C   // Stimme 2: Attack/Decay
.const SID_V2_SR             = $D40D   // Stimme 2: Sustain/Release
.const SID_V3_FREQ_LO        = $D40E   // Stimme 3: Frequenz niedrig
.const SID_V3_FREQ_HI        = $D40F   // Stimme 3: Frequenz hoch
.const SID_V3_PW_LO          = $D410   // Stimme 3: Pulsbreite niedrig
.const SID_V3_PW_HI          = $D411   // Stimme 3: Pulsbreite hoch (4 Bit)
.const SID_V3_CTRL           = $D412   // Stimme 3: Wellenform und Gate
.const SID_V3_AD             = $D413   // Stimme 3: Attack/Decay
.const SID_V3_SR             = $D414   // Stimme 3: Sustain/Release
.const SID_FILTER_LO         = $D415   // Filter-Grenzfrequenz (Bit 0-2)
.const SID_FILTER_HI         = $D416   // Filter-Grenzfrequenz (Bit 3-10)
.const SID_RES_FILT          = $D417   // Resonanz und Filterzuordnung
.const SID_MODE_VOL          = $D418   // Filtermodus und Lautstärke
.const SID_POTX              = $D419   // Paddle X (lesen)
.const SID_POTY              = $D41A   // Paddle Y (lesen)
.const SID_OSC3              = $D41B   // Oszillator 3 (lesen)
.const SID_ENV3              = $D41C   // Hüllkurve 3 (lesen)

// ---- SID: Bits im Steuerregister
.const SID_GATE              = $0001   // Ctrl: Gate
.const SID_SYNC              = $0002   // Ctrl: Synchronisation
.const SID_RING              = $0004   // Ctrl: Ringmodulation
.const SID_TEST              = $0008   // Ctrl: Test
.const SID_TRIANGLE          = $0010   // Ctrl: Dreieck
.const SID_SAW               = $0020   // Ctrl: Sägezahn
.const SID_PULSE             = $0040   // Ctrl: Rechteck
.const SID_NOISE             = $0080   // Ctrl: Rauschen

// ---- CIA 1 (Tastatur, Joystick, IRQ)
.const CIA1_PRA              = $DC00   // Port A
.const CIA1_PRB              = $DC01   // Port B
.const CIA1_DDRA             = $DC02   // Datenrichtung A
.const CIA1_DDRB             = $DC03   // Datenrichtung B
.const CIA1_TA_LO            = $DC04   // Timer A niedrig
.const CIA1_TA_HI            = $DC05   // Timer A hoch
.const CIA1_TB_LO            = $DC06   // Timer B niedrig
.const CIA1_TB_HI            = $DC07   // Timer B hoch
.const CIA1_TOD_10THS        = $DC08   // Uhr: Zehntelsekunden
.const CIA1_TOD_SEC          = $DC09   // Uhr: Sekunden
.const CIA1_TOD_MIN          = $DC0A   // Uhr: Minuten
.const CIA1_TOD_HRS          = $DC0B   // Uhr: Stunden
.const CIA1_SDR              = $DC0C   // Serielles Schieberegister
.const CIA1_ICR              = $DC0D   // Interrupt-Steuerung/-Status
.const CIA1_CRA              = $DC0E   // Steuerregister A
.const CIA1_CRB              = $DC0F   // Steuerregister B

// ---- CIA 2 (IEC, Userport, VIC-Bank, NMI)
.const CIA2_PRA              = $DD00   // Port A
.const CIA2_PRB              = $DD01   // Port B
.const CIA2_DDRA             = $DD02   // Datenrichtung A
.const CIA2_DDRB             = $DD03   // Datenrichtung B
.const CIA2_TA_LO            = $DD04   // Timer A niedrig
.const CIA2_TA_HI            = $DD05   // Timer A hoch
.const CIA2_TB_LO            = $DD06   // Timer B niedrig
.const CIA2_TB_HI            = $DD07   // Timer B hoch
.const CIA2_TOD_10THS        = $DD08   // Uhr: Zehntelsekunden
.const CIA2_TOD_SEC          = $DD09   // Uhr: Sekunden
.const CIA2_TOD_MIN          = $DD0A   // Uhr: Minuten
.const CIA2_TOD_HRS          = $DD0B   // Uhr: Stunden
.const CIA2_SDR              = $DD0C   // Serielles Schieberegister
.const CIA2_ICR              = $DD0D   // Interrupt-Steuerung/-Status
.const CIA2_CRA              = $DD0E   // Steuerregister A
.const CIA2_CRB              = $DD0F   // Steuerregister B

// ---- Farben
.const COL_BLACK             = $0000
.const COL_WHITE             = $0001
.const COL_RED               = $0002
.const COL_CYAN              = $0003
.const COL_PURPLE            = $0004
.const COL_GREEN             = $0005
.const COL_BLUE              = $0006
.const COL_YELLOW            = $0007
.const COL_ORANGE            = $0008
.const COL_BROWN             = $0009
.const COL_LIGHTRED          = $000A
.const COL_DARKGRAY          = $000B
.const COL_MIDGRAY           = $000C
.const COL_LIGHTGREEN        = $000D
.const COL_LIGHTBLUE         = $000E
.const COL_LIGHTGRAY         = $000F
