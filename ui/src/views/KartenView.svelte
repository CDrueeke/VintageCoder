<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { openExport } from "../lib/export.svelte";
  import { answerAsk, askName, askNo, askReplace, askYes, brushColor, kt, kartenRedo, kartenUndo, pointer, saveKarten, screenColor, screenDelete, screenDup, screenMove, screenNew, screenOrder, screenRename, screenSize, selChar, selScreen, selTile,
    selectionClear, selectionCopy, selectionMove, selectionNone, setBit7, setFilled, setLayers, setNorm, setMode2, setTab, setTileSet, setTool, showProp, textBack, textNewline, tileFromSelection, typeText } from "../lib/karten.svelte";
  import { bildschirmDateien } from "../lib/bildschirm.svelte";
  import { openKarten } from "../lib/karten.svelte";
  import { pickBildschirm } from "../lib/bildschirm.svelte";
  import ColorGrid from "../lib/components/ColorGrid.svelte";
  import Dialog from "../lib/components/Dialog.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import Segment from "../lib/components/Segment.svelte";
  import Switch from "../lib/components/Switch.svelte";
  import Tooltip from "../lib/components/Tooltip.svelte";
  import { dragSource, dropTarget } from "../lib/dnd.svelte";
  import { singleKeyOk, ui } from "../lib/state.svelte";
  import { t } from "../lib/i18n.svelte";
  // Karten-Editor (E10, nach Mockup neu-9): links Kachelraster (oder Zeichentafel) und Ebenen, in der Mitte Werkzeuge, die Karte und die Leiste aller Karten („Levels“) der Bank, rechts Eigenschaften der Karte,
  // Aufbau, Prüfung und Ausgabe. Die Karte ist ein Bild vom Kern; ein Zug mit der Maus wird als eine Absicht geschickt (ein Rückgängig-Schritt), die berührten Zellen leuchten schon beim Ziehen auf.
  const v = $derived(kt.view);
  const hex = (i: number) => "$" + i.toString(16).toUpperCase().padStart(2, "0");
  const dateien = $derived(bildschirmDateien(ui.project?.tree));
  let askOpen = $state(false); let neuOpen = $state(false);
  $effect(() => { askOpen = kt.ask !== null; });
  $effect(() => { if (!askOpen && kt.ask) void answerAsk("cancel"); });
  $effect(() => { neuOpen = !!v?.ask; });
  $effect(() => { if (!neuOpen && v?.ask) void askNo(); });
  const WERKZEUGE: Record<string, { icon: string; key: string }> = { pen: { icon: "pen", key: "D" }, line: { icon: "line", key: "L" }, rect: { icon: "rect", key: "R" }, fill: { icon: "fill", key: "F" }, pick: { icon: "pipette", key: "I" }, select: { icon: "select", key: "A" }, stamp: { icon: "grid", key: "S" }, text: { icon: "file", key: "T" }, erase: { icon: "eraser", key: "E" } };

  // --- Karte: Zellen unter dem Zeiger ---------------------------------------------------------------------------------------------------------------------------------------------
  let buehne = $state<HTMLElement | undefined>();
  let zug: [number, number][] | null = null; let rechts = false;
  let live = $state<[number, number][]>([]);      // Zellen des laufenden Zugs (sofort sichtbar)
  let gummi = $state<{ a: [number, number]; b: [number, number] } | null>(null);   // Linie, Rahmen und Auswahl: Anfang und Ende
  let cursor = $state<[number, number]>([0, 0]);
  const skala = 2;
  const geo = $derived.by(() => { const m = v?.map; if (!m || !v) return null; const r = v.rand * 8 * skala; return { l: (r / m.w) * 100, t: (r / m.h) * 100, w: ((m.cellsW * 8 * skala) / m.w) * 100, h: ((m.cellsH * 8 * skala) / m.h) * 100 }; });
  /** Zelle unter dem Zeiger; `null` außerhalb der Karte */
  function zelleAn(e: PointerEvent | MouseEvent, klemmen = false): [number, number] | null {
    const m = v?.map; if (!buehne || !m || !geo) return null; const r = buehne.getBoundingClientRect();
    const px = ((e.clientX - r.left) / r.width) * m.w, py = ((e.clientY - r.top) / r.height) * m.h; const rd = v!.rand * 8 * skala;
    let fx = Math.floor((px - rd) / (8 * skala)), fy = Math.floor((py - rd) / (8 * skala));
    if (fx < 0 || fy < 0 || fx >= m.cellsW || fy >= m.cellsH) { if (!klemmen) return null; fx = Math.max(0, Math.min(m.cellsW - 1, fx)); fy = Math.max(0, Math.min(m.cellsH - 1, fy)); }
    return [fx, fy];
  }
  const formWerkzeug = $derived(v ? ["line", "rect", "select"].includes(v.tool) : false);
  function down(e: PointerEvent) {
    if (!v || (e.button !== 0 && e.button !== 2)) return; e.preventDefault(); rechts = e.button === 2; const z = zelleAn(e, true); if (!z) return;
    cursor = z; buehne?.focus(); (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
    if (v.tool === "text" && !rechts) { void textAt(z); return; }
    const loeschZug = rechts && (v.tool === "pen" || v.tool === "erase");   // rechte Maustaste mit Stift: löschen (Hervorhebung wie beim Zeichnen)
    zug = [z]; live = formWerkzeug || (rechts && !loeschZug) ? [] : [z]; gummi = formWerkzeug && !rechts ? { a: z, b: z } : null;
  }
  function move(e: PointerEvent) {
    const z = zelleAn(e, !!zug); if (z) cursor = z; if (!zug || !z) return;
    const last = zug[zug.length - 1]!; if (last[0] === z[0] && last[1] === z[1]) return;
    zug.push(z); if (gummi) gummi = { a: gummi.a, b: z }; else if (!rechts || v?.tool === "pen" || v?.tool === "erase") live = [...live, z];
  }
  function up(e: PointerEvent) {
    if (!zug) return; const z = zelleAn(e, true) ?? zug[zug.length - 1]!; const pts = zug; zug = null;
    if (pts[pts.length - 1]![0] !== z[0] || pts[pts.length - 1]![1] !== z[1]) pts.push(z);
    const wasForm = gummi !== null; gummi = null;
    void pointer(wasForm ? [pts[0]!, pts[pts.length - 1]!] : pts, rechts).finally(() => { live = []; });
  }
  async function textAt(z: [number, number]) { const { kartenOp } = await import("../lib/karten.svelte"); await kartenOp({ kind: "text_at", x: z[0], y: z[1] }); }
  const liveSet = $derived(new Set(live.map(([x, y]) => `${x},${y}`)));
  const einheit = $derived(v ? (v.mode === "tile" ? [v.tw, v.th] : [1, 1]) : [1, 1]);
  /** Kästchen (in Prozent der Karte) für ein Rechteck von Zellen; im Kachelmodus rasten die Ecken am Kachelraster ein */
  function kasten(a: [number, number], b: [number, number]) {
    const m = v!.map!; let [x0, y0, x1, y1] = [Math.min(a[0], b[0]), Math.min(a[1], b[1]), Math.max(a[0], b[0]), Math.max(a[1], b[1])];
    if (v!.mode === "tile") { const [ox, oy] = m.origin; x0 = ox + Math.floor((x0 - ox) / einheit[0]!) * einheit[0]!; y0 = oy + Math.floor((y0 - oy) / einheit[1]!) * einheit[1]!; x1 = ox + Math.floor((x1 - ox) / einheit[0]!) * einheit[0]! + einheit[0]! - 1; y1 = oy + Math.floor((y1 - oy) / einheit[1]!) * einheit[1]! + einheit[1]! - 1; }
    return { l: (x0 / m.cellsW) * 100, t: (y0 / m.cellsH) * 100, w: ((x1 - x0 + 1) / m.cellsW) * 100, h: ((y1 - y0 + 1) / m.cellsH) * 100 };
  }
  const auswahlKasten = $derived(v?.selection && v.map ? (() => { const [x0, y0, x1, y1] = v.selection!; return v.mode === "tile" ? gitterKasten(x0, y0, x1, y1) : kasten([x0, y0], [x1, y1]); })() : null);
  /** Auswahl im Kachelmodus: Koordinaten sind Kacheln des Rasters */
  function gitterKasten(x0: number, y0: number, x1: number, y1: number) { const m = v!.map!; const [ox, oy] = m.origin; return { l: ((ox + x0 * v!.tw) / m.cellsW) * 100, t: ((oy + y0 * v!.th) / m.cellsH) * 100, w: (((x1 - x0 + 1) * v!.tw) / m.cellsW) * 100, h: (((y1 - y0 + 1) * v!.th) / m.cellsH) * 100 }; }
  // Vorschau von Linie und Rechteck: genau die Einheiten (Kacheln bzw. Zellen), die der Kern beim Loslassen setzt (gleiche Algorithmen wie `line_points` und `rect_points` des Modells)
  function aufEinheit(z: [number, number]): [number, number] {
    const m = v!.map!; if (v!.mode !== "tile") return z;
    const gx = Math.floor((z[0] - m.origin[0]) / v!.tw), gy = Math.floor((z[1] - m.origin[1]) / v!.th);
    return [Math.max(0, Math.min(v!.cols - 1, gx)), Math.max(0, Math.min(v!.rows - 1, gy))];
  }
  function linienEinheiten(a: [number, number], b: [number, number]): [number, number][] {
    let [x, y] = a; const [x1, y1] = b; const dx = Math.abs(x1 - x), dy = -Math.abs(y1 - y), sx = x < x1 ? 1 : -1, sy = y < y1 ? 1 : -1; let err = dx + dy; const out: [number, number][] = [];
    for (let i = 0; i < 100000; i++) { out.push([x, y]); if (x === x1 && y === y1) break; const e2 = 2 * err; if (e2 >= dy) { err += dy; x += sx; } if (e2 <= dx) { err += dx; y += sy; } }
    return out;
  }
  function rahmenEinheiten(a: [number, number], b: [number, number], gefuellt: boolean): [number, number][] {
    const [x0, x1, y0, y1] = [Math.min(a[0], b[0]), Math.max(a[0], b[0]), Math.min(a[1], b[1]), Math.max(a[1], b[1])]; const out: [number, number][] = [];
    for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) if (gefuellt || x === x0 || x === x1 || y === y0 || y === y1) out.push([x, y]);
    return out;
  }
  const formEinheiten = $derived.by((): [number, number][] => {
    if (!gummi || !v?.map || (v.tool !== "line" && v.tool !== "rect")) return [];
    const a = aufEinheit(gummi.a), b = aufEinheit(gummi.b); return v.tool === "line" ? linienEinheiten(a, b) : rahmenEinheiten(a, b, v.filled);
  });
  const einheitKasten = (u: [number, number]) => v!.mode === "tile" ? gitterKasten(u[0], u[1], u[0], u[1]) : { l: (u[0] / v!.map!.cellsW) * 100, t: (u[1] / v!.map!.cellsH) * 100, w: 100 / v!.map!.cellsW, h: 100 / v!.map!.cellsH };
  const gummiKasten = $derived(gummi && v?.tool === "select" ? kasten(gummi.a, gummi.b) : null);
  // Eigenschaften zeigen: Kacheln mit der gewählten Eigenschaft werden eingefärbt
  const eigenKaesten = $derived.by(() => {
    const m = v?.map; const bit = v?.brush.prop; if (!v || !m || bit === null || bit === undefined) return [];
    const out: { l: number; t: number; w: number; h: number }[] = [];
    m.grid.forEach((row, gy) => row.forEach((ti, gx) => { const k = v.tiles[ti]; if (k && (k.props & (1 << bit)) !== 0) out.push(gitterKasten(gx, gy, gx, gy)); }));
    return out;
  });
  const ansage = $derived.by(() => {
    if (!v?.map) return ""; const [x, y] = cursor; const m = v.map;
    if (v.mode === "tile") { const gx = Math.floor((x - m.origin[0]) / v.tw), gy = Math.floor((y - m.origin[1]) / v.th); const ti = m.grid[gy]?.[gx]; return ti === undefined ? t("karten-ansage-aussen", { x: x + 1, y: y + 1 }) : t("karten-ansage-kachel", { x: x + 1, y: y + 1, n: ti, name: v.tiles[ti]?.name ?? "" }); }
    return t("karten-ansage-zelle", { x: x + 1, y: y + 1 });
  });
  function bewege(dx: number, dy: number) { const m = v?.map; if (!m) return; cursor = [Math.max(0, Math.min(m.cellsW - 1, cursor[0] + dx)), Math.max(0, Math.min(m.cellsH - 1, cursor[1] + dy))]; }
  /** Bearbeiten-Befehle der Karte: c = Auswahl als Stempel merken, x = ausschneiden (merken und leeren), v = Stempel an der Cursorzelle einsetzen, a = alles auswählen */
  async function bearbeiten(was: string) {
    if (!v?.map) return;
    if (was === "c" || was === "x") { if (!v.selection || v.mode !== "tile") return; await selectionCopy(); if (was === "x") await selectionClear(); }
    else if (was === "v") { if (!v.stamp) return; const alt = v.tool; await setTool("stamp"); await pointer([cursor]); if (alt !== "stamp") await setTool(alt); }   // einmal einsetzen, das Werkzeug bleibt
    else if (was === "a") { await setTool("select"); await pointer([[0, 0], [v.map.cellsW - 1, v.map.cellsH - 1]]); }
  }
  function taste(e: KeyboardEvent) {
    if (!v) return; const ctrl = e.metaKey || e.ctrlKey;
    if (v.tool === "text" && v.mode === "cell" && !ctrl && !e.altKey) {
      if (e.key.length === 1) { e.preventDefault(); void typeText(e.key); return; } if (e.key === "Backspace") { e.preventDefault(); void textBack(); return; } if (e.key === "Enter") { e.preventDefault(); void textNewline(); return; }
    }
    if (ctrl && !e.altKey && e.key.toLowerCase() === "z") { e.preventDefault(); void (e.shiftKey ? kartenRedo() : kartenUndo()); return; }   // Cmd/Strg+Z und +Umschalt+Z auch bei Fokus auf der Karte
    if (ctrl && !e.altKey && !e.shiftKey && "cxva".includes(e.key.toLowerCase()) && e.key.length === 1) { e.preventDefault(); void bearbeiten(e.key.toLowerCase()); return; }   // Kopieren, Ausschneiden, Einfügen, Alles auswählen
    if (ctrl && !e.altKey && e.key.toLowerCase() === "y") { e.preventDefault(); void kartenRedo(); return; }
    const pfeil = ({ ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] } as Record<string, [number, number]>)[e.key];
    if (pfeil) { e.preventDefault(); if (e.altKey && v.selection) { void selectionMove(pfeil[0], pfeil[1]); return; } bewege(pfeil[0] * (e.shiftKey ? 4 : 1), pfeil[1] * (e.shiftKey ? 4 : 1)); return; }
    if (e.key === "Enter" || e.key === " ") { e.preventDefault(); void pointer([cursor], e.key === " " && e.shiftKey); return; }
    if (e.key === "Tab") { e.preventDefault(); void setMode2(v.mode === "tile" ? "cell" : "tile"); return; }
    if (e.key === "Delete" || e.key === "Backspace") { if (v.selection) { e.preventDefault(); void selectionClear(); } return; }
    if (e.key === "Escape") { if (v.selection) { e.preventDefault(); void selectionNone(); } return; }
    if (!singleKeyOk(e) || ctrl || e.altKey) return;
    const werkzeug = Object.entries(WERKZEUGE).find(([, w]) => w.key.toLowerCase() === e.key.toLowerCase())?.[0];
    if (werkzeug && v.tools.some((w) => w.id === werkzeug)) { e.preventDefault(); void setTool(werkzeug); }
  }
  // Zeichentafel (Zellenmodus) mit Roving Focus
  let cfokus = $state(0); let cz: HTMLElement[] = [];
  $effect(() => { if (v) cfokus = v.brush.cell.code; });
  function cWaehle(c: number, fok: boolean) { if (!v || c < 0 || c >= v.tafel.n) return; cfokus = c; if (fok) cz[c]?.focus(); void selChar(c); }
  function cTaste(e: KeyboardEvent) {
    if (!v) return; const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : e.key === "ArrowDown" ? 16 : e.key === "ArrowUp" ? -16 : e.key === "Home" ? -cfokus : e.key === "End" ? v.tafel.n - 1 - cfokus : 0;
    if (!d) return; e.preventDefault(); e.stopPropagation(); cWaehle(Math.max(0, Math.min(v.tafel.n - 1, cfokus + d)), true);
  }
  // Größe der Karte (rechts): Eingaben gelten beim Verlassen des Feldes
  let gw = $state(0), gh = $state(0), gox = $state(0), goy = $state(0), nameFeld = $state("");
  $effect(() => { const m = v?.map; if (m) { gw = m.cellsW; gh = m.cellsH; gox = m.origin[0]; goy = m.origin[1]; nameFeld = m.name; } });
  const groesseSetzen = () => { if (v && (gw !== v.map?.cellsW || gh !== v.map?.cellsH || gox !== v.map?.origin[0] || goy !== v.map?.origin[1])) void screenSize(Math.max(1, gw | 0), Math.max(1, gh | 0), Math.max(0, gox | 0), Math.max(0, goy | 0)); };
  const umbenennen = () => { if (v?.map && nameFeld.trim() && nameFeld.trim() !== v.map.name) void screenRename(nameFeld.trim()); };
  const rollen = [{ id: "bg", label: "karten-farbe-hintergrund" }, { id: "border", label: "karten-farbe-rand" }, { id: "fg", label: "karten-farbe-zeichen" }];
  let bit7 = $state(false); let zoom = $state(1); let gitter = $state(false); let grenzen = $state(false);
  const normen = $derived([{ id: "project", label: t("sprite-ansicht-projekt") }, { id: "pal", label: "PAL" }, { id: "ntsc", label: "NTSC" }]);
  const zoomStufen = [0.5, 0.75, 1, 1.5, 2, 3];
  const zoomSchritt = (d: number) => { const i = Math.max(0, Math.min(zoomStufen.length - 1, zoomStufen.indexOf(zoom) + d)); zoom = zoomStufen[i]!; };
  let askName_ = $state("");
  $effect(() => { askName_ = v?.ask?.name ?? ""; });
