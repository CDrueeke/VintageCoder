<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { call, type RecentProject } from "../lib/api";
  import { runCommand } from "../lib/commands";
  import { openCopy } from "../lib/kopieren.svelte";
  import { currentLang, t } from "../lib/i18n.svelte";
  import { willkommen, wz, wizStarten } from "../lib/wiz.svelte";
  import { fail, openProject, pickProject, ui } from "../lib/state.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import Wortmarke from "../lib/components/Wortmarke.svelte";
  import RecentList from "../lib/components/RecentList.svelte";
  import ExamplesDialog, { type Example } from "../lib/components/ExamplesDialog.svelte";
  // Startseite ohne Projekt: neues Projekt, öffnen, Beispiele, zuletzt geöffnete Projekte (anheften, entfernen, kopieren), erkannte Emulatoren, Tastenkürzel
  let recent = $state<RecentProject[]>([]);
  let loaded = $state(false);
  /** Die Startseite zeigt höchstens so viele zuletzt geöffnete Projekte (angeheftete zuerst) */
  const MAX_ZULETZT = 5;
  let examples = $state<Example[]>([]);
  let tools = $state<{ id: string; label: string; found: boolean; text: string }[]>([]);
  let allExamples = $state(false);
  let filter = $state("");
  const systeme = $derived([...new Set(examples.map((e) => e.system))]);
  const gezeigt = $derived([...examples].sort((a, b) => Number(b.featured) - Number(a.featured)).filter((e) => !filter || e.system === filter).slice(0, 8));
  const keyVon = (id: string): string => ui.commands.find((c) => c.id === id)?.keysDisplay ?? "";
  let version = $state("");
  // Willkommenskarte: nur solange noch nichts eingerichtet ist; „Später“ blendet sie dauerhaft aus (Einrichtung bleibt unter Einstellungen)
  let neu = $state(false);
  // „Neues erkannt“: Emulatoren, die jetzt gefunden werden und beim letzten Lauf des Assistenten nicht
  let news = $state<{ kind: string; label: string }[]>([]);
  async function newsAntwort(mode: "seen" | "off") { try { await call("wiz_news", { mode }); news = []; } catch (e) { fail(e); } }
  let weg = $state((() => { try { return localStorage.getItem("willkommen-weg") === "1"; } catch { return false; } })());
  const spaeter = () => { weg = true; willkommen.weg = true; try { localStorage.setItem("willkommen-weg", "1"); } catch { /* ohne Speicher */ } };
  $effect(() => { void ui.ready; void currentLang(); if (!ui.ready) return;   // neu laden, wenn die Sprache wechselt (Zuletzt „vor 3 Min.“, Beispiele, Programme)
    call("recent_list").then((r) => { recent = r; loaded = true; }).catch((e) => { fail(e); loaded = true; });
    call("examples_list").then((r) => (examples = r)).catch(() => {});
    call("wiz_stand").then((r) => { neu = r.start === "first" || r.incomplete; news = r.news ?? []; }).catch(() => {});
    call("tools_list").then((r) => (tools = r)).catch(() => {});
    call("app_info").then((i) => (version = i.version)).catch(() => {});
  });
  async function pin(p: RecentProject) { try { recent = await call("recent_pin", { path: p.path }); } catch (e) { fail(e); } }
  async function reveal(p: RecentProject) { try { await call("recent_reveal", { path: p.path }); } catch (e) { fail(e); } }
  async function drop(p: RecentProject) { try { recent = await call("recent_remove", { path: p.path }); } catch (e) { fail(e); } }
  async function openFile() { try { const f = await call("vcprj_pick"); if (f) await openProject(f); } catch (e) { fail(e); } }
  const keyCmds = ["palette", "projekt_neu", "projekt_oeffnen", "bauen", "starten", "speichern", "teilen", "zen", "layout_zuruecksetzen"];
  const keys = $derived(keyCmds.map((id) => ui.commands.find((c) => c.id === id)).filter((c) => c && c.keysDisplay).map((c) => ({ id: c!.id, keys: c!.keysDisplay })));
</script>

