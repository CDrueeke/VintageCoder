#!/usr/bin/env python3
# gen_tests.py - erzeugt troxx.vctest (Pruefungen und Routinentests, Atari) aus den Adressen in main.asm und gfx.inc.
# Aufruf im Projektordner: python3 tools/gen_tests.py
# Die Testfaelle der gemeinsamen Logik (bcd_add, legal_mask, greedy, neighbours, box_add, paint, cell_type, fill_*) entsprechen denen der Referenzfassung
# (examples/vic20/troxx/troxx.vctest) und der C64-Fassung, mit den Massen des Atari-Gitters (8 x 5 Rechtecke, 33 x 21 Zellen).  Neu fuer den Atari: Spielfeldzustand
# GRIDST statt Farbspeicher, Zeichen aus gfx.inc, Lage der Figuren (Player/Missiles), POKEY-Klangzustand, Eingabe ueber die PIA.
import json
import re

src = open('main.asm').read()
A = {}
for m in re.finditer(r'^\.const (\w+)\s*=\s*\$([0-9a-fA-F]+)\s*(?:\+ ZB)?', src, re.M):
    A[m.group(1)] = int(m.group(2), 16)
ZB = A['ZB']
for m in re.finditer(r'^\.const (\w+)\s*=\s*\$([0-9a-fA-F]+) \+ ZB', src, re.M):
    A[m.group(1)] = int(m.group(2), 16) + ZB
VB = A['VB']
for m in re.finditer(r'^\.const (\w+)\s*=\s*VB \+ \$([0-9a-fA-F]+)', src, re.M):
    A[m.group(1)] = VB + int(m.group(2), 16)
G = {}
for m in re.finditer(r'^\.const (\w+) = (\d+)', open('gfx.inc').read(), re.M):
    G[m.group(1)] = int(m.group(2))
BOXCNT = A['BOXCNT']
NBOX = 40
SCREEN = A['SCR']
GRIDST = A['GRIDST']
COLS = 40
ROW0 = 1
COL0 = 1
GRID_W, GRID_H = 33, 21
COL_LINE, COL_DONE = 4, 7
SND = A['SND']


def hx(v):
    return '$%x' % v


IT_KIND = '$%x' % A['IT_KIND']


def zp(name):
    return hx(A[name])


def scr(gx, gy):
    return SCREEN + (gy + ROW0) * COLS + gx + COL0


def st(gx, gy):
    return GRIDST + gy * GRID_W + gx


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

# ---- Zeichnen
tests.append(test("grid.draw: Ecke, Linie, T-Stueck, Innenraum (Zeichen und Zustand)", "grid.draw", None, [
    "[%s] == %d" % (hx(scr(0, 0)), G['CH_TL']), "[%s] == %d" % (hx(scr(1, 0)), G['CH_HLINE']), "[%s] == %d" % (hx(scr(4, 0)), G['CH_TDOWN']),
    "[%s] == %d" % (hx(scr(32, 0)), G['CH_TR']), "[%s] == %d" % (hx(scr(0, 1)), G['CH_VLINE']), "[%s] == 0" % hx(scr(1, 1)),
    "([%s] & 15) == 4" % hx(st(1, 0)), "([%s] & 15) == 4" % hx(st(0, 1))]))
tests.append(test("grid.draw: Kreuzung (gx 4, gy 4), Unterkante, Randspalten", "grid.draw", None, [
    "[%s] == %d" % (hx(scr(4, 4)), G['CH_CROSS']), "[%s] == %d" % (hx(scr(0, 20)), G['CH_BL']), "[%s] == %d" % (hx(scr(32, 20)), G['CH_BR']),
    "[%s] == %d" % (hx(scr(4, 20)), G['CH_TUP']), "[%s] == %d" % (hx(scr(0, 4)), G['CH_TRIGHT']), "[%s] == %d" % (hx(scr(32, 4)), G['CH_TLEFT']),
    "[%s] == %d" % (hx(scr(32, 5)), G['CH_VLINE']), "[%s] == %d" % (hx(scr(32, 19)), G['CH_VLINE'])]))

# ---- cell_type (Typ in A)
for (gx, gy, typ, nm) in [(0, 0, 3, "Ecke"), (4, 4, 3, "Knoten innen"), (32, 20, 3, "Ecke unten rechts"), (1, 0, 1, "waagerecht oben"),
                          (13, 8, 1, "waagerecht Mitte"), (0, 1, 2, "senkrecht links"), (20, 18, 2, "senkrecht Mitte"), (1, 1, 0, "Innenraum"),
                          (31, 19, 0, "Innenraum unten rechts"), (5, 2, 0, "Innenraum Mitte")]:
    tests.append(test("cell_type %s (%d, %d) = %d" % (nm, gx, gy, typ), "grid.cell_type", mem({zp('T0'): gx, zp('T1'): gy}), ["a == %d" % typ]))

# ---- neighbours: alle 93 Segmente (48 waagerecht, 45 senkrecht)
zero_boxes = [0] * NBOX
segs = []
for r in range(6):
    for c in range(8):
        segs.append(("H", r, c, 4 * c + 1, 4 * r, 1, [b for b in ((r * 8 + c) if r < 5 else None, (r - 1) * 8 + c if r > 0 else None) if b is not None]))
for r in range(5):
    for c in range(9):
        segs.append(("V", r, c, 4 * c, 4 * r + 1, 2, [b for b in ((r * 8 + c) if c < 8 else None, (r * 8 + c - 1) if c > 0 else None) if b is not None]))
