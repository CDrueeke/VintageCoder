#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Extraktion sichtbarer Texte der Oberfläche in die Fluent-Kataloge (i18n I1).

  python3 tools/i18n_extract.py scan  DATEI…                 Kandidaten und nicht erfasste Fundstellen auflisten (ändert nichts)
  python3 tools/i18n_extract.py apply DATEI… [--skip Zeile…]  Kandidaten durch vc_i18n::t!(…) ersetzen und crates/vc-i18n/catalog/de/<bereich>.ftl ergänzen
  python3 tools/i18n_extract.py consts DATEI                  `pub const NAME: &str = "…"` einer Textdatei in Funktionen umwandeln (texte.rs)
  python3 tools/i18n_extract.py todo                          Schlüssel, die in en noch fehlen (mit deutschem Text)

Jede Änderung wird durchgesehen (git diff); das Werkzeug erfindet keinen Wortlaut: der deutsche Text bleibt Zeichen für Zeichen erhalten.
Erfasst werden Zeichenketten in Anzeigeaufrufen (text, knopf, tb, abschnitt, klein, …, log.push, Err); Tests und Konstanten bleiben unberührt.
Nicht erfasste Fundstellen (Formatangaben wie {:02X}, Konstanten, unklarer Kontext) stehen in der Ausgabe von `scan` unter „NICHT ERFASST“."""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import i18n_lex as L  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "crates", "vc-i18n", "catalog")

# Aufrufe, deren Zeichenkettenargumente angezeigt werden
ANZEIGE = {"text", "knopf", "knopf_k", "tb", "abschnitt", "klein", "zeile", "chip", "kopf", "label", "hinweis", "seg", "w", "pe_zeile", "pe_wahl", "wahl",
           "karte", "karte_zu", "mono", "span", "text_input", "set_title", "set_description", "placeholder", "tooltip", "Err", "ok_or", "ok_or_else", "rich_text",
           "knopfname", "titel", "feld_klein", "feld", "checkbox", "zaehler", "tip", "ueberschrift", "badge", "zeile_k", "mono_zeile"}
# Aufrufe, deren Zeichenketten nie angezeigt werden (Kennungen, Pfade, Protokoll, Tests)
KEINE = {"json", "serde", "Path", "join", "ends_with", "starts_with", "contains", "replace", "get", "matches", "include_str", "cfg", "assert_eq", "assert", "req", "handle",
         "kind", "var", "var_os", "env", "from", "strip_prefix", "strip_suffix", "split", "trim_matches", "find", "eq_ignore_ascii_case", "insert", "remove", "entry",
         "unwrap_or", "or_insert", "debug_assert", "Id", "id", "log", "eprintln", "println", "debuglog", "write", "writeln", "File", "PathBuf", "format_args", "regex",
         "parse", "add_filter", "extension", "file_name", "with_extension", "open", "create", "read", "set_file_name"}


def slug(text, maxwords=6):
    t = text.lower()
    for a, b in (("ä", "ae"), ("ö", "oe"), ("ü", "ue"), ("ß", "ss")):
        t = t.replace(a, b)
    words = re.findall(r"[a-z0-9]+", t)
    return "-".join(words[:maxwords]) or "text"


def looks_text(s):
    letters = len(re.findall(r"[A-Za-zÄÖÜäöüß]", s))
    if letters < 2:
        return False
    if re.fullmatch(r"[a-z0-9_./:\-+*#@\\=,;()\[\]<>|&%$!?]+", s) and " " not in s:
        return False
    if re.fullmatch(r"[A-Z0-9_./\-+ ]{2,12}", s):        # PAL, NTSC, PRG, D64 …: technische Kürzel
        return False
    if re.fullmatch(r"\{[^}]*\}", s) or s.startswith(("http", "/", "~/", "./", "build/", "data/")) and " " not in s:
        return False
    if re.fullmatch(r"[a-z][a-z0-9_]*", s):
        return False
    if re.match(r"(Cmd|Alt|Umschalt|Strg|Ctrl|Shift)(\+|/)", s) or re.fullmatch(r"F[0-9]{1,2}", s):
        return False       # Kürzel
    if re.search(r"\.(rs|md|asm|bas|json|vcprj|inc|txt|png|prg|d64|vcsong|vcsprite|vcchar|vctest|vcscreen|vctiles|vcpal)\b", s) and " " not in s:
        return False       # Dateinamen
    return True


def callee_name(p):
    return p.split("::")[-1].lstrip(".") if p else ""


def klassifiziere(l):
    """('anzeige'|'keine'|'unklar', Name des entscheidenden Aufrufs)"""
    for p in reversed(l.path):
        n = callee_name(p)
        if not n or n in ("format", "vec", "row", "column", "container", "else", "if", "Some", "Ok", "String", "to_string", "into", "App", "Msg", "push", "in", "match", "as_str", "unwrap", "iter", "map", "collect", "bool", "str"):
            if n == "push":
                continue
            continue
        if n in ANZEIGE:
            return "anzeige", n
        if n in KEINE:
            return "keine", n
    return "unklar", ""


def log_push(src, l):
    """Steht das Literal in `….log.push(…)` (Protokollzeile, wird angezeigt)?"""
    if l.open_pos < 0:
        return False
    return bool(re.search(r"\blog\s*\.push\s*$", src[max(0, l.open_pos - 40):l.open_pos])) or bool(re.search(r"\blog\s*\.push\s*$", src[max(0, l.open_pos - 40):l.open_pos].rstrip()))


def split_args(src, start):
    """Argumente eines Aufrufs ab `start` (hinter der öffnenden Klammer): (Teilstrings, Position der schließenden Klammer)."""
    teile, tiefe, i, akt0, n = [], 0, start, start, len(src)
    while i < n:
        c = src[i]
        if c == '"':
            j = i + 1
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            i = j + 1
            continue
        if c == "'":
            m = re.match(r"'(?:\\.[^']*|[^\\'])'", src[i:i + 12])
            if m:
                i += m.end()
                continue
        if c in "([{":
            tiefe += 1
        elif c in ")]}":
            if tiefe == 0:
                teile.append(src[akt0:i])
                return teile, i
            tiefe -= 1
        elif c == "," and tiefe == 0:
            teile.append(src[akt0:i])
            akt0 = i + 1
        i += 1
    return teile, n


PLATZ = re.compile(r"\{\{|\}\}|\{([A-Za-z_][A-Za-z0-9_]*|[0-9]*)(:[^}]*)?\}")


def format_zerlegen(value, args):
    """Aus einem Formattext Fluent-Text und Parameter machen: (fluent_text, [(name, ausdruck)]) oder (None, Grund)."""
    params, pos, namen, out, last = [], 0, set(), [], 0
    for m in PLATZ.finditer(value):
        out.append(value[last:m.start()])
        last = m.end()
        if m.group(0) in ("{{", "}}"):
            return None, "geschweifte Klammer im Text"
        name, spec = m.group(1), m.group(2)
        if name == "" or name.isdigit():
            idx = pos if name == "" else int(name)
            pos += 1 if name == "" else 0
            if idx >= len(args):
                return None, "zu wenige Argumente"
            expr = args[idx].strip()
            base = expr if re.fullmatch(r"[a-z_][a-z0-9_]*", expr) else f"p{len(params) + 1}"
        else:
            expr, base = name, name
        if spec:
            mm = re.fullmatch(r":\.(\d+)", spec)
            if mm:      # Nachkommastellen: Zahlenformat der Sprache (Komma oder Punkt)
                expr = f"vc_i18n::num(({expr}) as f64, {mm.group(1)})"
            elif re.fullmatch(r":[0-9]*[xXbo]|:#?0?[0-9]*[xXbo]|:[<>^]?[0-9]+|:0[0-9]+", spec):
                expr = f'format!("{{{spec}}}", {expr})'      # Hex, Breite: bleibt wie es ist (Adressen werden nie lokalisiert)
            else:
                return None, "Formatangabe " + spec
            base = base if re.fullmatch(r"[a-z_][a-z0-9_]*", base) else f"p{len(params) + 1}"
        nm, k = base, 2
        while nm in namen and dict(params).get(nm) != expr:
            nm, k = f"{base}{k}", k + 1
        if nm not in namen:
            namen.add(nm)
            params.append((nm, expr))
        out.append("{ $" + nm + " }")
    out.append(value[last:])
    return "".join(out), params


def rand_zerlegen(text):
    """(führend, Kern, nachgestellt): Leerzeichen und Zeilenumbrüche an den Rändern bleiben im Code, der Kern kommt in den Katalog."""
    kern = text.strip()
    if not kern:
        return "", text, ""
    i = text.index(kern[0]) if kern else 0
    return text[:len(text) - len(text.lstrip())], kern, text[len(text.rstrip()):]


def rs_text(t):
    return t.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n").replace("\t", "\\t")


def fluent_wert(text):
    """Wert als Fluent-Zeile(n); None bei nicht darstellbaren Texten."""
    if "{" in text.replace("{ $", "").replace(" }", "}") and re.search(r"\{(?! \$)", text):
        return None
    if text != text.strip() or text == "":
        return None
    if "\n" in text:
        zeilen = text.split("\n")
        return "\n  ".join(zeilen) if all(z.strip() == z for z in zeilen) else None
    if text[0] in "[*.":
        return None
    return text


def lade_katalog(sprache, bereich):
    p = os.path.join(CATALOG, sprache, bereich + ".ftl")
    return open(p, encoding="utf-8").read() if os.path.exists(p) else ""


def alle_schluessel(sprache="de"):
    res = {}
    d = os.path.join(CATALOG, sprache)
    for f in os.listdir(d):
        if f.endswith(".ftl"):
            text = open(os.path.join(d, f), encoding="utf-8").read()
            marks = list(re.finditer(r"^([a-z][a-z0-9_-]*)\s*=[ \t]*", text, re.M))
            for i, m in enumerate(marks):
                ende = marks[i + 1].start() if i + 1 < len(marks) else len(text)
                wert = "\n".join(l for l in text[m.end():ende].split("\n") if not l.startswith("#")).strip()
                res[m.group(1)] = (f[:-4], wert.replace("\n", " ⏎ "))
    return res


class Kandidat:
    def __init__(self, **kw):
        self.__dict__.update(kw)


def kandidaten(pfad):
    src = open(pfad, encoding="utf-8").read()
    cut = L.test_module_start(src)
    cands, offen = [], []
    gesehen_format = set()
    for l in L.tokens(src):
        if l.start >= cut or not re.search(r"[A-Za-zÄÖÜäöüß]", l.value):
            continue
        art, name = klassifiziere(l)
        lp = log_push(src, l)
        if lp:
            art, name = "anzeige", "log.push"
        in_format = name == "format" or callee_name(l.callee) == "format"
        # format!("…", args): ganzen Aufruf ersetzen
        fmt = None
        if callee_name(l.callee) == "format" and l.argno == 0 and l.in_macro:
            args, ende = split_args(src, l.open_pos + 1)
            fmt = (l.open_pos, ende + 1, [a for a in args[1:] if a.strip()])
            if (l.open_pos, ende) in gesehen_format:
                continue
            gesehen_format.add((l.open_pos, ende))
            # Ist das format! selbst in einem Anzeigeaufruf?
            art2 = None
            for p in reversed(l.path[:-1]):
                n = callee_name(p)
                if n in ANZEIGE:
                    art2 = "anzeige"; break
                if n in KEINE:
                    art2 = "keine"; break
            if lp:
                art2 = "anzeige"
            art = art2 or "unklar"
        nach = src[l.end:l.end + 6].lstrip()
        vor = src[max(0, l.start - 4):l.start].rstrip()
        c = Kandidat(datei=pfad, zeile=l.line, wert=l.value, art=art, name=name, lit=l, fmt=fmt, const=l.const_item)
        if (nach.startswith("=>") or nach.startswith("|") and "=>" in src[l.end:l.end + 80]) or vor.endswith(("==", "!=")):
            if looks_text(l.value):
                offen.append((c, "Muster oder Vergleich"))
            continue
        if l.const_item:
            offen.append((c, "Konstante/Tabelle"))
        elif not looks_text(l.value):
            continue
        elif art == "anzeige":
            cands.append(c)
        elif art == "unklar":
            # Kontext nicht erkannt (eigene Hilfsfunktionen): als Kandidat nur, wenn es nach Anzeigetext aussieht; zur Durchsicht markiert
            if " " in l.value or l.value[:1].isupper():
                c.art = "unklar"
                cands.append(c)
            else:
                offen.append((c, "unklarer Kontext"))
    return src, cands, offen


def bereich_von(pfad):
    stem = os.path.splitext(os.path.basename(pfad))[0]
    return {"main": "gui"}.get(stem, stem.replace("_", "-"))


def schluessel_fuer(bereich, text, vorhanden, neu):
    """Schlüssel: gleicher Text im Bereich → derselbe Schlüssel; sonst <bereich>-<slug> (bei Kollision -2, -3 …)."""
    for k, t in neu.items():
        if t == text and k.startswith(bereich + "-"):
            return k
    for k, (_, t) in vorhanden.items():
        if t == text.replace("\n", "") and k.startswith(bereich + "-"):
            return k
    basis = bereich + "-" + slug(text)
    k, i = basis, 2
    while k in vorhanden or k in neu:
        k, i = f"{basis}-{i}", i + 1
    return k


def cmd_scan(dateien):
    n = 0
    for p in dateien:
        src, cands, offen = kandidaten(p)
        print(f"== {os.path.relpath(p, ROOT)}: {len(cands)} erfasst, {len(offen)} nicht erfasst")
        for c in cands:
            print(f"  {c.zeile:5d} [{c.name}] {c.wert[:90]!r}" + (f"  FORMAT args={len(c.fmt[2])}" if c.fmt else ""))
        for c, grund in offen:
            print(f"  {c.zeile:5d} NICHT ERFASST ({grund}) [{c.name}] {c.wert[:80]!r}")
        n += len(cands)
    print(f"{n} Kandidaten")


def ersetzungen(src, cands, bereich, skip):
    """Liste von (start, ende, neuer Text) und neue Katalogeinträge {schlüssel: de-text}"""
    vorhanden = alle_schluessel("de")
    neu, edits, uebersprungen = {}, [], []
    for c in cands:
        if c.zeile in skip:
            continue
        l = c.lit
        if c.fmt:
            start, ende, args = c.fmt
            text, params = format_zerlegen(l.value, args)
            if text is None:
                uebersprungen.append((c.zeile, c.wert, params))
                continue
            span = (start - len("format!"), ende)
            # `format` und `!` vor der Klammer mitnehmen
            m = re.search(r"format\s*!\s*$", src[:start])
            span = (m.start(), ende)
        else:
            text, params = l.value, []
            span = (l.start, l.end)
        lead, kern, trail = rand_zerlegen(text)
        wert = fluent_wert(kern)
        if wert is None:
            uebersprungen.append((c.zeile, c.wert, "nicht als Fluent-Wert darstellbar"))
            continue
        key = schluessel_fuer(bereich, kern, vorhanden, neu)
        neu[key] = kern
        call = f'vc_i18n::t!("{key}"' + "".join(f", {n} = {param_ausdruck(e)}" for n, e in params) + ")"
        if lead or trail:
            call = f'format!("{rs_text(lead)}{{}}{rs_text(trail)}", {call})'
        edits.append((span[0], span[1], call, c))
    return edits, neu, uebersprungen


def cmd_apply(dateien, skip, bereich_opt=None):
    for p in dateien:
        src, cands, _ = kandidaten(p)
        bereich = bereich_opt or bereich_von(p)
        edits, neu, uebersp = ersetzungen(src, cands, bereich, skip)
        edits.sort(key=lambda e: -e[0])
        # überlappende Ersetzungen (Literal innerhalb eines format!) vermeiden: äußere gewinnt
        belegt, final = [], []
        for e in sorted(edits, key=lambda e: e[0]):
            if any(not (e[1] <= b0 or e[0] >= b1) for b0, b1 in belegt):
                continue
            belegt.append((e[0], e[1]))
            final.append(e)
        for s, e, call, c in sorted(final, key=lambda e: -e[0]):
            ersatz = call
            vor = src[max(0, s - 1):s]
            src = src[:s] + ersatz + src[e:]
        open(p, "w", encoding="utf-8").write(src)
        keys = {c.lit.value: None for _, _, _, c in final}
        # Katalog ergänzen (nur die tatsächlich ersetzten)
        benutzt = {}
        for s, e, call, c in final:
            k = re.search(r'vc_i18n::t!\("([^"]+)"', call).group(1)
            text, params = (format_zerlegen(c.lit.value, c.fmt[2]) if c.fmt else (c.lit.value, []))
            benutzt[k] = rand_zerlegen(text)[1]
        if benutzt:
            kp = os.path.join(CATALOG, "de", bereich + ".ftl")
            alt = lade_katalog("de", bereich)
            if not alt:
                alt = f"# SPDX-License-Identifier: GPL-3.0-or-later\n# Texte der Oberfläche: {bereich} (aus {os.path.relpath(p, ROOT)} extrahiert, Wortlaut unverändert)\n"
            ctx = {}
            for s_, e_, call_, c_ in final:
                ctx.setdefault(re.search(r'vc_i18n::t!\("([^"]+)"', call_).group(1), f"{os.path.basename(p)}, {c_.name or 'Text'}")
            for k, t in benutzt.items():
                if re.search(rf"^{re.escape(k)}\s*=", alt, re.M):
                    continue
                v = fluent_wert(t)
                alt += f"\n# Kontext: {ctx.get(k, '')}\n{k} = {v}\n"
            open(kp, "w", encoding="utf-8").write(alt)
        print(f"{os.path.relpath(p, ROOT)}: {len(final)} ersetzt, {len(uebersp)} übersprungen, Bereich {bereich}")
        for z, w, g in uebersp:
            print(f"   übersprungen Zeile {z}: {w[:70]!r} ({g})")


CONST = re.compile(r'pub const ([A-Z][A-Z0-9_]*): &str = "((?:[^"\\]|\\.)*)";')


def cmd_consts(datei, bereich="texte"):
    """`pub const NAME: &str = "…";` in `pub fn name() -> String { vc_i18n::t!("texte-name") }` umwandeln, Katalog ergänzen, Verwendungen umschreiben."""
    src = open(datei, encoding="utf-8").read()
    kopf, rest = src, ""      # ganze Datei: auch Konstanten hinter einem Testmodul
    namen, eintraege = [], {}

    def ersatz(m):
        name, roh = m.group(1), m.group(2)
        text = L.unescape(roh)
        key = bereich + "-" + name.lower().replace("_", "-")
        wert = fluent_wert(text)
        if wert is None:
            return m.group(0)
        namen.append(name)
        eintraege[key] = text
        return f'pub fn {name.lower()}() -> String {{ vc_i18n::t!("{key}") }}'

    kopf = CONST.sub(ersatz, kopf)
    open(datei, "w", encoding="utf-8").write(kopf + rest)
    kp = os.path.join(CATALOG, "de", bereich + ".ftl")
    alt = lade_katalog("de", bereich) or f"# SPDX-License-Identifier: GPL-3.0-or-later\n# Texte aus {os.path.relpath(datei, ROOT)} (Wortlaut unverändert)\n"
    for k, t in eintraege.items():
        if not re.search(rf"^{re.escape(k)}\s*=", alt, re.M):
            alt += f"\n{k} = {fluent_wert(t)}\n"
    open(kp, "w", encoding="utf-8").write(alt)
    # Verwendungen in allen Rust-Dateien der Oberfläche umschreiben
    zaehler = 0
    muster = re.compile(r"\b((?:vc_gui_ui::)?texte|t)::(" + "|".join(sorted(namen, key=len, reverse=True)) + r")\b(?!\()")
    for crate in ("vc-gui", "vc-gui-ui"):
        for d, _, fs in os.walk(os.path.join(ROOT, "crates", crate, "src")):
            for f in fs:
                if not f.endswith(".rs"):
                    continue
                pf = os.path.join(d, f)
                t = open(pf, encoding="utf-8").read()
                n, c = muster.subn(lambda m: f"{m.group(1)}::{m.group(2).lower()}()", t)
                if c:
                    open(pf, "w", encoding="utf-8").write(n)
                    zaehler += c
    print(f"{len(namen)} Konstanten umgewandelt, {zaehler} Verwendungen umgeschrieben, Katalog {os.path.relpath(kp, ROOT)}")


def param_ausdruck(e):
    """Reine Namen und Feldzugriffe (`a`, `x.name`) als Verweis übergeben: so wird nichts verschoben; Aufrufe und Berechnungen bleiben wie sie sind."""
    e = e.strip()
    return "&" + e if re.fullmatch(r"[a-z_][a-z0-9_]*(\.[a-z_][a-z0-9_]*)*", e) else e


def cmd_refs(dateien):
    """Bestehende t!-Aufrufe: Parameter, die reine Namen oder Feldzugriffe sind, als Verweis übergeben."""
    for p in dateien:
        src = open(p, encoding="utf-8").read()
        out, pos, n = [], 0, 0
        for m in re.finditer(r'vc_i18n::t!\(\s*"[^"]+"', src):
            if m.start() < pos:
                continue
            teile, ende = split_args(src, m.end())
            neu = []
            for t in teile[1:]:
                mm = re.match(r"(\s*[a-z_][a-z0-9_]*\s*=\s*)(.*?)(\s*)$", t, re.S)
                if mm and param_ausdruck(mm.group(2)) != mm.group(2).strip():
                    neu.append(mm.group(1) + param_ausdruck(mm.group(2)) + mm.group(3)); n += 1
                else:
                    neu.append(t)
            out.append(src[pos:m.end()] + "".join("," + t for t in neu))
            pos = ende
        out.append(src[pos:])
        if n:
            open(p, "w", encoding="utf-8").write("".join(out))
        print(f"{os.path.relpath(p, ROOT)}: {n} Parameter als Verweis")


def cmd_todo():
    de, en = alle_schluessel("de"), alle_schluessel("en")
    fehlt = [(k, v) for k, v in de.items() if k not in en]
    for k, (b, t) in sorted(fehlt, key=lambda kv: (kv[1][0], kv[0])):
        print(f"{b}\t{k}\t{t}")
    print(f"{len(fehlt)} Schlüssel ohne englischen Text", file=sys.stderr)


def main(argv):
    if len(argv) < 2 or argv[1] in ("-h", "--help"):
        print(__doc__)
        return 0
    cmd, rest = argv[1], argv[2:]
    skip = set()
    if "--skip" in rest:
        i = rest.index("--skip")
        skip = {int(x) for x in rest[i + 1:]}
        rest = rest[:i]
    bereich = None
    if "--bereich" in rest:
        i = rest.index("--bereich")
        bereich = rest[i + 1]
        rest = rest[:i] + rest[i + 2:]
    if cmd == "scan":
        cmd_scan(rest)
    elif cmd == "apply":
        cmd_apply(rest, skip, bereich)
    elif cmd == "refs":
        cmd_refs(rest)
    elif cmd == "consts":
        cmd_consts(rest[0], bereich or "texte")
    elif cmd == "todo":
        cmd_todo()
    else:
        print("unbekannter Befehl", cmd)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
