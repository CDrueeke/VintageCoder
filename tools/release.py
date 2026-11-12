#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Release-Auswahl: liest FEATURES.md und release/registry.json und baut daraus ein Staging-Verzeichnis.

Die Auswahl steht ausschliesslich in FEATURES.md (Haekchen). Die Technik (Abhaengigkeiten, Dateien je Zeile) steht in
release/registry.json. Installer und Archive packen nur noch das Staging ein.
Nur Python-Standardbibliothek (getestet mit Python 3.9).
"""
import argparse, fnmatch, hashlib, json, os, re, shutil, struct, subprocess, sys, tarfile, time, zlib

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
STANDS = ["Konzept", "Spike", "In Arbeit", "Beta", "Stabil"]          # aufsteigend; ab "Beta" ausliefern
BETA = STANDS.index("Beta")
PREFIX_KIND = {"IDE": "ide", "STU": "studio", "EMU": "link", "HW": "link", "SYS": "target", "EX": "examples"}
SKIP_NAMES = {".DS_Store", "Thumbs.db"}
# Lage der Programme und Daten im Staging je Betriebssystem. Linux: FHS-Baum, aus dem .deb, .rpm und tar.gz gebildet werden (Konzept 8.4).
BIN_SUB = {"windows": "", "macos": "bin", "linux": "usr/bin"}
SHARE_SUB = {"linux": "usr/share/vintagecoder"}              # darunter data/ und examples/ (sonst direkt im Staging)
DOC_SUB = {"linux": "usr/share/doc/vintagecoder"}            # Lizenzen, README, docs/, editors/ (sonst direkt im Staging)
TEXT_EXT = {".md", ".json", ".asm", ".inc", ".bas", ".txt", ".html", ".iss", ".sh", ".py", ".toml", ".cfg", ".1", ".vcprj", ".ps1", ".command", ".css", ".js", ".ts"}


class ReleaseError(Exception):
    pass


# ---------------------------------------------------------------- Parser

def strip_comments(text):
    return re.sub(r"<!--.*?-->", lambda m: "\n" * m.group(0).count("\n"), text, flags=re.S)


def split_row(line):
    s = line.strip()
    if not (s.startswith("|") and s.endswith("|")):
        return None
    return [c.strip() for c in s[1:-1].split("|")]


ID_CELL = re.compile(r"^`([A-Z][A-Z0-9]*-[A-Z0-9-]+)`$")
CHECK_CELL = re.compile(r"^\[\s*([xX]?)\s*\]$")


def parse_features(text):
    """Liest nur Tabellenzeilen mit Kennung in Backticks (erste Zelle) und Haekchen (letzte Zelle). Gibt (rows, problems) zurueck."""
    rows, problems = [], []
    section = ""
    for n, line in enumerate(strip_comments(text).splitlines(), 1):
        if line.startswith("## "):
            section = line[3:].strip()
            continue
        cells = split_row(line)
        if not cells or len(cells) < 3:
            continue
        m = ID_CELL.match(cells[0])
        if not m:
            continue
        c = CHECK_CELL.match(cells[-1]) if cells[-1] else None
        if cells[-1] and not c:
            problems.append("FEATURES.md Zeile %d: unlesbares Haekchen '%s' bei %s" % (n, cells[-1], m.group(1)))
            continue
        stand = cells[-2] if len(cells) >= 4 else ""
        rows.append({"id": m.group(1), "label": cells[1], "stand": stand, "checked": bool(c and c.group(1)), "section": section, "line": n})
    return rows, problems


def parse_status(text):
    """Kennung -> Stand aus STATUS.md (Tabellen mit Spalte Stand)."""
    out, cols = {}, None
    for line in strip_comments(text).splitlines():
        cells = split_row(line)
        if not cells:
            cols = None
            continue
        if cells[0] == "Kennung":
            cols = cells
            continue
        m = ID_CELL.match(cells[0])
        if m and cols and "Stand" in cols and len(cells) == len(cols):
            out[m.group(1)] = cells[cols.index("Stand")]
    return out


# ---------------------------------------------------------------- Globs

def brace(p):
    m = re.search(r"\{([^{}]*)\}", p)
    if not m:
        return [p]
    out = []
    for alt in m.group(1).split(","):
        out += brace(p[:m.start()] + alt + p[m.end():])
    return out


def glob_rx(p):
    i, rx = 0, ""
    while i < len(p):
        if p.startswith("**/", i):
            rx += "(?:.*/)?"; i += 3
        elif p.startswith("**", i):
            rx += ".*"; i += 2
        elif p[i] == "*":
            rx += "[^/]*"; i += 1
        elif p[i] == "?":
            rx += "[^/]"; i += 1
        else:
            rx += re.escape(p[i]); i += 1
    return re.compile("^" + rx + "$")


class Globs:
    def __init__(self, patterns):
        self.pats = [(p, glob_rx(b)) for p in patterns for b in brace(p)]
        self.hit = set()

    def match(self, path):
        ok = False
        for p, rx in self.pats:
            if rx.match(path):
                self.hit.add(p); ok = True
        return ok

    def unmatched(self):
        names = {p for p, _ in self.pats}
        return sorted(names - self.hit)


# ---------------------------------------------------------------- Dateien

def walk(base, exclude_dirs=(), root=None):
    """Alle Dateien unter base als relative Pfade (Posix) zu root."""
    root = root or ROOT
    out = []
    start = os.path.join(root, base)
    for d, dirs, files in os.walk(start):
        dirs[:] = sorted(x for x in dirs if x not in exclude_dirs and not x.startswith("._"))
        for f in sorted(files):
            if f in SKIP_NAMES or f.startswith("._"):
                continue
            out.append(os.path.relpath(os.path.join(d, f), root).replace(os.sep, "/"))
    return out


def load_json(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def load_registry(path=None):
    path = path or os.path.join(ROOT, "release", "registry.json")
    reg = load_json(path)
    if reg.get("format") != 1:
        raise ReleaseError("registry.json: unbekanntes Format %r" % reg.get("format"))
    return reg


def example_projects(reg, root=None):
    """Alle Projektordner unter examples/ (Ordner mit .vcprj): Liste von 'ordner/projekt'."""
    root = root or ROOT
    ign = Globs(reg.get("examples_ignore", []))
    out = []
    base = os.path.join(root, "examples")
    for d in sorted(os.listdir(base)):
        dp = os.path.join(base, d)
        if not os.path.isdir(dp) or d.startswith("."):
            continue
        for p in sorted(os.listdir(dp)):
            pp = os.path.join(dp, p)
            if os.path.isdir(pp) and any(f.endswith(".vcprj") for f in os.listdir(pp)):
                out.append("%s/%s" % (d, p))
    return out


def project_package(reg, proj):
    """Genau ein Paket je Projekt. Gibt (kennung|None, [alle Treffer]) zurueck."""
    hits = []
    for fid, f in reg["features"].items():
        if f["kind"] != "examples":
            continue
        for pat in f.get("projects", []):
            if glob_rx(pat).match(proj):
                hits.append(fid); break
    return (hits[0] if len(hits) == 1 else None), hits


# ---------------------------------------------------------------- Auswahl

class Result:
    def __init__(self):
        self.errors, self.warnings, self.notes = [], [], []


def read_selection(features_path, reg):
    with open(features_path, encoding="utf-8") as f:
        rows, problems = parse_features(f.read())
    return rows, problems


def check_ids(rows, reg, res):
    seen = {}
    for r in rows:
        if r["id"] in seen:
            res.errors.append("FEATURES.md: Kennung %s steht doppelt (Zeilen %d und %d)" % (r["id"], seen[r["id"]], r["line"]))
        seen[r["id"]] = r["line"]
        f = reg["features"].get(r["id"])
        if f is None:
            res.errors.append("FEATURES.md Zeile %d: Kennung %s fehlt in release/registry.json" % (r["line"], r["id"]))
            continue
        pk = PREFIX_KIND.get(r["id"].split("-")[0])
        if pk != f["kind"]:
            res.errors.append("%s: Praefix passt nicht zur Art '%s' in der Registry" % (r["id"], f["kind"]))
        if r["stand"] not in STANDS:
            res.errors.append("FEATURES.md Zeile %d: unbekannter Stand '%s' bei %s (erlaubt: %s)" % (r["line"], r["stand"], r["id"], ", ".join(STANDS)))
    for fid in reg["features"]:
        if fid not in seen:
            res.errors.append("Registry-Kennung %s steht nicht in FEATURES.md" % fid)


def resolve(rows, reg, allow_unfinished=False, auto_deps=False):
    """Wertet die Auswahl aus. Gibt (selected:set, skipped:set, Result) zurueck."""
    res = Result()
    feats = reg["features"]
    sel = {r["id"] for r in rows if r["checked"] and r["id"] in feats}
    stand = {r["id"]: r["stand"] for r in rows}

    if auto_deps:
        changed = True
        while changed:
            changed = False
            for fid in sorted(sel):
                for need in feats[fid].get("needs", []):
                    if need not in sel:
                        sel.add(need); changed = True
                        res.notes.append("--auto-deps: %s ergaenzt (wird von %s gebraucht)" % (need, fid))

    skipped = set()
    if "IDE-APP" not in sel:
        for fid in sorted(sel):
            if feats[fid]["kind"] == "studio":
                skipped.add(fid)
                res.notes.append("%s uebersprungen: ohne IDE-APP gibt es keine Studios" % fid)
    eff = sel - skipped

    for fid in sorted(eff):
        f = feats[fid]
        for need in f.get("needs", []):
            if need not in eff:
                res.errors.append("%s braucht %s: bitte %s in FEATURES.md ankreuzen oder %s abwaehlen" % (fid, need, need, fid))
        anyn = f.get("needs_any", [])
        if anyn and not any(n in eff for n in anyn):
            res.errors.append("%s braucht mindestens eines von %s: bitte eines davon ankreuzen oder %s abwaehlen" % (fid, ", ".join(anyn), fid))
        for other, why in f.get("soft", []):
            if other not in eff:
                res.warnings.append("%s: %s nicht gewaehlt (%s)" % (fid, other, why))
        si = STANDS.index(stand[fid]) if stand.get(fid) in STANDS else 0
        if si < BETA:
            msg = "%s hat den Stand '%s' (unter Beta)" % (fid, stand.get(fid))
            if fid in reg.get("preview_ok", []):
                res.warnings.append(msg + ": als Vorschau ausgeliefert (preview_ok in release/registry.json)")
            elif allow_unfinished:
                res.warnings.append(msg + ": ausgeliefert wegen --allow-unfinished")
            else:
                res.errors.append(msg + ": abwaehlen oder --allow-unfinished verwenden")

    sys_ids = [s for s in eff if feats[s]["kind"] == "target"]
    for s in sorted(sys_ids):
        tid = feats[s]["id"]
        if not any(feats[e]["kind"] == "link" and s in feats[e].get("needs", []) + feats[e].get("needs_any", []) for e in eff):
            res.warnings.append("%s: keine Anbindung (EMU-/HW-) fuer dieses System gewaehlt: nur Bau, kein Start" % s)
    return eff, skipped, res


def selected_targets(eff, reg):
    return [reg["features"][f] for f in sorted(eff) if reg["features"][f]["kind"] == "target"]


def examples_to_ship(eff, reg, root=None):
    """Liste ('ordner/projekt', paket) der auszuliefernden Beispielprojekte."""
    dirs = {t["example_dir"] for t in selected_targets(eff, reg)}
    out = []
    for proj in example_projects(reg, root):
        pkg, _ = project_package(reg, proj)
        if pkg and pkg in eff and proj.split("/")[0] in dirs:
            out.append((proj, pkg))
    return out


def data_claims(reg):
    """Gibt (claims: feature -> Globs, core: Globs, never: Globs)."""
    claims = {fid: Globs(f.get("files", [])) for fid, f in reg["features"].items() if f.get("files")}
    return claims, Globs(reg.get("core", [])), Globs(reg.get("never", []))


def data_files_to_ship(eff, reg, root=None):
    """Dateien unter data/ nach Registry: core + Beanspruchungen gewaehlter Zeilen - never. Gibt (shipped, unclaimed) zurueck."""
    claims, core, never = data_claims(reg)
    shipped, unclaimed = [], []
    for p in walk("data", root=root):
        if never.match(p):
            continue
        claimed_by = [fid for fid, g in claims.items() if g.match(p)]
        in_core = core.match(p)
        if in_core or any(fid in eff for fid in claimed_by):
            shipped.append(p)
        elif not claimed_by:
            unclaimed.append(p)
            shipped.append(p)      # nicht beanspruchte Dateien gelten als core (check warnt)
    return shipped, unclaimed


def mentioned_target_ids(reg):
    ids = set()
    for f in reg["features"].values():
        if f["kind"] == "target":
            ids.add(f["id"]); ids.add(f["example_dir"])
        elif f["kind"] == "link":
            ids.add(f["id"])
    ids |= {"atari8bit", "vcs2600", "c16", "cbm5x0"}
    return ids


# ---------------------------------------------------------------- Pruefungen am Dateisystem

INCLUDE_RX = re.compile(r'^\s*[#.]include\s+([<"])([^>"]+)[>"]', re.I | re.M)


def sys_include_closure(files, root=None):
    """Prueft #include <x/y.asm> der ausgelieferten data/sys-Dateien gegen die ausgelieferte Menge. Gibt Fehlerliste zurueck."""
    root = root or ROOT
    have = set(files)
    errs = []
    for p in files:
        if not p.startswith("data/sys/") or not p.endswith((".asm", ".inc")):
            continue
        with open(os.path.join(root, p), encoding="utf-8", errors="replace") as f:
            text = f.read()
        for m in INCLUDE_RX.finditer(text):
            kind, name = m.group(1), m.group(2)
            if kind != "<":
                continue
            if "data/sys/" + name not in have:
                errs.append("%s: #include <%s> zeigt auf eine Datei, die nicht ausgeliefert wird" % (p, name))
    for p in files:
        if p.startswith("data/cpus/") and p.endswith(".cpu.json"):
            with open(os.path.join(root, p), encoding="utf-8") as f:
                base = json.load(f).get("extends")
            if base and "data/cpus/%s.cpu.json" % base not in have:
                errs.append("%s: extends %s, die Datei wird nicht ausgeliefert" % (p, base))
    return errs


