; Kachelsatz hg (2x2) und Screen-Bank levels, von Hand im Exportformat des Kachel-Editors (Konzept, Anhang A).
; Erzeugt von tools/gen_tiles_demo.py (die Daten sind frei erfunden, kein Bildinhalt).
; Eingebunden von demo.inc (vor dem Programm).  Das Format ist das, was die Kachel-Bibliothek (data/sys/common/tiles.asm) erwartet:
;   hg_c0 .. hg_c3   eine Tabelle je Zeichenzelle der Kachel (planar): hg_cK[n] = Bildschirmcode der Zelle K von Kachel n (n = 0..5);
;                     die Zellen zaehlen zeilenweise (Zelle 0 = oben links); alle Tabellen stehen unmittelbar hintereinander
;   hg_col            Farbe je Kachel (Farb-RAM-Wert, Bits 0..3; Bit 3 = Mehrfarbe)
;   hg_props          Eigenschaftenbyte je Kachel (frei verwendbar, z. B. fuer Kollisionen; P_SOLID = 1 ist ein Beispielwert fuer Bit 0)
;   hg_tw, hg_th      Kachelgroesse in Zeichenzellen (Breite, Hoehe), hg_count = Zahl der Kacheln
;   levels_map0..2    drei Karten (Screens) mit Kachelnummern, zeilenweise, je 15 Kacheln breit und 10 hoch (15 Kacheln = 30 Zellen breit)
;   levels_screens    Adresstabelle der Karten (lo-Tabelle, dann hi-Tabelle), levels_screens_lo/_hi zeigen auf deren Haelften

hg_c0:  .byte $11,$18,$1f,$26,$2d,$34
hg_c1:  .byte $14,$1b,$22,$29,$30,$37
hg_c2:  .byte $17,$1e,$25,$2c,$33,$3a
hg_c3:  .byte $1a,$21,$28,$2f,$36,$3d
hg_col:  .byte $01,$02,$05,$0f,$0d,$0e
hg_props:  .byte $00,$01,$01,$02,$04,$05
.const hg_tw = 2                        ; Kachelbreite in Zeichenzellen
.const hg_th = 2                        ; Kachelhoehe in Zeichenzellen
.const hg_count = 6                     ; Zahl der Kacheln im Satz
.const P_SOLID = 1                      ; Beispielbit im Eigenschaftenbyte (wird von der Bibliothek nicht ausgewertet)

levels_map0:  .byte $00,$04,$01,$00,$01,$01,$01,$02,$04,$03,$04,$01,$02,$03,$03
              .byte $04,$01,$02,$03,$00,$01,$01,$05,$02,$05,$04,$04,$04,$03,$04
              .byte $05,$00,$04,$05,$00,$04,$03,$01,$04,$01,$02,$03,$03,$02,$03
              .byte $02,$01,$04,$01,$05,$02,$03,$03,$01,$05,$02,$04,$04,$02,$01
              .byte $00,$01,$03,$01,$01,$00,$04,$04,$01,$04,$02,$05,$00,$00,$04
              .byte $05,$01,$02,$03,$00,$04,$05,$01,$04,$05,$01,$02,$02,$05,$02
              .byte $02,$00,$03,$00,$02,$05,$04,$02,$03,$02,$04,$01,$02,$00,$02
              .byte $01,$03,$00,$04,$00,$00,$05,$03,$03,$03,$05,$05,$00,$04,$02
              .byte $02,$04,$04,$05,$00,$02,$04,$02,$01,$04,$01,$02,$02,$02,$01
              .byte $00,$01,$04,$05,$02,$04,$02,$02,$00,$02,$01,$05,$04,$03,$00
levels_map1:  .byte $02,$00,$00,$03,$03,$03,$03,$04,$02,$03,$04,$02,$01,$00,$05
              .byte $00,$03,$01,$01,$04,$05,$01,$01,$04,$05,$01,$00,$00,$03,$00
              .byte $00,$04,$05,$03,$04,$02,$05,$02,$05,$05,$01,$02,$02,$05,$04
              .byte $00,$02,$02,$01,$00,$00,$01,$03,$01,$02,$00,$02,$03,$03,$01
              .byte $00,$05,$00,$02,$03,$03,$04,$01,$02,$00,$00,$02,$00,$04,$00
              .byte $03,$05,$04,$01,$03,$02,$05,$01,$05,$00,$02,$03,$01,$04,$04
              .byte $01,$05,$05,$00,$01,$05,$05,$03,$00,$03,$00,$05,$04,$05,$01
              .byte $04,$02,$01,$04,$03,$00,$00,$02,$03,$02,$05,$05,$02,$00,$01
              .byte $05,$05,$05,$05,$02,$05,$04,$01,$00,$01,$02,$00,$00,$05,$00
              .byte $00,$04,$04,$00,$05,$01,$03,$04,$02,$04,$03,$00,$00,$00,$01
levels_map2:  .byte $02,$00,$00,$02,$01,$02,$00,$01,$05,$03,$04,$02,$02,$03,$01
              .byte $05,$03,$00,$03,$02,$02,$04,$05,$04,$03,$04,$03,$05,$05,$00
              .byte $00,$03,$00,$00,$00,$00,$00,$05,$04,$02,$04,$04,$00,$00,$05
              .byte $04,$05,$00,$04,$05,$02,$04,$02,$01,$01,$00,$02,$00,$03,$03
              .byte $05,$01,$00,$01,$04,$05,$01,$00,$04,$02,$04,$02,$00,$03,$03
              .byte $01,$04,$00,$00,$04,$01,$02,$01,$04,$05,$05,$00,$03,$00,$01
              .byte $00,$05,$01,$00,$04,$05,$00,$02,$00,$04,$05,$04,$03,$05,$01
              .byte $05,$05,$02,$01,$02,$04,$04,$00,$02,$05,$04,$04,$01,$03,$03
              .byte $00,$01,$01,$03,$01,$02,$03,$03,$05,$01,$04,$01,$04,$04,$04
              .byte $01,$00,$03,$02,$03,$04,$03,$00,$00,$01,$02,$01,$04,$01,$01
levels_screens: .lohifill 3, levels_map0 + 150 * i
levels_screens_lo = levels_screens      ; niedrige Bytes der Kartenadressen (Eintrag n = Screen n)
levels_screens_hi = levels_screens + 3  ; hohe Bytes (hinter den drei niedrigen)
.const levels_count = 3                 ; Zahl der Screens der Bank
.const levels_tiles_w = 15              ; Kartenbreite in Kacheln (fuer alle Screens gleich)
.const levels_tiles_h = 10              ; Kartenhoehe in Kacheln
.const lv1_id = 0                       ; Screen-Nummern fuer screen_draw / screen_select (Namen aus dem Export)
.const lv2_id = 1
.const lv3_id = 2
