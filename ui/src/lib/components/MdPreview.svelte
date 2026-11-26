<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  // Markdown-Vorschau (E13): Blockbaum aus dem Kern, als DOM-Knoten (nie rohes HTML). Links: Web im Systembrowser (nur auf Klick), Datei im Projekt öffnet im Editor, Anker springt. Aufgaben lassen sich abhaken.
  import { call, type MdInline, type MdNode, type MdRender, type MdZiel } from "../api";
  import { docRev, ed, jumpTo, openFile, stateOf } from "../editor.svelte";
  import { md, mdAufgabe } from "../markdown.svelte";
  import { fail, toast } from "../state.svelte";
  import { t } from "../i18n.svelte";
  let { rel, group = 0 }: { rel: string; group?: number } = $props();
  let r = $state<MdRender | null>(null); let wieviel = $state(md.blocks); let host: HTMLElement;
  async function laden() {
    const st = stateOf(rel); if (!st) return;
    try { r = await call("md_render", { rel, text: st.doc.toString(), from: 0, count: wieviel }); } catch (e) { fail(e); }
  }
  $effect(() => { void ed.nonces[group as 0 | 1]; void docRev.n; void rel; void wieviel; void laden(); });
  function ziel(z: MdZiel, e: Event) {
    e.preventDefault();
    if (z.kind === "file") { if (z.line) void jumpTo(z.rel, z.line - 1, 0); else void openFile(z.rel); }
    else if (z.kind === "anchor") { host.querySelector<HTMLElement>(`[id="${CSS.escape("md-" + z.anchor)}"]`)?.scrollIntoView({ block: "start" }); }
    else if (z.kind === "web") { void call("md_url_open", { url: z.url }).then(() => toast(t("md-link-geoeffnet", { url: z.url }))).catch(fail); }
  }
</script>

{#snippet inl(k: MdInline[])}
  {#each k as x, i (i)}
    {#if x.t === "text"}{x.s}
    {:else if x.t === "code"}<code>{x.s}</code>
    {:else if x.t === "em"}<em>{@render inl(x.k)}</em>
    {:else if x.t === "strong"}<strong>{@render inl(x.k)}</strong>
    {:else if x.t === "strike"}<s>{@render inl(x.k)}</s>
    {:else if x.t === "link"}
      {#if x.to.kind === "blocked"}<span class="gesperrt" title={x.to.why}>{@render inl(x.k)}</span>
      {:else}{@const z = x.to}<button type="button" class="hl" data-md="link" data-kind={z.kind} title={z.kind === "web" ? z.url : x.title} onclick={(e) => ziel(z, e)}>{@render inl(x.k)}</button>{/if}
    {:else if x.t === "image"}
      {#if x.img}<img src={x.img} alt={x.alt} data-md="bild" />{:else}<span class="bildfehlt" data-md="bild-fehlt" title={x.to.kind === "blocked" ? x.to.why : t("md-bild-fehlt")}>🖼 {x.alt || t("md-bild")}</span>{/if}
    {:else if x.t === "br"}<br />{:else if x.t === "soft"}{" "}{/if}
  {/each}
{/snippet}

{#snippet blk(n: MdNode)}
  {#if n.b === "heading"}<svelte:element this={`h${n.level}`} id={"md-" + n.anchor} data-md="h">{@render inl(n.k)}</svelte:element>
  {:else if n.b === "para"}<p>{@render inl(n.k)}</p>
  {:else if n.b === "quote"}<blockquote>{#each n.nodes as c, i (i)}{@render blk(c)}{/each}</blockquote>
  {:else if n.b === "list"}
    <svelte:element this={n.start !== null ? "ol" : "ul"} start={n.start ?? undefined} class:tasks={n.items.some((x) => x.task !== null)}>
      {#each n.items as it, i (i)}<li class:task={it.task !== null}>{#if it.task !== null}<input type="checkbox" data-md="aufgabe" checked={it.task} aria-label={t("md-aufgabe")} onchange={() => void mdAufgabe(rel, it.nodes[0]?.lines[0] ?? n.lines[0] + i)} />{/if}{#each it.nodes as c, j (j)}{#if c.b === "para"}<span>{@render inl(c.k)}</span>{:else}{@render blk(c)}{/if}{/each}</li>{/each}
    </svelte:element>
  {:else if n.b === "code"}<pre data-md="code"><code>{n.text}</code></pre>
  {:else if n.b === "table"}
    <div class="tw"><table data-md="tabelle"><thead><tr>{#each n.head as c, i (i)}<th style:text-align={n.align[i] || undefined}>{@render inl(c)}</th>{/each}</tr></thead>
      <tbody>{#each n.rows as row, ri (ri)}<tr>{#each row as c, i (i)}<td style:text-align={n.align[i] || undefined}>{@render inl(c)}</td>{/each}</tr>{/each}</tbody></table></div>
  {:else if n.b === "rule"}<hr />
  {:else if n.b === "html"}<pre class="html" data-md="html">{n.text}</pre>{/if}
{/snippet}

<div class="vorschau" bind:this={host} data-md="vorschau" role="document" aria-label={t("md-vorschau-label")}>
  {#if r}
    {#each r.nodes as n, i (i)}{@render blk(n)}{:else}<p class="dim">{t("md-leer")}</p>{/each}
    {#if r.total > wieviel}<button type="button" class="btn" data-md="mehr" onclick={() => (wieviel += md.blocks)}>{t("md-mehr", { n: r.total - wieviel })}</button>{/if}
  {/if}
</div>

<style>
  .vorschau { flex: 1; min-width: 0; overflow: auto; padding: 18px 26px 40px; color: var(--text); line-height: 1.6; font-size: 14px; max-width: 100%; }
  .vorschau :global(h1), .vorschau :global(h2), .vorschau :global(h3), .vorschau :global(h4) { line-height: 1.25; margin: 1.2em 0 0.5em; } .vorschau :global(h1) { font-size: 1.7em; border-bottom: 1px solid var(--line2); padding-bottom: 0.2em; } .vorschau :global(h2) { font-size: 1.35em; }
  p { margin: 0.6em 0; } code { font-family: var(--font-mono, monospace); background: var(--raise); border-radius: 4px; padding: 0 4px; font-size: 0.92em; }
  pre { background: var(--raise); border-radius: 8px; padding: 10px 12px; overflow: auto; } pre code { background: none; padding: 0; }
  blockquote { margin: 0.6em 0; padding: 0 12px; border-left: 3px solid var(--line2); color: var(--dim); } ul, ol { padding-left: 1.5em; } ul.tasks { list-style: none; padding-left: 0.2em; } li.task { display: flex; gap: 8px; align-items: baseline; } li.task input { flex: none; }
  .tw { overflow: auto; } table { border-collapse: collapse; margin: 0.6em 0; } th, td { border: 1px solid var(--line2); padding: 4px 10px; } th { background: var(--raise); }
  img { max-width: 100%; } .hl { color: var(--accent2); text-decoration: underline; background: none; border: 0; padding: 0; font: inherit; cursor: pointer; text-align: inherit; } .gesperrt { color: var(--dim); text-decoration: line-through dotted; } .bildfehlt { color: var(--dim); border: 1px dashed var(--line2); border-radius: 6px; padding: 0 6px; } hr { border: 0; border-top: 1px solid var(--line2); } .html { color: var(--dim); } .dim { color: var(--dim); }
</style>