def check_filesystem(reg, res):
    claims, core, never = data_claims(reg)
    all_data = walk("data")
    sysws = mentioned_target_ids(reg)
    for p in all_data:
        for g in list(claims.values()) + [core, never]:
            g.match(p)
    for fid, g in claims.items():
        for pat in g.unmatched():
            res.errors.append("%s: Muster '%s' trifft keine Datei" % (fid, pat))
    for name, g in (("core", core), ("never", never)):
        for pat in g.unmatched():
            res.errors.append("Registry %s: Muster '%s' trifft keine Datei" % (name, pat))
    claimed = set()
    for p in all_data:
        if never.match(p) or core.match(p) or any(g.match(p) for g in claims.values()):
            claimed.add(p)
    for p in all_data:
        if p in claimed:
            continue
        parts = re.split(r"[/._-]", p)
        if any(t in sysws for t in parts):
            res.warnings.append("%s: nicht beansprucht und nennt ein System im Pfad (bitte in die Registry eintragen)" % p)
        else:
            res.notes.append("nicht beansprucht (gilt als core): %s" % p)
    # Ziele: jede Target-Datei beansprucht
    targets_present = sorted(os.path.basename(p)[:-len(".target.json")] for p in all_data if p.endswith(".target.json"))
    ids = {f["id"] for f in reg["features"].values() if f["kind"] == "target"}
    for t in targets_present:
        if t not in ids:
            res.errors.append("data/targets/%s.target.json gehoert zu keiner SYS-Zeile" % t)
    for t in sorted(ids - set(targets_present)):
        res.errors.append("SYS-Zeile mit id '%s' hat keine data/targets/%s.target.json" % (t, t))
    # Beispiele
    projs = example_projects(reg)
    dirs = {t["example_dir"] for t in reg["features"].values() if t["kind"] == "target"}
    for proj in projs:
        pkg, hits = project_package(reg, proj)
        if not hits:
            res.errors.append("examples/%s ist keinem Beispielpaket zugeordnet" % proj)
        elif len(hits) > 1:
            res.errors.append("examples/%s gehoert zu mehreren Paketen: %s" % (proj, ", ".join(hits)))
        if proj.split("/")[0] not in dirs:
            res.errors.append("examples/%s: der Ordner '%s' gehoert zu keinem Zielsystem (example_dir)" % (proj, proj.split("/")[0]))
    for fid, f in reg["features"].items():
        if f["kind"] != "examples":
            continue
        for pat in f.get("projects", []) + f.get("nobuild", []):
            rx = glob_rx(pat)
            if not any(rx.match(p) for p in projs):
                res.errors.append("%s: Projektmuster '%s' trifft kein Beispielprojekt" % (fid, pat))
        for pat in f.get("shared", []):
            rx = glob_rx(pat)
            if not any(rx.match(p) for p in walk("examples")):
                res.errors.append("%s: Muster '%s' trifft keine Datei" % (fid, pat))
    known = set(projs)
    for d in dirs:
        if not any(p.startswith(d + "/") for p in projs):
            res.warnings.append("Zielsystem-Ordner examples/%s enthaelt kein Projekt (Kriterium 3: Beispiel zeigt das Feature)" % d)
    # Quellen liegen: Beispiele ohne .asm/.bas/... als Daten-Beispiele melden
    for fid, f in reg["features"].items():
        for pat in f.get("nobuild", []):
            pass
    # Hinweise aus STATUS.md
    try:
        with open(os.path.join(ROOT, "STATUS.md"), encoding="utf-8") as fh:
            st = parse_status(fh.read())
        for nid in reg.get("notes", []):
            if nid not in st:
                res.errors.append("Registry notes: %s steht nicht in STATUS.md" % nid)
    except OSError:
        res.errors.append("STATUS.md fehlt")


