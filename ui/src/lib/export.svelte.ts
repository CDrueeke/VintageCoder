// SPDX-License-Identifier: GPL-3.0-or-later
// Export-Dialog der Medien-Editoren (E3.4): Format, Name, Adresse, Vorschau; speichern ins Projekt, per Include einbinden, direkt in den Quelltext kopieren,
// im Emulator testen. Für BASIC DATA-Zeilen, auf Wunsch mit Einleseroutine. Rust rechnet alles; hier steht nur der Dialogzustand.
import { call } from "./api";
import { buildRun } from "./bau.svelte";
import { insertAtCursorLine, sourceTabs, textOf } from "./editor.svelte";
import { t } from "./i18n.svelte";
import { fail, toast } from "./state.svelte";

export const ex = $state({
  open: false, docId: null as number | null, rel: "", kind: "", name: "", addr: "$2000", format: "asm", routine: false, tab: "text" as "text" | "hex", target: "",
  files: [] as [string, number][], text: "", hex: "", error: null as string | null, busy: false, message: "" as string, ok: true,
});
export const braucht = (f: string): boolean => f === "inc" || f === "prg";
/** Ziel des Einfügens: BASIC bei Format DATA, sonst Assembler */
export const zielSprache = (): "asm" | "basic" => (ex.format === "data" ? "basic" : "asm");
export const ziele = (): string[] => sourceTabs(zielSprache());

let seq = 0;
export async function openExport(docId: number): Promise<void> {
  try {
    const i = await call("export_info", { docId });
    Object.assign(ex, { open: true, docId, rel: i.rel, kind: i.kind, name: i.name, addr: i.addr, format: "asm", routine: false, tab: "text", message: "", error: null, files: [], text: "", hex: "" });
    ex.target = ziele()[0] ?? ""; await preview();
  } catch (e) { fail(e); }
}
export async function preview(): Promise<void> {
  if (ex.docId === null) return; const my = ++seq; ex.busy = true;
  try {
    const r = await call("export_preview", { docId: ex.docId, format: ex.format, name: ex.name, addr: ex.addr, routine: ex.routine });
    if (my !== seq) return; ex.files = r.files ?? []; ex.text = r.text ?? ""; ex.hex = r.hex ?? ""; ex.error = r.error ?? null;
  } catch (e) { if (my === seq) { ex.error = String(e); } } finally { if (my === seq) ex.busy = false; }
}
export function setFormat(f: string): void { ex.format = f; if (f !== "data") ex.routine = false; if (!ziele().includes(ex.target)) ex.target = ziele()[0] ?? ""; if (f === "data" && ex.tab === "hex") ex.tab = "text"; void preview(); }
export function close(): void { ex.open = false; }
const sag = (ok: boolean, text: string) => { ex.ok = ok; ex.message = text; };
const bereit = (): boolean => ex.error === null && ex.docId !== null && ex.name.trim() !== "";

export async function saveToProject(): Promise<boolean> {
  if (!bereit()) return false;
  try { const r = await call("export_run", { docId: ex.docId!, action: "save", format: ex.format, name: ex.name, addr: ex.addr }); sag(true, t("export-gespeichert", { files: (r.files ?? []).join(", ") })); const { reloadTree } = await import("./editor.svelte"); await reloadTree(); return true; }
  catch (e) { fail(e); return false; }
}
/** `.include` (Assembler) bzw. `#include` (BASIC) der erzeugten Datei an der Cursorzeile des Ziel-Reiters */
export async function includeIntoSource(): Promise<void> {
  if (!bereit()) return; if (!ex.target) { sag(false, t("export-kein-ziel")); return; }
  try {
    const r = await call("export_run", { docId: ex.docId!, action: "include", name: ex.name, addr: ex.addr, target: ex.target, text: textOf(ex.target) ?? "" });
    const { reloadTree } = await import("./editor.svelte"); await reloadTree();
    if (await insertAtCursorLine(ex.target, r.line ?? "")) { toast(t("export-eingefuegt", { doc: ex.target, line: r.line ?? "" })); close(); } else sag(false, t("export-kein-ziel"));
  } catch (e) { fail(e); }
}
/** Den erzeugten Text selbst in den Quelltext schreiben (ohne Include) */
export async function copyIntoSource(): Promise<void> {
  if (!bereit()) return; if (!ex.target) { sag(false, t("export-kein-ziel")); return; }
  try {
    const r = await call("export_run", { docId: ex.docId!, action: "copy", name: ex.name, addr: ex.addr, routine: ex.routine, targetBasic: ex.format === "data" });
    if (await insertAtCursorLine(ex.target, (r.text ?? "").replace(/\n+$/, ""))) { toast(t("export-kopiert", { doc: ex.target })); close(); } else sag(false, t("export-kein-ziel"));
  } catch (e) { fail(e); }
}
/** Als Assembler speichern, bauen und starten */
export async function testInEmulator(): Promise<void> {
  if (!bereit()) return; ex.format = "asm"; if (!(await saveToProject())) return; close(); await buildRun({ then: "run" });
}
