// SPDX-License-Identifier: GPL-3.0-or-later
// Debuggen (E6): die Sitzung läuft im Kern (`dbgsitzung.rs`, Arbeits-Thread); hier stehen nur der Anzeigezustand, die Abfrage im Takt und die Befehle. Haltepunkte der Editoren werden mit der Sitzung abgeglichen,
// die Ausführungszeile erscheint im Editor.
import { call, type DebugState } from "./api";
import { alleHaltepunkte, jumpTo, setDebugZeile } from "./editor.svelte";
import { bau, buildRun } from "./bau.svelte";
import { t } from "./i18n.svelte";
import { fail, setMode, toast } from "./state.svelte";
import { isTurbo } from "./turbo.svelte";

export const dbg = $state({
  s: null as DebugState | null, starting: false, watches: [] as string[],
  frame: null as null | { seq: number; w: number; h: number; img: string }, auto: false, slow: false, frameWanted: false,
  mem: "", fehler: "",
  chipInfo: [] as { id: string; name: string; start: number; len: number; regs: { name: string; addr: number; access: string; doc: string }[] }[], chipSel: [] as string[],
});

/** Chip-Fenster: Chips des Zielsystems laden und die gewählten Fenster dem Kern nennen (gelesen wird beim Anhalten) */
export async function chipsLaden(): Promise<void> {
  const ziel = bau.info?.target; if (!ziel) return;
  try { dbg.chipInfo = (await call("chips_info", { target: ziel })).chips; } catch { dbg.chipInfo = []; }
  dbg.chipSel = dbg.chipSel.filter((id) => dbg.chipInfo.some((c) => c.id === id));
}
export async function chipsWaehlen(ids: string[]): Promise<void> {
  dbg.chipSel = ids; if (!aktiv()) return;
  await cmd("chips", { wins: dbg.chipInfo.filter((c) => ids.includes(c.id)).map((c) => [c.start, c.len]) });
}
export const aktiv = (): boolean => !!dbg.s?.active;
let timer: ReturnType<typeof setInterval> | null = null; let laeuft = false; let bpsStand = ""; let letzteZeile = "";
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

export async function cmd(kind: string, extra: Record<string, unknown> = {}): Promise<boolean> {
  try { await call("debug_cmd", { kind, ...extra }); return true; } catch (e) { fail(e); return false; }
}

/** Bauen (falls nötig) und die Sitzung starten */
export async function debugStarten(): Promise<void> {
  if (isTurbo()) { toast(t("turboui-debuggen-hinweis"), true); return; }
  if (dbg.starting || aktiv()) { setMode("debug"); return; }
  dbg.starting = true;
  try {
    await buildRun({ then: "none" });
    const t0 = Date.now(); while (bau.running || (Date.now() - t0 < 500 && !bau.last)) await wait(100);
    if (!bau.last?.ok) return;
    const bps = alleHaltepunkte();
    await call("debug_start", { bps, watches: dbg.watches });
    bpsStand = JSON.stringify(bps); letzteZeile = ""; dbg.frame = null;
    void chipsLaden().then(() => chipsWaehlen(dbg.chipSel));
    setMode("debug"); toast(t("dbgui-startet")); beginnen();
  } catch (e) { fail(e); } finally { dbg.starting = false; }
}
/** „Diesen Test debuggen“: eine Sitzung mit der Falle des Tests, angehalten an ihrem Anfang (kein Bauen: der Kern bereitet den Test selbst vor) */
export async function debugTest(file: string, test: string): Promise<void> {
  if (dbg.starting) return; dbg.starting = true;
  try {
    await call("debug_start_test", { file, test });
    letzteZeile = ""; dbg.frame = null; void chipsLaden().then(() => chipsWaehlen(dbg.chipSel));
    setMode("debug"); toast(t("dbgui-test-startet", { name: test })); beginnen();
  } catch (e) { fail(e); } finally { dbg.starting = false; }
}
export async function debugEnde(): Promise<void> {
  try { await call("debug_stop"); } catch { /* schon weg */ }
  aufraeumen(); toast(t("dbgui-beendet"));
}
function aufraeumen(): void { if (timer) { clearInterval(timer); timer = null; } dbg.s = null; dbg.frame = null; setDebugZeile(null, null); letzteZeile = ""; }
function beginnen(): void { if (!timer) timer = setInterval(() => void abfrage(), 150); }

