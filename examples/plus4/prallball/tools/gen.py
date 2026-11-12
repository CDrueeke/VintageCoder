#!/usr/bin/env python3
"""Erzeugt die Datentabellen von PRALLBALL (Plus/4) im Ordner lib/:
  lib/dirtable.inc    Richtungstabelle des Balls (16 Geschwindigkeitsstufen x 8 Abprallzonen, Festkomma 1/64 Bildpunkt je Bild)
  lib/levels.inc      die 24 Levels aus levels/levels.txt (je 12 x 18 Steintypen) mit Levelart und Zeigertabellen
  lib/gfx.inc.chars   die eigenen Mehrfarbzeichen 64-67 (Steine, Wand, Block) und 127 (voller Block)
  lib/sprites.inc     die Zeichen-Sprites (Ball, Schlaeger, Laserschuss, Muenzen) als vorverschobene Streifen mit Masken
Die Ausgabedateien werden bei jedem Lauf vollstaendig neu geschrieben; Aenderungen von Hand gehen verloren.  Das Skript wechselt selbst in den
Ordner der Spieldatei (prallball.asm).  Bildpunkt-Werte der Sprites: siehe mp() und den Abschnitt "Zeichen-Sprites" unten.
Aufruf im Ordner examples/plus4/prallball:  python3 tools/gen.py"""
import math, re, os
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/plus4/prallball (os.chdir); alle Pfade gelten relativ dazu, der Aufruf funktioniert daher aus jedem Ordner.', epilog='Beispiele:\n  python3 examples/plus4/prallball/tools/gen.py\n  cd examples/plus4/prallball && python3 tools/gen.py\n\nSchreibt (im Projektordner examples/plus4/prallball, vorhandene Dateien werden ueberschrieben): lib/dirtable.inc, lib/levels.inc, lib/sprites.inc, lib/gfx.inc.chars.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))

# ---- Richtungstabelle: 16 Geschwindigkeitsstufen x 8 Zonen; Einheit 1/64 Bildpunkt je Bild (PAL 50 Hz)
ANG = [-60, -43, -26, -9, 9, 26, 43, 60]
def s16(v): return v & 0xffff
dxl, dxh, dyl, dyh = [], [], [], []
for lvl in range(16):
    speed = (1.6 + 0.1 * lvl) * 64
    for a in ANG:
        r = math.radians(a)
        dx = int(round(speed * math.sin(r))); dy = int(round(-speed * math.cos(r)))
        dxl.append(s16(dx) & 255); dxh.append(s16(dx) >> 8); dyl.append(s16(dy) & 255); dyh.append(s16(dy) >> 8)
def tab(name, vals):          # formatiert eine Tabelle als .byte-Zeilen zu je 16 Werten
    out = [name + ':']
    for i in range(0, len(vals), 16): out.append('        .byte ' + ', '.join('$%02x' % v for v in vals[i:i + 16]))
    return out
L = ['; ERZEUGT von tools/gen.py: Richtungstabelle der Ballgeschwindigkeit (Zone 0..7 = -60..+60 Grad), Einheit 1/64 Punkt je Bild',
     '; Index = Geschwindigkeitsstufe * 8 + Zone (Stufe 0..15, 128 Eintraege je Tabelle).  Zonen: -60, -43, -26, -9, +9, +26, +43, +60 Grad zur Senkrechten',
     '; (negativ = nach links).  Betrag der Geschwindigkeit = (1,6 + 0,1 * Stufe) Bildpunkte je Bild bei 50 Hz.',
     '; Jeder Wert ist eine 16-Bit-Zahl mit Vorzeichen (Zweierkomplement), getrennt in Low- (_l) und High-Byte (_h).  Y negativ = nach oben.']
for n, v, c in (('dir_xl', dxl, 'X-Komponente, Low-Byte'), ('dir_xh', dxh, 'X-Komponente, High-Byte ($FF = negativ)'),
                ('dir_yl', dyl, 'Y-Komponente, Low-Byte'), ('dir_yh', dyh, 'Y-Komponente, High-Byte (immer $FF: alle Richtungen zeigen nach oben)')):
    L.append('; ' + c)
    L += tab(n, v)
open('lib/dirtable.inc', 'w').write('\n'.join(L) + '\n')