</script>

{#if !v}
  <div class="blank" data-ka="start">
    <Icon name="grid" size={28} />
    <h2>{t("karten-leer-titel")}</h2>
    {#if !ui.project}<p>{t("karten-leer-kein-projekt")}</p>
    {:else}
      <p>{t("karten-leer-text")}</p>
      {#if dateien.length}<ul class="files" aria-label={t("karten-dateien")}>{#each dateien as f (f)}<li><button type="button" class="btn" data-ka="open" onclick={() => void openKarten(f)}>{f}</button></li>{/each}</ul>{:else}<p class="dim small">{t("karten-keine-dateien")}</p>{/if}
      <div class="row"><button type="button" class="btn pri" onclick={() => void pickBildschirm()}>{t("karten-datei-oeffnen")}</button></div>
    {/if}
  </div>
{:else}
  <div class="ke" aria-label={t("view-title-karten")} data-ka="editor">
    <section class="col l" aria-label={t("karten-links")}>
      {#if v.mode === "tile"}
        <h3>{t("karten-kacheln")} <span class="dim n">{v.setRel ? v.setRel.split("/").pop() : "–"}</span></h3>
        {#if v.setError}<p class="small bad" role="alert" data-ka="satzfehler">{v.setError}</p>{/if}
        <div class="raster" role="listbox" aria-label={t("karten-kachelliste", { n: v.tiles.length })} data-ka="kacheln">
          {#each v.tiles as k (k.i)}<button type="button" role="option" class="kz" class:on={k.i === v.brush.tile} aria-selected={k.i === v.brush.tile} aria-label={t("karten-kachel-ansage", { n: k.i, name: k.name })} title={`${k.i} · ${k.name}`} data-ka="kachel" data-i={k.i} onclick={() => void selTile(k.i)}><img src={k.img} alt="" draggable="false" /></button>{/each}
        </div>
      {:else}
        <h3>{t("karten-zeichen")} <span class="dim n">{v.charsets[v.reiter]?.split("/").pop() ?? "–"}</span></h3>
        {#if v.charsets.length > 1}<Segment small label={t("karten-zeichensatz")} items={v.charsets.map((c, i) => ({ id: String(i), label: c.split("/").pop() ?? String(i) }))} value={String(v.reiter)} onselect={(id) => void setTab(Number(id))} />{/if}
        <div class="tbox" data-ka="tafel"><img src={v.tafel.img} alt="" draggable="false" />
          <div class="tcells" role="listbox" aria-label={t("karten-tafel-label", { n: v.tafel.n })} tabindex="-1" onkeydown={cTaste}>
            {#each Array.from({ length: v.tafel.n }, (_, i) => i) as c (c)}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <div role="option" class="tz" class:on={c === v.brush.cell.code} bind:this={cz[c]} tabindex={c === cfokus ? 0 : -1} aria-selected={c === v.brush.cell.code} aria-setsize={v.tafel.n} aria-posinset={c + 1} aria-label={t("karten-zeichen-ansage", { n: c, hex: hex(c) })} data-ka="zeichen" data-c={c} onclick={() => cWaehle(c, true)}></div>
            {/each}
          </div>
        </div>
        {#if v.colorMode !== "none"}<ColorGrid palette={v.palette} value={v.brush.color} label={t("karten-zeichenfarbe")} onselect={(i) => void brushColor(i)} size={20} />{/if}
        <div class="sw2"><Switch checked={bit7} label={t("karten-bit7")} onchange={(on) => { bit7 = on; void setBit7(on); }} /></div>
      {/if}
      <h3>{t("karten-ebenen")}</h3>
      <div class="sw2"><Switch checked={v.layers.tiles} label={t("karten-ebene-kacheln")} onchange={(on) => void setLayers(on, undefined)} /></div>
      <div class="sw2"><Switch checked={v.layers.cells} label={t("karten-ebene-zellen")} onchange={(on) => void setLayers(undefined, on)} /></div>
      {#if v.propDefs.length}
        <div class="sw2"><Switch checked={v.brush.prop !== null} label={t("karten-eigenschaften-zeigen")} onchange={(on) => void showProp(on ? v.propDefs[0]!.bit : null)} /></div>
        {#if v.brush.prop !== null}<Segment small label={t("karten-eigenschaft")} items={v.propDefs.map((p) => ({ id: String(p.bit), label: p.name }))} value={String(v.brush.prop)} onselect={(id) => void showProp(Number(id))} />{/if}
      {/if}
    </section>

    <section class="col m" aria-label={t("karten-flaeche-label")}>
      <div class="bar" role="toolbar" aria-label={t("karten-werkzeuge")}>
        <Segment small label={t("karten-modus")} items={[{ id: "tile", label: t("karten-modus-kacheln") }, { id: "cell", label: t("karten-modus-zellen") }]} value={v.mode} onselect={(id) => void setMode2(id as "tile" | "cell")} />
        <span class="sep"></span>
        {#each v.tools as w (w.id)}<Tooltip text={`${w.label} (${WERKZEUGE[w.id]?.key ?? ""})`}><button type="button" class="ibtn" class:on={v.tool === w.id} aria-pressed={v.tool === w.id} aria-label={w.label} data-tool={w.id} onclick={() => void setTool(w.id)}><Icon name={WERKZEUGE[w.id]?.icon ?? "pen"} size={17} /></button></Tooltip>{/each}
        {#if v.tool === "rect"}<Tooltip text={t("karten-gefuellt")}><button type="button" class="ibtn" class:on={v.filled} aria-pressed={v.filled} aria-label={t("karten-gefuellt")} data-ka="gefuellt" onclick={() => void setFilled(!v.filled)}><Icon name="max" size={17} /></button></Tooltip>{/if}
        <span class="sep"></span>
        <Tooltip text={t("cmd-rueckgaengig")}><button type="button" class="ibtn" disabled={!v.canUndo} aria-label={t("cmd-rueckgaengig")} data-act="undo" onclick={() => void kartenUndo()}><Icon name="undo" size={17} /></button></Tooltip>
        <Tooltip text={t("cmd-wiederholen")}><button type="button" class="ibtn" disabled={!v.canRedo} aria-label={t("cmd-wiederholen")} data-act="redo" onclick={() => void kartenRedo()}><Icon name="redo" size={17} /></button></Tooltip>
        {#if v.tool === "select"}<button type="button" class="btn sm" data-act="alles-waehlen" onclick={() => void bearbeiten("a")}>{t("karten-alles-auswaehlen")}</button>{/if}
        {#if v.selection}<span class="sep"></span>
          <button type="button" class="btn sm" data-act="auswahl-stempel" disabled={v.mode !== "tile"} onclick={() => void selectionCopy()}>{t("karten-auswahl-stempel")}</button>
          <button type="button" class="btn sm" data-act="auswahl-kachel" onclick={() => void tileFromSelection(v.mode === "tile" ? (v.map?.origin[0] ?? 0) + v.selection![0] * v.tw : v.selection![0], v.mode === "tile" ? (v.map?.origin[1] ?? 0) + v.selection![1] * v.th : v.selection![1])}>{t("karten-auswahl-kachel")}</button>
          <button type="button" class="btn sm" data-act="auswahl-leeren" onclick={() => void selectionClear()}>{t("karten-auswahl-leeren")}</button>{/if}
        <span class="sep"></span>
        <Tooltip text={t("karten-zoom-aus")}><button type="button" class="ibtn" aria-label={t("karten-zoom-aus")} data-ka="zoom-aus" onclick={() => zoomSchritt(-1)}><Icon name="zoomout" size={17} /></button></Tooltip>
        <Tooltip text={t("karten-zoom-ein")}><button type="button" class="ibtn" aria-label={t("karten-zoom-ein")} data-ka="zoom-ein" onclick={() => zoomSchritt(1)}><Icon name="zoomin" size={17} /></button></Tooltip>
        <Tooltip text={t("karten-einpassen")}><button type="button" class="btn sm" data-ka="einpassen" onclick={() => (zoom = 1)}>{Math.round(zoom * 100)} %</button></Tooltip>
        <Tooltip text={t("karten-gitter")}><button type="button" class="ibtn" class:on={gitter} aria-pressed={gitter} aria-label={t("karten-gitter")} data-ka="gitter" onclick={() => (gitter = !gitter)}><Icon name="grid" size={17} /></button></Tooltip>
        <Tooltip text={t("karten-kachelgrenzen")}><button type="button" class="ibtn" class:on={grenzen} aria-pressed={grenzen} aria-label={t("karten-kachelgrenzen")} data-ka="grenzen" onclick={() => (grenzen = !grenzen)}><Icon name="max" size={17} /></button></Tooltip>
        <span class="grow"></span>
        <button type="button" class="btn sm" data-act="export" onclick={() => kt.docId !== null && void openExport(kt.docId)}>{t("export-titel")} …</button>
        <button type="button" class="btn sm" disabled={!v.dirty} data-act="save" onclick={() => void saveKarten()}>{t("cmd-speichern")}</button>
      </div>
      {#if v.map}<p class="chip mono" data-ka="chip">{v.map.name} · {t("karten-groesse-chip", { w: v.mode === "tile" ? v.cols : v.map.cellsW, h: v.mode === "tile" ? v.rows : v.map.cellsH })} · {t("karten-ursprung-chip", { x: v.map.origin[0], y: v.map.origin[1] })}</p>{/if}
      {#if v.map && geo}
        <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
        <div class="buehne" data-ka="buehne" style={`aspect-ratio:${v.map.w}/${v.map.h};width:min(${zoom * 100}%, ${1100 * zoom}px)`} bind:this={buehne} role="application" tabindex="0" aria-label={t("karten-flaeche-aria", { w: v.map.cellsW, h: v.map.cellsH })}
          onpointerdown={down} onpointermove={move} onpointerup={up} onkeydown={taste} oncopy={(e) => { e.preventDefault(); void bearbeiten("c"); }} oncut={(e) => { e.preventDefault(); void bearbeiten("x"); }} onpaste={(e) => { e.preventDefault(); void bearbeiten("v"); }} oncontextmenu={(e) => e.preventDefault()}>
          <img src={v.map.img} alt="" draggable="false" />
          <div class="ebene" style={`left:${geo.l}%;top:${geo.t}%;width:${geo.w}%;height:${geo.h}%`} aria-hidden="true">
            {#if gitter}<span class="raster1" style={`background-size:${100 / v.map.cellsW}% ${100 / v.map.cellsH}%`}></span>{/if}
            {#if grenzen && v.mode === "tile"}<span class="raster2" style={`left:${(v.map.origin[0] / v.map.cellsW) * 100}%;top:${(v.map.origin[1] / v.map.cellsH) * 100}%;right:0;bottom:0;background-size:${(v.tw / v.map.cellsW) * 100}% ${(v.th / v.map.cellsH) * 100}%`}></span>{/if}
            {#each eigenKaesten as k, i (i)}<span class="ek" style={`left:${k.l}%;top:${k.t}%;width:${k.w}%;height:${k.h}%`}></span>{/each}
            {#each live as [x, y], i (i)}<span class="lz" style={`left:${(x / v.map.cellsW) * 100}%;top:${(y / v.map.cellsH) * 100}%;width:${100 / v.map.cellsW}%;height:${100 / v.map.cellsH}%`}></span>{/each}
            {#each formEinheiten as u (u[0] + "," + u[1])}{@const k = einheitKasten(u)}<span class="lz fz" data-ka="vorschau" style={`left:${k.l}%;top:${k.t}%;width:${k.w}%;height:${k.h}%`}></span>{/each}
            {#if gummiKasten}<span class="gk" style={`left:${gummiKasten.l}%;top:${gummiKasten.t}%;width:${gummiKasten.w}%;height:${gummiKasten.h}%`}></span>{/if}
            {#if auswahlKasten}<span class="ak" data-ka="auswahl" style={`left:${auswahlKasten.l}%;top:${auswahlKasten.t}%;width:${auswahlKasten.w}%;height:${auswahlKasten.h}%`}></span>{/if}
            <span class="cu" style={`left:${(cursor[0] / v.map.cellsW) * 100}%;top:${(cursor[1] / v.map.cellsH) * 100}%;width:${100 / v.map.cellsW}%;height:${100 / v.map.cellsH}%`}></span>
          </div>
        </div>
        <p class="small dim ansage" role="status" aria-live="polite" data-ka="ansage">{ansage}</p>
      {/if}
      <div class="leiste" role="listbox" aria-label={t("karten-levels-label")} data-ka="levels">
        {#each v.screens as s (s.i)}
          <button type="button" role="option" class="lv" class:on={s.i === v.cur} aria-selected={s.i === v.cur} data-ka="level" data-i={s.i} title={`${s.name} · ${s.w} × ${s.h}`} onclick={() => void selScreen(s.i)} use:dragSource={() => ({ kind: "level", id: String(s.i), label: s.name })} use:dropTarget={() => ({ accept: (p) => p.kind === "level" && Number(p.id) !== s.i, drop: (p) => void screenOrder(Number(p.id), s.i) })}><img src={s.img} alt="" draggable="false" /><span>{s.name}</span></button>
        {/each}
        <button type="button" class="lv plus" data-ka="level-neu" onclick={() => void screenNew()}><Icon name="plus" size={20} /><span>{t("karten-neues-level")}</span></button>
      </div>
      <div class="row">
        <button type="button" class="btn sm" data-ka="level-dup" onclick={() => void screenDup()}>{t("karten-level-dupl")}</button>
        <button type="button" class="btn sm" data-ka="level-loeschen" disabled={v.screens.length < 2} onclick={() => void screenDelete()}>{t("karten-level-loeschen")}</button>
        <button type="button" class="btn sm" data-ka="level-vor" disabled={v.cur < 1} onclick={() => void screenMove(-1)}>{t("karten-level-vor")}</button>
        <button type="button" class="btn sm" data-ka="level-zurueck" disabled={v.cur >= v.screens.length - 1} onclick={() => void screenMove(1)}>{t("karten-level-zurueck")}</button>
      </div>
      <p class="foot small dim" data-ka="fuss">{#if v.dirty}<span class="warn">● {t("sprite-ungespeichert")} · </span>{/if}{v.title} · {t("karten-fuss", { n: v.cur + 1, m: v.screens.length })}{#if v.status} · {v.status}{/if}</p>
    </section>

    <section class="col r" aria-label={t("karten-karte-titel")}>
      <h3>{t("karten-karte-titel")}</h3>
      {#if v.map}
        <label class="feld"><span>{t("karten-name")}</span><input class="inp" data-ka="name" bind:value={nameFeld} onblur={umbenennen} onkeydown={(e) => e.key === "Enter" && umbenennen()} /></label>
        <label class="feld"><span>{t("karten-kachelsatz")}</span>
          <select class="inp" data-ka="satz" value={v.setRel ?? ""} onchange={(e) => void setTileSet(e.currentTarget.value)}>{#each v.sets as s (s)}<option value={s}>{s}</option>{/each}</select></label>
        <div class="feld"><span>{t("karten-groesse")} ({v.mode === "tile" ? t("karten-in-zellen") : t("karten-in-zellen")})</span>
          <div class="zahlen"><input class="inp" type="number" min="1" max="255" aria-label={t("karten-breite")} data-ka="breite" bind:value={gw} onchange={groesseSetzen} />×<input class="inp" type="number" min="1" max="255" aria-label={t("karten-hoehe")} data-ka="hoehe" bind:value={gh} onchange={groesseSetzen} /></div></div>
        <div class="feld"><span>{t("karten-ursprung")}</span>
          <div class="zahlen"><input class="inp" type="number" min="0" aria-label={t("karten-ursprung-x")} data-ka="ursprung-x" bind:value={gox} onchange={groesseSetzen} />,<input class="inp" type="number" min="0" aria-label={t("karten-ursprung-y")} data-ka="ursprung-y" bind:value={goy} onchange={groesseSetzen} /></div></div>
        <h3>{t("karten-farben")}</h3>
        {#each rollen as r (r.id)}{#if v.map.colors[r.id] !== undefined}<div class="farbzeile"><span class="small">{t(r.label)}</span><ColorGrid palette={v.palette} value={v.map.colors[r.id]!} label={t(r.label)} onselect={(i) => void screenColor(r.id, i)} size={16} /></div>{/if}{/each}
      {/if}
      {#if v.projectNorm}<h3>{t("karten-ansicht")}</h3><Segment small label={t("sprite-ansicht")} items={normen} value={v.norm} onselect={(id) => void setNorm(id)} />{/if}
      <h3>{t("karten-aufbau")}</h3>
      <p class="small" data-ka="bank">{v.title}<span class="dim"> · {t("karten-bank-levels", { n: v.screens.length })}</span></p>
      <h3>{t("karten-pruefung")}</h3>
      <ul class="pruef" data-ka="pruefung">
        {#if v.setError}<li class="bad">{v.setError}</li>{:else}<li class="ok">{t("karten-pruefung-satz-ok")}</li>{/if}
        {#if v.map && v.map.patches > 0}<li class="dim">{t("karten-pruefung-patches", { n: v.map.patches })}</li>{/if}
      </ul>
      <h3>{t("karten-ausgabe")}</h3>
      <div class="ausg"><span class="small dim">{t("karten-ausgabe-text", { n: v.outputBytes })}</span><button type="button" class="btn sm" data-act="export2" onclick={() => kt.docId !== null && void openExport(kt.docId)}>{t("export-titel")} …</button></div>
    </section>
  </div>
{/if}

{#if neuOpen && v?.ask}
  <Dialog bind:open={neuOpen} title={t("karten-neue-kachel-titel")}>
    {#snippet children()}
      {#if v!.ask!.dup !== null}<p class="small" data-ka="dup">{t("karten-neue-kachel-dup", { n: v!.ask!.dup })}</p>
      {:else}<label class="feld"><span>{t("karten-neue-kachel-name")}</span><input class="inp" data-ka="kachel-name" bind:value={askName_} onchange={() => void askName(askName_)} /></label>{/if}
      {#if v!.ask!.aligned}<div class="sw2"><Switch checked={v!.ask!.replace} label={t("karten-neue-kachel-ersetzen")} onchange={(on) => void askReplace(on)} /></div>
      {:else}<p class="small dim">{t("karten-neue-kachel-nicht-ausgerichtet")}</p>{/if}
    {/snippet}
    {#snippet footer()}
      <button type="button" class="btn" data-ka="kachel-abbrechen" onclick={() => void askNo()}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn pri" data-ka="kachel-ok" onclick={() => void askYes()}>{t("karten-neue-kachel-ok")}</button>
    {/snippet}
  </Dialog>
{/if}

{#if askOpen}
  <Dialog bind:open={askOpen} title={t("karten-frage-titel")}>
    {#snippet children()}<p>{t("sprite-frage-text", { name: v?.title ?? "" })}</p>{/snippet}
    {#snippet footer()}
      <button type="button" class="btn" onclick={() => void answerAsk("cancel")}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn" data-ask="discard" onclick={() => void answerAsk("discard")}>{t("close-verwerfen")}</button>
      <button type="button" class="btn pri" data-ask="save" onclick={() => void answerAsk("save")}>{t("close-speichern")}</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .blank { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; color: var(--dim); text-align: center; padding: 20px; }
  .blank h2 { color: var(--text); font-size: 18px; } .blank p { max-width: 420px; } .row { display: flex; gap: 6px; flex-wrap: wrap; } .files { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  .ke { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(230px, 280px) minmax(0, 1fr) 280px; gap: 10px; padding: 10px; overflow: auto; }
  @media (max-width: 1100px) { .ke { grid-template-columns: 250px minmax(0, 1fr); grid-auto-rows: min-content; } .col.r { grid-column: 1 / -1; } .col { overflow: visible; } } @media (max-width: 760px) { .ke { grid-template-columns: 1fr; } }
  .col { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 8px; min-height: 0; overflow: auto; } .col.m { min-height: 420px; }
  h3 { margin: 4px 0 0; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); display: flex; align-items: center; justify-content: space-between; gap: 6px; } .n { font-weight: normal; letter-spacing: 0; text-transform: none; }
  .raster { display: grid; grid-template-columns: repeat(auto-fill, minmax(52px, 1fr)); gap: 6px; max-height: 340px; overflow: auto; padding: 2px; }
  .kz { padding: 3px; border-radius: 8px; background: var(--raise); border: 2px solid transparent; cursor: pointer; } .kz img { display: block; width: 100%; image-rendering: pixelated; border-radius: 4px; } .kz.on { border-color: var(--accent); } .kz:hover { background: var(--hover); }
  .tbox { position: relative; width: 100%; background: #000; border-radius: 6px; overflow: hidden; } .tbox img { display: block; width: 100%; height: auto; image-rendering: pixelated; }
  .tcells { position: absolute; inset: 0; display: grid; grid-template-columns: repeat(16, 1fr); grid-auto-rows: 1fr; } .tz { cursor: pointer; } .tz.on { outline: 2px solid var(--accent); outline-offset: -2px; } .tz:hover { background: rgba(255, 255, 255, 0.18); } .tz:focus-visible { outline: 2px solid var(--accent2); outline-offset: -2px; }
  .bar { display: flex; align-items: center; gap: 4px; flex-wrap: wrap; } .grow { flex: 1; } .sep { width: 1px; height: 20px; background: var(--line2); margin: 0 6px; }
  .chip { align-self: flex-start; margin: 0; padding: 5px 12px; border-radius: 10px; border: 1px solid var(--line2); background: var(--raise); font-size: 12px; }
  .buehne { position: relative; width: min(100%, 1100px); background: #000; border-radius: 6px; overflow: hidden; touch-action: none; align-self: flex-start; outline: none; cursor: crosshair; } .buehne:focus-visible { box-shadow: 0 0 0 2px var(--accent2); } .buehne img { display: block; width: 100%; height: 100%; image-rendering: pixelated; pointer-events: none; }
  .ebene { position: absolute; pointer-events: none; } .ebene span { position: absolute; box-sizing: border-box; }
  .lz { background: color-mix(in srgb, var(--accent) 55%, transparent); outline: 1px solid var(--accent); outline-offset: -1px; } .gk { border: 2px dashed var(--accent2); background: color-mix(in srgb, var(--accent) 18%, transparent); } .ak { border: 2px solid #fff; mix-blend-mode: difference; }
  .ek { background: color-mix(in srgb, var(--warn) 45%, transparent); outline: 1px solid var(--warn); outline-offset: -1px; } .cu { outline: 1px solid rgba(255, 255, 255, 0.85); outline-offset: -1px; }
  .raster1 { inset: 0; background-image: linear-gradient(to right, rgba(255,255,255,0.18) 1px, transparent 1px), linear-gradient(to bottom, rgba(255,255,255,0.18) 1px, transparent 1px); } .raster2 { background-image: linear-gradient(to right, rgba(255,214,10,0.7) 1px, transparent 1px), linear-gradient(to bottom, rgba(255,214,10,0.7) 1px, transparent 1px); }
  .ansage { margin: 0; } .leiste { display: flex; gap: 10px; overflow-x: auto; padding: 4px 2px 12px; flex: none; }
  .lv { flex: none; width: 150px; display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 4px; border-radius: 10px; border: 2px solid transparent; background: transparent; color: var(--dim); cursor: pointer; font-size: 12px; } .lv img { width: 100%; border-radius: 6px; image-rendering: pixelated; display: block; }
  .lv.on { border-color: var(--accent); color: var(--text); } .lv:hover { background: var(--hover); } .lv.plus { justify-content: center; min-height: 100px; border: 1px dashed var(--line2); }
  .foot { margin: 0; margin-top: auto; } .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); } .bad { color: var(--err); } .ok { color: var(--ok, #4ade80); } .warn { color: var(--warn); } .inp { width: 100%; }
  .sw2 { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 4px 0; } .ausg { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .feld { display: flex; flex-direction: column; gap: 3px; font-size: 12px; color: var(--dim); } .zahlen { display: flex; align-items: center; gap: 6px; } .farbzeile { display: flex; flex-direction: column; gap: 4px; }
  .pruef { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; font-size: 12px; }
</style>
