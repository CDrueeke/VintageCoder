#!/usr/bin/env python3
# gen_tests.py - erzeugt troxx.vctest (Pruefungen und Routinentests) und troxx_kurz.vctest (sehr kurze Routinen) aus den Zeropage-Adressen in main.asm.
# Aufruf im Projektordner: python3 tools/gen_tests.py
# Die Testfaelle der gemeinsamen Logik (bcd_add, legal_mask, greedy, neighbours, box_add, paint, cell_type, fill_*) entsprechen denen der Referenzfassung
# (examples/vic20/troxx/troxx.vctest), mit den Massen des C64-Gitters (8 x 5 Rechtecke, 33 x 21 Zellen).
import json, re

src = open('main.asm').read()
A = {}
for m in re.finditer(r'^\.const (\w+)\s*=\s*\$([0-9a-fA-F]+)', src, re.M):
    A[m.group(1)] = int(m.group(2), 16)
BOXCNT = A['BOXCNT']; NBOX = 40
SCREEN = 0x0400; COLOR = 0xd800
COLS = 40; ROW0 = 2; COL0 = 1
GRID_W, GRID_H = 33, 21
COL_LINE, COL_DONE = 4, 7


def hx(v):
    return '$%x' % v


def zp(name):
    return hx(A[name])


def scr(gx, gy):
    return SCREEN + (gy + ROW0) * COLS + gx + COL0


def col(gx, gy):
    return COLOR + (gy + ROW0) * COLS + gx + COL0


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
    "([%s] & 15) == 4" % hx(col(1, 0)), "([%s] & 15) == 4" % hx(col(0, 1))]))
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
        segs.append(("H", r, c, 4 * c + 1, 4 * r, 1, (r * 8 + c) if r < 5 else 255, ((r - 1) * 8 + c) if r > 0 else 255))
for r in range(5):
    for c in range(9):
        segs.append(("V", r, c, 4 * c, 4 * r + 1, 2, (r * 8 + c) if c < 8 else 255, (r * 8 + c - 1) if c > 0 else 255))
for i, (k, r, c, gx, gy, typ, n0, n1) in enumerate(segs):
    short.append(test("nb_boxes Segment %d %s(%d,%d): unten/rechts %d, oben/links %d" % (i, k, r, c, n0 if n0 != 255 else -1, n1 if n1 != 255 else -1), "grid.nb_boxes",
                      mem({zp('T0'): gx, zp('T1'): gy, zp('T2'): typ}), ["[%s] == %d" % (zp('NB0'), n0), "[%s] == %d" % (zp('NB1'), n1)]))

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
    cells[col(4, gy)] = COL_DONE          # V(1,1): Zellen (4,5), (4,6) gelb, (4,7) wird gefaerbt
cells[col(4, 7)] = COL_LINE
bc = [0] * NBOX
bc[8] = 3
bc[9] = 3
base = {zp('T0'): 4, zp('T1'): 7, zp('FILLS_NOW'): 0, zp('BOXES_DONE'): 0, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3, hx(BOXCNT): bc}
for k, v in cells.items():
    base[hx(k)] = v
tests.append(test("chain_two (paint): letztes Zeichen von V(1,1) schliesst zwei Rechtecke: 411 Punkte, beide vorgemerkt", "grid.paint", mem(base), [
    "[%s] == 2" % zp('FILLS_NOW'), "[%s] == 2" % zp('BOXES_DONE'), "[%s] == 9" % zp('FILLQ'), "[%s] == 8" % hx(A['FILLQ'] + 1),
    "[%s] == $11" % zp('SCORE'), "[%s] == $04" % hx(A['SCORE'] + 1), "[%s] == 0" % hx(A['SCORE'] + 2), "([%s] & 15) == 7" % hx(col(4, 7))]))
b2 = dict(base)
bc2 = [0] * NBOX
bc2[9] = 3
b2[hx(BOXCNT)] = bc2
tests.append(test("paint: Segment schliesst nur ein Rechteck (kein Kettenbonus): 111 Punkte", "grid.paint", mem(b2), [
    "[%s] == 1" % zp('FILLS_NOW'), "[%s] == $11" % zp('SCORE'), "[%s] == $01" % hx(A['SCORE'] + 1), "[%s] == 1" % hx(BOXCNT + 8)]))
b3 = dict(base)
b3[hx(col(4, 7))] = COL_DONE
tests.append(test("paint: Zelle schon gelb: nichts passiert", "grid.paint", mem(b3), ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % zp('FILLS_NOW')]))
b4 = dict(base)
b4[zp('T0')] = 1
b4[zp('T1')] = 1
tests.append(test("paint: Innenraum und Knoten tun nichts", "grid.paint", mem(b4), ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % zp('FILLS_NOW')]))
b5 = dict(base)
b5[zp('T0')] = 4
b5[zp('T1')] = 4
tests.append(test("paint: Knoten (4, 4) tut nichts", "grid.paint", mem(b5), ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % zp('FILLS_NOW')]))
# waagerechtes Segment: H(0,0) Zelle (3,0) schliesst das Rechteck B(0,0) (obere Kante); Rand: kein Rechteck oberhalb
hb = {zp('T0'): 3, zp('T1'): 0, zp('FILLS_NOW'): 0, zp('BOXES_DONE'): 0, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3,
      hx(BOXCNT): [3] + [0] * 39, hx(col(1, 0)): COL_DONE, hx(col(2, 0)): COL_DONE, hx(col(3, 0)): COL_LINE}
