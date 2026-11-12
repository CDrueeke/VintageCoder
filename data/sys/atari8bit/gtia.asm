// Atari 8-Bit: GTIA (Farben, Player/Missile, Kollisionen, $D000)
// Konstanten für den Assembler: #include <atari8bit/gtia.asm>

// ---- Schreibregister
.const HPOSP0                = $D000   // Player 0: Position
.const HPOSP1                = $D001   // Player 1: Position
.const HPOSP2                = $D002   // Player 2: Position
.const HPOSP3                = $D003   // Player 3: Position
.const HPOSM0                = $D004   // Missile 0: Position
.const HPOSM1                = $D005   // Missile 1: Position
.const HPOSM2                = $D006   // Missile 2: Position
.const HPOSM3                = $D007   // Missile 3: Position
.const SIZEP0                = $D008   // Player 0: Breite
.const SIZEP1                = $D009   // Player 1: Breite
.const SIZEP2                = $D00A   // Player 2: Breite
.const SIZEP3                = $D00B   // Player 3: Breite
.const SIZEM                 = $D00C   // Missile-Breiten
.const GRAFP0                = $D00D   // Player 0: Grafikdaten
.const GRAFP1                = $D00E   // Player 1: Grafikdaten
.const GRAFP2                = $D00F   // Player 2: Grafikdaten
.const GRAFP3                = $D010   // Player 3: Grafikdaten
.const GRAFM                 = $D011   // Missile-Grafikdaten
.const COLPM0                = $D012   // Farbe Player/Missile 0
.const COLPM1                = $D013   // Farbe Player/Missile 1
.const COLPM2                = $D014   // Farbe Player/Missile 2
.const COLPM3                = $D015   // Farbe Player/Missile 3
.const COLPF0                = $D016   // Farbe Spielfeld 0
.const COLPF1                = $D017   // Farbe Spielfeld 1
.const COLPF2                = $D018   // Farbe Spielfeld 2
.const COLPF3                = $D019   // Farbe Spielfeld 3
.const COLBK                 = $D01A   // Hintergrundfarbe
.const PRIOR                 = $D01B   // Priorität und GTIA-Modi
.const VDELAY                = $D01C   // Vertikale Verzögerung
.const GRACTL                = $D01D   // Grafiksteuerung (Player/Missile an, Trigger-Latch)
.const HITCLR                = $D01E   // Kollisionen löschen
.const CONSOL                = $D01F   // Konsolentasten (lesen) / Lautsprecher (schreiben)

// ---- Leseregister
.const M0PF                  = $D000   // Kollision Missile 0 - Spielfeld
.const M1PF                  = $D001   // Kollision Missile 1 - Spielfeld
.const M2PF                  = $D002   // Kollision Missile 2 - Spielfeld
.const M3PF                  = $D003   // Kollision Missile 3 - Spielfeld
.const P0PF                  = $D004   // Kollision Player 0 - Spielfeld
.const P1PF                  = $D005   // Kollision Player 1 - Spielfeld
.const P2PF                  = $D006   // Kollision Player 2 - Spielfeld
.const P3PF                  = $D007   // Kollision Player 3 - Spielfeld
.const M0PL                  = $D008   // Kollision Missile 0 - Player
.const M1PL                  = $D009   // Kollision Missile 1 - Player
.const M2PL                  = $D00A   // Kollision Missile 2 - Player
.const M3PL                  = $D00B   // Kollision Missile 3 - Player
.const P0PL                  = $D00C   // Kollision Player 0 - Player
.const P1PL                  = $D00D   // Kollision Player 1 - Player
.const P2PL                  = $D00E   // Kollision Player 2 - Player
.const P3PL                  = $D00F   // Kollision Player 3 - Player
.const TRIG0                 = $D010   // Feuerknopf 0
.const TRIG1                 = $D011   // Feuerknopf 1
.const TRIG2                 = $D012   // Feuerknopf 2
.const TRIG3                 = $D013   // Feuerknopf 3
.const PAL                   = $D014   // PAL/NTSC-Kennung
