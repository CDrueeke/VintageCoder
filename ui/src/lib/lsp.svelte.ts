// SPDX-License-Identifier: GPL-3.0-or-later
// Sprachdienste im Editor (E1.2): Abgleich der offenen Texte mit dem Kern (`doc_open`/`doc_change`/`doc_close`, Festlegung in docs/text-dokumente.md),
// Diagnosen (Ereignis `lsp:diagnostics`) als Wellenlinie und als Probleme-Liste. Der Kern arbeitet auf dem Stand der Oberfläche, nie auf der Datei.
import { setDiagnostics, type Diagnostic } from "@codemirror/lint";
import { ChangeSet, type EditorState, type Transaction, type TransactionSpec } from "@codemirror/state";
import { call, CoreError, on, type LspDiag } from "./api";
import { msgText } from "./i18n.svelte";
import { fail } from "./state.svelte";

/** Was die Sprachdienste vom Editor brauchen (vom Editorzustand eingesetzt, vermeidet einen Importkreis) */
export interface Host {
  text(rel: string): string | undefined;
  /** Änderung am Zustand des Reiters (sichtbar oder im Hintergrund) */
  apply(rel: string, spec: (st: EditorState) => TransactionSpec): void;
}
let host: Host | null = null;
export function bindHost(h: Host): void { host = h; }

export const lsp = $state({ diags: {} as Record<string, LspDiag[]>, on: true });

interface Sync { version: number; pending: ChangeSet | null; timer: ReturnType<typeof setTimeout> | undefined; chain: Promise<void> }
const syncs = new Map<string, Sync>();
/** Ende der Anfragen eines geschlossenen Reiters: ein neues `doc_open` unter demselben Namen wartet darauf (sonst könnte das späte `doc_close` es wieder löschen) */
const tails = new Map<string, Promise<void>>();
const DELAY = 220;

const lastFail = new Map<string, number>();
function failOnce(e: unknown): void { const k = e instanceof CoreError ? e.code : String(e); const n = Date.now(); if ((lastFail.get(k) ?? 0) + 5000 < n) { lastFail.set(k, n); fail(e); } }

/** Reiter bei den Sprachdiensten anmelden (Text als Ganzes) */
export function lspOpen(rel: string, text: string): void {
  const old = syncs.get(rel); if (old) clearTimeout(old.timer);
  const s: Sync = { version: 0, pending: null, timer: undefined, chain: old?.chain ?? tails.get(rel) ?? Promise.resolve() };
  tails.delete(rel); syncs.set(rel, s);
  s.chain = s.chain.then(() => call("doc_open", { rel, text }).then(() => undefined, failOnce));
}
export function lspClose(rel: string): void {
  const s = syncs.get(rel); if (!s) return;
  clearTimeout(s.timer); syncs.delete(rel); delete lsp.diags[rel];
  s.chain = s.chain.then(() => call("doc_close", { rel }).then(() => undefined, () => undefined));
  tails.set(rel, s.chain);
}
export const lspHas = (rel: string): boolean => syncs.has(rel);

/** Änderungen eines Zustandsübergangs sammeln; gesendet wird nach kurzer Tippruhe, gebündelt */
export function lspChanged(rel: string, trs: readonly Transaction[]): void {
  const s = syncs.get(rel); if (!s) return;
  for (const tr of trs) if (tr.docChanged) s.pending = s.pending ? s.pending.compose(tr.changes) : tr.changes;
  if (!s.pending) return;
  clearTimeout(s.timer); s.timer = setTimeout(() => { void lspFlush(rel); }, DELAY);
}

/** Alles Ausstehende jetzt senden und warten, bis der Kern es hat (vor Hover, Vervollständigung usw.) */
export function lspFlush(rel: string): Promise<void> {
  const s = syncs.get(rel); if (!s) return Promise.resolve();
  clearTimeout(s.timer);
  const ch = s.pending;
  if (ch) {
    s.pending = null; const version = ++s.version;
    const changes: { from: number; to: number; insert: string }[] = [];
    ch.iterChanges((from, to, _f, _t, ins) => { changes.push({ from, to, insert: ins.toString() }); });
    s.chain = s.chain.then(() => call("doc_change", { rel, version, changes }).then(() => undefined, async (e) => {
      // Versionssprung oder unbekanntes Dokument: ganzen Text neu senden
      if (e instanceof CoreError && (e.code === "doc-version" || e.code === "doc-unbekannt")) {
        const text = host?.text(rel); if (text !== undefined) { s.version = version; await call("doc_open", { rel, text }).then(() => undefined, failOnce); return; }
      }
      failOnce(e);
    }));
  }
  return s.chain;
}
/** Stand ist beim Kern angekommen und es gibt nichts Ausstehendes */
export const lspFresh = (rel: string, version: number | null): boolean => { const s = syncs.get(rel); return !!s && !s.pending && (version === null || version === s.version); };

const SEV = ["error", "error", "warning", "info", "hint"] as const;

/** Meldung des Kerns als Diagnose für CodeMirror (Positionen im aktuellen Text; Spalte 0 bis Zeilenende = ganze Zeile) */
export function toCm(state: EditorState, d: LspDiag): Diagnostic | null {
  const doc = state.doc; if (d.line + 1 > doc.lines) return null;
  const ln = doc.line(d.line + 1);
  const from = ln.from + Math.min(d.from, ln.length);
  const le = doc.line(Math.min(d.endLine, doc.lines - 1) + 1);
  let to = le.from + Math.min(d.to, le.length);
  if (to <= from) to = Math.min(from + 1, doc.length);
  return { from, to: Math.max(to, from), severity: d.code === "W203" ? "info" : (SEV[d.severity] ?? "error"), message: msgText(d.key, d.args, d.message), source: d.code };
}

/** Diagnosen eines Reiters in dessen Editorzustand einsetzen */
export function applyDiags(rel: string): void {
  host?.apply(rel, (st) => setDiagnostics(st, (lsp.diags[rel] ?? []).map((d) => toCm(st, d)).filter((x): x is Diagnostic => x !== null)));
}

/** Zähler für die Statuszeile: Fehler, Warnungen, Hinweise */
export function counts(rels?: string[]): { errors: number; warnings: number; hints: number } {
  let errors = 0, warnings = 0, hints = 0;
  for (const [rel, items] of Object.entries(lsp.diags)) {
    if (rels && !rels.includes(rel)) continue;
    let lang = false;   // zu weite Sprünge (W203, `.branchlong auto` vom Nutzer bestellt) zählen je Datei einmal als Hinweis
    for (const d of items) { if (d.code === "W203") { lang = true; continue; } if (d.severity === 1) errors++; else if (d.severity === 2) warnings++; else if (d.severity === 3) hints++; }
    if (lang) hints++;
  }
  return { errors, warnings, hints };
}

let listening = false;
export async function startLsp(): Promise<void> {
  if (listening) return; listening = true;
  await on("lsp:diagnostics", (p) => {
    const ev = p as { rel: string; version: number | null; items: LspDiag[] };
    lsp.diags[ev.rel] = ev.items;
    // Meldungen zu einem älteren Stand würden falsch liegen: nur anwenden, wenn der Text seither unverändert ist (die nächste Antwort folgt)
    if (syncs.has(ev.rel) ? lspFresh(ev.rel, ev.version) : true) applyDiags(ev.rel);
  });
}
export function lspReset(): void { for (const s of syncs.values()) clearTimeout(s.timer); syncs.clear(); tails.clear(); lsp.diags = {}; }
