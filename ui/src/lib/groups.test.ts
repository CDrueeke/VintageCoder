// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import { activateIn, focusedRel, inGroup, isSplit, mergeAll, moveTo, removeTab, toggleSplit, toOtherHalf, type GState } from "./groups";

const mk = (rels: string[], active = rels[0] ?? null): GState => ({ tabs: rels.map((rel) => ({ rel, group: 0 as const })), active: [active, null], focus: 0 });
const names = (s: GState, g: 0 | 1) => inGroup(s, g).map((t) => t.rel);

describe("Reitergruppen", () => {
  it("Teilen schiebt den aktiven Reiter nach rechts und fokussiert die rechte Hälfte", () => {
    const s = mk(["a", "b", "c"], "b"); expect(toggleSplit(s)).toBe(true);
    expect([names(s, 0), names(s, 1), s.active, s.focus]).toEqual([["a", "c"], ["b"], ["c", "b"], 1]);
  });
  it("Der einzige Reiter lässt sich nicht abspalten", () => { const s = mk(["a"]); expect(toggleSplit(s)).toBe(false); expect(isSplit(s)).toBe(false); });
  it("Erneut teilen vereint beide Hälften, aktiv bleibt der Reiter der fokussierten", () => {
    const s = mk(["a", "b", "c"], "b"); toggleSplit(s); activateIn(s, "a"); toggleSplit(s);
    expect([isSplit(s), names(s, 0), focusedRel(s)]).toEqual([false, ["a", "c", "b"], "a"]);
  });
  it("Ein Reiter in die andere Hälfte: Quelle wählt den Nachbarn, leere Hälfte verschwindet", () => {
    const s = mk(["a", "b", "c"], "a"); moveTo(s, "c", 1); moveTo(s, "b", 1);
    expect([names(s, 0), names(s, 1)]).toEqual([["a"], ["c", "b"]]);
    moveTo(s, "a", 1); expect([isSplit(s), names(s, 0), s.focus]).toEqual([false, ["c", "b", "a"], 0]);
  });
  it("Schließen des letzten Reiters einer Hälfte hebt die Teilung auf", () => {
    const s = mk(["a", "b"], "a"); toggleSplit(s); removeTab(s, "b");
    expect([isSplit(s), names(s, 0), s.focus, focusedRel(s)]).toEqual([false, ["a"], 0, "a"]);
  });
  it("Schließt man links den letzten Reiter, rückt rechts nach links", () => {
    const s = mk(["a", "b"], "a"); moveTo(s, "b", 1); removeTab(s, "a");
    expect([isSplit(s), names(s, 0), focusedRel(s)]).toEqual([false, ["b"], "b"]);
  });
  it("Schließen des aktiven Reiters wählt den Nachbarn in derselben Hälfte", () => {
    const s = mk(["a", "b", "c", "d"], "a"); moveTo(s, "b", 1); moveTo(s, "c", 1); moveTo(s, "d", 1);
    activateIn(s, "c"); removeTab(s, "c"); expect([s.active[1], names(s, 1)]).toEqual(["d", ["b", "d"]]);
  });
  it("In die andere Hälfte schieben und Einfügen vor einem Reiter", () => {
    const s = mk(["a", "b", "c"], "a"); expect(toOtherHalf(s)).toBe(true); expect(names(s, 1)).toEqual(["a"]);
    moveTo(s, "c", 1, "a"); expect(names(s, 1)).toEqual(["c", "a"]);
    toOtherHalf(s); expect(focusedRel(s)).toBe("c"); expect(s.focus).toBe(0);
  });
  it("Aktivieren eines Reiters fokussiert seine Hälfte", () => { const s = mk(["a", "b"], "a"); moveTo(s, "b", 1); activateIn(s, "a"); expect([s.focus, focusedRel(s)]).toEqual([0, "a"]); expect(mergeAll(s)).toBeUndefined(); });
});
