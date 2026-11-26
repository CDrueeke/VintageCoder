// SPDX-License-Identifier: GPL-3.0-or-later
// Einstellungen (E12): die Einstellungsdatei der IDE als Entwurf. Speichern schickt den ganzen Entwurf zum Kern (der ihn prüft). Aussehen (Farbschema, Zoom, Editor) wirkt sofort und wird in der Oberfläche gemerkt.
import { call, type RomImportItem, type AboutInfo, type FeedbackMail, type RomsInfo, type UpdateStand } from "./api";
import { fail, setTheme, toast, ui } from "./state.svelte";
import { onLangChange, t } from "./i18n.svelte";

export type Cfg = Record<string, any>; // eslint-disable-line @typescript-eslint/no-explicit-any
export type SeiteId = "allgemein" | "ziele" | "roms" | "aussehen" | "editor" | "toolchain" | "kuerzel" | "sicherung" | "info";
export const SEITEN: SeiteId[] = ["allgemein", "ziele", "roms", "aussehen", "editor", "toolchain", "kuerzel", "sicherung", "info"];
export const cfg = $state({
  page: "allgemein" as SeiteId, saved: null as Cfg | null, draft: null as Cfg | null, file: null as string | null, home: null as string | null, projekte: "", mac: false,
  dialects: [] as { id: string; name: string; dialect: string | null }[], busy: false,
  imp: null as null | { path: string; targets: number; machines: number; projectsDir: string }, reset: false, msg: "", suche: "",
});
/** JSON mit sortierten Schlüsseln (die Reihenfolge der Felder darf keinen Unterschied machen) */
const stabil = (v: unknown): string => JSON.stringify(v, (_k, x) => (x && typeof x === "object" && !Array.isArray(x) ? Object.fromEntries(Object.entries(x).sort(([a], [b]) => a.localeCompare(b))) : x));
export const geaendert = (): boolean => !!cfg.draft && stabil(cfg.draft) !== stabil(cfg.saved);
const uebernehmen = (r: { settings: Cfg; file: string | null; home: string | null; effectiveProjects: string; mac: boolean; dialects?: typeof cfg.dialects }) => {
  const st = r.settings; for (const k of ["updates", "help", "assembler", "roms"]) st[k] ??= {};   // fehlende Gruppen als leere Gruppen, damit Felder gebunden werden können
  cfg.saved = st; cfg.draft = JSON.parse(JSON.stringify(st)); cfg.file = r.file; cfg.home = r.home; cfg.projekte = r.effectiveProjects; cfg.mac = r.mac; if (r.dialects) cfg.dialects = r.dialects;
};
export async function einstellungenOeffnen(page?: SeiteId): Promise<void> {
  if (page) cfg.page = page; ui.extra = "settings"; cfg.msg = "";
  try { uebernehmen(await call("settings_get")); } catch (e) { fail(e); }
}
export async function speichern(): Promise<boolean> {
  if (!cfg.draft) return false; cfg.busy = true;
  try { const r = await call("settings_save", { settings: cfg.draft }); uebernehmen({ ...r, dialects: cfg.dialects }); cfg.msg = t("setui-gespeichert"); return true; } catch (e) { fail(e); return false; } finally { cfg.busy = false; }
}
export function verwerfen(): void { if (cfg.saved) cfg.draft = JSON.parse(JSON.stringify(cfg.saved)); cfg.msg = ""; }
/** Einzelnen Wert sofort speichern (ohne anderen ungespeicherten Entwurf mitzunehmen) */
export async function einzeln(f: (c: Cfg) => void): Promise<void> {
  if (!cfg.saved) { try { uebernehmen(await call("settings_get")); } catch (e) { fail(e); return; } }
  if (!cfg.saved) return; const g = JSON.parse(JSON.stringify(cfg.saved)) as Cfg; f(g); if (cfg.draft) f(cfg.draft);
  try { const r = await call("settings_save", { settings: g }); for (const k of ["updates", "help", "assembler", "roms"]) r.settings[k] ??= {}; cfg.saved = r.settings; cfg.file = r.file; } catch (e) { fail(e); }
}
export async function dateiZeigen(): Promise<void> { try { await call("settings_reveal"); } catch (e) { fail(e); } }
export async function ordnerWaehlen(): Promise<void> { try { const p = await call("dir_pick", { start: cfg.draft?.projects_dir || cfg.projekte }); if (p && cfg.draft) cfg.draft.projects_dir = p; } catch (e) { fail(e); } }
export async function exportieren(): Promise<void> { try { const r = await call("settings_export"); if (r) cfg.msg = t("konfig-exportiert", { path: r.path }); } catch (e) { fail(e); } }
export async function importWaehlen(): Promise<void> { try { const r = await call("settings_import_pick"); if (r) cfg.imp = r; } catch (e) { fail(e); } }
export async function importJa(): Promise<void> { if (!cfg.imp) return; const p = cfg.imp.path; cfg.imp = null; try { uebernehmen({ ...(await call("settings_import_apply", { path: p })), dialects: cfg.dialects }); cfg.msg = t("konfig-uebernommen", { path: p }); } catch (e) { fail(e); } }
export async function resetJa(): Promise<void> { cfg.reset = false; try { uebernehmen({ ...(await call("settings_reset")), dialects: cfg.dialects }); setTheme("system"); cfg.msg = t("konfig-zurueckgesetzt"); toast(t("konfig-zurueckgesetzt")); } catch (e) { fail(e); } }
/** Pfad im persönlichen Ordner als „~“ */
export const kurz = (p: string): string => (cfg.home && p.startsWith(cfg.home) ? "~" + p.slice(cfg.home.length) : p);

