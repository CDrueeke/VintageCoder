#!/usr/bin/env python3
# gen_sprites.py - erzeugt sprites.inc (Sprite-Daten von Troxx) aus ASCII-Zeichnungen.
# Aufruf: python3 tools/gen_sprites.py > sprites.inc   (im Projektordner)
# Jedes Sprite: 16 x 16 Punkte, mehrfarbig (8 x 16 logische Punkte), mittig im 24 x 21-Feld des VIC-II (logische Spalten 2..9, Zeilen 2..17), 64 Byte.
import sys

ART = {}
# Mehrfarben-Sprites: 8 x 16 logische Punkte (je 2 Bildpunkte breit): '.' durchsichtig, 'o' Umriss (gemeinsame Farbe 0, schwarz),
# 'b' Koerperfarbe (je Sprite), 'w' Glanz und Augen (gemeinsame Farbe 1, weiss)
ART["player0"] = """
..oooo..
.obbbbo.
obwwbbbo
obwbbbbo
obbbbbbo
owwbbwwo
owowbwow
obbbbbbo
obbbbbbo
obobbobo
obboobbo
obbbbbbo
.obbbbo.
..oooo..
........
........
"""
ART["player1"] = """
..oooo..
.obbbbo.
obwwbbbo
obwbbbbo
obbbbbbo
obwbbwbo
obobbobo
obbbbbbo
obbbbbbo
obboobbo
obobbobo
obbbbbbo
.obbbbo.
..oooo..
........
........
"""
ART["burst"] = """
...bb...
o..bb..o
bo.bb.ob
.bobbob.
..obbo..
bbbwwbbb
bbwwwwbb
bbbwwbbb
..obbo..
.bobbob.
bo.bb.ob
o..bb..o
...bb...
........
........
........
"""
ART["runner0"] = """
o......o
bo....ob
.bbbbbb.
.bobbob.
bbwbbwbb
bbobbobb
bbbbbbbb
bbbbbbbb
bobbbbob
bobbbbob
ob.oo.bo
.o....o.
........
........
........
........
"""
ART["runner1"] = """
bo....ob
.o....o.
.bbbbbb.
.bobbob.
bbwbbwbb
bbobbobb
bbbbbbbb
bbbbbbbb
.bbbbbb.
ob.bb.bo
bo.oo.ob
o......o
........
........
........
........
"""
ART["wander0"] = """
..oooo..
.obbbbo.
obbbbbbo
obwwbwwo
obwobwoo
obbbbbbo
obbbbbbo
obbbbbbo
obbbbbbo
obbbbbbo
obbbbbbo
obobobbo
.o.o.o.o
........
........
........
"""
ART["wander1"] = """
..oooo..
.obbbbo.
obbbbbbo
obwwbwwo
obwobwoo
obbbbbbo
obbbbbbo
obbbbbbo
obbbbbbo
obbbbbbo
obbbbbbo
obbobobo
o.o.o.o.
........
........
........
"""
ART["guard0"] = """
..oooo..
.obbbbo.
obbbbbbo
obwbbwbo
obobbobo
obbbbbbo
.obbbbo.
..obbo..
.obobbo.
.ob.bo..
ob.ob.o.
o..o..o.
........
........
........
........
"""
ART["guard1"] = """
..oooo..
.obbbbo.
obbbbbbo
obwbbwbo
obobbobo
obbbbbbo
.obbbbo.
..obbo..
..obobo.
..ob.bo.
.o.bo.bo
.o..o..o
........
........
........
........
"""
ART["star"] = """
...oo...
...bb...
..obbo..
..obwo..
oobbwboo
obbwwbbo
.obbbbo.
..obbo..
..obbo..
.obbbbo.
.obo.obo
.oo..oo.
........
........
........
........
"""
ART["ice"] = """
.oooooo.
obwwwbbo
obwbbbbo
obwbbbbo
obbbbbbo
obbbbbbo
obbbbwbo
obbbbwwo
obbbbbbo
obbbbbbo
obbbbbbo
.oooooo.
........
........
........
........
"""

ORDER = ["player0", "player1", "burst", "runner0", "runner1", "wander0", "wander1", "guard0", "guard1", "star", "ice"]


def block(name):
    rows = [r for r in ART[name].strip("\n").split("\n")]
    assert len(rows) == 16, (name, len(rows))
    code = {".": 0, "o": 1, "b": 2, "w": 3}
    grid = [[0] * 12 for _ in range(21)]
    for y, r in enumerate(rows):
        assert len(r) == 8, (name, r)
        for x, c in enumerate(r):
            grid[y + 2][x + 2] = code[c]
    out = []
    for y in range(21):
        for b in range(3):
            v = 0
            for k in range(4):
                v = (v << 2) | grid[y][b * 4 + k]
            out.append(v)
    out.append(0)
    return out


print("; sprites.inc - erzeugt von tools/gen_sprites.py, nicht von Hand aendern")
print("; %d Sprites zu je 64 Byte: %s" % (len(ORDER), " ".join(ORDER)))
for n in ORDER:
    print("; " + n)
    d = block(n)
    for i in range(0, 64, 8):
        print("        .byte " + ",".join("$%02x" % v for v in d[i:i + 8]))