NB0 = A['NB0']
NB1 = A['NB1']
for i, (k, r, c, gx, gy, typ, boxes) in enumerate(segs):
    b0 = 8 * (gy // 4) + gx // 4
    if k == "H":
        n0 = b0 if gy < 20 else 255
        n1 = b0 - 8 if gy > 0 else 255
    else:
        n0 = b0 if gx < 32 else 255
        n1 = b0 - 1 if gx > 0 else 255
    assert sorted(x for x in (n0, n1) if x != 255) == sorted(boxes)
    tests.append(test("nb_boxes Segment %d %s(%d,%d): Rechtecke %s" % (i, k, r, c, boxes), "grid.nb_boxes",
                      mem({zp('T0'): gx, zp('T1'): gy, zp('T2'): typ}), ["[%s] == %d" % (hx(NB0), n0), "[%s] == %d" % (hx(NB1), n1)]))

# ---- box_add: vierte Kante merkt das Rechteck genau einmal vor (100 Punkte)
tests.append(test("box_add: vierte Kante: vorgemerkt, 100 Punkte, BOXES_DONE + 1", "grid.box_add",
                  dict(x=7, **mem({zp('FILLS_NOW'): 0, zp('BOXES_DONE'): 0, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3,
                                   hx(BOXCNT): [0] * 7 + [3] + [0] * 32})),
                  ["[%s] == 4" % hx(BOXCNT + 7), "[%s] == 1" % zp('FILLS_NOW'), "[%s] == 1" % zp('BOXES_DONE'), "[%s] == 7" % zp('FILLQ'),
                   "[%s] == 0" % zp('SCORE'), "[%s] == 1" % hx(A['SCORE'] + 1)]))
tests.append(test("box_add: dritte Kante: nichts vorgemerkt", "grid.box_add",
                  dict(x=7, **mem({zp('FILLS_NOW'): 0, zp('BOXES_DONE'): 0, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3,
                                   hx(BOXCNT): [0] * 7 + [2] + [0] * 32})),
                  ["[%s] == 3" % hx(BOXCNT + 7), "[%s] == 0" % zp('FILLS_NOW'), "[%s] == 0" % zp('BOXES_DONE')]))

# ---- paint: Kettenreaktion (zwei Rechtecke), ein Rechteck, schon abgefahren
cells = {}
for gy in (5, 6):
    cells[st(4, gy)] = COL_DONE          # V(1,1): Zellen (4,5), (4,6) abgefahren, (4,7) wird gefaerbt
cells[st(4, 7)] = COL_LINE
bc = [0] * NBOX
bc[8] = 3
bc[9] = 3
base = {zp('T0'): 4, zp('T1'): 7, zp('FILLS_NOW'): 0, zp('BOXES_DONE'): 0, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3, hx(BOXCNT): bc}
for k, v in cells.items():
    base[hx(k)] = v
tests.append(test("chain_two (paint): letztes Zeichen von V(1,1) schliesst zwei Rechtecke: 411 Punkte, beide vorgemerkt", "grid.paint", mem(base), [
    "[%s] == 2" % zp('FILLS_NOW'), "[%s] == 2" % zp('BOXES_DONE'), "[%s] == 9" % zp('FILLQ'), "[%s] == 8" % hx(A['FILLQ'] + 1),
    "[%s] == $11" % zp('SCORE'), "[%s] == $04" % hx(A['SCORE'] + 1), "[%s] == 0" % hx(A['SCORE'] + 2), "([%s] & 15) == 7" % hx(st(4, 7))]))
b2 = dict(base)
bc2 = [0] * NBOX
bc2[9] = 3
b2[hx(BOXCNT)] = bc2
tests.append(test("paint: Segment schliesst nur ein Rechteck (kein Kettenbonus): 111 Punkte", "grid.paint", mem(b2), [
    "[%s] == 1" % zp('FILLS_NOW'), "[%s] == $11" % zp('SCORE'), "[%s] == $01" % hx(A['SCORE'] + 1), "[%s] == 1" % hx(BOXCNT + 8)]))
b3 = dict(base)
b3[hx(st(4, 7))] = COL_DONE
tests.append(test("paint: Zelle schon abgefahren: nichts passiert", "grid.paint", mem(b3), ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % zp('FILLS_NOW')]))
b4 = dict(base)
b4[zp('T0')] = 1
b4[zp('T1')] = 1
tests.append(test("paint: Innenraum tut nichts", "grid.paint", mem(b4), ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % zp('FILLS_NOW')]))
b5 = dict(base)
b5[zp('T0')] = 4
b5[zp('T1')] = 4
tests.append(test("paint: Knoten (4, 4) tut nichts", "grid.paint", mem(b5), ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % zp('FILLS_NOW')]))
hb = {zp('T0'): 3, zp('T1'): 0, zp('FILLS_NOW'): 0, zp('BOXES_DONE'): 0, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3,
      hx(BOXCNT): [3] + [0] * 39, hx(st(1, 0)): COL_DONE, hx(st(2, 0)): COL_DONE, hx(st(3, 0)): COL_LINE}
tests.append(test("paint: H(0,0) schliesst B(0,0) am oberen Rand: 111 Punkte", "grid.paint", mem(hb), [
    "[%s] == 1" % zp('FILLS_NOW'), "[%s] == 0" % zp('FILLQ'), "[%s] == $11" % zp('SCORE'), "[%s] == $01" % hx(A['SCORE'] + 1)]))


# ---- Regel 16.2: Gegner zerstoeren nicht geschlossene Linien (seg_destroy, seg_is_closed)
SCORE = A['SCORE']


def dstate(cells_done, extra=None, boxes=None, score=(0, 0, 0), t=(6, 4, 1)):
    d = {zp('T0'): t[0], zp('T1'): t[1], zp('T2'): t[2], zp('SCORE'): list(score), zp('SCORE_OVF'): 0, zp('LIVES'): 3, zp('FLAGS'): 1, IT_KIND: 0,
         hx(BOXCNT): boxes or [0] * 40, hx(SND): [0] * 64}
    for (x, y) in cells_done:
        d[hx(st(x, y))] = COL_DONE
    if extra:
        d.update(extra)
    return mem(d)


def bx(**kw):
    b = [0] * 40
    for k, v in kw.items():
        b[int(k[1:])] = v
    return b


H11 = [(5, 4), (6, 4), (7, 4)]
tests.append(test("seg_destroy: teilweise abgefahrenes Segment: alles blau, 2 Punkte abgezogen, Zeichen zurueck", "grid.seg_destroy",
                  dstate([(5, 4), (6, 4)], {hx(scr(5, 4)): G['CH_HLINE_D'], hx(scr(6, 4)): G['CH_HLINE_D']}, score=(5, 0, 0)),
                  ["[%s] == 4" % hx(st(5, 4)), "[%s] == 4" % hx(st(6, 4)), "[%s] == 4" % hx(st(7, 4)), "[%s] == %d" % (hx(scr(5, 4)), G['CH_BLOCK']),
                   "[%s] == %d" % (hx(scr(6, 4)), G['CH_BLOCK']), "[%s] == 2" % zp('FLT'), "[%s] == 3" % zp('SCORE'), "[%s] == 0" % hx(BOXCNT + 9), "[%s] == 0" % hx(BOXCNT + 1)], timeoutMs=60000))
tests.append(test("seg_destroy: ganz abgefahrenes Segment ohne gefuelltes Rechteck: 13 Punkte, BOXCNT der Nachbarn - 1", "grid.seg_destroy",
                  dstate(H11, boxes=bx(b9=2, b1=1), score=(0x40, 0, 0)),
                  ["[%s] == 4" % hx(st(5, 4)), "[%s] == 4" % hx(st(7, 4)), "[%s] == $27" % zp('SCORE'), "[%s] == 1" % hx(BOXCNT + 9), "[%s] == 0" % hx(BOXCNT + 1)], timeoutMs=60000))
tests.append(test("seg_destroy: geschuetztes Segment (unteres Rechteck gefuellt) bleibt", "grid.seg_destroy",
                  dstate(H11, boxes=bx(b9=4, b1=2), score=(0x40, 0, 0)),
                  ["[%s] == 7" % hx(st(5, 4)), "[%s] == 7" % hx(st(7, 4)), "[%s] == $40" % zp('SCORE'), "[%s] == 4" % hx(BOXCNT + 9), "[%s] == 2" % hx(BOXCNT + 1)], timeoutMs=60000))
