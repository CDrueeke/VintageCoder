#!/usr/bin/env python3
# gen_assets.py - erzeugt assets.inc fuer TROXX auf dem MEGA65: Palette (256 Eintraege), Vollfarb-Kacheln (8 x 8, 64 Byte), Sprite-Palette
# (8 Sprites zu je 16 Eintraegen) und 16-Farben-Sprites (16 x 16, 128 Byte).  Aufruf im Projektordner:  python3 tools/gen_assets.py
# Die Ausgabe ist reiner Text (.byte); das Werkzeug wird nur zum Erzeugen gebraucht, nicht zum Bauen.
import colorsys
import sys

# ---------------------------------------------------------------- Palette
pal = [(0, 0, 0)] * 256


def setp(i, rgb):
    pal[i] = tuple(max(0, min(255, int(c))) for c in rgb)


def ramp(base, n, lo=0.28, hi=1.0, lift=0.0):
    """n Farben von dunkel nach hell aus der Grundfarbe (helle Stufen werden nach Weiss aufgehellt)."""
    out = []
    for s in range(n):
        f = lo + (hi - lo) * s / max(1, n - 1)
        c = [base[0] * f, base[1] * f, base[2] * f]
        if f > 0.8:
            t = (f - 0.8) / 0.2 * lift
            c = [x + (255 - x) * t for x in c]
        out.append(tuple(c))
    return out


# 0..15: Grundfarben (Reihenfolge wie VIC-II); 0 = dunkles Nachtblau als Hintergrund
C64 = [(10, 10, 28), (240, 240, 240), (190, 60, 60), (110, 230, 230), (190, 80, 200), (70, 200, 90), (60, 70, 210), (255, 225, 60),
       (240, 140, 50), (130, 90, 40), (250, 130, 130), (80, 80, 100), (140, 140, 160), (150, 255, 160), (130, 150, 255), (200, 200, 210)]
for i, c in enumerate(C64):
    setp(i, c)

# 16..24: Linienfarben (hell, mittel, dunkel): unberuehrt blau, Spieler 1 gelb, Spieler 2 cyan
LINE = {0: [(120, 130, 240), (62, 74, 200), (30, 36, 120)],
        1: [(255, 250, 170), (255, 222, 50), (205, 150, 0)],
        2: [(200, 255, 255), (70, 235, 235), (0, 150, 175)],
        3: [(255, 255, 255), (235, 235, 255), (190, 190, 235)]}
for st in range(4):
    for j in range(3):
        setp(16 + 3 * st + j, LINE[st][j])

# 32..87: Fuellfarben (7 Farben zu je 8 Stufen)
FILLC = [(40, 190, 90), (40, 170, 220), (140, 90, 220), (225, 70, 80), (225, 170, 40), (70, 130, 235), (240, 120, 40)]
FILL0 = 32
for c, base in enumerate(FILLC):
    for s, rgb in enumerate(ramp(base, 8, 0.25, 1.0, 0.45)):
        setp(FILL0 + 8 * c + s, rgb)

# 96..104: Tafeln (je Thema: Hintergrund, Rahmen hell, Rahmen dunkel): Spieler 1 braun/gelb, Spieler 2 petrol/cyan, Stufe blau
PANEL = [((32, 28, 14), (200, 160, 40), (110, 85, 20)), ((10, 32, 36), (60, 190, 200), (24, 100, 110)), ((14, 18, 44), (110, 120, 220), (50, 56, 130))]
PAN0 = 96
for t, (bg, hi, lo) in enumerate(PANEL):
    setp(PAN0 + 3 * t, bg)
    setp(PAN0 + 3 * t + 1, hi)
    setp(PAN0 + 3 * t + 2, lo)
# 112..117: Fortschrittsbalken, Herz
setp(112, (60, 220, 110))
setp(113, (30, 130, 70))
setp(114, (22, 26, 40))
setp(115, (235, 70, 90))
setp(116, (150, 30, 50))
setp(117, (255, 190, 200))
# 120..127: Glanz der Fuellanimation (hell bis fast weiss)
for s in range(8):
    f = s / 7
    setp(120 + s, (255 * (0.55 + 0.45 * f), 255 * (0.55 + 0.45 * f), 255 * (0.40 + 0.60 * f)))
# 128..191: Regenbogen fuer den Titel (Farbzyklus)
for i in range(64):
    r, g, b = colorsys.hsv_to_rgb(i / 64.0, 0.75, 1.0)
    setp(128 + i, (r * 255, g * 255, b * 255))

# ---------------------------------------------------------------- Kacheln
tiles = []


