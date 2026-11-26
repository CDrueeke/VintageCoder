#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Englische Beispielprojekte (Etappe I6): Spiegelordner examples/en/<system>/<projekt>/ mit den geänderten Textdateien.

Aufbau: `examples/<system>/<projekt>/` bleibt deutsch und unverändert. Der Spiegel enthält nur die Dateien mit englischem Text (gleicher Unterpfad);
beim Öffnen/Kopieren eines Beispiels in englischer Sprache werden diese Dateien über das deutsche Projekt gelegt.
Zwei Arten von Projekten:
  * Einstiegsbeispiele (alle außer den Prallball-Spielen): Kommentare, Bildschirmtexte (Zeichenketten), Projektname/-beschreibung und README englisch.
  * Große Spiele (examples/*/prallball*, examples/*/troxx): Programm bleibt byte-gleich; im Spiegel nur Projektdatei (Titel/Beschreibung) und README;
    der Kopfkommentar der Hauptdatei steht zweisprachig im deutschen Quelltext selbst.
Aufruf:
  python3 tools/examples_en.py anlegen    Kopien der Dateien mit deutschem Text im Spiegel anlegen (vorhandene bleiben)
  python3 tools/examples_en.py pruefen    prüft den Spiegel: Code gleich, nur Kommentare/Zeichenketten/Beschreibung verschieden, keine deutschen Reste
  python3 tools/examples_en.py bauen      baut deutsche und englische Fassung (Binärdatei `vcc` nötig) und vergleicht die Ausgabe
