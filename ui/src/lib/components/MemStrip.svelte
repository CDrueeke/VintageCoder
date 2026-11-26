<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { bau } from "../bau.svelte";
  import { t } from "../i18n.svelte";
  import Tooltip from "./Tooltip.svelte";
  // Speicherstreifen: wo das gebaute Programm im Adressraum liegt (64 KB, bei größeren Zielen bis zum letzten Segment)
  const segs = $derived(bau.last?.info?.segments ?? []);
  const top = $derived(Math.max(0x10000, ...segs.map(([s, l]) => s + l)));
  const hex = (n: number) => "$" + n.toString(16).toUpperCase().padStart(4, "0");
  const text = $derived(segs.map(([s, l]) => `${hex(s)}–${hex(s + l - 1)}`).join(", "));
</script>

{#if segs.length > 0}
  <Tooltip text={t("bar-speicher", { segs: text })}>
    <span class="strip" role="img" aria-label={t("bar-speicher", { segs: text })}>
      {#each segs as [s, l], i (i)}<i style={`left:${(s / top) * 100}%;width:max(2px,${(l / top) * 100}%)`}></i>{/each}
    </span>
  </Tooltip>
{/if}

<style>
  .strip { position: relative; display: inline-block; width: 120px; height: 8px; border-radius: 4px; background: var(--line); margin: 0 8px; vertical-align: middle; overflow: hidden; }
  i { position: absolute; top: 0; bottom: 0; background: var(--accent2); }
</style>
