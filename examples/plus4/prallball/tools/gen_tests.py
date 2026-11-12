#!/usr/bin/env python3
"""Erzeugt prallball.vctest (statische Pruefungen und Emulator-Tests im Trap-Modus fuer `vcc test --backend vice`).
Erwartungswerte (Richtungstabelle, Zufall, CRC, Levelzaehlungen) werden hier unabhaengig vom Programm berechnet.
Aufruf im Ordner examples/plus4/prallball:  python3 tools/gen_tests.py   (vorher: vcc build . - das Skript liest build/prallball.sym.txt fuer
Adressen wie song)
Abschnitte: add_score (BCD), Zufall (LFSR), set_vel, paddle_hit (Zonen, Bewegungseinfluss, Seitenaufprall), probe/move_ball (Karte, Durchschlag),
load_level (Zaehlung aller 24 Levels gegen levels/levels.txt), Ticker-Texte, Bestenliste, Plus/4-Hilfen (put_cell, draw_brick, clear_screen, hs_device),
Compositor tsp_render (Bitmuster unabhaengig aus den Bildern berechnet) sowie Klang und Musik (Register des TED).
Die Ausgabe prallball.vctest wird bei jedem Lauf vollstaendig neu geschrieben.  Der Trap-Modus ruft die Routinen einzeln mit vorbesetzten Registern
und Speicherzellen auf (setup) und prueft danach Ausdruecke (expect); die Falle (trap) liegt bei $7F00."""
import json, math, os, re
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/plus4/prallball (os.chdir); alle Pfade gelten relativ dazu, der Aufruf funktioniert daher aus jedem Ordner.', epilog='Beispiele:\n  python3 examples/plus4/prallball/tools/gen_tests.py\n  cd examples/plus4/prallball && python3 tools/gen_tests.py\n\nSchreibt (im Projektordner examples/plus4/prallball, vorhandene Dateien werden ueberschrieben): prallball.vctest.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))

# ---- Zeropage-Adressen (wie in prallball.asm)
ZP = dict(bxl=0x10, bxh=0x11, byl=0x12, byh=0x13, vxl=0x14, vxh=0x15, vyl=0x16, vyh=0x17, padl=0x18, padh=0x19, speed=0x1a, hits=0x1b,
          level=0x1d, bricks=0x1e, stuck=0x1f, sc0=0x20, sc1=0x21, sc2=0x22, bcol=0x28, brow=0x29, bidx=0x2a, padw=0x45, zstep=0x46,
          capact=0x48, tpierce=0x88, mask=0x23, oyl=0x39, oyh=0x3a, spdmod=0x50, sticky=0x52, cores=0x6f, lkind=0x6c, cb=0x5a)
def h(v): return '$%02x' % v

# ---- Richtungstabelle neu berechnen (gleiche Formel wie tools/gen.py, aber unabhaengig implementiert)
ANG = [-60, -43, -26, -9, 9, 26, 43, 60]
def vel(level, zone):
    speed = (1.6 + 0.1 * level) * 64
    a = math.radians(ANG[zone])
    dx = int(round(speed * math.sin(a))); dy = int(round(-speed * math.cos(a)))
    return dx & 0xffff, dy & 0xffff

tests = []
def test(name, setup, call, expect, ret=None, cycles=None):
    # Plus/4: nach dem Hochfahren ist das ROM eingeblendet (Lesen von $8000-$FFFF liefert ROM); ein Schreiben nach $FF3F blendet das RAM ein
    # (wie das Spiel es tut).  Zuerst setzen, damit die uebrigen Schreibzugriffe und Pruefungen das RAM sehen.
    mem = {'$ff3f': 0}; mem.update(setup.get('mem', {})); setup = dict(setup); setup['mem'] = mem
    t = {'name': name, 'setup': setup, 'call': call, 'expect': expect}
    if ret: t['return'] = ret
    # (Zyklenbudgets entfallen: VICE xplus4 misst keine Zyklen, der Test wuerde nur uebersprungen; die statische Pruefung unten bleibt)
    tests.append(t)

# ---- add_score (BCD)
test('add_score: 0999 + 10 = 1009', {'mem': {h(ZP['sc0']): 0x99, h(ZP['sc1']): 0x09, h(ZP['sc2']): 0}, 'a': '$10'}, 'add_score',
     ['[$20] == $09', '[$21] == $10', '[$22] == 0'], cycles=60)
test('add_score: Uebertrag in die hoechsten Ziffern', {'mem': {h(ZP['sc0']): 0x95, h(ZP['sc1']): 0x99, h(ZP['sc2']): 0}, 'a': '$10'}, 'add_score',
     ['[$20] == $05', '[$21] == $00', '[$22] == $01'])