def new():
    return [[0] * 8 for _ in range(8)]


def line_tile(L, R, U, D, st):
    hi, mid, lo = 16 + 3 * st, 17 + 3 * st, 18 + 3 * st
    t = new()
    for y in range(8):
        for x in range(8):
            h = (2 <= y <= 5) and ((L and x <= 5) or (R and x >= 2))
            v = (2 <= x <= 5) and ((U and y <= 5) or (D and y >= 2))
            if not (h or v):
                continue
            if h and v:
                col = mid
            elif h:
                col = hi if y == 2 else (lo if y == 5 else mid)
            else:
                col = hi if x == 2 else (lo if x == 5 else mid)
            t[y][x] = col
    return t


# Formen (links, rechts, oben, unten): waagerecht, senkrecht, Kreuz, 4 Ecken (OL, OR, UL, UR), T nach unten, oben, rechts, links
SHAPES = [(1, 1, 0, 0), (0, 0, 1, 1), (1, 1, 1, 1), (0, 1, 0, 1), (1, 0, 0, 1), (0, 1, 1, 0), (1, 0, 1, 0), (1, 1, 0, 1), (1, 1, 1, 0), (0, 1, 1, 1), (1, 0, 1, 1)]
for sh in SHAPES:
    for st in range(4):  # unberuehrt, Spieler 1, Spieler 2, Aufblitzen
        tiles.append(line_tile(*sh, st))
assert len(tiles) == 44
FILL_T = len(tiles)
for c in range(7):
    for k in range(5):
        t = new()
        for y in range(8):
            yy = k * 8 + y
            s = 6 - (yy * 6) // 39
            if yy & 1:
                s -= 1
            if yy == 0:
                s = 7
            if yy == 39:
                s = 0
            s = max(0, min(7, s))
            for x in range(8):
                t[y][x] = FILL0 + 8 * c + s
        tiles.append(t)
PAN_T = len(tiles)
for th in range(3):
    bg, hi, lo = PAN0 + 3 * th, PAN0 + 3 * th + 1, PAN0 + 3 * th + 2
    for piece in range(9):  # 0 oben links, 1 oben, 2 oben rechts, 3 links, 4 Mitte, 5 rechts, 6 unten links, 7 unten, 8 unten rechts
        r, c = divmod(piece, 3)
        t = new()
        for y in range(8):
            for x in range(8):
                col = bg
                top = (r == 0 and y == 1) and not (c == 0 and x < 1) and not (c == 2 and x > 6)
                bot = (r == 2 and y == 6) and not (c == 0 and x < 1) and not (c == 2 and x > 6)
                lef = (c == 0 and x == 1) and not (r == 0 and y < 1) and not (r == 2 and y > 6)
                rig = (c == 2 and x == 6) and not (r == 0 and y < 1) and not (r == 2 and y > 6)
                if top or lef:
                    col = hi
                if bot or rig:
                    col = lo
                t[y][x] = col
        tiles.append(t)
MISC_T = len(tiles)
HEART = ["..1...1.", ".111.111", "11111111", "11111111", ".111111.", "..1111..", "...11...", "........"]
heart = new()
heart_e = new()
for y, row in enumerate(HEART):
    for x, ch in enumerate(row):
        if ch == '1':
            heart[y][x] = 115
            heart_e[y][x] = 116
heart[1][1] = 117
heart[1][2] = 117
tiles.append(heart)
tiles.append(heart_e)
bar_full, bar_empty = new(), new()
for y in range(8):
    for x in range(8):
        if 2 <= y <= 5:
            bar_full[y][x] = 112 if y < 4 else 113
            bar_empty[y][x] = 114
tiles.append(bar_full)
tiles.append(bar_empty)
tiles.append([[255] * 8 for _ in range(8)])  # voller Block, Farbe aus dem Farb-RAM
glint = new()
for y in range(8):
    for x in range(8):
        glint[y][x] = 120 + 7 - y
tiles.append(glint)
NTILES = len(tiles)

