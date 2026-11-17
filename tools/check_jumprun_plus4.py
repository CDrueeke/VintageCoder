#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Prüft, dass das gebaute Plus/4-Beispiel jumprun jedes der vier Bilder so zeichnet, wie es die Kunstdaten (tools/jumprun_art.py) verlangen.

Das Programm läuft in VICE (xplus4, nur eine Instanz; sie wird danach beendet; TCP-Port 6512 muss frei sein). Über den Binary-Monitor hält der Läufer beim ersten
Aufruf von hero_update je Bild an und liest Bildschirm ($0C00, 1000 Zeichen) und Farb-RAM ($0800, 1000 Attribut-Bytes). Die Zeichencodes rechnet das Skript aus den
Kunstdaten (Kachel t: Zeichen 1+(t-1)*4+k, Kachel 0: Zeichen 0). Die Farben sind je Zeichen festgelegt (Tabelle char_col in lib/levels.inc); sie werden mit der Tabelle aus
dem Speicher des laufenden Programms (char_col) verglichen. Mit --ref ref.json (vorher mit --dump geschrieben) werden Bildschirm und Farb-RAM zusätzlich byte-genau
mit einem früheren Lauf verglichen. Zellen, in denen ein Zeichen-Sprite (Zeichen $E0 und höher) steht, bleiben außen vor. Verglichen wird ohne Statuszeile (Zeile 0, Spalten 1-33) und Laufschrift (Zeile 23, Spalten 7-31).
Aufruf (im Repository-Wurzelverzeichnis, nach `vcc build examples/plus4/jumprun`):
    python3 tools/check_jumprun_plus4.py [--prg build/jumprun.prg] [--sym build/jumprun.sym.txt] [--vice pfad/zu/xplus4] [--dump ref.json] [--ref ref.json]
Exit-Code 0 = alle vier Bilder stimmen."""
import argparse, glob, json, os, shutil, socket, struct, subprocess, sys, time
sys.path.insert(0, os.path.dirname(__file__))
import jumprun_art as A
HERE = os.path.join(os.path.dirname(__file__), '..', 'examples', 'plus4', 'jumprun')
PORT = 6512

def args():
    ap = argparse.ArgumentParser(description=__doc__.split('\nAufruf')[0], formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--prg', default=os.path.join(HERE, 'build', 'jumprun.prg'), help='gebautes Programm')
    ap.add_argument('--sym', default=os.path.join(HERE, 'build', 'jumprun.sym.txt'), help='Symboldatei zum Programm')
    ap.add_argument('--vice', default=None, help='Pfad zu xplus4 (Standard: im PATH, sonst /Applications/vice*/bin/xplus4)')
    ap.add_argument('--dump', default=None, metavar='DATEI', help='gelesene Rohdaten als JSON schreiben')
    ap.add_argument('--ref', default=None, metavar='DATEI', help='frühere Rohdaten (--dump) byte-genau vergleichen')
    a = ap.parse_args()
    a.vice = a.vice or shutil.which('xplus4') or next(iter(sorted(glob.glob('/Applications/vice*/bin/xplus4'))), None)
    if not a.vice: sys.exit('xplus4 nicht gefunden (--vice angeben)')
    return a

def run_vice(prg, sym, vice, _=True):
    addr = {}
    for l in open(sym, encoding='utf-8'):
        p = l.split()
        if len(p) >= 3 and not l.startswith(';'): addr[p[0]] = int(p[1].lstrip('$'), 16)
    proc = subprocess.Popen([vice, '-binarymonitor', '-binarymonitoraddress', f'ip4://127.0.0.1:{PORT}', '-warp', '+sound', '-autostart', os.path.abspath(prg)],
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        s = None
        for _ in range(100):
            try: s = socket.create_connection(('127.0.0.1', PORT)); break
            except OSError: time.sleep(0.2)
        if not s: sys.exit('keine Verbindung zum VICE-Monitor')
        s.settimeout(60); rid = [0]; buf = [b'']
        def rd(n):
            while len(buf[0]) < n:
                d = s.recv(65536)
                if not d: raise EOFError
                buf[0] += d
            r, buf[0] = buf[0][:n], buf[0][n:]; return r
        def resp():
            h = rd(12); ln, typ, err, rq = struct.unpack('<IBBI', h[2:12]); return typ, err, rq, rd(ln)
        def cmd(c, body=b''):
            rid[0] += 1; s.sendall(struct.pack('<BBIIB', 2, 2, len(body), rid[0], c) + body)
            while True:
                typ, err, rq, b = resp()
                if rq == rid[0]: return typ, err, b
        def mem(a, e):
            t, err, b = cmd(0x01, struct.pack('<BHHBH', 0, a, e, 0, 0)); assert err == 0, err
            return list(b[2:2 + struct.unpack('<H', b[:2])[0]])
        cmd(0x12, struct.pack('<HHBBBB', addr['hero_update'], addr['hero_update'], 1, 1, 4, 0)); cmd(0xaa)
        res = {}
        while len(res) < 4:
            while resp()[0] != 0x62: pass
            lvl = mem(addr['lvl'], addr['lvl'])[0]
            if str(lvl) not in res:
                res[str(lvl)] = {'screen': mem(0x0c00, 0x0fe7), 'color': mem(0x0800, 0x0be7)}
            cmd(0xaa)
        res['char_col'] = mem(addr['char_col'], addr['char_col'] + 255)
        return res
    finally:
        proc.kill()

def expected(i):
    scr = [0] * 1000
    for k, row in enumerate(A.LEVELS[i]()):
        for c, sym in enumerate(row):
            t = A.TILE_INDEX[sym]
            for j in range(4): scr[(2 * k + j // 2) * 40 + 2 * c + j % 2] = 1 + (t - 1) * 4 + j if t else 0
    return scr

def main():
    a = args()
    got = run_vice(a.prg, a.sym, a.vice, True)
    if a.dump: json.dump(got, open(a.dump, 'w'))
    ref = json.load(open(a.ref)) if a.ref else None
    cells0 = list(range(40, 920)) + [0] + list(range(34, 40)) + list(range(920, 927)) + list(range(952, 960))
    ok = True
    for i in range(4):
        scr = expected(i); g = got[str(i)]
        cells = [n for n in cells0 if g['screen'][n] < 0xe0 and scr[n] < 0xe0]       # Zellen der Zeichen-Sprites (Pool $E0 und höher) ausnehmen
        bad_s = [n for n in cells if g['screen'][n] != scr[n]]
        bad_c = [n for n in cells if g['color'][n] != got['char_col'][g['screen'][n]]]
        msg = f"Bild {i+1}: Zeichen {'ok' if not bad_s else f'{len(bad_s)} Abweichungen, erste Zelle {bad_s[0]}'}, Farbe je Zeichen {'ok' if not bad_c else f'{len(bad_c)} Abweichungen, erste Zelle {bad_c[0]}'}"
        ok &= not (bad_s or bad_c)
        if ref:
            r = ref[str(i)]; d1 = [n for n in cells if g['screen'][n] != r['screen'][n]]; d2 = [n for n in cells if g['color'][n] != r['color'][n]]
            msg += f", gegen Vorlauf: Zeichen {'gleich' if not d1 else len(d1)}, Farbe {'gleich' if not d2 else len(d2)}"; ok &= not (d1 or d2)
        print(msg)
    print('ALLE BILDER STIMMEN' if ok else 'ABWEICHUNG')
    sys.exit(0 if ok else 1)
if __name__ == '__main__': main()
