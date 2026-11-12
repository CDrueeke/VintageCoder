#!/usr/bin/env python3
"""Erzeugt die Datendateien von PRALLBALL (VC-20, 8 KB), die prallball.asm per .include einbindet:
  lib/dirtable.inc  Richtungstabelle dir_tab (8 Geschwindigkeitsstufen x 8 Abprallzonen)
  lib/levels.inc    Levels auf 10 Spalten, 4 Bit je Stein (Quelle: levels/levels.txt, 18 Spalten)
  lib/gfx.inc       eigene Mehrfarbzeichen (Stein links/rechts, Wand)
  lib/logo.inc      Titellogo "PRALL BALL" als Bitmuster
  lib/music.inc     gekuerztes Titellied (Quelle: ../../c64/prallball/lib/music.inc, muss vorhanden sein)
Aufruf im Ordner examples/vic20/prallball:  python3 tools/gen.py   (schreibt die lib-Dateien neu; ihre Kommentarkoepfe stehen in diesem Skript)"""
import math, re, os
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/vic20/prallball (os.chdir); alle Pfade gelten relativ dazu, der Aufruf funktioniert daher aus jedem Ordner.', epilog='Beispiele:\n  python3 examples/vic20/prallball/tools/gen.py\n  cd examples/vic20/prallball && python3 tools/gen.py\n\nSchreibt (im Projektordner examples/vic20/prallball, vorhandene Dateien werden ueberschrieben): lib/dirtable.inc, lib/levels.inc, lib/gfx.inc, lib/music.inc, lib/logo.inc.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))

# ---- Richtungstabelle: 8 Geschwindigkeitsstufen x 8 Zonen; je (vx, vy) als vorzeichenbehaftetes Byte, Einheit 1/32 Bildpunkt je Bild (PAL 50 Hz)
ANG = [-60, -43, -26, -9, 9, 26, 43, 60]
tab = []
for lvl in range(8):
    speed = (0.95 + 0.14 * lvl) * 32
    for a in ANG:
        r = math.radians(a)
        tab.append((int(round(speed * math.sin(r))), int(round(-speed * math.cos(r)))))
L = ['; dirtable.inc - ERZEUGT von tools/gen.py (nicht von Hand aendern)',
     '; dir_tab: Richtungstabelle fuer set_vel, 8 Geschwindigkeitsstufen (0..7) x 8 Abprallzonen (0..7) x (vx, vy) = 128 Byte; Index = (Stufe * 8 + Zone) * 2.',
     '; Zone 0..7 = Abprallwinkel -60, -43, -26, -9, +9, +26, +43, +60 Grad gegen die Senkrechte (von links nach rechts).  Betrag = (0,95 + 0,14 * Stufe) * 32;',
     '; vx = Betrag * sin(Winkel), vy = -Betrag * cos(Winkel) (negativ = nach oben), je ein vorzeichenbehaftetes Byte in 1/32 Bildpunkt je Bild (PAL, 50 Hz).',
     'dir_tab:']
for i in range(0, 64, 4):
    L.append('        .byte ' + ', '.join('%d, %d' % (vx & 255 if False else vx, vy) for vx, vy in tab[i:i + 4]).replace('-', '256 - ') if False else '        .byte ' + ', '.join('$%02x, $%02x' % (vx & 255, vy & 255) for vx, vy in tab[i:i + 4]))
open('lib/dirtable.inc', 'w').write('\n'.join(L) + '\n')

# ---- Levels: 18 -> 10 Spalten (Spalte j <- alte Spalte (18 j + 9) // 10, spiegelsymmetrisch), 12 Zeilen, je Stein 4 Bit (Typ 0..13); Boss-Level entfallen
TYPE = {'.': 0, 'a': 1, 'b': 2, 'c': 3, 'd': 4, 'e': 5, 'f': 6, 'g': 7, 'h': 8, 's': 9, 'G': 10, '#': 11, 'x': 12, 'p': 13, 'C': 11}
levels = []; cur = None
for line in open('levels/levels.txt', encoding='utf-8'):
    line = line.rstrip('\n')
    m = re.match(r'level (\d+) "(.*)"', line)
    if m: cur = {'n': int(m.group(1)), 'name': m.group(2), 'rows': []}; levels.append(cur); continue
    if cur is not None and len(line) == 18 and not line.startswith('#  '): cur['rows'].append(line)
