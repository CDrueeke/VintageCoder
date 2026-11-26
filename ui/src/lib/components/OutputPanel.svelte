<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { jumpTo } from "../editor.svelte";
  import { popupMenu } from "../cm-lsp";
  import { bau, clearLog, logText } from "../bau.svelte";
  import { t } from "../i18n.svelte";
  import { fail, toast } from "../state.svelte";
  // Ausgabe: Protokoll von Bauen und Starten; Kopieren und Leeren
  let box = $state<HTMLElement | undefined>();
  $effect(() => { void bau.log.length; queueMicrotask(() => { if (box) box.scrollTop = box.scrollHeight; }); });
  async function copy() { try { await navigator.clipboard.writeText(logText()); toast(t("output-kopiert")); } catch (e) { fail(e); } }
</script>

<div class="out">
  <div class="bar"><button type="button" class="btn" onclick={copy} disabled={bau.log.length === 0}>{t("output-kopieren")}</button><button type="button" class="btn" onclick={clearLog} disabled={bau.log.length === 0}>{t("output-leeren")}</button></div>
  <div class="log mono" bind:this={box} role="log" aria-live="polite" oncontextmenu={(e) => { e.preventDefault(); popupMenu(null, e.clientX, e.clientY, t("dock-output"), [{ label: t("cmd-protokoll_kopieren"), run: () => void copy() }, { label: t("cmd-protokoll_leeren"), run: clearLog }]); }}>
    {#if bau.log.length === 0}<p class="none">{t("output-leer")}</p>{/if}
    {#each bau.log as l, i (i)}{#if l.file}<button type="button" class="l jump {l.kind}" onclick={() => void jumpTo(l.file!, l.line! - 1, 0)}>{l.text}</button>{:else}<div class="l {l.kind}">{l.text}</div>{/if}{/each}
  </div>
</div>

<style>
  .out { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  .bar { display: flex; gap: 8px; padding: 4px 10px; }
  .log { flex: 1; overflow: auto; padding: 2px 12px 14px; font-size: 12px; white-space: pre-wrap; }
  .l.error { color: var(--err); } .l.warn { color: var(--warn); } .l.ok { color: var(--ok); }
  .jump { display: block; width: 100%; text-align: left; border: 0; background: none; font: inherit; padding: 0; cursor: pointer; text-decoration: underline dotted; text-underline-offset: 3px; }
  .jump:hover { background: var(--accentbg); } .jump:focus-visible { outline: none; box-shadow: var(--focus); }
  .jump.info { color: var(--text); }
  .none { margin: 0; color: var(--dim); font-family: var(--font-ui); }
</style>
