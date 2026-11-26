<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { LspSymbol } from "../api";
  import { call } from "../api";
  import { ed, jumpTo } from "../editor.svelte";
  import { bi } from "../basicinfo.svelte";
  import { t } from "../i18n.svelte";
  import { lsp, lspFlush, lspHas } from "../lsp.svelte";
  import { fail } from "../state.svelte";
  // Gliederung: Marken, Konstanten und Makros der Datei im Reiter; Klick springt dorthin
  let items = $state<LspSymbol[]>([]);
  let seq = 0;
  async function load(rel: string | null) {
    const my = ++seq;
    // natives BASIC: Marken und #define vom Kern (basic_info), kein Sprachdienst
    if (rel && bi.byRel[rel]) { items = bi.byRel[rel].outline; return; }
    if (!rel || !lspHas(rel)) { items = []; return; }
    try { await lspFlush(rel); const r = await call("lsp_outline", { rel }); if (my === seq) items = r; } catch (e) { if (my === seq) { items = []; fail(e); } }
  }
  // neu laden bei Reiterwechsel und wenn der Kern den Text neu geprüft hat
  $effect(() => { const rel = ed.active; void (rel ? lsp.diags[rel] : null); void (rel ? bi.byRel[rel] : null); void load(rel); });
</script>

<div class="o">
  {#if items.length === 0}
    <p class="none">{t("outline-leer")}</p>
  {:else}
    <ul>
      {#each items as s, i (i)}
        <li><button type="button" class="row" onclick={() => ed.active && void jumpTo(ed.active, s.line, 0)}>
          <span class="k k-{s.kind}">{t(`outline-${s.kind}`)}</span><span class="nm mono">{s.name}</span><span class="ln mono">{t("problems-line", { line: s.line + 1 })}</span>
        </button></li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .o { flex: 1; min-height: 0; overflow: auto; }
  ul { list-style: none; margin: 0; padding: 4px 0 14px; }
  .row { display: flex; gap: 10px; width: 100%; text-align: left; padding: 2px 12px; border: 0; background: none; color: var(--text); font-size: 12.5px; cursor: pointer; }
  .row:hover { background: var(--accentbg); }
  .row:focus-visible { outline: none; box-shadow: var(--focus); }
  .k { flex: none; width: 64px; color: var(--dim); font-size: 11.5px; }
  .k-label { color: var(--tok-label); } .k-macro { color: var(--tok-directive); } .k-const { color: var(--tok-number); }
  .nm { flex: 1; min-width: 0; }
  .ln { flex: none; color: var(--faint); }
  .none { margin: 0; padding: 8px 12px; color: var(--dim); font-size: 12.5px; }
</style>
