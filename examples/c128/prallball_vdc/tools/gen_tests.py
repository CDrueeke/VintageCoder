#!/usr/bin/env python3
"""Erzeugt prallball.vctest (statische Pruefungen und Emulator-Tests im Trap-Modus fuer `vcc test --backend vice`).
Erwartungswerte (Richtungstabelle, Zufall, CRC, Levelzaehlungen) werden hier unabhaengig vom Programm berechnet.
Aufruf im Ordner examples/c128/prallball_vdc:  python3 tools/gen_tests.py   (vorher: vcc build .)"""
import json, math, os, re
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/c128/prallball_vdc (os.chdir); alle Pfade gelten relativ dazu, der Aufruf funktioniert daher aus jedem Ordner.', epilog='Beispiele:\n  python3 examples/c128/prallball_vdc/tools/gen_tests.py\n  cd examples/c128/prallball_vdc && python3 tools/gen_tests.py\n\nSchreibt (im Projektordner examples/c128/prallball_vdc, vorhandene Dateien werden ueberschrieben): prallball.vctest.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
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
    # C128: nach dem Hochfahren sind die Systemroms eingeblendet; das Spiel arbeitet mit MMU-Konfiguration $3E (alles RAM plus E/A).  Ein Schreiben nach
    # $FF00 stellt sie ein; zuerst setzen, damit die uebrigen Schreibzugriffe und Pruefungen das RAM sehen.
    mem = {'$ff00': 0x3e}; mem.update(setup.get('mem', {})); setup = dict(setup); setup['mem'] = mem
    t = {'name': name, 'setup': setup, 'call': call, 'expect': expect}
    if ret: t['return'] = ret
    # (Zyklenbudgets entfallen: VICE x128 misst keine Zyklen; die statische Pruefung unten bleibt)
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
        h(ZP['sticky']): 0, h(ZP['cb']): 0, h(ZP['oyl']): 0, h(ZP['oyh']): 0x20, h(ZP['mask']): 0, h(ZP['byl']): 0x80, h(ZP['byh']): 0x2c, h(ZP['vyh']): 0, h(ZP['vyl']): 0x40}
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
test('tk_loadnew: Text 0 beginnt mit zwei Leerzeichen, dann "p" (16)', {'a': '0'}, 'tk_loadnew', ['[$c160] == 32', '[$c161] == 32', '[$c162] == 16', '[$c17f] == 32'])
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


# ---- C128 VDC: Hardware-Hilfen ------------------------------------------------------------------------------------------------
# Im Hauptspeicher: Spiegel SCR ($B000), Attributspiegel COLORRAM ($AC00), Schattenkopien SHC ($B400) und SHF ($B800), Zeichensatz CHARSET ($E000).
# Im VDC (per vdc_get lesbar; der VDC-Zustand bleibt zwischen den Tests erhalten): Bildschirm $0000 (2 Zellen je logische Zelle), Attribute $0800,
# Zeichensatz links $2000, rechts $3000.
for dev, want in ((8, 8), (9, 9), (11, 11), (0, 8), (7, 8), (16, 8), (30, 8)):
    test('hs_device: Startgeraet %d -> %d' % (dev, want), {'mem': {'hs_bootdev': dev}}, 'hs_device', ['x == %d' % want])
test('hs_init: merkt das KERNAL-Geraet aus $BA', {'mem': {'$ba': 9, 'hs_bootdev': 8}}, 'hs_init', ['[hs_bootdev] == 9'])
test('copy_font: Zeichen A (Bildschirmcode 1) aus dem Zeichen-ROM nach $E008; eigener Stein links bei $E200', {'mem': {'$e008': 0, '$e200': 0}}, 'copy_font',
     ['[$e008] == $18', '[$e009] == $3c', '[$e200] == $7f', '[$e207] == 0', '[$e000 + 127 * 8] == $ff'])
