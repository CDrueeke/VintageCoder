#!/usr/bin/env python3
"""Erzeugt lib/bricks.inc: die Steine von PRALLBALL fuer den MEGA65 als Vollfarbzeichen (FCM, 256 Farben, 8 x 8 Bildpunkte je Zeichen).

Ein Stein besteht aus zwei Zeichen nebeneinander (16 x 8 Bildpunkte): links Bildpunkte 0..7, rechts 8..15.  Jeder der 23 Steintypen und
eine weisse Glanzfassung (Typ 24, fuer das Glanzlicht, das ueber das Feld laeuft) hat acht Helligkeitsstufen; jede Stufe ist ein eigener
Eintrag der VIC-IV-Palette (Eintraege 32 + 8 * Typ + Stufe).  Bildpunkt 0 ist durchsichtig (Spalt zwischen den Steinen, abgerundete Ecken).
Form des Steins (15 x 7 Bildpunkte): helle Oberkante, helle linke Kante, dunkle untere und rechte Kante, senkrechter Farbverlauf und ein
diagonaler Glanzstreifen.
Ausgabe: brick_pat (EIN Stein = 2 Zeichen * 64 Byte, Werte 32 + Stufe; fcm_init erzeugt daraus die 24 Typen, indem es 8 * (Typ - 1) addiert),
         bpal_r/bpal_g/bpal_b (192 Eintraege, Halbbytes vertauscht wie die Register).
Aufruf im Ordner examples/mega65/prallball:  python3 tools/gen_bricks.py"""
import os
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/mega65/prallball (os.chdir); alle Pfade gelten relativ dazu, der Aufruf funktioniert daher aus jedem Ordner.', epilog='Beispiele:\n  python3 examples/mega65/prallball/tools/gen_bricks.py\n  cd examples/mega65/prallball && python3 tools/gen_bricks.py\n\nSchreibt (im Projektordner examples/mega65/prallball, vorhandene Dateien werden ueberschrieben): lib/bricks.inc.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
# Farbcode je Steintyp (Farb-RAM-Wert wie im Spiel: 8 + C64-Farbe), siehe brick_color in prallball.asm
BRICK_COLOR = [8, 10, 15, 13, 11, 14, 12, 9, 10, 9, 15, 8, 10, 9, 8, 8, 10, 12, 14, 11, 13, 15, 9, 9]
C64 = {0: (40, 44, 52), 1: (235, 235, 245), 2: (214, 52, 50), 3: (64, 214, 214), 4: (190, 70, 200), 5: (60, 200, 80), 6: (60, 80, 220), 7: (250, 220, 60)}
GOLD = (236, 176, 48)
STEEL = (150, 160, 178)
SILVER = (196, 204, 218)


def base_color(t):
    """t = 1..23 -> RGB; 24 = Glanzfassung (weiss)."""
    if t == 24:
        return (255, 255, 255)
    if t == 9:
        return SILVER
    if t == 10:
        return GOLD
    if t in (11, 14, 15):
        return STEEL
    c = BRICK_COLOR[t] & 7
    return C64[c]


def shade(rgb, s):
    f = s / 7.0
    out = []
    for c in rgb:
        if f < 0.6:
            v = c * (0.28 + f / 0.6 * 0.72)
        else:
            v = c + (255 - c) * ((f - 0.6) / 0.4) * 0.65
        out.append(max(0, min(255, int(round(v)))))
    return tuple(out)


def stufe(x, y):
    """Helligkeitsstufe 0..7 des Bildpunkts (x 0..15, y 0..7) oder None = durchsichtig."""
    if x == 15 or y == 7:
        return None
    if (x in (0, 14)) and (y in (0, 6)):
        return None                                      # abgerundete Ecken
    if y == 0:
        return 7
    if x == 0:
        return 6
    if y == 6:
        return 0
    if x == 14:
        return 1
    s = (5, 5, 4, 4, 3)[y - 1]
    if 1 <= y <= 3 and 2 <= x - 3 * (y - 1) <= 7:
        s += 2                                           # Glanzstreifen
    elif x - 2 * y > 8:
        s -= 1                                           # rechts etwas dunkler
    return max(0, min(7, s))


