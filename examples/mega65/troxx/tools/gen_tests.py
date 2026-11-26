#!/usr/bin/env python3
# gen_tests.py - erzeugt troxx.vctest (statische Pruefungen und Routinentests im Emulator) aus den Zeropage-Adressen in main.asm.
# Aufruf im Projektordner: python3 tools/gen_tests.py
# Die Testfaelle der gemeinsamen Logik (bcd_add, legal_mask, greedy, nb_boxes, find, place_all, hs_check) entsprechen denen der Referenzfassungen
# (examples/c64/troxx/troxx.vctest), mit den Massen des MEGA65-Gitters (14 x 7 Rechtecke, 71 x 43 Zellen).
#
# MEGA65: Der Testlaeufer startet Xemu mit dem Original-ROM; darin liegt das ROM-BASIC ueber $2000-$7FFF (und weitere ROMs ueber $8000-$FFFF), so dass Code,
#   den der Laeufer ins RAM schreibt, nicht ausgefuehrt wird.  Der erste Test ("Vorbereitung") blendet die ROMs aus (MAP 0, $D030 = 0, kein 8-Bit-Stapel noetig);
#   er laeuft vor allen anderen und das ROM bleibt danach aus.  Nach einer Zeitueberschreitung startet der Laeufer Xemu neu: dann schlaegt der Rest der
#   Datei fehl (das ist dann der erste Fehler, nicht die folgenden).
import json, re

src = open('main.asm', encoding='utf-8').read()
A = {}
for m in re.finditer(r'^\.const (\w+)\s*=\s*\$([0-9a-fA-F]+)', src, re.M):
    A[m.group(1)] = int(m.group(2), 16)
BOXCNT = A['BOXCNT']
CELL = 0xc000
GRID_W, GRID_H = 71, 43
NBOX = 98


def hx(v):
    return '$%x' % v


def zp(name, off=0):
    return hx(A[name] + off)


def cell(gx, gy):
    return CELL + gy * 80 + gx


def test(name, call, setup=None, expect=None, **kw):
    t = {"name": name}
    if setup:
        t["setup"] = setup
    t["call"] = call
    t["expect"] = expect
    t["timeoutMs"] = 20000
    t.update(kw)
    return t


tests = []

# ---- Vorbereitung: ROMs ausblenden (LDA #0 / STA $D030 / TAX / TAY / TAZ / MAP / EOM / RTS), liegt in der Falle ($1F40, freier Bereich)
prime = [0xA9, 0x00, 0x8D, 0x30, 0xD0, 0xAA, 0xA8, 0x4B, 0x5C, 0xEA, 0x60]
tests.append(test("Vorbereitung: ROMs ausblenden (Xemu-Start mit ROM-BASIC ueber $2000)", "$1f40",
                  {"mem": {"$1f40": prime}}, ["[$1f40] == $a9"]))

# ---- Zufall und Hilfsroutinen
tests.append(test("rnd: Startwert 1 -> $B8 (LSR, Uebertrag, EOR $B8)", "enemies.rnd", {"mem": {zp("RND"): 1}}, ["a == $b8", "[%s] == $b8" % zp("RND")]))
tests.append(test("rnd: gerader Startwert $B8 -> $5C (nur LSR)", "enemies.rnd", {"mem": {zp("RND"): 0xb8}}, ["a == $5c"]))
tests.append(test("level_idx: Stufe 1 -> Index 0", "enemies.level_idx", {"mem": {zp("LEVEL"): 1}}, ["y == 0"]))
tests.append(test("level_idx: Stufe 9 -> Index 8", "enemies.level_idx", {"mem": {zp("LEVEL"): 9}}, ["y == 8"]))
tests.append(test("level_idx: Stufe 12 (BCD $12) gilt wie 9", "enemies.level_idx", {"mem": {zp("LEVEL"): 0x12}}, ["y == 8"]))

