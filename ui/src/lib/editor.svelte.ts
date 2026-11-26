// SPDX-License-Identifier: GPL-3.0-or-later
// Zustand des Code-Editors (E1.1): offene Reiter, Text und Verlauf je Reiter als CodeMirror-Zustand, Speichern, Schließen, Dateiverwaltung im Projektbaum.
// Dokumentmodell: Text und Verlauf (Rückgängig) liegen im EditorState der Oberfläche; Rust liest und schreibt die Datei und liefert Färbung und Faltung.
import { closeDisk, dk, openDisk, saveDisk } from "./disk.svelte";
import { closeSong, istSongDatei, openSong, saveSong, tk } from "./tracker.svelte";
import { closeBildschirm, istBildschirmDatei, openBildschirm, bs, saveBildschirm } from "./bildschirm.svelte";
import { closeKacheln, istKachelDatei, kc, openKacheln, saveKacheln } from "./kachel.svelte";
import { closeKarten, kt, saveKarten } from "./karten.svelte";
import { closePalette, istPaletteDatei, openPalette, pal, savePalette } from "./palette.svelte";
import { closeSprite, istSpriteDatei, istZeichensatzDatei, openSprite, saveSprite, sp } from "./sprite.svelte";
import { EditorState, Text } from "@codemirror/state";
import { tick } from "svelte";
import { EditorView } from "@codemirror/view";
import { activateIn, isSplit, moveTo, removeTab, toggleSplit, toOtherHalf, type GState, type Group } from "./groups";
import { call, CoreError, type BasicInfo, type FileMarks, type MarkEntry, type Eol, type TextLang, type TreeNode, type TreeResult } from "./api";
import type { FormatHooks } from "./cm-format";
import { hexStattExtern, openHex } from "./daten.svelte";
import { extensions, jumpBookmark, langCompartment, langExtension, setDebugLine, marksEffect, setCycles, wrapCompartment, wrapExtension, readMarks, setColors, setExcluded, setFolds, toggleLineMark } from "./cm";
import { setBasicInfo } from "./basicinfo.svelte";
import { onLangChange, t } from "./i18n.svelte";
import { bindHost, lspChanged, lspClose, lspHas, lspOpen, lspReset, startLsp } from "./lsp.svelte";
import { fail, featureOn, setMode, toast, ui } from "./state.svelte";
import { fragen } from "./rueckfrage.svelte";
import { resetTests, startTests } from "./tests.svelte";
import { resetBau, startBau } from "./bau.svelte";
import { bindSearchTexts, resetSearch, startSuche } from "./suche.svelte";

export interface Tab { group: Group; eolDirty?: boolean; rel: string; lang: TextLang; eol: Eol; dirty: boolean; scratch?: "text" | "note"; title?: string; conflict?: boolean }
export interface Prompt { art?: string; title: string; label: string; value: string; ok: string; validate?: (v: string) => string | null; onok: (v: string, art?: string) => void | Promise<void>; oncancel?: () => void; arts?: { id: string; label: string; ext: string }[] }

export const ed = $state({
  tabs: [] as Tab[],
  active: null as string | null,
  cursor: { line: 1, col: 1, lines: 1, selected: 0 },
  closing: null as string | null,   // Reiter mit ungespeicherten Änderungen, der geschlossen werden soll (Nachfrage)
  prompt: null as Prompt | null,
  leaving: null as null | { names: string; items: { name: string; scratch: boolean }[] },
  fontSize: 13,
  wrap: true,                        // Zeilenumbruch (Standard an)
  prettyLine: true,                  // Zeile beim Verlassen formatieren
  dock: null as null | "problems" | "output" | "tests" | "search" | "outline",   // Ablage unter dem Editor (Probleme, Suche, Gliederung)
  nonces: [0, 0] as [number, number], // zählt je Hälfte hoch, wenn der Editor den Zustand seines aktiven Reiters neu laden muss
  activeBy: [null, null] as [string | null, string | null],   // aktiver Reiter je Hälfte (geteilte Ansicht); `active` ist der der fokussierten Hälfte
  focus: 0 as Group,                 // fokussierte Hälfte
  dragRel: null as string | null,    // Reiter, der gerade gezogen wird (zeigt die Ablagezonen der geteilten Ansicht)
});

/** Haken für die Sitzung (Modul `sitzung.svelte.ts` setzt sie; so bleibt die Abhängigkeit einseitig) */
export const sessionHooks = { scratch: (): void => {}, discard: (): void => {}, flush: (): void => {} };

const states = new Map<string, EditorState>();
const saved = new Map<string, Text>();
/** Ein Editorfenster je Hälfte; `view` ist das der fokussierten Hälfte */
const views: [EditorView | null, EditorView | null] = [null, null];
let view: EditorView | null = null;
export function attachView(g: Group, v: EditorView | null): void { views[g] = v; view = views[ed.focus] ?? views[0] ?? views[1]; }
export function getView(): EditorView | null { return views[ed.focus] ?? view; }
/** Das Fenster, das den Zustand des Reiters gerade zeigt (in welcher Hälfte auch immer) */
function viewOf(rel: string): EditorView | null { const st = states.get(rel); return st ? (views.find((v) => v && v.state === st) ?? null) : null; }
/** Nachladen anstoßen: für die Hälfte, die den Reiter zeigt (ohne Angabe für beide) */
function bump(rel?: string): void { const g = rel ? tabOf(rel)?.group : undefined; if (g === undefined) { ed.nonces[0]++; ed.nonces[1]++; } else ed.nonces[g]++; }
export function stateOf(rel: string): EditorState | undefined { return states.get(rel); }
const tabOf = (rel: string): Tab | undefined => ed.tabs.find((x) => x.rel === rel);

