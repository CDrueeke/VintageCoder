#!/usr/bin/env python3
"""Testlaeufer fuer PRALLBALL (VC-20, 8 KB): startet VICE xvic mit `-memory 8k` und dem Text-Monitor (-remotemonitor), laedt build/prallball.prg und ruft
Unterprogramme einzeln auf (Falle bei $3F00): Speicher und Register setzen, Unterprogramm ausfuehren, Ergebnis pruefen.  Erwartungswerte werden hier
unabhaengig vom Programm berechnet.  Grund: `vcc test --backend vice` startet xvic ohne Speichererweiterung.
Aufruf im Ordner examples/vic20/prallball:  python3 tools/run_tests.py   (vorher: vcc build .)   Optionen: --filter TEXT

Voraussetzungen: VICE xvic unter dem Pfad EXE (unten, ein macOS-Pfad: auf anderen Systemen anpassen), freier TCP-Port PORT fuer den Text-Monitor, und
build/prallball.prg samt build/prallball.sym.txt (Symbole mit Adressen, werden fuer die Unterprogrammaufrufe gelesen).  Es darf kein anderer xvic mit
Remote-Monitor auf diesem Port laufen.  Exit-Code 0 = alle Pruefungen bestanden, 1 = mindestens eine fehlgeschlagen.
Ablauf je Unterprogramm (Mon.call): Falle `jmp *` bei TRAP legen, Ruecksprungadresse so auf den Stapel schreiben, dass RTS dort landet, Register und PC setzen,
fortfahren, bis der Haltepunkt anspricht; dann Register und Speicher lesen.  Die Pruefgruppen (statisch, add_score, rand, set_vel, paddle_hit, cell_test,
hit_brick, load_level, cap, tsp, hs_, sfx, mus) lassen sich mit --filter einzeln waehlen."""
import math, os, re, socket, subprocess, sys, time
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\nArbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/vic20/prallball (os.chdir); der Aufruf funktioniert daher aus jedem Ordner. Das Werkzeug schreibt keine Dateien; es startet VICE xvic und beendet es am Ende wieder.",
    epilog="""Beispiele:
  python3 examples/vic20/prallball/tools/run_tests.py
  cd examples/vic20/prallball && python3 tools/run_tests.py --filter paddle_hit

Voraussetzung: vorher bauen (vcc build .), damit build/prallball.prg und build/prallball.sym.txt vorhanden sind.
Exit-Code 0 = alle Pruefungen bestanden, 1 = mindestens eine fehlgeschlagen.
-h/--help zeigt nur diesen Text und startet nichts.""",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.add_argument('--filter', default='', metavar='TEXT', help='nur Pruefgruppen ausfuehren, deren Name TEXT enthaelt (Standard: alle)')
_args = _cli.parse_args()   # vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
EXE = '/Applications/vice-arm64-gtk3-3.10/bin/xvic'      # Pfad zu xvic (VICE 3.10, macOS arm64) - auf dem eigenen System anpassen
PORT = 6510                                              # Standardport des VICE-Text-Monitors (-remotemonitor)
TRAP = 0x3f00                                            # Adresse der Falle im freien RAM der 8-KB-Erweiterung (hinter dem Programm, siehe prallball.asm: Reserve bis $3FFF)

# ---- Symbole: Name -> Adresse aus build/prallball.sym.txt (Zeilen "name  $adresse  dezimal")
SYM = {}
for line in open('build/prallball.sym.txt', encoding='utf-8'):
    p = line.split()
    if len(p) >= 2 and p[1].startswith('$') and not line.startswith(';'): SYM[p[0]] = int(p[1][1:], 16)
def A(x): return SYM[x] if isinstance(x, str) else x          # Symbolname oder Zahl -> Adresse

