; Kachelsatz hg (3x2) und Screen-Bank levels, von Hand im Exportformat des Kachel-Editors (Konzept, Anhang A).
; Erzeugt von tools/gen_tiles_demo.py (die Daten sind frei erfunden, kein Bildinhalt).
; Aufbau des Kachelsatzes hg: sechs Kacheln zu 3 x 2 Zeichen; je Zelle der Kachel eine Tabelle hg_c0..hg_c5 (Index = Kachelnummer,
; Wert = Bildschirmcode des Zeichens; die Tabellen liegen unmittelbar hintereinander), hg_col = Farbe je Kachel (auf dem Atari ohne
; Wirkung), hg_props = Eigenschaftenbyte je Kachel (Bit 0 = P_SOLID, die anderen Bits frei vergeben).  hg_tw/hg_th = Kachelbreite/-hoehe in Zeichen,
; hg_count = Anzahl.  Screen-Bank levels: drei Karten (levels_map0..2) zu 13 x 12 Kacheln (156 Bytes, zeilenweise Kachelnummern),
; levels_screens_lo/_hi = Adresstabellen (Low-/High-Bytes), levels_tiles_w/_h = Kartengroesse, lv1_id..lv3_id = Screen-Nummern 0..2.

hg_c0:  .byte $11,$18,$1f,$26,$2d,$34
hg_c1:  .byte $14,$1b,$22,$29,$30,$37
hg_c2:  .byte $17,$1e,$25,$2c,$33,$3a
hg_c3:  .byte $1a,$21,$28,$2f,$36,$3d
hg_c4:  .byte $1d,$24,$2b,$32,$39,$40
hg_c5:  .byte $20,$27,$2e,$35,$3c,$43
hg_col:  .byte $00,$00,$00,$00,$00,$00
hg_props:  .byte $00,$01,$01,$02,$04,$05
.const hg_tw = 3
.const hg_th = 2
.const hg_count = 6
.const P_SOLID = 1

levels_map0:  .byte $00,$04,$01,$00,$01,$01,$01,$02,$04,$03,$04,$01,$02
              .byte $03,$03,$04,$01,$02,$03,$00,$01,$01,$05,$02,$05,$04
              .byte $04,$04,$03,$04,$05,$00,$04,$05,$00,$04,$03,$01,$04
              .byte $01,$02,$03,$03,$02,$03,$02,$01,$04,$01,$05,$02,$03
              .byte $03,$01,$05,$02,$04,$04,$02,$01,$00,$01,$03,$01,$01
              .byte $00,$04,$04,$01,$04,$02,$05,$00,$00,$04,$05,$01,$02
              .byte $03,$00,$04,$05,$01,$04,$05,$01,$02,$02,$05,$02,$02
              .byte $00,$03,$00,$02,$05,$04,$02,$03,$02,$04,$01,$02,$00
              .byte $02,$01,$03,$00,$04,$00,$00,$05,$03,$03,$03,$05,$05
              .byte $00,$04,$02,$02,$04,$04,$05,$00,$02,$04,$02,$01,$04
              .byte $01,$02,$02,$02,$01,$00,$01,$04,$05,$02,$04,$02,$02
              .byte $00,$02,$01,$05,$04,$03,$00,$02,$00,$00,$03,$03,$03
levels_map1:  .byte $03,$04,$02,$03,$04,$02,$01,$00,$05,$00,$03,$01,$01
              .byte $04,$05,$01,$01,$04,$05,$01,$00,$00,$03,$00,$00,$04
              .byte $05,$03,$04,$02,$05,$02,$05,$05,$01,$02,$02,$05,$04
              .byte $00,$02,$02,$01,$00,$00,$01,$03,$01,$02,$00,$02,$03
              .byte $03,$01,$00,$05,$00,$02,$03,$03,$04,$01,$02,$00,$00
              .byte $02,$00,$04,$00,$03,$05,$04,$01,$03,$02,$05,$01,$05
              .byte $00,$02,$03,$01,$04,$04,$01,$05,$05,$00,$01,$05,$05
              .byte $03,$00,$03,$00,$05,$04,$05,$01,$04,$02,$01,$04,$03
              .byte $00,$00,$02,$03,$02,$05,$05,$02,$00,$01,$05,$05,$05
              .byte $05,$02,$05,$04,$01,$00,$01,$02,$00,$00,$05,$00,$00
              .byte $04,$04,$00,$05,$01,$03,$04,$02,$04,$03,$00,$00,$00
              .byte $01,$02,$00,$00,$02,$01,$02,$00,$01,$05,$03,$04,$02
levels_map2:  .byte $02,$03,$01,$05,$03,$00,$03,$02,$02,$04,$05,$04,$03
              .byte $04,$03,$05,$05,$00,$00,$03,$00,$00,$00,$00,$00,$05
              .byte $04,$02,$04,$04,$00,$00,$05,$04,$05,$00,$04,$05,$02
              .byte $04,$02,$01,$01,$00,$02,$00,$03,$03,$05,$01,$00,$01
              .byte $04,$05,$01,$00,$04,$02,$04,$02,$00,$03,$03,$01,$04
              .byte $00,$00,$04,$01,$02,$01,$04,$05,$05,$00,$03,$00,$01
              .byte $00,$05,$01,$00,$04,$05,$00,$02,$00,$04,$05,$04,$03
              .byte $05,$01,$05,$05,$02,$01,$02,$04,$04,$00,$02,$05,$04
              .byte $04,$01,$03,$03,$00,$01,$01,$03,$01,$02,$03,$03,$05
              .byte $01,$04,$01,$04,$04,$04,$01,$00,$03,$02,$03,$04,$03
              .byte $00,$00,$01,$02,$01,$04,$01,$01,$02,$00,$00,$04,$01
              .byte $00,$05,$03,$04,$01,$05,$01,$03,$03,$01,$01,$05,$01
levels_screens: .lohifill 3, levels_map0 + 156 * i
levels_screens_lo = levels_screens
levels_screens_hi = levels_screens + 3
.const levels_count = 3
.const levels_tiles_w = 13
.const levels_tiles_h = 12
.const lv1_id = 0
.const lv2_id = 1
.const lv3_id = 2
