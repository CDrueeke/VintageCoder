<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { DiskView } from "../api";
  import { t } from "../i18n.svelte";
  import { ui } from "../state.svelte";
  // Belegungskarte: eine Zeile je Spur, ein Kästchen je Sektor; ab 41 Spuren in zwei Blöcken nebeneinander. Canvas mit Textalternative:
  // Pfeiltasten wählen den Sektor, die Auswahl wird angesagt (aria-live). Die Farben kommen aus den Tokens des Themas.
  let { view, onselect }: { view: DiskView; onselect: (track: number, sector: number) => void } = $props();
  let box: HTMLDivElement | undefined = $state();
  let cv: HTMLCanvasElement | undefined = $state();
  let width = $state(600);
  const n = $derived(view.cells.length);
  const blocks = $derived(n > 40 ? 2 : 1);
  const per = $derived(Math.ceil(n / blocks));
  const maxS = $derived(Math.max(1, ...view.cells.map((r) => r.length)));
  const GAP = 2, BGAP = 14;
  const cw = $derived(Math.max(5, Math.min(26, Math.floor((width - (blocks - 1) * BGAP) / (blocks * maxS)) - GAP)));
  const ch = $derived(Math.max(4, Math.min(13, Math.round(cw * 0.55))));
  const cssH = $derived(per * (ch + GAP));
  const cssW = $derived(blocks * maxS * (cw + GAP) + (blocks - 1) * BGAP);
  const inChain = $derived(new Set(view.chainOn ? view.chain.map((a) => `${a[0]}/${a[1]}`) : []));
  const pos = (tr: number, s: number): [number, number] => { const b = Math.floor((tr - 1) / per), r = (tr - 1) % per; return [b * (maxS * (cw + GAP) + BGAP) + s * (cw + GAP), r * (ch + GAP)]; };

  function draw() {
    if (!cv) return;
    const dpr = window.devicePixelRatio || 1; cv.width = Math.round(cssW * dpr); cv.height = Math.round(cssH * dpr);
    const g = cv.getContext("2d"); if (!g) return; g.setTransform(dpr, 0, 0, dpr, 0, 0); g.clearRect(0, 0, cssW, cssH);
    const css = getComputedStyle(cv); const col = (v: string, f: string) => css.getPropertyValue(v).trim() || f;
    const C = { free: col("--check2", "#222"), used: col("--faint", "#667"), dir: col("--warn", "#fb2"), sys: col("--ok", "#4c8"), sel: col("--accent", "#86f"), bad: col("--err", "#f66"), cur: col("--text", "#fff") };
    view.cells.forEach((row, ti) => row.forEach((c, s) => {
      const [x, y] = pos(ti + 1, s);
      g.fillStyle = inChain.has(`${ti + 1}/${s}`) ? C.sel : c === -1 ? C.free : c === -2 ? C.sys : c === -3 ? C.dir : c <= -4 ? C.bad : C.used;
      g.fillRect(x, y, cw, ch);
    }));
    const [cx, cy] = pos(view.cur[0], view.cur[1]); g.strokeStyle = C.cur; g.lineWidth = 2; g.strokeRect(cx - 1, cy - 1, cw + 2, ch + 2);
  }
  $effect(() => { void [view, cw, ch, cssW, cssH, ui.theme, cv]; draw(); });
  $effect(() => { if (!box) return; const ro = new ResizeObserver(() => { width = box!.clientWidth; }); ro.observe(box); width = box.clientWidth; return () => ro.disconnect(); });

  function hit(e: PointerEvent) {
    const r = cv!.getBoundingClientRect(); const x = e.clientX - r.left, y = e.clientY - r.top;
    const bw = maxS * (cw + GAP) + BGAP; const b = Math.min(blocks - 1, Math.floor(x / bw)); const s = Math.floor((x - b * bw) / (cw + GAP)); const row = Math.floor(y / (ch + GAP));
    const tr = b * per + row + 1; if (tr < 1 || tr > n || s < 0 || s >= view.cells[tr - 1]!.length) return;
    onselect(tr, s);
  }
  // Tasten kurz hintereinander: vom zuletzt gewählten Sektor aus weiterrechnen, auch wenn die Antwort des Kerns noch aussteht
  let local: [number, number] | null = null;
  $effect(() => { void view; local = null; });
  function key(e: KeyboardEvent) {
    let [tr, s] = local ?? view.cur; const d = e.key === "ArrowRight" ? [0, 1] : e.key === "ArrowLeft" ? [0, -1] : e.key === "ArrowDown" ? [1, 0] : e.key === "ArrowUp" ? [-1, 0] : null;
    if (!d) return; e.preventDefault();
    tr = Math.max(1, Math.min(n, tr + d[0]!)); s = Math.max(0, Math.min(view.cells[tr - 1]!.length - 1, s + d[1]!));
    const cur = local ?? view.cur; if (tr !== cur[0] || s !== cur[1]) { local = [tr, s]; onselect(tr, s); }
  }
</script>

<div bind:this={box} class="map">
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div class="wrap" role="application" tabindex="0" aria-label={t("disk-karte-label")} onkeydown={key} onpointerdown={hit} style={`width:${cssW}px;height:${cssH}px`}>
    <canvas bind:this={cv} aria-hidden="true" style={`width:${cssW}px;height:${cssH}px`}></canvas>
  </div>
  <p class="sr" aria-live="polite">{t("disk-karte-ansage", { t: view.cur[0], s: view.cur[1], was: view.owner ?? "" })}</p>
</div>

<style>
  .map { width: 100%; overflow: auto; }
  .wrap { outline-offset: 3px; border-radius: 4px; } .wrap:focus-visible { outline: 2px solid var(--accent); }
  canvas { display: block; cursor: pointer; }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
</style>
