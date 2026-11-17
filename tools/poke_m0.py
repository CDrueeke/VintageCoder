#!/usr/bin/env python3
"""Messplan M0 des POKE-Designers (Entscheidung 16): Laufzeit von POKE-Formen in VICE.

Nur von Hand starten, nie in der CI (startet einen Emulator). Schreibt nichts nach examples/ oder ins Projekt, sondern in einen
Wegwerfordner. Je Variante laeuft 1000 Wiederholungen; das Programm schreibt die verbrauchte Zeit `TI` (Jiffies) in die Datei
`res` auf einer D64 im Wegwerfordner, die danach mit c1541 gelesen wird.

    python3 tools/poke_m0.py [--vcc PFAD] [--vice-dir ORDNER] [--n 1000]

Atari (atari800) ist nicht Teil dieses Skripts: der Emulator braucht Betriebssystem-ROM und einen Schreibweg, den die Messung
nicht ohne Weiteres liefert; die Atari-Zahlen stehen im Konzept als „nicht gemessen“.
"""
import argparse, os, shutil, subprocess, sys, tempfile

VARIANTEN = [
    ("leer (nur Schleife)", "rem"),
    ("poke Konstante", "poke 53280, 1"),
    ("poke peek or", "poke 53280, peek(53280) or 1"),
    ("poke v + 32 (Variable)", "poke v + 32, 1"),
    ("2 poke in 2 Zeilen", "poke 53280, 1\npoke 53281, 1"),
    ("2 poke in 1 Zeile", "poke 53280, 1 : poke 53281, 1"),
    ("2 poke mit Variable", "poke v + 32, 1 : poke v + 33, 1"),
]

def programm(koerper, n):
    return f"v = 53248\nt = ti\nfor i = 1 to {n}\n{koerper}\nnext i\nopen 1, 8, 2, \"res,s,w\"\nprint# 1, ti - t\nclose 1\n"

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--vcc", default=os.path.expanduser("~/.cache/vintagecoder-target-wunschspalte/debug/vcc"))
    ap.add_argument("--vice-dir", default="/Applications/vice-arm64-gtk3-3.10/bin")
    ap.add_argument("--n", type=int, default=1000)
    a = ap.parse_args()
    wurzel = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    x64 = os.path.join(a.vice_dir, "x64sc"); c1541 = os.path.join(a.vice_dir, "c1541")
    for p in (a.vcc, x64, c1541):
        if not os.path.exists(p): sys.exit(f"nicht gefunden: {p}")
    tmp = tempfile.mkdtemp(prefix="poke-m0-")
    erg = []
    for name, koerper in VARIANTEN:
        quelle = os.path.join(tmp, "m.bas"); open(quelle, "w").write(programm(koerper, a.n))
        prg = os.path.join(tmp, "m.prg"); d64 = os.path.join(tmp, "m.d64")
        subprocess.run([a.vcc, "basic", quelle, "--data-dir", os.path.join(wurzel, "data"), "-o", prg], check=True, capture_output=True)
        if os.path.exists(d64): os.remove(d64)
        subprocess.run([c1541, "-format", "m0,01", "d64", d64, "-write", prg, "m"], capture_output=True)
        subprocess.run([x64, "-console", "-warp", "+sound", "-limitcycles", "120000000", "-autostart", d64 + ":m"], capture_output=True, timeout=300)
        aus = os.path.join(tmp, "r.txt")
        if os.path.exists(aus): os.remove(aus)
        subprocess.run([c1541, d64, "-read", "res,s", aus], capture_output=True)
        try: erg.append((name, int(open(aus).read().split()[0])))
        except Exception: erg.append((name, None))
    leer = erg[0][1]
    print(f"{'Variante':28} {'Jiffies':>8} {'je Durchlauf (ohne Schleife)':>30}")
    for name, j in erg:
        if j is None: print(f"{name:28} {'?':>8}"); continue
        d = (j - leer) / a.n if leer is not None else float('nan')
        print(f"{name:28} {j:8d} {d:30.4f}")
    shutil.rmtree(tmp, ignore_errors=True)

if __name__ == "__main__":
    main()
