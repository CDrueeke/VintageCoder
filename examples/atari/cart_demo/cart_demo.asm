; =====================================================================================================================
; cart_demo.asm - Atari-Modul (Cartridge): 8-KB-Standardmodul oder XEGS-Modul mit Bankumschaltung
; =====================================================================================================================
;
; ZWECK
;   Zeigt, wie ein Atari-Programm als Steckmodul-Abbild (.car) aufgebaut wird.  Zwei Fassungen aus einer Quelle:
;     * Standard (ohne Schalter): 8-KB-Modul, ROM bei $A000-$BFFF.  Das Programm setzt nur die Hintergrundfarbe und wartet.
;     * -D XEGS=1: XEGS-Modul mit vier 8-KB-Bänken (32 KB).  Die Bänke 0-2 sind über das Bankregister $D500 nach
;       $8000-$9FFF einblendbar; Bank 3 liegt fest bei $A000-$BFFF und enthält den Code.
;
; WAS DER CODE TUT
;   1. SEI (IRQs aus; das Programm kehrt nicht zum OS zurück).
;   2. COLBK = $34 (Hintergrundfarbe Farbton 3, Helligkeit 4) - sichtbares Lebenszeichen.
;   3. Nur XEGS: Die Schleife `next` wählt nacheinander Bank 0, 1, 2 (Schreiben der Banknummer nach $D500), liest die
;      Kennung bei $8100 (Bank 0 = $A0, Bank 1 = $A1, Bank 2 = $A2; durch die `.byte`-Zeilen unten dort abgelegt) und speichert sie bei
;      $0600+Bank.  Danach wird Bank 3 wieder gewählt (A = 3 -> $D500).  Im Speicher stehen am Ende $0600-$0602 = $A0 $A1 $A2;
;      damit lässt sich im Emulator prüfen, dass die Bankumschaltung funktioniert.
;   4. Endlosschleife `done`.
;
; AUFBAU DES MODULS
;   * `.cartridge "8k"` bzw. `.cartridge "xegs"` legt Modultyp und Größe fest; `.bank n` wählt beim XEGS-Modul die Bank,
;     in die die folgenden Daten (`.org $8100`) gehören.  Der Code selbst steht bei `.org $a000`.
;   * Die Modul-Startvektoren ($BFFA-$BFFF) ergänzt der Assembler aus `.run start`.
;   * Das Bankregister $D500 ist hier als Zahl geschrieben (nicht Teil der Chip-Tabellen); $0600 ist die frei benutzbare Seite 6.
;
; BAUEN (vom Verzeichnis des Projektordners aus):
;   vcc asm cart_demo.asm -o build/cart8k.car   --target atari8 --format car --data-dir data
;   vcc asm cart_demo.asm -o build/cartxegs.car --target atari8 --format car -D XEGS=1 --data-dir data
;   (Pfad zur Quelle und zu `data` je nach Arbeitsverzeichnis anpassen.)
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }

.ifdef XEGS
.cartridge "xegs"
.bank 0
.org $8100
        .byte $a0                       ; Kennung Bank 0
.bank 1
.org $8100
        .byte $a1                       ; Kennung Bank 1
.bank 2
.org $8100
        .byte $a2                       ; Kennung Bank 2
.bank 3                                 ; Bank 3 = feste Bank bei $A000 mit dem Code
.else
.cartridge "8k"
.endif
.org $a000
start:  sei
        lda #$34
        sta gtia.colbk                  ; Hintergrund sichtbar färben
.ifdef XEGS
        ldx #0
next:   stx $d500                   ; Bank wählen (X = Banknummer 0..2)
        lda $8100                   ; Kennbyte der gewählten Bank (bei $8000-$9FFF eingeblendet)
        sta $0600,x                 ; Ablage: $0600 + Bank
        inx
        cpx #3
        bne next
        lda #3
        sta $d500                   ; Bank 3 wieder wählen (Grundzustand; der Code bei $A000 liegt ohnehin in der festen Bank)
.endif
done:   jmp done
.run start
