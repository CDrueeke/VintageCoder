; =====================================================================================================================
; text_demo.asm - Text direkt in den Textbildschirm schreiben (GRAPHICS 0) und Farben über OS-Schattenregister setzen
; =====================================================================================================================
;
; ZWECK
;   Einfachster Textausgabe-Test ohne CIO: Das Betriebssystem hat beim Start den Textbildschirm (BASIC-Modus 0, ANTIC-Modus 2,
;   40 x 24 Zeichen) eingerichtet; das Programm schreibt 28 Zeichencodes direkt in den Bildschirmspeicher und stellt Farben ein.
;   Gedacht als einfacher Test für die Momentaufnahme im Debugger (Speicherinhalt des Bildschirms ist leicht zu erkennen).
;
; WAS ZU SEHEN IST
;   Oben links in der ersten Textzeile der Text "HALLO ATARI - MOMENTAUFNAHME" (28 Zeichen), helle Schrift (Helligkeit 14) auf
;   Hintergrund Farbton 9/Helligkeit 4 mit Rand Farbton 2/Helligkeit 2.  Die Farben stehen in OS-Schattenregistern; das
;   Betriebssystem kopiert sie im nächsten Vertical Blank in die Hardware (GTIA COLPF1/COLPF2/COLBK), sie wirken also
;   erst im nächsten Bild.  Das OS bleibt aktiv (Attract-Modus kann die Farben nach einiger Zeit ohne Tastendruck ändern).
;
; BILDSCHIRMCODES
;   Im Textmodus 2 steht in jeder Zelle ein Bildschirmcode (nicht ATASCII): $00 = Leerzeichen, $0D = "-", $10-$19 = Ziffern,
;   $21-$3A = Buchstaben A-Z (ATASCII $41-$5A minus $20).  `msg` enthält:
;   $28 $21 $2C $2C $2F = HALLO, $00, $21 $34 $21 $32 $29 = ATARI, $00, $0D = -, $00, danach MOMENTAUFNAHME.
;
; SPEICHER:  Programm ab $2000; Text `msg` (28 Bytes) direkt dahinter.  Nullseite: SAVMSC $58/$59 (vom OS gesetzt, nur gelesen).
; OS-VARIABLEN (Konstanten unten): SAVMSC = Zeiger auf den Bildschirmspeicher, COLOR1/2/4 = Schatten von COLPF1/COLPF2/COLBK.
; BAUEN:     vcc asm examples/atari/text_demo/text_demo.asm --target atari8 --cpu sally -o text_demo.xex
; GRENZEN:   Annahme: Der OS-Textschirm GRAPHICS 0 ist aktiv (Standard nach dem Systemstart).
;            Keine Zeilenumbrüche, nur Zeile 1.  Keine Rückkehr ins OS (Endlosschleife).
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }

.const SAVMSC = $58             ; Zeiger auf den Bildschirmspeicher (2 Byte, Low/High; vom OS gesetzt)
.const COLOR1 = $02c5           ; Schriftleuchtdichte (Schatten von COLPF1; im Modus 2 zählt nur die Helligkeit)
.const COLOR2 = $02c6           ; Hintergrund (Schatten von COLPF2)
.const COLOR4 = $02c8           ; Rahmen (Schatten von COLBK)

.org $2000
start:  lda #$0e                ; Farbwert $0E = Farbton 0, Helligkeit 14 (hell)
        sta COLOR1
        lda #$94                ; Farbton 9, Helligkeit 4
        sta COLOR2
        lda #$22                ; Farbton 2, Helligkeit 2
        sta COLOR4
        ldy #0
.copy:  lda msg,y               ; Bildschirmcode holen ...
        sta (SAVMSC),y          ; ... und in die erste Textzeile schreiben (Y = Spalte)
        iny
        cpy #28                 ; Länge des Textes
        bne .copy
done:   jmp done

; Text als Bildschirmcodes (Länge 28), siehe Kopf
msg:    .byte $28,$21,$2c,$2c,$2f,$00,$21,$34,$21,$32,$29,$00,$0d,$00,$2d,$2f,$2d,$25,$2e,$34,$21,$35,$26,$2e,$21,$28,$2d,$25
.run start
