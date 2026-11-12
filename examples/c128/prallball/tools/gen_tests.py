#!/usr/bin/env python3
"""Erzeugt prallball.vctest (statische Pruefungen und Emulator-Tests im Trap-Modus fuer `vcc test --backend vice`).
Erwartungswerte (Richtungstabelle, Zufall, CRC, Levelzaehlungen) werden hier unabhaengig vom Programm berechnet.
Aufruf im Ordner examples/c128/prallball:  python3 tools/gen_tests.py   (vorher: vcc build .)
Aufbau: Jeder Test ruft eine Routine von prallball.asm ueber die Falle $BF00 auf (setup: Speicher "mem", Register a/x/y; call: Marke;
expect: Ausdruecke ueber Register, Flags und Speicher).  Zuerst stellt jeder Test ueber $FF00 die MMU-Konfiguration $3E ein (das Spiel
laeuft ohne Systemroms).  Danach folgen statische Pruefungen (Speicherplan, Ueberlappung, Zyklen von add_score).  Nach einer Aenderung
von prallball.asm neu bauen, dann dieses Skript ausfuehren, dann: vcc test . --backend vice (x128)."""
import json, math, os, re
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/c128/prallball (os.chdir); alle Pfade gelten relativ dazu, der Aufruf funktioniert daher aus jedem Ordner.', epilog='Beispiele:\n  python3 examples/c128/prallball/tools/gen_tests.py\n  cd examples/c128/prallball && python3 tools/gen_tests.py\n\nSchreibt (im Projektordner examples/c128/prallball, vorhandene Dateien werden ueberschrieben): prallball.vctest.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))

# ---- Zeropage-Adressen (wie in prallball.asm: bei Aenderungen dort hier nachziehen; die Tests rechnen sonst mit falschen Zellen)
ZP = dict(bxl=0x10, bxh=0x11, byl=0x12, byh=0x13, vxl=0x14, vxh=0x15, vyl=0x16, vyh=0x17, padl=0x18, padh=0x19, speed=0x1a, hits=0x1b,
          level=0x1d, bricks=0x1e, stuck=0x1f, sc0=0x20, sc1=0x21, sc2=0x22, bcol=0x28, brow=0x29, bidx=0x2a, padw=0x45, zstep=0x46,
          capact=0x48, tpierce=0x88, mask=0x23, oyl=0x39, oyh=0x3a, spdmod=0x50, sticky=0x52, cores=0x6f, lkind=0x6c, cb=0x5a)
def h(v): return '$%02x' % v      # Zahl -> Adress-/Wert-Schreibweise des Testformats ("$1d")

# ---- Richtungstabelle neu berechnen (gleiche Formel wie tools/gen.py, aber unabhaengig implementiert)
ANG = [-60, -43, -26, -9, 9, 26, 43, 60]
def vel(level, zone):
    speed = (1.6 + 0.1 * level) * 64
    a = math.radians(ANG[zone])
    dx = int(round(speed * math.sin(a))); dy = int(round(-speed * math.cos(a)))
    return dx & 0xffff, dy & 0xffff

tests = []
def test(name, setup, call, expect, ret=None, cycles=None):
    # name: Anzeigetext; setup: {'mem': {Adresse oder Marke: Wert oder Liste}, 'a'/'x'/'y': Registerwert}; call: Marke der Routine;
    # expect: Liste von Ausdruecken (z. B. '[$20] == $09', 'a == 0', 'fc', '!fc'); cycles wird nicht ausgewertet (siehe unten).
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

# ---- Zufall: Galois-LFSR nachrechnen (rand.asm: 16 Bit, nach rechts schieben, bei herausgefallenem Bit High-Byte XOR $B4; Ergebnis = lo XOR hi)
def lfsr(lo, hi):
    c = hi & 1; hi >>= 1
    c2 = lo & 1; lo = (lo >> 1) | (c << 7)
    if c2: hi ^= 0xb4
    return lo, hi, lo ^ hi
lo, hi = 0xa5, 0x5a                   # Startzustand $A5/$5A (wie rand_seed ihn setzt, wenn A xor Rasterzeile = 0); die Schleife darunter rechnet drei Schritte, benutzt wird nur lfsr() unten
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


# ---- C128: Hardware-Hilfen ---------------------------------------------------------------------------------------------------
for dev, want in ((8, 8), (9, 9), (11, 11), (0, 8), (7, 8), (16, 8), (30, 8)):
    test('hs_device: Startgeraet %d -> %d' % (dev, want), {'mem': {'hs_bootdev': dev}}, 'hs_device', ['x == %d' % want])
test('hs_init: merkt das KERNAL-Geraet aus $BA', {'mem': {'$ba': 9, 'hs_bootdev': 8}}, 'hs_init', ['[hs_bootdev] == 9'])
test('copy_font: Zeichen A (Bildschirmcode 1) aus dem Zeichen-ROM nach $8808; eigener Stein links bei $8A00', {'mem': {'$8808': 0, '$8a00': 0}}, 'copy_font',
     ['[$8808] == $18', '[$8809] == $3c', '[$8a00] == $15', '[$8a01] == $1f'])
