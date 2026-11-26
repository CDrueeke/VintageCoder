// SPDX-License-Identifier: GPL-3.0-or-later
// Bauen und Starten (E1.3): Zustand des letzten Baus, Protokoll („Ausgabe“), Meldungen des Baus („Probleme“), Startziel. Der Kern baut im Hintergrund und meldet Ereignisse.
import { call, on, type BuildDiag, type BuildDone, type BuildInfo, type RunTargets } from "./api";
import { ed, lspNeuAbgleichen, refreshCycles, saveAll } from "./editor.svelte";
import { t } from "./i18n.svelte";
import { fail, toast } from "./state.svelte";
import { loadTurbo, showBanner } from "./turbo.svelte";

export interface LogLine { text: string; kind: "info" | "ok" | "warn" | "error"; file?: string; line?: number }
export const bau = $state({
  running: false, starting: false, started: false,
  last: null as null | { ok: boolean; rel: string; at: number; info: BuildInfo | null; errors: number; warnings: number },
  info: null as BuildInfo | null,
  log: [] as LogLine[],
  problems: [] as BuildDiag[],
  targets: null as RunTargets | null,
  sel: null as null | { file: string; line: number; text: string },   // gewähltes Problem (für „Springen“ und „Kopieren“)
});

const line = (text: string, kind: LogLine["kind"] = "info", at?: { file: string; line: number }): void => { bau.log.push({ text, kind, ...(at && at.line > 0 ? at : {}) }); if (bau.log.length > 2000) bau.log.splice(0, bau.log.length - 2000); };

/** Bauen; `aktiv`: die Quelle im aktiven Reiter, sonst die Hauptquelle; `then: "run"`: danach starten */
export async function buildRun(opts: { aktiv?: boolean; then?: "none" | "run" } = {}): Promise<void> {
  if (bau.running) { toast(t("err-bau-laeuft"), true); return; }
  await saveAll();
  if (ed.tabs.some((x) => x.dirty && !x.scratch)) { toast(t("bau-erst-speichern"), true); return; }
  const active = ed.active && !ed.tabs.find((x) => x.rel === ed.active)?.scratch ? ed.active : null;
  try { await call("build_start", { active, onlyActive: !!opts.aktiv, then: opts.then ?? "none" }); }
  catch (e) { fail(e); }
}

/** Das letzte Ergebnis starten (ohne neu zu bauen); gibt es keines: erst bauen */
export async function runLast(): Promise<void> {
  if (!bau.info) { toast(t("err-nichts-zu-starten"), true); return; }
  bau.starting = true;
  try { await call("run_start"); } catch (e) { bau.starting = false; fail(e); }
}
export async function runStop(): Promise<void> {
  try { const was = await call("run_stop"); toast(t(was ? "run-beendet" : "run-nichts-laeuft")); bau.started = false; } catch (e) { fail(e); }
}
export async function loadTargets(rel?: string): Promise<void> { try { bau.targets = await call("run_targets", { rel }); } catch { /* ohne Projekt keine Ziele */ } }
export async function setTarget(id: string): Promise<void> { try { await call("run_target_set", { id }); await loadTargets(); } catch (e) { fail(e); } }
export function clearLog(): void { bau.log = []; }
export const logText = (): string => bau.log.map((l) => l.text).join("\n");