test('clear_screen: Spiegel und Schattenkopien leer (32), Attribute weiss (15)', {'mem': {'$b000': 5, '$b3e7': 5, '$b400': 5, '$ac00': 7, '$afe7': 7, '$b800': 7}}, 'clear_screen',
     ['[$b000] == 32', '[$b3e7] == 32', '[$b400] == 32', '[$b7e7] == 32', '[$ac00] == 15', '[$afe7] == 15', '[$b800] == 15', '[$bbe7] == 15', '[pool_n] == 0'])
test('VDC nach clear_screen: Bildschirm (Zelle 0) = 32', {'a': '0', 'y': '0'}, 'vdc_get', ['a == 32'])
test('VDC nach clear_screen: letzte Zelle des Bildschirms ($07CF) = 32', {'a': '7', 'y': '$cf'}, 'vdc_get', ['a == 32'])
test('VDC nach clear_screen: Attribut links (Zelle 0, $0800) = 15', {'a': '8', 'y': '0'}, 'vdc_get', ['a == 15'])
test('VDC nach clear_screen: Attribut rechts (Zelle 1, $0801) = 15 mit Bit 7', {'a': '8', 'y': '1'}, 'vdc_get', ['a == $8f'])
test('put_cell: Zeichen A (1) mit Farbe 7 (gelb) an Spalte 3, Zeile 2: Spiegel, Schattenkopie, Attribut 13', {'mem': {'$33': 7}, 'a': '1', 'x': 3, 'y': 2}, 'put_cell',
     ['[$b000 + 2 * 40 + 3] == 1', '[$b400 + 2 * 40 + 3] == 1', '[$ac00 + 2 * 40 + 3] == 13', '[$b800 + 2 * 40 + 3] == 13'])
# VDC-Adresse der Zelle (Zeile 2, Spalte 3): (2 * 40 + 3) * 2 = 166 = $A6
test('VDC nach put_cell: linke Zelle (Zeile 2, Spalte 3) = Zeichen 1', {'a': '0', 'y': '$a6'}, 'vdc_get', ['a == 1'])
test('VDC nach put_cell: rechte Zelle = Zeichen 1', {'a': '0', 'y': '$a7'}, 'vdc_get', ['a == 1'])
test('VDC nach put_cell: Attribut links = 13', {'a': '8', 'y': '$a6'}, 'vdc_get', ['a == 13'])
test('VDC nach put_cell: Attribut rechts = 13 mit Bit 7', {'a': '8', 'y': '$a7'}, 'vdc_get', ['a == $8d'])
test('draw_brick: Typ 3 an Spalte 1, Zeile 0 -> Zeichen 64/65 in Zeile 4, Spalte 4/5 (Attribut 5 = gruen)', {'mem': {'$28': 1, '$29': 0}, 'a': '3'}, 'draw_brick',
     ['[$b000 + 4 * 40 + 4] == 64', '[$b000 + 4 * 40 + 5] == 65', '[$ac00 + 4 * 40 + 4] == 5', '[$ac00 + 4 * 40 + 5] == 5', '[$b400 + 4 * 40 + 4] == 64', '[$b800 + 4 * 40 + 5] == 5'])
test('wr_attr: nur die Farbe der Zelle Spalte 3, Zeile 2 auf 9', {'a': '9', 'x': 3, 'y': 2}, 'wr_attr', ['[$ac00 + 2 * 40 + 3] == 9', '[$b800 + 2 * 40 + 3] == 9', '[$b000 + 2 * 40 + 3] == 1'])
test('VDC nach wr_attr: Attribut rechts = 9 mit Bit 7', {'a': '8', 'y': '$a7'}, 'vdc_get', ['a == $89'])
# Namenseingabe: Namensfeld unterstrichen (Attributbit 5), Cursorzelle in Umkehrschrift (Bit 6); VDC-Attribut der Zelle (Zeile 22, Spalte 16) = $0800 + (22 * 40 + 16) * 2 = $0F00
test('name_draw: Cursor auf Stelle 1 -> Spiegelattribute: Stelle 0 unterstrichen ($2F), Stelle 1 Umkehrschrift ($4F)', {'mem': {'$40': 1, 'hs_name': [0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48]}}, 'name_draw',
     ['[$ac00 + 22 * 40 + 16] == $2f', '[$ac00 + 22 * 40 + 17] == $4f', '[$ac00 + 22 * 40 + 18] == $2f'])
