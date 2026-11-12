#!/usr/bin/env python3
"""Erzeugt lib/dirtable.inc (Richtungstabelle), lib/levels.inc (Levels) und lib/gfx.inc.chars und lib/sprites.inc und lib/vdcfont.inc (Zeichen, Zeichen-Sprites, Dehnungstabellen) für PRALLBALL (C128, VDC).
Aufruf im Ordner examples/c128/prallball_vdc:  python3 tools/gen.py"""
import math, re, os
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/c128/prallball_vdc (os.chdir); alle Pfade gelten relativ dazu, der Aufruf funktioniert daher aus jedem Ordner.', epilog='Beispiele:\n  python3 examples/c128/prallball_vdc/tools/gen.py\n  cd examples/c128/prallball_vdc && python3 tools/gen.py\n\nSchreibt (im Projektordner examples/c128/prallball_vdc, vorhandene Dateien werden ueberschrieben): lib/dirtable.inc, lib/levels.inc, lib/vdcfont.inc, lib/sprites.inc, lib/gfx.inc.chars.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))

# ---- Richtungstabelle: 16 Geschwindigkeitsstufen x 8 Zonen; Einheit 1/64 Bildpunkt je Bild (PAL 50 Hz)
ANG = [-60, -43, -26, -9, 9, 26, 43, 60]
def s16(v): return v & 0xffff
dxl, dxh, dyl, dyh = [], [], [], []
for lvl in range(16):
    speed = (1.6 + 0.1 * lvl) * 64
    for a in ANG:
        r = math.radians(a)
        dx = int(round(speed * math.sin(r))); dy = int(round(-speed * math.cos(r)))
        dxl.append(s16(dx) & 255); dxh.append(s16(dx) >> 8); dyl.append(s16(dy) & 255); dyh.append(s16(dy) >> 8)
def tab(name, vals):
    out = [name + ':']
    for i in range(0, len(vals), 16): out.append('        .byte ' + ', '.join('$%02x' % v for v in vals[i:i + 16]))
    return out
L = ['; ERZEUGT von tools/gen.py: Richtungstabelle (Zone 0..7 = -60..+60 Grad), Einheit 1/64 Punkt je Bild']
for n, v in (('dir_xl', dxl), ('dir_xh', dxh), ('dir_yl', dyl), ('dir_yh', dyh)): L += tab(n, v)
open('lib/dirtable.inc', 'w').write('\n'.join(L) + '\n')

# ---- Levels
TYPE = {'.': 0, 'a': 1, 'b': 2, 'c': 3, 'd': 4, 'e': 5, 'f': 6, 'g': 7, 'h': 8, 's': 9, 'G': 10, '#': 11, 'x': 12, 'p': 13, 'C': 23}
# Zeichen im Level-Text -> Steintyp (wie brick_color/brick_pts in prallball.asm): a-h = Farbsteine 1..8 ('g' ist die siebte Farbe, NICHT Gold), s = Silber (9),
# G = Gold (10), # = unzerstoerbar (11), x = explosiv (12), p = Ueberraschung (13), C = Boss-Kern mit 8 Lebenspunkten (23); '.' = leer.
levels = []; cur = None
for line in open('levels/levels.txt', encoding='utf-8'):
    line = line.rstrip('\n')
    if line.startswith('#') and not line.startswith('#a') and ' ' in line[:2] + ' ' and re.match(r'#\s', line): continue
    m = re.match(r'level (\d+) "(.*)"', line)
    if m: cur = {'n': int(m.group(1)), 'name': m.group(2), 'rows': []}; levels.append(cur); continue
    if cur is not None and len(line) == 18: cur['rows'].append(line)
M = ['; ERZEUGT von tools/gen.py aus levels/levels.txt: je Level 216 Byte (12 Zeilen x 18 Spalten), Steintyp 0..13 und 23 (Boss-Kern)', 'level_count = %d' % len(levels)]
for lv in levels:
    assert len(lv['rows']) == 12, lv['n']
    M.append('level%d:   ; %s' % (lv['n'], lv['name']))
    for r in lv['rows']: M.append('        .byte ' + ', '.join(str(TYPE[c]) for c in r))
M.append('level_kind:    .byte ' + ', '.join('2' if l['name'].startswith('Boss') else '1' if l['name'].startswith('Bonus') else '0' for l in levels) + '   ; 0 normal, 1 Bonus (Zeit, kein Ballverlust), 2 Boss (Kerne, Angriffe)')
M.append('level_tab_lo:  .byte ' + ', '.join('<level%d' % l['n'] for l in levels))
M.append('level_tab_hi:  .byte ' + ', '.join('>level%d' % l['n'] for l in levels))
open('lib/levels.inc', 'w').write('\n'.join(M) + '\n')

