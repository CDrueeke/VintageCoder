#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Grundprüfung der Kataloge (i18n): crates/vc-i18n/catalog/{de,en}/*.ftl gegen den Code.

  * Jeder im Code benutzte Schlüssel (t!("…")) steht in de und en.
  * de und en haben dieselben Dateien, dieselben Schlüssel und je Schlüssel dieselben Parameter ({ $name }).
  * Jeder Aufruf t!("schlüssel", name = …) übergibt genau die Parameter des Schlüssels.
  * Kein Schlüssel bleibt ungenutzt (außer mit der Zeile "# veraltet seit …" direkt darüber).
  * Englische Kataloge enthalten keine Umlaute und kein ß, außer in den Eigennamen aus tools/i18n_names.txt.
Fehler = Exit 1. Aufruf: python3 tools/check_i18n.py"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "crates", "vc-i18n", "catalog")
NAMES = os.path.join(ROOT, "tools", "i18n_names.txt")
CODE_DIRS = ["crates"]
SKIP_CODE = {os.path.join("crates", "vc-i18n", "src", "tests.rs")}
SPRACHEN = ("de", "en")
KEY = re.compile(r"^([a-z][a-z0-9_]*(?:-[a-z0-9_]+)*)\s*=", re.M)
PARAM = re.compile(r"\{\s*\$([A-Za-z_][A-Za-z0-9_]*)")
UMLAUT = re.compile("[äöüÄÖÜß]")
DOPPELTE = []  # Schlüssel, die in einer Datei mehrfach stehen (füllt lies_katalog)


def lies_katalog(pfad):
    """Schlüssel → (Parametermenge, veraltet) einer .ftl-Datei (Mehrzeilenwerte gehören zum Schlüssel davor)."""
    text = open(pfad, encoding="utf-8").read()
    out = {}
    marks = [(m.group(1), m.start(), m.end()) for m in KEY.finditer(text)]
    sehen = set()
    for key, _, _ in marks:
        if key in sehen:
            DOPPELTE.append(f"{os.path.relpath(pfad, ROOT)}: Schlüssel {key} kommt in derselben Datei doppelt vor")
        sehen.add(key)
    for i, (key, start, _) in enumerate(marks):
        ende = marks[i + 1][1] if i + 1 < len(marks) else len(text)
        block = text[start:ende]
        # Kommentarzeilen vor dem nächsten Schlüssel gehören nicht mehr zu diesem Wert
        wert = "\n".join(l for l in block.split("\n") if not l.lstrip().startswith("#"))
        vorher = text[:start].rstrip("\n").split("\n")[-1] if start else ""
        out[key] = (set(PARAM.findall(wert)), vorher.strip().startswith("# veraltet seit"))
    return out


def kataloge():
    """sprache → datei → schlüssel → (parameter, veraltet)"""
    res = {}
    for sp in SPRACHEN:
        d = os.path.join(CATALOG, sp)
        res[sp] = {f: lies_katalog(os.path.join(d, f)) for f in sorted(os.listdir(d)) if f.endswith(".ftl")} if os.path.isdir(d) else {}
    return res


def code_dateien():
    for top in CODE_DIRS:
        for d, dirs, files in os.walk(os.path.join(ROOT, top)):
            dirs[:] = [x for x in dirs if x not in ("target", ".git")]
            for f in files:
                if f.endswith(".rs"):
                    p = os.path.join(d, f)
                    if os.path.relpath(p, ROOT) not in SKIP_CODE:
                        yield p


def argumente(text, start):
    """Teilt die Argumente eines Aufrufs ab `start` (hinter der öffnenden Klammer) bei Kommas der obersten Ebene; Zeichenketten werden übersprungen."""
    teile, tiefe, i, akt, n = [], 0, start, "", len(text)
    while i < n:
        c = text[i]
        if c == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            akt += text[i:j + 1]
            i = j + 1
            continue
        if c in "([{":
            tiefe += 1
        elif c in ")]}":
            if tiefe == 0:
                teile.append(akt)
                return teile
            tiefe -= 1
        elif c == "," and tiefe == 0:
            teile.append(akt)
            akt = ""
            i += 1
            continue
        akt += c
        i += 1
    return teile


