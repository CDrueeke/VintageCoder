// PET/CBM (2001, 3000er, 4000er, 8000er): Speicherbereiche, PIAs, VIA, CRTC
// Konstanten für den Assembler: #include <pet/hardware.asm>
// Spike-Stand: Adressen aus dem Gedächtnis, nur der Bildschirmspeicher und CHROUT sind im Emulator (VICE xpet 3.10) bestätigt.

// ---- Speicherbereiche
.const BASIC_START             = $0401   // BASIC-Programmanfang (BASIC 2 und 4)
.const SCREEN_RAM              = $8000   // Bildschirmspeicher (40 Spalten: 1000 Byte, 80 Spalten: 2000 Byte), Bildschirmcodes
.const CASSETTE_BUF1           = $027A   // Kassettenpuffer #1 (192 Byte)
.const CASSETTE_BUF2           = $033A   // Kassettenpuffer #2 (192 Byte)

// ---- PIA 1 (Tastatur, Kassette)
.const PIA1_PORTA              = $E810
.const PIA1_CRA                = $E811
.const PIA1_PORTB              = $E812
.const PIA1_CRB                = $E813

// ---- PIA 2 (IEEE-488)
.const PIA2_PORTA              = $E820
.const PIA2_CRA                = $E821
.const PIA2_PORTB              = $E822
.const PIA2_CRB                = $E823

// ---- VIA (Benutzeranschluss, Ton, IEEE-Steuerung)
.const VIA_ORB                 = $E840
.const VIA_ORA                 = $E841
.const VIA_DDRB                = $E842
.const VIA_DDRA                = $E843
.const VIA_T1C_LO              = $E844
.const VIA_T1C_HI              = $E845
.const VIA_T2C_LO              = $E848
.const VIA_T2C_HI              = $E849
.const VIA_SR                  = $E84A
.const VIA_ACR                 = $E84B
.const VIA_PCR                 = $E84C
.const VIA_IFR                 = $E84D
.const VIA_IER                 = $E84E

// ---- CRTC (nur 8032 und 8096/8296; bei den 40-Spalten-Geräten gibt es keinen Chip)
.const CRTC_ADDR               = $E880
.const CRTC_DATA               = $E881
