<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { call } from "../lib/api";
  import { hex, hexBreite, hexSeite, hexSuchen, hx, loadHexFiles, openHex, pickHexDatei } from "../lib/daten.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import Segment from "../lib/components/Segment.svelte";
  import { fail, ui } from "../lib/state.svelte";
  import { oeffneExtern } from "../lib/editor.svelte";
  import { t } from "../lib/i18n.svelte";
  // Hex-Ansicht (E11, nach Mockup neu-12): links die Binärdateien des Projekts, in der Mitte Adresse, Bytes und Text (nur lesen), rechts Auswahl, Deutung (Byte, Wort, Text) und Seitenwahl.
  // Die Bytes sind eine Auswahlliste (Pfeiltasten, Umschalt erweitert); gelesen werden höchstens die ersten 4 MiB.
  $effect(() => { if (ui.project) void loadHexFiles(); });
  const p = $derived(hx.page);
  const sel = $derived(hx.sel ? { lo: Math.min(hx.sel.a, hx.sel.b), hi: Math.max(hx.sel.a, hx.sel.b) } : null);
  const bytesFlach = $derived(p ? p.rows.flatMap((r) => r.bytes) : []);
  const basis = $derived(p ? p.page * p.pageSize : 0);
  const wert = (off: number): number | null => { const v = off >= basis ? bytesFlach[off - basis] : undefined; return v === undefined ? null : v; };
  const kb = (n: number) => (n >= 1024 ? `${(n / 1024).toFixed(1).replace(".", ",")} KB` : `${n} B`);
  const byteW = $derived(sel ? wert(sel.lo) : null);
  const wortW = $derived(sel && wert(sel.lo) !== null && wert(sel.lo + 1) !== null ? wert(sel.lo)! + wert(sel.lo + 1)! * 256 : null);
  const textW = $derived(sel ? Array.from({ length: Math.min(24, sel.hi - sel.lo + 1) }, (_, i) => wert(sel.lo + i)).map((b) => (b !== null && b >= 0x20 && b < 0x7f ? String.fromCharCode(b) : "·")).join("") : "");
  let anker = $state<number | null>(null); let fokus = $state(0);
  const inSel = (o: number) => !!sel && o >= sel.lo && o <= sel.hi;
  function waehle(o: number, shift: boolean) { if (shift && anker !== null) hx.sel = { a: anker, b: o }; else { anker = o; hx.sel = { a: o, b: o }; } fokus = o; }
  function taste(e: KeyboardEvent) {
    if (!p) return; const cur = fokus; const w = p.width; const max = basis + bytesFlach.length - 1;
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : e.key === "ArrowDown" ? w : e.key === "ArrowUp" ? -w : e.key === "PageDown" ? w * 16 : e.key === "PageUp" ? -w * 16 : e.key === "Home" ? -(cur - basis) : e.key === "End" ? max - cur : 0;
    if (!d) return; e.preventDefault(); const n = Math.max(basis, Math.min(max, cur + d)); waehle(n, e.shiftKey);
    queueMicrotask(() => document.querySelector<HTMLElement>(`[data-hx=byte][data-o='${n}']`)?.focus());
  }
  let zug = false;
  const download = async () => { if (!hx.rel) return; await oeffneExtern(hx.rel); };
</script>

