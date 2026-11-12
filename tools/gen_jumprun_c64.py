#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt die Assets des C64-Beispiels examples/c64/jumprun: Zeichensatz, Kachelsatz, Screen-Bank, Titelbild, Sprites, Song und die Tabellen (lib/levels.inc).
Die JSON-Dateien sind ganz normale Projektdateien (Zeichensatz-, Kachel-, Bildschirm-, Sprite-Editor und Tracker öffnen sie).
Aufruf: python3 tools/gen_jumprun_c64.py   (im Repository-Wurzelverzeichnis)"""
import json, os, sys
sys.path.insert(0, os.path.dirname(__file__))
import jumprun_art as A
OUT = os.path.join(os.path.dirname(__file__), '..', 'examples', 'c64', 'jumprun')

# w: schreibt eine JSON-Projektdatei (UTF-8, eingerueckt) relativ zum Projektordner des Beispiels.
def w(rel, obj):
    p = os.path.join(OUT, rel); os.makedirs(os.path.dirname(p), exist_ok=True)
    open(p, 'w', encoding='utf-8').write(json.dumps(obj, indent=1, ensure_ascii=False) + '\n')

# ---------------------------------------------------------------- Zeichensatz: 0 leer, 1..80 Kachelzeichen (20 Kacheln x 4), 81 Vollfläche, 84.. Schrift
CH_BLANK, CH_SOLID, CH_FONT = 0, 81, 84
# charset: der eigene Zeichensatz (256 Zeichen, Mehrfarb 4x8): Zeichen 0 leer, 1..80 die Kachelzeichen (je Kachel 4 Zeichen), 81 Vollflaeche (fuer
# die grossen Titelbuchstaben), ab 84 die Schrift. Die Kachelzeichen entstehen aus den 16x16-Kacheln: jedes Zeichen nimmt 8 Zeilen und jeden
# zweiten Bildpunkt (ein Mehrfarbpixel ist 2 Punkte breit); die Rollenbuchstaben werden ueber TILES auf die Werte 0..3 abgebildet.
def charset():
    frames = [None] * 256
    frames[0] = ["0000"] * 8
    for t, sym in enumerate([s[0] for s in A.TILES][1:]):
        for i, rows in enumerate(A.tile_chars_mc(sym)): frames[1 + t * 4 + i] = rows
    frames[CH_SOLID] = ["3333"] * 8
    for i, ch in enumerate(A.FONT_CHARS): frames[CH_FONT + i] = A.font_char_mc(ch)
    frames = [f or ["0000"] * 8 for f in frames]
    return {"format": 1, "kind": "charset", "profile": "c64.char.multicolor", "name": "chars", "colors": {"fg": 1, "bg": 3, "mc1": 9, "mc2": 8}, "frames": [{"rows": f} for f in frames]}

# tileset: der Kachelsatz "hg" (21 Kacheln zu 2x2 Zeichen, einmal für alle vier Bilder) im Format des Kachel-Editors. Eigenschaften: "solid" (Bit 0, tragend)
# und "ladder" (Bit 1, Leiter); die Werte 1 und 2 sind dieselben wie die frühere handgepflegte Tabelle tile_attr (0 leer, 1 tragend, 2 Leiter).
# pins: alle 256 Zeichen behalten ihre Nummer. Dann ist der Zielzeichensatz hg_chars der Spielzeichensatz chars.vcchar (Kachelzeichen 1..80, Vollfläche 81,
# Schrift ab 84), das Titelbild und die Schrift in lib/levels.inc bleiben gültig, und jumprun.asm legt hg_chars an $2000.
def tileset():
    tiles = []
    for t, (sym, name, fn, attr, m, own) in enumerate(A.TILES):
        cells = [0, 0, 0, 0] if t == 0 else [1 + (t - 1) * 4 + k for k in range(4)]
        tile = {"name": name, "cells": cells, "col": (own | 8) if t else 0}
        if attr == 1: tile["props"] = ["solid"]
        elif attr == 2: tile["props"] = ["ladder"]
        tiles.append(tile)
    return {"format": 1, "kind": "tileset", "profile": "c64.tileset", "name": "hg", "size": {"w": 2, "h": 2}, "colorMode": "tile",
            "charsets": ["graphics/chars.vcchar"], "pins": [{"cs": 0, "from": 0, "to": 255}],
            "propDefs": [{"name": "solid", "bit": 0}, {"name": "ladder", "bit": 1}], "tiles": tiles}

# bank: die Screen-Bank "levels" mit den vier Bildern lv1..lv4 (je 20x12 Kacheln = 40x24 Zeichen; Zeile 24 füllt die HAL mit der Laufschrift).
def bank():
    screens = []
    for i in range(4):
        grid = A.LEVELS[i]()
        screens.append({"name": f"lv{i+1}", "w": 40, "h": 24, "colors": {"border": 0, "bg": 3},
                        "tiles": {"set": 0, "origin": [0, 0], "grid": [[A.TILE_INDEX[s] for s in row] for row in grid]}})
    return {"format": 1, "kind": "screen", "profile": "c64.screen.text", "name": "levels", "tilesets": ["graphics/hg.vctiles"], "screens": screens}

def text_codes(s): return [CH_FONT + A.FONT_CHARS.index(c) for c in s.upper()]
# title: das Titelbild als Bildschirm (40x25 Zeichen + Farben): grosse Buchstaben aus Vollflaechen-Zeichen, Untertitel in Schrift, Wolken und Boden aus Kacheln.
def title():
    chars = [[CH_BLANK] * 40 for _ in range(25)]; attrs = [[0] * 40 for _ in range(25)]
    def big(word, row, color, gap=1):
        total = len(word) * 5 + (len(word) - 1) * gap; x0 = (40 - total) // 2
        for n, ch in enumerate(word):
            g = A.FONT[ch]
            for y in range(7):
                for x in range(5):
                    if g[y][x] == '1': chars[row + y][x0 + n * (5 + gap) + x] = CH_SOLID; attrs[row + y][x0 + n * (5 + gap) + x] = color | 8
    big('HUGO', 1, 7); big('&GUMBO', 9, 5)
    def line(s, row, color):
        x0 = (40 - len(s)) // 2
        for i, c in enumerate(text_codes(s)): chars[row][x0 + i] = c; attrs[row][x0 + i] = color
    line('EIN JUMP-AND-RUN-DEMO', 17, 1); line('VINTAGECODER', 0, 1)
    # Wolken und Boden aus Kacheln
    def put_tile(c, k, sym):
        t = A.TILE_INDEX[sym]; own = A.TILES[t][5] | 8
        for j in range(4):
            chars[2 * k + j // 2][2 * c + j % 2] = 1 + (t - 1) * 4 + j; attrs[2 * k + j // 2][2 * c + j % 2] = own
    for c, k in ((1, 1), (17, 6), (1, 7)): put_tile(c, k, 'C')
    for c in range(20): put_tile(c, 10, 'G'); put_tile(c, 11, 'D')
    return {"format": 1, "kind": "screen", "profile": "c64.screen.text", "name": "title", "colors": {"border": 0, "bg": 3, "fg": 1}, "charset": "graphics/chars.vcchar", "chars": chars, "attrs": attrs}

# Die Kunstdaten zaehlen 1 = Sprite-Farbe, 2 = Multicolor 1; die Hardware (und der Editor) kodieren 01 = Multicolor 1, 10 = Sprite-Farbe
HW = str.maketrans('12', '21')
# sprites: Held und Gegner je als Basis (Mehrfarb) und Overlay (Hires) mit allen Bildern (rechts blickend, dann gespiegelt).
def sprites():
    out = {}
    for who in ('hero', 'enemy'):
        fr = A.sprite_frames(who)
        base = {"format": 1, "kind": "sprite", "profile": "c64.sprite.multicolor", "name": f"{who}_base", "colors": {"sprite": 2 if who == 'hero' else 5, "mc1": 10, "mc2": 14}, "frames": [{"rows": [r.translate(HW) for r in b]} for b, o in fr]}
        over = {"format": 1, "kind": "sprite", "profile": "c64.sprite.hires", "name": f"{who}_over", "colors": {"sprite": 0}, "frames": [{"rows": o, "mode": "hires"} for b, o in fr]}
        out[who] = (base, over)
    return out

# ---------------------------------------------------------------- Musik (Tracker-Song, 3 Stimmen: Melodie, Bass, Zupfbegleitung)
# song: die Hintergrundmusik als Tracker-Song (3 Stimmen: Melodie, Bass, Zupfbegleitung; Pentatonik in C-Dur, 4 Zeilen = 1 Schlag).
def song():
    def pat(mel, bass, har):
        rows = []
        for r in range(32):
            rows.append([f"{mel[r]} 00" if r in mel else "--- --", f"{bass[r]} 01" if r in bass else "--- --", f"{har[r]} 02" if r in har else "--- --"])
        return {"rows": rows}
    A1 = pat({0: 'E-5', 2: 'G-5', 4: 'A-5', 6: 'G-5', 8: 'E-5', 12: 'D-5', 14: 'E-5', 16: 'C-5', 18: 'D-5', 20: 'E-5', 24: 'G-5', 26: 'E-5', 28: 'D-5', 30: 'C-5'},
             {0: 'C-3', 4: 'G-2', 8: 'C-3', 12: 'G-2', 16: 'A-2', 20: 'E-3', 24: 'F-2', 28: 'G-2'},
             {2: 'C-4', 6: 'E-4', 10: 'C-4', 14: 'E-4', 18: 'A-3', 22: 'C-4', 26: 'A-3', 30: 'B-3'})
    B1 = pat({0: 'G-5', 2: 'A-5', 4: 'C-6', 6: 'A-5', 8: 'G-5', 12: 'E-5', 14: 'G-5', 16: 'A-5', 18: 'G-5', 20: 'E-5', 24: 'D-5', 26: 'E-5', 28: 'G-5', 30: 'E-5'},
             {0: 'F-2', 4: 'C-3', 8: 'F-2', 12: 'C-3', 16: 'G-2', 20: 'D-3', 24: 'G-2', 28: 'D-3'},
             {2: 'A-3', 6: 'C-4', 10: 'A-3', 14: 'C-4', 18: 'B-3', 22: 'D-4', 26: 'B-3', 30: 'D-4'})
    return {"format": 1, "kind": "song", "profile": "c64.sid.song", "name": "tune", "speed": 6, "rows": 32,
            "instruments": [{"name": "Puls", "wave": "pulse", "pw": 2048, "a": 0, "d": 9, "s": 8, "r": 4}, {"name": "Bass", "wave": "saw", "pw": 2048, "a": 0, "d": 6, "s": 10, "r": 2},
                            {"name": "Zupf", "wave": "pulse", "pw": 1024, "a": 0, "d": 5, "s": 0, "r": 3}],
            "patterns": [A1, B1], "order": [0, 0, 1, 0], "loop": 0}

# ---------------------------------------------------------------- Tabellen
# inc: schreibt lib/levels.inc, die Tabellen fuer den Assembler: Kacheleigenschaften, Farben je Bild, Startwerte, Autopilot-Skripte und Texte.
def inc():
    L = ["; levels.inc - ERZEUGT von tools/gen_jumprun_c64.py (Spielwege aus tools/jumprun_art.py) - nicht von Hand ändern", ""]
    L.append("; Kacheleigenschaften je Kachelnummer (0 leer, 1 tragend, 2 Leiter) stehen im Kachelsatz graphics/hg.vctiles (Export: hg_props, hg_p_solid, hg_p_ladder)")
    # Farben je Bild: Mehrfarb 1/2, Rahmen, Himmelsbänder (6 Stück)
    pal = [(9, 8, [3, 3, 3, 3, 3, 14]), (11, 12, [0, 0, 0, 0, 0, 0]), (2, 10, [6, 6, 14, 14, 14, 14]), (14, 15, [6, 6, 6, 6, 6, 0])]
    L.append("lvl_mc1:\n        .byte " + ",".join(str(p[0]) for p in pal)); L.append("lvl_mc2:\n        .byte " + ",".join(str(p[1]) for p in pal))
    for b in range(6): L.append(f"lvl_sky{b}:\n        .byte " + ",".join(str(p[2][b]) for p in pal))
    L.append("; Start je Bild: Held x (16 Bit), Fuß-y, Blickrichtung; Gegner x, y, Richtung")
    xs, ys, ds, exs, eys, eds = [], [], [], [], [], []
    for f in A.PATHS:
        p = f(); x, y, d, ex, ey, ed = p.start; xs.append(x); ys.append(y); ds.append(d); exs.append(ex); eys.append(ey); eds.append(ed)
    L.append("lvl_hx:\n        .word " + ",".join(map(str, xs))); L.append("lvl_hy:\n        .byte " + ",".join(map(str, ys))); L.append("lvl_hd:\n        .byte " + ",".join(map(str, ds)))
    L.append("lvl_ex:\n        .word " + ",".join(map(str, exs))); L.append("lvl_ey:\n        .byte " + ",".join(map(str, eys))); L.append("lvl_ed:\n        .byte " + ",".join(map(str, eds)))
    for i, f in enumerate(A.PATHS):
        b = A.script_bytes(f()); L.append(f"script{i+1}:\n        .byte " + ",".join(map(str, b)))
    L.append("script_lo:\n        .byte " + ",".join(f"<script{i+1}" for i in range(4))); L.append("script_hi:\n        .byte " + ",".join(f">script{i+1}" for i in range(4)))
    for i in range(4):
        L.append(f"txt_lv{i+1}:\n        .byte " + ",".join(map(str, text_codes(A.LEVEL_NAMES[i].ljust(25)[:25]))))
    L.append("txt_lv_lo:\n        .byte " + ",".join(f"<txt_lv{i+1}" for i in range(4))); L.append("txt_lv_hi:\n        .byte " + ",".join(f">txt_lv{i+1}" for i in range(4)))
    L.append("txt_hud:\n        .byte " + ",".join(map(str, text_codes("HUGO 000000   BILD 1/4   LEBEN"))))
    L.append("txt_prompt:\n        .byte " + ",".join(map(str, text_codes("DEMO - NICHT SPIELBAR"))))
    L.append(f".const CH_BLANK = {CH_BLANK}\n.const CH_SOLID = {CH_SOLID}\n.const CH_FONT = {CH_FONT}\n.const CH_DIGIT0 = {CH_FONT + A.FONT_CHARS.index('0')}\n.const CH_STAR = {CH_FONT + A.FONT_CHARS.index('*')}")
    return "\n".join(L) + "\n"

def main():
    import argparse
    _ap = argparse.ArgumentParser(
        description="""Erzeugt die Assets des C64-Beispiels examples/c64/jumprun aus den gemeinsamen Kunstdaten (tools/jumprun_art.py).

