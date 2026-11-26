// SPDX-License-Identifier: GPL-3.0-or-later
// Turbo BASIC XL (Konzept „Turbo BASIC XL starten“, E3): Zustand der Projektdiskette, Assistent „Diskette bereitstellen“, Karte in den Projekt-Einstellungen.
// Fachliches (Prüfen, Kopieren, Verkleinern) macht der Kern (`vc_ide::turbo`); hier steht nur der Anzeigezustand und der Ablauf des Dialogs.
import { call, CoreError, on, type TurboReport, type TurboState } from "./api";
import { errorText, t } from "./i18n.svelte";
import { fail, setSub, toast, ui } from "./state.svelte";

/** Zustand der Diskette des geöffneten Projekts (`null`: noch nicht geladen oder kein Projekt) */
export const tb = $state({
  state: null as TurboState | null,
  /** Banner „Diskette fehlt“ (beim Starten ohne Diskette; lässt sich schließen) */
  banner: false,
  /** Rückmeldung der Karte (Ergebnis von „Prüfen“, „Verkleinern“) */
  note: null as null | { kind: "ok" | "warn"; text: string; lines: string[] },
  /** Diskette, die beim Wechsel in den Disk-Editor geöffnet werden soll */
  diskPending: null as string | null,
});

export const dlg = $state({
  open: false,
  mode: "provide" as "provide" | "replace" | "reduce",
  src: "",
  check: null as TurboReport | null,
  checking: false,
  /** T4: der Anwender hat sich bei einer nicht erkannten Diskette fürs Verwenden entschieden */
  anyway: false,
  /** T2: einmalige Bestätigung der Verkleinerung (nur Projektkopie) */
  reduce: true,
  busy: false,
  error: "",
  done: null as null | { disk: string; free: number; gitignore: boolean; removed: number },
});

export async function loadTurbo(): Promise<void> {
  if (!ui.project) { tb.state = null; tb.banner = false; tb.note = null; return; }
  try { tb.state = await call("turbo_state"); if (tb.state.turbo && tb.state.present) tb.banner = false; } catch { tb.state = null; }
}
/** Ist das geöffnete Projekt ein Turbo-BASIC-XL-Projekt? */
export const isTurbo = (): boolean => !!(tb.state?.turbo ?? ui.project?.turbo);

const err = (e: unknown): string => (e instanceof CoreError ? errorText(e.code, e.args) : String(e));

/** Name und Version der bereitgestellten Diskette für Karte und Statuszeile */
export function diskName(r: TurboReport | null | undefined): string {
  if (!r) return "";
  const name = r.name || "Turbo BASIC XL";
  return r.version && !name.includes(r.version) ? t("turboui-zustand-name", { name, version: r.version }) : t("turboui-zustand-name-ohne-version", { name });
}
export function statusText(): string {
  const s = tb.state; if (!s?.turbo) return "";
  if (!s.present || !s.report) return t("turboui-status-fehlt");
  return t("turboui-status", { name: diskName(s.report), free: s.report.free_sectors });
}

export function openProvide(mode: "provide" | "replace" = "provide"): void {
  if (!isTurbo()) { toast(t("turboui-nur-turbo"), true); return; }
  Object.assign(dlg, { open: true, mode, src: "", check: null, checking: false, anyway: false, reduce: true, busy: false, error: "", done: null });
}
export function openReduce(): void {
  const r = tb.state?.report?.reduce; if (!r) return;
  Object.assign(dlg, { open: true, mode: "reduce", src: "", check: null, checking: false, anyway: false, reduce: true, busy: false, error: "", done: null });
}

/** Schritt 1: Datei wählen (Dateidialog) und gleich prüfen */
export async function pickDisk(): Promise<void> {
  try {
    const p = await call("turbo_pick"); if (!p) return;
    await checkSource(p);
  } catch (e) { dlg.error = err(e); }
}
export async function checkSource(path: string): Promise<void> {
  dlg.src = path; dlg.check = null; dlg.anyway = false; dlg.error = ""; dlg.checking = true; dlg.done = null;
  try { dlg.check = (await call("turbo_check", { path })) as TurboReport; }
  catch (e) { dlg.error = err(e); }
  finally { dlg.checking = false; }
}

