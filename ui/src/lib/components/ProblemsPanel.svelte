<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { jumpTo } from "../editor.svelte";
  import { t, msgText } from "../i18n.svelte";
  import { popupMenu } from "../cm-lsp";
  import { bau, problemLines } from "../bau.svelte";
  import { fail, toast } from "../state.svelte";
  import { lsp } from "../lsp.svelte";
  import { call } from "../api";
  import { onMount } from "svelte";
  // Probleme: alle Meldungen aller Dateien (Fehler, Warnungen, Hinweise), nach Datei gruppiert und nach Zeile; Klick springt zur Stelle.
  // Die Statuszeile zählt nur die Datei im Reiter; hier steht der Stand des ganzen Projekts.
  const groups = $derived(Object.entries(lsp.diags).map(([rel, items]) => ({ rel, items: items.filter((d) => d.severity <= 3).sort((a, b) => a.line - b.line) })).filter((g) => g.items.length > 0).sort((a, b) => a.rel.localeCompare(b.rel)));
  const isLong = (d: { code: string }) => d.code === "W203";   // zu weite Sprünge: je Datei zu einer Zeile zusammengefasst, aufklappbar
  let openLong = $state<Record<string, boolean>>({});
  const errFiles = $derived(groups.filter((g) => g.items.some((d) => d.severity === 1)).length);
  const errors = $derived(groups.reduce((n, g) => n + g.items.filter((d) => d.severity === 1).length, 0));
  const sevText = (n: number) => t(`diag-sev-${n}`);
  async function copy(text: string) { try { await navigator.clipboard.writeText(text); toast(t("problems-kopiert")); } catch (e) { fail(e); } }
  /** Kontextmenü eines Problems: Springen, Kopieren, Alle kopieren */
  function menu(e: MouseEvent, file: string, line: number, col: number, text: string, code = "", id = "") {
    e.preventDefault(); bau.sel = { file, line, text };
    popupMenu(null, e.clientX, e.clientY, t("problems-title-menu"), [
      { label: t("cmd-problem_springen"), run: () => { if (line > 0) void jumpTo(file, line - 1, col); } },
      ...(code && erkl(code) ? [{ label: t("basiccode-anzeigen", { code }), run: () => { offen[id] = !offen[id]; } }] : []),
      { label: t("cmd-problem_kopieren"), run: () => void copy(text) },
      { label: t("cmd-probleme_alle_kopieren"), run: () => void copy(problemLines(lsp.diags).join("\n")) },
    ]);
  }
  // Erklärtexte der BASIC-Codes (B150 bis B196): einmal vom Kern holen; Überfahren zeigt den Text, Klick auf den Code klappt ihn unter der Zeile auf.
  type CodeInfo = { kind: "error" | "warning" | "hint"; disable: boolean; text: string };
  let codes = $state<Record<string, CodeInfo>>({});
  let offen = $state<Record<string, boolean>>({});
  onMount(() => { void call("basic_codes").then((m) => (codes = m)).catch(() => {}); });
  const erkl = (code: string): CodeInfo | undefined => codes[code];
  const tip = (code: string): string | undefined => erkl(code)?.text.replace(/`/g, "");
  const teile = (text: string) => text.split("`").map((s, i) => ({ s, code: i % 2 === 1 }));
  const toggle = (e: Event, id: string) => { e.stopPropagation(); offen[id] = !offen[id]; };
  const total = $derived(groups.reduce((n, g) => n + g.items.filter((d) => !isLong(d)).length + (g.items.some(isLong) ? 1 : 0), 0));
</script>

{#snippet karte(code: string)}
  {@const k = erkl(code)}
  {#if k}
    <li class="erkl" data-testid="problem-erklaerung">
      <p>{#each teile(k.text) as p}{#if p.code}<code>{p.s}</code>{:else}{p.s}{/if}{/each}</p>
      <p class="art">{k.disable ? t("basiccode-abschaltbar") : t("basiccode-fest")}</p>
    </li>
  {/if}
{/snippet}

<div class="problems">
  {#if groups.length === 0 && bau.problems.length === 0}
    <p class="none">{t("problems-none")}</p>
  {:else}
    {#if bau.problems.length > 0}
      <p class="file mono">{t("problems-bau")} ({bau.problems.length})</p>
      <ul>
        {#each bau.problems as d, i (i)}
          <li><button type="button" class="row sev{d.severity === "error" ? 1 : d.severity === "warning" ? 2 : 3}" onclick={() => { bau.sel = { file: d.file, line: d.line, text: d.message }; if (d.line > 0) void jumpTo(d.file, d.line - 1, 0); }} oncontextmenu={(e) => menu(e, d.file, d.line, 0, `${d.file}:${d.line}: ${d.message}`, d.code, `b${i}`)}>
            <span class="sev">{t(`diag-sev-${d.severity === "error" ? 1 : d.severity === "warning" ? 2 : 3}`)}</span>{#if d.code}<span class="code mono" class:hasinfo={!!erkl(d.code)} title={tip(d.code)} data-testid="problem-code" onclick={(e) => erkl(d.code) && toggle(e, `b${i}`)} role="presentation">{d.code}</span>{/if}<span class="msg">{d.message}</span><span class="loc mono">{d.file}{d.line > 0 ? ` · ${t("problems-line", { line: d.line })}` : ""}</span>
          </button></li>
          {#if d.code && offen[`b${i}`] && erkl(d.code)}{@render karte(d.code)}{/if}
        {/each}
      </ul>
    {/if}
    {#if groups.length > 0}<p class="sum" role="status">{t("problems-projekt", { e: errors, f: errFiles, n: total, m: groups.length })}</p>
    <ul>
      {#each groups as g (g.rel)}
        {@const longs = g.items.filter(isLong)}
        <li class="file mono">{g.rel} ({g.items.length - longs.length + (longs.length ? 1 : 0)})</li>
        {#each g.items.filter((d) => !isLong(d)) as d, i (g.rel + ":" + i)}
          <li><button type="button" class="row sev{d.severity}" onclick={() => { bau.sel = { file: g.rel, line: d.line + 1, text: d.message }; void jumpTo(g.rel, d.line, d.from); }} oncontextmenu={(e) => menu(e, g.rel, d.line + 1, d.from, `${g.rel}:${d.line + 1}: ${sevText(d.severity)} ${msgText(d.key, d.args, d.message)}`, d.code, `${g.rel}:${i}`)}>
            <span class="sev">{t(`diag-sev-${d.severity}`)}</span>
            {#if d.code}<span class="code mono" class:hasinfo={!!erkl(d.code)} title={tip(d.code)} data-testid="problem-code" onclick={(e) => erkl(d.code) && toggle(e, `${g.rel}:${i}`)} role="presentation">{d.code}</span>{/if}<span class="msg">{msgText(d.key, d.args, d.message)}</span>
            <span class="loc mono">{t("problems-line", { line: d.line + 1 })}</span>
          </button></li>
          {#if d.code && offen[`${g.rel}:${i}`] && erkl(d.code)}{@render karte(d.code)}{/if}
        {/each}
        {#if longs.length}
          <li><button type="button" class="row sev3" aria-expanded={!!openLong[g.rel]} onclick={() => (openLong[g.rel] = !openLong[g.rel])}>
            <span class="sev">{openLong[g.rel] ? "▾" : "▸"} {t("diag-sev-3")}</span><span class="msg">{t("problems-spruenge", { n: longs.length })}</span><span class="loc mono">{t("problems-aufklappen")}</span>
          </button></li>
          {#if openLong[g.rel]}
            {#each longs as d, i (g.rel + ":l" + i)}
              <li><button type="button" class="row sev3 sub" onclick={() => { bau.sel = { file: g.rel, line: d.line + 1, text: d.message }; void jumpTo(g.rel, d.line, d.from); }}>
                <span class="sev"></span><span class="msg">{msgText(d.key, d.args, d.message)}</span><span class="loc mono">{t("problems-line", { line: d.line + 1 })}</span>
              </button></li>
            {/each}
          {/if}
        {/if}
      {/each}
    </ul>
    {/if}
  {/if}
</div>

<style>
  .problems { flex: 1; min-height: 0; overflow: auto; }
  ul { list-style: none; margin: 0; padding: 4px 0 14px; }
  .row { display: flex; gap: 10px; width: 100%; text-align: left; padding: 3px 12px; border: 0; background: none; color: var(--text); font: inherit; font-size: 12.5px; cursor: pointer; }
  .row:hover { background: var(--accentbg); }
  .row:focus-visible { outline: none; box-shadow: var(--focus); }
  .sev { flex: none; width: 64px; font-weight: 600; }
  .sev1 .sev { color: var(--err); }
  .sev2 .sev { color: var(--warn); }
  .sev3 .sev { color: var(--dim); }
  .erkl { margin: 0 12px 4px 86px; padding: 6px 10px; border-left: 3px solid var(--accent); background: var(--accentbg); font-size: 12px; }
  .erkl p { margin: 0 0 3px; } .erkl p:last-child { margin: 0; }
  .erkl .art { color: var(--dim); }
  .erkl code { font-family: var(--mono, monospace); }
  .hasinfo { cursor: help; text-decoration: underline dotted; }
  .code { color: var(--dim); font-size: 11px; margin-right: 6px; flex: none; }
  .msg { flex: 1; min-width: 0; }
  .loc { flex: none; color: var(--faint); }
  .row.sub .msg { color: var(--dim); }
  .sum { margin: 0; padding: 6px 12px 2px; font-size: 12px; color: var(--dim); }
  .file { padding: 4px 12px 1px; font-size: 11.5px; color: var(--dim); }
  .none { margin: 0; padding: 8px 12px; color: var(--dim); font-size: 12.5px; }
</style>