test('VDC nach name_draw: Stelle 0 links = unterstrichen, weiss ($2F)', {'a': '$0f', 'y': '$00'}, 'vdc_get', ['a == $2f'])
test('VDC nach name_draw: Stelle 0 rechts = $AF (zweiter Zeichensatz)', {'a': '$0f', 'y': '$01'}, 'vdc_get', ['a == $af'])
test('VDC nach name_draw: Stelle 1 links = Umkehrschrift, weiss ($4F)', {'a': '$0f', 'y': '$02'}, 'vdc_get', ['a == $4f'])
test('VDC nach name_draw: Stelle 1 rechts = $CF', {'a': '$0f', 'y': '$03'}, 'vdc_get', ['a == $cf'])

# ---- Zeichen-Sprites (Compositor): Erwartungen unabhaengig aus den Bildern berechnet (1 Bit je Pixel, Feinlage = X mod 8)
def dblrows(rows): return [[1 if v else 0 for v in r for _ in (0, 1)] for r in rows]
BALL = dblrows([[0,2,0],[2,1,2],[2,2,3],[2,2,3],[2,3,3],[0,3,0]])
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
PAD16 = dblrows(padrows(True, True, 8))
def paint(layers):
    """layers: Liste (rows, x, y) von unten nach oben.  Ergebnis: {(zeile, spalte): [(daten, maske)] * 8} nur fuer Zellen mit Pixeln."""
    out = {}
    for rows, x, y in layers:
        for r, row in enumerate(rows):
            for c, p in enumerate(row):
                gx, gy = x + c, y + r
                out.setdefault((gy // 8, gx // 8), [[0, 0] for _ in range(8)])
    for rows, x, y in layers:
        for r, row in enumerate(rows):
            for c, p in enumerate(row):
                if not p: continue
                gx, gy = x + c, y + r
                ln = out[(gy // 8, gx // 8)][gy % 8]
                bit = 7 - gx % 8
                ln[0] |= 1 << bit; ln[1] |= 1 << bit
    return out
SL = dict(act=0xcc00, x=0xcc08, xh=0xcc28, y=0xcc10, img=0xcc18, col=0xcc20, pool=0xccc0)
def slot(mem, n, x, y, img, col):
    mem['$%x' % (SL['act'] + n)] = 1; mem['$%x' % (SL['x'] + n)] = x & 255; mem['$%x' % (SL['xh'] + n)] = x >> 8
    mem['$%x' % (SL['y'] + n)] = y; mem['$%x' % (SL['img'] + n)] = img; mem['$%x' % (SL['col'] + n)] = col
def comp_setup():
    mem = {}
    for n in range(7): mem['$%x' % (SL['act'] + n)] = 0
    mem['$%x' % SL['pool']] = 0
    mem['$e100'] = [0] * 8                       # Zeichen 32 (Leerzeichen) im Zeichensatz
    mem['$b000'] = [32] * 1000                   # Spiegel leer (Reste von Pool-Codes aus frueheren Tests wuerden als "schon belegt" gelten)
    mem['$b400'] = [32] * 1000                   # Schattencodes: Leerzeichen
    return mem
def expect_cells(cells, base=None, colour=None, order=None):
    exp = []
    for n, key in enumerate(order or sorted(cells)):
        row, col = key
        exp.append('[$b000 + %d * 40 + %d] == %d' % (row, col, 128 + n))
        if colour is not None: exp.append('[$ac00 + %d * 40 + %d] == %d' % (row, col, colour))
        for ln in range(8):
            d, m = cells[key][ln]
            b = (base or {}).get((key, ln), 0)
            exp.append('[$e400 + %d * 8 + %d] == %s' % (n, ln, h((b & ~m & 255) | d)))
    exp.append('[$ccc0] == %d' % len(order or cells))
    return exp
m = comp_setup(); slot(m, 0, 85, 100, 0, 15)          # x = 85 -> Spalte 10, Feinlage 5
test('tsp_render: Ball bei x=85 (Feinlage 5), y=100 (senkrecht 4)', {'mem': m}, 'tsp_render', expect_cells(paint([(BALL, 85, 100)]), colour=15))
for x, y in ((0, 16), (3, 33), (314, 190), (156, 8), (255, 77), (256, 77), (300, 150)):
    m = comp_setup(); slot(m, 0, x, y, 0, 13)
    cells = {k: v for k, v in paint([(BALL, x, y)]).items() if k[1] < 40 and k[0] < 24}
    test('tsp_render: Ball bei x=%d y=%d (X mit Hoch-Byte)' % (x, y), {'mem': m}, 'tsp_render', expect_cells(cells, colour=13))
m = comp_setup(); slot(m, 3, 40, 184, 1, 3)
test('tsp_render: Schlaeger (16 px) bei x=40, Zeile 23', {'mem': m}, 'tsp_render', expect_cells(paint([(PAD16, 40, 184)]), colour=3))
m = comp_setup(); slot(m, 3, 163, 184, 1, 3); slot(m, 0, 170, 182, 0, 15)
cells = paint([(PAD16, 163, 184), (BALL, 170, 182)])
paddle_cells = sorted(paint([(PAD16, 163, 184)]))
ball_new = [k for k in sorted(paint([(BALL, 170, 182)])) if k not in paddle_cells]
test('tsp_render: Schlaeger und Ball in gemeinsamen Zellen (Ball oben)', {'mem': m}, 'tsp_render', expect_cells(cells, order=paddle_cells + ball_new))
# Mehrfarb-/Hintergrundzeichen unter dem Sprite bleibt neben dem Sprite sichtbar: Stein links (Zeichen 64) im Schattenspeicher, Zelle (12, 10)
bg = [0x7f] * 7 + [0]
m = comp_setup(); slot(m, 0, 85, 100, 0, 15)
m['$%x' % (0xb400 + 12 * 40 + 10)] = 64
m['$e200'] = bg
cells = paint([(BALL, 85, 100)])
test('tsp_render: Hintergrundzeichen neben dem Sprite bleibt sichtbar (Hintergrund UND Maske ODER Bild)', {'mem': m}, 'tsp_render',
     expect_cells(cells, base={((12, 10), ln): bg[ln] for ln in range(8)}, colour=15))
# Zeile 24 und tiefer: keine Sprites
m = comp_setup(); slot(m, 0, 80, 192, 0, 15)
test('tsp_render: Sprite in Zeile 24 wird nicht gezeichnet', {'mem': m}, 'tsp_render', ['[$ccc0] == 0', '[$b000 + 24 * 40 + 10] == 32'])
# Wiederherstellen: eine Pool-Zelle des vorigen Bildes wird aus der Schattenkopie zurueckgesetzt (Spiegel und VDC)
m = comp_setup()
m['$cc40'] = 0x0a; m['$cc80'] = 0xb1                # Zelle bei $B10A = Index 266 = Zeile 6, Spalte 26
m['$ccc0'] = 1
m['$b10a'] = 128; m['$ad0a'] = 15
m['$%x' % (0xb400 + 266)] = 64; m['$%x' % (0xb800 + 266)] = 5
test('tsp_render: Zellen des letzten Bildes werden aus der Schattenkopie wiederhergestellt', {'mem': m}, 'tsp_render', ['[$b10a] == 64', '[$ad0a] == 5', '[$ccc0] == 0'])
# VDC nach der Wiederherstellung: Zelle (6, 26): VDC-Adresse (6*40+26)*2 = 532 = $0214
test('VDC nach Wiederherstellung: linke Zelle = Zeichen 64', {'a': '2', 'y': '$14'}, 'vdc_get', ['a == 64'])
test('VDC nach Wiederherstellung: rechte Zelle = Zeichen 64', {'a': '2', 'y': '$15'}, 'vdc_get', ['a == 64'])
test('VDC nach Wiederherstellung: Attribut links = 5, rechts = 5 mit Bit 7', {'a': '$0a', 'y': '$14'}, 'vdc_get', ['a == 5'])
test('VDC nach Wiederherstellung: Attribut rechts', {'a': '$0a', 'y': '$15'}, 'vdc_get', ['a == $85'])
# Ball im VDC: Zelle (12, 10) = Pool-Zeichen 128; linkes Halbzeichen im ersten Zeichensatz, rechtes im zweiten (gedehnt)
m = comp_setup(); slot(m, 0, 85, 100, 0, 15)
test('tsp_render (fuer VDC-Pruefung): Ball bei x=85, y=100', {'mem': m}, 'tsp_render', ['[$ccc0] == 4'])
cellsb = paint([(BALL, 85, 100)]); first = sorted(cellsb)[0]
d0 = cellsb[first][4][0]                      # Zeile 4 der ersten Zelle (der Ball beginnt in Pixelzeile 4)
def stretch(n): return sum(((n >> k) & 1) * 3 << (2 * k) for k in range(4))
test('VDC: Pool-Zeichen 128, Zeile 4, linke Haelfte (obere 4 Pixel gedehnt)', {'a': '$28', 'y': '4'}, 'vdc_get', ['a == %s' % h(stretch(d0 >> 4))])
test('VDC: Pool-Zeichen 128, Zeile 4, rechte Haelfte (untere 4 Pixel gedehnt)', {'a': '$38', 'y': '4'}, 'vdc_get', ['a == %s' % h(stretch(d0 & 15))])
test('VDC: Zelle (12, 10) zeigt Pool-Zeichen 128 (linke Zelle)', {'a': '3', 'y': '$d4'}, 'vdc_get', ['a == 128'])      # (12*40+10)*2 = 980 = $3D4
# Ticker: geaenderte Zeichen werden gedehnt in den VDC geladen
test('tk_upload: Ticker-Zeichen 0 (TK_BUF) wird gedehnt in den VDC geschrieben', {'mem': {'$e700': [0xf0, 0x0f, 0xaa, 0x55, 0xff, 0x00, 0x81, 0x18], 'tk_vdc': [0] * 256}}, 'tk_upload', ['[tk_vdc] == $f0'])
test('VDC nach tk_upload: Zeichen 224, Zeile 0, links = $FF (obere 4 Bit $F gedehnt)', {'a': '$2e', 'y': '0'}, 'vdc_get', ['a == $ff'])
test('VDC nach tk_upload: Zeichen 224, Zeile 0, rechts = $00', {'a': '$3e', 'y': '0'}, 'vdc_get', ['a == 0'])
test('VDC nach tk_upload: Zeichen 224, Zeile 1, rechts = $FF (untere 4 Bit $F gedehnt)', {'a': '$3e', 'y': '1'}, 'vdc_get', ['a == $ff'])
test('VDC nach tk_upload: Zeichen 224, Zeile 2, links = $CC (obere 4 Bit $A = 1010 gedehnt)', {'a': '$2e', 'y': '2'}, 'vdc_get', ['a == $cc'])

doc = {'kind': 'test', 'target': 'c128', 'program': 'build/prallball.prg', 'trap': '$bf00',
       'static': [
           {'check': 'overlap'},
           {'check': 'expr', 'name': 'Levels, Lied und Streifen enden vor den Spiegeln und Schattenkopien ($AC00)', 'expr': 'song_patterns + 16 * 80 < $ac00'},
           {'check': 'expr', 'name': 'Bestenliste (Umschaltung der Systemroms) und Zeichensatz-Kopie liegen unter $4000', 'expr': 'hs_io_off + 40 < $4000 && copy_font + 80 < $4000'},
           {'check': 'cycles', 'label': 'add_score', 'max': 60},
       ],
       'tests': tests}
json.dump(doc, open('prallball.vctest', 'w'), indent=1, ensure_ascii=False)
print(len(tests), 'Tests geschrieben')
