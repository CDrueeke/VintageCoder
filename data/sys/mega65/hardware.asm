// MEGA65: Hardware (VIC-II/SID/CIA wie C64, dazu VIC-IV, DMAgic, Hypervisor)
// Konstanten für den Assembler: #include <mega65/hardware.asm>

#include <c64/hardware.asm>
#include <mega65/vic4.asm>

// ---- Speicherbereiche
.const SCREEN_RAM_C65          = $0800   // Bildschirm im C65/MEGA65-Modus
.const BASIC_START             = $2001   // BASIC-65-Programmanfang
