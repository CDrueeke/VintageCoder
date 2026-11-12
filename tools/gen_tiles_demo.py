#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt die Daten und Testdateien der Beispiele examples/<system>/tiles_demo (C64, Atari 8 Bit, Plus/4, VC-20, C128, MEGA65).

Die Kacheldaten (data_3x2.inc, data_2x2.inc) haben das Exportformat des Kachel-Editors (Konzept, Anhang A). Der Python-Teil
unten ist die unabhängige Referenz: er "zeichnet" die Screens selbst in ein Abbild (Zeichen und, wo vorhanden, Farb-RAM)
und rechnet die CRC-16 (XMODEM) darüber; das Testprogramm rechnet dieselbe CRC über den echten Bildschirm.
Beim C64 sind demo.inc und die vier tiles_demo_*.asm von Hand geschrieben; bei den anderen Systemen erzeugt dieses Skript
auch sie (demo.inc aus einer Vorlage mit den Systemteilen).
Aufruf: python3 tools/gen_tiles_demo.py [system ...]   (im Repository-Wurzelverzeichnis; ohne Angabe alle Systeme)"""
import argparse, binascii, json, os, sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')

# Gemeinsame Kachelsätze: Kachelgröße, Anzahl der Screens; die Kartengröße (Kacheln) hängt vom Bildschirm ab
COL_C64 = [1, 2, 5, 7, 13, 14]       # Farbe je Kachel (Farb-RAM-Wert; Bit 3 = Mehrfarbe)
COL_C64[3] = 8 | 7                    # Kachel 3 mit Mehrfarbbit
COL_PLUS4 = [0x71, 0x42, 0x55, 0x6f, 0x5d, 0x3e]   # Attribut-Byte: Helligkeit * 16 + Farbton
PROPS = [0, 1, 1, 2, 4, 5]            # Eigenschaftenbyte je Kachel

SYSTEMS = {
    'c64': dict(
        dir='c64', title='C64', target='c64', lib='c64', cpu='6510', cols=40, rows=25, screen=0x0400, colorram=0xD800,
        colmask=15, blank_ch=0x20, blank_col=0x01, col=COL_C64,
        maps={'3x2': (12, 10), '2x2': (15, 10)}, mapram=0xC800, trap=None, cycles_tests=True, mode='trap', generate_asm=False,
        zp_forbid='$02-$21', end_expr='size(\"\") < $2000 - $0801', fcol=14),
    'plus4': dict(
        dir='plus4', title='Plus/4', target='plus4', lib='plus4', cpu='7501', cols=40, rows=25, screen=0x0C00, colorram=0x0800,
        colmask=0xff, blank_ch=0x20, blank_col=0x71, col=COL_PLUS4,
        maps={'3x2': (12, 10), '2x2': (15, 10)}, mapram=0x3000, trap=0x0340, cycles_tests=False, mode='trap', generate_asm=True,
        zp_crc=0x22, zp_forbid='$02-$21', org='$1001', stub=True, fcol=0x77,
        vcprj=dict(id='plus4', system='plus4', variant='pal', cpu='7501', emulator='vice'), limit=0x4000,
        init=['sei', 'lda #0', 'sta $ff19                  ; Rahmen schwarz', 'sta $ff15                  ; Hintergrund schwarz']),
    'vic20': dict(
        dir='vic20', title='VC-20 (Grundgerät)', target='vic20', lib='vic20', cpu='6502', cols=22, rows=23, screen=0x1E00, colorram=0x9600,
        colmask=15, blank_ch=0x20, blank_col=0x01, col=COL_C64,
        maps={'3x2': (7, 11), '2x2': (11, 11)}, mapram=0x0360, trap=0x0340, cycles_tests=False, mode='trap', generate_asm=True,
        zp_crc=0x22, zp_forbid='$02-$21', org='$1001', stub=True, fcol=7,
        vcprj=dict(id='vic20', system='vic20', variant='pal', memory='unexpanded', cpu='nmos6502', emulator='vice'), limit=0x1E00,
        init=['sei', 'lda #$08', 'sta VIC_COLOR              ; Hintergrund und Rahmen schwarz']),
    'c128': dict(
        dir='c128', title='C128 (VIC-IIe, 40 Spalten)', target='c128', lib='c128', cpu='8502', cols=40, rows=25, screen=0x0400, colorram=0xD800,
        colmask=15, blank_ch=0x20, blank_col=0x01, col=COL_C64,
        maps={'3x2': (12, 10), '2x2': (15, 10)}, mapram=0x3000, trap=0x0B00, cycles_tests=False, mode='trap', generate_asm=True,
        zp_crc=0x22, zp_forbid='$02-$21', org='$1c01', stub=True, fcol=7,
        vcprj=dict(id='c128', system='c128', variant='pal', cpu='8502', emulator='vice'), limit=0x3000,
        init=['sei', 'lda #0', 'sta VIC_BORDERCOLOR', 'sta VIC_BGCOLOR0']),
    'mega65': dict(
        dir='mega65', title='MEGA65 (VIC-II-Modus)', target='mega65', lib='mega65', cpu='45gs02', cols=40, rows=25, screen=0x0800, colorram=0xD800,
        colmask=15, blank_ch=0x20, blank_col=0x01, col=COL_C64,
        maps={'3x2': (12, 10), '2x2': (15, 10)}, mapram=0x3000, trap=0x1F00, cycles_tests=False, mode='static', generate_asm=True,
        zp_crc=0x22, zp_forbid='$02-$21', org='$2001', stub=True, fcol=7,
        vcprj=dict(id='mega65', system='mega65', variant=None, cpu='45gs02', emulator='xemu'), limit=0x7000,
        includes=['#include <mega65/gs4502.asm>   ; Makro mega65_enable_vic4'],
        init=['sei', 'mega65_enable_vic4          ; VIC-IV freischalten ($47, $53 nach $D02F)', 'lda #0',
              'sta $d030                  ; keine C65-ROM-Fenster', 'sta $d031                  ; 40 Spalten (H640 aus)',
              'sta $d054                  ; keine 16-Bit-Zeichen', 'lda #40', 'sta $d058                  ; Zeilenabstand 40 Byte',
              'lda #0', 'sta $d059', 'lda #40', 'sta $d05e                  ; 40 Zeichen je Zeile',
              'lda #((TILE_SCREEN >> 6) & $f0) | 4', 'sta $d018                  ; Bildschirm bei TILE_SCREEN, Zeichensatz-ROM',
              'lda #$08', 'sta $d016                  ; 40 Spalten, kein Mehrfarbtext',
              'lda #0', 'sta VIC_BORDERCOLOR', 'sta VIC_BGCOLOR0']),
    'atari': dict(
        dir='atari', title='Atari 8 Bit (ANTIC-Modus 2)', target='atari8', lib='atari8bit', cpu='sally', cols=40, rows=24, screen=0x4000, colorram=None,
        colmask=0, blank_ch=0x00, blank_col=0, col=None,
        maps={'3x2': (13, 12), '2x2': (20, 12)}, mapram=0x5000, trap=None, cycles_tests=False, mode='mailbox', generate_asm=True,
        zp_crc=0x80, zp_forbid='$00-$7f', org='$2000', stub=False, fcol=0, limit=0x3800,
        vcprj=dict(id='atari8', system='atari8', variant='pal', cpu='sally', emulator='atari800', model='800xl'),
        includes=['#include <atari8bit/test.inc>'], init=[]),
}


def build(s, mapsz, n=6, screens=3):
    tw, th = s['tw'], s['th']
    cells = [[(0x11 + 7 * t + 3 * k) & 0xff for t in range(n)] for k in range(tw * th)]   # planar: cells[k][t]
    maps = []
    seed = 12345
    for sc in range(screens):
        m = []
        for y in range(mapsz[1]):
            row = []
            for x in range(mapsz[0]):
                seed = (seed * 1103515245 + 12345) & 0x7fffffff     # einfacher Zufallsgenerator, nur für Testdaten
                row.append((seed >> 16) % n)
            m.append(row)
        maps.append(m)
    return cells, maps


def render(Y, s, cells, maps_sc, draw=None, rows=None):
    """Referenzbild: (Zeichen, Farbe) nach dem Löschen und Zeichnen. draw = Liste (kachel, tx, ty, farbe|None)."""
    cols, nrows, tw, th = Y['cols'], Y['rows'], s['tw'], s['th']
    scr = [Y['blank_ch']] * (cols * nrows)
    col = [Y['blank_col']] * (cols * nrows)
    COL = Y['col'] or [0] * 256

    def put(t, tx, ty, f):
        for k in range(tw * th):
            cy, cx = divmod(k, tw)
            o = (ty * th + cy) * cols + tx * tw + cx
            scr[o] = cells[k][t]
            col[o] = COL[t] if f is None else f
    if maps_sc is not None:
        r0, r1 = rows if rows else (0, s['mh'])
        for y in range(r0, r1):
            for x in range(s['mw']):
                put(maps_sc[y][x], x, y, None)
    for (t, tx, ty, f) in (draw or []):
        put(t, tx, ty, f)
    return scr, col


def crc(b):
    return binascii.crc_hqx(bytes(b), 0)


def bytes_lines(label, vals, per=16):
    out = []
    for i in range(0, len(vals), per):
        chunk = ','.join('$%02x' % v for v in vals[i:i + per])
        out.append((label + ':' if i == 0 else ' ' * (len(label) + 1)) + '  .byte ' + chunk)
    return out


# ---------------------------------------------------------------- demo.inc (nur für die erzeugten Systeme)
def demo_inc(Y):
    color = Y['colorram'] is not None
    zp = Y['zp_crc']
    lines = []
    A = lines.append
    A('; Gemeinsamer Teil des Beispiels tiles_demo (%s): zeigt die Kachel-Bibliothek (#include <%s/tiles.asm>) und enthält die' % (Y['title'], Y['lib']))
    A('; Prüfroutinen für vcc test. Die vier Programme tiles_demo_*.asm unterscheiden sich nur in Kachelgröße (DEMO_SET = 3 für')
    A('; 3x2, 2 für 2x2), TILE_SMC und TILE_MAPRAM; sie legen diese Schalter fest und binden dann diese Datei ein.')
    A('; Erzeugt von tools/gen_tiles_demo.py (nicht von Hand ändern).')
    A('.target %s' % Y['target'])
    A('.cpu %s { illegals = deny }' % Y['cpu'])
    A('#include <%s/tiles.asm>' % Y['lib'])
    for inc in Y.get('includes', []):
        A(inc)
    A('')
    A('; ZP für die Prüfroutinen (außerhalb der Kachelgruppe $F7-$FE)')
    A('.const ZP_CRC = $%02x             ; CRC-16 (2 Byte)' % zp)
    A('.const ZP_PTR = $%02x             ; Zeiger auf die zu prüfenden Daten (2 Byte)' % (zp + 2))
    A('.const ZP_X   = $%02x' % (zp + 4))
    A('.const ZP_Y   = $%02x' % (zp + 5))
    A('.const ZP_CNT = $%02x             ; Zähler (2 Byte)' % (zp + 6))
    A('.const AREA   = TILE_COLS * TILE_ROWS')
    A('')
    A('.org %s' % Y['org'])
    if Y['stub']:
        A('.basicstub start')
    A('start:  jmp main')
    A('')
    A('; ---------------------------------------------------------------- Daten und Bibliothek')
    A('.if DEMO_SET == 3')
    A('.include "data_3x2.inc"')
    A('.include "expect_3x2.inc"')
    A('.else')
    A('.include "data_2x2.inc"')
    A('.include "expect_2x2.inc"')
    A('.endif')
    A('')
    A('        tiles_use hg_c0, hg_col, hg_props, hg_tw, hg_th, hg_count, levels_screens_lo, levels_screens_hi, levels_tiles_w, levels_tiles_h')
    A('.const FX = DEMO_FX')
    A('.const FY = DEMO_FY')
    A('')
    A('; ---------------------------------------------------------------- Programm: zeigt Screen 0, dann ein paar Einzelkacheln')
    mailbox = Y['mode'] == 'mailbox'
    A('main:')
    if mailbox:
        A('        test_begin                  ; Postfach-Modus: das Programm prüft sich selbst (siehe unten), zeigt dann die Demo')
        A('        jsr selftest')
        A('        jsr setup_display')
    else:
        for l in Y['init']:
            A('        ' + l)
    A('        jsr clear_all')
    A('        screen_draw lv1_id          ; Screen 0 wählen und zeichnen')
    A('        tile_draw 3, 5, 2           ; Kachel 3 an Kachelposition (5, 2), nur Konstanten')
    A('        tile_draw_color 1, 0, 0, %d' % Y['fcol'])
    A('        tile_fill 4, FX, FY, 2, 2   ; 2 x 2 Kacheln der Kachel 4')
    A('        lda #1')
    A('        ldx #4')
    A('        ldy #6')
    A('        jsr tile_put                ; Laufzeit: A = Kachel, X = Spalte, Y = Zeile')
    if mailbox:
        A('        test_end')
    else:
        A('        jmp *')
    A('')
    if mailbox:
        A('; Display-Liste: 3 x 8 leere Zeilen, 24 Textzeilen im ANTIC-Modus 2 (40 Spalten) ab TILE_SCREEN (LMS), Zeichensatz im ROM ($E000).')
        A('; Modus 4 (Mehrfarbtext) unterscheidet sich nur im Befehlsbyte ($44/$04 statt $42/$02) und braucht einen eigenen Zeichensatz.')
        A('SDLSTL = $0230')
        A('CHBAS  = $02f4')
        A('setup_display:')
        A('        lda #<dlist')
        A('        sta SDLSTL')
        A('        lda #>dlist')
        A('        sta SDLSTL+1')
        A('        lda #$e0')
        A('        sta CHBAS')
        A('        rts')
        A('')
        A('selftest:                        ; alle Prüfungen nacheinander; jede liefert A = 0, wenn die Bildschirm-CRC stimmt')
        A('        jsr t_screen1')
        A('        assert_a_eq 0')
        A('        jsr t_rows')
        A('        assert_a_eq 0')
        A('        jsr t_macros')
        A('        assert_a_eq 0')
        A('        jsr t_puts')
        A('        assert_a_eq 0')
        for i in range(3):
            A('        ldx #SELF_X%d' % i)
            A('        ldy #SELF_Y%d' % i)
            A('        jsr t_map_get')
            A('        assert_a_eq SELF_T%d' % i)
            A('        ldx #SELF_X%d' % i)
            A('        ldy #SELF_Y%d' % i)
            A('        jsr t_prop')
            A('        assert_a_eq SELF_P%d' % i)
        A('        rts')
        A('')
    A('; ---------------------------------------------------------------- Prüfroutinen (Aufruf durch vcc test, siehe tiles_*.vctest)')
    A('; Bildschirm löschen (Zeichen = blank, Farbe = blank_col)')
    A('clear_all:')
    A('        ldx #0')
    A('        lda #$%02x' % Y['blank_ch'])
    area = Y['cols'] * Y['rows']
    offs = []
    o = 0
    while o + 256 <= area:
        offs.append(o)
        o += 256
    if area % 256:
        offs.append(area - 256)
    A('.l:')
    for o in offs:
        A('        sta TILE_SCREEN + $%x,x' % o)
    if color:
        A('        lda #$%02x' % Y['blank_col'])
        for o in offs:
            A('        sta TILE_COLORRAM + $%x,x' % o)
        A('        lda #$%02x' % Y['blank_ch'])
    A('        inx')
    A('        bne .l')
    A('        rts')
    A('')
    A('; crc_area: CRC-16 (XMODEM) über AREA Byte ab (ZP_PTR); X = Maske für jedes Byte -> Ergebnis in ZP_CRC')
    A('''crc_area:
        stx ZP_X
        lda #0
        sta ZP_CRC
        sta ZP_CRC+1
        lda #<AREA
        sta ZP_CNT
        lda #>AREA
        sta ZP_CNT+1
        ldy #0
.byte:  lda (ZP_PTR),y
        and ZP_X
        eor ZP_CRC+1
        sta ZP_CRC+1
        ldx #8
.bit:   asl ZP_CRC
        rol ZP_CRC+1
        bcc .nx
        lda ZP_CRC+1
        eor #$10
        sta ZP_CRC+1
        lda ZP_CRC
        eor #$21
        sta ZP_CRC
.nx:    dex
        bne .bit
        iny
        bne .pg
        inc ZP_PTR+1
.pg:    lda ZP_CNT
        bne .d1
        dec ZP_CNT+1
.d1:    dec ZP_CNT
        lda ZP_CNT
        ora ZP_CNT+1
        bne .byte
        rts
''')
    if color:
        A('; check(exp_scr, exp_col): A = 0, wenn Bildschirm und Farb-RAM die erwartete CRC haben; 1 = Bildschirm falsch, 2 = Farbe falsch')
        A('.macro check(exp_scr, exp_col)')
    else:
        A('; check(exp_scr): A = 0, wenn der Bildschirm die erwartete CRC hat; 1 = falsch (kein Farbspeicher)')
        A('.macro check(exp_scr)')
    A('''        lda #<TILE_SCREEN
        sta ZP_PTR
        lda #>TILE_SCREEN
        sta ZP_PTR+1
        ldx #$ff
        jsr crc_area
        lda ZP_CRC
        cmp #<exp_scr
        bne @bad1
        lda ZP_CRC+1
        cmp #>exp_scr
        bne @bad1''')
    if color:
        A('''        lda #<TILE_COLORRAM
        sta ZP_PTR
        lda #>TILE_COLORRAM
        sta ZP_PTR+1
        ldx #$%02x
        jsr crc_area
        lda ZP_CRC
        cmp #<exp_col
        bne @bad2
        lda ZP_CRC+1
        cmp #>exp_col
        bne @bad2''' % Y['colmask'])
    A('        lda #0')
    A('        rts')
    A('@bad1:  lda #1')
    A('        rts')
    if color:
        A('@bad2:  lda #2')
        A('        rts')
    A('.endmacro')
    A('')
    chk = lambda k: ('check EXP_%s_SCR, EXP_%s_COL' % (k, k)) if color else ('check EXP_%s_SCR' % k)
    A('t_screen1:                          ; ganzer Screen 1')
    A('        jsr clear_all')
    A('        screen_draw lv2_id')
    A('        ' + chk('S1'))
    A('')
    A('t_screen1_nocheck:')
    A('        jsr clear_all')
    A('        screen_draw lv2_id')
    A('        rts')
    A('')
    A('t_draw_only:                        ; nur das Zeichnen (für die Zyklenmessung)')
    A('        screen_draw lv2_id')
    A('        rts')
    A('')
    A('t_rows:                             ; Kachelzeilen 3..5 von Screen 3')
    A('        jsr clear_all')
    A('        screen_select lv3_id')
    A('        lda #3')
    A('        ldx #3')
    A('        jsr screen_draw_rows')
    A('        ' + chk('ROWS'))
    A('')
    A('t_macros:')
    A('        jsr clear_all')
    A('        tile_draw 3, 5, 2')
    A('        tile_draw_color 1, 0, 0, 14')
    A('        tile_fill 4, FX, FY, 2, 2')
    A('        ' + chk('MACRO'))
    A('')
    A('t_puts:')
    A('        jsr clear_all')
    A('        lda #5')
    A('        ldx #2')
    A('        ldy #3')
    A('        jsr tile_put')
    A('        lda #0')
    A('        ldx #levels_tiles_w - 1')
    A('        ldy #levels_tiles_h - 1')
    A('        jsr tile_put')
    A('        lda #3')
    A('        ldx #0')
    A('        ldy #0')
    A('        jsr tile_put')
    A('        ' + chk('PUT'))
    A('')
    A('''t_map_get:                          ; X = Spalte, Y = Zeile von Screen 2 (lv2_id) -> A = Kachelnummer
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp screen_map_get

t_prop:
        stx ZP_X
        sty ZP_Y
        screen_select lv2_id
        ldx ZP_X
        ldy ZP_Y
        jmp tile_prop
''')
    A('; ---------------------------------------------------------------- Laufzeit der Bibliothek (einmal, hinter dem Programm)')
    A('        tiles_runtime')
    A('        .assert * <= $%04x, "Programm überschreitet die Grenze des freien Speichers (Bildschirm, Display-Liste oder Karte)"' % Y['limit'])
    if mailbox:
        A('')
        A('        .org $3800                  ; Display-Liste (darf keine 1-KB-Grenze überschreiten)')
        A('dlist:  .byte $70,$70,$70')
        A('        .byte $42,<TILE_SCREEN,>TILE_SCREEN')
        A('        .repeat TILE_ROWS - 1')
        A('        .byte $02')
        A('        .endrepeat')
        A('        .byte $41,<dlist,>dlist')
        A('        .run start')
    return '\n'.join(lines) + '\n'


def vcprj(Y):
    t = {k: v for k, v in dict(id=Y['vcprj']['id'], system=Y['vcprj']['system'], model=Y['vcprj'].get('model'),
                               variant=Y['vcprj'].get('variant'), memory=Y['vcprj'].get('memory'),
                               cpu=Y['vcprj']['cpu'], illegals='deny', emulator=Y['vcprj']['emulator']).items() if v}
    files = ['tiles_demo_3x2_ptr.asm', 'tiles_demo_3x2_smc.asm', 'tiles_demo_2x2_ptr.asm', 'tiles_demo_2x2_smc.asm']
    d = {"format": 1,
         "project": {"name": "%s: tiles_demo" % Y['title'].split(' (')[0],
                     "description": "Kachel-Bibliothek #include <%s/tiles.asm>: 3x2- und 2x2-Kacheln, Zeiger- und SMC-Version, Test mit vcc test" % Y['lib']},
         "targets": [t], "sources": [{"targets": [t['id']], "kind": "asm", "files": files}], "main": files[0]}
    return json.dumps(d, indent=2, ensure_ascii=False) + '\n'


def main(which):
    for key in which:
        gen(key, SYSTEMS[key])


def gen(key, Y):
    OUT = os.path.join(ROOT, 'examples', Y['dir'], 'tiles_demo')
    os.makedirs(OUT, exist_ok=True)
    COLS, ROWS, SCREEN, COLORRAM = Y['cols'], Y['rows'], Y['screen'], Y['colorram']
    color = COLORRAM is not None
    COL = Y['col'] or [0] * 6
    if Y['generate_asm']:
        open(os.path.join(OUT, 'demo.inc'), 'w', encoding='utf-8').write(demo_inc(Y))
        open(os.path.join(OUT, 'tiles_demo.vcprj'), 'w', encoding='utf-8').write(vcprj(Y))
    for name, (mw, mh) in Y['maps'].items():
        tw, th = (3, 2) if name == '3x2' else (2, 2)
        s = dict(tw=tw, th=th, mw=mw, mh=mh, count=6, screens=3)
        cells, maps = build(s, (mw, mh))
        n = s['count']
        L = ['; Kachelsatz hg (%s) und Screen-Bank levels, von Hand im Exportformat des Kachel-Editors (Konzept, Anhang A).' % name,
             '; Erzeugt von tools/gen_tiles_demo.py (die Daten sind frei erfunden, kein Bildinhalt).', '']
        for k in range(tw * th):
            L += bytes_lines('hg_c%d' % k, cells[k])
        L += bytes_lines('hg_col', COL)
        L += bytes_lines('hg_props', PROPS)
        L += ['.const hg_tw = %d' % tw, '.const hg_th = %d' % th, '.const hg_count = %d' % n, '.const P_SOLID = 1', '']
        for sc in range(s['screens']):
            L += bytes_lines('levels_map%d' % sc, [v for row in maps[sc] for v in row], mw)
        L += ['levels_screens: .lohifill %d, levels_map0 + %d * i' % (s['screens'], mw * mh)]
        L += ['levels_screens_lo = levels_screens', 'levels_screens_hi = levels_screens + %d' % s['screens']]
        L += ['.const levels_count = %d' % s['screens'], '.const levels_tiles_w = %d' % mw, '.const levels_tiles_h = %d' % mh,
              '.const lv1_id = 0', '.const lv2_id = 1', '.const lv3_id = 2']
        open(os.path.join(OUT, 'data_%s.inc' % name), 'w', encoding='utf-8').write('\n'.join(L) + '\n')

        # Szenarien: (Name, Bild); Positionen für Systeme mit kleinem Bildschirm angepasst (FX/FY)
        fx, fy = (6, 6) if key == 'c64' else (min(6, mw - 2), min(6, mh - 2))
        put_calls = [(5, 2, 3), (0, mw - 1, mh - 1), (3, 0, 0)]                      # tile_put: A = Kachel, X = Spalte, Y = Zeile
        macro_draw = [(3, 5, 2, None), (1, 0, 0, 14)] + [(4, fx + dx, fy + dy, None) for dy in (0, 1) for dx in (0, 1)]
        if key == 'c64':
            macro_draw = [(3, 5, 2, None), (1, 0, 0, 14), (4, 6, 6, None), (4, 7, 6, None), (4, 6, 7, None), (4, 7, 7, None)]
        demo_draw = [(3, 5, 2, None), (1, 0, 0, Y['fcol'])] + [(4, fx + dx, fy + dy, None) for dy in (0, 1) for dx in (0, 1)] + [(1, 4, 6, None)]
        R = lambda *a, **k: render(Y, s, cells, *a, **k)
        sc = {
            'S1': R(maps[1]),
            'ROWS': R(maps[2], rows=(3, 6)),
            'MACRO': R(None, draw=macro_draw),
            'PUT': R(None, draw=[(t, x, y, None) for (t, x, y) in put_calls]),
        }
        E = ['; Erwartete CRC-16 (XMODEM) über Bildschirm (%d Byte)%s je Szenario; erzeugt von tools/gen_tiles_demo.py'
             % (COLS * ROWS, (' und Farb-RAM (%d Byte, Maske $%02x)' % (COLS * ROWS, Y['colmask'])) if color else ', ohne Farbspeicher')]
        for k2, (scr, col) in sc.items():
            E.append('.const EXP_%s_SCR = $%04x' % (k2, crc(scr)))
            if color:
                E.append('.const EXP_%s_COL = $%04x' % (k2, crc([c & Y['colmask'] for c in col])))
        E.append('.const MAP_TILES_W = %d' % mw)
        if Y['generate_asm']:
            E.append('.const DEMO_FX = %d' % fx)
            E.append('.const DEMO_FY = %d' % fy)
            mid3 = [(0, 0), (mw - 1, mh - 1), (3, 4)]
            if Y['mode'] == 'mailbox':
                E.append('; Selbsttest: drei Stellen der Karte von Screen 1 (Spalte, Zeile, Kachelnummer, Eigenschaftenbyte)')
                for i, (x, y) in enumerate(mid3):
                    t = maps[1][y][x]
                    E += ['.const SELF_X%d = %d' % (i, x), '.const SELF_Y%d = %d' % (i, y), '.const SELF_T%d = %d' % (i, t), '.const SELF_P%d = %d' % (i, PROPS[t])]
        open(os.path.join(OUT, 'expect_%s.inc' % name), 'w', encoding='utf-8').write('\n'.join(E) + '\n')

        def cell_exprs(img, ty0, tx0, t):
            exprs = []
            scr, col = img
            for k in range(tw * th):
                cy, cx = divmod(k, tw)
                o = (ty0 * th + cy) * COLS + tx0 * tw + cx
                exprs.append('[$%04x] == %d' % (SCREEN + o, scr[o]))
                if color:
                    if Y['colmask'] == 0xff:
                        exprs.append('[$%04x] == %d' % (COLORRAM + o, col[o]))
                    else:
                        exprs.append('([$%04x] & %d) == %d' % (COLORRAM + o, Y['colmask'], col[o] & Y['colmask']))
            return exprs

        s1 = sc['S1']
        tests = []
        tests.append({"name": "screen_draw 1: Bildschirm und Farb-RAM = Referenz" if color else "screen_draw 1: Bildschirm = Referenz",
                      "call": "t_screen1", "expect": ["a == 0"], "timeoutMs": 20000})
        tests.append({"name": "screen_draw 1: Stichproben links oben / rechts unten", "call": "t_screen1_nocheck",
                      "expect": cell_exprs(s1, 0, 0, maps[1][0][0]) + cell_exprs(s1, mh - 1, mw - 1, 0), "timeoutMs": 20000})
        tests.append({"name": "screen_draw_rows 3..5 von Screen 2: nur diese Zeilen (Referenz)", "call": "t_rows", "expect": ["a == 0"], "timeoutMs": 20000})
        tests.append({"name": "tile_draw, tile_draw_color, tile_fill (Referenz)", "call": "t_macros", "expect": ["a == 0"], "timeoutMs": 20000})
        tests.append({"name": "tile_put dreimal (Referenz)", "call": "t_puts", "expect": ["a == 0"], "timeoutMs": 20000})
        put_max = 150 if tw * th <= 4 else 190              # 150 Zyklen für 2x2 (Konzept 6.4), sonst anteilig
        for (t, x, y) in put_calls:
            img = sc['PUT']
            tc = {"name": "tile_put: Kachel %d nach (%d,%d)%s" % (t, x, y, ', Zyklen' if Y['cycles_tests'] else ''),
                  "setup": {"a": str(t), "x": x, "y": y}, "call": "tile_put",
                  "expect": cell_exprs(img, y, x, t), "timeoutMs": 2000}
            if Y['cycles_tests']:
                tc["cycles"] = {"max": put_max, "level": "error"}
            tests.append(tc)
        for (x, y) in [(0, 0), (mw - 1, mh - 1), (3, 4)]:
            t = maps[1][y][x]
            tests.append({"name": "screen_map_get (%d,%d) von Screen 1 = %d" % (x, y, t), "setup": {"x": x, "y": y},
                          "call": "t_map_get", "expect": ["a == %d" % t], "timeoutMs": 2000})
            tests.append({"name": "tile_prop (%d,%d) von Screen 1 = %d" % (x, y, PROPS[t]), "setup": {"x": x, "y": y},
                          "call": "t_prop", "expect": ["a == %d" % PROPS[t]], "timeoutMs": 2000})
        if Y['cycles_tests']:
            tests.append({"name": "screen_draw 1 (ganze Kachelebene) Zyklen", "call": "t_draw_only", "cycles": {"max": 40000, "level": "hint"}, "timeoutMs": 20000})
        if not Y['generate_asm']:
            pass
        for var, smc in (('ptr', False), ('smc', True)):
            vt = list(tests)
            if smc and name == '3x2':
                mp = [v for row in maps[1] for v in row]
                vt.append({"name": "TILE_MAPRAM: Karte von Screen 1 liegt nach screen_select bei $%04x" % Y['mapram'], "call": "t_screen1_nocheck",
                           "expect": ["[$%04x] == %d" % (Y['mapram'] + i, mp[i]) for i in (0, 1, mw + 1, mw * mh - 1)], "timeoutMs": 20000})
            static = [{"check": "overlap"}, {"check": "zeropage", "forbid": Y['zp_forbid']},
                      {"check": "cycles", "label": "tile_put", "max": put_max, "level": "error"}]
            if Y.get('end_expr'):
                static.append({"check": "expr", "name": "Programm endet vor $2000", "expr": Y['end_expr']})
            spec = {"kind": "test", "target": Y['target'], "program": "build/tiles_demo_%s_%s.%s" % (name, var, 'xex' if Y['target'] == 'atari8' else 'prg')}
            if Y['mode'] == 'mailbox':
                spec["mode"] = "mailbox"
                # Das Programm prüft sich selbst (siehe selftest in demo.inc); danach zeigt es die Demo, deren Zellen hier kontrolliert werden
                demo = R(maps[0], draw=demo_draw)
                spec["static"] = static
                spec["tests"] = [{"name": "Selbsttest (CRC aller Szenarien, map_get, prop) und Demo-Bild", "timeoutMs": 20000,
                                  "expect": cell_exprs(demo, 0, 0, 0) + cell_exprs(demo, 6, 4, 1)}]
            else:
                if Y['trap'] is not None:
                    spec["trap"] = "$%04x" % Y['trap']
                spec["static"] = static
                spec["tests"] = vt
            open(os.path.join(OUT, 'tiles_demo_%s_%s.vctest' % (name, var)), 'w', encoding='utf-8').write(json.dumps(spec, indent=1, ensure_ascii=False) + '\n')
            if Y['generate_asm']:
                a = ['; Kachel-Bibliothek, Variante %s_%s, siehe demo.inc' % (name, var), '.const DEMO_SET = %d' % tw,
                     '.const TILE_SMC = %d' % (1 if smc else 0)]
                if smc and name == '3x2':
                    a.append('.const TILE_MAPRAM = $%04x     ; Karte beim Wählen in den RAM-Puffer kopieren' % Y['mapram'])
                a.append('.include "demo.inc"')
                open(os.path.join(OUT, 'tiles_demo_%s_%s.asm' % (name, var)), 'w', encoding='utf-8').write('\n'.join(a) + '\n')
        print(key, name, {k2: ('%04x' % crc(v[0]), '%04x' % crc(v[1])) for k2, v in sc.items()})


if __name__ == '__main__':
    ap = argparse.ArgumentParser(
        description="""Erzeugt die Daten und Testdateien der Beispiele examples/<system>/tiles_demo (C64, Atari 8 Bit, Plus/4, VC-20, C128, MEGA65).