# ---- Gitter: Zeile, Zelltyp, Nachbarn
for y in (0, 1, 2, 42):
    ad = CELL + 80 * y
    tests.append(test("grid.set_row: Zeile %d -> CELL + %d" % (y, 80 * y), "grid.set_row", {"y": y},
                      ["[%s] == %s" % (zp("PTR_B"), hx(ad & 255)), "[%s] == %s" % (zp("PTR_B", 1), hx(ad >> 8))]))
for (gx, gy, typ, nm) in [(0, 0, 3, "Ecke"), (5, 6, 3, "Knoten"), (1, 0, 1, "waagerechte Linie oben"), (3, 12, 1, "waagerechte Linie"), (0, 1, 2, "senkrechte Linie links"),
                          (70, 41, 2, "senkrechte Linie rechts"), (2, 3, 0, "Innenraum"), (68, 40, 0, "Innenraum unten rechts")]:
    tests.append(test("cell_type: (%d, %d) %s -> %d" % (gx, gy, nm, typ), "grid.cell_type", {"mem": {zp("T0"): gx, zp("T1"): gy}}, ["a == %d" % typ]))
for (gx, gy, typ, n0, n1, nm) in [
        (1, 6, 1, 14, 0, "waagerecht, Rechteck (0,1) unten und (0,0) oben: 14 und 0"),
        (1, 0, 1, 0, 255, "waagerecht am oberen Rand: nur unten (0)"),
        (1, 42, 1, 255, 84, "waagerecht am unteren Rand: nur oben (84 = 14 * 6)"),
        (5, 1, 2, 1, 0, "senkrecht, Rechteck (1,0) rechts und (0,0) links"),
        (0, 1, 2, 0, 255, "senkrecht am linken Rand: nur rechts"),
        (70, 1, 2, 255, 13, "senkrecht am rechten Rand: nur links (13)")]:
    tests.append(test("nb_boxes: (%d, %d) %s" % (gx, gy, nm), "grid.nb_boxes", {"mem": {zp("T0"): gx, zp("T1"): gy, zp("T2"): typ}},
                      ["[%s] == %d" % (zp("NB0"), n0), "[%s] == %d" % (zp("NB1"), n1)]))

# legal_mask: T4 = gx, T5 = gy, T6 = Richtung; Bits 1 hoch, 2 runter, 4 links, 8 rechts
for (gx, gy, d, mask, nm) in [(5, 6, 1, 14, "Knoten (5, 6), Richtung runter: hoch ausgeschlossen"),
                              (0, 0, 0, 8, "Ecke (0, 0), Richtung hoch: runter ausgeschlossen = nur rechts"),
                              (0, 0, 3, 10, "Ecke (0, 0), Richtung rechts: runter und rechts"),
                              (70, 42, 2, 5, "Ecke (70, 42), Richtung links: hoch, links"),
                              (70, 0, 0, 4, "Ecke (70, 0), Richtung hoch: nur links (runter ist die Umkehr)"),
                              (35, 0, 3, 10, "Randknoten (35, 0), Richtung rechts: links ausgeschlossen = runter, rechts")]:
    tests.append(test("legal_mask: " + nm, "grid.legal_mask", {"mem": {zp("T4"): gx, zp("T5"): gy, zp("T6"): d}}, ["a == %d" % mask]))

# ---- Rechtecke: box_filled, seg_is_closed
tests.append(test("box_filled: BOXCNT 4 -> gefuellt", "grid.box_filled", {"x": 5, "mem": {hx(BOXCNT + 5): 4}}, ["fc == 1"]))
tests.append(test("box_filled: BOXCNT 3 -> nicht gefuellt", "grid.box_filled", {"x": 5, "mem": {hx(BOXCNT + 5): 3}}, ["fc == 0"]))
tests.append(test("box_filled: $FF (kein Rechteck) -> nicht gefuellt", "grid.box_filled", {"x": 255}, ["fc == 0"]))
tests.append(test("seg_is_closed: ein angrenzendes Rechteck gefuellt -> geschuetzt", "grid.seg_is_closed",
                  {"mem": {zp("T0"): 1, zp("T1"): 6, zp("T2"): 1, hx(BOXCNT + 14): 4}}, ["fc == 1"]))
