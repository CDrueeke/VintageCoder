// SPDX-License-Identifier: GPL-3.0-or-later
// Sprite-Editor (E3): eine geöffnete Sprite-Datei als Dokument im Kern (`pixel_*`); die Oberfläche hält nur Anzeigezustand (Zoom, Raster, Zellmarke)
// und sendet Absichten. Absichten laufen nacheinander (ein Strich ist ein Vorgang); solange welche unterwegs sind, bleibt die lokale Vorschau stehen.
import { onLangChange } from "./i18n.svelte";
import { call, type PixelView } from "./api";
import { fail, setMode, setSub, toast, ui } from "./state.svelte";
import type { TreeNode } from "./api";

export const sp = $state({
  docId: null as number | null, view: null as PixelView | null,
  /** Rückfrage bei ungespeicherter Datei: danach auszuführen */ ask: null as null | (() => void | Promise<void>),
  /** Zoom als ganze Zahl (Bildpunkte je Zelle, 100 % = 1); `null` = Einpassen */ zoom: null as number | null, grid: true,
  /** Zelle unter Maus oder Tastaturcursor */ cursor: null as [number, number] | null, pending: 0, onion: false, tab: "frames" as "frames" | "anims",
  /** Markierte Bilder der Bilderliste (aufsteigend); das bearbeitete Bild ist `view.fi` */ marked: [] as number[], anchor: 0, /** Fenster „Animation aus Auswahl“ */ animDialog: false, /** Große Vorschau der Animation */ previewDialog: false, /** Vorschau der neuen Animation sofort abspielen */ autoplay: false,
});
export const ZOOM_MIN = 1, ZOOM_MAX = 32;
/** Höchster Zoom für das geöffnete Bild: 3200 % für Sprites; sehr kleine Bilder (Zeichen mit 8 × 8 Punkten) dürfen größer, damit die Fläche gefüllt wird */
export const zoomMax = (): number => { const v = sp.view; return v ? Math.max(ZOOM_MAX, Math.floor(640 / Math.max(v.width * (v.aspect || 1), v.height))) : ZOOM_MAX; };

/** Dateien des Projekts, die der Sprite-Editor öffnet (Sprites und Player/Missile-Grafik) */
export const istSpriteDatei = (rel: string): boolean => /\.(vcsprite|vcpmg)$/i.test(rel);
/** Zeichensatz-Dateien öffnen im Zeichensatz-Editor (dasselbe Pixeldokument im Kern, andere Ansicht) */
export const istZeichensatzDatei = (rel: string): boolean => /\.vcchar$/i.test(rel);
export const istPixelDatei = (rel: string): boolean => istSpriteDatei(rel) || istZeichensatzDatei(rel);
export function zeichensatzDateien(tree: TreeNode[] | undefined): string[] {
  const out: string[] = []; const walk = (ns: TreeNode[]) => { for (const n of ns) { if (n.kind === "dir") walk(n.children ?? []); else if (istZeichensatzDatei(n.rel)) out.push(n.rel); } };
  walk(tree ?? []); return out;
}
export function spriteDateien(tree: TreeNode[] | undefined): string[] {
  const out: string[] = []; const walk = (ns: TreeNode[]) => { for (const n of ns) { if (n.kind === "dir") walk(n.children ?? []); else if (istSpriteDatei(n.rel)) out.push(n.rel); } };
  walk(tree ?? []); return out;
}

export function askThen(then: () => void | Promise<void>): void { if (sp.view?.dirty) sp.ask = then; else void then(); }

