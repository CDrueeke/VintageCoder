// SPDX-License-Identifier: GPL-3.0-or-later
// BASIC umwandeln im Editor: „In Labels umwandeln“, „In Nummern umwandeln“, „Neu nummerieren“ (Konzept „BASIC-Editor und Dialekte“, B1–B3).
// Das Fachliche (Modus, Umwandlung, Warnungen) macht der Kern (`vc_ide::basicconv`, Befehl `basic_convert`); hier stehen Zustand und Ablauf des Dialogs.
// Der Text kommt aus dem Editor (auch ungespeichert); „Anwenden“ ersetzt ihn als EINE Änderung (ein Rückgängig-Schritt), eingebundene Dateien schreibt der Kern.
import { isolateHistory } from "@codemirror/commands";
import { call, type BasicConvKind, type BasicConvResult } from "./api";
import { ed, getView, reloadFile, textOf } from "./editor.svelte";
import { t } from "./i18n.svelte";
import { fail, toast } from "./state.svelte";

export const bc = $state({
  open: false,
  kind: "labels" as Exclude<BasicConvKind, "mode">,
  rel: "",
  /** Text der Datei beim Öffnen des Dialogs (die Vorschau gilt für genau diesen Text) */
  text: "",
  includes: true,
  from: "" as string,
  to: "" as string,
  busy: false,
  result: null as BasicConvResult | null,
  /** Fehler beim Anwenden (z. B. eingebundene Datei hat ungespeicherte Änderungen) */
  error: "",
});

let seq = 0;
const num = (s: string): number | undefined => { const n = Number(s.trim()); return s.trim() !== "" && Number.isInteger(n) && n >= 0 ? n : undefined; };

/** Ist der aktive Reiter eine BASIC-Datei des Projekts (nicht unbenannt)? */
export function basicActive(): boolean { const tb = ed.tabs.find((x) => x.rel === ed.active); return !!tb && tb.lang === "basic" && !tb.scratch && !tb.rel.toLowerCase().endsWith(".sbas"); }

export async function openConv(kind: Exclude<BasicConvKind, "mode">): Promise<void> {
  const rel = ed.active; const tb = ed.tabs.find((x) => x.rel === rel);
  if (!rel || !tb || tb.scratch || tb.lang !== "basic") { toast(t("basicconv-nur-basic"), true); return; }
  const text = textOf(rel); if (text === null) return;
  Object.assign(bc, { open: true, kind, rel, text, includes: true, from: "", to: "", result: null, error: "", busy: false });
  await refresh();
}

/** Vorschau neu berechnen (nach einer Änderung der Optionen) */
export async function refresh(): Promise<void> {
  const my = ++seq; bc.busy = true; bc.error = "";
  const r = bc.kind === "renumber" ? { from: num(bc.from), to: num(bc.to) } : {};
  try {
    const res = await call("basic_convert", { kind: bc.kind, rel: bc.rel, text: bc.text, includes: bc.includes, ...(r.from !== undefined && r.to !== undefined ? { from: r.from, to: r.to } : {}) });
    if (my === seq) bc.result = res;
  } catch (e) { if (my === seq) { bc.result = null; fail(e); } }
  finally { if (my === seq) bc.busy = false; }
}

/** Kleinste Änderung, die `a` in `b` verwandelt (gemeinsamer Anfang und Ende bleiben, damit Cursor, Marken und Faltungen erhalten bleiben) */
export function minimalChange(a: string, b: string): { from: number; to: number; insert: string } | null {
  if (a === b) return null;
  let p = 0; const m = Math.min(a.length, b.length);
  while (p < m && a[p] === b[p]) p++;
  let s = 0;
  while (s < m - p && a[a.length - 1 - s] === b[b.length - 1 - s]) s++;
  return { from: p, to: a.length - s, insert: b.slice(p, b.length - s) };
}

export async function applyConv(): Promise<void> {
  const res = bc.result; if (!res?.ok || !res.changes?.length) return;
  // eingebundene Dateien mit ungespeicherten Änderungen würden überschrieben: erst speichern oder verwerfen
  const dirty = (res.files ?? []).filter((f) => f.changed && f.file !== bc.rel && ed.tabs.some((x) => x.rel === f.file && x.dirty)).map((f) => f.file);
  if (dirty.length) { bc.error = t("basicconv-include-ungespeichert", { dateien: dirty.join(", ") }); return; }
  const view = getView(); const cur = textOf(bc.rel);
  if (!view || ed.active !== bc.rel || cur === null) { bc.error = t("basicconv-reiter-gewechselt"); return; }
  if (cur !== bc.text) { bc.error = t("basicconv-text-geaendert"); await refresh(); return; }
  bc.busy = true;
  try {
    const r = bc.kind === "renumber" ? { from: num(bc.from), to: num(bc.to) } : {};
    const done = await call("basic_convert", { kind: bc.kind, rel: bc.rel, text: bc.text, includes: bc.includes, apply: true, ...(r.from !== undefined && r.to !== undefined ? { from: r.from, to: r.to } : {}) });
    if (!done.ok || done.text === undefined) { bc.result = done; bc.error = done.error ?? ""; return; }
    const ch = minimalChange(cur, done.text);
    if (ch) view.dispatch({ changes: ch, userEvent: "input.convert", annotations: isolateHistory.of("full"), scrollIntoView: true });
    for (const f of done.written ?? []) if (ed.tabs.some((x) => x.rel === f)) await reloadFile(f);
    bc.open = false; view.focus();
    toast(t("basicconv-fertig", { zeilen: res.changes.length, dateien: (done.written?.length ?? 0) + 1 }));
  } catch (e) { fail(e); } finally { bc.busy = false; }
}