tests.append(test("seg_is_closed: kein angrenzendes Rechteck gefuellt -> nicht geschuetzt", "grid.seg_is_closed",
                  {"mem": {zp("T0"): 1, zp("T1"): 6, zp("T2"): 1, hx(BOXCNT + 14): 3, hx(BOXCNT): 3}}, ["fc == 0"]))

# ---- Punkte (BCD): Spieler 1 = SCORE, Spieler 2 = SCORE + 4
S = A['SCORE']
L = A['LIVES']
tests.append(test("bcd_add: Uebertrag ueber alle Stellen", "hud.bcd_add", {"a": 0x99, "x": 0, "mem": {zp("WHO"): 0, hx(S): [1, 0, 0], zp("LIVES"): 3}},
                  ["[%s] == 0" % hx(S), "[%s] == 1" % hx(S + 1), "[%s] == 3" % hx(L)]))
tests.append(test("bcd_add: Extraleben bei 10000 Punkten", "hud.bcd_add", {"a": 0x50, "x": 0, "mem": {zp("WHO"): 0, hx(S): [0x50, 0x99, 0x09], zp("LIVES"): 3}},
                  ["[%s] == 0" % hx(S), "[%s] == 0" % hx(S + 1), "[%s] == $10" % hx(S + 2), "[%s] == 4" % hx(L)]))
tests.append(test("bcd_add: Leben hoechstens 9", "hud.bcd_add", {"a": 0x50, "x": 0, "mem": {zp("WHO"): 0, hx(S): [0x50, 0x99, 0x09], zp("LIVES"): 9}}, ["[%s] == 9" % hx(L)]))
tests.append(test("bcd_add: 500 Punkte (X = 1)", "hud.bcd_add", {"a": 5, "x": 1, "mem": {zp("WHO"): 0, hx(S): [0x20, 0x01, 0], zp("LIVES"): 3}},
                  ["[%s] == $20" % hx(S), "[%s] == $06" % hx(S + 1)]))
tests.append(test("bcd_add: Spieler 2 bucht in SCORE + 4, Leben von Spieler 2", "hud.bcd_add",
                  {"a": 0x50, "x": 0, "mem": {zp("WHO"): 1, hx(S): [1, 2, 3], hx(S + 4): [0x50, 0x99, 0x09], hx(L): [3, 3]}},
                  ["[%s] == 1" % hx(S), "[%s] == 0" % hx(S + 4), "[%s] == $10" % hx(S + 6), "[%s] == 3" % hx(L), "[%s] == 4" % hx(L + 1)]))
tests.append(test("bcd_sub: 25 von 100 -> 75", "hud.bcd_sub", {"a": 0x25, "mem": {zp("WHO"): 0, hx(S): [0, 1, 0]}}, ["[%s] == $75" % hx(S), "[%s] == 0" % hx(S + 1)]))
tests.append(test("bcd_sub: nie unter 0", "hud.bcd_sub", {"a": 0x50, "mem": {zp("WHO"): 0, hx(S): [0x20, 0, 0]}}, ["[%s] == 0" % hx(S), "[%s] == 0" % hx(S + 1), "[%s] == 0" % hx(S + 2)]))

# ---- Gegner
EN = A['EN']
tests.append(test("greedy: Gegner (5, 6), Figur (35, 6): rechts", "enemies.greedy",
                  {"mem": {zp("T4"): 5, zp("T5"): 6, zp("T7"): 15, zp("PX"): 35, zp("PY"): 6, zp("PACT"): [1, 0]}}, ["x == 3"]))
tests.append(test("greedy: Gegner (10, 18), Figur (10, 0): hoch", "enemies.greedy",
                  {"mem": {zp("T4"): 10, zp("T5"): 18, zp("T7"): 15, zp("PX"): 10, zp("PY"): 0, zp("PACT"): [1, 0]}}, ["x == 0"]))
