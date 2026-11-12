#!/usr/bin/env python3
"""Erzeugt die Levelentwuerfe 4-24 und schreibt levels/levels.txt (Level 1-3 und der Kommentarkopf bleiben unveraendert aus levels/levels_1_3.txt;
fehlt diese Datei, wird sie beim ersten Lauf aus dem Anfang der vorhandenen levels.txt abgeschnitten).
Die Muster entstehen aus kleinen Funktionen; das Ergebnis (levels.txt) ist die einzige Quelle fuer tools/gen.py und darf von Hand
nachbearbeitet werden (dann make_levels.py nicht mehr laufen lassen, es wuerde die Handarbeit ueberschreiben).
Zeichen im Levelentwurf: siehe den Kopf von levels/levels.txt.  Aufruf im Ordner examples/c64/prallball: python3 tools/make_levels.py"""
import os, math
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/c64/prallball (os.chdir); alle Pfade gelten relativ dazu, der Aufruf funktioniert daher aus jedem Ordner.', epilog='Beispiele:\n  python3 examples/c64/prallball/tools/make_levels.py\n  cd examples/c64/prallball && python3 tools/make_levels.py\n\nSchreibt (im Projektordner examples/c64/prallball, vorhandene Dateien werden ueberschrieben): levels/levels_1_3.txt, levels/levels.txt.\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
W, H = 18, 12                                          # Steinfeld: 18 Spalten x 12 Zeilen
COL = 'abcdefgh'                                       # die acht Steinfarben (Typ 1-8)
def grid(f):
    """Baut ein Level aus einer Funktion f(x, y) -> Zeichen (x = Spalte 0..17, y = Zeile 0..11)."""
    return [''.join(f(x, y) for x in range(W)) for y in range(H)]
def col(i): return COL[int(i) % 8]               # Farbe i (im Kreis ueber die acht Farben)
L = []                                                 # die Entwuerfe in der Reihenfolge der Levels 4..24
def add(name, rows): L.append((name, rows))
# Levels 4-24; 'Bonus ...' (Level 11 und 22) sind Bonuslevel, 'Boss ...' (Level 23 und 24) Bosslevel (gen.py erkennt es am Namensanfang)
add('Schachbrett', grid(lambda x, y: ('a' if (x + y) % 2 else 'e') if 1 <= y <= 8 else '.'))
add('Streifen', grid(lambda x, y: col(x // 2) if 1 <= y <= 8 else '.'))
add('Diagonale', grid(lambda x, y: col((x + y) // 2) if 1 <= y <= 9 and (x + y) % 3 != 0 else '.'))
add('Raute', grid(lambda x, y: col(abs(x - 8.5) // 1 + abs(y - 5)) if abs(x - 8.5) + 2 * abs(y - 5) <= 9 else '.'))
add('Rahmen', grid(lambda x, y: col(min(x, W - 1 - x, y - 1, 9 - y)) if 1 <= y <= 9 and (min(x, W - 1 - x, y - 1, 9 - y)) % 2 == 0 else '.'))
add('Zickzack', grid(lambda x, y: col(y) if y in (1, 3, 5, 7) and (x // 3 + y // 2) % 2 == 0 else ('s' if y in (2, 4, 6, 8) and x % 6 == 0 else '.')))
add('Mauer', grid(lambda x, y: ('#' if (y % 4 == 3 and x % 6 in (2, 3)) else col(y)) if 1 <= y <= 9 else '.'))
add('Bonus 1: Sternenhimmel', grid(lambda x, y: 'p' if 1 <= y <= 9 and (x * 7 + y * 5) % 6 == 0 else '.'))
add('Treppe', grid(lambda x, y: col(y) if 1 <= y <= 9 and x >= y * 2 - 2 else '.'))
add('Wellen', grid(lambda x, y: col(x // 2) if abs(y - (5 + 3 * math.sin(x / 2.2))) < 1.2 else '.'))
add('Kaefig', grid(lambda x, y: '#' if (x in (3, 14) and 2 <= y <= 9) or (y in (2, 9) and 3 <= x <= 14) else ('G' if 4 <= x <= 13 and 3 <= y <= 8 and (x + y) % 2 == 0 else ('c' if 4 <= x <= 13 and 3 <= y <= 8 else '.'))))
add('Spirale', grid(lambda x, y: col(int(math.hypot(x - 8.5, (y - 5) * 2) // 2)) if 3 <= math.hypot(x - 8.5, (y - 5) * 2) <= 9 and 0 <= y <= 10 else '.'))
add('Dreiecke', grid(lambda x, y: col(x // 6 + y) if 1 <= y <= 9 and (x % 6) >= abs(y % 6 - 3) and (x % 6) < 6 - abs(y % 6 - 3) else '.'))
add('Gitter', grid(lambda x, y: ('s' if x % 3 == 0 and 1 <= y <= 9 else 'd' if y % 2 == 1 and 1 <= y <= 9 else '.')))
add('Sanduhr', grid(lambda x, y: col(y) if 1 <= y <= 10 and abs(x - 8.5) >= abs(y - 5.5) - 0.5 and abs(x - 8.5) <= 8 else '.'))
add('Kreuz', grid(lambda x, y: ('G' if (x in (8, 9) and 1 <= y <= 10) or (y in (5, 6) and 3 <= x <= 14) else (col(x + y) if 1 <= y <= 10 and (x in (7, 10) or y in (4, 7)) else '.'))))
add('Zinnen', grid(lambda x, y: ('#' if y == 9 and x % 4 == 1 else col(y) if 1 <= y <= 7 and (x // 2) % 2 == 0 else 's' if y == 8 else '.')))
add('Silberstreif', grid(lambda x, y: ('s' if y % 2 == 1 else col(x)) if 1 <= y <= 9 else '.'))
add('Bonus 2: Schatzkiste', grid(lambda x, y: ('G' if 2 <= y <= 4 else 'p' if y in (5, 7) and x % 2 == 0 else 's' if y == 6 else '.') if 3 <= x <= 14 else '.'))
add('Boss 1: Waechter', grid(lambda x, y: '#' if (x in (4, 13) and 1 <= y <= 9) or (y == 1 and 4 <= x <= 13) else ('C' if 6 <= x <= 11 and 3 <= y <= 7 else ('s' if 5 <= x <= 12 and 2 <= y <= 8 else '.'))))
add('Boss 2: Festung', grid(lambda x, y: '#' if (y == 1 and x % 3 != 1) or (x in (1, 16) and 1 <= y <= 9) or (y == 9 and x % 4 == 1) else ('C' if 5 <= x <= 12 and 3 <= y <= 6 else ('s' if 2 <= x <= 15 and 2 <= y <= 8 else '.'))))
assert len(L) == 21, len(L)                            # Levels 4..24
head = open('levels/levels_1_3.txt', encoding='utf-8').read().rstrip('\n') if os.path.exists('levels/levels_1_3.txt') else None
if head is None:
    src = open('levels/levels.txt', encoding='utf-8').read()
    head = src[:src.index('level 4 ')].rstrip('\n') if 'level 4 ' in src else src.rstrip('\n')
    open('levels/levels_1_3.txt', 'w', encoding='utf-8').write(head + '\n')
out = [head, '']
for i, (name, rows) in enumerate(L):
    out.append('level %d "%s"' % (i + 4, name)); out += [r[:W] for r in rows]; out.append('')
open('levels/levels.txt', 'w', encoding='utf-8').write('\n'.join(out))
print('levels.txt:', 3 + len(L), 'Levels')