# ---- Levels: levels/levels.txt lesen ("level N \"Name\"" gefolgt von 12 Zeilen zu 18 Zeichen) und in Steintypen umsetzen
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
M = ['; ERZEUGT von tools/gen.py aus levels/levels.txt: je Level 216 Byte (12 Zeilen x 18 Spalten, zeilenweise), Steintyp 0..13 bzw. 23',
     '; Steintypen: 0 leer, 1..8 Farbsteine (1 Treffer), 9 Silber (2 Treffer: wird zu Typ 1), 10 Gold (3 Treffer: wird zu Silber), 11 unzerstoerbar,',
     '; 12 explosiv, 13 Ueberraschung (laesst immer eine Kapsel fallen), 16..23 Boss-Kern (Typ - 15 Lebenspunkte; der Generator setzt nur 23 = 8).',
     'level_count = %d   ; Anzahl der Levels' % len(levels)]
for lv in levels:
    assert len(lv['rows']) == 12, lv['n']
    M.append('level%d:   ; %s' % (lv['n'], lv['name']))
    for r in lv['rows']: M.append('        .byte ' + ', '.join(str(TYPE[c]) for c in r))
M.append('level_kind:    .byte ' + ', '.join('2' if l['name'].startswith('Boss') else '1' if l['name'].startswith('Bonus') else '0' for l in levels) + '   ; 0 normal, 1 Bonus (Zeit, kein Ballverlust), 2 Boss (Kerne, Angriffe)')
M.append('; Adressen der Levels (Index 0 = Level 1), Low- und High-Byte')
M.append('level_tab_lo:  .byte ' + ', '.join('<level%d' % l['n'] for l in levels))
M.append('level_tab_hi:  .byte ' + ', '.join('>level%d' % l['n'] for l in levels))
open('lib/levels.inc', 'w').write('\n'.join(M) + '\n')

# ---- Grafik (Multicolor-Textmodus: je Zeile 4 Pixel zu 2 Bit; 0 = Hintergrund $FF15, 1 = $FF16 (hell), 2 = $FF17 (dunkel), 3 = Farb-RAM)
def mcrow(px):                # 4 Pixel (0..3) -> ein Byte, linker Pixel in Bit 7-6
    v = 0
    for p in px: v = (v << 2) | p
    return v
def mcchar(rows): return [mcrow(r) for r in rows]
BRICK_L = mcchar([[0,1,1,1],[0,1,3,3],[0,1,3,3],[0,1,3,3],[0,1,3,3],[0,1,3,3],[0,2,2,2],[0,0,0,0]])
BRICK_R = mcchar([[1,1,1,0],[3,3,2,0],[3,3,2,0],[3,3,2,0],[3,3,2,0],[3,3,2,0],[2,2,2,0],[0,0,0,0]])
WALL    = mcchar([[1,1,1,1],[1,3,3,2],[1,3,3,2],[1,3,3,2],[1,3,3,2],[1,3,3,2],[1,3,3,2],[2,2,2,2]])
BLOCK   = WALL
def bytes_(rows): return ', '.join('$%02x' % r for r in rows)
G = ['; ERZEUGT von tools/gen.py: eigene Zeichen (Mehrfarb-Textmodus: je Zeile 4 Pixel zu 2 Bit, 00 = Hintergrund $FF15, 01 = $FF16 hell,',
     '; 10 = $FF17 dunkel, 11 = Zellfarbe aus dem Farb-RAM).  Je Zeichen 8 Byte (eine Pixelzeile je Byte, Bit 7-6 = linker Pixel).']
G.append('gfx_chars:   ; Zeichen 64 (Stein links), 65 (Stein rechts), 66 (Wand), 67 (Block fuer das Titellogo), danach der volle Block (Zeichen 127)')
for ch, nm in ((BRICK_L, 'Stein links'), (BRICK_R, 'Stein rechts'), (WALL, 'Wand'), (BLOCK, 'Block (Titellogo)'), ([255] * 8, 'voller Block')): G.append('        .byte ' + bytes_(ch) + '   ; ' + nm)
# ---- Zeichen-Sprites (der TED hat keine Sprites).  Jedes Bild wird fuer die vier Feinlagen 0..3 (Mehrfarbpixel im Zeichen) vorverschoben und in
# Streifen je Zeichenspalte zerlegt.  Pixelwerte wie im Zeichensatz: 0 durchsichtig, 1 = $FF16 (hell), 2 = $FF17 (dunkel), 3 = Farbe der Zelle.
# (Die Bilder werden hier mit den Spritewerten der C64-Fassung entworfen - 1 hell, 2 Spritefarbe, 3 dunkel - und von mp() umgesetzt.)
def mp(v): return {0: 0, 1: 1, 2: 3, 3: 2}[v]
ball = [[0,2,0],[2,1,2],[2,2,3],[2,2,3],[2,3,3],[0,3,0]]
def padrows(left_round, right_round, width):
    rows = []
    for r in range(6):
        base = {0: 1, 1: 1, 2: 2, 3: 2, 4: 3, 5: 3}[r]
        row = [base] * width
        if r == 1: row[0] = 1 if left_round else base
        if left_round and r in (0, 5): row[0] = 0
        if right_round and r in (0, 5): row[-1] = 0
        rows.append(row)
    return rows
