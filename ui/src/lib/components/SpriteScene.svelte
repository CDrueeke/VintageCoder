<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { PixelView } from "../api";
  import { ui } from "../state.svelte";
  // Vorschau im Spiel: das gewählte Bild vor Himmel und Wiese in den Farben des Zielsystems, mit angewandten Flags (doppelt breit/hoch); links vergrößert, rechts in Originalgröße.
  // Der Hintergrund ist fest; später kann ein Bildschirm aus dem Projekt dort stehen.
  let { view }: { view: PixelView } = $props();
  let cv: HTMLCanvasElement | undefined = $state();
  const W = 232, H = 118;
  $effect(() => {
    void [view, ui.theme, cv]; if (!cv) return;
    const dpr = window.devicePixelRatio || 1; cv.width = Math.round(W * dpr); cv.height = Math.round(H * dpr);
    const g = cv.getContext("2d"); if (!g) return; g.setTransform(dpr, 0, 0, dpr, 0, 0);
    g.fillStyle = view.scene.sky ?? "#6c9ed8"; g.fillRect(0, 0, W, H); g.fillStyle = view.scene.ground ?? "#4a8a3a"; g.fillRect(0, H - 22, W, 22);
    const t = view.thumbs[view.fi]; if (!t) return;
    const sz = view.pmg ? [1, 2, 0, 4][view.pmg.sizes[view.fi] ?? 0] || 1 : 1;   // SIZEP: 0 normal, 1 doppelt, 3 vierfach
    const x2 = view.pmg ? sz : view.flags?.x2 ? 2 : 1, y2 = view.flags?.y2 ? 2 : 1;
    const draw = (k: number, left: number) => {
      const top = H - 22 - t.pixels.length / t.w * k * y2 + 4;
      for (let y = 0; y < t.pixels.length / t.w; y++) for (let x = 0; x < t.w; x++) { const c = t.colors[t.pixels[y * t.w + x] ?? 0]; if (!c) continue; g.fillStyle = c; g.fillRect(Math.round(left + x * t.aspect * k * x2), Math.round(top + y * k * y2), Math.ceil(t.aspect * k * x2), Math.ceil(k * y2)); }
    };
    draw(2, 14); draw(1, W - (t.w * t.aspect) * x2 - 14);
  });
</script>

<canvas bind:this={cv} class="scene" aria-hidden="true" style={`width:${W}px;height:${H}px`}></canvas>

<style> .scene { border-radius: 8px; border: 1px solid var(--line2); max-width: 100%; } </style>
