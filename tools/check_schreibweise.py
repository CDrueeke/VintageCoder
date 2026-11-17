#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Schreibweisen-Scan: Der Fachbegriff fuer die ersten 256 Bytes des Speichers heisst in allen Texten des Projekts (Code, Oberflaeche,
Dokumentation, Hilfe, Handbuch, Tests, Beispiele, Aenderungsprotokoll) immer so, wie ihn die Hardware-Handbuecher schreiben; eine
deutsche Uebersetzung kommt nie vor. Jeder Treffer (ohne Beachtung der Gross-/Kleinschreibung) in einer versionierten Textdatei ist ein
Fehler. Ebenso verboten ist die Vorsilbe „Kil…ler“ (z. B. in „…feature“): stattdessen „Zusatzfunktion“. Die verbotenen Woerter stehen hier nur in Teilen, damit diese Datei selbst nicht trifft.

Aufruf (im Repository):
  python3 tools/check_schreibweise.py            alle versionierten Dateien (git ls-files); Rueckgabewert 1 bei Funden
  python3 tools/check_schreibweise.py ORDNER     alle Dateien unter ORDNER (fuer Tests)
Nur Standardbibliothek."""
import os, re, subprocess, sys

VERBOTEN = re.compile("|".join(["".join(["null", "seit"]), "".join(["kil", "ler"])]), re.I)
SELF = {"tools/check_schreibweise.py", "tools/test_check_schreibweise.py"}
BINAER_ENDUNGEN = {".png", ".icns", ".ico", ".otf", ".ttf", ".prg", ".d64", ".atr", ".bin", ".rom", ".zip", ".gz"}


def dateien(root, git):
    if git:
        out = subprocess.check_output(["git", "ls-files", "-z"], cwd=root)
        return sorted(p for p in out.decode("utf-8").split("\0") if p)
    erg = []
    for d, dirs, files in os.walk(root):
        dirs[:] = sorted(x for x in dirs if x != ".git")
        erg += [os.path.relpath(os.path.join(d, f), root).replace(os.sep, "/") for f in sorted(files)]
    return sorted(erg)


def scan(root, git=False):
    """Funde als Liste von (Datei, Zeile, Text); Zeile 0 = Treffer im Dateinamen."""
    funde = []
    for rel in dateien(root, git):
        if rel in SELF:
            continue
        if VERBOTEN.search(rel):
            funde.append((rel, 0, rel))
        p = os.path.join(root, rel)
        if os.path.splitext(rel)[1].lower() in BINAER_ENDUNGEN or not os.path.isfile(p) or os.path.islink(p):
            continue
        try:
            with open(p, "rb") as fh:
                roh = fh.read()
        except OSError:
            continue
        if b"\0" in roh[:4096]:
            continue
        for i, z in enumerate(roh.decode("utf-8", "replace").split("\n"), 1):
            if VERBOTEN.search(z):
                funde.append((rel, i, z.strip()[:120]))
    return funde


def main(argv):
    if len(argv) > 1 and argv[1] in ("-h", "--help"):
        print(__doc__); return 0
    root = argv[1] if len(argv) > 1 else os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    funde = scan(root, git=len(argv) <= 1)
    for rel, z, t in funde:
        print("%s:%d: [schreibweise] %s" % (rel, z, t))
    print("%d Fund(e)" % len(funde))
    return 1 if funde else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
