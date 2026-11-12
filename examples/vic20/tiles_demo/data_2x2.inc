; Kachelsatz hg (2x2) und Screen-Bank levels, von Hand im Exportformat des Kachel-Editors (Konzept, Anhang A).
; Erzeugt von tools/gen_tiles_demo.py (die Daten sind frei erfunden, kein Bildinhalt).
;
; hg_c0 .. hg_c3: planare Zellentabellen des Kachelsatzes: hg_cK[n] = Zeichencode (Bildschirmcode) der Zelle K von Kachel n (n = 0..5);
;   Zelle K liegt in Zeile K / 2 und Spalte K mod 2 der Kachel (2 Zeichen breit, 2 hoch).
; hg_col: Farb-RAM-Wert je Kachel (Bit 0..2 Farbe, Bit 3 = Mehrfarbzeichen). Die Tabelle ist die des C64-Beispiels; auf dem VIC-I setzen die Werte $0f, $0d, $0e
;   der Kacheln 3..5 darum das Mehrfarbbit.
; hg_props: Eigenschaftenbyte je Kachel (frei erfundene Testwerte; Bit 0 = P_SOLID).  hg_tw, hg_th: Kachelgroesse in Zeichen, hg_count: Anzahl Kacheln.
; levels_map0 .. levels_map2: drei Karten zu 11 x 11 Kacheln (121 Byte, zeilenweise, je Byte eine Kachelnummer); sie fuellen 22 von 22 Spalten und 22 von 23 Zeilen.
; levels_screens: .lohifill legt erst die 3 Low-Bytes, dann die 3 High-Bytes der Kartenadressen ab; levels_screens_lo/_hi zeigen auf die beiden Haelften.
; levels_count, levels_tiles_w/_h: Zahl und Groesse der Karten in Kacheln; lv1_id .. lv3_id: Screen-Nummern (0..2) fuer screen_select/screen_draw.

hg_c0:  .byte $11,$18,$1f,$26,$2d,$34
hg_c1:  .byte $14,$1b,$22,$29,$30,$37
hg_c2:  .byte $17,$1e,$25,$2c,$33,$3a
hg_c3:  .byte $1a,$21,$28,$2f,$36,$3d
hg_col:  .byte $01,$02,$05,$0f,$0d,$0e
hg_props:  .byte $00,$01,$01,$02,$04,$05
.const hg_tw = 2
.const hg_th = 2
.const hg_count = 6
.const P_SOLID = 1

levels_map0:  .byte $00,$04,$01,$00,$01,$01,$01,$02,$04,$03,$04
              .byte $01,$02,$03,$03,$04,$01,$02,$03,$00,$01,$01
              .byte $05,$02,$05,$04,$04,$04,$03,$04,$05,$00,$04
              .byte $05,$00,$04,$03,$01,$04,$01,$02,$03,$03,$02
              .byte $03,$02,$01,$04,$01,$05,$02,$03,$03,$01,$05
              .byte $02,$04,$04,$02,$01,$00,$01,$03,$01,$01,$00
              .byte $04,$04,$01,$04,$02,$05,$00,$00,$04,$05,$01
              .byte $02,$03,$00,$04,$05,$01,$04,$05,$01,$02,$02
              .byte $05,$02,$02,$00,$03,$00,$02,$05,$04,$02,$03
              .byte $02,$04,$01,$02,$00,$02,$01,$03,$00,$04,$00
              .byte $00,$05,$03,$03,$03,$05,$05,$00,$04,$02,$02
levels_map1:  .byte $04,$04,$05,$00,$02,$04,$02,$01,$04,$01,$02
              .byte $02,$02,$01,$00,$01,$04,$05,$02,$04,$02,$02
              .byte $00,$02,$01,$05,$04,$03,$00,$02,$00,$00,$03
              .byte $03,$03,$03,$04,$02,$03,$04,$02,$01,$00,$05
              .byte $00,$03,$01,$01,$04,$05,$01,$01,$04,$05,$01
              .byte $00,$00,$03,$00,$00,$04,$05,$03,$04,$02,$05
              .byte $02,$05,$05,$01,$02,$02,$05,$04,$00,$02,$02
              .byte $01,$00,$00,$01,$03,$01,$02,$00,$02,$03,$03
              .byte $01,$00,$05,$00,$02,$03,$03,$04,$01,$02,$00
              .byte $00,$02,$00,$04,$00,$03,$05,$04,$01,$03,$02
              .byte $05,$01,$05,$00,$02,$03,$01,$04,$04,$01,$05
levels_map2:  .byte $05,$00,$01,$05,$05,$03,$00,$03,$00,$05,$04
              .byte $05,$01,$04,$02,$01,$04,$03,$00,$00,$02,$03
              .byte $02,$05,$05,$02,$00,$01,$05,$05,$05,$05,$02
              .byte $05,$04,$01,$00,$01,$02,$00,$00,$05,$00,$00
              .byte $04,$04,$00,$05,$01,$03,$04,$02,$04,$03,$00
              .byte $00,$00,$01,$02,$00,$00,$02,$01,$02,$00,$01
              .byte $05,$03,$04,$02,$02,$03,$01,$05,$03,$00,$03
              .byte $02,$02,$04,$05,$04,$03,$04,$03,$05,$05,$00
              .byte $00,$03,$00,$00,$00,$00,$00,$05,$04,$02,$04
              .byte $04,$00,$00,$05,$04,$05,$00,$04,$05,$02,$04
              .byte $02,$01,$01,$00,$02,$00,$03,$03,$05,$01,$00
levels_screens: .lohifill 3, levels_map0 + 121 * i
levels_screens_lo = levels_screens
levels_screens_hi = levels_screens + 3
.const levels_count = 3
.const levels_tiles_w = 11
.const levels_tiles_h = 11
.const lv1_id = 0
.const lv2_id = 1
.const lv3_id = 2
