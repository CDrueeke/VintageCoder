<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { ed, jumpTo } from "../editor.svelte";
  import { t } from "../i18n.svelte";
  import { debugTest } from "../debug.svelte";
  import { loadTests, runTests, stopTests, tests } from "../tests.svelte";
  // Tests: Gruppen und Ergebnisse, Emulator wählbar (oder nur statische Prüfungen), Filter und „nur Fehler“, Detail mit Erwartung und Registern, Sprung zur Quelle
  $effect(() => { void ed.dock; if (!tests.loaded) void loadTests(); });
  const s = $derived(tests.suite);
  const hex = (n: number, w = 2) => "$" + n.toString(16).toUpperCase().padStart(w, "0");
  const mark = (st: string) => (st === "skipped" ? "○" : st === "timeout" ? "◷" : "●");
  const f = $derived(tests.filter.trim().toLowerCase());
  const visible = (g: NonNullable<typeof s>["groups"][number]) => g.items.filter((i) => (!tests.onlyFailed || (i.state === "failed" || i.state === "timeout") && i.level === "error") && (!f || i.name.toLowerCase().includes(f) || i.detail.toLowerCase().includes(f)));
  const selected = $derived(s?.groups.flatMap((g) => g.items).find((i) => tests.sel && i.file === tests.sel.file && i.name === tests.sel.name) ?? null);
  const total = $derived(s ? s.counts.passed + s.counts.failed + s.counts.timedOut + s.counts.skipped + s.counts.hints : 0);
</script>

<div class="tp">
  <div class="bar">
    <button type="button" class="btn" disabled={s?.running} onclick={() => void runTests("all")}>{t("tests-alle")}</button>
    <button type="button" class="btn" disabled={s?.running || !(s?.counts.failed || s?.counts.timedOut)} onclick={() => void runTests("failed")}>{t("tests-fehlgeschlagene")}</button>
    {#if s?.running}<button type="button" class="btn" onclick={() => void stopTests()}>{t("tests-abbrechen")}</button>{/if}
    <label class="sel">{t("tests-ziel")}
      <select bind:value={tests.backend} aria-label={t("tests-ziel")}><option value="">{t("tests-nur-statisch")}</option>{#each s?.backends ?? [] as b (b.id)}<option value={b.id}>{b.name}</option>{/each}</select>
    </label>
    <input type="search" bind:value={tests.filter} placeholder={t("tests-filter")} aria-label={t("tests-filter")} />
    <label class="chk"><input type="checkbox" bind:checked={tests.onlyFailed} /> {t("tests-nur-fehler")}</label>
    <span class="info" role="status" aria-live="polite">{#if s?.running}{t("tests-laeuft")}{:else if s && total > 0}{t("tests-stand", { p: s.counts.passed, n: total })}{:else if s}{t("tests-keine")}{/if}</span>
  </div>
  {#if s?.error}<p class="err" role="alert">{s.error}</p>{/if}
  <div class="body">
    <ul class="list">
      {#each s?.groups ?? [] as g (g.title + (g.file ?? ""))}
        <li class="grp"><span>{g.title}{g.backend ? ` · ${g.backend}` : ""}</span><span class="ms mono">{g.millis ? `${g.millis} ms` : ""}</span></li>
        {#each visible(g) as i (i.file + i.name)}
          <li><button type="button" class="row {i.state} {i.level}" aria-pressed={tests.sel?.name === i.name && tests.sel?.file === i.file} onclick={() => (tests.sel = { file: i.file, name: i.name })}>
            <span class="mk" aria-hidden="true">{mark(i.state)}</span><span class="nm">{i.name}</span><span class="state">{t(`tests-state-${i.state}`)}</span><span class="meas mono">{i.measured ?? ""}</span>
          </button></li>
        {/each}
      {/each}
    </ul>
    <div class="detail">
      {#if selected}
        <h3>{selected.name}</h3>
        <p>{t(`tests-state-${selected.state}`)}{selected.level === "hint" ? ` (${t("tests-hinweis")})` : ""}</p>
        {#if selected.detail}<p class="txt">{selected.detail}</p>{/if}
        {#if selected.measured}<p>{t("tests-gemessen")}: <span class="mono">{selected.measured}</span></p>{/if}
        {#if selected.regs}<p class="mono">PC {hex(selected.regs.pc, 4)} A {hex(selected.regs.a)} X {hex(selected.regs.x)} Y {hex(selected.regs.y)} SP {hex(selected.regs.sp)} P {hex(selected.regs.flags)}</p>{/if}
        {#if selected.location}<button type="button" class="btn" onclick={() => void jumpTo(selected.location![0], selected.location![1] - 1, 0)}>{t("tests-quelle")}</button>{/if}
        <button type="button" class="btn" onclick={() => void jumpTo(selected.file, 0, 0)}>{t("tests-testdatei")}</button>
        <button type="button" class="btn" data-tests="debuggen" disabled={s?.running} onclick={() => void debugTest(selected.file, selected.name)}>{t("tests-debuggen")}</button>
      {:else}<p class="none">{t("tests-waehlen")}</p>{/if}
    </div>
  </div>
</div>

<style>
  .tp { flex: 1; min-height: 0; display: flex; flex-direction: column; font-size: 12.5px; }
  .bar { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; padding: 4px 10px; }
  .bar input[type="search"], select { padding: 3px 6px; border-radius: 6px; border: 1px solid var(--line2); background: var(--card); color: var(--text); font: inherit; font-size: 12px; }
  .sel, .chk { font-size: 12px; color: var(--dim); display: flex; gap: 5px; align-items: center; } .info { color: var(--dim); font-size: 12px; }
  .err { margin: 0; padding: 2px 12px; color: var(--err); }
  .body { flex: 1; min-height: 0; display: flex; }
  .list { list-style: none; margin: 0; padding: 0 0 14px; overflow: auto; flex: 1.2; border-right: 1px solid var(--line); }
  .grp { display: flex; justify-content: space-between; padding: 5px 12px 2px; color: var(--dim); font-size: 11.5px; font-weight: 600; } .ms { font-weight: 400; }
  .row { display: flex; gap: 8px; width: 100%; text-align: left; padding: 2px 12px; border: 0; background: none; color: var(--text); font: inherit; cursor: pointer; }
  .row:hover, .row[aria-pressed="true"] { background: var(--accentbg); } .row:focus-visible { outline: none; box-shadow: var(--focus); }
  .mk { width: 14px; } .passed .mk { color: var(--ok); } .failed.error .mk, .timeout .mk { color: var(--err); } .failed.hint .mk { color: var(--warn); } .skipped .mk { color: var(--faint); }
  .nm { flex: 1; min-width: 0; } .state { color: var(--dim); } .meas { color: var(--faint); min-width: 70px; text-align: right; }
  .detail { flex: 1; overflow: auto; padding: 8px 14px; } .detail h3 { margin: 0 0 4px; font-size: 13px; } .detail p { margin: 0 0 6px; } .txt { white-space: pre-wrap; } .none { color: var(--dim); }
  .detail .btn { margin-right: 6px; }
</style>
