<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { SearchHit } from "../api";
  import { jumpTo } from "../editor.svelte";
  import { t } from "../i18n.svelte";
  import { cancelSearch, search, searchSoon, startSearch } from "../suche.svelte";
  // Suche im Projekt: Eingabe, Treffer nach Datei, Abbrechen; läuft im Hintergrund, ungespeicherte Texte zählen mit
  let input = $state<HTMLInputElement | undefined>();
  $effect(() => { void search.focus; queueMicrotask(() => input?.focus()); });
  const groups = $derived.by(() => { const out: { rel: string; hits: SearchHit[] }[] = []; for (const h of search.hits) { const g = out[out.length - 1]; if (g && g.rel === h.rel) g.hits.push(h); else out.push({ rel: h.rel, hits: [h] }); } return out; });
  function key(e: KeyboardEvent) { if (e.key === "Enter") { e.preventDefault(); void startSearch(); } else if (e.key === "Escape" && search.running) { e.preventDefault(); void cancelSearch(); } }
</script>

<div class="s">
  <div class="bar">
    <input bind:this={input} type="search" bind:value={search.text} placeholder={t("search-placeholder")} aria-label={t("search-label")} oninput={searchSoon} onkeydown={key} />
    <label class="chk"><input type="checkbox" bind:checked={search.case} onchange={() => void startSearch()} /> {t("cm-match-case")}</label>
    {#if search.running}<button type="button" class="btn" onclick={() => void cancelSearch()}>{t("search-abbrechen")}</button>{/if}
    <span class="info" role="status" aria-live="polite">
      {#if search.running}{t("search-laeuft", { n: search.hits.length })}
      {:else if search.done?.cancelled}{t("search-abgebrochen", { n: search.hits.length })}
      {:else if search.done?.truncated}{t("search-gekuerzt", { n: search.done.limit })}
      {:else if search.done}{search.hits.length === 0 ? t("search-nichts") : t("search-fertig", { n: search.hits.length, files: groups.length })}{/if}
    </span>
  </div>
  <ul>
    {#each groups as g (g.rel)}
      <li class="file mono">{g.rel}</li>
      {#each g.hits as h (h.rel + ":" + h.line)}
        <li><button type="button" class="row" onclick={() => void jumpTo(h.rel, h.line - 1, 0)}>
          <span class="ln mono">{h.line}</span>
          <span class="tx mono">{h.text.slice(0, h.from)}<mark>{h.text.slice(h.from, h.to)}</mark>{h.text.slice(h.to)}</span>
        </button></li>
      {/each}
    {/each}
  </ul>
</div>

<style>
  .s { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  .bar { display: flex; align-items: center; gap: 10px; padding: 6px 10px; }
  .bar input[type="search"] { flex: 1; max-width: 420px; padding: 4px 8px; border-radius: 6px; border: 1px solid var(--line2); background: var(--card); color: var(--text); font: inherit; font-size: 12.5px; }
  .bar input[type="search"]:focus-visible { outline: none; box-shadow: var(--focus); }
  .chk { font-size: 12px; color: var(--dim); display: flex; gap: 4px; align-items: center; }
  .info { font-size: 12px; color: var(--dim); }
  ul { list-style: none; margin: 0; padding: 0 0 14px; overflow: auto; flex: 1; }
  .file { padding: 4px 12px 1px; font-size: 11.5px; color: var(--dim); }
  .row { display: flex; gap: 10px; width: 100%; text-align: left; padding: 1px 12px; border: 0; background: none; color: var(--text); font-size: 12px; cursor: pointer; }
  .row:hover { background: var(--accentbg); }
  .row:focus-visible { outline: none; box-shadow: var(--focus); }
  .ln { flex: none; width: 44px; text-align: right; color: var(--faint); }
  .tx { white-space: pre; overflow: hidden; text-overflow: ellipsis; }
  mark { background: var(--accentbg); color: var(--text); outline: 1px solid var(--accent); border-radius: 2px; }
</style>