CALL = re.compile(r'\b(?:t|msg)!\(\s*"([^"]+)"|\bMsg::new\(\s*"([^"]+)"')


def benutzt():
    """(Datei, Zeile, Schlüssel, Parametermenge) aller t!-Aufrufe im Code"""
    res = []
    for p in code_dateien():
        text = open(p, encoding="utf-8", errors="replace").read()
        # Kommentarzeilen (auch Doku-Kommentare mit Beispielen) zählen nicht; Zeilenzahl bleibt gleich
        text = "\n".join("" if l.lstrip().startswith("//") else l for l in text.split("\n"))
        for m in CALL.finditer(text):
            teile = argumente(text, m.end())
            if m.group(2):          # Msg::new("schlüssel").with("name", …): Parameter stehen in .with("name", …)
                rest = text[m.end():m.end() + 600]
                namen_w = set(re.findall(r'\.with\(\s*"([A-Za-z_][A-Za-z0-9_]*)"', rest.split(";")[0]))
                res.append((os.path.relpath(p, ROOT), text.count("\n", 0, m.start()) + 1, m.group(2), namen_w))
                continue
            namen = set()
            for t in teile[1:]:
                mm = re.match(r"\s*([A-Za-z_][A-Za-z0-9_]*)\s*=(?!=)", t)
                if mm:
                    namen.add(mm.group(1))
            res.append((os.path.relpath(p, ROOT), text.count("\n", 0, m.start()) + 1, m.group(1), namen))
    return res


def eigennamen():
    try:
        return [l.strip() for l in open(NAMES, encoding="utf-8") if l.strip() and not l.startswith("#")]
    except OSError:
        return []


BEFEHLE_RS = [os.path.join(ROOT, "crates", "vc-gui-ui", "src", "befehle.rs"), os.path.join(ROOT, "crates", "vc-commands", "src", "lib.rs")]


def slug_menue(t):
    t = t.lower()
    for a, b in (("ä", "ae"), ("ö", "oe"), ("ü", "ue"), ("ß", "ss")):
        t = t.replace(a, b)
    return "-".join(re.findall(r"[a-z0-9]+", t))


def dynamische_schluessel():
    """Schlüssel, die nicht als Literal im Code stehen, sondern aus Tabellen entstehen: cmd-<Kennung> (BEFEHLE) und submenu-<Name> (UNTERMENUES)."""
    src = ""
    for pfad in BEFEHLE_RS:
        try:
            src += open(pfad, encoding="utf-8").read() + "\n"
        except OSError:
            pass
    # Platzhalter des Argumenteditors: der Schlüssel entsteht aus dem Namen (rest-platzhalter-<name>-label|hint, vc-shell/einst.rs)
    extra = set()
    try:
        extra = set(re.findall(r"^(rest-platzhalter-[a-z0-9-]+)\s*=", open(os.path.join(CATALOG, "de", "restmeldungen.ftl"), encoding="utf-8").read(), re.M))
    except OSError:
        pass
    if not src:
        return extra
    keys = extra | {"cmd-" + m.group(1) for m in re.finditer(r'^\s+i\(Cmd::\w+, "([^"]+)"', src, re.M)}
    blk = re.search(r"UNTERMENUES: &\[\(&str, &\[&str\]\)\] = &\[(.*?)\n\];", src, re.S)
    if blk:
        for m in re.finditer(r'\("([^"]+)", &\[([^\]]*)\]\)', blk.group(1)):
            for u in re.findall(r'"([^"]+)"', m.group(2)):
                keys.add("submenu-" + slug_menue(u))
    return keys