Schreibt im Projektordner examples/c64/jumprun (UTF-8, vorhandene Dateien werden überschrieben, fehlende Ordner angelegt): Zeichensatz (graphics/chars.vcchar), Kachelsatz (graphics/hg.vctiles), Screen-Bank (graphics/levels.vcscreen), Titelbild (graphics/title.vcscreen), Sprites (graphics/*_base.vcsprite, *_over.vcsprite), Song (music/tune.vcsong) und lib/levels.inc. Die JSON-Dateien sind normale Projektdateien der Editoren. Pfade werden relativ zum Ort des Skripts bestimmt; der Arbeitsordner ist egal.""",
        epilog="""Beispiele:
  python3 tools/gen_jumprun_c64.py
  cd tools && python3 gen_jumprun_c64.py

Das Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.""",
        formatter_class=argparse.RawDescriptionHelpFormatter)
    _ap.parse_args()
    w('graphics/chars.vcchar', charset())
    w('graphics/hg.vctiles', tileset()); w('graphics/levels.vcscreen', bank())
    w('graphics/title.vcscreen', title())
    for who, (b, o) in sprites().items(): w(f'graphics/{who}_base.vcsprite', b); w(f'graphics/{who}_over.vcsprite', o)
    w('music/tune.vcsong', song())
    os.makedirs(os.path.join(OUT, 'lib'), exist_ok=True)
    open(os.path.join(OUT, 'lib', 'levels.inc'), 'w', encoding='utf-8').write(inc())
    print('fertig:', OUT)
if __name__ == '__main__': main()
