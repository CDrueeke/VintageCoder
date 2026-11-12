#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Rekursiver 6502-Disassembler für ROM-Abbilder (eigener Code, kein ROM-Inhalt).

Aufruf:  romdis.py ROM BASISADRESSE AUSGABE [--title TEXT] [--seed ADR ...] [--vector ADR ...] [--table START:ANZAHL[:ADJ] ...]
                     [--jumptable START:ANZAHL ...] [--words START:ANZAHL ...] [--names DATEI ...] [--syms DATEI.asm ...] [--no-table-scan]

Verfahren
  1. Einstiegspunkte (Vektoren, --seed, Sprungtabellen) werden rekursiv verfolgt: Verzweigungen, JSR (Rückkehr wird
     angenommen) und JMP; Ende bei RTS, RTI, BRK, JMP.
  2. Zeigertabellen: Jedes 16-Bit-Wort in noch nicht erkanntem Bereich, das (±1, wie bei RTS-Sprungtabellen) auf eine
     plausible, noch nicht erkannte Codestelle zeigt, wird als weiterer Einstieg genommen; die Tabelle selbst wird als Wortliste ausgegeben.
  3. Der Rest wird als Daten ausgegeben: Texte (Bit 7 als Endekennung erkannt) als .text-artige Zeilen, sonst .byte mit ASCII-Spalte.
