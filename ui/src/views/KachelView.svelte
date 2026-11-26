<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { openExport } from "../lib/export.svelte";
  import { answerAsk, askNo, askYes, bit7, brushColor, brushMode, charsetAdd, charsetRemove, clearTile, copyTile, flipTile, kachelDateien, kachelRedo, kachelUndo, kc, openKacheln, paint, pasteTile, pickCell, pickKacheln, propNew, renameTile, rollTile, rotateTile, saveKacheln, selCell, selTile, setBg, setChar, setColorMode, setProp, setSize, setTab, tileColor, tileDelete, tileDup, tileMerge, tileNew } from "../lib/kachel.svelte";
  import ColorGrid from "../lib/components/ColorGrid.svelte";
  import Dialog from "../lib/components/Dialog.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import Segment from "../lib/components/Segment.svelte";
  import Switch from "../lib/components/Switch.svelte";
  import Tooltip from "../lib/components/Tooltip.svelte";
  import { runCommand } from "../lib/commands";
  import { singleKeyOk, ui } from "../lib/state.svelte";
  import { t } from "../lib/i18n.svelte";
  // Kachel-Editor (E8, nach Mockup neu-7): links der Kachelsatz (Liste mit Vorschau und Verwendungszähler), in der Mitte Werkzeuge, die große Kachel (Zellen aus dem Zielzeichensatz)
  // und die Zeichentafel, rechts die Karte der Kachel (Name, Farbe, Eigenschaften), Größe, Füllstand des Zielzeichensatzes und Satz-Aktionen.
  const v = $derived(kc.view);
  const dateien = $derived(kachelDateien(ui.project?.tree));
  const hex = (i: number) => "$" + i.toString(16).toUpperCase().padStart(2, "0");
  let askOpen = $state(false);
  $effect(() => { askOpen = kc.ask !== null; });
  $effect(() => { if (!askOpen && kc.ask) void answerAsk("cancel"); });
  const frageOffen = $derived(v?.ask != null);

  // Kachelliste: Roving Focus, Pfeile
  let tfokus = $state(0); let tzellen: HTMLElement[] = [];
  $effect(() => { if (v) tfokus = v.sel; });
  function tWaehle(i: number, fok: boolean) { if (!v || i < 0 || i >= v.count) return; tfokus = i; if (fok) tzellen[i]?.focus(); if (i !== v.sel) void selTile(i); }
  function spalten(): number { const z = tzellen.filter(Boolean); if (z.length < 2) return 1; const top = z[0]!.offsetTop; let n = 0; for (const e of z) { if (e.offsetTop === top) n++; else break; } return Math.max(1, n); }
  function tTaste(e: KeyboardEvent) {
    if (!v) return; const c = spalten();
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : e.key === "ArrowDown" ? c : e.key === "ArrowUp" ? -c : e.key === "Home" ? -tfokus : e.key === "End" ? v.count - 1 - tfokus : 0;
    if (!d) return; e.preventDefault(); e.stopPropagation(); tWaehle(Math.max(0, Math.min(v.count - 1, tfokus + d)), true);
  }

  // große Kachel: Zellen als Auswahlliste; Zug über mehrere Zellen = eine Absicht
  let zfokus = $state(0); let zz: HTMLElement[] = []; let zug: Set<number> | null = null;
  $effect(() => { if (v) zfokus = v.cur; });
  function zelleAn(e: PointerEvent): number | null { const el = document.elementFromPoint(e.clientX, e.clientY) as HTMLElement | null; const k = el?.closest("[data-k]")?.getAttribute("data-k"); return k == null ? null : Number(k); }
  function zDown(e: PointerEvent, k: number) {
    if (e.button === 2 || e.altKey) { e.preventDefault(); void pickCell(k); return; }
    if (e.button !== 0) return; (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId); zug = new Set([k]); zfokus = k; zz[k]?.focus();
  }
  function zMove(e: PointerEvent) { if (!zug) return; const k = zelleAn(e); if (k !== null) zug.add(k); }
  function zUp() { if (!zug) return; const z = [...zug]; zug = null; void paint(z); }
  function zTaste(e: KeyboardEvent) {
    if (!v) return;
    const dx = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0, dy = e.key === "ArrowDown" ? 1 : e.key === "ArrowUp" ? -1 : 0;
    if (dx || dy) { e.preventDefault(); e.stopPropagation(); const x = Math.max(0, Math.min(v.tw - 1, (zfokus % v.tw) + dx)), y = Math.max(0, Math.min(v.th - 1, Math.floor(zfokus / v.tw) + dy)); const k = y * v.tw + x; zfokus = k; zz[k]?.focus(); void selCell(k); return; }
    if (e.key === "Enter" || e.key === " ") { e.preventDefault(); e.stopPropagation(); void paint([zfokus]); }
    else if ((e.key === "i" || e.key === "I") && singleKeyOk(e)) { e.preventDefault(); e.stopPropagation(); void pickCell(zfokus); }
  }

  // Zeichentafel
  let cfokus = $state(0); let cz: HTMLElement[] = [];
  $effect(() => { if (v) cfokus = v.brush.code; });
  function cWaehle(c: number, fok: boolean) { if (!v || c < 0 || c >= v.tafel.n) return; cfokus = c; if (fok) cz[c]?.focus(); void setChar(c); }
  function cTaste(e: KeyboardEvent) {
    if (!v) return;
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : e.key === "ArrowDown" ? 16 : e.key === "ArrowUp" ? -16 : e.key === "Home" ? -cfokus : e.key === "End" ? v.tafel.n - 1 - cfokus : 0;
    if (!d) return; e.preventDefault(); e.stopPropagation(); cWaehle(Math.max(0, Math.min(v.tafel.n - 1, cfokus + d)), true);
  }

  let nameText = $state(""); $effect(() => { if (v?.tile) nameText = v.tile.name; });
  let neueProp = $state(""); let gw = $state(2), gh = $state(2); $effect(() => { if (v) { gw = v.tw; gh = v.th; } });
  const modi = $derived([{ id: "tile", label: t("kacheln-farbe-je-kachel") }, { id: "cell", label: t("kacheln-farbe-je-zelle") }, { id: "none", label: t("kacheln-farbe-ohne") }]);
  const pmodi = $derived([{ id: "both", label: t("kacheln-pinsel-beides") }, { id: "char", label: t("kacheln-pinsel-zeichen") }, { id: "color", label: t("kacheln-pinsel-farbe") }]);
  const fuellProzent = $derived(v?.fill && !v.fill.err && v.fill.max ? Math.min(100, Math.round((v.fill.used / v.fill.max) * 100)) : 0);
  const quadratisch = $derived(!!v && v.tw === v.th && v.tw > 1);
  let angebotSel = $state(""); $effect(() => { if (v && !v.angebot.includes(angebotSel)) angebotSel = v.angebot[0] ?? ""; });
