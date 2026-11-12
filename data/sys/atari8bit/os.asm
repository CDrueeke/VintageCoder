// Atari 8-Bit: Betriebssystem (Nullseite, Schattenregister, IOCB, Vektoren), PIA
// Konstanten für den Assembler: #include <atari8bit/os.asm>

// ---- PIA ($D300)
.const PORTA                 = $D300   // Port A (Joysticks 1/2)
.const PORTB                 = $D301   // Port B (XL/XE: Speicherbelegung)
.const PACTL                 = $D302   // Steuerung Port A
.const PBCTL                 = $D303   // Steuerung Port B

// ---- Nullseite
.const WARMST                = $0008   // Warmstart-Flag
.const BOOT                  = $0009   // Boot-Status
.const DOSVEC                = $000A   // DOS-Startvektor
.const DOSINI                = $000C   // DOS-Initialisierungsvektor
.const APPMHI                = $000E   // Obere Grenze des Anwenderspeichers
.const POKMSK                = $0010   // IRQEN-Schatten
.const BRKKEY                = $0011   // BREAK-Flag
.const RTCLOK                = $0012   // Echtzeituhr (3 Byte, 1/50 bzw. 1/60 s)
.const SOUNDR                = $0041   // Ton bei Ein-/Ausgabe
.const CRITIC                = $0042   // Kritischer Abschnitt
.const ATRACT                = $004D   // Bildschirmschoner-Zähler
.const DRKMSK                = $004E   // Abdunklungsmaske
.const COLRSH                = $004F   // Farbverschiebung
.const LMARGN                = $0052   // Linker Rand
.const RMARGN                = $0053   // Rechter Rand
.const ROWCRS                = $0054   // Cursor-Zeile
.const COLCRS                = $0055   // Cursor-Spalte (2 Byte)
.const DINDEX                = $0057   // Aktueller Grafikmodus
.const SAVMSC                = $0058   // Zeiger auf Bildschirmspeicher
.const RAMTOP                = $006A   // RAM-Obergrenze (Seiten)

// ---- Interrupt-Vektoren und Schatten
.const VDSLST                = $0200   // DLI-Vektor
.const VPRCED                = $0202   // Serial-Proceed-Vektor
.const VINTER                = $0204   // Serial-Interrupt-Vektor
.const VBREAK                = $0206   // BRK-Vektor
.const VKEYBD                = $0208   // Tastatur-IRQ-Vektor
.const VSERIN                = $020A   // Serial-Eingabe bereit
.const VSEROR                = $020C   // Serial-Ausgabe bereit
.const VSEROC                = $020E   // Serial-Ausgabe fertig
.const VTIMR1                = $0210   // POKEY-Timer 1
.const VTIMR2                = $0212   // POKEY-Timer 2
.const VTIMR4                = $0214   // POKEY-Timer 4
.const VIMIRQ                = $0216   // IRQ-Vektor
.const CDTMV1                = $0218   // Systemzeitgeber 1
.const CDTMV2                = $021A   // Systemzeitgeber 2
.const CDTMV3                = $021C   // Systemzeitgeber 3
.const CDTMV4                = $021E   // Systemzeitgeber 4
.const CDTMV5                = $0220   // Systemzeitgeber 5
.const VVBLKI                = $0222   // Immediate-VBI-Vektor
.const VVBLKD                = $0224   // Deferred-VBI-Vektor
.const SDMCTL                = $022F   // DMACTL-Schatten
.const SDLSTL                = $0230   // Display-List-Schatten (niedrig)
.const SDLSTH                = $0231   // Display-List-Schatten (hoch)
.const SSKCTL                = $0232   // SKCTL-Schatten
.const BRKKY                 = $0236   // BREAK-Vektor
.const GPRIOR                = $026F   // PRIOR-Schatten
.const PADDL0                = $0270   // Paddle 0-7 (8 Byte)
.const STICK0                = $0278   // Joystick 0-3 (4 Byte)
.const PTRIG0                = $027C   // Paddle-Knöpfe 0-7 (8 Byte)
.const STRIG0                = $0284   // Joystick-Knöpfe 0-3 (4 Byte)
.const PCOLR0                = $02C0   // COLPM0-Schatten
.const PCOLR1                = $02C1   // COLPM1-Schatten
.const PCOLR2                = $02C2   // COLPM2-Schatten
.const PCOLR3                = $02C3   // COLPM3-Schatten
.const COLOR0                = $02C4   // COLPF0-Schatten
.const COLOR1                = $02C5   // COLPF1-Schatten
.const COLOR2                = $02C6   // COLPF2-Schatten
.const COLOR3                = $02C7   // COLPF3-Schatten
.const COLOR4                = $02C8   // COLBK-Schatten
.const MEMTOP                = $02E5   // Obergrenze des freien Speichers
.const MEMLO                 = $02E7   // Untergrenze des freien Speichers
.const CHACT                 = $02F3   // CHACTL-Schatten
.const CHBAS                 = $02F4   // CHBASE-Schatten
.const CH                    = $02FC   // Letzter Tastaturcode ($FF = keiner)

