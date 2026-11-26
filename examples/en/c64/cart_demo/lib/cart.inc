; cart.inc - C64 cartridges: start header and bank switching.
;
; Usage (example see cart_demo.asm):
;
;   .cartridge "magicdesk", "MY CARTRIDGE"    ; type and name (crt output: vcc asm ... --format crt, or automatically through .cartridge)
;   .bank 0
;   .org $8000
;   cbm80_header cold, cold                   ; 9 bytes: cold start/warm start vector + signature "CBM80" (the cartridge does not start without it)
;   cold: ...
;
; Banks (8 KB, visible at $8000-$9FFF): .bank n / .org $8000 selects where the following code goes; at run time
;   Ocean:      sta $DE00 with bank number 0..63 (bit 7 always set: ROML active; banks from 16 are at $A000-$BFFF according to the help page "Modules")
;               -> ocean_bank
;   Magic Desk: sta $DE00 with bank number 0..63, bit 7 = switch the cartridge off          -> magicdesk_bank
; The assembler produces 16 to 64 banks for Ocean, 4 to 16 banks for Magic Desk (unused ones are filled with $FF).
; The code that switches must be in a bank that is the same in every bank (or run in RAM).
; None of the macros needs zero page or other files.

; cbm80_header(cold, warm) - cartridge header, located exactly at $8000 (the C64 checks bytes $8004-$8008 at reset).
;   cold: address jumped to at power-on/reset (cold start vector $8000/$8001)
;   warm: address for the warm start (vector $8002/$8003; jumped to on an NMI, e.g. the RESTORE key, as long as the signature is there)
;   Produces 9 bytes: .word cold, .word warm, then "CBM80".
.macro cbm80_header(cold, warm)
        .word cold
        .word warm
        .byte $c3, $c2, $cd, $38, $30       ; "CBM80": "CBM" as PETSCII capital letters (bit 7 set), "80" as normal digits
.endmacro

; ocean_bank(n) - Ocean cartridge: map in bank n (0..63, constant). Changes A and the flags N and Z; 5 bytes, 6 cycles.
;   Bit 7 in the value stays set (ROML active, as the Ocean register requires according to the header comment above).
.macro ocean_bank(n)
        lda #$80 | n
        sta $de00
.endmacro

; magicdesk_bank(n) - Magic Desk cartridge: map in bank n (0..63, constant). Changes A and the flags N and Z; 5 bytes, 6 cycles.
;   Caution: n with bit 7 set (>= 128) would switch the cartridge off.
.macro magicdesk_bank(n)
        lda #n
        sta $de00
.endmacro
