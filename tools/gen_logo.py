#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt das offizielle VintageCoder-Logo (Wortmarke „VintageCoder.“, Kurzform „VC.“, App-Icon) als SVG nach branding/.
Die Buchstaben stammen aus JetBrains Mono Bold (SIL Open Font License 1.1, ui/src/assets/fonts/) und stehen in den SVG-Dateien als Pfade;
es wird keine Schrift eingebettet oder mitgeliefert. Nur Standardbibliothek.

Farben: Violett #a99bff, Weiß #f1f1f4, Gelb #ffc83d (der Punkt gehört zum Logo), Hintergrund #0d1117 (Verlauf im Icon #171c26 bis #0d1117).
Zusätzlich zum Icon gibt es eine Fassung mit größerer Schrift für 16 und 32 Pixel (`vintagecoder-icon-klein.svg`).

Aufruf (Arbeitsordner egal): python3 tools/gen_logo.py
Die PNG-Fassungen (Icon, Icon klein, Wortmarke) entstehen daraus mit `node tools/svg2png.mjs` im Ordner ui, siehe branding/README.md;
alle Größen der App-Icons erzeugt anschließend python3 tools/gen_icon.py."""
import argparse, os, struct

argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter).parse_args()
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FONT = os.path.join(ROOT, "ui/src/assets/fonts/JetBrainsMono-Bold.ttf")
data = open(FONT, "rb").read()
def tables():
    n = struct.unpack(">H", data[4:6])[0]; t = {}
    for i in range(n):
        tag, _, off, ln = struct.unpack(">4sIII", data[12+16*i:28+16*i]); t[tag.decode()] = (off, ln)
    return t
T = tables()
head = T["head"][0]; UPM = struct.unpack(">H", data[head+18:head+20])[0]; LOCFMT = struct.unpack(">h", data[head+50:head+52])[0]
maxp = T["maxp"][0]; NG = struct.unpack(">H", data[maxp+4:maxp+6])[0]
hhea = T["hhea"][0]; NHM = struct.unpack(">H", data[hhea+34:hhea+36])[0]
hmtx = T["hmtx"][0]
loca = T["loca"][0]
def glyph_off(g):
    if LOCFMT == 0:
        a, b = struct.unpack(">HH", data[loca+2*g:loca+2*g+4]); return a*2, b*2
    a, b = struct.unpack(">II", data[loca+4*g:loca+4*g+8]); return a, b
def cmap():
    o = T["cmap"][0]; n = struct.unpack(">H", data[o+2:o+4])[0]; best = None
    for i in range(n):
        pid, eid, off = struct.unpack(">HHI", data[o+4+8*i:o+12+8*i])
        if (pid, eid) in ((3, 1), (0, 3), (0, 4), (3, 10)):
            fmt = struct.unpack(">H", data[o+off:o+off+2])[0]
            if fmt == 4: best = o+off
    s = best; segx2 = struct.unpack(">H", data[s+6:s+8])[0]; sc = segx2//2
    ends = struct.unpack(f">{sc}H", data[s+14:s+14+segx2]); starts = struct.unpack(f">{sc}H", data[s+16+segx2:s+16+2*segx2])
    deltas = struct.unpack(f">{sc}h", data[s+16+2*segx2:s+16+3*segx2]); ro_off = s+16+3*segx2
    ros = struct.unpack(f">{sc}H", data[ro_off:ro_off+segx2])
    def look(c):
        for i in range(sc):
            if starts[i] <= c <= ends[i]:
                if ros[i] == 0: return (c+deltas[i]) & 0xFFFF
                p = ro_off+2*i+ros[i]+2*(c-starts[i]); g = struct.unpack(">H", data[p:p+2])[0]
                return (g+deltas[i]) & 0xFFFF if g else 0
        return 0
    return look
look = cmap()
def advance(g):
    i = min(g, NHM-1); return struct.unpack(">H", data[hmtx+4*i:hmtx+4*i+2])[0]
def outline(g):
    a, b = glyph_off(g); gl = T["glyf"][0]
    if a == b: return []
    p = gl+a; nc = struct.unpack(">h", data[p:p+2])[0]
    if nc < 0:
        p += 10; res = []
        while True:
            fl, gi = struct.unpack(">HH", data[p:p+4]); p += 4
            if fl & 1: dx, dy = struct.unpack(">hh", data[p:p+4]); p += 4
            else: dx, dy = struct.unpack(">bb", data[p:p+2]); p += 2
            if not fl & 2: raise SystemExit("Punktanpassung nicht unterstuetzt")
            if fl & 8: p += 2
            elif fl & 0x40: p += 4
            elif fl & 0x80: p += 8
            for c in outline(gi): res.append([(x+dx, y+dy, on) for x, y, on in c])
            if not fl & 0x20: break
        return res
    p += 10; ends = list(struct.unpack(f">{nc}H", data[p:p+2*nc])); p += 2*nc
    il = struct.unpack(">H", data[p:p+2])[0]; p += 2+il; npts = ends[-1]+1
    flags = []
    while len(flags) < npts:
        f = data[p]; p += 1; flags.append(f)
        if f & 8:
            r = data[p]; p += 1; flags.extend([f]*r)
    xs, ys, v = [], [], 0
    for f in flags:
        if f & 2: d = data[p]; p += 1; v += d if f & 16 else -d
        elif not f & 16: v += struct.unpack(">h", data[p:p+2])[0]; p += 2
        xs.append(v)
    v = 0
    for f in flags:
        if f & 4: d = data[p]; p += 1; v += d if f & 32 else -d
        elif not f & 32: v += struct.unpack(">h", data[p:p+2])[0]; p += 2
        ys.append(v)
    out, s = [], 0
    for e in ends:
        out.append([(xs[i], ys[i], bool(flags[i] & 1)) for i in range(s, e+1)]); s = e+1
    return out
def path_d(g, ox, oy, sc):
    d = []
    for c in outline(g):
        n = len(c)
        # Startpunkt auf Kurve
        pts = c[:]
        if not pts[0][2]:
            if pts[-1][2]: pts = [pts[-1]] + pts[:-1]
            else: m = ((pts[0][0]+pts[-1][0])/2, (pts[0][1]+pts[-1][1])/2, True); pts = [m] + pts
        X = lambda x: round(ox + x*sc, 2); Y = lambda y: round(oy - y*sc, 2)
        d.append(f"M{X(pts[0][0])} {Y(pts[0][1])}")
        i = 1; N = len(pts)
        while i <= N:
            cur = pts[i % N]
            if cur[2]:
                d.append(f"L{X(cur[0])} {Y(cur[1])}"); i += 1
            else:
                nx = pts[(i+1) % N]
                if nx[2]: ex, ey = nx[0], nx[1]; i += 2
                else: ex, ey = (cur[0]+nx[0])/2, (cur[1]+nx[1])/2; i += 1
                d.append(f"Q{X(cur[0])} {Y(cur[1])} {X(ex)} {Y(ey)}")
        d.append("Z")
    return "".join(d)
def ink(parts, size):
    """Tintenbreite (x0..x1) und -hoehe (obere/untere Kante relativ zur Grundlinie) ohne Seitenabstand"""
    sc = size/UPM; x = 0; lo, hi, top, bot = 1e9, -1e9, -1e9, 1e9
    for text, _ in parts:
        for ch in text:
            g = look(ord(ch))
            for c in outline(g):
                for px, py, _o in c:
                    lo = min(lo, x+px*sc); hi = max(hi, x+px*sc); top = max(top, py*sc); bot = min(bot, py*sc)
            x += advance(g)*sc
    return lo, hi, top, bot
def run(parts, size, x0, y0):
    """parts: [(text, farbe)] -> SVG-Pfade, Breite"""
    sc = size/UPM; x = x0; out = []
    for text, col in parts:
        d = ""
        for ch in text:
            g = look(ord(ch)); d += path_d(g, x, y0, sc); x += advance(g)*sc
        out.append(f'<path fill="{col}" d="{d}"/>')
    return "".join(out), x-x0
BG, VIO, WEISS, GELB, ORANGE = "#0d1117", "#a99bff", "#f1f1f4", "#ffc83d", "#ff8a3d"
def wortmarke(parts, size, pad, bg=None):
    lo, hi, top, bot = ink(parts, size)
    W = hi-lo+2*pad; H = top-bot+2*pad
    body, _ = run(parts, size, pad-lo, pad+top)
    rect = f'<rect width="{W:.0f}" height="{H:.0f}" fill="{bg}"/>' if bg else ""
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W:.0f} {H:.0f}" width="{W:.0f}" height="{H:.0f}">{rect}{body}</svg>'
def icon(parts, size):
    lo, hi, top, bot = ink(parts, size)
    x0 = 512-(lo+hi)/2; y0 = 512+(top+bot)/2
    body, _ = run(parts, size, x0, y0)
    return ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">'
            f'<defs><linearGradient id="g" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#171c26"/><stop offset="1" stop-color="{BG}"/></linearGradient></defs>'
            f'<rect x="40" y="40" width="944" height="944" rx="230" fill="url(#g)"/>{body}</svg>')
vc = lambda: [("V", VIO), ("C", WEISS), (".", GELB)]
files = {
 "vintagecoder-wortmarke.svg": wortmarke([("Vintage", VIO), ("Coder", WEISS), (".", GELB)], 160, 90, BG),
 "vintagecoder-wortmarke-transparent.svg": wortmarke([("Vintage", VIO), ("Coder", WEISS), (".", GELB)], 160, 90, None),
 "vintagecoder-wortmarke-hell.svg": wortmarke([("Vintage", "#6b57e6"), ("Coder", "#1b1f27"), (".", "#f0a800")], 160, 90, "#ffffff"),
 "vintagecoder-vc.svg": wortmarke(vc(), 400, 120, BG),
 "vintagecoder-icon.svg": icon(vc(), 400),
 "vintagecoder-icon-klein.svg": icon(vc(), 500),
}
od = os.path.join(ROOT, "branding"); os.makedirs(od, exist_ok=True)
for n, s in files.items(): open(os.path.join(od, n), "w", encoding="utf-8").write(s)
# Svelte-Baustein der Oberfläche: dieselben Pfade, Farben über CSS-Variablen (hell/dunkel/Kontrast), damit das Logo in allen Erscheinungsbildern lesbar bleibt
import re
pfade = re.findall(r'<path fill="#[0-9a-f]+" d="([^"]*)"', files["vintagecoder-wortmarke-transparent.svg"]); vb = re.search(r'viewBox="0 0 (\d+) (\d+)"', files["vintagecoder-wortmarke-transparent.svg"]); vb = f"90 90 {int(vb.group(1)) - 180} {int(vb.group(2)) - 180}"   # ohne den Rand der SVG-Datei (90)
baustein = f'''<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<!-- Erzeugt von tools/gen_logo.py (nicht von Hand ändern): Wortmarke „VintageCoder.“ als Pfade (JetBrains Mono Bold, OFL), Farben über CSS-Variablen -->
<script lang="ts">
  let {{ hoehe = 48, titel = "VintageCoder" }}: {{ hoehe?: number; titel?: string }} = $props();
</script>

<svg class="wortmarke" viewBox="{vb}" height={{hoehe}} role="img" aria-label={{titel}} xmlns="http://www.w3.org/2000/svg">
  <path fill="var(--wm-a)" d="{pfade[0]}" />
  <path fill="var(--wm-b)" d="{pfade[1]}" />
  <path fill="var(--wm-c)" d="{pfade[2]}" />
</svg>

<style>
  .wortmarke {{ display: block; width: auto; max-width: 100%; --wm-a: #a99bff; --wm-b: #f1f1f4; --wm-c: #ffc83d; }}
  :global(:root[data-theme="light"]) .wortmarke {{ --wm-a: #6b57e6; --wm-b: #1b1f27; --wm-c: #f0a800; }}
  @media (prefers-color-scheme: light) {{ :global(:root:not([data-theme])) .wortmarke {{ --wm-a: #6b57e6; --wm-b: #1b1f27; --wm-c: #f0a800; }} }}
</style>
'''
open(os.path.join(ROOT, "ui/src/lib/components/Wortmarke.svelte"), "w", encoding="utf-8").write(baustein)
print("Logo erzeugt:", ", ".join(files))