Namen für Adressen kommen aus .const-Zeilen der übergebenen Assemblerdateien (z. B. data/sys/c64/basic.asm).
Nur legale NMOS-Befehle; ein undokumentierter Opcode beendet den Zweig (Daten).
"""
import re, sys, argparse

OPS = {}
def reg(op, mn, mode): OPS[op] = (mn, mode)
_c1 = ["ORA", "AND", "EOR", "ADC", "STA", "LDA", "CMP", "SBC"]
_m1 = ["izx", "zp", "imm", "abs", "izy", "zpx", "aby", "abx"]
for a, mn in enumerate(_c1):
    for b, m in enumerate(_m1):
        if mn == "STA" and m == "imm": continue
        reg(a * 32 + b * 4 + 1, mn, m)
_c2 = ["ASL", "ROL", "LSR", "ROR", "STX", "LDX", "DEC", "INC"]
for a, mn in enumerate(_c2):
    for b, m in ((1, "zp"), (2, "acc"), (3, "abs"), (5, "zpx"), (7, "abx")):
        if mn in ("STX", "LDX"):
            if m == "zpx": m = "zpy"
            if m == "abx": m = "aby"
            if m == "acc": continue
        if mn in ("DEC", "INC") and m == "acc": continue
        if mn == "STX" and m == "aby": continue
        reg(a * 32 + b * 4 + 2, mn, m)
reg(0xA2, "LDX", "imm")
for op, mn, m in ((0x24, "BIT", "zp"), (0x2C, "BIT", "abs"), (0x4C, "JMP", "abs"), (0x6C, "JMP", "ind"), (0x84, "STY", "zp"), (0x8C, "STY", "abs"),
                  (0x94, "STY", "zpx"), (0xA0, "LDY", "imm"), (0xA4, "LDY", "zp"), (0xAC, "LDY", "abs"), (0xB4, "LDY", "zpx"), (0xBC, "LDY", "abx"),
                  (0xC0, "CPY", "imm"), (0xC4, "CPY", "zp"), (0xCC, "CPY", "abs"), (0xE0, "CPX", "imm"), (0xE4, "CPX", "zp"), (0xEC, "CPX", "abs"),
                  (0x20, "JSR", "abs")):
    reg(op, mn, m)
for i, mn in enumerate(["BPL", "BMI", "BVC", "BVS", "BCC", "BCS", "BNE", "BEQ"]): reg(0x10 + i * 0x20, mn, "rel")
for op, mn in ((0x00, "BRK"), (0x40, "RTI"), (0x60, "RTS"), (0x08, "PHP"), (0x28, "PLP"), (0x48, "PHA"), (0x68, "PLA"), (0x88, "DEY"), (0xA8, "TAY"),
               (0xC8, "INY"), (0xE8, "INX"), (0x18, "CLC"), (0x38, "SEC"), (0x58, "CLI"), (0x78, "SEI"), (0x98, "TYA"), (0xB8, "CLV"), (0xD8, "CLD"),
               (0xF8, "SED"), (0x8A, "TXA"), (0x9A, "TXS"), (0xAA, "TAX"), (0xBA, "TSX"), (0xCA, "DEX"), (0xEA, "NOP")):
    reg(op, mn, "imp")
SIZE = {"imp": 1, "acc": 1, "imm": 2, "zp": 2, "zpx": 2, "zpy": 2, "izx": 2, "izy": 2, "rel": 2, "abs": 3, "abx": 3, "aby": 3, "ind": 3}
END = {"RTS", "RTI", "BRK", "JMP"}

class Rom:
    def __init__(s, data, base, names):
        s.d, s.base, s.end = data, base, base + len(data); s.names = names; s.rename = {}
        s.code = {}      # Adresse -> (mn, mode, operand, größe)
        s.labels = {}    # Adresse -> Art ('sub'|'loc'|'tab')
        s.seeds = []; s.tables = {}  # Adresse -> Länge in Wörtern
        s.cover = bytearray(len(data))  # 0 unbekannt, 1 Code (Anfang), 2 Code (Folgebyte), 3 Tabelle
    def inrom(s, a): return s.base <= a < s.end
    def byte(s, a): return s.d[a - s.base]
    def word(s, a): return s.byte(a) | s.byte(a + 1) << 8
    def decode(s, a):
        if not s.inrom(a): return None
        op = s.byte(a)
        if op not in OPS: return None
        mn, m = OPS[op]; n = SIZE[m]
        if a + n > s.end: return None
        v = 0
        if n == 2: v = s.byte(a + 1)
        if n == 3: v = s.word(a + 1)
        if m == "rel": v = (a + 2 + (v - 256 if v >= 128 else v)) & 0xFFFF
        return mn, m, v, n
    def trace(s, start, check_only=False):
        """Verfolgt Code ab start. check_only: liefert nur Plausibilität (Anzahl Befehle) ohne zu speichern"""
        todo = [start]; seen = {}; first = True
        while todo:
            a = todo.pop()
            while True:
                if a in seen: break
                if not s.inrom(a): break
                if a in s.code and not check_only: break
                ins = s.decode(a)
                if ins is None:
                    if check_only or first: return None
                    break
                first = False
                mn, m, v, n = ins
                for k in range(a, a + n):
                    if k != a and k in seen:
                        if check_only: return None
                        break
                seen[a] = ins
                if check_only and len(seen) > 40: return seen
                if m == "rel":
                    if not s.inrom(v):
                        if check_only: return None
                        seen.pop(a); break
                    todo.append(v)
                elif mn in ("JSR", "JMP") and m == "abs":
                    if s.inrom(v): todo.append(v)
                    elif v < 0x100:
                        if check_only: return None
                        seen.pop(a); break
                if mn in END: break
                a += n
        return seen
    def add_code(s, start, kind):
        seen = s.trace(start)
        if seen is None: return False
        for a, ins in seen.items():
            if any(s.cover[k - s.base] for k in range(a, a + ins[3])): continue
            s.code[a] = ins
            s.cover[a - s.base] = 1
            for k in range(1, ins[3]): s.cover[a + k - s.base] = 2
            mn, m, v, n = ins
            if m == "rel" and s.inrom(v): s.labels.setdefault(v, "loc")
            elif mn == "JSR" and s.inrom(v): s.labels[v] = "sub"
            elif mn == "JMP" and m == "abs" and s.inrom(v): s.labels.setdefault(v, "loc")
        if start not in s.code: return False
        s.labels.setdefault(start, kind)
        return True

def read_syms(files):
    names = {}
    for f in files:
        for line in open(f, encoding="utf-8"):
            m = re.match(r"\s*\.const\s+(\w+)\s*=\s*\$([0-9A-Fa-f]+)", line)
            if m: names.setdefault(int(m.group(2), 16), m.group(1))
    return names

def run(rom, seeds, vectors, table_scan, tables):
    for a in seeds: rom.add_code(a, "sub")
    for a in vectors:
        if rom.inrom(a) and rom.inrom(a + 1):
            t = rom.word(a)
            if rom.inrom(t): rom.add_code(t, "sub")
    # Zeigertabellen: wiederholt, bis nichts Neues hinzukommt
    for start, count, adj in tables:
        for k in range(count):
            t = rom.word(start + 2 * k) + adj
            if rom.inrom(t): rom.add_code(t, "sub")
        rom.tables[start] = (count, adj)
    a = rom.base
    while a < rom.end:   # Texte (Bit 7 am Ende, mind. 5 Zeichen) in unerkanntem Bereich schützen
        if rom.cover[a - rom.base] == 0:
            n = text_run(rom, a, rom.end)
            if n >= 5 and rom.byte(a + n - 1) & 0x80 and all(rom.cover[k - rom.base] == 0 for k in range(a, a + n)):
                for k in range(a, a + n): rom.cover[k - rom.base] = 3
                a += n; continue
        a += 1
    bad = set(); good = {}
    while table_scan:
        new = 0
        a = rom.base
        while a < rom.end - 1:
            if rom.cover[a - rom.base] == 0 and rom.cover[a + 1 - rom.base] == 0:
                for adj in (0, 1):
                    t = rom.word(a) + adj
                    if rom.inrom(t) and rom.cover[t - rom.base] == 0 and t not in bad and rom.decode(t) is not None:
                        if t not in good:
                            seen = rom.trace(t, check_only=True)
                            good[t] = bool(seen and len(seen) >= 4 and any(i[0] in END for i in seen.values()))
                        if not good[t]: bad.add(t); continue
                        if rom.add_code(t, "sub"): new += 1; break
                        bad.add(t)
            a += 1
        if not new: break

def fmt_operand(rom, mn, m, v):
    nm = lambda x: rom.names.get(x)
    if m == "imm": return "#$%02X" % v
    if m == "acc": return "A"
    if m == "imp": return ""
    if m == "rel": return rom.lab(v)
    if m in ("abs", "zp"):
        w = 4 if m == "abs" else 2
        if m == "abs" and rom.inrom(v) and mn in ("JSR", "JMP"): return rom.lab(v)
        return nm(v) or "$%0*X" % (w, v)
    if m == "ind": return "(%s)" % (nm(v) or "$%04X" % v)
    base = nm(v) or ("$%04X" % v if SIZE[m] == 3 else "$%02X" % v)
    return {"zpx": base + ",X", "zpy": base + ",Y", "abx": base + ",X", "aby": base + ",Y", "izx": "(%s,X)" % base, "izy": "(%s),Y" % base}[m]

def ascii_of(bs): return "".join(chr(b & 0x7F) if 0x20 <= (b & 0x7F) < 0x7F else "." for b in bs)

def text_run(rom, a, limit):
    """Texterkennung: Zeichenfolge mit Bit 7 am Ende (Schlüsselwort/Meldung) oder 0-Ende; gibt Länge oder 0"""
    n = 0
    while a + n < limit:
        b = rom.byte(a + n)
        if b & 0x80:
            ok = n >= 1 and (b & 0x7F) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 !\"#$%&'()*+,-./:;<=>?" and all(c in b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 $%#(')?.,-*" for c in rom.d[a - rom.base: a - rom.base + n]) and (n >= 2 or (b & 0x7F) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZ")
            return n + 1 if ok else 0
        if b == 0: return n + 1 if n >= 3 else 0
        if not 0x20 <= b < 0x7F: return 0
        n += 1
    return 0

def emit(rom, title, out):
    names = rom.names
    def lab(a):
        if a in rom.rename: return rom.rename[a]
        if rom.inrom(a) and a in rom.names and a in rom.labels: return rom.names[a]
        if a in rom.labels: return ("sub_%04X" if rom.labels[a] == "sub" else "loc_%04X") % a
        return "L_%04X" % a if rom.inrom(a) else "$%04X" % a
    rom.lab = lab
    # Textnamen automatisch: s_<TEXT>
    used = set(rom.rename.values())
    a = rom.base
    while a < rom.end:
        if rom.cover[a - rom.base] == 3 and (a == rom.base or rom.cover[a - 1 - rom.base] != 3) and a not in rom.rename:
            n = text_run(rom, a, rom.end)
            if n:
                t = "".join(chr(c & 0x7F) for c in rom.d[a - rom.base: a - rom.base + n]).strip()
                nm = "s_" + re.sub(r"[^A-Za-z0-9]+", "_", t).strip("_")[:24]
                while nm in used: nm += "_"
                used.add(nm); rom.rename[a] = nm
        a += 1
    w = out.append
    w("; " + title)
    w("; Rekursiv disassembliert mit tools/romdis.py; Original-ROM, urheberrechtlich geschützt, nur lokal verwenden.")
    nc = sum(1 for c in rom.cover if c == 1)
    w("; Befehle erkannt: %d, Codebytes: %d von %d (%.0f %%)" % (len(rom.code), sum(1 for c in rom.cover if c in (1, 2)), len(rom.cover), 100 * sum(1 for c in rom.cover if c in (1, 2)) / len(rom.cover)))
    w("")
    a = rom.base
    while a < rom.end:
        c = rom.cover[a - rom.base]
        if a in rom.labels or a in rom.rename:
            if rom.labels.get(a) == "sub": w("")
            w("%s:" % rom.lab(a))
        if c == 1:
            mn, m, v, n = rom.code[a]
            byts = " ".join("%02X" % rom.byte(a + k) for k in range(n))
            ops = fmt_operand(rom, mn, m, v)
            cm = ""
            if mn in ("JSR", "JMP") and m == "abs" and not rom.inrom(v) and v in names: cm = ""
            w("  %04X  %-8s  %-4s %s%s" % (a, byts, mn, ops, cm))
            a += n
            continue
        if c == 2:   # Folgebyte eines Befehls ohne eigenen Anfang (Überlappung): einzeln ausgeben
            w("  %04X  .byte   $%02X" % (a, rom.byte(a))); a += 1; continue
        # Daten: bis zum nächsten Code/Label
        b = a
        while b < rom.end and rom.cover[b - rom.base] in (0, 3) and (b == a or (b not in rom.labels and b not in rom.tables and b not in rom.rename)): b += 1
        # Wörter einer Zeigertabelle?
        while a < b:
            if a in rom.tables:
                cnt, adj = rom.tables[a]
                for k in range(cnt):
                    t = rom.word(a + 2 * k); tt = (t + adj) & 0xFFFF
                    ex = ("%s-%d" % (rom.lab(tt), adj) if adj else rom.lab(tt)) if tt in rom.labels else "$%04X" % t
                    w("  %04X  %02X %02X     .word %s" % (a + 2 * k, rom.byte(a + 2 * k), rom.byte(a + 2 * k + 1), ex))
                a += 2 * cnt; continue
            if rom.cover[a - rom.base] in (1, 2): break
            n = text_run(rom, a, b)
            if n >= 2 and rom.byte(a + n - 1) & 0x80 or n >= 5:
                s = rom.d[a - rom.base: a - rom.base + n]
                w("  %04X  .text   \"%s\"%s" % (a, ascii_of(s), "  ; Bit 7 am Ende" if s[-1] & 0x80 else ""))
                a += n; continue
            n = min(8, b - a)
            # Text im Datenstück früh abbrechen lassen
            for k in range(1, n):
                if text_run(rom, a + k, b) >= 4: n = k; break
            s = rom.d[a - rom.base: a - rom.base + n]
            w("  %04X  .byte   %-31s ; %s" % (a, ",".join("$%02X" % x for x in s), ascii_of(s)))
            a += n

def main():
    ap = argparse.ArgumentParser(
        description=__doc__,
        epilog="""Beispiele:
  python3 tools/romdis.py "$(vcc roms path c64/kernal)" E000 ausgabe/c64-kernal-E000.txt --title 'C64 KERNAL ($E000-$FFFF)' --syms data/sys/c64/kernal.asm --vector FFFA --vector FFFC --vector FFFE
  python3 tools/romdis.py "$(vcc roms path c64/basic)" A000 ausgabe/c64-basic-A000.txt --names tools/names/c64-basic.names --words A000:2 --table A00C:35:1 --seed B391

