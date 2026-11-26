// SPDX-License-Identifier: GPL-3.0-or-later
// Disk-Editor (E2): ein geöffnetes Diskettenabbild als Dokument im Kern (`doc_*`); die Oberfläche hält nur die Ansicht und sendet Absichten.
import { onLangChange } from "./i18n.svelte";
import { call, type DiskView } from "./api";
import { fail, setMode, setSub, toast, ui } from "./state.svelte";

export const dk = $state({ docId: null as number | null, view: null as DiskView | null, /** Rückfrage bei ungespeicherter Diskette: danach auszuführen */ ask: null as null | (() => void | Promise<void>) });
onLangChange(async () => { const id = dk.docId; if (id === null) return; try { dk.view = (await call("disk_view", { docId: id })) as DiskView; } catch { /* Ansicht bleibt */ } });   // Kern-Texte der Ansicht folgen der Sprache

/** Ist etwas ungespeichert, erst fragen (Speichern / Verwerfen / Abbrechen), sonst gleich ausführen */
export function askThen(then: () => void | Promise<void>): void { if (dk.view?.dirty) dk.ask = then; else void then(); }

/** Diskettenabbild öffnen (`rel` im Projekt oder `path` absolut); ein bereits geöffnetes wird zuvor geschlossen */
export async function openDisk(o: { rel?: string; path?: string }): Promise<void> {
  if (dk.view?.dirty) { dk.ask = () => openDisk(o); return; }
  try {
    const r = await call("disk_open", o);
    await closeDisk();
    dk.docId = r.docId; dk.view = r.view;
    setMode("daten"); setSub("daten", "disk");
  } catch (e) { fail(e); }
}
export async function pickDisk(): Promise<void> {
  try { const p = await call("disk_pick"); if (p) await openDisk({ path: p }); } catch (e) { fail(e); }
}
export async function closeDisk(): Promise<void> {
  const id = dk.docId; dk.docId = null; dk.view = null;
  if (id !== null) { try { await call("disk_close", { docId: id }); } catch { /* Kern schon weg */ } }
}
/** Eine Absicht an das Dokument schicken; der Kern antwortet mit der neuen Ansicht */
export async function diskOp(op: Record<string, unknown>): Promise<boolean> {
  if (dk.docId === null) return false;
  try { dk.view = await call("disk_apply", { docId: dk.docId, op }); const n = dk.view.note; if (n) toast(n.text, !n.ok); return true; } catch (e) { fail(e); return false; }
}
export const selectFile = (i: number): Promise<boolean> => diskOp({ kind: "file_select", i });
export const selectSector = (track: number, sector: number): Promise<boolean> => diskOp({ kind: "sector_select", track, sector });
export const follow = (): Promise<boolean> => diskOp({ kind: "follow" });
export const back = (): Promise<boolean> => diskOp({ kind: "back" });
/** Verzeichniseintrag `from` an die Stelle `to` setzen (ein Zug = ein Rückgängig-Schritt) */
export const fileMove = (from: number, to: number): Promise<boolean> => diskOp({ kind: "file_move", from, to });
export const diskUndo = (): Promise<boolean> => diskOp({ kind: "undo" });
export const diskRedo = (): Promise<boolean> => diskOp({ kind: "redo" });
/** Ist der Disk-Editor der aktive Bereich mit geöffneter Diskette? (Rückgängig/Wiederholen gehen dann an die Diskette) */
export const aktiv = (): boolean => !!dk.view && ui.mode === "daten" && ui.sub.daten === "disk";
export const setChain = (on: boolean): Promise<boolean> => diskOp({ kind: "chain", on });

/** Neue, leere Diskette anlegen (noch nicht gespeichert) */
export async function newDisk(format: string, name: string, id: string): Promise<void> {
  if (dk.view?.dirty) { dk.ask = () => newDisk(format, name, id); return; }
  try { const r = await call("disk_new", { format, name, id }); await closeDisk(); dk.docId = r.docId; dk.view = r.view; setMode("daten"); setSub("daten", "disk"); } catch (e) { fail(e); }
}
export const validate = (): Promise<boolean> => diskOp({ kind: "validate" });
export async function addFiles(): Promise<void> {
  try { const ps = await call("disk_add_pick"); if (ps.length) await diskOp({ kind: "add_files", paths: ps }); } catch (e) { fail(e); }
}
/** Speichern; ohne Pfad (neue Diskette) wie „Speichern unter …“ */
export async function saveDisk(): Promise<boolean> {
  const v = dk.view; if (!v) return true;
  if (v.path) return diskOp({ kind: "save" });
  return saveDiskAs();
}
export async function saveDiskAs(): Promise<boolean> {
  const v = dk.view; if (!v) return true;
  try { const p = await call("disk_save_pick", { name: v.path ? v.path.split("/").pop()! : `${v.title.replace(/[^A-Za-z0-9_.-]/g, "_") || "diskette"}.${v.format.toLowerCase()}` }); if (!p) return false; return diskOp({ kind: "save_as", path: p }); } catch (e) { fail(e); return false; }
}
/** Vorgemerkte Dateien ins Projekt übernehmen; danach den Dateibaum neu lesen */
export async function importMarked(): Promise<void> {
  if (dk.docId === null) return;
  try { dk.view = await call("disk_import", { docId: dk.docId }); const { reloadTree } = await import("./editor.svelte"); await reloadTree(); } catch (e) { fail(e); }
}

/** Antwort auf die Rückfrage bei ungespeicherter Diskette */
export async function answerAsk(what: "save" | "discard" | "cancel"): Promise<void> {
  const then = dk.ask; dk.ask = null; if (!then || what === "cancel") return;
  if (what === "save" && !(await saveDisk())) return;
  if (what === "discard") await closeDisk();
  await then();
}
/** Diskette im Standard-Emulator des Systems einlegen (Laufwerk 8); die Diskette muss gespeichert sein */
export const insertDisk = (): Promise<boolean> => diskOp({ kind: "insert", root: ui.project?.root });