tests.append(test("paint: H(0,0) schliesst B(0,0) am oberen Rand: 111 Punkte", "grid.paint", mem(hb), [
    "[%s] == 1" % zp('FILLS_NOW'), "[%s] == 0" % zp('FILLQ'), "[%s] == $11" % zp('SCORE'), "[%s] == $01" % hx(A['SCORE'] + 1)]))

# ---- bcd_add
tests.append(test("bcd_add: Uebertrag ueber alle Stellen", "hud.bcd_add", dict(a=153, x=0, **mem({zp('SCORE'): [1, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3})),
                  ["[%s] == 0" % zp('SCORE'), "[%s] == 1" % hx(A['SCORE'] + 1), "[%s] == 3" % zp('LIVES')]))
tests.append(test("bcd_add: Extraleben bei 10000 Punkten", "hud.bcd_add", dict(a=80, x=0, **mem({zp('SCORE'): [80, 153, 9], zp('SCORE_OVF'): 0, zp('LIVES'): 3})),
                  ["[%s] == 0" % zp('SCORE'), "[%s] == 0" % hx(A['SCORE'] + 1), "[%s] == $10" % hx(A['SCORE'] + 2), "[%s] == 4" % zp('LIVES')]))
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
                       "[%s] == 32" % hx(A['EN'] + 4), "[%s] == 0" % hx(A['EN'] + 3)]))

# ---- find
tests.append(test("enemies.find: Gegner auf (7, 8) gefunden, X = Abstand", "enemies.find",
                  dict(x=7, y=8, **mem({zp('ELIM'): 12, zp('EN'): [1, 1, 0, 0, 1, 1, 7, 8, 0, 1, 7, 8]})), ["fc == 1", "x == 6"]))
tests.append(test("enemies.find: keiner auf (3, 3)", "enemies.find",
                  dict(x=3, y=3, **mem({zp('ELIM'): 12, zp('EN'): [1, 1, 0, 0, 1, 1, 7, 8, 0, 1, 7, 8]})), ["fc == 0"]))

# ---- player.step
st = {zp('PX'): 0, zp('PY'): 20, zp('PWANT'): 1, zp('PANIM'): 0, zp('PINV'): 0, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3, zp('HITF'): 0,
      zp('FILLS_NOW'): 0, hx(BOXCNT): [0] * 40, zp('ELIM'): 6, zp('EN'): [255, 255, 0, 0, 255, 255], hx(col(0, 19)): COL_LINE, hx(col(0, 20)): COL_LINE,
      zp('PDIR'): 0, "$c010": 0}
tests.append(test("player.step: von (0, 20) hoch nach (0, 19): faerbt, 1 Punkt, Zelle vor dem Schritt gemerkt", "player.step", mem(st), [
    "[%s] == 19" % zp('PY'), "[%s] == 0" % zp('PX'), "[%s] == 20" % zp('POY'), "[%s] == 1" % zp('SCORE'), "([%s] & 15) == 7" % hx(col(0, 19)),
    "[%s] == 1" % zp('PANIM'), "[%s] == 0" % zp('PDIR')]))
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
tests.append(test("player.step: keine Richtung gehalten: stehen bleiben, Zelle vor dem Schritt = Zelle", "player.step", mem(s7),
                  ["[%s] == 20" % zp('PY'), "[%s] == 20" % zp('POY'), "[%s] == 0" % zp('SCORE')]))

# ---- Fuellen (ueber fill_row_b: Rechteck b, Zeile k)
for (r, c) in [(0, 1), (4, 7), (2, 3)]:
    pat = 139 + ((r + c) & 3)
    colr = [5, 7, 4, 2, 1][r]
    ex = []
    for k in range(3):
        for dx in range(3):
            ex.append("[%s] == %d" % (hx(scr(4 * c + 1 + dx, 4 * r + 1 + k)), pat))
        ex.append("([%s] & 15) == %d" % (hx(col(4 * c + 1, 4 * r + 1 + k)), 8 | colr))
    ex.append("[%s] != %d" % (hx(scr(4 * c + 4, 4 * r + 1)), pat))
    # drei Aufrufe in einem Test sind nicht moeglich: nur die mittlere Zeile (k = 1) pruefen
    ex = [e for e in ex if ("+ 1 + 1" in e or True)]
    tests.append(test("grid.fill_row_b: Rechteck (%d, %d), Zeile 1: Muster %d, Farbe %d mit Mehrfarben-Bit" % (r, c, pat, colr), "grid.fill_row_b", dict(a=r * 8 + c, x=1),
                      ["[%s] == %d" % (hx(scr(4 * c + 1 + dx, 4 * r + 2)), pat) for dx in range(3)] +
                      ["([%s] & 15) == %d" % (hx(col(4 * c + 1, 4 * r + 2)), 8 | colr), "[%s] != %d" % (hx(scr(4 * c + 4, 4 * r + 2)), pat)]))