ANZEIGE_CRATES = ["vc-gui", "vc-gui-ui", "vc-ide-model"]
# Etappe I2: Meldungen von Kern, Assembler, BASIC, Sprachserver, Kommandozeile, Medien, Emulatoren, Disk (rekursiv unter src/)
MELDUNG_CRATES = ["vc-core", "vc-asm", "vc-basic", "vc-lsp", "vc-cli", "vc-media", "vc-emu", "vc-disk", "vc-target", "vc-roms"]
ANZEIGE_DATEIEN_IDE = "blob b2a setup build launch targets update license importfiles copyproject snapshot diskimage emulators mega65net projfile settings reorganize platform structure feedback trace license tests/mod tests/suite tests/runner tests/mailbox tests/cycles".split()
AUSNAHMEN = os.path.join(ROOT, "tools", "i18n_ausnahmen.txt")
DEUTSCH = set("""und oder der die das den dem des ein eine einen einem einer nicht kein keine keinen keines ist sind wird werden wurde fehlt fehlen mit ohne für von zu zum zur auf aus bei nach über unter
bitte datei dateien ordner projekt bauen starten laden speichern öffnen schließen löschen fehler hinweis warnung wähle wählen gewählt neu alle noch schon nur auch wenn dann als wie ob kann können muss müssen
gibt hier dies diese dieser dieses nichts leer zeile zeilen spalte geändert ungültig ungültige gefunden verbunden bereit fertig läuft startet baut bestanden frei belegt gestern gestartet geladen benutzt angelegt
aufgeklappt zugeklappt ausgeklappt eingeklappt minimiert maximiert ja nein schmaler breiter niedriger höher aktuell gespeichert ungespeichert entfernt""".split())

# zusaetzliche deutsche Woerter, die nur in den Meldungs-Crates (MELDUNG_CRATES) gelten: dort stehen fast nur Meldungen
DEUTSCH_MELDUNG = set("""existiert bereits unbekannt unbekannte vorhanden erlaubt unterstützt zielsystem vorlage fehlend enthalten wellenform verzerrung erwartet erwartete
gesetzt zulässig mindestens höchstens zwischen innerhalb außerhalb statt sondern weil sowie jede jeder jedes zahl spalte laufwerk diskette disketten zeichensatz bildschirm abbild
unplausible kachel eintrag""".split())

def ausnahmen():
    """datei → Liste von Mustern (Teilzeichenketten); `*` = ganze Datei. Format der Datei: `pfad<TAB>muster`, Kommentare mit #."""
    out = {}
    if os.path.exists(AUSNAHMEN):
        for z in open(AUSNAHMEN, encoding="utf-8").read().split("\n"):
            if z.strip() and not z.startswith("#") and "\t" in z:
                f, m = z.split("\t", 1)
                out.setdefault(f.strip(), []).append(m.replace("\\n", "\n"))
    return out


def anzeige_dateien():
    for c in ANZEIGE_CRATES:
        d = os.path.join(ROOT, "crates", c, "src")
        for f in sorted(os.listdir(d)):
            if f.endswith(".rs"):
                yield os.path.join("crates", c, "src", f)
    for c in MELDUNG_CRATES:
        for d, dirs, files in os.walk(os.path.join(ROOT, "crates", c, "src")):
            dirs.sort()
            for f in sorted(files):
                if f.endswith(".rs"):
                    yield os.path.relpath(os.path.join(d, f), ROOT)
    for f in ANZEIGE_DATEIEN_IDE:
        p = os.path.join("crates", "vc-ide", "src", f + ".rs")
        if os.path.exists(os.path.join(ROOT, p)):
            yield p


def feste_texte(alle=False):
    """Deutsche Zeichenketten im Code der Anzeige-Crates, die nicht durch `t!` laufen und nicht in tools/i18n_ausnahmen.txt stehen.
    Als deutsch gilt: Umlaut/ß oder ein Wort aus DEUTSCH. Tests und Schlüssel (a-b-c) zählen nicht."""
    sys.path.insert(0, os.path.join(ROOT, "tools"))
    import i18n_lex as L
    ausn = ausnahmen()
    fehler = []
    for rel in anzeige_dateien():
        rel_u = rel.replace(os.sep, "/")
        muster = ausn.get(rel_u, [])
        if "*" in muster:
            continue
        src = open(os.path.join(ROOT, rel), encoding="utf-8").read()
        cut = L.test_module_start(src)
        if cut is not None:
            src = src[:cut]
        for lit in L.tokens(src):
            v = lit.value
            if not isinstance(v, str) or len(v) < 3 or re.fullmatch(r"[a-z0-9_]+(-[a-z0-9_]+)+", v):
                continue
            if lit.callee in ("vc_i18n::t", "t", "t_in", "vc_i18n::translate", "msg", "vc_core::msg"):
                continue
            woerter = set(re.findall(r"[A-Za-zÄÖÜäöüß]+", v.lower()))
            plus = DEUTSCH_MELDUNG if any(f"crates/{c}/" in rel_u for c in MELDUNG_CRATES) else set()
            if not (UMLAUT.search(v) or woerter & DEUTSCH or woerter & plus):
                continue
            if any((v == m[1:]) if m.startswith("=") else (m in v) for m in muster):  # "=text": nur genau dieses Literal
                continue
            fehler.append(f"{rel_u}:{lit.line}: fester deutscher Text {v[:70]!r} (in t!() fassen oder in tools/i18n_ausnahmen.txt begründen)")
    return fehler