class Mon:
    """Verbindung zum VICE-Text-Monitor: startet xvic, sendet Monitorbefehle und liest die Antworten."""
    def __init__(self):
        self.p = subprocess.Popen([EXE, '-console', '-silent', '-warp', '-memory', '8k', '-remotemonitor', '+confirmonexit'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        for _ in range(60):
            try: self.s = socket.create_connection(('127.0.0.1', PORT)); break
            except OSError: time.sleep(0.25)
        self.s.settimeout(0.04)
        time.sleep(3)
        self.drain()
    def drain(self):
        """Alles bereits Empfangene verwerfen."""
        try:
            while self.s.recv(65536): pass
        except Exception: pass
    def cmd(self, c, timeout=5.0):
        """Befehl senden, Antwort bis zur Eingabeaufforderung "(C:$xxxx) " lesen (und noch kurz warten, ob mehr nachkommt)."""
        self.drain()
        self.s.sendall((c + '\n').encode())
        out = ''; end = time.time() + timeout; quiet = 0
        while time.time() < end:
            try: d = self.s.recv(65536).decode(errors='replace')
            except Exception: d = ''
            if d: out += d; quiet = 0
            else: quiet += 1
            if re.search(r'\(C:\$[0-9a-f]{4}\) $', out) and quiet >= 2: return out
        return out
    def poke(self, addr, data):
        """Bytes (Zahl oder Liste, Symbol oder Adresse) in den Speicher schreiben, je Befehl hoechstens 16 Byte."""
        if isinstance(data, int): data = [data]
        for i in range(0, len(data), 16):
            self.cmd('> %04x %s' % (A(addr) + i, ' '.join('%02x' % (b & 255) for b in data[i:i + 16])))
    def peek(self, addr, n=1):
        """n Bytes ab addr lesen (Monitorbefehl m) und als Liste von Zahlen liefern."""
        a = A(addr); out = []
        o = self.cmd('m %04x %04x' % (a, a + n - 1))
        for l in o.split('\n'):
            m = re.match(r'>C:[0-9a-f]{4}\s+((?:[0-9a-f]{2}\s+)+)', l)
            if m: out += [int(x, 16) for x in m.group(1).split()]
        return out[:n]
    def regs(self):
        """Register lesen (Monitorbefehl r): Dict mit pc, a, x, y, sp und fl (Flags als 8 Zeichen, NV-BDIZC)."""
        o = self.cmd('r')
        m = re.search(r'\.;([0-9a-f]{4}) ([0-9a-f]{2}) ([0-9a-f]{2}) ([0-9a-f]{2}) ([0-9a-f]{2}) \S+ \S+ ([01]{8})', o)
        return dict(pc=int(m.group(1), 16), a=int(m.group(2), 16), x=int(m.group(3), 16), y=int(m.group(4), 16), sp=int(m.group(5), 16), fl=m.group(6))
    def call(self, label, a=0, x=0, y=0, timeout=20.0):
        """Unterprogramm label mit A/X/Y aufrufen und bis zur Rueckkehr (Falle bei TRAP) warten; liefert die Register mit den Flags c (Carry) und z (Zero)."""
        self.poke(TRAP, [0x4c, TRAP & 255, TRAP >> 8])         # jmp * als Falle
        self.cmd('del')
        self.cmd('break %04x' % TRAP)
        ret = TRAP - 1
        self.poke(0x01fe, [ret & 255, ret >> 8])               # Ruecksprungadresse: RTS landet auf der Falle
        for r in ('a=%02x' % a, 'x=%02x' % x, 'y=%02x' % y, 'sp=fd', 'fl=24', 'pc=%04x' % A(label)): self.cmd('r ' + r)
        self.s.sendall(b'x\n')
        out = ''; end = time.time() + timeout
        while time.time() < end:
            try: d = self.s.recv(65536).decode(errors='replace')
            except Exception: d = ''
            out += d
            if 'Stop on' in out: break
        else: raise TimeoutError('%s kehrt nicht zurueck' % label)
        time.sleep(0.05); self.drain()
        r = self.regs(); r['c'] = r['fl'][7] == '1'; r['z'] = r['fl'][6] == '1'
        return r
    def close(self):
        """xvic beenden."""
        try: self.cmd('quit', 1)
        except Exception: pass
        self.p.kill()

# ---- Auswahl und Auswertung: --filter TEXT fuehrt nur Pruefgruppen aus, deren Name TEXT enthaelt (want); check() merkt und druckt jedes Ergebnis
FILT = _args.filter
res = []
def check(name, ok, info=''):
    res.append((name, ok, info))
    print(('  OK       ' if ok else '  FEHLER   ') + name + ('' if ok else '  ' + info))
def want(name):
    return FILT in name

def zp(m, var, val): m.poke(SYM.get(var, var) if isinstance(var, str) else var, val)
# Zeropage-Adressen der Spielvariablen (Spiegel der .const-Liste in prallball.asm; hier von Hand gepflegt, nicht aus dem Programm gelesen)
ZP = dict(bxl=0x10, bxh=0x11, byl=0x12, byh=0x13, vxl=0x14, vxh=0x15, vyl=0x16, vyh=0x17, padx=0x18, padw=0x19, speed=0x1a, hits=0x1b, lives=0x1c, level=0x1d,
          bricks=0x1e, stuck=0x1f, sc0=0x20, sc1=0x21, sc2=0x22, mask=0x23, t5=0x29, pmov=0x2a, capact=0x32, capx=0x33, capy=0x34, captype=0x35, tpad=0x36, tspd=0x37,
          spdmod=0x38, zone=0x5a, pool_n=0x3d, bidx=0x4e, lkind=0x3a)
MAP = 0x0340                                   # Steinkarte (wie .const MAP im Programm)

# ---- Richtungstabelle unabhaengig neu berechnen (gleiche Formel wie tools/gen.py): vel() liefert (vx, vy) als 16-Bit-Wert in 1/256 Punkt je Bild
ANG = [-60, -43, -26, -9, 9, 26, 43, 60]
def vel(level, zone):
    sp = (0.95 + 0.14 * level) * 32
    a = math.radians(ANG[zone])
    vx = int(round(sp * math.sin(a))); vy = int(round(-sp * math.cos(a)))
    return vx * 8 & 0xffff, vy * 8 & 0xffff

def main():
    """Emulator starten, Programm laden und alle Pruefgruppen der Reihe nach ausfuehren; Exit-Code 1 bei Fehlern."""
    m = Mon()
    try:
        # Programm laden
        o = m.cmd('load "%s" 0' % os.path.abspath('build/prallball.prg'), 10)
        assert 'Loading' in o, o
        # ---- statische Pruefungen
        check('Teil 1 des Programms endet vor dem Zeichensatz ($1C00)', SYM['cap_color'] + 6 <= 0x1c00, hex(SYM['cap_color'] + 6))
        check('Teil 2 beginnt bei $2000', SYM['blit'] == 0x2000, hex(SYM['blit']))
        end = max(v for k, v in SYM.items() if k in ('song_patterns', 'level_kind'))
        check('Programmende liegt unter $3F00 (Falle des Testlaeufers, Ende des 8-KB-Ausbaus $3FFF)', end + 400 < 0x3f00, hex(end))
        # ---- add_score
        if want('add_score'):
            zp(m, 'sc0', [0x99, 0x09, 0x00]); r = m.call('add_score', a=0x10)
            check('add_score: 0999 + 10 = 1009', m.peek(0x20, 3) == [0x09, 0x10, 0x00], str(m.peek(0x20, 3)))
            zp(m, 'sc0', [0x95, 0x99, 0x00]); m.call('add_score', a=0x10)
            check('add_score: Uebertrag in die hoechsten Ziffern', m.peek(0x20, 3) == [0x05, 0x00, 0x01])
        # ---- Zufall
        if want('rand'):
            def lfsr(lo, hi):
                c = hi & 1; hi >>= 1; c2 = lo & 1; lo = (lo >> 1) | (c << 7)
                if c2: hi ^= 0xb4
                return lo, hi, lo ^ hi
            m.poke(0x02, [0xa5, 0x5a]); r = m.call('rand_byte'); lo, hi, a = lfsr(0xa5, 0x5a)
            check('rand_byte: erste Zahl aus Startwert $A5/$5A', r['a'] == a and m.peek(0x02, 2) == [lo, hi])
        # ---- set_vel
        if want('set_vel'):
            for lvl, zone, mod in ((0, 3, 0), (5, 0, 0), (7, 7, 0), (2, 2, 1), (6, 4, 1), (1, 4, 2)):
                zp(m, 'speed', lvl); zp(m, 'zone', zone); zp(m, 'spdmod', mod); m.call('set_vel')
                eff = lvl if mod == 0 else (min(7, lvl + 3) if mod == 1 else max(0, lvl - 3))
                vx, vy = vel(eff, zone)
                got = m.peek(0x14, 4)
                check('set_vel: Stufe %d Zone %d Extra %d' % (lvl, zone, mod), got == [vx & 255, vx >> 8, vy & 255, vy >> 8], '%s statt %s' % (got, [vx & 255, vx >> 8, vy & 255, vy >> 8]))
        # ---- paddle_hit: Schlaeger x=100..123 (24 breit), Zonenbreite 3
        if want('paddle_hit'):
            for bx, pm, zone, nm in ((112, 0, 4, 'Mitte'), (100, 0, 0, 'linke Kante'), (123, 0, 7, 'rechte Kante'), (112, 1, 3, 'Schlaeger bewegt links'), (112, 2, 5, 'Schlaeger bewegt rechts'),
                                      (100, 1, 0, 'links bleibt Zone 0'), (123, 2, 7, 'rechts bleibt Zone 7'), (106, 0, 2, 'Zone 2')):
                m.poke(0x10, [0, bx - 2 + 0, 0, 160, 0, 0, 0x40, 0]); zp(m, 'padx', 100); zp(m, 'padw', 24); zp(m, 'pmov', pm); zp(m, 'speed', 0); zp(m, 'hits', 0)
                zp(m, 'spdmod', 0)
                # Ballmitte = bxh + 2 -> Abstand zur linken Kante
                m.poke(0x11, bx - 2 + 2 - 2 + 0)                                   # bxh so, dass Ballmitte = bx
                m.call('paddle_hit')
                z = {0: 0}.get(0)
                exp = vel(0, zone)
                got = m.peek(0x14, 4)
                check('paddle_hit: %s -> Zone %d' % (nm, zone), got[0] == exp[0] & 255 and got[1] == exp[0] >> 8 and m.peek(0x13)[0] == 165 and m.peek(0x1b)[0] == 1, '%s, erwartet %s' % (got, exp))
        # ---- Steinkollision
        if want('cell_test'):
            m.poke(MAP, [0] * 120); m.poke(MAP, [1])
            for (px, py, hit, nm) in ((8, 16, 1, 'Stein oben links'), (7, 16, 1, 'linke Wand'), (176 - 8, 40, 1, 'rechte Wand'), (60, 12, 1, 'obere Wand'), (60, 130, 0, 'unter den Steinen'), (40, 40, 0, 'leerer Platz')):
                zp(m, 't5', 0); m.poke(0x4e, 0); zp(m, 'bricks', 1)
                m.poke(MAP, [1] + [0] * 119)
                r = m.call('cell_test', a=px, x=py)
                got = m.peek(0x29)[0]
                check('cell_test: %s' % nm, got == hit, 'Trefferflag %d' % got)
        # ---- hit_brick
        if want('hit_brick'):
            m.poke(MAP, [0] * 120); m.poke(MAP + 5, [10]); zp(m, 'bidx', 5); zp(m, 'bricks', 3); zp(m, 'sc0', [0, 0, 0]); zp(m, 'lkind', 0); zp(m, 'capact', 1)
            m.call('hit_brick'); check('hit_brick: Gold (10) -> Silber (9), Stein bleibt', m.peek(MAP + 5)[0] == 9 and m.peek(0x1e)[0] == 3)
            m.call('hit_brick'); check('hit_brick: Silber (9) -> normaler Stein (1)', m.peek(MAP + 5)[0] == 1 and m.peek(0x1e)[0] == 3)
            zp(m, 'sc0', [0, 0, 0])                    # Treffer 1 und 2 haben schon 90 + 50 Punkte gegeben (jeder Treffer gibt die Punkte des alten Typs)
            m.call('hit_brick'); check('hit_brick: normaler Stein wird zerstoert, Zaehler -1, 10 Punkte', m.peek(MAP + 5)[0] == 0 and m.peek(0x1e)[0] == 2 and m.peek(0x20, 3)[0] == 0x10)
            m.poke(MAP + 7, [11]); zp(m, 'bidx', 7); m.call('hit_brick'); check('hit_brick: unzerstoerbar (11) bleibt', m.peek(MAP + 7)[0] == 11 and m.peek(0x1e)[0] == 2)
            # explosiv: Nachbarn weg, unzerstoerbare bleiben
            m.poke(MAP, [0] * 120)
            for idx in (24, 25, 26, 34, 36, 44, 45, 46): m.poke(MAP + idx, [1])
            m.poke(MAP + 35, [12]); m.poke(MAP + 46, [11]); zp(m, 'bidx', 35); zp(m, 'bricks', 9); zp(m, 'capact', 1)
            m.call('hit_brick')
            left = m.peek(MAP, 120)
            check('hit_brick: explosiv (12) zerstoert die Nachbarn, unzerstoerbare bleiben', [i for i, v in enumerate(left) if v] == [46] and m.peek(0x1e)[0] == 9 - 8, str([i for i, v in enumerate(left) if v]))
        # ---- load_level: Steinzaehlung gegen levels.txt, auf 10 Spalten abgeleitet
        if want('load_level'):
            TYPE = {'.': 0, 'a': 1, 'b': 2, 'c': 3, 'd': 4, 'e': 5, 'f': 6, 'g': 7, 'h': 8, 's': 9, 'G': 10, '#': 11, 'x': 12, 'p': 13}
            lv = []; cur = None
            for line in open('levels/levels.txt', encoding='utf-8'):
                line = line.rstrip('\n'); mm = re.match(r'level (\d+) "(.*)"', line)
                if mm: cur = {'name': mm.group(2), 'rows': []}; lv.append(cur); continue
                if cur is not None and len(line) == 18 and not line.startswith('#  '): cur['rows'].append(line)
            lv = [l for l in lv if not l['name'].startswith('Boss')]
            for i, l in enumerate(lv):
                cells = [TYPE[row[(18 * j + 9) // 10]] for row in l['rows'] for j in range(10)]
                cnt = sum(1 for c in cells if c not in (0, 11)); kind = 1 if l['name'].startswith('Bonus') else 0
                zp(m, 'level', i); m.call('load_level', timeout=60)
                check('load_level %d (%s): %d Steine, Art %d, Karte stimmt' % (i + 1, l['name'], cnt, kind), m.peek(0x1e)[0] == cnt and m.peek(0x3a)[0] == kind and m.peek(MAP, 120) == cells, str(m.peek(0x1e)))
        # ---- Muenzen (update_cap): Paddel x=100..123, Muenze 8 breit auf Schlaegerhoehe (Unterkante capy + 8 im Fenster PADY .. PADY+5)
        if want('cap'):
            def cap(typ, capx, capy, setup=None):
                zp(m, 'padx', 100); zp(m, 'padw', 24); zp(m, 'capact', 1); zp(m, 'captype', typ); zp(m, 'capx', capx); zp(m, 'capy', capy)
                zp(m, 'tpad', 0); zp(m, 'tspd', 0); zp(m, 'spdmod', 0); zp(m, 'lives', 3); zp(m, 'sc0', [0, 0, 0]); zp(m, 'sfx_mute', 1)
                m.call('update_cap')
                return m.peek(0x32)[0], m.peek(0x19)[0], m.peek(0x36)[0], m.peek(0x38)[0], m.peek(0x37)[0], m.peek(0x1c)[0], m.peek(0x20, 3)
            r = cap(0, 104, 161); check('update_cap: B gefangen -> Schlaeger 32 breit, 20 s', r[:3] == (0, 32, 40), str(r))
            r = cap(1, 104, 161); check('update_cap: S gefangen -> Schlaeger 16 breit', r[:3] == (0, 16, 40), str(r))
            r = cap(2, 104, 161); check('update_cap: L gefangen -> langsamer (spdmod 2)', r[0] == 0 and r[3] == 2 and r[4] == 40, str(r))
            r = cap(3, 104, 161); check('update_cap: F gefangen -> schneller (spdmod 1)', r[0] == 0 and r[3] == 1, str(r))
            r = cap(4, 104, 161); check('update_cap: + gefangen -> ein Ball mehr', r[0] == 0 and r[5] == 4, str(r))
            r = cap(5, 104, 161); check('update_cap: P gefangen -> 500 Punkte', r[0] == 0 and r[6] == [0, 5, 0], str(r))
            r = cap(0, 40, 161); check('update_cap: Muenze neben dem Schlaeger wird nicht gefangen (faellt weiter)', r[0] == 1 and r[1] == 24, str(r))
            r = cap(0, 104, 175); check('update_cap: Muenze unten heraus (capy >= 176) verschwindet ohne Wirkung', r[0] == 0 and r[1] == 24, str(r))
        # ---- Zeichen-Sprites
        if want('tsp'):
            m.poke(MAP, [0] * 120)
            # Ball 5 x 5 bei x = 85 (Feinlage 5), y = 100
            BALL = [0x70, 0xf8, 0xf8, 0xf8, 0x70]
            def paint(layers):
                cells = {}
                for rows, x, y, col in layers:
                    for r, b in enumerate(rows):
                        yy = y + r
                        for k in range(8):
                            if b & (0x80 >> k):
                                gx = x + k
                                c = cells.setdefault((yy // 8, gx // 8), [[0] * 8, col])
                                c[0][yy % 8] |= 0x80 >> (gx % 8); c[1] = col
                return cells
            def expect(cells, order=None):
                ok = True; info = ''
                for n, key in enumerate(order or sorted(cells)):
                    row, col = key
                    code = m.peek(0x1000 + row * 22 + col)[0]; colr = m.peek(0x9400 + row * 22 + col)[0] & 15
                    gl = m.peek(0x1c00 + (80 + n) * 8, 8)
                    if code != 80 + n or colr != cells[key][1] or gl != cells[key][0]: ok = False; info += ' Zelle %s: Code %d, Farbe %d, Glyph %s statt %s' % (key, code, colr, gl, cells[key][0])
                return ok and m.peek(0x3d)[0] == len(cells), info
            for (bx, by) in ((85, 100), (8, 16), (160, 150), (11, 33)):
                m.poke(0x10, [0, bx, 0, by]); m.poke(0x32, [0]); zp(m, 'padx', 76); zp(m, 'padw', 24); zp(m, 'pool_n', 0)
                m.call('clear_screen'); zp(m, 'pool_n', 0)
                m.call('tsp_render', timeout=30)
                cells = paint([([0x7f, 0xff, 0xff, 0xff, 0x7f], 76, 170, 3), ([0xff] * 5, 84, 170, 3), ([0xfe, 0xff, 0xff, 0xff, 0xfe], 92, 170, 3), (BALL, bx, by, 1)])
                ok, info = expect(cells, order=None) if False else (None, '')
                # Reihenfolge der Vergabe: Schlaeger zuerst (Zeile 21, Spalten 9..12), dann der Ball
                order = []
                for rows, x, y, col in ([0] * 5, 76, 170, 3), ([0] * 5, 84, 170, 3), ([0] * 5, 92, 170, 3), (BALL, bx, by, 1):
                    for r in range(5):
                        for k in (x // 8, (x + 7) // 8) if False else range(x // 8, (x + 7) // 8 + 1):
                            key = ((y + r) // 8, k)
                            if key not in order and key[0] < 23: order.append(key)
                # genauer: Zellen mit gesetzten Pixeln in Vergabereihenfolge (Zeile fuer Zeile je Sprite)
                order = []
                for rows, x, y, col in ([0x7f, 0xff, 0xff, 0xff, 0x7f], 76, 170, 3), ([0xff] * 5, 84, 170, 3), ([0xfe, 0xff, 0xff, 0xff, 0xfe], 92, 170, 3), (BALL, bx, by, 1):
                    for r, b in enumerate(rows):
                        for k in range(8):
                            if b & (0x80 >> k):
                                key = ((y + r) // 8, (x + k) // 8)
                                if key not in order: order.append(key)
                ok, info = expect(cells, order)
                check('tsp_render: Schlaeger (24 breit bei x=76) und Ball bei (%d, %d)' % (bx, by), ok, info)
            # Rueckbau: Zellen des letzten Bildes wieder Hintergrund (Stein oder leer)
            m.poke(MAP, [0] * 120); m.poke(MAP, [3]); m.call('tsp_restore')
            check('tsp_restore: pool_n = 0', m.peek(0x3d)[0] == 0)
        # ---- Bestenliste
        if want('hs_'):
            m.call('hs_default'); b = m.peek('hs_buf', 20)
            check('hs_default: Kennung PRBL, Version 1, SPIELER, 1000 Punkte', bytes(b[:5]) == b'PRBL\x01' and bytes(b[5:12]) == b'SPIELER' and b[14] == 0x10 and b[13] == 0)
            crc = 0xffff
            for _ in range(125):
                for _ in range(8): crc = ((crc << 1) ^ 0x1021) & 0xffff if crc & 0x8000 else (crc << 1) & 0xffff
            m.poke('hs_buf', [0] * 127); m.call('hs_makecrc'); got = m.peek(SYM['hs_buf'] + 125, 2)
            check('hs_makecrc: CRC-16/CCITT ueber 125 Nullbytes', got == [crc & 255, crc >> 8], str(got))
            m.call('hs_default'); m.poke('hs_points', [1, 0, 0]); r = m.call('hs_check'); check('hs_check: 1 Punkt -> hinter allen Eintraegen ($FF)', r['a'] == 0xff)
            m.poke('hs_points', [0x50, 0x12, 0]); r = m.call('hs_check'); check('hs_check: 1250 Punkte -> Platz 0', r['a'] == 0)
            m.poke('hs_points', [0x50, 0x12, 0]); zp(m, 'hs_level', 7); m.poke('hs_name', list(b'ABCDEFGH')); m.call('hs_insert', a=0)
            b = m.peek('hs_buf', 20); check('hs_insert: Name, Punkte und Level an Platz 0, Rest rutscht', bytes(b[5:13]) == b'ABCDEFGH' and b[13] == 0x50 and b[14] == 0x12 and b[16] == 7)
            for dev, wantdev in ((8, 8), (9, 9), (11, 11), (0, 8), (16, 8)):
                zp(m, 'hs_bootdev', dev); r = m.call('hs_device'); check('hs_device: Startgeraet %d -> %d' % (dev, wantdev), r['x'] == wantdev)
        # ---- Klang
        if want('sfx'):
            for sid, val, noise in ((0, 0xe4, 0), (2, 0x9c, 1), (4, 0xf0, 0)):
                m.poke(0x900a, [0, 0, 0, 0]); zp(m, 'sfx_mute', 0); m.call('sfx_play', a=sid)
                reg = 0x900d if noise else 0x900b
                check('sfx_play: Effekt %d -> %s = $%02X' % (sid, 'Rauschen' if noise else 'Alt', val), m.peek(reg)[0] == val, hex(m.peek(reg)[0]))
            zp(m, 'sfx_mute', 1); m.poke(0x900b, 0); m.call('sfx_play', a=0); check('sfx_play: stumm im Demo-Modus', m.peek(0x900b)[0] == 0)
        if want('mus'):
            m.poke(0x04, [0, 0, SYM['song'] & 255, SYM['song'] >> 8]); zp(m, 'mus_bar', 3); zp(m, 'mus_row', 0); zp(m, 'mus_tk', 0); zp(m, 'sfx_t', 0)
            m.poke(0x900a, [0, 0, 0, 0]); m.call('mus_tick')
            # Takt 3: Muster (6, 7, 8): Melodie Note 56, Bass Note 25, Alt (Arpeggio) Note 49 laut Liedtabelle -- Werte siehe Tabellen
            def reg(m_, n):
                f = 440.0 * 2 ** ((n - 58) / 12); clock = 1108405
                while clock * m_ / (256 * f) > 126.5: f *= 2
                while clock * m_ / (256 * f) < 0.5: f /= 2
                k = max(1, min(127, round(clock * m_ / (256 * f))))
                return 0x80 | (127 - k)
            src = open('lib/music.inc').read()
            pats = [[int(x) for x in re.findall(r'\d+', l)] for l in src.split('song_patterns:')[1].strip().split('\n')]
            order = [tuple(int(x) for x in re.findall(r'\d+', l)) for l in src.split('song:')[1].split('song_patterns:')[0].strip().split('\n')[1:]]
            a, b_, c = order[3]
            n1, n2, n3 = pats[a][0], pats[b_][0], pats[c][0]
            got = m.peek(0x900a, 3)
            exp = [reg(1, n2) if 0 < n2 < 96 else 0, reg(2, n3) if 0 < n3 < 96 else 0, reg(4, n1) if 0 < n1 < 96 else 0]
            check('mus_tick: Zeilenbeginn Takt 3 -> Bass/Alt/Sopran = %s' % [hex(x) for x in exp], got == exp, str([hex(x) for x in got]))
            m.poke(0x900a, [0, 0, 0, 0]); zp(m, 'mus_tk', 0); zp(m, 'mus_row', 0)
    finally:
        m.close()
    bad = [r for r in res if not r[1]]
    print('%d bestanden, %d fehlgeschlagen' % (len(res) - len(bad), len(bad)))
    sys.exit(1 if bad else 0)
main()
