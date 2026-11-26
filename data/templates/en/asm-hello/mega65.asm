; {{name}} - MEGA65: change border color via the VIC-IV register
.target mega65
.cpu 45gs02 { illegals = deny }
#include <mega65/gs4502.asm>   // macro mega65_enable_vic4
.org $2001
.basicstub start

start:  mega65_enable_vic4      ; unlock VIC-IV ($47, then $53 to $D02F)
.loop:  inc vic2.border
        jmp .loop
