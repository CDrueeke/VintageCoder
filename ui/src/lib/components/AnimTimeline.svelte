<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { PixelView } from "../api";
  import { animMode, selectFrame, sp, stepAt } from "../sprite.svelte";
  import FrameThumb from "./FrameThumb.svelte";
  import Icon from "./Icon.svelte";
  import { t } from "../i18n.svelte";
  // Zeitleiste unter der Zeichenfläche: Abspielen der gewählten Animation (50 Bilder je Sekunde, Dauer je Schritt wie im Kern), Ablauf umschalten, Schritte anklicken.
  let { view }: { view: PixelView } = $props();
  let playing = $state(false); let cur = $state(0);
  const anim = $derived(view.anims?.[Math.min(view.selAnim, (view.anims?.length ?? 1) - 1)] ?? null);
  const MODES = ["loop", "pingpong", "once"] as const;
  /** Index des Schritts in `steps` für einen Index des ausgeschriebenen Ablaufs (Hin und zurück läuft rückwärts) */
  const stepIdx = (i: number): number => (anim && i >= anim.steps.length ? Math.max(0, anim.steps.length * 2 - 2 - i) : i);
  $effect(() => {
    if (!playing || !anim) return; const a = anim; const t0 = performance.now(); let raf = 0;
    const tick = () => { cur = stepAt(a.expanded, a.mode, (performance.now() - t0) / 1000); raf = requestAnimationFrame(tick); };
    raf = requestAnimationFrame(tick); return () => cancelAnimationFrame(raf);
  });
  $effect(() => { if (sp.autoplay && anim) { playing = true; cur = 0; sp.autoplay = false; } });
  $effect(() => { void anim?.name; if (!anim) { playing = false; cur = 0; } });
  const frame = $derived(anim ? anim.expanded[cur]?.[0] ?? anim.steps[0]?.[0] ?? 0 : 0);
  const th = $derived(view.thumbs[Math.min(frame, view.thumbs.length - 1)]);
  const ticks = $derived(anim ? anim.expanded.reduce((a, s) => a + s[1], 0) : 0);
  const nextMode = () => { if (!anim) return; const i = MODES.indexOf(anim.mode); void animMode(MODES[(i + 1) % MODES.length]!); };
</script>

<div class="tl" data-sp="timeline" aria-label={t("sprite-zeitleiste")}>
  {#if !anim}
    <p class="small dim">{t("sprite-keine-animation")}</p>
  {:else}
    <button type="button" class="play" class:on={playing} aria-pressed={playing} aria-label={playing ? t("sprite-stopp") : t("sprite-abspielen")} data-act="play" disabled={anim.steps.length === 0} onclick={() => { playing = !playing; if (!playing) cur = 0; }}><Icon name={playing ? "stop" : "play"} size={15} /></button>
    <button type="button" class="btn sm" data-act="anim-mode" title={t("sprite-ablauf")} onclick={nextMode}>{t(`sprite-ablauf-${anim.mode}`)}</button>
    <span class="nm" data-sp="anim-name"><b>{anim.name}</b> <span class="dim small">{t("sprite-anim-dauer", { ticks, s: (ticks / 50).toFixed(2) })}</span></span>
    <button type="button" class="ibtn" data-act="preview-big" title={t("sprite-vorschau-gross")} aria-label={t("sprite-vorschau-gross")} onclick={() => (sp.previewDialog = true)}><Icon name="max" size={16} /></button>
    {#if th}<span class="pv" data-sp="anim-preview" aria-hidden="true"><FrameThumb pixels={th.pixels} w={th.w} h={th.pixels.length / th.w} aspect={th.aspect} colors={th.colors} boxW={48} boxH={40} /></span>{/if}
    <div class="strip" role="group" aria-label={t("sprite-anim-schritte")}>
      {#each anim.steps as st, i (i)}
        {@const tn = view.thumbs[st[0]]}
        <button type="button" class="st" class:on={playing ? stepIdx(cur) === i : st[0] === view.fi} data-step={i} title={t("sprite-schritt-titel", { i: i + 1, f: st[0] + 1, d: st[1] })} onclick={() => void selectFrame(st[0])}>
          <span class="no">{st[0] + 1}</span>{#if tn}<FrameThumb pixels={tn.pixels} w={tn.w} h={tn.pixels.length / tn.w} aspect={tn.aspect} colors={tn.colors} boxW={40} boxH={34} />{/if}</button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .tl { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; min-height: 50px; padding: 6px 2px 0; }
  .play { width: 34px; height: 34px; border-radius: 10px; background: var(--accent); color: var(--on-accent); display: inline-flex; align-items: center; justify-content: center; flex: none; } .play:disabled { opacity: .4; } .play.on { background: var(--accentbg); color: var(--accent); }
  .nm { min-width: 0; } .dim { color: var(--dim); } .small { font-size: 12px; }
  .pv { background: repeating-conic-gradient(var(--check1) 0 25%, var(--check2) 0 50%) 0 0 / 8px 8px; border-radius: 6px; display: inline-flex; }
  .strip { display: flex; gap: 6px; overflow-x: auto; padding: 2px; flex: 1; min-width: 140px; }
  .st { position: relative; border-radius: 8px; border: 1px solid var(--line2); background: var(--raise); padding: 2px; flex: none; } .st.on { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent); }
  .no { position: absolute; left: 4px; top: 1px; font-size: 10px; color: var(--dim); }
</style>