# ---- Grafik: der VDC kennt nur EIN Pixel je Bit und eine Vordergrundfarbe je Zelle (Attribut); Pixelwert 1 = Vordergrund.
def bytes_(rows): return ', '.join('$%02x' % r for r in rows)
BRICK_L = [0x7f] * 7 + [0x00]        # Stein links: Luecke links (Bit 7) und unten
BRICK_R = [0xfe] * 7 + [0x00]        # Stein rechts: Luecke rechts (Bit 0) und unten
WALL    = [0xff, 0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0xff]
BLOCK   = [0x7e] + [0xff] * 6 + [0x7e]
G = ['; ERZEUGT von tools/gen.py: eigene Zeichen (einfarbig, 1 Bit je Pixel)']
G.append('gfx_chars:   ; Zeichen 64 (Stein links), 65 (Stein rechts), 66 (Wand), 67 (Block fuer das Titellogo), danach der volle Block (Zeichen 127)')
for ch in (BRICK_L, BRICK_R, WALL, BLOCK, [255] * 8): G.append('        .byte ' + bytes_(ch))
# Dehnungstabellen: jedes logische Zeichen (8 Pixel breit) wird auf dem VDC zwei Zellen breit: linke Haelfte = linke 4 Pixel (Bit 7..4) je doppelt, rechte = rechte 4 Pixel (Bit 3..0) je doppelt
def stretch(n): return sum(((n >> k) & 1) * 3 << (2 * k) for k in range(4))
V = ['; ERZEUGT von tools/gen.py: Dehnungstabellen fuer den VDC (80 Spalten): dbl_l = obere 4 Bit (linke 4 Pixel) eines Pixelbytes, dbl_r = untere 4 Bit (rechte 4 Pixel), je Pixel verdoppelt']
V.append('dbl_l:')
for i in range(0, 256, 16): V.append('        .byte ' + ', '.join('$%02x' % stretch(b >> 4) for b in range(i, i + 16)))
V.append('dbl_r:')
for i in range(0, 256, 16): V.append('        .byte ' + ', '.join('$%02x' % stretch(b & 15) for b in range(i, i + 16)))
open('lib/vdcfont.inc', 'w').write('\n'.join(V) + '\n')

# ---- Zeichen-Sprites (der VDC hat keine).  Bilder einfarbig (1 Bit je Pixel, 1 = Pixel gesetzt); fuer die 8 Feinlagen 0..7 vorverschoben und in Streifen je
# Zeichenspalte zerlegt.  Die Bilder entstehen aus den Mehrfarbbildern der C64-Fassung (jedes Mehrfarbpixel = 2 Bildpunkte; jeder nicht leere Wert = gesetzt).
def dbl(rows): return [[1 if v else 0 for v in r for _ in (0, 1)] for r in rows]
ball = [[0,2,0],[2,1,2],[2,2,3],[2,2,3],[2,3,3],[0,3,0]]
def padrows(left_round, right_round, width):
    rows = []
    for r in range(6):
        base = {0: 1, 1: 1, 2: 2, 3: 2, 4: 3, 5: 3}[r]
        row = [base] * width
        if r == 1: row[0] = 1 if left_round else base
        if left_round and r in (0, 5): row[0] = 0
        if right_round and r in (0, 5): row[-1] = 0
        rows.append(row)
    return rows
def hcat(a, b): return [x + y for x, y in zip(a, b)]
def coinrows(h):
    """Rotierende Muenze (Drehung um die WAAGERECHTE Achse), 6 Mehrfarbpixel = 12 Bildpunkte breit, Hoehe h (12, 10, 7, 3), senkrecht mittig in 12 Zeilen.
    Rahmen 8 Mehrfarbpixel (16 Bildpunkte, zwei Zeichen), Muenze in den Spalten 1..6."""
    rows = [[0] * 8 for _ in range(12)]
    top = (12 - h) // 2
    for r in range(h):
        dy = (r - (h - 1) / 2.0) / (h / 2.0)
        w = int(round(6 * math.sqrt(max(0.0, 1 - dy * dy))))
        w = max(2, w - (w % 2))
        if h <= 3: w = 6
        left = 1 + (6 - w) // 2
        for k in range(w): rows[top + r][left + k] = 1
    return rows
FONT = {'B': 0x7c66667c66667c00, 'S': 0x3c66603c06663c00, 'R': 0x7c66667c786c6600, 'L': 0x6060606060607e00, 'F': 0x7e60607c60606000,
        'K': 0x666c7870786c6600, '+': 0x0018187e7e181800, 'P': 0x7c66667c60606000, 'M': 0x63777f6b63636300, 'G': 0x3c66606e66663c00,
        'D': 0x786c6666666c7800}
