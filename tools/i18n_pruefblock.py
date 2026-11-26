#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt docs/i18n-pruefblock-oberflaeche.md: Deutsch und Englisch nebeneinander für die Kernbereiche der Oberfläche
(Lesezeit etwa 1–2 Stunden), damit der Inhaber den englischen Erstentwurf prüfen kann. Quelle sind die Kataloge; die Datei wird neu
geschrieben, nie von Hand ergänzt (Korrekturen gehören in crates/vc-i18n/catalog/en/*.ftl).
Aufruf: python3 tools/i18n_pruefblock.py"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CAT = os.path.join(ROOT, "crates", "vc-i18n", "catalog")
KEY = re.compile(r"^([a-z][a-z0-9_]*(?:-[a-z0-9_]+)*)\s*=\s*(.*)$")

# (Bereich, Überschrift, Hinweis)
BEREICHE = [
    ("cmd", "Befehle (Menüs, Befehlspalette)", "Name jedes Befehls; die Titel der Menüs stehen unter „Untermenü-Titel“ und in der Menüleiste („Hauptfenster“)."),
    ("submenu", "Untermenü-Titel", ""),
    ("keys", "Tastennamen", ""),
    ("gui", "Hauptfenster, Menüleiste, Statuszeile, Meldungen", ""),
    ("texte", "Gemeinsame Beschriftungen der Bereiche", ""),
    ("kontext", "Kontextmenüs", ""),
    ("startui", "Startseite", ""),
    ("assistent", "Einrichtungsassistent", ""),
    ("setup", "Assistent: Änderungsliste und Zusammenfassung", ""),
    ("setui", "Einstellungen", ""),
    ("neuprojekt", "Neues Projekt", ""),
    ("projeinst", "Projekt-Einstellungen", ""),
    ("infodlg", "Info-Fenster", ""),
    ("exportdlg", "Export", ""),
    ("importdlg", "Import", ""),
    ("kopierdlg", "Projekt kopieren", ""),
    ("studioaktionen", "Studio-Aktionen (Befehlspalette)", ""),
    ("studios", "Studio-Namen", ""),
    ("update", "Update-Prüfung", ""),
]


def lies(sprache, bereich):
    """Liste (Schlüssel, Wert, Kontext) in Dateireihenfolge; Mehrzeilenwerte mit ⏎ verbunden."""
    pfad = os.path.join(CAT, sprache, bereich + ".ftl")
    if not os.path.exists(pfad):
        return []
    out, kontext, cur = [], "", None
    for z in open(pfad, encoding="utf-8").read().split("\n"):
        m = KEY.match(z)
        if m:
            cur = [m.group(1), m.group(2), kontext]
            out.append(cur)
            kontext = ""
        elif z.startswith("#"):
            kontext = z[1:].strip() if z.startswith("# Kontext:") else ""
            cur = None
        elif z.startswith(" ") and cur is not None:
            cur[1] += " ⏎ " + z.strip()
        elif not z.strip():
            cur = None
    return [tuple(x) for x in out]


def zelle(t):
    return t.replace("|", "\\|").replace("{ $", "{$").strip() or "·"


def main():
    zeilen = ["<!-- SPDX-License-Identifier: GPL-3.0-or-later -->",
              "# Prüfblock Oberfläche: Deutsch und Englisch",
              "",
              "Erzeugt von `tools/i18n_pruefblock.py` aus den Katalogen (nicht von Hand ändern). Die englischen Texte sind ein **Erstentwurf**; maßgeblich für Begriffe ist",
              "[docs/glossar-i18n.md](glossar-i18n.md), für den Stil: einfache Sätze, „you“, Imperativ in Anleitungen. `{$name}` ist ein Platzhalter, der im Programm",
              "durch einen Wert ersetzt wird; `⏎` ist ein Zeilenumbruch im Text. „Zeropage“ heißt auf Englisch „zero page“, „Onion Skin“ bleibt unübersetzt.",
              "",
              "**So prüfen:** je Tabelle links lesen, rechts den englischen Text beurteilen; Korrekturen mit Schlüssel (erste Spalte) notieren oder direkt in",
              "`crates/vc-i18n/catalog/en/<bereich>.ftl` ändern. Länge: {GESAMT} Zeilen, Lesezeit etwa 1–2 Stunden. Die übrigen Bereiche (Studios, Debugger,",
              "Protokolltexte, Bauprotokoll …) stehen in den Katalogen und werden stichprobenartig geprüft.",
              ""]
    gesamt = 0
    inhalt = []
    for b, titel, hinweis in BEREICHE:
        de, en = lies("de", b), lies("en", b)
        eng = {k: v for k, v, _ in en}
        if not de:
            continue
        zeilen += [f"## {titel}", ""]
        if hinweis:
            zeilen += [hinweis, ""]
        zeilen += ["| Schlüssel | Deutsch | Englisch |", "|---|---|---|"]
        for k, v, ctx in de:
            zeilen.append(f"| `{k}` | {zelle(v)} | {zelle(eng.get(k, '(fehlt)'))} |")
            gesamt += 1
        zeilen.append("")
        inhalt.append((titel, len(de)))
    zeilen[10:10] = []
    kopf = zeilen[:11]
    ueb = ["**Inhalt:** " + " · ".join(f"{t} ({n})" for t, n in inhalt), ""]
    ziel = os.path.join(ROOT, "docs", "i18n-pruefblock-oberflaeche.md")
    text = "\n".join(kopf + ueb + zeilen[11:]) + "\n"
    open(ziel, "w", encoding="utf-8").write(text.replace("{GESAMT}", str(gesamt)))
    print(f"{ziel}: {gesamt} Zeilen")


if __name__ == "__main__":
    sys.exit(main())
