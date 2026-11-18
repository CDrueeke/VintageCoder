#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Handbuchbilder aus den Rohbildern der Oberfläche zusammenstellen (nur macOS, nutzt `sips`).

Ablauf:
  1. `cargo test -p vc-gui handbuch_bilder -- --ignored` schreibt die Rohbilder nach `build/handbuch/` (1600×1007, dunkel,
     Beispielprojekt „Hugo & Gumbo“ bzw. ein anderes Beispiel, ohne Marken, Projektordner unter /tmp/vc-handbuch).
  2. Dieses Skript schneidet Ausschnitte zu (Spalten des Assistenten, Debug-Spalten, Zeile mit Haltepunkt …) und legt jedes
     Bild unter `docs/handbuch/bilder/<kennung>.png` ab. Ausschnitte lassen Pfade des Rechners weg (z. B. bei den ROM-Seiten).

Aufruf: python3 tools/handbuchbilder.py [kennung …]   (ohne Angabe: alle)
"""
import shutil, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ROH = ROOT / "build" / "handbuch"
ZIEL = ROOT / "docs" / "handbuch" / "bilder"

# Kennung -> (Rohbild, x, y, Breite, Höhe) in Pixeln des 1600×1007-Rohbilds
AUSSCHNITTE = {
    "wizard-system": ("assistent-neues-projekt", 26, 88, 388, 430),
    "wizard-sprache": ("assistent-neues-projekt", 424, 88, 560, 330),
    "wizard-details": ("assistent-neues-projekt", 990, 88, 560, 830),
    "debug-register": ("debug-ueberblick", 262, 676, 198, 190),
    "debug-disassembler": ("debug-ueberblick", 458, 676, 585, 170),
    "debug-speicher": ("debug-ueberblick", 1036, 676, 346, 214),
    "debug-beobachten": ("debug-ueberblick", 1378, 676, 190, 200),
    "debug-haltepunkt-gutter": ("debug-ueberblick", 262, 330, 498, 80),
    "debug-pc-zeile": ("debug-ueberblick", 262, 330, 498, 80),
    "memory-map": ("memory-map", 1180, 56, 380, 940),
    "einstellungen-roms": ("einstellungen-roms", 270, 415, 1000, 362),
    "rom-verzeichnis": ("einstellungen-roms", 270, 415, 1000, 362),
    "palette-atari": ("sprite-atari-pmg", 1276, 294, 280, 470),
}
# Kennung -> Rohbild, wenn der Name abweicht (ganzes Bild)
GANZ = {"segmente-baenke": "memory-map", "asm-erstes-programm": "projekt-neu-ergebnis", "asm-fehler": "probleme-panel"}
# Rohbilder, die nur Quelle für Ausschnitte sind
NUR_QUELLE = {"assistent-neues-projekt"}


def sips(*a):
    subprocess.run(["sips", *map(str, a)], check=True, stdout=subprocess.DEVNULL)


def main(wahl):
    ZIEL.mkdir(parents=True, exist_ok=True)
    fertig = []
    for roh in sorted(ROH.glob("*.png")):
        k = roh.stem
        if k in NUR_QUELLE or k.startswith("web-") or (wahl and k not in wahl):
            continue
        shutil.copyfile(roh, ZIEL / f"{k}.png"); fertig.append(k)
    for k, q in GANZ.items():
        if wahl and k not in wahl: continue
        shutil.copyfile(ROH / f"{q}.png", ZIEL / f"{k}.png"); fertig.append(k)
    for k, (q, x, y, b, h) in AUSSCHNITTE.items():
        if wahl and k not in wahl: continue
        sips("--cropToHeightWidth", h, b, "--cropOffset", y, x, ROH / f"{q}.png", "--out", ZIEL / f"{k}.png"); fertig.append(k)
    print(len(fertig), "Bilder:", " ".join(sorted(fertig)))


if __name__ == "__main__":
    main(set(sys.argv[1:]))
