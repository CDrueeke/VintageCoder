// SPDX-License-Identifier: GPL-3.0-or-later
// Palette-Editor (E7): eine geöffnete Palettendatei (.vcpal) als Dokument im Kern; die Oberfläche hält nur den Anzeigezustand und sendet Absichten (nacheinander).
// Die Befehle `pixel_apply`/`pixel_view`/`pixel_close` sind allgemein (DocStore), nur das Öffnen ist eigen (`palette_open`).
import { onLangChange } from "./i18n.svelte";
import { call, callAny, type PaletteView, type TreeNode } from "./api";
import { fail, setMode, setSub, toast, ui } from "./state.svelte";

export const pal = $state({ docId: null as number | null, view: null as PaletteView | null, pending: 0, /** Rückfrage bei ungespeicherter Datei: danach auszuführen */ ask: null as null | (() => void | Promise<void>) });
onLangChange(async () => { const id = pal.docId; if (id === null) return; try { pal.view = (await callAny("pixel_view", { docId: id })) as PaletteView; } catch { /* Ansicht bleibt */ } });   // Kern-Texte der Ansicht folgen der Sprache

export const istPaletteDatei = (rel: string): boolean => /\.vcpal$/i.test(rel);
export const aktiv = (): boolean => !!pal.view && ui.mode === "grafik" && ui.sub.grafik === "palette";
export function paletteDateien(tree: TreeNode[] | undefined): string[] {
  const out: string[] = []; const walk = (ns: TreeNode[]) => { for (const n of ns) { if (n.kind === "dir") walk(n.children ?? []); else if (istPaletteDatei(n.rel)) out.push(n.rel); } };
  walk(tree ?? []); return out;
}

export async function openPalette(rel: string): Promise<void> {
  if (pal.view?.rel === rel) { setMode("grafik"); setSub("grafik", "palette"); return; }
  if (pal.view?.dirty) { pal.ask = () => openPalette(rel); return; }
  try {
    const r = await call("palette_open", { rel });
    await closePalette();
    pal.docId = r.docId; pal.view = r.view; setMode("grafik"); setSub("grafik", "palette");
  } catch (e) { fail(e); }
}
export async function closePalette(): Promise<void> {
  const id = pal.docId; pal.docId = null; pal.view = null;
  if (id !== null) { try { await call("pixel_close", { docId: id }); } catch { /* Kern schon weg */ } }
}
export async function pickPalette(): Promise<void> {
  try { const r = await call("file_pick"); if (r?.rel) { if (istPaletteDatei(r.rel)) await openPalette(r.rel); else toast(r.rel, true); } else if (r?.outside) toast(r.outside, true); } catch (e) { fail(e); }
}

let kette: Promise<unknown> = Promise.resolve(); let neueste: PaletteView | null = null;
/** Eine Absicht an das Dokument schicken; Antworten werden der Reihe nach angewandt, die Ansicht wechselt, wenn nichts mehr unterwegs ist */
export function paletteOp(op: Record<string, unknown>): Promise<boolean> {
  const id = pal.docId; if (id === null) return Promise.resolve(false);
  pal.pending++;
  const r = kette.then(async () => {
    try { neueste = (await callAny("pixel_apply", { docId: id, op })) as PaletteView; return true; }
    catch (e) { fail(e); try { neueste = (await callAny("pixel_view", { docId: id })) as PaletteView; } catch { /* weg */ } return false; }
  });
  kette = r;
  return r.finally(() => { pal.pending--; if (pal.pending === 0 && neueste && pal.docId === id) { pal.view = neueste; neueste = null; } });
}
export const setSel = (i: number): Promise<boolean> => paletteOp({ kind: "sel", i });
export const setKanal = (k: number, v: number): Promise<boolean> => paletteOp({ kind: "kanal", k, v });
export const setHex = (hex: string): Promise<boolean> => paletteOp({ kind: "hex", hex });
export const fertig = (): Promise<boolean> => paletteOp({ kind: "fertig" });
export const standardPalette = (): Promise<boolean> => paletteOp({ kind: "standard" });
export const paletteUndo = (): Promise<boolean> => paletteOp({ kind: "undo" });
export const paletteRedo = (): Promise<boolean> => paletteOp({ kind: "redo" });
export const savePalette = async (): Promise<boolean> => { const ok = await paletteOp({ kind: "save" }); if (ok) toast(pal.view?.status || "OK"); return ok; };
export function answerAsk(what: "save" | "discard" | "cancel"): Promise<void> {
  const then = pal.ask; pal.ask = null;
  return (async () => { if (what === "cancel" || !then) return; if (what === "save") { if (!(await savePalette())) return; } else { await closePalette(); } await then(); })();
}
