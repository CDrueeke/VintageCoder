#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Wendet die Vorschläge des Übersetzers („consider borrowing here“, `&` voranstellen) auf die Fehler nach der Textextraktion an (i18n I1).

  python3 tools/i18n_fixborrow.py -p vc-gui [-p …]      baut, wendet die maschinenlesbaren Vorschläge an und wiederholt bis nichts mehr zu tun ist"""
import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def runde(pakete, tests=False):
    cmd = ["cargo", "build", "--message-format=json"] + (["--tests"] if tests else []) + [a for p in pakete for a in ("-p", p)]
    out = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True).stdout
    edits, fehler = {}, 0
    for line in out.splitlines():
        try:
            m = json.loads(line)
        except ValueError:
            continue
        if m.get("reason") != "compiler-message" or m["message"]["level"] != "error":
            continue
        fehler += 1
        # Fehler innerhalb eines Makroaufrufs (t!): der Übersetzer macht dort keinen Vorschlag; wird `&str` erwartet und ein String geliefert, `&` voranstellen
        for sp in m["message"].get("spans", []):
            lab = sp.get("label") or ""
            site = sp
            while site.get("expansion"):
                site = site["expansion"]["span"]
            if ("expected `&str`, found `String`" in lab or "expected `&std::string::String`, found `std::string::String`" in lab or "expected `&String`, found `String`" in lab):
                try:
                    b = open(os.path.join(ROOT, site["file_name"]), "rb").read()
                except OSError:
                    continue
                if b[site["byte_start"]:site["byte_start"] + 12] == b"vc_i18n::t!(" or b[site["byte_start"]:site["byte_start"] + 8] == b"format!(":
                    edits.setdefault(site["file_name"], set()).add((site["byte_start"], site["byte_start"], "&"))
        for ch in m["message"].get("children", []):
            if not any(k in ch.get("message", "") for k in ("borrow", "call the method")):
                continue
            for sp in ch.get("spans", []):
                if sp.get("suggested_replacement") is not None and sp.get("suggestion_applicability") in ("MachineApplicable", "MaybeIncorrect"):
                    edits.setdefault(sp["file_name"], set()).add((sp["byte_start"], sp["byte_end"], sp["suggested_replacement"]))
    angewandt = 0
    for f, es in edits.items():
        p = os.path.join(ROOT, f)
        b = open(p, "rb").read()
        for s, e, r in sorted(es, reverse=True):
            b = b[:s] + r.encode() + b[e:]
            angewandt += 1
        open(p, "wb").write(b)
    return fehler, angewandt


def main(argv):
    pakete = [argv[i + 1] for i, a in enumerate(argv) if a == "-p"] or ["vc-gui"]
    for i in range(30):
        fehler, n = runde(pakete, "--tests" in argv)
        print(f"Runde {i + 1}: {fehler} Fehler, {n} Vorschläge angewandt")
        if fehler == 0 or n == 0:
            return 0 if fehler == 0 else 1
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