tiles = bytearray()                                      # nur Typ 1 (Muster); die anderen Typen unterscheiden sich nur durch den Palettenbereich
pal = []                                                 # (r, g, b) je Eintrag 32 + 8 * Typ + Stufe
for t in range(1, 25):
    rgb = base_color(t)
    for s in range(8):
        pal.append(shade(rgb, s))
    if t == 1:
        for half in (0, 1):
            for y in range(8):
                for x in range(8):
                    st = stufe(half * 8 + x, y)
                    tiles.append(0 if st is None else 32 + st)
assert len(tiles) == 2 * 64

# ---- Wand (ein Zeichen, Stahlblau, 8 Stufen = Palette 224..231) und Logo-Block (ein Zeichen: Rand fest weiss/dunkelgrau, Flaeche = Farbe
#      aus dem Farb-RAM, damit der Regenbogen des Titels weiterlaeuft: Bildpunktwert 255 nimmt die Farbe des Farb-RAM)
STEEL_BLUE = (120, 140, 205)
wall = bytearray()
for y in range(8):
    for x in range(8):
        if y == 0:
            st = 7
        elif x == 0:
            st = 6
        elif y == 7:
            st = 0
        elif x == 7:
            st = 1
        elif 3 <= x <= 4 and 3 <= y <= 4:
            st = 5 if (x, y) == (3, 3) else 2 if (x, y) == (4, 4) else 4          # Niete
        else:
            st = 3
        wall.append(224 + st)
for st in range(8):
    pal.append(shade(STEEL_BLUE, st))
pal.append((245, 245, 255))                              # Eintrag 232: Lichtkante des Logo-Blocks
pal.append((30, 30, 50))                                 # Eintrag 233: Schattenkante
logo = bytearray()
for y in range(8):
    for x in range(8):
        if y == 0 or x == 0:
            v = 232                                      # weisse Lichtkante
        elif y == 7 or x == 7:
            v = 233                                      # dunkle Schattenkante
        elif (x, y) in ((1, 1), (2, 1), (1, 2)):
            v = 232
        else:
            v = 255                                      # Farbe aus dem Farb-RAM
        logo.append(v)
tiles += wall + logo                                     # brick_pat: Stein (128 Byte), danach Wand (64) und Logo-Block (64)

# ---- Bruch-Animation: zwei Phasen, die zerbroeselnde Fassung des Steins.  Das 16 x 8 grosse Feld wird in 2 x 2 grosse Stuecke geteilt; je
#      Stueck bleibt es erhalten oder nicht, und es faellt um einige Zeilen (Bildpunkte, die aus dem Zeichen herausfallen, entfallen).
#      Werte wie beim Stein (32 + Stufe), fcm_init addiert den Palettenbereich des Typs.  Deterministisch (feste Zahlenfolge).
seed = 12345


def zufall():
    global seed
    seed = (seed * 1103515245 + 12345) & 0x7fffffff
    return (seed >> 16) & 255


def bruch(behalten, fall):
    feld = [[None] * 16 for _ in range(8)]
    for by in range(4):
        for bx in range(8):
            if zufall() >= behalten:
                continue
            e = zufall() % 3
            for dy in range(2):
                for dx in range(2):
                    x, y = bx * 2 + dx, by * 2 + dy
                    st = stufe(x, y)
                    ny = y + fall + e
                    if st is not None and ny < 7:
                        feld[ny][x] = 32 + min(7, st + 1)    # Bruchstuecke leuchten etwas heller
    out = bytearray()
    for half in (0, 1):
        for y in range(8):
            for x in range(8):
                v = feld[y][half * 8 + x]
                out.append(0 if v is None else v)
    return out


crumb = bruch(190, 1) + bruch(90, 3)
assert len(crumb) == 256


