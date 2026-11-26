<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { openExport } from "../lib/export.svelte";
  import { answerAsk, clearImage, flip, loadPng, loadTemplate, openSprite, pickSprite, resetChar, saveSprite, selectFrame, setColor, setNorm, setTool, setValue, shift, sp, spriteRedo, spriteUndo, zeichenBereiche, zeichensatzDateien, zoomFit, zoomStep } from "../lib/sprite.svelte";
  import NewSpriteDialog from "../lib/components/NewSpriteDialog.svelte";
  import CharGrid from "../lib/components/CharGrid.svelte";
  import FrameThumb from "../lib/components/FrameThumb.svelte";
  import PixelCanvas from "../lib/components/PixelCanvas.svelte";
  import Dialog from "../lib/components/Dialog.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import Segment from "../lib/components/Segment.svelte";
  import Tooltip from "../lib/components/Tooltip.svelte";
  import { ui } from "../lib/state.svelte";
  import { t } from "../lib/i18n.svelte";
  // Zeichensatz-Editor (E4): links die Übersicht aller Zeichen, in der Mitte Werkzeuge und der 8×8-Editor (dieselbe Zeichenfläche wie im Sprite-Editor), rechts Vorschau im Bild, Zeichen, Verschieben (nach Mockup neu-6)
  const v = $derived(sp.view?.kind === "charset" ? sp.view : null);
  const dateien = $derived(zeichensatzDateien(ui.project?.tree));
  let fit = $state(8);
  const zoomPct = $derived((sp.zoom ?? fit) * 100);
  let askOpen = $state(false);
  $effect(() => { askOpen = sp.ask !== null; });
  $effect(() => { if (!askOpen && sp.ask) void answerAsk("cancel"); });
  const tools = [{ id: "pen", icon: "pen", key: "D", label: "pixel-werkzeug-stift" }, { id: "erase", icon: "eraser", key: "E", label: "pixel-werkzeug-radierer" }, { id: "pick", icon: "pipette", key: "I", label: "pixel-werkzeug-pipette" },
    { id: "fill", icon: "fill", key: "F", label: "pixel-werkzeug-fuellen" }, { id: "line", icon: "line", key: "L", label: "pixel-werkzeug-linie" }, { id: "rect", icon: "rect", key: "R", label: "pixel-werkzeug-rechteck" }, { id: "select", icon: "select", key: "S", label: "pixel-werkzeug-auswahl" }];
  const pfeile = [{ d: "l", icon: "arrl", label: "sprite-links" }, { d: "r", icon: "arrr", label: "sprite-rechts" }, { d: "u", icon: "arru", label: "sprite-oben" }, { d: "d", icon: "arrd", label: "sprite-unten" }] as const;
  const normen = $derived([{ id: "project", label: t("sprite-ansicht-projekt") }, { id: "pal", label: "PAL" }, { id: "ntsc", label: "NTSC" }]);
  const hex = (i: number) => "$" + i.toString(16).toUpperCase().padStart(2, "0");
  const cur = $derived(v?.values[v.pv]);
  const nachbarn = $derived(v ? [v.fi - 1, v.fi, v.fi + 1].filter((i) => i >= 0 && i < v.frames) : []);

  // Bereiche: markieren einen Teil des Satzes in der Übersicht und springen zu dessen erstem Zeichen; „Frei“ nimmt von/bis
  let neu = $state(false);
  let bereichAnsage = $state(""); let bereich = $state<string | null>(null); let custom = $state<[number, number]>([0, 0]);
  const bereiche = $derived(v ? zeichenBereiche(v.target, v.frames) : []);
  const aktiv = $derived.by((): [number, number] | null => { if (!v || bereich === null) return null; if (bereich === "custom") return [Math.max(0, Math.min(custom[0], custom[1])), Math.min(v.frames - 1, Math.max(custom[0], custom[1]))]; const b = bereiche.find((x) => x.id === bereich); return b ? [b.von, b.bis] : null; });
  const marked = $derived(aktiv ? Array.from({ length: aktiv[1] - aktiv[0] + 1 }, (_, i) => aktiv[0] + i) : []);
  function waehleBereich(id: string, name: string) {
    bereich = bereich === id ? null : id; if (!v) return;
    if (bereich === "custom") custom = [v.fi, v.fi];
    if (aktiv) { void selectFrame(aktiv[0]); bereichAnsage = t("zeichensatz-bereich-ansage", { name, von: aktiv[0], bis: aktiv[1] }); }
  }
  let vorlageSel = $state("");
  $effect(() => { const c = v?.charset; if (c && !c.vorlagen.some((x) => x.ref === vorlageSel && !x.missing)) vorlageSel = c.vorlage || c.vorlagen.find((x) => !x.missing)?.ref || ""; });
  const abweichend = $derived(v?.charset?.geaendert ? v.charset.geaendert.filter(Boolean).length : 0);

  // Vorschau im Bild: ein Text in den Zeichen dieses Satzes (Bildschirmcodes: Atari nach ANTIC-Reihenfolge, die übrigen Systeme nach Commodore-Art)
  let text = $state(t("zeichensatz-vorschau-text"));
  let pv: HTMLCanvasElement | undefined = $state();
  const code = (ch: string, atari: boolean): number => { const a = ch.toUpperCase().charCodeAt(0); if (atari) return a >= 32 && a <= 95 ? a - 32 : 0; return a >= 65 && a <= 90 ? a - 64 : a >= 32 && a <= 63 ? a : 32; };
  $effect(() => {
    void [text, v?.thumbs, v?.scene, ui.theme, pv]; if (!pv || !v) return;
    const zeilen = text.split("\n").slice(0, 6); const cols = Math.max(14, ...zeilen.map((z) => z.length)); const k = 4; const zz = 8 * k, W = Math.min(cols, 40) * zz + 24, H = Math.max(zeilen.length, 2) * zz + 24; const dpr = window.devicePixelRatio || 1;
    pv.width = Math.round(W * dpr); pv.height = Math.round(H * dpr); pv.style.width = `${W}px`; pv.style.height = `${H}px`;
    const g = pv.getContext("2d"); if (!g) return; g.setTransform(dpr, 0, 0, dpr, 0, 0); g.fillStyle = v.scene.sky ?? "#3a3a7a"; g.fillRect(0, 0, W, H);
    const atari = v.target === "atari8"; const fg = v.thumbs[0]?.colors[1] ?? "#ffffff";
    zeilen.forEach((zl, r) => { [...zl.slice(0, 40)].forEach((ch, c) => { const th = v.thumbs[code(ch, atari)]; if (!th) return; g.fillStyle = th.colors[1] ?? fg; for (let y = 0; y < 8; y++) for (let x = 0; x < th.w; x++) if ((th.pixels[y * th.w + x] ?? 0) !== 0) g.fillRect(12 + c * zz + x * k * th.aspect, 12 + r * zz + y * k, k * th.aspect, k); }); });
  });