/** Sprite-Datei des Projekts öffnen; eine andere zuvor geöffnete wird geschlossen (bei Änderungen erst Rückfrage) */
export async function openSprite(rel: string): Promise<void> {
  if (sp.view?.rel === rel) { setMode("grafik"); setSub("grafik", istZeichensatzDatei(rel) ? "zeichensatz" : "sprites"); return; }
  if (sp.view?.dirty) { sp.ask = () => openSprite(rel); return; }
  try {
    const r = await call("pixel_open", { rel });
    await closeSprite();
    sp.docId = r.docId; sp.view = r.view; sp.zoom = null; sp.cursor = null; sp.marked = [];
    setMode("grafik"); setSub("grafik", istZeichensatzDatei(rel) ? "zeichensatz" : "sprites");
  } catch (e) { fail(e); }
}
export async function pickSprite(zeichensatz = false): Promise<void> {
  try { const r = await call("file_pick"); if (r?.rel) { if (zeichensatz ? istZeichensatzDatei(r.rel) : istSpriteDatei(r.rel)) await openSprite(r.rel); else toast(r.rel, true); } else if (r?.outside) toast(r.outside, true); } catch (e) { fail(e); }
}
export async function closeSprite(): Promise<void> {
  const id = sp.docId; sp.docId = null; sp.view = null; sp.cursor = null;
  if (id !== null) { try { await call("pixel_close", { docId: id }); } catch { /* Kern schon weg */ } }
}

