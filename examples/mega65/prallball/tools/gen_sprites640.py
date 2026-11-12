#!/usr/bin/env python3
"""Erzeugt lib/sprites640.inc: 16-Farb-Sprites fuer die hohe Aufloesung (640 x 400) von PRALLBALL auf dem MEGA65.

In 640 x 400 ist ein Spritepunkt ein echter Bildpunkt (SPRH640 in $D054 und SPRENV400 in $D076), ein 16-Farb-Sprite 16 Punkte breit.  Die Bilder haben
die doppelte Aufloesung der 320er Sprites (Kugel 12 x 12, Schlaeger 12 Zeilen hoch, Muenze 16 x 16, Schuss 4 x 16).  Jeder Sprite belegt 16 Zeilen
zu 8 Byte = 128 Byte (zwei Zeigerbloecke zu 64 Byte) im freien Bereich $8000-$87F7 der VIC-Bank 2 (Zeiger = (Adresse - $8000) / 64).
Die Farbwerte sind dieselben wie bei den 320er Sprites (Paletten ppal, cap_p*, spal): Kugel 1 Glanz, 2-12 Koerper; Schlaeger Zeile -> 1..10; Muenze 1 Glanz,
2-8 Koerper, 10 Rand, 11 Buchstabe; Schuss 11 weiss, 12 gelb, 13 orange (Palette der Sprites 6 und 7 siehe hpal_*: Schlaegerrampe + Schussfarben).
Aufruf im Ordner examples/mega65/prallball:  python3 tools/gen_sprites640.py"""
import math, os
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: egal; die Ausgabedatei wird relativ zum Ort des Skripts (../lib) geschrieben. Liest tools/gen_sprites16.py (Paletten und Hilfsfunktionen).', epilog='Beispiele:\n  python3 examples/mega65/prallball/tools/gen_sprites640.py\n  cd examples/mega65/prallball && python3 tools/gen_sprites640.py\n\nSchreibt (im Projektordner examples/mega65/prallball, vorhandene Dateien werden ueberschrieben): lib/sprites640.inc.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
sys_path = os.path.dirname(os.path.abspath(__file__))
exec(open(os.path.join(sys_path, 'gen_sprites16.py')).read().split("d = pack(ball(6))")[0])     # RAMP, ball(), PADP, sw() ...


def pack16(rows):
    """16 Zeilen * 16 Spalten -> 128 Byte (4 Bit je Punkt, erstes Pixel = hohes Halbbyte)"""
    data = []
    for y in range(16):
        row = rows[y] if y < len(rows) else []
        row = (row + [0] * 16)[:16]
        for x in range(0, 16, 2):
            data.append(row[x] * 16 + row[x + 1])
    return data


def blank():
    return [[0] * 16 for _ in range(16)]


def ball12():
    rows = blank()
    b = ball(12)
    for y in range(12):
        for x in range(12):
            rows[y + 2][x + 2] = b[y][x]
    return rows


PH = 12


def paddle12(left_round, right_round):
    rows = blank()
    r = PH / 2.0
    for y in range(PH):
        ins = int(round(r - math.sqrt(max(0.0, r * r - (y + 0.5 - r) ** 2))))
        idx = 1 + (y * 9) // (PH - 1)
        for x in range(16):
            if left_round and x < ins:
                continue
            if right_round and x >= 16 - ins:
                continue
            rows[y][x] = idx
    return rows


def coin16(h, y0, w=16):
    """Muenze w Spalten breit, h Zeilen ab Zeile y0 (Mitte der 16 x 16 Flaeche); Licht von links oben; 1 Glanz, 2-8 Koerper, 10 Rand."""
    rows = blank()
    cx = (w - 1) / 2.0
    cy = y0 + (h - 1) / 2.0
    for y in range(y0, y0 + h):
        for x in range(w):
            dx = (x - cx) / (w / 2.0)
            dy = (y - cy) / (h / 2.0)
            d2 = dx * dx + dy * dy
            if d2 > 1.0:
                continue
            d = math.sqrt(d2)
            if h <= 4:
                rows[y][x] = (2, 1, 5, 6)[min(3, y - y0)]
                continue
            lum = max(-1.0, min(1.0, -0.5 * dx - 0.6 * dy + 0.3))
            if d > 0.88:
                rows[y][x] = 10
            elif lum > 0.78 and d < 0.7:
                rows[y][x] = 1
            else:
                rows[y][x] = max(2, min(8, 2 + int((1 - lum) * 3.2)))
    return rows