def hcat(a, b): return [x + y for x, y in zip(a, b)]
def coinrows(h):
    """Rotierende Muenze (Drehung um die WAAGERECHTE Achse), 6 Mehrfarbpixel breit, Hoehe h (12, 10, 7, 3), senkrecht mittig in 12 Zeilen.
    Rahmen 8 Pixel breit (zwei Zeichen), Muenze in den Spalten 1..6."""
    rows = [[0] * 8 for _ in range(12)]
    top = (12 - h) // 2
    for r in range(h):
        dy = (r - (h - 1) / 2.0) / (h / 2.0)
        w = int(round(6 * math.sqrt(max(0.0, 1 - dy * dy))))
        w = max(2, w - (w % 2))
        if h <= 3: w = 6
        left = 1 + (6 - w) // 2
        for k in range(w):
            if r == 0 and h > 3: v = 1
            elif r == h - 1 and h > 3: v = 3
            elif h <= 3: v = (1, 2, 3)[r]
            elif k == 0: v = 1
            elif k == w - 1: v = 3
            else: v = 2
            rows[top + r][left + k] = v
    return rows
FONT = {'B': 0x7c66667c66667c00, 'S': 0x3c66603c06663c00, 'R': 0x7c66667c786c6600, 'L': 0x6060606060607e00, 'F': 0x7e60607c60606000,
        'K': 0x666c7870786c6600, '+': 0x0018187e7e181800, 'P': 0x7c66667c60606000, 'M': 0x63777f6b63636300, 'G': 0x3c66606e66663c00,
        'D': 0x786c6666666c7800}
CAPLET = 'BSRLFK+PMGD'                    # Reihenfolge wie cap_letter im Spiel (Muenzenart 0..10)
def coin_front(letter):
    rows = coinrows(12)
    g = FONT[letter]
    for gr in range(8):
        b = (g >> (8 * (7 - gr))) & 255
        for k in range(4):                # 4 Mehrfarbpixel = je 2 Bildpunkte; dunkel, wenn einer gesetzt ist
            two = (b >> (6 - 2 * k)) & 3
            rows[2 + gr][2 + k] = 3 if two else 2     # Spritewerte: 3 = dunkel (Buchstabe), 2 = Muenzenfarbe
    return rows
IM = []                                   # (name, rows(Zeilen x Pixel), nsh)
def add(name, rows, nsh=4): IM.append((name, [[mp(v) for v in r] for r in rows], nsh))
add('ball', ball)
add('pad16', padrows(True, True, 8))
add('pad32', hcat(padrows(True, False, 8), padrows(False, True, 8)))
add('pad48', hcat(padrows(True, False, 12), padrows(False, True, 12)))
add('shot', [[2]] * 8)
for n, h in (('edge', 3), ('half', 7), ('tq', 10)): add('coin_' + n, coinrows(h), 1)
for c in CAPLET: add('coin_' + ('plus' if c == '+' else c), coin_front(c), 1)
S = ['; ERZEUGT von tools/gen.py: Zeichen-Sprites des TED (vorverschobene Streifen).  Siehe prallball.asm (Abschnitt Zeichen-Sprites).',
     '; Je Bild: Daten- und Maskenstreifen.  Streifen (Feinlage s, Zeichenspalte c) beginnen bei Bildanfang + (s * NC + c) * (H + 16); jeder Streifen',
     '; hat 8 Leerzeilen davor und danach, damit jede senkrechte Verschiebung 0..7 ohne Pruefung gelesen werden kann.',
     '; Maske: 0 bei Pixeln des Bildes (der Hintergrund wird geloescht), 3 bei durchsichtigen Pixeln (der Hintergrund bleibt).',
     '; Zusammensetzen im Zeichen: Ergebnis = (Hintergrund UND Maske) ODER Daten.  Pixelwerte (2 Bit): 01 = $FF16, 10 = $FF17, 11 = Zellfarbe.',
     '; Bilder 0 = Ball, 1..3 = Schlaeger 16/32/48 px, 4 = Laserschuss, 5..7 = Muenze Kante/halb/3-4, 8..18 = Muenze von vorn mit Buchstaben B S R L F K + P M G D.']
