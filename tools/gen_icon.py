#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt alle App-Icons aus den Vorlagen in branding/ (nur Standardbibliothek).

Vorlagen (1024 × 1024, RGBA, siehe tools/gen_logo.py und branding/README.md):
  branding/vintagecoder-icon.png          „VC.“ auf dunkler, abgerundeter Kachel
  branding/vintagecoder-icon-klein.png    dieselbe Zeichnung mit größerer Schrift, für 16 und 32 Pixel (bessere Lesbarkeit)

Ausgabe (Pfade relativ zur Repository-Wurzel, vorhandene Dateien werden überschrieben):
  installer/windows/VintageCoder.ico                     16, 32, 48, 64, 128, 256
  installer/macos/VintageCoder-1024.png, VintageCoder.icns   (.icns nur auf macOS, über das Systemprogramm iconutil)
  installer/linux/icons/hicolor/<n>x<n>/apps/vintagecoder.png   16, 32, 48, 64, 128, 256, 512
  crates/vc-shell/icons/                                 alle Größen, die tauri.conf.json und der Windows-Store brauchen (32x32.png … icon.ico, icon.icns)

Aufruf (Arbeitsordner egal): python3 tools/gen_icon.py"""
import argparse, os, struct, subprocess, sys, tempfile, zlib

argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter).parse_args()
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
KLEIN_BIS = 32   # bis zu dieser Größe (einschließlich) kommt die Vorlage mit der größeren Schrift zum Einsatz


def png_lesen(pfad):
    """RGBA-PNG (8 Bit, ohne Verschachtelung) lesen: (Breite, Höhe, Zeilen als bytearray)"""
    b = open(pfad, "rb").read(); w, h, tiefe, typ, _, _, versch = struct.unpack(">IIBBBBB", b[16:29])
    if (tiefe, typ, versch) != (8, 6, 0): raise SystemExit(f"{pfad}: erwartet wird RGBA, 8 Bit, ohne Verschachtelung")
    i, daten = 8, b""
    while i < len(b):
        n, t = struct.unpack(">I4s", b[i:i + 8]); daten += b[i + 8:i + 8 + n] if t == b"IDAT" else b""; i += 12 + n
    raw = zlib.decompress(daten); stride = w * 4; zeilen = []; vor = bytearray(stride)
    for y in range(h):
        f = raw[y * (stride + 1)]; z = bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for x in range(stride):
            a = z[x - 4] if x >= 4 else 0; c = vor[x - 4] if x >= 4 else 0; u = vor[x]
            if f == 1: z[x] = (z[x] + a) & 255
            elif f == 2: z[x] = (z[x] + u) & 255
            elif f == 3: z[x] = (z[x] + (a + u) // 2) & 255
            elif f == 4:
                p = a + u - c; pa, pb, pc = abs(p - a), abs(p - u), abs(p - c)
                z[x] = (z[x] + (a if pa <= pb and pa <= pc else u if pb <= pc else c)) & 255
        zeilen.append(z); vor = z
    return w, h, zeilen


def verkleinern(w, h, zeilen, ziel):
    """Flächenmittel (Boxfilter mit Bruchteilen) auf ziel × ziel, mit vormultipliziertem Alpha"""
    def achse(n_ein, n_aus):
        s = n_ein / n_aus; out = []
        for k in range(n_aus):
            a, b = k * s, (k + 1) * s; i0, i1 = int(a), min(int(b - 1e-9), n_ein - 1)
            out.append([(i, (min(b, i + 1) - max(a, i)) / s) for i in range(i0, i1 + 1)])
        return out
    ax, ay = achse(w, ziel), achse(h, ziel)
    zw = []   # erst waagerecht (je Quellzeile auf `ziel` Spalten), dann senkrecht
    for z in zeilen:
        row = []
        for g in ax:
            a = r = gr = bl = 0.0
            for i, wt in g:
                al = z[i * 4 + 3] * wt; a += al; r += z[i * 4] * al; gr += z[i * 4 + 1] * al; bl += z[i * 4 + 2] * al
            row.append((a, r, gr, bl))
        zw.append(row)
    out = bytearray(ziel * ziel * 4)
    for y, g in enumerate(ay):
        for x in range(ziel):
            a = r = gr = bl = 0.0
            for j, wt in g:
                pa, pr, pg, pb = zw[j][x]; a += pa * wt; r += pr * wt; gr += pg * wt; bl += pb * wt
            o = (y * ziel + x) * 4
            if a > 0: out[o:o + 4] = bytes((min(255, round(r / a)), min(255, round(gr / a)), min(255, round(bl / a)), min(255, round(a))))
    return out


def png_schreiben(buf, n):
    raw = b"".join(b"\0" + bytes(buf[y * n * 4:(y + 1) * n * 4]) for y in range(n))
    def ch(t, d): return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d))
    return b"\x89PNG\r\n\x1a\n" + ch(b"IHDR", struct.pack(">IIBBBBB", n, n, 8, 6, 0, 0, 0)) + ch(b"IDAT", zlib.compress(raw, 9)) + ch(b"IEND", b"")


gross = png_lesen(os.path.join(ROOT, "branding/vintagecoder-icon.png"))
klein = png_lesen(os.path.join(ROOT, "branding/vintagecoder-icon-klein.png"))
_zwischen = {}
def icon(n):
    """PNG-Bytes des Icons in der Größe n × n"""
    if n not in _zwischen:
        quelle = klein if n <= KLEIN_BIS else gross
        _zwischen[n] = png_schreiben(b"".join(bytes(z) for z in quelle[2]), 1024) if n == 1024 else png_schreiben(verkleinern(*quelle, n), n)
    return _zwischen[n]


def schreiben(rel, daten):
    p = os.path.join(ROOT, rel); os.makedirs(os.path.dirname(p), exist_ok=True); open(p, "wb").write(daten)


def ico(groessen):
    """.ico mit eingebetteten PNGs"""
    kopf = struct.pack("<HHH", 0, 1, len(groessen)); off = 6 + 16 * len(groessen); eintraege = b""; daten = b""
    for s in groessen:
        p = icon(s); eintraege += struct.pack("<BBBBHHII", s % 256, s % 256, 0, 0, 1, 32, len(p), off + len(daten)); daten += p
    return kopf + eintraege + daten


icns = None
if sys.platform == "darwin":   # .icns über iconutil
    with tempfile.TemporaryDirectory() as t:
        d = os.path.join(t, "VintageCoder.iconset"); os.mkdir(d)
        for s, name in ((16, "16x16"), (32, "16x16@2x"), (32, "32x32"), (64, "32x32@2x"), (128, "128x128"), (256, "128x128@2x"),
                        (256, "256x256"), (512, "256x256@2x"), (512, "512x512"), (1024, "512x512@2x")):
            open(os.path.join(d, f"icon_{name}.png"), "wb").write(icon(s))
        subprocess.check_call(["iconutil", "-c", "icns", d, "-o", os.path.join(t, "icon.icns")]); icns = open(os.path.join(t, "icon.icns"), "rb").read()

# Windows und macOS (Installer)
schreiben("installer/windows/VintageCoder.ico", ico((16, 32, 48, 64, 128, 256)))
schreiben("installer/macos/VintageCoder-1024.png", icon(1024))
if icns: schreiben("installer/macos/VintageCoder.icns", icns)
# Linux: hicolor
for s in (16, 32, 48, 64, 128, 256, 512): schreiben(f"installer/linux/icons/hicolor/{s}x{s}/apps/vintagecoder.png", icon(s))
# Anwendung (tauri.conf.json: bundle.icon; Windows-Store-Kacheln)
for name, s in (("32x32.png", 32), ("64x64.png", 64), ("128x128.png", 128), ("128x128@2x.png", 256), ("icon.png", 512), ("StoreLogo.png", 50)): schreiben("crates/vc-shell/icons/" + name, icon(s))
for s in (30, 44, 71, 89, 107, 142, 150, 284, 310): schreiben(f"crates/vc-shell/icons/Square{s}x{s}Logo.png", icon(s))
schreiben("crates/vc-shell/icons/icon.ico", ico((16, 24, 32, 48, 64, 256)))
if icns: schreiben("crates/vc-shell/icons/icon.icns", icns)
print("Icons erzeugt" + ("" if icns else " (ohne .icns: iconutil gibt es nur auf macOS)"))