CAPLET = 'BSRLFK+PMGD'                    # Reihenfolge wie die Kapselarten 0..10 im Spiel (siehe cap_tcol/cap_pick in prallball.asm)
def coin_front(letter):
    px = dbl(coinrows(12))
    g = FONT[letter]
    for gr in range(8):
        b = (g >> (8 * (7 - gr))) & 255
        for k in range(8):                # Buchstabe als Loch (Hintergrund) in der Muenze, Bildpunkte 4..11
            if (b >> (7 - k)) & 1: px[2 + gr][4 + k] = 0
    return px
IM = []                                   # (name, rows(Zeilen x Pixel), nsh)
def add(name, px, nsh=8): IM.append((name, px, nsh))
add('ball', dbl(ball))
add('pad16', dbl(padrows(True, True, 8)))
add('pad32', dbl(hcat(padrows(True, False, 8), padrows(False, True, 8))))
add('pad48', dbl(hcat(padrows(True, False, 12), padrows(False, True, 12))))
add('shot', [[1, 1]] * 8)
for n, h in (('edge', 3), ('half', 7), ('tq', 10)): add('coin_' + n, dbl(coinrows(h)), 1)
for c in CAPLET: add('coin_' + ('plus' if c == '+' else c), coin_front(c), 1)
S = ['; ERZEUGT von tools/gen.py: Zeichen-Sprites des VDC (vorverschobene Streifen, 1 Bit je Pixel).  Siehe prallball.asm (Abschnitt Zeichen-Sprites).',
     '; Je Bild: Daten- und Maskenstreifen.  Streifen (Feinlage s, Zeichenspalte c) beginnen bei Bildanfang + (s * NC + c) * (H + 16); jeder Streifen',
     '; hat 8 Leerzeilen davor und danach, damit jede senkrechte Verschiebung 0..7 ohne Pruefung gelesen werden kann.',
     '; Maske: 0 bei gesetzten Pixeln des Bildes (der Hintergrund wird geloescht), 1 bei den uebrigen (der Hintergrund bleibt).']
tabs = {k: [] for k in ('dlo', 'dhi', 'mlo', 'mhi', 'h', 'nc', 'smask', 'stride', 'sset', 'used')}
names = []
for idx, (name, rows, nsh) in enumerate(IM):
    H = len(rows); W = len(rows[0]); NC = (W + 7 + 7) // 8 if nsh == 8 else (W + 7) // 8
    stride = H + 16
    data, mask, used = [], [], []
    for s in range(nsh):
        u = 0
        for c in range(NC):
            d = [0] * 8; m = [255] * 8
            for r in range(H):
                bd = 0
                for k in range(8):
                    x = 8 * c + k - s
                    p = rows[r][x] if 0 <= x < W else 0
                    bd = (bd << 1) | p
                d.append(bd); m.append(255 ^ bd)
            d += [0] * 8; m += [255] * 8
            if any(d[8:8 + H]): u = c + 1
            data += d; mask += m
        used.append(u)
    used += [used[0]] * (8 - len(used))
    S.append('imd_%d:   ; %s (%d x %d Pixel, %d Zeichenspalten, %d Feinlagen)' % (idx, name, W, H, NC, nsh))
    for i in range(0, len(data), 16): S.append('        .byte ' + ', '.join('$%02x' % v for v in data[i:i + 16]))
    S.append('imm_%d:' % idx)
    for i in range(0, len(mask), 16): S.append('        .byte ' + ', '.join('$%02x' % v for v in mask[i:i + 16]))
    tabs['dlo'].append('<imd_%d' % idx); tabs['dhi'].append('>imd_%d' % idx); tabs['mlo'].append('<imm_%d' % idx); tabs['mhi'].append('>imm_%d' % idx)
    tabs['h'].append(str(H)); tabs['nc'].append(str(NC)); tabs['smask'].append('7' if nsh == 8 else '0'); tabs['stride'].append(str(stride)); tabs['sset'].append(str(NC * stride)); tabs['used'] += [str(u) for u in used]
    names.append('IMG_' + name.upper() + ' = %d' % idx)
S.append('; Bildnummern')
for n in names: S.append('.const ' + n.replace(' = ', '    = '))
for k in ('dlo', 'dhi', 'mlo', 'mhi', 'h', 'nc', 'smask', 'stride', 'sset', 'used'):
    S.append('im_%s:   .byte %s' % (k, ', '.join(tabs[k])))
open('lib/sprites.inc', 'w').write('\n'.join(S) + '\n')
open('lib/gfx.inc.chars', 'w').write('\n'.join(G) + '\n')
print('ok:', len(levels), 'Levels,', len(IM), 'Bilder')
