// SPDX-License-Identifier: GPL-3.0-or-later
// Einrichtungsassistent (E12.3): Ablauf, Entwurf (im Kern) und Darstellung. Die Seiten für Ziele und ROMs sind die der Einstellungen; sie bearbeiten denselben Entwurf (`cfg.draft`).
import { onLangChange } from "./i18n.svelte";
import { call, type WizData } from "./api";
import { cfg, zl } from "./settings.svelte";
import { ed, fontStep } from "./editor.svelte";
import { startFertig } from "./ablegen.svelte";
import { fail, setTheme, setZoom, ui, type Theme } from "./state.svelte";

/** Willkommenskarte der Startseite „Später“: gemerkt (dauerhaft) und für den Hinweis in der Statuszeile */
export const willkommen = $state({ weg: (() => { try { return localStorage.getItem("willkommen-weg") === "1"; } catch { return false; } })() });
export const wz = $state({ hinweis: false, open: false, data: null as WizData | null, step: 0, abgewaehlt: [] as string[], report: null as null | { notes: string[]; copied: string[]; backup: string | null }, busy: false, vorher: null as null | { theme: Theme; zoom: number; font: number } });
const norm = (s: Record<string, unknown>) => { for (const k of ["updates", "help", "assembler", "roms"]) (s as Record<string, unknown>)[k] ??= {}; return s; };
const uebernehmen = (d: WizData) => { wz.data = d; cfg.draft = norm(JSON.parse(JSON.stringify(d.settings))); cfg.saved = cfg.saved ?? norm(JSON.parse(JSON.stringify(d.settings))); };
export const thema = (): string => wz.data?.steps[wz.step]?.id ?? "";

export async function wizStarten(mode = "quick"): Promise<void> {
  try {
    const d = await call("wiz_start", { mode }); wz.hinweis = false; wz.open = true; wz.step = 0; wz.abgewaehlt = []; wz.report = null; ui.extra = "wizard"; uebernehmen(d);
    cfg.saved = norm(JSON.parse(JSON.stringify(d.settings))); wz.vorher ??= { theme: ui.theme, zoom: ui.zoom, font: ed.fontSize };
    const t = d.steps.findIndex((s) => s.id === "welcome"); wz.step = t >= 0 ? t + 0 : 0;
    // die Seiten der Einstellungen brauchen die Systeme des Programms
    try { zl.info = (await call("ziele_info")).systems; } catch { /* ohne Liste */ }
  } catch (e) { fail(e); }
}
/// Beim Start: die Einrichtung öffnet sich NICHT von selbst. Fehlen Einstellungen, zeigt die Startseite die Willkommenskarte; mit offenem Projekt steht ein Hinweis in der Statuszeile
export async function wizAutoStart(): Promise<void> {
  try {
    await startFertig();   // wurde die App mit einem Projekt gestartet, öffnet sich erst das Projekt
    const s = await call("wiz_stand"); wz.hinweis = s.start === "first" && !s.suppressed;
  } catch { /* ohne Assistenten */ }
}

/** Entwurf der Einstellungsseiten (cfg.draft) zum Kern schicken */
export async function wizSpeichern(extra: Record<string, unknown> = {}): Promise<void> {
  if (!wz.open) return;
  try { const d = await call("wiz_set", { settings: cfg.draft, deselected: wz.abgewaehlt, ...extra }); wz.data = d; if (extra.settings === undefined) { /* cfg.draft bleibt, was die Seite bearbeitet hat */ } } catch (e) { fail(e); }
}
export async function wizSystem(id: string, on: boolean): Promise<void> { try { await wizSpeichern(); const d = await call("wiz_system", { id, on, deselected: wz.abgewaehlt }); uebernehmen(d); } catch (e) { fail(e); } }
onLangChange(() => { if (wz.open) void wizSetzen({}); });
export async function wizSetzen(teil: Record<string, unknown>): Promise<void> { try { const d = await call("wiz_set", { ...teil, deselected: wz.abgewaehlt }); wz.data = d; } catch (e) { fail(e); } }
export async function wizSchritt(d: number): Promise<void> {
  if (!wz.data) return; await wizSpeichern();
  wz.step = Math.max(0, Math.min(wz.data.steps.length - 1, wz.step + d));
}
export async function wizWeg(mode: string): Promise<void> { await wizStarten(mode); }
export async function wizUebernehmen(): Promise<boolean> {
  wz.busy = true;
  try { await wizSpeichern(); const d = await call("wiz_apply", { deselected: wz.abgewaehlt }); wz.report = d.report; uebernehmen(d); cfg.saved = norm(JSON.parse(JSON.stringify(d.settings))); wz.vorher = null; return true; }
  catch (e) { fail(e); return false; } finally { wz.busy = false; }
}
export async function wizRueckgaengig(): Promise<void> { try { const d = await call("wiz_undo", { deselected: wz.abgewaehlt }); wz.report = null; uebernehmen(d); } catch (e) { fail(e); } }
function darstellungZurueck(): void { if (wz.vorher) { setTheme(wz.vorher.theme); setZoom(wz.vorher.zoom); fontStep(wz.vorher.font - ed.fontSize); wz.vorher = null; } }
export async function wizBeenden(abbruch = false): Promise<void> {
  if (abbruch && !wz.data?.applied) darstellungZurueck();
  try { await call("wiz_end"); } catch { /* weg */ }
  wz.open = false; wz.hinweis = false; wz.data = null; wz.vorher = null; if (ui.extra === "wizard") ui.extra = null;
}
export async function wizSpaeter(): Promise<void> { const t = thema(); try { await call("wiz_later", { topic: t }); } catch { /* ohne */ } await wizBeenden(true); }
export const schalten = (id: string): void => { wz.abgewaehlt = wz.abgewaehlt.includes(id) ? wz.abgewaehlt.filter((k) => k !== id) : [...wz.abgewaehlt, id]; };
