<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { dropTarget } from "../dnd.svelte";
  import { EditorView } from "@codemirror/view";
  import { onDestroy, onMount } from "svelte";
  import { ui } from "../state.svelte";
  import type { Group } from "../groups";
  import { attachView, ed, focusGroup, lspTransactions, moveToGroup, onState, refresh, scrollMerken, scrollHolen, splitOn, stateOf } from "../editor.svelte";
  import { t } from "../i18n.svelte";
  import MdPreview from "./MdPreview.svelte";
  import { istMarkdown, md, mdBefehl, vorschauAn } from "../markdown.svelte";
  let { group = 0 }: { group?: Group } = $props();
  // Ein CodeMirror-Fenster für alle Reiter: beim Wechsel wird der Zustand des Reiters (Text, Verlauf, Auswahl, Faltungen) eingesetzt
  let host: HTMLDivElement;
  let view: EditorView | null = null;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let shown: string | null = null;
  let prev: string | null = null;
  let over = $state<"here" | "edge" | null>(null);
  // Markdown-Dateien: Vorschau oder Quelltext (je Reiter), im Quelltext eine Leiste mit den Schreibhilfen
  const tab = $derived(ed.tabs.find((x) => x.rel === ed.activeBy[group]));
  const mdTab = $derived(tab && !tab.scratch ? istMarkdown(tab.rel, tab.lang) : false);
  const preview = $derived(!!tab && mdTab && vorschauAn(tab.rel, tab.lang));
  const FORMAT: [string, string][] = [["md_fett", "B"], ["md_kursiv", "I"], ["md_durchgestrichen", "S"], ["md_code", "</>"], ["md_link", "🔗"], ["md_bild", "▣"], ["md_tiefer", "H+"], ["md_hoeher", "H−"], ["md_aufzaehlung", "•"], ["md_nummerierung", "1."], ["md_aufgaben", "☑"], ["md_zitat", "❝"], ["md_tabelle", "▦"]];

  onMount(() => {
    view = new EditorView({
      parent: host,
      dispatchTransactions(trs, v) {
        v.update(trs);
        if (shown) { onState(shown, v.state); lspTransactions(shown, trs); }
        if (trs.some((x) => x.docChanged) && shown) { const rel = shown; clearTimeout(timer); timer = setTimeout(() => void refresh(rel), 80); }
      },
    });
    attachView(group, view);
    view.scrollDOM.addEventListener("scroll", () => { if (view && shown && view.scrollDOM.scrollHeight > 0 && view.scrollDOM.clientHeight > 0) scrollMerken(shown, view.scrollDOM.scrollTop); }, { passive: true });   // laufend merken: beim Bereichswechsel ist das Fenster schon verborgen, wenn es abgebaut wird
    show();
  });
  onDestroy(() => { clearTimeout(timer); attachView(group, null); view?.destroy(); });

  function show() {
    if (!view) return;
    const rel = ed.activeBy[group]; shown = rel;
    if (!rel) { return; }
    const st = stateOf(rel); if (!st) return;
    if (prev && prev !== rel && view.scrollDOM.clientHeight > 0) scrollMerken(prev, view.scrollDOM.scrollTop);   // Bildlauf des Reiters merken, den wir verlassen
    prev = rel;
    view.setState(st);
    { const sn = scrollHolen(rel); const v0 = view; if (sn !== undefined && sn > 0) { let k = 0; const setze = () => { if (shown !== rel) return; v0.scrollDOM.scrollTop = sn; if (Math.abs(v0.scrollDOM.scrollTop - sn) > 2 && ++k < 12) setTimeout(setze, 50); }; requestAnimationFrame(setze); } }   // Bildlauf des Reiters wiederherstellen
    if (ed.focus === group) onState(rel, st); void refresh(rel); if (ed.focus === group) view.focus();
  }
  $effect(() => { void ed.nonces[group]; show(); });
  $effect(() => { void ed.fontSize; view?.requestMeasure(); });
  // Wenn ein Dialog oder die Befehlspalette schließt und danach nichts anderes den Fokus hat, gehört er dem Editor (sofort eingabebereit, z. B. nach „Gehe zu Zeile“)
  let wasOpen = false;
  $effect(() => {
    const open = !!ed.prompt || !!ed.closing || !!ed.leaving || ui.palette;
    if (wasOpen && !open) setTimeout(() => { const a = document.activeElement; if (view && (!a || a === document.body)) view.focus(); }, 40);
    wasOpen = open;
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="pane" onfocusin={() => focusGroup(group)} onpointerdown={() => focusGroup(group)}>
  <div class="col">
    {#if tab && mdTab}
      <div class="mdbar" role="toolbar" aria-label={t("md-leiste")} data-md="leiste">
        <div class="seg" role="group" aria-label={t("md-ansicht")}><button type="button" data-md="vorschau-an" aria-pressed={preview} onclick={() => (md.quelltext[tab!.rel] = false)}>{t("md-vorschau")}</button><button type="button" data-md="quelltext-an" aria-pressed={!preview} onclick={() => (md.quelltext[tab!.rel] = true)}>{t("md-quelltext")}</button></div>
        {#if !preview}{#each FORMAT as [id, zeichen] (id)}<button type="button" class="fb" data-md="format" data-id={id} title={t(`cmd-${id}`)} aria-label={t(`cmd-${id}`)} onclick={() => void mdBefehl(id)}>{zeichen}</button>{/each}{/if}
      </div>
    {/if}
    <div class="cmhost" class:weg={preview} bind:this={host}></div>
    {#if preview && tab}<MdPreview rel={tab.rel} {group} />{/if}
  </div>
  {#if ed.dragRel}
    <!-- Ablagezonen beim Ziehen eines Reiters: in diese Hälfte, oder (ungeteilt) an den rechten Rand zum Teilen -->
    <div class="drop" aria-hidden="true">
      <div class="z" class:on={over === "here"} use:dropTarget={() => ({ accept: (p) => p.kind === "tab", enter: () => (over = "here"), leave: () => (over = null), drop: (p) => { over = null; moveToGroup(p.id, group); } })} role="presentation">{#if splitOn()}{t("split-hierher")}{/if}</div>
      {#if !splitOn()}<div class="z edge" class:on={over === "edge"} use:dropTarget={() => ({ accept: (p) => p.kind === "tab", enter: () => (over = "edge"), leave: () => (over = null), drop: (p) => { over = null; moveToGroup(p.id, 1); } })} role="presentation">{t("split-rechts")}</div>{/if}
    </div>
  {/if}
</div>

<style>
  .pane { flex: 1; min-height: 0; min-width: 0; display: flex; position: relative; }
  .drop { position: absolute; inset: 0; display: flex; z-index: 5; }
  .z { flex: 1; display: grid; place-items: center; font-size: 13px; color: var(--dim); background: color-mix(in srgb, var(--card) 70%, transparent); border: 2px dashed transparent; }
  .z.edge { flex: 0 0 30%; border-left: 2px dashed var(--line2); }
  .z.on { background: var(--accentbg); border-color: var(--accent); color: var(--text); }
  .col { flex: 1; min-width: 0; min-height: 0; display: flex; flex-direction: column; }
  .mdbar { display: flex; gap: 4px; align-items: center; flex-wrap: wrap; padding: 4px 8px; border-bottom: 1px solid var(--line2); background: var(--card); } .mdbar .seg { display: flex; gap: 2px; margin-right: 8px; }
  .mdbar button { height: 26px; min-width: 28px; padding: 0 8px; border: 1px solid var(--line2); border-radius: 6px; background: var(--raise); color: var(--text); font: inherit; font-size: 12px; cursor: pointer; } .mdbar button[aria-pressed="true"] { background: var(--accentbg); border-color: var(--accent); }
  .cmhost.weg { display: none; }
  .cmhost { flex: 1; min-height: 0; min-width: 0; display: flex; }
  .cmhost :global(.cm-editor) { flex: 1; min-width: 0; }
</style>
