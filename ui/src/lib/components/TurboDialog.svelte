<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from "../i18n.svelte";
  import { canCopy, dlg, pickDisk, runProvide, runReduce, tb, verdict } from "../turbo.svelte";
  import Dialog from "./Dialog.svelte";
  // Assistent „Turbo-BASIC-Diskette bereitstellen“ (Konzept 7) und Bestätigung „Verkleinern“ (T2): Datei wählen → Prüfung mit Befunden → (bei nicht erkannter Diskette: Rückfrage, T4) → Kopieren nach turbo/
  const v = $derived(verdict(dlg.check));
  const title = $derived(t(dlg.mode === "replace" ? "turboui-dlg-ersetzen-titel" : dlg.mode === "reduce" ? "turboui-karte-titel" : "turboui-dlg-titel"));
  const red = $derived(dlg.mode === "reduce" ? tb.state?.report?.reduce : dlg.check?.reduce);
  const now = $derived(dlg.mode === "reduce" ? (tb.state?.report?.free_sectors ?? 0) : (dlg.check?.free_sectors ?? 0));
  const befund = $derived.by(() => {
    const c = dlg.check; if (!c) return [] as string[];
    const out: string[] = [];
    if (!c.dos2) out.push(t("turboui-dlg-befund-kein-dos2"));
    else if (!c.turbo) out.push(t("turboui-dlg-befund-nicht-erkannt"));
    for (const m of c.problems.slice(0, 3)) out.push(t("turboui-dlg-befund-fehler", { m }));
    return out;
  });
  let pickBtn = $state<HTMLButtonElement>();
  $effect(() => { if (dlg.open && dlg.mode !== "reduce" && !dlg.check && !dlg.done) queueMicrotask(() => pickBtn?.focus()); });
</script>

