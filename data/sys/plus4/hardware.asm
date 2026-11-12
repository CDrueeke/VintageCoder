// Plus/4 und C16: TED, ACIA 6551, 6529-Ports, ROM-Bankwahl
// Konstanten für den Assembler: #include <plus4/hardware.asm>

// ---- Prozessorport
.const CPU_DDR                 = $0000   // Datenrichtung des Prozessorports
.const CPU_PORT                = $0001   // Kassette, serieller Bus

// ---- Speicherbereiche
.const COLOR_RAM               = $0800   // Farbspeicher
.const SCREEN_RAM              = $0C00   // Standard-Bildschirmspeicher
.const BASIC_START             = $1001   // BASIC-Programmanfang

// ---- TED
.const TED_TIMER1_LO           = $FF00   // Timer 1 niedrig
.const TED_TIMER1_HI           = $FF01   // Timer 1 hoch
.const TED_TIMER2_LO           = $FF02   // Timer 2 niedrig
.const TED_TIMER2_HI           = $FF03   // Timer 2 hoch
.const TED_TIMER3_LO           = $FF04   // Timer 3 niedrig
.const TED_TIMER3_HI           = $FF05   // Timer 3 hoch
.const TED_CTRL1               = $FF06   // Steuerregister 1 (Y-Scroll, 24/25 Zeilen, Bitmap, ECM)
.const TED_CTRL2               = $FF07   // Steuerregister 2 (X-Scroll, 38/40 Spalten, Multicolor, PAL/NTSC)
.const TED_KEYLATCH            = $FF08   // Tastatur-/Joystick-Abfrage
.const TED_IRQ_FLAGS           = $FF09   // IRQ-Quellen (Schreiben von 1 löscht)
.const TED_IRQ_ENABLE          = $FF0A   // IRQ-Quellen freigeben, Rasterzeile Bit 8
.const TED_RASTER              = $FF0B   // Raster-IRQ-Zeile (Bit 0-7)
.const TED_CURSOR_HI           = $FF0C   // Cursorposition hoch
.const TED_CURSOR_LO           = $FF0D   // Cursorposition niedrig
.const TED_VOICE1_LO           = $FF0E   // Stimme 1: Frequenz niedrig
.const TED_VOICE2_LO           = $FF0F   // Stimme 2: Frequenz niedrig
.const TED_VOICE2_HI           = $FF10   // Stimme 2: Frequenz hoch
.const TED_SOUND               = $FF11   // Lautstärke und Stimmenauswahl
.const TED_BITMAP_BASE         = $FF12   // Bitmap-Adresse, Stimme 1 hoch, Zeichensatz RAM/ROM
.const TED_CHARSET_BASE        = $FF13   // Zeichensatzadresse, Schnellmodus
.const TED_SCREEN_BASE         = $FF14   // Bildschirmspeicheradresse
.const TED_BGCOLOR             = $FF15   // Hintergrundfarbe 0
.const TED_COLOR1              = $FF16   // Hintergrundfarbe 1
.const TED_COLOR2              = $FF17   // Hintergrundfarbe 2
.const TED_COLOR3              = $FF18   // Hintergrundfarbe 3
.const TED_BORDERCOLOR         = $FF19   // Rahmenfarbe
.const TED_CHARROW_HI          = $FF1A   // Zeichenzeile hoch
.const TED_CHARROW_LO          = $FF1B   // Zeichenzeile niedrig
.const TED_RASTER_HI           = $FF1C   // Aktuelle Rasterzeile Bit 8
.const TED_RASTER_LO           = $FF1D   // Aktuelle Rasterzeile Bit 0-7
.const TED_HCOUNT              = $FF1E   // Horizontaler Zähler
.const TED_BLINK               = $FF1F   // Blinkzähler, Zeichenzeile 0-7
.const TED_ROM_ON              = $FF3E   // ROM einblenden (Schreiben)
.const TED_RAM_ON              = $FF3F   // RAM einblenden (Schreiben)

// ---- TED: Farbtöne (Farbwert = Ton + 16 * Helligkeit 0-7)
.const TED_COL_BLACK           = $00
.const TED_COL_WHITE           = $01
.const TED_COL_RED             = $02
.const TED_COL_CYAN            = $03
.const TED_COL_PURPLE          = $04
.const TED_COL_GREEN           = $05
.const TED_COL_BLUE            = $06
.const TED_COL_YELLOW          = $07
.const TED_COL_ORANGE          = $08
.const TED_COL_BROWN           = $09
.const TED_COL_YELLOWGREEN     = $0A
.const TED_COL_PINK            = $0B
.const TED_COL_BLUEGREEN       = $0C
.const TED_COL_LIGHTBLUE       = $0D
.const TED_COL_DARKBLUE        = $0E
.const TED_COL_LIGHTGREEN      = $0F

// ---- ACIA 6551 (nur Plus/4)
.const ACIA_DATA               = $FD00   // Daten
.const ACIA_STATUS             = $FD01   // Status (lesen) / Reset (schreiben)
.const ACIA_COMMAND            = $FD02   // Befehlsregister
.const ACIA_CONTROL            = $FD03   // Steuerregister

// ---- 6529-Ports
.const PIO_PORT                = $FD10   // Port 6529 (Anwenderport, Kassettenstatus)
.const PIO_KEYBOARD            = $FD30   // Tastaturmatrix und Joystick-Spaltenauswahl

// ---- ROM-Bankwahl
.const ROM_BANK                = $FDD0   // Funktions-ROM wählen (Schreiben)
