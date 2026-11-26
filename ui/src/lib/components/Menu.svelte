<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script module lang="ts">
  export interface MenuItem { id: string; label: string; checked?: boolean; disabled?: boolean }
</script>

<script lang="ts">
  import { tick, type Snippet } from "svelte";
  // Aufklappmenü: Knopf mit aria-haspopup, Einträge role=menuitem/menuitemradio, Pfeiltasten, Esc, Klick daneben schließt
  let { label, items, onselect, radio = false, button, align = "left" }: { label: string; items: MenuItem[]; onselect: (id: string) => void; radio?: boolean; button: Snippet; align?: "left" | "right" } = $props();
  let open = $state(false);
  let pos = $state({ top: 0, left: 0, right: 0, side: "left" as "left" | "right", ready: false });   // Menü liegt über allem (fixed) und klappt am Fensterrand um
  let root: HTMLElement;
  let btn: HTMLButtonElement;
  function items_(): HTMLElement[] { return [...root.querySelectorAll<HTMLElement>('[role^="menuitem"]:not([aria-disabled="true"])')]; }
  async function toggle() {
    open = !open;
    if (open) {
      const r = btn.getBoundingClientRect(); pos = { top: r.bottom + 4, left: r.left, right: Math.max(8, window.innerWidth - r.right), side: align, ready: false };
      await tick();
      const pop = root.querySelector<HTMLElement>(".popup");   // nach dem Zeichnen messen: unten zu eng, dann über den Knopf; rechts zu eng, dann nach links
      if (pop) { const h = pop.offsetHeight, w = pop.offsetWidth; let top = r.bottom + 4; if (top + h > window.innerHeight - 8) top = Math.max(8, r.top - h - 4); pos.top = top; if (pos.side === "left" && r.left + w > window.innerWidth - 8) pos.side = "right"; if (pos.side === "right" && r.right - w < 8) pos.side = "left"; }
      pos.ready = true;
      queueMicrotask(() => (items_().find((x) => x.getAttribute("aria-checked") === "true") ?? items_()[0])?.focus());
    }
  }
  function close(focusBtn = true) { open = false; if (focusBtn) btn?.focus(); }
  function key(e: KeyboardEvent) {
    const list = items_(); const i = list.indexOf(document.activeElement as HTMLElement);
    if (e.key === "ArrowDown") { e.preventDefault(); list[(i + 1) % list.length]?.focus(); }
    else if (e.key === "ArrowUp") { e.preventDefault(); list[(i - 1 + list.length) % list.length]?.focus(); }
    else if (e.key === "Escape") { e.preventDefault(); close(); }
    else if (e.key === "Tab") close(false);
  }
  function pick(it: MenuItem) { if (it.disabled) return; onselect(it.id); close(); }
  function outside(e: PointerEvent) { if (open && !root.contains(e.target as Node)) open = false; }
</script>

<svelte:window onpointerdown={outside} onresize={() => (open = false)} />
<span bind:this={root} style="position:relative;display:inline-flex">
  <button bind:this={btn} type="button" class="ibtn" aria-haspopup="menu" aria-expanded={open} aria-label={label} onclick={toggle}>{@render button()}</button>
  {#if open}
    <div class="popup" role="menu" aria-label={label} tabindex="-1" style="position:fixed;top:{pos.top}px;{pos.side === 'right' ? `right:${pos.right}px` : `left:${pos.left}px`};visibility:{pos.ready ? 'visible' : 'hidden'};max-height:calc(100vh - 16px);overflow:auto" onkeydown={key}>
      {#each items as it (it.id)}
        <button type="button" role={radio ? "menuitemradio" : "menuitem"} aria-checked={radio ? !!it.checked : undefined} aria-disabled={it.disabled ? "true" : undefined} onclick={() => pick(it)}>{it.label}</button>
      {/each}
    </div>
  {/if}
</span>