export async function abfrage(): Promise<void> {
  if (laeuft) return; laeuft = true;
  try {
    const s = await call("debug_state");
    if (!s.active) { aufraeumen(); return; }
    for (const m of s.meldungen ?? []) toast(m.text, !m.ok);
    dbg.s = s;
    if (s.ended) { aufraeumen(); toast(t("dbgui-beendet")); return; }
    // Haltepunkte der Editoren abgleichen
    const bps = alleHaltepunkte(); const j = JSON.stringify(bps);
    if (j !== bpsStand) { bpsStand = j; await call("debug_cmd", { kind: "bps", bps }); }
    // Ausführungszeile im Editor
    const z = !s.running && s.ort ? `${s.ort.file}:${s.ort.line}` : "";
    if (z !== letzteZeile) { letzteZeile = z; if (s.ort && !s.running) setDebugZeile(s.ort.file, s.ort.line); else setDebugZeile(null, null); }
    // Bild nur holen, wenn es neu ist und gewünscht wird
    if (s.frame?.can && s.frame.seq !== (dbg.frame?.seq ?? 0) && (dbg.frameWanted || dbg.auto || dbg.slow)) { dbg.frameWanted = false; const f = await call("debug_frame", { seq: dbg.frame?.seq ?? 0 }); if (f) dbg.frame = f; }
  } catch { /* App schließt oder Sitzung weg */ } finally { laeuft = false; }
}

/** Befehle der Symbolleiste; danach sofort neu abfragen */
const dann = async (kind: string, extra: Record<string, unknown> = {}) => { if (await cmd(kind, extra)) { await wait(120); laeuft = false; await abfrage(); } };
export const weiter = () => dann("cont");
export const pause = () => dann("pause");
export const ueber = () => dann("over");
export const hinein = () => dann("into");
export const raus = () => dann("out");
/** Bis zur Cursorzeile des Editors laufen */
export async function hierher(rel: string | null, line: number): Promise<void> { if (!aktiv()) { toast(t("err-debug-keine-sitzung"), true); return; } if (!rel) { toast(t("dbgui-hierher-hinweis"), true); return; } await dann("runto", { rel, line }); }
export async function speicherAnzeigen(text: string): Promise<void> { await dann("mem", { text }); }
export async function punktHinzu(p: { pkind: string; start: number; end?: number; cond?: string; ignore?: number }): Promise<boolean> { const ok = await cmd("point", p); if (ok) await wait(150); return ok; }
export const punktWeg = (id: number) => dann("delpoint", { id });
/** Beobachten: Liste ändern (auch ohne Sitzung, sie gilt beim nächsten Start) */
export async function beobachtenSetzen(liste: string[]): Promise<void> { dbg.watches = liste; if (aktiv()) await dann("watches", { list: liste }); }
export const bildHolen = () => { dbg.frameWanted = true; return cmd("frame"); };
export async function bildModus(auto: boolean, slow: boolean): Promise<void> { dbg.auto = auto; dbg.slow = slow; await cmd("framemode", { auto, slow }); }
export const bildQuelle = (index: number) => cmd("framesource", { index });
export const textSenden = (text: string) => cmd("text", { text });
export const momentErkennen = () => dann("snapdetect");
export const momentSichern = (o: { name: string; sprites: boolean; tiles: boolean; rom: boolean }) => dann("snapsave", o);
/** Zur Quelltextzeile des Halts springen (wechselt in den Code-Bereich) */
export async function zumQuelltext(file: string, line: number): Promise<void> { setMode("code"); await jumpTo(file, line - 1); }
export const hex = (v: number, n: number) => v.toString(16).toUpperCase().padStart(n, "0");

