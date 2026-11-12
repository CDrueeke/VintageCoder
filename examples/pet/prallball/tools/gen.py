#!/usr/bin/env python3
"""Erzeugt die Tabellen fuer PRALLBALL (PET, 40 Spalten) aus levels/levels.txt und eigenen Formeln:
  lib/levels.inc    Levels (22, ohne Boss-Level), je Level 18 Spalten x 12 Zeilen, 1 Byte je Stein (Typ 0..13), zeilenweise = 216 Byte je Level
  lib/dirtable.inc  Richtungstabelle fuer set_vel: 8 Geschwindigkeitsstufen x 8 Abprallzonen x (vx, vy) als vorzeichenbehaftete 16-Bit-Zahlen in 1/256 Teilzelle je Bild
  lib/tabs.inc      Zeilenadressen der Overlay-Tabelle (OV + Zeile * 40), Zeilenanfaenge in der Levelkarte (Zeile * 18), Viertelblock-Tabelle (qtab)
Aufruf im Ordner examples/pet/prallball:  python3 tools/gen.py"""
import os, re, math
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\nArbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/pet/prallball (os.chdir); der Aufruf funktioniert daher aus jedem Ordner.", epilog='Beispiele:\n  python3 examples/pet/prallball/tools/gen.py\n  cd examples/pet/prallball && python3 tools/gen.py\n\nSchreibt (im Projektordner, vorhandene Dateien werden ueberschrieben): lib/levels.inc, lib/dirtable.inc, lib/tabs.inc.\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.', formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))

# ---- Levels
TYPE = {'.': 0, '#': 11, 'x': 12, 'p': 13, 's': 9, 'G': 10, 'C': 10}
for i, c in enumerate('abcdefgh'): TYPE[c] = i + 1
levels = []; cur = None
for line in open('levels/levels.txt', encoding='utf-8'):
    line = line.rstrip('\n')
    m = re.match(r'level (\d+) "(.*)"', line)
    if m: cur = {'n': int(m.group(1)), 'name': m.group(2), 'rows': []}; levels.append(cur); continue
    if cur is not None and line and not line.startswith('#') or (cur is not None and line.startswith('#') and len(line) == 18):
        if len(line) == 18: cur['rows'].append(line)
levels = [l for l in levels if not l['name'].startswith('Boss')]
M = ['; levels.inc - ERZEUGT von tools/gen.py aus levels/levels.txt (nicht von Hand aendern)',
     '; %d Levels (Boss-Level entfallen), je Level 18 Spalten x 12 Zeilen = 216 Byte, zeilenweise, 1 Byte je Stein.' % len(levels),
     '; Steintyp: 0 leer, 1..8 a..h, 9 Silber (s), 10 Gold (G), 11 unzerstoerbar (#), 12 explosiv (x), 13 Ueberraschung (p).',
     'level_count = %d' % len(levels), 'level_data:']
for lv in levels:
    assert len(lv['rows']) == 12, (lv['n'], len(lv['rows']))
    M.append('        ; %d %s' % (lv['n'], lv['name']))
    for r in lv['rows']: M.append('        .byte ' + ', '.join(str(TYPE[c]) for c in r))
M.append('level_tab_lo:    .byte ' + ', '.join('<(level_data + %d)' % (216 * i) for i in range(len(levels))))
M.append('level_tab_hi:    .byte ' + ', '.join('>(level_data + %d)' % (216 * i) for i in range(len(levels))))
open('lib/levels.inc', 'w', encoding='utf-8').write('\n'.join(M) + '\n')

# ---- Richtungstabelle: Betrag m(s) Teilzellen je Bild (60 Hz); Zone 0..7 = -60, -43, -26, -9, +9, +26, +43, +60 Grad gegen die Senkrechte
D = ['; dirtable.inc - ERZEUGT von tools/gen.py (nicht von Hand aendern)',
     '; dir_tab: 8 Geschwindigkeitsstufen (0..7) x 8 Abprallzonen (0..7) x (vx, vy), je 16 Bit vorzeichenbehaftet (niedrig, hoch) in 1/256 Teilzelle je Bild.',
     '; Index = (Stufe * 8 + Zone) * 4.  Betrag = 0,70 + 0,08 * Stufe Teilzellen je Bild; vx = Betrag * sin(Winkel), vy = -Betrag * cos(Winkel) (negativ = nach oben).',
     'dir_tab:']
ang = [-60, -43, -26, -9, 9, 26, 43, 60]
for s in range(8):
    mag = 0.70 + 0.08 * s
    row = []
    for a in ang:
        vx = round(mag * math.sin(math.radians(a)) * 256); vy = round(-mag * math.cos(math.radians(a)) * 256)
        row += [vx & 255, (vx >> 8) & 255, vy & 255, (vy >> 8) & 255]
    D.append('        .byte ' + ', '.join('$%02x' % b for b in row))
open('lib/dirtable.inc', 'w', encoding='utf-8').write('\n'.join(D) + '\n')

# ---- Tabellen: OV = $6C00 (Overlay, 1000 Byte), Zeilen * 40; Zeilen * 18; Viertelblock-Tabelle
OV = 0x6c00
T = ['; tabs.inc - ERZEUGT von tools/gen.py (nicht von Hand aendern)',
     '; ovrow_lo/hi: Adresse der Zeile 0..24 in der Overlay-Tabelle (OV + Zeile * 40); die Bildschirmzeile liegt $1400 darueber ($8000 - $6C00).',
     'ovrow_lo:       .byte ' + ', '.join('$%02x' % ((OV + 40 * r) & 255) for r in range(25)),
     'ovrow_hi:       .byte ' + ', '.join('$%02x' % ((OV + 40 * r) >> 8) for r in range(25)),
     '; rowlv: Zeilenanfang in der Levelkarte (Zeile 0..11 * 18)',
     'rowlv:          .byte ' + ', '.join(str(18 * r) for r in range(12)),
     '; qtab: Bildschirmcode je Viertelblock-Maske (Bit 0 oben links, 1 oben rechts, 2 unten links, 3 unten rechts).  Mit dem Zeichensatz des PET geprueft: es gibt',
     '; die Viertelbloecke 126 (oben links), 124 (oben rechts), 123 (unten links), 108 (unten rechts), 97 (linke Haelfte), 98 (untere Haelfte), 127 (oben links + unten rechts);',
     '; alle anderen Muster entstehen als Umkehrschrift (+128) des Gegenmusters.',
     'qtab:           .byte 32, 126, 124, 226, 123, 97, 255, 236, 108, 127, 225, 251, 98, 252, 254, 160']
open('lib/tabs.inc', 'w', encoding='utf-8').write('\n'.join(T) + '\n')
print('ok:', len(levels), 'Levels')
