<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import Ziele from "./einst/Ziele.svelte";
  import Roms from "./einst/Roms.svelte";
  import Segment from "../lib/components/Segment.svelte";
  import Switch from "../lib/components/Switch.svelte";
  import { cfg, ordnerWaehlen, romsLaden, zieleLaden, zl } from "../lib/settings.svelte";
  import { ed, fontStep, setPrettyLine, toggleWrap } from "../lib/editor.svelte";
  import { setLanguage, setTheme, setZoom, ui, type Theme } from "../lib/state.svelte";
  import { schalten, thema, wizBeenden, wizRueckgaengig, wizSchritt, wizSetzen, wizSpaeter, wizSpeichern, wizSystem, wizUebernehmen, wizWeg, wz } from "../lib/wiz.svelte";
  import { t } from "../lib/i18n.svelte";
  import type { LangChoice } from "../lib/api";
  // Einrichtungsassistent (E12.3): ein Assistent, zwei Wege (Schnelleinrichtung, alles selbst) und Teil-Assistenten. Er arbeitet auf einem Entwurf; geschrieben wird erst bei „Übernehmen“ (mit Sicherung, danach Rückgängig).
  const d = $derived(wz.data); const th = $derived(thema()); const idx = $derived(wz.step);
  const gruppen = $derived.by(() => { const g: { name: string; items: { id: string; title: string; i: number }[] }[] = []; (d?.steps ?? []).forEach((s, i) => { let x = g.find((y) => y.name === s.group); if (!x) { x = { name: s.group, items: [] }; g.push(x); } x.items.push({ id: s.id, title: s.title, i }); }); return g; });
  $effect(() => { if (th === "emulators" || th === "hardware" || th === "start_target" || th === "toolchain") void zieleLaden(); });
  $effect(() => { if (th === "roms") void romsLaden(); });
  const sprachen = $derived([{ id: "auto", label: t("einst-sprache-auto") }, { id: "de", label: "Deutsch" }, { id: "en", label: "English" }]);
  const schemata = $derived([{ id: "system", label: t("einst-schema-system") }, { id: "dark", label: t("einst-schema-dunkel") }, { id: "light", label: t("einst-schema-hell") }]);
  const geaendert = $derived((d?.changes ?? []).filter((c) => c.kind !== "same"));
  const bereiche = $derived([...new Set(geaendert.map((c) => c.area))]);
  async function schema(id: string) { setTheme(id as Theme); await wizSetzen({ view: { scheme: id } }); }
  async function zoom(dz: number) { setZoom(ui.zoom + dz); await wizSetzen({ view: { zoom: ui.zoom } }); }
  async function schrift(ds: number) { fontStep(ds); await wizSetzen({ view: { font: ed.fontSize } }); }
  const hatSystem = (id: string) => !!(d?.settings.targets as { system: string }[] | undefined)?.some((x) => x.system === id);
  const letzter = $derived(!!d && idx >= d.steps.length - 1); const vorletzterSummary = $derived(th === "summary");
  function taste(e: KeyboardEvent) { if (e.key === "Escape") { e.preventDefault(); void wizBeenden(true); } }
  async function ordner(e: Event) { const v = (e.currentTarget as HTMLInputElement).value.replace(/^~/, cfg.home ?? "~"); if (cfg.draft) cfg.draft.projects_dir = v; await wizSpeichern(); }
  async function ordnerWahl() { await ordnerWaehlen(); await wizSpeichern(); }
  // „Erstes Projekt“: nach dem Übernehmen direkt ein neues Projekt anlegen oder das erste kopierte Beispiel öffnen
  const bsp = $derived(wz.report?.copied.find((c) => /\/(jumprun|hello)$/.test(c)) ?? wz.report?.copied[0] ?? "");
  async function erstesProjekt(was: string) {
    const dest = wz.data?.examples.destEffective;   // vor dem Beenden merken
    await wizBeenden(false);
    if (was === "::new") { const { runCommand } = await import("../lib/commands"); await runCommand("projekt_neu"); return; }
    if (dest) { const { openProject } = await import("../lib/state.svelte"); await openProject(`${dest}/${was.replace(/^atari8\//, "atari/")}`); }
  }
