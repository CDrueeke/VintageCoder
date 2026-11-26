<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from "../i18n.svelte";
  import Dialog from "./Dialog.svelte";
  // „Alle Beispiele“: Suche über Titel, Beschreibung und System, gruppiert nach Zielsystem (Gruppen zugeklappt, bei einer Suche offen)
  export interface Example { name: string; system: string; title: string; description: string; path: string; readonly: boolean; featured: boolean; source?: string | null; image?: string | null }
  let { open = $bindable(false), examples, onopen, oncopy }: { open?: boolean; examples: Example[]; onopen: (e: Example) => void; oncopy: (e: Example) => void } = $props();
  let q = $state("");
  $effect(() => { if (open) q = ""; });
  const hits = $derived(examples.filter((e) => { const s = q.trim().toLowerCase(); return !s || `${e.title} ${e.description} ${e.system} ${e.name}`.toLowerCase().includes(s); }));
  const groups = $derived.by(() => { const m = new Map<string, Example[]>(); for (const e of hits) m.set(e.system, [...(m.get(e.system) ?? []), e]); return [...m.entries()]; });
</script>

<Dialog bind:open title={t("startui-beispielprojekte")} wide>
  {#snippet children()}
    <input class="inp" type="search" placeholder={t("start-beispiele-suche")} aria-label={t("start-beispiele-suche")} bind:value={q} />
    <div class="gr">
      {#each groups as [sys, list] (sys)}
        <details open={!!q.trim()}>
          <summary><b>{sys}</b> <span class="chip">{list.length}</span></summary>
          <ul class="plain">{#each list as e (e.path)}
            <li class="ex"><div class="grow"><b>{e.title}</b>{#if e.source} <span class="dim small" style="margin-left:6px" title={t("beispiele-herkunft", { paket: e.source })}>· {e.source}</span>{/if}<div class="dim small">{e.description}</div></div>
              {#if !e.readonly}<button type="button" class="btn sm" onclick={() => { open = false; onopen(e); }}>{t("startui-oeffnen")}</button>{/if}
              <button type="button" class="btn sm" onclick={() => { open = false; oncopy(e); }}>{t("startui-kopieren")}</button></li>
          {/each}</ul>
        </details>
      {:else}<p class="dim">{t("start-beispiele-keine")}</p>{/each}
    </div>
  {/snippet}
  {#snippet footer()}<button type="button" class="btn" onclick={() => (open = false)}>{t("infodlg-schliessen")}</button>{/snippet}
</Dialog>

<style>
  .inp { width: 100%; box-sizing: border-box; margin-bottom: 10px; }
  .gr { display: flex; flex-direction: column; gap: 6px; max-height: 55vh; overflow: auto; }
  details { border: 1px solid var(--line); border-radius: 8px; padding: 6px 10px; } summary { cursor: pointer; padding: 3px 0; }
  .plain { list-style: none; margin: 6px 0 2px; padding: 0; display: flex; flex-direction: column; gap: 8px; }
  .ex { display: flex; gap: 8px; align-items: center; } .grow { flex: 1; min-width: 0; }
  .chip { display: inline-block; padding: 0 7px; border-radius: 10px; border: 1px solid var(--line2); font-size: 11px; color: var(--dim); }
  .dim { color: var(--dim); } .small { font-size: 11.5px; }
</style>
