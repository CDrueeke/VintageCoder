#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt docs/diagnostik-codes.md aus den Katalogen (Schlüssel diag-*): Code, Schlüssel, Parameter mit Bedeutung, Beispiel in de und en.

Quelle sind die Kommentarzeilen über jedem Eintrag in crates/vc-i18n/catalog/de/diag-*.ftl:
    # Fehler E204: Symbol mehrfach definiert
    # $name: Name des Symbols | Beispiel: schleife
    diag-asm-e204 = Symbol '{ $name }' doppelt definiert
  python3 tools/diag_codes_doc.py            Datei schreiben
  python3 tools/diag_codes_doc.py --check    prüfen, ob die Datei aktuell ist und jeder Eintrag Kopfzeile und Parameterzeilen hat (Exit 1 sonst)"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CAT = os.path.join(ROOT, "crates", "vc-i18n", "catalog")
OUT = os.path.join(ROOT, "docs", "diagnostik-codes.md")
KEY = re.compile(r"^(diag-[a-z0-9_-]+)\s*=", re.M)
HEAD = re.compile(r"^#\s*(Fehler|Warnung|Hinweis)\s+([A-Z][0-9]{3}):\s*(.+)$")
PARAM = re.compile(r"^#\s*\$([A-Za-z_][A-Za-z0-9_]*):\s*(.*?)(?:\s*\|\s*Beispiel:\s*(.*))?$")
USED = re.compile(r"\$([A-Za-z_][A-Za-z0-9_]*)")


def lies(sprache):
    """key → dict(text, params(set), kommentar(list), datei)"""
    res = {}
    d = os.path.join(CAT, sprache)
    for f in sorted(os.listdir(d)):
        if not (f.startswith("diag-") and f.endswith(".ftl")):
            continue
        text = open(os.path.join(d, f), encoding="utf-8").read()
        zeilen = text.split("\n")
        i = 0
        kommentar = []
        while i < len(zeilen):
            z = zeilen[i]
            if z.startswith("#"):
                kommentar.append(z)
                i += 1
                continue
            m = KEY.match(z)
            if m:
                wert = [z.split("=", 1)[1].strip()]
                i += 1
                while i < len(zeilen) and (zeilen[i].startswith(" ") or zeilen[i].strip() == "" and i + 1 < len(zeilen) and zeilen[i + 1].startswith(" ")):
                    wert.append(zeilen[i])
                    i += 1
                body = "\n".join(wert)
                res[m.group(1)] = {"text": body, "params": set(USED.findall(body)), "kommentar": kommentar, "datei": f}
                kommentar = []
                continue
            if z.strip() == "":
                kommentar = []
            i += 1
    return res


def beispiel(text, werte):
    """Einfaches Einsetzen; Texte mit Auswahl (->) bleiben unverändert (Fluent-Quelltext)."""
    if "->" in text:
        return None
    return re.sub(r"\{\s*\$([A-Za-z_][A-Za-z0-9_]*)\s*\}", lambda m: werte.get(m.group(1), m.group(0)), text)


def erzeuge():
    de, en = lies("de"), lies("en")
    fehler, eintraege = [], []
    for key, e in de.items():
        kopf = None
        params = {}
        for k in e["kommentar"]:
            m = HEAD.match(k)
            if m:
                kopf = m.groups()
            m = PARAM.match(k)
            if m:
                params[m.group(1)] = (m.group(2), m.group(3) or "")
        if key == "diag-text":
            continue
        if not kopf:
            fehler.append(f"{e['datei']}: {key}: Kopfzeile '# Fehler|Warnung|Hinweis <CODE>: <Beschreibung>' fehlt")
            continue
        for p in sorted(e["params"]):
            if p not in params:
                fehler.append(f"{e['datei']}: {key}: Parameter ${p} ist nicht beschrieben ('# ${p}: Bedeutung | Beispiel: …')")
        for p in params:
            if p not in e["params"]:
                fehler.append(f"{e['datei']}: {key}: beschriebener Parameter ${p} kommt im Text nicht vor")
        if key not in en:
            fehler.append(f"{key}: englischer Eintrag fehlt")
        eintraege.append((kopf[1], key, kopf[0], kopf[2], params, e, en.get(key)))
    eintraege.sort(key=lambda x: (x[0], x[1]))
    o = ["<!-- SPDX-License-Identifier: GPL-3.0-or-later -->", "# Codes und Meldungen von Assembler, BASIC und Werkzeugen", "",
         "Erzeugt von `tools/diag_codes_doc.py` aus `crates/vc-i18n/catalog/de|en/diag-*.ftl`; nicht von Hand ändern (Änderungen im Katalog, dann Werkzeug erneut ausführen).", "",
         "**Maschinenlesbar** liefert jede Meldung (`Diagnostic`, LSP `data`, `vcc --json`):", "",
         "| Feld | Bedeutung |", "|---|---|",
         "| `code` | stabiler Code (E1xx–E3xx Fehler, W1xx/W2xx Warnungen, L… Lizenz, B1xx BASIC); gilt für Assembler **und** BASIC, einzelne Codes kommen in beiden vor |",
         "| `key` | Katalogschlüssel `diag-<crate>-<code>[-<variante>]` (eindeutig, nie umbenannt); die Oberfläche formatiert damit |",
         "| `args` | benannte Parameter (Zahl oder Text; ein Parameter kann selbst `{ key, args }` sein = verschachtelte Meldung) |",
         "| `range` | `{ line, col, end_line, end_col }`, 1-basiert; Spalte 0 = ganze Zeile |",
         "| `message` | fertiger Text in der gewählten Sprache (Anzeige, wenn die Oberfläche nicht selbst formatiert) |", "",
         "Formatieren in der Oberfläche: Vorlage `key` im Katalog der Sprache nachschlagen, `args` als Fluent-Variablen übergeben; verschachtelte Parameter zuerst formatieren und als Text einsetzen. Fehlt `key`, den `message`-Text anzeigen.", "",
         f"{len(eintraege)} Meldungen.", ""]
    letzter = None
    for code, key, art, kurz, params, e, ee in eintraege:
        if code != letzter:
            o += [f"## {code}", ""]
            letzter = code
        o.append(f"### `{key}` ({art}): {kurz}")
        o.append("")
        if params:
            o += ["| Parameter | Bedeutung | Beispiel |", "|---|---|---|"]
            for p, (b, bsp) in params.items():
                o.append(f"| `{p}` | {b} | {bsp} |")
            o.append("")
        werte = {p: (b[1] or "…") for p, b in params.items()}
        for lang, ent in (("de", e), ("en", ee)):
            if ent is None:
                continue
            b = beispiel(ent["text"], werte)
            o.append(f"- {lang}: " + (f"{b}" if b is not None else "(mit Mehrzahl/Auswahl) `" + ent["text"].replace("\n", " ").strip() + "`"))
        o.append("")
    return "\n".join(o).rstrip("\n") + "\n", fehler


def main():
    text, fehler = erzeuge()
    if "--check" in sys.argv:
        alt = open(OUT, encoding="utf-8").read() if os.path.exists(OUT) else ""
        if alt != text:
            fehler.append("docs/diagnostik-codes.md ist nicht aktuell (python3 tools/diag_codes_doc.py)")
        for f in fehler:
            print(f)
        return 1 if fehler else 0
    for f in fehler:
        print("Warnung:", f, file=sys.stderr)
    open(OUT, "w", encoding="utf-8").write(text)
    print(f"docs/diagnostik-codes.md geschrieben ({text.count(chr(10))} Zeilen)")
    return 1 if fehler else 0


if __name__ == "__main__":
    sys.exit(main())
