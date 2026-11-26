; cart_demo.asm - bank test for C64 cartridges (Magic Desk or Ocean): two 8 KB banks, each with its own ID at $8100
;
; Purpose: shows how a cartridge with several ROM banks is built: CBM80 header, .segment/.bank, bank switching through the
;   register $DE00 and - because the program that switches must not switch itself out of the window - a routine that is copied
;   to RAM and assembled there with .relocate for its run address. Macros and description of the switching: lib/cart.inc.
;
; Build (in the repository root; `vcc build examples/c64/cart_demo` produces the Magic Desk version build/cart_demo.crt):
;   vcc asm examples/c64/cart_demo/cart_demo.asm -o build/cart_md.crt --target c64                (Magic Desk)
;   vcc asm examples/c64/cart_demo/cart_demo.asm -o build/cart_oc.crt --target c64 -D OCEAN=1     (Ocean)
; The .crt is plugged in as a cartridge in the emulator (VICE: `x64sc -cartcrt build/cart_md.crt`). The assembler fills unused banks with
; $FF: Magic Desk gives 4 banks (32896 bytes), Ocean 16 banks (131392 bytes) in the .crt.
;
; Sequence after power-on/reset:
;   1. The C64 finds the CBM80 signature at $8004 and jumps through the cold start vector at $8000 to `cold`. This check runs early in the
;      KERNAL reset, before the initialization of screen and I/O; the program initializes nothing itself. The screen
;      is therefore not cleaned up, and the display depends on the state of color RAM. The memory monitor is enough for checking.
;   2. `cold` copies the 23 bytes of `image` (switching routine) to $C000 and calls it.
;   3. The routine in RAM switches bank 1 in, reads its ID from $8100 to $0400 (screen top left), switches bank 0 in again
;      and puts the ID found there to $0401 (the numbers are screen codes).
;   4. Endless loop (`jmp *`). Success means: $0400 = $B1 (bank 1) and $0401 = $B0 (bank 0).
;
; Memory map: bank 0 $8000-$9FFF (segment BANK0): CBM80 header $8000-$8008, program from $8009, ID $B0 at $8100.
;               Bank 1 $8000-$9FFF (segment BANK1): ID $B1 at $8100. RAM: routine at $C000 (23 bytes). Zero page: unused.
; Register: $DE00 = bank register of the cartridge (I/O area 1); a write access switches the bank.
; Limits: no return to BASIC, interrupts stay disabled; test program, not a useful cartridge.
.target c64
.cpu 6510 { illegals = deny }
.include "lib/cart.inc"             ; macros cbm80_header, ocean_bank, magicdesk_bank

.ifdef OCEAN
.cartridge "ocean", "BANK TEST OCEAN"            ; Ocean type: 16 to 64 banks, switching with bit 7 set
.else
.cartridge "magicdesk", "BANK TEST MAGIC DESK"   ; Magic Desk: 4 to 16 banks; default if OCEAN is not defined
.endif

.segment "BANK0", $8000, $9fff      ; name the segment and limit it to the 8 KB window (overflow is reported as E301)
.bank 0                             ; following bytes belong to bank 0
.org $8000
        cbm80_header cold, cold     ; 9 bytes: cold and warm start vector (both `cold`) and the CBM80 signature
cold:   sei
        ldx #0
copy:   lda image,x                 ; switching routine from ROM ...
        sta $c000,x                 ; ... copy to RAM at $C000 (at most 256 bytes, it has 23)
        inx
        cpx #image_end - image      ; length = difference of the real (ROM) addresses
        bne copy
        jsr $c000                   ; run the routine; it returns with RTS
        jmp *                       ; done: endless loop
image:
.relocate $c000                     ; from here on labels and `*` count as addresses in RAM ($C000), but the bytes are located here in ROM
        .ifdef OCEAN
        ocean_bank 1                ; map in bank 1 (from here $8000-$9FFF shows the contents of bank 1)
        .else
        magicdesk_bank 1
        .endif
        lda $8100                   ; ID of bank 1 ($B1)
        sta $0400                   ; to the screen, top left corner
        .ifdef OCEAN
        ocean_bank 0                ; back to bank 0
        .else
        magicdesk_bank 0
        .endif
        lda $8100                   ; ID of bank 0 ($B0)
        sta $0401                   ; next to it
        rts
.endrelocate
image_end:
.org $8100                          ; new segment (still belongs to BANK0): ID at the address the test reads in every bank
        .byte $b0                   ; ID bank 0

.segment "BANK1", $8000, $9fff
.bank 1                             ; following bytes belong to bank 1 (own program counter)
.org $8100
        .byte $b1                   ; ID bank 1
