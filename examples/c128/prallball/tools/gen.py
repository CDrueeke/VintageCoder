#!/usr/bin/env python3
"""Erzeugt die Datendateien von PRALLBALL (C128, VIC-II-Fassung) im Ordner lib/:
  lib/dirtable.inc      Richtungstabelle: 16 Geschwindigkeitsstufen x 8 Abprallzonen (dir_xl, dir_xh, dir_yl, dir_yh)
  lib/levels.inc        Levelkarten aus levels/levels.txt (je Level 12 x 18 Steintypen), Levelart und Adresstabellen
  lib/gfx.inc.chars     eigene Mehrfarb-Zeichen 64..67 (Stein links/rechts, Wand, Block)
  lib/gfx.inc.sprites   Sprite-Bloecke (Ball, Schlaeger, Kapsel mit Drehphasen, Schuss); in prallball.asm ab $8000 (Zeiger $00..$0A)
Die Ausgaben sind identisch zu denen der C64-Fassung; nur die Kommentare zu den Sprite-Zeigern gelten fuer die C128-Speicherbelegung.
Aufruf im Ordner examples/c128/prallball:  python3 tools/gen.py   (schreibt die Dateien neu; unter Windows mit CRLF-Zeilenenden)"""
import math, re, os
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/c128/prallball (os.chdir); alle Pfade gelten relativ dazu, der Aufruf funktioniert daher aus jedem Ordner.', epilog='Beispiele:\n  python3 examples/c128/prallball/tools/gen.py\n  cd examples/c128/prallball && python3 tools/gen.py\n\nSchreibt (im Projektordner examples/c128/prallball, vorhandene Dateien werden ueberschrieben): lib/dirtable.inc, lib/levels.inc, lib/gfx.inc.chars, lib/gfx.inc.sprites.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))

# ---- Richtungstabelle: 16 Geschwindigkeitsstufen x 8 Zonen; Einheit 1/64 Bildpunkt je Bild (PAL 50 Hz)
# Zone 0..7 = Abprallwinkel in Grad zur Senkrechten (negativ = nach links); Betrag der Geschwindigkeit = (1,6 + 0,1 * Stufe) Bildpunkte je Bild
ANG = [-60, -43, -26, -9, 9, 26, 43, 60]
def s16(v): return v & 0xffff
dxl, dxh, dyl, dyh = [], [], [], []
for lvl in range(16):
    speed = (1.6 + 0.1 * lvl) * 64
    for a in ANG:
        r = math.radians(a)
        dx = int(round(speed * math.sin(r))); dy = int(round(-speed * math.cos(r)))
        dxl.append(s16(dx) & 255); dxh.append(s16(dx) >> 8); dyl.append(s16(dy) & 255); dyh.append(s16(dy) >> 8)
def tab(name, vals):
    out = [name + ':']
    for i in range(0, len(vals), 16): out.append('        .byte ' + ', '.join('$%02x' % v for v in vals[i:i + 16]))
    return out
L = ['; ERZEUGT von tools/gen.py: Richtungstabelle (Zone 0..7 = -60, -43, -26, -9, +9, +26, +43, +60 Grad), Einheit 1/64 Punkt je Bild',
     '; Index = Stufe * 8 + Zone (Stufe 0..15, 128 Eintraege je Tabelle; set_vel in prallball.asm).  Betrag = (1,6 + 0,1 * Stufe) * 64;',
     '; dx = Betrag * sin(Winkel), dy = -Betrag * cos(Winkel) (negativ = nach oben).  _xl/_yl = Low-Byte, _xh/_yh = High-Byte (Zweierkomplement).']
for n, v in (('dir_xl', dxl), ('dir_xh', dxh), ('dir_yl', dyl), ('dir_yh', dyh)): L += tab(n, v)
open('lib/dirtable.inc', 'w').write('\n'.join(L) + '\n')