/** Reiter, dessen Zustand `st` ist (so findet ein Fenster der unfokussierten Hälfte seinen Reiter); sonst der aktive */
const relOfState = (st?: EditorState): string | null => { if (st) for (const [rel, s] of states) if (s === st) return rel; return ed.active; };
export const lspHooks = {
  rel: (st?: EditorState): string | null => { const r = relOfState(st); return r && lspHas(r) && tabOf(r)?.lang === "asm" ? r : null; },
  target: (st?: EditorState): { rel: string; lang: TextLang } | null => { const r = relOfState(st); const tb = r ? tabOf(r) : undefined; return tb && (tb.lang === "asm" || tb.lang === "basic") ? { rel: tb.scratch ? "" : tb.rel, lang: tb.lang } : null; },
  jump: (rel: string, line: number, col: number): void => { void jumpTo(rel, line, col); },
  run: (id: string): void => { void import("./commands").then((m) => m.runCommand(id)); },
  say: (text: string, error = false): void => toast(text, error),
  fail,
  includeOpen: (from: string, path: string, how: "open" | "reveal"): void => { void includeOeffnen(from, path, how); },
};
/** Absprung auf den Pfad einer Include-Zeile: Kern löst auf; Medium öffnet im passenden Editor (openFile), Quelltext als Reiter; fehlt die Datei, steht eine Meldung da */
async function includeOeffnen(from: string, path: string, how: "open" | "reveal"): Promise<void> {
  try {
    const r = await call("include_resolve", { from, path });
    if (r.status === "legacy") { toast(t("include-altpfad", { path }), true); return; }
    if (r.status !== "ok" || !r.rel) { toast(t("include-nicht-gefunden", { path }), true); return; }
    if (how === "reveal") { ui.sidebar = true; ui.selected = r.rel; return; }
    await openFile(r.rel);
  } catch (e) { fail(e); }
}
/** Prettyprint: nur Assembler und BASIC; „beim Verlassen der Zeile“ ist wählbar und gemerkt (Einstellungen kommen mit E12) */
export const fmtHooks: FormatHooks = {
  target: (st?: EditorState) => { const r = relOfState(st); const tb = r ? tabOf(r) : undefined; return tb && (tb.lang === "asm" || tb.lang === "basic") ? { rel: tb.scratch ? "" : tb.rel, lang: tb.lang } : null; },
  onLeave: () => ed.prettyLine,
  fail,
};
const commentToken = (): string | null => { const tb = ed.active ? tabOf(ed.active) : undefined; return !tb ? null : tb.lang === "asm" ? ";" : tb.lang === "basic" && tb.rel.toLowerCase().endsWith(".sbas") ? "'" : tb.lang === "markdown" || tb.lang === "plain" ? null : null; };
export function toggleWrap(): void {
  ed.wrap = !ed.wrap; try { localStorage.setItem("ed-wrap", ed.wrap ? "1" : "0"); } catch { /* weiter */ }
  const eff = wrapCompartment.reconfigure(wrapExtension(ed.wrap));
  for (const [rel, st] of [...states]) { const v = viewOf(rel); if (v) v.dispatch({ effects: eff }); else states.set(rel, st.update({ effects: eff }).state); }
}
/** Sprachwechsel: Suchleistentexte und Feldbeschriftung aller offenen Reiter neu setzen */
export function sprachenNeu(): void {
  for (const [rel, st] of [...states]) {
    const eff = langCompartment.reconfigure(langExtension(t("editor-aria", { name: tabOf(rel)?.title ?? rel })));
    const v = viewOf(rel); if (v) v.dispatch({ effects: eff }); else states.set(rel, st.update({ effects: eff }).state);
  }
}
onLangChange(sprachenNeu);
/** Zeilenende der Datei umschalten (LF ↔ CRLF); gilt beim nächsten Speichern, der Reiter ist bis dahin ungespeichert */
export function toggleEol(): void { const tb = ed.active ? tabOf(ed.active) : undefined; if (!tb || tb.scratch) return; tb.eol = tb.eol === "crlf" ? "lf" : "crlf"; tb.eolDirty = true; tb.dirty = true; }
export function setPrettyLine(on: boolean): void { ed.prettyLine = on; try { localStorage.setItem("ed-pretty", on ? "1" : "0"); } catch { /* weiter */ } }
function makeState(rel: string, text: string, lang: TextLang, name = rel): EditorState {
  return EditorState.create({ doc: text, extensions: extensions(lang, t("editor-aria", { name }), { onBreakpointClick: (line, shift) => { const v = viewOf(rel) ?? view; if (v) toggleLineMark(v, shift ? "bookmark" : "breakpoint", line); }, onGoto: gotoLineAsk, wrap: ed.wrap, lsp: lspHooks, fmt: fmtHooks, commentToken }) });
}

// Sprachdienste: Reiter mit Assembler und Datei melden sich beim Kern an (Text als Ganzes), jede Änderung geht gebündelt hinterher
bindHost({
  text: (rel) => states.get(rel)?.doc.toString(),
  apply: (rel, spec) => { const st = states.get(rel); if (!st) return; const v = viewOf(rel); if (v) v.dispatch(spec(st)); else states.set(rel, st.update(spec(st)).state); },
});
export function lspSyncTab(rel: string): void {
  const tab = tabOf(rel); const st = states.get(rel); if (!tab || !st) return;
  if (tab.lang === "asm" && !tab.scratch) { if (!lspHas(rel)) { lspOpen(rel, st.doc.toString()); } } else if (lspHas(rel)) lspClose(rel);
}
/** Ausführungszeile (ab 1) der Debug-Sitzung in `rel` markieren, in allen anderen Dateien löschen; `rel` null: überall löschen */
let debugZeile: { rel: string; line: number } | null = null;
export function setDebugZeile(rel: string | null, line: number | null): void {
  debugZeile = rel && line ? { rel, line } : null;
  for (const [r, st] of [...states]) {
    const eff = setDebugLine.of(r === rel ? line : null); const v = viewOf(r);
    if (v) { const zeigen = r === rel && line && line <= v.state.doc.lines ? [EditorView.scrollIntoView(v.state.doc.line(line).from, { y: "nearest", yMargin: 60 })] : []; v.dispatch({ effects: [eff, ...zeigen] }); }
    else states.set(r, st.update({ effects: eff }).state);
  }
}
/** Haltepunkte (Zeilen ab 1) aller geladenen Dateien */
export function alleHaltepunkte(): Record<string, number[]> { const o: Record<string, number[]> = {}; for (const [rel, g] of gemerkt) { if (!states.has(rel) && g.breakpoints.length) o[rel] = g.breakpoints.map((e) => e.line); } for (const [rel, st] of states) { const b = readMarks(st).breakpoints; if (b.length) o[rel] = b; } return o; }
/** Zu Zeile (ab 0) und Spalte einer Datei springen (Datei bei Bedarf öffnen), Zeile mittig */
export async function jumpTo(rel: string, line: number, col = 0): Promise<void> {
  if (ed.active !== rel) { await openFile(rel); await tick(); }   // Wechsel setzt den Zustand des Reiters ein; erst danach springen
  const v = getView(); const st = states.get(rel); if (!v || ed.active !== rel || !st) return;
  const doc = v.state.doc; const ln = doc.line(Math.min(Math.max(line, 0) + 1, doc.lines));
  v.dispatch({ selection: { anchor: ln.from + Math.min(col, ln.length) }, effects: EditorView.scrollIntoView(ln.from, { y: "center" }) }); v.focus();
}
/** Nach einem Bau haben sich Dateien auf der Platte geändert (z. B. build/export/*.asm): Sprachdienst für alle Assembler-Reiter neu abgleichen */
/** Zyklenzahl je Zeile des letzten Baus in die Reiter eintragen (Assembler, ungeänderte Reiter) */
export async function refreshCycles(only?: string): Promise<void> {
  for (const tb of ed.tabs) {
    if (tb.scratch || tb.lang !== "asm" || (only && tb.rel !== only)) continue;
    let z: [number, number, number][]; try { z = await call("build_cycles", { rel: tb.rel }); } catch { continue; }
    if (tb.dirty) continue;
    const eff = setCycles.of(z); const st = states.get(tb.rel); if (!st) continue;
    const v = viewOf(tb.rel); if (v) v.dispatch({ effects: eff }); else states.set(tb.rel, st.update({ effects: eff }).state);
  }
}
export function lspNeuAbgleichen(): void { for (const tb of ed.tabs) if (lspHas(tb.rel)) { lspClose(tb.rel); lspSyncTab(tb.rel); } }
export async function startSprachdienste(): Promise<void> { bindSearchTexts(dirtyTexts); await startLsp(); await startSuche(); await startBau(); await startTests(); }
export const lspTransactions = lspChanged;