# ----------------------------------------------------------------------------------------------------------- Oberfläche ui/ (E0)
UI_DIR = os.path.join(ROOT, "ui", "src")
UI_KEY = re.compile(r"^[a-z][a-z0-9_]*(?:-[a-z0-9_]+)+$")
UI_CALL = re.compile(r"(?<![A-Za-z0-9_$.])t\(\s*([\"'`])((?:(?!\1).)*)\1")


def ui_dateien():
    for d, dirs, files in os.walk(UI_DIR):
        dirs[:] = [x for x in dirs if x not in ("node_modules", "dist")]
        for f in sorted(files):
            if f.endswith((".ts", ".svelte")) and not f.endswith(".test.ts") and not f.endswith(".d.ts"):
                yield os.path.join(d, f)


def ohne_kommentare(text, svelte):
    """Kommentare durch Leerraum ersetzen (Zeilenzahl bleibt); Zeichenketten bleiben unberührt."""
    out, i, n = [], 0, len(text)
    while i < n:
        c = text[i]
        if c in "\"'`":
            j = i + 1
            while j < n and text[j] != c:
                j += 2 if text[j] == "\\" else 1
            out.append(text[i:j + 1]); i = j + 1; continue
        if text.startswith("//", i) and (i == 0 or text[i - 1] != ":"):
            j = text.find("\n", i); j = n if j < 0 else j
            out.append(" " * (j - i)); i = j; continue
        if text.startswith("/*", i):
            j = text.find("*/", i); j = n if j < 0 else j + 2
            out.append(re.sub(r"[^\n]", " ", text[i:j])); i = j; continue
        if svelte and text.startswith("<!--", i):
            j = text.find("-->", i); j = n if j < 0 else j + 3
            out.append(re.sub(r"[^\n]", " ", text[i:j])); i = j; continue
        out.append(c); i += 1
    return "".join(out)


def ui_aufrufe():
    """(Datei, Zeile, Schlüssel oder Präfix, dynamisch, Parameternamen oder None) aller t(…)-Aufrufe in ui/src"""
    res = []
    for p in ui_dateien():
        text = ohne_kommentare(open(p, encoding="utf-8").read(), p.endswith(".svelte"))
        for m in UI_CALL.finditer(text):
            key = m.group(2)
            dyn = "${" in key
            namen = None
            rest = text[m.end():]
            mm = re.match(r"\s*,\s*\{", rest)
            if mm and not dyn:
                tiefe, j, akt = 1, mm.end(), ""
                while j < len(rest) and tiefe:
                    ch = rest[j]
                    tiefe += ch in "{[(" ; tiefe -= ch in "}])"
                    akt += ch; j += 1
                inner = akt[:-1]
                top = re.sub(r"[\[(\{][^\]\)\}]*[\]\)\}]", "", inner)
                namen = set(re.findall(r"(?:^|,)\s*([A-Za-z_]\w*)\s*(?::|(?=,|$))", top))
            res.append((os.path.relpath(p, ROOT), text.count("\n", 0, m.start()) + 1, key.split("${")[0] if dyn else key, dyn, namen))
    return res


def ui_literale():
    """Zeichenketten in ui/src, die genau ein Katalogschlüssel sind (Tabellen wie nav.ts) → gelten als benutzt"""
    res = set()
    for p in ui_dateien():
        text = ohne_kommentare(open(p, encoding="utf-8").read(), p.endswith(".svelte"))
        res |= {m.group(2) for m in re.finditer(r"([\"'])([a-z][a-z0-9_]*(?:-[a-z0-9_]+)+)\1", text)}
    return res


