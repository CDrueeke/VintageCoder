<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { dropTarget } from "../dnd.svelte";
  import type { TreeNode } from "../api";
  import { copyPath, deleteAsk, findNode, moveInto, newDirIn, newFileIn, newNoteIn, openFile, parentOf, reloadTree, renameAsk, revealIn, treeUndo } from "../editor.svelte";
  import { runCommand } from "../commands";
  import { t } from "../i18n.svelte";
  import { ui } from "../state.svelte";
  import FileTree from "./FileTree.svelte";
  import AreaButton from "./AreaButton.svelte";
  import Icon from "./Icon.svelte";
  import Tooltip from "./Tooltip.svelte";
  import { alsInclude, includeGrund } from "../include.svelte";
  // Projektbaum mit Dateiverwaltung: Knöpfe „Neue Datei/Ordner“, Kontextmenü, Ziehen in Ordner oder in die Wurzel
  let { open = $bindable({}) }: { open?: Record<string, boolean> } = $props();
  let menu = $state<{ x: number; y: number; node: TreeNode | null } | null>(null);
  $effect(() => { const zu = () => { menu = null; }; window.addEventListener("vc:closemenus", zu); return () => window.removeEventListener("vc:closemenus", zu); });
  let rootOver = $state(false);
  let menuEl = $state<HTMLElement | undefined>();
  let filter = $state("");
  let showFilter = $state(false);
  // Zustand der Ordner je Projekt merken
  let restored: string | null = null;
  $effect(() => {
    const root = ui.project?.root ?? null; if (root === restored) return; restored = root;
    for (const k of Object.keys(open)) delete open[k];
    try { const s = root ? localStorage.getItem(`tree:${root}`) : null; if (s) Object.assign(open, JSON.parse(s)); } catch { /* ohne Speicher weiter */ }
  });
  $effect(() => { const root = ui.project?.root; const s = JSON.stringify(open); if (root && root === restored) { try { localStorage.setItem(`tree:${root}`, s); } catch { /* weiter */ } } });
  function filtered(nodes: TreeNode[], q: string): TreeNode[] {
    const out: TreeNode[] = [];
    for (const n of nodes) {
      if (n.kind === "dir") { const c = filtered(n.children ?? [], q); if (c.length || n.name.toLowerCase().includes(q)) out.push({ ...n, children: c }); }
      else if (n.name.toLowerCase().includes(q)) out.push(n);
    }
    return out;
  }
  const shown = $derived(ui.project ? (filter.trim() ? filtered(ui.project.tree, filter.trim().toLowerCase()) : ui.project.tree) : []);
  function dirs(nodes: TreeNode[], acc: Record<string, boolean> = {}): Record<string, boolean> { for (const n of nodes) if (n.kind === "dir") { acc[n.rel] = true; dirs(n.children ?? [], acc); } return acc; }
  const effOpen = $derived(filter.trim() ? dirs(shown) : open);
  function collapseAll() { for (const k of Object.keys(open)) delete open[k]; }
  // Gewählte Datei sichtbar machen: übergeordnete Ordner aufklappen
  $effect(() => { const s = ui.selected; if (!s) return; const parts = s.split("/").slice(0, -1); let acc = ""; for (const p of parts) { acc = acc ? `${acc}/${p}` : p; if (!open[acc]) open[acc] = true; } });

  /** Zielordner für „Neu“: gewählter Ordner, sonst Ordner der gewählten Datei, sonst Wurzel */
  export function targetFolder(): string {
    const sel = ui.selected; if (!sel || !ui.project) return "";
    const n = findNode(ui.project.tree, sel); return n?.kind === "dir" ? n.rel : parentOf(sel);
  }
  function show(n: TreeNode | null, x: number, y: number) { menu = { x, y, node: n }; queueMicrotask(() => menuEl?.querySelector<HTMLElement>('[role="menuitem"]')?.focus()); }
  function folderOf(n: TreeNode | null) { return n ? (n.kind === "dir" ? n.rel : parentOf(n.rel)) : ""; }
  function run(f: () => void) { f(); menu = null; }
  function key(e: KeyboardEvent) {
    const items = [...(menuEl?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])]; const i = items.indexOf(document.activeElement as HTMLElement);
    if (e.key === "ArrowDown") { e.preventDefault(); items[(i + 1) % items.length]?.focus(); } else if (e.key === "ArrowUp") { e.preventDefault(); items[(i - 1 + items.length) % items.length]?.focus(); }
    else if (e.key === "Escape") { e.preventDefault(); menu = null; }
  }
</script>

<svelte:window onpointerdown={(e) => { if (menu && !menuEl?.contains(e.target as Node)) menu = null; }} />

<div class="h"><span>{t("tree-heading")}</span><span style="flex:1"></span>
  <Tooltip text={t("tree-filter-label")}><button type="button" class="ibtn sm" aria-label={t("tree-filter-label")} aria-pressed={showFilter} onclick={() => { showFilter = !showFilter; if (!showFilter) filter = ""; }}><Icon name="search" size={15} /></button></Tooltip>
  <Tooltip text={t("tree-neue-datei")}><button type="button" class="ibtn sm" aria-label={t("tree-neue-datei")} onclick={() => newFileIn(targetFolder())}><Icon name="plus" size={15} /></button></Tooltip>
  <Tooltip text={t("tree-neuer-ordner")}><button type="button" class="ibtn sm" aria-label={t("tree-neuer-ordner")} onclick={() => newDirIn(targetFolder())}><Icon name="newfolder" size={15} /></button></Tooltip>
  <Tooltip text={t("cmd-projekt_schliessen")}><button type="button" class="ibtn sm" aria-label={t("cmd-projekt_schliessen")} onclick={() => void runCommand("projekt_schliessen")}><Icon name="close" size={15} /></button></Tooltip>
