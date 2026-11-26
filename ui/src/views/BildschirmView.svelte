<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { openExport } from "../lib/export.svelte";
  import { answerAsk, bildschirmDateien, bildschirmOp, bildschirmRedo, bildschirmUndo, bs, moveCursor, openBildschirm, pickBildschirm, rectOp, saveBildschirm, setChar, setCharset, setColor, setNorm, setRole, setTool, stroke, typeText, zerlegen, zerlegenVorschau } from "../lib/bildschirm.svelte";
  import { openKacheln } from "../lib/kachel.svelte";
  import { openAlsText, reloadTree } from "../lib/editor.svelte";
  import ColorGrid from "../lib/components/ColorGrid.svelte";
  import Dialog from "../lib/components/Dialog.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import Segment from "../lib/components/Segment.svelte";
  import Tooltip from "../lib/components/Tooltip.svelte";
  import { singleKeyOk, ui } from "../lib/state.svelte";
  import { t } from "../lib/i18n.svelte";
  // Bildschirm-Editor (E9, nach Mockup neu-8): links die Zeichentafel und die Zeichenfarbe, in der Mitte Werkzeuge und der Bildschirm (Rand, Hintergrund, Zeichen im Zeichensatz des Projekts),
  // rechts Größe, Farben des Systems, Zeichensatz, Ansicht und Ausgabe. Die Zellen sind eine Auswahlliste (Pfeiltasten bewegen den Cursor, Enter wendet das Werkzeug an).
  const v = $derived(bs.view);
  const dateien = $derived(bildschirmDateien(ui.project?.tree));
  const hex = (i: number) => "$" + i.toString(16).toUpperCase().padStart(2, "0");
  let askOpen = $state(false);
  $effect(() => { askOpen = bs.ask !== null; });
  $effect(() => { if (!askOpen && bs.ask) void answerAsk("cancel"); });
  const werkzeuge = [{ id: "char", icon: "pen", key: "D", label: "bildschirm-werkzeug-zeichen" }, { id: "rect", icon: "rect", key: "R", label: "bildschirm-werkzeug-rahmen" }, { id: "fill", icon: "fill", key: "F", label: "bildschirm-werkzeug-fuellen" },
    { id: "text", icon: "file", key: "T", label: "bildschirm-werkzeug-text" }, { id: "pick", icon: "pipette", key: "I", label: "bildschirm-werkzeug-pipette" }, { id: "erase", icon: "eraser", key: "E", label: "bildschirm-werkzeug-loeschen" }];

  // Zeichentafel (Auswahlliste mit Roving Focus)
  let cfokus = $state(0); let cz: HTMLElement[] = [];
  $effect(() => { if (v) cfokus = v.brush.code; });
  function cWaehle(c: number, fok: boolean) { if (!v || c < 0 || c >= v.tafel.n) return; cfokus = c; if (fok) cz[c]?.focus(); void setChar(c); }
  function cTaste(e: KeyboardEvent) {
    if (!v) return;
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : e.key === "ArrowDown" ? 16 : e.key === "ArrowUp" ? -16 : e.key === "Home" ? -cfokus : e.key === "End" ? v.tafel.n - 1 - cfokus : 0;
    if (!d) return; e.preventDefault(); e.stopPropagation(); cWaehle(Math.max(0, Math.min(v.tafel.n - 1, cfokus + d)), true);
  }

  // Bildschirm: Zellen als Auswahlliste; Zug über mehrere Zellen = eine Absicht, Rahmen von Ecke zu Ecke
  let zfokus = $state(0); let zz: HTMLElement[] = []; let zug: [number, number][] | null = null; let start: [number, number] | null = null; let rechts = false; let rahmenStart: [number, number] | null = $state(null);
  let live = $state<number[]>([]);   // Zellen des laufenden Zugs: sofort sichtbar, bis der Kern das Ergebnis liefert
  $effect(() => { if (v) zfokus = v.cursor[1] * v.w + v.cursor[0]; });
  const zelleAn = (e: PointerEvent): [number, number] | null => { const el = document.elementFromPoint(e.clientX, e.clientY) as HTMLElement | null; const k = el?.closest("[data-k]")?.getAttribute("data-k"); if (k == null || !v) return null; const i = Number(k); return [i % v.w, Math.floor(i / v.w)]; };
  function zDown(e: PointerEvent, x: number, y: number) {
    if (!v) return; if (e.button !== 0 && e.button !== 2) return; e.preventDefault(); rechts = e.button === 2; zfokus = y * v.w + x; zz[zfokus]?.focus();
    if (v.tool === "text") { void moveCursor(x, y); return; }
    start = [x, y]; zug = [[x, y]]; live = v.tool === "rect" ? [] : [y * v.w + x];
  }
  function zMove(e: PointerEvent) { if (!zug) return; const p = zelleAn(e); if (p && (p[0] !== zug[zug.length - 1]![0] || p[1] !== zug[zug.length - 1]![1])) { zug.push(p); if (v && v.tool !== "rect") live = [...live, p[1] * v.w + p[0]]; } }
  function zUp(e: PointerEvent) {
    if (!zug || !start) return; const p = zelleAn(e) ?? zug[zug.length - 1]!; const z = zug; const s0 = start; zug = null; start = null;
    if (v?.tool === "rect") { live = []; void rectOp(s0, p, rechts); } else void stroke(z, rechts).finally(() => { live = []; });
  }
  let puffer = ""; let timer: ReturnType<typeof setTimeout> | undefined;
  function leere() { if (timer) clearTimeout(timer); timer = undefined; if (puffer) { const t0 = puffer; puffer = ""; void typeText(t0); } }
  function zTaste(e: KeyboardEvent) {
    if (!v) return; const x = zfokus % v.w, y = Math.floor(zfokus / v.w);
    if (v.tool === "text" && e.key.length === 1 && !e.metaKey && !e.ctrlKey && !e.altKey) { e.preventDefault(); e.stopPropagation(); puffer += e.key; if (timer) clearTimeout(timer); timer = setTimeout(leere, 500); return; }
    if (e.key.startsWith("Arrow")) {
      e.preventDefault(); e.stopPropagation(); leere(); const dx = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0, dy = e.key === "ArrowDown" ? 1 : e.key === "ArrowUp" ? -1 : 0;
      const nx = Math.max(0, Math.min(v.w - 1, x + dx)), ny = Math.max(0, Math.min(v.h - 1, y + dy)); zfokus = ny * v.w + nx; zz[zfokus]?.focus(); void moveCursor(nx, ny); return;
    }
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault(); e.stopPropagation(); leere();
      if (v.tool === "rect") { if (!rahmenStart) rahmenStart = [x, y]; else { const a = rahmenStart; rahmenStart = null; void rectOp(a, [x, y], false); } } else if (v.tool !== "text") void stroke([[x, y]], false);
      return;
    }
    if (e.key === "Escape") { rahmenStart = null; leere(); return; }
    const w = werkzeuge.find((k) => k.key.toLowerCase() === e.key.toLowerCase()); if (w && singleKeyOk(e) && !e.metaKey && !e.ctrlKey && !e.altKey) { e.preventDefault(); e.stopPropagation(); void setTool(w.id); }
  }
  $effect(() => { if (v && v.tool !== "rect") rahmenStart = null; });

  const normen = $derived([{ id: "project", label: t("sprite-ansicht-projekt") }, { id: "pal", label: "PAL" }, { id: "ntsc", label: "NTSC" }]);
  const zellInfo = $derived(v ? t("bildschirm-zelle-info", { x: v.cursor[0] + 1, y: v.cursor[1] + 1, code: v.cursorCode, hex: hex(v.cursorCode) }) : "");
  const links = $derived(v ? (v.rand / v.imgW) * 100 : 0), breite = $derived(v ? ((v.w * 16) / v.imgW) * 100 : 0), oben = $derived(v ? (v.rand / v.imgH) * 100 : 0), hoehe = $derived(v ? ((v.h * 16) / v.imgH) * 100 : 0);
  // Nach dem Zerlegen: Ergebnis einmal abholen (Kern löschen lassen), Baum neu laden und den Kachelsatz öffnen
  let letzteZ: string | null = null;
  $effect(() => {
    const z = v?.zerlegt; if (!z) { letzteZ = null; return; } if (z.set === letzteZ) return; letzteZ = z.set; const satz = z.set;
    void (async () => { await bildschirmOp({ kind: "zerlegen_ende" }); await reloadTree(); bs.zerlegenOffen = false; await openKacheln(satz); })();
  });
