// SPDX-License-Identifier: GPL-3.0-or-later
// Dateien aus dem Dateimanager in das Fenster ziehen (Tauris natives Drop-Ereignis): jede Datei kommt nach Endung in den passenden Editor.
// Der Kern plant (`drop_plan`), hier wird ausgeführt: Projekt öffnen, Datei im Projekt öffnen, Diskettenabbild öffnen; Dateien außerhalb des Projekts erst nach Rückfrage kopieren.
import { call, type DropItem } from "./api";
import { dk, diskOp, openDisk } from "./disk.svelte";
import { confirmLeave, openFile, reloadTree } from "./editor.svelte";
import { flushSession } from "./sitzung.svelte";
import { t } from "./i18n.svelte";
import { fail, openProject, toast, ui } from "./state.svelte";
import { cfg } from "./settings.svelte";

export const drop = $state({ over: false, ask: null as null | { items: DropItem[] } });

// Aufträge (gezogene oder übergebene Pfade) laufen strikt nacheinander: ein zweiter Aufruf der Anwendung, der mitten im Öffnen des ersten ankommt, würde sonst parallel ein anderes
// Projekt öffnen (der zuerst begonnene Auftrag gewänne oder beide störten sich)
let serie: Promise<unknown> = Promise.resolve();
const nacheinander = <T,>(f: () => Promise<T>): Promise<T> => { const r = serie.then(f, f); serie = r.catch(() => undefined); return r; };

const wartend: string[][] = [];   // Pfade, die während der Rückfrage ankamen (Öffnen mit, zweiter Start): sie gehen nicht verloren, sondern kommen nach der Antwort an die Reihe
/** Gezogene Pfade ausführen. Mehrere Dateien: nacheinander; ein Projekt hat Vorrang (alles andere entfällt) */
export const handleDrop = (paths: string[]): Promise<void> => nacheinander(() => handleDropEcht(paths));
async function handleDropEcht(paths: string[]): Promise<void> {
  drop.over = false; if (!paths.length) return;
  if (drop.ask) { wartend.push(paths); return; }
  if (ui.extra === "settings" && cfg.page === "roms") { const { romsAbgelegt } = await import("./settings.svelte"); await romsAbgelegt(paths); return; }   // ROM-Seite der Einstellungen: ROM-Import statt Projektablage
  const imDisk = ui.mode === "daten" && ui.sub.daten === "disk";
  if (imDisk && dk.view?.fs && !paths.some((p) => /\.(d64|d71|d81|atr)$/i.test(p))) { await diskOp({ kind: "add_files", paths }); return; }   // Dateien auf eine offene Diskette ziehen: als Einträge hinzufügen
  let plan: DropItem[]; try { plan = await call("drop_plan", { paths }); } catch (e) { fail(e); return; }
  const proj = plan.find((x) => x.action === "project");
  if (proj) { if (await confirmLeave()) { await flushSession(); await openProject(proj.path); } return; }
  const copies: DropItem[] = [];
  for (const it of plan) {
    if (it.action === "open" && it.rel) await openFile(it.rel);
    else if (it.action === "disk") await openDisk({ path: it.path });
    else if (it.action === "copy") copies.push(it);
    else toast(imDisk && !dk.view && it.reason !== "folder" && it.reason !== "missing" ? t("disk-zuerst-diskette") : it.reason === "no-project" ? t("ablegen-kein-projekt", { name: it.name }) : it.reason === "folder" ? t("ablegen-ordner", { name: it.name }) : it.reason === "missing" ? t("ablegen-fehlt", { name: it.name }) : t("ablegen-typ", { name: it.name }), true);
  }
  if (copies.length) drop.ask = { items: copies };
}
/** Antwort auf „Ins Projekt kopieren und öffnen?“ */
export async function answerDrop(copy: boolean): Promise<void> {
  const a = drop.ask; drop.ask = null; if (!a) return;
  if (copy) {
    const rels: string[] = [];
    for (const it of a.items) { try { rels.push(await call("drop_copy", { path: it.path })); } catch (e) { fail(e); } }
    if (rels.length) { await reloadTree(); for (const r of rels) await openFile(r); toast(t("ablegen-kopiert", { n: rels.length })); }
  }
  const weiter = wartend.shift(); if (weiter) await handleDrop(weiter);
}

/** Eine fallengelassene Datei darf das Fenster nie selbst „öffnen“ (der Browser würde die Seite wechseln), falls das native Ereignis sie einmal nicht abfängt */
export function sperreDateiDrop(): void {
  const dateien = (e: DragEvent) => !!e.dataTransfer && [...e.dataTransfer.types].includes("Files");
  window.addEventListener("dragover", (e) => { if (dateien(e)) e.preventDefault(); });
  window.addEventListener("drop", (e) => { if (dateien(e)) e.preventDefault(); });
}

/** Wartet, bis alle bisher eingereihten Aufträge (Start mit Pfaden, Ziehen) fertig sind */
export const startFertig = (): Promise<void> => nacheinander(async () => {});

/** Start mit Pfaden (Kommandozeile, „Öffnen mit“, `open -a`): wie auf das Fenster gezogen; `--open` öffnet danach eine Datei im Projekt, `--view` zeigt eine Ansicht */
export const startPfade = (kind: string): Promise<void> => nacheinander(() => startPfadeEcht(kind));
async function startPfadeEcht(kind: string): Promise<void> {
  if (import.meta.env.DEV && kind === "tauri") return;   // Entwicklungs- und Prüfbau: das (versteckte) Fenster lässt die Pfade der Prüfbrücke, sonst holte es sie weg, bevor der Ablauftest sie sieht
  let a; try { a = await call("start_args"); } catch { return; }
  if (!a.paths.length && !a.open && !a.view) return;
  if (a.paths.length) await handleDropEcht(a.paths);
  if (a.open && ui.project) await openFile(a.open);
  if (a.view === "vo-test") { location.search = "?vo-test=a"; return; }   // Minimalbeispiel für die VoiceOver-Prüfung (siehe votest.ts)
  if (a.view) { const { runCommand } = await import("./commands"); await runCommand(`nav:${a.view === "sprites" ? "grafik:sprites" : a.view === "disk" ? "daten:disk" : a.view}`); }
}

let started = false;
/** Das native Drop-Ereignis abonnieren (nur in der echten Anwendung; das Ziehen im Fenster läuft über Zeigerereignisse, siehe dnd.svelte.ts) */
export async function startDrop(kind: string): Promise<void> {
  if (started || kind !== "tauri") return; started = true;
  try {
    const { getCurrentWebview } = await import("@tauri-apps/api/webview");
    await getCurrentWebview().onDragDropEvent((e) => {
      const p = e.payload;
      if (p.type === "enter" || p.type === "over") drop.over = !!ui.ready;
      else if (p.type === "leave") drop.over = false;
      else if (p.type === "drop") void handleDrop(p.paths);
    });
  } catch { /* kein natives Ereignis: Ziehen von Dateien geht dann nicht */ }
}
