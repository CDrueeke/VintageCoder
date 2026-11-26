// SPDX-License-Identifier: GPL-3.0-or-later
// Tests (E1.3): Läufe über `*.vctest` im Hintergrund; der Kern meldet den ganzen Stand als Ereignis `tests:update`.
import { onLangChange } from "./i18n.svelte";
import { call, on, type TestSuite } from "./api";
import { ed } from "./editor.svelte";
import { fail } from "./state.svelte";

export const tests = $state({ suite: null as TestSuite | null, backend: "" as string, filter: "", onlyFailed: false, sel: null as null | { file: string; name: string }, loaded: false });

export async function loadTests(): Promise<void> {
  try { const s = await call("tests_state"); tests.suite = s; if (!tests.loaded) { tests.loaded = true; tests.backend = s.backends?.[0]?.id ?? ""; } } catch (e) { fail(e); }
}
onLangChange(() => { if (tests.loaded) void loadTests(); });
export async function runTests(only: "all" | "failed" | "file" = "all", file?: string): Promise<void> {
  ed.dock = "tests";
  try { await call("tests_start", { only, file, backend: tests.backend || null }); } catch (e) { fail(e); }
}
export async function stopTests(): Promise<void> { try { await call("tests_stop"); } catch (e) { fail(e); } }
export function resetTests(): void { tests.suite = null; tests.sel = null; tests.loaded = false; }

let listening = false;
export async function startTests(): Promise<void> {
  if (listening) return; listening = true;
  await on("tests:update", (p) => { const keep = tests.suite; tests.suite = { ...(p as TestSuite), backends: keep?.backends, files: keep?.files }; });
}
