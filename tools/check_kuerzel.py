#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Gleicht die Tastenkürzel in Handbuch und Hilfe mit der Befehlstabelle der Oberfläche ab (crates/vc-gui-ui/src/befehle.rs).

    python3 tools/check_kuerzel.py            Abweichungen anzeigen (Exit 1 bei Funden)
    python3 tools/check_kuerzel.py --tabelle  alle Kürzel der Befehlstabelle als Markdown-Tabelle ausgeben

Prüft in beide Richtungen:
  * Jedes Kürzel der Befehlstabelle steht in `docs/handbuch/kapitel/20-kuerzel-faq.md` und in `crates/vc-ide/help/shortcuts.md`.
  * Jedes Kürzel, das Handbuch oder Hilfe nennen (Cmd/Strg-Kombinationen, F-Tasten mit Alt/Umschalt), kommt in der Befehlstabelle vor
    oder steht in der kurzen Liste fester Editor- und Studio-Tasten unten.
  * Menüwege im Text („Bearbeiten ▸ Markdown ▸ Fett“) gibt es in der Menüleiste: Menü, Untermenü und Eintrag müssen in der Befehlstabelle stehen.
Kürzel werden normalisiert: Strg = Cmd, „Cmd/Strg+“ = „Cmd+“, → = Rechts, ← = Links, „Eingabe“ = Enter, „,“ = Komma."""
import pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BEFEHLE = ROOT / "crates" / "vc-gui-ui" / "src" / "befehle.rs"
VOLL = [ROOT / "docs" / "handbuch" / "kapitel" / "20-kuerzel-faq.md", ROOT / "crates" / "vc-ide" / "help" / "shortcuts.md"]
GESAMT = sorted((ROOT / "docs" / "handbuch" / "kapitel").glob("*.md")) + sorted((ROOT / "crates" / "vc-ide" / "help").glob("*.md"))
# feste Tasten, die nicht in der Befehlstabelle stehen (Editor, Studios, Suchleiste, Tracker): dürfen im Text vorkommen
FEST = {"Cmd+A", "Cmd+C", "Cmd+X", "Cmd+V", "Cmd+Umschalt+Z", "Cmd+Y", "Cmd+D", "Cmd+Rechts", "Cmd+Links", "Cmd+Oben", "Cmd+Unten", "Cmd+Alt+Oben", "Cmd+Alt+Unten", "Cmd+Mausrad", "Cmd+Klick", "Umschalt+F10",
        "Cmd+Umschalt+Klick", "Cmd+Alt+Klick", "Cmd+Enter", "Cmd+Umschalt+Enter", "Cmd+Umschalt+Rechts", "Cmd+Umschalt+Links", "Cmd+Backspace", "Cmd+Entf", "Cmd+Home", "Cmd+End", "Cmd+Leertaste"}


def norm(s):
    s = s.replace(" ", "").replace("Ctrl", "Strg").replace("→", "Rechts").replace("←", "Links").replace("↑", "Oben").replace("↓", "Unten")
    s = s.replace("Eingabe", "Enter").replace("Strg/Cmd", "Cmd").replace("Cmd/Strg", "Cmd").replace("Strg", "Cmd").replace("Cmd/Cmd", "Cmd")
    s = re.sub(r"\+,$", "+Komma", s)
    return s


def tabelle():
    out = []
    for m in re.finditer(r'i\(Cmd::(\w+), "([^"]+)", "([^"]*)", "((?:[^"\\]|\\.)*)", "([^"]*)"', BEFEHLE.read_text(encoding="utf-8")):
        cmd, id_, menue, name, tasten = m.groups()
        if tasten:
            out.append((tasten, name, menue))
    return out


KEY_RX = re.compile(r"(?:(?:Cmd|Strg|Ctrl)(?:/(?:Strg|Cmd|Ctrl))?(?:\s?\+\s?(?:Alt|Umschalt|Shift))*\s?\+\s?(?:[A-Za-z0-9ÄÖÜ,.\\](?![A-Za-zäöüß])|Plus|Minus|Komma|Enter|Eingabe|Leertaste|Mausrad|Klick|Rechts|Links|Oben|Unten|[→←↑↓]|F\d{1,2}|Entf|Backspace|Home|End))"
                    r"|(?:(?:Alt|Umschalt)\+)+F\d{1,2}")


def menuepfade():
    """Alle gültigen Menüwege aus der Befehlstabelle: Mengen von Tupeln (Menü, Eintrag) und (Menü, Untermenü, Eintrag) sowie (Menü, Untermenü)."""
    t = BEFEHLE.read_text(encoding="utf-8")
    menues = re.search(r'pub const MENUES: &\[&str\] = &\[([^\]]*)\]', t).group(1)
    menues = re.findall(r'"([^"]+)"', menues)
    unter = {}
    for m in re.finditer(r'((?:[A-Z]\w+\s*\|?\s*)+)=> "([^"]+)",', t[t.index("pub fn untermenue"):]):
        for c in re.findall(r"[A-Z]\w+", m.group(1)): unter[c] = m.group(2)
    pfade = set()
    for m in re.finditer(r'i\(Cmd::(\w+), "([^"]+)", "([^"]*)", "((?:[^"\\]|\\.)*)", "([^"]*)"', t):
        cmd, _, menue, name, _ = m.groups()
        if menue not in menues: continue
        u = unter.get(cmd, "")
        kurz = name[len(u) + 2:] if u and name.startswith(u + ": ") else name
        for n in {name, kurz}:
            pfade.add((menue, u, n) if u else (menue, n))
        if u: pfade.add((menue, u))
    return menues, pfade


def pruefe_pfade(funde):
    menues, pfade = menuepfade()
    rx = re.compile(r"\b(" + "|".join(menues) + r")((?: ▸ [^▸*|`;)\n]+?)+)(?=\*\*|\)|\||`|;|,|\.(?:\s|$)|$)")
    def norm_n(x): return re.sub(r"\s*\(.*$", "", x.strip().lstrip("„").split("“")[0]).replace(" …", "").replace("…", "").strip()
    gueltig = {tuple(norm_n(x) for x in p) for p in pfade}
    for p in GESAMT:
        for nr, zeile in enumerate(p.read_text(encoding="utf-8").splitlines(), 1):
            for m in rx.finditer(zeile):
                teile = [m.group(1)] + [norm_n(x) for x in m.group(2).split("▸")[1:]]
                teile = [t for t in teile if t]
                if len(teile) < 2 or teile[1] in ("", ".", "Markdown:") : continue
                # Platzhalter („Markdown ▸ …“) und Texte hinter dem Eintrag sind kein Fehler: es zählt der längste gültige Anfang
                if any(tuple(teile[:n]) in gueltig for n in (3, 2)): continue
                funde.append(f"{p.relative_to(ROOT)}:{nr}: Menüweg {' ▸ '.join(teile)} gibt es nicht")


def main(argv):
    tab = tabelle()
    if "--tabelle" in argv:
        print("| Tastenkürzel | Befehl | Menü |\n|---|---|---|")
        for t, n, m in tab: print(f"| {t} | {n} | {m} |")
        return 0
    gueltig = {norm(t) for t, _, _ in tab} | FEST
    funde = []
    for p in VOLL:
        text = norm(p.read_text(encoding="utf-8"))
        for t, n, _ in tab:
            if norm(t) not in text and not any(norm(t) in norm(l) for l in p.read_text(encoding="utf-8").splitlines()):
                funde.append(f"{p.relative_to(ROOT)}: Kürzel {t} ({n}) fehlt")
    for p in GESAMT:
        for nr, zeile in enumerate(p.read_text(encoding="utf-8").splitlines(), 1):
            for m in KEY_RX.finditer(zeile):
                k = norm(m.group(0))
                if k not in gueltig and not re.fullmatch(r"F\d{1,2}", k):
                    funde.append(f"{p.relative_to(ROOT)}:{nr}: unbekanntes Kürzel {m.group(0)}")
    pruefe_pfade(funde)
    for f in funde: print(f)
    print(f"{len(funde)} Fund(e)")
    return 1 if funde else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
