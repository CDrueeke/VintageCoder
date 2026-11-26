// SPDX-License-Identifier: GPL-3.0-or-later
// CodeMirror-Anbindung der Sprachdienste (E1.2): Wellenlinien und Rand für Diagnosen, Karte beim Daraufzeigen (Fehlerkarte mit „Korrigieren“ und Befehlshilfe),
// Vervollständigung, Gehe zu Definition. Alles fragt den Kern (`lsp_*`); der Kern kennt Zeichen, hier gelten UTF-16-Einheiten.
import { autocompletion, type Completion, type CompletionContext, type CompletionResult } from "@codemirror/autocomplete";
import { isolateHistory } from "@codemirror/commands";
import { closeSearchPanel } from "@codemirror/search";
import { forEachDiagnostic, lintGutter, linter, nextDiagnostic, previousDiagnostic, type Diagnostic } from "@codemirror/lint";
import type { Extension } from "@codemirror/state";
import { closeHoverTooltips, EditorView, hoverTooltip, keymap, type Tooltip } from "@codemirror/view";
import { call, type LspFix, type LspHover, type LspItem, type TextLang } from "./api";
import { t } from "./i18n.svelte";
import { lspFlush } from "./lsp.svelte";
import { hoverKarte, renderMd, zerlegeHover } from "./md";

import { includePfadAn } from "./include-sprung";
export interface LspHooks {
  /** Pfad des gezeigten Reiters, wenn er Sprachdienste hat (Assembler), sonst null */
  rel(state?: import("@codemirror/state").EditorState): string | null;
  /** Pfad (bei unbenannten Reitern leer) und Art des gezeigten Reiters, wenn Karten und Zeichennamen möglich sind (Assembler, BASIC), sonst null */
  target(state?: import("@codemirror/state").EditorState): { rel: string; lang: TextLang } | null;
  /** Zu Datei, Zeile (ab 0) und Spalte springen */
  jump(rel: string, line: number, col: number): void;
  /** Befehl der Befehlstabelle ausführen (Kontextmenü) */
  run(id: string): void;
  /** Meldung zeigen (Fehler sichtbar) */
  say(text: string, error?: boolean): void;
  /** Fehler des Kerns zeigen */
  fail(e: unknown): void;
  /** Eingebundene Datei öffnen (Medium im Editor, Quelltext im Code-Editor) oder im Projektbaum zeigen; `from` ist die einbindende Datei */
  includeOpen(from: string, path: string, how: "open" | "reveal"): void;
}

const posOf = (view: EditorView, pos: number): { line: number; col: number } => { const ln = view.state.doc.lineAt(pos); return { line: ln.number - 1, col: pos - ln.from }; };

/** Quick-Fix anwenden: alle Änderungen in einem Schritt (ein Rückgängig) */
export function applyFix(view: EditorView, fix: LspFix): void {
  const doc = view.state.doc;
  const changes = fix.edits.filter((e) => e.line + 1 <= doc.lines).map((e) => { const ln = doc.line(e.line + 1); const le = doc.line(Math.min(e.endLine + 1, doc.lines)); return { from: ln.from + Math.min(e.from, ln.length), to: le.from + Math.min(e.to, le.length), insert: e.text }; });
  if (changes.length) view.dispatch({ changes, userEvent: "fix", annotations: isolateHistory.of("full") });   // eigener Rückgängig-Schritt (nicht mit dem Tippen davor verbunden)
}

export async function fixesAt(view: EditorView, rel: string, line: number): Promise<LspFix[]> {
  await lspFlush(rel);
  return call("lsp_fixes", { rel, from: line, to: line });
}

function card(): HTMLDivElement { const d = document.createElement("div"); d.className = "vc-card"; return d; }

