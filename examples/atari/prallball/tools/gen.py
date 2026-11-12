#!/usr/bin/env python3
"""Erzeugt lib/gfx.inc.chars: die Spielfeldzeichen (Codes 96..105) fuer den ANTIC-Modus 4 des Atari.
Je Zeichen 8 Byte, 4 Pixel zu 2 Bit: 00 = COLBK, 01 = COLPF0 (Himmel), 10 = COLPF1 (Stein), 11 = COLPF2 (Lichtkante), bei gesetztem Bit 7 des Zeichencodes COLPF3 (Metall).
Aufruf im Ordner examples/atari/prallball:  python3 tools/gen.py"""
import os
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/atari/prallball (os.chdir); alle Pfade gelten relativ dazu, der Aufruf funktioniert daher aus jedem Ordner.', epilog='Beispiele:\n  python3 examples/atari/prallball/tools/gen.py\n  cd examples/atari/prallball && python3 tools/gen.py\n\nSchreibt (im Projektordner examples/atari/prallball, vorhandene Dateien werden ueberschrieben): lib/gfx.inc.chars.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
def mc(px):
    v = 0
    for p in px: v = (v << 2) | p
    return v
def ch(rows): return [mc(r) for r in rows]
SKY = ch([[1, 1, 1, 1]] * 8)
BRL = ch([[1, 3, 3, 3], [1, 3, 2, 2], [1, 3, 2, 2], [1, 3, 2, 2], [1, 3, 2, 2], [1, 3, 2, 2], [1, 0, 0, 0], [1, 1, 1, 1]])
BRR = ch([[3, 3, 3, 1], [2, 2, 0, 1], [2, 2, 0, 1], [2, 2, 0, 1], [2, 2, 0, 1], [2, 2, 0, 1], [0, 0, 0, 1], [1, 1, 1, 1]])
MTL = ch([[1, 2, 2, 2], [1, 3, 3, 3], [1, 3, 3, 3], [1, 3, 3, 3], [1, 3, 3, 3], [1, 3, 3, 3], [1, 0, 0, 0], [1, 1, 1, 1]])
MTR = ch([[2, 2, 2, 1], [3, 3, 0, 1], [3, 3, 0, 1], [3, 3, 0, 1], [3, 3, 0, 1], [3, 3, 0, 1], [0, 0, 0, 1], [1, 1, 1, 1]])
HDL = ch([[1, 3, 3, 3], [1, 3, 0, 0], [1, 3, 0, 0], [1, 3, 0, 0], [1, 3, 0, 0], [1, 3, 0, 0], [1, 3, 3, 3], [1, 1, 1, 1]])
HDR = ch([[3, 3, 3, 1], [0, 0, 3, 1], [0, 0, 3, 1], [0, 0, 3, 1], [0, 0, 3, 1], [0, 0, 3, 1], [3, 3, 3, 1], [1, 1, 1, 1]])
WAL = ch([[2, 2, 2, 2], [3, 3, 3, 0], [3, 3, 3, 0], [3, 3, 3, 0], [3, 3, 3, 0], [3, 3, 3, 0], [3, 3, 3, 0], [2, 0, 0, 0]])
BLK = ch([[3, 3, 3, 3], [3, 2, 2, 0], [3, 2, 2, 0], [3, 2, 2, 0], [3, 2, 2, 0], [3, 2, 2, 0], [3, 2, 2, 0], [0, 0, 0, 0]])
BLACK = ch([[0, 0, 0, 0]] * 8)
chars = [('Himmel (Pixelwert 01 = COLPF0)', SKY), ('Stein links', BRL), ('Stein rechts', BRR), ('Metall links (+$80: PF3)', MTL), ('Metall rechts', MTR),
         ('unzerstoerbar links', HDL), ('unzerstoerbar rechts', HDR), ('Wand (+$80: PF3)', WAL), ('Block (Titellogo)', BLK), ('schwarz', BLACK)]
out = ['; ERZEUGT von tools/gen.py: Spielfeldzeichen fuer den Modus 4 (Codes 96..105); je Zeichen 8 Byte, 4 Pixel zu 2 Bit: 00 = COLBK, 01 = COLPF0 (Himmel), 10 = COLPF1 (Stein), 11 = COLPF2 (Lichtkante) bzw. COLPF3 mit gesetztem Bit 7 des Zeichencodes', 'gfx_chars:']
for i, (n, c) in enumerate(chars): out.append('        .byte ' + ', '.join('$%02x' % b for b in c) + '      ; %d: %s' % (96 + i, n))
open('lib/gfx.inc.chars', 'w').write('\n'.join(out) + '\n')
print('ok')
