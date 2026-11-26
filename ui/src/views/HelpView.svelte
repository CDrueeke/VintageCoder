<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { renderHilfe } from "../lib/hilfemd";
  import { bildLaden, bilderErlauben, hilfeOeffnen, hp, suchen, thema, zurueck } from "../lib/hilfe.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import { ui } from "../lib/state.svelte";
  import { t } from "../lib/i18n.svelte";
  // Hilfe (E13): links Suche und Themen nach Gruppen (Handbuch zuerst), rechts das Thema als lesbarer Text; Links zwischen Themen wirken im selben Fenster, „Zurück“ geht den Weg zurück.
  // Handbuchbilder kommen aus dem Zwischenspeicher; aus dem Netz nur nach deiner Zustimmung.
  let box = $state<HTMLElement | null>(null); let frage = $state(false); let einmal = $state(false);
  const gruppen = $derived.by(() => { const g: { name: string; items: typeof hp.themen }[] = []; for (const x of hp.themen) { let e = g.find((y) => y.name === x.group); if (!e) { e = { name: x.group, items: [] }; g.push(e); } e.items.push(x); } return g; });
  $effect(() => { if (!hp.themen.length) void hilfeOeffnen(); });
  $effect(() => {
    const doc = hp.doc; const b = box; void einmal; if (!b) return; b.replaceChildren(); frage = false; if (!doc) return;
    b.appendChild(renderHilfe(doc.markdown, {
      thema: (id) => void thema(id),
      bild: (id, alt, el) => { void bildLaden(id, einmal).then((r) => { if (r.state === "ok" && r.img) { const img = document.createElement("img"); img.src = r.img; img.alt = alt; el.replaceChildren(img); el.classList.add("da"); } else { el.classList.add("missing"); if (r.state === "ask") frage = true; else if (r.state === "error") el.title = r.error ?? ""; } }); },
    }));
  });
  /** Hilfe schließen: zurück in den Bereich, aus dem sie geöffnet wurde (der Bereich bleibt in `ui.mode` erhalten) */
  function schliessen() { ui.extra = null; }
  function taste(e: KeyboardEvent) { if (e.key !== "Escape" || e.defaultPrevented) return; if (hp.suche.trim()) { suchen(""); e.preventDefault(); return; } e.preventDefault(); schliessen(); }
  async function erlauben(immer: boolean) { frage = false; await bilderErlauben(immer); einmal = !einmal; }
</script>