let kette: Promise<unknown> = Promise.resolve(); let neueste: PixelView | null = null;
onLangChange(async () => { const id = sp.docId; if (id === null) return; try { sp.view = await call("pixel_view", { docId: id }); } catch { /* Ansicht bleibt */ } });   // Farbrollen und Wertnamen kommen vom Kern
/** Eine Absicht an das Dokument schicken; die Antworten werden der Reihe nach angewandt, die Ansicht wechselt erst, wenn nichts mehr unterwegs ist */
export function spriteOp(op: Record<string, unknown>): Promise<boolean> {
  const id = sp.docId; if (id === null) return Promise.resolve(false);
  sp.pending++;
  const r = kette.then(async () => {
    try { neueste = await call("pixel_apply", { docId: id, op }); return true; }
    catch (e) { fail(e); try { neueste = await call("pixel_view", { docId: id }); } catch { /* weg */ } return false; }
  });
  kette = r;
  return r.finally(() => { sp.pending--; if (sp.pending === 0 && neueste && sp.docId === id) { if (sp.view && sp.view.frames !== neueste.frames) sp.marked = []; sp.view = neueste; const n = neueste.note; if (n) toast(n.text, !n.ok); neueste = null; } });
}
export const setTool = (tool: string): Promise<boolean> => spriteOp({ kind: "tool", tool });
export const setValue = (v: number): Promise<boolean> => spriteOp({ kind: "value", v });
export const setColor = (key: string, index: number): Promise<boolean> => spriteOp({ kind: "color", key, index });
export const setNorm = (norm: string): Promise<boolean> => spriteOp({ kind: "norm", norm });
export const stroke = (points: [number, number][], erase: boolean): Promise<boolean> => spriteOp({ kind: "stroke", points, erase });
export const pick = (x: number, y: number): Promise<boolean> => spriteOp({ kind: "pick", x, y });
export const fill = (x: number, y: number, erase: boolean): Promise<boolean> => spriteOp({ kind: "fill", x, y, erase });
export const shape = (tool: "line" | "rect", a: [number, number], b: [number, number], erase: boolean): Promise<boolean> => spriteOp({ kind: "shape", tool, a, b, erase });
export const selectRect = (a: [number, number], b: [number, number]): Promise<boolean> => spriteOp({ kind: "select", a, b });
export const selectAll = (): Promise<boolean> => spriteOp({ kind: "select_all" });
export const selectNone = (): Promise<boolean> => spriteOp({ kind: "select_none" });
export const copySel = (cut: boolean): Promise<boolean> => spriteOp({ kind: cut ? "cut" : "copy" });
export const pasteAt = (): Promise<boolean> => spriteOp(sp.cursor ? { kind: "paste", x: sp.cursor[0], y: sp.cursor[1] } : { kind: "paste" });
export const clearSel = (): Promise<boolean> => spriteOp({ kind: "clear_selection" });
export const flip = (dir: "h" | "v"): Promise<boolean> => spriteOp({ kind: "flip", dir });
export const shift = (dir: "l" | "r" | "u" | "d"): Promise<boolean> => spriteOp({ kind: "shift", dir });
export const loadTemplate = (ref: string): Promise<boolean> => spriteOp({ kind: "vorlage", ref });
export const resetChar = (i: number): Promise<boolean> => spriteOp({ kind: "zeichen_original", i });
/** Bereiche des Zeichensatzes (Bildschirmcodes: Atari nach ANTIC-Reihenfolge, sonst Commodore-Art) */
export function zeichenBereiche(target: string, frames: number): { id: string; label: string; von: number; bis: number }[] {
  const atari = target === "atari8"; const k = (id: string, label: string, von: number, bis: number) => ({ id, label, von, bis: Math.min(bis, frames - 1) });
  return [k("all", "zeichensatz-bereich-alle", 0, frames - 1), atari ? k("buchstaben", "zeichensatz-bereich-buchstaben", 33, 58) : k("buchstaben", "zeichensatz-bereich-buchstaben", 1, 26),
    atari ? k("ziffern", "zeichensatz-bereich-ziffern", 16, 25) : k("ziffern", "zeichensatz-bereich-ziffern", 48, 57), atari ? k("zeichen", "zeichensatz-bereich-zeichen", 0, 15) : k("zeichen", "zeichensatz-bereich-zeichen", 32, 47)].filter((b) => b.von <= b.bis);
}
export const clearImage = (): Promise<boolean> => spriteOp({ kind: "clear" });
export const selectFrame = (i: number): Promise<boolean> => spriteOp({ kind: "frame", i });
/** Bild in der Liste anklicken: einfach = nur dieses bearbeiten und markieren, Umschalt = Bereich ab dem letzten Klick markieren, Strg/Befehl = einzelnes Bild zur Markierung hinzufügen oder wegnehmen */
export function markFrame(i: number, mod: { shiftKey?: boolean; ctrlKey?: boolean; metaKey?: boolean }): Promise<boolean> {
  if (mod.shiftKey) { const a = Math.min(sp.anchor, i), b = Math.max(sp.anchor, i); sp.marked = Array.from({ length: b - a + 1 }, (_, k) => a + k); }
  else if (mod.ctrlKey || mod.metaKey) { const had = sp.marked.includes(i); const base = sp.marked.length ? sp.marked : (sp.view ? [sp.view.fi] : []); sp.marked = had ? base.filter((x) => x !== i) : [...new Set([...base, i])].sort((x, y) => x - y); sp.anchor = i; if (had) return Promise.resolve(false); }
  else { sp.marked = [i]; sp.anchor = i; }
  return selectFrame(i);
}
export const markAll = (): void => { const n = sp.view?.frames ?? 0; sp.marked = Array.from({ length: n }, (_, k) => k); };
export const markNone = (): void => { sp.marked = []; };
/** Animation aus den markierten Bildern (Reihenfolge der Bilderliste); danach Reiter „Animation“ und Vorschau läuft */
export async function animFromMarked(name: string, mode: string, fps: number): Promise<boolean> {
  const delay = Math.max(1, Math.min(255, Math.round(50 / Math.max(1, Math.min(50, fps)))));
  const ok = await spriteOp({ kind: "anim_from_frames", frames: sp.marked, name, mode, delay });
  if (ok) { sp.tab = "anims"; sp.autoplay = true; sp.animDialog = false; }
  return ok;
}
export const stepFrame = (d: number): Promise<boolean> => { const v = sp.view; if (!v) return Promise.resolve(false); const i = Math.max(0, Math.min(v.frames - 1, v.fi + d)); return i === v.fi ? Promise.resolve(false) : selectFrame(i); };
export const frameNew = (): Promise<boolean> => spriteOp({ kind: "frame_new" });
/** Bild `von` an die Stelle `nach` setzen (Bilderliste umordnen) */
export const frameMove = (von: number, nach: number): Promise<boolean> => spriteOp({ kind: "frame_move", from: von, to: nach });
export const frameDup = (): Promise<boolean> => spriteOp({ kind: "frame_dup" });
export const frameDelete = (): Promise<boolean> => spriteOp({ kind: "frame_delete" });
export const setFrameMode = (multicolor: boolean): Promise<boolean> => spriteOp({ kind: "mode", multicolor });
export const setFlag = (flag: "x2" | "y2" | "bg", on: boolean): Promise<boolean> => spriteOp({ kind: "flag", flag, on });
export const animNew = (): Promise<boolean> => spriteOp({ kind: "anim_new" });
export const animDelete = (): Promise<boolean> => spriteOp({ kind: "anim_delete" });
export const animSelect = (i: number): Promise<boolean> => spriteOp({ kind: "anim_select", i });
export const animRename = (name: string): Promise<boolean> => spriteOp({ kind: "anim_rename", name });
export const animMode = (mode: string): Promise<boolean> => spriteOp({ kind: "anim_mode", mode });
export const animAddStep = (frame: number): Promise<boolean> => spriteOp({ kind: "anim_add_step", frame });
export const animDelStep = (i: number): Promise<boolean> => spriteOp({ kind: "anim_del_step", i });
export const animMove = (i: number, dir: number): Promise<boolean> => spriteOp({ kind: "anim_move", i, dir });
export const animMoveTo = (from: number, to: number): Promise<boolean> => spriteOp({ kind: "anim_move_to", from, to });
export const animDelayAll = (delay: number): Promise<boolean> => spriteOp({ kind: "anim_delay_all", delay });
export const animDelay = (i: number, delay: number): Promise<boolean> => spriteOp({ kind: "anim_delay", i, delay });
/** PNG-Bild in das gewählte Bild laden (Auswahldialog); der Kern speichert die Datei vorher */
export async function loadPng(): Promise<void> { try { const p = await call("png_pick"); if (p) await spriteOp({ kind: "png", path: p }); } catch (e) { fail(e); } }
/** Welcher Schritt des ausgeschriebenen Ablaufs läuft nach `t` Sekunden (50 Bilder je Sekunde)? Wie `Anim::step_at` im Kern */
export function stepAt(expanded: [number, number][], mode: string, t: number): number {
  const total = expanded.reduce((a, s) => a + s[1], 0); if (!expanded.length || total === 0) return 0;
  const tick0 = Math.max(0, Math.floor(t * 50)); const tick = mode === "once" ? Math.min(tick0, total - 1) : tick0 % total;
  let acc = 0; for (let i = 0; i < expanded.length; i++) { acc += expanded[i]![1]; if (tick < acc) return i; } return expanded.length - 1;
}
export const setPmgSize = (i: number, size: number): Promise<boolean> => spriteOp({ kind: "pmg_size", i, size });
export const setPmgPrior = (prior: number): Promise<boolean> => spriteOp({ kind: "pmg_prior", prior });
export const spriteUndo = (): Promise<boolean> => (sp.view?.canUndo ? spriteOp({ kind: "undo" }) : Promise.resolve(false));
export const spriteRedo = (): Promise<boolean> => (sp.view?.canRedo ? spriteOp({ kind: "redo" }) : Promise.resolve(false));
export const saveSprite = (): Promise<boolean> => (sp.view ? spriteOp({ kind: "save" }) : Promise.resolve(true));

export const zoomStep = (d: number, fit: number): void => { const z = sp.zoom ?? fit; sp.zoom = Math.max(ZOOM_MIN, Math.min(zoomMax(), Math.round(z) + d)); };
export const zoomFit = (): void => { sp.zoom = null; };

/** Antwort auf die Rückfrage bei ungespeicherter Sprite-Datei */
export async function answerAsk(what: "save" | "discard" | "cancel"): Promise<void> {
  const then = sp.ask; sp.ask = null; if (!then || what === "cancel") return;
  if (what === "save" && !(await saveSprite())) return;
  if (what === "discard") await closeSprite();
  await then();
}
export const aktiv = (): boolean => !!sp.view && ui.mode === "grafik" && (ui.sub.grafik === "sprites" || ui.sub.grafik === "zeichensatz");
