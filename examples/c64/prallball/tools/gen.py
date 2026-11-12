#!/usr/bin/env python3
"""Erzeugt die Datendateien von PRALLBALL (C64), die prallball.asm per .include einbindet:
  lib/dirtable.inc     Richtungstabelle dir_xl/dir_xh/dir_yl/dir_yh: 16 Geschwindigkeitsstufen x 8 Abprallzonen (Index = Stufe * 8 + Zone)
  lib/levels.inc       Steinkarten level1..level24 (je 216 Byte), level_kind, level_tab_lo/hi; Quelle: levels/levels.txt
  lib/gfx.inc.chars    die vier eigenen Zeichen 64..67 (Multicolor)
  lib/gfx.inc.sprites  die Sprite-Bloecke (Zeiger $40..$4A, in prallball.asm ab $5000): Ball, Schlaeger, Kapsel (4 Drehphasen), Laserschuss
Die Kommentare der erzeugten Dateien stehen in diesem Skript; die erzeugten Dateien nicht von Hand aendern.
Aufruf im Ordner examples/c64/prallball:  python3 tools/gen.py"""
import math, re, os
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/c64/prallball (os.chdir); alle Pfade gelten relativ dazu, der Aufruf funktioniert daher aus jedem Ordner.', epilog='Beispiele:\n  python3 examples/c64/prallball/tools/gen.py\n  cd examples/c64/prallball && python3 tools/gen.py\n\nSchreibt (im Projektordner examples/c64/prallball, vorhandene Dateien werden ueberschrieben): lib/dirtable.inc, lib/levels.inc, lib/gfx.inc.chars, lib/gfx.inc.sprites.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))

# ---- Richtungstabelle: 16 Geschwindigkeitsstufen x 8 Zonen; Einheit 1/64 Bildpunkt je Bild (PAL 50 Hz)
#      Zone = Trefferstelle auf dem Schlaeger (0 = ganz links); ANG = Abweichung von der Senkrechten in Grad (negativ = nach links).
#      Betrag der Geschwindigkeit = (1,6 + 0,1 * Stufe) Bildpunkte je Bild; vx = Betrag * sin(Winkel), vy = -Betrag * cos(Winkel)
#      (negativ = nach oben).  Jeder Wert hat 16 Bit (Zweierkomplement) und wird auf eine Low- und eine High-Byte-Tabelle verteilt.
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
L = ['; ERZEUGT von tools/gen.py: Richtungstabelle (Zone 0..7 = -60..+60 Grad), Index = Stufe * 8 + Zone (Stufe 0..15), Einheit 1/64 Bildpunkt je Bild']
for n, v in (('dir_xl', dxl), ('dir_xh', dxh), ('dir_yl', dyl), ('dir_yh', dyh)): L += tab(n, v)
open('lib/dirtable.inc', 'w').write('\n'.join(L) + '\n')

# ---- Levels
# Zeichen im Level-Text -> Steintyp (Bedeutung der Typen: prallball.asm, brick_color/brick_pts): a-h = Typ 1-8 (Farben), s = Silber (9),
# G = Gold (10), # = unzerstoerbar (11), x = explosiv (12), p = Ueberraschung (13), C = Boss-Kern (23 = 8 Treffer).
# 'g' ist die siebte Farbe (Typ 7); Gold schreibt man 'G'.
TYPE = {'.': 0, 'a': 1, 'b': 2, 'c': 3, 'd': 4, 'e': 5, 'f': 6, 'g': 7, 'h': 8, 's': 9, 'G': 10, '#': 11, 'x': 12, 'p': 13, 'C': 23}
# levels/levels.txt lesen: 'level N "Name"' beginnt ein Level, danach 12 Zeilen zu je 18 Zeichen (Spalten); Kommentarzeilen ('# ') werden uebersprungen.
levels = []; cur = None
for line in open('levels/levels.txt', encoding='utf-8'):
    line = line.rstrip('\n')
    if line.startswith('#') and not line.startswith('#a') and ' ' in line[:2] + ' ' and re.match(r'#\s', line): continue
    m = re.match(r'level (\d+) "(.*)"', line)
    if m: cur = {'n': int(m.group(1)), 'name': m.group(2), 'rows': []}; levels.append(cur); continue
    if cur is not None and len(line) == 18: cur['rows'].append(line)
