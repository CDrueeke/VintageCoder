#!/usr/bin/env python3
"""Komponiert die Titelmusik von PRALLBALL (d-Moll, 125 BPM, 80 Takte, drei Stimmen) und schreibt lib/music.inc.
Aufbau (Konzept, Abschnitt 5): Intro 4, A 16, B 16, A 16, Bruecke 8, A' 16, Outro 4 Takte = 80 Takte, etwa 2:34.
Stimmen: 1 Melodie (Puls), 2 Bass (Saege), 3 Schlagzeug (Bassdrum, Snare, Hi-Hat, Toms) bzw. Arpeggio in den ruhigen Teilen.
Die Melodie besteht aus einem Hauptmotiv (Hook) von vier Takten ueber Dm - Bb - F - C, das in Frage-und-Antwort-Form wiederkehrt.
Deterministisch (kein Zufall).  Ausgabe im Liedformat von examples/c64/prallball/lib/sound.asm (die Atari-Spieler examples/atari/prallball/lib/sound.asm liest dasselbe Format).
Aufruf im Ordner examples/atari/prallball:  python3 tools/compose.py"""
import os
import argparse
_cli = argparse.ArgumentParser(description=(__doc__ or "").strip() + "\n\n" + 'Arbeitsordner: Das Skript wechselt beim Start selbst in den Projektordner examples/atari/prallball (os.chdir); alle Pfade gelten relativ dazu, der Aufruf funktioniert daher aus jedem Ordner.', epilog='Beispiele:\n  python3 examples/atari/prallball/tools/compose.py\n  cd examples/atari/prallball && python3 tools/compose.py\n\nSchreibt (im Projektordner examples/atari/prallball, vorhandene Dateien werden ueberschrieben): lib/music.inc.\nHinweis Windows: Das Skript schreibt ohne ausdrueckliche Kodierung; bei Umlauten in den Ausgaben vorher PYTHONUTF8=1 setzen (Windows: set PYTHONUTF8=1).\n\nDas Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.',
                               formatter_class=argparse.RawDescriptionHelpFormatter)
_cli.parse_args()   # nur -h/--help; steht vor dem Wechsel des Arbeitsordners und vor jedem Dateizugriff
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
HOLD, REST = 0, 96
NOTE = {'C': 0, 'D': 2, 'E': 4, 'F': 5, 'G': 7, 'A': 9, 'B': 11}
def midi(name):
    base = NOTE[name[0]]; i = 1
    if len(name) > i and name[i] == '#': base += 1; i += 1
    elif len(name) > i and name[i] == 'b': base -= 1; i += 1
    return 12 * (int(name[i:]) + 1) + base
def idx(m): return m - 12 + 1                          # Notenindex im Liedformat: C0 (midi 12) = 1
def bar_from(events, lift=0):
    """events: Liste (Zeile, 'D5') oder (Zeile, None) = Pause."""
    p = [HOLD] * 16
    for row, nm in events: p[row] = REST if nm is None else idx(midi(nm) + lift)
    return p
# ---- Chords
CH = {'Dm': ('D', 'm'), 'Bb': ('Bb', 'M'), 'F': ('F', 'M'), 'C': ('C', 'M'), 'Gm': ('G', 'm'), 'A': ('A', 'M')}
def chord_tones(ch, octave):
    r = midi(CH[ch][0] + str(octave)); third = 3 if CH[ch][1] == 'm' else 4
    return [r, r + third, r + 7, r + 12]
# ---- Hauptmotiv: je Akkord ein Takt, Rhythmus 3-3-2-2-2-4 (Synkopen)
HOOK1 = {'Dm': [(0, 'A4'), (3, 'D5'), (6, 'F5'), (8, 'E5'), (10, 'D5'), (12, 'A4')],
         'Bb': [(0, 'D5'), (3, 'Bb4'), (6, 'D5'), (8, 'C5'), (10, 'Bb4'), (12, 'A4')],
         'F':  [(0, 'C5'), (3, 'F5'), (6, 'A5'), (8, 'G5'), (10, 'F5'), (12, 'C5')],
         'C':  [(0, 'E5'), (3, 'G5'), (6, 'E5'), (8, 'D5'), (12, 'C5')]}
HOOK2 = {'Dm': HOOK1['Dm'],
         'Bb': [(0, 'D5'), (3, 'F5'), (6, 'D5'), (8, 'C5'), (10, 'Bb4'), (12, 'D5')],
         'Gm': [(0, 'Bb4'), (3, 'D5'), (6, 'G5'), (8, 'F5'), (10, 'D5'), (12, 'Bb4')],
         'A':  [(0, 'E5'), (3, 'C#5'), (6, 'E5'), (8, 'A5'), (10, 'G5'), (12, 'E5')]}
def eighths(ch, order, octave=4, lift=0):
    t = chord_tones(ch, octave)
    p = [HOLD] * 16
    for i, k in enumerate(order): p[i * 2] = idx(t[k] + lift)
    return p
def halfnotes(ch, a, b, octave=5):
    t = chord_tones(ch, octave); p = [HOLD] * 16
    p[0] = idx(t[a]); p[8] = idx(t[b]); return p
