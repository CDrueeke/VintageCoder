#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Prüft, dass jede Textdatei in crates/, tools/, data/sys, data/templates, examples und editors
eine SPDX-Lizenzzeile (`SPDX-License-Identifier: …`) in den ersten Zeilen hat oder unter einem
Ordnervermerk liegt (Datei `LICENSE` oder `LICENSE.txt` im Ordner oder einem Oberordner innerhalb
des geprüften Bereichs). Ausnahmen stehen in AUSNAHMEN (Endungen und Pfadanfänge).
Fehler = Exit 1. Aufruf: python3 tools/check_spdx.py [--list]"""
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BEREICHE = ["crates", "tools", "data/sys", "data/templates", "examples", "editors"]
KOPFZEILEN = 12
SPDX = re.compile(r"SPDX-License-Identifier:\s*([A-Za-z0-9.+\-() ]+?)\s*(?:\*/|-->)?\s*$")
VERMERK = ("LICENSE", "LICENSE.txt")

# Dateien ohne Kommentarsyntax oder Binärdaten: kein Zeilenvermerk möglich (Ordnervermerk genügt, sonst Ausnahme).
AUSNAHME_ENDUNGEN = {
    ".json", ".png", ".ttf", ".otf", ".bin", ".prg", ".d64", ".atr", ".xex", ".crt", ".sid", ".ico", ".icns",
    ".svg", ".gif", ".jpg", ".pdf", ".lock", ".snap", ".chars", ".sprites", ".tur", ".ubuntu", ".names", ".txt",
    ".md", ".vcprj", ".vcchar", ".vcscreen", ".vctiles", ".vcsprite", ".vcsong", ".vcplay", ".vcpmg", ".vcpal",
    ".vctest", ".bas", ".gitignore", ".wav", ".mod", ".rmt", ".tmlanguage", ".yml", ".yaml", ".toml",
}
AUSNAHME_PFADE = [
    "crates/vc-gui/tests/ref/",  # lokale Bildvergleiche (ignoriert)
    "tools/testdata/",
    "tools/names/",
    "crates/vc-basic/tests/fixtures/",
    "tools/docker/",
]
ERLAUBT = re.compile(r"^(GPL|LGPL|AGPL)-[0-9.]+(-only|-or-later)?$|^(MIT|MIT-0|CC0-1\.0|Apache-2\.0|BSD-[23]-Clause|ISC|0BSD)$")


def dateien():
    out = subprocess.run(["git", "ls-files", "-z", "--"] + BEREICHE, cwd=ROOT, capture_output=True, check=True).stdout
    return [p for p in out.decode("utf-8").split("\0") if p]


def ist_text(pfad):
    try:
        with open(os.path.join(ROOT, pfad), "rb") as f:
            return b"\0" not in f.read(4096)
    except OSError:
        return False


def kennung(pfad):
    try:
        with open(os.path.join(ROOT, pfad), encoding="utf-8", errors="replace") as f:
            for _ in range(KOPFZEILEN):
                zeile = f.readline()
                if not zeile:
                    break
                m = SPDX.search(zeile.rstrip("\n"))
                if m:
                    return m.group(1).strip()
    except OSError:
        pass
    return None


def ordnervermerk(pfad):
    """Sucht LICENSE/LICENSE.txt im Ordner der Datei und aufwärts bis zum Bereichsanfang."""
    ordner = os.path.dirname(pfad)
    bereich = next((b for b in BEREICHE if pfad == b or pfad.startswith(b + "/")), "")
    while True:
        for n in VERMERK:
            if os.path.isfile(os.path.join(ROOT, ordner, n)):
                return os.path.join(ordner, n)
        if ordner == bereich or not ordner:
            return None
        ordner = os.path.dirname(ordner)


def ausgenommen(pfad):
    if any(pfad.startswith(p) for p in AUSNAHME_PFADE):
        return True
    return os.path.splitext(pfad)[1].lower() in AUSNAHME_ENDUNGEN or os.path.basename(pfad) in VERMERK


def pruefe():
    fehler = []
    for p in dateien():
        if not ist_text(p):
            continue
        k = kennung(p)
        if k is not None:
            if not all(ERLAUBT.match(t) for t in re.split(r"\s+(?:AND|OR|WITH)\s+", k) if not t.startswith("LicenseRef")):
                fehler.append(f"{p}: unbekannte SPDX-Kennung „{k}“")
            continue
        if ordnervermerk(p) or ausgenommen(p):
            continue
        fehler.append(f"{p}: keine SPDX-Zeile und kein Ordnervermerk")
    return fehler


def main():
    fehler = pruefe()
    for f in fehler:
        print(f)
    if fehler:
        print(f"check_spdx: {len(fehler)} Fund(e)")
        return 1
    print("check_spdx: 0 Funde")
    return 0


if __name__ == "__main__":
    sys.exit(main())
