#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Prüft Handbuchbilder und Bildplatzhalter in Handbuch und Hilfe (Regel: docs/doku-regeln-030.md).

Gemischter Zustand: Für Kennungen mit fertigem Bild steht im Text `![alt](bild:id "Unterschrift")`, für alle anderen der Platzhalter
`*[Bild folgt in E13: bild-id]*` als eigener Absatz. Quelle der Wahrheit ist `docs/handbuch/bilder/manifest.json`:
je Eintrag `id`, `status` (`vorhanden` | `offen`), `alt`, `caption` (deutsch), `alt_en`, `caption_en` (englisch).

Funde (Rückgabewert 1):
  * Platzhalter falsch geschrieben (Kennung fehlt oder ungültig, nicht als eigener Absatz)
  * Eintrag im Manifest unvollständig (Pflichtfelder, Status, englische Texte nur ASCII)
  * Status `vorhanden`, aber die PNG-Datei fehlt
  * `bild:`-Verweis auf eine Kennung, die im Manifest fehlt oder nicht `vorhanden` ist
  * Platzhalter für eine Kennung, die im Manifest fehlt oder schon `vorhanden` ist (dann gehört das Bild in den Text)
  * Alt-Text oder Unterschrift im Text weichen vom Manifest ab (deutsch: alt/caption, englisch: alt_en/caption_en)
Hinweise (kein Fund): PNG-Datei zu einer `offen`-Kennung (altes Bild der früheren Oberfläche), Kennungen ohne Stelle im Text.

Aufruf: python3 tools/check_bildplatzhalter.py [--liste]
"""
import json, re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BILDER = ROOT / "docs/handbuch/bilder"
GUT = re.compile(r"^\*\[Bild folgt in E13: [a-z0-9]+(-[a-z0-9]+)*\]\*$")
LOSE = re.compile(r"Bild folgt", re.I)
PH = re.compile(r"^\*\[Bild folgt in E13: ([a-z0-9-]+)\]\*$")
IMG = re.compile(r'^!\[([^\]]*)\]\(bild:([a-z0-9-]+)(?: "([^"]*)")?\)$')
IMG_LOSE = re.compile(r'!\[[^\]]*\]\(bild:')
ID = re.compile(r"^[a-z0-9]+(-[a-z0-9]+)*$")
# Texte je Sprache: (Ordner/Dateien, Feldnamen im Manifest)
SPRACHEN = (
    ("de", ["docs/handbuch/kapitel/*.md", "docs/handbuch/GLOSSAR.md", "crates/vc-ide/help/*.md"], "alt", "caption"),
    ("en", ["docs/handbuch/en/kapitel/*.md", "docs/handbuch/en/GLOSSAR.md", "crates/vc-ide/help/en/*.md"], "alt_en", "caption_en"),
)


def dateien(globs):
    erg = []
    for g in globs:
        erg += sorted(ROOT.glob(g))
    return erg


def main():
    funde, hinweise, liste = [], [], []
    mf = json.loads((BILDER / "manifest.json").read_text(encoding="utf-8"))["bilder"]
    man = {}
    for e in mf:
        i = e.get("id", "")
        if not ID.match(i): funde.append(f"manifest.json: ungültige Kennung {i!r}"); continue
        if i in man: funde.append(f"manifest.json: Kennung {i} doppelt")
        man[i] = e
        if e.get("status") not in ("vorhanden", "offen"): funde.append(f"manifest.json: {i}: status fehlt oder ungültig (vorhanden|offen)")
        for k in ("alt", "caption", "alt_en", "caption_en"):
            v = e.get(k)
            if not isinstance(v, str) or not v.strip(): funde.append(f"manifest.json: {i}: Feld {k} fehlt")
            elif k.endswith("_en") and not v.isascii(): funde.append(f"manifest.json: {i}: Feld {k} nicht nur ASCII")
            elif ("]" in v and k.startswith("alt")) or '"' in v: funde.append(f"manifest.json: {i}: Feld {k} enthält ] oder \"")
        png = BILDER / f"{i}.png"
        if e.get("status") == "vorhanden" and not png.exists(): funde.append(f"manifest.json: {i}: Status vorhanden, aber {png.relative_to(ROOT)} fehlt")
        if e.get("status") == "offen" and png.exists(): hinweise.append(f"{png.relative_to(ROOT)}: altes Bild zu einer offenen Kennung (zeigt die frühere Oberfläche)")
    gesehen = set()
    for sprache, globs, fa, fc in SPRACHEN:
        for f in dateien(globs):
            rel = f.relative_to(ROOT)
            zeilen = f.read_text(encoding="utf-8").splitlines()
            for n, l in enumerate(zeilen, 1):
                eigen = (n == 1 or zeilen[n - 2].strip() == "") and (n == len(zeilen) or zeilen[n].strip() == "")
                if LOSE.search(l):
                    m = PH.match(l.strip())
                    if not (GUT.match(l.strip()) and eigen):
                        funde.append(f"{rel}:{n}: Bildplatzhalter falsch geschrieben: {l.strip()[:70]!r}"); continue
                    i = m.group(1); gesehen.add(i); liste.append(f"{rel}:{n}: {l.strip()}")
                    if i not in man: funde.append(f"{rel}:{n}: Platzhalter für Kennung {i}, die im Manifest fehlt")
                    elif man[i].get("status") == "vorhanden": funde.append(f"{rel}:{n}: Platzhalter, aber das Bild {i} ist vorhanden (Bild einsetzen)")
                elif IMG_LOSE.search(l):
                    m = IMG.match(l.strip())
                    if not (m and eigen):
                        funde.append(f"{rel}:{n}: Bildverweis falsch geschrieben (eigener Absatz, ![alt](bild:id \"Unterschrift\")): {l.strip()[:70]!r}"); continue
                    alt, i, cap = m.group(1), m.group(2), m.group(3)
                    gesehen.add(i)
                    if i not in man: funde.append(f"{rel}:{n}: Bild {i} fehlt im Manifest"); continue
                    if man[i].get("status") != "vorhanden": funde.append(f"{rel}:{n}: Bild {i} ist im Manifest nicht vorhanden (Status offen)"); continue
                    if alt != man[i].get(fa) or (cap or "") != man[i].get(fc, ""):
                        funde.append(f"{rel}:{n}: Alt-Text oder Unterschrift von {i} weichen vom Manifest ab ({fa}/{fc})")
    for i in man:
        if i not in gesehen: hinweise.append(f"manifest.json: {i}: steht in keinem Text")
    if "--liste" in sys.argv:
        for l in liste: print(l)
    if "--hinweise" in sys.argv:
        for h in hinweise: print("Hinweis:", h)
    for l in funde: print(l)
    print(f"check_bildplatzhalter: {len(funde)} Funde, {len(liste)} Platzhalter, {len(hinweise)} Hinweise (mit --hinweise anzeigen)")
    return 1 if funde else 0


if __name__ == "__main__":
    sys.exit(main())
