#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Gemeinsame Grafikdaten und Spielwege für das Entwicklungsbeispiel "Hugo & Gumbo" (examples/*/jumprun):
Kacheln (16x16, Rollenbuchstaben), Held und Gegner (Mehrfarb-Basis), Bilder (20x12 Kacheln), Autopilot-Wege mit Prüfung."""
#
# WIE DIESES MODUL BENUTZT WIRD
#   Es ist die gemeinsame QUELLE aller Kunstdaten des Beispiels. Die Generatoren (gen_jumprun_c64.py und die Generatoren der anderen
#   Systeme) lesen von hier
#     * die Kacheln (TILES) und die Bildkarten (LEVELS): jedes Bild ist ein Raster aus 12 Zeilen x 20 Kacheln, Zeile 0 ist die Statuszeile,
#     * Held und Gegner (HERO_FRAMES, ENEMY_FRAMES): kleine Pixelbilder aus Ziffern, 0 = durchsichtig, 1/2/3 = drei Farben,
#     * die Wege des Autopiloten (PATHS) mit ihren Pruefungen,
#   und schreiben daraus die Projektdateien der Editoren (JSON) und die Tabellen fuer den Assembler (lib/levels.inc).
#   So entsteht nichts doppelt, und alle Systeme zeigen dieselben Figuren und Bilder.
#
# BILDER (Kacheln): Jede Kachel ist 16 x 16 Bildpunkte. Sie wird als Funktion beschrieben, die fuer jeden Punkt (x, y) einen "Rollenbuchstaben"
#   liefert (g = Gras, e = Erde, w = Weiss ...). Welche Farbe ein Buchstabe auf dem jeweiligen System bekommt und wie viele Farben je
#   Zeichen erlaubt sind, entscheidet der Generator des Systems (der C64 darf je Zeichen nur 4 Farben: Hintergrund, zwei gemeinsame
#   Mehrfarben und eine Zeichenfarbe; siehe die Abbildungstabellen in TILES).
#
# FIGUREN: Der Held ist 12 Mehrfarbpixel breit (= 24 Bildpunkte) und 19 Zeilen hoch (HEAD + LEGS). Fuer jede Pose gibt es ein Bild; die zweite Haelfte
#   des Spritesatzes sind dieselben Bilder gespiegelt (die Hardware kann nicht spiegeln). Der Umriss und Auge/Mund kommen aus
#   figure_overlay/overlay_details als zweite Ebene (Hires-Overlay).
#
# AUTOPILOT: Ein Weg (Path) beschreibt in Kommandos wie walk_to/climb_to/jump, was der Held tut. Dabei rechnet das Modul mit denselben Zahlen wie die
#   Spiellogik (1,5 Punkte je Bild beim Gehen, 32 Bilder je Sprung) und prueft nach jedem Sprung mit assert, dass der Held auf einer tragenden
#   Kachel landet. Faellt ein Weg durch, bricht die Erzeugung mit einer Meldung ab - es entsteht kein Spiel, das den Helden ins Leere springen laesst.
import math

# ---------------------------------------------------------------- 5x7-Schrift (Zeichensatz-Zeichen)
FONT = {}
def _g(ch, *rows): FONT[ch] = rows
_g('A',"01110","10001","10001","11111","10001","10001","10001"); _g('B',"11110","10001","10001","11110","10001","10001","11110")
_g('C',"01110","10001","10000","10000","10000","10001","01110"); _g('D',"11110","10001","10001","10001","10001","10001","11110")
_g('E',"11111","10000","10000","11110","10000","10000","11111"); _g('F',"11111","10000","10000","11110","10000","10000","10000")
_g('G',"01110","10001","10000","10111","10001","10001","01111"); _g('H',"10001","10001","10001","11111","10001","10001","10001")
_g('I',"01110","00100","00100","00100","00100","00100","01110"); _g('J',"00111","00010","00010","00010","00010","10010","01100")
_g('K',"10001","10010","10100","11000","10100","10010","10001"); _g('L',"10000","10000","10000","10000","10000","10000","11111")
_g('M',"10001","11011","10101","10101","10001","10001","10001"); _g('N',"10001","11001","10101","10011","10001","10001","10001")
_g('O',"01110","10001","10001","10001","10001","10001","01110"); _g('P',"11110","10001","10001","11110","10000","10000","10000")
_g('Q',"01110","10001","10001","10001","10101","10010","01101"); _g('R',"11110","10001","10001","11110","10100","10010","10001")
_g('S',"01111","10000","10000","01110","00001","00001","11110"); _g('T',"11111","00100","00100","00100","00100","00100","00100")
_g('U',"10001","10001","10001","10001","10001","10001","01110"); _g('V',"10001","10001","10001","10001","10001","01010","00100")
_g('W',"10001","10001","10001","10101","10101","11011","10001"); _g('X',"10001","10001","01010","00100","01010","10001","10001")
_g('Y',"10001","10001","01010","00100","00100","00100","00100"); _g('Z',"11111","00001","00010","00100","01000","10000","11111")
_g('0',"01110","10001","10011","10101","11001","10001","01110"); _g('1',"00100","01100","00100","00100","00100","00100","01110")
_g('2',"01110","10001","00001","00010","00100","01000","11111"); _g('3',"11110","00001","00001","01110","00001","00001","11110")
_g('4',"00010","00110","01010","10010","11111","00010","00010"); _g('5',"11111","10000","11110","00001","00001","10001","01110")
_g('6',"00110","01000","10000","11110","10001","10001","01110"); _g('7',"11111","00001","00010","00100","01000","01000","01000")
_g('8',"01110","10001","10001","01110","10001","10001","01110"); _g('9',"01110","10001","10001","01111","00001","00010","01100")
_g(' ',"00000","00000","00000","00000","00000","00000","00000"); _g('-',"00000","00000","00000","11111","00000","00000","00000")
_g('&',"01100","10010","10100","01000","10101","10010","01101"); _g('/',"00001","00010","00010","00100","01000","01000","10000")
_g(':',"00000","00100","00000","00000","00100","00000","00000"); _g('.',"00000","00000","00000","00000","00000","01100","01100")
_g('!',"00100","00100","00100","00100","00100","00000","00100"); _g('+',"00000","00100","00100","11111","00100","00100","00000")
_g('*',"00000","10101","01110","11111","01110","10101","00000")
FONT_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789&/-:.!+* "   # Reihenfolge im Zeichensatz (nach den Kachelzeichen)

# ---------------------------------------------------------------- Schrift fuer Mehrfarb-Zeichen mit nur 4 Punkten Breite (Atari, Plus/4)
# Jedes Zeichen ist 4 Punkte breit (ein Punkt = 2 Bildschirmpixel), 7 Zeilen; die meisten Buchstaben sind 3 Punkte breit mit einer Luecke rechts,
# M, N und W nutzen alle 4 Punkte.  Entworfen so, dass N/M/H, O/0/D/Q und B/8 unterscheidbar bleiben.
FONT4 = {}
def _f4(ch, rows): FONT4[ch] = [r.ljust(4, '.') for r in rows]
for _ch, _rows in {
 'A': ".X. X.X X.X XXX X.X X.X X.X", 'B': "XX. X.X X.X XX. X.X X.X XX.", 'C': ".XX X.. X.. X.. X.. X.. .XX", 'D': "XX. X.X X.X X.X X.X X.X XX.",
 'E': "XXX X.. X.. XX. X.. X.. XXX", 'F': "XXX X.. X.. XX. X.. X.. X..", 'G': ".XX X.. X.. X.X X.X X.X .XX", 'H': "X.X X.X X.X XXX X.X X.X X.X",
 'I': "XXX .X. .X. .X. .X. .X. XXX", 'J': "..X ..X ..X ..X ..X X.X .X.", 'K': "X.X X.X XX. XX. XX. X.X X.X", 'L': "X.. X.. X.. X.. X.. X.. XXX",
 'M': "X..X XXXX XXXX X..X X..X X..X X..X", 'N': "X..X XX.X XX.X X.XX X.XX X..X X..X", 'O': ".X. X.X X.X X.X X.X X.X .X.", 'P': "XX. X.X X.X XX. X.. X.. X..",
 'Q': ".X. X.X X.X X.X XXX XX. .XX", 'R': "XX. X.X X.X XX. XX. X.X X.X", 'S': ".XX X.. X.. .X. ..X ..X XX.", 'T': "XXX .X. .X. .X. .X. .X. .X.",
 'U': "X.X X.X X.X X.X X.X X.X XXX", 'V': "X.X X.X X.X X.X X.X .X. .X.", 'W': "X..X X..X X..X X..X XXXX XXXX X..X", 'X': "X.X X.X .X. .X. .X. X.X X.X",
 'Y': "X.X X.X X.X .X. .X. .X. .X.", 'Z': "XXX ..X ..X .X. X.. X.. XXX",
 '0': "XXX X.X X.X X.X X.X X.X XXX", '1': ".X. XX. .X. .X. .X. .X. XXX", '2': "XX. ..X ..X .X. X.. X.. XXX", '3': "XX. ..X ..X .X. ..X ..X XX.",
 '4': "X.X X.X X.X XXX ..X ..X ..X", '5': "XXX X.. X.. XX. ..X ..X XX.", '6': ".XX X.. X.. XX. X.X X.X .X.", '7': "XXX ..X ..X .X. .X. .X. .X.",
 '8': ".X. X.X X.X .X. X.X X.X .X.", '9': ".X. X.X X.X .XX ..X ..X XX.",
 '&': ".X. X.X .X. X.X X.X X.X .XX", '/': "..X ..X .X. .X. X.. X.. X..", '-': "... ... ... XXX ... ... ...", ':': "... .X. ... ... ... .X. ...",
 '.': "... ... ... ... ... ... .X.", '!': ".X. .X. .X. .X. .X. ... .X.", '+': "... .X. .X. XXX .X. .X. ...", '*': "... X.X .X. XXX .X. X.X ...",
 ' ': "... ... ... ... ... ... ...",
}.items(): _f4(_ch, _rows.split())
assert all(c in FONT4 for c in FONT_CHARS)

# ---------------------------------------------------------------- Kacheln (16x16, Rollenbuchstaben)
def _h(x, y, s=0): return ((x * 73856093) ^ (y * 19349663) ^ (s * 83492791)) & 0xffff
def t_grass(x, y):
    if y < 2: return 'g' if (x + (y * 3)) % 4 else 'G'
    if y < 4: return 'G' if _h(x // 2, y) % 3 else 'g'
    return 'e' if _h(x, y) % 7 else 'E'
def t_dirt(x, y): return 'E' if _h(x // 2, y // 2, 3) % 4 == 0 else ('s' if _h(x, y, 5) % 23 == 0 else 'e')
def t_brick(x, y):
    row = y // 4; off = 8 if row % 2 else 0
    if y % 4 == 3: return 'E'
    if (x + off) % 16 in (15, 0): return 'E'
    return 'o' if y % 4 == 0 else 'r'
def t_stone(x, y):
    if x == 0 or y == 0: return 'L'
    if x == 15 or y == 15: return 'S'
    return 's' if _h(x // 3, y // 3, 7) % 5 else 'S'
def t_ladder(x, y):
    if x in (2, 3, 12, 13): return 'e' if x in (2, 12) else 'E'
    if y % 5 == 1 and 4 <= x <= 11: return 'e' if y % 2 else 'o'
    return '.'
def t_wood(x, y):
    if y < 2: return 'o'
    if y < 6: return 'e' if (x + y) % 8 else 'E'
    if y == 6: return 'E'
    return '.'
def t_cloud(x, y):
    cx, cy = abs(x - 7.5), abs(y - 8.5)
    d = (cx / 7.5) ** 2 + (cy / 4.2) ** 2
    if y > 9 and d < 1.1: return 'L'
    if d < 0.8: return 'w'
    if d < 1.0: return 'L'
    return '.'
def t_cloudfloor(x, y):          # tragende Wolke: oben flach
    if y < 3: return 'w'
    if y < 7: return 'w' if _h(x // 3, y) % 5 else 'L'
    return 'L' if y < 10 else '.'
def t_water(x, y):
    w = (x + 2 * math.sin(y * 1.3 + x * 0.5)) % 8
    if y < 2: return 'c' if int(x + y) % 4 else 'w'
    return 'b' if w > 3 else 'B'
def t_door(x, y):
    if x < 2 or x > 13 or y < 1: return 'E'
    if 9 <= y <= 10 and 10 <= x <= 12: return 'y'
    return 'e' if y % 6 else 'E'
def t_coin(x, y):
    d = (x - 7.5) ** 2 + (y - 7.5) ** 2
    if d < 6: return 'w' if (x < 7 and y < 7) else 'y'
    if d < 22: return 'y' if x + y < 16 else 'o'
    if d < 30: return 'o'
    return '.'
def t_bush(x, y):
    d1 = (x - 5) ** 2 + (y - 10) ** 2; d2 = (x - 11) ** 2 + (y - 10) ** 2; d3 = (x - 8) ** 2 + (y - 6) ** 2
    if min(d1, d2, d3) < 30: return 'g' if _h(x, y, 2) % 3 else 'G'
    return '.'
def t_flower(x, y):
    if x in (7, 8) and y >= 8: return 'G'
    if abs(x - 7.5) + abs(y - 6) < 3.5 and not (abs(x - 7.5) + abs(y - 6) < 1.5): return 'r'
    if abs(x - 7.5) + abs(y - 6) < 1.5: return 'y'
    if y == 12 and x in (4, 5, 10, 11): return 'G'
    return '.'
def t_cave(x, y): return 'S' if _h(x // 4, y // 4, 9) % 6 == 0 else 'k'
def t_torch(x, y):
    if 6 <= x <= 9 and y >= 9: return 'e'
    d = (x - 7.5) ** 2 + ((y - 6) * 1.3) ** 2
    if d < 5: return 'y'
    if d < 14: return 'o'
    return 'k'
def t_crystal(x, y):
    if abs(x - 7.5) + abs(y - 9) * 0.7 < 6 and y > 3: return 'w' if x < 7 else ('c' if (x + y) % 5 else 'v')
    return 'k'
def t_batt(x, y):
    if y < 6 and 4 <= x <= 11 or y < 6 and (x < 3 or x > 12): return '.'
    if y < 6: return 'L' if x < 7 else 's'
    if y == 6: return 'L'
    return 'S' if _h(x // 4, y // 4, 4) % 4 == 0 else 's'
def t_flag(x, y):
    if x == 3 and 1 <= y: return 'e'
    if 4 <= x <= 12 and 1 <= y <= 7 and y - 1 < 7 - abs(x - 4) * 0.4: return 'r' if (x + y) % 5 else 'w'
    return '.'
def t_moon(x, y):
    d = (x - 8) ** 2 + (y - 8) ** 2; d2 = (x - 11) ** 2 + (y - 6) ** 2
    if d < 40 and d2 > 28: return 'y' if d < 30 else 'w'
    return '.'
def t_star(x, y):
    if (x, y) in ((7, 7), (8, 7), (7, 8), (8, 8)): return 'w'
    if (x, y) in ((6, 7), (9, 8), (7, 6), (8, 9)): return 'y'
    return '.'

# Kachelsymbol -> (Name, Zeichnen, Attribut 0 leer/1 tragend/2 Leiter, Mehrfarb-Abbildung Rolle->Wert (0 Hintergrund, 1/2 Mehrfarb, 3 Zeichenfarbe), Zeichenfarbe 0-7)
TILES = [
 ('.', 'leer', None, 0, {}, 0),
 ('G', 'gras', t_grass, 1, {'g': 3, 'G': 3, 'e': 1, 'E': 2}, 5),
 ('D', 'erde', t_dirt, 1, {'e': 1, 'E': 2, 's': 3}, 1),
 ('B', 'ziegel', t_brick, 1, {'r': 2, 'E': 1, 'o': 3}, 1),
 ('S', 'stein', t_stone, 1, {'L': 3, 'S': 1, 's': 2}, 1),
 ('H', 'leiter', t_ladder, 2, {'e': 2, 'E': 1, 'o': 3}, 7),
 ('P', 'holz', t_wood, 1, {'o': 3, 'e': 2, 'E': 1}, 7),
 ('C', 'wolke', t_cloud, 0, {'w': 3, 'L': 2}, 1),
 ('c', 'wolkenboden', t_cloudfloor, 1, {'w': 3, 'L': 2}, 1),
 ('W', 'wasser', t_water, 0, {'c': 3, 'w': 3, 'b': 3, 'B': 1}, 6),
 ('E', 'tuer', t_door, 0, {'E': 1, 'e': 2, 'y': 3}, 7),
 ('o', 'muenze', t_coin, 0, {'y': 3, 'w': 3, 'o': 2}, 7),
 ('b', 'busch', t_bush, 0, {'g': 3, 'G': 1}, 5),
 ('f', 'blume', t_flower, 0, {'G': 1, 'r': 3, 'y': 2}, 2),
 ('k', 'hoehle', t_cave, 0, {'S': 1, 'k': 0}, 1),
 ('T', 'fackel', t_torch, 0, {'e': 2, 'y': 3, 'o': 2, 'k': 0}, 7),
 ('x', 'kristall', t_crystal, 0, {'w': 3, 'c': 3, 'v': 2, 'k': 0}, 3),
 ('M', 'zinne', t_batt, 0, {'L': 3, 's': 2, 'S': 1}, 1),
 ('F', 'fahne', t_flag, 0, {'e': 1, 'r': 3, 'w': 2}, 2),
 ('m', 'mond', t_moon, 0, {'y': 3, 'w': 2}, 7),
 ('*', 'stern', t_star, 0, {'w': 3, 'y': 3}, 1),
]
TILE_INDEX = {t[0]: i for i, t in enumerate(TILES)}
def tile_grid(sym):
    fn = TILES[TILE_INDEX[sym]][2]
    return [['.'] * 16 for _ in range(16)] if fn is None else [[fn(x, y) for x in range(16)] for y in range(16)]
def tile_chars_mc(sym):
    """4 Zeichen (oben links, oben rechts, unten links, unten rechts) als je 8 Zeilen zu 4 Mehrfarb-Pixeln (Ziffern 0-3)"""
    m = TILES[TILE_INDEX[sym]][4]; g = tile_grid(sym); out = []
    for cy in (0, 1):
        for cx in (0, 1):
            rows = []
            for y in range(8):
                rows.append(''.join(str(m.get(g[cy * 8 + y][cx * 8 + x * 2], 0)) for x in range(4)))
            out.append(rows)
    return out
def font_char_mc(ch):
    """Schriftzeichen als Hires-Zeichen (Farbspeicher < 8): je 8 Bit in 4 Zweierpaare zerlegt"""
    rows = FONT.get(ch, FONT[' ']); out = []
    for y in range(8):
        bits = (rows[y] if y < 7 else '00000') + '000'
        out.append(''.join(str(int(bits[i * 2 : i * 2 + 2], 2)) for i in range(4)))
    return out

# ---------------------------------------------------------------- Bilder (12 Zeilen x 20 Kacheln; Zeile 0 = Statuszeile)
def _grid(fill='.'): return [[fill] * 20 for _ in range(12)]
def _put(m, c, k, ch):
    if 0 <= c < 20 and 0 <= k < 12: m[k][c] = ch
def _hl(m, c0, c1, k, ch):
    for c in range(c0, c1 + 1): _put(m, c, k, ch)
def _vl(m, c, k0, k1, ch):
    for k in range(k0, k1 + 1): _put(m, c, k, ch)
def level1():
    m = _grid(); _hl(m, 0, 19, 10, 'G'); _hl(m, 0, 19, 11, 'D')
    for c, k in ((2, 2), (11, 1), (16, 3)): _put(m, c, k, 'C')
    _hl(m, 3, 7, 7, 'P'); _vl(m, 6, 7, 9, 'H'); _hl(m, 10, 13, 5, 'P'); _hl(m, 15, 17, 7, 'P'); _vl(m, 14, 7, 9, 'H')
    for c, k, ch in ((1, 9, 'b'), (9, 9, 'f'), (10, 9, 'f'), (18, 8, 'E'), (18, 9, 'E'), (11, 4, 'o'), (12, 4, 'o'), (4, 6, 'o'), (16, 6, 'o'), (8, 9, 'o')): _put(m, c, k, ch)
    return m
def level2():
    m = _grid('k'); _hl(m, 0, 19, 1, 'S'); _hl(m, 0, 19, 10, 'S'); _hl(m, 0, 19, 11, 'S')
    _hl(m, 5, 8, 8, 'S'); _vl(m, 4, 8, 9, 'H'); _hl(m, 11, 15, 6, 'S'); _vl(m, 10, 6, 9, 'H'); _hl(m, 17, 19, 8, 'S')
    for c, k, ch in ((2, 9, 'T'), (16, 9, 'T'), (6, 7, 'x'), (14, 5, 'x'), (18, 6, 'E'), (18, 7, 'E'), (8, 7, 'o'), (13, 5, 'o'), (12, 9, 'W'), (13, 9, 'W'), (1, 9, 'x')): _put(m, c, k, ch)
    return m
def level3():
    m = _grid(); _hl(m, 0, 19, 10, 'B'); _hl(m, 0, 19, 11, 'B'); _hl(m, 0, 19, 6, 'B'); _hl(m, 0, 19, 3, 'B')
    for c in (1, 5, 9, 13, 17): _put(m, c, 2, 'M')
    _vl(m, 8, 6, 9, 'H'); _put(m, 11, 5, 'S')
    for c, k, ch in ((2, 1, 'F'), (10, 1, 'F'), (18, 1, 'F'), (18, 4, 'E'), (18, 5, 'E'), (15, 5, 'o'), (13, 5, 'o'), (5, 5, 'o')): _put(m, c, k, ch)
    return m
def level4():
    m = _grid()
    for c, k in ((2, 2), (7, 4), (13, 3), (17, 6)): _put(m, c, k, 'C')
    for c, k in ((16, 1), ): _put(m, c, k, 'm')
    for c, k in ((3, 5), (9, 2), (12, 1), (6, 1)): _put(m, c, k, '*')
    _hl(m, 0, 6, 10, 'c'); _vl(m, 6, 8, 9, 'H'); _hl(m, 7, 10, 8, 'c'); _hl(m, 12, 15, 6, 'c'); _hl(m, 16, 19, 4, 'c')
    for c, k, ch in ((18, 2, 'E'), (18, 3, 'E'), (9, 7, 'o'), (13, 5, 'o'), (17, 3, 'o')): _put(m, c, k, ch)
    return m
LEVELS = [level1, level2, level3, level4]
LEVEL_NAMES = ['WIESE - SONNIGER ANFANG', 'HOEHLE - GUMBO KLETTERT', 'BURG - LEITERN AUF UND AB', 'WOLKEN - DAS ZIEL IN SICHT']
def tile_attr(sym): return TILES[TILE_INDEX[sym]][3]

# ---------------------------------------------------------------- Figuren
HEAD = ["000011100000", "000111111000", "001111111100", "001122222110", "001222222200", "001222322200", "000222222000", "000022220000",
        "000111111000", "001111111100", "012211111220", "012211111220", "002011110200"]
LEGS = {
 'stand': ["000330033000"] * 6 + ["003330033300"] * 2,
 'walkA': ["000333330000", "000330033000", "003300003300", "003300003300", "033000000330", "033000000330", "330000000033", "333000000033"],
 'walkB': ["000333330000", "000333330000", "000333330000", "000033300000", "000033300000", "000033300000", "000333330000", "000333330000"],
 'walkC': ["000333330000", "000330033000", "000330033000", "003300003300", "003300003300", "033000000330", "033000000330", "333000000333"],
 'jump': ["000333330000", "003333333000", "033303033300", "033000003300", "000000000000", "000000000000", "000000000000", "000000000000"],
 'climbA': ["000333330000", "000330000000", "000330000000", "000033000000", "000033000000", "000033300000", "000333300000", "003333000000"],
 'climbB': ["000333330000", "000000033000", "000000033000", "000000330000", "000000330000", "000003330000", "000003333000", "000003333300"],
}
HERO_FRAMES = ['stand', 'walkA', 'walkB', 'walkC', 'jump', 'climbA', 'climbB']
def hero_frame(name):
    rows = HEAD[:7] + HEAD[8:]
    if name.startswith('climb'):
        rows = list(rows); rows[9] = "112211111211"; rows[10] = "122211111221"; rows[11] = "100011110001"
    full = rows + LEGS[name][1:8]
    assert all(len(r) == 12 for r in full) and len(full) == 19
    return full
ENEMY = ["000011110000", "000111111000", "001111111100", "011111111110", "011311131110", "011311131110", "011111111110", "011122221110", "011111111110", "001111111100"]
ENEMY_FRAMES = ['a', 'b', 'c']
def enemy_frame(name):
    rows = list(ENEMY)
    rows += {'a': ["000330033000", "000330033000", "003330033300"], 'b': ["003300003300", "003300003300", "033300003330"], 'c': ["000333330000", "000333330000", "003333333300"]}[name]
    assert all(len(r) == 12 for r in rows) and len(rows) == 13
    return rows
def mirror(rows): return [r[::-1] for r in rows]
def figure_overlay(rows, who):
    """Hires-Overlay (24x21): 1-Pixel-Umriss um die Basis (Basis liegt in den Zeilen 1..n, jeder Punkt 2 Pixel breit) + Gesichtsdetails"""
    h = len(rows); base = [[0] * 24 for _ in range(21)]
    for y in range(h):
        for x in range(12):
            if rows[y][x] != '0': base[y + 1][x * 2] = 1; base[y + 1][x * 2 + 1] = 1
    ov = [[0] * 24 for _ in range(21)]
    for y in range(21):
        for x in range(24):
            if base[y][x]: continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                xx, yy = x + dx, y + dy
                if 0 <= xx < 24 and 0 <= yy < 21 and base[yy][xx]: ov[y][x] = 1; break
    return ov
def overlay_details(ov, rows, who, flip):
    """Augen und Mund (nur der Hires-Overlay zeichnet so fein)"""
    if who == 'hero':
        ex = 14 if not flip else 8
        ov[7][ex] = ov[7][ex + 1] = 1                    # Auge (Basiszeile 5 + 1)
        ov[9][ex + (1 if not flip else -1)] = 1          # Mund
    else:
        for ex in (8, 14): ov[7][ex] = ov[7][ex + 1] = 1
    return ov
def sprite_rows_mc(rows, top=1):
    out = [['0'] * 12 for _ in range(21)]
    for y, r in enumerate(rows):
        for x, ch in enumerate(r): out[y + top][x] = ch
    return [''.join(r) for r in out]
def sprite_frames(who):
    """Liste (Basis-MC-Zeilen 21x12, Overlay-Zeilen 21x24) für alle Bilder: erst rechts blickend, dann gespiegelt"""
    names = HERO_FRAMES if who == 'hero' else ENEMY_FRAMES
    out = []
    for flip in (False, True):
        for n in names:
            rows = hero_frame(n) if who == 'hero' else enemy_frame(n)
            if flip: rows = mirror(rows)
            ov = figure_overlay(rows, who); overlay_details(ov, rows, who, flip)
            out.append((sprite_rows_mc(rows), [''.join(str(v) for v in r) for r in ov]))
    return out

# ---------------------------------------------------------------- Autopilot-Wege (logische Koordinaten 320x200, Fuß-Y = Oberkante der Kachelzeile)
# Befehle: WALK dir n | JUMP dir dx dy | CLIMB up(0)/down(1) n | WAIT n | SFX k | COIN col k | END
SPEED_NUM = 3    # Held geht 1,5 Punkte je Bild
class Hero:
    def __init__(self, x, y, d): self.x, self.y, self.d, self.cmds = x, y, d, []
class Path:
    def __init__(self, level_fn, x, y, d, ex, ey, ed):
        self.map = level_fn(); self.h = Hero(x, y, d); self.start = (x, y, d, ex, ey, ed)
    def tile(self, col, k): return self.map[k][col] if 0 <= col < 20 and 0 <= k < 12 else '.'
    def standable(self): return tile_attr(self.tile((self.h.x + 12) // 16, self.h.y // 16)) == 1
    def wait(self, n): self.h.cmds.append(('WAIT', n))
    def sfx(self, k): self.h.cmds.append(('SFX', k))
    def coin(self, col, k): self.h.cmds.append(('COIN', col, k))
    def walk_to(self, x):
        dx = x - self.h.x; n = round(abs(dx) * 2 / SPEED_NUM); d = 1 if dx < 0 else 0
        self.h.cmds.append(('WALK', d, n)); self.h.d = d; self.h.x += (n * SPEED_NUM) // 2 * (-1 if d else 1) if n else 0
    def climb_to(self, y):
        dy = y - self.h.y; n = abs(dy); self.h.cmds.append(('CLIMB', 1 if dy > 0 else 0, n)); self.h.y = y
    def jump(self, dx, dy):
        d = 1 if dx < 0 else 0; self.h.cmds.append(('JUMP', d, abs(dx), dy)); self.h.d = d; self.h.x += dx; self.h.y += dy
        assert self.standable(), f'Landung nicht auf tragender Kachel: x={self.h.x} y={self.h.y} Kachel={self.tile((self.h.x + 12) // 16, self.h.y // 16)!r}'
    def end(self): self.h.cmds.append(('END',))
def path1():
    p = Path(level1, 16, 160, 0, 0, 160, 0); p.wait(40); p.walk_to(89); p.climb_to(112)
    p.walk_to(100); p.sfx(0); p.jump(70, -32); p.sfx(1); p.walk_to(185); p.coin(11, 4); p.walk_to(200); p.coin(12, 4); p.walk_to(204)
    p.sfx(0); p.jump(46, 32); p.sfx(1); p.walk_to(262); p.sfx(0); p.jump(18, 48); p.sfx(1); p.walk_to(282); p.wait(50); p.end(); return p
def path2():
    p = Path(level2, 16, 160, 0, 0, 160, 0); p.wait(40); p.walk_to(58); p.climb_to(128); p.walk_to(118); p.coin(8, 7)
    p.sfx(0); p.jump(72, -32); p.sfx(1); p.walk_to(206); p.coin(13, 5); p.walk_to(236)
    p.sfx(0); p.jump(48, 32); p.sfx(1); p.wait(50); p.end(); return p
def path3():
    p = Path(level3, 16, 160, 0, 0, 160, 0); p.wait(40); p.walk_to(122); p.climb_to(96); p.walk_to(158)
    p.sfx(0); p.jump(56, 0); p.sfx(1); p.walk_to(236); p.coin(15, 5); p.walk_to(282); p.wait(50); p.end(); return p
def path4():
    p = Path(level4, 16, 160, 0, 0, 160, 0); p.wait(40); p.walk_to(90); p.climb_to(128); p.walk_to(150); p.coin(9, 7)
    p.sfx(0); p.jump(56, -32); p.sfx(1); p.coin(13, 5); p.walk_to(236); p.sfx(0); p.jump(26, -32); p.sfx(1); p.coin(17, 3); p.walk_to(284); p.wait(60); p.end(); return p
PATHS = [path1, path2, path3, path4]
def script_bytes(p):
    """Befehle als Bytes: 1 WALK d n, 2 JUMP d dx dy, 3 CLIMB d n, 4 WAIT n, 5 SFX k, 6 COIN col k, 0 END"""
    out = []
    for c in p.h.cmds:
        if c[0] == 'WALK':
            n = c[2]
            while n > 0: out += [1, c[1], min(255, n)]; n -= 255
        elif c[0] == 'JUMP': out += [2, c[1], c[2], c[3] & 255]
        elif c[0] == 'CLIMB': out += [3, c[1], c[2]]
        elif c[0] == 'WAIT': out += [4, c[1]]
        elif c[0] == 'SFX': out += [5, c[1]]
        elif c[0] == 'COIN': out += [6, c[1], c[2]]
        elif c[0] == 'END': out += [0]
    return out
if __name__ == '__main__':
    import argparse
    _ap = argparse.ArgumentParser(
        description="""Gemeinsame Grafikdaten und Spielwege für das Entwicklungsbeispiel Hugo & Gumbo (examples/*/jumprun): Kacheln, Held und Gegner, Bilder, Autopilot-Wege mit Prüfung.

Das Modul wird von den Generatoren tools/gen_jumprun_*.py eingelesen. Direkt gestartet prüft es die Autopilot-Wege und gibt je Bild eine Zeile aus
(Start, Anzahl Befehle, Bytes, Endposition); es liest und schreibt keine Dateien. Der Arbeitsordner ist egal.""",
        epilog="""Beispiele:
  python3 tools/jumprun_art.py
  cd tools && python3 jumprun_art.py

Das Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.""",
        formatter_class=argparse.RawDescriptionHelpFormatter)
    _ap.parse_args()
    for i, f in enumerate(PATHS):
        p = f(); print('Bild', i + 1, 'Start', p.start, 'Befehle', len(p.h.cmds), 'Bytes', len(script_bytes(p)), 'Ende x/y', p.h.x, p.h.y)
