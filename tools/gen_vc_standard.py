#!/usr/bin/env python3
# SPDX-License-Identifier: CC0-1.0
"""Erzeugt die eigenen Standardzeichensätze „VC-Standard“ (CC0-1.0) unter data/charsets/.

Gestaltung nach dem Stilblatt data/charsets/STIL.md („Fasen-Grotesk 8“, Stand 1): Die Buchstaben, Ziffern und Satzzeichen stehen unten als
5-Spalten-Raster (Spalten 1 bis 5 der 8×8-Zelle), die Grafikzeichen werden aus Regeln erzeugt (Linien auf Spalte/Zeile 3, Blöcke, Dreiecke,
Muster). Nichts davon stammt aus einem Original-ROM oder einem Nachbau; dieses Werkzeug liest keine ROMs.

Aufruf (im Repository):
  python3 tools/gen_vc_standard.py            schreibt data/charsets/** neu (Sätze, index.json, ausnahmen.json)
  python3 tools/gen_vc_standard.py --check    schreibt nichts, Rückgabewert 1, wenn die Dateien vom Erzeugnis abweichen
Nur Standardbibliothek. Das Ergebnis ist deterministisch (gleiche Eingabe, gleiche Bytes).

Der Prüfstempel (`origin.check`) wird hier mit Ergebnis „offen“ angelegt (er bleibt es: die Abstandsprüfung wurde nicht bestanden, kein `--stempel`; die Gestaltung ist freigegeben, siehe `origin.freigabe_vermerk`) (Hash des Satzes, keine Abstandsprüfung); die Abstandsprüfung gegen
die ROMs des Anwenders läuft nur lokal mit tools/charset_distance.py und setzt den Stempel auf „bestanden“.
"""
import hashlib, json, os, sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
OUT = os.path.join(ROOT, "data", "charsets")
STIL = "STIL.md@1"
TOOL = "tools/gen_vc_standard.py 1"
CREATED = "2026-11"
AUTHOR = "VintageCoder-Projekt (Entwurf aus Stilblatt-Regeln, Gestaltung vom Projektinhaber freigegeben)"
LICENSE = "CC0-1.0"

# ---------------------------------------------------------------------------------------------------------------------------------
# Text- und Satzzeichen: 5 Spalten breit, Bezugslinie = Zeile 5 (Versalhöhe 6: Zeilen 0 bis 5, x-Höhe 4: Zeilen 2 bis 5, Unterlängen 6 und 7).
# Jeder Eintrag: (oberste Zeile, [Zeilen]).  '#' = gesetzt.
# ---------------------------------------------------------------------------------------------------------------------------------
def g(top, *rows):
    return (top, list(rows))

CAPS = {
    "A": g(0, ".###.", "#...#", "#...#", "#####", "#...#", "#...#"),
    "B": g(0, "####.", "#...#", "####.", "#...#", "#...#", "####."),
    "C": g(0, ".####", "#....", "#....", "#....", "#....", ".####"),
    "D": g(0, "####.", "#...#", "#...#", "#...#", "#...#", "####."),
    "E": g(0, "#####", "#....", "####.", "#....", "#....", "#####"),
    "F": g(0, "#####", "#....", "####.", "#....", "#....", "#...."),
    "G": g(0, ".####", "#....", "#.###", "#...#", "#...#", ".###."),
    "H": g(0, "#...#", "#...#", "#####", "#...#", "#...#", "#...#"),
    "I": g(0, ".####", "..#..", "..#..", "..#..", "..#..", "####."),   # versetzte Plattenserifen
    "J": g(0, "..###", "...#.", "...#.", "...#.", "#..#.", ".##.."),
    "K": g(0, "#..##", "#.#..", "##...", "#.#..", "#..#.", "#...#"),   # oberer Arm mit Haken
    "L": g(0, "###..", "#....", "#....", "#....", "#...#", "#####"),   # Kopfserife links, Fuß mit Aufstrich
    "M": g(0, "#...#", "##.##", "#.#.#", "#...#", "#...#", "#...#"),
    "N": g(0, "#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#"),
    "O": g(0, ".###.", "#...#", "#...#", "#...#", "#...#", ".###."),
    "P": g(0, "####.", "#...#", "#...#", "####.", "#....", "#...."),
    "Q": g(0, ".###.", "#...#", "#...#", "#.#.#", "#..#.", ".##.#"),
    "R": g(0, "####.", "#...#", "#...#", "####.", "#..#.", "#...#"),
    "S": g(0, ".####", "#....", ".###.", "....#", "....#", "####."),
    "T": g(0, "#####", "#.#.#", "..#..", "..#..", "..#..", ".###."),   # Haken an den Balkenenden, Fußserife
    "U": g(0, "#...#", "#...#", "#...#", "#...#", "#...#", ".###."),
    "V": g(0, "#...#", "#...#", "#...#", ".#.#.", ".#.#.", "..#.."),
    "W": g(0, "#...#", "#...#", "#...#", "#.#.#", "##.##", "#...#"),
    "X": g(0, "#...#", ".#.#.", "..#..", "..#..", ".#.#.", "#...#"),
    "Y": g(0, "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#.."),
    "Z": g(0, "#####", "....#", "...#.", "..#..", ".#...", "#####"),
}

LOWER = {
    "a": g(2, ".###.", "....#", ".####", "#####"),
    "b": g(0, "#....", "#....", "####.", "#...#", "#...#", "####."),
    "c": g(2, ".####", "#....", "#....", ".####"),
    "d": g(0, "....#", "....#", ".####", "#...#", "#...#", ".####"),
    "e": g(2, ".###.", "#...#", "#####", ".####"),
    "f": g(0, "..##.", ".#...", "####.", ".#...", ".#...", ".#..."),
    "g": g(2, ".###.", "#...#", ".####", "....#", "#...#", ".###."),
    "h": g(0, "#....", "#....", "####.", "#...#", "#...#", "#...#"),
    "i": g(0, ".#...", ".....", ".#...", ".#...", ".#...", ".##.."),   # Stamm links, Fuß nach rechts
    "j": g(0, "...#.", ".....", "..##.", "...#.", "...#.", "...#.", "#..#.", ".##.."),
    "k": g(0, "#....", "#....", "#..#.", "###..", "#.#..", "#..#."),
    "l": g(0, ".##..", "..#..", "..#..", "..#..", "..#..", "..###"),   # Haken oben links, Fuß nach rechts
    "m": g(2, "##.#.", "#.#.#", "#.#.#", "#...#"),
    "n": g(2, "####.", "#...#", "#...#", "#...#"),
    "o": g(2, ".###.", "#...#", "#...#", ".###."),
    "p": g(2, "####.", "#...#", "#...#", "####.", "#....", "#...."),
    "q": g(2, ".####", "#...#", "#...#", ".####", "....#", "....#"),
    "r": g(2, "#.##.", "##..#", "#....", "#...."),
    "s": g(2, ".####", "#....", "....#", "####."),
    "t": g(0, ".#...", ".#...", "####.", ".#...", ".#...", "..##."),
    "u": g(2, "#...#", "#...#", "#...#", ".####"),
    "v": g(2, "#...#", "#...#", ".#.#.", "..#.."),
    "w": g(2, "#...#", "#.#.#", "#.#.#", ".#.#."),
    "x": g(2, "#...#", ".#.#.", ".#.#.", "#...#"),
    "y": g(2, "#...#", "#...#", ".####", "....#", "#...#", ".###."),
    "z": g(2, "#####", "..##.", ".##..", "#####"),
}

