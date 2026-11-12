#!/usr/bin/env python3
"""Erzeugt prallball.vctest fuer den Atari (statische Pruefungen und Emulator-Tests im Trap-Modus: `vcc test --backend atari800`).
Erwartungswerte (Richtungstabelle, CRC, Levelzaehlungen) werden hier unabhaengig vom Programm berechnet; Speicherstellen werden ueber ihre Marken angesprochen.
Aufruf im Ordner examples/atari/prallball:  python3 tools/gen_tests.py   (vorher: vcc build .)
Schreibt prallball.vctest (nicht von Hand aendern): 68 Emulator-Tests plus 3 statische Pruefungen (Ueberlappung, Programmende vor $8000, Zyklen von add_score)."""
import json, math, os, re
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/atari/prallball (os.chdir); alle Pfade gelten relativ dazu, der Aufruf funktioniert daher aus jedem Ordner.', epilog='Beispiele:\n  python3 examples/atari/prallball/tools/gen_tests.py\n  cd examples/atari/prallball && python3 tools/gen_tests.py\n\nSchreibt (im Projektordner examples/atari/prallball, vorhandene Dateien werden ueberschrieben): prallball.vctest.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
def h(v): return '$%02x' % v
ANG = [-60, -43, -26, -9, 9, 26, 43, 60]
def vel(level, zone):
    speed = (1.6 + 0.1 * level) * 64
    a = math.radians(ANG[zone])
    return int(round(speed * math.sin(a))) & 0xffff, int(round(-speed * math.cos(a))) & 0xffff
tests = []
def test(name, setup, call, expect, cycles=None):
    t = {'name': name, 'setup': setup, 'call': call, 'expect': expect}
    if cycles: t['cycles'] = {'max': cycles}
    tests.append(t)
test('add_score: 0999 + 10 = 1009', {'mem': {'sc0': 0x99, 'sc1': 0x09, 'sc2': 0}, 'a': '$10'}, 'add_score', ['[sc0] == $09', '[sc1] == $10', '[sc2] == 0'], cycles=60)
test('add_score: Uebertrag in die hoechsten Ziffern', {'mem': {'sc0': 0x95, 'sc1': 0x99, 'sc2': 0}, 'a': '$10'}, 'add_score', ['[sc0] == $05', '[sc1] == $00', '[sc2] == $01'])
for lvl, zone in ((0, 3), (5, 0), (15, 7)):
    dx, dy = vel(lvl, zone)
    test('set_vel: Stufe %d Zone %d' % (lvl, zone), {'mem': {'speed': lvl, 'spdmod': 0}, 'a': str(zone)}, 'set_vel',
         ['[vxl] == %s' % h(dx & 255), '[vxh] == %s' % h(dx >> 8), '[vyl] == %s' % h(dy & 255), '[vyh] == %s' % h(dy >> 8)], cycles=70)
base = {'padl': 100, 'padh': 0, 'padw': 32, 'zstep': 4, 'speed': 0, 'spdmod': 0, 'hits': 0, 'sticky': 0, 'cb': 0, 'byl': 0x80, 'byh': 0x2c, 'vyh': 0, 'vyl': 0x40, 'mask': 0, 'oyl': 0, 'oyh': 0}
dx, dy = vel(0, 4)
s = dict(base); s.update({'bxl': 0x00, 'bxh': 0x1d})
test('paddle_hit: Treffer in der Mitte -> Zone 4 und Abprall nach oben', {'mem': s}, 'paddle_hit', ['fc', '[vxl] == %s' % h(dx & 255), '[vyl] == %s' % h(dy & 255), '[vyh] == %s' % h(dy >> 8), '[hits] == 1'])
s = dict(base); s.update({'bxl': 0x00, 'bxh': 0x32})
test('paddle_hit: Ball neben dem Schlaeger -> kein Abprall', {'mem': s}, 'paddle_hit', ['!fc', '[hits] == 0'])
for mk, zone, nm in ((0x04, 3, 'nach links'), (0x08, 5, 'nach rechts'), (0x00, 4, 'stillstehend')):
    dx, dy = vel(0, zone)
    s = dict(base); s.update({'bxl': 0x00, 'bxh': 0x1d, 'mask': mk})
    test('paddle_hit: Schlaeger %s bewegt -> Zone %d' % (nm, zone), {'mem': s}, 'paddle_hit', ['fc', '[vxl] == %s' % h(dx & 255), '[vxh] == %s' % h(dx >> 8)])