tests.append(test("seg_destroy: geschuetzt durch das obere Rechteck", "grid.seg_destroy",
                  dstate(H11, boxes=bx(b9=2, b1=4), score=(0x40, 0, 0)), ["[%s] == 7" % hx(st(6, 4)), "[%s] == $40" % zp('SCORE'), "[%s] == 2" % hx(BOXCNT + 9)], timeoutMs=60000))
tests.append(test("seg_destroy: Randsegment H(0,0): nur ein Nachbar", "grid.seg_destroy",
                  dstate([(1, 0), (2, 0), (3, 0)], boxes=bx(b0=1), score=(0x20, 0, 0), t=(2, 0, 1)),
                  ["[%s] == 4" % hx(st(2, 0)), "[%s] == 0" % hx(BOXCNT), "[%s] == $07" % zp('SCORE')], timeoutMs=60000))
tests.append(test("seg_destroy: senkrechtes Segment V(1,2) mit zwei Nachbarn", "grid.seg_destroy",
                  dstate([(8, 5), (8, 6), (8, 7)], boxes=bx(b9=1, b10=1), score=(0x30, 0, 0), t=(8, 6, 2)),
                  ["[%s] == 4" % hx(st(8, 5)), "[%s] == 4" % hx(st(8, 7)), "[%s] == 0" % hx(BOXCNT + 9), "[%s] == 0" % hx(BOXCNT + 10), "[%s] == $17" % zp('SCORE'),
                   "[%s] == %d" % (hx(scr(8, 6)), G['CH_BLOCK'])], timeoutMs=60000))
tests.append(test("flash_end: Zeichen des Segments zurueck, Zeilenfarbe aus GAME_P2, Warteschlange leer", "grid.flash_end",
                  mem({zp('FLT'): 1, zp('FLN'): 1, zp('FLX'): 6, zp('FLY'): 4, zp('FLTY'): 1, hx(scr(5, 4)): G['CH_BLOCK'], hx(scr(6, 4)): G['CH_BLOCK'], IT_KIND: 0,
                       hx(A['RC_P2'] + 5): 14, zp('T0'): 9, zp('T1'): 8, zp('T2'): 7}),
                  ["[%s] == %d" % (hx(scr(5, 4)), G['CH_HLINE']), "[%s] == %d" % (hx(scr(6, 4)), G['CH_HLINE']), "[%s] == %d" % (hx(scr(7, 4)), G['CH_HLINE']),
                   "[%s] == 0" % zp('FLT'), "[%s] == 0" % zp('FLN'), "[%s] != 14" % hx(A['RC_P2'] + 5), "[%s] == 9" % zp('T0'), "[%s] == 8" % zp('T1'), "[%s] == 7" % zp('T2')]))
tests.append(test("seg_destroy: Punkte nie unter 0", "grid.seg_destroy",
                  dstate(H11, boxes=bx(b9=1, b1=1), score=(5, 0, 0)), ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % hx(SCORE + 1), "[%s] == 0" % hx(SCORE + 2)], timeoutMs=60000))
tests.append(test("seg_destroy: nichts abgefahren: nichts geschieht", "grid.seg_destroy",
                  dstate([], boxes=bx(b9=2), score=(0x40, 0, 0)), ["[%s] == $40" % zp('SCORE'), "[%s] == 2" % hx(BOXCNT + 9)], timeoutMs=60000))
tests.append(test("seg_is_closed: Nachbarrechteck gefuellt: C = 1", "grid.seg_is_closed", dstate([], boxes=bx(b9=4)), ["fc == 1"]))
tests.append(test("seg_is_closed: kein Nachbar gefuellt: C = 0", "grid.seg_is_closed", dstate([], boxes=bx(b9=3, b1=3)), ["fc == 0"]))
tests.append(test("bcd_sub: 13 von 40 = 27", "hud.bcd_sub", dict(a=0x13, **mem({zp('SCORE'): [0x40, 0, 0], zp('T7'): 0})), ["[%s] == $27" % zp('SCORE')]))
tests.append(test("bcd_sub: Uebertrag ueber die Stellen (10000 - 1)", "hud.bcd_sub", dict(a=1, **mem({zp('SCORE'): [0, 0, 1], zp('T7'): 0})),
                  ["[%s] == $99" % zp('SCORE'), "[%s] == $99" % hx(SCORE + 1), "[%s] == 0" % hx(SCORE + 2)]))