</script>

{#if !v}
  <div class="blank">
    <Icon name="memory" size={28} />
    <h2>{t("bildschirm-leer-titel")}</h2>
    {#if bs.kachel}
      <p data-bs="kachel">{t("bildschirm-kachel-meldung", { rel: bs.kachel })}</p>
      <div class="row"><button type="button" class="btn pri" data-bs="als-text" onclick={() => { const r = bs.kachel!; bs.kachel = null; void openAlsText(r); }}>{t("bildschirm-als-text")}</button></div>
    {/if}
    {#if !ui.project}<p>{t("bildschirm-leer-kein-projekt")}</p>
    {:else}
      <p>{t("bildschirm-leer-text")}</p>
      {#if dateien.length}<ul class="files" aria-label={t("bildschirm-dateien")}>{#each dateien as f (f)}<li><button type="button" class="btn" data-bs="open" onclick={() => void openBildschirm(f)}>{f}</button></li>{/each}</ul>
      {:else}<p class="dim small">{t("bildschirm-keine-dateien")}</p>{/if}
      <div class="row"><button type="button" class="btn pri" onclick={() => void pickBildschirm()}>{t("bildschirm-oeffnen")}</button></div>
    {/if}
  </div>
{:else}
  <div class="be" aria-label={t("view-title-bildschirm")}>
    <section class="col l" aria-label={t("bildschirm-zeichen-titel")}>
      <h3>{t("bildschirm-zeichen-titel")} <span class="dim n">{v.charset ? v.charset.split("/").pop() : t("bildschirm-ohne-zeichensatz")}</span></h3>
      <div class="tbox" data-bs="tafel">
        <img src={v.tafel.img} alt="" draggable="false" />
        <div class="tcells" role="listbox" aria-label={t("bildschirm-tafel-label", { n: v.tafel.n })} tabindex="-1" onkeydown={cTaste}>
          {#each Array.from({ length: v.tafel.n }, (_, i) => i) as c (c)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <div role="option" class="tz" class:on={c === v.brush.code} bind:this={cz[c]} tabindex={c === cfokus ? 0 : -1} aria-selected={c === v.brush.code} aria-setsize={v.tafel.n} aria-posinset={c + 1}
              aria-label={t("zeichensatz-ansage", { n: c, hex: hex(c) })} data-bs="zeichen" data-c={c} onclick={() => cWaehle(c, true)}></div>
          {/each}
        </div>
      </div>
      {#if v.farbRam}<h3>{t("bildschirm-farbe-titel")}</h3><ColorGrid palette={v.palette} value={v.brush.color} label={t("bildschirm-zeichenfarbe")} onselect={(i) => void setColor(i)} size={20} />{/if}
      {#if dateien.length > 1}<h3>{t("bildschirm-dateien")}</h3>
        <ul class="files" data-bs="files">{#each dateien as f (f)}<li><button type="button" class="fbtn" class:on={f === v.rel} aria-current={f === v.rel ? "true" : undefined} onclick={() => void openBildschirm(f)} title={f}><Icon name="memory" size={14} /><span class="nm">{f.split("/").pop()}</span>{#if f === v.rel && v.dirty}<span class="udot" title={t("sprite-ungespeichert")}>●</span>{/if}</button></li>{/each}</ul>{/if}
    </section>

    <section class="col m" aria-label={t("bildschirm-flaeche-label")}>
      <div class="bar" role="toolbar" aria-label={t("bildschirm-werkzeuge")}>
        {#each werkzeuge as w (w.id)}<Tooltip text={`${t(w.label)} (${w.key})`}><button type="button" class="ibtn" class:on={v.tool === w.id} aria-pressed={v.tool === w.id} aria-label={t(w.label)} data-tool={w.id} onclick={() => void setTool(w.id)}><Icon name={w.icon} size={17} /></button></Tooltip>{/each}
        <span class="sep"></span>
        <Tooltip text={t("cmd-rueckgaengig")}><button type="button" class="ibtn" disabled={!v.canUndo} aria-label={t("cmd-rueckgaengig")} data-act="undo" onclick={() => void bildschirmUndo()}><Icon name="undo" size={17} /></button></Tooltip>
        <Tooltip text={t("cmd-wiederholen")}><button type="button" class="ibtn" disabled={!v.canRedo} aria-label={t("cmd-wiederholen")} data-act="redo" onclick={() => void bildschirmRedo()}><Icon name="redo" size={17} /></button></Tooltip>
        <span class="grow"></span>
        <button type="button" class="btn sm" data-act="zerlegen" onclick={() => { bs.zerlegenOffen = true; void zerlegenVorschau(bs.zw, bs.zh); }}>{t("bildschirm-zerlegen")}</button>
        <button type="button" class="btn sm" data-act="export" onclick={() => bs.docId !== null && void openExport(bs.docId)}>{t("export-titel")} …</button>
        <button type="button" class="btn sm" disabled={!v.dirty} data-act="save" onclick={() => void saveBildschirm()}>{t("cmd-speichern")}</button>
      </div>
      <p class="chip mono" data-bs="chip">{v.tool === "text" ? t("bildschirm-text-hinweis") : v.tool === "rect" && rahmenStart ? t("bildschirm-rahmen-hinweis") : t("bildschirm-zeichen-hinweis")}</p>
      <div class="buehne" data-bs="buehne" style={`aspect-ratio:${v.imgW}/${v.imgH}`}>
        <img src={v.img} alt="" draggable="false" />
        <div class="cells" role="listbox" aria-label={t("bildschirm-zellen-label", { w: v.w, h: v.h })} tabindex="-1" style={`left:${links}%;top:${oben}%;width:${breite}%;height:${hoehe}%;grid-template-columns:repeat(${v.w}, 1fr);grid-template-rows:repeat(${v.h}, 1fr)`}
          onkeydown={zTaste} onpointermove={zMove} onpointerup={zUp} oncontextmenu={(e) => e.preventDefault()}>
          {#each v.codes as c, k (k)}
            <div role="option" class="zl" class:on={k === v.cursor[1] * v.w + v.cursor[0]} class:zug={live.includes(k)} bind:this={zz[k]} tabindex={k === zfokus ? 0 : -1} aria-selected={k === v.cursor[1] * v.w + v.cursor[0]} aria-setsize={v.codes.length} aria-posinset={k + 1}
              aria-label={t("bildschirm-zelle-ansage", { x: (k % v.w) + 1, y: Math.floor(k / v.w) + 1, code: c, hex: hex(c) })} data-bs="zelle" data-k={k} onfocus={() => (zfokus = k)} onpointerdown={(e) => zDown(e, k % v.w, Math.floor(k / v.w))}></div>
          {/each}
        </div>
      </div>
      <p class="foot small dim" data-bs="fuss">{#if v.dirty}<span class="warn">● {t("sprite-ungespeichert")} · </span>{/if}{v.title} · <span data-bs="zellinfo">{zellInfo}</span></p>
    </section>

    <section class="col r" aria-label={t("bildschirm-karte-titel")}>
      <h3>{t("bildschirm-karte-titel")}</h3>
      <div class="sw2"><span><b>{t("bildschirm-groesse")}</b><br /><span class="dim small">{v.farbRam ? t("bildschirm-groesse-farbram") : t("bildschirm-groesse-einfarbig")}</span></span><span class="mono" data-bs="groesse">{v.w} × {v.h}</span></div>
      <h3>{t("bildschirm-farben-system")}</h3>
      <p class="small dim">{t("bildschirm-rahmen")}</p><ColorGrid palette={v.palette} value={v.roles.border} label={t("bildschirm-rahmen")} onselect={(i) => void setRole("border", i)} size={v.palette.length > 64 ? 11 : 16} />
      <p class="small dim">{t("bildschirm-hintergrund")}</p><ColorGrid palette={v.palette} value={v.roles.bg} label={t("bildschirm-hintergrund")} onselect={(i) => void setRole("bg", i)} size={v.palette.length > 64 ? 11 : 16} />
      <p class="small dim">{t("bildschirm-zeichen-rolle")}</p><ColorGrid palette={v.palette} value={v.roles.fg} label={t("bildschirm-zeichen-rolle")} onselect={(i) => void setRole("fg", i)} size={v.palette.length > 64 ? 11 : 16} />
      <h3>{t("bildschirm-zeichensatz")}</h3>
      <select class="inp" data-bs="zeichensatz" aria-label={t("bildschirm-zeichensatz")} value={v.charset ?? ""} onchange={(e) => void setCharset(e.currentTarget.value || null)}>
        <option value="">{t("bildschirm-ohne-zeichensatz")}</option>{#each v.charsets as c (c)}<option value={c}>{c}</option>{/each}
      </select>
      {#if v.projectNorm}<Segment small label={t("sprite-ansicht")} items={normen} value={v.norm} onselect={(id) => void setNorm(id)} />{/if}
      <h3>{t("bildschirm-ausgabe")}</h3>
      <div class="ausg"><span class="small dim">{t("bildschirm-ausgabe-text", { n: v.w * v.h })}</span><button type="button" class="btn sm" data-act="export2" onclick={() => bs.docId !== null && void openExport(bs.docId)}>{t("export-titel")} …</button></div>
    </section>
  </div>
{/if}

{#if bs.zerlegenOffen && v}
  <Dialog bind:open={bs.zerlegenOffen} title={t("bildschirm-zerlegen-titel")}>
    {#snippet children()}
      <p class="small dim">{t("bildschirm-zerlegen-text")}</p>
      <Segment small label={t("bildschirm-kachelgroesse")} items={[{ id: "2", label: "2 × 2" }, { id: "3", label: "3 × 3" }, { id: "4", label: "4 × 4" }]} value={String(bs.zw)} onselect={(id) => { bs.zw = Number(id); bs.zh = Number(id); void zerlegenVorschau(bs.zw, bs.zh); }} />
      <p class="small" class:bad={v.zerlegen && !v.zerlegen.ok} data-bs="zerlegen-vorschau" role="status">{v.zerlegen?.text ?? ""}</p>
    {/snippet}
    {#snippet footer()}
      <button type="button" class="btn" data-bs="zerlegen-abbrechen" onclick={() => (bs.zerlegenOffen = false)}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn pri" data-bs="zerlegen-los" disabled={!v.zerlegen?.ok} onclick={() => void zerlegen(bs.zw, bs.zh)}>{t("bildschirm-zerlegen-los")}</button>
    {/snippet}
  </Dialog>
{/if}

{#if askOpen}
  <Dialog bind:open={askOpen} title={t("bildschirm-frage-titel")}>
    {#snippet children()}<p>{t("sprite-frage-text", { name: v?.title ?? "" })}</p>{/snippet}
    {#snippet footer()}
      <button type="button" class="btn" onclick={() => void answerAsk("cancel")}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn" data-ask="discard" onclick={() => void answerAsk("discard")}>{t("close-verwerfen")}</button>
      <button type="button" class="btn pri" data-ask="save" onclick={() => void answerAsk("save")}>{t("close-speichern")}</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .zl.zug { background: color-mix(in srgb, var(--accent) 55%, transparent); outline: 1px solid var(--accent); outline-offset: -1px; }
  .blank { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; color: var(--dim); text-align: center; padding: 20px; }
  .blank h2 { color: var(--text); font-size: 18px; } .blank p { max-width: 420px; } .row { display: flex; gap: 6px; flex-wrap: wrap; } .files { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  .be { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(250px, 300px) minmax(0, 1fr) 280px; gap: 10px; padding: 10px; overflow: auto; }
  @media (max-width: 1100px) { .be { grid-template-columns: 260px minmax(0, 1fr); grid-auto-rows: min-content; } .col.r { grid-column: 1 / -1; } .col { overflow: visible; } } @media (max-width: 760px) { .be { grid-template-columns: 1fr; } }
  .col { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 8px; min-height: 0; overflow: auto; } .col.m { min-height: 420px; }
  h3 { margin: 4px 0 0; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); display: flex; align-items: center; justify-content: space-between; gap: 6px; } .n { font-weight: normal; letter-spacing: 0; text-transform: none; }
  .tbox { position: relative; width: 100%; background: #000; border-radius: 6px; overflow: hidden; } .tbox img { display: block; width: 100%; height: auto; image-rendering: pixelated; }
  .tcells { position: absolute; inset: 0; display: grid; grid-template-columns: repeat(16, 1fr); grid-auto-rows: 1fr; } .tz { cursor: pointer; } .tz.on { outline: 2px solid var(--accent); outline-offset: -2px; } .tz:hover { background: rgba(255, 255, 255, 0.18); } .tz:focus-visible { outline: 2px solid var(--accent2); outline-offset: -2px; }
  .fbtn { display: flex; align-items: center; gap: 7px; width: 100%; padding: 6px 8px; border-radius: 8px; color: var(--text); text-align: left; } .fbtn:hover { background: var(--hover); } .fbtn.on { background: var(--accentbg); color: var(--accent2); }
  .nm { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } .udot, .warn { color: var(--warn); }
  .bar { display: flex; align-items: center; gap: 4px; flex-wrap: wrap; } .grow { flex: 1; } .sep { width: 1px; height: 20px; background: var(--line2); margin: 0 6px; }
  .chip { align-self: flex-start; margin: 0; padding: 5px 12px; border-radius: 10px; border: 1px solid var(--line2); background: var(--raise); font-size: 12px; }
  .buehne { position: relative; width: min(100%, 960px); background: #000; border-radius: 6px; overflow: hidden; touch-action: none; align-self: flex-start; } .buehne img { display: block; width: 100%; height: 100%; image-rendering: pixelated; }
  .cells { position: absolute; display: grid; } .zl { cursor: crosshair; } .zl.on { outline: 1px solid var(--accent); outline-offset: -1px; background: rgba(255, 255, 255, 0.12); } .zl:focus-visible { outline: 2px solid var(--accent2); outline-offset: -2px; }
  .foot { margin: 0; margin-top: auto; } .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); } .bad { color: var(--err); } .inp { width: 100%; }
  .sw2 { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 4px 0; } .ausg { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
</style>
