// SPDX-License-Identifier: GPL-3.0-or-later
// Bereich „Projekt“: Zustand der Projektdatei (Anzeige und sofortiges Speichern einzelner Felder). Der Kern prüft und speichert; ein abgelehnter Wert bleibt in der Datei unverändert.
import { onLangChange } from "./i18n.svelte";
import { call, type ProjektInfo } from "./api";
import { fail, ui } from "./state.svelte";

export const pj = $state({ info: null as ProjektInfo | null, root: "" as string, busy: false });

export async function loadProjekt(): Promise<void> {
  if (!ui.project) { pj.info = null; pj.root = ""; return; }
  try { pj.info = await call("projekt_info"); pj.root = ui.project.root; } catch (e) { pj.info = null; fail(e); }
}
onLangChange(() => { if (pj.info) void loadProjekt(); });
/** Felder ändern; true, wenn gespeichert (sonst Meldung, Anzeige zeigt wieder die Datei) */
export async function setzeProjekt(a: Parameters<typeof call<"projekt_set">>[1]): Promise<boolean> {
  pj.busy = true;
  try {
    pj.info = await call("projekt_set", a);
    if (ui.project && a?.name !== undefined) ui.project.name = pj.info.name;
    return true;
  } catch (e) { fail(e); await loadProjekt(); return false; }
  finally { pj.busy = false; }
}
