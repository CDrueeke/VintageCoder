<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import { t } from "../i18n.svelte";
  import { applyConv, bc, refresh } from "../basicconv.svelte";
  // Dialog „BASIC umwandeln“: Titel und kurze Erklärung, Modus des Programms, Optionen, Vorschau der geänderten Zeilen, Warnungen; Hauptaktion „Anwenden“ (ein Rückgängig-Schritt)
  const r = $derived(bc.result);
  const LIMIT = 200;
  const rows = $derived((r?.changes ?? []).slice(0, LIMIT));
  const diags = $derived([...(r?.diagnostics ?? []), ...(r?.warnings ?? [])]);
  const canApply = $derived(!!r?.ok && (r.changes?.length ?? 0) > 0 && !bc.busy);
  const modeText = $derived(r?.mode ? t(`basicconv-modus-${r.mode}`) : "");
  const short = (s: string) => s.replace(/\s+$/, "").replace(/\n/g, " ⏎ ");
  const rangeBad = $derived(bc.kind === "renumber" && ((bc.from.trim() === "") !== (bc.to.trim() === "")));
</script>

{#if bc.open}
  <Dialog bind:open={bc.open} title={t(`basicconv-titel-${bc.kind}`)} wide>
    {#snippet children()}
      <div class="bcd" data-testid="basicconv">
        <p class="dim">{t(`basicconv-erklaerung-${bc.kind}`)}</p>
        <p class="meta" role="status" data-testid="basicconv-modus">
          {#if r?.mode}<span>{t("basicconv-modus", { modus: modeText })}</span>{/if}
          {#if r?.numbering}<span class="mono">{t("basicconv-nummerierung", { start: r.numbering.start, step: r.numbering.step })}</span>{/if}
        </p>
        {#if (r?.includes ?? 0) > 0}
          <label class="opt"><input type="checkbox" bind:checked={bc.includes} onchange={() => void refresh()} data-testid="basicconv-includes" />{t("basicconv-includes", { n: r?.includes ?? 0 })}</label>
        {/if}
        {#if (r?.media ?? 0) > 0}<p class="dim small" data-testid="basicconv-medien">{t("basicconv-medien", { n: r?.media ?? 0 })}</p>{/if}
        {#if bc.kind === "renumber"}
          <fieldset class="range"><legend>{t("basicconv-bereich")}</legend>
            <label>{t("basicconv-bereich-von")}<input class="inp mono" inputmode="numeric" bind:value={bc.from} onchange={() => void refresh()} data-testid="basicconv-von" aria-invalid={rangeBad} /></label>
            <label>{t("basicconv-bereich-bis")}<input class="inp mono" inputmode="numeric" bind:value={bc.to} onchange={() => void refresh()} data-testid="basicconv-bis" aria-invalid={rangeBad} /></label>
            <span class="dim small">{t("basicconv-bereich-hinweis")}</span>
          </fieldset>
          {#if rangeBad}<p class="err small" role="alert">{t("basicconv-bereich-beide")}</p>{/if}
        {/if}

        {#if diags.length}
          <ul class="diags" aria-label={t("basicconv-meldungen")} data-testid="basicconv-meldungen">
            {#each diags as d, i (i)}<li class:err={d.severity === "error"}><span class="code mono">{d.code}</span><span>{d.message}</span>{#if d.line > 0}<span class="loc mono">{d.file} · {t("problems-line", { line: d.line })}</span>{/if}</li>{/each}
          </ul>
        {/if}
        {#if bc.error}<p class="err" role="alert" data-testid="basicconv-fehler">{bc.error}</p>{/if}

        <h3>{t("basicconv-vorschau")}</h3>
        {#if bc.busy && !r}<p class="dim">{t("basicconv-rechnet")}</p>
        {:else if r?.ok}
          <p class="sum" role="status" data-testid="basicconv-summe">{(r.changes?.length ?? 0) === 0 ? t("basicconv-nichts") : t("basicconv-summe", { zeilen: r.changes?.length ?? 0, labels: r.newLabels?.length ?? 0 })}</p>
          {#if rows.length}
            <ol class="rows" data-testid="basicconv-zeilen">
              {#each rows as c, i (i)}<li>{#if c.file !== bc.rel}<span class="fl mono">{c.file}</span>{/if}<span class="ln mono">{c.line}</span><span class="old mono">{short(c.old)}</span><span class="arrow" aria-hidden="true">→</span><span class="new mono">{short(c.new)}</span></li>{/each}
            </ol>
            {#if (r.changes?.length ?? 0) > LIMIT}<p class="dim small">{t("basicconv-mehr", { n: (r.changes?.length ?? 0) - LIMIT })}</p>{/if}
          {/if}
        {/if}
      </div>
    {/snippet}
    {#snippet footer()}
      <button type="button" class="btn" onclick={() => (bc.open = false)} data-testid="basicconv-abbrechen">{t("dialog-abbrechen")}</button>
      <button type="button" class="btn pri" disabled={!canApply || rangeBad} onclick={() => void applyConv()} data-testid="basicconv-anwenden">{t("basicconv-anwenden")}</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .bcd { display: flex; flex-direction: column; gap: 8px; min-width: min(760px, 86vw); }
  .dim { color: var(--dim); margin: 0; } .small { font-size: 12px; } .err { color: var(--err, #c33); }
  .meta { display: flex; gap: 16px; flex-wrap: wrap; margin: 0; } h3 { margin: 6px 0 0; font-size: 13px; }
  .opt { display: flex; gap: 8px; align-items: center; }
  .range { display: flex; gap: 12px; align-items: end; flex-wrap: wrap; border: 1px solid var(--line, #ccc); border-radius: 8px; padding: 8px 10px; } .range label { display: flex; flex-direction: column; gap: 3px; font-size: 12px; } .range .inp { width: 90px; }
  .diags { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 3px; } .diags li { display: flex; gap: 8px; flex-wrap: wrap; } .code { color: var(--dim); font-size: 11px; } .loc { margin-left: auto; color: var(--dim); font-size: 11px; }
  .rows { list-style: none; margin: 0; padding: 0; max-height: 36vh; overflow: auto; border: 1px solid var(--line, #ccc); border-radius: 8px; }
  .rows li { display: grid; grid-template-columns: 56px 1fr 18px 1fr; gap: 8px; padding: 3px 8px; font-size: 12px; align-items: start; } .rows li:nth-child(odd) { background: color-mix(in srgb, var(--dim) 7%, transparent); }
  .ln { color: var(--dim); } .rows li .fl { grid-column: 1 / -1; color: var(--dim); font-size: 11px; padding-top: 2px; } .old { color: var(--dim); overflow-wrap: anywhere; } .new { overflow-wrap: anywhere; }
  .sum { margin: 0; font-weight: 600; }
  @media (max-width: 700px) { .rows li { grid-template-columns: 48px 1fr; } .rows .arrow { display: none; } .rows .new { grid-column: 2; } }
</style>
