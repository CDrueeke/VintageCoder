<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { hex, loadSpeicher, speicherPreset, spk } from "../lib/daten.svelte";
  import Segment from "../lib/components/Segment.svelte";
  import { bau } from "../lib/bau.svelte";
  import { zumQuelltext } from "../lib/debug.svelte";
  import { t } from "../lib/i18n.svelte";
  import { ui } from "../lib/state.svelte";
  // Speicherkarte (E11, nach Mockup neu-13): 256 Seiten zu 256 Byte (16 × 16) aus dem letzten Bau. Links die Bereiche nach Art, in der Mitte die Karte mit Filter, rechts die gewählte Seite, die Bankumschaltung und die Prüfungen.
  $effect(() => { void bau.info; void loadSpeicher(); });
  const info = $derived(spk.info);
  const farbe: Record<string, string> = { zeropage: "#9aa3b2", stack: "#6b7686", screen: "#7dd3fc", sprites: "#fbbf24", charset: "#4ade80", music: "#fb923c", io: "#3f4858", rom: "#3a4352", basic: "#e879f9", free: "#232830" };
  const filterOk = (art: string, used: number) => spk.filter === "all" || (spk.filter === "prog" && used > 0) || (spk.filter === "ram" && art === "ram") || (spk.filter === "rom" && /rom|module/.test(art)) || (spk.filter === "io" && art === "io");
  const bereiche = $derived.by(() => { if (!info?.pages || !info.legend) return []; return info.legend.map((l) => { const ps = info.pages!.map((p, i) => ({ k: p[1], i })).filter((x) => x.k === l.key); return { ...l, first: ps[0]?.i ?? 0, n: ps.length }; }); });
  const filter = $derived([{ id: "all", label: t("spk-alle") }, { id: "prog", label: t("spk-programm") }, { id: "ram", label: "RAM" }, { id: "rom", label: "ROM" }, { id: "io", label: t("spk-ea") }]);
  const seiteWaehlen = (i: number) => void loadSpeicher(i);
  let fokus = $state(0); $effect(() => { if (info?.sel !== undefined) fokus = info.sel; });
  function taste(e: KeyboardEvent) {
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : e.key === "ArrowDown" ? 16 : e.key === "ArrowUp" ? -16 : e.key === "Home" ? -fokus : e.key === "End" ? 255 - fokus : 0; if (!d) return;
    e.preventDefault(); const n = Math.max(0, Math.min(255, fokus + d)); fokus = n; seiteWaehlen(n); queueMicrotask(() => document.querySelector<HTMLElement>(`[data-spk=seite][data-p='${n}']`)?.focus());
  }
  const stufe = (l: string) => (l === "ok" ? "✓" : l === "info" ? "·" : l === "warn" ? "⚠" : "✗");
</script>