</script>

<svelte:window onkeydown={taste} />
{#if d}
<div class="wz" aria-label={t("wiz-titel")} data-wz="assistent">
  <nav class="rail" aria-label={t("wiz-schritte")}>
    <h3>{t("wiz-titel")}</h3>
    {#each gruppen as g (g.name)}<div class="grp"><span class="gn">{g.name}</span>{#each g.items as s (s.id)}<button type="button" class="st" class:on={s.i === idx} class:done={s.i < idx} aria-current={s.i === idx ? "step" : undefined} data-wz="schritt" data-s={s.id} disabled={s.i > idx + 1 && s.i !== idx} onclick={() => { void wizSpeichern(); wz.step = s.i; }}>{s.i + 1}. {s.title}</button>{/each}</div>{/each}
  </nav>

  <section class="main" aria-label={d.steps[idx]?.title} data-wz={`thema-${th}`}>
    <h2>{d.steps[idx]?.title}</h2>
    {#if th === "welcome"}
      <div class="card"><h3>{t("einst-sprache")}</h3><Segment small label={t("einst-sprache")} items={sprachen} value={ui.choice === "xx" ? "auto" : ui.choice} onselect={(id) => void setLanguage(id as LangChoice)} /></div>
      <div class="card"><p>{t("wiz-willkommen-text")}</p>
        <div class="row"><button type="button" class="btn" class:pri={d.mode === "quick"} data-wz="weg-schnell" onclick={() => void wizWeg("quick")}>{t("wiz-schnell")}</button><button type="button" class="btn" class:pri={d.mode === "full"} data-wz="weg-voll" onclick={() => void wizWeg("full")}>{t("wiz-voll")}</button></div>
        <p class="small dim">{t("wiz-keine-installation")}</p></div>
      {#if d.later.length}<p class="small warn">{t("wiz-unvollstaendig")}</p>{/if}
    {:else if th === "design"}
      <div class="card"><h3>{t("einst-schema")}</h3><Segment small label={t("einst-schema")} items={schemata} value={d.view.scheme} onselect={(id) => void schema(id)} /></div>
      <div class="card"><h3>{t("einst-zoom")}</h3><div class="row"><button type="button" class="ibtn" aria-label={t("einst-zoom-kleiner")} data-wz="zoom-minus" onclick={() => void zoom(-0.1)}>−</button><span class="mono" data-wz="zoom">{Math.round(ui.zoom * 100)} %</span><button type="button" class="ibtn" aria-label={t("einst-zoom-groesser")} data-wz="zoom-plus" onclick={() => void zoom(0.1)}>+</button></div></div>
      <div class="card"><h3>{t("einst-schrift")}</h3><div class="row"><button type="button" class="ibtn" aria-label={t("einst-schrift-kleiner")} onclick={() => void schrift(-1)}>−</button><span class="mono">{ed.fontSize} pt</span><button type="button" class="ibtn" aria-label={t("einst-schrift-groesser")} onclick={() => void schrift(1)}>+</button></div></div>
    {:else if th === "systems"}
      <div class="card"><p>{t("wiz-systeme-text")}</p><ul class="liste" data-wz="systeme">{#each zl.info as s (s.id)}<li><Switch checked={hatSystem(s.id)} label={`${s.name} · ${s.group}`} onchange={(on) => void wizSystem(s.id, on)} /></li>{/each}</ul></div>
    {:else if th === "toolchain"}
      <div class="card"><p>{t("einst-assembler-eingebaut")}</p><p class="small dim">{t("wiz-emulatoren-text")}</p></div><Ziele />
    {:else if th === "emulators" || th === "hardware" || th === "start_target"}
      <p class="small dim">{th === "emulators" ? t("wiz-emulatoren-text") : th === "hardware" ? t("wiz-hardware-text") : t("ziele-start-hinweis")}</p><Ziele />
    {:else if th === "roms"}<Roms />
    {:else if th === "editor"}
      <div class="card"><h3>{t("einst-schrift")}</h3><div class="row"><button type="button" class="ibtn" aria-label={t("einst-schrift-kleiner")} onclick={() => void schrift(-1)}>−</button><span class="mono">{ed.fontSize} pt</span><button type="button" class="ibtn" aria-label={t("einst-schrift-groesser")} onclick={() => void schrift(1)}>+</button></div></div>
      <div class="card"><Switch checked={ed.wrap} label={t("einst-umbruch")} onchange={() => toggleWrap()} /><Switch checked={ed.prettyLine} label={t("einst-pretty")} onchange={(on) => setPrettyLine(on)} /></div>
    {:else if th === "projects"}
      <div class="card"><h3>{t("einst-projektordner")}</h3><div class="row"><input class="inp" data-wz="projekte" aria-label={t("einst-projektordner")} value={cfg.draft?.projects_dir || d.projects} onchange={ordner} /><button type="button" class="btn sm" onclick={() => void ordnerWahl()}>{t("einst-waehlen")}</button></div><p class="small dim">{t("einst-projektordner-hinweis")}</p></div>
      <div class="card"><h3>{t("wiz-beispiele")}</h3>
        {#if !d.examples.source}<p class="small dim">{t("wiz-beispiele-fehlen")}</p>{:else}
          <Switch checked={d.examples.copy} label={t("wiz-beispiele-kopieren")} onchange={(on) => void wizSetzen({ examples: { copy: on } })} />
          <p class="small dim" data-wz="beispiel-ziel">{t("wiz-beispiele-ziel", { p: d.examples.destEffective })}</p>
          {#if d.examples.copy}<ul class="liste" data-wz="beispiele">{#each d.examples.items as b (b.id)}<li><label class="chk"><input type="checkbox" checked={!d.examples.excluded.includes(b.id)} onchange={(e) => void wizSetzen({ examples: { excluded: e.currentTarget.checked ? d.examples.excluded.filter((x) => x !== b.id) : [...d.examples.excluded, b.id] } })} />{b.id}</label></li>{/each}</ul>{/if}
        {/if}</div>
    {:else if th === "summary"}
      <div class="card" data-wz="aenderungen">
        {#if wz.report}<p class="ok" data-wz="angewandt" role="status">{t("wiz-uebernommen")}</p>{#if wz.report.copied.length}<p class="small dim">{t("wiz-beispiele-kopiert", { n: wz.report.copied.length })}</p>{/if}{#if wz.report.backup}<p class="small dim">{t("wiz-sicherung", { p: wz.report.backup })}</p>{/if}<button type="button" class="btn sm" data-wz="rueckgaengig" onclick={() => void wizRueckgaengig()}>{t("wiz-rueckgaengig")}</button>
        {:else if !geaendert.length}<p data-wz="nothing">{t("wiz-nichts-zu-aendern")}</p>
        {:else}<p>{t("wiz-aenderungen-text")}</p>
          {#each bereiche as b (b)}<h3>{b}</h3><ul class="liste">{#each geaendert.filter((c) => c.area === b) as c (c.key)}<li class="chg"><label class="chk"><input type="checkbox" data-wz="aenderung" data-k={c.key} checked={!wz.abgewaehlt.includes(c.key)} onchange={() => schalten(c.key)} /><b class={c.kind}>{c.kind === "new" ? "+" : c.kind === "removed" ? "−" : "~"}</b> {c.label}</label><span class="small dim">{c.old ? `${c.old} → ` : ""}{c.new}</span>{#if c.note}<span class="small dim"> · {c.note}</span>{/if}</li>{/each}</ul>{/each}
        {/if}</div>
      {#if d.problems.length}<div class="card"><h3>{t("wiz-hinweise")}</h3><ul class="liste">{#each d.problems as p (p.text)}<li class="small warn">{p.text}</li>{/each}</ul></div>{/if}
    {:else if th === "done"}
      <div class="card"><p data-wz="finished">{t("wiz-fertig-text")}</p><p class="small dim">{t("wiz-fertig-spaeter")}</p></div>
      <div class="card" data-wz="erstes"><h3>{t("assistent-erstes-projekt")}</h3><p class="small dim">{t("wiz-erstes-text")}</p>
        <div class="row"><button type="button" class="btn pri sm" data-wz="erstes-neu" onclick={() => void erstesProjekt("::new")}>{t("wiz-erstes-neu")}</button>
          {#if bsp}<button type="button" class="btn sm" data-wz="erstes-beispiel" onclick={() => void erstesProjekt(bsp)}>{t("wiz-erstes-beispiel", { name: bsp })}</button>{/if}</div></div>
    {/if}

    <div class="bar">
      <button type="button" class="btn" data-wz="abbruch" onclick={() => void wizBeenden(true)}>{t("wiz-abbrechen")}</button>
      <button type="button" class="btn" data-wz="spaeter" onclick={() => void wizSpaeter()}>{t("wiz-spaeter")}</button>
      <span class="grow"></span>
      <button type="button" class="btn" data-wz="zurueck" disabled={idx === 0 || (wz.report !== null && th === "summary")} onclick={() => void wizSchritt(-1)}>{t("wiz-zurueck")}</button>
      {#if vorletzterSummary && !wz.report && geaendert.length}<button type="button" class="btn pri" data-wz="uebernehmen" disabled={wz.busy} onclick={() => void wizUebernehmen()}>{t("wiz-uebernehmen")}</button>
      {:else if letzter}<button type="button" class="btn pri" data-wz="schliessen" onclick={() => void wizBeenden()}>{t("wiz-schliessen")}</button>
      {:else}<button type="button" class="btn pri" data-wz="weiter" onclick={() => void wizSchritt(1)}>{t("wiz-weiter")}</button>{/if}
    </div>
  </section>
</div>
{/if}

<style>
  .wz { flex: 1; min-height: 0; display: grid; grid-template-columns: 240px minmax(0, 1fr); gap: 10px; padding: 10px; overflow: auto; } @media (max-width: 800px) { .wz { grid-template-columns: 1fr; } }
  .rail, .main { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 8px; overflow: auto; }
  h2 { margin: 0; font-size: 18px; } h3 { margin: 4px 0 4px; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); }
  .grp { display: flex; flex-direction: column; gap: 2px; margin-bottom: 6px; } .gn { font-size: 10px; text-transform: uppercase; color: var(--faint); letter-spacing: 0.07em; padding: 2px 8px; }
  .st { text-align: left; padding: 6px 10px; border-radius: 8px; color: var(--dim); } .st.done { color: var(--text); } .st.on { background: var(--accentbg); color: var(--accent2); } .st:disabled { opacity: 0.45; }
  .card { flex: none; background: var(--raise); border-radius: 10px; padding: 12px; display: flex; flex-direction: column; gap: 8px; max-width: 760px; } .card p { margin: 0; }
  .row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; } .row .inp { flex: 1; min-width: 200px; } .small { font-size: 12px; } .dim { color: var(--dim); } .warn { color: var(--warn); } .ok { color: var(--ok); }
  .liste { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; } .chk { display: flex; align-items: center; gap: 8px; } .chg { display: flex; flex-direction: column; } b.new { color: var(--ok); } b.removed { color: var(--err); } b.changed { color: var(--warn); }
  .bar { display: flex; gap: 8px; align-items: center; margin-top: auto; padding-top: 10px; position: sticky; bottom: -12px; background: var(--card); } .grow { flex: 1; }
</style>
