<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { antwort, rf } from "../rueckfrage.svelte";
  import { t } from "../i18n.svelte";
  import Dialog from "./Dialog.svelte";
  // „Abbrechen“ steht zuerst und bekommt den Fokus; Esc und Schließen zählen als Nein
  let abbrechen = $state<HTMLButtonElement | null>(null);
  $effect(() => { if (!rf.open && rf.pending) antwort(false); });   // Schließen mit Esc zählt als Nein
  $effect(() => { if (rf.open && abbrechen) { const b = abbrechen; setTimeout(() => b.focus(), 0); } });   // Vorgabe Abbrechen, auch bei Folgeaufrufen
</script>

<Dialog bind:open={rf.open} title={rf.title}>
  {#snippet children()}<p class="rf" data-rf="text">{rf.text}</p>{/snippet}
  {#snippet footer()}
    <button type="button" class="btn" bind:this={abbrechen} data-rf="cancel" onclick={() => antwort(false)}>{t("kopierdlg-abbrechen")}</button>
    <button type="button" class="btn danger" data-rf="confirm" onclick={() => antwort(true)}>{rf.ok}</button>
  {/snippet}
</Dialog>

<style>
  .rf { margin: 0; max-width: 460px; line-height: 1.5; } .danger { border-color: var(--err, #c33); color: var(--err, #c33); }
</style>