M = ['; ERZEUGT von tools/gen.py aus levels/levels.txt: je Level 216 Byte (12 Zeilen x 18 Spalten), Steintyp 0..13 oder 23 (Boss-Kern)', 'level_count = %d' % len(levels)]
for lv in levels:
    assert len(lv['rows']) == 12, lv['n']
    M.append('level%d:   ; %s' % (lv['n'], lv['name']))
    for r in lv['rows']: M.append('        .byte ' + ', '.join(str(TYPE[c]) for c in r))
M.append('level_kind:    .byte ' + ', '.join('2' if l['name'].startswith('Boss') else '1' if l['name'].startswith('Bonus') else '0' for l in levels) + '   ; 0 normal, 1 Bonus (Zeit, kein Ballverlust), 2 Boss (Kerne, Angriffe)')
M.append('; Adresstabellen der Levelkarten (Index = Levelnummer - 1)')
M.append('level_tab_lo:  .byte ' + ', '.join('<level%d' % l['n'] for l in levels))
M.append('level_tab_hi:  .byte ' + ', '.join('>level%d' % l['n'] for l in levels))
open('lib/levels.inc', 'w').write('\n'.join(M) + '\n')

# ---- Grafik (Multicolor-Textmodus: je Zeile 4 Pixel zu 2 Bit; 0 = Hintergrund $D021, 1 = $D022 (hell), 2 = $D023 (dunkel), 3 = Farb-RAM)
def mcrow(px):
    """Eine Zeile Multicolor-Pixel (Werte 0..3, je 2 Bit, links zuerst) zu einem Byte; 4 Pixel je Byte."""
    v = 0
    for p in px: v = (v << 2) | p
    return v
def mcchar(rows): return [mcrow(r) for r in rows]
# Stein = zwei Zeichen nebeneinander (links/rechts, je 8 x 8 px = 4 x 8 Multicolor-Pixel): Lichtkante oben und links (1), Schattenkante unten
# und rechts (2), Flaeche in der Farbe des Steins (3 = Farb-RAM), unterste Zeile und aeusserste Pixelspalte frei (0 = Fuge).  Wand und Logoblock sind rundum gerahmt.
BRICK_L = mcchar([[0,1,1,1],[0,1,3,3],[0,1,3,3],[0,1,3,3],[0,1,3,3],[0,1,3,3],[0,2,2,2],[0,0,0,0]])
BRICK_R = mcchar([[1,1,1,0],[3,3,2,0],[3,3,2,0],[3,3,2,0],[3,3,2,0],[3,3,2,0],[2,2,2,0],[0,0,0,0]])
WALL    = mcchar([[1,1,1,1],[1,3,3,2],[1,3,3,2],[1,3,3,2],[1,3,3,2],[1,3,3,2],[1,3,3,2],[2,2,2,2]])
BLOCK   = WALL
def bytes_(rows): return ', '.join('$%02x' % r for r in rows)
G = ['; ERZEUGT von tools/gen.py: eigene Zeichen (Multicolor-Textmodus: je Byte 4 Pixel zu 2 Bit; 00 Hintergrund $D021, 01 $D022, 10 $D023, 11 Farb-RAM)']
G.append('gfx_chars:   ; Zeichen 64 (Stein links), 65 (Stein rechts), 66 (Wand), 67 (Block fuer das Titellogo); alle Multicolor')
for ch in (BRICK_L, BRICK_R, WALL, BLOCK): G.append('        .byte ' + bytes_(ch))
# Sprites (Multicolor): je Zeile 2 Byte = 8 Pixel zu 2 Bit; 1 = $D025 (hell), 2 = Spritefarbe, 3 = $D026 (dunkel)
def spr(rows, nbytes):
    """Sprite-Block (64 Byte) aus Pixelzeilen (Werte 0..3); nbytes = Bytes je Zeile (1 = 4 Pixel, 2 = 8 Pixel, 3 = 12 Pixel); der Rest ist 0."""
    out = []
    for r in rows:
        r = list(r) + [0] * (nbytes * 4 - len(r))
        for k in range(nbytes): out.append(mcrow(r[k * 4:k * 4 + 4]))
        out += [0] * (3 - nbytes)
    out += [0] * (64 - len(out))
    return out