def ui_feste_texte():
    """Deutsche Texte in ui/src (Textknoten, Attribute, Zeichenketten) ohne t(…); Ausnahmen in tools/i18n_ausnahmen.txt."""
    ausn = ausnahmen()
    fehler = []
    for p in ui_dateien():
        rel = os.path.relpath(p, ROOT).replace(os.sep, "/")
        muster = ausn.get(rel, [])
        if "*" in muster:
            continue
        roh = open(p, encoding="utf-8").read()
        text = ohne_kommentare(roh, p.endswith(".svelte"))
        kandidaten = []
        for m in re.finditer(r"([\"'])((?:(?!\1)[^\\\n]|\\.)*)\1", text):
            kandidaten.append((m.start(), m.group(2)))
        if p.endswith(".svelte"):
            ohne = re.sub(r"<(script|style)\b.*?</\1>", lambda m: re.sub(r"[^\n]", " ", m.group(0)), text, flags=re.S)
            for m in re.finditer(r">([^<>{}]+)<", ohne):
                kandidaten.append((m.start(), m.group(1)))
        for pos, v in kandidaten:
            if len(v.strip()) < 3 or UI_KEY.match(v.strip()) or re.fullmatch(r"[a-z]+(_[a-z]+)+", v.strip()):
                continue
            woerter = set(re.findall(r"[A-Za-zÄÖÜäöüß]+", v.lower()))
            if not (UMLAUT.search(v) or woerter & DEUTSCH):
                continue
            if any(mm in v for mm in muster):
                continue
            fehler.append(f"{rel}:{roh.count(chr(10), 0, pos) + 1}: fester deutscher Text {v.strip()[:60]!r} in ui/ (in t(…) fassen oder in tools/i18n_ausnahmen.txt begründen)")
    return fehler


def pruefe_ui(alle):
    """alle: Sprache → Schlüssel → (Datei, Parameter). Rückgabe: (Fehler, benutzte Schlüssel, benutzte Präfixe)"""
    fehler, benutzt_, praefixe = [], set(), set()
    de = alle.get("de", {})
    for datei, zeile, key, dyn, namen in ui_aufrufe():
        if dyn:
            praefixe.add(key)
            if not any(k.startswith(key) for k in de):
                fehler.append(f"{datei}:{zeile}: kein Schlüssel mit Präfix {key} im Katalog")
            continue
        benutzt_.add(key)
        if key not in de or key not in alle.get("en", {}):
            fehler.append(f"{datei}:{zeile}: Schlüssel {key} fehlt im Katalog (de und en)")
        elif namen is not None and namen != de[key][1]:
            fehler.append(f"{datei}:{zeile}: t(\"{key}\") übergibt {sorted(namen)}, der Katalog erwartet {sorted(de[key][1])}")
        elif namen is None and de[key][1]:
            fehler.append(f"{datei}:{zeile}: t(\"{key}\") übergibt keine Parameter, der Katalog erwartet {sorted(de[key][1])}")
    benutzt_ |= {k for k in ui_literale() if k in de}
    fehler += ui_feste_texte()
    return fehler, benutzt_, praefixe