# ---- Levels
TYPE = {'.': 0, 'a': 1, 'b': 2, 'c': 3, 'd': 4, 'e': 5, 'f': 6, 'g': 7, 'h': 8, 's': 9, 'G': 10, '#': 11, 'x': 12, 'p': 13, 'C': 23}
# Im Text steht 'g' für Gold im Sinn der Beschreibung oben; hier gilt: a-h Farben, s Silber, g = Gold NUR wenn der Entwurf "Gold" will.
# Um Doppeldeutigkeit zu vermeiden, bedeutet 'g' im Level-Text die siebte Farbe; Gold schreibt man 'G'.
levels = []; cur = None
for line in open('levels/levels.txt', encoding='utf-8'):
    line = line.rstrip('\n')
    if line.startswith('#') and not line.startswith('#a') and ' ' in line[:2] + ' ' and re.match(r'#\s', line): continue
    m = re.match(r'level (\d+) "(.*)"', line)
    if m: cur = {'n': int(m.group(1)), 'name': m.group(2), 'rows': []}; levels.append(cur); continue
    if cur is not None and len(line) == 18: cur['rows'].append(line)
M = ['; ERZEUGT von tools/gen.py aus levels/levels.txt: je Level 216 Byte (12 Zeilen x 18 Spalten), Steintyp 0..13 und 23 (Boss-Kern)', 'level_count = %d' % len(levels)]
for lv in levels:
    assert len(lv['rows']) == 12, lv['n']
    M.append('level%d:   ; %s' % (lv['n'], lv['name']))
    for r in lv['rows']: M.append('        .byte ' + ', '.join(str(TYPE[c]) for c in r))
M.append('level_kind:    .byte ' + ', '.join('2' if l['name'].startswith('Boss') else '1' if l['name'].startswith('Bonus') else '0' for l in levels) + '   ; 0 normal, 1 Bonus (Zeit, kein Ballverlust), 2 Boss (Kerne, Angriffe)')
M.append('level_tab_lo:  .byte ' + ', '.join('<level%d' % l['n'] for l in levels))
M.append('level_tab_hi:  .byte ' + ', '.join('>level%d' % l['n'] for l in levels))
open('lib/levels.inc', 'w').write('\n'.join(M) + '\n')

# ---- Grafik (Multicolor-Textmodus: je Zeile 4 Pixel zu 2 Bit; 0 = Hintergrund $D021, 1 = $D022 (hell), 2 = $D023 (dunkel), 3 = Farb-RAM)
def mcrow(px):
    v = 0
    for p in px: v = (v << 2) | p
    return v
def mcchar(rows): return [mcrow(r) for r in rows]
BRICK_L = mcchar([[0,1,1,1],[0,1,3,3],[0,1,3,3],[0,1,3,3],[0,1,3,3],[0,1,3,3],[0,2,2,2],[0,0,0,0]])
BRICK_R = mcchar([[1,1,1,0],[3,3,2,0],[3,3,2,0],[3,3,2,0],[3,3,2,0],[3,3,2,0],[2,2,2,0],[0,0,0,0]])
WALL    = mcchar([[1,1,1,1],[1,3,3,2],[1,3,3,2],[1,3,3,2],[1,3,3,2],[1,3,3,2],[1,3,3,2],[2,2,2,2]])
BLOCK   = WALL
def bytes_(rows): return ', '.join('$%02x' % r for r in rows)
G = ['; ERZEUGT von tools/gen.py: eigene Zeichen und Sprites']
G.append('gfx_chars:   ; Zeichen 64 (Stein links), 65 (Stein rechts), 66 (Wand), 67 (Block fuer das Titellogo); alle Multicolor')
for ch in (BRICK_L, BRICK_R, WALL, BLOCK): G.append('        .byte ' + bytes_(ch))
# Sprites (Multicolor): je Zeile 2 Byte = 8 Pixel zu 2 Bit; 1 = $D025 (hell), 2 = Spritefarbe, 3 = $D026 (dunkel)
def spr(rows, nbytes):
    out = []
    for r in rows:
        r = list(r) + [0] * (nbytes * 4 - len(r))
        for k in range(nbytes): out.append(mcrow(r[k * 4:k * 4 + 4]))
        out += [0] * (3 - nbytes)
    out += [0] * (64 - len(out))
    return out
