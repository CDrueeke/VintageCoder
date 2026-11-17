#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt das offizielle VintageCoder-Logo in allen Varianten nach branding/ (SVG und PNG, nur Standardbibliothek).

Das Logo besteht aus Pixelblöcken (5x7-Pixelschrift, Lichtkante oben links, Schatten unten rechts) und einem gelben Punkt:
  - Icon     "VC."           auf violettem Kachelgrund (App-Icon; dieselbe Zeichnung wie tools/gen_icon.py)
  - Wortmarke "VintageCoder." auf violettem Grund (breit)
  - Marke / Wortmarke ohne Grund (transparent) in zwei Farbfassungen:
      -light  helle Blöcke, für dunkle Flächen
      -dark   dunkle Blöcke, für helle Flächen
Der gelbe Punkt ist in allen Fassungen gleich.

Aufruf (Arbeitsordner egal): python3 tools/gen_logo.py"""
import argparse, os, struct, zlib

argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter).parse_args()

G = {   # Zeichen -> (erste Zeile, Zeilen); Großbuchstaben 7 Zeilen, Kleinbuchstaben ab Zeile 2, "g" mit Unterlänge
    "V": (0, ["X...X", "X...X", "X...X", "X...X", ".X.X.", ".X.X.", "..X.."]),
    "C": (0, [".XXX.", "X...X", "X....", "X....", "X....", "X...X", ".XXX."]),
    "i": (0, ["X", "", "X", "X", "X", "X", "X"]),
    "n": (2, ["XXXX.", "X...X", "X...X", "X...X", "X...X"]),
    "t": (1, [".X..", "XXXX", ".X..", ".X..", ".X..", ".XXX"]),
    "a": (2, [".XXX.", "....X", ".XXXX", "X...X", ".XXXX"]),
    "g": (2, [".XXXX", "X...X", "X...X", "X...X", ".XXXX", "....X", "XXXX."]),
    "e": (2, [".XXX.", "X...X", "XXXXX", "X....", ".XXX."]),
    "o": (2, [".XXX.", "X...X", "X...X", "X...X", ".XXX."]),
    "d": (0, ["....X", "....X", ".XXXX", "X...X", "X...X", "X...X", ".XXXX"]),
    "r": (2, ["X.XXX", "XX...", "X....", "X....", "X...."]),
}
# Farben: (Fläche, Lichtkante, Schatten)
LIGHT = ("#b8b0f0", "#d3cdf8", "#7a70c8")      # Blöcke auf dunklem Grund (wie das Icon)
DARK = ("#4b3fb0", "#6b5fd0", "#2a206e")       # Blöcke auf hellem Grund
DOT = ("#ffd85a", "#ffe98f", "#d9a521")
BG_TOP, BG_BOT = "#4b3fb0", "#2a206e"


def layout(word):
    """Blöcke eines Wortes: ([(spalte, zeile)], Spalte des Punktes) – der Punkt steht auf der Grundlinie (Zeile 6)"""
    cells, x = [], 0
    for ch in word:
        y0, rows = G[ch]
        for r, row in enumerate(rows):
            cells += [(x + i, y0 + r) for i, v in enumerate(row) if v == "X"]
        x += max(len(r) for r in rows) + 1
    return cells, x


def rgb(h): return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5))


class Canvas:
    """Sammelt Rechtecke; erzeugt daraus SVG und PNG (gleiches Ergebnis)"""
    def __init__(s, w, h): s.w, s.h, s.rects, s.tile = w, h, [], None

    def rect(s, x, y, w, h, col): s.rects.append((x, y, w, h, col))

    def block(s, cx, cy, c, pal, ox, oy, icon=False):
        # Wortmarke: Fuge 1/20 der Kante; Icon: ohne Fuge, Licht 3 px, Schatten 6 px (wie tools/gen_icon.py)
        X, Y = ox + cx * c, oy + cy * c
        z, hb, lb = (c, 3, 6) if icon else (c - c // 20, max(2, c // 10), max(3, c // 7))
        m, hi, lo = pal
        s.rect(X, Y, z, z, m); s.rect(X + z - lb, Y, lb, z, lo); s.rect(X, Y + z - lb, z, lb, lo)
        s.rect(X, Y, z, hb, hi); s.rect(X, Y, hb, z, hi)

    def svg(s):
        o = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {s.w} {s.h}" width="{s.w}" height="{s.h}">']
        if s.tile:
            m, r = s.tile
            o.append(f'<defs><linearGradient id="g" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{BG_TOP}"/><stop offset="1" stop-color="{BG_BOT}"/></linearGradient></defs>')
            o.append(f'<rect x="{m}" y="{m}" width="{s.w - 2 * m}" height="{s.h - 2 * m}" rx="{r}" fill="url(#g)"/>')
        o += [f'<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{c}"/>' for x, y, w, h, c in s.rects]
        return "\n".join(o + ["</svg>\n"])

    def png(s):
        w, h = s.w, s.h; buf = bytearray(w * h * 4)
        if s.tile:
            m, r = s.tile; a, b = rgb(BG_TOP), rgb(BG_BOT)
            for y in range(m, h - m):
                col = tuple(int(a[i] + (b[i] - a[i]) * y / h) for i in range(3))
                for x in range(m, w - m):
                    cov = 1.0
                    if (x < m + r or x >= w - m - r) and (y < m + r or y >= h - m - r):   # Ecke: 2x2-Überabtastung
                        cx = m + r if x < m + r else w - m - r; cy = m + r if y < m + r else h - m - r
                        cov = sum((x + .25 + .5 * i - cx) ** 2 + (y + .25 + .5 * j - cy) ** 2 <= r * r for i in (0, 1) for j in (0, 1)) / 4
                    if cov: o = (y * w + x) * 4; buf[o:o + 4] = bytes((*col, int(cov * 255)))
        for x, y, rw, rh, c in s.rects:
            px = bytes((*rgb(c), 255))
            for yy in range(y, y + rh): o = (yy * w + x) * 4; buf[o:o + rw * 4] = px * rw
        raw = b"".join(b"\0" + bytes(buf[y * w * 4:(y + 1) * w * 4]) for y in range(h))
        ch = lambda t, d: struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d))
        return b"\x89PNG\r\n\x1a\n" + ch(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)) + ch(b"IDAT", zlib.compress(raw, 9)) + ch(b"IEND", b"")


def make(word, c, pad_x, pad_y, rows, pal, tile=None):
    cells, dx = layout(word)
    cols = dx + 1
    cv = Canvas(cols * c + 2 * pad_x, rows * c + 2 * pad_y)
    if tile: cv.tile = tile
    for cx, cy in cells: cv.block(cx, cy, c, pal, pad_x, pad_y)
    cv.block(dx, 6, c, DOT, pad_x, pad_y)
    return cv


def icon():
    """Wie tools/gen_icon.py: 1024x1024, Kachel mit Rand 40 und Eckenradius 230, Blöcke 50 px"""
    out = Canvas(1024, 1024); out.tile = (40, 230)
    w, h = 13 * 50, 7 * 50; x0, y0 = (1024 - w) // 2, (1024 - h) // 2 - 10
    cells, _ = layout("VC")
    for cx, cy in cells: out.block(cx, cy, 50, LIGHT, x0, y0, True)
    out.block(12, 6, 50, DOT, x0, y0, True)
    return out


root = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "branding")
os.makedirs(root, exist_ok=True)
files = {
    "vintagecoder-icon": icon(),
    "vintagecoder-wordmark": make("VintageCoder", 20, 90, 110, 9, LIGHT, tile=(0, 90)),
    "vintagecoder-wordmark-light": make("VintageCoder", 20, 0, 0, 9, LIGHT),
    "vintagecoder-wordmark-dark": make("VintageCoder", 20, 0, 0, 9, DARK),
    "vintagecoder-mark-light": make("VC", 40, 0, 0, 7, LIGHT),
    "vintagecoder-mark-dark": make("VC", 40, 0, 0, 7, DARK),
}
for name, cv in files.items():
    open(os.path.join(root, name + ".svg"), "w").write(cv.svg())
    open(os.path.join(root, name + ".png"), "wb").write(cv.png())
print("Logo erzeugt:", ", ".join(sorted(files)))
