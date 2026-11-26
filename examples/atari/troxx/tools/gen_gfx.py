#!/usr/bin/env python3
# gen_gfx.py - erzeugt gfx.inc fuer Troxx (Atari): die Zeichen 64..127 des Zeichensatzes (Modus 4, vier Pixelpaare je Zeichen).
# Aufruf (im Projektordner):  python3 tools/gen_gfx.py > gfx.inc
# Die Codes 0..63 (Schrift fuer die Textzeilen im Modus 2) kopiert das Programm zur Laufzeit aus dem Zeichen-ROM des Rechners.
# Pixelwerte je Paar: 0 Hintergrund (COLBK), 1 COLPF0 (unberuehrte Linie), 2 COLPF1 (abgefahrene Linie), 3 COLPF2 (Zeilenfarbe; mit gesetztem
# Bit 7 im Bildschirmcode COLPF3).  Eigene Entwuerfe, keine fremden Schriften.
import sys

# ---- Schrift der Seitenleiste: 3 Paare breit, 7 Zeilen hoch (viertes Paar und achte Zeile bleiben leer); Code = 64 + Bildschirmcode
FONT = {
 'A': "010101101111101101101", 'B': "110101101110101101110", 'C': "011100100100100100011", 'D': "110101101101101101110",
 'E': "111100100110100100111", 'H': "101101101111101101101", 'I': "111010010010010010111", 'L': "100100100100100100111",
 'N': "101111111101101101101", 'O': "010101101101101101010", 'P': "110101101110100100100", 'R': "110101101110110101101",
 'S': "011100100010001001110", 'T': "111010010010010010010", 'U': "101101101101101101111", 'V': "101101101101101010010",
 'X': "101101101010101101101", 'Y': "101101101010010010010",
 '0': "111101101101101101111", '1': "010110010010010010111", '2': "110001001010100100111", '3': "110001001010001001110",
 '4': "101101101111001001001", '5': "111100100110001001110", '6': "011100100110101101010", '7': "111001001010010010010",
 '8': "010101101010101101010", '9': "010101101011001001110",
 ' ': "0" * 21,
}

def internal(ch):
    if ch == ' ': return 0
    if '0' <= ch <= '9': return 16 + ord(ch) - 48
    return 33 + ord(ch) - 65

glyphs = {}      # Code -> 8 Zeilen zu je 4 Werten (0..3)
for ch, bits in FONT.items():
    rows = []
    for r in range(7):
        rows.append([3 if bits[r * 3 + c] == '1' else 0 for c in range(3)] + [0])
    rows.append([0, 0, 0, 0])
    glyphs[64 + internal(ch)] = rows

free = [c for c in range(64, 128) if c not in glyphs]
names = {}
def alloc(name, rows):
    c = free.pop(0)
    glyphs[c] = rows
    names[name] = c

def blank(): return [[0] * 4 for _ in range(8)]

# ---- Linienzeichen: Arme oben/unten/links/rechts; Mitte Zeilen 2..5, Paare 1..2
def line_glyph(up, down, left, right, v):
    g = blank()
    for r in range(8):
        for c in range(4):
            on = False
            if 2 <= r <= 5 and 1 <= c <= 2: on = True
            if up and r <= 1 and 1 <= c <= 2: on = True
            if down and r >= 6 and 1 <= c <= 2: on = True
            if left and c == 0 and 2 <= r <= 5: on = True
            if right and c == 3 and 2 <= r <= 5: on = True
            if on: g[r][c] = v
    return g

LINES = [('HLINE', 0, 0, 1, 1), ('VLINE', 1, 1, 0, 0), ('CROSS', 1, 1, 1, 1), ('TL', 0, 1, 0, 1), ('TR', 0, 1, 1, 0), ('BL', 1, 0, 0, 1),
         ('BR', 1, 0, 1, 0), ('TDOWN', 0, 1, 1, 1), ('TUP', 1, 0, 1, 1), ('TRIGHT', 1, 1, 0, 1), ('TLEFT', 1, 1, 1, 0)]
