// SPDX-License-Identifier: GPL-3.0-or-later
// Selbstmessung für die Abnahme (nur wenn das Frontend mit VITE_SELFTEST=1 gebaut und der Kern mit Feature `selftest` gestartet wurde):
// misst Start und Befehls-Rundlauf im echten Fenster, schreibt das Ergebnis über den Kern in die Datei aus VC_SELFTEST_OUT und beendet die App.
import { call, callAny, on } from "./api";
import { openFile } from "./editor.svelte";
import { openProject, setTheme, ui } from "./state.svelte";

/** Sprachdienste im echten Fenster: Projekt öffnen, Text mit Fehler anmelden, Diagnose, Hover, Vervollständigung, Suche abwarten */
async function lspProbe(path: string): Promise<Record<string, unknown>> {
  const out: Record<string, unknown> = {};
  try {
    await call("project_open", { path });
    let diag: { items: { key: string; line: number }[] } | null = null;
    await on("lsp:diagnostics", (p) => { const ev = p as { rel: string; items: { key: string; line: number }[] }; if (ev.rel === "probe.asm") diag = ev; });
    const text = ".target c64\n.org $0801\nstart:   lda #$00\n         sta $d020\n         lda nirgendwo\n         jmp start\n";
    await call("doc_open", { rel: "probe.asm", text });
    for (let i = 0; i < 100 && !diag; i++) await new Promise((r) => setTimeout(r, 50));
    out.diag_count = (diag as { items: unknown[] } | null)?.items.length ?? 0; out.diag_key = (diag as { items: { key: string }[] } | null)?.items[0]?.key ?? null;
    const h = await call("lsp_hover", { rel: "probe.asm", line: 2, col: 10 });
    out.hover_has_table = !!h && h.markdown.includes("LDA") && h.markdown.includes("|");
    out.complete_count = (await call("lsp_complete", { rel: "probe.asm", line: 4, col: 11 })).length;
    out.outline = (await call("lsp_outline", { rel: "probe.asm" })).map((s) => s.name);
    out.fixes_ok = Array.isArray(await call("lsp_fixes", { rel: "probe.asm", from: 0, to: 5 }));
    out.format_changed = (await call("format_lines", { text: "start: lda   #1", rel: "probe.asm", lang: "asm", from: 0, to: 0 })).length;
    let done = false; let hits = 0;
    await on("search:hits", (p) => { hits += (p as { hits: unknown[] }).hits.length; }); await on("search:done", () => { done = true; });
    await call("search_start", { text: "lda", case: false, overrides: { "probe.asm": text } });
    for (let i = 0; i < 100 && !done; i++) await new Promise((r) => setTimeout(r, 50));
    out.search_done = done; out.search_hits = hits;
    out.ok = out.diag_count === 1 && out.hover_has_table === true && (out.complete_count as number) > 50 && done;
  } catch (e) { out.error = String(e); out.ok = false; }
  return out;
}

/** Fehlerkarte im echten Fenster: Datei mit unbekanntem Symbol öffnen, Maus (als Ereignis) auf die rote Wellenlinie, Karte auslesen; in Hell und Dunkel */
async function cardProbe(project: string): Promise<Record<string, unknown>> {
  const out: Record<string, unknown> = {};
  try {
    await openProject(project); ui.extra = null; ui.mode = "code";
    await call("file_create", { rel: "karte.asm" });
    await call("file_write", { rel: "karte.asm", text: ".target c64\n.org $0801\nstart:   lda #$00\n         lda nirgendwo\n         jmp start\n", eol: "lf" });
    await openFile("karte.asm");
    const wait = async (f: () => boolean, ms = 8000) => { for (let i = 0; i < ms / 50 && !f(); i++) await new Promise((r) => setTimeout(r, 50)); return f(); };
    out.wavy = await wait(() => !!document.querySelector(".cm-lintRange-error"));
    for (const scheme of ["light", "dark"] as const) {
      setTheme(scheme); await new Promise((r) => setTimeout(r, 300));
      const el = document.querySelector(".cm-lintRange-error"); if (!el) { out[scheme] = "no-wavy-line"; continue; }
      const r = el.getBoundingClientRect(); const x = r.left + 4, y = r.top + r.height / 2;
      const target = document.elementFromPoint(x, y) ?? el;
      target.dispatchEvent(new MouseEvent("mousemove", { bubbles: true, clientX: x, clientY: y }));
      const shown = await wait(() => !!document.querySelector(".cm-tooltip .vc-diag"), 4000);
      const c = document.querySelector(".cm-tooltip .vc-diag .vc-diag-msg"); const tip = document.querySelector(".cm-tooltip");
      out[scheme] = shown && c && tip ? { text: c.textContent, fg: getComputedStyle(c).color, bg: getComputedStyle(tip).backgroundColor } : "no-card";
      target.dispatchEvent(new MouseEvent("mouseout", { bubbles: true })); await new Promise((r) => setTimeout(r, 300));
    }
    out.ok = typeof out.light === "object" && typeof out.dark === "object";
  } catch (e) { out.error = String(e); out.ok = false; }
  return out;
}

export async function selftest(): Promise<void> {
  let cfg: unknown;
  try { cfg = await callAny("selftest_config", null); } catch { return; }
  if (!cfg) return;
  const ready = performance.now();
  while (!ui.ready) await new Promise((r) => setTimeout(r, 20));
  const readyMs = performance.now();
  const cfg2 = (await callAny("selftest_config", null)) as { ms_since_start: number; project?: string | null } | null;
  const lsp = cfg2?.project ? await lspProbe(cfg2.project) : null;
  const card = cfg2?.project ? await cardProbe(cfg2.project) : null;
  ui.extra = "measure";
  let run: (() => Promise<unknown>) | undefined;
  for (let i = 0; i < 100 && !run; i++) { await new Promise((r) => setTimeout(r, 50)); run = (window as unknown as { __vcMeasure?: () => Promise<unknown> }).__vcMeasure; }
  const result = cfg2?.project ? null : run ? await run() : null;   // mit Projekt (Sprachdienste/Karte) läuft das Fenster verborgen: keine Bildzeitmessung
  await callAny("selftest_report", { startup: { init_ms: readyMs - ready, ready_ms_since_navigation: readyMs, ready_ms_since_process_start: cfg2?.ms_since_start ?? null }, measure: result, lsp, card });
}
