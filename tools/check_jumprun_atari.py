#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Prüft, dass das gebaute Atari-Beispiel jumprun jedes der vier Bilder so zeichnet, wie es die Kunstdaten (tools/jumprun_art.py) verlangen.

Das Programm läuft in atari800 (Text-Monitor über stdin/stdout, nur eine Instanz; sie wird danach beendet). Der Läufer setzt einen Haltepunkt auf hero_update
und liest bei jedem Halt den Bildschirmspeicher ($4000, 24 x 40 Zeichencodes) und die Bildnummer (lvl); sind alle vier Bilder gelesen, endet der Lauf.
Die Sollwerte rechnet das Skript selbst aus den Kunstdaten (Kachel t: Zeichen 1+(t-1)*4+k, mit gesetztem Bit 7 bei Gras und Busch, Kachel 0: Zeichen 0).
Verglichen werden die Zeilen 1 bis 22 ganz sowie Zeile 0 (Spalte 0 und 34-39) und Zeile 23 (Spalten 0-6 und 32-39); den Rest schreibt die HAL (Statuszeile, Laufschrift).
Aufruf (im Repository-Wurzelverzeichnis, nach `vcc build examples/atari/jumprun`):
    python3 tools/check_jumprun_atari.py [--xex build/jumprun.xex] [--sym build/jumprun.sym.txt] [--atari800 pfad/zu/atari800] [--dump ref.json]
Exit-Code 0 = alle vier Bilder stimmen. Mit --dump werden die gelesenen Bildschirme als JSON geschrieben (Vergleichsdatei für Umbauten)."""
import argparse, json, os, re, shutil, subprocess, sys, threading, time, queue
sys.path.insert(0, os.path.dirname(__file__))
import jumprun_art as A
HERE = os.path.join(os.path.dirname(__file__), '..', 'examples', 'atari', 'jumprun')
GREEN = {'G', 'b'}                    # Gras und Busch tragen Zeichenbit 7 (COLPF3)
SENT = '32639'                        # Ausgabe von `dec 7f7f`: Ende der Antwort eines Befehls

def args():
    ap = argparse.ArgumentParser(description=__doc__.split('\nAufruf')[0], formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--xex', default=os.path.join(HERE, 'build', 'jumprun.xex'), help='gebautes Programm (Standard: examples/atari/jumprun/build/jumprun.xex)')
    ap.add_argument('--sym', default=None, help='Symboldatei (Standard: neben dem Programm, jumprun.sym.txt)')
    ap.add_argument('--atari800', default=None, help='Pfad zu atari800 (Standard: im PATH)')
    ap.add_argument('--dump', default=None, metavar='DATEI', help='gelesene Bildschirme als JSON schreiben')
    ap.add_argument('--timeout', type=int, default=120, help='Zeitlimit in Sekunden (Standard 120)')
    a = ap.parse_args()
    a.sym = a.sym or os.path.join(os.path.dirname(a.xex), 'jumprun.sym.txt')
    a.atari800 = a.atari800 or shutil.which('atari800') or ('/opt/homebrew/bin/atari800' if os.path.exists('/opt/homebrew/bin/atari800') else None)
    if not a.atari800: sys.exit('atari800 nicht gefunden (--atari800 angeben)')
    return a

def symbols(path):
    addr = {}
    for l in open(path, encoding='utf-8'):
        p = l.split()
        if len(p) >= 3 and not l.startswith(';') and p[1].startswith('$'): addr[p[0]] = int(p[1][1:], 16)
    return addr

class Mon:
    def __init__(self, exe, xex):
        self.p = subprocess.Popen([exe, '-xl', '-pal', '-no-autosave-config', '-nobasic', '-monitor', '-run', os.path.abspath(xex)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, bufsize=1)
        self.q = queue.Queue()
        threading.Thread(target=self._rd, daemon=True).start()
    def _rd(self):
        for l in self.p.stdout: self.q.put(re.sub(r'\x1b\[[0-9;]*[A-Za-z]', '', l.rstrip('\n')))
    def send(self, c): self.p.stdin.write(c + '\n'); self.p.stdin.flush()
    def lines_until(self, pred, timeout):
        end = time.time() + timeout; out = []
        while time.time() < end:
            try: l = self.q.get(timeout=0.2)
            except queue.Empty: continue
            out.append(l)
            if pred(l): return out
        raise TimeoutError('atari800 antwortet nicht: ' + ' | '.join(out[-5:]))
    def exec(self, c):
        self.send(c); self.send('dec 7f7f')
        o = self.lines_until(lambda l: SENT in l, 8); return o[:-1]
    def mem(self, a, n):
        out = []
        while len(out) < n:
            for l in self.exec(f'm {a + len(out):04x}'):
                h, _, rest = l.partition(':')
                if len(h) == 4 and re.fullmatch(r'[0-9a-fA-F]{4}', h) and int(h, 16) == a + len(out):
                    out += [int(x, 16) for x in rest.split()[:16] if re.fullmatch(r'[0-9a-fA-F]{2}', x)]; break
            else: raise RuntimeError('Speicher nicht lesbar')
        return out[:n]
    def stop(self):
        try: self.p.kill(); self.p.wait(5)
        except Exception: pass

def run(a):
    addr = symbols(a.sym)
    m = Mon(a.atari800, a.xex)
    try:
        m.lines_until(lambda l: ' PC=' in l and 'A=' in l, 20)
        m.exec(f"bpc {addr['hero_update']:04x}")
        res, end = {}, time.time() + a.timeout
        while len(res) < 4 and time.time() < end:
            m.send('cont')
            m.lines_until(lambda l: l.startswith('(breakpoint at') or ('PC=' in l and 'A=' in l), 60)
            lvl = m.mem(addr['lvl'], 1)[0]
            if str(lvl) not in res: res[str(lvl)] = {'screen': m.mem(0x4000, 960)}
        if len(res) < 4: sys.exit(f'nur {len(res)} von 4 Bildern gelesen (Zeitlimit)')
        return res
    finally:
        m.stop()

def expected(i):
    scr = [0] * 960
    for k, row in enumerate(A.LEVELS[i]()):
        for c, sym in enumerate(row):
            t = A.TILE_INDEX[sym]
            for j in range(4):
                scr[(2 * k + j // 2) * 40 + 2 * c + j % 2] = ((1 + (t - 1) * 4 + j) | (0x80 if sym in GREEN else 0)) if t else 0
    return scr

def main():
    a = args()
    got = run(a)
    if a.dump: json.dump(got, open(a.dump, 'w'))
    ok = True
    cells = list(range(40, 920)) + [0] + list(range(34, 40)) + list(range(920, 927)) + list(range(952, 960))
    for i in range(4):
        scr = expected(i); g = got[str(i)]['screen']
        bad = [n for n in cells if g[n] != scr[n]]
        print(f"Bild {i+1}: Bildschirm {'ok' if not bad else f'{len(bad)} Abweichungen, erste Zelle {bad[0]} (ist {g[bad[0]]}, soll {scr[bad[0]]})'}")
        ok &= not bad
    print('ALLE BILDER STIMMEN' if ok else 'ABWEICHUNG')
    sys.exit(0 if ok else 1)
if __name__ == '__main__': main()
