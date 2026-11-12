; =====================================================================================================================
; regs_test.asm - Selbsttest der generierten Atari-Registernamen
; =====================================================================================================================
;
; ZWECK
;   Prüft, dass die aus data/chips/*.chip.json erzeugten Registernamen (antic.*, gtia.*, pokey.*) des Targets atari8 bei
;   der Übersetzung die richtigen Adressen ergeben.  Das Programm schreibt nacheinander bekannte Testwerte in je ein
;   Register von ANTIC, GTIA und POKEY und bleibt dann in einer Endlosschleife; im Emulator-Monitor
;   lassen sich die Werte in den Hardwareregistern ablesen (die meisten Register sind schreibgeschützt, siehe Hinweis).
;   (Die Datei lib/atari.inc daneben ist die generierte Konstantenliste, die dieses Programm nicht einbindet: die
;   Namen kommen aus .target atari8.)
;
; WAS GESCHRIEBEN WIRD (Wert -> Register, Adresse)
;   $00 -> ANTIC NMIEN  $D40E  (VBI/DLI aus, damit das OS die Farbregister nicht überschreibt)
;   $5A -> GTIA COLBK   $D01A  (Hintergrundfarbe: Farbton 5, Helligkeit 10)
;   $C4 -> GTIA COLPF2  $D018  (Playfield-Farbe 2: Farbton 12, Helligkeit 4)
;   $33 -> GTIA COLPM0  $D012  (Farbe Spieler/Missile 0)
;   $11 -> POKEY AUDF1  $D200  (Frequenzteiler Kanal 1)
;   $A8 -> POKEY AUDC1  $D201  (Bits 7-5 = 101 Verzerrung "reiner Ton", Bit 4 = 0, Bits 3-0 = Lautstärke 8)
;   $4F -> POKEY AUDCTL $D208  (Bits 6, 3, 2, 1, 0: Kanal 1 mit 1,79 MHz, Kanäle 3+4 als 16 Bit, Hochpass Kanal 1 und 2,
;                               15-kHz-Grundtakt)
;   Hinweis: Die Schreibregister D200/D201/D208 sind Schreib-Seiten, die Leseseiten derselben Adressen sind POT0/POT1/ALLPOT.
;   Die Werte sind absichtlich unterscheidbar gewählt, nicht musikalisch gemeint; AUDF1/AUDC1 geben (je nach Takt) einen Ton aus.
;
; SPEICHER: Programm ab $2000, keine Daten, keine Nullseite.
; BAUEN:    vcc asm examples/atari/regs_test/regs_test.asm --target atari8 --cpu sally -o regs_test.xex
; =====================================================================================================================
.target atari8
.cpu sally { illegals = deny }

.org $2000
start:  lda #0
        sta antic.nmien         ; NMI aus: das OS kopiert die Schattenregister nicht mehr in die Hardware
        lda #$5A
        sta gtia.colbk
        lda #$C4
        sta gtia.colpf2
        lda #$33
        sta gtia.colpm0
        lda #$11
        sta pokey.audf1
        lda #$A8
        sta pokey.audc1
        lda #$4F
        sta pokey.audctl
done:   jmp done                ; Endlosschleife: der Zustand bleibt für den Monitor stehen
.run start