<div class="start" aria-label={t("startui-vintagecoder")}>
  <div class="seite">
    <header class="hero">
      <h1 class="logo" data-start="logo"><Wortmarke hoehe={64} /></h1>
      <p class="dim lead">{t("startui-entwicklungsumgebung-fuer-die-6502-familie-commodore")}</p>
      {#if version}<span class="chip" data-start="version">v{version}</span>{/if}
    </header>

    <div class="aktionen">
      <button type="button" class="karte pri" aria-label={t("startui-neues-projekt")} data-start="new" onclick={() => void runCommand("projekt_neu")}><span class="ik"><Icon name="plus" size={22} /></span><span><b>{t("startui-neues-projekt")}</b><small>{t("startui-neues-sub")}</small></span>{#if keyVon("projekt_neu")}<kbd>{keyVon("projekt_neu")}</kbd>{/if}</button>
      <button type="button" class="karte" aria-label={t("startui-projekt-oeffnen")} data-start="oeffnen" onclick={pickProject}><span class="ik"><Icon name="folder" size={22} /></span><span><b>{t("startui-projekt-oeffnen")}</b><small>{t("startui-oeffnen-sub")}</small></span>{#if keyVon("projekt_oeffnen")}<kbd>{keyVon("projekt_oeffnen")}</kbd>{/if}</button>
      {#if examples.length}<button type="button" class="karte" aria-haspopup="dialog" aria-label={t("startui-beispiel-oeffnen")} data-start="beispiel" onclick={() => (allExamples = true)}><span class="ik"><Icon name="file" size={22} /></span><span><b>{t("startui-beispiel-oeffnen")}</b><small>{t("startui-beispiel-sub")}</small></span></button>{:else}<button type="button" class="karte" onclick={() => void openFile()}><span class="ik"><Icon name="file" size={22} /></span><span><b>{t("start-projektdatei")}</b></span></button>{/if}
    </div>
    <p class="mini"><button type="button" class="lnk" onclick={() => void openFile()}>{t("start-projektdatei")}</button></p>

    <section aria-label={t("startui-zuletzt-geoeffnete-projekte")} class="abschnitt">
      <h2>{t("startui-zuletzt-geoeffnete-projekte")}{#if recent.length}<span class="dim n">{t("startui-n-projekte", { n: recent.length })}</span>{/if}</h2>
      {#if loaded && recent.length === 0}<p class="dim empty" data-start="willkommen">{t("startui-noch-kein-projekt-geoeffnet-neues-projekt")}</p>{/if}
      <RecentList items={recent} onopen={(r) => void openProject(r.path)} onpin={(r) => void pin(r)} oncopy={(r) => void openCopy(r.path)} onreveal={(r) => void reveal(r)} onremove={(r) => void drop(r)} max={MAX_ZULETZT} />
    </section>

    {#if examples.length}
      <section aria-label={t("startui-beispielprojekte")} class="abschnitt" data-start="beispiele">
        <h2>{t("startui-beispielprojekte")} <span class="dim n">{t("startui-beispiele-sub")}</span></h2>
        <div class="chips" role="group" aria-label={t("startui-system-filter")}>
          <button type="button" class="fc" class:on={filter === ""} aria-pressed={filter === ""} data-start="filter" onclick={() => (filter = "")}>{t("startui-alle")}</button>
          {#each systeme as sy (sy)}<button type="button" class="fc" class:on={filter === sy} aria-pressed={filter === sy} data-start="filter" onclick={() => (filter = sy)}>{sy}</button>{/each}
        </div>
        <ul class="bsp">{#each gezeigt as e (e.path)}
          <li class="bk" data-start="beispiel-karte">
            {#if e.image}<img class="pic" src={e.image} alt="" loading="lazy" />{:else}<span class="pic ph" aria-hidden="true" style={`--h:${[...e.title].reduce((a, c) => (a * 31 + c.charCodeAt(0)) % 360, 11)}`}>{e.title.slice(0, 2).toUpperCase()}</span>{/if}
            <div class="txt"><div class="line"><b>{e.title}</b><span class="chip">{e.system}</span></div><div class="dim small">{e.description}</div></div>
            <div class="bt">{#if !e.readonly}<button type="button" class="btn sm" onclick={() => openProject(e.path)}>{t("startui-oeffnen")}</button>{/if}<button type="button" class="btn sm" onclick={() => void openCopy(e.path)}>{t("startui-kopieren")} …</button></div>
          </li>
        {/each}</ul>
        <div class="act"><button type="button" class="btn sm" onclick={() => (allExamples = true)}>{t("start-beispiele-alle")}</button></div>
      </section>
    {/if}

    {#if news.length && !neu && !wz.open}
      <section class="tafel" data-start="news" role="status" aria-label={t("startui-neues-erkannt")}>
        <h3>{t("startui-neues-erkannt")}</h3><p class="small">{t("startui-neues-text", { liste: news.map((x) => x.label).join(", ") })}</p>
        <div class="row"><button type="button" class="btn pri sm" data-start="news-einrichten" onclick={() => void wizStarten("emulators")}>{t("startui-neues-einrichten")}</button><button type="button" class="btn sm" data-start="news-merken" onclick={() => void newsAntwort("seen")}>{t("startui-neues-merken")}</button><button type="button" class="btn sm" data-start="news-aus" onclick={() => void newsAntwort("off")}>{t("startui-neues-aus")}</button></div>
      </section>
    {/if}
    <div class="unten">
      {#if neu && !weg && !wz.open}
      <section class="tafel willkommen" data-start="welcome-card" aria-label={t("startui-willkommen")}>
        <h3>{t("startui-willkommen")}</h3>
        <p class="small">{t("startui-willkommen-text")}</p>
        <div class="row"><button type="button" class="btn pri sm" data-start="welcome-setup" onclick={() => void wizStarten("quick")}>{t("startui-willkommen-einrichten")}</button><button type="button" class="btn sm" data-start="welcome-later" onclick={spaeter}>{t("startui-willkommen-spaeter")}</button></div>
      </section>
      {:else}
      <section class="tafel" aria-label={t("startui-toolchain-und-hardware")}>
        <h3>{t("startui-toolchain-und-hardware")}</h3>
        <span class="lbl">{t("startui-erkannte-emulatoren")}</span>
        <ul class="plain">{#each tools as x (x.id)}
          <li class="row"><span class="dot" class:ok={x.found} aria-hidden="true"></span><span class="mono grow" title={x.text}>{x.label}</span><span class="chip" class:okc={x.found}>{x.found ? t("startui-bereit") : t("startui-nicht-gefunden")}</span></li>
        {/each}</ul>
      </section>
      {/if}
      <section class="tafel" aria-label={t("startui-schnellreferenz")}>
        <h3>{t("startui-schnellreferenz")}</h3>
        <span class="lbl">{t("startui-tastenkuerzel")}</span>
        <ul class="plain">{#each keys as k (k.id)}<li class="row"><span class="grow">{t(`cmd-${k.id}`)}</span><kbd>{k.keys}</kbd></li>{/each}</ul>
      </section>
    </div>
    <p class="dim small fuss">{t("startui-frei-verwendbar")}</p>
  </div>
</div>

<ExamplesDialog bind:open={allExamples} {examples} onopen={(e) => void openProject(e.path)} oncopy={(e) => void openCopy(e.path)} />

<style>
  .start { flex: 1; min-height: 0; overflow: auto; padding: 24px 20px 40px; container-type: inline-size; color: var(--text); background: var(--bg); }
  .seite { max-width: 1180px; margin: 0 auto; display: flex; flex-direction: column; gap: 22px; }
  .hero { display: flex; flex-direction: column; gap: 8px; align-items: flex-start; }
  .logo { margin: 0; line-height: 0; max-width: 100%; }
  .lead { margin: 0; max-width: 620px; font-size: 16px; }
  .aktionen { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 12px; }
  .karte { display: flex; align-items: center; gap: 14px; padding: 16px 18px; border-radius: 14px; border: 1px solid var(--line2); background: var(--card); color: var(--text); text-align: left; cursor: pointer; }
  .karte:hover { background: var(--hover); } .karte.pri { background: var(--accent); border-color: var(--accent); color: #fff; } .karte.pri:hover { filter: brightness(1.08); background: var(--accent); }
  .karte b { display: block; font-size: 15px; } .karte small { display: block; font-size: 12px; opacity: 0.8; margin-top: 2px; } .karte .ik { width: 42px; height: 42px; border-radius: 10px; background: rgba(255, 255, 255, 0.14); display: flex; align-items: center; justify-content: center; flex: none; } .karte kbd { margin-left: auto; }
  .mini { margin: -14px 0 0; font-size: 12px; } .lnk { color: var(--dim); text-decoration: underline; font-size: 12px; }
  .abschnitt h2 { margin: 0 0 10px; font-size: 18px; display: flex; align-items: baseline; gap: 10px; } .n { font-size: 12px; font-weight: 400; } .empty { padding: 10px 0; }
  .chips { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 12px; } .fc { padding: 4px 14px; border-radius: 16px; border: 1px solid var(--line2); background: var(--card); color: var(--text); } .fc.on { border-color: var(--accent); background: var(--accentbg); color: var(--accent2); }
  .bsp { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(230px, 1fr)); gap: 12px; }
  .bk { display: flex; flex-direction: column; background: var(--card); border: 1px solid var(--line2); border-radius: 12px; overflow: hidden; }
  .bk .pic { width: 100%; height: 130px; object-fit: contain; background: #0b0c10; image-rendering: pixelated; display: block; } .bk .pic.ph { display: flex; align-items: center; justify-content: center; font: 700 36px var(--font-mono, monospace); color: rgba(255, 255, 255, 0.85); background: hsl(var(--h) 38% 30%); image-rendering: auto; }
  .bk .txt { padding: 10px 12px 4px; flex: 1; } .line { display: flex; justify-content: space-between; align-items: baseline; gap: 8px; } .bt { display: flex; gap: 6px; padding: 6px 12px 12px; flex-wrap: wrap; }
  .unten { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; } @container (max-width: 760px) { .unten { grid-template-columns: 1fr; } }
  .tafel { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 14px; } h3 { margin: 0 0 8px; font-size: 14px; }
  .act { display: flex; gap: 8px; flex-wrap: wrap; margin-top: 10px; }
  .plain { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 6px; }
  .row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; } .grow { flex: 1; min-width: 0; }
  .chip { display: inline-block; padding: 1px 8px; border-radius: 10px; border: 1px solid var(--line2); font-size: 11px; color: var(--dim); }
  .dot { width: 9px; height: 9px; border-radius: 50%; background: var(--faint); } .dot.ok { background: var(--ok, #2a8); }
  .dim { color: var(--dim); } .small { font-size: 11.5px; } .lbl { display: block; font-size: 10.5px; letter-spacing: 0.06em; color: var(--dim); margin: 2px 0 6px; } .fuss { margin: 0; }
  kbd { padding: 1px 6px; border: 1px solid var(--line2); border-radius: 5px; font-size: 11.5px; background: var(--raise); color: inherit; }
  .btn.sm { padding: 2px 8px; font-size: 12px; }
</style>
