// SPDX-License-Identifier: GPL-3.0-or-later
// Dialog „Projekt kopieren“ (E1.4): Name, Ort und was mitkopiert wird; der Kern kopiert (`vc_ide::copyproject`). Beispielprojekte lassen sich so gefahrlos ausprobieren.
import { call, CoreError } from "./api";
import { errorText, t } from "./i18n.svelte";
import { openProject, toast } from "./state.svelte";

export const kop = $state({ open: false, src: "", name: "", dir: "", withBuild: false, withGit: false, after: true, system: "", systems: [] as { id: string; name: string }[], problem: null as string | null, info: null as { files: number; bytes: number } | null, busy: false, error: "" });

export const humanBytes = (b: number): string => (b < 10_000 ? t("kopierdlg-b-byte", { b }) : b < 10_000_000 ? t("kopierdlg-p1-kb", { p1: Math.round(b / 1000) }) : t("kopierdlg-p1-mb", { p1: (b / 1e6).toFixed(1) }));

async function recheck(): Promise<void> {
  try { kop.problem = await call("copy_check", { src: kop.src, dir: kop.dir, name: kop.name }); kop.info = await call("copy_plan", { src: kop.src, withBuild: kop.withBuild, withGit: kop.withGit }); } catch { /* ohne Kern nichts */ }
}
export async function openCopy(src: string): Promise<void> {
  try {
    const dir = src.replace(/[\\/][^\\/]*$/, "") || src;
    Object.assign(kop, { open: true, src, dir, withBuild: false, withGit: false, after: true, system: "", error: "", busy: false });
    call("newproj_info").then((i) => (kop.systems = i.systems.map((x) => ({ id: x.id, name: x.name })))).catch(() => {});
    kop.name = await call("copy_suggest", { src, dir }); await recheck();
  } catch (e) { toast(e instanceof CoreError ? errorText(e.code, e.args) : String(e), true); }
}
export function setCopy<K extends "name" | "withBuild" | "withGit" | "after" | "system">(k: K, v: (typeof kop)[K]): void { kop[k] = v; kop.error = ""; void recheck(); }
export async function pickCopyDir(): Promise<void> { try { const p = await call("dir_pick", { start: kop.dir }); if (p) { kop.dir = p; kop.error = ""; await recheck(); } } catch (e) { toast(e instanceof CoreError ? errorText(e.code, e.args) : String(e), true); } }
export async function runCopy(): Promise<void> {
  if (kop.problem || kop.busy) return;
  kop.busy = true; kop.error = "";
  try {
    const r = await call("copy_run", { src: kop.src, dir: kop.dir, name: kop.name, withBuild: kop.withBuild, withGit: kop.withGit, system: kop.system || undefined });
    kop.open = false; toast(t("kopierdlg-projekt-kopiert-p1-p2-dateien-p3", { p1: r.root, p2: r.files, p3: humanBytes(r.bytes) }));
    for (const s of r.skipped) toast(t("kopierdlg-uebersprungen-s", { s }), true);
    // Rechtstext (Turbo BASIC XL): nach dem Weitergeben/Kopieren ein Hinweis, wenn eine Diskette des Anwenders im Projekt liegt
    if (kop.system) toast(t("kopierdlg-system-hinweis"), false);
    if (r.turboDisk) toast(t("rechtlich-turbo-export-nachher"), true);
    if (kop.after) await openProject(r.root);
  } catch (e) { kop.error = e instanceof CoreError ? errorText(e.code, e.args) : String(e); }
  finally { kop.busy = false; }
}
