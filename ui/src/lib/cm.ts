// SPDX-License-Identifier: GPL-3.0-or-later
// CodeMirror-6-Bausteine des Code-Editors (E1.1): Färbung und Faltbereiche kommen aus dem Kern (vc-ide-model), Rand mit Haltepunkten und Lesezeichen,
// Suchen/Ersetzen/Gehe zu Zeile mit Oberflächentexten aus dem Katalog. Der Text selbst und sein Verlauf liegen im EditorState.
import { acceptCompletion, closeBrackets, closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete";
import { cursorMatchingBracket, defaultKeymap, toggleLineComment, history, historyKeymap, indentLess, insertNewlineKeepIndent, insertTab } from "@codemirror/commands";
import { bracketMatching, foldGutter, foldKeymap, foldService, indentUnit } from "@codemirror/language";
import { highlightSelectionMatches, search, searchKeymap } from "@codemirror/search";
import { Compartment, EditorState, RangeSet, StateEffect, StateField, type Extension } from "@codemirror/state";
import { Decoration, type DecorationSet, drawSelection, EditorView, GutterMarker, gutter, highlightActiveLine, highlightActiveLineGutter, keymap, lineNumbers, placeholder } from "@codemirror/view";
import type { ColorLines, TextLang } from "./api";
import { formatOnLeave, type FormatHooks } from "./cm-format";
import { lspExtensions, type LspHooks } from "./cm-lsp";
import { t } from "./i18n.svelte";

/** Neue Färbung (vom Kern) einsetzen */
export const setColors = StateEffect.define<ColorLines>();
/** Neue Faltbereiche (Zeile ab 0 → letzte Zeile) einsetzen */
export const setFolds = StateEffect.define<[number, number][]>();
/** Zyklenzahl je Zeile (nach einem Bau): `[Zeile ab 1, kleinste, größte]` */
export const setCycles = StateEffect.define<[number, number, number][]>();
/** Durch `#if` ausgeschlossene Zeilen (ab 0) grau zeigen (natives BASIC) */
export const setExcluded = StateEffect.define<number[]>();
/** Zeile (ab 1) der Programmausführung bei einer Debug-Sitzung markieren; `null` löscht */
export const setDebugLine = StateEffect.define<number | null>();
const debugField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(v, tr) { v = v.map(tr.changes); for (const e of tr.effects) if (e.is(setDebugLine)) v = e.value && e.value >= 1 && e.value <= tr.state.doc.lines ? Decoration.set([Decoration.line({ class: "cm-debugline", attributes: { "data-debugline": "1" } }).range(tr.state.doc.line(e.value).from)]) : Decoration.none; return v; },
  provide: (f) => EditorView.decorations.from(f),
});
const toggleMark = StateEffect.define<{ line: number; kind: MarkKind }>();
const replaceMarks = StateEffect.define<{ breakpoints: number[]; bookmarks: number[] }>();

export type MarkKind = "breakpoint" | "bookmark";

const decoCache = new Map<string, Decoration>();
const deco = (kind: string) => { let d = decoCache.get(kind); if (!d) { d = Decoration.mark({ class: `tok-${kind}` }); decoCache.set(kind, d); } return d; };

const colorField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(v, tr) {
    v = v.map(tr.changes);
    for (const e of tr.effects) if (e.is(setColors)) {
      const doc = tr.state.doc; const out: ReturnType<ReturnType<typeof deco>["range"]>[] = [];
      for (const [i, toks] of e.value) {
        if (i + 1 > doc.lines) break;
        const ln = doc.line(i + 1);
        for (const [a, b, k] of toks) { const from = ln.from + Math.min(a, ln.length), to = ln.from + Math.min(b, ln.length); if (to > from) out.push(deco(k).range(from, to)); }
      }
      v = Decoration.set(out, true);
    }
    return v;
  },
  provide: (f) => EditorView.decorations.from(f),
});

const excludedLine = Decoration.line({ class: "cm-excluded" });
const excludedField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(v, tr) {
    v = v.map(tr.changes);
    for (const e of tr.effects) if (e.is(setExcluded)) {
      const doc = tr.state.doc;
      v = Decoration.set([...new Set(e.value)].filter((i) => i >= 0 && i < doc.lines).sort((a, b) => a - b).map((i) => excludedLine.range(doc.line(i + 1).from)), true);
    }
    return v;
  },
  provide: (f) => EditorView.decorations.from(f),
});