Rückgabewert 1 bei Funden."""
import json, os, re, shutil, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EX = ROOT / "examples"
EN = EX / "en"
UMLAUT = re.compile(r"[äöüÄÖÜß]")
NAMES = [l.strip() for l in (ROOT / "tools/i18n_names.txt").read_text(encoding="utf-8").splitlines() if l.strip() and not l.startswith("#")]
DE_WORDS = set("""und oder der die das den dem des ein eine einen einem einer nicht kein keine ist sind wird werden mit ohne für von zu zum zur auf aus bei nach über unter
wenn dann auch nur noch wie bis hier diese dieser dieses nach beim vor hinter zwischen jede jeder jedes alle dabei danach davor sonst weil damit""".split())
TEXT_EXT = {".asm", ".inc", ".bas", ".tur", ".md", ".txt", ".vcprj", ".vctest"}


def projekte():
    for s in sorted(p for p in EX.iterdir() if p.is_dir() and p.name != "en"):
        for p in sorted(q for q in s.iterdir() if q.is_dir()):
            if any(p.glob("*.vcprj")): yield s.name, p.name, p


def gross(name):
    """Große Spiele: Programm byte-gleich, im Spiegel nur .vcprj und README"""
    return name.startswith(("prallball", "troxx"))


def split_comment(line):
    """(Code, Kommentar); Kommentar ab ';' oder '//' (Assembler) ausserhalb von Anfuehrungszeichen; Zeichenketteninhalt bleibt im Code erhalten"""
    q = None
    for i, c in enumerate(line):
        if q:
            if c == q: q = None
        elif c in "\"'": q = c
        elif c == ";" or line[i:i + 2] == "//": return line[:i], line[i:]
    return line, ""


def mask_strings(code):
    """Zeichenketten durch leere ersetzen (Inhalt darf sich im englischen Einstiegsbeispiel unterscheiden)"""
    return re.sub(r'"[^"]*"', '""', code)


def bas_split(line):
    """BASIC (natives, ohne Zeilennummern): Kommentar ab `REM` (am Zeilenanfang oder nach `:`) oder `;` am Zeilenanfang, ausserhalb von Anfuehrungszeichen"""
    t = line.lstrip()
    if t.startswith(";"): return "", line
    q = False
    for i, c in enumerate(line):
        if c == '"': q = not q
        elif not q and line[i:i + 3].upper() == "REM" and (i == 0 or line[:i].rstrip().endswith(":") or not line[:i].strip()) and not line[i + 3:i + 4].isalnum():
            return line[:i], line[i:]
    return line, ""


def deutsch(text):
    if UMLAUT.search(re.sub("|".join(map(re.escape, NAMES)) or "$^", "", text)): return True
    return len(set(re.findall(r"[a-zäöüß]+", text.lower())) & DE_WORDS) >= 2


def hat_deutsch(path):
    t = path.read_text(encoding="utf-8", errors="replace")
    if path.suffix in (".md", ".txt", ".vcprj", ".vctest"): return deutsch(t)
    split = bas_split if path.suffix in (".bas", ".tur") else split_comment
    for l in t.split("\n"):
        code, com = split(l)
        if com and deutsch(com): return True
        for s in re.findall(r'"([^"]*)"', code):
            if deutsch(s): return True
    return False


def anlegen():
    n = 0
    for s, name, p in projekte():
        for f in sorted(p.rglob("*")):
            if not f.is_file() or f.suffix not in TEXT_EXT: continue
            rel = f.relative_to(p)
            if gross(name) and (f.suffix not in (".vcprj", ".md") or f.name == "HANDTEST.md"): continue  # HANDTEST.md (Handprüfliste) bleibt deutsch
            if f.suffix == ".inc" and "Automatisch erzeugt" in f.read_text(encoding="utf-8", errors="replace")[:200]: continue
            if not hat_deutsch(f): continue
            d = EN / s / name / rel
            if d.exists(): continue
            d.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(f, d); n += 1
    print(f"examples_en anlegen: {n} Dateien neu im Spiegel")


def vergleiche(rel, de, en, out, strikt):
    de_t, en_t = de.read_text(encoding="utf-8"), en.read_text(encoding="utf-8")
    ext = en.suffix
    if ext == ".vcprj" or ext == ".vctest":
        a, b = json.loads(de_t), json.loads(en_t)
        if ext == ".vcprj":
            for j in (a, b):
                j.get("project", {}).pop("name", None); j.get("project", {}).pop("description", None)
        else:
            for j in (a, b):
                for t in j.get("tests", []): t.pop("name", None)
        if a != b: out.append(f"{rel}: weicht außerhalb der Textfelder (Projektname/Beschreibung bzw. Testnamen) ab")
        return
    if ext == ".md" or ext == ".txt":
        la, lb = de_t.split("\n"), en_t.split("\n")
        key = lambda ls: (sum(1 for l in ls if l.startswith("#")), sum(1 for l in ls if re.match(r"\s*([-*]|\d+\.) ", l)), sum(1 for l in ls if l.startswith("|")), de_t.count("```") if ls is la else en_t.count("```"))
        if key(la) != key(lb): out.append(f"{rel}: Überschriften/Listen/Tabellen/Codeblöcke weichen ab {key(la)} gegen {key(lb)}")
        return
    split = bas_split if ext in (".bas", ".tur") else split_comment
    la, lb = de_t.split("\n"), en_t.split("\n")
    if len(la) != len(lb): out.append(f"{rel}: Zeilenzahl weicht ab ({len(la)} gegen {len(lb)})"); return
    for n, (x, y) in enumerate(zip(la, lb), 1):
        cx, kx = split(x); cy, ky = split(y)
        if strikt:
            if cx != cy: out.append(f"{rel}:{n}: Code (mit Zeichenketten) weicht ab")
        elif mask_strings(cx) != mask_strings(cy): out.append(f"{rel}:{n}: Code weicht ab"); continue
        if bool(kx) != bool(ky): out.append(f"{rel}:{n}: Kommentar fehlt oder zusätzlich")


FILTER = [a for a in sys.argv[2:]]  # optional: nur Projekte, deren Pfad "<system>/<projekt>" einen dieser Texte enthält


def gewaehlt(rel):
    return not FILTER or any(x in "/".join(Path(rel).parts[:2]) for x in FILTER)


def funde():
    """Funde des Spiegels als Liste (für tools/release.py check)"""
    out = []
    if not EN.exists(): return out
    for f in sorted(EN.rglob("*")):
        if not f.is_file(): continue
        rel = f.relative_to(EN)
        if not gewaehlt(rel): continue
        parts = rel.parts
        if len(parts) < 3: out.append(f"examples/en/{rel}: gehört in examples/en/<system>/<projekt>/"); continue
        de = EX / Path(*parts)
        if not de.exists(): out.append(f"examples/en/{rel}: keine deutsche Quelle"); continue
        vergleiche(f"examples/en/{rel}", de, f, out, gross(parts[1]))
        t = f.read_text(encoding="utf-8")
        for n, l in enumerate(t.split("\n"), 1):
            x = l
            for nm in NAMES: x = x.replace(nm, "")
            if UMLAUT.search(x): out.append(f"examples/en/{rel}:{n}: Umlaut/ß im Englischen: {l.strip()[:60]!r}")
            if re.search(r"\bVC-20\b", l): out.append(f"examples/en/{rel}:{n}: VIC-20 statt VC-20")
            if f.suffix not in (".vcprj", ".vctest") and f.suffix not in (".md", ".txt"):
                split = bas_split if f.suffix in (".bas", ".tur") else split_comment
                com = split(l)[1]
                if com and len(set(re.findall(r"[a-zäöüß]+", com.lower())) & DE_WORDS) >= 2: out.append(f"examples/en/{rel}:{n}: Kommentar noch deutsch: {com.strip()[:60]!r}")
    return out


def pruefen():
    out = funde()
    for l in out: print(l)
    print(f"examples_en pruefen: {len(out)} Funde")
    return 1 if out else 0


def vcc():
    for c in (ROOT / "target/debug/vcc", ROOT / "target/release/vcc"):
        if c.exists(): return str(c)
    sys.exit("vcc nicht gebaut (cargo build -p vc-cli)")


def bauen():
    """Jedes Spiegelprojekt: deutsche Fassung und englische Fassung bauen (vcc build); wo nur Kommentare abweichen, müssen die Bytes gleich sein"""
    out = []
    exe = vcc()
    seen = sorted({(p.parts[0], p.parts[1]) for p in (f.relative_to(EN) for f in EN.rglob("*") if f.is_file()) if len(p.parts) >= 3 and gewaehlt(p)})
    for s, name in seen:
        res = {}
        for lang in ("de", "en"):
            with tempfile.TemporaryDirectory() as td:
                dst = Path(td) / name
                shutil.copytree(EX / s / name, dst)
                if lang == "en":
                    for f in (EN / s / name).rglob("*"):
                        if f.is_file(): shutil.copyfile(f, dst / f.relative_to(EN / s / name))
                env = dict(os.environ, VC_LANG=lang, VC_DATA=str(ROOT / "data"))
                r = subprocess.run([exe, "build", str(dst)], capture_output=True, text=True, env=env, timeout=300)
                if r.returncode != 0 and ("keine Quellen" in r.stdout + r.stderr or "no sources" in r.stdout + r.stderr):
                    print(f"{s}/{name} ({lang}): keine Quellen, nicht baubar (Eigenschaft des Originals)")
                    res[lang] = None; continue
                if r.returncode != 0:
                    out.append(f"{s}/{name} ({lang}): Bau schlägt fehl: {(r.stdout + r.stderr).strip()[-300:]}")
                    res[lang] = None; continue
                res[lang] = {str(f.relative_to(dst / 'build')): f.read_bytes() for f in sorted((dst / "build").rglob("*")) if f.is_file() and f.suffix in (".prg", ".xex", ".crt", ".car", ".bin", ".d64", ".d81", ".a0")}
        if res.get("de") is not None and res.get("en") is not None:
            if gross(name):
                # Diskettenabbilder tragen den Projektnamen als Etikett (englischer Titel, z. B. "VIC-20" statt "VC-20"): nur die Programmdateien müssen gleich sein
                prog = lambda d: {k: v for k, v in d.items() if not k.endswith((".d64", ".d81"))}
                if prog(res["de"]) != prog(res["en"]): out.append(f"{s}/{name}: großes Spiel, aber Programmdatei weicht ab")
                elif res["de"] != res["en"]: print(f"{s}/{name}: Programm byte-gleich, nur das Diskettenetikett (Projektname) unterscheidet sich")
                else: print(f"{s}/{name}: Ausgabe byte-gleich")
            elif res["de"] == res["en"]: print(f"{s}/{name}: Ausgabe byte-gleich")
            else: print(f"{s}/{name}: Ausgabe unterscheidet sich (Zeichenketten übersetzt)")
    for l in out: print(l)
    print(f"examples_en bauen: {len(out)} Funde")
    return 1 if out else 0


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "pruefen"
    if cmd == "anlegen": anlegen(); sys.exit(0)
    sys.exit({"pruefen": pruefen, "bauen": bauen}.get(cmd, lambda: sys.exit(__doc__))())