def pruefe():
    fehler = []
    del DOPPELTE[:]
    k = kataloge()
    fehler += DOPPELTE
    de, en = k["de"], k["en"]
    for f in sorted(set(de) | set(en)):
        if f not in de:
            fehler.append(f"catalog/de/{f}: fehlt (nur englisch vorhanden)")
        if f not in en:
            fehler.append(f"catalog/en/{f}: fehlt (nur deutsch vorhanden)")
    alle = {}
    for sp, dateien in ((s, k[s]) for s in SPRACHEN):
        for f, keys in dateien.items():
            for key, (par, _) in keys.items():
                if key in alle.get(sp, {}):
                    fehler.append(f"catalog/{sp}/{f}: Schlüssel {key} doppelt (auch in {alle[sp][key][0]})")
                alle.setdefault(sp, {})[key] = (f, par)
    for key, (f, par) in alle.get("de", {}).items():
        if key not in alle.get("en", {}):
            fehler.append(f"catalog/en: Schlüssel {key} fehlt (de/{f})")
        elif alle["en"][key][1] != par:
            fehler.append(f"Schlüssel {key}: Parameter verschieden, de {sorted(par)} / en {sorted(alle['en'][key][1])}")
    for key, (f, _) in alle.get("en", {}).items():
        if key not in alle.get("de", {}):
            fehler.append(f"catalog/de: Schlüssel {key} fehlt (en/{f})")
    # Verwendung im Code
    benutzte = set()
    for datei, zeile, key, namen in benutzt():
        benutzte.add(key)
        if key not in alle.get("de", {}) or key not in alle.get("en", {}):
            fehler.append(f"{datei}:{zeile}: Schlüssel {key} fehlt im Katalog (de und en)")
        elif alle["de"][key][1] != namen:
            fehler.append(f"{datei}:{zeile}: t!(\"{key}\") übergibt {sorted(namen)}, der Katalog erwartet {sorted(alle['de'][key][1])}")
    ui_fehler, ui_benutzt, ui_praefixe = pruefe_ui(alle)
    fehler += ui_fehler
    benutzte |= ui_benutzt
    benutzte |= {k for k in alle.get("de", {}) if any(k.startswith(p) for p in ui_praefixe)}
    dyn = dynamische_schluessel()
    # Schlüssel, die als Zeichenkettenliteral im Code stehen (Tabellen, `&'static str`-Fehlerarten, `Msg`-Schlüsselwahl), zählen als benutzt
    lit = re.compile(r'"([a-z][a-z0-9_]*(?:-[a-z0-9_]+)+)"')
    for p in code_dateien():
        for m in lit.finditer(open(p, encoding="utf-8", errors="replace").read()):
            if m.group(1) in alle.get("de", {}):
                dyn.add(m.group(1))
    for key in sorted(dyn):
        for sp in SPRACHEN:
            if key not in alle.get(sp, {}):
                fehler.append(f"catalog/{sp}: Schlüssel {key} fehlt (Befehlstabelle in befehle.rs)")
    benutzte |= dyn
    for sp in SPRACHEN:
        for f, keys in k[sp].items():
            for key, (_, veraltet) in keys.items():
                if key not in benutzte and not veraltet and sp == "de":
                    fehler.append(f"catalog/{sp}/{f}: Schlüssel {key} wird nirgends benutzt (oder '# veraltet seit …' darüber setzen)")
    fehler += feste_texte()
    # Meldungsliste für die Oberfläche (docs/diagnostik-codes.md): Kopfzeile und Parameterbeschreibung je diag-Eintrag, Datei aktuell
    sys.path.insert(0, os.path.join(ROOT, "tools"))
    import diag_codes_doc
    text, f2 = diag_codes_doc.erzeuge()
    fehler += f2
    alt = open(diag_codes_doc.OUT, encoding="utf-8").read() if os.path.exists(diag_codes_doc.OUT) else ""
    if alt != text:
        fehler.append("docs/diagnostik-codes.md ist nicht aktuell (python3 tools/diag_codes_doc.py)")
    # Umlaute im Englischen
    namen = eigennamen()
    d = os.path.join(CATALOG, "en")
    if os.path.isdir(d):
        for f in sorted(os.listdir(d)):
            for n, zeile in enumerate(open(os.path.join(d, f), encoding="utf-8").read().split("\n"), 1):
                if zeile.lstrip().startswith("#"):      # Kommentare für Übersetzer dürfen deutsch sein
                    continue
                rest = zeile
                for name in namen:
                    rest = rest.replace(name, "")
                if UMLAUT.search(rest):
                    fehler.append(f"catalog/en/{f}:{n}: Umlaut im englischen Katalog (Eigennamen in tools/i18n_names.txt)")
    return fehler


def main():
    fehler = pruefe()
    for f in fehler:
        print(f)
    if fehler:
        print(f"check_i18n: {len(fehler)} Fund(e)")
        return 1
    print("check_i18n: 0 Funde")
    return 0


if __name__ == "__main__":
    sys.exit(main())