// -- Zielsysteme und Hardware ------------------------------------------------------------------------------------------------------------------------------------------------
export const zl = $state({ info: [] as { id: string; name: string; group: string; cpu: string; kinds: { id: string; label: string }[]; hardware: { id: string; label: string }[] }[], sel: "", proben: {} as Record<string, { ok: boolean; text: string; busy?: boolean }>, anwesend: {} as Record<string, { found: boolean; auto: boolean; text: string }>, ports: [] as string[], zeigen: {} as Record<string, boolean> });
export async function zieleLaden(): Promise<void> {
  try { zl.info = (await call("ziele_info")).systems; } catch (e) { fail(e); }
  const first = (cfg.draft?.targets as { system: string }[] | undefined)?.[0]?.system; if (!zl.sel || !(cfg.draft?.targets as { system: string }[] | undefined)?.some((x) => x.system === zl.sel)) zl.sel = first ?? "";
  void pruefenAlle(); void portsLaden();
}
export async function zielOp(op: Record<string, unknown>): Promise<void> {
  if (!cfg.draft) return;
  try { const r = await call("settings_op", { settings: cfg.draft, op }); const neu = r.settings; for (const k of ["updates", "help", "assembler", "roms"]) neu[k] ??= {}; cfg.draft = neu; if (op.kind === "add_target") zl.sel = String(op.system); if (op.kind === "remove_target" && zl.sel === op.system) zl.sel = (neu.targets as { system: string }[])[0]?.system ?? ""; void pruefenAlle(); } catch (e) { fail(e); }
}
/** Ist das Programm eines Emulators da? (je Emulator, nach Pfadänderung neu) */
export async function pruefen(emu: Record<string, unknown>): Promise<void> {
  try { zl.anwesend[String(emu.id)] = await call("ziele_probe", { kind: "presence", emulator: emu }); } catch { /* ohne Anzeige */ }
}
export async function pruefenAlle(): Promise<void> { for (const t of (cfg.draft?.targets ?? []) as { emulators: Record<string, unknown>[] }[]) for (const e of t.emulators ?? []) void pruefen(e); }
export async function emuSuchen(emu: Record<string, unknown>): Promise<void> { try { const p = await call("ziele_probe", { kind: "discover", ekind: emu.kind }); if (p) { emu.path = p; void pruefen(emu); } else zl.proben[String(emu.id)] = { ok: false, text: t("ziele-nicht-gefunden") }; } catch (e) { fail(e); } }
export async function emuTesten(emu: Record<string, unknown>): Promise<void> { const k = String(emu.id); zl.proben[k] = { ok: true, text: t("ziele-testet"), busy: true }; try { zl.proben[k] = await call("ziele_probe", { kind: "test", emulator: emu }); } catch (e) { zl.proben[k] = { ok: false, text: String((e as Error).message ?? e) }; } }
export async function maschineTesten(m: Record<string, unknown>, link?: string): Promise<void> { const k = "hw:" + String(m.id); zl.proben[k] = { ok: true, text: t("ziele-testet"), busy: true }; try { zl.proben[k] = await call("ziele_probe", { kind: "hardware", machine: m, link }); } catch (e) { zl.proben[k] = { ok: false, text: String((e as Error).message ?? e) }; } }
export async function portsLaden(): Promise<void> { try { zl.ports = await call("ziele_probe", { kind: "ports" }); } catch { /* keine Liste */ } }

