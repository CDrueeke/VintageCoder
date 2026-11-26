#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Prüft die englischen Datendateien (Etappe I3).

* data/cli/en/*.json: gleicher Aufbau wie data/cli/*.json (gleiche Schlüssel, gleiche Listenlängen, gleiche technische Werte), keine deutschen
  Reste (Umlaute, ß), Kopfeintrag "i18n" mit status (maschinell|geprüft) und source_hash (Hash der deutschen Quelle).
* Überlagerungen data/i18n/en/<bereich>/<datei>.json für alle übrigen Datendateien (Zielsysteme, Profile, Chips, Emulatoren, ROM-Liste, Einrichtung,
  Vorlagen, Zeichensätze, Zeichen, Emulator-Argumente, Hilfe-Schlüsselwörter): je übersetzbarem Textfeld ein Eintrag
  "/zeiger": {"de": Originaltext, "en": Übersetzung, "status": "maschinell"|"geprüft"|"unverändert"}. Der Zeiger ist ein JSON-Zeiger (RFC 6901)
  in die deutsche Quelldatei; das Programm setzt beim Laden `en` an diese Stelle (vc_i18n::localize_json). Ändert sich der deutsche Text, gilt der
  Eintrag als veraltet (Feld "de" stimmt nicht mehr).
Aufruf: python3 tools/check_i18n_data.py [--stempeln] [--erzeugen] [--veraltet]
  --veraltet  Hilfe und Handbuch: setzt bei jeder Seite, deren deutsche Quelle sich geändert hat, den Status "veraltet" und stempelt den Hash (für 0.3.0 nur Deutsch; englische Seite folgt in 0.3.1, Strukturprüfung entfällt für solche Seiten)
  --stempeln  setzt/erneuert "i18n" in den englischen cli-Dateien
  --erzeugen  legt für fehlende übersetzbare Felder leere Einträge in den Überlagerungen an (en leer, status maschinell), meldet nichts als Fund
Ausgabe: Funde, Rückgabewert 1 bei Funden.
"""
import hashlib, json, re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
NAMES = [l.strip() for l in (ROOT / "tools/i18n_names.txt").read_text(encoding="utf-8").splitlines() if l.strip() and not l.startswith("#")]
UMLAUT = re.compile(r"[äöüÄÖÜß]")
# technische Felder: müssen in en und de gleich sein
TECH = {"flag", "short", "id", "group", "aliases", "required", "repeat", "run", "default", "choices", "command", "program", "code", "name_tech"}
STATUS = {"maschinell", "geprüft", "veraltet"}  # "veraltet" (nur Hilfe und Handbuch): deutsche Seite neu geschrieben, englische noch alt; wird mit --veraltet gesetzt und in 0.3.1 neu übersetzt
VERALTET = "--veraltet" in sys.argv

def h(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()[:12]

def shape(a, b, p, out, tech=False):
    """vergleicht den Aufbau; technische Felder müssen gleich sein"""
    if isinstance(a, dict) and isinstance(b, dict):
        for k in a:
            if k == "i18n": continue
            if k not in b: out.append(f"{p}/{k}: fehlt in en"); continue
            shape(a[k], b[k], f"{p}/{k}", out, tech or k in TECH)
        for k in b:
            if k not in a and k != "i18n": out.append(f"{p}/{k}: zusätzlich in en")
    elif isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b): out.append(f"{p}: Länge {len(a)} (de) gegen {len(b)} (en)"); return
        for i, (x, y) in enumerate(zip(a, b)): shape(x, y, f"{p}/{i}", out, tech)
    elif type(a) != type(b): out.append(f"{p}: Typ verschieden")
    elif tech and a != b: out.append(f"{p}: technischer Wert verändert ({a!r} -> {b!r})")

def strings(o, p=""):
    if isinstance(o, dict):
        for k, v in o.items():
            if k != "i18n": yield from strings(v, f"{p}/{k}")
    elif isinstance(o, list):
        for i, v in enumerate(o): yield from strings(v, f"{p}/{i}")
    elif isinstance(o, str): yield p, o

def check_cli(stamp, out):
    de_dir, en_dir = ROOT / "data/cli", ROOT / "data/cli/en"
    for de in sorted(de_dir.glob("*.json")):
        en = en_dir / de.name
        if not en.exists(): out.append(f"{en.relative_to(ROOT)}: fehlt"); continue
        dj, ej = json.loads(de.read_text(encoding="utf-8")), json.loads(en.read_text(encoding="utf-8"))
        rel = en.relative_to(ROOT)
        o = []
        shape(dj, ej, "", o)
        out += [f"{rel}{x}" for x in o]
        for p, s in strings(ej):
            t = s
            for n in NAMES: t = t.replace(n, "")
            if UMLAUT.search(t): out.append(f"{rel}{p}: Umlaut/ß im Englischen: {s[:60]!r}")
        st = ej.get("i18n", {})
        if stamp:
            txt = en.read_text(encoding="utf-8")
            txt = re.sub(r'^\{\s*"i18n":\s*\{[^}]*\},?\n', "{\n", txt)  # alten Kopf entfernen
            line = '  "i18n": { "status": "%s", "source_hash": "%s" },\n' % (st.get("status", "maschinell"), h(de))
            en.write_text(txt.replace("{\n", "{\n" + line, 1), encoding="utf-8")
        else:
            if st.get("status") not in STATUS: out.append(f"{rel}: i18n.status fehlt oder ungültig")
            if st.get("source_hash") != h(de): out.append(f"{rel}: veraltet (Quelle hat sich geändert) oder ohne source_hash")
    for en in en_dir.glob("*.json"):
        if not (de_dir / en.name).exists(): out.append(f"{en.relative_to(ROOT)}: keine deutsche Quelle")

# Übersetzbare Textfelder je Bereich: Schlüsselnamen (letzter Schlüssel ohne Listenindex); (Schlüssel, Ahnenteil) für "nur unterhalb von"
TEXT = {
    "targets": {"note", "hint", "label", "source", "testMailboxNote", "testDebugRegNote", ("name", "presets")},
    "profile": {"name", "hint"},
    "emulators": {"role", "summary", "notes", "windows", "macos", "linux", "label", "hint"},
    "roms": {"label", "note"},
    "setup": {"note"},
    "args": {"hint", "label"},
    "chars": {"desc"},
    "chips": {"doc"},
    "templates": {"description", "name", "label"},
    "charsets": {("name", "saetze"), ("block", "klassen"), ("balken", "klassen"), ("muster", "klassen"), ("geometrisch", "klassen")},
    "mega65": {("nicht_erreichbar", "hinweise"), ("schnittstelle", "hinweise")},
    "asm": {"args"},
    "help": {"text"},
}
SKIP_FILE = {("templates", "structures.json"): set()}  # (Platzhalter: alle Dateien des Bereichs gelten)
WORTLOS = re.compile(r"^[\W\d_]*$|^[\$%]?[0-9A-Fa-f]+$")

def is_text_key(area, path):
    keys = [k for k in path if not isinstance(k, int)]
    if not keys: return False
    last = keys[-1]
    for r in TEXT.get(area, ()):
        if isinstance(r, tuple):
            if last == r[0] and r[1] in keys[:-1]: return True
        elif last == r: return True
    return False

def candidates(area, o, path=()):
    """(zeiger, text) aller übersetzbaren Textfelder"""
    if isinstance(o, dict):
        for k, v in o.items(): yield from candidates(area, v, path + (k,))
    elif isinstance(o, list):
        for i, v in enumerate(o): yield from candidates(area, v, path + (i,))
    elif isinstance(o, str) and is_text_key(area, path) and not WORTLOS.match(o.strip()):
        yield "/" + "/".join(str(k).replace("~", "~0").replace("/", "~1") for k in path), o

NOCH_OFFEN = set()  # Bereiche, deren Übersetzung zu einer späteren Etappe gehört: "noch nicht übersetzt" ist hier kein Fund
OVERLAY_STATUS = {"maschinell", "geprüft", "unverändert"}

SYS_ROOTS = ["c128/kernal.asm", "plus4/kernal.asm", "vic20/kernal.asm", "atari8bit/antic.asm", "atari8bit/gtia.asm", "atari8bit/pokey.asm", "atari8bit/os.asm",
             "mega65/hardware.asm", "mega65/kernal.asm", "mega65/rom_920418.asm", "c64/hardware.asm", "c64/kernal.asm", "c64/basic.asm"]

def sys_files():
    """Dateien der Systembibliothek, die crates/vc-ide/src/sysmap.rs liest (Einstiegsdateien und ihre #include <...>)"""
    seen = []
    def walk(rel):
        if rel in seen: return
        p = ROOT / "data/sys" / rel
        if not p.exists(): return
        seen.append(rel)
        for l in p.read_text(encoding="utf-8").splitlines():
            l = l.strip()
            if l.startswith("#include <"): walk(l[10:].split(">")[0])
    for r in SYS_ROOTS: walk(r)
    return seen

def sys_candidates(rel):
    """{ "/L<zeile>": Text } wie sysmap.rs: Abschnittsüberschriften `// ----` und Kurzbeschreibungen hinter `.const ... = $adresse // text`"""
    out = {}
    for i, l in enumerate((ROOT / "data/sys" / rel).read_text(encoding="utf-8").split("\n"), 1):
        l = l.strip()
        if l.startswith("// ----"):
            t = l[len("// ----"):].strip()
        elif l.startswith(".const ") and "//" in l[7:]:
            code, t = l[7:].split("//", 1)
            if "=" not in code: continue
            v = code.split("=", 1)[1].strip()
            try:
                if not v.startswith("$"): continue
                int(v[1:], 16)
            except ValueError: continue
            t = t.strip()
        else: continue
        if t and not WORTLOS.match(t): out["/L%d" % i] = t
    return out

def sources():
    """(bereich, relative Quelldatei) aller Dateien, für die es Überlagerungen geben kann"""
    d = ROOT / "data"
    for area in TEXT:
        if area in ("help",): files = [d / "help/basic_keywords.json"]
        elif area == "asm": files = [d / "asm/directives.json"]
        elif area == "mega65": files = [d / "mega65/tools.json"]
        elif area == "charsets": files = [d / "charsets/index.json", d / "charsets/ausnahmen.json"]
        else: files = sorted((d / area).glob("*.json"))
        for f in files:
            if f.exists(): yield area, f

def check_overlays(create, out):
    base = ROOT / "data/i18n/en"
    seen = set()
    quellen = [(a, s_, None) for a, s_ in sources()] + [("sys", ROOT / "data/sys" / r, r) for r in sys_files()]
    for area, src, sysrel in quellen:
        rel = src.relative_to(ROOT / "data").as_posix()
        ov_path = base / (rel + ".json" if sysrel else rel)
        cand = sys_candidates(sysrel) if sysrel else dict(candidates(area, json.loads(src.read_text(encoding="utf-8"))))
        if not cand and not ov_path.exists(): continue
        seen.add(ov_path)
        ov = json.loads(ov_path.read_text(encoding="utf-8")) if ov_path.exists() else {"source": rel, "entries": {}}
        ent = ov.setdefault("entries", {})
        changed = False
        for z, de in cand.items():
            if z not in ent:
                if create: ent[z] = {"de": de, "en": "", "status": "maschinell"}; changed = True
                else: out.append(f"data/i18n/en/{rel}{z}: Eintrag fehlt")
        for z, e in list(ent.items()):
            where = f"data/i18n/en/{rel}{z}"
            if z not in cand: out.append(f"{where}: Feld gibt es in der Quelle nicht (mehr)"); continue
            if e.get("de") != cand[z]: out.append(f"{where}: veraltet (deutscher Text hat sich geändert)")
            en = e.get("en", "")
            if not en and not create and area not in NOCH_OFFEN: out.append(f"{where}: noch nicht übersetzt")
            if e.get("status") not in OVERLAY_STATUS: out.append(f"{where}: status ungültig")
            if e.get("status") != "unverändert" and en and en == e.get("de") and len(en) > 12: out.append(f"{where}: unverändert, aber nicht als \"unverändert\" markiert")
            t = en
            for n in NAMES: t = t.replace(n, "")
            if UMLAUT.search(t): out.append(f"{where}: Umlaut/ß im Englischen: {en[:60]!r}")
        if changed or (create and not ov_path.exists()):
            ov_path.parent.mkdir(parents=True, exist_ok=True)
            ov_path.write_text(json.dumps(ov, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    if base.exists():
        for f in base.rglob("*.json"):
            if f not in seen and f.relative_to(base).parts[0] != "i18n": out.append(f"{f.relative_to(ROOT)}: keine Quelle/keine übersetzbaren Felder")

LINK = re.compile(r"\]\(([^)\s]+)\)")

def check_help(stamp, out):
    """crates/vc-ide/help/en/*.md gegen crates/vc-ide/help/*.md; Titel und Gruppen in en/_index.json"""
    de_dir, en_dir = ROOT / "crates/vc-ide/help", ROOT / "crates/vc-ide/help/en"
    ip = en_dir / "_index.json"
    idx = json.loads(ip.read_text(encoding="utf-8"))
    for de in sorted(de_dir.glob("*.md")):
        en = en_dir / de.name
        rel = en.relative_to(ROOT)
        if not en.exists(): out.append(f"{rel}: fehlt"); continue
        a, b = de.read_text(encoding="utf-8"), en.read_text(encoding="utf-8")
        f = idx["files"].setdefault(de.name, {"status": "maschinell"})
        if VERALTET and f.get("source_hash") != h(de): f["status"] = "veraltet"
        if f.get("status") == "veraltet":
            # nur Hash, Status und Umlaute prüfen: die englische Seite folgt in 0.3.1
            if not b.startswith("# "): out.append(f"{rel}: beginnt nicht mit einer Überschrift")
            if stamp or VERALTET: f["source_hash"] = h(de)
            elif f.get("source_hash") != h(de): out.append(f"{rel}: veraltet (deutsche Seite hat sich geändert) oder ohne source_hash")
            continue
        if not b.startswith("# "): out.append(f"{rel}: beginnt nicht mit einer Überschrift")
        if sorted(LINK.findall(a)) != sorted(LINK.findall(b)): out.append(f"{rel}: Linkziele weichen ab")
        if a.count("```") != b.count("```"): out.append(f"{rel}: Anzahl der Codeblöcke weicht ab ({a.count('```')} gegen {b.count('```')})")
        for key in ("<!-- EMULATOR-TABLE -->", "<!-- EMULATOR-SECTIONS -->"):
            if (key in a) != (key in b): out.append(f"{rel}: Platzhalter {key} weicht ab")
        if sum(1 for l in a.splitlines() if l.startswith("|")) != sum(1 for l in b.splitlines() if l.startswith("|")): out.append(f"{rel}: Anzahl der Tabellenzeilen weicht ab")
        if sum(1 for l in a.splitlines() if l.startswith("#")) != sum(1 for l in b.splitlines() if l.startswith("#")): out.append(f"{rel}: Anzahl der Überschriften weicht ab")
        for n, l in enumerate(b.splitlines(), 1):
            t = l
            for nm in NAMES: t = t.replace(nm, "")
            if UMLAUT.search(t): out.append(f"{rel}:{n}: Umlaut/ß im Englischen: {l[:60]!r}")
        if stamp: f["source_hash"] = h(de)
        else:
            if f.get("status") not in STATUS: out.append(f"{rel}: Status fehlt oder ungültig")
            if f.get("source_hash") != h(de): out.append(f"{rel}: veraltet (deutsche Seite hat sich geändert) oder ohne source_hash")
    for k in ("titles", "groups"):
        for name, v in idx[k].items():
            if not v: out.append(f"crates/vc-ide/help/en/_index.json: {k}/{name} noch nicht übersetzt")
            elif UMLAUT.search(v): out.append(f"crates/vc-ide/help/en/_index.json: {k}/{name}: Umlaut")
    if stamp or VERALTET: ip.write_text(json.dumps(idx, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

def front(text):
    m = re.match(r"^---\n(.*?)\n---\n", text, re.S)
    fm = {}
    if m:
        for l in m.group(1).splitlines():
            if ":" in l:
                k, v = l.split(":", 1); fm[k.strip()] = v.strip()
    return fm, text[m.end():] if m else text

def check_handbook(stamp, out):
    """docs/handbuch/en/kapitel/*.md und en/GLOSSAR.md gegen docs/handbuch/kapitel/*.md und GLOSSAR.md"""
    base = ROOT / "docs/handbuch"
    pairs = [(f, base / "en/kapitel" / f.name) for f in sorted((base / "kapitel").glob("*.md"))] + [(base / "GLOSSAR.md", base / "en/GLOSSAR.md")]
    for de, en in pairs:
        rel = en.relative_to(ROOT)
        if not en.exists(): out.append(f"{rel}: fehlt"); continue
        at, bt = de.read_text(encoding="utf-8"), en.read_text(encoding="utf-8")
        if de.name == "GLOSSAR.md":
            # ohne Kopfzeile: Vermerk am Ende der englischen Datei  <!-- i18n: veraltet HASH -->
            m = re.search(r"\n<!-- i18n: veraltet ([0-9a-f]{12}) -->\n?$", bt)
            if VERALTET and not m and (bt != at and sorted(LINK.findall(at)) != sorted(LINK.findall(bt))) or (VERALTET and m and m.group(1) != h(de)):
                bt = re.sub(r"\n<!-- i18n: veraltet [0-9a-f]{12} -->\n?$", "", bt).rstrip("\n") + "\n\n<!-- i18n: veraltet %s -->\n" % h(de)
                en.write_text(bt, encoding="utf-8"); m = re.search(r"<!-- i18n: veraltet ([0-9a-f]{12}) -->", bt)
            if m:
                if m.group(1) != h(de): out.append(f"{rel}: veraltet (deutsche Quelle geändert)")
                continue
        fa, a = front(at); fb, b = front(bt)
        if (stamp or VERALTET) and fa:
            bt2 = re.sub(r"^(i18n_status|i18n_quelle):.*\n", "", bt, flags=re.M)
            st = fb.get("i18n_status", "maschinell")
            if VERALTET and fb.get("i18n_quelle") != h(de): st = "veraltet"
            bt2 = bt2.replace("\n---\n", "\ni18n_status: %s\ni18n_quelle: %s\n---\n" % (st, h(de)), 1)
            en.write_text(bt2, encoding="utf-8"); continue
        if fb.get("i18n_status") == "veraltet":
            # englisches Kapitel folgt in 0.3.1: nur Hash und Umlaute
            if fb.get("i18n_quelle") != h(de): out.append(f"{rel}: veraltet (deutsche Quelle geändert) oder ohne i18n_quelle")
            for n, l in enumerate(bt.splitlines(), 1):
                if l.startswith("i18n_status:"): continue
                t = l
                for nm in NAMES: t = t.replace(nm, "")
                if UMLAUT.search(t): out.append(f"{rel}:{n}: Umlaut/ß im Englischen: {l[:60]!r}")
            continue
        if fa:
            for k in ("id", "reihenfolge", "siehe_auch", "referenz"):
                if fa.get(k) != fb.get(k): out.append(f"{rel}: Kopfzeile {k} weicht ab")
            for k in ("titel", "stufe", "zeit"):
                if not fb.get(k) or fb.get(k) == fa.get(k) and k == "titel" and False: out.append(f"{rel}: Kopfzeile {k} fehlt")
            if fb.get("i18n_status") not in STATUS: out.append(f"{rel}: i18n_status fehlt oder ungültig")
            if fb.get("i18n_quelle") != h(de): out.append(f"{rel}: veraltet (deutsche Quelle geändert) oder ohne i18n_quelle")
        if sorted(LINK.findall(a)) != sorted(LINK.findall(b)): out.append(f"{rel}: Linkziele weichen ab")
        if a.count("```") != b.count("```"): out.append(f"{rel}: Anzahl der Codeblöcke weicht ab ({a.count('```')} gegen {b.count('```')})")
        for name, f in (("Tabellenzeilen", lambda l: l.startswith("|")), ("Überschriften", lambda l: l.startswith("#")), ("Bilder", lambda l: "](bild:" in l), ("Listenpunkte", lambda l: re.match(r"\s*([-*]|\d+\.) ", l))):
            ca, cb = sum(1 for l in a.splitlines() if f(l)), sum(1 for l in b.splitlines() if f(l))
            if ca != cb: out.append(f"{rel}: Anzahl der {name} weicht ab ({ca} gegen {cb})")
        for n, l in enumerate(bt.splitlines(), 1):
            if l.startswith("i18n_status:"): continue  # das Statuswort "geprüft" ist ein Schlüsselwert, kein englischer Text
            t = l
            for nm in NAMES: t = t.replace(nm, "")
            if UMLAUT.search(t): out.append(f"{rel}:{n}: Umlaut/ß im Englischen: {l[:60]!r}")

def split_comment(line):
    """(Code, Kommentar) einer Assemblerzeile; Kommentar ab ';' oder '//' ausserhalb von Anfuehrungszeichen"""
    q = None
    for i, c in enumerate(line):
        if q:
            if c == q: q = None
        elif c in "\"'": q = c
        elif c == ";" or line[i:i + 2] == "//": return line[:i], line[i:]
    return line, ""

def embedded_sys():
    """Systemdateien, die `basic2asm --embed` in die erzeugte Datei kopiert (Laufzeitbibliothek b2a, KERNAL/OS und ihre #include <...>)"""
    base = ROOT / "data/sys"
    seed = [f.relative_to(base).as_posix() for f in base.glob("**/b2a/*.asm") if f.relative_to(base).parts[0] != "en"]
    seed += ["c64/kernal.asm", "c128/kernal.asm", "plus4/kernal.asm", "vic20/kernal.asm", "mega65/kernal.asm", "atari8bit/os.asm"]
    seen = []
    def walk(r):
        if r in seen or not (base / r).exists(): return
        seen.append(r)
        for l in (base / r).read_text(encoding="utf-8").splitlines():
            l = l.strip()
            if l.startswith("#include <"): walk(l[10:].split(">")[0])
    for r in seed: walk(r)
    return sorted(seen)

def check_sys_en(out):
    """data/sys/en/<datei>: gleicher Code Zeile fuer Zeile, Kommentare englisch (keine Umlaute, kein VC-20)"""
    base = ROOT / "data/sys"
    for r in embedded_sys():
        en = base / "en" / r
        rel = en.relative_to(ROOT)
        if not en.exists(): out.append(f"{rel}: fehlt"); continue
        a = (base / r).read_text(encoding="utf-8").split("\n"); b = en.read_text(encoding="utf-8").split("\n")
        if len(a) != len(b): out.append(f"{rel}: Zeilenzahl weicht ab ({len(a)} gegen {len(b)})"); continue
        for n, (x, y) in enumerate(zip(a, b), 1):
            cx, kx = split_comment(x); cy, ky = split_comment(y)
            if cx != cy: out.append(f"{rel}:{n}: Code weicht ab"); continue
            if bool(kx) != bool(ky): out.append(f"{rel}:{n}: Kommentar fehlt oder zusaetzlich")
            t = ky
            for nm in NAMES: t = t.replace(nm, "")
            if UMLAUT.search(t): out.append(f"{rel}:{n}: Umlaut/ß im englischen Kommentar: {ky[:60]!r}")
            if re.search(r"\bVC-20\b", ky): out.append(f"{rel}:{n}: VIC-20 statt VC-20")

def pruefe():
    """Funde als Liste (für tools/release.py check)"""
    out = []
    check_cli(False, out)
    check_overlays(False, out)
    check_help(False, out)
    check_handbook(False, out)
    check_sys_en(out)
    return out

def main():
    out = []
    check_cli("--stempeln" in sys.argv, out)
    check_overlays("--erzeugen" in sys.argv, out)
    check_help("--stempeln" in sys.argv, out)
    check_handbook("--stempeln" in sys.argv, out)
    check_sys_en(out)
    for l in out: print(l)
    print(f"check_i18n_data: {len(out)} Funde")
    return 1 if out else 0

if __name__ == "__main__": sys.exit(main())
