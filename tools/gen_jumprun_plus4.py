#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt die Assets des Plus/4-Beispiels examples/plus4/jumprun aus den gemeinsamen Kunstdaten (tools/jumprun_art.py).

Plus/4-Besonderheiten gegenueber C64 und Atari (siehe Konzept "Hugo & Gumbo"):
  * TED-Mehrfarb-Text 40x25: Je Zeichen sind vier Farben moeglich: 00 = Hintergrund ($FF15), 01 = Farbe 1 ($FF16), 10 = Farbe 2 ($FF17),
    11 = Zeichenfarbe aus dem Farb-RAM.  Mehrfarbe gilt nur fuer Zellen, deren Farbwert Bit 3 gesetzt hat; dann wird fuer Paar 11 der Farbton Bit 0-2 benutzt.
    Die Zeichensaetze gleichen deshalb dem Atari-Satz (gleiche Wertezuordnung 1/2/3); aus der Atari-Karte wird nur das Zeichenbit 7 (Gruen) entfernt.
  * KEINE Sprites: Held und Gegner sind "Zeichen-Sprites". Die Bilder (8 bzw. 8 Mehrfarbpunkte breit) werden hier um 0..3 Punkte nach rechts
    vorverschoben und als Zeilen zu je 3 Byte abgelegt; die Spielroutine setzt sie zur Laufzeit in 3 Spalten umdefinierter Zeichen zusammen.
    Hero: Umriss = Farbe 1 (dunkel), Koerper = Paar 11 (Zellfarbe je Zellzeile); Gegner: Koerper = Paar 11 (gruen), Augen = Farbe 1.
  * Musik: TED hat zwei Stimmen (Melodie, Bass); Tabellen aus denselben Noten wie beim Atari. (Der Tracker-Export fuer TED fehlt noch.)

Was dieses Skript erzeugt (alles unter examples/plus4/jumprun/):
  graphics/chars.vcchar, lvN.vctiles, title.vcscreen   Projektdateien der Editoren (Zeichensatz, Kachelkarten, Titelbild)
  lib/levels.inc     Assemblertabellen: Kacheleigenschaften, Farben, Startpositionen, Autopilot-Skripte, Texte, Zellfarben je Zeichen,
                     vorverschobene Figurenbilder, Musik, Klangeffekte.  Die HAL (hal_plus4.asm) und die Spiellogik lesen sie ein.

TED-Farben: Farbwert = Helligkeit * 16 + Farbton (Helligkeit 0..7, Farbton 0..15).  Im Farb-RAM bedeutet Bit 3 (+8) "Mehrfarbe", der
Farbton ist dort nur 3 Bit breit (0..7).  Die Register $FF15-$FF17 (Hintergrund, Farbe 1, Farbe 2) nehmen den vollen Farbwert ohne +8.

