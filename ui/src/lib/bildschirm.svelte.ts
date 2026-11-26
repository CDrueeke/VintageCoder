// SPDX-License-Identifier: GPL-3.0-or-later
// Bildschirm-Editor (E9): eine geöffnete Textbildschirm-Datei (.vcscreen) als Dokument im Kern (`bildschirm_open`; `pixel_apply`/`pixel_view`/`pixel_close` sind allgemein). Die Oberfläche hält nur den
// Anzeigezustand und sendet Absichten nacheinander; ein Zug über mehrere Zellen und getippter Text sind je eine Absicht (ein Rückgängig-Schritt).
import { onLangChange } from "./i18n.svelte";
import { call, callAny, CoreError, type BildschirmView, type TreeNode } from "./api";
import { fail, setMode, setSub, toast, ui } from "./state.svelte";

export const bs = $state({ docId: null as number | null, view: null as BildschirmView | null, pending: 0, ask: null as null | (() => void | Promise<void>), kachel: null as string | null, zerlegenOffen: false, zw: 2, zh: 2 });
onLangChange(async () => { const id = bs.docId; if (id === null) return; try { bs.view = (await callAny("pixel_view", { docId: id })) as BildschirmView; } catch { /* Ansicht bleibt */ } });   // Kern-Texte der Ansicht folgen der Sprache

export const istBildschirmDatei = (rel: string): boolean => /\.vcscreen$/i.test(rel);
export const aktiv = (): boolean => !!bs.view && ui.mode === "grafik" && ui.sub.grafik === "bildschirm";
export function bildschirmDateien(tree: TreeNode[] | undefined): string[] {
  const out: string[] = []; const walk = (ns: TreeNode[]) => { for (const n of ns) { if (n.kind === "dir") walk(n.children ?? []); else if (istBildschirmDatei(n.rel)) out.push(n.rel); } };
  walk(tree ?? []); return out;
}
/** Öffnen; `false`, wenn die Datei kein einfacher Textbildschirm ist (zum Beispiel eine Bank): sie öffnet dann als Text */
export async function openBildschirm(rel: string): Promise<boolean> {
  if (bs.view?.rel === rel) { setMode("grafik"); setSub("grafik", "bildschirm"); return true; }
  if (bs.view?.dirty) { bs.ask = () => { void openBildschirm(rel); }; return true; }
  try {
    const r = await call("bildschirm_open", { rel });
    await closeBildschirm();
    bs.kachel = null; bs.docId = r.docId; bs.view = r.view; setMode("grafik"); setSub("grafik", "bildschirm"); return true;
  } catch (e) {
    if (e instanceof CoreError && e.code === "bildschirm-kachel") {   // Bank mit Kacheln und mehreren Karten: der Karten-Editor; nur wenn auch er sie nicht lesen kann, die Erklärung mit „Als Text öffnen“
      const { openKarten } = await import("./karten.svelte"); if (await openKarten(rel)) { await closeBildschirm(); return true; }
      await closeBildschirm(); bs.kachel = rel; setMode("grafik"); setSub("grafik", "bildschirm"); return true;
    }
    fail(e); return false;
  }
}
export async function closeBildschirm(): Promise<void> {
  bs.kachel = null;
  const id = bs.docId; bs.docId = null; bs.view = null;
  if (id !== null) { try { await call("pixel_close", { docId: id }); } catch { /* Kern schon weg */ } }
}
export async function pickBildschirm(): Promise<void> {
  try { const r = await call("file_pick"); if (r?.rel) { if (istBildschirmDatei(r.rel)) await openBildschirm(r.rel); else toast(r.rel, true); } else if (r?.outside) toast(r.outside, true); } catch (e) { fail(e); }
}
let kette: Promise<unknown> = Promise.resolve(); let neueste: BildschirmView | null = null;
export function bildschirmOp(op: Record<string, unknown>): Promise<boolean> {
  const id = bs.docId; if (id === null) return Promise.resolve(false);
  bs.pending++;
  const r = kette.then(async () => {
    try { neueste = (await callAny("pixel_apply", { docId: id, op })) as BildschirmView; return true; }
    catch (e) { fail(e); try { neueste = (await callAny("pixel_view", { docId: id })) as BildschirmView; } catch { /* weg */ } return false; }
  });
  kette = r;
  return r.finally(() => { bs.pending--; if (bs.pending === 0 && neueste && bs.docId === id) { bs.view = neueste; neueste = null; } });
}
export const setTool = (tool: string) => bildschirmOp({ kind: "tool", tool });
export const setChar = (code: number) => bildschirmOp({ kind: "char", code });
export const setColor = (i: number) => bildschirmOp({ kind: "color", i });
export const setRole = (role: string, i: number) => bildschirmOp({ kind: "role", role, i });
export const setCharset = (rel: string | null) => bildschirmOp({ kind: "charset", rel });
export const stroke = (cells: [number, number][], right = false) => bildschirmOp({ kind: "stroke", cells, right });
export const rectOp = (from: [number, number], to: [number, number], right = false) => bildschirmOp({ kind: "rect", from: [from], to: [to], right });
export const moveCursor = (x: number, y: number) => bildschirmOp({ kind: "cursor", x, y });
export const typeText = (text: string) => bildschirmOp({ kind: "type", text });
export const setNorm = (norm: string) => bildschirmOp({ kind: "norm", norm });
export const zerlegenVorschau = (w: number, h: number) => bildschirmOp({ kind: "zerlegen_vorschau", w, h });
export const zerlegen = (w: number, h: number) => bildschirmOp({ kind: "zerlegen", w, h });
export const bildschirmUndo = () => bildschirmOp({ kind: "undo" });
export const bildschirmRedo = () => bildschirmOp({ kind: "redo" });
export const saveBildschirm = async (): Promise<boolean> => { const ok = await bildschirmOp({ kind: "save" }); if (ok) toast(bs.view?.status || "OK"); return ok; };
export function answerAsk(what: "save" | "discard" | "cancel"): Promise<void> {
  const then = bs.ask; bs.ask = null;
  return (async () => { if (what === "cancel" || !then) return; if (what === "save") { if (!(await saveBildschirm())) return; } else { await closeBildschirm(); } await then(); })();
}
