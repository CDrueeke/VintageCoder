// SPDX-License-Identifier: GPL-3.0-or-later
// Layoutregeln der Arbeitsfläche (reine Funktionen; Entsprechung zu `vc-ide-model::layout`, Konzept Abschnitt 5.1):
// jeder Bereich lässt sich zu einem Streifen minimieren (außer dem Dokumentenfeld); einer füllt die Fläche (maximiert), die übrigen sind dann eingeklappt.
export type Area = "seitenleiste" | "dokument" | "schublade" | "panel";
export type Show = "normal" | "min" | "max";
export const AREAS: readonly Area[] = ["seitenleiste", "dokument", "schublade", "panel"];

/** Was ein Bereich zeigt: bei einem Maximierten sind alle anderen eingeklappt; das Dokumentenfeld bleibt dann als Reiterleiste stehen */
export function anzeige(a: Area, open: Record<Area, boolean>, max: Area | null): Show {
  if (max) return max === a ? "max" : "min";
  return a === "dokument" || open[a] ? "normal" : "min";
}
/** Maximiert ↔ wiederhergestellt */
export const umschaltenMax = (max: Area | null, a: Area): Area | null => (max === a ? null : a);
/** Bereich unter dem Tastaturfokus (Fokus-Maximieren); ohne Fokus in einem Bereich gilt das Dokumentenfeld */
export function bereichVon(el: Element | null): Area {
  const a = el?.closest?.("[data-area]")?.getAttribute("data-area");
  return a === "seitenleiste" || a === "schublade" || a === "panel" || a === "dokument" ? a : "dokument";
}

/** Größen in Pixeln: Grenzen, damit nichts unbedienbar wird */
export const SIZES = { dockH: { min: 90, max: 700, def: 210 }, panelW: { min: 220, max: 700, def: 300 }, sideW: { min: 180, max: 520, def: 252 } } as const;
export type SizeKey = keyof typeof SIZES;
export const klemme = (k: SizeKey, v: number): number => Math.max(SIZES[k].min, Math.min(SIZES[k].max, Math.round(v)));

/** Gemerkter Teil des Layouts (ohne „maximiert“ und Zen: beides wird nie gemerkt) */
export interface Gemerkt { sidebar: boolean; dock: null | "problems" | "output" | "tests" | "search" | "outline"; panel: boolean; dockH: number; panelW: number; sideW: number }
export const VORGABE: Gemerkt = { sidebar: true, dock: null, panel: false, dockH: SIZES.dockH.def, panelW: SIZES.panelW.def, sideW: SIZES.sideW.def };
const DOCKS = ["problems", "output", "tests", "search", "outline"];
/** Gelesenen Text prüfen: Unbrauchbares fällt auf die Vorgabe zurück */
export function leseGemerkt(text: string | null): Gemerkt {
  const g: Gemerkt = { ...VORGABE };
  try {
    const o = JSON.parse(text ?? "null") as Partial<Gemerkt> | null; if (!o || typeof o !== "object") return g;
    if (typeof o.sidebar === "boolean") g.sidebar = o.sidebar;
    if (typeof o.panel === "boolean") g.panel = o.panel;
    if (o.dock === null || (typeof o.dock === "string" && DOCKS.includes(o.dock))) g.dock = o.dock as Gemerkt["dock"];
    for (const k of ["dockH", "panelW", "sideW"] as const) if (typeof o[k] === "number" && Number.isFinite(o[k])) g[k] = klemme(k, o[k]!);
  } catch { /* beschädigt: Vorgabe */ }
  return g;
}

/** Breitenklassen nach der Breite der Dokumentenfläche (Muster der Ansichten): breit ≥ 900, mittel 640–899, schmal < 640 Punkte */
export type Klasse = "breit" | "mittel" | "schmal";
export const klasse = (docBreite: number): Klasse => (docBreite >= 900 ? "breit" : docBreite >= 640 ? "mittel" : "schmal");
/** Breite der Dokumentenfläche, wenn Seitenleiste und Panel sie verdrängen würden (Rand 20, Abstände 10, Streifen 40) */
export function docBreiteVerdraengt(mainB: number, side: boolean, sideB: number, panel: boolean, panelB: number): number { return mainB - 20 - (side ? sideB + 10 : 50) - (panel ? panelB + 10 : 0); }
/** Zwei Hälften nebeneinander gibt es erst ab dieser Breite der Dokumentenfläche, darunter stehen sie untereinander */
export const STAPEL_UNTER = 1000;
