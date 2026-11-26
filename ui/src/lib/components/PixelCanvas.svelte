<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { PixelView } from "../api";
  import { ZOOM_MIN, zoomMax, clearSel, copySel, fill, flip, pasteAt, pick, selectAll, selectNone, selectRect, setTool, setValue, shape, sp, stepFrame, stroke, zoomFit, zoomStep } from "../sprite.svelte";
  import { popupMenu } from "../cm-lsp";
  import { t } from "../i18n.svelte";
  import { ui } from "../state.svelte";
  // Zeichenfläche (E3.1): Canvas mit lokalem Vorschaustrich. Beim Ziehen zeichnet die Fläche sofort selbst; beim Loslassen geht der ganze Strich als EIN Vorgang
  // an den Kern (ein Rückgängig-Schritt, Lücken werden dort mit Linien geschlossen). Textalternative: Pfeiltasten bewegen die Zellmarke, Eingabetaste setzt, die Zelle
  // wird angesagt (aria-live). Leertaste + Ziehen verschiebt die Fläche, Cmd/Strg + Mausrad zoomt.
  let { view, fit = $bindable(8) }: { view: PixelView; fit?: number } = $props();
  let box: HTMLDivElement | undefined = $state();
  let cv: HTMLCanvasElement | undefined = $state();
  let W = $state(600), H = $state(400);
  let local = $state.raw(new Uint8Array(0));
  let pan = $state<[number, number]>([0, 0]);
  let ansage = $state("");
  let space = $state(false), spaceUsed = false;
  let zug: { points: [number, number][]; erase: boolean; value: number } | null = null;
  /** Form im Aufbau (Linie, Rahmen, Auswahl): Anfang a, Ende b; `tastatur` = Anfang per Eingabetaste gesetzt, Ende folgt der Zellmarke */
  let form = $state<{ a: [number, number]; b: [number, number]; erase: boolean } | null>(null);
  let anker = $state<[number, number] | null>(null);
  let schieben = $state<{ x: number; y: number; px: number; py: number } | null>(null);

  const w = $derived(view.width), h = $derived(view.height), asp = $derived(view.aspect || 1);
  const fitZ = $derived(Math.max(ZOOM_MIN, Math.min(zoomMax(), Math.floor(Math.min((W - 24) / (w * asp), (H - 24) / h)))));
  $effect(() => { fit = fitZ; });
  const z = $derived(sp.zoom ?? fitZ);
  const cw = $derived(z * asp), ch = $derived(z);
  const ox = $derived(Math.round((W - w * cw) / 2 + pan[0])), oy = $derived(Math.round((H - h * ch) / 2 + pan[1]));

  // Stand des Kerns übernehmen, sobald kein Strich läuft
  $effect(() => { const p = view.pixels; if (!zug) local = Uint8Array.from(p); });
  $effect(() => { void view.fi; pan = [0, 0]; });

  const farbe = (v: number): string | null => view.valueColors[v] ?? null;
  function draw() {
    if (!cv) return;
    const dpr = window.devicePixelRatio || 1; cv.width = Math.round(W * dpr); cv.height = Math.round(H * dpr);
    const g = cv.getContext("2d"); if (!g) return; g.setTransform(dpr, 0, 0, dpr, 0, 0); g.clearRect(0, 0, W, H);
    const css = getComputedStyle(cv); const col = (n: string, f: string) => css.getPropertyValue(n).trim() || f;
    const c1 = col("--check1", "#14171c"), c2 = col("--check2", "#1a1e25"), line = col("--line2", "rgba(255,255,255,.12)"), acc = col("--accent", "#8c7dff");
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
      const c = farbe(local[y * w + x] ?? 0);
      g.fillStyle = c ?? ((x + y) % 2 === 0 ? c1 : c2); g.fillRect(ox + x * cw, oy + y * ch, cw, ch);
    }
    // Onion Skin: das vorherige Bild blass dahinter, nur wo das gewählte Bild durchsichtig ist
    if (sp.onion && view.fi > 0) {
      const pt = view.thumbs[view.fi - 1]; const ph = pt ? pt.pixels.length / pt.w : 0;
      if (pt && ph) { g.globalAlpha = 0.4; for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) { if ((local[y * w + x] ?? 0) !== 0) continue; const c = pt.colors[pt.pixels[Math.min(ph - 1, y) * pt.w + Math.min(pt.w - 1, Math.floor((x * pt.w) / w))] ?? 0]; if (c) { g.fillStyle = c; g.fillRect(ox + x * cw, oy + y * ch, cw, ch); } } g.globalAlpha = 1; }
    }
    if (sp.grid && Math.min(cw, ch) >= 6) {
      g.lineWidth = 1; g.strokeStyle = line; g.beginPath();
      for (let x = 0; x <= w; x++) { g.moveTo(ox + x * cw + 0.5, oy); g.lineTo(ox + x * cw + 0.5, oy + h * ch); }
      for (let y = 0; y <= h; y++) { g.moveTo(ox, oy + y * ch + 0.5); g.lineTo(ox + w * cw, oy + y * ch + 0.5); }
      g.stroke();
      // Achsenmarken: kräftigere Linie je Byte (8 Bildpunkte breit)
      const je = Math.max(1, Math.round(8 / asp)); g.strokeStyle = acc; g.globalAlpha = 0.35; g.beginPath();
      for (let x = je; x < w; x += je) { g.moveTo(ox + x * cw + 0.5, oy); g.lineTo(ox + x * cw + 0.5, oy + h * ch); }
      g.stroke(); g.globalAlpha = 1;
    }
    // Form im Aufbau: Vorschau in der gewählten Farbe (Linie, Rahmen); die Auswahl als gestrichelter Rahmen
    const f = form ?? (anker && sp.cursor ? { a: anker, b: sp.cursor, erase: false } : null);
    if (f && (view.tool === "line" || view.tool === "rect")) {
      const v = f.erase ? 0 : Math.min(view.pv, view.values.length - 1); const c = farbe(v);
      g.fillStyle = c ?? col("--text", "#fff");
      for (const [x, y] of view.tool === "line" ? linieZellen(f.a, f.b) : rahmenZellen(f.a, f.b)) g.fillRect(ox + x * cw, oy + y * ch, cw, ch);
      g.globalAlpha = 1;
    }
    const sel = f && view.tool === "select" ? [Math.min(f.a[0], f.b[0]), Math.min(f.a[1], f.b[1]), Math.max(f.a[0], f.b[0]), Math.max(f.a[1], f.b[1])] : view.sel;
    if (sel) { g.setLineDash([5, 4]); g.lineWidth = 2; g.strokeStyle = col("--text", "#fff"); g.strokeRect(ox + sel[0]! * cw + 1, oy + sel[1]! * ch + 1, (sel[2]! - sel[0]! + 1) * cw - 2, (sel[3]! - sel[1]! + 1) * ch - 2); g.strokeStyle = acc; g.lineDashOffset = 4.5; g.strokeRect(ox + sel[0]! * cw + 1, oy + sel[1]! * ch + 1, (sel[2]! - sel[0]! + 1) * cw - 2, (sel[3]! - sel[1]! + 1) * ch - 2); g.setLineDash([]); g.lineDashOffset = 0; }
    g.strokeStyle = col("--line2", "#444"); g.lineWidth = 1; g.strokeRect(ox - 0.5, oy - 0.5, w * cw + 1, h * ch + 1);
    const c = sp.cursor; if (c) { g.strokeStyle = acc; g.lineWidth = 2; g.strokeRect(ox + c[0] * cw + 1, oy + c[1] * ch + 1, cw - 2, ch - 2); }
  }
  $effect(() => { void [view, local, W, H, z, ox, oy, sp.grid, sp.onion, sp.cursor, ui.theme, cv, form, anker]; draw(); });
  $effect(() => { if (!box) return; const ro = new ResizeObserver(() => { W = box!.clientWidth; H = box!.clientHeight; }); ro.observe(box); W = box.clientWidth; H = box.clientHeight; return () => ro.disconnect(); });

  const zelle = (e: PointerEvent | WheelEvent): [number, number] | null => {
    const r = cv!.getBoundingClientRect(); const x = Math.floor((e.clientX - r.left - ox) / cw), y = Math.floor((e.clientY - r.top - oy) / ch);
    return x >= 0 && y >= 0 && x < w && y < h ? [x, y] : null;
  };
  const wertName = (x: number, y: number): string => { const v = local[y * w + x] ?? 0; return view.values[v]?.name || t("sprite-wert", { n: v + 1 }); };
  const sage = (x: number, y: number) => { ansage = t("sprite-zelle-ansage", { x, y, wert: wertName(x, y) }); };

  function linieZellen(a: [number, number], b: [number, number]): [number, number][] {
    const out: [number, number][] = []; let [x0, y0] = a; const [x1, y1] = b; const dx = Math.abs(x1 - x0), dy = -Math.abs(y1 - y0), sx = x0 < x1 ? 1 : -1, sy = y0 < y1 ? 1 : -1; let err = dx + dy;
    for (;;) { out.push([x0, y0]); if (x0 === x1 && y0 === y1) break; const e2 = 2 * err; if (e2 >= dy) { err += dy; x0 += sx; } if (e2 <= dx) { err += dx; y0 += sy; } }
    return out;
  }
  function rahmenZellen(a: [number, number], b: [number, number]): [number, number][] {
    const [x0, x1, y0, y1] = [Math.min(a[0], b[0]), Math.max(a[0], b[0]), Math.min(a[1], b[1]), Math.max(a[1], b[1])]; const out: [number, number][] = [];
    for (let x = x0; x <= x1; x++) { out.push([x, y0]); if (y1 !== y0) out.push([x, y1]); } for (let y = y0 + 1; y < y1; y++) { out.push([x0, y]); if (x1 !== x0) out.push([x1, y]); }
    return out;
  }
  $effect(() => { void view.tool; anker = null; form = null; });
  const FORMEN = ["line", "rect", "select"];
  /** Vorschau: Linie von a nach b in den lokalen Stand zeichnen */
  function vorschau(a: [number, number] | null, b: [number, number], v: number) {
    let [x0, y0] = a ?? b; const [x1, y1] = b; const dx = Math.abs(x1 - x0), dy = -Math.abs(y1 - y0), sx = x0 < x1 ? 1 : -1, sy = y0 < y1 ? 1 : -1; let err = dx + dy;
    for (;;) { local[y0 * w + x0] = v; if (x0 === x1 && y0 === y1) break; const e2 = 2 * err; if (e2 >= dy) { err += dy; x0 += sx; } if (e2 <= dx) { err += dx; y0 += sy; } }
    local = local.slice();
  }

  function down(e: PointerEvent) {
    cv!.parentElement!.focus();
    if (e.button === 1 || space) { schieben = { x: e.clientX, y: e.clientY, px: pan[0], py: pan[1] }; spaceUsed = true; cv!.setPointerCapture(e.pointerId); return; }
    if (e.button !== 0 && e.button !== 2) return;
    const c = zelle(e); if (!c) return; sp.cursor = c;
    const rechts = e.button === 2;
    if (view.tool === "select" && rechts) return;                                   // Rechtsklick öffnet dort das Menü
    if (view.tool === "pick" && !rechts) { void pick(c[0], c[1]); return; }
    if (view.tool === "fill") { void fill(c[0], c[1], rechts); return; }
    if (FORMEN.includes(view.tool)) { cv!.setPointerCapture(e.pointerId); anker = null; form = { a: c, b: c, erase: rechts }; return; }
    const erase = rechts || view.tool === "erase"; const v = erase ? 0 : Math.min(view.pv, view.values.length - 1);
    cv!.setPointerCapture(e.pointerId); zug = { points: [c], erase, value: v }; vorschau(null, c, v);
  }
  function move(e: PointerEvent) {
    if (schieben) { pan = [schieben.px + e.clientX - schieben.x, schieben.py + e.clientY - schieben.y]; return; }
    const c = zelle(e); sp.cursor = zug || c ? c ?? sp.cursor : null;
    if (form && c) { form = { ...form, b: c }; return; }
    if (zug && c) { const last = zug.points[zug.points.length - 1]!; if (c[0] !== last[0] || c[1] !== last[1]) { zug.points.push(c); vorschau(last, c, zug.value); } }
  }
  function up(e: PointerEvent) {
    if (schieben) { schieben = null; if (cv!.hasPointerCapture(e.pointerId)) cv!.releasePointerCapture(e.pointerId); return; }
    if (form) {
      const f = form; form = null; if (cv!.hasPointerCapture(e.pointerId)) cv!.releasePointerCapture(e.pointerId);
      if (view.tool === "select") { if (f.a[0] === f.b[0] && f.a[1] === f.b[1]) void selectNone(); else void selectRect(f.a, f.b); }
      else void shape(view.tool as "line" | "rect", f.a, f.b, f.erase);
      return;
    }
    if (!zug) return; const z0 = zug; zug = null; if (cv!.hasPointerCapture(e.pointerId)) cv!.releasePointerCapture(e.pointerId);
    void stroke(z0.points, z0.erase);
  }
  function leave() { if (!zug) sp.cursor = null; }
  function wheel(e: WheelEvent) {
    if (!(e.metaKey || e.ctrlKey)) return; e.preventDefault(); zoomStep(e.deltaY < 0 ? 1 : -1, fitZ);
  }

  function menue(e: MouseEvent) {
    e.preventDefault(); if (view.tool !== "select") return;
    const c = zelle(e as unknown as PointerEvent); if (c) sp.cursor = c;
    popupMenu(null, e.clientX, e.clientY, t("sprite-auswahl-menue"), [
      { label: t("cmd-kopieren"), disabled: !view.sel, run: () => void copySel(false) },
      { label: t("cmd-ausschneiden"), disabled: !view.sel, run: () => void copySel(true) },
      { label: t("cmd-einfuegen"), disabled: !view.hasClip, run: () => void pasteAt() },
      { label: t("cmd-alle_auswaehlen"), run: () => void selectAll() },
      { label: t("sprite-auswahl-loeschen"), disabled: !view.sel, run: () => void clearSel() },
      { label: t("sprite-auswahl-aufheben"), disabled: !view.sel, run: () => void selectNone() },
    ]);
  }
  function key(e: KeyboardEvent) {
    if (e.key === "PageUp" || e.key === "PageDown" || (e.altKey && !e.metaKey && !e.ctrlKey && (e.key === "ArrowLeft" || e.key === "ArrowRight"))) {
      e.preventDefault(); const d = e.key === "PageUp" || e.key === "ArrowLeft" ? -1 : 1; void stepFrame(d).then(() => { ansage = t("sprite-bild-ansage", { i: (sp.view?.fi ?? 0) + 1, n: sp.view?.frames ?? 1 }); }); return;
    }
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const k = e.key; const c = sp.cursor ?? [0, 0] as [number, number];
    const d = k === "ArrowRight" ? [1, 0] : k === "ArrowLeft" ? [-1, 0] : k === "ArrowDown" ? [0, 1] : k === "ArrowUp" ? [0, -1] : null;
    if (d) { e.preventDefault(); const n: [number, number] = [Math.max(0, Math.min(w - 1, c[0] + d[0]!)), Math.max(0, Math.min(h - 1, c[1] + d[1]!))]; sp.cursor = n; sage(n[0], n[1]); return; }
    if (k === " ") { e.preventDefault(); space = true; return; }
    if (k === "Enter") { e.preventDefault(); setzen(c); return; }
    if ((k === "Delete" || k === "Backspace") && view.tool === "select" && view.sel) { e.preventDefault(); void clearSel(); return; }
    if (k === "Delete" || k === "Backspace") { e.preventDefault(); void stroke([c], true).then(() => sage(c[0], c[1])); return; }
    if (/^[1-9]$/.test(k)) { const v = Number(k) - 1; if (v < view.values.length) { void setValue(v); ansage = t("sprite-wert-gewaehlt", { n: v + 1, name: view.values[v]?.name ?? "" }); } return; }
    const lk = k.toLowerCase();
    if (k === "Escape") { if (anker) { anker = null; ansage = t("sprite-abgebrochen"); } else if (view.sel) void selectNone(); return; }
    if (lk === "d") void setTool("pen"); else if (lk === "e") void setTool("erase"); else if (lk === "i") void setTool("pick");
    else if (lk === "f") void setTool("fill"); else if (lk === "l") void setTool("line"); else if (lk === "r") void setTool("rect"); else if (lk === "s") void setTool("select");
    else if (lk === "o") sp.onion = !sp.onion; else if (lk === "x") void flip("h"); else if (lk === "y") void flip("v");
    else if (k === "+" || k === "=") zoomStep(1, fitZ); else if (k === "-") zoomStep(-1, fitZ); else if (k === "0") zoomFit(); else if (lk === "g") sp.grid = !sp.grid;
  }
  function keyup(e: KeyboardEvent) { if (e.key === " ") { if (space && !spaceUsed) setzen(sp.cursor ?? [0, 0]); space = false; spaceUsed = false; } }
  function setzen(c: [number, number]) {
    if (!sp.cursor) sp.cursor = c;
    if (view.tool === "pick") { void pick(c[0], c[1]).then(() => ansage = t("sprite-wert-gewaehlt", { n: (sp.view?.pv ?? 0) + 1, name: sp.view?.values[sp.view.pv]?.name ?? "" })); return; }
    if (view.tool === "fill") { void fill(c[0], c[1], false).then(() => sage(c[0], c[1])); return; }
    if (FORMEN.includes(view.tool)) {
      if (!anker) { anker = c; ansage = t("sprite-anfang-gesetzt", { x: c[0], y: c[1] }); return; }
      const a = anker; anker = null;
      if (view.tool === "select") void selectRect(a, c).then(() => { ansage = t("sprite-auswahl-ansage", { w: Math.abs(c[0] - a[0]) + 1, h: Math.abs(c[1] - a[1]) + 1 }); });
      else void shape(view.tool as "line" | "rect", a, c, false).then(() => sage(c[0], c[1]));
      return;
    }
    const erase = view.tool === "erase"; void stroke([c], erase).then(() => sage(c[0], c[1]));
  }
  const status = $derived(sp.cursor ? t("sprite-zelle", { x: sp.cursor[0], y: sp.cursor[1] }) : "");
