#!/usr/bin/env python3
"""Erzeugt lib/gradient.inc: die 16 Palettenfarben und die Hintergrundverlaeufe (VIC-IV-Palette) fuer PRALLBALL auf dem MEGA65.
Aufruf im Ordner examples/mega65/prallball:  python3 tools/gen_gradient.py

Palette: Der VIC-IV hat 256 Farben mit je 8 Bit Rot, Gruen, Blau; im VIC-II-Modus sind die Eintraege 0-15 sichtbar.  Die Register
$D100/$D200/$D300 nehmen die Werte MIT VERTAUSCHTEN HALBBYTES (aus $AB wird $BA, gemessen in Xemu).
Verlaeufe: NB Baender zu je 4 Rasterzeilen; jedes Band setzt den Palettenteintrag 0 (Hintergrund) neu, siehe irq in prallball.asm."""
import os
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: egal; die Ausgabedatei wird relativ zum Ort des Skripts (../lib) geschrieben.', epilog='Beispiele:\n  python3 examples/mega65/prallball/tools/gen_gradient.py\n  cd examples/mega65/prallball && python3 tools/gen_gradient.py\n\nSchreibt (im Projektordner examples/mega65/prallball, vorhandene Dateien werden ueberschrieben): lib/gradient.inc.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
NB = 50                      # Baender je Bild (Index NB = Bildende)
FIRST, LAST = 4, 46          # Baender mit Farbe (Rasterzeile 68 bis 239); davor und danach schwarz (Kopf- und Fusszeile)

PAL = [(0, 0, 0), (255, 255, 255), (224, 40, 40), (64, 224, 224), (200, 64, 200), (48, 200, 64), (48, 64, 224), (255, 224, 48),
       (255, 144, 32), (144, 80, 24), (255, 120, 120), (72, 72, 80), (120, 120, 128), (144, 255, 144), (128, 160, 255), (200, 200, 208)]

def sw(v): v &= 255; return ((v << 4) | (v >> 4)) & 255
def mix(a, b, t): return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))
def ramp(stops, n):          # stops: Liste (Farbe); n Stufen gleichmaessig dazwischen
    out = []
    for i in range(n):
        p = i / (n - 1) * (len(stops) - 1)
        k = min(int(p), len(stops) - 2)
        out.append(mix(stops[k], stops[k + 1], p - k))
    return out
n = LAST - FIRST + 1
themes = [
    ramp([(16, 24, 110), (130, 40, 160), (235, 70, 60)], n),             # 0 Daemmerung
    ramp([(60, 72, 90), (30, 120, 60), (120, 76, 30)], n),                # 1 Wald
    ramp([(80, 36, 18), (205, 30, 20), (255, 150, 30)], n),               # 2 Glut
    [((40, 44, 64) if (i // 2) % 2 == 0 else (6, 6, 14)) for i in range(n)],  # 3 Nacht (Streifen zu je 8 Zeilen)
]
out = ['; ERZEUGT von tools/gen_gradient.py: Palette (VIC-IV, Halbbytes vertauscht) und Hintergrundverlaeufe.  NB = %d Baender zu 4 Zeilen.' % NB,
       '.const NB = %d' % NB, 'pal_r:  .byte ' + ', '.join('$%02x' % sw(c[0]) for c in PAL),
       'pal_g:  .byte ' + ', '.join('$%02x' % sw(c[1]) for c in PAL), 'pal_b:  .byte ' + ', '.join('$%02x' % sw(c[2]) for c in PAL),
       'bar_line: .byte ' + ', '.join(str(51 + 4 * k) for k in range(NB)) + ', 251',
       '; Je Verlauf (NB + 1) * 3 Byte: Rot, Gruen, Blau je Band; Index NB = Bildende (schwarz).', 'grad_themes:']
for t, th in enumerate(themes):
    out.append('; Verlauf %d' % t)
    for k in range(NB + 1):
        c = th[k - FIRST] if FIRST <= k <= LAST else (0, 0, 0)
        out.append('        .byte $%02x, $%02x, $%02x' % (sw(c[0]), sw(c[1]), sw(c[2])))
with open(os.path.join(os.path.dirname(__file__), '..', 'lib', 'gradient.inc'), 'w', newline='\n') as f: f.write('\n'.join(out) + '\n')
print('lib/gradient.inc geschrieben')