const oeffnet = new Map<string, Promise<void>>();
const alsText = new Set<string>();
/** Datei ausdrücklich als Text öffnen (zum Beispiel einen Kachel-Bildschirm, den der Bildschirm-Editor nicht bearbeitet) */
export function openAlsText(rel: string): Promise<void> { alsText.add(rel); setMode("code"); return openFile(rel); }
/** Datei öffnen; derselbe Pfad nie zweimal: eine laufende Öffnung wird wiederverwendet (zwei schnelle Klicks, Enter, Befehle würden sonst zwei Reiter mit gleichem Pfad anlegen) */
export function openFile(rel: string): Promise<void> {
  const laeuft = oeffnet.get(rel); if (laeuft) return laeuft;
  const p = openFileEcht(rel).finally(() => { oeffnet.delete(rel); });
  oeffnet.set(rel, p); return p;
}
async function openFileEcht(rel: string): Promise<void> {
  const ohne = (f: string) => { toast(t("release-ohne-editor", { name: baseOf(rel), editor: t(`release-editor-${f}`) })); };   // Editor nicht im Release: Datei öffnet als Text
  const nurText = alsText.delete(rel);
  if (nurText) { /* ausdrücklich als Text */ }
  else if (/\.(d64|d71|d81|atr)$/i.test(rel)) { if (featureOn("disk")) { await openDisk({ rel }); return; } ohne("disk"); }   // Diskettenabbilder öffnen im Disk-Editor
  else if (istZeichensatzDatei(rel)) { if (featureOn("char")) { await openSprite(rel); return; } ohne("char"); }   // Zeichensatz-Dateien im Zeichensatz-Editor
  else if (istSongDatei(rel)) { if (featureOn("tracker")) { await openSong(rel); return; } ohne("tracker"); }   // Songs im Tracker
  else if (istBildschirmDatei(rel)) { if (featureOn("screen")) { if (await openBildschirm(rel)) return; } else ohne("screen"); }   // Textbildschirme im Bildschirm-Editor (Banken öffnen als Text)
  else if (istKachelDatei(rel)) { if (featureOn("tiles")) { await openKacheln(rel); return; } ohne("tiles"); }   // Kachelsätze im Kachel-Editor
  else if (istPaletteDatei(rel)) { if (featureOn("palette")) { await openPalette(rel); return; } ohne("palette"); }   // Palettendateien im Palette-Editor
  else if (istSpriteDatei(rel)) { if (featureOn("sprite")) { await openSprite(rel); return; } ohne("sprite"); }   // Sprite-Dateien im Sprite-Editor
  if (tabOf(rel)) { activate(rel); return; }
  try {
    const f = await call("file_read", { rel });
    if (tabOf(f.rel)) { activate(f.rel); return; }   // inzwischen anderweitig geöffnet
    let st = makeState(f.rel, f.text, f.lang);
    const gm = gemerkt.get(f.rel);
    if (gm) { const an = (v: MarkEntry[]) => v.map((e) => markAnker(st.doc, e)).filter((l): l is number => l !== null); st = st.update({ effects: marksEffect(an(gm.breakpoints), an(gm.bookmarks)), selection: { anchor: Math.min(gm.head, st.doc.length) } }).state; }
    if (debugZeile?.rel === f.rel) st = st.update({ effects: setDebugLine.of(debugZeile.line) }).state;
    states.set(f.rel, st); saved.set(f.rel, st.doc);
    ed.tabs.push({ rel: f.rel, lang: f.lang, eol: f.eol, dirty: false, group: ed.focus });
    lspSyncTab(f.rel);
    activate(f.rel);
    stat(f.rel);
  } catch (e) {
    if (e instanceof CoreError && e.code === "datei-nichttext" && hexStattExtern(rel)) { await openHex(rel); }
    else if (e instanceof CoreError && e.code === "datei-nichttext") { await oeffneExtern(rel, () => toast(t("editor-nichttext-extern", { name: baseOf(rel) }))); }
    else fail(e);
  }
}

/** Gruppen der geteilten Ansicht als Sicht auf `ed` (die Logik steht in `groups.ts`) */
const gstate: GState = {
  get tabs() { return ed.tabs; }, set tabs(v) { ed.tabs = v as Tab[]; },
  get active() { return ed.activeBy; }, set active(v) { ed.activeBy = v; },
  get focus() { return ed.focus; }, set focus(v) { ed.focus = v; },
};
/** `ed.active` und die Auswahl im Baum folgen der fokussierten Hälfte */
function syncActive(): void { const a = ed.activeBy[ed.focus]; ed.active = a; const tb = a ? tabOf(a) : undefined; if (tb && !tb.scratch) ui.selected = tb.rel; }
export const splitOn = (): boolean => isSplit(gstate);

export function activate(rel: string): void { const tb = tabOf(rel); if (!tb) return; void refreshCycles(rel); activateIn(gstate, rel); syncActive(); bump(rel); if (debugZeile?.rel === rel) setTimeout(() => debugZeile && setDebugZeile(debugZeile.rel, debugZeile.line), 0); }
/** Fokus auf eine Hälfte setzen (Klick oder Tastatur im Fenster); der Editor zeigt dort schon den richtigen Reiter */
export function focusGroup(g: Group): void { if (ed.focus === g || !ed.activeBy[g]) return; ed.focus = g; syncActive(); const v = views[g], a = ed.activeBy[g]; if (v && a) onState(a, v.state); }
/** Nach einer Änderung der Gruppen: `active` angleichen und die Hälften neu laden, deren aktiver Reiter sich geändert hat (oder alle, wenn sich die Teilung geändert hat) */
function afterGroups(before: { act: [string | null, string | null]; split: boolean }): void {
  syncActive();
  if (before.split !== isSplit(gstate)) bump(); else for (const g of [0, 1] as const) if (before.act[g] !== ed.activeBy[g]) ed.nonces[g]++;
}
const snapshotGroups = (): { act: [string | null, string | null]; split: boolean } => ({ act: [ed.activeBy[0], ed.activeBy[1]], split: isSplit(gstate) });
/** Teilen (aktiver Reiter in die rechte Hälfte) oder Teilung aufheben (Cmd+Alt+T) */
export function splitToggle(): void { const b = snapshotGroups(); if (!toggleSplit(gstate)) toast(t("split-zu-wenig"), true); afterGroups(b); setTimeout(() => getView()?.focus(), 80); }
/** Aktiven Reiter in die andere Hälfte schieben */
export function otherHalf(): void { const b = snapshotGroups(); if (!toOtherHalf(gstate)) toast(t("split-zu-wenig"), true); afterGroups(b); }
/** Reiter an eine Hälfte übergeben (Ziehen an den Rand der Fläche) */
export function moveToGroup(rel: string, g: Group, before: string | null = null): void { const b = snapshotGroups(); if (!moveTo(gstate, rel, g, before)) { toast(t("split-zu-wenig"), true); return; } afterGroups(b); }
export const tabName = (rel: string): string => tabOf(rel)?.title ?? baseOf(rel);