</script>

{#if !v}
  <div class="blank">
    <Icon name="chip" size={28} />
    <h2>{t("zeichensatz-leer-titel")}</h2>
    {#if !ui.project}<p>{t("zeichensatz-leer-kein-projekt")}</p>
    {:else}
      <p>{t("zeichensatz-leer-text")}</p>
      {#if dateien.length}<ul class="files" aria-label={t("zeichensatz-dateien")}>{#each dateien as f (f)}<li><button type="button" class="btn" data-cs="open" onclick={() => void openSprite(f)}>{f}</button></li>{/each}</ul>
      {:else}<p class="dim small">{t("zeichensatz-keine-dateien")}</p>{/if}
      <div class="row"><button type="button" class="btn pri" onclick={() => void pickSprite(true)}>{t("sprite-oeffnen")}</button><button type="button" class="btn" data-act="new" onclick={() => (neu = true)}>{t("zeichensatz-neu")}</button></div>
    {/if}
  </div>
{:else}
  <div class="cs" aria-label={t("view-title-zeichensatz")}>
    <section class="col l" aria-label={t("zeichensatz-uebersicht-titel")}>
      <h3>{v.title.replace(/\.vcchar$/i, "")} <span class="n dim">{t("zeichensatz-anzahl", { n: v.frames })}</span></h3>
      <CharGrid view={v} {marked} changed={v.charset?.geaendert ?? null} />
      <h3>{t("zeichensatz-bereiche")}</h3>
      <div class="chips" role="group" aria-label={t("zeichensatz-bereiche")} data-cs="bereiche">
        {#each bereiche as b (b.id)}<button type="button" class="chipbtn" class:on={bereich === b.id} aria-pressed={bereich === b.id} data-bereich={b.id} onclick={() => waehleBereich(b.id, t(b.label))}>{t(b.label)}</button>{/each}
        <button type="button" class="chipbtn" class:on={bereich === "custom"} aria-pressed={bereich === "custom"} data-bereich="custom" onclick={() => waehleBereich("custom", t("zeichensatz-bereich-frei"))}>{t("zeichensatz-bereich-frei")}</button>
      </div>
      <span class="sr" role="status" aria-live="polite">{bereichAnsage}</span>
      {#if bereich === "custom"}<div class="custom"><label>{t("zeichensatz-bereich-von")} <input class="inp mono" type="number" min="0" max={v.frames - 1} data-cs="from" bind:value={custom[0]} /></label><label>{t("zeichensatz-bereich-bis")} <input class="inp mono" type="number" min="0" max={v.frames - 1} data-cs="to" bind:value={custom[1]} /></label></div>{/if}
      {#if dateien.length > 1}<h3>{t("zeichensatz-dateien")}</h3>
        <ul class="files" data-cs="files">{#each dateien as f (f)}<li><button type="button" class="fbtn" class:on={f === v.rel} aria-current={f === v.rel ? "true" : undefined} onclick={() => void openSprite(f)} title={f}><Icon name="chip" size={14} /><span class="nm">{f.split("/").pop()}</span>{#if f === v.rel && v.dirty}<span class="udot" title={t("sprite-ungespeichert")}>●</span>{/if}</button></li>{/each}</ul>{/if}
    </section>

    <section class="col m" aria-label={t("zeichensatz-flaeche-label")}>
      <div class="bar" role="toolbar" aria-label={t("sprite-werkzeuge")}>
        {#each tools as tl (tl.id)}
          <Tooltip text={`${t(tl.label)} (${tl.key})`}><button type="button" class="ibtn" class:on={v.tool === tl.id} aria-pressed={v.tool === tl.id} aria-label={t(tl.label)} data-tool={tl.id} onclick={() => void setTool(tl.id)}><Icon name={tl.icon} size={17} /></button></Tooltip>
        {/each}
        <span class="sep"></span>
        <Tooltip text={t("cmd-rueckgaengig")}><button type="button" class="ibtn" disabled={!v.canUndo} aria-label={t("cmd-rueckgaengig")} data-act="undo" onclick={() => void spriteUndo()}><Icon name="undo" size={17} /></button></Tooltip>
        <Tooltip text={t("cmd-wiederholen")}><button type="button" class="ibtn" disabled={!v.canRedo} aria-label={t("cmd-wiederholen")} data-act="redo" onclick={() => void spriteRedo()}><Icon name="redo" size={17} /></button></Tooltip>
        <span class="sep"></span>
        <Tooltip text={`${t("sprite-spiegeln-h")} (X)`}><button type="button" class="ibtn" aria-label={t("sprite-spiegeln-h")} data-act="fliph" onclick={() => void flip("h")}><Icon name="fliph" size={17} /></button></Tooltip>
        <Tooltip text={`${t("sprite-spiegeln-v")} (Y)`}><button type="button" class="ibtn" aria-label={t("sprite-spiegeln-v")} data-act="flipv" onclick={() => void flip("v")}><Icon name="flipv" size={17} /></button></Tooltip>
        <span class="grow"></span>
        <Tooltip text={`${t("sprite-raster")} (G)`}><button type="button" class="ibtn" class:on={sp.grid} aria-pressed={sp.grid} aria-label={t("sprite-raster")} data-act="grid" onclick={() => (sp.grid = !sp.grid)}><Icon name="grid" size={17} /></button></Tooltip>
        <Tooltip text={`${t("sprite-zoom-minus")} (−)`}><button type="button" class="ibtn" aria-label={t("sprite-zoom-minus")} data-act="zoomout" onclick={() => zoomStep(-1, fit)}><Icon name="zoomout" size={17} /></button></Tooltip>
        <span class="zoom mono" data-sp="zoom" aria-label={t("sprite-kurz-zoom")}>{t("sprite-zoom-text", { z: String(zoomPct) })}</span>
        <Tooltip text={`${t("sprite-zoom-plus")} (+)`}><button type="button" class="ibtn" aria-label={t("sprite-zoom-plus")} data-act="zoomin" onclick={() => zoomStep(1, fit)}><Icon name="zoomin" size={17} /></button></Tooltip>
        <Tooltip text={`${t("sprite-zoom-einpassen")} (0)`}><button type="button" class="ibtn" class:on={sp.zoom === null} aria-label={t("sprite-zoom-einpassen")} data-act="fit" onclick={zoomFit}><Icon name="max" size={17} /></button></Tooltip>
        <span class="sep"></span>
        <button type="button" class="btn sm" data-act="new" onclick={() => (neu = true)}>{t("zeichensatz-neu")}</button>
        <button type="button" class="btn sm" data-act="png" title={t("sprite-png-tipp")} onclick={() => void loadPng()}>{t("sprite-png")}</button>
        <button type="button" class="btn sm" data-act="export" onclick={() => sp.docId !== null && void openExport(sp.docId)}>{t("export-titel")} …</button>
        <button type="button" class="btn sm" disabled={!v.dirty} data-act="save" onclick={() => void saveSprite()}>{t("cmd-speichern")}</button>
      </div>
      <p class="chip mono" data-cs="chip"><b>{String.fromCharCode(v.target === "atari8" ? v.fi + 32 : v.fi >= 1 && v.fi <= 26 ? v.fi + 64 : v.fi >= 32 && v.fi < 64 ? v.fi : 63)}</b> {t("zeichensatz-chip", { n: v.fi, hex: hex(v.fi) })}</p>
      <div class="edit"><PixelCanvas view={v} bind:fit />
        <div class="side" aria-label={t("zeichensatz-umgebung")}>
          <h3>{t("zeichensatz-umgebung")}</h3>
          <div class="nb">{#each nachbarn as i (i)}{@const th = v.thumbs[i]!}
            <button type="button" class="nbtn" class:on={i === v.fi} data-cs="nachbar" data-i={i} aria-label={t("zeichensatz-ansage", { n: i, hex: hex(i) })} onclick={() => void selectFrame(i)}>
              <FrameThumb pixels={th.pixels} w={th.w} h={th.pixels.length / th.w} aspect={th.aspect} colors={th.colors} boxW={44} boxH={44} /><span class="k mono">{i}</span></button>{/each}</div>
        </div>
      </div>
      <p class="foot small dim" data-sp="fuss">{#if v.dirty}<span class="warn">● {t("sprite-ungespeichert")} · </span>{/if}{t("zeichensatz-fuss", { name: v.title, n: v.fi, hex: hex(v.fi) })}{#if v.sel} · {t("sprite-auswahl-ansage", { w: v.sel[2] - v.sel[0] + 1, h: v.sel[3] - v.sel[1] + 1 })}{/if}</p>
    </section>

    <section class="col r" aria-label={t("zeichensatz-vorschau-titel")}>
      <h3>{t("zeichensatz-vorschau-titel")}</h3>
      <canvas bind:this={pv} class="pvc" data-cs="vorschau" aria-label={t("zeichensatz-vorschau-label")}></canvas>
      <textarea class="inp txt" rows="2" data-cs="text" aria-label={t("zeichensatz-text-label")} bind:value={text}></textarea>
      {#if v.charset}
        <h3>{t("zeichensatz-vorlage-titel")}</h3>
        <div class="vl" data-cs="vorlage">
          <select class="inp" data-cs="vorlage-wahl" aria-label={t("zeichensatz-vorlage-label")} bind:value={vorlageSel}>
            {#each v.charset.vorlagen as x (x.ref)}<option value={x.ref} disabled={x.missing}>{x.label}{x.missing ? ` – ${t("zeichensatz-vorlage-fehlt")}` : ""}</option>{/each}
          </select>
          <button type="button" class="btn" data-act="vorlage" disabled={!vorlageSel} onclick={() => void loadTemplate(vorlageSel)}>{t("zeichensatz-vorlage-laden")}</button>
        </div>
        <p class="small dim">{t("zeichensatz-vorlage-hinweis")}</p>
        {#if v.charset.herkunft}
          <div class="sw2 her" data-cs="herkunft"><span><b>{t("zeichensatz-herkunft")}</b><br />{#each v.charset.herkunft.lines as z (z)}<span class="dim small">{z}<br /></span>{/each}</span><span class="mono badge">{v.charset.herkunft.label}</span></div>
        {:else}<p class="small dim" data-cs="herkunft">{t("zeichensatz-vorlage-keine")}</p>{/if}
        {#if v.charset.geaendert}
          <div class="sw2" data-cs="abweichung"><span class="small" data-cs="abweichung-text">{#if v.charset.geaendert[v.fi]}<b>{t("zeichensatz-geaendert")}</b>{:else}{t("zeichensatz-wie-vorlage")}{/if}<br /><span class="dim">{t("zeichensatz-aenderungen", { n: abweichend })}</span></span>
            <button type="button" class="btn sm" data-act="original" disabled={!v.charset.geaendert[v.fi]} onclick={() => void resetChar(v.fi)}>{t("zeichensatz-original-zurueck")}</button></div>
        {/if}
      {/if}
      <h3>{t("zeichensatz-karte-zeichen")}</h3>
      <div class="sw2"><span><b>{t("zeichensatz-nummer")}</b><br /><span class="dim small">{t("zeichensatz-nummer-hinweis")}</span></span><span class="mono" data-cs="nummer">{v.fi} · {hex(v.fi)}</span></div>
      <div class="sw2"><span><b>{t("zeichensatz-position")}</b><br /><span class="dim small">{t("zeichensatz-position-hinweis", { n: v.blockBytes })}</span></span><span class="mono" data-cs="position">+{hex(v.fi * 8)}</span></div>
      <h3>{t("sprite-karte-farben")}</h3>
      <div class="vals" role="radiogroup" aria-label={t("sprite-kurz-wert")}>
        {#each v.values as w (w.v)}
          <button type="button" class="val" class:on={w.v === v.pv} role="radio" aria-checked={w.v === v.pv} data-val={w.v} title={w.name} onclick={() => void setValue(w.v)}>
            <span class="wsw" class:tr={!w.color} style={w.color ? `background:${w.color}` : ""}></span><span class="k mono">{w.v + 1}</span><span class="vn">{(w.name || t("sprite-wert", { n: w.v + 1 })).split(" (")[0]}</span></button>
        {/each}
      </div>
      {#if v.normChoice}<Segment items={normen} value={v.norm} onselect={(id) => void setNorm(id)} label={t("sprite-ansicht")} small />{/if}
      {#if cur?.role}
        {@const rolle = v.slots.find((s) => s.key === cur.role)}
        <p class="small dim">{t("sprite-farbe-fuer", { name: rolle?.label ?? cur.name })}</p>
        <div class="farbraster" role="radiogroup" aria-label={t("sprite-farbe-fuer", { name: rolle?.label ?? cur.name })} data-sp="palette">
          {#each v.palette as p (p.i)}
            <button type="button" class="pc" class:on={rolle?.index === p.i} role="radio" aria-checked={rolle?.index === p.i} aria-label={`${p.i}`} title={`${p.i}`} data-idx={p.i} style={`background:${p.color}`} onclick={() => void setColor(cur.role!, p.i)}></button>
          {/each}
        </div>
      {/if}
      <h3>{t("sprite-verschieben")}</h3>
      <div class="pfeile" role="group" aria-label={t("sprite-verschieben")}>
        {#each pfeile as p (p.d)}<Tooltip text={t(p.label)}><button type="button" class="btn" aria-label={t(p.label)} data-shift={p.d} onclick={() => void shift(p.d)}><Icon name={p.icon} size={16} /></button></Tooltip>{/each}
      </div>
      <button type="button" class="btn" data-act="clear" onclick={() => void clearImage()}>{t("zeichensatz-zeichen-leeren")}</button>
    </section>
  </div>
{/if}

<NewSpriteDialog bind:open={neu} kinds={["charset"]} title="zeichensatz-neu-titel" defaultName={t("zeichensatz-neu-name")} />

{#if askOpen}
  <Dialog bind:open={askOpen} title={t("zeichensatz-frage-titel")}>
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
  .blank h2 { color: var(--text); font-size: 18px; } .blank p { max-width: 420px; } .row { display: flex; gap: 6px; flex-wrap: wrap; }
  .cs { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(260px, 340px) minmax(0, 1fr) 280px; gap: 10px; padding: 10px; overflow: auto; }
  @media (max-width: 1000px) { .cs { grid-template-columns: 260px minmax(0, 1fr); grid-auto-rows: min-content; } .col.r { grid-column: 1 / -1; } .col { overflow: visible; } }
  @media (max-width: 640px) { .cs { grid-template-columns: 1fr; } }
  .col { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 8px; min-height: 0; overflow: auto; }
  .col.m { min-height: 360px; }
  h3 { margin: 4px 0 0; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); } .n { font-weight: normal; letter-spacing: 0; text-transform: none; float: right; }
  .files { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; overflow: auto; }
  .fbtn { display: flex; align-items: center; gap: 7px; width: 100%; padding: 6px 8px; border-radius: 8px; color: var(--text); text-align: left; } .fbtn:hover { background: var(--hover); } .fbtn.on { background: var(--accentbg); color: var(--accent2); }
  .nm { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } .udot { color: var(--warn); }
  .bar { display: flex; align-items: center; gap: 4px; flex-wrap: wrap; } .grow { flex: 1; } .sep { width: 1px; height: 20px; background: var(--line2); margin: 0 6px; }
  .zoom { min-width: 56px; text-align: center; font-size: 12px; color: var(--dim); }
  .chip { align-self: flex-start; margin: 0; padding: 5px 12px; border-radius: 10px; border: 1px solid var(--line2); background: var(--raise); font-size: 12px; } .chip b { margin-right: 6px; }
  .edit { display: flex; gap: 12px; align-items: stretch; flex: 1; min-height: 380px; } .edit > :global(*:first-child) { flex: 1; min-width: 0; min-height: 360px; } .side { flex: none; width: 176px; }
  .nb { display: flex; gap: 6px; } .nbtn { display: flex; flex-direction: column; align-items: center; gap: 2px; padding: 3px; border-radius: 8px; background: var(--raise); border: 1px solid var(--line2); } .nbtn.on { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent); }
  .foot { margin: 0; } .warn { color: var(--warn); } .dim { color: var(--dim); } .small { font-size: 12px; }
  .pvc { max-width: 100%; height: auto; border-radius: 8px; image-rendering: pixelated; } .txt { width: 100%; resize: vertical; font-family: var(--mono, monospace); text-transform: uppercase; }
  .vals { display: grid; grid-template-columns: 1fr; gap: 6px; }
  .val { display: flex; align-items: center; gap: 6px; padding: 6px 7px; border-radius: 8px; background: var(--raise); border: 1px solid var(--line2); color: var(--text); font-size: 12px; text-align: left; min-width: 0; }
  .val.on { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent); }
  .wsw { width: 16px; height: 16px; border-radius: 4px; border: 1px solid var(--line2); flex: none; } .wsw.tr { background: repeating-conic-gradient(var(--check1) 0 25%, var(--check2) 0 50%) 0 0 / 8px 8px; }
  .k { color: var(--dim); } .vn { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .farbraster { display: grid; grid-template-columns: repeat(8, 1fr); gap: 3px; }
  .sw2 { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 4px 0; border-top: 1px solid var(--line); }
  .pfeile { display: grid; grid-template-columns: repeat(4, 1fr); gap: 6px; } .pfeile :global(.btn) { justify-content: center; }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); white-space: nowrap; }
  .chips { display: flex; flex-wrap: wrap; gap: 5px; } .chipbtn { padding: 4px 10px; border-radius: 999px; border: 1px solid var(--line2); background: var(--raise); color: var(--text); font-size: 12px; } .chipbtn.on { border-color: var(--accent); background: var(--accentbg); color: var(--accent2); }
  .custom { display: flex; gap: 8px; } .custom label { display: flex; align-items: center; gap: 5px; font-size: 12px; color: var(--dim); } .custom .inp { width: 72px; }
  .vl { display: flex; gap: 6px; flex-wrap: wrap; } .vl .inp { flex: 1; min-width: 140px; } .her { align-items: flex-start; } .badge { padding: 2px 8px; border-radius: 6px; border: 1px solid var(--line2); background: var(--raise); font-size: 11px; white-space: nowrap; }
  .pc { aspect-ratio: 1; border-radius: 5px; border: 1px solid var(--line2); min-width: 14px; } .pc.on { outline: 2px solid var(--accent); outline-offset: 1px; }
</style>