# ---------------------------------------------------------------- Sprites 16 x 16 (Nibble: 0 durchsichtig, 1 Umriss, 2 dunkel, 3 mittel, 4 hell, 5 Glanz,
#   6 weiss, 7 Pupille, 8 Mund, 9..c Eisblau dunkel..hell, d/e Stern innen/aussen)
SPR = {}
SPR['player_a'] = [
    "................", "....11111111....", "...1333333331...", "..133444444331..",
    ".13344444444331.", ".13445666654431.", ".13456766765431.", ".13456766765431.",
    ".13444666644331.", ".13444444444331.", ".13448888884331.", "..133488884331..",
    "..1333444433321.", "...13333333321..", "....11333311....", ".....11..11.....",
]
SPR['player_b'] = list(SPR['player_a'])
SPR['player_b'][15] = "....11....11...."
SPR['player_hit'] = [
    "................", "....11111111....", "...1333333331...", "..133444444331..",
    ".13344444444331.", ".13445666654431.", ".13457666676431.", ".13457666676431.",
    ".13444666644331.", ".13444444444331.", ".13444888844331.", "..133448844331..",
    "..1333444433321.", "...13333333321..", "....11333311....", ".....11..11.....",
]
SPR['runner_a'] = [
    "................", "..1..........1..", "..11........11..", "...1.111111.1...",
    "...1133333311...", "..133344444331..", ".13334566654331.", ".13334677674331.",
    ".13334566654331.", ".13333444443331.", ".13333333333331.", "..1113333331111.",
    "..11.13333321.11", ".11...1111111..1", ".1....1....1....", "................",
]
SPR['runner_b'] = list(SPR['runner_a'])
SPR['runner_b'][12] = "11...13333321..1"
SPR['runner_b'][13] = "1....1111111..11"
SPR['runner_b'][14] = "......1....1...."
SPR['wander_a'] = [
    "................", ".....111111.....", "...1133333311...", "..133344444331..",
    ".13334444444331.", ".13445666654431.", ".13456766765431.", ".13456766765431.",
    ".13334666644331.", ".13333444443331.", ".13333333333331.", ".13333333333321.",
    ".1333333333321..", ".13321133321.1..", ".1321.1321.11...", "..11..11........",
]
SPR['wander_b'] = list(SPR['wander_a'])
SPR['wander_b'][13] = ".1332.1133211..."
SPR['wander_b'][14] = ".1321.1321.1....."
SPR['wander_b'][15] = "..11..11.....1..."
SPR['guard_a'] = [
    "................", ".......11.......", "......1441......", ".....134431.....",
    "....13444431....", "...1344566431...", "..134456665431..", ".13445667665431.",
    "..134456665431..", "...1334566431...", "....13445431....", ".....134431.....",
    "......1331......", ".......11.......", "................", "................",
]
SPR['guard_b'] = list(SPR['guard_a'])
SPR['guard_b'][7] = ".13445766765431."
SPR['star'] = [
    "................", ".......ee.......", ".......ee.......", "......eddd......",
    "......edd.......", "eeeeeedddddeeee.", ".eddddddddddde..", "..eddddddddde...",
    "...eddddddde....", "...edddddddde...", "..eddddeedddde..", "..edddeeeeddde..",
    ".eddeee..eeedde.", ".eee........eee.", "................", "................",
]
SPR['ice'] = [
    "................", "..111111111111..", ".19cccccccccc91.", ".1cdddddddddc91.",
    ".1cdd99999ddc91.", ".1cd9ccccc9dc91.", ".1cd9cdddc9dc91.", ".1cd9cdddc9dc91.",
    ".1cd9ccccc9dc91.", ".1cdd99999ddc91.", ".1cdddddddddc91.", ".19cccccccccc91.",
    "..111111111111..", "................", "................", "................",
]
ORDER = ['player_a', 'player_b', 'player_hit', 'runner_a', 'runner_b', 'wander_a', 'wander_b', 'guard_a', 'guard_b', 'star', 'ice']
HEXMAP = {'.': 0, '1': 1, '2': 2, '3': 3, '4': 4, '5': 5, '6': 6, '7': 7, '8': 8, '9': 9, 'a': 10, 'b': 11, 'c': 12, 'd': 13, 'e': 14, 'f': 15}


def spr_bytes(rows):
    assert len(rows) == 16, len(rows)
    out = []
    for row in rows:
        if len(row) != 16:
            print("Hinweis: Sprite-Zeile mit %d statt 16 Zeichen angepasst: %s" % (len(row), row))
            row = (row + "." * 16)[:16]
        for x in range(0, 16, 2):
            out.append((HEXMAP[row[x]] << 4) | HEXMAP[row[x + 1]])  # linkes Pixel = hohes Nibble
    return out