// -- ROMs --------------------------------------------------------------------------------------------------------------------------------------------------------------------------
export const rm = $state({ info: null as RomsInfo | null, sel: "", busy: false });
export async function romsLaden(): Promise<void> { rm.busy = true; try { rm.info = await call("roms_info", { useForeign: !!cfg.draft?.roms?.use_foreign }); if (!rm.sel || !rm.info.systems.some((s) => s.id === rm.sel)) rm.sel = rm.info.systems[0]?.id ?? ""; } catch (e) { fail(e); } finally { rm.busy = false; } }
/** ROM-Import: Plan (nichts wird kopiert, bis bestätigt), dann nur die angehakten Einträge */
export const ri = $state({ paths: [] as string[], foreign: false, items: [] as RomImportItem[], pick: [] as number[], msg: "", busy: false, planned: false });
async function importPlan(paths: string[], foreign: boolean): Promise<void> {
  ri.busy = true; ri.msg = "";
  try {
    const r = await call("roms_import_plan", { paths, foreign, system: rm.sel });
    Object.assign(ri, { paths, foreign, items: r.items, pick: r.items.filter((x) => x.action === "copy").map((x) => x.i), planned: true });
    if (!r.items.length) ri.msg = t(foreign ? "roms-import-nichts-fremd" : "roms-import-nichts");
  } catch (e) { fail(e); } finally { ri.busy = false; }
}
export async function romsWaehlen(): Promise<void> { try { const p = await call("roms_pick"); if (p.length) await importPlan(p, false); } catch (e) { fail(e); } }
export const romsAbgelegt = (paths: string[]): Promise<void> => importPlan(paths, false);
export const romsFremdSuchen = (): Promise<void> => importPlan([], true);
export function romsHaken(i: number, an: boolean): void { ri.pick = an ? [...new Set([...ri.pick, i])] : ri.pick.filter((x) => x !== i); }
export async function romsImportieren(): Promise<void> {
  if (!ri.pick.length || ri.busy) return; ri.busy = true;
  try {
    const r = await call("roms_import_run", { paths: ri.paths, foreign: ri.foreign, system: rm.sel, pick: ri.pick });
    ri.msg = r.errors.length ? r.errors.join(" · ") : t("roms-import-fertig", { n: r.copied.length });
    ri.items = []; ri.pick = []; ri.planned = false; await romsLaden();
  } catch (e) { fail(e); } finally { ri.busy = false; }
}
export async function romsFestnageln(system: string, role: string, file: string): Promise<void> { try { await call("roms_pin", { system, role, file }); await romsLaden(); toast(t("roms-fest-gespeichert")); } catch (e) { fail(e); } }
/** Fundliste als Text (Systeme, Rollen, Zustand, Prüfsummen) in die Zwischenablage; ohne ROM-Inhalte */
export async function romsListeKopieren(): Promise<void> {
  if (!rm.info) return;
  const z = rm.info.systems.flatMap((s) => [s.label, ...s.roles.map((r) => `  ${r.label}: ${r.statusLabel}${r.path ? ` (${r.place ?? ""} ${r.revision ?? ""} CRC-32 ${r.crc32 ?? "–"})` : ""}`)]).join("\n");
  try { await navigator.clipboard.writeText(z); toast(t("roms-liste-kopiert")); } catch (e) { fail(e); }
}
onLangChange(() => { if (zl.info.length) void zieleLaden(); if (rm.info) void romsLaden(); if (inf.about) void infoLaden(); });
export async function romOrdner(): Promise<void> { try { await call("roms_open"); } catch (e) { fail(e); } }

// -- Info, Update, Feedback ----------------------------------------------------------------------------------------------------------------------------------------------------
export const inf = $state({ about: null as AboutInfo | null, upd: null as UpdateStand | null, busy: false, fehler: "", hinweise: false, fb: null as FeedbackMail | null, fbOpen: false, fbMsg: "", pruefte: false });
export async function infoLaden(): Promise<void> { try { inf.about = await call("info_about"); inf.upd = inf.about.update; } catch (e) { fail(e); } }
/** Stand für die Statusleiste (beim Start und nach einer Prüfung) */
export async function updateStandLaden(): Promise<void> { try { inf.upd = await call("info_update_state"); } catch { /* ohne Anzeige */ } }
/** Beim Start: wenn die gewählte Häufigkeit es erlaubt, im Hintergrund prüfen; danach den Stand neu lesen */
export async function updateAutoStart(): Promise<void> { await updateStandLaden(); try { inf.upd = await call("info_update_check", { auto: true }); } catch { /* still */ } }
export async function updatePruefen(): Promise<void> { inf.busy = true; inf.fehler = ""; try { const r = await call("info_update_check", { auto: false }); inf.upd = r; inf.fehler = r.error ?? ""; inf.pruefte = true; } catch (e) { fail(e); } finally { inf.busy = false; } }
export async function updateSpaeter(): Promise<void> { try { inf.upd = await call("info_update_later"); } catch (e) { fail(e); } }
export async function updateUeberspringen(version: string): Promise<void> { try { inf.upd = await call("info_update_skip", { version }); } catch (e) { fail(e); } }
export async function adresseOeffnen(which: "web" | "repo" | "release", version?: string): Promise<void> { try { await call("info_open", { which, version }); } catch (e) { fail(e); } }
/** Link einer Danksagung öffnen (nur auf Klick, im Standardbrowser; der Kern öffnet nur Adressen aus seiner Liste) */
export async function danksagungOeffnen(id: string, index: number): Promise<void> { try { await call("info_open", { which: "credit", id, index }); } catch (e) { fail(e); } }
export async function feedbackOeffnen(system?: string): Promise<void> { inf.fbMsg = ""; try { inf.fb = await call("info_feedback", { system }); inf.fbOpen = true; } catch (e) { fail(e); } }
export async function feedbackMail(): Promise<void> { if (!inf.fb) return; try { await call("info_mail", { url: inf.fb.url }); inf.fbMsg = t("info-mail-geoeffnet"); } catch (e) { fail(e); } }
export async function feedbackKopieren(): Promise<void> { if (!inf.fb) return; try { await navigator.clipboard.writeText(inf.fb.clipboard); inf.fbMsg = t("info-kopiert"); } catch { inf.fbMsg = t("info-kopieren-fehlt"); } }
