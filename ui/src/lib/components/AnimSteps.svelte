<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { PixelView } from "../api";
  import { animAddStep, animDelay, animDelayAll, animDelStep, animMove, animMoveTo, selectFrame } from "../sprite.svelte";
  import { dragSource, dropTarget } from "../dnd.svelte";
  import FrameThumb from "./FrameThumb.svelte";
  import Icon from "./Icon.svelte";
  import { t } from "../i18n.svelte";
  // Schritte einer Animation als Filmstreifen: Miniatur, Nummer, Dauer in Ticks, Pfeile, Entfernen, Ziehen zum Umordnen, „+“ zum Anhängen; dazu das Tempo für alle Schritte in Bildern je Sekunde.
  // Wird im Reiter „Animation“ und in der großen Vorschau benutzt; `playing` hebt den gerade laufenden Schritt hervor.
  type Anim = NonNullable<PixelView["anims"]>[number];
  let { view: v, an, playing = null }: { view: PixelView; an: Anim; playing?: number | null } = $props();
  const fpsOf = (a: { steps: [number, number][] }): string => { const d = new Set(a.steps.map((x) => x[1])); return d.size === 1 ? String(Math.round((50 / [...d][0]!) * 10) / 10) : ""; };
  const ticksText = (a: { steps: [number, number][] }): string => { const d = new Set(a.steps.map((x) => x[1])); return d.size === 1 ? t("sprite-anim-aus-ticks", { t: [...d][0]! }) : t("sprite-anim-tempo-gemischt"); };
</script>

<div class="tempo small"><label class="dim" for="sp-fps">{t("sprite-anim-aus-tempo")}</label>
  <input id="sp-fps" class="inp dl" type="number" min="1" max="50" data-sp="fps" value={fpsOf(an)} onchange={(e) => void animDelayAll(Math.max(1, Math.min(255, Math.round(50 / Math.max(1, Math.min(50, Number(e.currentTarget.value) || 8))))))} />
  <span class="dim" data-sp="fps-ticks">{ticksText(an)}</span></div>
<ol class="strip" data-sp="steps" aria-label={t("sprite-anim-schritte-liste")}>{#each an.steps as st, i (i)}
  <li class="card" class:cur={playing !== null ? playing === i : st[0] === v.fi} data-card={i}
    use:dragSource={() => ({ kind: "step", id: String(i), label: String(i + 1) })}
    use:dropTarget={() => ({ accept: (p) => p.kind === "step" && Number(p.id) !== i, drop: (p) => void animMoveTo(Number(p.id), i) })}>
    <button type="button" class="tb" aria-label={t("sprite-bild-ansage", { i: st[0] + 1, n: v.frames })} data-step-pick={i} onclick={() => void selectFrame(st[0])}>
      {#if v.thumbs[st[0]]}{@const th = v.thumbs[st[0]]!}<FrameThumb pixels={th.pixels} w={th.w} h={th.pixels.length / th.w} aspect={th.aspect} colors={th.colors} boxW={56} boxH={46} />{/if}</button>
    <span class="mono sn">{i + 1}</span>
    <input class="inp dl" type="number" min="1" max="255" value={st[1]} aria-label={t("sprite-anim-dauer-label")} title={t("sprite-anim-dauer-label")} data-step-delay={i} onchange={(e) => void animDelay(i, Number(e.currentTarget.value))} />
    <span class="cb">
      <button type="button" class="ibtn" aria-label={t("sprite-anim-hoch")} disabled={i === 0} data-step-up={i} onclick={() => void animMove(i, -1)}><Icon name="arrl" size={14} /></button>
      <button type="button" class="ibtn" aria-label={t("sprite-anim-runter")} disabled={i === an.steps.length - 1} data-step-down={i} onclick={() => void animMove(i, 1)}><Icon name="arrr" size={14} /></button>
      <button type="button" class="ibtn" aria-label={t("sprite-anim-entfernen")} data-step-del={i} onclick={() => void animDelStep(i)}><Icon name="x" size={14} /></button></span></li>
{/each}
  <li class="card add"><button type="button" class="plus" data-act="step-add" title={t("sprite-anim-anhaengen")} aria-label={t("sprite-anim-anhaengen")} onclick={() => void animAddStep(v.fi)}><Icon name="plus" size={18} /></button></li></ol>
<p class="small dim">{t("sprite-anim-ziehen-hinweis")}</p>

<style>
  .tempo { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
  .strip { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(78px, 1fr)); gap: 6px; }
  .card { position: relative; display: flex; flex-direction: column; align-items: center; gap: 3px; padding: 5px; border-radius: 9px; background: var(--raise); border: 1px solid var(--line2); cursor: grab; }
  .card.cur { outline: 2px solid var(--accent); outline-offset: -2px; } .card .tb { padding: 0; background: none; border: 0; } .card .sn { position: absolute; top: 3px; left: 6px; font-size: 10px; color: var(--dim); }
  .card .dl { width: 54px; } .card .cb { display: flex; gap: 1px; } .card.add { cursor: default; justify-content: center; min-height: 92px; border-style: dashed; } .plus { color: var(--accent2); padding: 14px; background: none; border: 0; }
  .card { position: relative; } .card .sn { position: absolute; top: 3px; left: 6px; font-size: 10px; color: var(--dim); } .card .tb { padding: 0; background: none; border: 0; } .card .cb { display: flex; gap: 1px; }
  .card.add { cursor: default; justify-content: center; min-height: 92px; border-style: dashed; } .plus { color: var(--accent2); padding: 14px; background: none; border: 0; } .dim { color: var(--dim); } .small { font-size: 12px; } .mono { font-family: var(--mono, monospace); }
</style>
