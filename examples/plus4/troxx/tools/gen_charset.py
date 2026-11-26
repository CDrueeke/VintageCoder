#!/usr/bin/env python3
# gen_charset.py - erzeugt charset.asm (69 eigene Zeichen ab Code 128) fuer Troxx (Plus/4).
# Aufruf im Projektordner: python3 tools/gen_charset.py > charset.asm
# Zeichen: 128..138 Linien und Knoten (vier Punkte dick), 139 Herz, 140 Stern, 141 Eisblock, 142 Auswahlzeiger, 143 Block (Logo, Mehrfarben), 144 Treffer,
# 145..148 Figur, 149..152 Laeufer, 153..156 Streuner, 157..160 Waechter (je vier Bilder), 161..196 Fuellmuster (4 Muster x 9 Lagen im Block, Mehrfarben).
# Mehrfarben-Bitpaare: 00 Hintergrund 0 (dunkel, Schatten), 01 Mehrfarben-Hintergrund 1 (Grund der Fuellung), 10 Mehrfarben-Hintergrund 2 (Glanz),
# 11 Farbe der Zelle (Muster).

def bits(rows):
    return [int(r.replace('.', '0').replace('X', '1'), 2) for r in rows]


out = []


def add(code, name, data):
    out.append((code, name, data))


def arms(n, e, so, w):
    """Linienzeichen: Mitte 4 x 4 Punkte (Zeilen/Spalten 2..5), dazu Arme nach Norden, Osten, Sueden, Westen"""
    g = [[0] * 8 for _ in range(8)]
    for y in range(2, 6):
        for x in range(2, 6):
            g[y][x] = 1
    for y in range(8):
        for x in range(8):
            if n and y < 2 and 2 <= x < 6:
                g[y][x] = 1
            if so and y >= 6 and 2 <= x < 6:
                g[y][x] = 1
            if w and x < 2 and 2 <= y < 6:
                g[y][x] = 1
            if e and x >= 6 and 2 <= y < 6:
                g[y][x] = 1
    return [sum(g[y][x] << (7 - x) for x in range(8)) for y in range(8)]


add(128, 'waagerechte Linie', arms(0, 1, 0, 1))
add(129, 'senkrechte Linie', arms(1, 0, 1, 0))
add(130, 'Kreuzung', arms(1, 1, 1, 1))
add(131, 'Ecke oben links', arms(0, 1, 1, 0))
add(132, 'Ecke oben rechts', arms(0, 0, 1, 1))
add(133, 'Ecke unten links', arms(1, 1, 0, 0))
add(134, 'Ecke unten rechts', arms(1, 0, 0, 1))
add(135, 'T nach unten', arms(0, 1, 1, 1))
add(136, 'T nach oben', arms(1, 1, 0, 1))
add(137, 'T nach rechts', arms(1, 1, 1, 0))
add(138, 'T nach links', arms(1, 0, 1, 1))
add(139, 'Herz', [0x66, 0xff, 0xff, 0xff, 0x7e, 0x3c, 0x18, 0x00])
add(140, 'Stern', [0x18, 0x18, 0xff, 0x7e, 0x3c, 0x3c, 0x66, 0xc3])
add(141, 'Eisblock', [0xff, 0x81, 0xb9, 0xa1, 0x81, 0x81, 0x81, 0xff])
add(142, 'Auswahlzeiger', [0x30, 0x38, 0x3c, 0x3e, 0x3c, 0x38, 0x30, 0x00])


def mc(rows):
    """rows: 8 Zeilen zu 4 Werten 0..3 -> Bytes"""
    return [(r[0] << 6) | (r[1] << 4) | (r[2] << 2) | r[3] for r in rows]


tile = [[2, 2, 2, 2]] + [[2, 3, 3, 3] for _ in range(6)] + [[1, 1, 1, 1]]
add(143, 'abgeschraegter Block (Logo, Mehrfarben)', mc(tile))
add(144, 'Treffer', [0x81, 0x42, 0x24, 0x18, 0x18, 0x24, 0x42, 0x81])

body = ['..XXXX..', '.XXXXXX.', 'XX.XX.XX', 'XXXXXXXX', 'XXXXXXXX', 'XXXXXXXX']
blink = list(body)
blink[2] = 'XXXXXXXX'
legs_a = ['.XXXXXX.', '.X.XX.X.']
legs_b = ['.XXXXXX.', 'X.X..X.X']
add(145, 'Figur Bild 1', bits(body + legs_a))
add(146, 'Figur Bild 2', bits(body + legs_b))
add(147, 'Figur Bild 3 (blinzelt)', bits(blink + legs_a))
add(148, 'Figur Bild 4', bits(body + legs_b))