/** Der Editor meldet einen geänderten Zustand des aktiven Reiters */
const diagZaehler = { onState: 0, geaendert: 0, speichern: 0, letzter: "" };
/** Diagnose des Editorzustands (nur über `localStorage.vcDiag = "1"` im echten Fenster abrufbar, siehe App.svelte): je Reiter Punkt, Längen und Vergleich mit dem gespeicherten Stand */
export function diagEditor(): unknown {
  return { zaehler: { ...diagZaehler }, aktiv: ed.active, fokus: ed.focus, reiter: ed.tabs.map((x) => { const st = states.get(x.rel), sv = saved.get(x.rel); return { rel: x.rel, dirty: x.dirty, eolDirty: !!x.eolDirty, conflict: !!x.conflict, scratch: x.scratch ?? null, eol: x.eol, docLen: st?.doc.length ?? null, savedLen: sv?.length ?? null, gleich: st && sv ? st.doc.eq(sv) : null }; }) };
}
export function onState(rel: string, st: EditorState): void {
  diagZaehler.onState++; diagZaehler.letzter = rel;
  const vorher = states.get(rel); if (vorher && !vorher.doc.eq(st.doc)) docRev.n++;   // die Markdown-Vorschau folgt jeder Textänderung (auch Rückgängig im verborgenen Editor)
  states.set(rel, st);
  { const mk = JSON.stringify(readMarks(st)); if (markKeys.get(rel) !== mk) { markKeys.set(rel, mk); marksRev.n++; } }   // Marken geändert (oder mit dem Text verschoben): die Sitzung speichert sie
  const tab = tabOf(rel); if (!tab) return;
  const s = saved.get(rel);
  tab.dirty = (s ? !st.doc.eq(s) : true) || !!tab.eolDirty;
  if (s && !st.doc.eq(s)) diagZaehler.geaendert++;
  if (tab.scratch) sessionHooks.scratch();
  if (rel === ed.active) {
    const h = st.selection.main; const ln = st.doc.lineAt(h.head);
    ed.cursor = { line: ln.number, col: h.head - ln.from + 1, lines: st.doc.lines, selected: Math.abs(h.to - h.from) };
  }
}

/** Färbung und Faltbereiche des Textes neu vom Kern holen und in den Zustand setzen (verworfen, wenn der Text inzwischen ein anderer ist) */
export async function refresh(rel: string): Promise<void> {
  const st = states.get(rel); const tab = tabOf(rel); if (!st || !tab || tab.lang === "plain") return;
  const doc = st.doc; const text = doc.toString();
  try {
    const [colors, folds] = await Promise.all([call("syntax_color", { text, lang: tab.lang, rel: tab.scratch ? "" : rel }), call("fold_ranges", { text, lang: tab.lang, rel: tab.scratch ? "" : rel })]);
    const cur = states.get(rel); if (!cur || cur.doc !== doc) return;
    // natives BASIC: Dialekt, Gliederung und ausgeschlossene Zeilen (`#if`) vom Kern
    let info: BasicInfo | null = null;
    if (tab.lang === "basic" && !tab.scratch && !rel.toLowerCase().endsWith(".sbas")) { try { info = await call("basic_info", { rel, text }); } catch { info = null; } }
    const cur2 = states.get(rel); if (!cur2 || cur2.doc !== doc) return;
    setBasicInfo(rel, info);
    const effects = [setColors.of(colors), setFolds.of(folds), ...(info ? [setExcluded.of(info.excluded)] : [])];
    const v = viewOf(rel); if (v && v.state.doc === doc) v.dispatch({ effects }); else states.set(rel, cur.update({ effects }).state);
  } catch { /* ohne Kern keine Färbung */ }
}

export async function saveTab(rel: string): Promise<boolean> {
  const tab = tabOf(rel); const st = states.get(rel); if (!tab || !st) return false;
  if (tab.scratch) return saveScratchAs(rel);   // wartet auf die Namensabfrage; `false` bei Abbruch
  try {
    const gemischt = tab.eol === "gemischt";
    diagZaehler.speichern++;
    await call("file_write", { rel, text: st.doc.toString(), eol: gemischt ? "lf" : tab.eol });
    if (gemischt) { tab.eol = "lf"; toast(t("editor-eol-vereinheitlicht", { name: rel })); }
    saved.set(rel, st.doc); tab.dirty = false; tab.eolDirty = false; tab.conflict = false; await stat(rel);
    return true;
  } catch (e) { fail(e); return false; }
}
/** Speichern des aktiven Reiters; bei einem unbenannten kehrt der Befehl sofort zurück (die Namensabfrage läuft weiter) */
export const saveActive = async (): Promise<void> => { const rel = ed.active; if (!rel) return; const p = saveTab(rel); if (!tabOf(rel)?.scratch) await p; };
export async function saveAll(): Promise<void> { for (const tb of [...ed.tabs]) if (tb.dirty) await saveTab(tb.rel); }

/** Bildlauf je Reiter: bleibt beim Wechsel der Reiter und der Bereiche erhalten (das Editorfenster wird beim Bereichswechsel abgebaut) */
/** Zählt Textänderungen aller Reiter (die Markdown-Vorschau liest ihn, um sich neu zu zeichnen) */
export const docRev = $state({ n: 0 });
const scrolls = new Map<string, number>();
export function scrollMerken(rel: string, top: number): void { if (states.has(rel)) scrolls.set(rel, top); }
export function scrollHolen(rel: string): number | undefined { return scrolls.get(rel); }
/** Haltepunkte, Lesezeichen und Cursor geschlossener Reiter (je Projekt): beim Wiederöffnen der Datei sind sie wieder da */
const gemerkt = new Map<string, { breakpoints: MarkEntry[]; bookmarks: MarkEntry[]; head: number }>();
/** Marken eines Zustands mit dem Text ihrer Zeile (zum Wiederfinden nach Änderungen der Datei) */
function markEntries(st: EditorState): { breakpoints: MarkEntry[]; bookmarks: MarkEntry[] } {
  const m = readMarks(st); const mk = (l: number): MarkEntry => ({ line: l, text: st.doc.line(l).text.trim() });
  return { breakpoints: m.breakpoints.filter((l) => l >= 1 && l <= st.doc.lines).map(mk), bookmarks: m.bookmarks.filter((l) => l >= 1 && l <= st.doc.lines).map(mk) };
}
/** Gemerkte Zeile in einem (möglicherweise geänderten) Text wiederfinden: gleiche Zeile mit gleichem Text, sonst die nächste Zeile mit gleichem Text (bis 200 Zeilen weit), sonst verworfen */
export function markAnker(doc: Text, e: MarkEntry): number | null {
  const gleich = (l: number): boolean => l >= 1 && l <= doc.lines && doc.line(l).text.trim() === e.text;
  if (gleich(e.line)) return e.line;
  if (!e.text) return e.line >= 1 && e.line <= doc.lines ? e.line : null;   // leere Zeile: Nummer behalten
  for (let d = 1; d <= 200; d++) { if (gleich(e.line - d)) return e.line - d; if (gleich(e.line + d)) return e.line + d; }
  return null;
}
/** Haltepunkte und Lesezeichen aller Dateien des Projekts (offene und geschlossene) zum Speichern in der Sitzung */
export function markierungenAlle(): Record<string, FileMarks> {
  const o: Record<string, FileMarks> = {};
  for (const [rel, g] of gemerkt) if (g.breakpoints.length || g.bookmarks.length) o[rel] = { breakpoints: g.breakpoints, bookmarks: g.bookmarks };
  for (const [rel, st] of states) { if (tabOf(rel)?.scratch) continue; const m = markEntries(st); if (m.breakpoints.length || m.bookmarks.length) o[rel] = m; else delete o[rel]; }
  return o;
}
/** Beim Projektöffnen: gespeicherte Marken einsetzen (sie wirken beim Öffnen der Dateien und beim Debuggen) */
export function markierungenLaden(m: Record<string, FileMarks>): void {
  gemerkt.clear(); for (const [rel, v] of Object.entries(m)) gemerkt.set(rel, { ...v, head: 0 }); markKeys.clear();
  // Dateien, die schon offen sind (Start mit --open läuft neben dem Lesen der Sitzung): die Marken jetzt einsetzen, sonst blieben sie leer und überschrieben die gespeicherten
  for (const [rel, st] of [...states]) {
    const g = gemerkt.get(rel); if (!g || tabOf(rel)?.scratch) continue; const m0 = readMarks(st); if (m0.breakpoints.length || m0.bookmarks.length) continue;
    const an = (v: MarkEntry[]) => v.map((e) => markAnker(st.doc, e)).filter((l): l is number => l !== null);
    const eff = marksEffect(an(g.breakpoints), an(g.bookmarks)); const v = viewOf(rel);
    if (v) v.dispatch({ effects: eff }); else states.set(rel, st.update({ effects: eff }).state);
  }
  marksRev.n++;
}
/** Zählt Änderungen der Marken (die Sitzung speichert sie dann) */
export const marksRev = $state({ n: 0 });
const markKeys = new Map<string, string>();
export function gemerktLeeren(): void { gemerkt.clear(); markKeys.clear(); scrolls.clear(); }

