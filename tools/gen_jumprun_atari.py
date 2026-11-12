#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Erzeugt die Assets des Atari-Beispiels examples/atari/jumprun aus den gemeinsamen Kunstdaten (tools/jumprun_art.py).

Atari-Besonderheiten gegenueber dem C64 (siehe Konzept "Hugo & Gumbo"):
  * ANTIC-Modus 4: Mehrfarb-Text 40x24, 4 Farben je Zeichen aus COLBK, COLPF0, COLPF1, COLPF2; Zeichenbit 7 waehlt statt COLPF2 die Farbe COLPF3.
    Die Zeichenfarbe ist also fuer den ganzen Bildschirm gleich (keine Farbe je Zeichen wie beim C64). Gras und Busch bekommen Bit 7 (gruen = COLPF3),
    die uebrigen Kacheln nutzen COLPF0/1/2, die je Bild eingestellt werden.
  * Held = vier uebereinanderliegende Spieler (Player-Missile-Grafik): P0 Umriss und Auge (schwarz), P1 Haut, P2 Hemd, P3 Hose, je 8 Farbtakte breit.
    Der Held ist deshalb schmaler als auf dem C64 (6 Mehrfarbpunkte Koerper + Umriss). Gegner = die vier Missiles als "5. Spieler" in COLPF3 (gruen).
  * Schrift: eigene 4-Punkte-Schrift A.FONT4 (aus dem 5x7-Zeichen waere N/M/H, O/0/D nicht mehr unterscheidbar).

Was erzeugt wird (alles unter examples/atari/jumprun):
  graphics/chars.vcchar   Zeichensatz (128 Zeichen zu 4 x 8 Mehrfarbpixeln, Profil atari.char.antic4)
  graphics/lv1..lv4.vctiles  Kachelkarten (Kachel = 2 x 2 Zeichen, 20 x 12 Kacheln je Bild)
  graphics/title.vcscreen    Titelbild (24 x 40 Zeichencodes)
  music/tune.vcsong          Musik (3 Stimmen, Kanal 4 bleibt frei)
  lib/levels.inc                Tabellen und Spielerdaten fuer den Assembler (nicht von Hand aendern)