# Ball: 6 x 6 px = 3 x 6 Multicolor-Pixel, Lichtpunkt (1) im oberen Drittel, Kugel in der Spritefarbe (2), Schattenseite unten rechts (3)
ball = [[0,2,0],[2,1,2],[2,2,3],[2,2,3],[2,3,3],[0,3,0]]
G.append('; SPRITES: Bloecke zu je 64 Byte; Sprite-Zeiger = (Adresse - $4000) / 64 (VIC-Bank 1).  prallball.asm legt sie mit .org $5000 ab.')
G.append('gfx_ball:    ; Zeiger $40 ($5000): Multicolor-Ball 6 x 6 px (3 x 6 Multicolor-Pixel)')
G.append('        .byte ' + bytes_(spr(ball, 1)))
def pad(left_round, right_round, width):
    """Schlaeger (oder Haelfte): 6 Zeilen hoch, Zeilen 0-1 hell (1), 2-3 Spritefarbe (2), 4-5 dunkel (3); an einem runden Ende fehlt das Eckpixel
    in der obersten und untersten Zeile.  width = Breite in Multicolor-Pixeln (8 = 16 px, 12 = 24 px)."""
    rows = []
    for r in range(6):
        base = {0: 1, 1: 1, 2: 2, 3: 2, 4: 3, 5: 3}[r]
        row = [base] * width
        if r == 1: row[0] = 1 if left_round else base
        if left_round and r in (0, 5): row[0] = 0
        if right_round and r in (0, 5): row[-1] = 0
        rows.append(row)
    return spr(rows, width // 4)
G.append('gfx_padl:   ; Zeiger $41 ($5040): Schlaegerhaelfte links (16 px = 8 Multicolor-Pixel)')
G.append('        .byte ' + bytes_(pad(True, False, 8)))
G.append('gfx_padr:   ; Zeiger $42 ($5080): Schlaegerhaelfte rechts (16 px)')
G.append('        .byte ' + bytes_(pad(False, True, 8)))
G.append('gfx_pad24l: ; Zeiger $43 ($50C0): breiter Schlaeger links (24 px = 12 Multicolor-Pixel)')
G.append('        .byte ' + bytes_(pad(True, False, 12)))
G.append('gfx_pad24r: ; Zeiger $44 ($5100): breiter Schlaeger rechts (24 px)')
G.append('        .byte ' + bytes_(pad(False, True, 12)))
G.append('gfx_pad16:  ; Zeiger $45 ($5140): schmaler Schlaeger (16 px, beide Enden rund)')
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
G.append('gfx_cap:    ; Zeiger $46 ($5180): Kapsel vorn (12 x 12 px); der Buchstabe wird zur Laufzeit in Byte 1 der Zeilen 2..9 eingetragen (spawn_cap)')
G.append('        .byte ' + bytes_(coin(12)))
G.append('gfx_shot:   ; Zeiger $47 ($51C0): Laserschuss (2 x 8 Bildpunkte, einfarbig, Hires-Sprite)')
G.append('        .byte ' + bytes_([0xc0, 0, 0] * 8 + [0] * 40))
G.append('gfx_cap1:   ; Zeiger $48 ($5200): Kapsel gekippt (10 Zeilen hoch)')
G.append('        .byte ' + bytes_(coin(10)))
G.append('gfx_cap2:   ; Zeiger $49 ($5240): Kapsel halb gekippt (7 Zeilen)')
G.append('        .byte ' + bytes_(coin(7)))
G.append('gfx_cap3:   ; Zeiger $4A ($5280): Kapsel von der Kante (3 Zeilen)')
G.append('        .byte ' + bytes_(coin(3)))
i = [k for k, l in enumerate(G) if l.startswith('; SPRITES')][0]        # ab hier gehoert alles in lib/gfx.inc.sprites
open('lib/gfx.inc.chars', 'w').write('\n'.join(G[:i]) + '\n')
open('lib/gfx.inc.sprites', 'w').write('\n'.join(G[i:]) + '\n')
print('ok:', len(levels), 'Levels')