def run_check(args):
    reg = load_registry(args.registry)
    res = Result()
    rows, problems = read_selection(args.features, reg)
    res.errors += problems
    check_ids(rows, reg, res)
    check_filesystem(reg, res)
    eff, skipped, r2 = resolve(rows, reg, args.allow_unfinished, args.auto_deps)
    res.errors += r2.errors; res.warnings += r2.warnings; res.notes += r2.notes
    shipped, _ = data_files_to_ship(eff, reg)
    res.errors += sys_include_closure(shipped)
    return reg, rows, eff, skipped, res


def print_result(res, verbose=False):
    for e in res.errors:
        print("FEHLER:  " + e)
    for w in res.warnings:
        print("Warnung: " + w)
    if verbose:
        for n in res.notes:
            print("Hinweis: " + n)
    elif res.notes:
        k = sum(1 for n in res.notes if n.startswith("nicht beansprucht"))
        for n in res.notes:
            if not n.startswith("nicht beansprucht"):
                print("Hinweis: " + n)
        if k:
            print("Hinweis: %d Dateien unter data/ gehoeren keiner Zeile und gelten als immer dabei (-v zeigt sie)" % k)


def cmd_check(args):
    reg, rows, eff, skipped, res = run_check(args)
    print_result(res, args.verbose)
    n_sel = sum(1 for r in rows if r["checked"])
    print("%d Zeilen in FEATURES.md, %d angehakt, %d Fehler, %d Warnungen" % (len(rows), n_sel, len(res.errors), len(res.warnings)))
    return 1 if res.errors else 0


# ---------------------------------------------------------------- Plan

def human(n):
    for u in ("B", "KB", "MB", "GB"):
        if n < 1024 or u == "GB":
            return "%d %s" % (n, u) if u == "B" else "%.1f %s" % (n, u)
        n /= 1024.0


def fsize(rel):
    return os.path.getsize(os.path.join(ROOT, rel))


def example_files(reg, proj):
    ex = set(reg.get("examples_exclude_dirs", []))
    return walk("examples/" + proj, exclude_dirs=ex)


def build_plan(reg, rows, eff, skipped):
    data, unclaimed = data_files_to_ship(eff, reg)
    ex = examples_to_ship(eff, reg)
    shared = []
    for proj, pkg in ex:
        for pat in reg["features"][pkg].get("shared", []):
            rx = glob_rx(pat)
            for p in walk("examples/common"):
                if rx.match(p) and p not in shared:
                    shared.append(p)
    ex_files = []
    for proj, pkg in ex:
        ex_files += example_files(reg, proj)
    ex_files += shared
    return {"data": data, "examples": ex, "example_files": sorted(set(ex_files)), "unclaimed": unclaimed}


