; data_3x2.inc – Kachelsatz hg (3x2) und Screen-Bank levels für das Beispiel tiles_demo, im Exportformat des Kachel-Editors
; (Konzept, Anhang A). Erzeugt von tools/gen_tiles_demo.py (die Daten sind frei erfunden, kein Bildinhalt): nicht von Hand ändern, eine
; Änderung geht beim nächsten Lauf des Skripts verloren.
;
; Kachelsatz hg: 6 Kacheln zu 6 Zellen, planar abgelegt – eine Tabelle je Zelle (hg_c0 bis hg_c5), Index = Kachelnummer 0-5; der Wert ist
; der Bildschirmcode der Zelle. hg_col = Farbe je Kachel (Farb-RAM-Wert, Bit 3 = Mehrfarbe: Kachel 3 hat $0F), hg_props = Eigenschaften-
; byte je Kachel (Bit 0 = P_SOLID; die übrigen Bits sind in diesem Beispiel nur Prüfwerte für tile_prop).
; Konstanten: hg_tw/hg_th = Kachelgröße in Zellen, hg_count = Anzahl der Kacheln.
;
; Screen-Bank levels: drei Karten (levels_map0..2) zu 12 x 10 Kacheln, zeilenweise, je Eintrag eine Kachelnummer 0-5. levels_screens ist
; eine lo/hi-Adresstabelle der Karten (Direktive .lohifill): levels_screens_lo = erste Tabelle, levels_screens_hi = dahinter. Konstanten:
; levels_count (Anzahl der Screens), levels_tiles_w/_h (Kartengröße in Kacheln), lv1_id..lv3_id (Screen-Nummern 0-2).

hg_c0:  .byte $11,$18,$1f,$26,$2d,$34
hg_c1:  .byte $14,$1b,$22,$29,$30,$37
hg_c2:  .byte $17,$1e,$25,$2c,$33,$3a
hg_c3:  .byte $1a,$21,$28,$2f,$36,$3d
hg_c4:  .byte $1d,$24,$2b,$32,$39,$40
hg_c5:  .byte $20,$27,$2e,$35,$3c,$43
hg_col:  .byte $01,$02,$05,$0f,$0d,$0e
hg_props:  .byte $00,$01,$01,$02,$04,$05
.const hg_tw = 3
.const hg_th = 2
.const hg_count = 6
.const P_SOLID = 1

levels_map0:  .byte $00,$04,$01,$00,$01,$01,$01,$02,$04,$03,$04,$01
              .byte $02,$03,$03,$04,$01,$02,$03,$00,$01,$01,$05,$02
              .byte $05,$04,$04,$04,$03,$04,$05,$00,$04,$05,$00,$04
              .byte $03,$01,$04,$01,$02,$03,$03,$02,$03,$02,$01,$04
              .byte $01,$05,$02,$03,$03,$01,$05,$02,$04,$04,$02,$01
              .byte $00,$01,$03,$01,$01,$00,$04,$04,$01,$04,$02,$05
              .byte $00,$00,$04,$05,$01,$02,$03,$00,$04,$05,$01,$04
              .byte $05,$01,$02,$02,$05,$02,$02,$00,$03,$00,$02,$05
              .byte $04,$02,$03,$02,$04,$01,$02,$00,$02,$01,$03,$00
              .byte $04,$00,$00,$05,$03,$03,$03,$05,$05,$00,$04,$02
levels_map1:  .byte $02,$04,$04,$05,$00,$02,$04,$02,$01,$04,$01,$02
              .byte $02,$02,$01,$00,$01,$04,$05,$02,$04,$02,$02,$00
              .byte $02,$01,$05,$04,$03,$00,$02,$00,$00,$03,$03,$03
              .byte $03,$04,$02,$03,$04,$02,$01,$00,$05,$00,$03,$01
              .byte $01,$04,$05,$01,$01,$04,$05,$01,$00,$00,$03,$00
              .byte $00,$04,$05,$03,$04,$02,$05,$02,$05,$05,$01,$02
              .byte $02,$05,$04,$00,$02,$02,$01,$00,$00,$01,$03,$01
              .byte $02,$00,$02,$03,$03,$01,$00,$05,$00,$02,$03,$03
              .byte $04,$01,$02,$00,$00,$02,$00,$04,$00,$03,$05,$04
              .byte $01,$03,$02,$05,$01,$05,$00,$02,$03,$01,$04,$04
levels_map2:  .byte $01,$05,$05,$00,$01,$05,$05,$03,$00,$03,$00,$05
              .byte $04,$05,$01,$04,$02,$01,$04,$03,$00,$00,$02,$03
              .byte $02,$05,$05,$02,$00,$01,$05,$05,$05,$05,$02,$05
              .byte $04,$01,$00,$01,$02,$00,$00,$05,$00,$00,$04,$04
              .byte $00,$05,$01,$03,$04,$02,$04,$03,$00,$00,$00,$01
              .byte $02,$00,$00,$02,$01,$02,$00,$01,$05,$03,$04,$02
              .byte $02,$03,$01,$05,$03,$00,$03,$02,$02,$04,$05,$04
              .byte $03,$04,$03,$05,$05,$00,$00,$03,$00,$00,$00,$00
              .byte $00,$05,$04,$02,$04,$04,$00,$00,$05,$04,$05,$00
              .byte $04,$05,$02,$04,$02,$01,$01,$00,$02,$00,$03,$03
levels_screens: .lohifill 3, levels_map0 + 120 * i
levels_screens_lo = levels_screens
levels_screens_hi = levels_screens + 3
.const levels_count = 3
.const levels_tiles_w = 12
.const levels_tiles_h = 10
.const lv1_id = 0
.const lv2_id = 1
.const lv3_id = 2
