<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { ed } from "../editor.svelte";
  import { lay, setSize, toggleMax } from "../layout.svelte";
  import { SIZES } from "../layout";
  import AreaButton from "./AreaButton.svelte";
  import Splitter from "./Splitter.svelte";
  import { t } from "../i18n.svelte";
  import Icon from "./Icon.svelte";
  import OutlinePanel from "./OutlinePanel.svelte";
  import OutputPanel from "./OutputPanel.svelte";
  import ProblemsPanel from "./ProblemsPanel.svelte";
  import TestsPanel from "./TestsPanel.svelte";
  import SearchPanel from "./SearchPanel.svelte";
  // Ablage unter dem Editor: Probleme, Suche im Projekt, Gliederung (Reiter mit Pfeiltasten)
  const tabs = ["problems", "output", "tests", "search", "outline"] as const;
  function key(e: KeyboardEvent, i: number) {
    const go = (j: number) => { const id = tabs[(j + tabs.length) % tabs.length]!; ed.dock = id; queueMicrotask(() => (e.currentTarget as HTMLElement).parentElement?.querySelector<HTMLElement>(`[data-tab="${id}"]`)?.focus()); };
    if (e.key === "ArrowRight") { e.preventDefault(); go(i + 1); } else if (e.key === "ArrowLeft") { e.preventDefault(); go(i - 1); }
  }
</script>

{#if ed.dock}
  {#if lay.max !== "schublade"}<Splitter value={lay.dockH} min={SIZES.dockH.min} max={SIZES.dockH.max} label={t("splitter-dock")} dir="up" reset={SIZES.dockH.def} onchange={(v) => setSize("dockH", v)} />{/if}
  <section class="dock" class:full={lay.max === "schublade"} style={lay.max === "schublade" ? "" : `height:${lay.dockH}px`} data-area="schublade" aria-label={t("dock-label")}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <header ondblclick={(e) => { if (!(e.target as HTMLElement).closest("button")) toggleMax("schublade"); }}>
      <div role="tablist" aria-label={t("dock-label")}>
        {#each tabs as id, i (id)}
          <button type="button" role="tab" class="dt" data-tab={id} aria-selected={ed.dock === id} tabindex={ed.dock === id ? 0 : -1} onclick={() => (ed.dock = id)} onkeydown={(e) => key(e, i)}>{t(`dock-${id}`)}</button>
        {/each}
      </div>
      <span class="acts"><AreaButton area="schublade" /><button type="button" class="ibtn" aria-label={t("dock-close")} onclick={() => { if (lay.max === "schublade") lay.max = null; ed.dock = null; }}><Icon name="close" size={12} /></button></span>
    </header>
    <div class="body" role="tabpanel">
      {#if ed.dock === "problems"}<ProblemsPanel />{:else if ed.dock === "output"}<OutputPanel />{:else if ed.dock === "tests"}<TestsPanel />{:else if ed.dock === "search"}<SearchPanel />{:else}<OutlinePanel />{/if}
    </div>
  </section>
{/if}

<style>
  .dock.full { flex: 1; height: auto; }
  .acts { display: flex; align-items: center; gap: 2px; }
  .dock { border-top: 1px solid var(--line); background: var(--raise); min-height: 90px; display: flex; flex-direction: column; flex: none; }
  header { display: flex; align-items: center; justify-content: space-between; padding: 2px 8px 0 6px; border-bottom: 1px solid var(--line); }
  [role="tablist"] { display: flex; gap: 2px; }
  .dt { border: 0; background: none; color: var(--dim); font: inherit; font-size: 12.5px; padding: 5px 10px; border-bottom: 2px solid transparent; cursor: pointer; }
  .dt[aria-selected="true"] { color: var(--text); border-bottom-color: var(--accent); }
  .dt:focus-visible { outline: none; box-shadow: var(--focus); border-radius: 6px; }
  .body { flex: 1; min-height: 0; display: flex; flex-direction: column; }
</style>