def cmd_plan(args):
    reg, rows, eff, skipped, res = run_check(args)
    print_result(res, False)
    if res.errors:
        print("Plan nicht moeglich: erst die Fehler beheben.")
        return 1
    plan = build_plan(reg, rows, eff, skipped)
    feats = reg["features"]
    print()
    print("Auswahl (%d Zeilen):" % len(eff))
    for sect in ("ide", "studio", "link", "target", "examples"):
        ids = [f for f in sorted(eff) if feats[f]["kind"] == sect]
        if ids:
            print("  %-9s %s" % (sect, ", ".join(ids)))
    off = [r["id"] for r in rows if r["id"] not in eff]
    print("  abgewaehlt: " + (", ".join(off) if off else "-"))
    print()
    print("Programme:   vcc, vc-lsp (immer)%s" % (", vintagecoder" if "IDE-APP" in eff else ""))
    dsize = sum(fsize(p) for p in plan["data"])
    esize = sum(fsize(p) for p in plan["example_files"])
    print("data/:       %d Dateien, %s (von %d)" % (len(plan["data"]), human(dsize), len(walk("data"))))
    print("examples/:   %d Projekte, %d Dateien, %s" % (len(plan["examples"]), len(plan["example_files"]), human(esize)))
    if args.verbose:
        for proj, pkg in plan["examples"]:
            print("  %-28s %s" % (proj, pkg))
    return 0


# ---------------------------------------------------------------- Verbotsscan

def crc32_of(path):
    c = 0
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            c = zlib.crc32(chunk, c)
    return "%08X" % (c & 0xFFFFFFFF)


def sha1_of(path):
    h = hashlib.sha1()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def ide_words(reg):
    """Verbotene Namen anderer Entwicklungsumgebungen. Die Registry speichert sie in Teilen (`forbidden_ide_words_parts`),
    damit die Namen selbst nirgends im Repository im Klartext stehen; hier werden sie zusammengesetzt."""
    return ["".join(parts) for parts in reg.get("forbidden_ide_words_parts", [])]


def technical_ide_token():
    """Technischer Name der Erweiterungsschnittstelle (Kleinschreibung): nur unter `ide_words_allow_technical` erlaubt."""
    return "vs" + "code"


def forbidden_scan(stage_dir, reg, share=""):
    """Gibt (Fehler, Warnungen) zurueck: ROM-Treffer, Emulator-Programmdateien, Namen anderer IDEs."""
    errs, warns = [], []
    roms = {}
    known = os.path.join(ROOT, "data", "roms", "known.json")
    if os.path.exists(known):
        for r in load_json(known).get("roms", []):
            roms.setdefault(r.get("size"), []).append((str(r.get("crc32", "")).upper(), str(r.get("sha1", "")).lower(), r.get("id")))
    emus = {n.lower() for n in reg.get("forbidden_emulator_files", [])}
    words = ide_words(reg)
    word_rx = re.compile("|".join(re.escape(w) for w in words)) if words else None
    allow = list(reg.get("ide_words_allow_technical", []))   # Ordner (an beliebiger Stelle im Staging, z. B. .../editors/ext/)
    # Technischer Name (Kleinschreibung, `engines`, `require`, Paketname): nur im Quelltext der Editor-Erweiterung erlaubt;
    # die Schreibweisen mit Leerzeichen/Großbuchstaben aus der Wortliste bleiben auch dort verboten.
    tech_rx = re.compile(re.escape(technical_ide_token()), re.I)
    for d, dirs, files in os.walk(stage_dir):
        for f in sorted(files):
            p = os.path.join(d, f)
            rel = os.path.relpath(p, stage_dir).replace(os.sep, "/")
            sub = rel[len(share) + 1:] if share and rel.startswith(share + "/") else rel
            base = re.sub(r"\.(exe|app|bin)$", "", f.lower())
            if base in emus:
                errs.append("Verbotsscan: %s ist eine Emulator-Programmdatei" % rel)
            sz = os.path.getsize(p)
            if sz in roms:
                c, s = crc32_of(p), sha1_of(p)
                for rc, rs, rid in roms[sz]:
                    if c == rc or s == rs:
                        errs.append("Verbotsscan: %s entspricht dem ROM %s" % (rel, rid))
            ext = os.path.splitext(f)[1].lower()
            if (word_rx or tech_rx) and ext in TEXT_EXT:
                with open(p, encoding="utf-8", errors="replace") as fh:
                    for n, line in enumerate(fh, 1):
                        m = word_rx.search(line) if word_rx else None
                        if not m and not any("/" + a.strip("/") + "/" in "/" + rel for a in allow):
                            m = tech_rx.search(line)
                        if m:
                            msg = "Verbotsscan: %s:%d nennt '%s'" % (rel, n, m.group(0))
                            (errs if sub.startswith(("data/", "examples/")) else warns).append(msg)
                            break
    return errs, warns


# ---------------------------------------------------------------- Staging

def stage_layout(os_name, out):
    """Pfade im Staging: bin (Programme), share (Ordner mit data/ und examples/), doc (Lizenzen, docs/, editors/)."""
    return {"bin": os.path.join(out, BIN_SUB[os_name]), "share": os.path.join(out, SHARE_SUB.get(os_name, "")),
            "doc": os.path.join(out, DOC_SUB.get(os_name, ""))}


def cargo_version():
    with open(os.path.join(ROOT, "Cargo.toml"), encoding="utf-8") as f:
        m = re.search(r'^version\s*=\s*"([^"]+)"', f.read(), re.M)
    return m.group(1) if m else "0.0.0"


def cargo_bin_dir():
    t = os.environ.get("CARGO_TARGET_DIR")
    if not t:
        out = subprocess.run(["cargo", "metadata", "--format-version", "1", "--no-deps"], cwd=ROOT, capture_output=True, text=True, check=True).stdout
        t = json.loads(out)["target_directory"]
    return os.path.join(t, "release")


def posix_join(*parts):
    return "/".join(p.strip("/") for p in parts if p)


def copy_file(rel, dest_root, root=ROOT, dest_rel=None):
    dst = os.path.join(dest_root, dest_rel or rel)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    shutil.copyfile(os.path.join(root, rel), dst)
    shutil.copymode(os.path.join(root, rel), dst)


def exe_name(name, os_name):
    return name + ".exe" if os_name == "windows" else name


def input_digest(rows, eff, reg_path, features_path):
    h = hashlib.sha256()
    h.update(json.dumps(sorted(eff)).encode())
    with open(reg_path, "rb") as f:
        h.update(f.read())
    return "sha256:" + h.hexdigest()


