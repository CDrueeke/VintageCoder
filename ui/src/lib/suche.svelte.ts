// SPDX-License-Identifier: GPL-3.0-or-later
// Projektsuche (E1.2): Eingabe, laufende Suche im Kern (Hintergrund, abbrechbar), Treffer als Ereignisse. Ältere Antworten (andere Nummer) werden verworfen.
import { call, on, type SearchDone, type SearchHit } from "./api";
import { fail } from "./state.svelte";

export const search = $state({ text: "", case: false, running: false, hits: [] as SearchHit[], done: null as SearchDone | null, id: 0, focus: 0 });
let texts: () => Record<string, string> = () => ({});
let timer: ReturnType<typeof setTimeout> | undefined;

/** Texte ungespeicherter Reiter liefern (sie gehen der Datei auf der Platte vor) */
export function bindSearchTexts(f: () => Record<string, string>): void { texts = f; }

export async function startSearch(): Promise<void> {
  clearTimeout(timer);
  search.hits = []; search.done = null;
  if (!search.text) { await cancelSearch(); return; }
  search.running = true;
  try { const r = await call("search_start", { text: search.text, case: search.case, overrides: texts() }); search.id = r.id; }
  catch (e) { search.running = false; fail(e); }
}
export function searchSoon(): void { clearTimeout(timer); timer = setTimeout(() => void startSearch(), 250); }
export async function cancelSearch(): Promise<void> { clearTimeout(timer); try { await call("search_cancel"); } catch (e) { fail(e); } search.running = false; }
export function resetSearch(): void { clearTimeout(timer); search.text = ""; search.hits = []; search.done = null; search.running = false; }

let listening = false;
export async function startSuche(): Promise<void> {
  if (listening) return; listening = true;
  await on("search:hits", (p) => { const ev = p as { id: number; hits: SearchHit[] }; if (ev.id >= search.id) { search.id = ev.id; search.hits.push(...ev.hits); } });
  await on("search:done", (p) => { const ev = p as SearchDone; if (ev.id >= search.id) { search.id = ev.id; search.done = ev; search.running = false; } });
}