s = dict(base); s.update({'bxl': 0x40, 'bxh': 0x18, 'oyl': 0x90, 'oyh': 0x2c, 'vxl': 0x40, 'vxh': 0})
test('paddle_hit: Seitenaufprall links -> weg nach links', {'mem': s}, 'paddle_hit', ['!fc', '[vxl] == $c0', '[vxh] == $ff', '[bxl] == $80', '[bxh] == $17', '[hits] == 0'])
s = dict(base); s.update({'bxl': 0x80, 'bxh': 0x20, 'oyl': 0x90, 'oyh': 0x2c, 'vxl': 0xc0, 'vxh': 0xff})
test('paddle_hit: Seitenaufprall rechts -> weg nach rechts', {'mem': s}, 'paddle_hit', ['!fc', '[vxl] == $40', '[vxh] == 0', '[bxl] == 0', '[bxh] == $21', '[hits] == 0'])
mapb = {'MAP': [1] + [0] * 215}
s = {'bxh': 4, 'bxl': 0, 'byh': 8, 'byl': 0}; s.update(mapb)
test('probe: Stein oben links wird gefunden', {'mem': s, 'x': 0, 'y': 0}, 'probe', ['a == 1', '[bcol] == 0', '[brow] == 0', '[bidx] == 0'])
s = {'bxh': 3, 'bxl': 0, 'byh': 8, 'byl': 0}; s.update(mapb)
test('probe: links vom Steinfeld kein Stein', {'mem': s, 'x': 0, 'y': 0}, 'probe', ['a == 0', 'fz'])
s = {'bxl': 0x20, 'bxh': 0x08, 'byl': 0x00, 'byh': 0x08, 'vxl': 0xc0, 'vxh': 0xff, 'vyl': 0x10, 'vyh': 0, 'bricks': 1, 'sc0': 0, 'sc1': 0, 'sc2': 0, 'lkind': 0, 'capact': 1, 'tpierce': 0}
s.update(mapb)
test('move_ball: Treffer links auf einen Stein: Stein weg, Punkte, Ball kehrt um', {'mem': s}, 'move_ball', ['[MAP] == 0', '[bricks] == 0', '[sc0] == $10', '[vxh] == 0'])
s2 = dict(s); s2.update({'MAP': [9] + [0] * 215, 'tpierce': 8})
test('move_ball: Durchschlag zerstoert Silber sofort und prallt nicht ab', {'mem': s2}, 'move_ball', ['[MAP] == 0', '[bricks] == 0', '[vxh] == $ff'])
s3 = dict(s); s3.update({'MAP': [11] + [0] * 215, 'tpierce': 8})
test('move_ball: Durchschlag prallt an unzerstoerbaren Steinen ab', {'mem': s3}, 'move_ball', ['[MAP] == 11', '[vxh] == 0'])
s4 = {'bxl': 0, 'bxh': 0x40, 'byl': 0, 'byh': 0x2f, 'vxl': 0, 'vxh': 0, 'vyl': 0x00, 'vyh': 0x02, 'padl': 16, 'padh': 0, 'padw': 32, 'zstep': 4}
test('move_ball: Ball unter dem Schlaeger geht verloren (A = 1)', {'mem': s4}, 'move_ball', ['a == 1'])
levels = []; cur = None
for line in open('levels/levels.txt', encoding='utf-8'):
    line = line.rstrip('\n')
    m = re.match(r'level (\d+) "(.*)"', line)
    if m: cur = {'name': m.group(2), 'rows': []}; levels.append(cur); continue
    if cur is not None and len(line) == 18: cur['rows'].append(line)
for i, lv in enumerate(levels):
    cnt = sum(1 for r in lv['rows'] for c in r if c not in '.#')
    cores = sum(1 for r in lv['rows'] for c in r if c == 'C')
    kind = 2 if lv['name'].startswith('Boss') else 1 if lv['name'].startswith('Bonus') else 0
    test('load_level %d (%s): %d Steine, %d Kerne, Art %d' % (i + 1, lv['name'], cnt, cores, kind), {'mem': {'level': i}}, 'load_level', ['[bricks] == %d' % cnt, '[cores] == %d' % cores, '[lkind] == %d' % kind])

# ---- Klangtabellen (POKEY): Teiler fuer bekannte Noten gegen die Formeln rechnen (PAL-Takt 1773447 Hz; 8 Bit: f = Takt/28 / (2 (n + 1)); 16 Bit: f = Takt / (2 (n + 7))); die Tabellen stehen in <atari8bit/notes.inc>, Index = Notenwert
def n8(idx): return max(1, min(255, int(round(1773447 / 28 / (2 * 16.3516 * 2 ** ((idx - 1) / 12.0)))) - 1))
def n16(idx): return max(8, min(65535, int(round(1773447 / (2 * 16.3516 * 2 ** ((idx - 1) / 12.0)))) - 7))
for nm, idx in (('D4', 51), ('A4', 58), ('D5', 63)):
    test('Klangtabelle 8 Bit: %s (Index %d)' % (nm, idx), {}, 'input_init', ['[note_pokey8 + %d] == %d' % (idx, n8(idx))])
