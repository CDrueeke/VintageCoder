// SPDX-License-Identifier: GPL-3.0-or-later
// Tracker (E5): ein geöffneter Song (.vcsong, .vcplay) als Dokument im Kern (`tracker_open`; `pixel_apply`/`pixel_view`/`pixel_close` sind allgemein). Die Oberfläche hält nur den Anzeigezustand und sendet
// Absichten nacheinander. Die Wiedergabe folgt mit E5.2.
import { onLangChange } from "./i18n.svelte";
import { call, callAny, CoreError, type AudioStand, type SongQuelle, type TrackerView, type TreeNode } from "./api";
import { reloadTree } from "./editor.svelte";
import { errorText } from "./i18n.svelte";
import { fail, setMode, toast, ui } from "./state.svelte";

export const tk = $state({ docId: null as number | null, view: null as TrackerView | null, pending: 0, ask: null as null | (() => void | Promise<void>) });
onLangChange(async () => { const id = tk.docId; if (id === null) return; try { tk.view = (await callAny("pixel_view", { docId: id })) as TrackerView; } catch { /* Ansicht bleibt */ } });   // Kern-Texte der Ansicht folgen der Sprache

export const istSongDatei = (rel: string): boolean => /\.(vcsong|vcplay)$/i.test(rel);
export const aktiv = (): boolean => !!tk.view && ui.mode === "musik";
export function songDateien(tree: TreeNode[] | undefined): string[] {
  const out: string[] = []; const walk = (ns: TreeNode[]) => { for (const n of ns) { if (n.kind === "dir") walk(n.children ?? []); else if (istSongDatei(n.rel)) out.push(n.rel); } };
  walk(tree ?? []); return out;
}
export async function openSong(rel: string): Promise<void> {
  if (tk.view?.rel === rel) { setMode("musik"); return; }
  if (tk.view?.dirty) { tk.ask = () => openSong(rel); return; }
  try {
    const r = await call("tracker_open", { rel });
    await closeSong();
    tk.docId = r.docId; tk.view = r.view; sicht.versteckt = []; setMode("musik");
  } catch (e) { fail(e); }
}
export async function closeSong(): Promise<void> {
  const id = tk.docId; tk.docId = null; tk.view = null;
  if (id !== null) { try { await call("pixel_close", { docId: id }); } catch { /* Kern schon weg */ } }
}
export async function pickSong(): Promise<void> {
  try { const r = await call("file_pick"); if (r?.rel) { if (istSongDatei(r.rel)) await openSong(r.rel); else toast(r.rel, true); } else if (r?.outside) toast(r.outside, true); } catch (e) { fail(e); }
}
let kette: Promise<unknown> = Promise.resolve(); let neueste: TrackerView | null = null;
export function trackerOp(op: Record<string, unknown>): Promise<boolean> {
  const id = tk.docId; if (id === null) return Promise.resolve(false);
  tk.pending++;
  const r = kette.then(async () => {
    try { neueste = (await callAny("pixel_apply", { docId: id, op })) as TrackerView; return true; }
    catch (e) { fail(e); try { neueste = (await callAny("pixel_view", { docId: id })) as TrackerView; } catch { /* weg */ } return false; }
  });
  kette = r;
  return r.finally(() => { tk.pending--; if (tk.pending === 0 && neueste && tk.docId === id) { tk.view = neueste; neueste = null; } });
}
export const op = trackerOp;
export const saveSong = async (): Promise<boolean> => { const ok = await trackerOp({ kind: "save" }); if (ok) toast(tk.view?.status || "OK"); return ok; };
export const songUndo = () => trackerOp({ kind: "undo" });
export const songRedo = () => trackerOp({ kind: "redo" });
export function answerAsk(what: "save" | "discard" | "cancel"): Promise<void> {
  const then = tk.ask; tk.ask = null;
  return (async () => { if (what === "cancel" || !then) return; if (what === "save") { if (!(await saveSong())) return; } else { await closeSong(); } await then(); })();
}
/** Klaviertasten nach der physischen Taste (Position wie bei US-QWERTY, unabhängig vom Tastaturlayout): untere Reihe Z S X D C V G B H N J M, obere Q 2 W 3 E R 5 T 6 Y 7 U I → Halbtöne über C */
export const KLAVIER: Record<string, number> = { KeyZ: 0, KeyS: 1, KeyX: 2, KeyD: 3, KeyC: 4, KeyV: 5, KeyG: 6, KeyB: 7, KeyH: 8, KeyN: 9, KeyJ: 10, KeyM: 11, KeyQ: 12, Digit2: 13, KeyW: 14, Digit3: 15, KeyE: 16, KeyR: 17, Digit5: 18, KeyT: 19, Digit6: 20, KeyY: 21, Digit7: 22, KeyU: 23, KeyI: 24 };

