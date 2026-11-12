// Atari VCS/2600: TIA (Bild, Ton), RIOT (RAM, Ports, Timer), Speicherbereiche und Makros
// Konstanten für den Assembler: #include <vcs2600/tia.asm>

// ---- Speicherbereiche
.const VCS_RAM                 = $0080   // Arbeitsspeicher (128 Byte)
.const VCS_RAM_END             = $00FF
.const VCS_ROM                 = $F000   // Modul-ROM (4 KB, Spiegel ab $1000)

// ---- TIA: Schreibregister
.const VSYNC                   = $00   // Vertikale Synchronisation
.const VBLANK                  = $01   // Vertikale Austastung, Eingänge
.const WSYNC                   = $02   // Auf Zeilenende warten
.const RSYNC                   = $03   // Zeilenzähler zurücksetzen
.const NUSIZ0                  = $04   // Anzahl/Größe Player 0 und Missile 0
.const NUSIZ1                  = $05   // Anzahl/Größe Player 1 und Missile 1
.const COLUP0                  = $06   // Farbe Player 0/Missile 0
.const COLUP1                  = $07   // Farbe Player 1/Missile 1
.const COLUPF                  = $08   // Farbe Spielfeld/Ball
.const COLUBK                  = $09   // Hintergrundfarbe
.const CTRLPF                  = $0A   // Spielfeldsteuerung
.const REFP0                   = $0B   // Player 0 spiegeln
.const REFP1                   = $0C   // Player 1 spiegeln
.const PF0                     = $0D   // Spielfeld 0
.const PF1                     = $0E   // Spielfeld 1
.const PF2                     = $0F   // Spielfeld 2
.const RESP0                   = $10   // Player 0 positionieren
.const RESP1                   = $11   // Player 1 positionieren
.const RESM0                   = $12   // Missile 0 positionieren
.const RESM1                   = $13   // Missile 1 positionieren
.const RESBL                   = $14   // Ball positionieren
.const AUDC0                   = $15   // Kanal 0: Klangart
.const AUDC1                   = $16   // Kanal 1: Klangart
.const AUDF0                   = $17   // Kanal 0: Frequenz
.const AUDF1                   = $18   // Kanal 1: Frequenz
.const AUDV0                   = $19   // Kanal 0: Lautstärke
.const AUDV1                   = $1A   // Kanal 1: Lautstärke
.const GRP0                    = $1B   // Grafik Player 0
.const GRP1                    = $1C   // Grafik Player 1
.const ENAM0                   = $1D   // Missile 0 an
.const ENAM1                   = $1E   // Missile 1 an
.const ENABL                   = $1F   // Ball an
.const HMP0                    = $20   // Feinbewegung Player 0
.const HMP1                    = $21   // Feinbewegung Player 1
.const HMM0                    = $22   // Feinbewegung Missile 0
.const HMM1                    = $23   // Feinbewegung Missile 1
.const HMBL                    = $24   // Feinbewegung Ball
.const VDELP0                  = $25   // Player 0 verzögern
.const VDELP1                  = $26   // Player 1 verzögern
.const VDELBL                  = $27   // Ball verzögern
.const RESMP0                  = $28   // Missile 0 an Player 0 koppeln
.const RESMP1                  = $29   // Missile 1 an Player 1 koppeln
.const HMOVE                   = $2A   // Feinbewegungen ausführen
.const HMCLR                   = $2B   // Feinbewegungen löschen
.const CXCLR                   = $2C   // Kollisionen löschen

// ---- TIA: Leseregister (gleiche Adressen wie VSYNC ff.)
.const CXM0P                   = $00   // Kollision Missile 0 - Player
.const CXM1P                   = $01   // Kollision Missile 1 - Player
.const CXP0FB                  = $02   // Kollision Player 0 - Spielfeld/Ball
.const CXP1FB                  = $03   // Kollision Player 1 - Spielfeld/Ball
.const CXM0FB                  = $04   // Kollision Missile 0 - Spielfeld/Ball
.const CXM1FB                  = $05   // Kollision Missile 1 - Spielfeld/Ball
.const CXBLPF                  = $06   // Kollision Ball - Spielfeld
.const CXPPMM                  = $07   // Kollision Player-Player, Missile-Missile
.const INPT0                   = $08   // Eingang 0
.const INPT1                   = $09   // Eingang 1
.const INPT2                   = $0A   // Eingang 2
.const INPT3                   = $0B   // Eingang 3
.const INPT4                   = $0C   // Eingang 4
.const INPT5                   = $0D   // Eingang 5

// ---- RIOT 6532
.const SWCHA                   = $0280   // Port A: Joysticks
.const SWACNT                  = $0281   // Port A: Datenrichtung
.const SWCHB                   = $0282   // Port B: Konsolenschalter
.const SWBCNT                  = $0283   // Port B: Datenrichtung
.const INTIM                   = $0284   // Timerstand (lesen)
.const TIMINT                  = $0285   // Timer-Status (lesen)
.const TIM1T                   = $0294   // Timer setzen, Takt 1
.const TIM8T                   = $0295   // Timer setzen, Takt 8
.const TIM64T                  = $0296   // Timer setzen, Takt 64
.const T1024T                  = $0297   // Timer setzen, Takt 1024

// ---- Makros
// Standard-Start: Interrupts aus, Dezimalmodus aus, Nullseite (TIA und RAM) und Stapel löschen
.macro CLEAN_START
 sei
 cld
 ldx #0
 txa
 tay
@clear: dex
 txs
 pha
 bne @clear
.endmacro

// Drei Zeilen vertikale Synchronisation (ein Bild beginnt)
.macro VERTICAL_SYNC
 lda #2
 sta VSYNC
 sta WSYNC
 sta WSYNC
 sta WSYNC
 lda #0
 sta VSYNC
.endmacro

// Anzahl Zeilen warten (n >= 1)
.macro WAIT_LINES(n)
 ldx #n
@w: sta WSYNC
 dex
 bne @w
.endmacro
