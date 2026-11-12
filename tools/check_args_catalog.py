#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Prüft data/args/*.json gegen die Hilfetexte der installierten Programme.

Jeder `flag`/`off_flag` des Katalogs muss im Hilfetext des Programms vorkommen. Aufruf:
  python3 tools/check_args_catalog.py [vice-x64sc=/pfad/x64sc ...]
Ohne Angaben werden übliche Orte durchsucht; nicht gefundene Programme werden gemeldet, nicht geprüft.
"""
import argparse, glob, json, os, re, shutil, subprocess, sys

_ap = argparse.ArgumentParser(
    description="""Prüft die Parameterkataloge data/args/*.json gegen die Hilfetexte der installierten Programme.

Jeder `flag`/`off_flag` eines Katalogs muss im Hilfetext des Programms vorkommen (VICE x64sc/x128/xplus4/xvic, atari800, xemu und der eigene
Assembler vcc asm). Dazu werden die Programme mit ihrer Hilfeoption aufgerufen (ihre Hilfe wird nur gelesen). Ohne Angaben werden übliche Orte
durchsucht (VICE unter /Applications, atari800 und xmega65 im PATH, target/debug|release/vcc im Repository); nicht gefundene Programme werden
gemeldet, nicht geprüft. Gelesen werden nur data/args/*.json; es wird nichts geschrieben. Exit-Code 0 = alles gefunden, 1 = fehlende Flags
oder doppelte Kennungen. Der Arbeitsordner ist egal (alle Pfade gelten relativ zum Ort des Skripts).""",
    epilog="""Beispiele:
  python3 tools/check_args_catalog.py
  python3 tools/check_args_catalog.py vice-x64sc=/Applications/vice-arm64-gtk3-3.10/bin/x64sc vcc-asm=target/debug/vcc

Bekannte Programmnamen: vice-x64sc, vice-x128, vice-xplus4, vice-xvic, atari800, xemu, vcc-asm.""",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_ap.add_argument("programm", nargs="*", metavar="PROGRAMM=PFAD",
                 help="Programm des Katalogs (Feld `program`) und Pfad seiner ausführbaren Datei, z. B. vice-x64sc=/pfad/x64sc; beliebig viele, ohne Angabe wird gesucht")
_args = _ap.parse_args()

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "data", "args")
VICE = "/Applications/vice-arm64-gtk3-3.10/bin"
GUESS = {
    "vice-x64sc": [VICE + "/x64sc"], "vice-x128": [VICE + "/x128"], "vice-xplus4": [VICE + "/xplus4"], "vice-xvic": [VICE + "/xvic"],
    "atari800": [shutil.which("atari800") or ""], "xemu": ["/Applications/xmega65.app/Contents/MacOS/xmega65", shutil.which("xmega65") or ""],
}
HELP = {"xemu": ["-h"], "atari800": ["-help"], "vcc-asm": ["--help"]}
REPO = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
GUESS["vcc-asm"] = [os.path.join(REPO, "target/debug/vcc"), os.path.join(REPO, "target/release/vcc")]

def help_text(prog, exe):
    out = subprocess.run([exe] + HELP.get(prog, ["-help"]), capture_output=True, text=True, timeout=60, stdin=subprocess.DEVNULL)
    return out.stdout + out.stderr

def main():
    given = dict(a.split("=", 1) for a in _args.programm)
    bad = 0
    for f in sorted(glob.glob(ROOT + "/*.json")):
        cat = json.load(open(f, encoding="utf-8"))
        prog = cat["program"]
        exe = given.get(prog) or next((g for g in GUESS.get(prog, []) if g and os.path.exists(g)), None)
        if not exe:
            print("%-12s nicht geprüft (Programm nicht gefunden)" % prog); continue
        text = help_text(prog, exe)
        flags = {t for p in cat["params"] for t in (p.get("flag"), p.get("off_flag")) if t}
        if prog == "vcc-asm":   # die Hilfe nennt die Schalter im Fließtext: [--illegals …] [-I verzeichnis]
            text = "\n".join(l for l in text.splitlines() if l.startswith("vcc asm") or l.startswith("\t[--labels"))
            miss = sorted(fl for fl in flags if not re.search(r"(?<![\w-])" + re.escape(fl) + r"(?![\w-])", text))
        else:
            miss = sorted(fl for fl in flags if not re.search(r"(?m)^\s*" + re.escape(fl) + r"(\s|$)", text))
        print("%-12s %d Flags, fehlend: %s" % (prog, len(flags), ", ".join(miss) or "keine"))
        bad += len(miss)
        ids = [p["id"] for p in cat["params"]]
        for i in {i for i in ids if ids.count(i) > 1}: print("  doppelte Kennung", i); bad += 1
    return 1 if bad else 0

sys.exit(main())