# ---- Bass
def bass(ch, style, root_oct=2):
    r = midi(CH[ch][0] + str(root_oct)); p = [HOLD] * 16
    if style == 'groove':     # synkopierte Achtel/Sechzehntel mit Oktavspruengen
        for row, off in ((0, 0), (2, 0), (3, 12), (6, 7), (8, 0), (10, 0), (11, 12), (14, 7), (15, 5)): p[row] = idx(r + off)
    elif style == 'drive':    # durchgehende Achtel
        for i, row in enumerate(range(0, 16, 2)): p[row] = idx(r + (0, 0, 12, 0, 7, 0, 12, 7)[i])
    elif style == 'half':
        p[0] = idx(r); p[8] = idx(r + 7)
    elif style == 'whole':
        p[0] = idx(r)
    return p
# ---- Schlagzeug (Stimme 3); Zahlen = 128 + Schlag
KICK, SNARE, HAT, OHAT, TOMH, TOML = 128, 129, 130, 131, 132, 133
def drums(kind, fill=False):
    p = [HOLD] * 16
    if kind == 'A':
        for r in (0, 8, 10): p[r] = KICK
        for r in (4, 12): p[r] = SNARE
        for r in (2, 6, 14): p[r] = HAT
        p[7] = OHAT
    elif kind == 'B':        # dichter: Hi-Hat auf jedem Achtel, Bassdrum auf jedem Schlag
        for r in (0, 4, 8, 12): p[r] = KICK
        for r in (4, 12): p[r] = SNARE
        for r in (2, 6, 10, 14): p[r] = HAT
        p[15] = HAT; p[7] = OHAT
    if fill:
        p[12] = SNARE; p[13] = TOMH; p[14] = TOML; p[15] = TOML
    return p
def arp(ch, up=True):
    t = chord_tones(ch, 4); seq = t if up else t[::-1]
    return [idx(seq[i % 4]) for i in range(16)]
def song():
    P = []
    silent = [HOLD] * 16
    def bar(m, b, d): P.append((m, b, d))
    # Intro (4): Arpeggio auf Stimme 3, ab Takt 3 der Bass
    for i, ch in enumerate(('Dm', 'Bb', 'F', 'C')): bar(silent if i < 2 else halfnotes(ch, 2, 3, 4), bass(ch, 'whole') if i >= 2 else silent, arp(ch))
    A1 = ['Dm', 'Bb', 'F', 'C']; A2 = ['Dm', 'Bb', 'Gm', 'A']
    def sectionA(octave_lift=0, drum='A'):
        for k, (prog, hook) in enumerate(((A1, HOOK1), (A2, HOOK2), (A1, HOOK1), (A2, HOOK2))):
            for i, ch in enumerate(prog):
                lift = octave_lift if k >= 2 else 0
                bar(bar_from(hook[ch], lift), bass(ch, 'groove'), drums(drum, fill=(i == 3)))
    sectionA()
    # B (16): fliessende Achtel ueber Gm - Dm - Bb - A (zweimal) und eine Steigerung
    B = ['Gm', 'Dm', 'Bb', 'A', 'Gm', 'Dm', 'Bb', 'A', 'Gm', 'Bb', 'F', 'A', 'Gm', 'Bb', 'A', 'A']
    for i, ch in enumerate(B):
        order = (2, 1, 0, 1, 2, 3, 2, 1) if (i // 2) % 2 == 0 else (0, 2, 1, 2, 3, 2, 1, 0)
        bar(eighths(ch, order, 4, 12 if i >= 8 else 0), bass(ch, 'drive'), drums('B', fill=(i % 4 == 3)))
    sectionA()
    # Bruecke (8): ruhig, lange Noten in der Melodie, Bass halbe Noten, Arpeggio statt Schlagzeug
    for i, ch in enumerate(('Bb', 'F', 'C', 'Gm', 'Bb', 'F', 'A', 'A')):
        bar(halfnotes(ch, 2, 3, 5), bass(ch, 'half'), arp(ch, up=(i % 2 == 0)))
    sectionA(12, 'B')       # A' : Melodie in der zweiten Haelfte eine Oktave hoeher, dichteres Schlagzeug
    # Outro (4)
    for i, ch in enumerate(('Dm', 'Bb', 'A', 'Dm')):
        bar(halfnotes(ch, 0, 2, 5) if i < 3 else bar_from([(0, 'D5')]), bass(ch, 'whole'), arp(ch) if i < 3 else silent)
    return P
P = song()
assert len(P) == 80, len(P)
pats = []; order = []
def pid(p):
    t = tuple(p)
    if t not in pats: pats.append(t)
    return pats.index(t)
for m, b, a in P: order.append((pid(m), pid(b), pid(a)))
assert len(pats) < 256
FPR = 6
hdr_len = 5 + len(order) * 3
out = ['; ERZEUGT von tools/compose.py: Titelmusik PRALLBALL (d-Moll, %d Takte, %d Muster).  Format siehe examples/c64/prallball/lib/sound.asm.' % (len(order), len(pats)), 'song:',
       '        .byte %d, %d, 1, <%d, >%d       ; Bilder je Zeile, Takte, Wiederholen, Abstand der Mustertabelle' % (FPR, len(order), hdr_len, hdr_len)]
for i, o in enumerate(order): out.append('        .byte ' + ', '.join(str(x) for x in o) + ('       ; Reihenfolge: je Takt Muster Stimme 1, 2, 3' if i == 0 else ''))
out.append('song_patterns:')
for p in pats: out.append('        .byte ' + ', '.join(str(x) for x in p))
open('lib/music.inc', 'w').write('\n'.join(out) + '\n')
print('Takte', len(order), 'Muster', len(pats), 'Bytes', hdr_len + 16 * len(pats))