ra = ['..X..X..', '...XX...', '..XXXX..', '.XX..XX.', 'XXXXXXXX', 'X.XXXX.X', 'X.X..X.X', '...XX...']
rb = ['..X..X..', 'X..XX..X', 'X.XXXX.X', 'XXX..XXX', 'XXXXXXXX', '.XXXXXX.', '..X..X..', '.X....X.']
ra2 = list(ra)
ra2[0] = '.X....X.'
rb2 = list(rb)
rb2[0] = '.X....X.'
add(149, 'Laeufer Bild 1', bits(ra))
add(150, 'Laeufer Bild 2', bits(rb))
add(151, 'Laeufer Bild 3', bits(ra2))
add(152, 'Laeufer Bild 4', bits(rb2))

ga = ['..XXXX..', '.XXXXXX.', 'XX.XX.XX', 'XXXXXXXX', 'XXXXXXXX', 'XXXXXXXX', 'XXXXXXXX', 'XX.XX.XX']
gb = list(ga)
gb[7] = 'X.XXXX.X'
ga2 = list(ga)
ga2[2] = 'XXX.XX.X'
gb2 = list(gb)
gb2[2] = 'XXX.XX.X'
add(153, 'Streuner Bild 1', bits(ga))
add(154, 'Streuner Bild 2', bits(gb))
add(155, 'Streuner Bild 3', bits(ga2))
add(156, 'Streuner Bild 4', bits(gb2))

qa = ['...XX...', '..XXXX..', '.XXXXXX.', 'XX.XX.XX', 'XXXXXXXX', '..X..X..', '.X.XX.X.', 'X.X..X.X']
qb = ['...XX...', '..XXXX..', '.XXXXXX.', 'XX.XX.XX', 'XXXXXXXX', '.X.XX.X.', 'X.X..X.X', '.X....X.']
qa2 = list(qa)
qa2[3] = 'XXX.XX.X'
qb2 = list(qb)
qb2[3] = 'XXX.XX.X'
add(157, 'Waechter Bild 1', bits(qa))
add(158, 'Waechter Bild 2', bits(qb))
add(159, 'Waechter Bild 3', bits(qa2))
add(160, 'Waechter Bild 4', bits(qb2))


def pattern(p):
    g = [[1] * 4 for _ in range(8)]
    for y in range(8):
        for x in range(4):
            if p == 0:      # Schraffur
                g[y][x] = 3 if (x + y) % 4 == 0 else 1
            elif p == 1:    # Punkte
                g[y][x] = 3 if (y % 4 == 1 and x == 1) or (y % 4 == 3 and x == 3) else 1
            elif p == 2:    # Mauer
                if y % 4 == 0:
                    g[y][x] = 2
                elif y < 4:
                    g[y][x] = 1 if x == 3 else 3
                else:
                    g[y][x] = 1 if x == 1 else 3
            else:           # Schachbrett
                g[y][x] = 3 if ((x // 2) + (y // 2)) % 2 == 0 else 1
    return g


names = ['Schraffur', 'Punkte', 'Mauer', 'Schachbrett']
code = 161
for p in range(4):
    for py in range(3):
        for px in range(3):
            g = pattern(p)
            if py == 2:
                g[7] = [0, 0, 0, 0]
            if px == 2:
                for y in range(8):
                    g[y][3] = 0
            if py == 0:
                g[0] = [2, 2, 2, 2]
            if px == 0:
                for y in range(8):
                    g[y][0] = 2
            add(code, 'Fuellung %s, Lage %d,%d (Zeile, Spalte)' % (names[p], py, px), mc(g))
            code += 1

n = len(out)
print('; =====================================================================================================================')
print('; charset.asm - eigene Grafikzeichen ab Zeichencode 128 (%d Zeichen zu 8 Byte)  [erzeugt von tools/gen_charset.py, nicht von Hand aendern]' % n)
print('; =====================================================================================================================')
print('; ZWECK     ted.init kopiert die Zeichen 0..127 aus dem Zeichen-ROM (Text) nach $4000 und diese Zeichen nach $4000 + 128 * 8.')
print(';           Linien sind vier Punkte dick (Zeilen/Spalten 2 bis 5) und einfarbig.  Fuellungen und Logo-Block nutzen Bitpaare (00 Hintergrund 0 = Schatten,')
print(';           01 Mehrfarben-Hintergrund 1 = Grund, 10 Mehrfarben-Hintergrund 2 = Glanz, 11 Farbe der Zelle = Muster) und werden mit Farbspeicher-Bit 3 = 1')
print(';           als Mehrfarben-Zeichen angezeigt.  Jedes Fuellmuster hat neun Zeichen (Lage im 3 x 3 Block): Glanz oben und links, Schatten unten und rechts.')
print('; =====================================================================================================================')
print()
print('GFX:')
for c, name, d in out:
    print('        .byte ' + ','.join('$%02x' % b for b in d) + '   ; %d %s' % (c, name))
print('.assert * - GFX == NGFX * 8, "NGFX stimmt nicht mit den Zeichendaten ueberein"')
assert out[0][0] == 128 and out[-1][0] == 128 + n - 1 and n == 69, n