<AreaButton area="seitenleiste" /></div>
{#if showFilter}<div style="padding:0 10px 6px"><input class="fi" type="search" bind:value={filter} placeholder={t("tree-filter-label")} aria-label={t("tree-filter-label")} autocomplete="off" spellcheck="false" onkeydown={(e) => { if (e.key === "Escape") { filter = ""; showFilter = false; } }} /></div>{/if}
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="treebox" class:over={rootOver}
  use:dropTarget={() => ({ accept: (p) => p.kind === "tree", enter: () => (rootOver = true), leave: () => (rootOver = false), drop: (p) => { rootOver = false; const n = ui.project ? findNode(ui.project.tree, p.id) : null; if (n) void moveInto(n.rel, n.kind, ""); } })}
  oncontextmenu={(e) => { if (e.target === e.currentTarget) { e.preventDefault(); show(null, e.clientX, e.clientY); } }}>
  {#if ui.project}
    {#if shown.length}
      <FileTree nodes={shown} selected={ui.selected} onselect={(rel) => { ui.selected = rel; void openFile(rel); }} open={effOpen}
        oncontext={(n, x, y) => show(n, x, y)} onrename={(n) => renameAsk(n.rel, n.kind)} ondelete={(n) => { if (n.kind === "file") void deleteAsk(n.rel); }}
        onmove={(n, folder) => void moveInto(n.rel, n.kind, folder)} />
    {:else}<p style="padding:8px 14px;color:var(--faint)">{filter.trim() ? t("tree-filter-keine") : t("tree-empty")}</p>{/if}
  {/if}
</div>

{#if menu}
  {@const node = menu?.node ?? null}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="ctx" role="menu" tabindex="-1" aria-label={t("tree-menu")} bind:this={menuEl} oncontextmenu={(e) => { e.preventDefault(); e.stopPropagation(); }} style={`left:${Math.min(menu?.x ?? 0, window.innerWidth - 230)}px;top:${Math.min(menu?.y ?? 0, window.innerHeight - 190)}px`} onkeydown={key}>
    {#if node?.kind === "file"}<button type="button" role="menuitem" onclick={() => run(() => void openFile(node!.rel))}>{t("tree-oeffnen")}</button>{/if}
    <button type="button" role="menuitem" onclick={() => run(() => newFileIn(folderOf(node)))}>{t("tree-neue-datei")}</button>
    <button type="button" role="menuitem" onclick={() => run(() => newNoteIn(folderOf(node)))}>{t("tree-neue-notiz")}</button>
    <button type="button" role="menuitem" onclick={() => run(() => newDirIn(folderOf(node)))}>{t("tree-neuer-ordner")}</button>
    {#if node}
      <button type="button" role="menuitem" onclick={() => run(() => renameAsk(node!.rel, node!.kind))}>{t("tree-umbenennen")}</button>
      {#if node.kind === "file"}<button type="button" role="menuitem" onclick={() => run(() => void deleteAsk(node!.rel))}>{t("tree-loeschen")}</button>{/if}
      {#if node.kind === "file"}{@const grund = includeGrund(node.rel, true)}<button type="button" role="menuitem" aria-disabled={grund ? "true" : undefined} class:grau={!!grund} title={grund ?? undefined} onclick={() => { if (!grund) run(() => alsInclude(node!.rel, true)); }}>{t("tree-als-include")}</button>{/if}
      <button type="button" role="menuitem" onclick={() => run(() => void copyPath(node!.rel))}>{t("tree-pfad-kopieren")}</button>
      <button type="button" role="menuitem" onclick={() => run(() => void revealIn(node!.rel))}>{t("tree-im-ordner-zeigen")}</button>
    {:else}
      <button type="button" role="menuitem" onclick={() => run(collapseAll)}>{t("tree-alle-zuklappen")}</button>
      <button type="button" role="menuitem" onclick={() => run(() => void reloadTree())}>{t("tree-neu-einlesen")}</button>
      <button type="button" role="menuitem" onclick={() => run(() => void treeUndo())}>{t("tree-rueckgaengig")}</button>
    {/if}
  </div>
{/if}

<style>
  .treebox { flex: 1; min-height: 0; overflow: auto; border-radius: 8px; }
  .treebox.over { box-shadow: inset 0 0 0 2px var(--accent); }
  .ctx { position: fixed; z-index: 1000; min-width: 200px; padding: 5px; background: var(--card); border: 1px solid var(--line2); border-radius: 10px; box-shadow: var(--shadow); display: flex; flex-direction: column; }
  .ctx button { text-align: left; padding: 7px 10px; border: 0; border-radius: 6px; background: none; color: var(--text); font: inherit; font-size: 13px; cursor: pointer; }
  .ctx button.grau { color: var(--muted, var(--text)); opacity: .5; cursor: default; }
  .ctx button:hover, .ctx button:focus-visible { background: var(--hover); outline: none; }
  :global(.tree .it.over) { box-shadow: inset 0 0 0 2px var(--accent); }
  .fi { width: 100%; height: 28px; padding: 0 8px; border-radius: 7px; border: 1px solid var(--line2); background: var(--raise); color: var(--text); font: inherit; font-size: 12.5px; }
  .fi:focus-visible { box-shadow: var(--focus); outline: none; }
  :global(.ibtn.sm) { width: 26px; height: 26px; }
</style>