/** Wiedergabe (E5.2): Stand aus `audio_poll` (nur solange der Song spielt), Mitscrollen ein/aus */
export const wg = $state({ stand: null as AudioStand | null, folgen: true });
let timer: ReturnType<typeof setInterval> | null = null; let laeuft = false;
async function tick(): Promise<void> {
  if (laeuft) return; laeuft = true;
  try {
    const s = await call("audio_poll"); wg.stand = s;
    const v = tk.view; if (!v || (!s.playing && !v.spielt)) { wiedergabeAbfrage(false); return; }
    if (!s.playing && v.spielt) { await trackerOp({ kind: "refresh" }); return; }
    if (s.playing && wg.folgen && v.kind === "song" && v.spielModus === "song" && s.ord !== v.ordSel && s.ord < v.order.length) await trackerOp({ kind: "ord_select", i: s.ord });
  } catch { /* App schließt */ } finally { laeuft = false; }
}
/** Abfrage ein-/ausschalten je nachdem, ob der Song spielt (aus einem Effekt der Ansicht aufgerufen) */
export function wiedergabeAbfrage(an: boolean): void {
  if (an && !timer) timer = setInterval(() => void tick(), 60);
  if (!an && timer) { clearInterval(timer); timer = null; wg.stand = null; }
}
export const spielen = (mode: "song" | "pattern") => trackerOp({ kind: "play", mode });
export const stoppen = () => trackerOp({ kind: "stop" });
export const umschaltenSpielen = (mode: "song" | "pattern") => (tk.view?.spielt ? stoppen() : spielen(mode));

/** Ausgeblendete Stimmen (nur Anzeige, sie klingen weiter); der Cursor springt nie auf eine ausgeblendete Stimme */
export const sicht = $state({ versteckt: [] as number[] });
export function stimmeUmschalten(c: number): void {
  const v = tk.view; if (!v) return;
  if (sicht.versteckt.includes(c)) { sicht.versteckt = sicht.versteckt.filter((x) => x !== c); return; }
  if (v.channels - sicht.versteckt.length <= 1) return;   // mindestens eine Stimme bleibt sichtbar
  sicht.versteckt = [...sicht.versteckt, c];
  if (v.cursor[1] === c) { const frei = Array.from({ length: v.channels }, (_, i) => i).find((i) => !sicht.versteckt.includes(i)); if (frei !== undefined) void trackerOp({ kind: "cursor", r: v.cursor[0], c: frei }); }
}
export const alleEinblenden = () => { sicht.versteckt = []; };
/** nächste sichtbare Stimme in Richtung `d` (±1) ab `c`; gibt den Abstand zurück (0 = keine) */
export function naechsteStimme(c: number, d: number, n: number): number { let x = c + d; while (x >= 0 && x < n && sicht.versteckt.includes(x)) x += d; return x < 0 || x >= n ? 0 : x - c; }

/** Klaviatur: kurz klingende Note (Halbtöne über der Oktave) als Anzeige; Klick spielt vor, trägt nichts ein */
export const kl = $state({ halbton: null as number | null });
let klTimer: ReturnType<typeof setTimeout> | null = null;
export function klingt(halbton: number): void { kl.halbton = halbton; if (klTimer) clearTimeout(klTimer); klTimer = setTimeout(() => { kl.halbton = null; }, 280); }
export const vorspielen = (halbton: number) => { klingt(halbton); return trackerOp({ kind: "vorschau", halbton }); };

/** Dialog „Song aus anderem System übernehmen“ und „RMT-Modul importieren“ */
export const imp = $state({ open: false, rmt: false, q: null as SongQuelle | null, name: "", ziel: "", ziele: [] as { id: string; name: string }[], subtune: 1, sekunden: 30, busy: false, fehler: "", warnungen: [] as string[] });
export async function importOeffnen(rmt: boolean): Promise<void> {
  const v = tk.view; if (!v) return;
  try {
    const q = await call("song_quelle_pick", { rmt }); if (!q) return;
    const profile = (await call("media_profiles", { kinds: ["song"] })).map((p) => ({ id: p.id, name: p.name }));
    const wunsch = rmt || v.pokey ? "atari.pokey.song" : "c64.sid.song";
    Object.assign(imp, { open: true, rmt, q, name: q.name ?? "", ziele: profile, ziel: profile.some((p) => p.id === wunsch) ? wunsch : (profile[0]?.id ?? ""), subtune: Math.max(1, q.sid?.start ?? 1), sekunden: 30, busy: false, fehler: q.error ?? "", warnungen: [] });
  } catch (e) { fail(e); }
}
export async function importAusfuehren(): Promise<void> {
  const v = tk.view; if (!v || !imp.q?.path || imp.busy) return;
  imp.busy = true; imp.fehler = "";
  try {
    const ordner = v.rel.includes("/") ? v.rel.slice(0, v.rel.lastIndexOf("/")) : "music";
    const r = await call("song_import_run", { path: imp.q.path, rmt: imp.rmt, name: imp.name.trim(), ordner, ziel: imp.ziel, subtune: imp.q.sid ? imp.subtune : 0, sekunden: imp.sekunden });
    imp.open = false; await reloadTree(); toast(r.warnings.length ? `${r.rel}: ${r.warnings.join(" · ")}` : r.rel); await openSong(r.rel);
  } catch (e) { imp.fehler = e instanceof CoreError ? errorText(e.code, e.args) : String(e); }
  finally { imp.busy = false; }
}