const foldField = StateField.define<Map<number, number>>({
  create: () => new Map(),
  update(v, tr) { for (const e of tr.effects) if (e.is(setFolds)) v = new Map(e.value); return v; },
});

const foldFromCore = foldService.of((state, lineStart, lineEnd) => {
  const last = state.field(foldField).get(state.doc.lineAt(lineStart).number - 1);
  if (last === undefined || last + 1 > state.doc.lines) return null;
  const to = state.doc.line(last + 1).to;
  return to > lineEnd ? { from: lineEnd, to } : null;
});

// Rand: Haltepunkte (roter Punkt) und Lesezeichen (Marke); je Zeile höchstens eines von beiden
class Mark extends GutterMarker {
  constructor(readonly kind: MarkKind) { super(); }
  eq(o: GutterMarker) { return o instanceof Mark && o.kind === this.kind; }
  toDOM() { const s = document.createElement("span"); s.className = `mk mk-${this.kind}`; s.setAttribute("aria-hidden", "true"); return s; }
}
const marks = { breakpoint: new Mark("breakpoint"), bookmark: new Mark("bookmark") };

const markField = StateField.define<RangeSet<Mark>>({
  create: () => RangeSet.empty,
  update(v, tr) {
    v = v.map(tr.changes);
    for (const e of tr.effects) {
      if (e.is(replaceMarks)) {
        const doc = tr.state.doc; const r = [] as ReturnType<Mark["range"]>[];
        for (const l of e.value.breakpoints) if (l >= 1 && l <= doc.lines) r.push(marks.breakpoint.range(doc.line(l).from));
        for (const l of e.value.bookmarks) if (l >= 1 && l <= doc.lines) r.push(marks.bookmark.range(doc.line(l).from));
        v = RangeSet.of(r, true);
      } else if (e.is(toggleMark)) {
        const pos = tr.state.doc.line(e.value.line).from; let had = false;
        v = v.update({ filter: (from, _to, val) => { if (from === pos && val.kind === e.value.kind) { had = true; return false; } return true; } });
        if (!had) v = v.update({ add: [marks[e.value.kind].range(pos)] });
      }
    }
    return v;
  },
});

// Rand: Zyklenzahl je Zeile (aus dem letzten Bau), klein und blass, bewegt sich mit dem Text
class Cyc extends GutterMarker {
  constructor(readonly text: string) { super(); }
  eq(o: GutterMarker) { return o instanceof Cyc && o.text === this.text; }
  toDOM() { const s = document.createElement("span"); s.className = "cyc"; s.textContent = this.text; return s; }
}
const cycField = StateField.define<RangeSet<Cyc>>({
  create: () => RangeSet.empty,
  update(v, tr) {
    v = v.map(tr.changes);
    for (const e of tr.effects) if (e.is(setCycles)) {
      const doc = tr.state.doc; const r = [] as ReturnType<Cyc["range"]>[];
      for (const [l, a, b] of e.value) if (l >= 1 && l <= doc.lines) r.push(new Cyc(a === b ? String(a) : `${a}–${b}`).range(doc.line(l).from));
      v = RangeSet.of(r, true);
    }
    return v;
  },
});

/** Zeilen (ab 1) der Haltepunkte und Lesezeichen eines Zustands */
export function readMarks(state: EditorState): { breakpoints: number[]; bookmarks: number[] } {
  const out = { breakpoints: [] as number[], bookmarks: [] as number[] };
  const c = state.field(markField).iter();
  while (c.value) { const l = state.doc.lineAt(c.from).number; (c.value.kind === "breakpoint" ? out.breakpoints : out.bookmarks).push(l); c.next(); }
  return out;
}
export function toggleLineMark(view: EditorView, kind: MarkKind, line?: number): void {
  const l = line ?? view.state.doc.lineAt(view.state.selection.main.head).number;
  view.dispatch({ effects: toggleMark.of({ line: l, kind }) });
}