tests.append(test("greedy: Gegner (35, 12), Figur (0, 12): links", "enemies.greedy",
                  {"mem": {zp("T4"): 35, zp("T5"): 12, zp("T7"): 15, zp("PX"): 0, zp("PY"): 12, zp("PACT"): [1, 0]}}, ["x == 2"]))
tests.append(test("greedy: nur erlaubte Richtungen (Maske links/runter), Figur rechts unten: die bessere der erlaubten", "enemies.greedy",
                  {"mem": {zp("T4"): 30, zp("T5"): 12, zp("T7"): 6, zp("PX"): 70, zp("PY"): 42, zp("PACT"): [1, 0]}}, ["x == 1"]))
tests.append(test("greedy: zwei Figuren: die naeher liegende gilt (Figur 2 bei (40, 6))", "enemies.greedy",
                  {"mem": {zp("T4"): 35, zp("T5"): 6, zp("T7"): 15, zp("PX"): [0, 40], zp("PY"): [42, 6], zp("PACT"): [1, 1]}}, ["x == 3"]))
tests.append(test("greedy: Figur 2 nicht im Spiel (PACT = 0): bleibt unbeachtet", "enemies.greedy",
                  {"mem": {zp("T4"): 35, zp("T5"): 6, zp("T7"): 15, zp("PX"): [0, 40], zp("PY"): [6, 6], zp("PACT"): [1, 0]}}, ["x == 2"]))
for lvl, n in [(1, 2), (3, 3), (5, 4), (7, 5)]:
    tests.append(test("place_all: Stufe %d -> %d Gegner" % (lvl, n), "enemies.place_all", {"mem": {zp("LEVEL"): lvl}},
                      ["[%s] == %d" % (zp("NENEMY"), n), "[%s] == %d" % (zp("ELIM"), 6 * n), "[%s] == %d" % (hx(EN + 0), 70), "[%s] == 0" % hx(EN + 1),
                       "[%s] == 0" % hx(EN + 6), "[%s] == 0" % hx(EN + 7)]))
tests.append(test("find: Gegner auf (7, 8) gefunden, X = Abstand in EN", "enemies.find",
                  {"x": 7, "y": 8, "mem": {zp("ELIM"): 12, hx(EN): [1, 1, 0, 0, 1, 1, 7, 8, 0, 1, 7, 8]}}, ["fc == 1", "x == 6"]))
tests.append(test("find: keiner auf (3, 3)", "enemies.find", {"x": 3, "y": 3, "mem": {zp("ELIM"): 12, hx(EN): [1, 1, 0, 0, 1, 1, 7, 8, 0, 1, 7, 8]}}, ["fc == 0"]))

# ---- Figur: Schritt (Gegner weit weg bei (70, 0))
far = [70, 0, 0, 0, 70, 0]


def step_mem(px, py, want, pinv=0, enemy=None, cells=None):
    m = {zp("FIG"): 0, zp("WHO"): 0, zp("PX"): [px, 0], zp("PY"): [py, 0], zp("PWANT"): [want, 0], zp("PINV"): [pinv, 0], zp("PACT"): [1, 0], zp("OWN"): [0, 0],
         zp("NENEMY"): 1, zp("ELIM"): 6, hx(EN): enemy or far, zp("HITF"): 0, zp("FILLS_NOW"): 0, hx(S): [0, 0, 0], zp("LIVES"): [3, 3], hx(A['IT_KIND']): 0}
    for (cx, cy, v) in (cells or []):
        m[hx(cell(cx, cy))] = v
    return m


tests.append(test("player.step: von (0, 42) hoch nach (0, 41): faerbt, 1 Punkt, Zelle vor dem Schritt gemerkt", "player.step",
                  {"mem": step_mem(0, 42, 1, cells=[(0, 41, 1)])},
                  ["[%s] == 41" % zp("PY"), "[%s] == 0" % zp("PX"), "[%s] == 42" % zp("POY"), "[%s] == 1" % hx(S), "[%s] == 2" % hx(cell(0, 41)), "[%s] == 1" % zp("PANIM")]))