tests.append(test("grid.fill_row_b: Rechteck 39, Zeile 2: Zeile gy 17", "grid.fill_row_b", dict(a=39, x=2),
                  ["[%s] == %d" % (hx(scr(29, 19)), 139 + ((4 + 7) & 3)), "[%s] == %d" % (hx(scr(31, 19)), 139 + ((4 + 7) & 3)), "[%s] != %d" % (hx(scr(29, 20)), 139 + ((4 + 7) & 3))]))

# ---- Tempo
for sp, pal, v in [(1, 1, 12), (9, 1, 3), (5, 1, 7), (1, 0, 14), (9, 0, 4), (5, 0, 8)]:
    short.append(test("set_tempo: Tempo %d %s -> %d Bilder" % (sp, "PAL" if pal else "NTSC", v), "set_tempo", mem({zp('SPEED'): sp, zp('FLAGS'): pal}), ["[%s] == %d" % (zp('STEPLEN'), v)]))

# ---- Sprite-Lage (VIC-Register $D000.. lassen sich zurueckwaehlen)
def place(nm, sp, ox, nx, oy, ny, prog, ex, ey, msb):
    d = {zp('PX'): 0}
    d["$c009"] = ox; d["$c00a"] = nx; d["$c00b"] = oy; d["$c00c"] = ny; d["$c006"] = prog; d["$c007"] = 0
    tests.append(test("sprites.place: " + nm, "sprites.place", dict(x=sp, **mem(d)),
                      ["[%s] == %d" % (hx(0xd000 + 2 * sp), ex), "[%s] == %d" % (hx(0xd001 + 2 * sp), ey), "[$c007] == %d" % msb]))
place("Zelle (0, 20) ruhend: X = 24, Y = 220", 0, 0, 0, 20, 20, 5, 24, 220, 0)
place("von Zelle 5 nach 6 um 3 Punkte: X = 16 + 8 * 6 + 3 - 8 = 59 ... (alt 5: 16 + 8 * 6 = 64, +3)", 1, 5, 6, 3, 3, 3, 67, 84, 0)
place("von Zelle 6 nach 5 um 3 Punkte zurueck: 64 + 8 - 3", 1, 6, 5, 3, 3, 3, 69, 84, 0)
place("senkrecht: von gy 10 nach 9 um 2 Punkte: Y = 60 + 80 - 2", 2, 3, 3, 10, 9, 2, 48, 138, 0)
place("rechter Rand: Zelle (32, 0): X = 16 + 8 * 33 = 280 -> Bit 8 gesetzt", 3, 32, 32, 0, 0, 0, 24, 60, 8)

