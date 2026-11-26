// SPDX-License-Identifier: GPL-3.0-or-later
// Daten (E11): Hex-Ansicht (Binärdateien nur lesen) und Speicherkarte. Der Kern rechnet (`daten.rs`); hier steht der Anzeigezustand.
import { call, type HexPage, type SpeicherInfo } from "./api";
import { fail, setMode, setSub, toast } from "./state.svelte";
import { t } from "./i18n.svelte";

export const hx = $state({ files: [] as { rel: string; size: number }[], rel: null as string | null, page: null as HexPage | null, sel: null as null | { a: number; b: number }, width: 16, query: "", status: "", loading: false });
export const spk = $state({ info: null as SpeicherInfo | null, preset: undefined as string | undefined, filter: "all" as "all" | "prog" | "ram" | "rom" | "io", loading: false });

/** Dateien, die in der Hex-Ansicht statt im Standardprogramm öffnen (wie im Kern) */
export const hexStattExtern = (rel: string): boolean => !/\.(png|jpe?g|gif|bmp|ico|webp|pdf|zip|gz|tar|7z|wav|mp3|ogg)$/i.test(rel);

export async function loadHexFiles(): Promise<void> { try { hx.files = await call("hex_list"); } catch (e) { fail(e); } }
export async function hexSeite(page: number): Promise<void> {
  if (!hx.rel) return; hx.loading = true;
  try { hx.page = await call("hex_page", { rel: hx.rel, page, width: hx.width }); } catch (e) { fail(e); hx.page = null; } finally { hx.loading = false; }
}
/** Beliebige Datei des Projekts im Hex-Editor öffnen (zum Beispiel ein gebautes Programm) */
export async function pickHexDatei(): Promise<void> {
  try { const r = await call("file_pick"); if (r?.rel) await openHex(r.rel); else if (r?.outside) toast(r.outside, true); } catch (e) { fail(e); }
}
export async function openHex(rel: string, off?: number): Promise<void> {
  setMode("daten"); setSub("daten", "hex");
  if (hx.rel !== rel) { hx.rel = rel; hx.sel = null; hx.status = ""; hx.query = ""; }
  await hexSeite(off === undefined ? (hx.page?.page ?? 0) : Math.floor(off / 4096)); void loadHexFiles();
  if (off !== undefined) hx.sel = { a: off, b: off };
}
export async function hexBreite(w: number): Promise<void> { hx.width = w; await hexSeite(hx.page?.page ?? 0); }
/** Suchen ab dem Byte hinter der Auswahl; springt zur Seite und wählt den Treffer */
export async function hexSuchen(): Promise<void> {
  if (!hx.rel || !hx.query.trim()) return;
  try {
    const von = hx.sel ? Math.max(hx.sel.a, hx.sel.b) + 1 : 0; const r = await call("hex_find", { rel: hx.rel, query: hx.query, from: von });
    if (!r) { hx.status = t("hexui-nichts-gefunden"); return; }
    hx.status = t("hexui-gefunden", { off: r.off.toString(16).toUpperCase().padStart(6, "0") });
    await hexSeite(Math.floor(r.off / 4096)); hx.sel = { a: r.off, b: r.off + r.len - 1 };
  } catch (e) { fail(e); }
}
export async function loadSpeicher(page?: number): Promise<void> {
  spk.loading = true;
  try { spk.info = await call("speicher_info", { preset: spk.preset, page }); if (spk.info.banking && spk.preset === undefined) spk.preset = spk.info.banking.selected ?? undefined; } catch (e) { fail(e); } finally { spk.loading = false; }
}
export async function speicherPreset(id: string): Promise<void> { spk.preset = id; await loadSpeicher(spk.info?.sel); }
export const hex = (v: number, n: number): string => v.toString(16).toUpperCase().padStart(n, "0");

