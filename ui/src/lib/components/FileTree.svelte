<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { TreeNode } from "../api";
  import { t } from "../i18n.svelte";
  import Icon from "./Icon.svelte";
  import { dragSource, dropTarget } from "../dnd.svelte";
  import { findNode } from "../editor.svelte";
  import { ui } from "../state.svelte";
  import FileTree from "./FileTree.svelte";
  // Dateibaum: role=tree, Pfeiltasten wechseln Zeilen, Rechts/Links klappen auf und zu, Eingabe öffnet, F2 benennt um, Entf löscht, Kontexttaste öffnet das Menü, Ziehen verschiebt in einen Ordner
  let { nodes, selected, onselect, depth = 0, open = $bindable({}), oncontext, onrename, ondelete, onmove }: { nodes: TreeNode[]; selected: string | null; onselect: (rel: string) => void; depth?: number; open?: Record<string, boolean>;
    oncontext?: (n: TreeNode, x: number, y: number) => void; onrename?: (n: TreeNode) => void; ondelete?: (n: TreeNode) => void; onmove?: (from: TreeNode, folder: string) => void } = $props();
  let over = $state<string | null>(null);
  function key(e: KeyboardEvent, n: TreeNode) {
    const items = [...(e.currentTarget as HTMLElement).closest('[role="tree"]')!.querySelectorAll<HTMLElement>('[role="treeitem"]')];
    const i = items.indexOf(e.currentTarget as HTMLElement);
    if (e.key === "ArrowDown") { e.preventDefault(); items[i + 1]?.focus(); }
    else if (e.key === "ArrowUp") { e.preventDefault(); items[i - 1]?.focus(); }
    else if (e.key === "ArrowRight" && n.kind === "dir") { e.preventDefault(); if (!open[n.rel]) open[n.rel] = true; else items[i + 1]?.focus(); }
    else if (e.key === "ArrowLeft" && n.kind === "dir" && open[n.rel]) { e.preventDefault(); open[n.rel] = false; }
    else if (e.key === "Enter" || e.key === " ") { e.preventDefault(); act(n); }
    else if (e.key === "F2") { e.preventDefault(); onrename?.(n); }
    else if (e.key === "Delete" || (e.key === "Backspace" && e.metaKey)) { e.preventDefault(); ondelete?.(n); }
    else if (e.key === "ContextMenu" || (e.key === "F10" && e.shiftKey)) { e.preventDefault(); const r = (e.currentTarget as HTMLElement).getBoundingClientRect(); oncontext?.(n, r.left + 24, r.bottom); }
  }
  function act(n: TreeNode) { if (n.kind === "dir") open[n.rel] = !open[n.rel]; else onselect(n.rel); }
</script>

<ul class={depth === 0 ? "tree" : ""} role={depth === 0 ? "tree" : "group"} aria-label={depth === 0 ? t("tree-label") : undefined}>
  {#each nodes as n (n.rel)}
    <li role="none">
      <div class="it" role="treeitem" tabindex={n.rel === selected || (selected === null && depth === 0 && n === nodes[0]) ? 0 : -1} aria-level={depth + 1} aria-selected={n.rel === selected} aria-expanded={n.kind === "dir" ? !!open[n.rel] : undefined} style={`padding-left:${10 + depth * 16}px`} class:over={over === n.rel} data-rel={n.rel}
        use:dragSource={() => ({ kind: "tree", id: n.rel, label: n.name, onend: () => { over = null; } })}
        use:dropTarget={() => ({ accept: (p) => p.kind === "tree" && n.kind === "dir" && p.id !== n.rel && !n.rel.startsWith(p.id + "/"), enter: () => { over = n.rel; }, leave: () => { if (over === n.rel) over = null; }, drop: (p) => { const d = findNode(ui.project?.tree ?? [], p.id); over = null; if (d) onmove?.(d, n.rel); } })}
        oncontextmenu={(e) => { e.preventDefault(); oncontext?.(n, e.clientX, e.clientY); }} onclick={() => act(n)} onkeydown={(e) => key(e, n)}>
        {#if n.kind === "dir"}<Icon name={open[n.rel] ? "chevd" : "chevr"} size={14} /><Icon name="folder" size={15} />{:else}<span style="width:14px"></span><Icon name="file" size={15} />{/if}
        <span>{n.name}</span>{#if n.mark === "turbo-disk"}<span class="tag mark" title={t("turboui-marke-hinweis")} data-mark="turbo-disk">{t("turboui-marke-diskette")}</span>{:else if n.tag}<span class="tag">{n.tag}</span>{/if}
      </div>
      {#if n.kind === "dir" && open[n.rel] && n.children}<FileTree nodes={n.children} {selected} {onselect} depth={depth + 1} bind:open {oncontext} {onrename} {ondelete} {onmove} />{/if}
    </li>
  {/each}
</ul>
