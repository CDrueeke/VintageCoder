// SPDX-License-Identifier: GPL-3.0-or-later
// Karten-Editor (E10): eine geöffnete Bank-Datei (.vcscreen mit Kachelsatz und mehreren Karten) als Dokument im Kern (`karten_open`; `pixel_apply`/`pixel_view`/`pixel_close` sind allgemein). Die Oberfläche hält nur
// den Anzeigezustand und sendet Absichten nacheinander; ein Zug mit der Maus (mehrere Zellen) ist eine Absicht (ein Rückgängig-Schritt).
import { onLangChange } from "./i18n.svelte";
import { call, callAny, type KartenView } from "./api";
import { fail, setMode, setSub, toast, ui } from "./state.svelte";

export const kt = $state({ docId: null as number | null, view: null as KartenView | null, pending: 0, ask: null as null | (() => void | Promise<void>) });
onLangChange(async () => { const id = kt.docId; if (id === null) return; try { kt.view = (await callAny("pixel_view", { docId: id })) as KartenView; } catch { /* Ansicht bleibt */ } });   // Kern-Texte der Ansicht folgen der Sprache

export const aktiv = (): boolean => !!kt.view && ui.mode === "grafik" && ui.sub.grafik === "karten";
/** Öffnen; `false`, wenn der Kern die Datei nicht als Karten-Bank lesen kann (Fehlermeldung ist dann schon gezeigt) */
export async function openKarten(rel: string): Promise<boolean> {
  if (kt.view?.rel === rel) { setMode("grafik"); setSub("grafik", "karten"); return true; }
  if (kt.view?.dirty) { kt.ask = () => { void openKarten(rel); }; return true; }
  try {
    const r = await call("karten_open", { rel });
    await closeKarten();
    kt.docId = r.docId; kt.view = r.view; setMode("grafik"); setSub("grafik", "karten"); return true;
  } catch (e) { fail(e); return false; }
}
export async function closeKarten(): Promise<void> {
  const id = kt.docId; kt.docId = null; kt.view = null;
  if (id !== null) { try { await call("pixel_close", { docId: id }); } catch { /* Kern schon weg */ } }
}
let kette: Promise<unknown> = Promise.resolve(); let neueste: KartenView | null = null;
export function kartenOp(op: Record<string, unknown>): Promise<boolean> {
  const id = kt.docId; if (id === null) return Promise.resolve(false);
  kt.pending++;
  const r = kette.then(async () => {
    try { neueste = (await callAny("pixel_apply", { docId: id, op })) as KartenView; return true; }
    catch (e) { fail(e); try { neueste = (await callAny("pixel_view", { docId: id })) as KartenView; } catch { /* weg */ } return false; }
  });
  kette = r;
  return r.finally(() => { kt.pending--; if (kt.pending === 0 && neueste && kt.docId === id) { kt.view = neueste; neueste = null; } });
}
export const selScreen = (i: number) => kartenOp({ kind: "screen", i });
export const setMode2 = (mode: "tile" | "cell") => kartenOp({ kind: "mode", mode });
export const setTool = (tool: string) => kartenOp({ kind: "tool", tool });
export const setFilled = (on: boolean) => kartenOp({ kind: "filled", on });
export const selTile = (i: number) => kartenOp({ kind: "tile", i });
export const selChar = (code: number) => kartenOp({ kind: "char", code });
export const setTab = (i: number) => kartenOp({ kind: "tab", i });
export const brushColor = (i: number) => kartenOp({ kind: "brush_color", i });
export const setBit7 = (on: boolean) => kartenOp({ kind: "bit7", on });
export const setNorm = (norm: string) => kartenOp({ kind: "norm", norm });
export const setLayers = (tiles?: boolean, cells?: boolean) => kartenOp({ kind: "layers", tiles, cells });
export const showProp = (bit: number | null) => kartenOp({ kind: "prop", bit });
export const pointer = (pts: [number, number][], right = false) => kartenOp({ kind: "pointer", pts, right });
export const typeText = (text: string) => kartenOp({ kind: "text", text });
export const textBack = () => kartenOp({ kind: "text_back" });
export const textNewline = () => kartenOp({ kind: "text_newline" });
export const selectionCopy = () => kartenOp({ kind: "selection_copy" });
export const selectionClear = () => kartenOp({ kind: "selection_clear" });
export const selectionMove = (dx: number, dy: number) => kartenOp({ kind: "selection_move", dx, dy });
export const selectionNone = () => kartenOp({ kind: "selection_none" });
export const tileFromSelection = (x: number, y: number) => kartenOp({ kind: "tile_from_selection", x, y });
export const askName = (name: string) => kartenOp({ kind: "ask_name", name });
export const askReplace = (on: boolean) => kartenOp({ kind: "ask_replace", on });
export const askYes = () => kartenOp({ kind: "ask_yes" });
export const askNo = () => kartenOp({ kind: "ask_no" });
export const screenNew = () => kartenOp({ kind: "screen_new" });
export const screenDup = () => kartenOp({ kind: "screen_dup" });
export const screenDelete = () => kartenOp({ kind: "screen_delete" });
export const screenMove = (d: number) => kartenOp({ kind: "screen_move", d });
export const screenOrder = (from: number, to: number) => kartenOp({ kind: "screen_order", from, to });
export const screenRename = (name: string, first = true) => kartenOp({ kind: "screen_rename", name, first });
export const screenSize = (w: number, h: number, ox: number, oy: number, first = true) => kartenOp({ kind: "screen_size", w, h, ox, oy, first });
export const screenColor = (role: string, i: number) => kartenOp({ kind: "screen_color", role, i });
export const setTileSet = (rel: string) => kartenOp({ kind: "set", rel });
export const kartenUndo = () => kartenOp({ kind: "undo" });
export const kartenRedo = () => kartenOp({ kind: "redo" });
export const saveKarten = async (): Promise<boolean> => { const ok = await kartenOp({ kind: "save" }); if (ok) toast(kt.view?.status || "OK"); return ok; };
export function answerAsk(what: "save" | "discard" | "cancel"): Promise<void> {
  const then = kt.ask; kt.ask = null;
  return (async () => { if (what === "cancel" || !then) return; if (what === "save") { if (!(await saveKarten())) return; } else { await closeKarten(); } await then(); })();
}
