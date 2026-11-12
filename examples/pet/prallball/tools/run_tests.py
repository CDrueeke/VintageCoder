#!/usr/bin/env python3
"""Testlaeufer fuer PRALLBALL (PET, 40 Spalten, BASIC 4): startet VICE xpet (-model 4032) mit dem Text-Monitor (-remotemonitor), laedt build/prallball.prg und ruft
Unterprogramme einzeln auf (Falle bei $6A00): Speicher und Register setzen, Unterprogramm ausfuehren, Ergebnis pruefen.  Erwartungswerte werden hier unabhaengig
vom Programm berechnet (Richtungstabelle, Viertelblock-Tabelle gegen die Zeichen-ROM-Datei von VICE, BCD-Rechnung).  Dazu kommen Laeufe der Testfassungen
(AUTOPLAY, TESTLOSE) mit Auswertung des Bildschirms und der Bestenliste auf einer Kopie der Diskette.
Grund fuer einen eigenen Laeufer: `vcc test` kennt das Ziel pet (noch) nicht.
Aufruf im Ordner examples/pet/prallball:  python3 tools/run_tests.py   (vorher: vcc build .)   Optionen: --filter TEXT
Voraussetzungen: VICE xpet unter dem Pfad EXE (macOS-Pfad: anpassen), freier TCP-Port PORT, build/prallball.prg und build/prallball.sym.txt, die Zeichen-ROM
characters-2.901447-10.bin im VICE-Datenordner (Pfad CHARROM) und vcc (Pfad VCC, fuer die Testfassungen).  Exit-Code 0 = alles bestanden, 1 = mindestens eine Pruefung fehlgeschlagen."""
import math, os, re, shutil, socket, subprocess, sys, tempfile, time
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\nArbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/pet/prallball (os.chdir). Es schreibt nur in einen Temporaerordner (Kopie der Diskette, Testfassungen) und startet VICE xpet; am Ende wird es beendet.",
    epilog="Beispiele:\n  python3 examples/pet/prallball/tools/run_tests.py\n  cd examples/pet/prallball && python3 tools/run_tests.py --filter hit_cell\n\n-h/--help zeigt nur diesen Text und startet nichts.",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.add_argument('--filter', default='', metavar='TEXT', help='nur Pruefgruppen ausfuehren, deren Name TEXT enthaelt (Standard: alle)')
_args = _cli.parse_args()
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
EXE = '/Applications/vice-arm64-gtk3-3.10/bin/xpet'
CHARROM = '/Applications/vice-arm64-gtk3-3.10/VICE.app/Contents/Resources/share/vice/PET/characters-2.901447-10.bin'
VCC = os.environ.get('VCC', '/tmp/vc-target/debug/vcc')
PORT = 6510
TRAP = 0x6a00
TMP = tempfile.mkdtemp(prefix='pet_tests_')
DISK = os.path.join(TMP, 'disk.d64')
shutil.copy('build/disk.d64', DISK)
SYM = {}
for line in open('build/prallball.sym.txt', encoding='utf-8'):
    p = line.split()
    if len(p) >= 2 and p[1].startswith('$') and not line.startswith(';'): SYM[p[0]] = int(p[1][1:], 16)
def A(x): return SYM[x] if isinstance(x, str) else x

