<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  // Tabelle mit Auswahl: role=grid, Pfeiltasten wählen die Zeile, aria-selected und aria-rowindex für die Sprachausgabe
  interface Col { id: string; label: string }
  let { cols, rows, label, selected = $bindable(0) }: { cols: Col[]; rows: Record<string, string>[]; label: string; selected?: number } = $props();
  let root: HTMLElement;
  function key(e: KeyboardEvent) {
    const d = e.key === "ArrowDown" ? 1 : e.key === "ArrowUp" ? -1 : e.key === "Home" ? -9999 : e.key === "End" ? 9999 : 0;
    if (!d) return;
    e.preventDefault();
    selected = Math.max(0, Math.min(rows.length - 1, selected + d));
    queueMicrotask(() => (root.querySelectorAll("tbody tr")[selected] as HTMLElement | undefined)?.focus());
  }
</script>

<table bind:this={root} class="tbl" role="grid" aria-label={label} aria-rowcount={rows.length + 1}>
  <thead><tr aria-rowindex="1">{#each cols as c (c.id)}<th role="columnheader" scope="col">{c.label}</th>{/each}</tr></thead>
  <tbody>
    {#each rows as r, i (i)}
      <tr aria-rowindex={i + 2} aria-selected={i === selected} tabindex={i === selected ? 0 : -1} onclick={() => (selected = i)} onkeydown={key}>
        {#each cols as c (c.id)}<td role="gridcell">{r[c.id]}</td>{/each}
      </tr>
    {/each}
  </tbody>
</table>