def shot16():
    rows = blank()
    for y in range(16):
        for x in range(4):
            if x in (1, 2):
                rows[y][x] = 11 if y < 4 else (12 if y < 14 else 13)
            elif 2 <= y <= 13:
                rows[y][x] = 13
    return rows


blocks = [('hs_ball', 'HP_BALL', ball12(), 'Kugel 12 x 12 in der Mitte der 16 x 16 Flaeche'),
          ('hs_padl', 'HP_PADL', paddle12(True, False), 'Schlaeger links (linkes Ende rund), 16 x 12'),
          ('hs_padm', 'HP_PADM', paddle12(False, False), 'Schlaeger Mitte (mit X-Verdopplung 32 Punkte breit), 16 x 12'),
          ('hs_padr', 'HP_PADR', paddle12(False, True), 'Schlaeger rechts (rechtes Ende rund), 16 x 12'),
          ('hs_cap', 'HP_CAP', coin16(16, 0), 'Muenze vorn 16 x 16 (der Buchstabe wird zur Laufzeit eingetragen)'),
          ('hs_capa', 'HP_CAPA', coin16(13, 1), 'Muenze gekippt, 13 Zeilen'),
          ('hs_capb', 'HP_CAPB', coin16(9, 3), 'Muenze halb gekippt, 9 Zeilen'),
          ('hs_capc', 'HP_CAPC', coin16(4, 6), 'Muenze von der Kante, 4 Zeilen'),
          ('hs_capbase', 'HP_CAPBASE', coin16(16, 0), 'Muenze vorn ohne Buchstabe (Quelle beim Auswuerfeln)'),
          ('hs_shot', 'HP_SHOT', shot16(), 'Laserschuss 4 x 16')]
out = ['; ERZEUGT von tools/gen_sprites640.py: 16-Farb-Sprites fuer 640 x 400 (je 128 Byte), ab $8000 in der VIC-Bank 2.', '.org $8000']
ptr = 0
for label, const, rows, comment in blocks:
    d = pack16(rows)
    out.append('.const %s = %d                         ; Zeiger (Block ab $8000 + 64 * Zeiger)' % (const, ptr))
    out.append('%s:   ; %s' % (label, comment))
    for k in range(0, 128, 16):
        out.append('        .byte ' + ', '.join('$%02x' % v for v in d[k:k + 16]))
    ptr += 2
# Palette der Sprites 6 und 7 in der hohen Aufloesung: Schlaegerrampe (Eintraege 0-10), Schuss 11 weiss, 12 gelb, 13 orange
HP = list(PADP)
HP[11], HP[12], HP[13] = (255, 255, 255), (255, 232, 60), (255, 150, 30)
out.append('; Palette der Sprites 6 und 7 in 640 x 400 (Halbbytes vertauscht): Schlaegerrampe und Schussfarben')
out.append('hpal_r: .byte ' + ', '.join('$%02x' % sw(c[0]) for c in HP))
out.append('hpal_g: .byte ' + ', '.join('$%02x' % sw(c[1]) for c in HP))
out.append('hpal_b: .byte ' + ', '.join('$%02x' % sw(c[2]) for c in HP))
assert ptr * 64 < 0x7f8
open(os.path.join(sys_path, '..', 'lib', 'sprites640.inc'), 'w', newline='\n').write('\n'.join(out) + '\n')
print('lib/sprites640.inc:', ptr * 64, 'Byte')
