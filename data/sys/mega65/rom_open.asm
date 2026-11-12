// MEGA65: OpenROMs (quelloffenes ROM)
// Konstanten für den Assembler: #include <mega65/rom_open.asm>

// Die OpenROMs garantieren nur die Sprungtabelle (mega65/kernal.asm). Interne Adressen und Variablen sind nicht festgelegt und unterscheiden sich von
// den geschlossenen ROMs; deshalb enthält diese Datei keine Systemvariablen.
.ifdef MEGA65_ROM
 .error "Es darf nur eine mega65/rom_*.asm eingebunden werden"
.endif
.const MEGA65_ROM = 1        // OpenROMs; zum Abfragen im Quelltext: .if MEGA65_ROM == 1
#include <mega65/kernal.asm>
