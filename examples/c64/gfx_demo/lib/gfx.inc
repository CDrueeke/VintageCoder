; gfx.inc – C64: Zeichensätze, Bildschirme und Kachelkarten aus Exporten laden.
;
; Benutzung:
;   .include "lib/gfx.inc"
;   ...
;   gfx_load_charset font, $2800          ; 2048 Bytes (Label aus dem ASM-Export)
;   gfx_charset_at $2800                  ; VIC auf den neuen Zeichensatz zeigen lassen
;   gfx_load_screen title_chars, title_attrs
;   gfx_colors title_border, title_bg
;   tiles_use level_tiles_c0, level_tiles_col, level_tiles_props, level_tiles_tw, level_tiles_th, level_tiles_count, level_screens_lo, level_screens_hi, level_level_tiles_w, level_level_tiles_h
;   gfx_draw_map level_level_id            ; Kachelkarte (Screen) der Bank zeichnen: dünner Mantel um screen_draw der Kachel-Bibliothek
;   ...
;   gfx_runtime                           ; EINMAL, an einer Stelle, die nicht ausgeführt wird (z. B. hinter dem Hauptprogramm)
;   tiles_runtime                         ; EINMAL, nur wenn Kachelkarten gezeichnet werden (Unterprogramme der Kachel-Bibliothek)
;
; Kachelkarten zeichnet die Kachel-Bibliothek (#include <c64/tiles.asm>, unten eingebunden): Kachelsatz (.vctiles) und Screen-Bank (.vcscreen)
; mit dem Editor entwerfen, exportieren (build/export/*.asm) und mit tiles_use binden; Befehle und Schalter stehen im Kopf von common/tiles.asm.
;
; Überblick über die Makros:
;   gfx_copy src, dst, len      allgemeines Kopieren (benutzt gfx_memcpy)           gfx_clear char, color      Text und Farbe füllen
;   gfx_load_charset src, dst   2048 Bytes Zeichensatz kopieren                     gfx_colors border, bg      Rahmen und Hintergrund
;   gfx_charset_at addr         $D018: Zeichensatzadresse einstellen                gfx_draw_map [sn]          Kachelkarte zeichnen
;   gfx_screen_at addr          $D018 und $0288: Bildschirmadresse einstellen       gfx_load_screen chars, attrs  Bildschirm + Farb-RAM laden
;   gfx_vic_bank addr           VIC-Bank (16 KB) wählen                             gfx_load_screen_chars chars   nur die Zeichencodes laden
; Alle Makros erwarten Konstanten (Labels/Zahlen) als Argumente. Sie ändern A und die Flags; gfx_clear zusätzlich X, die Kopier-Makros
; (gfx_copy, gfx_load_*) zusätzlich X, Y und die Zeropage-Zellen gfx_src, gfx_dst, gfx_len. Ein Interrupt, der dieselbe Zeropage-Gruppe
; benutzt, darf nicht zwischen Makroaufruf und gfx_memcpy dazwischenfahren.
;
; Belegte Zeropage: $F7-$FE (Zeiger; die Bibliothek benutzt dieselbe Gruppe, nie gleichzeitig mit gfx_memcpy):
;   gfx_len $F7/$F8 (Länge), gfx_src $FB/$FC (Quelle), gfx_dst $FD/$FE (Ziel); $F9/$FA bleiben von diesen Makros unberührt.
; Der Bildschirm liegt bei gfx_screen (Standard $0400, vor dem Include änderbar mit .const GFX_SCREEN = $0800 oder -D GFX_SCREEN=$0800;
; muss auf 256 Byte ausgerichtet sein). Das Farb-RAM ist fest $D800. Kartengröße: Grenzen der Kachel-Bibliothek.

.ifdef GFX_SCREEN
.const gfx_screen = GFX_SCREEN
.const TILE_SCREEN = GFX_SCREEN    ; die Kachel-Bibliothek zeichnet in denselben Bildschirm
.else
.const gfx_screen = $0400
.endif
#include <c64/tiles.asm>
.const gfx_colorram = $d800     ; Farb-RAM (nur die Bits 0-3 jedes Bytes zählen)
.const gfx_len  = $f7           ; 16 Bit: Länge beim Kopieren
.const gfx_src  = $fb           ; 16 Bit: Quelle
.const gfx_dst  = $fd           ; 16 Bit: Ziel

; --- Kopieren ---------------------------------------------------------------------------------------------------
; gfx_copy(src, dst, len) – Zeropage-Zeiger und Länge setzen und gfx_memcpy aufrufen (benötigt gfx_runtime irgendwo im Programm).
.macro gfx_copy(src, dst, len)      ; kopiert len Bytes von src nach dst (len höchstens 65535, Bereiche dürfen nicht überlappen)
        lda #<src
        sta gfx_src
        lda #>src
        sta gfx_src+1
        lda #<dst
        sta gfx_dst
        lda #>dst
        sta gfx_dst+1
        lda #<len
        sta gfx_len
        lda #>len
        sta gfx_len+1
        jsr gfx_memcpy
.endmacro

; --- Zeichensatz --------------------------------------------------------------------------------------------------
; gfx_load_charset(src, dst) – src = Label des Zeichensatzes (z. B. aus dem Export), dst = Zieladresse im RAM.
.macro gfx_load_charset(src, dst)   ; 256 Zeichen = 2048 Bytes; dst auf 2 KB ausgerichtet und im VIC-Bereich (nicht $1000-$1FFF/$9000-$9FFF)
        gfx_copy src, dst, 2048
.endmacro

; gfx_charset_at(addr) – addr = Adresse des Zeichensatzes (2 KB ausgerichtet). In $D018 sind die Bits 1-3 der Zeiger auf den Zeichensatz
;   in 2-KB-Schritten innerhalb der VIC-Bank; die Bits 4-7 (Bildschirmzeiger) bleiben erhalten.
.macro gfx_charset_at(addr)         ; Zeichensatzadresse im VIC-$d018 setzen (Bank 0: $0000-$3FFF; sonst zuerst gfx_vic_bank)
        lda $d018
        and #%11110001
        ora #((addr & $3fff) >> 11) << 1
        sta $d018
.endmacro

; gfx_screen_at(addr) – addr = Adresse des Bildschirmspeichers (1 KB ausgerichtet). In $D018 sind die Bits 4-7 der Bildschirmzeiger in
;   1-KB-Schritten innerhalb der VIC-Bank; die Bits 0-3 (Zeichensatzzeiger) bleiben erhalten. Das KERNAL schreibt seine Textausgabe
;   an die Seite, die in $0288 steht (hier die obere Adresshälfte von addr), damit PRINT und der Cursor denselben Bildschirm benutzen.
.macro gfx_screen_at(addr)          ; Bildschirmadresse im VIC-$d018 setzen (1-KB-Raster innerhalb der Bank); KERNAL-Ausgabe folgt per $0288
        lda $d018
        and #%00001111
        ora #((addr & $3fff) >> 10) << 4
        sta $d018
        lda #>addr
        sta $0288
.endmacro

; gfx_vic_bank(addr) – VIC-II auf eine andere 16-KB-Bank umschalten. Die Bankbits stehen invertiert in den Bits 0-1 von $DD00
;   (Port A von CIA 2): Bank 0 = %11, Bank 3 = %00, deshalb 3 - (addr >> 14). Die Bits 0-1 von $DD02 (Datenrichtung) müssen Ausgang sein.
;   Die übrigen Bits von $DD00 (u. a. serieller Bus) bleiben erhalten. Adressen danach (Bildschirm, Zeichensatz) gelten bankrelativ.
.macro gfx_vic_bank(addr)           ; VIC-Bank wählen: addr = $0000, $4000, $8000 oder $c000
        lda $dd02
        ora #3
        sta $dd02
        lda $dd00
        and #%11111100
        ora #3 - (addr >> 14)
        sta $dd00
.endmacro

; --- Bildschirm ---------------------------------------------------------------------------------------------------
.macro gfx_load_screen_chars(chars) ; 1000 Zeichencodes nach gfx_screen
        gfx_copy chars, gfx_screen, 1000
.endmacro

.macro gfx_load_screen(chars, attrs) ; Zeichencodes nach gfx_screen, Farben ins Farb-RAM
        gfx_copy chars, gfx_screen, 1000
        gfx_copy attrs, gfx_colorram, 1000
.endmacro

; gfx_colors(border, bg) – Farbnummern 0-15 als Konstanten (Rahmen $D020, Hintergrundfarbe 0 $D021).
.macro gfx_colors(border, bg)       ; Rahmen- und Hintergrundfarbe (z. B. aus <name>_border/_bg des Exports)
        lda #border
        sta $d020
        lda #bg
        sta $d021
.endmacro

; gfx_clear(char, color) – füllt alle 1000 Zellen von gfx_screen mit dem Bildschirmcode `char` und das Farb-RAM mit `color`.
;   Eine einzige Schleife mit X = 0..255 beschreibt vier überlappende 256-Byte-Fenster ($000, $100, $200, $2E8); das letzte Fenster
;   deckt die Zellen 744-999 ab, so dass die 1000 Zellen ohne zweite Schleife erreicht werden (einige Zellen werden doppelt beschrieben).
.macro gfx_clear(char, color)       ; Bildschirm mit Zeichen und Farbe füllen
        ldx #0
@l:     lda #char
        sta gfx_screen,x
        sta gfx_screen+$100,x
        sta gfx_screen+$200,x
        sta gfx_screen+$2e8,x
        lda #color
        sta gfx_colorram,x
        sta gfx_colorram+$100,x
        sta gfx_colorram+$200,x
        sta gfx_colorram+$2e8,x
        inx
        bne @l
.endmacro

; --- Kachelkarte ---------------------------------------------------------------------------------------------------
; Die Karte zeichnet die Kachel-Bibliothek. Der Mantel bleibt, damit vorhandene Programme gfx_draw_map weiter benutzen können;
; vorher muss tiles_use den Kachelsatz und die Screen-Bank binden. Zeichen und Farb-RAM entstehen wie zuvor ab Bildschirmposition 0/0.
.macro gfx_draw_map(sn=255)         ; Screen sn der Bank zeichnen (ohne Angabe: den zuletzt gewählten)
        screen_draw sn
.endmacro

; --- Laufzeitroutinen (einmal einbinden) -----------------------------------------------------------------------------
; gfx_memcpy – Unterprogramm hinter gfx_copy. Eingabe: gfx_src (Quelle), gfx_dst (Ziel), gfx_len (Länge), alle 16 Bit in der Zeropage.
;   Kopiert zuerst len/256 volle Seiten (Y läuft 0..255, danach rücken die oberen Adressbytes weiter), dann den Rest (len & 255, über X
;   heruntergezählt). Verändert A, X, Y und die Flags sowie gfx_src, gfx_dst (rücken um die kopierten ganzen Seiten vor) und
;   gfx_len+1 (danach 0). Keine Überlappungsbehandlung: Quelle und Ziel dürfen sich nicht überschneiden. Nicht interruptsicher, wenn
;   ein Interrupt dieselben Zeropage-Zellen benutzt.
.macro gfx_runtime
gfx_memcpy:                         ; gfx_src → gfx_dst, gfx_len Bytes
        ldy #0
@page:  lda gfx_len+1
        beq @tail                   ; keine ganze Seite mehr übrig
@l1:    lda (gfx_src),y
        sta (gfx_dst),y
        iny
        bne @l1
        inc gfx_src+1
        inc gfx_dst+1
        dec gfx_len+1
        jmp @page
@tail:  ldx gfx_len                 ; Rest unter 256 Bytes
        beq @done
@l2:    lda (gfx_src),y
        sta (gfx_dst),y
        iny
        dex
        bne @l2
@done:  rts
.endmacro
