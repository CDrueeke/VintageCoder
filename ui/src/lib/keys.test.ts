// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import { codeLabel, comboFromEvent, comboParts, learnLayout, matchesSpec, parseSpec } from "./keys";

const ev = (o: Partial<{ code: string; metaKey: boolean; ctrlKey: boolean; shiftKey: boolean; altKey: boolean }>) => ({ code: "KeyS", metaKey: false, ctrlKey: false, shiftKey: false, altKey: false, ...o });

describe("Kürzel", () => {
  it("zerlegt die Tabellenschreibweise", () => {
    expect(parseSpec("Cmd+Umschalt+S")).toEqual({ cmd: true, shift: true, alt: false, key: "S" });
    expect(parseSpec("")).toBeNull();
  });
  it("vergleicht nach physischer Taste, auf dem Mac mit Cmd, sonst Strg", () => {
    expect(matchesSpec(ev({ metaKey: true }), "Cmd+S", true)).toBe(true);
    expect(matchesSpec(ev({ ctrlKey: true }), "Cmd+S", true)).toBe(false);
    expect(matchesSpec(ev({ ctrlKey: true }), "Cmd+S", false)).toBe(true);
    expect(matchesSpec(ev({ ctrlKey: true, shiftKey: true }), "Cmd+S", false)).toBe(false);
    expect(matchesSpec(ev({ code: "F7" }), "F7", false)).toBe(true);
    expect(matchesSpec(ev({ code: "Comma", metaKey: true }), "Cmd+Komma", true)).toBe(true);
  });
  it("Modifikatoren allein sind kein Kürzel", () => {
    expect(comboFromEvent(ev({ code: "ShiftLeft", shiftKey: true }), true)).toBeNull();
  });
  it("Beschriftung folgt dem gelernten Layout (QWERTZ: Y und Z getauscht)", () => {
    expect(codeLabel("KeyY")).toBe("Y");
    learnLayout({ code: "KeyY", key: "z" });
    expect(codeLabel("KeyY")).toBe("Z");
    learnLayout({ code: "KeyY", key: "y" });
  });
  it("Anzeige je System", () => {
    const c = { code: "KeyS", cmd: true, shift: true, alt: false };
    const n = { ctrl: "Strg", shift: "Umschalt", alt: "Alt" };
    expect(comboParts(c, n, true)).toEqual(["⇧", "⌘", "S"]);
    expect(comboParts(c, n, false)).toEqual(["Strg", "Umschalt", "S"]);
  });
});
