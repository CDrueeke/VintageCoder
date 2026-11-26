// SPDX-License-Identifier: GPL-3.0-or-later
// Zahlenwerkzeuge (Bearbeiten ▸ Zahlenwerkzeuge …): Zustand des Dialogs; gerechnet wird im Kern (`zahlen_werte`), die Oberfläche zeigt und setzt ein.
import { call, type ZahlenWerte } from "./api";
import { getView } from "./editor.svelte";
import { fail, toast } from "./state.svelte";
import { t } from "./i18n.svelte";

export const zw = $state({ offen: false, text: "$D020", bits: 8 as 8 | 16, werte: null as ZahlenWerte | null });

async function neuRechnen(toggle?: number): Promise<void> {
  try { const r = await call("zahlen_werte", { text: zw.text, bits: zw.bits, ...(toggle === undefined ? {} : { toggle }) }); zw.werte = r; if (toggle !== undefined && r.ok) zw.text = r.text; } catch (e) { fail(e); }
}
export function zahlenOeffnen(): void { zw.offen = true; void neuRechnen(); }
export function zahlenText(text: string): void { zw.text = text; void neuRechnen(); }
export function zahlenBreite(bits: 8 | 16): void { zw.bits = bits; void neuRechnen(); }
export function zahlenBit(n: number): void { void neuRechnen(n); }

/** Schreibweise `art` (dez, hex, bin) des Wertes */
export function zahlenForm(art: "dec" | "hex" | "bin"): string | null { const w = zw.werte; return w?.ok ? (w[art] ?? null) : null; }
/** In die Zwischenablage kopieren */
export async function zahlenKopieren(art: "dec" | "hex" | "bin"): Promise<void> {
  const s = zahlenForm(art); if (!s) return;
  try { await navigator.clipboard.writeText(s); toast(t("zahlen-kopiert", { text: s })); } catch { toast(t("zwischenablage-nicht-verfuegbar"), true); }
}
/** An der Schreibmarke des Editors einsetzen (ein Rückgängig-Schritt) */
export function zahlenEinsetzen(art: "dec" | "hex" | "bin"): void {
  const s = zahlenForm(art); const v = getView(); if (!s) return;
  if (!v) { toast(t("zahlen-kein-editor"), true); return; }
  v.dispatch(v.state.replaceSelection(s), { scrollIntoView: true, userEvent: "input" }); v.focus(); zw.offen = false;
}
