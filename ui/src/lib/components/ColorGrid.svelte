<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from "../i18n.svelte";
  // Farbwahl aus der Palette des Zielsystems: Gruppe von Farbfeldern (Knöpfe mit Nummer und Farbwert im Namen), das gewählte ist gedrückt
  let { palette, value, onselect, label, size = 18 }: { palette: { i: number; color: string }[]; value: number; onselect: (i: number) => void; label: string; size?: number } = $props();
</script>
<div class="cg" role="group" aria-label={label}>
  {#each palette as p (p.i)}<button type="button" class="f" class:on={p.i === value} aria-pressed={p.i === value} aria-label={t("palette-farbe-ansage", { n: p.i, hex: "$" + p.i.toString(16).toUpperCase().padStart(2, "0"), farbe: p.color.toUpperCase() })} title={`${p.i}`} data-idx={p.i} style={`background:${p.color};width:${size}px;height:${size}px`} onclick={() => onselect(p.i)}></button>{/each}
</div>
<style>
  .cg { display: flex; flex-wrap: wrap; gap: 3px; } .f { border-radius: 4px; border: 1px solid rgba(255, 255, 255, 0.18); padding: 0; cursor: pointer; } .f.on { outline: 2px solid var(--accent); outline-offset: 1px; }
</style>
