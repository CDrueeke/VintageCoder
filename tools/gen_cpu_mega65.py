#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt data/cpus/45gs02.cpu.json aus den Tabellen des Xemu-Projekts (lgblgblgb/xemu, GPLv2+):
  xemu/cpu65ce02_disasm_tables.c (Mnemonics + Adressierungsarten je Opcode) und xemu/cpu65_mega65_timings.h (Zyklen bei 40 MHz).
Aufruf: gen_cpu.py <cpu65ce02_disasm_tables.c> <cpu65_mega65_timings.h> <ausgabe.cpu.json>"""
import argparse, json, re, sys
_ap = argparse.ArgumentParser(
    description="""Erzeugt data/cpus/45gs02.cpu.json (Befehlssatz der MEGA65-CPU 45GS02) aus den Tabellen des Xemu-Projekts (lgblgblgb/xemu, GPLv2+).

Liest TABELLEN (xemu/cpu65ce02_disasm_tables.c: Mnemonics und Adressierungsarten je Opcode) und ZEITEN (xemu/cpu65_mega65_timings.h: Zyklen bei 40 MHz)
und schreibt die Datei AUSGABE (UTF-8, wird überschrieben; der Ordner muss existieren). Der Arbeitsordner ist egal, wenn die Pfade passend angegeben werden.""",
    epilog="""Beispiele:
  python3 tools/gen_cpu_mega65.py ../xemu/xemu/cpu65ce02_disasm_tables.c ../xemu/xemu/cpu65_mega65_timings.h data/cpus/45gs02.cpu.json
  python3 tools/gen_cpu_mega65.py tabellen.c zeiten.h /tmp/45gs02.cpu.json""",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_ap.add_argument("tabellen", metavar="TABELLEN", help="cpu65ce02_disasm_tables.c aus dem Xemu-Quelltext (wird nur gelesen)")
_ap.add_argument("zeiten", metavar="ZEITEN", help="cpu65_mega65_timings.h aus dem Xemu-Quelltext (wird nur gelesen)")
_ap.add_argument("ausgabe", metavar="AUSGABE", help="zu schreibende Datei, z. B. data/cpus/45gs02.cpu.json")
_args = _ap.parse_args()
tables, timings, out = (open(_args.tabellen, encoding="utf-8").read(), open(_args.zeiten, encoding="utf-8").read(), _args.ausgabe)
names = re.findall(r'"([A-Z0-9]+)"', re.search(r"opcode_names\[256\] = \{(.*?)\};", tables, re.S).group(1))
adms = [int(x) for x in re.search(r"opcode_adms\[256\] = \{(.*?)\};", tables, re.S).group(1).split(",")]
cyc = [int(x) for x in re.search(r"TIMINGS_65GS_FAST\s*\{(.*?)\}", timings, re.S).group(1).split(",")]
assert len(names) == len(adms) == len(cyc) == 256
MODE = {0: "imp", 1: "imm", 2: "imw", 3: "zp", 4: "zpr", 5: "zpx", 6: "zpy", 7: "abs", 8: "abx", 9: "aby", 10: "rel", 11: "rl", 12: "izy", 13: "izz", 14: "isy", 15: "izx", 16: "ind", 17: "iax", 18: "acc"}
BYTES = {"imp": 1, "acc": 1, "imm": 2, "imw": 3, "zp": 2, "zpr": 3, "zpx": 2, "zpy": 2, "abs": 3, "abx": 3, "aby": 3, "rel": 2, "rl": 3, "izy": 2, "izz": 2, "isy": 2, "izx": 2, "ind": 3, "iax": 3}
LONG = {"BPL", "BMI", "BVC", "BVS", "BCC", "BCS", "BNE", "BEQ", "BRA"}
FLAGS = {"ADC": "NVZC", "SBC": "NVZC", "AND": "NZ", "ORA": "NZ", "EOR": "NZ", "ASL": "NZC", "LSR": "NZC", "ROL": "NZC", "ROR": "NZC", "ASR": "NZC", "ASW": "NZC", "ROW": "NZC",
         "LDA": "NZ", "LDX": "NZ", "LDY": "NZ", "LDZ": "NZ", "CMP": "NZC", "CPX": "NZC", "CPY": "NZC", "CPZ": "NZC", "BIT": "NVZ", "INC": "NZ", "DEC": "NZ", "INX": "NZ", "INY": "NZ", "INZ": "NZ",
         "DEX": "NZ", "DEY": "NZ", "DEZ": "NZ", "INW": "NZ", "DEW": "NZ", "TAX": "NZ", "TAY": "NZ", "TAZ": "NZ", "TXA": "NZ", "TYA": "NZ", "TZA": "NZ", "TSX": "NZ", "TBA": "NZ", "PLA": "NZ", "PLX": "NZ",
         "PLY": "NZ", "PLZ": "NZ", "NEG": "NZ", "TSY": "NZ", "PLP": "NVBDIZC", "RTI": "NVBDIZC", "SEC": "C", "CLC": "C", "SED": "D", "CLD": "D", "SEI": "I", "CLI": "I", "CLV": "V", "SEE": "E", "CLE": "E",
         "BRK": "BI", "TRB": "Z", "TSB": "Z"}
rows = []
for op in range(256):
    mn, mode = names[op], MODE[adms[op]]
    if mode == "rl" and mn in LONG: mn = "L" + mn
    rows.append([mn, mode, "$%02X" % op, BYTES[mode], cyc[op], "b" if mode in ("rel",) else "", FLAGS.get(mn.lstrip("L") if mode == "rl" else mn, ""), "official"])
seen = {}
for r in rows:
    k = (r[0], r[1])
    assert k not in seen, f"doppelt: {k} {seen[k]} {r[2]}"; seen[k] = r[2]
doc = {
    "$schema": "https://vintagecoder.dev/schema/cpu-1.json", "id": "45gs02",
    "_comment": "MEGA65 45GS02 (65CE02-Befehlssatz). Tabelle und 40-MHz-Zyklen aus den Tabellen des Xemu-Projekts (lgblgblgb/xemu, GPLv2+), erzeugt von tools/gen_cpu_mega65.py. Lange Sprünge: Präfix L (LBNE …).",
    "registers": ["A", "X", "Y", "Z", "B", "SP", "PC", "P"], "addressBits": 16, "features": ["mega65.ce02"],
    "opColumns": ["mnemonic", "mode", "opcode", "bytes", "cycles", "flags", "affects", "group"],
    # Q-Register (NEG NEG): 32-Bit-Befehle; $EA ("NOP"-Präfix) macht ($nn),Z zum 32-Bit-Zeiger [$nn],Z
    "prefixes": {"nop32": "$EA", "quad": "$42 $42"},
    "quadOps": {"LDQ": "LDA", "STQ": "STA", "ADCQ": "ADC", "SBCQ": "SBC", "ANDQ": "AND", "ORQ": "ORA", "EORQ": "EOR", "CPQ": "CMP", "BITQ": "BIT", "ASLQ": "ASL", "LSRQ": "LSR",
                "ROLQ": "ROL", "RORQ": "ROR", "ASRQ": "ASR", "INQ": "INC", "DEQ": "DEC", "NEGQ": "NEG"},
    "ops": rows,
}
text = json.dumps({k: v for k, v in doc.items() if k != "ops"}, indent=2, ensure_ascii=False)[:-2] + ',\n  "ops": [\n' + ",\n".join("    " + json.dumps(r, ensure_ascii=False) for r in rows) + "\n  ]\n}\n"
open(out, "w", encoding="utf-8").write(text)
print(f"{len(rows)} Opcodes → {out}")