def release_json(reg, eff, skipped, digest, version, reproducible):
    f = reg["features"]
    def ids(kind):
        return [f[x]["id"] for x in sorted(eff) if f[x]["kind"] == kind]
    studios_order = [f[x]["id"] for x in f if f[x]["kind"] == "studio" and x in eff]
    d = {"format": 1}
    if not reproducible:
        d["generated"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    d.update({"input": digest, "version": version, "ide": "IDE-APP" in eff,
              "studios": studios_order, "links": [f[x]["id"] for x in f if f[x]["kind"] == "link" and x in eff],
              "targets": [f[x]["id"] for x in f if f[x]["kind"] == "target" and x in eff],
              "examples": [x for x in f if f[x]["kind"] == "examples" and x in eff]})
    return d


def write_report(path, reg, rows, eff, skipped, res, plan, bins, os_name, version, status):
    feats = reg["features"]
    L = ["# Release-Bericht", "", "Version laut Cargo.toml: %s · Betriebssystem: %s" % (version, os_name), ""]
    L += ["## Auswahl", "", "| Kennung | Zeile | Stand | Im Release |", "|---|---|---|---|"]
    for r in rows:
        mark = "ja" if r["id"] in eff else ("uebersprungen" if r["id"] in skipped else "nein")
        L.append("| `%s` | %s | %s | %s |" % (r["id"], r["label"].replace("|", "/"), r["stand"], mark))
    L += ["", "## Inhalt", "", "- Programme: %s" % ", ".join(bins),
          "- `data/`: %d Dateien, %s" % (len(plan["data"]), human(sum(fsize(p) for p in plan["data"]))),
          "- `examples/`: %d Projekte, %d Dateien" % (len(plan["examples"]), len(plan["example_files"])), ""]
    L += ["## Warnungen", ""] + (["- " + w for w in res.warnings] or ["keine"]) + [""]
    L += ["## Hinweise fuer die Release-Notizen", ""]
    below = [r for r in rows if r["id"] in eff and r["stand"] in STANDS and STANDS.index(r["stand"]) < BETA]
    betas = [r for r in rows if r["id"] in eff and r["stand"] == "Beta"]
    for r in below:
        L.append("- `%s`: Stand %s (%s)" % (r["id"], r["stand"], "Vorschau, bewusst freigegeben" if r["id"] in reg.get("preview_ok", []) else "nur mit --allow-unfinished ausgeliefert"))
    if betas:
        L.append("- Beta: " + ", ".join("`%s`" % r["id"] for r in betas))
    for nid in reg.get("notes", []):
        L.append("- `%s`: Stand %s (fester Bestandteil, siehe STATUS.md)" % (nid, status.get(nid, "?")))
    L += ["", "## Grenzen", "",
          "- Studios, Anbindungen und Hilfeseiten schalten sich noch nicht zur Laufzeit ab; die Auswahl wirkt ueber die ausgelieferten Dateien.",
          "- Geteilte Datendateien (`templates/*.json`, `profile`, `roms/known.json`, `palettes/index.json`) werden unveraendert ausgeliefert und nennen auch nicht gewaehlte Systeme.", ""]
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(L))


def cmd_stage(args):
    reg, rows, eff, skipped, res = run_check(args)
    print_result(res, False)
    if res.errors:
        print("Staging abgebrochen: erst die Fehler beheben.")
        return 1
    os_name = args.os
    out = os.path.abspath(args.out)
    if os.path.exists(out) and os.listdir(out):
        if not args.force:
            print("FEHLER:  %s ist nicht leer (--force loescht es zuerst)" % out)
            return 1
        shutil.rmtree(out)
    version = cargo_version()
    plan = build_plan(reg, rows, eff, skipped)
    ide = "IDE-APP" in eff

    if args.dry_run:
        print("--dry-run: es wird nichts geschrieben.")
        return 0

    # 1. Programme
    names = ["vcc", "vc-lsp"] + (["vintagecoder"] if ide else [])
    if args.build:
        pkgs = ["-p", "vc-cli", "-p", "vc-lsp"] + (["-p", "vc-app"] if ide else [])
        print("cargo build --release " + " ".join(pkgs))
        subprocess.run(["cargo", "build", "--release"] + pkgs, cwd=ROOT, check=True)
    bin_dir = args.bin_dir or (cargo_bin_dir() if args.build else os.path.join(ROOT, "target", "release"))
    lay = stage_layout(os_name, out)
    bin_sub = BIN_SUB[os_name]
    share_sub, doc_sub = SHARE_SUB.get(os_name, ""), DOC_SUB.get(os_name, "")
    bins = []
    os.makedirs(out, exist_ok=True)
    for n in names:
        src = os.path.join(bin_dir, exe_name(n, os_name))
        if not os.path.isfile(src):
            if args.no_bin:
                print("Hinweis: %s nicht gefunden (--no-bin: ohne Programme)" % src)
                continue
            shutil.rmtree(out, ignore_errors=True)
            print("FEHLER:  Programm %s nicht gefunden (--build oder --bin-dir angeben, oder --no-bin)" % src)
            return 1
        dst = os.path.join(out, bin_sub, exe_name(n, os_name))
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        shutil.copyfile(src, dst)
        os.chmod(dst, 0o755)
        bins.append(exe_name(n, os_name))

    # 2. data/ und Beispiele
    for p in plan["data"]:
        copy_file(p, out, dest_rel=posix_join(share_sub, p))
    for p in plan["example_files"]:
        copy_file(p, out, dest_rel=posix_join(share_sub, p))
    # 3. feste Dateien (Linux: unter usr/share/doc/vintagecoder)
    for rel in reg.get("root_files", []):
        copy_file(rel, out, dest_rel=posix_join(doc_sub, rel))
    for d in reg.get("root_dirs", []):
        for p in walk(d):
            copy_file(p, out, dest_rel=posix_join(doc_sub, p))
    if ide:
        for rel in reg.get("ide_files", []):
            copy_file(rel, out, dest_rel=posix_join(doc_sub, "licenses/" + os.path.basename(rel)))

    # 4. release.json
    digest = input_digest(rows, eff, args.registry or os.path.join(ROOT, "release", "registry.json"), args.features)
    rj = release_json(reg, eff, skipped, digest, version, args.reproducible)
    with open(os.path.join(lay["share"], "data", "release.json"), "w", encoding="utf-8") as f:
        json.dump(rj, f, indent=2, ensure_ascii=False); f.write("\n")

    # 5. Betriebssystemteil
    os_stage(os_name, out, ide, bool(plan["examples"]), version, args)

    # 6. Pruefungen am Staging
    ierrs = sys_include_closure(walk("data", root=lay["share"]), root=lay["share"])
    ferrs, fwarns = forbidden_scan(out, reg, share_sub)
    ex_errs = example_include_check(lay["share"], plan["examples"])
    errs = ierrs + ferrs + ex_errs
    for w in fwarns:
        print("Warnung: " + w)
        res.warnings.append(w)
    for e in errs:
        print("FEHLER:  " + e)
    if args.verify_build:
        verr = verify_examples(out, plan["examples"], reg, os_name)
        for e in verr:
            print("FEHLER:  " + e)
        errs += verr
    if errs:
        print("Staging fehlerhaft (%d Fehler): %s" % (len(errs), out))
        return 1

    with open(os.path.join(ROOT, "STATUS.md"), encoding="utf-8") as fh:
        status = parse_status(fh.read())
    write_report(os.path.join(out, "release-report.md"), reg, rows, eff, skipped, res, plan, bins or ["(keine)"], os_name, version, status)
    n = sum(len(fs) for _, _, fs in os.walk(out))
    print("Staging fertig: %s (%d Dateien)" % (out, n))
    if os_name == "linux" and args.archive:
        arc = linux_archive(out, version, args.reproducible)
        print("Archiv: " + arc)
    return 0