/** Texte der Such-, Ersetzen- und Gehe-zu-Leiste (CodeMirror fragt sie über `phrases`) */
function phrases(): Record<string, string> {
  return {
    Find: t("cm-find"), Replace: t("cm-replace"), next: t("cm-next"), previous: t("cm-previous"), all: t("cm-all"), "match case": t("cm-match-case"),
    regexp: t("cm-regexp"), "by word": t("cm-by-word"), replace: t("cm-replace-one"), "replace all": t("cm-replace-all"), close: t("cm-close"),
    "current match": t("cm-current-match"), "on line": t("cm-on-line"), "replaced $ matches": t("cm-replaced-n"), "replaced match on line $": t("cm-replaced-one"),
    "Go to line": t("cm-goto-line"), go: t("cm-go"), "Folded lines": t("cm-folded-lines"), "Unfolded lines": t("cm-unfolded-lines"), to: t("cm-to"),
    "folded code": t("cm-folded-code"), unfold: t("cm-unfold"), "Fold line": t("cm-fold-line"), "Unfold line": t("cm-unfold-line"),
    "Selection deleted": t("cm-selection-deleted"), "Control character": t("cm-control-char"),
  };
}

/** Faltmarke im Rand: kräftiges Winkelsymbol (offen: nach unten, gefaltet: nach rechts, farbig hinterlegt) */
function foldMarker(open: boolean): HTMLElement {
  const s = document.createElement("span"); s.className = open ? "vc-fold" : "vc-fold vc-fold-closed"; s.title = open ? t("cm-fold-line") : t("cm-unfold-line");
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg"); svg.setAttribute("viewBox", "0 0 16 16"); svg.setAttribute("width", "14"); svg.setAttribute("height", "14"); svg.setAttribute("aria-hidden", "true");
  const p = document.createElementNS("http://www.w3.org/2000/svg", "path"); p.setAttribute("d", open ? "M3.5 6l4.5 4.5L12.5 6" : "M6 3.5l4.5 4.5L6 12.5"); p.setAttribute("fill", "none"); p.setAttribute("stroke", "currentColor"); p.setAttribute("stroke-width", "2.4"); p.setAttribute("stroke-linecap", "round"); p.setAttribute("stroke-linejoin", "round");
  svg.appendChild(p); s.appendChild(svg); return s;
}