Gelesen werden nur ROM sowie die mit --syms/--names angegebenen Dateien; geschrieben wird nur AUSGABE (UTF-8, Zeilenende LF, vorhandene Datei wird überschrieben;
der Ordner muss existieren). Der Arbeitsordner ist egal, wenn die Pfade passend angegeben werden. Alle Adressen sind hexadezimal ohne $. Mehrfach angebbare
Optionen (--seed, --vector, --table, --jumptable, --words, --names, --syms) dürfen wiederholt werden. tools/romdis-run.sh ruft das Werkzeug für alle ROMs auf.""",
        formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("rom", help="ROM-Abbild (Binärdatei, wird nur gelesen)")
    ap.add_argument("base", help="Ladeadresse (Basisadresse) des ROMs, hexadezimal, z. B. E000")
    ap.add_argument("out", help="zu schreibende Listing-Datei (Text)")
    ap.add_argument("--title", default="ROM", help="Überschrift des Listings (Standard: ROM)")
    ap.add_argument("--seed", action="append", default=[], help="zusätzlicher Einstiegspunkt (Adresse hexadezimal, z. B. B391)")
    ap.add_argument("--vector", action="append", default=[], help="Adresse eines 16-Bit-Zeigers auf Code (z. B. FFFC)")
    ap.add_argument("--table", action="append", default=[], help="START:ANZAHL[:ADJ] Zeigertabelle (ADJ=1 bei RTS-Tabellen)")
    ap.add_argument("--jumptable", action="append", default=[], help="START:ANZAHL Tabelle aus 3-Byte-JMP/JSR-Einträgen")
    ap.add_argument("--words", action="append", default=[], help="START:ANZAHL einfache Wortliste (Vektoren), ohne Einstieg")
    ap.add_argument("--names", action="append", default=[], help="Datei mit Zeilen 'ADRESSE NAME' (Hex), überschreibt die Labelnamen")
    ap.add_argument("--syms", action="append", default=[], help="Assemblerdatei mit .const-Zeilen, deren Namen als Adressnamen dienen (z. B. data/sys/c64/basic.asm)")
    ap.add_argument("--no-table-scan", action="store_true", help="keine automatische Suche nach Zeigertabellen (Verfahren 2)")
    a = ap.parse_args()
    base = int(a.base, 16)
    rom = Rom(open(a.rom, "rb").read(), base, read_syms(a.syms))
    seeds = [int(x, 16) for x in a.seed]
    for j in a.jumptable:
        s, n = j.split(":"); s = int(s, 16)
        seeds += [s + 3 * k for k in range(int(n))]
    tables = []
    for t in a.table:
        p = t.split(":"); tables.append((int(p[0], 16), int(p[1]), int(p[2]) if len(p) > 2 else 0))
    for f in a.names:
        for line in open(f, encoding="utf-8"):
            line = line.split(";")[0].strip()
            if line:
                ad, nm = line.split()[:2]; rom.rename[int(ad, 16)] = nm
    for t in a.words:
        p = t.split(":"); rom.tables[int(p[0], 16)] = (int(p[1]), 0)
    run(rom, seeds, [int(x, 16) for x in a.vector], not a.no_table_scan, tables)
    out = []; emit(rom, a.title, out)
    open(a.out, "w", encoding="utf-8").write("\n".join(out) + "\n")
    cov = sum(1 for c in rom.cover if c in (1, 2))
    print("%s: %d Befehle, %.0f %% Code" % (a.out, len(rom.code), 100 * cov / len(rom.cover)))

if __name__ == "__main__":
    main()