def example_include_check(out, examples):
    """Relative .include/#include "..." der ausgelieferten Beispiele muessen im Staging liegen."""
    errs = []
    for proj, _ in examples:
        for d, _, fs in os.walk(os.path.join(out, "examples", proj)):
            for f in fs:
                if os.path.splitext(f)[1].lower() not in (".asm", ".inc", ".bas"):
                    continue
                p = os.path.join(d, f)
                with open(p, encoding="utf-8", errors="replace") as fh:
                    text = fh.read()
                for m in INCLUDE_RX.finditer(text):
                    if m.group(1) == "<":
                        if not os.path.exists(os.path.join(out, "data", "sys", m.group(2))):
                            errs.append("%s: #include <%s> fehlt in data/sys des Staging" % (os.path.relpath(p, out), m.group(2)))
                        continue
                    if re.match(r"(?:\./)?build/", m.group(2)):
                        continue                      # vom Projektbau erzeugt (Medien-Export), liegt nie im Staging
                    tgt = os.path.normpath(os.path.join(d, m.group(2)))
                    if not os.path.exists(tgt) and not os.path.exists(os.path.join(out, "data", "sys", m.group(2))):
                        errs.append("%s: Include \"%s\" fehlt im Staging" % (os.path.relpath(p, out), m.group(2)))
    return errs


def verify_examples(out, examples, reg, os_name):
    """Baut jedes auszuliefernde, baubare Beispiel mit dem Staging-vcc (nur wenn es auf diesem Rechner laeuft)."""
    host = {"darwin": "macos", "win32": "windows"}.get(sys.platform, "linux")
    if host != os_name:
        print("Hinweis: --verify-build nur fuer das eigene Betriebssystem (%s), uebersprungen" % host)
        return []
    lay = stage_layout(os_name, out)
    vcc = os.path.join(lay["bin"], exe_name("vcc", os_name))
    if not os.path.exists(vcc):
        print("Hinweis: --verify-build ohne vcc im Staging uebersprungen")
        return []
    errs = []
    nobuild = Globs([p for f in reg["features"].values() for p in f.get("nobuild", [])])
    import tempfile
    for proj, _ in examples:
        if nobuild.match(proj):
            continue
        tmp = tempfile.mkdtemp(prefix="release-verify-")
        try:
            dst = os.path.join(tmp, "p")
            shutil.copytree(os.path.join(lay["share"], "examples", proj), dst)
            # Gemeinsame Dateien (../../common, ../prallball) relativ zum Projekt: Beispiele-Ordner nachbilden
            tree = os.path.join(tmp, "examples")
            shutil.copytree(os.path.join(lay["share"], "examples"), tree)
            r = subprocess.run([vcc, "build", os.path.join(tree, proj)], capture_output=True, text=True, timeout=300, stdin=subprocess.DEVNULL)
            if r.returncode != 0:
                errs.append("vcc build %s fehlgeschlagen: %s" % (proj, (r.stderr or r.stdout).strip().splitlines()[-1:] or ""))
            else:
                print("gebaut: " + proj)
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
    return errs


def os_stage(os_name, out, ide, has_examples, version, args):
    inst = os.path.join(ROOT, "installer")
    if os_name == "windows":
        if ide:
            copy_file("installer/windows/VintageCoder.ico", out, dest_rel="VintageCoder.ico")
        lines = ["; Erzeugt von tools/release.py stage --os windows: nicht von Hand aendern.",
                 '#define StageVersion "%s"' % version]
        if ide:
            lines.append("#define HasIde")
        if has_examples:
            lines.append("#define HasExamples")
        with open(os.path.join(out, "installer.inc"), "w", encoding="utf-8", newline="\n") as f:
            f.write("\n".join(lines) + "\n")
    elif os_name == "macos":
        if ide:
            copy_file("installer/macos/Info.plist.in", out, dest_rel="macos/Info.plist.in")
            for n in ("VintageCoder.icns",):
                copy_file("installer/macos/" + n, out, dest_rel="macos/" + n)
        copy_file("installer/macos/Kommandozeile installieren.command", out, dest_rel="macos/Kommandozeile installieren.command")
        os.chmod(os.path.join(out, "macos", "Kommandozeile installieren.command"), 0o755)
    elif os_name == "linux":
        # FHS-Baum (Konzept 8.4): Manpages, bei IDE Desktop-Datei und Symbol; Programme und Daten legt cmd_stage ab
        man = [f for f in sorted(os.listdir(os.path.join(inst, "linux", "man"))) if f.endswith(".1")]
        for f in man:
            if f == "vintagecoder.1" and not ide:
                continue
            copy_file("installer/linux/man/" + f, out, dest_rel="usr/share/man/man1/" + f)
        if ide:
            copy_file("installer/linux/vintagecoder.desktop", out, dest_rel="usr/share/applications/vintagecoder.desktop")
            png_halve(os.path.join(ROOT, "installer/macos/VintageCoder-1024.png"),
                      os.path.join(out, "usr/share/icons/hicolor/512x512/apps/vintagecoder.png"))


