// SPDX-License-Identifier: GPL-3.0-or-later
// Dialog „Neues Projekt“ (E1.4): Eingaben halten, Optionen je Zielsystem und die Vorschau vom Kern holen, Projekt anlegen und öffnen.
import { call, type NewProjForm, type NewProjInfo, type NewProjSystem } from "./api";
import { errorText } from "./i18n.svelte";
import { CoreError } from "./api";
import { toast, ui } from "./state.svelte";
import { onLangChange, t } from "./i18n.svelte";

export const np = $state({
  open: false, busy: false, error: "",
  info: null as NewProjInfo | null, opt: null as NewProjSystem | null, files: [] as string[],
  form: { system: "c64", extra: [], basic: false, card: null, name: "meinprojekt", dir: "", dialect: "", norm: "pal", memory: "", emulator: "", startOn: "", disk: "", charset: "auto", structure: 0, readme: false, git: false } as NewProjForm,
  dirPretty: "",
});

export const nameOk = (n: string): boolean => /^[A-Za-z0-9_-]{1,48}$/.test(n);
/** Vorlagenkarten, die zu Zielsystem und Sprache passen */
export const cards = (): NewProjInfo["cards"] => (np.info ? np.info.cards.filter((c) => c.basic === np.form.basic && c.systems.includes(np.form.system)) : []);

/** Auswahl nach einer Änderung gültig halten (wie `abgleichen` der alten Oberfläche) */
function align(): void {
  const f = np.form; const o = np.opt; if (!np.info) return;
  const ok = cards(); if (f.card === null || !ok.some((c) => c.i === f.card)) f.card = ok[0]?.i ?? null;
  const card = f.card === null ? null : np.info.cards[f.card];
  f.extra = f.extra.filter((e) => e !== f.system && !!card?.systems.includes(e));
  if (!o) return;
  if (!f.dialect || !o.dialects.some((d) => d.id === f.dialect)) f.dialect = o.dialects[0]?.id ?? "";
  if (f.disk && !o.disks.includes(f.disk)) f.disk = "";
  if (!o.emulators.some((e) => e.id === f.emulator)) f.emulator = o.emulators.find((e) => e.global)?.id ?? o.emulators[0]?.id ?? "";
  if (f.startOn && !o.machines.some((m) => m.id === f.startOn)) f.startOn = "";
  if (f.system === "vic20" && !f.memory) f.memory = o.memory[0]?.id ?? "";
  if (f.system !== "vic20") f.memory = "";
}

let timer: ReturnType<typeof setTimeout> | null = null;
function refreshPreview(): void {
  if (timer) clearTimeout(timer);
  timer = setTimeout(async () => { if (np.form.card === null) { np.files = []; return; } try { np.files = await call("newproj_preview", { ...np.form }); } catch { np.files = []; } }, 120);
}

export async function openNewProject(): Promise<void> {
  try {
    const info = await call("newproj_info"); np.info = info; np.error = ""; np.dirPretty = info.dirPretty;
    np.form = { ...np.form, system: info.default, extra: [], basic: false, card: null, name: np.form.name || "meinprojekt", dir: info.dir, dialect: "", norm: "pal", memory: "", emulator: "", startOn: "", disk: "", charset: "auto", structure: 0, readme: false, git: false };
    np.opt = await call("newproj_system", { system: np.form.system }); align(); np.open = true; refreshPreview();
  } catch (e) { toast(e instanceof CoreError ? errorText(e.code, e.args) : String(e), true); }
}
onLangChange(async () => { if (!np.open) return; try { const i = await call("newproj_info"); np.info = i; np.opt = await call("newproj_system", { system: np.form.system }); } catch { /* alter Stand bleibt */ } });
export async function setSystem(id: string): Promise<void> { np.form.system = id; np.form.card = null; np.opt = await call("newproj_system", { system: id }).catch(() => null); align(); refreshPreview(); }
export function setBasic(b: boolean): void { np.form.basic = b; np.form.card = null; align(); refreshPreview(); }
export function setCard(i: number): void { np.form.card = i; align(); refreshPreview(); }
export function setField<K extends keyof NewProjForm>(k: K, v: NewProjForm[K]): void { np.form[k] = v; np.error = ""; refreshPreview(); }
export function toggleExtra(id: string, on: boolean): void { np.form.extra = np.form.extra.filter((x) => x !== id); if (on) np.form.extra.push(id); refreshPreview(); }
export async function pickDir(): Promise<void> {
  try { const p = await call("dir_pick", { start: np.form.dir }); if (p) { np.form.dir = p; np.dirPretty = p; } } catch (e) { toast(e instanceof CoreError ? errorText(e.code, e.args) : String(e), true); }
}
export function closeNewProject(): void { np.open = false; }
/** Anlegen und öffnen; ein Fehler bleibt im Dialog sichtbar */
export async function createProject(): Promise<boolean> {
  if (!nameOk(np.form.name) || np.form.card === null) return false;
  np.busy = true; np.error = "";
  try {
    const r = await call("newproj_create", { ...np.form });
    np.open = false; ui.project = r; ui.selected = null; ui.mode = "code"; ui.extra = null;
    await call("menu_context", { project: true, editor: false }).catch(() => {});
    for (const n of r.notes) toast(n, true);
    toast(t("neuprojekt-angelegt", { dir: r.pretty }));
    return true;
  } catch (e) { np.error = e instanceof CoreError ? errorText(e.code, e.args) : String(e); return false; }
  finally { np.busy = false; }
}