const theme = EditorView.theme({
  "&": { height: "100%", color: "var(--text)", backgroundColor: "var(--codebg)", fontSize: "var(--ed-size, 13px)" },
  ".cm-scroller": { fontFamily: "var(--font-mono)", lineHeight: "1.55", fontVariantLigatures: "none" },
  ".cm-content": { caretColor: "var(--accent2)", padding: "6px 0" },
  ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--accent2)", borderLeftWidth: "2px" },
  "&.cm-focused": { outline: "none" },
  ".cm-gutters": { backgroundColor: "var(--codebg)", color: "var(--faint)", border: "none" },
  ".cm-excluded": { opacity: "0.42" },
  ".cm-activeLine": { backgroundColor: "var(--accentbg)" },
  ".cm-debugline": { backgroundColor: "rgba(250, 204, 21, 0.22)", boxShadow: "inset 3px 0 0 #facc15" },
  ".cm-activeLineGutter": { backgroundColor: "transparent", color: "var(--text)" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection": { backgroundColor: "var(--accentbg) !important" },
  ".cm-selectionMatch": { backgroundColor: "var(--line2)" },
  ".cm-matchingBracket": { outline: "1px solid var(--accent)", backgroundColor: "transparent" },
  ".cm-nonmatchingBracket": { outline: "1px solid var(--err)" },
  ".cm-foldPlaceholder": { backgroundColor: "var(--accentbg)", border: "1px solid var(--accent)", color: "var(--text)", fontWeight: "700", padding: "0 8px", margin: "0 4px", borderRadius: "4px", cursor: "pointer" },
  // Karten und Listen der Sprachdienste folgen dem Farbschema (Standardfarben von CodeMirror sind hell und gewinnen sonst)
  ".cm-tooltip": { backgroundColor: "var(--raise)", color: "var(--text)", border: "1px solid var(--line2)" },
  ".cm-tooltip.cm-tooltip-autocomplete > ul": { fontFamily: "var(--font-mono)", color: "var(--text)" },
  ".cm-tooltip.cm-tooltip-autocomplete > ul > li": { color: "var(--text)" },
  ".cm-tooltip.cm-tooltip-autocomplete > ul > li[aria-selected]": { backgroundColor: "var(--accentbg)", color: "var(--text)" },
  ".cm-completionDetail": { color: "var(--dim)" },
  ".cm-completionInfo": { backgroundColor: "var(--raise)", color: "var(--text)" },
  ".cm-panels": { backgroundColor: "var(--raise)", color: "var(--text)", borderBottom: "1px solid var(--line)" },
  ".cm-panel.cm-search": { padding: "6px 8px 6px 8px", fontFamily: "var(--font-ui)", fontSize: "12.5px" },
  ".cm-panel.cm-search input, .cm-panel.cm-search button": { fontFamily: "var(--font-ui)", fontSize: "12.5px", borderRadius: "6px", border: "1px solid var(--line2)", background: "var(--card)", color: "var(--text)", padding: "3px 8px" },
  ".cm-panel.cm-search input:focus-visible, .cm-panel.cm-search button:focus-visible": { boxShadow: "var(--focus)", outline: "none" },
  // Treffer der Suche: gelbe Fläche, Text darauf immer dunkel bzw. hell genug (WCAG AA), auch über gefärbtem Kommentar- und Befehlstext
  ".cm-searchMatch": { backgroundColor: "var(--find-bg)", outline: "1px solid var(--find-line)", borderRadius: "2px", color: "var(--text) !important", fontStyle: "normal !important" },
  ".cm-searchMatch span": { color: "var(--text) !important", fontStyle: "normal !important" },
  ".cm-searchMatch-selected": { backgroundColor: "var(--find-sel)", outline: "2px solid var(--text)", color: "var(--find-sel-text) !important" },
  ".cm-searchMatch-selected, .cm-searchMatch-selected span": { color: "var(--find-sel-text) !important", fontStyle: "normal !important" },
  ".cm-foldGutter .cm-gutterElement": { cursor: "pointer", display: "flex", alignItems: "flex-start", justifyContent: "center", paddingTop: "1px" },
  ".vc-fold": { display: "inline-flex", alignItems: "center", justifyContent: "center", width: "18px", height: "18px", color: "var(--dim)", borderRadius: "5px" },
  ".vc-fold:hover": { color: "var(--accent2)", backgroundColor: "var(--accentbg)" },
  ".vc-fold-closed": { color: "var(--accent2)", backgroundColor: "var(--accentbg)", boxShadow: "inset 0 0 0 1px var(--accent)" },
  ".cm-gutter-cyc": { color: "var(--faint)", fontSize: "11px", textAlign: "right", paddingRight: "4px" },
  ".cm-gutter-cyc .cyc": { opacity: "0.85" },
  ".cm-gutter-marks": { width: "16px" },
  ".cm-lineNumbers .cm-gutterElement": { cursor: "pointer" },
  ".cm-gutter-marks .cm-gutterElement": { cursor: "pointer", display: "flex", alignItems: "flex-start", justifyContent: "center", paddingTop: "5px" },
});

/** Zeilenumbruch (ansicht-weit, je Zustand umkonfigurierbar) */
export const wrapCompartment = new Compartment();
/** Sprachabhängiges: Texte der Suchleiste (`phrases`) und die Beschriftung des Eingabefelds; wird beim Sprachwechsel neu gesetzt */
export const langCompartment = new Compartment();
export const langExtension = (aria: string): Extension => [EditorState.phrases.of(phrases()), EditorView.contentAttributes.of({ "aria-label": aria, spellcheck: "false", autocorrect: "off", autocapitalize: "off", "aria-multiline": "true" })];
export const wrapExtension = (on: boolean): Extension => (on ? EditorView.lineWrapping : []);

export interface Hooks { onBreakpointClick: (line: number, shift: boolean) => void; onGoto: () => void; wrap: boolean; lsp: LspHooks; fmt: FormatHooks; commentToken: () => string | null }

