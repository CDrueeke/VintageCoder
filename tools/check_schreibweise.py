#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Schreibweisen-Scan: Der Fachbegriff fuer die ersten 256 Bytes des Speichers heisst in allen Texten des Projekts (Code, Oberflaeche,
Dokumentation, Hilfe, Handbuch, Tests, Beispiele, Aenderungsprotokoll) immer so, wie ihn die Hardware-Handbuecher schreiben; eine
deutsche Uebersetzung kommt nie vor. Jeder Treffer (ohne Beachtung der Gross-/Kleinschreibung) in einer versionierten Textdatei ist ein
Fehler. Ebenso verboten ist die deutsche Uebersetzung von „Onion Skin“ (das Wort bleibt englisch) und die Vorsilbe „Kil…ler“ (z. B. in „…feature“): stattdessen „Zusatzfunktion“. Die verbotenen Woerter stehen hier nur in Teilen, damit diese Datei selbst nicht trifft.

Aufruf (im Repository):
  python3 tools/check_schreibweise.py            alle versionierten Dateien (git ls-files); Rueckgabewert 1 bei Funden
  python3 tools/check_schreibweise.py ORDNER     alle Dateien unter ORDNER (fuer Tests)
Nur Standardbibliothek."""
import os, re, subprocess, sys

VERBOTEN = re.compile("|".join(["".join(["null", "seit"]), "".join(["kil", "ler"]), "".join(["zwiebel", "haut"])]), re.I)
SELF = {"tools/check_schreibweise.py", "tools/test_check_schreibweise.py", "docs/glossar-i18n.md"}  # das Glossar nennt die verbotenen Woerter, um sie zu verbieten
# Verbotene Schreibweisen je Sprache (i18n, Konzept Abschnitt 7): (Pfadanfang der Texte dieser Sprache, Muster, Begruendung).
# Englisch: "zero page" bzw. "zero-page addressing" ist richtig, nie zusammengeschrieben; "Onion Skin" bleibt genau so (nie klein oder mit Bindestrich).
# Deutsch: "Zeropage" und "Onion Skin" bleiben unuebersetzt, also keine englischen Ersatzschreibweisen "zero page"/"zero-page".
# Pfade der englischen und der deutschen Texte: (Anfaenge, ausgenommene Anfaenge)
EN = (("crates/vc-i18n/catalog/en/", "data/i18n/en/", "data/cli/en/", "crates/vc-ide/help/en/", "data/templates/en/", "docs/handbuch/en/", "README.en.md", "docs/cli.en.md"), ())
DE = (("crates/vc-i18n/catalog/de/", "crates/vc-ide/help/", "data/cli/", "docs/handbuch/kapitel/", "docs/handbuch/GLOSSAR.md", "README.md", "docs/cli.md"), ("crates/vc-ide/help/en/", "data/cli/en/"))
# VC-20 (deutsch) und VIC-20 (englisch): Entscheidung des Inhabers 2026-11-21; "VIC-II" (Chip) bleibt unberuehrt
SPRACHREGELN = [
    (EN, re.compile("zero" + "page", re.I), "englisch: zero page (zwei Woerter), nie Zeropage"),
    (EN, re.compile("onion" + "-" + "skin", re.I), "Onion Skin ohne Bindestrich"),
    (EN, re.compile("onion" + " " + "skin"), "Onion Skin gross geschrieben"),
    (EN, re.compile(r"\bVC-20\b"), "englisch: VIC-20 (nicht VC-20)"),
    (DE, re.compile("zero" + "[ -]" + "page", re.I), "deutsch: Zeropage, nicht zero page"),
    (DE, re.compile("onion" + "[- ]" + "skin"), "deutsch und englisch: Onion Skin"),
    (DE, re.compile(r"\bVIC-20\b"), "deutsch: VC-20 (nicht VIC-20)"),
    # „Studio“ entfällt: überall „Editor“ (nur im Wert eines Katalogeintrags; Schlüssel wie `diskstudio-…` bleiben)
    ((("crates/vc-i18n/catalog/",), ()), re.compile(r"^(?!\s*#).*stud" + "io", re.I), "nie „Studio“, immer „Editor“ (Glossar)"),
]
# Erklaerungen, die die Schreibweise der anderen Sprache ausdruecklich nennen (Glossar, Tabelle der Systeme): "(Zeropage)", "VC-20 (VIC-20)", "In Germany ... "VC-20""
ERKLAERT = re.compile(r"\(Zero" + r"page\)|VC-20 \(VIC-20\)|[\"\u201e]VC-20[\"\u201c]|hie\u00df der VIC-20|called \"VC-20\"")
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
        regeln = [(m, w) for (pre, aus), m, w in SPRACHREGELN if rel.startswith(pre) and not rel.startswith(aus)]
        for i, z in enumerate(roh.decode("utf-8", "replace").split("\n"), 1):
            if VERBOTEN.search(z):
                funde.append((rel, i, z.strip()[:120]))
            # in den Katalogen zaehlt nur der Text rechts vom Gleichheitszeichen (Schluessel wie studio-onion-skin sind ASCII-Namen)
            zt = z.split(" = ", 1)[1] if rel.endswith(".ftl") and " = " in z else z
            if rel.endswith(".md"): zt = re.sub(r"`[^`]*`", "", zt)  # Code (Schluessel, Befehle) bleibt unberuehrt
            if rel.startswith("data/i18n/en/") and '"en":' not in z: zt = ""  # in den Ueberlagerungen zaehlt nur das englische Feld, "de" ist das Original
            for m, w in regeln:
                if m.search(zt) and not (rel.startswith("docs/") and ERKLAERT.search(z)):
                    funde.append((rel, i, "%s: %s" % (w, z.strip()[:100])))
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
