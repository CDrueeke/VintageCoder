<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { openExport } from "../lib/export.svelte";
  import { answerAsk, fertig, openPalette, pal, paletteDateien, paletteRedo, paletteUndo, pickPalette, savePalette, setHex, setKanal, setSel, standardPalette } from "../lib/palette.svelte";
  import Dialog from "../lib/components/Dialog.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import Tooltip from "../lib/components/Tooltip.svelte";
  import { runCommand } from "../lib/commands";
  import { ui } from "../lib/state.svelte";
  import { t } from "../lib/i18n.svelte";
  // Palette-Editor (E7, nach Mockup neu-10): links das Farbraster (16 Spalten, Auswahlliste mit einer Option je Farbe, Roving Focus), rechts die Karte der gewählten Farbe
  // (Farbfeld, Regler Rot/Grün/Blau, Hexwert), „Im Einsatz“ (Dateien, die den Farbindex verwenden) und „Ausgabe“ (Export).
  const v = $derived(pal.view);
  const dateien = $derived(paletteDateien(ui.project?.tree));
  const COLS = 16;
  let askOpen = $state(false);
  $effect(() => { askOpen = pal.ask !== null; });
  $effect(() => { if (!askOpen && pal.ask) void answerAsk("cancel"); });
  const hexKurz = (i: number) => "$" + i.toString(16).toUpperCase().padStart(2, "0");
  let fokus = $state(0); let zellen: HTMLElement[] = [];
  $effect(() => { if (v) fokus = v.sel; });
  function waehle(i: number, fokussieren: boolean) { if (!v || i < 0 || i >= v.count) return; fokus = i; if (fokussieren) zellen[i]?.focus(); if (i !== v.sel) void setSel(i); }
  function taste(e: KeyboardEvent) {
    if (!v) return;
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : e.key === "ArrowDown" ? COLS : e.key === "ArrowUp" ? -COLS : e.key === "Home" ? -fokus : e.key === "End" ? v.count - 1 - fokus : e.key === "PageDown" ? COLS * 4 : e.key === "PageUp" ? -COLS * 4 : 0;
    if (!d) return; e.preventDefault(); e.stopPropagation(); waehle(Math.max(0, Math.min(v.count - 1, fokus + d)), true);
  }
  let hexText = $state(""); let hexOk = $state(true);
  $effect(() => { if (v) { hexText = v.hex.toUpperCase(); hexOk = true; } });
  const hexGueltig = (s: string) => /^#?[0-9a-fA-F]{6}$/.test(s.trim());
  function hexEingabe() { hexOk = hexGueltig(hexText); if (hexOk) void setHex(hexText.trim().startsWith("#") ? hexText.trim() : "#" + hexText.trim()); }
  const kanaele = $derived(v ? [{ k: 0, label: t("palette-rot"), farbe: "255,0,0", wert: v.rgb[0] }, { k: 1, label: t("palette-gruen"), farbe: "0,255,0", wert: v.rgb[1] }, { k: 2, label: t("palette-blau"), farbe: "0,0,255", wert: v.rgb[2] }] : []);
  const kindLabel = (k: string) => k === "sprite" ? t("palette-art-sprite") : k === "pmg" ? t("palette-art-pmg") : k === "charset" ? t("palette-art-zeichensatz") : k === "tiles" ? t("palette-art-kacheln") : t("palette-art-bildschirm");
</script>

