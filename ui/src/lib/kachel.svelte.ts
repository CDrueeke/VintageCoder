// SPDX-License-Identifier: GPL-3.0-or-later
// Kachel-Editor (E8): ein geöffneter Kachelsatz (.vctiles) als Dokument im Kern (`kacheln_open`; `pixel_apply`/`pixel_view`/`pixel_close` sind allgemein). Die Oberfläche hält nur
// den Anzeigezustand und sendet Absichten nacheinander; ein Pinselzug über mehrere Zellen ist eine Absicht (ein Rückgängig-Schritt).
import { onLangChange } from "./i18n.svelte";
import { call, callAny, type KachelView, type TreeNode } from "./api";
import { fail, setMode, setSub, toast, ui } from "./state.svelte";

export const kc = $state({ docId: null as number | null, view: null as KachelView | null, pending: 0, ask: null as null | (() => void | Promise<void>) });
onLangChange(async () => { const id = kc.docId; if (id === null) return; try { kc.view = (await callAny("pixel_view", { docId: id })) as KachelView; } catch { /* Ansicht bleibt */ } });   // Kern-Texte der Ansicht folgen der Sprache

export const istKachelDatei = (rel: string): boolean => /\.vctiles$/i.test(rel);
export const aktiv = (): boolean => !!kc.view && ui.mode === "grafik" && ui.sub.grafik === "kacheln";
export function kachelDateien(tree: TreeNode[] | undefined): string[] {
  const out: string[] = []; const walk = (ns: TreeNode[]) => { for (const n of ns) { if (n.kind === "dir") walk(n.children ?? []); else if (istKachelDatei(n.rel)) out.push(n.rel); } };
  walk(tree ?? []); return out;
}
export async function openKacheln(rel: string): Promise<void> {
  if (kc.view?.rel === rel) { setMode("grafik"); setSub("grafik", "kacheln"); return; }
  if (kc.view?.dirty) { kc.ask = () => openKacheln(rel); return; }
  try {
    const r = await call("kacheln_open", { rel });
    await closeKacheln();
    kc.docId = r.docId; kc.view = r.view; setMode("grafik"); setSub("grafik", "kacheln");
  } catch (e) { fail(e); }
}
export async function closeKacheln(): Promise<void> {
  const id = kc.docId; kc.docId = null; kc.view = null;
  if (id !== null) { try { await call("pixel_close", { docId: id }); } catch { /* Kern schon weg */ } }
}
export async function pickKacheln(): Promise<void> {
  try { const r = await call("file_pick"); if (r?.rel) { if (istKachelDatei(r.rel)) await openKacheln(r.rel); else toast(r.rel, true); } else if (r?.outside) toast(r.outside, true); } catch (e) { fail(e); }
}
let kette: Promise<unknown> = Promise.resolve(); let neueste: KachelView | null = null;
export function kachelOp(op: Record<string, unknown>): Promise<boolean> {
  const id = kc.docId; if (id === null) return Promise.resolve(false);
  kc.pending++;
  const r = kette.then(async () => {
    try { neueste = (await callAny("pixel_apply", { docId: id, op })) as KachelView; return true; }
    catch (e) { fail(e); try { neueste = (await callAny("pixel_view", { docId: id })) as KachelView; } catch { /* weg */ } return false; }
  });
  kette = r;
  return r.finally(() => { kc.pending--; if (kc.pending === 0 && neueste && kc.docId === id) { kc.view = neueste; neueste = null; } });
}
export const selTile = (i: number) => kachelOp({ kind: "tile", i });
export const selCell = (k: number) => kachelOp({ kind: "cell", k });
export const paint = (cells: number[]) => kachelOp({ kind: "paint", cells });
export const pickCell = (k: number) => kachelOp({ kind: "pick", k });
export const setChar = (code: number) => kachelOp({ kind: "char", code });
export const brushColor = (i: number) => kachelOp({ kind: "brush_color", i });
export const bit7 = (on: boolean) => kachelOp({ kind: "bit7", on });
export const brushMode = (mode: string) => kachelOp({ kind: "brush_mode", mode });
export const flipTile = (dir: "h" | "v") => kachelOp({ kind: "flip", dir });
export const rotateTile = () => kachelOp({ kind: "rotate" });
export const rollTile = (dx: number, dy: number) => kachelOp({ kind: "roll", dx, dy });
export const clearTile = () => kachelOp({ kind: "clear" });
export const copyTile = () => kachelOp({ kind: "copy" });
export const pasteTile = () => kachelOp({ kind: "paste" });
export const setSize = (w: number, h: number) => kachelOp({ kind: "size", w, h });
export const askYes = () => kachelOp({ kind: "ask_yes" });
export const askNo = () => kachelOp({ kind: "ask_no" });
export const tileNew = () => kachelOp({ kind: "tile_new" });
export const tileDup = () => kachelOp({ kind: "tile_dup" });
export const tileDelete = () => kachelOp({ kind: "tile_delete" });
export const tileMerge = () => kachelOp({ kind: "tile_merge" });
export const renameTile = (name: string) => kachelOp({ kind: "rename", name });
export const setColorMode = (mode: string) => kachelOp({ kind: "color_mode", mode });
export const tileColor = (i: number) => kachelOp({ kind: "tile_color", i });
export const setBg = (i: number) => kachelOp({ kind: "bg", i });
export const setProp = (bit: number, on: boolean) => kachelOp({ kind: "prop", bit, on });
export const propNew = (name: string) => kachelOp({ kind: "prop_new", name });
export const charsetAdd = (rel: string) => kachelOp({ kind: "charset_add", rel });
export const charsetRemove = (i: number) => kachelOp({ kind: "charset_remove", i });
export const setTab = (i: number) => kachelOp({ kind: "tab", i });
export const kachelUndo = () => kachelOp({ kind: "undo" });
export const kachelRedo = () => kachelOp({ kind: "redo" });
export const saveKacheln = async (): Promise<boolean> => { const ok = await kachelOp({ kind: "save" }); if (ok) toast(kc.view?.status || "OK"); return ok; };
export function answerAsk(what: "save" | "discard" | "cancel"): Promise<void> {
  const then = kc.ask; kc.ask = null;
  return (async () => { if (what === "cancel" || !then) return; if (what === "save") { if (!(await saveKacheln())) return; } else { await closeKacheln(); } await then(); })();
}