# ======================================================================================================================================
# Hohe Aufloesung (640 x 400): ein Stein hat 32 x 16 Bildpunkte = 4 x 2 Vollfarbzeichen.  Je Farbgruppe (10) drei Fassungen (Stein, Bruch 1,
# Bruch 2) zu je 512 Byte; die Gruppe benutzt den Palettenbereich eines LORES-Typs (hb_off = 8 * (Typ - 1), fcm_init addiert).
# Dazu Wand, Logo-Block (je 16 x 16 = 4 Zeichen) und ein voller Block (1 Zeichen) sowie die Tabelle Typ -> Gruppe.
# ======================================================================================================================================
import math
HW, HH, HR = 31, 15, 3
BAYER = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]


def h_innen(x, y):
    if not (0 <= x < HW and 0 <= y < HH):
        return False
    for cx, cy in ((HR, HR), (HW - 1 - HR, HR), (HR, HH - 1 - HR), (HW - 1 - HR, HH - 1 - HR)):
        if ((x < HR and cx == HR) or (x > HW - 1 - HR and cx != HR)) and ((y < HR and cy == HR) or (y > HH - 1 - HR and cy != HR)):
            if (x - cx) ** 2 + (y - cy) ** 2 > HR * HR:
                return False
    return True


def h_licht(x, y):
    v = 4.7 - 1.7 * (y / (HH - 1))
    dl, dr, dt, db = x, HW - 1 - x, y, HH - 1 - y
    d = min(dl, dr, dt, db)
    u = x - 1.6 * y
    if d >= 2:
        v += 2.1 * math.exp(-((u - 9) / 3.0) ** 2)
        v += 0.9 * math.exp(-((u - 19) / 2.0) ** 2)
        v += 1.6 * math.exp(-((x - 6) ** 2 + (y - 4) ** 2) / 7.0)
    if dt < 2:
        v = 7.0 - dt * 1.4
    if dl < 2:
        v = max(v, 6.2 - dl * 1.0)
    if db < 2:
        v = 0.5 + db * 0.9
    if dr < 2:
        v = min(v, 1.0 + dr * 0.9)
    return v


def h_stufe(x, y):
    if not h_innen(x, y):
        return None
    t = BAYER[y & 3][x & 3] / 16.0 - 0.5
    return max(0, min(7, int(math.floor(h_licht(x, y) + 0.5 + t * 0.9))))


def h_feld():
    return [[h_stufe(x, y) for x in range(32)] for y in range(16)]


def h_chars(feld):
    """16 x 32 Feld -> 8 Zeichen (4 x 2, Reihenfolge Zeile fuer Zeile) zu je 64 Byte; Werte 32 + Stufe, None = 0."""
    out = bytearray()
    for ky in range(2):
        for kx in range(4):
            for y in range(8):
                for x in range(8):
                    v = feld[ky * 8 + y][kx * 8 + x]
                    out.append(0 if v is None else 32 + v)
    return out


def h_bruch(basis, behalten, fall):
    feld = [[None] * 32 for _ in range(16)]
    for by in range(4):
        for bx in range(8):
            if zufall() >= behalten:
                continue
            e = zufall() % 3
            for dy in range(4):
                for dx in range(4):
                    x, y = bx * 4 + dx, by * 4 + dy
                    st = basis[y][x]
                    ny = y + fall + e * 2
                    if st is not None and ny < 15:
                        feld[ny][x] = min(7, st + 1)
    return feld


h0 = h_feld()
hb = h_chars(h0) + h_chars(h_bruch(h0, 190, 2)) + h_chars(h_bruch(h0, 90, 6))
assert len(hb) == 3 * 512


def quad(f):
    """16 x 16 -> 4 Zeichen (links oben, rechts oben, links unten, rechts unten) zu je 64 Byte."""
    out = bytearray()
    for qy in range(2):
        for qx in range(2):
            for y in range(8):
                for x in range(8):
                    out.append(f(qx * 8 + x, qy * 8 + y))
    return out