DIGITS = {
    "0": g(0, ".###.", "#...#", "#.#.#", "#...#", "#...#", ".###."),
    "1": g(0, "..#..", ".##..", "#.#..", "..#..", "..#..", "#####"),   # Fahne mit Aufstrich, Fuß über die ganze Breite
    "2": g(0, ".###.", "#...#", "...#.", "..#..", ".#...", "#####"),
    "3": g(0, "####.", "....#", ".###.", "....#", "....#", "####."),
    "4": g(0, "#...#", "#...#", "#...#", "#####", "....#", "....#"),   # offene Vier
    "5": g(0, "#####", "#....", "####.", "....#", "#...#", ".###."),
    "6": g(0, ".###.", "#....", "####.", "#...#", "#...#", ".###."),
    "7": g(0, "#####", "....#", "...#.", "..#..", "..#..", "..#.."),
    "8": g(0, ".###.", "#...#", ".###.", "#...#", "#...#", ".###."),
    "9": g(0, ".###.", "#...#", "#...#", ".####", "....#", ".###."),
}

PUNCT = {
    " ": g(0),
    "!": g(0, ".###.", "..#..", "..#..", "..#..", ".....", "..#.."),   # Kopfbalken
    '"': g(0, "#...#", "#...#", "#...#"),   # zwei weit gestellte Striche
    "#": g(0, ".#.#.", "#####", ".#.#.", ".#.#.", "#####", ".#.#."),
    "$": g(0, ".#.#.", ".###.", "#.#..", ".###.", "..#.#", ".###.", ".#.#."),
    "%": g(0, "##..#", "##..#", "...#.", "..#..", ".#.##", "#..##"),
    "&": g(0, ".##..", "#..#.", ".##..", ".#.#.", "#..#.", ".##.#"),
    "'": g(0, "##...", "#....", "#...."),
    "(": g(0, "...#.", "..#..", ".#...", ".#...", "..#..", "...#."),
    ")": g(0, ".#...", "..#..", "...#.", "...#.", "..#..", ".#..."),
    "*": g(1, "..#..", "#.#.#", ".###.", "#.#.#", "..#.."),
    "+": g(2, "..#..", ".###.", "..#.."),   # kleines Plus (3×3)
    ",": g(3, "..##.", "..##.", "..#..", ".#..."),
    "-": g(3, "###..", "..###"),   # gestufter Strich
    ".": g(4, "..#..", ".###."),
    "/": g(0, "....#", "...#.", "..#..", "..#..", ".#...", "#...."),
    ":": g(2, ".##..", ".....", ".##.."),
    ";": g(2, ".##..", ".....", "..##.", "..#..", ".#..."),
    "<": g(0, "...##", "..#..", ".#...", "#....", ".#...", "..#..", "...##"),   # Winkel mit Haken
    "=": g(2, ".###.", ".....", ".###."),
    ">": g(0, "##...", "..#..", "...#.", "....#", "...#.", "..#..", "##..."),
    "?": g(0, ".###.", "#...#", "...#.", "..#..", ".....", "..#.."),
    "@": g(0, ".###.", "#...#", "#.###", "#.#.#", "#..##", ".###."),
    "[": g(0, "####.", "#....", "#....", "#....", "#....", "#....", "####."),
    "]": g(0, ".####", "....#", "....#", "....#", "....#", "....#", ".####"),
    "\\": g(0, "#....", ".#...", "..#..", "..#..", "...#.", "....#"),
    "^": g(0, "..#..", ".#.#.", "#...#", "#...#"),
    "_": g(6, "#...#", "#####"),  # Ablage mit Randstrichen; liegt fest auf den Zeilen 6 und 7, wird nie verschoben
    "`": g(0, ".#...", "..#.."),
    "{": g(0, "..##.", "..#..", ".#...", "..#..", "..#..", "..##."),
    "}": g(0, ".##..", "..#..", "...#.", "..#..", "..#..", ".##.."),
    "|": g(0, "..#..", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."),
    "~": g(1, ".#...", "#.#.#", "...#."),
    "£": g(0, "..##.", ".#...", "####.", ".#...", ".#...", "#####"),
    "↑": g(0, "..#..", ".#.#.", "#...#", "..#..", "..#..", "..#.."),
    "←": g(0, "...#.", "..#..", ".#...", "#####", ".#...", "..#..", "...#."),
    "→": g(0, ".#...", "..#..", "...#.", "#####", "...#.", "..#..", ".#..."),
    "↓": g(0, "..#..", "..#..", "..#..", "#...#", ".#.#.", "..#.."),
    "¡": g(0, "..#..", ".....", "..#..", "..#..", "..#..", ".###."),
    "¿": g(0, "..#..", ".....", "..#..", "...#.", "#...#", ".###."),
}

# Zeichen mit dem Stempel „fest“ (nicht mit der Satzlage verschoben)
FEST = {"_"}

ALLTEXT = {}
for _d in (CAPS, LOWER, DIGITS, PUNCT):
    ALLTEXT.update(_d)


def blank():
    return [[0] * 8 for _ in range(8)]


def from_text(key, dy=0, table=None):
    """Zeichenbild (8×8) eines Textzeichens, um dy Zeilen nach unten verschoben (Satzlage)."""
    top, rows = (table or ALLTEXT)[key]
    if key in FEST:
        dy = 0
    b = blank()
    for i, r in enumerate(rows):
        y = top + i + dy
        assert 0 <= y < 8, (key, y)
        assert len(r) == 5, (key, r)
        for x, c in enumerate(r):
            if c == "#":
                b[y][x + 1] = 1
    return b


def with_accent(base_key, accent, dy=0):
    """Akzent (Zeilen 0 und 1, oberhalb der x-Höhe) auf ein kleines Grundzeichen; Versalien werden dafür um eine Zeile gestaucht."""
    top, rows = ALLTEXT[base_key]
    rows = list(rows)
    if base_key.isupper():
        # eine doppelte Zeile entfernen (Regel: Versalie mit Akzent hat 5 statt 6 Zeilen), dann um 1 nach unten
        for i in range(1, len(rows) - 1):
            if rows[i] == rows[i + 1]:
                del rows[i]
                break
        else:
            raise ValueError("keine doppelte Zeile in " + base_key)
        top = 1
    b = blank()
    for i, r in enumerate(rows):
        for x, c in enumerate(r):
            if c == "#":
                b[top + i][x + 1] = 1
    for y, r in enumerate(accent):
        for x, c in enumerate(r):
            if c == "#":
                b[y][x + 1] = 1
    return b


ACC = {
    "acute": ["...#.", "..#.."],
    "grave": [".#...", "..#.."],
    "circ": ["..#..", ".#.#."],
    "diaer": [".....", ".#.#."],
    "tilde": [".#..#", "#.##."],
    "ring": [".##..", ".##.."],
}


# ---------------------------------------------------------------------------------------------------------------------------------
# Grafikzeichen aus Regeln (8×8)
# ---------------------------------------------------------------------------------------------------------------------------------
def art(*rows):
    assert len(rows) == 8 and all(len(r) == 8 for r in rows), rows
    return [[1 if c == "#" else 0 for c in r] for r in rows]


def rect(x0, y0, x1, y1):
    b = blank()
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            b[y][x] = 1
    return b


def union(*bs):
    out = blank()
    for b in bs:
        for y in range(8):
            for x in range(8):
                out[y][x] |= b[y][x]
    return out


def hl(y, x0=0, x1=7):
    return rect(x0, y, x1, y)


def vl(x, y0=0, y1=7):
    return rect(x, y0, x, y1)


def invert(b):
    return [[1 - v for v in r] for r in b]


def diag(down):
    b = blank()
    for i in range(8):
        b[i][i if down else 7 - i] = 1
    return b


def tri(corner):
    """Dreieck, gefüllt, in einer Ecke: 'ul', 'ur', 'll', 'lr'."""
    b = blank()
    for y in range(8):
        for x in range(8):
            u = y < 4
            if corner == "ul" and x + y <= 7:
                b[y][x] = 1
            if corner == "ur" and x >= y:
                b[y][x] = 1
            if corner == "ll" and x <= y:
                b[y][x] = 1
            if corner == "lr" and x + y >= 7:
                b[y][x] = 1
    return b


def checker(phase=0):
    b = blank()
    for y in range(8):
        for x in range(8):
            if (x + y) % 2 == phase:
                b[y][x] = 1
    return b


def pattern25():
    # 25 %: jedes zweite Pixel jeder zweiten Zeile, Zeilen versetzt
    b = blank()
    for y in range(0, 8, 2):
        off = 0 if (y // 2) % 2 == 0 else 1
        for x in range(off, 8, 2):
            b[y][x] = 1
    return b


def hatch():
    b = blank()
    for y in range(8):
        for x in range(8):
            if (x + y) % 4 == 0:
                b[y][x] = 1
    return b


C = 3  # Mitte der Linien (Spalte und Zeile)


def line_h():
    return hl(C)


def line_v():
    return vl(C)


def corner_sq(up, down, left, right):
    b = blank()
    b[C][C] = 1
    if left:
        for x in range(0, C):
            b[C][x] = 1
    if right:
        for x in range(C + 1, 8):
            b[C][x] = 1
    if up:
        for y in range(0, C):
            b[y][C] = 1
    if down:
        for y in range(C + 1, 8):
            b[y][C] = 1
    return b


def heart():
    return art(".##..##.", "########", "########", "########", ".######.", "..####..", "...##...", "........")


def spade():
    return art("...##...", "..####..", ".######.", "########", "########", "...##...", "...##...", "..####..")


def diamond():
    return art("...##...", "..####..", ".######.", "########", ".######.", "..####..", "...##...", "........")


def club():
    return art("...##...", "..####..", "...##...", ".##..##.", "########", ".##..##.", "...##...", "..####..")


def circle_open():
    return art("..####..", ".#....#.", "#......#", "#......#", "#......#", "#......#", ".#....#.", "..####..")


def circle_full():
    return art("..####..", ".######.", "########", "########", "########", "########", ".######.", "..####..")


def pi_sign():
    return art("........", "#######.", ".#..#...", ".#..#...", ".#..#...", ".#..##..", "........", "........")


def check_mark():
    return art("........", ".......#", "......##", "#....##.", "##..##..", ".####...", "..##....", "........")


def corner_blocks(kind):
    b = {"ul": rect(0, 0, 3, 3), "ur": rect(4, 0, 7, 3), "ll": rect(0, 4, 3, 7), "lr": rect(4, 4, 7, 7)}
    return union(*[b[k] for k in kind])


def rounded(up, down, left, right):
    """Abgerundete Ecke (Fasenregel): genau ein waagerechter und ein senkrechter Arm, das Eckpixel in der Mitte fehlt, die Arme berühren sich diagonal."""
    assert up != down and left != right
    b = blank()
    for x in (range(0, C) if left else range(C + 1, 8)):
        b[C][x] = 1
    for y in (range(0, C) if up else range(C + 1, 8)):
        b[y][C] = 1
    return b


# ---------------------------------------------------------------------------------------------------------------------------------
# Belegung C64 (Bildschirmcodes = PETSCII $60 bis $7F als 64 bis 95, $A0 bis $BF als 96 bis 127). Quelle der Zuordnung Code → Bedeutung: die
# öffentlichen Unicode-Zuordnungstabellen (Namen, z. B. „LEFT ONE QUARTER BLOCK“), siehe BELEGUNG unten; die Formen sind eigene Regeln.
# Linien-Leitern: waagerecht Zeile 0 ($A3), 1 ($65), 2 ($64), 3 ($60), 4 ($63), 5 ($66), 6 ($72), 7 ($A4); senkrecht Spalte 0 ($A5), 1 ($74), 2 ($67),
# 3 ($7D), 4 ($62), 5 ($68), 6 ($79), 7 ($A7): „eine Viertel nach oben/unten/links/rechts“ der Namen heißt hier eine bzw. zwei Zeilen/Spalten Versatz.
# ---------------------------------------------------------------------------------------------------------------------------------
def half_shade(half):
    """Mittleres Raster (50 %) nur in einer Hälfte: 'l' links, 'o' oben, 'u' unten."""
    b = blank()
    for y in range(8):
        for x in range(8):
            ok = (half == "l" and x < 4) or (half == "o" and y < 4) or (half == "u" and y >= 4)
            if ok and (x + y) % 2 == 0:
                b[y][x] = 1
    return b


def stripes(left):
    """Schraffur (50 %): zwei Pixel breite Schrägstreifen, 'left' = nach links geneigt (\\), sonst nach rechts (/)."""
    b = blank()
    for y in range(8):
        for x in range(8):
            if ((x - y) if left else (x + y)) % 4 in (0, 1):
                b[y][x] = 1
    return b


def graphics(lower=False):
    G = {}
    G[64] = hl(3)                                # $60
    G[65] = spade()                              # $61
    G[66] = vl(4)                                # $62 senkrechte Linie
    G[67] = hl(4)                                # $63 waagerechte Linie
    G[68] = hl(2)                                # $64 eine Viertel nach oben
    G[69] = hl(1)                                # $65 zwei Viertel nach oben
    G[70] = hl(5)                                # $66 eine Viertel nach unten
    G[71] = vl(2)                                # $67 eine Viertel nach links
    G[72] = vl(5)                                # $68 eine Viertel nach rechts
    G[73] = rounded(False, True, True, False)    # $69 ╮ links + unten
    G[74] = rounded(True, False, False, True)    # $6A ╰ oben + rechts
    G[75] = rounded(True, False, True, False)    # $6B ╯ oben + links
    G[76] = union(vl(0), hl(7))                  # $6C Achtelblock oben und rechts (Winkel links + unten am Rand)
    G[77] = diag(True)                           # $6D ╲
    G[78] = diag(False)                          # $6E ╱
    G[79] = union(vl(0), hl(0))                  # $6F Achtelblock unten und rechts (Winkel links + oben am Rand)
    G[80] = union(vl(7), hl(0))                  # $70 Achtelblock unten und links (Winkel rechts + oben am Rand)
    G[81] = circle_full()                        # $71 ●
    G[82] = hl(6)                                # $72 zwei Viertel nach unten
    G[83] = heart()                              # $73
    G[84] = vl(1)                                # $74 zwei Viertel nach links
    G[85] = rounded(False, True, False, True)    # $75 ╭ unten + rechts
    G[86] = union(diag(True), diag(False))       # $76 ╳
    G[87] = circle_open()                        # $77 ○
    G[88] = club()                               # $78
    G[89] = vl(6)                                # $79 zwei Viertel nach rechts
    G[90] = diamond()                            # $7A
    G[91] = corner_sq(True, True, True, True)    # $7B ┼
    G[92] = half_shade("l")                      # $7C linke Hälfte mittleres Raster
    G[93] = line_v()                             # $7D │
    G[94] = pi_sign()                            # $7E π (Satz 2: ▒)
    G[95] = tri("ur")                            # $7F ◥ (Satz 2: Raster nach links geschrägt)
    G[96] = blank()                              # $A0
    G[97] = rect(0, 0, 3, 7)                     # $A1 ▌
    G[98] = rect(0, 4, 7, 7)                     # $A2 ▄
    G[99] = hl(0)                                # $A3 oberes Achtel
    G[100] = hl(7)                               # $A4 unteres Achtel
    G[101] = vl(0)                               # $A5 linkes Achtel
    G[102] = checker(0)                          # $A6 ▒
    G[103] = vl(7)                               # $A7 rechtes Achtel
    G[104] = half_shade("u")                     # $A8 untere Hälfte mittleres Raster
    G[105] = tri("ul")                           # $A9 ◤ (Satz 2: Raster nach rechts geschrägt)
    G[106] = rect(6, 0, 7, 7)                    # $AA rechtes Viertel
    G[107] = corner_sq(True, True, False, True)    # $AB ├
    G[108] = corner_blocks(["lr"])               # $AC kleines Quadrat unten rechts
    G[109] = corner_sq(True, False, False, True)   # $AD └
    G[110] = corner_sq(False, True, True, False)   # $AE ┐
    G[111] = rect(0, 6, 7, 7)                    # $AF unteres Viertel
    G[112] = corner_sq(False, True, False, True)   # $B0 ┌
    G[113] = corner_sq(True, False, True, True)    # $B1 ┴
    G[114] = corner_sq(False, True, True, True)    # $B2 ┬
    G[115] = corner_sq(True, True, True, False)    # $B3 ┤
    G[116] = rect(0, 0, 1, 7)                    # $B4 linkes Viertel
    G[117] = rect(0, 0, 2, 7)                    # $B5 linke drei Achtel
    G[118] = rect(5, 0, 7, 7)                    # $B6 rechte drei Achtel
    G[119] = rect(0, 0, 7, 1)                    # $B7 oberes Viertel
    G[120] = rect(0, 0, 7, 2)                    # $B8 obere drei Achtel
    G[121] = rect(0, 5, 7, 7)                    # $B9 untere drei Achtel
    G[122] = union(vl(7), hl(7))                 # $BA Achtelblock oben und links (Winkel rechts + unten am Rand) (Satz 2: Häkchen)
    G[123] = corner_blocks(["ll"])               # $BB kleines Quadrat unten links
    G[124] = corner_blocks(["ur"])               # $BC kleines Quadrat oben rechts
    G[125] = corner_sq(True, False, True, False)   # $BD ┘
    G[126] = corner_blocks(["ul"])               # $BE kleines Quadrat oben links
    G[127] = corner_blocks(["ul", "lr"])         # $BF zwei kleine Quadrate, diagonal von links oben nach rechts unten
    if lower:
        # Satz 2 (Klein/Groß): genau vier Grafikzeichen wechseln (PET/CBM ebenso: COMPUTE! 26, S. 130)
        G[94] = checker(0)
        G[95] = stripes(True)
        G[105] = stripes(False)
        G[122] = check_mark()
    return G


GFX = graphics()
GFX_LOWER = graphics(lower=True)


def c64_upper_half(sc28="£"):
    """Bildschirmcodes 0 bis 127 des Satzes Groß/Grafik (sc28: Zeichen auf Code 28, C64: £, PET/CBM: \\)."""
    chars = []
    for code in range(128):
        if code == 0:
            ch = "@"
        elif 1 <= code <= 26:
            ch = chr(64 + code)
        elif code == 27:
            ch = "["
        elif code == 28:
            ch = sc28
        elif code == 29:
            ch = "]"
        elif code == 30:
            ch = "↑"
        elif code == 31:
            ch = "←"
        elif 32 <= code <= 63:
            ch = chr(code)
        else:
            chars.append(GFX[code])
            continue
        chars.append(from_text(ch, dy=1))
    return chars


def c64_lower_half(sc28="£"):
    chars = []
    for code in range(128):
        if code == 0:
            ch = "@"
        elif 1 <= code <= 26:
            ch = chr(96 + code)
        elif code == 27:
            ch = "["
        elif code == 28:
            ch = sc28
        elif code == 29:
            ch = "]"
        elif code == 30:
            ch = "↑"
        elif code == 31:
            ch = "←"
        elif 32 <= code <= 63:
            ch = chr(code)
        elif 65 <= code <= 90:
            ch = chr(code)  # Versalien an den Codes 65 bis 90
        else:
            chars.append(GFX_LOWER[code])
            continue
        chars.append(from_text(ch, dy=0))
    return chars


def with_inverse(half):
    return half + [invert(b) for b in half]


# ---------------------------------------------------------------------------------------------------------------------------------
# Atari 8-Bit (interne Codes ANTIC): 0 bis 63 = ASCII 32 bis 95, 64 bis 95 = Steuerzeichen-Grafik, 96 bis 127 = ASCII 96 bis 127
# ---------------------------------------------------------------------------------------------------------------------------------
def atari_graphics():
    return [
        heart(),                               # 0 ♥
        corner_sq(True, True, False, True),    # 1 ├
        rect(6, 0, 7, 7),                      # 2 rechter Rand
        corner_sq(True, False, True, False),   # 3 ┘
        corner_sq(True, True, True, False),    # 4 ┤
        corner_sq(False, True, True, False),   # 5 ┐
        diag(False),                           # 6 ╱
        diag(True),                            # 7 ╲
        tri("lr"),                             # 8 ◢
        corner_blocks(["lr"]),                 # 9 ▗
        tri("ll"),                             # 10 ◣
        corner_blocks(["ur"]),                 # 11 ▝
        corner_blocks(["ul"]),                 # 12 ▘
        rect(0, 0, 7, 1),                      # 13 oberes Viertel
        rect(0, 6, 7, 7),                      # 14 unteres Viertel
        corner_blocks(["ll"]),                 # 15 ▖
        club(),                                # 16 ♣
        corner_sq(False, True, False, True),   # 17 ┌
        line_h(),                              # 18 ─
        corner_sq(True, True, True, True),     # 19 ┼
        art("........", "........", "..####..", ".######.", ".######.", "..####..", "........", "........"),  # 20 •
        rect(0, 4, 7, 7),                      # 21 ▄
        rect(0, 0, 1, 7),                      # 22 linker Rand
        corner_sq(False, True, True, True),    # 23 ┬
        corner_sq(True, False, True, True),    # 24 ┴
        rect(0, 0, 3, 7),                      # 25 ▌
        corner_sq(True, False, False, True),   # 26 └
        art("..####..", ".#....#.", "#..##..#", "#.#..#.#", "#.#..#.#", "#..##..#", ".#....#.", "..####.."),  # 27 Esc-Zeichen (Kreis mit Zeichen)
        from_text("↑"), from_text("↓"), from_text("←"), from_text("→"),  # 28 bis 31
    ]


def atari_arrow_clear():
    return art("####....", "###.....", "#.##....", "#..##...", "....##..", ".....##.", "......#.", "........")


def atari_standard():
    out = []
    for code in range(64):
        out.append(from_text(chr(32 + code)))
    out += atari_graphics()
    out.append(diamond())  # 96 ♦
    for c in "abcdefghijklmnopqrstuvwxyz":
        out.append(from_text(c))
    out.append(spade())                 # 123 ♠
    out.append(from_text("|"))          # 124
    out.append(atari_arrow_clear())     # 125 Bildschirm löschen
    out.append(art("........", ".#......", "##......", "#######.", "##......", ".#......", "........", "........"))  # 126 Rückschritt (Pfeil links... gespiegelt unten)
    out.append(art("........", "..#.....", "..##....", "..###...", "#######.", "..###...", "..##....", "..#....."))  # 127 Tabulator
    return out


# Variante „International“ (XL/XE): ersetzt die internen Codes 64 bis 90 (ATASCII 0 bis 26) sowie 96 (♦ → ¡) und 123 (♠ → Ä); 91 bis 95 (Esc, Pfeile)
# bleiben. Quellen: Wikipedia „ATASCII“ und Altirra Hardware Reference Manual (Kapitel 9, Zeichentabelle des Druckers 1020).
INTL = [
    ("a", "acute"), ("u", "grave"), ("N", "tilde"), ("E", "acute"), ("ç",), ("o", "circ"), ("o", "grave"), ("i", "grave"),   # 64 bis 71: á ù Ñ É ç ô ò ì
    ("£",), ("i", "diaer"), ("u", "diaer"), ("a", "diaer"), ("O", "diaer"), ("u", "acute"), ("o", "acute"), ("o", "diaer"),   # 72 bis 79: £ ï ü ä Ö ú ó ö
    ("U", "diaer"), ("a", "circ"), ("u", "circ"), ("i", "circ"), ("e", "acute"), ("e", "grave"), ("n", "tilde"), ("e", "circ"),   # 80 bis 87: Ü â û î é è ñ ê
    ("a", "ring"), ("a", "grave"), ("A", "ring"),                                                                           # 88 bis 90: å à Å
]
INTL_96 = ("¡",)
INTL_123 = ("A", "diaer")


def intl_glyph(spec):
    if len(spec) == 2:
        base, acc = spec
        # i ohne Punkt unter dem Akzent
        if base == "i":
            b = blank()
            for i, r in enumerate([".##..", "..#..", "..#..", ".###."]):
                for x, c in enumerate(r):
                    if c == "#":
                        b[2 + i][x + 1] = 1
            for y, r in enumerate(ACC[acc]):
                for x, c in enumerate(r):
                    if c == "#":
                        b[y][x + 1] = 1
            return b
        return with_accent(base, ACC[acc])
    ch = spec[0]
    if ch == "ß":
        return art("........", ".###....", "#...#...", "#..#....", "#...#...", "#...#...", "#.##....", "#.......")
    if ch in "çÇ":
        base = from_text("c" if ch == "ç" else "C")
        base[6][3] = 1
        base[7][2] = 1
        return base
    if ch == "ø":
        b = from_text("o")
        b[2][5] = 1
        b[3][4] = 1
        b[4][2] = 1
        b[5][1] = 1
        return b
    return from_text(ch)


def atari_international():
    std = atari_standard()
    out = list(std[:64]) + [intl_glyph(s) for s in INTL] + list(std[91:96]) + [intl_glyph(INTL_96)] + list(std[97:123]) + [intl_glyph(INTL_123)] + list(std[124:])
    assert len(out) == 128
    return out


# ---------------------------------------------------------------------------------------------------------------------------------
# Ausgabe
# ---------------------------------------------------------------------------------------------------------------------------------
def rows_of(b):
    return ["".join("1" if v else "0" for v in r) for r in b]


def set_hash(chars_rows):
    """SHA-256 des Satzes: Zeichen in Codereihenfolge, je Zeichen die Zeilen mit '|' verbunden, Zeichen mit '\\n' getrennt (UTF-8)."""
    return hashlib.sha256("\n".join("|".join(c) for c in chars_rows).encode("ascii")).hexdigest()


def double_rows(rows):
    out = []
    for r in rows:
        out += [r, r]
    return out


def fcm_rows(rows):
    return ["".join("01" if c == "1" else "00" for c in r) for r in rows]


# ---------------------------------------------------------------------------------------------------------------------------------
# Herkunft der Codebelegung (nur Codes und Zeichennamen aus öffentlichen Tabellen; keine Zeichenformen, keine ROM-Abbilder)
# ---------------------------------------------------------------------------------------------------------------------------------
BELEGUNG_DATUM = "2026-11-16"
Q_LTH = "Unicode-Zuordnung PETSCII von Triad (www.df.lth.se/~triad/krad/recode/: petscii_c64en_uc.txt, petscii_c64en_lc.txt, petscii_vic20en_uc.txt; nur Codes und Zeichennamen)"
Q_AIV = "Aivosto, Commodore PETSCII character sets (PDF, 2014; Tabellen C64, VIC-20, PET 8032 und Bildschirmcodes; nur Codes und Zeichennamen)"
Q_WIKI_P = "Wikipedia, Artikel PETSCII (Unicode-Zuordnung, Unterschiede PET/VIC-20/C64)"
Q_COMPUTE = "COMPUTE! Nr. 26, Juli 1982, S. 130, All About PET/CBM Character Sets (zwei Sätze wechseln 26 Buchstaben und vier Grafikzeichen)"
Q_WIKI_A = "Wikipedia, Artikel ATASCII (Tabellen Standard- und internationaler Satz)"
Q_ALTIRRA = "Altirra Hardware Reference Manual (Avery Lee), Kapitel 9, Tabelle der internationalen Zeichen (Codes 00 bis 1A, 1C bis 1F, 60, 7B)"
Q_PLUS4 = "plus4world.powweb.com, Forum (Grundzeichensatz von C64, Plus/4 und C128 gleich definiert; TED erzeugt die Invers-Hälfte selbst)"
Q_PLUS4_DOC = "Plus/4 Docs (Commodore, Abschnitt 11.1 und TED-Beschreibung): Zeichen-ROM $D000 bis $D7FF (2 KB); bei Standardbetrieb 128 Zeichen je Satz, die Invers-Hälfte erzeugt der TED über das Umkehrbit; Zeichenbasis in 1-KB-Schritten (Register $FF13)"

BELEGUNG = {
    "vc-c64-upper": ("quellengeprueft", [Q_LTH, Q_WIKI_P, Q_AIV], ""),
    "vc-c64-lower": ("quellengeprueft", [Q_LTH, Q_WIKI_P, Q_AIV], "Satz 2 wechselt gegenüber Satz 1 genau die Buchstaben und vier Grafikzeichen (Codes 94, 95, 105, 122)."),
    "vc-atari-standard": ("quellengeprueft", [Q_WIKI_A], "Nur eine Quelle (Tabelle) für die Grafikzeichen auf den Codes 0 bis 26 und 96, 123 bis 127; das Altirra-Handbuch bestätigt für 7D bis 7F nur die Bedeutung (Home, Rückschritt, Tab)."),
    "vc-atari-international": ("quellengeprueft", [Q_WIKI_A, Q_ALTIRRA], "Ersetzt 27 Zeichen (interne Codes 64 bis 90) sowie 96 und 123; Esc und Pfeile bleiben."),
    "vc-plus4-standard": ("quellengeprueft", [Q_AIV, Q_PLUS4, Q_PLUS4_DOC], "Belegung wie C64. Das Zeichen-ROM (2 KB) enthält zwei Sätze zu je 128 Zeichen (Groß/Grafik, Klein/Groß), die Invers-Hälfte erzeugt der TED; die Reihenfolge im ROM (Groß/Grafik zuerst) ist an VICE-Abbildern gemessen."),
    "vc-plus4-lower": ("quellengeprueft", [Q_AIV, Q_PLUS4, Q_PLUS4_DOC], "Belegung wie C64 Satz 2 (Klein/Groß). Das Zeichen-ROM (2 KB) enthält zwei Sätze zu je 128 Zeichen, die Invers-Hälfte erzeugt der TED; im ROM liegt er an zweiter Stelle (gemessen an VICE-Abbildern)."),
    "vc-pet-upper": ("quellengeprueft", [Q_AIV, Q_COMPUTE, Q_WIKI_P], "Wie VIC-20/C64, nur Code 28 ist ein Rückwärtsschrägstrich statt £ (PET 8032)."),
    "vc-pet-lower": ("quellengeprueft", [Q_AIV, Q_COMPUTE, Q_WIKI_P], "Wie VIC-20/C64, nur Code 28 ist ein Rückwärtsschrägstrich statt £ (PET 8032). Das PET 2001 der ersten Serie hat vertauschte Groß- und Kleinbuchstaben (nicht angelegt)."),
    "vc-vic20-upper": ("quellengeprueft", [Q_LTH, Q_AIV], "Die Zuordnung der Grafikzeichen ist laut Tabelle gleich der des C64."),
    "vc-vic20-lower": ("quellengeprueft", [Q_LTH, Q_AIV], "Die Zuordnung der Grafikzeichen ist laut Tabelle gleich der des C64."),
    "vc-c128-upper": ("quellengeprueft", [Q_AIV], "Gleiche PETSCII-Belegung wie C64 (Tabelle nennt C64, C16, C128 zusammen); Bildschirmcodes des 40-Spalten-Satzes nicht einzeln gegen eine C128-Tabelle geprüft."),
    "vc-c128-lower": ("quellengeprueft", [Q_AIV], "Gleiche PETSCII-Belegung wie C64 (Tabelle nennt C64, C16, C128 zusammen); Bildschirmcodes des 40-Spalten-Satzes nicht einzeln gegen eine C128-Tabelle geprüft."),
    "vc-c128-vdc-upper": ("vorlaeufig", [], "Keine Quelle für die Codebelegung des VDC-Zeichenspeichers abgefragt; Annahme: wie der 40-Spalten-Satz."),
    "vc-c128-vdc-lower": ("vorlaeufig", [], "Keine Quelle für die Codebelegung des VDC-Zeichenspeichers abgefragt; Annahme: wie der 40-Spalten-Satz."),
    "vc-mega65-upper": ("vorlaeufig", [], "Keine Quelle abgefragt; Annahme: Bildschirmcodes wie C64 (Vollfarbmodus, zwei Hex-Ziffern je Pixel)."),
    "vc-mega65-lower": ("vorlaeufig", [], "Keine Quelle abgefragt; Annahme: Bildschirmcodes wie C64 (Vollfarbmodus, zwei Hex-Ziffern je Pixel)."),
}


# Satzliste: (id, Systemordner, Profil, Satz, Name, Farben, Zeichen als Zeilenlisten, abgeleitet von)
def build_sets():
    up = [rows_of(b) for b in with_inverse(c64_upper_half())]
    lo = [rows_of(b) for b in with_inverse(c64_lower_half())]
    at_std = [rows_of(b) for b in atari_standard()]
    at_int = [rows_of(b) for b in atari_international()]
    sets = []

    def add(sid, system, profile, satz, name, colors, chars, basis=None, bild=None, inverse=True):
        # chars = Zeilen, wie sie in der Datei stehen; bild = dasselbe als 1-Bit-Bild 8×8 (für Ausnahmeliste und Eindeutigkeit)
        sets.append(dict(id=sid, system=system, profile=profile, satz=satz, name=name, colors=colors, chars=chars, basis=basis,
                         bild=bild if bild is not None else chars, inverse=inverse))

    add("vc-c64-upper", "c64", "c64.char.hires", "upper", "VC-Standard C64 Groß/Grafik", {"fg": 1, "bg": 6}, up)
    add("vc-c64-lower", "c64", "c64.char.hires", "lower", "VC-Standard C64 Klein/Groß", {"fg": 1, "bg": 6}, lo)
    add("vc-atari-standard", "atari8", "atari.char.antic2", "standard", "VC-Standard Atari", {"fg": 14, "bg": 0}, at_std, inverse=False)
    add("vc-atari-international", "atari8", "atari.char.antic2", "international", "VC-Standard Atari International", {"fg": 14, "bg": 0}, at_int, inverse=False)
    add("vc-plus4-standard", "plus4", "plus4.char.hires", "upper", "VC-Standard Plus/4 und C16 Groß/Grafik", {"fg": 1, "bg": 6}, up, basis=["vc-c64-upper"])
    pet_up = [rows_of(b) for b in with_inverse(c64_upper_half("\\"))]
    pet_lo = [rows_of(b) for b in with_inverse(c64_lower_half("\\"))]
    add("vc-plus4-lower", "plus4", "plus4.char.hires", "lower", "VC-Standard Plus/4 und C16 Klein/Groß", {"fg": 1, "bg": 6}, lo, basis=["vc-c64-lower"])
    add("vc-pet-upper", "pet", "pet.char.hires", "upper", "VC-Standard PET Groß/Grafik", {"fg": 1, "bg": 0}, pet_up, basis=["vc-c64-upper"])
    add("vc-pet-lower", "pet", "pet.char.hires", "lower", "VC-Standard PET Klein/Groß", {"fg": 1, "bg": 0}, pet_lo, basis=["vc-c64-lower"])
    add("vc-vic20-upper", "vic20", "vic20.char.hires", "upper", "VC-Standard VC-20 Groß/Grafik", {"fg": 0, "bg": 1}, up, basis=["vc-c64-upper"])
    add("vc-vic20-lower", "vic20", "vic20.char.hires", "lower", "VC-Standard VC-20 Klein/Groß", {"fg": 0, "bg": 1}, lo, basis=["vc-c64-lower"])
    add("vc-c128-upper", "c128", "c128.char.hires", "upper", "VC-Standard C128 40 Spalten Groß/Grafik", {"fg": 1, "bg": 6}, up, basis=["vc-c64-upper"])
    add("vc-c128-lower", "c128", "c128.char.hires", "lower", "VC-Standard C128 40 Spalten Klein/Groß", {"fg": 1, "bg": 6}, lo, basis=["vc-c64-lower"])
    add("vc-c128-vdc-upper", "c128", "c128.char.vdc", "vdc-upper", "VC-Standard C128 VDC 80 Spalten Groß/Grafik", {"fg": 15, "bg": 0},
        [double_rows(c) for c in up], basis=["vc-c64-upper"], bild=up)
    add("vc-c128-vdc-lower", "c128", "c128.char.vdc", "vdc-lower", "VC-Standard C128 VDC 80 Spalten Klein/Groß", {"fg": 15, "bg": 0},
        [double_rows(c) for c in lo], basis=["vc-c64-lower"], bild=lo)
    add("vc-mega65-upper", "mega65", "mega65.char.fcm", "upper", "VC-Standard MEGA65 Groß/Grafik (Vollfarbmodus)", {"palette": ""},
        [fcm_rows(c) for c in up], basis=["vc-c64-upper"], bild=up)
    add("vc-mega65-lower", "mega65", "mega65.char.fcm", "lower", "VC-Standard MEGA65 Klein/Groß (Vollfarbmodus)", {"palette": ""},
        [fcm_rows(c) for c in lo], basis=["vc-c64-lower"], bild=lo)
    return sets


# Zeichen mit zwingender Geometrie: jede Gestaltung zeichnet sie gleich (Stilblatt, Abschnitt 5)
def forced_kind(rows):
    """Name der Klasse oder None. rows: 8 Strings zu 8 Bit (1-Bit-Bild)."""
    b = [[int(c) for c in r] for r in rows]
    n = sum(map(sum, b))
    if n == 0:
        return "leer"
    if n == 64:
        return "vollblock"
    # Vereinigung der vier Viertel (Ecken)
    quads = {"ul": (0, 0), "ur": (4, 0), "ll": (0, 4), "lr": (4, 4)}
    inq = []
    for k, (x0, y0) in quads.items():
        s = sum(b[y][x] for y in range(y0, y0 + 4) for x in range(x0, x0 + 4))
        if s == 16:
            inq.append(k)
        elif s != 0:
            inq = None
            break
    if inq is not None and inq:
        return "block"
    # Balken an einem Rand (volle Breite oder Höhe)
    if is_bar(b):
        return "balken"
    # Muster (Raster 25/50/75 %)
    muster = [checker(0), checker(1), pattern25(), invert(pattern25()), half_shade("l"), half_shade("o"), half_shade("u"), stripes(True), stripes(False)]
    if b in muster or invert(b) in muster:
        return "muster"
    # Geometrisch erzwungen: auch die Bitnegation (Codes 128 bis 255 sind die Negation von 0 bis 127) gehört dazu
    if geometric(b) or geometric(invert(b)) or is_bar(invert(b)):
        return "geometrisch"
    return None


def grafikcode(system, satz, code):
    """Ist der Code ein Grafikzeichen (nicht Buchstabe, Ziffer, Satzzeichen)? Nur dort gilt die Klasse „geometrisch“: Buchstaben und Symbole werden nie
    als geometrisch erzwungen ausgenommen, auch wenn ein Strich (! | : …) so aussieht."""
    c = code % 128
    if system == "atari8":
        return 64 <= c <= 95 or c >= 123
    return 64 <= c <= 127 and not (satz.endswith("lower") and 65 <= c <= 90)


def is_bar(b):
    """Volle Zeilen oder Spalten, zusammenhängend (Balken, auch ein einzelner Strich an beliebiger Stelle)."""
    for y0 in range(8):
        for y1 in range(y0, 8):
            if all(all(b[y][x] for x in range(8)) if y0 <= y <= y1 else not any(b[y]) for y in range(8)):
                return True
    for x0 in range(8):
        for x1 in range(x0, 8):
            if all(all(b[y][x] for y in range(8)) if x0 <= x <= x1 else not any(b[y][x] for y in range(8)) for x in range(8)):
                return True
    return False


def geometric(b):
    """Linien, Rahmen, Ecken, T-Stücke, Kreuze, Dreiecke und Diagonalen: Formen, die an den Nachbarzellen nahtlos anschließen müssen und die jede
    Gestaltung gleich zeichnet (Entscheidung des Projektinhabers 2026-11-17). b: 8×8 Listen aus 0/1."""
    pix = {(x, y) for y in range(8) for x in range(8) if b[y][x]}
    if not pix:
        return False
    # Linien durch die Mitte (Spalte 3 / Zeile 3): Linie, Ecke, abgerundete Ecke, T-Stück, Kreuz
    if all(x == C or y == C for x, y in pix):
        return True
    # Rahmen: Vereinigung ganzer Randlinien (Zeile 0/7, Spalte 0/7), also Winkel und Rahmenstücke
    rand = [hl(0), hl(7), vl(0), vl(7)]
    for mask in range(1, 16):
        if b == union(*[r for i, r in enumerate(rand) if mask >> i & 1]):
            return True
    # Dreiecke und Diagonalen
    if any(b == tri(c) for c in ("ul", "ur", "ll", "lr")):
        return True
    return b in (diag(True), diag(False), union(diag(True), diag(False)))


def text_of(s, rows_text):
    head = '{\n  "format": 1,\n  "kind": "charset",\n  "profile": %s,\n  "name": %s,\n  "colors": %s,\n' % (
        json.dumps(s["profile"]), json.dumps(s["id"]), json.dumps(s["colors"], separators=(",", ":")))
    origin = {
        "id": s["id"], "rev": 1, "source": "vintagecoder-standard", "author": AUTHOR, "created": CREATED, "tool": TOOL, "style": STIL,
        "license": LICENSE,
        "check": {"date": None, "tool": "charset_distance 1", "result": "offen", "min_distance": None, "exceptions": s["exceptions"],
                  "set_sha256": s["sha"]},
        # Gestaltung durch den Projektinhaber freigegeben (2026-11-17); die Abstandsprüfung (origin.check) bleibt „offen“: sie wurde nicht bestanden und nicht gestempelt
        "freigabe": "erteilt", "freigabe_datum": "2026-11-17",
        "freigabe_vermerk": "Gestaltung vom Projektinhaber freigegeben; Abstandsprüfung nicht bestanden, Rest vom Projektinhaber akzeptiert",
    }
    bel = BELEGUNG[s["id"]]
    origin["belegung"] = {"status": bel[0], "datum": BELEGUNG_DATUM, "geprueft_gegen": bel[1], "hinweis": bel[2]}
    if s["basis"]:
        origin["basis"] = s["basis"]
    head += '  "origin": %s,\n  "frames": [\n' % json.dumps(origin, ensure_ascii=False, separators=(",", ":"))
    body = ",\n".join("    " + json.dumps({"rows": r}, separators=(",", ":")) for r in rows_text)
    return head + body + "\n  ]\n}\n"


def main(argv):
    check = "--check" in argv
    sets = build_sets()
    files = {}
    index = []
    ausnahmen = {"_comment": "Zeichen mit zwingender Geometrie (leer, Vollblock, Viertel-/Halbblöcke, Balken, 25/50/75-%-Raster, Linien/Rahmen/Ecken/T-Stücke/Kreuze/Dreiecke/Diagonalen): jede Gestaltung zeichnet sie gleich, "
                             "die Gleichheitsprüfung beanstandet sie nicht. Erzeugt von tools/gen_vc_standard.py aus den Zeichenbildern, Begründung in data/charsets/STIL.md Abschnitt 5.",
                 "klassen": {
                     "leer": "Leerzeichen: kein Pixel",
                     "vollblock": "Vollblock: alle Pixel",
                     "block": "Viertel- und Halbblöcke: Vereinigung der vier 4×4-Ecken",
                     "balken": "Balken: volle Zeilen oder Spalten, die an einem Rand oder in der Mitte durchgehen",
                     "muster": "Raster 25 %, 50 %, 75 %, Raster in einer Hälfte, Schrägschraffur; auch als Negativ",
                     "geometrisch": "geometrisch erzwungen (Nachbarzellen müssen nahtlos anschließen): Linien, Rahmen, Ecken, T-Stücke, Kreuze, Dreiecke, Diagonalen; auch als Negativ"},
                 "saetze": {}}
    for s in sets:
        s["sha"] = set_hash(s["chars"])
        ex = {}
        for code, ch in enumerate(s["bild"]):
            k = forced_kind(ch)
            if k == "geometrisch" and not grafikcode(s["system"], s["satz"], code):
                k = None
            if k:
                ex[str(code)] = k
        s["exceptions"] = len(ex)
        ausnahmen["saetze"][s["id"]] = ex
        rel = "%s/%s.vcchar" % (s["system"], s["id"])
        files[rel] = text_of(s, s["chars"])
        index.append({"id": s["id"], "system": s["system"], "profil": s["profile"], "satz": s["satz"], "name": s["name"], "revision": 1,
                      "lizenz": LICENSE, "datei": rel, "zeichen": len(s["chars"]), "hoehe": len(s["chars"][0]),
                      "abgeleitet_von": s["basis"] or [], "belegung": BELEGUNG[s["id"]][0], "belegung_datum": BELEGUNG_DATUM,
                      "satz_sha256": s["sha"]})
    files["index.json"] = json.dumps({"_comment": "Eigene Standardzeichensätze „VC-Standard“ (CC0-1.0). Erzeugt von tools/gen_vc_standard.py; keine Original-ROM-Zeichen, keine Nachbauten.",
                                      "format": 1, "stil": STIL, "saetze": index}, ensure_ascii=False, indent=2) + "\n"
    files["ausnahmen.json"] = json.dumps(ausnahmen, ensure_ascii=False, indent=1) + "\n"
    bad = 0
    for rel, txt in files.items():
        p = os.path.join(OUT, rel)
        if check:
            have = open(p, encoding="utf-8").read() if os.path.exists(p) else None
            if have != txt:
                print("abweichend:", rel)
                bad = 1
        else:
            os.makedirs(os.path.dirname(p), exist_ok=True)
            open(p, "w", encoding="utf-8").write(txt)
    if not check:
        print("%d Dateien unter data/charsets/ geschrieben" % len(files))
    return bad


if __name__ == "__main__":
    sys.exit(main(sys.argv))