<div class="sp" aria-label={t("view-title-speicher")}>
  {#if !info?.built}
    <div class="blank" data-spk="empty"><h2>{t("spk-leer-titel")}</h2><p>{t("spk-leer-text", { taste: ui.commands.find((c) => c.id === "bauen")?.keysDisplay ?? "F7" })}</p></div>
  {:else}
    <section class="col l" aria-label={t("spk-bereiche")}>
      <h3>{t("spk-bereiche")}</h3>
      <ul class="liste" data-spk="bereiche">{#each bereiche as b (b.key)}<li><button type="button" class="row" onclick={() => seiteWaehlen(b.first)}><i class="dot" style={`background:${farbe[b.key] ?? "#888"}`}></i><span class="nm">{b.label}</span><span class="mono dim small">${hex(b.first << 8, 4)}</span></button></li>{/each}</ul>
      <h3>{t("spk-programm-titel")}</h3>
      <div class="kv"><span>{t("spk-belegt")}</span><b class="mono" data-spk="used">{info.program} {t("spk-byte")}</b></div>
      <div class="kv"><span>{t("spk-frei")}</span><b class="mono" data-spk="free">{info.free} {t("spk-byte")}</b></div>
    </section>

    <section class="col m" aria-label={t("spk-karte")}>
      <div class="bar"><Segment small label={t("spk-filter")} items={filter} value={spk.filter} onselect={(id) => (spk.filter = id as typeof spk.filter)} /><span class="grow"></span><span class="chip">{t("spk-kaestchen")}</span></div>
      <div class="karte" role="listbox" aria-label={t("spk-karte-label")} tabindex="-1" data-spk="karte" onkeydown={taste}>
        {#each Array.from({ length: 16 }, (_, r) => r) as r (r)}
          <div class="kz" role="presentation"><span class="ra mono dim">${hex(r << 12, 4)}</span>
            {#each Array.from({ length: 16 }, (_, c) => r * 16 + c) as i (i)}{@const pg = info.pages![i]!}
              <button type="button" role="option" class="pz" class:sel={i === info.sel} class:used={pg[2] > 0} class:dim={!filterOk(pg[0], pg[2])} style={`background:${farbe[pg[1]] ?? "#333"}`} tabindex={i === fokus ? 0 : -1} aria-selected={i === info.sel} aria-setsize={256} aria-posinset={i + 1}
                aria-label={t("spk-seite-ansage", { p: hex(i, 2), kat: pg[1], used: pg[2] })} data-spk="seite" data-p={i} onclick={() => seiteWaehlen(i)}>{#if pg[2] > 0}<b style={`height:${Math.max(2, Math.round((pg[2] / 256) * 100))}%`}></b>{/if}</button>
            {/each}</div>
        {/each}
      </div>
      <p class="small dim">{t("spk-hinweis")}</p>
    </section>

    <section class="col r" aria-label={t("spk-seite-titel")}>
      {#if info.detail}{@const d = info.detail}
        <h3>{t("spk-seite", { p: hex(d.page, 2) })}</h3>
        <div class="kv"><span>{t("spk-adresse")}</span><b class="mono" data-spk="adresse">${hex(d.from, 4)}–${hex(d.to, 4)}</b></div>
        <div class="kv"><span>{t("spk-art")}</span><b data-spk="art">{d.kind} · {d.category}</b></div>
        {#if d.used > 0}<div class="kv"><span>{t("spk-programm-belegt")}</span><b class="mono">{d.used} / 256</b></div>{/if}
        {#each d.symbols as s (s[0] + s[1])}<div class="kv small"><span class="nm">{s[0]}</span><b class="mono">${hex(s[1], 4)}</b></div>{/each}
        {#each d.sources as q (q[0] + q[1])}<div class="kv small"><button type="button" class="lnk" onclick={() => void zumQuelltext(q[0], q[1])}>{q[0]}:{q[1]}{q[2] !== q[1] ? `–${q[2]}` : ""}</button></div>{/each}
      {/if}
      <h3>{t("spk-pruefungen")}</h3>
      <ul class="liste" data-spk="pruefungen">{#each info.checks ?? [] as c (c.text)}<li class={`ck ${c.level}`}><span aria-hidden="true">{stufe(c.level)}</span><span>{c.text}</span></li>{/each}</ul>
      {#if info.banking}
        <h3>{info.banking.register.name} (${hex(info.banking.register.addr, 4)})</h3>
        <div class="presets" role="radiogroup" aria-label={t("spk-umschaltung")} data-spk="presets">{#each info.banking.presets as p (p.id)}<button type="button" role="radio" class="pre" class:on={p.id === info.banking.selected} aria-checked={p.id === info.banking.selected} data-spk="preset" onclick={() => void speicherPreset(p.id)}>{p.label}</button>{/each}</div>
        {#if info.banking.presets.find((p) => p.id === info.banking?.selected)?.note}<p class="small dim">{info.banking.presets.find((p) => p.id === info.banking?.selected)?.note}</p>{/if}
      {/if}
    </section>
  {/if}
</div>

<style>
  .blank { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; color: var(--dim); text-align: center; padding: 20px; } .blank h2 { color: var(--text); font-size: 18px; } .blank p { max-width: 440px; }
  .sp { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(200px, 250px) minmax(0, 1fr) 300px; gap: 10px; padding: 10px; overflow: auto; }
  @media (max-width: 1150px) { .sp { grid-template-columns: 220px minmax(0, 1fr); grid-auto-rows: min-content; } .col.r { grid-column: 1 / -1; } .col { overflow: visible; } } @media (max-width: 760px) { .sp { grid-template-columns: 1fr; } }
  .col { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 8px; min-height: 0; overflow: auto; }
  h3 { margin: 4px 0 0; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); } .liste { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 3px; }
  .row { display: flex; align-items: center; gap: 8px; width: 100%; padding: 6px 8px; border-radius: 8px; color: var(--text); text-align: left; } .row:hover { background: var(--hover); } .dot { width: 10px; height: 10px; border-radius: 3px; flex: none; } .nm { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; }
  .bar { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; } .grow { flex: 1; } .chip { font-size: 12px; padding: 3px 10px; border-radius: 12px; border: 1px solid var(--line2); color: var(--dim); } .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); }
  .karte { background: var(--canvas, #0a0b0e); border-radius: 10px; padding: 12px; display: flex; flex-direction: column; gap: 4px; } .kz { display: grid; grid-template-columns: 6ch repeat(16, minmax(0, 1fr)); gap: 4px; align-items: center; } .ra { font-size: 11px; text-align: right; padding-right: 6px; }
  .pz { aspect-ratio: 1.15; border-radius: 6px; border: 1px solid rgba(255, 255, 255, 0.1); position: relative; padding: 0; min-width: 0; cursor: pointer; } .pz.dim { opacity: 0.28; } .pz.sel { outline: 2px solid var(--accent); outline-offset: 1px; } .pz:focus-visible { outline: 3px solid var(--accent2); outline-offset: 1px; }
  .pz b { position: absolute; left: 3px; right: 3px; bottom: 2px; background: var(--accent2); border-radius: 2px; opacity: 0.9; }
  .kv { display: flex; align-items: baseline; justify-content: space-between; gap: 8px; padding: 4px 0; border-bottom: 1px solid var(--line2); font-size: 13px; } .lnk { color: var(--accent2); text-decoration: underline; font-size: 12px; }
  .presets { display: flex; flex-direction: column; gap: 3px; } .pre { text-align: left; padding: 4px 8px; border-radius: 6px; font-size: 12px; border: 1px solid var(--line2); } .pre.on { background: var(--accentbg); border-color: var(--accent); color: var(--accent2); }
  .ck { display: flex; gap: 8px; font-size: 12px; } .ck.warn { color: var(--warn); } .ck.err { color: var(--err, #f87171); } .ck.ok { color: var(--ok, #4ade80); }
</style>
