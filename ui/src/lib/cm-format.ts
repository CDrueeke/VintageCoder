// SPDX-License-Identifier: GPL-3.0-or-later
// Prettyprint im Editor (E1.2): die bearbeitete Zeile wird beim Verlassen formatiert (wählbar), Zeile/Auswahl und ganze Datei auf Befehl (je ein Rückgängig-Schritt).
// Die Regeln stehen im Kern (`vc-ide-model::pretty`, Befehl `format_lines`); hier wird nur angewendet.
import { isolateHistory } from "@codemirror/commands";
import { StateField, type Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { call, type TextLang } from "./api";

export interface FormatHooks {
  /** Pfad und Art des gezeigten Reiters, wenn formatiert werden kann (Assembler, BASIC), sonst null */
  target(state?: import("@codemirror/state").EditorState): { rel: string; lang: TextLang } | null;
  /** Zeile beim Verlassen formatieren? */
  onLeave(): boolean;
  fail(e: unknown): void;
}

/** Zeilen `from..=to` (ab 0) des Editors formatieren; `true`, wenn sich etwas geändert hat. Ein Rückgängig-Schritt, Cursor und Auswahl bleiben. */
export async function formatLines(view: EditorView, hooks: FormatHooks, from: number, to: number): Promise<boolean> {
  const tg = hooks.target(view.state); if (!tg) return false;
  const doc = view.state.doc;
  try {
    const edits = await call("format_lines", { text: doc.toString(), rel: tg.rel, lang: tg.lang, from, to });
    // Wurde inzwischen weitergetippt, gilt die Antwort für jede Zeile weiter, deren Text noch derselbe ist (Zeilen oberhalb des Cursors ändern ihre Nummer nicht)
    const cur = view.state.doc;
    const changes = edits.filter(([l]) => l + 1 <= doc.lines && l + 1 <= cur.lines && cur.line(l + 1).text === doc.line(l + 1).text).map(([l, text]) => { const ln = cur.line(l + 1); return { from: ln.from, to: ln.to, insert: text }; });
    if (!changes.length) return false;
    view.dispatch({ changes, userEvent: "format", annotations: isolateHistory.of("full") });   // eigener Rückgängig-Schritt, auch kurz nach dem Tippen
    return true;
  } catch (e) { hooks.fail(e); return false; }
}

export const formatLine = (view: EditorView, hooks: FormatHooks): Promise<boolean> => {
  const s = view.state.selection.main; const d = view.state.doc;
  return formatLines(view, hooks, d.lineAt(s.from).number - 1, d.lineAt(s.to).number - 1);
};
export const formatFile = (view: EditorView, hooks: FormatHooks): Promise<boolean> => formatLines(view, hooks, 0, view.state.doc.lines - 1);

/** Zeile, in der zuletzt getippt wurde (nur Änderungen des Textes setzen sie) */
const editedLine = StateField.define<number | null>({
  create: () => null,
  update(v, tr) {
    if (tr.docChanged && (tr.isUserEvent("undo") || tr.isUserEvent("redo"))) return null;   // Rückgängig/Wiederholen formatiert nichts neu
    return tr.docChanged && !tr.isUserEvent("format") ? tr.state.doc.lineAt(tr.state.selection.main.head).number : v;
  },
});

/** Beim Wechsel der Cursorzeile die zuletzt bearbeitete Zeile formatieren (auch beim Verlassen mit Enter; unabhängig davon, ob die Zeile einen Fehler hat) */
export function formatOnLeave(hooks: FormatHooks): Extension {
  return [editedLine, EditorView.updateListener.of((u) => {
    if (!hooks.onLeave()) return;
    const view = u.view;
    if (u.docChanged) {
      // Enter (oder Einfügen mehrerer Zeilen) verlässt die bearbeitete Zeile im selben Schritt: ihre neue Nummer über die Änderung mitführen
      if (u.transactions.some((tr) => tr.isUserEvent("format") || tr.isUserEvent("undo") || tr.isUserEvent("redo"))) return;
      const prev = u.startState.field(editedLine); if (prev === null || prev > u.startState.doc.lines) return;
      const old = u.state.doc.lineAt(u.changes.mapPos(u.startState.doc.line(prev).from, -1)).number;
      const cur = u.state.doc.lineAt(u.state.selection.main.head).number;
      if (cur !== old) queueMicrotask(() => { void formatLines(view, hooks, old - 1, old - 1); });
      return;
    }
    if (!u.selectionSet) return;
    const ed = u.state.field(editedLine); if (ed === null) return;
    const cur = u.state.doc.lineAt(u.state.selection.main.head).number;
    if (cur === ed) return;
    queueMicrotask(() => { void formatLines(view, hooks, ed - 1, ed - 1); });
  })];
}

/** Zeilen der Auswahl (ohne Auswahl: bei `whole` die ganze Datei) als `[erste, letzte]` ab 1 */
function covered(view: EditorView, whole: boolean): [number, number] | null {
  const s = view.state.selection.main; const d = view.state.doc;
  if (s.empty) return whole ? [1, d.lines] : null;
  return [d.lineAt(s.from).number, d.lineAt(s.to).number];
}

/** Leerzeichen und Tabulatoren am Zeilenende entfernen (Auswahl oder Datei), ein Rückgängig-Schritt; keine Automatik beim Speichern (Byte-Gleichheit). `true`, wenn sich etwas änderte */
export function trimTrailing(view: EditorView): boolean {
  const r = covered(view, true)!; const d = view.state.doc; const changes = [];
  for (let n = r[0]; n <= r[1]; n++) { const ln = d.line(n); const m = /[ \t]+$/.exec(ln.text); if (m) changes.push({ from: ln.to - m[0].length, to: ln.to }); }
  if (!changes.length) return false;
  view.dispatch({ changes, userEvent: "format", annotations: isolateHistory.of("full") }); return true;
}

/** Markierte Zeilen sortieren (nach Zeichencode des Textes ohne Einrückung, ein Rückgängig-Schritt); ohne Auswahl über mehrere Zeilen `null` (nichts getan) */
export function sortLines(view: EditorView): boolean | null {
  const r = covered(view, false); if (!r || r[0] === r[1]) return null;
  const d = view.state.doc; const lines: string[] = []; for (let n = r[0]; n <= r[1]; n++) lines.push(d.line(n).text);
  const key = (l: string) => l.trim();   // Einrückung zählt nicht mit; gleiche Schlüssel behalten ihre Reihenfolge
  const sorted = [...lines].sort((a, b) => (key(a) < key(b) ? -1 : key(a) > key(b) ? 1 : 0));
  if (sorted.every((l, i) => l === lines[i])) return false;
  view.dispatch({ changes: { from: d.line(r[0]).from, to: d.line(r[1]).to, insert: sorted.join("\n") }, userEvent: "format", annotations: isolateHistory.of("full") }); return true;
}
