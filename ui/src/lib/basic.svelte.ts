// SPDX-License-Identifier: GPL-3.0-or-later
// BASIC-Einstellungen des Projekts (Abschnitt `basic` der Projektdatei): Zustand für die Karte in den Projekt-Einstellungen.
// Fachliches (Wertebereich B191, Schreiben der Projektdatei) macht der Kern (`vc-shell/src/basic.rs`).
import { call, type BasicState } from "./api";
import { t } from "./i18n.svelte";
import { fail, toast, ui } from "./state.svelte";

export const bs = $state({ state: null as BasicState | null });

export async function loadBasic(): Promise<void> {
  if (!ui.project) { bs.state = null; return; }
  try { bs.state = await call("basic_get"); } catch { bs.state = null; }
}

export async function saveBasic(v: { start: number; step: number; compact: boolean; check: boolean; disabled: string[] }): Promise<boolean> {
  try { bs.state = await call("basic_set", v); toast(t("basicui-gespeichert")); return true; } catch (e) { fail(e); return false; }
}
