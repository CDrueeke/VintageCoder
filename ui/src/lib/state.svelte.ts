// SPDX-License-Identifier: GPL-3.0-or-later
// Anzeigezustand der Oberfläche (Auswahl, Zoom, Fokus). Dokumente und Verlauf bleiben in Rust.
import { call, on, CoreError, type CommandInfo, type LangChoice, type OpenedProject } from "./api";
import { errorText, loadLanguage, t } from "./i18n.svelte";
import { MODES, SUBS, type Mode, type ModeDef, type Section } from "./nav";

export type Theme = "system" | "light" | "dark" | "contrast";
export type ExtraView = null | "gallery" | "measure" | "settings" | "wizard" | "help";

export interface Toast { id: number; text: string; error: boolean }

export const ui = $state({
  mode: "code" as Mode,
  sub: { projekt: "uebersicht", grafik: "sprites", daten: "disk" } as Record<string, string>,
  theme: "system" as Theme,
  zoom: 1,
  singleKeys: true,   // Einzeltasten der Editoren (Buchstaben/Ziffern: Werkzeuge, Noten); Cmd-/Alt-Kürzel gelten immer
  keyHints: true,     // Computertasten auf der Klaviatur anzeigen
  splash: true,       // Startbild beim Programmstart
  sidebar: true,
  zen: false,
  panel: false,   // Maschine-Panel rechts
  palette: false,
  extra: null as ExtraView,
  project: null as OpenedProject | null,
  selected: null as string | null,
  commands: [] as CommandInfo[],
  /** Release-Auswahl der Editoren: fehlt eine Kennung, gilt „an“ */ features: { all: true, on: {} as Record<string, boolean> },
  toasts: [] as Toast[],
  choice: "auto" as LangChoice,
  ready: false,
});

/** Ist der Editor im Release enthalten? (ohne Angabe: immer) */
export const featureOn = (f?: string): boolean => !f || ui.features.on[f] !== false;
/** Hauptbereiche und zweite Reihe nach der Release-Auswahl: abgewählte Editoren fehlen, ein Bereich ohne Rest auch */
export const visibleSubs = (m: Mode): Section[] => (SUBS[m] ?? []).filter((s) => !s.spaeter && featureOn(s.feature));
export const visibleModes = (): ModeDef[] => MODES.filter((m) => featureOn(m.feature) && (!SUBS[m.id] || visibleSubs(m.id).length > 0));

function store(key: string, value?: string): string | null {
  try { if (value !== undefined) localStorage.setItem(key, value); else return localStorage.getItem(key); } catch { /* ohne Speicher weiter */ }
  return null;
}

let nextToast = 1;
export function toast(text: string, error = false): void {
  const id = nextToast++;
  ui.toasts.push({ id, text, error });
  setTimeout(() => dismissToast(id), error ? 9000 : 4500);
}
export function dismissToast(id: number): void { ui.toasts = ui.toasts.filter((x) => x.id !== id); }
export function fail(e: unknown): void {
  if (e instanceof CoreError) toast(errorText(e.code, e.args), true);
  else toast(errorText("unbekannt", { code: String(e) }), true);
}

export function setMode(m: Mode): void { ui.mode = m; ui.extra = null; store("mode", m); }
export function setSub(m: Mode, id: string): void {
  if (SUBS[m]?.some((s) => s.id === id)) { ui.sub[m] = id; ui.mode = m; ui.extra = null; store(`sub-${m}`, id); }
}

export function applyTheme(): void {
  const r = document.documentElement;
  if (ui.theme === "system") r.removeAttribute("data-theme"); else r.setAttribute("data-theme", ui.theme);
}
export function setTheme(th: Theme): void { ui.theme = th; store("theme", th); applyTheme(); }
export function resolvedTheme(): "light" | "dark" {
  if (ui.theme === "contrast") return "dark";
  if (ui.theme !== "system") return ui.theme;
  return typeof matchMedia !== "undefined" && matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}
export function toggleTheme(): void { setTheme(resolvedTheme() === "dark" ? "light" : "dark"); }