tabs = {k: [] for k in ('dlo', 'dhi', 'mlo', 'mhi', 'h', 'nc', 'smask', 'stride', 'sset', 'used')}
names = []
for idx, (name, rows, nsh) in enumerate(IM):
    H = len(rows); W = len(rows[0]); NC = (W + 3 + 3) // 4 if nsh == 4 else (W + 3) // 4
    if nsh == 1: NC = (W + 3) // 4
    stride = H + 16
    data, mask, used = [], [], []
    for s in range(nsh):
        u = 0
        for c in range(NC):
            d = [0] * 8; m = [255] * 8
            for r in range(H):
                bd = bm = 0
                for k in range(4):
                    x = 4 * c + k - s
                    p = rows[r][x] if 0 <= x < W else 0
                    bd = (bd << 2) | p
                    bm = (bm << 2) | (0 if p else 3)
                d.append(bd); m.append(bm)
            d += [0] * 8; m += [255] * 8
            if any(d[8:8 + H]): u = c + 1
            data += d; mask += m
        used.append(u)
    used += [used[0]] * (4 - len(used))
    S.append('imd_%d:   ; %s (%d x %d Pixel, %d Zeichenspalten, %d Feinlagen)' % (idx, name, W, H, NC, nsh))
    for i in range(0, len(data), 16): S.append('        .byte ' + ', '.join('$%02x' % v for v in data[i:i + 16]))
    S.append('imm_%d:' % idx)
    for i in range(0, len(mask), 16): S.append('        .byte ' + ', '.join('$%02x' % v for v in mask[i:i + 16]))
    tabs['dlo'].append('<imd_%d' % idx); tabs['dhi'].append('>imd_%d' % idx); tabs['mlo'].append('<imm_%d' % idx); tabs['mhi'].append('>imm_%d' % idx)
    tabs['h'].append(str(H)); tabs['nc'].append(str(NC)); tabs['smask'].append('3' if nsh == 4 else '0'); tabs['stride'].append(str(stride)); tabs['sset'].append(str(NC * stride)); tabs['used'] += [str(u) for u in used]
    names.append('IMG_' + name.upper() + ' = %d' % idx)
S.append('; Bildnummern')
for n in names: S.append('.const ' + n.replace(' = ', '    = '))
TABDOC = {'dlo': 'Anfang der Datenstreifen, Low-Byte', 'dhi': 'Anfang der Datenstreifen, High-Byte', 'mlo': 'Anfang der Maskenstreifen, Low-Byte',
          'mhi': 'Anfang der Maskenstreifen, High-Byte', 'h': 'Hoehe in Pixelzeilen', 'nc': 'Zeichenspalten je Streifensatz (Breite plus Platz fuer die Verschiebung)',
          'smask': 'AND-Maske fuer die Feinlage: 3 = vier Feinlagen, 0 = nur Feinlage 0 (Muenzen liegen auf der Zeichengrenze)',
          'stride': 'Laenge eines Streifens in Byte (Hoehe + 16)', 'sset': 'Abstand zweier Feinlagen im Bild (Zeichenspalten * Streifenlaenge)',
          'used': 'Index = Bild * 4 + Feinlage: Zahl der Zeichenspalten, die Pixel enthalten'}
for k in ('dlo', 'dhi', 'mlo', 'mhi', 'h', 'nc', 'smask', 'stride', 'sset', 'used'):
    S.append('; ' + TABDOC[k] + ' (Index = Bildnummer)' if k != 'used' else '; ' + TABDOC[k])
    S.append('im_%s:   .byte %s' % (k, ', '.join(tabs[k])))
open('lib/sprites.inc', 'w').write('\n'.join(S) + '\n')
open('lib/gfx.inc.chars', 'w').write('\n'.join(G) + '\n')
print('ok:', len(levels), 'Levels,', len(IM), 'Bilder')
