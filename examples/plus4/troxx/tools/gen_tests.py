#!/usr/bin/env python3
# gen_tests.py - erzeugt troxx.vctest (Pruefungen und Routinentests) und troxx_kurz.vctest (sehr kurze Routinen) aus den Adressen in main.asm.
# Aufruf im Projektordner: python3 tools/gen_tests.py
# Die Testfaelle der gemeinsamen Logik (bcd_add, legal_mask, greedy, nb_boxes, box_add, paint, cell_type, fill_*) entsprechen denen der Referenzfassung
# (examples/vic20/troxx/troxx.vctest), mit den Massen des Plus/4-Gitters (8 x 5 Rechtecke, 33 x 21 Zellen) und der Schattenkopie (SHCH, SHCOL).
import json, re

src = open('main.asm').read()
A = {}
for m in re.finditer(r'^\.const (\w+)\s*=\s*\$([0-9a-fA-F]+)', src, re.M):
    A[m.group(1)] = int(m.group(2), 16)
for m in re.finditer(r'^\.const (\w+)\s*=\s*VARS(?:\s*\+\s*\$([0-9a-fA-F]+))?\s*(?:;|$)', src, re.M):
    A[m.group(1)] = 0x5000 + (int(m.group(2), 16) if m.group(2) else 0)
BOXCNT = A['BOXCNT']; NBOX = 40
SCREEN = 0x0c00; COLOR = 0x0800; SHCH = A['SHCH']; SHCOL = A['SHCOL']
COLS = 40; ROW0 = 2; COL0 = 3
GRID_W, GRID_H = 33, 21
COL_LINE, COL_DONE = A['COL_LINE'], A['COL_DONE']


def hx(v):
    return '$%x' % v


def zp(name):
    return hx(A[name])


def scr(gx, gy):
    return SCREEN + (gy + ROW0) * COLS + gx + COL0


def col(gx, gy):
    return COLOR + (gy + ROW0) * COLS + gx + COL0


def shch(gx, gy):
    return SHCH + gy * GRID_W + gx


def shcol(gx, gy):
    return SHCOL + gy * GRID_W + gx


def V(name, off=0):
    return hx(A[name] + off)


def T(n):
    return A['T%d' % n]


def test(name, call, setup=None, expect=None, **kw):
    t = {"name": name}
    if setup:
        t["setup"] = setup
    t["call"] = call
    t["expect"] = expect or []
    t["timeoutMs"] = 20000
    t.update(kw)
    return t


def mem(d):
    return {"mem": d}


tests = []
short = []

# ---- Zeichnen
tests.append(test("grid.draw: Ecke, Linie, T-Stueck, Innenraum (Zeichen und Farbe)", "grid.draw", None, [
    "[%s] == 131" % hx(scr(0, 0)), "[%s] == 128" % hx(scr(1, 0)), "[%s] == 135" % hx(scr(4, 0)),
    "[%s] == 132" % hx(scr(32, 0)), "[%s] == 129" % hx(scr(0, 1)), "[%s] == 32" % hx(scr(1, 1)),
    "[%s] == %d" % (hx(col(1, 0)), COL_LINE), "[%s] == %d" % (hx(col(0, 1)), COL_LINE),
    "[%s] == 131" % hx(shch(0, 0)), "[%s] == %d" % (hx(shcol(1, 0)), COL_LINE), "[%s] == 132" % hx(shch(32, 0))]))
tests.append(test("grid.draw: Kreuzung (gx 4, gy 4), Unterkante, Randspalten", "grid.draw", None, [
    "[%s] == 130" % hx(scr(4, 4)), "[%s] == 133" % hx(scr(0, 20)), "[%s] == 134" % hx(scr(32, 20)),
    "[%s] == 136" % hx(scr(4, 20)), "[%s] == 137" % hx(scr(0, 4)), "[%s] == 138" % hx(scr(32, 4)),
    "[%s] == 129" % hx(scr(32, 5)), "[%s] == 129" % hx(scr(32, 19))]))

# ---- cell_type (Typ in A)
for (gx, gy, typ, nm) in [(0, 0, 3, "Ecke"), (4, 4, 3, "Knoten innen"), (32, 20, 3, "Ecke unten rechts"), (1, 0, 1, "waagerecht oben"),
                          (13, 8, 1, "waagerecht Mitte"), (0, 1, 2, "senkrecht links"), (20, 18, 2, "senkrecht Mitte"), (1, 1, 0, "Innenraum"),
                          (31, 19, 0, "Innenraum unten rechts"), (5, 2, 0, "Innenraum Mitte")]:
    short.append(test("cell_type %s (%d, %d) = %d" % (nm, gx, gy, typ), "grid.cell_type",
                      mem({zp('T0'): gx, zp('T1'): gy}), ["a == %d" % typ]))

# ---- nb_boxes: alle 93 Segmente (48 waagerecht, 45 senkrecht): NB0 = unten/rechts, NB1 = oben/links ($FF = keins)
segs = []
for r in range(6):
    for c in range(8):
        segs.append(("H", r, c, 4 * c + 1, 4 * r, 1, (r * 8 + c) if r < 5 else 255, (r - 1) * 8 + c if r > 0 else 255))
for r in range(5):
    for c in range(9):
        segs.append(("V", r, c, 4 * c, 4 * r + 1, 2, (r * 8 + c) if c < 8 else 255, (r * 8 + c - 1) if c > 0 else 255))
