; Kachelsatz hg (2x2) und Screen-Bank levels im Exportformat des Kachel-Editors (Konzept, Anhang A), hier aber von
; tools/gen_tiles_demo.py erzeugt (die Daten sind frei erfunden, kein Bildinhalt).
;
; Kachelsatz hg: 6 Kacheln, jede 2x2 Zeichenzellen gross.
;   hg_c0..hg_c3  eine Tabelle je Zelle (planar): Index = Kachelnummer, Wert = Bildschirmcode der Zelle k; Zellen zeilenweise (k = Zeile * 2 + Spalte)
;   hg_col         Farb-RAM-Byte je Kachel (TED-Farbwert: Farbton + 16 * Helligkeit)
;   hg_props       Eigenschaftenbyte je Kachel (frei erfundene Beispielwerte; P_SOLID = 1 ist ein Bit davon)
;   hg_tw, hg_th, hg_count  Breite und Hoehe in Zellen und Anzahl der Kacheln
; Screen-Bank levels: 3 Karten (levels_map0..2), jede levels_tiles_w x levels_tiles_h Bytes zeilenweise mit Kachelnummern;
;   levels_screens ist die Adresstabelle (zuerst alle Low-Bytes, dann alle High-Bytes: .lohifill); lv1_id..lv3_id sind die Screen-Nummern 0..2.

hg_c0:  .byte $11,$18,$1f,$26,$2d,$34
hg_c1:  .byte $14,$1b,$22,$29,$30,$37
hg_c2:  .byte $17,$1e,$25,$2c,$33,$3a
hg_c3:  .byte $1a,$21,$28,$2f,$36,$3d
hg_col:  .byte $71,$42,$55,$6f,$5d,$3e
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