ball = [[0,2,0],[2,1,2],[2,2,3],[2,2,3],[2,3,3],[0,3,0]]
G.append('gfx_ball:    ; Sprite-Zeiger $00 (C128: $8000; C64-Fassung: $E0), 64 Byte: Multicolor-Ball 6 x 6 px (3 x 6 Multicolor-Pixel)')
G.append('        .byte ' + bytes_(spr(ball, 1)))
def pad(left_round, right_round, width):
    rows = []
    for r in range(6):
        base = {0: 1, 1: 1, 2: 2, 3: 2, 4: 3, 5: 3}[r]
        row = [base] * width
        if r == 1: row[0] = 1 if left_round else base
        if left_round and r in (0, 5): row[0] = 0
        if right_round and r in (0, 5): row[-1] = 0
        rows.append(row)
    return spr(rows, width // 4)
G.append('gfx_padl:   ; Zeiger $01 ($8040): Schlaegerhaelfte links (16 px = 8 Multicolor-Pixel)')
G.append('        .byte ' + bytes_(pad(True, False, 8)))
G.append('gfx_padr:   ; Zeiger $02 ($8080): Schlaegerhaelfte rechts (16 px)')
G.append('        .byte ' + bytes_(pad(False, True, 8)))
G.append('gfx_pad24l: ; Zeiger $03 ($80C0): breiter Schlaeger links (24 px = 12 Multicolor-Pixel)')
G.append('        .byte ' + bytes_(pad(True, False, 12)))
G.append('gfx_pad24r: ; Zeiger $04 ($8100): breiter Schlaeger rechts (24 px)')
G.append('        .byte ' + bytes_(pad(False, True, 12)))
G.append('gfx_pad16:  ; Zeiger $05 ($8140): schmaler Schlaeger (16 px, beide Enden rund)')
G.append('        .byte ' + bytes_(pad(True, True, 8)))
import math
def coin(h):
    """Rotierende Muenze (Drehung um die WAAGERECHTE Achse): immer 6 Multicolor-Pixel (12 px) breit in den Spalten 3..8, die Hoehe h
    (12, 10, 7, 3) schrumpft beim Wenden; senkrecht mittig in 12 Zeilen.  Schattierung: oben hell (1), Mitte Spritefarbe (2), unten dunkel (3),
    Rand links hell / rechts dunkel."""
    rows = [[0] * 12 for _ in range(12)]
    top = (12 - h) // 2
    for r in range(h):
        dy = (r - (h - 1) / 2.0) / (h / 2.0)
        w = int(round(6 * math.sqrt(max(0.0, 1 - dy * dy))))
        w = max(2, w - (w % 2))
        if h <= 3: w = 6
        left = 3 + (6 - w) // 2
        for k in range(w):
            if r == 0 and h > 3: v = 1
            elif r == h - 1 and h > 3: v = 3
            elif h <= 3: v = (1, 2, 3)[r]
            elif k == 0: v = 1
            elif k == w - 1: v = 3
            else: v = 2
            rows[top + r][left + k] = v
    out = []
    for r in rows:
        for k in range(3): out.append(mcrow(r[k * 4:k * 4 + 4]))
    out += [0] * (64 - len(out))
    return out
G.append('gfx_cap:    ; Zeiger $06 ($8180): Kapsel vorn (12 x 12 px); der Buchstabe wird zur Laufzeit in Byte 1 der Zeilen 2..9 eingetragen')
G.append('        .byte ' + bytes_(coin(12)))
G.append('gfx_shot:   ; Zeiger $07 ($81C0): Laserschuss (2 x 8 Bildpunkte, einfarbig)')
G.append('        .byte ' + bytes_([0xc0, 0, 0] * 8 + [0] * 40))
G.append('gfx_cap1:   ; Zeiger $08 ($8200): Kapsel gekippt (10 Zeilen hoch)')
G.append('        .byte ' + bytes_(coin(10)))
G.append('gfx_cap2:   ; Zeiger $09 ($8240): Kapsel halb gekippt (7 Zeilen)')
G.append('        .byte ' + bytes_(coin(7)))
G.append('gfx_cap3:   ; Zeiger $0A ($8280): Kapsel von der Kante (3 Zeilen)')
G.append('        .byte ' + bytes_(coin(3)))
i = [k for k, l in enumerate(G) if l.startswith('gfx_ball:')][0]
open('lib/gfx.inc.chars', 'w').write('\n'.join(G[:i]) + '\n')
open('lib/gfx.inc.sprites', 'w').write('\n'.join(G[i:]) + '\n')
print('ok:', len(levels), 'Levels')