{#if !v}
  <div class="blank">
    <Icon name="grid" size={28} />
    <h2>{t("palette-leer-titel")}</h2>
    {#if !ui.project}<p>{t("palette-leer-kein-projekt")}</p>
    {:else}
      <p>{t("palette-leer-text")}</p>
      {#if dateien.length}<ul class="files" aria-label={t("palette-pal-dateien")}>{#each dateien as f (f)}<li><button type="button" class="btn" data-pal="open" onclick={() => void openPalette(f)}>{f}</button></li>{/each}</ul>
      {:else}<p class="dim small">{t("palette-keine-dateien")}</p>{/if}
      <div class="row"><button type="button" class="btn pri" onclick={() => void pickPalette()}>{t("sprite-oeffnen")}</button></div>
    {/if}
  </div>
{:else}
  <div class="pe" aria-label={t("view-title-palette")}>
    <section class="col l" aria-label={t("palette-raster-titel")}>
      <div class="bar" role="toolbar" aria-label={t("palette-werkzeuge")}>
        <Tooltip text={t("cmd-rueckgaengig")}><button type="button" class="ibtn" disabled={!v.canUndo} aria-label={t("cmd-rueckgaengig")} data-act="undo" onclick={() => void paletteUndo()}><Icon name="undo" size={17} /></button></Tooltip>
        <Tooltip text={t("cmd-wiederholen")}><button type="button" class="ibtn" disabled={!v.canRedo} aria-label={t("cmd-wiederholen")} data-act="redo" onclick={() => void paletteRedo()}><Icon name="redo" size={17} /></button></Tooltip>
        <span class="sep"></span>
        <b class="nm">{v.name || v.title.replace(/\.vcpal$/i, "")}</b><span class="dim small">{t("palette-anzahl", { n: v.count })}</span>
        <span class="grow"></span>
        <button type="button" class="btn sm" data-act="standard" onclick={() => void standardPalette()}>{t("palette-standard")}</button>
        <button type="button" class="btn sm" data-act="export" onclick={() => pal.docId !== null && void openExport(pal.docId)}>{t("export-titel")} …</button>
        <button type="button" class="btn sm" disabled={!v.dirty} data-act="save" onclick={() => void savePalette()}>{t("cmd-speichern")}</button>
      </div>
      <div class="flaeche">
        <div class="raster" role="listbox" aria-label={t("palette-raster-label", { n: v.count })} tabindex="-1" onkeydown={taste} style={`grid-template-columns:repeat(${COLS}, minmax(0, 1fr))`}>
          {#each v.entries as c, i (i)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <div role="option" class="fz" class:on={i === v.sel} bind:this={zellen[i]} tabindex={i === fokus ? 0 : -1} aria-selected={i === v.sel} aria-setsize={v.count} aria-posinset={i + 1}
              aria-label={t("palette-farbe-ansage", { n: i, hex: hexKurz(i), farbe: c.toUpperCase() })} data-pal="farbe" data-i={i} style={`background:${c}`} onclick={() => waehle(i, true)}></div>
          {/each}
        </div>
      </div>
      {#if dateien.length > 1}<ul class="files" data-pal="files">{#each dateien as f (f)}<li><button type="button" class="fbtn" class:on={f === v.rel} aria-current={f === v.rel ? "true" : undefined} onclick={() => void openPalette(f)} title={f}><Icon name="grid" size={14} /><span class="n">{f.split("/").pop()}</span>{#if f === v.rel && v.dirty}<span class="udot" title={t("sprite-ungespeichert")}>●</span>{/if}</button></li>{/each}</ul>{/if}
    </section>

    <section class="col r" aria-label={t("palette-farbe-titel", { n: v.sel })}>
      <h3>{t("palette-farbe-titel", { n: v.sel })}</h3>
      <div class="swatch" data-pal="swatch" style={`background:${v.hex}`} role="img" aria-label={t("palette-farbfeld", { hex: v.hex.toUpperCase() })}></div>
      <div class="regler" role="group" aria-label={t("palette-regler")}>
        {#each kanaele as c (c.k)}
          <label class="rg"><span class="lbl">{c.label}</span>
            <input type="range" min="0" max="255" value={c.wert} data-pal={`kanal-${c.k}`} aria-label={c.label} style={`--c:${c.farbe}`}
              oninput={(e) => void setKanal(c.k, Number(e.currentTarget.value))} onchange={() => void fertig()} />
            <span class="wert mono" data-pal={`wert-${c.k}`}>{c.wert}</span></label>
        {/each}
      </div>
      <h3>{t("palette-hexwert")}</h3>
      <input class="inp mono" data-pal="hex" aria-label={t("palette-hexwert")} aria-invalid={!hexOk} bind:value={hexText} onchange={hexEingabe} oninput={() => (hexOk = hexGueltig(hexText))} />
      {#if !hexOk}<p class="small bad" role="status">{t("palette-hex-ungueltig")}</p>{/if}
      <h3>{t("palette-im-einsatz")}</h3>
      {#if v.uses.length}<ul class="uses" data-pal="einsatz">{#each v.uses as u (u.file)}<li><button type="button" class="lnk" onclick={() => void runCommand(`file:${u.file}`)}>{u.file}</button> <span class="dim small">{kindLabel(u.kind)}</span></li>{/each}</ul>
      {:else}<p class="small dim" data-pal="einsatz">{t("palette-nicht-im-einsatz")}</p>{/if}
      <h3>{t("palette-ausgabe")}</h3>
      <div class="ausg"><span class="small dim">{t("palette-ausgabe-text", { n: v.bytes })}</span><button type="button" class="btn sm" data-act="export2" onclick={() => pal.docId !== null && void openExport(pal.docId)}>{t("export-titel")} …</button></div>
      <p class="foot small dim" data-pal="fuss">{#if v.dirty}<span class="warn">● {t("sprite-ungespeichert")} · </span>{/if}{t("palette-fuss", { name: v.title, n: v.sel, hex: hexKurz(v.sel), count: v.count })}</p>
    </section>
  </div>
{/if}

{#if askOpen}
  <Dialog bind:open={askOpen} title={t("palette-frage-titel")}>
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
  .pe { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr) 300px; gap: 10px; padding: 10px; overflow: auto; }
  @media (max-width: 760px) { .pe { grid-template-columns: 1fr; grid-auto-rows: min-content; } .col { overflow: visible; } }
  .col { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 8px; min-height: 0; overflow: auto; }
  h3 { margin: 6px 0 0; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); }
  .bar { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; } .grow { flex: 1; } .sep { width: 1px; height: 20px; background: var(--line2); margin: 0 6px; } .nm { font-size: 14px; }
  .flaeche { background: var(--canvas, #0a0b0e); border-radius: 10px; padding: 14px; display: flex; justify-content: center; }
  .raster { display: grid; gap: 4px; width: min(100%, 640px, max(280px, calc(100vh - 380px))); } .fz { aspect-ratio: 1; border-radius: 6px; border: 1px solid rgba(255, 255, 255, 0.12); cursor: pointer; min-width: 0; }
  .fz.on { outline: 2px solid var(--accent); outline-offset: 2px; } .fz:focus-visible { outline: 3px solid var(--accent2); outline-offset: 2px; }
  .fbtn { display: flex; align-items: center; gap: 7px; width: 100%; padding: 6px 8px; border-radius: 8px; color: var(--text); text-align: left; } .fbtn:hover { background: var(--hover); } .fbtn.on { background: var(--accentbg); color: var(--accent2); }
  .n { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } .udot, .warn { color: var(--warn); }
  .swatch { height: 96px; border-radius: 12px; border: 1px solid var(--line2); }
  .regler { display: flex; flex-direction: column; gap: 8px; } .rg { display: grid; grid-template-columns: 44px 1fr 34px; align-items: center; gap: 8px; } .lbl { color: var(--dim); font-size: 12px; } .wert { text-align: right; font-size: 12px; }
  input[type="range"] { -webkit-appearance: none; appearance: none; height: 8px; border-radius: 4px; background: linear-gradient(90deg, rgb(0, 0, 0), rgb(var(--c))); outline-offset: 3px; }
  input[type="range"]::-webkit-slider-thumb { -webkit-appearance: none; width: 16px; height: 16px; border-radius: 50%; background: #fff; border: 1px solid var(--line2); box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4); }
  .inp { width: 100%; } .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); } .bad { color: var(--err); } .uses { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 3px; }
  .lnk { background: none; border: 0; padding: 0; color: var(--accent2); text-decoration: underline; cursor: pointer; font: inherit; }
  .ausg { display: flex; align-items: center; justify-content: space-between; gap: 8px; } .foot { margin-top: auto; }
</style>