def wand(x, y):
    dl, dr, dt, db = x, 15 - x, y, 15 - y
    if dt < 2:
        st = 7 - dt
    elif dl < 2:
        st = 6 - dl
    elif db < 2:
        st = db
    elif dr < 2:
        st = 1 + dr
    else:
        st = 3.6 - 0.8 * (y / 15.0) + 0.9 * math.exp(-((x - 4) ** 2 + (y - 4) ** 2) / 10.0)
        r2 = (x - 7.5) ** 2 + (y - 7.5) ** 2                 # Niete in der Mitte
        if r2 < 9.5:
            st = 4.8 + 1.4 * math.exp(-((x - 6.2) ** 2 + (y - 6.2) ** 2) / 2.5) - 0.07 * r2
        elif r2 < 14:
            st = 1.5                                         # Schatten um die Niete
        t = BAYER[y & 3][x & 3] / 16.0 - 0.5
        st = int(math.floor(st + 0.5 + t * 0.8))
    return 224 + max(0, min(7, int(st)))


def logo(x, y):
    dl, dr, dt, db = x, 15 - x, y, 15 - y
    if dt < 2 or dl < 2:
        return 232
    if db < 2 or dr < 2:
        return 233
    if (x, y) in ((3, 3), (4, 3), (3, 4), (5, 3), (3, 5)):
        return 232
    return 255


small = quad(wand) + quad(logo) + bytes([234] * 64)
assert len(small) == 9 * 64
pal.append((0, 0, 0))                                    # Eintrag 234: voller schwarzer Block (hohe Aufloesung)
# Typ -> Farbgruppe (0 rot, 1 gelb, 2 gruen, 3 tuerkis, 4 blau, 5 violett, 6 weiss/silber, 7 gold, 8 stahl, 9 Glanzfassung)
GRP_OF_COLOR = {2: 0, 7: 1, 5: 2, 3: 3, 6: 4, 4: 5, 1: 6, 0: 8}
grp = []
for t in range(25):
    if t == 24:
        grp.append(9)
    elif t == 10:
        grp.append(7)
    elif t == 9:
        grp.append(6)
    elif t in (11, 14, 15):
        grp.append(8)
    else:
        grp.append(GRP_OF_COLOR[BRICK_COLOR[t] & 7])
REP = [1, 2, 3, 4, 5, 6, 9, 10, 11, 24]                 # Beispieltyp je Gruppe (Palettenbereich)
hoff = [8 * (r - 1) for r in REP]


def swap(v):
    return ((v & 15) << 4) | (v >> 4)


out = ['; ERZEUGT von tools/gen_bricks.py: Vollfarb-Steine (FCM) und ihre Palette.', 'brick_pat:']
for i in range(0, len(tiles), 32):
    out.append('        .byte ' + ', '.join(str(b) for b in tiles[i:i + 32]))
out.append('crumb_pat:')
for i in range(0, len(crumb), 32):
    out.append('        .byte ' + ', '.join(str(b) for b in crumb[i:i + 32]))
out.append('hb_pat:')
for i in range(0, len(hb), 32):
    out.append('        .byte ' + ', '.join(str(b) for b in hb[i:i + 32]))
out.append('hs_pat:')
for i in range(0, len(small), 32):
    out.append('        .byte ' + ', '.join(str(b) for b in small[i:i + 32]))
out.append('brick_grp:')
out.append('        .byte ' + ', '.join(str(g) for g in grp))
out.append('hb_off:')
out.append('        .byte ' + ', '.join(str(o) for o in hoff))
for name, k in (('bpal_r', 0), ('bpal_g', 1), ('bpal_b', 2)):
    out.append(name + ':')
    vals = [swap(p[k]) for p in pal]
    for i in range(0, len(vals), 16):
        out.append('        .byte ' + ', '.join('$%02x' % v for v in vals[i:i + 16]))
open('lib/bricks.inc', 'w').write('\n'.join(out) + '\n')
print('Steine:', len(tiles), 'Byte; Palette:', len(pal), 'Eintraege')
