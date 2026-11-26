#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Pakete der neuen Oberfläche (vc-gui, vc-gui-ui, vc-gui-editor) samt Lizenz in THIRD-PARTY-NOTICES.md nachführen.

    python3 tools/gui_notices.py            # Abschnitt neu schreiben
    python3 tools/gui_notices.py --check    # nur prüfen, ob er aktuell ist (Exit 1 sonst)

Quelle ist `cargo metadata` (macOS, Windows, Linux, ohne Dev-Abhängigkeiten); der Abschnitt steht zwischen den Marken GUI-PAKETE:BEGIN/END."""
import json, subprocess, sys, pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
DOC = ROOT / "THIRD-PARTY-NOTICES.md"
GUI = ["vc-gui", "vc-gui-ui", "vc-gui-editor", "vc-i18n", "vc-shell", "vc-commands"]
PLATTFORMEN = ["aarch64-apple-darwin", "x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"]  # Zielsysteme der Ausgaben (macOS, Windows, Linux)
BEGIN, END = "<!-- GUI-PAKETE:BEGIN -->", "<!-- GUI-PAKETE:END -->"

def metadata():
    out = subprocess.run(["cargo", "metadata", "--format-version", "1", "--locked"] + [a for t in PLATTFORMEN for a in ("--filter-platform", t)], cwd=ROOT, check=True, capture_output=True, text=True).stdout
    return json.loads(out)

def pakete(md):
    """(Name, Version, Lizenz, Quelle) aller fremden Pakete, von denen die GUI-Crates (ohne Dev-Abhängigkeiten) abhängen; dazu die direkten Abhängigkeiten."""
    by_id = {p["id"]: p for p in md["packages"]}; members = {p["id"] for p in md["packages"] if p["id"] in set(md["workspace_members"])}
    nodes = {n["id"]: n for n in md["resolve"]["nodes"]}
    start = [i for i in members if by_id[i]["name"] in GUI]; seen = set(); stack = list(start)
    while stack:
        i = stack.pop()
        if i in seen: continue
        seen.add(i)
        for d in nodes[i]["deps"]:
            if any(k.get("kind") in (None, "build") for k in d["dep_kinds"]): stack.append(d["pkg"])
    fremd = sorted((by_id[i] for i in seen if i not in members), key=lambda p: (p["name"], p["version"]))
    direkt = sorted({d["name"] for i in start for d in by_id[i]["dependencies"] if d.get("kind") in (None, "build") and d.get("path") is None and not d.get("optional")})
    return fremd, direkt

def abschnitt(fremd, direkt):
    zeilen = [BEGIN, "", "### Pakete der neuen Oberfläche (`vc-gui`, `vc-gui-ui`, `vc-gui-editor`, `vc-i18n`)", "",
        f"Erzeugt mit `python3 tools/gui_notices.py` aus `cargo metadata` (macOS, Windows, Linux, ohne Dev-Abhängigkeiten): **{len(fremd)} Pakete** außer den eigenen. Direkte Abhängigkeiten der drei Crates: {', '.join('`' + n + '`' for n in direkt)}. Die Liste ist ein Auszug des Gesamtbestands oben und wird bei jeder Änderung der Abhängigkeiten neu erzeugt (Prüfung: `python3 -m unittest tools.test_gui_notices`).", "",
        "| Paket | Version | Lizenz | Quelle |", "|---|---|---|---|"]
    for p in fremd:
        lic = (p.get("license") or p.get("license_file") or "UNBEKANNT").replace("|", "/"); repo = p.get("repository") or p.get("homepage") or f"https://crates.io/crates/{p['name']}"
        zeilen.append(f"| `{p['name']}` | {p['version']} | {lic} | {repo} |")
    zeilen += ["", END]
    return "\n".join(zeilen)

def aktualisieren(text, neu):
    if BEGIN in text and END in text:
        a, b = text.index(BEGIN), text.index(END) + len(END); return text[:a] + neu + text[b:]
    k = text.index("## 4a. Logo")  # vor dem nächsten Hauptabschnitt einfügen
    return text[:k] + neu + "\n\n" + text[k:]

def main():
    fremd, direkt = pakete(metadata()); neu = abschnitt(fremd, direkt); text = DOC.read_text(encoding="utf-8"); neu_text = aktualisieren(text, neu)
    if "--check" in sys.argv:
        if neu_text != text: print("THIRD-PARTY-NOTICES.md ist nicht aktuell: python3 tools/gui_notices.py"); sys.exit(1)
        print(f"ok ({len(fremd)} Pakete)"); return
    DOC.write_text(neu_text, encoding="utf-8"); print(f"{len(fremd)} Pakete geschrieben")

if __name__ == "__main__": main()
