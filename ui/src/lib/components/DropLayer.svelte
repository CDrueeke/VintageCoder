<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import { answerDrop, drop } from "../ablegen.svelte";
  import { t } from "../i18n.svelte";
  // Hinweisfläche beim Ziehen von Dateien über das Fenster und die Rückfrage „Ins Projekt kopieren und öffnen?“
  let open = $state(false);
  $effect(() => { open = !!drop.ask; });
  $effect(() => { if (!open && drop.ask) void answerDrop(false); });
</script>

{#if drop.over}<div class="dz" role="status" aria-live="polite"><div class="box">{t("ablegen-hier")}</div></div>{/if}
{#if drop.ask}
  <Dialog bind:open title={t("ablegen-kopieren-titel")}>
    {#snippet children()}
      <p>{t("ablegen-kopieren-text", { n: drop.ask!.items.length })}</p>
      <ul class="names">{#each drop.ask!.items as it (it.path)}<li class="mono">{it.name}</li>{/each}</ul>
    {/snippet}
    {#snippet footer()}
      <button type="button" class="btn" data-drop="no" onclick={() => void answerDrop(false)}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn pri" data-drop="yes" onclick={() => void answerDrop(true)}>{t("ablegen-kopieren-ja")}</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .dz { position: fixed; inset: 0; z-index: 9000; display: flex; align-items: center; justify-content: center; background: rgb(0 0 0 / 0.35); pointer-events: none; }
  .box { padding: 22px 34px; border-radius: 16px; background: var(--card); color: var(--text); border: 2px dashed var(--accent); font-size: 16px; }
  .names { margin: 6px 0 0; padding-left: 18px; font-size: 12px; max-height: 160px; overflow: auto; }
</style>
