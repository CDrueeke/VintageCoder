<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { adresseOeffnen, danksagungOeffnen, cfg, dateiZeigen, feedbackOeffnen, inf, infoLaden, updatePruefen, updateSpaeter, updateUeberspringen, einzeln, exportieren, geaendert, importJa, importWaehlen, kurz, ordnerWaehlen, resetJa, romsLaden, SEITEN, speichern, verwerfen, zieleLaden, type SeiteId } from "../lib/settings.svelte";
  import { wizStarten } from "../lib/wiz.svelte";
  import ArgsEditor from "../lib/components/ArgsEditor.svelte";
  import Wortmarke from "../lib/components/Wortmarke.svelte";
  import Ziele from "./einst/Ziele.svelte";
  import Roms from "./einst/Roms.svelte";
  import Dialog from "../lib/components/Dialog.svelte";
  import Segment from "../lib/components/Segment.svelte";
  import Switch from "../lib/components/Switch.svelte";
  import { ed, fontStep, setPrettyLine, toggleWrap } from "../lib/editor.svelte";
  import { setKeyHints, setLanguage, setSingleKeys, setSplash, setTheme, setZoom, ui, type Theme } from "../lib/state.svelte";
  import { t } from "../lib/i18n.svelte";
  import type { LangChoice } from "../lib/api";
  // Einstellungen (E12.1): links die Seiten, rechts der Inhalt als Karten. Aussehen und Editor wirken sofort; die übrigen Werte sind ein Entwurf, der mit „Speichern“ in die Einstellungsdatei geht.
  const d = $derived(cfg.draft);
  const titel = (s: SeiteId) => t(`einst-seite-${s}`);
  $effect(() => { if (cfg.draft && cfg.page === "ziele") void zieleLaden(); });
  $effect(() => { if (cfg.draft && cfg.page === "roms") void romsLaden(); });
  $effect(() => { if (cfg.draft && cfg.page === "info") void infoLaden(); });
  let impOpen = $state(false); let resetOpen = $state(false);
  $effect(() => { impOpen = cfg.imp !== null; }); $effect(() => { if (!impOpen && cfg.imp) cfg.imp = null; });
  $effect(() => { resetOpen = cfg.reset; }); $effect(() => { if (!resetOpen && cfg.reset) cfg.reset = false; });
  const sprachen = $derived([{ id: "auto", label: t("einst-sprache-auto") }, { id: "de", label: "Deutsch" }, { id: "en", label: "English" }]);
  const schemata = $derived([{ id: "system", label: t("einst-schema-system") }, { id: "dark", label: t("einst-schema-dunkel") }, { id: "light", label: t("einst-schema-hell") }, { id: "contrast", label: t("einst-schema-kontrast") }]);
  const updates = $derived([{ id: "", label: t("einst-upd-fragen") }, { id: "never", label: t("einst-upd-nie") }, { id: "daily", label: t("einst-upd-taeglich") }, { id: "weekly", label: t("einst-upd-woechentlich") }]);
  const bilder = $derived([{ id: "", label: t("einst-bilder-fragen") }, { id: "always", label: t("einst-bilder-immer") }, { id: "never", label: t("einst-bilder-nie") }]);
  const befehle = $derived.by(() => { const q = cfg.suche.trim().toLowerCase(); return ui.commands.filter((c) => c.keysDisplay || c.keys).map((c) => ({ id: c.id, name: t(`cmd-${c.id}`), keys: c.keysDisplay || c.keys })).filter((c) => !q || c.name.toLowerCase().includes(q) || c.keys.toLowerCase().includes(q)).sort((a, b) => a.name.localeCompare(b.name)); });
  const zoomProzent = $derived(Math.round(ui.zoom * 100));
  function reset() { setSingleKeys(true); setKeyHints(true); setSplash(true); setTheme("system"); setZoom(1); fontStep(13 - ed.fontSize); if (!ed.wrap) toggleWrap(); setPrettyLine(true); cfg.msg = t("einst-darstellung-zurueck"); }
</script>

