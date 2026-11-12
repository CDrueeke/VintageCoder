// Atari 8-Bit: POKEY (Klang, Tastatur, Paddles, seriell, Zufall, $D200)
// Konstanten für den Assembler: #include <atari8bit/pokey.asm>

// ---- Schreibregister
.const AUDF1                 = $D200   // Kanal 1: Frequenz
.const AUDF2                 = $D202   // Kanal 2: Frequenz
.const AUDF3                 = $D204   // Kanal 3: Frequenz
.const AUDF4                 = $D206   // Kanal 4: Frequenz
.const AUDC1                 = $D201   // Kanal 1: Verzerrung und Lautstärke
.const AUDC2                 = $D203   // Kanal 2: Verzerrung und Lautstärke
.const AUDC3                 = $D205   // Kanal 3: Verzerrung und Lautstärke
.const AUDC4                 = $D207   // Kanal 4: Verzerrung und Lautstärke
.const AUDCTL                = $D208   // Audiosteuerung (Taktung, Filter, 16 Bit)
.const STIMER                = $D209   // Timer starten
.const SKREST                = $D20A   // Serielle Statusbits löschen
.const POTGO                 = $D20B   // Paddle-Abtastung starten
.const SEROUT                = $D20D   // Serielle Ausgabe
.const IRQEN                 = $D20E   // IRQ-Freigabe
.const SKCTL                 = $D20F   // Seriell-/Tastatursteuerung

// ---- Leseregister
.const POT0                  = $D200   // Paddle 0
.const POT1                  = $D201   // Paddle 1
.const POT2                  = $D202   // Paddle 2
.const POT3                  = $D203   // Paddle 3
.const POT4                  = $D204   // Paddle 4
.const POT5                  = $D205   // Paddle 5
.const POT6                  = $D206   // Paddle 6
.const POT7                  = $D207   // Paddle 7
.const ALLPOT                = $D208   // Paddle-Abtaststatus
.const KBCODE                = $D209   // Tastaturcode
.const RANDOM                = $D20A   // Zufallszahl
.const SERIN                 = $D20D   // Serielle Eingabe
.const IRQST                 = $D20E   // IRQ-Status
.const SKSTAT                = $D20F   // Seriell-/Tastaturstatus
