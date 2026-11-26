<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { Group } from "../groups";
  import { activate, closeEvery, closeOthers, closeTab, copyPath, ed, moveTab, revealIn, shiftTab, tabName } from "../editor.svelte";
  import { t } from "../i18n.svelte";
  import { dragSource, dropTarget } from "../dnd.svelte";
  import AreaButton from "./AreaButton.svelte";
  import Icon from "./Icon.svelte";
  // Reiterleiste: role=tablist, Pfeiltasten wechseln, Ziehen ordnet um, Mittelklick und × schließen, Punkt = ungespeichert
  let { group = 0 }: { group?: Group } = $props();
  const mine = $derived(ed.tabs.filter((x) => x.group === group));
  let drag: string | null = null;
  let over = $state<string | null>(null);
  let menu = $state<{ x: number; y: number; rel: string } | null>(null);
  $effect(() => { const zu = () => { menu = null; }; window.addEventListener("vc:closemenus", zu); return () => window.removeEventListener("vc:closemenus", zu); });
  let menuEl = $state<HTMLElement | undefined>();
  const scratchOf = (rel: string) => ed.tabs.find((x) => x.rel === rel)?.scratch;
  function show(rel: string, x: number, y: number) { menu = { x, y, rel }; activate(rel); queueMicrotask(() => menuEl?.querySelector<HTMLElement>('[role="menuitem"]')?.focus()); }
  function run(f: () => void) { f(); menu = null; }
  function mkey(e: KeyboardEvent) {
    const items = [...(menuEl?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])]; const i = items.indexOf(document.activeElement as HTMLElement);
    if (e.key === "ArrowDown") { e.preventDefault(); items[(i + 1) % items.length]?.focus(); } else if (e.key === "ArrowUp") { e.preventDefault(); items[(i - 1 + items.length) % items.length]?.focus(); } else if (e.key === "Escape") { e.preventDefault(); menu = null; }
  }
  function key(e: KeyboardEvent, rel: string) {
    const i = ed.tabs.findIndex((x) => x.rel === rel);
    const go = (j: number) => { const n = ed.tabs[(j + ed.tabs.length) % ed.tabs.length]; if (n) { activate(n.rel); queueMicrotask(() => (e.currentTarget as HTMLElement).parentElement?.querySelector<HTMLElement>(`[data-rel="${CSS.escape(n.rel)}"]`)?.focus()); } };
    if (e.key === "ArrowRight") { e.preventDefault(); go(i + 1); } else if (e.key === "ArrowLeft") { e.preventDefault(); go(i - 1); }
    else if (e.key === "Delete") { e.preventDefault(); closeTab(rel); }
    else if (e.key === "ContextMenu" || (e.key === "F10" && e.shiftKey)) { e.preventDefault(); const r = (e.currentTarget as HTMLElement).getBoundingClientRect(); show(rel, r.left, r.bottom); }
  }
</script>

