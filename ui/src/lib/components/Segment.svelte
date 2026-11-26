<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { Snippet } from "svelte";
  // Gruppe von Alternativen. kind="radio": Auswahl (role=radiogroup); kind="nav": Navigation (aria-current). Pfeiltasten wechseln, Tab verlässt die Gruppe.
  interface Item { id: string; label: string; icon?: string }
  let { items, value, onselect, label, kind = "radio", small = false, item }: { items: Item[]; value: string; onselect: (id: string) => void; label: string; kind?: "radio" | "nav"; small?: boolean; item?: Snippet<[Item]> } = $props();
  let root: HTMLElement;
  function key(e: KeyboardEvent, i: number) {
    const d = e.key === "ArrowRight" || e.key === "ArrowDown" ? 1 : e.key === "ArrowLeft" || e.key === "ArrowUp" ? -1 : 0;
    if (!d) return;
    e.preventDefault();
    const n = (i + d + items.length) % items.length;
    onselect(items[n]!.id);
    queueMicrotask(() => (root.querySelectorAll("button")[n] as HTMLElement | undefined)?.focus());
  }
</script>

<div bind:this={root} class="seg" class:small role={kind === "radio" ? "radiogroup" : "group"} aria-label={label}>
  {#each items as it, i (it.id)}
    {#if kind === "radio"}
      <button type="button" role="radio" aria-checked={it.id === value} tabindex={it.id === value ? 0 : -1} onclick={(e) => { (e.currentTarget as HTMLElement).focus(); onselect(it.id); }} onkeydown={(e) => key(e, i)}>
        {#if item}{@render item(it)}{:else}{it.label}{/if}
      </button>
    {:else}
      <button type="button" aria-current={it.id === value ? "page" : undefined} onclick={(e) => { (e.currentTarget as HTMLElement).focus(); onselect(it.id); }} onkeydown={(e) => key(e, i)}>
        {#if item}{@render item(it)}{:else}{it.label}{/if}
      </button>
    {/if}
  {/each}
</div>