</script>

<div bind:this={box} class="area">
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div class="wrap" role="application" tabindex="0" aria-label={t("sprite-flaeche-label")} aria-describedby="sp-hilfe" onkeydown={key} onkeyup={keyup} oncopy={(e) => { e.preventDefault(); void copySel(false); }} oncut={(e) => { e.preventDefault(); void copySel(true); }} onpaste={(e) => { e.preventDefault(); void pasteAt(); }}>
    <canvas bind:this={cv} data-ox={ox} data-oy={oy} data-cw={cw} data-ch={ch} data-z={z} style={`width:${W}px;height:${H}px;cursor:${schieben ? "grabbing" : space ? "grab" : "crosshair"}`}
      onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={up} onpointerleave={leave} onwheel={wheel} oncontextmenu={menue}></canvas>
  </div>
  <p id="sp-hilfe" class="sr">{t("sprite-flaeche-hilfe")}</p>
  <p class="sr" role="status" aria-live="polite" data-sp="ansage">{ansage}</p>
  <span class="cur mono" data-sp="zelle" aria-hidden="true">{status}</span>
</div>

<style>
  .area { position: relative; flex: 1; min-height: 0; min-width: 0; background: var(--canvas); border-radius: 10px; overflow: hidden; }
  .wrap { position: absolute; inset: 0; outline: none; } .wrap:focus-visible { box-shadow: inset 0 0 0 2px var(--accent); }
  canvas { display: block; touch-action: none; }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); }
  .cur { position: absolute; right: 12px; bottom: 10px; padding: 3px 9px; border-radius: 12px; background: var(--card); border: 1px solid var(--line2); font-size: 12px; color: var(--dim); pointer-events: none; } .cur:empty { display: none; }
</style>