tests.append(test("player.step: waagerecht rechts von (1, 42) nach (2, 42)", "player.step",
                  {"mem": step_mem(1, 42, 8, cells=[(2, 42, 1)])}, ["[%s] == 2" % zp("PX"), "[%s] == 42" % zp("PY"), "[%s] == 1" % zp("POX")]))
tests.append(test("player.step: auf einer waagerechten Linie ist hoch gesperrt", "player.step",
                  {"mem": step_mem(1, 42, 1)}, ["[%s] == 42" % zp("PY"), "[%s] == 1" % zp("PX")]))
tests.append(test("player.step: ausserhalb des Gitters kein Schritt (Ecke 0, 0, hoch)", "player.step",
                  {"mem": step_mem(0, 0, 1)}, ["[%s] == 0" % zp("PY")]))
tests.append(test("player.step: Gegner auf der Zielzelle = Treffer (Bit 0), Figur bleibt", "player.step",
                  {"mem": step_mem(0, 42, 1, enemy=[0, 41, 0, 0, 0, 41])}, ["[%s] == 1" % zp("HITF"), "[%s] == 42" % zp("PY")]))
tests.append(test("player.step: Gegner auf der Zielzelle waehrend der Unverwundbarkeit: kein Treffer, Figur bleibt", "player.step",
                  {"mem": step_mem(0, 42, 1, pinv=5, enemy=[0, 41, 0, 0, 0, 41])}, ["[%s] == 0" % zp("HITF"), "[%s] == 42" % zp("PY"), "[%s] == 4" % zp("PINV")]))
tests.append(test("player.step: keine Richtung gehalten: stehen bleiben, Zelle vor dem Schritt = Zelle", "player.step",
                  {"mem": step_mem(0, 42, 0)}, ["[%s] == 42" % zp("PY"), "[%s] == 42" % zp("POY"), "[%s] == 0" % hx(S)]))

# ---- Knotenregel (Regel 16.2): Knoten faerben (paint_node), zuruecksetzen (node_free / nodes_free), Zerstoerung beim Betreten eines Knotens (touch)
# Zustaende: 0 Innenraum, 1 Linie unberuehrt, 2 von Spieler 1 abgefahren, 3 von Spieler 2.  Knoten liegen bei gx mod 5 = 0 und gy mod 6 = 0, Beispiel (20, 18).
far6 = [70, 0, 0, 0, 70, 0]
fig_away = {zp("PACT"): [1, 0], zp("PX"): [0, 0], zp("PY"): [42, 0], zp("ELIM"): 6, hx(EN): far6, zp("WHO"): 0, hx(S): [0x50, 0, 0], zp("LIVES"): [3, 3]}


def around(nx, ny):
    """Die neun Zellen um den Knoten (nx, ny) im Grundzustand: Mitte und Linienzellen unberuehrt (1), Ecken Innenraum (0)."""
    d = {}
    for ddx in (-1, 0, 1):
        for ddy in (-1, 0, 1):
            d[hx(cell(nx + ddx, ny + ddy))] = 0 if (ddx and ddy) else 1
    return d


def nf(name, node_state, over, expect, extra=None):
    m = dict(fig_away)
    m.update(around(20, 18))
    m[hx(cell(20, 18))] = node_state
    for (x, y), v in over.items():
        m[hx(cell(x, y))] = v
    m[zp("NODEX")] = 20
    m[zp("NODEY")] = 18
    if extra:
        m.update(extra)
    tests.append(test("node_free: " + name, "grid.node_free", {"mem": m}, ["[%s] == %d" % (hx(cell(20, 18)), expect)]))