let listening = false;
export async function startBau(): Promise<void> {
  if (listening) return; listening = true;
  await on("build:started", (p) => {
    const ev = p as { rel: string };
    bau.running = true; bau.problems = []; line(t("bau-log-bauen", { rel: ev.rel }));
  });
  await on("build:done", (p) => {
    const ev = p as BuildDone; const r = ev.result; bau.running = false;
    // Turbo BASIC XL: fehlt nur die Diskette, ist das ein Hinweis (Banner „Bereitstellen“), kein Fehler des Programms
    if (ev.turbo) void loadTurbo();
    if (ev.turbo === "missing") { line(r.error ?? t("turboui-log-fehlt"), "warn"); line(t("turboui-log-fehlt"), "warn"); bau.starting = false; showBanner(); ed.dock = "output"; return; }
    for (const st of r.log?.stages ?? []) { line(`▸ ${st.title}${st.status === "error" ? " ✗" : ""}`); for (const l of st.lines ?? []) line("   " + l, st.status === "error" ? "error" : "info"); }
    bau.problems = (r.diagnostics ?? []).filter((d) => d.code !== "W203");
    const lang = (r.diagnostics ?? []).filter((d) => d.code === "W203").length; if (lang > 0) line(t("bau-log-spruenge", { n: lang }));
    for (const d of bau.problems) line(`${d.file}:${d.line}: ${d.message}`, d.severity === "error" ? "error" : d.severity === "warning" ? "warn" : "info", { file: d.file, line: d.line });
    if (r.error) line(t("bau-log-fehler", { e: r.error }), "error");
    if (r.disk) line(t("bau-log-disk", { file: r.disk.file, size: r.disk.size, programs: r.disk.programs.join(", ") }));
    if (r.disk_error) line(t("bau-log-disk-fehler", { e: r.disk_error }), "error");
    const errors = bau.problems.filter((d) => d.severity === "error").length + (r.error ? 1 : 0) + (r.ok ? 0 : r.diagnostics.length === 0 && !r.error ? 1 : 0);
    const warnings = bau.problems.filter((d) => d.severity === "warning").length;
    lspNeuAbgleichen();
    if (ev.ok) {
      line(t("bau-log-fertig", { size: r.size ?? 0, artifact: r.artifact ?? "" }), "ok");
      void call("build_info").then((i) => { void refreshCycles(); bau.info = i; bau.last = { ok: true, rel: ev.rel, at: Date.now(), info: i, errors, warnings }; void loadTargets(); }, fail);
    } else {
      bau.last = { ok: false, rel: ev.rel, at: Date.now(), info: bau.info, errors, warnings };
      toast(t("bau-fehlgeschlagen", { n: errors }), true);
      ed.dock = "problems";
    }
  });
  if (typeof document !== "undefined") document.addEventListener("vc:turbo-bereitgestellt", (e) => { const d = (e as CustomEvent<{ disk: string; free: number }>).detail; line(t("turboui-log-bereitgestellt", { disk: d.disk, free: d.free }), "ok"); });
  await on("run:started", (p) => { bau.starting = false; bau.started = true; line(t("run-log-gestartet", { line: (p as { line: string }).line }), "ok"); });
  await on("run:failed", (p) => { bau.starting = false; bau.started = false; const errs = (p as { errors: string[] }).errors; for (const e of errs) line(t("run-log-fehler", { e }), "error"); toast(t("run-fehlgeschlagen", { e: errs[0] ?? "" }), true); ed.dock = "output"; });
  await on("run:ended", (p) => {
    const code = (p as { code: number | null }).code; bau.starting = false; bau.started = false;
    line(code === 0 || code === null ? t("run-log-ende") : t("run-log-ende-code", { code }), code === 0 || code === null ? "info" : "warn");
    if (code !== 0 && code !== null) toast(t("run-ende-code", { code }), true);
  });
  await on("run:stopped", () => { bau.started = false; line(t("run-log-beendet")); });
}
export function resetBau(): void { bau.running = false; bau.starting = false; bau.started = false; bau.last = null; bau.info = null; bau.log = []; bau.problems = []; bau.targets = null; }

/** Alle Meldungen (Bauergebnis und Sprachdienste) als Textzeilen `datei:zeile: Art Meldung` */
export function problemLines(lsp: Record<string, { line: number; severity: number; message: string }[]>): string[] {
  const out: string[] = [];
  for (const d of bau.problems) out.push(`${d.file}${d.line > 0 ? ":" + d.line : ""}: ${d.severity} ${d.message}`);
  for (const [rel, items] of Object.entries(lsp)) for (const d of items) if (d.severity <= 3) out.push(`${rel}:${d.line + 1}: ${["", "error", "warning", "info"][d.severity]} ${d.message}`);
  return out;
}
