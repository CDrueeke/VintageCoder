#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt data/basic/basic65.dialect.json (BASIC 65 des MEGA65 (auch BASIC 7.0 der C128, BASIC 3.5 des Plus/4)): Schlüsselwörter aus `petcat -k65`, Tokencodes durch Tokenisieren
jedes Wortes mit `petcat -w65` (Einzelbyte-Tokens, $CE xx = Funktionen, $FE xx = Befehle).
Aufruf: gen_dialect.py <petcat> <ausgabe.json> [version=65 [id=basic65 [ladeadresse=$2001]]]   (z. B. 70 basic7 $1c01 | 3 basic35 $1001)"""
import argparse, json, os, subprocess, sys, tempfile
_ap = argparse.ArgumentParser(
    description="""Erzeugt eine BASIC-Dialektdatei (data/basic/*.dialect.json), z. B. data/basic/basic65.dialect.json (BASIC 65 des MEGA65; auch BASIC 7.0 der C128, BASIC 3.5 des Plus/4).

Die Schlüsselwörter kommen aus `petcat -k<VERSION>`, die Tokencodes durch Tokenisieren jedes Wortes mit `petcat -w<VERSION>` (Einzelbyte-Tokens,
$CE xx = Funktionen, $FE xx = Befehle). Dazu werden petcat-Aufrufe in einem temporären Ordner ausgeführt; gelesen wird außerdem
data/basic/cbm_v2.dialect.json (relativ zum Ort des Skripts, der Arbeitsordner ist egal). Geschrieben wird nur die Datei AUSGABE (wird überschrieben;
ohne ausdrückliche Kodierung, unter Windows vorher PYTHONUTF8=1 setzen, da die Datei Umlaute enthält).""",
    epilog="""Beispiele:
  python3 tools/gen_dialect.py petcat data/basic/basic65.dialect.json
  python3 tools/gen_dialect.py petcat data/basic/basic7.dialect.json 70 basic7 '$1c01'
  (BASIC 3.5 des Plus/4: ... 3 basic35 '$1001')""",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_ap.add_argument("petcat", metavar="PETCAT", help="Pfad oder Name des Programms petcat (VICE)")
_ap.add_argument("ausgabe", metavar="AUSGABE", help="zu schreibende Dialektdatei (.json)")
_ap.add_argument("version", metavar="VERSION", nargs="?", default="65", help="petcat-Version: 65 (MEGA65, Standard), 70 (C128 BASIC 7.0), 3 (Plus/4 BASIC 3.5)")
_ap.add_argument("id", metavar="ID", nargs="?", default="basic65", help="Kennung des Dialekts in der Datei (Standard: basic65)")
_ap.add_argument("ladeadresse", metavar="LADEADRESSE", nargs="?", default="$2001", help="Ladeadresse des BASIC-Programms (Standard: $2001; C128 $1c01, Plus/4 $1001)")
_args = _ap.parse_args()
petcat, out = _args.petcat, _args.ausgabe
ver, did, load = _args.version, _args.id, _args.ladeadresse
kw_out = subprocess.run([petcat, f"-k{ver}"], capture_output=True, text=True).stdout
words = []
for line in kw_out.splitlines():
    if "Available" in line or not line.strip() or line.startswith("/"): continue
    words += [w for w in line.split("\t")]
tmp = tempfile.mkdtemp(); rows = []; seen = set()
for w in words:
    if not w.strip(): continue
    t, p = os.path.join(tmp, "t.txt"), os.path.join(tmp, "t.prg")
    open(t, "w").write(f"10 {w}\n")
    if os.path.exists(p): os.remove(p)
    subprocess.run([petcat, f"-w{ver}", "-o", p, "--", t], capture_output=True)
    d = open(p, "rb").read()
    body = d[2 + 4: ]                      # Ladeadresse (2), Link (2), Zeilennummer (2) – danach die Tokens
    body = d[6:]
    body = body[:body.index(0)] if 0 in body else body
    code = body.hex()
    if len(body) not in (1, 2) or w in seen: continue
    seen.add(w); rows.append((w.upper(), code))
# Reihenfolge wie von petcat gelistet (die Suche nimmt den ersten Treffer)
tokens = ",\n".join(f'    ["{w}", "${c}"]' for w, c in rows)
ctrl = json.load(open(os.path.join(os.path.dirname(__file__), "..", "data", "basic", "cbm_v2.dialect.json")))["control"]
doc = '{\n  "$schema": "https://vintagecoder.dev/schema/basic-dialect-1.json",\n  "id": "' + did + '",\n  "family": "cbm",\n  "backend": "linked_lines",\n  "loadAddr": "' + load + '",\n  "varSigChars": 2,\n  "reservedVars": ["ST", "TI", "DS", "ER", "EL"],\n  "maxLineLen": 160,\n  "matchLongest": true,\n  "hardMaxLineLen": 255,\n  "numberStorage": "ascii_text",\n  "floatFormat": "mbf5",\n  "capabilities": {"doLoop": true, "graphic": true, "spriteCmds": ' + ("false" if ver == "3" else "true") + ', "trap": true},\n  "_comment": "Erzeugt von tools/gen_dialect.py aus petcat -k' + ver + ' / -w' + ver + '. Codes wie $FE3C bestehen aus Präfix und Folgebyte.",\n  "tokens": [\n' + tokens + '\n  ],\n  "control": ' + json.dumps(ctrl) + '\n}\n'
open(out, "w").write(doc); print(len(rows), "Tokens")
