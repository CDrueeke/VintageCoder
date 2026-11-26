<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from "../i18n.svelte";
  // Trennlinie zum Verstellen einer Größe: Ziehen mit der Maus, Pfeiltasten (Umschalt = große Schritte), Doppelklick stellt die Vorgabe her
  let { value, min, max, label, dir, reset, onchange }: { value: number; min: number; max: number; label: string; dir: "up" | "down" | "left" | "right"; reset: number; onchange: (v: number) => void } = $props();
  // dir: Richtung, in die der Zeiger bewegt wird, damit der Bereich WÄCHST (up: Bereich unten, Linie oben; left: Bereich rechts, Linie links)
  const horiz = $derived(dir === "left" || dir === "right");
  const sign = $derived(dir === "up" || dir === "left" ? 1 : -1);
  let start: { p: number; v: number } | null = null;
  function down(e: PointerEvent) { start = { p: horiz ? e.clientX : e.clientY, v: value }; (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId); e.preventDefault(); }
  function move(e: PointerEvent) { if (!start) return; const p = horiz ? e.clientX : e.clientY; onchange(start.v - sign * (p - start.p)); }
  function up() { start = null; }
  function key(e: KeyboardEvent) {
    const step = e.shiftKey ? 50 : 10; const grow = horiz ? (e.key === "ArrowLeft" ? 1 : e.key === "ArrowRight" ? -1 : 0) : (e.key === "ArrowUp" ? 1 : e.key === "ArrowDown" ? -1 : 0);
    if (grow && sign === 1) { e.preventDefault(); onchange(value + grow * step); } else if (grow && sign === -1) { e.preventDefault(); onchange(value - grow * step); }
    else if (e.key === "Home") { e.preventDefault(); onchange(min); } else if (e.key === "End") { e.preventDefault(); onchange(max); }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div class="sp" class:h={horiz} role="separator" tabindex="0" aria-orientation={horiz ? "vertical" : "horizontal"} aria-label={label} aria-valuenow={value} aria-valuemin={min} aria-valuemax={max} title={t("splitter-hilfe")}
  onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={up} onkeydown={key} ondblclick={() => onchange(reset)}></div>

<style>
  .sp { flex: none; position: relative; background: transparent; touch-action: none; }
  .sp:not(.h) { height: 6px; margin: -3px 0; cursor: row-resize; z-index: 2; }
  .sp.h { width: 6px; margin: 0 -3px; cursor: col-resize; z-index: 2; }
  .sp:hover, .sp:focus-visible, .sp:active { background: color-mix(in srgb, var(--accent) 45%, transparent); outline: none; }
</style>