function drop(rel: string): void {
  const i = ed.tabs.findIndex((x) => x.rel === rel); if (i < 0) return;
  const wasScratch = !!ed.tabs[i]!.scratch; const b = snapshotGroups();
  if (!wasScratch) { const st0 = states.get(rel); if (st0) { const m = markEntries(st0); if (m.breakpoints.length || m.bookmarks.length || st0.selection.main.head > 0) gemerkt.set(rel, { ...m, head: st0.selection.main.head }); else gemerkt.delete(rel); markKeys.delete(rel); marksRev.n++; } }
  scrolls.delete(rel);
  removeTab(gstate, rel); lspClose(rel); states.delete(rel); saved.delete(rel); stats.delete(rel);
  afterGroups(b);
  if (wasScratch) sessionHooks.scratch();
}
/** Reiter schließen; bei ungespeicherten Änderungen erst nachfragen */
export function closeTab(rel: string): void { const tb = tabOf(rel); if (!tb) return; if (tb.dirty) ed.closing = rel; else drop(rel); }
export async function closeDecision(what: "save" | "discard" | "cancel"): Promise<void> {
  const rel = ed.closing; ed.closing = null; if (!rel) return;
  if (what === "cancel") return;
  if (what === "save" && !(await saveTab(rel))) return;
  drop(rel);
}
export const hasDirty = (): boolean => ed.tabs.some((x) => x.dirty) || !!dk.view?.dirty;

/** Reiter verschieben (Ziehen): `rel` kommt an die Stelle von `before` */
export function moveTab(rel: string, before: string | null): void {
  const i = ed.tabs.findIndex((x) => x.rel === rel); if (i < 0) return;
  const target = before ? tabOf(before) : undefined;
  if (target && target.group !== ed.tabs[i]!.group) { moveToGroup(rel, target.group, before); return; }
  const [tb] = ed.tabs.splice(i, 1); const j = before ? ed.tabs.findIndex((x) => x.rel === before) : ed.tabs.length;
  ed.tabs.splice(j < 0 ? ed.tabs.length : j, 0, tb!);
}

/** Haltepunkte und Lesezeichen eines Reiters (Zeilen ab 1) */
export function markersOf(rel: string): { breakpoints: number[]; bookmarks: number[] } { const st = states.get(rel); return st ? readMarks(st) : { breakpoints: [], bookmarks: [] }; }
export function setMarkers(rel: string, breakpoints: number[], bookmarks: number[]): void { const st = states.get(rel); if (st) states.set(rel, st.update({ effects: marksEffect(breakpoints, bookmarks) }).state); }

// Dateiverwaltung im Projektbaum

function applyTree(r: TreeResult): void { if (ui.project) ui.project.tree = r.tree; }
export const parentOf = (rel: string): string => (rel.includes("/") ? rel.slice(0, rel.lastIndexOf("/")) : "");
export const baseOf = (rel: string): string => rel.slice(rel.lastIndexOf("/") + 1);

export function findNode(nodes: TreeNode[], rel: string): TreeNode | null { for (const n of nodes) { if (n.rel === rel) return n; const c = n.children && findNode(n.children, rel); if (c) return c; } return null; }

export async function createFile(rel: string): Promise<void> { try { const r = await call("file_create", { rel }); applyTree(r); await openFile(r.rel!); } catch (e) { fail(e); } }
export async function createDir(rel: string): Promise<void> { try { applyTree(await call("dir_create", { rel })); } catch (e) { fail(e); } }