for i, (k, r, c, gx, gy, typ, n0, n1) in enumerate(segs):
    short.append(test("nb_boxes Segment %d %s(%d,%d): NB0 %d, NB1 %d" % (i, k, r, c, n0, n1), "grid.nb_boxes",
                      mem({zp('T0'): gx, zp('T1'): gy, zp('T2'): typ}), ["[%s] == %d" % (V('NB0'), n0), "[%s] == %d" % (V('NB1'), n1)]))

# ---- box_add: vierte Kante merkt das Rechteck genau einmal vor (100 Punkte)
sc = [0, 0, 0, 0]
tests.append(test("box_add: vierte Kante: vorgemerkt, 100 Punkte, BOXES_DONE + 1", "grid.box_add",
                  dict(x=7, **mem({zp('FILLS_NOW'): 0, zp('BOXES_DONE'): 0, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3,
                                   hx(BOXCNT): [0] * 7 + [3] + [0] * 32})),
                  ["[%s] == 4" % hx(BOXCNT + 7), "[%s] == 1" % zp('FILLS_NOW'), "[%s] == 1" % zp('BOXES_DONE'), "[%s] == 7" % zp('FILLQ'),
                   "[%s] == 0" % zp('SCORE'), "[%s] == 1" % hx(A['SCORE'] + 1)]))
short.append(test("box_add: dritte Kante: nichts vorgemerkt", "grid.box_add",
                  dict(x=7, **mem({zp('FILLS_NOW'): 0, zp('BOXES_DONE'): 0, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3,
                                   hx(BOXCNT): [0] * 7 + [2] + [0] * 32})),
                  ["[%s] == 3" % hx(BOXCNT + 7), "[%s] == 0" % zp('FILLS_NOW'), "[%s] == 0" % zp('BOXES_DONE')]))

# ---- paint: Kettenreaktion (zwei Rechtecke), ein Rechteck, schon gelb
cells = {}
for gy in (5, 6):
    cells[shcol(4, gy)] = COL_DONE        # V(1,1): Zellen (4,5), (4,6) gelb, (4,7) wird gefaerbt
cells[shcol(4, 7)] = COL_LINE
bc = [0] * NBOX
bc[8] = 3
bc[9] = 3
base = {zp('T0'): 4, zp('T1'): 7, zp('FILLS_NOW'): 0, zp('BOXES_DONE'): 0, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3, hx(BOXCNT): bc}
for k, v in cells.items():
    base[hx(k)] = v
tests.append(test("chain_two (paint): letztes Zeichen von V(1,1) schliesst zwei Rechtecke: 411 Punkte, beide vorgemerkt", "grid.paint", mem(base), [
    "[%s] == 2" % zp('FILLS_NOW'), "[%s] == 2" % zp('BOXES_DONE'), "[%s] == 9" % zp('FILLQ'), "[%s] == 8" % hx(A['FILLQ'] + 1),
    "[%s] == $11" % zp('SCORE'), "[%s] == $04" % hx(A['SCORE'] + 1), "[%s] == 0" % hx(A['SCORE'] + 2), "[%s] == %d" % (hx(shcol(4, 7)), COL_DONE), "[%s] == %d" % (hx(col(4, 7)), COL_DONE)]))
b2 = dict(base)
bc2 = [0] * NBOX
bc2[9] = 3
b2[hx(BOXCNT)] = bc2
tests.append(test("paint: Segment schliesst nur ein Rechteck (kein Kettenbonus): 111 Punkte", "grid.paint", mem(b2), [
    "[%s] == 1" % zp('FILLS_NOW'), "[%s] == $11" % zp('SCORE'), "[%s] == $01" % hx(A['SCORE'] + 1), "[%s] == 1" % hx(BOXCNT + 8)]))
b3 = dict(base)
b3[hx(shcol(4, 7))] = COL_DONE
tests.append(test("paint: Zelle schon gelb: nichts passiert", "grid.paint", mem(b3), ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % zp('FILLS_NOW')]))
b4 = dict(base)
b4[zp('T0')] = 1
b4[zp('T1')] = 1
short.append(test("paint: Innenraum und Knoten tun nichts", "grid.paint", mem(b4), ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % zp('FILLS_NOW')]))
b5 = dict(base)
b5[zp('T0')] = 4
b5[zp('T1')] = 4
short.append(test("paint: Knoten (4, 4) tut nichts", "grid.paint", mem(b5), ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % zp('FILLS_NOW')]))
# waagerechtes Segment: H(0,0) Zelle (3,0) schliesst das Rechteck B(0,0) (obere Kante); Rand: kein Rechteck oberhalb
hb = {zp('T0'): 3, zp('T1'): 0, zp('FILLS_NOW'): 0, zp('BOXES_DONE'): 0, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3,
      hx(BOXCNT): [3] + [0] * 39, hx(shcol(1, 0)): COL_DONE, hx(shcol(2, 0)): COL_DONE, hx(shcol(3, 0)): COL_LINE}
tests.append(test("paint: H(0,0) schliesst B(0,0) am oberen Rand: 111 Punkte", "grid.paint", mem(hb), [
    "[%s] == 1" % zp('FILLS_NOW'), "[%s] == 0" % zp('FILLQ'), "[%s] == $11" % zp('SCORE'), "[%s] == $01" % hx(A['SCORE'] + 1)]))

# ---- bcd_add
short.append(test("bcd_add: Uebertrag ueber alle Stellen", "hud.bcd_add", dict(a=153, x=0, **mem({zp('SCORE'): [1, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3})),
                  ["[%s] == 0" % zp('SCORE'), "[%s] == 1" % hx(A['SCORE'] + 1), "[%s] == 3" % zp('LIVES')]))
short.append(test("bcd_add: Extraleben bei 10000 Punkten", "hud.bcd_add", dict(a=80, x=0, **mem({zp('SCORE'): [80, 153, 9], zp('SCORE_OVF'): 0, zp('LIVES'): 3})),
                  ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % hx(A['SCORE'] + 1), "[%s] == $10" % hx(A['SCORE'] + 2), "[%s] == 4" % zp('LIVES')]))
