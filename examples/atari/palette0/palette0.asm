; =====================================================================================================================
; palette0.asm - Farbtafel-Messprogramm, Seite 0: Farbwerte 0..126 (je 3 Scanlines)
; =====================================================================================================================
;
; ZWECK
;   Zeigt die Farben des Atari (GTIA) als waagerechte Balken zum Ausmessen/Vergleichen (z. B. mit Emulator-Screenshots).
;   Seite 0 (diese Datei) zeigt die Farbwerte 0..126, Seite 1 (palette1.asm) die Werte 128..254; beide Dateien sind
;   bis auf den Zählbereich von Y (0..63 bzw. 64..127) identisch.  Es werden nur gerade Farbwerte gezeigt: Bit 0 eines
;   Farbregisters ist ohne Wirkung, die Helligkeit steckt in den Bits 3-1 (8 Stufen), der Farbton in den Bits 7-4
;   (16 Töne; diese Seite: Töne 0-7).  Jeder Balken ist 3 Rasterzeilen hoch, 64 Balken = 192 Zeilen.
;
; WAS ZU SEHEN IST (von oben nach unten, PAL)
;   schwarz | 3 weiße Zeilen (Farbwert $0E = Marke) | 64 Balken Farbwert 2*Y | 3 weiße Zeilen | 1 schwarze Zeile | Rest schwarz
;   Die weißen Marken begrenzen die Tafel; die Tafel beginnt bei VCOUNT $10 (Rasterzeile 32).
;
; WAS DER CODE TUT
;   * NMIEN = 0 (kein VBI/DLI, das OS schreibt die Schattenregister nicht mehr in die Hardware) und DMACTL = 0 (Bildschirm-DMA
;     aus, ANTIC liefert keine Playfield-Daten): damit ist nur noch die Hintergrundfarbe COLBK zu sehen.  SEI ist zusätzlich gesetzt
;     (IRQs aus); das Programm kehrt nie zum OS zurück.
;   * Pro Bild wartet die Schleife `frame` per Abfrage, bis VCOUNT (Zeile / 2) den Wert $10 hat, und zeichnet dann zeilenweise.
;   * Jede Rasterzeile wird mit STA WSYNC synchronisiert (CPU wartet bis Zeilenende); der Farbwert wird erst nach dem WSYNC
;     in COLBK geschrieben und bleibt dann stehen, bis zwei weitere WSYNC vergangen sind -> 3 Zeilen je Balken.
;
; REGISTER: ANTIC NMIEN $D40E, DMACTL $D400, VCOUNT $D40B, WSYNC $D40A;  GTIA COLBK $D01A.
; SPEICHER: Programm ab $2000, keine Daten, keine Nullseite (Register A, X, Y werden benutzt).
; BAUEN:    vcc asm examples/atari/palette0/palette0.asm --target atari8 --cpu sally -o palette0.xex
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }
.org $2000
start:  sei                     ; IRQs aus (NMIs schaltet NMIEN weiter unten ab)
        lda #0
        sta antic.nmien
        sta antic.dmactl        ; Screen-DMA aus: nur COLBK sichtbar
frame:  lda antic.vcount        ; auf Zeile 32 (VCOUNT = $10) warten: feste Startzeile je Bild
        cmp #$10
        bne frame
        ldx #3                  ; 3 weiße Zeilen als obere Marke
mark1:  lda #$0e
        sta antic.wsync
        sta gtia.colbk
        dex
        bne mark1
        ldy #0                  ; Y = Balkennummer; Farbwert = 2*Y
bars:   tya                     ; A = Balkennummer Y
        asl a                   ; A = 2*Y = Farbwert (Bit 0 ist wirkungslos, daher nur gerade Werte; max. 126 bzw. 254)
        sta antic.wsync
        sta gtia.colbk          ; Farbwert 2*Y, bleibt über die nächsten zwei WSYNC stehen
        sta antic.wsync
        sta antic.wsync
        iny
        cpy #64                 ; Endwert (ausschließlich)
        bne bars
        ldx #3                  ; 3 weiße Zeilen als untere Marke
mark2:  lda #$0e
        sta antic.wsync
        sta gtia.colbk
        dex
        bne mark2
        lda #0                  ; zurück auf Schwarz, bis zum nächsten Bild
        sta antic.wsync
        sta gtia.colbk
        jmp frame
.run start