# ---- Zufall: Galois-LFSR nachrechnen
def lfsr(lo, hi):
    c = hi & 1; hi >>= 1
    c2 = lo & 1; lo = (lo >> 1) | (c << 7)
    if c2: hi ^= 0xb4
    return lo, hi, lo ^ hi
lo, hi = 0xa5, 0x5a
for n in range(1, 4):
    lo, hi, a = lfsr(lo, hi)
test('rand_byte: erste Zahl aus Startwert $A5/$5A', {'mem': {'$02': 0xa5, '$03': 0x5a}}, 'rand_byte',
     ['a == %s' % h(lfsr(0xa5, 0x5a)[2]), '[$02] == %s' % h(lfsr(0xa5, 0x5a)[0]), '[$03] == %s' % h(lfsr(0xa5, 0x5a)[1])], cycles=40)

# ---- set_vel
for lvl, zone in ((0, 3), (5, 0), (15, 7)):
    dx, dy = vel(lvl, zone)
    test('set_vel: Stufe %d Zone %d' % (lvl, zone), {'mem': {h(ZP['speed']): lvl, h(ZP['spdmod']): 0}, 'a': str(zone)}, 'set_vel',
         ['[$14] == %s' % h(dx & 255), '[$15] == %s' % h(dx >> 8), '[$16] == %s' % h(dy & 255), '[$17] == %s' % h(dy >> 8)], cycles=70)
dx, dy = vel(15, 2)
test('set_vel: "schneller" begrenzt auf Stufe 15', {'mem': {h(ZP['speed']): 14, h(ZP['spdmod']): 1}, 'a': '2'}, 'set_vel',
     ['[$14] == %s' % h(dx & 255), '[$16] == %s' % h(dy & 255)])
dx, dy = vel(0, 4)
test('set_vel: "langsamer" begrenzt auf Stufe 0', {'mem': {h(ZP['speed']): 2, h(ZP['spdmod']): 2}, 'a': '4'}, 'set_vel',
     ['[$14] == %s' % h(dx & 255), '[$16] == %s' % h(dy & 255)])

# ---- paddle_hit: Schlaeger bei x = 100..131, Ball 6 px breit; Ballkante x = 116 -> d = 16 -> Zone (16 + 3) / 4 = 4
base = {h(ZP['padl']): 100, h(ZP['padh']): 0, h(ZP['padw']): 32, h(ZP['zstep']): 4, h(ZP['speed']): 0, h(ZP['spdmod']): 0, h(ZP['hits']): 0,
        h(ZP['sticky']): 0, h(ZP['cb']): 0, h(ZP['byl']): 0x80, h(ZP['byh']): 0x2c, h(ZP['vyh']): 0, h(ZP['vyl']): 0x40}
dx, dy = vel(0, 4)
s = dict(base); s.update({h(ZP['bxl']): 0x00, h(ZP['bxh']): 0x1d})          # 116 * 64 = $1D00
test('paddle_hit: Treffer in der Mitte -> Zone 4 und Abprall nach oben', {'mem': s}, 'paddle_hit',
     ['fc', '[$14] == %s' % h(dx & 255), '[$16] == %s' % h(dy & 255), '[$17] == %s' % h(dy >> 8), '[$1b] == 1'])
s = dict(base); s.update({h(ZP['bxl']): 0x00, h(ZP['bxh']): 0x32})          # 200 * 64: weit rechts vom Schlaeger
test('paddle_hit: Ball neben dem Schlaeger -> kein Abprall', {'mem': s}, 'paddle_hit', ['!fc', '[$1b] == 0'])
dx, dy = vel(0, 0)
s = dict(base); s.update({h(ZP['bxl']): 0x00, h(ZP['bxh']): 0x19})          # 100 * 64: linke Kante -> Zone 0
test('paddle_hit: linke Schlaegerkante -> Zone 0 (-60 Grad)', {'mem': s}, 'paddle_hit', ['fc', '[$14] == %s' % h(dx & 255), '[$15] == %s' % h(dx >> 8)])


# ---- Einfluss der Schlaegerbewegung auf den Abprallwinkel (Mitte = Zone 4; links bewegt -> 3, rechts -> 5, an den Zonengrenzen begrenzt)
for mk, zone, nm in ((0x04, 3, 'nach links'), (0x08, 5, 'nach rechts'), (0x00, 4, 'stillstehend')):
    dx, dy = vel(0, zone)
    s = dict(base); s.update({h(ZP['bxl']): 0x00, h(ZP['bxh']): 0x1d, h(ZP['mask']): mk})
    test('paddle_hit: Schlaeger %s bewegt -> Zone %d' % (nm, zone), {'mem': s}, 'paddle_hit', ['fc', '[$14] == %s' % h(dx & 255), '[$15] == %s' % h(dx >> 8)])
