// SPDX-License-Identifier: GPL-3.0-or-later
// Markdown im Editor (E13): Vorschau/Quelltext je Reiter, „als Markdown behandeln“, Schreibhilfen (Kern `vc_md::edit`, Positionen in Zeichen, nicht UTF-16).
import { call } from "./api";
import { ed, getView } from "./editor.svelte";
import { fail, toast } from "./state.svelte";
import { t } from "./i18n.svelte";

export const md = $state({ quelltext: {} as Record<string, boolean>, als: {} as Record<string, boolean>, blocks: 200 });
/** Behandelt der Reiter die Datei als Markdown? (Endung/Art oder ausdrücklich gewählt) */
export const istMarkdown = (rel: string, lang: string): boolean => lang === "markdown" || !!md.als[rel];
export const vorschauAn = (rel: string, lang: string): boolean => istMarkdown(rel, lang) && !md.quelltext[rel];
const aktivTab = () => ed.tabs.find((x) => x.rel === ed.active);

export function mdUmschalten(): void { const tb = aktivTab(); if (!tb || !istMarkdown(tb.rel, tb.lang)) { toast(t("md-keine-markdown-datei"), true); return; } md.quelltext[tb.rel] = !md.quelltext[tb.rel]; }
export function mdAls(): void { const tb = aktivTab(); if (!tb) return; md.als[tb.rel] = !md.als[tb.rel]; if (md.als[tb.rel]) md.quelltext[tb.rel] = false; }

const OPS: Record<string, string> = { md_fett: "bold", md_kursiv: "italic", md_code: "code", md_durchgestrichen: "strike", md_link: "link", md_bild: "image", md_tiefer: "deeper", md_hoeher: "higher",
  md_aufzaehlung: "bullet", md_nummerierung: "numbered", md_aufgaben: "tasks", md_zitat: "quote", md_aufgabe: "task", md_tabelle: "table", md_ausrichten: "align" };
export const mdBefehle = Object.keys(OPS);
const cp = (s: string, i: number): number => [...s.slice(0, i)].length;           // UTF-16-Index → Zeichenindex
const u16 = (s: string, c: number): number => { let i = 0, n = 0; for (const ch of s) { if (n === c) break; i += ch.length; n++; } return n < c ? s.length : i; };

/** Schreibhilfe auf den Text des aktiven Reiters (nur im Quelltext): ein Rückgängig-Schritt, Auswahl wie vom Kern gemeldet */
export async function mdBefehl(id: string): Promise<void> {
  const tb = aktivTab(); const view = getView(); const op = OPS[id];
  if (!op || !tb || !view || !istMarkdown(tb.rel, tb.lang)) { toast(t("md-keine-markdown-datei"), true); return; }
  if (!md.quelltext[tb.rel]) { md.quelltext[tb.rel] = true; await new Promise((r) => setTimeout(r, 60)); }   // Schreibhilfen wirken im Quelltext
  const text = view.state.doc.toString(); const sel = view.state.selection.main;
  try {
    const r = await call("md_edit", { op, text, a: cp(text, sel.from), b: cp(text, sel.to) });
    if (!r) { toast(t("md-hier-nichts"), false); return; }
    let p = 0; const max = Math.min(text.length, r.text.length); while (p < max && text[p] === r.text[p]) p++;
    let q = 0; while (q < max - p && text[text.length - 1 - q] === r.text[r.text.length - 1 - q]) q++;
    view.dispatch({ changes: { from: p, to: text.length - q, insert: r.text.slice(p, r.text.length - q) }, selection: { anchor: u16(r.text, r.a), head: u16(r.text, r.b) }, userEvent: "input", scrollIntoView: true });
    view.focus();
  } catch (e) { fail(e); }
}
/** Aufgabe in der Vorschau abhaken: schaltet das Kästchen in der Quellzeile (Zeichenindex des Zeilenanfangs) um */
export async function mdAufgabe(rel: string, zeile: number): Promise<void> {
  const view = getView(); if (!view || ed.active !== rel || zeile >= view.state.doc.lines) return;
  const l = view.state.doc.line(zeile + 1); const text = view.state.doc.toString();
  try {
    const r = await call("md_edit", { op: "task", text, a: cp(text, l.from), b: cp(text, l.from) });
    if (!r) return;
    let p = 0; const max = Math.min(text.length, r.text.length); while (p < max && text[p] === r.text[p]) p++;
    let q = 0; while (q < max - p && text[text.length - 1 - q] === r.text[r.text.length - 1 - q]) q++;
    view.dispatch({ changes: { from: p, to: text.length - q, insert: r.text.slice(p, r.text.length - q) }, userEvent: "input" });
  } catch (e) { fail(e); }
}
