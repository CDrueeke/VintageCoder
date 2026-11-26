; Zeigt die vier Player des Raumschiffs (graphics/ship.vcpmg) auf dem Atari 800XL nebeneinander.
; Die Daten entstehen beim Einbinden von graphics/ship.vcpmg von selbst.
; Die Farben gehen in die Schattenregister (PCOLR0 bis PCOLR3): das Betriebssystem überschreibt COLPM0 bis COLPM3 sonst in jedem Bild.
; Zeile 0 eines Players liegt oberhalb des sichtbaren Bildes (sichtbar ab etwa Zeile 32), darum steht das Raumschiff im Bild ab Zeile 48.
.target atari8
.cpu sally { illegals = deny }

.const SDMCTL = $022f           ; Schattenregister von DMACTL (das Betriebssystem kopiert es jedes Bild)
.const GPRIOR = $026f           ; Schattenregister von PRIOR
.const PCOLR0 = $02c0           ; Schattenregister der Spielerfarben COLPM0 bis COLPM3 (vier Bytes)

.org $2000
start:  lda #$40
        sta antic.pmbase        ; Player/Missile-Seite = $4000 (Spieler 0 liegt bei $4400)
        lda #3
        sta gtia.gractl         ; Player und Missiles einschalten
        lda #ship_prior         ; Priorität und Breite kommen aus dem Export (Einstellungen im Sprite-Editor)
        sta GPRIOR
        lda #ship_sizep0
        sta gtia.sizep0
        lda #ship_sizep1
        sta gtia.sizep1
        lda #ship_sizep2
        sta gtia.sizep2
        lda #ship_sizep3
        sta gtia.sizep3
        lda #$3a                ; Anzeigeliste an, Player-DMA an, Einzelzeilen-Auflösung, normales Feld
        sta SDMCTL
        lda #ship_p0
        sta PCOLR0+0
        lda #ship_p1
        sta PCOLR0+1
        lda #ship_p2
        sta PCOLR0+2
        lda #ship_p3
        sta PCOLR0+3
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
.include "graphics/ship.vcpmg"
.run start
