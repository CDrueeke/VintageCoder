// SPDX-License-Identifier: GPL-3.0-or-later
// Layout der Arbeitsfläche: Zustand (Maximierung, Größen), Merken in der Oberfläche, Zurücksetzen, Fokus-Maximieren.
import { ed } from "./editor.svelte";
import { anzeige, bereichVon, docBreiteVerdraengt, klasse, klemme, leseGemerkt, STAPEL_UNTER, umschaltenMax, VORGABE, type Area, type Gemerkt, type Klasse, type Show, type SizeKey } from "./layout";
import { ui } from "./state.svelte";

export const lay = $state({ max: null as Area | null, dockH: VORGABE.dockH, panelW: VORGABE.panelW, sideW: VORGABE.sideW, mainW: 1440, docsW: 1440 });
const KEY = "layout";

const openFlags = (): Record<Area, boolean> => ({ seitenleiste: ui.sidebar, dokument: true, schublade: ed.dock !== null, panel: ui.panel });
export const show = (a: Area): Show => anzeige(a, openFlags(), lay.max);

export function toggleMax(a: Area): void { lay.max = umschaltenMax(lay.max, a); }
/** Fokus-Maximieren (⌘⇧Enter): der Bereich unter dem Tastaturfokus */
export function focusMax(): void { toggleMax(bereichVon(typeof document === "undefined" ? null : document.activeElement)); }
export function setSize(k: SizeKey, v: number): void { lay[k] = klemme(k, v); }

function snapshot(): Gemerkt { return { sidebar: ui.sidebar, dock: ed.dock, panel: ui.panel, dockH: lay.dockH, panelW: lay.panelW, sideW: lay.sideW }; }
function apply(g: Gemerkt): void { ui.sidebar = g.sidebar; ed.dock = g.dock; ui.panel = g.panel; lay.dockH = g.dockH; lay.panelW = g.panelW; lay.sideW = g.sideW; }
function write(): void { try { localStorage.setItem(KEY, JSON.stringify(snapshot())); } catch { /* ohne Speicher weiter */ } }

/** Layout zurücksetzen: alles auf die Vorgabe, Maximierung und Zen enden */
export function resetLayout(): void { lay.max = null; ui.zen = false; apply(VORGABE); try { localStorage.removeItem(KEY); } catch { /* weiter */ } }

let started = false;
export function startLayout(): void {
  if (started) return; started = true;
  let g = VORGABE; try { g = leseGemerkt(localStorage.getItem(KEY)); } catch { /* Vorgabe */ }
  apply(g);
  let timer: ReturnType<typeof setTimeout> | null = null;
  $effect.root(() => {
    $effect(() => { void JSON.stringify(snapshot()); if (timer) clearTimeout(timer); timer = setTimeout(write, 250); });
    // Wird ein anderer Bereich geöffnet, endet die Maximierung; schließt der maximierte Bereich, ebenfalls
    let vor = openFlags();
    $effect(() => {
      const jetzt = openFlags();
      if (lay.max) for (const a of ["seitenleiste", "schublade", "panel"] as const) { if (jetzt[a] && !vor[a] && lay.max !== a) lay.max = null; else if (!jetzt[a] && vor[a] && lay.max === a) lay.max = null; }
      vor = jetzt;
    });
    // Zen beendet eine Maximierung
    $effect(() => { if (ui.zen && lay.max) lay.max = null; });
  });
}

/** Breitenklasse der Dokumentenfläche, gerechnet so, als verdrängten Seitenleiste und Panel sie (so entsteht kein Hin und Her beim Umschalten) */
export const cls = (): Klasse => klasse(docBreiteVerdraengt(lay.mainW, ui.sidebar, lay.sideW, ui.panel, lay.panelW));
/** Schmal: Seitenleiste und Panel liegen über der Dokumentenfläche statt sie zu verdrängen */
export const overlay = (): boolean => cls() === "schmal" && lay.max === null;
/** Geteilte Ansicht: unter dieser Breite stehen die Hälften untereinander */
export const stacked = (): boolean => lay.docsW < STAPEL_UNTER;
/** Aktion: meldet die Breite eines Elements (Größenänderung) */
export function watchWidth(node: HTMLElement, set: (w: number) => void): { destroy(): void } {
  const ro = new ResizeObserver(() => set(Math.round(node.clientWidth))); ro.observe(node); set(Math.round(node.clientWidth));
  return { destroy: () => ro.disconnect() };
}