for n, u, d, l, r in LINES:
    alloc('CH_' + n, line_glyph(u, d, l, r, 1))
for n, u, d, l, r in LINES[:2]:
    alloc('CH_' + n + '_D', line_glyph(u, d, l, r, 2))      # abgefahren (gelb)
for n, u, d, l, r in [x for x in LINES if x[0] in ('CROSS', 'TDOWN', 'TUP', 'TRIGHT', 'TLEFT')]:
    alloc('CH_' + n + '_D', line_glyph(u, d, l, r, 2))      # gelber Knoten (Ecken benutzen die T-Stuecke, siehe NODE_FROM / NODE_TO in data.inc)
# ---- Fuellung: Rahmenzeichen mit Abschraegung (hell oben/links, dunkel unten/rechts), Mitte mit Muster
def bevel(top, bottom, left, right):
    g = blank()
    for r in range(8):
        for c in range(4):
            v = 3
            if (top and r == 0) or (left and c == 0): v = 2
            elif (bottom and r == 7) or (right and c == 3): v = 1
            g[r][c] = v
    return g
for n, t, b, l, r in [('FTL', 1, 0, 1, 0), ('FT', 1, 0, 0, 0), ('FTR', 1, 0, 0, 1), ('FL', 0, 0, 1, 0), ('FR', 0, 0, 0, 1),
                      ('FBL', 0, 1, 1, 0), ('FB', 0, 1, 0, 0), ('FBR', 0, 1, 0, 1)]:
    alloc('CH_' + n, bevel(t, b, l, r))
def pat(f):
    g = blank()
    for r in range(8):
        for c in range(4):
            g[r][c] = 3 if f(r, c) else 1
    return g
alloc('CH_FILL0', pat(lambda r, c: (r + 2 * c) % 4 < 2))                       # Schraffur
alloc('CH_FILL1', pat(lambda r, c: not (r % 4 == 1 and c % 2 == 1)))           # Punkte (Aussparungen)
alloc('CH_FILL2', pat(lambda r, c: r % 4 != 3 and c != (0 if (r // 4) % 2 == 0 else 2)))   # Mauer
alloc('CH_FILL3', pat(lambda r, c: ((r // 2) + c) % 2 == 0))                    # Schachbrett

def rows_from(strs):
    return [[int(ch) for ch in s] for s in strs]
alloc('CH_HEART', rows_from(["0000", "3030", "3330", "3330", "3330", "0300", "0000", "0000"]))
alloc('CH_STAR',  rows_from(["0000", "0220", "0220", "2222", "0220", "0220", "2002", "0000"]))
alloc('CH_ICE',   rows_from(["0000", "2222", "2331", "2331", "2111", "2111", "1111", "0000"]))
alloc('CH_BLOCK', rows_from(["3333"] * 8))       # voller Block (Balken, Titelbild)

print("; gfx.inc - ERZEUGT von tools/gen_gfx.py (nicht von Hand aendern).  Zeichen 64..127 des Zeichensatzes (Modus 4).")
print("; Schrift der Seitenleiste: Code = 64 + Bildschirmcode (Ziffern 80..89, Buchstaben 97..122, Leerzeichen 64); Grafikzeichen in den freien Codes.")
print(".const TXT_BASE = 64")
for n, c in names.items():
    print(f".const {n} = {c}")
print(f".const NGLYPH_FREE = {len(free)}                ; ungenutzte Codes")
print("GFX_CHARS:")
for c in range(64, 128):
    g = glyphs.get(c, blank())
    bs = []
    for r in range(8):
        v = 0
        for k in range(4):
            v = (v << 2) | g[r][k]
        bs.append(v)
    print("        .byte " + ", ".join("$%02x" % b for b in bs) + f"           ; {c}")
sys.stderr.write("frei: %d, benutzt: %d\n" % (len(free), 64 - len(free)))