TED-Tonhoehe: Stimme mit 10-Bit-Wert N klingt mit  f = 110840,5 / (1024 - N)  Hz  (PAL); ted_n() und sfx() rechnen mit genau dieser Formel.
Aufruf: python3 tools/gen_jumprun_plus4.py   (im Repository-Wurzelverzeichnis)"""
import json, os, sys, math
sys.path.insert(0, os.path.dirname(__file__))
import jumprun_art as A
import gen_jumprun_atari as G
OUT = os.path.join(os.path.dirname(__file__), '..', 'examples', 'plus4', 'jumprun')
w = lambda rel, obj: (os.makedirs(os.path.dirname(os.path.join(OUT, rel)), exist_ok=True), open(os.path.join(OUT, rel), 'w', encoding='utf-8').write(json.dumps(obj, indent=1, ensure_ascii=False) + '\n'))

# TED-Farbwert: Helligkeit * 16 + Farbton; fuer Zellfarben im Mehrfarbmodus kommt Bit 3 (8) dazu
# ted() gilt fuer die Register $FF15-$FF17 und die Himmelsbalken, cell() fuer Farb-RAM-Werte (Mehrfarb-Zellen).
def ted(lum, hue): return lum * 16 + hue
def cell(lum, hue): return ted(lum, hue) + 8
WHITE, GREEN = cell(7, 1), cell(4, 5)       # Standard-Zellfarbe weiss (Helligkeit 7, Farbton 1); gruen (4, 5) fuer Gras, Busch und das Titelwort

def charset():
    # Zeichensatz: aus dem Atari-Satz uebernommen (gleiche Zuordnung der Pixelpaare 01/10/11), auf 256 Zeichen aufgefuellt
    d = G.charset(); d['profile'] = 'plus4.char.multicolor'
    d['frames'] += [{"rows": ["0000"] * 8} for _ in range(256 - len(d['frames']))]     # 256 Zeichen; die Zeichen $E0-$F8 sind der Pool der Zeichen-Sprites
    d['colors'] = {"fg": 113, "bg": 0, "mc1": 34, "mc2": 42}      # Vorschaufarben des Editors (TED-Farbwerte: Zeichen weiss, Hintergrund schwarz)
    return d
def tilemap(i):
    # Kachelkarte von Bild i: 2 x 2 Zeichen je Kachel, Karte 20 x 12 -> 40 x 24 Zellen
    d = G.tilemap(i); d['profile'] = 'plus4.tilemap.2x2'; d['colors'] = {"bg": 0, "fg": 113}; return d
def title():
    # Titelbild: 24 Zeilen zu 40 Zeichencodes
    d = G.title(); d['profile'] = 'plus4.screen.text'; d['colors'] = {"bg": 0, "fg": 113}
    d['chars'] = [[c & 0x7f for c in row] for row in d['chars']]                         # Bit 7 (Gruen beim Atari) gibt es hier nicht: Gruen kommt aus dem Farb-RAM
    return d

# ---------------------------------------------------------------- Zeichen-Sprites
# Die Figurenbilder werden als Pixelpaare (0 = durchsichtig/Hintergrund, 1 = Farbe 1/Umriss, 3 = Paar 11/Zellfarbe) angelegt,
# 12 Paare breit (= 3 Zellen), und um 0..3 Paare nach rechts vorverschoben.  hal_update_sprites (HAL) waehlt dann zur Laufzeit je nach
# x mod 4 das passende Bild, sodass die Figur auf einen Mehrfarbpunkt genau steht, obwohl sie an ganze Zellen gebunden ist.
def pairs_to_bytes(pairs):
    # 12 Paare (links beginnend) zu 3 Byte; je Byte vier Paare, das linke im hoechsten Bitpaar (Bits 7-6)
    pairs = list(pairs) + [0] * (12 - len(pairs))
    return [sum(pairs[b * 4 + k] << (6 - 2 * k) for k in range(4)) for b in range(3)]
def shifted(rows):
    """rows: Liste von Zeilen mit je 8 Paarwerten -> 4 Verschiebungen (0..3 Punkte nach rechts), je Zeile 3 Byte"""
    return [[pairs_to_bytes([0] * s + r) for r in rows] for s in range(4)]
def hero_grids():
    # Held: 21 Zeilen x 8 Paare; Koerper = Paar 11, Umriss (4-Nachbarschaft der Koerperpunkte) = Paar 01, ein Auge; erst alle Bilder normal, dann gespiegelt
    out = []
    for flip in (False, True):
        for name in A.HERO_FRAMES:
            rows = A.hero_frame(name)
            if flip: rows = A.mirror(rows)
            body = [[0] * 8 for _ in range(21)]
            for y, r in enumerate(rows):
                for x in range(6): body[y + 1][x + 1] = 3 if int(r[x + 3]) else 0      # Koerper (alle Farben) = Paar 11
            res = [r[:] for r in body]
            for y in range(21):
                for x in range(8):
                    if body[y][x]: continue
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        xx, yy = x + dx, y + dy
                        if 0 <= xx < 8 and 0 <= yy < 21 and body[yy][xx]: res[y][x] = 1; break    # Umriss = Farbe 1
            res[6][5 if not flip else 2] = 1                                          # Auge
            out.append(res)
    return out
def enemy_grids():
    # Gegner: 13 Zeilen x 8 Paare; Koerper = Paar 11, Augen = Paar 01; je Bild normal und gespiegelt
    out = []
    for flip in (False, True):
        for name in A.ENEMY_FRAMES:
            rows = A.enemy_frame(name)
            if flip: rows = A.mirror(rows)
            out.append([[0 if r[x + 2] == '0' else (1 if r[x + 2] == '3' else 3) for x in range(8)] for r in rows])     # Augen (3) = Farbe 1, sonst Koerper = Paar 11
    return out

# ---------------------------------------------------------------- Musik
NOTE = {'C': 0, 'D': 2, 'E': 4, 'F': 5, 'G': 7, 'A': 9, 'B': 11}
def ted_n(name):
    # Notenname (z. B. 'E-5') -> TED-Tonwert N:  f = 440 * 2^((MIDI - 69) / 12),  N = 1024 - 110840,5 / f  (siehe f-Formel oben)
    n = NOTE[name[0]] + (1 if name[1] == '#' else 0) + 12 * (int(name[2]) + 1)      # MIDI-Nummer, C-4 = 60
    f = 440 * 2 ** ((n - 69) / 12)
    return round(1024 - 110840.5 / f)
def music():
    # Zwei Stimmen (Melodie, Bass), vier Muster zu 32 Zeilen = 128 Zeilen; die HAL spielt eine Zeile je 6 Bilder
    pats = {0: ({0: 'E-5', 2: 'G-5', 4: 'A-5', 6: 'G-5', 8: 'E-5', 12: 'D-5', 14: 'E-5', 16: 'C-5', 18: 'D-5', 20: 'E-5', 24: 'G-5', 26: 'E-5', 28: 'D-5', 30: 'C-5'},
                {0: 'C-3', 4: 'G-3', 8: 'C-3', 12: 'G-3', 16: 'A-3', 20: 'E-3', 24: 'F-3', 28: 'G-3'}),
            1: ({0: 'G-5', 2: 'A-5', 4: 'C-6', 6: 'A-5', 8: 'G-5', 12: 'E-5', 14: 'G-5', 16: 'A-5', 18: 'G-5', 20: 'E-5', 24: 'D-5', 26: 'E-5', 28: 'G-5', 30: 'E-5'},
                {0: 'F-3', 4: 'C-3', 8: 'F-3', 12: 'C-3', 16: 'G-3', 20: 'D-3', 24: 'G-3', 28: 'D-3'})}
    voices = [[], []]
    for p in (0, 0, 1, 0):
        for v in (0, 1):
            for r in range(32):
                n = pats[p][v].get(r)
                voices[v].append(ted_n(n) if n else 0xffff)            # $FFFF = keine neue Note (weiterklingen)
    return voices
def sfx():
    """Je Effekt: Startwert N (16 Bit), Aenderung je Bild (16 Bit, vorzeichenbehaftet), Rauschen (0/1), Dauer. Aus den Atari-Werten (AUDF) umgerechnet."""
    atari = [(100, -3, 0, 22), (40, 4, 1, 7), (20, 0, 0, 2), (60, -6, 0, 4), (40, -3, 0, 10), (120, 4, 0, 12), (70, -1, 0, 40), (90, -1, 0, 60)]
    out = []
    for f0, d, noise, dur in atari:
        def N(a): return max(1, min(1020, round(1024 - 110840.5 / (63921 / (2 * (a + 1))))))
        if noise: n0, dn = 1024 - 90, 0 if False else 6
        else:
            n0 = N(f0); dn = round((N(f0 + d * dur) - n0) / dur)
        out.append((n0, dn & 0xffff, noise, dur))
    return out

# ---------------------------------------------------------------- Tabellen
def inc():
    # Baut den Text von lib/levels.inc Abschnitt fuer Abschnitt (Farben, Zellfarben, Startwerte, Skripte, Texte, Figurenbilder, Musik, Effekte)
    L = ["; levels.inc - ERZEUGT von tools/gen_jumprun_plus4.py (Spielwege aus tools/jumprun_art.py) - nicht von Hand aendern", ""]
    L.append("; Kacheleigenschaften je Kachelnummer: 0 leer, 1 tragend, 2 Leiter")
    L.append("tile_attr:\n        .byte " + ",".join(str(t[3]) for t in A.TILES))
    # Farbwerte der Register $FF16 (Farbe 1), $FF17 (Farbe 2) und sechs Himmelsbalken ($FF15) je Bild; TED-Farbwert = Helligkeit * 16 + Farbton
    pal = [(ted(3, 9), ted(5, 9), [ted(5, 13), ted(5, 13), ted(4, 13), ted(4, 13), ted(3, 13), ted(2, 13)]),
           (ted(2, 1), ted(4, 1), [0] * 6),
           (ted(3, 2), ted(6, 2), [ted(3, 6), ted(4, 6), ted(5, 6), ted(6, 6), ted(7, 6), ted(7, 6)]),
           (ted(4, 14), ted(6, 14), [ted(1, 14), ted(1, 14), ted(2, 14), ted(2, 14), ted(3, 14), 0])]
    L.append("lvl_mc1:\n        .byte " + ",".join(str(p[0]) for p in pal)); L.append("lvl_mc2:\n        .byte " + ",".join(str(p[1]) for p in pal))
    for b in range(6): L.append(f"lvl_sky{b}:\n        .byte " + ",".join(str(p[2][b]) for p in pal))
    # Zellfarbe je Zeichencode: weiss, ausser Gras/Busch und dem gruenen Titelwort
    col = [WHITE] * 256
    for t, sym in enumerate([s[0] for s in A.TILES][1:]):
        if sym in G.GREEN:
            for k in range(4): col[1 + t * 4 + k] = GREEN
    col[G.CH_SOLID3] = GREEN
    L.append("; Zellfarbe (Farb-RAM-Wert, Bit 3 = Mehrfarbe) je Zeichencode\nchar_col:")
    for i in range(0, 256, 16): L.append("        .byte " + ",".join(str(c) for c in col[i:i + 16]))
    L.append("; Start je Bild: Held x (16 Bit), Fuss-y, Blickrichtung; Gegner x, y, Richtung")
    xs, ys, ds, exs, eys, eds = [], [], [], [], [], []
    for f in A.PATHS:
        p = f(); x, y, d, ex, ey, ed = p.start; xs.append(x); ys.append(y); ds.append(d); exs.append(ex); eys.append(ey); eds.append(ed)
    L.append("lvl_hx:\n        .word " + ",".join(map(str, xs))); L.append("lvl_hy:\n        .byte " + ",".join(map(str, ys))); L.append("lvl_hd:\n        .byte " + ",".join(map(str, ds)))
    L.append("lvl_ex:\n        .word " + ",".join(map(str, exs))); L.append("lvl_ey:\n        .byte " + ",".join(map(str, eys))); L.append("lvl_ed:\n        .byte " + ",".join(map(str, eds)))
    for i, f in enumerate(A.PATHS):
        b = A.script_bytes(f()); L.append(f"script{i+1}:\n        .byte " + ",".join(map(str, b)))
    L.append("script_lo:\n        .byte " + ",".join(f"<script{i+1}" for i in range(4))); L.append("script_hi:\n        .byte " + ",".join(f">script{i+1}" for i in range(4)))
    for i in range(4): L.append(f"txt_lv{i+1}:\n        .byte " + ",".join(map(str, G.text_codes(A.LEVEL_NAMES[i].ljust(25)[:25]))))
    L.append("txt_lv_lo:\n        .byte " + ",".join(f"<txt_lv{i+1}" for i in range(4))); L.append("txt_lv_hi:\n        .byte " + ",".join(f">txt_lv{i+1}" for i in range(4)))
    L.append("txt_hud:\n        .byte " + ",".join(map(str, G.text_codes("HUGO 000000   BILD 1/4   LEBEN"))))
    L.append("txt_prompt:\n        .byte " + ",".join(map(str, G.text_codes("DEMO - NICHT SPIELBAR"))))
    L.append(f".const CH_BLANK = {G.CH_BLANK}\n.const CH_FONT = {G.CH_FONT}\n.const CH_DIGIT0 = {G.CH_FONT + A.FONT_CHARS.index('0')}\n.const CH_STAR = {G.CH_FONT + A.FONT_CHARS.index('*')}")
    # Zeichen-Sprites: vorverschobene Bilder [Bild][Verschiebung][Zeile][3 Byte]
    # hero_lo/hero_hi bzw. enemy_lo/enemy_hi: Zeigertabellen; Index = Bildnummer * 4 + Verschiebung (so greift die HAL darauf zu)
    for name, grids, rows in (("hero", hero_grids(), 21), ("enemy", enemy_grids(), 13)):
        L.append(f"; {name}: {len(grids)} Bilder x 4 Verschiebungen x {rows} Zeilen x 3 Byte\n{name}_data:")
        for g in grids:
            for s, sh in enumerate(shifted(g)):
                L.append("        .byte " + ",".join(str(b) for r in sh for b in r))
        n = len(grids) * 4; size = rows * 3
        L.append(f"{name}_lo:\n        .byte " + ",".join(f"<({name}_data+{i * size})" for i in range(n)))
        L.append(f"{name}_hi:\n        .byte " + ",".join(f">({name}_data+{i * size})" for i in range(n)))
    # Musik: je Stimme 128 Zeilen (4 Muster zu 32)
    v = music()
    for k in (0, 1):
        L.append(f"mus{k+1}_lo:\n" + "\n".join("        .byte " + ",".join(str(n & 255) for n in v[k][i:i + 16]) for i in range(0, 128, 16)))
        L.append(f"mus{k+1}_hi:\n" + "\n".join("        .byte " + ",".join(str(n >> 8) for n in v[k][i:i + 16]) for i in range(0, 128, 16)))
    sx = sfx()
    L.append("; Effekte (8 x 6 Byte): N niedrig, N hoch, Aenderung niedrig, Aenderung hoch, Rauschen (0/1), Dauer\nsfx_tab:")
    for n0, dn, noise, dur in sx: L.append(f"        .byte {n0 & 255},{n0 >> 8},{dn & 255},{dn >> 8},{noise},{dur}")
    return "\n".join(L) + "\n"

def main():
    import argparse
    _ap = argparse.ArgumentParser(
        description="""Erzeugt die Assets des Plus/4-Beispiels examples/plus4/jumprun aus den gemeinsamen Kunstdaten (tools/jumprun_art.py).

