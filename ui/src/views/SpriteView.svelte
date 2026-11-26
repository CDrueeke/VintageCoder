<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { openExport } from "../lib/export.svelte";
  import { frameMove, markAll, markFrame, markNone, setPmgPrior, setPmgSize, animAddStep, animDelay, animDelayAll, animMoveTo, animDelete, animDelStep, animMove, animNew, animRename, animSelect, animMode, loadPng, answerAsk, clearImage, flip, frameDelete, frameDup, frameNew, openSprite, selectFrame, setFlag, setFrameMode as setSpriteMode, shift, pickSprite, saveSprite, setColor, setNorm, setTool, setValue, spriteDateien, spriteRedo, spriteUndo, sp, zoomFit, zoomStep } from "../lib/sprite.svelte";
  import AnimTimeline from "../lib/components/AnimTimeline.svelte";
  import { dragSource, dropTarget } from "../lib/dnd.svelte";
  import AnimPreviewDialog from "../lib/components/AnimPreviewDialog.svelte";
  import AnimSteps from "../lib/components/AnimSteps.svelte";
  import AnimFromSelectionDialog from "../lib/components/AnimFromSelectionDialog.svelte";
  import { popupMenu } from "../lib/cm-lsp";
  import NewSpriteDialog from "../lib/components/NewSpriteDialog.svelte";
  import FrameThumb from "../lib/components/FrameThumb.svelte";
  import SpriteScene from "../lib/components/SpriteScene.svelte";
  import PixelCanvas from "../lib/components/PixelCanvas.svelte";
  import Dialog from "../lib/components/Dialog.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import Segment from "../lib/components/Segment.svelte";
  import Tooltip from "../lib/components/Tooltip.svelte";
  import { ui } from "../lib/state.svelte";
  import { t } from "../lib/i18n.svelte";
  // Sprite-Editor (E3.1): links die Sprite-Dateien des Projekts, in der Mitte Werkzeuge und Zeichenfläche, rechts Farben (nach Mockup neu-4)
  const v = $derived(sp.view);
  const dateien = $derived(spriteDateien(ui.project?.tree));
  let fit = $state(8);
  let neu = $state(false);
  const zoomPct = $derived((sp.zoom ?? fit) * 100);
  let askOpen = $state(false);
  $effect(() => { askOpen = sp.ask !== null; });
  $effect(() => { if (!askOpen && sp.ask) void answerAsk("cancel"); });
  const tools = [{ id: "pen", icon: "pen", key: "D", label: "pixel-werkzeug-stift" }, { id: "erase", icon: "eraser", key: "E", label: "pixel-werkzeug-radierer" }, { id: "pick", icon: "pipette", key: "I", label: "pixel-werkzeug-pipette" },
    { id: "fill", icon: "fill", key: "F", label: "pixel-werkzeug-fuellen" }, { id: "line", icon: "line", key: "L", label: "pixel-werkzeug-linie" }, { id: "rect", icon: "rect", key: "R", label: "pixel-werkzeug-rechteck" }, { id: "select", icon: "select", key: "S", label: "pixel-werkzeug-auswahl" }];
  const pfeile = [{ d: "l", icon: "arrl", label: "sprite-links" }, { d: "r", icon: "arrr", label: "sprite-rechts" }, { d: "u", icon: "arru", label: "sprite-oben" }, { d: "d", icon: "arrd", label: "sprite-unten" }] as const;
  const cur = $derived(v?.values[v.pv]);
  const normen = $derived([{ id: "project", label: t("sprite-ansicht-projekt") }, { id: "pal", label: "PAL" }, { id: "ntsc", label: "NTSC" }]);
  function framesKey(e: KeyboardEvent) { if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "a") { e.preventDefault(); markAll(); } else if (e.key === "Escape" && sp.marked.length > 1) { markNone(); } }
  function frameMenu(e: MouseEvent, i: number) {
    e.preventDefault(); if (!sp.marked.includes(i)) void markFrame(i, {});
    popupMenu(null, e.clientX, e.clientY, t("sprite-bilder"), [
      { label: t("sprite-anim-aus-auswahl-punkte"), run: () => { if (sp.marked.length < 2) markAll(); sp.animDialog = true; } },
      { label: t("sprite-alle-markieren"), run: markAll },
      { label: t("sprite-markierung-aufheben"), run: markNone },
    ]);
  }
</script>