levels = [l for l in levels if not l['name'].startswith('Boss')]
M = ['; levels.inc - ERZEUGT von tools/gen.py aus levels/levels.txt (nicht von Hand aendern)',
     '; %d Levels (20 regulaere + 2 Bonus; die Boss-Level des C64 entfallen), je Level 10 Spalten x 12 Zeilen = 120 Steine zu 4 Bit = 60 Byte, zeilenweise;' % len(levels),
     '; zwei Steine je Byte, links = hohes Nibble.  Steintyp: 0 leer, 1..8 a..h, 9 Silber (s), 10 Gold (G), 11 unzerstoerbar (#), 12 explosiv (x), 13 Ueberraschung (p).',
     '; Die 18 Spalten der Entwuerfe werden auf 10 abgebildet: Spalte j <- Entwurfsspalte (18 * j + 9) // 10.  level_kind: je Level 0 = normal, 1 = Bonus.',
     'level_count = %d' % len(levels), 'level_data:']
kinds = []
for lv in levels:
    assert len(lv['rows']) == 12, lv['n']
    cells = []
    for row in lv['rows']:
        for j in range(10): cells.append(TYPE[row[(18 * j + 9) // 10]])
    by = [(cells[i] << 4) | cells[i + 1] for i in range(0, 120, 2)]
    M.append('        ; %d %s' % (lv['n'], lv['name']))
    for i in range(0, 60, 10): M.append('        .byte ' + ', '.join('$%02x' % b for b in by[i:i + 10]))
    kinds.append(1 if lv['name'].startswith('Bonus') else 0)
M.append('level_kind:    .byte ' + ', '.join(map(str, kinds)) + '   ; 0 normal, 1 Bonus (Zeit, kein Ballverlust)')
open('lib/levels.inc', 'w').write('\n'.join(M) + '\n')

# ---- Mehrfarbzeichen (VIC-I: Pixelpaar 00 Hintergrund, 01 Rahmenfarbe, 10 Zeichenfarbe, 11 Hilfsfarbe): Licht = Hilfsfarbe (weiss), Schatten = Rahmen (schwarz)
def mcrow(px):
    v = 0
    for p in px: v = (v << 2) | {0: 0, 1: 3, 2: 1, 3: 2}[p]      # Entwurf wie C64: 1 hell, 2 dunkel, 3 Zeichenfarbe
    return v
BRICK_L = [[0,1,1,1],[0,1,3,3],[0,1,3,3],[0,1,3,3],[0,1,3,3],[0,1,3,3],[0,2,2,2],[0,0,0,0]]
BRICK_R = [[1,1,1,0],[3,3,2,0],[3,3,2,0],[3,3,2,0],[3,3,2,0],[3,3,2,0],[2,2,2,0],[0,0,0,0]]
WALL    = [[1,1,1,1],[1,3,3,2],[1,3,3,2],[1,3,3,2],[1,3,3,2],[1,3,3,2],[1,3,3,2],[2,2,2,2]]
G = ['; gfx.inc - ERZEUGT von tools/gen.py (nicht von Hand aendern)',
     '; gfx_chars: drei eigene Mehrfarbzeichen zu je 8 Byte (video_init kopiert sie nach CHARSET + 64 * 8): 64 Stein links, 65 Stein rechts, 66 Wand (zugleich Logoblock).',
     '; VIC-I-Mehrfarbe (Farb-RAM-Bit 3 gesetzt): je zwei Bit = ein doppelt breiter Punkt, 4 Punkte je Zeile: 00 Hintergrund, 01 Rahmenfarbe (schwarz = Schatten),',
     '; 10 Zeichenfarbe (Farb-RAM Bits 0..2 = Steinfarbe), 11 Hilfsfarbe (Bits 4..7 von $900E, weiss = Licht).  Licht oben/links, Schatten unten/rechts, Fugen leer.',
     'gfx_chars:']
for ch in (BRICK_L, BRICK_R, WALL): G.append('        .byte ' + ', '.join('$%02x' % mcrow(r) for r in ch))
open('lib/gfx.inc', 'w').write('\n'.join(G) + '\n')
print('ok:', len(levels), 'Levels')

# ---- Lied: die ersten N Takte des C64-Liedes, benutzte Muster neu nummeriert
N = 16
src = open('../../c64/prallball/lib/music.inc').read()
head = re.search(r'\.byte (\d+), (\d+), (\d+), <(\d+), >(\d+)', src)
fpb = int(head.group(1)); total = int(head.group(2))
order = [tuple(int(x) for x in m.groups()) for m in re.finditer(r'^\s+\.byte (\d+), (\d+), (\d+)\s+; Reihenfolge', src, re.M)] if False else None
lines = src.split('\n'); i0 = [k for k, l in enumerate(lines) if l.startswith('song:')][0]
ordr = []; pats = []; k = i0 + 2
while not lines[k].startswith('song_patterns'):
    if not lines[k].strip() or lines[k].lstrip().startswith(';'):
        k += 1; continue                       # Kommentar- und Leerzeilen der C64-Datei ueberspringen
    m = re.match(r'\s+\.byte (\d+), (\d+), (\d+)', lines[k]); ordr.append(tuple(int(x) for x in m.groups())); k += 1
k += 1
while k < len(lines) and lines[k].strip().startswith('.byte'):
    pats.append([int(x) for x in re.findall(r'\d+', lines[k])]); k += 1
used = []; new_order = []
for t in ordr[:N]:
    row = []
    for p in t:
        if p not in used: used.append(p)
        row.append(used.index(p))
    new_order.append(tuple(row))
S = ['; music.inc - ERZEUGT von tools/gen.py (nicht von Hand aendern): gekuerztes Titellied (%d Takte, %d Muster) aus dem C64-Lied (examples/c64/prallball/lib/music.inc)' % (N, len(used)),
     '; Format (siehe examples/c64/prallball/lib/sound.asm; examples/vic20/prallball/lib/sound.asm liest es): Liedkopf mit 5 Byte = Bilder je Zeile, Takte, Wiederholen, Abstand der Mustertabelle',
     '; (16 Bit, vom Liedkopf aus); dann je Takt 3 Musternummern (Spur 0 Melodie = Sopran, 1 Bass, 2 Arpeggio = Alt und Schlagzeug = Rauschen); dann die Muster',
     '; zu je 16 Zeilen (1/16-Noten): 0 = halten, 1..95 = Note, 96 = Pause, 128 + n = Schlagzeug n (nur Spur 2).',
     'song:',
     '        .byte %d, %d, 1, <%d, >%d       ; Bilder je Zeile, Takte, Wiederholen, Abstand der Mustertabelle' % (fpb, N, 5 + 3 * N, 5 + 3 * N)]
for t in new_order: S.append('        .byte %d, %d, %d' % t)
S.append('song_patterns:')
for p in used: S.append('        .byte ' + ', '.join(str(b) for b in pats[p]))
open('lib/music.inc', 'w').write('\n'.join(S) + '\n')
print('Lied:', N, 'Takte,', len(used), 'Muster =', 5 + 3 * N + 16 * len(used), 'Byte')

# ---- Titellogo: "PRALL" (Zeilen 2..6, Spalte 1..) und "BALL" (Zeilen 8..12, Spalte 3..), 3 x 5 Blockschrift, je Zeile 24 Bit (Spalte 0 = hoechstes Bit)
FONT = {'P': ['111','101','111','100','100'], 'R': ['111','101','110','101','101'], 'A': ['111','101','111','101','101'],
        'L': ['100','100','100','100','111'], 'B': ['110','101','110','101','110']}
rows = []; ys = []
for word, col0, y0 in (('PRALL', 1, 2), ('BALL', 3, 8)):
    for r in range(5):
        line = [0] * 24
        for k, ch in enumerate(word):
            for x, bit in enumerate(FONT[ch][r]):
                if bit == '1': line[col0 + 4 * k + x] = 1
        v = 0
        for b in line: v = (v << 1) | b
        rows.append(v); ys.append(y0 + r)
LG = ['; logo.inc - ERZEUGT von tools/gen.py (nicht von Hand aendern)',
      '; logo_bits: Titellogo "PRALL" / "BALL" in 3 x 5 Blockschrift, 10 Zeilen zu je 3 Byte = 24 Bit (hoechstes Bit von Byte 0 = Bildschirmspalte 0; draw_logo benutzt',
      '; die Spalten 0..21); jedes gesetzte Bit wird ein Wandzeichen.  logo_y: Bildschirmzeile je Logozeile (PRALL: 2..6, BALL: 8..12).',
      'logo_bits:']
for v in rows: LG.append('        .byte $%02x, $%02x, $%02x' % (v >> 16, (v >> 8) & 255, v & 255))
LG.append('logo_y:      .byte ' + ', '.join(map(str, ys)))
open('lib/logo.inc', 'w').write('\n'.join(LG) + '\n')