short.append(test("bcd_add: 9999 + 1 = 10000 Punkte: ein Extraleben", "hud.bcd_add", dict(a=1, x=0, **mem({zp('SCORE'): [0x99, 0x99, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3})),
                  ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % hx(A['SCORE'] + 1), "[%s] == 1" % hx(A['SCORE'] + 2), "[%s] == 4" % zp('LIVES')]))
short.append(test("bcd_add: 9998 + 1 = 9999 Punkte: kein Extraleben", "hud.bcd_add", dict(a=1, x=0, **mem({zp('SCORE'): [0x98, 0x99, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3})),
                  ["[%s] == $99" % zp('SCORE'), "[%s] == 3" % zp('LIVES')]))
short.append(test("bcd_add: 9999 + 1 bei 9 Leben: bleibt 9", "hud.bcd_add", dict(a=1, x=0, **mem({zp('SCORE'): [0x99, 0x99, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 9})),
                  ["[%s] == 1" % hx(A['SCORE'] + 2), "[%s] == 9" % zp('LIVES')]))
short.append(test("bcd_add: Leben hoechstens 9", "hud.bcd_add", dict(a=80, x=0, **mem({zp('SCORE'): [80, 153, 9], zp('SCORE_OVF'): 0, zp('LIVES'): 9})),
                  ["[%s] == 9" % zp('LIVES')]))
short.append(test("bcd_add: 500 Punkte (X = 1)", "hud.bcd_add", dict(a=5, x=1, **mem({zp('SCORE'): [32, 1, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3})),
                  ["[%s] == $20" % zp('SCORE'), "[%s] == $06" % hx(A['SCORE'] + 1)]))

# ---- legal_mask (Bit 0 hoch, 1 runter, 2 links, 3 rechts)
def lm(nm, gx, gy, d, exp):
    tests.append(test("legal_mask: " + nm, "grid.legal_mask", mem({zp('T4'): gx, zp('T5'): gy, zp('T6'): d}), ["a == %d" % exp]))
lm("Knoten (4, 4), Richtung runter: hoch ausgeschlossen = links, rechts, runter", 4, 4, 1, 14)
lm("Ecke (0, 0), Richtung hoch: runter ausgeschlossen = nur rechts", 0, 0, 0, 8)
lm("Ecke (0, 0), Richtung rechts: runter und rechts", 0, 0, 3, 10)
lm("Ecke (32, 20), Richtung links: hoch, links (Umkehr rechts liegt ausserhalb)", 32, 20, 2, 5)
lm("Ecke (32, 0), Richtung hoch: nur links (runter ist die Umkehr)", 32, 0, 0, 4)
lm("Randknoten (16, 0), Richtung rechts: links ausgeschlossen = runter, rechts", 16, 0, 3, 10)

# ---- greedy (X = Richtung)
def gr(nm, gx, gy, mask, px, py, x):
    tests.append(test("greedy: " + nm, "enemies.greedy", mem({zp('T4'): gx, zp('T5'): gy, zp('T7'): mask, zp('PX'): px, zp('PY'): py}), ["x == %d" % x]))
gr("Gegner (4, 4), Figur (12, 4): rechts", 4, 4, 15, 12, 4, 3)
gr("Gegner (8, 8), Figur (8, 0): hoch", 8, 8, 15, 8, 0, 0)
gr("nur erlaubte Richtungen (Maske links/runter), Figur rechts unten: die bessere der erlaubten", 16, 8, 6, 32, 20, 1)
gr("Gegner (16, 12), Figur (0, 12): links", 16, 12, 15, 0, 12, 2)

# ---- Zufall (LFSR, Rueckkopplung $B8)
for st, nxt in [(1, 0xb8), (0xb8, 0x5c), (0x5c, 0x2e), (0x80, 0x40), (0xff, 0xc7)]:
    short.append(test("rnd: LFSR $%02X -> $%02X" % (st, nxt), "enemies.rnd", mem({zp('RND'): st}), ["a == $%x" % nxt, "[%s] == $%x" % (zp('RND'), nxt)]))
for mask in (1, 2, 4, 8, 6, 9, 15):
    short.append(test("pick_random: Maske %d liefert eine Richtung aus der Maske" % mask, "enemies.pick_random",
                      mem({zp('T7'): mask, zp('RND'): 0x5a}), ["x < 4"]))

# ---- level_idx, set_tempo
for lv, idx in [(1, 0), (5, 4), (9, 8), (0x10, 8), (0x25, 8)]:
    short.append(test("level_idx: Stufe BCD $%02X -> Index %d" % (lv, idx), "enemies.level_idx", mem({zp('LEVEL'): lv}), ["y == %d" % idx]))

# ---- Gegner platzieren: Zahl nach Stufe, Typen, Startknoten
for lv, n in [(1, 1), (2, 2), (4, 3), (7, 4)]:
    tests.append(test("enemies.place_all: Stufe %d -> %d Gegner" % (lv, n), "enemies.place_all", mem({zp('LEVEL'): lv}),
                      ["[%s] == %d" % (zp('NENEMY'), n), "[%s] == %d" % (zp('ELIM'), 6 * n), "[%s] == 32" % zp('EN'), "[%s] == 0" % hx(A['EN'] + 1),
                       "[%s] == 0" % hx(A['EN'] + 3)]))

# ---- find
short.append(test("enemies.find: Gegner auf (7, 8) gefunden, X = Abstand", "enemies.find",
                  dict(x=7, y=8, **mem({zp('ELIM'): 12, zp('EN'): [1, 1, 0, 0, 1, 1, 7, 8, 0, 1, 7, 8]})), ["fc == 1", "x == 6"]))
short.append(test("enemies.find: keiner auf (3, 3)", "enemies.find",
                  dict(x=3, y=3, **mem({zp('ELIM'): 12, zp('EN'): [1, 1, 0, 0, 1, 1, 7, 8, 0, 1, 7, 8]})), ["fc == 0"]))

# ---- player.step
st = {zp('PX'): 0, zp('PY'): 20, zp('PWANT'): 1, zp('PANIM'): 0, zp('PINV'): 0, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3, zp('HITF'): 0,
      zp('FILLS_NOW'): 0, hx(BOXCNT): [0] * 40, zp('ELIM'): 6, zp('EN'): [255, 255, 0, 0, 255, 255], hx(shcol(0, 19)): COL_LINE, hx(shcol(0, 20)): COL_LINE,
      zp('PDIR'): 0, V('IT_KIND'): 0}
tests.append(test("player.step: von (0, 20) hoch nach (0, 19): faerbt (Schattenkopie und Bildschirm), 1 Punkt", "player.step", mem(st), [
    "[%s] == 19" % zp('PY'), "[%s] == 0" % zp('PX'), "[%s] == 1" % zp('SCORE'), "[%s] == %d" % (hx(shcol(0, 19)), COL_DONE),
    "[%s] == %d" % (hx(col(0, 19)), COL_DONE), "[%s] == 1" % zp('PANIM'), "[%s] == 0" % zp('PDIR'),
    "[%s] == %d" % (hx(shcol(0, 20)), COL_DONE), "[%s] == %d" % (hx(col(0, 20)), COL_DONE)]))
s2 = dict(st); s2[zp('PWANT')] = 8; s2[zp('PX')] = 1; s2[zp('PY')] = 20
tests.append(test("player.step: waagerecht rechts von (1, 20) nach (2, 20)", "player.step", mem(s2), ["[%s] == 2" % zp('PX'), "[%s] == 20" % zp('PY')]))
s3 = dict(st); s3[zp('PWANT')] = 1; s3[zp('PX')] = 1; s3[zp('PY')] = 20
tests.append(test("player.step: auf einer waagerechten Linie ist hoch gesperrt", "player.step", mem(s3), ["[%s] == 20" % zp('PY'), "[%s] == 1" % zp('PX')]))
s4 = dict(st); s4[zp('PWANT')] = 1; s4[zp('PX')] = 0; s4[zp('PY')] = 0
tests.append(test("player.step: ausserhalb des Gitters kein Schritt (Ecke 0, 0, hoch)", "player.step", mem(s4), ["[%s] == 0" % zp('PY')]))
s5 = dict(st); s5[zp('EN')] = [0, 19, 0, 0, 0, 19]
tests.append(test("player.step: Gegner auf der Zielzelle = Treffer, Figur bleibt", "player.step", mem(s5), ["[%s] == 1" % zp('HITF'), "[%s] == 20" % zp('PY')]))
s6 = dict(s5); s6[zp('PINV')] = 5
tests.append(test("player.step: Gegner auf der Zielzelle waehrend der Unverwundbarkeit: kein Treffer, Figur bleibt", "player.step", mem(s6),
                  ["[%s] == 0" % zp('HITF'), "[%s] == 20" % zp('PY'), "[%s] == 4" % zp('PINV')]))
s7 = dict(st); s7[zp('PWANT')] = 0
tests.append(test("player.step: keine Richtung gehalten: stehen bleiben", "player.step", mem(s7),
                  ["[%s] == 20" % zp('PY'), "[%s] == 0" % zp('SCORE')]))

# ---- Fuellen: Zeichen 161 + 9 * ((r + c) mod 4) + 3 * Zeile + Spalte im Block, Farbe (3 + ((r + c) und 1) + Zeile) * 16 + Farbton + 8
def fcol(r, c, k):
    return ((3 + ((r + c) & 1) + k) << 4) | [5, 3, 4, 2, 7][r] | 8


for (r, c) in [(0, 1), (4, 7), (2, 3)]:
    ex = []
    for dy in range(3):
        for dx in range(3):
            ch = 161 + 9 * ((r + c) & 3) + 3 * dy + dx
            ex.append("[%s] == %d" % (hx(scr(4 * c + 1 + dx, 4 * r + 1 + dy)), ch))
            ex.append("[%s] == %d" % (hx(shch(4 * c + 1 + dx, 4 * r + 1 + dy)), ch))
        ex.append("[%s] == %d" % (hx(col(4 * c + 1, 4 * r + 1 + dy)), fcol(r, c, dy)))
        ex.append("[%s] == %d" % (hx(shcol(4 * c + 3, 4 * r + 1 + dy)), fcol(r, c, dy)))
    ex.append("[%s] != %d" % (hx(scr(4 * c + 4, 4 * r + 1)), 161 + 9 * ((r + c) & 3)))
    tests.append(test("grid.fill_box: Rechteck (%d, %d): Zeichen, Lage im Block und Farbverlauf (Bildschirm und Schattenkopie)" % (r, c), "grid.fill_box", dict(x=r, y=c, **mem({V('FILLGRAY'): 0})), ex))
ch39 = 161 + 9 * ((4 + 7) & 3) + 3 * 2
tests.append(test("grid.fill_row_b: Rechteck 39, Zeile 2: Zeile gy 19", "grid.fill_row_b", dict(a=39, x=2, **mem({V('FILLGRAY'): 0})),
                  ["[%s] == %d" % (hx(scr(29, 19)), ch39), "[%s] == %d" % (hx(scr(31, 19)), ch39 + 2), "[%s] != %d" % (hx(scr(29, 20)), ch39)]))
short.append(test("grid.fill_colour: Rechteck (1, 2), Zeile 1: Helligkeit 3 + 1 + 1, Farbton 3 (cyan), Mehrfarben-Bit", "grid.fill_colour", dict(a=1, x=1, y=2, **mem({V('FILLGRAY'): 0})),
                  ["a == %d" % fcol(1, 2, 1)]))
short.append(test("grid.fill_colour: graue Hindernisbloecke haben eine feste Farbe", "grid.fill_colour", dict(a=1, x=1, y=2, **mem({V('FILLGRAY'): 1})),
                  ["a == %d" % (A['COL_GRAY'] | 8)]))

# ---- zweites Layout: prefill (sechs Rechtecke grau, Kanten gelb, BOXCNT neu gezaehlt)
tests.append(test("grid.prefill: sechs Hindernisbloecke, Kanten gelb, BOXES_DONE 6, BOXCNT der Nachbarn", "grid.prefill", mem({zp('BOXES_DONE'): 0, hx(BOXCNT): [0] * 40, V('FILLGRAY'): 0, hx(SHCOL): [COL_LINE] * 693}),
                  ["[%s] == 6" % zp('BOXES_DONE'), "[%s] == 4" % hx(BOXCNT + 9), "[%s] == 4" % hx(BOXCNT + 30), "[%s] == 4" % hx(BOXCNT + 19), "[%s] == 4" % hx(BOXCNT + 20),
                   "[%s] == 1" % hx(BOXCNT + 8), "[%s] == 1" % hx(BOXCNT + 10), "[%s] == 0" % hx(BOXCNT + 0), "[%s] == 1" % hx(BOXCNT + 18),
                   "[%s] == %d" % (hx(shcol(8, 6)), COL_DONE), "[%s] == %d" % (hx(shcol(5, 4)), COL_DONE), "[%s] == 0" % V('FILLGRAY'),
                   "[%s] == %d" % (hx(shcol(6, 5)), A['COL_GRAY'] | 8)], timeoutMs=60000))

# ---- Tempo
for sp, pal, v in [(1, 1, 12), (9, 1, 3), (5, 1, 7), (1, 0, 14), (9, 0, 4), (5, 0, 8)]:
    short.append(test("set_tempo: Tempo %d %s -> %d Bilder" % (sp, "PAL" if pal else "NTSC", v), "set_tempo", mem({zp('SPEED'): sp, zp('FLAGS'): pal}), ["[%s] == %d" % (zp('STEPLEN'), v)]))

# ---- Gegenstaende
if True:
    pk = {zp('PX'): 9, zp('PY'): 8, V('IT_KIND'): 1, V('IT_X'): 9, V('IT_Y'): 8, V('FPS'): 50, zp('RND'): 0x33, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3,
          zp('FLAGS'): 1}
    tests.append(test("items.check: Stern aufgenommen: 50 Punkte, Gegenstand weg", "items.check", mem(pk), ["[%s] == $50" % zp('SCORE'), "[%s] == 0" % V('IT_KIND')]))
    pk2 = dict(pk); pk2[V('IT_KIND')] = 2
    tests.append(test("items.check: Eisblock aufgenommen: Starre 250 Bilder (PAL), Gegenstand weg", "items.check", mem(pk2),
                      ["[%s] == 250" % V('FREEZE'), "[%s] == 0" % V('FREEZE', 1), "[%s] == 0" % V('IT_KIND')]))
    pk3 = dict(pk); pk3[zp('PX')] = 10
    short.append(test("items.check: Figur nicht auf dem Gegenstand: nichts passiert", "items.check", mem(pk3), ["[%s] == 0" % zp('SCORE'), "[%s] == 1" % V('IT_KIND')]))
    pk4 = dict(pk2); pk4[zp('FLAGS')] = 0
    tests.append(test("items.check: Eisblock, NTSC: Starre 300 Bilder", "items.check", mem(pk4), ["[%s] == 44" % V('FREEZE'), "[%s] == 1" % V('FREEZE', 1)]))

# ---- Spielerwechsel (zwei Spieler)
ps = A['PSAVE']
sw = {V('NPLAYERS'): 2, zp('CUR'): 0, V('PFRESH'): [0, 0], zp('LIVES'): 2, zp('LEVEL'): 3, zp('SCORE'): [1, 2, 3], zp('BOXES_DONE'): 5,
      hx(ps): [0] * 48, hx(ps + 48): [5, 1, 3, 0, 0, 0, 0, 0]}
tests.append(test("switch_player: Spieler 1 -> 2: Block gesichert, geladen, STATE = levelinit", "switch_player", mem(sw),
                  ["fc == 1", "[%s] == 1" % zp('CUR'), "[%s] == 1" % zp('STATE'), "[%s] == 5" % zp('SPEED'), "[%s] == 3" % zp('LIVES'), "[%s] == 2" % hx(ps + 2),
                   "[%s] == 3" % hx(ps + 1), "[%s] == 1" % hx(ps + 3), "[%s] == 1" % V('RESTORE')], timeoutMs=60000))
sw2 = dict(sw); sw2[hx(ps + 48)] = [5, 1, 0, 0, 0, 0, 0, 0]
short.append(test("switch_player: anderer Spieler ohne Leben: kein Wechsel", "switch_player", mem(sw2), ["fc == 0", "[%s] == 0" % zp('CUR')]))
sw3 = dict(sw); sw3[V('NPLAYERS')] = 1
short.append(test("switch_player: ein Spieler: kein Wechsel", "switch_player", mem(sw3), ["fc == 0", "[%s] == 0" % zp('CUR')]))

# ---- Eingabe: die Joystick-Leitungen lassen sich im Testlauf nicht setzen (Tastaturmatrix); getestet wird die Zuordnung der Anschluesse
for jm, cur, port in [(0, 0, 2), (0, 1, 2), (1, 0, 1), (1, 1, 1), (2, 0, 1), (2, 1, 2)]:
    short.append(test("input.port_of: JOYMODE %d, Spieler %d -> Anschluss %d" % (jm, cur + 1, port), "input.port_of", mem({V('JOYMODE'): jm, zp('CUR'): cur}), ["x == %d" % port]))

# ---- Titelmenue
for nm, row, pl, jm, d, exp in [("Spieler 1 -> 2", 0, 1, 0, 1, (V("NPLAYERS"), 2)), ("Spieler 2 -> 1 setzt zwei Joysticks zurueck", 0, 2, 2, 255, (V("JOYMODE"), 0)),
                                ("Joystick 0 -> 1 mit einem Spieler", 1, 1, 0, 1, (V("JOYMODE"), 1)), ("Joystick 1 -> 2 mit einem Spieler gesperrt", 1, 1, 1, 1, (V("JOYMODE"), 1)),
                                ("Joystick 1 -> 2 mit zwei Spielern", 1, 2, 1, 1, (V("JOYMODE"), 2)), ("Joystick 0 nach links bleibt 0", 1, 2, 0, 255, (V("JOYMODE"), 0))]:
    short.append(test("title_change: " + nm, "title_change", dict(a=d, **mem({V("SELROW"): row, V("NPLAYERS"): pl, V("JOYMODE"): jm})), ["[%s] == %d" % exp]))

# ---- Bestenliste
HB = A['HS_BUF']
tests.append(test("hiscore.default: Kennung, erster Platz 10000 Punkte, letzter 1000, Pruefsumme stimmt", "hiscore.default", None, [
    "[%s] == $54" % hx(HB), "[%s] == 1" % hx(HB + 4), "[%s] == 1" % hx(HB + 10), "[%s] == 0" % hx(HB + 9), "[%s] == $10" % hx(HB + 5 + 9 * 7 + 4), "[%s] == 20" % hx(HB + 5)]))
for nm, sc, pl in [("ueber dem ersten Platz (10001)", [1, 0, 1], 0), ("zwischen Platz 3 und 4 (7500)", [0, 0x75, 0], 3), ("genau gleich dem letzten (1000): nicht in der Liste", [0, 0x10, 0], 0xff),
                   ("unter dem letzten (500)", [0, 5, 0], 0xff), ("gleich Platz 1 (10000): dahinter", [0, 0, 1], 1), ("knapp ueber dem letzten (1001)", [1, 0x10, 0], 9)]:
    lst = [0x54, 0x58, 0x48, 0x53, 1]
    mid = [0x00, 0x90, 0x80, 0x70, 0x60, 0x50, 0x40, 0x30, 0x20, 0x10]
    hi = [1] + [0] * 9
    for i in range(10):
        lst += [20, 18, 24, 0, mid[i], hi[i], 1]
    lst += [0, 0]
    short.append(test("hiscore.place: Endstand %s -> Platz %s" % (nm, "keiner" if pl == 0xff else pl + 1), "hiscore.place", mem({V('HS_SC'): sc, hx(HB): lst}), ["x == %d" % pl]))

# ---- Klangmodul (Zustand; die TED-Register zeigen nur den zuletzt geschriebenen Wert)
SN = A['SND']
tests.append(test("sound.play: Segment-Ton startet Stimme 1 (Prioritaet 2)", "sound.play", dict(a=1, **mem({hx(SN): [0] * 32, zp('FLAGS'): 1})),
                  ["[%s] == 2" % hx(SN + 6), "[%s] == 1" % hx(SN + 4), "[%s] == 0" % hx(SN + 7)]))
tests.append(test("sound.play: gleich wichtiger laufender Effekt wird nicht ersetzt", "sound.play", dict(a=1, **mem({hx(SN): [0] * 6 + [2] + [0] * 25, zp('FLAGS'): 1})),
                  ["[%s] == 2" % hx(SN + 6), "[%s] == 0" % hx(SN + 4)]))
tests.append(test("sound.play: wichtigerer Effekt (Treffer, Prioritaet 5) ersetzt einen Schritt-Klick auf beiden Stimmen", "sound.play", dict(a=4, **mem({hx(SN): [0] * 6 + [1, 1] + [0] * 24, zp('FLAGS'): 1})),
                  ["[%s] == 5" % hx(SN + 6), "[%s] == 5" % hx(SN + 7), "[%s] == 1" % hx(SN + 13)]))
short.append(test("sound.play: waehrend der Titelmusik keine Effekte", "sound.play", dict(a=4, **mem({hx(SN): [0] * 15 + [1] + [0] * 16, zp('FLAGS'): 1})),
                  ["[%s] == 0" % hx(SN + 6)]))
tests.append(test("sound.music_start: beide Stimmen mit Prioritaet 255, Musik-Flag", "sound.music_start", mem({hx(SN): [0] * 32, zp('FLAGS'): 1}),
                  ["[%s] == 255" % hx(SN + 6), "[%s] == 255" % hx(SN + 7), "[%s] == 1" % hx(SN + 15)]))


# ---- Regel 16.2 (Gegner zerstoeren Linien): enemies.one mit einem Gegner, der ein Innenzeichen betritt; mit ENEMY_DESTROYS_LINES = 1 (troxx.vctest) und 0 (troxx_regel0.vctest)
def destroy_tests(rule):
    out = []
    pre = "" if rule else "(Regel aus) "

    def case(nm, en, cells, box, score, exp_cells, exp_box, exp_score, exp_x, call="enemies.one", freeze=0, sound=None):
        m = {zp('EN'): en, zp('ELIM'): 6, zp('NENEMY'): 1, zp('ECUR'): 0, zp('HITF'): 0, zp('PX'): 20, zp('PY'): 20, zp('PINV'): 0, zp('RND'): 0x5a,
             zp('SCORE'): score, zp('SCORE_OVF'): 0, zp('LIVES'): 3, zp('FILLS_NOW'): 0, V('FREEZE'): [freeze, 0], V('FIGOFF'): 1, V('FILLGRAY'): 0,
             zp('EPH'): 0, hx(BOXCNT): box, hx(SHCOL): [COL_LINE] * 693}
        for (gx, gy) in cells:
            m[hx(shcol(gx, gy))] = COL_DONE
        ex = ["[%s] == %d" % (zp('EN'), exp_x)]
        for (gx, gy) in exp_cells:
            ex.append("[%s] == %d" % (hx(shcol(gx, gy)), COL_DONE))
        for (gx, gy) in cells:
            if (gx, gy) not in exp_cells:
                ex.append("[%s] == %d" % (hx(shcol(gx, gy)), COL_LINE))
                ex.append("[%s] == %d" % (hx(scr(gx, gy)), TPLCH(gx, gy)))
        for (b, v) in exp_box:
            ex.append("[%s] == %d" % (hx(BOXCNT + b), v))
        ex.append("[%s] == $%02x" % (zp('SCORE'), exp_score[0]))
        ex.append("[%s] == $%02x" % (hx(A['SCORE'] + 1), exp_score[1]))
        out.append(test(pre + nm, call, mem(m), ex, timeoutMs=30000))

    def TPLCH(gx, gy):
        return 128 if gy % 4 == 0 else 129

    boxes0 = [0] * 40
    def bx(**kw):
        b = list(boxes0)
        for k, v in kw.items():
            b[int(k[1:])] = v
        return b
    H = [(1, 0), (2, 0), (3, 0)]
    en = [1, 0, 3, 0, 0, 0]
    def keep(c):          # Regel aus: alles bleibt
        return c
    cells_part = [(1, 0), (2, 0)]
    case("Gegner betritt teilweise gefaerbtes Segment: Zeichen zurueck, 2 Punkte ab", en, cells_part, bx(), [5, 0, 0],
         [] if rule else cells_part, [(0, 0)], (3, 0) if rule else (5, 0), 2)
    case("Gegner betritt abgeschlossenes Segment ohne gefuelltes Nachbarrechteck: BOXCNT - 1, 13 Punkte ab", en, H, bx(b0=1), [0x20, 0, 0],
         [] if rule else H, [(0, 0 if rule else 1)], (7, 0) if rule else (0x20, 0), 2)
    case("geschuetztes Segment (angrenzendes Rechteck gefuellt) bleibt", en, H, bx(b0=4), [0x20, 0, 0], H, [(0, 4)], (0x20, 0), 2)
    case("Randsegment (nur ein Nachbarrechteck, unten): BOXCNT des oberen Rechtecks - 1", [1, 20, 3, 0, 0, 0], [(1, 20), (2, 20), (3, 20)], bx(b32=1), [0x20, 0, 0],
         [] if rule else [(1, 20), (2, 20), (3, 20)], [(32, 0 if rule else 1)], (7, 0) if rule else (0x20, 0), 2)
    V2 = [(4, 5), (4, 6), (4, 7)]
    case("Segment mit zwei angrenzenden Rechtecken: beide BOXCNT - 1", [4, 5, 1, 0, 0, 0], V2, bx(b8=1, b9=1), [0x20, 0, 0],
         [] if rule else V2, [(8, 0 if rule else 1), (9, 0 if rule else 1)], (7, 0) if rule else (0x20, 0), 4)
    case("Punktuntergrenze 0: nie unter 0", en, H, bx(b0=1), [5, 0, 0], [] if rule else H, [(0, 0 if rule else 1)], (0, 0) if rule else (5, 0), 2)
    case("ungefaerbtes Segment: nichts passiert", en, [], bx(), [5, 0, 0], [], [(0, 0)], (5, 0), 2)
    if not rule:
        case("Knotenzeichen zerstoert nichts (Gegner laeuft in den Knoten (4, 0))", [3, 0, 3, 0, 0, 0], H, bx(b0=1), [0x20, 0, 0], H, [(0, 1)], (0x20, 0), 4)
    if rule:
        case("eingefrorener Gegner zerstoert nichts (enemies.step)", en, H, bx(b0=1), [0x20, 0, 0], H, [(0, 1)], (0x20, 0), 1, call="enemies.step", freeze=100)
    return out



# ---- Knotenregel: Gegner betritt einen Knoten (alle dort endenden freien Segmente), freistehende gelbe Knoten werden blau
def node_cases(rule):
    pre = "" if rule else "(Regel aus) "
    out = []
    cells = [(1, 0), (2, 0), (6, 0), (4, 1), (4, 2), (4, 3)]
    m = {zp('EN'): [3, 0, 3, 0, 0, 0], zp('ELIM'): 6, zp('NENEMY'): 1, zp('ECUR'): 0, zp('HITF'): 0, zp('PX'): 20, zp('PY'): 18, zp('PINV'): 0, zp('RND'): 0x5a,
         zp('SCORE'): [0x30, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3, zp('FILLS_NOW'): 0, V('FREEZE'): [0, 0], V('FIGOFF'): 1, V('FILLGRAY'): 0,
         zp('EPH'): 0, hx(BOXCNT): [1, 1] + [0] * 38, hx(SHCOL): [COL_LINE] * 693}
    for (gx, gy) in cells + [(4, 0)]:
        m[hx(shcol(gx, gy))] = COL_DONE
    ex = ["[%s] == 4" % zp('EN'), "[%s] == %d" % (hx(shcol(4, 0)), COL_DONE)]
    for (gx, gy) in cells:
        ex.append("[%s] == %d" % (hx(shcol(gx, gy)), COL_LINE if rule else COL_DONE))
    ex += ["[%s] == %d" % (hx(BOXCNT), 0 if rule else 1), "[%s] == %d" % (hx(BOXCNT + 1), 0 if rule else 1), "[%s] == $%02x" % (zp('SCORE'), 0x14 if rule else 0x30)]
    out.append(test(pre + "Gegner betritt den Knoten (4, 0): alle drei dort endenden Segmente zerstoert (2 + 1 + 13 Punkte), der Knoten bleibt gelb", "enemies.one", mem(m), ex, timeoutMs=30000))
    return out


def nodes_free_cases():
    out = []
    def nf(nm, node, extra, stay, en=None, pxy=(255, 255)):
        m = {zp('T4'): 255, zp('T5'): 255, zp('PX'): pxy[0], zp('PY'): pxy[1], zp('ELIM'): 6, zp('EN'): en or [255, 255, 0, 0, 0, 0], hx(SHCOL): [COL_LINE] * 693, hx(SHCH): [32] * 693}
        m[hx(shcol(*node))] = COL_DONE
        for (gx, gy, c, ch) in extra:
            if c is not None:
                m[hx(shcol(gx, gy))] = c
            if ch is not None:
                m[hx(shch(gx, gy))] = ch
        out.append(test("nodes_free: " + nm, "grid.nodes_free", mem(m), ["[%s] == %d" % (hx(shcol(*node)), COL_DONE if stay else COL_LINE)], timeoutMs=30000))
    nf("freistehender gelber Knoten (8, 8) wird blau", (8, 8), [], False)
    nf("Knoten mit gelber Nachbarzelle (9, 8) bleibt", (8, 8), [(9, 8, COL_DONE, None)], True)
    nf("Knoten mit gefuelltem Nachbarrechteck (Fuellzeichen auf (9, 9)) bleibt", (8, 8), [(9, 9, None, 161)], True)
    nf("Knoten mit Figur bleibt", (8, 8), [], True, pxy=(8, 8))
    nf("Knoten mit Gegner bleibt", (8, 8), [], True, en=[8, 8, 0, 0, 0, 0])
    nf("Eckknoten (0, 0) ohne Nachbarn wird blau (Rand)", (0, 0), [], False)
    nf("Randknoten (32, 20) mit gelber Nachbarzelle (31, 20) bleibt", (32, 20), [(31, 20, COL_DONE, None)], True)
    return out


tests += destroy_tests(1)
tests += node_cases(1)
tests += nodes_free_cases()
regel0 = destroy_tests(0) + node_cases(0)

def main():
    static = [
        {"check": "size", "segment": "PROGRAM", "max": 12000, "maxEnd": "$4000", "name": "Programm <= 12000 Byte, endet vor dem Zeichensatz-RAM ($4000)"},
        {"check": "overlap"},
        {"check": "zeropage", "forbid": "$80-$ff", "name": "Zeropage: $80 und hoeher bleibt dem KERNAL"},
    ]
    PAD = {"grid.cell_type": "pad_cell_type", "grid.nb_boxes": "pad_nb_boxes", "enemies.rnd": "pad_rnd", "enemies.pick_random": "pad_pick_random",
           "enemies.level_idx": "pad_level_idx", "set_tempo": "pad_set_tempo", "input.port_of": "pad_port_of",
           "title_change": "pad_title_change", "grid.paint": "pad_paint", "enemies.find": "pad_find", "hud.bcd_add": "pad_bcd_add", "grid.fill_colour": "pad_fill_colour", "hiscore.place": "pad_place", "items.check": "pad_items_check", "grid.box_add": "pad_box_add", "switch_player": "pad_switch_player", "sound.play": "pad_sound_play"}
    for t in list(tests):
        if t["name"].startswith("eingefrorener") or t["name"].startswith("player.step: keine Richtung"):
            tests.remove(t)
            short.append(t)
    PAD["enemies.step"] = "pad_enemies_step"
    PAD["player.step"] = "pad_player_step"
    for t in short:
        t["call"] = PAD[t["call"]]
    for fn, ts, st, prg in (("troxx.vctest", tests, static, "build/main.prg"), ("troxx_kurz.vctest", short, [], "build/pad.prg"), ("troxx_regel0.vctest", regel0, [], "build0/main.prg")):
        d = {"kind": "test", "target": "plus4", "program": prg, "trap": "$0340", "static": st, "tests": ts}
        with open(fn, "w") as f:
            json.dump(d, f, indent=1, ensure_ascii=False)
            f.write("\n")
        print(fn, len(ts), "Tests")


main()