function rekey(oldRel: string, newRel: string): void {
  const tb = tabOf(oldRel); if (!tb) return;
  const st = states.get(oldRel), sv = saved.get(oldRel);
  const ss = stats.get(oldRel); stats.delete(oldRel); if (ss !== undefined) stats.set(newRel, ss);
  states.delete(oldRel); saved.delete(oldRel);
  if (st) states.set(newRel, st);
  if (sv) saved.set(newRel, sv);
  lspClose(oldRel);
  tb.rel = newRel; ed.activeBy = ed.activeBy.map((x) => (x === oldRel ? newRel : x)) as [string | null, string | null]; if (ed.active === oldRel) { ed.active = newRel; ui.selected = newRel; }
  lspSyncTab(newRel);
}
/** Nach Umbenennen oder Verschieben: Reiter umbenennen und die Dateien neu lesen, in denen Include-Pfade angepasst wurden */
async function nachUmzug(r: TreeResult): Promise<void> {
  for (const [a, b] of r.moved ?? []) rekey(a, b);
  for (const [f] of r.rewritten ?? []) if (tabOf(f)) await reloadFile(f);
  const n = (r.rewritten ?? []).reduce((s, x) => s + x[1], 0);
  if (n > 0) toast(t("tree-includes-angepasst", { n }));
}
export async function renameFile(old: string, neu: string): Promise<void> {
  if (hasDirty()) { toast(t("tree-umbenennen-erst-speichern"), true); return; }
  try { const r = await call("file_rename", { old, new: neu }); applyTree(r); await nachUmzug(r); if (ui.selected === old) ui.selected = r.rel!; } catch (e) { fail(e); }
}
export async function renameDir(old: string, neu: string): Promise<void> {
  if (hasDirty()) { toast(t("tree-umbenennen-erst-speichern"), true); return; }
  try { const r = await call("dir_rename", { old, new: neu }); applyTree(r); await nachUmzug(r); if (ui.selected?.startsWith(old + "/")) ui.selected = null; } catch (e) { fail(e); }
}
export async function deleteFile(rel: string, trash = true): Promise<void> {
  try {
    const r = await call("file_delete", { rel, trash }); applyTree(r); drop(rel);
    if (r.trashed) toast(t("tree-geloescht-papierkorb", { name: baseOf(rel) })); else toast(t("tree-geloescht-endgueltig", { name: baseOf(rel) }));
  } catch (e) {
    // Papierkorb nicht verfügbar: die Datei ist noch da; endgültig nur nach ausdrücklichem Ja (Vorgabe: Abbrechen)
    if (trash && e instanceof CoreError && e.code === "fileops-papierkorb-fehlgeschlagen") { if (await fragen(t("rf-papierkorb-titel"), t("rf-papierkorb-text", { name: baseOf(rel) }), t("rf-endgueltig-loeschen"))) await deleteFile(rel, false); }
    else fail(e);
  }
}
/** Datei im Standardprogramm des Systems öffnen; Programme (.exe, .sh, .app …) nur nach ausdrücklichem Ja (Vorgabe: Abbrechen) */
export async function oeffneExtern(rel: string, danach?: () => void): Promise<void> {
  try { await call("file_external", { rel }); danach?.(); }
  catch (e) {
    if (e instanceof CoreError && e.code === "extern-ausfuehrbar") {
      if (!(await fragen(t("rf-programm-titel"), t("rf-programm-text", { name: baseOf(rel) }), t("rf-trotzdem-oeffnen")))) return;
      try { await call("file_external", { rel, confirmed: true }); danach?.(); } catch (e2) { fail(e2); }
    } else fail(e);
  }
}
/** Datei oder Ordner in einen Zielordner ziehen (`""` = Projektwurzel): Include-Pfade und Projektverweise ziehen mit, vorher wird gesichert */
export async function moveInto(rel: string, kind: "file" | "dir", folder: string): Promise<void> {
  const neu = folder ? `${folder}/${baseOf(rel)}` : baseOf(rel);
  if (neu === rel || (kind === "dir" && (folder === rel || folder.startsWith(rel + "/")))) return;
  if (hasDirty()) { toast(t("tree-verschieben-erst-speichern"), true); return; }
  try {
    const r = await call("tree_move", { from: rel, folder }); applyTree(r);
    for (const [a, b] of r.moved) rekey(a, b);
    for (const [f] of r.rewritten) if (tabOf(f)) await reloadFile(f);
    toast(t("tree-verschoben", { name: baseOf(rel), n: r.rewritten.reduce((s, x) => s + x[1], 0) }));
  } catch (e) { fail(e); }
}
export async function treeUndo(): Promise<void> {
  if (hasDirty()) { toast(t("tree-verschieben-erst-speichern"), true); return; }
  try {
    const r = await call("tree_undo"); applyTree(r);
    for (const [a, b] of r.moves) rekey(b, a);
    for (const tb of [...ed.tabs]) if (!tb.scratch) await reloadFile(tb.rel);
    toast(t("tree-rueckgaengig-gemacht", { n: r.n }));
  } catch (e) { fail(e); }
}
export async function reloadTree(): Promise<void> { try { applyTree(await call("tree_reload")); } catch (e) { fail(e); } }
export async function copyPath(rel: string): Promise<void> {
  try { const p = await call("abs_path", { rel }); await navigator.clipboard.writeText(p); toast(t("tree-pfad-kopiert", { path: p })); } catch (e) { fail(e); }
}
export async function revealIn(rel: string): Promise<void> { try { await call("file_reveal", { rel }); } catch (e) { fail(e); } }

