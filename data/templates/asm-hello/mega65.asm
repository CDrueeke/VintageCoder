; {{name}} – MEGA65: Rahmenfarbe über das VIC-IV-Register wechseln
.target mega65
.cpu 45gs02 { illegals = deny }
#include <mega65/gs4502.asm>   // Makro mega65_enable_vic4
.org $2001
.basicstub start

start:  mega65_enable_vic4      ; VIC-IV freischalten ($47, dann $53 nach $D02F)
.loop:  inc vic2.border
        jmp .loop
