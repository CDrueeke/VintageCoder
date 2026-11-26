<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import Segment from "./Segment.svelte";
  import AnimSteps from "./AnimSteps.svelte";
  import FrameThumb from "./FrameThumb.svelte";
  import Icon from "./Icon.svelte";
  import { animMode, sp, stepAt } from "../sprite.svelte";
  import { t } from "../i18n.svelte";
  // Große Vorschau der gewählten Animation: läuft sofort, Zoom wählbar (Einpassen oder ganze Vergrößerung), Ablauf umschalten; darunter die Schritte zum direkten Bearbeiten (Dauer, Reihenfolge, Entfernen, Anhängen).
  const view = $derived(sp.view); const anim = $derived(view?.anims?.[Math.min(view.selAnim, (view.anims?.length ?? 1) - 1)] ?? null);
  let playing = $state(true); let cur = $state(0); let zoom = $state("fit");
  const stepIdx = (i: number): number => (anim && i >= anim.steps.length ? Math.max(0, anim.steps.length * 2 - 2 - i) : i);
  $effect(() => { if (sp.previewDialog) { playing = true; cur = 0; zoom = "fit"; } });
  $effect(() => {
    if (!sp.previewDialog || !playing || !anim) return; const a = anim; const t0 = performance.now(); let raf = 0;
    const tick = () => { cur = stepAt(a.expanded, a.mode, (performance.now() - t0) / 1000); raf = requestAnimationFrame(tick); };
    raf = requestAnimationFrame(tick); return () => cancelAnimationFrame(raf);
  });
  const step = $derived(anim ? stepIdx(cur) : 0);
  const frame = $derived(anim ? anim.expanded[cur]?.[0] ?? anim.steps[0]?.[0] ?? 0 : 0);
  const th = $derived(view ? view.thumbs[Math.min(frame, view.thumbs.length - 1)] : undefined);
  const BOX_W = 640, BOX_H = 380;
  const sz = $derived(th && zoom !== "fit" ? { w: Math.max(BOX_W, th.w * th.aspect * Number(zoom)), h: Math.max(BOX_H, (th.pixels.length / th.w) * Number(zoom)) } : { w: BOX_W, h: BOX_H });
</script>

{#if sp.previewDialog && view && anim}
  <Dialog bind:open={sp.previewDialog} title={t("sprite-vorschau-gross-titel", { name: anim.name })} wide>
    {#snippet children()}
      <div class="bar">
        <button type="button" class="play" class:on={playing} aria-pressed={playing} aria-label={playing ? t("sprite-stopp") : t("sprite-abspielen")} data-pv="play" onclick={() => (playing = !playing)}><Icon name={playing ? "stop" : "play"} size={16} /></button>
        <Segment items={[{ id: "loop", label: t("sprite-ablauf-loop") }, { id: "pingpong", label: t("sprite-ablauf-pingpong") }, { id: "once", label: t("sprite-ablauf-once") }]} value={anim.mode} onselect={(id) => void animMode(id)} label={t("sprite-ablauf")} small />
        <label class="small dim" for="pv-zoom">{t("sprite-zoom-label")}</label>
        <select id="pv-zoom" class="inp sel" data-pv="zoom" bind:value={zoom}><option value="fit">{t("sprite-zoom-einpassen")}</option>{#each [4, 8, 16, 24] as z (z)}<option value={String(z)}>{z} ×</option>{/each}</select>
        <span class="small dim" data-pv="step">{t("sprite-vorschau-schritt", { i: step + 1, n: anim.steps.length })}</span>
      </div>
      <div class="stage" data-pv="stage" style={`max-width:${BOX_W}px;max-height:${BOX_H}px`}>
        {#if th}<div class="chk" style={`width:${sz.w}px;height:${sz.h}px`}><FrameThumb pixels={th.pixels} w={th.w} h={th.pixels.length / th.w} aspect={th.aspect} colors={th.colors} boxW={sz.w} boxH={sz.h} scale={zoom === "fit" ? 0 : Number(zoom)} /></div>{/if}
      </div>
      <AnimSteps {view} an={anim} playing={playing ? step : null} />
    {/snippet}
    {#snippet footer()}
      <button type="button" class="btn pri" data-pv="close" onclick={() => (sp.previewDialog = false)}>{t("dialog-schliessen")}</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .bar { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; margin-bottom: 8px; }
  .play { width: 34px; height: 34px; border-radius: 10px; background: var(--accent); color: var(--on-accent); display: inline-flex; align-items: center; justify-content: center; } .play.on { background: var(--accentbg); color: var(--accent2); }
  .stage { overflow: auto; overscroll-behavior: contain; border-radius: 10px; border: 1px solid var(--line2); margin-bottom: 8px; }
  .chk { background: repeating-conic-gradient(var(--check1) 0 25%, var(--check2) 0 50%) 0 0 / 16px 16px; display: flex; align-items: center; justify-content: center; }
  .small { font-size: 12px; } .dim { color: var(--dim); }
</style>