HUES = [  # (Umriss, dunkel, mittel, hell, Glanz) je Sprite 0..7
    ((90, 60, 0), (200, 150, 10), (255, 215, 40), (255, 245, 140), (255, 255, 230)),   # 0 Spieler 1 gelb
    ((0, 60, 80), (20, 150, 170), (60, 225, 235), (170, 250, 255), (235, 255, 255)),   # 1 Spieler 2 cyan
    ((80, 10, 90), (150, 40, 170), (205, 80, 225), (235, 150, 245), (250, 215, 255)),  # 2 Gegner violett
    ((90, 10, 20), (170, 30, 40), (235, 70, 70), (250, 140, 130), (255, 215, 210)),    # 3 Gegner rot
    ((10, 30, 100), (30, 80, 190), (70, 140, 250), (150, 200, 255), (220, 240, 255)),  # 4 Gegner blau
    ((10, 80, 40), (30, 150, 70), (70, 220, 110), (150, 250, 170), (220, 255, 230)),   # 5 Gegner gruen
    ((90, 50, 0), (190, 120, 10), (250, 170, 30), (255, 215, 110), (255, 245, 210)),   # 6 Gegenstand orange
    ((60, 60, 70), (110, 110, 130), (170, 170, 190), (220, 220, 235), (250, 250, 255)),  # 7 frei grau
]
spal = [(0, 0, 0)] * 128
for n, (o, d, m, h, g) in enumerate(HUES):
    base = 16 * n
    spal[base + 1] = o
    spal[base + 2] = d
    spal[base + 3] = m
    spal[base + 4] = h
    spal[base + 5] = g
    spal[base + 6] = (250, 250, 250)
    spal[base + 7] = (20, 20, 30)
    spal[base + 8] = (120, 30, 40)
    spal[base + 9] = (50, 110, 190)
    spal[base + 10] = (90, 160, 225)
    spal[base + 11] = (140, 205, 250)
    spal[base + 12] = (200, 235, 255)
    spal[base + 13] = (255, 245, 150)
    spal[base + 14] = (255, 200, 30)
    spal[base + 15] = (255, 255, 255)


def sw(v):
    """Halbbytes vertauscht (so erwarten es die Palettenregister des MEGA65)."""
    v = int(v) & 255
    return ((v & 15) << 4) | (v >> 4)


out = []
out.append("; =====================================================================================================================")
out.append("; assets.inc - ERZEUGT von tools/gen_assets.py (nicht von Hand aendern): Palette, Kacheln, Sprite-Palette, Sprites")
out.append("; =====================================================================================================================")
out.append(".const NTILES = %d                   ; Anzahl der Vollfarb-Kacheln (je 64 Byte)" % NTILES)
out.append(".const T_FILL  = %d                   ; erste Fuellkachel: T_FILL + 5 * Farbe + Zeile im Rechteck" % FILL_T)
out.append(".const T_PANEL = %d                   ; Tafelkacheln: T_PANEL + 9 * Thema + Teil (0..8 zeilenweise)" % PAN_T)
out.append(".const T_HEART = %d" % MISC_T)
out.append(".const T_HEART_E = %d" % (MISC_T + 1))
out.append(".const T_BAR_FULL = %d" % (MISC_T + 2))
out.append(".const T_BAR_EMPTY = %d" % (MISC_T + 3))
out.append(".const T_SOLID = %d                  ; voller Block (Pixelwert 255: Farbe aus dem Farb-RAM)" % (MISC_T + 4))
out.append(".const T_GLINT = %d                  ; Glanzzeile" % (MISC_T + 5))
out.append(".const NSPR_IMG = %d" % len(ORDER))
for nm, tab in (('PAL_R', 0), ('PAL_G', 1), ('PAL_B', 2)):
    out.append("%s:" % nm)
    vals = [sw(pal[i][tab]) for i in range(256)]
    for i in range(0, 256, 16):
        out.append("        .byte " + ", ".join("$%02x" % v for v in vals[i:i + 16]))
for nm, tab in (('SPAL_R', 0), ('SPAL_G', 1), ('SPAL_B', 2)):
    out.append("%s:" % nm)
    vals = [sw(spal[i][tab]) for i in range(128)]
    for i in range(0, 128, 16):
        out.append("        .byte " + ", ".join("$%02x" % v for v in vals[i:i + 16]))
out.append("TILES:")
for t in tiles:
    flat = [v for row in t for v in row]
    for i in range(0, 64, 16):
        out.append("        .byte " + ", ".join("%d" % v for v in flat[i:i + 16]))
out.append("SPRITES:")
for nm in ORDER:
    out.append("        ; " + nm)
    b = spr_bytes(SPR[nm])
    for i in range(0, 128, 16):
        out.append("        .byte " + ", ".join("$%02x" % v for v in b[i:i + 16]))
open(sys.argv[1] if len(sys.argv) > 1 else "assets.inc", "w", encoding="utf-8", newline="\n").write("\n".join(out) + "\n")
print("Kacheln", NTILES, "Sprites", len(ORDER))



