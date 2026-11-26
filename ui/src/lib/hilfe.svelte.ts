// SPDX-License-Identifier: GPL-3.0-or-later
// Hilfe (E13): Themenliste, Suche, Thema öffnen (mit Zurück), Wort nachschlagen (F1), Handbuchbilder (nur mit Zustimmung aus dem Netz). Der Kern liefert Themen und Markdown.
import { onLangChange } from "./i18n.svelte";
import { call } from "./api";
import { ed, getView } from "./editor.svelte";
import { einzeln } from "./settings.svelte";
import { fail, ui } from "./state.svelte";

export interface HThema { id: string; group: string; title: string }
export interface HTreffer { id: string; title: string; snippet: string }
export const hp = $state({ themen: [] as HThema[], suche: "", treffer: [] as HTreffer[], doc: null as null | { title: string; markdown: string; id: string }, verlauf: [] as string[], busy: false, nichts: "", ask: null as null | (() => void) });
const ziel = (): string => (ui.project?.systems?.[0] ?? "").toLowerCase();

onLangChange(async () => { if (!hp.themen.length) return; try { hp.themen = (await call("help_topics")).topics; } catch { /* alte Liste bleibt */ } if (hp.doc) await thema(hp.doc.id); });
export async function hilfeOeffnen(key?: string): Promise<void> {
  ui.extra = "help";
  if (!hp.themen.length) { try { hp.themen = (await call("help_topics")).topics; } catch (e) { fail(e); } }
  if (key !== undefined) await thema(key); else if (!hp.doc && !hp.themen.length) hp.doc = null;
}
export async function thema(key: string, merken = true): Promise<void> {
  hp.nichts = "";
  try {
    const d = await call("help_open", { key, target: ziel(), lang: ed.tabs.find((x) => x.rel === ed.active)?.lang === "basic" ? "basic" : "asm" });
    if (!d) { hp.nichts = key; return; }
    if (merken && hp.doc?.id) hp.verlauf.push(hp.doc.id); hp.doc = d;
  } catch (e) { fail(e); }
}
export async function zurueck(): Promise<void> { const k = hp.verlauf.pop(); if (k) await thema(k, false); else hp.doc = null; }
let timer: ReturnType<typeof setTimeout> | null = null;
export function suchen(q: string): void { hp.suche = q; if (timer) clearTimeout(timer); timer = setTimeout(async () => { if (!q.trim()) { hp.treffer = []; return; } try { hp.treffer = await call("help_search", { q, target: ziel() }); } catch (e) { fail(e); } }, 150); }
/** Wort unter dem Cursor des aktiven Editors */
export function wortUnterCursor(): string {
  const v = getView(); if (!v) return ""; const h = v.state.selection.main;
  if (!h.empty) return v.state.sliceDoc(h.from, h.to).trim().slice(0, 60);
  const ln = v.state.doc.lineAt(h.head); const txt = ln.text; let a = h.head - ln.from, b = a; const ok = (c: string) => /[A-Za-z0-9_$%.#]/.test(c);
  while (a > 0 && ok(txt[a - 1]!)) a--; while (b < txt.length && ok(txt[b]!)) b++; return txt.slice(a, b);
}
export async function hilfeZuWort(): Promise<void> { const w = wortUnterCursor(); await hilfeOeffnen(w || undefined); }

// -- Bilder ------------------------------------------------------------------------------------------------------------------------------------------------------------------------
const cache = new Map<string, string>();
/** Bild nachladen; `ask`: die Zustimmung zum Laden aus dem Netz fehlt (Rückfrage), `never`: abgelehnt */
export async function bildLaden(id: string, hole = false): Promise<{ state: string; img?: string; error?: string }> {
  const c = cache.get(id); if (c) return { state: "ok", img: c };
  try { const r = (await call("help_image", { id, fetch: hole })) as { state: string; img?: string; error?: string }; if (r.state === "ok" && r.img) cache.set(id, r.img); return r; } catch (e) { return { state: "error", error: String((e as Error).message ?? e) }; }
}
export async function bilderErlauben(immer: boolean): Promise<void> { await einzeln((c) => { c.help = { ...(c.help ?? {}), images: immer ? "always" : "never" }; }); }