Je System werden in examples/<system>/tiles_demo geschrieben (UTF-8, vorhandene Dateien werden überschrieben, der Ordner wird bei Bedarf angelegt):
die Kacheldaten data_3x2.inc und data_2x2.inc (Exportformat des Kachel-Editors), die Testbeschreibungen tiles_demo_<größe>_<variante>.vctest und - außer beim
C64, wo sie von Hand geschrieben sind - demo.inc und die tiles_demo_*.asm. Der Python-Teil ist die unabhängige Referenz: er zeichnet die Screens selbst in ein Abbild
und rechnet die CRC-16 (XMODEM) darüber. ACHTUNG: Ein Lauf erzeugt viele Dateien. Pfade gelten relativ zum Repository (aus dem Ort des Skripts bestimmt);
der Arbeitsordner ist egal.""",
        epilog="""Beispiele:
  python3 tools/gen_tiles_demo.py            alle Systeme
  python3 tools/gen_tiles_demo.py c64 plus4  nur C64 und Plus/4

Gültige Systemnamen: c64, plus4, vic20, c128, mega65, atari.""",
        formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("system", nargs="*", metavar="SYSTEM", help="Systeme, die erzeugt werden sollen: c64, plus4, vic20, c128, mega65, atari (Standard: alle)")
    main(ap.parse_args().system or list(SYSTEMS))