for nm, idx in (('D2', 27), ('A2', 34)):
    v = n16(idx)
    test('Klangtabelle 16 Bit: %s (Index %d)' % (nm, idx), {}, 'input_init', ['[note_pokey16_lo + %d] == %d' % (idx, v & 255), '[note_pokey16_hi + %d] == %d' % (idx, v >> 8)])
test('name_clear: Namenspuffer mit ASCII-Leerzeichen, Cursor 0, getippt-Merker', {'mem': {'hs_name': [0x45, 0x58, 0x59, 0x5a, 0x41, 0x42, 0x43, 0x44], 'npos': 5, 'ntyped': 0}}, 'name_clear', ['[hs_name] == $20', '[hs_name + 3] == $20', '[hs_name + 7] == $20', '[npos] == 0', '[ntyped] == 1'])

# ---- Tastatur: Atari-Tastencode im OS-Puffer CH ($02FC) -> ASCII; CH wird danach auf $FF geloescht
for code, ch, nm in ((0x3a, ord('D'), 'D'), (0x3f, ord('A'), 'A'), (0x17, ord('Z'), 'Z'), (0x16, ord('X'), 'X'), (0x2f, ord('Q'), 'Q'), (0x2a, ord('E'), 'E'), (0x21, 32, 'Leertaste'), (0x0c, 13, 'RETURN'), (0x34, 20, 'BACKSPACE'), (0x1f, ord('1'), '1'), (0x32, ord('0'), '0'), (0x18, ord('4'), '4')):
    test('input_char: Taste %s (Code $%02x) -> %d' % (nm, code, ch), {'mem': {'$02fc': code, 'in_ckey': 0xff}}, 'input_char', ['a == %d' % ch, '[$02fc] == $ff', '[in_ckey] == %d' % (code & 0x3f)])
test('input_char: Code mit Shift-Bit ($40) wird wie ohne Shift gelesen (D)', {'mem': {'$02fc': 0x7a, 'in_ckey': 0xff}}, 'input_char', ['a == %d' % ord('D')])
test('input_char: dieselbe Taste wurde schon direkt gemeldet -> nicht doppelt', {'mem': {'$02fc': 0x3a, 'in_ckey': 0x3a}}, 'input_char', ['a == 0', '[$02fc] == $ff'])
test('input_char: Puffer leer ($FF) und keine Taste gedrueckt -> 0', {'mem': {'$02fc': 0xff}}, 'input_char', ['a == 0'])
test('hs_default: Standardliste mit SPIELER und 1000..100 Punkten', {}, 'hs_default', ['[hs_buf] == $50', '[hs_buf + 3] == $4c', '[hs_buf + 4] == 1', '[hs_buf + 5] == $53', '[hs_buf + 14] == $10', '[hs_buf + 5 + 12 * 9 + 9] == $01'])
crc = 0xffff
for _ in range(125):
    for _ in range(8):
        crc = ((crc << 1) ^ 0x1021) & 0xffff if crc & 0x8000 else (crc << 1) & 0xffff
test('hs_makecrc: CRC-16/CCITT ueber 125 Nullbytes', {}, 'hs_makecrc', ['[hs_buf + 125] == %s' % h(crc & 255), '[hs_buf + 126] == %s' % h(crc >> 8)])
test('hs_check: 1 Punkt gegen leere Liste -> Platz 0', {'mem': {'hs_points': 1}}, 'hs_check', ['a == 0'])
test('hs_check: 0 Punkte -> nicht in der Liste ($FF)', {'mem': {'hs_points': 0}}, 'hs_check', ['a == $ff'])
test('hs_insert: Name und Punkte an Platz 0', {'mem': {'hs_points': [0x50, 0x12, 0x00], 'hs_level': 7, 'hs_name': [0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48]}, 'a': '0'}, 'hs_insert', ['[hs_buf + 5] == $41', '[hs_buf + 12] == $48', '[hs_buf + 13] == $50', '[hs_buf + 14] == $12', '[hs_buf + 16] == 7'])
doc = {'kind': 'test', 'target': 'atari8', 'program': 'build/prallball.xex',
       'static': [{'check': 'overlap'},
                  {'check': 'expr', 'name': 'Programm endet vor dem Bildschirmspeicher bei $8000', 'expr': 'song_patterns + 16 * 80 < $8000'},
                  {'check': 'cycles', 'label': 'add_score', 'max': 60}],
       'tests': tests}
json.dump(doc, open('prallball.vctest', 'w'), indent=1, ensure_ascii=False)
print(len(tests), 'Tests geschrieben')
