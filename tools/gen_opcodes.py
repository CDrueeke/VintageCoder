#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt data/cpus/nmos6502.cpu.json aus der Opcode-Matrix (offizielle + undokumentierte NMOS-Opcodes)."""
import argparse, json, sys, os
_ap = argparse.ArgumentParser(
    description="""Erzeugt data/cpus/nmos6502.cpu.json aus der Opcode-Matrix (offizielle und undokumentierte NMOS-Opcodes des 6502, 256 Einträge).

Liest nichts und schreibt die eine Datei data/cpus/nmos6502.cpu.json (vorhandene wird überschrieben; Pfad relativ zum Ort des Skripts, der Arbeitsordner ist egal).
Die Datei wird ohne ausdrückliche Kodierung geschrieben; unter Windows vorher PYTHONUTF8=1 setzen (Windows-Eingabeaufforderung: set PYTHONUTF8=1).""",
    epilog="""Beispiele:
  python3 tools/gen_opcodes.py
  cd tools && python3 gen_opcodes.py

Das Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.""",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_ap.parse_args()
ops = {}
def add(m, mode, code, cyc, flags="", aff="", grp="official"):
    size = {"imp":1,"acc":1,"imm":2,"zp":2,"zpx":2,"zpy":2,"rel":2,"izx":2,"izy":2,"abs":3,"abx":3,"aby":3,"ind":3}[mode]
    assert code not in ops, hex(code)
    ops[code] = [m, mode, f"${code:02X}", size, cyc, flags, aff, grp]
AFF = dict(ORA="NZ",AND="NZ",EOR="NZ",ADC="NVZC",SBC="NVZC",LDA="NZ",CMP="NZC",ASL="NZC",ROL="NZC",LSR="NZC",ROR="NZC",LDX="NZ",LDY="NZ",
           DEC="NZ",INC="NZ",CPX="NZC",CPY="NZC",BIT="NVZ",DEX="NZ",DEY="NZ",INX="NZ",INY="NZ",TAX="NZ",TAY="NZ",TXA="NZ",TYA="NZ",TSX="NZ",PLA="NZ",PLP="NVDIZC",RTI="NVDIZC",
           CLC="C",SEC="C",CLI="I",SEI="I",CLD="D",SED="D",CLV="V",BRK="BI")
# cc=01
m01 = ["ORA","AND","EOR","ADC","STA","LDA","CMP","SBC"]; modes01 = ["izx","zp","imm","abs","izy","zpx","aby","abx"]
base = dict(imm=2,zp=3,zpx=4,zpy=4,abs=4,abx=4,aby=4,izx=6,izy=5)
for i, m in enumerate(m01):
    for b, mode in enumerate(modes01):
        if m == "STA" and mode == "imm": continue
        cyc = base[mode]; fl = ""
        if mode in ("abx","aby","izy"):
            if m == "STA": cyc += 1
            else: fl = "p"
        add(m, mode, (i << 5) | (b << 2) | 1, cyc, fl, AFF.get(m, ""))
# cc=10
m10 = ["ASL","ROL","LSR","ROR","STX","LDX","DEC","INC"]
rmw = dict(zp=5, zpx=6, abs=6, abx=7)
for i, m in enumerate(m10):
    for b, mode in enumerate(["imm","zp","acc","abs",None,"zpx",None,"abx"]):
        if mode is None: continue
        code = (i << 5) | (b << 2) | 2
        if m in ("ASL","ROL","LSR","ROR"):
            if mode == "imm": continue
            cyc = 2 if mode == "acc" else rmw[mode]; add(m, mode, code, cyc, "", AFF[m])
        elif m in ("DEC","INC"):
            if mode in ("imm","acc"): continue
            add(m, mode, code, rmw[mode], "", AFF[m])
        elif m == "STX":
            if mode in ("imm","acc","abx"): continue
            add(m, "zpy" if mode == "zpx" else mode, code, {"zp":3,"zpx":4,"abs":4}[mode], "", "")
        elif m == "LDX":
            if mode == "acc": continue
            md = "zpy" if mode == "zpx" else ("aby" if mode == "abx" else mode)
            add(m, md, code, {"imm":2,"zp":3,"zpx":4,"abs":4,"abx":4}[mode], "p" if mode == "abx" else "", "NZ")
# cc=00
for code, m, mode, cyc, fl in [(0x24,"BIT","zp",3,""),(0x2C,"BIT","abs",4,""),(0x4C,"JMP","abs",3,""),(0x6C,"JMP","ind",5,""),
    (0x84,"STY","zp",3,""),(0x8C,"STY","abs",4,""),(0x94,"STY","zpx",4,""),(0xA0,"LDY","imm",2,""),(0xA4,"LDY","zp",3,""),(0xAC,"LDY","abs",4,""),
    (0xB4,"LDY","zpx",4,""),(0xBC,"LDY","abx",4,"p"),(0xC0,"CPY","imm",2,""),(0xC4,"CPY","zp",3,""),(0xCC,"CPY","abs",4,""),
    (0xE0,"CPX","imm",2,""),(0xE4,"CPX","zp",3,""),(0xEC,"CPX","abs",4,"")]:
    add(m, mode, code, cyc, fl, AFF.get(m, ""))
for code, m in [(0x10,"BPL"),(0x30,"BMI"),(0x50,"BVC"),(0x70,"BVS"),(0x90,"BCC"),(0xB0,"BCS"),(0xD0,"BNE"),(0xF0,"BEQ")]:
    add(m, "rel", code, 2, "b")
for code, m, cyc in [(0x00,"BRK",7),(0x08,"PHP",3),(0x18,"CLC",2),(0x28,"PLP",4),(0x38,"SEC",2),(0x40,"RTI",6),(0x48,"PHA",3),(0x58,"CLI",2),
    (0x60,"RTS",6),(0x68,"PLA",4),(0x78,"SEI",2),(0x88,"DEY",2),(0x98,"TYA",2),(0xA8,"TAY",2),(0xB8,"CLV",2),(0xC8,"INY",2),(0xD8,"CLD",2),
    (0xE8,"INX",2),(0xF8,"SED",2),(0x8A,"TXA",2),(0x9A,"TXS",2),(0xAA,"TAX",2),(0xBA,"TSX",2),(0xCA,"DEX",2),(0xEA,"NOP",2)]:
    add(m, "imp", code, cyc, "", AFF.get(m, ""))
add("JSR","abs",0x20,6)
# undokumentiert, stabil
for base_, m, aff in [(0x03,"SLO","NZC"),(0x23,"RLA","NZC"),(0x43,"SRE","NZC"),(0x63,"RRA","NVZC"),(0xC3,"DCP","NZC"),(0xE3,"ISC","NVZC")]:
    for off, mode, cyc in [(0x00,"izx",8),(0x04,"zp",5),(0x0C,"abs",6),(0x10,"izy",8),(0x14,"zpx",6),(0x18,"aby",7),(0x1C,"abx",7)]:
        add(m, mode, base_ + off, cyc, "", aff, "illegal.stable")
for code, mode, cyc, fl in [(0x83,"izx",6,""),(0x87,"zp",3,""),(0x8F,"abs",4,""),(0x97,"zpy",4,"")]: add("SAX", mode, code, cyc, fl, "", "illegal.stable")
for code, mode, cyc, fl in [(0xA3,"izx",6,""),(0xA7,"zp",3,""),(0xAF,"abs",4,""),(0xB3,"izy",5,"p"),(0xB7,"zpy",4,""),(0xBF,"aby",4,"p")]: add("LAX", mode, code, cyc, fl, "NZ", "illegal.stable")
for code, m, aff in [(0x0B,"ANC","NZC"),(0x2B,"ANC","NZC"),(0x4B,"ALR","NZC"),(0x6B,"ARR","NVZC"),(0xCB,"AXS","NZC"),(0xEB,"SBC","NVZC")]: add(m, "imm", code, 2, "", aff, "illegal.stable")
add("LAS","aby",0xBB,4,"p","NZ","illegal.stable")
# undokumentiert, instabil
add("XAA","imm",0x8B,2,"","NZ","illegal.unstable"); add("LXA","imm",0xAB,2,"","NZ","illegal.unstable")
add("SHA","izy",0x93,6,"","","illegal.unstable"); add("SHA","aby",0x9F,5,"","","illegal.unstable"); add("TAS","aby",0x9B,5,"","","illegal.unstable")
add("SHY","abx",0x9C,5,"","","illegal.unstable"); add("SHX","aby",0x9E,5,"","","illegal.unstable")
for code in (0x02,0x12,0x22,0x32,0x42,0x52,0x62,0x72,0x92,0xB2,0xD2,0xF2): add("JAM","imp",code,0,"","","illegal.unstable")
# undokumentierte NOPs (stabil)
for code in (0x1A,0x3A,0x5A,0x7A,0xDA,0xFA): add("NOP","imp",code,2,"","","illegal.stable")
for code in (0x80,0x82,0x89,0xC2,0xE2): add("NOP","imm",code,2,"","","illegal.stable")
for code in (0x04,0x44,0x64): add("NOP","zp",code,3,"","","illegal.stable")
add("NOP","abs",0x0C,4,"","","illegal.stable")
for code in (0x14,0x34,0x54,0x74,0xD4,0xF4): add("NOP","zpx",code,4,"","","illegal.stable")
for code in (0x1C,0x3C,0x5C,0x7C,0xDC,0xFC): add("NOP","abx",code,4,"p","","illegal.stable")
assert len(ops) == 256, len(ops)
rows = [ops[c] for c in sorted(ops)]
out = os.path.join(os.path.dirname(__file__), "..", "data", "cpus", "nmos6502.cpu.json")
lines = ",\n".join("    " + json.dumps(r, ensure_ascii=False) for r in rows)
open(out, "w").write('{\n  "$schema": "https://vintagecoder.dev/schema/cpu-1.json",\n  "id": "nmos6502",\n  "registers": ["A", "X", "Y", "SP", "PC", "P"],\n  "addressBits": 16,\n  "opColumns": ["mnemonic", "mode", "opcode", "bytes", "cycles", "flags", "affects", "group"],\n  "ops": [\n' + lines + '\n  ]\n}\n')
print("ok", len(rows), "Opcodes;", sum(1 for r in rows if r[7] == "official"), "offiziell")