nf("Knoten von Spieler 1 ohne abgefahrenes Nachbarsegment: wird zurueckgesetzt", 2, {}, 1)
nf("Knoten von Spieler 2 ohne abgefahrenes Nachbarsegment: wird zurueckgesetzt", 3, {}, 1)
nf("abgefahrenes Segment rechts (21, 18): Knoten bleibt", 2, {(21, 18): 2}, 2)
nf("abgefahrenes Segment oben (20, 17) von Spieler 2: Knoten bleibt", 2, {(20, 17): 3}, 2)
nf("abgefahrenes Segment links (19, 18): Knoten bleibt", 2, {(19, 18): 2}, 2)
nf("gefuelltes Rechteck diagonal (21, 19) (Kachel): Knoten bleibt", 3, {(21, 19): 20}, 3)
nf("Zelle (21, 18) blitzt nur auf (Zustand 4): zaehlt nicht als abgefahren", 2, {(21, 18): 4}, 1)
nf("die Figur steht auf dem Knoten: bleibt", 2, {}, 2, {zp("PX"): [20, 0], zp("PY"): [18, 0]})
nf("die Figur 2 steht auf dem Knoten: bleibt", 2, {}, 2, {zp("PACT"): [1, 1], zp("PX"): [0, 20], zp("PY"): [42, 18]})
nf("Figur 2 nicht im Spiel (PACT = 0), steht aber auf den Koordinaten: Knoten wird zurueckgesetzt", 2, {}, 1, {zp("PACT"): [1, 0], zp("PX"): [0, 20], zp("PY"): [42, 18]})
nf("ein Gegner steht auf dem Knoten: bleibt", 2, {}, 2, {hx(EN): [20, 18, 0, 0, 20, 18]})
tests.append(test("node_free: unberuehrter Knoten bleibt unberuehrt", "grid.node_free",
                  {"mem": dict(dict(fig_away, **around(20, 18)), **{zp("NODEX"): 20, zp("NODEY"): 18})}, ["[%s] == 1" % hx(cell(20, 18))]))
tests.append(test("node_free: Knoten in der Ecke (0, 0) ohne Nachbarn im Gitter: wird zurueckgesetzt, kein Zugriff ausserhalb", "grid.node_free",
                  {"mem": dict(fig_away, **{zp("NODEX"): 0, zp("NODEY"): 0, hx(cell(0, 0)): 2, hx(cell(1, 0)): 1, hx(cell(0, 1)): 1, hx(cell(1, 1)): 0})},
                  ["[%s] == 1" % hx(cell(0, 0))]))
tests.append(test("node_free: Knoten in der Ecke (70, 42): abgefahrenes Segment links (69, 42): bleibt", "grid.node_free",
                  {"mem": dict(fig_away, **{zp("NODEX"): 70, zp("NODEY"): 42, hx(cell(70, 42)): 2, hx(cell(69, 42)): 2, hx(cell(70, 41)): 1, hx(cell(69, 41)): 0})},
                  ["[%s] == 2" % hx(cell(70, 42))]))
# paint_node: T0 = gx, T1 = gy, WHO = Spieler
tests.append(test("paint_node: Knoten (20, 18), Spieler 1: Zustand 2", "grid.paint_node",
                  {"mem": {zp("T0"): 20, zp("T1"): 18, zp("WHO"): 0, hx(cell(20, 18)): 1}}, ["[%s] == 2" % hx(cell(20, 18))]))
tests.append(test("paint_node: Knoten (20, 18), Spieler 2: Zustand 3", "grid.paint_node",
                  {"mem": {zp("T0"): 20, zp("T1"): 18, zp("WHO"): 1, hx(cell(20, 18)): 1}}, ["[%s] == 3" % hx(cell(20, 18))]))
tests.append(test("paint_node: Linienzelle (22, 18) bleibt unberuehrt", "grid.paint_node",
                  {"mem": {zp("T0"): 22, zp("T1"): 18, zp("WHO"): 0, hx(cell(22, 18)): 1}}, ["[%s] == 1" % hx(cell(22, 18))]))
tests.append(test("paint_node: Innenraum (22, 20) bleibt leer", "grid.paint_node",
                  {"mem": {zp("T0"): 22, zp("T1"): 20, zp("WHO"): 0, hx(cell(22, 20)): 0}}, ["[%s] == 0" % hx(cell(22, 20))]))
