#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Prüft, dass das gebaute C64-Beispiel jumprun jedes der vier Bilder so zeichnet, wie es die Kunstdaten (tools/jumprun_art.py) verlangen.

Das Programm läuft in VICE (x64sc, nur eine Instanz; sie wird danach beendet). Über den Binary-Monitor hält der Läufer beim ersten Aufruf von
hero_update je Bild an und liest Bildschirm-RAM ($0400-$07E7), Farb-RAM ($D800-$DBE7) sowie $D022/$D023 und die Himmelsfarben (skyc).
Die Sollwerte rechnet das Skript selbst aus den Kunstdaten (Kachel t: Zeichen 1+(t-1)*4+k, Farbe (eigene Farbe | 8); Kachel 0: Zeichen 0, Farbe 0,
wie die frühere hal_put_tile). Bildschirmzeile 0 und 24 (Statuszeile, Laufschrift) schreibt die HAL; von Zeile 0 gilt nur Spalte 0 und 34-39.
Aufruf (im Repository-Wurzelverzeichnis, nach `vcc build examples/c64/jumprun`):
    python3 tools/check_jumprun_c64.py [--prg build/jumprun.prg] [--sym build/jumprun.sym.txt] [--vice pfad/zu/x64sc] [--dump ref.json (schreibt die Rohdaten)]
Exit-Code 0 = alle vier Bilder stimmen."""
import argparse, glob, json, os, shutil, socket, struct, subprocess, sys, time
sys.path.insert(0, os.path.dirname(__file__))
import jumprun_art as A
HERE = os.path.join(os.path.dirname(__file__), '..', 'examples', 'c64', 'jumprun')
PORT = 6511

def args():
    ap = argparse.ArgumentParser(
        description="""Prüft, dass das gebaute C64-Beispiel jumprun jedes der vier Bilder so zeichnet, wie es die Kunstdaten (tools/jumprun_art.py) verlangen.

Das Programm läuft in VICE (x64sc, nur eine Instanz; sie wird danach beendet; TCP-Port 6511 muss frei sein). Über den Binary-Monitor hält der Läufer beim ersten
Aufruf von hero_update je Bild an; gelesen werden Bildschirm-RAM, Farb-RAM, $D022/$D023 und die Himmelsfarben und mit den aus den Kunstdaten berechneten Sollwerten
verglichen. Voraussetzung: das Beispiel ist gebaut (vcc build examples/c64/jumprun). Es wird nichts geschrieben, außer mit --dump (Rohdaten als JSON).
Exit-Code 0 = alle vier Bilder stimmen. Die Standardpfade gelten relativ zum Ort des Skripts; der Arbeitsordner ist nur für relativ angegebene Optionspfade wichtig.""",
        epilog="""Beispiele (im Repository-Wurzelordner, nach `vcc build examples/c64/jumprun`):
  python3 tools/check_jumprun_c64.py
  python3 tools/check_jumprun_c64.py --vice /Applications/vice-arm64-gtk3-3.10/bin/x64sc --dump ref.json""",
        formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--prg', default=os.path.join(HERE, 'build', 'jumprun.prg'), help='gebautes Programm (Standard: examples/c64/jumprun/build/jumprun.prg)')
    ap.add_argument('--sym', default=os.path.join(HERE, 'build', 'jumprun.sym.txt'), help='Symboldatei zum Programm (Standard: examples/c64/jumprun/build/jumprun.sym.txt)')
    ap.add_argument('--vice', default=None, help='Pfad zu x64sc (Standard: x64sc im PATH, sonst /Applications/vice*/bin/x64sc)')
    ap.add_argument('--dump', default=None, metavar='DATEI', help='die gelesenen Rohdaten als JSON in diese Datei schreiben (Standard: nicht schreiben)')
    pa = ap.parse_args()
    a = {'--prg': pa.prg, '--sym': pa.sym, '--vice': pa.vice, '--dump': pa.dump}
    if not a['--vice']:
        a['--vice'] = shutil.which('x64sc') or next(iter(sorted(glob.glob('/Applications/vice*/bin/x64sc'))), None)
    if not a['--vice']: sys.exit('x64sc nicht gefunden (--vice angeben)')
    return a

def run_vice(prg, sym, vice):
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
                res[str(lvl)] = {'screen': mem(0x0400, 0x07e7), 'color': [c & 15 for c in mem(0xd800, 0xdbe7)], 'mc': [mem(0xd022, 0xd022)[0] & 15, mem(0xd023, 0xd023)[0] & 15],
                                 'skyc': mem(addr['skyc'], addr['skyc'] + 5)}
            cmd(0xaa)
        return res
    finally:
        proc.kill()

def expected(i):
    scr, col = [0] * 1000, [0] * 1000
    for k, row in enumerate(A.LEVELS[i]()):
        for c, sym in enumerate(row):
            t = A.TILE_INDEX[sym]
            for j in range(4):
                cell = (2 * k + j // 2) * 40 + 2 * c + j % 2
                scr[cell] = 1 + (t - 1) * 4 + j if t else 0
                col[cell] = (A.TILES[t][5] | 8) if t else 0
    return scr, col

def main():
    a = args()
    got = run_vice(a['--prg'], a['--sym'], a['--vice'])
    if a['--dump']: json.dump(got, open(a['--dump'], 'w'))
    pal = [(9, 8, [3, 3, 3, 3, 3, 14]), (11, 12, [0] * 6), (2, 10, [6, 6, 14, 14, 14, 14]), (14, 15, [6, 6, 6, 6, 6, 0])]
    ok = True
    for i in range(4):
        scr, col = expected(i); g = got[str(i)]
        cells = [n for n in range(40, 960)] + [0] + list(range(34, 40))
        bad_s = [n for n in cells if g['screen'][n] != scr[n]]; bad_c = [n for n in cells if g['color'][n] != col[n]]
        bad_p = g['mc'] != list(pal[i][:2]) or g['skyc'] != pal[i][2]
        print(f"Bild {i+1}: Bildschirm {'ok' if not bad_s else f'{len(bad_s)} Abweichungen, erste Zelle {bad_s[0]}'}, Farb-RAM {'ok' if not bad_c else f'{len(bad_c)} Abweichungen, erste Zelle {bad_c[0]}'}, Mehrfarb/Himmel {'ok' if not bad_p else 'FALSCH'}")
        ok &= not (bad_s or bad_c or bad_p)
    print('ALLE BILDER STIMMEN' if ok else 'ABWEICHUNG')
    sys.exit(0 if ok else 1)
if __name__ == '__main__': main()