def png_halve(src, dst):
    """Verkleinert ein 8-Bit-RGBA-PNG (nicht verschachtelt) auf die Haelfte (Mittelwert aus 2x2). Nur Standardbibliothek.
    hicolor kennt 1024x1024 nicht; 512x512 ist die groesste Standardgroesse."""
    with open(src, "rb") as f:
        raw = f.read()
    pos, idat, w, h = 8, b"", 0, 0
    while pos < len(raw):
        n, typ = struct.unpack(">I4s", raw[pos:pos + 8])
        body = raw[pos + 8:pos + 8 + n]
        if typ == b"IHDR":
            w, h, depth, ctype, _, _, inter = struct.unpack(">IIBBBBB", body)
            if (depth, ctype, inter) != (8, 6, 0):
                raise ReleaseError("%s: nur 8-Bit-RGBA ohne Verschachtelung unterstuetzt" % src)
        elif typ == b"IDAT":
            idat += body
        pos += 12 + n
    data, bpp, stride = zlib.decompress(idat), 4, w * 4
    rows, prev = [], bytearray(stride)
    for y in range(h):
        ft, line = data[y * (stride + 1)], bytearray(data[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for i in range(stride):
            a = line[i - bpp] if i >= bpp else 0
            b = prev[i]
            c = prev[i - bpp] if i >= bpp else 0
            if ft == 1: line[i] = (line[i] + a) & 255
            elif ft == 2: line[i] = (line[i] + b) & 255
            elif ft == 3: line[i] = (line[i] + (a + b) // 2) & 255
            elif ft == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                line[i] = (line[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(line); prev = line
    out = bytearray()
    for y in range(0, h - 1, 2):
        out.append(0)
        for x in range(0, w - 1, 2):
            for k in range(4):
                out.append((rows[y][x * 4 + k] + rows[y][x * 4 + 4 + k] + rows[y + 1][x * 4 + k] + rows[y + 1][x * 4 + 4 + k] + 2) // 4)
    def chunk(t, b):
        return struct.pack(">I", len(b)) + t + b + struct.pack(">I", zlib.crc32(t + b) & 0xffffffff)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    with open(dst, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w // 2, h // 2, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(bytes(out), 9)) + chunk(b"IEND", b""))


def linux_archive(out, version, reproducible):
    """tar.gz neben dem Staging (<out>.tar.gz) mit Oberordner vintagecoder-<version>-linux-<arch>; gleiche Funktion wie `package --format tar`."""
    import linux_pkg
    machine = os.uname().machine if hasattr(os, "uname") else "x86_64"
    path = out.rstrip("/") + ".tar.gz"
    linux_pkg.make_tar(out, path, version, machine, linux_pkg.REPRODUCIBLE_EPOCH if reproducible else None)
    return path


def cmd_package(args):
    """Linux-Pakete aus einem FHS-Staging (Konzept 8.4). Version aus Cargo.toml, nie aus dem Skript."""
    import linux_pkg
    cfg = linux_pkg.load_config(args.packages)
    distro = args.distro
    if not distro and args.format in ("deb", "rpm", "auto"):
        try:
            with open("/etc/os-release", encoding="utf-8") as f:
                distro = linux_pkg.detect_distro(f.read())
        except OSError:
            distro = None
        if not distro:
            raise ReleaseError("--distro angeben (%s); /etc/os-release nicht auswertbar" % ", ".join(linux_pkg.DISTROS))
        print("Distribution erkannt: " + distro)
    version = cargo_version()
    try:
        res = linux_pkg.package(args.stage, args.out, args.format, distro, version, arch=args.arch, cfg=cfg, reproducible=args.reproducible)
    except linux_pkg.PackageError as e:
        raise ReleaseError(str(e))
    for name, kind in res:
        print("Paket (%s): %s" % (kind, os.path.join(os.path.abspath(args.out), name)))
    return 0


# ---------------------------------------------------------------- bootstrap

def cmd_bootstrap(args):
    """Entwurf von registry.json aus dem Bestand (data/targets, data/args, examples/*/*/*.vcprj). Einmalig, danach von Hand."""
    out = args.out
    if out and os.path.exists(out) and not args.force:
        print("FEHLER:  %s existiert (--force ueberschreibt)" % out)
        return 1
    sysdir = {"atari8": "atari8bit"}
    exdir = {"atari8": "atari"}
    feats = {"IDE-APP": {"kind": "ide", "id": "ide", "needs": [], "files": []}}
    for p in sorted(walk("data/targets")):
        t = load_json(os.path.join(ROOT, p))["target"]
        tid = t["id"]
        files = ["data/targets/%s.target.json" % tid]
        if os.path.isdir(os.path.join(ROOT, "data/sys", sysdir.get(tid, tid))):
            files.append("data/sys/%s/**" % sysdir.get(tid, tid))
        if t.get("chips"):
            files.append("data/chips/{%s}.chip.json" % ",".join(t["chips"]))
        files.append("data/cpus/%s.cpu.json" % t["cpu"])
        kid = "SYS-" + {"atari8": "ATARI", "vic20": "VC20"}.get(tid, tid.upper())
        feats[kid] = {"kind": "target", "id": tid, "example_dir": exdir.get(tid, tid), "files": files}
    for p in sorted(walk("data/args")):
        n = os.path.basename(p)[:-5]
        if n == "vcc-asm":
            continue
        key = "EMU-VICE" if n.startswith("vice-") else "EMU-" + n.upper()
        f = feats.setdefault(key, {"kind": "link", "id": "vice" if n.startswith("vice-") else n, "needs": [], "files": []})
        f["files"].append("data/args/vice-*.json" if n.startswith("vice-") else p)
        f["files"] = sorted(set(f["files"]))
    pk = {"EX-HELLO": [], "EX-JUMPRUN": [], "EX-PRALLBALL": [], "EX-DEMOS": []}
    for proj in example_projects({"examples_ignore": []}):
        name = proj.split("/")[1]
        key = "EX-JUMPRUN" if name == "jumprun" else "EX-PRALLBALL" if name.startswith("prallball") else \
              "EX-HELLO" if re.search(r"hello|basic_native|border", name) else "EX-DEMOS"
        pk[key].append(proj)
    for k, v in pk.items():
        feats[k] = {"kind": "examples", "projects": v}
    reg = {"format": 1, "features": feats, "core": ["data/cli/**", "data/sys/common/**", "data/args/vcc-asm.json"], "never": ["data/sys/vcs2600/**"]}
    text = json.dumps(reg, indent=2, ensure_ascii=False) + "\n"
    if out:
        with open(out, "w", encoding="utf-8") as f:
            f.write(text)
        print("Entwurf geschrieben: " + out)
    else:
        sys.stdout.write(text)
    return 0


# ---------------------------------------------------------------- explain

def cmd_explain(args):
    reg, rows, eff, skipped, res = run_check(args)
    fid = args.explain
    feats = reg["features"]
    if fid not in feats:
        print("FEHLER:  unbekannte Kennung %s" % fid)
        return 1
    f = feats[fid]
    print("%s (%s): %s" % (fid, f["kind"], next((r["label"] for r in rows if r["id"] == fid), "")))
    print("  im Release: %s" % ("ja" if fid in eff else "nein"))
    print("  braucht:    %s" % (", ".join(f.get("needs", [])) or "-"))
    if f.get("needs_any"):
        print("  braucht eines von: %s" % ", ".join(f["needs_any"]))
    users = [k for k, v in feats.items() if fid in v.get("needs", []) or fid in v.get("needs_any", [])]
    print("  gebraucht von: %s" % (", ".join(users) or "-"))
    if f.get("files"):
        print("  Dateien:    " + ", ".join(f["files"]))
    return 0


# ---------------------------------------------------------------- CLI

def build_parser():
    ap = argparse.ArgumentParser(
        prog="release.py",
        description="""Release-Auswahl: liest die Haekchen aus FEATURES.md und die Technik aus release/registry.json und erzeugt ein Staging-Verzeichnis,
das die Installer (Windows: Inno Setup, macOS: build-macos.sh, Linux: Archiv) nur noch einpacken.

Unterbefehle:
  check      Auswahl, Abhaengigkeiten und Dateien pruefen (Exit-Code 1 bei Fehlern)
  plan       zeigen, was ausgeliefert wuerde (Zeilen, Dateien, Groessen); schreibt nichts
  stage      Staging-Verzeichnis fuer ein Betriebssystem erzeugen (data/release.json, Beispiele, Programme, release-report.md)
  package    Linux-Pakete (.deb, .rpm, tar.gz) aus dem Linux-Staging bauen (dpkg-deb, rpmbuild; Konzept 8.4)
  bootstrap  Entwurf von release/registry.json aus dem Bestand (einmalig; danach von Hand pflegen)

Die Kommandozeile vcc und der Sprachserver vc-lsp sind immer dabei. Bei verletzter Abhaengigkeit bricht das Skript ab und nennt, was
anzukreuzen ist; die Auswahl wird nie stillschweigend geaendert (Ausnahme: --auto-deps). Die Versionsnummer wird nur aus Cargo.toml gelesen.
Der Arbeitsordner ist egal (alle Pfade gelten relativ zum Repository, ausser --out und --bin-dir).""",
        epilog="""Beispiele:
  python3 tools/release.py check
  python3 tools/release.py plan -v
  python3 tools/release.py check --explain STU-TRACKER
  python3 tools/release.py stage --os macos --out build/stage-macos --build
  python3 tools/release.py stage --os windows --out build/stage-windows --bin-dir target/release --force
  python3 tools/release.py stage --os linux --out build/stage-linux --build --archive --reproducible
  python3 tools/release.py check --features tools/testdata/nur-cli-c64.md""",
        formatter_class=argparse.RawDescriptionHelpFormatter)
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("--features", default=os.path.join(ROOT, "FEATURES.md"), metavar="DATEI", help="Auswahldatei (Standard: FEATURES.md im Repository)")
    common.add_argument("--registry", default=None, metavar="DATEI", help="Registry (Standard: release/registry.json)")
    common.add_argument("--allow-unfinished", action="store_true", help="gewaehlte Zeilen mit Stand unter Beta zulassen (der Bericht markiert sie)")
    common.add_argument("--auto-deps", action="store_true", help="fehlende harte Abhaengigkeiten ergaenzen und melden, statt abzubrechen (nur fuer Experimente)")
    common.add_argument("--explain", metavar="KENNUNG", help="nur erklaeren, wer die Zeile braucht und was daran haengt (z. B. STU-TILES)")
    common.add_argument("-v", "--verbose", action="store_true", help="auch Hinweise ausgeben (z. B. nicht beanspruchte Dateien, Beispielliste)")
    sub = ap.add_subparsers(dest="cmd", metavar="UNTERBEFEHL")
    sub.add_parser("check", parents=[common], help="Auswahl und Registry pruefen", formatter_class=argparse.RawDescriptionHelpFormatter,
                   description="Prueft FEATURES.md gegen release/registry.json und das Dateisystem: Kennungen, Staende, Abhaengigkeiten, Dateizuordnung, Beispiele.")
    sub.add_parser("plan", parents=[common], help="zeigen, was ausgeliefert wuerde",
                   description="Wertet die Auswahl aus und nennt Zeilen, Dateizahlen und Groessen. Schreibt keine Dateien.")
    st = sub.add_parser("stage", parents=[common], help="Staging-Verzeichnis erzeugen", formatter_class=argparse.RawDescriptionHelpFormatter,
                        description="""Erzeugt das Staging fuer ein Betriebssystem. Inhalt: Programme (vcc, vc-lsp, bei IDE-APP vintagecoder), data/ nach Registry,
data/release.json, Beispiele, Lizenzen, Dokumentation, Betriebssystemteil (Windows: installer.inc; macOS: macos/; Linux: FHS-Baum usr/bin, usr/share/vintagecoder, usr/share/man, bei IDE .desktop und Symbol)
und release-report.md. Danach laufen Verbotsscan (keine ROMs, keine Emulator-Programme, keine Namen anderer IDEs in data/ und examples/),
Include-Pruefung der Systemdateien und Beispiele.""")
    st.add_argument("--os", required=True, choices=["windows", "macos", "linux"], help="Zielbetriebssystem (aendert nur die Verpackung, nicht die Auswahl)")
    st.add_argument("--out", required=True, metavar="ORDNER", help="Ausgabeordner des Staging (muss leer sein oder fehlen)")
    st.add_argument("--build", action="store_true", help="vorher cargo build --release (vc-cli, vc-lsp, bei IDE-APP vc-app); ein Bau je Betriebssystem")
    st.add_argument("--bin-dir", metavar="ORDNER", help="Ordner mit den gebauten Programmen (Standard: target/release; mit --build der Zielordner von Cargo)")
    st.add_argument("--no-bin", action="store_true", help="fehlende Programme nicht als Fehler behandeln (nur Daten stagen, z. B. fuer Tests)")
    st.add_argument("--force", action="store_true", help="vorhandenen Ausgabeordner zuerst loeschen")
    st.add_argument("--dry-run", action="store_true", help="nur pruefen und planen, nichts schreiben")
    st.add_argument("--reproducible", action="store_true", help="keinen Zeitstempel schreiben, Archiv mit festen Zeiten (zwei Laeufe ergeben gleiche Pruefsummen)")
    st.add_argument("--archive", action="store_true", help="Linux: zusaetzlich <out>.tar.gz erzeugen")
    st.add_argument("--verify-build", action="store_true", help="jedes ausgelieferte baubare Beispiel mit dem Staging-vcc bauen (nur auf dem eigenen Betriebssystem)")
    pk = sub.add_parser("package", help="Linux-Pakete (.deb, .rpm, tar.gz) aus dem Staging bauen", formatter_class=argparse.RawDescriptionHelpFormatter,
                        description="""Baut aus einem Staging von `stage --os linux` (FHS-Baum) die Pakete: vintagecoder-cli (vcc, vc-lsp, Daten) und, wenn die IDE gewaehlt ist,
vintagecoder (IDE, haengt vom CLI-Paket in gleicher Version ab). Werkzeuge: dpkg-deb und rpmbuild (im Bau-Container, tools/docker/). Metadaten aus der Auswahl
(data/release.json im Staging) und release/linux-packages.json; Version aus Cargo.toml (SemVer-Vorabversionen: deb 0.1.0~alpha, rpm Release 0.1.alpha).
Neben jedes Paket kommt <datei>.files (erwartete Dateiliste fuer die Pruefung im Test-Container) und SHA256SUMS.""",
                        epilog="""Beispiele:
  python3 tools/release.py package --os linux --format deb --distro debian --stage build/stage-linux --out build/pakete
  python3 tools/release.py package --os linux --format auto --distro fedora --stage build/stage-linux --out build/pakete
  python3 tools/release.py package --os linux --format tar --stage build/stage-linux --out build/pakete""")
    pk.add_argument("--os", required=True, choices=["linux"], help="Betriebssystem (nur linux)")
    pk.add_argument("--format", required=True, choices=["deb", "rpm", "tar", "auto"], help="Paketformat; auto = Format der Distribution plus tar")
    pk.add_argument("--distro", choices=["debian", "ubuntu", "fedora", "opensuse"], help="Zieldistribution (Standard: aus /etc/os-release)")
    pk.add_argument("--stage", required=True, metavar="ORDNER", help="Staging von `stage --os linux`")
    pk.add_argument("--out", required=True, metavar="ORDNER", help="Ausgabeordner der Pakete")
    pk.add_argument("--arch", metavar="ARCH", help="Architektur der Pakete (Standard: dieser Rechner, z. B. amd64/x86_64)")
    pk.add_argument("--packages", metavar="DATEI", help="Paketmetadaten (Standard: release/linux-packages.json)")
    pk.add_argument("--reproducible", action="store_true", help="feste Zeitstempel (SOURCE_DATE_EPOCH), gleiche Eingabe ergibt gleiche Pruefsummen (soweit dpkg/rpm es erlauben)")
    bs = sub.add_parser("bootstrap", help="Entwurf von registry.json erzeugen",
                        description="Liest data/targets, data/args und examples/*/*/*.vcprj und schreibt einen Entwurf (stdout oder --out). Einmalig; die Registry wird danach von Hand gepflegt.")
    bs.add_argument("--out", metavar="DATEI", help="Datei statt stdout")
    bs.add_argument("--force", action="store_true", help="vorhandene Datei ueberschreiben")
    return ap


def main(argv=None):
    ap = build_parser()
    args = ap.parse_args(argv)
    if not args.cmd:
        ap.print_help()
        return 2
    try:
        if args.cmd == "bootstrap":
            return cmd_bootstrap(args)
        if getattr(args, "explain", None):
            return cmd_explain(args)
        return {"check": cmd_check, "plan": cmd_plan, "stage": cmd_stage, "package": cmd_package}[args.cmd](args)
    except ReleaseError as e:
        print("FEHLER:  %s" % e)
        return 1


if __name__ == "__main__":
    sys.exit(main())