<div class="hx" aria-label={t("view-title-hex")}>
  <section class="col l" aria-label={t("hexui-dateien")}>
    <h3>{t("hexui-dateien")}</h3>
    {#if !ui.project}<p class="dim small">{t("hexui-kein-projekt")}</p>
    {:else if !hx.files.length}<p class="dim small" data-hx="empty">{t("hexui-keine-dateien")}</p>
    {:else}<ul class="files" data-hx="files">{#each hx.files as f (f.rel)}<li><button type="button" class="fbtn" class:on={f.rel === hx.rel} aria-current={f.rel === hx.rel ? "true" : undefined} data-hx="file" onclick={() => void openHex(f.rel)} title={f.rel}><Icon name="file" size={14} /><span class="nm">{f.rel.split("/").pop()}</span><span class="dim small">{kb(f.size)}</span></button></li>{/each}</ul>{/if}
    {#if ui.project}<div class="row"><button type="button" class="btn sm" data-hx="pick" onclick={() => void pickHexDatei()}>{t("hexui-datei-oeffnen")}</button></div><p class="dim small">{t("hexui-listen-hinweis")}</p>{/if}
  </section>

  <section class="col m" aria-label={t("hexui-ansicht")}>
    {#if !p}<div class="blank"><Icon name="memory" size={28} /><h2>{t("hexui-leer-titel")}</h2><p>{t("hexui-leer-text")}</p></div>
    {:else}
      <div class="bar">
        <div class="suche"><Icon name="search" size={15} /><input class="inp" data-hx="suche" placeholder={t("hexui-suche")} aria-label={t("hexui-suche")} bind:value={hx.query} onkeydown={(e) => { if (e.key === "Enter") void hexSuchen(); }} /></div>
        <button type="button" class="btn sm" data-hx="suchen" onclick={() => void hexSuchen()}>{t("hexui-weiter-suchen")}</button>
        <Segment small label={t("hexui-breite")} items={[{ id: "16", label: t("hexui-16") }, { id: "8", label: t("hexui-8") }]} value={String(hx.width)} onselect={(id) => void hexBreite(Number(id))} />
        <span class="chip">{t("hexui-nur-lesen")}</span>
      </div>
      {#if hx.status}<p class="small dim" role="status" data-hx="status">{hx.status}</p>{/if}
      <div class="grid mono" data-hx="raster">
        <div class="kopf"><span class="a">{t("hexui-adr")}</span><span class="bs">{#each Array.from({ length: p.width }, (_, i) => i) as i (i)}<i>{hex(i, 2)}</i>{/each}</span><span class="tx">{t("hexui-text")}</span></div>
        <div class="rows" role="listbox" aria-label={t("hexui-liste", { n: p.loaded })} tabindex="-1" onkeydown={taste} onpointerup={() => (zug = false)}>
          {#each p.rows as r (r.off)}
            <div class="z" role="presentation"><span class="a dim">{hex(r.off, 6)}</span>
              <span class="bs" role="presentation">{#each r.bytes as b, i (i)}<!-- svelte-ignore a11y_click_events_have_key_events -->
                <span role="option" class="b" class:sel={inSel(r.off + i)} class:nul={b === 0} tabindex={r.off + i === fokus || (!fokus && r.off === basis && i === 0) ? 0 : -1} aria-selected={inSel(r.off + i)} aria-setsize={bytesFlach.length} aria-posinset={r.off + i - basis + 1} aria-label={t("hexui-byte-ansage", { off: hex(r.off + i, 6), v: hex(b, 2) })} data-hx="byte" data-o={r.off + i}
                  onpointerdown={(e) => { zug = true; waehle(r.off + i, e.shiftKey); }} onpointerenter={() => { if (zug && anker !== null) hx.sel = { a: anker, b: r.off + i }; }}>{hex(b, 2)}</span>{/each}</span>
              <span class="tx dim">{r.text}</span></div>
          {/each}
        </div>
      </div>
    {/if}
  </section>

  <section class="col r" aria-label={t("hexui-seite-label")}>
    {#if p}
      <h3>{t("hexui-auswahl")}</h3>
      {#if sel}
        <div class="kv" data-hx="auswahl"><span>{t("hexui-bereich")}</span><b class="mono">${hex(sel.lo, 4)}{sel.hi !== sel.lo ? `–$${hex(sel.hi, 4)}` : ""}</b></div><p class="small dim">{t("hexui-bytes-ab", { n: sel.hi - sel.lo + 1, off: hex(sel.lo, 4) })}{p.load !== null ? " · " + t("hexui-speicheradresse", { a: hex((p.load + sel.lo - 2) & 0xffff, 4) }) : ""}</p>
        <h3>{t("hexui-gedeutet")}</h3>
        {#if byteW !== null}<div class="kv"><span>{t("hexui-byte")}</span><b class="mono" data-hx="deut-byte">{byteW} · ${hex(byteW, 2)} · %{byteW.toString(2).padStart(8, "0")}</b></div>{/if}
        {#if wortW !== null}<div class="kv"><span>{t("hexui-wort")}</span><b class="mono" data-hx="deut-wort">{wortW} · ${hex(wortW, 4)}</b></div>{/if}
        <div class="kv"><span>{t("hexui-text-deutung")}</span><b class="mono" data-hx="deut-text">{textW}</b></div>
      {:else}<p class="dim small">{t("hexui-nichts-gewaehlt")}</p>{/if}
      <h3>{t("hexui-datei")}</h3>
      <div class="kv"><span class="nm">{hx.rel?.split("/").pop()}</span><span class="dim small mono">{p.total} {t("hexui-bytes")}</span></div>
      {#if p.load !== null}<p class="small dim" data-hx="lade">{t("hexui-ladeadresse", { a: hex(p.load, 4) })}</p>{/if}
      <p class="small dim" data-hx="umfang">{p.truncated ? t("hexui-nur-erste", { mib: 4, n: p.total }) : t("hexui-ganze-datei")}</p>
      <div class="seite" role="group" aria-label={t("hexui-seite-label")}>
        <button type="button" class="ibtn" disabled={p.page === 0} aria-label={t("hexui-anfang")} data-hx="anfang" onclick={() => void hexSeite(0)}>⏮</button>
        <button type="button" class="ibtn" disabled={p.page === 0} aria-label={t("hexui-vorige")} data-hx="zurueck" onclick={() => void hexSeite(p.page - 1)}><Icon name="arrl" size={16} /></button>
        <span class="mono" data-hx="seite">{p.page + 1} / {p.pages}</span>
        <button type="button" class="ibtn" disabled={p.page >= p.pages - 1} aria-label={t("hexui-naechste")} data-hx="weiter" onclick={() => void hexSeite(p.page + 1)}><Icon name="chevr" size={16} /></button>
        <button type="button" class="ibtn" disabled={p.page >= p.pages - 1} aria-label={t("hexui-ende")} data-hx="ende" onclick={() => void hexSeite(p.pages - 1)}>⏭</button>
      </div>
      <p class="small dim">{t("hexui-4kb")}</p>
      <button type="button" class="btn sm" data-hx="extern" onclick={() => void download()}>{t("hexui-extern")}</button>
    {/if}
  </section>
</div>

<style>
  .blank { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; color: var(--dim); text-align: center; padding: 20px; } .blank h2 { color: var(--text); font-size: 18px; }
  .hx { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(200px, 260px) minmax(0, 1fr) 280px; gap: 10px; padding: 10px; overflow: auto; }
  @media (max-width: 1150px) { .hx { grid-template-columns: 220px minmax(0, 1fr); grid-auto-rows: min-content; } .col.r { grid-column: 1 / -1; } .col { overflow: visible; } } @media (max-width: 760px) { .hx { grid-template-columns: 1fr; } }
  .col { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 8px; min-height: 0; overflow: auto; } .col.m { min-height: 460px; }
  h3 { margin: 4px 0 0; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); } .files { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 3px; }
  .fbtn { display: flex; align-items: center; gap: 7px; width: 100%; padding: 6px 8px; border-radius: 8px; color: var(--text); text-align: left; } .fbtn:hover { background: var(--hover); } .fbtn.on { background: var(--accentbg); color: var(--accent2); } .nm { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .bar { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; } .suche { display: flex; align-items: center; gap: 6px; flex: 1; min-width: 200px; } .suche .inp { width: 100%; } .chip { font-size: 12px; padding: 3px 10px; border-radius: 12px; border: 1px solid var(--line2); color: var(--dim); }
  .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); } .grid { background: var(--code_bg, var(--raise)); border-radius: 10px; padding: 8px 10px; overflow: auto; flex: 1; font-size: 13px; max-height: 68vh; }
  .kopf, .z { display: grid; grid-template-columns: 7ch minmax(0, 1fr) 17ch; gap: 12px; align-items: center; } .kopf { color: var(--dim); font-size: 11px; position: sticky; top: 0; background: var(--code_bg, var(--raise)); padding-bottom: 4px; }
  .bs { display: flex; gap: 4px; flex-wrap: nowrap; } .bs i { font-style: normal; width: 2.4ch; text-align: center; } .b { width: 2.4ch; text-align: center; border-radius: 3px; cursor: default; } .b.nul { color: var(--faint); } .b.sel { background: var(--accentbg); color: var(--accent2); font-weight: 700; } .b:focus-visible { outline: 2px solid var(--accent2); }
  .kv { display: flex; align-items: baseline; justify-content: space-between; gap: 8px; padding: 4px 0; border-bottom: 1px solid var(--line2); font-size: 13px; } .seite { display: flex; align-items: center; justify-content: space-between; gap: 4px; }
</style>