dx, dy = vel(0, 0)
s = dict(base); s.update({h(ZP['bxl']): 0x00, h(ZP['bxh']): 0x19, h(ZP['mask']): 0x04})
test('paddle_hit: Bewegung nach links an Zone 0 bleibt Zone 0', {'mem': s}, 'paddle_hit', ['fc', '[$14] == %s' % h(dx & 255)])
# ---- Seitenaufprall: Ball war schon auf Schlaegerhoehe (alte Y >= 178 px): kein Abprall oben, sondern zur Seite
s = dict(base); s.update({h(ZP['bxl']): 0x40, h(ZP['bxh']): 0x18, h(ZP['oyl']): 0x90, h(ZP['oyh']): 0x2c, h(ZP['vxl']): 0x40, h(ZP['vxh']): 0x00, h(ZP['mask']): 0})   # Ballkante 97 px: links vom Schlaeger
test('paddle_hit: Seitenaufprall links -> weg nach links, kein Schlaegerabprall', {'mem': s}, 'paddle_hit',
     ['!fc', '[$14] == $c0', '[$15] == $ff', '[$10] == $80', '[$11] == $17', '[$1b] == 0'])
s = dict(base); s.update({h(ZP['bxl']): 0x80, h(ZP['bxh']): 0x20, h(ZP['oyl']): 0x90, h(ZP['oyh']): 0x2c, h(ZP['vxl']): 0xc0, h(ZP['vxh']): 0xff, h(ZP['mask']): 0})  # Ballkante 130 px: rechts
test('paddle_hit: Seitenaufprall rechts -> weg nach rechts', {'mem': s}, 'paddle_hit',
     ['!fc', '[$14] == $40', '[$15] == 0', '[$10] == 0', '[$11] == $21', '[$1b] == 0'])

# ---- probe / move_ball mit der Steinkarte ($C000)
mapb = {'$c000': [1] + [0] * 215}                      # Steinkarte: ein Stein, sonst leer (der Speicher ist sonst beliebig belegt)
s = {h(ZP['bxh']): 4, h(ZP['bxl']): 0, h(ZP['byh']): 8, h(ZP['byl']): 0}
s.update(mapb)
test('probe: Stein oben links wird gefunden', {'mem': s, 'x': 0, 'y': 0}, 'probe',
     ['a == 1', '[$28] == 0', '[$29] == 0', '[$2a] == 0'])
s = {h(ZP['bxh']): 3, h(ZP['bxl']): 0, h(ZP['byh']): 8, h(ZP['byl']): 0}; s.update(mapb)
test('probe: links vom Steinfeld kein Stein', {'mem': s, 'x': 0, 'y': 0}, 'probe', ['a == 0', 'fz'])
# Ball bewegt sich nach links in den Stein (Spalte 0, Zeile 0): px 32, vx = -1 px, vy klein positiv
s = {h(ZP['bxl']): 0x20, h(ZP['bxh']): 0x08, h(ZP['byl']): 0x00, h(ZP['byh']): 0x08, h(ZP['vxl']): 0xc0, h(ZP['vxh']): 0xff, h(ZP['vyl']): 0x10, h(ZP['vyh']): 0,
     h(ZP['bricks']): 1, h(ZP['sc0']): 0, h(ZP['sc1']): 0, h(ZP['sc2']): 0, h(ZP['lkind']): 0, h(ZP['capact']): 1}
s.update(mapb)
test('move_ball: Treffer links auf einen Stein: Stein weg, Punkte, Ball kehrt um', {'mem': s}, 'move_ball',
     ['[$c000] == 0', '[$1e] == 0', '[$20] == $10', '[$15] == 0', '([$14] | [$15]) != 0'])
# Ball faellt unten heraus
s = {h(ZP['bxl']): 0, h(ZP['bxh']): 0x40, h(ZP['byl']): 0, h(ZP['byh']): 0x2f, h(ZP['vxl']): 0, h(ZP['vxh']): 0, h(ZP['vyl']): 0x00, h(ZP['vyh']): 0x02,
     h(ZP['padl']): 16, h(ZP['padh']): 0, h(ZP['padw']): 32, h(ZP['zstep']): 4}
test('move_ball: Ball unter dem Schlaeger geht verloren (A = 1)', {'mem': s}, 'move_ball', ['a == 1'])


# ---- Durchschlag: Ball zerstoert Silber in einem Treffer und prallt nicht ab; # prallt weiter ab
s = {h(ZP['bxl']): 0x20, h(ZP['bxh']): 0x08, h(ZP['byl']): 0x00, h(ZP['byh']): 0x08, h(ZP['vxl']): 0xc0, h(ZP['vxh']): 0xff, h(ZP['vyl']): 0x10, h(ZP['vyh']): 0,
     h(ZP['bricks']): 1, h(ZP['sc0']): 0, h(ZP['sc1']): 0, h(ZP['sc2']): 0, h(ZP['lkind']): 0, h(ZP['capact']): 1, h(ZP['tpierce']): 8}