<Dialog bind:open={dlg.open} {title} wide>
  {#snippet children()}
    <div class="td">
      {#if dlg.mode === "reduce"}
        {#if red}
          <p>{t("turboui-dlg-verkleinern-text", { count: red.removes.length, free: red.free, now })}</p>
          <p class="mono small dim" data-testid="reduce-files">{red.removes.join(", ")}</p>
        {/if}
        {#if dlg.error}<p class="err" role="alert">{dlg.error}</p>{/if}
      {:else}
        <p class="legal">{t("rechtlich-turbo-assistent")}</p>
        {#if dlg.done}
          <p class="okline" role="status" data-testid="provide-done">{t("turboui-dlg-fertig", { disk: dlg.done.disk, free: dlg.done.free })}</p>
          {#if dlg.done.gitignore}<p class="dim small">{t("turboui-dlg-fertig-gitignore")}</p>{/if}
        {:else}
          <section aria-labelledby="td-1">
            <h3 id="td-1">{t("turboui-dlg-schritt-datei")}</h3>
            <div class="row">
              <button type="button" class="btn" bind:this={pickBtn} onclick={() => void pickDisk()}>{t("turboui-dlg-datei-waehlen")}</button>
              <span class="mono small grow" title={dlg.src}>{dlg.src ? t("turboui-dlg-quelle", { path: dlg.src }) : t("turboui-dlg-keine-datei")}</span>
            </div>
          </section>
          {#if dlg.checking}<p class="dim" role="status">{t("turboui-dlg-pruefung-laeuft")}</p>{/if}
          {#if dlg.check}
            <section aria-labelledby="td-2" data-testid="provide-check">
              <h3 id="td-2">{t("turboui-dlg-schritt-pruefen")}</h3>
              {#if dlg.check.sameAsCopy}
                <p class="warn" role="alert">{t("turboui-dlg-quelle-ist-kopie", { disk: dlg.check.dest ?? "" })}</p>
              {:else if v === "ok"}
                <p class="okline" role="status">{t("turboui-dlg-befund-erkannt", { name: dlg.check.name + (dlg.check.version && !dlg.check.name.includes(dlg.check.version) ? ` ${dlg.check.version}` : ""), count: dlg.check.free_sectors, kb: dlg.check.kb, dest: dlg.check.dest ?? "" })}</p>
              {:else}
                {#each befund as b (b)}<p class="warn" role="alert">{b}</p>{/each}
                <p><b>{t("turboui-dlg-trotzdem-frage")}</b></p>
                <div class="row">
                  <button type="button" class="btn" class:pri={dlg.anyway} aria-pressed={dlg.anyway} onclick={() => (dlg.anyway = true)}>{t("turboui-dlg-trotzdem-ja")}</button>
                  <button type="button" class="btn" onclick={() => { dlg.check = null; dlg.src = ""; dlg.anyway = false; pickBtn?.focus(); }}>{t("turboui-dlg-trotzdem-nein")}</button>
                </div>
              {/if}
              {#if dlg.check.destExists}<p class="dim small">{t("turboui-dlg-kopie-existiert", { dest: dlg.check.dest ?? "" })}</p>
              {:else if dlg.check.current}<p class="dim small">{t("turboui-dlg-aktuelle-kopie", { disk: dlg.check.current })}</p>{/if}
            </section>
            {#if canCopy() && !dlg.check.sameAsCopy}
              <section aria-labelledby="td-3">
                <h3 id="td-3">{t("turboui-dlg-schritt-kopieren")}</h3>
                {#if dlg.check.reduce}
                  <fieldset class="opts">
                    <legend>{t("turboui-dlg-verkleinern")}</legend>
                    <p class="small dim">{t("turboui-dlg-verkleinern-text", { count: dlg.check.reduce.removes.length, free: dlg.check.reduce.free, now: dlg.check.free_sectors })}</p>
                    <label class="chk"><input type="radio" name="td-red" checked={dlg.reduce} onchange={() => (dlg.reduce = true)} />{t("turboui-dlg-verkleinern-bestaetigt")}</label>
                    <label class="chk"><input type="radio" name="td-red" checked={!dlg.reduce} onchange={() => (dlg.reduce = false)} />{t("turboui-dlg-verkleinern-nein")}</label>
                  </fieldset>
                {/if}
                <p class="dim small">{t("turboui-hinweis-gitignore")}</p>
              </section>
            {/if}
          {/if}
        {/if}
        {#if dlg.error}<p class="err" role="alert">{t("turboui-dlg-fehler", { detail: dlg.error })}</p>{/if}
      {/if}
    </div>
  {/snippet}
  {#snippet footer()}
    {#if dlg.done}
      <button type="button" class="btn pri" onclick={() => (dlg.open = false)}>{t("turboui-dlg-schliessen")}</button>
    {:else if dlg.mode === "reduce"}
      <button type="button" class="btn" disabled={dlg.busy} onclick={() => (dlg.open = false)}>{t("turboui-dlg-abbrechen")}</button>
      <button type="button" class="btn pri" disabled={dlg.busy || !red} onclick={() => void runReduce()}>{t("turboui-dlg-verkleinern-bestaetigt")}</button>
    {:else}
      <button type="button" class="btn" disabled={dlg.busy} onclick={() => (dlg.open = false)}>{t("turboui-dlg-abbrechen")}</button>
      <button type="button" class="btn pri" disabled={!canCopy() || dlg.busy || !!dlg.check?.sameAsCopy} onclick={() => void runProvide()}>{t("turboui-dlg-kopieren")}</button>
    {/if}
  {/snippet}
</Dialog>

<style>
  .td { color: var(--text); min-width: 520px; max-width: 640px; } @media (max-width: 640px) { .td { min-width: 0; } }
  h3 { font-size: 12px; margin: 14px 0 6px; color: var(--dim); letter-spacing: 0.04em; text-transform: uppercase; }
  .legal { margin: 0 0 6px; padding: 8px 10px; border-radius: 8px; background: var(--raise); border: 1px solid var(--line2); font-size: 12.5px; }
  .row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; } .grow { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .chk { display: flex; gap: 8px; align-items: flex-start; margin: 6px 0; font-size: 13px; }
  .opts { border: 1px solid var(--line2); border-radius: 8px; margin: 0 0 8px; padding: 8px 10px; } .opts legend { font-size: 12px; font-weight: 600; padding: 0 4px; }
  .small { font-size: 11.5px; margin: 4px 0; } .dim { color: var(--dim); } .err { color: var(--err); margin: 8px 0 0; } .warn { color: var(--warn); margin: 4px 0; } .okline { color: var(--ok); margin: 4px 0; }
  @media (forced-colors: active) { .legal, .opts { border: 1px solid CanvasText; } }
</style>