// Eingabe eines Namens (Neu, Umbenennen, Löschen-Nachfrage)
export function askName(p: Prompt): void { ed.prompt = p; }
export function nameError(v: string): string | null {
  const s = v.trim();
  if (!s) return t("tree-name-leer");
  if (/[\\:*?"<>|]/.test(s)) return t("tree-name-zeichen");
  return null;
}
const ARTS = [{ id: "asm", label: "tree-art-asm", ext: ".asm" }, { id: "basic", label: "tree-art-basic", ext: ".bas" }, { id: "text", label: "tree-art-text", ext: ".txt" }, { id: "notiz", label: "tree-art-notiz", ext: ".md" }, { id: "none", label: "tree-art-keine", ext: "" }];
/** Art zu einem eingegebenen Namen (nach der Endung); `null`, solange keine Endung dasteht */
export function artOfName(name: string): string | null {
  const m = /\.([A-Za-z0-9]+)$/.exec(name.trim()); if (!m) return null;
  const e = m[1]!.toLowerCase();
  return ["asm", "inc", "s"].includes(e) ? "asm" : ["bas", "sbas", "lst", "tur"].includes(e) ? "basic" : e === "txt" ? "text" : ["md", "markdown"].includes(e) ? "notiz" : "none";
}
/** Freier Name im Ordner: `name`, sonst `name-2.ext`, `name-3.ext` … */
export function freeName(folder: string, name: string): string {
  const tree = ui.project?.tree ?? []; const taken = (n: string) => !!findNode(tree, folder ? `${folder}/${n}` : n);
  if (!taken(name)) return name;
  const m = /^(.*?)(\.[^.]+)?$/.exec(name)!; const base = m[1]!, ext = m[2] ?? "";
  for (let i = 2; i < 1000; i++) { const n = `${base}-${i}${ext}`; if (!taken(n)) return n; }
  return name;
}
const withExt = (name: string, ext: string) => (/\.[A-Za-z0-9]+$/.test(name) ? name : name + ext);
export function newFileIn(folder: string, art = "asm"): void {
  const arts = ARTS.map((a) => ({ id: a.id, label: t(a.label), ext: a.ext }));
  askName({ title: t("tree-neue-datei"), label: t("tree-name-label"), value: "", ok: t("dialog-anlegen"), validate: nameError, arts, art,
    onok: (v, id) => { const ext = ARTS.find((a) => a.id === id)?.ext ?? ".asm"; const n = withExt(v.trim(), ext); return createFile(folder ? `${folder}/${n}` : n); } });
}
export function newNoteIn(folder: string): void {
  askName({ title: t("tree-neue-notiz"), label: t("tree-name-label"), value: freeName(folder, "notiz.md"), ok: t("dialog-anlegen"), validate: nameError, onok: (v) => { const n = withExt(v.trim(), ".md"); return createFile(folder ? `${folder}/${n}` : n); } });
}
export function newDirIn(folder: string): void { askName({ title: t("tree-neuer-ordner"), label: t("tree-name-label"), value: "", ok: t("dialog-anlegen"), validate: nameError, onok: (v) => createDir(folder ? `${folder}/${v.trim()}` : v.trim()) }); }
export function renameAsk(rel: string, kind: "file" | "dir"): void {
  askName({ title: t("tree-umbenennen"), label: t("tree-name-label"), value: baseOf(rel), ok: t("dialog-umbenennen"), validate: nameError, onok: (v) => { const n = parentOf(rel) ? `${parentOf(rel)}/${v.trim()}` : v.trim(); return kind === "file" ? renameFile(rel, n) : renameDir(rel, n); } });
}
export async function deleteAsk(rel: string): Promise<void> {
  let korb = true; try { korb = await call("tree_can_trash"); } catch { /* weiter mit Papierkorb */ }
  askName({ title: t("tree-loeschen"), label: korb ? t("tree-loeschen-frage", { name: baseOf(rel) }) : t("tree-loeschen-endgueltig-frage", { name: baseOf(rel) }), value: "", ok: t("dialog-loeschen"), onok: () => deleteFile(rel, korb) });
}

/** Beim Projektwechsel oder Schließen alles zurücksetzen */
/** Texte der ungespeicherten Reiter (Datei → Text), für die Projektsuche */
/** Aktueller (auch ungespeicherter) Text eines offenen Reiters */
export function textOf(rel: string): string | null { return states.get(rel)?.doc.toString() ?? null; }
/** Offene Quelltext-Reiter einer Sprache (`asm` oder `basic`), der aktive zuerst; unbenannte Reiter zählen nicht */
export function sourceTabs(lang: "asm" | "basic"): string[] { const t = ed.tabs.filter((x) => x.lang === lang && !x.scratch).map((x) => x.rel); return t.sort((a, b) => (b === ed.active ? 1 : 0) - (a === ed.active ? 1 : 0)); }
/** Text unter der Cursorzeile des Reiters `rel` einfügen (am Zeilenanfang: davor, sonst hinter der Zeile); wechselt dazu in die Code-Ansicht */
export async function insertAtCursorLine(rel: string, text: string): Promise<boolean> {
  activate(rel); ui.mode = "code"; ui.extra = null;
  let v: EditorView | null = null;
  for (let i = 0; i < 30 && !v; i++) { await new Promise((r) => requestAnimationFrame(() => r(null))); const c = getView(); if (c && ed.active === rel) v = c; }
  if (!v) return false;
  const head = v.state.selection.main.head; const ln = v.state.doc.lineAt(head); const front = head === ln.from; const at = front ? ln.from : ln.to;
  const ins = front ? `${text}\n` : `\n${text}`;
  v.dispatch({ changes: { from: at, insert: ins }, selection: { anchor: at + ins.length }, scrollIntoView: true }); v.focus(); return true;
}
export function dirtyTexts(): Record<string, string> { const o: Record<string, string> = {}; for (const tb of ed.tabs) if (tb.dirty && !tb.scratch) { const st = states.get(tb.rel); if (st) o[tb.rel] = st.doc.toString(); } return o; }
export function resetEditor(): void { lspReset(); resetSearch(); resetBau(); resetTests(); ed.tabs = []; ed.active = null; ed.activeBy = [null, null]; ed.focus = 0; gemerktLeeren(); states.clear(); saved.clear(); stats.clear(); ed.closing = null; ed.prompt = null; bump(); }

/** Gehe zu Zeile: eigener Dialog (statt der CodeMirror-Leiste, die beim Reiterwechsel stört) */
export function gotoLineAsk(): void {
  const v = getView(); if (!v) return; const max = v.state.doc.lines;
  const bad = (s: string) => { const n = Number(s.trim()); return Number.isInteger(n) && n >= 1 && n <= max ? null : t("editor-goto-invalid", { max }); };
  askName({ title: t("editor-goto-title"), label: t("editor-goto-label", { max }), value: String(ed.cursor.line), ok: t("editor-goto-ok"), validate: bad,
    onok: (s) => { const ln = v.state.doc.line(Number(s.trim())); v.dispatch({ selection: { anchor: ln.from }, scrollIntoView: true }); v.focus(); } });
}

// Abfrage ungespeicherter Änderungen vor Projektwechsel und Beenden
let leaveResolve: ((go: boolean) => void) | null = null;
/** `true`, wenn fortgefahren werden darf (nichts offen, gespeichert oder verworfen); sonst `false` (Abbrechen) */
export function confirmLeave(): Promise<boolean> {
  const dirty = ed.tabs.filter((x) => x.dirty); const disk = [...(dk.view?.dirty ? [{ name: `${dk.view.title} (${dk.view.format})`, scratch: false }] : []), ...(kt.view?.dirty ? [{ name: kt.view.title, scratch: false }] : []), ...(sp.view?.dirty ? [{ name: sp.view.title, scratch: false }] : []), ...(pal.view?.dirty ? [{ name: pal.view.title, scratch: false }] : []), ...(kc.view?.dirty ? [{ name: kc.view.title, scratch: false }] : []), ...(bs.view?.dirty ? [{ name: bs.view.title, scratch: false }] : []), ...(tk.view?.dirty ? [{ name: tk.view.title, scratch: false }] : [])];
  if (!dirty.length && !disk.length) { void closeSprite(); void closePalette(); void closeKacheln(); void closeKarten(); void closeBildschirm(); void closeSong(); return Promise.resolve(true); }
  ed.leaving = { names: [...dirty.map((x) => tabName(x.rel)), ...disk.map((x) => x.name)].join(", "), items: [...dirty.map((x) => ({ name: tabName(x.rel), scratch: !!x.scratch })), ...disk] };
  return new Promise((res) => { leaveResolve = res; });
}
export async function leaveDecision(what: "save" | "discard" | "cancel"): Promise<void> {
  const r = leaveResolve; leaveResolve = null;
  ed.leaving = null;
  // saveAll wartet auch auf die Namensabfragen unbenannter Reiter
  if (what === "save") { await saveAll(); if (dk.view?.dirty) await saveDisk(); if (sp.view?.dirty) await saveSprite(); if (pal.view?.dirty) await savePalette(); if (kc.view?.dirty) await saveKacheln(); if (kt.view?.dirty) await saveKarten(); if (bs.view?.dirty) await saveBildschirm(); if (tk.view?.dirty) await saveSong(); if (hasDirty() || sp.view?.dirty || pal.view?.dirty || kc.view?.dirty || bs.view?.dirty || tk.view?.dirty) { r?.(false); return; } }
  if (what === "discard") { sessionHooks.discard(); if (dk.view?.dirty) await closeDisk(); } if (what !== "cancel") { await closeSprite(); await closePalette(); await closeKacheln(); await closeKarten(); await closeBildschirm(); await closeSong(); sessionHooks.flush(); } r?.(what !== "cancel");
}

// Änderungen von außen: beim Zurückkehren ins Fenster Größe und Zeit der offenen Dateien vergleichen
const stats = new Map<string, [number, number] | null>();
async function stat(rel: string): Promise<void> { try { const [s] = await call("file_stats", { rels: [rel] }); stats.set(rel, s ?? null); } catch { /* ohne Kern nichts */ } }
export async function reloadFile(rel: string): Promise<void> {
  const tab = tabOf(rel); if (!tab) return;
  try {
    const f = await call("file_read", { rel }); const old = states.get(rel); const mk = markersOf(rel); const head = old?.selection.main.head ?? 0;
    const st = makeState(rel, f.text, f.lang); states.set(rel, st); saved.set(rel, st.doc);
    tab.eol = f.eol; tab.lang = f.lang; tab.dirty = false; tab.conflict = false;
    setMarkers(rel, mk.breakpoints, mk.bookmarks);
    const s2 = states.get(rel)!; states.set(rel, s2.update({ selection: { anchor: Math.min(head, s2.doc.length) } }).state);
    lspClose(rel); lspSyncTab(rel);
    await stat(rel); bump(rel);
  } catch (e) { fail(e); }
}
export async function checkExternal(): Promise<void> {
  if (ui.project) { try { const r = await call("tree_reload"); if (JSON.stringify(r.tree) !== JSON.stringify(ui.project.tree)) ui.project.tree = r.tree; } catch { /* Projekt schon zu */ } }   // von außen angelegte oder gelöschte Dateien erscheinen beim Zurückkehren ins Fenster
  const rels = ed.tabs.filter((x) => !x.scratch).map((x) => x.rel); if (!rels.length) return;
  let cur; try { cur = await call("file_stats", { rels }); } catch { return; }
  for (let i = 0; i < rels.length; i++) {
    const rel = rels[i]!, now = cur[i] ?? null, was = stats.get(rel);
    if (was === undefined) { stats.set(rel, now); continue; }
    if ((now === null && was === null) || (now && was && now[0] === was[0] && now[1] === was[1])) continue;
    stats.set(rel, now); const tab = tabOf(rel); if (!tab) continue;
    if (now === null) { toast(t("editor-extern-geloescht", { name: baseOf(rel) }), true); tab.dirty = true; continue; }
    if (tab.dirty) { if (!tab.conflict) { tab.conflict = true; toast(t("editor-extern-konflikt", { name: baseOf(rel) }), true); } }
    else { await reloadFile(rel); toast(t("editor-extern-neu", { name: baseOf(rel) })); }
  }
}

// Unbenannte Reiter und Notizen (noch ohne Datei; „Speichern“ fragt nach dem Namen)
let scratchN = 0;
/** Unbenannte Reiter mit Inhalt (für die Zwischenkopie) */
export function scratchTexts(): { name: string; text: string; note: boolean }[] {
  const o: { name: string; text: string; note: boolean }[] = [];
  for (const tb of ed.tabs) if (tb.scratch) { const st = states.get(tb.rel); const text = st?.doc.toString() ?? ""; if (text.trim()) o.push({ name: tb.title ?? tb.rel, text, note: tb.scratch === "note" }); }
  return o;
}
/** Einen gesicherten unbenannten Reiter wieder anlegen (mit Inhalt, als ungespeichert) */
export function restoreScratch(text: string, note: boolean): void {
  newScratch(note);
  const rel = ed.active!; const st = states.get(rel); if (!st) return;
  states.set(rel, st.update({ changes: { from: 0, to: st.doc.length, insert: text } }).state); bump(rel);
  const tb = tabOf(rel); if (tb) tb.dirty = true;
}
export function newScratch(note: boolean): void {
  scratchN++; const id = `\u0001scratch:${scratchN}`; const lang: TextLang = note ? "markdown" : "asm"; const title = t(note ? "editor-notiz" : "editor-unbenannt", { n: scratchN });
  const st = makeState(id, "", lang, title); states.set(id, st); saved.set(id, st.doc);
  ed.tabs.push({ rel: id, lang, eol: "lf", dirty: false, scratch: note ? "note" : "text", title, group: ed.focus }); activate(id);
}
const langOf = (rel: string): TextLang => { const l = rel.toLowerCase(); return /\.(asm|inc|s)$/.test(l) ? "asm" : /\.(sbas|bas|lst|tur)$/.test(l) ? "basic" : /\.(md|markdown)$/.test(l) ? "markdown" : "plain"; };
function saveScratchAs(rel: string): Promise<boolean> {
  const tab = tabOf(rel); const st = states.get(rel); if (!tab || !st) return Promise.resolve(false);
  const note = tab.scratch === "note"; const arts = ARTS.map((a) => ({ id: a.id, label: t(a.label), ext: a.ext }));
  return new Promise<boolean>((res) => {
    askName({ title: t("editor-speichern-unter"), label: t("tree-name-label"), value: freeName("", note ? "notiz.md" : "neu.asm"), ok: t("close-speichern"), validate: nameError, arts, art: note ? "notiz" : "asm",
      oncancel: () => res(false),
      onok: async (v, id) => {
        try {
          const name = withExt(v.trim(), ARTS.find((a) => a.id === id)?.ext ?? "");
          const r = await call("file_create", { rel: name }); applyTree(r); const nrel = r.rel!;
          await call("file_write", { rel: nrel, text: st.doc.toString(), eol: "lf" });
          rekey(rel, nrel); const nt = tabOf(nrel)!; nt.scratch = undefined; sessionHooks.scratch(); nt.title = undefined; nt.lang = langOf(nrel);
          saved.set(nrel, st.doc); nt.dirty = false; await stat(nrel); ui.selected = nrel; void refresh(nrel);
          res(true);
        } catch (e) { fail(e); res(false); }
      } });
  });
}

// Art des Reiters (Text, Assembler, BASIC) umschalten: bestimmt Färbung und Faltung
export function setLang(lang: TextLang): void {
  const rel = ed.active; const tab = rel ? tabOf(rel) : undefined; if (!rel || !tab) return;
  tab.lang = lang; lspSyncTab(rel);
  if (lang === "plain" || lang === "markdown") { const eff = [setColors.of([]), setFolds.of([])]; const v0 = viewOf(rel); if (v0) v0.dispatch({ effects: eff }); else { const s = states.get(rel); if (s) states.set(rel, s.update({ effects: eff }).state); } }
  else void refresh(rel);
}

// Schriftgröße des Editors (9 bis 28 Punkt), bleibt erhalten
export function initEditor(): void { try { const v = Number(localStorage.getItem("ed-size")); if (v >= 9 && v <= 28) ed.fontSize = v; ed.prettyLine = localStorage.getItem("ed-pretty") !== "0"; ed.wrap = localStorage.getItem("ed-wrap") !== "0"; } catch { /* weiter */ } applyFont(); }
function applyFont(): void { document.documentElement.style.setProperty("--ed-size", `${ed.fontSize}px`); }
export function fontStep(d: number): void { ed.fontSize = Math.max(9, Math.min(28, ed.fontSize + d)); try { localStorage.setItem("ed-size", String(ed.fontSize)); } catch { /* weiter */ } applyFont(); for (const v of views) v?.requestMeasure(); }

// Reiter: nächster, voriger, verschieben
export function stepTab(d: number): void { const l = ed.tabs.filter((x) => x.group === ed.focus); const i = l.findIndex((x) => x.rel === ed.active); if (i < 0 || !l.length) return; activate(l[(i + d + l.length) % l.length]!.rel); }
export function shiftTab(d: number): void {
  const l = ed.tabs.filter((x) => x.group === ed.focus); const i = l.findIndex((x) => x.rel === ed.active); const j = i + d; if (i < 0 || j < 0 || j >= l.length) return;
  const a = ed.tabs.indexOf(l[i]!), b = ed.tabs.indexOf(l[j]!); const [tb] = ed.tabs.splice(a, 1); ed.tabs.splice(b, 0, tb!);
}
export function closeOthers(rel: string): void { for (const tb of [...ed.tabs]) if (tb.rel !== rel) closeTab(tb.rel); }
export function closeEvery(): void { for (const tb of [...ed.tabs]) closeTab(tb.rel); }

export function bookmarkStep(dir: 1 | -1): void { const v = getView(); if (v) jumpBookmark(v, dir); }
export function bookmarkToggle(): void { const v = getView(); if (v) toggleLineMark(v, "bookmark"); }