tests.append(test("player.step: der Schritt auf einen Knoten faerbt ihn (von (0, 37) hoch auf Knoten (0, 36)), ohne Punkte", "player.step",
                  {"mem": dict(step_mem(0, 37, 1, cells=[(0, 36, 1), (0, 37, 1)]))},
                  ["[%s] == 36" % zp("PY"), "[%s] == 2" % hx(cell(0, 36)), "[%s] == 0" % hx(S)]))

# seg_destroy: die Endknoten werden mit zurueckgesetzt, wenn kein weiteres abgefahrenes Segment anschliesst
def dn_mem(extra=None):
    m = dict(fig_away)
    m.update(around(20, 18))
    m.update(around(25, 18))
    for x in (21, 22, 23, 24):
        m[hx(cell(x, 18))] = 2
    m[hx(cell(20, 18))] = 2
    m[hx(cell(25, 18))] = 2
    m.update({zp("T0"): 22, zp("T1"): 18, zp("T2"): 1, zp("FIG"): 0, hx(BOXCNT + 32): 3, hx(BOXCNT + 46): 3, zp("FREEZE"): [0, 0]})
    if extra:
        m.update(extra)
    return m


tests.append(test("seg_destroy: beide Endknoten ohne weiteres abgefahrenes Segment werden mit zurueckgesetzt", "grid.seg_destroy", {"mem": dn_mem()},
                  ["[%s] == 1" % hx(cell(20, 18)), "[%s] == 1" % hx(cell(25, 18)), "[%s] == 1" % hx(cell(22, 18))], timeoutMs=60000))
tests.append(test("seg_destroy: Endknoten (20, 18) mit abgefahrenem Segment darueber (20, 17) bleibt, (25, 18) wird zurueckgesetzt", "grid.seg_destroy",
                  {"mem": dn_mem({hx(cell(20, 17)): 2})}, ["[%s] == 2" % hx(cell(20, 18)), "[%s] == 1" % hx(cell(25, 18))], timeoutMs=60000))
tests.append(test("seg_destroy: das Segment grenzt an ein gefuelltes Rechteck: nichts geschieht, Knoten bleiben", "grid.seg_destroy",
                  {"mem": dn_mem({hx(BOXCNT + 46): 4})}, ["[%s] == 2" % hx(cell(20, 18)), "[%s] == 2" % hx(cell(22, 18))], timeoutMs=60000))
# touch: ein Gegner betritt den Knoten (20, 18): alle dort endenden Segmente werden geprueft
def touch_mem(tx, ty, extra=None):
    m = dict(fig_away)
    m.update(around(20, 18))
    m.update({zp("T4"): tx, zp("T5"): ty, zp("FIG"): 0, zp("ECUR"): 0, zp("FREEZE"): [0, 0]})
    for b in (31, 32, 45, 46):
        m[hx(BOXCNT + b)] = 3                   # abgeschlossene Segmente zaehlen ihre Rechtecke herunter (3 = noch nicht gefuellt)
    for x in (21, 22, 23, 24):
        m[hx(cell(x, 18))] = 2
    for y in (13, 14, 15, 16, 17):
        m[hx(cell(20, y))] = 3
    for y in (19, 20, 21, 22, 23):
        m[hx(cell(20, y))] = 2
    for x in (15, 16, 17, 18, 19):
        m[hx(cell(x, 18))] = 3
    m[hx(cell(20, 18))] = 2
    if extra:
        m.update(extra)
    return m


tests.append(test("touch: Gegner betritt den Knoten (20, 18): alle vier Segmente werden zerstoert, Knoten wird zurueckgesetzt", "enemies.touch", {"mem": touch_mem(20, 18)},
                  ["[%s] == 1" % hx(cell(22, 18)), "[%s] == 1" % hx(cell(20, 15)), "[%s] == 1" % hx(cell(20, 21)), "[%s] == 1" % hx(cell(17, 18)),
                   "[%s] == 1" % hx(cell(20, 18)), "[%s] == 20" % zp("T4"), "[%s] == 18" % zp("T5")], timeoutMs=120000))