Schreibt im Projektordner examples/plus4/jumprun (UTF-8, vorhandene Dateien werden überschrieben, fehlende Ordner angelegt): Zeichensatz (graphics/chars.vcchar), je Bild eine Kachelkarte (graphics/lv1..lv4.vctiles), Titelbild (graphics/title.vcscreen) und lib/levels.inc. Die JSON-Dateien sind normale Projektdateien der Editoren. Pfade werden relativ zum Ort des Skripts bestimmt; der Arbeitsordner ist egal.""",
        epilog="""Beispiele:
  python3 tools/gen_jumprun_plus4.py
  cd tools && python3 gen_jumprun_plus4.py

Das Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.""",
        formatter_class=argparse.RawDescriptionHelpFormatter)
    _ap.parse_args()
    # Alle Dateien schreiben: Editor-Projekte (graphics/) und die Tabellen lib/levels.inc
    w('graphics/chars.vcchar', charset())
    for i in range(4): w(f'graphics/lv{i+1}.vctiles', tilemap(i))
    w('graphics/title.vcscreen', title())
    os.makedirs(os.path.join(OUT, 'lib'), exist_ok=True)
    open(os.path.join(OUT, 'lib', 'levels.inc'), 'w', encoding='utf-8').write(inc())
    print('fertig:', OUT)
if __name__ == '__main__': main()