/** Was der Befund erlaubt: erkannt, nicht erkannt (Rückfrage), Fehler (keine Bereitstellung ohne Rückfrage) oder nichts */
export function verdict(c: TurboReport | null): "none" | "ok" | "unrecognized" {
  if (!c) return "none";
  if (c.sameAsCopy) return "none";
  return c.dos2 && c.turbo && c.problems.length === 0 ? "ok" : "unrecognized";
}
/** Darf kopiert werden? Erkannte Disketten ja; sonst nur nach der Entscheidung „trotzdem verwenden“ */
export const canCopy = (): boolean => { const v = verdict(dlg.check); return v === "ok" || (v === "unrecognized" && dlg.anyway); };

export async function runProvide(): Promise<void> {
  if (!dlg.check || dlg.busy || !canCopy()) return;
  dlg.busy = true; dlg.error = "";
  try {
    const reduce = dlg.reduce && !!dlg.check.reduce;
    const r = await call("turbo_provide", { src: dlg.src, reduce, anyway: dlg.anyway, replace: !!dlg.check.destExists || !!dlg.check.current });
    tb.state = r.state; tb.banner = false; tb.note = null;
    if (ui.project) ui.project.tree = r.tree;
    dlg.done = { disk: r.disk, free: r.free, gitignore: r.gitignoreAdded, removed: r.removed.length };
    document.dispatchEvent(new CustomEvent("vc:turbo-bereitgestellt", { detail: { disk: r.disk, free: r.free } }));
  } catch (e) { dlg.error = err(e); }
  finally { dlg.busy = false; }
}

/** Verkleinern der bereitgestellten Kopie (T2, nach der Bestätigung im Dialog) */
export async function runReduce(): Promise<void> {
  if (dlg.busy) return;
  dlg.busy = true; dlg.error = "";
  try {
    const r = await call("turbo_reduce");
    tb.state = r.state;
    dlg.open = false;
    tb.note = { kind: "ok", text: t("turboui-verkleinern-fertig", { count: r.removed.length, free: r.state.free ?? 0 }), lines: [] };
  } catch (e) { dlg.error = err(e); }
  finally { dlg.busy = false; }
}

/** Die bereitgestellte Kopie noch einmal prüfen (Karte, Befehlspalette) */
export async function checkCopy(): Promise<void> {
  if (!isTurbo()) { toast(t("turboui-nur-turbo"), true); return; }
  try {
    const s = (await call("turbo_check", {})) as TurboState; tb.state = s;
    const r = s.report; if (!r) { tb.note = { kind: "warn", text: t("turboui-zustand-fehlt"), lines: [] }; return; }
    const bad = r.problems.length > 0 || !r.turbo || !r.dos2;
    tb.note = { kind: bad ? "warn" : "ok", text: bad ? t("turboui-geprueft", { text: r.problems[0] ?? t("turboui-zustand-nicht-turbo") }) : t("turboui-geprueft", { text: `${diskName(r)}, ${t("turboui-zustand-frei", { count: r.free_sectors, kb: r.kb })}` }), lines: r.lines };
    if (ui.mode === "code" && ui.extra === null) toast(tb.note.text, bad);
  } catch (e) { fail(e); }
}

export async function saveSettings(program: string, files: string[]): Promise<boolean> {
  try { tb.state = await call("turbo_set", { program, files }); toast(t("turboui-gespeichert")); return true; } catch (e) { fail(e); return false; }
}

/** Im Disk-Editor öffnen: der Editor selbst kommt in einer späteren Etappe; bis dahin wechselt der Befehl dorthin und merkt sich die Datei */
export function openInDiskEditor(): void {
  if (!isTurbo()) { toast(t("turboui-nur-turbo"), true); return; }
  const d = tb.state?.disk; if (!d) { toast(t("turboui-zustand-fehlt"), true); return; }
  tb.diskPending = d; setSub("daten", "disk"); toast(t("turboui-disk-editor-bald", { disk: d }));
}

/** Beim Starten ohne Diskette: Banner zeigen */
export function showBanner(): void { tb.banner = true; }

/** Beim Sprachwechsel: Texte aus dem Kern (Prüfzeilen, Namen) neu holen, Rückmeldungen der Karte verwerfen */
export async function startTurbo(): Promise<void> {
  await on("lang:changed", () => { tb.note = null; void loadTurbo(); });
}
