<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { PixelView } from "../api";
  import { selectFrame } from "../sprite.svelte";
  import { ui } from "../state.svelte";
  import { t } from "../i18n.svelte";
  // Übersicht aller Zeichen eines Satzes (16 je Zeile) auf EINER Zeichenfläche: Klick wählt, Pfeiltasten bewegen, Pos1/Ende springen. Textalternative: das gewählte Zeichen wird angesagt.
  let { view, marked = [], changed = null }: { view: PixelView; marked?: number[]; changed?: boolean[] | null } = $props();
  let cv: HTMLCanvasElement | undefined = $state(); let breite = $state(320);
  const COLS = 16; const zeilen = $derived(Math.ceil(view.frames / COLS));
  const zelle = $derived(Math.max(10, Math.floor(breite / COLS)));
  const hex = (i: number) => "$" + i.toString(16).toUpperCase().padStart(2, "0");
  $effect(() => {
    void [view.thumbs, view.fi, marked, changed, zelle, ui.theme, cv]; if (!cv) return;
    const dpr = window.devicePixelRatio || 1; const W = zelle * COLS, H = zelle * zeilen;
    cv.width = Math.round(W * dpr); cv.height = Math.round(H * dpr); cv.style.width = `${W}px`; cv.style.height = `${H}px`;
    const g = cv.getContext("2d"); if (!g) return; g.setTransform(dpr, 0, 0, dpr, 0, 0);
    const css = getComputedStyle(cv); const flaeche = css.getPropertyValue("--raise").trim() || "#1c2030", linie = css.getPropertyValue("--line2").trim() || "#333", akzent = css.getPropertyValue("--accent").trim() || "#8b7bff", text = css.getPropertyValue("--text").trim() || "#fff";
    g.fillStyle = flaeche; g.fillRect(0, 0, W, H);
    for (let i = 0; i < view.frames; i++) {
      const th = view.thumbs[i]; if (!th) continue; const cx = (i % COLS) * zelle, cy = Math.floor(i / COLS) * zelle; const h = th.pixels.length / th.w;
      if (th.colors[0]) { g.fillStyle = th.colors[0]; g.fillRect(cx, cy, zelle, zelle); }   // Hintergrundfarbe des Satzes wie auf dem Bildschirm
      const k = Math.max(1, Math.floor((zelle - 2) / Math.max(th.w * th.aspect, h))); const ox = cx + Math.floor((zelle - th.w * th.aspect * k) / 2), oy = cy + Math.floor((zelle - h * k) / 2);
      for (let y = 0; y < h; y++) for (let x = 0; x < th.w; x++) { const v = th.pixels[y * th.w + x] ?? 0; if (v === 0) continue; g.fillStyle = th.colors[v] ?? text; g.fillRect(ox + x * th.aspect * k, oy + y * k, th.aspect * k, k); }
      if (changed?.[i]) { g.fillStyle = akzent; g.beginPath(); g.arc(cx + zelle - 4, cy + 4, 2.2, 0, Math.PI * 2); g.fill(); }   // Punkt: weicht von der Vorlage ab
      if (marked.includes(i)) { g.fillStyle = akzent + "33"; g.fillRect(cx, cy, zelle, zelle); }
    }
    g.strokeStyle = linie; g.lineWidth = 1; g.beginPath();
    for (let c = 0; c <= COLS; c++) { g.moveTo(c * zelle + .5, 0); g.lineTo(c * zelle + .5, H); } for (let r = 0; r <= zeilen; r++) { g.moveTo(0, r * zelle + .5); g.lineTo(W, r * zelle + .5); } g.stroke();
    const sx = (view.fi % COLS) * zelle, sy = Math.floor(view.fi / COLS) * zelle; g.strokeStyle = akzent; g.lineWidth = 2; g.strokeRect(sx + 1, sy + 1, zelle - 2, zelle - 2);
  });
  // Auswahlliste für Bildschirmleser: 256 Optionen (ohne Raster- oder Tabellen-Rolle, die VoiceOver mit langen Hinweisen beantwortet) über der Zeichenfläche; nur die gewählte Option ist per Tab erreichbar (Roving Focus), der Fokus wandert mit den Pfeiltasten von Option zu Option. VoiceOver sagt „Zeichen n, $hh, n von 256“.
  let fokus = $state(0); let zellen: HTMLElement[] = [];
  $effect(() => { fokus = view.fi; });
  function waehle(i: number, fokussieren: boolean) {
    if (i < 0 || i >= view.frames) return; fokus = i; if (fokussieren) zellen[i]?.focus(); if (i !== view.fi) void selectFrame(i);
  }
  function taste(e: KeyboardEvent) {
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : e.key === "ArrowDown" ? COLS : e.key === "ArrowUp" ? -COLS : e.key === "Home" ? -fokus : e.key === "End" ? view.frames - 1 - fokus : e.key === "PageDown" ? COLS * 4 : e.key === "PageUp" ? -COLS * 4 : 0;
    if (!d) return; e.preventDefault(); e.stopPropagation(); waehle(Math.max(0, Math.min(view.frames - 1, fokus + d)), true);
  }
</script>

<div class="wrap" bind:clientWidth={breite}>
  <div class="gridbox" data-cs="grid" style={`width:${zelle * COLS}px;height:${zelle * zeilen}px`}>
    <canvas bind:this={cv} aria-hidden="true"></canvas>
    <div class="ov" role="listbox" aria-label={t("zeichensatz-uebersicht-kurz")} tabindex="-1" style={`grid-template-columns:repeat(${COLS}, ${zelle}px);grid-auto-rows:${zelle}px`} onkeydown={taste}>
      {#each Array.from({ length: view.frames }, (_, i) => i) as i (i)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div role="option" class="zl" bind:this={zellen[i]} tabindex={i === fokus ? 0 : -1} aria-selected={i === view.fi} aria-setsize={view.frames} aria-posinset={i + 1}
          aria-label={t("zeichensatz-ansage", { n: i, hex: hex(i) })} data-cs="zelle" data-i={i} onclick={() => waehle(i, true)}></div>
      {/each}
    </div>
  </div>
</div>

<style>
  .wrap { width: 100%; } canvas { display: block; border-radius: 6px; }
  .gridbox { position: relative; } .ov { position: absolute; inset: 0; }
  .ov { display: grid; } .zl { cursor: pointer; }
  .zl:focus-visible { outline: 2px solid var(--accent); outline-offset: -3px; }
</style>
