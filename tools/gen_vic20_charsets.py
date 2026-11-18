#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt data/charsets/vic20/*.charset.json aus den Zeichen-ROMs von VICE (VIC20/chargen-901460-0x.bin, je 4 KB: zuerst
Groß-/Grafikzeichen, dann Klein-/Großbuchstaben, je 256 Zeichen mit invertierten Zeichen im zweiten Block).
Aufruf: python3 tools/gen_vic20_charsets.py [VICE-Datenordner]; als Probe wird der C64-Satz 901225-01 neu erzeugt und mit data/charsets/c64 verglichen."""
import json, os, sys
VICE = sys.argv[1] if len(sys.argv) > 1 else "/Applications/vice-arm64-gtk3-3.10/VICE.app/Contents/Resources/share/vice"
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "data", "charsets")

def frames(rom, off):
    return [{"rows": [format(b, "08b") for b in rom[off + 8 * c: off + 8 * c + 8]]} for c in range(256)]

def text(profile, name, colors, fr):
    head = '{\n  "format": 1,\n  "kind": "charset",\n  "profile": %s,\n  "name": %s,\n  "colors": %s,\n  "frames": [\n' % (json.dumps(profile), json.dumps(name), json.dumps(colors, separators=(",", ":")))
    return head + ",\n".join("    " + json.dumps(f, separators=(",", ":")) for f in fr) + "\n  ]\n}\n"

# Probe: C64 neu erzeugen und mit dem vorhandenen Satz vergleichen
c64 = open(os.path.join(VICE, "C64", "chargen-901225-01.bin"), "rb").read()
ref = open(os.path.join(ROOT, "c64", "c64_standard_upper.charset.json"), encoding="utf-8").read()
assert text("c64.char.hires", "c64_standard_upper", {"fg": 1, "bg": 6}, frames(c64, 0)) == ref, "Layout-Annahme stimmt nicht mit dem vorhandenen C64-Satz überein"

out = os.path.join(ROOT, "vic20"); os.makedirs(out, exist_ok=True)
for rom in ("901460-03", "901460-02"):
    data = open(os.path.join(VICE, "VIC20", f"chargen-{rom}.bin"), "rb").read()
    assert len(data) == 4096
    for kind, off in (("upper", 0), ("lower", 2048)):
        name = f"vic20_{rom.replace('-', '_')}_{kind}"
        open(os.path.join(out, name + ".charset.json"), "w", encoding="utf-8").write(text("vic20.char.hires", name, {"fg": 0, "bg": 1}, frames(data, off)))
        print(name)
