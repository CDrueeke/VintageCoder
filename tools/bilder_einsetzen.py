#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Setzt Handbuchbilder in Handbuch und Hilfe ein oder nimmt sie wieder heraus, passend zum Status im Manifest.

Für jede Kennung mit `status: vorhanden` wird der Absatz `*[Bild folgt in E13: id]*` durch `![alt](bild:id "Unterschrift")` ersetzt
(deutsche Texte: alt/caption, englische: alt_en/caption_en). Bereits eingesetzte Bildzeilen werden aus dem Manifest neu geschrieben.
Kennungen mit `status: offen` werden zurück in Platzhalter verwandelt (Bildzeile -> Platzhalter).
Aufruf: python3 tools/bilder_einsetzen.py   (danach: python3 tools/check_bildplatzhalter.py und python3 tools/check_i18n_data.py --stempeln)
"""
import json, re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPRACHEN = (
    (["docs/handbuch/kapitel/*.md", "docs/handbuch/GLOSSAR.md", "crates/vc-ide/help/*.md"], "alt", "caption"),
    (["docs/handbuch/en/kapitel/*.md", "docs/handbuch/en/GLOSSAR.md", "crates/vc-ide/help/en/*.md"], "alt_en", "caption_en"),
)
PH = re.compile(r"(?m)^\*\[Bild folgt in E13: ([a-z0-9-]+)\]\*$")
IMG = re.compile(r'(?m)^!\[[^\n]*\]\(bild:([a-z0-9-]+)(?: "[^"\n]*")?\)$')


def main():
    man = {e["id"]: e for e in json.loads((ROOT / "docs/handbuch/bilder/manifest.json").read_text(encoding="utf-8"))["bilder"]}
    geaendert = 0
    for globs, fa, fc in SPRACHEN:
        for g in globs:
            for f in sorted(ROOT.glob(g)):
                s = o = f.read_text(encoding="utf-8")
                def bild(m):
                    e = man.get(m.group(1))
                    return m.group(0) if not e or e["status"] != "vorhanden" else '![%s](bild:%s "%s")' % (e[fa], m.group(1), e[fc])
                def platz(m):
                    e = man.get(m.group(1))
                    return m.group(0) if not e or e["status"] != "offen" else "*[Bild folgt in E13: %s]*" % m.group(1)
                s = PH.sub(bild, s)
                s = IMG.sub(lambda m: bild(m) if man.get(m.group(1), {}).get("status") == "vorhanden" else platz(m), s)
                if s != o:
                    f.write_text(s, encoding="utf-8"); geaendert += 1
                    print("geändert:", f.relative_to(ROOT))
    print(f"bilder_einsetzen: {geaendert} Dateien geändert")


if __name__ == "__main__":
    main()
