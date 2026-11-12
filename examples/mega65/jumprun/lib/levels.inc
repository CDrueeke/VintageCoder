; levels.inc - ERZEUGT von tools/gen_jumprun_c64.py (Spielwege aus tools/jumprun_art.py) - nicht von Hand ändern

; Kacheleigenschaften je Kachelnummer: 0 leer, 1 tragend, 2 Leiter
tile_attr:
        .byte 0,1,1,1,1,2,1,0,1,0,0,0,0,0,0,0,0,0,0,0,0
lvl_mc1:
        .byte 9,11,2,14
lvl_mc2:
        .byte 8,12,10,15
lvl_sky0:
        .byte 3,0,6,6
lvl_sky1:
        .byte 3,0,6,6
lvl_sky2:
        .byte 3,0,14,6
lvl_sky3:
        .byte 3,0,14,6
lvl_sky4:
        .byte 3,0,14,6
lvl_sky5:
        .byte 14,0,14,0
; Start je Bild: Held x (16 Bit), Fuß-y, Blickrichtung; Gegner x, y, Richtung
lvl_hx:
        .word 16,16,16,16
lvl_hy:
        .byte 160,160,160,160
lvl_hd:
        .byte 0,0,0,0
lvl_ex:
        .word 0,0,0,0
lvl_ey:
        .byte 160,160,160,160
lvl_ed:
        .byte 0,0,0,0
script1:
        .byte 4,40,1,0,49,3,0,48,1,0,7,5,0,2,0,70,224,5,1,1,0,11,6,11,4,1,0,10,6,12,4,1,0,3,5,0,2,0,46,32,5,1,1,0,8,5,0,2,0,18,48,5,1,1,0,1,4,50,0
script2:
        .byte 4,40,1,0,28,3,0,32,1,0,40,6,8,7,5,0,2,0,72,224,5,1,1,0,11,6,13,5,1,0,20,5,0,2,0,48,32,5,1,4,50,0
script3:
        .byte 4,40,1,0,71,3,0,64,1,0,24,5,0,2,0,56,0,5,1,1,0,15,6,15,5,1,0,31,4,50,0
script4:
        .byte 4,40,1,0,49,3,0,32,1,0,41,6,9,7,5,0,2,0,56,224,5,1,6,13,5,1,0,20,5,0,2,0,26,224,5,1,6,17,3,1,0,15,4,60,0
script_lo:
        .byte <script1,<script2,<script3,<script4
script_hi:
        .byte >script1,>script2,>script3,>script4
txt_lv1:
        .byte 106,92,88,102,88,128,122,128,102,98,97,97,92,90,88,101,128,84,97,89,84,97,90,128,128
txt_lv2:
        .byte 91,98,88,91,95,88,128,122,128,90,104,96,85,98,128,94,95,88,103,103,88,101,103,128,128
txt_lv3:
        .byte 85,104,101,90,128,122,128,95,88,92,103,88,101,97,128,84,104,89,128,104,97,87,128,84,85
txt_lv4:
        .byte 106,98,95,94,88,97,128,122,128,87,84,102,128,109,92,88,95,128,92,97,128,102,92,86,91
txt_lv_lo:
        .byte <txt_lv1,<txt_lv2,<txt_lv3,<txt_lv4
txt_lv_hi:
        .byte >txt_lv1,>txt_lv2,>txt_lv3,>txt_lv4
txt_hud:
        .byte 91,104,90,98,128,110,110,110,110,110,110,128,128,128,85,92,95,87,128,111,121,114,128,128,128,95,88,85,88,97
txt_prompt:
        .byte 87,88,96,98,128,122,128,97,92,86,91,103,128,102,99,92,88,95,85,84,101
.const CH_BLANK = 0
.const CH_SOLID = 81
.const CH_FONT = 84
.const CH_DIGIT0 = 110
.const CH_STAR = 127