{#if !v || v.kind === "charset"}
  <div class="blank">
    <Icon name="sprite" size={28} />
    <h2>{t("sprite-leer-titel")}</h2>
    {#if !ui.project}<p>{t("sprite-leer-kein-projekt")}</p>
    {:else}
      <p>{t("sprite-leer-text")}</p>
      {#if dateien.length}<ul class="files" aria-label={t("sprite-dateien")}>{#each dateien as f (f)}<li><button type="button" class="btn" onclick={() => void openSprite(f)}>{f}</button></li>{/each}</ul>
      {:else}<p class="dim small">{t("sprite-keine-dateien")}</p>{/if}
      <div class="row"><button type="button" class="btn pri" onclick={() => void pickSprite()}>{t("sprite-oeffnen")}</button><button type="button" class="btn" data-act="new" onclick={() => (neu = true)}>{t("sprite-neu")}</button></div>
    {/if}
  </div>
{:else}
  <div class="sprite" aria-label={t("view-title-sprites")}>
    <section class="col l" aria-label={t("sprite-dateien")}>
      <h3>{t("sprite-dateien")}</h3>
      <ul class="files" data-sp="files">{#each dateien as f (f)}
        <li><button type="button" class="fbtn" class:on={f === v.rel} aria-current={f === v.rel ? "true" : undefined} onclick={() => void openSprite(f)} title={f}>
          <Icon name="sprite" size={14} /><span class="nm">{f.split("/").pop()}</span>{#if f === v.rel && v.dirty}<span class="udot" title={t("sprite-ungespeichert")}>●</span>{/if}</button></li>
      {/each}</ul>
      <button type="button" class="btn" onclick={() => void pickSprite()}>{t("sprite-oeffnen")}</button>
      <button type="button" class="btn" data-act="new" onclick={() => (neu = true)}>{t("sprite-neu")}</button>
      <Segment items={[{ id: "frames", label: t("sprite-reiter-bilder") }, ...(v.anims ? [{ id: "anims", label: t("sprite-reiter-anim") }] : [])]} value={sp.tab} onselect={(id) => (sp.tab = id as "frames" | "anims")} label={t("sprite-bilder")} small />
      {#if sp.tab === "frames" || !v.anims}
      <p class="small dim n">{t("sprite-bilder-n", { n: v.frames })}</p>
      <div class="frames" role="listbox" aria-multiselectable="true" tabindex="-1" aria-label={t("sprite-bilder")} data-sp="frames" onkeydown={framesKey} oncontextmenu={(e) => e.preventDefault()}>
        {#each v.thumbs as th, i (i)}
          {@const mk = sp.marked.includes(i)}
          <button type="button" class="fr" class:on={i === v.fi} class:mk role="option" aria-selected={mk || i === v.fi} data-frame={i} data-marked={mk ? "1" : undefined} use:dragSource={() => ({ kind: "frame", id: String(i), label: String(i + 1) })} use:dropTarget={() => ({ accept: (p) => p.kind === "frame" && Number(p.id) !== i && !v.fixedFrames, drop: (p) => void frameMove(Number(p.id), i) })} title={v.frameNames[i] ?? String(i + 1)} onclick={(e) => void markFrame(i, e)} oncontextmenu={(e) => frameMenu(e, i)}>
            <FrameThumb pixels={th.pixels} w={th.w} h={th.pixels.length / th.w} aspect={th.aspect} colors={th.colors} boxW={64} boxH={50} /><span class="fn">{v.frameNames[i] ?? i + 1}</span>{#if mk && sp.marked.length > 1}<span class="mkdot" aria-hidden="true">✓</span>{/if}</button>
        {/each}
      </div>
      {#if v.anims && sp.marked.length >= 2}
        <div class="marks" data-sp="marks" role="status"><span class="small">{t("sprite-bilder-markiert", { n: sp.marked.length })}</span>
          <div class="row"><button type="button" class="btn sm pri" data-act="anim-from" onclick={() => (sp.animDialog = true)}>{t("sprite-anim-aus-auswahl")}</button>
            <button type="button" class="btn sm" data-act="mark-none" onclick={markNone}>{t("sprite-markierung-aufheben")}</button></div></div>
      {:else if v.anims && v.frames > 1}<p class="small dim" data-sp="mark-hint">{t("sprite-markier-hinweis")}</p><div class="row"><button type="button" class="btn sm" data-act="play-all" onclick={() => { markAll(); sp.animDialog = true; }}>{t("sprite-vorschau-abspielen")}</button></div>{/if}
      {#if !v.fixedFrames}
        <div class="row">
          <button type="button" class="btn sm" data-act="frame-new" onclick={() => void frameNew()}>{t("sprite-bild-neu")}</button>
          <button type="button" class="btn sm" data-act="frame-left" disabled={v.fi < 1} onclick={() => void frameMove(v.fi, v.fi - 1)}>{t("sprite-bild-links")}</button>
          <button type="button" class="btn sm" data-act="frame-right" disabled={v.fi >= v.frames - 1} onclick={() => void frameMove(v.fi, v.fi + 1)}>{t("sprite-bild-rechts")}</button>
          <button type="button" class="btn sm" data-act="frame-dup" onclick={() => void frameDup()}>{t("sprite-bild-dupl")}</button>
          <button type="button" class="btn sm" data-act="frame-del" disabled={v.frames < 2} onclick={() => void frameDelete()}>{t("sprite-bild-loeschen")}</button>
        </div>
      {/if}
      {:else}
        {@const an = v.anims[Math.min(v.selAnim, Math.max(0, v.anims.length - 1))]}
        {#if v.animError}<p class="small bad">{t("sprite-anim-fehler", { e: v.animError })}</p>{/if}
        <div class="row" data-sp="anims">{#each v.anims as a, i (i)}<button type="button" class="btn sm tgl" class:on={i === v.selAnim} aria-pressed={i === v.selAnim} data-anim={i} onclick={() => void animSelect(i)}>{a.name}</button>{/each}</div>
        <div class="row"><button type="button" class="btn sm" data-act="anim-new" onclick={() => void animNew()}>{t("sprite-anim-neu")}</button>
          <button type="button" class="btn sm" data-act="anim-del" disabled={!an} onclick={() => void animDelete()}>{t("sprite-anim-loeschen")}</button></div>
        {#if !an}<p class="small dim">{t("sprite-anim-keine")}</p>
        {:else}
          <label class="fld small"><span class="dim">{t("sprite-anim-name")}</span><input class="inp" data-sp="anim-rename" value={an.name} onchange={(e) => void animRename(e.currentTarget.value)} /></label>
          <Segment items={[{ id: "loop", label: t("sprite-ablauf-loop") }, { id: "pingpong", label: t("sprite-ablauf-pingpong") }, { id: "once", label: t("sprite-ablauf-once") }]} value={an.mode} onselect={(id) => void animMode(id)} label={t("sprite-ablauf")} small />
          <AnimSteps view={v} {an} />
        {/if}
      {/if}
    </section>

    <section class="col m" aria-label={t("sprite-flaeche-label")}>
      <div class="bar" role="toolbar" aria-label={t("sprite-werkzeuge")}>
        {#each tools as tl (tl.id)}
          <Tooltip text={`${t(tl.label)} (${tl.key})`}><button type="button" class="ibtn" class:on={v.tool === tl.id} aria-pressed={v.tool === tl.id} aria-label={t(tl.label)} data-tool={tl.id} onclick={() => void setTool(tl.id)}><Icon name={tl.icon} size={17} /></button></Tooltip>
        {/each}
        <span class="sep"></span>
        <Tooltip text={`${t("sprite-spiegeln-h")} (X)`}><button type="button" class="ibtn" aria-label={t("sprite-spiegeln-h")} data-act="fliph" onclick={() => void flip("h")}><Icon name="fliph" size={17} /></button></Tooltip>
        <Tooltip text={`${t("sprite-spiegeln-v")} (Y)`}><button type="button" class="ibtn" aria-label={t("sprite-spiegeln-v")} data-act="flipv" onclick={() => void flip("v")}><Icon name="flipv" size={17} /></button></Tooltip>
        <span class="sep"></span>
        <Tooltip text={t("cmd-rueckgaengig")}><button type="button" class="ibtn" disabled={!v.canUndo} aria-label={t("cmd-rueckgaengig")} data-act="undo" onclick={() => void spriteUndo()}><Icon name="undo" size={17} /></button></Tooltip>
        <Tooltip text={t("cmd-wiederholen")}><button type="button" class="ibtn" disabled={!v.canRedo} aria-label={t("cmd-wiederholen")} data-act="redo" onclick={() => void spriteRedo()}><Icon name="redo" size={17} /></button></Tooltip>
        <span class="grow"></span>
        <Tooltip text={`${t("sprite-onion")} (O)`}><button type="button" class="btn sm tgl" class:on={sp.onion} aria-pressed={sp.onion} data-act="onion" disabled={v.fi === 0} onclick={() => (sp.onion = !sp.onion)}>{t("sprite-onion")}</button></Tooltip>
        <Tooltip text={`${t("sprite-raster")} (G)`}><button type="button" class="ibtn" class:on={sp.grid} aria-pressed={sp.grid} aria-label={t("sprite-raster")} data-act="grid" onclick={() => (sp.grid = !sp.grid)}><Icon name="grid" size={17} /></button></Tooltip>
        <Tooltip text={`${t("sprite-zoom-minus")} (−)`}><button type="button" class="ibtn" aria-label={t("sprite-zoom-minus")} data-act="zoomout" onclick={() => zoomStep(-1, fit)}><Icon name="zoomout" size={17} /></button></Tooltip>
        <span class="zoom mono" data-sp="zoom" aria-label={t("sprite-kurz-zoom")}>{t("sprite-zoom-text", { z: String(zoomPct) })}</span>
        <Tooltip text={`${t("sprite-zoom-plus")} (+)`}><button type="button" class="ibtn" aria-label={t("sprite-zoom-plus")} data-act="zoomin" onclick={() => zoomStep(1, fit)}><Icon name="zoomin" size={17} /></button></Tooltip>
        <Tooltip text={`${t("sprite-zoom-einpassen")} (0)`}><button type="button" class="ibtn" class:on={sp.zoom === null} aria-label={t("sprite-zoom-einpassen")} data-act="fit" onclick={zoomFit}><Icon name="max" size={17} /></button></Tooltip>
        <span class="sep"></span>
        <button type="button" class="btn sm" data-act="png" title={t("sprite-png-tipp")} onclick={() => void loadPng()}>{t("sprite-png")}</button>
        <button type="button" class="btn sm" data-act="export" onclick={() => sp.docId !== null && void openExport(sp.docId)}>{t("export-titel")} …</button>
        <button type="button" class="btn sm" disabled={!v.dirty} data-act="save" onclick={() => void saveSprite()}>{t("cmd-speichern")}</button>
      </div>
      <PixelCanvas view={v} bind:fit />
      {#if v.anims}<AnimTimeline view={v} />{/if}
      <p class="foot small dim" data-sp="fuss">{#if v.dirty}<span class="warn">● {t("sprite-ungespeichert")} · </span>{/if}{t("sprite-fuss", { name: v.title, w: v.width, h: v.height, i: v.fi + 1, n: v.frames })}{#if v.sel} · {t("sprite-auswahl-ansage", { w: v.sel[2] - v.sel[0] + 1, h: v.sel[3] - v.sel[1] + 1 })}{/if}</p>
    </section>

    <section class="col r" aria-label={t("sprite-karte-farben")}>
      <h3>{t("sprite-karte-farben")}</h3>
      <div class="vals" role="radiogroup" aria-label={t("sprite-kurz-wert")}>
        {#each v.values as w (w.v)}
          <button type="button" class="val" class:on={w.v === v.pv} role="radio" aria-checked={w.v === v.pv} data-val={w.v} title={w.name} onclick={() => void setValue(w.v)}>
            <span class="wsw" class:tr={!w.color} style={w.color ? `background:${w.color}` : ""}></span><span class="k mono">{w.v + 1}</span><span class="vn">{(w.name || t("sprite-wert", { n: w.v + 1 })).split(" (")[0]}</span></button>
        {/each}
      </div>
      <h3>{t("sprite-palette", { name: v.normActive.toUpperCase() })}</h3>
      {#if v.normChoice}<Segment items={normen} value={v.norm} onselect={(id) => void setNorm(id)} label={t("sprite-ansicht")} small />{/if}
      {#if cur?.role}
        {@const rolle = v.slots.find((s) => s.key === cur.role)}
        <p class="small dim">{t("sprite-farbe-fuer", { name: rolle?.label ?? cur.name })}</p>
        <div class="farbraster" role="radiogroup" aria-label={t("sprite-farbe-fuer", { name: rolle?.label ?? cur.name })} data-sp="palette">
          {#each v.palette as p (p.i)}
            <button type="button" class="pc" class:on={rolle?.index === p.i} role="radio" aria-checked={rolle?.index === p.i} aria-label={`${p.i}`} title={`${p.i}`} data-idx={p.i} style={`background:${p.color}`} onclick={() => void setColor(cur.role!, p.i)}></button>
          {/each}
        </div>
      {:else}<p class="small dim">{t("sprite-durchsichtig")}</p>{/if}
      {#if v.pmg}
        <h3>{t("sprite-karte-pmg")}</h3>
        {#each v.pmg.sizes as sz, i (i)}
          <label class="sw2"><span><b>{t("sprite-pmg-breite", { n: i })}</b><br /><span class="dim small">SIZEP{i} ${(0xd008 + i).toString(16).toUpperCase()}</span></span>
            <select class="inp sel" data-pmg-size={i} value={sz} onchange={(e) => void setPmgSize(i, Number(e.currentTarget.value))}>
              {#each v.pmg.sizeChoices as c (c)}<option value={c} selected={c === sz}>{t(`sprite-pmg-breite-${c}`)}</option>{/each}</select></label>
        {/each}
        <label class="sw2"><span><b>{t("sprite-pmg-prior")}</b><br /><span class="dim small">PRIOR $D01B</span></span>
          <select class="inp sel" data-pmg-prior value={v.pmg.prior} onchange={(e) => void setPmgPrior(Number(e.currentTarget.value))}>
            <option value={0} selected={v.pmg.prior === 0}>{t("sprite-pmg-prior-0")}</option>{#each v.pmg.priorChoices as c (c)}<option value={c} selected={c === v.pmg.prior}>{t(`sprite-pmg-prior-${c}`)}</option>{/each}</select></label>
        <p class="small dim" data-pmg-sichtbar>{t("sprite-pmg-sichtbar")}</p>
      {/if}
      {#if v.modes || v.flags}
        <h3>{t("sprite-karte-sprite")}</h3>
        {#if v.modes}
          <div class="sw2"><span><b>{t("sprite-multicolor")}</b><br /><span class="dim small">{v.modes[1]?.hint}</span></span>
            <button type="button" role="switch" class="swt" aria-checked={v.multicolor} aria-label={t("sprite-multicolor")} data-flag="mc" onclick={() => void setSpriteMode(!v.multicolor)}></button></div>
        {/if}
        {#if v.flags}
          {#each [["x2", "sprite-doppelt-breit", v.width * v.aspect * 2], ["y2", "sprite-doppelt-hoch", v.height * 2], ["bg", "sprite-hinter-hintergrund", 0]] as [key, lab, pts] (key)}
            <div class="sw2"><span><b>{t(String(lab))}</b>{#if key !== "bg"}<br /><span class="dim small">{t("sprite-im-spiel", { n: Number(pts) })}</span>{/if}</span>
              <button type="button" role="switch" class="swt" aria-checked={v.flags[key as "x2" | "y2" | "bg"]} aria-label={t(String(lab))} data-flag={key} onclick={() => void setFlag(key as "x2" | "y2" | "bg", !v.flags![key as "x2" | "y2" | "bg"])}></button></div>
          {/each}
        {/if}
      {/if}
      <h3>{t("sprite-karte-vorschau")}</h3>
      <SpriteScene view={v} />
      <p class="small dim" data-sp="speicher">{t("sprite-speicher", { bytes: v.blockBytes, n: v.frames, total: v.blockBytes * v.frames })}</p>
      <h3>{t("sprite-verschieben")}</h3>
      <div class="pfeile" role="group" aria-label={t("sprite-verschieben")}>
        {#each pfeile as p (p.d)}<Tooltip text={t(p.label)}><button type="button" class="btn" aria-label={t(p.label)} data-shift={p.d} onclick={() => void shift(p.d)}><Icon name={p.icon} size={16} /></button></Tooltip>{/each}
      </div>
      <button type="button" class="btn" data-act="clear" onclick={() => void clearImage()}>{t("sprite-leeren")}</button>
    </section>
  </div>
{/if}

<NewSpriteDialog bind:open={neu} />
<AnimFromSelectionDialog />
<AnimPreviewDialog />
{#if sp.ask}
  <Dialog bind:open={askOpen} title={t("sprite-frage-titel")}>
    {#snippet children()}<p>{t("sprite-frage-text", { name: v?.title ?? "" })}</p>{/snippet}
    {#snippet footer()}
      <button type="button" class="btn" onclick={() => void answerAsk("cancel")}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn" onclick={() => void answerAsk("discard")}>{t("close-verwerfen")}</button>
      <button type="button" class="btn pri" onclick={() => void answerAsk("save")}>{t("close-speichern")}</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .fr.mk { outline: 2px solid var(--accent); outline-offset: -2px; } .fr { position: relative; } .mkdot { position: absolute; top: 2px; right: 4px; font-size: 11px; color: var(--accent2); }
  .marks { display: flex; flex-direction: column; gap: 6px; padding: 6px 8px; border-radius: 8px; background: var(--accentbg); }
  .blank { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; color: var(--dim); text-align: center; padding: 20px; }
  .blank h2 { color: var(--text); font-size: 18px; } .blank p { max-width: 420px; } .row { display: flex; gap: 6px; flex-wrap: wrap; }
  .sprite { flex: 1; min-height: 0; display: grid; grid-template-columns: 220px minmax(0, 1fr) 260px; gap: 10px; padding: 10px; overflow: auto; }
  @media (max-width: 1000px) { .sprite { grid-template-columns: 200px minmax(0, 1fr); } .col.r { grid-column: 1 / -1; } }
  @media (max-width: 640px) { .sprite { grid-template-columns: 1fr; } }
  .col { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 8px; min-height: 0; overflow: auto; }
  .col.m { min-height: 360px; }
  h3 { margin: 4px 0 0; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); }
  .files { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; overflow: auto; }
  .fbtn { display: flex; align-items: center; gap: 7px; width: 100%; padding: 6px 8px; border-radius: 8px; color: var(--text); text-align: left; } .fbtn:hover { background: var(--hover); } .fbtn.on { background: var(--accentbg); color: var(--accent2); }
  .nm { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } .udot { color: var(--warn); }
  .bar { display: flex; align-items: center; gap: 4px; flex-wrap: wrap; } .grow { flex: 1; } .sep { width: 1px; height: 20px; background: var(--line2); margin: 0 6px; }
  .zoom { min-width: 56px; text-align: center; font-size: 12px; color: var(--dim); }
  .foot { margin: 0; } .warn { color: var(--warn); } .dim { color: var(--dim); } .small { font-size: 12px; }
  .vals { display: grid; grid-template-columns: 1fr; gap: 6px; }
  .val { display: flex; align-items: center; gap: 6px; padding: 6px 7px; border-radius: 8px; background: var(--raise); border: 1px solid var(--line2); color: var(--text); font-size: 12px; text-align: left; min-width: 0; }
  .val.on { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent); }
  .wsw { width: 16px; height: 16px; border-radius: 4px; border: 1px solid var(--line2); flex: none; } .wsw.tr { background: repeating-conic-gradient(var(--check1) 0 25%, var(--check2) 0 50%) 0 0 / 8px 8px; }
  .k { color: var(--dim); } .vn { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .farbraster { display: grid; grid-template-columns: repeat(8, 1fr); gap: 3px; }
  .frames { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; overflow: auto; min-height: 60px; } .fr { display: flex; flex-direction: column; align-items: center; gap: 2px; padding: 3px; border-radius: 8px; background: var(--raise); border: 1px solid var(--line2); color: var(--dim); font-size: 11px; } .fr.on { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent); color: var(--text); } .fr :global(canvas) { background: repeating-conic-gradient(var(--check1) 0 25%, var(--check2) 0 50%) 0 0 / 8px 8px; border-radius: 4px; max-width: 100%; }
  .n { font-weight: normal; letter-spacing: 0; text-transform: none; } .tgl.on { background: var(--accentbg); color: var(--accent); border-color: var(--accent); }
  .sw2 { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 4px 0; border-top: 1px solid var(--line); } .swt { width: 34px; height: 20px; border-radius: 10px; background: var(--hover); position: relative; flex: none; } .swt::after { content: ""; position: absolute; top: 2px; left: 2px; width: 16px; height: 16px; border-radius: 50%; background: #fff; box-shadow: 0 1px 2px rgba(0,0,0,.4); transition: left .12s; } .swt[aria-checked="true"] { background: var(--accent); } .swt[aria-checked="true"]::after { left: 16px; }
  .sel { width: 118px; flex: none; }
  .pfeile { display: grid; grid-template-columns: repeat(4, 1fr); gap: 6px; } .pfeile :global(.btn) { justify-content: center; }
  .pc { aspect-ratio: 1; border-radius: 5px; border: 1px solid var(--line2); min-width: 14px; } .pc.on { outline: 2px solid var(--accent); outline-offset: 1px; }
</style>
