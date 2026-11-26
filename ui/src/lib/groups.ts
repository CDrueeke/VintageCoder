// SPDX-License-Identifier: GPL-3.0-or-later
// Reitergruppen der geteilten Ansicht: höchstens zwei Gruppen (links 0, rechts 1), jeder Reiter gehört zu genau einer, jede Gruppe hat einen aktiven Reiter.
// Reine Datenlogik (Entsprechung zu `vc-gui/src/gruppen.rs`); die Funktionen verändern das übergebene Modell an Ort und Stelle.
export type Group = 0 | 1;
export interface GTab { rel: string; group: Group }
export interface GState { tabs: GTab[]; active: [string | null, string | null]; focus: Group }

export const inGroup = (s: GState, g: Group): GTab[] => s.tabs.filter((t) => t.group === g);
export const isSplit = (s: GState): boolean => s.tabs.some((t) => t.group === 1);

/** Leere Gruppen verschwinden (ist links leer, rückt rechts nach links); aktive Reiter und Fokus bleiben gültig */
export function normalize(s: GState): void {
  const has0 = s.tabs.some((t) => t.group === 0), has1 = s.tabs.some((t) => t.group === 1);
  if (!has0 && has1) { for (const t of s.tabs) t.group = 0; s.active = [s.active[1], null]; s.focus = 0; }
  else if (!has1) { s.active = [s.active[0], null]; s.focus = 0; }
  for (const g of [0, 1] as const) { const l = inGroup(s, g); if (!l.some((t) => t.rel === s.active[g])) s.active[g] = l[0]?.rel ?? null; }
}

/** Nachbar, der nach dem Entfernen des Reiters an Index `i` der Gruppe aktiv wird */
const neighbour = (rest: GTab[], i: number): string | null => rest[Math.min(i, rest.length - 1)]?.rel ?? null;

/** Reiter in eine Gruppe setzen, vor den Reiter `before` (sonst ans Ende). Die rechte Gruppe entsteht bei Bedarf; der letzte Reiter lässt sich nicht abspalten (`false`). */
export function moveTo(s: GState, rel: string, group: Group, before: string | null = null): boolean {
  const tab = s.tabs.find((t) => t.rel === rel); if (!tab) return false;
  if (group === 1 && !isSplit(s) && inGroup(s, 0).length < 2) return false;
  const from = tab.group; const idx = inGroup(s, from).findIndex((t) => t.rel === rel);
  s.tabs.splice(s.tabs.indexOf(tab), 1); tab.group = group;
  const j = before ? s.tabs.findIndex((t) => t.rel === before) : -1;
  s.tabs.splice(j < 0 ? s.tabs.length : j, 0, tab);
  if (from !== group && s.active[from] === rel) s.active[from] = neighbour(inGroup(s, from), idx);
  s.active[group] = rel; s.focus = group; normalize(s);
  return true;
}
/** Beide Gruppen vereinen; aktiv bleibt der Reiter der fokussierten Gruppe */
export function mergeAll(s: GState): void { const a = s.active[s.focus]; for (const t of s.tabs) t.group = 0; s.active = [a, null]; s.focus = 0; normalize(s); }
/** Teilen (aktiver Reiter nach rechts) oder, wenn geteilt, vereinen. `false`: Teilen nicht möglich (nur ein Reiter) */
export function toggleSplit(s: GState): boolean { if (isSplit(s)) { mergeAll(s); return true; } const a = s.active[s.focus]; return a ? moveTo(s, a, 1) : false; }
/** Aktiven Reiter in die andere Hälfte schieben (teilt, wenn noch nicht geteilt) */
export function toOtherHalf(s: GState): boolean { const a = s.active[s.focus]; return a ? moveTo(s, a, s.focus === 0 ? 1 : 0) : false; }
/** Reiter entfernen; die Gruppe wählt den Nachbarn, leere Gruppen verschwinden */
export function removeTab(s: GState, rel: string): void {
  const tab = s.tabs.find((t) => t.rel === rel); if (!tab) return;
  const g = tab.group; const idx = inGroup(s, g).findIndex((t) => t.rel === rel);
  s.tabs.splice(s.tabs.indexOf(tab), 1);
  if (s.active[g] === rel) s.active[g] = neighbour(inGroup(s, g), idx);
  normalize(s);
}
/** Reiter aktivieren: die Gruppe des Reiters bekommt den Fokus */
export function activateIn(s: GState, rel: string): boolean { const t = s.tabs.find((x) => x.rel === rel); if (!t) return false; s.active[t.group] = rel; s.focus = t.group; return true; }
export const focusedRel = (s: GState): string | null => s.active[s.focus];
