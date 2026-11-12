; cart_demo.asm – Banktest für C64-Module (Magic Desk oder Ocean): zwei 8-KB-Bänke, jede mit eigener Kennung bei $8100
;
; Zweck: zeigt, wie ein Steckmodul mit mehreren ROM-Bänken aufgebaut wird: CBM80-Kopf, .segment/.bank, Bankumschaltung über das
;   Register $DE00 und – weil das umschaltende Programm sich nicht selbst aus dem Fenster wegschalten darf – eine Routine, die ins RAM
;   kopiert und dort mit .pseudopc für ihre Laufadresse übersetzt wird. Makros und Beschreibung der Umschaltung: lib/cart.inc.
;
; Bauen (im Repository-Wurzelverzeichnis; `vcc build examples/c64/cart_demo` erzeugt die Magic-Desk-Fassung build/cart_demo.crt):
;   vcc asm examples/c64/cart_demo/cart_demo.asm -o build/cart_md.crt --target c64                (Magic Desk)
;   vcc asm examples/c64/cart_demo/cart_demo.asm -o build/cart_oc.crt --target c64 -D OCEAN=1     (Ocean)
; Das .crt wird im Emulator als Modul eingesteckt (VICE: `x64sc -cartcrt build/cart_md.crt`). Der Assembler füllt unbenutzte Bänke mit
; $FF auf: Magic Desk ergibt 4 Bänke (32896 Bytes), Ocean 16 Bänke (131392 Bytes) im .crt.
;
; Ablauf nach dem Einschalten/Reset:
;   1. Der C64 findet bei $8004 die Kennung CBM80 und springt über den Kaltstartvektor bei $8000 nach `cold`. Diese Prüfung läuft früh im
;      KERNAL-Reset, vor der Initialisierung von Bildschirm und Ein-/Ausgabe; das Programm initialisiert selbst nichts. Der Bildschirm
;      ist deshalb nicht aufgeräumt, und die Anzeige hängt vom Zustand des Farb-RAM ab. Zum Prüfen genügt der Speichermonitor.
;   2. `cold` kopiert die 23 Bytes von `image` (Umschaltroutine) nach $C000 und ruft sie auf.
;   3. Die Routine im RAM schaltet Bank 1 ein, liest deren Kennung von $8100 nach $0400 (Bildschirm links oben), schaltet Bank 0 wieder
;      ein und legt die dortige Kennung nach $0401 (die Zahlen sind Bildschirmcodes).
;   4. Endlosschleife (`jmp *`). Erfolg heißt: $0400 = $B1 (Bank 1) und $0401 = $B0 (Bank 0).
;
; Speicherplan: Bank 0 $8000-$9FFF (Segment BANK0): CBM80-Kopf $8000-$8008, Programm ab $8009, Kennung $B0 bei $8100.
;               Bank 1 $8000-$9FFF (Segment BANK1): Kennung $B1 bei $8100. RAM: Routine bei $C000 (23 Bytes). Zeropage: unbenutzt.
; Register: $DE00 = Bankregister des Moduls (I/O-Bereich 1); ein Schreibzugriff schaltet die Bank um.
; Grenzen: keine Rückkehr ins BASIC, Interrupts bleiben gesperrt; Testprogramm, kein nützliches Modul.
.target c64
.cpu 6510 { illegals = deny }
.include "lib/cart.inc"             ; Makros cbm80_header, ocean_bank, magicdesk_bank

.ifdef OCEAN
.cartridge "ocean", "BANKTEST OCEAN"            ; Ocean-Typ: 16 bis 64 Bänke, Umschaltung mit Bit 7 gesetzt
.else
.cartridge "magicdesk", "BANKTEST MAGIC DESK"   ; Magic Desk: 4 bis 16 Bänke; Standard, wenn OCEAN nicht definiert ist
.endif

.segment "BANK0", $8000, $9fff      ; Segment benennen und auf das 8-KB-Fenster begrenzen (Überlauf meldet E301)
.bank 0                             ; folgende Bytes gehören in Bank 0
.org $8000
        cbm80_header cold, cold     ; 9 Bytes: Kalt- und Warmstartvektor (beide `cold`) und die Kennung CBM80
cold:   sei
        ldx #0
copy:   lda image,x                 ; Umschaltroutine aus dem ROM ...
        sta $c000,x                 ; ... ins RAM bei $C000 kopieren (höchstens 256 Bytes, sie hat 23)
        inx
        cpx #image_end - image      ; Länge = Differenz der echten (ROM-)Adressen
        bne copy
        jsr $c000                   ; Routine ausführen; sie kehrt mit RTS zurück
        jmp *                       ; fertig: Endlosschleife
image:
.pseudopc $c000                     ; ab hier gelten Labels und `*` als Adressen im RAM ($C000), die Bytes liegen aber hier im ROM
        .ifdef OCEAN
        ocean_bank 1                ; Bank 1 einblenden (ab hier zeigt $8000-$9FFF den Inhalt von Bank 1)
        .else
        magicdesk_bank 1
        .endif
        lda $8100                   ; Kennung von Bank 1 ($B1)
        sta $0400                   ; auf den Bildschirm, linke obere Ecke
        .ifdef OCEAN
        ocean_bank 0                ; zurück zu Bank 0
        .else
        magicdesk_bank 0
        .endif
        lda $8100                   ; Kennung von Bank 0 ($B0)
        sta $0401                   ; daneben
        rts
.endpseudopc
image_end:
.org $8100                          ; neues Segment (gehört weiter zu BANK0): Kennung an der Adresse, die der Test in jeder Bank liest
        .byte $b0                   ; Kennung Bank 0

.segment "BANK1", $8000, $9fff
.bank 1                             ; folgende Bytes gehören in Bank 1 (eigener Programmzähler)
.org $8100
        .byte $b1                   ; Kennung Bank 1