function diagCard(view: EditorView, hooks: LspHooks, d: Diagnostic, fixes: LspFix[]): HTMLElement {
  const c = document.createElement("div"); const sevNr = ({ error: 1, warning: 2, info: 3, hint: 4 } as Record<string, number>)[d.severity] ?? 1; c.className = `vc-diag vc-diag-${sevNr}`;
  const h = document.createElement("div"); h.className = "vc-diag-head";
  const sev = document.createElement("span"); sev.className = "vc-diag-sev"; sev.textContent = t(`diag-sev-${sevNr}`);
  const code = document.createElement("span"); code.className = "vc-diag-code"; code.textContent = d.source ?? "";
  h.append(sev, code); c.appendChild(h);
  const m = document.createElement("div"); m.className = "vc-diag-msg"; m.textContent = d.message; c.appendChild(m);
  if (fixes.length) c.appendChild(fixRow(view, fixes));
  void hooks; return c;
}

function fixRow(view: EditorView, fixes: LspFix[]): HTMLElement {
  const row = document.createElement("div"); row.className = "vc-diag-actions";
  for (const f of fixes) {
    const b = document.createElement("button"); b.type = "button"; b.className = "btn"; b.textContent = f.title;
    b.onclick = () => { applyFix(view, f); view.focus(); };
    row.appendChild(b);
  }
  return row;
}

export interface MenuItem { label: string; run: () => void; disabled?: boolean }
/** Kleines Menü an einer Stelle (Pfeiltasten, Enter, Esc, Klick daneben schließt; der Fokus kehrt in den Editor zurück) */
export function popupMenu(view: EditorView | null, x: number, y: number, label: string, entries: MenuItem[]): void {
  const menu = document.createElement("div"); menu.className = "vc-fixmenu"; menu.setAttribute("role", "menu"); menu.setAttribute("aria-label", label);
  menu.style.left = `${Math.round(Math.min(x, window.innerWidth - 260))}px`; menu.style.top = `${Math.round(Math.min(y, window.innerHeight - 40 * entries.length - 16))}px`;
  const close = () => { menu.remove(); document.removeEventListener("pointerdown", away, true); view?.focus(); };
  const away = (e: Event) => { if (!menu.contains(e.target as Node)) close(); };
  const items: HTMLButtonElement[] = [];
  for (const en of entries) {
    const b = document.createElement("button"); b.type = "button"; b.setAttribute("role", "menuitem"); b.textContent = en.label; b.disabled = !!en.disabled;
    b.onclick = () => { close(); en.run(); };
    b.onkeydown = (e) => {
      const live = items.filter((i) => !i.disabled); const i = live.indexOf(b);
      if (e.key === "ArrowDown") { e.preventDefault(); live[(i + 1) % live.length]?.focus(); } else if (e.key === "ArrowUp") { e.preventDefault(); live[(i - 1 + live.length) % live.length]?.focus(); }
      else if (e.key === "Escape") { e.preventDefault(); close(); }
    };
    items.push(b); menu.appendChild(b);
  }
  document.body.appendChild(menu); document.addEventListener("pointerdown", away, true); items.find((i) => !i.disabled)?.focus();
}

/** Korrigieren (⌘.): Vorschläge zur Cursorzeile in einem kleinen Menü am Cursor; ohne Vorschlag eine sichtbare Meldung */
export async function quickFix(view: EditorView, hooks: LspHooks): Promise<void> {
  const rel = hooks.rel(); if (!rel) { hooks.say(t("lsp-keine-dienste"), true); return; }
  const sel = view.state.selection.main; const doc = view.state.doc;
  const from = doc.lineAt(sel.from).number - 1, to = doc.lineAt(sel.to).number - 1;
  let fixes: LspFix[];
  try { await lspFlush(rel); fixes = await call("lsp_fixes", { rel, from, to }); } catch (e) { hooks.fail(e); return; }
  if (!fixes.length) { hooks.say(t("lsp-keine-korrektur"), true); return; }
  const at = view.coordsAtPos(sel.head) ?? { left: 100, bottom: 100 };
  popupMenu(view, at.left, at.bottom + 4, t("cmd-korrigieren"), fixes.map((f) => ({ label: f.title, run: () => applyFix(view, f) })));
}

