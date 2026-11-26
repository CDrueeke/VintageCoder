<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { onDestroy } from "svelte";
  import type { Snippet } from "svelte";
  // Tooltip bei Maus und bei Tastaturfokus; Text hängt per aria-describedby am Inhalt (Sprachausgabe liest ihn nach dem Namen)
  let { text, children }: { text: string; children: Snippet } = $props();
  let show = $state(false);
  let x = $state(0), y = $state(0);
  let wrap: HTMLElement;
  let tip = $state<HTMLElement | undefined>();
  let anchor = { top: 0, bottom: 0 };
  let timer: ReturnType<typeof setTimeout> | undefined;
  const id = `tip-${Math.random().toString(36).slice(2, 8)}`;
  let pointer: { x: number; y: number } | null = null;
  // Hohe Auslöser (z. B. der Streifen der eingeklappten Seitenleiste): der Tooltip erscheint am Mauszeiger, sonst unter dem Auslöser
  function enter(e?: Event) {
    pointer = e instanceof PointerEvent ? { x: e.clientX, y: e.clientY } : null;
    timer = setTimeout(() => { if (!wrap) return; const r = wrap.getBoundingClientRect(); anchor = { top: pointer && r.height > 48 ? pointer.y : r.top, bottom: pointer && r.height > 48 ? pointer.y : r.bottom }; if (pointer && r.height > 48) { x = pointer.x + 14; y = pointer.y + 18; } else { x = r.left; y = r.bottom + 6; } show = true; }, 350);
  }
  onDestroy(() => clearTimeout(timer));
  // Am Fenster- oder Bildschirmrand anpassen: nach oben aufklappen, wenn unten kein Platz ist; rechts nicht abschneiden
  $effect(() => {
    if (!show || !tip) return;
    const r = tip.getBoundingClientRect(); let nx = x, ny = y;
    if (r.bottom > window.innerHeight - 6) ny = Math.max(6, anchor.top - r.height - 8);
    if (r.right > window.innerWidth - 6) nx = Math.max(6, window.innerWidth - r.width - 8);
    if (nx !== x) x = nx; if (ny !== y) y = ny;
  });
  function leave() { clearTimeout(timer); show = false; }
</script>

<span bind:this={wrap} role="presentation" aria-describedby={id} style="display:inline-flex;max-width:100%" onpointerenter={enter} onpointerleave={leave} onfocusin={enter} onfocusout={leave} onkeydown={(e) => e.key === "Escape" && leave()}>
  {@render children()}
</span>
<span {id} bind:this={tip} role="tooltip" class={show ? "tip" : "sr-only"} style={show ? `left:${x}px;top:${y}px` : undefined}>{text}</span>