<svelte:window onkeydown={taste} />
<div class="hv" aria-label={t("view-title-hilfe")}>
  <aside class="col l" aria-label={t("hilfe-themen")}>
    <div class="kopf"><button type="button" class="ibtn" data-hp="schliessen" aria-label={t("hilfe-schliessen")} title={t("hilfe-schliessen")} onclick={schliessen}><Icon name="x" size={16} /></button><span class="grow small dim">{t("view-title-hilfe")}</span></div>
    <input class="inp" type="search" data-hp="suche" aria-label={t("hilfe-suche")} placeholder={t("hilfe-suche")} value={hp.suche} oninput={(e) => suchen(e.currentTarget.value)} />
    {#if hp.suche.trim()}
      <ul class="liste" data-hp="treffer">{#each hp.treffer as r (r.id)}<li><button type="button" class="tb" data-hp="treffer-eintrag" onclick={() => void thema(r.id)}><b>{r.title}</b><span class="small dim">{r.snippet}</span></button></li>{:else}<li class="small dim">{t("hilfe-nichts-gefunden")}</li>{/each}</ul>
    {:else}
      {#each gruppen as g (g.name)}<h3>{g.name}</h3><ul class="liste" data-hp="themen">{#each g.items as x (x.id)}<li><button type="button" class="tb" class:on={hp.doc?.id === x.id} aria-current={hp.doc?.id === x.id ? "page" : undefined} data-hp="thema" data-id={x.id} onclick={() => void thema(x.id)}>{x.title}</button></li>{/each}</ul>{/each}
    {/if}
  </aside>
  <section class="col m" aria-label={hp.doc?.title ?? t("view-title-hilfe")}>
    <div class="bar">{#if hp.doc}<button type="button" class="btn sm" data-hp="zurueck" onclick={() => void zurueck()}><Icon name="arrl" size={14} /> {t("hilfe-zurueck")}</button>{/if}</div>
    {#if frage}<div class="frage" role="group" aria-label={t("hilfe-bilder-frage")} data-hp="bilder-frage"><p class="small">{t("hilfe-bilder-text")}</p><div class="row"><button type="button" class="btn sm" data-hp="bild-einmal" onclick={() => { frage = false; einmal = true; }}>{t("hilfe-bilder-einmal")}</button><button type="button" class="btn sm" data-hp="bild-immer" onclick={() => void erlauben(true)}>{t("hilfe-bilder-immer")}</button><button type="button" class="btn sm" data-hp="bild-nie" onclick={() => void erlauben(false)}>{t("hilfe-bilder-nie")}</button></div></div>{/if}
    {#if hp.nichts}<p class="dim" role="status" data-hp="nichts-da">{t("hilfe-kein-eintrag", { w: hp.nichts })}</p>{/if}
    {#if !hp.doc && !hp.nichts}<div class="blank"><Icon name="search" size={26} /><h2>{t("hilfe-willkommen")}</h2><p>{t("hilfe-willkommen-text")}</p></div>{/if}
    <article class="doc" data-hp="doc" bind:this={box}></article>
  </section>
</div>

<style>
  .kopf { display: flex; align-items: center; gap: 6px; } .kopf .grow { flex: 1; }
  .hv { flex: 1; min-height: 0; display: grid; grid-template-columns: 280px minmax(0, 1fr); gap: 10px; padding: 10px; overflow: auto; } @media (max-width: 760px) { .hv { grid-template-columns: 1fr; } }
  .col { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 8px; min-height: 0; overflow: auto; }
  h3 { margin: 8px 0 2px; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); } .liste { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 1px; }
  .tb { height: auto; min-height: 0; flex: none; line-height: 1.35; display: flex; flex-direction: column; align-items: flex-start; width: 100%; text-align: left; padding: 5px 8px; border-radius: 8px; color: var(--text); gap: 2px; } .tb > b { display: block; max-width: 100%; } .tb > span { display: -webkit-box; -webkit-line-clamp: 3; line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; overflow-wrap: anywhere; max-width: 100%; white-space: normal; } .tb:hover { background: var(--hover); } .tb.on { background: var(--accentbg); color: var(--accent2); }
  .bar { display: flex; gap: 8px; } .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); } .row { display: flex; gap: 8px; flex-wrap: wrap; } .inp { width: 100%; }
  .blank { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; color: var(--dim); text-align: center; padding: 30px; flex: 1; } .blank h2 { color: var(--text); font-size: 18px; } .blank p { max-width: 420px; }
  .frage { background: var(--raise); border-radius: 10px; padding: 10px; display: flex; flex-direction: column; gap: 6px; max-width: 760px; }
  .doc { max-width: 760px; line-height: 1.55; }
  .doc :global(h2) { font-size: 22px; margin: 4px 0 10px; } .doc :global(h3) { font-size: 17px; margin: 18px 0 6px; text-transform: none; letter-spacing: 0; color: var(--text); } .doc :global(h4), .doc :global(h5) { font-size: 14px; margin: 14px 0 4px; text-transform: none; letter-spacing: 0; color: var(--text); }
  .doc :global(p) { margin: 0 0 10px; } .doc :global(ul), .doc :global(ol) { margin: 0 0 10px; padding-left: 22px; } .doc :global(li) { margin: 2px 0; }
  .doc :global(code) { font-family: var(--font-mono, monospace); background: var(--raise); padding: 1px 5px; border-radius: 4px; font-size: 0.92em; } .doc :global(pre) { background: var(--raise); border-radius: 8px; padding: 10px; overflow: auto; margin: 0 0 12px; } .doc :global(pre code) { background: none; padding: 0; }
  .doc :global(table) { border-collapse: collapse; margin: 0 0 12px; font-size: 13px; display: block; overflow: auto; } .doc :global(th), .doc :global(td) { border: 1px solid var(--line2); padding: 4px 8px; text-align: left; vertical-align: top; } .doc :global(th) { background: var(--raise); }
  .doc :global(blockquote) { margin: 0 0 10px; padding: 6px 12px; border-left: 3px solid var(--accent); background: var(--raise); border-radius: 0 8px 8px 0; } .doc :global(hr) { border: 0; border-top: 1px solid var(--line2); margin: 14px 0; }
  .doc :global(a.hl) { color: var(--accent2); text-decoration: underline; } .doc :global(img) { max-width: 100%; border-radius: 8px; border: 1px solid var(--line2); display: block; margin: 8px 0; } .doc :global(.bild.missing) { color: var(--dim); font-style: italic; }
</style>