def one_setup(frozen):
    d = {zp('ELIM'): 6, zp('ECUR'): 0, zp('EN'): [5, 4, 3, 0, 5, 4], zp('PX'): 0, zp('PY'): 20, zp('PINV'): 0, zp('HITF'): 0, zp('EPH'): 0,
         zp('SCORE'): [0x40, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3, zp('FLAGS'): 1, IT_KIND: 0, hx(BOXCNT): [0] * 40, hx(SND): [0] * 64,
         zp('FREEZE'): 100 if frozen else 0}
    for (x, y) in H11:
        d[hx(st(x, y))] = COL_DONE
    return mem(d)


tests.append(test("enemies.one: Gegner betritt ein Innenzeichen eines offenen Segments: Segment zerstoert", "enemies.one", one_setup(False),
                  ["[%s] == 6" % hx(A['EN']), "[%s] == 4" % hx(st(5, 4)), "[%s] == 4" % hx(st(7, 4)), "[%s] == $27" % zp('SCORE')], timeoutMs=60000))
tests.append(test("enemies.step: eingefrorener Gegner (Eisblock) zerstoert nichts", "enemies.step", one_setup(True),
                  ["[%s] == 5" % hx(A['EN']), "[%s] == 7" % hx(st(5, 4)), "[%s] == $40" % zp('SCORE')], timeoutMs=60000))
RULE0 = [test("enemies.one (Regel aus): Gegner betritt ein Innenzeichen: nichts wird zerstoert", "enemies.one", one_setup(False),
              ["[%s] == 6" % hx(A['EN']), "[%s] == 7" % hx(st(5, 4)), "[%s] == 7" % hx(st(6, 4)), "[%s] == $40" % zp('SCORE')])]


# ---- Knotenregel: paint_node, show (gelber Knoten), node_free, nodes_free, touch an Knoten
NODEX = A['NODEX']
NODEY = A['NODEY']
tests.append(test("paint_node: die Figur betritt den Knoten (4, 4): Zustand abgefahren", "grid.paint_node", mem({zp('T0'): 4, zp('T1'): 4, hx(st(4, 4)): COL_LINE}), ["[%s] == 7" % hx(st(4, 4))]))
tests.append(test("paint_node: Linienzelle bleibt unberuehrt", "grid.paint_node", mem({zp('T0'): 5, zp('T1'): 4, hx(st(5, 4)): COL_LINE}), ["[%s] == 4" % hx(st(5, 4))]))
for (gx, gy, base, ye, nm) in [(4, 4, 'CH_CROSS', 'CH_CROSS_D', "Kreuzung"), (0, 0, 'CH_TL', 'CH_TDOWN_D', "Ecke oben links"), (32, 20, 'CH_BR', 'CH_TUP_D', "Ecke unten rechts"),
                               (8, 0, 'CH_TDOWN', 'CH_TDOWN_D', "T-Stueck oben"), (0, 8, 'CH_TRIGHT', 'CH_TRIGHT_D', "T-Stueck links"), (32, 8, 'CH_TLEFT', 'CH_TLEFT_D', "T-Stueck rechts")]:
    tests.append(test("grid.show: abgefahrener Knoten %s (%d, %d) wird gelb" % (nm, gx, gy), "grid.show", mem({zp('T0'): gx, zp('T1'): gy, hx(st(gx, gy)): COL_DONE, hx(scr(gx, gy)): G[base]}),
                      ["[%s] == %d" % (hx(scr(gx, gy)), G[ye])]))
tests.append(test("grid.show: unberuehrter Knoten behaelt sein blaues Zeichen", "grid.show", mem({zp('T0'): 4, zp('T1'): 4, hx(st(4, 4)): COL_LINE, hx(scr(4, 4)): 0}),
                  ["[%s] == %d" % (hx(scr(4, 4)), G['CH_CROSS'])]))


def nf(nm, extra, exp_state, exp_char, node=(4, 4), cells=(), px=(0, 20), en=None):
    d = {zp('PX'): px[0], zp('PY'): px[1], zp('ELIM'): 6 if en else 0, zp('EN'): en or [255, 255, 0, 0, 255, 255], hx(st(node[0], node[1])): COL_DONE,
         hx(scr(node[0], node[1])): G['CH_CROSS_D'], zp('NODEX'): node[0], zp('NODEY'): node[1]}
    for dx in (-1, 0, 1):
        for dy in (-1, 0, 1):
            x, y = node[0] + dx, node[1] + dy
            if (dx or dy) and 0 <= x < 33 and 0 <= y < 21:
                d[hx(st(x, y))] = COL_LINE
    for (x, y, v) in cells:
        d[hx(st(x, y))] = v
    tests.append(test("node_free: " + nm, "grid.node_free", mem(d), ["[%s] == %d" % (hx(st(node[0], node[1])), exp_state), "[%s] == %d" % (hx(scr(node[0], node[1])), exp_char)]))


nf("freistehender gelber Knoten wird zurueckgesetzt (Zustand und Zeichen)", None, COL_LINE, G['CH_CROSS'])
nf("angrenzendes gelbes Segment (rechts) haelt den Knoten", None, COL_DONE, G['CH_CROSS_D'], cells=[(5, 4, COL_DONE)])
nf("angrenzendes gelbes Segment (unten) haelt den Knoten", None, COL_DONE, G['CH_CROSS_D'], cells=[(4, 5, COL_DONE)])
nf("angrenzendes gefuelltes Rechteck (Diagonalzelle) haelt den Knoten", None, COL_DONE, G['CH_CROSS_D'], cells=[(5, 5, 5)])
nf("gefuelltes Rechteck oben links haelt den Knoten", None, COL_DONE, G['CH_CROSS_D'], cells=[(3, 3, 5)])
nf("die Figur steht auf dem Knoten: er bleibt", None, COL_DONE, G['CH_CROSS_D'], px=(4, 4))
nf("ein Gegner steht auf dem Knoten: er bleibt", None, COL_DONE, G['CH_CROSS_D'], en=[4, 4, 0, 0, 4, 4])
nf("Eckknoten (0, 0) frei: zurueckgesetzt", None, COL_LINE, G['CH_TL'], node=(0, 0))
nf("Eckknoten (0, 0) mit gelbem Segment (1, 0): bleibt", None, COL_DONE, G['CH_TL'] if False else G['CH_CROSS_D'], node=(0, 0), cells=[(1, 0, COL_DONE)])
tests.append(test("node_free: nicht abgefahrener Knoten bleibt unberuehrt", "grid.node_free", mem({zp('NODEX'): 4, zp('NODEY'): 4, hx(st(4, 4)): COL_LINE, hx(scr(4, 4)): 0, zp('PX'): 0, zp('PY'): 20, zp('ELIM'): 0}),
                  ["[%s] == 4" % hx(st(4, 4)), "[%s] == 0" % hx(scr(4, 4))]))
tests.append(test("nodes_free: alle gelben freistehenden Knoten werden zurueckgesetzt, ein gehaltener bleibt", "grid.nodes_free",
                  mem(dict([(zp('PX'), 0), (zp('PY'), 20), (zp('ELIM'), 0), (hx(st(4, 4)), COL_DONE), (hx(st(8, 8)), COL_DONE), (hx(st(12, 12)), COL_DONE), (hx(st(13, 12)), COL_DONE)] +
                           [(hx(st(x, y)), COL_LINE) for x in range(0, 33) for y in range(0, 21) if (x, y) not in [(4, 4), (8, 8), (12, 12), (13, 12)]])),
                  ["[%s] == 4" % hx(st(4, 4)), "[%s] == 4" % hx(st(8, 8)), "[%s] == 7" % hx(st(12, 12))]))


def tch(nm, boxes, exp_states, exp_score, fln):
    d = {zp('T4'): 4, zp('T5'): 4, zp('SCORE'): [0x40, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3, zp('FLAGS'): 1, IT_KIND: 0, zp('FLN'): 0, zp('FLT'): 0,
         hx(BOXCNT): boxes, hx(SND): [0] * 64, zp('PX'): 0, zp('PY'): 20, zp('ELIM'): 0, hx(st(4, 4)): COL_DONE, hx(st(5, 4)): COL_DONE, hx(st(4, 5)): COL_DONE,
         hx(st(4, 6)): COL_DONE}
    for (x, y) in [(3, 3), (3, 4), (3, 5), (4, 3), (5, 3), (5, 5), (4, 7), (5, 6), (5, 7), (6, 4), (7, 4), (3, 6)]:
        d[hx(st(x, y))] = COL_LINE
    tests.append(test("enemies.touch: " + nm, "enemies.touch", mem(d), ["[%s] == %d" % (hx(st(x, y)), v) for (x, y, v) in exp_states] + ["[%s] == %s" % (zp('SCORE'), exp_score), "[%s] == %d" % (zp('FLN'), fln)], timeoutMs=60000))


tch("Gegner betritt den Knoten (4, 4): beide Segmente ohne gefuelltes Rechteck zerstoert, Knoten frei zurueck, 3 Punkte ab", [0] * 40,
    [(5, 4, 4), (4, 5, 4), (4, 6, 4), (4, 4, 4)], "$37", 2)
tch("beide Segmente neben einem gefuellten Rechteck (B(1,1)): nichts passiert, Knoten bleibt gelb", bx(b9=4),
    [(5, 4, 7), (4, 5, 7), (4, 6, 7), (4, 4, 7)], "$40", 0)
tch("nur das senkrechte Segment ist geschuetzt (links B(1,0) gefuellt), das waagerechte (unten B(1,1) offen) wird zerstoert", bx(b8=4),
    [(5, 4, 4), (4, 5, 7), (4, 6, 7), (4, 4, 7)], "$39", 1)

# ---- show: Zeichen aus dem Zustand
tests.append(test("grid.show: abgefahrene waagerechte Linie (3, 0) bekommt das gelbe Zeichen", "grid.show",
                  mem({zp('T0'): 3, zp('T1'): 0, hx(st(3, 0)): COL_DONE}), ["[%s] == %d" % (hx(scr(3, 0)), G['CH_HLINE_D'])]))
tests.append(test("grid.show: abgefahrene senkrechte Linie (0, 2) bekommt das gelbe Zeichen", "grid.show",
                  mem({zp('T0'): 0, zp('T1'): 2, hx(st(0, 2)): COL_DONE}), ["[%s] == %d" % (hx(scr(0, 2)), G['CH_VLINE_D'])]))
tests.append(test("grid.show: unberuehrte Linie behaelt das blaue Zeichen", "grid.show",
                  mem({zp('T0'): 5, zp('T1'): 4, hx(st(5, 4)): COL_LINE, hx(scr(5, 4)): 0}), ["[%s] == %d" % (hx(scr(5, 4)), G['CH_HLINE'])]))
tests.append(test("grid.show: Knoten (4, 4) zeigt wieder die Kreuzung", "grid.show",
                  mem({zp('T0'): 4, zp('T1'): 4, hx(st(4, 4)): COL_LINE, hx(scr(4, 4)): G['CH_STAR']}), ["[%s] == %d" % (hx(scr(4, 4)), G['CH_CROSS'])]))
tests.append(test("grid.show: Innenraum bleibt unberuehrt", "grid.show",
                  mem({zp('T0'): 2, zp('T1'): 2, hx(scr(2, 2)): 9}), ["[%s] == 9" % hx(scr(2, 2))]))

# ---- bcd_add
tests.append(test("bcd_add: Uebertrag ueber alle Stellen", "hud.bcd_add", dict(a=153, x=0, **mem({zp('SCORE'): [1, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3})),
                  ["[%s] == 0" % zp('SCORE'), "[%s] == 1" % hx(A['SCORE'] + 1), "[%s] == 3" % zp('LIVES')]))
tests.append(test("bcd_add: Extraleben bei 10000 Punkten", "hud.bcd_add", dict(a=80, x=0, **mem({zp('SCORE'): [80, 153, 9], zp('SCORE_OVF'): 0, zp('LIVES'): 3})),
                  ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % hx(A['SCORE'] + 1), "[%s] == $10" % hx(A['SCORE'] + 2), "[%s] == 4" % zp('LIVES')]))
tests.append(test("bcd_add: 9999 + 1 = 10000: ein Extraleben", "hud.bcd_add", dict(a=1, x=0, **mem({zp('SCORE'): [0x99, 0x99, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3})),
                  ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % hx(A['SCORE'] + 1), "[%s] == 1" % hx(A['SCORE'] + 2), "[%s] == 4" % zp('LIVES')]))
tests.append(test("bcd_add: 9998 + 1 = 9999: kein Extraleben", "hud.bcd_add", dict(a=1, x=0, **mem({zp('SCORE'): [0x98, 0x99, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3})),
                  ["[%s] == $99" % zp('SCORE'), "[%s] == $99" % hx(A['SCORE'] + 1), "[%s] == 0" % hx(A['SCORE'] + 2), "[%s] == 3" % zp('LIVES')]))
tests.append(test("bcd_add: 9999 + 1 bei 9 Leben: bleibt 9", "hud.bcd_add", dict(a=1, x=0, **mem({zp('SCORE'): [0x99, 0x99, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 9})),
                  ["[%s] == 1" % hx(A['SCORE'] + 2), "[%s] == 9" % zp('LIVES')]))
tests.append(test("bcd_add: Leben hoechstens 9", "hud.bcd_add", dict(a=80, x=0, **mem({zp('SCORE'): [80, 153, 9], zp('SCORE_OVF'): 0, zp('LIVES'): 9})),
                  ["[%s] == 9" % zp('LIVES')]))
tests.append(test("bcd_add: 500 Punkte (X = 1)", "hud.bcd_add", dict(a=5, x=1, **mem({zp('SCORE'): [32, 1, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3})),
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
for s0, nxt in [(1, 0xb8), (0xb8, 0x5c), (0x5c, 0x2e), (0x80, 0x40), (0xff, 0xc7)]:
    tests.append(test("rnd: LFSR $%02X -> $%02X" % (s0, nxt), "enemies.rnd", mem({zp('RND'): s0}), ["a == $%x" % nxt, "[%s] == $%x" % (zp('RND'), nxt)]))
for mask in (1, 2, 4, 8, 6, 9, 15):
    tests.append(test("pick_random: Maske %d liefert eine Richtung aus der Maske" % mask, "enemies.pick_random", mem({zp('T7'): mask, zp('RND'): 0x5a}), ["x < 4"]))

# ---- level_idx
for lv, idx in [(1, 0), (5, 4), (9, 8), (0x10, 8), (0x25, 8)]:
    tests.append(test("level_idx: Stufe BCD $%02X -> Index %d" % (lv, idx), "enemies.level_idx", mem({zp('LEVEL'): lv}), ["y == %d" % idx]))

# ---- Gegner platzieren: Zahl nach Stufe, Typen, Startknoten
for lv, n in [(1, 1), (2, 2), (4, 3), (7, 4)]:
    tests.append(test("enemies.place_all: Stufe %d -> %d Gegner" % (lv, n), "enemies.place_all", mem({zp('LEVEL'): lv}),
                      ["[%s] == %d" % (zp('NENEMY'), n), "[%s] == %d" % (zp('ELIM'), 6 * n), "[%s] == 32" % zp('EN'), "[%s] == 0" % hx(A['EN'] + 1),
                       "[%s] == 32" % hx(A['EN'] + 4), "[%s] == 0" % hx(A['EN'] + 3)]))

# ---- find
tests.append(test("enemies.find: Gegner auf (7, 8) gefunden, X = Abstand", "enemies.find",
                  dict(x=7, y=8, **mem({zp('ELIM'): 12, zp('EN'): [1, 1, 0, 0, 1, 1, 7, 8, 0, 1, 7, 8]})), ["fc == 1", "x == 6"]))
tests.append(test("enemies.find: keiner auf (3, 3)", "enemies.find",
                  dict(x=3, y=3, **mem({zp('ELIM'): 12, zp('EN'): [1, 1, 0, 0, 1, 1, 7, 8, 0, 1, 7, 8]})), ["fc == 0"]))

# ---- player.step
IT_KIND = hx(A['IT_KIND'])
stt = {zp('PX'): 0, zp('PY'): 20, zp('PWANT'): 1, zp('PANIM'): 0, zp('PINV'): 0, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3, zp('HITF'): 0,
       zp('FILLS_NOW'): 0, hx(BOXCNT): [0] * 40, zp('ELIM'): 6, zp('EN'): [255, 255, 0, 0, 255, 255], hx(st(0, 19)): COL_LINE, hx(st(0, 20)): COL_LINE,
       zp('PDIR'): 0, IT_KIND: 0}
tests.append(test("player.step: von (0, 20) hoch nach (0, 19): faerbt, 1 Punkt, Zelle vor dem Schritt gemerkt, Zeichen gelb", "player.step", mem(stt), [
    "[%s] == 19" % zp('PY'), "[%s] == 0" % zp('PX'), "[%s] == 20" % zp('POY'), "[%s] == 1" % zp('SCORE'), "([%s] & 15) == 7" % hx(st(0, 19)),
    "[%s] == 1" % zp('PANIM'), "[%s] == 0" % zp('PDIR'), "[%s] == %d" % (hx(scr(0, 19)), G['CH_VLINE_D'])]))
s2 = dict(stt)
s2[zp('PWANT')] = 8
s2[zp('PX')] = 1
s2[zp('PY')] = 20
tests.append(test("player.step: waagerecht rechts von (1, 20) nach (2, 20)", "player.step", mem(s2), ["[%s] == 2" % zp('PX'), "[%s] == 20" % zp('PY')]))
s3 = dict(stt)
s3[zp('PWANT')] = 1
s3[zp('PX')] = 1
s3[zp('PY')] = 20
tests.append(test("player.step: auf einer waagerechten Linie ist hoch gesperrt", "player.step", mem(s3), ["[%s] == 20" % zp('PY'), "[%s] == 1" % zp('PX')]))
s4 = dict(stt)
s4[zp('PWANT')] = 1
s4[zp('PX')] = 0
s4[zp('PY')] = 0
tests.append(test("player.step: ausserhalb des Gitters kein Schritt (Ecke 0, 0, hoch)", "player.step", mem(s4), ["[%s] == 0" % zp('PY')]))
s5 = dict(stt)
s5[zp('EN')] = [0, 19, 0, 0, 0, 19]
tests.append(test("player.step: Gegner auf der Zielzelle = Treffer, Figur bleibt", "player.step", mem(s5), ["[%s] == 1" % zp('HITF'), "[%s] == 20" % zp('PY')]))
s6 = dict(s5)
s6[zp('PINV')] = 5
tests.append(test("player.step: Gegner auf der Zielzelle waehrend der Unverwundbarkeit: kein Treffer, Figur bleibt", "player.step", mem(s6),
                  ["[%s] == 0" % zp('HITF'), "[%s] == 20" % zp('PY'), "[%s] == 4" % zp('PINV')]))
s7 = dict(stt)
s7[zp('PWANT')] = 0
tests.append(test("player.step: keine Richtung gehalten: stehen bleiben, Zelle vor dem Schritt = Zelle", "player.step", mem(s7),
                  ["[%s] == 20" % zp('PY'), "[%s] == 20" % zp('POY'), "[%s] == 0" % zp('SCORE')]))

# ---- Fuellen: Rahmen mit Abschraegung, Muster in der Mitte
for (r, c) in [(0, 1), (4, 7), (2, 3)]:
    pat = [G['CH_FILL0'], G['CH_FILL1'], G['CH_FILL2'], G['CH_FILL3']][(r + c) & 3]
    ex = []
    x0, y0 = 4 * c + 1, 4 * r + 1
    layout = [[G['CH_FTL'], G['CH_FT'], G['CH_FTR']], [G['CH_FL'], pat, G['CH_FR']], [G['CH_FBL'], G['CH_FB'], G['CH_FBR']]]
    for dy in range(3):
        for dx in range(3):
            ex.append("[%s] == %d" % (hx(scr(x0 + dx, y0 + dy)), layout[dy][dx]))
    ex.append("[%s] == 5" % hx(st(4 * c + 2, 4 * r + 2)))
    ex.append("[%s] != %d" % (hx(scr(4 * c + 4, 4 * r + 1)), pat))
    ex.append("[%s] != %d" % (hx(scr(4 * c + 1, 4 * r + 4)), pat))
    tests.append(test("grid.fill_box: Rechteck (%d, %d): Rahmen und Muster %d" % (r, c, pat), "grid.fill_box", dict(x=r, y=c), ex))
pat39 = [G['CH_FILL0'], G['CH_FILL1'], G['CH_FILL2'], G['CH_FILL3']][(4 + 7) & 3]
tests.append(test("grid.fill_row_b: Rechteck 39, Zeile 2: Zeile gy 19, Rahmen unten", "grid.fill_row_b", dict(a=39, x=2),
                  ["[%s] == %d" % (hx(scr(29, 19)), G['CH_FBL']), "[%s] == %d" % (hx(scr(30, 19)), G['CH_FB']), "[%s] == %d" % (hx(scr(31, 19)), G['CH_FBR']),
                   "[%s] != %d" % (hx(scr(29, 20)), G['CH_FBL'])]))
tests.append(test("grid.fill_row_b: Rechteck 39, Zeile 1: Muster in der Mitte", "grid.fill_row_b", dict(a=39, x=1),
                  ["[%s] == %d" % (hx(scr(30, 18)), pat39), "[%s] == %d" % (hx(scr(29, 18)), G['CH_FL']), "[%s] == %d" % (hx(scr(31, 18)), G['CH_FR'])]))

# ---- Tempo
for sp, pal, v in [(1, 1, 12), (9, 1, 3), (5, 1, 7), (1, 0, 14), (9, 0, 4), (5, 0, 8)]:
    tests.append(test("set_tempo: Tempo %d %s -> %d Bilder" % (sp, "PAL" if pal else "NTSC", v), "set_tempo", mem({zp('SPEED'): sp, zp('FLAGS'): pal}), ["[%s] == %d" % (zp('STEPLEN'), v)]))


# ---- Lage der Figuren: HPOS = 47 + 4 * (gx + 1), Zeile = 32 + 8 * (gy + 1); Zwischenwerte nach SPPROG
def place(nm, ox, nx, oy, ny, prog, ex, ey):
    d = {zp('SPT0'): ox, zp('SPT1'): nx, zp('SPT2'): oy, zp('SPT3'): ny, zp('SPPROG'): prog}
    tests.append(test("sprites.place: " + nm, "sprites.place", mem(d), ["[%s] == %d" % (zp('SPX'), ex), "[%s] == %d" % (zp('SPY'), ey)]))


place("Zelle (0, 20) ruhend: HPOS 51, Zeile 200", 0, 0, 20, 20, 5, 51, 200)
place("von Zelle 5 nach 6 um 4 Punkte: zwei Farbtakte", 5, 6, 3, 3, 4, 47 + 4 * 6 + 2, 64)
place("von Zelle 6 nach 5 um 3 Punkte zurueck: ein Farbtakt", 6, 5, 3, 3, 3, 47 + 4 * 7 - 1, 64)
place("senkrecht: von gy 10 nach 9 um 2 Punkte", 3, 3, 10, 9, 2, 47 + 4 * 4, 32 + 8 * 11 - 2)
place("rechter Rand: Zelle (32, 0): HPOS 47 + 4 * 33 = 179", 32, 32, 0, 0, 0, 179, 40)
place("senkrecht abwaerts von gy 0 nach 1 um 5 Punkte", 2, 2, 0, 1, 5, 47 + 4 * 3, 40 + 5)

# ---- Gegenstaende
pk = {zp('PX'): 9, zp('PY'): 8, IT_KIND: 1, zp('IT_X'): 9, zp('IT_Y'): 8, zp('FPS'): 50, zp('JOYMODE'): 0, zp('RND'): 0x33, zp('SCORE'): [0, 0, 0],
      zp('SCORE_OVF'): 0, zp('LIVES'): 3, zp('FLAGS'): 1, hx(st(9, 8)): COL_LINE}
tests.append(test("items.check: Stern aufgenommen: 50 Punkte, Gegenstand weg", "items.check", mem(pk), ["[%s] == $50" % zp('SCORE'), "[%s] == 0" % IT_KIND]))
pk2 = dict(pk)
pk2[IT_KIND] = 2
tests.append(test("items.check: Eisblock aufgenommen: Starre 250 Bilder (PAL), Gegenstand weg", "items.check", mem(pk2),
                  ["[%s] == 250" % zp('FREEZE'), "[%s] == 0" % hx(A['FREEZE'] + 1), "[%s] == 0" % IT_KIND]))
pk3 = dict(pk)
pk3[zp('PX')] = 10
tests.append(test("items.check: Figur nicht auf dem Gegenstand: nichts passiert", "items.check", mem(pk3), ["[%s] == 0" % zp('SCORE'), "[%s] == 1" % IT_KIND]))
pk4 = dict(pk2)
pk4[zp('FLAGS')] = 0
tests.append(test("items.check: Eisblock, NTSC: Starre 300 Bilder", "items.check", mem(pk4), ["[%s] == 44" % zp('FREEZE'), "[%s] == 1" % hx(A['FREEZE'] + 1)]))
tests.append(test("items.draw_on: Stern erscheint als Zeichen in seiner Zelle", "items.draw_on", mem({IT_KIND: 1, zp('IT_X'): 6, zp('IT_Y'): 4}),
                  ["[%s] == %d" % (hx(scr(6, 4)), G['CH_STAR'])]))
tests.append(test("items.draw_off: Zelle zeigt wieder die Linie", "items.draw_off", mem({IT_KIND: 2, zp('IT_X'): 6, zp('IT_Y'): 4, hx(scr(6, 4)): G['CH_ICE'], hx(st(6, 4)): COL_LINE}),
                  ["[%s] == %d" % (hx(scr(6, 4)), G['CH_HLINE'])]))

# ---- Spielerwechsel (zwei Spieler)
PSAVE = A['PSAVE']
RESTORE = A['RESTORE']
sw = {zp('NPLAYERS'): 2, zp('CUR'): 0, zp('PFRESH'): [0, 0], zp('LIVES'): 2, zp('LEVEL'): 3, zp('SCORE'): [1, 2, 3], zp('BOXES_DONE'): 5,
      hx(PSAVE): [0] * 48, hx(PSAVE + 48): [5, 1, 3, 0, 0, 0, 0, 0]}
tests.append(test("switch_player: Spieler 1 -> 2: Block gesichert, geladen, STATE = levelinit", "switch_player", mem(sw),
                  ["fc == 1", "[%s] == 1" % zp('CUR'), "[%s] == 1" % zp('STATE'), "[%s] == 5" % zp('SPEED'), "[%s] == 3" % zp('LIVES'), "[%s] == 2" % hx(PSAVE + 2),
                   "[%s] == 3" % hx(PSAVE + 1), "[%s] == 1" % hx(PSAVE + 3), "[%s] == 1" % zp('RESTORE')], timeoutMs=60000))
sw2 = dict(sw)
sw2[hx(PSAVE + 48)] = [5, 1, 0, 0, 0, 0, 0, 0]
tests.append(test("switch_player: anderer Spieler ohne Leben: kein Wechsel", "switch_player", mem(sw2), ["fc == 0", "[%s] == 0" % zp('CUR')]))
sw3 = dict(sw)
sw3[zp('NPLAYERS')] = 1
tests.append(test("switch_player: ein Spieler: kein Wechsel", "switch_player", mem(sw3), ["fc == 0", "[%s] == 0" % zp('CUR')]))

# ---- Eingabe: Anschluss je Spieler
for jm, cur, port in [(0, 0, 1), (0, 1, 1), (1, 0, 2), (1, 1, 2), (2, 0, 1), (2, 1, 2)]:
    tests.append(test("input.port_of: JOYMODE %d, Spieler %d -> Anschluss %d" % (jm, cur + 1, port), "input.port_of", mem({zp('JOYMODE'): jm, zp('CUR'): cur}), ["x == %d" % port]))

# ---- Titelmenue
for nm, row, pl, jm, d, exp in [("Spieler 1 -> 2", 0, 1, 0, 1, (zp('NPLAYERS'), 2)), ("Spieler 2 -> 1 setzt zwei Joysticks zurueck", 0, 2, 2, 255, (zp('JOYMODE'), 0)),
                                ("Joystick Anschluss 1 -> 2 mit einem Spieler", 1, 1, 0, 1, (zp('JOYMODE'), 1)),
                                ("Joystick 2 -> zwei Sticks mit einem Spieler gesperrt", 1, 1, 1, 1, (zp('JOYMODE'), 1)),
                                ("Joystick 2 -> zwei Sticks mit zwei Spielern", 1, 2, 1, 1, (zp('JOYMODE'), 2)), ("Joystick 0 nach links bleibt 0", 1, 2, 0, 255, (zp('JOYMODE'), 0))]:
    tests.append(test("title_change: " + nm, "title_change", dict(a=d, **mem({zp('SELROW'): row, zp('NPLAYERS'): pl, zp('JOYMODE'): jm})), ["[%s] == %d" % exp]))

# ---- Bestenliste
HS_BUF = A['HS_BUF']
HS_SC = A['HS_SC']
tests.append(test("hiscore.default: Kennung, erster Platz 10000 Punkte, letzter 1000", "hiscore.default", None, [
    "[%s] == $54" % hx(HS_BUF), "[%s] == 1" % hx(HS_BUF + 4), "[%s] == 1" % hx(HS_BUF + 10), "[%s] == 0" % hx(HS_BUF + 9),
    "[%s] == $10" % hx(HS_BUF + 5 + 9 * 7 + 4), "[%s] == 20" % hx(HS_BUF + 5)]))
for nm, sc, pl in [("ueber dem ersten Platz (10001)", [1, 0, 1], 0), ("zwischen Platz 3 und 4 (7500)", [0, 0x75, 0], 3), ("genau gleich dem letzten (1000): nicht in der Liste", [0, 0x10, 0], 0xff),
                   ("unter dem letzten (500)", [0, 5, 0], 0xff), ("gleich Platz 1 (10000): dahinter", [0, 0, 1], 1), ("knapp ueber dem letzten (1001)", [1, 0x10, 0], 9)]:
    lst = [0x54, 0x58, 0x48, 0x53, 1]
    mid = [0x00, 0x90, 0x80, 0x70, 0x60, 0x50, 0x40, 0x30, 0x20, 0x10]
    hi = [1] + [0] * 9
    for i in range(10):
        lst += [20, 18, 24, 0, mid[i], hi[i], 1]
    lst += [0, 0]
    tests.append(test("hiscore.place: Endstand %s -> Platz %s" % (nm, "keiner" if pl == 0xff else pl + 1), "hiscore.place", mem({hx(HS_SC): sc, hx(HS_BUF): lst}), ["x == %d" % pl]))
tests.append(test("hiscore.toatari: Namenszeichen A (1) -> 33, Z (26) -> 58, Leerzeichen (32) -> 0", "hiscore.toatari", dict(a=1), ["a == 33"]))
tests.append(test("hiscore.toatari: Z -> 58", "hiscore.toatari", dict(a=26), ["a == 58"]))
tests.append(test("hiscore.toatari: Leerzeichen -> 0", "hiscore.toatari", dict(a=32), ["a == 0"]))

# ---- Klangmodul (Zustand je Stimme; die POKEY-Register sind nur beschreibbar)
S_PRIO = SND + 12
S_TIME = SND + 8
S_INIT = SND + 28
S_AUDC = SND + 24
S_MUSIC = SND + 45
tests.append(test("sound.play: Segment-Ton startet Stimme 1 (Prioritaet 2)", "sound.play", dict(a=1, **mem({hx(SND): [0] * 64, zp('FLAGS'): 1})),
                  ["[%s] == 2" % hx(S_PRIO), "[%s] == 1" % hx(S_TIME), "[%s] == $aa" % hx(S_INIT), "[%s] == $a0" % hx(S_AUDC)]))
tests.append(test("sound.play: gleich wichtiger laufender Effekt wird nicht ersetzt", "sound.play",
                  dict(a=1, **mem({hx(SND): [0] * 12 + [2, 0, 0, 0] + [0] * 48, zp('FLAGS'): 1})), ["[%s] == 2" % hx(S_PRIO), "[%s] == 0" % hx(S_TIME)]))
tests.append(test("sound.play: wichtigerer Effekt (Treffer, Prioritaet 5) ersetzt einen Schritt-Klick", "sound.play",
                  dict(a=4, **mem({hx(SND): [0] * 12 + [0, 0, 0, 1] + [0] * 48, zp('FLAGS'): 1})), ["[%s] == 5" % hx(S_PRIO + 3), "[%s] == 5" % hx(S_PRIO + 2)]))
tests.append(test("sound.play: Kettenreaktion belegt alle vier Stimmen", "sound.play", dict(a=3, **mem({hx(SND): [0] * 64, zp('FLAGS'): 1})),
                  ["[%s] == 4" % hx(S_PRIO), "[%s] == 4" % hx(S_PRIO + 1), "[%s] == 4" % hx(S_PRIO + 2), "[%s] == 4" % hx(S_PRIO + 3)]))
tests.append(test("sound.play: waehrend der Titelmusik keine Effekte", "sound.play", dict(a=4, **mem({hx(SND): [0] * 45 + [1] + [0] * 18, zp('FLAGS'): 1})),
                  ["[%s] == 0" % hx(S_PRIO + 2)]))
tests.append(test("sound.music_start: alle vier Stimmen mit Prioritaet 255, Musik-Flag", "sound.music_start", mem({hx(SND): [0] * 64, zp('FLAGS'): 1}),
                  ["[%s] == 255" % hx(S_PRIO), "[%s] == 255" % hx(S_PRIO + 1), "[%s] == 255" % hx(S_PRIO + 2), "[%s] == 255" % hx(S_PRIO + 3), "[%s] == 1" % hx(S_MUSIC)]))


# ---- Eingabe ueber die PIA (nur im Programm build/pia.xex mit -D TESTPIA=1: PORTA kurz als Ausgang, dann Lesen; TRIG0/TRIG1 lassen sich nicht vorgeben)
pia = []
for nm, port, val, exp in [("Anschluss 1 keine Richtung", 1, 0xff, 0), ("Anschluss 1 hoch", 1, 0xfe, 1), ("Anschluss 1 runter", 1, 0xfd, 2), ("Anschluss 1 links", 1, 0xfb, 4),
                           ("Anschluss 1 rechts", 1, 0xf7, 8), ("Anschluss 1 hoch und rechts", 1, 0xf6, 9), ("Anschluss 1: Anschluss 2 gedrueckt stoert nicht", 1, 0x0f, 0),
                           ("Anschluss 2 keine Richtung", 2, 0xff, 0), ("Anschluss 2 hoch", 2, 0xef, 1), ("Anschluss 2 runter", 2, 0xdf, 2), ("Anschluss 2 links", 2, 0xbf, 4),
                           ("Anschluss 2 rechts", 2, 0x7f, 8), ("Anschluss 2 runter und links", 2, 0x9f, 6), ("Anschluss 2: Anschluss 1 gedrueckt stoert nicht", 2, 0xf0, 0)]:
    pia.append(test("input.read_port: " + nm, "pia_read", dict(a=val, x=port), ["a == %d" % exp]))


def main():
    static = [
        {"check": "expr", "name": "Programm endet vor dem Zeichensatz-RAM ($7000)", "expr": "PRG_END <= $7000"},
        {"check": "expr", "name": "Programm kleiner als 20 KB", "expr": "PRG_END - $2000 <= 20000"},
        {"check": "overlap"},
    ]
    for fn, ts, stt_, prg in (("troxx.vctest", tests, static, "build/main.xex"), ("troxx_pia.vctest", pia, [], "build/pia.xex"), ("troxx_regel0.vctest", RULE0, [], "build/regel0.xex")):
        d = {"kind": "test", "target": "atari8", "program": prg, "static": stt_, "tests": ts}
        with open(fn, "w") as f:
            json.dump(d, f, indent=1, ensure_ascii=False)
            f.write("\n")
        print(fn, len(ts), "Tests")


main()
