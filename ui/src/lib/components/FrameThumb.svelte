<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { ui } from "../state.svelte";
  // Kleine Vorschau eines Bildes (Bilderliste, Vorschau im Spiel): Pixel, Breite und Farbe je Wert kommen aus dem Kern; durchsichtig bleibt durchsichtig
  let { pixels, w, h, aspect = 1, colors, scale = 0 , boxW = 72, boxH = 56 }: { pixels: number[]; w: number; h: number; aspect?: number; colors: (string | null)[]; scale?: number; boxW?: number; boxH?: number } = $props();
  let cv: HTMLCanvasElement | undefined = $state();
  $effect(() => {
    void [pixels, w, h, aspect, colors, scale, boxW, boxH, ui.theme, cv]; if (!cv) return;
    const dpr = window.devicePixelRatio || 1; cv.width = Math.round(boxW * dpr); cv.height = Math.round(boxH * dpr);
    const g = cv.getContext("2d"); if (!g) return; g.setTransform(dpr, 0, 0, dpr, 0, 0); g.clearRect(0, 0, boxW, boxH);
    const k = scale || Math.max(1, Math.floor(Math.min(boxW / (w * aspect), boxH / h))); const ox = Math.round((boxW - w * aspect * k) / 2), oy = Math.round((boxH - h * k) / 2);
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) { const c = colors[pixels[y * w + x] ?? 0]; if (c) { g.fillStyle = c; g.fillRect(ox + x * aspect * k, oy + y * k, aspect * k, k); } }
  });
</script>

<canvas bind:this={cv} aria-hidden="true" style={`width:${boxW}px;height:${boxH}px`}></canvas>