function hover(hooks: LspHooks): Extension {
  return hoverTooltip(async (view, pos): Promise<Tooltip | null> => {
    const rel = hooks.rel(view.state); const tg = hooks.target(view.state); if (!tg) return null;
    const diags: Diagnostic[] = []; let dfrom = pos, dto = pos;
    forEachDiagnostic(view.state, (d, from, to) => { if (pos >= from && pos <= Math.max(to, from + 1)) { diags.push(d); dfrom = Math.min(dfrom === pos ? from : dfrom, from); dto = Math.max(dto, to); } });
    let info: LspHover | null = null; let fixes: LspFix[] = [];
    const p = posOf(view, pos);
    // Die Fehlerkarte darf nie an der Befehlshilfe hängen: scheitert eine Anfrage, bleibt die Meldung der Wellenlinie trotzdem stehen (und der Fehler wird sichtbar gemeldet)
    try {
      if (rel) { await lspFlush(rel); info = await call("lsp_hover", { rel, line: p.line, col: p.col }); }
    } catch (e) { hooks.fail(e); }
    try { if (rel) fixes = await call("lsp_fixes", { rel, from: p.line, to: p.line }); } catch (e) { hooks.fail(e); }
    try {
      // Zeichenangaben {clr} in Zeichenketten und Zahlen, Register und Wertzerlegung im BASIC kommen nicht vom Sprachdienst
      if (!info) info = await call("text_hover", { rel: tg.rel, lang: tg.lang, text: view.state.doc.toString(), line: p.line, col: p.col });
    } catch (e) { hooks.fail(e); }
    if (!diags.length && !info) return null;
    const hoverFixes = diags.length ? [] : fixes;   // Vorschläge ohne Meldung (alter Direktivenname) hängen an der Befehlshilfe
    let from = pos, to = pos;
    if (info) { const doc = view.state.doc; if (info.line + 1 <= doc.lines) { const ln = doc.line(info.line + 1); from = ln.from + Math.min(info.from, ln.length); to = ln.from + Math.min(info.to, ln.length); } }
    if (diags.length) { from = Math.min(from, dfrom); to = Math.max(to, dto); }
    return { pos: from, end: to, above: true, create: () => {
      const dom = card();
      for (const d of diags) dom.appendChild(diagCard(view, hooks, d, fixes));
      if (info) dom.appendChild(hoverKarte(info.markdown, t("hover-karte-label", { name: zerlegeHover(info.markdown).title || t("hover-karte-hilfe") })));
      if (hoverFixes.length) dom.appendChild(fixRow(view, hoverFixes));
      return { dom };
    } };
  }, { hoverTime: 350 });
}

const KIND: Record<number, string> = { 14: "keyword", 5: "property", 6: "variable", 21: "constant", 3: "function" };

function toCompletion(it: LspItem): Completion {
  const first = (it.sortText ?? "5")[0] ?? "5";
  const boost = ({ "0": 20, "1": 10, "2": 0, "3": -10, "9": -40 } as Record<string, number>)[first] ?? 0;
  const c: Completion = { label: it.label, type: KIND[it.kind ?? 0] ?? "text", boost };
  if (it.detail) c.detail = it.detail;
  if (it.documentation) { const doc = it.documentation; c.info = () => { const d = document.createElement("div"); d.className = "vc-cinfo"; d.appendChild(renderMd(doc)); return d; }; }
  return c;
}

