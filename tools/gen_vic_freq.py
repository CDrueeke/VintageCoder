#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Frequenztabellen für den VIC-I-Player (VC-20): Note 1..96 (C-0 = 1) -> Registerwert ($80 | N) für $900A (Bass), $900B (Alt), $900C (Sopran), $900D (Rauschen).
VIC-I (in VICE gemessen, 2026-11-06): f = Takt * m / (256 * (127 - N)), m = 1 (Bass), 2 (Alt), 4 (Sopran); N = 0..126. PAL-Takt 1108405 Hz, NTSC 1022727 Hz.
Töne unter dem tiefsten Ton der Stimme werden oktaviert nach oben, Töne über N = 126 (k = 1) nach unten. Das Rauschen bekommt N = 24 + Note (nur Klangfarbe, keine Tonhöhe).
Aufruf: python3 tools/gen_vic_freq.py > examples/vic20/music_demo/lib/vic_freq.inc"""
import argparse
_ap = argparse.ArgumentParser(
    description=__doc__ + """

Das Werkzeug liest nichts und schreibt den Assembler-Text (PAL- und NTSC-Tabelle, mit .ifdef NTSC wählbar) auf die Standardausgabe, selbst keine Datei;
die Ausgabe wird mit `>` in eine Datei umgeleitet. Der Arbeitsordner ist egal.""",
    epilog="""Beispiele:
  python3 tools/gen_vic_freq.py > examples/vic20/music_demo/lib/vic_freq.inc
  python3 tools/gen_vic_freq.py | head -20

Das Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und gibt keine Tabelle aus.""",
    formatter_class=argparse.RawDescriptionHelpFormatter)
_ap.parse_args()
PAL, NTSC = 1108405, 1022727
def tone(clock, m):
    out = []
    for n in range(1, 97):
        f = 440.0 * 2 ** ((n - 58) / 12)
        lo = clock * m / (256 * 126.0) * 0.999
        while clock * m / (256 * f) > 126.5: f *= 2
        while clock * m / (256 * f) < 0.5: f /= 2
        k = max(1, min(127, round(clock * m / (256 * f))))
        out.append(0x80 | (127 - k))
    return out
def noise():
    return [0x80 | min(126, 24 + n) for n in range(1, 97)]
def emit(clock, ntsc):
    o = []
    for name, tab in (("bass", tone(clock, 1)), ("alto", tone(clock, 2)), ("sopr", tone(clock, 4)), ("nois", noise())):
        o.append(f"mus_f_{name}:")
        for i in range(0, 96, 12): o.append("        .byte " + ",".join(f"${v:02x}" for v in tab[i:i + 12]))
    return o
print("; vic_freq.inc – VIC-I-Registerwerte ($80 | N) für Note 1..96, ERZEUGT von tools/gen_vic_freq.py. Mit .const NTSC = 1 die NTSC-Tabelle.")
print(".ifdef NTSC"); print("\n".join(emit(NTSC, True))); print(".else"); print("\n".join(emit(PAL, False))); print(".endif")
