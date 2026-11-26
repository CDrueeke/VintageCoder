<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { newDisk } from "../disk.svelte";
  import { t } from "../i18n.svelte";
  import Dialog from "./Dialog.svelte";
  import Segment from "./Segment.svelte";
  // Neue, leere Diskette: Format, Name und Kennung; sie bleibt „ungespeichert“, bis man sie speichert
  let { open = $bindable(false) }: { open?: boolean } = $props();
  let format = $state("d64"); let name = $state("DISKETTE"); let id = $state("01");
  const atari = $derived(format === "atr");
  const bad = $derived(!atari && (!name.trim() || name.length > 16));
  async function make() { if (bad) return; open = false; await newDisk(format, name.trim().toUpperCase(), id); }
</script>

<Dialog bind:open title={t("disk-neu-titel")}>
  {#snippet children()}
    <div class="f">
      <div><span class="l">{t("disk-neu-format")}</span><Segment small label={t("disk-neu-format")} items={[{ id: "d64", label: "D64" }, { id: "d71", label: "D71" }, { id: "d81", label: "D81" }, { id: "atr", label: "ATR" }]} value={format} onselect={(v) => (format = v)} /></div>
      {#if !atari}
        <label><span class="l">{t("disk-neu-name")}</span><input class="inp" bind:value={name} maxlength="16" aria-invalid={bad} onkeydown={(e) => { if (e.key === "Enter") void make(); }} /></label>
        <label><span class="l">{t("disk-neu-id")}</span><input class="inp" bind:value={id} maxlength="2" onkeydown={(e) => { if (e.key === "Enter") void make(); }} /></label>
      {/if}
    </div>
  {/snippet}
  {#snippet footer()}
    <button type="button" class="btn" onclick={() => (open = false)}>{t("dialog-abbrechen")}</button>
    <button type="button" class="btn pri" disabled={bad} onclick={() => void make()}>{t("disk-neu-anlegen")}</button>
  {/snippet}
</Dialog>

<style>
  .f { display: flex; flex-direction: column; gap: 12px; min-width: 300px; } .l { display: block; font-size: 11.5px; color: var(--dim); margin-bottom: 4px; } .inp { width: 100%; }
</style>