</script>

{#if !v}
  <div class="blank">
    <Icon name="layers" size={28} />
    <h2>{t("kacheln-leer-titel")}</h2>
    {#if !ui.project}<p>{t("kacheln-leer-kein-projekt")}</p>
    {:else}
      <p>{t("kacheln-leer-text")}</p>
      {#if dateien.length}<ul class="files" aria-label={t("kacheln-dateien")}>{#each dateien as f (f)}<li><button type="button" class="btn" data-kc="open" onclick={() => void openKacheln(f)}>{f}</button></li>{/each}</ul>
      {:else}<p class="dim small">{t("kacheln-keine-dateien")}</p>{/if}
      <div class="row"><button type="button" class="btn pri" onclick={() => void pickKacheln()}>{t("sprite-oeffnen")}</button></div>
    {/if}
  </div>
{:else}
  <div class="ke" aria-label={t("view-title-kacheln")}>
    <section class="col l" aria-label={t("kacheln-satz-titel", { name: v.name })}>
      <h3>{t("kacheln-satz-titel", { name: v.name })} <Tooltip text={t("kacheln-neu")}><button type="button" class="ibtn plus" data-act="tile-new" aria-label={t("kacheln-neu")} onclick={() => void tileNew()}><Icon name="plus" size={16} /></button></Tooltip></h3>
      <div class="liste" role="listbox" aria-label={t("kacheln-liste-label", { n: v.count })} tabindex="-1" onkeydown={tTaste} data-kc="liste">
        {#each v.tiles as k (k.i)}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div role="option" class="kt" class:on={k.i === v.sel} bind:this={tzellen[k.i]} tabindex={k.i === tfokus ? 0 : -1} aria-selected={k.i === v.sel} aria-setsize={v.count} aria-posinset={k.i + 1}
            aria-label={t("kacheln-ansage", { n: k.i, name: k.name, uses: k.uses })} data-kc="kachel" data-i={k.i} onclick={() => tWaehle(k.i, true)}>
            <img src={k.img} alt="" /><span class="kn">{k.name}</span><span class="ku dim mono">{k.uses}×{#if k.double !== null} <span class="dbl" title={t("kacheln-doppelt", { j: k.double })}>=</span>{/if}</span>
          </div>
        {/each}
      </div>
      {#if dateien.length > 1}<h3>{t("kacheln-dateien")}</h3>
        <ul class="files" data-kc="files">{#each dateien as f (f)}<li><button type="button" class="fbtn" class:on={f === v.rel} aria-current={f === v.rel ? "true" : undefined} onclick={() => void openKacheln(f)} title={f}><Icon name="layers" size={14} /><span class="n">{f.split("/").pop()}</span>{#if f === v.rel && v.dirty}<span class="udot" title={t("sprite-ungespeichert")}>●</span>{/if}</button></li>{/each}</ul>{/if}
    </section>

    <section class="col m" aria-label={t("kacheln-flaeche-label")}>
      <div class="bar" role="toolbar" aria-label={t("kacheln-werkzeuge")}>
        <Tooltip text={t("cmd-rueckgaengig")}><button type="button" class="ibtn" disabled={!v.canUndo} aria-label={t("cmd-rueckgaengig")} data-act="undo" onclick={() => void kachelUndo()}><Icon name="undo" size={17} /></button></Tooltip>
        <Tooltip text={t("cmd-wiederholen")}><button type="button" class="ibtn" disabled={!v.canRedo} aria-label={t("cmd-wiederholen")} data-act="redo" onclick={() => void kachelRedo()}><Icon name="redo" size={17} /></button></Tooltip>
        <span class="sep"></span>
        <Tooltip text={t("sprite-spiegeln-h")}><button type="button" class="ibtn" aria-label={t("sprite-spiegeln-h")} data-act="fliph" onclick={() => void flipTile("h")}><Icon name="fliph" size={17} /></button></Tooltip>
        <Tooltip text={t("sprite-spiegeln-v")}><button type="button" class="ibtn" aria-label={t("sprite-spiegeln-v")} data-act="flipv" onclick={() => void flipTile("v")}><Icon name="flipv" size={17} /></button></Tooltip>
        <Tooltip text={t("kacheln-drehen")}><button type="button" class="btn sm" disabled={!quadratisch} aria-label={t("kacheln-drehen")} data-act="rotate" onclick={() => void rotateTile()}>90°</button></Tooltip>
        <Tooltip text={t("sprite-links")}><button type="button" class="ibtn" aria-label={t("kacheln-rollen-links")} data-act="roll-l" onclick={() => void rollTile(-1, 0)}><Icon name="arrl" size={16} /></button></Tooltip>
        <Tooltip text={t("sprite-rechts")}><button type="button" class="ibtn" aria-label={t("kacheln-rollen-rechts")} data-act="roll-r" onclick={() => void rollTile(1, 0)}><Icon name="arrr" size={16} /></button></Tooltip>
        <Tooltip text={t("sprite-oben")}><button type="button" class="ibtn" aria-label={t("kacheln-rollen-oben")} data-act="roll-u" onclick={() => void rollTile(0, -1)}><Icon name="arru" size={16} /></button></Tooltip>
        <Tooltip text={t("sprite-unten")}><button type="button" class="ibtn" aria-label={t("kacheln-rollen-unten")} data-act="roll-d" onclick={() => void rollTile(0, 1)}><Icon name="arrd" size={16} /></button></Tooltip>
        <span class="sep"></span>
        <button type="button" class="btn sm" data-act="clear" onclick={() => void clearTile()}>{t("kacheln-leeren")}</button>
        <button type="button" class="btn sm" data-act="copy" onclick={() => void copyTile()}>{t("cmd-kopieren")}</button>
        <button type="button" class="btn sm" data-act="paste" disabled={!v.hasClip} onclick={() => void pasteTile()}>{t("cmd-einfuegen")}</button>
        <span class="grow"></span>
        <button type="button" class="btn sm" data-act="export" onclick={() => kc.docId !== null && void openExport(kc.docId)}>{t("export-titel")} …</button>
        <button type="button" class="btn sm" disabled={!v.dirty} data-act="save" onclick={() => void saveKacheln()}>{t("cmd-speichern")}</button>
      </div>
      {#if v.colorMode === "cell"}<div class="bar"><span class="small dim">{t("kacheln-pinsel-label")}</span><Segment small label={t("kacheln-pinsel-label")} items={pmodi} value={v.brush.mode} onselect={(id) => void brushMode(id)} /></div>{/if}
      {#if v.tile}
        <p class="chip mono" data-kc="chip"><b>{v.tile.name}</b> {t("kacheln-chip", { w: v.tw, h: v.th, cs: (v.charsets[v.reiter] ?? "").split("/").pop() ?? "" })}</p>
        <div class="arbeit">
          <div class="gross" style={`aspect-ratio:${v.tile.w}/${v.tile.h}`} data-kc="gross">
            <img src={v.tile.img} alt="" draggable="false" />
            <div class="cells" role="listbox" aria-label={t("kacheln-zellen-label", { w: v.tw, h: v.th })} tabindex="-1" style={`grid-template-columns:repeat(${v.tw}, 1fr);grid-template-rows:repeat(${v.th}, 1fr)`} onkeydown={zTaste} onpointermove={zMove} onpointerup={zUp} oncontextmenu={(e) => e.preventDefault()}>
              {#each v.cells as c, k (k)}
                <div role="option" class="zl" class:on={k === v.cur} bind:this={zz[k]} tabindex={k === zfokus ? 0 : -1} aria-selected={k === v.cur} aria-setsize={v.cells.length} aria-posinset={k + 1}
                  aria-label={t("kacheln-zelle-ansage", { x: (k % v.tw) + 1, y: Math.floor(k / v.tw) + 1, code: c.code, hex: hex(c.code) })} data-kc="zelle" data-k={k} onpointerdown={(e) => zDown(e, k)}></div>
              {/each}
            </div>
          </div>
          <div class="tafel">
            <h3>{t("kacheln-zeichen-waehlen")}</h3>
            {#if v.charsets.length > 1}<div class="tabs" role="group" aria-label={t("kacheln-zeichensaetze")}>{#each v.charsets as cs, i (cs)}<button type="button" class="btn sm" class:on={i === v.reiter} aria-pressed={i === v.reiter} data-kc="reiter" onclick={() => void setTab(i)}>{cs.split("/").pop()}</button>{/each}</div>{/if}
            <div class="tbox" style={`aspect-ratio:${v.tafel.w}/${v.tafel.h}`} data-kc="tafel">
              <img src={v.tafel.img} alt="" draggable="false" />
              <div class="tcells" role="listbox" aria-label={t("kacheln-tafel-label", { n: v.tafel.n })} tabindex="-1" onkeydown={cTaste}>
                {#each Array.from({ length: v.tafel.n }, (_, i) => i) as c (c)}
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <div role="option" class="tz" class:on={c === v.brush.code} bind:this={cz[c]} tabindex={c === cfokus ? 0 : -1} aria-selected={c === v.brush.code} aria-setsize={v.tafel.n} aria-posinset={c + 1}
                    aria-label={t("zeichensatz-ansage", { n: c, hex: hex(c) })} title={t("zeichensatz-ansage", { n: c, hex: hex(c) })} data-kc="zeichen" data-c={c} onclick={() => cWaehle(c, true)}></div>
                {/each}
              </div>
            </div>
            <p class="small dim">{t("kacheln-tafel-hinweis")}</p>
            {#if v.target === "atari8" || v.target === "c64" || v.target === "mega65"}<div class="fld"><Switch checked={v.brush.sel} label={t("kacheln-bit7")} onchange={(on) => void bit7(on)} /></div>{/if}
          </div>
        </div>
      {/if}
      <p class="foot small dim" data-kc="fuss">{#if v.dirty}<span class="warn">● {t("sprite-ungespeichert")} · </span>{/if}{t("kacheln-fuss", { name: v.title, n: v.sel, count: v.count })}</p>
    </section>

    <section class="col r" aria-label={t("kacheln-karte-titel")}>
      <h3>{t("kacheln-karte-titel")}</h3>
      <input class="inp" data-kc="name" aria-label={t("kacheln-name")} bind:value={nameText} onchange={(e) => void renameTile(e.currentTarget.value)} />
      <h3>{t("kacheln-farbe-titel")}</h3>
      <Segment small label={t("kacheln-farbmodus")} items={modi} value={v.colorMode} onselect={(id) => void setColorMode(id)} />
      {#if v.colorMode === "tile" && v.tile}<ColorGrid palette={v.palette} value={v.tile.col} label={t("kacheln-kachelfarbe")} onselect={(i) => void tileColor(i)} />
      {:else if v.colorMode === "cell"}<p class="small dim">{t("kacheln-pinselfarbe")}</p><ColorGrid palette={v.palette} value={v.brush.color} label={t("kacheln-pinselfarbe")} onselect={(i) => void brushColor(i)} />{/if}
      <p class="small dim">{t("kacheln-hintergrund")}</p><ColorGrid palette={v.palette} value={v.bg} label={t("kacheln-hintergrund")} onselect={(i) => void setBg(i)} size={14} />
      <h3>{t("kacheln-eigenschaften")}</h3>
      {#each v.propDefs as p (p.bit)}<div class="prop" data-kc="prop"><Switch checked={!!v.tile && (v.tile.props & (1 << p.bit)) !== 0} label={p.name} onchange={(on) => void setProp(p.bit, on)} /></div>
      {:else}<p class="small dim">{t("kacheln-keine-eigenschaften")}</p>{/each}
      {#if v.propDefs.length < 8}<div class="row"><input class="inp" data-kc="prop-neu" aria-label={t("kacheln-eigenschaft-neu")} placeholder={t("kacheln-eigenschaft-neu")} bind:value={neueProp} onkeydown={(e) => { if (e.key === "Enter") { void propNew(neueProp); neueProp = ""; } }} />
        <button type="button" class="btn sm" data-act="prop-neu" disabled={!neueProp.trim()} onclick={() => { void propNew(neueProp); neueProp = ""; }}>{t("kacheln-hinzufuegen")}</button></div>{/if}
      <h3>{t("kacheln-groesse")}</h3>
      <div class="row"><label class="small dim">{t("kacheln-breite")} <input class="inp num" type="number" min="1" max="16" data-kc="gw" bind:value={gw} /></label><label class="small dim">{t("kacheln-hoehe")} <input class="inp num" type="number" min="1" max="16" data-kc="gh" bind:value={gh} /></label>
        <button type="button" class="btn sm" data-act="size" disabled={gw === v.tw && gh === v.th} onclick={() => void setSize(gw, gh)}>{t("kacheln-uebernehmen")}</button></div>
      <h3>{t("kacheln-zielzeichensatz")}</h3>
      {#if v.fill}
        {#if v.fill.err}<p class="small bad" data-kc="fill">{v.fill.err}</p>
        {:else}<div class="bar2" role="progressbar" aria-valuemin="0" aria-valuemax={v.fill.max} aria-valuenow={v.fill.used} aria-label={t("kacheln-fuellstand-label")}><div style={`width:${fuellProzent}%`}></div></div>
          <p class="small" data-kc="fill"><b>{v.fill.used}</b> {t("kacheln-fuellstand", { max: v.fill.max, frei: v.fill.max - v.fill.used })}</p>{/if}
        <p class="small dim">{t("kacheln-fuellstand-hinweis")}</p>
      {/if}
      {#if v.angebot.length}<div class="row"><select class="inp" data-kc="angebot" aria-label={t("kacheln-zeichensatz-hinzu")} bind:value={angebotSel}>{#each v.angebot as a (a)}<option value={a}>{a}</option>{/each}</select><button type="button" class="btn sm" data-act="cs-add" onclick={() => void charsetAdd(angebotSel)}>{t("kacheln-hinzufuegen")}</button></div>{/if}
      {#if v.charsets.length > 1}<div class="row">{#each v.charsets as cs, i (cs)}<button type="button" class="btn sm" data-act="cs-remove" disabled={v.usedCharset[i]} title={v.usedCharset[i] ? t("kacheln-zeichensatz-benutzt") : ""} onclick={() => void charsetRemove(i)}>{t("kacheln-entfernen", { name: cs.split("/").pop() ?? cs })}</button>{/each}</div>{/if}
      {#if v.banks.length}<p class="small dim" data-kc="banken">{t("kacheln-banken", { n: v.banks.length })}</p>{/if}
      <h3>{t("kacheln-satz-aktionen")}</h3>
      <div class="row"><button type="button" class="btn sm" data-act="tile-dup" onclick={() => void tileDup()}>{t("kacheln-duplizieren")}</button>
        <button type="button" class="btn sm" data-act="tile-merge" disabled={v.tiles[v.sel]?.double == null} onclick={() => void tileMerge()}>{t("kacheln-zusammenfuehren")}</button>
        <button type="button" class="btn sm" data-act="tile-delete" disabled={v.count < 2} onclick={() => void tileDelete()}>{t("kacheln-loeschen")}</button></div>
    </section>
  </div>
{/if}

{#if frageOffen && v?.ask}
  <Dialog open={true} title={v.ask.kind === "delete" ? t("kacheln-loeschen-titel") : t("kacheln-groesse-titel")}>
    {#snippet children()}<p data-kc="frage">{v.ask?.kind === "delete" ? t("kacheln-loeschen-text", { i: v.ask.i, uses: v.ask.uses }) : t("kacheln-groesse-text", { w: v.ask?.kind === "size" ? v.ask.w : 0, h: v.ask?.kind === "size" ? v.ask.h : 0 })}</p>{/snippet}
    {#snippet footer()}
      <button type="button" class="btn" data-ask="no" onclick={() => void askNo()}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn pri" data-ask="yes" onclick={() => void askYes()}>{v.ask?.kind === "delete" ? t("kacheln-loeschen") : t("kacheln-uebernehmen")}</button>
    {/snippet}
  </Dialog>
{/if}

{#if askOpen}
  <Dialog bind:open={askOpen} title={t("kacheln-frage-titel")}>
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
  .blank h2 { color: var(--text); font-size: 18px; } .blank p { max-width: 420px; } .row { display: flex; gap: 6px; flex-wrap: wrap; align-items: center; } .files { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  .ke { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(220px, 300px) minmax(0, 1fr) 300px; gap: 10px; padding: 10px; overflow: auto; }
  @media (max-width: 1100px) { .ke { grid-template-columns: 240px minmax(0, 1fr); grid-auto-rows: min-content; } .col.r { grid-column: 1 / -1; } .col { overflow: visible; } } @media (max-width: 760px) { .ke { grid-template-columns: 1fr; } }
  .col { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 8px; min-height: 0; overflow: auto; } .col.m { min-height: 420px; }
  h3 { margin: 4px 0 0; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); display: flex; align-items: center; justify-content: space-between; gap: 6px; } .plus { width: 24px; height: 24px; }
  .liste { display: grid; grid-template-columns: repeat(auto-fill, minmax(84px, 1fr)); gap: 8px; }
  .kt { display: flex; flex-direction: column; align-items: center; gap: 2px; padding: 6px; border-radius: 10px; border: 1px solid transparent; cursor: pointer; min-width: 0; } .kt:hover { background: var(--hover); } .kt.on { border-color: var(--accent); background: var(--accentbg); } .kt:focus-visible { outline: 2px solid var(--accent2); }
  .kt img { width: 100%; max-width: 64px; image-rendering: pixelated; border-radius: 4px; } .kn { font-size: 12px; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } .ku { font-size: 11px; } .dbl { color: var(--warn); font-weight: 700; }
  .fbtn { display: flex; align-items: center; gap: 7px; width: 100%; padding: 6px 8px; border-radius: 8px; color: var(--text); text-align: left; } .fbtn:hover { background: var(--hover); } .fbtn.on { background: var(--accentbg); color: var(--accent2); }
  .n { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } .udot, .warn { color: var(--warn); }
  .bar { display: flex; align-items: center; gap: 4px; flex-wrap: wrap; } .grow { flex: 1; } .sep { width: 1px; height: 20px; background: var(--line2); margin: 0 6px; }
  .chip { align-self: flex-start; margin: 0; padding: 5px 12px; border-radius: 10px; border: 1px solid var(--line2); background: var(--raise); font-size: 12px; } .chip b { margin-right: 6px; }
  .arbeit { display: flex; gap: 16px; flex-wrap: wrap; align-items: flex-start; }
  .gross { position: relative; width: min(100%, 420px); flex: none; background: #000; border-radius: 6px; overflow: hidden; touch-action: none; } .gross img { display: block; width: 100%; height: 100%; image-rendering: pixelated; }
  .cells { position: absolute; inset: 0; display: grid; } .zl { border: 1px solid rgba(255, 255, 255, 0.14); cursor: crosshair; } .zl.on { outline: 2px solid var(--accent); outline-offset: -2px; } .zl:focus-visible { outline: 3px solid var(--accent2); outline-offset: -3px; }
  .tafel { flex: 1; min-width: 240px; max-width: 360px; display: flex; flex-direction: column; gap: 6px; } .tabs { display: flex; gap: 4px; flex-wrap: wrap; } .tabs .on { border-color: var(--accent); background: var(--accentbg); }
  .tbox { position: relative; width: 100%; background: #000; border-radius: 6px; overflow: hidden; } .tbox img { display: block; width: 100%; height: 100%; image-rendering: pixelated; }
  .tcells { position: absolute; inset: 0; display: grid; grid-template-columns: repeat(16, 1fr); } .tz { cursor: pointer; box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.14); } .tz.on { outline: 2px solid var(--accent); outline-offset: -2px; } .tz:hover { background: rgba(255, 255, 255, 0.18); } .tz:focus-visible { outline: 2px solid var(--accent2); outline-offset: -2px; }
  .foot { margin: 0; margin-top: auto; } .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); } .bad { color: var(--err); } .fld { margin: 4px 0; } .prop { margin: 2px 0; }
  .inp { width: 100%; } .inp.num { width: 64px; } .bar2 { height: 8px; border-radius: 4px; background: var(--raise); overflow: hidden; } .bar2 > div { height: 100%; background: var(--accent); }
</style>