<div class="tbar" class:focus={ed.focus === group && ed.tabs.some((x) => x.group === 1)} data-area="dokument">
<div class="tabs" role="tablist" aria-label={t("tabs-label")}>
  {#each mine as tb (tb.rel)}
    <div class="tab" class:on={tb.rel === ed.active} class:over={over === tb.rel} role="tab" tabindex={tb.rel === ed.active ? 0 : -1} data-rel={tb.rel} aria-selected={tb.rel === ed.active} title={tb.title ?? tb.rel} oncontextmenu={(e) => { e.preventDefault(); show(tb.rel, e.clientX, e.clientY); }}
      use:dragSource={() => ({ kind: "tab", id: tb.rel, label: tabName(tb.rel), onstart: () => { drag = tb.rel; ed.dragRel = tb.rel; }, onend: () => { drag = null; over = null; ed.dragRel = null; } })}
      use:dropTarget={() => ({ accept: (p) => p.kind === "tab" && p.id !== tb.rel, enter: () => (over = tb.rel), leave: () => { if (over === tb.rel) over = null; }, drop: (p) => { moveTab(p.id, tb.rel); drag = null; over = null; ed.dragRel = null; } })}
      onclick={() => activate(tb.rel)} onauxclick={(e) => { if (e.button === 1) closeTab(tb.rel); }} onkeydown={(e) => key(e, tb.rel)}>
      <span class="nm">{tabName(tb.rel)}</span>
      {#if tb.dirty}<span class="dot" role="img" aria-label={t("tab-ungespeichert")}></span>{/if}
      <button type="button" class="x" data-nodrag tabindex="-1" aria-label={t("tab-schliessen", { name: tabName(tb.rel) })} onclick={(e) => { e.stopPropagation(); closeTab(tb.rel); }}><Icon name="close" size={12} /></button>
    </div>
  {/each}
</div>
  {#if group === 0 || !ed.tabs.some((x) => x.group === 1)}<span class="tact"><AreaButton area="dokument" /></span>{/if}
</div>
<svelte:window onpointerdown={(e) => { if (menu && !menuEl?.contains(e.target as Node)) menu = null; }} />
{#if menu}
  {@const mrel = menu?.rel ?? ""}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="ctx" role="menu" tabindex="-1" aria-label={t("tabs-label")} bind:this={menuEl} oncontextmenu={(e) => { e.preventDefault(); e.stopPropagation(); }} style={`left:${Math.min(menu?.x ?? 0, window.innerWidth - 230)}px;top:${Math.min(menu?.y ?? 0, window.innerHeight - 220)}px`} onkeydown={mkey}>
    <button type="button" role="menuitem" onclick={() => run(() => closeTab(mrel))}>{t("tab-schliessen", { name: tabName(mrel) })}</button>
    <button type="button" role="menuitem" onclick={() => run(() => closeOthers(mrel))}>{t("tab-andere-schliessen")}</button>
    <button type="button" role="menuitem" onclick={() => run(() => closeEvery())}>{t("tab-alle-schliessen")}</button>
    <button type="button" role="menuitem" onclick={() => run(() => shiftTab(-1))}>{t("tab-nach-links")}</button>
    <button type="button" role="menuitem" onclick={() => run(() => shiftTab(1))}>{t("tab-nach-rechts")}</button>
    {#if !scratchOf(mrel)}
      <button type="button" role="menuitem" onclick={() => run(() => void copyPath(mrel))}>{t("tree-pfad-kopieren")}</button>
      <button type="button" role="menuitem" onclick={() => run(() => void revealIn(mrel))}>{t("tree-im-ordner-zeigen")}</button>
    {/if}
  </div>
{/if}

<style>
  .tbar { display: flex; align-items: flex-end; flex: none; border-bottom: 1px solid var(--line); }
  .tbar .tabs { flex: 1; min-width: 0; border-bottom: 0; }
  .tact { padding: 0 8px 6px 0; }
  .ctx { position: fixed; z-index: 1000; min-width: 200px; padding: 5px; background: var(--card); border: 1px solid var(--line2); border-radius: 10px; box-shadow: var(--shadow); display: flex; flex-direction: column; }
  .ctx button { text-align: left; padding: 7px 10px; border: 0; border-radius: 6px; background: none; color: var(--text); font: inherit; font-size: 13px; cursor: pointer; }
  .ctx button:hover, .ctx button:focus-visible { background: var(--hover); outline: none; }
  .tabs { display: flex; gap: 2px; padding: 0 8px; height: 38px; align-items: flex-end; overflow-x: auto; flex: none; border-bottom: 1px solid var(--line); scrollbar-width: thin; }
  .tab { display: flex; align-items: center; gap: 6px; height: 31px; padding: 0 8px 0 12px; border-radius: 8px 8px 0 0; color: var(--dim); font-size: 12.5px; cursor: pointer; white-space: nowrap; border: 1px solid transparent; border-bottom: 0; }
  .tab:hover { background: var(--hover); }
  .tab.on { background: var(--codebg); color: var(--text); border-color: var(--line); }
  .tab.over { box-shadow: inset 2px 0 0 var(--accent); }
  .tab:focus-visible { box-shadow: var(--focus); outline: none; }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--accent2); flex: none; }
  .x { display: grid; place-items: center; width: 18px; height: 18px; border: 0; border-radius: 5px; background: none; color: var(--faint); cursor: pointer; }
  .x:hover { background: var(--hover); color: var(--text); }
</style>