/** Alle Erweiterungen eines Textes; `aria` ist die Beschriftung des Eingabefelds */
export function extensions(lang: TextLang, aria: string, hooks: Hooks): Extension {
  void lang;
  return [
    wrapCompartment.of(wrapExtension(hooks.wrap)), theme, colorField, excludedField, foldField, foldFromCore, markField, debugField,
    history(), drawSelection(), highlightActiveLine(), highlightActiveLineGutter(), highlightSelectionMatches(),
    lineNumbers({ domEventHandlers: { mousedown: (v, b, e) => { if ((e as MouseEvent).button !== 0) return false; e.preventDefault(); hooks.onBreakpointClick(v.state.doc.lineAt(b.from).number, (e as MouseEvent).shiftKey); return true; } } }),   // Klick auf die Zeilennummer setzt/entfernt den Haltepunkt
    cycField,
    gutter({ class: "cm-gutter-cyc", markers: (v) => v.state.field(cycField), initialSpacer: (v) => (v.state.field(cycField).size > 0 ? new Cyc("00–00") : new Cyc("")) }),
    gutter({ class: "cm-gutter-marks", markers: (v) => v.state.field(markField), initialSpacer: () => marks.breakpoint,
      domEventHandlers: { mousedown: (v, b, e) => { e.preventDefault(); hooks.onBreakpointClick(v.state.doc.lineAt(b.from).number, (e as MouseEvent).shiftKey); return true; } } }),
    foldGutter({ foldingChanged: (u) => u.transactions.some((tr) => tr.effects.some((e) => e.is(setFolds))), markerDOM: (open) => foldMarker(open) }),
    bracketMatching(), closeBrackets(),
    EditorState.languageData.of(() => [{ closeBrackets: { brackets: ["(", "[", "{", '"'], before: ")]}:;>,.'\" \t+-*/<=#&|" } }]),
    ...lspExtensions(hooks.lsp), formatOnLeave(hooks.fmt),
    EditorState.languageData.of(() => { const c = hooks.commentToken(); return c ? [{ commentTokens: { line: c } }] : []; }),
    indentUnit.of("\t"), EditorState.tabSize.of(4), langCompartment.of(langExtension(aria)),
    search({ top: true }),
    keymap.of([
      { key: "Enter", run: insertNewlineKeepIndent },
      { key: "Tab", run: (v) => acceptCompletion(v) || insertTab(v), shift: indentLess },
      { key: "Mod-F2", run: (v) => { toggleLineMark(v, "bookmark"); return true; } },
      { key: "Mod-Alt-k", run: (v) => { toggleLineMark(v, "bookmark"); return true; } },
      { key: "Mod-Alt-j", run: (v) => { jumpBookmark(v, 1); return true; } },
      { key: "Mod-Alt-h", run: (v) => { jumpBookmark(v, -1); return true; } },
      { key: "F2", run: (v) => { jumpBookmark(v, 1); return true; } },
      { key: "Shift-F2", run: (v) => { jumpBookmark(v, -1); return true; } },
      { key: "F9", run: (v) => { toggleLineMark(v, "breakpoint"); return true; } },
      { key: "Mod-/", run: toggleLineComment },
      { key: "Mod-Alt-c", run: toggleLineComment },
      { key: "Ctrl-m", run: cursorMatchingBracket },
      { key: "Mod-l", run: () => { hooks.onGoto(); return true; } },
      ...closeBracketsKeymap, ...completionKeymap, ...foldKeymap, ...searchKeymap, ...historyKeymap, ...defaultKeymap,
    ]),
    placeholder(""),
  ];
}

/** Zum nächsten (dir = 1) oder vorigen Lesezeichen springen, rundum */
export function jumpBookmark(view: EditorView, dir: 1 | -1): boolean {
  const lines = readMarks(view.state).bookmarks.sort((a, b) => a - b);
  if (!lines.length) return false;
  const cur = view.state.doc.lineAt(view.state.selection.main.head).number;
  const next = dir === 1 ? (lines.find((l) => l > cur) ?? lines[0]!) : ([...lines].reverse().find((l) => l < cur) ?? lines[lines.length - 1]!);
  const pos = view.state.doc.line(next).from;
  view.dispatch({ selection: { anchor: pos }, scrollIntoView: true });
  return true;
}

export function marksEffect(breakpoints: number[], bookmarks: number[]): StateEffect<unknown> { return replaceMarks.of({ breakpoints, bookmarks }); }