s.update({'$c000': [9] + [0] * 215})            # Silberstein (normal 2 Treffer)
test('move_ball: Durchschlag zerstoert Silber sofort und prallt nicht ab', {'mem': s}, 'move_ball', ['[$c000] == 0', '[$1e] == 0', '[$15] == $ff'])
s2 = dict(s); s2.update({'$c000': [11] + [0] * 215, h(ZP['bricks']): 1})
test('move_ball: Durchschlag prallt an unzerstoerbaren Steinen trotzdem ab', {'mem': s2}, 'move_ball', ['[$c000] == 11', '[$15] == 0'])
s3 = dict(s); s3.update({h(ZP['tpierce']): 0})
test('move_ball: ohne Durchschlag wird Silber nur beschaedigt (wird zu Typ 1) und der Ball prallt ab', {'mem': s3}, 'move_ball', ['[$c000] == 1', '[$15] == 0'])

# ---- Levels: Steinzaehlung, Art, Kerne gegen levels/levels.txt
levels = []; cur = None
for line in open('levels/levels.txt', encoding='utf-8'):
    line = line.rstrip('\n')
    m = re.match(r'level (\d+) "(.*)"', line)
    if m: cur = {'name': m.group(2), 'rows': []}; levels.append(cur); continue
    if cur is not None and len(line) == 18 and not line.startswith('#  '): cur['rows'].append(line)
TYPE = {'.': 0, '#': 11, 'C': 23}
for i, lv in enumerate(levels):
    cnt = sum(1 for r in lv['rows'] for c in r if c not in '.#')
    cores = sum(1 for r in lv['rows'] for c in r if c == 'C')
    kind = 2 if lv['name'].startswith('Boss') else 1 if lv['name'].startswith('Bonus') else 0
    exp = ['[$1e] == %d' % cnt, '[$6f] == %d' % cores, '[$6c] == %d' % kind]
    test('load_level %d (%s): %d Steine, %d Kerne, Art %d' % (i + 1, lv['name'], cnt, cores, kind), {'mem': {h(ZP['level']): i}}, 'load_level', exp)
    assert cnt > 0


# ---- Ticker: Texte laden (Bildschirmcodes: p = 16, * = 42, Leerzeichen = 32)
test('tk_loadnew: Text 0 beginnt mit einem Leerzeichen, dann "p" (16)', {'a': '0'}, 'tk_loadnew', ['[$c160] == 32', '[$c161] == 16', '[$c17e] == 32', '[$c17f] == 32'])
test('tk_loadnew: Text 2 enthaelt "***" in der Mitte', {'a': '2'}, 'tk_loadnew', ['[$c167] == 42', '[$c168] == 42', '[$c169] == 42', '[$c166] == 32'])

# ---- Bestenliste
test('hs_default: Standardliste mit SPIELER und 1000..100 Punkten', {}, 'hs_default',
     ['[hs_buf] == $50', '[hs_buf + 3] == $4c', '[hs_buf + 4] == 1', '[hs_buf + 5] == $53', '[hs_buf + 14] == $10', '[hs_buf + 13] == 0', '[hs_buf + 5 + 12 * 9 + 9] == $01'])
crc = 0xffff
for _ in range(125):
    for _ in range(8):
        crc = ((crc << 1) ^ 0x1021) & 0xffff if crc & 0x8000 else (crc << 1) & 0xffff
test('hs_makecrc: CRC-16/CCITT ueber 125 Nullbytes', {}, 'hs_makecrc', ['[hs_buf + 125] == %s' % h(crc & 255), '[hs_buf + 126] == %s' % h(crc >> 8)])
test('hs_check: 1 Punkt gegen leere Liste -> Platz 0', {'mem': {'hs_points': 1}}, 'hs_check', ['a == 0'])
test('hs_check: 0 Punkte -> nicht in der Liste ($FF)', {'mem': {'hs_points': 0}}, 'hs_check', ['a == $ff'])
test('hs_insert: Name und Punkte an Platz 0', {'mem': {'hs_points': [0x50, 0x12, 0x00], 'hs_level': 7, 'hs_name': [0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48]}, 'a': '0'}, 'hs_insert',
     ['[hs_buf + 5] == $41', '[hs_buf + 12] == $48', '[hs_buf + 13] == $50', '[hs_buf + 14] == $12', '[hs_buf + 16] == 7'])


# ---- Plus/4: Hardware-Hilfen ------------------------------------------------------------------------------------------------
# Umsetzung der Farben und Zellen
test('put_cell: Zeichen A, weiss: Bildschirm, Schattenkopie, Farb-RAM und Schattenfarbe', {'mem': {'$33': 1}, 'a': '1', 'x': 3, 'y': 2}, 'put_cell',
     ['[$0c00 + 2 * 40 + 3] == 1', '[$c400 + 2 * 40 + 3] == 1', '[$0800 + 2 * 40 + 3] == $71', '[$c800 + 2 * 40 + 3] == $71'])