class Mon:
    """Verbindung zum VICE-Text-Monitor: startet xpet, sendet Monitorbefehle und liest die Antworten."""
    def __init__(self):
        self.p = subprocess.Popen([EXE, '-model', '4032', '-console', '-silent', '-warp', '+sound', '-8', DISK, '-remotemonitor', '+confirmonexit'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
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
        """xpet beenden."""
        try: self.cmd('quit', 1)
        except Exception: pass
        self.p.kill()


FILT = _args.filter
res = []
def check(name, ok, info=''):
    res.append((name, ok, info))
    print(('  OK       ' if ok else '  FEHLER   ') + name + ('' if ok else '  ' + info))
def want(name): return FILT in name
def zp(m, var, val): m.poke(var, val)
def loadprg(m, path='build/prallball.prg'):
    o = m.cmd('load "%s" 0' % os.path.abspath(path), 10); assert 'Loading' in o, o
def build_variant(name, defs):
    """Testfassung bauen: Projekt in einen Temporaerordner kopieren, die Schalter als .const voranstellen und mit vcc build uebersetzen.
    Liefert (prg, symbole): die Symbole der Fassung (Adressen verschieben sich gegenueber der Hauptfassung)."""
    d = os.path.join(TMP, name)
    shutil.copytree('.', d, ignore=shutil.ignore_patterns('build', 'tools', '__pycache__'))
    src = open(os.path.join(d, 'prallball.asm'), encoding='utf-8').read()
    pre = ''.join('.const %s = %s\n' % tuple(x.split('=')) for x in defs)
    open(os.path.join(d, 'prallball.asm'), 'w', encoding='utf-8').write(pre + src)
    subprocess.run([VCC, 'build', d], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    sym = {}
    for line in open(os.path.join(d, 'build', 'prallball.sym.txt'), encoding='utf-8'):
        p = line.split()
        if len(p) >= 2 and p[1].startswith('$') and not line.startswith(';'): sym[p[0]] = int(p[1][1:], 16)
    return os.path.join(d, 'build', 'prallball.prg'), sym
def screen(m): return m.peek(0x8000, 1000)
def row(scr, r): return scr[r * 40:(r + 1) * 40]
def text(scr, r):
    return ''.join(chr(c + 64) if 1 <= c <= 26 else (chr(c) if 32 <= c < 96 else '.') for c in row(scr, r))

# ---- Richtungstabelle unabhaengig neu berechnen (gleiche Formel wie tools/gen.py)
ANG = [-60, -43, -26, -9, 9, 26, 43, 60]
def vel(level, zone):
    mag = 0.70 + 0.08 * level
    a = math.radians(ANG[zone])
    vx = round(mag * math.sin(a) * 256); vy = round(-mag * math.cos(a) * 256)
    return vx & 0xffff, vy & 0xffff

def main():
    m = Mon()
    try:
        loadprg(m)
        P = SYM
        # ---- statische Pruefungen
        end = SYM['level_tab_hi'] + 22
        check('Programmende liegt unter der Levelkarte ($6B00)', end < 0x6b00, hex(end))
        if want('qtab'):
            d = open(CHARROM, 'rb').read()
            def glyph(c): return [d[(c & 127) * 8 + r] ^ (0xff if c & 128 else 0) for r in range(8)]
            qt = m.peek('qtab', 16)
            for mask in range(16):
                want_rows = []
                for r in range(8):
                    left = 0xf0 if mask & (1 if r < 4 else 4) else 0
                    right = 0x0f if mask & (2 if r < 4 else 8) else 0
                    want_rows.append(left | right)
                check('qtab[%d] = Bildschirmcode %d zeigt die Viertelblock-Maske %s' % (mask, qt[mask], format(mask, '04b')), glyph(qt[mask]) == want_rows, str(glyph(qt[mask])))
        # ---- add_score
        if want('add_score'):
            zp(m, 'sc0', [0x99, 0x09, 0x00]); m.call('add_score', a=0x10)
            check('add_score: 0999 + 10 = 1009', m.peek('sc0', 3) == [0x09, 0x10, 0x00], str(m.peek('sc0', 3)))
            zp(m, 'sc0', [0x95, 0x99, 0x00]); m.call('add_score', a=0x10)
            check('add_score: Uebertrag in die hoechsten Ziffern', m.peek('sc0', 3) == [0x05, 0x00, 0x01])
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
            for lvl, zone in ((0, 3), (5, 0), (7, 7), (2, 2), (6, 4), (1, 5)):
                zp(m, 'spdlvl', lvl); m.call('set_vel', a=zone)
                vx, vy = vel(lvl, zone); got = m.peek('vxl', 4)
                check('set_vel: Stufe %d Zone %d' % (lvl, zone), got == [vx & 255, vx >> 8, vy & 255, vy >> 8], '%s statt %s' % (got, [vx & 255, vx >> 8, vy & 255, vy >> 8]))
        # ---- Zellen und Overlay
        if want('cell'):
            m.poke('OV', [0] * 1000); m.poke(0x8000, [0x20] * 40)
            r = m.call('setcell', x=5, y=3)
            check('setcell: Spalte 5, Zeile 3 -> Overlay $6C00 + 3 * 40 + 5, Bildschirm $8000 + 125', m.peek('cp', 2) == [(0x6c00 + 125) & 255, (0x6c00 + 125) >> 8] and m.peek('sp', 2) == [125, 0x80])
            m.call('ov_or', a=4); scr = m.peek(0x8000 + 125)[0]; ov = m.peek(0x6c00 + 125)[0]
            check('ov_or: Bit 2 (unten links) -> Zeichen 123', ov == 4 and scr == 123, '%d/%d' % (ov, scr))
            m.call('ov_or', a=8); scr = m.peek(0x8000 + 125)[0]
            check('ov_or: Bit 3 dazu -> untere Haelfte (Zeichen 98)', scr == 98, str(scr))
            m.call('ov_clr', a=4); scr = m.peek(0x8000 + 125)[0]
            check('ov_clr: Bit 2 weg -> unten rechts (Zeichen 108)', scr == 108, str(scr))
            m.call('ov_clr', a=8); scr = m.peek(0x8000 + 125)[0]; ov = m.peek(0x6c00 + 125)[0]
            check('ov_clr: letztes Bit weg -> Leerzeichen', scr == 32 and ov == 0)
            m.poke(0x8000 + 125, [0xa0]); r = m.call('cell_static', x=5, y=3)
            check('cell_static: Vollblock = fest (C=1)', r['c'])
            m.poke(0x8000 + 125, [0x20]); r = m.call('cell_static', x=5, y=3)
            check('cell_static: Leerzeichen = frei (C=0)', not r['c'])
            m.poke(0x8000 + 125, [0x7b]); m.poke(0x6c00 + 125, [4]); r = m.call('cell_static', x=5, y=3)
            check('cell_static: Zelle mit Ball oder Schlaeger = frei', not r['c'])
            m.poke(0x8000 + 125, [0x20]); m.poke(0x6c00 + 125, [0])
        # ---- paddle_test: Schlaeger px = 20, pw = 12, Ball bxh variiert; Zone = rel * 8 / (pw + 2), rel = bx - px + 1
        if want('paddle_test'):
            for bx in (19, 20, 22, 26, 31, 32, 33, 18, 34):
                zp(m, 'px', 20); zp(m, 'pw', 12); zp(m, 'spdlvl', 0); m.poke('bxh', [bx]); m.poke('t1', [47]); zp(m, 'byh', 47)
                r = m.call('paddle_test')
                rel = bx - 20 + 1
                hit = 0 <= rel <= 13
                zone = min(7, rel * 8 // 14) if hit else None
                if hit:
                    vx, vy = vel(0, zone)
                    ok = r['c'] and m.peek('vxl', 4) == [vx & 255, vx >> 8, vy & 255, vy >> 8] and m.peek('byh')[0] == 46
                    check('paddle_test: Ball bei %d (rel %d) -> Zone %d, Ball auf Zeile 46' % (bx, rel, zone), ok, '%s %s' % (r, m.peek('vxl', 4)))
                else:
                    check('paddle_test: Ball bei %d daneben (rel %d)' % (bx, rel), not r['c'])
        # ---- Steine: Levelkarte und Zeichnen
        if want('load_level'):
            zp(m, 'level', 0); m.call('clear_screen'); m.call('load_level')
            lv = m.peek('LV', 216)
            check('load_level: Level 1 hat 7 Reihen zu 18 Steinen (a..f und Gold)', sum(1 for t in lv if t) == 126 and m.peek('bricks')[0] == 126, str(m.peek('bricks')))
            scr = screen(m)
            check('load_level: Stein a (Typ 1) an Zeile 5 sitzt auf Spalte 2/3 mit $A0/$EC', row(scr, 5)[2] == 0xa0 and row(scr, 5)[3] == 0xec, str(row(scr, 5)[:6]))
            check('load_level: Stein b (Typ 2) an Zeile 6 zeigt Raster $66', row(scr, 6)[2] == 0x66 and row(scr, 6)[3] == 0x66)
            check('load_level: Gold (Typ 10) an Zeile 11 zeigt $E6', row(scr, 11)[2] == 0xe6)
            zp(m, 'level', 2); m.call('load_level')
            lv3 = m.peek('LV', 216)
            check('load_level: Level 3 enthaelt unzerstoerbare Steine (11), sie zaehlen nicht', 11 in lv3 and m.peek('bricks')[0] == sum(1 for t in lv3 if t not in (0, 11)), str(m.peek('bricks')))
        # ---- hit_cell: Zelle (Spalte 2 + 2 * bc, Zeile 3 + br); Stein in der Levelkarte setzen
        if want('hit_cell'):
            def setup(types):
                m.call('clear_screen'); m.poke('LV', [0] * 216)
                for (br, bc), t in types.items(): m.poke('LV', 0) if False else m.poke(A('LV') + br * 18 + bc, [t])
                m.poke('bricks', [20]); zp(m, 'sc0', [0, 0, 0]); zp(m, 'hits', [0]); zp(m, 'lvdone', [0]); m.poke('spdlvl', [0])
            for t, nxt, pts, nm in ((1, 0, 0x10, 'a: zerstoert, 10 Punkte'), (2, 0, 0x20, 'b: zerstoert, 20 Punkte'), (8, 0, 0x80, 'h: zerstoert, 80 Punkte'),
                                    (9, 1, 0, 'Silber wird einfacher Stein'), (10, 9, 0, 'Gold wird Silber'), (11, 11, 0, 'unzerstoerbar bleibt')):
                setup({(3, 4): t}); m.call('hit_cell', x=2 + 2 * 4, y=3 + 3)       # linke Zelle des Steins (Spalte 4, Zeile 3)
                got = m.peek(A('LV') + 3 * 18 + 4)[0]; b = m.peek('bricks')[0]
                want_b = 19 if t < 9 else 20
                check('hit_cell: Typ %d (%s)' % (t, nm), got == nxt and b == want_b and m.peek('sc0')[0] == pts, 'Typ jetzt %d, Steine %d, Punkte %x' % (got, b, m.peek('sc0')[0]))
            setup({(3, 4): 1}); m.call('hit_cell', x=2 + 2 * 4 + 1, y=6)          # rechte Zelle desselben Steins zaehlt auch
            check('hit_cell: rechte Zelle des Steins trifft den Stein', m.peek(A('LV') + 3 * 18 + 4)[0] == 0)
            scr = screen(m); check('hit_cell: zerstoerter Stein verschwindet vom Bildschirm', scr[6 * 40 + 10] == 0x20 and scr[6 * 40 + 11] == 0x20)
            setup({(0, 0): 1}); m.poke('bricks', [1]); m.call('hit_cell', x=2, y=3)
            check('hit_cell: letzter Stein -> lvdone = 1', m.peek('lvdone')[0] == 1)
            setup({(3, 4): 12, (3, 3): 1, (2, 4): 2, (4, 5): 3, (3, 6): 11, (3, 2): 1}); m.call('hit_cell', x=10, y=6)
            lvn = lambda br, bc: m.peek(A('LV') + br * 18 + bc)[0]
            check('hit_cell: explosiver Stein zerstoert die Nachbarn, nicht den unzerstoerbaren und nicht die ferneren',
                  lvn(3, 4) == 0 and lvn(3, 3) == 0 and lvn(2, 4) == 0 and lvn(4, 5) == 0 and lvn(3, 6) == 11 and lvn(3, 2) == 1, str([lvn(3, 4), lvn(3, 3), lvn(2, 4), lvn(4, 5), lvn(3, 6), lvn(3, 2)]))
            # Wand: kein Stein -> Wandton, nichts veraendert
            setup({}); m.call('hit_cell', x=0, y=10)
            check('hit_cell: Wandzelle aendert die Levelkarte nicht', m.peek('LV', 216) == [0] * 216)
        # ---- Eingabe: $97 = gehaltene Taste
        if want('input'):
            m.call('input_init')
            for key, mask, nm in ((0x5a, 4, 'Z = links'), (0x2c, 4, ', = links'), (0x58, 8, 'X = rechts'), (0x2e, 8, '. = rechts'), (0x20, 0x10, 'Leertaste = Feuer'), (0x50, 0x20, 'P = Pause'),
                                  (0x03, 0x40, 'RUN/STOP = Abbruch'), (0x41, 2, 'A = runter'), (0x51, 1, 'Q = hoch'), (0xff, 0, 'keine Taste'), (0x44, 0, 'unbelegte Taste')):
                m.call('input_init'); m.poke(0x97, [key]); r = m.call('input_poll')
                check('input_poll: %s' % nm, r['a'] == mask, '%02x' % r['a'])
            m.call('input_init'); m.poke(0x97, [0x20]); a1 = m.call('input_poll')['a']; a2 = m.call('input_poll')['a']
            check('input_poll: Feuer ist eine Flanke (gehalten = 0)', a1 == 0x10 and a2 == 0)
        # ---- Klang: sfx_beep stellt VIA ein; sfx_tick schaltet nach der Dauer aus
        if want('sfx'):
            m.call('sfx_beep', a=90, x=2)
            check('sfx_beep: Hilfssteuerung $10 (Schiebetakt aus Zeitgeber 2), Dauer 2 Bilder', m.peek(0xe84b)[0] == 0x10 and m.peek('snd_t')[0] == 2)
            m.call('sfx_tick'); m.call('sfx_tick')
            check('sfx_tick: nach 2 Bildern ist der Ton aus ($E84B = 0)', m.peek(0xe84b)[0] == 0)
        # ---- Bestenliste
        if want('hs_'):
            m.call('hs_default'); check('hs_default: erster Eintrag SPIELER, 1000 Punkte', m.peek('hs_buf', 21)[5:13] == list(b'SPIELER ') and m.peek(A('hs_buf') + 5 + 8, 3) == [0, 0x10, 0])
            m.poke('hs_points', [0x34, 0x12, 0x00]); r = m.call('hs_check'); check('hs_check: 1234 Punkte -> Platz 0', r['a'] == 0)
            m.poke('hs_points', [0x50, 0x00, 0x00]); r = m.call('hs_check'); check('hs_check: 50 Punkte -> nicht in der Liste ($FF)', r['a'] == 0xff)
            m.poke('hs_points', [0x34, 0x12, 0x00]); m.poke('hs_level', [5]); m.poke('hs_name', list(b'ABCDEFGH')); m.call('hs_default'); m.call('hs_insert', a=0)
            check('hs_insert: Eintrag an Platz 0, alter erster Eintrag rutscht auf 1', m.peek(A('hs_buf') + 5, 12) == list(b'ABCDEFGH') + [0x34, 0x12, 0, 5] and m.peek(A('hs_buf') + 17, 8) == list(b'SPIELER '))
        # ---- Interrupt: Bildzaehler
        if want('irq'):
            loadprg(m); m.poke(0x04, [0]); m.poke(0x90, [A('irq') & 255, A('irq') >> 8]); m.poke('oldirq', [0x55, 0xe4])
            m.cmd('del'); m.poke('in_kbclr', [1]); m.poke(0x9e, [3]); m.cmd('r fl=20'); m.s.sendall(b'x\n'); time.sleep(1.0); m.s.sendall(b'\n'); time.sleep(0.3); m.drain()
            check('irq: Bildzaehler laeuft, Tastaturpuffer wird geleert', m.peek(0x04)[0] > 20 and m.peek(0x9e)[0] == 0, '%d/%d' % (m.peek(0x04)[0], m.peek(0x9e)[0]))
        m.close()
    except Exception:
        try: m.close()
        except Exception: pass
        raise

def play_tests():
    """Laeufe der Testfassungen: AUTOPLAY (Spiel laeuft, Steine fallen) und TESTLOSE (Spielende, Namenseingabe, Speichern auf die Diskettenkopie)."""
    if not want('play'): return
    auto, sym = build_variant('auto', ['AUTOPLAY=1']); SYM.clear(); SYM.update(sym)
    m = Mon()
    try:
        m.cmd('load "%s" 0' % auto, 10); m.poke(0xd4, [8]); m.cmd('r pc=040d'); m.s.sendall(b'x\n'); time.sleep(8); m.s.sendall(b'\n'); time.sleep(0.4); m.drain()
        sc = m.peek('sc0', 3); br = m.peek('bricks')[0]
        check('play: AUTOPLAY spielt (nach 8 s Punkte > 0, weniger als 108 Steine)', sc != [0, 0, 0] and br < 108, '%s / %d' % (sc, br))
        scr = screen(m); check('play: Anzeige Zeile 0 beginnt mit PUNKTE', text(scr, 0).startswith('PUNKTE '), text(scr, 0))
        m.close()
    except Exception:
        m.close(); raise
    lose, sym = build_variant('lose', ['AUTOPLAY=1', 'TESTLOSE=1', 'STARTLIVES=1']); SYM.clear(); SYM.update(sym)
    m = Mon()
    try:
        m.cmd('load "%s" 0' % lose, 10); m.poke(0xd4, [8]); m.poke('sc0', [0]); m.cmd('r pc=040d'); m.s.sendall(b'x\n')
        # warten, bis das Spiel im Namenseingabe-Zustand haengt oder zum Titel zurueckkehrt; Punkte vorher nach oben setzen, damit ein Bestenlisten-Platz entsteht
        t0 = time.time(); seen_over = False
        while time.time() - t0 < 60:
            time.sleep(1.5); m.s.sendall(b'\n'); time.sleep(0.3); m.drain()
            m.poke('sc2', [0x01])                                         # 100000 Punkte: Platz 0
            scr = screen(m)
            if 'NEUER REKORD' in ''.join(text(scr, r) for r in range(25)): seen_over = True; break
            m.s.sendall(b'x\n')
        check('play: TESTLOSE -> Spielende, "NEUER REKORD!" erscheint', seen_over)
        if seen_over:
            m.poke(0x9e, [3]); m.poke(0x026f, [ord('K'), ord('A'), 0x0d]); m.cmd('r fl=20'); m.s.sendall(b'x\n'); time.sleep(4); m.s.sendall(b'\n'); time.sleep(0.3); m.drain()
            e0 = m.peek(A('hs_buf') + 5, 8)
            check('play: Namenseingabe K, A, RETURN -> erster Eintrag der Liste heisst "KA"', e0 == list(b'KA      '), str(e0))
            time.sleep(0.1)
        m.close()
    except Exception:
        m.close(); raise
    ls = subprocess.run([VCC, 'disk', 'list', DISK], capture_output=True, text=True).stdout
    check('play: PRALL.HS steht als SEQ-Datei auf der Diskettenkopie', 'PRALL.HS' in ls and 'SEQ' in ls, ls)

if __name__ == '__main__':
    try:
        main(); play_tests()
    finally:
        shutil.rmtree(TMP, ignore_errors=True)
    bad = [r for r in res if not r[1]]
    print('\n%d bestanden, %d fehlgeschlagen' % (len(res) - len(bad), len(bad)))
    sys.exit(1 if bad else 0)
