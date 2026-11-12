// Atari 8-Bit: ANTIC (Anzeigeprozessor, $D400)
// Konstanten für den Assembler: #include <atari8bit/antic.asm>

// ---- ANTIC
.const DMACTL                = $D400   // DMA-Steuerung (Spielfeldbreite, Player/Missile, Display-List)
.const CHACTL                = $D401   // Zeichensatz-Steuerung
.const DLISTL                = $D402   // Display-List-Adresse niedrig
.const DLISTH                = $D403   // Display-List-Adresse hoch
.const HSCROL                = $D404   // Horizontales Feinscrollen
.const VSCROL                = $D405   // Vertikales Feinscrollen
.const PMBASE                = $D407   // Player/Missile-Basisseite
.const CHBASE                = $D409   // Zeichensatz-Basisseite
.const WSYNC                 = $D40A   // Auf Zeilenende warten (Schreiben)
.const VCOUNT                = $D40B   // Rasterzeile / 2 (lesen)
.const PENH                  = $D40C   // Lichtgriffel horizontal
.const PENV                  = $D40D   // Lichtgriffel vertikal
.const NMIEN                 = $D40E   // NMI freigeben (DLI, VBI)
.const NMIRES                = $D40F   // NMI-Status löschen (Schreiben)
.const NMIST                 = $D40F   // NMI-Status (Lesen)
