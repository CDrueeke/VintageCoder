#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt data/chips/{vic4,dmagic,mega65}.chip.json aus der I/O-Karte des MEGA65-Kerns (MEGA65/mega65-core, iomap.txt, GPLv3).
Zeilenformat der Quelle: `<Modus C64|C65|GS> <$Adresse[.Bit]> <CHIP:NAME[@ALIAS]> <Beschreibung>`; nur ganze Register mit symbolischem Namen.
Aufruf: gen_chips.py <iomap.txt> <ausgabeverzeichnis>"""
import argparse, json, re, sys
_ap = argparse.ArgumentParser(
    description="""Erzeugt data/chips/{vic4,dmagic,mega65}.chip.json aus der I/O-Karte des MEGA65-Kerns (iomap.txt aus MEGA65/mega65-core, GPLv3).

Liest die Datei IOMAP (Zeilenformat: `<Modus C64|C65|GS> <$Adresse[.Bit]> <CHIP:NAME[@ALIAS]> <Beschreibung>`; nur ganze Register mit symbolischem Namen)
und schreibt in den Ordner AUSGABEVERZEICHNIS die drei Dateien vic4.chip.json, dmagic.chip.json und mega65.chip.json (UTF-8, vorhandene werden überschrieben).
Das Ausgabeverzeichnis muss existieren. Der Arbeitsordner ist egal, wenn die Pfade passend angegeben werden.""",
    epilog="""Beispiele:
  python3 tools/gen_chips_mega65.py ../mega65-core/iomap.txt data/chips
  python3 tools/gen_chips_mega65.py /pfad/zu/iomap.txt /tmp/chips""",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_ap.add_argument("iomap", metavar="IOMAP", help="iomap.txt des MEGA65-Kerns (wird nur gelesen)")
_ap.add_argument("ausgabe", metavar="AUSGABEVERZEICHNIS", help="Ordner für vic4/dmagic/mega65.chip.json (muss existieren), z. B. data/chips")
_args = _ap.parse_args()
src, outdir = open(_args.iomap, encoding="utf-8", errors="replace").read(), _args.ausgabe
GROUPS = {   # Chip-ID → (Anzeigename, Fenster, {Quellchip: Priorität})
    "vic4": ("VIC-IV (MEGA65, mit VIC-III-Registern)", ("$D02F", "$D07F"), {"VIC-IV": 0, "VIC-III": 1}),
    "dmagic": ("DMAgic (F018B) / Enhanced DMA", ("$D700", "$D70F"), {"DMA": 0}),
    "mega65": ("MEGA65-Systemregister (Hypervisor, SD-Karte, F011-Floppy)", ("$D640", "$D6FF"), {"HCPU": 0, "CPU": 1, "SD": 2, "F011": 3, "SDFDC": 4}),
}
regs = {k: {} for k in GROUPS}
DOC_OVERRIDES = {("vic4", "key"): "Schlüsselregister: erst $47, dann $53 schreiben schaltet die erweiterten VIC-IV-Register frei (vorher wirkungslos); Makro mega65_enable_vic4"}   # deutsche Erklärung statt des englischen Quelltexts
for line in src.splitlines():
    m = re.match(r"^(C64|C65|GS)\s+\$([0-9A-Fa-f]{3,4})\s+([A-Z0-9-]+):([A-Z0-9_]+)(?:@\S+)?(?:\s+(.*))?$", line)
    if not m: continue
    mode, addr, chip, name, doc = m.group(1), int(m.group(2), 16), m.group(3), m.group(4), (m.group(5) or "").strip()
    if name in ("VIC-IV", "VIC-III", "VIC-II") or chip == "SUMMARY" or doc.startswith("@"): continue
    for cid, (_, (lo, hi), srcs) in GROUPS.items():
        if chip in srcs and int(lo[1:], 16) <= addr <= int(hi[1:], 16):
            key = name.lower()
            old = regs[cid].get(key)
            if old is None or srcs[chip] < old[2]: regs[cid][key] = (addr, doc[:150], srcs[chip])
for cid, (title, (lo, hi), _) in GROUPS.items():
    items = sorted(((a, n, DOC_OVERRIDES.get((cid, n), d)) for n, (a, d, _) in regs[cid].items()), key=lambda x: (x[0], x[1]))
    doc = {"$schema": "https://vintagecoder.dev/schema/chip-1.json", "id": cid, "name": title, "window": {"space": "cpu16", "start": lo, "end": hi}, "regs": []}
    text = json.dumps({k: v for k, v in doc.items() if k != "regs"}, indent=2, ensure_ascii=False)[:-2] + ',\n  "_comment": "Quelle: MEGA65/mega65-core iomap.txt; erzeugt von tools/gen_chips_mega65.py",\n  "regs": [\n'
    text += ",\n".join("    " + json.dumps({"name": n, "addr": "$%04X" % a, "access": "rw", "doc": d}, ensure_ascii=False) for a, n, d in items) + "\n  ]\n}\n"
    open(f"{outdir}/{cid}.chip.json", "w", encoding="utf-8").write(text)
    print(cid, len(items), "Register")