<div class="es" aria-label={t("einst-titel")}>
  <nav class="col l" aria-label={t("einst-seiten")}>
    <h3>{t("einst-titel")}</h3>
    <ul class="nav" role="list">{#each SEITEN as s (s)}<li><button type="button" class="nb" class:on={cfg.page === s} aria-current={cfg.page === s ? "page" : undefined} data-es="pagebtn" data-s={s} onclick={() => { cfg.page = s; cfg.msg = ""; }}>{titel(s)}</button></li>{/each}</ul>
  </nav>

  <section class="col m" aria-label={titel(cfg.page)} data-es={`page-${cfg.page}`}>
    <h2>{titel(cfg.page)}</h2>
    {#if !d}<p class="dim">{t("einst-laedt")}</p>
    {:else if cfg.page === "allgemein"}
      <div class="card"><h3>{t("einst-sprache")}</h3><Segment small label={t("einst-sprache")} items={sprachen} value={ui.choice === "xx" ? "auto" : ui.choice} onselect={(id) => void setLanguage(id as LangChoice)} /><p class="small dim">{t("einst-sprache-hinweis")}</p></div>
      <div class="card"><h3>{t("einst-updates")}</h3><Segment small label={t("einst-updates")} items={updates} value={d.updates?.check ?? ""} onselect={(id) => { d.updates = { ...(d.updates ?? {}), check: id }; }} />
        <label class="small dim">{t("einst-upd-url")}<input class="inp" data-es="upd-url" bind:value={d.updates.manifest_url} placeholder={t("einst-upd-url-leer")} /></label>
        <p class="small dim">{t("einst-upd-hinweis")}</p></div>
      <div class="card"><h3>{t("einst-startbild")}</h3><Switch checked={ui.splash} label={t("einst-startbild-an")} onchange={(on) => setSplash(on)} /></div>
      <div class="card"><h3>{t("einst-hilfe")}</h3><Segment small label={t("einst-hilfe-bilder")} items={bilder} value={d.help?.images ?? ""} onselect={(id) => { d.help = { ...(d.help ?? {}), images: id }; }} />
        <label class="small dim">{t("einst-hilfe-url")}<input class="inp" data-es="hilfe-url" value={d.help?.images_url ?? ""} oninput={(e) => { d.help = { ...(d.help ?? {}), images_url: e.currentTarget.value }; }} placeholder={t("einst-hilfe-url-leer")} /></label></div>
      <div class="card"><h3>{t("einst-projektordner")}</h3>
        <div class="row"><input class="inp" data-es="projekte" aria-label={t("einst-projektordner")} value={kurz(d.projects_dir || "")} placeholder={kurz(cfg.projekte)} onchange={(e) => { d.projects_dir = e.currentTarget.value.replace(/^~/, cfg.home ?? "~"); }} /><button type="button" class="btn sm" data-es="folder" onclick={() => void ordnerWaehlen()}>{t("einst-waehlen")}</button></div>
        <p class="small dim">{t("einst-projektordner-hinweis")}</p></div>
      <div class="card"><h3>{t("wiz-einrichtung")}</h3><p class="small dim">{t("wiz-einrichtung-text")}</p>
        <div class="row"><button type="button" class="btn sm" data-es="wiz-schnell" onclick={() => void wizStarten("quick")}>{t("wiz-schnell")}</button><button type="button" class="btn sm" data-es="wiz-voll" onclick={() => void wizStarten("full")}>{t("wiz-voll")}</button></div>
        <div class="row"><button type="button" class="btn sm" onclick={() => void wizStarten("roms")}>{t("wiz-teil-roms")}</button><button type="button" class="btn sm" onclick={() => void wizStarten("emulators")}>{t("wiz-teil-emulatoren")}</button><button type="button" class="btn sm" onclick={() => void wizStarten("hardware")}>{t("wiz-teil-hardware")}</button><button type="button" class="btn sm" onclick={() => void wizStarten("design")}>{t("wiz-teil-design")}</button></div></div>
      <div class="card"><h3>{t("einst-fehlersuche")}</h3><Switch checked={!!d.debug_log} label={t("einst-debuglog")} onchange={(on) => void einzeln((c) => { c.debug_log = on; })} /><p class="small dim">{t("einst-debuglog-hinweis")}</p></div>
      <div class="card"><h3>{t("einst-datei")}</h3><p class="mono small" data-es="file">{cfg.file ? kurz(cfg.file) : "–"}</p><button type="button" class="btn sm" data-es="file-show" onclick={() => void dateiZeigen()}>{t("einst-datei-zeigen")}</button></div>
    {:else if cfg.page === "ziele"}<Ziele />
    {:else if cfg.page === "roms"}<Roms />
    {:else if cfg.page === "aussehen"}
      <div class="card"><h3>{t("einst-schema")}</h3><Segment small label={t("einst-schema")} items={schemata} value={ui.theme} onselect={(id) => setTheme(id as Theme)} /></div>
      <div class="card"><h3>{t("einst-zoom")}</h3><div class="row"><button type="button" class="ibtn" aria-label={t("einst-zoom-kleiner")} data-es="zoom-minus" onclick={() => setZoom(ui.zoom - 0.1)}>−</button><span class="mono" data-es="zoom">{zoomProzent} %</span><button type="button" class="ibtn" aria-label={t("einst-zoom-groesser")} data-es="zoom-plus" onclick={() => setZoom(ui.zoom + 0.1)}>+</button><button type="button" class="btn sm" data-es="zoom-null" onclick={() => setZoom(1)}>100 %</button></div><p class="small dim">{t("einst-zoom-hinweis")}</p></div>
      <div class="card"><button type="button" class="btn sm" data-es="darstellung-zurueck" onclick={reset}>{t("einst-darstellung-zuruecksetzen")}</button></div>
    {:else if cfg.page === "editor"}
      <div class="card"><h3>{t("einst-schrift")}</h3><div class="row"><button type="button" class="ibtn" aria-label={t("einst-schrift-kleiner")} data-es="schrift-minus" onclick={() => fontStep(-1)}>−</button><span class="mono" data-es="schrift">{ed.fontSize} pt</span><button type="button" class="ibtn" aria-label={t("einst-schrift-groesser")} data-es="schrift-plus" onclick={() => fontStep(1)}>+</button></div></div>
      <div class="card"><Switch checked={ed.wrap} label={t("einst-umbruch")} onchange={() => toggleWrap()} /><Switch checked={ed.prettyLine} label={t("einst-pretty")} onchange={(on) => setPrettyLine(on)} /><p class="small dim">{t("einst-pretty-hinweis")}</p></div>
      <div class="card"><h3>{t("einst-menueleiste")}</h3><p class="small dim">{cfg.mac ? t("einst-menueleiste-mac") : t("einst-menueleiste-win")}</p></div>
    {:else if cfg.page === "toolchain"}
      <div class="card"><h3>{t("einst-assembler")}</h3><p>{t("einst-assembler-eingebaut")}</p>
        <ArgsEditor kind="asm" owner={d.assembler} titel={t("einst-assembler-optionen")} />
        <p class="small dim">{t("einst-include-suche")}</p></div>
      <div class="card"><h3>{t("einst-basic-dialekt")}</h3><p class="small dim">{t("einst-basic-hinweis")}</p>
        <ul class="kv" data-es="dialekte">{#each cfg.dialects as x (x.id)}<li><span>{x.name}</span><b>{x.dialect ?? "–"}</b></li>{/each}</ul></div>
    {:else if cfg.page === "kuerzel"}
      <div class="card"><h3>{t("einst-tasten")}</h3>
        <Switch checked={ui.singleKeys} label={t("einst-einzeltasten")} onchange={(on) => setSingleKeys(on)} /><p class="small dim">{t("einst-einzeltasten-hinweis")}</p>
        <Switch checked={ui.keyHints} label={t("einst-tasten-klaviatur")} onchange={(on) => setKeyHints(on)} /><p class="small dim">{t("einst-tastenlayout-hinweis")}</p></div>
      <div class="card"><h3>{t("einst-kuerzel")}</h3><input class="inp" data-es="kuerzel-suche" placeholder={t("einst-kuerzel-suche")} aria-label={t("einst-kuerzel-suche")} bind:value={cfg.suche} />
        <p class="small dim">{t("einst-kuerzel-fest")}</p>
        <ul class="kv" data-es="kuerzel">{#each befehle as c (c.id)}<li><span>{c.name}</span><b class="mono">{c.keys}</b></li>{:else}<li class="dim">{t("einst-kuerzel-keine")}</li>{/each}</ul></div>
    {:else if cfg.page === "info"}
      <div class="card" data-es="about"><h3>{t("info-ueber")}</h3>
        <p class="uebername" data-info="name"><Wortmarke hoehe={30} titel={inf.about?.name ?? "VintageCoder"} /> <span class="mono" data-info="version">{inf.about?.version}</span></p>
        <p class="small dim" data-info="copyright">{t("about-copyright")}</p><p class="small dim">{t("about-lizenz-satz")} {t("about-gewaehrleistung")}</p>
        <div class="row"><button type="button" class="btn sm" data-info="web" onclick={() => void adresseOeffnen("web")}>{t("info-webseite")}</button><button type="button" class="btn sm" data-info="repo" onclick={() => void adresseOeffnen("repo")}>{t("info-quelltext")}</button><button type="button" class="btn sm" data-info="feedback" onclick={() => void feedbackOeffnen()}>{t("info-feedback")} …</button></div>
        <p class="small dim">{t("info-kein-netz")}</p></div>
      {#if inf.about?.credits?.length}
        <div class="card" data-es="danksagungen"><h3>{t("about-danksagungen")}</h3><p class="small dim">{t("about-danksagungen-hinweis")}</p>
          <ul class="kv credits">{#each inf.about.credits as c (c.id)}
            <li data-credit={c.id}><div><b>{c.name}</b>{#if c.license} <span class="small dim">· {c.license}</span>{/if}</div><div class="small dim">{t(`credit-${c.id}`)}</div>
              {#if c.links.length}<div class="small">{#each c.links as l, i (l.url)}<a href={l.url} class="lnk" data-credit-link title={l.url} aria-label={t("about-link-oeffnen", { name: c.name, url: l.label })} onclick={(e) => { e.preventDefault(); void danksagungOeffnen(c.id, i); }}>{l.label}</a>{#if i < c.links.length - 1}<span class="dim"> · </span>{/if}{/each}</div>{/if}</li>
          {/each}</ul></div>
      {/if}
      <div class="card" data-es="update"><h3>{t("einst-updates")}</h3>
        <p class="small" data-info="state">{inf.upd?.lastCheck ? t("info-letzte-pruefung", { t: inf.upd.lastCheck, r: inf.upd.lastResult }) : t("info-nie-geprueft")}</p>
        <div class="row"><button type="button" class="btn sm" data-info="check" disabled={inf.busy || inf.upd?.enabled === false} onclick={() => void updatePruefen()}>{inf.busy ? t("info-prueft") : t("info-jetzt-pruefen")}</button></div>
        {#if inf.fehler}<p class="small warn" role="alert" data-info="error">{inf.fehler}</p>{/if}
        {#if inf.upd?.eval?.update}{@const u = inf.upd.eval.update}
          <div class="fresh" data-info="fresh"><p><b>{t("info-neue-version", { v: u.version })}</b> <span class="small dim">{u.published}</span></p>
            {#if u.formatWarning}<p class="small warn">{t("info-format-warnung")}</p>{/if}
            <pre class="notes small" data-info="notes">{u.notes}</pre>
            <div class="row"><button type="button" class="btn sm" data-info="view" onclick={() => void adresseOeffnen("release", u.version)}>{t("info-aenderungen")}</button><button type="button" class="btn sm" data-info="later" onclick={() => void updateSpaeter()}>{t("info-spaeter")}</button><button type="button" class="btn sm" data-info="skip" onclick={() => void updateUeberspringen(u.version)}>{t("info-ueberspringen")}</button></div></div>
        {:else if inf.pruefte && !inf.fehler}<p class="small ok" data-info="current">{t("info-aktuell")}</p>{/if}
        {#if inf.upd?.eval?.yanked}<p class="small warn">{t("info-zurueckgezogen")}</p>{/if}
        <p class="small dim">{t("info-update-hinweis-text")}</p></div>
      <div class="card"><h3>{t("info-drittanbieter")}</h3><button type="button" class="btn sm" data-info="notices" aria-expanded={inf.hinweise} onclick={() => (inf.hinweise = !inf.hinweise)}>{inf.hinweise ? t("info-hinweise-zu") : t("info-hinweise-auf")}</button>{#if inf.hinweise}<pre class="notes small" data-info="notices-text">{inf.about?.notices}</pre>{/if}</div>
    {:else if cfg.page === "sicherung"}
      <div class="card"><h3>{t("konfig-titel")}</h3><p class="small dim">{t("konfig-hinweis")}</p>
        <div class="row"><button type="button" class="btn sm" data-es="export" onclick={() => void exportieren()}>{t("konfig-exportieren")}</button><button type="button" class="btn sm" data-es="import" onclick={() => void importWaehlen()}>{t("konfig-importieren")}</button><button type="button" class="btn sm" data-es="reset" onclick={() => (cfg.reset = true)}>{t("konfig-zuruecksetzen")}</button></div></div>
    {/if}
    {#if cfg.msg}<p class="small ok" role="status" data-es="msg">{cfg.msg}</p>{/if}
    {#if d && (cfg.page === "allgemein" || cfg.page === "toolchain" || cfg.page === "ziele" || cfg.page === "roms")}
      <div class="row foot"><button type="button" class="btn pri" data-es="save" disabled={!geaendert() || cfg.busy} onclick={() => void speichern()}>{t("einst-speichern")}</button><button type="button" class="btn" data-es="discard" disabled={!geaendert()} onclick={verwerfen}>{t("einst-verwerfen")}</button>{#if geaendert()}<span class="small warn">{t("einst-ungespeichert")}</span>{/if}</div>
    {/if}
  </section>
</div>

{#if impOpen && cfg.imp}
  <Dialog bind:open={impOpen} title={t("konfig-importieren")}>
    {#snippet children()}<p>{t("konfig-import-frage", { z: cfg.imp?.targets ?? 0, m: cfg.imp?.machines ?? 0 })}</p><p class="small dim">{t("konfig-passwoerter-bleiben")}</p>{/snippet}
    {#snippet footer()}<button type="button" class="btn" onclick={() => (cfg.imp = null)}>{t("dialog-abbrechen")}</button><button type="button" class="btn pri" data-es="import-ja" onclick={() => void importJa()}>{t("konfig-uebernehmen")}</button>{/snippet}
  </Dialog>
{/if}
{#if resetOpen}
  <Dialog bind:open={resetOpen} title={t("konfig-zuruecksetzen")}>
    {#snippet children()}<p>{t("konfig-reset-frage")}</p>{/snippet}
    {#snippet footer()}<button type="button" class="btn" onclick={() => (cfg.reset = false)}>{t("dialog-abbrechen")}</button><button type="button" class="btn pri" data-es="reset-ja" onclick={() => void resetJa()}>{t("konfig-zuruecksetzen")}</button>{/snippet}
  </Dialog>
{/if}

<style>
  .uebername { display: flex; align-items: center; gap: 12px; margin-bottom: 4px; }
  .credits { display: flex; flex-direction: column; gap: 8px; } .credits li { display: flex; flex-direction: column; gap: 2px; padding-bottom: 6px; border-bottom: 1px solid var(--line); } .credits li:last-child { border-bottom: 0; } .credits .lnk { color: var(--accent2); text-decoration: underline; cursor: pointer; }
  .es { flex: 1; min-height: 0; display: grid; grid-template-columns: 220px minmax(0, 1fr); gap: 10px; padding: 10px; overflow: auto; } @media (max-width: 760px) { .es { grid-template-columns: 1fr; } }
  .col { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 10px; min-height: 0; overflow: auto; }
  h2 { margin: 0; font-size: 18px; } h3 { margin: 0 0 6px; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); }
  .nav { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 2px; } .nb { width: 100%; text-align: left; padding: 7px 10px; border-radius: 8px; color: var(--text); } .nb:hover { background: var(--hover); } .nb.on { background: var(--accentbg); color: var(--accent2); }
  .card { flex: none; background: var(--raise); border-radius: 10px; padding: 12px; display: flex; flex-direction: column; gap: 8px; max-width: 720px; } .card p { margin: 0; }
  .card > .btn { align-self: flex-start; } .row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; } .row .inp { flex: 1; min-width: 200px; } label { display: flex; flex-direction: column; gap: 4px; } .inp { width: 100%; }
  .small { font-size: 12px; } .dim { color: var(--dim); } .warn { color: var(--warn); } .ok { color: var(--ok); } .foot { margin-top: auto; padding: 8px 0; position: sticky; bottom: -12px; background: var(--card); z-index: 1; }
  .notes { white-space: pre-wrap; max-height: 320px; overflow: auto; background: var(--card); border-radius: 8px; padding: 8px; margin: 0; font-family: var(--font-mono, monospace); } .fresh { border-top: 1px solid var(--line2); padding-top: 8px; display: flex; flex-direction: column; gap: 6px; }
  .kv { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 2px; } .kv li { display: flex; justify-content: space-between; gap: 10px; padding: 4px 0; border-bottom: 1px solid var(--line2); font-size: 13px; }
</style>
