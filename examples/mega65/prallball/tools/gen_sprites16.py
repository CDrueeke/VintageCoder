#!/usr/bin/env python3
"""Erzeugt lib/sprites16.inc: 16-Farb-Sprites (VIC-IV Full-Colour-Sprite-Modus) fuer PRALLBALL auf dem MEGA65.
Aufruf im Ordner examples/mega65/prallball:  python3 tools/gen_sprites16.py

Format (MEGA65 Chipset Reference, Full-Colour Sprite Mode): 16 Pixel breit, 4 Bit je Pixel (8 Byte je Zeile, erstes Pixel = hohes Halbbyte),
21 Zeilen = 168 Byte (3 Zeigerbloecke zu 64 Byte).  Farbe des Pixels = Palettenindex Spritenummer * 16 + Wert (Palettenbank der Sprites);
der Wert, der in der Spritefarbe ($D027 + n, unteres Halbbyte) steht, ist durchsichtig (hier 0).
Palette: Rampe fuer die Kugel (Wert 1 = Glanzlicht ... 12 = Schatten); die Halbbytes der Register $D100/$D200/$D300 sind vertauscht."""
import math, os
RAMP = [(0, 0, 0), (255, 255, 255), (225, 238, 255), (190, 218, 255), (150, 190, 255), (112, 160, 252), (80, 130, 240), (58, 104, 222),
        (42, 82, 198), (30, 62, 170), (20, 46, 140), (13, 32, 108), (9, 22, 78), (40, 60, 120), (200, 222, 255), (255, 255, 255)]
def sw(v): return ((v << 4) | (v >> 4)) & 255

def ball(n):                         # n x n Kugel, Licht von links oben
    rows = []
    r = n / 2
    for y in range(n):
        row = []
        for x in range(n):
            dx, dy = (x + 0.5 - r) / r, (y + 0.5 - r) / r
            d2 = dx * dx + dy * dy
            if d2 > 1.0:
                row.append(0); continue
            nz = math.sqrt(1 - d2)
            lum = max(0.0, (-dx * 0.5 - dy * 0.6 + nz * 0.62))     # Lambert, Licht von links oben
            if lum > 0.93 and x < r and y < r: row.append(1); continue
            row.append(max(2, min(12, 12 - int(lum * 11.5))))
        rows.append(row)
    return rows

def pack(rows):
    data = []
    for y in range(21):
        row = rows[y] if y < len(rows) else []
        row = (row + [0] * 16)[:16]
        for x in range(0, 16, 2): data.append(row[x] * 16 + row[x + 1])
    return data + [0] * (192 - len(data))

def lum(c): return 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
GOLD = [(0, 0, 0)] + [(min(255, round(lum(c) * 1.05)), round(lum(c) * 0.80), round(lum(c) * 0.22)) for c in RAMP[1:]]
GOLD[1] = (255, 255, 230)
PADP = [(0, 0, 0), (255, 255, 255), (214, 230, 255), (168, 200, 255), (120, 164, 250), (80, 128, 236), (52, 92, 204), (34, 64, 164),
        (22, 44, 124), (14, 30, 90), (8, 20, 60), (60, 60, 70), (100, 100, 110), (150, 150, 160), (200, 200, 210), (255, 255, 255)]
INSET = [2, 1, 0, 0, 1, 2]               # Einrueckung der runden Enden je Zeile (Bildpunkte)