Die Beschreibungsdateien (.vcchar, .vctiles, .vcscreen, .vcsong; Inhalt JSON) werden von der IDE beim Bauen nach build/export/*.asm exportiert; die Zeichen- und Pixelwerte
sind Zahlen 0..3, wobei im Mehrfarbtext jedes Pixel (4 je Zeichenzeile) zwei Bit belegt:
  0 = COLBK (Hintergrund), 1 = COLPF0, 2 = COLPF1, 3 = COLPF2 (bzw. COLPF3, wenn Bit 7 des Zeichencodes gesetzt ist).
Alle Kunstdaten (Kacheln, Figuren, Wege, Schrift) stammen aus tools/jumprun_art.py und sind fuer alle Systeme gleich; dieses Skript
rechnet sie nur in die Atari-Formate um.
Aufruf: python3 tools/gen_jumprun_atari.py   (im Repository-Wurzelverzeichnis)"""
import json, os, sys
sys.path.insert(0, os.path.dirname(__file__))
import jumprun_art as A
OUT = os.path.join(os.path.dirname(__file__), '..', 'examples', 'atari', 'jumprun')

# w: schreibt ein JSON-Objekt (lesbar eingerueckt, Umlaute unveraendert) in eine Datei unter OUT und legt fehlende Ordner an
def w(rel, obj):
    p = os.path.join(OUT, rel); os.makedirs(os.path.dirname(p), exist_ok=True)
    open(p, 'w', encoding='utf-8').write(json.dumps(obj, indent=1, ensure_ascii=False) + '\n')

# AMAP: Farbrolle der Kunstdaten (Buchstabe in den Kachelbildern von jumprun_art) ->
# Pixelwert 0..3 je Kachelsymbol.  Nicht aufgefuehrte Rollen werden zu 0 (Hintergrund).
# Rolle -> Pixelwert (0 Hintergrund, 1 COLPF0, 2 COLPF1, 3 COLPF2 bzw. COLPF3 bei gesetztem Zeichenbit 7), je Kachel
AMAP = {'G': {'g': 3, 'G': 3, 'e': 1, 'E': 2}, 'D': {'e': 1, 'E': 2, 's': 3}, 'B': {'r': 2, 'E': 1, 'o': 3}, 'S': {'L': 3, 'S': 1, 's': 2}, 'H': {'e': 2, 'E': 1, 'o': 3},
        'P': {'o': 3, 'e': 2, 'E': 1}, 'C': {'w': 3, 'L': 3}, 'c': {'w': 3, 'L': 3}, 'W': {'c': 3, 'w': 3, 'b': 2, 'B': 1}, 'E': {'E': 1, 'e': 2, 'y': 3}, 'o': {'y': 2, 'w': 3, 'o': 1},
        'b': {'g': 3, 'G': 1}, 'f': {'G': 1, 'r': 3, 'y': 2}, 'k': {'S': 1, 'k': 0}, 'T': {'e': 1, 'y': 3, 'o': 2, 'k': 0}, 'x': {'w': 3, 'c': 2, 'v': 1, 'k': 0},
        'M': {'L': 3, 's': 2, 'S': 1}, 'F': {'e': 1, 'r': 2, 'w': 3}, 'm': {'y': 2, 'w': 3}, '*': {'w': 3, 'y': 3}}
# Kachelsymbole 'G' (Gras) und 'b' (Busch): ihre Zeichen bekommen Bit 7 (0x80), der Pixelwert 3 erscheint dann in COLPF3 = gruen, das in allen Bildern gleich bleibt
GREEN = {'G', 'b'}                       # Kacheln mit gesetztem Zeichenbit 7 (Farbe 3 = COLPF3 = gruen)
# Zeichenbelegung: 0 leer; 1..80 die 20 Kacheln zu je 4 Zeichen; 81 Vollflaeche Farbe 3; 82.. Schrift; 127 Vollflaeche Farbe 2
CH_BLANK, CH_SOLID3, CH_FONT, CH_SOLID2 = 0, 81, 82, 127   # 127: Vollflaeche in Farbe 1 (Titelbuchstaben "HUGO")

def tile_chars(sym):
    # Zerlegt das 16 x 16 Punkte grosse Kachelbild in 4 Zeichen (oben links, oben rechts, unten links, unten rechts). Waagerecht werden je zwei
    # logische Punkte zu einem Mehrfarbpixel zusammengefasst (x * 2): die Zeichen sind 4 Pixel = 8 logische Punkte breit.
    g = A.tile_grid(sym); m = AMAP.get(sym, {}); out = []
    for cy in (0, 1):
        for cx in (0, 1):
            out.append([''.join(str(m.get(g[cy * 8 + y][cx * 8 + x * 2], 0)) for x in range(4)) for y in range(8)])
    return out
def font_char(ch):
    """Schrift: 4 Punkte breit (A.FONT4), Zeile 8 leer; Pixelwert 3 (COLPF2 bzw. Zeichenfarbe)"""
    rows = A.FONT4.get(ch, A.FONT4[' '])
    return [''.join('3' if c == 'X' else '0' for c in rows[y]) if y < 7 else '0000' for y in range(8)]
# text_codes: Zeichenfolge -> Zeichencodes der Schrift (Grossbuchstaben; Position in FONT_CHARS + CH_FONT)
def text_codes(s): return [CH_FONT + A.FONT_CHARS.index(c) for c in s.upper()]

def charset():
    # Alle 128 Zeichen: zunaechst leer (0000 = vier Hintergrundpixel), dann Kachelzeichen, zwei Vollflaechen und die Schrift.
    # 'colors' sind nur Vorschaufarben fuer den Editor (Atari-Farbwerte als Zahl: 14 = grau-hell usw.), nicht die Farben im Spiel.
    fr = [["0000"] * 8 for _ in range(128)]
    for t, sym in enumerate([s[0] for s in A.TILES][1:]):
        for i, rows in enumerate(tile_chars(sym)): fr[1 + t * 4 + i] = rows
    fr[CH_SOLID3] = ["3333"] * 8; fr[CH_SOLID2] = ["2222"] * 8
    for i, ch in enumerate(A.FONT_CHARS): fr[CH_FONT + i] = font_char(ch)
    return {"format": 1, "kind": "charset", "profile": "atari.char.antic4", "name": "chars", "colors": {"fg": 14, "bg": 0, "mc1": 34, "mc2": 42}, "frames": [{"rows": f} for f in fr]}

def tilemap(i):
    # Kachelkarte von Bild i: Kachel t besteht aus den Zeichen 1 + (t-1)*4 .. +3 (Kachel 0 ist leer = Zeichen 0); 'solid' = tragende Kachel.
    # 'map' ist das 20 x 12 grosse Raster der Kachelnummern aus A.LEVELS.
    tiles = []
    for t, (sym, name, fn, attr, m, own) in enumerate(A.TILES):
        chars = [0, 0, 0, 0] if t == 0 else [1 + (t - 1) * 4 + k for k in range(4)]
        tiles.append({"name": name, "chars": chars, "color": 0, "solid": attr == 1})
    grid = A.LEVELS[i]()
    return {"format": 1, "kind": "tilemap", "profile": "atari.tilemap.2x2", "name": f"lv{i+1}", "colors": {"bg": 0, "fg": 14}, "charset": "graphics/chars.vcchar", "tiles": tiles,
            "map": [[A.TILE_INDEX[s] for s in row] for row in grid]}

def title():
    # Titelbild: 24 Zeilen x 40 Zeichencodes.  'HUGO' (Zeile 1) in Farbe 2, '&GUMBO' (Zeile 9) in Farbe 3 mit Bit 7 (COLPF3 = gruen),
    # dazu Untertitel, Wolken, eine Grasreihe (Zeile 10) mit Erde darunter (Zeile 11).  Grosse Buchstaben: 5 x 7 Zeichen je Buchstabe aus A.FONT,
    # ein Pixel der Schrift = ein Vollflaechenzeichen.
    chars = [[CH_BLANK] * 40 for _ in range(24)]
    def big(word, row, code):
        total = len(word) * 5 + (len(word) - 1); x0 = (40 - total) // 2
        for n, ch in enumerate(word):
            g = A.FONT[ch]
            for y in range(7):
                for x in range(5):
                    if g[y][x] == '1': chars[row + y][x0 + n * 6 + x] = code
    big('HUGO', 1, CH_SOLID2); big('&GUMBO', 9, CH_SOLID3 | 0x80)
    def line(s, row):
        x0 = (40 - len(s)) // 2
        for i, c in enumerate(text_codes(s)): chars[row][x0 + i] = c
    line('EIN JUMP-AND-RUN-DEMO', 17); line('VINTAGECODER', 0)
    def put_tile(c, k, sym):
        t = A.TILE_INDEX[sym]
        for j in range(4): chars[2 * k + j // 2][2 * c + j % 2] = (1 + (t - 1) * 4 + j) | (0x80 if sym in GREEN else 0)
    for c, k in ((1, 1), (17, 6), (1, 7)): put_tile(c, k, 'C')
    for c in range(20): put_tile(c, 10, 'G'); put_tile(c, 11, 'D')
    return {"format": 1, "kind": "screen", "profile": "atari.screen.antic2", "name": "title", "colors": {"bg": 0, "fg": 14}, "charset": "graphics/chars.vcchar", "chars": chars}

# ---------------------------------------------------------------- Musik (POKEY, RMT): 3 Stimmen, Kanal 4 bleibt fuer die Klangeffekte frei
def song():
    # Zwei Muster zu je 32 Zeilen (Melodie, Bass, Zupfstimme; die vierte Spur bleibt leer = Kanal 4 fuer Klangeffekte), Reihenfolge A A B A, Tempo 6
    # (Bilder je Zeile).  Eintrag 'E-5 00' = Note E in Oktave 5 mit Instrument 00.  Die Huellkurven 'env' sind Lautstaerken 15..0 je Bild;
    # alle Instrumente reine Toene (dist 'pure'); 'loop' = ab dort wird die Huellkurve wiederholt (Haltephase).
    def pat(mel, bass, har):
        rows = []
        for r in range(32):
            rows.append([f"{mel[r]} 00" if r in mel else "--- --", f"{bass[r]} 01" if r in bass else "--- --", f"{har[r]} 02" if r in har else "--- --", "--- --"])
        return {"rows": rows}
    A1 = pat({0: 'E-5', 2: 'G-5', 4: 'A-5', 6: 'G-5', 8: 'E-5', 12: 'D-5', 14: 'E-5', 16: 'C-5', 18: 'D-5', 20: 'E-5', 24: 'G-5', 26: 'E-5', 28: 'D-5', 30: 'C-5'},
             {0: 'C-3', 4: 'G-3', 8: 'C-3', 12: 'G-3', 16: 'A-3', 20: 'E-3', 24: 'F-3', 28: 'G-3'},
             {2: 'C-4', 6: 'E-4', 10: 'C-4', 14: 'E-4', 18: 'A-4', 22: 'C-4', 26: 'A-4', 30: 'B-4'})
    B1 = pat({0: 'G-5', 2: 'A-5', 4: 'C-6', 6: 'A-5', 8: 'G-5', 12: 'E-5', 14: 'G-5', 16: 'A-5', 18: 'G-5', 20: 'E-5', 24: 'D-5', 26: 'E-5', 28: 'G-5', 30: 'E-5'},
             {0: 'F-3', 4: 'C-3', 8: 'F-3', 12: 'C-3', 16: 'G-3', 20: 'D-3', 24: 'G-3', 28: 'D-3'},
             {2: 'A-4', 6: 'C-4', 10: 'A-4', 14: 'C-4', 18: 'B-4', 22: 'D-4', 26: 'B-4', 30: 'D-4'})
    return {"format": 1, "kind": "song", "profile": "atari.pokey.song", "name": "tune", "speed": 6, "rows": 32,
            "instruments": [{"name": "Ton", "dist": "pure", "env": [15, 14, 12, 10, 8, 7, 6, 5, 4, 4, 3, 3, 2, 2, 1, 1], "loop": 15},
                            {"name": "Bass", "dist": "pure", "env": [15, 15, 14, 13, 12, 12, 11, 11, 10, 10, 9, 9, 8, 8, 8, 8], "loop": 15},
                            {"name": "Zupf", "dist": "pure", "env": [10, 8, 5, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], "loop": 15}],
            "patterns": [A1, B1], "order": [0, 0, 1, 0], "loop": 0}

# ---------------------------------------------------------------- Spielerdaten (PMG): je Bild 21 Zeilen x 4 Spieler fuer den Helden, 13 Zeilen Missile-Bytes fuer den Gegner
def hero_pm():
    # Fuer jedes der 14 Heldenbilder (7 nach rechts, 7 gespiegelt) werden die vier Spielerbytefolgen gebildet:
    # grid = 8 Takte breites Raster (Bit 7 links); der 6 Punkte breite Koerper (Spalten 3..8 des Bildes) wird nach rechts um eins versetzt eingetragen,
    # damit links und rechts je ein Takt fuer den Umriss bleibt.  Der Umriss ist jeder leere Punkt, der einen Koerperpunkt zur Nachbarn hat (4 Richtungen).
    # Der Koerper beginnt in Zeile 1, Zeile 0 (und die letzte) bleibt dem Umriss; deshalb 21 Zeilen.  Pixelrollen im Bildraster: 1 Hemd, 2 Haut, 3 Hose.
    """Liste je Bild: (P0..P3), jeweils 21 Bytes. Koerper = Spalten 3..8 des 12 Punkte breiten Bildes (6 Punkte), Umriss 1 Punkt darum -> 8 Takte."""
    out = []
    for flip in (False, True):
        for name in A.HERO_FRAMES:
            rows = A.hero_frame(name)
            if flip: rows = A.mirror(rows)
            # 8 Spalten (Spielerbits 7..0), Zeile 0 = Umrisszeile oben
            grid = [[0] * 8 for _ in range(21)]
            for y, r in enumerate(rows):
                for x in range(6): grid[y + 1][x + 1] = int(r[x + 3])
            mask = [[1 if v else 0 for v in row] for row in grid]
            outline = [[0] * 8 for _ in range(21)]
            for y in range(21):
                for x in range(8):
                    if mask[y][x]: continue
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        xx, yy = x + dx, y + dy
                        if 0 <= xx < 8 and 0 <= yy < 21 and mask[yy][xx]: outline[y][x] = 1; break
            eye_x = 5 if not flip else 2
            outline[6][eye_x] = 1                        # Auge (nur P0 kann es schwarz zeichnen)
            def byte(bits): return sum(b << (7 - i) for i, b in enumerate(bits))
            p0 = [byte(outline[y]) for y in range(21)]
            p1 = [byte([1 if grid[y][x] == 2 else 0 for x in range(8)]) for y in range(21)]    # Haut
            p2 = [byte([1 if grid[y][x] == 1 else 0 for x in range(8)]) for y in range(21)]    # Hemd
            p3 = [byte([1 if grid[y][x] == 3 else 0 for x in range(8)]) for y in range(21)]    # Hose
            out.append((p0, p1, p2, p3))
    return out
def enemy_pm():
    # Gegner: jedes Bild liefert 13 Bytes fuer den Missile-Speicher.  Zwei Pixel breite Gruppen (Spalten 2j, 2j+1) gehoeren zu Missile j; die vier
    # Missiles sitzen im selben Byte (je 2 Bit).  Wert 0 (leer) und 3 (Augen) bleiben unbesetzt, die Augen sind also Loecher in der gruenen Figur.
    """Je Bild 13 Bytes fuer den Missile-Speicher: Missile j (j = 0..3, von links) belegt die Spalten 2j, 2j+1 der 8 Takte breiten Silhouette (Bildspalten 2..9);
    Missile 0 = Bits 1..0, Missile 1 = Bits 3..2, Missile 2 = Bits 5..4, Missile 3 = Bits 7..6 (das hoehere Bit eines Paares ist links)."""
    out = []
    for flip in (False, True):
        for name in A.ENEMY_FRAMES:
            rows = A.enemy_frame(name)
            if flip: rows = A.mirror(rows)
            col = []
            for r in rows:
                bits = [1 if (r[x + 2] != '0' and r[x + 2] != '3') else 0 for x in range(8)]      # Augen (Wert 3) bleiben durchsichtig
                b = 0
                for j in range(4): b |= ((bits[2 * j] << 1) | bits[2 * j + 1]) << (2 * j)
                col.append(b)
            out.append(col)
    return out

def inc():
    # Erzeugt lib/levels.inc als Text: Kacheleigenschaften, Bit 7 je Kachel, Farben je Bild, Startpositionen, Autopilot-Skripte, Texte und
    # die Spielerdaten.  Die Farbwerte in 'pal' sind Atari-Farben (Farbton * 16 + Helligkeit), je Bild (COLPF0, COLPF1, COLPF2) und sechs Himmelsbalken;
    # Bild 2 (Hoehle) hat schwarzen Himmel (00), Bild 4 (Wolken) endet im untersten Balken mit schwarz.
    L = ["; levels.inc - ERZEUGT von tools/gen_jumprun_atari.py (Spielwege aus tools/jumprun_art.py) - nicht von Hand aendern", ""]
    L.append("; Kacheleigenschaften je Kachelnummer: 0 leer, 1 tragend, 2 Leiter")
    L.append("tile_attr:\n        .byte " + ",".join(str(t[3]) for t in A.TILES))
    L.append("; Zeichenbit 7 je Kachel (gruen = COLPF3): 128 fuer Gras und Busch, sonst 0")
    L.append("tile_hi:\n        .byte " + ",".join('128' if t[0] in GREEN else '0' for t in A.TILES))
    # Farben je Bild: COLPF0, COLPF1, COLPF2 und sechs Himmelsbalken (COLBK); Atari-Farbwert = Farbton * 16 + Helligkeit
    pal = [((0x24, 0x2a, 0x0e), [0x9c, 0x9a, 0x98, 0x96, 0x94, 0x92]), ((0x04, 0x08, 0x0e), [0x00] * 6),
           ((0x34, 0x3a, 0x0e), [0x84, 0x86, 0x88, 0x8a, 0x8c, 0x8c]), ((0x84, 0x8a, 0x0e), [0x72, 0x72, 0x74, 0x74, 0x76, 0x00])]
    L.append("lvl_pf0:\n        .byte " + ",".join(str(p[0][0]) for p in pal)); L.append("lvl_pf1:\n        .byte " + ",".join(str(p[0][1]) for p in pal)); L.append("lvl_pf2:\n        .byte " + ",".join(str(p[0][2]) for p in pal))
    for b in range(6): L.append(f"lvl_sky{b}:\n        .byte " + ",".join(str(p[1][b]) for p in pal))
    L.append("; Start je Bild: Held x (16 Bit), Fuss-y, Blickrichtung; Gegner x, y, Richtung")
    xs, ys, ds, exs, eys, eds = [], [], [], [], [], []
    for f in A.PATHS:
        p = f(); x, y, d, ex, ey, ed = p.start; xs.append(x); ys.append(y); ds.append(d); exs.append(ex); eys.append(ey); eds.append(ed)
    L.append("lvl_hx:\n        .word " + ",".join(map(str, xs))); L.append("lvl_hy:\n        .byte " + ",".join(map(str, ys))); L.append("lvl_hd:\n        .byte " + ",".join(map(str, ds)))
    L.append("lvl_ex:\n        .word " + ",".join(map(str, exs))); L.append("lvl_ey:\n        .byte " + ",".join(map(str, eys))); L.append("lvl_ed:\n        .byte " + ",".join(map(str, eds)))
    for i, f in enumerate(A.PATHS):
        b = A.script_bytes(f()); L.append(f"script{i+1}:\n        .byte " + ",".join(map(str, b)))
    L.append("script_lo:\n        .byte " + ",".join(f"<script{i+1}" for i in range(4))); L.append("script_hi:\n        .byte " + ",".join(f">script{i+1}" for i in range(4)))
    for i in range(4): L.append(f"txt_lv{i+1}:\n        .byte " + ",".join(map(str, text_codes(A.LEVEL_NAMES[i].ljust(25)[:25]))))
    L.append("txt_lv_lo:\n        .byte " + ",".join(f"<txt_lv{i+1}" for i in range(4))); L.append("txt_lv_hi:\n        .byte " + ",".join(f">txt_lv{i+1}" for i in range(4)))
    L.append("txt_hud:\n        .byte " + ",".join(map(str, text_codes("HUGO 000000   BILD 1/4   LEBEN"))))
    L.append("txt_prompt:\n        .byte " + ",".join(map(str, text_codes("DEMO - NICHT SPIELBAR"))))
    L.append(f".const CH_BLANK = {CH_BLANK}\n.const CH_FONT = {CH_FONT}\n.const CH_DIGIT0 = {CH_FONT + A.FONT_CHARS.index('0')}\n.const CH_STAR = {CH_FONT + A.FONT_CHARS.index('*')}")
    # Spielerdaten: je Spieler eine Tabelle (Bild * 21 + Zeile)
    hp = hero_pm()
    for k in range(4):
        L.append(f"hero_p{k}:")
        for f, fr in enumerate(hp): L.append("        .byte " + ",".join(str(b) for b in fr[k]))
    L.append("enemy_m:")
    for fr in enemy_pm(): L.append("        .byte " + ",".join(str(b) for b in fr))
    return "\n".join(L) + "\n"

# main: schreibt alle Editor-Projektdateien und lib/levels.inc
def main():
    import argparse
    _ap = argparse.ArgumentParser(
        description="""Erzeugt die Assets des Atari-Beispiels examples/atari/jumprun aus den gemeinsamen Kunstdaten (tools/jumprun_art.py).

Schreibt im Projektordner examples/atari/jumprun (UTF-8, vorhandene Dateien werden überschrieben, fehlende Ordner angelegt): Zeichensatz (graphics/chars.vcchar), je Bild eine Kachelkarte (graphics/lv1..lv4.vctiles), Titelbild (graphics/title.vcscreen), Song (music/tune.vcsong) und lib/levels.inc. Die JSON-Dateien sind normale Projektdateien der Editoren. Pfade werden relativ zum Ort des Skripts bestimmt; der Arbeitsordner ist egal.""",
        epilog="""Beispiele:
  python3 tools/gen_jumprun_atari.py
  cd tools && python3 gen_jumprun_atari.py

Das Werkzeug hat keine weiteren Optionen; -h/--help zeigt nur diesen Text und fuehrt nichts aus.""",
        formatter_class=argparse.RawDescriptionHelpFormatter)
    _ap.parse_args()
    w('graphics/chars.vcchar', charset())
    for i in range(4): w(f'graphics/lv{i+1}.vctiles', tilemap(i))
    w('graphics/title.vcscreen', title())
    w('music/tune.vcsong', song())
    os.makedirs(os.path.join(OUT, 'lib'), exist_ok=True)
    open(os.path.join(OUT, 'lib', 'levels.inc'), 'w', encoding='utf-8').write(inc())
    print('fertig:', OUT)
if __name__ == '__main__': main()
