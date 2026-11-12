; cart.inc – C64-Module: Startkopf und Bankumschaltung.
;
; Benutzung (Beispiel siehe cart_demo.asm):
;
;   .cartridge "magicdesk", "MEIN MODUL"      ; Typ und Name (crt-Ausgabe: vcc asm … --format crt, oder automatisch durch .cartridge)
;   .bank 0
;   .org $8000
;   cbm80_header cold, cold                   ; 9 Bytes: Kaltstart-/Warmstartvektor + Kennung „CBM80“ (sonst startet das Modul nicht)
;   cold: ...
;
; Bänke (8 KB, sichtbar bei $8000-$9FFF): .bank n / .org $8000 wählt, wohin der folgende Code kommt; zur Laufzeit schaltet
;   Ocean:      sta $DE00 mit Banknummer 0..63 (Bit 7 stets gesetzt: ROML aktiv; Bänke ab 16 liegen laut Hilfeseite „Module“ bei $A000-$BFFF)
;               → ocean_bank
;   Magic Desk: sta $DE00 mit Banknummer 0..63, Bit 7 = Modul abschalten          → magicdesk_bank
; Der Assembler erzeugt für Ocean 16 bis 64, für Magic Desk 4 bis 16 Bänke (unbenutzte werden mit $FF aufgefüllt).
; Der Code zum Umschalten muss in einer Bank liegen, die in jeder Bank gleich ist (oder im RAM laufen).
; Alle Makros benötigen keine Zeropage und keine weiteren Dateien.

; cbm80_header(cold, warm) – Modulkopf, steht genau bei $8000 (der C64 prüft beim Reset die Bytes $8004-$8008).
;   cold: Adresse, die beim Einschalten/Reset angesprungen wird (Kaltstartvektor $8000/$8001)
;   warm: Adresse für den Warmstart (Vektor $8002/$8003; wird bei einem NMI, z. B. der RESTORE-Taste, angesprungen, solange die Kennung steht)
;   Erzeugt 9 Bytes: .word cold, .word warm, dann „CBM80“.
.macro cbm80_header(cold, warm)
        .word cold
        .word warm
        .byte $c3, $c2, $cd, $38, $30       ; „CBM80“: „CBM“ als PETSCII-Großbuchstaben (Bit 7 gesetzt), „80“ als normale Ziffern
.endmacro

; ocean_bank(n) – Ocean-Modul: Bank n (0..63, Konstante) einblenden. Verändert A und die Flags N und Z; 5 Bytes, 6 Takte.
;   Bit 7 im Wert bleibt gesetzt (ROML aktiv, so verlangt es das Ocean-Register laut Kopfkommentar oben).
.macro ocean_bank(n)
        lda #$80 | n
        sta $de00
.endmacro

; magicdesk_bank(n) – Magic-Desk-Modul: Bank n (0..63, Konstante) einblenden. Verändert A und die Flags N und Z; 5 Bytes, 6 Takte.
;   Achtung: n mit gesetztem Bit 7 (>= 128) würde das Modul abschalten.
.macro magicdesk_bank(n)
        lda #n
        sta $de00
.endmacro
