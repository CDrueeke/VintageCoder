; Zeigt die vier Player des Raumschiffs (graphics/ship.vcpmg) auf dem Atari 800XL nebeneinander.
; Der Export (build/export/ship.asm) entsteht beim Bauen von selbst.
.target atari8
.cpu sally { illegals = deny }

.const SDMCTL = $022f           ; Schattenregister von DMACTL (das Betriebssystem kopiert es jedes Bild)

.org $2000
start:  lda #$40
        sta antic.pmbase        ; Player/Missile-Seite = $4000 (Spieler 0 liegt bei $4400)
        lda #3
        sta gtia.gractl         ; Player und Missiles einschalten
        lda #$3a                ; Anzeigeliste an, Player-DMA an, Einzelzeilen-Auflösung, normales Feld
        sta SDMCTL
        lda #ship_p0
        sta gtia.colpm0
        lda #ship_p1
        sta gtia.colpm1
        lda #ship_p2
        sta gtia.colpm2
        lda #ship_p3
        sta gtia.colpm3
        lda #100                ; vier Player nebeneinander, je 8 Pixel breit
        sta gtia.hposp0
        lda #108
        sta gtia.hposp1
        lda #116
        sta gtia.hposp2
        lda #124
        sta gtia.hposp3
done:   jmp done

.org $4400                      ; Player 0; Player 1 bis 3 folgen im Abstand von $100
.include "build/export/ship.asm"
.run start