tests.append(test("touch: Gegner betritt ein Innenzeichen (22, 18): nur dieses Segment wird zerstoert", "enemies.touch", {"mem": touch_mem(22, 18)},
                  ["[%s] == 1" % hx(cell(22, 18)), "[%s] == 3" % hx(cell(20, 15)), "[%s] == 2" % hx(cell(20, 21)), "[%s] == 3" % hx(cell(17, 18))], timeoutMs=60000))
tests.append(test("touch: Gegner betritt den Knoten (20, 18), rechts grenzt ein gefuelltes Rechteck: dieses Segment bleibt", "enemies.touch",
                  {"mem": touch_mem(20, 18, {hx(BOXCNT + 46): 4})}, ["[%s] == 2" % hx(cell(22, 18)), "[%s] == 1" % hx(cell(20, 15))], timeoutMs=120000))

# ---- Bestenliste (Standardliste 1000, 900, ... 100 Punkte; Punkte BCD, niedrige Ziffern zuerst, siehe hiscore.asm)
tests.append(test("hiscore.default: Kennung \"TRXX\", erster Platz 1000, letzter 100, HISCORE = 1000", "hiscore.default", None,
                  ["[hiscore.hs_buf] == $54", "[hiscore.hs_buf + 1] == $52", "[hiscore.hs_buf + 5 + 8] == 0", "[hiscore.hs_buf + 5 + 9] == $10", "[hiscore.hs_buf + 5 + 10] == 0",
                   "[hiscore.hs_buf + 5 + 9 * 12 + 9] == 1", "[%s] == $10" % zp("HISCORE", 1)]))
# Standardliste: Kennung, Version, zehn Eintraege (Name 8, Punkte 3, Stufe 1), Pruefsumme (ohne Bedeutung fuer hs_check); jeder hs_check-Test schreibt sie selbst
default_list = [0x54, 0x52, 0x58, 0x58, 1]
for i in range(10):
    default_list += [0x20] * 8 + [0, int(str(10 - i), 16), 0, 1]
default_list += [0, 0]
for (nm, pts, place) in [("Endstand ueber dem ersten Platz (1001) -> Platz 1", [1, 0x10, 0], 0), ("Endstand zwischen Platz 3 und 4 (750) -> Platz 4", [0x50, 7, 0], 3),
                         ("Endstand genau gleich dem letzten (100): nicht in der Liste", [0, 1, 0], 255), ("Endstand unter dem letzten (50)", [0x50, 0, 0], 255),
                         ("Endstand gleich Platz 1 (1000): dahinter -> Platz 2", [0, 0x10, 0], 1), ("Endstand knapp ueber dem letzten (101) -> Platz 10", [1, 1, 0], 9),
                         ("Endstand mit Zehntausendern (10000) -> Platz 1", [0, 0, 1], 0)]:
    tests.append(test("hs_check: " + nm, "hiscore.hs_check", {"mem": {"hiscore.hs_points": pts, "hiscore.hs_buf": default_list}}, ["a == %d" % place]))

d = {"kind": "test", "target": "mega65", "program": "build/main.prg", "trap": "$1f00",
     "static": [
         {"check": "size", "segment": "PROGRAM", "max": 40000, "maxEnd": "$b800", "name": "Programm endet vor den Sprite-Daten ($B800)"},
         {"check": "overlap"},
         {"check": "expr", "name": "Schattengitter passt vor die Spielvariablen: CELL + 43 * 80 <= BOXCNT", "expr": "$c000 + 43 * 80 <= $ce00"},
         {"check": "expr", "name": "Gesichertes Gitter passt in $1000 Byte", "expr": "$cd6f - $c000 + 98 + 3 <= $1000"},
     ],
     "tests": tests}
open('troxx.vctest', 'w', encoding='utf-8', newline='\n').write(json.dumps(d, indent=1, ensure_ascii=False) + "\n")
print(len(tests), "Tests")
