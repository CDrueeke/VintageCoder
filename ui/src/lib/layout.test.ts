// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import { anzeige, bereichVon, docBreiteVerdraengt, klasse, klemme, leseGemerkt, umschaltenMax, VORGABE, type Area } from "./layout";

const offen = (o: Partial<Record<Area, boolean>> = {}): Record<Area, boolean> => ({ seitenleiste: true, dokument: true, schublade: true, panel: false, ...o });

describe("Layoutregeln", () => {
  it("ohne Maximierung: offen = normal, zu = minimiert, das Dokumentenfeld bleibt normal", () => {
    expect([anzeige("seitenleiste", offen(), null), anzeige("schublade", offen({ schublade: false }), null), anzeige("panel", offen(), null), anzeige("dokument", offen(), null)]).toEqual(["normal", "min", "min", "normal"]);
  });
  it("ein maximierter Bereich füllt die Fläche, alle anderen sind eingeklappt", () => {
    expect(anzeige("schublade", offen(), "schublade")).toBe("max");
    expect([anzeige("seitenleiste", offen(), "schublade"), anzeige("dokument", offen(), "schublade"), anzeige("panel", offen({ panel: true }), "schublade")]).toEqual(["min", "min", "min"]);
  });
  it("Maximieren schaltet um, Wiederherstellen kehrt zurück", () => {
    expect(umschaltenMax(null, "panel")).toBe("panel"); expect(umschaltenMax("panel", "panel")).toBeNull(); expect(umschaltenMax("panel", "dokument")).toBe("dokument");
  });
  it("Fokus-Maximieren: Bereich unter dem Fokus, sonst das Dokumentenfeld", () => {
    const el = (a: string | null) => ({ closest: (s: string) => (s === "[data-area]" && a ? { getAttribute: () => a } : null) }) as unknown as Element;
    expect(bereichVon(el("schublade"))).toBe("schublade"); expect(bereichVon(el(null))).toBe("dokument"); expect(bereichVon(el("quatsch"))).toBe("dokument"); expect(bereichVon(null)).toBe("dokument");
  });
  it("Größen werden begrenzt", () => { expect(klemme("dockH", 5)).toBe(90); expect(klemme("dockH", 5000)).toBe(700); expect(klemme("panelW", 300.4)).toBe(300); });
  it("Gemerktes Layout: gültige Werte bleiben, Unbrauchbares fällt auf die Vorgabe", () => {
    expect(leseGemerkt(null)).toEqual(VORGABE); expect(leseGemerkt("kaputt")).toEqual(VORGABE);
    expect(leseGemerkt(JSON.stringify({ sidebar: false, dock: "output", panel: true, dockH: 300, panelW: 10, sideW: "x" }))).toEqual({ ...VORGABE, sidebar: false, dock: "output", panel: true, dockH: 300, panelW: 220 });
    expect(leseGemerkt(JSON.stringify({ dock: "unbekannt" })).dock).toBeNull();
  });
  it("Breitenklassen: breit ab 900, mittel ab 640, darunter schmal", () => { expect([klasse(1200), klasse(900), klasse(899), klasse(640), klasse(639), klasse(0)]).toEqual(["breit", "breit", "mittel", "mittel", "schmal", "schmal"]); });
  it("Breite der Dokumentenfläche bei verdrängenden Leisten", () => {
    expect(docBreiteVerdraengt(1440, true, 252, false, 300)).toBe(1440 - 20 - 262);
    expect(docBreiteVerdraengt(1440, false, 252, true, 300)).toBe(1440 - 20 - 50 - 310);
    expect(klasse(docBreiteVerdraengt(900, true, 252, false, 300))).toBe("schmal");
  });
});