test('put_cell: Mehrfarbfarbe 9 (weiss) wird zum TED-Wert $79', {'mem': {'$33': 9}, 'a': '64', 'x': 0, 'y': 24}, 'put_cell',
     ['[$0c00 + 24 * 40] == 64', '[$0800 + 24 * 40] == $79', '[$c800 + 24 * 40] == $79'])
test('draw_brick: Typ 3 an Spalte 1, Zeile 0 -> zwei Zeichen in Bildschirmzeile 4, Spalte 4/5 (Farbe gruen = $5d)', {'mem': {'$28': 1, '$29': 0}, 'a': '3'}, 'draw_brick',
     ['[$0c00 + 4 * 40 + 4] == 64', '[$0c00 + 4 * 40 + 5] == 65', '[$0800 + 4 * 40 + 4] == $5d', '[$0800 + 4 * 40 + 5] == $5d', '[$c400 + 4 * 40 + 4] == 64', '[$c800 + 4 * 40 + 5] == $5d'])
test('draw_brick: Typ 0 loescht beide Zeichen (auch in der Schattenkopie)', {'mem': {'$28': 1, '$29': 0, '$%x' % (0x0c00 + 4 * 40 + 4): [64, 65], '$%x' % (0xc400 + 4 * 40 + 4): [64, 65]}, 'a': '0'}, 'draw_brick',
     ['[$0c00 + 4 * 40 + 4] == 32', '[$0c00 + 4 * 40 + 5] == 32', '[$c400 + 4 * 40 + 4] == 32', '[$c400 + 4 * 40 + 5] == 32'])
test('clear_screen: Bildschirm und Schattenkopie leer, weiss; keine Pool-Zellen', {'mem': {'$ccc0': 7}}, 'clear_screen',
     ['[$0c00] == 32', '[$0fe7] == 32', '[$c400] == 32', '[$c7e7] == 32', '[$0800] == $71', '[$0be7] == $71', '[$c800] == $71', '[$cbe7] == $71', '[$ccc0] == 0'])

# Bootgeraet fuer die Bestenliste
for dev, want in ((8, 8), (9, 9), (11, 11), (0, 8), (7, 8), (16, 8), (30, 8)):
    test('hs_device: KERNAL-Geraet %d -> %d' % (dev, want), {'mem': {'$ae': dev}}, 'hs_device', ['x == %d' % want])

# ---- Zeichen-Sprites (Compositor), Erwartungen unabhaengig aus den Bildern berechnet (tools/gen.py: Streifen entstehen dort, hier pixelweise)
def mp(v): return {0: 0, 1: 1, 2: 3, 3: 2}[v]
BALL = [[mp(v) for v in r] for r in [[0,2,0],[2,1,2],[2,2,3],[2,2,3],[2,3,3],[0,3,0]]]
def padrows(left_round, right_round, width):
    rows = []
    for r in range(6):
        base = {0: 1, 1: 1, 2: 2, 3: 2, 4: 3, 5: 3}[r]
        row = [base] * width
        if r == 1: row[0] = 1 if left_round else base
        if left_round and r in (0, 5): row[0] = 0
        if right_round and r in (0, 5): row[-1] = 0
        rows.append([mp(v) for v in row])
    return rows