def paddle(cols, left_round, right_round, k=1):
    """6 Zeilen hoch, `cols` Spalten breit (k = Pixelverdopplung durch X-Expansion: Einrueckung / k); Zeile y hat den Farbwert y + 1"""
    rows = []
    for y in range(6):
        ins = max(0, INSET[y] // k) if k > 1 else INSET[y]
        row = []
        for x in range(16):
            if x >= cols: row.append(0); continue
            if left_round and x < ins: row.append(0); continue
            if right_round and x >= cols - ins: row.append(0); continue
            row.append(y + 1)
        rows.append(row)
    return rows

BASE = [(0, 0, 0), (255, 255, 255), (224, 40, 40), (64, 224, 224), (200, 64, 200), (48, 200, 64), (48, 64, 224), (255, 224, 48),
        (255, 144, 32), (144, 80, 24), (255, 120, 120), (72, 72, 80), (120, 120, 128), (144, 255, 144), (128, 160, 255), (200, 200, 208)]
CAP_COLOR = [14, 10, 2, 5, 7, 8, 1, 3, 4, 13, 12]     # wie cap_color in prallball.asm (Farbindex je Kapselart)

def coin(h, y0):
    """Muenze (12 Spalten breit, Spalten 0-11, Zeile y0 bis y0+h-1), Licht von links oben; Index 1 Glanz, 2-8 Koerper hell->dunkel, 10 Rand."""
    rows = [[0] * 16 for _ in range(21)]
    cx = 5.5
    cy = y0 + (h - 1) / 2
    for y in range(y0, y0 + h):
        for x in range(12):
            dx = (x - cx) / 6.0
            dy = (y - cy) / (h / 2.0)
            d2 = dx * dx + dy * dy
            if d2 > 1.0: continue
            d = math.sqrt(d2)
            if h <= 3:                                 # Kante: schmaler Streifen
                rows[y][x] = (2, 1, 5)[min(2, y - y0)]
                continue
            lum = max(-1.0, min(1.0, -0.5 * dx - 0.6 * dy + 0.3))
            if d > 0.86: rows[y][x] = 10
            elif lum > 0.78 and d < 0.7: rows[y][x] = 1
            else: rows[y][x] = max(2, min(8, 2 + int((1 - lum) * 3.2)))
    return rows

def shot():
    rows = [[0] * 16 for _ in range(21)]
    for y in range(8):
        for x in range(2): rows[y][x] = 1 if y < 2 else 2
    return rows

def ramp_for(col):
    b = BASE[col]
    out = [(0, 0, 0)] * 16
    out[1] = tuple(round(v + (255 - v) * 0.8) for v in b)
    for i in range(2, 9):
        f = 1.25 - (i - 2) * 0.115
        out[i] = tuple(min(255, round(v * f)) for v in b)
    out[9] = tuple(round(v * 0.45) for v in b)
    out[10] = tuple(round(v * 0.28) for v in b)
    out[11] = (12, 12, 36)
    for i in range(12, 16): out[i] = b
    return out

d = pack(ball(6))
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: egal; die Ausgabedatei wird relativ zum Ort des Skripts (../lib) geschrieben.', epilog='Beispiele:\n  python3 examples/mega65/prallball/tools/gen_sprites16.py\n  cd examples/mega65/prallball && python3 tools/gen_sprites16.py\n\nSchreibt (im Projektordner examples/mega65/prallball, vorhandene Dateien werden ueberschrieben): lib/sprites16.inc.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
out = ['; ERZEUGT von tools/gen_sprites16.py: 16-Farb-Sprites (VIC-IV) und ihre Palette.  Die Spritedaten stehen ZUERST (Anfang = $B000 = Zeiger $C0).',
       '; Kugel 6 x 6 Bildpunkte (links oben im Sprite), 192 Byte (3 Zeigerbloecke)', 'gfx_ball16:']
for k in range(0, 192, 16): out.append('        .byte ' + ', '.join('$%02x' % v for v in d[k:k + 16]))
def block(label, rows, org, comment):
    dd = pack(rows)
    out.append('.org $%04x' % org)
    out.append('%s:   ; %s' % (label, comment))
    for k in range(0, 192, 16): out.append('        .byte ' + ', '.join('$%02x' % v for v in dd[k:k + 16]))
block('gfx_pad16s',  paddle(16, True, True),   0xb0c0, 'Schlaeger 16 px, beide Enden rund (Zeiger $C3)')
block('gfx_padl16',  paddle(16, True, False),  0xb180, 'Schlaeger links 16 px, linkes Ende rund (Zeiger $C6)')
block('gfx_padr16',  paddle(16, False, True),  0xb240, 'Schlaeger rechts 16 px, rechtes Ende rund (Zeiger $C9)')
block('gfx_padr32x', paddle(16, False, True, 2), 0xb300, 'Schlaeger rechts, X-verdoppelt = 32 px (Zeiger $CC)')
block('gfx_cap16', coin(12, 0), 0xb500, 'Muenze vorn, 12 x 12 (Zeiger $D4; der Buchstabe wird zur Laufzeit eingetragen)')
block('gfx_cap16a', coin(10, 1), 0xb5c0, 'Muenze gekippt, 10 Zeilen (Zeiger $D7)')
block('gfx_cap16b', coin(7, 2), 0xb680, 'Muenze halb gekippt, 7 Zeilen (Zeiger $DA)')
block('gfx_cap16c', coin(3, 5), 0xb740, 'Muenze von der Kante, 3 Zeilen (Zeiger $DD)')
block('cap_base', coin(12, 0), 0xb800, 'Muenze vorn ohne Buchstabe (Quelle beim Auswuerfeln)')
block('gfx_shot16', shot(), 0xb8c0, 'Laserschuss 2 x 8 Bildpunkte (Zeiger $E3)')
out.append('.org $b9c0')
out += ['; Palette der Schlaegersprites 1 und 2 (Eintraege 16-47, Halbbytes vertauscht)',
        'ppal_r: .byte ' + ', '.join('$%02x' % sw(c[0]) for c in PADP), 'ppal_g: .byte ' + ', '.join('$%02x' % sw(c[1]) for c in PADP),
        'ppal_b: .byte ' + ', '.join('$%02x' % sw(c[2]) for c in PADP)]
out += ['; Schuss-Palette (Sprites 6 und 7, Eintraege 96-127): 1 weiss, 2 gelb',
        'shot_r: .byte ' + ', '.join('$%02x' % sw(v) for v in [0, 255, 255, 255] + [255] * 12),
        'shot_g: .byte ' + ', '.join('$%02x' % sw(v) for v in [0, 255, 232, 150] + [232] * 12),
        'shot_b: .byte ' + ', '.join('$%02x' % sw(v) for v in [0, 255, 60, 30] + [60] * 12)]
RAMPS = [ramp_for(c) for c in CAP_COLOR]
out += ['; Paletten der Kapselarten (je 16 Eintraege = Sprite 0, Bank 1): cap_pr/pg/pb, Index = Art * 16 + Eintrag']
for nm, k in (('cap_pr', 0), ('cap_pg', 1), ('cap_pb', 2)):
    out.append(nm + ':')
    for r in RAMPS: out.append('        .byte ' + ', '.join('$%02x' % sw(c[k]) for c in r))
out += ['; Paletten der Kugel (Halbbytes vertauscht): blau (spal) und gold (spalg, bei Durchschlag)',
        'spal_r: .byte ' + ', '.join('$%02x' % sw(c[0]) for c in RAMP), 'spal_g: .byte ' + ', '.join('$%02x' % sw(c[1]) for c in RAMP),
        'spal_b: .byte ' + ', '.join('$%02x' % sw(c[2]) for c in RAMP),
        'spalg_r: .byte ' + ', '.join('$%02x' % sw(c[0]) for c in GOLD), 'spalg_g: .byte ' + ', '.join('$%02x' % sw(c[1]) for c in GOLD),
        'spalg_b: .byte ' + ', '.join('$%02x' % sw(c[2]) for c in GOLD)]
# Tempo-Glut: je Geschwindigkeitsgruppe (Stufe 4-7, 8-11, 12-15) wird die blaue Kugel zur roten hin ueberblendet (Rot und Blau vertauscht), 1/3, 2/3, 3/3.
#   heat_r/g/b: je 3 x 16 Eintraege, Index = (Gruppe - 1) * 16 + Eintrag
HOT = [[], [], []]
for lv in (1, 2, 3):
    k = lv / 3.0
    for c in RAMP:
        HOT[0].append(int(round(c[0] + (c[2] - c[0]) * k)))
        HOT[1].append(int(round(c[1] + (0.45 * c[1] - c[1]) * k)))
        HOT[2].append(int(round(c[2] + (c[0] - c[2]) * k)))
for nm, ch in (('heat_r', 0), ('heat_g', 1), ('heat_b', 2)):
    out.append(nm + ':')
    for lv in range(3): out.append('        .byte ' + ', '.join('$%02x' % sw(v) for v in HOT[ch][lv * 16:lv * 16 + 16]))
open(os.path.join(os.path.dirname(__file__), '..', 'lib', 'sprites16.inc'), 'w', newline='\n').write('\n'.join(out) + '\n')
print('lib/sprites16.inc geschrieben')
for r in ball(6): print(''.join('%x' % v if v else '.' for v in r))
