#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt das App-Icon: installer/macos/VintageCoder.icns (macOS) und installer/windows/VintageCoder.ico (Windows).
Reine Standardbibliothek. Aufruf im Projektordner: python3 tools/gen_icon.py  (iconutil nur auf macOS für die .icns)"""
import argparse, os, struct, subprocess, sys, tempfile, zlib
_ap = argparse.ArgumentParser(
    description="""Erzeugt das App-Icon (nur Standardbibliothek): installer/windows/VintageCoder.ico (Windows), installer/macos/VintageCoder-1024.png und - nur auf macOS,
über das Systemprogramm iconutil - installer/macos/VintageCoder.icns.

Die Pfade werden relativ zum Ort des Skripts bestimmt (Repository-Wurzel), der Arbeitsordner ist egal; fehlende Ordner unter installer/ werden angelegt, vorhandene
Dateien überschrieben. Binärdateien: keine Kodierungsfragen.""",
    epilog="""Beispiele:
  python3 tools/gen_icon.py
  cd tools && python3 gen_icon.py

Das Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.""",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_ap.parse_args()

N = 1024
BG_TOP, BG_BOT = (0x4B, 0x3F, 0xB0), (0x2A, 0x20, 0x6E)   # C64-Blau, dunkler Verlauf
FG, SHADE, CUR = (0xB8, 0xB0, 0xF0), (0x7A, 0x70, 0xC8), (0xFF, 0xD8, 0x5A)
V = ["X...X", "X...X", "X...X", "X...X", ".X.X.", ".X.X.", "..X.."]
C = [".XXX.", "X...X", "X....", "X....", "X....", "X...X", ".XXX."]

def cells():
    out = []   # (spalte, zeile, farbe)
    for ox, g in ((0, V), (6, C)):
        for y, row in enumerate(g):
            for x, ch in enumerate(row):
                if ch == "X": out.append((ox + x, y, FG))
    out.append((12, 6, CUR))
    return out

def render():
    cell = 50; w = 13 * cell; h = 7 * cell
    x0 = (N - w) // 2; y0 = (N - h) // 2 - 10
    px = [[None] * N for _ in range(N)]
    r = N * 0.225; m = 40   # Außenrand, Eckenradius
    def inside(fx, fy):
        cx = min(max(fx, m + r), N - m - r); cy = min(max(fy, m + r), N - m - r)
        return (fx - cx) ** 2 + (fy - cy) ** 2 <= r * r and m <= fx <= N - m and m <= fy <= N - m
    blocks = {}
    for cx, cy, col in cells():
        for dy in range(cell):
            for dx in range(cell):
                blocks[(x0 + cx * cell + dx, y0 + cy * cell + dy)] = (col, dx, dy)
    buf = bytearray(N * N * 4)
    for y in range(N):
        t = y / N
        bg = tuple(int(BG_TOP[i] + (BG_BOT[i] - BG_TOP[i]) * t) for i in range(3))
        for x in range(N):
            cov = sum(inside(x + .25 + .5 * i, y + .25 + .5 * j) for i in (0, 1) for j in (0, 1)) / 4
            if cov == 0: continue
            col = bg
            b = blocks.get((x, y))
            if b:
                col, dx, dy = b
                if dx >= cell - 6 or dy >= cell - 6: col = SHADE if col == FG else (0xC8, 0x9A, 0x20)
                if dx < 3 or dy < 3: col = tuple(min(255, c + 25) for c in col)
            o = (y * N + x) * 4
            buf[o:o + 4] = bytes((*col, int(cov * 255)))
    return buf

def scale(buf, size):
    f = N // size; out = bytearray(size * size * 4)
    for y in range(size):
        for x in range(size):
            a = r = g = b = 0
            for j in range(f):
                base = ((y * f + j) * N + x * f) * 4
                for i in range(f):
                    o = base + i * 4; al = buf[o + 3]
                    a += al; r += buf[o] * al; g += buf[o + 1] * al; b += buf[o + 2] * al
            o = (y * size + x) * 4
            if a: out[o:o + 4] = bytes((r // a, g // a, b // a, a // (f * f)))
    return out

def png(buf, size):
    raw = b"".join(b"\0" + bytes(buf[y * size * 4:(y + 1) * size * 4]) for y in range(size))
    def ch(t, d): return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d))
    return b"\x89PNG\r\n\x1a\n" + ch(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)) + ch(b"IDAT", zlib.compress(raw, 9)) + ch(b"IEND", b"")

root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
big = render()
pngs = {s: png(big if s == N else scale(big, s), s) for s in (16, 32, 64, 128, 256, 512, 1024)}

# Windows: .ico mit eingebetteten PNGs (16, 32, 64, 128, 256)
os.makedirs(os.path.join(root, "installer/windows"), exist_ok=True)
sizes = (16, 32, 64, 128, 256)
head = struct.pack("<HHH", 0, 1, len(sizes)); off = 6 + 16 * len(sizes); ents = b""; data = b""
for s in sizes:
    p = pngs[s]; ents += struct.pack("<BBBBHHII", s % 256, s % 256, 0, 0, 1, 32, len(p), off + len(data)); data += p
open(os.path.join(root, "installer/windows/VintageCoder.ico"), "wb").write(head + ents + data)
open(os.path.join(root, "installer/macos/VintageCoder-1024.png"), "wb").write(pngs[1024])

# macOS: .icns über iconutil
if sys.platform == "darwin":
    with tempfile.TemporaryDirectory() as t:
        d = os.path.join(t, "VintageCoder.iconset"); os.mkdir(d)
        for s, name in ((16, "16x16"), (32, "16x16@2x"), (32, "32x32"), (64, "32x32@2x"), (128, "128x128"), (256, "128x128@2x"),
                        (256, "256x256"), (512, "256x256@2x"), (512, "512x512"), (1024, "512x512@2x")):
            open(os.path.join(d, f"icon_{name}.png"), "wb").write(pngs[s])
        subprocess.check_call(["iconutil", "-c", "icns", d, "-o", os.path.join(root, "installer/macos/VintageCoder.icns")])
print("Icon erzeugt")
