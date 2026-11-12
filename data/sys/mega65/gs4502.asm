// MEGA65: Hilfsmakros für den 45GS02 (VIC-IV freischalten, 40 MHz, 32-Bit-Werte)
// Konstanten für den Assembler: #include <mega65/gs4502.asm>

#include <mega65/hardware.asm>

.macro mega65_enable_vic4
// Schaltet die erweiterten VIC-IV-Register frei. Der VIC-IV ist der erweiterte Grafikchip des MEGA65; seine Register sind nach dem
// Einschalten gesperrt (C64-Verträglichkeit) und bleiben ohne diese Schlüsselfolge wirkungslos: $47, dann $53 nach VIC4_KEY ($D02F).
// Aufruf: mega65_enable_vic4   (keine Argumente; verändert A). Einmal am Programmanfang genügt.
// Gleichwertig von Hand: lda #$47 / sta vic4.key / lda #$53 / sta vic4.key
 lda #$47
 sta VIC4_KEY
 lda #$53
 sta VIC4_KEY
.endmacro

.macro mega65_fast
// 40 MHz einschalten (65 in die Datenrichtung des Prozessorports)
 lda #65
 sta CPU_DDR
.endmacro

.macro ld32(dst, val)
// 32-Bit-Wert (niedrigstes Byte zuerst) an dst..dst+3 schreiben
 lda #(val) & $ff
 sta dst
 lda #((val) >> 8) & $ff
 sta dst+1
 lda #((val) >> 16) & $ff
 sta dst+2
 lda #((val) >> 24) & $ff
 sta dst+3
.endmacro
