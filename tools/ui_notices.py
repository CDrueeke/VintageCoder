#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""npm-Pakete der Web-Oberfläche (`ui/package-lock.json`) samt Lizenz in THIRD-PARTY-NOTICES.md nachführen und prüfen.

    python3 tools/ui_notices.py            # Abschnitt neu schreiben
    python3 tools/ui_notices.py --check    # prüfen: Abschnitt aktuell, jede Lizenz GPL-3.0-kompatibel, Paketzahl im Budget, nur Registry-Quellen

Der Abschnitt steht zwischen den Marken UI-PAKETE:BEGIN/END. Nur für die Plattform-Zusatzpakete (optionale Binärpakete wie `@rolldown/binding-*`)
gilt dieselbe Lizenz wie für ihr Hauptpaket; sie werden einzeln mitgezählt, aber nur für die Plattform des Entwicklers installiert."""
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
DOC = ROOT / "THIRD-PARTY-NOTICES.md"
LOCK = ROOT / "ui" / "package-lock.json"
BEGIN, END = "<!-- UI-PAKETE:BEGIN -->", "<!-- UI-PAKETE:END -->"
BUDGET = 125  # Einträge der Sperrdatei (alle Plattformen; auf einem System werden etwa 55 installiert); Konzept 5: „höchstens etwa 60 Pakete“ je System
# Mit GPL-3.0-or-later vereinbar (MPL-2.0 nur für Werkzeuge der Bauzeit, nichts davon gelangt in das ausgelieferte Frontend)
ERLAUBT = {"MIT", "Apache-2.0", "ISC", "BSD-2-Clause", "BSD-3-Clause", "0BSD", "MPL-2.0", "Zlib", "Unlicense", "CC0-1.0"}


def pakete():
    lock = json.loads(LOCK.read_text(encoding="utf-8"))["packages"]
    out = []
    for pfad, p in sorted(lock.items()):
        if not pfad:
            continue
        name = pfad.split("node_modules/")[-1]
        out.append({"name": name, "version": p.get("version", "?"), "license": p.get("license", "UNBEKANNT"), "dev": bool(p.get("dev")), "optional": bool(p.get("optional")), "resolved": p.get("resolved", "")})
    return out


def kennungen(ausdruck):
    return [x for x in re.split(r"\s+(?:OR|AND)\s+|[()]", ausdruck) if x.strip()]


def fehler(pk):
    f = []
    if len(pk) > BUDGET:
        f.append(f"{len(pk)} Pakete in ui/package-lock.json, Budget {BUDGET}")
    for p in pk:
        k = kennungen(p["license"])
        if not k or not any(x.strip() in ERLAUBT for x in k):
            f.append(f"{p['name']}@{p['version']}: Lizenz „{p['license']}“ nicht als GPL-3.0-kompatibel freigegeben (tools/ui_notices.py, ERLAUBT)")
        if p["resolved"] and not p["resolved"].startswith("https://registry.npmjs.org/"):
            f.append(f"{p['name']}: Quelle {p['resolved']} ist nicht die npm-Registry")
        if p["name"].startswith("@tauri-apps/plugin-") and "updater" in p["name"]:
            f.append(f"{p['name']}: Updater-Plugin ist verboten")
    return f


def abschnitt(pk):
    laufzeit = [p for p in pk if not p["dev"]]
    z = [BEGIN, "", "### Pakete der Web-Oberfläche (`ui/`, npm)", "",
         f"Erzeugt mit `python3 tools/ui_notices.py` aus `ui/package-lock.json`: **{len(pk)} Einträge** (alle Plattformen; davon {len(laufzeit)} im ausgelieferten Frontend bzw. zur Laufzeit, der Rest nur zur Bauzeit und zum Prüfen). Installation mit `npm ci --ignore-scripts`. "
         "Das gebaute Frontend enthält nur den Code von `svelte`, `@fluent/bundle` und `@tauri-apps/api` (und deren Laufzeitabhängigkeiten); MPL-2.0-Pakete (`lightningcss*`) sind Werkzeuge der Bauzeit und gelangen nicht in das Programm. Die Bauwerkzeuge laden bei der Installation keine Skripte aus (`--ignore-scripts`). "
         "Nach der Wahl der Festbreitenschrift (Folgefrage F2) kommt deren Lizenz hinzu.", "",
         "| Paket | Version | Lizenz | Verwendung |", "|---|---|---|---|"]
    for p in pk:
        art = "Bauzeit/Prüfung" if p["dev"] else "Laufzeit"
        if p["optional"]:
            art += " (Plattformpaket)"
        z.append(f"| `{p['name']}` | {p['version']} | {p['license'].replace('|', '/')} | {art} |")
    z += ["", END]
    return "\n".join(z)


def aktualisieren(text, neu):
    if BEGIN in text and END in text:
        a, b = text.index(BEGIN), text.index(END) + len(END)
        return text[:a] + neu + text[b:]
    k = text.index("## 4a. Logo")
    return text[:k] + neu + "\n\n" + text[k:]


def main():
    pk = pakete()
    f = fehler(pk)
    text = DOC.read_text(encoding="utf-8")
    neu_text = aktualisieren(text, abschnitt(pk))
    if "--check" in sys.argv:
        for m in f:
            print(m)
        if neu_text != text:
            f.append("THIRD-PARTY-NOTICES.md ist für ui/ nicht aktuell: python3 tools/ui_notices.py")
            print(f[-1])
        print(f"ui_notices: {len(f)} Fund(e), {len(pk)} Pakete")
        sys.exit(1 if f else 0)
    for m in f:
        print("WARNUNG:", m)
    DOC.write_text(neu_text, encoding="utf-8")
    print(f"{len(pk)} Pakete geschrieben")


if __name__ == "__main__":
    main()