// ---- IOCB (8 Blöcke zu 16 Byte ab $0340)
.const IOCB0                 = $0340   // Kanal 0 (Editor)
.const IOCB1                 = $0350   // Kanal 1
.const IOCB2                 = $0360   // Kanal 2
.const IOCB3                 = $0370   // Kanal 3
.const IOCB4                 = $0380   // Kanal 4
.const IOCB5                 = $0390   // Kanal 5
.const IOCB6                 = $03A0   // Kanal 6
.const IOCB7                 = $03B0   // Kanal 7
.const ICHID                 = $0340   // Offset: Gerätekennung
.const ICDNO                 = $0341   // Offset: Gerätenummer
.const ICCOM                 = $0342   // Offset: Kommando
.const ICSTA                 = $0343   // Offset: Status
.const ICBAL                 = $0344   // Offset: Pufferadresse niedrig
.const ICBAH                 = $0345   // Offset: Pufferadresse hoch
.const ICPTL                 = $0346   // Offset: PUT-Routine niedrig
.const ICPTH                 = $0347   // Offset: PUT-Routine hoch
.const ICBLL                 = $0348   // Offset: Pufferlänge niedrig
.const ICBLH                 = $0349   // Offset: Pufferlänge hoch
.const ICAX1                 = $034A   // Offset: Zusatz 1 (Öffnungsmodus)
.const ICAX2                 = $034B   // Offset: Zusatz 2
.const ICAX3                 = $034C   // Offset: Zusatz 3
.const ICAX4                 = $034D   // Offset: Zusatz 4

// ---- CIO-Kommandos und Modi
.const CIO_OPEN              = $0003   // OPEN
.const CIO_GETREC            = $0005   // Zeile lesen
.const CIO_GETCHR            = $0007   // Bytes lesen
.const CIO_PUTREC            = $0009   // Zeile schreiben
.const CIO_PUTCHR            = $000B   // Bytes schreiben
.const CIO_CLOSE             = $000C   // CLOSE
.const CIO_STATUS            = $000D   // STATUS
.const CIO_SPECIAL           = $000E   // Gerätespezifisch
.const OPNIN                 = $0004   // ICAX1: Lesen
.const OPNOT                 = $0008   // ICAX1: Schreiben
.const OPNINOT               = $000C   // ICAX1: Lesen und Schreiben

// ---- Betriebssystem-Einsprünge
.const EDITRV                = $E400   // Editor-Gerätetabelle
.const SCRENV                = $E410   // Bildschirm-Gerätetabelle
.const KEYBDV                = $E420   // Tastatur-Gerätetabelle
.const PRINTV                = $E430   // Drucker-Gerätetabelle
.const CASETV                = $E440   // Kassetten-Gerätetabelle
.const CIOV                  = $E456   // Zentrale Ein-/Ausgabe
.const SIOV                  = $E459   // Serielle Ein-/Ausgabe
.const SETVBV                = $E45C   // VBI-Vektoren und Zeitgeber setzen
.const SYSVBV                = $E45F   // Ende-Sprung für Immediate-VBI
.const XITVBV                = $E462   // Ende-Sprung für Deferred-VBI
.const WARMSV                = $E474   // Warmstart
.const COLDSV                = $E477   // Kaltstart

// ---- Fließkommapaket des Betriebssystems (6-Byte-BCD: Byte 0 = Vorzeichen + Exponent zur Basis 100 (Excess $40), Byte 1-5 = Ziffernpaare)
.const FR0                   = $00D4   // Gleitkomma-Register 0 (6 Byte); Ganzzahl-Ein-/Ausgabe: FR0 niedrig, FR0+1 hoch
.const FR1                   = $00E0   // Gleitkomma-Register 1 (6 Byte)
.const CIX                   = $00F2   // Index in den Eingabepuffer (AFP)
.const INBUFF                = $00F3   // Zeiger auf den Text (AFP liest, FASC liefert)
.const FLPTR                 = $00FC   // Zeiger für FLD0P/FLD1P/FST0P
.const LBUFF                 = $0580   // Zeilenpuffer des Pakets (FASC schreibt hierhin)
.const AFP                   = $D800   // Text ab (INBUFF)+CIX -> FR0; C gesetzt: kein gültiges Zahlenformat
.const FASC                  = $D8E6   // FR0 -> Text bei (INBUFF); letztes Zeichen hat Bit 7 gesetzt
.const IFP                   = $D9AA   // Ganzzahl 0..65535 in FR0/FR0+1 -> FR0
.const FPI                   = $D9D2   // FR0 -> Ganzzahl (gerundet) in FR0/FR0+1; C gesetzt: außerhalb 0..65535
.const ZFR0                  = $DA44   // FR0 = 0
.const ZF1                   = $DA46   // 6 Byte ab (X) auf Null setzen
.const FSUB                  = $DA60   // FR0 = FR0 - FR1; C gesetzt: Überlauf
.const FADD                  = $DA66   // FR0 = FR0 + FR1
.const FMUL                  = $DADB   // FR0 = FR0 * FR1
.const FDIV                  = $DB28   // FR0 = FR0 / FR1; C gesetzt: Division durch 0 oder Überlauf
.const FLD0R                 = $DD89   // FR0 = (X niedrig/Y hoch)
.const FLD0P                 = $DD8D   // FR0 = (FLPTR)
.const FLD1R                 = $DD98   // FR1 = (X/Y)
.const FLD1P                 = $DD9C   // FR1 = (FLPTR)
.const FST0R                 = $DDA7   // (X/Y) = FR0
.const FST0P                 = $DDAB   // (FLPTR) = FR0
.const FMOVE                 = $DDB6   // FR1 = FR0
.const EXP                   = $DDC0   // FR0 = e^FR0; C gesetzt: Überlauf
.const EXP10                 = $DDCC   // FR0 = 10^FR0
.const LOG                   = $DECD   // FR0 = ln(FR0); C gesetzt: FR0 <= 0
.const LOG10                 = $DED1   // FR0 = log10(FR0)
.const PLYEVL                = $DD40   // Polynom auswerten: A = Anzahl der Koeffizienten, X/Y = Tafel (6-Byte-BCD, höchste Potenz zuerst), FR0 = Argument