test('clear_screen: Bildschirm bei $8400 leer (32), Farb-RAM weiss', {'mem': {'$8400': 5, '$87e7': 5, '$d800': 7, '$dbe7': 7}}, 'clear_screen',
     ['[$8400] == 32', '[$87e7] == 32', '([$d800] & 15) == 1', '([$dbe7] & 15) == 1'])
test('put_cell: Zeichen A mit Farbe an Spalte 3, Zeile 2 (Bildschirm $8400, Farb-RAM $D800)', {'mem': {'$33': 7}, 'a': '1', 'x': 3, 'y': 2}, 'put_cell',
     ['[$8400 + 2 * 40 + 3] == 1', '([$d800 + 2 * 40 + 3] & 15) == 7'])
test('vic_init: VIC-Bank 2 (CIA2 Bits 0-1 = 01), Bildschirm $8400 / Zeichensatz $8800 ($D018 = $12), 1 MHz', {'mem': {'$d030': 1}}, 'vic_init',
     ['([$dd00] & 3) == 1', '([$d018] & $fe) == $12', '([$d030] & 1) == 0', '([$ffff] * 256 + [$fffe]) == irq'])
# Der IRQ-Vektor ($FFFE) wird oben im Test von vic_init geprueft; hier nur, dass an rti_only der Opcode RTI ($40) steht (Ziel von NMI/RESET).
test('rti_only: Ziel des NMI-/RESET-Vektors enthaelt den Opcode RTI ($40)', {}, 'hs_device', ['[rti_only] == $40'])

# ---- Zusatztasten des C128 (input_xkey, Tabelle xk_tab; die Zeilenreihenfolge der Spalten K0..K2 wurde gegen die KERNAL-Dekodiertabelle ab $FA80 geprueft)
# Ohne gedrueckte Zusatztaste liefert die Tastatur $FF: in_pad bleibt unveraendert, die Zusatzspalten stehen danach wieder auf $FF.
test('input_xkey: keine Zusatztaste gedrueckt -> in_pad unveraendert, $D02F wieder $FF', {'mem': {'$8e': 0x00}}, 'input_xkey', ['[$8e] == 0', '[$d02f] == $ff'])
# Die Tabelle: Spaltenmaske, Zeilenmaske, Aktion (IN_UP $01, IN_DOWN $02, IN_LEFT $04, IN_RIGHT $08, IN_FIRE $10, IN_PAUSE $20, IN_ABORT $40).
xk = [(0xfe, 0x01, 0x20), (0xfe, 0x02, 0x01), (0xfe, 0x04, 0x10), (0xfe, 0x10, 0x02), (0xfe, 0x20, 0x04), (0xfd, 0x01, 0x40), (0xfd, 0x10, 0x10),
      (0xfd, 0x20, 0x08), (0xfb, 0x02, 0x10), (0xfb, 0x08, 0x01), (0xfb, 0x10, 0x02), (0xfb, 0x20, 0x04), (0xfb, 0x40, 0x08)]
test('xk_tab: HELP/ESC/Ziffernblock/Cursortasten -> Pause/Abbruch/Richtungen/Feuer', {}, 'input_xkey',
     ['[xk_tab + %d] == %s' % (3 * i + j, h(v)) for i, e in enumerate(xk) for j, v in enumerate(e)] + ['[xk_tab + %d] == 0' % (3 * len(xk))])
# spawn_cap: erzwungene Art 6 (capforce = 7, '+') muss Farbe weiss (1) von cap_color[6] bekommen (Art 0 waere hellblau 14): faengt den
# frueheren Fehler ab, bei dem immer Art 0 benutzt wurde.
test('spawn_cap: erzwungene Art 6 (+) -> Muenzenfarbe weiss, captype 6', {'mem': {'$71': 7, '$48': 0, '$28': 3, '$29': 2}}, 'spawn_cap',
     ['[$4c] == 6', '([$d02a] & 15) == 1', '[$48] == 1'])
# ---- Testdatei: statische Pruefungen (Speicherplan wie im Kopf von prallball.asm) und die Tests oben
doc = {'kind': 'test', 'target': 'c128', 'program': 'build/prallball.prg', 'trap': '$bf00',
       'static': [
           {'check': 'overlap'},
           {'check': 'expr', 'name': 'Programmcode endet vor den Sprite-Daten bei $8000', 'expr': 'gfx_chars + 40 < $8000'},
           {'check': 'expr', 'name': 'Sprite-Daten liegen bei $8000 (VIC-Bank 2, Zeiger $00)', 'expr': 'gfx_ball == $8000'},
           {'check': 'expr', 'name': 'Levels beginnen bei $9400 (der VIC sieht bei $9000-$9FFF nur das Zeichen-ROM)', 'expr': 'level1 == $9400'},
           {'check': 'expr', 'name': 'Levels und Lied enden vor der Steinkarte ($C000)', 'expr': 'song_patterns + 16 * 80 < $c000'},
           {'check': 'expr', 'name': 'Bestenliste (Umschaltung der Systemroms) und Zeichensatz-Kopie liegen unter $4000', 'expr': 'hs_io_off + 40 < $4000 && copy_font + 80 < $4000'},
           {'check': 'cycles', 'label': 'add_score', 'max': 60},
       ],
       'tests': tests}
json.dump(doc, open('prallball.vctest', 'w'), indent=1, ensure_ascii=False)
print(len(tests), 'Tests geschrieben')