export async function setLanguage(choice: LangChoice): Promise<void> {
  try {
    const r = await call("lang_set", { choice });
    ui.choice = choice;
    await loadLanguage(r.lang);
  } catch (e) { fail(e); }
}

export async function openProject(path: string): Promise<void> {
  try {
    ui.project = await call("project_open", { path });
    ui.selected = null;
    ui.mode = "code";
    ui.extra = null;
    await call("menu_context", { project: true, editor: false });
  } catch (e) { fail(e); }
}
export async function pickProject(): Promise<void> {
  try { const p = await call("project_pick"); if (p) await openProject(p); } catch (e) { fail(e); }
}
export async function closeProject(): Promise<void> {
  try { await call("project_close"); ui.project = null; ui.selected = null; } catch (e) { fail(e); }
}

/** Start: gespeicherte Wahl lesen, Sprache und Befehlstabelle laden, auf Ereignisse hören */
export async function init(): Promise<void> {
  const th = store("theme"); if (th === "light" || th === "dark" || th === "system" || th === "contrast") ui.theme = th;
  initZoom(); initPrefs();
  // Der zuletzt benutzte Bereich wird beim Start NICHT zurückgeholt: ohne Projekt gehört die Startseite („Code“) hin, mit Projekt setzt das Öffnen den Bereich selbst
  for (const k of ["projekt", "grafik", "daten"] as const) { const s = store(`sub-${k}`); if (s && SUBS[k]?.some((x) => x.id === s)) ui.sub[k] = s; }
  applyTheme();
  try {
    const info = await call("app_info");
    document.documentElement.dataset.platform = info.os === "macos" ? "mac" : info.os;
    const l = await call("lang_get");
    ui.choice = l.choice;
    await loadLanguage(l.lang);
    ui.commands = await call("commands_table");
    try { const f = await call("features_get"); ui.features = { all: f.all, on: Object.fromEntries(f.studios.map((x) => [x.id, x.on])) }; } catch { /* alles an */ }
  } catch (e) {
    if (e instanceof CoreError && e.code === "keine-verbindung") {
      document.documentElement.dataset.platform = /Mac/.test(navigator.platform) ? "mac" : "other";
    } else { console.error(e); }
  }
  await on("lang:changed", async (p) => { const lang = (p as { lang: "de" | "en" | "xx" }).lang; await loadLanguage(lang); try { ui.commands = await call("commands_table"); } catch { /* Tabelle bleibt */ } });
  ui.ready = true;
  void t; // Übersetzer ist mit loadLanguage bereit
}

/** Oberflächen-Zoom (0,7 bis 2,0), gemerkt; wirkt auf die ganze Oberfläche */
export function setZoom(z: number): void { ui.zoom = Math.round(Math.max(0.7, Math.min(2, z)) * 100) / 100; store("zoom", String(ui.zoom)); document.documentElement.style.setProperty("zoom", ui.zoom === 1 ? "" : String(ui.zoom)); }
export function setSingleKeys(on: boolean): void { ui.singleKeys = on; store("singleKeys", on ? "1" : "0"); }
export function setKeyHints(on: boolean): void { ui.keyHints = on; store("keyHints", on ? "1" : "0"); }
export function setSplash(on: boolean): void { ui.splash = on; store("splash", on ? "1" : "0"); }
/** Darf eine Taste ohne Cmd/Strg/Alt als Kürzel wirken? */
export const singleKeyOk = (e: KeyboardEvent): boolean => ui.singleKeys || e.ctrlKey || e.metaKey || e.altKey;
export function initPrefs(): void { const th = store("theme"); if (th === "light" || th === "dark" || th === "system" || th === "contrast") { ui.theme = th; applyTheme(); } ui.singleKeys = store("singleKeys") !== "0"; ui.keyHints = store("keyHints") !== "0"; ui.splash = store("splash") !== "0"; }
export function initZoom(): void { const z = Number(store("zoom")); if (z >= 0.7 && z <= 2) setZoom(z); }
initPrefs();   // früh lesen: das Startbild entscheidet beim ersten Zeichnen