PAD16 = padrows(True, True, 8)
def paint(layers):
    """layers: Liste (rows, x, y) von unten nach oben.  Ergebnis: {(zeile, spalte): [(daten, maske)] * 8} nur fuer Zellen mit Pixeln, in Zeichenreihenfolge."""
    cells = {}
    for rows, x, y in layers:
        for r, row in enumerate(rows):
            for c, p in enumerate(row):
                gx, gy = x + c, y + r
                key = (gy // 8, gx // 4)
                cells.setdefault(key, [[0, 0] for _ in range(8)])
    out = {}
    for key in sorted(cells):
        lines = [[0, 0] for _ in range(8)]
        out[key] = lines
    for rows, x, y in layers:
        for r, row in enumerate(rows):
            for c, p in enumerate(row):
                if not p: continue
                gx, gy = x + c, y + r
                ln = out[(gy // 8, gx // 4)][gy % 8]
                sh = 6 - 2 * (gx % 4)
                ln[0] = (ln[0] & ~(3 << sh) & 255) | (p << sh)
                ln[1] |= 3 << sh
    return out
SL = dict(act=0xcc00, x=0xcc08, y=0xcc10, img=0xcc18, col=0xcc20, dl_lo=0xcc40, dl_hi=0xcc80, pool=0xccc0)
def slot(mem, n, x, y, img, col):
    mem['$%x' % (SL['act'] + n)] = 1; mem['$%x' % (SL['x'] + n)] = x; mem['$%x' % (SL['y'] + n)] = y
    mem['$%x' % (SL['img'] + n)] = img; mem['$%x' % (SL['col'] + n)] = col
def comp_setup(base_cells=None):
    """Leerer Zustand: alle Schlitze aus, keine Pool-Zellen, Zeichen 32 leer; Schattenkopie ueberall Leerzeichen / weiss (Hires)."""
    mem = {}
    for n in range(7): mem['$%x' % (SL['act'] + n)] = 0
    mem['$%x' % SL['pool']] = 0
    mem['$f100'] = [0] * 8                       # Zeichen 32 (Leerzeichen) im Zeichensatz
    mem['$0c00'] = [32] * 1000                   # Bildschirm leer (Reste von Pool-Codes aus frueheren Tests wuerden als "schon belegt" gelten)
    return mem
def expect_cells(cells, base=None, colour=None, order=None):
    """Erwartungen fuer die Pool-Zeichen 128 + n in der Reihenfolge der Zellen (Zeile, Spalte)."""
    exp = []
    for n, key in enumerate(order or sorted(cells)):
        row, col = key
        exp.append('[$0c00 + %d * 40 + %d] == %d' % (row, col, 128 + n))
        if colour is not None: exp.append('[$0800 + %d * 40 + %d] == %s' % (row, col, h(colour)))
        for ln in range(8):
            d, m = cells[key][ln]
            b = (base or {}).get((key, ln), 0)
            exp.append('[$f400 + %d * 8 + %d] == %s' % (n, ln, h((b & ~m & 255) | d)))
    exp.append('[$ccc0] == %d' % len(order or cells))
    return exp
m = comp_setup(); slot(m, 0, 10, 100, 0, 0x79)
cells = paint([(BALL, 10, 100)])
test('tsp_render: Ball bei x=10 (Feinlage 2), y=100 (senkrecht 4): vier Pool-Zeichen mit dem richtigen Bitmuster', {'mem': m}, 'tsp_render', expect_cells(cells, colour=0x79))
for x, y in ((0, 16), (3, 33), (39, 190), (156, 8)):
    m = comp_setup(); slot(m, 0, x, y, 0, 0x7f)
    cells = paint([(BALL, x, y)])
    cells = {k: v for k, v in cells.items() if k[1] < 40 and k[0] < 24}
    test('tsp_render: Ball bei x=%d y=%d' % (x, y), {'mem': m}, 'tsp_render', expect_cells(cells, colour=0x7f))
m = comp_setup(); slot(m, 3, 5, 184, 1, 0x6e)
cells = paint([(PAD16, 5, 184)])
test('tsp_render: Schlaeger (16 px) bei x=5, Zeile 23', {'mem': m}, 'tsp_render', expect_cells(cells, colour=0x6e))
# Schlaeger und Ball teilen sich Zellen: der Ball liegt oben auf, die Zellfarbe ist die des zuletzt gezeichneten (Ball)
m = comp_setup(); slot(m, 3, 20, 184, 1, 0x6e); slot(m, 0, 22, 182, 0, 0x79)
cells = paint([(PAD16, 20, 184), (BALL, 22, 182)])
order = sorted(cells, key=lambda k: k)
# Reihenfolge der Vergabe: erst der Schlaeger (Zeile 23, Spalten 5, 6), dann der Ball: Zeile 22 (Spalten 5, 6); Zeile 23 gehoert schon dem Schlaeger
order = [(23, 5), (23, 6), (22, 5), (22, 6)]
test('tsp_render: Schlaeger und Ball in gemeinsamen Zellen (Ball oben)', {'mem': m}, 'tsp_render', expect_cells(cells, order=order))
# Muenzen (Bilder aus 8 Pixel breitem Rahmen, nur eine Feinlage; Vorderseite mit Buchstaben)
import math
def coinrows(h):
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
    return [[mp(v) for v in r] for r in rows]
COIN_EDGE = coinrows(3)
FONT_B = 0x7c66667c66667c00
def coin_b():
    rows = [[{0: 0, 1: 1, 2: 3, 3: 2}[v] for v in r] for r in [[0] * 8 for _ in range(12)]]
    raw = coinrows(12)                          # schon abgebildet; fuer den Buchstaben die Muenzenfarbe (3) durch Buchstabenpixel (2) ersetzen
    for gr in range(8):
        b = (FONT_B >> (8 * (7 - gr))) & 255
        for k in range(4):
            two = (b >> (6 - 2 * k)) & 3
            raw[2 + gr][2 + k] = 2 if two else 3
    return raw
m = comp_setup(); slot(m, 4, 8, 100, 5, 0x5f)          # IMG_COIN_EDGE
test('tsp_render: Muenze (Kante) bei x=8, y=100: Zeichenspalten 2 und 3', {'mem': m}, 'tsp_render', expect_cells(paint([(COIN_EDGE, 8, 100)]), colour=0x5f))
m = comp_setup(); slot(m, 4, 8, 100, 8, 0x6e)          # IMG_COIN_B (Vorderseite mit "B")
test('tsp_render: Muenze (Vorderseite B) bei x=8, y=100 mit Buchstaben', {'mem': m}, 'tsp_render', expect_cells(paint([(coin_b(), 8, 100)]), colour=0x6e))
m = comp_setup(); slot(m, 5, 77, 40, 4, 0x7f)           # IMG_SHOT: ein Mehrfarbpixel breit, 8 Zeilen
test('tsp_render: Laserschuss bei x=77, y=40', {'mem': m}, 'tsp_render', expect_cells(paint([([[3]] * 8, 77, 40)]), colour=0x7f))
# Hintergrund: ein Mehrfarbzeichen (Stein links, Code 64) unter dem Sprite bleibt ausserhalb des Sprites erhalten
bg = [0x15, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x2a, 0x00]
m = comp_setup(); slot(m, 0, 10, 100, 0, 0x79)
m['$%x' % (0xc400 + 12 * 40 + 2)] = 64; m['$%x' % (0xc800 + 12 * 40 + 2)] = 0x5a
m['$f200'] = bg
cells = paint([(BALL, 10, 100)])
test('tsp_render: Mehrfarb-Hintergrundzeichen bleibt neben dem Sprite sichtbar (Hintergrund UND Maske ODER Bild)', {'mem': m}, 'tsp_render',
     expect_cells(cells, base={((12, 2), ln): bg[ln] for ln in range(8)}, colour=None))
# Hires-Zelle (Text) unter dem Sprite: als Hintergrund gilt ein leeres Zeichen
m = comp_setup(); slot(m, 0, 10, 100, 0, 0x79)
m['$%x' % (0xc400 + 12 * 40 + 2)] = 1; m['$%x' % (0xc800 + 12 * 40 + 2)] = 0x71
m['$f008'] = [0xff] * 8
test('tsp_render: Text (Hires-Zelle) unter dem Sprite zaehlt als leer', {'mem': m}, 'tsp_render', expect_cells(paint([(BALL, 10, 100)]), colour=0x79))
# Zeile 24 und tiefer: keine Sprites
m = comp_setup(); slot(m, 0, 10, 192, 0, 0x79)
test('tsp_render: Sprite in Zeile 24 wird nicht gezeichnet', {'mem': m}, 'tsp_render', ['[$ccc0] == 0', '[$0c00 + 24 * 40 + 2] == 32', '[$0c00 + 24 * 40 + 3] == 32'])
# Wiederherstellen: eine Pool-Zelle des vorigen Bildes wird aus der Schattenkopie zurueckgesetzt
m = comp_setup()
m['$cc40'] = 0x0a; m['$cc80'] = 0x0d + 0           # Zelle bei $0D0A = Zeile 11, Spalte 28 (0xD0A - 0xC00 = 266 = 6 * 40 + 26)
m['$ccc0'] = 1
m['$0d0a'] = 128; m['$090a'] = 0x79
m['$%x' % (0xc400 + 266)] = 64; m['$%x' % (0xc800 + 266)] = 0x5a
test('tsp_render: Zellen des letzten Bildes werden aus der Schattenkopie wiederhergestellt', {'mem': m}, 'tsp_render',
     ['[$0d0a] == 64', '[$090a] == $5a', '[$ccc0] == 0'])
# ---- Klang: Effekte auf Stimme 2 des TED, Lautstaerke 8
for sid, n, rn in ((0, 717, 0), (1, 562, 0), (2, 800, 1), (4, 794, 0)):
    test('sfx_play: Effekt %d -> Tonhoehe N=%d, Stimme 2 an%s' % (sid, n, ', Rauschen' if rn else ''), {'mem': {'sfx_mute': 0, 'ted_vol': 0, 'sfx_t': 0}, 'a': str(sid)}, 'sfx_play',
         ['[$ff0f] == %s' % h(n & 255), '([$ff10] & 3) == %d' % (n >> 8), '([$ff11] & $20) == $20', '([$ff11] & $40) == %s' % ('$40' if rn else '0'), '([$ff11] & $0f) == 8'])
test('sfx_play: stumm im Demo-Modus (sfx_mute)', {'mem': {'$ff11': 0, 'sfx_mute': 1}, 'a': '0'}, 'sfx_play', ['([$ff11] & $20) == 0'])
test('sfx_update: Effekt endet, Stimme 2 geht aus', {'mem': {'sfx_t': 1, 'sfx_id': 0, 'ted_vol': 0x68}}, 'sfx_update', ['([$ff11] & $60) == 0', '[sfx_t] == 0'])
# Musik: Zeilenbeginn.  Takt 3 des Liedes (Muster 6, 7, 8): Melodie Note 56, Bass Note 25, Spur 3 (Arpeggio) entfaellt.
sym = {}
for line in open('build/prallball.sym.txt', encoding='utf-8'):
    p = line.split()
    if len(p) >= 2 and p[1].startswith('$') and not line.startswith(';'): sym[p[0]] = int(p[1][1:], 16)
def ted_n(note):
    f = 16.3516 * 2 ** ((note - 1) / 12.0)
    while f < 110.0: f *= 2
    return max(1, min(1022, round(1024 - 110840.5 / f)))
song = sym['song']
mm = {'$04': 0, '$05': 0, '$06': song & 255, '$07': song >> 8, 'mus_bar': 3, 'mus_row': 0, 'mus_tk': 0, 'mus_pend': 0, 'mus_dr': 0, 'mus_b_on': 0, 'sfx_t': 0, 'ted_vol': 0x70}
n1, n2 = ted_n(56), ted_n(25)
test('mus_tick: Zeilenbeginn Takt 3 -> Melodie N=%d (Stimme 1), Bass N=%d (Stimme 2), beide aus bis zum naechsten Bild' % (n1, n2), {'mem': mm}, 'mus_tick',
     ['[$ff0e] == %s' % h(n1 & 255), '([$ff12] & 3) == %d' % (n1 >> 8), '[$ff0f] == %s' % h(n2 & 255), '([$ff10] & 3) == %d' % (n2 >> 8),
      '([$ff11] & $30) == 0', '[mus_pend] == $30', '[mus_b_on] == 1', '[mus_tk] == 1'])
m2 = dict(mm); m2.update({'mus_tk': 1, 'mus_pend': 0x30, 'ted_vol': 0})
test('mus_tick: Bild 1 der Zeile schaltet die Stimmen wieder an', {'mem': m2}, 'mus_tick', ['([$ff11] & $30) == $30', '[mus_pend] == 0', '[mus_tk] == 2'])
# Schlagzeug (Takt 7: Spur 3 Muster 11 beginnt mit Schlag 0 = Bassdrum): Stimme 2 wird Rauschen
m3 = dict(mm); m3.update({'mus_bar': 7})
test('mus_tick: Takt 7 beginnt mit einer Bassdrum: Stimme 2 rauscht (Bass setzt aus)', {'mem': m3}, 'mus_tick', ['([$ff11] & $60) == $60', '[mus_dr] == 4'])
# Rauschstoss laeuft aus: Rauschen aus, Bass wieder an
m4 = {'mus_dr': 1, 'mus_b_on': 1, 'mus_b_lo': 0x10, 'mus_b_hi': 2, 'sfx_t': 0, 'ted_vol': 0x60, 'mus_tk': 3, '$04': 0, '$05': 0, '$06': song & 255, '$07': song >> 8, 'mus_bar': 3, 'mus_row': 0, 'mus_pend': 0}
test('mus_tick: Schlagzeugstoss zu Ende -> Rauschen aus, Bass wieder an', {'mem': m4}, 'mus_tick', ['([$ff11] & $40) == 0', '([$ff11] & $20) == $20', '[$ff0f] == $10', '([$ff10] & 3) == 2'])
test('mus_stop: alle Stimmen still', {'mem': {'ted_vol': 0x70}}, 'mus_stop', ['([$ff11] & $70) == 0'])
# Frequenztabelle: n = 1024 - 110840.5 / f; A4 = 440 Hz = Halbton 57 (C0 = 16,35 Hz)
f = 16.3516 * 2 ** (57 / 12.0); nn = round(1024 - 110840.5 / f)
test('Frequenztabelle: Halbton 57 (A4 = 440 Hz) -> N=%d' % nn, {}, 'tsp_hide', ['[note_ted_lo + 58] == %s' % h(nn & 255), '[note_ted_hi + 58] == %d' % (nn >> 8)])

doc = {'kind': 'test', 'target': 'plus4', 'program': 'build/prallball.prg', 'trap': '$7f00',
       'static': [
           {'check': 'overlap'},
           {'check': 'expr', 'name': 'Puffer der Bestenliste liegen unter $8000 (das KERNAL liest/schreibt sie bei eingeblendetem ROM)', 'expr': 'hs_tmp + 128 < $8000'},
           {'check': 'expr', 'name': 'Programm, Streifen, Levels und Lied enden vor den Puffern ab $C000', 'expr': 'song_patterns + 16 * 80 < $c000'},
           {'check': 'expr', 'name': 'Streifen der Zeichen-Sprites sind eingebunden', 'expr': 'imd_0 > $1001'},
           {'check': 'cycles', 'label': 'add_score', 'max': 60},
       ],
       'tests': tests}
json.dump(doc, open('prallball.vctest', 'w'), indent=1, ensure_ascii=False)
print(len(tests), 'Tests geschrieben')
