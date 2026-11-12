#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt Namensdateien (`ADRESSE NAME`) für tools/romdis.py: BASIC-Befehle und -Funktionen aus den Zeigertabellen und den Dialektdateien,
dazu die in der Systemdokumentation veröffentlichten Namen der KERNAL-/OS-/BASIC-Einsprünge. Enthält nur Adressen und Namen, keinen ROM-Inhalt.
Aufruf: romnames.py   (liest die ROMs aus dem ROM-Verzeichnis über `vcc roms path`, schreibt tools/names/*.names)"""
import argparse, json, os, re
_ap = argparse.ArgumentParser(
    description="""Erzeugt Namensdateien (`ADRESSE NAME`) für tools/romdis.py: BASIC-Befehle und -Funktionen aus den Zeigertabellen und den Dialektdateien, dazu die in der
Systemdokumentation veröffentlichten Namen der KERNAL-/OS-/BASIC-Einsprünge. Enthält nur Adressen und Namen, keinen ROM-Inhalt.

Liest die ROMs des Benutzers aus dem ROM-Verzeichnis (über `vcc roms path <system/rolle>`; das Programm vcc ist mit der Umgebungsvariablen VCC wählbar) und
data/basic/*.dialect.json. Schreibt tools/names/{c64-basic,c64-kernal,atari-os-b,atari-xl,atari-basic}.names sowie tools/names/c64-basic.local.names und
c64-kernal.local.names (lokal, nicht einchecken; nur gefüllt, wenn die Umgebungsvariable KLASEK_HEADS auf eine lokale Abschnittsliste zeigt); vorhandene
Dateien werden überschrieben. Die Dateien werden ohne ausdrückliche Kodierung geschrieben: unter Windows vorher PYTHONUTF8=1 setzen. Pfade gelten relativ zum
Ort des Skripts, der Arbeitsordner ist egal.""",
    epilog="""Beispiele:
  python3 tools/romnames.py
  KLASEK_HEADS=pfad/zur/abschnittsliste.txt python3 tools/romnames.py

Das Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.""",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_ap.parse_args()
root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
def rd(p):
    """Datei aus dem Projekt, oder (Angabe `rom:system/rolle`) eine ROM-Datei aus dem ROM-Verzeichnis von VintageCoder (`vcc roms path`, VCC=… wählt vcc)"""
    if p.startswith("rom:"):
        import subprocess
        p = subprocess.check_output([os.environ.get("VCC", "vcc"), "roms", "path", p[4:]], text=True).strip()
    return open(os.path.join(root, p), "rb").read()
words = lambda rom, off, n: [rom[off + 2 * i] | rom[off + 2 * i + 1] << 8 for i in range(n)]
def ident(n):
    n = n.replace("#", "_FILE").replace("$", "_S").replace("(", "").replace("?", "QMARK").replace(">", "GT").replace("<", "LT").replace("=", "EQ").replace("+", "ADD").replace("-", "SUB").replace("*", "MUL").replace("/", "DIV").replace("^", "POW")
    return re.sub(r"[^A-Za-z0-9_]", "_", n)
def tokens(dialect):
    d = json.load(open(os.path.join(root, "data/basic", dialect + ".dialect.json")))
    return {0x80 + i: t[0] for i, t in enumerate(d["tokens"])}
STOP = {"the", "a", "an", "to", "of", "and", "in", "on", "for", "is", "are", "from", "with", "then", "do", "into", "at", "by", "or", "as"}
SKIP = re.compile(r"constant|unused|spare|orphan|not referenced|\?\?|^start of|table|keyword|message|signature|vectors?$|^# |streams|limits for|values|jiffy|series", re.I)
def derive(h):
    """Kurzname (englisch, snake_case) aus einer Abschnittsüberschrift; Überschriften mit Tabellen/Konstanten bleiben unbenannt"""
    if SKIP.search(h): return None
    h = re.split(r"[,.(]| - ", h)[0].strip()
    m = re.match(r"perform (.*)", h, re.I)
    if m: h = m.group(1)
    h = h.replace("$", "S").replace("#", " file ")
    ws = [w for w in re.findall(r"[A-Za-z0-9]+", h) if w.lower() not in STOP]
    return "_".join(w.lower() for w in ws[:5]) or None
def klasek(heads_file, lo, hi, taken):
    """Zusätzliche Namen aus der (lokal gelesenen) Abschnittsliste des kommentierten C64-ROM-Listings; nur Adresse -> eigener Kurzname"""
    out = []; used = set(taken.values())
    if not heads_file or not os.path.exists(heads_file): return out
    for line in open(heads_file, encoding="utf-8"):
        a, h = line.rstrip("\n").split(" ", 1); a = int(a, 16)
        if not lo <= a <= hi or a in taken: continue
        n = derive(h)
        if not n: continue
        n = "r_" + n
        while n in used: n += "_"
        used.add(n); out.append((a, n))
    return out
def write(name, lines):
    with open(os.path.join(root, "tools/names", name), "w") as f:
        f.write("; Namen für tools/romdis.py (Adresse Name); aus Systemdokumentation und Dialektdateien, ohne ROM-Inhalt\n")
        seen = set()
        for a, n in lines:
            if a in seen: continue
            seen.add(a); f.write("%04X %s\n" % (a, n))
    print(name, len(seen))

# ---------- C64 BASIC (901226-01) ----------
rom = rd("rom:c64/basic"); tk = tokens("cbm_v2")
L = []
api = {0xB850: "FSUB", 0xB853: "FSUBT", 0xB867: "FADD", 0xB86A: "FADDT", 0xBA28: "FMULT", 0xBA2B: "FMULTT", 0xBB0F: "FDIV", 0xBB12: "FDIVT", 0xBF7B: "FPWRT", 0xBF78: "FPWR",
       0xBF71: "SQR", 0xBFED: "EXP", 0xB9EA: "LOG", 0xBFB4: "NEGOP", 0xBC58: "ABS", 0xBC39: "SGN", 0xBCCC: "INT", 0xBC5B: "FCOMP", 0xBCF3: "FIN", 0xBDDD: "FOUT",
       0xBBA2: "MOVFM", 0xBBD4: "MOVMF", 0xBC0C: "MOVAF", 0xBBFC: "MOVFA", 0xBC1B: "ROUND", 0xBA8C: "CONUPK", 0xB391: "GIVAYF", 0xB1AA: "FACINX", 0xAB1E: "STROUT",
       0xBDCD: "LINPRT", 0xB7F7: "GETADR", 0xB79B: "GETBYTC", 0xAD9E: "FRMEVL", 0xAD8A: "FRMNUM", 0xAEFD: "CHKCOM", 0xAEFA: "CHKOPN", 0xAEF7: "CHKCLS", 0xAEFF: "SYNCHR",
       0xA437: "ERROR", 0xA43A: "ERRMSG", 0xA474: "READY", 0xA480: "MAIN", 0xA483: "MAIN1", 0xA560: "INLIN", 0xA57C: "CRUNCH", 0xA613: "FINDLN", 0xA642: "NEW_CMD",
       0xA65E: "CLR_CMD", 0xA68E: "RUNC", 0xA69C: "LIST_CMD", 0xA7AE: "NEWSTT", 0xA7E4: "GONE"}
L += list(api.items())
for i, w in enumerate(words(rom, 0x00C, 35)): L.append((w + 1, "cmd_" + ident(tk.get(0x80 + i, "X%d" % i))))
for i, w in enumerate(words(rom, 0x052, 23)): L.append((w + 1, "fn_" + ident(tk.get(0xB4 + i, "X%d" % i))))
for a, n in ((0xA000, "vec_cold_start"), (0xA002, "vec_warm_start"), (0xA004, "id_cbmbasic"), (0xA00C, "table_statements"), (0xA052, "table_functions"), (0xA080, "table_operators"),
             (0xA09E, "table_keywords"), (0xA19E, "table_errors"), (0xA326, "table_error_pointers")):
    L.append((a, n))
for a, n in ((0xAFE8 + 1, "op_AND"), (0xAFE5 + 1, "op_OR"), (0xAED3 + 1, "op_NOT"), (0xB015 + 1, "op_CMP")): L.append((a, n))
# Schlüsselwörter einzeln als Datenlabel (kw_PRINT …): Tabelle ab $A09E, jedes Wort endet mit gesetztem Bit 7
a = 0x09E; n = 0
while a < 0x19E:
    st = a
    while not rom[a] & 0x80: a += 1
    name = "".join(chr(c & 0x7F) for c in rom[st:a + 1])
    L.append((0xA000 + st, "kw_" + ident(name))); a += 1
HEADS = os.environ.get("KLASEK_HEADS")
write("c64-basic.names", L)
write("c64-basic.local.names", klasek(HEADS, 0xA000, 0xBFFF, dict(L)))   # abgeleitet aus fremden Überschriften: lokal, nicht einchecken

# ---------- C64 KERNAL (901227-03) ----------
rom = rd("rom:c64/kernal")
L = [(0xFCE2, "RESET"), (0xFE43, "nmi_entry"), (0xFF48, "irq_entry"), (0xEA31, "irq_handler"), (0xEA81, "irq_return"), (0xEA87, "SCNKEY_impl"), (0xFE66, "brk_handler"),
     (0xFE47, "nmi_handler"), (0xFFFA, "vec_nmi"), (0xFFFC, "vec_reset"), (0xFFFE, "vec_irq"), (0xFD30, "table_ram_vectors"),
     (0xFF5B, "CINT_impl"), (0xFDA3, "IOINIT_impl"), (0xFD50, "RAMTAS_impl"), (0xFD15, "RESTOR_impl"), (0xFD1A, "VECTOR_impl"),
     (0xFE18, "SETMSG_impl"), (0xEDB9, "SECOND_impl"), (0xEDC7, "TKSA_impl"), (0xFE25, "MEMTOP_impl"), (0xFE34, "MEMBOT_impl"), (0xFE21, "SETTMO_impl"),
     (0xEE13, "ACPTR_impl"), (0xEDDD, "CIOUT_impl"), (0xEDEF, "UNTLK_impl"), (0xEDFE, "UNLSN_impl"), (0xED0C, "LISTEN_impl"), (0xED09, "TALK_impl"),
     (0xFE07, "READST_impl"), (0xFE00, "SETLFS_impl"), (0xFDF9, "SETNAM_impl"), (0xF6E4, "SETTIM_impl"), (0xF6DD, "RDTIM_impl"), (0xF69B, "UDTIM_impl"),
     (0xE505, "SCREEN_impl"), (0xE50A, "PLOT_impl"), (0xE500, "IOBASE_impl"), (0xE544, "clear_screen"), (0xE566, "home_cursor"), (0xE5A0, "init_vic"),
     (0xE264, "COS"), (0xE26B, "SIN"), (0xE2B4, "TAN"), (0xE30E, "ATN"), (0xE097, "RND")]
vec = ["irq_handler", "brk_handler", "nmi_handler", "OPEN_impl", "CLOSE_impl", "CHKIN_impl", "CKOUT_impl", "CLRCHN_impl", "CHRIN_impl", "CHROUT_impl", "STOP_impl", "GETIN_impl",
       "CLALL_impl", "usrcmd_handler", "LOAD_impl", "SAVE_impl"]
for i, w in enumerate(words(rom, 0x1D30, 16)): L.append((w, vec[i]))
write("c64-kernal.names", L)
write("c64-kernal.local.names", klasek(HEADS, 0xE000, 0xFFFF, dict(L)))

# ---------- Atari OS (OS-B und XL/XE) ----------
fp = [(0xD800, "AFP"), (0xD8E6, "FASC"), (0xD9AA, "IFP"), (0xD9D2, "FPI"), (0xDA60, "FSUB"), (0xDA66, "FADD"), (0xDADB, "FMUL"), (0xDB28, "FDIV"), (0xDD89, "FLD0R"),
      (0xDD8D, "FLD0P"), (0xDD98, "FLD1R"), (0xDD9C, "FLD1P"), (0xDDA7, "FST0R"), (0xDDAB, "FST0P"), (0xDDB6, "FMOVE"), (0xDDC0, "EXP"), (0xDDCC, "EXP10"), (0xDECD, "LOG"), (0xDED1, "LOG10")]
os_ = [(0xE450, "DISKIV"), (0xE453, "DSKINV"), (0xE456, "CIOV"), (0xE459, "SIOV"), (0xE45C, "SETVBV"), (0xE45F, "SYSVBV"), (0xE462, "XITVBV"), (0xE465, "SIOINV"), (0xE468, "SENDEV"),
       (0xE46B, "INTINV"), (0xE46E, "CIOINV"), (0xE471, "BLKBDV"), (0xE474, "WARMSV"), (0xE477, "COLDSV"), (0xE47D, "RBLOKV"), (0xE480, "CSOPIV"),
       (0xFFFA, "vec_nmi"), (0xFFFC, "vec_reset"), (0xFFFE, "vec_irq")]
def vecs(path, base):
    r = rd(path); o = 0xFFFA - base
    return [(r[o + 2 * i] | r[o + 2 * i + 1] << 8, n) for i, n in enumerate(("nmi_entry", "reset_entry", "irq_entry"))]
write("atari-os-b.names", fp + os_ + vecs("rom:atari8/os-a", 0xD800))
write("atari-xl.names", fp + os_ + vecs("rom:atari8/os-xl", 0xC000))

# ---------- Atari BASIC ----------
rom = rd("rom:atari8/basic"); L = [(0xA4A1, "table_statement_names")]
a = 0x4A1
while a < 0x5F1:
    st = a
    while not rom[a] & 0x80: a += 1
    name = "".join(chr(c & 0x7F) for c in rom[st:a + 1]); a += 3
    L.append((0xA000 + st, "stmt_" + ident(name)))
write("atari-basic.names", L)
