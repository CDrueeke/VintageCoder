; vic4.inc – MEGA65: VIC-IV freischalten, 256-Farben-Palette in eine Palettenbank laden und für Text/Bitmap wählen.
; (Einen Vollfarbmodus/FCM enthält diese Datei nicht; dafür siehe examples/mega65/prallball.)
;
;   .include "lib/vic4.inc"
;   vic4_unlock                                   ; VIC-IV-Register freischalten ($47, $53 nach $D02F)
;   vic4_load_palette pal_red, pal_green, pal_blue, 1    ; 256 Farben in Palettenbank 1 laden (Export einer Palette)
;   vic4_use_palette 1                            ; Bank 1 für Text/Bitmap (und Sprites) wählen
;   ...
;   vic4_palette_runtime                          ; EINMAL hinter dem Hauptprogramm (nicht ausgeführt)
;
; Die Tabellen kommen aus dem Export einer Palette (`<name>_red`, `_green`, `_blue`, je 256 Bytes mit vertauschten Halbbytes).
; Palettenbank wählt $D070: Bits 7-6 = Bank, die bei $D100-$D3FF eingeblendet wird; Bits 5-4 = Text/Bitmap; Bits 3-2 = Sprites; Bits 1-0 = alternative Palette.
; Belegte Zeropage: $F7-$FA ($F7/$F8 = Quellzeiger, $F9/$FA = Zielzeiger beim Kopieren).
; Hinweis: Die Makros enthalten kein SEI/CLI; wer in einem Interrupt dieselbe Palette benutzt, muss selbst sperren.  Alle Makros
; verändern A (vic4_load_palette auch Y und die Zeropage); die Registerbelegung von $D070 stammt aus der MEGA65-Registerbeschreibung.

#include <mega65/gs4502.asm>

; vic4_unlock: VIC-IV-Register freischalten (Voraussetzung für $D070).  Verändert A.
.macro vic4_unlock
        mega65_enable_vic4                            ; Makro aus <mega65/gs4502.asm>: $47, $53 nach $D02F
.endmacro

; vic4_load_palette(red, green, blue, bank): Die drei 256-Byte-Tabellen red/green/blue (Halbbytes vertauscht, Export einer Palette) in die
;   Register $D100/$D200/$D300 der Palettenbank `bank` (0-3) kopieren.  Braucht vic4_palette_runtime (Unterprogramm vic4_copy_page).
;   Verändert A, Y, $F7-$FA und setzt $D070 am Ende auf 0 (Bank 0 eingeblendet, Text/Bitmap/Sprites Bank 0).
.macro vic4_load_palette(red, green, blue, bank)
        lda #(bank & 3) << 6        ; Bank bei $D100-$D3FF einblenden
        sta $d070
        lda #<red
        sta $f7
        lda #>red
        sta $f8
        lda #$00
        sta $f9
        lda #$d1
        sta $fa
        jsr vic4_copy_page
        lda #<green
        sta $f7
        lda #>green
        sta $f8
        lda #$d2
        sta $fa
        jsr vic4_copy_page
        lda #<blue
        sta $f7
        lda #>blue
        sta $f8
        lda #$d3
        sta $fa
        jsr vic4_copy_page
        lda #0
        sta $d070                   ; zurück auf Bank 0 (löscht dabei auch alle anderen Bits von $D070!)
.endmacro

; vic4_use_palette(bank): Palettenbank `bank` (0-3) für Text/Bitmap wählen ($D070 Bit 5-4); die übrigen Bits bleiben.  Verändert A.
.macro vic4_use_palette(bank)
        lda $d070
        and #%11001111
        ora #(bank & 3) << 4        ; Text/Bitmap
        sta $d070
.endmacro

; vic4_palette_runtime: das Kopierunterprogramm vic4_copy_page, EINMAL hinter dem Hauptprogramm ablegen (nicht im Ablauf).
.macro vic4_palette_runtime
vic4_copy_page:                     ; Eingabe: ($F7) = Quelle, ($F9) = Ziel; kopiert 256 Byte mit Y = 0..255 (Quelle beliebig, hier Low-Byte 0 beim Ziel); verändert A, Y
        ldy #0
@l:     lda ($f7),y
        sta ($f9),y
        iny
        bne @l
        rts
.endmacro