function completionSource(hooks: LspHooks) {
  return async (ctx: CompletionContext): Promise<CompletionResult | null> => {
    const rel = hooks.rel(ctx.state); const tg = hooks.target(ctx.state); if (!tg) return null;
    const line = ctx.state.doc.lineAt(ctx.pos);
    const before = line.text.slice(0, ctx.pos - line.from);
    // Nach `{` in einer Zeichenkette: Namen der Zeichen und Farben (`{clr}`, `{rvs on}` …)
    const br = /\{[\w $*-]*$/.exec(before);
    if (br && (before.match(/"/g)?.length ?? 0) % 2 === 1) {
      try {
        const names = await call("char_names", { rel: tg.rel, lang: tg.lang, line_text: line.text, col: ctx.pos - line.from });
        if (ctx.aborted || !names.length) return null;
        return { from: line.from + br.index + 1, options: names.map(([label, detail], i) => ({ label, detail, type: "constant", boost: -i / 1000 + 5, apply: (v: EditorView, _c: Completion, from: number, to: number) => { const next = v.state.sliceDoc(to, to + 1) === "}"; v.dispatch({ changes: { from, to, insert: next ? label : label + "}" }, selection: { anchor: from + label.length + 1 }, userEvent: "input.complete" }); } })), validFor: /^[\w $*-]*$/ };
      } catch (e) { hooks.fail(e); return null; }
    }
    if (!rel) return null;
    const w = ctx.matchBefore(/[\w.$@]*/); if (!w || (w.from === w.to && !ctx.explicit)) return null;
    if (before.includes(";") && !/["']/.test(before)) return null;   // im Kommentar nichts vorschlagen
    try {
      await lspFlush(rel);
      const items = await call("lsp_complete", { rel, line: line.number - 1, col: ctx.pos - line.from });
      if (ctx.aborted) return null;
      const typed = w.text;
      // Alte Namen von Direktiven erscheinen erst, wenn man sie schon tippt (und stehen weit unten); die kanonischen kommen zuerst
      const keep = items.filter((i) => !(i.sortText ?? "").startsWith("9") || (typed.length >= 2 && i.label.toLowerCase().startsWith(typed.toLowerCase())));
      return { from: w.from, options: keep.map(toCompletion), validFor: /^[\w.$@]*$/ };
    } catch (e) { hooks.fail(e); return null; }
  };
}

/** Zum Namen unter dem Cursor (oder an `pos`) springen; ohne Fund eine sichtbare Meldung */
export async function gotoDefinition(view: EditorView, hooks: LspHooks, at?: number): Promise<void> {
  const rel = hooks.rel(); if (!rel) { hooks.say(t("lsp-keine-dienste"), true); return; }
  const p = posOf(view, at ?? view.state.selection.main.head);
  try {
    await lspFlush(rel);
    const d = await call("lsp_definition", { rel, line: p.line, col: p.col });
    if (!d) { hooks.say(t("lsp-keine-definition"), true); return; }
    hooks.jump(d.rel, d.line, d.col);
  } catch (e) { hooks.fail(e); }
}

/** Alle Erweiterungen der Sprachdienste für einen Editorzustand */
export function lspExtensions(hooks: LspHooks): Extension[] {
  return [
    // Die Diagnosen kommen vom Kern; CodeMirror zeichnet nur Wellenlinie und Rand, die Karte macht `hover`
    linter(null, { tooltipFilter: () => [] }),
    lintGutter({ tooltipFilter: () => [], hoverTime: 1e9 }),
    hover(hooks),
    autocompletion({ override: [completionSource(hooks)], activateOnTyping: true, activateOnTypingDelay: 120, icons: true, maxRenderedOptions: 60 }),
    EditorView.domEventHandlers({
      contextmenu: (e, view) => {
        if (e.button !== 2 && e.type !== "contextmenu") return false;
        e.preventDefault(); closeSearchPanel(view);       // die Suchleiste schließt, wenn man im Text rechtsklickt
        const pos = view.posAtCoords({ x: e.clientX, y: e.clientY });
        const sel = view.state.selection.main;
        if (pos !== null && (pos < sel.from || pos > sel.to)) view.dispatch({ selection: { anchor: pos } });
        const has = !view.state.selection.main.empty;
        const inc = pos !== null ? includePfadAn(view, pos) : null, von = hooks.target(view.state)?.rel ?? "";
        popupMenu(view, e.clientX, e.clientY, t("editor-kontextmenu"), [
          ...(inc && von ? [
            { label: t("include-oeffnen"), run: () => hooks.includeOpen(von, inc, "open") },
            { label: t("include-im-baum-zeigen"), run: () => hooks.includeOpen(von, inc, "reveal") },
          ] : []),
          { label: t("cmd-ausschneiden"), disabled: !has, run: () => { view.focus(); document.execCommand("cut"); } },
          { label: t("cmd-kopieren"), disabled: !has, run: () => { view.focus(); document.execCommand("copy"); } },
          { label: t("cmd-einfuegen"), run: () => { navigator.clipboard.readText().then((x) => { view.dispatch(view.state.replaceSelection(x), { userEvent: "input.paste", scrollIntoView: true }); }, (er) => hooks.fail(er)); } },
          { label: t("cmd-alle_auswaehlen"), run: () => hooks.run("alle_auswaehlen") },
          { label: t("cmd-gehe_zu_definition"), disabled: !hooks.rel(), run: () => hooks.run("gehe_zu_definition") },
          { label: t("cmd-korrigieren"), disabled: !hooks.rel(), run: () => hooks.run("korrigieren") },
          { label: t("cmd-kommentar_umschalten"), run: () => hooks.run("kommentar_umschalten") },
          { label: t("cmd-gehe_zu_zeile"), run: () => hooks.run("gehe_zu_zeile") },
          { label: t("cmd-lesezeichen_umschalten"), run: () => hooks.run("lesezeichen_umschalten") },
          { label: t("cmd-alles_falten"), run: () => hooks.run("alles_falten") },
          { label: t("cmd-alles_entfalten"), run: () => hooks.run("alles_entfalten") },
          { label: t("cmd-haltepunkt_umschalten"), run: () => hooks.run("haltepunkt_umschalten") },
          ...(hooks.target(view.state)?.lang === "basic" && hooks.target(view.state)?.rel ? [
            { label: t("cmd-basic_in_labels"), run: () => hooks.run("basic_in_labels") },
            { label: t("cmd-basic_in_nummern"), run: () => hooks.run("basic_in_nummern") },
            { label: t("cmd-basic_neu_nummerieren"), run: () => hooks.run("basic_neu_nummerieren") },
          ] : []),
        ]);
        return true;
      },
      dblclick: (e, view) => {
        if (e.button !== 0) return false;
        const pos = view.posAtCoords({ x: e.clientX, y: e.clientY }); const von = hooks.target(view.state)?.rel;
        const inc = pos !== null ? includePfadAn(view, pos) : null; if (!inc || !von) return false;   // sonst wählt der Doppelklick wie bisher das Wort
        e.preventDefault(); hooks.includeOpen(von, inc, "open"); return true;
      },
      mousedown: (e, view) => {
        if (!(e.metaKey || e.ctrlKey) || e.button !== 0) return false;
        const pos = view.posAtCoords({ x: e.clientX, y: e.clientY }); if (pos === null) return false;
        e.preventDefault(); void gotoDefinition(view, hooks, pos); return true;
      },
    }),
    keymap.of([
      { key: "F12", run: (v) => { void gotoDefinition(v, hooks); return true; } },
      { key: "F8", run: nextDiagnostic, shift: previousDiagnostic },
      { key: "Mod-.", run: (v) => { void quickFix(v, hooks); return true; } },
      { key: "Escape", run: (v) => { if (!document.querySelector(".cm-tooltip-hover")) return false; v.dispatch({ effects: closeHoverTooltips }); return true; } },   // Esc schließt die Hover-Karte (sonst bleibt Esc dem Editor)
    ]),
  ];
}