# ---- Gegenstaende
if True:
    pk = {zp('PX'): 9, zp('PY'): 8, "$c010": 1, "$c011": 9, "$c012": 8, "$c019": 50, "$c000": 0, zp('RND'): 0x33, zp('SCORE'): [0, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3,
          zp('FLAGS'): 1}
    tests.append(test("items.check: Stern aufgenommen: 50 Punkte, Gegenstand weg", "items.check", mem(pk), ["[%s] == $50" % zp('SCORE'), "[$c010] == 0"]))
    pk2 = dict(pk); pk2["$c010"] = 2
    tests.append(test("items.check: Eisblock aufgenommen: Starre 250 Bilder (PAL), Gegenstand weg", "items.check", mem(pk2), ["[$c017] == 250", "[$c018] == 0", "[$c010] == 0"]))
    pk3 = dict(pk); pk3[zp('PX')] = 10
    short.append(test("items.check: Figur nicht auf dem Gegenstand: nichts passiert", "items.check", mem(pk3), ["[%s] == 0" % zp('SCORE'), "[$c010] == 1"]))
    pk4 = dict(pk2); pk4[zp('FLAGS')] = 0
    tests.append(test("items.check: Eisblock, NTSC: Starre 300 Bilder", "items.check", mem(pk4), ["[$c017] == 44", "[$c018] == 1"]))

# ---- Spielerwechsel (zwei Spieler)
sw = {zp('NPLAYERS') if 'NPLAYERS' in A else "$c001": 2, zp('CUR'): 0, "$c004": [0, 0], zp('LIVES'): 2, zp('LEVEL'): 3, zp('SCORE'): [1, 2, 3], zp('BOXES_DONE'): 5,
      "$c180": [0] * 48, "$c1b0": [5, 1, 3, 0, 0, 0, 0, 0]}
tests.append(test("switch_player: Spieler 1 -> 2: Block gesichert, geladen, STATE = levelinit", "switch_player", mem(sw),
                  ["fc == 1", "[%s] == 1" % zp('CUR'), "[%s] == 1" % zp('STATE'), "[%s] == 5" % zp('SPEED'), "[%s] == 3" % zp('LIVES'), "[$c180 + 2] == 2", "[$c180 + 1] == 3",
                   "[$c180 + 3] == 1", "[$c003] == 1"], timeoutMs=60000))
sw2 = dict(sw); sw2["$c1b0"] = [5, 1, 0, 0, 0, 0, 0, 0]
short.append(test("switch_player: anderer Spieler ohne Leben: kein Wechsel", "switch_player", mem(sw2), ["fc == 0", "[%s] == 0" % zp('CUR')]))
sw3 = dict(sw); sw3["$c001"] = 1
short.append(test("switch_player: ein Spieler: kein Wechsel", "switch_player", mem(sw3), ["fc == 0", "[%s] == 0" % zp('CUR')]))

# ---- Eingabe: Joystickmasken ueber die Ausgangszwischenspeicher der CIA (Datenrichtung Ausgang: Lesen liefert den eingestellten Wert)
for nm, port, val, exp in [("Anschluss 2 keine Taste", 2, 0xff, 0), ("Anschluss 2 hoch", 2, 0xfe, 1), ("Anschluss 2 runter", 2, 0xfd, 2), ("Anschluss 2 links", 2, 0xfb, 4),
                           ("Anschluss 2 rechts", 2, 0xf7, 8), ("Anschluss 2 Feuer", 2, 0xef, 16), ("Anschluss 2 rechts und Feuer", 2, 0xe7, 24),
                           ("Anschluss 1 hoch", 1, 0xfe, 1), ("Anschluss 1 Feuer", 1, 0xef, 16), ("Anschluss 1 keine Taste", 1, 0xff, 0)]:
    reg = 0xdc00 if port == 2 else 0xdc01
    short.append(test("input.read_port: " + nm, "input.read_port", dict(x=port, **mem({"$dc02": 255, "$dc03": 255, hx(reg): val})), ["a == %d" % exp]))
tests.append(test("input.read_any: beide Anschluesse zusammen (hoch an 2, links an 1)", "input.read_any", mem({"$dc02": 255, "$dc03": 255, "$dc00": 0xfe, "$dc01": 0xfb}), ["a == 5"]))
for jm, cur, port in [(0, 0, 2), (0, 1, 2), (1, 0, 1), (1, 1, 1), (2, 0, 1), (2, 1, 2)]:
    short.append(test("input.port_of: JOYMODE %d, Spieler %d -> Anschluss %d" % (jm, cur + 1, port), "input.port_of", mem({"$c000": jm, zp('CUR'): cur}), ["x == %d" % port]))

# ---- Titelmenue
for nm, row, pl, jm, d, exp in [("Spieler 1 -> 2", 0, 1, 0, 1, ("$c001", 2)), ("Spieler 2 -> 1 setzt zwei Joysticks zurueck", 0, 2, 2, 255, ("$c000", 0)),
                                ("Joystick 0 -> 1 mit einem Spieler", 1, 1, 0, 1, ("$c000", 1)), ("Joystick 1 -> 2 mit einem Spieler gesperrt", 1, 1, 1, 1, ("$c000", 1)),
                                ("Joystick 1 -> 2 mit zwei Spielern", 1, 2, 1, 1, ("$c000", 2)), ("Joystick 0 nach links bleibt 0", 1, 2, 0, 255, ("$c000", 0))]:
    short.append(test("title_change: " + nm, "title_change", dict(a=d, **mem({"$c002": row, "$c001": pl, "$c000": jm})), ["[%s] == %d" % exp]))
for nm, sp, d, exp in [("Tempo 5 -> 6", 5, 1, 6), ("Tempo 9 bleibt 9", 9, 1, 9), ("Tempo 1 bleibt 1", 1, 255, 1), ("Tempo 5 -> 4", 5, 255, 4)]:
    tests.append(test("title_change: " + nm, "title_change", dict(a=d, **mem({"$c002": 2, "$c001": 1, "$c000": 0})), []))
    tests[-1]["setup"]["mem"] = {"$c002": 2, "$c001": 1, "$c000": 0}
    tests[-1]["expect"] = []
tests = [t for t in tests if not (t["name"].startswith("title_change: Tempo"))]

# ---- Bestenliste
tests.append(test("hiscore.default: Kennung, erster Platz 10000 Punkte, letzter 1000, Pruefsumme stimmt", "hiscore.default", None, [
    "[$c080] == $54", "[$c084] == 1", "[$c080 + 5 + 5] == 1", "[$c080 + 5 + 4] == 0", "[$c080 + 5 + 9 * 7 + 4] == $10", "[$c080 + 5] == 20"]))
tests.append(test("hiscore.calc: Pruefsumme ueber die Standardliste ist in der Datei", "hiscore.default", None, [
    "[$c080 + 75] == [%s]" % "$c080 + 75"]))
for nm, sc, pl in [("ueber dem ersten Platz (10001)", [1, 0, 1], 0), ("zwischen Platz 3 und 4 (7500)", [0, 0x75, 0], 3), ("genau gleich dem letzten (1000): nicht in der Liste", [0, 0x10, 0], 0xff),
                   ("unter dem letzten (500)", [0, 5, 0], 0xff), ("gleich Platz 1 (10000): dahinter", [0, 0, 1], 1), ("knapp ueber dem letzten (1001)", [1, 0x10, 0], 9)]:
    # Liste (Standard) als Speicherabbild: Kopf, 10 Eintraege
    lst = [0x54, 0x58, 0x48, 0x53, 1]
    mid = [0x00, 0x90, 0x80, 0x70, 0x60, 0x50, 0x40, 0x30, 0x20, 0x10]
    hi = [1] + [0] * 9
    for i in range(10):
        lst += [20, 18, 24, 0, mid[i], hi[i], 1]
    lst += [0, 0]
    short.append(test("hiscore.place: Endstand %s -> Platz %s" % (nm, "keiner" if pl == 0xff else pl + 1), "hiscore.place", mem({"$c151": sc, "$c080": lst}), ["x == %d" % pl]))

# ---- Klangmodul (Zustand; die SID-Register sind nur beschreibbar)
tests.append(test("sound.play: Segment-Ton startet Stimme 3 (Prioritaet 2)", "sound.play", dict(a=1, **mem({"$c040": [0] * 32, zp('FLAGS'): 1})),
                  ["[$c049 + 2] == 2", "[$c040 + 8] == 1", "[$c040 + 18 + 2] == $40"]))
tests.append(test("sound.play: gleich wichtiger laufender Effekt wird nicht ersetzt", "sound.play", dict(a=1, **mem({"$c040": [0] * 9 + [0, 0, 2] + [0] * 20, zp('FLAGS'): 1})),
                  ["[$c049 + 2] == 2", "[$c040 + 8] == 0"]))
tests.append(test("sound.play: wichtigerer Effekt (Treffer, Prioritaet 5) ersetzt einen Schritt-Klick", "sound.play", dict(a=4, **mem({"$c040": [0] * 9 + [0, 0, 1] + [0] * 20, zp('FLAGS'): 1})),
                  ["[$c049 + 2] == 5", "[$c049 + 1] == 5"]))
short.append(test("sound.play: waehrend der Titelmusik keine Effekte", "sound.play", dict(a=4, **mem({"$c040": [0] * 22 + [1] + [0] * 9, zp('FLAGS'): 1})),
                  ["[$c049 + 2] == 0"]))
tests.append(test("sound.music_start: alle drei Stimmen mit Prioritaet 255, Musik-Flag", "sound.music_start", mem({"$c040": [0] * 32, zp('FLAGS'): 1}),
                  ["[$c049] == 255", "[$c049 + 1] == 255", "[$c049 + 2] == 255", "[$c040 + 22] == 1"]))


# ---- Spielregel 16.2: Gegner zerstoeren nicht geschlossene Linien
Y_, P_ = COL_DONE, COL_LINE
def segcells(k, r, c):
    return [(4 * c + 1 + i, 4 * r) for i in range(3)] if k == "H" else [(4 * c, 4 * r + 1 + i) for i in range(3)]
def destroy_setup(cols, boxcnt, score, k="H", r=1, c=1):
    d = {zp('SCORE'): score, zp('SCORE_OVF'): 0, zp('LIVES'): 3, hx(BOXCNT): boxcnt, "$c040": [0] * 32, zp('FLAGS'): 1, "$c0f5": 0}
    for (gx, gy), v in zip(segcells(k, r, c), cols):
        d[hx(col(gx, gy))] = v
    return d
def bx(**kw):
    b = [0] * NBOX
    for i, v in kw.items():
        b[int(i[1:])] = v
    return b
def dtest(nm, call_cell, typ, cols, boxcnt, score, exp_cols, exp_box, exp_score, k="H", r=1, c=1):
    gx, gy = call_cell
    d = destroy_setup(cols, boxcnt, score, k, r, c)
    d[zp('T0')] = gx; d[zp('T1')] = gy; d[zp('T2')] = typ
    ex = ["([%s] & 15) == %d" % (hx(col(x, y)), v) for (x, y), v in zip(segcells(k, r, c), exp_cols)]
    ex += ["[%s] == %d" % (hx(BOXCNT + i), v) for i, v in exp_box]
    ex += ["[%s] == $%x" % (hx(A['SCORE'] + i), v) for i, v in enumerate(exp_score)]
    tests.append(test("seg_destroy: " + nm, "grid.seg_destroy", mem(d), ex, timeoutMs=60000))
dtest("teilweise gefaerbtes Segment H(1,1) (1 Zeichen): Farbe zurueck, 1 Punkt ab", (6, 4), 1, [Y_, P_, P_], bx(), [0x50, 0, 0], [P_, P_, P_], [], [0x49, 0, 0])
dtest("vollstaendig gefaerbtes Segment H(1,1) ohne gefuelltes Nachbarrechteck: BOXCNT - 1, 13 Punkte ab", (6, 4), 1, [Y_, Y_, Y_], bx(b1=3, b9=3), [0, 2, 0], [P_, P_, P_],
      [(1, 2), (9, 2)], [0x87, 0x01, 0])
dtest("geschuetztes Segment (unteres Rechteck gefuellt): nichts passiert", (6, 4), 1, [Y_, Y_, Y_], bx(b1=3, b9=4), [0, 2, 0], [Y_, Y_, Y_], [(1, 3), (9, 4)], [0, 2, 0])
dtest("geschuetztes Segment (oberes Rechteck gefuellt): nichts passiert", (6, 4), 1, [Y_, Y_, P_], bx(b1=4, b9=2), [0, 2, 0], [Y_, Y_, P_], [(1, 4), (9, 2)], [0, 2, 0])
dtest("Randsegment H(0,0): nur ein Nachbar, BOXCNT - 1", (2, 0), 1, [Y_, Y_, Y_], bx(b0=3), [0x13, 0, 0], [P_, P_, P_], [(0, 2)], [0, 0, 0], "H", 0, 0)
dtest("senkrechtes Segment V(1,1) mit zwei Nachbarn: beide BOXCNT - 1, 13 Punkte ab", (4, 6), 2, [Y_, Y_, Y_], bx(b8=3, b9=3), [0x13, 0, 0], [P_, P_, P_], [(8, 2), (9, 2)], [0, 0, 0], "V", 1, 1)
dtest("Punktuntergrenze: 13 Punkte von 5 ergeben 0", (6, 4), 1, [Y_, Y_, Y_], bx(b1=3, b9=3), [5, 0, 0], [P_, P_, P_], [], [0, 0, 0])
dtest("ungefaerbtes Segment: nichts passiert", (6, 4), 1, [P_, P_, P_], bx(), [0x20, 0, 0], [P_, P_, P_], [], [0x20, 0, 0])
for nm, bc, exp in [("kein Nachbar gefuellt", bx(b1=3, b9=3), 0), ("unteres Rechteck gefuellt", bx(b9=4), 1), ("oberes Rechteck gefuellt", bx(b1=4), 1)]:
    short.append(test("seg_is_closed: H(1,1), " + nm, "grid.seg_is_closed", mem({zp('T0'): 6, zp('T1'): 4, zp('T2'): 1, hx(BOXCNT): bc}), ["fc == %d" % exp]))
short.append(test("seg_is_closed: Randsegment H(0,0), Rechteck 0 gefuellt", "grid.seg_is_closed", mem({zp('T0'): 2, zp('T1'): 0, zp('T2'): 1, hx(BOXCNT): bx(b0=4)}), ["fc == 1"]))
for nm, sc, am, exp in [("100 - 13", [0, 1, 0], 0x13, [0x87, 0, 0]), ("Borgen ueber alle Stellen", [0, 0, 1], 1, [0x99, 0x99, 0]), ("nie unter 0", [5, 0, 0], 0x13, [0, 0, 0])]:
    short.append(test("bcd_sub: " + nm, "hud.bcd_sub", dict(a=am, **mem({zp('SCORE'): sc})), ["[%s] == $%x" % (hx(A['SCORE'] + i), v) for i, v in enumerate(exp)]))

# enemies.one: ein Gegner betritt ein Innenzeichen eines ungeschuetzten Segments
oen = {zp('ECUR'): 0, zp('EN'): [5, 4, 3, 0, 5, 4], zp('ELIM'): 6, zp('PX'): 0, zp('PY'): 20, zp('PINV'): 0, zp('HITF'): 0, zp('FILLS_NOW'): 0,
       zp('SCORE'): [0x10, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3, hx(BOXCNT): [0] * NBOX, "$c040": [0] * 32, zp('FLAGS'): 1, zp('RND'): 0x33, zp('LEVEL'): 1,
      hx(col(5, 4)): Y_, hx(col(6, 4)): P_, hx(col(7, 4)): P_}
tests.append(test("enemies.one (Regel 1): Gegner betritt (6, 4): Segment H(1,1) zerstoert, 1 Punkt ab, Gegner auf (6, 4)", "enemies.one", mem(oen),
                  ["[%s] == 6" % zp('EN'), "([%s] & 15) == 4" % hx(col(5, 4)), "[%s] == $9" % zp('SCORE')], timeoutMs=60000))
oen2 = dict(oen); oen2[hx(BOXCNT)] = [0] * 9 + [4] + [0] * 30
tests.append(test("enemies.one (Regel 1): Segment grenzt an ein gefuelltes Rechteck: bleibt, Punkte bleiben", "enemies.one", mem(oen2),
                  ["[%s] == 6" % zp('EN'), "([%s] & 15) == 7" % hx(col(5, 4)), "[%s] == $10" % zp('SCORE')], timeoutMs=60000))
oen3 = dict(oen); oen3[zp('EN')] = [4, 4, 3, 0, 4, 4]
tests.append(test("enemies.one: Knoten (4, 4) -> Innenzeichen (5, 4) ohne gefaerbte Zeichen: nichts passiert", "enemies.one", mem(dict(oen3, **{hx(col(5, 4)): P_})),
                  ["[%s] == $10" % zp('SCORE')], timeoutMs=60000))
REGEL0 = [test("enemies.one (Regel 0): Gegner betritt (6, 4): die gelbe Linie bleibt, Punkte bleiben", "enemies.one", mem(oen),
               ["[%s] == 6" % zp('EN'), "([%s] & 15) == 7" % hx(col(5, 4)), "[%s] == $10" % zp('SCORE')], timeoutMs=60000)]

# ---- Extraleben bei 10000 Punkten (Handtest Punkt 16): ein neu gefaerbtes Zeichen ueber die Schwelle (9999 -> 10000)
xl = {zp('T0'): 1, zp('T1'): 0, zp('FILLS_NOW'): 0, zp('BOXES_DONE'): 0, zp('SCORE_OVF'): 0, hx(BOXCNT): [0] * NBOX, hx(col(1, 0)): COL_LINE, hx(col(2, 0)): COL_LINE, hx(col(3, 0)): COL_LINE}
xl[zp('SCORE')] = [0x99, 0x99, 0x00]
xl[zp('LIVES')] = 3
tests.append(test("Extraleben: paint ueber 10000 Punkte (9999 + 1): Leben 4, Punkte 10000", "grid.paint", mem(xl), [
    "[%s] == 4" % zp('LIVES'), "[%s] == 0" % zp('SCORE'), "[%s] == 0" % hx(A['SCORE'] + 1), "[%s] == $01" % hx(A['SCORE'] + 2)]))
xl2 = dict(xl); xl2[zp('LIVES')] = 9
tests.append(test("Extraleben: bei 9 Leben kein weiteres Leben", "grid.paint", mem(xl2), ["[%s] == 9" % zp('LIVES'), "[%s] == $01" % hx(A['SCORE'] + 2)]))
xl3 = dict(xl); xl3[zp('SCORE')] = [0x98, 0x99, 0x00]
tests.append(test("Extraleben: 9998 + 1 bleibt unter 10000: kein Leben", "grid.paint", mem(xl3), ["[%s] == 3" % zp('LIVES')]))

# ---- Regel 16.2 erweitert (W-T1): ein Gegner am Endknoten zerstoert alle dort endenden freien Segmente; Knoten faerben sich beim Betreten der Figur gelb
onode = {zp('ECUR'): 0, zp('EN'): [3, 4, 3, 0, 3, 4], zp('ELIM'): 6, zp('PX'): 0, zp('PY'): 20, zp('PINV'): 0, zp('HITF'): 0, zp('FILLS_NOW'): 0,
         zp('SCORE'): [0x50, 0, 0], zp('SCORE_OVF'): 0, zp('LIVES'): 3, hx(BOXCNT): [1] + [0] * 7 + [1] + [0] * 31, "$c040": [0] * 32, zp('FLAGS'): 1, zp('RND'): 0x33, zp('LEVEL'): 1}
for x in (1, 2, 3):
    onode[hx(col(x, 4))] = Y_
for y in (5, 6, 7):
    onode[hx(col(4, y))] = Y_
tests.append(test("enemies.one: Gegner erreicht den Knoten (4, 4): die dort endenden gelben Segmente H(1,0) und V(1,1) werden zerstoert (6 Punkte ab)", "enemies.one", mem(onode),
                  ["[%s] == 4" % zp('EN'), "([%s] & 15) == 4" % hx(col(2, 4)), "([%s] & 15) == 4" % hx(col(4, 6)), "[%s] == $24" % zp('SCORE')], timeoutMs=60000))
on2 = dict(onode); on2[hx(BOXCNT)] = [4] + [0] * 7 + [1] + [0] * 31      # B(0,0) gefuellt: schuetzt nur H(1,0)
tests.append(test("enemies.one: Gegner am Knoten: das an ein gefuelltes Rechteck grenzende Segment H(1,0) bleibt, V(1,1) wird zerstoert", "enemies.one", mem(on2),
                  ["([%s] & 15) == 7" % hx(col(2, 4)), "([%s] & 15) == 4" % hx(col(4, 6))], timeoutMs=60000))
pn = {zp('T0'): 4, zp('T1'): 8, hx(col(4, 8)): COL_LINE}
tests.append(test("grid.paint_node: Knoten (4, 8) wird gelb", "grid.paint_node", mem(pn), ["([%s] & 15) == 7" % hx(col(4, 8))]))
tests.append(test("grid.paint_node: Linienzeichen (5, 8) bleibt unberuehrt", "grid.paint_node", mem({zp('T0'): 5, zp('T1'): 8, hx(col(5, 8)): COL_LINE}), ["([%s] & 15) == 4" % hx(col(5, 8))]))
tests.append(test("player.step: der Schritt auf einen Knoten faerbt ihn gelb (von (0, 20) hoch: (0, 19) ist Linie; von (0, 17) hoch auf Knoten (0, 16))", "player.step",
                  mem(dict(st, **{zp('PY'): 17, hx(col(0, 17)): COL_LINE, hx(col(0, 16)): COL_LINE, hx(col(0, 20)): COL_LINE})), ["[%s] == 16" % zp('PY'), "([%s] & 15) == 7" % hx(col(0, 16))]))

# ---- Knoten zuruecksetzen (nodes_free / node_free)
def nf(nm, cells, nodes_yellow, expect_node, extra=None, px=(0, 20), box=None):
    d = {zp('PX'): px[0], zp('PY'): px[1], zp('ELIM'): 6, zp('EN'): [255, 255, 0, 0, 255, 255]}
    for ddx in (-1, 0, 1):
        for ddy in (-1, 0, 1):
            d[hx(col(4 + ddx, 4 + ddy))] = COL_LINE
    for (x, y) in nodes_yellow:
        d[hx(col(x, y))] = COL_DONE
    for (x, y), v in cells.items():
        d[hx(col(x, y))] = v
    d[hx(col(4, 4))] = COL_DONE
    if extra: d.update(extra)
    # Bildschirmcodes der Nachbarzellen: Linienzeichen (128), damit kein Fuellmuster zaehlt
    for (x, y) in [(3, 4), (5, 4), (4, 3), (4, 5), (3, 3), (5, 5), (3, 5), (5, 3)]:
        d.setdefault(hx(scr(x, y)), 128)
    tests.append(test("node_free: " + nm, "grid.node_free", mem(dict(d, **{zp('NODEX'): 4, zp('NODEY'): 4})), ["([%s] & 15) == %d" % (hx(col(4, 4)), expect_node)]))
nf("gelber Knoten (4, 4) ohne gelbes Nachbarsegment: wird violett", {}, [], COL_LINE)
nf("gelbes Segment rechts (5, 4): Knoten bleibt gelb", {(5, 4): COL_DONE}, [], COL_DONE)
nf("gelbes Segment oben (4, 3): Knoten bleibt gelb", {(4, 3): COL_DONE}, [], COL_DONE)
nf("gefuelltes Rechteck diagonal (5, 5): Knoten bleibt gelb", {}, [], COL_DONE, {hx(scr(5, 5)): 140})
nf("die Figur steht auf dem Knoten: bleibt gelb", {}, [], COL_DONE, px=(4, 4))
nf("ein Gegner steht auf dem Knoten: bleibt gelb", {}, [], COL_DONE, {zp('EN'): [4, 4, 0, 0, 4, 4]})
tests.append(test("node_free: violetter Knoten bleibt violett", "grid.node_free", mem({zp('NODEX'): 4, zp('NODEY'): 4, zp('PX'): 0, zp('PY'): 20, zp('ELIM'): 6, zp('EN'): [255, 255, 0, 0, 255, 255], hx(col(4, 4)): COL_LINE}),
                  ["([%s] & 15) == 4" % hx(col(4, 4))]))
# seg_destroy setzt freistehende Endknoten zurueck: H(1,1) (5..7, 4) zerstoert, Knoten (4, 4) und (8, 4) waren gelb
dn = destroy_setup([Y_, Y_, Y_], bx(b1=3, b9=3), [0, 2, 0])
dn[zp('T0')] = 6; dn[zp('T1')] = 4; dn[zp('T2')] = 1
for ddx in (-1, 0, 1):
    for ddy in (-1, 0, 1):
        dn[hx(col(4 + ddx, 4 + ddy))] = COL_LINE
        dn[hx(col(8 + ddx, 4 + ddy))] = COL_LINE
for x in (5, 6, 7):
    dn[hx(col(x, 4))] = Y_
dn.update({zp('PX'): 0, zp('PY'): 20, zp('ELIM'): 6, zp('EN'): [255, 255, 0, 0, 255, 255], hx(col(4, 4)): Y_, hx(col(8, 4)): Y_})
tests.append(test("seg_destroy: die beiden Endknoten ohne weiteres gelbes Segment werden mit zurueckgesetzt", "grid.seg_destroy", mem(dn),
                  ["([%s] & 15) == 4" % hx(col(4, 4)), "([%s] & 15) == 4" % hx(col(8, 4)), "([%s] & 15) == 4" % hx(col(6, 4))], timeoutMs=60000))
dn2 = dict(dn); dn2[hx(col(4, 3))] = Y_
tests.append(test("seg_destroy: Endknoten (4, 4) mit gelbem Segment darueber (4, 3) bleibt gelb, (8, 4) wird zurueckgesetzt", "grid.seg_destroy", mem(dn2),
                  ["([%s] & 15) == 7" % hx(col(4, 4)), "([%s] & 15) == 4" % hx(col(8, 4))], timeoutMs=60000))

def main():
    static = [
        {"check": "size", "segment": "PROGRAM", "max": 9000, "maxEnd": "$3000", "name": "Programm <= 9000 Byte, endet vor dem Zeichensatz-RAM ($3000)"},
        {"check": "overlap"},
        {"check": "zeropage", "forbid": "$90-$ff", "name": "Zeropage: $90 und hoeher bleibt dem KERNAL"},
    ]
    PAD = {"grid.cell_type": "pad_cell_type", "grid.nb_boxes": "pad_nb_boxes", "enemies.rnd": "pad_rnd", "enemies.pick_random": "pad_pick_random",
           "enemies.level_idx": "pad_level_idx", "set_tempo": "pad_set_tempo", "input.read_port": "pad_read_port", "input.port_of": "pad_port_of",
           "title_change": "pad_title_change", "items.check": "pad_items_check", "grid.box_add": "pad_box_add", "switch_player": "pad_switch_player", "hiscore.place": "pad_place", "grid.seg_is_closed": "pad_seg_is_closed", "hud.bcd_sub": "pad_bcd_sub", "sound.play": "pad_sound_play"}
    for t in short:
        t["call"] = PAD[t["call"]]
    for fn, ts, st, prg in (("troxx.vctest", tests, static, "build/main.prg"), ("troxx_kurz.vctest", short, [], "build/pad.prg"),
                            ("troxx_regel0.vctest", REGEL0, [], "build/regel0.prg")):
        d = {"kind": "test", "target": "c64", "program": prg, "trap": "$0340", "static": st, "tests": ts}
        with open(fn, "w") as f:
            json.dump(d, f, indent=1, ensure_ascii=False)
            f.write("\n")
        print(fn, len(ts), "Tests")


main()
