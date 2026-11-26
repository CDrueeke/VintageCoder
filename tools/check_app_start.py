#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Startparameter gegen die GEBAUTE Anwendung prüfen (macOS und Linux): `vintagecoder <Ordner> --open <Datei>` öffnet das Projekt und trägt es in die Zuletzt-Liste ein,
auch wenn dort schon ein anderes Projekt als letztes steht, und ein zweiter Aufruf bei laufender Anwendung öffnet sein Projekt in der laufenden (Einzelinstanz).
Aufruf: python3 tools/check_app_start.py <Pfad zur Programmdatei, z. B. VintageCoder.app/Contents/MacOS/vintagecoder>   (eigenes HOME, berührt keine Benutzerdaten)"""
import json, os, shutil, subprocess, sys, tempfile, time

def recent(home):
    try: return [r["path"] for r in json.load(open(os.path.join(home, ".config", "vintagecoder", "recent.json")))]
    except Exception: return []

def warte(home, pfad, sek=40):
    ende = time.time() + sek
    while time.time() < ende:
        r = recent(home)
        if r and os.path.realpath(r[0]) == os.path.realpath(pfad): return True
        time.sleep(1)
    return False

def main():
    if len(sys.argv) < 2 or not os.path.exists(sys.argv[1]): print(__doc__); return 2
    exe = os.path.abspath(sys.argv[1]); repo = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
    home = tempfile.mkdtemp(prefix="vc-appstart-"); fehler = 0; procs = []
    try:
        # Beispielprojekt: im Repository `examples/c64/jumprun`, im Paketordner `beispiele/c64/jumprun` (neben dem Skript)
        hier = os.path.dirname(os.path.abspath(__file__))
        quelle = next((p for p in (os.path.join(repo, "examples", "c64", "jumprun"), os.path.join(hier, "beispiele", "c64", "jumprun")) if os.path.isdir(p)), None)
        if quelle is None: print("FEHLER: Beispielprojekt c64/jumprun nicht gefunden (examples/ oder beispiele/)"); return 2
        for n in ("a", "b", "c"): shutil.copytree(quelle, os.path.join(home, n))
        env = dict(os.environ, HOME=home); env.pop("XDG_CONFIG_HOME", None)
        start = lambda *a: subprocess.Popen([exe, *a], env=env, cwd=home, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        procs.append(start(os.path.join(home, "a"), "--open", "jumprun.asm"))
        ok = warte(home, os.path.join(home, "a")); print(("ok" if ok else "FEHLER") + ": Aufruf mit Ordner und --open öffnet das Projekt und trägt es zuerst in die Zuletzt-Liste ein"); fehler += not ok
        procs.append(start(os.path.join(home, "b"), "--open", "jumprun.asm"))   # laufende Anwendung: Einzelinstanz leitet weiter
        ok = warte(home, os.path.join(home, "b")); print(("ok" if ok else "FEHLER") + ": zweiter Aufruf bei laufender Anwendung öffnet sein Projekt in der laufenden"); fehler += not ok
        for p in procs: p.terminate()
        time.sleep(3); [p.kill() for p in procs if p.poll() is None]; time.sleep(2)
        # Neustart: ein anderes Projekt steht als letztes in der Zuletzt-Liste und in der Sitzung; der Startpfad gewinnt
        procs = [start(os.path.join(home, "c"), "--open", "jumprun.asm")]
        ok = warte(home, os.path.join(home, "c")); print(("ok" if ok else "FEHLER") + ": Startpfad schlägt die Sitzung (anderes Projekt zuletzt benutzt)"); fehler += not ok
    finally:
        for p in procs: p.kill()
        shutil.rmtree(home, ignore_errors=True)
    print(f"\n{3} Prüfungen, {fehler} Fehler"); return 1 if fehler else 0

if __name__ == "__main__": sys.exit(main())
