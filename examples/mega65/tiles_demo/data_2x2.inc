; Kachelsatz hg (2x2) und Screen-Bank levels, von Hand im Exportformat des Kachel-Editors (Konzept, Anhang A).
; Erzeugt von tools/gen_tiles_demo.py (die Daten sind frei erfunden, kein Bildinhalt).
; Aufbau: Eine Kachel besteht aus 2 x 2 Zeichenzellen; hg_c0, hg_c1, ... sind je eine Tabelle pro Zelle (zeilenweise von links oben,
;   planar): hg_cN[k] = Bildschirmcode der Zelle N von Kachel k (k = 0..5).  Die Tabellen liegen unmittelbar hintereinander.
; hg_col = Farbe je Kachel (Farb-RAM-Wert, Bit 3 = Mehrfarbe), hg_props = Eigenschaftenbyte je Kachel (Bit 0 = P_SOLID).
; levels_map0..2 = drei Karten (Screens 0..2) aus Kachelnummern, zeilenweise, levels_tiles_w x levels_tiles_h Kacheln;
;   levels_screens (lo/hi) = Adressen der Karten, lv1_id..lv3_id = Screen-Nummern.

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
levels_screens_lo = levels_screens
levels_screens_hi = levels_screens + 3
.const levels_count = 3
.const levels_tiles_w = 15
.const levels_tiles_h = 10
.const lv1_id = 0
.const lv2_id = 1
.const lv3_id = 2
