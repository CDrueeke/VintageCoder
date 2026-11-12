#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Frequenztabelle für den TED-Player (Plus/4): Note 1..96 (C-0 = 1) -> 10-Bit-Wert für $FF0E/$FF12 (Stimme 1) und $FF0F/$FF10 (Stimme 2).
TED: f = Takt/8 / (1024 - N); PAL-Takt 886724 Hz (NTSC 894886 Hz). Noten unter dem tiefsten Ton (N = 0, PAL 108 Hz) werden oktaviert nach oben,
Noten über N = 1021 auf N = 1021 begrenzt.
Aufruf: python3 tools/gen_ted_freq.py [--ntsc] > examples/multi/music_demo/hal/ted_freq.inc"""
import argparse, sys
_ap = argparse.ArgumentParser(
    description="""Frequenztabelle für den TED-Player (Plus/4): Note 1..96 (C-0 = 1) -> 10-Bit-Wert für $FF0E/$FF12 (Stimme 1) und $FF0F/$FF10 (Stimme 2).

TED: f = Takt/8 / (1024 - N); PAL-Takt 886724 Hz (NTSC 894886 Hz). Noten unter dem tiefsten Ton (N = 0, PAL 108 Hz) werden oktaviert nach oben,
Noten über N = 1021 auf N = 1021 begrenzt. Das Werkzeug schreibt den Assembler-Text (ted_freq_lo/ted_freq_hi) auf die Standardausgabe und
selbst keine Datei; die Ausgabe wird mit `>` in eine Datei umgeleitet. Der Arbeitsordner ist egal.""",
    epilog="""Beispiele:
  python3 tools/gen_ted_freq.py > examples/multi/music_demo/hal/ted_freq.inc
  python3 tools/gen_ted_freq.py --ntsc > ted_freq_ntsc.inc""",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_ap.add_argument("--ntsc", action="store_true", help="NTSC-Takt (894886 Hz) statt PAL (886724 Hz) verwenden")
_args = _ap.parse_args()
PAL = 886724 / 8
NTSC = 894886 / 8
def table(base):
    out = []
    for n in range(1, 97):
        f = 440.0 * 2 ** ((n - 58) / 12)
        low = base / 1024
        while f < low: f *= 2
        v = round(1024 - base / f)
        out.append(max(0, min(1021, v)))
    return out
print("; ted_freq.inc – TED-Frequenzwerte (PAL) für Note 1..96, ERZEUGT von tools/gen_ted_freq.py. Töne unter 108 Hz sind oktaviert.")
tab = table(NTSC if _args.ntsc else PAL)
lo = ",".join(str(v & 255) for v in tab); hi = ",".join(str(v >> 8) for v in tab)
print(f"ted_freq_lo:\n        .byte {lo}\nted_freq_hi:\n        .byte {hi}")
