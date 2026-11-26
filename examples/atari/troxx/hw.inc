; =====================================================================================================================
; hw.inc - Atari 8 Bit (XL/XE): Registeradressen und Betriebssystem-Schatten, die Troxx benutzt
; =====================================================================================================================
; Eigene Namensliste (wie examples/atari/prallball), damit das Projekt in sich geschlossen ist.  Quelle der Adressen: Hardware-Handbuch
;   (ANTIC $D400, GTIA $D000, POKEY $D200, PIA $D300) und Betriebssystem-Variablen (Seite 2).
; GTIA schreibend: HPOSP0..3 $D000-$D003, HPOSM0..3 $D004-$D007, SIZEP0..3 $D008-$D00B, SIZEM $D00C, COLPM0..3 $D012-$D015, COLPF0..3 $D016-$D019,
;   COLBK $D01A, PRIOR $D01B, GRACTL $D01D.  Lesend: TRIG0 $D010 (Feuer Anschluss 1), TRIG1 $D011 (Anschluss 2), PAL $D014, CONSOL $D01F.
; =====================================================================================================================
.const HPOSP0   = $d000
.const HPOSM0   = $d004
.const SIZEP0   = $d008
.const SIZEM    = $d00c
.const COLPM0   = $d012
.const COLPF0   = $d016
.const COLPF1   = $d017
.const COLPF2   = $d018
.const COLPF3   = $d019
.const COLBK    = $d01a
.const PRIOR    = $d01b
.const GRACTL   = $d01d
.const CONSOL   = $d01f
.const TRIG0    = $d010
.const TRIG1    = $d011
.const GTIA_PAL = $d014
.const AUDF1    = $d200
.const AUDC1    = $d201
.const AUDF2    = $d202
.const AUDC2    = $d203
.const AUDF3    = $d204
.const AUDC3    = $d205
.const AUDF4    = $d206
.const AUDC4    = $d207
.const AUDCTL   = $d208
.const SKCTL    = $d20f
.const PORTA    = $d300
.const PACTL    = $d302
.const DMACTL   = $d400
.const HSCROL   = $d404
.const PMBASE   = $d407
.const CHBASE   = $d409
.const WSYNC    = $d40a
.const VCOUNT   = $d40b
.const NMIEN    = $d40e
; Betriebssystem
.const RTCLOK   = $14                   ; niedrigstes Byte der Systemuhr: zaehlt im Bildruecklauf (50 / 60 Hz)
.const ATRACT   = $4d                   ; Abdunklungszaehler des Betriebssystems (jedes Bild auf 0 setzen)
.const VDSLST   = $0200                 ; Vektor des Display-List-Interrupts
.const VVBLKD   = $0224                 ; Vektor des verzoegerten Bildruecklauf-Interrupts
.const SDMCTL   = $022f                 ; Schatten von DMACTL
.const SDLSTL   = $0230                 ; Schatten der Display-List-Adresse (2 Byte)
.const GPRIOR   = $026f                 ; Schatten von PRIOR
.const PCOLR0   = $02c0                 ; Schatten von COLPM0..3
.const COLOR0   = $02c4                 ; Schatten von COLPF0..3 und COLBK ($02C8)
.const CHBAS    = $02f4                 ; Schatten von CHBASE
.const XITVBV   = $e462                 ; Ende eines verzoegerten Bildruecklauf-Interrupts
.const CIOV     = $e456
