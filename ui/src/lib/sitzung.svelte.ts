// SPDX-License-Identifier: GPL-3.0-or-later
// Sitzung: zuletzt offene Reiter je Projekt merken und (wenn das Projekt es wünscht) beim Öffnen wiederherstellen; Zwischenkopie unbenannter Reiter.
// Gespeichert wird im Kern (`session.json`, `scratch.json`), nicht in der Oberfläche; so gelten beide Dateien auch für die alte Oberfläche.
import { call } from "./api";
import { ed, openFile, activate, restoreScratch, scratchTexts, sessionHooks, markierungenAlle, markierungenLaden, marksRev } from "./editor.svelte";
import { fail, ui } from "./state.svelte";

export const sess = $state({ ready: false, root: null as string | null, lastKey: "", enabled: false, offer: null as null | { name: string; text: string; note: boolean }[] });

let timer: ReturnType<typeof setTimeout> | null = null;
const keyOf = (rels: string[], active: string | null, marks: unknown = null): string => rels.join("\n") + "\u0000" + (active ?? "") + "\u0000" + JSON.stringify(marks);

/** Offene Dateien des Projekts (ohne unbenannte Reiter) und aktive Datei */
function snapshot(): { open: string[]; active: string | null; marks: ReturnType<typeof markierungenAlle> } {
  const open = ed.tabs.filter((x) => !x.scratch).map((x) => x.rel);
  const a = ed.active && open.includes(ed.active) ? ed.active : null;
  void marksRev.n;   // Haltepunkte und Lesezeichen gehören zur Sitzung und bleiben über einen Neustart
  return { open, active: a, marks: markierungenAlle() };
}

/** Nach dem Öffnen eines Projekts: Sitzung lesen und, wenn eingeschaltet, die Reiter wieder öffnen */
export async function restoreSession(root: string): Promise<void> {
  sess.ready = false; sess.root = root;
  try {
    const s = await call("session_get");
    if (ui.project?.root !== root) return;
    sess.enabled = s.enabled; markierungenLaden(s.marks ?? {});
    if (s.enabled) { for (const rel of s.open) { await openFile(rel); if (ui.project?.root !== root) return; } if (s.active) activate(s.active); }
  } catch (e) { fail(e); }
  if (ui.project?.root !== root) return;
  { const sn = snapshot(); sess.lastKey = keyOf(sn.open, sn.active, sn.marks); } sess.ready = true;
}

/** Sofort schreiben (vor dem Beenden oder Projektwechsel) */
export async function flushSession(): Promise<void> {
  if (timer) { clearTimeout(timer); timer = null; }
  if (!sess.ready || !ui.project) return;
  const s = snapshot(); const k = keyOf(s.open, s.active, s.marks); if (k === sess.lastKey) return;
  sess.lastKey = k;
  try { await call("session_set", s); } catch { /* die Sitzung ist nur eine Bequemlichkeit */ }
}

export async function setRestoreTabs(on: boolean): Promise<void> {
  try { await call("restore_tabs_set", { on }); sess.enabled = on; } catch (e) { fail(e); }
}

/** Zwischenkopie: unbenannte Reiter mit Inhalt sichern (verzögert); beim Beenden mit Verwerfen löscht `clearScratch` sie */
let stimer: ReturnType<typeof setTimeout> | null = null;
export function scheduleScratch(): void {
  if (stimer) clearTimeout(stimer);
  stimer = setTimeout(() => { stimer = null; void writeScratch(); }, 1500);
}
export async function writeScratch(): Promise<void> {
  if (stimer) { clearTimeout(stimer); stimer = null; }
  try { await call("scratch_set", { entries: scratchTexts() }); } catch { /* ohne Kern nichts */ }
}
export async function clearScratch(): Promise<void> { if (stimer) { clearTimeout(stimer); stimer = null; } try { await call("scratch_set", { entries: [] }); } catch { /* weiter */ } }

/** Beim Start: gibt es gesicherte unbenannte Reiter einer früheren Sitzung? Dann fragt ein Dialog */
export async function checkScratch(): Promise<void> {
  try { const r = await call("scratch_get"); if (r.enabled && r.entries.length) sess.offer = r.entries.map((e) => ({ name: e.name, text: e.text, note: e.note })); } catch { /* ohne Kern nichts */ }
}
export async function scratchAnswer(restore: boolean): Promise<void> {
  const o = sess.offer; sess.offer = null; if (!o) return;
  if (restore) { for (const e of o) restoreScratch(e.text, e.note); ui.mode = "code"; await writeScratch(); } else await clearScratch();
}

export function startSitzung(): void {
  sessionHooks.scratch = scheduleScratch; sessionHooks.discard = () => void clearScratch(); sessionHooks.flush = () => void flushSession();
  $effect.root(() => {
    $effect(() => {
      const s = snapshot(); const k = keyOf(s.open, s.active, s.marks);
      if (!sess.ready || !ui.project || k === sess.lastKey) return;
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => void flushSession(), 300);
    });
  });
}
