<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "../lib/i18n.svelte";
  import { attachStroke, run, CELL, H, W, type MeasureResult } from "../lib/measure";
  import Card from "../lib/components/Card.svelte";
  import DataTable from "../lib/components/DataTable.svelte";
  // Entwicklungsansicht für Prüfpunkt G1: Messwerte ablesen; das Ergebnis liegt zusätzlich als JSON in `window.__vcMeasureResult`
  let canvas: HTMLCanvasElement;
  let bundled = false;
  let busy = $state(false);
  let step = $state("");
  let result = $state<MeasureResult | null>(null);
  let row = $state(0);
  onMount(() => {
    const off = attachStroke(canvas, () => bundled);
    (window as unknown as Record<string, unknown>).__vcMeasure = start;
    return off;
  });
  async function start() {
    busy = true;
    try {
      result = await run(canvas, (b) => (bundled = b), (s) => (step = s));
      (window as unknown as Record<string, unknown>).__vcMeasureResult = result;
    } finally { busy = false; }
    return result;
  }
  const f = (x: number) => x.toFixed(2);
  const rows = $derived((result?.stats ?? []).map((s) => ({ test: t(s.id), n: String(s.n), mean: f(s.mean), median: f(s.median), max: f(s.max) })));
</script>

<div style="overflow:auto;padding:14px;display:flex;flex-direction:column;gap:12px;min-height:0">
  <div><h2 style="font-size:18px">{t("measure-title")}</h2><p style="color:var(--dim);max-width:720px">{t("measure-intro")}</p></div>
  <div style="display:flex;gap:16px;flex-wrap:wrap;align-items:flex-start">
    <div class="stage" style="padding:12px" role="img" aria-label={t("measure-canvas-label")}><canvas bind:this={canvas} width={W * CELL} height={H * CELL} style="width:{W * CELL}px;height:{H * CELL}px;background:var(--check1)"></canvas></div>
    <div style="display:flex;flex-direction:column;gap:10px;min-width:0;flex:1">
      <div><button type="button" class="btn pri" disabled={busy} onclick={start}>{busy ? t("measure-running") : t("measure-run")}</button>{#if busy}<span class="mono" style="margin-left:10px;color:var(--dim)">{step}</span>{/if}</div>
      {#if result}
        <Card title={`${t("measure-col-test")} (${t("measure-unit-ms")})`}><div class="c" style="padding:0"><DataTable bind:selected={row} label={t("measure-title")} {rows} cols={[{ id: "test", label: t("measure-col-test") }, { id: "n", label: t("measure-col-n") }, { id: "mean", label: t("measure-col-mean") }, { id: "median", label: t("measure-col-median") }, { id: "max", label: t("measure-col-max") }]} /></div></Card>
        <Card title={t("measure-env")}><div class="c mono" style="font-size:11.5px;color:var(--dim)">{#each Object.entries(result.env) as [k, v] (k)}<div>{k}: {v}</div>{/each}</div></Card>
      {/if}
    </div>
  </div>
</div>
